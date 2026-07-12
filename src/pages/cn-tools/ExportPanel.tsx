import { useState } from "react";
import { useTranslation } from "react-i18next";
import { FileDown, FileText } from "lucide-react";
import { Button } from "@/components/ui";

export type ExportFormat = "pegasus" | "gamelist";

interface Props {
  isExporting: boolean;
  onExport: (format: ExportFormat) => void;
}

/** 导出面板:导出按钮 + 格式选择菜单 */
export default function ExportPanel({ isExporting, onExport }: Props) {
  const { t } = useTranslation();
  const [showMenu, setShowMenu] = useState(false);

  const handleSelect = (format: ExportFormat) => {
    setShowMenu(false);
    onExport(format);
  };

  return (
    <div className="relative">
      <Button
        size="sm"
        onClick={() => setShowMenu(!showMenu)}
        disabled={isExporting}
        loading={isExporting}
        className="text-xs"
      >
        {!isExporting && <FileDown className="w-3.5 h-3.5" />}
        {t("cnRomTools.footer.exportMetadata")}
      </Button>

      {/* 导出格式选择菜单 */}
      {showMenu && (
        <div className="absolute bottom-full right-0 mb-2 bg-bg-secondary border-[length:var(--border-width)] border-border-default rounded-[var(--radius-md)] [box-shadow:var(--shadow-card)] overflow-hidden z-20 min-w-[180px]">
          <button
            onClick={() => handleSelect("pegasus")}
            className="w-full px-4 py-3 text-left text-sm text-text-primary hover:bg-bg-tertiary transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)] flex items-center gap-2"
          >
            <FileText className="w-4 h-4 text-accent-primary" />
            {t("cnRomTools.export.pegasus")}
          </button>
          <button
            onClick={() => handleSelect("gamelist")}
            className="w-full px-4 py-3 text-left text-sm text-text-primary hover:bg-bg-tertiary transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)] flex items-center gap-2 border-t border-border-default"
          >
            <FileText className="w-4 h-4 text-accent-success" />
            {t("cnRomTools.export.gamelist")}
          </button>
        </div>
      )}
    </div>
  );
}
