# ModernRetroRomManager

[简体中文](./README.zh-CN.md) | English | [Privacy Policy / 隐私政策](./PRIVACY.md)

ModernRetroRomManager (MRRM) is a modern, cross-platform, open-source ROM library manager. It is designed as a lightweight alternative to tools such as ARRM and Skraper, with a React interface and a native Rust/Tauri backend.

## Features

- Scan and organize multi-platform ROM libraries
- Organize multi-disc games automatically on save: each disc is moved into one subfolder, an `.m3u` playlist is generated, the library and export collapse to a single entry, and RetroArch/batocera can swap discs (recognizes `(Disc N)`, `Disc A/B`, `CD1/CD2`, and Chinese markers like `第1碟`)
- Manage multiple independent libraries, rename them, and switch the active library from Settings or the ROM sidebar
- Open large local, SD-card, and Samba libraries quickly with an immediate persistent index, per-system lazy loading, and non-blocking background incremental scans
- Run a visible full refresh when needed, with current-system progress and structured Console logs
- Identify supported cartridge games from ROM headers, serials, and embedded databases
- Match filename-based platforms and modified ROM sets with normalized game names
- Search multiple metadata providers, with platform-aware confidence ranking and fallback handling
- Scrape individual games, selected games, an entire platform, or the complete ROM library; optionally force a full rescrape that bypasses existing metadata, assets, and caches
- Select optional media types, reuse persistent scrape results across libraries, and cache alternative artwork for review
- Translate metadata for one game, a selection, or an entire platform through a configurable OpenAI-compatible endpoint, with merged requests, reasoning-effort control, adaptive retries, and per-language completion markers
- Optional AI name-resolution fallback for batch scraping: translated ROMs that cannot be resolved locally are matched by the LLM against the built-in No-Intro title list, with local result caching
- Export the active or a selected library for EmulationStation or Pegasus with file-level progress, live speed, cancellation, same-size skipping, and an optional ROM-and-assets-only filter
- Inspect scan, scrape, provider, and error messages in a level-filtered bottom Console
- ROM library right-click menu to open file location, hide/unhide, and permanently delete; hidden ROMs are neither shown nor synced on export
- Optional one-way export sync deletes ROMs at the target that are hidden/removed in the library (BIOS always kept), keeping the target in sync
- Import custom themes and use the built-in responsive dark interface; the Retro theme bundles Zpix for consistent English and Chinese pixel text
- Use the interface in Simplified Chinese, Traditional Chinese, English, French, German, Italian, Spanish, or Russian
- Run on Windows, Linux, and macOS through Tauri 2

## Library model

Each configured scan directory is an independent library. One library is active at a time, and its child folders may contain different console systems. Libraries have stable IDs, editable names, and separate persistent ROM indexes, so switching back to a previously scanned library can reuse its index. Existing single-directory and multi-directory settings are migrated automatically; the first existing directory becomes active.

## Metadata providers

- [ScreenScraper](https://www.screenscraper.fr/)
- [SteamGridDB](https://www.steamgriddb.com/)
- [TheGamesDB](https://thegamesdb.net/)

Provider credentials, rate limits, worker threads, priority, and optional media types can be configured in the application settings.
ScreenScraper application credentials are bundled with MRRM, so users only need to enter their own ScreenScraper member username and password.

AI metadata translation is configured separately with a custom endpoint, API key, model, target language, reasoning effort, and batch context limit. Requests are sent by the native backend; incomplete structured responses are split and retried, while per-language markers prevent completed metadata from being translated again.

## Technology

- Frontend: React 19, TypeScript, Tailwind CSS 4, Vite
- Backend: Rust, Tauri 2
- Package management: pnpm and Cargo

## Development

Requirements:

- The latest Node.js LTS
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

Pushing a `v*` tag triggers CI builds for a self-contained Windows x64 portable EXE, a Windows installer, a Microsoft Store MSIX, and a Linux x86_64 AppImage. The matching GitHub Release is published automatically rather than left as a draft.

At runtime, an existing `config` folder beside the executable has the highest priority for portable use. If it does not exist, MRRM stores configuration under the current user's application-data directory. Bundled lookup data is embedded in the executable and extracted into the selected configuration directory when needed.

The versioning, changelog, and portable release workflow is documented in [docs/release-process.md](./docs/release-process.md). Run `pnpm release:check` before publishing a release.

## Data acknowledgements

- [yingw/rom-name-cn](https://github.com/yingw/rom-name-cn) provides valuable Chinese and English ROM name mappings.
- [emu.jy6d.com](http://emu.jy6d.com/dz/) provides additional bilingual game-name data.

## License

MIT License
