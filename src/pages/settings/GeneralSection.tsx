import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { Folder, HardDrive, Plus, RefreshCw, Trash2 } from "lucide-react";
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
  Dialog,
  EmptyState,
  IconButton,
  Input,
  Select,
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
  const { language, setLanguage, viewMode, setViewMode } = useAppStore();
  const {
    scanDirectories,
    fetchScanDirectories,
    addScanDirectory,
    removeScanDirectory,
    isScanning,
    scanProgress,
    fetchRoms,
  } = useRomStore();

  const [isAddDialogOpen, setIsAddDialogOpen] = useState(false);
  const [newDirPath, setNewDirPath] = useState("");
  const [isValidPath, setIsValidPath] = useState(false);
  const [configDir, setConfigDir] = useState<string | null>(null);
  const [mediaDir, setMediaDir] = useState<string | null>(null);

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
        }
      }

      setNewDirPath("");
      setIsValidPath(false);
    } catch (error) {
      console.error("Error adding directory:", error);
    }
  };

  const handleMetadataImport = async (file: MetadataFileInfo) => {
    try {
      await addScanDirectory(pendingDirPath, file.format);
      setIsMetadataDialogOpen(false);
      setIsAddDialogOpen(false);
      setPendingDirPath("");
      setDetectedMetadata([]);
    } catch (error) {
      console.error("Error importing metadata:", error);
    }
  };

  const handleSkipImport = async () => {
    try {
      await addScanDirectory(pendingDirPath, "none");
      setIsMetadataDialogOpen(false);
      setIsAddDialogOpen(false);
      setPendingDirPath("");
      setDetectedMetadata([]);
    } catch (error) {
      console.error("Error adding directory:", error);
    }
  };

  const handleImportAsRoot = async () => {
    try {
      await invoke("add_directory", {
        path: pendingDirPath,
        metadataFormat: "auto",
        isRoot: true,
        systemId: null,
      });
      await fetchScanDirectories();
      await fetchRoms();
      setIsRootDialogOpen(false);
      setPendingDirPath("");
      setDetectedSubDirs([]);
    } catch (error) {
      console.error("Error adding root directory:", error);
    }
  };

  const handleSelectSubDirectory = async (
    subDir: SubDirectoryInfo,
    format: string,
  ) => {
    try {
      await invoke("add_directory", {
        path: subDir.path,
        metadataFormat: format,
        isRoot: false,
        systemId: subDir.name,
      });
      await fetchScanDirectories();
      await fetchRoms();
      setIsRootDialogOpen(false);
      setPendingDirPath("");
      setDetectedSubDirs([]);
    } catch (error) {
      console.error("Error adding sub directory:", error);
    }
  };

  const handleScan = async () => {
    await Promise.all([fetchScanDirectories(), fetchRoms()]);
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

      {/* 扫描目录 */}
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
          <Button size="sm" onClick={() => setIsAddDialogOpen(true)}>
            <Plus className="w-4 h-4" />
            {t("settings.scanDirectories.addDirectory")}
          </Button>
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
                key={dir.path}
                className="group p-4 bg-bg-secondary border-[length:var(--border-width)] border-border-default rounded-[var(--radius-lg)] hover:border-border-hover transition-all duration-[var(--motion-fast)] ease-[var(--motion-easing)] flex items-center justify-between"
              >
                <div className="flex items-center gap-4 overflow-hidden">
                  <div className="w-10 h-10 bg-bg-tertiary rounded-[var(--radius-md)] flex items-center justify-center flex-shrink-0">
                    <HardDrive className="w-5 h-5 text-accent-secondary" />
                  </div>
                  <div className="min-w-0">
                    <div
                      className="text-text-primary font-medium truncate text-sm"
                      title={dir.path}
                    >
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
                  <IconButton
                    size="sm"
                    onClick={handleScan}
                    disabled={isScanning}
                    title={t("common.refresh")}
                    aria-label={t("common.refresh")}
                  >
                    <RefreshCw
                      className={clsx("w-4 h-4", isScanning && "animate-spin")}
                    />
                  </IconButton>
                  <IconButton
                    size="sm"
                    variant="danger"
                    onClick={() => removeScanDirectory(dir.path)}
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
