use std::fs;

use base64::{engine::general_purpose, Engine as _};

use crate::models::{AppConfig, PrivacySettingsConfig, ProxyConfig, WebDavConfig, WebDavCredentials};
use crate::storage::config_file_path;

const ENCRYPT_KEY: &[u8] = b"t-countdown-2024-encrypt-key!@#$";

fn xor_cipher(data: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, b)| b ^ ENCRYPT_KEY[i % ENCRYPT_KEY.len()])
        .collect()
}

fn encrypt_str(plain: &str) -> String {
    general_purpose::STANDARD.encode(xor_cipher(plain.as_bytes()))
}

fn decrypt_str(encoded: &str) -> Result<String, String> {
    let encrypted = general_purpose::STANDARD
        .decode(encoded)
        .map_err(|e| e.to_string())?;
    let decrypted = xor_cipher(&encrypted);
    String::from_utf8(decrypted).map_err(|e| e.to_string())
}

pub fn read_config() -> AppConfig {
    config_file_path()
        .ok()
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn write_config(config: &AppConfig) -> Result<(), String> {
    let path = config_file_path()?;
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

pub fn save_webdav_config(server: String, username: String, password: String) -> Result<(), String> {
    let mut config = read_config();
    config.webdav = Some(WebDavConfig {
        server,
        username: encrypt_str(&username),
        password: encrypt_str(&password),
    });
    write_config(&config)
}

pub fn load_webdav_config() -> Result<Option<(String, String)>, String> {
    match read_config().webdav {
        Some(config) => {
            let username = decrypt_str(&config.username)?;
            Ok(Some((config.server, username)))
        }
        None => Ok(None),
    }
}

pub fn clear_webdav_config() -> Result<(), String> {
    let mut config = read_config();
    config.webdav = None;
    write_config(&config)
}

pub fn save_webdav_proxy_config(enabled: bool, port: Option<u16>) -> Result<(), String> {
    let mut config = read_config();
    config.webdav_proxy = ProxyConfig {
        enabled,
        port: if enabled { port } else { None },
    };
    write_config(&config)
}

pub fn load_webdav_proxy_config() -> Result<ProxyConfig, String> {
    Ok(read_config().webdav_proxy)
}

pub fn save_privacy_settings(
    enabled: bool,
    long_press_duration: u64,
    mask_mode: String,
    mask_image: String,
) -> Result<(), String> {
    let mut config = read_config();
    let normalized_mode = if mask_mode == "image" {
        "image".to_string()
    } else {
        "blur".to_string()
    };

    config.privacy = PrivacySettingsConfig {
        enabled,
        long_press_duration: long_press_duration.clamp(300, 5000),
        mask_mode: normalized_mode,
        mask_image,
    };

    write_config(&config)
}

pub fn load_privacy_settings() -> Result<PrivacySettingsConfig, String> {
    Ok(read_config().privacy)
}

pub fn load_webdav_credentials() -> Result<WebDavCredentials, String> {
    let config = read_config();
    let webdav = config.webdav.ok_or("WebDAV 未配置".to_string())?;
    Ok(WebDavCredentials {
        server: webdav.server,
        username: decrypt_str(&webdav.username)?,
        password: decrypt_str(&webdav.password)?,
    })
}
