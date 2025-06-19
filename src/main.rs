extern crate winapi;
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::{mem, ptr, thread, time::Duration};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use winapi::shared::hidusage::*;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::*;

#[derive(Clone)]
struct GlobalConfig {
    require_right_hold: bool,
}
#[derive(Clone)]
struct Loadout {
    name: String,
    weapon_ids: Vec<String>,
}
#[derive(Clone)]
struct AppState {
    weapons: Arc<HashMap<String, Weapon>>,
    global_config: Arc<GlobalConfig>,

    left_hold_active: Arc<AtomicBool>,
    right_hold_active: Arc<AtomicBool>,
    loadout: Arc<Loadout>,
    current_weapon_index: Arc<AtomicUsize>,
}
#[derive(Clone)]
struct SingleFireConfig {
    trigger_delay_ms: u32,
    recoil_completion_ms: u32,
    release_delay_ms: u32,
    dx: f32,
    dy: f32,
    mag_size: u32,
    autofire: bool,
}
#[derive(Clone)]
struct FullAutoStandardConfig {
    rpm: u128,
    first_shot_scale: f32,
    exponential_factor: f32,
    dx: f32,
    dy: f32,
    mag_size: u32,
}

#[derive(Clone)]
enum Weapon {
    SingleFire(SingleFireConfig),
    FullAutoStandard(FullAutoStandardConfig),
}

fn move_down (
    dx_total: f32,
    dy_total: f32,
    splits: u32,
    total_interval: Duration,
    wait_first: bool,
) {
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
fn handle_hold_lmb (
    weapons: Arc<HashMap<String, Weapon>>,
    global_config: Arc<GlobalConfig>,

    left_hold_active: Arc<AtomicBool>,
    right_hold_active: Arc<AtomicBool>,
    loadout: Arc<Loadout>,
    current_index: Arc<AtomicUsize>,
) {
    'outer: loop {
        // Check that the right button is also held down
        if global_config.require_right_hold && !right_hold_active.load(Ordering::SeqCst) {
            println!("Right button not held, continuing hold loop.");

            if !left_hold_active.load(Ordering::SeqCst) {
                // If the left button is not held, exit the loop
                println!("Left button not held either, exiting hold loop.");
                return;
            }

            std::thread::sleep(Duration::from_millis(10));
            continue 'outer;
        }

        let weapon_ind = current_index.load(Ordering::SeqCst);
        let weapon_id = loadout.weapon_ids
            .get(weapon_ind)
            .unwrap_or_else(|| {
                // Set the weapon to the first one if the index is out of bounds
                current_index.store(0, Ordering::SeqCst);
                &loadout.weapon_ids[0]
            });
        let weapon = weapons.get(weapon_id).unwrap_or_else(|| {
            // If the weapon is not found, default to the first one
            &weapons.values().next().expect("No weapons available")
        });

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
                move_down(first_dx, first_dy, 3, interval, true);

                let mut iteration = 0;
                while left_hold_active.load(Ordering::SeqCst) && !(global_config.require_right_hold && !right_hold_active.load(Ordering::SeqCst)) {
                    let dy_total = config.dy * config.exponential_factor.powf(iteration as f32);
                    move_down(config.dx, dy_total, 10, interval, false);

                    println!(":3 -");
                    iteration += 1;

                    // Check if the weapon has been changed
                    let new_weapon_ind = current_index.load(Ordering::SeqCst);
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

                'inner: while left_hold_active.load(Ordering::SeqCst) && !(global_config.require_right_hold && !right_hold_active.load(Ordering::SeqCst)) {
                    // Move down for the next shot
                    move_down(
                        config.dx,
                        config.dy,
                        10,
                        recoil_completion,
                        true
                    );

                    if !left_hold_active.load(Ordering::SeqCst) || 
                        !config.autofire ||
                        (global_config.require_right_hold && !right_hold_active.load(Ordering::SeqCst)) 
                    {
                        break 'inner;
                    }

                    // Pull the trigger
                    unsafe {
                        // "Click" (press 'm' on keyboard)
                        let mut input = INPUT {
                            type_: INPUT_KEYBOARD,
                            u: mem::zeroed(),
                        };
                        *input.u.ki_mut() = KEYBDINPUT {
                            wVk: 0,
                            wScan: 0x32, // 'M' key
                            dwFlags: 0 | KEYEVENTF_SCANCODE,
                            time: 0,
                            dwExtraInfo: 0,
                        };
                    
                        SendInput(1, &mut input, mem::size_of::<INPUT>() as i32);
                    }

                    std::thread::sleep(release_delay);

                    // Release the trigger
                    unsafe {
                        let mut input = INPUT {
                            type_: INPUT_KEYBOARD,
                            u: mem::zeroed(),
                        };
                        *input.u.ki_mut() = KEYBDINPUT {
                            wVk: 0,
                            wScan: 0x32, // 'M' key
                            dwFlags: KEYEVENTF_KEYUP | KEYEVENTF_SCANCODE,
                            time: 0,
                            dwExtraInfo: 0,
                        };
                        SendInput(1, &mut input, mem::size_of::<INPUT>() as i32);
                    }

                    std::thread::sleep(trigger_delay);

                    // Check if the weapon has been changed
                    let new_weapon_ind = current_index.load(Ordering::SeqCst);
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
                            let weapons_clone = state.weapons.clone();
                            let global_config_clone = state.global_config.clone();
                            let left_hold_clone = state.left_hold_active.clone();
                            let right_hold_clone = state.right_hold_active.clone();
                            let loadout_clone = state.loadout.clone();
                            let current_index_clone = state.current_weapon_index.clone();

                            thread::spawn(|| { handle_hold_lmb(weapons_clone, global_config_clone, left_hold_clone, right_hold_clone, loadout_clone, current_index_clone) });
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

                    // When the '1' key is pressed, switch to the first weapon
                    if flags as u32 & RI_KEY_BREAK != 0 && keyboard.VKey == 0x31 { // '1' key
                        println!("Switching to weapon 1");
                        state.current_weapon_index.store(0, Ordering::SeqCst);
                    }
                    // When the '2' key is pressed, switch to the second weapon
                    if flags as u32 & RI_KEY_BREAK != 0 && keyboard.VKey == 0x32 { // '2' key
                        println!("Switching to weapon 2");
                        state.current_weapon_index.store(1, Ordering::SeqCst);
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
fn main() {
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

    println!("[ Started with loadout \"{}\" ]", loadout.name);
    for (i, weapon_id) in loadout.weapon_ids.iter().enumerate() {
        println!("[ Weapon {}: {} ]", i + 1, weapon_id);
    }

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

        let state = AppState {
            weapons: Arc::new(weapons),
            global_config: Arc::new(global_config),

            left_hold_active: Arc::new(AtomicBool::new(false)),
            right_hold_active: Arc::new(AtomicBool::new(false)),
            loadout: Arc::new(loadout),
            current_weapon_index: Arc::new(AtomicUsize::new(0)),
        };
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
