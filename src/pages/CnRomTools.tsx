import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Languages } from "lucide-react";
import { isTauri, api } from "@/lib/api";
import { Button, Dialog, Input, toast } from "@/components/ui";
import type { GameSystem } from "@/types";
import {
  useCnRomToolsStore,
  type MatchProgress,
  type ScanProgress,
} from "@/stores/cnRomToolsStore";
import AboutCard from "./cn-tools/AboutCard";
import DirectoryPicker from "./cn-tools/DirectoryPicker";
import ScanResultTable, { type SortOrder } from "./cn-tools/ScanResultTable";
import MatchToolbar from "./cn-tools/MatchToolbar";
import type { ExportFormat } from "./cn-tools/ExportPanel";

export default function CnRomTools() {
  const { t } = useTranslation();
  const [isSettingName, setIsSettingName] = useState(false);
  const [isAddingTag, setIsAddingTag] = useState(false);
  const [isExporting, setIsExporting] = useState(false);
  const [isOrganizing, setIsOrganizing] = useState(false);
  const [showArchiveDialog, setShowArchiveDialog] = useState(false);
  const [archivePassword, setArchivePassword] = useState("");

  // 从 store 获取状态
  const {
    checkPath,
    setCheckPath,
    checkResults,
    isScanning: isChecking,
    isFixing,
    scanProgress,
    matchProgress,
    setScanProgress,
    setMatchProgress,
    scan,
    autoFix,
    updateEnglishName,
    updateExtractedCnName,
  } = useCnRomToolsStore();

  // 系统列表(从后端获取)
  const [systems, setSystems] = useState<GameSystem[]>([]);
  const [selectedSystem, setSelectedSystem] = useState<GameSystem | null>(null);
  const [showSystemPicker, setShowSystemPicker] = useState(false);

  // 排序状态
  const [sortOrder, setSortOrder] = useState<SortOrder>(null);

  // 加载系统列表
  useEffect(() => {
    api
      .getSystems()
      .then(setSystems)
      .catch((error) => console.error("Failed to load systems:", error));
  }, []);

  // 监听扫描/匹配进度事件
  useEffect(() => {
    if (!isTauri()) return;

    const unlistens: (() => void)[] = [];

    const setupListeners = async () => {
      const { listen } = await import("@tauri-apps/api/event");
      unlistens.push(
        await listen<MatchProgress>("naming-match-progress", (event) => {
          setMatchProgress(event.payload);
        }),
      );
      unlistens.push(
        await listen<ScanProgress>("scan-progress", (event) => {
          setScanProgress(event.payload);
        }),
      );
    };

    setupListeners();

    return () => {
      unlistens.forEach((fn) => fn());
    };
  }, [setMatchProgress, setScanProgress]);

  // 切换排序:降序 -> 升序 -> 取消
  const toggleSort = () =>
    setSortOrder((order) => (order === null ? "desc" : order === "desc" ? "asc" : null));

  // 从目录名匹配系统(优先当前目录,其次上级目录)
  const matchSystemFromPath = (path: string): GameSystem | null => {
    const parts = path.split(/[/\\]/).filter(Boolean);
    const dirsToTry = [
      parts[parts.length - 1]?.toLowerCase() || "",
      parts[parts.length - 2]?.toLowerCase() || "",
    ].filter(Boolean);

    for (const dirName of dirsToTry) {
      const matched = systems.find((sys) => sys.id === dirName);
      if (matched) return matched;
    }
    return null;
  };

  const handleBrowse = async () => {
    if (!isTauri()) return;
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({ directory: true, multiple: false });
      if (selected && typeof selected === "string") {
        setCheckPath(selected);
        // 尝试从目录名匹配系统;没匹配到则弹出系统选择器
        const matched = matchSystemFromPath(selected);
        setSelectedSystem(matched);
        if (!matched) setShowSystemPicker(true);
        // 自动触发扫描
        scan(selected);
      }
    } catch (error) {
      console.error("Failed to select directory:", error);
    }
  };

  const handleCheck = () => {
    if (!checkPath) return;
    scan();
  };

  const handleOrganizeArchives = async () => {
    if (!checkPath || !selectedSystem || !isTauri()) return;
    setIsOrganizing(true);
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const result = await invoke<{
        archives: number;
        extracted: number;
        skipped: number;
        archived: number;
      }>("organize_rom_archives", {
        directory: checkPath,
        system: selectedSystem.id,
        password: archivePassword,
      });
      toast.success(t("cnRomTools.alerts.organizeComplete", result));
      setShowArchiveDialog(false);
      await scan();
    } catch (error) {
      toast.error(t("cnRomTools.alerts.organizeFailed", { error: String(error) }));
    } finally {
      setIsOrganizing(false);
    }
  };

  const handleAutoFix = async () => {
    if (!checkPath) return;
    if (!selectedSystem) {
      toast.info(t("cnRomTools.alerts.selectPlatformFirst"));
      return;
    }
    await autoFix(selectedSystem.id);
    setSortOrder(null); // 重置排序状态,防止编辑时列表跳动
  };

  const handleSetAsRomName = async () => {
    if (!checkPath) return;
    setIsSettingName(true);
    try {
      if (isTauri()) {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("set_extracted_cn_as_name", { directory: checkPath });
      }
      handleCheck();
    } catch (error) {
      console.error("Failed to set ROM name:", error);
      toast.error(t("cnRomTools.alerts.setFailed", { error: String(error) }));
    } finally {
      setIsSettingName(false);
    }
  };

  const handleAddAsTag = async () => {
    if (!checkPath) return;
    setIsAddingTag(true);
    try {
      if (isTauri()) {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("add_english_as_tag", { directory: checkPath });
      }
    } catch (error) {
      console.error("Failed to add tag:", error);
      toast.error(t("cnRomTools.alerts.addFailed", { error: String(error) }));
    } finally {
      setIsAddingTag(false);
    }
  };

  const handleExport = async (format: ExportFormat) => {
    if (!checkPath) return;
    try {
      if (!isTauri()) return;

      const { save } = await import("@tauri-apps/plugin-dialog");
      const defaultName = format === "pegasus" ? "metadata.pegasus.txt" : "gamelist.xml";
      const savePath = await save({
        defaultPath: checkPath + "/" + defaultName,
        filters:
          format === "pegasus"
            ? [{ name: t("cnRomTools.export.pegasusFilter"), extensions: ["txt"] }]
            : [{ name: t("cnRomTools.export.gamelistFilter"), extensions: ["xml"] }],
      });
      if (!savePath) return;

      setIsExporting(true);
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("export_cn_metadata", { directory: checkPath, targetPath: savePath, format });
      toast.success(t("cnRomTools.alerts.exportSuccess", { path: savePath }));
    } catch (error) {
      console.error("Failed to export:", error);
      toast.error(t("cnRomTools.alerts.exportFailed", { error: String(error) }));
    } finally {
      setIsExporting(false);
    }
  };

  // 后端持久化编辑后的名称
  const persistNameChange = async (command: string, file: string, args: Record<string, string>) => {
    try {
      if (isTauri()) {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke(command, { directory: checkPath, file, ...args });
      }
    } catch (error) {
      console.error("Failed to save name:", error);
      toast.error(t("cnRomTools.alerts.saveFailed", { error: String(error) }));
    }
  };

  const handleSaveEnglishName = (file: string, value: string) => {
    updateEnglishName(file, value);
    persistNameChange("update_english_name", file, { englishName: value });
  };

  const handleSaveExtractedCnName = (file: string, value: string) => {
    updateExtractedCnName(file, value);
    persistNameChange("update_extracted_cn_name", file, { extractedCnName: value });
  };

  const missingCount = checkResults.filter((r) => !r.english_name).length;

  return (
    <div className="rr-page flex flex-col h-full bg-bg-primary">
      {/* Header */}
      <div className="shrink-0 flex items-center justify-between px-6 py-4 border-b border-border-default bg-bg-primary/50 backdrop-blur-md">
        <div className="flex items-center gap-3">
          <Languages className="w-6 h-6 text-accent-primary" />
          <h1 className="text-xl font-bold text-text-primary tracking-tight">
            {t("cnRomTools.title")}
          </h1>
        </div>
      </div>

      {/* Content */}
      <div className="flex-1 min-h-0 flex flex-col p-6 gap-6 overflow-hidden">
        <AboutCard />

        {/* Directory Check */}
        <section className="flex-1 min-h-0 flex flex-col gap-4">
          <DirectoryPicker
            checkPath={checkPath}
            isChecking={isChecking}
            isFixing={isFixing}
            isOrganizing={isOrganizing}
            scanProgress={scanProgress}
            systems={systems}
            selectedSystem={selectedSystem}
            systemPickerOpen={showSystemPicker}
            onSystemPickerOpenChange={setShowSystemPicker}
            onSelectSystem={setSelectedSystem}
            onBrowse={handleBrowse}
            onRefresh={handleCheck}
            onOrganize={() => setShowArchiveDialog(true)}
          />

          <Dialog
            open={showArchiveDialog}
            onClose={() => !isOrganizing && setShowArchiveDialog(false)}
            title={t("cnRomTools.archive.title")}
            className="max-w-md"
          >
            <div className="flex flex-col gap-4">
              <p className="text-sm text-text-secondary">{t("cnRomTools.archive.description")}</p>
              <Input
                type="password"
                value={archivePassword}
                onChange={(event) => setArchivePassword(event.target.value)}
                placeholder={t("cnRomTools.archive.passwordPlaceholder")}
                disabled={isOrganizing}
              />
              <div className="flex justify-end gap-2">
                <Button
                  variant="ghost"
                  onClick={() => setShowArchiveDialog(false)}
                  disabled={isOrganizing}
                >
                  {t("cnRomTools.archive.cancel")}
                </Button>
                <Button onClick={handleOrganizeArchives} loading={isOrganizing}>
                  {t("cnRomTools.archive.confirm")}
                </Button>
              </div>
            </div>
          </Dialog>

          {/* Results */}
          {checkResults.length > 0 && (
            <div className="flex-1 min-h-0 flex flex-col bg-bg-secondary rounded-[var(--radius-lg)] border-[length:var(--border-width)] border-border-default overflow-hidden">
              <ScanResultTable
                results={checkResults}
                sortOrder={sortOrder}
                onToggleSort={toggleSort}
                isSettingName={isSettingName}
                isAddingTag={isAddingTag}
                onSetAsRomName={handleSetAsRomName}
                onAddAsTag={handleAddAsTag}
                onSaveEnglishName={handleSaveEnglishName}
                onSaveExtractedCnName={handleSaveExtractedCnName}
              />
              <MatchToolbar
                totalCount={checkResults.length}
                missingCount={missingCount}
                isFixing={isFixing}
                matchProgress={matchProgress}
                isExporting={isExporting}
                onMatch={handleAutoFix}
                onExport={handleExport}
              />
            </div>
          )}
        </section>
      </div>
    </div>
  );
}
