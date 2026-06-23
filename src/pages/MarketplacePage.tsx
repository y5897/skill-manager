import { useState, useEffect, useCallback } from "react";
import { useNavigate } from "react-router-dom";
import { getInstalledSkills, installMarketplaceSkill, uninstallInstalledSkill, updateInstalledSkill } from "../lib/api";
import { useToast } from "../context/ToastContext";
import Button from "../components/ui/Button";
import Card from "../components/ui/Card";
import Spinner from "../components/ui/Spinner";
import MarketplaceCard from "../components/MarketplaceCard";
import type { MarketplaceItem, InstalledSkill } from "../types";

const GITHUB_TOPIC = "skill-manager-skill";

export default function MarketplacePage() {
  const { toast } = useToast();
  const navigate = useNavigate();
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<MarketplaceItem[]>([]);
  const [searching, setSearching] = useState(false);
  const [installed, setInstalled] = useState<InstalledSkill[]>([]);
  const [installingId, setInstallingId] = useState<string | null>(null);
  const [searched, setSearched] = useState(false);

  const loadInstalled = useCallback(async () => {
    try { setInstalled(await getInstalledSkills()); }
    catch { /* ignore */ }
  }, []);

  useEffect(() => { loadInstalled(); }, [loadInstalled]);

  const searchMarket = async () => {
    setSearching(true);
    setSearched(true);
    setResults([]);
    try {
      const q = query.trim();
      const searchQuery = q
        ? `topic:${GITHUB_TOPIC}+${encodeURIComponent(q)}`
        : `topic:${GITHUB_TOPIC}`;

      const res = await fetch(
        `https://api.github.com/search/repositories?q=${searchQuery}&sort=stars&order=desc&per_page=30`,
        { headers: { Accept: "application/vnd.github.v3+json" } }
      );
      if (!res.ok) throw new Error(`GitHub API error: ${res.status}`);
      const data = await res.json();

      const items: MarketplaceItem[] = await Promise.all(
        (data.items || []).map(async (repo: any) => {
          let latestRelease: { tag_name: string; download_url: string } | undefined;

          try {
            const relRes = await fetch(
              `https://api.github.com/repos/${repo.full_name}/releases/latest`,
              { headers: { Accept: "application/vnd.github.v3+json" } }
            );
            if (relRes.ok) {
              const relData = await relRes.json();
              const asset = relData.assets?.[0];
              if (asset) {
                latestRelease = {
                  tag_name: relData.tag_name,
                  download_url: asset.browser_download_url,
                };
              }
            }
          } catch { /* no release */ }

          return {
            name: repo.name,
            slug: repo.name,
            full_name: repo.full_name,
            description: repo.description || "",
            stars: repo.stargazers_count || 0,
            owner: repo.owner?.login || "",
            repo: repo.name,
            default_branch: repo.default_branch || "main",
            html_url: repo.html_url,
            topics: repo.topics || [],
            updated_at: repo.updated_at || "",
            latest_release: latestRelease,
          };
        })
      );

      setResults(items);
    } catch (e) {
      toast(`搜索失败: ${e}`, "error");
    } finally {
      setSearching(false);
    }
  };

  const isInstalled = (slug: string) => installed.find((i) => i.slug === slug);

  const handleInstall = async (item: MarketplaceItem) => {
    if (!item.latest_release) {
      toast("该 Skill 没有 GitHub Release，无法安装", "warning");
      return;
    }
    setInstallingId(item.slug);
    try {
      await installMarketplaceSkill(
        item.latest_release.download_url,
        item.name,
        item.slug,
        item.owner,
        item.repo,
        item.latest_release.tag_name
      );
      toast(`安装成功: ${item.name}`, "success");
      await loadInstalled();
    } catch (e) {
      toast(`安装失败: ${e}`, "error");
    } finally {
      setInstallingId(null);
    }
  };

  const handleUninstall = async (id: string, name: string) => {
    if (!confirm(`确定卸载 ${name}？将从所有 Agent 目录中移除。`)) return;
    try {
      await uninstallInstalledSkill(id);
      toast(`卸载成功: ${name}`, "success");
      await loadInstalled();
    } catch (e) {
      toast(`卸载失败: ${e}`, "error");
    }
  };

  const handleUpdate = async (inst: InstalledSkill) => {
    try {
      const res = await fetch(
        `https://api.github.com/repos/${inst.repo_owner}/${inst.repo_name}/releases/latest`,
        { headers: { Accept: "application/vnd.github.v3+json" } }
      );
      if (!res.ok) throw new Error("无法获取最新版本");
      const data = await res.json();
      const asset = data.assets?.[0];
      if (!asset) throw new Error("没有可下载的 Release 附件");

      await updateInstalledSkill(inst.id, asset.browser_download_url, data.tag_name);
      toast(`更新成功: ${inst.skill_name} (${data.tag_name})`, "success");
      await loadInstalled();
    } catch (e) {
      toast(`更新失败: ${e}`, "error");
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") searchMarket();
  };

  return (
    <div className="max-w-5xl">
      <div className="flex items-center justify-between mb-4">
        <h2 className="text-xl font-semibold text-gray-800 dark:text-gray-100">📦 Skill 市场</h2>
      </div>

      <div className="flex gap-2 mb-4">
        <input
          placeholder="搜索 Skill (留空浏览全部)..."
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={handleKeyDown}
          className="flex-1 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 rounded-md px-3 py-2 text-sm outline-none focus:border-blue-500 text-gray-800 dark:text-gray-200"
        />
        <Button variant="primary" onClick={searchMarket} loading={searching}>
          搜索
        </Button>
      </div>

      {/* ─── 已安装列表 ─── */}
      {installed.length > 0 && (
        <div className="mb-6">
          <h3 className="text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
            已安装 ({installed.length})
          </h3>
          <div className="space-y-1">
            {installed.map((inst) => (
              <Card key={inst.id} className="flex items-center gap-3">
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className="text-sm font-medium text-gray-800 dark:text-gray-200">{inst.skill_name}</span>
                    <span className="text-xs text-gray-400 font-mono">{inst.slug}</span>
                    <span className="text-xs px-1.5 py-0.5 rounded bg-green-50 dark:bg-green-900/30 text-green-700 dark:text-green-300">
                      {inst.remote_version}
                    </span>
                  </div>
                  <div className="text-xs text-gray-400 mt-0.5">
                    {inst.repo_owner}/{inst.repo_name}
                  </div>
                </div>
                <div className="flex gap-1">
                  <Button size="xs" variant="primary" onClick={() => handleUpdate(inst)}>
                    更新
                  </Button>
                  <Button size="xs" variant="ghost" onClick={() => navigate(`/skills/${inst.skill_id}`)}>
                    查看
                  </Button>
                  <Button size="xs" variant="danger" onClick={() => handleUninstall(inst.id, inst.skill_name)}>
                    卸载
                  </Button>
                </div>
              </Card>
            ))}
          </div>
        </div>
      )}

      {/* ─── 搜索结果 ─── */}
      {searching ? (
        <div className="flex justify-center py-12"><Spinner size="lg" /></div>
      ) : searched ? (
        results.length === 0 ? (
          <div className="text-center py-16 text-gray-400 dark:text-gray-500">
            <div className="text-3xl mb-2">🏪</div>
            <p className="text-sm">未找到任何 Skill</p>
            <p className="text-xs mt-1">目前市场中的 Skill 还不多，你可以创建自己的 Skill 并打上 <code className="bg-gray-100 dark:bg-gray-700 px-1 rounded">skill-manager-skill</code> Topic</p>
          </div>
        ) : (
          <div className="space-y-2">
            <p className="text-xs text-gray-400 mb-1">共 {results.length} 个结果</p>
            {results.map((item) => (
              <MarketplaceCard
                key={item.full_name}
                item={item}
                isInstalled={!!isInstalled(item.slug)}
                installedVersion={isInstalled(item.slug)?.remote_version}
                installing={installingId === item.slug}
                onInstall={handleInstall}
              />
            ))}
          </div>
        )
      ) : (
        <div className="text-center py-16 text-gray-400 dark:text-gray-500">
          <div className="text-4xl mb-3">🔍</div>
          <p className="text-sm">在上方搜索框中输入关键词，或留空点击搜索浏览全部 Skill</p>
          <p className="text-xs mt-2">
            数据来源: GitHub Topic <code className="bg-gray-100 dark:bg-gray-700 px-1 rounded">skill-manager-skill</code>
          </p>
        </div>
      )}
    </div>
  );
}