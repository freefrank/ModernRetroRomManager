import { useEffect, useState, type KeyboardEvent } from "react";
import { Gamepad2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Card } from "@/components/ui";
import { resolveSystemLogoUrl } from "@/lib/api";

interface SystemCardProps {
  /** 系统名称(同 romStore.availableSystems[].name) */
  name: string;
  /** 该系统下的游戏数量 */
  romCount: number;
  /** 系统 logo 文件名(打包资源 logo/ 下),缺省时显示占位图标 */
  logoFile?: string;
  onClick: () => void;
}

/** 系统货架卡片:系统 logo(缺省回退占位图标)、系统名与游戏数,点击进入单系统页 */
export default function SystemCard({ name, romCount, logoFile, onClick }: SystemCardProps) {
  const { t } = useTranslation();
  const [logoUrl, setLogoUrl] = useState<string | null>(null);
  const [imgError, setImgError] = useState(false);

  useEffect(() => {
    let mounted = true;
    setLogoUrl(null);
    setImgError(false);
    if (!logoFile) return;
    resolveSystemLogoUrl(logoFile).then((url) => {
      if (mounted) setLogoUrl(url);
    });
    return () => {
      mounted = false;
    };
  }, [logoFile]);

  const handleKeyDown = (e: KeyboardEvent<HTMLDivElement>) => {
    if (e.key === "Enter" || e.key === " ") {
      e.preventDefault();
      onClick();
    }
  };

  return (
    <Card
      role="button"
      tabIndex={0}
      onClick={onClick}
      onKeyDown={handleKeyDown}
      className="group cursor-pointer select-none p-6 flex flex-col items-center gap-3 text-center hover:border-border-hover"
    >
      <div className="w-full h-16 rounded-[var(--radius-md)] bg-bg-tertiary border-[length:var(--border-width)] border-border-default flex items-center justify-center overflow-hidden">
        {logoUrl && !imgError ? (
          <img
            src={logoUrl}
            alt={name}
            loading="lazy"
            className="max-w-full max-h-full object-contain p-2"
            onError={() => setImgError(true)}
          />
        ) : (
          <Gamepad2 className="w-8 h-8 text-text-muted group-hover:text-accent-primary transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)]" />
        )}
      </div>
      <div className="min-w-0 w-full">
        <h3 className="font-semibold text-text-primary truncate" title={name}>
          {name}
        </h3>
        <p className="text-sm text-text-secondary mt-1">
          {t("library.gameCount", { count: romCount })}
        </p>
      </div>
    </Card>
  );
}
