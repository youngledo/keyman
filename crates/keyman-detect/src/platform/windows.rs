use anyhow::Result;
use std::sync::Mutex;
use sysinfo::{ProcessesToUpdate, System};
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::detector::ProcessDetector;

pub struct WindowsProcessDetector {
    sys: Mutex<System>,
}

impl WindowsProcessDetector {
    pub fn new() -> Self {
        Self {
            sys: Mutex::new(System::new()),
        }
    }
}

impl ProcessDetector for WindowsProcessDetector {
    fn is_process_running(&self, name: &str) -> Result<bool> {
        // 使用 sysinfo 作为跨平台进程检测的基础
        let mut sys = self.sys.lock().unwrap();
        sys.refresh_processes(ProcessesToUpdate::All, true);
        let name_lower = name.to_lowercase();
        let found = sys.processes().values().any(|p| {
            let proc_name = p.name().to_string_lossy().to_lowercase();
            proc_name.contains(&name_lower)
        });
        Ok(found)
    }

    fn is_window_focused(&self, pid: u32) -> Result<bool> {
        unsafe {
            // 获取当前前台窗口
            let foreground_window = GetForegroundWindow();
            if foreground_window.is_invalid() {
                return Ok(false);
            }

            // 获取前台窗口的进程 ID
            let mut window_pid: u32 = 0;
            let _ = GetWindowThreadProcessId(foreground_window, Some(&mut window_pid));

            Ok(window_pid == pid)
        }
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