import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { X, Database } from "lucide-react";
import { useScraperStore } from "@/stores/scraperStore";
import { useRomStore } from "@/stores/romStore";

interface BatchScrapeDialogProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function BatchScrapeDialog({ isOpen, onClose }: BatchScrapeDialogProps) {
  const { configs, fetchConfigs } = useScraperStore();
  const { startBatchScrape, selectedRomIds } = useRomStore();
  const [provider, setProvider] = useState("steamgriddb");

  useEffect(() => {
    fetchConfigs();
  }, [fetchConfigs]);

  const handleStart = async () => {
    onClose();
    await startBatchScrape(provider);
  };

  if (!isOpen) return null;

  const activeProviders = Object.values(configs).filter(c => c.enabled);

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
          className="relative w-full max-w-md bg-bg-primary border border-border-default rounded-2xl shadow-2xl overflow-hidden"
        >
          <div className="p-6 border-b border-border-default flex items-center justify-between">
            <h2 className="text-xl font-bold text-text-primary">Batch Scrape</h2>
            <button onClick={onClose} className="p-2 rounded-lg hover:bg-bg-tertiary text-text-secondary">
              <X className="w-6 h-6" />
            </button>
          </div>

          <div className="p-6 space-y-6">
            <p className="text-text-secondary">
              You are about to scrape metadata and media for <span className="text-text-primary font-bold">{selectedRomIds.size}</span> selected games.
            </p>

            <div>
              <label className="text-xs font-bold text-text-muted uppercase tracking-wider mb-2 block">Provider</label>
              <select
                value={provider}
                onChange={(e) => setProvider(e.target.value)}
                className="w-full bg-bg-secondary border border-border-default rounded-lg px-3 py-3 text-sm text-text-primary focus:outline-none focus:border-accent-primary"
              >
                {activeProviders.length === 0 ? (
                  <option disabled>No providers enabled</option>
                ) : (
                  activeProviders.map(c => (
                    <option key={c.provider} value={c.provider}>
                      {c.provider.toUpperCase()}
                    </option>
                  ))
                )}
              </select>
            </div>

            <div className="flex justify-end gap-3 pt-4">
              <button
                onClick={onClose}
                className="px-6 py-2 rounded-xl text-text-primary hover:bg-bg-tertiary transition-colors font-medium text-sm"
              >
                Cancel
              </button>
              <button
                onClick={handleStart}
                className="px-8 py-2 bg-accent-primary hover:bg-accent-primary/90 text-text-primary rounded-xl font-bold transition-colors shadow-lg shadow-accent-primary/20 flex items-center gap-2"
              >
                <Database className="w-4 h-4" />
                Start Scrape
              </button>
            </div>
          </div>
        </motion.div>
      </div>
    </AnimatePresence>
  );
}
