use anyhow::Result;
use crate::key::VirtualKey;
use crate::event::RawKeyEvent;

pub enum HookResult {
    Pass,
    Suppress,
    Replace(VirtualKey),
}

pub trait KeyboardHook {
    fn install(&mut self, callback: Box<dyn Fn(&RawKeyEvent) -> HookResult + Send>) -> Result<()>;
    fn uninstall(&mut self) -> Result<()>;
}
