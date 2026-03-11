use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct WebDavConfig {
    pub server: String,
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct ProxyConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub port: Option<u16>,
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct AppConfig {
    pub webdav: Option<WebDavConfig>,
    #[serde(default)]
    pub webdav_proxy: ProxyConfig,
}

pub struct WebDavCredentials {
    pub server: String,
    pub username: String,
    pub password: String,
}
