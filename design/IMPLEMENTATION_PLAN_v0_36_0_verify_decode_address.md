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
- **`cli_gui_schema.rs`** `gui_schema_lists_all_subcommands`: 23 → **25** (add `decode-address`, `verify-message`).
- The freebie touches NO clap flag NAME (the `convert --from`/`--to` node list is free-text `node=value`, not a clap `ValueEnum`) ⇒ no `schema_mirror` impact for Phase 1.

---

## Phase 1 — Freebie: `convert --from` help lists `entropy`

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/convert.rs:171-188` (the `--from` long-help node enum)
- Test: `crates/mnemonic-toolkit/tests/cli_convert.rs` (or the existing convert test file — grep at impl)
- Modify: `docs/manual/src/40-cli-reference/41-mnemonic.md` (convert `--from` node list)

**Context:** `convert.rs:147` and `:862` error strings already enumerate `entropy`; `--to` possible-values already includes it; `convert --from entropy=<hex>` already works. Only the `--from` **long-help doc-comment** (`:171-185`) omits the `entropy` row. Pure doc + lock-test.

- [ ] **Step 1 — RED: regression test** the entropy edge is supported.

```rust
// tests/cli_convert.rs (new test)
#[test]
fn convert_from_entropy_to_phrase_is_supported() {
    // 16-byte all-zero entropy → canonical 12-word "abandon…about".
    let out = run_mnemonic(&["convert", "--from", "entropy=00000000000000000000000000000000", "--to", "phrase"]);
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert!(String::from_utf8_lossy(&out.stdout).contains("abandon abandon abandon"));
}
```

- [ ] **Step 2** — run it; expect PASS (behavior already exists — this is a lock-test, not a driver). Run: `cargo test --test cli_convert convert_from_entropy_to_phrase_is_supported`.

- [ ] **Step 3 — fix the help doc-comment.** In the `///` block at `convert.rs:172-185`, insert after the `seedqr` row (line 173-176) and before `xprv` (line 177):

```rust
    ///   entropy          raw BIP-39 entropy as hex (16/20/24/28/32 bytes; secret)
```

- [ ] **Step 4 — assert the help text now lists entropy.**

```rust
#[test]
fn convert_help_from_lists_entropy_node() {
    let out = run_mnemonic(&["convert", "--help"]);
    let help = String::from_utf8_lossy(&out.stdout);
    // The `<node> is one of:` enum must name entropy.
    assert!(help.contains("entropy") && help.contains("raw BIP-39 entropy"));
}
```

- [ ] **Step 5** — `cargo test --test cli_convert` → both PASS.
- [ ] **Step 6 — manual mirror.** Add the `entropy` node row to the convert `--from` enumeration in `41-mnemonic.md`.
- [ ] **Step 7 — commit.** `git add crates/mnemonic-toolkit/src/cmd/convert.rs crates/mnemonic-toolkit/tests/cli_convert.rs docs/manual/src/40-cli-reference/41-mnemonic.md`

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
- [ ] **Step 3 — (R0-gated) honest wording.** *Only if R0 approves touching the shared message.* Options for R0: (a) add a dedicated `(ElectrumPhrase, _)` refusal arm with an honest message before the shared barrier; (b) leave `:460` as-is (the loose test in Step 1 passes either way). **Default: defer to R0 disposition; do NOT widen scope unilaterally.**
- [ ] **Step 4 — commit** (folds into Phase 1's convert commit if same files, else separate).

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
    pub networks: Vec<&'static str>,   // address layer can't disambiguate testnet/signet/regtest
    pub script_type: &'static str,     // p2pkh|p2sh|p2wpkh|p2wsh|p2tr|unknown
    pub witness_version: Option<u8>,   // Some(0|1|…) for segwit, None for legacy
    pub script_pubkey_hex: String,
    pub address_normalized: String,
}

pub(crate) fn decode_address(input: &str) -> Result<DecodedAddress, ToolkitError> {
    let unchecked: Address<NetworkUnchecked> = input.trim().parse()
        .map_err(|e| ToolkitError::DecodeAddress(format!("not a valid Bitcoin address: {e}")))?;
    // Determine which networks this address is valid for (prefix/HRP based).
    let mut networks = Vec::new();
    for (label, net) in [("mainnet", Network::Bitcoin), ("testnet", Network::Testnet),
                          ("signet", Network::Signet), ("regtest", Network::Regtest)] {
        if unchecked.is_valid_for_network(net) { networks.push(label); }
    }
    // assume_checked is safe post-validation; pick the first matching net for script derivation.
    let checked = unchecked.clone().assume_checked();
    let spk = checked.script_pubkey();
    let script_type = match checked.address_type() {
        Some(t) => address_type_str(t),   // map AddressType → &'static str
        None => "unknown",
    };
    let witness_version = checked.witness_program().map(|wp| wp.version().to_num());
    Ok(DecodedAddress {
        networks, script_type, witness_version,
        script_pubkey_hex: hex::encode(spk.as_bytes()),
        address_normalized: checked.to_string(),
    })
}
```

(`address_type_str` maps `AddressType::{P2pkh,P2sh,P2wpkh,P2wsh,P2tr}` → the lowercase strings; confirm exact `AddressType` variants + `witness_program()`/`version().to_num()` API at impl against `bitcoin 0.32.8`.)

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
fn testnet_signet_regtest_ambiguity_reported_as_set() {
    let d = decode_address("tb1qw508d6qejxtdg4y5r3zarvary0c5xw7kxpjzsx").unwrap();
    // tb1 HRP is testnet AND signet
    assert!(d.networks.contains(&"testnet") && d.networks.contains(&"signet"));
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

**Files:**
- Modify: `crates/mnemonic-toolkit/Cargo.toml` (`bip322 = "0.0.10"`)
- Create: `crates/mnemonic-toolkit/src/verify_message.rs` (core)
- Create: `crates/mnemonic-toolkit/src/cmd/verify_message.rs` (CLI)
- Modify: `error.rs` (+`VerifyMessage(String)` — alpha placement in the legacy-unsorted V-region; **R0 to pin exact spot** — candidate: between `Unset`@320 and `XpubParse`@336)
- Modify: `main.rs` + `cmd/mod.rs`
- Test: `crates/mnemonic-toolkit/tests/cli_verify_message.rs`
- Modify: `cli_gui_schema.rs` (add `"verify-message"`; →25), manual, `cli-subcommands.list`

**Core API (`verify_message.rs`):**

```rust
//! Binary-private: PUBLIC message-signature verification. NO secrets, NO signing.
use bitcoin::sign_message::{signed_msg_hash, MessageSignature};
use bitcoin::{Address, address::NetworkUnchecked};
use bitcoin::secp256k1::Secp256k1;
use crate::error::ToolkitError;

#[derive(PartialEq, Debug)]
pub(crate) enum SigFormat { Legacy, Bip322, Auto }

pub(crate) struct VerifyOutcome {
    pub valid: bool,
    pub format_matched: &'static str,   // "legacy" | "bip322" | "none"
}

pub(crate) fn verify_message(
    address: &str, message: &str, signature: &str, fmt: SigFormat,
) -> Result<VerifyOutcome, ToolkitError> {
    let try_legacy = |.| -> Option<bool> {
        let sig = MessageSignature::from_base64(signature).ok()?;   // only 65-byte recoverable decodes
        let addr: Address<NetworkUnchecked> = address.parse().ok()?;
        let addr = addr.assume_checked();
        let digest = signed_msg_hash(message);
        let secp = Secp256k1::verification_only();
        sig.is_signed_by_address(&secp, &addr, digest).ok()
    };
    let try_bip322 = |.| -> bool {
        bip322::verify_simple_encoded(address, message, signature).is_ok()
    };
    match fmt {
        SigFormat::Legacy => Ok(mk(try_legacy().ok_or_else(|| ToolkitError::VerifyMessage(
            "signature is not a valid legacy (65-byte recoverable) base64 signature".into()))?, "legacy")),
        SigFormat::Bip322 => Ok(mk(try_bip322(), "bip322")),
        SigFormat::Auto => {
            if let Some(v) = try_legacy() { return Ok(mk(v, "legacy")); }
            Ok(mk(try_bip322(), "bip322"))
        }
    }
}
```

(Pseudocode `|.|` = closures; finalize at impl. `is_signed_by_address(secp, address, msg_hash) -> Result<bool>` confirmed in `bitcoin 0.32.8`. `mk(valid, fmt)` builds `VerifyOutcome`. Decide at impl whether `--format bip322` should also try `verify_full_encoded` as a fallback to `verify_simple_encoded`.)

**CLI (`cmd/verify_message.rs`):** `VerifyMessageArgs { address: String, message: Option<String>, message_file: Option<PathBuf>, message_stdin: bool (ArgGroup, default ""? — decide: message is REQUIRED, one-of the three), signature: String, format: VerifyFormat (clap ValueEnum: auto|legacy|bip322, default auto), json: bool }`. `run<R,W,E>`. **Exit code:** invalid signature ⇒ non-zero (precedent: a verify tool signals failure via exit; emit structured result first). `--json` ⇒ `{address, format_matched, valid, format_requested}`. NO secret flags.

**Test oracles:**
- **Legacy:** source a known-good vector from the `bitcoin` crate `sign_message` tests OR generate one externally and hard-code `(address, message, base64_sig)`; assert valid==true, and a tampered message ⇒ valid==false.
- **BIP-322:** the **BIP-322 spec test vectors** — P2WPKH address `bc1q9vza2e8x573nczrlzms0wvx3gsqjx7vavgkx0l`, message `""` and `"Hello World"` with their published `verify_simple_encoded` signatures (cross-checked against the `bip322` crate's own tests); assert valid==true; wrong-message ⇒ valid==false. Add a P2TR BIP-322 vector if the spec/crate provides one.

- [ ] **Step 1** — add `bip322 = "0.0.10"` to Cargo.toml; `cargo build` (confirms shared `bitcoin 0.32.8`, no duplicate; regen Cargo.lock).
- [ ] **Step 2 — RED unit tests** (legacy valid/invalid + BIP-322 spec vectors valid/invalid + auto-dispatch).
- [ ] **Step 3** — run; expect FAIL (module absent).
- [ ] **Step 4** — implement `verify_message.rs` core (legacy + bip322 + auto).
- [ ] **Step 5** — add `ToolkitError::VerifyMessage(String)` (per R0 placement; `message` ⇒ `format!("verify-message: {msg}")`; exit 1; kind `"VerifyMessage"`).
- [ ] **Step 6** — implement `cmd/verify_message.rs`; register in `main.rs` + `cmd/mod.rs`.
- [ ] **Step 7 — CLI integration tests**: legacy + bip322 valid/invalid; `--format` override; `--message-stdin`; non-zero exit on invalid; `--json` shape.
- [ ] **Step 8** — add `"verify-message"` to `cli_gui_schema.rs` (→25); manual chapter; `cli-subcommands.list` += `mnemonic verify-message`.
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
- [ ] **Step 2 — FOLLOWUPS:** close/file as needed (e.g., file `verify-message-bip322-full-format-coverage` if `verify_full_encoded` deferred; `decode-address-scriptpubkey-to-address-reverse` if reverse deferred; note `electrum-native-seed-address-derivation` as a real future feature surfaced by the spot-check).
- [ ] **Step 3 — manual lint:** `make -C docs/manual lint MNEMONIC_BIN=<v0.36.0> MD_BIN=md MS_BIN=ms MK_BIN=mk` → flag-coverage now checks both new chapters + convert freebie; 0 errors.
- [ ] **Step 4 — end-of-cycle opus review** (whole-cycle diff) → persist `design/agent-reports/v0_36_0-end-of-cycle-review.md` verbatim → fold any Minors → GREEN.
- [ ] **Step 5 — ship toolkit:** merge→master (ff), tag `mnemonic-toolkit-v0.36.0`, push, GH release; verify rust + manual + install-pin-check CI.
- [ ] **Step 6 — ship GUI:** tag `mnemonic-gui-v0.21.0`, push, GH release; verify schema-mirror + build CI.

---

## Self-review (writing-plans checklist)

- **Spec coverage:** Freebie (P1), electrum lock-tests (P2), decode-address (P3), verify-message legacy+BIP-322 (P4), GUI (P5), release (P6). ✓ All four user items covered.
- **Type consistency:** `DecodedAddress`, `VerifyOutcome`, `SigFormat` used consistently; `ToolkitError::{DecodeAddress,VerifyMessage}(String)` String-payload (no detail-block arm). ✓
- **Open R0 questions (flagged, not placeholders):** (a) exact alphabetical slot for `VerifyMessage` in the legacy-unsorted tail; (b) whether to refine the electrum refusal message (`:460`) or leave it; (c) `--format bip322` simple-vs-full fallback; (d) verify-message invalid-signature exit-code convention (non-zero vs always-0-with-json); (e) `bip322 0.0.x` dep acceptability (rust-bitcoin org, verify-only). These are design judgments for the architect, not unspecified impl details.
