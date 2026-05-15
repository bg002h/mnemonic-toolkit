# `mnemonic bundle` {#mnemonic-bundle}

Synthesise a 3-card engraving bundle (`ms1` + `mk1` + `md1`) from
a master BIP-39 phrase or other secret-bearing seed input. This is
the headline subcommand of the `mnemonic` tab and the entry point
for every fresh wallet the constellation handles. Inputs flow
through the `--slot` repeating flag (one occurrence per (slot,
subkey) tuple); outputs are the three engravable card strings on
stdout, with an optional human-readable engraving-card panel on
stderr and an optional JSON envelope when `--json` is set.

:::danger
The worked examples in this chapter use the canonical all-`abandon`
BIP-39 test vector. **Never engrave or fund** a wallet derived
from this phrase — chain watchers have swept it continuously since
2017. The [§14 Defense 2](#secret-handling) cold-node operational
warning applies to every secret-bearing invocation under this
subcommand: the GUI's v0.3.0 run-confirm modal renders
secret-bearing argv tokens in plaintext, including pasted BIP-39
phrases on the slot editor's secret-bearing rows. Operate on a
cold/airgapped machine.
:::

## Outline {#mnemonic-bundle-outline}

- [`--network`](#mnemonic-bundle-network) — Bitcoin network for derivations + address encoding (required)
- [`--template`](#mnemonic-bundle-template) — pre-built descriptor template (required unless `--descriptor*`)
- [`--descriptor`](#mnemonic-bundle-descriptor) — user-supplied BIP-388 descriptor (XOR with `--descriptor-file`)
- [`--descriptor-file`](#mnemonic-bundle-descriptor-file) — descriptor read from a single-line UTF-8 file
- [`--language`](#mnemonic-bundle-language) — BIP-39 wordlist (default `english`)
- [`--passphrase`](#mnemonic-bundle-passphrase) — BIP-39 mnemonic-extension passphrase (XOR with `--passphrase-stdin`)
- [`--passphrase-stdin`](#mnemonic-bundle-passphrase-stdin) — read `--passphrase` from stdin (raw, NULL-byte preserving)
- [`--account`](#mnemonic-bundle-account) — BIP-32 account index (default 0; refused under descriptor mode)
- [`--json`](#mnemonic-bundle-json) — emit envelope JSON (`ms1`/`mk1`/`md1` + metadata)
- [`--no-engraving-card`](#mnemonic-bundle-no-engraving-card) — suppress the human-readable engraving-card panel
- [`--multisig-path-family`](#mnemonic-bundle-multisig-path-family) — `bip48` or `bip87` (default `bip87`)
- [`--privacy-preserving`](#mnemonic-bundle-privacy-preserving) — suppress master fingerprint from `mk1` + engraving card
- [`--self-check`](#mnemonic-bundle-self-check) — re-parse the emitted bundle and verify round-trip
- [`--threshold`](#mnemonic-bundle-threshold) — multisig threshold K (1 ≤ K ≤ N ≤ 16)
- [`--slot`](#mnemonic-bundle-slot) — repeating; `@N.<subkey>=<value>` (the input grammar; rendered by the slot editor)

## `--network` {#mnemonic-bundle-network}

The Bitcoin network the bundle targets. Required, no default.
Determines BIP-32 chain coin-type (0 for mainnet; 1 for testnet /
signet / regtest), the address HRP / version bytes embedded in the
emitted descriptor, and the BCH-code variant the cards encode. The
GUI renders this flag as a Dropdown widget; the `?` help-icon
deep-links here.

### Outline {#mnemonic-bundle-network-outline}

- [`mainnet`](#mnemonic-bundle-network-mainnet)
- [`testnet`](#mnemonic-bundle-network-testnet)
- [`signet`](#mnemonic-bundle-network-signet)
- [`regtest`](#mnemonic-bundle-network-regtest)

### `mainnet` {#mnemonic-bundle-network-mainnet}

Production Bitcoin mainnet. BIP-44 coin-type 0 (`m/.../0'/.../`).
Address HRP `bc1` for SegWit; version 0x00 for P2PKH. Use this
for any wallet that will hold real funds.

### `testnet` {#mnemonic-bundle-network-testnet}

The legacy public test network. BIP-44 coin-type 1. Address HRP
`tb1`. Funds are valueless. Note: most production-track tooling
has migrated to `signet`; testnet remains for compatibility
testing only.

### `signet` {#mnemonic-bundle-network-signet}

The signature-secured test network (Bitcoin Core's preferred
post-2021 testnet). Coin-type 1. HRP `tb1`. Funds are valueless;
the signet-faucet at `https://signet.bc-2.jp/` is the canonical
way to acquire test sats.

### `regtest` {#mnemonic-bundle-network-regtest}

A locally-controlled regression-test network. Coin-type 1. HRP
`bcrt1`. Funds exist only on the local node (typically
`bitcoind -regtest` or Polar). Use for offline integration tests
that need a controllable chain.

## `--template` {#mnemonic-bundle-template}

The pre-built descriptor template the bundle materialises. Marked
**required** by the conditional-visibility engine unless
`--descriptor` or `--descriptor-file` is set; clap-level
`required_unless_present_any` enforces the same constraint.
The GUI Dropdown widget renders this flag with a `?` help-icon.

The 10 templates split into two families: 4 single-sig (`bip44`,
`bip49`, `bip84`, `bip86`) and 6 multisig (`wsh-multi` /
`wsh-sortedmulti` for legacy P2WSH multisig; `sh-wsh-multi` /
`sh-wsh-sortedmulti` for nested P2SH-P2WSH multisig; `tr-multi-a`
/ `tr-sortedmulti-a` for Taproot multisig). For multisig
templates, `--threshold` and the `--slot` editor's per-cosigner
rows become required.

### Outline {#mnemonic-bundle-template-outline}

- [`bip44`](#mnemonic-bundle-template-bip44)
- [`bip49`](#mnemonic-bundle-template-bip49)
- [`bip84`](#mnemonic-bundle-template-bip84)
- [`bip86`](#mnemonic-bundle-template-bip86)
- [`wsh-multi`](#mnemonic-bundle-template-wsh-multi)
- [`wsh-sortedmulti`](#mnemonic-bundle-template-wsh-sortedmulti)
- [`sh-wsh-multi`](#mnemonic-bundle-template-sh-wsh-multi)
- [`sh-wsh-sortedmulti`](#mnemonic-bundle-template-sh-wsh-sortedmulti)
- [`tr-multi-a`](#mnemonic-bundle-template-tr-multi-a)
- [`tr-sortedmulti-a`](#mnemonic-bundle-template-tr-sortedmulti-a)

### `bip44` {#mnemonic-bundle-template-bip44}

Legacy P2PKH single-sig (`m/44'/<coin>'/<account>'/...`). Address
prefix `1` on mainnet. Used for compatibility with very old wallet
software; new wallets should prefer `bip84` (native SegWit) for
lower fees.

### `bip49` {#mnemonic-bundle-template-bip49}

Nested P2SH-P2WPKH single-sig (`m/49'/<coin>'/<account>'/...`).
Address prefix `3` on mainnet. SegWit benefits with backwards
compatibility to non-SegWit-aware wallets.

### `bip84` {#mnemonic-bundle-template-bip84}

Native SegWit P2WPKH single-sig (`m/84'/<coin>'/<account>'/...`).
Address prefix `bc1q` on mainnet. The current default for most
single-sig wallets; lowest fee, modern address format.

### `bip86` {#mnemonic-bundle-template-bip86}

Taproot single-sig P2TR (`m/86'/<coin>'/<account>'/...`). Address
prefix `bc1p` on mainnet. Privacy-preserving (key-path spends
indistinguishable from script-path spends to outside observers);
cooperative-spend-only at the GUI level (no scripts beyond the
internal-key path).

### `wsh-multi` {#mnemonic-bundle-template-wsh-multi}

Native P2WSH multisig with a *strict-order* `multi(...)`
descriptor. Cosigner order is fixed by `--slot` index. Used when
script reproduction across implementations must be deterministic
relative to a known cosigner ordering.

### `wsh-sortedmulti` {#mnemonic-bundle-template-wsh-sortedmulti}

Native P2WSH multisig with `sortedmulti(...)`. Cosigner pubkeys
are lexicographically sorted at script-construction time, so
cosigner ordering at backup time is irrelevant for spending. The
preferred multisig variant for human-engraved backups; recovery
does not require remembering slot order.

### `sh-wsh-multi` {#mnemonic-bundle-template-sh-wsh-multi}

Nested P2SH-P2WSH multisig with strict-order `multi(...)`. Use
for backwards compatibility with cosigner wallets that don't
emit native-SegWit multisig descriptors. Strictly higher fees
than `wsh-multi`.

### `sh-wsh-sortedmulti` {#mnemonic-bundle-template-sh-wsh-sortedmulti}

Nested P2SH-P2WSH multisig with `sortedmulti(...)`. The legacy
backwards-compatible variant of `wsh-sortedmulti`.

### `tr-multi-a` {#mnemonic-bundle-template-tr-multi-a}

Taproot multisig with `multi_a(...)` (script-path spend; cosigner
order fixed). Privacy-preserving on cooperative spends (single
Schnorr signature aggregated from all cosigners); falls back to
script-path on uncooperative spends.

### `tr-sortedmulti-a` {#mnemonic-bundle-template-tr-sortedmulti-a}

Taproot multisig with `sortedmulti_a(...)`. The order-agnostic
variant of `tr-multi-a`; preferred for human-engraved backups
where cosigner slot-ordering may drift.

## `--descriptor` {#mnemonic-bundle-descriptor}

A user-supplied BIP-388 descriptor string. Mutually-required-one-of
with `--template` (and clap-level conflicts with both `--template`
and `--descriptor-file`). When used, the conditional-visibility
engine disables `--descriptor-file`. Several other flags become
refused under descriptor-mode: `--threshold`,
`--multisig-path-family`, and any non-zero `--account` are not
permitted because the descriptor itself fully specifies them.

The GUI renders this flag as a plain Text widget with no `?`
help-icon (per [§33 Option C placement](#help-icons-and-deep-links-into-this-manual)
— Text fields rely on their hover tooltip).

## `--descriptor-file` {#mnemonic-bundle-descriptor-file}

Path to a single-line UTF-8 file containing the descriptor.
Tolerates a trailing newline. XOR with `--descriptor` (the GUI
disables one when the other has a value). Useful for descriptors
that exceed shell-quote-friendly length, or for sharing
descriptors via filesystem rather than copy-paste.

The GUI renders this flag as a Path widget; the OS file picker is
not yet wired (FOLLOWUP `gui-file-picker-affordance`); the field
is a plain Text widget that accepts a filesystem path string. No
`?` help-icon.

## `--language` {#mnemonic-bundle-language}

The BIP-39 wordlist used to interpret any phrase passed via
`--slot @N.phrase=<value>`. Optional; defaults to `english`. Same
10 allowed values as in [`mnemonic final-word
--language`](#mnemonic-final-word-language); the cross-tab
naming convention applies (`simplifiedchinese` etc., no hyphen
— compare `ms encode --lang chinese-simplified`).

### Outline {#mnemonic-bundle-language-outline}

- [`english`](#mnemonic-bundle-language-english)
- [`simplifiedchinese`](#mnemonic-bundle-language-simplifiedchinese)
- [`traditionalchinese`](#mnemonic-bundle-language-traditionalchinese)
- [`czech`](#mnemonic-bundle-language-czech)
- [`french`](#mnemonic-bundle-language-french)
- [`italian`](#mnemonic-bundle-language-italian)
- [`japanese`](#mnemonic-bundle-language-japanese)
- [`korean`](#mnemonic-bundle-language-korean)
- [`portuguese`](#mnemonic-bundle-language-portuguese)
- [`spanish`](#mnemonic-bundle-language-spanish)

### `english` {#mnemonic-bundle-language-english}

The BIP-39 English wordlist (2048 entries). Default when
`--language` is omitted.

### `simplifiedchinese` {#mnemonic-bundle-language-simplifiedchinese}

BIP-39 Simplified Chinese wordlist (UTF-8). The `mnemonic` tab
joins the qualifier as a single token. Cross-tab divergence with
`ms encode --lang chinese-simplified` is documented at
[`ms encode --lang`](#ms-encode-lang).

### `traditionalchinese` {#mnemonic-bundle-language-traditionalchinese}

BIP-39 Traditional Chinese wordlist.

### `czech` {#mnemonic-bundle-language-czech}

BIP-39 Czech wordlist.

### `french` {#mnemonic-bundle-language-french}

BIP-39 French wordlist.

### `italian` {#mnemonic-bundle-language-italian}

BIP-39 Italian wordlist.

### `japanese` {#mnemonic-bundle-language-japanese}

BIP-39 Japanese wordlist. ASCII-space separators accepted; the
canonical ideographic-space (U+3000) separator is normalised at
parse time.

### `korean` {#mnemonic-bundle-language-korean}

BIP-39 Korean wordlist.

### `portuguese` {#mnemonic-bundle-language-portuguese}

BIP-39 Portuguese wordlist.

### `spanish` {#mnemonic-bundle-language-spanish}

BIP-39 Spanish wordlist.

## `--passphrase` {#mnemonic-bundle-passphrase}

The optional BIP-39 mnemonic-extension passphrase (sometimes
called "the 25th word"). Concatenated with the phrase via PBKDF2
to produce the BIP-32 seed. Empty passphrase (default) is the
common case; non-empty passphrases produce a wholly distinct
wallet from the same phrase.

The GUI renders this as a `SecretLineEdit` widget (masked text
field). Schema-`secret: true`. Conditional: disabled by the
visibility engine when `--passphrase-stdin` is set (XOR pair). Any
non-empty value triggers the run-confirm modal at click-Run time.

## `--passphrase-stdin` {#mnemonic-bundle-passphrase-stdin}

Boolean flag. When set, the CLI reads the passphrase from stdin
(raw bytes, preserving NULL bytes; strips at most one trailing
`\r?\n`). The GUI surfaces stdin routing through the
secret-bearing widget; the passphrase does not appear in argv.

Schema-`secret: true` (the *flag* triggers the modal because its
presence implies a stdin secret will be passed). Conditional:
disabled when `--passphrase` is set. Single-stdin-per-invocation:
mutually exclusive with any `--slot @N.<secret>=-` row.

## `--account` {#mnemonic-bundle-account}

BIP-32 account index (default 0). Range 0..2_147_483_647 (the
hardened-derivation limit). The GUI renders this as a Number
widget; no `?` help-icon (Number widgets are not in the
help-icon class).

Refused under descriptor mode: when `--descriptor` or
`--descriptor-file` is set AND `--account != 0`, the CLI emits
`--account != 0 is meaningful only with --template; descriptor mode encodes account index in the @i origin path.`
(byte-exact mirror of `mode_text::DESCRIPTOR_WITH_NONZERO_ACCOUNT`
at `crates/mnemonic-toolkit/src/cmd/bundle.rs:91-101`).

## `--json` {#mnemonic-bundle-json}

Boolean flag. When set, emits the bundle as a JSON envelope on
stdout instead of the plain three-card text output. Envelope
shape (per `mnemonic-toolkit/src/format.rs` `BundleJson` struct;
field order is part of the schema, schema_version `"4"`):

```json
{
  "schema_version": "4",
  "mode": "full",
  "network": "mainnet",
  "template": "bip84",
  "descriptor": null,
  "account": 0,
  "origin_path": "m/84'/0'/0'",
  "origin_paths": null,
  "master_fingerprint": "73c5da0a",
  "ms1": ["ms10entrsq..."],
  "mk1": ["mk1qprsqhpqq..."],
  "md1": ["md1zsxdspq...", "md1zsxdspq..."],
  "multisig": null,
  "privacy_preserving": false
}
```

`mode` is `"full"` when the bundle includes secret-bearing slot
input (the master `ms1` is emitted) or `"watch-only"` when every
input slot is public (the `ms1` array becomes length-N empty
strings — sentinels marking the watch-only slots). `template` is
populated in template mode; `descriptor` is the verbatim
user-supplied string in descriptor mode (the two are mutually
`Option`-exclusive). `origin_path` and `origin_paths` are also
mutually exclusive — `origin_paths` (plural) appears only for
divergent-path multisig.

For multisig templates `mk1` becomes a nested array (one
sub-array per cosigner via the `MkField::Multi` discriminated
union; `serde(untagged)` flattens to bare nested arrays in JSON)
and `multisig` becomes a populated `MultisigInfo` block
(`template`, `threshold`, `cosigner_count`, `path_family`,
`cosigners[]`). `master_fingerprint` is `null` when
`--privacy-preserving` is set OR when the bundle is multisig
(per-cosigner fingerprints land in the `cosigners[]`
sub-objects).

## `--no-engraving-card` {#mnemonic-bundle-no-engraving-card}

Boolean flag. When set, suppresses the human-readable
engraving-card panel that the CLI normally emits to stderr
alongside the three card strings. Use this for piping the bundle
through downstream tooling without the panel cluttering stderr.

## `--multisig-path-family` {#mnemonic-bundle-multisig-path-family}

Dropdown. The BIP-32 path family used for multisig templates.
Two allowed values; default `bip87`.

### Outline {#mnemonic-bundle-multisig-path-family-outline}

- [`bip48`](#mnemonic-bundle-multisig-path-family-bip48)
- [`bip87`](#mnemonic-bundle-multisig-path-family-bip87)

### `bip48` {#mnemonic-bundle-multisig-path-family-bip48}

The BIP-48 multisig path family
(`m/48'/<coin>'/<account>'/<script-type>'/<address-type>'/...`).
The legacy multisig path; matches the convention used by older
hardware wallets and Sparrow's earlier multisig defaults.

### `bip87` {#mnemonic-bundle-multisig-path-family-bip87}

The BIP-87 multisig path family
(`m/87'/<coin>'/<account>'/...`). The current default; preferred
for new multisig setups and matches modern hardware-wallet
conventions.

Refused under descriptor mode (the descriptor specifies the
path family directly).

## `--privacy-preserving` {#mnemonic-bundle-privacy-preserving}

Boolean flag. When set, suppresses the master fingerprint from
the emitted `mk1` card AND from the engraving-card panel. Useful
when you want to engrave an `mk1` that does not link the card
back to the master seed via the well-known fingerprint
correspondence.

In multisig contexts, the per-cosigner fingerprints in the JSON
envelope's `cosigners[]` sub-objects are also nulled. The bundle
remains mathematically correct; only the fingerprint metadata is
suppressed.

## `--self-check` {#mnemonic-bundle-self-check}

Boolean flag. When set, the CLI re-parses the freshly-emitted
bundle (`ms1`/`mk1`/`md1`) end-to-end and asserts that the
re-derivation produces byte-identical card strings. Exits non-zero
on any drift. This is a runtime safety check; recommended for
production engraving workflows.

## `--threshold` {#mnemonic-bundle-threshold}

Number widget. Multisig threshold K — for a K-of-N bundle, set
this to K. Allowed range 1 to 16 inclusive. Refused under
descriptor mode AND under single-sig templates (the CLI emits
`--threshold is meaningful only with a multisig --template; single-sig templates ignore threshold.`
under a single-sig template — byte-exact mirror of
`mode_text::THRESHOLD_WITHOUT_MULTISIG`; under descriptor mode it
emits `--threshold is meaningful only with a multisig --template; descriptor mode encodes K directly.`
mirroring `mode_text::DESCRIPTOR_WITH_THRESHOLD`).

For multisig templates, the GUI's slot editor expects N cosigner
rows (where N = total slot rows after the master row); K is the
number that must sign to spend.

## `--slot` {#mnemonic-bundle-slot}

The repeating input flag. Grammar: `@N.<subkey>=<value>` per
occurrence; one occurrence per (slot, subkey) tuple. Allowed
subkeys (per `mnemonic-toolkit/src/slot_input.rs` `SlotSubkey`
enum, 8 variants in canonical order): `phrase`, `entropy`,
`xpub`, `master_xpub`, `fingerprint`, `path`, `wif`, `xprv`. For
secret-bearing subkeys (`phrase`, `entropy`, `wif`, `xprv`) the
suffix `=-` reads the value from stdin. The watch-only subkeys
(`xpub`, `master_xpub`, `fingerprint`, `path`) NEVER consume
stdin — those values are public so no argv-leakage protection
applies. `master_xpub` is an export-wallet-shaped optional add-on
to any watch-only set that already includes `xpub`; in pure
bundle invocations the canonical 7 are sufficient. (Note: the
toolkit's parser refusal message currently lists only 7 — see
`slot_input.rs::ParseError`'s `expected one of: phrase, entropy,
xpub, fingerprint, path, wif, xprv` — for historical reasons.
The 8th `master_xpub` subkey IS accepted; the prose lag is
tracked at design/FOLLOWUPS.md.)

The GUI renders the slot input not as a series of `--slot` text
fields but as a structured **slot editor** — a per-row table at
the bottom of the form (visible because `bundle` has
`allows_slots: true`). Each row carries:

- An **index** spinner (`@N`, drag-value 0..15).
- A **subkey** Dropdown (the 8 subkeys above).
- A **value** text field whose width and obscuration depend on
  the subkey choice (secret-bearing subkeys render as
  `SecretLineEdit`).
- An **✕** remove button.
- A **+ Add slot** button below the last row.

The repeating-flag `?` help-icon for `--slot` lives at the
"Slot rows:" label above the table. The argv assembler emits
`--slot @0.phrase=...` style tokens in slot-index ascending
order regardless of the order the user added the rows.

A single-sig bundle has exactly one slot row (e.g. `@0.phrase=...`).
A 2-of-3 multisig bundle has three slots (one per cosigner), each
of which contributes either a `phrase=` (you control the cosigner
seed) or an `xpub=` plus optional `fingerprint=` and `path=` (you
have the cosigner's public material only).

## Worked example — single-sig bundle from the canonical phrase

1. Switch to **mnemonic** tab; pick **Bundle (emit 3-card)** in
   the subcommand selector.
2. Set `--network` to `mainnet`, `--template` to `bip84`. Leave
   `--account` (defaults to 0), `--language`, and `--passphrase`
   empty.
3. **Clear `--multisig-path-family`** (it is seeded to `bip87`
   on first launch — see chapter 31's default-launch state).
   Leaving the seeded `bip87` value with a single-sig template
   triggers the `mode_text::PATH_FAMILY_WITHOUT_MULTISIG` refusal
   listed in the table below; the GUI's conditional-visibility
   engine does not yet auto-disable this flag for single-sig
   templates (FOLLOWUP `gui-bundle-multisig-flags-conditional`).
4. In the slot editor, the seeded default row is `@0` with
   subkey `xpub`. Change subkey to `phrase` (the row's value
   field flips to a masked `SecretLineEdit`).
5. Type or paste the canonical phrase into the value field:

   ```text
   abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about
   ```

6. The `Preview:` line should resemble:

   ```text
   mnemonic bundle --network mainnet --template bip84 --slot "@0.phrase=abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
   ```

7. Click **Run**. The run-confirm modal appears with the full
   slot row visible (the v0.3.0 redaction gap applies — see the
   `:::danger` admonition at the top of this chapter). Click
   **Run** in the modal.

The output panel renders the three card strings on stdout:

```text
ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f
mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4
mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh
md1zsxdspqqqpm6jzzqqvqz6qu79mg9p2sgfff6p2eph8wftp5uf6gqnlgzqqqnymv0
md1zsxdspq259s3jnsrcrhnlagpftrf9apnc3m9fy8uqfc85cha4nqnh5k67ey2hzyc
md1zsxdspqjd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nvqhuuyvzgaejah6
```

Plus the engraving-card panel on stderr (rendered in the output
panel's stderr region; suppress via `--no-engraving-card`).
The bundle round-trips: pass these three cards back into
`mnemonic verify-bundle` and the verifier asserts the seed
phrase + descriptor reconstruct the same three card strings.

## Refusals

The CLI refuses several inconsistent flag combinations with
mode-violation errors. The GUI does not pre-validate; it submits
the form and surfaces the CLI's refusal in the output panel's
stderr region.

All `mode:` strings below are byte-exact mirrors of the
`mode_text::*` constants at
`crates/mnemonic-toolkit/src/cmd/bundle.rs:91-101`. Drift is
SPEC §6.6 / §6.9 byte-pinned and integration-test-gated.

| Trigger | Refusal message |
|---|---|
| `--descriptor` AND `--template` | `--descriptor and --template are mutually exclusive; pick descriptor passthrough or template, not both.` |
| `--descriptor` AND `--descriptor-file` | `--descriptor and --descriptor-file are mutually exclusive; supply the descriptor inline or via file, not both.` |
| `--descriptor*` AND `--threshold` | `--threshold is meaningful only with a multisig --template; descriptor mode encodes K directly.` |
| `--descriptor*` AND `--multisig-path-family` | `--multisig-path-family is meaningful only with --template; descriptor mode encodes paths directly via @i/path syntax.` |
| `--descriptor*` AND `--account != 0` | `--account != 0 is meaningful only with --template; descriptor mode encodes account index in the @i origin path.` |
| `--threshold` AND single-sig template | `--threshold is meaningful only with a multisig --template; single-sig templates ignore threshold.` |
| `--multisig-path-family` AND single-sig template | `--multisig-path-family is meaningful only with a multisig --template.` |
| `--passphrase` AND `--passphrase-stdin` | clap-level `conflicts_with` error: `the argument '--passphrase-stdin' cannot be used with '--passphrase'` |

The conditional-visibility engine pre-disables several of these
combinations in the GUI form (`--descriptor` disables
`--descriptor-file` and vice versa; `--passphrase` disables
`--passphrase-stdin`); the disabled widgets are still visible
but greyed out, matching the egui `add_enabled_ui(false)`
affordance.

## Advisories

The CLI emits stderr advisories for non-fatal but security-relevant
conditions. The GUI surfaces these in the output panel's stderr
region after the run completes.

| Trigger | Stderr advisory |
|---|---|
| Inline `--slot @N.<secret>=<value>` (any of `phrase`, `entropy`, `wif`, `xprv`) | `warning: secret material on argv (--slot @N.<subkey>=) — pipe via --slot @N.<subkey>=- to avoid /proc/$PID/cmdline exposure` |
| Inline `--passphrase <value>` | `warning: secret material on argv (--passphrase) — use --passphrase-stdin to avoid /proc/$PID/cmdline exposure` |
| `--self-check` failure (re-derivation drift) | exits non-zero with `self-check failed: round-trip drift on <card-class>` |

Note that the GUI's command-line preview always uses the inline
form (the GUI does not pipe argv via stdin by default); the
argv-leakage advisories therefore fire on every secret-bearing
GUI run. The cold-node operational mitigation in [§14 Defense
2](#secret-handling) — only run the GUI on an airgapped machine
— is the primary defense against this exposure.
