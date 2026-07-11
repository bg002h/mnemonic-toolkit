# R0 review — `SPEC_minors_M2_M3_M4.md` (round 1) — Fable, adversarial

**Persisted verbatim per CLAUDE.md.** Toolkit v0.84.0. External spec texts fetched fresh.

## Load-bearing probes — CLEARED
- **M2 (SLIP-39):** `slip39.rs:757` string confirmed; SLIP-0039 spec verbatim "There is no way to verify that the correct passphrase was used" — digest is passphrase-independent (checked at Shamir interpolation, pre-decryption). Lockstep loci exact (`cli_slip39_refusals.rs:336`, `41-mnemonic.md:2194`); 3 sites total, zero GUI/manual-gui/examples hits. Proposed message factually correct.
- **M3 (BIP-85 Portuguese):** bip-0085.mediawiki §BIP39 — Portuguese IS `9'` (after Czech `8'`); test vectors English-only → divergence oracle is the honest best. `derive_child.rs:342-358` refuses; `language.rs` maps Portuguese↔9 at :66/:86/:107/:131. No lockstep (portuguese already in `--language` enum, GUI schema `mnemonic.rs:175`).
- **M4 exit-4 (LOAD-BEARING):** `import_wallet.rs:418 Ok(0)` unconditional; lenient loop only `Err`s when strict; strict=exit 2 abort. **Exit 4 already used (`ImportWalletSeedMismatch`, `overlay.rs:207`) but blob-mode-only + same VERIFY-ME semantic class → non-colliding; 2/3/5 taken → 4 correct.**

## Findings
**Critical:** none.
**Important:**
- **I1** — M4 scope gap: the combined `--blob`+`--bsms-round1` tail (`:1363 Ok(0)`) ALSO exits 0 on `Failed`; user decision "any Failed" has no mode qualifier → extend any-Failed→4 to `:1363` too (recommend), or explicitly scope + rationale. **[FOLDED — both modes.]**
- **I2** — two existing tests pin old exit-0 & will break: `cli_bsms_round1.rs:110-131` cell_5 (`.code(0)`; cell_5 IS the flipped-sig fixture, no new fixture) + `cli_import_wallet_bsms_encrypted.rs:422-468` (`.success()`). Both →`.code(4)`. **[FOLDED — added to M4 loci.]**
- **I3** — release ritual omits the `.examples-build/` version site: `gen.sh:44` FATALs unless binary=0.84.0; `examples.yml` push trigger includes `crates/**` → v0.85.0 bump CI-FATALs (banked Cycle-A gotcha). Must bump gen.sh pins + regen Examples.md + Examples.pdf. **[FOLDED — mandatory ritual step.]**
**Minor:**
- **m1** — `bip39` not a dev-dep → add `bip39 = {version="2", features=["all-languages"]}`; don't port the Japanese `!is_ascii` check (Portuguese all-ASCII → spurious-RED). **[FOLDED.]**
- **m2** — fold the resolved exit-4 non-collision fact (don't leave "implementer must verify" open). **[FOLDED.]**
- **m3** — GUI run-modal will present lenient-Failed as failed at the next pin bump (desired, lagging) — CHANGELOG note. **[FOLDED into the CHANGELOG callout.]**

## VERDICT: OPEN (0C / 3I) → folds applied, re-dispatch for convergence
Both load-bearing probes cleared; the 3 Importants fold without re-litigating the user decision (I1 scope ruling = both modes; I2/I3 mechanical). SPEC updated.

---
**FOLD STATUS (opus, 2026-07-11):** I1 (both modes `:404-419`+`:1363`), I2 (2 test flips added to loci), I3 (.examples-build regen mandatory), m1 (bip39 dev-dep + no-is_ascii), m2 (exit-4 fact resolved), m3 (CHANGELOG GUI note) all folded. Acceptance #3/#5 updated. Convergence R0 re-dispatched.