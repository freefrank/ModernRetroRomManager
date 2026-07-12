import { useAppStore } from "@/stores/appStore";
import { resolveTheme } from "./registry";

/** 主题背景叠加层:单个 fixed 元素,效果名由当前主题的 backdrop 插槽决定 */
export default function Backdrop() {
  const themeId = useAppStore((s) => s.themeId);
  const customThemes = useAppStore((s) => s.customThemes);
  const theme = resolveTheme(themeId, customThemes);
  const fx = theme.effects?.backdrop?.name ?? "none";
  if (fx === "none") return null;
  return <div className="rr-backdrop" data-fx={fx} aria-hidden="true" />;
}
