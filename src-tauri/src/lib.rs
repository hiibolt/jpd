mod winapi;
mod recoil;
mod types;

use parking_lot::{Mutex, RwLock};
use tauri::{ipc::Channel, App, Builder, Manager};
use window_vibrancy::apply_acrylic;
use anyhow::{anyhow, Result};

use std::{path::PathBuf, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, Arc}};

use crate::{types::{KeyStatus, LoadedGames}, winapi::main_recoil};
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
        if new_category_index <     game.categories.as_ref().map(|c| c.len()).unwrap_or(0) {
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
    save_data(&state).map_err(|e| format!("Failed to save game data: {}", e))?;

    Ok(state.global_config.read_arc().clone())
}
#[tauri::command]
async fn change_horizontal_multiplier (
    state: tauri::State<'_, AppState>,
    new_multiplier: f32
) -> Result<GlobalConfig, String> {
    if new_multiplier > 0.0 {
        state.global_config.write_arc().mouse_config.horizontal_multiplier = new_multiplier;
        save_data(&state).map_err(|e| format!("Failed to save game data: {}", e))?;
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
        save_data(&state).map_err(|e| format!("Failed to save game data: {}", e))?;
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
        if let Some(category) = game.categories
            .as_ref().ok_or(format!("Game `{}` does not have data loaded.", current_game_index))?
            .get(current_category_index)
        {
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
        .weapons.as_mut()
        .ok_or(format!("No weapons found in game `{}`", current_game_index))?
        .get_mut(&weapon_id)
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
                "rpm" => weapon_config.rpm = new_value.as_u64().ok_or("Invalid value for rpm")?,
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
    save_data(&state).map_err(|e| format!("Failed to save game data: {}", e))?;
    println!("Updated field `{}` for weapon `{}` in game `{}` to `{}`",
        field, weapon_id, current_game_index, new_value
    );

    // Return the updated games list
    Ok(state.games.read_arc().clone())
}
#[tauri::command]
async fn load_games_wrapper (
    state: tauri::State<'_, AppState>
) -> Result<(), String> {
    let LoadedGames {
        game_data,
        ..
    } = load_games((*state.assets_dir_path).clone()).await?;

    *state.games.write_arc() = game_data;

    Ok(())
}

fn get_weapon_id (
    state: &AppState,
) -> Result<String> {
    let games = state.games.read_arc();
    let current_game_index = state.current_game_index.load(Ordering::SeqCst);
    let current_game = &games.get(current_game_index)
        .ok_or(anyhow!("Game index {} not found", current_game_index))?;

    // Get the current category
    let current_category_index = state.current_category_index.load(Ordering::SeqCst);
    let current_category = &current_game.categories
        .as_ref().ok_or(anyhow!("Game `{}` does not have categories loaded", current_game.name))?
        .get(current_category_index)
        .ok_or(anyhow!("Category index {} not found in game `{}`", current_category_index, current_game.name))?;

    // Get the current loadout
    let current_loadout_index = state.current_loadout_index.load(Ordering::SeqCst);
    let current_loadout = &current_category.loadouts.get(current_loadout_index)
        .ok_or(anyhow!("Loadout index {} not found in category `{}` in game `{}`", current_loadout_index, current_category.name, current_game.name))?;

    // Get the current weapon index
    let weapon_ind = state.current_weapon_index.load(Ordering::SeqCst);
    let weapon_id = current_loadout.weapon_ids.get(weapon_ind)
        .ok_or(anyhow!("Weapon index `{}` not found in loadout `{}` in category `{}` in game `{}`", weapon_ind, current_loadout.name, current_category.name, current_game.name))?;

    Ok(weapon_id.clone())
}
async fn load_games (
    assets_dir_path: PathBuf
) -> Result<LoadedGames, String> {
    // Load each game's data from `assets/games.json`
    let mut games_ret = Vec::new();
    let mut key_statuses_ret = Vec::new();

    // Load the games
    let server_base_url = "http://localhost:4777";
    let game_list = reqwest::get(format!("{server_base_url}/v1/products/jpd"))
        .await
        .map_err(|e| format!("Failed to fetch games list: {}", e))?
        .json::<Vec<String>>()
        .await
        .map_err(|e| format!("Failed to read games list: {}", e))?;
    println!("Available games configs: {:?}", game_list);

    for game_id in &game_list {
        // Attempt to load the contents of `assets/{game}.key`
        let game_key_path = assets_dir_path.join(format!("{}.key", game_id));
        let key = if game_key_path.exists() {
            match std::fs::read_to_string(&game_key_path) {
                Ok(key) => {
                    println!("Loaded key for game `{}` from file: {}", game_id, game_key_path.display());
                    key
                },
                Err(e) => {
                    eprintln!("Failed to read key file for game `{}`: {}", game_id, e);
                    games_ret.push(Game {
                        name: game_id.clone(),
                        key: None,
                        categories: None,
                        weapons: None,
                    });
                    continue; // Skip this game if the key file cannot be read
                }
            }
        } else {
            println!("No key file found for game `{}`, skipping", game_id);
            games_ret.push(Game {
                name: game_id.clone(),
                key: None,
                categories: None,
                weapons: None,
            });
            continue; // Skip this game if no key file exists
        };

        let url = format!(
            "{server_base_url}/v1/validate/jpd/{}/{}",
            game_id, key
        );

        let key_status = reqwest::get(url)
            .await
            .map_err(|e| format!("Failed to validate key for game `{}`: {}", game_id, e))?
            .json::<KeyStatus>()
            .await
            .map_err(|e| format!("Failed to read key status for game `{}`: {}", game_id, e))?;

        if let KeyStatus::Valid { config, .. } = &key_status {
            println!("Key for game `{}` is valid, loading config", game_id);
            let mut config = config.clone();
            config.key = Some(key);
            games_ret.push(config);
        } else {
            println!("Key for game `{}` is invalid or expired, skipping", game_id);
            games_ret.push(Game {
                name: game_id.clone(),
                key: Some(key),
                categories: None,
                weapons: None,
            });
            continue; // Skip this game if the key is invalid
        }

        key_statuses_ret.push(key_status);
    }

    println!("Loaded {} games", games_ret.len());

    Ok(LoadedGames {
        game_data: games_ret,
        key_statuses: key_statuses_ret,
    })
}
fn load_config (
    assets_dir_path: &PathBuf
) -> Result<GlobalConfig, String> {
    let config_path = assets_dir_path.join("config.json");

    // If the `assets/config.json` file does not exist, create it with default values
    if !config_path.exists() {
        let default_config = GlobalConfig::default();
        std::fs::write(
            &config_path,
            serde_json::to_string_pretty(&default_config).map_err(|e| format!("Failed to serialize default config: {}", e))?
        ).map_err(|e| format!("Failed to write default config file: {}", e))?;
    }

    // Load global config from `assets/config.json`
    println!("Loading global config from: {}", config_path.display());
    let global_config: GlobalConfig = serde_json::from_str(
        &std::fs::read_to_string(config_path).map_err(|e| format!("Failed to read config file: {}", e))?
    ).map_err(|e| format!("Failed to parse global config: {}", e))?;

    Ok(global_config)
}
fn save_data(
    state: &AppState
) -> Result<(), String> {
    let assets_dir_path = &state.assets_dir_path;

    // Save config to `assets/config.json`
    let global_config: GlobalConfig = state.global_config.read_arc().clone();
    std::fs::write(
        assets_dir_path.join("config.json"),
        serde_json::to_string_pretty(&global_config).map_err(|e| e.to_string())?
    ).map_err(|e| format!("Failed to write config file: {}", e))?;

    // Save each game's data to `assets/<game_name>/data.json`
    let games_dir_path = assets_dir_path.join("games");
    for game in state.games.read_arc().iter() {
        let game_path = games_dir_path.join(&game.name);
        let game_contents = serde_json::to_string_pretty(game)
            .map_err(|e| format!("Failed to serialize game data: {}", e))?;

        std::fs::create_dir_all(&game_path).map_err(|e| format!("Failed to create game directory for {}: {}", game.name, e))?;
        std::fs::write(
            game_path.join("data.json"),
            game_contents
        ).map_err(|e| format!("Failed to write game data for {}: {}", game.name, e))?;
    }

    Ok(())
}

async fn setup(
    app: &mut App
) -> AppState {
    let resource_assets_dir = app.path().resource_dir()
        .expect("Failed to get resource directory")
        .join("assets");
    let assets_dir_path = Arc::new(if resource_assets_dir.exists() {
        println!("Using resource directory: {}", resource_assets_dir.display());
        resource_assets_dir
    } else {
        println!("Using default assets directory: ./assets");
        PathBuf::from("assets")
    });

    let config = match load_config(&assets_dir_path) {
        Ok(data) => data,
        Err(e) => {
            // If loading data fails, create an `assets/logs` directory and log the error
            let log_dir = assets_dir_path.join("logs");
            std::fs::create_dir_all(&log_dir).expect("Failed to create logs directory");
            
            let log_file_path = log_dir.join("error.log");
            std::fs::write(&log_file_path, &e).expect("Failed to write error log");
            eprintln!("Failed to load data: {}. Error logged to {}", e, log_file_path.display());
            
            panic!("Failed to load data: {}", e);
        }
    };

    let (event_tx, event_rx) = std::sync::mpsc::channel();
    let state = AppState {
        games:           Arc::new(RwLock::new(vec!())),
        global_config:   Arc::new(RwLock::new(config)),
        assets_dir_path,

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
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            get_games,
            get_config,

            change_game,
            change_category,
            change_loadout,
            change_horizontal_multiplier,
            change_vertical_multiplier,
            change_setting,

            load_games_wrapper,
            set_weapon_config,
            start_channel_reads
        ])
        .setup(|app| {
            let state = tauri::async_runtime::block_on(setup(app));
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