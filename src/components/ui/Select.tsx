import { forwardRef, type SelectHTMLAttributes } from "react";

type Props = SelectHTMLAttributes<HTMLSelectElement>;

export const Select = forwardRef<HTMLSelectElement, Props>(function Select(
  { className = "", children, ...rest },
  ref,
) {
  return (
    <select
      ref={ref}
      className={
        "rr-select h-10 w-full px-3 text-sm bg-bg-primary text-text-primary " +
        "rounded-[var(--radius-md)] border-[length:var(--border-width)] border-border-default " +
        "transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)] " +
        "hover:border-border-hover focus:border-border-highlight focus-visible:outline-none " +
        "disabled:opacity-50 disabled:pointer-events-none " +
        className
      }
      {...rest}
    >
      {children}
    </select>
  );
});
