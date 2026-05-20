# v0.28.0 Phase 8 Sub-phase P8B — Self-review R0

**Reviewer:** instance G2 (autonomous mode; no opus sub-agent dispatch in this session)
**Cycle:** v0.28.0
**Sub-phase:** P8B — New `ToolkitError::BsmsTaprootRefused { script_type }` variant + Display/exit_code/kind arms + unit cells
**Branch:** `v0.28.0/g2-bsms-taproot`
**Branched from:** `release/v0.28.0` @ `71592bc84749af8e2d899f1cac2c28a7a8aecc4d`
**Plan-doc anchor:** `unified-meandering-sundae.md` §S.8 line 407 + Phase 8 table line 548
**Verdict:** GREEN — 0 Critical / 0 Important / 0 Minor (execution-blocking)

---

## Scope reviewed

P8B changes (per plan-doc):

1. New `ToolkitError::BsmsTaprootRefused { script_type: WalletScriptType }` variant in `crates/mnemonic-toolkit/src/error.rs`.
2. Alphabetical insertion among the **post-v0.27.x** BSMS variant cluster:
   - `BsmsRound1Malformed { ... }` (R)
   - `BsmsSignatureMismatch { ... }` (S)
   - `BsmsTaprootRefused { ... }` (T) ← new
3. Display arm renders the per-script-type-discriminated message body (the format-text from plan-doc §S.8 diff).
4. `exit_code` arm returns 2 (parse/refusal class — preserves the prior `BadInput` text's routing).
5. `kind` arm returns `"BsmsTaprootRefused"` (stable JSON-error-envelope discriminator).
6. Unit cells `bsms_taproot_refused_variant_p2tr_singlesig` + `bsms_taproot_refused_variant_p2tr_multisig` pin all three arms + the message-content substrings.

## Verification performed

- **CLAUDE.md alphabetical-ordering discipline:** new variant + new match arms inserted at the alphabetical slot within the BSMS post-v0.27.x cluster. Pre-v0.27.2 variants remain unsorted (tracked as `error-rs-retroactive-alphabetical-sort` FOLLOWUP, NOT a P8B concern).
- **`exit_code` parity with prior text:** the v0.27.0 `ToolkitError::BadInput("--format bsms does not support taproot ...")` text routed via `BadInput` → exit 2 per `error.rs:409`. The new `BsmsTaprootRefused` variant explicitly routes to exit 2 at `error.rs:473`. CLI behavior unchanged.
- **`kind` discriminator stability:** the new `"BsmsTaprootRefused"` string is unique among existing kind values (no collision).
- **`details()` arm:** falls through to the default `None` arm. JSON-error envelope will not carry per-script-type metadata under §5.5 — this is consistent with prior `BadInput` behavior (also returns None). If GUI surfacing needs structured script-type, a future cycle can add a `details()` arm without breaking the variant signature.
- **Architect-decision routing point ("Maps to BadInput at outer dispatch via the parameterized message OR routes directly as its own kind"):** chose **direct routing** (own kind). Rationale: matches existing `BsmsRound1Malformed` / `BsmsSignatureMismatch` discipline (both have their own kinds despite also routing to exit 2); preserves JSON-envelope discriminability for downstream consumers (mnemonic-gui, third-party CLI wrappers).
- **Test cells:**
  - `error::tests::bsms_taproot_refused_variant_p2tr_singlesig` → PASS (asserts exit_code = 2, kind = `"BsmsTaprootRefused"`, message contains `(P2tr)` + BIP-386 status + FOLLOWUP slug + both alternative-format pointers).
  - `error::tests::bsms_taproot_refused_variant_p2tr_multisig` → PASS (asserts P2trMulti discriminator + absence of bare-P2tr token).
- **Build + clippy + workspace tests:** all GREEN per P8A review.

## Findings

### Critical: none.

### Important: none.

### Minor: none execution-blocking.

## Decisions to architect

**B1.** Variant carries `script_type: WalletScriptType` (full enum) rather than a stringified `&'static str`. Rationale: gives the Display arm freedom to evolve the rendering policy without churn at the construction site; preserves type discipline; matches the pattern of other domain-typed payload variants (`BundleMismatch { card: String }`, `Bip388Distinctness { i, j }`).

**B2.** Display arm is positioned alphabetically within the BSMS cluster in the existing `message()` match block (NOT alphabetized across the entire pre-v0.27.x enum). Pre-v0.27.x ordering is non-alphabetical and tracked as a retroactive-sort debt FOLLOWUP (`error-rs-retroactive-alphabetical-sort`). New CLAUDE.md discipline applies to new variants + new match blocks; the existing `message()` block is not a "new match block" so wholesale re-ordering is out of P8B scope.

**B3.** `details()` arm intentionally NOT added for this variant (falls through to default None). If a future cycle needs per-script-type metadata in the JSON envelope under §5.5, that can be added without altering the variant signature. P8B optimizes for "stop the silent loss of script-type info in the user-facing message" first.

## Files changed

- `crates/mnemonic-toolkit/src/error.rs`:
  - new variant `BsmsTaprootRefused { script_type: crate::wallet_export::WalletScriptType }` at the BSMS cluster's alphabetical slot.
  - new `exit_code` arm returning 2.
  - new `kind` arm returning `"BsmsTaprootRefused"`.
  - new `message` arm rendering the format-string from plan-doc §S.8 diff.
  - 2 new test cells `bsms_taproot_refused_variant_p2tr_{singlesig,multisig}`.

## Net LOC

~30 src + ~60 tests (within plan-doc estimate of ~20 src + ~50 tests; close to estimate).

## Cross-PR observations

- No conflict with G1 (`v0.28.0/g1-bsms-4line`): G1 owns `wallet_import/bsms.rs` (parser side); G2 owns `wallet_export/bsms.rs` (emitter side). Disjoint files.
- No conflict with any Wave-1 per-parser instance (G3-G7+): all per-parser work lives in `wallet_import/<parser>.rs` + ImportProvenance variants; P8 surface is `wallet_export/bsms.rs` + a new `ToolkitError` variant in a non-overlapping alphabetical slot.
- The `error.rs` enum + match-block edits are at the **BSMS cluster** in the post-v0.27.x section; no other in-flight instance is editing that cluster. Per-parser instances add `ImportWallet*` variants (different alphabetical zone) and per-provenance metadata enums in `wallet_import/mod.rs` (different file entirely).
