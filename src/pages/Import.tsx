import { useEffect, useMemo, useState } from "react";
import { FileArchive, FileDown, FolderOutput } from "lucide-react";
import { useTranslation } from "react-i18next";
import DirectoryInput from "@/components/common/DirectoryInput";
import { Button, Card, Select, toast } from "@/components/ui";
import { api } from "@/lib/api";
import { useRomStore } from "@/stores/romStore";
import type { RomSystemSummary, ScanDirectory } from "@/types";

type ExportFormat = "pegasus" | "emulationstation";

const ALL_SYSTEMS = "__all__";
const normalizePath = (path: string) => path.replace(/\\/g, "/");

export default function Import() {
  const { t } = useTranslation();
  const { exportData, exportLibraryData, isExporting, exportProgress } = useRomStore();
  const [libraries, setLibraries] = useState<ScanDirectory[]>([]);
  const [libraryId, setLibraryId] = useState("");
  const [systems, setSystems] = useState<RomSystemSummary[]>([]);
  const [scope, setScope] = useState(ALL_SYSTEMS);
  const [format, setFormat] = useState<ExportFormat>("pegasus");
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
    if (!libraryId) {
      setSystems([]);
      return;
    }
    let cancelled = false;
    setIsLoadingSystems(true);
    setScope(ALL_SYSTEMS);
    api.getLibraryRomSummary(libraryId)
      .then((items) => {
        if (!cancelled) setSystems(items);
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
  const selectedSystem = useMemo(
    () => systems.find((system) => system.system === scope),
    [systems, scope],
  );
  const totalRoms = useMemo(
    () => systems.reduce((total, system) => total + system.romCount, 0),
    [systems],
  );

  useEffect(() => {
    const path = scope === ALL_SYSTEMS ? selectedLibrary?.path : selectedSystem?.path;
    setTargetDirectory(path ? normalizePath(path) : "");
  }, [scope, selectedLibrary?.path, selectedSystem?.path]);

  const handleExport = async () => {
    if (!selectedLibrary || !targetDirectory.trim()) return;
    try {
      if (scope === ALL_SYSTEMS) {
        await exportLibraryData(selectedLibrary.id, format, targetDirectory.trim());
      } else if (selectedSystem) {
        await exportData(
          selectedSystem.system,
          selectedSystem.path,
          format,
          targetDirectory.trim(),
        );
      }
    } catch (error) {
      toast.error(t("import.export.failed", { error: String(error) }));
    }
  };

  const canExport = Boolean(
    selectedLibrary
      && targetDirectory.trim()
      && !isLoadingSystems
      && (scope === ALL_SYSTEMS ? systems.length > 0 : selectedSystem),
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

            <label className="block">
              <span className="mb-2 block text-sm font-medium">{t("import.export.scope")}</span>
              <Select
                value={scope}
                disabled={!selectedLibrary || isLoadingSystems}
                onChange={(event) => setScope(event.target.value)}
              >
                <option value={ALL_SYSTEMS}>
                  {isLoadingSystems
                    ? t("import.export.loadingSystems")
                    : t("import.export.entireLibrary", { systems: systems.length, roms: totalRoms })}
                </option>
                {systems.map((system) => (
                  <option key={`${system.system}-${system.path}`} value={system.system}>
                    {system.system} ({system.romCount}) — {normalizePath(system.path)}
                  </option>
                ))}
              </Select>
            </label>

            <label className="block">
              <span className="mb-2 block text-sm font-medium">{t("import.export.format")}</span>
              <Select value={format} onChange={(event) => setFormat(event.target.value as ExportFormat)}>
                <option value="pegasus">Pegasus — metadata.pegasus.txt</option>
                <option value="emulationstation">EmulationStation — gamelist.xml</option>
              </Select>
            </label>

            <div>
              <span className="mb-2 block text-sm font-medium">{t("import.export.target")}</span>
              <DirectoryInput value={targetDirectory} onChange={setTargetDirectory} />
              <p className="mt-2 text-xs text-text-muted">{t("import.export.targetHint")}</p>
            </div>

            <div className="rounded-[var(--radius-md)] border border-border-default bg-bg-primary/50 p-4 text-sm text-text-secondary">
              <div className="flex items-center gap-2">
                <FileArchive className="h-4 w-4 text-accent-primary" />
                {scope === ALL_SYSTEMS
                  ? t("import.export.libraryContents")
                  : t("import.export.contents")}
              </div>
              {isExporting && exportProgress && (
                <div className="mt-3">
                  <div className="mb-1 flex justify-between text-xs">
                    <span>{exportProgress.message}</span><span>{exportProgress.current}%</span>
                  </div>
                  <div className="h-1.5 overflow-hidden rounded-full bg-bg-tertiary">
                    <div className="h-full bg-accent-primary" style={{ width: `${exportProgress.current}%` }} />
                  </div>
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
              {scope === ALL_SYSTEMS
                ? t("import.export.libraryAction")
                : t("import.export.action")}
            </Button>
          </div>
        </Card>
      </div>
    </div>
  );
}
