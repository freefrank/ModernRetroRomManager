import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { X, Search, Image as ImageIcon, Check, Loader2, Gamepad2, Play } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useScraperStore } from "@/stores/scraperStore";
import type { Rom } from "@/types";
import { clsx } from "clsx";

interface ScrapedGame {
  source_id: string;
  name: string;
}

interface ScrapedMedia {
  url: string;
  asset_type: string;
  width?: number;
  height?: number;
}

interface ScrapeDialogProps {
  rom: Rom;
  isOpen: boolean;
  onClose: () => void;
}

export default function ScrapeDialog({ rom, isOpen, onClose }: ScrapeDialogProps) {
  const { configs, fetchConfigs } = useScraperStore();
  const [query, setQuery] = useState(rom.metadata?.name || rom.filename);
  const [provider, setProvider] = useState("steamgriddb");
  const [results, setResults] = useState<ScrapedGame[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [selectedGameId, setSelectedGameId] = useState<string | null>(null);
  const [selectedGame, setSelectedGame] = useState<ScrapedGame | null>(null);
  const [media, setMedia] = useState<ScrapedMedia[]>([]);
  const [selectedMediaUrls, setSelectedMediaUrls] = useState<Set<string>>(new Set());
  const [isLoadingMedia, setIsLoadingMedia] = useState(false);
  const [isApplying, setIsApplying] = useState(false);

  useEffect(() => {
    fetchConfigs();
  }, [fetchConfigs]);

  const handleSearch = async () => {
    if (!query) return;
    setIsSearching(true);
    try {
      const searchResults = await invoke<ScrapedGame[]>("search_game", { query, provider });
      setResults(searchResults);
      setSelectedGameId(null);
      setSelectedGame(null);
      setMedia([]);
      setSelectedMediaUrls(new Set());
    } catch (error) {
      console.error("Search failed:", error);
    } finally {
      setIsSearching(false);
    }
  };

  const handleSelectGame = async (game: ScrapedGame) => {
    setSelectedGameId(game.source_id);
    setSelectedGame(game);
    setIsLoadingMedia(true);
    setSelectedMediaUrls(new Set());
    try {
      const mediaResults = await invoke<ScrapedMedia[]>("get_scraper_game_media", { sourceId: game.source_id, provider });
      setMedia(mediaResults);
    } catch (error) {
      console.error("Failed to load media:", error);
    } finally {
      setIsLoadingMedia(false);
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
    if (!selectedGame || selectedMediaUrls.size === 0) return;
    setIsApplying(true);
    try {
      const selectedMedia = media.filter(m => selectedMediaUrls.has(m.url));
      await invoke("apply_scraped_data", {
        options: {
          romId: rom.id,
          gameDetails: selectedGame,
          selectedMedia
        }
      });
      onClose();
    } catch (error) {
      console.error("Apply failed:", error);
    } finally {
      setIsApplying(false);
    }
  };

  if (!isOpen) return null;

  const activeProviders = Object.entries(configs)
    .filter(([, c]) => c.enabled)
    .map(([provider]) => provider);

  return (
    <AnimatePresence>
      <div className="fixed inset-0 z-[60] flex items-center justify-center p-4">
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          onClick={onClose}
          className="absolute inset-0 bg-black/80 backdrop-blur-sm"
        />

        <motion.div
          initial={{ scale: 0.9, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          exit={{ scale: 0.9, opacity: 0 }}
          className="relative w-full max-w-4xl bg-bg-primary border border-border-default rounded-2xl shadow-2xl overflow-hidden flex flex-col max-h-[90vh]"
        >
          {/* Header */}
          <div className="p-6 border-b border-border-default flex items-center justify-between">
            <div>
              <h2 className="text-xl font-bold text-text-primary">Scrape Game Information</h2>
              <p className="text-sm text-text-secondary mt-1">{rom.filename}</p>
            </div>
            <button onClick={onClose} className="p-2 rounded-lg hover:bg-bg-tertiary text-text-secondary">
              <X className="w-6 h-6" />
            </button>
          </div>

          <div className="flex-1 flex overflow-hidden">
            {/* Left: Search & Results */}
            <div className="w-1/3 border-r border-border-default p-6 flex flex-col gap-6 overflow-y-auto">
              <div className="space-y-4">
                <div>
                  <label className="text-xs font-bold text-text-muted uppercase tracking-wider mb-2 block">Provider</label>
                  <select
                    value={provider}
                    onChange={(e) => setProvider(e.target.value)}
                    className="w-full bg-bg-secondary border border-border-default rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:border-accent-primary"
                  >
                    {activeProviders.length === 0 ? (
                      <option disabled>No providers enabled</option>
                    ) : (
                      activeProviders.map(p => (
                        <option key={p} value={p}>
                          {p.toUpperCase()}
                        </option>
                      ))
                    )}
                  </select>
                </div>

                <div>
                  <label className="text-xs font-bold text-text-muted uppercase tracking-wider mb-2 block">Search Query</label>
                  <div className="flex gap-2">
                    <input
                      type="text"
                      value={query}
                      onChange={(e) => setQuery(e.target.value)}
                      onKeyDown={(e) => e.key === 'Enter' && handleSearch()}
                      className="flex-1 bg-bg-secondary border border-border-default rounded-lg px-3 py-2 text-sm text-text-primary focus:outline-none focus:border-accent-primary"
                    />
                    <button
                      onClick={handleSearch}
                      disabled={isSearching}
                      className="p-2 bg-accent-primary rounded-lg text-text-primary disabled:opacity-50"
                    >
                      {isSearching ? <Loader2 className="w-5 h-5 animate-spin" /> : <Search className="w-5 h-5" />}
                    </button>
                  </div>
                </div>
              </div>

              <div className="flex-1 space-y-2">
                <label className="text-xs font-bold text-text-muted uppercase tracking-wider mb-2 block">Results</label>
                {results.length === 0 ? (
                  <div className="text-center py-8 text-text-muted text-sm border border-dashed border-border-default rounded-xl">
                    No results found
                  </div>
                ) : (
                  results.map((res) => (
                    <button
                      key={res.source_id}
                      onClick={() => handleSelectGame(res)}
                      className={clsx(
                        "w-full text-left p-3 rounded-xl border transition-all",
                        selectedGameId === res.source_id
                          ? "bg-accent-primary/20 border-accent-primary text-text-primary"
                          : "bg-bg-tertiary border-transparent text-text-secondary hover:bg-border-hover hover:text-text-primary"
                      )}
                    >
                      <div className="font-medium text-sm">{res.name}</div>
                      <div className="text-[10px] opacity-50 mt-1">ID: {res.source_id}</div>
                    </button>
                  ))
                )}
              </div>
            </div>

            {/* Right: Media Selection */}
            <div className="flex-1 p-6 flex flex-col gap-6 overflow-y-auto bg-bg-primary">
              <div>
                <h3 className="text-lg font-bold text-text-primary mb-1">Available Media</h3>
                <p className="text-sm text-text-secondary">Select assets to download and apply to your game.</p>
              </div>

              {isLoadingMedia ? (
                <div className="flex-1 flex flex-col items-center justify-center text-text-muted">
                  <Loader2 className="w-12 h-12 animate-spin mb-4" />
                  <p>Loading media from {provider}...</p>
                </div>
              ) : selectedGameId ? (
                media.length === 0 ? (
                  <div className="flex-1 flex flex-col items-center justify-center text-text-muted border border-dashed border-border-default rounded-2xl">
                    <ImageIcon className="w-12 h-12 mb-4 opacity-20" />
                    <p>No media found for this game</p>
                  </div>
                ) : (
                  <div className="grid grid-cols-2 md:grid-cols-3 gap-4">
                    {media.map((m, idx) => (
                      <div
                        key={idx}
                        onClick={() => toggleMediaSelection(m.url)}
                        className={clsx(
                          "group relative aspect-[3/4] bg-bg-secondary rounded-xl border overflow-hidden transition-all cursor-pointer",
                          selectedMediaUrls.has(m.url) ? "border-accent-primary ring-2 ring-accent-primary" : "border-border-default hover:border-accent-primary/50"
                        )}
                      >
                        {m.asset_type === "video" ? (
                          <div className="w-full h-full flex flex-col items-center justify-center bg-black">
                            <Play className="w-8 h-8 text-accent-primary mb-2" />
                            <span className="text-[10px] text-text-muted">Video Preview</span>
                          </div>
                        ) : (
                          <img src={m.url} alt="" className="w-full h-full object-cover" />
                        )}
                        <div className="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-end p-3">
                          <div className="text-[10px] text-text-primary font-bold uppercase bg-accent-primary px-2 py-0.5 rounded">
                            {m.asset_type}
                          </div>
                        </div>
                        <div className={clsx(
                          "absolute top-2 right-2 w-6 h-6 bg-accent-primary rounded-full flex items-center justify-center transition-opacity",
                          selectedMediaUrls.has(m.url) ? "opacity-100" : "opacity-0 group-hover:opacity-50"
                        )}>
                          <Check className="w-4 h-4 text-text-primary" />
                        </div>
                      </div>
                    ))}
                  </div>
                )
              ) : (
                <div className="flex-1 flex flex-col items-center justify-center text-text-muted border border-dashed border-border-default rounded-2xl">
                  <Gamepad2 className="w-16 h-16 mb-4 opacity-10" />
                  <p>Search and select a game on the left to see media</p>
                </div>
              )}
            </div>
          </div>

          {/* Footer */}
          <div className="p-4 bg-bg-secondary border-t border-border-default flex justify-end gap-3">
            <button
              onClick={onClose}
              className="px-6 py-2 rounded-xl text-text-primary hover:bg-bg-tertiary transition-colors font-medium text-sm"
            >
              Cancel
            </button>
            <button
              disabled={!selectedGameId || selectedMediaUrls.size === 0 || isApplying}
              onClick={handleApply}
              className="px-8 py-2 bg-accent-primary hover:bg-accent-primary/90 text-text-primary rounded-xl font-bold transition-colors shadow-lg shadow-accent-primary/20 disabled:opacity-50 disabled:cursor-not-allowed text-sm flex items-center gap-2"
            >
              {isApplying && <Loader2 className="w-4 h-4 animate-spin" />}
              Apply Selected
            </button>
          </div>
        </motion.div>
      </div>
    </AnimatePresence>
  );
}
