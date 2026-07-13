import { useEffect, useMemo, useState } from "react";
import { FileArchive, FileDown, FolderOutput } from "lucide-react";
import { useTranslation } from "react-i18next";
import DirectoryInput from "@/components/common/DirectoryInput";
import { Button, Card, Select, toast } from "@/components/ui";
import { useRomStore } from "@/stores/romStore";

type ExportFormat = "pegasus" | "emulationstation";

export default function Import() {
  const { t } = useTranslation();
  const { systemRoms, fetchRoms, exportData, isExporting, exportProgress } = useRomStore();
  const [system, setSystem] = useState("");
  const [format, setFormat] = useState<ExportFormat>("pegasus");
  const [targetDirectory, setTargetDirectory] = useState("");

  useEffect(() => {
    if (systemRoms.length === 0) fetchRoms();
  }, [fetchRoms, systemRoms.length]);

  const selected = useMemo(
    () => systemRoms.find((entry) => entry.system === system),
    [system, systemRoms],
  );

  useEffect(() => {
    if (!system && systemRoms.length > 0) setSystem(systemRoms[0].system);
  }, [system, systemRoms]);

  useEffect(() => {
    if (selected?.path) setTargetDirectory(selected.path);
  }, [selected]);

  const handleExport = async () => {
    if (!selected || !targetDirectory.trim()) return;
    try {
      await exportData(selected.system, selected.path, format, targetDirectory.trim());
    } catch (error) {
      toast.error(t("import.export.failed", { error: String(error) }));
    }
  };

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
              <span className="mb-2 block text-sm font-medium">{t("import.export.system")}</span>
              <Select value={system} onChange={(event) => setSystem(event.target.value)}>
                {systemRoms.map((entry) => (
                  <option key={`${entry.system}-${entry.path}`} value={entry.system}>
                    {entry.system} ({entry.roms.length})
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
                {t("import.export.contents")}
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
              disabled={!selected || !targetDirectory.trim()}
              loading={isExporting}
              onClick={handleExport}
            >
              <FileDown className="h-4 w-4" />
              {t("import.export.action")}
            </Button>
          </div>
        </Card>
      </div>
    </div>
  );
}
