use keyman_hook::event::RawKeyEvent;
use keyman_hook::key::VirtualKey;

pub struct ToggleController {
    enabled: bool,
    toggle_key: VirtualKey,
    game_active: bool,
}

impl ToggleController {
    pub fn new(toggle_key: VirtualKey) -> Self {
        Self {
            enabled: false,
            toggle_key,
            game_active: false,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_game_active(&mut self, active: bool) {
        self.game_active = active;
    }

    pub fn set_toggle_key(&mut self, key: VirtualKey) {
        self.toggle_key = key;
    }

    /// Process a key event. Returns true if toggle state changed.
    pub fn process_event(&mut self, event: &RawKeyEvent) -> bool {
        if !self.game_active {
            return false;
        }

        if event.key == self.toggle_key && event.pressed {
            self.enabled = !self.enabled;
            return true;
        }

        false
    }
}
