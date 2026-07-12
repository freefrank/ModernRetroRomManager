# 主题包开发规范(.rrtheme)

本文档是 ModernRetroRomManager 主题包格式的权威契约,面向手工制作主题包的作者与未来的主题设计器 webapp(其导出产物必须符合本规范)。对应实现:前端 `src/theme/`(校验/应用)、`src/stores/appStore.ts`(CSS 资产解析)、后端 `src-tauri/src/commands/theme.rs`(导入/清洗)。

## 一、包结构

主题包是一个 **zip 压缩包**,扩展名 `.rrtheme`,顶层结构固定:

```
mytheme.rrtheme
├── theme.json        # 清单:元信息 + 令牌 + 效果配置(必需)
├── style.css         # 自定义 CSS 层(可选,进阶表现力)
└── assets/           # 主题私有资源(可选):图片、woff2 字体等
    ├── background.png
    └── display.woff2
```

约束:

- **theme.json 必需**,缺失则拒绝导入;简单主题只需要这一个文件
- **解压后总大小 ≤ 20MB**,超限拒绝导入
- 包内禁止 `..`、绝对路径、盘符等路径穿越条目(zip-slip 防护),违规拒绝导入
- 导入流程:设置页「外观」→「导入主题包」→ 校验清单 + CSS 清洗 → 解压至 `config/themes/<id>/` → 出现在主题列表;**同 id 重复导入整目录覆盖**

## 二、theme.json 清单

### 顶层字段

| 字段 | 类型 | 必需 | 说明 |
|------|------|------|------|
| `schemaVersion` | number | 是 | 当前只支持 `1`,其他值拒绝导入 |
| `id` | string | 是 | 主题唯一标识,须匹配 `^[a-z0-9][a-z0-9-]*$`(小写字母/数字开头,只含小写字母、数字、连字符) |
| `name` | string | 是 | 主题显示名(建议简体中文) |
| `author` | string | 否 | 作者署名 |
| `tokens` | object | 是 | 设计令牌表(见下,可只写部分) |
| `effects` | object | 否 | 效果插槽配置(见下) |

容错规则:

- **缺失令牌回退默认主题**(`retro-arcade`)对应值,因此可以只覆盖想改的令牌
- **未知令牌 / 未知插槽被忽略**并产生告警,不阻断导入
- 令牌值中出现 `url(http…)`、`url(//…)`、`@import`、`javascript:`、`expression(` 任意一项,**整包拒绝导入**
- 非法 JSON、顶层字段(schemaVersion/id/name/tokens)缺失或类型错误 → 拒绝导入并给中文错误提示;单个令牌值类型错误按未知令牌处理(忽略 + 告警),不阻断导入

### 令牌全集(27 个)

#### 颜色(14)

| 令牌 | 含义 |
|------|------|
| `bg-primary` | 主背景(页面底色) |
| `bg-secondary` | 次级背景(卡片、面板) |
| `bg-tertiary` | 三级背景(悬浮块、输入框底) |
| `accent-primary` | 主强调色(选中态、主按钮) |
| `accent-secondary` | 次强调色(渐变副色、计数高亮) |
| `accent-success` | 成功语义色 |
| `accent-warning` | 警告语义色 |
| `accent-error` | 错误语义色 |
| `text-primary` | 主文本 |
| `text-secondary` | 次要文本 |
| `text-muted` | 弱化文本(占位、辅助说明) |
| `border-default` | 默认边框 |
| `border-hover` | 悬停边框 |
| `border-highlight` | 高亮/焦点边框 |

#### 字体(3)

| 令牌 | 含义 |
|------|------|
| `font-display` | 标题字体栈(h1/h2 消费;像素字体只覆盖拉丁字符时中文自动回退后续字体) |
| `font-body` | 正文字体栈 |
| `font-mono` | 等宽字体栈(路径、代码) |

字体只能引用系统字体栈或主题包 `assets/` 内自带的字体文件(经 style.css `@font-face` + 相对路径引入),**禁止外部 URL**。

#### 形状(3)

| 令牌 | 含义 |
|------|------|
| `radius-sm` | 小圆角(输入框、行内元素) |
| `radius-md` | 中圆角(按钮、小卡片) |
| `radius-lg` | 大圆角(卡片、弹窗) |

#### 质感(4)

| 令牌 | 含义 |
|------|------|
| `border-width` | 基件边框宽度 |
| `shadow-card` | 卡片阴影(复古主题可用硬阴影如 `3px 3px 0 <色>`) |
| `shadow-dialog` | 弹窗阴影 |
| `glow-accent` | 强调发光(box-shadow 值;不需要发光写 `none`) |

#### 动效(3)

| 令牌 | 含义 |
|------|------|
| `motion-fast` | 快速过渡时长(hover 等,如 `120ms`) |
| `motion-normal` | 常规过渡时长(面板展开、页面进场,如 `240ms`) |
| `motion-easing` | 缓动函数(复古主题可用 `steps(3, end)` 做像素感运动) |

运行时所有令牌被写入 `<html>` 的 CSS 变量(`--bg-primary` 等),`data-theme="<id>"` 标记当前主题。

### 效果插槽(5 个)

`effects` 对象的键是插槽名,值为 `{ "name": "<效果名>", "opacity"?: number, "color"?: string }`:

| 插槽 | 作用点 |
|------|--------|
| `backdrop` | 全屏背景叠加层(单个 `position: fixed` 元素) |
| `cardHover` | 卡片悬停反馈(`.rr-card:hover`) |
| `buttonPress` | 按钮按压反馈(`.rr-button:active`) |
| `pageTransition` | 页面切换进场(`.rr-page`) |
| `focusRing` | 键盘焦点态(`.rr-button/.rr-input/.rr-select/.rr-card:focus-visible`) |

参数:

- `opacity`:目前仅 `backdrop` 消费(写入 `--fx-backdrop-opacity`,缺省 `0.05`)
- `color`:预留字段,当前版本不消费
- 未配置的插槽视为 `none`;引用**未知效果名回退 `none`** 并告警

### 效果名目录(10 个)

| 效果名 | 适用插槽 | 表现 |
|--------|----------|------|
| `none` | 全部 | 关闭该插槽效果 |
| `scanlines` | backdrop | CRT 扫描线叠加(线色随 `text-primary`,亮度由 `opacity` 控制;full 档附呼吸动画) |
| `crt-flicker` | pageTransition | CRT 开机闪烁一帧(仅 full 档) |
| `hard-shift` | cardHover / buttonPress | 复古硬阴影位移(hover 上浮 + 硬阴影,按压下沉) |
| `pixel-jitter` | cardHover | 1px 像素抖动(low 档退化为静态 1px 位移) |
| `neon-pulse` | cardHover / focusRing | 霓虹呼吸发光(消费 `glow-accent`;呼吸动画仅 full 档) |
| `glitch-text` | cardHover | 故障闪码(色相偏移 + 位移,仅 full 档) |
| `gradient-border` | cardHover / focusRing | 双色流动渐变描边(`accent-primary` ↔ `accent-secondary`;流动仅 full 档) |
| `soft-glow` | cardHover | 柔和光晕 + 轻微上浮 |
| `fade-scale` | cardHover / buttonPress / pageTransition | 纯净缩放淡入 |

效果实现只使用 `transform` / `opacity` / `filter` / `box-shadow`(合成层友好),目录随版本经 `schemaVersion` 扩充。

## 三、style.css 约定

`style.css` 是可选的进阶层,用于效果目录覆盖不到的表现力(自定义 `@keyframes`、背景图、`@font-face` 等)。**只允许 CSS,永远不允许 JS。**

### rr-* 稳定类名钩子

UI 基件根元素带稳定类名,主题 CSS 只能通过它们定点覆盖(内部结构类名不承诺稳定):

| 类名 | 元素 |
|------|------|
| `rr-page` | 页面根容器(pageTransition 作用点) |
| `rr-sidebar` | 侧边栏 rail(左侧图标栏) |
| `rr-sidebar-panel` | 侧边栏上下文面板(系统树/工具列表) |
| `rr-backdrop` | 全屏背景叠加层(当前效果名在其 `data-fx` 属性上) |
| `rr-button` | 按钮 |
| `rr-icon-button` | 图标按钮 |
| `rr-card` | 卡片 |
| `rr-input` | 输入框 |
| `rr-select` | 下拉选择 |
| `rr-dialog` | 弹窗 |
| `rr-toast` | 通知条 |
| `rr-badge` | 徽章 |
| `rr-tabs` | 标签页 |
| `rr-tooltip` | 工具提示 |
| `rr-spinner` | 加载指示器 |
| `rr-empty` | 空状态 |

建议样式值继续引用令牌变量(`var(--accent-primary)` 等),使自定义层与令牌层联动。

### data-motion 门控义务

应用在 `<html>` 上维护 `data-motion="off|low|full"` 三档动效开关,主题 CSS **必须遵循同一约定**(验收抽查):

- **off**:禁止一切动画与过渡 —— 应用层有全局 `!important` 兜底强制关停,但主题不应依赖兜底
- **low**:仅允许 `transition` 过渡,禁止 `animation` 动画
- **full**:全部开放

写法约定:任何 `animation` 规则必须挂在 `[data-motion="full"]` 前缀选择器下;`transition` 规则用 `[data-motion]:not([data-motion="off"])` 前缀:

```css
[data-motion="full"] .rr-card:hover {
  animation: my-shine 1.2s ease-in-out infinite;
}
```

### 导入清洗规则

`style.css` 在导入时由后端强制清洗,不符合预期的内容会被剥离或整包拒绝:

| 内容 | 处理 |
|------|------|
| `</style` 或 `javascript:`(不区分大小写) | **整包拒绝导入** |
| `@import …;` | 剥离 |
| `url(http…)` / `url(//…)` 外部引用 | 替换为空 `url()` 并计数告警 |
| `url(assets/<path>)` 相对路径 | 重写为 `url(__RR_ASSET__/<path>)` 占位符 |

其余 CSS 原样保留。**清洗后的版本落盘**,原始 `style.css` 不保留。

### assets 相对路径与 __RR_ASSET__ 机制

CSS 中引用包内资源必须写 `url(assets/<相对路径>)`,链路如下:

1. 导入时后端将其重写为 `url(__RR_ASSET__/<相对路径>)` 占位符
2. 运行时前端加载主题,对占位符做路径安全校验(拒绝空段、`.`、`..`、`\`、盘符、绝对路径,非法项置为空 `url()`)
3. 合法路径经 Tauri `convertFileSrc(<主题目录>/assets/<相对路径>)` 解析为本地资源 URL 注入 `<style id="rr-theme-pack-css">`

因此资源**只能放在 `assets/` 下并以相对路径引用**,不允许任何网络加载(安全 + 离线)。

## 四、最小示例包

目录树:

```
midnight-sakura.rrtheme
├── theme.json
├── style.css
└── assets/
    └── petals.png
```

完整示例 `theme.json`(27 令牌全写 + 效果配置;实际可只写想覆盖的令牌):

```json
{
  "schemaVersion": 1,
  "id": "midnight-sakura",
  "name": "夜樱",
  "author": "示例作者",
  "tokens": {
    "bg-primary": "#171019",
    "bg-secondary": "#211726",
    "bg-tertiary": "#2c1f33",
    "accent-primary": "#f472b6",
    "accent-secondary": "#c084fc",
    "accent-success": "#4ade80",
    "accent-warning": "#fbbf24",
    "accent-error": "#f87171",
    "text-primary": "#faf5fb",
    "text-secondary": "#b39cb8",
    "text-muted": "#6f5d75",
    "border-default": "rgba(244,114,182,0.16)",
    "border-hover": "rgba(244,114,182,0.34)",
    "border-highlight": "rgba(244,114,182,0.6)",
    "font-display": "'Segoe UI', 'Microsoft YaHei', sans-serif",
    "font-body": "-apple-system, 'Segoe UI', 'Microsoft YaHei', Roboto, sans-serif",
    "font-mono": "Consolas, 'Courier New', monospace",
    "radius-sm": "6px",
    "radius-md": "10px",
    "radius-lg": "14px",
    "border-width": "1px",
    "shadow-card": "0 4px 16px rgba(0,0,0,0.35)",
    "shadow-dialog": "0 12px 48px rgba(0,0,0,0.55)",
    "glow-accent": "0 0 14px rgba(244,114,182,0.45)",
    "motion-fast": "120ms",
    "motion-normal": "240ms",
    "motion-easing": "cubic-bezier(0.2, 0.8, 0.2, 1)"
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

示例 `style.css`(演示 assets 引用与 data-motion 门控):

```css
/* 侧边栏面板铺一层半透明花瓣纹理(包内资源,相对路径) */
.rr-sidebar-panel {
  background-image: url(assets/petals.png);
  background-size: 240px;
  background-blend-mode: soft-light;
}

/* 卡片悬停微光扫过:动画必须挂 full 档 */
[data-motion="full"] .rr-card:hover {
  animation: sakura-shimmer 1.8s var(--motion-easing) infinite;
}

@keyframes sakura-shimmer {
  0%, 100% { filter: brightness(1); }
  50% { filter: brightness(1.08); }
}
```

## 五、发布前自检清单

- [ ] `theme.json` 为合法 JSON,`schemaVersion: 1`,`id` 符合 `^[a-z0-9][a-z0-9-]*$`
- [ ] 令牌名全部在 27 个全集内(未知令牌会被忽略)
- [ ] 效果名/插槽名在目录内(未知效果回退 `none`)
- [ ] 无任何外部 URL(`http`、`//`)、`@import`、脚本向量
- [ ] 资源全部位于 `assets/` 并以 `url(assets/…)` 相对路径引用
- [ ] `animation` 规则全部挂 `[data-motion="full"]` 前缀
- [ ] 解压后总大小 ≤ 20MB
- [ ] 在动效三档(关/低/全)与四套内置主题切换往返下均无残留样式
