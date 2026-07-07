use crate::models::{AgentSyncResult, SyncReport};
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub fn resolve_username(path: &str) -> String {
    path.replace("{{USERNAME}}", &whoami::username())
}

pub fn hash_content(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    format!("{:x}", hasher.finalize())
}

pub(crate) fn copy_dir_contents(src: &Path, dst: &Path) -> Result<(), String> {
    if !src.is_dir() {
        return Err("Source not a directory".to_string());
    }
    fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    for entry in walkdir::WalkDir::new(src) {
        let entry = entry.map_err(|e| e.to_string())?;
        let relative = entry.path().strip_prefix(src).map_err(|e| e.to_string())?;
        let target = dst.join(relative);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target).map_err(|e| e.to_string())?;
        } else {
            fs::copy(entry.path(), &target).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

/// Remove a skill directory from every active agent.
pub fn remove_skill_from_all_agents(
    conn: &Connection,
    slug: &str,
) -> Result<Vec<AgentSyncResult>, String> {
    let mut stmt = conn
        .prepare("SELECT id, name, skills_path FROM agents WHERE is_active = 1 ORDER BY created_at ASC")
        .map_err(|e| e.to_string())?;
    let agents: Vec<(String, String, String)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    let mut results = Vec::new();
    for (agent_id, agent_name, raw_path) in agents {
        let skill_dir = Path::new(&resolve_username(&raw_path)).join(slug);
        let mut result = AgentSyncResult {
            agent_id,
            agent_name,
            success: true,
            error: None,
        };
        if skill_dir.exists() {
            if let Err(e) = fs::remove_dir_all(&skill_dir) {
                result.success = false;
                result.error = Some(format!("Failed to remove dir: {}", e));
            }
        }
        results.push(result);
    }
    Ok(results)
}

fn delete_skill_records(conn: &Connection, skill_id: &str, skill_name: &str) -> Result<(), String> {
    conn.execute("DELETE FROM agent_skills WHERE skill_id = ?1", [skill_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM skill_files WHERE skill_id = ?1", [skill_id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM skills WHERE id = ?1", [skill_id])
        .map_err(|e| e.to_string())?;

    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    conn.execute(
        "INSERT INTO activities (type, message, created_at) VALUES ('delete', ?1, ?2)",
        rusqlite::params![format!("删除 Skill: {}", skill_name), now],
    )
    .ok();

    Ok(())
}

/// Delete a skill from the database and every active agent directory.
/// Returns the skill name on success.
pub fn delete_skill_cascade(conn: &Connection, skill_id: &str) -> Result<String, String> {
    let (name, slug): (String, String) = conn
        .query_row(
            "SELECT name, slug FROM skills WHERE id = ?1",
            [skill_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("Skill not found: {}", e))?;

    remove_skill_from_all_agents(conn, &slug)?;
    delete_skill_records(conn, skill_id, &name)?;
    Ok(name)
}

pub fn sync_skill(conn: &Connection, skill_id: &str) -> Result<SyncReport, String> {
    let (skill_name, slug): (String, String) = conn
        .query_row(
            "SELECT name, slug FROM skills WHERE id = ?1",
            [skill_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .map_err(|e| format!("Skill not found: {}", e))?;

    let mut stmt = conn
        .prepare("SELECT id, name, skills_path FROM agents WHERE is_active = 1 ORDER BY created_at ASC")
        .map_err(|e| e.to_string())?;
    let agents: Vec<(String, String, String)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    // The first active agent is the source of truth for this skill.
    // If the skill no longer exists there, treat it as deleted and propagate
    // the deletion to every other active agent.
    let source_info = get_source_path(conn, &slug)?;
    if source_info.is_none() {
        let agent_results = remove_skill_from_all_agents(conn, &slug)?;
        delete_skill_records(conn, skill_id, &skill_name)?;
        return Ok(SyncReport {
            skill_id: skill_id.to_string(),
            skill_name,
            agent_results,
        });
    }

    let (source_agent_id, src) = source_info.unwrap();
    let mut agent_results = Vec::new();
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    for (agent_id, agent_name, raw_path) in &agents {
        let agent_path = resolve_username(raw_path);
        let skill_dir = Path::new(&agent_path).join(&slug);

        let mut result = AgentSyncResult {
            agent_id: agent_id.clone(),
            agent_name: agent_name.clone(),
            success: true,
            error: None,
        };

        // Skip deletion for the source agent to avoid destroying the source before copying.
        if agent_id != &source_agent_id {
            // Mirror the source directory exactly: remove stale files/folders,
            // then recreate and copy the latest content.
            if skill_dir.exists() {
                if let Err(e) = fs::remove_dir_all(&skill_dir) {
                    result.success = false;
                    result.error = Some(format!("Failed to clean dir: {}", e));
                    agent_results.push(result);
                    continue;
                }
            }

            if let Err(e) = fs::create_dir_all(&skill_dir) {
                result.success = false;
                result.error = Some(format!("Failed to create dir: {}", e));
                agent_results.push(result);
                continue;
            }

            if let Err(e) = copy_dir_contents(Path::new(&src), &skill_dir) {
                result.success = false;
                result.error = Some(format!("Sync failed: {}", e));
            }
        }
        agent_results.push(result);
    }

    for (agent_id, _, _) in &agents {
        let success = agent_results
            .iter()
            .any(|r| r.agent_id == *agent_id && r.success);
        if success {
            conn.execute(
                "UPDATE agent_skills SET sync_status = 'synced', synced_at = ?1 WHERE agent_id = ?2 AND skill_id = ?3",
                rusqlite::params![now, agent_id, skill_id],
            ).ok();
        }
    }

    conn.execute(
        "INSERT INTO activities (type, message, created_at) VALUES ('sync', ?1, ?2)",
        rusqlite::params![format!("同步 Skill: {}", skill_name), now],
    )
    .ok();

    Ok(SyncReport {
        skill_id: skill_id.to_string(),
        skill_name,
        agent_results,
    })
}

fn get_source_path(conn: &Connection, slug: &str) -> Result<Option<(String, String)>, String> {
    let mut stmt = conn
        .prepare("SELECT id, skills_path FROM agents WHERE is_active = 1 ORDER BY created_at ASC LIMIT 1")
        .map_err(|e| e.to_string())?;
    let (agent_id, raw_path): (String, String) = stmt
        .query_row([], |row| Ok((row.get(0)?, row.get(1)?)))
        .map_err(|_| "No active agent".to_string())?;
    let path = resolve_username(&raw_path);
    let full = Path::new(&path).join(slug);
    if full.exists() {
        Ok(Some((agent_id, full.to_string_lossy().to_string())))
    } else {
        Ok(None)
    }
}

pub fn sync_skill_to_all(conn: &Connection) -> Result<Vec<SyncReport>, String> {
    let mut stmt = conn
        .prepare("SELECT id FROM skills")
        .map_err(|e| e.to_string())?;
    let skill_ids: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();
    let mut reports = Vec::new();
    for id in skill_ids {
        match sync_skill(conn, &id) {
            Ok(r) => reports.push(r),
            Err(e) => log::error!("Sync skill {} failed: {}", id, e),
        }
    }
    Ok(reports)
}
