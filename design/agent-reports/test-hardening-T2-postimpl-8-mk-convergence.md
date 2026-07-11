# Convergence R0 — mk #8 fold: LONG re-verify guard (`bch.rs:504`) pin — Fable, adversarial

**Persisted verbatim per CLAUDE.md.** Scoped convergence on the fold that closed the prior post-impl Important (long-guard constructively reachable). Repo `/scratch/code/shibboleth/mnemonic-key` @ `main`, uncommitted. Verified by execution; tree byte-clean.

## Execution evidence
**1. Green — PASS.** `cargo test -p mk-codec` = **189 passed / 0 failed** (was 188). New `mined_reverify_long_kats_imply_valid`; file 9→10 tests.

**2. The 3 cells are GENUINE guard-reaching constructions — PASS (independently reconstructed in a scratch crate outside the repo).** `data[i]=(i·m+a) mod 32` → `bch_create_checksum_long` → one real error → `r[107-d] ^= M_POLY[d]`. Per cell: reconstruction byte-identical to each pinned literal (`bch_correct_implies_valid.rs:416-441`); cell0 byte-identical to the reviewer's §4 vector; true error weight = 15 (≥5, beyond t=4); off-codeword per the independent oracle; raw `decode_long_errors` (residue via own polymod) → non-empty in-range ≤4 fit = `([40],[13])`/`([40],[30])`/`([55],[5])`; applying the fit fails both `bch_verify_long` and the independent oracle; `bch_correct_long("mk",&r)` = `Err(BchUncorrectable)` unmutated (the `:504` guard is the sole rejector).

**3. Non-tautology — PASS by mutation.** `bch.rs:504`→`if true`: RED at `:463` on the **independent-oracle implication** (`:461-467`), NOT the `is_err` pin (`:471-475`) — panic "returned Ok whose data fails independent re-verification — the re-verify guard at bch.rs:504 was bypassed". A mutated-source probe proved all 3 cells individually return Ok-unverifiable (corrections=1 each) — redundancy real. Reverted; `bch.rs` sha256 matches pre-mutation; suite GREEN.

**4. `is_err()` pin not over-fit — PASS.** The minimal BM fit for each vector fails re-verify ⇒ by uniqueness of bounded-distance decoding on the 8-syndrome window, no codeword lies within distance ≤4 ⇒ refusal is semantically forced; no legitimate refactor false-REDs the pin.

**5. Oracle independence — PASS.** `independent_verify_long` (:81-88) drives `independent_polymod` (:55-68): self-contained BIP-93 ms32 loop seeded `POLYMOD_INIT`, `== MK_LONG_CONST`; calls neither `bch_verify_long` nor `polymod_run`; shares only `pub const` code-definition constants. Scratch reimplementation agreed on every check.

**6. Minor-1 — PASS.** `fuzz-smoke.yml` "All three mk targets" + matrix adds the target; `fuzz/Cargo.toml` `[[bin]]` matches; corpus seed `long_guard_probe_cell0` = 108 raw bytes byte-identical to cell0; executed through the real fuzz binary (nightly-2026-04-27) clean.

**7. NO-BUMP + gates — PASS.** `git diff crates/mk-codec/src/` EMPTY (hash-verified); clippy `--workspace --all-targets -D warnings` clean; `cargo +1.95.0 fmt --all -- --check` clean; no stray `_verify_long.rs`; tree = the 4 fold files + corpus dir (+ out-of-scope RECON_T4, untouched).

## Findings
Critical: none. Important: none — the prior Important is genuinely closed. Minor: none (observational: the KAT loop panics on the first failing cell, but all 3 proven individually guard-reaching).

**VERDICT: GREEN (0C/0I)**

---
**SHIP (opus, 2026-07-10):** #8 fold converged GREEN. All T2 legs GREEN (#6 toolkit, #7 md, #8 mk). Proceeding to the T2 bundle ship (3 direct-FF commits, NO-BUMP, no tags).