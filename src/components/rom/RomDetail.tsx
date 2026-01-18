import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { X, Calendar, User, Building2, Globe, Gamepad2, Play, Star } from "lucide-react";
import { useTranslation } from "react-i18next";
import { resolveMediaUrlAsync } from "@/lib/api";
import type { Rom } from "@/types";
import ScrapeDialog from "./ScrapeDialog";

interface RomDetailProps {
  rom: Rom | null;
  onClose: () => void;
}

function useMediaUrl(path: string | undefined): string | null {
  const [url, setUrl] = useState<string | null>(null);
  useEffect(() => {
    if (!path) {
      setUrl(null);
      return;
    }
    resolveMediaUrlAsync(path).then(setUrl);
  }, [path]);
  return url;
}

export default function RomDetail({ rom, onClose }: RomDetailProps) {
  const { t } = useTranslation();
  const [isScrapeDialogOpen, setIsScrapeDialogOpen] = useState(false);
  
  const heroSource = rom?.background || rom?.screenshot || rom?.box_front;
  const heroUrl = useMediaUrl(heroSource);
  const videoUrl = useMediaUrl(rom?.video);
  const logoUrl = useMediaUrl(rom?.logo);

  if (!rom) return null;

  return (
    <>
      <AnimatePresence>
        {rom && (
          <>
            {/* Backdrop */}
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              onClick={onClose}
              className="fixed inset-0 bg-black/60 backdrop-blur-sm z-40"
            />

            {/* Panel */}
            <motion.div
              initial={{ x: "100%" }}
              animate={{ x: 0 }}
              exit={{ x: "100%" }}
              transition={{ type: "spring", damping: 25, stiffness: 200 }}
              className="fixed right-0 top-0 bottom-0 w-full max-w-lg bg-bg-primary border-l border-border-default z-50 overflow-y-auto shadow-2xl"
            >
              {/* Header Image/Video */}
              <div className="relative aspect-video w-full bg-bg-secondary overflow-hidden">
                {videoUrl ? (
                  <video
                    src={videoUrl}
                    autoPlay
                    muted
                    loop
                    className="w-full h-full object-cover"
                  />
                ) : heroUrl ? (
                  <img src={heroUrl} alt="" className="w-full h-full object-cover" />
                ) : (
                  <div className="absolute inset-0 flex items-center justify-center text-text-muted/10">
                    <Gamepad2 className="w-24 h-24" />
                  </div>
                )}

                <button
                  onClick={onClose}
                  className="absolute top-4 right-4 p-2 rounded-full bg-bg-primary/50 hover:bg-bg-tertiary text-text-primary transition-colors backdrop-blur-md z-10"
                >
                  <X className="w-5 h-5" />
                </button>

                <div className="absolute bottom-0 left-0 right-0 p-6 bg-gradient-to-t from-bg-primary via-bg-primary/40 to-transparent">
                  {logoUrl ? (
                    <img src={logoUrl} alt={rom.name} className="h-12 mb-2 object-contain drop-shadow-lg" />
                  ) : (
                    <h2 className="text-3xl font-bold text-text-primary mb-2 leading-tight">
                      {rom.name}
                    </h2>
                  )}
                  <div className="flex items-center gap-3 text-sm">
                    <span className="px-2 py-0.5 rounded bg-bg-tertiary text-text-primary font-medium uppercase text-xs border border-border-default">
                      {rom.system}
                    </span>
                    {rom.rating && (
                      <div className="flex items-center gap-1 text-accent-warning">
                        <Star className="w-4 h-4 fill-current" />
                        <span>{rom.rating}</span>
                      </div>
                    )}
                  </div>
                </div>
              </div>

              {/* Actions */}
              <div className="p-6 flex gap-3 border-b border-border-default">
                <button className="flex-1 flex items-center justify-center gap-2 py-3 bg-accent-primary hover:bg-accent-primary/90 text-text-primary rounded-xl font-medium transition-colors shadow-lg shadow-accent-primary/20">
                  <Play className="w-5 h-5" />
                  Play
                </button>
                <button className="flex-1 py-3 bg-bg-tertiary hover:bg-border-hover text-text-primary rounded-xl font-medium transition-colors border border-border-default">
                  {t("common.edit", { defaultValue: "Edit" })}
                </button>
                <button
                  onClick={() => setIsScrapeDialogOpen(true)}
                  className="flex-1 py-3 bg-bg-tertiary hover:bg-border-hover text-text-primary rounded-xl font-medium transition-colors border border-border-default"
                >
                  Scrape
                </button>
              </div>

              {/* Details */}
              <div className="p-6 space-y-6">
                {/* Description */}
                <div>
                  <h3 className="text-sm font-medium text-text-muted uppercase tracking-widest mb-3">
                    Description
                  </h3>
                  <p className="text-text-secondary leading-relaxed text-sm">
                    {rom.description || rom.summary || "No description available."}
                  </p>
                </div>

                {/* Info Grid */}
                <div className="grid grid-cols-2 gap-4">
                  <div className="space-y-1">
                    <span className="text-xs text-text-muted flex items-center gap-1">
                      <Calendar className="w-3 h-3" /> Release Date
                    </span>
                    <p className="text-sm text-text-primary">
                      {rom.release || "Unknown"}
                    </p>
                  </div>
                  <div className="space-y-1">
                    <span className="text-xs text-text-muted flex items-center gap-1">
                      <Building2 className="w-3 h-3" /> Developer
                    </span>
                    <p className="text-sm text-text-primary truncate">
                      {rom.developer || "Unknown"}
                    </p>
                  </div>
                  <div className="space-y-1">
                    <span className="text-xs text-text-muted flex items-center gap-1">
                      <Globe className="w-3 h-3" /> Publisher
                    </span>
                    <p className="text-sm text-text-primary truncate">
                      {rom.publisher || "Unknown"}
                    </p>
                  </div>
                  <div className="space-y-1">
                    <span className="text-xs text-text-muted flex items-center gap-1">
                      <User className="w-3 h-3" /> Players
                    </span>
                    <p className="text-sm text-text-primary">
                      {rom.players || "Unknown"}
                    </p>
                  </div>
                </div>

                {/* File Info */}
                <div className="pt-6 border-t border-border-default">
                  <h3 className="text-sm font-medium text-text-muted uppercase tracking-widest mb-3">
                    File Info
                  </h3>
                  <div className="bg-bg-secondary rounded-lg p-4 space-y-2 text-xs font-mono text-text-secondary border border-border-default">
                    {/* Size 暂未提供 */}
                    {/* <div className="flex justify-between">
                      <span>Size:</span>
                      <span className="text-text-primary">{Math.round((rom.size / 1024 / 1024) * 100) / 100} MB</span>
                    </div> */}
                    <div className="flex justify-between">
                      <span>Path:</span>
                      <span className="text-text-primary truncate max-w-[200px]" title={rom.file}>{rom.file}</span>
                    </div>
                    {/* CRC32/MD5 暂未提供 */}
                  </div>
                </div>
              </div>
            </motion.div>
          </>
        )}
      </AnimatePresence>
      <ScrapeDialog
        rom={rom}
        isOpen={isScrapeDialogOpen}
        onClose={() => setIsScrapeDialogOpen(false)}
      />
    </>
  );
}
