import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Check, Trash2, Upload } from "lucide-react";
import { clsx } from "clsx";
import { useAppStore } from "@/stores/appStore";
import { BUILTIN_THEMES, DEFAULT_THEME_ID } from "@/theme/registry";
import type { LoadedTheme, MotionLevel } from "@/theme/types";
import { isTauri, themeApi } from "@/lib/api";
import { Badge, Button, Dialog, IconButton, Tabs, useToast } from "@/components/ui";

/** 主题卡片色板取样的 5 个令牌键 */
const SWATCH_KEYS = [
  "bg-primary",
  "bg-secondary",
  "accent-primary",
  "accent-secondary",
  "text-primary",
] as const;

const errorMessage = (e: unknown): string =>
  e instanceof Error ? e.message : String(e);

export default function AppearanceSection() {
  const { t } = useTranslation();
  const toast = useToast();
  const {
    themeId,
    setTheme,
    motion,
    setMotion,
    customThemes,
    refreshCustomThemes,
  } = useAppStore();
  const [importing, setImporting] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<LoadedTheme | null>(null);
  const [deleting, setDeleting] = useState(false);
  const desktop = isTauri();

  const allThemes = [...BUILTIN_THEMES, ...customThemes];

  const handleImport = async () => {
    if (!desktop || importing) return;
    setImporting(true);
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const path = await open({
        multiple: false,
        filters: [
          {
            name: t("settings.appearance.themePackFilter"),
            extensions: ["rrtheme"],
          },
        ],
      });
      if (typeof path !== "string") return;
      const info = await themeApi.importThemePack(path);
      await refreshCustomThemes();
      let name = "";
      try {
        name = (JSON.parse(info.manifest_json) as { name?: string }).name ?? "";
      } catch {
        // 名称仅用于提示,解析失败可忽略
      }
      toast.success(t("settings.appearance.importSuccess", { name }));
    } catch (e) {
      toast.error(
        t("settings.appearance.importFailed", { error: errorMessage(e) }),
      );
    } finally {
      setImporting(false);
    }
  };

  const handleDelete = async () => {
    if (!deleteTarget || deleting) return;
    setDeleting(true);
    try {
      const deletingCurrent = deleteTarget.id === themeId;
      await themeApi.deleteCustomTheme(deleteTarget.id);
      await refreshCustomThemes();
      // 删除的是当前主题时,显式回退默认主题并持久化
      if (deletingCurrent) setTheme(DEFAULT_THEME_ID);
      toast.success(t("settings.appearance.deleteSuccess"));
      setDeleteTarget(null);
    } catch (e) {
      toast.error(
        t("settings.appearance.deleteFailed", { error: errorMessage(e) }),
      );
    } finally {
      setDeleting(false);
    }
  };

  const motionItems = (["off", "low", "full"] as const).map((level) => ({
    value: level,
    label: t(`settings.appearance.motion.${level}`),
  }));

  return (
    <div className="space-y-8">
      {/* 主题选择 */}
      <section>
        <div className="flex items-start justify-between gap-4 mb-4">
          <div>
            <h2 className="text-lg font-medium text-text-primary">
              {t("settings.appearance.themeTitle")}
            </h2>
            <p className="text-sm text-text-secondary mt-1">
              {t("settings.appearance.themeDescription")}
            </p>
          </div>
          <div className="flex flex-col items-end gap-1 shrink-0">
            <Button
              variant="ghost"
              size="sm"
              onClick={handleImport}
              disabled={!desktop}
              loading={importing}
            >
              <Upload className="w-4 h-4" />
              {t("settings.appearance.importTheme")}
            </Button>
            {!desktop && (
              <span className="text-xs text-text-muted">
                {t("settings.appearance.desktopOnly")}
              </span>
            )}
          </div>
        </div>

        <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3">
          {allThemes.map((tm) => {
            const active = tm.id === themeId;
            return (
              <div
                key={tm.id}
                role="button"
                tabIndex={0}
                onClick={() => setTheme(tm.id)}
                onKeyDown={(e) => {
                  if (e.key === "Enter" || e.key === " ") {
                    e.preventDefault();
                    setTheme(tm.id);
                  }
                }}
                className={clsx(
                  "rr-card flex flex-col gap-3 p-4 cursor-pointer bg-bg-secondary",
                  "rounded-[var(--radius-lg)] border-[length:var(--border-width)]",
                  "transition-all duration-[var(--motion-fast)] ease-[var(--motion-easing)]",
                  "focus-visible:outline-none focus-visible:border-border-highlight",
                  active
                    ? "border-accent-primary ring-1 ring-accent-primary"
                    : "border-border-default hover:border-border-hover",
                )}
              >
                {/* 5 色色板:主题数据展示,内联 style 使用 manifest 令牌值 */}
                <div className="flex h-6 overflow-hidden rounded-[var(--radius-sm)] border-[length:var(--border-width)] border-border-default">
                  {SWATCH_KEYS.map((key) => (
                    <span
                      key={key}
                      className="flex-1"
                      style={{ backgroundColor: tm.tokens[key] }}
                      title={key}
                    />
                  ))}
                </div>
                <div className="flex items-center justify-between gap-2 min-h-8">
                  <span
                    className={clsx(
                      "text-sm font-medium truncate",
                      active ? "text-text-primary" : "text-text-secondary",
                    )}
                  >
                    {tm.name}
                  </span>
                  <span className="flex items-center gap-1 shrink-0">
                    {!tm.builtin && (
                      <Badge>{t("settings.appearance.customBadge")}</Badge>
                    )}
                    {active && <Check className="w-4 h-4 text-accent-primary" />}
                    {!tm.builtin && (
                      <IconButton
                        size="sm"
                        variant="danger"
                        aria-label={t("common.delete")}
                        onClick={(e) => {
                          e.stopPropagation();
                          setDeleteTarget(tm);
                        }}
                      >
                        <Trash2 className="w-4 h-4" />
                      </IconButton>
                    )}
                  </span>
                </div>
              </div>
            );
          })}
        </div>
      </section>

      {/* 动效开关 */}
      <section>
        <h2 className="text-lg font-medium text-text-primary">
          {t("settings.appearance.motionTitle")}
        </h2>
        <p className="text-sm text-text-secondary mt-1 mb-4">
          {t("settings.appearance.motionDescription")}
        </p>
        <Tabs
          items={motionItems}
          value={motion}
          onChange={(v) => setMotion(v as MotionLevel)}
        />
      </section>

      {/* 删除确认 */}
      <Dialog
        open={!!deleteTarget}
        onClose={() => setDeleteTarget(null)}
        title={t("settings.appearance.deleteTitle")}
        footer={
          <>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => setDeleteTarget(null)}
            >
              {t("common.cancel")}
            </Button>
            <Button
              variant="danger"
              size="sm"
              loading={deleting}
              onClick={handleDelete}
            >
              {t("common.delete")}
            </Button>
          </>
        }
      >
        <p className="text-sm text-text-secondary">
          {t("settings.appearance.deleteConfirm", {
            name: deleteTarget?.name ?? "",
          })}
        </p>
      </Dialog>
    </div>
  );
}
