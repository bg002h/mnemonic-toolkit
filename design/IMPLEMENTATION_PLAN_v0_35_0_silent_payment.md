# v0.35.0 `mnemonic silent-payment` (BIP-352 receiver address) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development / executing-plans. Steps use checkbox (`- [ ]`). This plan-doc embeds the approved spec.

**Goal:** New top-level subcommand `mnemonic silent-payment` deriving the BIP-352 **receiver** static address (base + labeled m≥1) from a seed-bearing secret, plus the scan/spend pubkeys, derivation paths, and (advisory-gated) the scan/spend private keys.

**Architecture (R0 C-1):** Mirrors the `mnemonic nostr` cycle — a **binary-private** module `src/silent_payment.rs` (declared `mod silent_payment;` in `main.rs`, **NOT** `pub mod` in `lib.rs`; it uses `crate::error::ToolkitError`, exactly like `nostr.rs`, because `error.rs` is binary-private) for derivation + label tweak + bech32m encode (unit-tested against BIP-352 vectors), + a thin `src/cmd/silent_payment.rs` (arg parse, secret resolution, render). Hand-rolled with existing deps; no new crate.

**Tech Stack:** Rust; `bitcoin = "0.32"` (bip32 + secp256k1 0.29.1 + `bitcoin::bech32` = bech32 0.11.1); `sha2 = "0.10"` (tagged hash). Reuses `CliNetwork`, `secret_advisory`, `mlock`, `derive_slot` seed spine.

**SemVer:** **MINOR — `v0.34.7 → v0.35.0`** (new top-level subcommand). **Mandatory lockstep:** GUI `schema_mirror` (new `SubcommandSchema`) + manual chapter. Out of scope (documented): sender output construction + chain scanning (no tx inputs / chain access / signing — the toolkit boundary).

**Approved design:** user-approved 2026-05-23 — **base + labels (m≥1)**, **emit scan/spend privkeys**, **reject `--label 0`** (reserved change label; change-address emission deferred). Opus architect consult: RECOMMENDATION proceed-with-corrections (crypto verified vs BIP-352 primary source; C1–C5 folded below). Recon: `recon-silent-payments.md`.

**Crypto verified vs BIP-352 (`bitcoin/bips` mediawiki, 2026-05-23) + API verified vs vendored crates:**
- Paths: scan `m/352'/coin'/account'/1'/0`, spend `m/352'/coin'/account'/0'/0` (BIP-32 child scalars, no extra hashing).
- Payload (always 66 B): `ser_P(B_scan) ‖ ser_P(B_m)`; **base: `B_m = B_spend`**; **labeled: `B_m = B_spend + t·G`** where `t = tagged_hash("BIP0352/Label", ser_256(b_scan) ‖ ser_32(m))`, `ser_32(m)` = 4-byte big-endian u32.
- Address = bech32m, HRP `sp` (mainnet) / `tsp` (testnet/signet/regtest), version symbol `q` (Fe32(0)).
- **C1 encoding (the #1 risk):** version symbol must be prepended as a raw Fe32, NOT via `bech32::encode(hrp,&data)` (no version symbol) and NOT `segwit::encode` (enforces the 90-char cap → rejects the ~117-char SP address). BIP-352 lifts the limit to 1023.

---

## Task 1: library `src/silent_payment.rs` (TDD against BIP-352 vectors)

**Files:** new `crates/mnemonic-toolkit/src/silent_payment.rs`; `src/main.rs` (`mod silent_payment;` — **binary-private, NOT lib.rs**; uses `crate::error::ToolkitError` like `nostr.rs`).

### The module API (write these signatures)
```rust
//! BIP-352 Silent Payments — RECEIVER static address derivation.
//! Out of scope (no tx inputs / chain / signing): sender output construction,
//! chain scanning. See `recon-silent-payments.md`.
use bitcoin::bech32::{Bech32m, Fe32, Hrp};
use bitcoin::bech32::primitives::iter::{ByteIterExt, Fe32IterExt};
use bitcoin::secp256k1::{PublicKey, Scalar, Secp256k1, SecretKey, Signing, Verification};

/// HRP per BIP-352: mainnet → "sp", all testnets (testnet/signet/regtest) → "tsp".
pub fn sp_hrp(network: bitcoin::Network) -> Hrp { /* Bitcoin → sp, else tsp; Hrp::parse(...).expect (static) */ }

/// `tagged_hash("BIP0352/Label", ser_256(b_scan) || ser_32(m))` via sha2:
/// SHA256(SHA256(tag) || SHA256(tag) || msg). Returns the 32-byte digest.
fn bip0352_label_hash(b_scan: &SecretKey, m: u32) -> [u8; 32] { /* sha2::Sha256 */ }

/// B_m for label m≥1: B_spend + t·G via `PublicKey::add_exp_tweak`.
/// `t = Scalar::from_be_bytes(label_hash)?` (rejects t ≥ n — propagate as SilentPayment err).
pub fn labeled_spend_key<C: Verification>(secp: &Secp256k1<C>, b_scan: &SecretKey, b_spend_pub: &PublicKey, m: u32) -> Result<PublicKey, ToolkitError> { ... }

/// Encode an sp/tsp address: version `q` (Fe32::Q) + (ser_P(B_scan) || ser_P(B_m)) → bech32m.
/// C1: `core::iter::once(Fe32::Q).chain(payload.iter().copied().bytes_to_fes()).with_checksum::<Bech32m>(&hrp).chars().collect()`.
pub fn encode_sp_address(hrp: Hrp, b_scan_pub: &PublicKey, b_m_pub: &PublicKey) -> String { ... }

/// Derive (b_scan, b_spend) from a master Xpriv at m/352'/coin'/account'/{1',0'}/0.
pub fn derive_scan_spend<C: Signing>(secp: &Secp256k1<C>, master: &bitcoin::bip32::Xpriv, coin_type: u32, account: u32) -> Result<(SecretKey, SecretKey), ToolkitError> { /* DerivationPath::from_str + derive_priv */ }
```
- `ser_P` = `PublicKey::serialize()` (33-byte compressed).
- All `SecretKey`/scalar locals in `Zeroizing` where feasible; `b_scan`/`b_spend` are secret.

- [ ] **Step 1: Vendor the BIP-352 test vectors.** Download `https://raw.githubusercontent.com/bitcoin/bips/master/bip-0352/send_and_receive_test_vectors.json` → `crates/mnemonic-toolkit/tests/fixtures/bip352/send_and_receive_test_vectors.json` (record the source URL + fetch date + git SHA in a header comment in the test file). The "receiving" cases give raw hex `scan_priv_key` + `spend_priv_key` + `labels` (ints) + `expected.addresses` (index 0 = base, then one per label).

- [ ] **Step 2: Write the failing vector test** (`#[cfg(test)] mod tests` in `silent_payment.rs`): for each receiving vector, parse `scan_priv_key`/`spend_priv_key` hex → `SecretKey`s → derive pubkeys → assert `encode_sp_address(base)` == `expected.addresses[0]`, and for each label m in `labels` assert the labeled address matches the corresponding `expected.addresses[i]`. (Inject raw scalars — bypass BIP-32 — since the vectors are key-based, not seed-based.) Run → RED (functions unimplemented).

- [ ] **Step 3: Implement** `sp_hrp`, `bip0352_label_hash`, `labeled_spend_key`, `encode_sp_address`, `derive_scan_spend`. Run vector test → GREEN.

- [ ] **Step 4: Add a seed→path derivation oracle test.** The BIP vectors don't exercise `m/352'`. Add a small independent check: a known (seed, account, network) → expected (b_scan, b_spend) computed by an independent reference (pure-Python or a hardcoded vector derived + cross-checked manually, per the nostr-cycle oracle pattern). Assert `derive_scan_spend` matches. Document the oracle's provenance in the test.

- [ ] **Step 5: main.rs** add `mod silent_payment;` (near `mod nostr;` — binary-private; do NOT add to lib.rs). The `#[cfg(test)] mod tests` runs under the BIN test target (`cargo test -p mnemonic-toolkit silent_payment`, not `--lib`, per the v0.34.3 wallet_import lesson). GREEN; clippy clean.

- [ ] **Step 6: Commit.** `git commit -m "feat(silent-payment): BIP-352 receiver address derivation library (vector-validated)"`

---

## Task 2: error variant + `cmd/silent_payment.rs` + registration

**Files:** `src/error.rs`; new `src/cmd/silent_payment.rs`; `src/cmd/mod.rs`; `src/main.rs`.

- [ ] **Step 1 (C3, R0 I-2): `ToolkitError::SilentPayment(String)`** — add the variant + the per-variant arms **between `RepairShortCircuit` (`error.rs:270`) and `SlotInputViolation` (`error.rs:275`)** ("Silent" < "Slot"; NOT near a `Seedqr*`/`Slip39*` variant — those don't exist, they're library-local errors). Arms: variant def + `message()` (@~717-723, via `format!`) + `exit_code()` (@~492, =1, parse/usage class like nostr) + `kind()` (@~549). **R1 M-b:** `error.rs` ALSO has a 5th per-variant block — the `Option<serde_json::Value>` detail match with a `_ => None` catch-all (@~764-766); `SilentPayment(String)` carries no structured detail, so it correctly falls through `_ => None` and needs **NO** arm there (don't search for a missing one). `Display::fmt` (@~770) just delegates to `message()` — no per-variant `write!`.

- [ ] **Step 2: `SilentPaymentArgs`** (clap) in `cmd/silent_payment.rs`, mirroring `cmd/nostr.rs`:
  - `--secret <STRING>` / `--secret-file <PATH>` / `--secret-stdin` (mutually exclusive ArgGroup) — **seed-bearing** (phrase / ms1 / entropy / master-xprv). Help text: "seed-bearing secret (phrase / ms1 / entropy / master-xprv) — single-key WIF/minikey is refused (cannot derive m/352')".
  - `--network <mainnet|testnet|signet|regtest>` (`CliNetwork`, default mainnet).
  - `--account <u32>` (default 0).
  - `--label <m>` (`Vec<u32>`, repeating) — **m≥1**; `--label 0` → `SilentPayment("m=0 is the reserved BIP-352 change label and must never be published; use m≥1")` (C2).
  - `--json`.

- [ ] **Step 3: `run<R,W,E>(args, stdin, stdout, stderr) -> Result<u8>`** (mirror `cmd::nostr::run`):
  - Resolve the secret → master `Xpriv` via a NEW helper `resolve_master_xpriv(secret: &str, network: CliNetwork) -> Result<Xpriv, ToolkitError>` in `cmd/silent_payment.rs` (R0 I-1 — no existing single helper does this; primitives are scattered). Value-sniff in order:
    1. **xprv/tprv** → `bitcoin::bip32::Xpriv::from_str(s)` (Ok → master directly).
    2. **ms1…** (HRP `ms`) → `ms_codec::decode` → `Payload::Entr` entropy bytes → `bip39::Mnemonic::from_entropy_in(Language::English, &entropy)` → `derive_slot::derive_master_seed(&mnemonic, "")` → `Xpriv::new_master(network.network_kind(), &seed)`.
    3. **BIP-39 phrase** (contains ASCII whitespace / parses via `Mnemonic::parse_in(Language::English, s)`) → `derive_master_seed("")` → `Xpriv::new_master`.
    4. **entropy hex** (16/20/24/28/32 bytes) → `Mnemonic::from_entropy_in` → seed → `Xpriv::new_master`.
    5. else → `ToolkitError::SilentPayment("expected a seed-bearing secret (BIP-39 phrase / ms1 / entropy-hex / xprv); a single private key (WIF/minikey) cannot derive m/352'")` — this refuses WIF/minikey by exclusion with a clear message.
    Empty BIP-39 passphrase in v1 (a `--passphrase` override is a deferred FOLLOWUP). `mlock`-pin + `Zeroizing` the seed/entropy intermediates (mirror `derive_slot`).
  - Inline `--secret` → `secret_advisory::secret_in_argv_warning("--secret", "--secret-stdin")`.
  - `derive_scan_spend` → b_scan/b_spend; pubkeys B_scan/B_spend.
  - Emit (human): base address; one labeled address per `--label m` (m≥1); `B_scan`/`B_spend` (compressed hex) + the two derivation paths. Then the **secret block** (after `secret_advisory::secret_on_stdout_warning_unconditional`): **(C4)** `scan_priv (b_scan) — online/hot key` and `spend_priv (b_spend) — COLD, full spending authority`.
  - `--json`: `{ kind, network, account, address (base), labeled: [{m, address}], scan_pubkey, spend_pubkey, scan_path, spend_path, scan_priv?, spend_priv? }`. Privkeys only in the secret-input path.

- [ ] **Step 4: register** in `cmd/mod.rs` (`pub mod silent_payment;`) + `main.rs`: add `SilentPayment(cmd::silent_payment::SilentPaymentArgs)` to `enum Command` (`main.rs:60-95`, insertion-ordered — place near `Nostr` ~`:82`) + a dispatch arm (`main.rs:113-147`) calling `cmd::silent_payment::run(...)`. (R0 M-3: enum is insertion-ordered, not alphabetical; schema_mirror is order-insensitive.) The v0.34.7 process-hardening hook stays first in `main()`.

- [ ] **Step 5: integration tests** `tests/cli_silent_payment.rs` — base address from a known phrase (mainnet sp1 + testnet tsp1); a labeled address (`--label 1`); `--label 0` refused (exit 1, "reserved change label"); WIF input refused; `--json` shape; secret-on-stdout + argv advisories fire; `--pubkey`-less (no privkeys leaked when... n/a — SP always takes a secret). Reuse a fixture seed whose sp address is cross-checkable.

- [ ] **Step 6: build + full regression + clippy.** `cargo test -p mnemonic-toolkit` green; `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` clean. Commit.

---

## Task 3: secret-taxonomy + manual + version artifacts

- [ ] **Step 1 (C5, R0 M-1): `src/secrets.rs`** — `flag_is_secret` (`secrets.rs:49-64`) already covers `--secret`/`--secret-stdin` globally by flag-name (nostr precedent), so silent-payment's secret flags are covered with NO change. Update the rationale comment ("only nostr uses these names" → "nostr + silent-payment") + the `nostr_secret_flags_are_secret` test note at `secrets.rs:124-129`. Confirm `--secret-file` stays non-secret (path). `secret_taxonomy.rs` (NodeType/SlotSubkey token arrays) + the GUI `secret_taxonomy_pin` projection are unchanged (no new secret VALUE-bearing node type).
- [ ] **Step 2: manual chapter** — `docs/manual/src/40-cli-reference/41-mnemonic.md`: add a `## mnemonic silent-payment` section (synopsis + flag table incl. `--secret*`/`--network`/`--account`/`--label`/`--json`/`--help` + a description of base vs labeled + the m=0 refusal + sender/scanning out-of-scope note). Add to the TOC. Flag-coverage lint REQUIRES every flag documented. Add any new cspell words (e.g. `sp1`, `tsp`, `BIP0352` — verify which trip cspell).
- [ ] **Step 3: version** `Cargo.toml` `0.34.7 → 0.35.0`; `cargo build` regen lock. `scripts/install.sh:32` self-pin `mnemonic-toolkit-v0.34.7 → v0.35.0` (R0 M-2 — known release-CI failure if missed).
- [ ] **Step 4: CHANGELOG `[0.35.0]`** — new `silent-payment` subcommand (BIP-352 receiver address; base + labels m≥1; scan/spend privkeys advisory-gated; sender/scanning out of scope; vector-validated). File FOLLOWUPs: `silent-payment-change-address-m0` (deferred change emission) + `silent-payment-labels-scanning-helper` (any sender/scan helper, v1+).
- [ ] **Step 5: manual lint** `make -C docs/manual lint MNEMONIC_BIN=... MD_BIN=md MS_BIN=ms MK_BIN=mk` → 6/6 (with the new section + flags documented). Commit.

---

## Task 4: end-of-cycle review + ship (GATED)
- [ ] **End-of-cycle opus review → GREEN (0C/0I)**; persist to `design/agent-reports/v0_35_0-end-of-cycle-review.md`.
- [ ] **Ship toolkit** (user go-ahead): merge→master, push, tag `mnemonic-toolkit-v0.35.0`, GH release.

## Task 5: paired GUI schema-mirror lockstep
- [ ] Add the `silent-payment` `SubcommandSchema` to `mnemonic-gui/src/schema/mnemonic.rs` (flags: `--secret` [secret], `--secret-file` [path], `--secret-stdin` [secret], `--network` [Dropdown NETWORKS], `--account` [text/number], `--label` [text, repeating], `--json` [bool], `--no-auto-repair` [global]); bump toolkit pin v0.34.7→v0.35.0 (+ Cargo.toml git-dep) + GUI version 0.19.3→0.20.0 (new subcommand mirrored = MINOR-ish; or PATCH 0.19.4 — match GUI convention); schema gates green vs the v0.35.0 binary; ship paired `mnemonic-gui` release.

---

## R0 MUST-VERIFY (architect-flagged)
1. **bech32 0.11.1 method names** — `Fe32::Q` (=Fe32(0) ✓), `ByteIterExt::bytes_to_fes` ✓, `Fe32IterExt::with_checksum::<Bech32m>` ✓, `Hrp::parse` ✓, the `Encoder::chars()` finalizer (verify the exact finalize call). Import path `bitcoin::bech32::primitives::iter::{ByteIterExt,Fe32IterExt}` (verify re-export depth).
2. **secp256k1 0.29.1** — `Scalar::from_be_bytes([u8;32]) -> Result<_,OutOfRangeError>` ✓ (scalar.rs:68), `PublicKey::add_exp_tweak(mut self, secp, &Scalar) -> Result<PublicKey,_>` ✓ (key.rs:556 — takes **self by value**; call as `b_spend_pub.add_exp_tweak(&secp,&t)` — works via `PublicKey: Copy`; computes `B_spend + t·G`, NOT `t·B_spend`). The `labeled_spend_key` signature may take `b_spend_pub: PublicKey` (by value) to match.
3. **Vector harness** — vectors give raw scan/spend HEX (not seeds); index 0 = base, then per-label in `labels` order. Confirm the JSON field names (`scan_priv_key`/`spend_priv_key`/`labels`/`expected.addresses`) against the downloaded file.
4. **m=0 / labels** — base always emitted + independent of labels; `--label 0` refused.
5. **error-variant alphabetical neighbors** in all 4 `error.rs` match blocks.

## Self-review (writing-plans)
- **Spec coverage:** derivation + base + labeled + privkeys + refusals + JSON + manual + GUI + vectors. ✓
- **No placeholders:** module API, exact crypto, exact bech32/secp/sha2 calls, arg surface, test plan written. ✓
- **Type consistency:** `CliNetwork`, `Xpriv`, `SecretKey/PublicKey/Scalar`, `Fe32/Bech32m/Hrp`, `ToolkitError::SilentPayment`. ✓
- **SemVer/lockstep:** MINOR; GUI schema_mirror + manual mandatory. ✓
- **Risk:** crypto correctness (mitigated by the official-vector oracle + the C1 encoding-path fix + the architect's primary-source verification); the dual-oracle (vectors are key-based → need the separate seed→path check).
