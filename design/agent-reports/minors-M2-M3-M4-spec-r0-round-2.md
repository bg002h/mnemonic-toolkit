# R0 convergence (round 2) — `SPEC_minors_M2_M3_M4.md` — Fable, adversarial

**Persisted per CLAUDE.md.** Round-1's 3 Importants + 3 Minors verified CLOSED by the folds (I1 both-modes exact — `:418`+`:1363` are the only two success-exits, `round1_verifications` flows past `:1363` unchecked; I2 the two exit-0 tests confirmed [cell_5 genuine flipped-sig fixture, encrypted case]; I3 `.examples-build` FATAL + `crates/**` trigger confirmed, corpus 0 content-overlap; version sites complete + accurate). BUT the I1 fold opened a new gap:

## Findings
**Important:**
- **I4 (fold-introduced):** the combined-blob arm (`:1363`) exit-4 change ships UNTESTED — the plan lists only the two standalone flips; the only combined test (cell_13, `cli_bsms_round1.rs:287-314`) uses a VERIFIED record, so a `:1363`-check-revert survives the suite (violates the RED-under-mutation standard for the arm the SPEC calls "arguably worse"). Fix: add `combined_blob_round1_lenient_failed_exits_4` (blob `tests/fixtures/wallet_import/bsms-1of1-singlesig.txt` + cell_5's flipped-sig TV1 → `.code(4)` + envelope emitted). **[FOLDED — M4 loci + acceptance #3.]**

**Minor:**
- **m4:** m1 fold premise WRONG — `bip39` is ALREADY a runtime `[dependencies]` entry (`Cargo.toml:52`, `all-languages`) + tests already `use bip39` (`cli_derive_child.rs:390`). Drop the dev-dep add (no dep change; no Cargo.lock delta). **[FOLDED — M3 + release ritual + acceptance #5.]**
- **m5:** `gen.sh` version pins also at `:724` (prose) — list it. **[FOLDED.]**

## VERDICT round 2: OPEN (0C / 1I) → folds applied, re-dispatch round 3.

---
**FOLD STATUS (opus, 2026-07-11):** I4 (combined-mode test added to M4 loci + acceptance #3), m4 (bip39-already-a-dep corrected in M3 + ritual + acceptance #5 — no dep change), m5 (gen.sh:724) all folded. Round-3 convergence R0 re-dispatched.