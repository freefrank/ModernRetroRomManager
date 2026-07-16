import { useState, useMemo, useEffect, useRef } from "react";
import { useParams, useNavigate, Link } from "react-router-dom";
import {
  Search,
  LayoutGrid,
  List,
  Ghost,
  Database,
  Grid3X3,
  Wand2,
  ChevronRight,
  SearchX,
  Languages,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import { useRomStore } from "@/stores/romStore";
import { useAppStore } from "@/stores/appStore";
import { clsx } from "clsx";
import { useDebounce } from "@/hooks/useDebounce";
import type { Rom, ViewMode } from "@/types";
import { aiTranslationApi, ps3Api, scraperApi } from "@/lib/api";
import { romMetadata } from "@/lib/metadataTranslation";
import { groupTranslationRequests } from "@/lib/translationBatch";
import { Button, Dialog, EmptyState, IconButton, Input, toast } from "@/components/ui";

import RomView from "@/components/rom/RomView";
import RomDetail from "@/components/rom/RomDetail";
import BatchScrapeDialog from "@/components/rom/BatchScrapeDialog";

const VIEW_MODES: { mode: ViewMode; icon: typeof Grid3X3; labelKey: string }[] = [
  { mode: "cover", icon: Grid3X3, labelKey: "library.viewMode.cover" },
  { mode: "grid", icon: LayoutGrid, labelKey: "library.viewMode.grid" },
  { mode: "list", icon: List, labelKey: "library.viewMode.list" },
];

/** 单系统库页:/library/:systemId */
export default function Library() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { systemId } = useParams<{ systemId: string }>();
  const systemName = systemId ? decodeURIComponent(systemId) : "";

  const {
    fetchRoms,
    fetchSystemRoms,
    isLoadingRoms,
    selectedRomIds,
    toggleRomSelection,
    selectAllRoms,
    clearSelection,
    isBatchScraping,
    batchProgress,
    systemRoms,
    setSelectedSystem,
    scanDirectories,
  } = useRomStore();
  const activeLibrary = scanDirectories.find(item => item.isActive);
  const { viewMode, setViewMode, searchQuery, setSearchQuery } = useAppStore();
  const debouncedSearch = useDebounce(searchQuery, 300);
  const [activeRom, setActiveRom] = useState<Rom | null>(null);
  const [isBatchDialogOpen, setIsBatchDialogOpen] = useState(false);
  const [batchScope, setBatchScope] = useState<"selection" | "platform">("selection");
  const [isGeneratingBoxart, setIsGeneratingBoxart] = useState(false);
  const [boxartProgress, setBoxartProgress] = useState({ current: 0, total: 0 });
  const [cardScale, setCardScale] = useState(1);
  const [translateScope, setTranslateScope] = useState<"selection" | "platform">("selection");
  const [isTranslateDialogOpen, setIsTranslateDialogOpen] = useState(false);
  const [isTranslating, setIsTranslating] = useState(false);
  const [translateProgress, setTranslateProgress] = useState({ current: 0, total: 0 });
  const translateCancelRef = useRef(false);

  const systemData = systemRoms.find((s) => s.system === systemName);
  const romsOfSystem = useMemo(() => systemData?.roms ?? [], [systemData]);

  useEffect(() => {
    if (systemName && !systemData) void fetchSystemRoms(systemName);
  }, [fetchSystemRoms, systemData, systemName]);

  // 路由即为选中系统；数据随后按需加载，批量抓取等逻辑可立即拿到目标平台。
  useEffect(() => {
    setSelectedSystem(systemName ?? null);
  }, [systemName, setSelectedSystem]);

  // 系统内搜索过滤
  const filteredRoms = useMemo(() => {
    if (!debouncedSearch) return romsOfSystem;
    const lowerQuery = debouncedSearch.toLowerCase();
    return romsOfSystem.filter((r) =>
      [r.name, r.chinese_name, r.english_name]
        .some((name) => name?.toLowerCase().includes(lowerQuery)),
    );
  }, [romsOfSystem, debouncedSearch]);

  const isPs3 = systemName.toLowerCase().includes("ps3");

  const handleBatchGenerateBoxart = async () => {
    if (!isPs3 || romsOfSystem.length === 0) return;

    setIsGeneratingBoxart(true);
    setBoxartProgress({ current: 0, total: romsOfSystem.length });

    let successCount = 0;
    let failCount = 0;

    for (let i = 0; i < romsOfSystem.length; i++) {
      const rom = romsOfSystem[i];
      setBoxartProgress({ current: i + 1, total: romsOfSystem.length });

      try {
        const result = await ps3Api.generateBoxart(rom.file, rom.directory, rom.system);
        if (result.success) {
          successCount++;
        } else {
          failCount++;
          console.error(`Failed to generate boxart for ${rom.name}:`, result.error);
        }
      } catch (error) {
        failCount++;
        console.error(`Error generating boxart for ${rom.name}:`, error);
      }
    }

    setIsGeneratingBoxart(false);
    toast.success(t("library.boxart.done", { success: successCount, failed: failCount }));

    // 刷新 ROM 列表以显示新生成的 boxart
    await fetchRoms();
  };

  const translationRoms = translateScope === "selection"
    ? romsOfSystem.filter(rom => selectedRomIds.has(rom.file))
    : romsOfSystem;

  const openTranslation = (scope: "selection" | "platform") => {
    translateCancelRef.current = false;
    setTranslateScope(scope);
    setTranslateProgress({ current: 0, total: 0 });
    setIsTranslateDialogOpen(true);
  };

  const closeTranslationDialog = () => {
    if (isTranslating) translateCancelRef.current = true;
    setIsTranslateDialogOpen(false);
  };

  const handleBatchTranslate = async () => {
    if (translationRoms.length === 0) return;
    setIsTranslating(true);
    translateCancelRef.current = false;
    setTranslateProgress({ current: 0, total: translationRoms.length });
    let success = 0;
    let failed = 0;

    const translationRequest = (rom: Rom) => ({
      system: rom.system,
      file_name: rom.file,
      metadata: romMetadata(rom),
    });

    const translateOne = async (rom: Rom) => {
      try {
        const metadata = await aiTranslationApi.translateMetadata(translationRequest(rom));
        await scraperApi.saveTempMetadata(rom.system, rom.directory, rom.file, metadata);
        success += 1;
      } catch (error) {
        failed += 1;
        console.error(`AI metadata translation failed for ${rom.file}:`, error);
      }
    };

    const config = await aiTranslationApi.getConfig().catch(() => null);
    if (config?.merge_batch_requests) {
      const batches = groupTranslationRequests(
        translationRoms,
        translationRequest,
        config.batch_token_limit,
      );
      let processed = 0;
      for (const batch of batches) {
        if (translateCancelRef.current) break;
        if (batch.length === 1) {
          await translateOne(batch[0]);
          processed += 1;
          setTranslateProgress({ current: processed, total: translationRoms.length });
          continue;
        }
        try {
          const results = await aiTranslationApi.translateMetadataBatch(batch.map(translationRequest));
          const resultByIndex = new Map(results.map(result => [result.index, result.metadata]));
          for (const [index, rom] of batch.entries()) {
            const metadata = resultByIndex.get(index);
            if (!metadata) {
              if (!translateCancelRef.current) await translateOne(rom);
              continue;
            }
            try {
              await scraperApi.saveTempMetadata(rom.system, rom.directory, rom.file, metadata);
              success += 1;
            } catch (error) {
              failed += 1;
              console.error(`AI metadata translation save failed for ${rom.file}:`, error);
            }
          }
        } catch (error) {
          console.error("Merged AI metadata translation failed; falling back to individual requests:", error);
          if (!translateCancelRef.current) {
            for (const rom of batch) {
              if (translateCancelRef.current) break;
              await translateOne(rom);
            }
          }
        }
        processed += batch.length;
        setTranslateProgress({
          current: Math.min(processed, translationRoms.length),
          total: translationRoms.length,
        });
      }
    } else {
      for (const [index, rom] of translationRoms.entries()) {
        if (translateCancelRef.current) break;
        await translateOne(rom);
        setTranslateProgress({ current: index + 1, total: translationRoms.length });
      }
    }
    setIsTranslating(false);
    if (!translateCancelRef.current) setIsTranslateDialogOpen(false);
    await fetchRoms();
    toast.success(t("library.translation.done", { success, failed }));
  };

  if (!systemData && isLoadingRoms) {
    return (
      <div className="rr-page flex h-full items-center justify-center max-w-[1600px] mx-auto w-full">
        <div className="flex items-center gap-3 text-text-secondary">
          <span className="h-5 w-5 animate-spin rounded-full border-2 border-accent-primary border-t-transparent" />
          {t("common.loading")}
        </div>
      </div>
    );
  }

  // 查无此系统:空态 + 返回货架
  if (!systemData) {
    return (
      <div className="rr-page flex h-full items-center justify-center max-w-[1600px] mx-auto w-full">
        <EmptyState
          className="w-full max-w-md"
          icon={<Ghost />}
          title={t("library.systemNotFound.title")}
          description={t("library.systemNotFound.description", { name: systemName })}
          action={
            <Button onClick={() => navigate("/library")}>
              {t("library.systemNotFound.back")}
            </Button>
          }
        />
      </div>
    );
  }

  return (
    <div className="rr-page flex flex-col h-full space-y-6 max-w-[1600px] mx-auto w-full relative">
      {/* Batch Actions Toolbar (Floating) */}
      <div
        className={clsx(
          "fixed bottom-8 left-1/2 -translate-x-1/2 z-30 transform",
          "transition-all duration-[var(--motion-normal)] ease-[var(--motion-easing)]",
          selectedRomIds.size > 0
            ? "translate-y-0 opacity-100"
            : "translate-y-20 opacity-0 pointer-events-none",
        )}
      >
        <div className="bg-bg-secondary/90 backdrop-blur-xl border-[length:var(--border-width)] border-border-default rounded-[var(--radius-lg)] [box-shadow:var(--shadow-dialog)] p-2 flex items-center gap-4 px-4">
          <div className="text-text-primary text-sm font-medium pl-2 border-r border-border-default pr-4">
            {t("library.batch.selectedCount", { count: selectedRomIds.size })}
          </div>

          <Button size="sm" onClick={() => { setBatchScope("selection"); setIsBatchDialogOpen(true); }} disabled={isBatchScraping}>
            <Database className="w-4 h-4" />
            {t("library.batch.scrape")}
          </Button>

          <Button size="sm" variant="ghost" onClick={() => openTranslation("selection")} disabled={isTranslating}>
            <Languages className="w-4 h-4" />
            {t("library.translation.selected")}
          </Button>

          <Button size="sm" variant="ghost" onClick={clearSelection}>
            {t("common.cancel")}
          </Button>
        </div>
      </div>

      {/* Batch Progress Bar (Top) */}
      {isBatchScraping && batchProgress && (
        <div className="fixed top-0 left-0 right-0 z-50 bg-bg-secondary border-b border-accent-primary/30">
          <div className="h-1 bg-bg-tertiary">
            <div
              className="h-full bg-accent-primary transition-all duration-[var(--motion-normal)] ease-[var(--motion-easing)]"
              style={{ width: `${(batchProgress.current / batchProgress.total) * 100}%` }}
            />
          </div>
          <div className="max-w-[1600px] mx-auto px-6 py-2 flex items-center justify-between text-xs font-medium">
            <span className="text-text-primary flex items-center gap-2">
              <span className="w-2 h-2 rounded-full bg-accent-primary animate-pulse" />
              {t("library.batch.scraping")}
            </span>
            <span className="text-text-muted">
              {batchProgress.message} ({batchProgress.current}/{batchProgress.total})
            </span>
          </div>
        </div>
      )}

      {/* Header Section */}
      <div
        className={clsx(
          "flex flex-col gap-6 md:flex-row md:items-center md:justify-between sticky top-0 z-10 bg-bg-primary/50 backdrop-blur-md py-4 -mt-4 pt-8 transition-all duration-[var(--motion-normal)] ease-[var(--motion-easing)]",
          isBatchScraping && "mt-8",
        )}
      >
        <div className="min-w-0">
          {/* 面包屑:ROM 库 / {系统名} */}
          <nav
            aria-label={t("library.breadcrumb")}
            className="flex items-center gap-1 text-sm text-text-secondary mb-2"
          >
            <Link
              to="/library"
              className="hover:text-text-primary transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)]"
            >
              {activeLibrary?.name ?? t("library.title")}
            </Link>
            <ChevronRight className="w-4 h-4 text-text-muted" />
            <span className="text-text-primary font-medium truncate">{systemName}</span>
          </nav>
          <h1 className="text-4xl font-bold tracking-tight text-text-primary mb-2 truncate">
            {systemName}
          </h1>
          <p className="text-text-secondary font-medium">
            {t("library.gameCount", { count: romsOfSystem.length })}
          </p>
        </div>

        <div className="flex flex-wrap items-center gap-4">
          <Button
            variant="ghost"
            onClick={() => selectAllRoms(filteredRoms.map(rom => rom.file))}
            disabled={filteredRoms.length === 0}
          >
            {t("library.batch.selectAll")}
          </Button>
          <Button
            variant="ghost"
            onClick={() => openTranslation("platform")}
            disabled={isTranslating || romsOfSystem.length === 0}
          >
            <Languages className="w-5 h-5" />
            <span className="hidden md:inline">{t("library.translation.platform")}</span>
          </Button>
          <Button
            variant="ghost"
            onClick={() => { setBatchScope("platform"); setIsBatchDialogOpen(true); }}
            disabled={isBatchScraping || romsOfSystem.length === 0}
          >
            <Database className="w-5 h-5" />
            <span className="hidden md:inline">{t("library.batch.scrapePlatform")}</span>
          </Button>

          {/* PS3 Boxart Generation Button */}
          {isPs3 && (
            <Button
              onClick={handleBatchGenerateBoxart}
              disabled={romsOfSystem.length === 0}
              loading={isGeneratingBoxart}
              title={t("library.boxart.tooltip")}
            >
              <Wand2 className="w-5 h-5" />
              <span className="hidden md:inline">
                {isGeneratingBoxart
                  ? t("library.boxart.generating", {
                      current: boxartProgress.current,
                      total: boxartProgress.total,
                    })
                  : t("library.boxart.generate")}
              </span>
            </Button>
          )}

          {/* 系统内搜索 */}
          <div className="relative w-full md:w-72">
            <Search className="w-4 h-4 text-text-muted absolute left-3 top-1/2 -translate-y-1/2 pointer-events-none" />
            <Input
              type="text"
              placeholder={t("library.search")}
              value={searchQuery}
              onChange={(e) => setSearchQuery(e.target.value)}
              className="pl-9"
            />
          </div>

          {/* 卡片尺寸滑杆(列表模式下隐藏) */}
          {viewMode !== "list" && (
            <label className="flex items-center gap-2 text-xs text-text-secondary">
              <span className="hidden lg:inline">{t("library.cardSize")}</span>
              <input
                type="range"
                min={0.7}
                max={1.6}
                step={0.15}
                value={cardScale}
                onChange={(e) => setCardScale(Number(e.target.value))}
                aria-label={t("library.cardSize")}
                className="w-24 accent-accent-primary"
              />
            </label>
          )}

          {/* View Toggle */}
          <div className="flex items-center gap-1 p-1 bg-bg-secondary rounded-[var(--radius-md)] border-[length:var(--border-width)] border-border-default">
            {VIEW_MODES.map(({ mode, icon: Icon, labelKey }) => (
              <IconButton
                key={mode}
                size="sm"
                variant={viewMode === mode ? "primary" : "ghost"}
                onClick={() => setViewMode(mode)}
                title={t(labelKey)}
                aria-label={t(labelKey)}
              >
                <Icon className="w-5 h-5" />
              </IconButton>
            ))}
          </div>
        </div>
      </div>

      {/* Content Area */}
      <div className="flex-1 min-h-0">
        {filteredRoms.length === 0 ? (
          <div className="flex h-full items-center justify-center">
            <EmptyState
              className="w-full max-w-md"
              icon={<SearchX />}
              title={t("library.noResults")}
              description={t("library.noResultsHint")}
            />
          </div>
        ) : (
          <RomView
            roms={filteredRoms}
            viewMode={viewMode}
            cardScale={cardScale}
            selectedIds={selectedRomIds}
            onRomClick={setActiveRom}
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
        scope={batchScope}
      />

      <Dialog
        open={isTranslateDialogOpen}
        onClose={closeTranslationDialog}
        title={t("library.translation.dialogTitle")}
        footer={
          <>
            <Button variant="ghost" onClick={closeTranslationDialog}>
              {isTranslating ? t("library.translation.stop") : t("common.cancel")}
            </Button>
            <Button onClick={handleBatchTranslate} loading={isTranslating}>
              {isTranslating
                ? t("library.translation.progress", translateProgress)
                : t("library.translation.confirm", { count: translationRoms.length })}
            </Button>
          </>
        }
      >
        <p className="text-sm leading-relaxed text-text-secondary">
          {t("library.translation.warning", { count: translationRoms.length })}
        </p>
      </Dialog>
    </div>
  );
}
