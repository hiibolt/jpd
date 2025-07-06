extern crate winapi;

use crate::{get_weapon_id, load_data, save_data};
use crate::recoil::handle_hold_lmb;
use crate::types::{AppEvent, AppState, LoadedData, Weapon};

use std::time::Duration;
use std::{mem, ptr, thread};
use std::sync::atomic::Ordering;
use winapi::shared::hidusage::*;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::*;

pub fn press_key (
    key: char
) {
    let key_as_scancode = char_to_scancode(key);

    unsafe {
        let mut input = INPUT {
            type_: INPUT_KEYBOARD,
            u: mem::zeroed(),
        };
        *input.u.ki_mut() = KEYBDINPUT {
            wVk: 0,
            wScan: key_as_scancode,
            dwFlags: 0 | KEYEVENTF_SCANCODE,
            time: 0,
            dwExtraInfo: 0,
        };

        SendInput(1, &mut input, mem::size_of::<INPUT>() as i32);
    }
}
pub fn release_key (
    key: char
) {
    let key_as_scancode = char_to_scancode(key);

    unsafe {
        let mut input = INPUT {
            type_: INPUT_KEYBOARD,
            u: mem::zeroed(),
        };
        *input.u.ki_mut() = KEYBDINPUT {
            wVk: 0,
            wScan: key_as_scancode,
            dwFlags: KEYEVENTF_KEYUP | KEYEVENTF_SCANCODE,
            time: 0,
            dwExtraInfo: 0,
        };

        SendInput(1, &mut input, mem::size_of::<INPUT>() as i32);
    }
}
pub fn press_and_release_key (
    key: char,
    release_delay: Duration,
) {
    press_key(key);

    thread::sleep(release_delay);

    release_key(key);
}
pub fn char_to_scancode(c: char) -> u16 {
    unsafe {
        let as_vk = char_to_vk(c);

        let return_value = MapVirtualKeyExA (
            as_vk as u32,
            MAPVK_VK_TO_VSC,
            GetKeyboardLayout(0),
        );
        if return_value == 0 {
            panic!("Failed to convert character '{}' to scancode", c);
        }
        return_value as u16
    }
}
pub fn char_to_vk(c: char) -> u16 {
    unsafe {
        let return_value = VkKeyScanExW(
            c as u16,
            GetKeyboardLayout(0),
        );
        if return_value == -1 {
            panic!("Failed to convert character '{}' to virtual key code", c);
        }
        (return_value & 0xFF) as u16
    }
}
unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CREATE => {
            unsafe { 
                let createstruct = lparam as *const CREATESTRUCTW;
                let state_ptr = (*createstruct).lpCreateParams as *mut AppState;
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, state_ptr as isize);
            }
            0
        }
        WM_INPUT => {
            let mut dw_size = 0u32;
            unsafe {
                GetRawInputData(
                    lparam as HRAWINPUT,
                    RID_INPUT,
                    ptr::null_mut(),
                    &mut dw_size,
                    mem::size_of::<RAWINPUTHEADER>() as u32,
                );
            }

            let mut raw_input_bytes = vec![0u8; dw_size as usize];
            let result = unsafe {
                GetRawInputData(
                    lparam as HRAWINPUT,
                    RID_INPUT,
                    raw_input_bytes.as_mut_ptr() as *mut _,
                    &mut dw_size,
                    mem::size_of::<RAWINPUTHEADER>() as u32,
                )
            };
            if result != dw_size {
                return 0;
            }

            let raw: &RAWINPUT = unsafe { &*(raw_input_bytes.as_ptr() as *const RAWINPUT) };
            // Handle mouse inputs
            if raw.header.dwType == RIM_TYPEMOUSE {
                let mouse = unsafe { raw.data.mouse() };
                let flags = mouse.usButtonFlags;

                // Cast the window pointer to AppState
                let state_ptr = unsafe { 
                    GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppState
                };
                if !state_ptr.is_null() {
                    let state: &AppState = unsafe { &*state_ptr };

                    // Handle RMB down and up events
                    if flags & RI_MOUSE_RIGHT_BUTTON_DOWN != 0 {
                        println!(":3 [RMB] v");

                        state.right_hold_active.store(true, Ordering::SeqCst);
                    }
                    if flags & RI_MOUSE_RIGHT_BUTTON_UP != 0 {
                        println!(":3 [RMB] ^");
                        state.right_hold_active.store(false, Ordering::SeqCst);
                    }

                    // Handle LMB down and up events
                    if flags & RI_MOUSE_LEFT_BUTTON_DOWN != 0 {
                        println!(":3 [LMB] v");

                        // If the hold is not already active, start a new thread
                        if !state.left_hold_active.load(Ordering::SeqCst) {
                            state.left_hold_active.store(true, Ordering::SeqCst);

                            let state_cloned = state.clone();
                            thread::spawn(|| { handle_hold_lmb(
                                state_cloned
                            ) });
                        }
                    }
                    if flags & RI_MOUSE_LEFT_BUTTON_UP != 0 {
                        println!(":3 [LMB] ^");
                        state.left_hold_active.store(false, Ordering::SeqCst);
                    }
                }
            }

            // Handle keyboard inputs
            if raw.header.dwType == RIM_TYPEKEYBOARD {
                let keyboard = unsafe { raw.data.keyboard() };
                let flags = keyboard.Flags;

                // Cast the window pointer to AppState
                let state_ptr = unsafe { 
                    GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppState
                };
                if !state_ptr.is_null() {
                    let state: &AppState = unsafe { &*state_ptr };

                    let global_config = state.global_config.read_arc().clone();
                    let primary_key   = char_to_vk(global_config.keybinds.primary_weapon);
                    let secondary_key = char_to_vk(global_config.keybinds.secondary_weapon);

                    if flags as u32 & RI_KEY_BREAK == 0 {
                        return 0; // Ignore key press events
                    }

                    if keyboard.VKey == primary_key {
                        println!("Switching to weapon 1");
                        state.current_weapon_index.store(0, Ordering::SeqCst);

                        // Emit an event that the weapon has been switched
                        if let Err(e) = state.events_channel_sender.send(AppEvent::SwitchedWeapon {
                            weapon_ind: 0,
                        }) {
                            eprintln!("Failed to send event: {}", e);
                        }
                    } else if keyboard.VKey == secondary_key {
                        println!("Switching to weapon 2");
                        state.current_weapon_index.store(1, Ordering::SeqCst);

                        // Emit an event that the weapon has been switched
                        if let Err(e) = state.events_channel_sender.send(AppEvent::SwitchedWeapon {
                            weapon_ind: 1,
                        }) {
                            eprintln!("Failed to send event: {}", e);
                        }
                    } else if keyboard.VKey as i32 == VK_PRIOR || keyboard.VKey as i32 == VK_NEXT 
                        || keyboard.VKey as i32 == VK_HOME || keyboard.VKey as i32 == VK_END
                    {
                        let current_weapon_id = match get_weapon_id(state) {
                            Ok(id) => id,
                            Err(e) => {
                                eprintln!("Error getting weapon ID: {}", e);
                                return 0;
                            }
                        };
                        let mut games = state.games.write_arc();
                        let current_game = if let Some(game) = games.get_mut(state.current_game_index.load(Ordering::SeqCst)) { game } else {
                            eprintln!("Current game index out of bounds");
                            return 0;
                        };
                        
                        let weapon = if let Some(weapon) = current_game.weapons
                            .as_mut()
                            .map(|w| w.get_mut(&current_weapon_id))
                            .flatten()
                        {
                            weapon 
                        } else {
                            eprintln!("Weapon not found: {}", current_weapon_id);
                            return 0;
                        };

                        let (dx_mut_ref, dy_mut_ref) = match weapon {
                            Weapon::SingleFire(config) => (&mut config.dx, &mut config.dy),
                            Weapon::FullAutoStandard(config) => (&mut config.dx, &mut config.dy)
                        };
                        *dx_mut_ref += if keyboard.VKey as i32 == VK_HOME { 0.1 } else if keyboard.VKey as i32 == VK_END { -0.1 } else { 0.0 };
                        *dy_mut_ref += if keyboard.VKey as i32 == VK_PRIOR { 0.1 } else if keyboard.VKey as i32 == VK_NEXT { -0.1 } else { 0.0 };

                        // Round the values to 2 decimal places
                        *dx_mut_ref = (*dx_mut_ref * 100.0).round() / 100.0;
                        *dy_mut_ref = (*dy_mut_ref * 100.0).round() / 100.0;

                        // Emit an event that the config has been updated
                        drop(games); // Drop the lock before saving
                        if let Err(e) = state.events_channel_sender.send(AppEvent::UpdatedGames {
                            games: state.games.read_arc().clone(),
                        }) {
                            eprintln!("Failed to send event: {}", e);
                        }

                        // Save the updated config
                        if let Err(e) = save_data(state) {
                            eprintln!("Failed to save data: {}", e);
                        }
                    } else if keyboard.VKey as i32 == VK_INSERT {
                        println!("Reloading config...");

                        // Load the config from the file
                        let game_data = match load_data(&state.assets_dir_path) {
                            Ok(LoadedData { 
                                available_games,
                                game_data,
                                global_config,
                                key_statuses
                            }) => {
                                println!("Saving game config...");
                                *state.games.write_arc() = game_data.clone();
                                println!("Saving global config...");
                                *state.global_config.write_arc() = global_config;
                                println!("Successfully reloaded config");

                                Some(game_data)
                            },
                            Err(e) => {
                                eprintln!("Failed to load data: {}", e);

                                None
                            }
                        };

                        // Emit an event that the config has been updated
                        println!("Emitting updated games event...");
                        if let Some(game_data) = game_data {
                            if let Err(e) = state.events_channel_sender.send(AppEvent::UpdatedGames {
                                games: game_data,
                            }) {
                                eprintln!("Failed to send event: {}", e);
                            }
                        }
                        println!("Config reloaded successfully");
                    }
                }
            }

            0
        }
        WM_DESTROY => {
            let state_ptr = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppState };
            if !state_ptr.is_null() {
                unsafe { drop(Box::from_raw(state_ptr)); }
            }
            unsafe { PostQuitMessage(0) };

            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
fn to_wstring(s: &str) -> Vec<u16> {
    use std::os::windows::ffi::OsStrExt;
    std::ffi::OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}
pub fn main_recoil (
    state: AppState
) {
    println!("Starting `clc-jpd`...");

    unsafe {
        let hinstance = GetModuleHandleW(ptr::null());
        let class_name = to_wstring("RawInputWnd");

        let wnd_class = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wnd_proc),
            hInstance: hinstance,
            lpszClassName: class_name.as_ptr(),
            ..mem::zeroed()
        };
        RegisterClassW(&wnd_class);

        let state_ptr = Box::into_raw(Box::new(state)) as *mut _;

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            to_wstring("RawInputListener").as_ptr(),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            ptr::null_mut(),
            hinstance,
            state_ptr,
        );

        let rid = [
            RAWINPUTDEVICE {
                usUsagePage: HID_USAGE_PAGE_GENERIC,
                usUsage: HID_USAGE_GENERIC_MOUSE,
                dwFlags: RIDEV_INPUTSINK,
                hwndTarget: hwnd,
            },
            RAWINPUTDEVICE {
                usUsagePage: HID_USAGE_PAGE_GENERIC,
                usUsage: HID_USAGE_GENERIC_KEYBOARD,
                dwFlags: RIDEV_INPUTSINK,
                hwndTarget: hwnd,
            },
        ];
        RegisterRawInputDevices(rid.as_ptr(), 2, mem::size_of::<RAWINPUTDEVICE>() as u32);

        let mut msg: MSG = mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}