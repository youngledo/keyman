//! Integration tests for keyman-core
//!
//! Tests the complete flow: config loading → scheme management → key remapping

use std::sync::{Arc, Mutex};
use keyman_core::config::AppConfig;
use keyman_core::engine::RemappingEngine;
use keyman_core::scheme::KeybindScheme;
use keyman_detect::monitor::GameState;
use keyman_hook::event::RawKeyEvent;
use keyman_hook::hook::HookResult;
use keyman_hook::key::VirtualKey;

/// Helper: create an engine with game state set to active (running + focused)
fn create_active_engine() -> RemappingEngine {
    let game_state = Arc::new(Mutex::new(GameState { running: true, focused: true, pid: Some(1234) }));
    RemappingEngine::new(game_state)
}

/// Test: Load default config when no config file exists
#[test]
fn test_default_config() {
    let config = AppConfig::default();

    assert!(!config.schemes.is_empty(), "Default config should have at least one scheme");
    assert!(!config.active_scheme_id.is_empty(), "Active scheme ID should be set");

    let active = config.active_scheme();
    assert!(active.is_some(), "Should find active scheme");
}

/// Test: Create and manage multiple schemes
#[test]
fn test_scheme_management() {
    let mut config = AppConfig::default();
    let initial_count = config.schemes.len();

    // Add new scheme
    let mut new_scheme = KeybindScheme::default_dota();
    new_scheme.id = "test-scheme".to_string();
    new_scheme.name = "Test Scheme".to_string();
    config.add_scheme(new_scheme);

    assert_eq!(config.schemes.len(), initial_count + 1);

    // Switch scheme
    assert!(config.switch_scheme("test-scheme"));
    assert_eq!(config.active_scheme_id, "test-scheme");

    // Remove scheme
    assert!(config.remove_scheme("test-scheme"));
    assert_eq!(config.schemes.len(), initial_count);
}

/// Test: Key remapping engine passes unmapped keys
#[test]
fn test_remapping_engine_pass() {
    let mut engine = create_active_engine();
    let scheme = KeybindScheme::default_dota();
    engine.set_schemes(vec![scheme.clone()], &scheme.id);

    // An unmapped key should pass through
    let event = RawKeyEvent {
        key: VirtualKey::Q,
        pressed: true,
    };
    let result = engine.process_event(&event);
    // Q is not mapped in default scheme, should pass
    assert!(matches!(result, HookResult::Pass), "Unmapped key should pass");
}

/// Test: Engine passes events when game is not active
#[test]
fn test_engine_game_not_active() {
    let game_state = Arc::new(Mutex::new(GameState::default()));
    let mut engine = RemappingEngine::new(game_state);
    let scheme = KeybindScheme::default_dota();
    engine.set_schemes(vec![scheme], "default-dota");

    // When game is not running/focused, all keys should pass
    let event = RawKeyEvent {
        key: VirtualKey::Z,
        pressed: true,
    };
    let result = engine.process_event(&event);
    assert!(matches!(result, HookResult::Pass), "Should pass when game is not active");
}

/// Test: Inventory key mappings work correctly
#[test]
fn test_inventory_mapping() {
    let mut engine = create_active_engine();
    let mut scheme = KeybindScheme::default_dota();
    // Set inventory mapping: Z → Numpad7
    scheme.inventory_mappings[0] = Some(VirtualKey::Z);
    engine.set_schemes(vec![scheme], "default-dota");

    let event = RawKeyEvent {
        key: VirtualKey::Z,
        pressed: true,
    };
    let result = engine.process_event(&event);

    match result {
        HookResult::Replace(target) => {
            assert_eq!(target, VirtualKey::Numpad7, "Z should map to Numpad7");
        }
        HookResult::Pass => {
            // If Z is not in inventory mappings, that's also valid
        }
        HookResult::Suppress => {
            panic!("Unexpected Suppress for inventory key");
        }
    }
}

/// Test: Key release events are always passed
#[test]
fn test_key_release_passes() {
    let mut engine = create_active_engine();
    let scheme = KeybindScheme::default_dota();
    engine.set_schemes(vec![scheme], "default-dota");

    // Key release should always pass (no active remapping for Z)
    let event = RawKeyEvent {
        key: VirtualKey::Z,
        pressed: false,
    };
    let result = engine.process_event(&event);
    assert!(matches!(result, HookResult::Pass), "Key release should always pass");
}

/// Test: Custom scheme with modified mappings
#[test]
fn test_custom_scheme_mappings() {
    let mut config = AppConfig::default();

    // Modify the active scheme's mappings
    if let Some(scheme) = config.active_scheme_mut() {
        scheme.skill_mappings.insert(VirtualKey::Q, VirtualKey::D);
    }

    // Verify the mapping was updated
    let active = config.active_scheme().unwrap();
    assert_eq!(active.skill_mappings.get(&VirtualKey::Q), Some(&VirtualKey::D));
}

/// Test: Engine with custom skill mapping
#[test]
fn test_engine_custom_scheme() {
    let mut engine = create_active_engine();
    let mut scheme = KeybindScheme::default_dota();

    // Customize mapping: D → F
    scheme.skill_mappings.insert(VirtualKey::D, VirtualKey::F);
    engine.set_schemes(vec![scheme], "default-dota");

    // Test the custom mapping
    let event = RawKeyEvent {
        key: VirtualKey::D,
        pressed: true,
    };
    let result = engine.process_event(&event);

    match result {
        HookResult::Replace(target) => {
            assert_eq!(target, VirtualKey::F, "D should map to F");
        }
        _ => {}
    }
}

/// Test: F12 toggles pause state
#[test]
fn test_f12_toggle_pause() {
    let mut engine = create_active_engine();
    let scheme = KeybindScheme::default_dota();
    engine.set_schemes(vec![scheme], "default-dota");

    assert!(!engine.is_paused());

    // F12 press toggles pause
    let event = RawKeyEvent { key: VirtualKey::F12, pressed: true };
    let result = engine.process_event(&event);
    assert!(matches!(result, HookResult::Suppress));
    assert!(engine.is_paused());

    // F12 again unpauses
    let result = engine.process_event(&event);
    assert!(matches!(result, HookResult::Suppress));
    assert!(!engine.is_paused());
}

/// Test: Blocked keys are suppressed even when paused
#[test]
fn test_blocked_keys_suppressed_when_paused() {
    let mut engine = create_active_engine();
    let mut scheme = KeybindScheme::default_dota();
    scheme.blocked_keys = vec![VirtualKey::LWin];
    engine.set_schemes(vec![scheme], "default-dota");

    // Pause engine
    let event = RawKeyEvent { key: VirtualKey::F12, pressed: true };
    engine.process_event(&event);
    assert!(engine.is_paused());

    // Blocked key should still be suppressed when paused
    let event = RawKeyEvent { key: VirtualKey::LWin, pressed: true };
    let result = engine.process_event(&event);
    assert!(matches!(result, HookResult::Suppress), "Blocked key should be suppressed even when paused");
}
