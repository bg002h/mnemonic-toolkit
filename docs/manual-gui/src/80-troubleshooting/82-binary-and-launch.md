# Binary, launch, and version mismatch {#binary-and-launch}

The GUI is a thin shell over the four sibling CLIs (`mnemonic`,
`md`, `ms`, `mk`); every **Run** click spawns the corresponding
subprocess. When the subprocess can't be found, can't be
executed, or returns a version-mismatch banner, the GUI surfaces
the failure as a top-of-form banner or an output-panel error.

## Symptom matrix

| Symptom | Likely cause | Fix |
|---|---|---|
| `Pinned: mnemonic ?` (literal `?`) in the top-of-form banner | The `mnemonic` binary is not on `$PATH` or `$MNEMONIC_BIN` does not point to an executable | Install `mnemonic-toolkit` per [chapter 21/22/23](#install-linux); set `$MNEMONIC_BIN` if the binary lives outside `$PATH`. |
| `Pinned: md ?` / `Pinned: ms ?` / `Pinned: mk ?` | The matching sibling CLI is missing from `$PATH` | Install `md-cli` / `ms-cli` / `mk-cli` (each ships as a separate `cargo install` target — see [§12 relation-to-cli](#relation-to-cli)). |
| Top-of-form banner reads `Pinned: <name> <version-string>` but the version differs from the chapter's pinned upstream | Sibling-CLI version on `$PATH` is newer or older than the GUI was tested against | Re-install the matching pinned version per `docs/manual-gui/pinned-upstream.toml`; the GUI WILL still run against drifted versions, but help-text and refusal messages may differ from this manual. |
| `error: failed to spawn subprocess: …` in the output panel after **Run** | Binary on `$PATH` but not executable, or sandboxed away (Flatpak / Snap) | `chmod +x` the binary, or invoke via an absolute path via `$MNEMONIC_BIN`. Flatpak users: GUI cannot reach binaries outside the sandbox without an `--filesystem=host` permission. |
| GUI window never appears (process exits silently on Linux) | wgpu / egui graphics-stack mismatch — Wayland\index{Wayland} compositor without vulkan, or older Mesa | Verify `vulkaninfo` returns at least one device; fall back to OpenGL via `WGPU_BACKEND=gl`. See [chapter 21 wgpu/egui notes](#install-linux). winit ≥ 0.30 auto-selects the windowing-system backend from the standard `WAYLAND_DISPLAY` / `DISPLAY` env vars (the pre-0.29 `WINIT_UNIX_BACKEND` override no longer exists). |
| GUI launches but the window is black or full of artefacts | wgpu device picked an unstable backend | Set `WGPU_BACKEND=gl` (force OpenGL) or `WGPU_BACKEND=vulkan` explicitly. |

## Verifying the pinned-upstream tags

The GUI's `Pinned:` labels are the CLI releases it was built
against; `docs/manual-gui/pinned-upstream.toml` records the same tags
for this manual:

- `mnemonic-toolkit-v0.105.1`
- `descriptor-mnemonic-md-cli-v0.20.3`
- `ms-cli-v0.20.1`
- `mk-cli-v0.13.0`

The toolkit installer installs these, except `ms`, which it already
installs at a newer release (next section). Run each binary with
`--version` to compare; the GUI does not check them for you. The GUI's
secret channels were measured against exactly these binaries, so an
older release can misread or refuse the channel spellings the GUI
writes (for example `ms derive --passphrase @env:…` before `ms`
0.20); below `mnemonic` 0.104.0 even the macOS/Windows interim path
fails (older releases reject its `--allow-argv-secret`). See
[Version pinning](#version-pinning). Re-run the toolkit installer to
put its releases in place; the next section lists the one it installs
newer than these.

## When the installer is ahead of the GUI {#installer-ahead}

The toolkit installer currently installs one CLI release newer than
this GUI was built against: `ms-cli-v0.20.2` (the GUI's label reads
`Pinned: ms 0.20.1`). The forms run unchanged, and `ms --version`
printing `0.20.2` is expected.

Three outputs differ, all on invalid or unusual input; none changes a
derived wallet.

- **`ms inspect`.** A share at share index `s` (the secret itself,
  which `ms combine` refuses) now reads `FAIL: would NOT combine`,
  where it read `OK: K-of-N share (would combine: …)`. A mnem string
  whose language code `ms decode` refuses now reads `FAIL: would NOT
  decode v0.1`, where it read `OK: would decode v0.2`.
- **`ms combine`.** For a hashlock preimage share set, `--to entropy`
  now prints the 64-hex preimage (with a `warning:` line that it is a
  preimage, not seed entropy) and `--to ms1` prints the `hash` plate
  string; both used to print the preimage record, which `--to phrase`
  (the default) still prints.
- **`ms repair`.** A character outside the codex32 alphabet is now
  named (`error: invalid character 'b' (not in codex32 alphabet)`),
  where it used to be reported as a string-length error.

## When the GUI is older than the CLI

If your installed sibling CLI is **newer** than the GUI's
`Pinned:`, you may see new flags in `--help` that the GUI's
schema does not enumerate (so they're absent from the form), or
new dropdown variants in `--help` that the GUI's dropdown does
not list. The GUI ships its own schema; it does not
re-introspect the CLI at launch. Fix: pin both to matching
versions, or fall back to the terminal for the new flags.

## When the GUI is newer than the CLI

The reverse — GUI ships a schema entry for a flag the installed
CLI hasn't grown yet — surfaces as a clap-level
`unexpected argument` refusal at **Run** time. The output panel
will carry the byte-exact clap error. Fix: upgrade the CLI to
the version listed in `pinned-upstream.toml`.
