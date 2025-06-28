mod winapi;
mod recoil;
mod types;

use parking_lot::{Mutex, RwLock};
use tauri::{ipc::Channel, Builder, Manager};
use window_vibrancy::apply_acrylic;

use std::{path::Path, sync::{atomic::{AtomicBool, AtomicUsize}, Arc}};

use crate::winapi::main_recoil;
use crate::types::{AppEvent, AppState, Game, GlobalConfig, Weapon};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn get_games(state: tauri::State<'_, AppState>) -> Vec<Game> {
    state.games.read_arc().clone()
}
#[tauri::command]
fn get_config(state: tauri::State<'_, AppState>) -> GlobalConfig {
    state.global_config.read_arc().clone()
}
#[tauri::command]
async fn start_channel_reads (
    state: tauri::State<'_, AppState>,
    channel: Channel<AppEvent>,
) -> Result<String, String> {
    while let Ok(event) = state.events_channel_reciever.lock().recv() {
        channel.send(event).expect("Failed to send event");
    }

    Err(String::from("Channel reads closed early?"))
}
#[tauri::command]
async fn change_game (
    state: tauri::State<'_, AppState>,
    new_game_index: usize
) -> Result<usize, String> {
    if new_game_index < state.games.read_arc().len() {
        state.current_game_index.store(new_game_index, std::sync::atomic::Ordering::Relaxed);
        println!("Changed game to index {}", new_game_index);
        return Ok(new_game_index);
    }

    Err(format!("Invalid game index: {}", new_game_index))
}
#[tauri::command]
async fn change_category (
    state: tauri::State<'_, AppState>,
    new_category_index: usize
) -> Result<usize, String> {
    let current_game_index = state.current_game_index.load(std::sync::atomic::Ordering::Relaxed);

    if let Some(game) = state.games.read_arc().get(current_game_index) {
        if new_category_index < game.categories.len() {
            state.current_category_index.store(new_category_index, std::sync::atomic::Ordering::Relaxed);
            println!("Changed category to index {}", new_category_index);
            return Ok(new_category_index);
        }
    }

    Err(format!("Invalid category index: {}", new_category_index))
}
#[tauri::command]
async fn change_setting (
    state: tauri::State<'_, AppState>,
    setting: String,
    value: serde_json::Value
) -> Result<GlobalConfig, String> {
    match setting.as_str() {
        "primary_weapon" | "secondary_weapon" | "alternative_fire" => {
            let new_value = value.as_str().ok_or("Invalid value! Must be a string.")?;
            if new_value.len() != 1 {
                return Err("Primary weapon must be a single character".to_string());
            }

            match setting.as_str() {
                "primary_weapon" => state.global_config.write_arc().keybinds.primary_weapon = new_value.chars().next().unwrap(),
                "secondary_weapon" => state.global_config.write_arc().keybinds.secondary_weapon = new_value.chars().next().unwrap(),
                "alternative_fire" => state.global_config.write_arc().keybinds.alternative_fire = new_value.chars().next().unwrap(),
                _ => unreachable!(),
            }
        },
        _ => return Err(format!("Unknown setting: {}", setting)),
    }

    println!("Updated setting `{}` to `{}`", setting, value);
    save_data((*state).clone()).map_err(|e| format!("Failed to save game data: {}", e))?;

    Ok(state.global_config.read_arc().clone())
}
#[tauri::command]
async fn change_horizontal_multiplier (
    state: tauri::State<'_, AppState>,
    new_multiplier: f32
) -> Result<GlobalConfig, String> {
    if new_multiplier > 0.0 {
        state.global_config.write_arc().mouse_config.horizontal_multiplier = new_multiplier;
        save_data((*state).clone()).map_err(|e| format!("Failed to save game data: {}", e))?;
        println!("Changed horizontal multiplier to {}", new_multiplier);
        
        return Ok(state.global_config.read_arc().clone());
    }

    Err(format!("Invalid horizontal multiplier: {}", new_multiplier))
}
#[tauri::command]
async fn change_vertical_multiplier (
    state: tauri::State<'_, AppState>,
    new_multiplier: f32
) -> Result<GlobalConfig, String> {
    if new_multiplier > 0.0 {
        state.global_config.write_arc().mouse_config.vertical_multiplier = new_multiplier;
        save_data((*state).clone()).map_err(|e| format!("Failed to save game data: {}", e))?;
        println!("Changed vertical multiplier to {}", new_multiplier);

        return Ok(state.global_config.read_arc().clone());
    }

    Err(format!("Invalid vertical multiplier: {}", new_multiplier))
}
#[tauri::command]
async fn change_loadout (
    state: tauri::State<'_, AppState>,
    new_loadout_index: usize
) -> Result<usize, String> {
    let current_game_index = state.current_game_index.load(std::sync::atomic::Ordering::Relaxed);
    let current_category_index = state.current_category_index.load(std::sync::atomic::Ordering::Relaxed);

    if let Some(game) = state.games.read_arc().get(current_game_index) {
        if let Some(category) = game.categories.get(current_category_index) {
            if new_loadout_index < category.loadouts.len() {
                state.current_loadout_index.store(new_loadout_index, std::sync::atomic::Ordering::Relaxed);
                println!("Changed loadout to index {}", new_loadout_index);
                return Ok(new_loadout_index);
            }
        }
    }

    Err(format!("Invalid loadout index: {}", new_loadout_index))
}
#[tauri::command]
fn set_weapon_config(
    state: tauri::State<'_, AppState>,
    weapon_id: String,

    field: String,
    new_value: serde_json::Value
) -> Result<Vec<Game>, String> {
    // Load the current game, category, loadout, and weapon indices
    let current_game_index = state.current_game_index.load(std::sync::atomic::Ordering::Relaxed);
    
    match state.games.write_arc()
        .get_mut(current_game_index).ok_or(format!("Game index {} not found", current_game_index))?
        .weapons.get_mut(&weapon_id)
        .ok_or(format!("Weapon ID `{}` not found in game `{}`", weapon_id, current_game_index))?
    {
        Weapon::SingleFire(weapon_config) => {
            match field.as_str() {
                "name" => weapon_config.name = new_value.as_str().ok_or("Invalid value for name")?.to_string(),
                "description" => weapon_config.description = new_value.as_str().map(|s| s.to_string()),
                "trigger_delay_ms" => weapon_config.trigger_delay_ms = new_value.as_u64().ok_or("Invalid value for trigger_delay_ms")? as u32,
                "recoil_completion_ms" => weapon_config.recoil_completion_ms = new_value.as_u64().ok_or("Invalid value for recoil_completion_ms")? as u32,
                "release_delay_ms" => weapon_config.release_delay_ms = new_value.as_u64().ok_or("Invalid value for release_delay_ms")? as u32,
                "dx" => weapon_config.dx = new_value.as_f64().ok_or("Invalid value for dx")? as f32,
                "dy" => weapon_config.dy = new_value.as_f64().ok_or("Invalid value for dy")? as f32,
                "mag_size" => weapon_config.mag_size = new_value.as_u64().ok_or("Invalid value for mag_size")? as u32,
                "autofire" => weapon_config.autofire = new_value.as_bool().ok_or("Invalid value for autofire")?,
                _ => return Err(format!("Unknown field: {}", field)),
            }
        },
        Weapon::FullAutoStandard(weapon_config) => {
            match field.as_str() {
                "name" => weapon_config.name = new_value.as_str().ok_or("Invalid value for name")?.to_string(),
                "description" => weapon_config.description = new_value.as_str().map(|s| s.to_string()),
                "rpm" => weapon_config.rpm = new_value.as_u64().ok_or("Invalid value for rpm")? as u128,
                "first_shot_scale" => weapon_config.first_shot_scale = new_value.as_f64().ok_or("Invalid value for first_shot_scale")? as f32,
                "exponential_factor" => weapon_config.exponential_factor = new_value.as_f64().ok_or("Invalid value for exponential_factor")? as f32,
                "dx" => weapon_config.dx = new_value.as_f64().ok_or("Invalid value for dx")? as f32,
                "dy" => weapon_config.dy = new_value.as_f64().ok_or("Invalid value for dy")? as f32,
                "mag_size" => weapon_config.mag_size = new_value.as_u64().ok_or("Invalid value for mag_size")? as u32,
                _ => return Err(format!("Unknown field: {}", field)),
            }
        },
    }

    // Save the updated game data
    save_data((*state).clone()).map_err(|e| format!("Failed to save game data: {}", e))?;
    println!("Updated field `{}` for weapon `{}` in game `{}` to `{}`",
        field, weapon_id, current_game_index, new_value
    );

    // Return the updated games list
    Ok(state.games.read_arc().clone())
}
fn load_data() -> Result<(Vec<Game>, GlobalConfig), String> {
    let assets_dir_path = Path::new("..").join("assets");
    let config_path = assets_dir_path.join("config.json");
    let games_dir_path = assets_dir_path.join("games");

    // If the `assets/config.json` file does not exist, create it with default values
    if !config_path.exists() {
        let default_config = GlobalConfig::default();
        std::fs::write(
            &config_path,
            serde_json::to_string_pretty(&default_config).map_err(|e| e.to_string())?
        ).map_err(|e| e.to_string())?;
    }

    // Load global config from `assets/config.json``
    let global_config: GlobalConfig = serde_json::from_str(
        &std::fs::read_to_string(config_path).map_err(|e| e.to_string())?
    ).map_err(|e| e.to_string())?;

    // Load each game's data from `assets/<game_name>/data.json`
    let mut games = Vec::new();
    for entry in std::fs::read_dir(games_dir_path).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().map_err(|e| e.to_string())?.is_dir() {
            let game_name = entry.file_name().to_string_lossy().to_string();
            let game_data_path = entry.path().join("data.json");

            match std::fs::read_to_string(&game_data_path) {
                Ok(data) => match serde_json::from_str::<Game>(&data) {
                    Ok(game) => games.push(game),
                    Err(e) => return Err(format!("Failed to parse game data for {}: {}", game_name, e)),
                },
                Err(_) => return Err(format!("Failed to read game data for {}", game_name)),
            }
        }
    }

    Ok((games, global_config))
}
fn save_data(
    state: AppState
) -> Result<(), String> {
    let assets_dir_path = Path::new("..").join("assets");

    // Save config to `assets/config.json`
    let global_config: GlobalConfig = state.global_config.read_arc().clone();
    std::fs::write(
        assets_dir_path.join("config.json"),
        serde_json::to_string_pretty(&global_config).map_err(|e| e.to_string())?
    ).map_err(|e| e.to_string())?;

    // Save each game's data to `assets/<game_name>/data.json`
    let games_dir_path = assets_dir_path.join("games");
    for game in state.games.read_arc().iter() {
        let game_path = games_dir_path.join(&game.name);
        let game_contents = serde_json::to_string_pretty(game)
            .map_err(|e| e.to_string())?;

        std::fs::create_dir_all(&game_path).map_err(|e| e.to_string())?;
        std::fs::write(
            game_path.join("data.json"),
            game_contents
        ).map_err(|e| e.to_string())?;
    }

    Ok(())
}

async fn setup() -> AppState {
    let (games, global_config) = load_data()
        .expect("Failed to load game data!");

    let (event_tx, event_rx) = std::sync::mpsc::channel();
    let state = AppState {
        games:         Arc::new(RwLock::new(games)),
        global_config: Arc::new(RwLock::new(global_config)),

        events_channel_sender:   Arc::new(event_tx),
        events_channel_reciever: Arc::new(Mutex::new(event_rx)),

        left_hold_active:       Arc::new(AtomicBool::new(false)),
        right_hold_active:      Arc::new(AtomicBool::new(false)),
        current_game_index:     Arc::new(AtomicUsize::new(0)),
        current_category_index: Arc::new(AtomicUsize::new(0)),
        current_loadout_index:  Arc::new(AtomicUsize::new(0)),
        current_weapon_index:   Arc::new(AtomicUsize::new(0)),
    };
    let state_cloned = state.clone();

    tokio::spawn(async move {
        main_recoil(state_cloned);
    });

    state
}
pub fn run() {
    Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_games,
            get_config,

            change_game,
            change_category,
            change_loadout,
            change_horizontal_multiplier,
            change_vertical_multiplier,
            change_setting,

            set_weapon_config,
            start_channel_reads
        ])
        .setup(|app| {
            let state = tauri::async_runtime::block_on(setup());
            let window = app.get_webview_window("main").expect("Failed to get main window");

            // Apply vibrancy effect on Windows
            apply_acrylic(
                &window,
                Some((18, 18, 18, 125))
            ).expect("Unsupported platform! 'apply_blur' is only supported on Windows");

            // Register the application state
            app.manage(state);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}