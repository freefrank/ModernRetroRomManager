import { useState, useEffect } from "react";
import { Search, LayoutGrid, List, Filter, Plus, Ghost, Database, X, Grid3X3 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { useRomStore } from "@/stores/romStore";
import { useAppStore } from "@/stores/appStore";
import { clsx } from "clsx";
import { useDebounce } from "@/hooks/useDebounce";
import type { Rom } from "@/types";

import RomView from "@/components/rom/RomView";
import RomDetail from "@/components/rom/RomDetail";
import BatchScrapeDialog from "@/components/rom/BatchScrapeDialog";
import DirectoryInput from "@/components/common/DirectoryInput";
import MetadataImportDialog from "@/components/common/MetadataImportDialog";
import RootDirectoryDialog from "@/components/common/RootDirectoryDialog";

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

export default function Library() {
  const { t } = useTranslation();
  const {
    roms,
    fetchRoms,
    addScanDirectory,
    stats,
    selectedRomIds,
    toggleRomSelection,
    clearSelection,
    isBatchScraping,
    batchProgress
  } = useRomStore();
  const { viewMode, setViewMode, searchQuery, setSearchQuery } = useAppStore();
  const debouncedSearch = useDebounce(searchQuery, 300);
  const [activeRom, setActiveRom] = useState<Rom | null>(null);
  const [isBatchDialogOpen, setIsBatchDialogOpen] = useState(false);
  const [isAddDirDialogOpen, setIsAddDirDialogOpen] = useState(false);
  const [newDirPath, setNewDirPath] = useState("");
  const [isValidPath, setIsValidPath] = useState(false);

  // 元数据检测状态
  const [detectedMetadata, setDetectedMetadata] = useState<MetadataFileInfo[]>([]);
  const [isMetadataDialogOpen, setIsMetadataDialogOpen] = useState(false);
  const [pendingDirPath, setPendingDirPath] = useState("");
  
  const [isRootDialogOpen, setIsRootDialogOpen] = useState(false);
  const [detectedSubDirs, setDetectedSubDirs] = useState<SubDirectoryInfo[]>([]);

  useEffect(() => {
    fetchRoms({ searchQuery: debouncedSearch });
  }, [fetchRoms, debouncedSearch]);

  const handleAddDirectory = async () => {
    if (!isValidPath || !newDirPath.trim()) return;
    try {
      // 先检测元数据文件
      const scanResult = await invoke<DirectoryScanResult>("scan_directory", { path: newDirPath });

      if (scanResult.metadata_files.length > 0) {
        // 发现元数据，显示选择对话框
        setPendingDirPath(newDirPath);
        setDetectedMetadata(scanResult.metadata_files);
        setIsAddDirDialogOpen(false);
        setIsMetadataDialogOpen(true);
      } else if (scanResult.sub_directories.length > 0) {
        // 发现子目录，可能是根目录
        setPendingDirPath(newDirPath);
        setDetectedSubDirs(scanResult.sub_directories);
        setIsAddDirDialogOpen(false);
        setIsRootDialogOpen(true);
      } else {
        // 无元数据，直接添加目录
        await addScanDirectory(newDirPath);
        setIsAddDirDialogOpen(false);
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
      setIsAddDirDialogOpen(false);
      setPendingDirPath("");
      setDetectedMetadata([]);
    } catch (error) {
      console.error("Error importing metadata:", error);
    }
  };

  const handleSkipImport = async () => {
    try {
      // 跳过元数据导入，使用 'none' 格式
      await addScanDirectory(pendingDirPath, "none");
      setIsMetadataDialogOpen(false);
      setIsAddDirDialogOpen(false);
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
      await fetchRoms();
      setIsRootDialogOpen(false);
      setPendingDirPath("");
      setDetectedSubDirs([]);
    } catch (error) {
      console.error("Error adding sub directory:", error);
    }
  };

  const handleRomClick = (rom: Rom) => {
    // Single click logic: Select if multi-select mode (Ctrl/Shift held - TODO), else open Detail
    // For now: Always open detail, Selection is handled by dedicated checkbox or specialized interactions
    setActiveRom(rom);
  };

  return (
    <div className="flex flex-col h-full space-y-6 max-w-[1600px] mx-auto w-full relative">
      {/* Batch Actions Toolbar (Floating) */}
      <div
        className={clsx(
          "fixed bottom-8 left-1/2 -translate-x-1/2 z-30 transition-all duration-300 ease-out transform",
          selectedRomIds.size > 0 ? "translate-y-0 opacity-100" : "translate-y-20 opacity-0 pointer-events-none"
        )}
      >
        <div className="bg-bg-secondary/90 backdrop-blur-xl border border-border-default rounded-2xl shadow-2xl p-2 flex items-center gap-4 px-4">
          <div className="text-text-primary font-medium pl-2 border-r border-border-default pr-4">
            <span className="text-accent-primary font-bold">{selectedRomIds.size}</span> Selected
          </div>

          <button
            onClick={() => setIsBatchDialogOpen(true)}
            disabled={isBatchScraping}
            className="flex items-center gap-2 px-4 py-2 bg-accent-primary hover:bg-accent-primary/90 text-text-primary rounded-xl transition-colors font-medium shadow-lg shadow-accent-primary/20"
          >
            <Database className="w-4 h-4" />
            Batch Scrape
          </button>

          <button
            onClick={clearSelection}
            className="px-4 py-2 hover:bg-bg-tertiary text-text-primary rounded-xl transition-colors text-sm"
          >
            Cancel
          </button>
        </div>
      </div>

      {/* Batch Progress Bar (Top) */}
      {isBatchScraping && batchProgress && (
        <div className="fixed top-0 left-0 right-0 z-50 bg-bg-secondary border-b border-accent-primary/30 shadow-lg">
          <div className="h-1 bg-bg-tertiary">
            <div
              className="h-full bg-accent-primary transition-all duration-300 ease-out"
              style={{ width: `${(batchProgress.current / batchProgress.total) * 100}%` }}
            />
          </div>
          <div className="max-w-[1600px] mx-auto px-6 py-2 flex items-center justify-between text-xs font-medium">
            <span className="text-text-primary flex items-center gap-2">
              <span className="w-2 h-2 rounded-full bg-accent-primary animate-pulse" />
              Batch Scraping...
            </span>
            <span className="text-text-muted">{batchProgress.message} ({batchProgress.current}/{batchProgress.total})</span>
          </div>
        </div>
      )}

      {/* Header Section */}
      <div className={clsx("flex flex-col gap-6 md:flex-row md:items-center md:justify-between sticky top-0 z-10 bg-bg-primary/50 backdrop-blur-md py-4 -mt-4 pt-8 transition-all", isBatchScraping && "mt-8")}>
        <div>
          <h1 className="text-4xl font-bold tracking-tight text-text-primary mb-2">
            {t("library.title")}
          </h1>
          <p className="text-text-secondary font-medium">
            {t("library.gameCount", { count: stats.totalRoms })}
          </p>
        </div>

        <div className="flex items-center gap-4">
          {/* Spotlight Search */}
          <div className="relative group">
            <div className="absolute -inset-0.5 bg-gradient-to-r from-accent-primary to-accent-secondary rounded-xl blur opacity-20 group-hover:opacity-40 transition duration-200"></div>
            <div className="relative flex items-center bg-bg-secondary rounded-xl border border-border-default w-full md:w-80 transition-colors focus-within:border-accent-primary/50 focus-within:bg-bg-tertiary">
              <Search className="w-5 h-5 text-text-muted ml-4" />
              <input
                type="text"
                placeholder={t("library.search")}
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="w-full bg-transparent border-none focus:ring-0 text-sm px-3 py-3 text-text-primary placeholder:text-text-muted focus:outline-none"
              />
              <div className="hidden md:flex items-center gap-1 pr-3">
                <kbd className="hidden sm:inline-block px-2 py-0.5 rounded text-[10px] font-mono font-medium bg-bg-tertiary text-text-muted border border-border-default">⌘K</kbd>
              </div>
            </div>
          </div>

          {/* View Toggle & Filters */}
          <div className="flex items-center gap-2 p-1 bg-bg-secondary rounded-xl border border-border-default">
            <button
              onClick={() => setViewMode("cover")}
              className={clsx(
                "p-2 rounded-lg transition-all",
                viewMode === "cover"
                  ? "bg-accent-primary text-text-primary shadow-lg shadow-accent-primary/20"
                  : "text-text-muted hover:text-text-primary hover:bg-bg-tertiary"
              )}
              title={t("library.viewMode.cover")}
            >
              <Grid3X3 className="w-5 h-5" />
            </button>
            <button
              onClick={() => setViewMode("grid")}
              className={clsx(
                "p-2 rounded-lg transition-all",
                viewMode === "grid"
                  ? "bg-accent-primary text-text-primary shadow-lg shadow-accent-primary/20"
                  : "text-text-muted hover:text-text-primary hover:bg-bg-tertiary"
              )}
              title={t("library.viewMode.grid")}
            >
              <LayoutGrid className="w-5 h-5" />
            </button>
            <button
              onClick={() => setViewMode("list")}
              className={clsx(
                "p-2 rounded-lg transition-all",
                viewMode === "list"
                  ? "bg-accent-primary text-text-primary shadow-lg shadow-accent-primary/20"
                  : "text-text-muted hover:text-text-primary hover:bg-bg-tertiary"
              )}
              title={t("library.viewMode.list")}
            >
              <List className="w-5 h-5" />
            </button>
          </div>

          <button className="p-3 rounded-xl bg-bg-secondary border border-border-default text-text-secondary hover:text-text-primary hover:border-border-hover hover:bg-bg-tertiary transition-all">
            <Filter className="w-5 h-5" />
          </button>
        </div>
      </div>

      {/* Content Area */}
      <div className="flex-1 min-h-0">
        {roms.length === 0 ? (
          <div className="flex flex-col items-center justify-center min-h-[400px]">
            <div className="text-center max-w-md mx-auto relative group cursor-default mb-16">
              {/* Glowing Effect Background */}
              <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-64 h-64 bg-accent-primary/10 rounded-full blur-[80px] group-hover:bg-accent-primary/20 transition-all duration-700"></div>

              <div className="relative">
                <div className="w-32 h-32 mx-auto mb-8 rounded-3xl bg-gradient-to-br from-bg-tertiary to-bg-secondary border border-border-default flex items-center justify-center shadow-2xl group-hover:scale-105 transition-transform duration-500 ring-1 ring-border-default">
                  <Ghost className="w-16 h-16 text-text-muted group-hover:text-accent-primary transition-colors duration-300" />
                </div>

                <h2 className="text-3xl font-bold text-text-primary mb-3 tracking-tight">
                  {t("library.empty.title")}
                </h2>
                <p className="text-text-secondary mb-8 text-lg leading-relaxed">
                  {t("library.empty.description")}
                </p>

                <button
                  onClick={() => setIsAddDirDialogOpen(true)}
                  className="relative inline-flex group/btn"
                >
                  <div className="absolute transition-all duration-300 opacity-70 -inset-px bg-gradient-to-r from-accent-primary to-accent-secondary rounded-xl blur-lg group-hover/btn:opacity-100 group-hover/btn:-inset-1 group-hover/btn:duration-200 animate-tilt"></div>
                  <div className="relative inline-flex items-center gap-2 px-8 py-4 bg-bg-primary rounded-xl leading-none text-text-primary transition duration-200 border border-border-default hover:bg-bg-secondary">
                    <Plus className="w-5 h-5 text-accent-secondary" />
                    <span className="font-semibold tracking-wide">{t("library.empty.addDirectory")}</span>
                  </div>
                </button>
              </div>
            </div>
          </div>
        ) : (
          <RomView
            roms={roms}
            viewMode={viewMode}
            selectedIds={selectedRomIds}
            onRomClick={handleRomClick}
            onToggleSelect={(id) => toggleRomSelection(id, true)}
          />
        )}
      </div>

      {/* Detail Panel */}
      <RomDetail rom={activeRom} onClose={() => setActiveRom(null)} />

      {/* Batch Scrape Dialog */}
      <BatchScrapeDialog
        isOpen={isBatchDialogOpen}
        onClose={() => setIsBatchDialogOpen(false)}
      />

      {/* Add Directory Dialog */}
      {isAddDirDialogOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
          <div
            className="absolute inset-0 bg-black/60 backdrop-blur-sm"
            onClick={() => setIsAddDirDialogOpen(false)}
          />
          <div className="relative w-full max-w-md bg-bg-primary border border-border-default rounded-2xl shadow-2xl p-6">
            <div className="flex items-center justify-between mb-6">
              <h3 className="text-lg font-bold text-text-primary">{t("settings.scanDirectories.addDirectory")}</h3>
              <button
                onClick={() => setIsAddDirDialogOpen(false)}
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
                onClick={() => setIsAddDirDialogOpen(false)}
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

