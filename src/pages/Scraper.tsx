import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useNavigate } from "react-router-dom";
import { useScraperStore } from "@/stores/scraperStore";
import { Settings2, AlertCircle, KeyRound } from "lucide-react";
import { Badge, Button, Card, Spinner } from "@/components/ui";
import { clsx } from "clsx";

export default function Scraper() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { providers, fetchProviders, isLoading } = useScraperStore();

  useEffect(() => {
    fetchProviders();
  }, [fetchProviders]);

  const hasConfiguredCredentials = providers.some((p) => p.has_credentials);
  const enabledCount = providers.filter((p) => p.enabled).length;
  const credentialsCount = providers.filter((p) => p.has_credentials).length;
  // 能力枚举 → 中文标签(未知值回退原文)
  const formatCapability = (cap: string) =>
    t(`scraper.capabilities.${cap}`, { defaultValue: cap });

  return (
    <div className="rr-page flex flex-col h-full bg-bg-primary">
      {/* 工具栏 */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-border-default bg-bg-primary/50 backdrop-blur-md sticky top-0 z-10">
        <div className="flex items-center gap-3">
          <Settings2 className="w-6 h-6 text-accent-primary" />
          <h1 className="text-xl font-bold text-text-primary tracking-tight">
            {t("scraper.title")}
          </h1>
        </div>
        {isLoading && <Spinner size={20} className="text-accent-primary" />}
      </div>

      <div className="flex-1 p-8 overflow-auto">
        <div className="max-w-5xl mx-auto space-y-8">
          {/* 状态总览 */}
          <section>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
              <Card className="p-4">
                <div className="text-xs text-text-muted mb-1 uppercase tracking-widest font-bold">
                  {t("scraper.status.enabledSources")}
                </div>
                {/* 语义配色:有启用源为 success,一个未启用为中性 */}
                <div
                  className={clsx(
                    "text-2xl font-bold",
                    enabledCount > 0 ? "text-accent-success" : "text-text-secondary"
                  )}
                >
                  {enabledCount} / {providers.length}
                </div>
              </Card>
              <Card className="p-4">
                <div className="text-xs text-text-muted mb-1 uppercase tracking-widest font-bold">
                  {t("scraper.status.configuredCredentials")}
                </div>
                {/* 语义配色:已配置凭据为 success,缺失为 warning */}
                <div
                  className={clsx(
                    "text-2xl font-bold",
                    credentialsCount > 0 ? "text-accent-success" : "text-accent-warning"
                  )}
                >
                  {credentialsCount}
                </div>
              </Card>
            </div>
          </section>

          {/* 数据源状态列表 */}
          <section>
            <h2 className="text-lg font-medium text-text-primary mb-4">
              {t("scraper.providerList.title")}
            </h2>
            <div className="space-y-3">
              {providers.map((p) => (
                <Card key={p.id} className="p-4 flex items-center gap-4">
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2 flex-wrap">
                      <span className="font-bold text-text-primary">{p.name}</span>
                      <Badge variant={p.enabled ? "success" : "default"}>
                        {p.enabled
                          ? t("scraper.status.enabled")
                          : t("scraper.status.disabled")}
                      </Badge>
                      {p.has_credentials ? (
                        <Badge variant="info">
                          <KeyRound className="w-3 h-3" />
                          {t("scraper.status.authenticated")}
                        </Badge>
                      ) : (
                        p.requires_credentials && (
                          <Badge variant="warning">
                            <KeyRound className="w-3 h-3" />
                            {t("scraper.status.missingCredentials")}
                          </Badge>
                        )
                      )}
                    </div>
                    <div className="text-xs text-text-muted mt-1">
                      {p.capabilities.map(formatCapability).join(" · ")}
                    </div>
                  </div>
                  <Button variant="ghost" size="sm" onClick={() => navigate("/settings")}>
                    {t("scraper.providerList.configure")}
                  </Button>
                </Card>
              ))}
            </div>
          </section>

          {/* API 配置提醒 */}
          {!hasConfiguredCredentials && (
            <section>
              <Card className="p-6 border-accent-warning/40 bg-accent-warning/10">
                <div className="flex items-start gap-4">
                  <div className="w-12 h-12 bg-accent-warning/20 rounded-[var(--radius-md)] flex items-center justify-center flex-shrink-0">
                    <AlertCircle className="w-6 h-6 text-accent-warning" />
                  </div>
                  <div className="flex-1">
                    <h3 className="text-lg font-bold text-accent-warning mb-2">
                      {t("scraper.noCredentials.title")}
                    </h3>
                    <p className="text-sm text-text-secondary mb-4 leading-relaxed">
                      {t("scraper.noCredentials.description")}
                    </p>
                    <Button variant="ghost" size="sm" onClick={() => navigate("/settings")}>
                      {t("scraper.noCredentials.goToSettings")}
                    </Button>
                  </div>
                </div>
              </Card>
            </section>
          )}
        </div>
      </div>
    </div>
  );
}
