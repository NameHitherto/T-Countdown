use std::fs;
use std::path::PathBuf;
use std::sync::OnceLock;
use base64::{Engine as _, engine::general_purpose};
use serde::{Deserialize, Serialize};

// ========== 常量 ==========

const ENCRYPT_KEY: &[u8] = b"t-countdown-2024-encrypt-key!@#$";

// ========== 全局 AppHandle ==========

static APP_HANDLE: OnceLock<tauri::AppHandle> = OnceLock::new();

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

// ========== Windows 桌面图标可见性监听 ==========
//
// 使用 windows crate (windows-rs) 实现，采用双重机制：
//
//   1. SetWinEventHook 监听 EVENT_OBJECT_SHOW / EVENT_OBJECT_HIDE，
//      收到事件后通过 SHGetSetSettings 查询 Shell 的 fHideIcons 标志位。
//
//   2. 兜底定时器每 500ms 通过 SHGetSetSettings 轮询，确保不遗漏。
//
// 关键修复：通过 run_on_main_thread 确保 Tauri 窗口操作在主线程执行。

#[cfg(windows)]
mod desktop_icon_monitor {
    use std::sync::atomic::{AtomicBool, Ordering};

    /// 桌面图标是否处于隐藏状态（用于去重，避免重复操作）
    static ICONS_HIDDEN: AtomicBool = AtomicBool::new(false);

    // ---- SHGetSetSettings FFI（权威状态源）----

    #[link(name = "shell32")]
    extern "system" {
        fn SHGetSetSettings(lpss: *mut u8, dwMask: u32, bSet: i32);
    }
    const SSF_HIDEICONS: u32 = 0x00004000;

    // Win32 事件常量（内联定义，避免 crate 模块路径差异）
    const EVENT_OBJECT_SHOW: u32 = 0x8002;
    const EVENT_OBJECT_HIDE: u32 = 0x8003;
    const WINEVENT_OUTOFCONTEXT: u32 = 0x0000;
    const WINEVENT_SKIPOWNPROCESS: u32 = 0x0002;

    /// 通过 Shell API 权威检测桌面图标是否被隐藏。
    /// SHELLSTATE 第一个 DWORD 是 bitfield，fHideIcons 位于 bit 12。
    fn are_desktop_icons_hidden() -> bool {
        unsafe {
            let mut buf = [0u8; 64];
            SHGetSetSettings(buf.as_mut_ptr(), SSF_HIDEICONS, 0);
            let flags = u32::from_ne_bytes([buf[0], buf[1], buf[2], buf[3]]);
            (flags >> 12) & 1 != 0
        }
    }

    /// 同步 Tauri 挂件的显隐状态。
    /// 【关键】必须通过 run_on_main_thread 分发到主线程执行窗口操作。
    fn sync_widget(hidden: bool) {
        if let Some(handle) = crate::APP_HANDLE.get() {
            let handle = handle.clone();
            let handle2 = handle.clone();
            let _ = handle.run_on_main_thread(move || {
                use tauri::Manager;
                if let Some(win) = handle2.webview_windows().get("main") {
                    let result = if hidden { win.hide() } else { win.show() };
                    match result {
                        Ok(_) => eprintln!(
                            "[desktop-monitor] 窗口{}成功",
                            if hidden { "隐藏" } else { "显示" }
                        ),
                        Err(e) => eprintln!(
                            "[desktop-monitor] 窗口{}失败: {}",
                            if hidden { "隐藏" } else { "显示" },
                            e
                        ),
                    }
                } else {
                    eprintln!("[desktop-monitor] 未找到 main 窗口");
                }
            });
        } else {
            eprintln!("[desktop-monitor] APP_HANDLE 未初始化");
        }
    }

    /// 检查桌面图标实际状态，仅在状态变化时同步挂件显隐
    fn check_and_sync() {
        let hidden = are_desktop_icons_hidden();
        let prev = ICONS_HIDDEN.swap(hidden, Ordering::SeqCst);
        if hidden != prev {
            eprintln!(
                "[desktop-monitor] 状态变化: {} -> {}",
                if prev { "隐藏" } else { "显示" },
                if hidden { "隐藏" } else { "显示" }
            );
            sync_widget(hidden);
        }
    }

    // ---- WinEvent 回调 ----

    /// `SetWinEventHook` 回调：收到 SHOW/HIDE 窗口事件时检查桌面图标状态。
    unsafe extern "system" fn win_event_callback(
        _hook: windows::Win32::UI::Accessibility::HWINEVENTHOOK,
        _event: u32,
        _hwnd: windows::Win32::Foundation::HWND,
        id_object: i32,
        _id_child: i32,
        _id_event_thread: u32,
        _dw_ms_event_time: u32,
    ) {
        // 只关心窗口级别的事件 (OBJID_WINDOW = 0)
        if id_object != 0 {
            return;
        }
        check_and_sync();
    }

    // ---- 定时器回调（兜底轮询）----

    /// 每 500ms 由消息循环触发，通过 SHGetSetSettings 检查桌面图标状态。
    unsafe extern "system" fn timer_proc(
        _hwnd: windows::Win32::Foundation::HWND,
        _msg: u32,
        _id: usize,
        _time: u32,
    ) {
        check_and_sync();
    }

    // ---- 启动监听 ----

    /// 在独立线程中启动桌面图标可见性监听。
    pub fn start() {
        std::thread::spawn(|| {
            // 等待 Explorer Shell 完成桌面初始化
            std::thread::sleep(std::time::Duration::from_secs(2));

            // 初始状态检查
            let hidden = are_desktop_icons_hidden();
            ICONS_HIDDEN.store(hidden, Ordering::SeqCst);
            eprintln!("[desktop-monitor] 初始状态: icons_hidden={}", hidden);
            if hidden {
                sync_widget(true);
            }

            unsafe {
                use windows::Win32::UI::Accessibility::{SetWinEventHook, UnhookWinEvent};
                use windows::Win32::UI::WindowsAndMessaging::{
                    SetTimer, KillTimer, GetMessageW, TranslateMessage, DispatchMessageW, MSG,
                };

                // 注册 WinEvent 钩子，监听 SHOW / HIDE 事件（即时响应通道）
                let hook = SetWinEventHook(
                    EVENT_OBJECT_SHOW,
                    EVENT_OBJECT_HIDE,
                    None,                        // hmodWinEventProc (None = out-of-context)
                    Some(win_event_callback),
                    0,                           // idProcess  (0 = 所有进程)
                    0,                           // idThread   (0 = 所有线程)
                    WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
                );
                eprintln!("[desktop-monitor] SetWinEventHook: hook={:?}", hook);

                // 兜底定时器：每 500ms 轮询一次 SHGetSetSettings
                let timer_id = SetTimer(None, 1, 500, Some(timer_proc));
                eprintln!("[desktop-monitor] SetTimer: timer_id={}", timer_id);

                // 消息循环 —— 同时驱动 WinEvent 回调和定时器
                let mut msg = MSG::default();
                while GetMessageW(&mut msg, None, 0, 0).0 > 0 {
                    let _ = TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                // 收到 WM_QUIT 后清理
                let _ = UnhookWinEvent(hook);
                if timer_id != 0 {
                    let _ = KillTimer(None, timer_id);
                }
            }
        });
    }
}

// ========== 系统托盘 ==========

fn setup_tray(app: &mut tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    use tauri::tray::TrayIconBuilder;
    use tauri::menu::{Menu, MenuItem};
    use tauri::image::Image;

    let quit_item = MenuItem::with_id(app, "quit", "关闭软件", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit_item])?;
    let icon = Image::from_bytes(include_bytes!("../icons/countdown.png"))?;

    TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .tooltip("T-Countdown")
        .on_menu_event(|app, event| {
            if event.id == "quit" {
                app.exit(0);
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
            let _ = APP_HANDLE.set(app.handle().clone());
            // 系统托盘
            setup_tray(app)?;
            // 启动桌面图标可见性监听
            #[cfg(windows)]
            desktop_icon_monitor::start();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
