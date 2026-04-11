pub mod detector;
pub mod monitor;
pub mod platform;

// Re-export platform-specific detector for convenience
#[cfg(target_os = "macos")]
pub use platform::MacosProcessDetector;

#[cfg(target_os = "windows")]
pub use platform::WindowsProcessDetector;

#[cfg(target_os = "linux")]
pub use platform::LinuxProcessDetector;

// Re-export monitor types
pub use monitor::GameMonitor;
pub use monitor::GameDetectionService;
pub use monitor::GameState;

/// Create a platform-specific process detector
#[cfg(target_os = "macos")]
pub fn create_detector() -> Box<dyn detector::ProcessDetector> {
    Box::new(MacosProcessDetector::new())
}

#[cfg(target_os = "windows")]
pub fn create_detector() -> Box<dyn detector::ProcessDetector> {
    Box::new(WindowsProcessDetector::new())
}

#[cfg(target_os = "linux")]
pub fn create_detector() -> Box<dyn detector::ProcessDetector> {
    Box::new(LinuxProcessDetector::new())
}
