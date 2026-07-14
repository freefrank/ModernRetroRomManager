import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import {
  Activity,
  Globe,
  GripVertical,
  Info,
  Key,
  Save,
  Settings2,
  Shield,
} from "lucide-react";
import { clsx } from "clsx";
import { useScraperStore } from "@/stores/scraperStore";
import { scraperApi } from "@/lib/api";
import type { ScraperCredentials, ScraperProviderInfo } from "@/types";
import { Badge, Button, IconButton, Input, toast } from "@/components/ui";

const getProviderIcon = (id: string) => {
  switch (id) {
    case "screenscraper":
      return <Globe className="w-5 h-5" />;
    case "steamgriddb":
      return <Activity className="w-5 h-5" />;
    default:
      return <Settings2 className="w-5 h-5" />;
  }
};

const getProviderColor = (id: string) => {
  switch (id) {
    case "screenscraper":
      return "bg-accent-error/20 text-accent-error border-accent-error/30";
    case "steamgriddb":
      return "bg-accent-primary/20 text-accent-primary border-accent-primary/30";
    default:
      return "bg-bg-tertiary text-text-primary border-border-default";
  }
};

export default function ScraperSection() {
  const { t } = useTranslation();
  const {
    providers,
    fetchProviders,
    configureProvider,
    setProviderEnabled,
    setProviderPriority,
  } = useScraperStore();
  const [editingProvider, setEditingProvider] = useState<string | null>(null);
  const [credentials, setCredentials] = useState<ScraperCredentials>({});
  const [mediaTypes, setMediaTypes] = useState<string[]>([]);
  const [testingProvider, setTestingProvider] = useState<string | null>(null);
  const mediaTypeOptions = [
    "boxfront", "boxback", "box3d", "logo", "screenshot", "titlescreen",
    "hero", "banner", "icon", "video", "manual",
  ];

  // 能力枚举 → 中文标签(未知值回退原文)
  const formatCapabilities = (capabilities: string[]) =>
    capabilities
      .map((cap) => t(`scraper.capabilities.${cap}`, { defaultValue: cap }))
      .join(" · ");

  // 拖拽排序状态(改用 mouse 事件)
  const [draggedProvider, setDraggedProvider] = useState<string | null>(null);
  const [dragOverProvider, setDragOverProvider] = useState<string | null>(null);
  const [isDragging, setIsDragging] = useState(false);
  const [dragOffset, setDragOffset] = useState({ x: 0, y: 0 });
  const [mousePos, setMousePos] = useState({ x: 0, y: 0 });

  useEffect(() => {
    fetchProviders();
    scraperApi.getMediaTypes().then(setMediaTypes).catch(console.error);
  }, [fetchProviders]);

  const handleMediaTypeChange = async (type: string, enabled: boolean) => {
    const next = enabled
      ? Array.from(new Set([...mediaTypes, type]))
      : mediaTypes.filter((value) => value !== type);
    setMediaTypes(next);
    try {
      await scraperApi.setMediaTypes(next);
    } catch (error) {
      console.error("Failed to save media types:", error);
      setMediaTypes(await scraperApi.getMediaTypes());
    }
  };

  const handleMouseMove = (e: MouseEvent) => {
    if (isDragging) {
      setMousePos({ x: e.clientX, y: e.clientY });
    }
  };

  const handleMouseUp = async () => {
    if (!isDragging || !draggedProvider || !dragOverProvider) {
      setDraggedProvider(null);
      setDragOverProvider(null);
      setIsDragging(false);
      return;
    }

    // 获取排序后的 provider 列表
    const sorted = [...providers].sort((a, b) => a.priority - b.priority);
    const draggedIndex = sorted.findIndex((p) => p.id === draggedProvider);
    const targetIndex = sorted.findIndex((p) => p.id === dragOverProvider);

    if (draggedIndex === -1 || targetIndex === -1) {
      setDraggedProvider(null);
      setDragOverProvider(null);
      setIsDragging(false);
      return;
    }

    // 重新排列
    const newProviders = [...sorted];
    const [removed] = newProviders.splice(draggedIndex, 1);
    newProviders.splice(targetIndex, 0, removed);

    // 重新计算优先级(从10开始,每个间隔10)
    try {
      for (let i = 0; i < newProviders.length; i++) {
        const newPriority = (i + 1) * 10;
        if (newProviders[i].priority !== newPriority) {
          await setProviderPriority(newProviders[i].id, newPriority);
        }
      }
    } catch (error) {
      console.error("Failed to update provider priorities:", error);
    }

    setDraggedProvider(null);
    setDragOverProvider(null);
    setIsDragging(false);
  };

  // 全局 mouseUp 和 mouseMove 监听器
  useEffect(() => {
    if (isDragging) {
      window.addEventListener("mouseup", handleMouseUp);
      window.addEventListener("mousemove", handleMouseMove);
      return () => {
        window.removeEventListener("mouseup", handleMouseUp);
        window.removeEventListener("mousemove", handleMouseMove);
      };
    }
    // handleMouseMove/handleMouseUp 未做 useCallback 记忆化,若加入依赖数组会导致
    // 拖拽过程中每次渲染都重新订阅/取消订阅监听器,属于行为改动,故保留原有依赖数组。
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isDragging, draggedProvider, dragOverProvider]);

  const handleToggleEnabled = async (providerId: string, enabled: boolean) => {
    try {
      await setProviderEnabled(providerId, enabled);
    } catch (error) {
      console.error("Failed to toggle provider:", error);
    }
  };

  const handleEditConfig = (provider: ScraperProviderInfo) => {
    setEditingProvider(provider.id);
    setCredentials({
      username: provider.configured_username,
      rate_limit: provider.rate_limit,
      threads: provider.threads,
    });
  };

  const handleTestProvider = async (providerId: string) => {
    setTestingProvider(providerId);
    try {
      toast.success(await scraperApi.testProvider(providerId));
    } catch (error) {
      toast.error(t("settings.apiConfig.testFailed", { error: String(error) }));
    } finally {
      setTestingProvider(null);
    }
  };

  const handleSaveConfig = async () => {
    if (!editingProvider) return;
    try {
      await configureProvider(editingProvider, credentials);
      setEditingProvider(null);
    } catch (error) {
      console.error("Failed to save credentials:", error);
    }
  };

  // 拖拽处理函数(改用 mouse 事件)
  const handleMouseDown = (e: React.MouseEvent, providerId: string) => {
    // 如果点击的是按钮或开关,不启动拖拽
    const target = e.target as HTMLElement;
    if (target.closest("button") || target.closest("label")) {
      return;
    }

    // 记录鼠标相对于元素的偏移
    const rect = e.currentTarget.getBoundingClientRect();
    setDragOffset({
      x: e.clientX - rect.left,
      y: e.clientY - rect.top,
    });
    setMousePos({ x: e.clientX, y: e.clientY });

    setDraggedProvider(providerId);
    setIsDragging(true);
  };

  const handleMouseEnter = (providerId: string) => {
    if (isDragging && draggedProvider && draggedProvider !== providerId) {
      setDragOverProvider(providerId);
    }
  };

  // 按优先级排序的 providers
  const sortedProviders = [...providers].sort((a, b) => a.priority - b.priority);

  return (
    <div className="space-y-8">
      <section>
        <h2 className="text-lg font-medium text-text-primary mb-4 flex items-center gap-2">
          <Shield className="w-5 h-5 text-accent-primary" />
          {t("settings.apiConfig.title")}
        </h2>

        <div className="grid grid-cols-1 gap-4">
          {sortedProviders.map((p) => (
            <div
              key={p.id}
              onMouseDown={(e) => handleMouseDown(e, p.id)}
              onMouseEnter={() => handleMouseEnter(p.id)}
              className={clsx(
                "group relative overflow-hidden rounded-[var(--radius-lg)] border-[length:var(--border-width)] transition-all duration-[var(--motion-normal)] ease-[var(--motion-easing)] cursor-move select-none",
                p.enabled
                  ? "bg-bg-secondary border-border-hover"
                  : "bg-bg-primary/50 border-border-default opacity-70",
                draggedProvider === p.id && "opacity-50 scale-95",
                dragOverProvider === p.id && "ring-2 ring-accent-primary",
              )}
            >
              <div className="flex items-center p-5">
                <div className="cursor-grab active:cursor-grabbing text-text-muted hover:text-accent-primary transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)] mr-3">
                  <GripVertical className="w-5 h-5" />
                </div>
                <div
                  className={clsx(
                    "w-12 h-12 rounded-[var(--radius-md)] flex items-center justify-center border-[length:var(--border-width)]",
                    getProviderColor(p.id),
                  )}
                >
                  {getProviderIcon(p.id)}
                </div>

                <div className="ml-4 flex-1">
                  <div className="flex items-center gap-2">
                    <h3 className="font-bold text-text-primary text-lg">
                      {p.name}
                    </h3>
                    {p.has_credentials ? (
                      <Badge variant="success" className="uppercase">
                        {t("settings.apiConfig.authenticated")}
                      </Badge>
                    ) : (
                      p.requires_credentials && (
                        <Badge variant="warning">
                          {t("scraper.status.missingCredentials")}
                        </Badge>
                      )
                    )}
                  </div>
                  <div className="flex items-center gap-3 mt-1">
                    <span className="text-xs text-text-muted flex items-center gap-1">
                      <Activity className="w-3 h-3" />
                      {formatCapabilities(p.capabilities)}
                    </span>
                    <span className="text-xs text-text-muted">
                      {t("settings.apiConfig.stats", {
                        matched: p.matched_count,
                        selected: p.selected_count,
                        cached: p.cache_hit_count,
                        errors: p.error_count,
                      })}
                    </span>
                  </div>
                  {p.last_error && (
                    <p className="mt-1 max-w-2xl truncate text-xs text-accent-error" title={p.last_error}>
                      {t("settings.apiConfig.lastError", { error: p.last_error })}
                    </p>
                  )}
                </div>

                <div className="flex items-center gap-4">
                  {p.has_credentials && (
                    <Button
                      variant="ghost"
                      size="sm"
                      loading={testingProvider === p.id}
                      onClick={() => handleTestProvider(p.id)}
                    >
                      {t("settings.apiConfig.testConnection")}
                    </Button>
                  )}
                  <IconButton
                    onClick={() => handleEditConfig(p)}
                    title={t("settings.apiConfig.editConfig")}
                    aria-label={t("settings.apiConfig.editConfig")}
                  >
                    <Key className="w-5 h-5" />
                  </IconButton>

                  <label className="relative inline-flex items-center cursor-pointer">
                    <input
                      type="checkbox"
                      className="sr-only peer"
                      checked={p.enabled}
                      onChange={(e) => handleToggleEnabled(p.id, e.target.checked)}
                    />
                    <div className="w-12 h-6 bg-bg-tertiary rounded-full border-[length:var(--border-width)] border-border-default peer peer-checked:bg-accent-primary peer-checked:border-accent-primary after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-text-primary after:rounded-full after:h-5 after:w-5 after:transition-all after:duration-[var(--motion-fast)] after:ease-[var(--motion-easing)] peer-checked:after:translate-x-6"></div>
                  </label>
                </div>
              </div>

              {/* 配置编辑展开面板 */}
              {editingProvider === p.id && (
                <div className="px-5 pb-5 border-t border-border-default bg-bg-primary/30">
                  <div className="pt-5 space-y-4">
                    <div className="flex items-start gap-3 p-3 bg-accent-primary/10 border-[length:var(--border-width)] border-accent-primary/20 rounded-[var(--radius-md)] mb-4">
                      <Info className="w-4 h-4 text-accent-primary mt-0.5" />
                      <p className="text-xs text-text-secondary leading-relaxed">
                        {t("settings.apiConfig.credentialsHint")}
                      </p>
                    </div>

                    {p.id === "screenscraper" && (
                      <>
                        <p className="text-xs text-text-muted">
                          {t("settings.apiConfig.screenScraperAuthHint")}
                        </p>
                        <div>
                          <label className="block text-sm font-medium text-text-primary mb-2">
                            {t("settings.apiConfig.username")}
                          </label>
                          <Input
                            type="text"
                            value={credentials.username || ""}
                            onChange={(e) =>
                              setCredentials({
                                ...credentials,
                                username: e.target.value,
                              })
                            }
                            placeholder={t("settings.apiConfig.usernamePlaceholder")}
                          />
                        </div>
                        <div>
                          <label className="block text-sm font-medium text-text-primary mb-2">
                            {t("settings.apiConfig.password")}
                          </label>
                          <Input
                            type="password"
                            value={credentials.password || ""}
                            onChange={(e) =>
                              setCredentials({
                                ...credentials,
                                password: e.target.value,
                              })
                            }
                            placeholder={t(
                              p.has_saved_password
                                ? "settings.apiConfig.savedPasswordPlaceholder"
                                : "settings.apiConfig.passwordPlaceholder",
                            )}
                          />
                        </div>
                      </>
                    )}

                    {(p.id === "steamgriddb" || p.id === "thegamesdb") && (
                      <div>
                        <label className="block text-sm font-medium text-text-primary mb-2">
                          {t("settings.apiConfig.apiKey")}
                        </label>
                        <Input
                          type="text"
                          value={credentials.api_key || ""}
                          onChange={(e) =>
                            setCredentials({
                              ...credentials,
                              api_key: e.target.value,
                            })
                          }
                          className="font-mono"
                          placeholder={t("settings.apiConfig.apiKeyPlaceholder")}
                        />
                      </div>
                    )}

                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <label className="block text-sm font-medium text-text-primary mb-2">
                          {t("settings.apiConfig.rateLimit")}
                        </label>
                        <Input
                          type="number"
                          min={1}
                          max={60}
                          value={credentials.rate_limit ?? 1}
                          onChange={(e) => setCredentials({ ...credentials, rate_limit: Number(e.target.value) })}
                        />
                      </div>
                      <div>
                        <label className="block text-sm font-medium text-text-primary mb-2">
                          {t("settings.apiConfig.threads")}
                        </label>
                        <Input
                          type="number"
                          min={1}
                          max={32}
                          value={credentials.threads ?? 1}
                          onChange={(e) => setCredentials({ ...credentials, threads: Number(e.target.value) })}
                        />
                      </div>
                    </div>
                    <p className="text-xs text-text-muted">{t("settings.apiConfig.rateLimitHint")}</p>

                    <div className="flex justify-end gap-3 pt-2">
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => setEditingProvider(null)}
                      >
                        {t("common.cancel")}
                      </Button>
                      <Button size="sm" onClick={handleSaveConfig}>
                        <Save className="w-4 h-4" />
                        {t("common.save")}
                      </Button>
                    </div>
                  </div>
                </div>
              )}
            </div>
          ))}
        </div>
      </section>

      <section>
        <h2 className="text-lg font-medium text-text-primary mb-2">
          {t("settings.apiConfig.mediaSelection")}
        </h2>
        <p className="text-sm text-text-muted mb-4">
          {t("settings.apiConfig.mediaSelectionHint")}
        </p>
        <div className="grid grid-cols-2 sm:grid-cols-3 gap-3">
          {mediaTypeOptions.map((type) => (
            <label
              key={type}
              className="flex items-center gap-3 p-3 rounded-[var(--radius-md)] bg-bg-secondary border border-border-default cursor-pointer"
            >
              <input
                type="checkbox"
                checked={mediaTypes.includes(type)}
                onChange={(event) => handleMediaTypeChange(type, event.target.checked)}
              />
              <span className="text-sm text-text-primary">
                {t(`settings.apiConfig.mediaTypes.${type}`)}
              </span>
            </label>
          ))}
        </div>
      </section>

      {/* 拖拽幽灵元素 */}
      {isDragging && draggedProvider && (
        <div
          className="fixed pointer-events-none z-50 opacity-80 w-[37.5rem] max-w-[90vw]"
          style={{
            left: mousePos.x - dragOffset.x,
            top: mousePos.y - dragOffset.y,
          }}
        >
          {(() => {
            const provider = providers.find((p) => p.id === draggedProvider);
            if (!provider) return null;

            return (
              <div className="group relative overflow-hidden rounded-[var(--radius-lg)] border-[length:var(--border-width)] bg-bg-secondary border-border-hover [box-shadow:var(--shadow-dialog)]">
                <div className="flex items-center p-5">
                  <div className="cursor-grab text-text-muted mr-3">
                    <GripVertical className="w-5 h-5" />
                  </div>
                  <div
                    className={clsx(
                      "w-12 h-12 rounded-[var(--radius-md)] flex items-center justify-center border-[length:var(--border-width)]",
                      getProviderColor(provider.id),
                    )}
                  >
                    {getProviderIcon(provider.id)}
                  </div>
                  <div className="ml-4 flex-1">
                    <div className="flex items-center gap-2">
                      <h3 className="font-bold text-text-primary text-lg">
                        {provider.name}
                      </h3>
                      {provider.has_credentials && (
                        <Badge variant="success" className="uppercase">
                          {t("settings.apiConfig.authenticated")}
                        </Badge>
                      )}
                    </div>
                    <div className="flex items-center gap-3 mt-1">
                      <span className="text-xs text-text-muted flex items-center gap-1">
                        <Activity className="w-3 h-3" />
                        {formatCapabilities(provider.capabilities)}
                      </span>
                    </div>
                  </div>
                </div>
              </div>
            );
          })()}
        </div>
      )}
    </div>
  );
}
