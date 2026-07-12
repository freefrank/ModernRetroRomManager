import { useTranslation } from "react-i18next";
import { useRomStore } from "@/stores/romStore";

// 字节数 → 可读体积文案(仅在 totalSize 有数据时使用)
function formatSize(bytes: number): string {
  if (bytes >= 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  }
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export default function StatusBar() {
  const { t } = useTranslation();
  const stats = useRomStore((state) => state.stats);

  return (
    <footer className="h-7 flex items-center justify-between px-4 bg-bg-secondary border-t border-border-default text-xs text-text-muted">
      <div className="flex items-center gap-4">
        <span>{t("statusBar.romCount", { count: stats.totalRoms })}</span>
        <span>{t("statusBar.scrapedCount", { count: stats.scrapedRoms })}</span>
      </div>
      <div className="flex items-center gap-4">
        {/* 后端暂无文件体积数据(totalSize 恒 0),有数据后自动显示 */}
        {stats.totalSize > 0 && (
          <span>{t("statusBar.storageUsed", { size: formatSize(stats.totalSize) })}</span>
        )}
      </div>
    </footer>
  );
}
