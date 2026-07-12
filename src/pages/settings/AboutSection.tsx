import { useTranslation } from "react-i18next";
import { Card } from "@/components/ui";

const REPO_URL = "https://github.com/freefrank/ModernRetroRomManager";

export default function AboutSection() {
  const { t } = useTranslation();
  return (
    <section>
      <h2 className="text-lg font-medium text-text-primary mb-4">
        {t("settings.about.title")}
      </h2>

      <Card className="p-4">
        <div className="flex items-center gap-4">
          <div className="w-16 h-16 bg-gradient-to-br from-accent-primary to-accent-secondary rounded-[var(--radius-lg)] flex items-center justify-center [box-shadow:var(--shadow-card)]">
            <span className="text-2xl font-bold text-bg-primary">MR</span>
          </div>
          <div>
            <h3 className="text-lg font-bold text-text-primary">
              ModernRetroRomManager
            </h3>
            <p className="text-text-secondary text-sm">
              {t("settings.about.version", { version: APP_VERSION })}
            </p>

            <div className="flex items-center gap-4 mt-2">
              <a
                href={REPO_URL}
                className="text-accent-primary hover:text-accent-secondary hover:underline text-xs transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)]"
                target="_blank"
                rel="noopener noreferrer"
              >
                {t("settings.about.repository")}
              </a>
              <span className="text-text-muted text-xs">•</span>
              <span className="text-text-muted text-xs">
                {t("settings.about.license")}
              </span>
            </div>
          </div>
        </div>
      </Card>
    </section>
  );
}
