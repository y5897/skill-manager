import type { MarketplaceItem } from "../types";

interface MarketplaceCardProps {
  item: MarketplaceItem;
  isInstalled: boolean;
  installedVersion?: string;
  installing: boolean;
  onInstall: (item: MarketplaceItem) => void;
}

export default function MarketplaceCard({ item, isInstalled, installedVersion, installing, onInstall }: MarketplaceCardProps) {
  return (
    <div className="bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg p-4 hover:border-blue-300 dark:hover:border-blue-500 hover:shadow-sm transition-all">
      <div className="flex items-start justify-between gap-4">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <span className="font-medium text-gray-800 dark:text-gray-200 truncate">{item.name}</span>
            <span className="text-xs px-1.5 py-0.5 rounded bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400 font-mono">
              {item.slug}
            </span>
          </div>
          <div className="text-xs text-gray-400 dark:text-gray-500 mt-0.5">
            {item.full_name}
          </div>
          {item.description && (
            <p className="text-xs text-gray-500 dark:text-gray-400 mt-1.5 line-clamp-2">{item.description}</p>
          )}
          <div className="flex items-center gap-3 mt-2">
            <span className="text-xs text-amber-600 dark:text-amber-400">★ {item.stars}</span>
            {item.topics.length > 0 && (
              <div className="flex gap-1 flex-wrap">
                {item.topics.filter(t => t !== "skill-manager-skill").slice(0, 3).map((t) => (
                  <span key={t} className="text-xs px-1.5 py-0.5 rounded bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400">
                    {t}
                  </span>
                ))}
              </div>
            )}
          </div>
        </div>
        <div className="shrink-0">
          {isInstalled ? (
            <span className="inline-flex items-center gap-1 px-2.5 py-1 rounded text-xs bg-green-50 dark:bg-green-900/30 text-green-700 dark:text-green-300">
              已安装
              {installedVersion && <span className="text-green-500">({installedVersion})</span>}
            </span>
          ) : (
            <button
              onClick={() => onInstall(item)}
              disabled={installing || !item.latest_release}
              className="px-3 py-1.5 rounded text-xs bg-blue-600 text-white hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer"
            >
              {installing ? "安装中..." : item.latest_release ? "安装" : "无 Release"}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}