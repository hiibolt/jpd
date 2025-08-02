mod winapi;
mod recoil;
mod types;
mod commands;

use parking_lot::{Mutex, RwLock};
use tauri::{App, Builder, Manager};
use anyhow::{anyhow, Result};

use std::{path::PathBuf, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, Arc}, collections::HashMap};

use crate::{
    commands::{
        config::*,
        selection::*,
        app::*,
        state::*
    }, 
    types::{KeyStatus, KeyStatusResponse, LoadedGames}, 
    winapi::{get_hardware_identifier, main_recoil}
};
use crate::types::{AppEvent, AppState, Game, GlobalConfig};

pub const SERVER_BASE_URL: &'static str = "http://45.146.252.244:4777";

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

    // Get the current weapon index (0 = primary, 1 = secondary)
    let weapon_ind = if current_loadout.secondaries.is_empty() {
        0
    } else {
        state.current_weapon_index.load(Ordering::SeqCst)
    };
    let weapon_id = if weapon_ind == 0 {
        // Primary weapon
        if current_loadout.primaries.is_empty() {
            return Err(anyhow!("No primary weapons in loadout `{}` in category `{}` in game `{}`", current_loadout.name, current_category.name, current_game.name));
        }
        current_loadout.primaries.get(current_loadout.selected_primary)
            .ok_or(anyhow!("Selected primary index `{}` not found in loadout `{}` in category `{}` in game `{}`", current_loadout.selected_primary, current_loadout.name, current_category.name, current_game.name))?
    } else {
        // Secondary weapon
        if current_loadout.secondaries.is_empty() {
            return Err(anyhow!("No secondary weapons in loadout `{}` in category `{}` in game `{}`", current_loadout.name, current_category.name, current_game.name));
        }
        current_loadout.secondaries.get(current_loadout.selected_secondary)
            .ok_or(anyhow!("Selected secondary index `{}` not found in loadout `{}` in category `{}` in game `{}`", current_loadout.selected_secondary, current_loadout.name, current_category.name, current_game.name))?
    };

    Ok(weapon_id.clone())
}
fn load_local_games(games_dir_path: &PathBuf) -> Result<HashMap<String, Game>, String> {
    use std::collections::HashMap;
    let mut local_games = HashMap::new();
    
    if !games_dir_path.exists() {
        println!("Games directory does not exist: {}", games_dir_path.display());
        return Ok(local_games);
    }
    
    // Read all subdirectories in the games directory
    let entries = std::fs::read_dir(games_dir_path)
        .map_err(|e| format!("Failed to read games directory: {}", e))?;
        
    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        
        if path.is_dir() {
            let game_name = path.file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| "Invalid game directory name".to_string())?
                .to_string();
                
            let data_file = path.join("data.json");
            if data_file.exists() {
                match std::fs::read_to_string(&data_file) {
                    Ok(content) => {
                        match serde_json::from_str::<Game>(&content) {
                            Ok(game) => {
                                println!("Loaded local game config: {}", game_name);
                                local_games.insert(game_name, game);
                            },
                            Err(e) => {
                                eprintln!("Failed to parse local game config for {}: {}", game_name, e);
                            }
                        }
                    },
                    Err(e) => {
                        eprintln!("Failed to read local game config for {}: {}", game_name, e);
                    }
                }
            }
        }
    }
    
    Ok(local_games)
}
pub fn merge_game_configs(local_game: &mut Game, remote_game: &Game) {
    // Only update if remote has valid data
    if let Some(remote_categories) = &remote_game.categories {
        if let Some(local_categories) = &mut local_game.categories {
            // Merge categories
            for remote_category in remote_categories {
                if let Some(local_category) = local_categories.iter_mut()
                    .find(|c| c.name == remote_category.name) {
                    // Merge loadouts within the category
                    for remote_loadout in &remote_category.loadouts {
                        if !local_category.loadouts.iter().any(|l| l.name == remote_loadout.name) {
                            println!("Adding new loadout '{}' to category '{}'", remote_loadout.name, remote_category.name);
                            local_category.loadouts.push(remote_loadout.clone());
                        }
                    }
                } else {
                    // Add new category entirely
                    println!("Adding new category '{}'", remote_category.name);
                    local_categories.push(remote_category.clone());
                }
            }
        } else {
            // No local categories, use remote ones
            local_game.categories = remote_game.categories.clone();
        }
    }
    
    if let Some(remote_weapons) = &remote_game.weapons {
        if let Some(local_weapons) = &mut local_game.weapons {
            // Add new weapons, but don't overwrite existing ones
            for (weapon_id, remote_weapon) in remote_weapons {
                if !local_weapons.contains_key(weapon_id) {
                    println!("Adding new weapon '{}'", weapon_id);
                    local_weapons.insert(weapon_id.clone(), remote_weapon.clone());
                }
            }
        } else {
            // No local weapons, use remote ones
            local_game.weapons = remote_game.weapons.clone();
        }
    }
}
async fn load_games (
    config_dir: PathBuf
) -> Result<LoadedGames, String> {
    let mut games_ret = Vec::new();
    
    // First, load any existing local games from the config directory
    let games_dir_path = config_dir.join("games");
    let mut local_games = load_local_games(&games_dir_path)?;
    
    // Then fetch the remote games list to check for updates
    let game_list = match reqwest::get(format!("{SERVER_BASE_URL}/v1/products/jpd")).await {
        Ok(response) => {
            match response.json::<Vec<String>>().await {
                Ok(list) => {
                    println!("Available remote games configs: {:?}", list);
                    list
                },
                Err(e) => {
                    eprintln!("Failed to parse remote games list, using basic game info only: {}", e);
                    // If we can't get remote data, only provide basic game info (no local configs)
                    let basic_games: Vec<Game> = local_games.keys().map(|name| Game {
                        name: name.clone(),
                        key: None,
                        key_status: None,
                        categories: None,
                        weapons: None,
                    }).collect();
                    return Ok(LoadedGames { game_data: basic_games });
                }
            }
        },
        Err(e) => {
            eprintln!("Failed to fetch remote games list, using basic game info only: {}", e);
            // If we can't connect, only provide basic game info (no local configs)
            let basic_games: Vec<Game> = local_games.keys().map(|name| Game {
                name: name.clone(),
                key: None,
                key_status: None,
                categories: None,
                weapons: None,
            }).collect();
            return Ok(LoadedGames { game_data: basic_games });
        }
    };

    for game_id in game_list.clone() {
        // Check if we have a local version of this game
        let has_local_game = local_games.contains_key(&game_id);
        
        // Attempt to load the contents of `{config_dir}/{game}.key`
        let game_key_path = config_dir.join(format!("{}.key", game_id));
        let key = if game_key_path.exists() {
            match std::fs::read_to_string(&game_key_path) {
                Ok(key) => {
                    println!("Loaded key for game `{}` from file: {}", game_id, game_key_path.display());
                    key
                },
                Err(e) => {
                    eprintln!("Failed to read key file for game `{}`: {}", game_id, e);
                    // No key file readable - only provide basic game info
                    games_ret.push(Game {
                        name: game_id.clone(),
                        key: None,
                        key_status: None,
                        categories: None,
                        weapons: None,
                    });
                    continue;
                }
            }
        } else {
            println!("No key file found for game `{}`", game_id);
            // No key file - only provide basic game info
            games_ret.push(Game {
                name: game_id.clone(),
                key: None,
                key_status: None,
                categories: None,
                weapons: None,
            });
            continue;
        };

        let hardware_id = get_hardware_identifier();
        // Simple URL encoding for the hardware identifier - replace problematic characters
        let encoded_hardware_id = hardware_id
            .replace(":", "%3A")
            .replace(" ", "%20")
            .replace("&", "%26")
            .replace("=", "%3D")
            .replace("?", "%3F")
            .replace("#", "%23");
        let url = format!(
            "{SERVER_BASE_URL}/v1/validate/jpd/{}/{}/{}",
            game_id, key, encoded_hardware_id
        );

        let key_response_st = reqwest::get(url).await
            .map_err(|e| format!("Failed to validate key for game `{}`: {}", game_id, e))?
            .text().await
            .map_err(|e| format!("Failed to read key response for game `{}`: {}", game_id, e))?;
        let key_response: KeyStatusResponse = serde_json::from_str(&key_response_st)
            .map_err(|e| format!("Failed to convert key response for game `{}`: {}\n\n{}", game_id, e, key_response_st))?;

        match &key_response {
            KeyStatusResponse::Valid { timestamp, config, .. } => {
                println!("Key for game `{}` is valid, processing config", game_id);
                
                if has_local_game {
                    // Merge remote config with local config
                    let mut local_game = local_games.remove(&game_id).unwrap();
                    merge_game_configs(&mut local_game, config);
                    local_game.key = Some(key.clone());
                    local_game.key_status = Some(KeyStatus::Valid { 
                        key: key,
                        timestamp: *timestamp 
                    });
                    games_ret.push(local_game);
                } else {
                    // Use remote config directly
                    let mut full_game = config.clone();
                    full_game.key = Some(key.clone());
                    full_game.key_status = Some(KeyStatus::Valid { 
                        key: key,
                        timestamp: *timestamp 
                    });
                    games_ret.push(full_game);
                }
            },
            KeyStatusResponse::HWIDMismatch { key } => {
                println!("Key for game `{}` has HWID mismatch", game_id);
                // Remove from local_games to prevent duplicate in final loop
                local_games.remove(&game_id);
                // HWID mismatch - only provide basic game info (no local config)
                games_ret.push(Game {
                    name: game_id.clone(),
                    key: Some(key.clone()),
                    key_status: Some(KeyStatus::HWIDMismatch { key: key.clone() }),
                    categories: None,
                    weapons: None,
                });
            },
            KeyStatusResponse::Invalid { key } => {
                println!("Key for game `{}` is invalid", game_id);
                // Remove from local_games to prevent duplicate in final loop
                local_games.remove(&game_id);
                // Invalid key - only provide basic game info (no local config)
                games_ret.push(Game {
                    name: game_id.clone(),
                    key: Some(key.clone()),
                    key_status: Some(KeyStatus::Invalid { key: key.clone() }),
                    categories: None,
                    weapons: None,
                });
            },
            KeyStatusResponse::Expired { key, timestamp } => {
                println!("Key for game `{}` is expired", game_id);
                // Remove from local_games to prevent duplicate in final loop
                local_games.remove(&game_id);
                // Expired key - only provide basic game info (no local config)
                games_ret.push(Game {
                    name: game_id.clone(),
                    key: Some(key.clone()),
                    key_status: Some(KeyStatus::Expired { 
                        key: key.clone(), 
                        timestamp: *timestamp 
                    }),
                    categories: None,
                    weapons: None,
                });
            },
            KeyStatusResponse::Banned { key } => {
                println!("Key for game `{}` is banned", game_id);
                // Remove from local_games to prevent duplicate in final loop
                local_games.remove(&game_id);
                // Banned key - only provide basic game info (no local config)
                games_ret.push(Game {
                    name: game_id.clone(),
                    key: Some(key.clone()),
                    key_status: Some(KeyStatus::Banned { key: key.clone() }),
                    categories: None,
                    weapons: None,
                });
            },
        }
    }
    
    // Add any remaining local games that weren't in the remote list (basic info only)
    for (game_name, local_game) in local_games {
        println!("Adding local-only game: {} (basic info only)", game_name);
        games_ret.push(Game {
            name: game_name,
            key: local_game.key,
            key_status: local_game.key_status,
            categories: None,  // Never load local configs without server validation
            weapons: None,     // Never load local configs without server validation
        });
    }

    println!("Loaded {} games", games_ret.len());

    Ok(LoadedGames {
        game_data: games_ret,
    })
}
fn load_config (
    config_dir: &PathBuf
) -> Result<GlobalConfig, String> {
    let config_path = config_dir.join("config.json");

    // If the `{config_dir}/config.json` file does not exist, create it with default values
    if !config_path.exists() {
        let default_config = GlobalConfig::default();
        std::fs::write(
            &config_path,
            serde_json::to_string_pretty(&default_config).map_err(|e| format!("Failed to serialize default config: {}", e))?
        ).map_err(|e| format!("Failed to write default config file: {}", e))?;
    }

    // Load global config from `{config_dir}/config.json`
    println!("Loading global config from: {}", config_path.display());
    let global_config: GlobalConfig = serde_json::from_str(
        &std::fs::read_to_string(config_path).map_err(|e| format!("Failed to read config file: {}", e))?
    ).map_err(|e| format!("Failed to parse global config: {}", e))?;

    Ok(global_config)
}
fn save_data(
    state: &AppState
) -> Result<(), String> {
    let config_dir = &state.config_dir_path;

    // Save config to `{config_dir}/config.json`
    let global_config: GlobalConfig = state.global_config.read_arc().clone();
    std::fs::write(
        config_dir.join("config.json"),
        serde_json::to_string_pretty(&global_config).map_err(|e| e.to_string())?
    ).map_err(|e| format!("Failed to write config file: {}", e))?;

    // Save each game's data to `{config_dir}/<game_name>/data.json`
    let games_dir_path = config_dir.join("games");
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
    // Note: We no longer automatically check for updates here
    // Updates will be checked manually via the check_for_updates command

    // Get the config directory path
    let config_dir_path = Arc::new(app.path().app_config_dir()
        .expect("Failed to get resource directory"));

    // If the config directory path does not exist, create it
    if !config_dir_path.exists() {
        std::fs::create_dir_all(&*config_dir_path)
            .expect("Failed to create config directory");
    }

    let config = match load_config(&config_dir_path) {
        Ok(data) => data,
        Err(e) => {
            // If loading data fails, create an `{config_dir}/logs` directory and log the error
            let log_dir = config_dir_path.join("logs");
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
        config_dir_path,

        events_channel_sender:   Arc::new(event_tx),
        events_channel_reciever: Arc::new(Mutex::new(event_rx)),

        left_hold_active:       Arc::new(AtomicBool::new(false)),
        right_hold_active:      Arc::new(AtomicBool::new(false)),
        current_game_index:     Arc::new(AtomicUsize::new(0)),
        current_category_index: Arc::new(AtomicUsize::new(0)),
        current_loadout_index:  Arc::new(AtomicUsize::new(0)),
        current_weapon_index:   Arc::new(AtomicUsize::new(0)),
        
        grid_layout_info:       Arc::new(RwLock::new(Default::default())),
        last_shot_times:        Arc::new(RwLock::new(HashMap::new())),
    };
    let state_cloned = state.clone();

    tokio::spawn(async move {
        main_recoil(state_cloned);
    });

    state
}
pub fn run() {
    Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_games,
            get_config,
            get_version,
            load_games_wrapper,
            submit_game_key,

            check_for_updates,
            perform_update,
            exit_app,
            restart_app,
            start_channel_reads,

            change_game,
            change_category,
            change_loadout,
            change_primary_weapon,
            change_secondary_weapon,
            
            change_horizontal_multiplier,
            change_vertical_multiplier,
            change_acog_horizontal_multiplier,
            change_acog_vertical_multiplier,
            change_scroll_wheel_weapon_swap,
            change_setting,
            change_weapon_config,
            reset_config_from_server,
            change_grid_layout

        ])
        .setup(|app| {
            let state = tauri::async_runtime::block_on(setup(app));

            // Register the application state
            app.manage(state);

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Function for cycling through weapon types (used by scroll wheel)
pub fn cycle_weapon_type(state: &AppState, scroll_up: bool) -> Result<usize, String> {
    let current_game_index = state.current_game_index.load(std::sync::atomic::Ordering::Relaxed);
    let current_category_index = state.current_category_index.load(std::sync::atomic::Ordering::Relaxed);
    let current_loadout_index = state.current_loadout_index.load(std::sync::atomic::Ordering::Relaxed);
    let current_weapon_index = state.current_weapon_index.load(std::sync::atomic::Ordering::Relaxed);

    if let Some(game) = state.games.read_arc().get(current_game_index) {
        if let Some(category) = game.categories
            .as_ref().ok_or("Game does not have data loaded.")?
            .get(current_category_index)
        {
            if let Some(loadout) = category.loadouts.get(current_loadout_index) {
                // Check if both primary and secondary weapons are available
                let has_primary = !loadout.primaries.is_empty();
                let has_secondary = !loadout.secondaries.is_empty();
                
                if !has_primary && !has_secondary {
                    return Err("No weapons available in current loadout".to_string());
                }
                
                let next_weapon_index = if has_primary && has_secondary {
                    // Both weapon types available, cycle between them
                    if scroll_up {
                        (current_weapon_index + 1) % 2
                    } else {
                        if current_weapon_index == 0 { 1 } else { 0 }
                    }
                } else if has_primary {
                    // Only primary available
                    0
                } else {
                    // Only secondary available  
                    1
                };
                
                // Only update if there's actually a change
                if next_weapon_index != current_weapon_index {
                    state.current_weapon_index.store(next_weapon_index, std::sync::atomic::Ordering::Relaxed);
                    
                    // Clear shot timing for trigger cap when switching weapons
                    crate::recoil::clear_current_weapon_timing(state);
                    
                    println!("Cycled to weapon index {} (scroll {})", next_weapon_index, if scroll_up { "up" } else { "down" });
                    
                    // Send event to update the frontend
                    if let Err(e) = state.events_channel_sender.send(AppEvent::SwitchedWeapon {
                        weapon_ind: next_weapon_index,
                    }) {
                        eprintln!("Failed to send SwitchedWeapon event: {}", e);
                    }
                }
                
                return Ok(next_weapon_index);
            }
        }
    }

    Err("Invalid game, category, or loadout state".to_string())
}

// Function for cycling through categories (used by INSERT key)
fn cycle_category(state: &AppState) -> Result<usize, String> {
    let current_game_index = state.current_game_index.load(std::sync::atomic::Ordering::Relaxed);
    let current_category_index = state.current_category_index.load(std::sync::atomic::Ordering::Relaxed);

    if let Some(game) = state.games.read_arc().get(current_game_index) {
        let category_count = game.categories.as_ref().map(|c| c.len()).unwrap_or(0);
        
        if category_count == 0 {
            return Err("No categories available".to_string());
        }
        
        // Cycle to next category, wrapping around to 0 if at the end
        let next_category_index = (current_category_index + 1) % category_count;
        
        state.current_category_index.store(next_category_index, std::sync::atomic::Ordering::Relaxed);
        state.current_loadout_index.store(0, std::sync::atomic::Ordering::Relaxed); // Reset loadout to first
        
        println!("Cycled to category index {} (from {})", next_category_index, current_category_index);
        
        // Send events to update the frontend
        if let Err(e) = state.events_channel_sender.send(AppEvent::SwitchedCategory {
            category_ind: next_category_index,
        }) {
            eprintln!("Failed to send SwitchedCategory event: {}", e);
        }
        
        if let Err(e) = state.events_channel_sender.send(AppEvent::SwitchedLoadout {
            loadout_ind: 0,
        }) {
            eprintln!("Failed to send SwitchedLoadout event: {}", e);
        }
        
        return Ok(next_category_index);
    }

    Err("No game selected".to_string())
}