import { forwardRef, type HTMLAttributes, type ReactNode } from "react";

interface Props extends Omit<HTMLAttributes<HTMLSpanElement>, "content"> {
  /** 提示内容 */
  content: ReactNode;
  side?: "top" | "bottom";
}

export const Tooltip = forwardRef<HTMLSpanElement, Props>(function Tooltip(
  { content, side = "top", className = "", children, ...rest },
  ref,
) {
  const sides = {
    top: "bottom-full left-1/2 -translate-x-1/2 mb-1.5",
    bottom: "top-full left-1/2 -translate-x-1/2 mt-1.5",
  } as const;
  return (
    <span
      ref={ref}
      className={`rr-tooltip group relative inline-flex ${className}`}
      {...rest}
    >
      {children}
      <span
        role="tooltip"
        className={
          `pointer-events-none absolute z-50 whitespace-nowrap px-2 py-1 text-xs ` +
          "bg-bg-tertiary text-text-primary " +
          "rounded-[var(--radius-sm)] border-[length:var(--border-width)] border-border-default " +
          "[box-shadow:var(--shadow-card)] " +
          "opacity-0 group-hover:opacity-100 group-focus-within:opacity-100 " +
          "transition-opacity duration-[var(--motion-fast)] ease-[var(--motion-easing)] " +
          sides[side]
        }
      >
        {content}
      </span>
    </span>
  );
});
