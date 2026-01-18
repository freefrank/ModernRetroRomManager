//! 标准化数据结构 - ScraperManager 统一输入输出类型

use serde::{Deserialize, Serialize};

// ============================================================================
// 输入类型
// ============================================================================

/// ROM Hash 信息，用于精确匹配
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RomHash {
    pub crc32: Option<String>,
    pub md5: Option<String>,
    pub sha1: Option<String>,
}

/// Scrape 查询参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapeQuery {
    /// 游戏名（解析自文件名）
    pub name: String,
    /// 系统标识 (snes, psx, gba...)
    pub system: Option<String>,
    /// ROM Hash (用于精确匹配)
    pub hash: Option<RomHash>,
    /// 原始文件名
    pub file_name: String,
}

impl ScrapeQuery {
    pub fn new(name: String, file_name: String) -> Self {
        Self {
            name,
            system: None,
            hash: None,
            file_name,
        }
    }

    pub fn with_system(mut self, system: impl Into<String>) -> Self {
        self.system = Some(system.into());
        self
    }

    pub fn with_hash(mut self, hash: RomHash) -> Self {
        self.hash = Some(hash);
        self
    }
}

// ============================================================================
// 输出类型 - 搜索结果
// ============================================================================

/// 搜索结果（来自单个 provider）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// 来源 provider 标识
    pub provider: String,
    /// provider 内部 ID
    pub source_id: String,
    /// 游戏名
    pub name: String,
    /// 发行年份
    pub year: Option<String>,
    /// 系统/平台
    pub system: Option<String>,
    /// 缩略图 URL
    pub thumbnail: Option<String>,
    /// 匹配置信度 0.0-1.0
    pub confidence: f32,
}

// ============================================================================
// 输出类型 - 游戏元数据
// ============================================================================

/// 标准化游戏元数据
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GameMetadata {
    pub name: String,
    pub english_name: Option<String>,
    pub description: Option<String>,
    pub release_date: Option<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub genres: Vec<String>,
    pub players: Option<String>,
    pub rating: Option<f64>,
}

// ============================================================================
// 输出类型 - 媒体资产
// ============================================================================

/// 媒体类型枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    BoxFront,
    BoxBack,
    Box3D,
    Screenshot,
    TitleScreen,
    Logo,
    Icon,
    Hero,
    Banner,
    Video,
    Manual,
    Other,
}

impl MediaType {
    /// 从字符串解析媒体类型
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "boxfront" | "box-2d" | "box-front" | "cover" => MediaType::BoxFront,
            "boxback" | "box-back" => MediaType::BoxBack,
            "box3d" | "box-3d" => MediaType::Box3D,
            "screenshot" | "ss" => MediaType::Screenshot,
            "titlescreen" | "title" => MediaType::TitleScreen,
            "logo" | "wheel" | "clearlogo" => MediaType::Logo,
            "icon" => MediaType::Icon,
            "hero" => MediaType::Hero,
            "banner" => MediaType::Banner,
            "video" => MediaType::Video,
            "manual" => MediaType::Manual,
            _ => MediaType::Other,
        }
    }

    /// 转换为字符串
    pub fn as_str(&self) -> &'static str {
        match self {
            MediaType::BoxFront => "boxfront",
            MediaType::BoxBack => "boxback",
            MediaType::Box3D => "box3d",
            MediaType::Screenshot => "screenshot",
            MediaType::TitleScreen => "titlescreen",
            MediaType::Logo => "logo",
            MediaType::Icon => "icon",
            MediaType::Hero => "hero",
            MediaType::Banner => "banner",
            MediaType::Video => "video",
            MediaType::Manual => "manual",
            MediaType::Other => "other",
        }
    }
}

/// 媒体资产
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaAsset {
    /// 来源 provider
    pub provider: String,
    /// 资源 URL
    pub url: String,
    /// 媒体类型
    pub asset_type: MediaType,
    /// 宽度
    pub width: Option<u32>,
    /// 高度
    pub height: Option<u32>,
}

// ============================================================================
// 聚合结果
// ============================================================================

/// Scrape 聚合结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScrapeResult {
    /// 聚合后的元数据
    pub metadata: GameMetadata,
    /// 所有媒体资产
    pub media: Vec<MediaAsset>,
    /// 数据来源列表
    pub sources: Vec<String>,
}

// ============================================================================
// Provider 能力
// ============================================================================

use std::collections::HashSet;

/// Provider 支持的能力
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderCapability {
    /// 支持名称搜索
    Search,
    /// 支持 Hash 精确匹配
    HashLookup,
    /// 提供元数据
    Metadata,
    /// 提供媒体资产
    Media,
}

/// Provider 能力集合
#[derive(Debug, Clone, Default)]
pub struct Capabilities {
    inner: HashSet<ProviderCapability>,
}

impl Capabilities {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, cap: ProviderCapability) -> Self {
        self.inner.insert(cap);
        self
    }

    pub fn has(&self, cap: ProviderCapability) -> bool {
        self.inner.contains(&cap)
    }

    pub fn all(&self) -> &HashSet<ProviderCapability> {
        &self.inner
    }
}
