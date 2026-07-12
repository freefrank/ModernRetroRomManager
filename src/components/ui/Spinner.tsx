import { forwardRef, type SVGAttributes } from "react";
import { useTranslation } from "react-i18next";

type Props = SVGAttributes<SVGSVGElement> & {
  /** 像素尺寸(宽高) */
  size?: number;
};

export const Spinner = forwardRef<SVGSVGElement, Props>(function Spinner(
  { size = 20, className = "", ...rest },
  ref,
) {
  const { t } = useTranslation();
  return (
    <svg
      ref={ref}
      role="status"
      aria-label={t("ui.spinner.loading")}
      width={size}
      height={size}
      viewBox="0 0 24 24"
      fill="none"
      className={`rr-spinner animate-spin ${className}`}
      {...rest}
    >
      <circle
        cx="12"
        cy="12"
        r="10"
        stroke="currentColor"
        strokeWidth="3"
        className="opacity-25"
      />
      <path
        d="M22 12a10 10 0 0 1-10 10"
        stroke="currentColor"
        strokeWidth="3"
        strokeLinecap="round"
      />
    </svg>
  );
});
