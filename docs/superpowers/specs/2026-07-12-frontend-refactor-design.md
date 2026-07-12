# 前端重构设计文档

日期:2026-07-12
状态:待用户批准

## 背景与目标

现有前端(约 6500 行,5 个页面)存在两类问题:视觉质量差,以及一批功能性错误(审计见 `docs/AUTID_FIX.md`)。本次重构目标:

1. 建立**可扩展的多主题系统**,内置 4 套主题,支持导入第三方主题包
2. 重做 Library 布局(侧边栏系统树 + 系统货架首页)
3. 建立统一 UI 基件库,消灭散落的硬编码样式
4. 修复纯前端错误 + Scraper 配置闭环(Rust)
5. 引入 ESLint 质量门禁

**非目标**:Web 模式(`server/`)一致性补齐;光亮主题(可由未来主题包补充);主题设计器 webapp(列入路线图,单独立项)。

## 一、主题系统(主题即数据)

### 主题包格式

主题是一个 JSON 文件,内置主题与用户导入的主题走**同一格式**:

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
- **字体安全**:主题包只能引用应用内置的字体栈别名(如 `var(--font-system)`、内置像素字体),不允许加载外部字体文件/URL(安全 + 离线)。
- **应用机制**:运行时将选中主题的 tokens 写入 `<html>` 上的 CSS 变量(`data-theme` 标记 id);Tailwind v4 `@theme` 继续映射这些变量(沿用现有架构)。

### 内置主题(4 套)

| id | 名称 | 特征 |
|----|------|------|
| `retro-arcade` | 复古游戏厅(**默认**) | 像素标题字体、0 圆角、硬阴影、街机红/金/紫 |
| `modern-dark` | 现代极简暗色 | 中性深灰、单强调色、大圆角软阴影,封面当主角 |
| `cyberpunk` | 赛博霓虹 | 深蓝黑底、霓虹青/品红、发光描边(克制用量) |
| `violet` | 紫罗兰 | 延续现有 Indigo/Purple 基调的精修版 |

### 导入主题包

- 设置页「外观」区:主题卡片选择器(内置 4 套 + 已导入)+「导入主题包」按钮
- 导入流程:文件选择器(.json)→ 前端校验 → Tauri 命令存入 `config/themes/` → 出现在主题列表
- 需要的 Rust 命令:`list_custom_themes`、`import_theme_pack`、`delete_custom_theme`(读写 config 目录,复用现有 settings 模式)
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
