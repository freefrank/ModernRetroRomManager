export const TOKEN_KEYS = [
  "bg-primary","bg-secondary","bg-tertiary",
  "accent-primary","accent-secondary","accent-success","accent-warning","accent-error",
  "text-primary","text-secondary","text-muted",
  "border-default","border-hover","border-highlight",
  "font-display","font-body","font-mono",
  "radius-sm","radius-md","radius-lg",
  "border-width","shadow-card","shadow-dialog","glow-accent",
  "motion-fast","motion-normal","motion-easing",
] as const;
export type TokenKey = (typeof TOKEN_KEYS)[number];
export type ThemeTokens = Record<TokenKey, string>;

export const EFFECT_SLOTS = ["backdrop","cardHover","buttonPress","pageTransition","focusRing"] as const;
export type EffectSlot = (typeof EFFECT_SLOTS)[number];

export const EFFECT_NAMES = ["scanlines","crt-flicker","hard-shift","pixel-jitter","neon-pulse","glitch-text","gradient-border","soft-glow","fade-scale","none"] as const;
export type EffectName = (typeof EFFECT_NAMES)[number];

export interface EffectConfig { name: EffectName; opacity?: number; color?: string; }

export interface ThemeManifest {
  schemaVersion: 1;
  id: string;
  name: string;
  author?: string;
  tokens: Partial<ThemeTokens>;
  effects?: Partial<Record<EffectSlot, EffectConfig>>;
}

export interface LoadedTheme extends ThemeManifest {
  builtin: boolean;
  /** 导入主题:解压目录绝对路径(assets 解析用);内置为 undefined */
  dir?: string;
  /** 导入主题:清洗后的自定义 CSS 内容 */
  customCss?: string;
}

export type MotionLevel = "off" | "low" | "full";
