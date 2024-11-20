use std::{
    collections::HashMap,
    ffi::c_void,
    mem::zeroed,
    ops::Sub,
    ptr::null_mut,
    sync::Mutex,
};

use once_cell::sync::Lazy;
use windows::{
    core::Result,
    Win32::{
        Foundation::{ HANDLE, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM },
        Graphics::Gdi::{ COLOR_WINDOW, HBRUSH },
        System::SystemInformation::GetTickCount,
        UI::{
            Input::{
                GetRawInputData,
                GetRawInputDeviceList,
                RegisterRawInputDevices,
                HRAWINPUT,
                RAWINPUT,
                RAWINPUTDEVICE,
                RAWINPUTDEVICELIST,
                RAWINPUTHEADER,
                RIDEV_INPUTSINK,
                RID_INPUT,
            },
            WindowsAndMessaging::{
                CallNextHookEx,
                CreateWindowExW,
                DefWindowProcW,
                DispatchMessageW,
                GetMessageW,
                PeekMessageW,
                RegisterClassExW,
                SendMessageW,
                SetWindowsHookExW,
                TranslateMessage,
                CS_NOCLOSE,
                HCURSOR,
                HICON,
                KBDLLHOOKSTRUCT,
                MSG,
                PM_REMOVE,
                WH_KEYBOARD_LL,
                WINDOWS_HOOK_ID,
                WINDOW_EX_STYLE,
                WINDOW_STYLE,
                WM_INPUT,
                WM_KEYDOWN,
                WM_KEYUP,
                WNDCLASSEXW,
                WNDCLASS_STYLES,
            },
        },
    },
};
use windows_core::PCWSTR;

static WINDOW_HANDLE: Lazy<Mutex<u32>> = Lazy::new(|| Mutex::new(0));
static WM_KEY_REQUEST: u32 = 0x6f;
static DECISION_BUFFER: Lazy<Mutex<Vec<LRESULT>>> = Lazy::new(||
    Mutex::new(Vec::new())
);

fn to_pcwstr(s: &str) -> PCWSTR {
    let wide: Vec<u16> = s.encode_utf16().chain(Some(0)).collect();
    PCWSTR(wide.as_ptr())
}

pub fn raw_input_processor() {
    let class_name = to_pcwstr("XPALM_RAW_HANDLE");
    let window_name = to_pcwstr("XPALM_RAW_HANDLE");

    unsafe {
        let wc = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            style: WNDCLASS_STYLES(0x0200),
            lpfnWndProc: Some(raw_keyboard_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: HINSTANCE(null_mut()),
            lpszClassName: class_name,
            hIcon: HICON(null_mut()),
            hCursor: HCURSOR(null_mut()),
            hbrBackground: HBRUSH(COLOR_WINDOW.0 as *mut c_void),
            lpszMenuName: PCWSTR(null_mut()),
            hIconSm: HICON(null_mut()),
        };

        RegisterClassExW(&wc);
    }

    let hwnd: HWND;
    unsafe {
        hwnd = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class_name,
            window_name,
            WINDOW_STYLE(CS_NOCLOSE.0),
            0,
            0,
            0,
            0,
            None,
            None,
            None,
            Some(null_mut())
        ).unwrap();
    }

    {
        let mut window_handle = WINDOW_HANDLE.lock().unwrap();
        *window_handle = hwnd.0 as u32;
    }

    let ticket = [
        RAWINPUTDEVICE {
            usUsagePage: 1,
            usUsage: 6,
            dwFlags: RIDEV_INPUTSINK,
            hwndTarget: hwnd,
        },
    ];

    unsafe {
        SetWindowsHookExW(
            WINDOWS_HOOK_ID(WH_KEYBOARD_LL.0),
            Some(global_keyboard_proc),
            None,
            0
        ).unwrap();
    }

    unsafe {
        RegisterRawInputDevices(
            &ticket,
            size_of::<RAWINPUTDEVICE>() as u32
        ).unwrap();
    }

    unsafe {
        let mut msg: MSG = zeroed();
        while GetMessageW(&mut msg, hwnd, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

extern "system" fn raw_keyboard_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM
) -> LRESULT {
    unsafe {
        match msg {
            WM_INPUT => {
                let mut raw: RAWINPUT = zeroed();
                let mut dw_size = size_of::<RAWINPUT>() as u32;

                let result = GetRawInputData(
                    HRAWINPUT(lparam.0 as *mut c_void),
                    RID_INPUT,
                    Some(&mut raw as *mut _ as *mut c_void),
                    &mut dw_size,
                    size_of::<RAWINPUTHEADER>() as u32
                );

                if result == u32::MAX {
                    println!("Failed to get raw input data");
                }
                {
                    let mut decision_buffer = DECISION_BUFFER.lock().unwrap();

                    if raw.header.hDevice == HANDLE(0x10041 as *mut c_void) {
                        let key_code = raw.data.keyboard.VKey;
                        let pressing = raw.data.keyboard.Flags == 0;

                        decision_buffer.push(LRESULT(1));
                    } else {
                        decision_buffer.push(LRESULT(0));
                    }
                }
            }
            0x6f => {
                {
                    let mut decision_buffer = DECISION_BUFFER.lock().unwrap();

                    if !decision_buffer.is_empty() {
                        return decision_buffer.pop().unwrap();
                    }
                }

                return LRESULT(0);
            }
            _ => (),
        }

        DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

extern "system" fn global_keyboard_proc(
    code: i32,
    wparam: WPARAM,
    lparam: LPARAM
) -> LRESULT {
    if code == 0 {
        unsafe {
            let kb_struct = *(lparam.0 as *const KBDLLHOOKSTRUCT);
            let key_code = kb_struct.vkCode;
        }
    }

    let window_handle = WINDOW_HANDLE.lock().unwrap();
    unsafe {
        if
            SendMessageW(
                HWND(*window_handle as *mut c_void),
                WM_KEY_REQUEST,
                wparam,
                lparam
            ) == LRESULT(1)
        {
            return LRESULT(1);
        }
    }

    unsafe { CallNextHookEx(None, code, wparam, lparam) }
}

fn grab_devices() -> Result<Vec<RAWINPUTDEVICELIST>> {
    let mut devices_count: u32 = 0;

    unsafe {
        GetRawInputDeviceList(
            None,
            &mut devices_count,
            std::mem::size_of::<RAWINPUTDEVICELIST>() as u32
        );
    }

    let mut devices: Vec<RAWINPUTDEVICELIST> =
        vec![
            RAWINPUTDEVICELIST::default();
            devices_count as usize
        ];

    unsafe {
        GetRawInputDeviceList(
            Some(devices.as_mut_ptr()),
            &mut devices_count,
            std::mem::size_of::<RAWINPUTDEVICELIST>() as u32
        );
    }

    Ok(devices)
}
