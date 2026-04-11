use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::scheme::KeybindScheme;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            x: 100.0,
            y: 100.0,
            width: 800.0,
            height: 600.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub active_scheme_id: String,
    pub schemes: Vec<KeybindScheme>,
    pub window_state: WindowState,
}

impl Default for AppConfig {
    fn default() -> Self {
        let default_scheme = KeybindScheme::default_dota();
        let active_scheme_id = default_scheme.id.clone();
        Self {
            active_scheme_id,
            schemes: vec![default_scheme],
            window_state: WindowState::default(),
        }
    }
}

impl AppConfig {
    pub fn load_or_default() -> Self {
        Self::load().unwrap_or_default()
    }

    pub fn active_scheme(&self) -> Option<&KeybindScheme> {
        self.schemes.iter().find(|s| s.id == self.active_scheme_id)
    }

    pub fn active_scheme_mut(&mut self) -> Option<&mut KeybindScheme> {
        self.schemes.iter_mut().find(|s| s.id == self.active_scheme_id)
    }

    // --- Scheme CRUD ---

    pub fn add_scheme(&mut self, scheme: KeybindScheme) {
        self.schemes.push(scheme);
    }

    pub fn remove_scheme(&mut self, id: &str) -> bool {
        let len_before = self.schemes.len();
        self.schemes.retain(|s| s.id != id);
        self.schemes.len() < len_before
    }

    pub fn switch_scheme(&mut self, id: &str) -> bool {
        if self.schemes.iter().any(|s| s.id == id) {
            self.active_scheme_id = id.to_string();
            true
        } else {
            false
        }
    }

    // --- Import/Export ---

    pub fn export_scheme(scheme: &KeybindScheme, path: &std::path::Path) -> Result<()> {
        #[derive(Serialize)]
        struct ExportFile {
            version: u32,
            #[serde(flatten)]
            scheme: KeybindScheme,
        }
        let file = ExportFile {
            version: 1,
            scheme: scheme.clone(),
        };
        let content = serde_json::to_string_pretty(&file)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn import_scheme(path: &std::path::Path) -> Result<KeybindScheme> {
        let content = std::fs::read_to_string(path)?;
        let value: serde_json::Value = serde_json::from_str(&content)?;

        // Validate version field
        let version = value.get("version")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        if version != 1 {
            anyhow::bail!("Unsupported scheme file version: {}", version);
        }

        let scheme: KeybindScheme = serde_json::from_value(value)?;
        Ok(scheme)
    }

    // --- Config file I/O ---

    pub fn config_dir() -> std::path::PathBuf {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".config")
            .join("keyman")
    }

    pub fn config_path() -> std::path::PathBuf {
        Self::config_dir().join("config.json")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        let content = std::fs::read_to_string(&path)?;
        let config: AppConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let dir = Self::config_dir();
        std::fs::create_dir_all(&dir)?;
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(Self::config_path(), content)?;
        Ok(())
    }
}
