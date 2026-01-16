use crate::db::{get_connection, models::System, schema::systems};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemInfo {
    pub id: String,
    pub name: String,
    pub short_name: String,
    pub manufacturer: Option<String>,
    pub release_year: Option<i32>,
    pub extensions: Vec<String>,
}

impl From<System> for SystemInfo {
    fn from(s: System) -> Self {
        Self {
            id: s.id,
            name: s.name,
            short_name: s.short_name,
            manufacturer: s.manufacturer,
            release_year: s.release_year,
            extensions: serde_json::from_str(&s.extensions).unwrap_or_default(),
        }
    }
}

/// 获取所有游戏系统
#[tauri::command]
pub fn get_systems() -> Result<Vec<SystemInfo>, String> {
    let mut conn = get_connection().map_err(|e| e.to_string())?;

    let result = systems::table
        .load::<System>(&mut conn)
        .map_err(|e| e.to_string())?;

    Ok(result.into_iter().map(SystemInfo::from).collect())
}

/// 获取单个游戏系统
#[tauri::command]
pub fn get_system(id: String) -> Result<Option<SystemInfo>, String> {
    let mut conn = get_connection().map_err(|e| e.to_string())?;

    let result = systems::table
        .filter(systems::id.eq(&id))
        .first::<System>(&mut conn)
        .optional()
        .map_err(|e| e.to_string())?;

    Ok(result.map(SystemInfo::from))
}
