import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import { FolderSearch, ChevronDown, RefreshCw, Search } from "lucide-react";
import { clsx } from "clsx";
import type { GameSystem } from "@/types";
import type { ScanProgress } from "@/stores/cnRomToolsStore";
import { Button, Dialog, Input } from "@/components/ui";

interface Props {
  checkPath: string;
  isChecking: boolean;
  isFixing: boolean;
  scanProgress: ScanProgress | null;
  systems: GameSystem[];
  selectedSystem: GameSystem | null;
  systemPickerOpen: boolean;
  onSystemPickerOpenChange: (open: boolean) => void;
  onSelectSystem: (system: GameSystem) => void;
  onBrowse: () => void;
  onRefresh: () => void;
}

/** 目录选择区:平台选择按钮 + 路径输入 + 浏览/刷新按钮 + 平台选择弹窗 */
export default function DirectoryPicker({
  checkPath,
  isChecking,
  isFixing,
  scanProgress,
  systems,
  selectedSystem,
  systemPickerOpen,
  onSystemPickerOpenChange,
  onSelectSystem,
  onBrowse,
  onRefresh,
}: Props) {
  const { t } = useTranslation();
  const [systemFilter, setSystemFilter] = useState("");

  // 过滤后的系统列表
  const filteredSystems = useMemo(() => {
    if (!systemFilter.trim()) return systems;
    const lower = systemFilter.toLowerCase();
    return systems.filter(
      (s) => s.name.toLowerCase().includes(lower) || s.id.toLowerCase().includes(lower),
    );
  }, [systemFilter, systems]);

  const displaySystemName = selectedSystem?.name || t("cnRomTools.selectPlatform");

  return (
    <>
      <div className="shrink-0 flex items-center gap-2">
        <FolderSearch className="w-5 h-5 text-accent-primary" />
        {/* 系统名称按钮 - 点击可选择 */}
        <button
          onClick={() => onSystemPickerOpenChange(true)}
          className="text-lg font-bold text-text-primary flex items-center gap-1 hover:text-accent-primary transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)]"
        >
          {displaySystemName}
          <ChevronDown className="w-4 h-4" />
        </button>
      </div>

      <div className="shrink-0 flex gap-3">
        <div className="flex-1 flex gap-2">
          <Input
            type="text"
            value={checkPath}
            readOnly
            placeholder={t("cnRomTools.selectDirectoryPlaceholder")}
            className="flex-1"
          />
          <Button variant="ghost" onClick={onBrowse} className="whitespace-nowrap">
            {t("cnRomTools.browse")}
          </Button>
        </div>
        <Button
          onClick={onRefresh}
          disabled={!checkPath || isChecking || isFixing}
          loading={isChecking}
          className="whitespace-nowrap"
        >
          {!isChecking && <RefreshCw className="w-4 h-4" />}
          {isChecking && scanProgress
            ? `(${scanProgress.current}/${scanProgress.total})`
            : t("cnRomTools.refresh")}
        </Button>
      </div>

      {/* 系统选择弹窗 */}
      <Dialog
        open={systemPickerOpen}
        onClose={() => {
          onSystemPickerOpenChange(false);
          setSystemFilter("");
        }}
        title={t("cnRomTools.systemPicker.title")}
        className="max-w-md"
      >
        <div className="flex flex-col gap-3 max-h-[60vh]">
          <div className="relative shrink-0">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-text-muted pointer-events-none" />
            <Input
              type="text"
              value={systemFilter}
              onChange={(e) => setSystemFilter(e.target.value)}
              placeholder={t("cnRomTools.systemPicker.searchPlaceholder")}
              className="pl-10"
              autoFocus
            />
          </div>
          <div className="flex-1 min-h-0 overflow-auto custom-scrollbar">
            {filteredSystems.length === 0 ? (
              <div className="p-6 text-center text-text-muted text-sm">
                {t("cnRomTools.systemPicker.noResults")}
              </div>
            ) : (
              <div className="flex flex-col gap-0.5">
                {filteredSystems.map((sys) => (
                  <button
                    key={sys.id}
                    onClick={() => {
                      onSelectSystem(sys);
                      onSystemPickerOpenChange(false);
                      setSystemFilter("");
                    }}
                    className={clsx(
                      "w-full px-4 py-3 text-left rounded-[var(--radius-md)] transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)] flex items-center gap-3",
                      selectedSystem?.id === sys.id
                        ? "bg-accent-primary/20 text-accent-primary"
                        : "hover:bg-bg-tertiary text-text-primary",
                    )}
                  >
                    <span className="font-medium">{sys.name}</span>
                    <span className="text-xs text-text-muted ml-auto">{sys.id}</span>
                  </button>
                ))}
              </div>
            )}
          </div>
        </div>
      </Dialog>
    </>
  );
}
