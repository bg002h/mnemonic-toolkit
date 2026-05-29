# R2 ARCHITECT REVIEW — `SPEC_path_raw_bracketed_bare_unification.md`

**Reviewed at:** `origin/master = dd7c228`. Sonnet feature-dev:code-reviewer, R2 fold-verify of the R1 I-A fold. Persisted verbatim. All source citations independently verified.

## R1 I-A fold confirmation

The I-A fold is **correctly executed**. All three source-truth checks pass:

1. `export-wallet` accepts `--slot @N.path=` at `export_wallet.rs:111-116` (confirmed) and routes through `resolve_slots` at `export_wallet.rs:347` (confirmed).
2. The export consumers do NOT fold `h`→`'` today — `coldcard.rs:308-317` `normalize_path` only prepends `m/`; `pipeline.rs:33-44` `key_origin_str` only strips `m/` and emits verbatim — confirmed. `sparrow.rs:130` passes `path_raw` directly to `normalize_derivation`; `electrum.rs:163-164` emits `path_raw.clone()` verbatim. The A4 wire-value change is real on all four consumers.
3. Every `@N.path=` in the test suite passes canonical apostrophe form — confirmed by exhaustive grep of `tests/cli_export_wallet*.rs` and all other test files. No test passes `48h` or any non-canonical form. A4's "currently untested; suite stays green" claim is verified correct.

A4 text in §6, T10 in §8, Phase 2 assignment in §9, and the §10 GUI/CHANGELOG note all correctly reference A4. No new dangling citations introduced.

## CRITICAL
None.

## IMPORTANT
None.

## MINOR

### M-1. §10 "User-visible changes" sentence omits A3 and A4
Line 191 lists only §1.1 + A1 + A2; A3/A4 are declared wire-value changes on user-facing surfaces. The adjacent GUI-decision sentence names A3/A4, so a diligent implementer wouldn't miss them, but the CHANGELOG guidance line is incomplete. Fix: append A3/A4 to the line. *(Folded post-R2.)*

### M-2. §10 Manual mirror note covers only `bundle --import-json --json` samples
Phase 5 manual grep is narrowly scoped; A4 could affect `export-wallet` transcripts using non-canonical `@N.path=` (none exist per the test sweep, so low risk). Fix (optional): broaden the grep instruction. *(Folded post-R2.)*

## Internal consistency cross-check
- A4 ↔ A3: same root cause (`resolve_slots` `Xpub` arm, `bundle.rs:547` `p.value.clone()`) flowing through different consumer sets (C5/C6/C7 vs C1/C2/C3/C4). No contradiction.
- A4 ↔ §5 C4: C4 rewrite keeps the fallback path-bearing (both callers `pipeline.rs:75`, `bip388.rs:73` pass `template_origin_path_no_m(...)`). Consistent.
- A4 ↔ §3: `bracketed_origin()` non-empty arm → canonical `[fp/48'/0'/0'/2']`; `origin_path_bare()` → `m/48'/0'/0'/2'`. Correct.
- A4 ↔ §8 T10: tests `@0.path=48h/0h/0h/2h` → canonical coldcard output, Phase 2. Consistent.
- R0 C-1/C-2 narratives undisturbed by the A4 fold.
- §4(a) M-A line numbers verified: `specter.rs:255` ✓, `sparrow.rs:437` ✓, `json_envelope.rs:362` ✓.

## VERDICT: GREEN (0C / 0I)
Counts: **Critical 0, Important 0, Minor 2.** The R1 I-A fold is correctly executed, all source citations verified, no fold-introduced drift rises to Important. The two minors are cosmetic §10 documentation gaps with no implementation risk (folded post-R2). Implementation may proceed.
