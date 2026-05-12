# v0.8.1 Phase 1 review — r1

Date: 2026-05-12
Reviewer: opus-architect (r1) via general-purpose agent
Branch: export-wallet-v0.8.1-recover
Phase 1 delta: db08a21..43f64b8 (9 commits)

## Summary

- Byte-exact fixtures: 0C / 0I / 1L / 0N
- Refusal pointer texts: 1C / 1I / 0L / 0N
- SPEC drift: 0C / 1I / 0L / 0N
- collect_missing wiring: 0C / 1I / 0L / 0N
- Manual mirror: 0C / 0I / 1L / 0N
- FOLLOWUPS schema: 0C / 0I / 0L / 0N
- --wallet-name truncation: 0C / 0I / 0L / 1N
- Test consistency: 0C / 1I / 0L / 0N
- Cross-cutting: 0C / 0I / 1L / 0N

Total: 1C / 4I / 3L / 1N

---

## Findings — Refusal pointer texts

### C-1 — Jade singlesig refusal stderr emits doubled `error:` prefix

**Location:** `crates/mnemonic-toolkit/src/wallet_export/jade.rs:50-52`
**Evidence:** Runtime stderr from `mnemonic export-wallet --format jade --template bip84 ...` is literally:
```
error: error: mnemonic export-wallet --format jade emits multisig wallet config only; for singlesig setups Jade reads the seed on-device. Use --format coldcard for a singlesig JSON or --format bitcoin-core for a descriptor.
```
The emitter source builds the string `"error: mnemonic export-wallet ..."`, then `ToolkitError::Display` (`error.rs:410`) prepends another `error: `. SPEC §6 pins the byte-exact stderr as a SINGLE `error:` prefix. The Coldcard bip86 refusal at `coldcard.rs:109-111` correctly omits the `error:` prefix in the source string (matches the convention that Display adds the prefix). Jade is the outlier.

**Why the test mask:** `cli_export_wallet_jade.rs:166-170` uses `stderr.contains("error: mnemonic ...")` — a substring check is satisfied by `error: error: mnemonic ...`.

**Fix:** Strip the leading `"error: "` from the `jade.rs` string literal so Display owns the prefix. Tighten the test to byte-exact `assert_eq!(stderr.trim_end(), "error: mnemonic export-wallet --format jade emits multisig wallet config only; ...")`.

### I-1 — `build_missing_fields_refusal` will produce the same doubled `error:` prefix once wired

**Location:** `crates/mnemonic-toolkit/src/wallet_export/mod.rs:271-284` (constructor) + `crates/mnemonic-toolkit/src/error.rs:350-352` (router) + `error.rs:410` (Display formatter)
**Evidence:** `build_missing_fields_refusal` produces `"error: mnemonic export-wallet --format {format} requires the following missing fields:\n  - ..."`. `ToolkitError::ExportWalletMissingFields`'s `user_text()` returns this verbatim, and Display prepends another `error: `. SPEC §4 pins a SINGLE `error:`. Currently latent — `collect_missing` is placeholder, so no caller materializes `ExportWalletMissingFields`. Will manifest as a SPEC violation as soon as Phase 1 step 3 fixtures get pinned against the SPEC text.

**Fix:** Strip the leading `"error: "` from the `build_missing_fields_refusal` source string so Display owns the prefix.

---

## Findings — SPEC drift

### I-2 — `master_xpub` SPEC §5.1 conditional emission is unwired through the resolution pipeline

**Location:** `crates/mnemonic-toolkit/src/wallet_export/coldcard.rs:200-204` + `crates/mnemonic-toolkit/src/wallet_export/mod.rs:315-342` (`EmitInputs`) + `crates/mnemonic-toolkit/src/synthesize.rs::ResolvedSlot`
**Evidence:** SPEC §2 and §5.1 ship two normative claims:
- (a) the slot grammar accepts `@N.master_xpub=<base58>`,
- (b) the top-level `xpub` field is emitted iff `@0.master_xpub=` was supplied.

(a) is shipped: `slot_input.rs` parses `master_xpub`, defines `MasterXpub`, `is_legal_set` accepts all legal extensions. But (b) is unwired: `ResolvedSlot` doesn't carry the field, `EmitInputs` doesn't carry it, `coldcard.rs:204` hard-codes `xpub: None`. Today supplying `--slot @0.master_xpub=xpub6...` is silently dropped — the user's stated intent is ignored. The SPEC amendment (commit 284f349) shipped ahead of the implementation that realizes it.

**Fix:** Either (a) plumb `master_xpub: Option<bitcoin::bip32::Xpub>` through `synthesize::resolve_slots` → `ResolvedSlot` → `EmitInputs` and replace the `xpub: None` literal with the resolved value, OR (b) file a FOLLOWUPS entry (suggested slug: `coldcard-master-xpub-plumbing-pending`, tier `v0.8.2`) AND emit a refusal when `master_xpub` is supplied for an emitter that drops it. The current state — grammar accepts, implementation drops, no error, no FOLLOWUPS — is the worst of three outcomes.

---

## Findings — collect_missing wiring

### I-3 — `collect_missing` returns `Vec::new()` in both emitters; IMPL_PLAN Phase 1 step 3 fixture list unfulfilled

**Location:** `crates/mnemonic-toolkit/src/wallet_export/coldcard.rs:24-30` + `crates/mnemonic-toolkit/src/wallet_export/jade.rs:24-26` + `design/IMPLEMENTATION_PLAN_v0_8_export_wallet_expansion.md:59`
**Evidence:** Both impls return `Vec::new()` with a "Phase 1.2 placeholder" comment. IMPL_PLAN Phase 1 step 3 enumerates six refusal fixtures (`coldcard_missing_xfp_refusal.stderr`, `coldcard_multisig_template_skeleton_mismatch_refusal.stderr`, `coldcard_bip86_pending_firmware_refusal.stderr`, `jade_singlesig_refusal.stderr`, `jade_tr_multi_a_refusal.stderr`, `multi_missing_fields_aggregate_refusal.stderr`). Zero `.stderr` files exist under `crates/mnemonic-toolkit/tests/export_wallet/`. Tests use `.contains()` substring checks. The SPEC §4 deterministic-order contract — pinned twice (R1-N3 fold, R2-L4 reword) — is unexercised end-to-end.

**Fix:** Wire `ColdcardEmitter::collect_missing` to populate `MissingField::MasterFingerprint { slot }` etc. when slots lack required inputs. Wire `IncompatibleFormatForTemplate` for singlesig-path-receives-multisig-template (and vice versa). Then pin the six `.stderr` fixtures.

---

## Findings — Byte-exact fixtures

### L-1 — Field order in Coldcard JSON fixtures diverges from upstream's alphabetical order, but matches the SPEC example

**Location:** `crates/mnemonic-toolkit/tests/export_wallet/coldcard_generic_bip84_mainnet.json:5-12` (and the bip49 / bip44 fixtures) + `design/SPEC_export_wallet_v0_8.md:95-104` (SPEC example)
**Evidence:** Upstream Coldcard's sample uses alphabetical key order in each sub-object: `_pub → deriv → first → name → xfp → xpub`. The toolkit (and the SPEC example) uses `name → deriv → xfp → xpub → _pub → first`. Tolerance varies by downstream consumer; the toolkit's choice is internally consistent (SPEC = implementation = fixtures), but differs from upstream sample.

**Fix:** Either (a) accept the divergence as intentional toolkit policy with a one-line code comment, OR (b) re-pin `ColdcardSubDerivation` fields in alphabetical order. Recommend (a) with the comment, since downstream consumers parse by key name.

---

## Findings — Test consistency

### I-4 — Refusal tests use `.contains()` substring checks; the byte-exact contract is unenforced

**Location:** `crates/mnemonic-toolkit/tests/cli_export_wallet_coldcard.rs:178-183, 272-276` + `crates/mnemonic-toolkit/tests/cli_export_wallet_jade.rs:165-170, 207-212`
**Evidence:** All four refusal tests assert `stderr.contains(expected)` rather than `assert_eq!(stderr.trim_end(), expected)`. The Jade singlesig test (`cli_export_wallet_jade.rs:166`) is precisely the case where `.contains()` masks the C-1 doubled-`error:` bug.

**Fix:** Land the six pinned `.stderr` fixture files per IMPL_PLAN Phase 1 step 3 and assert via `assert_eq!(stderr, std::fs::read_to_string(FIXTURE)?)`. Interim: change `.contains()` to `assert_eq!(stderr.trim_end(), format!("error: {}", expected))`.

---

## Findings — Manual mirror

### L-2 — Manual doesn't document `master_xpub` conditional emission or the 20-char `--wallet-name` truncation

**Location:** `docs/manual/src/40-cli-reference/41-mnemonic.md:152, 158`
**Evidence:** Manual line 152 lists `master_xpub` in the `--slot` subkey enumeration. Manual line 158 documents `--wallet-name` but says only "default `<template-human-name>-<account>`". Neither the SPEC §5.1 conditional emission of top-level `xpub`, nor the SPEC §5.2 20-char truncation of the `Name:` field is documented user-facing.

**Fix:** Add one prose paragraph under the `mnemonic export-wallet` reference noting (a) the `--wallet-name` 20-char truncation for Coldcard multisig only, (b) the `master_xpub` slot subkey controls top-level `xpub` emission in Coldcard generic JSON.

---

## Findings — --wallet-name truncation

### N-1 — `String::truncate` on a multi-byte UTF-8 wallet name will panic at a non-char-boundary

**Location:** `crates/mnemonic-toolkit/src/wallet_export/coldcard.rs:286-289`
**Evidence:** `if name.len() > 20 { name.truncate(20); }` — `String::truncate` panics if the byte index is not a char boundary. A user passing `--wallet-name 'привет привет привет привет'` (Cyrillic) where byte 20 lands mid-codepoint will crash. ASCII-only inputs work.

**Fix:** Replace `name.truncate(20)` with a char-boundary-safe truncation or refuse non-ASCII wallet names at the clap layer.

---

## Findings — Cross-cutting

### L-3 — Three unused-import warnings introduced by the Phase 0.6 wallet_export submodule split

**Location:** `crates/mnemonic-toolkit/src/wallet_export/mod.rs:20-24`
**Evidence:** `cargo build` surfaces three unused-import warnings for `format_bip388_wallet_policy`, `format_bitcoin_core_importdescriptors`, and `descriptor_to_bip388_wallet_policy` — re-exported but no caller outside the submodule consumes them.

**Fix:** Drop the bare-function re-exports; keep only the `*Emitter` types.

---

## Convergence note

Phase 1 ships working Coldcard + Jade emitters with all 12 integration tests passing and zero v0.7 regressions. The byte-exact fixtures are pinned and consistent.

**Blocking for Phase 2 promotion:**
- **C-1** (Jade doubled `error:` prefix) — one-line source-string fix.
- **I-1** (latent `build_missing_fields_refusal` doubled prefix) — same fix shape.
- **I-2** (`master_xpub` unwired) — plumb or FOLLOWUPS + refuse-on-supply guard.
- **I-3** (`collect_missing` placeholder + missing `.stderr` fixtures) — IMPL_PLAN exit gate incomplete.
- **I-4** (refusal tests use `.contains()`) — fold with I-3 fixture-pinning.

**Non-blocking:** L-1 (comment), L-2 (manual prose), L-3 (drop dead re-exports), N-1 (char-boundary-safe truncate).

Recommend Phase 1.11+ patch cycle to fold C-1 + I-1 + I-3 + I-4 (one focused commit each); I-2 best resolved by deciding plumb-now vs FOLLOWUPS-and-refuse.
