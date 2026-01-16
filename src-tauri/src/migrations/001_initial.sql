-- 游戏系统/平台表
CREATE TABLE IF NOT EXISTS systems (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  short_name TEXT NOT NULL,
  manufacturer TEXT,
  release_year INTEGER,
  extensions TEXT NOT NULL,
  igdb_platform_id INTEGER,
  thegamesdb_platform_id INTEGER
);

-- ROM 表
CREATE TABLE IF NOT EXISTS roms (
  id TEXT PRIMARY KEY NOT NULL,
  filename TEXT NOT NULL,
  path TEXT NOT NULL,
  system_id TEXT NOT NULL REFERENCES systems(id),
  size INTEGER NOT NULL,
  crc32 TEXT,
  md5 TEXT,
  sha1 TEXT,
  created_at TEXT NOT NULL DEFAULT (datetime('now')),
  updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- ROM 元数据表
CREATE TABLE IF NOT EXISTS rom_metadata (
  rom_id TEXT PRIMARY KEY NOT NULL REFERENCES roms(id) ON DELETE CASCADE,
  name TEXT NOT NULL,
  description TEXT,
  release_date TEXT,
  developer TEXT,
  publisher TEXT,
  genre TEXT,
  players INTEGER,
  rating REAL,
  region TEXT,
  scraper_source TEXT,
  scraped_at TEXT
);

-- 媒体资产表
CREATE TABLE IF NOT EXISTS media_assets (
  id TEXT PRIMARY KEY NOT NULL,
  rom_id TEXT NOT NULL REFERENCES roms(id) ON DELETE CASCADE,
  asset_type TEXT NOT NULL,
  path TEXT NOT NULL,
  width INTEGER,
  height INTEGER,
  file_size INTEGER,
  source_url TEXT,
  downloaded_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- API 密钥配置表
CREATE TABLE IF NOT EXISTS api_configs (
  id TEXT PRIMARY KEY NOT NULL,
  provider TEXT NOT NULL,
  api_key TEXT,
  api_secret TEXT,
  username TEXT,
  password TEXT,
  enabled INTEGER NOT NULL DEFAULT 1,
  priority INTEGER NOT NULL DEFAULT 0
);

-- 扫描目录配置表
CREATE TABLE IF NOT EXISTS scan_directories (
  id TEXT PRIMARY KEY NOT NULL,
  path TEXT NOT NULL,
  system_id TEXT REFERENCES systems(id),
  recursive INTEGER NOT NULL DEFAULT 1,
  enabled INTEGER NOT NULL DEFAULT 1,
  last_scan TEXT
);

-- 索引
CREATE INDEX IF NOT EXISTS idx_roms_system ON roms(system_id);
CREATE INDEX IF NOT EXISTS idx_roms_hash ON roms(crc32, md5, sha1);
CREATE INDEX IF NOT EXISTS idx_media_rom ON media_assets(rom_id);
CREATE INDEX IF NOT EXISTS idx_media_type ON media_assets(asset_type);
CREATE INDEX IF NOT EXISTS idx_scan_dir_system ON scan_directories(system_id);
