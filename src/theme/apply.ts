import { BUILTIN_THEMES, DEFAULT_THEME_ID } from "./registry";
import { EFFECT_SLOTS, type LoadedTheme, type MotionLevel, type ThemeTokens } from "./types";

const STYLE_EL_ID = "rr-theme-pack-css";
const defaultTokens = (): ThemeTokens =>
  ({ ...(BUILTIN_THEMES.find(t => t.id === DEFAULT_THEME_ID)!.tokens as ThemeTokens) });

export function applyTheme(theme: LoadedTheme, motion: MotionLevel): void {
  const root = document.documentElement;
  const tokens = { ...defaultTokens(), ...theme.tokens }; // 缺失令牌回退默认主题
  for (const [k, v] of Object.entries(tokens)) root.style.setProperty(`--${k}`, v);

  root.dataset.theme = theme.id;
  root.dataset.motion = motion;
  for (const slot of EFFECT_SLOTS) {
    const cfg = theme.effects?.[slot];
    root.dataset[`fx${slot.charAt(0).toUpperCase()}${slot.slice(1).toLowerCase()}`] = cfg?.name ?? "none";
  }
  root.style.setProperty("--fx-backdrop-opacity", String(theme.effects?.backdrop?.opacity ?? 0.05));

  // 导入主题的自定义 CSS 层(内置主题无)
  let styleEl = document.getElementById(STYLE_EL_ID) as HTMLStyleElement | null;
  if (theme.customCss) {
    if (!styleEl) { styleEl = document.createElement("style"); styleEl.id = STYLE_EL_ID; document.head.appendChild(styleEl); }
    styleEl.textContent = theme.customCss;
  } else styleEl?.remove();
}
