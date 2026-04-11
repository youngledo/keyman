use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use keyman_hook::key::VirtualKey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeybindScheme {
    pub id: String,
    pub name: String,
    pub skill_mappings: HashMap<VirtualKey, VirtualKey>,
    pub inventory_mappings: [Option<VirtualKey>; 6],
    /// Keys to suppress when game is active (e.g. Win key to prevent accidental Start menu)
    #[serde(default)]
    pub blocked_keys: Vec<VirtualKey>,
    pub toggle_key: VirtualKey,
}

impl KeybindScheme {
    pub fn default_dota() -> Self {
        Self {
            id: "default-dota".to_string(),
            name: crate::i18n::t_scheme_name(1),
            skill_mappings: HashMap::new(),
            inventory_mappings: [None; 6],
            blocked_keys: vec![VirtualKey::LWin, VirtualKey::RWin],
            toggle_key: VirtualKey::F12,
        }
    }
}
