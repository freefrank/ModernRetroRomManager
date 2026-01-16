use crate::db::{get_connection, models::ApiConfig, schema::api_configs};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiConfigInfo {
    pub id: String,
    pub provider: String,
    pub api_key: Option<String>,
    pub client_id: Option<String>, // Mapping 'username' to client_id for OAuth providers
    pub client_secret: Option<String>, // Mapping 'api_secret'
    pub enabled: bool,
    pub priority: i32,
}

impl From<ApiConfig> for ApiConfigInfo {
    fn from(c: ApiConfig) -> Self {
        Self {
            id: c.id,
            provider: c.provider,
            api_key: c.api_key,
            client_id: c.username, // Reuse username field for client_id
            client_secret: c.api_secret,
            enabled: c.enabled,
            priority: c.priority,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateApiConfig {
    pub provider: String,
    pub api_key: Option<String>,
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub enabled: Option<bool>,
}

// ... inside save_api_config ...

        // Update
        diesel::update(api_configs::table.filter(api_configs::id.eq(existing_config.id)))
            .set((
                api_configs::api_key.eq(config.api_key),
                // Map client_id OR username to username column
                api_configs::username.eq(config.client_id.or(config.username)),
                api_configs::api_secret.eq(config.client_secret),
                api_configs::password.eq(config.password),
                api_configs::enabled.eq(config.enabled.unwrap_or(true)),
            ))
            
// ... inside Insert ...
            username: config.client_id.or(config.username),
            password: config.password,
