import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import { 
  X, Calendar, User, Building2, Globe, Gamepad2, Play, Star, 
  Eye, EyeOff, Save, Download, Edit2, Trash2, LayoutGrid, Info, Check, Loader2
} from "lucide-react";
import { resolveMediaUrlAsync } from "@/lib/api";
import type { Rom } from "@/types";
import { useRomStore } from "@/stores/romStore";
import ScrapeDialog from "./ScrapeDialog";
import { clsx } from "clsx";

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
  const { exportData } = useRomStore();
  const [isScrapeDialogOpen, setIsScrapeDialogOpen] = useState(false);
  const [isPreview, setIsPreview] = useState(false);
  const [isExporting, setIsExporting] = useState(false);

  useEffect(() => {
    if (rom) {
      setIsPreview(rom.has_temp_metadata);
    }
  }, [rom]);

  const displayData = isPreview && rom?.temp_data ? {
    ...rom,
    name: rom.temp_data.name || rom.name,
    description: rom.temp_data.description || rom.description,
    developer: rom.temp_data.developer || rom.developer,
    publisher: rom.temp_data.publisher || rom.publisher,
    genre: rom.temp_data.genre || rom.genre,
    release: rom.temp_data.release || rom.release,
    rating: rom.temp_data.rating || rom.rating,
    box_front: rom.temp_data.box_front || rom.box_front,
  } : rom;

  const heroSource = displayData?.background || displayData?.screenshot || displayData?.box_front;
  const heroUrl = useMediaUrl(heroSource);
  const videoUrl = useMediaUrl(displayData?.video);
  const logoUrl = useMediaUrl(displayData?.logo);

  const handleExport = async () => {
    if (!rom) return;
    setIsExporting(true);
    try {
      await exportData(rom.system, rom.directory);
      setIsPreview(false);
    } catch (error) {
      console.error("Export failed:", error);
    } finally {
      setIsExporting(false);
    }
  };

  if (!rom) return null;

  return (
    <>
      <AnimatePresence>
        {rom && (
          <>
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              onClick={onClose}
              className="fixed inset-0 bg-black/60 backdrop-blur-sm z-40"
            />

            <motion.div
              initial={{ x: "100%" }}
              animate={{ x: 0 }}
              exit={{ x: "100%" }}
              transition={{ type: "spring", damping: 25, stiffness: 200 }}
              className="fixed right-0 top-0 bottom-0 w-full max-w-lg bg-bg-primary border-l border-border-default z-50 overflow-y-auto shadow-2xl flex flex-col"
            >
              {/* Header Image/Video */}
              <div className="relative aspect-video w-full bg-bg-secondary overflow-hidden shrink-0">
                {videoUrl ? (
                  <video src={videoUrl} autoPlay muted loop className="w-full h-full object-cover" />
                ) : heroUrl ? (
                  <img src={heroUrl} alt="" className="w-full h-full object-cover" />
                ) : (
                  <div className="absolute inset-0 flex items-center justify-center text-text-muted/10">
                    <Gamepad2 className="w-24 h-24" />
                  </div>
                )}

                <div className="absolute top-4 right-4 left-4 flex justify-between items-center z-10">
                  <button
                    onClick={onClose}
                    className="p-2 rounded-full bg-bg-primary/50 hover:bg-bg-tertiary text-text-primary transition-colors backdrop-blur-md"
                  >
                    <X className="w-5 h-5" />
                  </button>

                  <div className="flex gap-2">
                    {rom.has_temp_metadata && (
                      <button
                        onClick={() => setIsPreview(!isPreview)}
                        className={clsx(
                          "flex items-center gap-2 px-3 py-1.5 rounded-full font-bold text-xs backdrop-blur-md transition-all shadow-lg",
                          isPreview ? "bg-accent-primary text-bg-primary" : "bg-black/40 text-white border border-white/10"
                        )}
                      >
                        {isPreview ? <Eye className="w-3.5 h-3.5" /> : <EyeOff className="w-3.5 h-3.5" />}
                        {isPreview ? "预览数据" : "原始数据"}
                      </button>
                    )}
                  </div>
                </div>

                <div className="absolute bottom-0 left-0 right-0 p-6 bg-gradient-to-t from-bg-primary via-bg-primary/40 to-transparent">
                  {logoUrl ? (
                    <img src={logoUrl} alt={displayData?.name} className="h-12 mb-2 object-contain drop-shadow-lg" />
                  ) : (
                    <h2 className="text-3xl font-bold text-text-primary mb-2 leading-tight">
                      {displayData?.name}
                    </h2>
                  )}
                  <div className="flex items-center gap-3 text-sm">
                    <span className="px-2 py-0.5 rounded bg-bg-tertiary text-text-primary font-medium uppercase text-xs border border-border-default">
                      {rom.system}
                    </span>
                    {displayData?.rating && (
                      <div className="flex items-center gap-1 text-accent-warning">
                        <Star className="w-4 h-4 fill-current" />
                        <span>{displayData.rating}</span>
                      </div>
                    )}
                  </div>
                </div>
              </div>

              {/* Actions */}
              <div className="p-6 flex gap-3 border-b border-border-default bg-bg-primary sticky top-0 z-20">
                <button className="flex-1 flex items-center justify-center gap-2 py-3 bg-accent-primary hover:bg-accent-primary/90 text-text-primary rounded-xl font-bold transition-all shadow-lg shadow-accent-primary/20 active:scale-95">
                  <Play className="w-5 h-5 fill-current" />
                  Play
                </button>
                <button
                  onClick={() => setIsScrapeDialogOpen(true)}
                  className="px-4 py-3 bg-bg-tertiary hover:bg-border-hover text-text-primary rounded-xl font-bold transition-all border border-border-default"
                  title="抓取元数据"
                >
                  <Download className="w-5 h-5" />
                </button>
                <button className="px-4 py-3 bg-bg-tertiary hover:bg-border-hover text-text-primary rounded-xl font-bold transition-all border border-border-default">
                  <Edit2 className="w-5 h-5" />
                </button>
              </div>

              {/* Details */}
              <div className="flex-1 p-6 space-y-8">
                {isPreview && (
                  <div className="p-4 bg-accent-primary/5 border border-accent-primary/20 rounded-2xl flex items-start gap-4">
                    <div className="w-10 h-10 rounded-xl bg-accent-primary/10 flex items-center justify-center shrink-0">
                      <Info className="w-5 h-5 text-accent-primary" />
                    </div>
                    <div className="flex-1">
                      <h4 className="text-sm font-bold text-text-primary tracking-tight">预览抓取数据</h4>
                      <p className="text-xs text-text-muted mt-1 leading-relaxed">
                        这些数据仅保存在临时目录中。您可以检查描述和封面，满意后点击底部的“导出到库”将其永久应用。
                      </p>
                    </div>
                  </div>
                )}

                <div>
                  <h3 className="text-xs font-black text-text-muted uppercase tracking-widest mb-3 flex items-center gap-2">
                    <LayoutGrid className="w-3.5 h-3.5" />
                    游戏描述
                  </h3>
                  <p className="text-text-secondary leading-relaxed text-sm font-medium">
                    {displayData?.description || displayData?.summary || "暂无描述。"}
                  </p>
                </div>

                <div className="grid grid-cols-2 gap-6">
                  <InfoItem icon={<Calendar />} label="发行日期" value={displayData?.release} />
                  <InfoItem icon={<Building2 />} label="开发商" value={displayData?.developer} />
                  <InfoItem icon={<Globe />} label="发行商" value={displayData?.publisher} />
                  <InfoItem icon={<User />} label="游戏类型" value={displayData?.genre} />
                </div>

                <div className="pt-6 border-t border-border-default">
                  <h3 className="text-xs font-black text-text-muted uppercase tracking-widest mb-3">文件详情</h3>
                  <div className="bg-bg-secondary/50 rounded-xl p-4 space-y-2 text-[11px] font-mono text-text-secondary border border-border-default shadow-inner">
                    <div className="flex justify-between gap-4">
                      <span className="shrink-0 opacity-50 uppercase">文件名:</span>
                      <span className="text-text-primary truncate" title={rom.file}>{rom.file}</span>
                    </div>
                    <div className="flex justify-between gap-4">
                      <span className="shrink-0 opacity-50 uppercase">本地路径:</span>
                      <span className="text-text-primary truncate" title={rom.directory}>{rom.directory}</span>
                    </div>
                  </div>
                </div>
              </div>

              {/* Footer Actions */}
              <div className="p-6 border-t border-border-default bg-bg-secondary/30 flex justify-between items-center px-8 shrink-0">
                <button className="flex items-center gap-2 px-4 py-2 text-sm font-bold text-red-400 hover:bg-red-400/10 rounded-xl transition-all">
                  <Trash2 className="w-4 h-4" />
                  移除
                </button>
                
                <div className="flex gap-3">
                  {isPreview ? (
                    <button
                      onClick={handleExport}
                      disabled={isExporting}
                      className="flex items-center gap-2 px-8 py-2.5 bg-accent-primary text-bg-primary rounded-xl font-black text-sm shadow-xl shadow-accent-primary/20 hover:opacity-90 transition-all active:scale-95 disabled:opacity-50"
                    >
                      {isExporting ? <Loader2 className="w-4 h-4 animate-spin" /> : <Save className="w-4 h-4" />}
                      导出到库
                    </button>
                  ) : (
                    <button
                      onClick={onClose}
                      className="flex items-center gap-2 px-8 py-2.5 bg-bg-tertiary text-text-primary rounded-xl font-black text-sm hover:bg-border-hover transition-all active:scale-95"
                    >
                      <Check className="w-4 h-4" />
                      完成
                    </button>
                  )}
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

function InfoItem({ icon, label, value }: { icon: React.ReactNode, label: string, value?: string }) {
  return (
    <div className="flex items-start gap-3">
      <div className="w-8 h-8 rounded-lg bg-bg-tertiary flex items-center justify-center text-accent-primary shrink-0 border border-border-default">
        {icon}
      </div>
      <div className="min-w-0">
        <div className="text-[9px] font-bold text-text-muted uppercase tracking-widest leading-none mb-1">{label}</div>
        <div className="text-sm font-bold text-text-primary truncate">{value || "未知"}</div>
      </div>
    </div>
  );
}
