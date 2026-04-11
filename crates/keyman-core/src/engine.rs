use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use keyman_hook::event::RawKeyEvent;
use keyman_hook::key::VirtualKey;
use keyman_hook::hook::HookResult;
use crate::scheme::KeybindScheme;
use keyman_detect::monitor::GameState;

/// WC3 inventory slot numpad keys: slots 1-6 correspond to
/// Numpad7, Numpad8, Numpad4, Numpad5, Numpad1, Numpad2
const INVENTORY_NUMPAD_KEYS: [VirtualKey; 6] = [
    VirtualKey::Numpad7,
    VirtualKey::Numpad8,
    VirtualKey::Numpad4,
    VirtualKey::Numpad5,
    VirtualKey::Numpad1,
    VirtualKey::Numpad2,
];

pub struct RemappingEngine {
    /// All available schemes
    schemes: Vec<KeybindScheme>,
    /// Index of the active scheme
    active_index: usize,
    /// In-game pause state (F12 toggle) - pauses remapping but keeps blocked keys active
    paused: bool,
    /// Shared game state from detection service
    game_state: Arc<Mutex<GameState>>,
    /// Tracks currently pressed source keys that have been remapped,
    /// so their release events can also be replaced.
    active_remappings: HashMap<VirtualKey, VirtualKey>,
}

impl RemappingEngine {
    pub fn new(game_state: Arc<Mutex<GameState>>) -> Self {
        Self {
            schemes: Vec::new(),
            active_index: 0,
            paused: false,
            game_state,
            active_remappings: HashMap::new(),
        }
    }

    pub fn set_schemes(&mut self, schemes: Vec<KeybindScheme>, active_id: &str) {
        self.active_index = schemes.iter().position(|s| s.id == active_id).unwrap_or(0);
        self.schemes = schemes;
    }

    /// Set the active scheme by id (from UI)
    pub fn set_active_scheme(&mut self, id: &str) {
        if let Some(idx) = self.schemes.iter().position(|s| s.id == id) {
            self.active_index = idx;
            self.active_remappings.clear();
        }
    }

    /// Get the id of the currently active scheme
    pub fn active_scheme_id(&self) -> Option<&str> {
        self.schemes.get(self.active_index).map(|s| s.id.as_str())
    }

    /// Whether the scheme was changed in-game (F11), returns the new id if changed
    pub fn take_scheme_change(&mut self) -> Option<String> {
        // This is checked by UI on render to detect F11 changes
        None
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Whether the game is currently running and focused
    fn is_game_active(&self) -> bool {
        let state = self.game_state.lock().unwrap();
        state.running && state.focused
    }

    /// Cycle to the next scheme, returns the new active scheme id
    fn cycle_scheme(&mut self) -> &str {
        if self.schemes.len() > 1 {
            self.active_index = (self.active_index + 1) % self.schemes.len();
            self.active_remappings.clear();
        }
        self.schemes[self.active_index].id.as_str()
    }

    pub fn process_event(&mut self, event: &RawKeyEvent) -> HookResult {
        if !self.is_game_active() {
            return HookResult::Pass;
        }

        // Handle toggle key: F12 toggles pause state, suppress the key
        if event.pressed && event.key == VirtualKey::F12 {
            self.paused = !self.paused;
            return HookResult::Suppress;
        }

        // Handle scheme switch: F11 cycles to next scheme
        if event.pressed && event.key == VirtualKey::F11 {
            self.cycle_scheme();
            return HookResult::Suppress;
        }

        let Some(scheme) = self.schemes.get(self.active_index) else {
            return HookResult::Pass;
        };

        // Always suppress blocked keys when game is active (even when paused)
        if event.pressed && scheme.blocked_keys.contains(&event.key) {
            return HookResult::Suppress;
        }

        if self.paused {
            return HookResult::Pass;
        }

        if event.pressed {
            // Key down: check mappings
            // Check skill mappings
            if let Some(target) = scheme.skill_mappings.get(&event.key) {
                self.active_remappings.insert(event.key, *target);
                return HookResult::Replace(*target);
            }

            // Check inventory mappings: source key → numpad key
            for (i, inv_key) in scheme.inventory_mappings.iter().enumerate() {
                if let Some(inv) = inv_key {
                    if *inv == event.key {
                        let target = INVENTORY_NUMPAD_KEYS[i];
                        self.active_remappings.insert(event.key, target);
                        return HookResult::Replace(target);
                    }
                }
            }

            HookResult::Pass
        } else {
            // Key up: if this source key was previously remapped, replace the release too
            if let Some(target) = self.active_remappings.remove(&event.key) {
                return HookResult::Replace(target);
            }
            HookResult::Pass
        }
    }
}

/// Thread-safe wrapper for RemappingEngine, used by the keyboard hook callback.
pub type SharedEngine = Arc<Mutex<RemappingEngine>>;
