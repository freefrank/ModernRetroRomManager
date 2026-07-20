# 归档文档

本目录存放**已完成或已过期的一次性文档**——历史审计清单、已实施的重构计划等。它们记录了项目某个阶段的决策与执行,保留供追溯,但**不再反映当前代码现状**,请勿据此开发。

当前有效文档见 `docs/` 根目录与仓库根的 `README.md` / `README.zh-CN.md` / `CHANGELOG.md`。

## 清单

| 文档 | 归档原因 |
|------|----------|
| `AUDIT_FIX.md` | 2026-04 的深度代码审计与修复计划,列出的 Pending Action Items(命名匹配索引化、`local_cn` 元数据补全、前端搜索过滤优化等)均已落实。 |
| `AUTID_FIX.md` | 2026-01 基于早期 `plan.md` 的审计清单(文件名为当时笔误 "AUTID")。所列关键风险(API 鉴权、导入导出下线、版本注入、Scraper 配置闭环等)均已修复。 |
| `2026-07-12-frontend-refactor-plan.md` | 2026-07 前端重构的逐任务执行计划(主题包体系、UI 基件库、Library 货架布局、Scraper 配置闭环)。重构已完成并合入。其**设计契约** `docs/superpowers/specs/2026-07-12-frontend-refactor-design.md` 仍为活跃参考,未归档。 |
