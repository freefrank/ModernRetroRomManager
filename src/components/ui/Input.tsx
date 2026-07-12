import { forwardRef, type InputHTMLAttributes } from "react";

type Props = InputHTMLAttributes<HTMLInputElement>;

export const Input = forwardRef<HTMLInputElement, Props>(function Input(
  { className = "", ...rest },
  ref,
) {
  return (
    <input
      ref={ref}
      className={
        "rr-input h-10 w-full px-3 text-sm bg-bg-primary text-text-primary " +
        "placeholder:text-text-muted " +
        "rounded-[var(--radius-md)] border-[length:var(--border-width)] border-border-default " +
        "transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)] " +
        "hover:border-border-hover focus:border-border-highlight focus-visible:outline-none " +
        "disabled:opacity-50 disabled:pointer-events-none " +
        className
      }
      {...rest}
    />
  );
});
