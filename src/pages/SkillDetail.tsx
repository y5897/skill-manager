import { useState, useEffect, useCallback, useRef } from "react";
import { useParams, useNavigate } from "react-router-dom";
import { getSkillDetail, readSkillMd, updateSkillMd, uploadFiles, removeFile, readFileContent, syncSkillToAgents } from "../lib/api";
import { useToast } from "../context/ToastContext";
import Card from "../components/ui/Card";
import Button from "../components/ui/Button";
import Spinner from "../components/ui/Spinner";
import FileTree from "../components/ui/FileTree";
import MarkdownPreview from "../components/MarkdownPreview";
import type { SkillDetail as SkillDetailType, SkillFile } from "../types";

export default function SkillDetail() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { toast } = useToast();
  const [skill, setSkill] = useState<SkillDetailType | null>(null);
  const [mdContent, setMdContent] = useState("");
  const [editing, setEditing] = useState(false);
  const [editContent, setEditContent] = useState("");
  const [syncing, setSyncing] = useState(false);
  const [previewFile, setPreviewFile] = useState<SkillFile | null>(null);
  const [fileContent, setFileContent] = useState("");
  const [dragOver, setDragOver] = useState(false);
  const dropRef = useRef<HTMLDivElement>(null);

  const load = useCallback(async () => {
    if (!id) return;
    try {
      const s = await getSkillDetail(id);
      setSkill(s);
      const content = await readSkillMd(id);
      setMdContent(content);
    } catch (e) { console.error(e); }
  }, [id]);

  useEffect(() => { load(); }, [load]);

  const handleEdit = () => { setEditContent(mdContent); setEditing(true); };

  const handleSave = async () => {
    if (!id) return;
    try {
      await updateSkillMd(id, editContent);
      setMdContent(editContent);
      setEditing(false);
      toast("SKILL.md 已保存", "success");
      await load();
    } catch (e) { toast(`保存失败: ${e}`, "error"); }
  };

  const handleSync = async () => {
    if (!id) return;
    setSyncing(true);
    try {
      const report = await syncSkillToAgents(id);
      const ok = report.agent_results.filter((a) => a.success).length;
      toast(`同步完成: ${ok}/${report.agent_results.length} 个 Agent`, ok === report.agent_results.length ? "success" : "warning");
      await load();
    } catch (e) { toast(`同步失败: ${e}`, "error"); }
    finally { setSyncing(false); }
  };

  const handleUpload = async (paths: string[]) => {
    if (!id || !paths.length) return;
    try {
      const result = await uploadFiles(id, paths);
      toast(`上传成功: ${result.length} 个文件`, "success");
      await load();
    } catch (e) { toast(`上传失败: ${e}`, "error"); }
  };

  const handleSelectFiles = () => {
    const input = document.createElement("input");
    input.type = "file";
    input.multiple = true;
    input.onchange = async () => {
      if (!input.files?.length) return;
      const paths = Array.from(input.files).map((f) => (f as any).path);
      await handleUpload(paths);
    };
    input.click();
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragOver(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragOver(false);
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setDragOver(false);
    if (!e.dataTransfer.files?.length) return;
    const paths = Array.from(e.dataTransfer.files).map((f) => (f as any).path);
    await handleUpload(paths);
  };

  const handleDeleteFile = async (relativePath: string) => {
    if (!id || !confirm(`确定删除 ${relativePath}？`)) return;
    try {
      await removeFile(id, relativePath);
      toast("文件已删除", "success");
      if (previewFile?.relative_path === relativePath) setPreviewFile(null);
      await load();
    } catch (e) { toast(`删除失败: ${e}`, "error"); }
  };

  const handleViewFile = async (f: SkillFile) => {
    if (!id || f.is_directory) return;
    if (previewFile?.relative_path === f.relative_path) {
      setPreviewFile(null);
      return;
    }
    try {
      const content = await readFileContent(id, f.relative_path);
      setFileContent(content);
      setPreviewFile(f);
    } catch (e) {
      setFileContent(`(无法预览: ${e})`);
      setPreviewFile(f);
    }
  };

  if (!skill) return <div className="flex items-center gap-2 text-gray-500 py-8"><Spinner size="lg" />加载中...</div>;

  return (
    <div className="max-w-5xl">
      <div className="flex items-center gap-2 mb-4">
        <button onClick={() => navigate("/skills")} className="text-sm text-blue-600 dark:text-blue-400 hover:underline cursor-pointer">&larr; 返回</button>
        <h2 className="text-xl font-semibold text-gray-800 dark:text-gray-100">{skill.name}</h2>
        <span className="text-xs text-gray-400 dark:text-gray-500 font-mono">{skill.slug}</span>
        {skill.description && <span className="text-xs text-gray-500 dark:text-gray-400 ml-2">— {skill.description}</span>}
      </div>

      <div className="grid grid-cols-3 gap-4 mb-4">
        <Card>
          <div className="text-xs text-gray-500 dark:text-gray-400">绑定 Agent</div>
          <div className="text-lg font-bold text-gray-800 dark:text-gray-100">{skill.agents.length}</div>
          <div className="text-xs text-gray-400 dark:text-gray-500 mt-1">
            {skill.agents.map((a) => a.display_name || a.name).join(", ") || "无"}
          </div>
        </Card>
        <Card>
          <div className="text-xs text-gray-500 dark:text-gray-400">文件数</div>
          <div className="text-lg font-bold text-gray-800 dark:text-gray-100">{skill.files.length}</div>
        </Card>
        <Card className="flex flex-col justify-center">
          <Button variant="success" onClick={handleSync} loading={syncing}>
            🔄 同步到所有 Agent
          </Button>
        </Card>
      </div>

      <div className="grid grid-cols-12 gap-4 mb-4">
        <div className="col-span-7">
          <Card>
            <div className="flex items-center justify-between mb-2">
              <h3 className="text-sm font-semibold text-gray-700 dark:text-gray-300">SKILL.md</h3>
              {!editing && (
                <div className="flex gap-1">
                  <Button size="xs" variant="secondary" onClick={() => { setEditing(false); }}>预览</Button>
                  <Button size="xs" variant="secondary" onClick={handleEdit}>编辑</Button>
                </div>
              )}
            </div>
            {editing ? (
              <div className="space-y-2">
                <textarea
                  value={editContent}
                  onChange={(e) => setEditContent(e.target.value)}
                  rows={20}
                  className="w-full border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 rounded-md px-3 py-2 text-sm font-mono outline-none focus:border-blue-500 resize-none text-gray-800 dark:text-gray-200"
                />
                <div className="flex gap-2">
                  <Button variant="primary" onClick={handleSave}>保存</Button>
                  <Button variant="secondary" onClick={() => setEditing(false)}>取消</Button>
                </div>
              </div>
            ) : (
              <div className="max-h-[480px] overflow-auto scrollbar-thin bg-gray-50 dark:bg-gray-900 rounded p-3">
                <MarkdownPreview content={mdContent} />
              </div>
            )}
          </Card>
        </div>

        <div className="col-span-5">
          <Card>
            <div className="flex items-center justify-between mb-2">
              <h3 className="text-sm font-semibold text-gray-700 dark:text-gray-300">文件管理</h3>
              <Button size="xs" variant="primary" onClick={handleSelectFiles}>上传文件</Button>
            </div>

            <div
              ref={dropRef}
              onDragOver={handleDragOver}
              onDragLeave={handleDragLeave}
              onDrop={handleDrop}
              className={`mb-2 border-2 border-dashed rounded-md p-4 text-center text-xs transition-colors cursor-pointer ${
                dragOver
                  ? "border-blue-400 bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400"
                  : "border-gray-300 dark:border-gray-600 text-gray-400 dark:text-gray-500 hover:border-gray-400"
              }`}
              onClick={handleSelectFiles}
            >
              {dragOver ? "释放文件以上传" : "拖拽文件到此处上传"}
            </div>

            <div className="max-h-[400px] overflow-auto scrollbar-thin">
              <FileTree
                files={skill.files}
                onSelect={handleViewFile}
                onDelete={handleDeleteFile}
                selected={previewFile?.relative_path ?? null}
              />
            </div>
          </Card>
        </div>
      </div>

      {previewFile && (
        <Card>
          <div className="flex items-center justify-between mb-2">
            <h3 className="text-sm font-semibold text-gray-700 dark:text-gray-300">
              预览: <span className="font-mono">{previewFile.relative_path}</span>
            </h3>
            <Button size="xs" variant="ghost" onClick={() => setPreviewFile(null)}>关闭</Button>
          </div>
          {previewFile.relative_path.match(/\.(png|jpg|jpeg|gif|svg|webp|ico)/i) ? (
            <div className="flex justify-center bg-gray-100 dark:bg-gray-800 rounded p-4">
              <div className="text-sm text-gray-400">图片预览需要文件路径，请在 Agent 目录中查看原始文件</div>
            </div>
          ) : (
            <pre className="text-xs text-gray-600 dark:text-gray-300 whitespace-pre-wrap font-mono bg-gray-50 dark:bg-gray-900 rounded p-3 max-h-96 overflow-auto border border-gray-200 dark:border-gray-700">
              {fileContent}
            </pre>
          )}
        </Card>
      )}
    </div>
  );
}