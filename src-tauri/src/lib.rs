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
fn set_autofire(
    state: tauri::State<'_, AppState>,
    enabled: bool,
    weapon: String
) -> Result<(), String> {
    // Load the current game, category, loadout, and weapon indices
    let current_game_index = state.current_game_index.load(std::sync::atomic::Ordering::Relaxed);
    let current_category_index = state.current_category_index.load(std::sync::atomic::Ordering::Relaxed);
    let current_loadout_index = state.current_loadout_index.load(std::sync::atomic::Ordering::Relaxed);
    let current_weapon_index = state.current_weapon_index.load(std::sync::atomic::Ordering::Relaxed);
    
    // Get the weapon ID from the current game, category, loadout, and weapon indices
    let weapon_id = state.games.read_arc()
        .get(current_game_index).ok_or(format!("Game index {} not found", current_game_index))?
        .categories.get(current_category_index).ok_or(format!("Category index {} not found", current_category_index))?
        .loadouts.get(current_loadout_index).ok_or(format!("Loadout index {} not found", current_loadout_index))?
        .weapon_ids.get(current_weapon_index).ok_or(format!("Weapon index {} not found", current_weapon_index))?
        .to_string();

    match state.games.write_arc()
        .get_mut(current_game_index)
        .ok_or(format!("Game index {} not found", current_game_index))?
        .weapons.get_mut(&weapon_id)
        .ok_or(format!("Weapon ID {} not found in game!", weapon_id))?
    {
        Weapon::SingleFire(weapon_config) => {
            // Set the autofire property of the SingleFireConfig
            weapon_config.autofire = enabled;
        },
        _ => return Err(format!("Weapon {} is not a single-fire weapon", weapon_id)),
    }

    // Save the updated game data
    if let Err(e) = save_data((*state).clone()) {
        return Err(format!("Failed to save game data: {}", e));
    }

    println!("Set autofire for weapon {} to {}", weapon, enabled);
    Ok(())
}
fn load_data() -> Result<(Vec<Game>, GlobalConfig), String> {
    let assets_dir_path = Path::new("..").join("assets");
    let config_path = assets_dir_path.join("config.json");
    let games_dir_path = assets_dir_path.join("games");

    // Load global config from `assets/config.json`
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
    let global_config: GlobalConfig = (*state.global_config).clone();
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
        global_config: Arc::new(global_config),

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

            change_game,
            change_category,
            change_loadout,

            set_autofire,
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