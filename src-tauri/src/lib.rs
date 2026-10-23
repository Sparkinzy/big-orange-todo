mod db;
mod win;

use db::{Db, Todo};
use std::sync::Mutex;
use tauri::menu::{CheckMenuItem, Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{
    AppHandle, Emitter, Manager, State, WebviewUrl, WebviewWindowBuilder,
};
use tauri_plugin_autostart::ManagerExt as _;
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

#[tauri::command]
fn list_todos(db: State<Db>) -> Vec<Todo> {
    let c = db.0.lock().unwrap();
    db::all(&c)
}

#[tauri::command]
fn get_todo(db: State<Db>, id: i64) -> Option<Todo> {
    let c = db.0.lock().unwrap();
    db::get(&c, id)
}

#[tauri::command]
fn add_todo(app: AppHandle, db: State<Db>, title: String, due: Option<String>) -> Option<Todo> {
    let r = {
        let c = db.0.lock().unwrap();
        db::add(&c, &title, due.as_deref())
    };
    let _ = app.emit("refresh", ());
    r
}

#[tauri::command]
fn toggle_todo(app: AppHandle, db: State<Db>, id: i64, done: bool) {
    {
        let c = db.0.lock().unwrap();
        db::toggle(&c, id, done);
    }
    let _ = app.emit("refresh", ());
}

#[tauri::command]
fn update_todo(app: AppHandle, db: State<Db>, id: i64, title: String, due: Option<String>) {
    {
        let c = db.0.lock().unwrap();
        db::update(&c, id, &title, due.as_deref());
    }
    if let Some(w) = app.get_webview_window(&format!("card-{id}")) {
        let _ = w.set_title(&title);
    }
    let _ = app.emit("refresh", ());
}

#[tauri::command]
fn set_content(app: AppHandle, db: State<Db>, id: i64, content: String) {
    {
        let c = db.0.lock().unwrap();
        db::set_content(&c, id, &content);
    }
    let _ = app.emit("refresh", ());
}

#[tauri::command]
fn set_category(app: AppHandle, db: State<Db>, id: i64, category: Option<String>) {
    {
        let c = db.0.lock().unwrap();
        db::set_category(&c, id, category.as_deref());
    }
    let _ = app.emit("refresh", ());
}

#[tauri::command]
fn delete_todo(app: AppHandle, db: State<Db>, id: i64) {
    {
        let c = db.0.lock().unwrap();
        db::remove(&c, id);
    }
    if let Some(w) = app.get_webview_window(&format!("card-{id}")) {
        let _ = w.close();
    }
    let _ = app.emit("refresh", ());
}

#[tauri::command]
fn set_mode(app: AppHandle, mode: String) {
    win::set_mode(&app, &mode);
}

#[tauri::command]
fn expand(app: AppHandle) {
    win::expand(&app);
}

#[tauri::command]
fn collapse(app: AppHandle) {
    win::collapse(&app);
}

#[tauri::command]
fn hide_window(app: AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
    }
}

#[tauri::command]
fn quit_app(app: AppHandle) {
    app.exit(0);
}

/// 卡片默认出现位置：从面板左侧开始横向排开，排满一行继续往上另起一行
fn card_position(app: &AppHandle, cw: i32, ch: i32) -> (i32, i32) {
    let Some(main) = app.get_webview_window("main") else {
        return (200, 200);
    };
    let (Ok(p), Ok(s)) = (main.outer_position(), main.outer_size()) else {
        return (200, 200);
    };
    let (sx, sy, sw, sh, _) = win::work_area_of(&main).unwrap_or((0, 0, 1920, 1080, 1.0));
    let (mx, my, mw, mh) = (p.x, p.y, s.width as i32, s.height as i32);

    let n = app
        .webview_windows()
        .keys()
        .filter(|k| k.starts_with("card-"))
        .count() as i32;
    let step = cw + 12;

    // 面板左侧一行能放几张
    let cols = (((mx - sx) / step).max(1)).min(8);
    let col = n % cols;
    let row = n / cols;

    let mut x = mx - cw - 12 - step * col;
    if x < sx {
        x = sx + 12;
    }
    let y = (my + (mh - ch) / 2 + row * (ch + 12)).clamp(sy, (sy + sh - ch).max(sy));
    (x, y)
}

fn spawn_card(app: &AppHandle, id: i64, pos: Option<(i32, i32)>) {
    let label = format!("card-{id}");
    if let Some(w) = app.get_webview_window(&label) {
        let _ = w.show();
        let _ = w.set_focus();
        return;
    }

    let win_title = {
        let db = app.state::<Db>();
        let c = db.0.lock().unwrap();
        db::get(&c, id).map(|t| t.title).unwrap_or_else(|| "便利贴".to_string())
    };

    let mut b = WebviewWindowBuilder::new(app, &label, WebviewUrl::default())
        .title(win_title)
        .inner_size(400.0, 500.0)
        .min_inner_size(320.0, 300.0)
        .decorations(false)
        .transparent(true)
        .no_redirection_bitmap(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .shadow(false)
        .resizable(true)
        .visible(false);

    if let Ok(w) = b.build() {
        let _ = w.set_background_color(Some(tauri::window::Color(0, 0, 0, 0)));
        let (x, y) = match pos {
            Some(p) => p,
            None => {
                let scale = w.scale_factor().unwrap_or(1.0);
                card_position(app, (400.0 * scale) as i32, (500.0 * scale) as i32)
            }
        };
        let _ = w.set_position(tauri::PhysicalPosition::new(x, y));
        let _ = w.show();
        let _ = w.set_focus();
    }
}

#[tauri::command]
async fn open_card(app: AppHandle, id: i64) {
    let scale = app
        .get_webview_window("main")
        .and_then(|w| w.scale_factor().ok())
        .unwrap_or(1.0);
    // 库里存的是逻辑坐标，窗口定位要物理坐标
    let pos = {
        let db = app.state::<Db>();
        let c = db.0.lock().unwrap();
        c.query_row(
            "SELECT px, py FROM todos WHERE id = ?1",
            [id],
            |r| Ok((r.get::<_, Option<i32>>(0)?, r.get::<_, Option<i32>>(1)?)),
        )
        .ok()
        .and_then(|(x, y)| match (x, y) {
            (Some(x), Some(y)) => Some(((x as f64 * scale) as i32, (y as f64 * scale) as i32)),
            _ => None,
        })
    };
    spawn_card(&app, id, pos);
}

#[tauri::command]
fn close_card(app: AppHandle, id: i64) {
    if let Some(w) = app.get_webview_window(&format!("card-{id}")) {
        let _ = w.close();
    }
}

#[tauri::command]
fn set_pin(app: AppHandle, id: i64, pinned: bool) {
    let label = format!("card-{id}");
    let mut pos = (0i32, 0i32);
    if let Some(w) = app.get_webview_window(&label) {
        if let Ok(p) = w.outer_position() {
            let s = w.scale_factor().unwrap_or(1.0);
            pos = ((p.x as f64 / s) as i32, (p.y as f64 / s) as i32);
        }
    }
    let db = app.state::<Db>();
    let c = db.0.lock().unwrap();
    db::set_pin(&c, id, pinned, pos.0, pos.1);
}

fn open_panel(app: &AppHandle) {
    win::set_mode(app, "panel");
    let _ = app.emit("cmd", "panel");
}

fn open_ball(app: &AppHandle) {
    win::set_mode(app, "ball");
    let _ = app.emit("cmd", "ball");
}

fn toggle_panel(app: &AppHandle) {
    if win::current_mode(app) == "ball" {
        open_panel(app);
    } else {
        open_ball(app);
    }
}

fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let panel = MenuItem::with_id(app, "panel", "展开面板", true, None::<&str>)?;
    let ball = MenuItem::with_id(app, "ball", "悬浮球模式", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "隐藏到托盘", true, None::<&str>)?;
    let auto_on = app.autolaunch().is_enabled().unwrap_or(false);
    let auto = CheckMenuItem::with_id(app, "auto", "开机自启", true, auto_on, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&panel, &ball, &hide, &auto, &quit])?;

    let mut builder = TrayIconBuilder::with_id("main")
        .tooltip("便利贴")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, e| match e.id.as_ref() {
            "panel" => open_panel(app),
            "ball" => open_ball(app),
            "hide" => {
                if let Some(w) = app.get_webview_window("main") {
                    let _ = w.hide();
                }
            }
            "auto" => {
                let au = app.autolaunch();
                if au.is_enabled().unwrap_or(false) {
                    let _ = au.disable();
                } else {
                    let _ = au.enable();
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, e| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = e
            {
                toggle_panel(tray.app_handle());
            }
        });

    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        // 必须第一个注册：第二个实例直接退出，把已有窗口唤到前台
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            open_panel(app);
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.show();
                let _ = w.set_focus();
            }
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _sc, e| {
                    if e.state() == ShortcutState::Pressed {
                        toggle_panel(app);
                    }
                })
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            list_todos,
            get_todo,
            add_todo,
            toggle_todo,
            update_todo,
            set_content,
            set_category,
            delete_todo,
            set_mode,
            expand,
            collapse,
            hide_window,
            quit_app,
            open_card,
            close_card,
            set_pin
        ])
        .setup(|app| {
            app.manage(Db(Mutex::new(db::open())));
            app.manage(win::WinState::default());

            let handle = app.handle().clone();
            if let Some(w) = app.get_webview_window("main") {
                let _ = w.set_background_color(Some(tauri::window::Color(0, 0, 0, 0)));
                w.on_window_event(move |e| match e {
                    tauri::WindowEvent::CloseRequested { api, .. } => {
                        // 关窗不退出：面板态收成小猫，退出只能走托盘菜单
                        api.prevent_close();
                        let _ = handle.emit("cmd", "collapse");
                    }
                    _ => {}
                });
            }

            // 启动时锁定主屏工作区（已排除任务栏），球落在右下角
            if let Some(w) = app.get_webview_window("main") {
                if let Some((ax, ay, aw, ah, sc)) = win::work_area_of(&w) {
                    *app.state::<win::WinState>().screen.lock().unwrap() = (ax, ay, aw, ah, sc);
                    let _ = w.set_size(tauri::LogicalSize::new(win::BALL, win::BALL));
                    let side_px = (win::BALL * sc) as i32;
                    let pos = (ax + aw - side_px - 40, ay + ah - side_px - 40);
                    *app.state::<win::WinState>().expect.lock().unwrap() = Some(pos);
                    let _ = w.set_position(tauri::PhysicalPosition::new(pos.0, pos.1));
                }
            }

            win::set_mode(app.handle(), "ball");

            // 恢复上次钉在桌面的卡片
            let pinned = {
                let db = app.state::<Db>();
                let c = db.0.lock().unwrap();
                db::pinned_ids(&c)
            };
            let scale = app
                .get_webview_window("main")
                .and_then(|w| w.scale_factor().ok())
                .unwrap_or(1.0);
            for (id, x, y) in pinned {
                let pos = match (x, y) {
                    (Some(x), Some(y)) => {
                        Some(((x as f64 * scale) as i32, (y as f64 * scale) as i32))
                    }
                    _ => None,
                };
                spawn_card(app.handle(), id, pos);
            }

            setup_tray(app.handle())?;
            win::spawn_watch(app.handle().clone());
            let sc = Shortcut::new(Some(Modifiers::ALT), Code::KeyQ);
            let _ = app.global_shortcut().register(sc);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("启动失败");
}
