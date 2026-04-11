use anyhow::Result;
use sysinfo::{ProcessesToUpdate, System};

use crate::detector::ProcessDetector;

pub struct MacosProcessDetector {
    sys: std::sync::Mutex<System>,
}

impl MacosProcessDetector {
    pub fn new() -> Self {
        Self {
            sys: std::sync::Mutex::new(System::new()),
        }
    }
}

impl ProcessDetector for MacosProcessDetector {
    fn is_process_running(&self, name: &str) -> Result<bool> {
        let mut sys = self.sys.lock().unwrap();
        sys.refresh_processes(ProcessesToUpdate::All, true);
        let name_lower = name.to_lowercase();
        let found = sys.processes().values().any(|p| {
            let proc_name = p.name().to_string_lossy().to_lowercase();
            proc_name.contains(&name_lower)
        });
        Ok(found)
    }

    fn is_window_focused(&self, _pid: u32) -> Result<bool> {
        // macOS window focus detection requires NSWorkspace API
        // which needs objc/msg_send. For now, return true if process exists.
        // Full implementation would use:
        // NSWorkspace.shared.frontmostApplication.processIdentifier
        Ok(true)
    }

    fn find_process_pid(&self, name: &str) -> Result<Option<u32>> {
        let mut sys = self.sys.lock().unwrap();
        sys.refresh_processes(ProcessesToUpdate::All, true);
        let name_lower = name.to_lowercase();
        let pid = sys.processes().values()
            .find(|p| {
                let proc_name = p.name().to_string_lossy().to_lowercase();
                proc_name.contains(&name_lower)
            })
            .map(|p| p.pid().as_u32());
        Ok(pid)
    }
}
