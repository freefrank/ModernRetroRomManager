import type { KeyboardEvent } from "react";
import { Gamepad2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { Card } from "@/components/ui";

interface SystemCardProps {
  /** 系统名称(同 romStore.availableSystems[].name) */
  name: string;
  /** 该系统下的游戏数量 */
  romCount: number;
  onClick: () => void;
}

/** 系统货架卡片:logo 占位、系统名与游戏数,点击进入单系统页 */
export default function SystemCard({ name, romCount, onClick }: SystemCardProps) {
  const { t } = useTranslation();

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
      <div className="w-16 h-16 rounded-[var(--radius-md)] bg-bg-tertiary border-[length:var(--border-width)] border-border-default flex items-center justify-center">
        <Gamepad2 className="w-8 h-8 text-text-muted group-hover:text-accent-primary transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)]" />
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
