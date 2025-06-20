mod winapi;

use tauri::{Builder, Manager};
use std::{collections::HashMap, sync::{atomic::{AtomicBool, AtomicUsize}, Arc}};

use crate::winapi::{main_recoil, AppState, FullAutoStandardConfig, GlobalConfig, Loadout, SingleFireConfig, Weapon};

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    println!("Greeted from Rust: {}", name);
    format!("Hello, {}! You've been greeted from Rust!", name)
}
#[tauri::command]
fn get_loadout(state: tauri::State<'_, AppState>) -> Loadout {
    (*state.loadout).clone()
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
    let loadout = Loadout {
        name: "Twitch".to_string(),
        weapon_ids: vec!(String::from("417"), String::from("P12")),
    };
    let global_config = GlobalConfig {
        require_right_hold: true,
    };
    let state = AppState {
        weapons: Arc::new(weapons),
        global_config: Arc::new(global_config),

        left_hold_active: Arc::new(AtomicBool::new(false)),
        right_hold_active: Arc::new(AtomicBool::new(false)),
        loadout: Arc::new(loadout),
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
        .invoke_handler(tauri::generate_handler![greet, get_loadout])
        .setup(|app| {
            let state = tauri::async_runtime::block_on(setup());

            app.manage(state);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}