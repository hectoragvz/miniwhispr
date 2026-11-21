# Tauri + React for a Mac-only v1

v1 targets macOS only, which on its own would point to a native Swift app with the simplest access to global hotkeys, the microphone, and text insertion. We chose Tauri (Rust core, React UI for the Toast, first-run setup, and windows) instead, because Windows support is planned and a shared core plus a shared UI avoids a rewrite. OS-specific work (global shortcut capture, simulated paste, permissions) lives in small per-platform native modules behind the Rust core.

## Considered Options

- **Native Swift (AppKit/SwiftUI):** best macOS integration, but nothing carries over to Windows.
- **Electron:** cross-platform and React-friendly, but a much heavier always-on process for a menu bar app.
- **Tauri:** cross-platform, light at idle, React for UI, and Rust is readable enough for the owner to maintain.
