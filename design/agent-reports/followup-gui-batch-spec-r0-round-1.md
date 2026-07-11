# R0 review — SPEC_followup_gui_batch.md (round 1) — Fable, adversarial

**Persisted per CLAUDE.md.** Verified vs mnemonic-gui @ fac2521 + installed binaries; mutations run live + reverted.

## CRITICAL
**C1 — S2 defaults leg infeasible: md/ms/mk emit gui-schema v1 (ZERO `default_value` fields; only mnemonic v5).** Extended defaults gate spurious-REDs 13 correct mirror entries (mirror=Some, JSON=None) AND is blind to 7 real mirror omissions (None==None GREEN — F6 class). Third category = producer-capability gap (not "real drift" nor "missing gate"). Verified: md 0/35 defaults, ms 0/36, mk 0/32; 13 mirror defaults all cross-check correct vs --help (0 real drift); 7 clap defaults MISSING from mirrors (md encode/verify/address --network etc., benign opts[0]-coincidence today). **Rescope:** (a) choices-only for md/ms/mk (JSON has non-null choices; 0 day-one drift); (b) one-sided defaults guard (JSON-present→==mirror; self-arms); (c) cross-repo FOLLOWUP for sibling v5 emission. **[FOLDED.]**

## IMPORTANT
**I1 — S1 mutation wrong: full `'?h?` REDs the CURRENT suite** (L12 apostrophe suffix-origin row `canonicity_drift.rs:119` covers the `'` half). The genuine blind spot = `h?`-ONLY (keep `'?`): under it all 26 classifier-touching tests stay green + the new row REDs. Regex loci `conditional.rs:110/112/114`, suffix group = 2nd `(?:/\d+'?h?)*` after `@\d+`. Fixture Expect=canonical confirmed. **[FOLDED.]**
**I2 — allowlist keying:** `DEFAULT_VALUE_ALLOWLIST` keyed `(subcommand,flag)`; subcommand names collide (md/ms/mk all `encode`) → must key `(cli,subcommand,flag)`. **[FOLDED.]**

## MINOR
M1 `resolvable()` hardcoded to mnemonic → per-CLI variant in tests/, not src/ (NO-BUMP). M2 fixture-count 25→26. M3 local binaries newer than CI pins → arbitrated by schema-mirror.yml's exact-pin install (no workflow change). M4 optional unit companion row. M5 drop stray "--schema" (all siblings expose gui-schema). **[M1/M2/M5 folded; M3/M4 informational.]**

## VERDICT: OPEN (1C/2I). S1 fixture ship-ready once mutation reworded; S2 rescoped to choices-only + self-arming defaults guard + cross-repo FOLLOWUP.

---
**FOLD STATUS (opus, 2026-07-11):** C1 (S2→choices-only + one-sided defaults guard + 3-category STOP + FOLLOWUP), I1 (h?-only mutation), I2 (cli-keyed allowlist), M1/M2/M5 folded. Convergence R0 re-dispatched.
