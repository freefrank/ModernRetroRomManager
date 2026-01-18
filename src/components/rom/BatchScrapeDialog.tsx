import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { X, Database, Globe, Activity, Loader2, PlayCircle, CheckCircle2, AlertCircle } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useScraperStore } from "@/stores/scraperStore";
import { useRomStore } from "@/stores/romStore";
import { clsx } from "clsx";

interface BatchScrapeDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function BatchScrapeDialog({ isOpen, onClose }: BatchScrapeDialogProps) {
  const { t } = useTranslation();
  const { providers, fetchProviders } = useScraperStore();
  const { startBatchScrape, selectedRomIds, isBatchScraping, batchProgress } = useRomStore();
  const [selectedProviderId, setSelectedProviderId] = useState<string>("");

  useEffect(() => {
    fetchProviders();
  }, [fetchProviders]);

  useEffect(() => {
    if (providers.length > 0 && !selectedProviderId) {
      const firstEnabled = providers.find(p => p.enabled);
      if (firstEnabled) setSelectedProviderId(firstEnabled.id);
    }
  }, [providers, selectedProviderId]);

  const handleStart = async () => {
    if (!selectedProviderId) return;
    await startBatchScrape(selectedProviderId);
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
          onClick={isBatchScraping ? undefined : onClose}
          className="absolute inset-0 bg-black/80 backdrop-blur-sm"
        />

        <motion.div
          initial={{ scale: 0.9, opacity: 0, y: 20 }}
          animate={{ scale: 1, opacity: 1, y: 0 }}
          exit={{ scale: 0.9, opacity: 0, y: 20 }}
          className="relative w-full max-w-lg bg-bg-primary border border-border-default rounded-[2rem] shadow-2xl overflow-hidden"
        >
          {/* Header */}
          <div className="p-6 border-b border-border-default flex items-center justify-between bg-bg-secondary/30">
            <div className="flex items-center gap-3">
              <Database className="w-6 h-6 text-accent-primary" />
              <h2 className="text-xl font-bold text-text-primary tracking-tight">{t("scraper.batch.title")}</h2>
            </div>
            {!isBatchScraping && (
              <button onClick={onClose} className="p-2 rounded-xl hover:bg-bg-tertiary text-text-secondary transition-colors">
                <X className="w-6 h-6" />
              </button>
            )}
          </div>

          <div className="p-8 space-y-8">
            {isBatchScraping ? (
              /* 进度展示 */
              <div className="space-y-6 py-4">
                <div className="flex items-center justify-between mb-2">
                  <div className="flex items-center gap-2 text-accent-primary">
                    <Loader2 className="w-5 h-5 animate-spin" />
                    <span className="font-bold text-sm uppercase tracking-widest">{t("scraper.batch.scraping")}</span>
                  </div>
                  <span className="text-xl font-black text-text-primary">
                    {batchProgress ? Math.round((batchProgress.current / batchProgress.total) * 100) : 0}%
                  </span>
                </div>

                {/* 进度条 */}
                <div className="h-4 bg-bg-tertiary rounded-full overflow-hidden border border-border-default shadow-inner">
                  <motion.div 
                    initial={{ width: 0 }}
                    animate={{ width: `${batchProgress ? (batchProgress.current / batchProgress.total) * 100 : 0}%` }}
                    className="h-full bg-gradient-to-r from-accent-primary to-accent-primary/60"
                  />
                </div>

                <div className="flex flex-col items-center gap-2 p-4 bg-bg-secondary rounded-2xl border border-border-default">
                  <div className="text-sm font-medium text-text-primary">
                    {batchProgress?.message || t("scraper.batch.preparing")}
                  </div>
                  <div className="text-xs text-text-muted">
                    {t("scraper.batch.processed", { current: batchProgress?.current || 0, total: batchProgress?.total || 0 })}
                  </div>
                </div>

                {batchProgress?.finished && (
                  <motion.div 
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    className="flex items-center gap-3 p-4 bg-green-500/10 border border-green-500/20 rounded-2xl text-green-400"
                  >
                    <CheckCircle2 className="w-6 h-6 shrink-0" />
                    <div>
                      <div className="font-bold">{t("scraper.batch.completed")}</div>
                      <div className="text-xs opacity-80">{t("scraper.batch.completedDesc")}</div>
                    </div>
                  </motion.div>
                )}
              </div>
            ) : (
              /* 配置界面 */
              <>
                <div className="flex items-start gap-4 p-4 bg-bg-secondary rounded-2xl border border-border-default border-l-4 border-l-accent-primary shadow-sm">
                  <PlayCircle className="w-10 h-10 text-accent-primary shrink-0 opacity-80" />
                  <div>
                    <p className="text-text-primary font-bold text-lg leading-tight">
                      {t("scraper.batch.pendingTitle", { count: selectedRomIds.size })}
                    </p>
                    <p className="text-sm text-text-muted mt-1 leading-relaxed">
                      {t("scraper.batch.pendingDesc")}
                    </p>
                  </div>
                </div>

                <div className="space-y-4">
                  <div className="flex items-center justify-between px-1">
                    <label className="text-xs font-black text-text-muted uppercase tracking-widest">{t("scraper.batch.selectSource")}</label>
                    <span className="text-[10px] text-accent-primary font-bold">{t("scraper.batch.recommended")}</span>
                  </div>
                  
                  <div className="grid grid-cols-1 gap-3">
                    {activeProviders.length === 0 ? (
                      <div className="flex items-center gap-3 p-4 bg-red-500/10 border border-red-500/20 rounded-2xl text-red-400">
                        <AlertCircle className="w-5 h-5" />
                        <span className="text-sm font-bold">{t("scraper.batch.noSourceEnabled")}</span>
                      </div>
                    ) : (
                      activeProviders.map(p => (
                        <button
                          key={p.id}
                          onClick={() => setSelectedProviderId(p.id)}
                          className={clsx(
                            "flex items-center gap-4 p-4 rounded-2xl border transition-all duration-300 text-left group",
                            selectedProviderId === p.id 
                              ? "bg-accent-primary/10 border-accent-primary shadow-lg shadow-accent-primary/5" 
                              : "bg-bg-secondary/50 border-transparent hover:bg-bg-tertiary hover:border-border-hover"
                          )}
                        >
                          <div className={clsx(
                            "w-10 h-10 rounded-xl flex items-center justify-center transition-colors",
                            selectedProviderId === p.id ? "bg-accent-primary text-bg-primary" : "bg-bg-tertiary text-text-secondary group-hover:text-accent-primary"
                          )}>
                            {p.id === "screenscraper" ? <Globe className="w-5 h-5" /> : <Activity className="w-5 h-5" />}
                          </div>
                          <div className="flex-1">
                            <div className="font-bold text-text-primary group-hover:text-accent-primary transition-colors">{p.name}</div>
                            <div className="text-[10px] text-text-muted uppercase font-bold tracking-tighter mt-0.5">
                              {p.capabilities.join(" • ")}
                            </div>
                          </div>
                          {selectedProviderId === p.id && (
                            <CheckCircle2 className="w-5 h-5 text-accent-primary" />
                          )}
                        </button>
                      ))
                    )}
                  </div>
                </div>
              </>
            )}

            <div className="flex justify-end gap-4 pt-2">
              {!isBatchScraping ? (
                <>
                  <button
                    onClick={onClose}
                    className="px-6 py-2.5 rounded-xl text-text-secondary hover:text-text-primary hover:bg-bg-tertiary transition-all font-bold text-sm"
                  >
                    {t("common.cancel")}
                  </button>
                  <button
                    onClick={handleStart}
                    disabled={!selectedProviderId || activeProviders.length === 0}
                    className="flex items-center gap-2 px-10 py-2.5 bg-accent-primary text-bg-primary text-sm font-black rounded-xl hover:opacity-90 active:scale-95 transition-all shadow-xl shadow-accent-primary/25 disabled:opacity-30 disabled:cursor-not-allowed disabled:shadow-none"
                  >
                    <Database className="w-4 h-4" />
                    {t("scraper.batch.start")}
                  </button>
                </>
              ) : (
                batchProgress?.finished && (
                  <button
                    onClick={onClose}
                    className="px-10 py-2.5 bg-accent-primary text-bg-primary text-sm font-black rounded-xl hover:opacity-90 active:scale-95 transition-all shadow-xl shadow-accent-primary/25"
                  >
                    {t("common.finish")}
                  </button>
                )
              )}
            </div>
          </div>
        </motion.div>
      </div>
    </AnimatePresence>
  );
}
