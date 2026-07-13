import { useState, useEffect } from "react";
import { Search, Image as ImageIcon, Check, Gamepad2, Play, Activity, Globe, Info, Star, Calendar, User, LayoutGrid } from "lucide-react";
import { useTranslation } from "react-i18next";
import { scraperApi } from "@/lib/api";
import { useScraperStore } from "@/stores/scraperStore";
import { useRomStore } from "@/stores/romStore";
import type { Rom, ScraperSearchResult, ScraperGameMetadata, ScraperMediaAsset } from "@/types";
import { Button, Dialog, Input, Badge, Spinner, EmptyState } from "@/components/ui";
import { clsx } from "clsx";
import MediaTypeSelector from "./MediaTypeSelector";
import { cacheSearchResults, getCachedSearchResults } from "@/lib/scraperSearchCache";
import { normalizeMediaType, selectOneAssetPerType } from "@/lib/mediaSelection";

interface ScrapeDialogProps {
  rom: Rom;
  isOpen: boolean;
  onClose: () => void;
}

function confidenceVariant(confidence: number): "success" | "warning" | "error" {
  if (confidence >= 0.9) return "success";
  if (confidence >= 0.6) return "warning";
  return "error";
}

export default function ScrapeDialog({ rom, isOpen, onClose }: ScrapeDialogProps) {
  const { t } = useTranslation();
  const { providers, fetchProviders } = useScraperStore();
  const fetchRoms = useRomStore(state => state.fetchRoms);
  const [query, setQuery] = useState(rom.english_name?.trim() || rom.name);
  const [selectedProviderId, setSelectedProviderId] = useState<string>("");
  const [results, setResults] = useState<ScraperSearchResult[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [selectedResult, setSelectedResult] = useState<ScraperSearchResult | null>(null);
  const [metadata, setMetadata] = useState<ScraperGameMetadata | null>(null);
  const [media, setMedia] = useState<ScraperMediaAsset[]>([]);
  const [selectedMediaUrls, setSelectedMediaUrls] = useState<Set<string>>(new Set());
  const [isLoadingDetails, setIsLoadingDetails] = useState(false);
  const [isApplying, setIsApplying] = useState(false);
  const [mediaTypes, setMediaTypes] = useState<string[]>([]);

  const clearSelectedDetails = () => {
    setSelectedResult(null);
    setMetadata(null);
    setMedia([]);
    setSelectedMediaUrls(new Set());
  };

  const restoreCachedResults = (searchQuery: string) => {
    const cached = getCachedSearchResults({
      query: searchQuery,
      fileName: rom.file,
      system: rom.system,
      directory: rom.directory,
    });
    setResults(cached || []);
    clearSelectedDetails();
  };

  useEffect(() => {
    fetchProviders();
    scraperApi.getMediaTypes().then(setMediaTypes).catch(console.error);
  }, [fetchProviders]);

  useEffect(() => {
    if (isOpen) {
      const initialQuery = rom.english_name?.trim() || rom.name;
      setQuery(initialQuery);
      restoreCachedResults(initialQuery);
    }
    // 仅在打开弹窗或切换 ROM 时恢复缓存；详情状态由弹窗内部交互管理。
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isOpen, rom.english_name, rom.name]);

  useEffect(() => {
    if (providers.length > 0 && !selectedProviderId) {
      const firstEnabled = providers.find(p => p.enabled);
      if (firstEnabled) setSelectedProviderId(firstEnabled.id);
    }
  }, [providers, selectedProviderId]);

  const handleSearch = async () => {
    if (!query) return;
    setIsSearching(true);
    try {
      const searchResults = await scraperApi.search(query, rom.file, rom.system, rom.directory);
      const sortedResults = searchResults.sort((a, b) => b.confidence - a.confidence);
      setResults(sortedResults);
      cacheSearchResults({
        query,
        fileName: rom.file,
        system: rom.system,
        directory: rom.directory,
      }, sortedResults);
      clearSelectedDetails();
      await fetchProviders();
    } catch (error) {
      console.error("Search failed:", error);
    } finally {
      setIsSearching(false);
    }
  };

  const handleSelectResult = async (result: ScraperSearchResult) => {
    setSelectedResult(result);
    setIsLoadingDetails(true);
    setSelectedMediaUrls(new Set());
    try {
      const [metaResult, mediaResults] = await Promise.all([
        scraperApi.getMetadata(result.provider, result.source_id),
        scraperApi.getMedia(result.provider, result.source_id, {
          romDirectory: rom.directory,
          system: rom.system,
          romId: rom.file,
          mediaTypes,
        })
      ]);
      setMetadata(metaResult);
      setMedia(mediaResults);

      setSelectedMediaUrls(selectOneAssetPerType(mediaResults));
    } catch (error) {
      console.error("Failed to load details:", error);
    } finally {
      setIsLoadingDetails(false);
    }
  };

  const toggleMediaSelection = (url: string) => {
    const next = new Set(selectedMediaUrls);
    if (next.has(url)) {
      next.delete(url);
    } else {
      next.add(url);
    }
    setSelectedMediaUrls(next);
  };

  const handleApply = async () => {
    if (!selectedResult || !metadata) return;
    setIsApplying(true);
    try {
      const selectedMedia = media.filter(m => selectedMediaUrls.has(m.url));

      await scraperApi.applyScrapedData({
        rom_id: rom.file,
        directory: rom.directory,
        system: rom.system,
        metadata,
        selected_media: selectedMedia,
        provider_id: selectedResult.provider,
      });

      await fetchRoms();
      await fetchProviders();
      onClose();
    } catch (error) {
      console.error("Apply failed:", error);
    } finally {
      setIsApplying(false);
    }
  };

  const activeProviders = providers.filter(p => p.enabled);

  return (
    <Dialog
      open={isOpen}
      onClose={onClose}
      style={{ maxWidth: "72rem" }}
      title={
        <span className="flex items-center gap-3">
          <span className="w-10 h-10 rounded-[var(--radius-md)] bg-accent-primary/10 flex items-center justify-center border-[length:var(--border-width)] border-accent-primary/20 shrink-0">
            <Search className="w-5 h-5 text-accent-primary" />
          </span>
          <span className="min-w-0">
            <span className="block text-lg font-bold text-text-primary tracking-tight">{t("scraper.dialog.title")}</span>
            <span className="flex items-center gap-2 mt-0.5">
              <Badge variant="info" className="uppercase">
                {rom.system?.toUpperCase() || t("common.notAvailable")}
              </Badge>
              <span className="text-xs font-normal text-text-muted truncate max-w-md">{rom.file}</span>
            </span>
          </span>
        </span>
      }
      footer={
        <div className="flex flex-1 justify-between items-center gap-4">
          <div className="flex items-center gap-2 text-text-muted min-w-0">
            <Info className="w-4 h-4 shrink-0" />
            <span className="text-xs font-medium truncate">{t("scraper.dialog.applyNote")}</span>
          </div>
          <div className="flex items-center gap-3 shrink-0">
            <Button variant="ghost" onClick={onClose}>
              {t("common.cancel")}
            </Button>
            <Button
              disabled={!selectedResult || !metadata || selectedMediaUrls.size === 0}
              loading={isApplying}
              onClick={handleApply}
            >
              <Check className="w-4 h-4" />
              {t("scraper.dialog.confirmAndApply")}
            </Button>
          </div>
        </div>
      }
    >
      <div className="flex overflow-hidden h-[62vh] -mx-5 -my-3">
        {/* Left: Search & Results */}
        <div className="w-[340px] shrink-0 border-r-[length:var(--border-width)] border-border-default p-5 flex flex-col gap-5 bg-bg-secondary/10 overflow-y-auto custom-scrollbar">
          <div className="space-y-3">
            <MediaTypeSelector value={mediaTypes} onChange={setMediaTypes} />
            <div className="flex items-center gap-2">
              <Input
                type="text"
                value={query}
                onChange={(e) => {
                  setQuery(e.target.value);
                  restoreCachedResults(e.target.value);
                }}
                onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
                placeholder={t("scraper.dialog.searchPlaceholder")}
              />
              <Button size="sm" onClick={handleSearch} loading={isSearching} className="shrink-0">
                {t("scraper.dialog.searchButton")}
              </Button>
            </div>

            <div className="flex items-center justify-between text-[10px] font-bold text-text-muted uppercase tracking-widest px-1">
              <span>{t("scraper.dialog.resultsCount", { count: results.length })}</span>
              {activeProviders.length > 0 && (
                <span className="text-accent-primary">
                  {t("scraper.apiConfig.activeSources", { count: activeProviders.length })}
                </span>
              )}
            </div>
          </div>

          <div className="flex-1 space-y-2.5 pr-1 overflow-y-auto custom-scrollbar">
            {results.length === 0 ? (
              <EmptyState icon={<Gamepad2 />} title={t("scraper.dialog.noResults")} />
            ) : (
              results.map((res) => {
                const isSelected =
                  selectedResult?.source_id === res.source_id && selectedResult?.provider === res.provider;
                return (
                  <button
                    key={`${res.provider}-${res.source_id}`}
                    type="button"
                    onClick={() => handleSelectResult(res)}
                    aria-pressed={isSelected}
                    className={clsx(
                      "rr-card w-full text-left p-3 rounded-[var(--radius-lg)] border-[length:var(--border-width)] group",
                      "transition-all duration-[var(--motion-fast)] ease-[var(--motion-easing)]",
                      "focus-visible:outline-none focus-visible:border-border-highlight",
                      isSelected
                        ? "bg-accent-primary/10 border-border-highlight"
                        : "bg-bg-secondary/50 border-border-default hover:bg-bg-tertiary hover:border-border-hover"
                    )}
                  >
                    <div className="flex items-start gap-3">
                      {res.thumbnail ? (
                        <img src={res.thumbnail} className="w-12 h-16 object-cover rounded-[var(--radius-sm)]" alt="" />
                      ) : (
                        <div className="w-12 h-16 bg-bg-tertiary rounded-[var(--radius-sm)] flex items-center justify-center border-[length:var(--border-width)] border-border-default">
                          <ImageIcon className="w-6 h-6 opacity-20" />
                        </div>
                      )}
                      <div className="flex-1 min-w-0">
                        <div className="font-bold text-sm text-text-primary truncate group-hover:text-accent-primary transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)]">
                          {res.name}
                        </div>
                        <div className="flex items-center gap-2 mt-1.5">
                          <Badge className="uppercase">
                            {res.provider === "screenscraper" ? <Globe className="w-2.5 h-2.5" /> : <Activity className="w-2.5 h-2.5" />}
                            {res.provider}
                          </Badge>
                          {res.year && <span className="text-[10px] text-text-muted font-bold">{res.year}</span>}
                          {typeof res.asset_count === "number" && (
                            <span className="text-[10px] text-text-muted font-bold">
                              {t("scraper.dialog.assetCount", { count: res.asset_count })}
                            </span>
                          )}
                        </div>
                      </div>
                      <Badge variant={confidenceVariant(res.confidence)}>
                        {Math.round(res.confidence * 100)}%
                      </Badge>
                    </div>
                  </button>
                );
              })
            )}
          </div>
        </div>

        {/* Right: Details & Media */}
        <div className="flex-1 p-6 flex flex-col gap-8 overflow-y-auto bg-bg-primary relative custom-scrollbar">
          {isLoadingDetails ? (
            <div className="absolute inset-0 z-20 flex flex-col items-center justify-center bg-bg-primary/60 backdrop-blur-sm text-accent-primary">
              <Spinner size={48} className="mb-4" />
              <p className="font-bold tracking-widest text-sm uppercase">{t("scraper.dialog.loadingDetails")}</p>
            </div>
          ) : null}

          {selectedResult ? (
            <>
              {/* Game Details */}
              <div className="flex gap-8">
                <div className="w-48 shrink-0">
                  {media.find(m => normalizeMediaType(m.asset_type) === "boxfront") ? (
                    <img
                      src={media.find(m => normalizeMediaType(m.asset_type) === "boxfront")?.url}
                      className="w-full aspect-[3/4] object-cover rounded-[var(--radius-lg)] border-[length:var(--border-width)] border-border-default [box-shadow:var(--shadow-card)]"
                      alt={t("scrapeDialog.coverAlt")}
                    />
                  ) : (
                    <div className="w-full aspect-[3/4] bg-bg-secondary rounded-[var(--radius-lg)] border-[length:var(--border-width)] border-dashed border-border-default flex items-center justify-center">
                      <ImageIcon className="w-12 h-12 opacity-10" />
                    </div>
                  )}
                </div>

                <div className="flex-1 space-y-4">
                  <div className="flex items-start justify-between">
                    <div>
                      <h3 className="text-3xl font-black text-text-primary tracking-tight leading-none mb-2">
                        {metadata?.name || selectedResult.name}
                      </h3>
                      <div className="flex flex-wrap gap-2">
                        {metadata?.genres.map(g => (
                          <Badge key={g}>{g}</Badge>
                        ))}
                      </div>
                    </div>
                    {metadata?.rating && (
                      <div className="flex flex-col items-end">
                        <div className="flex items-center gap-1 text-accent-warning">
                          <Star className="w-5 h-5 fill-current" />
                          <span className="text-xl font-black">{metadata.rating.toFixed(1)}</span>
                        </div>
                        <span className="text-[10px] text-text-muted font-bold uppercase tracking-tighter">{t("scraper.dialog.userRating")}</span>
                      </div>
                    )}
                  </div>

                  <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 p-4 bg-bg-secondary/50 rounded-[var(--radius-lg)] border-[length:var(--border-width)] border-border-default">
                    <div className="flex items-center gap-2.5">
                      <Calendar className="w-4 h-4 text-accent-primary" />
                      <div>
                        <div className="text-[9px] text-text-muted uppercase font-bold tracking-widest">{t("scraper.dialog.releaseDate")}</div>
                        <div className="text-xs font-bold text-text-primary">{metadata?.release_date || t("common.notAvailable")}</div>
                      </div>
                    </div>
                    <div className="flex items-center gap-2.5">
                      <User className="w-4 h-4 text-accent-primary" />
                      <div>
                        <div className="text-[9px] text-text-muted uppercase font-bold tracking-widest">{t("scraper.dialog.developer")}</div>
                        <div className="text-xs font-bold text-text-primary truncate">{metadata?.developer || t("common.notAvailable")}</div>
                      </div>
                    </div>
                    <div className="flex items-center gap-2.5">
                      <Activity className="w-4 h-4 text-accent-primary" />
                      <div>
                        <div className="text-[9px] text-text-muted uppercase font-bold tracking-widest">{t("scraper.dialog.publisher")}</div>
                        <div className="text-xs font-bold text-text-primary truncate">{metadata?.publisher || t("common.notAvailable")}</div>
                      </div>
                    </div>
                    <div className="flex items-center gap-2.5">
                      <LayoutGrid className="w-4 h-4 text-accent-primary" />
                      <div>
                        <div className="text-[9px] text-text-muted uppercase font-bold tracking-widest">{t("scraper.dialog.sourceSystem")}</div>
                        <div className="text-xs font-bold text-text-primary uppercase">{selectedResult.system || t("scrapeDialog.genericSystem")}</div>
                      </div>
                    </div>
                  </div>

                  <p className="text-sm text-text-secondary leading-relaxed line-clamp-3">
                    {metadata?.description || t("scraper.dialog.noDescription")}
                  </p>
                </div>
              </div>

              {/* Media Grid */}
              <div>
                <div className="flex items-center justify-between mb-4">
                  <h4 className="text-sm font-black text-text-primary uppercase tracking-widest flex items-center gap-2">
                    <ImageIcon className="w-4 h-4 text-accent-primary" />
                    {t("scraper.dialog.mediaAssets", { count: media.length })}
                  </h4>
                  <div className="text-[10px] font-bold text-text-muted">
                    {t("scraper.dialog.selectedAssets", { count: selectedMediaUrls.size })}
                  </div>
                </div>

                <div className="grid grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
                  {media.map((m, idx) => {
                    const isChecked = selectedMediaUrls.has(m.url);
                    return (
                      <button
                        key={`${m.url}-${idx}`}
                        type="button"
                        onClick={() => toggleMediaSelection(m.url)}
                        aria-pressed={isChecked}
                        className={clsx(
                          "group relative aspect-[3/4] bg-bg-secondary rounded-[var(--radius-md)] border-[length:var(--border-width)] overflow-hidden cursor-pointer",
                          "transition-all duration-[var(--motion-fast)] ease-[var(--motion-easing)]",
                          "focus-visible:outline-none",
                          isChecked
                            ? "border-border-highlight"
                            : "border-border-default hover:border-border-hover"
                        )}
                      >
                        {m.asset_type === "video" ? (
                          <div className="w-full h-full flex flex-col items-center justify-center bg-bg-primary/80">
                            <div className="w-10 h-10 rounded-full bg-accent-primary/20 flex items-center justify-center border-[length:var(--border-width)] border-accent-primary/40 group-hover:scale-110 transition-transform duration-[var(--motion-fast)] ease-[var(--motion-easing)]">
                              <Play className="w-5 h-5 text-accent-primary fill-current ml-0.5" />
                            </div>
                            <span className="text-[10px] text-accent-primary font-bold mt-2 tracking-widest uppercase">{t("scrapeDialog.videoAsset")}</span>
                          </div>
                        ) : (
                          <img src={m.url} alt="" className="w-full h-full object-cover transition-transform duration-[var(--motion-normal)] ease-[var(--motion-easing)] group-hover:scale-110" />
                        )}

                        <div className="absolute inset-x-0 bottom-0 bg-gradient-to-t from-bg-primary/80 to-transparent p-2.5 translate-y-2 group-hover:translate-y-0 transition-transform duration-[var(--motion-fast)] ease-[var(--motion-easing)] text-left">
                          <Badge variant="info" className="uppercase">
                            {m.asset_type}
                          </Badge>
                          {m.width && m.height && (
                            <div className="text-[8px] text-text-muted mt-1 font-bold">{m.width}x{m.height}</div>
                          )}
                        </div>

                        <div className={clsx(
                          "absolute top-2 right-2 w-6 h-6 rounded-full flex items-center justify-center",
                          "transition-all duration-[var(--motion-fast)] ease-[var(--motion-easing)] transform",
                          isChecked
                            ? "bg-accent-primary scale-110 opacity-100"
                            : "bg-bg-primary/60 opacity-0 group-hover:opacity-100 scale-90"
                        )}>
                          <Check className="w-3.5 h-3.5 text-bg-primary stroke-[4px]" />
                        </div>
                      </button>
                    );
                  })}
                </div>
              </div>
            </>
          ) : (
            <div className="flex-1 flex flex-col items-center justify-center text-text-muted opacity-40 select-none">
              <div className="relative mb-6">
                <Gamepad2 className="w-32 h-32" />
                <Search className="w-12 h-12 absolute -bottom-2 -right-2 text-accent-primary" />
              </div>
              <h3 className="text-2xl font-black uppercase tracking-tighter mb-2">{t("scraper.dialog.readyToScrape")}</h3>
              <p className="max-w-xs text-center text-sm font-medium leading-relaxed">
                {t("scraper.dialog.readyToScrapeDesc")}
              </p>
            </div>
          )}
        </div>
      </div>
    </Dialog>
  );
}
