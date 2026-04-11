use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Language {
    Zh,
    En,
}

static LANG: AtomicU8 = AtomicU8::new(Language::Zh as u8);

/// Initialize i18n by detecting system language. Call once at startup.
pub fn init() {
    let lang = detect_language();
    LANG.store(lang as u8, Ordering::Relaxed);
}

/// Get the current language.
pub fn lang() -> Language {
    match LANG.load(Ordering::Relaxed) {
        1 => Language::En,
        _ => Language::Zh,
    }
}

/// Set the language at runtime (for UI toggle).
pub fn set_lang(lang: Language) {
    LANG.store(lang as u8, Ordering::Relaxed);
}

/// Toggle between Chinese and English, returns the new language.
pub fn toggle_lang() -> Language {
    let current = lang();
    let next = match current {
        Language::Zh => Language::En,
        Language::En => Language::Zh,
    };
    LANG.store(next as u8, Ordering::Relaxed);
    next
}

/// Label to show on the toggle button (shows the *other* language).
pub fn toggle_label() -> &'static str {
    match lang() {
        Language::Zh => "EN",
        Language::En => "中文",
    }
}

/// Translate a key to the current language.
pub fn t(key: &str) -> &'static str {
    match lang() {
        Language::Zh => match key {
            "scheme" => "方案:",
            "new" => "新建",
            "delete" => "删除",
            "rename" => "重命名",
            "rename_scheme" => "重命名方案",
            "cancel" => "取消",
            "confirm" => "确定",
            "skill" => "技能",
            "inventory" => "物品栏",
            "key" => "按键",
            "map_to" => "映射到",
            "inv_slot" => "物品栏",
            "block_win" => "屏蔽 Win 键",
            "f11_switch" => "F11 切换方案",
            "f12_pause" => "F12 暂停改键",
            "f12_toggle" => "(F12 切换)",
            "enabled" => "已启用",
            "in_game" => "游戏中",
            "paused" => "已暂停",
            "no_active_scheme" => "无活动方案",
            _ => "",
        },
        Language::En => match key {
            "scheme" => "Scheme:",
            "new" => "New",
            "delete" => "Delete",
            "rename" => "Rename",
            "rename_scheme" => "Rename Scheme",
            "cancel" => "Cancel",
            "confirm" => "OK",
            "skill" => "Skills",
            "inventory" => "Inventory",
            "key" => "Key",
            "map_to" => "Map To",
            "inv_slot" => "Slot",
            "block_win" => "Block Win Key",
            "f11_switch" => "F11 Switch Scheme",
            "f12_pause" => "F12 Pause Remap",
            "f12_toggle" => "(F12 Toggle)",
            "enabled" => "Enabled",
            "in_game" => "In Game",
            "paused" => "Paused",
            "no_active_scheme" => "No active scheme",
            _ => "",
        },
    }
}

// --- Formatted message helpers ---

pub fn t_key_used(key_name: &str) -> String {
    match lang() {
        Language::Zh => format!("按键 {} 已被使用，请选择其他按键", key_name),
        Language::En => format!("Key {} is already in use", key_name),
    }
}

pub fn t_cannot_delete_last() -> String {
    match lang() {
        Language::Zh => "无法删除最后一个方案".into(),
        Language::En => "Cannot delete the last scheme".into(),
    }
}

pub fn t_scheme_exists(name: &str) -> String {
    match lang() {
        Language::Zh => format!("方案名 \"{}\" 已存在", name),
        Language::En => format!("Scheme name \"{}\" already exists", name),
    }
}

pub fn t_unsupported_key(key: &str) -> String {
    match lang() {
        Language::Zh => format!("不支持的按键: {}", key),
        Language::En => format!("Unsupported key: {}", key),
    }
}

pub fn t_scheme_name(idx: usize) -> String {
    match lang() {
        Language::Zh => format!("方案 {}", idx),
        Language::En => format!("Scheme {}", idx),
    }
}

pub fn t_app_title() -> String {
    match lang() {
        Language::Zh => "键盘侠 (Keyman)".into(),
        Language::En => "Keyman".into(),
    }
}

fn detect_language() -> Language {
    #[cfg(target_os = "windows")]
    {
        let lang_id = unsafe { GetUserDefaultUILanguage() };
        let primary = lang_id & 0x3FF;
        if primary == 0x04 {
            Language::Zh
        } else {
            Language::En
        }
    }

    #[cfg(target_os = "macos")]
    {
        // Check LANG env var first
        if let Ok(lang) = std::env::var("LANG") {
            if lang.starts_with("zh") {
                return Language::Zh;
            }
        }
        // Fallback: read macOS system locale
        if let Ok(output) = std::process::Command::new("defaults")
            .args(["read", "-g", "AppleLocale"])
            .output()
        {
            let locale = String::from_utf8_lossy(&output.stdout);
            if locale.starts_with("zh") {
                return Language::Zh;
            }
        }
        Language::En
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        match std::env::var("LANG") {
            Ok(lang) if lang.starts_with("zh") => Language::Zh,
            _ => Language::En,
        }
    }
}

#[cfg(target_os = "windows")]
unsafe extern "system" {
    fn GetUserDefaultUILanguage() -> u16;
}
