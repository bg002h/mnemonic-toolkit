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

The modal **redacts secret values**: every argv token that carries a
secret (the BIP-39 phrase, `ms1` string, passphrase, `--share`, raw
`minikey` / `xprv` / WIF, or a secret `--from <node>=<value>` token)
renders as a fixed `••••` sentinel, not in plaintext. The literal
secret is never drawn on screen in the confirmation modal. Internally
the GUI builds a parallel display-mask alongside the real argv and
substitutes the sentinel for each masked token; the *unredacted* argv
is still what spawns when you click **Run**.

**Residual exposure — flag names are still visible; only the secret
VALUE is masked.** The modal shows e.g. `--passphrase ••••` or
`--share ••••`: the flag NAME appears in cleartext, only its secret
value is replaced by the sentinel. (For composite `--from
<node>=<value>` tokens the entire `node=value` token is masked, so
even the `phrase=` / `minikey=` prefix is hidden in that one case.)
The presence of a secret-class flag — and therefore the *fact* that
you are running a secret-bearing invocation — remains observable to
anything that can read the screen, even though the secret bytes
themselves do not. The mask is a *redaction* of the on-screen value,
not proof the secret has left no other trace: it is still in process
memory until the on-exit zeroize sweep runs (see Defense 3), and the
*unredacted* argv is what is actually passed to the spawned subprocess
— so on a shared or multi-user host the secret is briefly observable in
that child's `/proc/<pid>/cmdline` (or `ps`) exactly as a direct CLI
invocation would be. The modal redaction closes the *on-screen* exposure
only; closing the spawned-argv exposure (rewriting secret values to an
`@env:`-style channel) is tracked separately and not yet shipped.

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

- The GUI does not **echo** typed secrets back to you in plaintext.
  Secret text fields render as masked dots; the only confirmation
  is the count of characters typed.
- The GUI does not log secrets. Tracing output to stderr (via
  `--debug` or `RUST_LOG`) strips secret values from log lines;
  the test suite verifies no secret literals survive the tracing
  formatter.
- The GUI does not write secrets to its session-state JSON. See
  Defense 1.
