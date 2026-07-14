# 版本与发布流程

本项目使用 Codex 项目级 Agent Hook（`.codex/hooks.json`）约束版本和发布文档流程，不安装或修改 Git Hook。首次用 Codex 打开项目时，需要审阅并信任该 Hook。

## 日常开发

普通功能或修复完成后小步提交，不修改版本号。Commit message 使用简体中文；除非用户明确要求，否则 Agent 不得 push 或创建 tag。

## 准备发布

1. 运行 `pnpm release:bump -- <版本号>`。脚本会同步：
   - `package.json`
   - `src-tauri/tauri.conf.json`
   - `src-tauri/Cargo.toml`
   - `src-tauri/Cargo.lock`
2. 根据上一个版本之后的中文 commits 填写 `CHANGELOG.md`，删除自动生成的“待补充”。
3. 检查用户可见行为、安装方式、支持平台或配置是否变化；如有变化，同步更新英文 `README.md`、中文 `README.zh-CN.md` 和相关 `docs/` 文档。
4. 运行前端测试、Rust 测试和 `pnpm release:check`。
5. 构建 Windows portable，并确认产物目录名、应用内版本号和文件名一致。
6. 只有用户明确要求后才 push；只有用户明确要求发布时才创建 `v<版本号>` tag。

## 自动构建与 GitHub Release

推送 `v*` tag 后，GitHub Actions 会并行构建 Windows x64 portable ZIP 和 Linux x86_64 AppImage。两项构建均成功后，CI 从 `CHANGELOG.md` 提取对应版本说明，创建 GitHub Release 并上传两个产物。

也可以在 Actions 页面手动运行该 workflow 仅验证和下载构建产物；手动运行不会创建 Release。

## Agent Hook 的作用

- 当请求提到版本、构建、文档或发布时，向 Agent 注入上述流程。
- Agent 执行包含版本文件的 `git commit` 前，检查四个版本文件与 `CHANGELOG.md` 是否全部暂存且一致。
- Agent 执行 `git push` 或 `git tag` 前，再次检查版本一致性和当前版本的 CHANGELOG 内容。
- Hook 不会自动提交、push、打 tag，也不会影响用户在 Codex 之外手动运行 Git。

Hook 只是流程守卫；可随时手动运行 `pnpm release:check` 进行同样的发布检查。
