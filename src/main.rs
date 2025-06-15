//! Minimal example using `winapi` to print `:3` on left click, with thread-safe state.

extern crate winapi;
use std::{mem, ptr, thread, time::Duration};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use winapi::shared::hidusage::*;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::*;

#[derive(Clone)]
struct AppState {
    hold_active: Arc<AtomicBool>,
}

fn pull_down (
    state: Arc<AtomicBool>,
) {
    let interval = Duration::from_millis(100);
    while state.load(Ordering::SeqCst) {
        unsafe {
            let mut move_input = INPUT {
                type_: INPUT_MOUSE,
                u: mem::zeroed(),
            };
            *move_input.u.mi_mut() = MOUSEINPUT {
                dx: 0,
                dy: 3,
                mouseData: 0,
                dwFlags: MOUSEEVENTF_MOVE,
                time: 0,
                dwExtraInfo: 0,
            };

            let inputs = [move_input];
            SendInput(
                inputs.len() as u32,
                inputs.as_ptr() as *mut _,
                mem::size_of::<INPUT>() as i32
            );
        }


        println!(":3 -");
        thread::sleep(interval);
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

                            thread::spawn(|| { pull_down(hold_clone) });
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
