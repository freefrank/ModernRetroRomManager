import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { useScraperStore } from "@/stores/scraperStore";
import { Settings2, Activity, Loader2, AlertCircle } from "lucide-react";

export default function Scraper() {

  const { t } = useTranslation();
  const navigate = useNavigate();
  const { providers, fetchProviders, isLoading } = useScraperStore();

  useEffect(() => {
    fetchProviders();
  }, [fetchProviders]);

  const hasConfiguredCredentials = providers.filter(p => p.has_credentials).length > 0;

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

          {/* API 配置提醒 */}
          {!hasConfiguredCredentials && (
            <section>
              <div className="p-6 bg-yellow-500/10 border border-yellow-500/30 rounded-2xl">
                <div className="flex items-start gap-4">
                  <div className="w-12 h-12 bg-yellow-500/20 rounded-xl flex items-center justify-center flex-shrink-0">
                    <AlertCircle className="w-6 h-6 text-yellow-400" />
                  </div>
                  <div className="flex-1">
                    <h3 className="text-lg font-bold text-yellow-400 mb-2">未配置 API 凭证</h3>
                    <p className="text-sm text-yellow-200 mb-4 leading-relaxed">
                      Scraper 功能需要配置 API 凭证才能使用。请前往设置页面配置 ScreenScraper 或 SteamGridDB 的 API 凭证。
                    </p>
                    <button
                      onClick={() => navigate('/settings')}
                      className="px-4 py-2 bg-yellow-500/20 text-yellow-400 rounded-xl hover:bg-yellow-500/30 transition-all text-sm font-medium border border-yellow-500/30"
                    >
                      前往设置页面
                    </button>
                  </div>
                </div>
              </div>
            </section>
          )}
        </div>
      </div>
    </div>
  );
}
