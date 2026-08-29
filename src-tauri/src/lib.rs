mod ai;
mod backup;
mod care;
mod engine;
mod extensions;
mod packs;
mod visibility;

use std::{
    fs,
    path::PathBuf,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use ai::{AiChatInput, AiChatResponse, AiCompanion, AiSettings, SuggestedAction};
use backup::BackupFile;
use care::{CareAction, CareActionResult, CareDifficulty, CareState};
use engine::{Mode, PetEngine, PetSettings, PetSize, PetSnapshot, Point, Rect, TickInput};
use extensions::{BehaviorExtension, ExtensionAction};
use packs::PetPackManifest;
use serde::{Deserialize, Serialize};
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
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use visibility::{Visibility, VisibilityState};

type EngineState = Mutex<PetEngine>;
type CareStore = Mutex<CareState>;
type AiStore = Mutex<AiCompanion>;
type VisibilityStore = Mutex<VisibilityState>;

fn now_ms() -> Result<u64, String> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())
        .map(|duration| duration.as_millis() as u64)
}

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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeSnapshot {
    #[serde(flatten)]
    pet: PetSnapshot,
    care: Option<CareState>,
    care_notifications: bool,
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
fn tick(
    local_hour: u8,
    window: WebviewWindow,
    state: State<'_, EngineState>,
    care: State<'_, CareStore>,
) -> Result<RuntimeSnapshot, String> {
    if local_hour > 23 {
        return Err("Local hour must be between 0 and 23.".to_string());
    }
    let geometry = desktop_geometry(&window)?;
    let now_ms = now_ms()?;

    let (snapshot, settings) = {
        let mut engine = state
            .lock()
            .map_err(|_| "Pet state is unavailable".to_string())?;
        let snapshot = engine.tick(TickInput {
            now_ms,
            local_hour,
            cursor: geometry.cursor,
            work_area: geometry.work_area,
            window_size: geometry.window_size,
            window_position: geometry.window_position,
        });
        (snapshot, engine.settings())
    };

    let care = {
        let mut care = care
            .lock()
            .map_err(|_| "Care state is unavailable".to_string())?;
        care.advance(now_ms, settings.care_difficulty, settings.care_enabled);
        settings.care_enabled.then(|| care.clone())
    };

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

    Ok(RuntimeSnapshot {
        pet: snapshot,
        care,
        care_notifications: settings.care_notifications,
    })
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
    settings: SaveSettingsInput,
    app: AppHandle,
    window: WebviewWindow,
    state: State<'_, EngineState>,
) -> Result<(), String> {
    let SaveSettingsInput {
        name,
        kind,
        pet_size,
        movement_speed,
        sound_volume,
        reduced_motion,
        day_night_enabled,
        care_enabled,
        care_notifications,
        care_difficulty,
    } = settings;
    let name = name.trim();
    if name.is_empty() || name.chars().count() > 24 || name.chars().any(char::is_control) {
        return Err("Use a name between 1 and 24 characters.".to_string());
    }
    if !valid_pet_id(&kind) || !pet_is_available(&app, &kind)? {
        return Err("Choose a valid installed pet.".to_string());
    }
    let pet_size =
        PetSize::parse(&pet_size).ok_or_else(|| "Choose a valid pet size.".to_string())?;
    if !movement_speed.is_finite() || !(0.5..=2.0).contains(&movement_speed) {
        return Err("Movement speed must be between 0.5 and 2.0.".to_string());
    }
    if sound_volume > 100 {
        return Err("Sound volume must be between 0 and 100.".to_string());
    }
    let care_difficulty = match care_difficulty.as_str() {
        "easy" => CareDifficulty::Easy,
        "normal" => CareDifficulty::Normal,
        "hard" => CareDifficulty::Hard,
        _ => return Err("Choose Easy, Normal, or Hard care difficulty.".to_string()),
    };
    let settings = PetSettings {
        name: name.to_string(),
        kind,
        pet_size,
        movement_speed,
        sound_volume,
        reduced_motion,
        day_night_enabled,
        care_enabled,
        care_notifications,
        care_difficulty,
    };
    validate_pet_settings(&app, &settings)?;
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

fn validate_pet_settings(app: &AppHandle, settings: &PetSettings) -> Result<(), String> {
    let name = settings.name.trim();
    if name.is_empty() || name.chars().count() > 24 || name.chars().any(char::is_control) {
        return Err("Use a name between 1 and 24 characters.".to_string());
    }
    if !valid_pet_id(&settings.kind) || !pet_is_available(app, &settings.kind)? {
        return Err("Choose a valid installed pet.".to_string());
    }
    if !settings.movement_speed.is_finite() || !(0.5..=2.0).contains(&settings.movement_speed) {
        return Err("Movement speed must be between 0.5 and 2.0.".to_string());
    }
    if settings.sound_volume > 100 {
        return Err("Sound volume must be between 0 and 100.".to_string());
    }
    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SaveSettingsInput {
    name: String,
    kind: String,
    pet_size: String,
    movement_speed: f64,
    sound_volume: u8,
    reduced_motion: bool,
    day_night_enabled: bool,
    care_enabled: bool,
    care_notifications: bool,
    care_difficulty: String,
}

fn valid_pet_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && !value.starts_with(['-', '.'])
        && !value.ends_with(['-', '.'])
        && !value.contains("..")
        && value.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || character == '-'
                || character == '.'
        })
}

fn pet_is_available(app: &AppHandle, id: &str) -> Result<bool, String> {
    if matches!(id, "puppy" | "cat" | "fox") {
        return Ok(true);
    }
    packs::list_installed_packs(&packs_path(app)?)
        .map(|packs| packs.iter().any(|pack| pack.id == id))
        .map_err(|error| error.to_string())
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

fn packs_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|directory| directory.join("packs"))
        .map_err(|error| error.to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct InstalledPackView {
    id: String,
    name: String,
    version: String,
    species: String,
    manifest_path: String,
}

fn installed_pack_view(
    root: &std::path::Path,
    manifest: PetPackManifest,
) -> Result<InstalledPackView, String> {
    let manifest_path = root.join(&manifest.id).join("pet-pack.json");
    let manifest_path = manifest_path
        .to_str()
        .ok_or_else(|| "The installed pet pack path is not valid Unicode.".to_string())?;
    Ok(InstalledPackView {
        id: manifest.id,
        name: manifest.name,
        version: manifest.version,
        species: manifest.species,
        manifest_path: manifest_path.to_string(),
    })
}

#[tauri::command]
fn list_installed_pets(app: AppHandle) -> Result<Vec<InstalledPackView>, String> {
    let root = packs_path(&app)?;
    packs::list_installed_packs(&root)
        .map_err(|error| error.to_string())?
        .into_iter()
        .map(|manifest| installed_pack_view(&root, manifest))
        .collect()
}

#[tauri::command]
fn install_pet_pack(source_directory: String, app: AppHandle) -> Result<InstalledPackView, String> {
    let root = packs_path(&app)?;
    let manifest = packs::install_pack(std::path::Path::new(&source_directory), &root)
        .map_err(|error| error.to_string())?;
    installed_pack_view(&root, manifest)
}

#[tauri::command]
fn remove_pet_pack(
    id: String,
    app: AppHandle,
    state: State<'_, EngineState>,
) -> Result<(), String> {
    let root = packs_path(&app)?;
    packs::remove_installed_pack(&root, &id).map_err(|error| error.to_string())?;
    let mut engine = state
        .lock()
        .map_err(|_| "Pet settings are unavailable".to_string())?;
    let mut settings = engine.settings();
    if settings.kind == id {
        settings.kind = "puppy".to_string();
        save_settings_file(&app, &settings)?;
        engine.set_settings(settings);
    }
    Ok(())
}

#[tauri::command]
fn export_pet_pack(
    id: String,
    destination_directory: String,
    app: AppHandle,
) -> Result<String, String> {
    packs::export_installed_pack(
        &packs_path(&app)?,
        &id,
        std::path::Path::new(&destination_directory),
    )
    .map_err(|error| error.to_string())?
    .to_str()
    .map(str::to_string)
    .ok_or_else(|| "The exported pet pack path is not valid Unicode.".to_string())
}

fn care_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|directory| directory.join("care.json"))
        .map_err(|error| error.to_string())
}

fn load_care_file(app: &AppHandle) -> CareState {
    care_path(app)
        .ok()
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_else(|| CareState::new(now_ms().unwrap_or(0)))
}

fn save_care_file(app: &AppHandle, care: &CareState) -> Result<(), String> {
    let path = care_path(app)?;
    let directory = path
        .parent()
        .ok_or_else(|| "Care path has no parent directory".to_string())?;
    fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    let json = serde_json::to_string_pretty(care).map_err(|error| error.to_string())?;
    fs::write(path, json).map_err(|error| error.to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CareView {
    name: String,
    enabled: bool,
    state: CareState,
}

fn care_view(settings: PetSettings, care: &CareState) -> CareView {
    CareView {
        name: settings.name,
        enabled: settings.care_enabled,
        state: care.clone(),
    }
}

#[tauri::command]
fn get_care_state(
    app: AppHandle,
    engine: State<'_, EngineState>,
    care: State<'_, CareStore>,
) -> Result<CareView, String> {
    let settings = engine
        .lock()
        .map_err(|_| "Pet settings are unavailable".to_string())?
        .settings();
    let mut care = care
        .lock()
        .map_err(|_| "Care state is unavailable".to_string())?;
    care.advance(now_ms()?, settings.care_difficulty, settings.care_enabled);
    save_care_file(&app, &care)?;
    Ok(care_view(settings, &care))
}

#[tauri::command]
fn perform_care_action(
    action: String,
    app: AppHandle,
    engine: State<'_, EngineState>,
    care: State<'_, CareStore>,
) -> Result<CareView, String> {
    let action = match action.as_str() {
        "feed" => CareAction::Feed,
        "play" => CareAction::Play,
        "wash" => CareAction::Wash,
        "pet" => CareAction::Pet,
        "sleep" => CareAction::Sleep,
        "wake" => CareAction::Wake,
        "rest" => CareAction::Rest,
        "restock" => CareAction::Restock,
        _ => return Err("Unknown care action.".to_string()),
    };
    apply_care_action(action, &app, &engine, &care)
}

fn apply_care_action(
    action: CareAction,
    app: &AppHandle,
    engine: &EngineState,
    care: &CareStore,
) -> Result<CareView, String> {
    let settings = engine
        .lock()
        .map_err(|_| "Pet settings are unavailable".to_string())?
        .settings();
    let mut care = care
        .lock()
        .map_err(|_| "Care state is unavailable".to_string())?;
    match care.apply(
        action,
        now_ms()?,
        settings.care_difficulty,
        settings.care_enabled,
    ) {
        CareActionResult::Applied => {}
        CareActionResult::Disabled => {
            return Err("Enable the care system in Settings first.".to_string());
        }
        CareActionResult::OutOfStock => {
            return Err(match action {
                CareAction::Feed => "No food left. Restock supplies first.",
                CareAction::Play => "No toys left. Restock supplies first.",
                _ => "Supplies are out of stock.",
            }
            .to_string());
        }
    }
    save_care_file(app, &care)?;
    let view = care_view(settings, &care);
    drop(care);

    match action {
        CareAction::Play => set_mode(app, Mode::Play),
        CareAction::Sleep => set_mode(app, Mode::Sleep),
        CareAction::Wake => set_mode(app, Mode::Auto),
        _ => {}
    }
    Ok(view)
}

fn set_mode(app: &AppHandle, mode: Mode) {
    let settings = if let Ok(mut engine) = app.state::<EngineState>().lock() {
        engine.set_mode(mode);
        Some(engine.settings())
    } else {
        None
    };
    if let (Some(settings), Ok(mut care)) = (settings, app.state::<CareStore>().lock()) {
        if let Ok(now) = now_ms() {
            care.advance(now, settings.care_difficulty, settings.care_enabled);
        }
        care.sleeping = mode == Mode::Sleep;
        let _ = save_care_file(app, &care);
    }
}

#[tauri::command]
fn export_backup(
    path: String,
    engine: State<'_, EngineState>,
    care: State<'_, CareStore>,
) -> Result<(), String> {
    let settings = engine
        .lock()
        .map_err(|_| "Pet settings are unavailable".to_string())?
        .settings();
    let care = care
        .lock()
        .map_err(|_| "Care state is unavailable".to_string())?
        .clone();
    backup::write_backup(
        std::path::Path::new(&path),
        &BackupFile::new(settings, care),
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn import_backup(
    path: String,
    app: AppHandle,
    engine: State<'_, EngineState>,
    care: State<'_, CareStore>,
) -> Result<(), String> {
    let mut backup =
        backup::read_backup(std::path::Path::new(&path)).map_err(|error| error.to_string())?;
    validate_pet_settings(&app, &backup.settings)?;
    backup.care.advance(
        now_ms()?,
        backup.settings.care_difficulty,
        backup.settings.care_enabled,
    );
    save_settings_file(&app, &backup.settings)?;
    save_care_file(&app, &backup.care)?;
    engine
        .lock()
        .map_err(|_| "Pet settings are unavailable".to_string())?
        .set_settings(backup.settings);
    *care
        .lock()
        .map_err(|_| "Care state is unavailable".to_string())? = backup.care;
    Ok(())
}

fn behaviors_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|directory| directory.join("behaviors"))
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn install_behavior_extension(path: String, app: AppHandle) -> Result<BehaviorExtension, String> {
    extensions::install_extension(std::path::Path::new(&path), &behaviors_path(&app)?)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn list_behavior_extensions(app: AppHandle) -> Result<Vec<BehaviorExtension>, String> {
    extensions::list_extensions(&behaviors_path(&app)?).map_err(|error| error.to_string())
}

#[tauri::command]
fn remove_behavior_extension(id: String, app: AppHandle) -> Result<(), String> {
    extensions::remove_extension(&behaviors_path(&app)?, &id).map_err(|error| error.to_string())
}

#[tauri::command]
fn set_extension_action(action: ExtensionAction, app: AppHandle) {
    set_mode(
        &app,
        match action {
            ExtensionAction::Auto => Mode::Auto,
            ExtensionAction::Follow => Mode::Follow,
            ExtensionAction::Play => Mode::Play,
            ExtensionAction::Bark => Mode::Bark,
            ExtensionAction::Sleep => Mode::Sleep,
        },
    );
}

fn visibility_path(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_config_dir()
        .map(|directory| directory.join("visibility.json"))
        .map_err(|error| error.to_string())
}

fn load_visibility_file(app: &AppHandle) -> VisibilityState {
    visibility_path(app)
        .ok()
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

fn save_visibility_file(app: &AppHandle, state: &VisibilityState) -> Result<(), String> {
    let path = visibility_path(app)?;
    fs::create_dir_all(
        path.parent()
            .ok_or_else(|| "Visibility path has no parent directory".to_string())?,
    )
    .map_err(|error| error.to_string())?;
    fs::write(
        path,
        serde_json::to_vec_pretty(state).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())
}

fn apply_visibility(app: &AppHandle, visibility: Visibility) -> Result<(), String> {
    let window = app
        .get_webview_window("pet")
        .ok_or_else(|| "Pet window is unavailable".to_string())?;
    if visibility.is_visible() {
        window.show().map_err(|error| error.to_string())?;
        #[cfg(target_os = "macos")]
        show_on_every_space(&window)?;
    } else {
        window.hide().map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn toggle_pet_visibility_inner(app: &AppHandle) -> Result<VisibilityState, String> {
    let store = app.state::<VisibilityStore>();
    let mut state = store
        .lock()
        .map_err(|_| "Visibility state is unavailable".to_string())?;
    let mut next = state.clone();
    apply_visibility(app, next.toggle())?;
    save_visibility_file(app, &next)?;
    *state = next.clone();
    Ok(next)
}

#[tauri::command]
fn get_visibility_settings(state: State<'_, VisibilityStore>) -> Result<VisibilityState, String> {
    state
        .lock()
        .map(|state| state.clone())
        .map_err(|_| "Visibility state is unavailable".to_string())
}

#[tauri::command]
fn toggle_pet_visibility(app: AppHandle) -> Result<VisibilityState, String> {
    toggle_pet_visibility_inner(&app)
}

#[tauri::command]
fn save_visibility_shortcut(
    shortcut: String,
    app: AppHandle,
    state: State<'_, VisibilityStore>,
) -> Result<VisibilityState, String> {
    let mut current = state
        .lock()
        .map_err(|_| "Visibility state is unavailable".to_string())?;
    let mut next = current.clone();
    next.set_shortcut(&shortcut)
        .map_err(|error| error.to_string())?;
    if next.shortcut != current.shortcut {
        app.global_shortcut()
            .unregister(current.shortcut.as_str())
            .map_err(|error| error.to_string())?;
        if let Err(error) = app.global_shortcut().register(next.shortcut.as_str()) {
            let _ = app.global_shortcut().register(current.shortcut.as_str());
            return Err(error.to_string());
        }
    }
    if let Err(error) = save_visibility_file(&app, &next) {
        if next.shortcut != current.shortcut {
            let _ = app.global_shortcut().unregister(next.shortcut.as_str());
            let _ = app.global_shortcut().register(current.shortcut.as_str());
        }
        return Err(error);
    }
    *current = next.clone();
    Ok(next)
}

#[tauri::command]
fn get_ai_settings(ai: State<'_, AiStore>) -> Result<AiSettings, String> {
    ai.lock()
        .map(|companion| companion.settings())
        .map_err(|_| "AI settings are unavailable".to_string())
}

#[tauri::command]
fn save_ai_settings(settings: AiSettings, ai: State<'_, AiStore>) -> Result<AiSettings, String> {
    ai.lock()
        .map_err(|_| "AI settings are unavailable".to_string())?
        .save_settings(settings)
}

#[tauri::command]
fn clear_ai_memory(ai: State<'_, AiStore>) -> Result<(), String> {
    ai.lock()
        .map_err(|_| "AI settings are unavailable".to_string())?
        .clear_memory()
}

fn add_pet_context(
    mut input: AiChatInput,
    engine: &EngineState,
    care: &CareStore,
) -> Result<AiChatInput, String> {
    let settings = engine
        .lock()
        .map_err(|_| "Pet settings are unavailable".to_string())?
        .settings();
    let care = care
        .lock()
        .map_err(|_| "Care state is unavailable".to_string())?
        .clone();
    if input.mood.is_none() {
        input.mood = Some(
            if care.sleeping {
                "sleepy"
            } else if care.hunger >= 65 {
                "hungry"
            } else if care.energy <= 35 {
                "tired"
            } else if care.happiness <= 40 {
                "lonely"
            } else {
                "happy"
            }
            .to_string(),
        );
    }
    let user_context = input
        .context
        .take()
        .unwrap_or_default()
        .chars()
        .take(280)
        .collect::<String>();
    input.context = Some(format!(
        "Pet {} ({}) care: hunger {}, energy {}, happiness {}, cleanliness {}, food {}, toys {}, sleeping {}. {}",
        settings.name,
        settings.kind,
        care.hunger,
        care.energy,
        care.happiness,
        care.cleanliness,
        care.food,
        care.toys,
        care.sleeping,
        user_context
    ));
    Ok(input)
}

async fn run_ai_chat(input: AiChatInput, ai: &AiStore, now: u64) -> Result<AiChatResponse, String> {
    let prepared = ai
        .lock()
        .map_err(|_| "AI settings are unavailable".to_string())?
        .prepare_chat(input, now)?;
    let network_copy = prepared.clone();
    let completed = tauri::async_runtime::spawn_blocking(move || ai::execute_chat(&network_copy))
        .await
        .map_err(|error| error.to_string())??;
    ai.lock()
        .map_err(|_| "AI settings are unavailable".to_string())?
        .finish_chat(prepared, completed, now)
}

#[tauri::command]
async fn ai_chat(
    input: AiChatInput,
    ai: State<'_, AiStore>,
    engine: State<'_, EngineState>,
    care: State<'_, CareStore>,
) -> Result<AiChatResponse, String> {
    let input = add_pet_context(input, &engine, &care)?;
    run_ai_chat(input, &ai, now_ms()?).await
}

#[tauri::command]
async fn ai_proactive(
    mood: Option<String>,
    context: Option<String>,
    ai: State<'_, AiStore>,
    engine: State<'_, EngineState>,
    care: State<'_, CareStore>,
) -> Result<Option<AiChatResponse>, String> {
    let now = now_ms()?;
    if !ai
        .lock()
        .map_err(|_| "AI settings are unavailable".to_string())?
        .proactive_due(now)
    {
        return Ok(None);
    }
    let input = add_pet_context(
        AiChatInput {
            message: String::new(),
            mood,
            context,
            proactive: true,
        },
        &engine,
        &care,
    )?;
    run_ai_chat(input, &ai, now).await.map(Some)
}

#[tauri::command]
fn confirm_ai_action(
    action: String,
    app: AppHandle,
    engine: State<'_, EngineState>,
    care: State<'_, CareStore>,
) -> Result<(), String> {
    match SuggestedAction::parse(&action).ok_or_else(|| "Unknown AI action.".to_string())? {
        SuggestedAction::Bark => set_mode(&app, Mode::Bark),
        SuggestedAction::Feed => {
            apply_care_action(CareAction::Feed, &app, &engine, &care)?;
        }
        SuggestedAction::Play => {
            apply_care_action(CareAction::Play, &app, &engine, &care)?;
        }
        SuggestedAction::Wash => {
            apply_care_action(CareAction::Wash, &app, &engine, &care)?;
        }
        SuggestedAction::Pet => {
            apply_care_action(CareAction::Pet, &app, &engine, &care)?;
        }
        SuggestedAction::Sleep => {
            apply_care_action(CareAction::Sleep, &app, &engine, &care)?;
        }
        SuggestedAction::Wake => {
            apply_care_action(CareAction::Wake, &app, &engine, &care)?;
        }
    }
    Ok(())
}

fn open_settings(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("settings") {
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }

    WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("settings.html".into()))
        .title("Virtual Pet Settings")
        .inner_size(430.0, 830.0)
        .resizable(false)
        .always_on_top(true)
        .center()
        .build()?;
    Ok(())
}

fn open_care(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("care") {
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }

    WebviewWindowBuilder::new(app, "care", WebviewUrl::App("care.html".into()))
        .title("Pet Care")
        .inner_size(410.0, 430.0)
        .resizable(false)
        .always_on_top(true)
        .center()
        .build()?;
    Ok(())
}

fn open_packs(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("packs") {
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }

    WebviewWindowBuilder::new(app, "packs", WebviewUrl::App("packs.html".into()))
        .title("Pet Packs")
        .inner_size(500.0, 540.0)
        .resizable(false)
        .always_on_top(true)
        .center()
        .build()?;
    Ok(())
}

fn open_pack_preview(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("pack-preview") {
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }
    WebviewWindowBuilder::new(
        app,
        "pack-preview",
        WebviewUrl::App("pack-preview.html".into()),
    )
    .title("Pet Pack Preview")
    .inner_size(760.0, 620.0)
    .resizable(true)
    .always_on_top(true)
    .center()
    .build()?;
    Ok(())
}

fn open_data(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("data") {
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }
    WebviewWindowBuilder::new(app, "data", WebviewUrl::App("data.html".into()))
        .title("Virtual Pet Data")
        .inner_size(470.0, 500.0)
        .resizable(false)
        .always_on_top(true)
        .center()
        .build()?;
    Ok(())
}

fn open_extensions(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("extensions") {
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }
    WebviewWindowBuilder::new(app, "extensions", WebviewUrl::App("extensions.html".into()))
        .title("Pet Behaviors")
        .inner_size(520.0, 560.0)
        .resizable(true)
        .always_on_top(true)
        .center()
        .build()?;
    Ok(())
}

fn open_ai(app: &AppHandle) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window("ai") {
        window.show()?;
        window.set_focus()?;
        return Ok(());
    }
    WebviewWindowBuilder::new(app, "ai", WebviewUrl::App("ai.html".into()))
        .title("AI Companion")
        .inner_size(560.0, 720.0)
        .resizable(true)
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
    let care = MenuItem::with_id(app, "care", "Care…", true, None::<&str>)?;
    let visibility = MenuItem::with_id(app, "visibility", "Hide/show pet", true, None::<&str>)?;
    let packs = MenuItem::with_id(app, "packs", "Pet packs…", true, None::<&str>)?;
    let preview = MenuItem::with_id(app, "preview", "Preview pet packs…", true, None::<&str>)?;
    let behaviors = MenuItem::with_id(app, "behaviors", "Behaviors…", true, None::<&str>)?;
    let data = MenuItem::with_id(app, "data", "Backup and export…", true, None::<&str>)?;
    let ai = MenuItem::with_id(app, "ai", "AI companion…", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Name and pet…", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Virtual Pet", true, None::<&str>)?;
    let menu = Menu::with_items(
        app,
        &[
            &auto,
            &follow,
            &play,
            &bark,
            &sleep,
            &care,
            &visibility,
            &packs,
            &preview,
            &behaviors,
            &data,
            &ai,
            &settings,
            &separator,
            &quit,
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
            "care" => {
                let _ = open_care(app);
            }
            "visibility" => {
                let _ = toggle_pet_visibility_inner(app);
            }
            "packs" => {
                let _ = open_packs(app);
            }
            "preview" => {
                let _ = open_pack_preview(app);
            }
            "behaviors" => {
                let _ = open_extensions(app);
            }
            "data" => {
                let _ = open_data(app);
            }
            "ai" => {
                let _ = open_ai(app);
            }
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
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state() == ShortcutState::Pressed {
                        let _ = toggle_pet_visibility_inner(app);
                    }
                })
                .build(),
        )
        .plugin(tauri_plugin_notification::init())
        .invoke_handler(tauri::generate_handler![
            tick,
            get_settings,
            save_settings,
            get_care_state,
            perform_care_action,
            list_installed_pets,
            install_pet_pack,
            remove_pet_pack,
            export_pet_pack,
            export_backup,
            import_backup,
            install_behavior_extension,
            list_behavior_extensions,
            remove_behavior_extension,
            set_extension_action,
            get_visibility_settings,
            toggle_pet_visibility,
            save_visibility_shortcut,
            get_ai_settings,
            save_ai_settings,
            clear_ai_memory,
            ai_chat,
            ai_proactive,
            confirm_ai_action
        ])
        .setup(|app| {
            app.manage(Mutex::new(PetEngine::new(load_settings_file(app.handle()))));
            app.manage(Mutex::new(load_care_file(app.handle())));
            let visibility = load_visibility_file(app.handle());
            let shortcut = visibility.shortcut.as_str().to_string();
            let initial_visibility = visibility.visibility;
            app.manage(Mutex::new(visibility));
            let ai_path = app.path().app_config_dir()?.join("ai.json");
            app.manage(Mutex::new(AiCompanion::load(ai_path)));

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

            app.global_shortcut().register(shortcut.as_str())?;
            apply_visibility(app.handle(), initial_visibility)?;
            build_tray(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Virtual Pet");
}

#[cfg(test)]
mod tests {
    use super::valid_pet_id;

    #[test]
    fn pet_ids_are_safe_catalog_keys() {
        assert!(valid_pet_id("fluffy-cat-2"));
        assert!(valid_pet_id("example.fluffy-cat-2"));
        assert!(!valid_pet_id("../cat"));
        assert!(!valid_pet_id("Cat"));
        assert!(!valid_pet_id(""));
    }
}
