use crate::db::Database;
use crate::models::{ActivityItem, DashboardStats, ScanResult};
use crate::sync::{hash_content, resolve_username};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use tauri::State;

#[tauri::command]
pub fn scan_all_agents(db: State<Database>) -> Result<ScanResult, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut errors = Vec::new();

    let mut astmt = conn
        .prepare("SELECT id, name, skills_path FROM agents WHERE is_active = 1")
        .map_err(|e| e.to_string())?;
    let agents: Vec<(String, String, String)> = astmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let agents_scanned = agents.len() as i64;
    let mut all_skill_folders: HashMap<String, Vec<(String, String)>> = HashMap::new();

    for (agent_id, agent_name, raw_path) in &agents {
        let path = resolve_username(raw_path);
        let dir = Path::new(&path);
        if !dir.exists() {
            errors.push(format!("{}: directory not found: {}", agent_name, path));
            continue;
        }
        let read_dir = match fs::read_dir(dir) {
            Ok(d) => d,
            Err(e) => {
                errors.push(format!("{}: {}", agent_name, e));
                continue;
            }
        };

        for entry in read_dir.flatten() {
            if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                continue;
            }
            let folder_name = entry.file_name().to_string_lossy().to_string();
            let md_path = entry.path().join("SKILL.md");
            if !md_path.exists() {
                continue;
            }

            all_skill_folders
                .entry(folder_name.clone())
                .or_default()
                .push((agent_id.clone(), agent_name.clone()));
        }
    }

    let skills_found = all_skill_folders.len() as i64;
    let mut new_skills: i64 = 0;
    let mut changed_skills: i64 = 0;
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    for (folder_name, agent_refs) in &all_skill_folders {
        let existing: Option<String> = conn
            .query_row(
                "SELECT id FROM skills WHERE slug = ?1",
                [folder_name],
                |row| row.get(0),
            )
            .ok();

        if let Some(skill_id) = existing {
            // Ensure all scanning agents are linked
            for (agent_id, _) in agent_refs {
                conn.execute(
                    "INSERT OR IGNORE INTO agent_skills (agent_id, skill_id, sync_status, synced_at) VALUES (?1, ?2, 'synced', ?3)",
                    rusqlite::params![agent_id, skill_id, now],
                ).ok();
            }

            let mut changed = false;
            for (agent_id, _) in agent_refs {
                let raw_path: String = conn
                    .query_row(
                        "SELECT skills_path FROM agents WHERE id = ?1",
                        [agent_id],
                        |row| row.get(0),
                    )
                    .ok()
                    .unwrap_or_default();
                if raw_path.is_empty() {
                    continue;
                }

                let dir = Path::new(&resolve_username(&raw_path)).join(folder_name);
                if !dir.exists() {
                    continue;
                }

                for entry in walkdir::WalkDir::new(&dir).max_depth(10) {
                    let entry = match entry {
                        Ok(e) => e,
                        Err(_) => continue,
                    };
                    if !entry.file_type().is_file() {
                        continue;
                    }
                    let relative = entry.path().strip_prefix(&dir).unwrap_or(entry.path());
                    let rel_str = relative.to_string_lossy().to_string();

                    let content = match fs::read(entry.path()) {
                        Ok(c) => c,
                        Err(_) => continue,
                    };
                    let hash = hash_content(&content);

                    let db_hash: Option<String> = conn.query_row(
                        "SELECT content_hash FROM skill_files WHERE skill_id = ?1 AND relative_path = ?2",
                        rusqlite::params![skill_id, rel_str], |row| row.get(0)
                    ).ok();

                    if db_hash.as_deref() != Some(&hash) {
                        conn.execute(
                            "UPDATE skill_files SET content_hash = ?1, file_size = ?2, updated_at = ?3 WHERE skill_id = ?4 AND relative_path = ?5",
                            rusqlite::params![hash, content.len() as i64, now, skill_id, rel_str],
                        ).ok();
                        changed = true;
                    }
                }
            }
            if changed {
                changed_skills += 1;
                // Mark linked agents as pending
                for (agent_id, _) in agent_refs {
                    conn.execute(
                        "UPDATE agent_skills SET sync_status = 'pending' WHERE agent_id = ?1 AND skill_id = ?2",
                        rusqlite::params![agent_id, skill_id],
                    ).ok();
                }
            }
        } else {
            let skill_id = uuid::Uuid::new_v4().to_string();
            let (agent_id, _) = &agent_refs[0];
            let raw_path: String = conn
                .query_row(
                    "SELECT skills_path FROM agents WHERE id = ?1",
                    [agent_id],
                    |row| row.get(0),
                )
                .ok()
                .unwrap_or_default();

            let dir = Path::new(&resolve_username(&raw_path)).join(folder_name);
            let md_path = dir.join("SKILL.md");
            let md_content = fs::read_to_string(&md_path).unwrap_or_default();
            let (skill_name, description) = parse_frontmatter(&md_content);

            conn.execute(
                "INSERT INTO skills (id, name, slug, description, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                rusqlite::params![skill_id, skill_name, folder_name, description, now, now],
            ).ok();

            // Link all agents that have this skill folder
            for (agent_id, _) in agent_refs {
                conn.execute(
                    "INSERT INTO agent_skills (agent_id, skill_id, sync_status, synced_at) VALUES (?1, ?2, 'synced', ?3)",
                    rusqlite::params![agent_id, skill_id, now],
                ).ok();
            }

            for entry in walkdir::WalkDir::new(&dir).max_depth(10) {
                let entry = match entry {
                    Ok(e) => e,
                    Err(_) => continue,
                };
                let relative = entry.path().strip_prefix(&dir).unwrap_or(entry.path());
                let rel_str = relative.to_string_lossy().to_string();
                let file_name = entry.file_name().to_string_lossy().to_string();

                let (file_id, hash, file_size, is_dir) = if entry.file_type().is_dir() {
                    (uuid::Uuid::new_v4().to_string(), String::new(), 0i64, true)
                } else {
                    let content = fs::read(entry.path()).unwrap_or_default();
                    (
                        uuid::Uuid::new_v4().to_string(),
                        hash_content(&content),
                        content.len() as i64,
                        false,
                    )
                };

                conn.execute(
                    "INSERT INTO skill_files (id, skill_id, relative_path, file_name, content_hash, file_size, is_directory, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    rusqlite::params![file_id, skill_id, rel_str, file_name, hash, file_size, is_dir, now],
                ).ok();
            }

            let activity_skill_name = if skill_name == "unnamed" {
                folder_name.clone()
            } else {
                skill_name.clone()
            };
            conn.execute(
                "INSERT INTO activities (type, message, created_at) VALUES ('create', ?1, ?2)",
                rusqlite::params![format!("扫描发现 Skill: {}", activity_skill_name), now],
            )
            .ok();
            new_skills += 1;
        }
    }

    // Detect skills that have been removed from every active agent.
    let mut deleted_skills: i64 = 0;
    let mut db_stmt = conn
        .prepare("SELECT id, slug FROM skills")
        .map_err(|e| e.to_string())?;
    let db_skills: Vec<(String, String)> = db_stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    for (skill_id, slug) in &db_skills {
        if !all_skill_folders.contains_key(slug) {
            if crate::sync::delete_skill_cascade(&conn, skill_id).is_ok() {
                deleted_skills += 1;
            }
        }
    }

    Ok(ScanResult {
        agents_scanned,
        skills_found,
        new_skills,
        changed_skills,
        deleted_skills,
        errors,
    })
}

#[tauri::command]
pub fn get_scan_history(db: State<Database>) -> Result<Vec<String>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare("SELECT message FROM activities ORDER BY created_at DESC LIMIT 20")
        .map_err(|e| e.to_string())?;
    let items = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    Ok(items)
}

#[tauri::command]
pub fn get_dashboard_stats(db: State<Database>) -> Result<DashboardStats, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let total_agents: i64 = conn
        .query_row("SELECT COUNT(*) FROM agents", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    let active_agents: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM agents WHERE is_active = 1",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let total_skills: i64 = conn
        .query_row("SELECT COUNT(*) FROM skills", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;
    let total_files: i64 = conn
        .query_row("SELECT COUNT(*) FROM skill_files", [], |row| row.get(0))
        .map_err(|e| e.to_string())?;

    let pending_sync_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM agent_skills WHERE sync_status = 'pending'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let mut stmt = conn
        .prepare(
            "SELECT type, message, created_at FROM activities ORDER BY created_at DESC LIMIT 10",
        )
        .map_err(|e| e.to_string())?;

    let recent_activities: Vec<ActivityItem> = stmt
        .query_map([], |row| {
            Ok(ActivityItem {
                r#type: row.get(0)?,
                message: row.get(1)?,
                time: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(DashboardStats {
        total_agents,
        active_agents,
        total_skills,
        total_files,
        pending_sync_count,
        recent_activities,
    })
}

fn parse_frontmatter(content: &str) -> (String, String) {
    let content_trimmed = content.trim();
    if !content_trimmed.starts_with("---") {
        return ("unnamed".to_string(), String::new());
    }

    let end = match content_trimmed[3..].find("---") {
        Some(pos) => pos + 3,
        None => return ("unnamed".to_string(), String::new()),
    };

    let frontmatter = &content_trimmed[3..end];
    let mut name = String::from("unnamed");
    let mut description = String::new();

    for line in frontmatter.lines() {
        let line = line.trim();
        if let Some(val) = line.strip_prefix("name:") {
            name = val.trim().trim_matches('"').to_string();
        } else if let Some(val) = line.strip_prefix("description:") {
            description = val.trim().trim_matches('"').to_string();
        }
    }

    (name, description)
}
