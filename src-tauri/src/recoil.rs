extern crate winapi;
use std::sync::{mpsc::Sender, atomic::AtomicUsize};
use std::{mem, thread, time::Duration};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use parking_lot::RwLock;
use winapi::um::winuser::*;

use crate::types::{AppEvent, Game, GlobalConfig, Weapon};
use crate::winapi::press_and_release_key;

pub fn move_down (
    config: &GlobalConfig,

    dx_total: f32,
    dy_total: f32,
    splits: u32,
    total_interval: Duration,
    wait_first: bool,
) {
    let dx_total = dx_total * config.mouse_config.horizontal_multiplier;
    let dy_total = dy_total * config.mouse_config.vertical_multiplier;
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
    games:         Arc<RwLock<Vec<Game>>>,
    global_config: Arc<RwLock<GlobalConfig>>,

    events_channel_sender: Arc<Sender<AppEvent>>,

    left_hold_active:  Arc<AtomicBool>,
    right_hold_active: Arc<AtomicBool>,
    current_game_index:     Arc<AtomicUsize>,
    current_category_index: Arc<AtomicUsize>,
    current_loadout_index:  Arc<AtomicUsize>,
    current_weapon_index:   Arc<AtomicUsize>,
) {
    'outer: loop {
        let global_config = &*global_config.read_arc();

        // Check that the right button is also held down
        if global_config.keybinds.require_right_hold && !right_hold_active.load(Ordering::SeqCst) {
            // Emit an event that shooting has stopped
            if let Err(e) = events_channel_sender.send(AppEvent::StoppedShooting) {
                eprintln!("Failed to send event: {}", e);
            }

            if !left_hold_active.load(Ordering::SeqCst) {
                // If the left button is not held, exit the loop
                return;
            }

            std::thread::sleep(Duration::from_millis(10));
            continue 'outer;
        }

        // Get the current game
        let games = games.read_arc();
        let current_game_index = current_game_index.load(Ordering::SeqCst);
        if current_game_index >= games.len() {
            eprintln!("Invalid game index: {}", current_game_index);
            return;
        }
        let current_game = &games[current_game_index];

        // Get the current category
        let current_category_index = current_category_index.load(Ordering::SeqCst);
        if current_category_index >= current_game.categories.len() {
            eprintln!("Invalid category index `{}` for game `{}`", current_category_index, current_game.name);
            return;
        }
        let current_category = &current_game.categories[current_category_index];

        // Get the current loadout
        let current_loadout_index = current_loadout_index.load(Ordering::SeqCst);
        if current_loadout_index >= current_category.loadouts.len() {
            eprintln!("Invalid loadout index `{}` for category `{}` in game `{}`", current_loadout_index, current_category.name, current_game.name);
            return;
        }
        let current_loadout = &current_category.loadouts[current_loadout_index];

        // Get the current weapon index
        let weapon_ind = current_weapon_index.load(Ordering::SeqCst);
        if weapon_ind >= current_loadout.weapon_ids.len() {
            eprintln!("Invalid weapon index `{}` for loadout `{}` in category `{}` in game `{}`", weapon_ind, current_loadout.name, current_category.name, current_game.name);
            return;
        }
        let weapon_id = &current_loadout.weapon_ids[weapon_ind];
        // Get the weapon configuration
        let weapon = match current_game.weapons.get(weapon_id) {
            Some(weapon) => weapon.clone(),
            None => {
                eprintln!("Weapon not found: {}", weapon_id);
                return;
            }
        };

        // Emit an event that shooting has started
        if let Err(e) = events_channel_sender.send(AppEvent::StartedShooting { weapon_ind }) {
            eprintln!("Failed to send event: {}", e);
        }

        println!("Game index {current_game_index}, category index {current_category_index}, loadout index {current_loadout_index}, weapon index {weapon_ind}");
        println!("Controlling weapon: {}", weapon_id);
        let mut rounds_fired = 1;
        match weapon {
            Weapon::FullAutoStandard(config) => {
                let seconds_in_minute = 60u128;
                let nanoseconds_in_second = 1_000_000_000u128;
                let nanoseconds_per_move = (nanoseconds_in_second * seconds_in_minute) / config.rpm;
                let interval = Duration::from_nanos(nanoseconds_per_move as u64);

                // Handle the first shot with scaled movement
                let first_shot_scale = config.first_shot_scale;
                let first_dx = config.dx * first_shot_scale;
                let first_dy = config.dy * first_shot_scale;
                move_down(global_config, first_dx, first_dy, 3, interval, true);

                let mut iteration = 0;
                while left_hold_active.load(Ordering::SeqCst) && !(global_config.keybinds.require_right_hold && !right_hold_active.load(Ordering::SeqCst)) {
                    let dy_total = config.dy * config.exponential_factor.powf(iteration as f32);
                    move_down(global_config, config.dx, dy_total, 10, interval, false);

                    println!(":3 -");
                    iteration += 1;

                    // Check if the weapon has been changed
                    let new_weapon_ind = current_weapon_index.load(Ordering::SeqCst);
                    if new_weapon_ind != weapon_ind {
                        // If the weapon has changed, exit the loop
                        println!("Weapon changed while firing, exiting hold loop.");
                        continue 'outer;
                    }

                    rounds_fired += 1;
                    if rounds_fired > config.mag_size {
                        println!("Reached mag size limit, exiting hold loop.");
                        break 'outer;
                    }
                }
            }
            Weapon::SingleFire(config) => {
                let trigger_delay = Duration::from_millis(config.trigger_delay_ms as u64);
                let recoil_completion = Duration::from_millis(config.recoil_completion_ms as u64);
                let release_delay = Duration::from_millis(config.release_delay_ms as u64);

                while left_hold_active.load(Ordering::SeqCst) && !(global_config.keybinds.require_right_hold && !right_hold_active.load(Ordering::SeqCst)) {
                    // Move down for the next shot
                    move_down(
                        global_config, 
                        config.dx,
                        config.dy,
                        10,
                        recoil_completion,
                        true
                    );

                    if !left_hold_active.load(Ordering::SeqCst) || 
                        !config.autofire ||
                        (global_config.keybinds.require_right_hold && !right_hold_active.load(Ordering::SeqCst)) 
                    {
                        break 'outer;
                    }

                    press_and_release_key(
                        global_config.keybinds.alternative_fire,
                        release_delay
                    );

                    std::thread::sleep(trigger_delay);

                    // Check if the weapon has been changed
                    let new_weapon_ind = current_weapon_index.load(Ordering::SeqCst);
                    if new_weapon_ind != weapon_ind {
                        // If the weapon has changed, exit the loop
                        println!("Weapon changed while firing, exiting hold loop.");
                        continue 'outer;
                    }
                    println!("[SF] :3 -");

                    rounds_fired += 1;
                    if rounds_fired > config.mag_size {
                        println!("Reached mag size limit, exiting hold loop.");
                        break 'outer;
                    }
                }
            }
        }
        if !left_hold_active.load(Ordering::SeqCst) {
            println!("Left button released, exiting hold loop.");
            break;
        }
    }

    // Emit an event that shooting has stopped
    if let Err(e) = events_channel_sender.send(AppEvent::StoppedShooting) {
        eprintln!("Failed to send event: {}", e);
    }
}