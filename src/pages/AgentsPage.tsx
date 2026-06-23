import { useState, useEffect, useCallback } from "react";
import { getAgents, addAgent, removeAgent, updateAgent } from "../lib/api";
import { useToast } from "../context/ToastContext";
import Card from "../components/ui/Card";
import Button from "../components/ui/Button";
import Modal from "../components/ui/Modal";
import Spinner from "../components/ui/Spinner";
import type { Agent } from "../types";

const USER_VAR = "{{USERNAME}}";

export default function AgentsPage() {
  const { toast } = useToast();
  const [agents, setAgents] = useState<Agent[]>([]);
  const [loading, setLoading] = useState(true);
  const [showAdd, setShowAdd] = useState(false);
  const [name, setName] = useState("");
  const [path, setPath] = useState("");
  const [editingId, setEditingId] = useState<string | null>(null);
  const [editPath, setEditPath] = useState("");

  const load = useCallback(async () => {
    setLoading(true);
    try { setAgents(await getAgents()); }
    catch (e) { console.error(e); }
    finally { setLoading(false); }
  }, []);

  useEffect(() => { load(); }, [load]);

  const handleAdd = async () => {
    if (!name.trim() || !path.trim()) return;
    try {
      await addAgent(name.trim(), path.trim());
      setName(""); setPath("");
      setShowAdd(false);
      toast("Agent 添加成功", "success");
      await load();
    } catch (e) { toast(`添加失败: ${e}`, "error"); }
  };

  const handleToggle = async (agent: Agent) => {
    try {
      await updateAgent(agent.id, agent.name, agent.skills_path, !agent.is_active);
      toast(agent.is_active ? "Agent 同步已关闭" : "Agent 同步已开启", "info");
      await load();
    } catch (e) { toast(`操作失败: ${e}`, "error"); }
  };

  const handleRemove = async (id: string) => {
    if (!confirm("确定删除此 Agent？此操作不可恢复。")) return;
    try {
      await removeAgent(id);
      toast("Agent 已删除", "success");
      await load();
    } catch (e) { toast(`删除失败: ${e}`, "error"); }
  };

  const handleEditPath = async (agent: Agent) => {
    if (!editPath.trim()) return;
    try {
      await updateAgent(agent.id, agent.name, editPath.trim(), agent.is_active);
      setEditingId(null);
      setEditPath("");
      toast("路径已更新", "success");
      await load();
    } catch (e) { toast(`更新失败: ${e}`, "error"); }
  };

  return (
    <div className="max-w-4xl">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-semibold text-gray-800 dark:text-gray-100">Agent 管理</h2>
        <Button variant="primary" size="sm" onClick={() => setShowAdd(true)}>
          + 添加 Agent
        </Button>
      </div>

      <div className="mb-4 bg-blue-50 dark:bg-blue-900/30 border border-blue-200 dark:border-blue-800 rounded-md p-3 text-sm text-blue-700 dark:text-blue-300">
        提示：路径中的 <code className="bg-blue-100 dark:bg-blue-800 px-1 rounded">{USER_VAR}</code> 会自动替换为当前用户名。
      </div>

      <Modal open={showAdd} onClose={() => setShowAdd(false)} title="添加自定义 Agent" width="max-w-md">
        <div className="space-y-3">
          <input
            placeholder="Agent 名称 (如 my-agent)"
            value={name}
            onChange={(e) => setName(e.target.value)}
            className="w-full border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 rounded-md px-3 py-2 text-sm outline-none focus:border-blue-500 text-gray-800 dark:text-gray-200"
          />
          <input
            placeholder={`Skills 目录路径 (如 C:\\Users\\${USER_VAR}\\.my-agent\\skills)`}
            value={path}
            onChange={(e) => setPath(e.target.value)}
            className="w-full border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 rounded-md px-3 py-2 text-sm outline-none focus:border-blue-500 text-gray-800 dark:text-gray-200 font-mono text-xs"
          />
          <div className="flex gap-2 pt-1">
            <Button variant="primary" onClick={handleAdd}>确认添加</Button>
            <Button variant="secondary" onClick={() => setShowAdd(false)}>取消</Button>
          </div>
        </div>
      </Modal>

      {loading ? (
        <div className="flex justify-center py-12"><Spinner size="lg" /></div>
      ) : (
        agents.map((agent) => (
          <Card key={agent.id} className="mb-2 flex items-start gap-4">
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2">
                <span className="font-medium text-gray-800 dark:text-gray-200">{agent.display_name || agent.name}</span>
                <span className={`text-xs px-2 py-0.5 rounded-full ${
                  agent.is_active
                    ? "bg-green-100 dark:bg-green-900/40 text-green-700 dark:text-green-300"
                    : "bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400"
                }`}>
                  {agent.is_active ? "活跃" : "已关闭"}
                </span>
                <span className="text-xs text-gray-400 font-mono">({agent.name})</span>
              </div>

              {editingId === agent.id ? (
                <div className="mt-2 flex gap-2">
                  <input value={editPath} onChange={(e) => setEditPath(e.target.value)}
                    className="flex-1 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 rounded-md px-2 py-1 text-xs font-mono outline-none focus:border-blue-500 text-gray-800 dark:text-gray-200" />
                  <Button size="xs" variant="primary" onClick={() => handleEditPath(agent)}>保存</Button>
                  <Button size="xs" variant="secondary" onClick={() => setEditingId(null)}>取消</Button>
                </div>
              ) : (
                <div className="text-xs text-gray-500 dark:text-gray-400 mt-1 font-mono truncate hover:text-gray-700 dark:hover:text-gray-300 cursor-pointer"
                  onClick={() => { setEditingId(agent.id); setEditPath(agent.skills_path); }} title="点击编辑路径">
                  {agent.skills_path} <span className="ml-1 text-blue-400">(编辑)</span>
                </div>
              )}

              <div className="text-xs text-gray-400 dark:text-gray-500 mt-0.5">绑定 {agent.skills_count} 个 Skill</div>
            </div>

            <div className="flex items-center gap-2 shrink-0 mt-1">
              <div className={`text-xs px-2 py-1 rounded-full ${
                agent.sync_status === "synced" ? "bg-green-100 dark:bg-green-900/40 text-green-700 dark:text-green-300"
                : agent.sync_status === "pending" ? "bg-amber-100 dark:bg-amber-900/40 text-amber-700 dark:text-amber-300"
                : agent.sync_status === "conflict" ? "bg-red-100 dark:bg-red-900/40 text-red-700 dark:text-red-300"
                : "bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400"
              }`}>
                {agent.sync_status === "synced" ? "已同步" : agent.sync_status === "pending" ? "待同步" : agent.sync_status === "conflict" ? "冲突" : "未知"}
              </div>
              <Button size="xs" variant={agent.is_active ? "danger" : "success"} onClick={() => handleToggle(agent)}>
                {agent.is_active ? "关闭同步" : "开启同步"}
              </Button>
              <Button size="xs" variant="danger" onClick={() => handleRemove(agent.id)}>删除</Button>
            </div>
          </Card>
        ))
      )}

      {!loading && agents.length === 0 && (
        <p className="text-gray-400 dark:text-gray-500 text-sm text-center py-12">暂无 Agent，点击上方按钮添加</p>
      )}
    </div>
  );
}