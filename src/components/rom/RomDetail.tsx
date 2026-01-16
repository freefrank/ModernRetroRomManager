import { motion, AnimatePresence } from "framer-motion";
import { X, Calendar, User, Building2, Globe, Gamepad2, Play, Star } from "lucide-react";
import { useTranslation } from "react-i18next";
import type { Rom } from "@/types";

interface RomDetailProps {
  rom: Rom | null;
  onClose: () => void;
}

export default function RomDetail({ rom, onClose }: RomDetailProps) {
  const { t } = useTranslation();

  if (!rom) return null;

  return (
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
            className="fixed right-0 top-0 bottom-0 w-full max-w-lg bg-[#0B0C15] border-l border-white/10 z-50 overflow-y-auto shadow-2xl"
          >
            {/* Header Image */}
            <div className="relative aspect-video w-full bg-[#151621]">
              <div className="absolute inset-0 flex items-center justify-center text-white/5">
                <Gamepad2 className="w-24 h-24" />
              </div>
              
              <button
                onClick={onClose}
                className="absolute top-4 right-4 p-2 rounded-full bg-black/50 hover:bg-white/10 text-white transition-colors backdrop-blur-md"
              >
                <X className="w-5 h-5" />
              </button>

              <div className="absolute bottom-0 left-0 right-0 p-6 bg-gradient-to-t from-[#0B0C15] to-transparent">
                <h2 className="text-3xl font-bold text-white mb-2 leading-tight">
                  {rom.metadata?.name || rom.filename}
                </h2>
                <div className="flex items-center gap-3 text-sm">
                  <span className="px-2 py-0.5 rounded bg-white/10 text-white font-medium uppercase text-xs border border-white/5">
                    {rom.systemId}
                  </span>
                  {rom.metadata?.rating && (
                    <div className="flex items-center gap-1 text-accent-warning">
                      <Star className="w-4 h-4 fill-current" />
                      <span>{rom.metadata.rating.toFixed(1)}</span>
                    </div>
                  )}
                </div>
              </div>
            </div>

            {/* Actions */}
            <div className="p-6 flex gap-3 border-b border-white/5">
              <button className="flex-1 flex items-center justify-center gap-2 py-3 bg-accent-primary hover:bg-accent-primary/90 text-white rounded-xl font-medium transition-colors shadow-lg shadow-accent-primary/20">
                <Play className="w-5 h-5" />
                Play
              </button>
              <button className="flex-1 py-3 bg-white/5 hover:bg-white/10 text-white rounded-xl font-medium transition-colors border border-white/5">
                {t("common.edit", { defaultValue: "Edit" })}
              </button>
              <button className="flex-1 py-3 bg-white/5 hover:bg-white/10 text-white rounded-xl font-medium transition-colors border border-white/5">
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
                  {rom.metadata?.description || "No description available."}
                </p>
              </div>

              {/* Info Grid */}
              <div className="grid grid-cols-2 gap-4">
                <div className="space-y-1">
                  <span className="text-xs text-text-muted flex items-center gap-1">
                    <Calendar className="w-3 h-3" /> Release Date
                  </span>
                  <p className="text-sm text-white">
                    {rom.metadata?.releaseDate || "Unknown"}
                  </p>
                </div>
                <div className="space-y-1">
                  <span className="text-xs text-text-muted flex items-center gap-1">
                    <Building2 className="w-3 h-3" /> Developer
                  </span>
                  <p className="text-sm text-white truncate">
                    {rom.metadata?.developer || "Unknown"}
                  </p>
                </div>
                <div className="space-y-1">
                  <span className="text-xs text-text-muted flex items-center gap-1">
                    <Globe className="w-3 h-3" /> Publisher
                  </span>
                  <p className="text-sm text-white truncate">
                    {rom.metadata?.publisher || "Unknown"}
                  </p>
                </div>
                <div className="space-y-1">
                  <span className="text-xs text-text-muted flex items-center gap-1">
                    <User className="w-3 h-3" /> Players
                  </span>
                  <p className="text-sm text-white">
                    {rom.metadata?.players || "Unknown"}
                  </p>
                </div>
              </div>

              {/* File Info */}
              <div className="pt-6 border-t border-white/5">
                <h3 className="text-sm font-medium text-text-muted uppercase tracking-widest mb-3">
                  File Info
                </h3>
                <div className="bg-[#151621] rounded-lg p-4 space-y-2 text-xs font-mono text-text-secondary border border-white/5">
                  <div className="flex justify-between">
                    <span>Size:</span>
                    <span className="text-white">{Math.round((rom.size / 1024 / 1024) * 100) / 100} MB</span>
                  </div>
                  <div className="flex justify-between">
                    <span>Path:</span>
                    <span className="text-white truncate max-w-[200px]" title={rom.path}>{rom.filename}</span>
                  </div>
                  <div className="flex justify-between">
                    <span>CRC32:</span>
                    <span className="text-white">{rom.crc32 || "-"}</span>
                  </div>
                  <div className="flex justify-between">
                    <span>MD5:</span>
                    <span className="text-white truncate max-w-[200px]" title={rom.md5}>{rom.md5 || "-"}</span>
                  </div>
                </div>
              </div>
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  );
}
