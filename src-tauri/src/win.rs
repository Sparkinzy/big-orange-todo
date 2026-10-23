use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, LogicalSize, Manager, PhysicalPosition, WebviewWindow};

pub const PANEL_W: f64 = 340.0;
pub const PANEL_H: f64 = 520.0;
pub const BALL: f64 = 56.0;
/// 收起态：趴在窗口边缘的小猫，窗口尺寸按素材比例（592:721）
pub const PEEK_W: f64 = 96.0;
pub const PEEK_H: f64 = 117.0;
/// 拖到离屏幕边这么近，松手就贴边收起（QQ 那种灵敏度）
const SNAP_PX: f64 = 26.0;

type Screen = (i32, i32, i32, i32, f64);

/// 收起态探头的悬停判定、离开判定都靠这个线程，不用 WebView 的 mouseenter/mouseleave
#[cfg(windows)]
const POLL_MS: u64 = 110;
/// 光标离开面板多久算「真的走了」
#[cfg(windows)]
const LEAVE_MS: u64 = 420;

#[cfg(windows)]
mod sys {
    use std::ffi::c_void;

    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    pub struct RECT {
        pub left: i32,
        pub top: i32,
        pub right: i32,
        pub bottom: i32,
    }

    #[repr(C)]
    #[derive(Default)]
    pub struct MONITORINFO {
        pub cb_size: u32,
        pub rc_monitor: RECT,
        pub rc_work: RECT,
        pub dw_flags: u32,
    }

    #[repr(C)]
    #[derive(Default, Clone, Copy)]
    pub struct POINT {
        pub x: i32,
        pub y: i32,
    }

    #[link(name = "user32")]
    extern "system" {
        fn MonitorFromWindow(hwnd: *mut c_void, flags: u32) -> *mut c_void;
        fn GetMonitorInfoW(hmon: *mut c_void, info: *mut MONITORINFO) -> i32;
        fn GetCursorPos(p: *mut POINT) -> i32;
        fn GetWindowRect(hwnd: *mut c_void, r: *mut RECT) -> i32;
        fn GetAsyncKeyState(vk: i32) -> i16;
    }

    /// 显示器工作区（已排除任务栏），物理像素
    pub fn work_area(hwnd: *mut c_void) -> Option<(i32, i32, i32, i32)> {
        unsafe {
            let m = MonitorFromWindow(hwnd, 2);
            if m.is_null() {
                return None;
            }
            let mut mi = MONITORINFO {
                cb_size: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            if GetMonitorInfoW(m, &mut mi) == 0 {
                return None;
            }
            let r = mi.rc_work;
            Some((r.left, r.top, r.right - r.left, r.bottom - r.top))
        }
    }

    pub fn cursor() -> (i32, i32) {
        let mut p = POINT::default();
        unsafe { GetCursorPos(&mut p) };
        (p.x, p.y)
    }

    /// 窗口矩形（物理像素）：x, y, 宽, 高
    pub fn window_rect(hwnd: *mut c_void) -> Option<(i32, i32, i32, i32)> {
        let mut r = RECT::default();
        if unsafe { GetWindowRect(hwnd, &mut r) } == 0 {
            return None;
        }
        Some((r.left, r.top, r.right - r.left, r.bottom - r.top))
    }

    pub fn lbutton_down() -> bool {
        unsafe { (GetAsyncKeyState(0x01) as u16 & 0x8000) != 0 }
    }
}

pub struct WinState {
    pub side: Mutex<String>,
    pub mode: Mutex<String>,
    pub screen: Mutex<Screen>,
    /// 程序最后一次设置的目标位置，用来区分「程序移动」和「用户拖动」
    pub expect: Mutex<Option<(i32, i32)>>,
    pub collapsed: AtomicBool,
    /// 光标离开面板的起始时刻，0 = 在面板内
    pub away: AtomicU64,
    /// 打开面板后光标是否进过面板，进过才允许自动收起
    pub armed: AtomicBool,
    pub cooldown: AtomicU64,
}

impl Default for WinState {
    fn default() -> Self {
        Self {
            side: Mutex::new("right".to_string()),
            mode: Mutex::new("ball".to_string()),
            screen: Mutex::new((0, 0, 1920, 1080, 1.0)),
            expect: Mutex::new(None),
            collapsed: AtomicBool::new(false),
            away: AtomicU64::new(0),
            armed: AtomicBool::new(false),
            cooldown: AtomicU64::new(0),
        }
    }
}

fn place(w: &WebviewWindow, st: &WinState, x: i32, y: i32) {
    *st.expect.lock().unwrap() = Some((x, y));
    let _ = w.set_position(PhysicalPosition::new(x, y));
}

fn ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn hold(app: &AppHandle, dur: u64) {
    app.state::<WinState>()
        .cooldown
        .store(ms() + dur, Ordering::SeqCst);
}

/// 窗口所在显示器的工作区：x, y, 宽, 高, 缩放（工作区已排除任务栏）
fn screen_of(w: &WebviewWindow) -> Option<Screen> {
    let m = w.current_monitor().ok().flatten();
    let scale = m.as_ref().map(|m| m.scale_factor()).unwrap_or(1.0);

    #[cfg(windows)]
    if let Ok(h) = w.hwnd() {
        if let Some((x, y, ww, wh)) = sys::work_area(h.0) {
            return Some((x, y, ww, wh, scale));
        }
    }

    let m = m?;
    let p = m.position();
    let s = m.size();
    Some((p.x, p.y, s.width as i32, s.height as i32, scale))
}

pub fn work_area_of(w: &WebviewWindow) -> Option<(i32, i32, i32, i32, f64)> {
    screen_of(w)
}

pub fn current_mode(app: &AppHandle) -> String {
    app.state::<WinState>().mode.lock().unwrap().clone()
}

pub fn collapse(app: &AppHandle) {
    let Some(w) = app.get_webview_window("main") else { return };
    let st = app.state::<WinState>();
    let (mx, my, mw, mh, scale) = *st.screen.lock().unwrap();
    let (Ok(pos), Ok(sz)) = (w.outer_position(), w.outer_size()) else { return };

    let left_gap = pos.x - mx;
    let right_gap = mx + mw - (pos.x + sz.width as i32);
    let side = if left_gap <= right_gap { "left" } else { "right" };
    let peek_w = (PEEK_W * scale).round() as i32;
    let peek_h = (PEEK_H * scale).round() as i32;
    let panel_h = (PANEL_H * scale) as i32;

    st.collapsed.store(true, Ordering::SeqCst);
    st.armed.store(false, Ordering::SeqCst);
    st.away.store(0, Ordering::SeqCst);
    *st.side.lock().unwrap() = side.to_string();
    hold(app, 600);

    // 收起时窗口换成角色尺寸，角色垂直居中于原面板区域
    let x = if side == "left" { mx } else { mx + mw - peek_w };
    let y = (pos.y + (panel_h - peek_h) / 2).clamp(my, (my + mh - peek_h).max(my));
    let _ = w.set_size(LogicalSize::new(PEEK_W, PEEK_H));
    place(&w, &*st, x, y);
    let _ = app.emit("snap", serde_json::json!({ "side": side }));
}

pub fn expand(app: &AppHandle) {
    let Some(w) = app.get_webview_window("main") else { return };
    let st = app.state::<WinState>();
    let (mx, my, mw, mh, scale) = *st.screen.lock().unwrap();
    let Ok(pos) = w.outer_position() else { return };

    let side = st.side.lock().unwrap().clone();
    let pw = (PANEL_W * scale) as i32;
    let ph = (PANEL_H * scale) as i32;
    let peek_h = (PEEK_H * scale) as i32;
    let x = if side == "left" { mx } else { mx + mw - pw };
    let y = (pos.y - (ph - peek_h) / 2).clamp(my, (my + mh - ph).max(my));

    st.collapsed.store(false, Ordering::SeqCst);
    // 点悬浮球 / 碰探头也是展开面板，mode 必须跟着改，否则光标轮询会以为还在球形
    *st.mode.lock().unwrap() = "panel".to_string();
    hold(app, 600);
    let _ = w.set_size(LogicalSize::new(PANEL_W, PANEL_H));
    place(&w, &*st, x, y);
}

pub fn set_mode(app: &AppHandle, mode: &str) {
    let Some(w) = app.get_webview_window("main") else { return };
    let st = app.state::<WinState>();

    // 用已锁定的屏幕参数，避免窗口跨屏后 current_monitor 跳到另一块屏（混合 DPI 会算错尺寸）
    let (mx, my, mw, mh, scale) = *st.screen.lock().unwrap();

    st.collapsed.store(false, Ordering::SeqCst);
    *st.mode.lock().unwrap() = mode.to_string();
    hold(app, 700);

    let pos = w.outer_position().unwrap_or(PhysicalPosition::new(mx, my));

    if mode == "ball" {
        let _ = w.set_size(LogicalSize::new(BALL, BALL));
        let span = (BALL * scale) as i32 + 8;
        let x = pos.x.clamp(mx + 8, (mx + mw - span).max(mx + 8));
        let y = pos.y.clamp(my + 8, (my + mh - span).max(my + 8));
        place(&w, &*st, x, y);
    } else {
        let _ = w.set_size(LogicalSize::new(PANEL_W, PANEL_H));
        let pw = (PANEL_W * scale) as i32;
        let ph = (PANEL_H * scale) as i32;
        let y = pos.y.clamp(my, (my + mh - ph).max(my));
        let left_gap = pos.x - mx;
        let x = if left_gap < 240 {
            mx
        } else {
            mx + mw - pw
        };
        place(&w, &*st, x, y);
        let side = if x == mx { "left" } else { "right" };
        *st.side.lock().unwrap() = side.to_string();
        let _ = app.emit("snap", serde_json::json!({ "side": side }));
    }

    let _ = w.show();
    let _ = w.set_focus();
}

/// 光标轮询：离开面板自动收起、碰到探头自动展开。
/// WebView 的 mouseleave 会被窗口 resize 误触发，所以判定一律走真实坐标。
#[cfg(windows)]
pub fn spawn_watch(app: AppHandle) {
    std::thread::spawn(move || {
        let mut drag = Drag::default();
        loop {
            std::thread::sleep(Duration::from_millis(POLL_MS));
            watch(&app, &mut drag);
        }
    });
}

/// 拖动判定：按住左键把面板拖到屏幕边，松手才收（QQ 面板那套）。
/// 不用 WindowEvent::Moved —— 模态拖动期间回调会被压到最后，拿到时左键已经松开。
#[cfg(windows)]
#[derive(Default)]
struct Drag {
    down: bool,
    from: (i32, i32),
    near: bool,
}

#[cfg(windows)]
fn watch(app: &AppHandle, drag: &mut Drag) {
    let Some(w) = app.get_webview_window("main") else { return };
    let st = app.state::<WinState>();

    if st.mode.lock().unwrap().as_str() != "panel" {
        st.away.store(0, Ordering::SeqCst);
        st.armed.store(false, Ordering::SeqCst);
        return;
    }

    let Ok(h) = w.hwnd() else { return };
    let Some((rx, ry, rw, rh)) = sys::window_rect(h.0) else { return };
    let (cx, cy) = sys::cursor();
    let inside = cx >= rx && cx < rx + rw && cy >= ry && cy < ry + rh;
    let now = ms();
    let cool = now >= st.cooldown.load(Ordering::SeqCst);
    let down = sys::lbutton_down();

    // 拖到边上松手 → 贴边收起
    let (mx, _my, mw, _mh, scale) = *st.screen.lock().unwrap();
    let snap = (SNAP_PX * scale).round() as i32;
    if down && !drag.down {
        drag.down = true;
        drag.from = (rx, ry);
        drag.near = false;
    }
    if down {
        let left_gap = rx - mx;
        let right_gap = mx + mw - (rx + rw);
        drag.near = !st.collapsed.load(Ordering::SeqCst)
            && (left_gap.abs() <= snap || right_gap.abs() <= snap);
    } else if drag.down {
        drag.down = false;
        let moved = (rx, ry) != drag.from;
        let want = drag.near;
        drag.near = false;
        if want && moved && cool {
            hold(app, 500);
            let _ = app.emit("cmd", "collapse");
            return;
        }
    }

    if st.collapsed.load(Ordering::SeqCst) {
        st.away.store(0, Ordering::SeqCst);
        if !inside {
            // 探头被「离开」过，才允许悬停展开；拖动贴边收起时就不至于原地又弹回来
            st.armed.store(true, Ordering::SeqCst);
        } else if st.armed.load(Ordering::SeqCst) && cool {
            hold(app, 700);
            let _ = app.emit("cmd", "expand");
        }
        return;
    }

    // 在面板内，或者正按着左键（拖动/点选）——都算没走
    if inside || sys::lbutton_down() {
        st.away.store(0, Ordering::SeqCst);
        st.armed.store(true, Ordering::SeqCst);
        return;
    }

    // 打开面板后光标还没进来过（Alt+Q、托盘、切回面板），先不收，免得刚弹出就消失
    if !st.armed.load(Ordering::SeqCst) {
        return;
    }

    let since = st.away.load(Ordering::SeqCst);
    if since == 0 {
        st.away.store(now, Ordering::SeqCst);
        return;
    }
    if now - since >= LEAVE_MS && cool {
        st.away.store(0, Ordering::SeqCst);
        hold(app, 400);
        let _ = app.emit("cmd", "collapse");
    }
}
