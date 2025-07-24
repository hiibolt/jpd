use crate::{save_data, types::{AppEvent, AppState, Game}};

#[tauri::command]
pub async fn change_game (
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
pub async fn change_category (
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
pub async fn change_loadout (
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
                
                // Emit event for loadout change
                let _ = state.events_channel_sender.send(AppEvent::SwitchedLoadout {
                    loadout_ind: new_loadout_index,
                });
                
                return Ok(new_loadout_index);
            }
        }
    }

    Err(format!("Invalid loadout index: {}", new_loadout_index))
}
#[tauri::command]
pub async fn change_primary_weapon (
    state: tauri::State<'_, AppState>,
    new_primary_index: usize
) -> Result<Vec<Game>, String> {
    let current_game_index = state.current_game_index.load(std::sync::atomic::Ordering::Relaxed);
    let current_category_index = state.current_category_index.load(std::sync::atomic::Ordering::Relaxed);
    let current_loadout_index = state.current_loadout_index.load(std::sync::atomic::Ordering::Relaxed);

    let mut games = state.games.write_arc();
    if let Some(game) = games.get_mut(current_game_index) {
        if let Some(category) = game.categories
            .as_mut().ok_or("Game does not have data loaded.")?
            .get_mut(current_category_index)
        {
            if let Some(loadout) = category.loadouts.get_mut(current_loadout_index) {
                if new_primary_index < loadout.primaries.len() {
                    loadout.selected_primary = new_primary_index;
                    println!("Changed primary weapon to index {}", new_primary_index);
                    
                    // Save the updated data
                    drop(games); // Release the lock before calling save_data
                    save_data(&state).map_err(|e| format!("Failed to save data: {}", e))?;
                    
                    return Ok(state.games.read_arc().clone());
                } else {
                    return Err(format!("Primary weapon index {} out of bounds", new_primary_index));
                }
            }
        }
    }
    
    Err("Failed to change primary weapon".to_string())
}
#[tauri::command]
pub async fn change_secondary_weapon (
    state: tauri::State<'_, AppState>,
    new_secondary_index: usize
) -> Result<Vec<Game>, String> {
    let current_game_index = state.current_game_index.load(std::sync::atomic::Ordering::Relaxed);
    let current_category_index = state.current_category_index.load(std::sync::atomic::Ordering::Relaxed);
    let current_loadout_index = state.current_loadout_index.load(std::sync::atomic::Ordering::Relaxed);

    let mut games = state.games.write_arc();
    if let Some(game) = games.get_mut(current_game_index) {
        if let Some(category) = game.categories
            .as_mut().ok_or("Game does not have data loaded.")?
            .get_mut(current_category_index)
        {
            if let Some(loadout) = category.loadouts.get_mut(current_loadout_index) {
                if new_secondary_index < loadout.secondaries.len() {
                    loadout.selected_secondary = new_secondary_index;
                    println!("Changed secondary weapon to index {}", new_secondary_index);
                    
                    // Save the updated data
                    drop(games); // Release the lock before calling save_data
                    save_data(&state).map_err(|e| format!("Failed to save data: {}", e))?;
                    
                    return Ok(state.games.read_arc().clone());
                } else {
                    return Err(format!("Secondary weapon index {} out of bounds", new_secondary_index));
                }
            }
        }
    }
    
    Err("Failed to change secondary weapon".to_string())
}