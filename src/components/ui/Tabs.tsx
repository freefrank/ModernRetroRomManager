import { forwardRef, type HTMLAttributes, type ReactNode } from "react";

export interface TabItem {
  value: string;
  label: ReactNode;
  disabled?: boolean;
}

interface Props extends Omit<HTMLAttributes<HTMLDivElement>, "onChange"> {
  items: TabItem[];
  /** 受控:当前选中值 */
  value: string;
  onChange: (value: string) => void;
}

export const Tabs = forwardRef<HTMLDivElement, Props>(function Tabs(
  { items, value, onChange, className = "", ...rest },
  ref,
) {
  return (
    <div
      ref={ref}
      role="tablist"
      className={
        "rr-tabs inline-flex items-center gap-1 p-1 bg-bg-secondary " +
        "rounded-[var(--radius-md)] border-[length:var(--border-width)] border-border-default " +
        className
      }
      {...rest}
    >
      {items.map((item) => {
        const active = item.value === value;
        return (
          <button
            key={item.value}
            type="button"
            role="tab"
            aria-selected={active}
            disabled={item.disabled}
            onClick={() => onChange(item.value)}
            className={
              "h-8 px-3 text-sm select-none rounded-[var(--radius-sm)] " +
              "transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)] " +
              "disabled:opacity-50 disabled:pointer-events-none focus-visible:outline-none " +
              (active
                ? "bg-accent-primary text-bg-primary font-medium"
                : "text-text-secondary hover:text-text-primary hover:bg-bg-tertiary")
            }
          >
            {item.label}
          </button>
        );
      })}
    </div>
  );
});
