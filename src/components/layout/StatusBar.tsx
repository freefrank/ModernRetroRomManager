import { useTranslation } from "react-i18next";

export default function StatusBar() {
  const { t } = useTranslation();

  return (
    <footer className="h-7 flex items-center justify-between px-4 bg-bg-secondary border-t border-border-default text-xs text-text-muted">
      <div className="flex items-center gap-4">
        <span>{t("statusBar.romCount", { count: 0 })}</span>
        <span>{t("statusBar.scrapedCount", { count: 0 })}</span>
      </div>
      <div className="flex items-center gap-4">
        <span>{t("statusBar.storageUsed", { size: "0 MB" })}</span>
      </div>
    </footer>
  );
}
