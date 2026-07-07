use crate::db::Database;
use crate::models::{AgentBrief, Skill, SkillDetail, SkillFile};
use crate::sync::{hash_content, resolve_username};
use std::fs;
use std::path::Path;
use tauri::State;

#[tauri::command]
pub fn get_skills(
    db: State<Database>,
    agent_id: Option<String>,
    search: Option<String>,
) -> Result<Vec<Skill>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let query = if let Some(_aid) = &agent_id {
        "SELECT s.id, s.name, s.slug, s.description,
                (SELECT COUNT(*) FROM agent_skills WHERE skill_id = s.id) as agents_count,
                (SELECT COUNT(*) FROM skill_files sf WHERE sf.skill_id = s.id) as files_count,
                s.created_at, s.updated_at
         FROM skills s
         WHERE s.id IN (SELECT skill_id FROM agent_skills WHERE agent_id = ?1)
         ORDER BY s.name"
            .to_string()
    } else {
        "SELECT s.id, s.name, s.slug, s.description,
                (SELECT COUNT(*) FROM agent_skills WHERE skill_id = s.id) as agents_count,
                (SELECT COUNT(*) FROM skill_files sf WHERE sf.skill_id = s.id) as files_count,
                s.created_at, s.updated_at
         FROM skills s ORDER BY s.name"
            .to_string()
    };

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;

    let skills = if let Some(ref aid) = agent_id {
        stmt.query_map([aid], |row| {
            Ok(Skill {
                id: row.get(0)?,
                name: row.get(1)?,
                slug: row.get(2)?,
                description: row.get(3)?,
                agents_count: row.get(4)?,
                files_count: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .filter(|s: &Skill| {
            if let Some(ref q) = search {
                if q.is_empty() {
                    return true;
                }
                let q = q.to_lowercase();
                s.name.to_lowercase().contains(&q)
                    || s.slug.to_lowercase().contains(&q)
                    || s.description.to_lowercase().contains(&q)
            } else {
                true
            }
        })
        .collect()
    } else {
        stmt.query_map([], |row| {
            Ok(Skill {
                id: row.get(0)?,
                name: row.get(1)?,
                slug: row.get(2)?,
                description: row.get(3)?,
                agents_count: row.get(4)?,
                files_count: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .filter(|s: &Skill| {
            if let Some(ref q) = search {
                if q.is_empty() {
                    return true;
                }
                let q = q.to_lowercase();
                s.name.to_lowercase().contains(&q)
                    || s.slug.to_lowercase().contains(&q)
                    || s.description.to_lowercase().contains(&q)
            } else {
                true
            }
        })
        .collect()
    };

    Ok(skills)
}

#[tauri::command]
pub fn get_skill_detail(db: State<Database>, id: String) -> Result<SkillDetail, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let (name, slug, description, created_at, updated_at): (
        String,
        String,
        String,
        String,
        String,
    ) = conn
        .query_row(
            "SELECT name, slug, description, created_at, updated_at FROM skills WHERE id = ?1",
            [&id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            },
        )
        .map_err(|e| format!("Skill not found: {}", e))?;

    let mut fstmt = conn
        .prepare(
            "SELECT id, relative_path, file_name, content_hash, file_size, is_directory, updated_at
         FROM skill_files WHERE skill_id = ?1 ORDER BY relative_path",
        )
        .map_err(|e| e.to_string())?;

    let files: Vec<SkillFile> = fstmt
        .query_map([&id], |row| {
            Ok(SkillFile {
                id: row.get(0)?,
                relative_path: row.get(1)?,
                file_name: row.get(2)?,
                content_hash: row.get(3)?,
                file_size: row.get(4)?,
                is_directory: row.get(5)?,
                updated_at: row.get(6)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let agents: Vec<AgentBrief> = {
        let mut astmt = conn
            .prepare(
                "SELECT a.id, a.name, a.display_name, COALESCE(ag.sync_status, 'unknown')
             FROM agents a
             LEFT JOIN agent_skills ag ON ag.agent_id = a.id AND ag.skill_id = ?1
             WHERE a.is_active = 1 ORDER BY a.name",
            )
            .map_err(|e| e.to_string())?;
        let rows = astmt
            .query_map([&id], |row| {
                Ok(AgentBrief {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    display_name: row.get(2)?,
                    sync_status: row.get(3)?,
                })
            })
            .map_err(|e| e.to_string())?;
        rows.filter_map(|r| r.ok()).collect()
    };

    let agents_count = agents.len() as i64;
    let files_count = files.len() as i64;

    Ok(SkillDetail {
        id,
        name,
        slug,
        description,
        agents_count,
        files_count,
        created_at,
        updated_at,
        files,
        agents,
    })
}

#[tauri::command]
pub fn create_skill(
    db: State<Database>,
    name: String,
    description: String,
) -> Result<Skill, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let id = uuid::Uuid::new_v4().to_string();
    let slug = sanitize_slug(&name);
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let md_content = format!(
        "---\nname: \"{}\"\ndescription: \"{}\"\n---\n\n# {}\n\n## 简介\n\n待补充\n",
        name, description, name
    );

    conn.execute(
        "INSERT INTO skills (id, name, slug, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![id, name, slug, description, now, now],
    ).map_err(|e| format!("Failed to create skill: {}", e))?;

    let md_id = uuid::Uuid::new_v4().to_string();
    conn.execute(
        "INSERT INTO skill_files (id, skill_id, relative_path, file_name, content_hash, file_size, is_directory, updated_at)
         VALUES (?1, ?2, 'SKILL.md', 'SKILL.md', ?3, ?4, 0, ?5)",
        rusqlite::params![md_id, id, hash_content(md_content.as_bytes()), md_content.len() as i64, now],
    ).map_err(|e| e.to_string())?;

    let mut astmt = conn
        .prepare("SELECT id, skills_path FROM agents WHERE is_active = 1")
        .map_err(|e| e.to_string())?;
    let agent_data: Vec<(String, String)> = astmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    for (_, raw_path) in &agent_data {
        let dir = Path::new(&resolve_username(raw_path)).join(&slug);
        if let Err(e) = fs::create_dir_all(&dir) {
            log::warn!("Failed to create dir {}: {}", dir.display(), e);
            continue;
        }
        let md_path = dir.join("SKILL.md");
        if let Err(e) = fs::write(&md_path, &md_content) {
            log::warn!("Failed to write {}: {}", md_path.display(), e);
        }
    }

    for (agent_id, _) in &agent_data {
        conn.execute(
            "INSERT INTO agent_skills (agent_id, skill_id, sync_status, synced_at) VALUES (?1, ?2, 'pending', ?3)",
            rusqlite::params![agent_id, id, now],
        ).ok();
    }

    let agents_count = agent_data.len() as i64;

    let now2 = now.clone();
    conn.execute(
        "INSERT INTO activities (type, message, created_at) VALUES ('create', ?1, ?2)",
        rusqlite::params![format!("创建 Skill: {}", name), now2],
    )
    .ok();

    Ok(Skill {
        id,
        name,
        slug,
        description,
        agents_count,
        files_count: 1,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn update_skill_md(db: State<Database>, id: String, content: String) -> Result<Skill, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    conn.execute(
        "UPDATE skills SET updated_at = ?1 WHERE id = ?2",
        rusqlite::params![now, &id],
    )
    .map_err(|e| e.to_string())?;

    let (name, slug): (String, String) = conn
        .query_row(
            "SELECT name, slug FROM skills WHERE id = ?1",
            [&id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("Skill not found: {}", e))?;

    let hash = hash_content(content.as_bytes());
    let file_size = content.len() as i64;

    conn.execute(
        "UPDATE skill_files SET content_hash = ?1, file_size = ?2, updated_at = ?3 WHERE skill_id = ?4 AND relative_path = 'SKILL.md'",
        rusqlite::params![hash, file_size, now, &id],
    ).map_err(|e| e.to_string())?;

    let mut astmt = conn
        .prepare("SELECT id, skills_path FROM agents WHERE is_active = 1")
        .map_err(|e| e.to_string())?;
    let agent_data: Vec<(String, String)> = astmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    for (agent_id, _) in &agent_data {
        conn.execute(
            "UPDATE agent_skills SET sync_status = 'pending' WHERE agent_id = ?1 AND skill_id = ?2",
            rusqlite::params![agent_id, &id],
        )
        .ok();
    }

    for (_, raw_path) in &agent_data {
        let dir = Path::new(&resolve_username(raw_path)).join(&slug);
        if let Err(_) = fs::create_dir_all(&dir) {
            continue;
        }
        fs::write(dir.join("SKILL.md"), &content).ok();
    }

    conn.execute(
        "INSERT INTO activities (type, message, created_at) VALUES ('update', ?1, ?2)",
        rusqlite::params![format!("修改 Skill: {}", name), now],
    )
    .ok();

    let agents_count = agent_data.len() as i64;
    let files_count = {
        let mut stmt = conn
            .prepare("SELECT COUNT(*) FROM skill_files WHERE skill_id = ?1")
            .map_err(|e| e.to_string())?;
        stmt.query_row([&id], |row| row.get::<_, i64>(0))
            .map_err(|e| e.to_string())?
    };

    Ok(Skill {
        id,
        name,
        slug,
        description: String::new(),
        agents_count,
        files_count,
        created_at: String::new(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn delete_skill(db: State<Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    crate::sync::delete_skill_cascade(&conn, &id)?;
    Ok(())
}

#[tauri::command]
pub fn read_skill_md(db: State<Database>, id: String) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let slug: String = conn
        .query_row("SELECT slug FROM skills WHERE id = ?1", [&id], |row| {
            row.get(0)
        })
        .map_err(|e| format!("Skill not found: {}", e))?;

    let mut astmt = conn
        .prepare("SELECT skills_path FROM agents WHERE is_active = 1 LIMIT 1")
        .map_err(|e| e.to_string())?;
    let raw_path: String = astmt
        .query_row([], |row| row.get(0))
        .map_err(|_| "No active agent".to_string())?;

    let md_path = Path::new(&resolve_username(&raw_path))
        .join(&slug)
        .join("SKILL.md");
    fs::read_to_string(&md_path).map_err(|e| format!("Failed to read SKILL.md: {}", e))
}

#[tauri::command]
pub fn upload_files(
    db: State<Database>,
    skill_id: String,
    file_paths: Vec<String>,
) -> Result<Vec<SkillFile>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let slug: String = conn
        .query_row(
            "SELECT slug FROM skills WHERE id = ?1",
            [&skill_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Skill not found: {}", e))?;

    let mut astmt = conn
        .prepare("SELECT id, skills_path FROM agents WHERE is_active = 1")
        .map_err(|e| e.to_string())?;
    let agent_data: Vec<(String, String)> = astmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut results = Vec::new();
    for file_path_str in &file_paths {
        let src = Path::new(file_path_str);
        if !src.exists() {
            continue;
        }

        let file_name = src
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let relative_path = file_name.clone();
        let file_size = src.metadata().map(|m| m.len() as i64).unwrap_or(0);
        let content = fs::read(src).map_err(|e| e.to_string())?;
        let content_hash = hash_content(&content);

        let file_id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO skill_files (id, skill_id, relative_path, file_name, content_hash, file_size, is_directory, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0, ?7)",
            rusqlite::params![file_id, &skill_id, relative_path, file_name, content_hash, file_size, now],
        ).map_err(|e| e.to_string())?;

        for (_, raw_path) in &agent_data {
            let target_dir = Path::new(&resolve_username(raw_path)).join(&slug);
            let target = target_dir.join(&relative_path);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent).ok();
            }
            fs::write(&target, &content).ok();
        }

        results.push(SkillFile {
            id: file_id,
            relative_path: relative_path.clone(),
            file_name,
            content_hash,
            file_size,
            is_directory: false,
            updated_at: now.clone(),
        });
    }

    for (agent_id, _) in &agent_data {
        conn.execute(
            "UPDATE agent_skills SET sync_status = 'pending' WHERE agent_id = ?1 AND skill_id = ?2",
            rusqlite::params![agent_id, &skill_id],
        )
        .ok();
    }

    Ok(results)
}

#[tauri::command]
pub fn remove_file(
    db: State<Database>,
    skill_id: String,
    relative_path: String,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let slug: String = conn
        .query_row(
            "SELECT slug FROM skills WHERE id = ?1",
            [&skill_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Skill not found: {}", e))?;

    conn.execute(
        "DELETE FROM skill_files WHERE skill_id = ?1 AND relative_path = ?2",
        rusqlite::params![skill_id, relative_path],
    )
    .map_err(|e| e.to_string())?;

    let mut astmt = conn
        .prepare("SELECT id, skills_path FROM agents WHERE is_active = 1")
        .map_err(|e| e.to_string())?;
    let agent_data: Vec<(String, String)> = astmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    for (_, raw_path) in &agent_data {
        let target = Path::new(&resolve_username(raw_path))
            .join(&slug)
            .join(&relative_path);
        if target.exists() {
            fs::remove_file(&target).ok();
        }
    }

    for (agent_id, _) in &agent_data {
        conn.execute(
            "UPDATE agent_skills SET sync_status = 'pending' WHERE agent_id = ?1 AND skill_id = ?2",
            rusqlite::params![agent_id, &skill_id],
        )
        .ok();
    }

    Ok(())
}

#[tauri::command]
pub fn read_file_content(
    db: State<Database>,
    skill_id: String,
    relative_path: String,
) -> Result<String, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let slug: String = conn
        .query_row(
            "SELECT slug FROM skills WHERE id = ?1",
            [&skill_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Skill not found: {}", e))?;

    let mut astmt = conn
        .prepare("SELECT skills_path FROM agents WHERE is_active = 1 LIMIT 1")
        .map_err(|e| e.to_string())?;
    let raw_path: String = astmt
        .query_row([], |row| row.get(0))
        .map_err(|_| "No active agent".to_string())?;

    let file_path = Path::new(&resolve_username(&raw_path))
        .join(&slug)
        .join(&relative_path);
    fs::read_to_string(&file_path).map_err(|e| format!("Failed to read file: {}", e))
}

fn sanitize_slug(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.is_empty() {
        "unnamed_skill".to_string()
    } else {
        cleaned.trim_matches('_').to_string()
    }
}
