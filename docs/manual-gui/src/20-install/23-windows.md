# Installing on Windows

`mnemonic-gui` runs on Windows 10 (1809+) and Windows 11. The GUI
uses `wgpu`'s DirectX 12 backend by default (Vulkan and DX11
fallbacks available); no GPU configuration is required on
modern systems.

## Prerequisites

1. **A working Rust toolchain.** Install via `rustup`:

   - Download `rustup-init.exe` from <https://rustup.rs> and run it.
   - Accept the defaults; rustup installs the MSVC toolchain by
     default, which is what `wgpu` and `egui` expect.
   - Verify in a fresh `cmd.exe` or PowerShell:
     `cargo --version && rustc --version`.

   `mnemonic-gui`'s `Cargo.toml` declares MSRV 1.85
   (`rust-version = "1.85"`).

2. **Visual Studio Build Tools** (for the MSVC linker). If you do
   not have Visual Studio installed, download the Build Tools from
   <https://visualstudio.microsoft.com/downloads/> and install the
   "Desktop development with C++" workload. `rustup-init` will
   prompt for this if it is missing.

3. **The four constellation CLIs on your `$PATH`.** `mnemonic-gui`
   requires `mnemonic`, `md`, `ms`, and `mk` to already be installed.
   See the CLI manual's install chapter, or in short:

   ```pwsh
   cargo install --locked --git https://github.com/bg002h/mnemonic-toolkit.git mnemonic-toolkit
   cargo install --locked md-cli
   cargo install --locked ms-cli
   cargo install --locked mk-cli
   ```

   Verify with `mnemonic --version`, etc. (Cargo's binary directory
   is at `%USERPROFILE%\.cargo\bin`; rustup adds it to your `PATH`.)

## Installing the GUI

Path A — install from source via `cargo`:

```pwsh
cargo install --locked --git https://github.com/bg002h/mnemonic-gui.git mnemonic-gui
```

Path B — clone and build:

```pwsh
git clone https://github.com/bg002h/mnemonic-gui.git
cd mnemonic-gui
cargo build --release --bin mnemonic-gui
.\target\release\mnemonic-gui.exe
```

Path C — prebuilt binary from GitHub releases. Every
`mnemonic-gui-v*` release attaches a
`mnemonic-gui-${VERSION}-x86_64-windows.zip` asset (built from
`.github/workflows/build.yml`). Download, extract, and place
`mnemonic-gui.exe` in a directory on your `PATH`.

:::danger
**Prebuilt binaries are NOT code-signed at v1.0.** Windows
SmartScreen will show a "Windows protected your PC" dialog on first
launch ("Don't run" by default; click "More info" → "Run anyway" to
bypass). Verify the SHA-256 sum against the release notes BEFORE
bypassing SmartScreen.

Defender may also flag the unsigned binary as a potential threat.
This is a known false-positive class for unsigned Rust binaries; the
fix is code-signing (FOLLOWUP `gui-code-signing-windows` in the GUI
repo — Authenticode certificate, EV variant for SmartScreen
reputation). Until then, install from source via Path A to avoid
the warning.
:::

Verify with `mnemonic-gui --version`.

## Window-occlusion (OS-snapshot defense)

The GUI calls `SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE)`
on its main window at startup (`mnemonic-gui/src/platform.rs:91`).
This opts the window out of:

- DXGI desktop-duplication API capture (used by most modern
  screen-recorders and the Windows Snipping Tool).
- The built-in `Win+Print Screen` and `Win+Shift+S` screenshot
  tools — they capture a blank window where `mnemonic-gui` would be.
- Remote-desktop / virtual-display capture pipelines that respect
  display affinity (most do).

This is automatic; you do not need to configure anything. The GUI
logs `"OS-snapshot occlusion (Windows): WDA_EXCLUDEFROMCAPTURE applied"`
at startup when the defense activates successfully. Use `--debug`
to see the log line.

:::primer
**What this does NOT protect against:** a physical camera pointed at
your monitor; older Win32 GDI-based `BitBlt` captures (some legacy
screen-readers); kernel-mode drivers that bypass user-mode display
affinity. The defense is best-effort: it covers the modern capture
APIs that everyday tooling uses, not every conceivable bypass.
:::

## Ctrl-C handling

`mnemonic-gui` on Windows installs a `ctrlc` handler that catches
Console `Ctrl-C` events (`main.rs:175-185`). The handler triggers a
clean shutdown via `ViewportCommand::Close`, which in turn fires
the on-exit zeroize sweep (chapter 14 Defense 3) over all
secret-class form buffers before the process exits.

There is **no Windows equivalent to `SIGTERM`**; the Ctrl-C path is
the only graceful-shutdown channel. If you launch the GUI from
`Run` or from a desktop shortcut (no console attached), the
zeroize sweep still runs on a normal window-close event — but you
cannot send a signal externally. To force-terminate with the
zeroize sweep, close the GUI window via its title-bar `×` button
rather than killing the process from Task Manager.

## A note on the `mnemonic-gui.exe` filename

The Cargo `package.name` is `mnemonic-gui`; the produced binary
on Windows is `mnemonic-gui.exe`. Path-based detection in scripts
should account for the `.exe` suffix. The bundled `--version` and
`--help` flags work identically across platforms.
