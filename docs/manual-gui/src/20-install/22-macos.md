# Installing on macOS

`mnemonic-gui` runs on macOS 11 (Big Sur) and later, on both Apple
Silicon (`aarch64-apple-darwin`) and Intel (`x86_64-apple-darwin`).
The GUI uses the system's Metal graphics backend via `wgpu`; no
extra GPU configuration is required.

## Prerequisites

1. **A working Rust toolchain.** Install via `rustup`:

   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

   Verify: `cargo --version && rustc --version`. `mnemonic-gui`'s
   `Cargo.toml` declares MSRV 1.85 (`rust-version = "1.85"`).
   Apple Silicon users: rustup detects the target automatically; no
   extra `--target` flag needed.

2. **The four constellation CLIs on your `$PATH`.** `mnemonic-gui`
   requires `mnemonic`, `md`, `ms`, and `mk` to already be installed.
   See the CLI manual's install chapter, or in short:

   ```sh
   cargo install --locked --git https://github.com/bg002h/mnemonic-toolkit.git mnemonic-toolkit
   cargo install --locked md-cli
   cargo install --locked ms-cli
   cargo install --locked mk-cli
   ```

   Verify with `mnemonic --version`, etc.

3. **Xcode Command Line Tools** (for the C linker and system
   headers). Run `xcode-select --install` if not already installed.
   A full Xcode is NOT required.

## Installing the GUI

Path A — install from source via `cargo`:

```sh
cargo install --locked --git https://github.com/bg002h/mnemonic-gui.git mnemonic-gui
```

Path B — clone and build:

```sh
git clone https://github.com/bg002h/mnemonic-gui.git
cd mnemonic-gui
cargo build --release --bin mnemonic-gui
./target/release/mnemonic-gui
```

Path C — prebuilt binary from GitHub releases. Every
`mnemonic-gui-v*` release attaches per-architecture assets; for
macOS these are `mnemonic-gui-${VERSION}-aarch64-macos.tar.gz`
(Apple Silicon) and `mnemonic-gui-${VERSION}-x86_64-macos.tar.gz`
(Intel). Download the asset that matches your CPU, extract, and
place `mnemonic-gui` on your `$PATH`.

:::danger
**Prebuilt binaries are NOT yet code-signed nor notarised at v1.0.**
macOS Gatekeeper will refuse to run them on first launch with a
"`mnemonic-gui` cannot be opened because the developer cannot be
verified" dialog. Workarounds:

- **Recommended:** install from source via Path A (no Gatekeeper
  involvement).
- **Tolerable:** clear the quarantine attribute manually after
  download: `xattr -d com.apple.quarantine mnemonic-gui`. Verify
  the SHA-256 sum against the release notes BEFORE clearing.
- **Avoid:** right-click → Open → Open Anyway. This sets a
  per-binary trust without any signature verification, leaving you
  unable to detect tampering on later updates.

Both code-signing and notarisation are tracked as a single FOLLOWUP
`gui-code-signing-mac-developer-id` in the GUI repo (Apple Developer
ID + notarytool submission to Apple's notarisation service).
:::

Verify with `mnemonic-gui --version`.

## Window-occlusion (OS-snapshot defense)

The GUI sets `NSWindowSharingType::NSWindowSharingNone` on its
main window at startup (`mnemonic-gui/src/platform.rs:64`). This
opts the window out of:

- Screen-recording via `CGWindowListCreateImage` / `CGDisplayStream`
  (used by QuickTime Player and most third-party recorders).
- The OS-rendered window thumbnail in Mission Control / Dock /
  Cmd-Tab application switcher.
- AirPlay receivers that honour the per-window sharing-type opt-out
  (most do via the modern `ScreenCaptureKit` path; legacy
  whole-display mirroring may still capture the framebuffer because
  `setSharingType` only affects per-window capture APIs).
- The system screenshot tools (`Cmd-Shift-3`, `Cmd-Shift-4`, the
  Screenshot.app utility) — they capture a blank window where
  `mnemonic-gui` would be.

This is automatic; you do not need to configure anything. The GUI
logs `"OS-snapshot occlusion (macOS): NSWindowSharingType::NSWindowSharingNone applied"`
at startup when the defense activates successfully. Use `--debug`
to see the log line.

:::primer
**What this does NOT protect against:** a physical camera pointed
at your monitor; a kernel-level screen-grab utility you've granted
explicit permissions to (rare, but possible — e.g., remote-support
tools you've authorised); or apps that have been granted Screen
Recording access in System Settings → Privacy & Security and that
implement their own non-CGWindow-API capture. If you've granted
"Screen Recording" to an app you don't trust, revoke it before
using `mnemonic-gui`.
:::

## Ctrl-C / SIGTERM handling

`mnemonic-gui` on macOS installs a `signal-hook` thread that
catches `SIGINT` and `SIGTERM` (`main.rs:147-172`). Either signal
triggers a clean shutdown via `ViewportCommand::Close`, which in
turn fires the on-exit zeroize sweep (chapter 14 Defense 3) over
all secret-class form buffers before the process exits.

This means you can safely Ctrl-C the GUI from the terminal that
spawned it; secrets will be zeroed before the process leaves. If
the event loop is unresponsive for any reason, the signal handler
escalates to `process::exit(130)` after a 3-second grace period.
