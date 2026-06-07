# R0 Architect Gate — Round 2 (fold-confirmation) — `SPEC_technical_manual_g2_bare_ci_fix.md`

> Round 1 = GREEN (0C/0I) + 3 Minors (M1/M2/M3); all folded. This round confirms the folds are clean. Reviewer had Read/Glob/Grep; parent persists.

**Verdict: GREEN (0C / 0I).** One new Minor (a fold-introduced cross-reference number, non-blocking).

The three Round-1 Minors (M1/M2/M3) all landed correctly. The folds were SPEC-wording-only and did not alter Items 1/2/3's change instructions, the guard logic, the 12-token table, the trigger, the AUTHORING edits, the disposition, or the verification plan's other steps. The substance Round 1 verified is intact.

## Critical
None.

## Important
None.

## Minor

**M-new (line 97) — stale ship-plan step back-reference (fold artifact).** The M2 out-of-scope paragraph ends "...filed as a separate one-line FOLLOWUP `technical-manual-glossary-timestamparg-default-prose-stale` **(ship plan step 4)**." But the FOLLOWUP filing actually lives in **ship-plan step 3** (the `FOLLOWUPS.md` step that both flips the resolved entry and files the new one). Step 4 is stage/commit/push and files nothing. Classic fold artifact: M2 merged the file-new instruction into step 3 rather than creating a new step 4, but the back-pointer in line 97 was not updated. An implementer executing step 3 files the FOLLOWUP with the correct id regardless, so no code/verification outcome changes — Minor, not Important. Mechanical fix: line 97 `(ship plan step 4)` → `(ship plan step 3)`. A Minor does not reopen the gate or require another R0 round.

## Fold confirmation

- **M1 — LANDED, accurate, not overstated.** The reword states 5 of the 6 non-symbol checks scan docs text only and the 6th (api-surface-coverage) reads `lib.rs`/`format.rs` but is warning-only via `lint.sh:82 || warn`. Verified against `lint.sh`: line 82 is exactly `|| warn "api-surface-coverage reported gaps (warning only; …)"`; only `err()` sets `fail=1`, `warn()` does not. So symbol-ref-check (step 7) is the only blocking check a code diff can react to → "the 6 cannot newly fail" is correct, hedged with "red only if master already red." Only occurrence of "docs text only" — contradicts nothing.
- **M2 — LANDED, consistent, FOLLOWUP-filing in the ship plan.** The OUT-OF-SCOPE framing (path-pin-only; G2 unaffected because `ExportWalletArgs`+`timestamp` segments both exist) does not contradict Item 3's line-385 instruction: Item 3 qualifies only the bare token `cmd/export_wallet.rs::ExportWalletArgs::timestamp`, leaving the prose and the already-qualified `…/mod.rs::TimestampArg` alone — qualify-the-path, leave-the-prose. FOLLOWUP id `technical-manual-glossary-timestamparg-default-prose-stale` is byte-identical at both occurrences (no orphan/typo), filed in ship-plan step 3.
- **M3 — LANDED, coherent.** §6.2 gained the regression assertion: the toolkit-only post-qualify run must confirm the address-derivation chapters' bare `to_miniscript.rs`/`address_derivation.rs` refs STILL `skip:absent-sibling` (codec-only — md-codec-owned), not flip into `unqualified-toolkit`. Coherent precisely because the risk is real: the guard fires on `repos_with == ["toolkit"]`, which for these bare codec refs triggers only if a future toolkit file collides on basename — exactly the silent-capture the skip-count-unchanged assertion catches at runtime.

## Notes
- The two-part guard mechanism, Decision B's four exclusions, the 12 grep-verified tokens, and Decisions A/C are untouched by the folds; Round 1's verification stands and was not re-derived.
- Gate stays GREEN (0C/0I). The lone Minor is a one-token cross-ref fix; folding it does not warrant another R0 round.
