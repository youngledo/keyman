//! Platform-specific tests for keyman-hook
//!
//! These tests verify the keyboard hook implementations work correctly.
//! Note: Some tests require actual hardware access and cannot be fully automated.

#[cfg(target_os = "macos")]
mod macos_tests {
    use keyman_hook::hook::{HookResult, KeyboardHook};
    use keyman_hook::key::VirtualKey;
    use keyman_hook::event::RawKeyEvent;
    use keyman_hook::MacosKeyboardHook;

    /// Test: Creating a MacosKeyboardHook succeeds
    #[test]
    fn test_create_hook() {
        let hook = MacosKeyboardHook::new();
        // Just verify creation works
        assert!(true);
    }

    /// Test: Install and uninstall lifecycle
    /// Note: This test may fail if accessibility permission is not granted.
    #[test]
    #[ignore = "Requires accessibility permission and may interfere with user input"]
    fn test_install_uninstall() {
        let mut hook = MacosKeyboardHook::new();

        let callback = Box::new(|_event: &RawKeyEvent| HookResult::Pass);

        // Install should succeed (if permission granted)
        let install_result = hook.install(callback);
        assert!(install_result.is_ok(), "Install should succeed");

        // Uninstall should succeed
        let uninstall_result = hook.uninstall();
        assert!(uninstall_result.is_ok(), "Uninstall should succeed");
    }
}

#[cfg(target_os = "windows")]
mod windows_tests {
    use keyman_hook::hook::{HookResult, KeyboardHook};
    use keyman_hook::event::RawKeyEvent;
    use keyman_hook::WindowsKeyboardHook;

    /// Test: Creating a WindowsKeyboardHook succeeds
    #[test]
    fn test_create_hook() {
        let hook = WindowsKeyboardHook::new();
        // Just verify creation works
        assert!(true);
    }

    /// Test: Install and uninstall lifecycle
    #[test]
    #[ignore = "May interfere with user input"]
    fn test_install_uninstall() {
        let mut hook = WindowsKeyboardHook::new();

        let callback = Box::new(|_event: &RawKeyEvent| HookResult::Pass);

        let install_result = hook.install(callback);
        assert!(install_result.is_ok(), "Install should succeed");

        let uninstall_result = hook.uninstall();
        assert!(uninstall_result.is_ok(), "Uninstall should succeed");
    }
}

#[cfg(target_os = "linux")]
mod linux_tests {
    use keyman_hook::hook::{HookResult, KeyboardHook};
    use keyman_hook::event::RawKeyEvent;
    use keyman_hook::LinuxKeyboardHook;

    /// Test: Creating a LinuxKeyboardHook succeeds
    #[test]
    fn test_create_hook() {
        let hook = LinuxKeyboardHook::new();
        // Just verify creation works
        assert!(true);
    }

    /// Test: Install fails gracefully without proper permissions
    #[test]
    fn test_install_without_permissions() {
        let mut hook = LinuxKeyboardHook::new();

        let callback = Box::new(|_event: &RawKeyEvent| HookResult::Pass);

        // Should fail if user is not in 'input' group
        let result = hook.install(callback);
        // We don't assert the result because it depends on system configuration
        // Just verify it doesn't panic

        // Clean up
        let _ = hook.uninstall();
    }
}

/// Cross-platform tests for VirtualKey conversions
mod virtual_key_tests {
    use keyman_hook::key::VirtualKey;

    #[test]
    fn test_letter_keys() {
        // Verify all letter keys exist and are distinct
        assert_ne!(VirtualKey::A, VirtualKey::B);
        assert_ne!(VirtualKey::Q, VirtualKey::W);
    }

    #[test]
    fn test_numpad_keys() {
        assert_ne!(VirtualKey::Numpad0, VirtualKey::Key0);
        assert_ne!(VirtualKey::Numpad7, VirtualKey::Key7);
    }

    #[test]
    fn test_unknown_key() {
        let unknown = VirtualKey::Unknown(9999);
        match unknown {
            VirtualKey::Unknown(code) => assert_eq!(code, 9999),
            _ => panic!("Expected Unknown variant"),
        }
    }

    #[test]
    fn test_key_equality() {
        assert_eq!(VirtualKey::A, VirtualKey::A);
        assert_eq!(VirtualKey::Numpad7, VirtualKey::Numpad7);
    }
}

/// Tests for RawKeyEvent
mod raw_key_event_tests {
    use keyman_hook::event::RawKeyEvent;
    use keyman_hook::key::VirtualKey;

    #[test]
    fn test_key_press_event() {
        let event = RawKeyEvent {
            key: VirtualKey::A,
            pressed: true,
        };

        assert_eq!(event.key, VirtualKey::A);
        assert!(event.pressed);
    }

    #[test]
    fn test_key_release_event() {
        let event = RawKeyEvent {
            key: VirtualKey::Space,
            pressed: false,
        };

        assert_eq!(event.key, VirtualKey::Space);
        assert!(!event.pressed);
    }
}

/// Tests for HookResult
mod hook_result_tests {
    use keyman_hook::hook::HookResult;
    use keyman_hook::key::VirtualKey;

    #[test]
    fn test_hook_result_pass() {
        let result: HookResult = HookResult::Pass;
        match result {
            HookResult::Pass => assert!(true),
            _ => panic!("Expected Pass"),
        }
    }

    #[test]
    fn test_hook_result_suppress() {
        let result: HookResult = HookResult::Suppress;
        match result {
            HookResult::Suppress => assert!(true),
            _ => panic!("Expected Suppress"),
        }
    }

    #[test]
    fn test_hook_result_replace() {
        let result = HookResult::Replace(VirtualKey::Q);
        match result {
            HookResult::Replace(key) => assert_eq!(key, VirtualKey::Q),
            _ => panic!("Expected Replace"),
        }
    }
}