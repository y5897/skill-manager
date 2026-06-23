import { useState, useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { getSkills, createSkill, deleteSkill, syncSkillToAgents } from "../lib/api";
import { useToast } from "../context/ToastContext";
import Card from "../components/ui/Card";
import Button from "../components/ui/Button";
import Modal from "../components/ui/Modal";
import Spinner from "../components/ui/Spinner";
import type { Skill } from "../types";

export default function SkillsPage() {
  const navigate = useNavigate();
  const { toast } = useToast();
  const [skills, setSkills] = useState<Skill[]>([]);
  const [loading, setLoading] = useState(true);
  const [search, setSearch] = useState("");
  const [showCreate, setShowCreate] = useState(false);
  const [name, setName] = useState("");
  const [desc, setDesc] = useState("");

  const load = useCallback(async () => {
    setLoading(true);
    try { setSkills(await getSkills(undefined, search || undefined)); }
    catch (e) { console.error(e); }
    finally { setLoading(false); }
  }, [search]);

  useEffect(() => { load(); }, [load]);

  const handleCreate = async () => {
    if (!name.trim()) return;
    try {
      await createSkill(name.trim(), desc.trim());
      setName(""); setDesc("");
      setShowCreate(false);
      toast("Skill 创建成功", "success");
      await load();
    } catch (e) { toast(`创建失败: ${e}`, "error"); }
  };

  const handleDelete = async (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (!confirm("确定删除此 Skill？将从所有 Agent 目录中移除。")) return;
    try {
      await deleteSkill(id);
      toast("Skill 已删除", "success");
      await load();
    } catch (e) { toast(`删除失败: ${e}`, "error"); }
  };

  const handleSync = async (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      const report = await syncSkillToAgents(id);
      const ok = report.agent_results.filter((a) => a.success).length;
      toast(`同步完成: ${ok}/${report.agent_results.length} 个 Agent`, ok === report.agent_results.length ? "success" : "warning");
    } catch (e) { toast(`同步失败: ${e}`, "error"); }
  };

  return (
    <div className="max-w-4xl">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-semibold text-gray-800 dark:text-gray-100">Skill 资源库</h2>
        <Button variant="primary" size="sm" onClick={() => setShowCreate(true)}>
          + 新建 Skill
        </Button>
      </div>

      <input
        placeholder="搜索 Skill..."
        value={search} onChange={(e) => setSearch(e.target.value)}
        className="w-full border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 rounded-md px-3 py-2 text-sm outline-none focus:border-blue-500 text-gray-800 dark:text-gray-200 mb-4"
      />

      <Modal open={showCreate} onClose={() => setShowCreate(false)} title="新建 Skill" width="max-w-md">
        <div className="space-y-3">
          <input
            placeholder="Skill 名称"
            value={name} onChange={(e) => setName(e.target.value)}
            className="w-full border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 rounded-md px-3 py-2 text-sm outline-none focus:border-blue-500 text-gray-800 dark:text-gray-200"
          />
          <textarea
            placeholder="描述（可选）"
            value={desc} onChange={(e) => setDesc(e.target.value)}
            rows={3}
            className="w-full border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 rounded-md px-3 py-2 text-sm outline-none focus:border-blue-500 text-gray-800 dark:text-gray-200 resize-none"
          />
          <div className="flex gap-2 pt-1">
            <Button variant="primary" onClick={handleCreate}>确认创建</Button>
            <Button variant="secondary" onClick={() => setShowCreate(false)}>取消</Button>
          </div>
        </div>
      </Modal>

      <div className="space-y-2">
        {loading ? (
          <div className="flex justify-center py-12"><Spinner size="lg" /></div>
        ) : (
          skills.map((skill) => (
            <Card key={skill.id} hover onClick={() => navigate(`/skills/${skill.id}`)}>
              <div className="flex items-center gap-4">
                <div className="flex-1 min-w-0">
                  <div className="font-medium text-gray-800 dark:text-gray-200">{skill.name}</div>
                  <div className="text-xs text-gray-400 dark:text-gray-500 mt-0.5 font-mono">{skill.slug}</div>
                  {skill.description && (
                    <div className="text-xs text-gray-500 dark:text-gray-400 mt-1 line-clamp-1">{skill.description}</div>
                  )}
                </div>
                <div className="text-xs text-gray-500 dark:text-gray-400 whitespace-nowrap">{skill.agents_count} 个 Agent</div>
                <div className="text-xs text-gray-500 dark:text-gray-400 whitespace-nowrap">{skill.files_count} 个文件</div>
                <div className="text-xs text-gray-400 dark:text-gray-500 whitespace-nowrap">{skill.updated_at}</div>
                <Button size="xs" variant="primary" onClick={(e) => handleSync(skill.id, e)}>同步</Button>
                <Button size="xs" variant="danger" onClick={(e) => handleDelete(skill.id, e)}>删除</Button>
              </div>
            </Card>
          ))
        )}

        {!loading && skills.length === 0 && (
          <p className="text-gray-400 dark:text-gray-500 text-sm text-center py-8">
            {search ? "未搜索到匹配的 Skill" : "暂无 Skill，请先扫描或创建"}
          </p>
        )}
      </div>
    </div>
  );
}