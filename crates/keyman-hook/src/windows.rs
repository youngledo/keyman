use anyhow::{Result, bail};
use std::sync::Mutex;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::core::PCWSTR;

use crate::event::RawKeyEvent;
use crate::hook::{HookResult, KeyboardHook};
use crate::key::VirtualKey;

pub struct WindowsKeyboardHook {
    hook_handle: Option<HHOOK>,
}

impl WindowsKeyboardHook {
    pub fn new() -> Self {
        Self { hook_handle: None }
    }
}

unsafe fn vk_code_to_virtual_key(vk_code: u32) -> VirtualKey {
    match vk_code {
        0x41 => VirtualKey::A, 0x42 => VirtualKey::B, 0x43 => VirtualKey::C,
        0x44 => VirtualKey::D, 0x45 => VirtualKey::E, 0x46 => VirtualKey::F,
        0x47 => VirtualKey::G, 0x48 => VirtualKey::H, 0x49 => VirtualKey::I,
        0x4A => VirtualKey::J, 0x4B => VirtualKey::K, 0x4C => VirtualKey::L,
        0x4D => VirtualKey::M, 0x4E => VirtualKey::N, 0x4F => VirtualKey::O,
        0x50 => VirtualKey::P, 0x51 => VirtualKey::Q, 0x52 => VirtualKey::R,
        0x53 => VirtualKey::S, 0x54 => VirtualKey::T, 0x55 => VirtualKey::U,
        0x56 => VirtualKey::V, 0x57 => VirtualKey::W, 0x58 => VirtualKey::X,
        0x59 => VirtualKey::Y, 0x5A => VirtualKey::Z,
        0x30 => VirtualKey::Key0, 0x31 => VirtualKey::Key1, 0x32 => VirtualKey::Key2,
        0x33 => VirtualKey::Key3, 0x34 => VirtualKey::Key4, 0x35 => VirtualKey::Key5,
        0x36 => VirtualKey::Key6, 0x37 => VirtualKey::Key7, 0x38 => VirtualKey::Key8,
        0x39 => VirtualKey::Key9,
        0x70 => VirtualKey::F1, 0x71 => VirtualKey::F2,
        0x72 => VirtualKey::F3, 0x73 => VirtualKey::F4,
        0x74 => VirtualKey::F5, 0x75 => VirtualKey::F6,
        0x76 => VirtualKey::F7, 0x77 => VirtualKey::F8,
        0x78 => VirtualKey::F9, 0x79 => VirtualKey::F10,
        0x7A => VirtualKey::F11, 0x7B => VirtualKey::F12,
        0x60 => VirtualKey::Numpad0, 0x61 => VirtualKey::Numpad1,
        0x62 => VirtualKey::Numpad2, 0x63 => VirtualKey::Numpad3,
        0x64 => VirtualKey::Numpad4, 0x65 => VirtualKey::Numpad5,
        0x66 => VirtualKey::Numpad6, 0x67 => VirtualKey::Numpad7,
        0x68 => VirtualKey::Numpad8, 0x69 => VirtualKey::Numpad9,
        0x20 => VirtualKey::Space, 0x0D => VirtualKey::Enter,
        0x1B => VirtualKey::Escape, 0x09 => VirtualKey::Tab,
        0x08 => VirtualKey::Backspace, 0x2E => VirtualKey::Delete,
        0x10 => VirtualKey::Shift, 0x11 => VirtualKey::Control,
        0x12 => VirtualKey::Alt, 0x14 => VirtualKey::CapsLock,
        0x91 => VirtualKey::ScrollLock, 0x90 => VirtualKey::NumLock,
        0x5B => VirtualKey::LWin, 0x5C => VirtualKey::RWin,
        0x26 => VirtualKey::Up, 0x28 => VirtualKey::Down,
        0x25 => VirtualKey::Left, 0x27 => VirtualKey::Right,
        0x24 => VirtualKey::Home, 0x23 => VirtualKey::End,
        0x21 => VirtualKey::PageUp, 0x22 => VirtualKey::PageDown,
        0x13 => VirtualKey::Pause,
        0x2D => VirtualKey::Insert,
        0xBD => VirtualKey::Minus, 0xBB => VirtualKey::Equal,
        0xDB => VirtualKey::LeftBracket, 0xDD => VirtualKey::RightBracket,
        0xDC => VirtualKey::Backslash, 0xBA => VirtualKey::Semicolon,
        0xDE => VirtualKey::Quote, 0xBC => VirtualKey::Comma,
        0xBE => VirtualKey::Period, 0xBF => VirtualKey::Slash,
        _ => VirtualKey::Unknown(vk_code),
    }
}

unsafe fn virtual_key_to_vk_code(key: VirtualKey) -> u16 {
    match key {
        VirtualKey::A => 0x41, VirtualKey::B => 0x42, VirtualKey::C => 0x43,
        VirtualKey::D => 0x44, VirtualKey::E => 0x45, VirtualKey::F => 0x46,
        VirtualKey::G => 0x47, VirtualKey::H => 0x48, VirtualKey::I => 0x49,
        VirtualKey::J => 0x4A, VirtualKey::K => 0x4B, VirtualKey::L => 0x4C,
        VirtualKey::M => 0x4D, VirtualKey::N => 0x4E, VirtualKey::O => 0x4F,
        VirtualKey::P => 0x50, VirtualKey::Q => 0x51, VirtualKey::R => 0x52,
        VirtualKey::S => 0x53, VirtualKey::T => 0x54, VirtualKey::U => 0x55,
        VirtualKey::V => 0x56, VirtualKey::W => 0x57, VirtualKey::X => 0x58,
        VirtualKey::Y => 0x59, VirtualKey::Z => 0x5A,
        VirtualKey::Key0 => 0x30, VirtualKey::Key1 => 0x31, VirtualKey::Key2 => 0x32,
        VirtualKey::Key3 => 0x33, VirtualKey::Key4 => 0x34, VirtualKey::Key5 => 0x35,
        VirtualKey::Key6 => 0x36, VirtualKey::Key7 => 0x37, VirtualKey::Key8 => 0x38,
        VirtualKey::Key9 => 0x39,
        VirtualKey::F1 => VK_F1.0, VirtualKey::F2 => VK_F2.0, VirtualKey::F3 => VK_F3.0,
        VirtualKey::F4 => VK_F4.0, VirtualKey::F5 => VK_F5.0, VirtualKey::F6 => VK_F6.0,
        VirtualKey::F7 => VK_F7.0, VirtualKey::F8 => VK_F8.0, VirtualKey::F9 => VK_F9.0,
        VirtualKey::F10 => VK_F10.0, VirtualKey::F11 => VK_F11.0, VirtualKey::F12 => VK_F12.0,
        VirtualKey::Numpad0 => VK_NUMPAD0.0, VirtualKey::Numpad1 => VK_NUMPAD1.0,
        VirtualKey::Numpad2 => VK_NUMPAD2.0, VirtualKey::Numpad3 => VK_NUMPAD3.0,
        VirtualKey::Numpad4 => VK_NUMPAD4.0, VirtualKey::Numpad5 => VK_NUMPAD5.0,
        VirtualKey::Numpad6 => VK_NUMPAD6.0, VirtualKey::Numpad7 => VK_NUMPAD7.0,
        VirtualKey::Numpad8 => VK_NUMPAD8.0, VirtualKey::Numpad9 => VK_NUMPAD9.0,
        VirtualKey::NumpadAdd => VK_ADD.0, VirtualKey::NumpadSubtract => VK_SUBTRACT.0,
        VirtualKey::NumpadMultiply => VK_MULTIPLY.0, VirtualKey::NumpadDivide => VK_DIVIDE.0,
        VirtualKey::NumpadEnter => VK_RETURN.0,
        VirtualKey::Space => VK_SPACE.0, VirtualKey::Enter => VK_RETURN.0,
        VirtualKey::Escape => VK_ESCAPE.0, VirtualKey::Tab => VK_TAB.0,
        VirtualKey::Backspace => VK_BACK.0, VirtualKey::Delete => VK_DELETE.0,
        VirtualKey::Insert => VK_INSERT.0,
        VirtualKey::Pause => VK_PAUSE.0,
        VirtualKey::Shift => VK_SHIFT.0, VirtualKey::Control => VK_CONTROL.0,
        VirtualKey::Alt => VK_MENU.0, VirtualKey::CapsLock => VK_CAPITAL.0,
        VirtualKey::ScrollLock => VK_SCROLL.0, VirtualKey::NumLock => VK_NUMLOCK.0,
        VirtualKey::Up => VK_UP.0, VirtualKey::Down => VK_DOWN.0,
        VirtualKey::Left => VK_LEFT.0, VirtualKey::Right => VK_RIGHT.0,
        VirtualKey::Home => VK_HOME.0, VirtualKey::End => VK_END.0,
        VirtualKey::PageUp => VK_PRIOR.0, VirtualKey::PageDown => VK_NEXT.0,
        VirtualKey::Minus => VK_OEM_MINUS.0, VirtualKey::Equal => VK_OEM_PLUS.0,
        VirtualKey::LeftBracket => VK_OEM_4.0, VirtualKey::RightBracket => VK_OEM_6.0,
        VirtualKey::Backslash => VK_OEM_5.0, VirtualKey::Semicolon => VK_OEM_1.0,
        VirtualKey::Quote => VK_OEM_7.0, VirtualKey::Comma => VK_OEM_COMMA.0,
        VirtualKey::Period => VK_OEM_PERIOD.0, VirtualKey::Slash => VK_OEM_2.0,
        VirtualKey::Unknown(code) => code as u16,
        _ => 0,
    }
}

/// Magic value placed in dwExtraInfo to identify simulated key events
/// that should not be re-processed by the hook.
const SIMULATED_KEY_MARKER: usize = 0xDEAD_BEEF;

unsafe fn send_key(key: VirtualKey, pressed: bool) {
    let vk_code = unsafe { virtual_key_to_vk_code(key) };
    let flags = if pressed {
        KEYBD_EVENT_FLAGS(0)
    } else {
        KEYEVENTF_KEYUP
    };

    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VIRTUAL_KEY(vk_code),
                wScan: 0,
                dwFlags: flags,
                time: 0,
                dwExtraInfo: SIMULATED_KEY_MARKER,
            },
        },
    };

    unsafe { let _ = SendInput(&[input], std::mem::size_of::<INPUT>() as i32); }
}

static CALLBACK: Mutex<Option<Box<dyn Fn(&RawKeyEvent) -> HookResult + Send + 'static>>> = Mutex::new(None);

impl KeyboardHook for WindowsKeyboardHook {
    fn install(&mut self, callback: Box<dyn Fn(&RawKeyEvent) -> HookResult + Send>) -> Result<()> {
        {
            let mut cb = CALLBACK.lock().unwrap();
            *cb = Some(callback);
        }

        unsafe {
            let hook = SetWindowsHookExW(
                WH_KEYBOARD_LL,
                Some(low_level_keyboard_proc),
                GetModuleHandleW(PCWSTR::null()).ok().map(|h| HINSTANCE(h.0)),
                0,
            );

            let hook = match hook {
                Ok(h) => h,
                Err(_) => bail!("Failed to install keyboard hook"),
            };

            self.hook_handle = Some(hook);
        }

        Ok(())
    }

    fn uninstall(&mut self) -> Result<()> {
        if let Some(hook) = self.hook_handle.take() {
            unsafe {
                let _ = UnhookWindowsHookEx(hook);
            }
        }
        {
            let mut cb = CALLBACK.lock().unwrap();
            *cb = None;
        }
        Ok(())
    }
}

unsafe extern "system" fn low_level_keyboard_proc(
    n_code: i32,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if n_code >= 0 {
        let kb_struct = unsafe { &*(l_param.0 as *const KBDLLHOOKSTRUCT) };

        // Skip simulated key events to prevent infinite loops
        if kb_struct.dwExtraInfo != 0 {
            return unsafe { CallNextHookEx(None, n_code, w_param, l_param) };
        }

        let vk_code = kb_struct.vkCode;
        let pressed = matches!(
            w_param.0 as u32,
            WM_KEYDOWN | WM_SYSKEYDOWN,
        );

        let key = unsafe { vk_code_to_virtual_key(vk_code) };
        let event = RawKeyEvent { key, pressed };

        let cb = CALLBACK.lock().unwrap();
        if let Some(ref callback) = *cb {
            match callback(&event) {
                HookResult::Pass => {}
                HookResult::Suppress => {
                    return LRESULT(1);
                }
                HookResult::Replace(new_key) => {
                    // Suppress original key and send replacement
                    // Must release lock before calling send_key to avoid deadlock
                    drop(cb);
                    unsafe { send_key(new_key, pressed) };
                    return LRESULT(1);
                }
            }
        }
    }

    unsafe { CallNextHookEx(None, n_code, w_param, l_param) }
}
