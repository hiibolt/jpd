extern crate winapi;
use std::{mem, thread, time::Duration};
use std::sync::atomic::Ordering;
use winapi::um::winuser::*;

use crate::get_weapon_id;
use crate::types::{AppEvent, AppState, GlobalConfig, Weapon};
use crate::winapi::{press_key, release_key};

pub fn move_down (
    config: &GlobalConfig,
    weapon: &Weapon,
    dx_total: f32,
    dy_total: f32,
    splits: u32,
    total_interval: Duration,
    wait_first: bool,
) {
    // Check if weapon has "ACOG" in description to determine which multiplier to use
    let has_acog = match weapon {
        Weapon::SingleFire(w_config) => w_config.description.as_ref().map_or(false, |desc| desc.to_uppercase().contains("ACOG")),
        Weapon::SingleShot(w_config) => w_config.description.as_ref().map_or(false, |desc| desc.to_uppercase().contains("ACOG")),
        Weapon::FullAutoStandard(w_config) => w_config.description.as_ref().map_or(false, |desc| desc.to_uppercase().contains("ACOG")),
    };

    println!("Has ACOG: {}", has_acog);

    let (h_multiplier, v_multiplier) = if has_acog {
        (config.mouse_config.acog_horizontal_multiplier, config.mouse_config.acog_vertical_multiplier)
    } else {
        (config.mouse_config.horizontal_multiplier, config.mouse_config.vertical_multiplier)
    };

    let dx_total = dx_total * h_multiplier;
    let dy_total = dy_total * v_multiplier;
    let mut dx_accum = 0.0;
    let mut dy_accum = 0.0;

    let dx_step = dx_total / splits as f32;
    let dy_step = dy_total / splits as f32;

    for _ in 0..splits {
        if wait_first { thread::sleep(total_interval / splits); }

        dx_accum += dx_step;
        dy_accum += dy_step;

        // Take integer part to send via SendInput
        let dx_send = dx_accum.round() as i32;
        let dy_send = dy_accum.round() as i32;

        // Subtract sent portion so remainder accumulates
        dx_accum -= dx_send as f32;
        dy_accum -= dy_send as f32;

        unsafe {
            let mut move_input = INPUT {
                type_: INPUT_MOUSE,
                u: mem::zeroed(),
            };
            *move_input.u.mi_mut() = MOUSEINPUT {
                dx: dx_send,
                dy: dy_send,
                mouseData: 0,
                dwFlags: MOUSEEVENTF_MOVE,
                time: 0,
                dwExtraInfo: 0,
            };

            SendInput(
                1,
                &mut move_input as *mut _,
                std::mem::size_of::<INPUT>() as i32,
            );
        }
        
        if !wait_first { thread::sleep(total_interval / splits); }
    }
}
pub fn handle_hold_lmb (
    state: AppState,
) {
    let mut shooting_started = false;
    
    'outer: loop {
        let global_config = &*state.global_config.read_arc();

        // Check that the right button is also held down
        if global_config.keybinds.require_right_hold && !state.right_hold_active.load(Ordering::SeqCst) {
            // Only send StoppedShooting if we previously started shooting
            if shooting_started {
                if let Err(e) = state.events_channel_sender.send(AppEvent::StoppedShooting) {
                    eprintln!("Failed to send event: {}", e);
                }
                shooting_started = false;
            }

            if !state.left_hold_active.load(Ordering::SeqCst) {
                // If the left button is not held, exit the loop
                return;
            }

            std::thread::sleep(Duration::from_millis(10));
            continue 'outer;
        }

        // Get the current game
        let current_game = state.games.read_arc();
        let current_game_index = state.current_game_index.load(Ordering::SeqCst);
        let current_game = match current_game.get(current_game_index) {
            Some(game) => game,
            None => {
                eprintln!("Game index {} not found", current_game_index);
                return;
            }
        };
        let weapon_id = match get_weapon_id(&state) {
            Ok(weapon_id) => weapon_id,
            Err(e) => {
                eprintln!("Error getting weapon ID: {}", e);
                return;
            }
        };

        // Get the weapon configuration
        let weapon_ind = state.current_weapon_index.load(Ordering::SeqCst);
        let weapon = match current_game.weapons.as_ref().map(|w| w.get(&weapon_id)).flatten() {
            Some(weapon) => weapon.clone(),
            None => {
                eprintln!("Weapon not found: {}", weapon_id);
                return;
            }
        };

        // Emit an event that shooting has started
        if !shooting_started {
            if let Err(e) = state.events_channel_sender.send(AppEvent::StartedShooting { weapon_ind }) {
                eprintln!("Failed to send event: {}", e);
            }
            shooting_started = true;
        }

        println!("Controlling weapon: {}", weapon_id);
        match &weapon {
            Weapon::FullAutoStandard(config) => {
                if !config.enabled {
                    println!("FullAutoStandard weapon disabled: {}", weapon_id);
                    if shooting_started {
                        if let Err(e) = state.events_channel_sender.send(AppEvent::StoppedShooting) {
                            eprintln!("Failed to send event: {}", e);
                        }
                    }
                    break 'outer;
                }
                
                let seconds_in_minute = 60u128;
                let nanoseconds_in_second = 1_000_000_000u128;
                let nanoseconds_per_move = (nanoseconds_in_second * seconds_in_minute) / (config.rpm as u128);
                let interval = Duration::from_nanos(nanoseconds_per_move as u64);

                // Handle the first shot with scaled movement
                let first_shot_scale = config.first_shot_scale;
                let first_dx = config.dx * first_shot_scale;
                let first_dy = config.dy * first_shot_scale;
                move_down(global_config, &weapon, first_dx, first_dy, 3, interval, true);

                let mut iteration = 0;
                while state.left_hold_active.load(Ordering::SeqCst) && !(global_config.keybinds.require_right_hold && !state.right_hold_active.load(Ordering::SeqCst)) {
                    let dy_total = config.dy * config.exponential_factor.powf(iteration as f32);
                    move_down(global_config, &weapon, config.dx, dy_total, 10, interval, false);

                    println!(":3 -");
                    iteration += 1;

                    // Check if the weapon has been changed
                    let new_weapon_ind = state.current_weapon_index.load(Ordering::SeqCst);
                    if new_weapon_ind != weapon_ind {
                        // If the weapon has changed, exit the loop
                        println!("Weapon changed while firing, exiting hold loop.");
                        continue 'outer;
                    }
                }
            }
            Weapon::SingleFire(config) => {
                let trigger_delay = Duration::from_millis(config.trigger_delay_ms as u64);
                let recoil_completion: Duration = Duration::from_millis(config.recoil_completion_ms as u64);
                let release_delay: Duration = Duration::from_millis(config.release_delay_ms as u64);

                while state.left_hold_active.load(Ordering::SeqCst) && !(global_config.keybinds.require_right_hold && !state.right_hold_active.load(Ordering::SeqCst)) {
                    press_key(global_config.keybinds.alternative_fire);
                    
                    // Only apply recoil control if enabled
                    if config.enabled {
                        move_down(
                            global_config, 
                            &weapon,
                            config.dx,
                            config.dy,
                            10,
                            recoil_completion,
                            true
                        );
                    } else {
                        // If recoil control is disabled, just wait for the recoil completion time
                        std::thread::sleep(recoil_completion);
                    }

                    if !state.left_hold_active.load(Ordering::SeqCst) || 
                        !config.autofire ||
                        (global_config.keybinds.require_right_hold && !state.right_hold_active.load(Ordering::SeqCst)) 
                    {
                        break 'outer;
                    }

                    std::thread::sleep(release_delay);

                    release_key(global_config.keybinds.alternative_fire);

                    std::thread::sleep(trigger_delay);

                    // Check if the weapon has been changed
                    let new_weapon_ind = state.current_weapon_index.load(Ordering::SeqCst);
                    if new_weapon_ind != weapon_ind {
                        // If the weapon has changed, exit the loop
                        println!("Weapon changed while firing, exiting hold loop.");
                        continue 'outer;
                    }
                    println!("[SF] :3 -");
                }
            },
            Weapon::SingleShot(config) => {
                if !config.enabled {
                    println!("SingleShot weapon disabled: {}", weapon_id);
                    if shooting_started {
                        if let Err(e) = state.events_channel_sender.send(AppEvent::StoppedShooting) {
                            eprintln!("Failed to send event: {}", e);
                        }
                    }
                    break 'outer;
                }
                
                let recoil_completion: Duration = Duration::from_millis(config.recoil_completion_ms as u64);

                // Move down for the shot
                move_down(
                    global_config, 
                    &weapon,
                    config.dx,
                    config.dy,
                    10,
                    recoil_completion,
                    true
                );

                break 'outer;
            }
        }
        if !state.left_hold_active.load(Ordering::SeqCst) {
            println!("Left button released, exiting hold loop.");
            break;
        }
    }

    // Emit an event that shooting has stopped (only if it was started)
    if shooting_started {
        if let Err(e) = state.events_channel_sender.send(AppEvent::StoppedShooting) {
            eprintln!("Failed to send event: {}", e);
        }
    }
}