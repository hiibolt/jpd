use crate::{load_games, save_data, types::{AppState, Game, GlobalConfig, LoadedGames, Weapon}};

#[tauri::command]
pub fn change_grid_layout(
    state: tauri::State<'_, AppState>,
    loadouts_per_row: usize
) -> Result<(), String> {
    let mut grid_layout = state.grid_layout_info.write_arc();
    grid_layout.loadouts_per_row = loadouts_per_row;
    
    println!("Updated grid layout: {} loadouts per row", loadouts_per_row);
    
    Ok(())
}
#[tauri::command]
pub async fn reset_config_from_server(
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
#[tauri::command]
pub fn change_weapon_config(
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
        }
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
pub async fn change_setting (
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
pub async fn change_horizontal_multiplier (
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
pub async fn change_vertical_multiplier (
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
pub async fn change_acog_horizontal_multiplier (
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
pub async fn change_acog_vertical_multiplier (
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
pub async fn change_scroll_wheel_weapon_swap (
    state: tauri::State<'_, AppState>,
    enabled: bool
) -> Result<GlobalConfig, String> {
    state.global_config.write_arc().mouse_config.scroll_wheel_weapon_swap = enabled;
    save_data(&state).map_err(|e| format!("Failed to save game data: {}", e))?;
    println!("Changed scroll wheel weapon swap to {}", enabled);

    Ok(state.global_config.read_arc().clone())
}