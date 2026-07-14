import { forwardRef, type ButtonHTMLAttributes } from "react";
import { Spinner } from "./Spinner";

interface Props extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "ghost" | "danger";
  size?: "sm" | "md";
  loading?: boolean;
}

export const Button = forwardRef<HTMLButtonElement, Props>(function Button(
  { variant = "primary", size = "md", loading, className = "", children, disabled, ...rest }, ref) {
  const base = "rr-button inline-flex items-center justify-center gap-2 whitespace-nowrap font-medium select-none [&>svg]:shrink-0 " +
    "rounded-[var(--radius-md)] border-[length:var(--border-width)] " +
    "transition-all duration-[var(--motion-fast)] ease-[var(--motion-easing)] " +
    "disabled:opacity-50 disabled:pointer-events-none focus-visible:outline-none " +
    "focus-visible:border-border-highlight";
  const variants = {
    primary: "bg-accent-primary text-bg-primary border-transparent hover:brightness-110",
    ghost: "bg-transparent text-text-secondary border-border-default hover:text-text-primary hover:border-border-hover",
    danger: "bg-transparent text-accent-error border-accent-error hover:bg-accent-error hover:text-bg-primary",
  } as const;
  const sizes = { sm: "h-8 px-3 text-sm", md: "h-10 px-4 text-base" } as const;
  return (
    <button ref={ref} disabled={disabled || loading}
      className={`${base} ${variants[variant]} ${sizes[size]} ${className}`} {...rest}>
      {loading && <Spinner size={16} />}{children}
    </button>
  );
});
