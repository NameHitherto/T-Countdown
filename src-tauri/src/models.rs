use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct WebDavConfig {
    pub server: String,
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct AppConfig {
    pub webdav: Option<WebDavConfig>,
}

pub struct WebDavCredentials {
    pub server: String,
    pub username: String,
    pub password: String,
}
