import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { useRomStore } from "@/stores/romStore";
import { useAppStore, THEMES } from "@/stores/appStore";
import { useScraperStore } from "@/stores/scraperStore";
import { Folder, Trash2, RefreshCw, Plus, HardDrive, X, Settings2, Key, Globe, Shield, Activity, Save, Info, GripVertical } from "lucide-react";
import { clsx } from "clsx";
import DirectoryInput from "@/components/common/DirectoryInput";
import MetadataImportDialog from "@/components/common/MetadataImportDialog";
import RootDirectoryDialog from "@/components/common/RootDirectoryDialog";
import type { ScraperCredentials } from "@/types";

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

export default function Settings() {
  const { t } = useTranslation();
  const { theme, setTheme } = useAppStore();
  const {
    scanDirectories,
    fetchScanDirectories,
    addScanDirectory,
    removeScanDirectory,
    isScanning,
    scanProgress,
    fetchRoms,
  } = useRomStore();

  // Scraper store
  const { providers, fetchProviders, configureProvider, setProviderEnabled, setProviderPriority } = useScraperStore();
  const [editingProvider, setEditingProvider] = useState<string | null>(null);
  const [credentials, setCredentials] = useState<ScraperCredentials>({});

  // 拖拽排序状态（改用 mouse 事件）
  const [draggedProvider, setDraggedProvider] = useState<string | null>(null);
  const [dragOverProvider, setDragOverProvider] = useState<string | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [dragOffset, setDragOffset] = useState({ x: 0, y: 0 });
  const [mousePos, setMousePos] = useState({ x: 0, y: 0 });

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
    fetchProviders();
  }, [fetchProviders]);

  // 全局 mouseUp 和 mouseMove 监听器
  useEffect(() => {
    if (isDragging) {
      window.addEventListener('mouseup', handleMouseUp);
      window.addEventListener('mousemove', handleMouseMove);
      return () => {
        window.removeEventListener('mouseup', handleMouseUp);
        window.removeEventListener('mousemove', handleMouseMove);
      };
    }
  }, [isDragging, draggedProvider, dragOverProvider]);

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

  const handleAddDirectory = async () => {
    if (!isValidPath || !newDirPath.trim()) return;
    try {
      const scanResult = await invoke<DirectoryScanResult>("scan_directory", { path: newDirPath });

      if (scanResult.metadata_files.length > 0) {
        setPendingDirPath(newDirPath);
        setDetectedMetadata(scanResult.metadata_files);
        setIsAddDialogOpen(false);
        setIsMetadataDialogOpen(true);
      } else if (scanResult.is_root_directory && scanResult.sub_directories.length > 0) {
        setPendingDirPath(newDirPath);
        setDetectedSubDirs(scanResult.sub_directories);
        setIsAddDialogOpen(false);
        setIsRootDialogOpen(true);
      } else if (scanResult.sub_directories.length > 0) {
        setPendingDirPath(newDirPath);
        setDetectedSubDirs(scanResult.sub_directories);
        setIsAddDialogOpen(false);
        setIsRootDialogOpen(true);
      } else {
        await addScanDirectory(newDirPath);
        setIsAddDialogOpen(false);
      }

      setNewDirPath("");
      setIsValidPath(false);
    } catch (error) {
      console.error("Error adding directory:", error);
    }
  };

  const handleMetadataImport = async (file: MetadataFileInfo) => {
    try {
      // 添加目录，记录元数据格式
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

  const handleSelectSubDirectory = async (subDir: SubDirectoryInfo, format: string) => {
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

  // Scraper 相关函数
  const handleToggleEnabled = async (providerId: string, enabled: boolean) => {
    try {
      await setProviderEnabled(providerId, enabled);
    } catch (error) {
      console.error("Failed to toggle provider:", error);
    }
  };

  const handleEditConfig = (provider: any) => {
    setEditingProvider(provider.id);
    setCredentials({});
  };

  const handleSaveConfig = async () => {
    if (!editingProvider) return;
    try {
      await configureProvider(editingProvider, credentials);
      setEditingProvider(null);
    } catch (error) {
      console.error("Failed to save credentials:", error);
    }
  };

  // 拖拽处理函数（改用 mouse 事件）
  const handleMouseDown = (e: React.MouseEvent, providerId: string) => {
    // 如果点击的是按钮或开关，不启动拖拽
    const target = e.target as HTMLElement;
    if (target.closest('button') || target.closest('label')) {
      return;
    }

    console.log("🎯 Mouse down:", providerId);

    // 记录鼠标相对于元素的偏移
    const rect = e.currentTarget.getBoundingClientRect();
    setDragOffset({
      x: e.clientX - rect.left,
      y: e.clientY - rect.top
    });
    setMousePos({ x: e.clientX, y: e.clientY });

    setDraggedProvider(providerId);
    setIsDragging(true);
  };

  const handleMouseEnter = (providerId: string) => {
    if (isDragging && draggedProvider && draggedProvider !== providerId) {
      console.log("📍 Mouse enter:", providerId);
      setDragOverProvider(providerId);
    }
  };

  const handleMouseMove = (e: MouseEvent) => {
    if (isDragging) {
      setMousePos({ x: e.clientX, y: e.clientY });
    }
  };

  const handleMouseUp = async () => {
    if (!isDragging || !draggedProvider || !dragOverProvider) {
      setDraggedProvider(null);
      setDragOverProvider(null);
      setIsDragging(false);
      return;
    }

    console.log("🎯 Mouse up - Drop on:", dragOverProvider, "from:", draggedProvider);

    // 获取排序后的 provider 列表
    const sortedProviders = [...providers].sort((a, b) => a.priority - b.priority);
    const draggedIndex = sortedProviders.findIndex(p => p.id === draggedProvider);
    const targetIndex = sortedProviders.findIndex(p => p.id === dragOverProvider);

    if (draggedIndex === -1 || targetIndex === -1) {
      setDraggedProvider(null);
      setDragOverProvider(null);
      setIsDragging(false);
      return;
    }

    // 重新排列
    const newProviders = [...sortedProviders];
    const [removed] = newProviders.splice(draggedIndex, 1);
    newProviders.splice(targetIndex, 0, removed);

    // 重新计算优先级（从10开始，每个间隔10）
    try {
      for (let i = 0; i < newProviders.length; i++) {
        const newPriority = (i + 1) * 10;
        if (newProviders[i].priority !== newPriority) {
          await setProviderPriority(newProviders[i].id, newPriority);
        }
      }
    } catch (error) {
      console.error("Failed to update provider priorities:", error);
    }

    setDraggedProvider(null);
    setDragOverProvider(null);
    setIsDragging(false);
  };

  // 按优先级排序的 providers
  const sortedProviders = [...providers].sort((a, b) => a.priority - b.priority);

  const getProviderIcon = (id: string) => {
    switch (id) {
      case "screenscraper": return <Globe className="w-5 h-5" />;
      case "steamgriddb": return <Activity className="w-5 h-5" />;
      default: return <Settings2 className="w-5 h-5" />;
    }
  };

  const getProviderColor = (id: string) => {
    switch (id) {
      case "screenscraper": return "bg-red-500/20 text-red-400 border-red-500/30";
      case "steamgriddb": return "bg-blue-500/20 text-blue-400 border-blue-500/30";
      default: return "bg-bg-tertiary text-text-primary border-border-default";
    }
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
              <h2 className="text-lg font-medium text-text-primary mb-4">{t("settings.appearance.title")}</h2>

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
                  <div key={dir.path} className="group p-4 bg-bg-secondary border border-border-default rounded-xl hover:border-border-hover transition-all flex items-center justify-between">
                    <div className="flex items-center gap-4 overflow-hidden">
                      <div className="w-10 h-10 bg-bg-tertiary rounded-lg flex items-center justify-center flex-shrink-0">
                        <HardDrive className="w-5 h-5 text-accent-secondary" />
                      </div>
                      <div className="min-w-0">
                        <div className="text-text-primary font-medium truncate text-sm" title={dir.path}>{dir.path}</div>
                        <div className="text-xs text-text-muted mt-0.5">
                          Metadata: {dir.metadataFormat}
                        </div>
                      </div>
                    </div>

                    <div className="flex items-center gap-2">
                      <button
                        onClick={handleScan}
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
                        onClick={() => removeScanDirectory(dir.path)}
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

          <section>
            <h2 className="text-lg font-medium text-text-primary mb-4">{t("settings.storage.title")}</h2>

            <div className="space-y-4">
              <div>
                <label className="block text-sm text-text-secondary mb-1">{t("settings.storage.configDirectory")}</label>
                <div className="flex gap-2">
                  <input
                    type="text"
                    value={configDir ?? t("settings.storage.defaultLocation")}
                    readOnly
                    className="flex-1 px-3 py-2 bg-bg-secondary border border-border-default rounded-lg text-sm text-text-secondary focus:outline-none"
                  />
                </div>
              </div>
              <div>
                <label className="block text-sm text-text-secondary mb-1">{t("settings.storage.mediaDirectory")}</label>
                <div className="flex gap-2">
                  <input
                    type="text"
                    value={mediaDir ?? t("settings.storage.defaultLocation")}
                    readOnly
                    className="flex-1 px-3 py-2 bg-bg-secondary border border-border-default rounded-lg text-sm text-text-secondary focus:outline-none"
                  />
                </div>
              </div>
            </div>
          </section>

          {/* API 配置 */}
          <section>
            <h2 className="text-lg font-medium text-text-primary mb-4 flex items-center gap-2">
              <Shield className="w-5 h-5 text-accent-primary" />
              API 配置
            </h2>

            <div className="grid grid-cols-1 gap-4">
              {sortedProviders.map((p) => (
                <div
                  key={p.id}
                  onMouseDown={(e) => handleMouseDown(e, p.id)}
                  onMouseEnter={() => handleMouseEnter(p.id)}
                  className={clsx(
                    "group relative overflow-hidden rounded-2xl border transition-all duration-300 cursor-move select-none",
                    p.enabled ? "bg-bg-secondary border-border-hover" : "bg-bg-primary/50 border-border-default opacity-70",
                    draggedProvider === p.id && "opacity-50 scale-95",
                    dragOverProvider === p.id && "ring-2 ring-accent-primary"
                  )}
                >
                  <div className="flex items-center p-5">
                    <div className="cursor-grab active:cursor-grabbing text-text-muted hover:text-accent-primary transition-colors mr-3">
                      <GripVertical className="w-5 h-5" />
                    </div>
                    <div className={clsx("w-12 h-12 rounded-xl flex items-center justify-center border", getProviderColor(p.id))}>
                      {getProviderIcon(p.id)}
                    </div>

                    <div className="ml-4 flex-1">
                      <div className="flex items-center gap-2">
                        <h3 className="font-bold text-text-primary text-lg">{p.name}</h3>
                        {p.has_credentials && (
                          <span className="px-2 py-0.5 rounded-md bg-green-500/10 text-green-400 text-[10px] font-bold uppercase tracking-tighter border border-green-500/20">
                            已认证
                          </span>
                        )}
                      </div>
                      <div className="flex items-center gap-3 mt-1">
                        <span className="text-xs text-text-muted flex items-center gap-1">
                          <Activity className="w-3 h-3" />
                          {p.capabilities.join(", ").toUpperCase()}
                        </span>
                      </div>
                    </div>

                    <div className="flex items-center gap-4">
                      <button
                        onClick={() => handleEditConfig(p)}
                        className="p-2.5 rounded-xl bg-bg-tertiary text-text-secondary hover:text-accent-primary hover:bg-bg-primary transition-all border border-transparent hover:border-accent-primary/30"
                        title="编辑配置"
                      >
                        <Key className="w-5 h-5" />
                      </button>

                      <label className="relative inline-flex items-center cursor-pointer">
                        <input
                          type="checkbox"
                          className="sr-only peer"
                          checked={p.enabled}
                          onChange={(e) => handleToggleEnabled(p.id, e.target.checked)}
                        />
                        <div className="w-12 h-6 bg-bg-tertiary rounded-full border border-border-default peer peer-checked:bg-accent-primary peer-checked:border-accent-primary after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-text-primary after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-6 shadow-sm"></div>
                      </label>
                    </div>
                  </div>

                  {/* 配置编辑展开面板 */}
                  {editingProvider === p.id && (
                    <div className="px-5 pb-5 border-t border-border-default bg-bg-primary/30 animate-in slide-in-from-top-2 duration-200">
                      <div className="pt-5 space-y-4">
                        <div className="flex items-start gap-3 p-3 bg-blue-500/10 border border-blue-500/20 rounded-xl mb-4">
                          <Info className="w-4 h-4 text-blue-400 mt-0.5" />
                          <p className="text-xs text-blue-200 leading-relaxed">
                            请输入 API 凭证。这些信息将安全存储在本地配置文件中。
                          </p>
                        </div>

                        {p.id === "screenscraper" && (
                          <>
                            <div>
                              <label className="block text-sm font-medium text-text-primary mb-2">用户名</label>
                              <input
                                type="text"
                                value={credentials.username || ""}
                                onChange={(e) => setCredentials({ ...credentials, username: e.target.value })}
                                className="w-full px-3 py-2 bg-bg-secondary border border-border-default rounded-lg text-sm text-text-primary focus:outline-none focus:border-accent-primary"
                                placeholder="ScreenScraper 用户名"
                              />
                            </div>
                            <div>
                              <label className="block text-sm font-medium text-text-primary mb-2">密码</label>
                              <input
                                type="password"
                                value={credentials.password || ""}
                                onChange={(e) => setCredentials({ ...credentials, password: e.target.value })}
                                className="w-full px-3 py-2 bg-bg-secondary border border-border-default rounded-lg text-sm text-text-primary focus:outline-none focus:border-accent-primary"
                                placeholder="ScreenScraper 密码"
                              />
                            </div>
                          </>
                        )}

                        {p.id === "steamgriddb" && (
                          <div>
                            <label className="block text-sm font-medium text-text-primary mb-2">API Key</label>
                            <input
                              type="text"
                              value={credentials.api_key || ""}
                              onChange={(e) => setCredentials({ ...credentials, api_key: e.target.value })}
                              className="w-full px-3 py-2 bg-bg-secondary border border-border-default rounded-lg text-sm text-text-primary focus:outline-none focus:border-accent-primary font-mono"
                              placeholder="SteamGridDB API Key"
                            />
                          </div>
                        )}

                        <div className="flex justify-end gap-3 pt-2">
                          <button
                            onClick={() => setEditingProvider(null)}
                            className="px-4 py-2 rounded-lg bg-bg-tertiary text-text-secondary hover:bg-bg-primary transition-all text-sm font-medium"
                          >
                            取消
                          </button>
                          <button
                            onClick={handleSaveConfig}
                            className="px-4 py-2 rounded-lg bg-accent-primary text-white hover:opacity-90 transition-all text-sm font-medium flex items-center gap-2"
                          >
                            <Save className="w-4 h-4" />
                            保存
                          </button>
                        </div>
                      </div>
                    </div>
                  )}
                </div>
              ))}
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
                  <p className="text-text-secondary text-sm">v{APP_VERSION}</p>


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
                placeholder={t("directoryInput.placeholder")}
              />


            <div className="flex justify-end gap-3 mt-6">
              <button
                onClick={() => setIsAddDialogOpen(false)}
                className="px-4 py-2 rounded-xl text-text-primary hover:bg-bg-tertiary transition-colors text-sm font-medium"
              >
                {t("common.cancel")}
              </button>
              <button
                onClick={handleAddDirectory}
                disabled={!isValidPath}
                className="px-6 py-2 bg-accent-primary hover:bg-accent-primary/90 text-text-primary rounded-xl font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed text-sm"
              >
                {t("settings.scanDirectories.addDirectory")}
              </button>
            </div>
          </div>
        </div>
      )}

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

      {/* 拖拽幽灵元素 */}
      {isDragging && draggedProvider && (
        <div
          className="fixed pointer-events-none z-50 opacity-80"
          style={{
            left: mousePos.x - dragOffset.x,
            top: mousePos.y - dragOffset.y,
            width: '600px'
          }}
        >
          {(() => {
            const provider = providers.find(p => p.id === draggedProvider);
            if (!provider) return null;

            return (
              <div className="group relative overflow-hidden rounded-2xl border bg-bg-secondary border-border-hover shadow-2xl">
                <div className="flex items-center p-5">
                  <div className="cursor-grab text-text-muted mr-3">
                    <GripVertical className="w-5 h-5" />
                  </div>
                  <div className={clsx("w-12 h-12 rounded-xl flex items-center justify-center border", getProviderColor(provider.id))}>
                    {getProviderIcon(provider.id)}
                  </div>
                  <div className="ml-4 flex-1">
                    <div className="flex items-center gap-2">
                      <h3 className="font-bold text-text-primary text-lg">{provider.name}</h3>
                      {provider.has_credentials && (
                        <span className="px-2 py-0.5 rounded-md bg-green-500/10 text-green-400 text-[10px] font-bold uppercase tracking-tighter border border-green-500/20">
                          已认证
                        </span>
                      )}
                    </div>
                    <div className="flex items-center gap-3 mt-1">
                      <span className="text-xs text-text-muted flex items-center gap-1">
                        <Activity className="w-3 h-3" />
                        {provider.capabilities.join(", ").toUpperCase()}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            );
          })()}
        </div>
      )}
    </div>
  );
}
