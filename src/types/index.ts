export interface Agent {
  id: string;
  name: string;
  display_name: string;
  skills_path: string;
  is_active: boolean;
  skills_count: number;
  sync_status: SyncStatus;
  created_at: string;
}

export interface Skill {
  id: string;
  name: string;
  slug: string;
  description: string;
  agents_count: number;
  files_count: number;
  created_at: string;
  updated_at: string;
}

export interface SkillDetail extends Skill {
  files: SkillFile[];
  agents: AgentBrief[];
}

export interface SkillFile {
  id: string;
  relative_path: string;
  file_name: string;
  content_hash: string;
  file_size: number;
  is_directory: boolean;
  updated_at: string;
}

export interface AgentBrief {
  id: string;
  name: string;
  display_name: string;
  sync_status: SyncStatus;
}

export type SyncStatus = "synced" | "pending" | "conflict" | "unknown";

export interface ScanResult {
  agents_scanned: number;
  skills_found: number;
  new_skills: number;
  changed_skills: number;
  errors: string[];
}

export interface SyncReport {
  skill_id: string;
  skill_name: string;
  agent_results: AgentSyncResult[];
}

export interface AgentSyncResult {
  agent_id: string;
  agent_name: string;
  success: boolean;
  error?: string;
}

export interface DashboardStats {
  total_agents: number;
  active_agents: number;
  total_skills: number;
  total_files: number;
  pending_sync_count: number;
  recent_activities: ActivityItem[];
}

export interface ActivityItem {
  type: "create" | "update" | "delete" | "sync";
  message: string;
  time: string;
}
