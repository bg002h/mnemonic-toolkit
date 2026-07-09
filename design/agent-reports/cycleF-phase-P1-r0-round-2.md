# P1 per-phase R0 review — ms1-repair-demote-to-candidate — round 2

**Verdict: GREEN (0 Critical / 0 Important)**
**Reviewer:** Fable, per user directive. `mnemonic-secret` worktree @ `0212b2e` (base `c2fd4eb`); incremental diff `9552700→0212b2e` = exactly the 3 announced folds.
**Dispatched:** 2026-07-09 (Cycle F, per-phase P1 R0 round 2 — convergence). Persisted verbatim.

## Fold verification
- **I-1 FIXED.** `main.rs:129` → `(exit 4 = VERIFY-ME candidate)`; long-help → "Exit 4 on correction-applied (Cycle F demotion — a corrected ms1 is an UNVERIFIED candidate…; D26)". Verified LIVE (`ms --help` + `ms repair --help`) → so `ms gen-man`→`ms-repair.1` too. No residual `REPAIR_APPLIED`/behavioral-exit-5 on the funds surface (sole `src/` hit = `cmd/repair.rs:28`, the correct "exit 5 is effectively unreachable for ms repair" explanation, not drift).
- **M-1 FIXED.** `repair_json_envelope_shape` pins the full 5-field D27 order `schema_version<kind<verdict<corrected_chunks<repairs`; live JSON confirms.
- **M-2 FIXED.** comment reworded (reason BODY byte-identical, ms-cli prepends `repair: `).

## Round-1 GREEN properties — all still hold on `0212b2e`
Funds: 1-subst→exit 4 + 1 UNVERIFIED line; clean→exit 0 no line; uncorrectable→exit 2; exit 5 unreachable; advisory fixed static text. D27 byte-match `schema_version,kind,verdict,corrected_chunks,repairs` = toolkit P0; verdict candidate/blessed; no IndelJson. Secret-hygiene: all buffers Zeroizing; fold added only doc text + a test assertion. NO-BUMP: Cargo.toml/lock/ms-codec 0-line diff; no version bump. mlock g6: 0-line diff.

## Counts
`cargo test -p ms-cli` → **225 passed, 0 failed, 5 ignored**; clippy clean.

**Gate: CONVERGED (0C/0I). P1 cleared to advance to P2 (docs lockstep).**
