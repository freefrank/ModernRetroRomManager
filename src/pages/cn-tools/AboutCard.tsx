import { useState } from "react";
import { useTranslation } from "react-i18next";
import { Download, Loader2, Info, ExternalLink } from "lucide-react";
import { isTauri } from "@/lib/api";
import { toast } from "@/components/ui";

const REPO_URL = "https://github.com/yingw/rom-name-cn";

/** 关于信息卡:数据来源说明 + 更新数据库入口 */
export default function AboutCard() {
  const { t } = useTranslation();
  const [isUpdating, setIsUpdating] = useState(false);

  const handleUpdate = async () => {
    setIsUpdating(true);
    try {
      if (isTauri()) {
        const { invoke } = await import("@tauri-apps/api/core");
        await invoke("update_cn_repo");
      }
      toast.success(t("cnRomTools.alerts.databaseUpdateSuccess"));
    } catch (error) {
      console.error("Failed to update CN repo:", error);
      toast.error(t("cnRomTools.alerts.updateFailed", { error: String(error) }));
    } finally {
      setIsUpdating(false);
    }
  };

  return (
    <section className="shrink-0 bg-bg-secondary rounded-[var(--radius-lg)] border-[length:var(--border-width)] border-border-default overflow-hidden">
      <div className="p-6 border-b border-border-default">
        <div className="flex items-center gap-3 mb-2">
          <Info className="w-5 h-5 text-accent-primary" />
          <h2 className="text-lg font-bold text-text-primary">{t("cnRomTools.about.title")}</h2>
        </div>
        <p className="text-sm text-text-secondary leading-relaxed">
          {t("cnRomTools.about.description.part1")}{" "}
          <a
            href={REPO_URL}
            target="_blank"
            rel="noopener noreferrer"
            className="text-accent-primary hover:underline font-bold"
          >
            yingw/rom-name-cn
          </a>{" "}
          {t("cnRomTools.about.description.part2")}
        </p>
      </div>
      <div className="bg-bg-tertiary/50 p-4 flex items-center justify-between">
        <div className="text-xs text-text-muted font-medium">{t("cnRomTools.about.dataSource")}</div>
        <div className="flex gap-4">
          <a
            href={REPO_URL}
            target="_blank"
            rel="noopener noreferrer"
            className="flex items-center gap-1.5 text-xs font-bold text-text-primary hover:text-accent-primary transition-colors duration-[var(--motion-fast)]"
          >
            {t("cnRomTools.about.visitRepo")} <ExternalLink className="w-3 h-3" />
          </a>
          <button
            onClick={handleUpdate}
            disabled={isUpdating}
            className="flex items-center gap-1.5 text-xs font-bold text-accent-primary hover:opacity-80 transition-opacity duration-[var(--motion-fast)] disabled:opacity-50"
          >
            {isUpdating ? (
              <Loader2 className="w-3 h-3 animate-spin" />
            ) : (
              <Download className="w-3 h-3" />
            )}
            {isUpdating ? t("cnRomTools.about.updating") : t("cnRomTools.about.updateDatabase")}
          </button>
        </div>
      </div>
    </section>
  );
}
