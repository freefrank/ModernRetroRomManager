//! Pegasus metadata file parser and exporter
//! 
//! Format: key: value pairs
//! - Lines starting with # are comments
//! - Empty lines are ignored
//! - Multiline values start with space/tab
//! - `collection:` defines a collection
//! - `game:` defines a game entry

use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};
use encoding_rs::GBK;
use chardetng::EncodingDetector;

/// Pegasus collection entry
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PegasusCollection {
    pub name: String,
    pub short_name: Option<String>,
    pub extensions: Vec<String>,
    pub files: Vec<String>,
    pub ignore_files: Vec<String>,
    pub launch_command: Option<String>,
    pub workdir: Option<String>,
}

/// Pegasus game entry with full media asset support
/// See: https://pegasus-frontend.org/docs/user-guide/meta-assets/
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PegasusGame {
    pub name: String,
    pub file: Option<String>,
    pub files: Vec<String>,
    pub developer: Option<String>,
    pub publisher: Option<String>,
    pub genre: Option<String>,
    pub players: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub release: Option<String>,
    pub rating: Option<String>,
    pub sort_title: Option<String>,
    // Media assets
    pub box_front: Option<String>,
    pub box_back: Option<String>,
    pub box_spine: Option<String>,
    pub box_full: Option<String>,
    pub cartridge: Option<String>,
    pub logo: Option<String>,
    pub marquee: Option<String>,
    pub bezel: Option<String>,
    pub gridicon: Option<String>,
    pub flyer: Option<String>,
    pub background: Option<String>,
    pub music: Option<String>,
    pub screenshot: Option<String>,
    pub titlescreen: Option<String>,
    pub video: Option<String>,
    #[serde(skip)] // extra 字段单独处理
    pub extra: HashMap<String, String>,
}


/// Parse result
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PegasusMetadata {
    pub collections: Vec<PegasusCollection>,
    pub games: Vec<PegasusGame>,
}

/// Detect encoding and decode bytes to string
fn decode_bytes_to_string(bytes: &[u8]) -> String {
    // Try UTF-8 first (with BOM check)
    if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        // UTF-8 with BOM
        if let Ok(s) = std::str::from_utf8(&bytes[3..]) {
            return s.to_string();
        }
    }
    
    // Try UTF-8 without BOM
    if let Ok(s) = std::str::from_utf8(bytes) {
        // Check if it looks like valid text (no replacement chars after re-encoding)
        if !s.contains('\u{FFFD}') {
            return s.to_string();
        }
    }
    
    // Use chardetng for encoding detection
    let mut detector = EncodingDetector::new();
    detector.feed(bytes, true);
    let encoding = detector.guess(None, true);
    
    // Decode with detected encoding
    let (decoded, _, had_errors) = encoding.decode(bytes);
    if !had_errors {
        return decoded.into_owned();
    }
    
    // Fallback: try GBK (common for Chinese files)
    let (decoded, _, _) = GBK.decode(bytes);
    decoded.into_owned()
}

/// Parse a Pegasus metadata file
pub fn parse_pegasus_file(path: &Path) -> Result<PegasusMetadata, String> {
    let bytes = std::fs::read(path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    
    let content = decode_bytes_to_string(&bytes);
    parse_pegasus_content(&content)
}

/// Parse Pegasus metadata from string content
pub fn parse_pegasus_content(content: &str) -> Result<PegasusMetadata, String> {
    let mut result = PegasusMetadata::default();
    let mut current_collection: Option<PegasusCollection> = None;
    let mut current_game: Option<PegasusGame> = None;
    let mut current_key: Option<String> = None;
    let mut current_value = String::new();
    
    for line in content.lines() {
        // Skip comments and empty lines
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        
        // Check if line is continuation (starts with space/tab)
        if line.starts_with(' ') || line.starts_with('\t') {
            // Append to current value
            let trimmed = line.trim();
            if trimmed == "." {
                current_value.push_str("\n\n");
            } else {
                if !current_value.is_empty() {
                    current_value.push(' ');
                }
                current_value.push_str(trimmed);
            }
            continue;
        }
        
        // Process previous key-value if exists
        if let Some(key) = current_key.take() {
            apply_key_value(&key, &current_value, &mut current_collection, &mut current_game);
            current_value.clear();
        }
        
        // Parse new key: value
        if let Some(colon_pos) = line.find(':') {
            let key = line[..colon_pos].trim().to_lowercase();
            let value = line[colon_pos + 1..].trim().to_string();
            
            // Check for special keys that start new entries
            match key.as_str() {
                "collection" => {
                    // Save previous game if exists
                    if let Some(game) = current_game.take() {
                        result.games.push(game);
                    }
                    // Save previous collection if exists
                    if let Some(coll) = current_collection.take() {
                        result.collections.push(coll);
                    }
                    // Start new collection
                    current_collection = Some(PegasusCollection {
                        name: value,
                        ..Default::default()
                    });
                }
                "game" => {
                    // Save previous game if exists
                    if let Some(game) = current_game.take() {
                        result.games.push(game);
                    }
                    // Start new game
                    current_game = Some(PegasusGame {
                        name: value,
                        ..Default::default()
                    });
                }
                _ => {
                    current_key = Some(key);
                    current_value = value;
                }
            }
        }
    }
    
    // Process last key-value
    if let Some(key) = current_key.take() {
        apply_key_value(&key, &current_value, &mut current_collection, &mut current_game);
    }
    
    // Save remaining entries
    if let Some(game) = current_game {
        result.games.push(game);
    }
    if let Some(coll) = current_collection {
        result.collections.push(coll);
    }
    
    Ok(result)
}

fn apply_key_value(
    key: &str,
    value: &str,
    collection: &mut Option<PegasusCollection>,
    game: &mut Option<PegasusGame>,
) {
    if let Some(ref mut g) = game {
        let first_value = || value.split_whitespace().next().map(|v| v.to_string());
        
        match key {
            "file" => g.file = Some(value.to_string()),
            "files" => g.files = value.split_whitespace().map(|s| s.to_string()).collect(),
            "developer" | "developers" => g.developer = Some(value.to_string()),
            "publisher" | "publishers" => g.publisher = Some(value.to_string()),
            "genre" | "genres" => g.genre = Some(value.to_string()),
            "players" => g.players = Some(value.to_string()),
            "summary" => g.summary = Some(value.to_string()),
            "description" => g.description = Some(value.to_string()),
            "release" => g.release = Some(value.to_string()),
            "rating" => g.rating = Some(value.to_string()),
            "sort_title" | "sort_name" | "sort-by" => g.sort_title = Some(value.to_string()),
            
            "assets.boxfront" | "assets.box_front" | "assets.boxart2d" | "boxart" | "cover" => {
                g.box_front = first_value();
            }
            "assets.boxback" | "assets.box_back" => g.box_back = first_value(),
            "assets.boxspine" | "assets.box_spine" => g.box_spine = first_value(),
            "assets.boxfull" | "assets.box_full" => g.box_full = first_value(),
            "assets.cartridge" | "assets.disc" | "assets.cart" => g.cartridge = first_value(),
            "assets.logo" | "assets.wheel" => g.logo = first_value(),
            "assets.marquee" | "assets.banner" => g.marquee = first_value(),
            "assets.bezel" | "assets.screenmarquee" => g.bezel = first_value(),
            "assets.gridicon" | "assets.steam" | "assets.poster" => g.gridicon = first_value(),
            "assets.flyer" => g.flyer = first_value(),
            "assets.background" | "assets.fanart" => g.background = first_value(),
            "assets.music" => g.music = first_value(),
            "assets.screenshot" | "assets.screenshots" => g.screenshot = first_value(),
            "assets.titlescreen" | "assets.title_screen" => g.titlescreen = first_value(),
            "assets.video" | "assets.videos" => g.video = first_value(),
            
            "x-english-name" => {
                g.extra.insert("x-english-name".to_string(), value.to_string());
            }

            _ if key.starts_with("x-") => {
                g.extra.insert(key.to_string(), value.to_string());
            }
            _ => {}
        }

        return;
    }
    
    // Collection properties
    if let Some(ref mut c) = collection {
        match key {
            "shortname" | "short_name" => c.short_name = Some(value.to_string()),
            "extension" | "extensions" => c.extensions = value.split_whitespace().map(|s| s.to_string()).collect(),
            "files" => c.files = value.split_whitespace().map(|s| s.to_string()).collect(),
            "ignore-file" | "ignore-files" => c.ignore_files = value.split_whitespace().map(|s| s.to_string()).collect(),
            "launch" | "command" => c.launch_command = Some(value.to_string()),
            "workdir" | "cwd" => c.workdir = Some(value.to_string()),
            _ => {}
        }
    }
}

/// Export games to Pegasus metadata format
pub fn export_to_pegasus(
    collection_name: &str,
    games: &[PegasusGame],
    extensions: Option<&[&str]>,
) -> String {
    let mut output = String::new();
    
    // Collection header
    output.push_str(&format!("collection: {}\n", collection_name));
    
    if let Some(exts) = extensions {
        output.push_str(&format!("extensions: {}\n", exts.join(" ")));
    }
    
    output.push('\n');
    
    // Games
    for game in games {
        output.push_str(&format!("game: {}\n", game.name));
        
        if let Some(ref file) = game.file {
            output.push_str(&format!("file: {}\n", file));
        }
        
        if !game.files.is_empty() {
            output.push_str(&format!("files: {}\n", game.files.join(" ")));
        }
        
        if let Some(ref dev) = game.developer {
            output.push_str(&format!("developer: {}\n", dev));
        }
        
        if let Some(ref pub_) = game.publisher {
            output.push_str(&format!("publisher: {}\n", pub_));
        }
        
        if let Some(ref genre) = game.genre {
            output.push_str(&format!("genre: {}\n", genre));
        }
        
        if let Some(ref players) = game.players {
            output.push_str(&format!("players: {}\n", players));
        }
        
        if let Some(ref summary) = game.summary {
            output.push_str(&format!("summary: {}\n", summary));
        }
        
        if let Some(ref desc) = game.description {
            // Handle multiline descriptions
            let lines: Vec<&str> = desc.lines().collect();
            if lines.is_empty() {
                output.push_str(&format!("description: {}\n", desc));
            } else {
                output.push_str(&format!("description: {}\n", lines[0]));
                for line in &lines[1..] {
                    if line.is_empty() {
                        output.push_str("  .\n");
                    } else {
                        output.push_str(&format!("  {}\n", line));
                    }
                }
            }
        }
        
        if let Some(ref release) = game.release {
            output.push_str(&format!("release: {}\n", release));
        }
        
        if let Some(ref rating) = game.rating {
            output.push_str(&format!("rating: {}\n", rating));
        }
        
        // Custom fields
        for (k, v) in &game.extra {
            output.push_str(&format!("{}: {}\n", k, v));
        }
        
        output.push('\n');
    }
    
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic() {
        let content = r#"
collection: SNES
extensions: sfc smc

game: Super Mario World
file: Super Mario World.sfc
developer: Nintendo
genre: Platform
players: 2
rating: 95%
"#;
        let result = parse_pegasus_content(content).unwrap();
        assert_eq!(result.collections.len(), 1);
        assert_eq!(result.collections[0].name, "SNES");
        assert_eq!(result.games.len(), 1);
        assert_eq!(result.games[0].name, "Super Mario World");
        assert_eq!(result.games[0].developer, Some("Nintendo".to_string()));
    }
}
