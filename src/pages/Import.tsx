import { useEffect, useMemo, useState } from "react";
import { FileArchive, FileDown, FolderOutput } from "lucide-react";
import { useTranslation } from "react-i18next";
import DirectoryInput from "@/components/common/DirectoryInput";
import { Button, Card, Select, toast } from "@/components/ui";
import { api } from "@/lib/api";
import { useRomStore } from "@/stores/romStore";
import type { RomSystemSummary, ScanDirectory } from "@/types";

type ExportFormat = "both" | "pegasus" | "emulationstation";
type ExportNameMode = "original" | "chinese";

const normalizePath = (path: string) => path.replace(/\\/g, "/");

interface ExportSettings {
  export_format?: string;
  export_name_mode?: string;
  export_target_directory?: string;
}

export default function Import() {
  const { t } = useTranslation();
  const { exportLibraryData, cancelExport, isExporting, exportProgress } = useRomStore();
  const [libraries, setLibraries] = useState<ScanDirectory[]>([]);
  const [libraryId, setLibraryId] = useState("");
  const [systems, setSystems] = useState<RomSystemSummary[]>([]);
  const [selectedSystemPaths, setSelectedSystemPaths] = useState<string[]>([]);
  const [format, setFormat] = useState<ExportFormat>("both");
  const [nameMode, setNameMode] = useState<ExportNameMode>("original");
  const [romAssetsOnly, setRomAssetsOnly] = useState(false);
  const [syncDelete, setSyncDelete] = useState(false);
  const [targetDirectory, setTargetDirectory] = useState("");
  const [isLoadingSystems, setIsLoadingSystems] = useState(false);

  useEffect(() => {
    let cancelled = false;
    api.getDirectories()
      .then((items) => {
        if (cancelled) return;
        setLibraries(items);
        const initial = items.find((item) => item.isActive) || items[0];
        setLibraryId(initial?.id || "");
      })
      .catch((error) => toast.error(t("import.export.libraryLoadFailed", { error: String(error) })));
    return () => {
      cancelled = true;
    };
  }, [t]);

  useEffect(() => {
    let cancelled = false;
    api.getAppSettings<ExportSettings>()
      .then((settings) => {
        if (cancelled) return;
        if (
          settings.export_format === "both" ||
          settings.export_format === "pegasus" ||
          settings.export_format === "emulationstation"
        ) {
          setFormat(settings.export_format);
        }
        if (settings.export_name_mode === "original" || settings.export_name_mode === "chinese") {
          setNameMode(settings.export_name_mode);
        }
        if (settings.export_target_directory) {
          setTargetDirectory(settings.export_target_directory);
        }
      })
      .catch((error) => console.error("Failed to load export preferences:", error));
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    if (!libraryId) {
      setSystems([]);
      return;
    }
    let cancelled = false;
    setIsLoadingSystems(true);
    setSelectedSystemPaths([]);
    api.getLibraryRomSummary(libraryId)
      .then((items) => {
        if (!cancelled) {
          setSystems(items);
          setSelectedSystemPaths(items.map((system) => system.path));
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setSystems([]);
          toast.error(t("import.export.libraryLoadFailed", { error: String(error) }));
        }
      })
      .finally(() => {
        if (!cancelled) setIsLoadingSystems(false);
      });
    return () => {
      cancelled = true;
    };
  }, [libraryId, t]);

  const selectedLibrary = useMemo(
    () => libraries.find((library) => library.id === libraryId),
    [libraries, libraryId],
  );
  useEffect(() => {
    setTargetDirectory("");
  }, [libraryId]);

  const allSystemsSelected = systems.length > 0 && selectedSystemPaths.length === systems.length;
  const toggleSystem = (path: string) => {
    setSelectedSystemPaths((current) => (
      current.includes(path)
        ? current.filter((item) => item !== path)
        : [...current, path]
    ));
  };

  const handleFormatChange = (value: ExportFormat) => {
    setFormat(value);
    void api.updateAppSetting("export_format", value)
      .catch((error) => console.error("Failed to save export format:", error));
  };

  const handleNameModeChange = (value: ExportNameMode) => {
    setNameMode(value);
    void api.updateAppSetting("export_name_mode", value)
      .catch((error) => console.error("Failed to save export name mode:", error));
  };

  const handleExport = async () => {
    if (!selectedLibrary || !targetDirectory.trim()) return;
    // 记住本次导出目录,下次打开导出页自动填入
    void api.updateAppSetting("export_target_directory", targetDirectory.trim())
      .catch((error) => console.error("Failed to save export target:", error));
    try {
      await exportLibraryData(
        selectedLibrary.id,
        format,
        targetDirectory.trim(),
        nameMode,
        romAssetsOnly,
        allSystemsSelected ? undefined : selectedSystemPaths,
        syncDelete,
      );
    } catch (error) {
      toast.error(t("import.export.failed", { error: String(error) }));
    }
  };

  const canExport = Boolean(
    selectedLibrary
      && targetDirectory.trim()
      && !isLoadingSystems
      && selectedSystemPaths.length > 0,
  );

  return (
    <div className="rr-page flex h-full flex-col">
      <div className="sticky top-0 z-10 flex items-center justify-between border-b border-border-default bg-bg-primary/50 px-6 py-4 backdrop-blur-md">
        <h1 className="text-xl font-bold text-text-primary">{t("import.title")}</h1>
      </div>

      <div className="flex-1 overflow-auto p-6">
        <Card className="mx-auto max-w-3xl p-6">
          <div className="mb-6 flex items-start gap-4">
            <div className="rounded-[var(--radius-md)] border border-accent-primary/30 bg-accent-primary/10 p-3 text-accent-primary">
              <FolderOutput className="h-6 w-6" />
            </div>
            <div>
              <h2 className="text-lg font-bold">{t("import.export.title")}</h2>
              <p className="mt-1 text-sm text-text-muted">{t("import.export.description")}</p>
            </div>

          </div>

          <div className="space-y-5">
            <label className="block">
              <span className="mb-2 block text-sm font-medium">{t("import.export.library")}</span>
              <Select value={libraryId} onChange={(event) => setLibraryId(event.target.value)}>
                {libraries.length === 0 && <option value="">{t("import.export.noLibraries")}</option>}
                {libraries.map((library) => (
                  <option key={library.id} value={library.id}>
                    {library.name} — {normalizePath(library.path)}
                  </option>
                ))}
              </Select>
            </label>

            <div>
              <span className="mb-2 block text-sm font-medium">{t("import.export.scope")}</span>
              <div className="mb-2 flex items-center gap-2">
                <Button variant="ghost" size="sm" disabled={isLoadingSystems} onClick={() => setSelectedSystemPaths(systems.map((system) => system.path))}>
                  {t("import.export.selectAll")}
                </Button>
                <Button variant="ghost" size="sm" disabled={isLoadingSystems} onClick={() => setSelectedSystemPaths([])}>
                  {t("import.export.selectNone")}
                </Button>
                <span className="text-xs text-text-muted">
                  {t("import.export.selectedSystems", { selected: selectedSystemPaths.length, total: systems.length })}
                </span>
              </div>
              <div className="max-h-56 space-y-1 overflow-auto rounded-[var(--radius-md)] border border-border-default bg-bg-primary/50 p-2">
                {isLoadingSystems ? (
                  <p className="p-2 text-sm text-text-muted">{t("import.export.loadingSystems")}</p>
                ) : systems.map((system) => (
                  <label key={`${system.system}-${system.path}`} className="flex cursor-pointer items-center gap-3 rounded-[var(--radius-sm)] px-2 py-2 hover:bg-bg-tertiary">
                    <input
                      type="checkbox"
                      className="h-4 w-4 accent-accent-primary"
                      checked={selectedSystemPaths.includes(system.path)}
                      onChange={() => toggleSystem(system.path)}
                    />
                    <span className="min-w-0 flex-1 truncate text-sm text-text-primary" title={normalizePath(system.path)}>
                      {system.system} ({system.romCount}) — {normalizePath(system.path)}
                    </span>
                  </label>
                ))}
              </div>
            </div>

            <label className="block">
              <span className="mb-2 block text-sm font-medium">{t("import.export.format")}</span>
              <Select value={format} onChange={(event) => handleFormatChange(event.target.value as ExportFormat)}>
                <option value="both">两者都导出 — gamelist.xml + metadata.pegasus.txt</option>
                <option value="pegasus">Pegasus — metadata.pegasus.txt</option>
                <option value="emulationstation">EmulationStation — gamelist.xml</option>
              </Select>
            </label>

            <label className="block">
              <span className="mb-2 block text-sm font-medium">{t("import.export.nameMode")}</span>
              <Select value={nameMode} onChange={(event) => handleNameModeChange(event.target.value as ExportNameMode)}>
                <option value="original">{t("import.export.nameOriginal")}</option>
                <option value="chinese">{t("import.export.nameChinese")}</option>
              </Select>
              <p className="mt-2 text-xs text-text-muted">{t("import.export.nameModeHint")}</p>
            </label>

            <div>
              <span className="mb-2 block text-sm font-medium">{t("import.export.target")}</span>
              <DirectoryInput value={targetDirectory} onChange={setTargetDirectory} />
              <p className="mt-2 text-xs text-text-muted">{t("import.export.targetHint")}</p>
            </div>

            <label className="flex items-start gap-3 rounded-[var(--radius-md)] border border-border-default bg-bg-primary/50 p-4">
              <input
                type="checkbox"
                className="mt-0.5 h-4 w-4 accent-accent-primary"
                checked={romAssetsOnly}
                onChange={(event) => setRomAssetsOnly(event.target.checked)}
              />
              <span>
                <span className="block text-sm font-medium">{t("import.export.romAssetsOnly")}</span>
                <span className="mt-1 block text-xs text-text-muted">
                  {t("import.export.romAssetsOnlyHint")}
                </span>
              </span>
            </label>

            <label className="flex items-start gap-3 rounded-[var(--radius-md)] border border-border-default bg-bg-primary/50 p-4">
              <input
                type="checkbox"
                className="mt-0.5 h-4 w-4 accent-accent-error"
                checked={syncDelete}
                onChange={(event) => setSyncDelete(event.target.checked)}
              />
              <span>
                <span className="block text-sm font-medium">{t("import.export.syncDelete")}</span>
                <span className="mt-1 block text-xs text-text-muted">
                  {t("import.export.syncDeleteHint")}
                </span>
              </span>
            </label>

            <div className="rounded-[var(--radius-md)] border border-border-default bg-bg-primary/50 p-4 text-sm text-text-secondary">
              <div className="flex items-center gap-2">
                <FileArchive className="h-4 w-4 text-accent-primary" />
                {t("import.export.selectionContents", { count: selectedSystemPaths.length })}
              </div>
              {isExporting && exportProgress && (
                <div className="mt-3">
                  <div className="mb-1 flex justify-between text-xs">
                    <span>{exportProgress.message}</span><span>{exportProgress.current}%</span>
                  </div>
                  <div className="h-1.5 overflow-hidden rounded-full bg-bg-tertiary">
                    <div className="h-full bg-accent-primary" style={{ width: `${exportProgress.current}%` }} />
                  </div>
                  {!exportProgress.finished && (
                    <Button className="mt-3" variant="danger" onClick={() => void cancelExport()}>
                      {t("common.stop")}
                    </Button>
                  )}
                </div>
              )}
            </div>

            <Button
              className="w-full"
              disabled={!canExport}
              loading={isExporting}
              onClick={handleExport}
            >
              <FileDown className="h-4 w-4" />
              {t("import.export.selectionAction", { count: selectedSystemPaths.length })}
            </Button>
          </div>
        </Card>
      </div>
    </div>
  );
}
