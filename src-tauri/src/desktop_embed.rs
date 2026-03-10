use windows::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowExW, FindWindowW, GetClassNameW,
    GetWindowLongPtrW, SendMessageTimeoutW,
    SetParent, SetWindowLongPtrW, SetWindowPos, ShowWindow,
    GWL_EXSTYLE, GWL_STYLE,
    SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
    SW_SHOWNOACTIVATE, SMTO_NORMAL,
    WS_CHILD, WS_POPUP,
    WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
};
use windows::core::{w, PCWSTR};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

/// 查找桌面层窗口，兼容 Wallpaper Engine 等动态壁纸软件。
///
/// 正常桌面结构: Progman → SHELLDLL_DefView → SysListView32
/// Wallpaper Engine 激活后: Progman 收到 0x052C 消息 → 系统将 SHELLDLL_DefView
/// 移到一个 WorkerW 中，另一个 WorkerW 在 Progman 下方作为壁纸层。
///
/// 策略:
/// 1. 先枚举所有 WorkerW，看谁的子窗口包含 SHELLDLL_DefView（说明 WE 模式激活）
///    → 返回该 WorkerW
/// 2. 如果没找到（普通桌面），向 Progman 发送 0x052C 创建 WorkerW 层，
///    再枚举 WorkerW 找到合适的父窗口
/// 3. 最终回退到 Progman 本身
pub fn find_desktop_parent() -> Option<HWND> {
    let progman = unsafe { FindWindowW(w!("Progman"), PCWSTR::null()) }.ok()?;

    // 第一次尝试：找已有的承载 SHELLDLL_DefView 的 WorkerW
    if let Some(worker) = find_workerw_with_shelldll() {
        return Some(worker);
    }

    // 发送未公开消息 0x052C 让系统分裂出 WorkerW
    // 这与 Wallpaper Engine 使用的是同一个消息
    unsafe {
        let _ = SendMessageTimeoutW(
            progman,
            0x052C,
            WPARAM(0xD),
            LPARAM(0x1),
            SMTO_NORMAL,
            1000,
            None,
        );
    }

    // 第二次尝试：发送消息后再找 WorkerW
    if let Some(worker) = find_workerw_with_shelldll() {
        return Some(worker);
    }

    // 最终回退到 Progman
    Some(progman)
}

/// 枚举所有 WorkerW 窗口，返回包含 SHELLDLL_DefView 子窗口的那个
fn find_workerw_with_shelldll() -> Option<HWND> {
    let mut result: HWND = HWND::default();

    unsafe {
        let _ = EnumWindows(
            Some(enum_worker_callback),
            LPARAM(&raw mut result as isize),
        );
    }

    if result == HWND::default() {
        None
    } else {
        Some(result)
    }
}

/// EnumWindows 回调：检查每个顶层窗口是否是包含 SHELLDLL_DefView 的 WorkerW
unsafe extern "system" fn enum_worker_callback(hwnd: HWND, lparam: LPARAM) -> windows::core::BOOL {
    let mut class_buf = [0u16; 256];
    let len = GetClassNameW(hwnd, &mut class_buf) as usize;
    let class_name = OsString::from_wide(&class_buf[..len]);

    if class_name == "WorkerW" {
        // 检查此 WorkerW 是否包含 SHELLDLL_DefView 子窗口
        if let Ok(shell) = FindWindowExW(Some(hwnd), None, w!("SHELLDLL_DefView"), PCWSTR::null()) {
            if shell != HWND::default() {
                // 找到了！这就是承载桌面图标的 WorkerW
                let result_ptr = lparam.0 as *mut HWND;
                *result_ptr = hwnd;
                return windows::core::BOOL(0); // 停止枚举
            }
        }
    }

    windows::core::BOOL(1) // 继续枚举
}

/// 将指定窗口嵌入桌面层，使其永远停留在桌面上方、其他窗口下方。
pub fn embed_to_desktop(hwnd: HWND) -> bool {
    let parent = match find_desktop_parent() {
        Some(p) => p,
        None => {
            eprintln!("[desktop_embed] 未能找到桌面父窗口");
            return false;
        }
    };

    unsafe {
        // 修改普通窗口样式：去掉 WS_POPUP，加上 WS_CHILD
        let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
        let new_style = (style as u32 & !WS_POPUP.0) | WS_CHILD.0;
        SetWindowLongPtrW(hwnd, GWL_STYLE, new_style as isize);

        // 修改扩展样式：加上 NOACTIVATE + TOOLWINDOW（不在 Alt-Tab 中出现）
        let ex_style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
        let new_ex = ex_style as u32 | WS_EX_NOACTIVATE.0 | WS_EX_TOOLWINDOW.0;
        SetWindowLongPtrW(hwnd, GWL_EXSTYLE, new_ex as isize);

        // 设置父窗口为桌面层
        let prev = SetParent(hwnd, Some(parent));
        if prev.is_err() {
            eprintln!("[desktop_embed] SetParent 失败");
            return false;
        }

        // 确保窗口可见（SetParent + 样式变更后需要显式显示）
        let _ = ShowWindow(hwnd, SW_SHOWNOACTIVATE);

        // 刷新位置，不改变 z-order
        let _ = SetWindowPos(
            hwnd,
            None,
            0, 0, 0, 0,
            SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOZORDER,
        );
    }

    println!("[desktop_embed] 窗口已嵌入桌面层");
    true
}
