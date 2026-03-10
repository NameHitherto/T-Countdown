pub fn get_autostart() -> Result<bool, String> {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        match hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run") {
            Ok(key) => Ok(key.get_value::<String, _>("T-Countdown").is_ok()),
            Err(_) => Ok(false),
        }
    }
    #[cfg(not(windows))]
    {
        Ok(false)
    }
}

pub fn set_autostart(enable: bool) -> Result<(), String> {
    #[cfg(windows)]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (key, _) = hkcu
            .create_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Run")
            .map_err(|e| e.to_string())?;

        if enable {
            let exe = std::env::current_exe().map_err(|e| e.to_string())?;
            key.set_value("T-Countdown", &exe.to_string_lossy().to_string())
                .map_err(|e| e.to_string())
        } else {
            match key.delete_value("T-Countdown") {
                Ok(_) => Ok(()),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(e) => Err(format!("删除启动项失败: {}", e)),
            }
        }
    }
    #[cfg(not(windows))]
    {
        if enable {
            Err("仅支持 Windows 系统".to_string())
        } else {
            Ok(())
        }
    }
}

pub fn setup_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::image::Image;
    use tauri::menu::{Menu, MenuItem};
    use tauri::tray::TrayIconBuilder;
    use tauri::Manager;

    let toggle_item = MenuItem::with_id(app, "toggle", "隐藏软件", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "关闭软件", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&toggle_item, &quit_item])?;
    let icon = Image::from_bytes(include_bytes!("../icons/countdown.png"))?;

    TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .tooltip("T-Countdown")
        .on_menu_event(move |app, event| {
            if event.id == "quit" {
                app.exit(0);
            } else if event.id == "toggle" {
                if let Some(win) = app.webview_windows().get("main") {
                    if win.is_visible().unwrap_or(true) {
                        let _ = win.hide();
                        if let Some(item) = menu.get("toggle") {
                            if let Some(menu_item) = item.as_menuitem() {
                                let _ = menu_item.set_text("显示软件");
                            }
                        }
                    } else {
                        let _ = win.show();
                        let _ = win.set_focus();
                        if let Some(item) = menu.get("toggle") {
                            if let Some(menu_item) = item.as_menuitem() {
                                let _ = menu_item.set_text("隐藏软件");
                            }
                        }
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

pub fn embed_main_window(app: &tauri::App) {
    #[cfg(windows)]
    {
        use tauri::Manager;

        if let Some(win) = app.webview_windows().get("main") {
            if let Ok(hwnd) = win.hwnd() {
                crate::desktop_embed::embed_to_desktop(hwnd);
            }
        }
    }
}
