import { useEffect, useRef } from "react";
import { Eye, EyeOff, Trash2 } from "lucide-react";
import { useTranslation } from "react-i18next";

interface RomContextMenuProps {
  x: number;
  y: number;
  isHidden: boolean;
  onToggleHidden: () => void;
  onDelete: () => void;
  onClose: () => void;
}

export default function RomContextMenu({
  x,
  y,
  isHidden,
  onToggleHidden,
  onDelete,
  onClose,
}: RomContextMenuProps) {
  const ref = useRef<HTMLDivElement>(null);
  const { t } = useTranslation();

  useEffect(() => {
    const handlePointer = (event: MouseEvent) => {
      if (ref.current && !ref.current.contains(event.target as Node)) onClose();
    };
    const handleKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    document.addEventListener("mousedown", handlePointer);
    document.addEventListener("keydown", handleKey);
    return () => {
      document.removeEventListener("mousedown", handlePointer);
      document.removeEventListener("keydown", handleKey);
    };
  }, [onClose]);

  // 防止菜单溢出视口右/下边缘
  const left = Math.min(x, window.innerWidth - 190);
  const top = Math.min(y, window.innerHeight - 110);

  return (
    <div
      ref={ref}
      role="menu"
      className="rr-card fixed z-50 min-w-[170px] py-1 rounded-[var(--radius-md)] bg-bg-secondary border-[length:var(--border-width)] border-border-default shadow-[var(--shadow-dialog)]"
      style={{ left, top }}
    >
      <button
        type="button"
        role="menuitem"
        onClick={() => {
          onToggleHidden();
          onClose();
        }}
        className="w-full flex items-center gap-2.5 px-3 py-2 text-sm text-text-primary hover:bg-bg-tertiary transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)]"
      >
        {isHidden ? <Eye className="w-4 h-4 shrink-0" /> : <EyeOff className="w-4 h-4 shrink-0" />}
        <span>{isHidden ? t("library.context.unhide") : t("library.context.hide")}</span>
      </button>
      <button
        type="button"
        role="menuitem"
        onClick={() => {
          onDelete();
          onClose();
        }}
        className="w-full flex items-center gap-2.5 px-3 py-2 text-sm text-accent-error hover:bg-accent-error/10 transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)]"
      >
        <Trash2 className="w-4 h-4 shrink-0" />
        <span>{t("library.context.delete")}</span>
      </button>
    </div>
  );
}
