import { CheckCircle2, RefreshCw, ScanSearch } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useRomStore } from "@/stores/romStore";

export default function RomScanProgress() {
  const { t } = useTranslation();
  const progress = useRomStore((state) => state.scanProgress);

  if (!progress) return null;
  const percent = progress.total > 0
    ? Math.min(100, Math.round((progress.current / progress.total) * 100))
    : 0;

  return (
    <div className="relative z-30 shrink-0 border-t border-border-default bg-bg-secondary/95 px-3 py-1.5 backdrop-blur-xl">
      <div className="flex items-center gap-2 text-xs">
        {progress.finished ? (
          <CheckCircle2 className="h-3.5 w-3.5 shrink-0 text-accent-success" />
        ) : progress.changed ? (
          <ScanSearch className="h-3.5 w-3.5 shrink-0 text-accent-primary" />
        ) : (
          <RefreshCw className="h-3.5 w-3.5 shrink-0 animate-spin text-accent-secondary" />
        )}
        <span className="shrink-0 font-medium text-text-primary">
          {t(`scan.mode.${progress.mode}`)}
        </span>
        <span className="min-w-0 flex-1 truncate text-text-secondary">
          {progress.system || progress.message}
        </span>
        <span className="shrink-0 tabular-nums text-text-muted">
          {progress.total > 0 ? `${progress.current}/${progress.total} · ${percent}%` : t("scan.preparing")}
        </span>
      </div>
      <div className="mt-1 h-1 overflow-hidden rounded-full bg-bg-tertiary">
        <div
          className="h-full bg-accent-primary transition-[width] duration-[var(--motion-fast)] ease-[var(--motion-easing)]"
          style={{ width: progress.total > 0 ? `${percent}%` : "8%" }}
        />
      </div>
    </div>
  );
}
