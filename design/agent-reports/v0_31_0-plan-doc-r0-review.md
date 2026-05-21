# v0.31.0 plan-doc R0 review

**Reviewer:** opus
**Round:** R0
**Plan under review:** design/PLAN_mnemonic_toolkit_v0_31_0.md
**Brainstorm v1 (7a library):** design/BRAINSTORM_v0_31_0_bsms_bip129_encryption_v1_7a_library.md
**Date:** 2026-05-21
**Source SHA:** 6e522bb (master HEAD)

## Critical (C)

### C1. Plan-doc misidentifies the orchestrator decrypt insertion site; `BsmsParser::parse` is NOT inside the `Some("bsms")` arm.

**Citation:** plan-doc L186-238 says "Place this BEFORE the existing `BsmsParser::parse(&blob, stderr)` call in the bsms arm". Plan-doc L92 cites `Some("bsms") => ` "after L535". Verified against source:

- `Some("bsms") =>` is at `import_wallet.rs:265`, NOT after L535 (L535 is the `Some("jade") =>` arm).
- `BsmsParser::parse(&blob, stderr)?` is at `import_wallet.rs:796`, inside a **separate** `match format_str { ... }` dispatch block (L795-811), NOT inside the `Some("bsms")` arm at L265-316.
- The `Some("bsms")` arm only does sniff-mismatch checks then evaluates to the literal `"bsms"` (L315). It carries no parse call.

**Impact:** the Phase 3 Step 3 orchestrator block as written does not fit into the location described. Phase 3 also needs `let blob` at L260 changed to `let mut blob`.

**Fix:** rewrite Phase 3 Step 3 to lock the exact insertion site. Place the decrypt block between the format-resolution match (ends at L768) and the parser-dispatch match (starts at L795), guarded by `if format_str == "bsms" && args.bsms_encryption_token.is_some()`. Change L260's `let blob` to `let mut blob`.

### C2. Phase 4 Step 2 "no-regression" fixture `bsms-2of3-decay-bip129-4line.bsms` does not exist.

**Citation:** plan-doc L489 names fixture `bsms-2of3-decay-bip129-4line.bsms`; no such file exists. Real fixtures use `.txt` extension and different names (`bsms-2line-multi-2of3.txt`, `bsms-1of1-singlesig.txt`, etc.).

**Impact:** the no-regression cell as authored compiles but runs against a `.exists()` returning `false`, so the assertion is silently skipped — vacuous pass.

**Fix:** use `bsms-2line-multi-2of3.txt` (or any existing fixture); remove the `if blob.exists()` guard so the assertion is load-bearing.

## Important (I)

### I1. Plan-doc's `read_bsms_token` uses `path == Path::new("-")`; the existing sibling `read_blob` precedent uses `path.as_os_str() == "-"`.

**Fix:** change `read_bsms_token` to mirror `read_blob`: `if path.as_os_str() == "-"`.

### I2. Stdin-contention guard between `--blob=-` and `--bsms-encryption-token=-` is in the Risk Register but not in the plan code.

**Fix:** add an explicit refusal: `if blob_is_stdin && token_is_stdin { Err(ToolkitError::BadInput("--blob=- and --bsms-encryption-token=- cannot both read from stdin")) }`. Add integration cell.

### I3. Plan claims `BsmsMacMismatch` slots "between `BsmsImported` (if exists)..." — verified there is no `BsmsImported`.

Actual variants in error.rs:
- L28: `BsmsRound1Malformed`
- L34: `BsmsSignatureMismatch`
- L43: `BsmsTaprootImportRefused`
- L56: `BsmsTaprootRefused`

`BsmsMacMismatch` slots BETWEEN `BsmsRound1Malformed` (L28) and `BsmsSignatureMismatch` (L34).

**Fix:** update plan-doc + insert at the correct alphabetical slot.

### I4. Test `token_via_stdin` has no assertions — it is dead documentation, not a regression guard.

**Fix:** add explicit assertions (`stderr.contains("BIP-129 encrypted Round-2 envelope decrypted")`).

### I5. MAC verify uses byte-by-byte `!=`; plan documents as "adequate for non-interactive use" but `subtle::ConstantTimeEq` is in transitive deps.

**Fix:** use `subtle::ConstantTimeEq` for the MAC compare. Cleaner threat model.

### I6. Phase 3 Step 3 `iv` slicing variable semantics unclear.

**Fix:** add a 3-line code comment block explaining the BIP-129 wire shape.

### I7. Manual lint invocation in Phase 5 Step 4 omits MD_BIN/MS_BIN/MK_BIN.

**Fix:** include all four bins in the lint invocation per the canonical convention.

## Minor (M)

### M1-M4. Various polish items.

## Verdict

**YELLOW — fold then proceed.**

C1 + C2 are mechanical but real (would either fail to compile, fail to integrate cleanly, or silently skip an intended test). All Importants are tightening + sibling-precedent alignment. After folding the plan is GREEN to dispatch into Phase 2 implementation.
