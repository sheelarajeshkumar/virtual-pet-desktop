mod engine;

use std::{
    fs,
    path::PathBuf,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use engine::{Mode, PetEngine, PetKind, PetSettings, PetSnapshot, Point, Rect, TickInput};
#[cfg(target_os = "macos")]
use tauri::LogicalPosition;
#[cfg(not(target_os = "macos"))]
use tauri::PhysicalPosition;
use tauri::{
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager, State, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

type EngineState = Mutex<PetEngine>;

#[cfg(target_os = "macos")]
fn show_on_every_space(window: &WebviewWindow) -> Result<(), String> {
    use objc2_app_kit::{NSStatusWindowLevel, NSWindow, NSWindowCollectionBehavior};

    window
        .set_visible_on_all_workspaces(true)
        .map_err(|error| error.to_string())?;
    unsafe {
        let native = &*window
            .ns_window()
            .map_err(|error| error.to_string())?
            .cast::<NSWindow>();
        native.setCollectionBehavior(
            native.collectionBehavior()
                | NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::CanJoinAllApplications
                | NSWindowCollectionBehavior::Transient
                | NSWindowCollectionBehavior::IgnoresCycle
                | NSWindowCollectionBehavior::FullScreenAuxiliary,
        );
        native.setLevel(NSStatusWindowLevel);
        native.orderFrontRegardless();
    }
    Ok(())
}

struct DesktopGeometry {
    cursor: Point,
    work_area: Rect,
    window_size: Point,
    window_position: Point,
}

#[cfg(target_os = "macos")]
fn desktop_geometry(window: &WebviewWindow) -> Result<DesktopGeometry, String> {
    let cursor = window
        .cursor_position()
        .map_err(|error| error.to_string())?;
    let primary_scale = window
        .primary_monitor()
        .map_err(|error| error.to_string())?
        .map(|monitor| monitor.scale_factor())
        .unwrap_or(1.0);
    let cursor = cursor.to_logical::<f64>(primary_scale);
    let monitors = window
        .available_monitors()
        .map_err(|error| error.to_string())?;
    let monitor = monitors
        .into_iter()
        .find(|monitor| {
            let scale = monitor.scale_factor();
            let position = monitor.position().to_logical::<f64>(scale);
            let size = monitor.size().to_logical::<f64>(scale);
            cursor.x >= position.x
                && cursor.x < position.x + size.width
                && cursor.y >= position.y
                && cursor.y < position.y + size.height
        })
        .or_else(|| window.current_monitor().ok().flatten())
        .ok_or_else(|| "No display is available".to_string())?;
    let destination_scale = monitor.scale_factor();
    let work = monitor.work_area();
    let work_position = work.position.to_logical::<f64>(destination_scale);
    let work_size = work.size.to_logical::<f64>(destination_scale);
    let current_scale = window
        .current_monitor()
        .map_err(|error| error.to_string())?
        .map(|monitor| monitor.scale_factor())
        .unwrap_or(primary_scale);
    let window_position = window
        .outer_position()
        .map_err(|error| error.to_string())?
        .to_logical::<f64>(current_scale);
    let window_size = window
        .outer_size()
        .map_err(|error| error.to_string())?
        .to_logical::<f64>(current_scale);

    Ok(DesktopGeometry {
        cursor: Point::new(cursor.x, cursor.y),
        work_area: Rect::new(
            work_position.x,
            work_position.y,
            work_size.width,
            work_size.height,
        ),
        window_size: Point::new(window_size.width, window_size.height),
        window_position: Point::new(window_position.x, window_position.y),
    })
}

#[cfg(not(target_os = "macos"))]
fn desktop_geometry(window: &WebviewWindow) -> Result<DesktopGeometry, String> {
    let cursor = window
        .cursor_position()
        .map_err(|error| error.to_string())?;
    let monitor = window
        .monitor_from_point(cursor.x, cursor.y)
        .map_err(|error| error.to_string())?
        .or_else(|| window.current_monitor().ok().flatten())
        .ok_or_else(|| "No display is available".to_string())?;
    let work = monitor.work_area();
    let window_position = window.outer_position().map_err(|error| error.to_string())?;
    let window_size = window.outer_size().map_err(|error| error.to_string())?;

    Ok(DesktopGeometry {
        cursor: Point::new(cursor.x, cursor.y),
        work_area: Rect::new(
            work.position.x as f64,
            work.position.y as f64,
            work.size.width as f64,
            work.size.height as f64,
        ),
        window_size: Point::new(window_size.width as f64, window_size.height as f64),
        window_position: Point::new(window_position.x as f64, window_position.y as f64),
    })
}

#[tauri::command]
fn tick(window: WebviewWindow, state: State<'_, EngineState>) -> Result<PetSnapshot, String> {
    let geometry = desktop_geometry(&window)?;
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_millis() as u64;

    let snapshot = state
        .lock()
        .map_err(|_| "Pet state is unavailable".to_string())?
        .tick(TickInput {
            now_ms,
            cursor: geometry.cursor,
            work_area: geometry.work_area,
            window_size: geometry.window_size,
            window_position: geometry.window_position,
        });

    if (snapshot.position.x - geometry.window_position.x).abs() >= 1.0
        || (snapshot.position.y - geometry.window_position.y).abs() >= 1.0
    {
        #[cfg(target_os = "macos")]
        window
            .set_position(LogicalPosition::new(
                snapshot.position.x,
                snapshot.position.y,
            ))
            .map_err(|error| error.to_string())?;

        #[cfg(not(target_os = "macos"))]
        window
            .set_position(PhysicalPosition::new(
                snapshot.position.x.round() as i32,
                snapshot.position.y.round() as i32,
            ))
            .map_err(|error| error.to_string())?;
    }

    Ok(snapshot)
}

#[tauri::command]
fn get_settings(state: State<'_, EngineState>) -> Result<PetSettings, String> {
    state
        .lock()
        .map(|engine| engine.settings())
        .map_err(|_| "Pet settings are unavailable".to_string())
}

#[tauri::command]
fn save_settings(
    name: String,
    kind: String,
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, EngineState>,
) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 24 || name.chars().any(char::is_control) {
        return Err("Use a name between 1 and 24 characters.".to_string());
    }
    let kind = PetKind::parse(&kind).ok_or_else(|| "Choose Cat or Puppy.".to_string())?;
    let settings = PetSettings {
        name: name.to_string(),
        kind,
    };

    save_settings_file(&app, &settings)?;
    state
        .lock()
        .map_err(|_| "Pet settings are unavailable".to_string())?
        .set_settings(settings);

    if window.label() == "settings" {
        window.close().map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|directory| directory.join("settings.json"))
        .map_err(|error| error.to_string())
}

fn load_settings_file(app: &AppHandle) -> PetSettings {
    settings_path(app)
        .ok()
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

fn save_settings_file(app: &AppHandle, settings: &PetSettings) -> Result<(), String> {
    let path = settings_path(app)?;
    let directory = path
        .parent()
        .ok_or_else(|| "Settings path has no parent directory".to_string())?;
    fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    let json = serde_json::to_string_pretty(settings).map_err(|error| error.to_string())?;
    fs::write(path, json).map_err(|error| error.to_string())
}

fn set_mode(app: &AppHandle, mode: Mode) {
    if let Ok(mut engine) = app.state::<EngineState>().lock() {
        engine.set_mode(mode);
    }
}

fn open_settings(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("settings") {
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }

    WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("settings.html".into()))
        .title("Virtual Pet Settings")
        .inner_size(390.0, 285.0)
        .resizable(false)
        .always_on_top(true)
        .center()
        .build()?;
    Ok(())
}

fn tray_icon() -> Image<'static> {
    const SIZE: usize = 16;
    let mut rgba = vec![0_u8; SIZE * SIZE * 4];
    let orange = [255, 122, 26, 255];
    let ink = [36, 22, 17, 255];
    let pixels = [
        (3, 4),
        (4, 3),
        (6, 2),
        (9, 2),
        (11, 3),
        (12, 5),
        (5, 7),
        (6, 6),
        (7, 6),
        (8, 6),
        (9, 6),
        (10, 7),
        (4, 8),
        (5, 9),
        (6, 10),
        (7, 10),
        (8, 10),
        (9, 10),
        (10, 9),
        (11, 8),
    ];
    for (x, y) in pixels {
        let index = (y * SIZE + x) * 4;
        rgba[index..index + 4].copy_from_slice(if y <= 5 { &orange } else { &ink });
    }
    Image::new_owned(rgba, SIZE as u32, SIZE as u32)
}

fn build_tray(app: &tauri::App) -> tauri::Result<()> {
    let auto = MenuItem::with_id(app, "auto", "Auto", true, None::<&str>)?;
    let follow = MenuItem::with_id(app, "follow", "Call over", true, None::<&str>)?;
    let play = MenuItem::with_id(app, "play", "Play", true, None::<&str>)?;
    let bark = MenuItem::with_id(app, "bark", "Bark", true, None::<&str>)?;
    let sleep = MenuItem::with_id(app, "sleep", "Sleep", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Name and pet…", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Virtual Pet", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &auto, &follow, &play, &bark, &sleep, &settings, &separator, &quit,
        ],
    )?;

    TrayIconBuilder::with_id("virtual-pet")
        .icon(tray_icon())
        .tooltip("Virtual Pet")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "auto" => set_mode(app, Mode::Auto),
            "follow" => set_mode(app, Mode::Follow),
            "play" => set_mode(app, Mode::Play),
            "bark" => set_mode(app, Mode::Bark),
            "sleep" => set_mode(app, Mode::Sleep),
            "settings" => {
                let _ = open_settings(app);
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .build(app)?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![tick, get_settings, save_settings])
        .setup(|app| {
            app.manage(Mutex::new(PetEngine::new(load_settings_file(app.handle()))));

            #[cfg(target_os = "macos")]
            app.handle()
                .set_activation_policy(tauri::ActivationPolicy::Accessory)?;

            let window = app
                .get_webview_window("pet")
                .ok_or_else(|| "Pet window was not created".to_string())?;
            window.set_ignore_cursor_events(true)?;
            window.set_focusable(false)?;

            #[cfg(target_os = "macos")]
            show_on_every_space(&window)?;

            #[cfg(not(target_os = "macos"))]
            window.set_visible_on_all_workspaces(true)?;

            build_tray(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Virtual Pet");
}
