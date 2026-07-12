import { forwardRef, type HTMLAttributes, type ReactNode } from "react";
import { useTranslation } from "react-i18next";

interface Props extends Omit<HTMLAttributes<HTMLDivElement>, "title"> {
  /** 图标插槽 */
  icon?: ReactNode;
  /** 标题(缺省用 i18n 默认文案) */
  title?: ReactNode;
  /** 描述文本 */
  description?: ReactNode;
  /** 操作按钮插槽 */
  action?: ReactNode;
}

export const EmptyState = forwardRef<HTMLDivElement, Props>(function EmptyState(
  { icon, title, description, action, className = "", children, ...rest },
  ref,
) {
  const { t } = useTranslation();
  return (
    <div
      ref={ref}
      className={
        "rr-empty flex flex-col items-center justify-center gap-3 px-6 py-10 text-center " +
        "rounded-[var(--radius-lg)] border-[length:var(--border-width)] border-dashed border-border-default " +
        className
      }
      {...rest}
    >
      {icon && <div className="text-text-muted [&>svg]:w-12 [&>svg]:h-12">{icon}</div>}
      <div className="font-medium text-text-primary">{title ?? t("ui.empty.title")}</div>
      {description && <div className="text-sm text-text-secondary">{description}</div>}
      {action && <div className="mt-2">{action}</div>}
      {children}
    </div>
  );
});
