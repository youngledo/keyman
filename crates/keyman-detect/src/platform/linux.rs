//! Linux process and window detection implementation
//!
//! Process detection uses `/proc` filesystem (available on all Linux systems).
//! Window focus detection uses X11 via xcb (for X11 sessions) or falls back to
//! a simple heuristic for Wayland sessions.

use anyhow::Result;
use std::sync::Mutex;
use sysinfo::{ProcessesToUpdate, System};
use xcb::x::{Atom, GetProperty, GetPropertyReply};
use xcb::{Xid, XidNew};

use crate::detector::ProcessDetector;

/// WC3 process names to look for
const WC3_PROCESS_NAMES: &[&str] = &[
    "war3.exe",
    "Warcraft III.exe",
    "warcraft3",
    "Warcraft III.app", // Wine wrapper
];

pub struct LinuxProcessDetector {
    sys: Mutex<System>,
}

impl LinuxProcessDetector {
    pub fn new() -> Self {
        Self {
            sys: Mutex::new(System::new()),
        }
    }

    /// Get the PID of the foreground window (X11 only)
    fn get_active_window_pid(&self) -> Result<u32> {
        // Connect to X11 display
        let (conn, screen_num) = xcb::Connection::connect(None)?;
        let setup = conn.get_setup();
        let screen = setup.roots().nth(screen_num as usize)
            .ok_or_else(|| anyhow::anyhow!("No screen found"))?;

        let net_active_window = self.atom_name_to_id(&conn, "_NET_ACTIVE_WINDOW")?;
        let net_wm_pid = self.atom_name_to_id(&conn, "_NET_WM_PID")?;

        // Get _NET_ACTIVE_WINDOW property
        let active_window_atom = conn.send_request(&GetProperty {
            delete: false,
            window: screen.root(),
            property: Atom::new(net_active_window),
            r#type: xcb::x::ATOM_WINDOW,
            long_offset: 0,
            long_length: 1,
        });

        let reply: GetPropertyReply = conn.wait_for_reply(active_window_atom)?;
        let window = reply.value::<u32>()
            .first()
            .copied()
            .ok_or_else(|| anyhow::anyhow!("No active window"))?;

        // Get _NET_WM_PID property of the active window
        let pid_atom = conn.send_request(&GetProperty {
            delete: false,
            window: xcb::x::Window::new(window),
            property: Atom::new(net_wm_pid),
            r#type: xcb::x::ATOM_CARDINAL,
            long_offset: 0,
            long_length: 1,
        });

        let pid_reply: GetPropertyReply = conn.wait_for_reply(pid_atom)?;
        let pid = pid_reply.value::<u32>()
            .first()
            .copied()
            .ok_or_else(|| anyhow::anyhow!("No PID for active window"))?;

        Ok(pid)
    }

    /// Convert atom name to ID
    fn atom_name_to_id(&self, conn: &xcb::Connection, name: &str) -> Result<u32> {
        let cookie = conn.send_request(&xcb::x::InternAtom {
            only_if_exists: true,
            name: name.as_bytes(),
        });
        let reply = conn.wait_for_reply(cookie)?;
        Ok(reply.atom().resource_id())
    }
}

impl ProcessDetector for LinuxProcessDetector {
    fn is_process_running(&self, name: &str) -> Result<bool> {
        // Use sysinfo as cross-platform base for process detection
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
        // Try X11 first
        match self.get_active_window_pid() {
            Ok(active_pid) => {
                return Ok(active_pid == pid);
            }
            Err(_) => {
                // X11 not available (possibly Wayland session)
                // Wayland doesn't allow applications to query the active window
                // for security reasons. Fall back to just checking if process exists.
                // This is a limitation of Wayland's security model.
            }
        }

        // Fallback for Wayland or X11 failure
        // Just check if the process is running
        let mut sys = self.sys.lock().unwrap();
        sys.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            sysinfo::ProcessRefreshKind::nothing(),
        );

        Ok(sys.processes().values().any(|p| p.pid().as_u32() == pid))
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

impl Default for LinuxProcessDetector {
    fn default() -> Self {
        Self::new()
    }
}
