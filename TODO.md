# TODO

> 当前聚焦**桌面版（Tauri）**开发，Web 后端相关任务整体挂起（见文末）。

## 高优先级

- [ ] 批量 Scrape 任务队列与进度反馈
  - 位置：`docs/plan.md` 路线图 Phase 2
  - 需要设计任务队列、取消/重试、进度事件和 UI 状态。
  - 已有基础：扫描停止、媒体候选缓存复用、批量抓取匹配（PS1 序列号 / N64 卡带头）。

- [ ] 更多 Scraper Provider 集成
  - 目标：IGDB、MobyGames，降低对 screenscraper.fr 的依赖。
  - 复用现有 `ScraperManager` 多源聚合与优先级机制。

## 中优先级

- [ ] 完善拖拽添加目录校验
  - 位置：`src/components/layout/Layout.tsx`（当前 `Layout.tsx:30` 仍注释“假设用户拖拽的是文件夹”）。
  - 拖拽事件循环处理多个路径，默认假设都是文件夹。
  - 建议调用后端 `validate_path`，或让 `add_directory` 返回明确中文错误。

## 低优先级

- [ ] 清理旧导入接口英文错误信息
  - 位置：`src-tauri/src/commands/import.rs:6,11`
  - 两个 deprecated 兼容入口的错误信息仍是英文，应改为简体中文。

- [ ] 整理审计文档命名
  - 当前同时存在 `docs/AUDIT_FIX.md` 与 `docs/AUTID_FIX.md`。
  - `AUTID_FIX.md` 疑似拼写错误，确认后合并或改名。

## 环境准备

- [ ] 恢复后执行基础验证：
  - `pnpm build`
  - `cd src-tauri && cargo check`

---

## 已完成（自上次整理起）

- [x] 导入/导出页面接口对齐：`Import.tsx` 已改用 `exportLibraryData`，不再调用旧 `export_to_emulationstation`。
- [x] ScreenScraper systemid 映射：`screenscraper.rs` 已提供完整 `system_id()` 平台别名映射表。
- [x] 清理 `src/pages/Library.tsx` 中的过期 TODO 注释。

## 挂起（Web 后端，暂不推进）

- [ ] Web 后端补齐 EmulationStation 解析（`server/src/rom-service.ts:415` 仍为 TODO，退回普通 ROM 扫描）。
- [ ] Web 版用户权限管理。
- [ ] Web 版在线导出与 ZIP 打包下载。
