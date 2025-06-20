mod winapi;

use tauri::{ipc::Channel, Builder, Manager};
use window_vibrancy::apply_acrylic;

use std::{collections::HashMap, sync::{atomic::{AtomicBool, AtomicUsize}, Arc}};

use crate::winapi::{main_recoil, AppEvent, AppState, FullAutoStandardConfig, GlobalConfig, Loadout, SingleFireConfig, Weapon};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    println!("Greeted from Rust: {}", name);
    format!("Hello, {}! You've been greeted from Rust!", name)
}
#[tauri::command]
fn get_loadouts(state: tauri::State<'_, AppState>) -> Vec<Loadout> {
    (*state.loadouts).clone()
}
#[tauri::command]
fn get_weapons(state: tauri::State<'_, AppState>) -> HashMap<String, Weapon> {
    (*state.weapons).clone()
}
#[tauri::command]
async fn start_channel_reads (
    state: tauri::State<'_, AppState>,
    channel: Channel<AppEvent>,
) -> Result<String, String> {
    while let Ok(event) = state.events_channel_reciever.lock().await.recv() {
        channel.send(event).expect("Failed to send event");
    }

    Err(String::from("Channel reads closed early?"))
}
#[tauri::command]
async fn change_loadout (
    state: tauri::State<'_, AppState>,
    new_loadout_index: usize
) -> Result<usize, String> {
    if new_loadout_index >= state.loadouts.len() {
        return Err(format!("Invalid loadout index: {}", new_loadout_index));
    }

    state.current_loadout_index.store(new_loadout_index, std::sync::atomic::Ordering::SeqCst);
    state.current_weapon_index.store(0, std::sync::atomic::Ordering::SeqCst);
    
    let current_loadout = &state.loadouts[new_loadout_index];
    println!("Changed to loadout: {}", current_loadout.name);
    Ok(new_loadout_index)
}

async fn setup() -> AppState {
    let weapons = HashMap::from([
        (String::from("R4-C"), Weapon::FullAutoStandard(FullAutoStandardConfig {
            rpm: 860,
            first_shot_scale: 1.23,
            exponential_factor: 1.007,
            dx: -5.0,
            dy: 129.5,
            mag_size: 26,
        })),
        (String::from("417"), Weapon::SingleFire(SingleFireConfig {
            trigger_delay_ms: 90,
            recoil_completion_ms: 10,
            release_delay_ms: 25,
            dx: 0.0,
            dy: 46.5,
            mag_size: 21,
            autofire: true,
        })),
        (String::from("P12"), Weapon::SingleFire(SingleFireConfig {
            trigger_delay_ms: 80,
            recoil_completion_ms: 10,
            release_delay_ms: 25,
            dx: 0.5,
            dy: 22.0,
            mag_size: 17,
            autofire: true,
        })),
    ]);
    let loadouts = Vec::from([
        Loadout {
            name: "Twitch".to_string(),
            weapon_ids: vec!(String::from("417"), String::from("P12")),
        },
        Loadout {
            name: "Ash".to_string(),
            weapon_ids: vec!(String::from("R4-C"), String::from("417")),
        },
        Loadout {
            name: "Default".to_string(),
            weapon_ids: vec!(String::from("R4-C"), String::from("417")),
        },
    ]);
    let global_config = GlobalConfig {
        require_right_hold: true,
    };
    let (event_tx, event_rx) = std::sync::mpsc::channel();
    let state = AppState {
        weapons: Arc::new(weapons),
        loadouts: Arc::new(loadouts),
        global_config: Arc::new(global_config),

        events_channel_sender: Arc::new(event_tx),
        events_channel_reciever: Arc::new(tokio::sync::Mutex::new(event_rx)),

        left_hold_active: Arc::new(AtomicBool::new(false)),
        right_hold_active: Arc::new(AtomicBool::new(false)),
        current_loadout_index: Arc::new(AtomicUsize::new(0)),
        current_weapon_index: Arc::new(AtomicUsize::new(0)),
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
            get_loadouts,
            change_loadout,
            get_weapons,
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