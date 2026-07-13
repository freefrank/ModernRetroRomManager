# ModernRetroRomManager

[简体中文](./README.zh-CN.md) | English

ModernRetroRomManager (MRRM) is a modern, cross-platform, open-source ROM library manager. It is designed as a lightweight alternative to tools such as ARRM and Skraper, with a React interface and a native Rust/Tauri backend.

## Features

- Scan and organize multi-platform ROM libraries
- Identify supported cartridge games from ROM headers, serials, and embedded databases
- Match filename-based platforms and modified ROM sets with normalized game names
- Search multiple metadata providers, with platform-aware confidence ranking and fallback handling
- Scrape individual games, selected games, an entire platform, or the complete ROM library
- Select optional media types and cache alternative artwork for review
- Read and write EmulationStation `gamelist.xml` and Pegasus `metadata.pegasus.txt`
- Import custom themes and use the built-in responsive dark interface
- Run on Windows, Linux, and macOS through Tauri 2

## Metadata providers

- [ScreenScraper](https://www.screenscraper.fr/)
- [SteamGridDB](https://www.steamgriddb.com/)
- [TheGamesDB](https://thegamesdb.net/)

Provider credentials, rate limits, worker threads, priority, and optional media types can be configured in the application settings.

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

## Data acknowledgements

- [yingw/rom-name-cn](https://github.com/yingw/rom-name-cn) provides valuable Chinese and English ROM name mappings.
- [emu.jy6d.com](http://emu.jy6d.com/dz/) provides additional bilingual game-name data.

## License

MIT License
