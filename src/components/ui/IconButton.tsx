import { forwardRef, type ButtonHTMLAttributes } from "react";

interface Props extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "ghost" | "primary" | "danger";
  size?: "sm" | "md";
}

export const IconButton = forwardRef<HTMLButtonElement, Props>(function IconButton(
  { variant = "ghost", size = "md", className = "", children, ...rest },
  ref,
) {
  const base =
    "rr-icon-button inline-flex items-center justify-center select-none shrink-0 " +
    "rounded-[var(--radius-md)] border-[length:var(--border-width)] " +
    "transition-all duration-[var(--motion-fast)] ease-[var(--motion-easing)] " +
    "disabled:opacity-50 disabled:pointer-events-none focus-visible:outline-none " +
    "focus-visible:border-border-highlight";
  const variants = {
    ghost:
      "bg-transparent text-text-secondary border-transparent hover:text-text-primary hover:bg-bg-tertiary",
    primary: "bg-accent-primary text-bg-primary border-transparent hover:brightness-110",
    danger:
      "bg-transparent text-accent-error border-transparent hover:bg-accent-error/10",
  } as const;
  const sizes = { sm: "h-8 w-8", md: "h-10 w-10" } as const;
  return (
    <button
      ref={ref}
      type="button"
      className={`${base} ${variants[variant]} ${sizes[size]} ${className}`}
      {...rest}
    >
      {children}
    </button>
  );
});
