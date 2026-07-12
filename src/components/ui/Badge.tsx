import { forwardRef, type HTMLAttributes } from "react";

interface Props extends HTMLAttributes<HTMLSpanElement> {
  variant?: "default" | "success" | "warning" | "error" | "info";
}

export const Badge = forwardRef<HTMLSpanElement, Props>(function Badge(
  { variant = "default", className = "", children, ...rest },
  ref,
) {
  const base =
    "rr-badge inline-flex items-center gap-1 px-2 py-0.5 text-xs font-medium leading-tight " +
    "rounded-[var(--radius-sm)] border-[length:var(--border-width)]";
  const variants = {
    default: "border-border-default bg-bg-tertiary text-text-secondary",
    success: "border-accent-success/40 bg-accent-success/10 text-accent-success",
    warning: "border-accent-warning/40 bg-accent-warning/10 text-accent-warning",
    error: "border-accent-error/40 bg-accent-error/10 text-accent-error",
    info: "border-accent-primary/40 bg-accent-primary/10 text-accent-primary",
  } as const;
  return (
    <span ref={ref} className={`${base} ${variants[variant]} ${className}`} {...rest}>
      {children}
    </span>
  );
});
