import { useState, useEffect } from "react";
import {
  X, Calendar, User, Building2, Globe, Gamepad2, Star, Languages,
  Eye, EyeOff, Save, Download, Edit2, Trash2, LayoutGrid, Info, Check, Wand2
} from "lucide-react";
import { resolveMediaUrlAsync, scraperApi, ps3Api, aiTranslationApi } from "@/lib/api";
import { romMetadata } from "@/lib/metadataTranslation";
import type { Rom } from "@/types";
import { useRomStore } from "@/stores/romStore";
import { useTranslation } from "react-i18next";
import ScrapeDialog from "./ScrapeDialog";
import { Button, IconButton, Input, EmptyState, Spinner, Tabs, useToast } from "@/components/ui";
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

export default function RomDetail({ rom: selectedRom, onClose }: RomDetailProps) {
  const { t } = useTranslation();
  const toast = useToast();
  const { exportData, updateTempMetadata, deleteTempMedia, isExporting, exportProgress } = useRomStore();
  const rom = useRomStore(state => {
    if (!selectedRom) return null;
    return state.systemRoms
      .flatMap(system => system.roms)
      .find(item =>
        item.file === selectedRom.file &&
        item.system === selectedRom.system &&
        item.directory === selectedRom.directory
      ) ?? selectedRom;
  });
  const [isScrapeDialogOpen, setIsScrapeDialogOpen] = useState(false);
  const [isPreview, setIsPreview] = useState(false);
  const [isEditing, setIsEditing] = useState(false);
  const [editForm, setEditForm] = useState<Partial<Rom & { _activeTab: string }>>({});
  const [tempMedia, setTempMedia] = useState<{ asset_type: string, path: string }[]>([]);
  const [isGeneratingBoxart, setIsGeneratingBoxart] = useState(false);
  const [isTranslating, setIsTranslating] = useState(false);

  const open = !!rom;
  const [entered, setEntered] = useState(false);
  useEffect(() => {
    if (!open) {
      setEntered(false);
      return;
    }
    let inner = 0;
    const outer = requestAnimationFrame(() => {
      inner = requestAnimationFrame(() => setEntered(true));
    });
    return () => {
      cancelAnimationFrame(outer);
      cancelAnimationFrame(inner);
    };
  }, [open]);

  useEffect(() => {
    if (rom) {
      setIsPreview(rom.has_temp_metadata);
      setIsEditing(false);
      if (rom.has_temp_metadata) {
        scraperApi.getTempMediaList(rom.system, rom.file, rom.directory).then(setTempMedia);
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
      const newList = await scraperApi.getTempMediaList(rom.system, rom.file, rom.directory);
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
        // 刷新 tempMedia 列表
        const newTempMedia = await scraperApi.getTempMediaList(rom.system, rom.file, rom.directory);
        setTempMedia(newTempMedia);
        setIsPreview(true);

        // 刷新 ROM store 以更新库视图
        const { fetchRoms } = useRomStore.getState();
        await fetchRoms();
      } else {
        toast.error(t("romDetail.actions.boxartFailed", { error: result.error }));
      }
    } catch (error) {
      console.error("Generate boxart failed:", error);
      toast.error(t("romDetail.actions.boxartFailed", { error: String(error) }));
    } finally {
      setIsGeneratingBoxart(false);
    }
  };

  const handleTranslate = async () => {
    if (!rom) return;
    setIsTranslating(true);
    try {
      const translated = await aiTranslationApi.translateMetadata({
        system: rom.system,
        file_name: rom.file,
        metadata: romMetadata(displayData || rom),
      });
      await updateTempMetadata(rom.system, rom.directory, rom.file, translated);
      setIsPreview(true);
      toast.success(t("romDetail.translation.success"));
    } catch (error) {
      toast.error(t("romDetail.translation.failed", { error: String(error) }));
    } finally {
      setIsTranslating(false);
    }
  };

  if (!rom) return null;

  const activeTab = editForm._activeTab || "info";
  const heroLoading =
    (!!currentData?.video && !videoUrl) || (!currentData?.video && !!heroSource && !heroUrl);

  return (
    <>
      <div
        onClick={onClose}
        className={clsx(
          "fixed inset-0 bg-bg-primary/60 backdrop-blur-sm z-40",
          "transition-opacity duration-[var(--motion-normal)] ease-[var(--motion-easing)]",
          entered ? "opacity-100" : "opacity-0"
        )}
      />

      <div
        className={clsx(
          "fixed right-0 top-0 bottom-0 w-full max-w-lg bg-bg-primary z-50 overflow-y-auto flex flex-col",
          "border-l-[length:var(--border-width)] border-border-default [box-shadow:var(--shadow-dialog)]",
          "transition-transform duration-[var(--motion-normal)] ease-[var(--motion-easing)]",
          entered ? "translate-x-0" : "translate-x-full"
        )}
      >
        {/* Header Image/Video */}
        <div className="relative aspect-video w-full bg-bg-secondary overflow-hidden shrink-0">
          {videoUrl ? (
            <video key={videoUrl} src={videoUrl} autoPlay muted loop className="w-full h-full object-cover" />
          ) : heroUrl ? (
            <img key={heroUrl} src={heroUrl} alt="" className="w-full h-full object-cover" />
          ) : heroLoading ? (
            <div className="absolute inset-0 flex items-center justify-center text-text-muted">
              <Spinner size={32} />
            </div>
          ) : (
            <EmptyState
              icon={<Gamepad2 />}
              title={t("romDetail.cover.noImage")}
              className="absolute inset-4 gap-2 py-4"
            />
          )}

          <div className="absolute top-4 right-4 left-4 flex justify-between items-center z-10">
            <IconButton
              size="sm"
              aria-label={t("common.cancel")}
              onClick={onClose}
              className="backdrop-blur-md"
            >
              <X className="w-5 h-5" />
            </IconButton>

            <div className="flex gap-2">
              {rom.has_temp_metadata && !isEditing && (
                <Button
                  size="sm"
                  variant={isPreview ? "primary" : "ghost"}
                  onClick={() => setIsPreview(!isPreview)}
                  className="backdrop-blur-md"
                >
                  {isPreview ? <Eye className="w-3.5 h-3.5" /> : <EyeOff className="w-3.5 h-3.5" />}
                  {isPreview ? t("romDetail.preview.mode") : t("romDetail.preview.raw")}
                </Button>
              )}
            </div>
          </div>

          <div className="absolute bottom-0 left-0 right-0 p-6 bg-gradient-to-t from-bg-primary via-bg-primary/40 to-transparent">
            {logoUrl ? (
              <img src={logoUrl} alt={currentData?.name} className="h-12 mb-2 object-contain" />
            ) : (
              <div className="mb-2">
                {isEditing ? (
                  <Input
                    value={editForm.name || ""}
                    onChange={e => setEditForm({ ...editForm, name: e.target.value })}
                    className="font-bold"
                  />
                ) : (
                  <h2 className="text-3xl font-bold text-text-primary leading-tight">
                    {currentData?.name}
                  </h2>
                )}
              </div>
            )}
            <div className="flex items-center gap-3 text-sm">
              <span className="px-2 py-0.5 rounded-[var(--radius-sm)] bg-bg-tertiary text-text-primary font-medium uppercase text-xs border-[length:var(--border-width)] border-border-default">
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

        <div className="px-4 py-3 border-b-[length:var(--border-width)] border-border-default bg-bg-primary shrink-0">
          <Tabs
            items={[
              { value: "info", label: t("romDetail.tabs.info") },
              { value: "media", label: t("romDetail.tabs.media") },
            ]}
            value={activeTab}
            onChange={(tab) => setEditForm({ ...editForm, _activeTab: tab })}
          />
        </div>

        {/* Action Buttons */}
        <div className="p-4 border-b-[length:var(--border-width)] border-border-default bg-bg-secondary/30">
          <div className="flex justify-between items-center gap-3">
            {isEditing ? (
              <>
                <Button variant="ghost" onClick={() => setIsEditing(false)}>
                  {t("romDetail.actions.cancelEdit")}
                </Button>
                <Button onClick={handleSaveEdit}>
                  <Check className="w-4 h-4" /> {t("romDetail.actions.saveEdit")}
                </Button>
              </>
            ) : (
              <>
                <div className="flex gap-2">
                  <IconButton
                    onClick={() => setIsScrapeDialogOpen(true)}
                    title={t("romDetail.actions.scrape")}
                    aria-label={t("romDetail.actions.scrape")}
                  >
                    <Download className="w-4 h-4" />
                  </IconButton>
                  <IconButton
                    onClick={handleStartEdit}
                    title={t("romDetail.actions.edit")}
                    aria-label={t("romDetail.actions.edit")}
                  >
                    <Edit2 className="w-4 h-4" />
                  </IconButton>
                  <IconButton
                    onClick={handleTranslate}
                    disabled={isTranslating}
                    title={t("romDetail.actions.translate")}
                    aria-label={t("romDetail.actions.translate")}
                  >
                    {isTranslating ? <Spinner size={16} /> : <Languages className="w-4 h-4" />}
                  </IconButton>
                  {rom.system && rom.system.toLowerCase().includes('ps3') && (
                    <IconButton
                      onClick={handleGenerateBoxart}
                      disabled={isGeneratingBoxart}
                      title={t("romDetail.actions.generateBoxart")}
                      aria-label={t("romDetail.actions.generateBoxart")}
                    >
                      {isGeneratingBoxart ? <Spinner size={16} /> : <Wand2 className="w-4 h-4" />}
                    </IconButton>
                  )}
                  {isPreview ? (
                    <Button
                      onClick={handleExport}
                      loading={isExporting}
                      className="relative overflow-hidden"
                    >
                      {isExporting ? (
                        <span className="hidden sm:inline">
                          {exportProgress
                            ? t("romDetail.actions.exporting", { progress: exportProgress.current })
                            : t("romDetail.actions.exportingSimple")}
                        </span>
                      ) : (
                        <>
                          <Save className="w-3.5 h-3.5" />
                          <span className="hidden sm:inline">{t("romDetail.actions.export")}</span>
                        </>
                      )}
                      {isExporting && exportProgress && (
                        <span
                          className="absolute bottom-0 left-0 h-1 bg-text-primary/30 transition-[width] duration-[var(--motion-normal)] ease-[var(--motion-easing)]"
                          style={{ width: `${exportProgress.current}%` }}
                        />
                      )}
                    </Button>
                  ) : (
                    <Button variant="ghost" onClick={onClose}>
                      <Check className="w-3.5 h-3.5" />
                      <span className="hidden sm:inline">{t("common.finish")}</span>
                    </Button>
                  )}
                </div>
              </>
            )}
          </div>
        </div>

        <div className="flex-1 p-6 space-y-8 overflow-y-auto custom-scrollbar">
          {activeTab === "info" ? (
            <>
              {isPreview && !isEditing && (
                <div className="p-4 bg-accent-primary/5 border-[length:var(--border-width)] border-accent-primary/20 rounded-[var(--radius-lg)] flex items-start gap-4">
                  <div className="w-10 h-10 rounded-[var(--radius-md)] bg-accent-primary/10 flex items-center justify-center shrink-0">
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
                    className="w-full bg-bg-primary p-3 text-sm text-text-primary placeholder:text-text-muted rounded-[var(--radius-md)] border-[length:var(--border-width)] border-border-default hover:border-border-hover focus:border-border-highlight focus-visible:outline-none transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)] resize-none font-medium leading-relaxed"
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
                <EmptyState icon={<Gamepad2 />} title={t("romDetail.media.empty")} />
              ) : (
                <div className="grid grid-cols-2 gap-4">
                  {tempMedia.map(m => (
                    <div key={m.asset_type} className="group relative aspect-video bg-bg-secondary rounded-[var(--radius-md)] border-[length:var(--border-width)] border-border-default overflow-hidden">
                      <MediaPreview path={m.path} />
                      <div className="absolute inset-0 bg-bg-primary/70 opacity-0 group-hover:opacity-100 transition-opacity duration-[var(--motion-fast)] ease-[var(--motion-easing)] flex flex-col items-center justify-center p-2">
                        <span className="text-[10px] font-black text-bg-primary uppercase tracking-widest mb-2 bg-accent-primary px-2 py-0.5 rounded-[var(--radius-sm)]">
                          {m.asset_type}
                        </span>
                        <IconButton
                          size="sm"
                          variant="danger"
                          onClick={() => handleDeleteMedia(m.asset_type)}
                          title={t("common.delete")}
                          aria-label={t("common.delete")}
                        >
                          <Trash2 className="w-4 h-4" />
                        </IconButton>
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

          <div className="pt-6 border-t-[length:var(--border-width)] border-border-default">
            <h3 className="text-xs font-black text-text-muted uppercase tracking-widest mb-3">{t("romDetail.tabs.files")}</h3>
            <div className="bg-bg-secondary/50 rounded-[var(--radius-md)] p-4 space-y-2 text-[11px] font-mono text-text-secondary border-[length:var(--border-width)] border-border-default">
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
      </div>
      <ScrapeDialog rom={rom} isOpen={isScrapeDialogOpen} onClose={() => setIsScrapeDialogOpen(false)} />
    </>
  );
}

function MediaPreview({ path }: { path: string }) {
  const url = useMediaUrl(path);
  if (!url) {
    return (
      <div className="w-full h-full bg-bg-tertiary flex items-center justify-center text-text-muted">
        <Spinner size={20} />
      </div>
    );
  }
  if (path.toLowerCase().endsWith(".mp4") || path.toLowerCase().endsWith(".webm")) {
    return <video src={url} className="w-full h-full object-cover" muted loop />;
  }
  return <img src={url} alt="" className="w-full h-full object-cover" />;
}

function InfoItem({ icon, label, value }: { icon: React.ReactNode, label: string, value?: string }) {
  const { t } = useTranslation();
  return (
    <div className="flex items-start gap-3">
      <div className="w-8 h-8 rounded-[var(--radius-sm)] bg-bg-tertiary flex items-center justify-center text-accent-primary shrink-0 border-[length:var(--border-width)] border-border-default">{icon}</div>
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
      <Input value={value || ""} onChange={e => onChange(e.target.value)} className="font-bold" />
    </div>
  );
}
