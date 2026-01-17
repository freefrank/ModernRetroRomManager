import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useRomStore } from "@/stores/romStore";
import { useAppStore, THEMES } from "@/stores/appStore";
import { Folder, Trash2, RefreshCw, Plus, HardDrive, X } from "lucide-react";
import { clsx } from "clsx";
import DirectoryInput from "@/components/common/DirectoryInput";

export default function Settings() {
  const { t } = useTranslation();
  const { theme, setTheme } = useAppStore();
  const {
    scanDirectories,
    fetchScanDirectories,
    addScanDirectory,
    removeScanDirectory,
    startScan,
    isScanning,
    scanProgress
  } = useRomStore();

  const [isAddDialogOpen, setIsAddDialogOpen] = useState(false);
  const [newDirPath, setNewDirPath] = useState("");
  const [isValidPath, setIsValidPath] = useState(false);

  useEffect(() => {
    fetchScanDirectories();
  }, [fetchScanDirectories]);

  const handleAddDirectory = async () => {
    if (!isValidPath || !newDirPath.trim()) return;
    try {
      await addScanDirectory(newDirPath);
      setIsAddDialogOpen(false);
      setNewDirPath("");
      setIsValidPath(false);
    } catch (error) {
      console.error("Error adding directory:", error);
    }
  };

  const handleScan = async (id: string) => {
    if (isScanning) return;
    await startScan(id);
  };

  return (
    <div className="flex flex-col h-full">
      {/* 工具栏 */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-border-default bg-bg-primary/50 backdrop-blur-md sticky top-0 z-10">
        <h1 className="text-xl font-bold text-text-primary">{t("settings.title")}</h1>
      </div>

      {/* 内容区 */}
      <div className="flex-1 p-6 overflow-auto">
        <div className="max-w-3xl space-y-8">

          {/* 外观设置 */}
          <section>
            <h2 className="text-lg font-medium text-text-primary mb-4">外观主题</h2>
            <div className="grid grid-cols-4 gap-3">
              {THEMES.map((t) => (
                <button
                  key={t.id}
                  onClick={() => setTheme(t.id)}
                  className={clsx(
                    "p-4 rounded-xl border transition-all flex flex-col items-center gap-2",
                    theme === t.id
                      ? "bg-bg-secondary border-accent-primary text-text-primary ring-1 ring-accent-primary"
                      : "bg-bg-secondary border-border-default text-text-secondary hover:border-border-hover"
                  )}
                >
                  <span className="text-2xl">{t.icon}</span>
                  <span className="text-sm font-medium">{t.name}</span>
                </button>
              ))}
            </div>
          </section>

          {/* 扫描目录 */}
          <section>
            <div className="flex items-center justify-between mb-4">
              <div>
                <h2 className="text-lg font-medium text-text-primary">{t("settings.scanDirectories.title")}</h2>
                <p className="text-sm text-text-secondary mt-1">
                  {t("settings.scanDirectories.description")}
                </p>
              </div>
              <button
                onClick={() => setIsAddDialogOpen(true)}
                className="flex items-center gap-2 px-4 py-2 bg-accent-primary hover:bg-accent-primary/90 text-white rounded-lg transition-colors text-sm font-medium"
              >
                <Plus className="w-4 h-4" />
                {t("settings.scanDirectories.addDirectory")}
              </button>
            </div>

            {/* 扫描进度 */}
            {isScanning && scanProgress && (
              <div className="mb-4 p-4 bg-bg-secondary border border-accent-primary/30 rounded-xl relative overflow-hidden">
                <div className="absolute inset-0 bg-accent-primary/5 animate-pulse"></div>
                <div className="relative z-10">
                  <div className="flex justify-between text-sm mb-2">
                    <span className="text-text-primary font-medium">{t("common.loading")}</span>
                    <span className="text-accent-primary">{scanProgress.current} {scanProgress.total ? `/ ${scanProgress.total}` : ''}</span>
                  </div>
                  <div className="h-2 bg-bg-tertiary rounded-full overflow-hidden">
                    <div
                      className="h-full bg-accent-primary transition-all duration-300"
                      style={{ width: scanProgress.total ? `${(scanProgress.current / scanProgress.total) * 100}%` : '100%' }}
                    ></div>
                  </div>
                  <p className="text-xs text-text-muted mt-2 truncate">{scanProgress.message}</p>
                </div>
              </div>
            )}

            <div className="space-y-3">
              {scanDirectories.length === 0 ? (
                <div className="p-8 bg-bg-secondary border border-dashed border-border-default rounded-xl text-center">
                  <div className="w-16 h-16 mx-auto mb-4 bg-bg-tertiary rounded-full flex items-center justify-center">
                    <Folder className="w-8 h-8 text-text-muted" />
                  </div>
                  <p className="text-text-secondary mb-4">{t("settings.scanDirectories.empty")}</p>
                  <button
                    onClick={() => setIsAddDialogOpen(true)}
                    className="text-accent-primary hover:text-accent-primary/80 text-sm font-medium"
                  >
                    {t("settings.scanDirectories.addDirectory")}
                  </button>
                </div>
              ) : (
                scanDirectories.map((dir) => (
                  <div key={dir.id} className="group p-4 bg-bg-secondary border border-border-default rounded-xl hover:border-border-hover transition-all flex items-center justify-between">
                    <div className="flex items-center gap-4 overflow-hidden">
                      <div className="w-10 h-10 bg-bg-tertiary rounded-lg flex items-center justify-center flex-shrink-0">
                        <HardDrive className="w-5 h-5 text-accent-secondary" />
                      </div>
                      <div className="min-w-0">
                        <div className="text-text-primary font-medium truncate text-sm" title={dir.path}>{dir.path}</div>
                        <div className="text-xs text-text-muted mt-0.5">
                          {dir.lastScan ? `Last scan: ${dir.lastScan}` : "Never scanned"}
                        </div>
                      </div>
                    </div>

                    <div className="flex items-center gap-2">
                      <button
                        onClick={() => handleScan(dir.id)}
                        disabled={isScanning}
                        className={clsx(
                          "p-2 rounded-lg transition-colors",
                          isScanning
                            ? "text-text-muted cursor-not-allowed opacity-50"
                            : "text-text-secondary hover:text-text-primary hover:bg-bg-tertiary"
                        )}
                        title={t("common.refresh")}
                      >
                        <RefreshCw className={clsx("w-4 h-4", isScanning && "animate-spin")} />
                      </button>
                      <button
                        onClick={() => removeScanDirectory(dir.id)}
                        className="p-2 rounded-lg text-text-secondary hover:text-accent-error hover:bg-accent-error/10 transition-colors"
                        title={t("common.delete")}
                      >
                        <Trash2 className="w-4 h-4" />
                      </button>
                    </div>
                  </div>
                ))
              )}
            </div>
          </section>

          {/* 存储设置 (Mock) */}
          <section>
            <h2 className="text-lg font-medium text-text-primary mb-4">{t("settings.storage.title")}</h2>

            <div className="space-y-4">
              <div>
                <label className="block text-sm text-text-secondary mb-1">{t("settings.storage.databaseLocation")}</label>
                <div className="flex gap-2">
                  <input
                    type="text"
                    value={t("settings.storage.defaultLocation")}
                    readOnly
                    className="flex-1 px-3 py-2 bg-bg-secondary border border-border-default rounded-lg text-sm text-text-secondary focus:outline-none"
                  />
                  <button className="px-4 py-2 bg-bg-tertiary hover:bg-border-hover text-text-primary rounded-lg transition-colors text-sm border border-border-default">
                    {t("settings.storage.browse")}
                  </button>
                </div>
              </div>
            </div>
          </section>

          {/* 关于 */}
          <section>
            <h2 className="text-lg font-medium text-text-primary mb-4">{t("settings.about.title")}</h2>

            <div className="p-4 bg-bg-secondary border border-border-default rounded-xl">
              <div className="flex items-center gap-4">
                <div className="w-16 h-16 bg-gradient-to-br from-accent-primary to-accent-secondary rounded-xl flex items-center justify-center shadow-lg shadow-accent-primary/20">
                  <span className="text-2xl font-bold text-white">MR</span>
                </div>
                <div>
                  <h3 className="text-lg font-bold text-text-primary">ModernRetroRomManager</h3>
                  <p className="text-text-secondary text-sm">v0.1.0</p>
                  <div className="flex gap-4 mt-2">
                    <a
                      href="https://github.com/dotslash/modern-retro-rom-manager"
                      className="text-accent-primary hover:text-accent-secondary hover:underline text-xs"
                      target="_blank"
                      rel="noopener noreferrer"
                    >
                      GitHub
                    </a>
                    <span className="text-text-muted text-xs">•</span>
                    <span className="text-text-muted text-xs">MIT License</span>
                  </div>
                </div>
              </div>
            </div>
          </section>
        </div>
      </div>

      {/* 添加目录弹窗 */}
      {isAddDialogOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
          <div
            className="absolute inset-0 bg-black/60 backdrop-blur-sm"
            onClick={() => setIsAddDialogOpen(false)}
          />
          <div className="relative w-full max-w-md bg-bg-primary border border-border-default rounded-2xl shadow-2xl p-6">
            <div className="flex items-center justify-between mb-6">
              <h3 className="text-lg font-bold text-text-primary">添加扫描目录</h3>
              <button
                onClick={() => setIsAddDialogOpen(false)}
                className="p-2 rounded-lg hover:bg-bg-tertiary text-text-secondary"
              >
                <X className="w-5 h-5" />
              </button>
            </div>

            <DirectoryInput
              value={newDirPath}
              onChange={setNewDirPath}
              onValidPath={(v) => setIsValidPath(v.exists && v.is_directory && v.readable)}
              placeholder="例如: C:\ROMs\SNES 或 /home/user/roms"
            />

            <div className="flex justify-end gap-3 mt-6">
              <button
                onClick={() => setIsAddDialogOpen(false)}
                className="px-4 py-2 rounded-xl text-text-primary hover:bg-bg-tertiary transition-colors text-sm font-medium"
              >
                取消
              </button>
              <button
                onClick={handleAddDirectory}
                disabled={!isValidPath}
                className="px-6 py-2 bg-accent-primary hover:bg-accent-primary/90 text-text-primary rounded-xl font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed text-sm"
              >
                添加
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
