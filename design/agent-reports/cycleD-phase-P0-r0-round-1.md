# PER-PHASE R0 (P0) — concrete-nonranged-xpub — round 1

**Verdict: GREEN (0 Critical / 0 Important)** — 1 trivial non-blocking cosmetic Minor.
**Reviewer:** adversarial opus architect, funds-weighted (read-only). Worktree @ `d24704a7` (content byte-identical to HEAD `6b80adee` — the P0 commit was amended for message accuracy only; diff empty). Diff `cec8acc2..d24704a7` — 2 files, +430 (`pipeline.rs` +19; new `cli_concrete_nonranged_xpub_reject.rs` +411).
**Dispatched:** 2026-07-07 (Cycle D, per-phase R0 over the P0 code, funds-weighted). Persisted verbatim per CLAUDE.md.

The code faithfully implements SPEC §5 + the plan; the funds anchor is genuine and non-tautological; the test matrix is complete; no collateral. Advance to the whole-diff review + release.

## Independent verification (re-ran)
`cargo test -p mnemonic-toolkit` → **3635 passed / 0 failed** (203 binaries; doc-tests green). New file 11/11. `cargo clippy -p … -- -D warnings` → clean. `lex_residue_floor_accepts_bare_at_n_d1_deferred` + `lex_bare_at_zero` → ok; `parse_descriptor.rs` 0 diff lines (pin unmodified).

## 1. Reject check (pipeline.rs:352-359) — faithful, correct
Placement after `let m = cap.get(0)` @341, before push @361/decode @369 (clean early return). Byte-exact prefix `import-wallet: bsms: parse error: ` @354 → remap to DescriptorParse exit 2 (bundle/verify) / import `.replacen` (empirically: §6.1/6.2 exit 2 + no bsms: leak; §6.8-descriptor remaps to `import-wallet: descriptor:`; §6.8-bsms keeps native prefix). `@{idx}` names the right key (§6.6 asserts `@0`). Borrow/slice/boundary safe (m.end() at ASCII base58 end = valid char boundary; empty tail → reject no panic; idx:u8 Copy). Does NOT fire for ranged (§6.3 `/*`, §6.4/§6.9-mp `/<0;1>/*` round-trip; §6.7 `/0/*` passes the `/` through → floor rejects). Unreachable for hand-typed `@N` (inside key_regex loop; §6.5 `wpkh(@0)` succeeds).

## 2. §6.2 funds anchor — non-tautological, proves reject-before-comparison
Builds the card via the ranged `/*` spelling (byte-identical md1 card the bug produced — both collapse to `UseSitePath{multipath:None, wildcard_hardened:false}`); verifies the NON-ranged descriptor against it; asserts !success, exit 2, !`result: ok`, !`import-wallet: bsms:`, AND `@0`+`no derivation suffix`. The parse-reject text can ONLY come from `concrete_keys_to_placeholders` (re-parse), so asserting it proves the reject fired BEFORE `verify_emit_from_expected` card comparison. Differential vs §6.3 positive control (ranged round-trips) — the only varying axis is `/*`. Genuine.

## 3. Matrix completeness (§6.1-6.10) — complete; M1/M2/M3 satisfied
All cells present. M1: fixtures are real `[fp/path]xpub` (`84'/0'/0'`, `48'/0'/0'/2'`, `86'/0'/0'`) → key_regex matches, NEW check fires (not "no keys found"). M2: §6.5 hand-typed literal `wpkh(@0)` (AtN path). M3: "no bsms: leak" scoped to bundle/verify only; bsms-import cell asserts native prefix kept. Fixtures are real mainnet xpubs (ranged forms decode + round-trip). No tautological cell.

## 4. Collateral — none
2 files only. No new ToolkitError variant (reuses ImportWalletParse); error.rs/parse_descriptor.rs/mlock.rs 0 diff lines. No-op for every `/`-suffixed key → all normal descriptors + other importers unaffected (full suite 3635/0). Deterministic (pure string inspection).

## 5. Bare-`@N` template + unit pin — exercised + green + unmodified.

## Minor (non-blocking, cosmetic — do NOT gate)
**M-1:** §6.1's `stderr.contains("/*") && stderr.contains("/<0;1>/*")` is logically redundant (`/<0;1>/*` ends in `/*`). The message contains both remedies verbatim so it passes and is not wrong; optionally tighten to the ranged remedy's own phrase (`` `/*` (ranged) ``). No funds/correctness impact. **Coordinator disposition: left as-is (passing, cosmetic; not worth re-committing a green test).**

**GREEN — 0C/0I.** Funds property (verify-bundle false-pass closed at re-parse before card comparison) empirically proven; suite 3635/0 + clippy clean, independently reproduced. Clear to advance to the mandatory post-impl whole-diff review, then the v0.79.0 release ritual.
