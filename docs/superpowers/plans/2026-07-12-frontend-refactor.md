# 前端重构实施计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 按设计文档 `docs/superpowers/specs/2026-07-12-frontend-refactor-design.md` 重构前端:主题包体系(4 内置 + 可导入 `.rrtheme`)、UI 基件库、Library 系统树+货架布局、纯前端错误修复、Scraper 配置闭环。

**Architecture:** 主题即数据(JSON 清单 + CSS 变量运行时应用 + 效果目录 + 可选自定义 CSS 层);统一 UI 基件消费令牌;波次推进,波内并行 subagent,波间 PM 验收。

**Tech Stack:** React 19 + TypeScript + Tailwind v4 (`@theme` 映射 CSS 变量) + Zustand + i18next + Tauri v2 (Rust) + Vitest(新增)+ ESLint 9(新增)

**执行约定(PM 必读):**

- 波内并行任务如触碰同一文件即改为串行;Wave 3 三个页面任务已通过 i18n 按页分文件(Task 5)保证文件不相交
- 每个任务的最后一步都是「验收自检 + 中文 commit」;PM 在波结束后运行全量门禁:`pnpm tsc --noEmit && pnpm lint && pnpm build`(Rust 任务另加 `cargo clippy`、`cargo test`)
- UI 里程碑(Wave 3 / 4 / 5 末)PM 用交叉编译 exe + Windows 互操作截图核对四主题渲染(流程见项目 memory `windows-portable-build`)
- 现有 8 套旧主题类将被移除;`settings.theme` 旧值/未知值一律回退 `retro-arcade`
- **禁止**:硬编码颜色/圆角/阴影(必须走令牌)、新增英文 UI 文案(必须走 i18n)、`git push`

---

## 令牌与效果契约(所有任务的共同基准)

### 令牌全集(theme.json `tokens` 键名 → CSS 变量 `--<键名>`)

颜色(沿用现有变量名,页面无需大改):
`bg-primary` `bg-secondary` `bg-tertiary` `accent-primary` `accent-secondary` `accent-success` `accent-warning` `accent-error` `text-primary` `text-secondary` `text-muted` `border-default` `border-hover` `border-highlight`

新增——字体:`font-display` `font-body` `font-mono`;形状:`radius-sm` `radius-md` `radius-lg`;质感:`border-width` `shadow-card` `shadow-dialog` `glow-accent`;动效:`motion-fast` `motion-normal` `motion-easing`

### 效果插槽与取值

| 插槽(theme.json `effects` 键) | html 属性 | 可选效果名 |
|---|---|---|
| `backdrop` | `data-fx-backdrop` | `scanlines` `none` |
| `cardHover` | `data-fx-cardhover` | `hard-shift` `pixel-jitter` `neon-pulse` `soft-glow` `fade-scale` `gradient-border` `none` |
| `buttonPress` | `data-fx-buttonpress` | `hard-shift` `fade-scale` `none` |
| `pageTransition` | `data-fx-pagetransition` | `crt-flicker` `fade-scale` `none` |
| `focusRing` | `data-fx-focusring` | `neon-pulse` `gradient-border` `none` |

动效开关:`<html data-motion="off|low|full">`。`off` = 禁全部 animation/transition;`low` = 仅 transition;`full` = 全开。效果 CSS 一律写成 `[data-motion="full"][data-fx-cardhover="neon-pulse"] .rr-card:hover { … }` 形式(backdrop 静态纹理在 low 下保留,动画在 full 下才有)。

### UI 基件稳定类名钩子(供主题包 style.css 定点覆盖)

`rr-button` `rr-icon-button` `rr-card` `rr-input` `rr-select` `rr-dialog` `rr-toast` `rr-empty` `rr-spinner` `rr-badge` `rr-tabs` `rr-tooltip` `rr-sidebar` `rr-statusbar` `rr-topbar`

---

# Wave 0:协作基建(2 任务,可并行)

## Task 1: 子代理定义与项目 skills

**Files:**
- Create: `.claude/agents/frontend-impl.md`、`.claude/agents/rust-impl.md`、`.claude/agents/qa-verify.md`
- Create: `.claude/skills/theme-tokens/SKILL.md`、`.claude/skills/frontend-verify/SKILL.md`

- [ ] **Step 1: 写三个子代理定义**

`.claude/agents/frontend-impl.md`:

```markdown
---
name: frontend-impl
description: 前端实现专员。实现 React/TypeScript/Tailwind 组件与页面,严格遵循主题令牌体系。用于本项目前端重构的编码任务。
tools: Read, Edit, Write, Bash, Grep, Glob
---

你是本项目的前端实现专员。开工前必读:
1. `docs/superpowers/specs/2026-07-12-frontend-refactor-design.md`(设计契约)
2. `.claude/skills/theme-tokens/SKILL.md`(令牌使用铁律)

铁律:
- 颜色/圆角/阴影/字体/动效时长一律走令牌(Tailwind 类或 `var(--…)`),禁止硬编码
- UI 基件必须带 `rr-*` 稳定类名;所有用户可见文案走 i18n(简体中文为主语言)
- 只做任务范围内的事,不顺手重构;交付前运行 `pnpm tsc --noEmit` 与 `pnpm lint` 确保零错误
- 完成后用中文 commit message 提交,禁止 push
```

`.claude/agents/rust-impl.md`:

```markdown
---
name: rust-impl
description: Tauri Rust 后端实现专员。实现 Tauri commands、主题包导入、Scraper 配置持久化。用于本项目 Rust 侧编码任务。
tools: Read, Edit, Write, Bash, Grep, Glob
---

你是本项目的 Tauri 后端实现专员。开工前必读设计文档
`docs/superpowers/specs/2026-07-12-frontend-refactor-design.md` 的相关章节。

铁律:
- 新命令必须注册进 `src-tauri/src/lib.rs` 的 `invoke_handler`
- 错误信息面向用户的部分用简体中文
- 交付前 `cargo fmt && cargo clippy -- -D warnings && cargo test` 全绿
- 完成后用中文 commit message 提交,禁止 push
```

`.claude/agents/qa-verify.md`:

```markdown
---
name: qa-verify
description: 验收专员。按验收清单核查前端/Rust 交付物,只读代码与运行检查,不修改产品代码。
tools: Read, Bash, Grep, Glob
---

你是验收专员,按 `.claude/skills/frontend-verify/SKILL.md` 的清单逐项核查并输出验收报告
(通过项/不通过项/证据)。你不修改产品代码;发现问题只报告,由 PM 决定返工。
```

- [ ] **Step 2: 写 theme-tokens skill**

`.claude/skills/theme-tokens/SKILL.md`:

```markdown
---
name: theme-tokens
description: 本项目主题令牌体系使用规范。写任何带样式的前端代码前必读。
---

# 主题令牌使用规范

## 令牌全集
颜色:bg-primary/bg-secondary/bg-tertiary、accent-primary/accent-secondary、
accent-success/warning/error、text-primary/secondary/muted、
border-default/hover/highlight
字体:font-display(标题)/font-body(正文)/font-mono(等宽)
形状:radius-sm/md/lg  质感:border-width、shadow-card、shadow-dialog、glow-accent
动效:motion-fast、motion-normal、motion-easing

## 用法
- Tailwind 类:`bg-bg-secondary` `text-text-primary` `border-border-default`
  `rounded-[var(--radius-md)]`(圆角/阴影没有映射类,用任意值语法或 CSS)
- CSS:`var(--shadow-card)` `var(--motion-normal)` 等
- 过渡动画:`transition-... duration-[var(--motion-normal)] ease-[var(--motion-easing)]`

## 铁律
1. 禁止出现字面量颜色(#hex、rgb()、颜色名),包括 Tailwind 调色板类(bg-slate-800 等)
2. 禁止字面量圆角/阴影/动画时长;需要新令牌先报告 PM,不擅自加
3. UI 基件根元素必须带对应 rr-* 类名(清单见实施计划「令牌与效果契约」节)
4. 效果插槽由主题决定,组件不感知主题:hover 光效等交给
   `[data-fx-*]` 属性选择器驱动的效果 CSS,组件只挂 rr-* 类
5. 例外白名单:src/theme/**(主题定义本身)、src/index.css 的令牌声明区
```

- [ ] **Step 3: 写 frontend-verify skill**

`.claude/skills/frontend-verify/SKILL.md`:

```markdown
---
name: frontend-verify
description: 前端交付验收清单。每波结束或验收任务时执行。
---

# 验收清单

## 静态门禁(全部必须零错误)
1. `pnpm tsc --noEmit`
2. `pnpm lint`
3. `pnpm build`
4. 硬编码颜色扫描(白名单外必须零命中):
   `rg -n '#[0-9a-fA-F]{3,8}\b|rgb\(' src --glob '!src/theme/**' --glob '!src/index.css' --glob '!src/vite-env.d.ts' -g '!*.json'`
5. 英文硬编码抽查:`rg -n '"[A-Z][a-z]+ [A-Za-z ]+"' src/pages src/components -g '!*.test.*'`
   命中处逐一确认是否用户可见文案(可见即违规,须走 i18n)

## Rust 门禁(涉及 Rust 的波次)
6. `cd src-tauri && cargo fmt --check && cargo clippy -- -D warnings && cargo test`

## 单元测试
7. `pnpm vitest run`

## 视觉核对(UI 里程碑,由 PM 执行)
8. 交叉编译 + 互操作启动 + 逐主题截图(4 套),核对:令牌生效、效果插槽表现、
   无未样式化元素、动效开关三档行为
```

- [ ] **Step 4: 提交**

```bash
git add .claude/agents .claude/skills
git commit -m "新增前端重构子代理定义与项目技能规范"
```

## Task 2: 质量门禁(ESLint 9 + Vitest)

**Files:**
- Create: `eslint.config.js`、`vitest.config.ts`
- Modify: `package.json`(devDependencies + scripts)

- [ ] **Step 1: 安装依赖**

```bash
pnpm add -D eslint@9 typescript-eslint eslint-plugin-react-hooks eslint-plugin-react-refresh globals vitest @vitest/coverage-v8 jsdom
```

- [ ] **Step 2: 写 `eslint.config.js`**

```js
import js from "@eslint/js";
import tseslint from "typescript-eslint";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import globals from "globals";

export default tseslint.config(
  { ignores: ["dist", "src-tauri", "server", "node_modules", "scripts"] },
  {
    files: ["src/**/*.{ts,tsx}"],
    extends: [js.configs.recommended, ...tseslint.configs.recommended],
    languageOptions: { globals: globals.browser },
    plugins: { "react-hooks": reactHooks, "react-refresh": reactRefresh },
    rules: {
      ...reactHooks.configs.recommended.rules,
      "react-refresh/only-export-components": "off",
      "@typescript-eslint/no-unused-vars": ["error", { argsIgnorePattern: "^_" }],
    },
  }
);
```

- [ ] **Step 3: 写 `vitest.config.ts`**

```ts
import { defineConfig } from "vitest/config";
import path from "node:path";

export default defineConfig({
  resolve: { alias: { "@": path.resolve(__dirname, "src") } },
  test: { environment: "jsdom", include: ["src/**/*.test.{ts,tsx}"] },
});
```

- [ ] **Step 4: 加 scripts 并跑通**

`package.json` scripts 增加:`"lint": "eslint src"`、`"test": "vitest run"`。
运行 `pnpm lint`;对既有代码的报错逐个最小修复(未使用变量删除、依赖数组补全等),**不做行为改动**。运行 `pnpm tsc --noEmit && pnpm build` 确认无回归。

- [ ] **Step 5: 提交**

```bash
git add -A && git commit -m "引入 ESLint 9 与 Vitest 质量门禁并清零存量告警"
```

---

# Wave 1:主题地基(1 任务,串行——所有后续工作的依赖)

## Task 3: 主题包体系核心

**Files:**
- Create: `src/theme/types.ts`、`src/theme/validate.ts`、`src/theme/validate.test.ts`、`src/theme/apply.ts`、`src/theme/registry.ts`、`src/theme/effects.css`、`src/theme/Backdrop.tsx`
- Create: `src/theme/builtin/retro-arcade.json`、`modern-dark.json`、`cyberpunk.json`、`violet.json`
- Create: `src/assets/fonts/press-start-2p-latin.woff2`(下载)
- Modify: `src/index.css`(重写)、`src/stores/appStore.ts`、`src/components/layout/Layout.tsx`(挂 Backdrop)
- Modify: `src/pages/Settings.tsx`(仅最小适配保编译,重构在 Task 7)

- [ ] **Step 1: 定义类型 `src/theme/types.ts`**

```ts
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
```

- [ ] **Step 2: TDD 校验器——先写 `src/theme/validate.test.ts`**

```ts
import { describe, it, expect } from "vitest";
import { validateManifest } from "./validate";

const ok = { schemaVersion: 1, id: "t", name: "T", tokens: { "bg-primary": "#000" } };

describe("validateManifest", () => {
  it("接受最小合法清单", () => {
    expect(validateManifest(ok).ok).toBe(true);
  });
  it("拒绝错误 schemaVersion / 缺 id / 缺 tokens", () => {
    expect(validateManifest({ ...ok, schemaVersion: 2 }).ok).toBe(false);
    expect(validateManifest({ ...ok, id: "" }).ok).toBe(false);
    expect(validateManifest({ ...ok, tokens: undefined }).ok).toBe(false);
  });
  it("剔除未知令牌键并告警,未知效果名回退 none", () => {
    const r = validateManifest({ ...ok, tokens: { "bg-primary": "#000", evil: "x" },
      effects: { cardHover: { name: "not-exist" } } });
    expect(r.ok).toBe(true);
    expect(r.manifest!.tokens).not.toHaveProperty("evil");
    expect(r.manifest!.effects!.cardHover!.name).toBe("none");
    expect(r.warnings.length).toBeGreaterThan(0);
  });
  it("拒绝令牌值中的外部 url 与脚本向量", () => {
    expect(validateManifest({ ...ok, tokens: { "bg-primary": "url(http://x)" } }).ok).toBe(false);
    expect(validateManifest({ ...ok, tokens: { "bg-primary": "expression(a)" } }).ok).toBe(false);
  });
});
```

- [ ] **Step 3: 运行确认失败,再实现 `src/theme/validate.ts` 使其通过**

`pnpm vitest run src/theme` 先 FAIL(模块不存在)。实现:

```ts
import { TOKEN_KEYS, EFFECT_SLOTS, EFFECT_NAMES, type ThemeManifest, type TokenKey, type EffectSlot } from "./types";

export interface ValidateResult { ok: boolean; manifest?: ThemeManifest; warnings: string[]; error?: string; }

const FORBIDDEN = /url\s*\(\s*['"]?\s*(https?:|\/\/)|expression\s*\(|javascript:|@import/i;

export function validateManifest(raw: unknown): ValidateResult {
  const warnings: string[] = [];
  const m = raw as Record<string, unknown>;
  if (!m || typeof m !== "object") return { ok: false, warnings, error: "清单不是对象" };
  if (m.schemaVersion !== 1) return { ok: false, warnings, error: "不支持的 schemaVersion" };
  if (typeof m.id !== "string" || !/^[a-z0-9][a-z0-9-]*$/.test(m.id)) return { ok: false, warnings, error: "id 非法" };
  if (typeof m.name !== "string" || !m.name) return { ok: false, warnings, error: "缺少 name" };
  if (!m.tokens || typeof m.tokens !== "object") return { ok: false, warnings, error: "缺少 tokens" };

  const tokens: Partial<Record<TokenKey, string>> = {};
  for (const [k, v] of Object.entries(m.tokens as Record<string, unknown>)) {
    if (!(TOKEN_KEYS as readonly string[]).includes(k)) { warnings.push(`未知令牌 ${k} 已忽略`); continue; }
    if (typeof v !== "string") { warnings.push(`令牌 ${k} 值非字符串,已忽略`); continue; }
    if (FORBIDDEN.test(v)) return { ok: false, warnings, error: `令牌 ${k} 含被禁止的内容` };
    tokens[k as TokenKey] = v;
  }

  const effects: ThemeManifest["effects"] = {};
  if (m.effects && typeof m.effects === "object") {
    for (const [slot, cfg] of Object.entries(m.effects as Record<string, { name?: string; opacity?: number; color?: string }>)) {
      if (!(EFFECT_SLOTS as readonly string[]).includes(slot)) { warnings.push(`未知插槽 ${slot} 已忽略`); continue; }
      const name = (EFFECT_NAMES as readonly string[]).includes(cfg?.name ?? "") ? cfg!.name! : "none";
      if (name !== cfg?.name) warnings.push(`插槽 ${slot} 引用未知效果,已回退 none`);
      effects[slot as EffectSlot] = { name: name as never, opacity: cfg?.opacity, color: cfg?.color };
    }
  }
  return { ok: true, warnings, manifest: { schemaVersion: 1, id: m.id, name: m.name, author: typeof m.author === "string" ? m.author : undefined, tokens, effects } };
}
```

`pnpm vitest run src/theme` → PASS。

- [ ] **Step 4: 四套内置主题 JSON(完整值,不许省略)**

`src/theme/builtin/retro-arcade.json`:

```json
{
  "schemaVersion": 1, "id": "retro-arcade", "name": "复古游戏厅", "author": "built-in",
  "tokens": {
    "bg-primary": "#1b1b2f", "bg-secondary": "#24243e", "bg-tertiary": "#2e2e4e",
    "accent-primary": "#e94560", "accent-secondary": "#f0a500",
    "accent-success": "#4ade80", "accent-warning": "#f0a500", "accent-error": "#e94560",
    "text-primary": "#ffffff", "text-secondary": "#9090a8", "text-muted": "#5a5a70",
    "border-default": "#3a3a5c", "border-hover": "#533483", "border-highlight": "#e94560",
    "font-display": "'Press Start 2P', 'Segoe UI', 'Microsoft YaHei', sans-serif",
    "font-body": "-apple-system, 'Segoe UI', 'Microsoft YaHei', Roboto, sans-serif",
    "font-mono": "Consolas, 'Courier New', monospace",
    "radius-sm": "0px", "radius-md": "0px", "radius-lg": "2px",
    "border-width": "2px",
    "shadow-card": "3px 3px 0 #533483", "shadow-dialog": "6px 6px 0 #16162a",
    "glow-accent": "none",
    "motion-fast": "100ms", "motion-normal": "200ms", "motion-easing": "steps(3, end)"
  },
  "effects": {
    "backdrop": { "name": "scanlines", "opacity": 0.05 },
    "cardHover": { "name": "hard-shift" },
    "buttonPress": { "name": "hard-shift" },
    "pageTransition": { "name": "crt-flicker" },
    "focusRing": { "name": "none" }
  }
}
```

`src/theme/builtin/modern-dark.json`:

```json
{
  "schemaVersion": 1, "id": "modern-dark", "name": "现代极简", "author": "built-in",
  "tokens": {
    "bg-primary": "#101014", "bg-secondary": "#17171c", "bg-tertiary": "#1f1f26",
    "accent-primary": "#4f8cff", "accent-secondary": "#8a8a94",
    "accent-success": "#34d399", "accent-warning": "#fbbf24", "accent-error": "#f87171",
    "text-primary": "#f2f2f5", "text-secondary": "#a0a0aa", "text-muted": "#5c5c66",
    "border-default": "rgba(255,255,255,0.07)", "border-hover": "rgba(255,255,255,0.14)", "border-highlight": "rgba(79,140,255,0.55)",
    "font-display": "-apple-system, 'Segoe UI', 'Microsoft YaHei', Roboto, sans-serif",
    "font-body": "-apple-system, 'Segoe UI', 'Microsoft YaHei', Roboto, sans-serif",
    "font-mono": "Consolas, 'Courier New', monospace",
    "radius-sm": "6px", "radius-md": "10px", "radius-lg": "16px",
    "border-width": "1px",
    "shadow-card": "0 2px 12px rgba(0,0,0,0.4)", "shadow-dialog": "0 12px 48px rgba(0,0,0,0.6)",
    "glow-accent": "none",
    "motion-fast": "120ms", "motion-normal": "240ms", "motion-easing": "cubic-bezier(0.2, 0.8, 0.2, 1)"
  },
  "effects": {
    "backdrop": { "name": "none" },
    "cardHover": { "name": "fade-scale" },
    "buttonPress": { "name": "fade-scale" },
    "pageTransition": { "name": "fade-scale" },
    "focusRing": { "name": "none" }
  }
}
```

`src/theme/builtin/cyberpunk.json`:

```json
{
  "schemaVersion": 1, "id": "cyberpunk", "name": "赛博霓虹", "author": "built-in",
  "tokens": {
    "bg-primary": "#0a0e17", "bg-secondary": "#0d1220", "bg-tertiary": "#141b30",
    "accent-primary": "#00e5ff", "accent-secondary": "#ff2a6d",
    "accent-success": "#05ffa1", "accent-warning": "#ffd700", "accent-error": "#ff2a6d",
    "text-primary": "#eaf6ff", "text-secondary": "#7a8db0", "text-muted": "#4a5a7a",
    "border-default": "rgba(0,229,255,0.18)", "border-hover": "rgba(0,229,255,0.4)", "border-highlight": "rgba(255,42,109,0.6)",
    "font-display": "'Segoe UI', 'Microsoft YaHei', sans-serif",
    "font-body": "-apple-system, 'Segoe UI', 'Microsoft YaHei', Roboto, sans-serif",
    "font-mono": "Consolas, 'Courier New', monospace",
    "radius-sm": "2px", "radius-md": "4px", "radius-lg": "8px",
    "border-width": "1px",
    "shadow-card": "0 0 12px rgba(0,229,255,0.12)", "shadow-dialog": "0 0 40px rgba(0,229,255,0.25)",
    "glow-accent": "0 0 10px rgba(0,229,255,0.6)",
    "motion-fast": "100ms", "motion-normal": "220ms", "motion-easing": "cubic-bezier(0.3, 0, 0.2, 1)"
  },
  "effects": {
    "backdrop": { "name": "scanlines", "opacity": 0.03 },
    "cardHover": { "name": "neon-pulse" },
    "buttonPress": { "name": "fade-scale" },
    "pageTransition": { "name": "fade-scale" },
    "focusRing": { "name": "neon-pulse" }
  }
}
```

`src/theme/builtin/violet.json`:

```json
{
  "schemaVersion": 1, "id": "violet", "name": "紫罗兰", "author": "built-in",
  "tokens": {
    "bg-primary": "#13111c", "bg-secondary": "#1a1726", "bg-tertiary": "#241f33",
    "accent-primary": "#8b5cf6", "accent-secondary": "#6366f1",
    "accent-success": "#10b981", "accent-warning": "#f59e0b", "accent-error": "#ef4444",
    "text-primary": "#f8fafc", "text-secondary": "#a29fb8", "text-muted": "#6b6780",
    "border-default": "rgba(139,92,246,0.14)", "border-hover": "rgba(139,92,246,0.3)", "border-highlight": "rgba(139,92,246,0.55)",
    "font-display": "-apple-system, 'Segoe UI', 'Microsoft YaHei', Roboto, sans-serif",
    "font-body": "-apple-system, 'Segoe UI', 'Microsoft YaHei', Roboto, sans-serif",
    "font-mono": "Consolas, 'Courier New', monospace",
    "radius-sm": "6px", "radius-md": "10px", "radius-lg": "14px",
    "border-width": "1px",
    "shadow-card": "0 4px 16px rgba(0,0,0,0.35)", "shadow-dialog": "0 12px 48px rgba(0,0,0,0.55)",
    "glow-accent": "0 0 14px rgba(139,92,246,0.45)",
    "motion-fast": "120ms", "motion-normal": "240ms", "motion-easing": "cubic-bezier(0.2, 0.8, 0.2, 1)"
  },
  "effects": {
    "backdrop": { "name": "none" },
    "cardHover": { "name": "soft-glow" },
    "buttonPress": { "name": "fade-scale" },
    "pageTransition": { "name": "fade-scale" },
    "focusRing": { "name": "gradient-border" }
  }
}
```

- [ ] **Step 5: 像素字体本地化**

```bash
mkdir -p src/assets/fonts
# 从 Google Fonts CSS API 解析 latin woff2 真实地址并下载(OFL 许可)
curl -sA "Mozilla/5.0" "https://fonts.googleapis.com/css2?family=Press+Start+2P&display=swap" \
  | grep -oE "https://[^)]+\.woff2" | head -1 | xargs -I{} curl -sL -o src/assets/fonts/press-start-2p-latin.woff2 {}
ls -la src/assets/fonts/   # 确认文件 > 10KB
```

- [ ] **Step 6: 重写 `src/index.css`**

删除 Google Fonts `@import` 与全部 8 个旧主题类(`.dark`/`.cyberpunk`/`.ocean`/`.forest`/`.sunset`/`.rose`/`.nord` 及 `:root` 光亮值)。保留并扩展:

```css
@import "tailwindcss";
@import "./theme/effects.css";

@font-face {
  font-family: "Press Start 2P";
  src: url("./assets/fonts/press-start-2p-latin.woff2") format("woff2");
  font-display: swap;
  unicode-range: U+0000-00FF;
}

@theme {
  --color-bg-primary: var(--bg-primary);
  --color-bg-secondary: var(--bg-secondary);
  --color-bg-tertiary: var(--bg-tertiary);
  --color-accent-primary: var(--accent-primary);
  --color-accent-secondary: var(--accent-secondary);
  --color-accent-success: var(--accent-success);
  --color-accent-warning: var(--accent-warning);
  --color-accent-error: var(--accent-error);
  --color-text-primary: var(--text-primary);
  --color-text-secondary: var(--text-secondary);
  --color-text-muted: var(--text-muted);
  --color-border-default: var(--border-default);
  --color-border-hover: var(--border-hover);
  --color-border-highlight: var(--border-highlight);
  --font-family-display: var(--font-display);
  --font-family-sans: var(--font-body);
  --font-family-mono: var(--font-mono);
  --radius-sm: var(--radius-sm);
  --radius-md: var(--radius-md);
  --radius-lg: var(--radius-lg);
}

/* 兜底:apply.ts 未运行时的最小可读值(取 retro-arcade 同值) */
:root {
  --bg-primary: #1b1b2f; --bg-secondary: #24243e; --bg-tertiary: #2e2e4e;
  /* …其余 24 个令牌同 retro-arcade.json,逐一列出… */
}

/* 动效三档全局门控 */
[data-motion="off"] *, [data-motion="off"] *::before, [data-motion="off"] *::after {
  animation: none !important; transition: none !important;
}

/* Global Styles(沿用原有 reset/scrollbar,body 字体改 var(--font-body)) */
```

(`:root` 兜底块必须把 retro-arcade 的 27 个令牌完整写死一遍,防白屏。)

- [ ] **Step 7: `src/theme/effects.css`(效果目录全量实现)**

要求:只用 `transform`/`opacity`/`filter`/`box-shadow`;全部受 `data-motion` 门控;按 `[data-fx-<slot>="<name>"]` 作用到 `rr-*` 钩子。完整实现 10 个效果,核心示例(其余同构):

```css
/* backdrop: scanlines(静态纹理 low 保留,滚动动画仅 full) */
.rr-backdrop[data-fx="scanlines"] {
  position: fixed; inset: 0; pointer-events: none; z-index: 9999;
  background: repeating-linear-gradient(0deg, rgba(0,0,0,0.35) 0 1px, transparent 1px 3px);
  opacity: var(--fx-backdrop-opacity, 0.05);
}

/* cardHover: hard-shift */
[data-motion]:not([data-motion="off"])[data-fx-cardhover="hard-shift"] .rr-card:hover {
  transform: translate(-2px, -2px);
  box-shadow: 5px 5px 0 var(--border-hover);
}
[data-fx-buttonpress="hard-shift"] .rr-button:active { transform: translate(2px, 2px); box-shadow: none; }

/* cardHover: neon-pulse(动画仅 full) */
[data-fx-cardhover="neon-pulse"] .rr-card:hover { box-shadow: var(--glow-accent); border-color: var(--accent-primary); }
[data-motion="full"][data-fx-cardhover="neon-pulse"] .rr-card:hover { animation: rr-neon-pulse 1.6s ease-in-out infinite; }
@keyframes rr-neon-pulse {
  0%,100% { box-shadow: var(--glow-accent); }
  50% { box-shadow: 0 0 4px var(--accent-primary); }
}

/* pageTransition: crt-flicker(仅 full;由页面容器 .rr-page 进场触发) */
[data-motion="full"][data-fx-pagetransition="crt-flicker"] .rr-page { animation: rr-crt 240ms steps(2, end); }
@keyframes rr-crt { 0% { opacity: 0.4; transform: scaleY(0.96); filter: brightness(1.6); } 100% { opacity: 1; transform: none; filter: none; } }
```

(`pixel-jitter`、`glitch-text`、`gradient-border`、`soft-glow`、`fade-scale` 按同样模式完整写出;`none` 无规则。)

- [ ] **Step 8: `src/theme/apply.ts` + `src/theme/registry.ts` + `src/theme/Backdrop.tsx`**

```ts
// registry.ts
import retroArcade from "./builtin/retro-arcade.json";
import modernDark from "./builtin/modern-dark.json";
import cyberpunk from "./builtin/cyberpunk.json";
import violet from "./builtin/violet.json";
import type { LoadedTheme, ThemeManifest } from "./types";

export const DEFAULT_THEME_ID = "retro-arcade";
export const BUILTIN_THEMES: LoadedTheme[] = [retroArcade, modernDark, cyberpunk, violet]
  .map(m => ({ ...(m as ThemeManifest), builtin: true }));

/** 内置 + 已导入(导入列表由调用方传入,Wave 2 接后端) */
export function resolveTheme(id: string | undefined, custom: LoadedTheme[] = []): LoadedTheme {
  const all = [...BUILTIN_THEMES, ...custom];
  return all.find(t => t.id === id) ?? all.find(t => t.id === DEFAULT_THEME_ID)!;
}
```

```ts
// apply.ts
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
```

注意 dataset 键与 CSS 属性选择器的大小写映射:`data-fx-cardhover` ↔ `dataset.fxCardhover`。`Backdrop.tsx` 渲染 `<div className="rr-backdrop" data-fx={activeBackdropName} />`,从 appStore 读当前主题,挂进 `Layout.tsx` 根节点。

- [ ] **Step 9: 改造 `src/stores/appStore.ts`**

- 删除 `ThemeMode`/`THEMES`/`applyThemeToDOM`(全项目 `rg` 确认引用点:`Settings.tsx` 一处,`Sidebar.tsx` 若有)
- 新状态:`themeId: string`(默认 `DEFAULT_THEME_ID`)、`motion: MotionLevel`(默认 `"full"`)、`customThemes: LoadedTheme[]`(本 wave 恒 `[]`)
- `setTheme(id)` → `applyTheme(resolveTheme(id, customThemes), motion)` + 持久化 `saveSettingToBackend("theme", id)`
- `setMotion(level)` → 重新 applyTheme + 持久化 `"motion_level"`
- `initFromBackend`:读 `settings.theme`,经 `resolveTheme` 解析(旧值 `dark`/`ocean` 等自然回退默认);读 `settings.motion_level`——无存储值时默认:`window.matchMedia("(prefers-reduced-motion: reduce)").matches ? "low" : "full"`(尊重系统减弱动效偏好,用户手动设置后以设置为准)
- `Settings.tsx` 最小适配:旧主题网格暂时改为遍历 `BUILTIN_THEMES`(`id`/`name`),调 `setTheme(id)`——只求编译通过与功能不断,视觉重构在 Task 7

- [ ] **Step 10: 验收自检 + 提交**

```bash
pnpm vitest run && pnpm tsc --noEmit && pnpm lint && pnpm build
pnpm dev  # PM 浏览器抽查:四主题切换即时生效、扫描线叠加层可见、动效三档行为正确
git add -A && git commit -m "建立主题包体系:令牌全集、效果目录与四套内置主题"
```

---

# Wave 2:基件与后端(3 任务,可并行;文件互不相交)

## Task 4: UI 基件库

**Files:**
- Create: `src/components/ui/Button.tsx`、`IconButton.tsx`、`Card.tsx`、`Input.tsx`、`Select.tsx`、`Dialog.tsx`、`Toast.tsx`(含 `useToast` + `<Toaster/>`)、`EmptyState.tsx`、`Spinner.tsx`、`Badge.tsx`、`Tabs.tsx`、`Tooltip.tsx`、`index.ts`
- Modify: `src/components/layout/Layout.tsx`(挂 `<Toaster/>`)

规范(每个组件必须满足):
1. 根元素带对应 `rr-*` 类 + `forwardRef` + 透传 `className`
2. 样式只用令牌类/变量;过渡统一 `duration-[var(--motion-fast)] ease-[var(--motion-easing)]`
3. 圆角 `rounded-[var(--radius-md)]`(卡片/弹窗 lg,徽章 sm),边框 `border-[length:var(--border-width)]`,卡片阴影 `[box-shadow:var(--shadow-card)]`
4. 变体用 props 表达。Button:`variant: "primary"|"ghost"|"danger"`、`size: "sm"|"md"`、`loading`;Dialog:受控 `open/onClose`、ESC 关闭、遮罩点击关闭、内部滚动(`max-h-[80vh] overflow-y-auto`)、标题插槽;Toast:`success|error|info` 三型,4s 自动消失,右下角堆叠
5. 每个组件写最小 Vitest 冒烟测试(渲染 + rr-* 类存在 + 变体 class 差异),放同目录 `*.test.tsx`

代表实现(Button,其余组件同标准):

```tsx
import { forwardRef, type ButtonHTMLAttributes } from "react";
import { Spinner } from "./Spinner";

interface Props extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: "primary" | "ghost" | "danger";
  size?: "sm" | "md";
  loading?: boolean;
}

export const Button = forwardRef<HTMLButtonElement, Props>(function Button(
  { variant = "primary", size = "md", loading, className = "", children, disabled, ...rest }, ref) {
  const base = "rr-button inline-flex items-center justify-center gap-2 font-medium select-none " +
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
```

- [ ] Step 1: 按上述规范实现全部 12 个基件 + index.ts 汇出
- [ ] Step 2: 每件冒烟测试,`pnpm vitest run src/components/ui` 全绿
- [ ] Step 3: `pnpm dev` 下四主题目检基件外观(PM 抽查)
- [ ] Step 4: `pnpm tsc --noEmit && pnpm lint` 后提交:`git commit -m "新增消费主题令牌的统一 UI 基件库"`

## Task 5: i18n 按页拆分(为 Wave 3 并行铺路)

**Files:**
- Modify: `src/i18n/index.ts`;把 `src/i18n/locales/zh-CN.json`、`en.json` 拆为 `src/i18n/locales/{zh-CN,en}/{common,library,scraper,settings,cnTools,import}.json`

- [ ] Step 1: 按现有 key 的页面归属拆文件(跨页共用的进 `common.json`),`index.ts` 里 `import` 后深合并为单 translation 对象,**key 全名不变**(不引入 namespace,避免改调用点)
- [ ] Step 2: `pnpm dev` 抽查中英文均正常显示;`pnpm tsc --noEmit && pnpm build` 全绿
- [ ] Step 3: 提交:`git commit -m "i18n 词条按页面拆分以支持并行开发"`

## Task 6: 主题导入 Rust 命令

**Files:**
- Create: `src-tauri/src/commands/theme.rs`
- Modify: `src-tauri/src/commands/mod.rs`、`src-tauri/src/lib.rs`(注册命令)、`src/lib/api.ts`(前端封装)、`src/types/index.ts`

命令契约(前端 `api.ts` 同步封装三个函数):

```rust
/// 列出 config/themes/ 下全部已导入主题:
/// 返回 Vec<CustomThemeInfo> { manifest_json: String, dir: String, custom_css: Option<String> }
#[tauri::command]
pub fn list_custom_themes(app: tauri::AppHandle) -> Result<Vec<CustomThemeInfo>, String>

/// 导入 .rrtheme(zip):解压→读 theme.json→校验(id/schemaVersion/大小上限 20MB)
/// →CSS 清洗→写入 config/themes/<id>/(已存在同 id 则整目录覆盖)
#[tauri::command]
pub fn import_theme_pack(app: tauri::AppHandle, file_path: String) -> Result<CustomThemeInfo, String>

#[tauri::command]
pub fn delete_custom_theme(app: tauri::AppHandle, id: String) -> Result<(), String>
```

- [ ] **Step 1: TDD——`theme.rs` 内先写单元测试**(`#[cfg(test)]`):
  - `sanitize_css`:剥离 `@import …;`、`url(http…)`/`url(//…)`(替换为 `url()` 空值并计数)、拒绝 `</style`、`javascript:`;保留 `url(assets/xxx.png)` 相对路径并重写为占位符 `__RR_ASSET__/xxx.png`(前端用 convertFileSrc 替换)
  - `validate_manifest_json`:合法通过;错误 schemaVersion / 非法 id(路径穿越字符)拒绝
  - zip 解压:条目路径含 `..` 拒绝(zip-slip 防护);超 20MB 拒绝
- [ ] Step 2: `cargo test` FAIL → 实现 → PASS
- [ ] Step 3: 命令注册进 `lib.rs` `invoke_handler`;错误信息中文
- [ ] Step 4: `api.ts` 增加 `listCustomThemes()`/`importThemePack(path)`/`deleteCustomTheme(id)`(Tauri invoke;Web 模式抛"桌面版专用"错误);`types/index.ts` 加 `CustomThemeInfo`
- [ ] Step 5: `cargo fmt && cargo clippy -- -D warnings && cargo test`、`pnpm tsc --noEmit` 全绿,提交:`git commit -m "实现主题包导入的 Tauri 命令与前端封装"`

---

# Wave 3:页面重构(3 任务,可并行;i18n 已按页分文件)

**共同要求**:页面根容器挂 `rr-page` 类(吃 pageTransition 效果);全部弹窗/按钮/输入替换为 `components/ui` 基件;该页所有用户可见英文文案迁入对应 i18n 分文件(zh-CN 为准,en 同步补);disabled/loading/empty 三态齐全。

## Task 7: Library 新布局(A+B)

**Files:**
- Create: `src/pages/LibraryShelf.tsx`(系统货架)、`src/components/rom/SystemCard.tsx`
- Modify: `src/App.tsx`(路由)、`src/components/layout/Sidebar.tsx`(系统树)、`src/pages/Library.tsx`(改为单系统页)、`src/i18n/locales/{zh-CN,en}/library.json`

- [ ] **Step 1: 路由改造(`src/App.tsx`)**

```tsx
<Route path="/" element={<Layout />}>
  <Route index element={<Navigate to="/library" replace />} />
  <Route path="library" element={<LibraryShelf />} />
  <Route path="library/:systemId" element={<Library />} />
  <Route path="scraper" element={<Scraper />} />
  <Route path="cn-tools" element={<CnRomTools />} />
  <Route path="import" element={<Import />} />
  <Route path="settings" element={<Settings />} />
</Route>
```

`systemId` 使用 `encodeURIComponent(system.name)`;`Library.tsx` 内 `useParams()` 解码后过滤 `romStore.systemRoms`,查无此系统显示 `EmptyState` + 返回货架按钮。

- [ ] **Step 2: `LibraryShelf.tsx`**:遍历 `romStore.availableSystems`,每系统一张 `SystemCard`(系统 logo 沿用现有 logo 资源解析逻辑,见 `Sidebar.tsx` 现有实现;名称;`{count} 个游戏`),点击 `navigate(…)`;另有一张「+ 添加目录」卡跳设置;库为空时保留现有空态(幽灵图标 + 添加目录)迁移到基件 `EmptyState`
- [ ] **Step 3: Sidebar 系统树**:「ROM 库」项可展开(chevron 旋转过渡走 motion 令牌),子项 = `availableSystems`(名称 + 数量 Badge),`NavLink` 到 `/library/:systemId`,当前路由高亮;侧边栏根挂 `rr-sidebar`
- [ ] **Step 4: `Library.tsx` 瘦身**:去掉全库聚合逻辑改为单系统数据源;保留网格/列表切换、虚拟滚动、卡片尺寸滑杆、系统内搜索;顶部面包屑 `ROM 库 / {系统名}`;RomView 卡片挂 `rr-card`
- [ ] Step 5: 本页英文文案清零(含审计项 8 在本页部分);`pnpm vitest run && pnpm tsc --noEmit && pnpm lint && pnpm build` 全绿
- [ ] Step 6: 提交:`git commit -m "Library 重构为系统货架与侧边栏系统树布局"`

## Task 8: Settings 重构(含主题选择与导入)

**Files:**
- Modify: `src/pages/Settings.tsx`(全面重写,拆出子组件)、`src/stores/appStore.ts`(customThemes 接后端)、`vite.config.ts`、`src/vite-env.d.ts`、`src/i18n/locales/{zh-CN,en}/settings.json`
- Create: `src/pages/settings/AppearanceSection.tsx`、`GeneralSection.tsx`、`ScraperSection.tsx`、`AboutSection.tsx`(Settings.tsx 只留 Tabs 骨架,每节一个文件)

- [ ] **Step 1: 版本号修复(审计项 3)**:`vite.config.ts` 确认/补上 `define: { APP_VERSION: JSON.stringify(pkg.version) }`;`vite-env.d.ts` 声明 `declare const APP_VERSION: string;`;全项目 `rg "import.meta.env.APP_VERSION"` 改为 `APP_VERSION`(含 `Sidebar.tsx`)
- [ ] **Step 2: AppearanceSection**:
  - 主题卡片网格:`BUILTIN_THEMES + customThemes`,每卡渲染该主题 5 色色板(直接读 manifest tokens 内联 style——此处属于主题数据展示,硬编码扫描白名单加 `src/pages/settings/AppearanceSection.tsx` 不必要,内联 style 用变量值不触发扫描)+ 名称 + 选中态(`gradient-border` 效果);点击 `setTheme(id)`
  - 「导入主题包」按钮:`@tauri-apps/plugin-dialog` 选 `.rrtheme` → `api.importThemePack(path)` → 成功 Toast + 刷新列表;失败 Toast 显示后端中文错误
  - 导入主题卡带删除按钮(`Dialog` 二次确认 → `api.deleteCustomTheme`)
  - 动效开关三档(`SegmentedControl` 用 `Tabs` 基件实现):调 `setMotion`
  - appStore:`initFromBackend` 时 `api.listCustomThemes()` 装载 `customThemes`(解析 manifest_json→validateManifest→LoadedTheme,customCss 直通,`__RR_ASSET__` 占位符用 `convertFileSrc(dir + …)` 替换)
- [ ] **Step 3: General/Scraper/About**:语言切换、view_mode 默认值迁移进 General;Scraper 凭证表单迁移进 ScraperSection(逻辑不动,Wave 4 才闭环);About 显示 `APP_VERSION`、许可、仓库链接
- [ ] Step 4: 本页英文文案清零;门禁全绿;提交:`git commit -m "重构设置页:主题选择/导入/动效开关与版本号修复"`

## Task 9: CnRomTools + Import 重构

**Files:**
- Modify: `src/pages/CnRomTools.tsx`(拆出 `src/pages/cn-tools/` 子组件,单文件 ≤300 行)、`src/pages/Import.tsx`、`src/i18n/locales/{zh-CN,en}/{cnTools,import}.json`

- [ ] **Step 1: CnRomTools 拆分**:按现有 UI 区块拆 `DirectoryPicker.tsx`、`ScanResultTable.tsx`、`MatchToolbar.tsx`、`ExportPanel.tsx`;逻辑与 store 调用**原样保留**(store 在 `cnRomToolsStore.ts`,不动),只换壳:表格行 hover、置信度 Badge(高≥90 success / 中≥60 warning / 低 error)、进度条走令牌
- [ ] **Step 2: Import 页(审计项 2)**:后端未实现的导入/导出入口**下线**——按钮移除,原位置放 `EmptyState`(icon + 「该功能开发中,当前版本请使用中文 ROM 工具箱的导出能力」+ 跳转按钮到 `/cn-tools`);保留已实现的 metadata 导入对话框入口(确认 `MetadataImportDialog.tsx` 走的命令在 `lib.rs` 已注册,已注册即保留)
- [ ] Step 3: 两页英文文案清零;门禁全绿;提交:`git commit -m "重构中文工具箱与导入页并下线未实现入口"`

**Wave 3 验收(PM)**:三任务合并后全量门禁 + 交叉编译 exe 四主题截图核对(货架页、系统页、设置页、工具箱)。

---

# Wave 4:详情与 Scraper(2 任务,可并行)

## Task 10: ROM 详情与 Scrape 弹窗重构

**Files:**
- Modify: `src/components/rom/RomDetail.tsx`、`ScrapeDialog.tsx`、`BatchScrapeDialog.tsx`、`src/components/common/RootDirectoryDialog.tsx`、`MetadataImportDialog.tsx`、`DirectoryInput.tsx`、`src/i18n/locales/{zh-CN,en}/library.json`

- [ ] Step 1: 全部弹窗迁移到 `ui/Dialog`;按钮/输入/下拉迁移到基件;RomDetail 封面区加载骨架(`Spinner`)与缺图占位(`EmptyState` 紧凑型)
- [ ] Step 2: ScrapeDialog 搜索结果列表:候选项挂 `rr-card`,置信度 `Badge`,选中态走 `focusRing` 效果;媒体资产勾选网格统一复选样式
- [ ] Step 3: 文案清零、门禁全绿、提交:`git commit -m "重构 ROM 详情与抓取弹窗为统一基件"`

## Task 11: Scraper 配置闭环(审计项 5、6)

**Files:**
- Modify: `src-tauri/src/commands/scraper.rs`、`src-tauri/src/scraper/manager.rs`、`src/pages/Scraper.tsx`、`src/pages/settings/ScraperSection.tsx`、`src/stores/scraperStore.ts`、`src/lib/api.ts`

- [ ] **Step 1(Rust,TDD)**:`manager.rs` 加 `provider_infos()`(遍历注册 provider 返回 id/名称/是否需凭证/当前启用态,来源 settings);`scraper.rs` 的 `get_scraper_providers` 改为调它,删除硬编码列表;`save_scraper_config` 保存后重建 manager 实例使启用态即时生效;搜索聚合处接入 `matcher.rs::rank_results` 排序。每步先写 `#[cfg(test)]` 测试(provider 列表随 settings 变化;rank_results 对已知输入的排序断言)再实现
- [ ] Step 2(前端):`scraperStore`/`ScraperSection` 改为消费动态 provider 列表;开关与凭证保存即时生效(保存后重新拉取列表断言状态一致);`Scraper.tsx` 状态页显示各 provider 启用/凭证状态 Badge
- [ ] Step 3: `cargo fmt && cargo clippy -- -D warnings && cargo test` + 前端门禁全绿;提交:`git commit -m "Scraper 配置闭环:动态 provider 列表与评分排序接入"`

---

# Wave 5:收尾(1 任务,串行)

## Task 12: 清理与全局走查

**Files:**
- Delete: `src/utils/media.ts`(若确认无引用)、`src/lib/api.ts` 中 `resolveMediaUrl`
- Modify: 走查中发现的零散问题;`CHANGELOG.md`;`package.json`(版本 → 0.3.0)

- [ ] Step 1: 死代码清理(审计「无用代码清单」前端部分):`rg` 确认零引用后删除;`pnpm build` 确认 tree-shake 无断链
- [ ] Step 2: 全局英文文案终扫(frontend-verify 清单第 5 项),i18n en 分文件补齐缺口(fallback 不报 missing key)
- [ ] Step 2b: 写主题包开发规范 `docs/theme-pack-guide.md`:`.rrtheme` 结构、theme.json 字段表(令牌全集/效果插槽/取值)、style.css 约定(rr-* 钩子清单、data-motion 门控义务、清洗规则)、assets 引用方式、最小示例包;这是主题设计器 webapp 的输出契约文档
- [ ] Step 3: qa-verify 代理跑完整验收清单出报告;PM 修复报告中的不通过项
- [ ] Step 4: 四主题 × 全页面截图终验(PM);`package.json` 版本改 `0.3.0`,CHANGELOG 从本次全部 commit 总结
- [ ] Step 5: 提交:`git commit -m "前端重构收尾:死代码清理与 0.3.0 版本"`

---

## 任务依赖图

```
Task1 ┐                         ┌ Task7 ┐
Task2 ┴→ Task3 →┬ Task4 ┬──────→│ Task8 ├→┬ Task10 ┬→ Task12
                ├ Task5 ┘       └ Task9 ┘ └ Task11 ┘
                └ Task6 ────────↗(Task8 依赖 Task6)
```
