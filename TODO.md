# TODO

## 高优先级

- [x] 修复导入/导出页面与后端接口不一致
  - `src/pages/Import.tsx` 已改用 `export_scraped_data`，并新增系统选择。
  - 页面英文文案已统一进入 i18n（简体中文/英文）。

- [x] Web 后端补齐 EmulationStation 解析
  - `server/src/rom-service.ts` 已实现 `gamelist.xml` 解析。
  - 字段与 Tauri 端保持一致（含 rating 百分比转换、缺失 ROM 跳过、相对媒体路径解析）。

- [x] 修复 ScreenScraper systemid 映射
  - `src-tauri/src/system_mapping.rs` 新增 `find_screenscraper_system_id`。
  - hash 查询与文件名搜索仅在映射成功时附带 `systemeid`。

## 中优先级

- [x] 完善拖拽添加目录校验
  - `src/components/layout/Layout.tsx` 拖入路径先经 `validate_path` 校验。
  - 非文件夹路径给出中文错误提示（自动消失的通知条）。

- [ ] 批量 Scrape 任务队列与进度反馈
  - 已有：后台任务 + 进度事件 + 取消（`cancel_batch_scrape`）+ 成功/失败统计
    + Hash 精确匹配 + 默认媒体下载。
  - 待做：失败任务重试、任务队列持久化（参见 `docs/plan.md` 路线图 Phase 2）。

## 已修复的关键链路（2026-06-10 第二轮）

- [x] 重启后 Scraper 失效（启动时恢复 provider 注册）
- [x] 临时元数据写读路径不一致（统一为 `temp/{库}/{系统}/metadata.pegasus.txt`）
- [x] 多源聚合 source_id 跨源串用
- [x] ScreenScraper 评分 0~20 未归一化
- [x] Hash 匹配从未接线（crc32fast/md5/sha1 依赖已声明但未使用）
- [x] 批量抓取不下载媒体
- [x] 导出 ES 时 gamelist.xml 缺 `<image>` 字段

## 低优先级

- [x] 清理过期 TODO 注释
  - `src/pages/Library.tsx` 点击逻辑注释已改为普通说明。

- [x] 整理审计文档命名
  - `docs/AUTID_FIX.md` 已更名为 `docs/AUDIT_GUIDE.md`。

- [x] 清理旧导入接口英文错误信息
  - `src-tauri/src/commands/import.rs`、`export.rs` 的兼容接口错误信息已改为简体中文。

## 环境准备

- [x] 确认当前开发环境可运行 `pnpm`
- [x] 确认当前开发环境可运行 `cargo`
- [x] 基础验证通过：
  - `pnpm build` ✅
  - `cd server && pnpm build` ✅
  - `cd src-tauri && cargo check` ✅
