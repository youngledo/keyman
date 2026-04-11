use anyhow::Result;

pub trait ProcessDetector {
    fn is_process_running(&self, name: &str) -> Result<bool>;
    fn is_window_focused(&self, pid: u32) -> Result<bool>;
    fn find_process_pid(&self, name: &str) -> Result<Option<u32>>;
}
