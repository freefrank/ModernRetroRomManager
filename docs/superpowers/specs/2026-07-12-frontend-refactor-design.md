# 前端重构设计文档

日期:2026-07-12
状态:待用户批准

## 背景与目标

现有前端(约 6500 行,5 个页面)存在两类问题:视觉质量差,以及一批功能性错误(审计见 `docs/archive/AUTID_FIX.md`)。本次重构目标:

1. 建立**可扩展的多主题系统**,内置 4 套主题,支持导入第三方主题包
2. 重做 Library 布局(侧边栏系统树 + 系统货架首页)
3. 建立统一 UI 基件库,消灭散落的硬编码样式
4. 修复纯前端错误 + Scraper 配置闭环(Rust)
5. 引入 ESLint 质量门禁

**非目标**:Web 模式(`server/`)一致性补齐;光亮主题(可由未来主题包补充);主题设计器 webapp(列入路线图,单独立项)。

## 一、主题系统(主题即数据)

### 主题包格式

主题包是一个 **zip 压缩包**(扩展名 `.rrtheme`),内置主题与用户导入的主题走**同一结构**:

```
mytheme.rrtheme
├── theme.json        # 清单:元信息 + 令牌 + 效果配置(必需)
├── style.css         # 自定义 CSS 层(可选,进阶表现力)
└── assets/           # 主题私有资源(可选)
    ├── background.png   # 背景图
    ├── display.woff2    # 自定义标题字体
    └── ...
```

- **theme.json** 是核心清单(结构见下),简单主题只需要这一个文件
- **style.css** 供效果目录覆盖不到的进阶需求:导入时做安全清洗——剥离 `@import`、外部 `url()`(仅允许 `assets/` 相对路径)、任何脚本向量;UI 基件全部带稳定类名钩子(`rr-button`、`rr-card`、`rr-sidebar`…)供其定点覆盖
- **assets/** 中的字体、图片由清单/CSS 以相对路径引用,运行时经 Tauri asset protocol 解析,不允许网络加载
- 导入流程:选择 `.rrtheme` → 校验清单与 CSS 清洗 → 解压到 `config/themes/<id>/` → 出现在主题列表

**theme.json** 结构:

```json
{
  "schemaVersion": 1,
  "id": "retro-arcade",
  "name": "复古游戏厅",
  "author": "built-in",
  "tokens": {
    "bg-base": "#1b1b2f",
    "bg-surface": "#24243e",
    "bg-elevated": "#2e2e4e",
    "accent": "#e94560",
    "accent-secondary": "#f0a500",
    "text-primary": "#ffffff",
    "text-secondary": "#9090a8",
    "text-muted": "#5a5a70",
    "border-default": "#3a3a5c",
    "border-strong": "#533483",
    "success": "#4ade80",
    "warning": "#f0a500",
    "error": "#e94560",
    "font-display": "'Press Start 2P', var(--font-cjk-fallback)",
    "font-body": "var(--font-system)",
    "font-mono": "var(--font-mono-system)",
    "radius-sm": "0px",
    "radius-md": "0px",
    "radius-lg": "2px",
    "shadow-card": "3px 3px 0 var(--border-strong)",
    "glow-accent": "none",
    "border-width": "2px"
  }
}
```

规则:

- **令牌四组**:颜色 / 字体 / 形状(圆角)/ 质感(阴影、发光、边框宽)。复古主题的硬阴影、赛博主题的霓虹光都通过令牌表达,组件代码不感知主题。
- **校验与容错**:`schemaVersion` 必须支持;缺失令牌回退到默认主题对应值;非法 JSON 拒绝导入并给中文错误提示。
- **字体**:可引用应用内置字体栈别名(如 `var(--font-system)`、内置像素字体),或主题包 `assets/` 内自带的字体文件;禁止外部 URL(安全 + 离线)。
- **应用机制**:运行时将选中主题的 tokens 写入 `<html>` 上的 CSS 变量(`data-theme` 标记 id);Tailwind v4 `@theme` 继续映射这些变量(沿用现有架构)。

### 动效与光效(效果系统)

主题的个性不止配色,还包括动效与光效。为兼顾表现力、导入安全与性能,分三层:

**1. 动效令牌(motion tokens)** — 与颜色令牌同级,进主题包 `tokens`:

```json
"motion-duration-fast": "120ms",
"motion-duration-normal": "240ms",
"motion-easing": "cubic-bezier(0.2, 0.8, 0.2, 1)"
```

基件库全部过渡动画消费这些令牌(复古主题可用阶梯式 `steps()` 缓动做"像素感"运动)。

**2. 效果目录(effect catalog)** — 应用内置一组**具名、可参数化**的光效/动效原语,主题包按名引用并调参,不能注入任意代码:

```json
"effects": {
  "backdrop": { "name": "scanlines", "opacity": 0.06 },
  "cardHover": { "name": "neon-pulse", "color": "var(--accent)" },
  "pageTransition": { "name": "crt-flicker" },
  "buttonPress": { "name": "hard-shift" }
}
```

首批目录(按内置主题需求实现):

| 效果名 | 类型 | 用途示例 |
|--------|------|----------|
| `scanlines` | 背景叠加层 | 复古:CRT 扫描线 |
| `crt-flicker` | 页面切换 | 复古:开机闪烁一帧 |
| `hard-shift` | 按压反馈 | 复古:硬阴影位移(3px 按下) |
| `pixel-jitter` | hover | 复古:1px 抖动 |
| `neon-pulse` | hover/focus 光效 | 赛博:霓虹呼吸发光 |
| `glitch-text` | hover | 赛博:标题故障闪码 |
| `gradient-border` | 选中态 | 赛博/紫罗兰:流动渐变描边 |
| `soft-glow` | hover 光效 | 紫罗兰:柔和光晕 |
| `fade-scale` | hover/过渡 | 现代极简:纯净缩放淡入 |
| `none` | — | 任意插槽关闭效果 |

插槽(slot)固定:`backdrop` / `cardHover` / `pageTransition` / `buttonPress` / `focusRing`。目录随版本扩充,`schemaVersion` 把关兼容性;主题包引用未知效果名时回退 `none` 并告警。

**3. 性能与可访问性约束**:

- 效果实现只允许 `transform` / `opacity` / `filter` 合成层属性,禁止触发 layout;`backdrop` 叠加层用单个 `position: fixed` 元素,不进虚拟列表
- 虚拟滚动中的卡片 hover 效果必须 GPU 友好,滚动帧率不因主题降级
- 尊重 `prefers-reduced-motion`,并在设置页提供动效开关(关 / 低 / 全):低 = 仅过渡无光效,关 = 全部禁用(含主题包自定义动效)
- **CSS 扩展动效/光效**:效果目录覆盖不到的表现力,主题包在 `style.css` 里用 `@keyframes`、`animation`、`box-shadow`/`filter` 光效自由扩展,配合 `rr-*` 类名钩子定点作用;经导入清洗(无外部引用、无脚本向量),JS 永远不允许
- 动效开关实现方式:`data-motion="off|low|full"` 属性 + 内置样式统一门控;主题包 CSS 需遵循同一约定(文档写入主题包开发规范,验收抽查)

### 内置主题(4 套)

| id | 名称 | 特征 |
|----|------|------|
| `retro-arcade` | 复古游戏厅(**默认**) | 像素标题字体、0 圆角、硬阴影、街机红/金/紫 |
| `modern-dark` | 现代极简暗色 | 中性深灰、单强调色、大圆角软阴影,封面当主角 |
| `cyberpunk` | 赛博霓虹 | 深蓝黑底、霓虹青/品红、发光描边(克制用量) |
| `violet` | 紫罗兰 | 延续现有 Indigo/Purple 基调的精修版 |

### 导入主题包

- 设置页「外观」区:主题卡片选择器(内置 4 套 + 已导入,含预览色板)+「导入主题包」按钮
- 需要的 Rust 命令:`list_custom_themes`、`import_theme_pack`(解压 `.rrtheme`、校验清单、CSS 清洗)、`delete_custom_theme`(读写 config 目录,复用现有 settings 模式;zip 解压复用已有 `zip` 依赖)
- settings 的 `theme` 字段存主题 id;旧值(`dark`/`light`)及一切未知 id 回退默认主题

### 字体本地化

去掉 Google Fonts CDN `@import`,所有字体 woff2 打包进应用(正文可直接用系统字体栈;像素风标题字体本地打包,仅覆盖拉丁字符,中文回退系统字体)。

## 二、UI 基件库 `src/components/ui/`

Button / IconButton / Card / Input / Select / Dialog / Toast / EmptyState / Spinner / Badge / Tabs / Tooltip。

- 全部只消费令牌,**禁止硬编码颜色/圆角/阴影**(ESLint + 验收走查把关)
- 现有页面散落的自绘弹窗、按钮、输入框全部收编替换

## 三、Library 新布局(A+B 组合)

- `/library` = 系统货架首页:每个系统一张大卡片(logo、名称、游戏数),点击进入
- `/library/:systemId` = 该系统的游戏网格/列表:保留网格/列表切换、虚拟滚动、卡片尺寸滑杆、系统内搜索
- 侧边栏「ROM 库」展开系统树(名称 + 数量),点击直达 `/library/:systemId`;树与货架数据同源(romStore)
- 顶栏全局搜索跨系统
- ROM 详情、Scrape 弹窗改用基件库重做视觉,交互逻辑不动

## 四、纯前端错误修复(随页面波次走)

1. 版本号显示为空 → 统一 `APP_VERSION` 注入(`vite.config.ts` define + 使用处统一)
2. UI 英文硬编码 → 全部进 i18n 简体中文词条
3. Import/Export 未实现入口 → 下线并显示"开发中"提示
4. 死代码清理:`resolveMediaUrl`、`utils/media.ts::getMediaUrl` 等

## 五、Scraper 配置闭环(Rust)

- Provider 列表由后端 `ScraperManager` 动态生成,替换前端硬编码
- 启用开关、凭证保存后持久化并重启生效
- 搜索结果接入 `matcher.rs::rank_results` 评分排序(兑现审计项 6)

## 六、执行模式(PM + 并行 subagent)

### Wave 0:协作基建(串行)
- `.claude/agents/`:`frontend-impl`(前端实现)、`rust-impl`(Tauri 后端)、`qa-verify`(验收)三个子代理定义
- 项目 skills:令牌使用规范(给 agent 引用)、验收清单流程
- ESLint 安装与配置(含禁止硬编码颜色的规则)

### Wave 1:主题地基(串行,所有后续工作的依赖)
主题包格式与校验、4 套内置主题 JSON、运行时应用与切换、字体本地化、`@theme` 映射扩展

### Wave 2(并行 ×2)
UI 基件库 ‖ 主题导入的 Rust 命令

### Wave 3(并行 ×3)
Library 新布局 ‖ Settings 重构(含主题选择器/导入 UI)‖ CnRomTools + Import 重构

### Wave 4(并行 ×2)
ROM 详情 + Scrape 弹窗 ‖ Scraper 配置闭环(Rust)

### Wave 5:收尾(串行)
死代码清理、i18n 全量走查、四主题真机截图核对

### 每波验收标准(PM 执行)
1. `pnpm tsc --noEmit`、ESLint、`pnpm build` 全绿
2. UI 波次:交叉编译 Windows exe,互操作启动 + 截图,核对 4 个主题实际渲染
3. 验收通过 → 中文 commit(不 push)

## 路线图(本次不做)

- **主题设计器 webapp**:独立小型 Web 应用,可视化调令牌、实时预览、导出 `schemaVersion` 兼容的主题包 JSON。主题包格式即其输出契约。
- 光亮主题包、社区主题分享。

## 验收总标准

- 4 套主题在全部页面视觉一致可用,切换即时生效、持久化
- 主题包导入:合法包成功入列,非法包中文报错
- 审计清单第 2、3、5、6、8 项及死代码项全部关闭
- ESLint 零告警,`tsc` 零错误
- 所有 UI 文本简体中文
