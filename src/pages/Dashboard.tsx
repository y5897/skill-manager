import { useState, useEffect, useCallback } from "react";
import { getDashboardStats, scanAllAgents, syncAllSkills } from "../lib/api";
import { useToast } from "../context/ToastContext";
import Card from "../components/ui/Card";
import Button from "../components/ui/Button";
import Spinner from "../components/ui/Spinner";
import type { DashboardStats } from "../types";

export default function Dashboard() {
  const { toast } = useToast();
  const [stats, setStats] = useState<DashboardStats | null>(null);
  const [scanning, setScanning] = useState(false);
  const [syncing, setSyncing] = useState(false);

  const load = useCallback(async () => {
    try {
      const s = await getDashboardStats();
      setStats(s);
    } catch (e) {
      console.error(e);
    }
  }, []);

  useEffect(() => { load(); }, [load]);

  const handleScan = async () => {
    setScanning(true);
    try {
      const r = await scanAllAgents();
      toast(`扫描完成: ${r.agents_scanned} 个 Agent, ${r.skills_found} 个 Skill, 新增 ${r.new_skills}, 变更 ${r.changed_skills}`, "success");
      await load();
    } catch (e) {
      toast(`扫描失败: ${e}`, "error");
    } finally {
      setScanning(false);
    }
  };

  const handleSync = async () => {
    setSyncing(true);
    try {
      const reports = await syncAllSkills();
      const ok = reports.filter((r) => r.agent_results.every((a) => a.success)).length;
      toast(`同步完成: ${ok}/${reports.length} 个 Skill 同步成功`, ok === reports.length ? "success" : "warning");
      await load();
    } catch (e) {
      toast(`同步失败: ${e}`, "error");
    } finally {
      setSyncing(false);
    }
  };

  if (!stats) {
    return <div className="flex items-center gap-2 text-gray-500"><Spinner />加载中...</div>;
  }

  const statCards = [
    { label: "Agent 总数", value: stats.total_agents, sub: `${stats.active_agents} 个活跃` },
    { label: "Skill 总数", value: stats.total_skills },
    { label: "文件总数", value: stats.total_files },
    { label: "待同步", value: stats.pending_sync_count, warn: stats.pending_sync_count > 0 },
  ];

  return (
    <div className="max-w-4xl">
      <h2 className="text-xl font-semibold text-gray-800 dark:text-gray-100 mb-4">仪表盘</h2>

      <div className="grid grid-cols-4 gap-4 mb-6">
        {statCards.map(({ label, value, sub, warn }) => (
          <Card key={label}>
            <div className="text-sm text-gray-500 dark:text-gray-400">{label}</div>
            <div className={`text-2xl font-bold mt-1 ${warn ? "text-amber-500 dark:text-amber-400" : "text-gray-800 dark:text-gray-100"}`}>
              {value}
            </div>
            {sub && <div className="text-xs text-gray-400 dark:text-gray-500 mt-1">{sub}</div>}
          </Card>
        ))}
      </div>

      <div className="flex gap-3 mb-6">
        <Button variant="primary" onClick={handleScan} loading={scanning}>
          🔍 扫描所有 Agent
        </Button>
        <Button variant="success" onClick={handleSync} loading={syncing}>
          🔄 同步全部 Skill
        </Button>
      </div>

      <Card>
        <h3 className="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-3">最近活动</h3>
        {stats.recent_activities.length === 0 ? (
          <p className="text-gray-400 dark:text-gray-500 text-sm">暂无活动记录</p>
        ) : (
          <ul className="space-y-2">
            {stats.recent_activities.map((a, i) => (
              <li key={i} className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400">
                <span>
                  {a.type === "create" && "🟢"}
                  {a.type === "update" && "🔵"}
                  {a.type === "delete" && "🔴"}
                  {a.type === "sync" && "🔄"}
                </span>
                <span>{a.message}</span>
                <span className="text-gray-400 dark:text-gray-500 text-xs ml-auto">{a.time}</span>
              </li>
            ))}
          </ul>
        )}
      </Card>
    </div>
  );
}