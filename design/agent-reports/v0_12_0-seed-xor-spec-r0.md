# v0.12.0 Phase 0 — Seed XOR SPEC R0 reviewer report

**Phase:** P0 — SPEC author + FOLLOWUP entry
**Round:** R0 round 1 (single-pass clean LOCK)
**Reviewer:** Opus (`feature-dev:code-reviewer`)
**Date:** 2026-05-14
**Document under review:** `design/SPEC_seed_xor_v0_12_0.md` (rendered from plan-mode §A at `~/.claude/plans/radiant-seeking-teacup.md`)
**Predecessor tag:** `mnemonic-toolkit-v0.11.0` (final-word completer, shipped at `f6c036a`)

## Verdict

**0 Critical / 0 Important / 1 Nice-to-have — R0 LOCK round 1.**

v0.12.0 Phase 1 (library impl) cleared to start pending user authorization.

## Scope reviewed

All 11 mandatory reviewer checks per the dispatch prompt:
- Critical: SPEC vs plan-mode artifact alignment; Coldcard byte-exact compatibility; library entry-point signatures compile; cross-ref paths in §5 still exist.
- Important: FOLLOWUP entry shape; lint-row count baseline (21 at v0.11.0); advisory wording byte-exactness; §2.5 row 9 wording vs single-stdin precedent; `schema_version: "1"` precedent; per-share output Zeroize discipline.
- Nice-to-have: worked-example presence.

## Key validations

1. **SPEC document byte-equivalent to plan §A** (modulo header reformat). Beneficial drift noted: §4 G5 in the standalone document correctly says "5 advisory classes" matching §2.6's 5-row table, while the plan §A.4 G5 said "4 advisory classes" against its own 5-row §A.2.6. Standalone SPEC resolves the plan-internal inconsistency without introducing new claims. Plan file remains R0-LOCKED at round 3; no plan-file backfill needed (inconsistency was within the plan, fixed during the standalone-SPEC render at this P0).

2. **Coldcard upstream verified.** `assert len(raw_secret) in (16, 24, 32)` quote-exact at `shared/xor_seed.py`. SHA256d-deterministic generation formula deferred to G1 vendor vector at P1 (correct SPEC-level abstraction).

3. **Library signatures compile.** `rand_core::CryptoRng + rand_core::RngCore` `impl`-bound is valid Rust (both are crate-root traits in `rand_core = "0.6"`; not a `dyn` shape so no E0225). `Zeroizing<Vec<u8>>` is the correct type for heap-zeroize-on-drop.

4. **All 9 §5 cross-ref paths grep-verified** against current toolkit source (file paths + line ranges all intact at v0.11.0 baseline).

5. **FOLLOWUPS entry well-formed.** `seed-xor-coldcard-compat` at `FOLLOWUPS.md:66-74` follows the file's prevailing precedent: `Status: open (...)`, `Tier: v0.12.0-feature`, `Companion: [[slip39-shamir-secret-sharing]]`. The non-Status `In flight:` annotation is within precedent (per the v0.13.0 plan I1 fold convention).

6. **Lint-row count baseline confirmed:** `lint_argv_secret_flags.rs` has exactly 21 `label:` rows at v0.11.0 baseline. SPEC §4 G6 "21 → 23" math correct (split + combine = 2 new rows).

7. **Per-share output Zeroize discipline correct.** `Vec<Zeroizing<Vec<u8>>>` zeroizes each inner heap on drop; the outer Vec's backing array contains only fat pointers + `Zeroizing` wrappers (no secret material). No outer-Vec wrap needed.

## Nice-to-have

**N1.** SPEC ships without a worked example — consistent with v0.11.0 SPEC precedent (manual chapter carries the worked example, authored at P3). Acceptable.

## R0 LOCK

v0.12.0 P0 SPEC R0 LOCK round 1. Phase 1 cleared to start.
