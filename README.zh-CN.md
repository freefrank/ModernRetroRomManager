# ModernRetroRomManager

简体中文 | [English](./README.md)

ModernRetroRomManager（MRRM）是一款现代化、跨平台、开源的 ROM 游戏库管理器。项目采用 React 界面和原生 Rust/Tauri 后端，目标是提供比 ARRM、Skraper 等传统工具更轻量、易用的管理体验。

## 功能特性

- 扫描并整理多平台 ROM 游戏库
- 使用持久索引、按系统延迟加载和自动增量扫描快速打开大型游戏库
- 可随时执行带当前系统和进度提示的全量刷新，并在 Console 中查看结构化日志
- 通过 ROM Header、序列号和内置数据库识别支持的卡带游戏
- 对文件名型平台和魔改 ROM 集进行名称标准化匹配
- 聚合多个元数据 Provider，并根据平台匹配度、资源完整度进行置信度排序和故障切换
- 支持单游戏、已选游戏、整个平台或整个 ROM 库批量刮削
- 可选择需要下载的媒体类型，跨 ROM 库复用持久抓取缓存，并保留备选图片供用户确认
- 按 EmulationStation `gamelist.xml` 与 Pegasus `metadata.pegasus.txt` 格式导出元数据及相关媒体资源
- 在支持日志级别筛选的底部 Console 中查看扫描、抓取、Provider 和错误信息
- 支持导入自定义主题，并提供响应式暗色界面
- 基于 Tauri 2 支持 Windows、Linux 和 macOS

## 游戏库模型

0.5 版本将一个已配置的 ROM 根目录视为当前活动游戏库。根目录下可以包含不同 Console System，各系统会独立建立索引并刷新。多个独立游戏库的管理和切换计划在后续版本实现。

## 元数据来源

- [ScreenScraper](https://www.screenscraper.fr/)
- [SteamGridDB](https://www.steamgriddb.com/)
- [TheGamesDB](https://thegamesdb.net/)

可以在应用设置中配置 Provider 凭据、请求速率、并发线程、优先级和可选媒体类型。
MRRM 已内置 ScreenScraper 应用开发者凭据，用户只需填写自己的 ScreenScraper 会员用户名和密码。

## 技术栈

- 前端：React 19、TypeScript、Tailwind CSS 4、Vite
- 后端：Rust、Tauri 2
- 包管理：pnpm、Cargo

## 开发运行

环境要求：

- Node.js 20 或更高版本
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

版本号、更新日志和 portable 发布流程见 [docs/release-process.md](./docs/release-process.md)。发布前运行 `pnpm release:check`。

## 数据与致谢

- [yingw/rom-name-cn](https://github.com/yingw/rom-name-cn) 提供了宝贵的 ROM 中英文名称对照数据。
- [emu.jy6d.com](http://emu.jy6d.com/dz/) 提供了补充的游戏中英文名称数据。

## 许可证

MIT License
