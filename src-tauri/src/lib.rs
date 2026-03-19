mod commands;
mod config;
#[cfg(windows)]
mod desktop_embed;
mod models;
mod storage;
mod system;
mod webdav;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            commands::load_data,
            commands::save_data,
            commands::test_webdav,
            commands::save_webdav_config,
            commands::load_webdav_config,
            commands::clear_webdav_config,
            commands::save_webdav_proxy_config,
            commands::load_webdav_proxy_config,
            commands::webdav_upload,
            commands::webdav_download,
            commands::get_autostart,
            commands::set_autostart,
            commands::check_update,
            commands::download_and_install_update,
        ])
        .setup(|app| {
            system::setup_tray(app)?;
            system::embed_main_window(app);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}