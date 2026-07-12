import { forwardRef, type HTMLAttributes } from "react";
import { create } from "zustand";
import { CheckCircle2, AlertCircle, Info, X } from "lucide-react";
import { useTranslation } from "react-i18next";

export type ToastType = "success" | "error" | "info";

export interface ToastItem {
  id: number;
  type: ToastType;
  message: string;
}

interface ToastState {
  toasts: ToastItem[];
  push: (type: ToastType, message: string) => number;
  dismiss: (id: number) => void;
}

const TOAST_DURATION_MS = 4000;
let nextToastId = 1;

export const useToastStore = create<ToastState>((set) => ({
  toasts: [],
  push: (type, message) => {
    const id = nextToastId++;
    set((s) => ({ toasts: [...s.toasts, { id, type, message }] }));
    setTimeout(() => {
      set((s) => ({ toasts: s.toasts.filter((item) => item.id !== id) }));
    }, TOAST_DURATION_MS);
    return id;
  },
  dismiss: (id) => set((s) => ({ toasts: s.toasts.filter((item) => item.id !== id) })),
}));

/** 稳定的 toast API,可在组件内外调用 */
export const toast = {
  success: (message: string) => useToastStore.getState().push("success", message),
  error: (message: string) => useToastStore.getState().push("error", message),
  info: (message: string) => useToastStore.getState().push("info", message),
};

/** 组件内使用:const toast = useToast(); toast.success("...") */
export function useToast() {
  return toast;
}

interface ToastProps extends HTMLAttributes<HTMLDivElement> {
  type?: ToastType;
  onClose?: () => void;
}

const icons = {
  success: CheckCircle2,
  error: AlertCircle,
  info: Info,
} as const;

export const Toast = forwardRef<HTMLDivElement, ToastProps>(function Toast(
  { type = "info", onClose, className = "", children, ...rest },
  ref,
) {
  const { t } = useTranslation();
  const variants = {
    success: "border-accent-success/50",
    error: "border-accent-error/50",
    info: "border-accent-primary/50",
  } as const;
  const iconColors = {
    success: "text-accent-success",
    error: "text-accent-error",
    info: "text-accent-primary",
  } as const;
  const Icon = icons[type];
  return (
    <div
      ref={ref}
      role="alert"
      className={
        `rr-toast flex items-center gap-2.5 min-w-56 max-w-96 px-4 py-3 text-sm ` +
        "bg-bg-secondary text-text-primary " +
        "rounded-[var(--radius-md)] border-[length:var(--border-width)] " +
        "[box-shadow:var(--shadow-card)] " +
        "transition-all duration-[var(--motion-fast)] ease-[var(--motion-easing)] " +
        `${variants[type]} ${className}`
      }
      {...rest}
    >
      <Icon className={`w-4 h-4 shrink-0 ${iconColors[type]}`} />
      <div className="flex-1 break-words">{children}</div>
      {onClose && (
        <button
          type="button"
          aria-label={t("ui.toast.dismiss")}
          onClick={onClose}
          className={
            "shrink-0 text-text-muted hover:text-text-primary " +
            "transition-colors duration-[var(--motion-fast)] ease-[var(--motion-easing)]"
          }
        >
          <X className="w-4 h-4" />
        </button>
      )}
    </div>
  );
});

/** 全局单例,挂在 Layout 根部;右下角堆叠 */
export function Toaster() {
  const toasts = useToastStore((s) => s.toasts);
  const dismiss = useToastStore((s) => s.dismiss);
  return (
    <div className="fixed bottom-4 right-4 z-[100] flex flex-col items-end gap-2 pointer-events-none">
      {toasts.map((item) => (
        <Toast
          key={item.id}
          type={item.type}
          onClose={() => dismiss(item.id)}
          className="pointer-events-auto"
        >
          {item.message}
        </Toast>
      ))}
    </div>
  );
}
