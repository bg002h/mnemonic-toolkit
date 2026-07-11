# Post-impl whole-diff R0 — T2-c (#8) mk-codec `bch_correct_ok_implies_valid` — Fable, adversarial

**Persisted verbatim per CLAUDE.md.** Repo `/scratch/code/shibboleth/mnemonic-key` @ `main` (uncommitted). All claims verified by execution; every mutation reverted; tree left byte-clean.

## 1. Green in my hands
`cargo test -p mk-codec`: **188 passed / 0 failed** (lib 157, bch_adversarial 4, **bch_correct_implies_valid 9/9**, canonical_payload 4, error_coverage 2, indel_reject_contract 2, proptest_roundtrip 4, round_trip 3, vectors 3). Re-ran clean after all reverts.

## 2. Oracle independence — CONFIRMED
`independent_polymod` (`tests/bch_correct_implies_valid.rs:54-67`) is a self-contained BIP-93 ms32 loop; never calls `bch_verify_*`/`polymod_run`. Shared items are DEFINITION constants: `GEN_REGULAR`/`GEN_LONG` BIP-93 generators (`bch.rs:173,225`, pinned by `generator_polynomial_evaluates_to_zero_at_specified_roots` `bch_decode.rs:681`), `POLYMOD_INIT` codex32 init (`bch.rs:207`), `MK_REGULAR_CONST`/`MK_LONG_CONST` derivation-pinned vs SHA-256(NUMS_DOMAIN) (`consts.rs:85-93`), `HRP_EXPAND_MK=[3,3,0,13,11]` hand-verified + pinned (`bch.rs:984`). Runtime decoder never touches GEN_*/INIT outside cfg(test).

## 3. RED-proofs re-reproduced by execution (mutate → RED → revert byte-clean)
| Mutation | Result |
|---|---|
| `bch.rs:451` regular re-verify → `if true` | `mined_reverify_regular_kats_imply_valid` **RED cell 0**; all others incl. random proptests GREEN → mined cells are the only reliable gate |
| `bch_decode.rs:566` drop `\|\| deg > 4` | `mined_cap5_regular_kats_bound_corrections` **RED** ("applied 5 > 4") |
| `bch_decode.rs:212` `REGULAR_J_START` 77→78 | `bch_correct_le4_is_unconditional_regular` **RED** at `.expect` (273) — an `Err` FAILS → non-vacuous |
Mined-cell genuineness (unmutated): 4 REVERIFY cells have ≥5 in-bounds nonzero injections, raw `decode_regular_errors` returns a 3-error spurious fit failing `bch_verify_regular`; 3 CAP5 cells have exactly-5 positions, return `None` under the unmutated cap.

## 4. PROBE — LONG-GUARD RESIDUAL: **NOT honest — the guard is constructively reachable** (key deliverable)
Constructed a guard-reaching long vector on the first attempt; proved (a) the shipped leg does NOT pin `bch.rs:504`, (b) a pinned cell WOULD.

**Why random mining failed but construction succeeds.** The guard fires iff the raw decoder returns a full consistent solution whose implied pattern does not reproduce the syndromes → requires BM's LFSR length l > deg(trimmed Λ); rare at random and for the long code suppressed by all roots landing in-range ((108/1023)^deg). But the 8-syndrome window's Frobenius closure ({1019..1022,0..3}→·32 mod 1023 → exactly g_long's 15 roots, matching `bch_decode.rs:62-66`) lets one build a GF(32) polynomial perturbing exactly ONE syndrome: **M(x)=g_long(x)/m₂(x)**, m₂ = minimal poly of the pair {γ^1019,γ^895}. M (deg 13, coeffs `[1,20,4,17,1,20,23,23,20,1,17,4,20,1]` for x^0..x^13) vanishes at 7 of 8 window points; δ=M(γ^1019)≠0 at `syndromes[0]` only.

**Recipe:** `data[i]=(i·7+3) mod 32` for i∈0..93; `c = data ‖ bch_create_checksum_long("mk",data)` (108 symbols); one real error `r[40]^=13`; then `r[107-d]^=M[d]` for d∈0..13. True weight = **15** (≥5 ground truth). Syndrome seq has linear complexity 2, unique connection poly `[1,λ₁,0]` → BM trims the trailing zero → deg-1 locator; Chien finds the single in-range root (real error); Forney returns the real magnitude. Raw `decode_long_errors` → `([40],[13])` (non-empty in-range ≤4 fit); applying it leaves residual syndrome (δ,0,…,0)≠0 → **fails re-verify**.

**Execution evidence:** Unmutated: raw → `Some([40],[13])`, fix fails `bch_verify_long`; `bch_correct_long` → `Err` (the `:504` guard rejects). 3/3 magnitude variants (13,5,30) reach the guard. Mutation (`bch.rs:504`→`if true`): shipped file stays **9/9 GREEN** (gap), constructed probe **RED** (`bch_correct_long("mk",&r).is_err()` fails; returned Ok.data fails independent re-verify). Vector executes through the fuzz binary cleanly (valid seed).

**Vector** (data_with_checksum, 108 symbols; raw fit positions [40] mag [13]):
```
[3,10,17,24,31,6,13,20,27,2,9,16,23,30,5,12,19,26,1,8,15,22,29,4,11,18,25,0,7,14,21,28,
 3,10,17,24,31,6,13,20,22,2,9,16,23,30,5,12,19,26,1,8,15,22,29,4,11,18,25,0,7,14,21,28,
 3,10,17,24,31,6,13,20,27,2,9,16,23,30,5,12,19,26,1,8,15,22,29,4,11,18,25,0,7,14,
 14,8,15,1,31,13,1,20,13,28,13,2,8,9]
```
**Ruling: "byte-identical logic" fallback REJECTED — real, fixable gap.** SPEC §T2-c scopes the mined-KAT RED-proof to BOTH sites (`bch.rs:451`/`:504`); the leg ships only regular cells.

## 5. Fuzz wiring — PASS
Builds under pinned `nightly-2026-04-27` (all 3 bins); smoke 623k execs/20s no findings. `fuzz/Cargo.toml` `[[bin]]` matches convention; `fuzz-smoke.yml` matrix matches; length-band selection matches `bch_code_for_length` (`bch.rs:117-124`).

## 6. NO-BUMP + gates — PASS
`git diff crates/mk-codec/src/` empty (after every revert + final); clippy `--workspace --all-targets -D warnings` clean; `cargo +1.95.0 fmt --all -- --check` clean (no mlock.rs in repo). Final tree = the 4 diff files. Removed all byproducts. (Out-of-scope: `design/RECON_T4_mk_external_oracle.md` appeared mid-review from a parallel session — untouched.)

## Findings
**Critical:** none.
**Important 1:** LONG re-verify guard (`bch.rs:504`) is deterministically RED-provable but unpinned — the "not RED-provable" residual is falsified. Under guard deletion the new test file stays 9/9 GREEN. Fix: add a `REVERIFY_LONG_CELLS` mined-KAT using the construction above (or regenerate via the M-polynomial recipe with varied data/positions/magnitudes); satisfies the SPEC §T2-c `:451`/`:504` clause. ~15 lines.
**Minor 1:** `fuzz-smoke.yml:83-86` stale "both/either" comment now above a 3-target matrix; new target has no committed seed corpus (both existing ship 4 seeds each). Update comment + consider committing the constructed vector as a seed (exercises the near-guard path).

## VERDICT: **OPEN (0 Critical / 1 Important)**
Regular-code legs excellent (all RED-proofs reproduce, oracle independent, mined cells genuine, nets non-vacuous). The single Important is precisely the targeted residual: the long-code guard is constructively reachable → pin it before ship. Vector + recipe + derivation in §4 for direct folding.

---
**FOLD STATUS (opus, 2026-07-10):** Important-1 BLOCKS the #8 ship (gate: no ship with an open Important). Folding — resumed the #8 implementer to add a long-guard KAT (≥2 constructed cells via the §4 vector + M-polynomial recipe; 3 magnitude variants) RED-proven under `bch.rs:504→if true`, + Minor-1 (fuzz-smoke comment + seed corpus). Post-fold RE-ENTERS the review loop (scoped convergence R0) before ship. #6 GREEN; #7 pending; T2 ships as a bundle once all three converge.