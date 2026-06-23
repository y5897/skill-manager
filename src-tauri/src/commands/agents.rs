use crate::db::Database;
use crate::models::Agent;
use tauri::State;

fn get_display_name(name: &str) -> String {
    match name {
        "opencode" => "OpenCode",
        "claude-code" => "Claude Code",
        "codex" => "Codex",
        "qoder" => "Qoder",
        "trae-cn" => "Trae CN",
        "codebuddy" => "CodeBuddy",
        "workbuddy" => "WorkBuddy",
        "hermes" => "Hermes",
        _ => name,
    }
    .to_string()
}

#[tauri::command]
pub fn get_agents(db: State<Database>) -> Result<Vec<Agent>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let mut stmt = conn
        .prepare(
            "SELECT a.id, a.name, a.display_name, a.skills_path, a.is_active,
                (SELECT COUNT(*) FROM agent_skills WHERE agent_id = a.id) as skills_count
         FROM agents a ORDER BY a.name",
        )
        .map_err(|e| e.to_string())?;

    let agents = stmt
        .query_map([], |row| {
            let id: String = row.get(0)?;
            Ok(Agent {
                id: id.clone(),
                name: row.get(1)?,
                display_name: row.get(2)?,
                skills_path: row.get(3)?,
                is_active: row.get(4)?,
                skills_count: row.get(5)?,
                sync_status: resolve_sync_status(&conn, &id),
                created_at: "".to_string(),
            })
        })
        .map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

    Ok(agents)
}

    fn resolve_sync_status(conn: &rusqlite::Connection, agent_id: &str) -> String {
    let stmt = conn
        .prepare(
            "SELECT COUNT(*) FROM agent_skills WHERE agent_id = ?1 AND sync_status = 'pending'",
        )
        .ok();
    match stmt {
        Some(mut s) => {
            let count: i64 = s.query_row([agent_id], |row| row.get(0)).unwrap_or(0);
            if count > 0 {
                "pending".to_string()
            } else {
                "synced".to_string()
            }
        }
        None => "unknown".to_string(),
    }
}

#[tauri::command]
pub fn add_agent(db: State<Database>, name: String, skills_path: String) -> Result<Agent, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let id = uuid::Uuid::new_v4().to_string();
    let display_name = get_display_name(&name);
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    conn.execute(
        "INSERT INTO agents (id, name, display_name, skills_path, is_active, created_at) VALUES (?1, ?2, ?3, ?4, 1, ?5)",
        rusqlite::params![id, name, display_name, skills_path, now],
    ).map_err(|e| format!("Failed to add agent: {}", e))?;

    Ok(Agent {
        id,
        name,
        display_name,
        skills_path,
        is_active: true,
        skills_count: 0,
        sync_status: "unknown".to_string(),
        created_at: now,
    })
}

#[tauri::command]
pub fn remove_agent(db: State<Database>, id: String) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM agent_skills WHERE agent_id = ?1", [&id])
        .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM agents WHERE id = ?1", [id])
        .map_err(|e| format!("Failed to remove agent: {}", e))?;
    Ok(())
}

#[tauri::command]
pub fn update_agent(
    db: State<Database>,
    id: String,
    name: String,
    skills_path: String,
    is_active: bool,
) -> Result<Agent, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let display_name = get_display_name(&name);
    conn.execute(
        "UPDATE agents SET name = ?1, display_name = ?2, skills_path = ?3, is_active = ?4 WHERE id = ?5",
        rusqlite::params![name, display_name, skills_path, is_active, &id],
    ).map_err(|e| format!("Failed to update agent: {}", e))?;

    let skills_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM agent_skills WHERE agent_id = ?1",
            [&id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    Ok(Agent {
        id: id.clone(),
        name: name.clone(),
        display_name: display_name.clone(),
        skills_path: skills_path.clone(),
        is_active,
        skills_count,
        sync_status: resolve_sync_status(&conn, &id),
        created_at: "".to_string(),
    })
}