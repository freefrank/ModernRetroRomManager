import { ChevronDown } from "lucide-react";
import { useTranslation } from "react-i18next";

export const MEDIA_TYPES = [
  "boxfront", "boxback", "box3d", "logo", "screenshot", "titlescreen",
  "hero", "banner", "icon", "video", "manual",
];

interface Props {
  value: string[];
  onChange: (value: string[]) => void;
}

export default function MediaTypeSelector({ value, onChange }: Props) {
  const { t } = useTranslation();
  return (
    <details className="rounded-[var(--radius-md)] border border-border-default bg-bg-secondary/50">
      <summary className="cursor-pointer list-none flex items-center justify-between p-3 text-sm font-medium text-text-primary">
        <span>{t("scraper.mediaOptions.title")} · {t("scraper.mediaOptions.selected", { count: value.length })}</span>
        <ChevronDown className="w-4 h-4 text-text-muted" />
      </summary>
      <div className="grid grid-cols-2 sm:grid-cols-3 gap-2 p-3 pt-0">
        {MEDIA_TYPES.map((type) => (
          <label key={type} className="flex items-center gap-2 text-xs text-text-secondary">
            <input
              type="checkbox"
              checked={value.includes(type)}
              onChange={(event) => onChange(event.target.checked
                ? Array.from(new Set([...value, type]))
                : value.filter((item) => item !== type))}
            />
            {t(`settings.apiConfig.mediaTypes.${type}`)}
          </label>
        ))}
      </div>
    </details>
  );
}
