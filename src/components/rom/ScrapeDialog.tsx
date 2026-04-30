import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { X, Search, Image as ImageIcon, Check, Loader2, Gamepad2, Play, Activity, Globe, Info, Star, Calendar, User, LayoutGrid } from "lucide-react";
import { useTranslation } from "react-i18next";
import { scraperApi } from "@/lib/api";
import { useScraperStore } from "@/stores/scraperStore";
import type { Rom, ScraperSearchResult, ScraperGameMetadata, ScraperMediaAsset } from "@/types";
import { clsx } from "clsx";

interface ScrapeDialogProps {
  rom: Rom;
  isOpen: boolean;
  onClose: () => void;
}

export default function ScrapeDialog({ rom, isOpen, onClose }: ScrapeDialogProps) {
  const { t } = useTranslation();
  const { providers, fetchProviders } = useScraperStore();
  const [query, setQuery] = useState(rom.name);
  const [selectedProviderId, setSelectedProviderId] = useState<string>("");
  const [results, setResults] = useState<ScraperSearchResult[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [selectedResult, setSelectedResult] = useState<ScraperSearchResult | null>(null);
  const [metadata, setMetadata] = useState<ScraperGameMetadata | null>(null);
  const [media, setMedia] = useState<ScraperMediaAsset[]>([]);
  const [selectedMediaUrls, setSelectedMediaUrls] = useState<Set<string>>(new Set());
  const [isLoadingDetails, setIsLoadingDetails] = useState(false);
  const [isApplying, setIsApplying] = useState(false);

  useEffect(() => {
    fetchProviders();
  }, [fetchProviders]);

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
      const searchResults = await scraperApi.search(query, rom.file, rom.system);
      setResults(searchResults.sort((a, b) => b.confidence - a.confidence));
      setSelectedResult(null);
      setMetadata(null);
      setMedia([]);
      setSelectedMediaUrls(new Set());
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
        scraperApi.getMedia(result.provider, result.source_id)
      ]);
      setMetadata(metaResult);
      setMedia(mediaResults);
      
      const defaultSelection = new Set<string>();
      mediaResults.forEach(m => {
        if (m.asset_type === "boxfront" || m.asset_type === "box-2D" || m.asset_type === "box-2d") {
          defaultSelection.add(m.url);
        }
      });
      setSelectedMediaUrls(defaultSelection);
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
        selected_media: selectedMedia
      });
      
      onClose();
    } catch (error) {
      console.error("Apply failed:", error);
    } finally {
      setIsApplying(false);
    }
  };

  if (!isOpen) return null;

  const activeProviders = providers.filter(p => p.enabled);

  return (
    <AnimatePresence>
      <div className="fixed inset-0 z-[60] flex items-center justify-center p-4">
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onClick={onClose}
          className="absolute inset-0 bg-black/85 backdrop-blur-md"
        />

        <motion.div
          initial={{ scale: 0.95, opacity: 0, y: 20 }}
          animate={{ scale: 1, opacity: 1, y: 0 }}
          exit={{ scale: 0.95, opacity: 0, y: 20 }}
          className="relative w-full max-w-6xl bg-bg-primary border border-border-default rounded-[2rem] shadow-2xl overflow-hidden flex flex-col max-h-[90vh]"
        >
          {/* Header */}
          <div className="px-8 py-6 border-b border-border-default flex items-center justify-between bg-bg-secondary/30">
            <div className="flex items-center gap-4">
              <div className="w-12 h-12 rounded-2xl bg-accent-primary/10 flex items-center justify-center border border-accent-primary/20">
                <Search className="w-6 h-6 text-accent-primary" />
              </div>
              <div>
                <h2 className="text-xl font-bold text-text-primary tracking-tight">{t("scraper.dialog.title")}</h2>
                <div className="flex items-center gap-2 mt-0.5">
                  <span className="text-xs font-bold text-accent-primary uppercase tracking-tighter bg-accent-primary/10 px-1.5 py-0.5 rounded">
                    {rom.system?.toUpperCase() || t("common.notAvailable")}
                  </span>
                  <span className="text-xs text-text-muted truncate max-w-md">{rom.file}</span>
                </div>
              </div>
            </div>
            <button onClick={onClose} className="p-2.5 rounded-xl hover:bg-bg-tertiary text-text-secondary transition-colors">
              <X className="w-6 h-6" />
            </button>
          </div>

          <div className="flex-1 flex overflow-hidden">
            {/* Left: Search & Results */}
            <div className="w-[380px] border-r border-border-default p-6 flex flex-col gap-6 bg-bg-secondary/10 overflow-y-auto">
              <div className="space-y-4">
                <div className="relative group">
                  <div className="absolute inset-y-0 left-3 flex items-center pointer-events-none text-text-muted group-focus-within:text-accent-primary transition-colors">
                    <Search className="w-4 h-4" />
                  </div>
                  <input
                    type="text"
                    value={query}
                    onChange={(e) => setQuery(e.target.value)}
                    onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
                    placeholder={t("scraper.dialog.searchPlaceholder")}
                    className="w-full bg-bg-primary border border-border-default rounded-xl pl-10 pr-12 py-3 text-sm text-text-primary focus:outline-none focus:border-accent-primary focus:ring-1 focus:ring-accent-primary/30 transition-all shadow-inner"
                  />
                  <div className="absolute inset-y-1.5 right-1.5">
                    <button
                      onClick={handleSearch}
                      disabled={isSearching}
                      className="h-full px-3 bg-accent-primary rounded-lg text-bg-primary font-bold text-xs disabled:opacity-50 transition-opacity flex items-center gap-1.5"
                    >
                      {isSearching ? <Loader2 className="w-3.5 h-3.5 animate-spin" /> : t("scraper.dialog.searchButton")}
                    </button>
                  </div>
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
                  <div className="flex flex-col items-center justify-center py-16 text-text-muted text-center border-2 border-dashed border-border-default rounded-3xl opacity-50">
                    <Gamepad2 className="w-12 h-12 mb-3 opacity-20" />
                    <p className="text-sm">{t("scraper.dialog.noResults")}</p>
                  </div>
                ) : (
                  results.map((res) => (
                    <button
                      key={`${res.provider}-${res.source_id}`}
                      onClick={() => handleSelectResult(res)}
                      className={clsx(
                        "w-full text-left p-4 rounded-[1.25rem] border transition-all duration-300 group relative overflow-hidden",
                        selectedResult?.source_id === res.source_id && selectedResult?.provider === res.provider
                          ? "bg-accent-primary/10 border-accent-primary shadow-lg shadow-accent-primary/5"
                          : "bg-bg-secondary/50 border-transparent hover:bg-bg-tertiary hover:border-border-hover"
                      )}
                    >
                      <div className="flex items-start gap-3">
                        {res.thumbnail ? (
                          <img src={res.thumbnail} className="w-12 h-16 object-cover rounded-lg shadow-sm" alt="" />
                        ) : (
                          <div className="w-12 h-16 bg-bg-tertiary rounded-lg flex items-center justify-center border border-border-default">
                            <ImageIcon className="w-6 h-6 opacity-20" />
                          </div>
                        )}
                        <div className="flex-1 min-w-0">
                          <div className="font-bold text-sm text-text-primary truncate group-hover:text-accent-primary transition-colors">
                            {res.name}
                          </div>
                          <div className="flex items-center gap-2 mt-1.5">
                            <span className="text-[10px] font-bold uppercase tracking-tighter text-text-muted bg-bg-tertiary px-1.5 py-0.5 rounded border border-border-default flex items-center gap-1">
                              {res.provider === "screenscraper" ? <Globe className="w-2.5 h-2.5" /> : <Activity className="w-2.5 h-2.5" />}
                              {res.provider}
                            </span>
                            {res.year && <span className="text-[10px] text-text-muted font-bold">{res.year}</span>}
                          </div>
                        </div>
                        <div className="text-[10px] font-black text-accent-primary bg-accent-primary/10 rounded-full px-2 py-0.5">
                          {Math.round(res.confidence * 100)}%
                        </div>
                      </div>
                    </button>
                  ))
                )}
              </div>
            </div>

            {/* Right: Details & Media */}
            <div className="flex-1 p-8 flex flex-col gap-8 overflow-y-auto bg-bg-primary relative custom-scrollbar">
              {isLoadingDetails ? (
                <div className="absolute inset-0 z-20 flex flex-col items-center justify-center bg-bg-primary/60 backdrop-blur-sm text-accent-primary">
                  <Loader2 className="w-12 h-12 animate-spin mb-4" />
                  <p className="font-bold tracking-widest text-sm uppercase">{t("scraper.dialog.loadingDetails")}</p>
                </div>
              ) : null}

              {selectedResult ? (
                <>
                  {/* Game Details Card */}
                  <div className="flex gap-8 animate-in fade-in slide-in-from-right-4 duration-500">
                    <div className="w-48 shrink-0">
                      {media.find(m => m.asset_type === "boxfront" || m.asset_type === "box-2D") ? (
                        <img 
                          src={media.find(m => m.asset_type === "boxfront" || m.asset_type === "box-2D")?.url} 
                          className="w-full aspect-[3/4] object-cover rounded-2xl shadow-2xl border-2 border-border-default"
                          alt="Cover"
                        />
                      ) : (
                        <div className="w-full aspect-[3/4] bg-bg-secondary rounded-2xl border-2 border-dashed border-border-default flex items-center justify-center">
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
                              <span key={g} className="px-2 py-0.5 rounded-full bg-bg-tertiary border border-border-default text-[10px] font-bold text-text-secondary">
                                {g}
                              </span>
                            ))}
                          </div>
                        </div>
                        {metadata?.rating && (
                          <div className="flex flex-col items-end">
                            <div className="flex items-center gap-1 text-yellow-400">
                              <Star className="w-5 h-5 fill-current" />
                              <span className="text-xl font-black">{metadata.rating.toFixed(1)}</span>
                            </div>
                            <span className="text-[10px] text-text-muted font-bold uppercase tracking-tighter">{t("scraper.dialog.userRating")}</span>
                          </div>
                        )}
                      </div>

                      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4 p-4 bg-bg-secondary/50 rounded-2xl border border-border-default">
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
                            <div className="text-xs font-bold text-text-primary uppercase">{selectedResult.system || "GENERIC"}</div>
                          </div>
                        </div>
                      </div>

                      <p className="text-sm text-text-secondary leading-relaxed line-clamp-3">
                        {metadata?.description || t("scraper.dialog.noDescription")}
                      </p>
                    </div>
                  </div>

                  {/* Media Grid */}
                  <div className="animate-in fade-in slide-in-from-bottom-4 duration-700 delay-200">
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
                      {media.map((m, idx) => (
                        <div
                          key={`${m.url}-${idx}`}
                          onClick={() => toggleMediaSelection(m.url)}
                          className={clsx(
                            "group relative aspect-[3/4] bg-bg-secondary rounded-[1rem] border overflow-hidden transition-all cursor-pointer shadow-sm",
                            selectedMediaUrls.has(m.url) 
                              ? "border-accent-primary ring-2 ring-accent-primary/30" 
                              : "border-border-default hover:border-border-hover"
                          )}
                        >
                          {m.asset_type === "video" ? (
                            <div className="w-full h-full flex flex-col items-center justify-center bg-black/80">
                              <div className="w-10 h-10 rounded-full bg-accent-primary/20 flex items-center justify-center border border-accent-primary/40 group-hover:scale-110 transition-transform">
                                <Play className="w-5 h-5 text-accent-primary fill-current ml-0.5" />
                              </div>
                              <span className="text-[10px] text-accent-primary font-bold mt-2 tracking-widest uppercase">Video</span>
                            </div>
                          ) : (
                            <img src={m.url} alt="" className="w-full h-full object-cover transition-transform duration-500 group-hover:scale-110" />
                          )}
                          
                          <div className="absolute inset-x-0 bottom-0 bg-gradient-to-t from-black/80 to-transparent p-2.5 translate-y-2 group-hover:translate-y-0 transition-transform">
                            <div className="text-[9px] text-text-primary font-black uppercase bg-accent-primary/80 backdrop-blur-sm px-1.5 py-0.5 rounded-md inline-block">
                              {m.asset_type}
                            </div>
                            {m.width && m.height && (
                              <div className="text-[8px] text-white/50 mt-1 font-bold">{m.width}x{m.height}</div>
                            )}
                          </div>

                          <div className={clsx(
                            "absolute top-2 right-2 w-6 h-6 rounded-full flex items-center justify-center transition-all duration-300 transform",
                            selectedMediaUrls.has(m.url) 
                              ? "bg-accent-primary scale-110 opacity-100 shadow-lg shadow-accent-primary/30" 
                              : "bg-black/40 opacity-0 group-hover:opacity-100 scale-90"
                          )}>
                            <Check className="w-3.5 h-3.5 text-bg-primary stroke-[4px]" />
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                </>
              ) : (
                <div className="flex-1 flex flex-col items-center justify-center text-text-muted opacity-30 select-none">
                  <div className="relative mb-6">
                    <Gamepad2 className="w-32 h-32 animate-pulse" />
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

          {/* Footer */}
          <div className="px-8 py-5 bg-bg-secondary/50 border-t border-border-default flex justify-between items-center">
            <div className="flex items-center gap-3 text-text-muted">
               <Info className="w-4 h-4" />
               <span className="text-xs font-medium">{t("scraper.dialog.applyNote")}</span>
            </div>
            <div className="flex items-center gap-4">
              <button
                onClick={onClose}
                className="px-6 py-2.5 rounded-xl text-text-secondary hover:text-text-primary hover:bg-bg-tertiary transition-all font-bold text-sm"
              >
                {t("common.cancel")}
              </button>
              <button
                disabled={!selectedResult || !metadata || selectedMediaUrls.size === 0 || isApplying}
                onClick={handleApply}
                className="group relative px-10 py-2.5 bg-accent-primary overflow-hidden hover:opacity-90 text-bg-primary rounded-xl font-black transition-all shadow-xl shadow-accent-primary/25 disabled:opacity-30 disabled:cursor-not-allowed disabled:shadow-none text-sm flex items-center gap-2 active:scale-95"
              >
                {isApplying && <Loader2 className="w-4 h-4 animate-spin" />}
                <Check className="w-4 h-4 stroke-[3px]" />
                {t("scraper.dialog.confirmAndApply")}
                <div className="absolute inset-0 bg-white/20 translate-x-full group-hover:-translate-x-full transition-transform duration-700 skew-x-12" />
              </button>
            </div>
          </div>
        </motion.div>
      </div>
    </AnimatePresence>
  );
}
