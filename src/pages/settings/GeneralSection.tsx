import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useSearchParams } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { Check, Folder, HardDrive, Pencil, Plus, RefreshCw, Trash2, X } from "lucide-react";
import { clsx } from "clsx";
import i18n, { languages } from "@/i18n";
import { useAppStore } from "@/stores/appStore";
import { useRomStore } from "@/stores/romStore";
import type { ViewMode } from "@/types";
import DirectoryInput from "@/components/common/DirectoryInput";
import MetadataImportDialog from "@/components/common/MetadataImportDialog";
import RootDirectoryDialog from "@/components/common/RootDirectoryDialog";
import {
  Button,
  Badge,
  Dialog,
  EmptyState,
  IconButton,
  Input,
  Select,
  toast,
} from "@/components/ui";

interface MetadataFileInfo {
  format: string;
  format_name: string;
  file_path: string;
  file_name: string;
}

interface SubDirectoryInfo {
  name: string;
  path: string;
  metadata_files: MetadataFileInfo[];
}

interface DirectoryScanResult {
  is_root_directory: boolean;
  metadata_files: MetadataFileInfo[];
  sub_directories: SubDirectoryInfo[];
}

const VIEW_MODES: ViewMode[] = ["grid", "list", "cover"];

export default function GeneralSection() {
  const { t } = useTranslation();
  const [searchParams, setSearchParams] = useSearchParams();
  const { language, setLanguage, viewMode, setViewMode, consoleEnabled, setConsoleEnabled } = useAppStore();
  const {
    scanDirectories,
    fetchScanDirectories,
    addScanDirectory,
    removeScanDirectory,
    activateLibrary,
    renameLibrary,
    isScanning,
    isBatchScraping,
    scanProgress,
    scanLibrary,
  } = useRomStore();

  const [isAddDialogOpen, setIsAddDialogOpen] = useState(false);
  const [newDirPath, setNewDirPath] = useState("");
  const [isValidPath, setIsValidPath] = useState(false);
  const [configDir, setConfigDir] = useState<string | null>(null);
  const [mediaDir, setMediaDir] = useState<string | null>(null);
  const [editingLibraryId, setEditingLibraryId] = useState<string | null>(null);
  const [editingLibraryName, setEditingLibraryName] = useState("");

  // 元数据检测状态
  const [detectedMetadata, setDetectedMetadata] = useState<MetadataFileInfo[]>([]);
  const [isMetadataDialogOpen, setIsMetadataDialogOpen] = useState(false);
  const [pendingDirPath, setPendingDirPath] = useState("");

  // 根目录扫描状态
  const [isRootDialogOpen, setIsRootDialogOpen] = useState(false);
  const [detectedSubDirs, setDetectedSubDirs] = useState<SubDirectoryInfo[]>([]);

  useEffect(() => {
    fetchScanDirectories();
  }, [fetchScanDirectories]);

  useEffect(() => {
    if (searchParams.get("addDirectory") === "1") {
      setIsAddDialogOpen(true);
      setSearchParams({ tab: "general" }, { replace: true });
    }
  }, [searchParams, setSearchParams]);

  useEffect(() => {
    const loadPaths = async () => {
      try {
        const [configPath, mediaPath] = await Promise.all([
          invoke<string>("get_config_dir"),
          invoke<string>("get_media_dir"),
        ]);
        setConfigDir(configPath);
        setMediaDir(mediaPath);
      } catch (error) {
        console.error("Failed to load config paths:", error);
      }
    };

    loadPaths();
  }, []);

  // 旧存储值 "zh" 兼容为 "zh-CN" 展示
  const languageValue = language === "zh" ? "zh-CN" : language;
  const handleLanguageChange = (code: string) => {
    i18n.changeLanguage(code);
    setLanguage(code);
  };

  const handleAddDirectory = async () => {
    if (!isValidPath || !newDirPath.trim()) return;
    try {
      const scanResult = await invoke<DirectoryScanResult>("scan_directory", {
        path: newDirPath,
      });

      if (scanResult.metadata_files.length > 0) {
        setPendingDirPath(newDirPath);
        setDetectedMetadata(scanResult.metadata_files);
        setIsAddDialogOpen(false);
        setIsMetadataDialogOpen(true);
      } else if (scanResult.sub_directories.length > 0) {
        setPendingDirPath(newDirPath);
        setDetectedSubDirs(scanResult.sub_directories);
        setIsAddDialogOpen(false);
        setIsRootDialogOpen(true);
      } else {
        // 先关闭弹窗,后台继续扫描,避免 UI 阻塞感
        setIsAddDialogOpen(false);
        try {
          await addScanDirectory(newDirPath);
        } catch (err) {
          console.error("Failed to add scan directory:", err);
          toast.error(t("settings.scanDirectories.addFailed", { error: String(err) }));
        }
      }

      setNewDirPath("");
      setIsValidPath(false);
    } catch (error) {
      console.error("Error adding directory:", error);
      toast.error(t("settings.scanDirectories.addFailed", { error: String(error) }));
    }
  };

  const handleMetadataImport = async (file: MetadataFileInfo) => {
    setIsMetadataDialogOpen(false);
    try {
      await addScanDirectory(pendingDirPath, file.format);
      setIsAddDialogOpen(false);
      setPendingDirPath("");
      setDetectedMetadata([]);
    } catch (error) {
      console.error("Error importing metadata:", error);
      toast.error(t("settings.scanDirectories.addFailed", { error: String(error) }));
    }
  };

  const handleSkipImport = async () => {
    setIsMetadataDialogOpen(false);
    try {
      await addScanDirectory(pendingDirPath, "none");
      setIsAddDialogOpen(false);
      setPendingDirPath("");
      setDetectedMetadata([]);
    } catch (error) {
      console.error("Error adding directory:", error);
      toast.error(t("settings.scanDirectories.addFailed", { error: String(error) }));
    }
  };

  const handleImportAsRoot = async () => {
    setIsRootDialogOpen(false);
    try {
      await addScanDirectory(pendingDirPath, "auto", true, null);
      setPendingDirPath("");
      setDetectedSubDirs([]);
    } catch (error) {
      console.error("Error adding root directory:", error);
      toast.error(t("settings.scanDirectories.addFailed", { error: String(error) }));
    }
  };

  const handleSelectSubDirectory = async (
    subDir: SubDirectoryInfo,
    format: string,
  ) => {
    setIsRootDialogOpen(false);
    try {
      await addScanDirectory(subDir.path, format, false, subDir.name);
      setPendingDirPath("");
      setDetectedSubDirs([]);
    } catch (error) {
      console.error("Error adding sub directory:", error);
      toast.error(t("settings.scanDirectories.addFailed", { error: String(error) }));
    }
  };

  const handleScan = async () => {
    await scanLibrary(true);
  };

  const handleActivateLibrary = async (id: string, fullScan = false) => {
    if (isScanning || isBatchScraping) return;
    try {
      await activateLibrary(id, fullScan);
    } catch (error) {
      toast.error(t("settings.scanDirectories.switchFailed", { error: String(error) }));
    }
  };

  const startRenamingLibrary = (id: string, name: string) => {
    setEditingLibraryId(id);
    setEditingLibraryName(name);
  };

  const cancelRenamingLibrary = () => {
    setEditingLibraryId(null);
    setEditingLibraryName("");
  };

  const saveLibraryName = async (id: string) => {
    const name = editingLibraryName.trim();
    if (!name) return;
    try {
      await renameLibrary(id, name);
      cancelRenamingLibrary();
      toast.success(t("settings.scanDirectories.renameSuccess"));
    } catch (error) {
      toast.error(t("settings.scanDirectories.renameFailed", { error: String(error) }));
    }
  };

  const handleRemoveLibrary = async (id: string) => {
    try {
      await removeScanDirectory(id);
    } catch (error) {
      toast.error(t("settings.scanDirectories.removeFailed", { error: String(error) }));
    }
  };

  return (
    <div className="space-y-8">
      {/* 语言 */}
      <section>
        <h2 className="text-lg font-medium text-text-primary">
          {t("settings.language.title")}
        </h2>
        <p className="text-sm text-text-secondary mt-1 mb-4">
          {t("settings.language.description")}
        </p>
        <Select
          value={languageValue}
          onChange={(e) => handleLanguageChange(e.target.value)}
          className="max-w-xs"
        >
          {languages.map((lang) => (
            <option key={lang.code} value={lang.code}>
              {lang.name}
            </option>
          ))}
        </Select>
      </section>

      {/* 默认视图 */}
      <section>
        <h2 className="text-lg font-medium text-text-primary">
          {t("settings.general.viewModeTitle")}
        </h2>
        <p className="text-sm text-text-secondary mt-1 mb-4">
          {t("settings.general.viewModeDescription")}
        </p>
        <Select
          value={viewMode}
          onChange={(e) => setViewMode(e.target.value as ViewMode)}
          className="max-w-xs"
        >
          {VIEW_MODES.map((mode) => (
            <option key={mode} value={mode}>
              {t(`settings.general.viewMode.${mode}`)}
            </option>
          ))}
        </Select>
      </section>

      <section>
        <h2 className="text-lg font-medium text-text-primary">
          {t("settings.general.consoleTitle")}
        </h2>
        <p className="text-sm text-text-secondary mt-1 mb-4">
          {t("settings.general.consoleDescription")}
        </p>
        <label className="inline-flex items-center gap-3 text-sm text-text-primary cursor-pointer">
          <input
            type="checkbox"
            checked={consoleEnabled}
            onChange={(event) => setConsoleEnabled(event.target.checked)}
            className="h-4 w-4 accent-[var(--accent-primary)]"
          />
          {t("settings.general.consoleEnabled")}
        </label>
      </section>

      {/* Library 管理 */}
      <section>
        <div className="flex items-center justify-between mb-4">
          <div>
            <h2 className="text-lg font-medium text-text-primary">
              {t("settings.scanDirectories.title")}
            </h2>
            <p className="text-sm text-text-secondary mt-1">
              {t("settings.scanDirectories.description")}
            </p>
          </div>
          <div className="flex items-center gap-2">
            <Button
              variant="ghost"
              size="sm"
              onClick={handleScan}
              disabled={isScanning || isBatchScraping || scanDirectories.length === 0}
            >
              <RefreshCw className={clsx("w-4 h-4", isScanning && "animate-spin")} />
              {t("settings.scanDirectories.fullScan")}
            </Button>
            <Button
              size="sm"
              onClick={() => setIsAddDialogOpen(true)}
              disabled={isScanning || isBatchScraping}
            >
              <Plus className="w-4 h-4" />
              {t("settings.scanDirectories.addDirectory")}
            </Button>
          </div>
        </div>

        {/* 扫描进度 */}
        {isScanning && scanProgress && (
          <div className="mb-4 p-4 bg-bg-secondary border-[length:var(--border-width)] border-accent-primary/30 rounded-[var(--radius-lg)] relative overflow-hidden">
            <div className="absolute inset-0 bg-accent-primary/5 animate-pulse"></div>
            <div className="relative z-10">
              <div className="flex justify-between text-sm mb-2">
                <span className="text-text-primary font-medium">
                  {t("common.loading")}
                </span>
                <span className="text-accent-primary">
                  {scanProgress.current}{" "}
                  {scanProgress.total ? `/ ${scanProgress.total}` : ""}
                </span>
              </div>
              <div className="h-2 bg-bg-tertiary rounded-full overflow-hidden">
                <div
                  className="h-full bg-accent-primary transition-all duration-[var(--motion-normal)] ease-[var(--motion-easing)]"
                  style={{
                    width: scanProgress.total
                      ? `${(scanProgress.current / scanProgress.total) * 100}%`
                      : "100%",
                  }}
                ></div>
              </div>
              <p className="text-xs text-text-muted mt-2 truncate">
                {scanProgress.message}
              </p>
            </div>
          </div>
        )}

        <div className="space-y-3">
          {scanDirectories.length === 0 ? (
            <EmptyState
              icon={<Folder />}
              title={t("settings.scanDirectories.empty")}
              action={
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setIsAddDialogOpen(true)}
                >
                  {t("settings.scanDirectories.addDirectory")}
                </Button>
              }
            />
          ) : (
            scanDirectories.map((dir) => (
              <div
                key={dir.id}
                role="button"
                tabIndex={editingLibraryId === dir.id ? -1 : 0}
                aria-current={dir.isActive ? "true" : undefined}
                onClick={() => void handleActivateLibrary(dir.id)}
                onKeyDown={(event) => {
                  if (event.key === "Enter" || event.key === " ") {
                    event.preventDefault();
                    void handleActivateLibrary(dir.id);
                  }
                }}
                className={clsx(
                  "group p-4 bg-bg-secondary border-[length:var(--border-width)] rounded-[var(--radius-lg)] transition-all duration-[var(--motion-fast)] ease-[var(--motion-easing)] flex items-center justify-between",
                  dir.isActive
                    ? "border-accent-primary/70 bg-accent-primary/5"
                    : "border-border-default hover:border-border-hover cursor-pointer",
                )}
              >
                <div className="flex items-center gap-4 overflow-hidden">
                  <div className="w-10 h-10 bg-bg-tertiary rounded-[var(--radius-md)] flex items-center justify-center flex-shrink-0">
                    <HardDrive className="w-5 h-5 text-accent-secondary" />
                  </div>
                  <div className="min-w-0">
                    {editingLibraryId === dir.id ? (
                      <Input
                        autoFocus
                        value={editingLibraryName}
                        onChange={(event) => setEditingLibraryName(event.target.value)}
                        onClick={(event) => event.stopPropagation()}
                        onKeyDown={(event) => {
                          event.stopPropagation();
                          if (event.key === "Enter") void saveLibraryName(dir.id);
                          if (event.key === "Escape") cancelRenamingLibrary();
                        }}
                        aria-label={t("settings.scanDirectories.libraryName")}
                        className="h-8 max-w-sm"
                      />
                    ) : (
                      <div className="flex items-center gap-2 min-w-0">
                        <span className="text-text-primary font-medium truncate text-sm">
                          {dir.name}
                        </span>
                        {dir.isActive && (
                          <Badge variant="info">{t("settings.scanDirectories.active")}</Badge>
                        )}
                      </div>
                    )}
                    <div className="text-xs text-text-muted mt-0.5 truncate" title={dir.path}>
                      {dir.path}
                    </div>
                    <div className="text-xs text-text-muted mt-0.5">
                      {t("settings.scanDirectories.metadataFormat", {
                        format: dir.metadataFormat,
                      })}
                    </div>
                  </div>
                </div>

                <div className="flex items-center gap-2">
                  {editingLibraryId === dir.id ? (
                    <>
                      <IconButton
                        size="sm"
                        variant="primary"
                        onClick={(event) => {
                          event.stopPropagation();
                          void saveLibraryName(dir.id);
                        }}
                        disabled={!editingLibraryName.trim()}
                        title={t("common.save")}
                        aria-label={t("common.save")}
                      >
                        <Check className="w-4 h-4" />
                      </IconButton>
                      <IconButton
                        size="sm"
                        onClick={(event) => {
                          event.stopPropagation();
                          cancelRenamingLibrary();
                        }}
                        title={t("common.cancel")}
                        aria-label={t("common.cancel")}
                      >
                        <X className="w-4 h-4" />
                      </IconButton>
                    </>
                  ) : (
                    <IconButton
                      size="sm"
                      onClick={(event) => {
                        event.stopPropagation();
                        startRenamingLibrary(dir.id, dir.name);
                      }}
                      title={t("settings.scanDirectories.rename")}
                      aria-label={t("settings.scanDirectories.rename")}
                    >
                      <Pencil className="w-4 h-4" />
                    </IconButton>
                  )}
                  <IconButton
                    size="sm"
                    onClick={(event) => {
                      event.stopPropagation();
                      void handleActivateLibrary(dir.id, true);
                    }}
                    disabled={isScanning || isBatchScraping}
                    title={t("settings.scanDirectories.scanThisLibrary")}
                    aria-label={t("settings.scanDirectories.scanThisLibrary")}
                  >
                    <RefreshCw
                      className={clsx("w-4 h-4", isScanning && "animate-spin")}
                    />
                  </IconButton>
                  <IconButton
                    size="sm"
                    variant="danger"
                    onClick={(event) => {
                      event.stopPropagation();
                      void handleRemoveLibrary(dir.id);
                    }}
                    disabled={isScanning || isBatchScraping}
                    title={t("common.delete")}
                    aria-label={t("common.delete")}
                  >
                    <Trash2 className="w-4 h-4" />
                  </IconButton>
                </div>
              </div>
            ))
          )}
        </div>
      </section>

      {/* 存储设置 */}
      <section>
        <h2 className="text-lg font-medium text-text-primary mb-4">
          {t("settings.storage.title")}
        </h2>

        <div className="space-y-4">
          <div>
            <label className="block text-sm text-text-secondary mb-1">
              {t("settings.storage.configDirectory")}
            </label>
            <Input
              type="text"
              value={configDir ?? t("settings.storage.defaultLocation")}
              readOnly
            />
          </div>
          <div>
            <label className="block text-sm text-text-secondary mb-1">
              {t("settings.storage.mediaDirectory")}
            </label>
            <Input
              type="text"
              value={mediaDir ?? t("settings.storage.defaultLocation")}
              readOnly
            />
          </div>
        </div>
      </section>

      {/* 添加目录弹窗 */}
      <Dialog
        open={isAddDialogOpen}
        onClose={() => setIsAddDialogOpen(false)}
        title={t("settings.scanDirectories.dialogTitle")}
        footer={
          <>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setIsAddDialogOpen(false)}
            >
              {t("common.cancel")}
            </Button>
            <Button size="sm" disabled={!isValidPath} onClick={handleAddDirectory}>
              {t("settings.scanDirectories.addDirectory")}
            </Button>
          </>
        }
      >
        <DirectoryInput
          value={newDirPath}
          onChange={setNewDirPath}
          onValidPath={(v) =>
            setIsValidPath(v.exists && v.is_directory && v.readable)
          }
          placeholder={t("directoryInput.placeholder")}
        />
      </Dialog>

      {/* 元数据导入对话框 */}
      <MetadataImportDialog
        isOpen={isMetadataDialogOpen}
        onClose={() => setIsMetadataDialogOpen(false)}
        metadataFiles={detectedMetadata}
        onImport={handleMetadataImport}
        onSkip={handleSkipImport}
      />

      {/* 根目录扫描对话框 */}
      <RootDirectoryDialog
        isOpen={isRootDialogOpen}
        onClose={() => setIsRootDialogOpen(false)}
        subDirectories={detectedSubDirs}
        onImportAsRoot={handleImportAsRoot}
        onSelectSubDirectory={handleSelectSubDirectory}
      />
    </div>
  );
}
