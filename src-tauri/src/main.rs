#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

/// true while the widget is attached to the desktop layer (child of the desktop window)
static PINNED: AtomicBool = AtomicBool::new(false);

/// drag state: (cursor x, cursor y, window x, window y) at mouse-down, all physical pixels
#[derive(Default)]
struct DragState(Mutex<Option<(f64, f64, i32, i32)>>);

fn es<E: ToString>(e: E) -> String {
    e.to_string()
}

// ---------------------------------------------------------------------------------------------
// Windows only: stick the widget to the desktop so "Show desktop" (Win+D) does not hide it.
// The widget becomes a child of the window that hosts the desktop icons, drawn above the icons
// but below every normal application window.
// ---------------------------------------------------------------------------------------------
#[cfg(windows)]
mod desk {
    use std::ptr::{null, null_mut};
    use windows_sys::Win32::Foundation::{HWND, LPARAM, POINT, RECT};
    use windows_sys::Win32::Graphics::Gdi::ScreenToClient;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, FindWindowExW, FindWindowW, GetParent, GetWindowLongPtrW, GetWindowRect,
        SetParent, SetWindowLongPtrW, SetWindowPos, GWL_STYLE, HWND_TOP, SWP_FRAMECHANGED,
        SWP_NOACTIVATE, SWP_NOSIZE, SWP_NOZORDER, SWP_SHOWWINDOW, WS_CHILD, WS_POPUP,
    };

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    unsafe extern "system" fn enum_cb(hwnd: HWND, lparam: LPARAM) -> i32 {
        let cls = wide("SHELLDLL_DefView");
        if !FindWindowExW(hwnd, null_mut(), cls.as_ptr(), null()).is_null() {
            *(lparam as *mut HWND) = hwnd;
            return 0; // stop
        }
        1 // continue
    }

    /// The window that directly contains the desktop-icons view (Progman, or a WorkerW).
    unsafe fn desktop_host() -> HWND {
        let prog_name = wide("Progman");
        let prog = FindWindowW(prog_name.as_ptr(), null());
        if prog.is_null() {
            return null_mut();
        }
        let cls = wide("SHELLDLL_DefView");
        if !FindWindowExW(prog, null_mut(), cls.as_ptr(), null()).is_null() {
            return prog;
        }
        let mut found: HWND = null_mut();
        EnumWindows(Some(enum_cb), &mut found as *mut HWND as LPARAM);
        found
    }

    pub fn pin(h: isize) -> bool {
        unsafe {
            let hwnd = h as HWND;
            let host = desktop_host();
            if host.is_null() {
                return false;
            }
            let mut r: RECT = std::mem::zeroed();
            GetWindowRect(hwnd, &mut r);
            let st = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
            SetWindowLongPtrW(hwnd, GWL_STYLE, ((st & !WS_POPUP) | WS_CHILD) as isize);
            SetParent(hwnd, host);
            if GetParent(hwnd) != host {
                SetWindowLongPtrW(hwnd, GWL_STYLE, st as isize);
                return false;
            }
            let mut p = POINT { x: r.left, y: r.top };
            ScreenToClient(host, &mut p);
            SetWindowPos(
                hwnd,
                HWND_TOP,
                p.x,
                p.y,
                0,
                0,
                SWP_NOSIZE | SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_SHOWWINDOW,
            );
            true
        }
    }

    pub fn unpin(h: isize) {
        unsafe {
            let hwnd = h as HWND;
            let mut r: RECT = std::mem::zeroed();
            GetWindowRect(hwnd, &mut r);
            let st = GetWindowLongPtrW(hwnd, GWL_STYLE) as u32;
            SetParent(hwnd, null_mut());
            SetWindowLongPtrW(hwnd, GWL_STYLE, ((st & !WS_CHILD) | WS_POPUP) as isize);
            SetWindowPos(
                hwnd,
                HWND_TOP,
                r.left,
                r.top,
                0,
                0,
                SWP_NOSIZE | SWP_NOACTIVATE | SWP_FRAMECHANGED | SWP_SHOWWINDOW,
            );
        }
    }

    /// Move a pinned (child) window to SCREEN coordinates x,y.
    pub fn move_child(h: isize, x: i32, y: i32) -> bool {
        unsafe {
            let hwnd = h as HWND;
            let parent = GetParent(hwnd);
            if parent.is_null() {
                return false;
            }
            let mut p = POINT { x, y };
            ScreenToClient(parent, &mut p);
            SetWindowPos(
                hwnd,
                null_mut(),
                p.x,
                p.y,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            ) != 0
        }
    }
}

#[cfg(windows)]
fn raw_hwnd(w: &WebviewWindow) -> Option<isize> {
    w.hwnd().ok().map(|h| h.0 as isize)
}
#[cfg(windows)]
fn os_pin(w: &WebviewWindow) -> bool {
    raw_hwnd(w).map(desk::pin).unwrap_or(false)
}
#[cfg(windows)]
fn os_unpin(w: &WebviewWindow) {
    if let Some(h) = raw_hwnd(w) {
        desk::unpin(h);
    }
}
#[cfg(windows)]
fn os_move_child(w: &WebviewWindow, x: i32, y: i32) -> bool {
    raw_hwnd(w).map(|h| desk::move_child(h, x, y)).unwrap_or(false)
}
#[cfg(not(windows))]
fn os_pin(_w: &WebviewWindow) -> bool {
    false
}
#[cfg(not(windows))]
fn os_unpin(_w: &WebviewWindow) {}
#[cfg(not(windows))]
fn os_move_child(_w: &WebviewWindow, _x: i32, _y: i32) -> bool {
    false
}

// ---------------------------------------------------------------------------------------------
// helpers
// ---------------------------------------------------------------------------------------------
fn move_to(w: &WebviewWindow, x: i32, y: i32) {
    if PINNED.load(Ordering::Relaxed) && os_move_child(w, x, y) {
        return;
    }
    let _ = w.set_position(PhysicalPosition::new(x, y));
}

fn on_screen(w: &WebviewWindow, x: i32, y: i32, pw: i32, ph: i32) -> bool {
    let (cx, cy) = (x + pw / 2, y + ph / 2);
    w.available_monitors()
        .map(|ms| {
            ms.iter().any(|m| {
                let p = m.position();
                let s = m.size();
                cx >= p.x && cx < p.x + s.width as i32 && cy >= p.y && cy < p.y + s.height as i32
            })
        })
        .unwrap_or(true)
}

fn default_pos(w: &WebviewWindow, pw: i32) -> (i32, i32) {
    let sf = w.scale_factor().unwrap_or(1.0);
    let mon = w
        .primary_monitor()
        .ok()
        .flatten()
        .or_else(|| w.current_monitor().ok().flatten());
    match mon {
        Some(m) => {
            let p = m.position();
            let s = m.size();
            (
                p.x + s.width as i32 - pw - (28.0 * sf) as i32,
                p.y + (48.0 * sf) as i32,
            )
        }
        None => (100, 100),
    }
}

// ---------------------------------------------------------------------------------------------
// commands used by the widget window
// ---------------------------------------------------------------------------------------------
#[tauri::command]
fn init_window(
    w: WebviewWindow,
    x: Option<i32>,
    y: Option<i32>,
    width: f64,
    height: f64,
    pin: bool,
) -> Result<(i32, i32), String> {
    let sf = w.scale_factor().map_err(es)?;
    let pw = (width * sf).round() as i32;
    let ph = (height * sf).round() as i32;
    w.set_size(PhysicalSize::new(pw as u32, ph as u32)).map_err(es)?;
    let (px, py) = match (x, y) {
        (Some(x), Some(y)) if on_screen(&w, x, y, pw, ph) => (x, y),
        _ => default_pos(&w, pw),
    };
    w.set_position(PhysicalPosition::new(px, py)).map_err(es)?;
    if pin {
        PINNED.store(os_pin(&w), Ordering::Relaxed);
    }
    w.show().map_err(es)?;
    Ok((px, py))
}

#[tauri::command]
fn force_show(w: WebviewWindow) -> Result<(), String> {
    w.show().map_err(es)
}

#[tauri::command]
fn drag_start(w: WebviewWindow, st: tauri::State<'_, DragState>) -> Result<(), String> {
    let c = w.cursor_position().map_err(es)?;
    let p = w.outer_position().map_err(es)?;
    *st.0.lock().unwrap() = Some((c.x, c.y, p.x, p.y));
    Ok(())
}

#[tauri::command]
fn drag_move(w: WebviewWindow, st: tauri::State<'_, DragState>) -> Result<(), String> {
    let g = *st.0.lock().unwrap();
    if let Some((cx, cy, px, py)) = g {
        let c = w.cursor_position().map_err(es)?;
        move_to(
            &w,
            px + (c.x - cx).round() as i32,
            py + (c.y - cy).round() as i32,
        );
    }
    Ok(())
}

#[tauri::command]
fn drag_end(w: WebviewWindow, st: tauri::State<'_, DragState>) -> Result<(i32, i32), String> {
    *st.0.lock().unwrap() = None;
    let p = w.outer_position().map_err(es)?;
    Ok((p.x, p.y))
}

/// Resize (logical px). `center` keeps the visual centre fixed; otherwise the top-left stays put.
#[tauri::command]
fn set_size(w: WebviewWindow, width: f64, height: f64, center: bool) -> Result<(), String> {
    let sf = w.scale_factor().map_err(es)?;
    let nw = (width * sf).round() as i32;
    let nh = (height * sf).round() as i32;
    let old = w.outer_size().map_err(es)?;
    let pos = w.outer_position().map_err(es)?;
    let (mut nx, mut ny) = (pos.x, pos.y);
    if center {
        nx += (old.width as i32 - nw) / 2;
        ny += (old.height as i32 - nh) / 2;
    }
    // keep the whole widget on its monitor
    if let Ok(Some(m)) = w.current_monitor() {
        let (mp, ms) = (m.position(), m.size());
        nx = nx.min(mp.x + ms.width as i32 - nw).max(mp.x);
        ny = ny.min(mp.y + ms.height as i32 - nh).max(mp.y);
    }
    w.set_size(PhysicalSize::new(nw as u32, nh as u32)).map_err(es)?;
    if nx != pos.x || ny != pos.y {
        move_to(&w, nx, ny);
    }
    Ok(())
}

#[tauri::command]
fn set_pin(w: WebviewWindow, on: bool) -> Result<bool, String> {
    let now = PINNED.load(Ordering::Relaxed);
    if on && !now {
        PINNED.store(os_pin(&w), Ordering::Relaxed);
    } else if !on && now {
        os_unpin(&w);
        PINNED.store(false, Ordering::Relaxed);
    }
    Ok(PINNED.load(Ordering::Relaxed))
}

#[tauri::command]
fn reset_pos(w: WebviewWindow) -> Result<(i32, i32), String> {
    let size = w.outer_size().map_err(es)?;
    let (x, y) = default_pos(&w, size.width as i32);
    move_to(&w, x, y);
    Ok((x, y))
}

#[tauri::command]
fn set_autostart(app: AppHandle, on: bool) -> Result<(), String> {
    let m = app.autolaunch();
    let r = if on { m.enable() } else { m.disable() };
    r.map_err(es)
}

#[tauri::command]
fn quit(app: AppHandle) {
    app.exit(0);
}

// ---------------------------------------------------------------------------------------------
// settings window: created on demand next to the widget, destroyed when closed (no idle RAM)
// ---------------------------------------------------------------------------------------------
fn open_settings_impl(app: &AppHandle, area: Option<(i32, i32, i32, i32)>) -> Result<(), String> {
    if let Some(s) = app.get_webview_window("settings") {
        let _ = s.set_focus();
        return Ok(());
    }
    let main = app.get_webview_window("main").ok_or("no main window")?;
    let sf = main.scale_factor().map_err(es)?;
    let pos = main.outer_position().map_err(es)?;
    let size = main.outer_size().map_err(es)?;
    let (wx, wy, ww, wh) = (pos.x, pos.y, size.width as i32, size.height as i32);

    // usable screen area (without the taskbar); the page reports it, else guess from the monitor
    let guess = match main.current_monitor().ok().flatten() {
        Some(m) => (
            m.position().x,
            m.position().y,
            m.size().width as i32,
            m.size().height as i32 - (48.0 * sf) as i32,
        ),
        None => (0, 0, 1920, 1032),
    };
    let (ax, ay, aw, ah) = match area {
        Some((x, y, w, h))
            if w > 300 && h > 300 && wx + ww / 2 >= x && wx + ww / 2 < x + w && wy + wh / 2 >= y && wy + wh / 2 < y + h =>
        {
            (x, y, w, h)
        }
        _ => guess,
    };

    let gap = (6.0 * sf) as i32;
    let m8 = (8.0 * sf) as i32;
    let pw = (372.0 * sf) as i32;
    let full_h = ((620.0 * sf) as i32).min(ah - 2 * m8);
    let min_h = (380.0 * sf) as i32;
    let clamp_y = |h: i32| wy.max(ay + m8).min(ay + ah - h - m8);
    let clamp_x = |w: i32| (wx + ww / 2 - w / 2).max(ax + m8).min(ax + aw - w - m8);
    let right_x = wx + ww + gap;
    let left_x = wx - pw - gap;
    let below_h = (ay + ah - m8) - (wy + wh + gap);
    let above_h = (wy - gap) - (ay + m8);

    // like a Windows flyout: first side that has room (right, left, below, above)
    let (x, y, ph) = if right_x + pw <= ax + aw - m8 {
        (right_x, clamp_y(full_h), full_h)
    } else if left_x >= ax + m8 {
        (left_x, clamp_y(full_h), full_h)
    } else if below_h >= min_h {
        let h = full_h.min(below_h);
        (clamp_x(pw), wy + wh + gap, h)
    } else if above_h >= min_h {
        let h = full_h.min(above_h);
        (clamp_x(pw), wy - gap - h, h)
    } else {
        (clamp_x(pw), clamp_y(full_h), full_h)
    };

    WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html".into()))
        .title("Dayly settings")
        .inner_size(372.0, ph as f64 / sf)
        .position(x as f64 / sf, y as f64 / sf)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .focused(true)
        .build()
        .map_err(es)?;
    Ok(())
}

// async: creating a window from a sync command can deadlock on Windows
#[tauri::command]
async fn open_settings(
    app: AppHandle,
    ax: Option<i32>,
    ay: Option<i32>,
    aw: Option<i32>,
    ah: Option<i32>,
) -> Result<(), String> {
    let area = match (ax, ay, aw, ah) {
        (Some(a), Some(b), Some(c), Some(d)) => Some((a, b, c, d)),
        _ => None,
    };
    open_settings_impl(&app, area)
}

#[tauri::command]
fn close_settings(app: AppHandle) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.close();
    }
}

fn main() {
    // a few switches that stop WebView2 doing background work we do not need
    std::env::set_var(
        "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
        "--disable-background-networking --disable-component-update --disable-sync --no-pings --disable-breakpad --disable-features=Translate,OptimizationHints",
    );

    tauri::Builder::default()
        .manage(DragState::default())
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .setup(|app| {
            let open = MenuItem::with_id(app, "open", "Settings", true, None::<&str>)?;
            let pin = MenuItem::with_id(app, "pin", "Stick to desktop: on / off", true, None::<&str>)?;
            let reset = MenuItem::with_id(app, "reset", "Reset position", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Quit Dayly", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &pin, &reset, &quit])?;
            let mut tray = TrayIconBuilder::new()
                .tooltip("Dayly")
                .menu(&menu)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "open" => {
                        let _ = app.emit("tray", "open");
                    }
                    "pin" => {
                        let _ = app.emit("tray", "pin");
                    }
                    "reset" => {
                        let _ = app.emit("tray", "reset");
                    }
                    "quit" => app.exit(0),
                    _ => {}
                });
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray.build(app)?;
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            init_window,
            force_show,
            drag_start,
            drag_move,
            drag_end,
            set_size,
            set_pin,
            reset_pos,
            set_autostart,
            quit,
            open_settings,
            close_settings
        ])
        .run(tauri::generate_context!())
        .expect("error while running Dayly");
}
