use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VirtualKey {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
    // Numbers
    Key0, Key1, Key2, Key3, Key4, Key5, Key6, Key7, Key8, Key9,
    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    F13, F14, F15, F16, F17, F18, F19,
    // Numpad
    Numpad0, Numpad1, Numpad2, Numpad3, Numpad4,
    Numpad5, Numpad6, Numpad7, Numpad8, Numpad9,
    NumpadAdd, NumpadSubtract, NumpadMultiply, NumpadDivide,
    NumpadEnter, NumpadDecimal, NumpadEqual,
    // Special keys
    Space, Enter, Escape, Tab, Backspace, Delete, Insert,
    Home, End, PageUp, PageDown, Pause,
    // Modifiers
    Shift, Control, Alt, Command, CapsLock,
    ScrollLock, NumLock,
    // Windows keys
    LWin, RWin,
    // Arrow keys
    Up, Down, Left, Right,
    // Punctuation & symbols
    Minus, Equal, LeftBracket, RightBracket,
    Backslash, Semicolon, Quote, Comma, Period, Slash,
    // Media
    VolumeUp, VolumeDown, Mute,
    // Other
    Unknown(u32),
}
