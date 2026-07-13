import { useState } from "react";
import { useSearchParams } from "react-router-dom";
import { useTranslation } from "react-i18next";
import { Tabs } from "@/components/ui";
import AppearanceSection from "./settings/AppearanceSection";
import GeneralSection from "./settings/GeneralSection";
import ScraperSection from "./settings/ScraperSection";
import AboutSection from "./settings/AboutSection";

type SettingsTab = "appearance" | "general" | "scraper" | "about";

const TAB_VALUES: SettingsTab[] = ["appearance", "general", "scraper", "about"];

export default function Settings() {
  const { t } = useTranslation();
  const [searchParams, setSearchParams] = useSearchParams();
  const requestedTab = searchParams.get("tab") as SettingsTab | null;
  const [tab, setTab] = useState<SettingsTab>(
    requestedTab && TAB_VALUES.includes(requestedTab) ? requestedTab : "appearance",
  );

  const items = TAB_VALUES.map((value) => ({
    value,
    label: t(`settings.tabs.${value}`),
  }));

  return (
    <div className="rr-page flex flex-col h-full">
      {/* 工具栏 */}
      <div className="flex items-center justify-between gap-4 px-6 py-4 border-b border-border-default bg-bg-primary/50 backdrop-blur-md sticky top-0 z-10">
        <h1 className="text-xl font-bold text-text-primary">
          {t("settings.title")}
        </h1>
        <Tabs
          items={items}
          value={tab}
          onChange={(value) => {
            const next = value as SettingsTab;
            setTab(next);
            setSearchParams(next === "appearance" ? {} : { tab: next });
          }}
        />
      </div>

      {/* 内容区 */}
      <div className="flex-1 p-6 overflow-auto">
        <div className="max-w-3xl">
          {tab === "appearance" && <AppearanceSection />}
          {tab === "general" && <GeneralSection />}
          {tab === "scraper" && <ScraperSection />}
          {tab === "about" && <AboutSection />}
        </div>
      </div>
    </div>
  );
}
