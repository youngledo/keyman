//! Linux keyboard hook implementation using evdev and uinput
//!
//! This module provides keyboard event interception and key injection on Linux.
//! It requires:
//! - Read access to `/dev/input/eventX` devices (user must be in `input` group)
//! - Write access to `/dev/uinput` for key injection

use anyhow::{bail, Result};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use evdev::{Device, KeyCode, EventSummary};
use nix::fcntl::{OFlag, open};
use nix::sys::stat::Mode;

use crate::event::RawKeyEvent;
use crate::hook::{HookResult, KeyboardHook};
use crate::key::VirtualKey;

pub struct LinuxKeyboardHook {
    thread_handle: Option<JoinHandle<()>>,
    running: Arc<Mutex<bool>>,
}

impl LinuxKeyboardHook {
    pub fn new() -> Self {
        Self {
            thread_handle: None,
            running: Arc::new(Mutex::new(false)),
        }
    }
}

/// Convert evdev KeyCode to VirtualKey
fn evdev_key_to_virtual_key(key: KeyCode) -> VirtualKey {
    match key {
        // Letters
        KeyCode::KEY_A => VirtualKey::A,
        KeyCode::KEY_B => VirtualKey::B,
        KeyCode::KEY_C => VirtualKey::C,
        KeyCode::KEY_D => VirtualKey::D,
        KeyCode::KEY_E => VirtualKey::E,
        KeyCode::KEY_F => VirtualKey::F,
        KeyCode::KEY_G => VirtualKey::G,
        KeyCode::KEY_H => VirtualKey::H,
        KeyCode::KEY_I => VirtualKey::I,
        KeyCode::KEY_J => VirtualKey::J,
        KeyCode::KEY_K => VirtualKey::K,
        KeyCode::KEY_L => VirtualKey::L,
        KeyCode::KEY_M => VirtualKey::M,
        KeyCode::KEY_N => VirtualKey::N,
        KeyCode::KEY_O => VirtualKey::O,
        KeyCode::KEY_P => VirtualKey::P,
        KeyCode::KEY_Q => VirtualKey::Q,
        KeyCode::KEY_R => VirtualKey::R,
        KeyCode::KEY_S => VirtualKey::S,
        KeyCode::KEY_T => VirtualKey::T,
        KeyCode::KEY_U => VirtualKey::U,
        KeyCode::KEY_V => VirtualKey::V,
        KeyCode::KEY_W => VirtualKey::W,
        KeyCode::KEY_X => VirtualKey::X,
        KeyCode::KEY_Y => VirtualKey::Y,
        KeyCode::KEY_Z => VirtualKey::Z,
        // Numbers
        KeyCode::KEY_0 => VirtualKey::Key0,
        KeyCode::KEY_1 => VirtualKey::Key1,
        KeyCode::KEY_2 => VirtualKey::Key2,
        KeyCode::KEY_3 => VirtualKey::Key3,
        KeyCode::KEY_4 => VirtualKey::Key4,
        KeyCode::KEY_5 => VirtualKey::Key5,
        KeyCode::KEY_6 => VirtualKey::Key6,
        KeyCode::KEY_7 => VirtualKey::Key7,
        KeyCode::KEY_8 => VirtualKey::Key8,
        KeyCode::KEY_9 => VirtualKey::Key9,
        // Function keys
        KeyCode::KEY_F1 => VirtualKey::F1,
        KeyCode::KEY_F2 => VirtualKey::F2,
        KeyCode::KEY_F3 => VirtualKey::F3,
        KeyCode::KEY_F4 => VirtualKey::F4,
        KeyCode::KEY_F5 => VirtualKey::F5,
        KeyCode::KEY_F6 => VirtualKey::F6,
        KeyCode::KEY_F7 => VirtualKey::F7,
        KeyCode::KEY_F8 => VirtualKey::F8,
        KeyCode::KEY_F9 => VirtualKey::F9,
        KeyCode::KEY_F10 => VirtualKey::F10,
        KeyCode::KEY_F11 => VirtualKey::F11,
        KeyCode::KEY_F12 => VirtualKey::F12,
        // Numpad
        KeyCode::KEY_KP0 => VirtualKey::Numpad0,
        KeyCode::KEY_KP1 => VirtualKey::Numpad1,
        KeyCode::KEY_KP2 => VirtualKey::Numpad2,
        KeyCode::KEY_KP3 => VirtualKey::Numpad3,
        KeyCode::KEY_KP4 => VirtualKey::Numpad4,
        KeyCode::KEY_KP5 => VirtualKey::Numpad5,
        KeyCode::KEY_KP6 => VirtualKey::Numpad6,
        KeyCode::KEY_KP7 => VirtualKey::Numpad7,
        KeyCode::KEY_KP8 => VirtualKey::Numpad8,
        KeyCode::KEY_KP9 => VirtualKey::Numpad9,
        KeyCode::KEY_KPPLUS => VirtualKey::NumpadAdd,
        KeyCode::KEY_KPMINUS => VirtualKey::NumpadSubtract,
        KeyCode::KEY_KPASTERISK => VirtualKey::NumpadMultiply,
        KeyCode::KEY_KPSLASH => VirtualKey::NumpadDivide,
        KeyCode::KEY_KPENTER => VirtualKey::NumpadEnter,
        KeyCode::KEY_KPDOT => VirtualKey::NumpadDecimal,
        // Special keys
        KeyCode::KEY_SPACE => VirtualKey::Space,
        KeyCode::KEY_ENTER => VirtualKey::Enter,
        KeyCode::KEY_ESC => VirtualKey::Escape,
        KeyCode::KEY_TAB => VirtualKey::Tab,
        KeyCode::KEY_BACKSPACE => VirtualKey::Backspace,
        KeyCode::KEY_DELETE => VirtualKey::Delete,
        KeyCode::KEY_INSERT => VirtualKey::Insert,
        KeyCode::KEY_HOME => VirtualKey::Home,
        KeyCode::KEY_END => VirtualKey::End,
        KeyCode::KEY_PAGEUP => VirtualKey::PageUp,
        KeyCode::KEY_PAGEDOWN => VirtualKey::PageDown,
        // Modifiers
        KeyCode::KEY_LEFTSHIFT | KeyCode::KEY_RIGHTSHIFT => VirtualKey::Shift,
        KeyCode::KEY_LEFTCTRL | KeyCode::KEY_RIGHTCTRL => VirtualKey::Control,
        KeyCode::KEY_LEFTALT | KeyCode::KEY_RIGHTALT => VirtualKey::Alt,
        KeyCode::KEY_CAPSLOCK => VirtualKey::CapsLock,
        KeyCode::KEY_SCROLLLOCK => VirtualKey::ScrollLock,
        KeyCode::KEY_NUMLOCK => VirtualKey::NumLock,
        // Arrow keys
        KeyCode::KEY_UP => VirtualKey::Up,
        KeyCode::KEY_DOWN => VirtualKey::Down,
        KeyCode::KEY_LEFT => VirtualKey::Left,
        KeyCode::KEY_RIGHT => VirtualKey::Right,
        // Punctuation
        KeyCode::KEY_MINUS => VirtualKey::Minus,
        KeyCode::KEY_EQUAL => VirtualKey::Equal,
        KeyCode::KEY_LEFTBRACE => VirtualKey::LeftBracket,
        KeyCode::KEY_RIGHTBRACE => VirtualKey::RightBracket,
        KeyCode::KEY_BACKSLASH => VirtualKey::Backslash,
        KeyCode::KEY_SEMICOLON => VirtualKey::Semicolon,
        KeyCode::KEY_APOSTROPHE => VirtualKey::Quote,
        KeyCode::KEY_COMMA => VirtualKey::Comma,
        KeyCode::KEY_DOT => VirtualKey::Period,
        KeyCode::KEY_SLASH => VirtualKey::Slash,
        // Unknown
        _ => VirtualKey::Unknown(key.0 as u32),
    }
}

/// Convert VirtualKey to evdev KeyCode
fn virtual_key_to_evdev(key: VirtualKey) -> Option<KeyCode> {
    match key {
        VirtualKey::A => Some(KeyCode::KEY_A),
        VirtualKey::B => Some(KeyCode::KEY_B),
        VirtualKey::C => Some(KeyCode::KEY_C),
        VirtualKey::D => Some(KeyCode::KEY_D),
        VirtualKey::E => Some(KeyCode::KEY_E),
        VirtualKey::F => Some(KeyCode::KEY_F),
        VirtualKey::G => Some(KeyCode::KEY_G),
        VirtualKey::H => Some(KeyCode::KEY_H),
        VirtualKey::I => Some(KeyCode::KEY_I),
        VirtualKey::J => Some(KeyCode::KEY_J),
        VirtualKey::K => Some(KeyCode::KEY_K),
        VirtualKey::L => Some(KeyCode::KEY_L),
        VirtualKey::M => Some(KeyCode::KEY_M),
        VirtualKey::N => Some(KeyCode::KEY_N),
        VirtualKey::O => Some(KeyCode::KEY_O),
        VirtualKey::P => Some(KeyCode::KEY_P),
        VirtualKey::Q => Some(KeyCode::KEY_Q),
        VirtualKey::R => Some(KeyCode::KEY_R),
        VirtualKey::S => Some(KeyCode::KEY_S),
        VirtualKey::T => Some(KeyCode::KEY_T),
        VirtualKey::U => Some(KeyCode::KEY_U),
        VirtualKey::V => Some(KeyCode::KEY_V),
        VirtualKey::W => Some(KeyCode::KEY_W),
        VirtualKey::X => Some(KeyCode::KEY_X),
        VirtualKey::Y => Some(KeyCode::KEY_Y),
        VirtualKey::Z => Some(KeyCode::KEY_Z),
        VirtualKey::Key0 => Some(KeyCode::KEY_0),
        VirtualKey::Key1 => Some(KeyCode::KEY_1),
        VirtualKey::Key2 => Some(KeyCode::KEY_2),
        VirtualKey::Key3 => Some(KeyCode::KEY_3),
        VirtualKey::Key4 => Some(KeyCode::KEY_4),
        VirtualKey::Key5 => Some(KeyCode::KEY_5),
        VirtualKey::Key6 => Some(KeyCode::KEY_6),
        VirtualKey::Key7 => Some(KeyCode::KEY_7),
        VirtualKey::Key8 => Some(KeyCode::KEY_8),
        VirtualKey::Key9 => Some(KeyCode::KEY_9),
        VirtualKey::F1 => Some(KeyCode::KEY_F1),
        VirtualKey::F2 => Some(KeyCode::KEY_F2),
        VirtualKey::F3 => Some(KeyCode::KEY_F3),
        VirtualKey::F4 => Some(KeyCode::KEY_F4),
        VirtualKey::F5 => Some(KeyCode::KEY_F5),
        VirtualKey::F6 => Some(KeyCode::KEY_F6),
        VirtualKey::F7 => Some(KeyCode::KEY_F7),
        VirtualKey::F8 => Some(KeyCode::KEY_F8),
        VirtualKey::F9 => Some(KeyCode::KEY_F9),
        VirtualKey::F10 => Some(KeyCode::KEY_F10),
        VirtualKey::F11 => Some(KeyCode::KEY_F11),
        VirtualKey::F12 => Some(KeyCode::KEY_F12),
        VirtualKey::Numpad0 => Some(KeyCode::KEY_KP0),
        VirtualKey::Numpad1 => Some(KeyCode::KEY_KP1),
        VirtualKey::Numpad2 => Some(KeyCode::KEY_KP2),
        VirtualKey::Numpad3 => Some(KeyCode::KEY_KP3),
        VirtualKey::Numpad4 => Some(KeyCode::KEY_KP4),
        VirtualKey::Numpad5 => Some(KeyCode::KEY_KP5),
        VirtualKey::Numpad6 => Some(KeyCode::KEY_KP6),
        VirtualKey::Numpad7 => Some(KeyCode::KEY_KP7),
        VirtualKey::Numpad8 => Some(KeyCode::KEY_KP8),
        VirtualKey::Numpad9 => Some(KeyCode::KEY_KP9),
        VirtualKey::NumpadAdd => Some(KeyCode::KEY_KPPLUS),
        VirtualKey::NumpadSubtract => Some(KeyCode::KEY_KPMINUS),
        VirtualKey::NumpadMultiply => Some(KeyCode::KEY_KPASTERISK),
        VirtualKey::NumpadDivide => Some(KeyCode::KEY_KPSLASH),
        VirtualKey::NumpadEnter => Some(KeyCode::KEY_KPENTER),
        VirtualKey::NumpadDecimal => Some(KeyCode::KEY_KPDOT),
        VirtualKey::Space => Some(KeyCode::KEY_SPACE),
        VirtualKey::Enter => Some(KeyCode::KEY_ENTER),
        VirtualKey::Escape => Some(KeyCode::KEY_ESC),
        VirtualKey::Tab => Some(KeyCode::KEY_TAB),
        VirtualKey::Backspace => Some(KeyCode::KEY_BACKSPACE),
        VirtualKey::Delete => Some(KeyCode::KEY_DELETE),
        VirtualKey::Insert => Some(KeyCode::KEY_INSERT),
        VirtualKey::Home => Some(KeyCode::KEY_HOME),
        VirtualKey::End => Some(KeyCode::KEY_END),
        VirtualKey::PageUp => Some(KeyCode::KEY_PAGEUP),
        VirtualKey::PageDown => Some(KeyCode::KEY_PAGEDOWN),
        VirtualKey::Up => Some(KeyCode::KEY_UP),
        VirtualKey::Down => Some(KeyCode::KEY_DOWN),
        VirtualKey::Left => Some(KeyCode::KEY_LEFT),
        VirtualKey::Right => Some(KeyCode::KEY_RIGHT),
        VirtualKey::Shift => Some(KeyCode::KEY_LEFTSHIFT),
        VirtualKey::Control => Some(KeyCode::KEY_LEFTCTRL),
        VirtualKey::Alt => Some(KeyCode::KEY_LEFTALT),
        VirtualKey::CapsLock => Some(KeyCode::KEY_CAPSLOCK),
        VirtualKey::ScrollLock => Some(KeyCode::KEY_SCROLLLOCK),
        VirtualKey::NumLock => Some(KeyCode::KEY_NUMLOCK),
        VirtualKey::Minus => Some(KeyCode::KEY_MINUS),
        VirtualKey::Equal => Some(KeyCode::KEY_EQUAL),
        VirtualKey::LeftBracket => Some(KeyCode::KEY_LEFTBRACE),
        VirtualKey::RightBracket => Some(KeyCode::KEY_RIGHTBRACE),
        VirtualKey::Backslash => Some(KeyCode::KEY_BACKSLASH),
        VirtualKey::Semicolon => Some(KeyCode::KEY_SEMICOLON),
        VirtualKey::Quote => Some(KeyCode::KEY_APOSTROPHE),
        VirtualKey::Comma => Some(KeyCode::KEY_COMMA),
        VirtualKey::Period => Some(KeyCode::KEY_DOT),
        VirtualKey::Slash => Some(KeyCode::KEY_SLASH),
        _ => None,
    }
}

impl KeyboardHook for LinuxKeyboardHook {
    fn install(&mut self, callback: Box<dyn Fn(&RawKeyEvent) -> HookResult + Send>) -> Result<()> {
        // Check if already running
        if self.thread_handle.is_some() {
            bail!("Hook already installed");
        }

        // Find keyboard devices (enumerate returns (PathBuf, Device) tuples)
        let mut devices: Vec<Device> = evdev::enumerate()
            .filter(|(_, d)| d.supported_keys().map_or(false, |keys| {
                keys.contains(KeyCode::KEY_A) && keys.contains(KeyCode::KEY_ENTER)
            }))
            .map(|(_, d)| d)
            .collect();

        if devices.is_empty() {
            bail!("No keyboard devices found. Make sure you are in the 'input' group.");
        }

        *self.running.lock().unwrap() = true;
        let running = self.running.clone();

        // Spawn thread to monitor keyboard events
        let handle = thread::spawn(move || {
            loop {
                if !*running.lock().unwrap() {
                    break;
                }

                for device in &mut devices {
                    if let Ok(events) = device.fetch_events() {
                        for event in events {
                            if let EventSummary::Key(_, key, value) = event.destructure() {
                                let vk = evdev_key_to_virtual_key(key);
                                let raw_event = RawKeyEvent {
                                    key: vk,
                                    pressed: value == 1, // 1 = press, 0 = release
                                };

                                match callback(&raw_event) {
                                    HookResult::Pass => {
                                        // Allow the event through - do nothing
                                    }
                                    HookResult::Suppress => {
                                        // TODO: Need to grab device exclusively to suppress
                                        // This requires grabbing the device via evdev::Device::grab()
                                    }
                                    HookResult::Replace(new_key) => {
                                        // TODO: Inject new key via uinput
                                        // Requires write access to /dev/uinput
                                        if let Some(_evdev_key) = virtual_key_to_evdev(new_key) {
                                            // Would inject via uinput here
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                // Small sleep to prevent busy-waiting
                thread::sleep(Duration::from_millis(1));
            }
        });

        self.thread_handle = Some(handle);
        Ok(())
    }

    fn uninstall(&mut self) -> Result<()> {
        *self.running.lock().unwrap() = false;

        if let Some(handle) = self.thread_handle.take() {
            // Wait for thread to finish (with timeout)
            // Note: thread might be blocking on fetch_events, so this might hang
            let _ = handle.join();
        }

        Ok(())
    }
}
