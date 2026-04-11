use anyhow::{Result, bail};
use core_foundation::runloop::{CFRunLoop, kCFRunLoopCommonModes};
use core_graphics::event::{
    CGEventTap, CGEventTapLocation, CGEventTapPlacement, CGEventTapOptions,
    CGEventType, EventField,
};
use std::sync::Mutex;

use crate::event::RawKeyEvent;
use crate::hook::{HookResult, KeyboardHook};
use crate::key::VirtualKey;

pub struct MacosKeyboardHook {
    tap: Option<CGEventTap<'static>>,
}

impl MacosKeyboardHook {
    pub fn new() -> Self {
        Self { tap: None }
    }
}

fn keycode_to_virtual_key(keycode: u16) -> VirtualKey {
    match keycode {
        0x00 => VirtualKey::A,
        0x01 => VirtualKey::S,
        0x02 => VirtualKey::D,
        0x03 => VirtualKey::F,
        0x04 => VirtualKey::H,
        0x05 => VirtualKey::G,
        0x06 => VirtualKey::Z,
        0x07 => VirtualKey::X,
        0x08 => VirtualKey::C,
        0x09 => VirtualKey::V,
        0x0B => VirtualKey::B,
        0x0C => VirtualKey::Q,
        0x0D => VirtualKey::W,
        0x0E => VirtualKey::E,
        0x0F => VirtualKey::R,
        0x10 => VirtualKey::Y,
        0x11 => VirtualKey::T,
        0x12 => VirtualKey::Key1,
        0x13 => VirtualKey::Key2,
        0x14 => VirtualKey::Key3,
        0x15 => VirtualKey::Key4,
        0x16 => VirtualKey::Key6,
        0x17 => VirtualKey::Key5,
        0x18 => VirtualKey::Equal,
        0x19 => VirtualKey::Key9,
        0x1A => VirtualKey::Key7,
        0x1B => VirtualKey::Minus,
        0x1C => VirtualKey::Key8,
        0x1D => VirtualKey::Key0,
        0x1E => VirtualKey::RightBracket,
        0x1F => VirtualKey::O,
        0x20 => VirtualKey::U,
        0x21 => VirtualKey::LeftBracket,
        0x22 => VirtualKey::I,
        0x23 => VirtualKey::P,
        0x24 => VirtualKey::Enter,
        0x25 => VirtualKey::L,
        0x26 => VirtualKey::J,
        0x27 => VirtualKey::Quote,
        0x28 => VirtualKey::K,
        0x29 => VirtualKey::Semicolon,
        0x2A => VirtualKey::Backslash,
        0x2B => VirtualKey::Comma,
        0x2C => VirtualKey::Slash,
        0x2D => VirtualKey::N,
        0x2E => VirtualKey::M,
        0x2F => VirtualKey::Period,
        0x30 => VirtualKey::Tab,
        0x31 => VirtualKey::Space,
        0x33 => VirtualKey::Backspace,
        0x35 => VirtualKey::Escape,
        0x37 => VirtualKey::Command,
        0x38 => VirtualKey::Shift,
        0x39 => VirtualKey::CapsLock,
        0x3A => VirtualKey::Alt,
        0x3B => VirtualKey::Control,
        0x41 => VirtualKey::NumpadDecimal,
        0x43 => VirtualKey::NumpadMultiply,
        0x45 => VirtualKey::NumpadAdd,
        0x47 => VirtualKey::NumLock,
        0x4B => VirtualKey::NumpadDivide,
        0x4C => VirtualKey::NumpadEnter,
        0x4E => VirtualKey::NumpadSubtract,
        0x51 => VirtualKey::NumpadEqual,
        0x52 => VirtualKey::Numpad0,
        0x53 => VirtualKey::Numpad1,
        0x54 => VirtualKey::Numpad2,
        0x55 => VirtualKey::Numpad3,
        0x56 => VirtualKey::Numpad4,
        0x57 => VirtualKey::Numpad5,
        0x58 => VirtualKey::Numpad6,
        0x59 => VirtualKey::Numpad7,
        0x5B => VirtualKey::Numpad8,
        0x5C => VirtualKey::Numpad9,
        0x60 => VirtualKey::F5,
        0x61 => VirtualKey::F6,
        0x62 => VirtualKey::F7,
        0x63 => VirtualKey::F3,
        0x64 => VirtualKey::F8,
        0x65 => VirtualKey::F9,
        0x67 => VirtualKey::F11,
        0x69 => VirtualKey::F13,
        0x6A => VirtualKey::F16,
        0x6B => VirtualKey::F14,
        0x6D => VirtualKey::F10,
        0x6F => VirtualKey::F12,
        0x71 => VirtualKey::F15,
        0x73 => VirtualKey::Home,
        0x74 => VirtualKey::PageUp,
        0x75 => VirtualKey::Delete,
        0x76 => VirtualKey::F4,
        0x77 => VirtualKey::End,
        0x78 => VirtualKey::F2,
        0x79 => VirtualKey::PageDown,
        0x7A => VirtualKey::F1,
        0x7B => VirtualKey::Left,
        0x7C => VirtualKey::Right,
        0x7D => VirtualKey::Down,
        0x7E => VirtualKey::Up,
        _ => VirtualKey::Unknown(keycode as u32),
    }
}

static CALLBACK: Mutex<Option<Box<dyn Fn(&RawKeyEvent) -> HookResult + Send + 'static>>> = Mutex::new(None);

impl KeyboardHook for MacosKeyboardHook {
    fn install(&mut self, callback: Box<dyn Fn(&RawKeyEvent) -> HookResult + Send>) -> Result<()> {
        {
            let mut cb = CALLBACK.lock().unwrap();
            *cb = Some(callback);
        }

        let tap = CGEventTap::new(
            CGEventTapLocation::HID,
            CGEventTapPlacement::HeadInsertEventTap,
            CGEventTapOptions::ListenOnly,
            vec![
                CGEventType::KeyDown,
                CGEventType::KeyUp,
                CGEventType::FlagsChanged,
            ],
            |_proxy, event_type, event| {
                let keycode = event.get_integer_value_field(EventField::KEYBOARD_EVENT_KEYCODE) as u16;
                let key = keycode_to_virtual_key(keycode);
                let pressed = matches!(event_type, CGEventType::KeyDown);

                let raw_event = RawKeyEvent { key, pressed };

                let cb = CALLBACK.lock().unwrap();
                if let Some(ref callback) = *cb {
                    match callback(&raw_event) {
                        HookResult::Pass => None,
                        HookResult::Suppress => None,
                        HookResult::Replace(_) => None,
                    }
                } else {
                    None
                }
            },
        );

        let tap = match tap {
            Ok(t) => t,
            Err(_) => bail!("Failed to create CGEventTap. Ensure accessibility permission is granted."),
        };

        unsafe {
            let source = tap.mach_port.create_runloop_source(0)
                .expect("Failed to create runloop source");
            let run_loop = CFRunLoop::get_current();
            run_loop.add_source(&source, kCFRunLoopCommonModes);
            tap.enable();
        }

        self.tap = Some(tap);
        Ok(())
    }

    fn uninstall(&mut self) -> Result<()> {
        self.tap = None;
        {
            let mut cb = CALLBACK.lock().unwrap();
            *cb = None;
        }
        Ok(())
    }
}
