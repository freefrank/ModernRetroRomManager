import { forwardRef, useEffect, type HTMLAttributes, type ReactNode } from "react";
import { createPortal } from "react-dom";
import { X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { IconButton } from "./IconButton";

interface Props extends Omit<HTMLAttributes<HTMLDivElement>, "title"> {
  /** 受控:是否打开 */
  open: boolean;
  /** 关闭回调(ESC / 遮罩点击 / 关闭按钮) */
  onClose: () => void;
  /** 标题插槽 */
  title?: ReactNode;
  /** 底部插槽(操作按钮区) */
  footer?: ReactNode;
}

export const Dialog = forwardRef<HTMLDivElement, Props>(function Dialog(
  { open, onClose, title, footer, className = "", children, ...rest },
  ref,
) {
  const { t } = useTranslation();

  useEffect(() => {
    if (!open) return;
    const onKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [open, onClose]);

  if (!open) return null;

  return createPortal(
    <div
      className="fixed inset-0 z-[90] flex items-center justify-center p-4 bg-bg-primary/60 backdrop-blur-sm"
      onClick={onClose}
    >
      <div
        ref={ref}
        role="dialog"
        aria-modal="true"
        onClick={(e) => e.stopPropagation()}
        className={
          "rr-dialog w-full max-w-lg max-h-[80vh] overflow-y-auto custom-scrollbar " +
          "bg-bg-secondary text-text-primary " +
          "rounded-[var(--radius-lg)] border-[length:var(--border-width)] border-border-default " +
          "[box-shadow:var(--shadow-dialog)] " +
          className
        }
        {...rest}
      >
        <div className="flex items-center justify-between gap-4 px-5 pt-4 pb-2">
          <h2 className="text-lg font-semibold text-text-primary">{title}</h2>
          <IconButton size="sm" aria-label={t("ui.dialog.close")} onClick={onClose}>
            <X className="w-4 h-4" />
          </IconButton>
        </div>
        <div className="px-5 py-3">{children}</div>
        {footer && (
          <div className="flex items-center justify-end gap-2 px-5 pb-4 pt-2">{footer}</div>
        )}
      </div>
    </div>,
    document.body,
  );
});
