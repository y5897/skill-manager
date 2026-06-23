import { invoke } from "@tauri-apps/api/core";
import type {
  Agent,
  Skill,
  SkillDetail,
  SkillFile,
  ScanResult,
  SyncReport,
  DashboardStats,
} from "../types";

// ─── Agent ───
export const getAgents = () => invoke<Agent[]>("get_agents");
export const addAgent = (name: string, skillsPath: string) =>
  invoke<Agent>("add_agent", { name, skillsPath });
export const removeAgent = (id: string) => invoke<void>("remove_agent", { id });
export const updateAgent = (id: string, name: string, skillsPath: string, isActive: boolean) =>
  invoke<Agent>("update_agent", { id, name, skillsPath, isActive });

// ─── Scanner ───
export const scanAllAgents = () => invoke<ScanResult>("scan_all_agents");
export const getScanHistory = () => invoke<string[]>("get_scan_history");

// ─── Skill ───
export const getSkills = (agentId?: string, search?: string) =>
  invoke<Skill[]>("get_skills", { agentId, search });
export const getSkillDetail = (id: string) =>
  invoke<SkillDetail>("get_skill_detail", { id });
export const createSkill = (name: string, description: string) =>
  invoke<Skill>("create_skill", { name, description });
export const updateSkillMd = (id: string, content: string) =>
  invoke<Skill>("update_skill_md", { id, content });
export const deleteSkill = (id: string) => invoke<void>("delete_skill", { id });

// ─── Files ───
export const readSkillMd = (id: string) => invoke<string>("read_skill_md", { id });
export const uploadFiles = (skillId: string, filePaths: string[]) =>
  invoke<SkillFile[]>("upload_files", { skillId, filePaths });
export const removeFile = (skillId: string, relativePath: string) =>
  invoke<void>("remove_file", { skillId, relativePath });
export const readFileContent = (skillId: string, relativePath: string) =>
  invoke<string>("read_file_content", { skillId, relativePath });

// ─── Sync ───
export const syncSkillToAgents = (skillId: string) =>
  invoke<SyncReport>("sync_skill_to_agents", { skillId });
export const syncAllSkills = () => invoke<SyncReport[]>("sync_all_skills");
export const getDashboardStats = () => invoke<DashboardStats>("get_dashboard_stats");


