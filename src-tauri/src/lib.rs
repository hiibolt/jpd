mod winapi;
mod recoil;
mod types;

use parking_lot::{Mutex, RwLock};
use tauri::{ipc::Channel, App, Builder, Manager};
use tauri_plugin_updater::UpdaterExt;
use window_vibrancy::apply_acrylic;
use anyhow::{anyhow, Result};

use std::{path::PathBuf, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, Arc}, collections::HashMap};

use crate::{types::{KeyStatus, KeyStatusResponse, LoadedGames}, winapi::{main_recoil, get_hardware_identifier}};
use crate::types::{AppEvent, AppState, Game, GlobalConfig, Weapon};

const SERVER_BASE_URL: &'static str = "http://5.249.162.64:4777";

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
fn get_version(
    app: tauri::AppHandle
) -> String {
    app.config().version
        .as_ref()
        .map(|st| st.clone())
        .unwrap_or(String::from("?.?.?"))
}
#[tauri::command]
async fn restart_app(
    app: tauri::AppHandle
) -> Result<(), String> {
    app.restart();
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
async fn change_acog_horizontal_multiplier (
    state: tauri::State<'_, AppState>,
    new_multiplier: f32
) -> Result<GlobalConfig, String> {
    if new_multiplier > 0.0 {
        state.global_config.write_arc().mouse_config.acog_horizontal_multiplier = new_multiplier;
        save_data(&state).map_err(|e| format!("Failed to save game data: {}", e))?;
        println!("Changed ACOG horizontal multiplier to {}", new_multiplier);
        
        return Ok(state.global_config.read_arc().clone());
    }

    Err(format!("Invalid ACOG horizontal multiplier: {}", new_multiplier))
}
#[tauri::command]
async fn change_acog_vertical_multiplier (
    state: tauri::State<'_, AppState>,
    new_multiplier: f32
) -> Result<GlobalConfig, String> {
    if new_multiplier > 0.0 {
        state.global_config.write_arc().mouse_config.acog_vertical_multiplier = new_multiplier;
        save_data(&state).map_err(|e| format!("Failed to save game data: {}", e))?;
        println!("Changed ACOG vertical multiplier to {}", new_multiplier);

        return Ok(state.global_config.read_arc().clone());
    }

    Err(format!("Invalid ACOG vertical multiplier: {}", new_multiplier))
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
                "autofire" => weapon_config.autofire = new_value.as_bool().ok_or("Invalid value for autofire")?,
                "enabled" => weapon_config.enabled = new_value.as_bool().ok_or("Invalid value for enabled")?,
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
                "enabled" => weapon_config.enabled = new_value.as_bool().ok_or("Invalid value for enabled")?,
                _ => return Err(format!("Unknown field: {}", field)),
            }
        },
        Weapon::SingleShot(weapon_config) => {
            match field.as_str() {
                "name" => weapon_config.name = new_value.as_str().ok_or("Invalid value for name")?.to_string(),
                "description" => weapon_config.description = new_value.as_str().map(|s| s.to_string()),
                "recoil_completion_ms" => weapon_config.recoil_completion_ms = new_value.as_u64().ok_or("Invalid value for recoil_completion_ms")? as u32,
                "dx" => weapon_config.dx = new_value.as_f64().ok_or("Invalid value for dx")? as f32,
                "dy" => weapon_config.dy = new_value.as_f64().ok_or("Invalid value for dy")? as f32,
                "enabled" => weapon_config.enabled = new_value.as_bool().ok_or("Invalid value for enabled")?,

                _ => return Err(format!("Unknown field: {}", field)),
            }
        },
        Weapon::None(weapon_config) => {
            match field.as_str() {
                "name" => weapon_config.name = new_value.as_str().ok_or("Invalid value for name")?.to_string(),
                "description" => weapon_config.description = new_value.as_str().map(|s| s.to_string()),
                "enabled" => weapon_config.enabled = new_value.as_bool().ok_or("Invalid value for enabled")?,
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
    } = load_games((*state.config_dir_path).clone()).await?;

    *state.games.write_arc() = game_data;

    Ok(())
}

#[tauri::command]
async fn submit_game_key(
    state: tauri::State<'_, AppState>,
    game_name: String,
    key: String,
) -> Result<Vec<Game>, String> {
    let config_dir = (*state.config_dir_path).clone();
    
    // Write the key to the appropriate .key file
    let key_file_path = config_dir.join(format!("{}.key", game_name));
    std::fs::write(&key_file_path, &key)
        .map_err(|e| format!("Failed to write key file {}: {}", key_file_path.display(), e))?;
    
    println!("Updated key file for game '{}' at: {}", game_name, key_file_path.display());
    
    // Validate the key with the server and get the full response
    let hardware_id = get_hardware_identifier();
    // Simple URL encoding for the hardware identifier - replace problematic characters
    let encoded_hardware_id = hardware_id
        .replace(":", "%3A")
        .replace(" ", "%20")
        .replace("&", "%26")
        .replace("=", "%3D")
        .replace("?", "%3F")
        .replace("#", "%23");
    let url = format!("{SERVER_BASE_URL}/v1/validate/jpd/{}/{}/{}", 
        game_name, 
        key, 
        encoded_hardware_id
    );
    
    let key_response = reqwest::get(url)
        .await
        .map_err(|e| format!("Failed to validate key: {}", e))?
        .text()
        .await
        .map_err(|e| format!("Failed to read key response: {}", e))?;
    let key_response: KeyStatusResponse = serde_json::from_str(&key_response)
        .map_err(|e| format!("Failed to read key response: {}\n\nResponse: {}", e, key_response))?;
    
    // Update the game with the new key and status
    let mut games = state.games.write_arc();
    if let Some(game) = games.iter_mut().find(|g| g.name == game_name) {
        game.key = Some(key);
        
        // Extract the key status and update the game accordingly
        match &key_response {
            KeyStatusResponse::Valid { key: _, timestamp, config } => {
                // For valid keys, update with the full game configuration
                game.key_status = Some(KeyStatus::Valid { 
                    key: config.key.clone().unwrap_or_default(),
                    timestamp: *timestamp 
                });
                game.categories = config.categories.clone();
                game.weapons = config.weapons.clone();
                println!("Updated game '{}' with full configuration from server", game_name);
            },
            KeyStatusResponse::Invalid { key } => {
                game.key_status = Some(KeyStatus::Invalid { key: key.clone() });
                game.categories = None;  // Clear categories for invalid keys
                game.weapons = None;     // Clear weapons for invalid keys
                println!("Key for game '{}' is invalid", game_name);
            },
            KeyStatusResponse::Expired { key, timestamp } => {
                game.key_status = Some(KeyStatus::Expired { 
                    key: key.clone(), 
                    timestamp: *timestamp 
                });
                game.categories = None;  // Clear categories for expired keys
                game.weapons = None;     // Clear weapons for expired keys
                println!("Key for game '{}' is expired", game_name);
            },
            KeyStatusResponse::Banned { key } => {
                game.key_status = Some(KeyStatus::Banned { key: key.clone() });
                game.categories = None;  // Clear categories for banned keys
                game.weapons = None;     // Clear weapons for banned keys
                println!("Key for game '{}' is banned", game_name);
            },
            KeyStatusResponse::HWIDMismatch { key } => {
                game.key_status = Some(KeyStatus::HWIDMismatch { key: key.clone() });
                game.categories = None;  // Clear categories for HWID mismatch
                game.weapons = None;     // Clear weapons for HWID mismatch
                println!("Key for game '{}' has HWID mismatch", game_name);
            },
        }
    }
    
    let updated_games = games.clone();
    
    // Notify frontend of updated games
    let _ = state.events_channel_sender.send(AppEvent::UpdatedGames {
        games: updated_games.clone(),
    });
    
    Ok(updated_games)
}
#[tauri::command]
async fn reset_config_from_server(
    state: tauri::State<'_, AppState>
) -> Result<Vec<Game>, String> {
    let config_dir = (*state.config_dir_path).clone();
    let games_dir_path = config_dir.join("games");
    
    // First, completely remove the games directory to clear all local data
    if games_dir_path.exists() {
        std::fs::remove_dir_all(&games_dir_path)
            .map_err(|e| format!("Failed to remove games directory: {}", e))?;
        println!("Cleared all local game data from: {}", games_dir_path.display());
    }
    
    // Now reload fresh data from the server (this will only use server data, no local merging)
    let LoadedGames { game_data } = load_games(config_dir).await?;
    
    // Update the application state with the fresh data
    *state.games.write_arc() = game_data.clone();
    
    // Reset current indices to 0 since the game list may have changed
    state.current_game_index.store(0, std::sync::atomic::Ordering::Relaxed);
    state.current_category_index.store(0, std::sync::atomic::Ordering::Relaxed);
    state.current_loadout_index.store(0, std::sync::atomic::Ordering::Relaxed);
    state.current_weapon_index.store(0, std::sync::atomic::Ordering::Relaxed);
    
    println!("Successfully reset all game configurations from server. Loaded {} games.", game_data.len());
    
    Ok(game_data)
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

fn merge_game_configs(local_game: &mut Game, remote_game: &Game) {
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

    for game_id in &game_list {
        // Check if we have a local version of this game
        let has_local_game = local_games.contains_key(game_id);
        
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

        let key_response = match reqwest::get(url).await {
            Ok(response) => {
                match response.text().await {
                    Ok(text) => text,
                    Err(e) => {
                        eprintln!("Failed to read key response for game `{}`: {}", game_id, e);
                        // Network error - only provide basic game info (no fallback to local config)
                        games_ret.push(Game {
                            name: game_id.clone(),
                            key: Some(key),
                            key_status: None,
                            categories: None,
                            weapons: None,
                        });
                        continue;
                    }
                }
            },
            Err(e) => {
                eprintln!("Failed to validate key for game `{}`: {}", game_id, e);
                // Network error - only provide basic game info (no fallback to local config)
                games_ret.push(Game {
                    name: game_id.clone(),
                    key: Some(key),
                    key_status: None,
                    categories: None,
                    weapons: None,
                });
                continue;
            }
        };
        
        let key_response: KeyStatusResponse = match serde_json::from_str(&key_response) {
            Ok(response) => response,
            Err(e) => {
                eprintln!("Failed to convert key response for game `{}`: {}\n\n{}", game_id, e, key_response);
                // Parse error - only provide basic game info (no fallback to local config)
                games_ret.push(Game {
                    name: game_id.clone(),
                    key: Some(key),
                    key_status: None,
                    categories: None,
                    weapons: None,
                });
                continue;
            }
        };

        match &key_response {
            KeyStatusResponse::Valid { timestamp, config, .. } => {
                println!("Key for game `{}` is valid, processing config", game_id);
                
                if has_local_game {
                    // Merge remote config with local config
                    let mut local_game = local_games.remove(game_id).unwrap();
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

async fn update(app: tauri::AppHandle) -> tauri_plugin_updater::Result<()> {
    if let Some(update) = app.updater()?.check().await? {
        let mut downloaded = 0;

        // alternatively we could also call update.download() and update.install() separately
        update.download_and_install(
                |chunk_length, content_length| {
                    downloaded += chunk_length;
                    println!("downloaded {downloaded} from {content_length:?}");
                },
                || {
                println!("download finished");
                },
            )
            .await?;

        println!("update installed");
        app.restart();
    }

  Ok(())
}
async fn setup(
    app: &mut App
) -> AppState {
    // First, check for updates
    let app_handle_cloned = app.handle().clone();
    tauri::async_runtime::spawn(async move {
        update(app_handle_cloned).await.expect("Failed to run updater!");
    });

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
        .invoke_handler(tauri::generate_handler![
            get_games,
            get_config,
            get_version,
            restart_app,

            change_game,
            change_category,
            change_loadout,
            change_horizontal_multiplier,
            change_vertical_multiplier,
            change_acog_horizontal_multiplier,
            change_acog_vertical_multiplier,
            change_setting,

            load_games_wrapper,
            set_weapon_config,
            start_channel_reads,
            submit_game_key,
            reset_config_from_server
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