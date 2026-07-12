import { useTranslation } from "react-i18next";
import { AlertTriangle, Search } from "lucide-react";
import type { MatchProgress } from "@/stores/cnRomToolsStore";
import { Button } from "@/components/ui";
import ExportPanel, { type ExportFormat } from "./ExportPanel";

interface Props {
  totalCount: number;
  missingCount: number;
  isFixing: boolean;
  matchProgress: MatchProgress | null;
  isExporting: boolean;
  onMatch: () => void;
  onExport: (format: ExportFormat) => void;
}

/** 结果表底栏:统计信息 + 匹配英文名按钮 + 导出面板 */
export default function MatchToolbar({
  totalCount,
  missingCount,
  isFixing,
  matchProgress,
  isExporting,
  onMatch,
  onExport,
}: Props) {
  const { t } = useTranslation();

  return (
    <div className="shrink-0 px-6 py-4 bg-bg-tertiary/30 border-t border-border-default flex items-center justify-between">
      <div className="flex items-center gap-4 text-xs text-text-muted font-medium">
        <span>{t("cnRomTools.footer.totalFiles", { count: totalCount })}</span>
        {missingCount > 0 && (
          <span className="text-accent-error flex items-center gap-1.5">
            <AlertTriangle className="w-3.5 h-3.5" />
            {t("cnRomTools.footer.missingEnglishNames", { count: missingCount })}
          </span>
        )}
      </div>

      <div className="flex items-center gap-2">
        {/* 匹配英文名按钮 */}
        <Button
          size="sm"
          onClick={onMatch}
          disabled={isFixing}
          loading={isFixing}
          className="text-xs min-w-[120px]"
        >
          {!isFixing && <Search className="w-3.5 h-3.5" />}
          {isFixing && matchProgress
            ? `(${matchProgress.current}/${matchProgress.total})`
            : t("cnRomTools.footer.matchEnglishName")}
        </Button>

        {/* 导出按钮 */}
        <ExportPanel isExporting={isExporting} onExport={onExport} />
      </div>
    </div>
  );
}
