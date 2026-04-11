//! Tests for scheme import/export functionality

use keyman_core::config::AppConfig;
use keyman_core::scheme::KeybindScheme;
use keyman_hook::key::VirtualKey;
use std::io::Write;
use tempfile::NamedTempFile;

/// Test: Export scheme to JSON file
#[test]
fn test_export_scheme() {
    let scheme = KeybindScheme::default_dota();

    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let path = temp_file.path();

    let result = AppConfig::export_scheme(&scheme, path);
    assert!(result.is_ok(), "Export should succeed");

    // Verify file content
    let content = std::fs::read_to_string(path).expect("Failed to read exported file");
    assert!(content.contains("\"version\": 1"), "Should contain version field");
    assert!(content.contains(&scheme.id), "Should contain scheme id");
    assert!(content.contains(&scheme.name), "Should contain scheme name");
}

/// Test: Import scheme from JSON file
#[test]
fn test_import_scheme() {
    // First export
    let scheme = KeybindScheme::default_dota();
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let path = temp_file.path();

    AppConfig::export_scheme(&scheme, path).expect("Export failed");

    // Then import
    let imported = AppConfig::import_scheme(path);
    assert!(imported.is_ok(), "Import should succeed: {:?}", imported.err());

    let imported_scheme = imported.unwrap();
    assert_eq!(imported_scheme.id, scheme.id);
    assert_eq!(imported_scheme.name, scheme.name);
}

/// Test: Import rejects invalid version
#[test]
fn test_import_invalid_version() {
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");

    // Write invalid version
    let invalid_content = r#"{
        "version": 999,
        "id": "test",
        "name": "Test",
        "skill_mappings": {},
        "inventory_mappings": [null, null, null, null, null, null],
        "toggle_key": "F12",
        "blocked_keys": []
    }"#;
    temp_file.write_all(invalid_content.as_bytes()).expect("Failed to write");

    let result = AppConfig::import_scheme(temp_file.path());
    assert!(result.is_err(), "Should reject unsupported version");
}

/// Test: Import handles missing version (defaults to 0, should fail)
#[test]
fn test_import_missing_version() {
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");

    // Missing version field
    let invalid_content = r#"{
        "id": "test",
        "name": "Test",
        "skill_mappings": {},
        "inventory_mappings": [null, null, null, null, null, null],
        "toggle_key": "F12",
        "blocked_keys": []
    }"#;
    temp_file.write_all(invalid_content.as_bytes()).expect("Failed to write");

    let result = AppConfig::import_scheme(temp_file.path());
    assert!(result.is_err(), "Should reject missing version");
}

/// Test: Import handles malformed JSON
#[test]
fn test_import_malformed_json() {
    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");

    let malformed = r#"{ not valid json }"#;
    temp_file.write_all(malformed.as_bytes()).expect("Failed to write");

    let result = AppConfig::import_scheme(temp_file.path());
    assert!(result.is_err(), "Should reject malformed JSON");
}

/// Test: Round-trip export/import preserves all data
#[test]
fn test_round_trip() {
    let mut scheme = KeybindScheme::default_dota();

    // Customize the scheme
    scheme.name = "Custom Scheme".to_string();
    scheme.skill_mappings.insert(VirtualKey::D, VirtualKey::F);
    scheme.skill_mappings.insert(VirtualKey::F, VirtualKey::G);
    scheme.inventory_mappings = [
        Some(VirtualKey::A),
        Some(VirtualKey::S),
        Some(VirtualKey::D),
        Some(VirtualKey::F),
        Some(VirtualKey::G),
        Some(VirtualKey::H),
    ];

    // Export
    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    let path = temp_file.path();
    AppConfig::export_scheme(&scheme, path).expect("Export failed");

    // Import
    let imported = AppConfig::import_scheme(path).expect("Import failed");

    // Verify all data preserved
    assert_eq!(imported.id, scheme.id);
    assert_eq!(imported.name, scheme.name);
    assert_eq!(imported.skill_mappings, scheme.skill_mappings);
    assert_eq!(imported.inventory_mappings, scheme.inventory_mappings);
}

/// Test: Add imported scheme to config
#[test]
fn test_import_into_config() {
    // Create and export a scheme
    let mut scheme = KeybindScheme::default_dota();
    scheme.id = "imported-scheme".to_string();
    scheme.name = "Imported Scheme".to_string();

    let temp_file = NamedTempFile::new().expect("Failed to create temp file");
    AppConfig::export_scheme(&scheme, temp_file.path()).expect("Export failed");

    // Import into config
    let mut config = AppConfig::default();
    let initial_count = config.schemes.len();

    let imported = AppConfig::import_scheme(temp_file.path()).expect("Import failed");
    config.add_scheme(imported);

    assert_eq!(config.schemes.len(), initial_count + 1);
    assert!(config.schemes.iter().any(|s| s.id == "imported-scheme"));
}
