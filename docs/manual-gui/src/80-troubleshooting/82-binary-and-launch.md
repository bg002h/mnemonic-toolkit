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

The toolkit installer installs these, except `mnemonic`, `md` and `ms`, which it already
installs at newer releases (next section). Run each binary with
`--version` to compare; the GUI does not check them for you. The GUI's
secret channels were measured against exactly these binaries, so an
older release can misread or refuse the channel spellings the GUI
writes (for example `ms derive --passphrase @env:…` before `ms`
0.20); below `mnemonic` 0.104.0 even the macOS/Windows interim path
fails (older releases reject its `--allow-argv-secret`). See
[Version pinning](#version-pinning). Re-run the toolkit installer to
put its releases in place; the next section lists the three it installs
newer than these.

## When the installer is ahead of the GUI {#installer-ahead}

The toolkit installer currently installs three CLI releases newer than
this GUI was built against: `mnemonic-toolkit-v0.106.0` (the GUI's label
reads `Pinned: mnemonic 0.105.1`), `descriptor-mnemonic-md-cli-v0.20.4`
(label `Pinned: md 0.20.3`) and `ms-cli-v0.20.4` (label
`Pinned: ms 0.20.1`). The forms run unchanged, and `mnemonic --version`,
`md --version` and `ms --version` printing the newer versions is expected.

Outputs differ, all on invalid or unusual input or on the recovery line
of a card; none changes a derived wallet. First `mnemonic`, three
outputs:

- **`bundle` with `--import-json`, and `export-wallet` with
  `--from-import-json`.** An envelope whose parts disagree is now
  refused (exit 1, nothing on stdout, so the output panel shows only the
  error), where it printed wrong cards or wrong wallet files at exit 0.
  An envelope that `import-wallet --json` wrote still passes. If one is
  refused, run `import-wallet` again on the same wallet file to make a
  new one.
- **`bundle`.** For a descriptor that is not a plain k-of-n (a
  timelocked inheritance, a decaying or nested policy, a taproot
  descriptor with several leaves), the card no longer prints
  `# Threshold: K of N`; its recovery line reads `# Recovery: general
  policy, not k-of-n (the md1 policy decides which key sets can spend);
  keep all N signing keys, …`. A plain k-of-n descriptor and every
  multisig template print both lines as before. In `--json`,
  `multisig.threshold` is `null` for such a policy.
- **`bundle` with `--import-json` and `--threshold`.** The
  `--threshold` field no longer sets K: the card takes K from the
  envelope's descriptor, so `--threshold 1` on a 2-of-3 envelope prints
  `# Threshold: 2 of 3`, where it printed `1 of 3`.

Then `ms`, five outputs:

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
- **`ms split` with `--out`.** A file that already exists (a
  directory, a symlink or `/dev/null` too) is now refused before the
  secret is read (exit 64, nothing written, the file left as it was):
  `error: --out shares.txt already exists; a new share set will not
  overwrite it (choose another file, or move the old one first)`,
  where it was overwritten at exit 0. Choose a new file name.
- **`ms decode`, `ms derive`, `ms hashlock` and `ms repair`.** An ms1
  whose tag is reserved for a later format (`seed`, for example) still
  exits 3 with nothing on stdout, and now prints `error: tag "seed"
  reserved-not-emitted in v0.1; deferred to v0.2+` on stderr, where
  the output panel showed nothing. With `--json` the output is
  unchanged, and `ms verify` still prints `OK: valid future format
  (v0.2+, tag seed)` and exits 3.

Last `md`, four outputs:

- **`md encode`.** `--fingerprint` takes hex digits only: `@0=+f+f+f+f`
  is now refused (`md: --fingerprint @0: invalid hex byte: +f`) where
  it was read as a fingerprint. A multipath alternative of 2^31 or more
  is now refused as hardened (`md: template parse error: …`, naming
  the alternative as hardened), where the error was a `md: codec error:`
  saying the value 2147483648 exceeds the single-extension range.
  With `--policy-id-fingerprint`, the `policy-id-fingerprint: 0x…` line
  now goes to stderr, so stdout carries only the md1 string(s).
- **`md encode --json`.** `older(65536)` is now refused with `md: codec
  error: older(65536) is not what consensus enforces: …`, where it
  printed a card that locks for 0 blocks. A card that needs several
  chunks now prints `chunk_set_id` and `chunks`, where it failed with
  `md: codec error: payload is N data symbols; the codex32 regular
  code caps single strings at 80 …`.
- **`md inspect`.** A card that carries keys now ends with the stderr
  note `note: stdout is watch-only — public keys only, cannot spend`,
  where it read `note: stdout is a keyless descriptor template (no
  keys)`.
- **`md decode`.** A single key at its canonical origin, on a card that
  carries no fingerprint for it (for example `wpkh` at `m/84'/0'/0'`
  minted without `--fingerprint`), no longer prints the stderr note
  `key origins carried by this card …` and its `--path` hint. A card
  that carries the fingerprint still prints both.

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
