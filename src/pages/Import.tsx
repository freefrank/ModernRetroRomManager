import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { open } from "@tauri-apps/plugin-dialog";
import { clsx } from "clsx";
import { CheckCircle2, AlertCircle } from "lucide-react";

interface ExportProgress {
  current: number;
  total: number;
  message: string;
  finished: boolean;
}

export default function Import() {
  const { t } = useTranslation();
  const importing = false;
  const [exporting, setExporting] = useState<string | null>(null);
  const [result, setResult] = useState<{ success: boolean; message: string } | null>(null);
  const [progress, setProgress] = useState<ExportProgress | null>(null);

  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setupListener = async () => {
      unlisten = await listen<ExportProgress>("export-progress", (event) => {
        setProgress(event.payload);
        if (event.payload.finished) {
          setExporting(null);
          setResult({
            success: true,
            message: "Export completed successfully!",
          });
          setProgress(null);
        }
      });
    };

    setupListener();

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  const handleImport = async () => {
    setResult({
      success: true,
      message: t("import.importSection.deprecated"),
    });
  };


  const handleExport = async (format: string) => {
    setResult(null);
    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });

      if (selected && typeof selected === "string") {
        setExporting(format);
        setProgress({ current: 0, total: 0, message: "Starting export...", finished: false });
        await invoke("export_to_emulationstation", { targetDir: selected });
      }
    } catch (error) {
      console.error("Export failed:", error);
      setResult({
        success: false,
        message: String(error),
      });

      setExporting(null);
      setProgress(null);
    }
  };

  return (
    <div className="flex flex-col h-full">
      {/* 工具栏 */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-border-default bg-bg-primary/50 backdrop-blur-md sticky top-0 z-10">
        <h1 className="text-xl font-bold text-text-primary">{t("import.title")}</h1>
      </div>

      {/* 内容区 */}
      <div className="flex-1 p-6 overflow-auto">
        <div className="max-w-3xl space-y-8">

          {/* Result Alert */}
          {result && (
            <div className={clsx(
              "p-4 rounded-xl border flex items-center gap-3",
              result.success ? "bg-accent-success/10 border-accent-success/20 text-accent-success" : "bg-accent-error/10 border-accent-error/20 text-accent-error"
            )}>
              {result.success ? <CheckCircle2 className="w-5 h-5" /> : <AlertCircle className="w-5 h-5" />}
              <span className="font-medium">{result.message}</span>
            </div>
          )}

          {/* Progress Bar */}
          {progress && (
            <div className="mb-4 p-4 bg-bg-secondary border border-accent-primary/30 rounded-xl relative overflow-hidden">
              <div className="absolute inset-0 bg-accent-primary/5 animate-pulse"></div>
              <div className="relative z-10">
                <div className="flex justify-between text-sm mb-2">
                  <span className="text-text-primary font-medium">{t("common.loading")}</span>
                  <span className="text-accent-primary">{progress.current} {progress.total > 0 ? `/ ${progress.total}` : ''}</span>
                </div>
                <div className="h-2 bg-bg-tertiary rounded-full overflow-hidden">
                  <div
                    className="h-full bg-accent-primary transition-all duration-300"
                    style={{ width: progress.total > 0 ? `${(progress.current / progress.total) * 100}%` : '100%' }}
                  ></div>
                </div>
                <p className="text-xs text-text-muted mt-2 truncate">{progress.message}</p>
              </div>
            </div>
          )}

          {/* 导入 */}
          <section>
            <h2 className="text-lg font-medium text-text-primary mb-4">{t("import.importSection.title")}</h2>
            <p className="text-text-secondary mb-6 text-sm">
              {t("import.importSection.description")}
            </p>

            <div className="grid grid-cols-2 gap-4">
              {/* EmulationStation */}
              <button
                onClick={handleImport}
                disabled={!!importing || !!exporting}
                className="group p-4 bg-bg-secondary border border-border-default rounded-xl hover:border-accent-primary/50 transition-all text-left disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <div className="flex items-center gap-4 mb-2">
                  <div className="w-12 h-12 bg-orange-500/20 rounded-xl flex items-center justify-center flex-shrink-0 group-hover:scale-110 transition-transform">
                    <span className="text-orange-400 font-bold text-lg">ES</span>
                  </div>
                  <div>
                    <h3 className="font-bold text-text-primary group-hover:text-accent-primary transition-colors">EmulationStation</h3>
                    <p className="text-xs text-text-secondary mt-0.5">gamelist.xml</p>
                  </div>
                </div>
              </button>

              {/* Other Placeholders */}
              {["Pegasus/Recalbox", "LaunchBox", "RetroArch"].map((name) => (
                <button
                  key={name}
                  disabled
                  className="p-4 bg-bg-secondary border border-border-default rounded-xl opacity-50 cursor-not-allowed text-left"
                >
                  <div className="flex items-center gap-4 mb-2">
                    <div className="w-12 h-12 bg-bg-tertiary rounded-xl flex items-center justify-center flex-shrink-0">
                      <span className="text-text-muted font-bold text-lg">{name.substring(0, 2).toUpperCase()}</span>
                    </div>
                    <div>
                      <h3 className="font-bold text-text-muted">{name}</h3>
                      <p className="text-xs text-text-muted mt-0.5">Coming Soon</p>
                    </div>
                  </div>
                </button>
              ))}
            </div>
          </section>

          {/* 导出 */}
          <section>
            <h2 className="text-lg font-medium text-text-primary mb-4">{t("import.exportSection.title")}</h2>
            <p className="text-text-secondary mb-6 text-sm">
              {t("import.exportSection.description")}
            </p>

            <div className="grid grid-cols-2 gap-4">
              <button
                onClick={() => handleExport("emulationstation")}
                disabled={!!importing || !!exporting}
                className="group p-4 bg-bg-secondary border border-border-default rounded-xl hover:border-accent-primary/50 transition-all text-left disabled:opacity-50 disabled:cursor-not-allowed"
              >
                <div className="flex items-center gap-4 mb-2">
                  <div className="w-12 h-12 bg-orange-500/20 rounded-xl flex items-center justify-center flex-shrink-0 group-hover:scale-110 transition-transform">
                    <span className="text-orange-400 font-bold text-lg">ES</span>
                  </div>
                  <div>
                    <h3 className="font-bold text-text-primary group-hover:text-accent-primary transition-colors">EmulationStation</h3>
                    <p className="text-xs text-text-secondary mt-0.5">gamelist.xml + ROMs</p>
                  </div>
                </div>
              </button>
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}
