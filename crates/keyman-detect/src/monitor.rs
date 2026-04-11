use std::sync::{Arc, Mutex};

use crate::detector::ProcessDetector;

/// WC3 process names to detect
const WC3_PROCESS_NAMES: &[&str] = &[
    "war3",
    "Warcraft III",
    "Warcraft III.exe",
    "warcraft_iii",
    "Warcraft III.app", // macOS/Wine wrapper
];

/// Game state shared between threads
#[derive(Clone, Default)]
pub struct GameState {
    pub running: bool,
    pub focused: bool,
    pub pid: Option<u32>,
}

pub struct GameMonitor {
    state: Arc<Mutex<GameState>>,
    detector: Box<dyn ProcessDetector>,
}

impl GameMonitor {
    pub fn new(detector: Box<dyn ProcessDetector>) -> Self {
        Self {
            state: Arc::new(Mutex::new(GameState::default())),
            detector,
        }
    }

    /// Get a cloneable handle to the game state for UI updates
    pub fn state_handle(&self) -> Arc<Mutex<GameState>> {
        self.state.clone()
    }

    pub fn is_game_active(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.running && state.focused
    }

    pub fn is_game_running(&self) -> bool {
        self.state.lock().unwrap().running
    }

    pub fn is_game_focused(&self) -> bool {
        self.state.lock().unwrap().focused
    }

    /// Check if WC3 is running, store PID if found
    pub fn check_process(&mut self) -> bool {
        for name in WC3_PROCESS_NAMES {
            if let Ok(Some(pid)) = self.detector.find_process_pid(name) {
                let mut state = self.state.lock().unwrap();
                state.running = true;
                state.pid = Some(pid);
                return true;
            }
        }
        let mut state = self.state.lock().unwrap();
        state.running = false;
        state.pid = None;
        false
    }

    /// Check if a WC3 window is focused (real foreground window detection)
    pub fn check_focus(&mut self) -> bool {
        let mut state = self.state.lock().unwrap();
        if let Some(pid) = state.pid {
            if let Ok(focused) = self.detector.is_window_focused(pid) {
                state.focused = focused;
                return focused;
            }
        }
        state.focused = false;
        false
    }

    /// Perform a full check (process + focus)
    pub fn check(&mut self) -> (bool, bool) {
        let running = self.check_process();
        let focused = if running { self.check_focus() } else { false };
        (running, focused)
    }
}

/// Background game detection service
pub struct GameDetectionService {
    state: Arc<Mutex<GameState>>,
}

impl GameDetectionService {
    /// Create a new service that runs detection in background
    pub fn start() -> Self {
        let state = Arc::new(Mutex::new(GameState::default()));
        let state_clone = state.clone();

        std::thread::spawn(move || {
            let mut monitor = GameMonitor::new(crate::create_detector());
            monitor.state = state_clone.clone();

            loop {
                std::thread::sleep(std::time::Duration::from_secs(2));
                monitor.check();
            }
        });

        Self { state }
    }

    /// Get current game state
    pub fn get_state(&self) -> GameState {
        self.state.lock().unwrap().clone()
    }

    /// Get a handle to the state for sharing
    pub fn state_handle(&self) -> Arc<Mutex<GameState>> {
        self.state.clone()
    }
}
