use std::path::PathBuf;
use tauri::Manager;

mod commands;
mod db;
mod models;
mod sync;

fn get_app_data_dir(app: &tauri::AppHandle) -> PathBuf {
    app.path().app_data_dir().unwrap_or_else(|_| {
        let mut p = PathBuf::new();
        p.push("skill_manager_data");
        p
    })
}

#[tauri::command]
fn sync_skill_to_agents(
    db: tauri::State<db::Database>,
    skill_id: String,
) -> Result<models::SyncReport, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    sync::sync_skill(&conn, &skill_id)
}

#[tauri::command]
fn sync_all_skills(db: tauri::State<db::Database>) -> Result<Vec<models::SyncReport>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    sync::sync_skill_to_all(&conn)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let app_dir = get_app_data_dir(app.handle());
            std::fs::create_dir_all(&app_dir).ok();

            let db = db::Database::new(app_dir.clone())
                .expect("Failed to initialize database");

            {
                let conn = db.conn.lock().unwrap();
                let count: i64 = conn
                    .query_row("SELECT COUNT(*) FROM agents", [], |row| row.get(0))
                    .unwrap_or(0);

                if count == 0 {
                    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                    let default_agents: Vec<(&str, &str, &str)> = vec![
                        ("opencode", "OpenCode", r"C:\Users\{{USERNAME}}\.config\opencode\skills"),
                        ("claude-code", "Claude Code", r"C:\Users\{{USERNAME}}\.claude\skills"),
                        ("codex", "Codex", r"C:\Users\{{USERNAME}}\.codex\skills"),
                        ("qoder", "Qoder", r"C:\Users\{{USERNAME}}\.qoder\skills"),
                        ("trae-cn", "Trae CN", r"C:\Users\{{USERNAME}}\.trae-cn\skills"),
                        ("codebuddy", "CodeBuddy", r"C:\Users\{{USERNAME}}\.codebuddy\skills"),
                        ("workbuddy", "WorkBuddy", r"C:\Users\{{USERNAME}}\.workbuddy\skills"),
                        ("hermes", "Hermes", r"C:\Users\{{USERNAME}}\.hermes\skills"),
                    ];

                    for (name, display_name, path) in default_agents {
                        let id = uuid::Uuid::new_v4().to_string();
                        conn.execute(
                            "INSERT INTO agents (id, name, display_name, skills_path, is_active, created_at) VALUES (?1, ?2, ?3, ?4, 1, ?5)",
                            rusqlite::params![id, name, display_name, path, now],
                        ).ok();
                    }
                }
            }

            app.manage(db);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::agents::get_agents,
            commands::agents::add_agent,
            commands::agents::remove_agent,
            commands::agents::update_agent,
            commands::scanner::scan_all_agents,
            commands::scanner::get_scan_history,
            commands::scanner::get_dashboard_stats,
            commands::skills::get_skills,
            commands::skills::get_skill_detail,
            commands::skills::create_skill,
            commands::skills::update_skill_md,
            commands::skills::delete_skill,
            commands::skills::read_skill_md,
            commands::skills::upload_files,
            commands::skills::remove_file,
            commands::skills::read_file_content,
            commands::marketplace::install_marketplace_skill,
            commands::marketplace::get_installed_skills,
            commands::marketplace::uninstall_installed_skill,
            commands::marketplace::update_installed_skill,
            sync_skill_to_agents,
            sync_all_skills,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
