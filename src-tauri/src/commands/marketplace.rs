use crate::db::Database;
use crate::sync::resolve_username;
use std::fs;
use std::io::{Cursor, Read};
use std::path::Path;
use tauri::State;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct InstalledSkill {
    pub id: String,
    pub skill_id: String,
    pub skill_name: String,
    pub slug: String,
    pub repo_owner: String,
    pub repo_name: String,
    pub remote_version: String,
    pub installed_at: String,
    pub updated_at: String,
}

#[tauri::command]
pub fn install_marketplace_skill(
    db: State<Database>,
    download_url: String,
    name: String,
    slug: String,
    repo_owner: String,
    repo_name: String,
    version: String,
) -> Result<InstalledSkill, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    // Download zip from URL
    let response = ureq::get(&download_url)
        .call()
        .map_err(|e| format!("Download failed: {}", e))?;

    let mut zip_bytes: Vec<u8> = Vec::new();
    response
        .into_body()
        .into_reader()
        .read_to_end(&mut zip_bytes)
        .map_err(|e| format!("Read failed: {}", e))?;

    // Create skill in database
    let skill_id = uuid::Uuid::new_v4().to_string();
    let description = format!("从市场安装: {}/{}", repo_owner, repo_name);

    conn.execute(
        "INSERT INTO skills (id, name, slug, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![skill_id, name, slug, description, now, now],
    )
    .map_err(|e| format!("Failed to create skill: {}", e))?;

    // Extract zip and write files
    let cursor = Cursor::new(zip_bytes.clone());
    let mut archive =
        zip::ZipArchive::new(cursor).map_err(|e| format!("Failed to open zip: {}", e))?;

    // Determine the base path inside the zip (first directory, or root)
    let base_dir = find_base_dir(&mut archive);
    let mut has_md = false;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("Zip entry error: {}", e))?;

        if entry.is_dir() {
            continue;
        }

        let entry_path = entry.name().to_string();
        let relative = if let Some(ref base) = base_dir {
            if let Some(stripped) = entry_path.strip_prefix(base) {
                stripped.to_string()
            } else {
                continue;
            }
        } else {
            let parts: Vec<&str> = entry_path.split(&['/', '\\'][..]).collect();
            if parts.len() > 1 { parts[1..].join("/") } else { entry_path.clone() }
        };

        if relative.is_empty() {
            continue;
        }

        if relative == "SKILL.md" {
            has_md = true;
        }

        let mut content = Vec::new();
        entry
            .read_to_end(&mut content)
            .map_err(|e| format!("Read entry failed: {}", e))?;

        let file_size = content.len() as i64;
        let file_name = Path::new(&relative)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let file_id = uuid::Uuid::new_v4().to_string();
        let hash = crate::sync::hash_content(&content);

        conn.execute(
            "INSERT INTO skill_files (id, skill_id, relative_path, file_name, content_hash, file_size, is_directory, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7)",
            rusqlite::params![file_id, skill_id, relative, file_name, hash, file_size, now],
        )
        .ok();
    }

    // If no SKILL.md in zip, generate one
    if !has_md {
        let md_content = format!(
            "---\nname: \"{}\"\ndescription: \"从市场安装\"\n---\n\n# {}\n\n> 来源: {}/{}\n",
            name, name, repo_owner, repo_name
        );
        let file_id = uuid::Uuid::new_v4().to_string();
        let hash = crate::sync::hash_content(md_content.as_bytes());
        conn.execute(
            "INSERT INTO skill_files (id, skill_id, relative_path, file_name, content_hash, file_size, is_directory, updated_at)
             VALUES (?1, ?2, 'SKILL.md', 'SKILL.md', ?3, ?4, 0, ?5)",
            rusqlite::params![file_id, skill_id, hash, md_content.len() as i64, now],
        )
        .ok();
    }

    // Copy files to all active agent directories
    let mut astmt = conn
        .prepare("SELECT id, skills_path FROM agents WHERE is_active = 1")
        .map_err(|e| e.to_string())?;
    let agent_data: Vec<(String, String)> = astmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    // Re-read files from DB to write to agent dirs
    let mut fstmt = conn
        .prepare("SELECT relative_path FROM skill_files WHERE skill_id = ?1")
        .map_err(|e| e.to_string())?;
    let db_files: Vec<(String, Vec<u8>)> = fstmt
        .query_map([&skill_id], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .filter_map(|rel| {
            // Read file from zip again
            let cursor = Cursor::new(zip_bytes.clone());
            let mut archive = zip::ZipArchive::new(cursor).ok()?;
            let base_dir = find_base_dir(&mut archive);

            for i in 0..archive.len() {
                let mut entry = archive.by_index(i).ok()?;
                if entry.is_dir() {
                    continue;
                }
                let entry_path = entry.name().to_string();
                let candidate = if let Some(ref base) = base_dir {
                    entry_path.strip_prefix(base)?.to_string()
                } else {
                    entry_path
                };
                if candidate == rel {
                    let mut content = Vec::new();
                    entry.read_to_end(&mut content).ok()?;
                    return Some((rel, content));
                }
            }
            None
        })
        .collect();

    for (_, raw_path) in &agent_data {
        let dir = Path::new(&resolve_username(raw_path)).join(&slug);
        if let Err(e) = fs::create_dir_all(&dir) {
            log::warn!("Failed to create dir {}: {}", dir.display(), e);
            continue;
        }

        // Write SKILL.md content
        let md_content = format!(
            "---\nname: \"{}\"\ndescription: \"从市场安装\"\n---\n\n# {}\n\n> 来源: {}/{}\n",
            name, name, repo_owner, repo_name
        );
        fs::write(dir.join("SKILL.md"), &md_content).ok();

        for (relative, content) in &db_files {
            let target = dir.join(relative);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).ok();
            }
            fs::write(&target, content).ok();
        }
    }

    // Link to all active agents
    for (agent_id, _) in &agent_data {
        conn.execute(
            "INSERT INTO agent_skills (agent_id, skill_id, sync_status, synced_at) VALUES (?1, ?2, 'pending', ?3)",
            rusqlite::params![agent_id, skill_id, now],
        )
        .ok();
    }

    // Record installation
    let install_id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO installed_skills (id, skill_id, repo_owner, repo_name, remote_version, installed_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        rusqlite::params![install_id, skill_id, repo_owner, repo_name, version, now, now],
    )
    .map_err(|e| format!("Failed to record installation: {}", e))?;

    conn.execute(
        "INSERT INTO activities (type, message, created_at) VALUES ('create', ?1, ?2)",
        rusqlite::params![format!("从市场安装 Skill: {}", name), now],
    )
    .ok();

    Ok(InstalledSkill {
        id: install_id,
        skill_id,
        skill_name: name,
        slug,
        repo_owner,
        repo_name,
        remote_version: version,
        installed_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn get_installed_skills(db: State<Database>) -> Result<Vec<InstalledSkill>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT i.id, i.skill_id, s.name, s.slug, i.repo_owner, i.repo_name, i.remote_version, i.installed_at, i.updated_at
             FROM installed_skills i
             JOIN skills s ON s.id = i.skill_id
             ORDER BY i.installed_at DESC",
        )
        .map_err(|e| e.to_string())?;

    let results = stmt
        .query_map([], |row| {
            Ok(InstalledSkill {
                id: row.get(0)?,
                skill_id: row.get(1)?,
                skill_name: row.get(2)?,
                slug: row.get(3)?,
                repo_owner: row.get(4)?,
                repo_name: row.get(5)?,
                remote_version: row.get(6)?,
                installed_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(results)
}

#[tauri::command]
pub fn uninstall_installed_skill(db: State<Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let skill_id: String = conn
        .query_row(
            "SELECT skill_id FROM installed_skills WHERE id = ?1",
            [&id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Installed skill not found: {}", e))?;

    crate::sync::delete_skill_cascade(&conn, &skill_id)?;

    Ok(())
}

#[tauri::command]
pub fn update_installed_skill(
    db: State<Database>,
    id: String,
    download_url: String,
    new_version: String,
) -> Result<InstalledSkill, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let (skill_id, repo_owner, repo_name): (String, String, String) = conn
        .query_row(
            "SELECT skill_id, repo_owner, repo_name FROM installed_skills WHERE id = ?1",
            [&id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(|e| format!("Installed skill not found: {}", e))?;

    let (name, slug): (String, String) = conn
        .query_row("SELECT name, slug FROM skills WHERE id = ?1", [&skill_id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .map_err(|e| format!("Skill not found: {}", e))?;

    // Download new version
    let response = ureq::get(&download_url)
        .call()
        .map_err(|e| format!("Download failed: {}", e))?;
    let mut zip_bytes: Vec<u8> = Vec::new();
    response
        .into_body()
        .into_reader()
        .read_to_end(&mut zip_bytes)
        .map_err(|e| format!("Read failed: {}", e))?;

    // Clean old files
    conn.execute("DELETE FROM skill_files WHERE skill_id = ?1", [&skill_id])
        .map_err(|e| e.to_string())?;

    // Extract new files
    let cursor = Cursor::new(zip_bytes.clone());
    let mut archive =
        zip::ZipArchive::new(cursor).map_err(|e| format!("Failed to open zip: {}", e))?;
    let base_dir = find_base_dir(&mut archive);
    let mut has_md = false;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("Zip entry error: {}", e))?;
        if entry.is_dir() {
            continue;
        }
        let entry_path = entry.name().to_string();
        let relative = if let Some(ref base) = base_dir {
            if let Some(stripped) = entry_path.strip_prefix(base) {
                stripped.to_string()
            } else {
                continue;
            }
        } else {
            if entry_path.contains('/') || entry_path.contains('\\') {
                let parts: Vec<&str> = entry_path.split(&['/', '\\'][..]).collect();
                parts[1..].join("/")
            } else {
                entry_path.clone()
            }
        };
        if relative.is_empty() {
            continue;
        }
        if relative == "SKILL.md" {
            has_md = true;
        }

        let mut content = Vec::new();
        entry
            .read_to_end(&mut content)
            .map_err(|e| format!("Read entry failed: {}", e))?;
        let file_id = uuid::Uuid::new_v4().to_string();
        let hash = crate::sync::hash_content(&content);
        let file_size = content.len() as i64;
        let file_name = Path::new(&relative)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        conn.execute(
            "INSERT INTO skill_files (id, skill_id, relative_path, file_name, content_hash, file_size, is_directory, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7)",
            rusqlite::params![file_id, skill_id, relative, file_name, hash, file_size, now],
        )
        .ok();
    }

    if !has_md {
        let md_content = format!(
            "---\nname: \"{}\"\ndescription: \"从市场安装\"\n---\n\n# {}\n\n> 来源: {}/{}\n",
            name, name, repo_owner, repo_name
        );
        let file_id = uuid::Uuid::new_v4().to_string();
        let hash = crate::sync::hash_content(md_content.as_bytes());
        conn.execute(
            "INSERT INTO skill_files (id, skill_id, relative_path, file_name, content_hash, file_size, is_directory, updated_at)
             VALUES (?1, ?2, 'SKILL.md', 'SKILL.md', ?3, ?4, 0, ?5)",
            rusqlite::params![file_id, skill_id, hash, md_content.len() as i64, now],
        )
        .ok();
    }

    // Update agent directories
    let mut astmt = conn
        .prepare("SELECT skills_path FROM agents WHERE is_active = 1")
        .map_err(|e| e.to_string())?;
    let paths: Vec<String> = astmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    for raw_path in &paths {
        let dir = Path::new(&resolve_username(raw_path)).join(&slug);
        if dir.exists() {
            fs::remove_dir_all(&dir).ok();
        }
        fs::create_dir_all(&dir).ok();
    }

    let mut fstmt = conn
        .prepare("SELECT relative_path FROM skill_files WHERE skill_id = ?1")
        .map_err(|e| e.to_string())?;
    let relative_paths: Vec<String> = fstmt
        .query_map([&skill_id], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let cursor = Cursor::new(zip_bytes.clone());
    let mut archive =
        zip::ZipArchive::new(cursor).map_err(|e| format!("Failed to open zip: {}", e))?;
    let base_dir = find_base_dir(&mut archive);

    for relative in &relative_paths {
        if relative == "SKILL.md" {
            continue;
        }
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
            if entry.is_dir() {
                continue;
            }
            let entry_path = entry.name().to_string();
            let candidate = if let Some(ref base) = base_dir {
                entry_path.strip_prefix(base).unwrap_or(&entry_path).to_string()
            } else {
                entry_path
            };
            if &candidate == relative {
                let mut content = Vec::new();
                entry.read_to_end(&mut content).map_err(|e| e.to_string())?;
                for raw_path in &paths {
                    let target = Path::new(&resolve_username(raw_path)).join(&slug).join(relative);
                    if let Some(parent) = target.parent() {
                        fs::create_dir_all(parent).ok();
                    }
                    fs::write(&target, &content).ok();
                }
                break;
            }
        }
    }

    // Write SKILL.md
    let md_content = format!(
        "---\nname: \"{}\"\ndescription: \"从市场安装\"\n---\n\n# {}\n\n> 来源: {}/{}\n",
        name, name, repo_owner, repo_name
    );
    for raw_path in &paths {
        let target = Path::new(&resolve_username(raw_path)).join(&slug).join("SKILL.md");
        fs::write(&target, &md_content).ok();
    }

    // Update version
    conn.execute(
        "UPDATE installed_skills SET remote_version = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![new_version, now, &id],
    )
    .map_err(|e| e.to_string())?;

    conn.execute(
        "UPDATE skills SET updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, &skill_id],
    )
    .ok();

    conn.execute(
        "INSERT INTO activities (type, message, created_at) VALUES ('update', ?1, ?2)",
        rusqlite::params![format!("更新市场 Skill: {}", name), now],
    )
    .ok();

    Ok(InstalledSkill {
        id,
        skill_id,
        skill_name: name,
        slug,
        repo_owner,
        repo_name,
        remote_version: new_version,
        installed_at: String::new(),
        updated_at: now,
    })
}

fn find_base_dir(archive: &mut zip::ZipArchive<Cursor<Vec<u8>>>) -> Option<String> {
    for i in 0..archive.len() {
        let entry = archive.by_index(i).ok()?;
        let name = entry.name().to_string();
        if name.contains('/') {
            let first_dir = name.split('/').next()?.to_string();
            if !first_dir.is_empty() {
                return Some(first_dir + "/");
            }
        } else if name.contains('\\') {
            let first_dir = name.split('\\').next()?.to_string();
            if !first_dir.is_empty() {
                return Some(first_dir + "\\");
            }
        }
    }
    None
}
