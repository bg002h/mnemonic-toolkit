# Glossary {#glossary}

Terse definitions for v1.0 of the GUI manual. Mirrors the CLI
manual's glossary where concepts overlap; adds the GUI-specific
terms (form widgets, conditional-visibility, run-confirm modal).

## bundle\index{bundle}

The output of `mnemonic bundle`: the three engraving-ready cards
(`ms1`, one or more `mk1`, one `md1`) that together recover a
wallet. The cycle is "synthesise → engrave → verify-bundle →
store"; the cards travel separately and recombine via the
codecs' cross-binding metadata.

## card

One of the three card types in the m-format constellation. The
**`ms1` card** is the BIP-39 entropy backup (secret-bearing); the
**`mk1` card** is the xpub backup (public); the **`md1` card** is
the wallet-policy template backup (public). Each card type ships
its own SHA-pinned codec (`ms-codec`, `mk-codec`, `md-codec`) and
its own CLI (`ms`, `mk`, `md`).

## m-format constellation

The umbrella term for the four crates + four CLIs of this
project: the seed-card codec (`ms`), the xpub-card codec (`mk`),
the wallet-policy-card codec (`md`), and the integration toolkit
(`mnemonic`). Each card encodes one role of a complete wallet
backup; the codecs share a common alphabet, error-correction
discipline, and engravability constraint.

## ms1\index{ms1}

A string in the form `ms10entr…` encoding the BIP-39 entropy
bytes for a single seed under the BIP-93 / codex32 envelope.
Carrying the `ms1` recovers the wallet. See
[chapter 60](#ms-per-tab-reference).

## mk1\index{mk1}

A string (or chunked set of strings) in the form `mk1q…`
encoding an extended public key plus origin metadata plus one
or more `policy_id_stub` bindings. Public material; sharing it
exposes chain-watch capability but not spend capability. See
[chapter 70](#mk-per-tab-reference).

## md1\index{md1}

A string in the form `md1zsx…` encoding a BIP-388 wallet-policy
template plus the bound-public-key references that the bundle's
`mk1` cards carry. Public material. See
[chapter 50](#md-per-tab-reference).

## slot

A bundle input row in the GUI form: one row per cosigner in a
multisig template, each carrying that cosigner's seed input
(`@N.phrase=...`, `@N.ms1=...`, `@N.xpub=...`, `@N.xprv=...`).
Single-sig templates have a single slot `@0`; an M-of-N multisig
has N slots `@0` through `@N-1`. The slot row's secret-bearing
sub-fields trigger the run-confirm modal.

## SubcommandSchema

The Rust type at `mnemonic-gui/src/schema/mod.rs::SubcommandSchema`
that describes one subcommand's form shape: the human-name shown
in the subcommand selector ComboBox, the list of FlagSchemas, the
positional-args, the `allows_slots` boolean, and the optional
`conditional` function. The four `schema/{mnemonic,md,ms,mk}.rs`
modules each ship a `const SCHEMA: Schema` containing one
`SubcommandSchema` per supported subcommand.

## FlagSchema

The Rust type at `mnemonic-gui/src/schema/mod.rs::FlagSchema` that
describes one flag's form widget: name (with leading dashes), one
of the `FlagKind` widget variants (Boolean / Text / Path /
Dropdown / NodeValueComposite / TaggedOrIndexed), `required` /
`repeating` flags, help-text, and the `secret` boolean that
gates the run-confirm modal.

## NodeValueComposite

A `FlagKind` variant used for `--from <type>=<value>` style
flags (e.g. `mnemonic convert --from`, `mnemonic derive-child
--from`). The GUI renders a Dropdown (the type-tag) plus a
context-sensitive value widget (Text / SecretLineEdit / Path)
that swaps shape based on the chosen tag. Six subcommands use
this widget across the `mnemonic` tab (`convert`,
`derive-child`, `slip39-split`, `seed-xor-split`,
`seed-xor-combine`, `final-word`).

## TaggedOrIndexed

A `FlagKind` variant used for flags accepting either a literal
tag value OR an `@N` cosigner-index value. Currently only
`mnemonic export-wallet --taproot-internal-key` uses this — tag
`nums` (BIP-341 NUMS point) or `@N` (cosigner N's xpub).

## conditional-visibility\index{conditional-visibility}

The per-subcommand state machine at
`mnemonic-gui/src/form/conditional.rs` that elevates or suppresses
flag widgets based on the current form state. Each conditional
function returns a `FlagVisibility` (a vec of `(flag, Visibility)`
overrides where `Visibility ∈ {Required, Disabled, Hidden}`)
that the form layer applies before rendering. See
[chapter 83](#form-and-output) for surprises.

## run-confirm modal\index{run-confirm modal}

The modal dialog that the GUI surfaces between **Run** click and
subprocess spawn whenever any secret-bearing flag has a non-empty
value (per `mnemonic-gui/src/secrets.rs::should_confirm_run`).
Confirms the user's intent before the secret material leaves the
form. See [§14 Defense 2](#secret-handling) for the threat-model
rationale and the v0.3.0 redaction gap.

## SecretLineEdit\index{SecretLineEdit}

The form widget for `secret: true` flag inputs. Wraps egui's
`TextEdit::password(true)` to mask the rendered characters; the
underlying value lives only in egui's internal buffer and is
never persisted to disk by the GUI.

## codex32

The BIP-93 string encoding (HRP + threshold + share-index +
tag + payload + BCH checksum) that all three m-format cards
build on. The `ms1` strings ARE codex32 strings (HRP `ms`); the
`mk1` and `md1` cards use codex32-adjacent envelopes with
distinct alphabets and BCH polynomials. See the CLI manual's
codex32 primer (Appendix E) for the wire-format details.

## BIP-39\index{BIP-39}

The 12 / 15 / 18 / 21 / 24-word mnemonic encoding of wallet
entropy. The `ms` family operates on BIP-39 entropy bytes and
phrases.

## passphrase\index{passphrase}

The optional BIP-39 mnemonic-extension passphrase ("the 25th
word"). Distinct from a wallet password; produces a wholly
distinct wallet from the same phrase. The `mnemonic bundle` /
`verify-bundle` / `convert` / `derive-child` workflows accept it
via the `--passphrase` (or `--passphrase-stdin`) flag pair.

## cross-binding\index{cross-binding}

The `policy_id_stub` bytes embedded in `mk1` cards that bind the
xpub to a specific `md1` wallet-policy template. The `bundle`
workflow computes the binding from the active `md1` and emits
matching `mk1`s; `verify-bundle` re-derives the binding from the
re-synthesised cards and asserts byte-identity against the input
cards.

## mnemonic-gui\index{mnemonic-gui}

The GUI application this manual documents. Source at
`https://github.com/bg002h/mnemonic-gui`. Spawns subprocesses for
the four sibling CLIs; ships its own schema modules
(`schema/{mnemonic,md,ms,mk}.rs`) that describe each CLI tab.
v0.3.0 is the version this manual is pinned against.

## Wayland\index{Wayland}

The Linux display-server protocol. The GUI runs under both X11
and Wayland; the `winit` ≥ 0.30 backend auto-selects from
`WAYLAND_DISPLAY` / `DISPLAY`. Wayland adds the
`ext-screencopy` exposure surface for screen capture not present
on X11; see [§84 screenshots](#secrets-and-os).

## screenreader\index{screenreader}

OS-level assistive-tech voice-output. Orca (Linux), NVDA
(Windows), VoiceOver (macOS). The `SecretLineEdit` widget
advertises itself as a password-input via AccessKit; per-app
screenreader policy varies. See [§84](#secrets-and-os).

## clipboard\index{clipboard}

The OS clipboard. The GUI's **Copy command** button copies the
constructed argv to the clipboard for terminal-paste; OS-level
clipboard history (KDE Klipper, Universal Clipboard, Windows
Clipboard History) may retain the copied text after the GUI
closes. See [§84](#secrets-and-os).

## screenshot\index{screenshot}

OS-level window-capture facility. macOS and Windows builds of
the GUI install OS-level capture exclusion at startup via
`platform.rs::apply_window_capture_protection`; Linux has no
compositor-level analogue at v0.3.0.
