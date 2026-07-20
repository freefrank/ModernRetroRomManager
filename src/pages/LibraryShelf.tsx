import { useCallback, useEffect, useMemo, useState } from "react";
import { useNavigate } from "react-router-dom";
import { Database, Eye, EyeOff, Ghost, Plus, Save, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useRomStore, normalizeDirKey } from "@/stores/romStore";
import { Button, Card, Dialog, EmptyState, IconButton, toast } from "@/components/ui";
import SystemCard from "@/components/rom/SystemCard";
import SystemContextMenu from "@/components/rom/SystemContextMenu";
import BatchScrapeDialog from "@/components/rom/BatchScrapeDialog";
import { api, isTauri } from "@/lib/api";
import type { GameSystem } from "@/types";

/** 系统货架卡片的最小信息(取自 romStore.availableSystems) */
interface ShelfSystem {
  name: string;
  path: string;
  romCount: number;
}

const norm = (s: string | undefined) => (s ?? "").trim().toLowerCase();

/** 用 ROM 目录名优先匹配完整目录映射，再回退预置系统。 */
function findSystemLogo(
  systems: GameSystem[],
  folderName: string,
  mappedLogos: Record<string, string>
): string | undefined {
  const target = norm(folderName);
  if (!target) return undefined;
  if (mappedLogos[target]) return mappedLogos[target];
  const hit = systems.find(
    (s) => norm(s.id) === target || norm(s.shortName) === target || norm(s.name) === target
  );
  return hit?.logo;
}

/** 系统货架:每个系统一张卡片,点击进入单系统库页 */
export default function LibraryShelf() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { availableSystems, stats, systems, fetchSystems, scanDirectories, exportData, scanLibrary, isExporting } = useRomStore();
  const [isBatchDialogOpen, setIsBatchDialogOpen] = useState(false);
  const [isSavingLibrary, setIsSavingLibrary] = useState(false);
  const [mappedLogos, setMappedLogos] = useState<Record<string, string>>({});
  // 被隐藏系统的规范化目录路径集合
  const [hiddenSystemPaths, setHiddenSystemPaths] = useState<Set<string>>(new Set());
  const [showHidden, setShowHidden] = useState(false);
  const [contextMenu, setContextMenu] = useState<{ sys: ShelfSystem; x: number; y: number } | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<ShelfSystem | null>(null);
  const activeLibrary = scanDirectories.find(item => item.isActive);

  const refreshHiddenSystems = async () => {
    try {
      const paths = await api.getHiddenSystems();
      setHiddenSystemPaths(new Set(paths.map(normalizeDirKey)));
    } catch (error) {
      console.error("Failed to load hidden systems:", error);
    }
  };

  // 把整个库中所有含待保存抓取数据的平台逐个原地写回 ROM 目录
  // (整库导出命令不允许原地写回,因此逐平台走单系统保存路径)。
  // 写回前先做多碟物理整理:各碟塞进 <基名>/ 子文件夹、外层写 <基名>.m3u,
  // 并把临时元数据里的各碟折叠成一条指向 m3u 的记录(命令对非光盘/无多碟为空操作)。
  const handleSaveLibrary = async () => {
    setIsSavingLibrary(true);
    try {
      const sourceSystems = await api.getRoms();
      const pending = sourceSystems.filter(
        sys => sys.path && sys.roms.some(rom => rom.has_temp_metadata),
      );
      if (pending.length === 0) {
        toast.info(t("library.shelf.saveNoPending"));
        return;
      }
      let organizedDiscGames = 0;
      const invoke = isTauri()
        ? (await import("@tauri-apps/api/core")).invoke
        : null;
      for (const sys of pending) {
        if (invoke) {
          try {
            const report = await invoke<{ changed: boolean; groups: number }>(
              "organize_multidisc_games",
              { directory: sys.path, system: sys.system },
            );
            if (report?.changed) organizedDiscGames += report.groups;
          } catch (error) {
            // 整理失败不应阻断保存:记录并继续原地写回。
            console.error(`多碟整理失败 [${sys.system}]:`, error);
          }
        }
        await exportData(sys.system, sys.path, "both", sys.path, "original");
      }
      toast.success(t("library.shelf.saveSuccess", { count: pending.length }));
      if (organizedDiscGames > 0) {
        toast.info(t("library.actions.multidiscOrganized", { count: organizedDiscGames }));
        // 折叠改动了临时元数据:全量重扫,库视图各多碟游戏收成一条。
        await scanLibrary(true);
      }
    } catch (error) {
      toast.error(t("library.actions.saveFailed", { error: String(error) }));
    } finally {
      setIsSavingLibrary(false);
    }
  };

  // 装载预置系统列表以解析各系统 logo
  useEffect(() => {
    if (systems.length === 0) {
      fetchSystems();
    }
  }, [systems.length, fetchSystems]);

  useEffect(() => {
    let mounted = true;
    api.getSystemLogoMap()
      .then((logos) => {
        if (mounted) setMappedLogos(logos);
      })
      .catch((error) => {
        console.error("Failed to load system logo map:", error);
      });
    return () => {
      mounted = false;
    };
  }, []);

  // 启动加载被隐藏系统清单
  useEffect(() => {
    void refreshHiddenSystems();
  }, []);

  const logoByName = useMemo(() => {
    const map = new Map<string, string | undefined>();
    for (const sys of availableSystems) {
      map.set(sys.name, findSystemLogo(systems, sys.name, mappedLogos));
    }
    return map;
  }, [availableSystems, mappedLogos, systems]);

  const isSystemHidden = useCallback(
    (sys: ShelfSystem) => hiddenSystemPaths.has(normalizeDirKey(sys.path)),
    [hiddenSystemPaths],
  );

  // 当前库中处于隐藏态的系统数量(仅统计实际存在的系统)
  const hiddenCount = useMemo(
    () => availableSystems.filter(isSystemHidden).length,
    [availableSystems, isSystemHidden],
  );

  // 依据隐藏集与「显示隐藏系统」开关过滤货架
  const visibleSystems = useMemo(
    () => (showHidden ? availableSystems : availableSystems.filter((sys) => !isSystemHidden(sys))),
    [availableSystems, isSystemHidden, showHidden],
  );

  const handleToggleHidden = async (sys: ShelfSystem) => {
    const hidden = isSystemHidden(sys);
    try {
      await api.setSystemHidden(sys.path, !hidden);
      await refreshHiddenSystems();
    } catch (error) {
      toast.error(t("library.shelf.context.hideFailed", { error: String(error) }));
    }
  };

  const handleDeleteSystem = async () => {
    if (!deleteTarget) return;
    const target = deleteTarget;
    setDeleteTarget(null);
    try {
      await api.deleteSystemDirectory(target.path);
      toast.success(t("library.shelf.context.deleteSuccess", { name: target.name }));
      await Promise.all([scanLibrary(true), refreshHiddenSystems()]);
    } catch (error) {
      toast.error(t("library.shelf.context.deleteFailed", { error: String(error) }));
    }
  };

  return (
    <div className="rr-page flex flex-col h-full max-w-[1600px] mx-auto w-full">
      {/* Header */}
      <header className="py-4 flex flex-wrap items-center justify-between gap-4">
        <div>
          <h1 className="text-4xl font-bold tracking-tight text-text-primary mb-2">
            {activeLibrary?.name ?? t("library.title")}
          </h1>
          <p className="text-text-secondary font-medium">
            {t("library.shelf.systemCount", { count: availableSystems.length })}
            {" · "}
            {t("library.gameCount", { count: stats.totalRoms })}
          </p>
        </div>
        <div className="flex items-center gap-3">
          {hiddenCount > 0 && (
            <IconButton
              size="sm"
              variant={showHidden ? "primary" : "ghost"}
              onClick={() => setShowHidden((value) => !value)}
              title={t("library.shelf.context.showHidden", { count: hiddenCount })}
              aria-label={t("library.shelf.context.showHidden", { count: hiddenCount })}
            >
              {showHidden ? <Eye className="w-5 h-5" /> : <EyeOff className="w-5 h-5" />}
            </IconButton>
          )}
          <Button onClick={() => setIsBatchDialogOpen(true)} disabled={stats.totalRoms === 0}>
            <Database className="w-4 h-4" />
            {t("library.batch.scrapeLibrary")}
          </Button>
          <Button
            variant="ghost"
            onClick={() => void handleSaveLibrary()}
            disabled={stats.totalRoms === 0 || isExporting || isSavingLibrary}
            loading={isSavingLibrary}
            title={t("library.actions.saveHint")}
          >
            {!isSavingLibrary && <Save className="w-4 h-4" />}
            <span>{t("library.actions.save")}</span>
          </Button>
        </div>
      </header>

      {availableSystems.length === 0 ? (
        /* 空态:库为空,引导去设置添加扫描目录 */
        <div className="flex-1 flex items-center justify-center">
          <EmptyState
            className="w-full max-w-md"
            icon={<Ghost />}
            title={t("library.empty.title")}
            description={t("library.empty.description")}
            action={
              <Button onClick={() => navigate("/settings?tab=general&section=libraries&addDirectory=1")}>
                <Plus className="w-4 h-4" />
                {t("library.empty.addDirectory")}
              </Button>
            }
          />
        </div>
      ) : (
        <div className="flex-1 min-h-0 overflow-y-auto custom-scrollbar pb-6">
          <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
            {visibleSystems.map((sys) => (
              <SystemCard
                key={sys.name}
                name={sys.name}
                romCount={sys.romCount}
                logoFile={logoByName.get(sys.name)}
                isHidden={isSystemHidden(sys)}
                onClick={() => navigate(`/library/${encodeURIComponent(sys.name)}`)}
                onContextMenu={(event) => {
                  event.preventDefault();
                  setContextMenu({
                    sys: { name: sys.name, path: sys.path, romCount: sys.romCount },
                    x: event.clientX,
                    y: event.clientY,
                  });
                }}
              />
            ))}

            {/* 「+ 添加目录」卡:直接打开添加目录界面 */}
            <Card
              role="button"
              tabIndex={0}
              onClick={() => navigate("/settings?tab=general&section=libraries&addDirectory=1")}
              onKeyDown={(e) => {
                if (e.key === "Enter" || e.key === " ") {
                  e.preventDefault();
                  navigate("/settings?tab=general&section=libraries&addDirectory=1");
                }
              }}
              className="cursor-pointer select-none border-dashed bg-transparent p-6 flex flex-col items-center justify-center gap-2 text-text-muted hover:text-text-primary hover:border-border-hover"
            >
              <Plus className="w-8 h-8" />
              <span className="text-sm font-medium">{t("library.shelf.addCard")}</span>
              <span className="text-xs">{t("library.shelf.addCardHint")}</span>
            </Card>
          </div>
        </div>
      )}
      <BatchScrapeDialog
        isOpen={isBatchDialogOpen}
        onClose={() => setIsBatchDialogOpen(false)}
        scope="library"
      />

      {/* 系统卡片右键菜单 */}
      {contextMenu && (
        <SystemContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          isHidden={isSystemHidden(contextMenu.sys)}
          onToggleHidden={() => void handleToggleHidden(contextMenu.sys)}
          onDelete={() => setDeleteTarget(contextMenu.sys)}
          onClose={() => setContextMenu(null)}
        />
      )}

      {/* 删除系统确认(永久删除整个平台目录,不可恢复) */}
      <Dialog
        open={Boolean(deleteTarget)}
        onClose={() => setDeleteTarget(null)}
        title={
          <span className="flex items-center gap-3 text-accent-error">
            <Trash2 className="w-5 h-5 shrink-0" />
            {t("library.shelf.context.deleteConfirmTitle")}
          </span>
        }
        footer={
          <>
            <Button variant="ghost" onClick={() => setDeleteTarget(null)}>
              {t("common.cancel")}
            </Button>
            <Button variant="danger" onClick={() => void handleDeleteSystem()}>
              <Trash2 className="w-4 h-4" />
              {t("library.shelf.context.delete")}
            </Button>
          </>
        }
      >
        <p className="text-sm text-text-secondary leading-relaxed">
          {t("library.shelf.context.deleteConfirmMessage", {
            name: deleteTarget ? deleteTarget.name : "",
          })}
        </p>
      </Dialog>
    </div>
  );
}
