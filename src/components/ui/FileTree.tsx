import { useState, type ReactNode } from "react";
import type { SkillFile } from "../../types";

interface TreeNode {
  name: string;
  path: string;
  isDir: boolean;
  file?: SkillFile;
  children: TreeNode[];
}

function buildTree(files: SkillFile[]): TreeNode[] {
  const root: TreeNode[] = [];
  const map = new Map<string, TreeNode>();

  const sorted = [...files].sort((a, b) => a.relative_path.localeCompare(b.relative_path));

  for (const f of sorted) {
    const parts = f.relative_path.replace(/\\/g, "/").split("/");
    let current = root;
    let acc = "";
    for (let i = 0; i < parts.length; i++) {
      const part = parts[i];
      acc = acc ? `${acc}/${part}` : part;
      const isLast = i === parts.length - 1;
      let node = map.get(acc);
      if (!node) {
        node = {
          name: part,
          path: acc,
          isDir: !isLast || f.is_directory,
          file: isLast && !f.is_directory ? f : undefined,
          children: [],
        };
        map.set(acc, node);
        current.push(node);
      }
      current = node.children;
    }
  }
  return root;
}

function FileTreeNode({
  node,
  depth,
  onSelect,
  onDelete,
  selected,
}: {
  node: TreeNode;
  depth: number;
  onSelect: (f: SkillFile) => void;
  onDelete: (path: string) => void;
  selected: string | null;
}) {
  const [expanded, setExpanded] = useState(true);

  return (
    <div>
      <div
        className={`flex items-center gap-1.5 px-2 py-1 rounded text-sm hover:bg-gray-50 dark:hover:bg-gray-700 cursor-pointer group ${
          node.file && selected === node.file.relative_path ? "bg-blue-50 dark:bg-blue-900/30" : ""
        }`}
        style={{ paddingLeft: `${depth * 16 + 8}px` }}
        onClick={() => {
          if (node.isDir) setExpanded(!expanded);
          else if (node.file) onSelect(node.file);
        }}
      >
        {node.isDir ? (
          <span className="text-xs text-gray-400 w-4">{expanded ? "▾" : "▸"}</span>
        ) : (
          <span className="text-xs text-gray-400 w-4">
            {node.name.endsWith(".md") ? "📝" : node.name.match(/\.(png|jpg|jpeg|gif|svg|webp)/i) ? "🖼" : "📄"}
          </span>
        )}
        <span className="text-gray-700 dark:text-gray-300 truncate flex-1">{node.name}</span>
        {node.file && (
          <span className="text-xs text-gray-400">
            {node.file.file_size > 1024 ? `${(node.file.file_size / 1024).toFixed(1)} KB` : `${node.file.file_size} B`}
          </span>
        )}
        {node.file && (
          <button
            onClick={(e) => { e.stopPropagation(); onDelete(node.path); }}
            className="text-red-400 hover:text-red-600 opacity-0 group-hover:opacity-100 text-xs cursor-pointer"
          >
            删除
          </button>
        )}
      </div>
      {node.isDir && expanded && node.children.map((child) => (
        <FileTreeNode key={child.path} node={child} depth={depth + 1} onSelect={onSelect} onDelete={onDelete} selected={selected} />
      ))}
    </div>
  );
}

interface FileTreeProps {
  files: SkillFile[];
  onSelect: (f: SkillFile) => void;
  onDelete: (path: string) => void;
  selected: string | null;
}

export default function FileTree({ files, onSelect, onDelete, selected }: FileTreeProps) {
  const tree = buildTree(files);
  return (
    <div className="border border-gray-200 dark:border-gray-700 rounded-md bg-gray-50 dark:bg-gray-900 p-1">
      {tree.length === 0 ? (
        <p className="text-gray-400 text-sm text-center py-4">暂无文件</p>
      ) : (
        tree.map((node) => (
          <FileTreeNode key={node.path} node={node} depth={0} onSelect={onSelect} onDelete={onDelete} selected={selected} />
        ))
      )}
    </div>
  );
}