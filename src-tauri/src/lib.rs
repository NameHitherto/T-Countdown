use std::fs;
use std::path::PathBuf;
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};

// ========== 常量 ==========

const ENCRYPT_KEY: &[u8] = b"t-countdown-2024-encrypt-key!@#$";

// ========== 路径 ==========

fn data_dir() -> Result<PathBuf, String> {
    let docs = dirs::document_dir().ok_or("无法获取 Documents 目录".to_string())?;
    let dir = docs.join("T-Countdown");
    if !dir.exists() {
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    }
    Ok(dir)
}

fn data_file_path() -> Result<PathBuf, String> {
    Ok(data_dir()?.join("data.json"))
}

fn config_file_path() -> Result<PathBuf, String> {
    Ok(data_dir()?.join("config.json"))
}

// ========== 配置结构 ==========

#[derive(Serialize, Deserialize, Clone, Default)]
struct WebDavConfig {
    server: String,
    username: String, // 加密后
    password: String, // 加密后
}

#[derive(Serialize, Deserialize, Clone, Default)]
struct AppConfig {
    webdav: Option<WebDavConfig>,
}

fn read_config() -> AppConfig {
    config_file_path()
        .ok()
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn write_config(config: &AppConfig) -> Result<(), String> {
    let path = config_file_path()?;
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

// ========== 加密 ==========

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

// ========== WebDAV 辅助 ==========

fn make_auth_header(username: &str, password: &str) -> String {
    let creds = format!("{}:{}", username, password);
    format!(
        "Basic {}",
        general_purpose::STANDARD.encode(creds.as_bytes())
    )
}

/// 读取 Windows 系统代理设置
#[cfg(windows)]
fn get_system_proxy() -> Option<String> {
    use winreg::enums::*;
    use winreg::RegKey;
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let settings = hkcu
        .open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings")
        .ok()?;
    let enabled: u32 = settings.get_value("ProxyEnable").ok()?;
    if enabled == 0 {
        return None;
    }
    let server: String = settings.get_value("ProxyServer").ok()?;
    if server.is_empty() {
        return None;
    }
    // ProxyServer 可能是 "host:port" 或 "http=h:p;https=h:p" 格式
    if server.contains('=') {
        for part in server.split(';') {
            let part = part.trim();
            if let Some(addr) = part.strip_prefix("https=") {
                return Some(format!("http://{}", addr));
            }
        }
        for part in server.split(';') {
            let part = part.trim();
            if let Some(addr) = part.strip_prefix("http=") {
                return Some(format!("http://{}", addr));
            }
        }
        None
    } else {
        Some(format!("http://{}", server))
    }
}

fn create_webdav_agent() -> ureq::Agent {
    let mut builder = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(15));

    // 检测代理：优先环境变量，其次系统代理
    let proxy_url = std::env::var("HTTPS_PROXY")
        .or_else(|_| std::env::var("https_proxy"))
        .or_else(|_| std::env::var("HTTP_PROXY"))
        .or_else(|_| std::env::var("http_proxy"))
        .or_else(|_| std::env::var("ALL_PROXY"))
        .or_else(|_| std::env::var("all_proxy"))
        .ok()
        .filter(|s| !s.is_empty());

    #[cfg(windows)]
    let proxy_url = proxy_url.or_else(get_system_proxy);

    if let Some(url) = proxy_url {
        if let Ok(proxy) = ureq::Proxy::new(&url) {
            builder = builder.proxy(proxy);
        }
    }

    builder.build()
}

// ========== Tauri 命令：数据 ==========

#[tauri::command]
fn load_data() -> Result<String, String> {
    let path = data_file_path()?;
    if !path.exists() {
        return Ok("[]".to_string());
    }
    fs::read_to_string(&path).map_err(|e| e.to_string())
}

#[tauri::command]
fn save_data(json: String) -> Result<(), String> {
    let path = data_file_path()?;
    fs::write(&path, json).map_err(|e| e.to_string())
}

// ========== Tauri 命令：WebDAV ==========

#[tauri::command]
async fn test_webdav(server: String, username: String, password: String) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let url = if server.ends_with('/') {
            server
        } else {
            format!("{}/", server)
        };
        let auth = make_auth_header(&username, &password);
        let agent = create_webdav_agent();

        match agent
            .request("PROPFIND", &url)
            .set("Authorization", &auth)
            .set("Depth", "0")
            .call()
        {
            Ok(_) => Ok(()),
            Err(ureq::Error::Status(code, _)) => {
                if code == 401 || code == 403 {
                    Err("认证失败，请检查账号和应用密码".to_string())
                } else {
                    Err(format!("连接失败，状态码: {}", code))
                }
            }
            Err(e) => Err(format!("网络错误: {}", e)),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
fn save_webdav_config(server: String, username: String, password: String) -> Result<(), String> {
    let mut config = read_config();
    config.webdav = Some(WebDavConfig {
        server,
        username: encrypt_str(&username),
        password: encrypt_str(&password),
    });
    write_config(&config)
}

#[tauri::command]
fn load_webdav_config() -> Result<Option<(String, String)>, String> {
    let config = read_config();
    match config.webdav {
        Some(wc) => {
            let username = decrypt_str(&wc.username)?;
            Ok(Some((wc.server, username)))
        }
        None => Ok(None),
    }
}

#[tauri::command]
fn clear_webdav_config() -> Result<(), String> {
    let mut config = read_config();
    config.webdav = None;
    write_config(&config)
}

#[tauri::command]
async fn webdav_upload(json: String) -> Result<(), String> {
    let config = read_config();
    let wc = config.webdav.ok_or("WebDAV 未配置".to_string())?;
    let username = decrypt_str(&wc.username)?;
    let password = decrypt_str(&wc.password)?;

    tokio::task::spawn_blocking(move || {
        let auth = make_auth_header(&username, &password);
        let base_url = if wc.server.ends_with('/') {
            wc.server.clone()
        } else {
            format!("{}/", wc.server)
        };
        let folder_url = format!("{}T-Countdown/", base_url);
        let file_url = format!("{}data.json", folder_url);
        let agent = create_webdav_agent();

        // 创建目录（忽略已存在的错误）
        let _ = agent
            .request("MKCOL", &folder_url)
            .set("Authorization", &auth)
            .call();

        // 上传数据
        agent
            .put(&file_url)
            .set("Authorization", &auth)
            .set("Content-Type", "application/json; charset=utf-8")
            .send_string(&json)
            .map_err(|e| format!("上传失败: {}", e))?;

        Ok(())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn webdav_download() -> Result<String, String> {
    let config = read_config();
    let wc = config.webdav.ok_or("WebDAV 未配置".to_string())?;
    let username = decrypt_str(&wc.username)?;
    let password = decrypt_str(&wc.password)?;

    tokio::task::spawn_blocking(move || {
        let auth = make_auth_header(&username, &password);
        let base_url = if wc.server.ends_with('/') {
            wc.server.clone()
        } else {
            format!("{}/", wc.server)
        };
        let file_url = format!("{}T-Countdown/data.json", base_url);
        let agent = create_webdav_agent();

        match agent.get(&file_url).set("Authorization", &auth).call() {
            Ok(resp) => {
                let body = resp.into_string().map_err(|e| format!("读取失败: {}", e))?;
                // 验证响应是有效的 JSON，防止代理干扰返回非 JSON 内容
                if serde_json::from_str::<serde_json::Value>(&body).is_err() {
                    return Err(
                        "下载的数据格式无效，可能是网络代理干扰，请检查网络连接".to_string(),
                    );
                }
                Ok(body)
            }
            Err(ureq::Error::Status(404, _)) => Ok("[]".to_string()),
            Err(e) => Err(format!("下载失败: {}", e)),
        }
    })
    .await
    .map_err(|e| e.to_string())?
}

// ========== Tauri 命令：开机自启 ==========

#[tauri::command]
fn get_autostart() -> Result<bool, String> {
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

#[tauri::command]
fn set_autostart(enable: bool) -> Result<(), String> {
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

// ========== 系统托盘 ==========

fn setup_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::tray::TrayIconBuilder;
    use tauri::menu::{Menu, MenuItem};
    use tauri::image::Image;
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
                        // 更新菜单文本为“显示软件”
                        if let Some(item) = menu.get("toggle") {
                            if let Some(mi) = item.as_menuitem() {
                                let _ = mi.set_text("显示软件");
                            }
                        }
                    } else {
                        let _ = win.show();
                        let _ = win.set_focus();
                        // 更新菜单文本为“隐藏软件”
                        if let Some(item) = menu.get("toggle") {
                            if let Some(mi) = item.as_menuitem() {
                                let _ = mi.set_text("隐藏软件");
                            }
                        }
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

// ========== 启动 ==========

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            load_data,
            save_data,
            test_webdav,
            save_webdav_config,
            load_webdav_config,
            clear_webdav_config,
            webdav_upload,
            webdav_download,
            get_autostart,
            set_autostart,
        ])
        .setup(|app| {
            // 系统托盘
            setup_tray(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
