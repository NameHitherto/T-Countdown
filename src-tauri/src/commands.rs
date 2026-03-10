use crate::{config, storage, system, webdav};

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
