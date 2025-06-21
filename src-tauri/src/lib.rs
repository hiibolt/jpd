mod winapi;

use parking_lot::{Mutex, RwLock};
use tauri::{ipc::Channel, Builder, Manager};
use window_vibrancy::apply_acrylic;

use std::{collections::HashMap, path::Path, sync::{atomic::{AtomicBool, AtomicUsize}, Arc}};

use crate::winapi::{main_recoil, AppEvent, AppState, Category, FullAutoStandardConfig, Game, GlobalConfig, Loadout, SingleFireConfig, Weapon};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    println!("Greeted from Rust: {}", name);
    format!("Hello, {}! You've been greeted from Rust!", name)
}
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
        state.current_category_index.store(0, std::sync::atomic::Ordering::Relaxed);
        state.current_loadout_index.store(0, std::sync::atomic::Ordering::Relaxed);
        state.current_weapon_index.store(0, std::sync::atomic::Ordering::Relaxed);
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

    println!("Set autofire for weapon {} to {}", weapon, enabled);
    Ok(())
}

async fn save_game_data (
    state: AppState
) -> Result<(), String> {
    let assets_dir_path = Path::new("..").join("assets");

    // Save config to `assets/config.json`
    let global_config: GlobalConfig = (*state.global_config).clone();
    tokio::fs::write(
        assets_dir_path.join("config.json"),
        serde_json::to_string_pretty(&global_config).map_err(|e| e.to_string())?
    ).await.map_err(|e| e.to_string())?;

    // Save each game's data to `assets/<game_name>/game.json`
    let games_dir_path = assets_dir_path.join("games");
    for game in state.games.read_arc().iter() {
        let game_path = games_dir_path.join(&game.name);
        
        tokio::fs::create_dir_all(&game_path).await.map_err(|e| e.to_string())?;
        tokio::fs::write(
            game_path.join("game.json"),
            serde_json::to_string_pretty(game).map_err(|e| e.to_string())?
        ).await.map_err(|e| e.to_string())?;
    }

    Ok(())
}

async fn setup() -> AppState {
    let games = Vec::from([
        Game {
            name: "Rainbow Six Siege".to_string(),
            categories: vec!(
                Category {
                    name: "Attackers".to_string(),
                    loadouts: vec!(
                        Loadout {
                            name: "Twitch".to_string(),
                            weapon_ids: vec!(String::from("R6_417"), String::from("R6_P12")),
                        },
                        Loadout {
                            name: "Ash".to_string(),
                            weapon_ids: vec!(String::from("R6_R4-C"), String::from("R6_417")),
                        },
                    ),
                },
                Category {
                    name: "Defenders".to_string(),
                    loadouts: vec!(
                        Loadout {
                            name: "Jäger".to_string(),
                            weapon_ids: vec!(String::from("R6_R4-C"), String::from("R6_P12")),
                        },
                        Loadout {
                            name: "Bandit".to_string(),
                            weapon_ids: vec!(String::from("R6_417"), String::from("R6_P12")),
                        },
                    ),
                },
            ),
            weapons: HashMap::from([
                (String::from("R6_R4-C"), Weapon::FullAutoStandard(FullAutoStandardConfig {
                    name: String::from("R4-C"),
                    rpm: 860,
                    first_shot_scale: 1.23,
                    exponential_factor: 1.007,
                    dx: -5.0,
                    dy: 129.5,
                    mag_size: 26,
                })),
                (String::from("R6_417"), Weapon::SingleFire(SingleFireConfig {
                    name: String::from("417"),
                    trigger_delay_ms: 90,
                    recoil_completion_ms: 10,
                    release_delay_ms: 25,
                    dx: 0.0,
                    dy: 46.5,
                    mag_size: 21,
                    autofire: true,
                })),
                (String::from("R6_P12"), Weapon::SingleFire(SingleFireConfig {
                    name: String::from("P12"),
                    trigger_delay_ms: 80,
                    recoil_completion_ms: 10,
                    release_delay_ms: 25,
                    dx: 0.5,
                    dy: 22.0,
                    mag_size: 17,
                    autofire: true,
                })),
            ])
        },
        Game {
            name: "Rust".to_string(),
            categories: vec!( ),
            weapons: HashMap::from([
                (String::from("Rust_AK47"), Weapon::FullAutoStandard(FullAutoStandardConfig {
                    name: String::from("AK-47"),
                    rpm: 600,
                    first_shot_scale: 1.15,
                    exponential_factor: 1.005,
                    dx: -3.0,
                    dy: 100.0,
                    mag_size: 30,
                })),
                (String::from("Rust_M4A4"), Weapon::SingleFire(SingleFireConfig {
                    name: String::from("M4A4"),
                    trigger_delay_ms: 70,
                    recoil_completion_ms: 8,
                    release_delay_ms: 20,
                    dx: 0.0,
                    dy: 40.0,
                    mag_size: 30,
                    autofire: true,
                })),
            ]),
        },
    ]);

    let global_config = GlobalConfig {
        require_right_hold: true,
    };
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

    save_game_data(state_cloned.clone())
        .await
        .expect("Failed to save game data");

    tokio::spawn(async move {
        main_recoil(state_cloned);
    });

    state
}
pub fn run() {
    Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
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