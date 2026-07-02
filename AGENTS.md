## Short project description
Allium is a custom game launcher and backend daemon for Miyoo handheld devices, restructured to support both native Miyoo, macOS Simulator, and Minime Buildroot platform targets.

## Project folder structure
- `crates/`
  - `activity-tracker/` — User activity and gameplay logs tracker.
  - `allium-launcher/` — The main graphical user interface for Allium.
  - `allium-menu/` — Quick settings and system in-game overlay menu.
  - `alliumd/` — Background daemon handling system state, power management, and updates.
  - `common/` — Shared libraries, platform abstractions, hardware interfaces, and utilities.
  - `ffi/` — Foreign Function Interface bindings for Miyoo system calls.
  - `libretro/` — Bindings generator for the Libretro API.
  - `myctl/` — Miyoo hardware control tool for brightness, volume, and display adjustments.
  - `play/` — Standard standalone emulator player and RetroArch launcher frontend.
  - `say/`, `screenshot/`, `screenshot-viewer/`, `show/` — Ancillary helper utilities.
- `static/` — Assets, localizations, themes, default RetroArch configurations, and device migration files.
- `third-party/` — External dependencies and patches for building dufs, collie, and RetroArch.
- `toolchain/` — Docker configurations for cross-compiling to the armv7-unknown-linux-gnueabihf target.
- `Makefile` — Build system wrapper coordinating compilation, packaging, and deployments.

## Agent directives
- Emphasize safety, reliability, and correctness: compilation warnings must be fixed at the root cause, never bypassed or silenced.
- Code changes relating to Minime or Play should be protected using conditional compilation (`#[cfg(feature = "minime")]`).
- Never introduce breaking changes or refactoring to upstream Miyoo/Simulator code.
- Ensure all quality gates (`cargo clippy`, `cargo fmt`, and workspace tests) pass cleanly before finalizing work.
- Always check that any modified code is covered by tests where applicable.
