# v0.13.0 Phase 0 — SLIP-39 SPEC R0 reviewer report

**Phase:** P0 — SPEC author + FOLLOWUPS in-flight annotation
**Round:** R0 round 1 (clean LOCK)
**Reviewer:** Opus (`feature-dev:code-reviewer`)
**Date:** 2026-05-14
**Document under review:** `design/SPEC_slip39_v0_13_0.md` (rendered from plan §B at `~/.claude/plans/radiant-seeking-teacup.md`)
**Predecessor tag:** `mnemonic-toolkit-v0.12.0` (seed-xor splitter, shipped at `63b4503`)

## Verdict

**0 Critical / 0 Important / 3 Nice-to-have — R0 LOCK round 1.**

v0.13.0 Phase 1a (math primitives: GF(256) + Lagrange) cleared to start.

## Scope reviewed

All 13 mandatory reviewer checks per the dispatch:
- Critical: SPEC vs plan §B alignment; crypto-primitive correctness against SLIP-0039 spec (GF(256), 4-round Feistel, PBKDF2 formula, digest construction, hierarchy bounds); vectors.json mapping correctness; library entry-point signatures; cross-ref paths.
- Important: FOLLOWUPS in-flight annotation; refusal-table row count consistency; lint-row count baseline math; subcommand-count math; Share Zeroize design; JSON envelope schema; iteration-exponent advisory threshold; G8 smoke-test location.

## Key validations

1. **SPEC vs plan §B alignment.** Differential clean. Standalone document drops the §B.* prefix; tables byte-equivalent to plan §B. No drift introduced.

2. **Crypto-primitive correctness against SLIP-0039 spec.** WebFetch of spec confirmed all 6 claims:
   - GF(256) Rijndael polynomial x^8 + x^4 + x^3 + x + 1: matches.
   - 4-round Feistel with PBKDF2-derived round keys, no AES: matches.
   - PBKDF2 formula `iterations = 10000 × 2^E` (= `2500 << e` per round × 4 rounds): matches.
   - Identifier 15 bits + iteration_exponent 4 bits: matches.
   - Digest = `HMAC-SHA256(key=R, msg=S)[0:4]` over UNENCRYPTED master secret S; `(digest || R)` payload Shamir-shared: matches.
   - Max group_count = 16, member_count ≤ 16 per group: matches §2.5 row 4.

3. **vectors.json mapping correctness.** Full 45-vector enumeration verified via WebFetch: 15 positive (indices 0, 3, 16, 17, 18, 19, 22, 35, 36, 37, 40, 41, 42, 43, 44) + 30 negative. All §2.5 row → vectors.json citations decode correctly under 1-based file-order indexing (e.g. row 7 cites `#6, #25` → 0-based array slots `#5, #24` → "different identifiers" 128/256-bit pair).

4. **Library entry-point signatures.** `rng: &mut (impl rand_core::CryptoRng + rand_core::RngCore)` matches v0.12.0 `seed_xor.rs:82` precedent. `Share` struct's `#[derive(Zeroize, ZeroizeOnDrop)]` with `#[zeroize]` on value-bytes + `#[zeroize(skip)]` on 8 metadata fields is structurally sound per `zeroize_derive`'s attribute syntax.

5. **All 10 §5 cross-ref paths grep-verified** at v0.12.0 post-PE HEAD (lines listed in dispatch all intact).

6. **FOLLOWUPS in-flight annotation well-formed.** Status remains `open`; In flight line adds P0–PE roadmap + plan cross-cite + hand-roll-path-locked rationale + vectors.json vendor note; Tier re-scoped from `v1+` to `v0.13.0-feature`; Companion line back-points to resolved `seed-xor-coldcard-compat`. Mirrors v0.12.0 P0 R0 LOCK precedent.

7. **Refusal-table row count triple-consistent.** §2.5 enumerates 18 numbered rows; header says "18 classes"; G5 says "All 18 refusal classes". Match.

8. **Lint-row count baseline math.** `lint_argv_secret_flags.rs:178-187` asserts `CANONICAL_FLAG_ROWS.len() == 23` at v0.12.0 baseline; SPEC G6 says 23 → 27 (+4 rows). Correct.

9. **Subcommand-count math.** 8 variants in `src/main.rs:41-58`; SPEC G7 says "bumps from 8 to 9". Correct.

10. **Iteration-exponent advisory threshold.** E ≥ 5 yields 320,000 PBKDF2 iters (≈200-500ms commodity x86); E=10 yields 10.24M iters (≈34s on Raspberry Pi 3 at 300K iter/sec). "E >= 10 may exceed 30s on weak hardware" wording defensible.

## Nice-to-have findings (non-blocking, for P1c fold-in)

**N1.** SPEC §2.5 "vectors.json #N" citations follow 1-based file-order convention while the actual JSON array is 0-indexed. Add a one-line footnote at P1c (alongside the `tests/lib_slip39_vectors.rs` harness) pinning the convention: "Citations follow the 1-based file-order convention of `python-shamir-mnemonic/vectors.json`; subtract 1 for array-index access in `lib_slip39_vectors.rs`." Non-blocking for R0.

**N2.** SPEC ships without a worked example — consistent with v0.11.0 + v0.12.0 precedent (manual chapter carries the example, authored at P3). Acceptable.

**N3.** §2.5 row 5 `--group 1,1` refusal is spec-permissible but stricter than spec — clearly labeled as "toolkit policy", not framed as spec authority. Acceptable.

## R0 LOCK

v0.13.0 P0 R0 LOCK round 1. Phase 1a (GF(256) Rijndael + Lagrange interpolation primitives) cleared to start. P1a deliverables per plan: `src/slip39/{mod, gf256, lagrange}.rs` + `tests/lib_slip39_math.rs` with field-axiom property tests + Lagrange-at-spec-test-vectors pinned; TDD RED-first, GREEN follows.

The three nice-to-have items (vectors.json indexing footnote at P1c, worked-example deferred to manual, toolkit-policy framing) are filed for the natural fold sites during the implementation phases. None block any phase boundary.
