use crate::{SERVER_BASE_URL, load_games, merge_game_configs, types::{AppEvent, AppState, Game, GlobalConfig, KeyStatus, KeyStatusResponse, LoadedGames}, winapi::get_hardware_identifier};

#[tauri::command]
pub fn get_games(state: tauri::State<'_, AppState>) -> Vec<Game> {
    state.games.read_arc().clone()
}
#[tauri::command]
pub fn get_config(state: tauri::State<'_, AppState>) -> GlobalConfig {
    state.global_config.read_arc().clone()
}
#[tauri::command]
pub fn get_version(
    app: tauri::AppHandle
) -> String {
    app.config().version
        .as_ref()
        .map(|st| st.clone())
        .unwrap_or(String::from("?.?.?"))
}
#[tauri::command]
pub async fn load_games_wrapper (
    state: tauri::State<'_, AppState>
) -> Result<(), String> {
    let LoadedGames {
        game_data,
    } = load_games((*state.config_dir_path).clone()).await?;

    *state.games.write_arc() = game_data;

    Ok(())
}

#[tauri::command]
pub async fn submit_game_key(
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
                
                // Use merge logic to preserve existing weapon configurations
                merge_game_configs(game, config);
                
                println!("Updated game '{}' with configuration from server (preserving local changes)", game_name);
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