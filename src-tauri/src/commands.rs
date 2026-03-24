use serde::Serialize;
use tauri::{AppHandle, Runtime};
use tauri_plugin_updater::UpdaterExt;

use crate::{config, storage, system, webdav};

#[derive(Serialize)]
pub struct AvailableUpdate {
    pub version: String,
    pub body: Option<String>,
}

fn with_update_proxy<R: Runtime>(
    app: &AppHandle<R>,
    proxy_port: Option<u16>,
) -> Result<tauri_plugin_updater::UpdaterBuilder, String> {
    let builder = app.updater_builder();

    if let Some(port) = proxy_port {
        let proxy = format!("http://127.0.0.1:{}", port)
            .parse::<url::Url>()
            .map_err(|e| e.to_string())?;
        Ok(builder.proxy(proxy))
    } else {
        Ok(builder.no_proxy())
    }
}

#[tauri::command]
pub fn load_data() -> Result<String, String> {
    storage::load_data()
}

#[tauri::command]
pub fn save_data(json: String) -> Result<(), String> {
    storage::save_data(json)
}

#[tauri::command]
pub async fn test_webdav(server: String, username: String, password: String) -> Result<(), String> {
    webdav::test_webdav(server, username, password).await
}

#[tauri::command]
pub fn save_webdav_config(server: String, username: String, password: String) -> Result<(), String> {
    config::save_webdav_config(server, username, password)
}

#[tauri::command]
pub fn load_webdav_config() -> Result<Option<(String, String)>, String> {
    config::load_webdav_config()
}

#[tauri::command]
pub fn clear_webdav_config() -> Result<(), String> {
    config::clear_webdav_config()
}

#[tauri::command]
pub fn save_webdav_proxy_config(enabled: bool, port: Option<u16>) -> Result<(), String> {
    config::save_webdav_proxy_config(enabled, port)
}

#[tauri::command]
pub fn load_webdav_proxy_config() -> Result<crate::models::ProxyConfig, String> {
    config::load_webdav_proxy_config()
}

#[tauri::command]
pub fn save_privacy_settings(
    enabled: bool,
    long_press_duration: u64,
    mask_mode: String,
    mask_image: String,
) -> Result<(), String> {
    config::save_privacy_settings(enabled, long_press_duration, mask_mode, mask_image)
}

#[tauri::command]
pub fn load_privacy_settings() -> Result<crate::models::PrivacySettingsConfig, String> {
    config::load_privacy_settings()
}

#[tauri::command]
pub fn save_sync_settings(auto_sync_interval_seconds: u64) -> Result<(), String> {
    config::save_sync_settings(auto_sync_interval_seconds)
}

#[tauri::command]
pub fn load_sync_settings() -> Result<crate::models::SyncSettingsConfig, String> {
    config::load_sync_settings()
}

#[tauri::command]
pub async fn webdav_upload(json: String) -> Result<(), String> {
    webdav::upload(json).await
}

#[tauri::command]
pub async fn webdav_download() -> Result<String, String> {
    webdav::download().await
}

#[tauri::command]
pub fn get_autostart() -> Result<bool, String> {
    system::get_autostart()
}

#[tauri::command]
pub fn set_autostart(enable: bool) -> Result<(), String> {
    system::set_autostart(enable)
}

#[tauri::command]
pub async fn check_update<R: Runtime>(
    app: AppHandle<R>,
    proxy_port: Option<u16>,
) -> Result<Option<AvailableUpdate>, String> {
    let updater = with_update_proxy(&app, proxy_port)?
        .build()
        .map_err(|e| e.to_string())?;
    let update = updater.check().await.map_err(|e| e.to_string())?;

    Ok(update.map(|update| AvailableUpdate {
        version: update.version,
        body: update.body,
    }))
}

#[tauri::command]
pub async fn download_and_install_update<R: Runtime>(
    app: AppHandle<R>,
    proxy_port: Option<u16>,
) -> Result<(), String> {
    let updater = with_update_proxy(&app, proxy_port)?
        .build()
        .map_err(|e| e.to_string())?;
    let update = updater.check().await.map_err(|e| e.to_string())?;

    let update = update.ok_or("当前没有可安装的更新".to_string())?;
    update
        .download_and_install(|_, _| {}, || {})
        .await
        .map_err(|e| e.to_string())
}
