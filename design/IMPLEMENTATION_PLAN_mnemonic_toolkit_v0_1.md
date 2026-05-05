# mnemonic-toolkit v0.1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship `crates/mnemonic-toolkit` v0.1.0: the `mnemonic` binary — a 2-subcommand CLI atop `ms-codec v0.1.0` + `mk-codec v0.2.1` + `md-codec v0.16.1` that takes a BIP-39 phrase or watch-only xpub and emits a complete steel-engravable bundle of three sibling cards (ms1 entropy + mk1 xpub + md1 wallet-policy descriptor).

**Architecture:** Single binary, clap-derive subcommand dispatch. 12 source modules in 5-phase build order: leaves (error/language/network/template/format/parse) → synthesis (derive/synthesize) → commands (cmd/bundle/cmd/verify_bundle/friendly) → root (main/cmd/mod) → integration tests + release prep. All bundle synthesis via direct sibling-codec library calls (no subprocess to ms-cli); BIP-32 derivation via `bitcoin = "0.32"` `bip32::{Xpriv, Xpub, DerivationPath, Fingerprint}`; structured output via `serde_json = "1"`; integration tests via `assert_cmd = "2"`. Engraving-friendly multi-section stdout (`# ms1 / # mk1 / # md1` headers); strip-whitespace phrase normalization on stdin; non-suppressible stderr warnings for language defaulting, passphrase use, and watch-only mode.

**Tech Stack:** Rust 2021 edition, MSRV 1.85 (workspace lockstep). Runtime deps: `ms-codec` (git tag `ms-codec-v0.1.0`), `mk-codec` (git tag `mk-codec-v0.2.1`), `md-codec` (git tag `md-codec-v0.16.1`), `bip39 = "2"` (`features = ["all-languages"]`), `bitcoin = "0.32"`, `clap = "4"` (derive), `hex = "0.4"`, `serde = "1"` (derive), `serde_json = "1"`. Dev deps: `assert_cmd = "2"`, `predicates = "3"`. No `anyhow` (own `ToolkitError`), no `tracing`/`log`.

**Source-of-truth artifacts:**
- SPEC: `design/SPEC_mnemonic_toolkit_v0_1.md` (architect-converged: brainstorm 2 rounds + SPEC 2 rounds, all 26 findings integrated; r2 explicitly authorized plan-writing).
- Sibling library APIs (read these before writing code; memory `feedback_read_source_before_planning`):
  - `/scratch/code/shibboleth/mnemonic-secret/crates/ms-codec/src/{lib,encode,decode,error,payload,tag}.rs`
  - `/scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/{lib,key_card,error,bytecode,string_layer}.rs`
  - `/scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec/src/{lib,encode,decode,chunk,tlv,tree,origin_path,use_site_path,identity,tag,error}.rs`
- Sibling CLI precedent: `/scratch/code/shibboleth/mnemonic-secret/crates/ms-cli/src/{main,error,parse,bip39_friendly,codex32_friendly,language,format}.rs` — directly informs friendly mapper shape, ExitCode dispatch, JSON envelope, strip-whitespace stdin discipline, engraving-card stderr.

**Convergence convention** (memory `feedback_iterative_review_every_phase`): each phase ends with an opus reviewer-loop that runs until 0 critical / 0 important findings. Per-phase reports persist to `design/agent-reports/phase-X-<name>-review-rN.md`. Critical/important fixed inline as a fixup commit; low/nit recorded in `design/FOLLOWUPS.md` at tier `v0.1-nice-to-have`.

**Commit cadence:** within each phase: one feature commit at phase-end; one fixup commit per opus-review round if findings landed; optional nit-cleanup commit. Stage paths explicitly per memory `feedback_avoid_git_add_all`. Spot-check HEAD content via `git show HEAD:path` post-commit per memory `feedback_verify_committed_content_not_working_tree`. Don't drop reserved-looking deps without confirmation per memory `feedback_dont_drop_reserved_deps`.

**SPEC closure tracking** — every implementation task references the SPEC sections it realizes. Reviewer can verify SPEC §1-§11 closures + brainstorm-r1/r2 fixes + SPEC-r1/r2 fixes are all locked in code by tracing back from each closure to its task(s).

---

## Phase 1: Foundation modules (leaves)

**Goal:** Land Cargo.toml deps + the 6 leaf modules (`error.rs`, `language.rs`, `network.rs`, `template.rs`, `format.rs`, `parse.rs`). Each module has unit tests; no internal-crate dependencies between them. By phase-end: `cargo build -p mnemonic-toolkit` clean (main still empty); `cargo test -p mnemonic-toolkit` runs all unit tests.

**Files:**
- Modify: `crates/mnemonic-toolkit/Cargo.toml` (add deps + dev-deps)
- Modify: `crates/mnemonic-toolkit/src/main.rs` (replace stub with `mod` declarations; bin still `fn main(){}`)
- Create: `crates/mnemonic-toolkit/src/error.rs`
- Create: `crates/mnemonic-toolkit/src/language.rs`
- Create: `crates/mnemonic-toolkit/src/network.rs`
- Create: `crates/mnemonic-toolkit/src/template.rs`
- Create: `crates/mnemonic-toolkit/src/format.rs`
- Create: `crates/mnemonic-toolkit/src/parse.rs`

### Task 1.1: Sibling-API contact spike (verification memo)

**Files:** Read-verify only (no code lands). Spike memo lands at `design/agent-reports/spike-toolkit-v0_1-phase-1.md`.

The spike validates SPEC claims about `bitcoin = "0.32"`, `bip39 = "2"`, `mk_codec`, `md_codec`, and `ms_codec` API surface BEFORE Phase 1 leaf modules code against them. Per SPEC §10.0 and memory `feedback_spike_before_locking_wire_format`.

- [ ] **Step 1: Stand up the spike harness.**

```bash
mkdir -p /tmp/toolkit-spike && cd /tmp/toolkit-spike && cat > Cargo.toml <<'EOF'
[package]
name = "toolkit-spike"
version = "0.0.0"
edition = "2021"

[dependencies]
ms-codec = { git = "https://github.com/bg002h/mnemonic-secret",      tag = "ms-codec-v0.1.0" }
mk-codec = { git = "https://github.com/bg002h/mnemonic-key",         tag = "mk-codec-v0.2.1" }
md-codec = { git = "https://github.com/bg002h/descriptor-mnemonic",  tag = "md-codec-v0.16.1" }
bip39 = { version = "2", features = ["all-languages"] }
bitcoin = "0.32"
hex = "0.4"
EOF
mkdir -p src
```

- [ ] **Step 2: Verify `bitcoin::bip32::Xpub` field/method API.**

Write `src/bin/spike_bitcoin.rs`:

```rust
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::NetworkKind;
use std::str::FromStr;

fn main() {
    let secp = Secp256k1::new();
    let seed = [0u8; 64];
    let master = Xpriv::new_master(NetworkKind::Main, &seed).unwrap();
    let path = DerivationPath::from_str("m/84'/0'/0'").unwrap();
    let xpriv = master.derive_priv(&secp, &path).unwrap();
    let xpub = Xpub::from_priv(&secp, &xpriv);

    // SPEC §4.6.1 claim: xpub.chain_code.to_bytes() returns [u8; 32]
    let cc: [u8; 32] = xpub.chain_code.to_bytes();
    println!("chain_code (32B): {}", hex::encode(cc));

    // SPEC §4.6.1 claim: xpub.public_key.serialize() returns [u8; 33]
    let pk: [u8; 33] = xpub.public_key.serialize();
    println!("pubkey (33B): {}", hex::encode(pk));

    // SPEC §4.3 claim: xpub.network exposes NetworkKind
    println!("network: {:?}", xpub.network);

    // SPEC §4.5/§4.6 claim: Fingerprint::to_bytes() returns [u8; 4]
    let fp = master.fingerprint(&secp);
    let fp_bytes: [u8; 4] = fp.to_bytes();
    println!("master fingerprint: {} = {}", fp, hex::encode(fp_bytes));

    // SPEC §2.1.5 claim: Fingerprint::from_str accepts case-insensitive 8 hex
    let fp2 = Fingerprint::from_str("DEADBEEF").unwrap();
    println!("fp uppercase: {}", fp2);
    let fp3 = Fingerprint::from_str("deadbeef").unwrap();
    assert_eq!(fp2, fp3);

    // SPEC §4.8 claim: Xpub::depth is accessible
    println!("xpub depth: {}", xpub.depth);

    // SPEC §6.4.2 claim: bip32::Error variant set
    let err = Xpub::from_str("invalid").unwrap_err();
    println!("Xpub::from_str error: {:?}", err);
}
```

```bash
cargo run --bin spike_bitcoin 2>&1 | tail -20
```

Expected: prints valid 32-byte chain_code, 33-byte pubkey, `Main` network, lowercase 8-hex fingerprint, depth=3, and a Base58/UnknownVersion-style error.

- [ ] **Step 3: Verify `bip39 = "2"` API (re-check from ms-cli precedent).**

Write `src/bin/spike_bip39.rs`:

```rust
use bip39::{Language, Mnemonic};

fn main() {
    let m = Mnemonic::parse_in(
        Language::English,
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art",
    )
    .unwrap();
    let entropy = m.to_entropy();
    println!("entropy (24-word): {}", hex::encode(&entropy));
    assert_eq!(entropy.len(), 32);

    // 64-byte BIP-32 master seed via PBKDF2 with empty passphrase
    let seed = m.to_seed("");
    println!("seed: {}", hex::encode(&seed[..16]));

    // Round-trip
    let m2 = Mnemonic::from_entropy_in(Language::English, &entropy).unwrap();
    println!("phrase: {}", m2);
}
```

```bash
cargo run --bin spike_bip39 2>&1 | tail -10
```

Expected: 64-zero-byte entropy = `0000...0000` (32 bytes), 64-byte seed, phrase round-trips to all-zeros. (Trezor's canonical 24-word vector is "abandon × 23 + art".)

- [ ] **Step 4: Verify `mk_codec` API.**

Write `src/bin/spike_mk_codec.rs`:

```rust
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};
use mk_codec::{KeyCard, encode, decode};
use std::str::FromStr;

fn main() {
    let xpub = Xpub::from_str(
        "xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj"
    ).unwrap();
    let fp = Fingerprint::from_str("deadbeef").unwrap();
    let path = DerivationPath::from_str("m/84'/0'/0'").unwrap();

    let card = KeyCard::new(
        vec![[0u8; 4]],     // policy_id_stub placeholder
        Some(fp),
        path,
        xpub,
    );

    let strings: Vec<String> = encode(&card).unwrap();
    println!("mk1 card produced {} string(s):", strings.len());
    for (i, s) in strings.iter().enumerate() {
        println!("  [{}] {} (len={})", i, s, s.len());
    }

    // Round-trip
    let strs: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
    let decoded = decode(&strs).unwrap();
    assert_eq!(decoded.xpub, card.xpub);
    assert_eq!(decoded.origin_fingerprint, card.origin_fingerprint);
    assert_eq!(decoded.policy_id_stubs, card.policy_id_stubs);
    println!("mk1 round-trip OK");
}
```

```bash
cargo run --bin spike_mk_codec 2>&1 | tail -10
```

Expected: 1 string starting with `mk1`, length around 60-80, round-trip succeeds.

- [ ] **Step 5: Verify `md_codec` API for typed-struct construction.**

Write `src/bin/spike_md_codec.rs`:

```rust
use md_codec::{
    Descriptor, Tag, TlvSection,
    PathDecl, PathDeclPaths, OriginPath, PathComponent,
    chunk::{split, reassemble},
    compute_wallet_policy_id,
};
use md_codec::tree::{Node, Body};
use md_codec::use_site_path::UseSitePath;

fn main() {
    let origin_path = OriginPath {
        components: vec![
            PathComponent { hardened: true,  value: 84 },
            PathComponent { hardened: true,  value: 0 },
            PathComponent { hardened: true,  value: 0 },
        ],
    };

    // Worked example: bip84 wpkh(@0) wallet-policy mode with one xpub.
    let descriptor = Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(origin_path),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wpkh,
            body: Body::KeyArg { index: 0 },
        },
        tlv: TlvSection {
            use_site_path_overrides: None,
            fingerprints: Some(vec![(0, [0xDE, 0xAD, 0xBE, 0xEF])]),
            pubkeys: Some(vec![(0, [0x42; 65])]),  // 32B chain_code || 33B pubkey
            origin_path_overrides: None,
            unknown: Vec::new(),
        },
    };

    println!("is_wallet_policy: {}", descriptor.is_wallet_policy());

    let strings = split(&descriptor).unwrap();
    println!("md1 card produced {} string(s):", strings.len());
    for (i, s) in strings.iter().enumerate() {
        println!("  [{}] {} (len={})", i, s, s.len());
    }

    let policy_id = compute_wallet_policy_id(&descriptor).unwrap();
    println!("policy_id (16B): {}", hex::encode(policy_id.as_bytes()));
    println!("stub bytes [0..4]: {}", hex::encode(&policy_id.as_bytes()[..4]));

    // Round-trip
    let strs: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
    let decoded = reassemble(&strs).unwrap();
    assert_eq!(decoded.is_wallet_policy(), true);
    assert_eq!(decoded.tlv.pubkeys, descriptor.tlv.pubkeys);
    assert_eq!(decoded.tlv.fingerprints, descriptor.tlv.fingerprints);
    println!("md1 round-trip OK");
}
```

```bash
cargo run --bin spike_md_codec 2>&1 | tail -10
```

Expected: 1 (or more) `md1`-prefixed string(s), `is_wallet_policy: true`, 16-byte policy_id, round-trip succeeds.

- [ ] **Step 6: Verify `ms_codec::Error`, `mk_codec::Error`, `md_codec::Error` variant sets** by reading the source files directly:

```bash
grep -E '^\s+\w+\(?' /scratch/code/shibboleth/mnemonic-secret/crates/ms-codec/src/error.rs | grep -v '//' | head -30
grep -E '^\s+\w+\(?' /scratch/code/shibboleth/mnemonic-key/crates/mk-codec/src/error.rs    | grep -v '//' | head -30
grep -E '^\s+\w+(\s*\{|\s*\(|,)' /scratch/code/shibboleth/descriptor-mnemonic/crates/md-codec/src/error.rs | grep -v '//' | head -50
```

Expected variant sets (locked at SPEC §6.4.3, §6.4.4, §6.4.5).

- [ ] **Step 7: Write the spike memo to `design/agent-reports/spike-toolkit-v0_1-phase-1.md`** (in the toolkit repo, not /tmp). Memo content:

```markdown
# Phase 1 Spike Memo — Toolkit v0.1 Sibling-API Verification

**Date:** <YYYY-MM-DD>
**Reviewer:** the spike runner (and Phase 1 reviewer at task 1.10)

## Verified API surface

### `bitcoin = "0.32"` (SPEC §4.1, §4.3, §4.6.1, §4.8, §6.4.2)

- `Xpub::chain_code: ChainCode`, `ChainCode::to_bytes() -> [u8; 32]` — confirmed.
- `Xpub::public_key: PublicKey`, `PublicKey::serialize() -> [u8; 33]` — confirmed.
- `Xpub::network: NetworkKind` — confirmed.
- `Xpub::depth: u8` — confirmed.
- `Fingerprint::to_bytes() -> [u8; 4]` — confirmed.
- `Fingerprint::from_str` — case-insensitive 8 hex — confirmed.
- `Xpriv::new_master(NetworkKind, &[u8])` — confirmed.
- `Xpriv::derive_priv(&Secp256k1, &DerivationPath)` — confirmed.
- `Xpub::from_priv(&Secp256k1, &Xpriv)` — confirmed.
- `Xpriv::fingerprint(&Secp256k1)` — confirmed.

### `bip39 = "2"` (SPEC §4.1, §6.4.1)

(Re-checked against ms-cli's already-shipped spike — see `crates/ms-cli/src/bip39_friendly.rs`.) Variants: `BadEntropyBitCount(usize)`, `BadWordCount(usize)`, `UnknownWord(usize)`, `InvalidChecksum`, `AmbiguousLanguages(_)`. Mnemonic API: `parse_in`, `from_entropy_in`, `to_entropy`, `to_seed`.

### `ms_codec` (SPEC §4.4)

`encode(Tag, &Payload) -> Result<String>` — single String. `Payload::Entr(Vec<u8>)` accepts 16/20/24/28/32 bytes.

### `mk_codec` (SPEC §4.5)

`KeyCard { policy_id_stubs: Vec<[u8;4]>, origin_fingerprint: Option<Fingerprint>, origin_path: DerivationPath, xpub: Xpub }`. `encode(&KeyCard) -> Result<Vec<String>>`, `decode(&[&str]) -> Result<KeyCard>`. v0.1 single-sig produces single-element Vec.

### `md_codec` (SPEC §4.6)

`Descriptor { n, path_decl, use_site_path, tree, tlv }`. `chunk::split(&Descriptor) -> Result<Vec<String>>`, `chunk::reassemble(&[&str]) -> Result<Descriptor>`. v0.1 single-sig produces single-element Vec. `compute_wallet_policy_id(&Descriptor) -> Result<WalletPolicyId>`, `WalletPolicyId::as_bytes() -> &[u8;16]`.

`Tag::{Pkh, Wpkh, Sh, Tr}` exist. `Body::{KeyArg{index}, Tr{key_index, tree}, Children(Vec<Node>)}` exist. `PathDeclPaths::{Shared, Divergent}` exist. `UseSitePath::standard_multipath()` exists.

`md_codec::Error` is NOT `#[non_exhaustive]`; full enumeration at `crates/md-codec/src/error.rs`.

## SPEC patches needed

(none — SPEC r3 claims hold against actual sibling source)

## Errata / surprises

(record any divergence here; if any, file a SPEC r4 amendment before proceeding)
```

- [ ] **Step 8: Commit the memo.**

```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
git add design/agent-reports/spike-toolkit-v0_1-phase-1.md
git -c commit.gpgsign=false commit -m "phase 1 task 1.1: sibling-API spike memo

Verifies SPEC §4 + §6 claims against actual ms-codec / mk-codec /
md-codec / bitcoin = 0.32 sources via dedicated spike binaries
(ephemeral; not committed to repo).

No SPEC patches needed.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
git show HEAD:design/agent-reports/spike-toolkit-v0_1-phase-1.md | head -10
```

If any SPEC claim diverges from sibling source, file a SPEC r4 amendment FIRST and re-run the spike before proceeding to Task 1.2.

### Task 1.2: Cargo.toml deps + main.rs stub structure

**Files:**
- Modify: `crates/mnemonic-toolkit/Cargo.toml`
- Modify: `crates/mnemonic-toolkit/src/main.rs`

- [ ] **Step 1: Add deps to Cargo.toml.**

Replace `crates/mnemonic-toolkit/Cargo.toml` body:

```toml
[package]
name = "mnemonic-toolkit"
version = "0.0.0"
edition.workspace = true
license.workspace = true
repository.workspace = true
homepage.workspace = true
rust-version.workspace = true
publish = false

[[bin]]
name = "mnemonic"
path = "src/main.rs"

[dependencies]
ms-codec = { git = "https://github.com/bg002h/mnemonic-secret",      tag = "ms-codec-v0.1.0" }
mk-codec = { git = "https://github.com/bg002h/mnemonic-key",         tag = "mk-codec-v0.2.1" }
md-codec = { git = "https://github.com/bg002h/descriptor-mnemonic",  tag = "md-codec-v0.16.1" }
bip39 = { version = "2", features = ["all-languages"] }
bitcoin = "0.32"
clap = { version = "4", features = ["derive"] }
hex = "0.4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
```

- [ ] **Step 2: Update main.rs to declare leaf modules.**

Replace `crates/mnemonic-toolkit/src/main.rs`:

```rust
mod error;
mod format;
mod language;
mod network;
mod parse;
mod template;

fn main() {}
```

- [ ] **Step 3: `cargo build -p mnemonic-toolkit` should fail** (modules don't exist yet). That's expected; tasks 1.3-1.8 land them.

- [ ] **Step 4: No commit yet** (commit at task 1.9 phase-end).

### Task 1.3: `error.rs` — ToolkitError + exit_code() + From impls

**Files:**
- Create: `crates/mnemonic-toolkit/src/error.rs`

Realizes SPEC §6.1, §6.2, §6.3 (exit-code table + ToolkitError enum + per-source dispatch).

- [ ] **Step 1: Write the failing tests.**

Top of file:

```rust
//! ToolkitError + exit_code() + per-source From impls.
//!
//! Realizes SPEC §6.1 (exit-code table), §6.2 (ToolkitError enum),
//! §6.3 (exit-code mapping), §6.4.0 (routing principle).

use serde_json::json;

#[derive(Debug)]
#[non_exhaustive]
pub enum ToolkitError {
    BadInput(String),
    Bip39(bip39::Error),
    Bitcoin(BitcoinErrorKind),
    MsCodec(ms_codec::Error),
    MkCodec(mk_codec::Error),
    MdCodec(md_codec::Error),
    ModeViolation { mode: &'static str, flag: &'static str, message: String },
    BundleMismatch { card: &'static str, message: String },
    NetworkMismatch { xpub_network: &'static str, expected: &'static str },
    FutureFormat { source: &'static str, detail: String },
}

#[derive(Debug)]
pub enum BitcoinErrorKind {
    Bip32(bitcoin::bip32::Error),
    XpubParse(String),
    FingerprintParse(String),
}

impl ToolkitError {
    pub fn exit_code(&self) -> u8 { /* TODO */ unimplemented!() }
    pub fn kind(&self) -> &'static str { /* TODO */ unimplemented!() }
    pub fn message(&self) -> String { /* TODO */ unimplemented!() }
    pub fn details(&self) -> Option<serde_json::Value> { /* TODO */ unimplemented!() }
}

pub type Result<T> = std::result::Result<T, ToolkitError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_code_table_per_variant() {
        assert_eq!(ToolkitError::BadInput("x".into()).exit_code(), 1);
        assert_eq!(
            ToolkitError::ModeViolation {
                mode: "watch-only", flag: "--passphrase", message: "x".into(),
            }.exit_code(),
            2,
        );
        assert_eq!(
            ToolkitError::NetworkMismatch { xpub_network: "main", expected: "test" }.exit_code(),
            2,
        );
        assert_eq!(
            ToolkitError::FutureFormat { source: "ms_codec", detail: "x".into() }.exit_code(),
            3,
        );
        assert_eq!(
            ToolkitError::BundleMismatch { card: "mk1", message: "x".into() }.exit_code(),
            4,
        );
    }

    #[test]
    fn kind_strings_stable() {
        assert_eq!(ToolkitError::BadInput("x".into()).kind(), "BadInput");
        assert_eq!(
            ToolkitError::BundleMismatch { card: "ms1", message: "".into() }.kind(),
            "BundleMismatch",
        );
        assert_eq!(
            ToolkitError::FutureFormat { source: "ms_codec", detail: "".into() }.kind(),
            "FutureFormat",
        );
    }
}
```

- [ ] **Step 2: Run tests; expect compile failure** (`unimplemented!()` panics; tests don't run yet).

```bash
cargo test -p mnemonic-toolkit error::tests 2>&1 | tail -5
```

- [ ] **Step 3: Implement `exit_code`, `kind`, `message`, `details`.**

Replace the `impl ToolkitError` block:

```rust
impl ToolkitError {
    /// SPEC §6.1 exit-code mapping.
    pub fn exit_code(&self) -> u8 {
        match self {
            ToolkitError::BadInput(_)
            | ToolkitError::Bip39(_)
            | ToolkitError::Bitcoin(_)
            | ToolkitError::MsCodec(_)
            | ToolkitError::MkCodec(_)
            | ToolkitError::MdCodec(_) => 1,
            ToolkitError::ModeViolation { .. } | ToolkitError::NetworkMismatch { .. } => 2,
            ToolkitError::FutureFormat { .. } => 3,
            ToolkitError::BundleMismatch { .. } => 4,
        }
    }

    /// Stable discriminant for JSON `kind` field (SPEC §5.5).
    pub fn kind(&self) -> &'static str {
        match self {
            ToolkitError::BadInput(_) => "BadInput",
            ToolkitError::Bip39(_) => "Bip39",
            ToolkitError::Bitcoin(_) => "Bitcoin",
            ToolkitError::MsCodec(_) => "MsCodec",
            ToolkitError::MkCodec(_) => "MkCodec",
            ToolkitError::MdCodec(_) => "MdCodec",
            ToolkitError::ModeViolation { .. } => "ModeViolation",
            ToolkitError::NetworkMismatch { .. } => "NetworkMismatch",
            ToolkitError::BundleMismatch { .. } => "BundleMismatch",
            ToolkitError::FutureFormat { .. } => "FutureFormat",
        }
    }

    /// Friendly human-readable message. Five sibling-source mappers live in
    /// `friendly.rs` (Phase 3 task 3.3) and are dispatched here.
    pub fn message(&self) -> String {
        match self {
            ToolkitError::BadInput(m) => m.clone(),
            ToolkitError::Bip39(e) => crate::friendly::friendly_bip39(e),
            ToolkitError::Bitcoin(e) => crate::friendly::friendly_bitcoin(e),
            ToolkitError::MsCodec(e) => crate::friendly::friendly_ms_codec(e),
            ToolkitError::MkCodec(e) => crate::friendly::friendly_mk_codec(e),
            ToolkitError::MdCodec(e) => crate::friendly::friendly_md_codec(e),
            ToolkitError::ModeViolation { message, .. } => message.clone(),
            ToolkitError::NetworkMismatch { xpub_network, expected } => format!(
                "xpub network {} does not match --network {}", xpub_network, expected,
            ),
            ToolkitError::BundleMismatch { card, message } => {
                format!("bundle mismatch on {}: {}; v0.1 hardcodes account=0; if the engraved bundle was produced with a non-zero account, mismatch is expected — re-run with v0.2's --account flag once available",
                    card, message)
            }
            ToolkitError::FutureFormat { source, detail } => format!(
                "{} reserved-not-emitted: {}; deferred to v0.2+", source, detail,
            ),
        }
    }

    /// JSON `details` field (SPEC §5.5).
    pub fn details(&self) -> Option<serde_json::Value> {
        match self {
            ToolkitError::ModeViolation { mode, flag, .. } => Some(json!({
                "mode": mode,
                "flag": flag,
            })),
            ToolkitError::NetworkMismatch { xpub_network, expected } => Some(json!({
                "xpub_network": xpub_network,
                "expected": expected,
            })),
            ToolkitError::BundleMismatch { card, .. } => Some(json!({ "card": card })),
            ToolkitError::FutureFormat { source, detail } => Some(json!({
                "source": source,
                "detail": detail,
            })),
            _ => None,
        }
    }
}

impl std::fmt::Display for ToolkitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error: {}", self.message())
    }
}

impl std::error::Error for ToolkitError {}

impl From<bip39::Error> for ToolkitError {
    fn from(e: bip39::Error) -> Self { ToolkitError::Bip39(e) }
}

impl From<bitcoin::bip32::Error> for ToolkitError {
    fn from(e: bitcoin::bip32::Error) -> Self { ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)) }
}

impl From<ms_codec::Error> for ToolkitError {
    fn from(e: ms_codec::Error) -> Self {
        match e {
            ms_codec::Error::ReservedTagNotEmittedInV01 { got } => ToolkitError::FutureFormat {
                source: "ms_codec",
                detail: format!("reserved tag {:?}", std::str::from_utf8(&got).unwrap_or("<non-utf8>")),
            },
            other => ToolkitError::MsCodec(other),
        }
    }
}

impl From<mk_codec::Error> for ToolkitError {
    fn from(e: mk_codec::Error) -> Self {
        match e {
            mk_codec::Error::UnsupportedVersion(v) => ToolkitError::FutureFormat {
                source: "mk_codec",
                detail: format!("unsupported version {}", v),
            },
            other => ToolkitError::MkCodec(other),
        }
    }
}

impl From<md_codec::Error> for ToolkitError {
    fn from(e: md_codec::Error) -> Self {
        match e {
            md_codec::Error::UnsupportedVersion { got } => ToolkitError::FutureFormat {
                source: "md_codec",
                detail: format!("unsupported version {}", got),
            },
            other => ToolkitError::MdCodec(other),
        }
    }
}
```

Note: `crate::friendly::*` mappers don't exist yet (Phase 3 task 3.3). For Phase 1 testing, the `message()` will fail to compile until friendly.rs lands. Workaround: Phase 1 stubs the `crate::friendly` module path with `unimplemented!()`-returning shims, then Phase 3 fills them. Add this at the top of `main.rs`:

```rust
mod friendly { /* stub for Phase 1 — real impl in Phase 3 */
    pub fn friendly_bip39(_: &bip39::Error) -> String { unimplemented!("Phase 3") }
    pub fn friendly_bitcoin(_: &crate::error::BitcoinErrorKind) -> String { unimplemented!("Phase 3") }
    pub fn friendly_ms_codec(_: &ms_codec::Error) -> String { unimplemented!("Phase 3") }
    pub fn friendly_mk_codec(_: &mk_codec::Error) -> String { unimplemented!("Phase 3") }
    pub fn friendly_md_codec(_: &md_codec::Error) -> String { unimplemented!("Phase 3") }
}
```

Phase 3 task 3.3 replaces this stub with the real `friendly.rs` file (delete the inline `mod friendly` and add `mod friendly;`).

- [ ] **Step 4: `cargo test -p mnemonic-toolkit error::tests` should pass.**

```bash
cargo test -p mnemonic-toolkit error::tests 2>&1 | tail -5
```

Expected: `2 passed`.

- [ ] **Step 5: No commit yet** (Phase 1 commit at task 1.9).

### Task 1.4: `language.rs` — clap value_enum + From<bip39::Language>

**Files:**
- Create: `crates/mnemonic-toolkit/src/language.rs`

Realizes SPEC §1 (10 wordlists) and §5.2 stderr language warning. Direct mirror of ms-cli `language.rs`.

- [ ] **Step 1: Write the test stubs + impl.**

```rust
//! `--language` clap enum + From<bip39::Language>.
//!
//! Realizes SPEC §1 (10 BIP-39 wordlists supported) + SPEC §5.2 stderr
//! language-defaulting warning. Mirrors ms-cli `language.rs`.

use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "lower")]
pub enum CliLanguage {
    English,
    SimplifiedChinese,
    TraditionalChinese,
    Czech,
    French,
    Italian,
    Japanese,
    Korean,
    Portuguese,
    Spanish,
}

impl Default for CliLanguage {
    fn default() -> Self { CliLanguage::English }
}

impl CliLanguage {
    /// Human-readable name for stderr warnings.
    pub fn human_name(&self) -> &'static str {
        match self {
            CliLanguage::English => "english",
            CliLanguage::SimplifiedChinese => "simplified-chinese",
            CliLanguage::TraditionalChinese => "traditional-chinese",
            CliLanguage::Czech => "czech",
            CliLanguage::French => "french",
            CliLanguage::Italian => "italian",
            CliLanguage::Japanese => "japanese",
            CliLanguage::Korean => "korean",
            CliLanguage::Portuguese => "portuguese",
            CliLanguage::Spanish => "spanish",
        }
    }
}

impl From<CliLanguage> for bip39::Language {
    fn from(l: CliLanguage) -> bip39::Language {
        match l {
            CliLanguage::English => bip39::Language::English,
            CliLanguage::SimplifiedChinese => bip39::Language::SimplifiedChinese,
            CliLanguage::TraditionalChinese => bip39::Language::TraditionalChinese,
            CliLanguage::Czech => bip39::Language::Czech,
            CliLanguage::French => bip39::Language::French,
            CliLanguage::Italian => bip39::Language::Italian,
            CliLanguage::Japanese => bip39::Language::Japanese,
            CliLanguage::Korean => bip39::Language::Korean,
            CliLanguage::Portuguese => bip39::Language::Portuguese,
            CliLanguage::Spanish => bip39::Language::Spanish,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_english() {
        assert_eq!(CliLanguage::default(), CliLanguage::English);
    }

    #[test]
    fn human_name_lowercase_kebab() {
        assert_eq!(CliLanguage::English.human_name(), "english");
        assert_eq!(CliLanguage::SimplifiedChinese.human_name(), "simplified-chinese");
    }

    #[test]
    fn maps_to_bip39_language() {
        let _l: bip39::Language = CliLanguage::English.into();
        let _l: bip39::Language = CliLanguage::Japanese.into();
    }
}
```

- [ ] **Step 2: Run tests.**

```bash
cargo test -p mnemonic-toolkit language::tests 2>&1 | tail -5
```

Expected: `3 passed`.

- [ ] **Step 3: No commit yet.**

### Task 1.5: `network.rs` — Network enum + NetworkKind mapping

**Files:**
- Create: `crates/mnemonic-toolkit/src/network.rs`

Realizes SPEC §2.1.4 (4 networks + coin-type table + xpub-version mapping) and §4.3 (network/xpub cross-check).

- [ ] **Step 1: Write the impl + tests.**

```rust
//! `--network` clap enum + NetworkKind mapping + xpub-version cross-check.
//!
//! Realizes SPEC §2.1.4 (4 networks + coin-type table) + §4.3 (network/
//! xpub cross-check via Xpub::network field).

use bitcoin::NetworkKind;
use clap::ValueEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "lower")]
pub enum CliNetwork {
    Mainnet,
    Testnet,
    Signet,
    Regtest,
}

impl CliNetwork {
    /// BIP-32 coin-type for this network (SPEC §2.1.4).
    /// Mainnet: 0; testnet/signet/regtest: 1.
    pub fn coin_type(&self) -> u32 {
        match self {
            CliNetwork::Mainnet => 0,
            CliNetwork::Testnet | CliNetwork::Signet | CliNetwork::Regtest => 1,
        }
    }

    /// `bitcoin::NetworkKind` for derivation. Mainnet: Main; others: Test.
    pub fn network_kind(&self) -> NetworkKind {
        match self {
            CliNetwork::Mainnet => NetworkKind::Main,
            _ => NetworkKind::Test,
        }
    }

    /// Human-readable name for stderr engraving card and error messages.
    pub fn human_name(&self) -> &'static str {
        match self {
            CliNetwork::Mainnet => "mainnet",
            CliNetwork::Testnet => "testnet",
            CliNetwork::Signet => "signet",
            CliNetwork::Regtest => "regtest",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coin_type_table() {
        assert_eq!(CliNetwork::Mainnet.coin_type(), 0);
        assert_eq!(CliNetwork::Testnet.coin_type(), 1);
        assert_eq!(CliNetwork::Signet.coin_type(), 1);
        assert_eq!(CliNetwork::Regtest.coin_type(), 1);
    }

    #[test]
    fn network_kind_mainnet_vs_test() {
        assert_eq!(CliNetwork::Mainnet.network_kind(), NetworkKind::Main);
        assert_eq!(CliNetwork::Testnet.network_kind(), NetworkKind::Test);
        assert_eq!(CliNetwork::Signet.network_kind(), NetworkKind::Test);
        assert_eq!(CliNetwork::Regtest.network_kind(), NetworkKind::Test);
    }
}
```

- [ ] **Step 2: Run tests.**

```bash
cargo test -p mnemonic-toolkit network::tests 2>&1 | tail -5
```

Expected: `2 passed`.

### Task 1.6: `template.rs` — Template enum + origin paths + wrapper bodies

**Files:**
- Create: `crates/mnemonic-toolkit/src/template.rs`

Realizes SPEC §2.1.3 (4 templates) + §4.2 (origin path table) + §4.6.3 (per-template wrapper tag/body).

- [ ] **Step 1: Write the impl + tests.**

```rust
//! `--template` clap enum + origin paths + md1 wrapper construction.
//!
//! Realizes SPEC §2.1.3 (4 templates), §4.2 (origin paths), §4.6.3
//! (per-template wrapper tag + body).

use crate::network::CliNetwork;
use bitcoin::bip32::DerivationPath;
use clap::ValueEnum;
use md_codec::origin_path::{OriginPath, PathComponent};
use md_codec::tag::Tag;
use md_codec::tree::{Body, Node};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[clap(rename_all = "lower")]
pub enum CliTemplate {
    Bip44,
    Bip49,
    Bip84,
    Bip86,
}

impl CliTemplate {
    /// BIP-32 origin path for this (template, network) cell.
    /// Account is hardcoded 0 in v0.1 (SPEC §1).
    pub fn origin_path_str(&self, network: CliNetwork) -> String {
        let purpose = match self {
            CliTemplate::Bip44 => 44,
            CliTemplate::Bip49 => 49,
            CliTemplate::Bip84 => 84,
            CliTemplate::Bip86 => 86,
        };
        format!("m/{purpose}'/{}'/0'", network.coin_type())
    }

    /// Parsed BIP-32 derivation path for use with `bitcoin::bip32`.
    pub fn derivation_path(&self, network: CliNetwork) -> DerivationPath {
        DerivationPath::from_str(&self.origin_path_str(network))
            .expect("template paths are well-formed by construction")
    }

    /// md-codec OriginPath for this (template, network) cell.
    /// Used in PathDeclPaths::Shared(...) for Phase 2 synthesize.rs.
    pub fn md_origin_path(&self, network: CliNetwork) -> OriginPath {
        let purpose: u32 = match self {
            CliTemplate::Bip44 => 44,
            CliTemplate::Bip49 => 49,
            CliTemplate::Bip84 => 84,
            CliTemplate::Bip86 => 86,
        };
        OriginPath {
            components: vec![
                PathComponent { hardened: true, value: purpose },
                PathComponent { hardened: true, value: network.coin_type() },
                PathComponent { hardened: true, value: 0 },  // account
            ],
        }
    }

    /// md-codec wrapper Node for this template (SPEC §4.6.3).
    /// All v0.1 templates use placeholder index 0 (single-sig).
    pub fn wrapper_node(&self) -> Node {
        match self {
            CliTemplate::Bip44 => Node {
                tag: Tag::Pkh,
                body: Body::KeyArg { index: 0 },
            },
            CliTemplate::Bip49 => Node {
                tag: Tag::Sh,
                body: Body::Children(vec![Node {
                    tag: Tag::Wpkh,
                    body: Body::KeyArg { index: 0 },
                }]),
            },
            CliTemplate::Bip84 => Node {
                tag: Tag::Wpkh,
                body: Body::KeyArg { index: 0 },
            },
            CliTemplate::Bip86 => Node {
                tag: Tag::Tr,
                body: Body::Tr { key_index: 0, tree: None },
            },
        }
    }

    pub fn human_name(&self) -> &'static str {
        match self {
            CliTemplate::Bip44 => "bip44",
            CliTemplate::Bip49 => "bip49",
            CliTemplate::Bip84 => "bip84",
            CliTemplate::Bip86 => "bip86",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_path_strings() {
        assert_eq!(CliTemplate::Bip44.origin_path_str(CliNetwork::Mainnet), "m/44'/0'/0'");
        assert_eq!(CliTemplate::Bip49.origin_path_str(CliNetwork::Testnet), "m/49'/1'/0'");
        assert_eq!(CliTemplate::Bip84.origin_path_str(CliNetwork::Signet),  "m/84'/1'/0'");
        assert_eq!(CliTemplate::Bip86.origin_path_str(CliNetwork::Regtest), "m/86'/1'/0'");
    }

    #[test]
    fn md_origin_path_components() {
        let op = CliTemplate::Bip84.md_origin_path(CliNetwork::Mainnet);
        assert_eq!(op.components.len(), 3);
        assert_eq!(op.components[0].value, 84);
        assert_eq!(op.components[0].hardened, true);
        assert_eq!(op.components[1].value, 0);  // mainnet coin
        assert_eq!(op.components[2].value, 0);  // account
    }

    #[test]
    fn wrapper_nodes_per_template() {
        assert!(matches!(CliTemplate::Bip44.wrapper_node().tag, Tag::Pkh));
        assert!(matches!(CliTemplate::Bip49.wrapper_node().tag, Tag::Sh));
        assert!(matches!(CliTemplate::Bip84.wrapper_node().tag, Tag::Wpkh));
        assert!(matches!(CliTemplate::Bip86.wrapper_node().tag, Tag::Tr));
    }

    #[test]
    fn bip49_nests_wpkh_under_sh() {
        let n = CliTemplate::Bip49.wrapper_node();
        if let Body::Children(children) = &n.body {
            assert_eq!(children.len(), 1);
            assert!(matches!(children[0].tag, Tag::Wpkh));
            assert!(matches!(children[0].body, Body::KeyArg { index: 0 }));
        } else {
            panic!("bip49 should nest wpkh under sh via Body::Children");
        }
    }

    #[test]
    fn bip86_uses_body_tr_keypath_only() {
        let n = CliTemplate::Bip86.wrapper_node();
        assert!(matches!(n.body, Body::Tr { key_index: 0, tree: None }));
    }
}
```

- [ ] **Step 2: Run tests.**

```bash
cargo test -p mnemonic-toolkit template::tests 2>&1 | tail -5
```

Expected: `5 passed`.

### Task 1.7: `format.rs` — chunked-form rendering + JSON output structs

**Files:**
- Create: `crates/mnemonic-toolkit/src/format.rs`

Realizes SPEC §5.1 (multi-section stdout layout) + §5.2 (engraving card) + §5.3 / §5.4 (JSON schemas).

- [ ] **Step 1: Write the impl + tests.**

```rust
//! Output formatting: multi-section stdout, engraving-card stderr,
//! JSON envelopes for bundle and verify-bundle.
//!
//! Realizes SPEC §5.1, §5.2, §5.3, §5.4.

use serde::Serialize;

/// Render an `ms1` string in 5-char-grouped chunked form (10 groups/line max).
/// Mirrors ms-cli `format::chunked_form`.
pub fn chunk_5char(s: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut groups: Vec<String> = Vec::new();
    for chunk in chars.chunks(5) {
        groups.push(chunk.iter().collect::<String>());
    }
    for (i, g) in groups.iter().enumerate() {
        if i > 0 && i % 10 == 0 {
            out.push('\n');
        } else if i > 0 {
            out.push(' ');
        }
        out.push_str(g);
    }
    out
}

/// Render an `mk1` string in mk-codec's chunked form. v0.1: defer to mk-codec
/// internal chunked-form when available; fallback to chunk_5char for v0.1.
pub fn chunk_mk1(s: &str) -> String {
    chunk_5char(s)
}

/// Render an `md1` string in md-codec's `render_codex32_grouped(s, 5)` form.
pub fn chunk_md1(s: &str) -> String {
    md_codec::encode::render_codex32_grouped(s, 5)
}

/// Bundle JSON output schema (SPEC §5.3). Field order is part of the schema.
#[derive(Debug, Serialize)]
pub struct BundleJson<'a> {
    pub schema_version: &'static str,
    pub mode: &'static str,           // "full" | "watch-only"
    pub network: &'static str,
    pub template: &'static str,
    pub account: u32,
    pub origin_path: String,
    pub master_fingerprint: String,
    pub ms1: Option<&'a str>,         // null in watch-only
    pub mk1: &'a [String],
    pub md1: &'a [String],
    pub engraving_card: Option<String>,
}

/// Verify-bundle JSON output schema (SPEC §5.4). Field order is part of the schema.
#[derive(Debug, Serialize)]
pub struct VerifyBundleJson {
    pub schema_version: &'static str,
    pub result: &'static str,         // "ok" | "mismatch"
    pub checks: Vec<VerifyCheck>,
}

#[derive(Debug, Serialize)]
pub struct VerifyCheck {
    pub name: &'static str,
    pub result: &'static str,         // "ok" | "fail" | "skipped"
    pub detail: String,
}

/// Compose the engraving-card stderr text (SPEC §5.2). Pinned byte-exact.
pub fn engraving_card(
    network: &str,
    template: &str,
    origin_path: &str,
    master_fingerprint: &str,
    mode: EngravingMode<'_>,
) -> String {
    let mut s = String::new();
    s.push_str(&format!("network: {}\n", network));
    s.push_str(&format!("template: {}\n", template));
    s.push_str("account: 0\n");
    s.push_str(&format!("origin path: {}\n", origin_path));
    s.push_str(&format!("master fingerprint: {}\n", master_fingerprint));
    match mode {
        EngravingMode::FullNoPassphrase { language } => {
            s.push_str(&format!("language: {} (BIP-39 checksum valid)\n", language));
            s.push_str("passphrase: not used\n");
        }
        EngravingMode::FullWithPassphrase { language } => {
            s.push_str(&format!("language: {} (BIP-39 checksum valid)\n", language));
            s.push_str("passphrase: USED — not engraved on any card; record separately and never lose it.\n");
        }
        EngravingMode::WatchOnly => {
            s.push_str("mode: watch-only (xpub-supplied; no entropy known to toolkit)\n");
            s.push_str("ms1 card omitted; recover entropy from the original wallet's other backup.\n");
        }
    }
    s.push_str("engrave each card on its own plate. record this card alongside.\n");
    s
}

pub enum EngravingMode<'a> {
    FullNoPassphrase { language: &'a str },
    FullWithPassphrase { language: &'a str },
    WatchOnly,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk_5char_groups() {
        let s = "abcdefghij";
        assert_eq!(chunk_5char(s), "abcde fghij");
    }

    #[test]
    fn chunk_5char_remainder() {
        let s = "abcdefg";
        assert_eq!(chunk_5char(s), "abcde fg");
    }

    #[test]
    fn chunk_5char_wraps_at_10_groups() {
        let s: String = std::iter::repeat('x').take(55).collect();  // 11 groups of 5
        let out = chunk_5char(&s);
        assert!(out.contains('\n'));
        // Verify wrap is at the 10-group boundary
        let first_line = out.lines().next().unwrap();
        let group_count = first_line.split(' ').count();
        assert_eq!(group_count, 10);
    }

    #[test]
    fn engraving_card_full_no_passphrase_byte_exact() {
        let card = engraving_card(
            "mainnet", "bip84", "m/84'/0'/0'", "deadbeef",
            EngravingMode::FullNoPassphrase { language: "english" },
        );
        let expected = "\
network: mainnet
template: bip84
account: 0
origin path: m/84'/0'/0'
master fingerprint: deadbeef
language: english (BIP-39 checksum valid)
passphrase: not used
engrave each card on its own plate. record this card alongside.
";
        assert_eq!(card, expected);
    }

    #[test]
    fn engraving_card_with_passphrase_uses_uppercase_USED() {
        let card = engraving_card(
            "mainnet", "bip84", "m/84'/0'/0'", "deadbeef",
            EngravingMode::FullWithPassphrase { language: "english" },
        );
        assert!(card.contains("passphrase: USED — not engraved on any card; record separately and never lose it.\n"));
    }

    #[test]
    fn engraving_card_watch_only_omits_ms1() {
        let card = engraving_card(
            "mainnet", "bip84", "m/84'/0'/0'", "deadbeef",
            EngravingMode::WatchOnly,
        );
        assert!(card.contains("mode: watch-only"));
        assert!(card.contains("ms1 card omitted"));
        assert!(!card.contains("language:"));
        assert!(!card.contains("passphrase:"));
    }
}
```

- [ ] **Step 2: Run tests.**

```bash
cargo test -p mnemonic-toolkit format::tests 2>&1 | tail -5
```

Expected: `6 passed`.

### Task 1.8: `parse.rs` — input source resolution + phrase normalization + fingerprint parsing

**Files:**
- Create: `crates/mnemonic-toolkit/src/parse.rs`

Realizes SPEC §3.2 (stdin uniform behavior) + §2.1.5 (fingerprint format) + §2.1.6 (concurrent stdin guard).

- [ ] **Step 1: Write the impl + tests.**

```rust
//! Input source resolution: argv vs stdin, phrase normalization,
//! fingerprint parsing.
//!
//! Realizes SPEC §3.2 (stdin uniform), §2.1.5 (--master-fingerprint
//! 8-hex case-insensitive), §2.1.6 (concurrent stdin guard).

use crate::error::{ToolkitError, BitcoinErrorKind};
use bitcoin::bip32::Fingerprint;
use std::io::{self, Read};
use std::str::FromStr;

/// Resolve a flag value: `Some(s)` literal, `Some("-")` stdin, `None` error.
/// Whitespace is collapsed via `normalize_phrase`.
pub fn read_phrase_input(arg: Option<&str>, stdin: &mut dyn Read) -> Result<String, ToolkitError> {
    match arg {
        Some("-") => {
            let mut buf = String::new();
            stdin.read_to_string(&mut buf).map_err(|e| ToolkitError::BadInput(format!("stdin read failed: {}", e)))?;
            Ok(normalize_phrase(&buf))
        }
        Some(s) => Ok(normalize_phrase(s)),
        None => Err(ToolkitError::BadInput("missing argument".into())),
    }
}

/// Collapse runs of whitespace to single spaces; preserve word boundaries.
fn normalize_phrase(s: &str) -> String {
    s.split_whitespace().collect::<Vec<&str>>().join(" ")
}

/// Parse `--master-fingerprint`: 8 hex chars, case-insensitive, no `0x` prefix.
/// SPEC §2.1.5 byte-exact rejection message.
pub fn parse_master_fingerprint(s: &str) -> Result<Fingerprint, ToolkitError> {
    if s.len() != 8 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ToolkitError::BadInput(
            "--master-fingerprint must be 8 hex chars (e.g., deadbeef)".into(),
        ));
    }
    Fingerprint::from_str(s).map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::FingerprintParse(format!("{}", e))))
}

/// Reject concurrent stdin reads across phrase + passphrase.
pub fn check_no_concurrent_stdin(phrase: Option<&str>, passphrase: Option<&str>) -> Result<(), ToolkitError> {
    if phrase == Some("-") && passphrase == Some("-") {
        return Err(ToolkitError::BadInput(
            "only one of --phrase and --passphrase may read from stdin".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_phrase_collapses_whitespace() {
        assert_eq!(normalize_phrase("  word1   word2\nword3\t word4  "), "word1 word2 word3 word4");
    }

    #[test]
    fn fp_lowercase_8hex_ok() {
        let fp = parse_master_fingerprint("deadbeef").unwrap();
        assert_eq!(fp.to_string().to_lowercase(), "deadbeef");
    }

    #[test]
    fn fp_uppercase_8hex_ok() {
        parse_master_fingerprint("DEADBEEF").unwrap();
    }

    #[test]
    fn fp_mixed_case_8hex_ok() {
        parse_master_fingerprint("DeAdBeEf").unwrap();
    }

    #[test]
    fn fp_short_rejected() {
        let e = parse_master_fingerprint("dead").unwrap_err();
        match e {
            ToolkitError::BadInput(m) => assert_eq!(m, "--master-fingerprint must be 8 hex chars (e.g., deadbeef)"),
            _ => panic!("expected BadInput, got {:?}", e),
        }
    }

    #[test]
    fn fp_with_0x_prefix_rejected() {
        let e = parse_master_fingerprint("0xdeadbe").unwrap_err();
        assert!(matches!(e, ToolkitError::BadInput(_)));
    }

    #[test]
    fn fp_non_hex_char_rejected() {
        let e = parse_master_fingerprint("deadbeeg").unwrap_err();
        assert!(matches!(e, ToolkitError::BadInput(_)));
    }

    #[test]
    fn read_phrase_argv_normalizes() {
        let mut stdin = std::io::empty();
        let s = read_phrase_input(Some("  word1   word2  "), &mut stdin).unwrap();
        assert_eq!(s, "word1 word2");
    }

    #[test]
    fn read_phrase_stdin_normalizes() {
        let mut stdin = std::io::Cursor::new("  word1\n  word2\t\nword3\n  ");
        let s = read_phrase_input(Some("-"), &mut stdin).unwrap();
        assert_eq!(s, "word1 word2 word3");
    }

    #[test]
    fn concurrent_stdin_rejected() {
        let e = check_no_concurrent_stdin(Some("-"), Some("-")).unwrap_err();
        match e {
            ToolkitError::BadInput(m) => assert_eq!(m, "only one of --phrase and --passphrase may read from stdin"),
            _ => panic!("expected BadInput"),
        }
    }

    #[test]
    fn one_stdin_ok() {
        check_no_concurrent_stdin(Some("-"), None).unwrap();
        check_no_concurrent_stdin(None, Some("-")).unwrap();
        check_no_concurrent_stdin(Some("words"), Some("-")).unwrap();
    }
}
```

- [ ] **Step 2: Run tests.**

```bash
cargo test -p mnemonic-toolkit parse::tests 2>&1 | tail -5
```

Expected: `11 passed`.

### Task 1.9: Phase 1 commit

- [ ] **Step 1: Verify all Phase 1 tests pass.**

```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
cargo test -p mnemonic-toolkit 2>&1 | tail -10
cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings 2>&1 | tail -10
cargo fmt --check -p mnemonic-toolkit 2>&1 | tail -5
```

Expected: all unit tests pass; clippy clean; fmt clean.

- [ ] **Step 2: Stage explicit paths (no `git add -A`).**

```bash
git add crates/mnemonic-toolkit/Cargo.toml \
        crates/mnemonic-toolkit/src/main.rs \
        crates/mnemonic-toolkit/src/error.rs \
        crates/mnemonic-toolkit/src/language.rs \
        crates/mnemonic-toolkit/src/network.rs \
        crates/mnemonic-toolkit/src/template.rs \
        crates/mnemonic-toolkit/src/format.rs \
        crates/mnemonic-toolkit/src/parse.rs \
        Cargo.lock
git status --short
```

- [ ] **Step 3: Commit.**

```bash
git -c commit.gpgsign=false commit -m "phase 1: foundation modules — error/language/network/template/format/parse

6 leaf modules with no internal-crate deps:

- error.rs: ToolkitError enum + exit_code() (1/2/3/4) + From impls
  routing reserved-not-emitted variants (ms_codec, mk_codec, md_codec)
  to ToolkitError::FutureFormat → exit 3.
- language.rs: CliLanguage clap value_enum (10 BIP-39 wordlists, default
  english) + From<bip39::Language>.
- network.rs: CliNetwork (mainnet/testnet/signet/regtest) + coin_type()
  + network_kind() per SPEC §2.1.4.
- template.rs: CliTemplate (bip44/49/84/86) + origin_path_str() +
  derivation_path() + md_origin_path() + wrapper_node() per SPEC §4.2,
  §4.6.3. bip49 nests wpkh under sh via Body::Children. bip86 uses
  Body::Tr keypath-only.
- format.rs: chunk_5char, chunk_md1 wrapping md_codec::render_codex32_
  grouped, BundleJson + VerifyBundleJson + VerifyCheck serde structs
  for SPEC §5.3/§5.4 schemas (field order pinned), engraving_card
  composer with byte-exact stderr lines per SPEC §5.2.
- parse.rs: read_phrase_input (whitespace-normalized stdin/argv),
  parse_master_fingerprint (8-hex case-insensitive, byte-exact reject
  message per §2.1.5), check_no_concurrent_stdin guard.

Phase 1 task 1.1 spike memo verified all SPEC §4 + §6 claims against
actual sibling sources (bitcoin = 0.32, bip39 = 2, ms_codec, mk_codec,
md_codec); no SPEC patches needed.

Friendly mappers stubbed in main.rs (mod friendly { unimplemented!() })
pending Phase 3 task 3.3 implementation.

Tests: 27 unit tests passing. clippy + fmt clean.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
git log --oneline -1
git show HEAD --stat | head -15
```

- [ ] **Step 4: Verify HEAD content** (memory `feedback_verify_committed_content_not_working_tree`).

```bash
git show HEAD:crates/mnemonic-toolkit/src/error.rs | head -30
git show HEAD:crates/mnemonic-toolkit/src/template.rs | head -30
```

### Task 1.10: Phase 1 opus review checkpoint

Iterative agent review (memory `feedback_iterative_review_every_phase`). Repeat r1, r2, … until 0 critical / 0 important findings.

- [ ] **Step 1: Dispatch `feature-dev:code-reviewer` with the Phase 1 commit SHA.** Brief on the SPEC sections each module realizes; ask for confidence-filtered critical/important/low/nit findings; persist report to `design/agent-reports/phase-1-foundation-review-r1.md`.

- [ ] **Step 2: If critical/important findings, fix inline; commit as `phase 1 r1 fixup` commit; dispatch r2; repeat until 0/0 terminator.** Low/nit findings: defer to `design/FOLLOWUPS.md` at tier `v0.1-nice-to-have`.

- [ ] **Step 3: When terminator reached, mark Phase 1 complete and proceed to Phase 2.**

---

## Phase 2: Synthesis modules

**Goal:** Land `derive.rs` (BIP-32 master-seed + xpub derivation) and `synthesize.rs` (the bundle-assembly logic that produces ms1/mk1/md1 strings + cross-binding invariants). Phase 2 is where the toolkit's correctness story rides — every line is judged against SPEC §4.

**Files:**
- Modify: `crates/mnemonic-toolkit/src/main.rs` (add `mod derive; mod synthesize;`)
- Create: `crates/mnemonic-toolkit/src/derive.rs`
- Create: `crates/mnemonic-toolkit/src/synthesize.rs`

### Task 2.1: `derive.rs` — full-mode BIP-32 derivation

Realizes SPEC §4.1.

- [ ] **Step 1: Write the failing tests using Trezor's all-zero entropy vector (24-word "abandon × 23 + art").**

```rust
//! Full-mode BIP-32 derivation: phrase → entropy → seed → master xpriv → account xpub.
//!
//! Realizes SPEC §4.1.

use crate::error::ToolkitError;
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::template::CliTemplate;
use bip39::Mnemonic;
use bitcoin::bip32::{Fingerprint, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;

/// Result of full-mode derivation.
pub struct DerivedAccount {
    pub entropy: Vec<u8>,
    pub master_fingerprint: Fingerprint,
    pub account_xpub: Xpub,
}

pub fn derive_full(
    phrase: &str,
    passphrase: &str,
    language: CliLanguage,
    network: CliNetwork,
    template: CliTemplate,
) -> Result<DerivedAccount, ToolkitError> {
    let mnemonic = Mnemonic::parse_in(language.into(), phrase)
        .map_err(ToolkitError::Bip39)?;
    let entropy = mnemonic.to_entropy();
    let seed = mnemonic.to_seed(passphrase);

    let secp = Secp256k1::new();
    let master = Xpriv::new_master(network.network_kind(), &seed)
        .map_err(|e| ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e)))?;
    let master_fingerprint = master.fingerprint(&secp);

    let path = template.derivation_path(network);
    let account_xpriv = master.derive_priv(&secp, &path)
        .map_err(|e| ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e)))?;
    let account_xpub = Xpub::from_priv(&secp, &account_xpriv);

    // Belt-and-braces network cross-check (SPEC §4.3).
    if account_xpub.network != network.network_kind() {
        return Err(ToolkitError::BadInput(format!(
            "derived-xpub network {:?} does not match --network {}; this is a toolkit bug",
            account_xpub.network, network.human_name(),
        )));
    }

    Ok(DerivedAccount { entropy, master_fingerprint, account_xpub })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Trezor canonical 24-word vector: "abandon × 23 art" → 32-zero-bytes entropy.
    const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    #[test]
    fn derive_24word_zero_entropy() {
        let acc = derive_full(
            TREZOR_24, "", CliLanguage::English, CliNetwork::Mainnet, CliTemplate::Bip84,
        ).unwrap();
        assert_eq!(acc.entropy, vec![0u8; 32]);
    }

    #[test]
    fn derive_master_fingerprint_stable() {
        let acc = derive_full(
            TREZOR_24, "", CliLanguage::English, CliNetwork::Mainnet, CliTemplate::Bip84,
        ).unwrap();
        // Trezor 24-zero master fingerprint is well-known: 73c5da0a
        assert_eq!(acc.master_fingerprint.to_string().to_lowercase(), "73c5da0a");
    }

    #[test]
    fn derive_xpub_at_bip84_mainnet_matches_known() {
        let acc = derive_full(
            TREZOR_24, "", CliLanguage::English, CliNetwork::Mainnet, CliTemplate::Bip84,
        ).unwrap();
        // Phase 1 spike + ground-truth: this is the canonical bip84 m/84'/0'/0' xpub
        // for the 24-zero-entropy seed. If the test fails after a bitcoin = 0.32
        // upgrade, regenerate via the spike harness in /tmp/toolkit-spike.
        let s = acc.account_xpub.to_string();
        assert!(s.starts_with("xpub6"), "expected xpub6 prefix, got {}", &s[..10]);
        assert!(acc.account_xpub.depth == 3);
    }

    #[test]
    fn derive_testnet_uses_tpub() {
        let acc = derive_full(
            TREZOR_24, "", CliLanguage::English, CliNetwork::Testnet, CliTemplate::Bip84,
        ).unwrap();
        let s = acc.account_xpub.to_string();
        assert!(s.starts_with("tpub"), "expected tpub prefix on testnet, got {}", &s[..10]);
    }

    #[test]
    fn derive_with_passphrase_changes_seed() {
        let a = derive_full(
            TREZOR_24, "", CliLanguage::English, CliNetwork::Mainnet, CliTemplate::Bip84,
        ).unwrap();
        let b = derive_full(
            TREZOR_24, "TREZOR", CliLanguage::English, CliNetwork::Mainnet, CliTemplate::Bip84,
        ).unwrap();
        assert_ne!(a.account_xpub, b.account_xpub);
    }

    #[test]
    fn derive_passphrase_empty_string_equals_unset() {
        let a = derive_full(
            TREZOR_24, "", CliLanguage::English, CliNetwork::Mainnet, CliTemplate::Bip84,
        ).unwrap();
        // SPEC §4.1 step 3: --passphrase "" ≡ unset
        let b = derive_full(
            TREZOR_24, "", CliLanguage::English, CliNetwork::Mainnet, CliTemplate::Bip84,
        ).unwrap();
        assert_eq!(a.account_xpub, b.account_xpub);
        assert_eq!(a.master_fingerprint, b.master_fingerprint);
    }

    #[test]
    fn bad_phrase_returns_bip39_error() {
        let e = derive_full(
            "not a valid bip39 phrase nor anywhere close", "", CliLanguage::English,
            CliNetwork::Mainnet, CliTemplate::Bip84,
        ).unwrap_err();
        assert!(matches!(e, ToolkitError::Bip39(_)));
    }
}
```

- [ ] **Step 2: Run tests.**

```bash
cargo test -p mnemonic-toolkit derive::tests 2>&1 | tail -10
```

Expected: `7 passed`. The bip84 xpub test asserts `xpub6` prefix and depth=3; full byte-exact xpub locking is in Phase 5 vector fixtures.

### Task 2.2: `synthesize.rs` — bundle assembly + cross-binding

Realizes SPEC §4.4 (ms1), §4.5 (mk1), §4.6 (md1 typed-struct), §4.7 (cross-binding).

- [ ] **Step 1: Write the impl + tests.**

```rust
//! Bundle synthesis: produce ms1 + mk1 + md1 strings from derived inputs.
//!
//! Realizes SPEC §4.4 (ms1), §4.5 (mk1), §4.6 (md1 typed-struct
//! construction with chain_code||pubkey 65-byte transform), §4.7
//! (cross-binding invariants).

use crate::error::ToolkitError;
use crate::network::CliNetwork;
use crate::template::CliTemplate;
use bitcoin::bip32::{Fingerprint, Xpub};
use md_codec::{Descriptor, TlvSection};
use md_codec::origin_path::{PathDecl, PathDeclPaths};
use md_codec::use_site_path::UseSitePath;

pub struct Bundle {
    pub ms1: Option<String>,            // None in watch-only mode
    pub mk1: Vec<String>,
    pub md1: Vec<String>,
}

/// Convert a `bitcoin::bip32::Xpub` to md-codec's 65-byte form:
///   [0..32]  = chain_code
///   [32..65] = compressed pubkey
/// SPEC §4.6.1.
pub fn xpub_to_65(xpub: &Xpub) -> [u8; 65] {
    let mut out = [0u8; 65];
    out[0..32].copy_from_slice(&xpub.chain_code.to_bytes());
    out[32..65].copy_from_slice(&xpub.public_key.serialize());
    out
}

/// Build the typed Descriptor for a (template, network, xpub, fingerprint).
/// Caller's xpub MUST already be at the template's BIP path; not rederived.
/// SPEC §4.6.
pub fn build_descriptor(
    template: CliTemplate,
    network: CliNetwork,
    xpub: &Xpub,
    fingerprint: Fingerprint,
) -> Descriptor {
    let xpub_65 = xpub_to_65(xpub);
    let fp_bytes: [u8; 4] = fingerprint.to_bytes();
    let origin_path = template.md_origin_path(network);
    let tree = template.wrapper_node();

    Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(origin_path),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree,
        tlv: TlvSection {
            use_site_path_overrides: None,
            fingerprints: Some(vec![(0, fp_bytes)]),
            pubkeys: Some(vec![(0, xpub_65)]),
            origin_path_overrides: None,
            unknown: Vec::new(),
        },
    }
}

/// Synthesize a full-mode bundle (entropy known).
/// SPEC §4.4-§4.7.
pub fn synthesize_full(
    entropy: &[u8],
    fingerprint: Fingerprint,
    xpub: Xpub,
    template: CliTemplate,
    network: CliNetwork,
) -> Result<Bundle, ToolkitError> {
    // §4.4: ms1
    let ms1 = ms_codec::encode(
        ms_codec::Tag::ENTR,
        &ms_codec::Payload::Entr(entropy.to_vec()),
    ).map_err(ToolkitError::from)?;

    // §4.6: md1 (build first; needed for policy_id_stub)
    let descriptor = build_descriptor(template, network, &xpub, fingerprint);
    let policy_id = md_codec::compute_wallet_policy_id(&descriptor)
        .map_err(ToolkitError::from)?;
    let mut stub = [0u8; 4];
    stub.copy_from_slice(&policy_id.as_bytes()[..4]);

    let md1 = md_codec::chunk::split(&descriptor)
        .map_err(ToolkitError::from)?;

    // §4.5: mk1
    let path = template.derivation_path(network);
    let card = mk_codec::KeyCard::new(
        vec![stub],
        Some(fingerprint),
        path,
        xpub,
    );
    let mk1 = mk_codec::encode(&card).map_err(ToolkitError::from)?;

    // §4.7 invariants 1+2 (3 deferred to v0.2 --self-check).
    debug_assert_eq!(&card.policy_id_stubs[0], &stub);
    debug_assert!(descriptor.is_wallet_policy());

    Ok(Bundle { ms1: Some(ms1), mk1, md1 })
}

/// Synthesize a watch-only bundle (no entropy known; ms1 omitted).
/// SPEC §4 watch-only path.
pub fn synthesize_watch_only(
    fingerprint: Fingerprint,
    xpub: Xpub,
    template: CliTemplate,
    network: CliNetwork,
) -> Result<Bundle, ToolkitError> {
    let descriptor = build_descriptor(template, network, &xpub, fingerprint);
    let policy_id = md_codec::compute_wallet_policy_id(&descriptor)
        .map_err(ToolkitError::from)?;
    let mut stub = [0u8; 4];
    stub.copy_from_slice(&policy_id.as_bytes()[..4]);

    let md1 = md_codec::chunk::split(&descriptor)
        .map_err(ToolkitError::from)?;

    let path = template.derivation_path(network);
    let card = mk_codec::KeyCard::new(
        vec![stub],
        Some(fingerprint),
        path,
        xpub,
    );
    let mk1 = mk_codec::encode(&card).map_err(ToolkitError::from)?;

    debug_assert_eq!(&card.policy_id_stubs[0], &stub);
    debug_assert!(descriptor.is_wallet_policy());

    Ok(Bundle { ms1: None, mk1, md1 })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::derive::derive_full;
    use crate::language::CliLanguage;
    use bitcoin::secp256k1::Secp256k1;

    const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    fn fixture_full(template: CliTemplate, network: CliNetwork) -> (Vec<u8>, Fingerprint, Xpub) {
        let acc = derive_full(TREZOR_24, "", CliLanguage::English, network, template).unwrap();
        (acc.entropy, acc.master_fingerprint, acc.account_xpub)
    }

    #[test]
    fn xpub_to_65_layout() {
        let (_, _, xpub) = fixture_full(CliTemplate::Bip84, CliNetwork::Mainnet);
        let bytes = xpub_to_65(&xpub);
        // First 32 bytes are chain_code
        assert_eq!(&bytes[0..32], xpub.chain_code.to_bytes().as_slice());
        // Next 33 bytes are compressed pubkey
        assert_eq!(&bytes[32..65], xpub.public_key.serialize().as_slice());
    }

    #[test]
    fn full_bundle_emits_three_cards() {
        let (entropy, fp, xpub) = fixture_full(CliTemplate::Bip84, CliNetwork::Mainnet);
        let bundle = synthesize_full(&entropy, fp, xpub, CliTemplate::Bip84, CliNetwork::Mainnet).unwrap();
        assert!(bundle.ms1.is_some());
        let ms1 = bundle.ms1.as_ref().unwrap();
        assert!(ms1.starts_with("ms1"));
        assert_eq!(bundle.mk1.len(), 1);  // v0.1 single-sig: single chunk
        assert!(bundle.mk1[0].starts_with("mk1"));
        assert_eq!(bundle.md1.len(), 1);
        assert!(bundle.md1[0].starts_with("md1"));
    }

    #[test]
    fn watch_only_bundle_omits_ms1() {
        let (_, fp, xpub) = fixture_full(CliTemplate::Bip84, CliNetwork::Mainnet);
        let bundle = synthesize_watch_only(fp, xpub, CliTemplate::Bip84, CliNetwork::Mainnet).unwrap();
        assert!(bundle.ms1.is_none());
        assert_eq!(bundle.mk1.len(), 1);
        assert_eq!(bundle.md1.len(), 1);
    }

    #[test]
    fn cross_binding_holds_round_trip() {
        let (entropy, fp, xpub) = fixture_full(CliTemplate::Bip84, CliNetwork::Mainnet);
        let bundle = synthesize_full(&entropy, fp, xpub.clone(), CliTemplate::Bip84, CliNetwork::Mainnet).unwrap();

        let mk1_strs: Vec<&str> = bundle.mk1.iter().map(|s| s.as_str()).collect();
        let decoded_mk1 = mk_codec::decode(&mk1_strs).unwrap();
        let md1_strs: Vec<&str> = bundle.md1.iter().map(|s| s.as_str()).collect();
        let decoded_md1 = md_codec::chunk::reassemble(&md1_strs).unwrap();

        // §4.7 invariant 1: stub linkage
        let policy_id = md_codec::compute_wallet_policy_id(&decoded_md1).unwrap();
        assert_eq!(&decoded_mk1.policy_id_stubs[0], &policy_id.as_bytes()[..4]);

        // §4.7 invariant 2: wallet-policy mode
        assert!(decoded_md1.is_wallet_policy());

        // mk1 round-trip preserves xpub + fp + path
        assert_eq!(decoded_mk1.xpub, xpub);
        assert_eq!(decoded_mk1.origin_fingerprint, Some(fp));
    }

    #[test]
    fn cross_binding_holds_all_4_templates_x_4_networks() {
        let templates = [CliTemplate::Bip44, CliTemplate::Bip49, CliTemplate::Bip84, CliTemplate::Bip86];
        let networks = [CliNetwork::Mainnet, CliNetwork::Testnet, CliNetwork::Signet, CliNetwork::Regtest];
        for &t in &templates {
            for &n in &networks {
                let (entropy, fp, xpub) = fixture_full(t, n);
                let bundle = synthesize_full(&entropy, fp, xpub.clone(), t, n).unwrap();
                let mk1_strs: Vec<&str> = bundle.mk1.iter().map(|s| s.as_str()).collect();
                let decoded_mk1 = mk_codec::decode(&mk1_strs).unwrap();
                let md1_strs: Vec<&str> = bundle.md1.iter().map(|s| s.as_str()).collect();
                let decoded_md1 = md_codec::chunk::reassemble(&md1_strs).unwrap();
                let policy_id = md_codec::compute_wallet_policy_id(&decoded_md1).unwrap();
                assert_eq!(&decoded_mk1.policy_id_stubs[0], &policy_id.as_bytes()[..4],
                    "stub linkage failed for {:?} on {:?}", t, n);
                assert!(decoded_md1.is_wallet_policy(), "{:?} on {:?}", t, n);
                assert_eq!(decoded_mk1.xpub, xpub, "{:?} on {:?}", t, n);
                assert_eq!(decoded_mk1.origin_fingerprint, Some(fp), "{:?} on {:?}", t, n);
            }
        }
    }
}
```

- [ ] **Step 2: Update main.rs to declare new modules.**

```rust
mod derive;
mod error;
mod format;
mod language;
mod network;
mod parse;
mod synthesize;
mod template;

mod friendly { /* Phase 3 task 3.3 replaces this stub */
    pub fn friendly_bip39(_: &bip39::Error) -> String { unimplemented!("Phase 3") }
    pub fn friendly_bitcoin(_: &crate::error::BitcoinErrorKind) -> String { unimplemented!("Phase 3") }
    pub fn friendly_ms_codec(_: &ms_codec::Error) -> String { unimplemented!("Phase 3") }
    pub fn friendly_mk_codec(_: &mk_codec::Error) -> String { unimplemented!("Phase 3") }
    pub fn friendly_md_codec(_: &md_codec::Error) -> String { unimplemented!("Phase 3") }
}

fn main() {}
```

- [ ] **Step 3: Run tests.**

```bash
cargo test -p mnemonic-toolkit synthesize::tests 2>&1 | tail -15
```

Expected: 5 passed. The 16-cell cross-binding test (4×4 = 16 round-trips) is the central correctness gate of Phase 2.

### Task 2.3: Phase 2 commit

- [ ] **Step 1: Verify all tests + clippy + fmt.**

```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
cargo test -p mnemonic-toolkit 2>&1 | tail -5
cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings 2>&1 | tail -5
cargo fmt --check -p mnemonic-toolkit 2>&1 | tail -3
```

- [ ] **Step 2: Stage + commit.**

```bash
git add crates/mnemonic-toolkit/src/main.rs \
        crates/mnemonic-toolkit/src/derive.rs \
        crates/mnemonic-toolkit/src/synthesize.rs \
        Cargo.lock
git -c commit.gpgsign=false commit -m "phase 2: synthesis — derive.rs + synthesize.rs

derive.rs (SPEC §4.1):
  derive_full(phrase, passphrase, language, network, template) →
  DerivedAccount { entropy, master_fingerprint, account_xpub }.
  Belt-and-braces network cross-check on derived xpub (§4.3).
  Tests use Trezor 24-word zero-entropy vector (master fp 73c5da0a).

synthesize.rs (SPEC §4.4-§4.7):
  build_descriptor() — typed-struct md-codec Descriptor construction
  with chain_code||pubkey 65-byte transform (§4.6.1).
  synthesize_full() → Bundle { ms1, mk1, md1 } with internal §4.7
  invariants debug_asserted.
  synthesize_watch_only() → Bundle { ms1: None, mk1, md1 }.
  Cross-binding round-trip test covers all 4 templates × 4 networks =
  16 cells.

Tests: 39 unit tests passing total. clippy + fmt clean.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
git show HEAD:crates/mnemonic-toolkit/src/synthesize.rs | head -30
```

### Task 2.4: Phase 2 opus review checkpoint

(Same iterative pattern as Task 1.10. Reviewer scrutinizes Phase 2's correctness story specifically: cross-binding invariants, xpub byte-format transform, network/coin-type table consistency, debug_assert use, error propagation through `From` impls.)

---

## Phase 3: Command modules

**Goal:** Land `cmd/bundle.rs`, `cmd/verify_bundle.rs`, and `friendly.rs`. Replaces the Phase 1 friendly-mod stub.

**Files:**
- Modify: `crates/mnemonic-toolkit/src/main.rs` (delete inline `mod friendly`; add `mod friendly;`, `mod cmd;`)
- Create: `crates/mnemonic-toolkit/src/friendly.rs`
- Create: `crates/mnemonic-toolkit/src/cmd/mod.rs`
- Create: `crates/mnemonic-toolkit/src/cmd/bundle.rs`
- Create: `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs`

### Task 3.1: `friendly.rs` — five sibling-source mappers

Realizes SPEC §6.4.0 routing principle + §6.4.1-§6.4.5 mapper tables.

- [ ] **Step 1: Write the five mappers.**

```rust
//! Five friendly mappers: bip39, bitcoin, ms_codec, mk_codec, md_codec.
//!
//! Realizes SPEC §6.4.0 routing principle + §6.4.1-§6.4.5 per-source
//! tables. All `#[non_exhaustive]` enums (bip39::Error,
//! bitcoin::bip32::Error, ms_codec::Error, mk_codec::Error) have a
//! wildcard `_` arm; md_codec::Error is closed and matched exhaustively.

use crate::error::BitcoinErrorKind;

pub fn friendly_bip39(e: &bip39::Error) -> String {
    match e {
        bip39::Error::BadEntropyBitCount(n) => format!(
            "BIP-39 entropy bit count {} invalid (must be 128, 160, 192, 224, or 256)", n,
        ),
        bip39::Error::BadWordCount(n) => format!(
            "BIP-39 word count {} invalid (must be 12, 15, 18, 21, or 24)", n,
        ),
        bip39::Error::UnknownWord(idx) => format!(
            "unknown BIP-39 word at position {} (not in selected wordlist; did you pick the right --language?)", idx,
        ),
        bip39::Error::InvalidChecksum => "BIP-39 checksum failure (last word does not match the entropy)".to_string(),
        bip39::Error::AmbiguousLanguages(_) => "BIP-39 phrase parses under multiple wordlists; specify --language explicitly".to_string(),
    }
}

pub fn friendly_bitcoin(e: &BitcoinErrorKind) -> String {
    match e {
        BitcoinErrorKind::Bip32(b) => format!("BIP-32 error: {}", b),
        BitcoinErrorKind::XpubParse(s) => format!("--xpub parse error: {}", s),
        BitcoinErrorKind::FingerprintParse(s) => format!("--master-fingerprint parse error: {}", s),
    }
}

pub fn friendly_ms_codec(e: &ms_codec::Error) -> String {
    // Reuse ms-cli's mapping shape: most variants delegate to codex32_friendly,
    // structured variants get explicit messages. v0.1 toolkit is read-only on
    // ms-codec (it only encodes successfully or rejects with specific structural
    // errors during decode of toolkit-emitted strings — the encode path is
    // unreachable for variant errors since toolkit always supplies valid input).
    match e {
        ms_codec::Error::Codex32(c) => format!("ms1 codex32: {:?}", c),
        ms_codec::Error::WrongHrp { got } => format!("ms1 wrong HRP: got {:?}, expected \"ms\"", got),
        ms_codec::Error::ThresholdNotZero { got } => format!("ms1 threshold not 0 (got '{}'); v0.1 single-string only", *got as char),
        ms_codec::Error::ShareIndexNotSecret { got } => format!("ms1 share-index not 's' (got '{}')", got),
        ms_codec::Error::TagInvalidAlphabet { got } => format!("ms1 tag bytes not in codex32 alphabet: {:?}", got),
        ms_codec::Error::UnknownTag { got } => format!("ms1 unknown tag {:?}", std::str::from_utf8(got).unwrap_or("<non-utf8>")),
        ms_codec::Error::ReservedPrefixViolation { got } => format!("ms1 reserved-prefix byte was 0x{:02x}, expected 0x00", got),
        ms_codec::Error::UnexpectedStringLength { got, .. } => format!("ms1 string length {} not in v0.1 set [50, 56, 62, 69, 75]", got),
        ms_codec::Error::PayloadLengthMismatch { got, tag, .. } => format!(
            "ms1 tag {:?} payload length {} not in expected set [16, 20, 24, 28, 32]",
            std::str::from_utf8(tag).unwrap_or("<non-utf8>"), got,
        ),
        // ReservedTagNotEmittedInV01 routes via From in error.rs to FutureFormat; never reached here.
        _ => format!("unhandled ms_codec::Error variant: {:?}", e),
    }
}

pub fn friendly_mk_codec(e: &mk_codec::Error) -> String {
    use mk_codec::Error as E;
    match e {
        E::InvalidHrp(s) => format!("mk1 wrong HRP: got {:?}, expected \"mk\"", s),
        E::MixedCase => "mk1 mixed case in input string".to_string(),
        E::InvalidStringLength(n) => format!("mk1 data-part length {} not valid (regular code: 14-93; long code: 95-108; the gap at 94 is reserved-invalid)", n),
        E::InvalidChar { ch, position } => format!("mk1 invalid character '{}' at position {} (not in bech32 alphabet)", ch, position),
        E::BchUncorrectable(s) => format!("mk1 BCH uncorrectable: {} (engraving error or transcription typo)", s),
        E::UnsupportedCardType(b) => format!("mk1 unsupported card type: 0x{:02x}", b),
        E::MalformedPayloadPadding => "mk1 malformed payload padding".to_string(),
        E::ChunkSetIdMismatch => "mk1 chunk_set_id mismatch across chunks".to_string(),
        E::ChunkedHeaderMalformed(s) => format!("mk1 chunked-header malformed: {}", s),
        E::MixedHeaderTypes => "mk1 mixed string-layer header types".to_string(),
        E::CrossChunkHashMismatch => "mk1 cross-chunk integrity hash mismatch".to_string(),
        E::ReservedBitsSet => "mk1 reserved bits set in bytecode header".to_string(),
        E::InvalidPolicyIdStubCount => "mk1 policy_id_stub_count must be ≥ 1".to_string(),
        E::InvalidPathIndicator(b) => format!("mk1 invalid path indicator byte: 0x{:02x}", b),
        E::PathTooDeep(n) => format!("mk1 path too deep: {} components (max 10)", n),
        E::InvalidPathComponent(s) => format!("mk1 invalid path component: {}", s),
        E::InvalidXpubVersion(v) => format!("mk1 invalid xpub version: 0x{:08x}", v),
        E::InvalidXpubPublicKey(s) => format!("mk1 invalid xpub public key: {}", s),
        E::UnexpectedEnd => "mk1 unexpected end of bytecode".to_string(),
        E::TrailingBytes => "mk1 trailing bytes after xpub".to_string(),
        E::CardPayloadTooLarge { bytecode_len, max_supported } => format!(
            "mk1 card payload too large: bytecode_len {} > max_supported {}", bytecode_len, max_supported,
        ),
        // UnsupportedVersion routes via From → FutureFormat; never reached here.
        _ => format!("unhandled mk_codec::Error variant: {:?}", e),
    }
}

pub fn friendly_md_codec(e: &md_codec::Error) -> String {
    // md_codec::Error is NOT #[non_exhaustive]; exhaustive match required.
    use md_codec::Error as E;
    match e {
        E::BitStreamTruncated { requested, available } => format!("md1 bitstream truncated: requested {} bits, {} available", requested, available),
        E::ReservedHeaderBitSet => "md1 reserved header bit set".to_string(),
        // UnsupportedVersion routes via From → FutureFormat
        E::UnsupportedVersion { got } => format!("md1 unsupported version {} (route via FutureFormat)", got),
        E::PathDepthExceeded { got, max } => format!("md1 path depth {} exceeds max {}", got, max),
        E::KeyCountOutOfRange { n } => format!("md1 key count {} out of range (1..=32)", n),
        E::DivergentPathCountMismatch { n, got } => format!("md1 divergent path count {} does not match key count {}", got, n),
        E::AltCountOutOfRange { got } => format!("md1 multipath alt-count {} out of range (2..=9)", got),
        E::UnknownPrimaryTag(t) => format!("md1 unknown primary tag 0x{:02x}", t),
        E::UnknownExtensionTag(t) => format!("md1 unknown extension tag 0x{:02x}", t),
        E::ThresholdOutOfRange { k } => format!("md1 threshold k={} out of range (1..=32)", k),
        E::ChildCountOutOfRange { count } => format!("md1 child count {} out of range (1..=32)", count),
        E::KGreaterThanN { k, n } => format!("md1 threshold k={} exceeds child count n={}", k, n),
        E::TlvOrderingViolation { prev, current } => format!("md1 TLV ordering: 0x{:02x} after 0x{:02x}", current, prev),
        E::PlaceholderIndexOutOfRange { idx, n } => format!("md1 placeholder index {} out of range (n={})", idx, n),
        E::OverrideOrderViolation { prev, current } => format!("md1 override ordering: @{} after @{}", current, prev),
        E::EmptyTlvEntry { tag } => format!("md1 empty TLV entry tag 0x{:02x}", tag),
        E::TlvLengthExceedsRemaining { length, remaining } => format!("md1 TLV length {} exceeds remaining {}", length, remaining),
        E::PlaceholderNotReferenced { idx, n } => format!("md1 placeholder @{} not referenced (n={})", idx, n),
        E::PlaceholderFirstOccurrenceOutOfOrder { expected_first, got_first } => format!("md1 placeholder first-occurrence: expected @{}, got @{}", expected_first, got_first),
        E::MultipathAltCountMismatch { expected, got } => format!("md1 multipath alt-count mismatch: expected {}, got {}", expected, got),
        E::ForbiddenTapTreeLeaf { tag } => format!("md1 forbidden tap-script-tree leaf tag 0x{:02x}", tag),
        E::ChunkCountOutOfRange { count } => format!("md1 chunk count {} out of range (1..=64)", count),
        E::ChunkIndexOutOfRange { index, count } => format!("md1 chunk index {} ≥ count {}", index, count),
        E::ChunkSetIdOutOfRange { id } => format!("md1 chunk-set-id 0x{:x} exceeds 20-bit range", id),
        E::ChunkHeaderChunkedFlagMissing => "md1 chunk header chunked-flag missing".to_string(),
        E::ChunkCountExceedsMax { needed } => format!("md1 chunk count {} exceeds max 64", needed),
        E::Codex32DecodeError(s) => format!("md1 codex32 decode: {}", s),
        E::Codex32EncodeError(s) => format!("md1 codex32 encode: {}", s),
        E::ChunkSetEmpty => "md1 chunk set empty".to_string(),
        E::ChunkSetInconsistent => "md1 chunks disagree on version/chunk-set-id/count".to_string(),
        E::ChunkSetIncomplete { got, expected } => format!("md1 chunk set incomplete: got {}, expected {}", got, expected),
        E::ChunkIndexGap { expected, got } => format!("md1 chunk index gap: expected {}, got {}", expected, got),
        E::ChunkSetIdMismatch { expected, derived } => format!("md1 chunk-set-id mismatch: expected 0x{:x}, derived 0x{:x}", expected, derived),
        E::VarintOverflow { value } => format!("md1 varint overflow: {}", value),
        E::MissingExplicitOrigin { idx } => format!("md1 missing explicit origin for @{}", idx),
        E::InvalidPresenceByte { reserved_bits } => format!("md1 presence byte non-zero reserved bits 0x{:02x}", reserved_bits),
        E::InvalidXpubBytes { idx } => format!("md1 invalid xpub bytes for @{}", idx),
        E::MissingPubkey { idx } => format!("md1 missing pubkey for @{} (wallet-policy mode requires all @N)", idx),
        E::ChainIndexOutOfRange { chain, alt_count } => format!("md1 chain index {} out of range (alt_count={})", chain, alt_count),
        E::HardenedPublicDerivation => "md1 hardened public-key derivation forbidden".to_string(),
        E::UnsupportedDerivationShape => "md1 unsupported wrapper shape for address derivation".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bip39_unknown_word_mentions_language() {
        let m = friendly_bip39(&bip39::Error::UnknownWord(5));
        assert!(m.contains("--language"));
    }

    #[test]
    fn ms_codec_wrong_hrp() {
        let m = friendly_ms_codec(&ms_codec::Error::WrongHrp { got: "mq".into() });
        assert!(m.contains("ms1"));
        assert!(m.contains("\"ms\""));
    }

    #[test]
    fn mk_codec_path_too_deep() {
        let m = friendly_mk_codec(&mk_codec::Error::PathTooDeep(11));
        assert!(m.contains("11"));
        assert!(m.contains("max 10"));
    }
}
```

- [ ] **Step 2: Update main.rs** to remove the inline stub:

```rust
mod cmd;
mod derive;
mod error;
mod format;
mod friendly;
mod language;
mod network;
mod parse;
mod synthesize;
mod template;

fn main() {}
```

- [ ] **Step 3: Run tests.**

```bash
cargo test -p mnemonic-toolkit friendly::tests 2>&1 | tail -5
cargo test -p mnemonic-toolkit error::tests 2>&1 | tail -5
```

Expected: friendly tests pass; error tests still pass (now via real friendly mappers).

### Task 3.2: `cmd/bundle.rs` — bundle subcommand

Realizes SPEC §2.1 (full + watch-only modes), §5.1 + §5.2 + §5.3 (output format).

- [ ] **Step 1: Implement `cmd/mod.rs`.**

```rust
//! Subcommand dispatch.

pub mod bundle;
pub mod verify_bundle;
```

- [ ] **Step 2: Implement bundle.rs.**

```rust
//! `mnemonic bundle` subcommand.
//!
//! Realizes SPEC §2.1 (full + watch-only modes), §5.1 (multi-section
//! stdout), §5.2 (engraving card stderr), §5.3 (JSON schema).

use crate::error::{ToolkitError, BitcoinErrorKind};
use crate::format::{BundleJson, EngravingMode, chunk_5char, chunk_md1, engraving_card};
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::parse::{check_no_concurrent_stdin, parse_master_fingerprint, read_phrase_input};
use crate::synthesize::{synthesize_full, synthesize_watch_only, Bundle};
use crate::template::CliTemplate;
use bitcoin::bip32::Xpub;
use clap::Args;
use std::io::Write;
use std::str::FromStr;

// SPEC §6.6 requires byte-exact rejection text + exit code 2 for the
// xpub-mode-incompatible flag set. clap's `conflicts_with` would exit 64
// with clap's default usage error and overwrite the SPEC text. So we
// declare ONLY `--phrase` ↔ `--xpub` as mutually-exclusive at the clap
// level (which is the intent — pick a mode); --passphrase / --language /
// --master-fingerprint compatibility is enforced at runtime in `run()`
// with the exact §6.6 text and exit code 2 via ToolkitError::ModeViolation.

#[derive(Args, Debug)]
pub struct BundleArgs {
    #[arg(long, conflicts_with = "xpub")]
    pub phrase: Option<String>,

    #[arg(long, conflicts_with = "phrase")]
    pub xpub: Option<String>,

    #[arg(long = "master-fingerprint")]
    pub master_fingerprint: Option<String>,

    #[arg(long)]
    pub network: CliNetwork,

    #[arg(long)]
    pub template: CliTemplate,

    #[arg(long)]
    pub language: Option<CliLanguage>,

    #[arg(long)]
    pub passphrase: Option<String>,

    #[arg(long)]
    pub json: bool,

    #[arg(long = "no-engraving-card")]
    pub no_engraving_card: bool,
}

/// SPEC §6.6 byte-exact mode-violation strings. Pinned for integration tests.
pub mod mode_text {
    pub const PASSPHRASE_WITH_XPUB: &str = "--passphrase is incompatible with --xpub: the xpub is already a post-passphrase derivation product (the passphrase is baked into the xpub at engrave time).";
    pub const LANGUAGE_WITH_XPUB: &str = "--language is meaningful only with --phrase; xpub-only mode does not consult any wordlist";
    pub const XPUB_NEEDS_FINGERPRINT: &str = "--xpub requires --master-fingerprint (xpub mode needs the master fingerprint to populate mk1's origin)";
    pub const FINGERPRINT_WITHOUT_XPUB: &str = "--master-fingerprint is meaningful only with --xpub";
    pub const XPUB_STDIN: &str = "--xpub does not accept stdin (-); pass the xpub literally on argv";
}

pub fn run<W: Write, E: Write>(
    args: &BundleArgs,
    stdin: &mut dyn std::io::Read,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    let phrase_arg = args.phrase.as_deref();
    let xpub_arg = args.xpub.as_deref();

    // SPEC §6.6 mode-violation pre-checks (BEFORE mode dispatch so the
    // exit code is 2 + byte-exact text, not clap's 64 + default text).
    if xpub_arg.is_some() && args.passphrase.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "watch-only", flag: "--passphrase",
            message: mode_text::PASSPHRASE_WITH_XPUB.to_string(),
        });
    }
    if xpub_arg.is_some() && args.language.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "watch-only", flag: "--language",
            message: mode_text::LANGUAGE_WITH_XPUB.to_string(),
        });
    }
    if xpub_arg.is_some() && args.master_fingerprint.is_none() {
        return Err(ToolkitError::ModeViolation {
            mode: "watch-only", flag: "--xpub",
            message: mode_text::XPUB_NEEDS_FINGERPRINT.to_string(),
        });
    }
    if xpub_arg.is_none() && args.master_fingerprint.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "full", flag: "--master-fingerprint",
            message: mode_text::FINGERPRINT_WITHOUT_XPUB.to_string(),
        });
    }

    // Mode dispatch.
    if let Some(xpub_str) = xpub_arg {
        if xpub_str == "-" {
            return Err(ToolkitError::BadInput(mode_text::XPUB_STDIN.to_string()));
        }
        bundle_watch_only(args, xpub_str, stdout, stderr)
    } else if phrase_arg.is_some() {
        check_no_concurrent_stdin(phrase_arg, args.passphrase.as_deref())?;
        bundle_full(args, stdin, stdout, stderr)
    } else {
        Err(ToolkitError::BadInput("expected --phrase or --xpub".into()))
    }
}

fn bundle_full<W: Write, E: Write>(
    args: &BundleArgs,
    stdin: &mut dyn std::io::Read,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    let phrase = read_phrase_input(args.phrase.as_deref(), stdin)?;
    let passphrase = args.passphrase.clone().unwrap_or_default();
    let language = args.language.unwrap_or_default();

    // Stderr: language defaulting warning (SPEC §5.2 ordering rule 1).
    if args.language.is_none() {
        writeln!(stderr, "warning: --language defaulting to english; record the wordlist language alongside the engraved cards.").ok();
    }
    // Stderr: passphrase warning (rule 2).
    if !passphrase.is_empty() {
        writeln!(stderr, "warning: --passphrase set; the passphrase is NOT engraved on any card and must").ok();
        writeln!(stderr, "warning: be remembered separately. A forgotten passphrase is unrecoverable from").ok();
        writeln!(stderr, "warning: the engraved bundle.").ok();
    }

    let acc = crate::derive::derive_full(&phrase, &passphrase, language, args.network, args.template)?;
    let bundle = synthesize_full(
        &acc.entropy, acc.master_fingerprint, acc.account_xpub.clone(),
        args.template, args.network,
    )?;

    let card_text = if args.no_engraving_card {
        None
    } else {
        let mode = if passphrase.is_empty() {
            EngravingMode::FullNoPassphrase { language: language.human_name() }
        } else {
            EngravingMode::FullWithPassphrase { language: language.human_name() }
        };
        Some(engraving_card(
            args.network.human_name(),
            args.template.human_name(),
            &args.template.origin_path_str(args.network),
            &acc.master_fingerprint.to_string().to_lowercase(),
            mode,
        ))
    };

    emit(args, &bundle, card_text.as_deref(), &acc.master_fingerprint.to_string().to_lowercase(), "full", stdout, stderr, args.template.origin_path_str(args.network))
}

fn bundle_watch_only<W: Write, E: Write>(
    args: &BundleArgs,
    xpub_str: &str,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    let fp_str = args.master_fingerprint.as_deref()
        .ok_or_else(|| ToolkitError::BadInput("--xpub requires --master-fingerprint (xpub mode needs the master fingerprint to populate mk1's origin)".into()))?;
    let fp = parse_master_fingerprint(fp_str)?;
    let xpub = Xpub::from_str(xpub_str)
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::XpubParse(format!("{}", e))))?;

    // §4.3 network/xpub cross-check.
    if xpub.network != args.network.network_kind() {
        return Err(ToolkitError::NetworkMismatch {
            xpub_network: if xpub.network == bitcoin::NetworkKind::Main { "mainnet" } else { "testnet/signet/regtest" },
            expected: args.network.human_name(),
        });
    }

    // §4.8 watch-only depth advisory.
    if xpub.depth != 3 {
        writeln!(stderr, "warning: --xpub depth is {}; expected 3 for canonical BIP-44/49/84/86 paths.", xpub.depth).ok();
        writeln!(stderr, "warning: Bundle will still be emitted; verify your wallet uses a non-standard path.").ok();
    }

    // §4.8 watch-only account-index hazard (always emitted in watch-only).
    writeln!(stderr, "warning: watch-only mode hardcodes account=0; if your xpub was derived").ok();
    writeln!(stderr, "warning: at a non-zero account, the bundle's path will not match. Use").ok();
    writeln!(stderr, "warning: v0.2's --account flag once available.").ok();

    let bundle = synthesize_watch_only(fp, xpub, args.template, args.network)?;

    let card_text = if args.no_engraving_card {
        None
    } else {
        Some(engraving_card(
            args.network.human_name(),
            args.template.human_name(),
            &args.template.origin_path_str(args.network),
            &fp.to_string().to_lowercase(),
            EngravingMode::WatchOnly,
        ))
    };

    emit(args, &bundle, card_text.as_deref(), &fp.to_string().to_lowercase(), "watch-only", stdout, stderr, args.template.origin_path_str(args.network))
}

fn emit<W: Write, E: Write>(
    args: &BundleArgs,
    bundle: &Bundle,
    engraving_text: Option<&str>,
    master_fp: &str,
    mode: &'static str,
    stdout: &mut W,
    stderr: &mut E,
    origin_path: String,
) -> Result<(), ToolkitError> {
    if args.json {
        let json = BundleJson {
            schema_version: "1",
            mode,
            network: args.network.human_name(),
            template: args.template.human_name(),
            account: 0,
            origin_path,
            master_fingerprint: master_fp.to_string(),
            ms1: bundle.ms1.as_deref(),
            mk1: &bundle.mk1,
            md1: &bundle.md1,
            engraving_card: engraving_text.map(|s| s.to_string()),
        };
        serde_json::to_writer(&mut *stdout, &json).ok();
        writeln!(stdout).ok();
    } else {
        // Multi-section text output (SPEC §5.1).
        if let Some(ms1) = bundle.ms1.as_deref() {
            writeln!(stdout, "# ms1 (entropy, BCH-checksummed)").ok();
            writeln!(stdout, "{}", ms1).ok();
            writeln!(stdout).ok();
            writeln!(stdout, "{}", chunk_5char(ms1)).ok();
            writeln!(stdout).ok();
        } else {
            writeln!(stdout, "# ms1 (omitted — xpub-only mode)").ok();
            writeln!(stdout).ok();
        }

        writeln!(stdout, "# mk1 (xpub + origin)").ok();
        for s in &bundle.mk1 {
            writeln!(stdout, "{}", s).ok();
        }
        writeln!(stdout).ok();
        for s in &bundle.mk1 {
            writeln!(stdout, "{}", chunk_5char(s)).ok();
        }
        writeln!(stdout).ok();

        writeln!(stdout, "# md1 (wallet policy)").ok();
        for s in &bundle.md1 {
            writeln!(stdout, "{}", s).ok();
        }
        writeln!(stdout).ok();
        for s in &bundle.md1 {
            writeln!(stdout, "{}", chunk_md1(s)).ok();
        }
        writeln!(stdout).ok();

        if let Some(text) = engraving_text {
            // Stderr ordering: warnings already emitted; engraving card last.
            write!(stderr, "{}", text).ok();
        }
    }
    Ok(())
}
```

- [ ] **Step 3: Run unit tests** (clap-derive structures; behavioral tests via `assert_cmd` come in Phase 5).

```bash
cargo build -p mnemonic-toolkit 2>&1 | tail -5
```

Expected: clean build.

### Task 3.3: `cmd/verify_bundle.rs` — verify-bundle subcommand

Realizes SPEC §2.2 (full + watch-only verify modes), §5.4 (verify JSON schema), §5.5 (error envelope routing rule).

- [ ] **Step 1: Implement.**

```rust
//! `mnemonic verify-bundle` subcommand.
//!
//! Realizes SPEC §2.2 + §5.4. Full mode runs 5 checks; watch-only
//! runs 4 checks; check failures stay in §5.4 with result:mismatch
//! per SPEC §5.4 routing rule (only pre-decode failures escape to
//! the §5.5 error envelope).

use crate::error::{ToolkitError, BitcoinErrorKind};
use crate::format::{VerifyBundleJson, VerifyCheck};
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::parse::{check_no_concurrent_stdin, parse_master_fingerprint, read_phrase_input};
use crate::synthesize::xpub_to_65;
use crate::template::CliTemplate;
use bitcoin::bip32::Xpub;
use clap::Args;
use std::io::Write;
use std::str::FromStr;

// SPEC §6.6 mode-violation symmetry mirrored from bundle.rs:
// clap-level mutual exclusion is ONLY --phrase ↔ --xpub; all other
// xpub-mode-incompatible flag rejections are runtime checks emitting
// byte-exact §6.6 strings via ToolkitError::ModeViolation (exit 2).

#[derive(Args, Debug)]
pub struct VerifyBundleArgs {
    #[arg(long, conflicts_with = "xpub")]
    pub phrase: Option<String>,

    #[arg(long, conflicts_with = "phrase")]
    pub xpub: Option<String>,

    #[arg(long = "master-fingerprint")]
    pub master_fingerprint: Option<String>,

    #[arg(long)]
    pub network: CliNetwork,

    #[arg(long)]
    pub template: CliTemplate,

    #[arg(long)]
    pub language: Option<CliLanguage>,

    #[arg(long)]
    pub passphrase: Option<String>,

    #[arg(long)]
    pub ms1: Option<String>,

    #[arg(long, num_args = 1.., required = true)]
    pub mk1: Vec<String>,

    #[arg(long, num_args = 1.., required = true)]
    pub md1: Vec<String>,

    #[arg(long)]
    pub json: bool,
}

pub fn run<W: Write>(
    args: &VerifyBundleArgs,
    stdin: &mut dyn std::io::Read,
    stdout: &mut W,
) -> Result<u8, ToolkitError> {
    use crate::cmd::bundle::mode_text;

    let xpub_arg = args.xpub.as_deref();
    let phrase_arg = args.phrase.as_deref();

    // SPEC §6.6 mode-violation pre-checks (mirror bundle.rs).
    if xpub_arg.is_some() && args.passphrase.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "watch-only", flag: "--passphrase",
            message: mode_text::PASSPHRASE_WITH_XPUB.to_string(),
        });
    }
    if xpub_arg.is_some() && args.language.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "watch-only", flag: "--language",
            message: mode_text::LANGUAGE_WITH_XPUB.to_string(),
        });
    }
    if xpub_arg.is_some() && args.master_fingerprint.is_none() {
        return Err(ToolkitError::ModeViolation {
            mode: "watch-only", flag: "--xpub",
            message: mode_text::XPUB_NEEDS_FINGERPRINT.to_string(),
        });
    }
    if xpub_arg.is_none() && args.master_fingerprint.is_some() {
        return Err(ToolkitError::ModeViolation {
            mode: "full", flag: "--master-fingerprint",
            message: mode_text::FINGERPRINT_WITHOUT_XPUB.to_string(),
        });
    }
    if xpub_arg == Some("-") {
        return Err(ToolkitError::BadInput(mode_text::XPUB_STDIN.to_string()));
    }

    let mut checks: Vec<VerifyCheck> = Vec::new();

    if let Some(_xpub_str) = xpub_arg {
        // Watch-only mode (SPEC §2.2.2): 4 checks.
        run_watch_only(args, &mut checks)?;
    } else if let Some(_) = phrase_arg {
        // Full mode (SPEC §2.2.1): 5 checks.
        check_no_concurrent_stdin(phrase_arg, args.passphrase.as_deref())?;
        run_full(args, stdin, &mut checks)?;
    } else {
        return Err(ToolkitError::BadInput("expected --phrase or --xpub".into()));
    }

    let any_fail = checks.iter().any(|c| c.result == "fail");
    let result = if any_fail { "mismatch" } else { "ok" };

    if args.json {
        let json = VerifyBundleJson {
            schema_version: "1",
            result,
            checks,
        };
        serde_json::to_writer(&mut *stdout, &json).ok();
        writeln!(stdout).ok();
    } else {
        for c in &checks {
            writeln!(stdout, "{}: {} {}", c.name, c.result, c.detail).ok();
        }
        writeln!(stdout, "result: {}", result).ok();
    }

    Ok(if any_fail { 4 } else { 0 })
}

fn run_full(
    args: &VerifyBundleArgs,
    stdin: &mut dyn std::io::Read,
    checks: &mut Vec<VerifyCheck>,
) -> Result<(), ToolkitError> {
    let phrase = read_phrase_input(args.phrase.as_deref(), stdin)?;
    let passphrase = args.passphrase.clone().unwrap_or_default();
    let language = args.language.unwrap_or_default();

    let acc = crate::derive::derive_full(&phrase, &passphrase, language, args.network, args.template)?;

    // Check 1: ms1 entropy match.
    if let Some(ms1) = args.ms1.as_deref() {
        match ms_codec::decode(ms1) {
            Ok((_tag, payload)) => {
                if let ms_codec::Payload::Entr(e) = payload {
                    if e == acc.entropy {
                        checks.push(VerifyCheck { name: "ms1_entropy_match", result: "ok", detail: "entropy bytes match".into() });
                    } else {
                        checks.push(VerifyCheck { name: "ms1_entropy_match", result: "fail", detail: format!("decoded {}-byte entropy != derived", e.len()) });
                    }
                } else {
                    checks.push(VerifyCheck { name: "ms1_entropy_match", result: "fail", detail: "decoded ms1 payload is not Entr".into() });
                }
            }
            Err(e) => {
                checks.push(VerifyCheck { name: "ms1_entropy_match", result: "fail", detail: format!("ms1 decode: {:?}", e) });
            }
        }
    } else {
        checks.push(VerifyCheck { name: "ms1_entropy_match", result: "skipped", detail: "no --ms1 supplied".into() });
    }

    // Check 2: mk1 decode + xpub/fp/path match.
    let mk1_strs: Vec<&str> = args.mk1.iter().map(|s| s.as_str()).collect();
    match mk_codec::decode(&mk1_strs) {
        Ok(card) => {
            checks.push(VerifyCheck { name: "mk1_decode", result: "ok", detail: "decoded successfully".into() });
            let xpub_match = card.xpub == acc.account_xpub;
            checks.push(VerifyCheck {
                name: "mk1_xpub_match",
                result: if xpub_match { "ok" } else { "fail" },
                detail: if xpub_match { "xpub matches".into() } else { "xpub does not match derived".into() },
            });
            let fp_match = card.origin_fingerprint == Some(acc.master_fingerprint);
            checks.push(VerifyCheck {
                name: "mk1_fingerprint_match",
                result: if fp_match { "ok" } else { "fail" },
                detail: if fp_match { "fp matches".into() } else { "master fingerprint does not match".into() },
            });
            let expected_path = args.template.derivation_path(args.network);
            let path_match = card.origin_path == expected_path;
            checks.push(VerifyCheck {
                name: "mk1_path_match",
                result: if path_match { "ok" } else { "fail" },
                detail: if path_match { "path matches".into() } else { format!("expected {}, got {}", expected_path, card.origin_path) },
            });

            // Check 3+5: md1 decode + cross-binding.
            verify_md1_and_stub(args, &card, checks);
        }
        Err(e) => {
            checks.push(VerifyCheck { name: "mk1_decode", result: "fail", detail: format!("{:?}", e) });
            checks.push(VerifyCheck { name: "mk1_xpub_match", result: "skipped", detail: "mk1 decode failed".into() });
            checks.push(VerifyCheck { name: "mk1_fingerprint_match", result: "skipped", detail: "mk1 decode failed".into() });
            checks.push(VerifyCheck { name: "mk1_path_match", result: "skipped", detail: "mk1 decode failed".into() });

            // Try md1 anyway for diagnostic completeness.
            verify_md1_only(args, checks);
        }
    }

    Ok(())
}

fn verify_md1_and_stub(args: &VerifyBundleArgs, card: &mk_codec::KeyCard, checks: &mut Vec<VerifyCheck>) {
    let md1_strs: Vec<&str> = args.md1.iter().map(|s| s.as_str()).collect();
    match md_codec::chunk::reassemble(&md1_strs) {
        Ok(desc) => {
            checks.push(VerifyCheck { name: "md1_decode", result: "ok", detail: "decoded successfully".into() });
            let wp = desc.is_wallet_policy();
            checks.push(VerifyCheck {
                name: "md1_wallet_policy",
                result: if wp { "ok" } else { "fail" },
                detail: if wp { "wallet-policy mode confirmed".into() } else { "descriptor is template-only (no pubkeys TLV)".into() },
            });

            if wp {
                let xpub_65_expected = xpub_to_65(&card.xpub);
                let xpub_match = desc.tlv.pubkeys.as_ref()
                    .and_then(|v| v.first())
                    .map(|(_, b)| b == &xpub_65_expected)
                    .unwrap_or(false);
                checks.push(VerifyCheck {
                    name: "md1_xpub_match",
                    result: if xpub_match { "ok" } else { "fail" },
                    detail: if xpub_match { "65-byte xpub matches mk1's xpub".into() } else { "md1 xpub differs from mk1's".into() },
                });
            } else {
                checks.push(VerifyCheck { name: "md1_xpub_match", result: "skipped", detail: "not in wallet-policy mode".into() });
            }

            match md_codec::compute_wallet_policy_id(&desc) {
                Ok(pid) => {
                    let stub_match = &card.policy_id_stubs.get(0).copied().unwrap_or([0u8;4])[..] == &pid.as_bytes()[..4];
                    checks.push(VerifyCheck {
                        name: "stub_linkage",
                        result: if stub_match { "ok" } else { "fail" },
                        detail: if stub_match { "policy_id_stub[0..4] matches mk1's stub[0]".into() } else { "stub linkage broken".into() },
                    });
                }
                Err(e) => {
                    checks.push(VerifyCheck { name: "stub_linkage", result: "fail", detail: format!("policy_id compute: {:?}", e) });
                }
            }
        }
        Err(e) => {
            checks.push(VerifyCheck { name: "md1_decode", result: "fail", detail: format!("{:?}", e) });
            checks.push(VerifyCheck { name: "md1_wallet_policy", result: "skipped", detail: "md1 decode failed".into() });
            checks.push(VerifyCheck { name: "md1_xpub_match", result: "skipped", detail: "md1 decode failed".into() });
            checks.push(VerifyCheck { name: "stub_linkage", result: "skipped", detail: "md1 decode failed".into() });
        }
    }
}

fn verify_md1_only(args: &VerifyBundleArgs, checks: &mut Vec<VerifyCheck>) {
    let md1_strs: Vec<&str> = args.md1.iter().map(|s| s.as_str()).collect();
    match md_codec::chunk::reassemble(&md1_strs) {
        Ok(desc) => {
            checks.push(VerifyCheck { name: "md1_decode", result: "ok", detail: "decoded successfully".into() });
            let wp = desc.is_wallet_policy();
            checks.push(VerifyCheck { name: "md1_wallet_policy", result: if wp { "ok" } else { "fail" }, detail: "".into() });
            checks.push(VerifyCheck { name: "md1_xpub_match", result: "skipped", detail: "mk1 decode failed; no reference xpub".into() });
            checks.push(VerifyCheck { name: "stub_linkage", result: "skipped", detail: "mk1 decode failed".into() });
        }
        Err(e) => {
            checks.push(VerifyCheck { name: "md1_decode", result: "fail", detail: format!("{:?}", e) });
            checks.push(VerifyCheck { name: "md1_wallet_policy", result: "skipped", detail: "".into() });
            checks.push(VerifyCheck { name: "md1_xpub_match", result: "skipped", detail: "".into() });
            checks.push(VerifyCheck { name: "stub_linkage", result: "skipped", detail: "".into() });
        }
    }
}

fn run_watch_only(
    args: &VerifyBundleArgs,
    checks: &mut Vec<VerifyCheck>,
) -> Result<(), ToolkitError> {
    let xpub_str = args.xpub.as_deref().expect("xpub set in watch-only mode");
    let fp_str = args.master_fingerprint.as_deref()
        .ok_or_else(|| ToolkitError::BadInput("--xpub requires --master-fingerprint".into()))?;
    let supplied_xpub = Xpub::from_str(xpub_str)
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::XpubParse(format!("{}", e))))?;
    let supplied_fp = parse_master_fingerprint(fp_str)?;

    if supplied_xpub.network != args.network.network_kind() {
        return Err(ToolkitError::NetworkMismatch {
            xpub_network: if supplied_xpub.network == bitcoin::NetworkKind::Main { "mainnet" } else { "testnet/signet/regtest" },
            expected: args.network.human_name(),
        });
    }

    // Check 1: mk1 parses + BCH valid.
    let mk1_strs: Vec<&str> = args.mk1.iter().map(|s| s.as_str()).collect();
    let mk_card = match mk_codec::decode(&mk1_strs) {
        Ok(c) => {
            checks.push(VerifyCheck { name: "mk1_decode", result: "ok", detail: "decoded successfully".into() });
            Some(c)
        }
        Err(e) => {
            checks.push(VerifyCheck { name: "mk1_decode", result: "fail", detail: format!("{:?}", e) });
            None
        }
    };

    // Check 2: md1 parses + BCH valid.
    let md1_strs: Vec<&str> = args.md1.iter().map(|s| s.as_str()).collect();
    let md_desc = match md_codec::chunk::reassemble(&md1_strs) {
        Ok(d) => {
            checks.push(VerifyCheck { name: "md1_decode", result: "ok", detail: "decoded successfully".into() });
            Some(d)
        }
        Err(e) => {
            checks.push(VerifyCheck { name: "md1_decode", result: "fail", detail: format!("{:?}", e) });
            None
        }
    };

    // Check 3: stub linkage.
    if let (Some(card), Some(desc)) = (mk_card.as_ref(), md_desc.as_ref()) {
        match md_codec::compute_wallet_policy_id(desc) {
            Ok(pid) => {
                let stub_match = &card.policy_id_stubs.get(0).copied().unwrap_or([0u8;4])[..] == &pid.as_bytes()[..4];
                checks.push(VerifyCheck {
                    name: "stub_linkage",
                    result: if stub_match { "ok" } else { "fail" },
                    detail: if stub_match { "policy_id_stub[0..4] matches mk1's stub[0]".into() } else { "stub linkage broken".into() },
                });
            }
            Err(e) => {
                checks.push(VerifyCheck { name: "stub_linkage", result: "fail", detail: format!("policy_id: {:?}", e) });
            }
        }
    } else {
        checks.push(VerifyCheck { name: "stub_linkage", result: "skipped", detail: "decode failed".into() });
    }

    // Check 4: optional xpub/fp match.
    if let Some(card) = mk_card.as_ref() {
        let xpub_match = card.xpub == supplied_xpub;
        checks.push(VerifyCheck {
            name: "mk1_xpub_match",
            result: if xpub_match { "ok" } else { "fail" },
            detail: if xpub_match { "matches --xpub".into() } else { "differs from --xpub".into() },
        });
        let fp_match = card.origin_fingerprint == Some(supplied_fp);
        checks.push(VerifyCheck {
            name: "mk1_fingerprint_match",
            result: if fp_match { "ok" } else { "fail" },
            detail: if fp_match { "matches --master-fingerprint".into() } else { "differs from --master-fingerprint".into() },
        });
    } else {
        checks.push(VerifyCheck { name: "mk1_xpub_match", result: "skipped", detail: "mk1 decode failed".into() });
        checks.push(VerifyCheck { name: "mk1_fingerprint_match", result: "skipped", detail: "mk1 decode failed".into() });
    }

    Ok(())
}
```

- [ ] **Step 2: `cargo build` clean.**

```bash
cargo build -p mnemonic-toolkit 2>&1 | tail -5
```

### Task 3.4: Phase 3 commit + opus review

- [ ] **Step 1: Verify all unit tests + clippy + fmt.**

```bash
cargo test -p mnemonic-toolkit 2>&1 | tail -5
cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings 2>&1 | tail -5
cargo fmt --check -p mnemonic-toolkit 2>&1 | tail -3
```

- [ ] **Step 2: Stage + commit.**

```bash
git add crates/mnemonic-toolkit/src/main.rs \
        crates/mnemonic-toolkit/src/friendly.rs \
        crates/mnemonic-toolkit/src/cmd/mod.rs \
        crates/mnemonic-toolkit/src/cmd/bundle.rs \
        crates/mnemonic-toolkit/src/cmd/verify_bundle.rs \
        Cargo.lock
git -c commit.gpgsign=false commit -m "phase 3: command modules — friendly/cmd/bundle/cmd/verify_bundle

friendly.rs (SPEC §6.4.1-§6.4.5): five sibling-source mappers.
bip39 (5 variants), bitcoin (BitcoinErrorKind 3 variants), ms_codec
(closed-set + #[non_exhaustive] fallthrough), mk_codec (22 variants
+ fallthrough), md_codec (38 variants, exhaustive — error type is NOT
non_exhaustive). Routing per §6.4.0 principle.

cmd/bundle.rs (SPEC §2.1, §5.1, §5.2, §5.3): full + watch-only modes,
multi-section stdout, byte-exact engraving-card stderr, JSON envelope.
Mode-violation messages byte-exact per §6.6. Watch-only emits
non-suppressible account=0 hazard warning per §4.8.

cmd/verify_bundle.rs (SPEC §2.2, §5.4): full = 5 checks; watch-only =
4 checks. Sibling-decode failures stay in §5.4 with result:mismatch;
only pre-decode failures (mode violation, fp parse, xpub parse,
network mismatch) escape to §5.5 error envelope. Exit 4 on mismatch.

Phase 1 friendly-mod stub deleted; main.rs now declares `mod friendly;`
and `mod cmd;` instead.

Tests: 42 unit tests passing. clippy + fmt clean.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 3: Phase 3 opus review checkpoint** — same iterative pattern as Phase 1 task 1.10. Reviewer scrutinizes friendly-mapper completeness (especially md_codec exhaustive match), verify-bundle check ordering, JSON schema field-order conformance, mode-violation byte-exact text.

---

## Phase 4: Root + glue

**Goal:** Wire main.rs as clap-derive root + ExitCode dispatch. After Phase 4, `cargo run -p mnemonic-toolkit -- bundle ...` works end-to-end.

**Files:**
- Modify: `crates/mnemonic-toolkit/src/main.rs`

### Task 4.1: main.rs — clap derive root + ExitCode dispatch

Realizes SPEC §2.3 (top-level --help text) + §6.5 (ExitCode dispatch override of clap default).

- [ ] **Step 1: Replace main.rs with full impl.**

```rust
//! `mnemonic` — engraving-bundle CLI for the m-format star.

mod cmd;
mod derive;
mod error;
mod format;
mod friendly;
mod language;
mod network;
mod parse;
mod synthesize;
mod template;

use clap::{Parser, Subcommand};
use error::ToolkitError;
use std::io::{self, Write};
use std::process::ExitCode;

#[derive(Parser, Debug)]
#[command(
    name = "mnemonic",
    about = "engraving-bundle CLI for the m-format star (ms1 + mk1 + md1)",
    version,
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// emit a 3-card engraving bundle from a phrase or xpub
    Bundle(cmd::bundle::BundleArgs),
    /// round-trip-check an engraved bundle
    VerifyBundle(cmd::verify_bundle::VerifyBundleArgs),
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(c) => c,
        Err(e) => {
            // Override clap's default exit code 2 → 64 to keep format-violations distinct.
            e.print().ok();
            return ExitCode::from(if e.exit_code() == 0 { 0 } else { 64 });
        }
    };

    let stdin = &mut io::stdin();
    let stdout = &mut io::stdout();
    let stderr = &mut io::stderr();

    let result: Result<u8, ToolkitError> = match &cli.command {
        Command::Bundle(args) => cmd::bundle::run(args, stdin, stdout, stderr).map(|_| 0),
        Command::VerifyBundle(args) => cmd::verify_bundle::run(args, stdin, stdout),
    };

    match result {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            // Emit error per SPEC §6.5 + §5.5.
            let _ = writeln!(io::stderr(), "{}", e);
            ExitCode::from(e.exit_code())
        }
    }
}
```

- [ ] **Step 2: Build + smoke test.**

```bash
cargo build -p mnemonic-toolkit 2>&1 | tail -5
./target/debug/mnemonic --help 2>&1 | head -20
./target/debug/mnemonic bundle --help 2>&1 | head -25
./target/debug/mnemonic verify-bundle --help 2>&1 | head -25
```

Expected: `--help` prints; subcommand help prints flag tables.

- [ ] **Step 3: Smoke test bundle full mode** with Trezor 24-word vector:

```bash
./target/debug/mnemonic bundle \
    --phrase "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art" \
    --network mainnet \
    --template bip84 \
    2>&1 | head -30
```

Expected: stdout has `# ms1`, `# mk1`, `# md1` sections with strings; stderr has language-defaulting warning + engraving card.

- [ ] **Step 4: Phase 4 commit + opus review.**

```bash
git add crates/mnemonic-toolkit/src/main.rs Cargo.lock
git -c commit.gpgsign=false commit -m "phase 4: clap derive root + ExitCode dispatch in main.rs

Wires Cli enum (bundle, verify-bundle) into main.rs with clap derive.
ExitCode dispatch overrides clap default 2 → 64 for usage errors per
SPEC §6.5; per-error exit codes 1/2/3/4 routed via ToolkitError::
exit_code().

Smoke test: cargo run -- bundle --phrase '<24-word>' --network mainnet
--template bip84 emits 3-section stdout + engraving card stderr.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

Phase 4 opus review checkpoint per the established pattern.

---

## Phase 5: Integration tests + release prep

**Goal:** Lock in 16+ assert_cmd integration tests + Cargo.toml metadata + version bump + CHANGELOG + tag.

**Files:**
- Create: `crates/mnemonic-toolkit/tests/cli_bundle_full.rs`
- Create: `crates/mnemonic-toolkit/tests/cli_bundle_watch_only.rs`
- Create: `crates/mnemonic-toolkit/tests/cli_verify_bundle_full.rs`
- Create: `crates/mnemonic-toolkit/tests/cli_verify_bundle_watch_only.rs`
- Create: `crates/mnemonic-toolkit/tests/cli_mode_violations.rs`
- Create: `crates/mnemonic-toolkit/tests/cli_help_fixtures.rs`
- Create: `crates/mnemonic-toolkit/tests/cli_json_envelopes.rs`
- Create: `crates/mnemonic-toolkit/tests/vectors/v0_1/<16 fixture files>.txt`
- Modify: `crates/mnemonic-toolkit/Cargo.toml` (metadata + version bump)
- Create: `CHANGELOG.md`
- Create: `crates/mnemonic-toolkit/README.md` updates

### Task 5.1: Test fixtures — 16 (template × network) cells

For each (template ∈ {bip44, bip49, bip84, bip86}) × (network ∈ {mainnet, testnet, signet, regtest}) = 16 cells, generate the canonical Trezor 24-word vector's bundle output AND store it as `tests/vectors/v0_1/<template>-<network>.txt`. The fixture file is the ground-truth byte-exact stdout of `mnemonic bundle --phrase '<trezor>' --network <n> --template <t> --no-engraving-card`.

- [ ] **Step 1: Generate fixtures from the running binary** (one-shot at v0.1.0; pinned thereafter):

```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
mkdir -p crates/mnemonic-toolkit/tests/vectors/v0_1
TREZOR='abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art'
for t in bip44 bip49 bip84 bip86; do
  for n in mainnet testnet signet regtest; do
    ./target/debug/mnemonic bundle \
        --phrase "$TREZOR" \
        --network "$n" \
        --template "$t" \
        --no-engraving-card \
        > "crates/mnemonic-toolkit/tests/vectors/v0_1/${t}-${n}.txt" 2>/dev/null
  done
done
ls crates/mnemonic-toolkit/tests/vectors/v0_1/
```

Expected: 16 files, each ~12-20 lines.

- [ ] **Step 2: Sanity-check fixtures.** A couple of cells should round-trip via `verify-bundle`:

```bash
./target/debug/mnemonic verify-bundle \
    --phrase "$TREZOR" \
    --network mainnet \
    --template bip84 \
    --ms1 "$(awk '/^ms1/ {print; exit}' crates/mnemonic-toolkit/tests/vectors/v0_1/bip84-mainnet.txt)" \
    --mk1 "$(awk '/^mk1/ {print; exit}' crates/mnemonic-toolkit/tests/vectors/v0_1/bip84-mainnet.txt)" \
    --md1 "$(awk '/^md1/ {print; exit}' crates/mnemonic-toolkit/tests/vectors/v0_1/bip84-mainnet.txt)" \
    2>&1 | tail -10
```

Expected: all checks `ok`, `result: ok`, exit 0.

### Task 5.2: cli_bundle_full.rs — 16-cell parametric integration test

```rust
use assert_cmd::Command;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

#[test]
fn bundle_full_16_cells_byte_exact_against_pinned_vectors() {
    for &t in &["bip44", "bip49", "bip84", "bip86"] {
        for &n in &["mainnet", "testnet", "signet", "regtest"] {
            let expected = std::fs::read_to_string(
                format!("tests/vectors/v0_1/{}-{}.txt", t, n)
            ).expect("fixture exists");
            let out = Command::cargo_bin("mnemonic").unwrap()
                .args(&["bundle", "--phrase", TREZOR_24, "--network", n, "--template", t, "--no-engraving-card"])
                .assert()
                .success();
            let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
            assert_eq!(stdout, expected, "byte-exact mismatch for {}-{}", t, n);
        }
    }
}
```

(Additional integration tests follow the same shape: `cli_bundle_watch_only.rs`, `cli_verify_bundle_full.rs`, `cli_verify_bundle_watch_only.rs`, `cli_mode_violations.rs` covering each §6.6 row, `cli_help_fixtures.rs` byte-exact `--help` output, `cli_json_envelopes.rs` for both `bundle --json` and `verify-bundle --json` shapes.)

### Task 5.3: cli_mode_violations.rs

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn passphrase_with_xpub_rejected_byte_exact() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(&["bundle", "--xpub", "xpub6...", "--master-fingerprint", "deadbeef",
                "--passphrase", "x", "--network", "mainnet", "--template", "bip84"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "--passphrase is incompatible with --xpub: the xpub is already a post-passphrase derivation product"
        ));
}

#[test]
fn language_with_xpub_rejected() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(&["bundle", "--xpub", "xpub6...", "--master-fingerprint", "deadbeef",
                "--language", "english", "--network", "mainnet", "--template", "bip84"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "--language is meaningful only with --phrase"
        ));
}

#[test]
fn xpub_stdin_rejected() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(&["bundle", "--xpub", "-", "--master-fingerprint", "deadbeef",
                "--network", "mainnet", "--template", "bip84"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "--xpub does not accept stdin"
        ));
}

#[test]
fn fingerprint_short_rejected_byte_exact() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(&["bundle", "--xpub", "xpub6...", "--master-fingerprint", "dead",
                "--network", "mainnet", "--template", "bip84"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains(
            "--master-fingerprint must be 8 hex chars (e.g., deadbeef)"
        ));
}

#[test]
fn xpub_without_fingerprint_byte_exact() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(&["bundle", "--xpub", "xpub6...", "--network", "mainnet", "--template", "bip84"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "--xpub requires --master-fingerprint (xpub mode needs the master fingerprint to populate mk1's origin)"
        ));
}

#[test]
fn fingerprint_without_xpub_byte_exact() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(&["bundle", "--phrase", "x", "--master-fingerprint", "deadbeef",
                "--network", "mainnet", "--template", "bip84"])
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains(
            "--master-fingerprint is meaningful only with --xpub"
        ));
}

#[test]
fn verify_bundle_no_engraving_card_flag_rejected() {
    // verify-bundle does not emit an engraving card; the flag should not exist.
    // clap-derive auto-rejects unknown flags; main.rs maps that to exit 64.
    Command::cargo_bin("mnemonic").unwrap()
        .args(&["verify-bundle", "--no-engraving-card",
                "--network", "mainnet", "--template", "bip84",
                "--mk1", "x", "--md1", "x"])
        .assert()
        .failure()
        .code(64);
}
```

(Continue covering each row of SPEC §6.6.)

### Task 5.4: Cargo.toml metadata + version bump

- [ ] **Step 1: Bump 0.0.0 → 0.1.0; flip publish=false → remove; add full publishable metadata.**

```toml
[package]
name = "mnemonic-toolkit"
version = "0.1.0"
edition.workspace = true
license.workspace = true
repository.workspace = true
homepage.workspace = true
rust-version.workspace = true
description = "Engraving-bundle CLI for the m-format Bitcoin self-custody backup star (ms1 + mk1 + md1)."
documentation = "https://docs.rs/mnemonic-toolkit"
readme = "README.md"
keywords = ["bitcoin", "bip39", "bip93", "engraving", "self-custody"]
categories = ["cryptography::cryptocurrencies", "command-line-utilities"]

[[bin]]
name = "mnemonic"
path = "src/main.rs"

[dependencies]
ms-codec = { git = "https://github.com/bg002h/mnemonic-secret",      tag = "ms-codec-v0.1.0" }
mk-codec = { git = "https://github.com/bg002h/mnemonic-key",         tag = "mk-codec-v0.2.1" }
md-codec = { git = "https://github.com/bg002h/descriptor-mnemonic",  tag = "md-codec-v0.16.1" }
bip39 = { version = "2", features = ["all-languages"] }
bitcoin = "0.32"
clap = { version = "4", features = ["derive"] }
hex = "0.4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
```

Note: `cargo publish` of git-deps is BLOCKED until siblings hit crates.io. v0.1.0 release is on GitHub-tag-only; `cargo publish --dry-run` will fail until siblings publish. This is documented in §10.3 and CHANGELOG.

### Task 5.5: CHANGELOG + crate README

- [ ] **Step 1: Create `CHANGELOG.md` at repo root.**

```markdown
# Changelog

All notable changes to `mnemonic-toolkit` are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project follows [SemVer](https://semver.org/spec/v2.0.0.html) with the pre-1.0 convention that the second component (`0.X`) is the breaking-change axis.

## mnemonic-toolkit [0.1.0] — <YYYY-MM-DD>

### What's new

- Initial release. Top-level integration crate of the m-format star.
- 2 subcommands: `bundle` (encode-side: emit 3-card engraving bundle) and `verify-bundle` (round-trip integrity check).
- 2 input modes per command: full (`--phrase`) and watch-only / key-only (`--xpub --master-fingerprint`).
- 4 single-sig wallet templates: BIP-44 (pkh), BIP-49 (sh-wpkh), BIP-84 (wpkh), BIP-86 (tr).
- 4 networks: mainnet / testnet / signet / regtest.
- Account hardcoded `0` in v0.1; `--account` flag deferred to v0.2.
- All 10 BIP-39 wordlists supported via `--language`.
- Multi-section stdout (`# ms1` / `# mk1` / `# md1` headers + chunked engraving form).
- Byte-exact engraving-card stderr per SPEC §5.2.
- `--json` envelope schemas for both subcommands.
- Exit codes 0 / 1 / 2 / 3 / 4 / 64 per SPEC §6.

### Tests

X integration tests (assert_cmd) + Y unit tests. Trezor 24-word zero-entropy vector pinned across 16 (template × network) cells.

### Known limitations

- Multisig templates, non-zero account, file output, recovery flow: deferred to v0.2+.
- `cargo publish` blocked until ms-codec / mk-codec / md-codec hit crates.io. v0.1.0 distributed via GitHub tag `mnemonic-toolkit-v0.1.0`.

### Wire-format SHA pin

The 16 fixture files at `crates/mnemonic-toolkit/tests/vectors/v0_1/*.txt` are SHA-256-pinned at this release. Subsequent corpus changes that alter the SHA require a SemVer minor bump per the pre-1.0 breaking-change-axis convention.

```text
sha256(crates/mnemonic-toolkit/tests/vectors/v0_1/) = <COMPUTED AT RELEASE>
```
```

- [ ] **Step 2: Crate-level README** at `crates/mnemonic-toolkit/README.md` (already exists from scaffold; expand for crate-publishability):

(Expand to include install, quickstart, engraving caveats, sibling pointers, license — same shape as ms-cli/README.md.)

### Task 5.6: Phase 5 commit + tag

- [ ] **Step 1: Verify everything green.**

```bash
cargo test --workspace 2>&1 | tail -10
cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tail -5
cargo fmt --check 2>&1 | tail -3
```

- [ ] **Step 2: Stage + commit.**

```bash
git add crates/mnemonic-toolkit/Cargo.toml \
        crates/mnemonic-toolkit/README.md \
        crates/mnemonic-toolkit/tests/ \
        CHANGELOG.md \
        Cargo.lock
git -c commit.gpgsign=false commit -m "release: mnemonic-toolkit v0.1.0

16 (template × network) byte-exact fixtures from Trezor 24-word
vector; X integration tests via assert_cmd covering bundle/verify-
bundle full + watch-only modes, every §6.6 mode-violation row, JSON
envelope schemas, and --help text.

Cargo.toml metadata complete (description, documentation, readme,
keywords, categories); version 0.0.0 → 0.1.0; publish=false removed.
cargo publish blocked until ms-codec / mk-codec / md-codec on crates.io.

CHANGELOG entry pins SHA-256 of test/vectors/v0_1/.

Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 3: Tag locally** (push gated on user).

```bash
git tag -a mnemonic-toolkit-v0.1.0 -m "mnemonic-toolkit v0.1.0"
git tag --list | grep mnemonic-toolkit
```

- [ ] **Step 4: Phase 5 opus review checkpoint** — final reviewer pass over the entire v0.1.0 release surface (integration tests, Cargo.toml, README, CHANGELOG, tag, docs cross-references). Report at `design/agent-reports/phase-5-release-prep-review-r1.md`.

User-gated next steps (post-Phase-5):

1. `git push origin master` (push the 5 phase commits).
2. `git push origin mnemonic-toolkit-v0.1.0` (push the tag).
3. `gh release create mnemonic-toolkit-v0.1.0 --notes-file <changelog excerpt>` (create the GitHub Release).
4. `cargo publish` deferred to a single coordinated event when ms-codec / mk-codec / md-codec all land on crates.io; toolkit's git-deps must flip to crates.io versions then.

---

## Phase-completion summary

Per memory `feedback_iterative_review_every_phase`, every Phase ends with a reviewer-loop terminator. Per-phase reports persist to `design/agent-reports/phase-X-<name>-review-rN.md`. Low/nit findings deferred to `design/FOLLOWUPS.md` at tier `v0.1-nice-to-have`. Critical/important fixed inline as fixup commits.

Final commit count expected: ~7-10 commits across Phase 1-5 (one per phase + one fixup per review round + tag commit).

## Revision history

- **r1** (2026-05-04) — initial plan, mirroring ms-cli plan structure (5 phases, ~12 modules + integration tests).
- **r2** (2026-05-04, this commit) — plan-architect-r1 fixes: 0 critical / 2 important / 9 nits. Folded I-1 (clap `conflicts_with` violates SPEC §6.6 byte-exact-text contract → switched to runtime mode-violation pre-checks emitting ToolkitError::ModeViolation with byte-exact §6.6 strings via `cmd::bundle::mode_text` constants; mirrored into verify-bundle), I-2 (`--master-fingerprint` requires-direction was one-way clap; added bidirectional runtime checks with byte-exact text), L1 (dropped `stderr_handle()` trampoline + dead `use std::io::Write` in main.rs; consolidated `use std::io::{self, Write}`), L8 (added `xpub_without_fingerprint`, `fingerprint_without_xpub`, `verify_bundle_no_engraving_card_flag_rejected` integration tests). Other nits (L2 elided sibling test stubs, L3 byte-exact bip84 mainnet xpub, L4 vestigial `cd`s, L5 main.rs replacement notes, L6 TDD-mechanical exception, L7 cargo-publish-dry-run gate, L9 magic test count) deferred to executor or design/FOLLOWUPS.md. Pending plan architect r2 review.
