# Post-impl whole-diff R0 — T2-b (#7) md-codec `bch_exhaustive_sweep` + PGZ `parity_smoke` — Fable, adversarial

**Persisted verbatim per CLAUDE.md.** Repo `/scratch/code/shibboleth/descriptor-mnemonic` @ `db0e1275`, uncommitted TEST-only diff. All claims verified by execution.

## 1. Green in my hands
- `cargo test -p md-codec`: exit 0, 23 binaries, **431 passed / 0 failed / 2 ignored**.
- `bch_exhaustive_sweep.rs`: 3 passed + 1 ignored, 26.25s (single-error 93×31, bounded 2-error 4278×8, tripwire 1e5). `full_two_error_sweep` in `--release`: ok, 33.56s.
- `parity_smoke.rs`: 3 passed, 3.03s.

## 2. PGZ true independence — CONFIRMED
`grep` over `mod pgz` (parity_smoke.rs:72-339): zero `use`/`super::`/`crate::`/`md_codec` tokens inside. Stage-verified: syndromes by direct GF(1024) Horner of the received word `P(x)=INIT·x^N+Σuᵢx^{N-1-i}+T(x)` at `β^{77+m}` (:200-217, not residue unpacking); largest-nonsingular-ν Hankel solve by Gaussian elimination (:256-275, σ re-indexing re-derived correct); direct root trial over the 93-orbit (:277-291); linear-system magnitudes w/ GF(32)-subfield check (:295-309); own exp/log tables from a searched primitive element (:114-172); own re-verify (:317-324). `md_codec` imports (:63-66) used only outside the module for the SUBJECT side (`md_bch` :372-389 replicates `chunk.rs`) + encoder input. Cell 1 validates the reference vs injected ground truth first ⇒ non-tautological. Both sides re-verify ⇒ accept-sets provably `{w: d(w,C)≤4}`; agreement is a theorem, disagreement ⇒ bug.

## 3. Local-constant probe — VALUES INDEPENDENTLY DERIVED, all match the wire definition
Standalone Python, no md code:
- `hrp_expand("md")=[3,3,0,13,4]` from BIP-173; local `hrp_expand_md()` (:175-185) matches.
- `POLYMOD_INIT=0x23181b3`: derived as the fold of `hrp_expand("ms")` from residue 1 = BIP-93 codex32 `ms32_polymod` init. md1 BIP (`bip-mnemonic-descriptor.mediawiki:317`) pins BIP-93 `ms32_polymod`; sibling ms-codec carries the same generator (`ms-codec/src/bch.rs:24`).
- `MD_CONST=0x0815c07747a3392e7`: reproduced from its NUMS rule, top-65 bits of SHA-256("shibbolethnums")=`0x815c07747a3392e7`; verbatim `T_REGULAR` in the BIP (:325).
- `β=8ζ` order 93; `J_START=77`: evaluated `g(x)=x¹³+GEN[0]` at all 93 orbit points → root set `{17,20,46,49,52,77-84}`; the 8-consecutive window at 77 is UNIQUE. BIP :342 states `{β⁷⁷…β⁸⁴}`. Nothing checked against md's copy.

## 4. Cap-relax substitution — JUSTIFIED; the chien L≤4 consistency IS the operative capacity gate
- (a) `deg>4`→`deg>8` (bch_decode.rs:432): probe count **10→10**; tripwire GREEN. The SPEC's literal mutation genuinely does not spike (dead RED-proof). Cause: BM returns a deg≤4 locator for ~91.6% of beyond-t words (length-8 sequence linear-complexity concentrates at 4) — chien consistency rejects them; the cap only pre-filters the deg-5..8 tail chien+Forney reject anyway.
- (b) Removing `error_degrees.len() != deg` in `chien_search` (:308-310) AND the defensive re-check (:438): tripwire **FAILED: "spiked: 91575 … exceeds pinned bound 50 (baseline 10)"** — the exact impl-claimed number. **Faithful gate, not a stand-in.** Residual (honestly documented, bch_exhaustive_sweep.rs:36-39): deleting `deg>4` alone stays uncaught — but by execution that deletion is behavior-invisible at 1e5 beyond-t trials, so there is nothing for a behavioral test to catch.

## 5. Other RED-proofs re-reproduced by execution
- Position-map `k=L-1-d`→`k=d` (:453): sweep FAILED at pos 0.
- Inconsistent shift `β^J`→`β^{J+1}` in `compute_syndromes_regular` (:202), Forney unchanged: "mined cell 0: md-codec must BCH-accept (miscorrect)" FAILED (md flips to reject; PGZ cells green — reference untouched). Mined cells are the load-bearing RED carrier.
- CONSISTENT shift (`REGULAR_J_START=78`, syndromes+Forney): REDs the sweep AND md's pre-existing unit tests (`one_error_decodes_correctly_regular`, `two_errors_decode_correctly_regular`). NOT over-fit: β^85 is not a generator root + the 8-window at 77 is unique ⇒ j_start has no gauge freedom ⇒ a consistent shift is a genuine functional regression, RED is correct. File doc (:57-61) names only the inconsistent shift.

## 6. Mined cells genuine
All 4: 5 distinct positions in 0..93, nonzero GF(32) magnitudes ⇒ Hamming distance exactly 5 (beyond t=4). Executed: md accepts, output≠original (real miscorrection), PGZ accepts the SAME miscorrected codeword, public `decode_with_correction` doesn't report `TooManyErrors`. Oracle = PGZ, not md's decode.

## 7. Acceptance bound
Re-measured at (L=93, seed `0x1234_5678_9ABC_DEF0`, e∈5..=8, 1e5): exactly 10 = pinned baseline. Deterministic (fixed-seed xorshift, zero flake). Bound 50 = 5× headroom vs mutation spike 91,575 = 1831× the bound — alive, not dead.

## 8. NO-BUMP + gates
`git diff crates/md-codec/src/` EMPTY; `git status` = ` M tests/parity_smoke.rs` + `?? tests/bch_exhaustive_sweep.rs`. fmt `rustup run stable cargo fmt --all --check` clean (ci.yml:56-59 floats stable); clippy `--workspace --all-targets -D warnings` exit 0 on 1.85.0; 1.85 compat under rust-toolchain.toml. Silent self-skip removal confirmed vs HEAD (old :84-94 returned green on HOME-unset/missing binary; new file always-on).

## Tree cleanliness
Every mutation reverted; probe file deleted. Final sha256 of parity_smoke.rs / bch_exhaustive_sweep.rs / bch_decode.rs byte-identical to pre-review.

## Findings
**Critical: none. Important: none.**
- Minor (informational): `deg>4` cap deletion alone uncaught — verified behavior-invisible at 1e5 beyond-t trials; substitution rationale documented (bch_exhaustive_sweep.rs:36-39).
- Minor (informational): parity cell 2's random accept-branch executes 0 times at this seed (≈0.4/4000 expected); accept-side agreement carried by the 4 deterministic mined cells (comment :521-524), confirmed load-bearing by the inconsistent-shift mutation.

**VERDICT: GREEN (0C/0I)**

---
**FOLD STATUS (opus, 2026-07-10):** 0C/0I → ready to ship. No fold needed. Awaiting the #8 long-guard fold + re-gate before the T2 bundle ship.