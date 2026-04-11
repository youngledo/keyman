use anyhow::Result;

/// System tray abstraction for status indicator.
#[derive(Clone, Copy, PartialEq)]
pub enum TrayStatus {
    Disabled,
    Enabled,
    InGame,
}

pub trait SystemTray {
    fn create(&mut self) -> Result<()>;
    fn set_status(&mut self, status: TrayStatus) -> Result<()>;
    fn set_tooltip(&mut self, text: &str) -> Result<()>;
    fn destroy(&mut self) -> Result<()>;
}

/// Stub tray for platforms without native tray support.
pub struct StubTray;

impl SystemTray for StubTray {
    fn create(&mut self) -> Result<()> { Ok(()) }
    fn set_status(&mut self, _status: TrayStatus) -> Result<()> { Ok(()) }
    fn set_tooltip(&mut self, _text: &str) -> Result<()> { Ok(()) }
    fn destroy(&mut self) -> Result<()> { Ok(()) }
}

#[cfg(target_os = "macos")]
pub fn create_tray() -> Box<dyn SystemTray> {
    // macOS tray would use NSStatusItem via objc2
    // For now, use stub until objc2-app-kit integration is added
    Box::new(StubTray)
}

#[cfg(not(target_os = "macos"))]
pub fn create_tray() -> Box<dyn SystemTray> {
    Box::new(StubTray)
}
