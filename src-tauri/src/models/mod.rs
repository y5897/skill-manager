use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub skills_path: String,
    pub is_active: bool,
    pub skills_count: i64,
    pub sync_status: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub agents_count: i64,
    pub files_count: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDetail {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub agents_count: i64,
    pub files_count: i64,
    pub created_at: String,
    pub updated_at: String,
    pub files: Vec<SkillFile>,
    pub agents: Vec<AgentBrief>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFile {
    pub id: String,
    pub relative_path: String,
    pub file_name: String,
    pub content_hash: String,
    pub file_size: i64,
    pub is_directory: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBrief {
    pub id: String,
    pub name: String,
    pub display_name: String,
    pub sync_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub agents_scanned: i64,
    pub skills_found: i64,
    pub new_skills: i64,
    pub changed_skills: i64,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncReport {
    pub skill_id: String,
    pub skill_name: String,
    pub agent_results: Vec<AgentSyncResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSyncResult {
    pub agent_id: String,
    pub agent_name: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_agents: i64,
    pub active_agents: i64,
    pub total_skills: i64,
    pub total_files: i64,
    pub pending_sync_count: i64,
    pub recent_activities: Vec<ActivityItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityItem {
    pub r#type: String,
    pub message: String,
    pub time: String,
}
