# PLAN R0 review — concrete-nonranged-xpub-implied-wildcard — round 1

**Verdict: GREEN (0 Critical / 0 Important)** — 5 Minor (non-blocking, test-precision guard-rails).
**Reviewer:** adversarial opus architect (read-only). Verified @ toolkit `23fb34d6` (== `e092f679` for every cited source file — the two intervening commits are design-docs only; source diff empty).
**Dispatched:** 2026-07-07 (Cycle D, IMPLEMENTATION_PLAN R0 loop round 1). Persisted verbatim per CLAUDE.md.

The plan faithfully executes SPEC §5; the P0 code sketch is correct (scope/borrow/slice sound); TDD complete with the funds anchor RED-first; the 5 SPEC folds are consistently reflected; the release ritual covers version sites + gates. Cleared for single-implementer dispatch after folding (or waiving) the Minors.

## 1. P0 sketch faithfully implements SPEC §5 — YES
`if !descriptor[m.end()..].starts_with('/') { return Err(ImportWalletParse("import-wallet: bsms: parse error: concrete key @{idx} …")) }` after `let m = cap.get(0)` (@341), before xpub decode (@351):
- `m` bound @341, valid. `idx` declared @338, incremented @384 (loop end) → at loop top holds the CURRENT key's 0-based index → `@{idx}` names the offending key correctly (matches §6.6). Borrow-safe (transient immutable borrow, `bool` result, `idx: u8` Copy). Slice/byte-boundary safe (m.end() at end of ASCII base58; empty tail → reject, no panic; same idiom @342/@391). Byte-exact prefix satisfies the remap strip (@413-414 → DescriptorParse exit 2) + importer `.replacen`. Early `return` on first non-ranged key discards partial `placeholder_form`. No bug.

## 2. TDD completeness + §6.2 card-construction subtlety — complete + non-tautological
All §6.1-6.10 assigned, RED-first, funds anchor §6.2 RED-first. **The `/*`-spelling card construction is VALID:** `wpkh([fp]xpub)` and `wpkh([fp]xpub/*)` produce a BYTE-IDENTICAL md1 card (both → `UseSitePath{multipath:None, wildcard_hardened:false}` via `make_use_site_path` @334-337 — the structural identity that blinds the lexer). So the `/*`-built card IS exactly the pre-fix bug's card; the test asserts the ORIGINAL no-wildcard descriptor now REJECTS against that genuinely-ranged card = the C1-class false-pass closed. Reject timing (`?` @verify_bundle.rs:1368 before verify_emit_from_expected @1373) re-verified accurate.

## 3. Fold-drift (5 SPEC Minors → plan) — consistent
M1 taproot §6.9 ✓; M2 CHANGELOG-new-not-rewrite ✓; M3 FOLLOWUPS flip ✓; M4 byte-exact prefix + test ✓; M5 placement/completeness ✓. No drift.

## 4. Funds/behavior risks — covered (3 test-precision cautions → Minors 2-4)
## 5. Missing steps — none. fmt/clippy (`-p` only, mlock exempt), per-phase full suite, whole-diff endpoint, v0.79.0 version sites (+ CHANGELOG new-entry + FOLLOWUPS flip + re-vendor N/A) all present; no GUI/schema/manual lockstep (no clap change). Cross-checked vs the release ritual + Cycle-A `.examples-build/` gotcha — complete.

## Minor findings (non-blocking — plan §6 test-cell clarifications)
- **M1** — test cells MUST use a real `[fp/path]xpub` (≥1 path component; `key_regex` path group is `(?:/\d+(?:'|h)?)+`, mandatory `+`). A no-path `[deadbeef]xpub` doesn't match `key_regex` → the check never fires → a reject cell would pass for the WRONG reason ("no [fp/path]xpub keys found") and an accept cell would fail. Use the recon fixture `[73c5da0a/84h/0h/0h]xpub6CatWdiZ…` (or equiv with-path origin); cite it in the plan.
- **M2** — §6.5 ACCEPT must feed a hand-typed literal `@N` descriptor (`wpkh(@0)`, AtN direct-lex path, never enters `concrete_keys_to_placeholders`), NOT a concrete descriptor with `--md1-form=template` (that correctly still rejects). Make the routing distinction explicit.
- **M3** — scope the "no `import-wallet: bsms:`" assertion to bundle/verify-bundle ONLY. `import-wallet --format bsms` legitimately keeps `import-wallet: bsms:` (correct format prefix); `--format descriptor` → `import-wallet: descriptor:`. A blanket assertion would false-fail the bsms-import cell.
- **M4** — assert §6.2 on the parse-reject MESSAGE ("@0 … no derivation suffix …"), not merely exit 2 (a card-comparison failure also exits non-zero) — proves the reject fired at re-parse before comparison.
- **M5** — if the full suite surfaces a pre-existing test asserting the OLD silent-accept, update the TEST (it encoded the bug), don't weaken the fix. Corpus grep found ZERO such tests (the one concrete-non-ranged descriptor is an `xpub-search` `contains_at_n_placeholder` helper, off the choke-point path). Caveat, not an expected fold.

**R0 exit: GREEN (0C/0I).** Both Cycle D gates GREEN. Fold the 5 test-precision Minors then proceed to single-implementer TDD (no mechanism change; drift risk near-zero).
