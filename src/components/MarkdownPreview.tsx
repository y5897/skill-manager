import { useMemo } from "react";

const heading = (text: string) => {
  const m = text.match(/^(#{1,6})\s+(.+)/);
  if (!m) return null;
  const level = m[1].length;
  const size = ["text-2xl", "text-xl", "text-lg", "text-base", "text-sm", "text-xs"][level - 1];
  return `<h${level} class="${size} font-bold text-gray-800 dark:text-gray-100 mt-4 mb-2">${m[2]}</h${level}>`;
};

const codeBlock = (text: string) => {
  const m = text.match(/^```(\w*)\n([\s\S]*?)```/);
  if (!m) return null;
  return `<pre class="bg-gray-100 dark:bg-gray-900 rounded-md p-3 my-2 overflow-x-auto text-xs font-mono text-gray-700 dark:text-gray-300 border border-gray-200 dark:border-gray-700"><code>${escapeHtml(m[2])}</code></pre>`;
};

const inlineCode = (text: string) =>
  text.replace(/`([^`]+)`/g, '<code class="bg-gray-100 dark:bg-gray-900 px-1 rounded text-xs font-mono text-pink-600 dark:text-pink-400">$1</code>');

const boldItalic = (text: string) =>
  text.replace(/\*\*\*(.+?)\*\*\*/g, '<strong><em>$1</em></strong>')
      .replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>')
      .replace(/\*(.+?)\*/g, '<em>$1</em>');

const link = (text: string) =>
  text.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" class="text-blue-600 dark:text-blue-400 underline hover:no-underline" target="_blank" rel="noopener">$1</a>');

const listItem = (text: string) => {
  const lines = text.split("\n");
  const rendered = lines.map((line) => {
    const m2 = line.match(/^(\s*)[-*]\s+(.+)/);
    if (m2) return `${m2[1]}• ${m2[2]}`;
    const m3 = line.match(/^(\s*)\d+\.\s+(.+)/);
    if (m3) return `${m3[1]}${m3[2]}`;
    return line;
  });
  return rendered.join("<br/>");
};

const horizontalRule = (text: string) =>
  text.match(/^---+\s*$/) ? '<hr class="my-4 border-gray-300 dark:border-gray-600"/>' : null;

const paragraph = (text: string) => {
  if (!text.trim()) return "";
  let html = escapeHtml(text);
  html = boldItalic(html);
  html = inlineCode(html);
  html = link(html);
  return `<p class="text-sm text-gray-600 dark:text-gray-300 leading-relaxed mb-2">${html}</p>`;
};

function escapeHtml(s: string) {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

function renderLine(line: string): string {
  return heading(line)
    ?? codeBlock(line)
    ?? horizontalRule(line)
    ?? listItem(line)
    ?? paragraph(line);
}

interface MarkdownPreviewProps {
  content: string;
  className?: string;
}

export default function MarkdownPreview({ content, className = "" }: MarkdownPreviewProps) {
  const html = useMemo(() => {
    const blocks = content.split(/(```[\s\S]*?```)/g);
    return blocks
      .map((block) => {
        if (block.startsWith("```")) return renderLine(block);
        return block.split("\n").map(renderLine).join("");
      })
      .join("");
  }, [content]);

  return (
    <div
      className={`prose prose-sm max-w-none dark:prose-invert ${className}`}
      dangerouslySetInnerHTML={{ __html: html || "<p class='text-gray-400 italic'>暂无内容</p>" }}
    />
  );
}