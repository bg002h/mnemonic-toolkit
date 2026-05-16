# Installing on Linux {#install-linux}

`mnemonic-gui` runs on every mainstream Linux desktop (X11 and
Wayland; GNOME, KDE Plasma, Sway, Hyprland, XFCE). It is a wgpu-based
egui application: the underlying graphics stack uses Vulkan or OpenGL
depending on what your driver exposes. Below covers binary install,
the four CLI prerequisites, and the Linux-specific
graphics-stack notes you may need.

## Prerequisites

1. **A working Rust toolchain.** `mnemonic-gui`'s `Cargo.toml`
   declares MSRV 1.85 (`rust-version = "1.85"`). Install via `rustup`:

   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

   Verify: `cargo --version && rustc --version`. (Distribution
   packages also work but tend to lag the pinned toolchain.)

2. **The four constellation CLIs on your `$PATH`.** `mnemonic-gui`
   requires `mnemonic`, `md`, `ms`, and `mk` to be already installed.
   Install them from the CLI manual's "Installing the toolkit"
   chapter, or in short:

   ```sh
   cargo install --locked --git https://github.com/bg002h/mnemonic-toolkit.git mnemonic-toolkit
   cargo install --locked md-cli
   cargo install --locked ms-cli
   cargo install --locked mk-cli
   ```

   The GUI pins specific tags of these (see chapter 12); install at
   or above the pinned tags. Verify with `mnemonic --version` etc.

3. **System libraries for the graphics stack.** Most distributions
   ship these already; if not, install the development headers your
   distro's `wgpu` package requires. On Debian/Ubuntu:

   ```sh
   sudo apt install libxkbcommon-dev libwayland-dev libx11-dev \
                    libxcursor-dev libxrandr-dev libxi-dev \
                    libgl1-mesa-dev
   ```

   On Fedora/RHEL/Arch the equivalents have similar names; the
   `wgpu` upstream tracks distro-specific requirements.

## Installing the GUI

Path A — install from source via `cargo` (recommended pre-v1.0):

```sh
cargo install --locked --git https://github.com/bg002h/mnemonic-gui.git mnemonic-gui
```

This compiles the source and writes `mnemonic-gui` into
`~/.cargo/bin/`. Ensure that directory is on your `$PATH` (most
`rustup` installs add it automatically).

Path B — clone and build for contributors:

```sh
git clone https://github.com/bg002h/mnemonic-gui.git
cd mnemonic-gui
cargo build --release --bin mnemonic-gui
./target/release/mnemonic-gui
```

Path C — prebuilt binary from GitHub releases. Every
`mnemonic-gui-v*` release attaches per-architecture assets; for
Linux these are `mnemonic-gui-${VERSION}-x86_64-linux.tar.gz` and
`mnemonic-gui-${VERSION}-aarch64-linux.tar.gz` (built from
`.github/workflows/build.yml`). Download the asset that matches
your CPU, extract, and place `mnemonic-gui` somewhere on your
`$PATH`. Prebuilt binaries are unsigned at v1.0; verify the
SHA-256 sum against the release notes.

Verify with `mnemonic-gui --version`.

## Graphics-stack notes

### Wayland-keepalive (KDE / Hyprland / Sway / GNOME under Wayland)

The GUI uses `egui`'s reactive paint loop, which only wakes when
input arrives. An idle window can go many seconds between Wayland
surface commits — long enough that KDE/KWin (and some other
compositors) flag the client "Not Responding" in the title bar.
The GUI mitigates this by spawning a background thread that calls
`ctx.request_repaint()` once per second. Idle CPU stays near zero
because each woken frame does no GPU work when state is unchanged.

This is automatic; you do not need to configure anything. If you
see "Not Responding" titles on your Wayland compositor, file a bug
— the keepalive should prevent it.

:::primer
The keepalive only works under the `wgpu` renderer (the default).
`egui_glow` on Wayland silently drops cross-thread wake events; the
GUI does not use `egui_glow` for this reason. The decision to use
`wgpu` over `egui_glow` is recorded in `mnemonic-gui/FOLLOWUPS.md`
under the resolved entry `gui-glow-wayland-loop-broken` (v0.1.1
renderer swap).
:::

### Vulkan vs OpenGL backend selection

`wgpu` picks Vulkan if your driver supports it, falling back to
OpenGL/GLES otherwise. To force a specific backend (debugging,
old hardware):

```sh
WGPU_BACKEND=vulkan mnemonic-gui    # force Vulkan
WGPU_BACKEND=gl mnemonic-gui        # force OpenGL
```

The output panel will not show backend-selection details at v0.3;
run with `--debug` to surface them in stderr.

### Verbose tracing

```sh
mnemonic-gui --debug    # equivalent to RUST_LOG=debug
RUST_LOG=mnemonic_gui=debug,wgpu=warn mnemonic-gui    # finer-grained
```

The default filter suppresses noisy `wgpu` swap-chain warnings that
fire during the 1 Hz keepalive idle repaints. Override the default
filter via `RUST_LOG` if you need to debug wgpu itself.

## OS-snapshot occlusion gap (Linux)

**Linux has no compositor-level API equivalent to macOS's
`NSWindowSharingType::None` or Windows's `WDA_EXCLUDEFROMCAPTURE`
at v0.3.** This means screen-recording tools (OBS, ffmpeg, the
compositor's built-in screenshot) can capture the GUI's window
contents — including secret-bearing form fields.

The GUI cannot defend against this directly. If you are entering
catastrophic-on-leak material (a BIP-39 phrase, an `ms1` string,
etc.) on a Linux system:

- Close any screen-recording software before opening the GUI.
- If you must record (debugging a bug), use Defense 1–3 from
  chapter 14 (never-persist invariant, run-confirm modal, on-exit
  zeroize sweep) — the recording captures what's on-screen, but
  the GUI does not leave the secret on disk.
- Camera-based screenshot threats (a phone pointed at the monitor)
  are out of scope for any software.

The FOLLOWUP `gui-os-snapshot-secret-occlusion-linux` in the GUI
repo tracks the Linux occlusion gap; an upstream Wayland protocol
extension (`wlr-screencopy` blocking) is the most plausible path.
