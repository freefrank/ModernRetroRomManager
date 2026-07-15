# ModernRetroRomManager

简体中文 | [English](./README.md)

ModernRetroRomManager（MRRM）是一款现代化、跨平台、开源的 ROM 游戏库管理器。项目采用 React 界面和原生 Rust/Tauri 后端，目标是提供比 ARRM、Skraper 等传统工具更轻量、易用的管理体验。

## 功能特性

- 扫描并整理多平台 ROM 游戏库
- 管理多个相互独立的游戏库，支持改名，并可从设置页或 ROM 侧栏切换当前游戏库
- 使用持久索引、按系统延迟加载和自动增量扫描快速打开大型游戏库
- 可随时执行带当前系统和进度提示的全量刷新，并在 Console 中查看结构化日志
- 通过 ROM Header、序列号和内置数据库识别支持的卡带游戏
- 对文件名型平台和魔改 ROM 集进行名称标准化匹配
- 聚合多个元数据 Provider，并根据平台匹配度、资源完整度进行置信度排序和故障切换
- 支持单游戏、已选游戏、整个平台或整个 ROM 库批量刮削，并可选择忽略已有元数据、资产与缓存进行全量重新抓取
- 可选择需要下载的媒体类型，跨 ROM 库复用持久抓取缓存，并保留备选图片供用户确认
- 可通过自定义 OpenAI-compatible 接口翻译单个游戏、已选游戏或整个平台的 metadata
- 可选择当前或指定游戏库，按 EmulationStation `gamelist.xml` 与 Pegasus `metadata.pegasus.txt` 格式导出元数据及相关媒体资源
- 在支持日志级别筛选的底部 Console 中查看扫描、抓取、Provider 和错误信息
- 支持导入自定义主题，并提供响应式暗色界面；Retro 主题内置 Zpix，统一显示中英文像素字体
- 界面支持简体中文、繁体中文、英语、法语、德语、意大利语、西班牙语和俄语
- 基于 Tauri 2 支持 Windows、Linux 和 macOS

## 游戏库模型

每个已配置的扫描目录都是一个独立游戏库，同一时间只有一个游戏库处于激活状态。游戏库根目录下可以包含不同 Console System；每个游戏库拥有稳定 ID、可编辑名称和独立持久索引，切回已经扫描过的游戏库时可直接复用其索引。旧版单目录或多目录配置会自动迁移，并默认激活原列表中的第一条目录。

## 元数据来源

- [ScreenScraper](https://www.screenscraper.fr/)
- [SteamGridDB](https://www.steamgriddb.com/)
- [TheGamesDB](https://thegamesdb.net/)

可以在应用设置中配置 Provider 凭据、请求速率、并发线程、优先级和可选媒体类型。
MRRM 已内置 ScreenScraper 应用开发者凭据，用户只需填写自己的 ScreenScraper 会员用户名和密码。

AI Metadata 翻译可单独配置端点、API Key、模型和目标语言。请求由原生后端发送，译文先保存为预览 metadata，确认后再导出到游戏库。

## 技术栈

- 前端：React 19、TypeScript、Tailwind CSS 4、Vite
- 后端：Rust、Tauri 2
- 包管理：pnpm、Cargo

## 开发运行

环境要求：

- 最新 Node.js LTS
- pnpm
- 最新稳定版 Rust 工具链
- 当前操作系统所需的 Tauri 系统依赖

安装依赖并启动桌面开发版本：

```bash
pnpm install
pnpm tauri dev
```

构建生产版本：

```bash
pnpm tauri build
```

推送 `v*` tag 后，CI 会构建单文件 Windows x64 portable EXE、Windows 安装程序、Microsoft Store MSIX 和 Linux x86_64 AppImage，并自动正式发布对应的 GitHub Release，不保留为 Draft。

运行时，若程序旁边已经存在 `config` 文件夹，会优先作为 portable 配置目录；否则回退到当前用户的 AppData 应用数据目录。内置匹配数据会打进可执行文件，并在需要时释放到所选配置目录。

版本号、更新日志和 portable 发布流程见 [docs/release-process.md](./docs/release-process.md)。发布前运行 `pnpm release:check`。

## 数据与致谢

- [yingw/rom-name-cn](https://github.com/yingw/rom-name-cn) 提供了宝贵的 ROM 中英文名称对照数据。
- [emu.jy6d.com](http://emu.jy6d.com/dz/) 提供了补充的游戏中英文名称数据。

## 许可证

MIT License
