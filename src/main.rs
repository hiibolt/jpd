//! Minimal example using `winapi` to print `:3` on left click, with thread-safe state.

extern crate winapi;
use std::{mem, ptr, thread, time::Duration};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use winapi::shared::hidusage::*;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::*;

const WEAPON: &'static str = "417";

#[derive(Clone)]
struct AppState {
    hold_active: Arc<AtomicBool>,
    weapon: Arc<WeaponType>,
}
#[derive(Clone)]
struct SingleFireConfig {
    trigger_delay_ms: u32,
    recoil_completion_ms: u32,
    release_delay_ms: u32,
    dx: f32,
    dy: f32,
    autofire: bool,
}
#[derive(Clone)]
struct FullAutoStandardConfig {
    rpm: u128,
    first_shot_scale: f32,
    exponential_factor: f32,
    dx: f32,
    dy: f32,
}

#[derive(Clone)]
enum WeaponType {
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
    state: Arc<AtomicBool>,
    weapon: Arc<WeaponType>,
) {
    match &*weapon {
        WeaponType::FullAutoStandard(config) => {
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
            while state.load(Ordering::SeqCst) {
                let dy_total = config.dy * config.exponential_factor.powf(iteration as f32);
                move_down(config.dx, dy_total, 10, interval, false);

                println!(":3 -");
                iteration += 1;
            }
        }
        WeaponType::SingleFire(config) => {
            let trigger_delay = Duration::from_millis(config.trigger_delay_ms as u64);
            let recoil_completion = Duration::from_millis(config.recoil_completion_ms as u64);
            let release_delay = Duration::from_millis(config.release_delay_ms as u64);

            while state.load(Ordering::SeqCst) {
                // Move down for the next shot
                move_down(
                    config.dx,
                    config.dy,
                    10,
                    recoil_completion,
                    true
                );

                if !state.load(Ordering::SeqCst) || !config.autofire {
                    break;
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

                println!("[SF] :3 -");
            }
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
            if raw.header.dwType == RIM_TYPEMOUSE {
                let mouse = unsafe { raw.data.mouse() };
                let flags = mouse.usButtonFlags;

                // Cast the window pointer to AppState
                let state_ptr = unsafe { 
                    GetWindowLongPtrW(hwnd, GWLP_USERDATA) as *mut AppState
                };
                if !state_ptr.is_null() {
                    let state: &AppState = unsafe { &*state_ptr };

                    // Handle mouse down and up events
                    if flags & RI_MOUSE_LEFT_BUTTON_DOWN != 0 {
                        println!(":3 v");

                        // If the hold is not already active, start a new thread
                        if !state.hold_active.load(Ordering::SeqCst) {
                            state.hold_active.store(true, Ordering::SeqCst);
                            let hold_clone = state.hold_active.clone();
                            let weapon_clone = state.weapon.clone();

                            thread::spawn(|| { handle_hold_lmb(hold_clone, weapon_clone) });
                        }
                    }
                    if flags & RI_MOUSE_LEFT_BUTTON_UP != 0 {
                        println!(":3 ^");
                        state.hold_active.store(false, Ordering::SeqCst);
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
    let weapon_r4_c = WeaponType::FullAutoStandard(FullAutoStandardConfig {
        rpm: 860,
        first_shot_scale: 1.23,
        exponential_factor: 1.007,
        dx: -5.0,
        dy: 129.5,
    });
    let weapon_417 = WeaponType::SingleFire(SingleFireConfig {
        trigger_delay_ms: 80,
        recoil_completion_ms: 10,
        release_delay_ms: 25,
        dx: -1.0,
        dy: 42.5,
        autofire: true,
    });
    let weapon = match WEAPON {
        "R4-C" => weapon_r4_c,
        "417" => weapon_417,
        _ => panic!("Unsupported weapon"),
    };

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
            hold_active: Arc::new(AtomicBool::new(false)),
            weapon: Arc::new(weapon),
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

        let rid = RAWINPUTDEVICE {
            usUsagePage: HID_USAGE_PAGE_GENERIC,
            usUsage: HID_USAGE_GENERIC_MOUSE,
            dwFlags: RIDEV_INPUTSINK,
            hwndTarget: hwnd,
        };
        RegisterRawInputDevices(&rid, 1, mem::size_of::<RAWINPUTDEVICE>() as u32);

        let mut msg: MSG = mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
