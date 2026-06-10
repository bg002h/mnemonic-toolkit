# R0 round-2 architect review — PLAN_hrp_case_insensitive_probes (2026-06-10)

Reviewer: Fable 5 architect agent (R0 round 2, post-fold verification). Source 38db912. Verdict: RED (0 Critical / 1 Important I1-r2 — inherited oracle error in round-1's own I1 prescription / 3 Minor; all six round-1 folds verified faithful). Review verbatim below.

---

## Critical

None.

## Important

**I1-r2 — The I1 fold's inverted-test oracle is mis-specified for the fixture the test retains; "codec-attribution" is unsatisfiable with `MS1XXX` (empirically proven).** The plan flips `validate_flag_hrp_case_mismatch_distinguishable` to "assert acceptance-then-codec-attribution (exit still 2)". But the typed-flag repair path is NOT a decode path: `repair_card`'s Ms1 arm pre-gates every chunk through the toolkit's own `parse_chunk` (`repair.rs:785`, fn at :569-631) BEFORE `repair_via_ms_codec` (:793). `parse_chunk` lowercases (:574) and length-gates (:617-631) — so the existing 6-char fixture `MS1XXX` dies at the toolkit parse step and **never reaches ms-codec**. Empirically verified via the lowercase twin: `repair --ms1 ms1xxx` → exit 2, stderr `repair: chunk 0 parse failed before correction could run: data-part length 3 is outside BIP-93's valid range [14, 93] ∪ [96, 108]`. Round-1's own I1 wording missed the :785 pre-gate — the fold transcribed it faithfully, so this is fold-faithful but claim-wrong; same mis-specified-oracle class as the v0.49.1 I2 lesson. Concrete fix, one clause: either (a) keep `MS1XXX`: assert absence of `"case mismatch"` + presence of the parse-step marker (`"parse failed before correction could run"` / `"data-part length"`) + exit 2; or (b) swap to a full-length uppercase fixture (uppercased `VALID_MS1`, repair.rs:1483): reaches ms-codec → `WrongHrp{got:"MS"}` → mapped at repair.rs:859-863 → `repair: chunk 0 HRP mismatch — expected 'ms', found 'MS' …` (the toolkit's RepairError::HrpMismatch translation, NOT friendly.rs's decode-path string — confirmed live: `convert --from ms1=<UPPER-full>` → `error: ms1 wrong HRP: got "MS", expected "ms"`). Either way pin the concrete post-fix marker so red-first can go green without the two forbidden "fixes" (skipping the pre-gate, or lowercasing before the codec).

## Minor

**M1-r2 — The test's third assert (:423-426, `contains("--ms1")`) survives inversion vacuously** (post-relaxation stderr still contains `--ms1` via the M3 secret-argv advisory, not via error attribution). Drop or retarget (assert advisory and error independently).

**M2-r2 — The positional-MS1 cell should pin the command + marker.** "ms-codec-attributed error" is satisfiable through **inspect** (decode path → friendly `ms1 wrong HRP: got "MS", expected "ms"` — confirmed live) but through **repair** renders as the toolkit HrpMismatch translation. Add "through inspect" + the literal marker.

**M3-r2 — Citation nit:** the test spans :403-427; the plan cites ":404-428".

## Fold-verification

All six round-1 folds present and anchor-accurate at 38db912: I1 (fn :404, marker :418-421, doc :396-402 — behavioral expectation inherits the pre-gate miss → I1-r2), I2 (silent_payment :134/:171-176 verified; "ms-codec-attributed" ACCURATE there — true decode path, marker confirmed live), M1 (friendly.rs:79 catch-all verified), M2 (:121-124/:143-150 verified), M3 (scripts/install.sh:32 = the self-pin line; load-bearing given the v0.53.1 history), M5 (verify_bundle.rs:1242 rides the shared fn — verified). M4 retained. Whole-plan re-scan: all anchors exact; UnknownHrp full-echo re-confirmed live (full 51-char uppercase secret on stderr today); no fold-drift.

## Verdict

**NOT GREEN — 0 Critical / 1 Important / 3 Minor.** One targeted fold (pin the inverted-test oracle per (a) or (b), sweep the minors in the same edit), then round 3.
