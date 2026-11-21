# 01: Menu bar app skeleton, CI and repo basics

**Spec:** `.scratch/miniwhispr-v1/spec.md`

**What to build:** Miniwhispr builds and runs as a Tauri (Rust core) + React + TypeScript app that lives only in the macOS menu bar, with no Dock icon. The menu has a "Launch at login" toggle (on by default) and Quit. The repo is ready to be open source and stays green: anyone can clone it, follow the README, and build it, and CI checks every push.

**Blocked by:** None (can start immediately)

**Status:** resolved

- [ ] The app builds with the documented commands and appears as a menu bar icon with no Dock icon
- [ ] App name is "Miniwhispr" everywhere; bundle identifier `com.hectoragvz.miniwhispr`
- [ ] Menu contains "Launch at login" (checked by default, toggling it adds or removes the login item) and Quit
- [ ] After reboot with "Launch at login" on, Miniwhispr is back in the menu bar
- [ ] README lists prerequisites (Xcode Command Line Tools, Rust via rustup, Node and pnpm, a Groq API key) and the build steps
- [ ] A one-time script creates a local self-signed code-signing certificate, and builds use it so macOS permissions survive rebuilds; documented in the README
- [ ] MIT LICENSE file and a `.gitignore` excluding build output and dependencies
- [ ] GitHub Actions CI on push and pull request runs `cargo fmt --check`, `cargo clippy` with warnings as errors, `cargo test`, `tsc`, ESLint and Vitest, with no network secrets, and passes
- [ ] At least one trivial Rust test and one trivial Vitest test exist so every CI step runs real tests
