# `mnemonic-toolkit` v0.1 Design Spec — engraving-bundle CLI for the m-format star

**Status:** v0.1 surface locked (brainstorm converged 2026-05-04 after r2 architect review: 0 critical / 0 important on r2). Reference implementation: `crates/mnemonic-toolkit/`.
**Companion documents:**

- ms1 wire format / ms-codec library SPEC: [`bg002h/mnemonic-secret`](https://github.com/bg002h/mnemonic-secret) `design/SPEC_ms_v0_1.md`
- ms-cli surface: [`bg002h/mnemonic-secret`](https://github.com/bg002h/mnemonic-secret) `design/SPEC_ms_cli_v0_1.md` — direct sibling precedent for CLI shape, error/JSON discipline, engraving framing.
- mk1 wire format / mk-codec library SPEC: [`bg002h/mnemonic-key`](https://github.com/bg002h/mnemonic-key) `design/SPEC_mk_v0_1.md`
- md1 wire format / md-codec library: [`bg002h/descriptor-mnemonic`](https://github.com/bg002h/descriptor-mnemonic) — `bip/bip-mnemonic-descriptor.mediawiki` and `crates/md-codec/`.
- BRAINSTORM (rationale chain): conversation transcript only, per the workflow refinement that brainstorm reviews persist in transcripts; the architect r1/r2 findings and their integrations are summarized in §9.

This document specifies the user-facing CLI surface for `mnemonic` and the **bundle synthesis rules** that bind ms1 + mk1 + md1 cards together. It does not re-specify any sibling wire format.

---

## §1. Scope

`mnemonic` is a single binary (`crates/mnemonic-toolkit/src/main.rs` → `mnemonic` executable) that takes a BIP-39 phrase (or watch-only xpub) and emits a complete steel-engravable bundle of three sibling cards:

| Card | HRP | Format | Carries |
|---|---|---|---|
| **ms1** | `ms` | `ms-codec` | BIP-39 entropy bytes |
| **mk1** | `mk` | `mk-codec` | xpub + origin (master fp + BIP-32 path) + policy stub |
| **md1** | `md` | `md-codec` | wallet policy (template tree + bound xpub) |

The three cards are mutually self-checking: each carries its own BCH checksum from its sibling codec, and the toolkit cross-binds them via the 4-byte `policy_id_stub` (= `SHA-256(canonical wallet-policy preimage)[0..4]`) carried on each mk1 card.

In scope for v0.1:

- Two subcommands: `bundle` (encode-side: emit a 3-card bundle) and `verify-bundle` (round-trip integrity check).
- Two input modes per command: **full mode** (`--phrase`, derives xpub) and **key-only / watch-only mode** (`--xpub --master-fingerprint`, no entropy known).
- Four single-sig wallet-policy templates: BIP-44 (`pkh`), BIP-49 (`sh-wpkh`), BIP-84 (`wpkh`), BIP-86 (`tr`).
- Networks: `mainnet`, `testnet`, `signet`, `regtest`. Network selection drives BIP-32 coin-type and xpub version-byte cross-checks.
- Account index hardcoded to `0` (forward-pointer for v0.2: `--account` flag).
- All 10 BIP-39 wordlists supported via `--language` (default `english`, with non-suppressible stderr warning per ms-cli precedent).

Out of scope for v0.1 (deferred to mnemonic-toolkit v0.2+ or never):

- **Multisig templates** (BIP-48 `wsh-multi`, `sh-wsh-multi`, sortedmulti, k-of-n threshold). md1's typed-struct `Descriptor` supports them; v0.2 adds toolkit-side template selection and per-`@N` xpub sourcing.
- **Multi-account / non-zero account** (`--account` flag).
- **Custom origin paths** beyond the canonical BIP-44/49/84/86 single-sig forms.
- **File output** (`--output <dir>`, PDF/SVG card layouts). v0.1 emits to stdout only.
- **Recovery flow** (decode 3 strings → produce an xpriv / wallet-importable artifact). The 3-card-decode-then-rebuild path is a v0.3+ concern.
- **K-of-N share encoding within ms1** (= ms-codec v0.2 territory; toolkit's bundle command will gain share-split/share-combine in lockstep).
- **Color / formatting / interactive prompts.** Pure batch tool.

### §1.1 Engraving as the load-bearing user persona

Same framing as ms-cli SPEC §1.1: every CLI decision is judged against "does this make a steel-plate backup more correct, or less?" Concretely:

- Stdout has explicit `# ms1` / `# mk1` / `# md1` section headers so an engraver can plate-by-plate copy each in turn without conflating them.
- Each card section emits the canonical-form string AND its sibling-codec's chunked engraving form (5-char groups for ms1, mk1's chunked form for mk1, md1's chunked form for md1).
- Stderr emits a byte-exact engraving card with template/network/account/path metadata so the engraver records the wallet's restoration parameters alongside the cards.
- `verify-bundle` lets the engraver round-trip the engraved strings (typed-back from the plates) against the original phrase or xpub before believing the backup.

---

## §2. Command surface

### §2.1 `mnemonic bundle` — emit a 3-card engraving bundle

```text
mnemonic bundle --phrase <words>           --network <net> --template <t> [--language <lang>] [--passphrase <p>] [--json] [--no-engraving-card]
mnemonic bundle --xpub <xpub> --master-fingerprint <fp> --network <net> --template <t>                                  [--json] [--no-engraving-card]
```

`--phrase` and `--xpub` are **mode selectors** and form a clap mutually-exclusive group; exactly one must be supplied. The set of admissible auxiliary flags differs by mode.

#### §2.1.1 Full mode (`--phrase ...`)

Required: `--phrase`, `--network`, `--template`.
Optional: `--language` (default `english`), `--passphrase`, `--json`, `--no-engraving-card`.
Forbidden in full mode (rejected with exit 2 BadInput): `--xpub`, `--master-fingerprint`.

`--phrase -` reads the phrase from stdin (whitespace-normalized; see §3.2).

#### §2.1.2 Watch-only mode (`--xpub ... --master-fingerprint ...`)

Required: `--xpub`, `--master-fingerprint`, `--network`, `--template`.
Optional: `--json`, `--no-engraving-card`.
Forbidden in watch-only mode (rejected with exit 2 BadInput, byte-exact messages in §7.5):
- `--phrase` — "the phrase is not known in xpub-only mode"
- `--passphrase` — "`--passphrase` is incompatible with `--xpub`: the xpub is already a post-passphrase derivation product (the passphrase is baked into the xpub at engrave time)."
- `--language` — "`--language` is meaningful only with `--phrase`; xpub-only mode does not consult any wordlist"

`--xpub` does **not** accept stdin (`-`): xpubs are short (~111 chars), no stdin path is needed, and disallowing it removes a class of stdin-conflict bugs surfaced by ms-cli Phase 4.

#### §2.1.3 `--template` enum

| Value | BIP | Wrapper | Purpose tag in path |
|---|---|---|---|
| `bip44` | 44 | `pkh(@0)` | 44' |
| `bip49` | 49 | `sh(wpkh(@0))` | 49' |
| `bip84` | 84 | `wpkh(@0)` | 84' |
| `bip86` | 86 | `tr(@0)` | 86' |

Other values rejected by clap with exit 64.

#### §2.1.4 `--network` enum + path coin-type

| `--network` | Coin type | xpub version (mainnet/testnet bytes) |
|---|---|---|
| `mainnet` | `0'` | `0x0488B21E` |
| `testnet` | `1'` | `0x043587CF` |
| `signet` | `1'` | `0x043587CF` |
| `regtest` | `1'` | `0x043587CF` |

The coin-type drives the BIP-32 origin path; the xpub version-byte set is the same for testnet/signet/regtest (per `bitcoin = "0.32"` `bip32::Xpub::NetworkKind`).

#### §2.1.5 `--master-fingerprint` format

8 hex characters, case-insensitive, no `0x` prefix. Mapped to `bitcoin::bip32::Fingerprint`. Other forms rejected with exit 2 BadInput; byte-exact message: `"--master-fingerprint must be 8 hex chars (e.g., deadbeef)"`. (Resolves r2-I4.)

#### §2.1.6 `--passphrase` (full mode only)

If supplied, mixed into BIP-32 master seed derivation per BIP-39 PBKDF2. The toolkit emits a **non-suppressible stderr warning** because the passphrase is not engraved on any card and a forgotten passphrase is unrecoverable from the bundle alone:

```text
warning: --passphrase set; the passphrase is NOT engraved on any card and must
warning: be remembered separately. A forgotten passphrase is unrecoverable from
warning: the engraved bundle.
```

(`--passphrase -` reads from stdin; concurrent `--phrase -` + `--passphrase -` is rejected with exit 2 BadInput, byte-exact `"only one of --phrase and --passphrase may read from stdin"`.)

### §2.2 `mnemonic verify-bundle` — round-trip integrity check

```text
mnemonic verify-bundle --phrase <words>           --network <net> --template <t> [--language <lang>] [--passphrase <p>] --ms1 <s> --mk1 <s>... --md1 <s>...
mnemonic verify-bundle --xpub <xpub> --master-fingerprint <fp> --network <net> --template <t>                          --mk1 <s>... --md1 <s>...
```

The `--mk1` and `--md1` flags are **repeatable** (clap `num_args = 1..` ; ≥ 1 required, otherwise exit 64 clap usage error). Each appearance contributes one chunk; mk-codec's `decode(&[&str])` and md-codec's `chunk::reassemble(&[&str])` handle multi-chunk reassembly. v0.1 single-sig is typically single-chunk per card, so the common case is one `--mk1` and one `--md1` flag, but the SPEC accepts N. (Resolves SPEC r1-L9.)

`--ms1` is **single-string** in v0.1 (ms-codec v0.1 emits one string total per card; multi-string K-of-N share encoding lands in v0.2 and the SPEC then reframes `--ms1` as repeatable).

`verify-bundle` does NOT emit an engraving card; `--no-engraving-card` is not a flag of this command (rejected with exit 64 clap usage error if supplied).

#### §2.2.1 Full-mode verify-bundle (`--phrase`)

Five checks, in order. Each runs to completion before the next; failures are reported batch-style in JSON mode, first-failure-wins in text mode.

1. **Re-derive xpub** from `--phrase` + `--passphrase` + `--network` + `--template` (§4 derivation rules).
2. **Decode `--ms1`** via `ms_codec::decode`; assert decoded entropy equals the entropy derived from `--phrase` and `--language`.
3. **Decode `--mk1...`** via `mk_codec::decode(&[&str])`; assert decoded `xpub == derived_xpub` AND `origin_fingerprint == Some(derived_master_fingerprint)` AND `origin_path` matches the template's BIP path.
4. **Decode `--md1...`** via `md_codec::chunk::reassemble(&[&str])`; assert the descriptor is in **wallet-policy mode** (`tlv.pubkeys.is_some() && !.is_empty()`) AND the policy's bound xpub (after re-extracting 65 bytes per §4.6.1) equals `derived_xpub` AND the descriptor's wrapper shape matches the template (e.g., `Tag::Pkh` for bip44).
5. **Cross-binding:** `compute_wallet_policy_id(&decoded_md1).as_bytes()[0..4] == decoded_mk1.policy_id_stubs[0]`.

On full mismatch: exit 4 `BundleMismatch{card}`, message includes the offending card identifier and the v0.2 forward-pointer hint (`"v0.1 hardcodes account=0; if the engraved bundle was produced with a non-zero account, mismatch is expected — re-run with v0.2's --account flag once available"`). (Resolves r1-I5.)

#### §2.2.2 Watch-only-mode verify-bundle (`--xpub --master-fingerprint`)

Four checks, in order. Watch-only cannot verify the xpub is at the claimed BIP-32 path because the master seed is unknown — the SPEC explicitly warns:

```text
warning: watch-only verify-bundle does not verify --xpub is actually at the
warning: claimed BIP path m/<purpose>'/<coin>'/0' (no master seed available
warning: for re-derivation). Use --phrase mode for end-to-end verification.
```

The four checks (resolves r1-I3):

1. **mk1 parses + BCH valid** (`mk_codec::decode(&[&str])` succeeds).
2. **md1 parses + BCH valid** (`md_codec::chunk::reassemble(&[&str])` succeeds).
3. **Cross-binding stub linkage:** `compute_wallet_policy_id(&decoded_md1).as_bytes()[0..4] == decoded_mk1.policy_id_stubs[0]`.
4. **Optional xpub/fingerprint match:** if `--xpub` matches `decoded_mk1.xpub` AND `--master-fingerprint` matches `decoded_mk1.origin_fingerprint`. (Always reported; failure surfaces via exit 4.)

### §2.3 `--help` text (locked)

Top-level (`mnemonic --help`):

```text
mnemonic — engraving-bundle CLI for the m-format star (ms1 + mk1 + md1)

USAGE:
    mnemonic <COMMAND>

COMMANDS:
    bundle           emit a 3-card engraving bundle from a phrase or xpub
    verify-bundle    round-trip-check an engraved bundle

OPTIONS:
    -h, --help       print help
    -V, --version    print version

Each command supports --help for its full flag set.
```

`mnemonic bundle --help` and `mnemonic verify-bundle --help`: clap-derive default rendering of the flag tables specified in §2.1 / §2.2. (Pinned byte-exactly per integration-test fixture; see §10.)

---

## §3. Input/output discipline

### §3.1 Stdout vs stderr conventions

- **stdout** is the bundle output (multi-line text or JSON; see §5).
- **stderr** is engraving cards, language warnings, passphrase warnings, watch-only warnings.
- **Errors** print to stderr as `error: <message>` (matches ms-cli SPEC §3.1).
- `--json` mode routes the bundle to stdout as a single JSON document; engraving-card text moves to a `engraving_card` field within that JSON; stderr is reserved for warnings and errors only.

### §3.2 Stdin uniform behavior

Same shape as ms-cli SPEC §3.2:

- `--phrase -` or `--passphrase -` reads from stdin, whitespace-normalized via `read_phrase_input` (collapse runs of whitespace to single spaces; preserve word boundaries). NOT the `strip_whitespace` ms-cli uniform — phrases need spaces preserved between words.
- For `verify-bundle`, the `--ms1`, `--mk1`, `--md1` flags do NOT support `-` (their values are short ms1/mk1/md1 strings supplied directly on argv; multi-chunk handled by repetition).
- Concurrent `-` reads across `--phrase` and `--passphrase` are rejected (only one stdin reader allowed).

### §3.3 No interactive prompts

Pure batch tool. No `read_password`-style prompts. If an input is missing, exit 64 (clap usage) or exit 2 BadInput.

---

## §4. Bundle synthesis rules

This section specifies how a single (`phrase` OR (`xpub` + `master_fingerprint`)) input plus `(network, template)` produces three cards. The toolkit's whole correctness story rides on these rules being byte-deterministic.

### §4.1 Derivation in full mode

1. `mnemonic = bip39::Mnemonic::parse_in(language, phrase)?` — validates the BIP-39 4-bit checksum.
2. `entropy = mnemonic.to_entropy()` — 16/20/24/28/32 bytes.
3. `seed = mnemonic.to_seed(passphrase.unwrap_or(""))` — 64-byte BIP-32 master seed via PBKDF2. Note: `--passphrase ""` (empty string) and an absent `--passphrase` produce the same master seed (BIP-39 PBKDF2 with empty-passphrase suffix); the toolkit treats the two as equivalent and emits no passphrase warning when `--passphrase` is unset OR explicitly empty.
4. `master = bitcoin::bip32::Xpriv::new_master(network_kind, &seed)?`.
5. `master_fingerprint = master.fingerprint(&secp)` — 4 bytes.
6. `origin_path = template.origin_path(network)` (table in §4.2).
7. `account_xpriv = master.derive_priv(&secp, &origin_path)?`.
8. `account_xpub = bitcoin::bip32::Xpub::from_priv(&secp, &account_xpriv)`.

After step 5, in `--phrase` mode there is no separate user-supplied master fingerprint; step 5's computed value is used everywhere.

### §4.2 Origin paths per (template, network)

| Template | Mainnet | Non-mainnet (testnet/signet/regtest) |
|---|---|---|
| bip44 | `m/44'/0'/0'` | `m/44'/1'/0'` |
| bip49 | `m/49'/0'/0'` | `m/49'/1'/0'` |
| bip84 | `m/84'/0'/0'` | `m/84'/1'/0'` |
| bip86 | `m/86'/0'/0'` | `m/86'/1'/0'` |

Eight distinct paths total. `account` is hardcoded `0` per §1 / Q5.

### §4.3 Network ↔ xpub-version cross-check

The `bitcoin::bip32::Xpub` struct exposes a `NetworkKind` (mainnet / testnet) via its `network` field (see Phase 1 spike — exact field/method name validated against `bitcoin = "0.32"` source before SPEC compliance lands).

In **full mode**, the derived xpub's network is forced to match `--network` by step 4 of §4.1. Belt-and-braces: after derivation, assert `derived_xpub.network == network_kind_for(--network)`; mismatch is an internal-bug exit (exit 2 with `"derived-xpub network <X> does not match --network <Y>; this is a toolkit bug"`).

In **watch-only mode**, the user-supplied `--xpub` parses via `Xpub::from_str`. If `xpub.network != network_kind_for(--network)`, exit 2 BadInput with byte-exact `"xpub network <X> does not match --network <Y>"`. (Resolves r1-I1.)

### §4.4 ms1 card synthesis (full mode only; omitted in watch-only)

```text
let payload = ms_codec::Payload::Entr(entropy.to_vec());
let ms1_string = ms_codec::encode(ms_codec::Tag::ENTR, &payload)?;
```

Entropy length is one of {16, 20, 24, 28, 32}; `ms_codec` library validates and the encoder cannot produce an invalid `entr` payload by construction.

### §4.5 mk1 card synthesis

The `mk_codec::KeyCard` struct (verified against `crates/mk-codec/src/key_card.rs`):

```rust
KeyCard {
    policy_id_stubs: vec![<derived_4_bytes>],
    origin_fingerprint: Some(<master_fingerprint>),    // see §4.5.1 below
    origin_path:        <DerivationPath from §4.2>,
    xpub:               <Xpub from §4.1 step 8 OR --xpub literal>,
}
```

The `policy_id_stubs[0]` is computed AFTER the md1 descriptor is constructed (§4.6) — toolkit builds md1 first, then hashes it for the stub, then assembles mk1.

`origin_fingerprint`:
- Full mode: `Some(master_fingerprint)` from §4.1 step 5 (computed from the master xpriv).
- Watch-only mode: `Some(--master-fingerprint)` parsed from the user flag — **authoritative**: the user-supplied `--master-fingerprint` value is used verbatim in BOTH `mk1.origin_fingerprint` AND `md1.tlv.fingerprints[0]`. The toolkit does NOT attempt to derive a fingerprint from the xpub (xpubs identify the parent's fingerprint via `Xpub::parent_fingerprint`, not the master); deriving the master fingerprint from an xpub is generally impossible. Watch-only verify-bundle's check 4 cross-checks `--master-fingerprint` against `decoded_mk1.origin_fingerprint`. (Resolves SPEC r1-I1.)

Encoding:

```text
let mk1_strings: Vec<String> = mk_codec::encode(&keycard)?;
```

`mk_codec::encode` returns `Vec<String>` (multi-chunk capable). v0.1 single-sig is typically single-chunk; SPEC §5 layout iterates the Vec for forward-compat. (Resolves r2-Nit-1.)

#### §4.5.1 `origin_fingerprint` is always `Some(_)`, never `None`

mk-codec supports `origin_fingerprint: None` as a privacy-preserving mode (closure Q-8 of mk-codec). v0.1 toolkit does NOT use that mode: every emitted mk1 card carries the master fingerprint. The privacy-preserving option is a v0.2+ toolkit flag (`--privacy-preserving`).

### §4.6 md1 card synthesis (typed-struct construction)

The md-codec `Descriptor` is a typed struct (`crates/md-codec/src/encode.rs:17-28`); toolkit constructs it field-by-field, NOT by string-templating. (Resolves r1-C2.)

```rust
use md_codec::{Descriptor, encode::encode_string, tlv::TlvSection,
               origin_path::{PathDecl, PathDeclPaths, OriginPath, PathComponent},
               use_site_path::UseSitePath,
               tree::{Node, Body},
               tag::Tag};

// Per (template, network):
let descriptor = Descriptor {
    n: 1,
    path_decl: PathDecl {
        n: 1,
        paths: PathDeclPaths::Shared(template.origin_path(network).into()),
    },
    use_site_path: UseSitePath::standard_multipath(),  // <0;1>/*
    tree: Node {
        tag: template.wrapper_tag(),                    // Tag::Pkh / Tag::ShWpkh / Tag::Wpkh / Tag::Tr
        body: template.wrapper_body(0),                 // Body::KeyArg{index:0} or Body::Tr{key_index:0, tree:None}
    },
    tlv: TlvSection {
        use_site_path_overrides: None,
        fingerprints: Some(vec![(0, master_fingerprint.to_bytes())]),  // [u8; 4]
        pubkeys: Some(vec![(0, xpub_to_65_bytes(&xpub))]),             // [u8; 65]
        origin_path_overrides: None,
        unknown: Vec::new(),
    },
};
```

The descriptor satisfies `Descriptor::is_wallet_policy() == true` (resolves r1-C3): `tlv.pubkeys.is_some() && !.is_empty()`.

#### §4.6.1 xpub byte-format transform (resolves r2-I1)

mk-codec carries `bitcoin::bip32::Xpub` (full 78-byte BIP-32 form). md-codec's `tlv.pubkeys` carries `[u8; 65]` = chain code (32 B) || compressed pubkey (33 B):

```rust
fn xpub_to_65_bytes(xpub: &bitcoin::bip32::Xpub) -> [u8; 65] {
    let mut out = [0u8; 65];
    out[0..32].copy_from_slice(&xpub.chain_code.to_bytes());
    out[32..65].copy_from_slice(&xpub.public_key.serialize());
    out
}
```

Phase 1 spike validates the exact `Xpub::chain_code.to_bytes()` and `Xpub::public_key.serialize()` API surface against `bitcoin = "0.32"` source.

#### §4.6.2 v0.1 emits `PathDeclPaths::Shared` only (resolves r2-I2)

Single-sig has one xpub (`@0`), so `path_decl.paths = PathDeclPaths::Shared(...)`. v0.2 multisig will add `PathDeclPaths::Divergent(...)` when cosigners use distinct paths; that path drives md-codec's `Tag::OriginPaths = 0x36` TLV. v0.1 toolkit MUST NOT emit `Divergent`.

#### §4.6.3 Per-template wrapper tags + bodies

| Template | `Node.tag` | `Node.body` |
|---|---|---|
| bip44 | `Tag::Pkh` | `Body::KeyArg { index: 0 }` |
| bip49 | (sh-wrapped wpkh — needs nested Node; see below) | |
| bip84 | `Tag::Wpkh` | `Body::KeyArg { index: 0 }` |
| bip86 | `Tag::Tr` | `Body::Tr { key_index: 0, tree: None }` (keypath-only) |

bip49 nests `wpkh(@0)` inside `sh(...)`:

```rust
Node {
    tag: Tag::Sh,
    body: Body::Children(vec![
        Node { tag: Tag::Wpkh, body: Body::KeyArg { index: 0 } },
    ]),
}
```

Phase 1 spike validates `Tag::Sh` / `Tag::Pkh` / `Tag::Wpkh` / `Tag::Tr` exist with these names in `crates/md-codec/src/tag.rs`.

#### §4.6.4 md1 encoding

md-codec exposes two string-layer encoders (verified against `crates/md-codec/src/lib.rs:36-38`):

- `encode_md1_string(&Descriptor) -> Result<String, Error>` — single canonical-form string (codex32-wrapped). Suitable when the descriptor fits a single codex32 length bracket.
- `chunk::split(&Descriptor) -> Result<Vec<String>, Error>` — produces a chunked form (`Vec<String>` of length ≥ 1) for any descriptor; v0.1 single-sig is expected to be a single-element vec but the API returns `Vec<String>` symmetrically with mk-codec's `encode`.

v0.1 toolkit uses `chunk::split` for symmetry with mk1 + forward-compat with v0.2 multisig (which may produce longer descriptors that overflow single-string brackets):

```rust
let md1_strings: Vec<String> = md_codec::chunk::split(&descriptor)?;
```

Decode side uses `chunk::reassemble(&[&str]) -> Result<Descriptor, Error>` accordingly. Phase 1 spike validates the single-element-Vec assumption for all 8 (template × network) v0.1 cells.

### §4.7 Cross-binding invariants

After §4.4–§4.6 produce the three cards, the toolkit asserts internal consistency before emitting. Failure is an exit-2 toolkit-bug (these are unreachable for inputs that passed validation):

1. `compute_wallet_policy_id(&descriptor).as_bytes()[0..4] == keycard.policy_id_stubs[0]` (consequence of how `policy_id_stubs[0]` was computed in §4.5).
2. `descriptor.is_wallet_policy()` is true.

(A `--self-check` flag that immediately re-decodes the just-emitted bundle and re-runs §2.2.1's full check suite is deferred to v0.2; see §8.)

### §4.8 Xpub depth advisory (resolves r2-L3)

In watch-only mode, after `Xpub::from_str(--xpub)` the toolkit asserts `xpub.depth == 3` (3 hardened components from master: purpose / coin / account). Mismatch is a **soft warning**, not an error:

```text
warning: --xpub depth is <N>; expected 3 for canonical BIP-44/49/84/86 paths.
warning: Bundle will still be emitted; verify your wallet uses a non-standard path.
```

Some users may intentionally supply non-standard xpubs (e.g., a sub-account xpub for testing). The toolkit warns but does not reject.

**Watch-only account-index hazard:** the xpub bytes do not encode the account index used during derivation; v0.1 hardcodes `account = 0` in the bundle's mk1 origin path and md1 path declaration. If the user's xpub was actually derived at a non-zero account (e.g., `m/84'/0'/5'`), the bundle silently misrepresents the wallet's restoration path. The toolkit emits an additional non-suppressible stderr warning in watch-only mode:

```text
warning: watch-only mode hardcodes account=0; if your xpub was derived
warning: at a non-zero account, the bundle's path will not match. Use
warning: v0.2's --account flag once available.
```

---

## §5. Output format

### §5.1 Default text-mode stdout layout (`bundle` command)

Multi-section, one section per card. Section header is a `# ` line followed by the canonical-form string AND its chunked engraving form.

**Full mode:**

```text
# ms1 (entropy, BCH-checksummed)
ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f

ms10en trsqqq qqqqq qqqqq qqqqq qqqqq qqqqq qqqqc j9sxr aq34v7f

# mk1 (xpub + origin)
mk10...

mk10... -... -... -...

# md1 (wallet policy)
md10...

md10... -... -... -...
```

The chunked form for each section uses that sibling codec's chunked-form rendering (5-char groups for ms1; mk-codec's chunked-form output for mk1; md-codec's `render_codex32_grouped` for md1). The blank line between canonical form and chunked form is mandatory (matches ms-cli SPEC §4).

**Watch-only mode:**

```text
# ms1 (omitted — xpub-only mode)

# mk1 (xpub + origin)
mk10...
...

# md1 (wallet policy)
md10...
...
```

The byte-exact `# ms1 (omitted — xpub-only mode)\n` line plus following blank line is required (resolves r2-L2 — pinned to integration-test fixtures).

### §5.2 Engraving stderr card (suppressed by `--no-engraving-card`)

Byte-exact lines (resolves r2-L2):

**Full mode without passphrase:**

```text
network: <mainnet|testnet|signet|regtest>
template: <bip44|bip49|bip84|bip86>
account: 0
origin path: m/<purpose>'/<coin>'/0'
master fingerprint: <8 hex chars>
language: <english|...> (BIP-39 checksum valid)
passphrase: not used
engrave each card on its own plate. record this card alongside.
```

**Full mode with passphrase:**

```text
network: <mainnet|testnet|signet|regtest>
template: <bip44|bip49|bip84|bip86>
account: 0
origin path: m/<purpose>'/<coin>'/0'
master fingerprint: <8 hex chars>
language: <english|...> (BIP-39 checksum valid)
passphrase: USED — not engraved on any card; record separately and never lose it.
engrave each card on its own plate. record this card alongside.
```

**Watch-only mode:**

```text
network: <mainnet|testnet|signet|regtest>
template: <bip44|bip49|bip84|bip86>
account: 0
origin path: m/<purpose>'/<coin>'/0'
master fingerprint: <8 hex chars>
mode: watch-only (xpub-supplied; no entropy known to toolkit)
ms1 card omitted; recover entropy from the original wallet's other backup.
engrave each card on its own plate. record this card alongside.
```

**Stderr emission order** (locked; integration fixtures depend on this):

1. `--language` defaulting warning (if applicable; full mode only).
2. `--passphrase` set warning (if applicable; full mode only).
3. Watch-only mode warning (if applicable; watch-only mode only).
4. The engraving card (suppressed by `--no-engraving-card`).

Errors print before any of the above and short-circuit the run.

### §5.3 `bundle --json` schema

**Field order is part of the schema** (serde_json preserves struct insertion order; integration fixtures assert byte-exact JSON):

```json
{
  "schema_version": "1",
  "mode": "full" | "watch-only",
  "network": "mainnet",
  "template": "bip84",
  "account": 0,
  "origin_path": "m/84'/0'/0'",
  "master_fingerprint": "deadbeef",
  "ms1": "ms1...",                 // null in watch-only mode
  "mk1": ["mk10..."],              // Vec<String> per mk_codec::encode
  "md1": ["md10..."],              // Vec<String> per md_codec::chunk::split
  "engraving_card": "network: ...\ntemplate: ...\n..."
}
```

`--json` and `--no-engraving-card` are independent. With `--json` alone, `engraving_card` is populated. With both, `engraving_card` is `null`.

### §5.4 `verify-bundle --json` schema

**Field order is part of the schema** (same rule as §5.3):

```json
{
  "schema_version": "1",
  "result": "ok" | "mismatch",
  "checks": [
    { "name": "ms1_entropy_match",     "result": "ok" | "fail" | "skipped", "detail": "..." },
    { "name": "mk1_decode",            "result": "ok" | "fail",             "detail": "..." },
    { "name": "mk1_xpub_match",        "result": "ok" | "fail" | "skipped", "detail": "..." },
    { "name": "mk1_fingerprint_match", "result": "ok" | "fail" | "skipped", "detail": "..." },
    { "name": "mk1_path_match",        "result": "ok" | "fail" | "skipped", "detail": "..." },
    { "name": "md1_decode",            "result": "ok" | "fail",             "detail": "..." },
    { "name": "md1_wallet_policy",     "result": "ok" | "fail",             "detail": "..." },
    { "name": "md1_xpub_match",        "result": "ok" | "fail" | "skipped", "detail": "..." },
    { "name": "stub_linkage",          "result": "ok" | "fail",             "detail": "..." }
  ]
}
```

`skipped` covers checks not applicable in watch-only mode (entropy/path-rederivation). **Exit code is 4 when `result == "mismatch"` and 0 when `result == "ok"`** — consumers must check both stdout JSON and exit code.

**Sibling-codec decode failures inside `verify-bundle`** (e.g., `--mk1` BCH uncorrectable) surface as `checks[i].result = "fail"` with diagnostic detail; the run still emits the §5.4 envelope with `result: "mismatch"`. Only pre-decode failures (mode violations, `--master-fingerprint` parse error, network / xpub mismatch in watch-only) route to the §5.5 error envelope.

### §5.5 Error JSON envelope

Same shape as ms-cli SPEC §5.4:

```json
{
  "schema_version": "1",
  "error": {
    "kind": "BadInput" | "Bip39" | "Bitcoin" | "MsCodec" | "MkCodec" | "MdCodec" | "BundleMismatch" | "ModeViolation",
    "exit_code": 1 | 2 | 3 | 4,
    "message": "...",
    "details": { ... } | null
  }
}
```

---

## §6. Errors and exit codes

### §6.1 Exit-code table

| Code | Meaning |
|---|---|
| 0 | success |
| 1 | user-input error (bad hex, bad path, BIP-39 parse, xpub parse, etc.) |
| 2 | format violation / mode violation (xpub-passphrase combo, network/xpub mismatch, etc.) |
| 3 | valid-but-future-format (reserved-not-yet-emitted variant from any sibling codec) |
| 4 | verify mismatch (`verify-bundle` round-trip failed; `BundleMismatch{card}`) |
| 64 | clap usage error (overrides clap's default 2 to keep format-violation distinct) |

### §6.2 `ToolkitError` enum

```rust
#[derive(Debug)]
#[non_exhaustive]
pub enum ToolkitError {
    BadInput(String),
    Bip39(bip39::Error),
    Bitcoin(BitcoinErrorKind),                // wraps bitcoin::bip32::Error / Xpub::from_str / Fingerprint::from_str
    MsCodec(ms_codec::Error),
    MkCodec(mk_codec::Error),
    MdCodec(md_codec::Error),
    ModeViolation { mode: &'static str, flag: &'static str, message: &'static str },
    BundleMismatch { card: &'static str, message: String },
    NetworkMismatch { xpub_network: &'static str, expected: &'static str },
    FutureFormat { source: &'static str, detail: String },  // routes any sibling-codec "reserved-not-emitted" variant to exit 3
}
```

### §6.3 Exit-code mapping

| Variant | Exit |
|---|---|
| `BadInput`, `Bip39`, `Bitcoin`, `MsCodec`, `MkCodec`, `MdCodec` (most variants) | 1 |
| `ModeViolation`, `NetworkMismatch`, `MsCodec::WrongHrp`/etc. format-violation variants | 2 |
| `FutureFormat` (any sibling's reserved-not-emitted variant) | 3 |
| `BundleMismatch` | 4 |

Per-sibling format-violation routing is enumerated in §6.4 below.

### §6.4 Friendly mappers + dispatch tables

#### §6.4.0 Exit-code routing principle (locked)

Across all five friendly mappers below, exit-code routing follows two rules:

- **Exit 1 (user-input):** length-bracket violations, hex-parse failures, BIP-39 word-set violations, BIP-32 derivation-input errors. These are typo-correctable: the user can re-engrave or re-type and try again.
- **Exit 2 (format violation):** structural / wire-format violations (wrong HRP, malformed payload padding, BCH uncorrectable, reserved bits set, mode-violation flag combinations, network/xpub mismatch). The string is the right shape but its bits are wrong.
- **Exit 3 (future format):** any sibling-codec "reserved-not-emitted" variant (e.g., `ms_codec::Error::ReservedTagNotEmittedInV01`, `mk_codec::Error::UnsupportedVersion`, `md_codec::Error::UnsupportedVersion`). Routes to `ToolkitError::FutureFormat`.
- **Exit 4 (verify mismatch):** only `ToolkitError::BundleMismatch{card}` from `verify-bundle`.

When a new sibling-codec variant lands in v0.X+, place it in the bucket whose principle it most closely matches. The fallthrough `_` arm for `#[non_exhaustive]` enums routes to exit 1 with `format!("unhandled <crate>::Error variant: {:?}", e)` — explicitly worse-message than a curated mapping, motivating mapper-table updates.

Five friendly mappers, one per error source:

#### §6.4.1 `friendly_bip39(&bip39::Error) -> String`

5 variants, mapped per ms-cli SPEC §6.2 (verbatim — this code is copy-paste-with-attribution from ms-cli):

| Variant | Message |
|---|---|
| `BadEntropyBitCount(n)` | "BIP-39 entropy bit count `n` invalid (must be 128, 160, 192, 224, or 256)" |
| `BadWordCount(n)` | "BIP-39 word count `n` invalid (must be 12, 15, 18, 21, or 24)" |
| `UnknownWord(idx)` | "unknown BIP-39 word at position `idx` (not in selected wordlist; did you pick the right --language?)" |
| `InvalidChecksum` | "BIP-39 checksum failure (last word does not match the entropy)" |
| `AmbiguousLanguages(_)` | "BIP-39 phrase parses under multiple wordlists; specify --language explicitly" |

#### §6.4.2 `friendly_bitcoin(&BitcoinErrorKind) -> String`

Wraps `bitcoin::bip32::Error` (variants: `CannotDeriveFromHardenedKey`, `Secp256k1`, `InvalidChildNumber`, `InvalidDerivationPathFormat`, `UnknownVersion`, `WrongExtendedKeyLength`, `Base58`, `Hex`, `InvalidPublicKeyHexLength`) + `Xpub::from_str` + `Fingerprint::from_str`. Phase 1 spike enumerates the actual `bitcoin = "0.32"` `bip32::Error` variant set.

#### §6.4.3 `friendly_ms_codec(&ms_codec::Error) -> String`

Delegates to ms-cli's existing dispatch table (§6.1.1 of ms-cli SPEC); ms_codec is `#[non_exhaustive]` so the mapper has a fallthrough `_` arm with `format!("unhandled ms_codec::Error variant: {:?}", other)`.

#### §6.4.4 `friendly_mk_codec(&mk_codec::Error) -> String`

mk_codec::Error variant set (verified against `crates/mk-codec/src/error.rs`):

| Variant | Routing | Message |
|---|---|---|
| `InvalidHrp(s)` | exit 2 | "wrong HRP: got `{s:?}`, expected \"mk\"" |
| `MixedCase` | exit 2 | "mixed case in mk1 input string" |
| `InvalidStringLength(n)` | exit 1 | "mk1 data-part length `n` not valid (regular code: 14-93; long code: 95-108; the gap at 94 is reserved-invalid)" |
| `InvalidChar { ch, position }` | exit 1 | "invalid character `ch` at position `position` (not in bech32 alphabet)" |
| `BchUncorrectable(s)` | exit 1 | "mk1 BCH uncorrectable: `s` (engraving error or transcription typo)" |
| `UnsupportedCardType(b)` | exit 2 | "mk1 unsupported card type: 0x`{b:02x}`" |
| `MalformedPayloadPadding` | exit 2 | "mk1 malformed payload padding" |
| `ChunkSetIdMismatch` | exit 2 | "mk1 chunk_set_id mismatch across chunks" |
| `ChunkedHeaderMalformed(s)` | exit 2 | "mk1 chunked-header malformed: `s`" |
| `MixedHeaderTypes` | exit 2 | "mk1 mixed string-layer header types" |
| `CrossChunkHashMismatch` | exit 2 | "mk1 cross-chunk integrity hash mismatch" |
| `UnsupportedVersion(v)` | exit 3 | "mk1 unsupported version: `v` (toolkit v0.1 reads version 0 only)" |
| `ReservedBitsSet` | exit 2 | "mk1 reserved bits set in bytecode header" |
| `InvalidPolicyIdStubCount` | exit 2 | "mk1 policy_id_stub_count must be ≥ 1" |
| `InvalidPathIndicator(b)` | exit 2 | "mk1 invalid path indicator byte: 0x`{b:02x}`" |
| `PathTooDeep(n)` | exit 2 | "mk1 path too deep: `n` components (max 10)" |
| `InvalidPathComponent(s)` | exit 2 | "mk1 invalid path component: `s`" |
| `InvalidXpubVersion(v)` | exit 2 | "mk1 invalid xpub version: 0x`{v:08x}`" |
| `InvalidXpubPublicKey(s)` | exit 2 | "mk1 invalid xpub public key: `s`" |
| `UnexpectedEnd` | exit 2 | "mk1 unexpected end of bytecode" |
| `TrailingBytes` | exit 2 | "mk1 trailing bytes after xpub" |
| `CardPayloadTooLarge { .. }` | exit 2 | "mk1 card payload too large: `bytecode_len` > `max_supported`" |
| `_` (non_exhaustive fallthrough) | exit 1 | `format!("unhandled mk_codec::Error variant: {:?}", other)` |

#### §6.4.5 `friendly_md_codec(&md_codec::Error) -> String`

md_codec::Error is NOT `#[non_exhaustive]` (verified at `crates/md-codec/src/error.rs:6`); the variant set is closed and exhaustive matching is required (no `_` wildcard arm). Routing per §6.4.0 principle:

| Routing bucket | Variants (per `crates/md-codec/src/error.rs`) |
|---|---|
| **Exit 1 (user-input)** | `Codex32DecodeError`, `Codex32EncodeError` (codex32-layer typos / BCH-correction edge cases that the user can re-engrave) |
| **Exit 2 (format violation)** | `BitStreamTruncated`, `ReservedHeaderBitSet`, `PathDepthExceeded`, `KeyCountOutOfRange`, `DivergentPathCountMismatch`, `AltCountOutOfRange`, `UnknownPrimaryTag`, `UnknownExtensionTag`, `ThresholdOutOfRange`, `ChildCountOutOfRange`, `KGreaterThanN`, `TlvOrderingViolation`, `PlaceholderIndexOutOfRange`, `OverrideOrderViolation`, `EmptyTlvEntry`, `TlvLengthExceedsRemaining`, `PlaceholderNotReferenced`, `PlaceholderFirstOccurrenceOutOfOrder`, `MultipathAltCountMismatch`, `ForbiddenTapTreeLeaf`, `ChunkCountOutOfRange`, `ChunkIndexOutOfRange`, `ChunkSetIdOutOfRange`, `ChunkHeaderChunkedFlagMissing`, `ChunkCountExceedsMax`, `ChunkSetEmpty`, `ChunkSetInconsistent`, `ChunkSetIncomplete`, `ChunkIndexGap`, `ChunkSetIdMismatch`, `VarintOverflow`, `MissingExplicitOrigin`, `InvalidPresenceByte`, `InvalidXpubBytes`, `MissingPubkey`, `ChainIndexOutOfRange`, `HardenedPublicDerivation`, `UnsupportedDerivationShape` |
| **Exit 3 (future format)** | `UnsupportedVersion` |

Per-variant message text is locked in Phase 1 task 1.1 (parallel to ms-cli's `crates/ms-cli/src/codex32_friendly.rs` shape). Routing above is the SPEC contract; messages may be refined for clarity without re-opening this SPEC.

### §6.5 Display rules

Same as ms-cli SPEC §6.3:
- text mode: `error: <message>` to stderr, then exit code.
- `--json` mode: §5.5 envelope to stdout, then exit code.
- Exit code 64: clap usage errors (overrides clap's default 2).

### §6.6 Mode-violation messages (byte-exact)

Pinned byte-exact for integration tests (resolves r1-I6 + r2-I5). Mode-violation messages are **plain text without backticks** for ergonomic stderr rendering — backticks would appear literally in the terminal. The implementation pins these as `pub const` strings in `cmd/bundle::mode_text` (mirror imports for verify-bundle).

| Trigger | Routing | Message (byte-exact) |
|---|---|---|
| `--passphrase` with `--xpub` | ModeViolation → exit 2 | `--passphrase is incompatible with --xpub: the xpub is already a post-passphrase derivation product (the passphrase is baked into the xpub at engrave time).` |
| `--language` with `--xpub` | ModeViolation → exit 2 | `--language is meaningful only with --phrase; xpub-only mode does not consult any wordlist` |
| `--xpub` without `--master-fingerprint` | ModeViolation → exit 2 | `--xpub requires --master-fingerprint (xpub mode needs the master fingerprint to populate mk1's origin)` |
| `--master-fingerprint` without `--xpub` | ModeViolation → exit 2 | `--master-fingerprint is meaningful only with --xpub` |
| `--xpub -` (stdin sentinel for xpub) | BadInput → **exit 1** | `--xpub does not accept stdin (-); pass the xpub literally on argv` |
| `--phrase` with `--xpub` | clap mutual-exclusion → **exit 64** (clap default text) | (not byte-exact-pinned — usage error, not mode violation) |
| `verify-bundle` mode-violations | (same as `bundle`; symmetric mirror) | imports `cmd::bundle::mode_text` constants |

**Routing note:** `--xpub -` is exit 1 (BadInput) — it's a syntactic-input violation, not a mode mismatch, so it's the only row in this table that does NOT route through `ToolkitError::ModeViolation`. The message stays in `mode_text` for centralized pinning.

---

## §7. Engraving guidance

### §7.1 The three-card backup workflow

1. User runs `mnemonic bundle --phrase ...` (or `--xpub ...` for migration from existing wallet).
2. Toolkit emits ms1 + mk1 + md1 strings to stdout, plus engraving-card metadata to stderr.
3. User engraves EACH card on its OWN plate (one card per plate; cross-card mixing defeats the per-card BCH checksum's localizing property).
4. User records the engraving-card stderr text alongside the plates (paper or co-engraved metadata plate).
5. User round-trips by typing the engraved strings back into `mnemonic verify-bundle` BEFORE relying on the backup.

### §7.2 Watch-only restoration

If the user has only the xpub (e.g., already-existing wallet), they can still produce mk1 + md1 cards via watch-only mode. The ms1 entropy card must come from the wallet's BIP-39 backup (or the wallet's separate ms1 produced earlier). Watch-only's `verify-bundle` cannot prove the xpub is at the claimed BIP path — the user must trust the originating wallet's path declaration.

### §7.3 Passphrase hazard

A passphrase ("25th word") is mixed into the BIP-32 master seed via PBKDF2 but is NOT engraved on any card in v0.1. A forgotten passphrase makes the bundle alone insufficient for restoration. The non-suppressible stderr warning is the toolkit's UX gate. v0.1 deliberately does not propose a fourth-card passphrase carrier; the user records the passphrase separately.

### §7.4 Wordlist-language hazard

ms1 v0.1 does NOT carry the BIP-39 wordlist language on the wire (per ms-codec SPEC §6.3). Decoders default to English, and a user whose original wallet used a non-English wordlist will silently derive a different BIP-32 master seed. The toolkit's `--language` flag is the mitigation in **encode-side**; the engraving card records the language for future decoders. Users with non-English wallets MUST record the wordlist language alongside the cards.

---

## §8. Out-of-scope items deferred

| Feature | Tier | Tracker |
|---|---|---|
| Multisig templates (BIP-48 wsh-multi, sh-wsh-multi, sortedmulti, k-of-n) | v0.2 | toolkit `design/FOLLOWUPS.md` once repo lands |
| `--account` flag (non-zero account) | v0.2 | toolkit FOLLOWUPS |
| `--xpub`-input multisig (multiple xpubs + threshold) | v0.2 | toolkit FOLLOWUPS |
| `--output <dir>` (write 3 files instead of stdout) | v0.3 | toolkit FOLLOWUPS |
| PDF / SVG card layout | v0.3+ | toolkit FOLLOWUPS |
| Recovery flow (3 strings → wallet artifact / xpriv) | v0.3+ | toolkit FOLLOWUPS |
| `--privacy-preserving` (mk1 with `origin_fingerprint = None`) | v0.2+ | toolkit FOLLOWUPS |
| K-of-N share encoding (ms1 multi-string) | v0.2 lockstep with ms-codec v0.2 | cross-repo |
| `--self-check` flag (toolkit emits + immediately verifies internally; replaces v0.1's removed §4.7 invariant 3) | v0.2 | toolkit FOLLOWUPS |
| Color / interactive prompts | never | — |

---

## §9. Closures from brainstorm

### §9.1 Q1–Q5 closures (locked)

| Q | Closure | Rationale |
|---|---|---|
| Q1 (repo) | New repo `bg002h/mnemonic-toolkit` | clean separation, matches sibling-repo pattern |
| Q2 (headline cmd) | `mnemonic bundle` + `mnemonic verify-bundle` | encode + round-trip-check covers v0.1 needs |
| Q3 (input scope) | BIP-39 phrase OR (xpub + master_fingerprint); v0.1 single-sig only | "permissive-but-narrow"; key-only mode enables migration from existing wallets |
| Q4 (output) | multi-section stdout + engraving-card stderr | engraver workflow |
| Q5 (template select) | `--template` flag (bip44/49/84/86); account hardcoded 0 | YAGNI on multi-template; v0.2 expands |

### §9.2 r1 architect findings (resolved)

| Finding | Resolution |
|---|---|
| C1 — policy_id_stub source | `compute_wallet_policy_id(&Descriptor).as_bytes()[0..4]` (§4.5, §4.6) |
| C2 — descriptor synthesis | typed-struct construction (§4.6) |
| C3 — vocabulary | "wallet policy" not "descriptor template" (§4.6, applies throughout) |
| I1 — network/xpub mismatch | exit 2 BadInput, byte-exact message (§4.3, §6.6) |
| I2 — friendly mapper enum | full variant lists in §6.4 |
| I3 — watch-only verify checks | 4 checks listed in §2.2.2 |
| I4 — `--xpub -` disallowed | §2.1.2, §6.6 |
| I5 — account-0 hint in BundleMismatch | §2.2.1 |
| I6 — passphrase-with-xpub byte-exact text | §6.6 |

### §9.3 r2 architect findings (resolved)

| Finding | Resolution |
|---|---|
| I-r2-1 — xpub byte-format transform | §4.6.1 with worked code |
| I-r2-2 — `PathDeclPaths::Shared` only in v0.1 | §4.6.2 |
| I-r2-3 — `--mk1` / `--md1` repeatable | §2.2 |
| I-r2-4 — `--master-fingerprint` 8-hex format | §2.1.5 |
| I-r2-5 — `--language` xpub-incompatibility | §2.1.2, §6.6 |
| L1 — `id.as_bytes()[0..4]` pseudocode | §4.6, §4.7 |
| L2 — byte-exact engraving-card lines | §5.2 |
| L3 — xpub depth advisory | §4.8 |
| L4 — `_` wildcard arm for `#[non_exhaustive]` mappers | §6.4.3, §6.4.4 |

---

## §10. Reference implementation

### §10.0 Module dependency graph (build order)

5 phases, mirroring ms-cli SPEC §10.0.

**Phase 1 — leaves (no internal deps):**

- `error.rs` — `ToolkitError` enum + exit-code mapper.
- `language.rs` — `--language` enum (10 wordlists, default-warning behavior).
- `network.rs` — `--network` enum (4 networks) + `NetworkKind` mapping + xpub-version table.
- `template.rs` — `Template` enum (4 templates) + per-template wrapper-tag/body + `origin_path(network)` table.
- `format.rs` — chunked-form rendering (delegates to each sibling codec's renderer).
- `parse.rs` — stdin/argv input helpers (`read_phrase_input`, `read_input`, fingerprint parsing).

Phase 1 task 1.1 is a **verification spike** (no code lands): read `bitcoin = "0.32"` (`bip32::Xpub` field/method names, `bip32::Error` variant set, `Xpub::chain_code.to_bytes()`, `Xpub::public_key.serialize()`), `mk_codec::*` (`KeyCard` struct shape, `encode/decode` signatures, `Error` variant set), `md_codec::*` (`Descriptor` field shape, `Tag` variants, `Body` variants, `chunk::split` / `chunk::reassemble` signatures, `Error` variant set). Output: a memo at `design/agent-reports/spike-toolkit-v0_1-phase-1.md` confirming each SPEC §4 + §6 claim against actual source, OR a list of SPEC patches needed before code lands. The memo blocks Phase 2 from starting.

**Phase 2 — synthesis:**

- `derive.rs` — full-mode BIP-32 derivation chain (§4.1).
- `synthesize.rs` — bundle synthesis: ms1 / mk1 / md1 construction + cross-binding invariants (§4.4–§4.7).

**Phase 3 — commands:**

- `cmd/bundle.rs` — `bundle` subcommand wiring.
- `cmd/verify_bundle.rs` — `verify-bundle` subcommand wiring.
- `friendly.rs` — five friendly mappers (§6.4).

**Phase 4 — root:**

- `main.rs` — clap derive root + `ExitCode` dispatch.
- `cmd/mod.rs` — re-exports.

**Phase 5 — integration tests + release prep:**

- `crates/mnemonic-toolkit/tests/` — `assert_cmd`-based integration tests covering: bundle full mode (each template × each network, BIP-39 vector phrases), bundle watch-only mode, verify-bundle round-trip (full + watch), every mode-violation message, every friendly-mapper variant via fault injection, JSON envelope shape.
- `Cargo.toml` metadata bump (description, documentation, readme, keywords, categories), version 0.0.0 → 0.1.0, flip `publish = false` → remove or `true`, `cargo publish --dry-run` clean.

### §10.1 Test strategy

- Phase 1 modules: per-module unit tests; spike memo enforced before any other Phase 1 code lands.
- Phase 2 synthesize: round-trip property test — fix template + network + entropy, derive xpub, build all three cards, decode each, assert §4.7 invariants hold.
- Phase 3 commands: `assert_cmd` integration test per (subcommand × mode × outcome) cell.
- Phase 4 main: `--help` byte-exact fixtures (one per subcommand). `--version` ground-truth check.
- **Reference test vector (v0.1):** Trezor's canonical 24-word all-zero entropy mnemonic ("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art") drives at least one bundle fixture per (template × network) cell — 4 templates × 4 networks = 16 fixtures locked in `crates/mnemonic-toolkit/tests/vectors/v0_1/*.txt`. The byte-exact ms1 / mk1 / md1 outputs become the SHA-pinned regression corpus for v0.1.0.
- Per-phase opus reviewer-loop until 0 critical / 0 important findings; reports persist to `design/agent-reports/phase-X-<name>-review-rN.md`. Low/nit deferred to `design/FOLLOWUPS.md`.

### §10.2 CI gates

- `cargo build --workspace`
- `cargo clippy --all-targets -D warnings`
- `cargo fmt --check`
- `cargo test --workspace`
- `cargo publish --dry-run -p mnemonic-toolkit` — **skipped pre-crates.io**: toolkit's git-deps to ms-codec / mk-codec / md-codec block dry-run packaging until the three siblings publish to crates.io. Phase 5 documents the expected failure; the gate becomes mandatory only when `[dependencies]` flips from git tags to crates.io versions.

### §10.3 Dependency model

```toml
[dependencies]
ms-codec  = { git = "https://github.com/bg002h/mnemonic-secret",      tag = "ms-codec-v0.1.0" }
mk-codec  = { git = "https://github.com/bg002h/mnemonic-key",         tag = "mk-codec-v0.2.1" }
md-codec  = { git = "https://github.com/bg002h/descriptor-mnemonic",  tag = "md-codec-v0.16.1" }
bip39     = { version = "2", features = ["all-languages"] }
bitcoin   = "0.32"
clap      = { version = "4", features = ["derive"] }
hex       = "0.4"
serde     = { version = "1", features = ["derive"] }
serde_json = "1"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
```

Git-deps until siblings hit crates.io in lockstep; toolkit's own `cargo publish` is gated on all three siblings being on crates.io. v0.1.0 release tags `mnemonic-toolkit-v0.1.0`.

### §10.4 Module file layout (locked)

```
crates/mnemonic-toolkit/src/
├── main.rs                — clap derive root + ExitCode dispatch
├── cmd/
│   ├── mod.rs             — re-exports
│   ├── bundle.rs          — bundle subcommand
│   └── verify_bundle.rs   — verify-bundle subcommand
├── derive.rs              — BIP-32 derivation chain (full mode)
├── synthesize.rs          — bundle synthesis + cross-binding invariants
├── template.rs            — Template enum + origin paths + wrapper tags/bodies
├── network.rs             — Network enum + NetworkKind mapping + xpub-version table
├── language.rs            — --language enum (10 wordlists)
├── format.rs              — chunked-form rendering wrappers
├── parse.rs               — stdin/argv input helpers
├── friendly.rs            — five friendly mappers (bip39/bitcoin/ms_codec/mk_codec/md_codec)
└── error.rs               — ToolkitError enum + exit-code mapping
```

(Mirrors ms-cli's module layout; new files: `derive.rs`, `synthesize.rs`, `template.rs`, `network.rs`, plus `friendly.rs` consolidating the five sibling mappers — chosen over per-mapper files because each mapper is shorter than ms-cli's per-source mappers and keeping them adjacent makes routing-principle compliance auditable in one place.)

---

## §11. Cross-format and v0.x roadmap

The m-format star at v0.1.0:

| Crate | Version | Status |
|---|---|---|
| md-codec | 0.16.1 | shipped, on GitHub |
| mk-codec | 0.2.1 | shipped, on GitHub |
| ms-codec | 0.1.0 | shipped, on GitHub (crates.io: gated on user `cargo login`) |
| ms-cli | 0.1.0 | shipped, on GitHub (crates.io: gated) |
| mnemonic-toolkit | 0.1.0 | this SPEC |

v0.2 multisig lockstep:
- ms-codec v0.2: K-of-N share encoding (`0x00` reserved-prefix-byte → real share-grouping).
- mk-codec v0.3+: ↑ keep emitting single mk1 per cosigner; toolkit v0.2 emits N mk1 cards for N cosigners.
- md-codec v0.17+: per-`@N` divergent paths via `Tag::OriginPaths = 0x36` (already shipped in v0.10.0 — md-codec is ahead of the toolkit's needs).
- mnemonic-toolkit v0.2: `--template wsh-multi-2-of-3` etc.; `--cosigner` repeatable for xpub-only multisig; `--account <N>` for non-zero account.

---

## Appendix A — provenance

This SPEC was authored 2026-05-04 by the maintainer (bg002h) with Opus 4.7 via the standard repo workflow:

1. Brainstorm Q1–Q5 + watch-only-mode amendment (in conversation transcript).
2. Sections 1–6 of design presented inline; user approved with sequential Y answers.
3. Architect r1 review of brainstorm: 3 critical / 6 important / 4 nits — all integrated as design revisions.
4. Architect r2 review of brainstorm-with-r1-fixes: 0 critical / 5 important / 4 nits / 6 affirmations — r2 important items folded into this SPEC document directly.
5. SPEC drafted → architect SPEC reviewer-loop convergence is the next gate before plan-writing.

## Revision history

- **r1** (2026-05-04) — initial SPEC integrating brainstorm Q1–Q5 + brainstorm-r1 architect (C1/C2/C3/I1-I6/4nits) + brainstorm-r2 architect (5 important + 4 nits).
- **r2** (2026-05-04) — SPEC-architect-r1 fixes integrated inline: C1 (md_codec API rename `encode_md1_string`→`chunk::split` for symmetry), I1 (`--master-fingerprint` authoritative in watch-only), I2 (exit-routing principle locked in §6.4.0), L1 (drop §4.7 invariant 3 → §8 forward-pointer), L2 (md_codec routing buckets inlined), L3 (spike memo path locked), L4 (JSON field-order pinned), L5 (stderr ordering locked), L6 (--passphrase "" ≡ unset), L7 (verify-bundle mode-violation symmetry), L8 (verify-bundle no engraving card), L9 (--mk1/--md1 num_args=1..), L10 (Trezor 24-word vector pinned).
- **r3** (2026-05-04, this commit) — SPEC-architect-r2 polish: 0 critical / 0 important. Verbose r2-nit-5 (verify-bundle JSON exit-code semantics + sibling-decode-failure routing in §5.4), r2-nit-8 (watch-only account-index hazard in §4.8), r2-nit-10 (friendly.rs consolidation rationale in §10.4). Architect r2 explicitly authorized transition to plan-writing.
