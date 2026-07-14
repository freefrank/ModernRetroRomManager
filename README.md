# ModernRetroRomManager

[简体中文](./README.zh-CN.md) | English

ModernRetroRomManager (MRRM) is a modern, cross-platform, open-source ROM library manager. It is designed as a lightweight alternative to tools such as ARRM and Skraper, with a React interface and a native Rust/Tauri backend.

## Features

- Scan and organize multi-platform ROM libraries
- Manage multiple independent libraries, rename them, and switch the active library from Settings or the ROM sidebar
- Open large libraries quickly with a persistent index, per-system lazy loading, and automatic incremental scans
- Run a visible full refresh when needed, with current-system progress and structured Console logs
- Identify supported cartridge games from ROM headers, serials, and embedded databases
- Match filename-based platforms and modified ROM sets with normalized game names
- Search multiple metadata providers, with platform-aware confidence ranking and fallback handling
- Scrape individual games, selected games, an entire platform, or the complete ROM library
- Select optional media types, reuse persistent scrape results across libraries, and cache alternative artwork for review
- Export metadata and related media for EmulationStation `gamelist.xml` and Pegasus `metadata.pegasus.txt`
- Inspect scan, scrape, provider, and error messages in a level-filtered bottom Console
- Import custom themes and use the built-in responsive dark interface
- Run on Windows, Linux, and macOS through Tauri 2

## Library model

Each configured scan directory is an independent library. One library is active at a time, and its child folders may contain different console systems. Libraries have stable IDs, editable names, and separate persistent ROM indexes, so switching back to a previously scanned library can reuse its index. Existing single-directory and multi-directory settings are migrated automatically; the first existing directory becomes active.

## Metadata providers

- [ScreenScraper](https://www.screenscraper.fr/)
- [SteamGridDB](https://www.steamgriddb.com/)
- [TheGamesDB](https://thegamesdb.net/)

Provider credentials, rate limits, worker threads, priority, and optional media types can be configured in the application settings.
ScreenScraper application credentials are bundled with MRRM, so users only need to enter their own ScreenScraper member username and password.

## Technology

- Frontend: React 19, TypeScript, Tailwind CSS 4, Vite
- Backend: Rust, Tauri 2
- Package management: pnpm and Cargo

## Development

Requirements:

- Node.js 20 or later
- pnpm
- Latest stable Rust toolchain
- Tauri system prerequisites for your operating system

Install dependencies and start the desktop development build:

```bash
pnpm install
pnpm tauri dev
```

Create a production build:

```bash
pnpm tauri build
```

The versioning, changelog, and portable release workflow is documented in [docs/release-process.md](./docs/release-process.md). Run `pnpm release:check` before publishing a release.

## Data acknowledgements

- [yingw/rom-name-cn](https://github.com/yingw/rom-name-cn) provides valuable Chinese and English ROM name mappings.
- [emu.jy6d.com](http://emu.jy6d.com/dz/) provides additional bilingual game-name data.

## License

MIT License
