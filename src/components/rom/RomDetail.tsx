import { useState, useEffect } from "react";
import { motion, AnimatePresence } from "framer-motion";
import {
  X, Calendar, User, Building2, Globe, Gamepad2, Star,
  Eye, EyeOff, Save, Download, Edit2, Trash2, LayoutGrid, Info, Check, Loader2, Play, Wand2
} from "lucide-react";
import { resolveMediaUrlAsync, scraperApi, ps3Api } from "@/lib/api";
import type { Rom } from "@/types";
import { useRomStore } from "@/stores/romStore";
import { useTranslation } from "react-i18next";
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
  const { t } = useTranslation();
  const { exportData, updateTempMetadata, deleteTempMedia, isExporting, exportProgress } = useRomStore();
  const [isScrapeDialogOpen, setIsScrapeDialogOpen] = useState(false);
  const [isPreview, setIsPreview] = useState(false);
  const [isEditing, setIsEditing] = useState(false);
  const [editForm, setEditForm] = useState<Partial<Rom & { _activeTab: string }>>({});
  const [tempMedia, setTempMedia] = useState<{ asset_type: string, path: string }[]>([]);
  const [isGeneratingBoxart, setIsGeneratingBoxart] = useState(false);

  useEffect(() => {
    if (rom) {
      setIsPreview(rom.has_temp_metadata);
      setIsEditing(false);
      if (rom.has_temp_metadata) {
        scraperApi.getTempMediaList(rom.system, rom.file).then(setTempMedia);
      } else {
        setTempMedia([]);
      }
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
    box_front: tempMedia.find(m => m.asset_type === "boxfront")?.path || rom.temp_data.box_front || rom.box_front,
    video: tempMedia.find(m => m.asset_type === "video")?.path || rom.temp_data.video || rom.video,
    logo: tempMedia.find(m => m.asset_type === "logo")?.path || rom.temp_data.logo || rom.logo,
    background: tempMedia.find(m => m.asset_type === "background")?.path || rom.temp_data.background || rom.background,
    screenshot: tempMedia.find(m => m.asset_type === "screenshot")?.path || rom.temp_data.screenshot || rom.screenshot,
  } : rom;

  const currentData = isEditing ? { ...displayData, ...editForm } : displayData;

  const heroSource = currentData?.background || currentData?.screenshot || currentData?.box_front;
  const heroUrl = useMediaUrl(heroSource);
  const videoUrl = useMediaUrl(currentData?.video);
  const logoUrl = useMediaUrl(currentData?.logo);

  const handleExport = async () => {
    if (!rom) return;
    try {
      await exportData(rom.system, rom.directory);
      setIsPreview(false);
    } catch (error) {
      console.error("Export failed:", error);
    }
  };

  const handleStartEdit = () => {
    setEditForm({ ...displayData, _activeTab: editForm._activeTab || "info" });
    setIsEditing(true);
  };

  const handleSaveEdit = async () => {
    if (!rom || !editForm) return;
    try {
      await updateTempMetadata(rom.system, rom.directory, rom.file, {
        name: editForm.name || "",
        description: editForm.description,
        developer: editForm.developer,
        publisher: editForm.publisher,
        genres: editForm.genre ? [editForm.genre] : [],
        release_date: editForm.release,
        rating: typeof editForm.rating === 'string' ? parseFloat(editForm.rating) : editForm.rating,
      });
      setIsEditing(false);
      setIsPreview(true);
    } catch (error) {
      console.error("Save edit failed:", error);
    }
  };

  const handleDeleteMedia = async (assetType: string) => {
    if (!rom) return;
    try {
      await deleteTempMedia(rom.system, rom.file, assetType);
      const newList = await scraperApi.getTempMediaList(rom.system, rom.file);
      setTempMedia(newList);
    } catch (error) {
      console.error("Delete media failed:", error);
    }
  };

  const handleGenerateBoxart = async () => {
    if (!rom) return;

    setIsGeneratingBoxart(true);
    try {
      const result = await ps3Api.generateBoxart(rom.file, rom.directory, rom.system);
      if (result.success) {
        alert(`Boxart 生成成功！\n路径: ${result.boxartPath}`);
        // 可以在这里刷新 ROM 数据或更新显示
      } else {
        alert(`Boxart 生成失败: ${result.error}`);
      }
    } catch (error) {
      console.error("Generate boxart failed:", error);
      alert(`Boxart 生成失败: ${error}`);
    } finally {
      setIsGeneratingBoxart(false);
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
                  <video key={videoUrl} src={videoUrl} autoPlay muted loop className="w-full h-full object-cover" />
                ) : heroUrl ? (
                  <img key={heroUrl} src={heroUrl} alt="" className="w-full h-full object-cover" />
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
                    {rom.has_temp_metadata && !isEditing && (
                      <button
                        onClick={() => setIsPreview(!isPreview)}
                        className={clsx(
                          "flex items-center gap-2 px-3 py-1.5 rounded-full font-bold text-xs backdrop-blur-md transition-all shadow-lg",
                          isPreview ? "bg-accent-primary text-bg-primary" : "bg-black/40 text-white border border-white/10"
                        )}
                      >
                        {isPreview ? <Eye className="w-3.5 h-3.5" /> : <EyeOff className="w-3.5 h-3.5" />}
                        {isPreview ? t("romDetail.preview.mode") : t("romDetail.preview.raw")}
                      </button>
                    )}
                  </div>
                </div>

                <div className="absolute bottom-0 left-0 right-0 p-6 bg-gradient-to-t from-bg-primary via-bg-primary/40 to-transparent">
                  {logoUrl ? (
                    <img src={logoUrl} alt={currentData?.name} className="h-12 mb-2 object-contain drop-shadow-lg" />
                  ) : (
                    <div className="mb-2">
                      {isEditing ? (
                        <input
                          value={editForm.name || ""}
                          onChange={e => setEditForm({ ...editForm, name: e.target.value })}
                          className="text-3xl font-bold bg-bg-secondary/50 border border-accent-primary/30 rounded px-2 w-full text-text-primary outline-none focus:border-accent-primary"
                        />
                      ) : (
                        <h2 className="text-3xl font-bold text-text-primary leading-tight">
                          {currentData?.name}
                        </h2>
                      )}
                    </div>
                  )}
                  <div className="flex items-center gap-3 text-sm">
                    <span className="px-2 py-0.5 rounded bg-bg-tertiary text-text-primary font-medium uppercase text-xs border border-border-default">
                      {rom.system}
                    </span>
                    {!isEditing && currentData?.rating && (
                      <div className="flex items-center gap-1 text-accent-warning">
                        <Star className="w-4 h-4 fill-current" />
                        <span>{currentData.rating}</span>
                      </div>
                    )}
                  </div>
                </div>
              </div>

              <div className="flex border-b border-border-default bg-bg-primary shrink-0 overflow-x-auto no-scrollbar">
                {["info", "media"].map((tab) => (
                  <button
                    key={tab}
                    onClick={() => setEditForm({ ...editForm, _activeTab: tab })}
                    className={clsx(
                      "px-6 py-4 text-xs font-black uppercase tracking-widest transition-all relative",
                      (editForm._activeTab || "info") === tab
                        ? "text-accent-primary"
                        : "text-text-muted hover:text-text-primary"
                    )}
                  >
                    {tab === "info" ? t("romDetail.tabs.info") : t("romDetail.tabs.media")}
                    {(editForm._activeTab || "info") === tab && (
                      <motion.div layoutId="active_tab" className="absolute bottom-0 left-0 right-0 h-0.5 bg-accent-primary" />
                    )}
                  </button>
                ))}
              </div>

              <div className="flex-1 p-6 space-y-8 overflow-y-auto custom-scrollbar">
                {(editForm._activeTab || "info") === "info" ? (
                  <>
                    {isPreview && !isEditing && (
                      <div className="p-4 bg-accent-primary/5 border border-accent-primary/20 rounded-2xl flex items-start gap-4">
                        <div className="w-10 h-10 rounded-xl bg-accent-primary/10 flex items-center justify-center shrink-0">
                          <Info className="w-5 h-5 text-accent-primary" />
                        </div>
                        <div className="flex-1">
                          <h4 className="text-sm font-bold text-text-primary tracking-tight">{t("romDetail.preview.note")}</h4>
                          <p className="text-xs text-text-muted mt-1 leading-relaxed">
                            {t("romDetail.preview.noteDesc")}
                          </p>
                        </div>
                      </div>
                    )}

                    <div>
                      <h3 className="text-xs font-black text-text-muted uppercase tracking-widest mb-3 flex items-center gap-2">
                        <LayoutGrid className="w-3.5 h-3.5" />
                        {t("romDetail.fields.description")}
                      </h3>
                      {isEditing ? (
                        <textarea
                          value={editForm.description || ""}
                          onChange={e => setEditForm({ ...editForm, description: e.target.value })}
                          rows={6}
                          className="w-full bg-bg-secondary/50 border border-border-default rounded-xl p-3 text-sm text-text-primary outline-none focus:border-accent-primary transition-all resize-none font-medium leading-relaxed"
                        />
                      ) : (
                        <p className="text-text-secondary leading-relaxed text-sm font-medium">
                          {currentData?.description || currentData?.summary || t("romDetail.fields.noDescription")}
                        </p>
                      )}
                    </div>

                    <div className="grid grid-cols-2 gap-6">
                      {isEditing ? (
                        <>
                          <EditItem label={t("romDetail.fields.releaseDate")} value={editForm.release} onChange={v => setEditForm({...editForm, release: v})} />
                          <EditItem label={t("romDetail.fields.developer")} value={editForm.developer} onChange={v => setEditForm({...editForm, developer: v})} />
                          <EditItem label={t("romDetail.fields.publisher")} value={editForm.publisher} onChange={v => setEditForm({...editForm, publisher: v})} />
                          <EditItem label={t("romDetail.fields.genre")} value={editForm.genre} onChange={v => setEditForm({...editForm, genre: v})} />
                        </>
                      ) : (
                        <>
                          <InfoItem icon={<Calendar />} label={t("romDetail.fields.releaseDate")} value={currentData?.release} />
                          <InfoItem icon={<Building2 />} label={t("romDetail.fields.developer")} value={currentData?.developer} />
                          <InfoItem icon={<Globe />} label={t("romDetail.fields.publisher")} value={currentData?.publisher} />
                          <InfoItem icon={<User />} label={t("romDetail.fields.genre")} value={currentData?.genre} />
                        </>
                      )}
                    </div>
                  </>
                ) : (
                  <div className="space-y-6">
                    <h3 className="text-xs font-black text-text-muted uppercase tracking-widest mb-4">{t("romDetail.media.title")}</h3>
                    {tempMedia.length === 0 ? (
                      <div className="py-12 border-2 border-dashed border-border-default rounded-2xl flex flex-col items-center justify-center text-text-muted opacity-50">
                        <Gamepad2 className="w-12 h-12 mb-2" />
                        <p className="text-sm font-bold">{t("romDetail.media.empty")}</p>
                      </div>
                    ) : (
                      <div className="grid grid-cols-2 gap-4">
                        {tempMedia.map(m => (
                          <div key={m.asset_type} className="group relative aspect-video bg-bg-secondary rounded-xl border border-border-default overflow-hidden shadow-sm">
                            <MediaPreview path={m.path} />
                            <div className="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity flex flex-col items-center justify-center p-2">
                              <span className="text-[10px] font-black text-white uppercase tracking-widest mb-2 bg-accent-primary px-2 py-0.5 rounded shadow-lg">
                                {m.asset_type}
                              </span>
                              <button 
                                onClick={() => handleDeleteMedia(m.asset_type)}
                                className="p-2 rounded-lg bg-red-500/20 text-red-400 hover:bg-red-500 hover:text-white transition-all border border-red-500/30"
                                title={t("common.delete")}
                              >
                                <Trash2 className="w-4 h-4" />
                              </button>
                            </div>
                          </div>
                        ))}
                      </div>
                    )}
                    <p className="text-[10px] text-text-muted font-medium leading-relaxed italic">
                      {t("romDetail.media.deleteNote")}
                    </p>
                  </div>
                )}

                <div className="pt-6 border-t border-border-default">
                  <h3 className="text-xs font-black text-text-muted uppercase tracking-widest mb-3">{t("romDetail.tabs.files")}</h3>
                  <div className="bg-bg-secondary/50 rounded-xl p-4 space-y-2 text-[11px] font-mono text-text-secondary border border-border-default shadow-inner">
                    <div className="flex justify-between gap-4">
                      <span className="shrink-0 opacity-50 uppercase">{t("romDetail.fields.filename")}:</span>
                      <span className="text-text-primary truncate" title={rom.file}>{rom.file}</span>
                    </div>
                    <div className="flex justify-between gap-4">
                      <span className="shrink-0 opacity-50 uppercase">{t("romDetail.fields.localPath")}:</span>
                      <span className="text-text-primary truncate" title={rom.directory}>{rom.directory}</span>
                    </div>
                  </div>
                </div>
              </div>

              {/* Footer Actions */}
              <div className="p-6 border-t border-border-default bg-bg-secondary/30 shrink-0">
                <div className="flex justify-between items-center px-2">
                  {isEditing ? (
                    <>
                      <button onClick={() => setIsEditing(false)} className="px-6 py-2.5 text-text-secondary hover:text-text-primary font-bold text-sm transition-all">{t("romDetail.actions.cancelEdit")}</button>
                      <button onClick={handleSaveEdit} className="flex items-center gap-2 px-8 py-2.5 bg-accent-primary text-bg-primary rounded-xl font-black text-sm shadow-xl shadow-accent-primary/20 hover:opacity-90 active:scale-95 transition-all">
                        <Check className="w-4 h-4 stroke-[3px]" /> {t("romDetail.actions.saveEdit")}
                      </button>
                    </>
                  ) : (
                    <>
                      <button className="flex-1 flex items-center justify-center gap-2 py-3 bg-accent-primary hover:bg-accent-primary/90 text-text-primary rounded-xl font-bold transition-all shadow-lg shadow-accent-primary/20 active:scale-95">
                        <Play className="w-5 h-5 fill-current" />
                        {t("romDetail.actions.play")}
                      </button>
                      <div className="flex gap-2">
                        <button onClick={() => setIsScrapeDialogOpen(true)} className="p-3 bg-bg-tertiary hover:bg-border-hover text-text-primary rounded-xl font-bold border border-border-default transition-all" title={t("romDetail.actions.scrape")}><Download className="w-5 h-5" /></button>
                        <button onClick={handleStartEdit} className="p-3 bg-bg-tertiary hover:bg-border-hover text-text-primary rounded-xl font-bold border border-border-default transition-all" title={t("romDetail.actions.edit")}><Edit2 className="w-5 h-5" /></button>
                        {rom.system && rom.system.toLowerCase().includes('ps3') && (
                          <button
                            onClick={handleGenerateBoxart}
                            disabled={isGeneratingBoxart}
                            className="p-3 bg-bg-tertiary hover:bg-border-hover text-text-primary rounded-xl font-bold border border-border-default transition-all disabled:opacity-50 disabled:cursor-not-allowed"
                            title="生成 PS3 Boxart"
                          >
                            {isGeneratingBoxart ? <Loader2 className="w-5 h-5 animate-spin" /> : <Wand2 className="w-5 h-5" />}
                          </button>
                        )}
                      </div>
                      <div className="flex gap-3">
                        {isPreview ? (
                          <button onClick={handleExport} disabled={isExporting} className="group relative flex items-center gap-2 px-8 py-2.5 bg-accent-primary text-bg-primary rounded-xl font-black text-sm shadow-xl shadow-accent-primary/20 hover:opacity-90 transition-all active:scale-95 disabled:opacity-50 overflow-hidden">
                            {isExporting ? (
                              <div className="flex items-center gap-2">
                                <Loader2 className="w-4 h-4 animate-spin" />
                                <span>{exportProgress ? t("romDetail.actions.exporting", { progress: exportProgress.current }) : t("romDetail.actions.exportingSimple")}</span>
                              </div>
                            ) : (
                              <>
                                <Save className="w-4 h-4" />
                                <span>{t("romDetail.actions.export")}</span>
                              </>
                            )}
                            {isExporting && exportProgress && (
                              <motion.div initial={{ width: 0 }} animate={{ width: `${exportProgress.current}%` }} className="absolute bottom-0 left-0 h-1 bg-white/30" />
                            )}
                          </button>
                        ) : (
                          <button onClick={onClose} className="flex items-center gap-2 px-8 py-2.5 bg-bg-tertiary text-text-primary rounded-xl font-black text-sm hover:bg-border-hover transition-all active:scale-95">
                            <Check className="w-4 h-4" /> {t("common.finish")}
                          </button>
                        )}
                      </div>
                    </>
                  )}
                </div>
              </div>
            </motion.div>
          </>
        )}
      </AnimatePresence>
      <ScrapeDialog rom={rom} isOpen={isScrapeDialogOpen} onClose={() => setIsScrapeDialogOpen(false)} />
    </>
  );
}

function MediaPreview({ path }: { path: string }) {
  const url = useMediaUrl(path);
  if (!url) return <div className="w-full h-full bg-bg-tertiary animate-pulse" />;
  if (path.toLowerCase().endsWith(".mp4") || path.toLowerCase().endsWith(".webm")) {
    return <video src={url} className="w-full h-full object-cover" muted loop />;
  }
  return <img src={url} alt="" className="w-full h-full object-cover" />;
}

function InfoItem({ icon, label, value }: { icon: React.ReactNode, label: string, value?: string }) {
  const { t } = useTranslation();
  return (
    <div className="flex items-start gap-3">
      <div className="w-8 h-8 rounded-lg bg-bg-tertiary flex items-center justify-center text-accent-primary shrink-0 border border-border-default">{icon}</div>
      <div className="min-w-0">
        <div className="text-[9px] font-bold text-text-muted uppercase tracking-widest leading-none mb-1">{label}</div>
        <div className="text-sm font-bold text-text-primary truncate">{value || t("common.notAvailable")}</div>
      </div>
    </div>
  );
}

function EditItem({ label, value, onChange }: { label: string, value?: string, onChange: (v: string) => void }) {
  return (
    <div className="space-y-1.5">
      <label className="text-[10px] font-black text-text-muted uppercase tracking-widest px-1">{label}</label>
      <input value={value || ""} onChange={e => onChange(e.target.value)} className="w-full bg-bg-secondary/50 border border-border-default rounded-xl px-3 py-2 text-sm text-text-primary outline-none focus:border-accent-primary transition-all font-bold" />
    </div>
  );
}
