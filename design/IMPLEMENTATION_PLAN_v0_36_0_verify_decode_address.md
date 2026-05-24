# v0.36.0 Implementation Plan — `verify-message` + `decode-address` (+ convert-help freebie + electrum lock-tests)

> **For agentic workers:** per-phase TDD; tests before impl; per-phase opus reviewer-loop until 0C/0I; persist each review to `design/agent-reports/` before the fold. NO parallel code-gen agents. Steps use checkbox (`- [ ]`) syntax.

**Goal:** One MINOR cycle adding two public-data subcommands — `mnemonic verify-message` (legacy `signmessage` + BIP-322 verify-only) and `mnemonic decode-address` (address string → network/type/witness-version/validity/scriptPubKey) — plus a `convert --from` help fix and Electrum spot-check lock-tests.

**Architecture:** Two new binary-private modules each with a core lib (`src/verify_message.rs`, `src/decode_address.rs`) + a CLI wrapper (`src/cmd/{verify_message,decode_address}.rs`), mirroring the `silent_payment` layout (binary-private because they return `crate::error::ToolkitError`, and `error.rs` is NOT in `lib.rs`). Verification is **public** — no secret material, no mlock, no secret-class flags. New dep `bip322` (rust-bitcoin org). Legacy verify reuses the in-tree `bitcoin::sign_message` primitive proven in `wallet_import/bsms_verify.rs`.

**Tech Stack:** Rust; `bitcoin 0.32.8` (`sign_message::{signed_msg_hash, MessageSignature}`, `Address`, `WitnessProgram`); `bip322 = "0.0.10"` (`verify_simple_encoded`, `verify_full_encoded`); `secp256k1 0.29.1`; clap-derive.

**Source SHA:** all citations grep-verified against `origin/master` @ `e128ad4` (2026-05-23).

---

## SemVer + lockstep

- **SemVer MINOR → v0.36.0** (two new top-level subcommands). New dep `bip322 0.0.10`. `Cargo.lock` regen (per `cargo-lock-version-bump-lockstep`).
- **GUI lockstep (MINOR):** add `decode-address` + `verify-message` `SubcommandSchema`s to `mnemonic-gui/src/schema/mnemonic.rs`; pin bump → GUI MINOR. NO secret flags ⇒ no secret-projection delta. Run `schema_mirror` with `MNEMONIC_BIN=<v0.36.0>`.
- **Manual lockstep:** two new chapters under `docs/manual/src/40-cli-reference/41-mnemonic.md`; convert freebie line; **add both subcommands to `docs/manual/tests/cli-subcommands.list`** so flag-coverage actually checks them (the v0.35.0 lesson — that file's omission silently un-wires the chapter check).
- **`cli_gui_schema.rs`** `gui_schema_lists_all_subcommands` is a hardcoded **sorted vec** (`:71-101`), not a count. Insert `"decode-address"` between `"convert"`(:76) and `"derive-child"`(:77); insert `"verify-message"` between `"verify-bundle"`(:92) and `"xpub-search-account-of-descriptor"`(:93) (`verify-b` < `verify-m`). Update the prose count comment (:69) to 25.
- The freebie touches NO clap flag NAME (the `convert --from`/`--to` node list is free-text `node=value`, not a clap `ValueEnum`) ⇒ no `schema_mirror` impact for Phase 1.

---

## Phase 1 — convert `--from entropy` lock-test (+ optional wording enrichment)

> **R0 C1 CORRECTION:** the "freebie" was a FALSE POSITIVE. `convert.rs:175` ALREADY documents the entropy node: `///   entropy          raw entropy hex (secret)`. Root cause: the controller's grep patterns (survey + recon) omitted the literal "entropy", so the existing row was never seen. There is **no missing row to add**. Phase 1 is recharacterized: a regression lock-test (valuable) + an OPTIONAL one-line wording enrichment.

**Files:**
- Modify (OPTIONAL): `crates/mnemonic-toolkit/src/cmd/convert.rs:175` (enrich the existing entropy row's wording)
- Test: `crates/mnemonic-toolkit/tests/cli_convert.rs` (or existing convert test file — grep at impl)
- Modify (only if Step 3 done): `docs/manual/src/40-cli-reference/41-mnemonic.md`

**Context:** `convert.rs:147` + `:862` error strings, `--to` possible-values, AND the `--from` long-help (`:175`) all already name `entropy`; `convert --from entropy=<hex>` already works. So this phase adds regression coverage, not a fix.

- [ ] **Step 1 — regression lock-test** (PASS-on-write — behavior + help both already correct):

```rust
// tests/cli_convert.rs (new test)
#[test]
fn convert_from_entropy_to_phrase_is_supported() {
    // 16-byte all-zero entropy → canonical 12-word "abandon…about".
    let out = run_mnemonic(&["convert", "--from", "entropy=00000000000000000000000000000000", "--to", "phrase"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert!(String::from_utf8_lossy(&out.stdout).contains("abandon abandon abandon"));
}

#[test]
fn convert_help_from_documents_entropy_node() {
    let out = run_mnemonic(&["convert", "--help"]);
    // Loose assert (R0): the existing row already says "entropy" — do NOT
    // assert specific wording so the optional Step-3 enrichment can't break it.
    assert!(String::from_utf8_lossy(&out.stdout).contains("entropy"));
}
```

- [ ] **Step 2** — `cargo test --test cli_convert convert_from_entropy` → both PASS (lock-tests).
- [ ] **Step 3 — (OPTIONAL) wording enrichment.** If desired, refine `convert.rs:175` `raw entropy hex (secret)` → `raw entropy hex, 16/20/24/28/32 bytes (secret)` (matches the byte-length set the parser accepts). Drive with a RED→GREEN test asserting the byte-length phrase IF taken; otherwise SKIP this step. Mirror to manual only if taken.
- [ ] **Step 4 — commit.** `git add crates/mnemonic-toolkit/tests/cli_convert.rs` (+ convert.rs/manual only if Step 3 taken).

---

## Phase 2 — Electrum spot-check lock-tests + honest refusal wording

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/convert.rs:460` (refusal message — *conditional on R0*)
- Test: `crates/mnemonic-toolkit/tests/cli_convert.rs`

**Context:** Spot-check VERIFIED (this session): `electrum-phrase → entropy` matches the committed vectors (SEGWIT `0708661136ef5411cf61f6e07fcfd4efd8`, STANDARD `2738290a29d0c8b7523ac6ea9c63370191`); `electrum-phrase → address` is refused at the valid-edge guard (`convert.rs:640-641` whitelists only `(ElectrumPhrase,Entropy)`/`(Entropy,ElectrumPhrase)`). The refusal currently uses the **shared** one-way-barrier message (`:460`): `"--to {} is cryptographically unrecoverable from --from {} (one-way derivation barrier)"`. For electrum-phrase that wording is imprecise (it's *unimplemented*, not cryptographically unrecoverable).

- [ ] **Step 1 — lock-tests (PASS-on-write; behavior exists).**

```rust
#[test]
fn electrum_phrase_to_entropy_matches_vectors() {
    let segwit = "wild father tree among universe such mobile favorite target dynamic credit identify";
    let out = run_mnemonic_stdin(&["convert", "--from", "electrum-phrase=-", "--to", "entropy"], segwit);
    assert!(String::from_utf8_lossy(&out.stdout).contains("0708661136ef5411cf61f6e07fcfd4efd8"));
}

#[test]
fn electrum_phrase_to_address_is_refused() {
    let segwit = "wild father tree among universe such mobile favorite target dynamic credit identify";
    let out = run_mnemonic_stdin(&["convert", "--from", "electrum-phrase=-", "--to", "address"], segwit);
    assert!(!out.status.success());
    // wording asserted loosely so the Step-3 message refinement doesn't break it
    assert!(String::from_utf8_lossy(&out.stderr).to_lowercase().contains("electrum"));
}
```

- [ ] **Step 2** — run; expect PASS. `cargo test --test cli_convert electrum_`.
- [ ] **Step 3 — (R0 disposition (b)) LEAVE `convert.rs:460` as-is.** The shared barrier message is reused by many edges; the loose lock-test (`contains("electrum")`) passes because the message interpolates `from.as_str()`="electrum-phrase". Honest-wording refinement deferred to FOLLOWUP `electrum-phrase-address-refusal-honest-wording`. Do NOT touch the shared refusal plumbing this cycle.
- [ ] **Step 4 — commit** (folds into Phase 1's convert/test commit).

---

## Phase 3 — `mnemonic decode-address`

**Files:**
- Create: `crates/mnemonic-toolkit/src/decode_address.rs` (core)
- Create: `crates/mnemonic-toolkit/src/cmd/decode_address.rs` (CLI)
- Modify: `crates/mnemonic-toolkit/src/error.rs` (+`DecodeAddress(String)` between `CosignersFile`@89 and `DeriveChildLengthNotApplicable`@94, in def + each match block)
- Modify: `crates/mnemonic-toolkit/src/main.rs` (`mod decode_address;` near `:17`; `Command::DecodeAddress(...)` near `:83`; dispatch near `:135`)
- Modify: `crates/mnemonic-toolkit/src/cmd/mod.rs` (`pub mod decode_address;`)
- Test: `crates/mnemonic-toolkit/tests/cli_decode_address.rs`
- Modify: `crates/mnemonic-toolkit/tests/cli_gui_schema.rs` (add `"decode-address"`; 23→24 here, →25 after Phase 4)
- Modify: `docs/manual/src/40-cli-reference/41-mnemonic.md` + `docs/manual/tests/cli-subcommands.list`

**Core API (`decode_address.rs`):**

```rust
//! Binary-private: public-data address decoder. NO secrets.
use bitcoin::{Address, Network, address::NetworkUnchecked};
use crate::error::ToolkitError;

pub(crate) struct DecodedAddress {
    pub networks: Vec<&'static str>,   // address layer can't disambiguate testnet/testnet4/signet
    pub script_type: String,           // p2pkh|p2sh|p2wpkh|p2wsh|p2tr|p2a|unknown (via AddressType Display)
    pub witness_version: Option<u8>,   // Some(0|1|…) for segwit, None for legacy
    pub script_pubkey_hex: String,
    pub address_normalized: String,
}

pub(crate) fn decode_address(input: &str) -> Result<DecodedAddress, ToolkitError> {
    let unchecked: Address<NetworkUnchecked> = input.trim().parse()
        .map_err(|e| ToolkitError::DecodeAddress(format!("not a valid Bitcoin address: {e}")))?;
    // Determine which networks this address is valid for (prefix/HRP based).
    // R0 M2: tb1 HRP is valid for Testnet|Testnet4|Signet (NOT regtest, which is
    // the distinct bcrt1 HRP). Include testnet4 in the probe set.
    let mut networks = Vec::new();
    for (label, net) in [("mainnet", Network::Bitcoin), ("testnet", Network::Testnet),
                          ("testnet4", Network::Testnet4), ("signet", Network::Signet),
                          ("regtest", Network::Regtest)] {
        if unchecked.is_valid_for_network(net) { networks.push(label); }
    }
    // assume_checked is safe post-validation; pick the first matching net for script derivation.
    let checked = unchecked.clone().assume_checked();
    let spk = checked.script_pubkey();
    // R0 I2: AddressType is #[non_exhaustive] (6 variants incl P2a). Use the
    // crate's Display (yields lowercase "p2pkh"…"p2tr"/"p2a") — forward-compatible,
    // no enumeration, no compile-blocking 5-arm match.
    let script_type: String = checked.address_type().map(|t| t.to_string())
        .unwrap_or_else(|| "unknown".to_string());
    let witness_version = checked.witness_program().map(|wp| wp.version().to_num());
    Ok(DecodedAddress {
        networks, script_type, witness_version,
        script_pubkey_hex: hex::encode(spk.as_bytes()),
        address_normalized: checked.to_string(),
    })
}
```

(`script_type` is now `String`, not `&'static str` — adjust the struct field. `wp.version().to_num() -> u8` confirmed (R0). `is_valid_for_network`/`assume_checked`/`witness_program`/`script_pubkey` all confirmed @ bitcoin 0.32.8.)

**CLI (`cmd/decode_address.rs`):** `DecodeAddressArgs { address: String (positional), json: bool }`; `run<W,E>(...)`. Human output lists the 5 fields; `--json` emits `{address, networks:[…], script_type, witness_version, script_pubkey, valid:true}`.

- [ ] **Step 1 — RED unit tests** (canonical BIP-173/350 vectors):

```rust
#[test]
fn p2wpkh_mainnet_bip173_vector() {
    let d = decode_address("bc1qw508d6qejxtdg4y5r3zarvary0c5xw7kv8f3t4").unwrap();
    assert_eq!(d.script_type, "p2wpkh");
    assert_eq!(d.witness_version, Some(0));
    assert_eq!(d.script_pubkey_hex, "0014751e76e8199196d454941c45d1b3a323f1433bd6");
    assert!(d.networks.contains(&"mainnet"));
}
#[test]
fn p2pkh_mainnet_script_pubkey() {
    let d = decode_address("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2").unwrap();
    assert_eq!(d.script_type, "p2pkh");
    assert_eq!(d.witness_version, None);
    assert!(d.script_pubkey_hex.starts_with("76a914") && d.script_pubkey_hex.ends_with("88ac"));
}
#[test]
fn p2tr_witness_v1() {
    // BIP-350 example P2TR
    let d = decode_address("bc1p0xlxvlhemja6c4dqv22uapctqupfhlxm9h8z3k2e72q4k9hcz7vqzk5jj0").unwrap();
    assert_eq!(d.script_type, "p2tr");
    assert_eq!(d.witness_version, Some(1));
}
#[test]
fn invalid_address_errors() { assert!(decode_address("not-an-address").is_err()); }
#[test]
fn tb1_hrp_valid_for_testnet_testnet4_signet_not_regtest() {
    let d = decode_address("tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx").unwrap();
    // tb1 HRP is valid for testnet + testnet4 + signet; regtest is bcrt1 (distinct).
    assert!(d.networks.contains(&"testnet") && d.networks.contains(&"signet"));
    assert!(d.networks.contains(&"testnet4"));
    assert!(!d.networks.contains(&"regtest"));
}
```

- [ ] **Step 2** — run; expect FAIL (module absent). `cargo test --lib decode_address` (NOTE: binary-private unit tests run under the BIN target, not `--lib` — use `cargo test --bin mnemonic decode_address` or the integration test).
- [ ] **Step 3** — implement `decode_address.rs` core.
- [ ] **Step 4** — add `ToolkitError::DecodeAddress(String)` (def @ alpha pos; `message` arm `format!("decode-address: {msg}")`; `exit_code` ⇒ 1; `kind` ⇒ `"DecodeAddress"`; falls through the `_=>None` detail block — no arm needed, the silent_payment lesson).
- [ ] **Step 5** — implement `cmd/decode_address.rs`; register in `main.rs` + `cmd/mod.rs`.
- [ ] **Step 6 — CLI integration tests** (`tests/cli_decode_address.rs`): human output + `--json` shape + invalid-address non-zero exit.
- [ ] **Step 7** — add `"decode-address"` to `cli_gui_schema.rs` subcommand list + bump count comment.
- [ ] **Step 8** — manual chapter + `cli-subcommands.list` += `mnemonic decode-address`.
- [ ] **Step 9** — `cargo test --bin mnemonic` + `cargo test --test cli_decode_address` → GREEN; `cargo clippy --all-targets`.
- [ ] **Step 10 — per-phase opus review** → persist `design/agent-reports/v0_36_0-phase-3-r0-review.md` → fold → GREEN → commit.

---

## Phase 4 — `mnemonic verify-message` (legacy + BIP-322, verify-only)

> **R0 C2 — address-type partition (load-bearing).** `bitcoin 0.32.8`'s `is_signed_by_address` is **P2PKH-ONLY** (`sign_message.rs:146-161` → `Err(UnsupportedAddressType)` for segwit/wrapped/taproot). The `bip322` crate is the **complement**: it supports P2WPKH/P2SH-P2WPKH/P2TR and **refuses P2PKH** (`verify.rs:67-98`). So the two partition cleanly by address type. Design:
> - **legacy** path → P2PKH only.
> - **bip322** path → P2WPKH / P2SH-P2WPKH / P2TR.
> - **auto** → P2PKH ⇒ legacy; else ⇒ bip322.
> - `--format legacy` on a non-P2PKH address ⇒ honest `VerifyMessage` error ("legacy signmessage verification is P2PKH-only; use --format bip322 or auto for segwit/taproot"), NOT the misleading "bad base64" message.

**Files:**
- Modify: `crates/mnemonic-toolkit/Cargo.toml` (`bip322 = "=0.0.10"` — exact pin per R0 disposition (e); crate name is `bip322`, NOT `bip322-rs`)
- Create: `crates/mnemonic-toolkit/src/verify_message.rs` (core)
- Create: `crates/mnemonic-toolkit/src/cmd/verify_message.rs` (CLI)
- Modify: `error.rs` (+`VerifyMessage(String)` — **R0 I1: between `UnknownHrp`(285-290) and `XpubSearchNoMatch`(291-311)** in the variant def + `exit_code`(500-501) + `kind`(558-559) + `message`(732-743); NO `details` arm — String payload falls through `_=>None`@773)
- Modify: `main.rs` (`mod verify_message;`; `Command::VerifyMessage`; dispatch) + `cmd/mod.rs` (`pub mod verify_message;`)
- Test: `crates/mnemonic-toolkit/tests/cli_verify_message.rs`
- Modify: `cli_gui_schema.rs` (insert `"verify-message"` between `"verify-bundle"`/`"xpub-search-account-of-descriptor"`), manual, `cli-subcommands.list`

**Core API (`verify_message.rs`):**

```rust
//! Binary-private: PUBLIC message-signature verification. NO secrets, NO signing.
//! Address-type partition (R0 C2): legacy=P2PKH only; bip322=P2WPKH/P2SH-P2WPKH/P2TR.
use bitcoin::sign_message::{signed_msg_hash, MessageSignature};
use bitcoin::address::{Address, AddressType, NetworkUnchecked};
use bitcoin::secp256k1::Secp256k1;
use crate::error::ToolkitError;

#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum SigFormat { Legacy, Bip322, Auto }

pub(crate) struct VerifyOutcome {
    pub valid: bool,
    pub format_matched: &'static str,   // "legacy" | "bip322"
}

fn parse_addr(address: &str) -> Result<bitcoin::Address, ToolkitError> {
    let a: Address<NetworkUnchecked> = address.trim().parse()
        .map_err(|e| ToolkitError::VerifyMessage(format!("invalid address: {e}")))?;
    Ok(a.assume_checked())
}

/// Legacy "Bitcoin Signed Message" verify — P2PKH ONLY (bitcoin 0.32.8 limit).
fn verify_legacy(address: &str, message: &str, signature: &str) -> Result<bool, ToolkitError> {
    let addr = parse_addr(address)?;
    if addr.address_type() != Some(AddressType::P2pkh) {
        return Err(ToolkitError::VerifyMessage(
            "legacy signmessage verification is P2PKH-only; use --format bip322 or auto \
             for segwit/taproot addresses".into()));
    }
    let sig = MessageSignature::from_base64(signature).map_err(|e|
        ToolkitError::VerifyMessage(format!("signature is not a valid base64 recoverable signature: {e}")))?;
    let digest = signed_msg_hash(message);
    let secp = Secp256k1::verification_only();
    sig.is_signed_by_address(&secp, &addr, digest)
        .map_err(|e| ToolkitError::VerifyMessage(format!("legacy verify failed: {e}")))
}

/// BIP-322 simple verify — P2WPKH/P2SH-P2WPKH/P2TR (crate refuses P2PKH).
fn verify_bip322(address: &str, message: &str, signature: &str) -> bool {
    bip322::verify_simple_encoded(address.trim(), message, signature).is_ok()
}

pub(crate) fn verify_message(address: &str, message: &str, signature: &str, fmt: SigFormat)
    -> Result<VerifyOutcome, ToolkitError>
{
    let is_p2pkh = parse_addr(address)?.address_type() == Some(AddressType::P2pkh);
    match fmt {
        SigFormat::Legacy => Ok(VerifyOutcome { valid: verify_legacy(address, message, signature)?, format_matched: "legacy" }),
        SigFormat::Bip322 => Ok(VerifyOutcome { valid: verify_bip322(address, message, signature), format_matched: "bip322" }),
        SigFormat::Auto if is_p2pkh => Ok(VerifyOutcome { valid: verify_legacy(address, message, signature)?, format_matched: "legacy" }),
        SigFormat::Auto => Ok(VerifyOutcome { valid: verify_bip322(address, message, signature), format_matched: "bip322" }),
    }
}
```

(`is_signed_by_address(&self, secp, &Address, sha256d::Hash) -> Result<bool,_>` + `signed_msg_hash(&str)->sha256d::Hash` confirmed @ bitcoin 0.32.8 (R0). R0 disposition (c): `--format bip322` uses `verify_simple_encoded` ONLY — do NOT fall back to `verify_full_encoded` (different signature encodings: witness-stack vs full-tx base64); full-format → future `--format bip322-full` FOLLOWUP.)

**CLI (`cmd/verify_message.rs`):** `VerifyMessageArgs { address: String (--address, required), message/message_file/message_stdin (one-of via ArgGroup; message is required content), signature: String (--signature, required), format: VerifyFormat (clap ValueEnum auto|legacy|bip322, default auto), json: bool }`. `run<R,W,E>(args,stdin,stdout,stderr)`. **Exit code (R0 disposition (d)):** malformed/undecodable input (bad address, bad base64, `--format legacy` on non-P2PKH) ⇒ `VerifyMessage` error → exit 1 (stderr). Cleanly-decoded-but-`valid==false` ⇒ exit 1 with the structured `valid:false` result on stdout (human + `--json`), no stderr error. `--json` ⇒ `{address, format_requested, format_matched, valid}`. NO secret flags, NO mlock (verification is public).

**Test oracles:**
- **Legacy (P2PKH):** hard-code a known-good `(P2PKH address, message, base64_sig)` vector (source: `bitcoin` crate `sign_message` tests or an external signer; the address MUST be P2PKH per C2). Assert valid==true; tampered message ⇒ valid==false; a non-P2PKH address with `--format legacy` ⇒ `VerifyMessage` error.
- **BIP-322:** the crate's own / BIP-322 mediawiki vector — P2WPKH `bc1q9vza2e8x573nczrlzms0wvx3gsqjx7vavgkx0l`, messages `""` and `"Hello World"` with their published `verify_simple_encoded` signatures (R0-confirmed = the crate's `SEGWIT_ADDRESS` test). Assert valid==true; wrong-message ⇒ valid==false.
- **auto dispatch:** P2PKH vector resolves via legacy; segwit vector via bip322 (assert `format_matched`).

- [ ] **Step 1** — add `bip322 = "=0.0.10"` to Cargo.toml; `cargo build` (confirms shared `bitcoin 0.32.8`, no duplicate; regen Cargo.lock).
- [ ] **Step 2 — RED unit tests** (legacy valid/invalid + BIP-322 spec vectors valid/invalid + auto-dispatch).
- [ ] **Step 3** — run; expect FAIL (module absent).
- [ ] **Step 4** — implement `verify_message.rs` core (legacy + bip322 + auto).
- [ ] **Step 5** — add `ToolkitError::VerifyMessage(String)` (per R0 placement; `message` ⇒ `format!("verify-message: {msg}")`; exit 1; kind `"VerifyMessage"`).
- [ ] **Step 6** — implement `cmd/verify_message.rs`; register in `main.rs` + `cmd/mod.rs`.
- [ ] **Step 7 — CLI integration tests**: legacy + bip322 valid/invalid; `--format` override; `--message-stdin`; non-zero exit on invalid; `--json` shape.
- [ ] **Step 8** — insert `"verify-message"` between `"verify-bundle"`(:92)/`"xpub-search-account-of-descriptor"`(:93) in `cli_gui_schema.rs` (count comment →25); manual chapter; `cli-subcommands.list` += `mnemonic verify-message`.
- [ ] **Step 9** — full suite + clippy GREEN.
- [ ] **Step 10 — per-phase opus review** → persist `design/agent-reports/v0_36_0-phase-4-r0-review.md` → fold → GREEN → commit.

---

## Phase 5 — GUI lockstep (mnemonic-gui MINOR)

**Files (mnemonic-gui):** `src/schema/mnemonic.rs` (2 SubcommandSchemas + 2 FLAGS consts), `Cargo.toml` (version + toolkit pin), `pinned-upstream.toml`, `Cargo.lock`, `CHANGELOG.md`.

- [ ] **Step 1** — `DECODE_ADDRESS_FLAGS` (`--json` bool, `--no-auto-repair` global; positional `<address>`) + `VERIFY_MESSAGE_FLAGS` (`--address` text, `--message` text, `--message-file` path, `--message-stdin` bool, `--signature` text, `--format` Dropdown(["auto","legacy","bip322"]), `--json` bool, `--no-auto-repair` global). Verify against the toolkit's `gui-schema` JSON (flag-NAME set is what `schema_mirror` gates). **Re-dump `gui-schema` for both subcommands at impl** and match flag names exactly.
- [ ] **Step 2** — 2 `SubcommandSchema` entries; bump `pinned_version` → `"mnemonic 0.36.0"`.
- [ ] **Step 3** — pin `mnemonic-toolkit-v0.35.0 → v0.36.0` (Cargo.toml git-dep + pinned-upstream.toml); GUI version `0.20.0 → 0.21.0`; `cargo update -p mnemonic-toolkit`.
- [ ] **Step 4** — `MNEMONIC_BIN=<v0.36.0 debug binary> cargo test`: `mnemonic_schema_flag_names_match_help_text` + `secret_drift_gate_*` + conditional-drift GREEN; full suite GREEN; clippy clean.
- [ ] **Step 5** — CHANGELOG `[0.21.0]` MINOR entry; commit.

---

## Phase 6 — Release + end-of-cycle gate

- [ ] **Step 1 — toolkit version/lock/install/changelog:** `Cargo.toml` 0.36.0; `Cargo.lock` regen (`cargo update -p mnemonic-toolkit` in-repo); `scripts/install.sh:32` self-pin `mnemonic-toolkit-v0.36.0`; `CHANGELOG.md` `[0.36.0]` MINOR entry.
- [ ] **Step 2 — FOLLOWUPS:** file (per R0 dispositions): `verify-message-bip322-full-format` (`--format bip322-full` via `verify_full_encoded`); `electrum-phrase-address-refusal-honest-wording` (refine `convert.rs:460` for the electrum edge); `electrum-native-seed-address-derivation` (real future feature — Electrum PBKDF2("electrum")+m/0/i derivation, surfaced by the spot-check); optionally `decode-address-scriptpubkey-to-address-reverse`.
- [ ] **Step 3 — manual lint:** `make -C docs/manual lint MNEMONIC_BIN=<v0.36.0> MD_BIN=md MS_BIN=ms MK_BIN=mk` → flag-coverage now checks both new chapters + convert freebie; 0 errors.
- [ ] **Step 4 — end-of-cycle opus review** (whole-cycle diff) → persist `design/agent-reports/v0_36_0-end-of-cycle-review.md` verbatim → fold any Minors → GREEN.
- [ ] **Step 5 — ship toolkit:** merge→master (ff), tag `mnemonic-toolkit-v0.36.0`, push, GH release; verify rust + manual + install-pin-check CI.
- [ ] **Step 6 — ship GUI:** tag `mnemonic-gui-v0.21.0`, push, GH release; verify schema-mirror + build CI.

---

## Self-review (writing-plans checklist)

- **Spec coverage:** Freebie (P1), electrum lock-tests (P2), decode-address (P3), verify-message legacy+BIP-322 (P4), GUI (P5), release (P6). ✓ All four user items covered.
- **Type consistency:** `DecodedAddress` (`script_type: String`), `VerifyOutcome`, `SigFormat` used consistently; `ToolkitError::{DecodeAddress,VerifyMessage}(String)` String-payload (no detail-block arm). ✓
- **R0 dispositions folded (all RESOLVED):** C1 (no phantom freebie — lock-test only); C2 (legacy=P2PKH / bip322=segwit+taproot partition); I1 (`VerifyMessage` between `UnknownHrp`/`XpubSearchNoMatch`); I2 (`AddressType::to_string()`); M1 (exact gui_schema vec positions); M2 (testnet4 + rename); M3 (4-arg run). (a) slot fixed; (b) electrum `:460` left as-is + FOLLOWUP; (c) bip322 simple-only + FOLLOWUP for full; (d) exit-code convention pinned; (e) dep acceptable, exact-pinned `=0.0.10` `bip322`.
