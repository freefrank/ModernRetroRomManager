import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { useScraperStore } from "@/stores/scraperStore";
import { clsx } from "clsx";

export default function Scraper() {
  const { t } = useTranslation();
  const { configs, fetchConfigs, saveConfig } = useScraperStore();

  useEffect(() => {
    fetchConfigs();
  }, [fetchConfigs]);

  const handleSave = async (provider: string, data: Record<string, unknown>) => {
    try {
      await saveConfig(provider, data);
    } catch (error) {
      console.error("Failed to save config:", error);
    }
  };

  const providers = [
    {
      id: "igdb",
      name: "IGDB",
      description: t("scraper.providers.igdb.description"),
      color: "purple",
      fields: [
        { key: "client_id", label: t("scraper.fields.clientId"), type: "text" },
        { key: "client_secret", label: t("scraper.fields.clientSecret"), type: "password" },
      ],
    },
    {
      id: "steamgriddb",
      name: "SteamGridDB",
      description: t("scraper.providers.steamgriddb.description"),
      color: "blue",
      fields: [{ key: "api_key", label: t("scraper.fields.apiKey"), type: "password" }],
    },
    {
      id: "thegamesdb",
      name: "TheGamesDB",
      description: t("scraper.providers.thegamesdb.description"),
      color: "green",
      fields: [{ key: "api_key", label: t("scraper.fields.apiKey"), type: "password" }],
    },
    {
      id: "mobygames",
      name: "MobyGames",
      description: t("scraper.providers.mobygames.description"),
      color: "orange",
      fields: [{ key: "api_key", label: t("scraper.fields.apiKey"), type: "password" }],
    },
    {
      id: "screenscraper",
      name: "ScreenScraper",
      description: t("scraper.providers.screenscraper.description"),
      color: "red",
      fields: [
        { key: "username", label: "Username", type: "text" },
        { key: "password", label: "Password", type: "password" },
      ],
    },
  ];

  return (
    <div className="flex flex-col h-full">
      {/* 工具栏 */}
      <div className="flex items-center justify-between px-6 py-4 border-b border-border-default bg-bg-primary/50 backdrop-blur-md sticky top-0 z-10">
        <h1 className="text-xl font-bold text-text-primary">{t("scraper.title")}</h1>
      </div>

      {/* API 配置 */}
      <div className="flex-1 p-6 overflow-auto">
        <div className="max-w-3xl">
          <h2 className="text-lg font-medium mb-4 text-text-primary">{t("scraper.apiConfig.title")}</h2>
          <p className="text-text-secondary mb-6 text-sm">
            {t("scraper.apiConfig.description")}
          </p>

          <div className="space-y-4">
            {providers.map((p) => {
              const config = configs[p.id] || { enabled: false };
              const colorClass = {
                purple: "bg-purple-500/20 text-purple-400",
                blue: "bg-blue-500/20 text-blue-400",
                green: "bg-green-500/20 text-green-400",
                orange: "bg-orange-500/20 text-orange-400",
                red: "bg-red-500/20 text-red-400",
              }[p.color] || "bg-bg-tertiary text-text-primary";

              return (
                <div key={p.id} className="p-4 bg-bg-secondary border border-border-default rounded-xl hover:border-border-hover transition-colors">
                  <div className="flex items-center justify-between mb-4">
                    <div className="flex items-center gap-3">
                      <div className={clsx("w-10 h-10 rounded-lg flex items-center justify-center font-bold text-sm", colorClass)}>
                        {p.name.substring(0, 2).toUpperCase()}
                      </div>
                      <div>
                        <h3 className="font-medium text-text-primary">{p.name}</h3>
                        <p className="text-sm text-text-secondary">{p.description}</p>
                      </div>
                    </div>
                    <label className="relative inline-flex items-center cursor-pointer">
                      <input
                        type="checkbox"
                        className="sr-only peer"
                        checked={config.enabled}
                        onChange={(e) => handleSave(p.id, { enabled: e.target.checked })}
                      />
                      <div className="w-11 h-6 bg-bg-tertiary rounded-full peer peer-checked:bg-accent-primary peer-checked:after:translate-x-full after:content-[''] after:absolute after:top-0.5 after:left-[2px] after:bg-text-primary after:rounded-full after:h-5 after:w-5 after:transition-all"></div>
                    </label>
                  </div>

                  {config.enabled && (
                    <div className={clsx("grid gap-4", p.fields.length > 1 ? "grid-cols-2" : "grid-cols-1")}>
                      {p.fields.map((field) => (
                        <div key={field.key}>
                          <label className="block text-xs text-text-muted mb-1.5 uppercase font-medium tracking-wider">
                            {field.label}
                          </label>
                          <input
                            type={field.type}
                            placeholder={`Enter ${field.label}`}
                            // @ts-ignore
                            defaultValue={config[field.key] || ""}
                            onBlur={(e) => handleSave(p.id, { [field.key]: e.target.value })}
                            className="w-full px-3 py-2 bg-bg-primary border border-border-default rounded-lg focus:outline-none focus:border-accent-primary text-sm text-text-primary"
                          />
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
}
