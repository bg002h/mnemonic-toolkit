# Secrets, the OS, and operational hygiene {#secrets-and-os}

Secret-bearing inputs — BIP-39 phrases, `ms1` strings,
passphrases, SLIP-39 shares, XOR shares — leave the GUI through
two narrow channels: the
**SecretLineEdit**\index{SecretLineEdit} widget masks them in
the form (with a deliberate, hold-to-reveal 👁 toggle for
verification — see [§14](#secret-reveal-toggle)), and the
**run-confirm modal**\index{run-confirm
modal} requires explicit confirmation before spawning the
subprocess. The chapter [§14 Defense 2](#secret-handling)
covers the threat-model rationale; this chapter covers the
operational symptoms that the GUI and the surrounding OS can
expose.

## Known limitations

| Limitation | Cause | Mitigation |
|---|---|---|
| Clipboard\index{clipboard} retains pasted phrases after **Copy command** | OS-level clipboard history (KDE Klipper, macOS Universal Clipboard, Windows Clipboard History) may persist the copied argv string | Clear the clipboard manually after **Copy command** finishes (`xsel -bc` on Linux, `pbcopy < /dev/null` on macOS, **Win+V → Clear all** on Windows). |
| Spawned-subprocess argv exposure | The **unredacted** argv is what actually spawns on **Run**; on a shared / multi-user host a secret-bearing value is briefly observable in the child's `/proc/<pid>/cmdline` (or `ps`), exactly as a direct CLI invocation would be | The run-confirm modal and the output-panel `argv:` echo are both **on-screen** masked (`••••` sentinel) — see [§14 Defense 2](#secret-handling) — but the on-screen redaction does not cover the spawned argv. Run secret-bearing flows on a single-user / cold node. (Rewriting secret values to an `@env:`-style channel before spawn is tracked separately and not yet shipped.) |
| Bundle multisig conditional gap | `--multisig-path-family` + `--threshold` not disabled under single-sig templates | See [§83](#form-and-output); fill carefully or fall back to the CLI. |

**Resolved since v0.39.0.** Earlier manual versions listed two limitations
that no longer hold: (1) the run-confirm modal rendering argv tokens in
plaintext — the modal (and the output-panel `argv:` echo) now redact each
secret-bearing value as a fixed `••••` sentinel; and (2) multi-row / slot
text widgets (`--slot @N.subkey=value`, repeating `--share`) not auto-masking
repeating secret-bearing rows — slot and repeating-secret tokens now carry a
per-row mask bit, so each secret row is masked individually in both the modal
and the `argv:` echo.

## OS-level snapshot risks

The host OS can capture the GUI's window in ways the GUI's
platform-occlusion layer either does or does not block. On
macOS and Windows the GUI installs OS-level capture exclusion
at startup (per [§14 Defense 4](#secret-handling) and
`mnemonic-gui/src/platform.rs::apply_window_capture_protection`):
macOS via `NSWindowSharingType::NSWindowSharingNone`, Windows
via `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)`. Linux has
no compositor-level analogue (still tracked at FOLLOWUP
`gui-os-snapshot-secret-occlusion`); Linux users carry the full
residual exposure surface.

- **Screenshots\index{screenshot}**:
  - **macOS / Windows**: the GUI window is already excluded
    from `Cmd+Shift+3`, PrintScreen, and the system screenshot
    APIs. The exclusion fails open (logs a warning) if the
    platform call fails; verify by attempting a screenshot —
    the GUI window should render as a black rectangle.
  - **Linux**: there is no equivalent occlusion. Disable
    `gnome-screenshot` / `kwin_wayland`'s capture features
    before filling secret-bearing fields, or move the GUI to a
    virtual desktop / TTY you do not capture from.
- **Screen recording** (Zoom, Meet, Loom, OBS): on macOS and
  Windows the same window-capture exclusion applies. On
  Wayland the `ext-screencopy` protocol can capture window
  contents without permission prompts and bypasses the
  Linux GUI's lack of occlusion; close all recording software
  before launching.
- **Accessibility tools / screenreader\index{screenreader}**:
  the `SecretLineEdit` widget renders via egui's password mode
  (`TextEdit::password(true)`), which advertises a
  password-input role through AccessKit. Each screenreader
  (Orca / NVDA / VoiceOver) applies its own per-app policy for
  password fields — most announce nothing or a generic
  "asterisk" per character, but configurable. Verify your
  specific screenreader does not announce the underlying
  characters before trusting the threat model. The 👁 reveal
  toggle ([§14](#secret-reveal-toggle)) latches for keyboard /
  assistive-technology activation, so a revealed field
  advertises its plaintext through AccessKit deliberately —
  verify your screenreader's behavior on the revealed state too.
- **Window-capture by other applications**: macOS Mission
  Control thumbnails and Windows Alt+Tab thumbnails are
  subject to the OS-level exclusion above (i.e. the thumbnail
  is suppressed or rendered black). On Linux, KDE
  `kwin_wayland`'s window-thumbnail feature and GNOME's
  switcher previews CAN surface a low-resolution snapshot of
  the GUI window — the snapshot does not include a masked
  `SecretLineEdit`'s underlying value (those characters only
  exist in egui's internal buffer, not the rendered frame),
  **unless the field is revealed via the 👁 toggle**, in which
  case the plaintext is drawn to the frame and can be captured
  like any other visible widget (re-mask before you look away);
  adjacent form widgets (dropdowns, plain-text fields) ARE
  visible regardless.

## Cold-node operational mitigation

For high-stakes secret operations (initial bundle synthesis,
SLIP-39 share generation, BIP-85 derive-child), the
recommended threat-model posture is:

1. **Air-gapped machine**. Use a USB-booted live OS (Tails,
   PureOS) with no network adapter enabled.
2. **No clipboard manager**. Boot a clean session with
   clipboard history disabled.
3. **No screenshot tooling**. Uninstall or disable any active
   capture daemon.
4. **Transcribe outputs to physical media** (paper, steel
   plate) directly from the GUI's output panel; do **not**
   pipe the output to disk.
5. **Power-cycle after each session**. Tails wipes RAM on
   shutdown by default; on PureOS or a generic live ISO,
   `systemctl poweroff` is sufficient because the disk session
   is ephemeral.

The Bundle and Verify-Bundle workflows are explicitly designed
to support this posture: bundle synthesis produces engraving-ready
cards; verify-bundle round-trips physically engraved cards back
into the bundle for confirmation. Neither operation needs
network access.

## Reporting suspected secret leakage

If you suspect the GUI exposed a secret in a way this manual
does not cover (clipboard, log file, screenshot, OS metric):

1. **Stop using the affected wallet immediately**. Sweep funds
   from any wallet whose seed touched a compromised channel.
2. **Capture the failure mode**. A reproduction is more useful
   than a report; if you can reproduce on a non-secret test
   wallet (use the canonical all-`abandon` phrase) the project
   can diagnose without seeing real secrets.
3. **File a report** at the `mnemonic-gui` GitHub repo. Tag
   `secret-handling` for triage.
