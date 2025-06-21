mod winapi;

use parking_lot::Mutex;
use tauri::{ipc::Channel, Builder, Manager};
use window_vibrancy::apply_acrylic;

use std::{collections::HashMap, sync::{atomic::{AtomicBool, AtomicUsize}, Arc}};

use crate::winapi::{main_recoil, AppEvent, AppState, Category, FullAutoStandardConfig, Game, GlobalConfig, Loadout, SingleFireConfig, Weapon};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    println!("Greeted from Rust: {}", name);
    format!("Hello, {}! You've been greeted from Rust!", name)
}
#[tauri::command]
fn get_games(state: tauri::State<'_, AppState>) -> Vec<Game> {
    (*state.games).clone()
}
#[tauri::command]
fn get_weapons(state: tauri::State<'_, AppState>) -> HashMap<String, Weapon> {
    (*state.weapons.lock()).clone()
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
async fn change_loadout (
    state: tauri::State<'_, AppState>,
    new_loadout_index: usize
) -> Result<usize, String> {
    let current_game_index = state.current_game_index.load(std::sync::atomic::Ordering::Relaxed);
    let current_category_index = state.current_category_index.load(std::sync::atomic::Ordering::Relaxed);

    if let Some(game) = state.games.get(current_game_index) {
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
    if let Some(weapon_config) = state.weapons.lock().get_mut(&weapon) {
        if let Weapon::SingleFire(config) = weapon_config {
            config.autofire = enabled;
            println!("Set autofire for {} to {}", weapon, enabled);
            return Ok(());
        }
    }
    Err(format!("Weapon {} not found or not a single-fire weapon", weapon))
}

async fn setup() -> AppState {
    let weapons = HashMap::from([
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
    ]);
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
            )
        },
    ]);

    let global_config = GlobalConfig {
        require_right_hold: true,
    };
    let (event_tx, event_rx) = std::sync::mpsc::channel();
    let state = AppState {
        games:         Arc::new(games),
        weapons:       Arc::new(Mutex::new(weapons)),
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
            greet,
            get_games,
            change_loadout,
            get_weapons,
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