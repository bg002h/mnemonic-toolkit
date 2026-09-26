# Secret handling

`mnemonic-gui` treats every flag marked `secret: true` in the GUI's
schema as catastrophic-on-leak material. As of v1.0 the
schema-`secret: true` set covers the following classes:

- **BIP-39 phrases**\index{BIP-39} — `ms --phrase`; the canonical
  12/24-word mnemonic.
- **Raw entropy bytes** — `ms --hex`; the underlying entropy in hex.
- **`ms1` strings**\index{ms1} — `mnemonic <subcommand> --ms1`; the
  bech32-style encoding of the entropy plus a checksum.
- **BIP-39 passphrases**\index{passphrase} — `mnemonic <subcommand>
  --passphrase`; the optional 25th-word extension to a phrase.
- **BIP-38 passphrases** — `mnemonic <subcommand> --bip38-passphrase`;
  a distinct cryptographic passphrase used by BIP-38-encrypted
  minikey paths.
- **SLIP-39 passphrases** — `mnemonic slip39 split/combine
  --passphrase`; mechanically distinct from the BIP-39 passphrase
  even though the flag name is shared (different subcommand
  context).
- **SLIP-39 share phrases** — `mnemonic slip39 combine --share`;
  the per-share secret material itself.

Public material (`mk1`, `md1`, fingerprints, paths, xpubs, derivation
templates) is **NOT** secret-class and does NOT trigger the
run-confirm modal or any of the other defenses below. The
`?` help-icon button still attaches to those flags (per the §1.6
Option C affordance contract), but the runtime treats them as
ordinary text.

The schema is the type-level single source of truth: anything marked
`secret: true` in `mnemonic-gui/src/schema/*.rs` flows through the
`SecretLineEdit` widget, never persists, and triggers run-confirm.
Anything `secret: false` does not. Anyone who reads a secret-class
value can reconstruct your full wallet and spend your funds. The GUI's
secret-handling model has four independent defenses, each addressing
a different leak vector.

## Defense 1 — type-level never-persist invariant

Secret-class form fields live in `FormState.secret_widgets`, a map
that is `#[serde(skip)]` at the type level. This means:

- Save form state to disk → secret fields are skipped, even if you
  asked for them to be persisted.
- Load form state from disk → secret fields default-construct to
  empty.

This is a *compile-time* guarantee: serde's codegen cannot serialize
or deserialize a `#[serde(skip)]` field. The persistence layer cannot
accidentally leak a secret because the type system forbids it. The
test suite (`tests/persistence.rs::cell_2_never_persist_audit_strips_all_secret_flags`)
empirically verifies this against the schema's `secret: true` flags.

:::primer
Type-level invariants are stronger than runtime checks. Even if a
future code path mistakenly tries to serialise the secret widget,
the compiler will refuse — and the schema's `secret: true` boolean
is the single source of truth for what counts as a secret.
:::

## Defense 2 — run-confirm modal

Before a subprocess fires for any form containing a secret-class
flag, a modal pops up showing the assembled argv as it will be
passed to the subprocess. The modal has two buttons: **Run**
(confirm) and **Cancel** (abort). The modal title is
"Confirm secret-bearing run" and it is centered. There is no
Escape-key affordance: you must click **Run** or **Cancel**
explicitly to dismiss the modal. This is intentional under the
security-relevant-modal threat model — an accidental Escape that
fires a secret-bearing run would be a worse UX failure mode than
requiring a deliberate click. This guards against:

- Muscle-memory clicks on a pre-populated form.
- Forms reloaded from disk that you'd forgotten contained a secret.
- Unintentional invocations from a stuck **Run** button.

What the modal shows depends on how the secret travels (see
[Secret channels](#secret-channels) below). On Linux no secret is on
the command line at all: the modal's argv carries only a private
reference such as `phrase=@env:MNEMONIC_GUI_S0` or
`--passphrase-stdin`, its first line reads *"This invocation sends
these secrets privately to mnemonic:"*, and a **Secrets:** list names
each secret and its channel. On macOS and Windows, where the secret
still goes on the command line for now, the modal's first line reads
*"This invocation passes secret-bearing arguments to …"* and every
argv token that carries a secret renders as a fixed `••••` sentinel;
the *unredacted* argv is what spawns when you click **Run**.

**Residual exposure.** The flag NAMES stay visible (`--passphrase`,
`--slot @N.phrase=`), so anything that can read the screen can tell a
secret-bearing run is in progress. On macOS and Windows the secret is
also briefly readable in the child process's command line (`ps`),
exactly as in a direct CLI invocation. On Linux it is not: it reaches
the child through its environment, its standard input, or a pipe,
none of which another user's `ps` shows. In every case the secret is
still in the GUI's memory until the on-exit zeroize sweep runs (see
Defense 3).

### The CLI's argv refusal and the GUI's `--allow-argv-secret` {#secret-argv-opt-in}

The `mnemonic` and `ms` CLIs **refuse** a secret on the command line
— before parsing, reading or writing anything — unless the invocation
carries `--allow-argv-secret`. In a shell that refusal is exactly
right: the shell has already written the line to its history.

On Linux the GUI never needs the flag, because no secret goes on the
command line. On macOS and Windows (the interim path, see
[Secret channels](#secret-channels-other-os)) the **Run** path adds
`--allow-argv-secret` itself, directly after the subcommand, whenever
the assembled argv carries secret material; the GUI spawns its child
with no shell, so there is no history to leak into, and the modal
shows the flag. It is **GUI-managed**: there is no widget for it, and
it is never taken from form state. The **Copy command** buttons never
include it (see [What you see](#secret-channels-what-you-see)).

**General hygiene (no longer load-bearing).** With the modal redaction
in place, running secret-bearing flows on a cold / airgapped machine is
operational hygiene rather than the security model's load-bearing
element — but it is still good practice, because it bounds the blast
radius if any *other* secret surface (process memory, swap, a
screenshot of a non-redacted field) is captured. A machine whose
network connection is physically disabled or non-existent removes the
on-screen-to-network exfiltration path entirely. Two cold-node
patterns, if you choose to adopt one:

- A dedicated offline machine that never connects to the internet,
  with Bitcoin block updates delivered via sneakernet using
  `bitcoind`'s `loadblock` startup option (download `blk*.dat` files
  on a hot machine; transfer via removable media; load them on the
  cold node).
- A node that receives Bitcoin block updates one-way via a
  Blockstream Satellite receiver (the satellite link is
  receive-only at the radio layer; the node itself never speaks to
  the internet).

## Defense 3 — on-exit zeroize sweep

When the GUI window closes (either by normal close or by Ctrl-C /
SIGTERM), the `on_exit` hook runs `secrets::zeroize_form_state` over
every per-form state in memory. This explicitly overwrites the
secret-widget buffers with zeros before the process exits. The
sweep is *best-effort*: it cannot reach buffers that the OS has
already swapped out to disk, and it cannot reach buffers in the
heap allocator's free list that haven't been reused yet.

For stronger guarantees against swap-out, the host OS should run
the GUI under a `mlock`-aware shim or with `vm.swappiness=0`. The
GUI itself does not call `mlock` on its secret buffers today
(FOLLOWUP `gui-mlock-secret-buffers`).

## Defense 4 — OS-snapshot occlusion

On macOS, the GUI sets the window's `NSWindowSharingType` to
`None` so screen-recording APIs and the OS-level Mission Control
preview cannot capture the window's contents. On Windows, it sets
`WDA_EXCLUDEFROMCAPTURE` for the same effect on `BitBlt` / DXGI
capture. On Linux, **no equivalent compositor API exists at v0.3**;
the FOLLOWUP `gui-os-snapshot-secret-occlusion` tracks this gap.

This defense protects against:

- A screen-recording tool running in the background.
- A screenshot taken at the OS level (e.g., `cmd-shift-3` on macOS).
- The OS-rendered window thumbnail in Mission Control / Alt-Tab.

It does **not** protect against:

- A screenshot tool that uses a screen-grab API your platform's
  capture-protection doesn't cover (rare, but possible — research
  what your OS exposes).
- Anyone with a camera looking at your monitor.

## Secret channels — how a secret reaches the CLI {#secret-channels}

A form's secret — a phrase, an `ms1` card, a passphrase, a share, a
hashlock phrase — has to reach the CLI somehow. The command line is
the one place it must not go: other users' `ps`, `/proc` and crash
reports can read it. Since mnemonic-gui v0.63.0 the GUI chooses a
**private channel** for every secret on Linux, and never falls back
to the command line: if it cannot send a secret privately, it refuses
to run.

### On Linux: every secret goes privately {#secret-channels-linux}

For each secret the GUI picks one of three channels, using only the
ones measured to deliver the exact bytes to that input of the pinned
CLI:

- **Its own environment variable.** The GUI puts the value in a
  variable of the child process, `MNEMONIC_GUI_S0`, `MNEMONIC_GUI_S1`,
  …, and passes the CLI's own reference to it: `--from
  phrase=@env:MNEMONIC_GUI_S0`, `--slot @0.phrase=@env:MNEMONIC_GUI_S0`.
  Before spawning it removes every `MNEMONIC_GUI_*` variable it
  inherited, so only its own are set.
- **Standard input.** One secret per run can go on stdin, through the
  input's own switch (`--passphrase-stdin`, `--hashlock-phrase-stdin`)
  or a `-` in its place (`--hex -`, a positional `-`). The GUI adds the
  line ending the CLI strips, so the CLI receives the exact value.
- **A pipe.** Where neither fits, a pipe the child inherits, named on
  the command line as `/dev/fd/N` (for example `--in /dev/fd/3`). The
  payload is written and closed before the child starts; it is limited
  to 4096 bytes.

The command line then carries only these references. For example,
`restore` with a typed phrase and passphrase runs as

```text
mnemonic restore --from 'phrase=@env:MNEMONIC_GUI_S0' --format bitcoin-core --template bip84 --network mainnet --language english --passphrase-stdin
```

with no `--allow-argv-secret` and no CLI warning about argv.

### `-` and `@env:VAR` in a secret field {#secret-channels-typed-sentinels}

On the command line, `-` means "read it from stdin" and `@env:VAR`
means "read it from the environment variable `VAR`". In a GUI secret
field these spellings keep their meaning, and the GUI **never sends
the characters themselves as your secret**, on any OS:

- **`@env:VAR`** — the GUI reads `VAR` from **its own** environment
  when you click **Run** (so export it before you start the GUI), and
  sends the value over the private channel as if you had typed it. The
  value never appears on screen; the Preview says where it came from,
  as `(value of $VAR)`. `VAR` must match `[A-Z_][A-Z0-9_]*`, must be
  set, and must not be empty.
- **`-`** — refused. The GUI has no standard input of its own to
  forward, so there is nothing to read. Type the value, or use
  `@env:VAR`.

A consequence: a secret that is literally `-`, or that starts with
`@env`, cannot be entered in the GUI at all (see the lookalike rule
below). Nobody should choose such a passphrase.

### Refusals {#secret-channels-refusals}

When a secret cannot be sent safely, **Run** is disabled, the form
shows *"Run refused — `<code>`: `<field>`: `<reason>`"*, both Copy
buttons are disabled with the same reason as their tooltip, and
nothing is sent anywhere. Each refusal below was reproduced by typing
the value into `restore --passphrase` in the GUI:

| You typed | Refusal code | The form says |
|---|---|---|
| `-` | `C1-dash` | the GUI has no stdin of its own to forward; type the value, or use @env:VAR |
| `@env:VAR`, `VAR` not set | `C1-env-unset` | `$VAR` is not set in the GUI's environment |
| `@env:VAR`, `VAR` empty | `C1-env-empty` | `$VAR` is empty (after the CLI's @env: rule) |
| `@env:my_var` (not `[A-Z_][A-Z0-9_]*`) | `C1-bad-name` | `"my_var" is not a valid name ([A-Z_][A-Z0-9_]*)` |
| `@env:MNEMONIC_GUI_S0` | `C1-reserved-name` | MNEMONIC_GUI_* names are the GUI's own |
| a value that reads like `-` or `@env…` after normalizing: ` - `, `@ENV:X`, `@environment`, a zero-width space before `-`, a full-width hyphen-minus (U+FF0D), or a variable holding `@env:OTHER` | `value-looks-like-a-channel` | after trimming and case-folding this value reads like `-` or `@env…`, which a CLI may treat as a channel; nobody wants that as a secret |
| a value ending in CR or LF: typed, or read from a variable that still ends in one after the CLI's `@env:` rule (see below) | `value-ends-in-newline` | the value ends in a newline or CR (check how the variable was set); CLIs differ in how many they strip, so the GUI does not send it |
| a value containing a NUL byte | `nul-in-value` | the value contains a NUL byte |

The **lookalike rule** normalizes the value first (Unicode NFKC,
control and invisible format characters removed, surrounding
whitespace stripped, case folded) and refuses it if the result is `-`
or starts with `@env`. It applies to typed values and to values read
from `@env:VAR` alike, so a variable can never smuggle a channel
spelling to the CLI. A value that merely *starts* with `-` (such as
`-lead`) is not a lookalike and goes privately on Linux.

A value read from `@env:VAR` is first given the same treatment the
CLI's own `@env:VAR` would give it on that input. For passphrase and
password inputs (`--passphrase`, `--bip38-passphrase`,
`--decrypt-password`) that strips **one** trailing newline, so a
variable set from a file with one newline works; a variable holding
`hunter2` and two newlines is refused. Seed, share and card inputs are
taken verbatim, so a variable ending in a newline is refused there.

The `@env:MNEMONIC_GUI_…` refusal applies in **every** field, public
ones included, because those names belong to the GUI's own channels.

### What you see: Preview, the dialog, and Copy {#secret-channels-what-you-see}

- **Preview.** The `Preview:` line shows the command line as it will
  run — with references, not secrets — and under it one line per
  secret: the field, its channel, and where the value came from.

  ```text
  Preview: mnemonic restore --from 'phrase=@env:MNEMONIC_GUI_S0' … --passphrase-stdin
    --from phrase= ← env MNEMONIC_GUI_S0 (typed)
    --passphrase ← stdin via --passphrase-stdin + '\r\n' (value of $MY_PW)
  ```

  `(typed)` means you typed the value; `(value of $MY_PW)` means the
  GUI read it from `$MY_PW`. `+ '\r\n'` is the line ending the GUI
  appends for the CLI to strip.
- **The confirm dialog.** Its first line reads *"This invocation sends
  these secrets privately to mnemonic:"*. It lists the command line
  (references only) and, under **Secrets:**, the same per-secret lines
  as the Preview. **Run** and **Cancel** work as described under
  Defense 2.
- **Copy command.** The copied text never contains a secret value and
  never contains `--allow-argv-secret`. It is built from the Linux
  plan on **every** OS, with one comment line per secret telling you
  how to supply it in your shell:

  ```text
  # --from phrase=: IFS= read -rs MNEMONIC_GUI_S0; export MNEMONIC_GUI_S0   (fish: read -s -x --delimiter \n MNEMONIC_GUI_S0)
  # stdin (--passphrase): type it, then Enter, then Ctrl-D
  mnemonic restore --from 'phrase=@env:MNEMONIC_GUI_S0' … --passphrase-stdin
  ```

  A secret you supplied as `@env:MY_PW` is copied as your own
  reference instead. Where the CLI itself reads `@env:` for that input
  (every passphrase), that is `--passphrase @env:MY_PW` with the
  comment `# --passphrase: read by the CLI from $MY_PW`; otherwise the
  value is piped, for example `# --phrase: stdin from $MY_SEED` and
  `printf '%s\r\n' "$MY_SEED" | mnemonic xpub-search path-of-xpub
  --phrase-stdin …`. A file-bound
  secret becomes `--in <FILE>` with a comment naming what the file
  must hold. Copy is **disabled** when the plan is refused (the tooltip
  is the refusal), when a typed value that would go through an
  environment variable spans several lines (*"this value spans lines;
  use Run, or paste it into the command's own stdin prompt"*), and, for
  **Copy command (Windows)** only, when a secret needs stdin (*"needs a
  pipe; use the POSIX copy"*).

  **The one exception** is not a secret field: a private extended key
  (`xprv…`) pasted into a public `md` field
  ([`md descriptor --key`](#md-descriptor-key),
  [`md shape-key --descriptor`](#md-shape-key-descriptor), the
  [`md decompose`](#md-decompose) positional). md reads those only
  from the command line, so the GUI masks the key on screen, but a
  copied command would carry it; the buttons then read
  *"Copy command (POSIX) — reveals secret"* and *"Copy command
  (Windows) — reveals secret"*, and the confirm dialog uses the
  *"passes secret-bearing arguments"* wording.

### On macOS and Windows: the interim path {#secret-channels-other-os}

Private channels are switched on per OS, and only for an OS whose
continuous-integration job runs the GUI's real-binary channel tests.
Today that is Linux only. On macOS and Windows the GUI still resolves
`-` and `@env:VAR` exactly as above and applies every refusal above,
but then puts the resolved secret on the command line, with
`--allow-argv-secret`:

- the Preview and the confirm dialog show each secret as `••••`, and
  the per-secret lines read `← argv + --allow-argv-secret (interim)`;
- the dialog's first line reads *"This invocation passes
  secret-bearing arguments to …"*;
- the secret is briefly visible in the child's command line to `ps`
  on a multi-user machine;
- one extra refusal applies, `value-starts-with-dash`: a secret that
  starts with `-` would read as a flag on the command line. The GUI
  sends it as `--flag=VALUE` where that form was measured to deliver
  the exact bytes, and refuses it where it was not (for example
  `ms hashlock --hashlock-phrase`).

Copy command behaves the same on every OS: it is built from the
private plan, so a pasted command behaves like a Linux run.

### The CLIs on their own: `-`, `@env:VAR`, and the terminal prompt {#secret-channels-cli}

The same spellings work when you run the CLIs yourself, in
mnemonic-toolkit v0.105.1 and ms-cli v0.20.1 and later. Measured with
the all-`abandon` phrase and the passphrase `TREZOR` (fingerprint
`b4e3f5ed`; with no passphrase, `73c5da0a`):

- **`--passphrase -`** reads the passphrase from standard input, the
  same as `--passphrase-stdin`; **`--passphrase @env:VAR`** reads it
  from the environment variable `VAR`. Both apply to `--passphrase` in
  `mnemonic` and `ms`, and to `--bip38-passphrase` and
  `--decrypt-password` in `mnemonic`. (Before these releases both
  spellings were taken literally, as a passphrase of `-` or
  `@env:VAR`, which derived a different wallet.)
- **Exactly one line ending is stripped** from stdin: `TREZOR` followed
  by `\n` or `\r\n` gives `b4e3f5ed`; `TREZOR` followed by two `\n`
  gives another wallet.
- **An empty passphrase is a warning, not a refusal:**
  `warning: --passphrase from stdin is empty; proceeding with the
  EMPTY passphrase` (or `… from environment variable VAR is empty …`),
  and the run derives the no-passphrase wallet. An unset variable is
  refused: `error: --passphrase: env-var VAR referenced by sentinel is
  not set`, exit 1.
- **A passphrase typed on the command line** is refused before parsing
  unless you add `--allow-argv-secret`; with it, the CLI warns
  `warning: secret material on argv (--passphrase) — read it privately
  with --passphrase - or --passphrase-stdin (stdin), or --passphrase
  @env:VAR (environment variable)` and proceeds.
- **At a terminal**, `--passphrase -` (or `--passphrase-stdin`) prints
  `Enter passphrase:` on stderr and reads one line without echoing it.
  Anything typed or pasted after that line is **discarded** rather than
  left for your shell to run, and a masked preview of each discarded
  line is printed:

  ```text
  Enter passphrase:
  note: discarded 2 line(s) typed after the passphrase (not run, not used):
    curl exa… (2 words, 16 chars)
    echo hel… (3 words, 16 chars)
  ```

The GUI never relies on your typing `-` at such a prompt: it has no
terminal to prompt on, which is why a `-` in a GUI field is refused.

## The reveal toggle — deliberate, display-only exposure {#secret-reveal-toggle}

Every secret-class field masks its value on load (see Defense 2's
"masked dots"). Since `mnemonic-gui-v0.57.0` each such field also carries
a small **reveal button** so you can *deliberately* check what you
typed — verifying a long BIP-39 phrase or passphrase against a paper
backup is a real need, and forcing a re-type is worse UX than a bounded,
opt-in reveal. In the structural form renders (the `.gui` gallery) the
affordance shows as a trailing `[reveal]` marker on each masked secret
row, e.g. `--passphrase text (secret) -> <masked> [reveal]`; that marker
is the eye button, not part of the value.

The reveal is **bounded and opt-in by construction**:

- **Hold-to-reveal is the primary interaction.** Press and hold the eye
  (pointer button down) to unmask; release to re-mask. A pointer **tap
  does not latch** — reveal lasts only while you hold.
- **A bounded latch backs keyboard / accessibility use.** Activating the
  eye by keyboard or an assistive-technology click latches the reveal so
  keyboard-only and screen-reader users are not required to hold a
  pointer button. There is **no timeout** — the latch is released by the
  auto-hide triggers below, not a timer.
- **Exactly one field can be revealed at a time.** Revealing a second
  field re-masks the first (a single-revealed-field invariant).
- **Auto-hide is aggressive.** The reveal (hold or latch) clears the
  moment you click **Run**, when the field loses focus, when the window
  loses focus (you Alt-Tab away), or when you switch tab or subcommand.
  There is a one-frame window on window-focus-loss where the field can
  still read as revealed before the next repaint masks it; treat a
  revealed field as on-screen until you have looked away and back.

Crucially, the reveal is **display-only and never widens any other
surface.** Regardless of whether a field is revealed:

- the **run-confirm modal** (Defense 2) still renders every secret token
  as `••••`;
- the output panel's **`argv:` echo** never shows the secret, and the
  **Copy command** text never contains it (see
  [What you see](#secret-channels-what-you-see));
- the **paste-warn** modal, the **never-persist** invariant (Defense 1),
  and the **on-exit zeroize sweep** (Defense 3) are entirely unaffected —
  the reveal state is transient UI chrome, not part of `FormState`, so it
  cannot reach disk.

The reveal appears on the primary masked-secret widgets — single-line
secret fields, secret slot rows, and secret composite value fields. It is
deliberately **not** wired to the build-descriptor tree key fields yet
(those mask value-conditionally on an xprv-shaped key; tracked as a
fast-follow), so a masked tree key has no eye at v0.57.0.

## Pasting secrets — the paste-warn modal

When you paste into a secret-class text field for the first time in
a session, a `paste-warn` modal asks for explicit confirmation. The
modal reminds you that:

- The paste source (clipboard) may have a content history readable
  by other applications.
- Some platform clipboards are *synced* (iCloud, Windows Cloud
  Clipboard) and may have transmitted the secret to another device.
- Typing the secret directly is safer than pasting in most threat
  models — though slower and more error-prone.

You can opt out for the rest of the session via the modal's
"don't warn again" checkbox; the opt-out does not persist across
sessions (the modal returns on next launch). Per-flag suppression
is **not** offered (would create an inconsistent default state across
forms).

## What the GUI deliberately does NOT do

- The GUI does not **echo** typed secrets back to you in plaintext by
  default. Secret text fields render as masked dots; the passive
  confirmation is the count of characters typed. The one exception is the
  deliberate, opt-in [reveal toggle](#secret-reveal-toggle) — a
  hold-to-reveal affordance you actuate on a single field; it is
  display-only and never widens any other surface (the run-confirm modal,
  the `argv:` echo / copy-command, persistence, and the exit sweep all
  stay masked regardless).
- The GUI does not log secrets. Tracing output to stderr (via
  `--debug` or `RUST_LOG`) strips secret values from log lines;
  the test suite verifies no secret literals survive the tracing
  formatter.
- The GUI does not write secrets to its session-state JSON. See
  Defense 1.
