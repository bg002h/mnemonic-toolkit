# v0.35.0 — Plan-doc R0 architect review (opus) — MANDATORY pre-implementation gate

**Date:** 2026-05-23
**Cycle:** v0.35.0 `mnemonic silent-payment` (BIP-352 receiver address)
**Branch:** `v0.35.0-silent-payment`
**Reviewer:** opus (feature-dev:code-reviewer), R0 (round 0)
**Target:** `design/IMPLEMENTATION_PLAN_v0_35_0_silent_payment.md`

---

## Critical

**C-1 — `silent_payment.rs` as `pub mod` in `lib.rs` while returning `ToolkitError` is a hard compile blocker.** Plan `:25,32-52,65`. `error.rs` is binary-private (`mod error;` in `main.rs:12`, deliberately ABSENT from `lib.rs`); library modules each define a local `*Error` (lib.rs:14-52 doc). The nostr precedent the plan cites is actually a BINARY-private module (`mod nostr;` in `main.rs:17`, NOT `pub mod nostr` in lib.rs) — which is why `nostr.rs:13` can use `crate::error::ToolkitError`. A `pub mod silent_payment` in lib.rs cannot name `ToolkitError` → won't compile. **Fix (chosen):** mirror nostr exactly — declare `mod silent_payment;` in `main.rs` (binary-private), keep the `ToolkitError`-returning signatures, drop the `lib.rs pub mod` line; Task 1 Step 5 edits `main.rs`, not `lib.rs`.

## Important

**I-1 — "reuse the seed-resolution path" is unspecified; no such helper exists.** Plan `:51,78,85-86`. No single helper takes an arbitrary secret string → classifies (phrase/ms1/entropy/xprv) → refuses WIF/minikey → returns master `Xpriv`. Primitives exist but scattered: `Xpriv::from_str` (xprv), `ms_codec::decode` (ms1, used in `convert.rs:1404-1418`), `Mnemonic::parse_in`/`from_entropy_in` + `derive_slot::derive_master_seed` (`:32`) + `Xpriv::new_master` (`derive_slot.rs:59`). **Fix:** specify a new `resolve_master_xpriv(secret, network) -> Result<Xpriv, ToolkitError>` value-sniff classifier (xprv-prefix → ms1-prefix → BIP-39-words → entropy-hex → else SilentPayment error covering WIF/minikey), with exact decode steps + empty BIP-39 passphrase (a `--passphrase` follow-on).

**I-2 — error-variant alphabetical-neighbor citation is grep-falsified.** Plan `:75,123` says "after `Seedqr*`/before `Slip39*`" but NO such variants exist in `error.rs` (those subcommands use library-local errors). Actual sorted region: `RepairShortCircuit` (`error.rs:270`) → **`SilentPayment`** → `SlotInputViolation` (`error.rs:275`) ("Sil" < "Slo"). **Fix:** insert between `RepairShortCircuit` and `SlotInputViolation` in the variant def + each of `Display`/`message`/`exit_code`/`kind`.

## Minor
- **M-1** — `flag_is_secret` is in `src/secrets.rs:49-64` (not `secret_taxonomy.rs`); the rationale comment + `nostr_secret_flags_are_secret` test to update are at `secrets.rs:124-129`. C5 substance correct; fix the file path.
- **M-2** — self-pin is `scripts/install.sh:32` (not repo-root `install.sh`).
- **M-3** — `Command` enum (`main.rs:60-95`) + dispatch are insertion-ordered, not alphabetical; specify the insertion point (near `Nostr`). Non-blocking (schema_mirror order-insensitive).
- **M-4** — `add_exp_tweak(mut self, …)` takes self by value (works via `PublicKey: Copy`); computes `B_spend + t·G` (correct). Cosmetic doc note.

---

## Verification summary (CONFIRMED CORRECT — no action)
- **Crypto vs BIP-352 primary source:** scan `m/352'/coin'/account'/1'/0`, spend `.../0'/0`; payload `ser_P(B_scan)‖ser_P(B_m)`, base `B_m=B_spend`, labeled `B_m=B_spend+tagged_hash("BIP0352/Label",ser_256(b_scan)‖ser_32(m))·G` (ser_32 = 4-byte BE); HRP `sp`/`tsp`, version `q`; 1023-char limit. ✓
- **C1 encoding idiom:** `once(Fe32::Q).chain(payload.bytes_to_fes()).with_checksum::<Bech32m>(&hrp).chars().collect()` is the correct bech32 0.11.1 versioned-non-segwit-bech32m path; `Encoder` defaults witness_version None → prepended Fe32::Q is in-checksum data; no length cap on the iterator path; `bitcoin::bech32::primitives::iter::{ByteIterExt,Fe32IterExt}` resolves (`bitcoin` does `pub extern crate bech32`). ✓
- **secp256k1 0.29.1:** `Scalar::from_be_bytes([u8;32]) -> Result<_,OutOfRangeError>` (scalar.rs:68); `PublicKey::add_exp_tweak<C:Verification>` (key.rs:556). ✓
- **Tagged hash via sha2 0.10** (live dep `Cargo.toml:35`): `SHA256(SHA256(tag)‖SHA256(tag)‖msg)` correct. ✓
- **Vectors:** `given.key_material.{scan_priv_key,spend_priv_key}` (hex), `given.labels` (ints), `expected.addresses` (idx 0 = base, then per-label). ✓
- **Toolkit fit:** `cmd::nostr::run<R,W,E>(...) -> Result<u8>` (`nostr.rs:136`); `secret_advisory::{secret_in_argv_warning,secret_on_stdout_warning_unconditional}`; `derive_slot.rs:59-66` spine. ✓
- **Version/SemVer:** 0.34.7→0.35.0 MINOR (new subcommand); GUI schema_mirror + manual mandatory. ✓
- **m=0 refusal (C2):** BIP-352 "We reserve m=0 … the wallet never hands out m=0." ✓

VERDICT: RED (1C/2I) — crypto + APIs + vectors verified correct; fold C-1 (binary-private module), I-1 (specify resolver), I-2 (error neighbors) + M-1..4, re-grep, re-dispatch R1.

---

## Fold disposition (controller) — round 0 → R1
Folding: C-1 (binary-private `mod silent_payment;` in main.rs, keep ToolkitError sigs); I-1 (specify `resolve_master_xpriv` value-sniff classifier with exact decode steps + empty-passphrase + `--passphrase` follow-on note); I-2 (neighbors `RepairShortCircuit`/`SlotInputViolation`); M-1 (secrets.rs path); M-2 (scripts/install.sh:32); M-3 (Command insertion near Nostr); M-4 (add_exp_tweak self-by-value note). Re-dispatching R1.

---

## R1 (round 1) — VERDICT: GREEN (0C/0I)
All folds VERIFIED against live source: **C-1** (`mod nostr;` is binary-private @`main.rs:17`, uses `crate::error::ToolkitError`, absent from lib.rs — plan now mirrors with `mod silent_payment;`); **I-1** (all `resolve_master_xpriv` primitives exist with cited sigs: `Xpriv::from_str`, `ms_codec::decode`→`Payload::Entr` @`convert.rs:1405-1412`, `Mnemonic::{parse_in,from_entropy_in}`, `derive_slot::derive_master_seed`@:32, `Xpriv::new_master`@:59; **sniff order proven false-positive-free** — xprv base58check-prefix, ms1 bech32-HRP, phrase checksum-validated before contiguous-hex, structurally disjoint); **I-2** (`SilentPayment` between `RepairShortCircuit`@:270 and `SlotInputViolation`@:275 in all blocks: variant def, exit_code @492-3, kind @549-50, message @717-23; Seedqr*/Slip39* confirmed nonexistent); **M-1** (`secrets.rs:49-64`), **M-2** (`scripts/install.sh:32`), **M-3** (Command insertion-ordered near Nostr), **M-4** (add_exp_tweak self-by-value). No new drift; crypto/C1/vectors unchanged.
Two new doc Minors: **M-a** dispatch cite `113-145`→`113-147`; **M-b** error.rs has a 5th per-variant block (`Option<serde_json::Value>` detail, `_=>None` @764-766) — `SilentPayment(String)` correctly falls through (no arm needed; note so the implementer doesn't search). Both FOLDED.
**0C/0I gate satisfied — implementation may proceed.**
