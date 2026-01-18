import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";
import { useScraperStore } from "@/stores/scraperStore";
import { clsx } from "clsx";
import { Settings2, Key, Globe, Shield, Activity, Save, Loader2, Info, Languages, Download } from "lucide-react";
import type { ScraperCredentials } from "@/types";
import { isTauri } from "@/lib/api";

export default function Scraper() {
  const { t } = useTranslation();
  const { providers, fetchProviders, configureProvider, setProviderEnabled, isLoading } = useScraperStore();
  const [editingProvider, setEditingProvider] = useState<string | null>(null);
  const [credentials, setCredentials] = useState<ScraperCredentials>({});
  const [isUpdatingCn, setIsUpdatingCn] = useState(false);

  useEffect(() => {
    fetchProviders();
  }, [fetchProviders]);

  const handleToggleEnabled = async (providerId: string, enabled: boolean) => {
    try {
      await setProviderEnabled(providerId, enabled);
    } catch (error) {
      console.error("Failed to toggle provider:", error);
    }
  };

  const handleEditConfig = (provider: any) => {
    setEditingProvider(provider.id);
    setCredentials({}); 
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

  const handleUpdateCnRepo = async () => {
    setIsUpdatingCn(true);
    try {
      if (isTauri()) {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("update_cn_repo");
      }
      // alert("中文数据库更新成功");
    } catch (error) {
      console.error("Failed to update CN repo:", error);
      // alert("更新失败: " + String(error));
    } finally {
      setIsUpdatingCn(false);
    }
  };

  const getProviderIcon = (id: string) => {
    switch (id) {
      case "screenscraper": return <Globe className="w-5 h-5" />;
      case "steamgriddb": return <Activity className="w-5 h-5" />;
      case "local_cn_repo": return <Languages className="w-5 h-5" />;
      default: return <Settings2 className="w-5 h-5" />;
    }
  };

  const getProviderColor = (id: string) => {
    switch (id) {
      case "screenscraper": return "bg-red-500/20 text-red-400 border-red-500/30";
      case "steamgriddb": return "bg-blue-500/20 text-blue-400 border-blue-500/30";
      case "local_cn_repo": return "bg-yellow-500/20 text-yellow-400 border-yellow-500/30";
      default: return "bg-bg-tertiary text-text-primary border-border-default";
    }
  };

  return (
    <div className="flex flex-col h-full bg-bg-primary">
      {/* 工具栏 */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-border-default bg-bg-primary/50 backdrop-blur-md sticky top-0 z-10">
        <div className="flex items-center gap-3">
          <Settings2 className="w-6 h-6 text-accent-primary" />
          <h1 className="text-xl font-bold text-text-primary tracking-tight">{t("scraper.title")}</h1>
        </div>
        {isLoading && <Loader2 className="w-5 h-5 text-accent-primary animate-spin" />}
      </div>

      <div className="flex-1 p-8 overflow-auto">
        <div className="max-w-5xl mx-auto">
          {/* 状态总览 */}
          <section className="mb-10">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <div className="p-4 bg-bg-secondary rounded-2xl border border-border-default">
                <div className="text-xs text-text-muted mb-1 uppercase tracking-widest font-bold">{t("scraper.status.enabledSources")}</div>
                <div className="text-2xl font-bold text-accent-primary">
                  {providers.filter(p => p.enabled).length} / {providers.length}
                </div>
              </div>
              <div className="p-4 bg-bg-secondary rounded-2xl border border-border-default">
                <div className="text-xs text-text-muted mb-1 uppercase tracking-widest font-bold">{t("scraper.status.avgConfidence")}</div>
                <div className="text-2xl font-bold text-green-400">92%</div>
              </div>
              <div className="p-4 bg-bg-secondary rounded-2xl border border-border-default">
                <div className="text-xs text-text-muted mb-1 uppercase tracking-widest font-bold">{t("scraper.status.configuredCredentials")}</div>
                <div className="text-2xl font-bold text-blue-400">
                  {providers.filter(p => p.has_credentials).length}
                </div>
              </div>
            </div>
          </section>

          {/* Provider 列表 */}
          <section>
            <h2 className="text-lg font-bold mb-6 text-text-primary flex items-center gap-2">
              <Shield className="w-5 h-5 text-accent-primary" />
              {t("scraper.apiConfig.title")}
            </h2>
            
            <div className="grid grid-cols-1 gap-4">
              {providers.map((p) => (
                <div 
                  key={p.id} 
                  className={clsx(
                    "group relative overflow-hidden rounded-2xl border transition-all duration-300",
                    p.enabled ? "bg-bg-secondary border-border-hover" : "bg-bg-primary/50 border-border-default opacity-70"
                  )}
                >
                  <div className="flex items-center p-5">
                    <div className={clsx("w-12 h-12 rounded-xl flex items-center justify-center border", getProviderColor(p.id))}>
                      {getProviderIcon(p.id)}
                    </div>
                    
                    <div className="ml-4 flex-1">
                      <div className="flex items-center gap-2">
                        <h3 className="font-bold text-text-primary text-lg">{p.name}</h3>
                        {p.has_credentials && (
                          <span className="px-2 py-0.5 rounded-md bg-green-500/10 text-green-400 text-[10px] font-bold uppercase tracking-tighter border border-green-500/20">
                            {t("scraper.status.authenticated")}
                          </span>
                        )}
                      </div>
                      <div className="flex items-center gap-3 mt-1">
                        <span className="text-xs text-text-muted flex items-center gap-1">
                          <Activity className="w-3 h-3" />
                          {p.capabilities.join(", ").toUpperCase()}
                        </span>
                      </div>
                    </div>

                    <div className="flex items-center gap-4">
                      <button
                        onClick={() => handleEditConfig(p)}
                        className="p-2.5 rounded-xl bg-bg-tertiary text-text-secondary hover:text-accent-primary hover:bg-bg-primary transition-all border border-transparent hover:border-accent-primary/30"
                        title={t("common.edit")}
                      >
                        <Key className="w-5 h-5" />
                      </button>

                      <label className="relative inline-flex items-center cursor-pointer">
                        <input
                          type="checkbox"
                          className="sr-only peer"
                          checked={p.enabled}
                          onChange={(e) => handleToggleEnabled(p.id, e.target.checked)}
                        />
                        <div className="w-12 h-6 bg-bg-tertiary rounded-full border border-border-default peer peer-checked:bg-accent-primary peer-checked:border-accent-primary after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-text-primary after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:after:translate-x-6 shadow-sm"></div>
                      </label>
                    </div>
                  </div>

                  {/* 配置编辑展开面 */}
                  {editingProvider === p.id && (
                    <div className="px-5 pb-5 border-t border-border-default bg-bg-primary/30 animate-in slide-in-from-top-2 duration-200">
                      <div className="pt-5 space-y-4">
                        <div className="flex items-start gap-3 p-3 bg-blue-500/10 border border-blue-500/20 rounded-xl mb-4">
                          <Info className="w-4 h-4 text-blue-400 mt-0.5" />
                          <p className="text-xs text-blue-200 leading-relaxed">
                            {t("scraper.apiConfig.credentialsNote")}
                          </p>
                        </div>

                        {p.id === "local_cn_repo" ? (
                          <div className="flex justify-end gap-3 pt-2">
                            <button
                              onClick={handleUpdateCnRepo}
                              disabled={isUpdatingCn}
                              className="flex items-center gap-2 px-6 py-2 bg-accent-primary text-bg-primary text-sm font-bold rounded-xl hover:opacity-90 active:scale-95 transition-all shadow-lg shadow-accent-primary/20 disabled:opacity-50"
                            >
                              {isUpdatingCn ? <Loader2 className="w-4 h-4 animate-spin" /> : <Download className="w-4 h-4" />}
                              更新数据库
                            </button>
                          </div>
                        ) : p.id === "screenscraper" ? (
                          <div className="grid grid-cols-2 gap-4">
                            <div>
                              <label className="block text-[10px] text-text-muted mb-1.5 uppercase font-bold tracking-wider">{t("scraper.fields.username")}</label>
                              <input
                                type="text"
                                placeholder={t("scraper.fields.username")}
                                value={credentials.username || ""}
                                onChange={(e) => setCredentials({ ...credentials, username: e.target.value })}
                                className="w-full px-4 py-2.5 bg-bg-secondary border border-border-default rounded-xl focus:outline-none focus:border-accent-primary text-sm text-text-primary shadow-inner"
                              />
                            </div>
                            <div>
                              <label className="block text-[10px] text-text-muted mb-1.5 uppercase font-bold tracking-wider">{t("scraper.fields.password")}</label>
                              <input
                                type="password"
                                placeholder={t("scraper.fields.password")}
                                value={credentials.password || ""}
                                onChange={(e) => setCredentials({ ...credentials, password: e.target.value })}
                                className="w-full px-4 py-2.5 bg-bg-secondary border border-border-default rounded-xl focus:outline-none focus:border-accent-primary text-sm text-text-primary shadow-inner"
                              />
                            </div>
                          </div>
                        ) : (
                          <div>
                            <label className="block text-[10px] text-text-muted mb-1.5 uppercase font-bold tracking-wider">{t("scraper.fields.apiKey")}</label>
                            <input
                              type="password"
                              placeholder={t("scraper.fields.apiKey")}
                              value={credentials.api_key || ""}
                              onChange={(e) => setCredentials({ ...credentials, api_key: e.target.value })}
                              className="w-full px-4 py-2.5 bg-bg-secondary border border-border-default rounded-xl focus:outline-none focus:border-accent-primary text-sm text-text-primary shadow-inner"
                            />
                          </div>
                        )}

                        <div className="flex justify-end gap-3 pt-2">
                          <button
                            onClick={() => setEditingProvider(null)}
                            className="px-4 py-2 text-sm font-bold text-text-secondary hover:text-text-primary transition-colors"
                          >
                            {t("common.cancel")}
                          </button>
                          <button
                            onClick={handleSaveConfig}
                            className="flex items-center gap-2 px-6 py-2 bg-accent-primary text-bg-primary text-sm font-bold rounded-xl hover:opacity-90 active:scale-95 transition-all shadow-lg shadow-accent-primary/20"
                          >
                            <Save className="w-4 h-4" />
                            {t("scraper.dialog.confirmAndApply")}
                          </button>
                        </div>
                      </div>
                    </div>
                  )}
                </div>
              ))}
            </div>
          </section>
        </div>
      </div>
    </div>
  );
}
