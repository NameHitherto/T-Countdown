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

fn default_privacy_long_press_duration() -> u64 {
    500
}

fn default_privacy_mask_mode() -> String {
    "blur".to_string()
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PrivacySettingsConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_privacy_long_press_duration")]
    pub long_press_duration: u64,
    #[serde(default = "default_privacy_mask_mode")]
    pub mask_mode: String,
    #[serde(default)]
    pub mask_image: String,
}

impl Default for PrivacySettingsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            long_press_duration: default_privacy_long_press_duration(),
            mask_mode: default_privacy_mask_mode(),
            mask_image: String::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Default)]
pub struct AppConfig {
    pub webdav: Option<WebDavConfig>,
    #[serde(default)]
    pub webdav_proxy: ProxyConfig,
    #[serde(default)]
    pub privacy: PrivacySettingsConfig,
}

pub struct WebDavCredentials {
    pub server: String,
    pub username: String,
    pub password: String,
}
