# 审计与修复指南（基于 docs/plan.md）

## 审计范围
- 前端（React/Vite）
- Tauri Rust 后端
- Web 后端（Express）
- 文档与配置

## 关键风险与差距（按优先级）
1. 安全：所有 API 缺少 `X-API-Key` 鉴权，且 `/api/media` 允许任意路径读取
   - 影响：任何人可调用 API/读取任意文件路径，违反安全要求。
   - 涉及：`server/src/index.ts`
   - 修复建议：
     - 为 `/api/*` 添加统一中间件校验 `X-API-Key`。
     - `/api/media` 仅允许访问 `ROMS_DIR` 或配置的媒体目录下的文件（`path.resolve` + 前缀校验）。

2. 功能缺失：导入/导出在 Tauri 端被禁用，但 UI 仍提供入口
   - 影响：用户点击后直接报错，功能无法使用。
   - 涉及：`src-tauri/src/commands/export.rs`、`src-tauri/src/commands/import.rs`、`src/pages/Import.tsx`
   - 修复建议：实现导入/导出，或临时移除入口并给出清晰中文说明。

3. 版本注入不一致：定义了 `APP_VERSION` 常量，但前端使用 `import.meta.env.APP_VERSION`
   - 影响：版本号显示为空，违反“前端通过 APP_VERSION 注入”的规则。
   - 涉及：`vite.config.ts`、`src/pages/Settings.tsx`、`src/components/layout/Sidebar.tsx`
   - 修复建议：统一使用 `APP_VERSION`，或在 Vite 中注入 `import.meta.env.APP_VERSION`。

4. 后端版本读取缺失
   - 影响：未按要求在后端运行时读取版本（`fs.readFileSync`）。
   - 涉及：`server/src/index.ts`
   - 修复建议：启动时读取 `server/package.json` 并暴露到 `/api/health` 等接口。

5. Scraper 配置与能力信息硬编码，开关逻辑不闭环
   - 影响：UI 状态与实际启用状态不一致，配置无法持久化。
   - 涉及：`src-tauri/src/commands/scraper.rs`
   - 修复建议：从 `ScraperManager`/settings 动态生成 provider 列表，保存启用状态与凭证。

6. 智能匹配评分未接入，计划功能未兑现
   - 影响：`Jaro-Winkler` 评分与排序未用于搜索结果。
   - 涉及：`src-tauri/src/scraper/manager.rs`、`src-tauri/src/scraper/matcher.rs`
   - 修复建议：在搜索结果合并后调用 `rank_results` 或统一重算 confidence。

7. Web 后端缺少 EmulationStation 解析
   - 影响：Web 模式下与 Tauri 行为不一致（计划中应支持）。
   - 涉及：`server/src/rom-service.ts`
   - 修复建议：实现 XML 解析逻辑，保持与 `src-tauri/src/rom_service.rs` 一致。

8. UI 文本中仍存在英文硬编码
   - 影响：不符合“所有 UI 文本使用简体中文”的要求。
   - 涉及：`src/pages/Import.tsx`、`src-tauri/src/commands/export.rs`、`src-tauri/src/commands/import.rs`、`src-tauri/src/commands/config.rs`
   - 修复建议：统一进入 i18n 词条并使用简体中文。

9. 性能：Web 端 `scanRomsDirectory` 请求期间同步 IO，目录大时阻塞明显
   - 影响：慢响应、阻塞 Node 事件循环。
   - 涉及：`server/src/rom-service.ts`
   - 修复建议：改为异步 IO + 缓存（例如缓存扫描结果与变更时间）。

## 无用代码/功能清单
- `src/utils/media.ts`：`getMediaUrl` 未被调用。
- `src/lib/api.ts`：`resolveMediaUrl` 未被调用且同步返回 Promise（潜在隐患）。
- `src-tauri/src/commands/scraper.rs`：`copy_dir_recursive` 未被使用。
- `src-tauri/src/commands/config.rs`：`list_directory` 未暴露到 `invoke_handler`。
- `src-tauri/src/scraper/matcher.rs`：`rank_results` 等未被实际调用（功能名存实亡）。

## 已补充测试
- 新增 EmulationStation 解析的最小测试：`src-tauri/src/rom_service.rs`
  - 验证 `gamelist.xml` 能被正确解析，避免回归。

## 修复建议步骤（建议顺序）
1. 先补安全：统一 `X-API-Key` 鉴权 + `/api/media` 路径白名单。
2. 修复版本注入与后端版本读取。
3. 处理 Import/Export：要么实现、要么下线入口并清晰提示。
4. Scraper 配置闭环：provider 列表动态化 + 启用状态持久化。
5. 接入 matcher 的排序逻辑，兑现计划中的评分能力。
6. Web 端补齐 EmulationStation 解析逻辑。
7. 清理无用代码，减少维护成本。
8. 全面中文化 UI 文本并走 i18n。
