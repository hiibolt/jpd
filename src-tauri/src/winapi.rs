extern crate winapi;

use crate::recoil::handle_hold_lmb;
use crate::types::{AppState, AppEvent};

use std::{mem, ptr, thread};
use std::sync::atomic::Ordering;
use winapi::shared::hidusage::*;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::*;

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
                            let games_clone = state.games.clone();
                            let global_config_clone = state.global_config.clone();

                            let events_channel_sender_clone = state.events_channel_sender.clone();

                            let left_hold_clone = state.left_hold_active.clone();
                            let right_hold_clone = state.right_hold_active.clone();
                            let current_game_index_clone = state.current_game_index.clone();
                            let current_category_index_clone = state.current_category_index.clone();
                            let current_loadout_clone = state.current_loadout_index.clone();
                            let current_index_clone = state.current_weapon_index.clone();

                            thread::spawn(|| { handle_hold_lmb(
                                games_clone,
                                global_config_clone,

                                events_channel_sender_clone,
                                
                                left_hold_clone,
                                right_hold_clone,
                                current_game_index_clone,
                                current_category_index_clone,
                                current_loadout_clone,
                                current_index_clone
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

                    // When the '1' key is pressed, switch to the first weapon
                    if flags as u32 & RI_KEY_BREAK != 0 && keyboard.VKey == 0x31 { // '1' key
                        println!("Switching to weapon 1");
                        state.current_weapon_index.store(0, Ordering::SeqCst);

                        // Emit an event that the weapon has been switched
                        if let Err(e) = state.events_channel_sender.send(AppEvent::SwitchedWeapon {
                            weapon_ind: 0,
                        }) {
                            eprintln!("Failed to send event: {}", e);
                        }
                    }
                    // When the '2' key is pressed, switch to the second weapon
                    if flags as u32 & RI_KEY_BREAK != 0 && keyboard.VKey == 0x32 { // '2' key
                        println!("Switching to weapon 2");
                        state.current_weapon_index.store(1, Ordering::SeqCst);

                        // Emit an event that the weapon has been switched
                        if let Err(e) = state.events_channel_sender.send(AppEvent::SwitchedWeapon {
                            weapon_ind: 1,
                        }) {
                            eprintln!("Failed to send event: {}", e);
                        }
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