import { useState, useEffect, useRef } from "react";
import { Database, Globe, Activity, PlayCircle, CheckCircle2, AlertCircle } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useScraperStore } from "@/stores/scraperStore";
import { useRomStore, type BatchScrapeScope } from "@/stores/romStore";
import { Button, Dialog, Spinner } from "@/components/ui";
import { clsx } from "clsx";
import { scraperApi } from "@/lib/api";
import MediaTypeSelector from "./MediaTypeSelector";

interface BatchScrapeDialogProps {
  isOpen: boolean;
  onClose: () => void;
  scope?: BatchScrapeScope;
}

export default function BatchScrapeDialog({ isOpen, onClose, scope = "selection" }: BatchScrapeDialogProps) {
  const { t } = useTranslation();
  const { providers, fetchProviders } = useScraperStore();
  const { startBatchScrape, selectedRomIds, selectedSystem, systemRoms, isBatchScraping, batchProgress } = useRomStore();
  const [selectedProviderIds, setSelectedProviderIds] = useState<string[]>([]);
  const [mediaTypes, setMediaTypes] = useState<string[]>([]);
  const providersInitialized = useRef(false);

  useEffect(() => {
    fetchProviders();
    scraperApi.getMediaTypes().then(setMediaTypes).catch(console.error);
  }, [fetchProviders]);

  useEffect(() => {
    const enabledIds = providers.filter(p => p.enabled).map(p => p.id);
    if (enabledIds.length > 0 && !providersInitialized.current) {
      setSelectedProviderIds(enabledIds);
      providersInitialized.current = true;
    }
  }, [providers]);

  const handleStart = async () => {
    if (selectedProviderIds.length === 0) return;
    await startBatchScrape(selectedProviderIds, mediaTypes, scope);
  };

  const toggleProvider = (providerId: string) => {
    setSelectedProviderIds(current => current.includes(providerId)
      ? current.filter(id => id !== providerId)
      : [...current, providerId]);
  };

  // 批量抓取进行中禁止关闭(完成后允许)
  const handleClose = () => {
    if (isBatchScraping && !batchProgress?.finished) return;
    onClose();
  };

  const activeProviders = providers.filter(p => p.enabled);
  const targetCount = scope === "library"
    ? systemRoms.reduce((count, item) => count + item.roms.length, 0)
    : scope === "platform"
      ? systemRoms.find(item => item.system === selectedSystem)?.roms.length || 0
      : selectedRomIds.size;
  const progressPercent = batchProgress
    ? Math.round((batchProgress.current / batchProgress.total) * 100)
    : 0;

  return (
    <Dialog
      open={isOpen}
      onClose={handleClose}
      title={
        <span className="flex items-center gap-3">
          <Database className="w-5 h-5 text-accent-primary shrink-0" />
          {t("scraper.batch.title")}
        </span>
      }
      footer={
        !isBatchScraping ? (
          <>
            <Button variant="ghost" onClick={onClose}>
              {t("common.cancel")}
            </Button>
            <Button
              onClick={handleStart}
              disabled={selectedProviderIds.length === 0 || activeProviders.length === 0 || targetCount === 0}
            >
              <Database className="w-4 h-4" />
              {t("scraper.batch.start")}
            </Button>
          </>
        ) : (
          batchProgress?.finished && (
            <Button onClick={onClose}>{t("common.finish")}</Button>
          )
        )
      }
    >
      <div className="space-y-6 py-2">
        {isBatchScraping ? (
          /* 进度展示 */
          <div className="space-y-6 py-2">
            <div className="flex items-center justify-between mb-2">
              <div className="flex items-center gap-2 text-accent-primary">
                <Spinner size={20} />
                <span className="font-bold text-sm uppercase tracking-widest">{t("scraper.batch.scraping")}</span>
              </div>
              <span className="text-xl font-black text-text-primary">{progressPercent}%</span>
            </div>

            {/* 进度条 */}
            <div className="h-4 bg-bg-tertiary rounded-full overflow-hidden border-[length:var(--border-width)] border-border-default">
              <div
                className="h-full bg-gradient-to-r from-accent-primary to-accent-primary/60 transition-[width] duration-[var(--motion-normal)] ease-[var(--motion-easing)]"
                style={{ width: `${progressPercent}%` }}
              />
            </div>

            <div className="flex flex-col items-center gap-2 p-4 bg-bg-secondary rounded-[var(--radius-lg)] border-[length:var(--border-width)] border-border-default">
              <div className="text-sm font-medium text-text-primary">
                {batchProgress?.message || t("scraper.batch.preparing")}
              </div>
              <div className="text-xs text-text-muted">
                {t("scraper.batch.processed", { current: batchProgress?.current || 0, total: batchProgress?.total || 0 })}
              </div>
            </div>

            {batchProgress?.finished && (
              <div className="flex items-center gap-3 p-4 bg-accent-success/10 border-[length:var(--border-width)] border-accent-success/20 rounded-[var(--radius-lg)] text-accent-success">
                <CheckCircle2 className="w-6 h-6 shrink-0" />
                <div>
                  <div className="font-bold">{t("scraper.batch.completed")}</div>
                  <div className="text-xs opacity-80">{t("scraper.batch.completedDesc")}</div>
                </div>
              </div>
            )}
          </div>
        ) : (
          /* 配置界面 */
          <>
            <div className="flex items-start gap-4 p-4 bg-bg-secondary rounded-[var(--radius-lg)] border-[length:var(--border-width)] border-border-default">
              <PlayCircle className="w-10 h-10 text-accent-primary shrink-0 opacity-80" />
              <div>
                <p className="text-text-primary font-bold text-lg leading-tight">
                  {t("scraper.batch.pendingTitle", { count: targetCount })}
                </p>
                <p className="text-sm text-text-muted mt-1 leading-relaxed">
                  {t("scraper.batch.pendingDesc")}
                </p>
              </div>
            </div>

            <div className="space-y-4">
              <MediaTypeSelector value={mediaTypes} onChange={setMediaTypes} />
              <div className="flex items-center justify-between px-1">
                <label className="text-xs font-black text-text-muted uppercase tracking-widest">{t("scraper.batch.selectSource")}</label>
                <span className="text-[10px] text-accent-primary font-bold">{t("scraper.batch.recommended")}</span>
              </div>

              <div className="grid grid-cols-1 gap-3">
                {activeProviders.length === 0 ? (
                  <div className="flex items-center gap-3 p-4 bg-accent-error/10 border-[length:var(--border-width)] border-accent-error/20 rounded-[var(--radius-lg)] text-accent-error">
                    <AlertCircle className="w-5 h-5" />
                    <span className="text-sm font-bold">{t("scraper.batch.noSourceEnabled")}</span>
                  </div>
                ) : (
                  activeProviders.map(p => {
                    const isSelected = selectedProviderIds.includes(p.id);
                    return (
                      <button
                        key={p.id}
                        type="button"
                        onClick={() => toggleProvider(p.id)}
                        aria-pressed={isSelected}
                        className={clsx(
                          "rr-card flex items-center gap-4 p-4 rounded-[var(--radius-lg)] border-[length:var(--border-width)] text-left group",
                          "transition-all duration-[var(--motion-fast)] ease-[var(--motion-easing)]",
                          "focus-visible:outline-none focus-visible:border-border-highlight",
                          isSelected
                            ? "bg-accent-primary/10 border-border-highlight"
                            : "bg-bg-secondary/50 border-border-default hover:bg-bg-tertiary hover:border-border-hover"
                        )}
                      >
                        <div className={clsx(
                          "w-10 h-10 rounded-[var(--radius-md)] flex items-center justify-center",
                          "transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)]",
                          isSelected ? "bg-accent-primary text-bg-primary" : "bg-bg-tertiary text-text-secondary group-hover:text-accent-primary"
                        )}>
                          {p.id === "screenscraper" ? <Globe className="w-5 h-5" /> : <Activity className="w-5 h-5" />}
                        </div>
                        <div className="flex-1">
                          <div className="font-bold text-text-primary group-hover:text-accent-primary transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)]">{p.name}</div>
                          <div className="text-[10px] text-text-muted font-bold tracking-tighter mt-0.5">
                            {p.capabilities
                              .map((cap) => t(`scraper.capabilities.${cap}`, { defaultValue: cap }))
                              .join(" · ")}
                          </div>
                        </div>
                        {isSelected && (
                          <CheckCircle2 className="w-5 h-5 text-accent-primary" />
                        )}
                      </button>
                    );
                  })
                )}
              </div>
            </div>
          </>
        )}
      </div>
    </Dialog>
  );
}
