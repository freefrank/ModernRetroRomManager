# Updater 实施计划(启动检测 GitHub 新版本并提醒)

> 状态:待实施。已与用户确认取向;暂缓,优先处理其它 bug。

## 已确认取向
- **仅提醒 + 打开下载页**(不集成 Tauri 自动更新器,不需签名密钥/更新清单)。
- **启动 Toast + 关于页徽标**。
- **每天最多一次 + 可忽略此版本**。
- 桌面(Tauri)模式专用;Web/server 模式跳过。普通功能提交,不 bump 版本。

## 行为总览
- 启动后异步、非阻塞检查,失败静默。
- 数据源:`GET https://api.github.com/repos/freefrank/ModernRetroRomManager/releases/latest`(只返回稳定版,排除 draft/prerelease)。
- 命中新版 → 启动弹一次可关闭 Toast:**前往下载** / **忽略此版本**;关于页常驻徽标 + **立即检查**。

## 后端(Rust)

### 新增 `src-tauri/src/commands/updater.rs`
- 命令 `check_for_update() -> Result<UpdateInfo, String>`:
  - `reqwest` GET,请求头 `User-Agent: MRRM-Updater`、`Accept: application/vnd.github+json`,超时 ~10s。
  - 解析 `tag_name` / `html_url` / `name` / `body` / `published_at`。
  - 版本比较:剥 `v` 前缀后按 `.` 分段数值比较,与 `env!("CARGO_PKG_VERSION")` 比。
  - 返回结构:
    ```rust
    struct UpdateInfo {
      current_version: String,
      latest_version: String,
      is_update_available: bool,
      release_url: String,
      release_notes: String,   // 可截断
      published_at: String,
    }
    ```
  - 出错(网络/限流/解析)返回 `Err(String)`,前端静默处理。
- 命令 `open_url(url: String)`:用已初始化的 `opener` 插件 `app.opener().open_url(url, None)` 打开系统浏览器(供 Toast「前往下载」)。
- **单元测试**:版本比较(`v` 前缀、多位数如 `1.4.10 > 1.4.9`、相等、更旧不提示)。

### 改动
- `src-tauri/src/commands/mod.rs`:挂 `pub mod updater;`。
- `src-tauri/src/lib.rs`:`invoke_handler!` 追加 `check_for_update`、`open_url`。
- **检查点**:`opener` 已在 `rom.rs` 用于 `open_rom_location`,但 `open_url` 可能需在 capabilities 里补 `opener:allow-open-url` 权限——实施时确认。

## 前端

- `src/types/index.ts`:加 `UpdateInfo` 接口。
- `src/lib/api.ts`:
  - `checkForUpdate(): Promise<UpdateInfo | null>` → `isTauri()` 才 `invoke("check_for_update")`,否则 `null`。
  - `openUrl(url)` → `invoke("open_url")`。
- 新增 `src/hooks/useUpdateCheck.ts`(封装节流 + 忽略):
  - 节流:app 设置键 `update_last_check`(`yyyy-mm-dd`),同日已查则跳过。
  - 忽略:app 设置键 `update_skipped_version`;`latest === skipped` 时不弹 Toast。
  - 复用 `get_app_settings` / `update_app_setting` 持久化。
- `src/App.tsx` 启动 effect(仅 `isTauri()`):命中新版且未被忽略 → `toast` 带两动作(前往下载 = `api.openUrl(release_url)`;忽略此版本 = 写 `update_skipped_version`)。
- `src/pages/settings/AboutSection.tsx`:加「立即检查」按钮(绕过节流强制查)+ 状态行(检查中 / 已是最新 / 发现新版本 X / 检查失败)+ 有新版时版本号旁徽标。
- i18n:8 语言 `i18n/locales/*/settings.json` 补 `settings.update.*`(checking / upToDate / available / download / skip / checkNow / failed 等)。

## 关键取舍与边界
- 未认证 GitHub API 限流 60 次/时/IP,每天一次绰绰有余;UA 头必带否则 403。
- `latest <= current`(含 CI 异常 tag)一律不提示。
- 检查/网络失败时启动侧完全静默,只有关于页手动检查才显示「检查失败」。

## 涉及文件清单
| 侧 | 文件 |
|---|---|
| 后端 | 新增 `commands/updater.rs`;改 `commands/mod.rs`、`lib.rs` |
| 前端 | 改 `lib/api.ts`、`types/index.ts`、`App.tsx`、`pages/settings/AboutSection.tsx`;新增 `hooks/useUpdateCheck.ts` |
| i18n | 8 × `i18n/locales/*/settings.json` |
| 测试 | `updater.rs` 版本比较单测 |

**工作量**:小到中等,单条独立功能,可一次实现完。
