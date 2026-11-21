# Miniwhispr

Hold Ctrl + Space, speak, and clean, punctuated text lands wherever you are typing. Miniwhispr lives in the macOS menu bar and works in English and Spanish.

It is a personal, open-source project: there are no downloads, you clone it and build it yourself. It sends your audio to [Groq](https://groq.com) for transcription and Cleanup, using your own API key (see `docs/adr/0002-groq-cloud-for-transcription-and-cleanup.md`).

## Requirements

- macOS 13 (Ventura) or later
- **Xcode Command Line Tools:** `xcode-select --install`
- **Rust via rustup:** `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`. The exact Rust version is pinned in `rust-toolchain.toml` and installs itself on first build.
- **Node 26 and pnpm 10:** the Node version is in `.nvmrc` (`nvm use` or `fnm use` picks it up). Then `corepack enable` provides the pnpm version pinned in `package.json`.
- **A Groq API key:** create one at [console.groq.com](https://console.groq.com/keys). Miniwhispr asks for it on first run and stores it in your macOS Keychain.

## Build

1. Create the local signing certificate, once per Mac:

   ```sh
   ./scripts/create-signing-cert.sh
   ```

   macOS ties Microphone and Accessibility permissions to an app's signature. Signing every build with this same certificate keeps those permissions across rebuilds. The script asks for your login password once, to trust the certificate. The first build may also ask to let `codesign` use the key: choose **Always Allow**.

2. Install dependencies and build:

   ```sh
   pnpm install
   pnpm tauri build
   ```

3. Move the app to Applications and open it:

   ```sh
   rm -rf /Applications/Miniwhispr.app
   cp -R target/release/bundle/macos/Miniwhispr.app /Applications/
   open /Applications/Miniwhispr.app
   ```

Miniwhispr appears in the menu bar, with no Dock icon. It turns on **Launch at login** the first time it runs. You can turn it off from its menu.

Hold Ctrl + Space to dictate. The first time, macOS asks for two permissions: **Accessibility**, to catch the shortcut, and **Microphone**. Grant both.

## Develop

```sh
pnpm tauri dev
```

`tauri dev` runs an unbundled build. Launch at login only works in the bundled app from **Build**.

Checks (the same ones CI runs):

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
pnpm tsc
pnpm lint
pnpm test
```

## License

[MIT](LICENSE)
