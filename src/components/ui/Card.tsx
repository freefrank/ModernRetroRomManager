import { forwardRef, type HTMLAttributes } from "react";

type Props = HTMLAttributes<HTMLDivElement>;

export const Card = forwardRef<HTMLDivElement, Props>(function Card(
  { className = "", children, ...rest },
  ref,
) {
  return (
    <div
      ref={ref}
      className={
        "rr-card bg-bg-secondary text-text-primary " +
        "rounded-[var(--radius-lg)] border-[length:var(--border-width)] border-border-default " +
        "[box-shadow:var(--shadow-card)] " +
        "transition-all duration-[var(--motion-fast)] ease-[var(--motion-easing)] " +
        className
      }
      {...rest}
    >
      {children}
    </div>
  );
});
