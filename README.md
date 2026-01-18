# ModernRetroRomManager - 现代化 Retro ROM 管理软件

打造一款**现代化、跨平台、开源**的 Retro ROM 管理软件，替代老旧的 ARRM 和 Skraper。

## ✨ 特性

- 🎨 **现代化 UI**: 基于 React + TailwindCSS 的 Cyberpunk 风格界面
- ⚡ **高性能**: Rust (Tauri) 后端，原生级性能，极低的资源占用
- 🎮 **多系统支持**: 预置 NES, SNES, PSX, GBA 等 17+ 种主流游戏系统
- 📄 **Metadata 驱动**: 直接读取 `gamelist.xml` 或 `metadata.txt`，无需本地数据库
- 🌐 **跨平台**: 支持 Windows, macOS, Linux

## 🏗️ 技术栈

- **Frontend**: React 19, TypeScript, TailwindCSS v4, Framer Motion, Lucide React
- **Backend**: Rust, Tauri v2
- **Tooling**: Vite, pnpm, Cargo

## 📚 资源与致谢

- **中文 ROM 数据库**: [yingw/rom-name-cn](https://github.com/yingw/rom-name-cn) - 提供了极其宝贵的 ROM 中英文对照数据，使得本软件能够实现精确的中文名匹配与搜索。
- **Scraper 数据源**:
  - [ScreenScraper](https://www.screenscraper.fr/)
  - [SteamGridDB](https://www.steamgriddb.com/)

## 🚀 快速开始

### 前置要求

- Node.js (v20+)
- pnpm
- Rust (最新稳定版)

### 安装依赖

```bash
pnpm install
```

### 开发模式运行

```bash
pnpm tauri dev
```

### 构建生产版本

```bash
pnpm tauri build
```

## 📄 许可证

MIT License
