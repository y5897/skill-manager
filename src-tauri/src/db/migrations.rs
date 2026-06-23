use rusqlite::Connection;

pub fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS agents (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL UNIQUE,
            display_name TEXT NOT NULL,
            skills_path TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL
            );",
    )?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS skills (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            slug TEXT NOT NULL UNIQUE,
            description TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
            );",
    )?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS agent_skills (
            agent_id TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
            skill_id TEXT NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
            sync_status TEXT NOT NULL DEFAULT 'unknown',
            synced_at TEXT,
            PRIMARY KEY (agent_id, skill_id)
            );",
    )?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS skill_files (
            id TEXT PRIMARY KEY,
            skill_id TEXT NOT NULL REFERENCES skills(id) ON DELETE CASCADE,
            relative_path TEXT NOT NULL,
            file_name TEXT NOT NULL,
            content_hash TEXT NOT NULL DEFAULT '',
            file_size INTEGER NOT NULL DEFAULT 0,
            is_directory INTEGER NOT NULL DEFAULT 0,
            updated_at TEXT NOT NULL
            );",
    )?;

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS activities (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            type TEXT NOT NULL,
            message TEXT NOT NULL,
            created_at TEXT NOT NULL
            );",
    )?;

    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_skill_files_skill ON skill_files(skill_id);",
    )?;
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_activities_time ON activities(created_at);",
    )?;
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_agent_skills_agent ON agent_skills(agent_id);",
    )?;
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_agent_skills_skill ON agent_skills(skill_id);",
    )?;

    Ok(())
}
