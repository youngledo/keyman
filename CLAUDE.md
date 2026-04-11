# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Keyman (键盘侠)** is a key remapping tool for Warcraft 3 (Reforged). It remaps skill and inventory hotkeys (the inconvenient Numpad keys) to user-chosen keys, activating only when the WC3 game window is in the foreground. Built with Rust 2024 edition and GPUI (Zed editor's UI framework).

## Build & Test Commands

```bash
cargo build --release          # Build (output: target/release/keyman.exe on Windows)
cargo test                      # Run all tests
cargo test -p keyman-core       # Run tests for a specific crate
cargo test test_default_config  # Run a single test by name
```

No linter or formatter configuration exists (no clippy.toml, rustfmt.toml).

## Workspace Architecture

```
keyman (binary)
├── keyman-ui       → GPUI-based user interface
├── keyman-core     → Remapping engine, config, i18n
│   ├── keyman-hook   → Cross-platform keyboard hook
│   └── keyman-detect → Game process/window detection
├── keyman-hook
└── keyman-detect
```

### Crate Responsibilities

- **keyman-core**: `RemappingEngine` processes `RawKeyEvent` → `HookResult` (Pass/Suppress/Replace). `AppConfig` persists JSON to `~/.config/keyman/config.json`. `i18n` module provides lock-free bilingual (Zh/En) translations with auto-detection.
- **keyman-hook**: Platform-conditional (`cfg(target_os)`) keyboard hooks. `VirtualKey` enum, `RawKeyEvent`, `KeyboardHook` trait. Windows uses `SetWindowsHookExW` + `SendInput` for replacement; macOS uses `CGEventTap` (ListenOnly); Linux uses `evdev` polling.
- **keyman-detect**: `ProcessDetector` trait + `GameDetectionService` (background thread, 2s polling). Detects WC3 processes by name, checks window focus. `GameState` shared via `Arc<Mutex<GameState>>`.
- **keyman-ui**: `KeymanApp` view (~950 lines in `app.rs`). Key capture flow: click cell → capture mode → key press → validate & apply. Manages scheme CRUD, key mapping tables, language toggle.

### Key Data Flow

1. `GameDetectionService` polls → updates shared `GameState`
2. OS keyboard hook fires → callback queries `RemappingEngine::process_event()`
3. Engine checks `GameState` (game focused?), applies mappings from active `KeybindScheme`
4. Returns `HookResult` — hook suppresses/replaces/injects keys accordingly
5. WC3 inventory slots map to: Numpad7, Numpad8, Numpad4, Numpad5, Numpad1, Numpad2

## Platform Implementation Status

- **Windows**: Fully functional (keyboard hook, window focus detection)
- **macOS**: Keyboard hook is listen-only (Replace/Suppress unimplemented); window focus always returns true
- **Linux**: Hook polling works via evdev; Suppress/Replace not yet implemented (needs uinput)

## Notable Patterns

- The keyboard hook handle is intentionally leaked with `std::mem::forget()` — the Windows low-level hook must outlive the GPUI view lifecycle
- Windows key replacement uses a `SIMULATED_KEY_MARKER` (0xDEAD_BEEF) in `dwExtraInfo` to prevent re-processing injected keys
- `ToggleController` is decoupled from the engine for independent pause/resume state management
- `VirtualKey` is Serde-serializable for config persistence; `Unknown(u32)` variant handles unmapped key codes

## Dependencies

GPUI and its components are pulled as git dependencies from `zed-industries/zed` and `huacnlee/gpui-component`. These require a DirectX 11 GPU on Windows.
