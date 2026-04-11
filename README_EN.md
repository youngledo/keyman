**English** | **[中文](README.md)**

# Keyman

A key remapping tool for Warcraft 3 — customize skill and inventory hotkeys.

## Features

- Custom skill and inventory key mappings (add/remove dynamically)
- Multiple scheme management (create, switch, rename, delete)
- Auto-detect game window, only active when game is in foreground
- F11 to switch scheme in-game, F12 to pause/resume remapping
- Block Win key to prevent accidental presses
- Bilingual UI (Chinese/English), follows system language with manual toggle
- Cross-platform support (Windows, macOS, Linux)

## Installation

### Windows

**Requirements: Windows 10 v1809 or later, GPU with DirectX 11 support.**

Download `keyman.exe` from [Releases](../../releases) and run it.

### macOS

On first launch, grant keyboard monitoring access in **System Settings > Privacy & Security > Accessibility**.

### Linux

```bash
sudo usermod -a -G input $USER
```

## Build

```bash
cargo build --release
```

Windows output: `target/release/keyman.exe`

## Usage

1. Click a cell in the Skills/Inventory table, then press the key you want to map
2. Press Delete/Backspace to clear a mapping
3. Check "Block Win Key" at the bottom
4. Press F12 in-game to pause/resume remapping, F11 to switch scheme
5. Click the language button in the top-right to toggle Chinese/English UI

## Project Structure

```
src/main.rs               # Application entry
crates/
  keyman-core/            # Remapping engine, config, i18n
  keyman-hook/            # Keyboard hook (cross-platform)
  keyman-detect/          # Game process and window detection
  keyman-ui/              # User interface (GPUI)
assets/                   # App icons
```

## Tech Stack

- [Rust](https://www.rust-lang.org/)
- [GPUI](https://github.com/zed-industries/zed) - GPU-accelerated UI framework
- [gpui-component](https://github.com/huacnlee/gpui-component) - UI component library

## License

MIT License
