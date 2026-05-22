# `mnemonic nostr` (nostr-key wrappers) Implementation Plan — v0.34.0

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `mnemonic nostr` subcommand that wraps an existing nostr key (`npub`/`nsec`) as Bitcoin addresses, descriptors, and (for `nsec`) a WIF — across taproot (key-path) and non-taproot (even-y) script types.

**Architecture:** A binary-crate crypto/derivation module (`src/nostr.rs`: NIP-19 decode + even-y normalization + address/descriptor/WIF derivation), then a thin CLI layer (`src/cmd/nostr.rs`) that parses args, calls it, and renders output. **R0 C1 fix:** `nostr.rs` is a **binary-crate module** (`mod nostr;` in `main.rs`), NOT a `lib.rs` module — it depends on binary-crate types (`crate::error::ToolkitError`, `crate::cmd::convert::ScriptType`, `crate::network::CliNetwork`) that `lib.rs` does not re-export, and nothing external consumes nostr as a library API (the GUI consumes the CLI). The pure-crypto-in-`lib` convention (electrum_crypto/seedqr) is acknowledged and waived for this reason. No m-format cards (verified infeasible; see spec §10).

**Tech Stack:** Rust; `bitcoin` 0.32 (re-exports `bech32` 0.11 + `secp256k1` 0.29); `miniscript` 13 (patched fork) for descriptor checksums; `zeroize`. Spec: `design/BRAINSTORM_v0_34_0_nostr_key_wrappers.md`. Source SHA baseline: `f501ec3`.

**Conventions (CLAUDE.md):** per-phase TDD (tests before impl); explicit `git add` (never `-A`); new `ToolkitError` variants in alphabetical-by-variant order; per-phase opus reviewer-loop to 0 critical / 0 important; commit messages end with the `Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>` trailer.

---

## File structure

- **Create** `crates/mnemonic-toolkit/src/nostr.rs` — **binary-crate** crypto/decode module (decode, normalize, address, descriptor, wif). One responsibility: nostr-key ↔ Bitcoin primitives.
- **Create** `crates/mnemonic-toolkit/src/cmd/nostr.rs` — clap `NostrArgs` + `run()` + output rendering.
- **Create** `crates/mnemonic-toolkit/tests/cli_nostr.rs` — CLI integration cells.
- **Modify** `crates/mnemonic-toolkit/src/error.rs` — add `NostrKeyParse(String)` variant + `kind()` + `message()` + `exit_code()` arms (NO Display arm — Display delegates to `message()`; R0 C4).
- **Modify** `crates/mnemonic-toolkit/src/main.rs` — `mod nostr;` (binary-crate module registration; R0 C1) + `Command::Nostr` enum arm + dispatch arm.
- **Modify** `crates/mnemonic-toolkit/src/cmd/mod.rs` — `pub mod nostr;` (the CLI module).
- **Modify** `crates/mnemonic-toolkit/src/secrets.rs` — mark `--secret` / `--secret-stdin` secret in `flag_is_secret` (single-arg by flag name; R0 C3).
- **Modify** `crates/mnemonic-toolkit/src/cmd/convert.rs` — confirm `ScriptType` + `parse_script_type_arg` are `pub` AND **add `impl ScriptType { pub fn as_str(self) -> &'static str }`** (R0 C2 — it does not exist today).
- **Modify** `docs/manual/src/40-cli-reference/41-mnemonic.md` — `nostr` subcommand chapter.
- **Modify** `crates/mnemonic-toolkit/Cargo.toml` — version `0.34.0` (bech32 already transitive via `bitcoin`; use `bitcoin::bech32` re-export — no new Cargo.toml dep line).
- **Cross-repo (paired)** `mnemonic-gui/src/schema/mnemonic.rs` + `mnemonic-gui/src/secrets.rs` + pin bump.

---

## Phase A0 — Error variant + module skeleton

### Task A0.1: Add `NostrKeyParse` error variant

**Files:**
- Modify: `crates/mnemonic-toolkit/src/error.rs` (enum + `kind()` + `message()` + `exit_code()`)

> **R0 C4:** `impl Display for ToolkitError` (`error.rs:747-751`) delegates to `self.message()` — there is NO per-variant Display match to edit. The compiler-forced exhaustive matches are `kind()` (`error.rs:489-538`, no wildcard), `message()` (`error.rs:543-713`, no wildcard), and `exit_code()` (`error.rs:436`). `details()` (`error.rs:720-743`) has `_ => None` and needs no arm. Add arms to all THREE exhaustive matches + the enum.

- [ ] **Step 1: Add the variant** — insert alphabetically between `NetworkMismatch` (`error.rs:243`) and `Repair` (`error.rs:250`) in `enum ToolkitError`:

```rust
    /// A nostr key (`npub`/`nsec` NIP-19 bech32 or 64-hex) failed to decode or
    /// validate (bad bech32/HRP/length, not-on-curve x-only, out-of-range scalar).
    NostrKeyParse(String),
```

- [ ] **Step 2: Add the `kind()` arm** — in `pub fn kind` (`error.rs:489-538`), alphabetically before the `Repair` arm:

```rust
            ToolkitError::NostrKeyParse(_) => "NostrKeyParse",
```

- [ ] **Step 3: Add the `message()` arm** — in `pub fn message` (`error.rs:543-713`), alphabetically before the `Repair` arm:

```rust
            ToolkitError::NostrKeyParse(msg) => format!("nostr: {msg}"),
```

(Match the surrounding arms' return type — if `message()` returns `String`, use `format!`; if it returns `Cow`/`&str` per-arm, mirror that. The user-facing prefix `nostr:` lives here since `Display` prepends `error: ` then calls `message()`.)

- [ ] **Step 4: Add the `exit_code()` arm** — in `pub fn exit_code` (`error.rs:436`), between `NetworkMismatch` (`error.rs:476`) and `Repair` (`error.rs:477`):

```rust
            ToolkitError::NostrKeyParse(_) => 1,
```

- [ ] **Step 5: Build** — Run: `cargo build -p mnemonic-toolkit`  Expected: compiles cleanly (no non-exhaustive-match errors). If the compiler names another exhaustive match, add a `NostrKeyParse` arm there too.

- [ ] **Step 6: Commit**

```bash
git add crates/mnemonic-toolkit/src/error.rs
git commit -m "feat(nostr): add ToolkitError::NostrKeyParse variant" -m "Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Task A0.2: Create the `nostr` binary-crate module

**Files:**
- Create: `crates/mnemonic-toolkit/src/nostr.rs`
- Modify: `crates/mnemonic-toolkit/src/main.rs` (R0 C1 — binary-crate module, NOT `lib.rs`)

- [ ] **Step 1: Create the module with its doc header**

```rust
//! Nostr-key wrappers — NIP-19 (`npub`/`nsec`) decode, BIP-340 even-y
//! normalization, and Bitcoin address/descriptor/WIF derivation for the
//! `mnemonic nostr` subcommand.
//!
//! A nostr key is a BIP-340 x-only secp256k1 key. Taproot (`p2tr`) is the
//! native mapping — the x-only key IS the taproot internal key, no parity
//! fabrication. Non-taproot (`p2pkh`/`p2wpkh`/`p2sh-p2wpkh`) uses the BIP-340
//! even-y compressed form `02‖x` (mirrors `cost/strip.rs` §11). For `nsec`,
//! the secret is normalized to even-y so the emitted WIF controls the emitted
//! address (see `normalize_to_even_y`).

use crate::error::ToolkitError;
use bitcoin::secp256k1::{Parity, PublicKey, Secp256k1, SecretKey, Signing, Verification, XOnlyPublicKey};
use bitcoin::CompressedPublicKey;
use zeroize::Zeroizing;
```

- [ ] **Step 2: Register the module** — add to `crates/mnemonic-toolkit/src/main.rs` (near the other `mod` lines `mod cmd; mod error; mod network;` at `main.rs:5,12,16`):

```rust
mod nostr;
```

(Binary-crate module — `crate::error::ToolkitError`, `crate::cmd::convert::ScriptType`, `crate::network::CliNetwork` all resolve here; `mlock` is reached via `mnemonic_toolkit::mlock`.)

- [ ] **Step 3: Build** — Run: `cargo build -p mnemonic-toolkit`  Expected: compiles (unused-import warnings OK for now).

- [ ] **Step 4: Commit**

```bash
git add crates/mnemonic-toolkit/src/nostr.rs crates/mnemonic-toolkit/src/main.rs
git commit -m "feat(nostr): scaffold nostr binary-crate module" -m "Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Task A0.3: Add `ScriptType::as_str` (R0 C2)

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/convert.rs`

- [ ] **Step 1: Add the method** — `ScriptType` (`convert.rs:357-362`) has no `impl`; add one (canonical strings that round-trip with `parse_script_type_arg` at `convert.rs:364`):

```rust
impl ScriptType {
    /// Canonical lowercase tag (round-trips with `parse_script_type_arg`).
    pub fn as_str(self) -> &'static str {
        match self {
            ScriptType::P2pkh => "p2pkh",
            ScriptType::P2wpkh => "p2wpkh",
            ScriptType::P2shP2wpkh => "p2sh-p2wpkh",
            ScriptType::P2tr => "p2tr",
        }
    }
}
```

- [ ] **Step 2: Add a round-trip unit test** (in `convert.rs` tests):

```rust
#[test]
fn script_type_as_str_round_trips() {
    for st in [ScriptType::P2pkh, ScriptType::P2wpkh, ScriptType::P2shP2wpkh, ScriptType::P2tr] {
        assert_eq!(parse_script_type_arg(st.as_str()).unwrap(), st);
    }
}
```

(Requires `ScriptType: PartialEq + Copy` — confirm the derive at `convert.rs:357`; add `#[derive(PartialEq)]` if missing, which is non-breaking.)

- [ ] **Step 3: Run** — Run: `cargo test -p mnemonic-toolkit script_type_as_str_round_trips`  Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add crates/mnemonic-toolkit/src/cmd/convert.rs
git commit -m "feat(convert): add ScriptType::as_str (reused by nostr)" -m "Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Phase A — `nostr` crypto/decode library

### Task A1: NIP-19 decode (`decode_npub` / `decode_nsec`)

**Files:**
- Modify: `crates/mnemonic-toolkit/src/nostr.rs`

- [ ] **Step 1: Write failing tests** — append to `nostr.rs`:

```rust
#[cfg(test)]
mod decode_tests {
    use super::*;

    // NIP-19 spec vectors. NOTE: the npub and nsec below are DISTINCT keys
    // (not a keypair); each bech32↔hex row is internally consistent, which is
    // all these decode tests assert. (R0 I3: prior plan used an invalid nsec
    // checksum `…fe9` and falsely labelled the pair "same key".)
    const NPUB: &str = "npub10elfcs4fr0l0r8af98jlmgdh9c8tcxjvz9qkw038js35mp4dma8qzvjptg";
    const PUB_HEX: &str = "7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e";
    const NSEC: &str = "nsec1vl029mgpspedva04g90vltkh6fvh240zqtv9k0t9af8935ke9laqsnlfe5";
    const SEC_HEX: &str = "67dea2ed018072d675f5415ecfaed7d2597555e202d85b3d65ea4e58d2d92ffa";

    #[test]
    fn npub_bech32_decodes_to_expected_xonly() {
        let k = decode_npub(NPUB).unwrap();
        assert_eq!(k.to_string(), PUB_HEX);
    }

    #[test]
    fn npub_hex_decodes_equal_to_bech32() {
        assert_eq!(decode_npub(PUB_HEX).unwrap(), decode_npub(NPUB).unwrap());
    }

    #[test]
    fn nsec_bech32_decodes_to_expected_scalar() {
        let s = decode_nsec(NSEC).unwrap();
        assert_eq!(hex::encode(s.secret_bytes()), SEC_HEX);
    }

    #[test]
    fn wrong_hrp_is_refused() {
        // npub string handed to decode_nsec → HRP mismatch.
        let err = decode_nsec(NPUB).unwrap_err();
        assert!(matches!(err, ToolkitError::NostrKeyParse(_)));
    }

    #[test]
    fn bad_bech32_is_refused() {
        assert!(matches!(decode_npub("npub1notvalid"), Err(ToolkitError::NostrKeyParse(_))));
    }
}
```

- [ ] **Step 2: Run tests, verify they fail**

Run: `cargo test -p mnemonic-toolkit nostr::decode_tests`
Expected: FAIL — `decode_npub`/`decode_nsec` not found.

- [ ] **Step 3: Implement decode** — add to `nostr.rs` (before the test module):

```rust
/// Decode an `npub1…` (NIP-19 bech32) or 64-hex string into an x-only key.
pub fn decode_npub(input: &str) -> Result<XOnlyPublicKey, ToolkitError> {
    let bytes = decode_nostr_key(input, "npub")?;
    XOnlyPublicKey::from_slice(&bytes)
        .map_err(|_| ToolkitError::NostrKeyParse("not a valid secp256k1 x-only public key".into()))
}

/// Decode an `nsec1…` (NIP-19 bech32) or 64-hex string into a secret key.
pub fn decode_nsec(input: &str) -> Result<SecretKey, ToolkitError> {
    let bytes = decode_nostr_key(input, "nsec")?;
    SecretKey::from_slice(&bytes)
        .map_err(|_| ToolkitError::NostrKeyParse("not a valid secp256k1 secret key".into()))
}

/// Shared decode: 64-hex OR NIP-19 bech32 (HRP-checked) → 32 zeroizing bytes.
fn decode_nostr_key(input: &str, expected_hrp: &str) -> Result<Zeroizing<Vec<u8>>, ToolkitError> {
    let trimmed = input.trim();
    if trimmed.len() == 64 && trimmed.bytes().all(|b| b.is_ascii_hexdigit()) {
        let v = hex::decode(trimmed)
            .map_err(|e| ToolkitError::NostrKeyParse(format!("invalid hex key: {e}")))?;
        return Ok(Zeroizing::new(v));
    }
    let (hrp, data) = bitcoin::bech32::decode(trimmed)
        .map_err(|e| ToolkitError::NostrKeyParse(format!("invalid bech32 nostr key: {e}")))?;
    let expected = bitcoin::bech32::Hrp::parse(expected_hrp).expect("static nostr HRP is valid");
    if hrp != expected {
        return Err(ToolkitError::NostrKeyParse(format!(
            "expected an '{expected_hrp}' key but got HRP '{hrp}'"
        )));
    }
    if data.len() != 32 {
        return Err(ToolkitError::NostrKeyParse(format!(
            "{expected_hrp} key must decode to 32 bytes; got {}",
            data.len()
        )));
    }
    Ok(Zeroizing::new(data))
}
```

- [ ] **Step 4: Run tests, verify they pass**

Run: `cargo test -p mnemonic-toolkit nostr::decode_tests`
Expected: PASS (all 5). If `NPUB`/`NSEC`/`PUB_HEX` constants disagree, re-derive from the NIP-19 spec test vectors and update the constants (the bech32↔hex relationship is the invariant under test).

- [ ] **Step 5: Commit**

```bash
git add crates/mnemonic-toolkit/src/nostr.rs
git commit -m "feat(nostr): NIP-19 npub/nsec decode (bech32 + hex, HRP-checked)" -m "Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Task A2: Even-y normalization

**Files:**
- Modify: `crates/mnemonic-toolkit/src/nostr.rs`

- [ ] **Step 1: Write failing test**

```rust
#[cfg(test)]
mod normalize_tests {
    use super::*;

    fn secp() -> Secp256k1<bitcoin::secp256k1::All> { Secp256k1::new() }

    #[test]
    fn normalized_secret_always_has_even_y_pubkey() {
        // Iterate several deterministic scalars; after normalization the pubkey
        // parity must be Even and the x-only must be unchanged.
        for seed in 1u8..=20 {
            let mut bytes = [0u8; 32];
            bytes[31] = seed;
            let sk = SecretKey::from_slice(&bytes).unwrap();
            let (xonly_before, _) = sk.x_only_public_key(&secp());
            let (norm, negated) = normalize_to_even_y(&secp(), sk);
            let (xonly_after, parity_after) = norm.x_only_public_key(&secp());
            assert_eq!(parity_after, Parity::Even, "seed {seed}: not even-y after normalize");
            assert_eq!(xonly_before, xonly_after, "seed {seed}: x-only changed");
            // negated iff the original pubkey was odd-y
            let (_, parity_before) = sk.x_only_public_key(&secp());
            assert_eq!(negated, parity_before == Parity::Odd, "seed {seed}: negate flag wrong");
        }
    }
}
```

- [ ] **Step 2: Run test, verify it fails**

Run: `cargo test -p mnemonic-toolkit nostr::normalize_tests`
Expected: FAIL — `normalize_to_even_y` not found.

- [ ] **Step 3: Implement**

```rust
/// Normalize a secret to BIP-340 even-y form. If `d·G` has odd y, returns
/// `n−d` (so the key matches the even-y `02‖x` address and the taproot
/// internal key); else returns `d` unchanged. Returns `(normalized, negated?)`.
/// The x-only pubkey is parity-independent, so it is unchanged either way.
pub fn normalize_to_even_y<C: Signing>(secp: &Secp256k1<C>, secret: SecretKey) -> (SecretKey, bool) {
    let (_xonly, parity) = secret.x_only_public_key(secp);
    match parity {
        Parity::Odd => (secret.negate(), true),
        Parity::Even => (secret, false),
    }
}
```

- [ ] **Step 4: Run test, verify it passes**

Run: `cargo test -p mnemonic-toolkit nostr::normalize_tests`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/mnemonic-toolkit/src/nostr.rs
git commit -m "feat(nostr): BIP-340 even-y secret normalization" -m "Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Task A3: Address + descriptor + WIF derivation (+ the even-y consistency invariant)

**Files:**
- Modify: `crates/mnemonic-toolkit/src/nostr.rs`
- Confirm `ScriptType` is `pub` in `crates/mnemonic-toolkit/src/cmd/convert.rs:357`

- [ ] **Step 1: Write failing tests** — this is the crux test (WIF↔address consistency) plus a known-vector address check:

```rust
#[cfg(test)]
mod derive_tests {
    use super::*;
    use crate::cmd::convert::ScriptType;
    use crate::network::CliNetwork;
    use std::str::FromStr;

    fn secp() -> Secp256k1<bitcoin::secp256k1::All> { Secp256k1::new() }

    // The CRUX: for every script type, the WIF derived from an nsec must control
    // the address derived from the corresponding npub. Iterate scalars so we hit
    // both even-y and odd-y originals (exercising the negate path).
    #[test]
    fn wif_controls_the_npub_address_all_script_types() {
        for seed in 1u8..=10 {
            let mut bytes = [0u8; 32];
            bytes[31] = seed;
            let sk = SecretKey::from_slice(&bytes).unwrap();
            let (xonly, _) = sk.x_only_public_key(&secp());      // the published npub key
            let (norm, _) = normalize_to_even_y(&secp(), sk);

            for st in [ScriptType::P2pkh, ScriptType::P2wpkh, ScriptType::P2shP2wpkh, ScriptType::P2tr] {
                let addr_from_pub = address_for(&secp(), xonly, st, CliNetwork::Mainnet);
                // Re-derive the address from the normalized secret's pubkey.
                let (xonly_from_secret, _) = norm.x_only_public_key(&secp());
                let addr_from_secret = address_for(&secp(), xonly_from_secret, st, CliNetwork::Mainnet);
                assert_eq!(addr_from_pub, addr_from_secret, "seed {seed} {st:?}: WIF/npub address mismatch");
            }

            // WIF round-trips and is even-y (parity-consistent).
            let wif = wif_for(&norm, CliNetwork::Mainnet);
            let pk = bitcoin::PrivateKey::from_wif(&wif).unwrap();
            let (_, parity) = pk.inner.x_only_public_key(&secp());
            assert_eq!(parity, Parity::Even, "seed {seed}: WIF key not even-y");
        }
    }

    #[test]
    fn descriptor_has_checksum_and_expected_prefix() {
        let xonly = decode_npub("npub10elfcs4fr0l0r8af98jlmgdh9c8tcxjvz9qkw038js35mp4dma8qzvjptg").unwrap();
        let tr = descriptor_for(xonly, ScriptType::P2tr).unwrap();
        assert!(tr.starts_with("tr(") && tr.contains('#'), "got {tr}");
        let wpkh = descriptor_for(xonly, ScriptType::P2wpkh).unwrap();
        assert!(wpkh.starts_with("wpkh(02") || wpkh.starts_with("wpkh(03"), "got {wpkh}");
        // miniscript must accept our own output round-trip (checksum valid).
        assert!(miniscript::Descriptor::<miniscript::DescriptorPublicKey>::from_str(&tr).is_ok());
    }
}
```

- [ ] **Step 2: Run tests, verify they fail**

Run: `cargo test -p mnemonic-toolkit nostr::derive_tests`
Expected: FAIL — `address_for` / `descriptor_for` / `wif_for` not found.

- [ ] **Step 3: Implement** — add to `nostr.rs`:

```rust
use crate::cmd::convert::ScriptType;
use crate::network::CliNetwork;
use bitcoin::Address;
use std::str::FromStr;

/// Even-y compressed pubkey (`02‖x`) from an x-only key.
pub fn even_y_compressed(xonly: XOnlyPublicKey) -> CompressedPublicKey {
    CompressedPublicKey(PublicKey::from_x_only_public_key(xonly, Parity::Even))
}

/// Render the Bitcoin address for an x-only nostr key under `script_type`.
pub fn address_for<C: Verification>(
    secp: &Secp256k1<C>,
    xonly: XOnlyPublicKey,
    script_type: ScriptType,
    network: CliNetwork,
) -> String {
    let compressed = even_y_compressed(xonly);
    match script_type {
        ScriptType::P2pkh => Address::p2pkh(compressed, network.network_kind()).to_string(),
        ScriptType::P2wpkh => Address::p2wpkh(&compressed, network.known_hrp()).to_string(),
        ScriptType::P2shP2wpkh => Address::p2shwpkh(&compressed, network.network_kind()).to_string(),
        ScriptType::P2tr => Address::p2tr(secp, xonly, None, network.known_hrp()).to_string(),
    }
}

/// Build the checksummed Bitcoin descriptor wrapping the nostr key.
pub fn descriptor_for(xonly: XOnlyPublicKey, script_type: ScriptType) -> Result<String, ToolkitError> {
    let body = match script_type {
        ScriptType::P2tr => format!("tr({xonly})"),
        ScriptType::P2wpkh => format!("wpkh({})", even_y_compressed(xonly)),
        ScriptType::P2pkh => format!("pkh({})", even_y_compressed(xonly)),
        ScriptType::P2shP2wpkh => format!("sh(wpkh({}))", even_y_compressed(xonly)),
    };
    let desc = miniscript::Descriptor::<miniscript::DescriptorPublicKey>::from_str(&body)
        .map_err(|e| ToolkitError::NostrKeyParse(format!("descriptor build failed: {e}")))?;
    Ok(desc.to_string()) // Display appends the BIP-380 `#checksum`
}

/// Plain compressed WIF for the (already even-y-normalized) secret.
pub fn wif_for(secret: &SecretKey, network: CliNetwork) -> String {
    bitcoin::PrivateKey { compressed: true, network: network.network_kind(), inner: *secret }.to_wif()
}

/// Electrum imported-key script-type prefix (Electrum `bitcoin.py` SCRIPT_TYPES).
/// NOTE: `p2sh-p2wpkh` maps to Electrum's `p2wpkh-p2sh`. Verify exact strings
/// against Electrum source before release (spec §9 item 1).
pub fn electrum_prefix(script_type: ScriptType) -> &'static str {
    match script_type {
        ScriptType::P2pkh => "p2pkh:",
        ScriptType::P2wpkh => "p2wpkh:",
        ScriptType::P2shP2wpkh => "p2wpkh-p2sh:",
        ScriptType::P2tr => "p2tr:",
    }
}
```

- [ ] **Step 4: Run tests, verify they pass**

Run: `cargo test -p mnemonic-toolkit nostr::derive_tests`
Expected: PASS. If `Address::p2pkh`/`p2wpkh`/`p2shwpkh` arg-type errors appear, mirror the exact call forms in `crates/mnemonic-toolkit/src/cmd/convert.rs:1558-1564` (they compile against the same `CompressedPublicKey`).

- [ ] **Step 5: Add cross-impl fixture test** — pin one full key's four addresses + descriptors against an external oracle (Bitcoin Core `getdescriptorinfo` for checksums; any nostr lib for the npub↔hex). Append:

```rust
#[cfg(test)]
mod cross_impl_fixture {
    use super::*;
    use crate::cmd::convert::ScriptType;
    use crate::network::CliNetwork;

    // Oracle: key 3bf0c6...459d (NIP-19 spec npub). Expected values produced by
    // an independent tool (Bitcoin Core getdescriptorinfo + bdk) and pinned here.
    // Regenerate via tests/external/regen_nostr_vectors.* (added in Task A3.6).
    #[test]
    fn pinned_addresses_for_known_key() {
        let secp = Secp256k1::new();
        let xonly = decode_npub("npub10elfcs4fr0l0r8af98jlmgdh9c8tcxjvz9qkw038js35mp4dma8qzvjptg").unwrap();
        // EXPECTED_* filled in Step 6 from the oracle output (no guessing).
        assert_eq!(address_for(&secp, xonly, ScriptType::P2tr, CliNetwork::Mainnet), EXPECTED_P2TR);
        assert_eq!(address_for(&secp, xonly, ScriptType::P2wpkh, CliNetwork::Mainnet), EXPECTED_P2WPKH);
        assert_eq!(address_for(&secp, xonly, ScriptType::P2pkh, CliNetwork::Mainnet), EXPECTED_P2PKH);
        assert_eq!(address_for(&secp, xonly, ScriptType::P2shP2wpkh, CliNetwork::Mainnet), EXPECTED_P2SH);
    }
    const EXPECTED_P2TR: &str = "<fill from oracle>";
    const EXPECTED_P2WPKH: &str = "<fill from oracle>";
    const EXPECTED_P2PKH: &str = "<fill from oracle>";
    const EXPECTED_P2SH: &str = "<fill from oracle>";
}
```

- [ ] **Step 6: Generate oracle values + write the regen script** — Create `crates/mnemonic-toolkit/tests/external/regen_nostr_vectors.md` documenting the exact oracle commands (e.g. `bitcoin-cli getdescriptorinfo "wpkh(02…)"` for the checksum; the even-y `02‖x` derivation). Run the oracle, paste the four real addresses into the `EXPECTED_*` constants, then re-run `cargo test -p mnemonic-toolkit nostr::cross_impl_fixture` and confirm PASS. (This replaces the `<fill from oracle>` placeholders with verified values — the placeholder must NOT survive this step.)

- [ ] **Step 7: Commit**

```bash
git add crates/mnemonic-toolkit/src/nostr.rs crates/mnemonic-toolkit/tests/external/regen_nostr_vectors.md
git commit -m "feat(nostr): address/descriptor/WIF derivation + even-y consistency + cross-impl fixture" -m "Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Phase B — `mnemonic nostr` CLI

### Task B1: `NostrArgs` + wiring into the Command enum

**Files:**
- Create: `crates/mnemonic-toolkit/src/cmd/nostr.rs`
- Modify: `crates/mnemonic-toolkit/src/cmd/mod.rs`, `crates/mnemonic-toolkit/src/main.rs`

- [ ] **Step 1: Define the args + a stub `run`** — create `cmd/nostr.rs`:

```rust
//! `mnemonic nostr` — wrap an existing nostr key (`npub`/`nsec`) as Bitcoin
//! addresses, descriptors, and (for `nsec`) a WIF. See SPEC
//! `design/BRAINSTORM_v0_34_0_nostr_key_wrappers.md`.

use crate::cmd::convert::ScriptType;
use crate::error::ToolkitError;
use crate::network::CliNetwork;
use clap::Args;
use std::io::{Read, Write};

#[derive(Args, Debug)]
pub struct NostrArgs {
    /// Public key: `npub1…` (NIP-19) or 64-hex x-only. Watch-only outputs.
    #[arg(long, group = "key")]
    pub pubkey: Option<String>,

    /// Secret key: `nsec1…` (NIP-19) or 64-hex scalar. Adds WIF. SECRET — leaks via argv.
    #[arg(long, group = "key")]
    pub secret: Option<String>,

    /// Read the secret key from a file (avoids argv exposure).
    #[arg(long = "secret-file", group = "key")]
    pub secret_file: Option<std::path::PathBuf>,

    /// Read the secret key from stdin.
    #[arg(long = "secret-stdin", group = "key")]
    pub secret_stdin: bool,

    /// Address/descriptor script type. Defaults to `p2tr` when neither this nor
    /// `--all-script-types` is given.
    #[arg(long = "script-type", value_parser = crate::cmd::convert::parse_script_type_arg, conflicts_with = "all_script_types")]
    pub script_type: Option<ScriptType>,

    /// Emit descriptor + address for all four script types.
    #[arg(long = "all-script-types")]
    pub all_script_types: bool,

    /// Bitcoin network (affects address HRP + WIF version byte).
    // R0 I5: `default_value_t` renders via ValueEnum::to_possible_value (compiles;
    // cf. seedqr.rs:71). Do NOT add a `Default`/`#[default]` derive to CliNetwork.
    #[arg(long, value_enum, default_value_t = CliNetwork::Mainnet)]
    pub network: CliNetwork,

    /// Emit JSON instead of the human-readable block.
    #[arg(long)]
    pub json: bool,
}

// R0 I2: by-ref `&NostrArgs` + `Result<u8, ToolkitError>` (mirrors
// cmd/electrum_decrypt.rs:85 + cmd/seedqr.rs:121; dispatch is `match &cli.command`).
pub fn run<R: Read, W: Write, E: Write>(
    _args: &NostrArgs,
    _stdin: &mut R,
    _stdout: &mut W,
    _stderr: &mut E,
) -> Result<u8, ToolkitError> {
    // exactly-one key input is enforced by the struct-level ArgGroup (Task B5).
    todo!("implemented in B2–B5")
}
```

- [ ] **Step 2: Register the module** — add to `crates/mnemonic-toolkit/src/cmd/mod.rs` (alphabetically): `pub mod nostr;`

- [ ] **Step 3: Wire the Command enum + dispatch** — in `crates/mnemonic-toolkit/src/main.rs`, add to `enum Command` (R0 M1: the enum is feature-grouped, NOT alphabetical — place naturally, e.g. after `Seedqr`):

```rust
    /// Wrap an existing nostr key (npub/nsec) as Bitcoin addresses/descriptors/WIF.
    Nostr(cmd::nostr::NostrArgs),
```

and the dispatch arm — `run` returns the exit code directly, so NO `.map(|_| 0)` (mirrors `Command::ElectrumDecrypt`, `main.rs:121`):

```rust
        Command::Nostr(args) => cmd::nostr::run(args, stdin, stdout, stderr),
```

- [ ] **Step 4: Build** — Run: `cargo build -p mnemonic-toolkit`  Expected: compiles (the `todo!()` is fine for build; it would only panic if executed).

- [ ] **Step 5: Commit**

```bash
git add crates/mnemonic-toolkit/src/cmd/nostr.rs crates/mnemonic-toolkit/src/cmd/mod.rs crates/mnemonic-toolkit/src/main.rs
git commit -m "feat(nostr): NostrArgs + Command::Nostr wiring (stub run)" -m "Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Task B2: pubkey path → output (default p2tr)

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/nostr.rs`
- Create: `crates/mnemonic-toolkit/tests/cli_nostr.rs`

- [ ] **Step 1: Write failing integration test** — create `tests/cli_nostr.rs`:

```rust
use assert_cmd::Command; // matches the pattern used by other tests/cli_*.rs
use predicates::prelude::*;

const NPUB: &str = "npub10elfcs4fr0l0r8af98jlmgdh9c8tcxjvz9qkw038js35mp4dma8qzvjptg";

#[test]
fn pubkey_default_p2tr_emits_descriptor_and_address() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--pubkey", NPUB])
        .assert()
        .success()
        .stdout(predicate::str::contains("script-type: p2tr"))
        .stdout(predicate::str::contains("descriptor:  tr("))
        .stdout(predicate::str::contains("address:     bc1p"));
}
```

(Confirm the test harness crate matches the other `tests/cli_*.rs` files — they use `assert_cmd`/`predicates`. If those files use a local helper instead, mirror it.)

- [ ] **Step 2: Run, verify it fails**

Run: `cargo test -p mnemonic-toolkit --test cli_nostr pubkey_default_p2tr`
Expected: FAIL — `run` hits `todo!()` / output absent.

- [ ] **Step 3: Implement the pubkey path** — replace `run`'s body with input resolution + a render helper:

```rust
pub fn run<R: Read, W: Write, E: Write>(
    args: &NostrArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let types: Vec<ScriptType> = if args.all_script_types {
        vec![ScriptType::P2tr, ScriptType::P2wpkh, ScriptType::P2shP2wpkh, ScriptType::P2pkh]
    } else {
        vec![args.script_type.unwrap_or(ScriptType::P2tr)]
    };

    // Resolve the key input (exactly one; secret variants covered in B3/B5).
    let pubkey = args.pubkey.as_deref();
    if let Some(p) = pubkey {
        let xonly = crate::nostr::decode_npub(p)?;
        writeln!(stdout, "nostr key (public)").map_err(ToolkitError::Io)?;
        writeln!(stdout, "  x-only:      {xonly}").map_err(ToolkitError::Io)?;
        for st in &types {
            writeln!(stdout, "  script-type: {}", st.as_str()).map_err(ToolkitError::Io)?;
            writeln!(stdout, "  descriptor:  {}", crate::nostr::descriptor_for(xonly, *st)?).map_err(ToolkitError::Io)?;
            writeln!(stdout, "  address:     {}", crate::nostr::address_for(&secp, xonly, *st, args.network)).map_err(ToolkitError::Io)?;
        }
        return Ok(0);
    }

    let _ = (stdin, stderr); // used by B3/B5
    Err(ToolkitError::NostrKeyParse(
        "exactly one of --pubkey / --secret / --secret-file / --secret-stdin is required".into(),
    ))
}
```

(`ScriptType::as_str()` exists at `convert.rs:67`. Confirm it is `pub`; if not, expose a small `pub fn as_str` or reuse the existing display.)

- [ ] **Step 4: Run, verify it passes**

Run: `cargo test -p mnemonic-toolkit --test cli_nostr pubkey_default_p2tr`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/mnemonic-toolkit/src/cmd/nostr.rs crates/mnemonic-toolkit/tests/cli_nostr.rs
git commit -m "feat(nostr): pubkey path emits descriptor + address (default p2tr)" -m "Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Task B3: secret path → WIF + Electrum hint + even-y advisory

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/nostr.rs`, `crates/mnemonic-toolkit/tests/cli_nostr.rs`

- [ ] **Step 1: Write failing tests**

```rust
const NSEC: &str = "nsec1vl029mgpspedva04g90vltkh6fvh240zqtv9k0t9af8935ke9laqsnlfe5";

#[test]
fn secret_emits_wif_and_electrum_hint() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--secret", NSEC, "--script-type", "p2wpkh"])
        .assert()
        .success()
        .stdout(predicate::str::contains("wif:"))
        .stdout(predicate::str::contains("electrum:    p2wpkh:"));
}

#[test]
fn secret_via_stdin_works() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--secret-stdin", "--script-type", "p2tr"])
        .write_stdin(format!("{NSEC}\n"))
        .assert()
        .success()
        .stdout(predicate::str::contains("wif:"));
}
```

- [ ] **Step 2: Run, verify they fail**

Run: `cargo test -p mnemonic-toolkit --test cli_nostr secret_`
Expected: FAIL — secret path returns the "exactly one…" error.

- [ ] **Step 3: Implement the secret path** — add, before the final `Err(...)` in `run`, a secret resolver + render:

```rust
    // Resolve a secret from --secret / --secret-file / --secret-stdin.
    let secret_input: Option<zeroize::Zeroizing<String>> = if let Some(s) = &args.secret {
        // Inline secret leaked via argv — advise (mirrors electrum-decrypt).
        writeln!(stderr, "warning: nostr: --secret was passed inline and is visible in process args; prefer --secret-file or --secret-stdin").map_err(ToolkitError::Io)?;
        Some(zeroize::Zeroizing::new(s.clone()))
    } else if let Some(path) = &args.secret_file {
        Some(zeroize::Zeroizing::new(std::fs::read_to_string(path).map_err(ToolkitError::Io)?.trim().to_string()))
    } else if args.secret_stdin {
        let mut buf = String::new();
        stdin.read_to_string(&mut buf).map_err(ToolkitError::Io)?;
        Some(zeroize::Zeroizing::new(buf.trim().to_string()))
    } else {
        None
    };

    if let Some(sec) = secret_input {
        let _pin = mnemonic_toolkit::mlock::pin_pages_for(sec.as_bytes()); // R0 I1: lib module via crate name
        let raw = crate::nostr::decode_nsec(&sec)?;
        let (norm, negated) = crate::nostr::normalize_to_even_y(&secp, raw);
        if negated {
            writeln!(stderr, "notice: nostr: secret normalized to even-y (BIP-340) for address consistency").map_err(ToolkitError::Io)?;
        }
        let (xonly, _) = norm.x_only_public_key(&secp);
        writeln!(stdout, "nostr key (secret)").map_err(ToolkitError::Io)?;
        writeln!(stdout, "  x-only:      {xonly}").map_err(ToolkitError::Io)?;
        let wif = crate::nostr::wif_for(&norm, args.network);
        for st in &types {
            writeln!(stdout, "  script-type: {}", st.as_str()).map_err(ToolkitError::Io)?;
            writeln!(stdout, "  descriptor:  {}", crate::nostr::descriptor_for(xonly, *st)?).map_err(ToolkitError::Io)?;
            writeln!(stdout, "  address:     {}", crate::nostr::address_for(&secp, xonly, *st, args.network)).map_err(ToolkitError::Io)?;
            writeln!(stdout, "  electrum:    {}{wif}", crate::nostr::electrum_prefix(*st)).map_err(ToolkitError::Io)?;
        }
        writeln!(stdout, "  wif:         {wif}").map_err(ToolkitError::Io)?;
        return Ok(0);
    }
```

(`crate::mlock::pin_pages_for` is at `src/mlock.rs:90`. If `decode_nsec` should accept `&Zeroizing<String>`, deref to `&str` via `&sec`.)

- [ ] **Step 4: Run, verify they pass**

Run: `cargo test -p mnemonic-toolkit --test cli_nostr secret_`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/mnemonic-toolkit/src/cmd/nostr.rs crates/mnemonic-toolkit/tests/cli_nostr.rs
git commit -m "feat(nostr): secret path — WIF, Electrum hint, even-y advisory, mlock" -m "Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Task B4: `--all-script-types` + `--json`

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/nostr.rs`, `crates/mnemonic-toolkit/tests/cli_nostr.rs`

- [ ] **Step 1: Write failing tests**

```rust
#[test]
fn all_script_types_emits_four() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--pubkey", NPUB, "--all-script-types"])
        .assert().success()
        .stdout(predicate::str::contains("tr("))
        .stdout(predicate::str::contains("wpkh("))
        .stdout(predicate::str::contains("pkh("))
        .stdout(predicate::str::contains("sh(wpkh("));
}

#[test]
fn json_output_is_valid_and_has_fields() {
    let out = Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--pubkey", NPUB, "--json"])
        .assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["kind"], "public");
    assert!(v["outputs"][0]["descriptor"].is_string());
    assert!(v["outputs"][0]["address"].is_string());
}
```

- [ ] **Step 2: Run, verify they fail**

Run: `cargo test -p mnemonic-toolkit --test cli_nostr all_script_types json_output`
Expected: `all_script_types` may already pass (B2 handles `types`); `json_output` FAILS (no JSON branch).

- [ ] **Step 3: Implement `--json`** — add a `serde::Serialize` output struct and branch at the top of `run` after computing `types` and the key. Refactor B2/B3 to build a `Vec<OutputRow { script_type, descriptor, address, electrum }>` + optional `wif`, then either render text (existing) or `serde_json::to_writer_pretty`. Concrete struct:

```rust
#[derive(serde::Serialize)]
struct NostrJson<'a> {
    kind: &'a str,           // "public" | "secret"
    x_only: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    wif: Option<String>,
    outputs: Vec<OutputRow>,
}
#[derive(serde::Serialize)]
struct OutputRow {
    script_type: String,
    descriptor: String,
    address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    electrum: Option<String>,
}
```

Build the `Vec<OutputRow>` once (shared by text + JSON paths — DRY); when `--json`, `serde_json::to_writer_pretty(stdout, &json)?` then a trailing newline.

- [ ] **Step 4: Run, verify they pass**

Run: `cargo test -p mnemonic-toolkit --test cli_nostr all_script_types json_output`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/mnemonic-toolkit/src/cmd/nostr.rs crates/mnemonic-toolkit/tests/cli_nostr.rs
git commit -m "feat(nostr): --all-script-types + --json output" -m "Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Task B5: refusals (no key / multiple keys / bad key / HRP mismatch)

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/nostr.rs`, `crates/mnemonic-toolkit/tests/cli_nostr.rs`

- [ ] **Step 1: Write failing tests**

```rust
#[test]
fn no_key_input_is_refused() {
    Command::cargo_bin("mnemonic").unwrap().args(["nostr"]).assert().failure();
}
#[test]
fn pubkey_and_secret_together_is_refused() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--pubkey", NPUB, "--secret", NSEC])
        .assert().failure(); // clap group="key" rejects >1
}
#[test]
fn nsec_to_pubkey_flag_is_refused() {
    Command::cargo_bin("mnemonic").unwrap()
        .args(["nostr", "--pubkey", NSEC])
        .assert().failure().stderr(predicate::str::contains("HRP"));
}
```

- [ ] **Step 2: Run, verify they fail/behave**

Run: `cargo test -p mnemonic-toolkit --test cli_nostr _refused`
Expected: `pubkey_and_secret_together` likely already passes (clap group). `no_key_input` passes via the final `Err`. `nsec_to_pubkey` passes via `decode_npub` HRP check. Fix any that don't (e.g. add `required = true` semantics via a manual check if clap `group` does not enforce required).

- [ ] **Step 3: Make the group required** — ensure exactly-one: add `#[group(required = true, multiple = false)] struct`-level via `#[command(group(clap::ArgGroup::new("key").required(true).multiple(false)))]` on `NostrArgs`, OR keep the per-arg `group="key"` and add `.required(true)` semantics. Simplest: annotate the struct:

```rust
#[derive(Args, Debug)]
#[command(group(clap::ArgGroup::new("key").required(true).multiple(false).args(["pubkey","secret","secret_file","secret_stdin"])))]
pub struct NostrArgs { /* drop the per-field group="key"; keep fields */ }
```

Then the final `Err(...)` fallback in `run` becomes unreachable for the no-key case (clap handles it) — keep it as a defensive `unreachable!`-style error or remove.

- [ ] **Step 4: Run, verify they pass**

Run: `cargo test -p mnemonic-toolkit --test cli_nostr`
Expected: PASS (all cli_nostr cells).

- [ ] **Step 5: Commit**

```bash
git add crates/mnemonic-toolkit/src/cmd/nostr.rs crates/mnemonic-toolkit/tests/cli_nostr.rs
git commit -m "feat(nostr): required-exactly-one key group + refusal cells" -m "Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Phase C — Lockstep (secret projection, gui-schema, manual, version)

### Task C1: toolkit secret projection

**Files:**
- Modify: `crates/mnemonic-toolkit/src/secrets.rs`

- [ ] **Step 1: Write failing test** — in `secrets.rs` tests, assert `--secret` and `--secret-stdin` under `nostr` are secret:

```rust
#[test]
fn nostr_secret_flags_are_secret() {
    // R0 C3: flag_is_secret(flag_name: &str) — single arg, subcommand-agnostic.
    assert!(flag_is_secret("--secret"));
    assert!(flag_is_secret("--secret-stdin"));
    assert!(!flag_is_secret("--pubkey"));
    assert!(!flag_is_secret("--secret-file")); // path, not the secret itself
}
```

(`secrets.rs:49` `pub fn flag_is_secret(flag_name: &str) -> bool`. `--secret`/`--secret-stdin` become globally secret across subcommands — acceptable, only `nostr` uses these names. If `--secret`/`--secret-file`/`--secret-stdin` ever collide with another subcommand's non-secret flag, revisit.)

- [ ] **Step 2: Run, verify it fails** — Run: `cargo test -p mnemonic-toolkit secrets:: ` Expected: FAIL.

- [ ] **Step 3: Implement** — add `nostr`'s `--secret` / `--secret-stdin` to the secret-flag table in `flag_is_secret` (follow the exact structure used for `electrum-decrypt`'s `--decrypt-password` / `--decrypt-password-stdin`).

- [ ] **Step 4: Run, verify it passes** — Run: `cargo test -p mnemonic-toolkit secrets::` Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/mnemonic-toolkit/src/secrets.rs
git commit -m "feat(nostr): mark --secret/--secret-stdin secret in flag_is_secret" -m "Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Task C2: gui-schema JSON includes `nostr`

**Files:**
- Verify: `crates/mnemonic-toolkit/src/cmd/gui_schema.rs` (auto-walks clap; no edit expected)

- [ ] **Step 1: Run gui-schema + assert the subcommand appears**

Run: `cargo run -p mnemonic-toolkit -- gui-schema | python3 -c "import json,sys; d=json.load(sys.stdin); names=[s['name'] for s in d['subcommands']]; assert 'nostr' in names, names; print('nostr present')"`
Expected: `nostr present`. If absent, gui-schema enumerates a hardcoded list — add `nostr` there (mirror how `seedqr`/`electrum-decrypt` are listed).

- [ ] **Step 2: Confirm flags + secret projection in JSON** — verify `--script-type` / `--network` dropdown values and that `--secret`/`--secret-stdin` carry the secret marker (A.0 recon discipline — check JSON, not just `--help`).

- [ ] **Step 3: Commit (only if a file changed)**

```bash
git add crates/mnemonic-toolkit/src/cmd/gui_schema.rs
git commit -m "feat(nostr): ensure gui-schema JSON emits the nostr subcommand" -m "Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Task C3: manual chapter

**Files:**
- Modify: `docs/manual/src/40-cli-reference/41-mnemonic.md`

- [ ] **Step 1: Add the `nostr` section** — mirror the clap `--help` output exactly: synopsis, every flag (`--pubkey`, `--secret`, `--secret-file`, `--secret-stdin`, `--script-type`, `--all-script-types`, `--network`, `--json`), the even-y normalization note, and the Electrum import recipe. Follow the prose style of the existing `electrum-decrypt` / `seedqr` sections in the same file.

- [ ] **Step 2: Run the manual lint** — Run: `make -C docs/manual lint MNEMONIC_BIN=$(cargo build -p mnemonic-toolkit --message-format=json 2>/dev/null | python3 -c "import json,sys; [print(o['executable']) for l in sys.stdin if (o:=json.loads(l)).get('reason')=='compiler-artifact' and o.get('executable')]" | tail -1) MD_BIN=md MS_BIN=ms MK_BIN=mk`
Expected: PASS — the bidirectional flag-coverage check finds every `nostr` flag documented. (If `MD/MS/MK_BIN` aren't built locally, run only the mnemonic side per `docs/manual/tests/lint.sh` usage.)

- [ ] **Step 3: Commit**

```bash
git add docs/manual/src/40-cli-reference/41-mnemonic.md
git commit -m "docs(manual): nostr subcommand chapter (CLI-reference mirror)" -m "Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Task C4: version bump + CHANGELOG + full regression

**Files:**
- Modify: `crates/mnemonic-toolkit/Cargo.toml` (`version = "0.34.0"`), `CHANGELOG.md` (or the repo's changelog location)

- [ ] **Step 1: Bump version** — `crates/mnemonic-toolkit/Cargo.toml:3` → `version = "0.34.0"`.

- [ ] **Step 2: CHANGELOG entry** — add a `v0.34.0` section: new `mnemonic nostr` subcommand; npub/nsec → addresses/descriptor/WIF; taproot + non-taproot; even-y normalization; no m-format cards; SemVer MINOR.

- [ ] **Step 3: Full regression + clippy**

Run: `cargo test -p mnemonic-toolkit` Expected: PASS (full suite; cell count increased by the new tests).
Run: `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` Expected: clean.

- [ ] **Step 4: Commit**

```bash
git add crates/mnemonic-toolkit/Cargo.toml CHANGELOG.md
git commit -m "release(toolkit): mnemonic-toolkit v0.34.0 — mnemonic nostr key wrappers" -m "Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

### Task C5: paired `mnemonic-gui` schema-mirror (cross-repo)

**Files (in `/scratch/code/shibboleth/mnemonic-gui`):**
- Modify: `src/schema/mnemonic.rs` (new `nostr` `SubcommandSchema`), `src/secrets.rs` (`--secret`/`--secret-stdin` secret), `pinned-upstream.toml` + `Cargo.toml` (toolkit pin → `v0.34.0`)

- [ ] **Step 1: Add the `nostr` SubcommandSchema** — mirror the toolkit clap surface: flags + `--script-type`/`--network` dropdown enums; `secret: true` on `--secret` + `--secret-stdin`, `--secret-file` non-secret.
- [ ] **Step 2: Update `src/secrets.rs`** — add `nostr`'s `--secret`/`--secret-stdin` to the GUI `flag_is_secret` mirror (so `schema_mirror_secret_drift` stays green).
- [ ] **Step 3: Bump the toolkit pin** to `v0.34.0` in `pinned-upstream.toml` + `Cargo.toml`.
- [ ] **Step 4: Run the mirror tests** — Run: `cargo test -p mnemonic-gui schema_mirror` Expected: PASS (flag-name parity + secret projection against the pinned binary).
- [ ] **Step 5: Commit (in the gui repo, on a paired branch)**

```bash
git -C /scratch/code/shibboleth/mnemonic-gui add src/schema/mnemonic.rs src/secrets.rs pinned-upstream.toml Cargo.toml
git -C /scratch/code/shibboleth/mnemonic-gui commit -m "feat(schema): mirror mnemonic nostr subcommand (toolkit v0.34.0)" -m "Co-Authored-By: Claude Opus 4.7 (1M context) <noreply@anthropic.com>"
```

---

## Self-review (writing-plans checklist)

**1. Spec coverage:**
- §1 scope (both directions, npub+nsec, addr+WIF+descriptor, no cards) → A1/A3/B2/B3 ✓
- §2 CLI surface (all flags, default p2tr, ArgGroup, autodetect) → B1/B2/B5 ✓
- §3 crypto (npub decode+validate, even-y, p2tr BIP-86, descriptor checksum, nsec normalize+WIF) → A1/A2/A3 ✓
- §4 output + advisory → B3/B4. **R0 I4:** no stdout TTY-redaction pathway exists in the toolkit (`convert` emits WIF plainly; `convert-minikey-stdout-redaction` is an OPEN follow-up). B3 emits WIF plainly — consistent with current behavior; hygiene rests on the argv advisory + `flag_is_secret`. Spec §4 corrected (no `[SECRET]` marker / redaction claim).
- §5 errors (NostrKeyParse, all cases, exit 1, alphabetical) → A0.1/A1/B5 ✓
- §6 testing (NIP-19 KAT, even-y crux, cross-impl fixture, network, secret hygiene, json) → A1/A2/A3/B3/B4 ✓
- §7 lockstep (SemVer, schema_mirror, secret projection both sides, manual, gui-schema JSON, bech32 transitive) → C1–C5 ✓

**2. Placeholder scan:** The only intentional placeholders are the `EXPECTED_*` address constants in Task A3 Step 5 — **explicitly resolved in Step 6** from a real oracle before that task's commit. No surviving placeholders.

**3. Type consistency:** `ScriptType` (convert.rs), `CliNetwork`, `decode_npub`/`decode_nsec`/`normalize_to_even_y`/`address_for`/`descriptor_for`/`wif_for`/`electrum_prefix`/`even_y_compressed` names are used identically across A3, B2, B3, B4. `NostrArgs` field names (`pubkey`/`secret`/`secret_file`/`secret_stdin`/`script_type`/`all_script_types`/`network`/`json`) match the ArgGroup `args([...])` list in B5.

**Open issues:**
- **O1 — WIF stdout redaction. RESOLVED (R0 I4):** no shared TTY-redaction pathway exists; emit WIF plainly (matching `convert`); spec §4 corrected. No redaction wiring/cell needed.
- **O2 — Electrum prefix strings. OPEN:** `electrum_prefix` (A3 Step 3) uses `p2wpkh-p2sh:` for `p2sh-p2wpkh`; verify all four against Electrum source (`electrum/bitcoin.py` `SCRIPT_TYPES`) before C4. (Does not block compilation; only the hint string's exactness.)
- **O3 — `ScriptType::as_str` / `parse_script_type_arg` visibility. RESOLVED (R0 C2):** `ScriptType::as_str` does NOT exist today — added in Task A0.3; `parse_script_type_arg` is `pub` (`convert.rs:364`, confirmed).

---

## Per-phase reviewer-loop (CLAUDE.md)
After each phase (A0/A, B, C) dispatch the opus `feature-dev:code-reviewer` and persist the review verbatim to `design/agent-reports/v0_34_0-phase-<N>-<round>-review.md` BEFORE folding; loop until 0 critical / 0 important. Run an end-of-cycle opus review before tagging.
