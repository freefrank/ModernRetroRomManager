# TODO

## 高优先级

- [ ] 修复导入/导出页面与后端接口不一致
  - `src/pages/Import.tsx` 仍调用旧接口 `export_to_emulationstation`。
  - Tauri 端实际入口应使用 `export_scraped_data`。
  - 同步处理页面中的英文文案，统一进入 i18n，并使用简体中文。

- [ ] Web 后端补齐 EmulationStation 解析
  - 位置：`server/src/rom-service.ts`
  - 当前 `readEmulationStationRoms` 仍是 TODO，会退回普通 ROM 文件扫描。
  - 需要解析 `gamelist.xml`，并尽量与 Tauri 端返回字段保持一致。

- [ ] 修复 ScreenScraper systemid 映射
  - 位置：`src-tauri/src/scraper/screenscraper.rs`
  - 当前 hash 查询把本地 system 字符串直接传给 `systemeid`。
  - 需要建立本地系统 ID 到 ScreenScraper systemid 的映射。

## 中优先级

- [ ] 完善拖拽添加目录校验
  - 位置：`src/components/layout/Layout.tsx`
  - 当前拖拽事件循环处理多个路径，但默认假设路径都是文件夹。
  - 建议调用后端 `validate_path` 或让 `add_directory` 返回明确中文错误。

- [ ] 批量 Scrape 任务队列与进度反馈
  - 位置：`docs/plan.md` 路线图 Phase 2
  - 需要设计任务队列、取消/重试、进度事件和 UI 状态。

## 低优先级

- [ ] 清理过期 TODO 注释
  - `src/pages/Library.tsx` 中点击逻辑注释已与现状基本一致，可以删除 TODO 或改成普通说明。

- [ ] 整理审计文档命名
  - 当前同时存在 `docs/AUDIT_FIX.md` 和 `docs/AUTID_FIX.md`。
  - `AUTID_FIX.md` 疑似拼写错误，后续确认后可合并或改名。

- [ ] 清理旧导入接口英文错误信息
  - 位置：`src-tauri/src/commands/import.rs`
  - 当前接口保留为 deprecated 兼容入口，但错误信息是英文。
  - 若保留接口，应改为简体中文。

## 环境准备

- [ ] 确认当前开发环境可运行 `pnpm`
- [ ] 确认当前开发环境可运行 `cargo`
- [ ] 恢复后执行基础验证：
  - `pnpm build`
  - `cd src-tauri && cargo check`

