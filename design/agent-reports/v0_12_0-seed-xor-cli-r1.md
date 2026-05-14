# v0.12.0 Phase 2 — Seed XOR CLI R1 reviewer report

**Phase:** P2 — CLI surface (`mnemonic seed-xor split/combine`)
**Round:** R1 round 1 (clean LOCK)
**Reviewer:** Opus (`feature-dev:code-reviewer`)
**Date:** 2026-05-14
**Commit under review:** `48241e7` (P2 GREEN)
**Predecessor:** `d8a5f30` (P1 R1 LOCK)

## Verdict

**0 Critical / 0 Important / 3 Nice-to-have — R1 LOCK round 1.**

Phase 3 (manual chapter + cli-subcommands list) cleared to start.

## Scope reviewed

All 16 mandatory reviewer checks per the dispatch — split into 6 Critical-class
(Cycle A discipline, Cycle B discipline, Zeroizing wraps, cardinality
assertion, multi-stdin refusal, per-share BIP-39 checksum recompute) and
10 Important-class (refusal/advisory wording, JSON envelope shape +
SHA pins, subcommand wiring, lint anchors, `bip39::Error` mapping,
permission-mode test mechanic).

Files reviewed:
- `crates/mnemonic-toolkit/src/cmd/seed_xor.rs` (~446 LOC)
- `crates/mnemonic-toolkit/src/cmd/mod.rs` + `src/main.rs`
- 5 CLI test files (44 tests total, all passing)
- `crates/mnemonic-toolkit/tests/cli_gui_schema.rs` (6 → 7 user-facing subcommands)
- `tests/lint_argv_secret_flags.rs` (+2 rows, 21 → 23) + `tests/lint_zeroize_discipline.rs` (+1 row)

## Key validations

1. **Cycle A discipline correct.** `secret_in_argv_warning` fires per-occurrence
   for every inline `--from phrase=<v>` AND every inline `--share phrase=<v>`
   (NOT deduped). Verified by `combine_inline_share_emits_argv_leakage_advisory_per_share`
   asserting count == 2 for 2 inline shares.

2. **Cycle B discipline correct.** mlock Site 1 pins on: master phrase bytes
   (line 130), parsed entropy (line 145), each parsed share string (line 259),
   recovered entropy (line 294). 4 pins total — bounded by max --shares value.

3. **Zeroizing wraps complete.** Every secret-bearing local wraps:
   master_phrase, entropy, share_phrases, share_strings, share_entropies,
   recovered, phrase. No bare String/Vec<u8> secret carriers.

4. **`--shares N` cardinality hard refusal.** Lines 229-234 enforce
   `--share count == --shares N`; test `refusal_combine_cardinality_mismatch`
   asserts the exact stem `"requires exactly 3 --share arguments"`.

5. **Multi-stdin refusal with defense-in-depth.** Primary count-based check
   at lines 220-226 + defensive per-share check at lines 248-253. Both
   surface the same refusal stem.

6. **Per-share BIP-39 checksum recompute.** Lines 157-164 wrap each raw-
   entropy share via `Mnemonic::from_entropy_in(lang, ...)` before emit.
   Round-trip tests at 12/15/18/21/24-word verify correctness.

7. **JSON envelope SHA-pinned anchors at GREEN:**
   - abandon×12 N=2 deterministic: `d368c70aabb6d3bab7d75b79f8a61a8340db6ac94c57250db6354fe235861af3`
   - Trezor legal×12 N=3 deterministic: `85d53f7e83db167b1223b8b23bbe2baca060e7aefad50f6034b5b65750883871`

8. **`#[cfg(unix)]` permission-mode test mechanic correct.** `NamedTempFile`
   kept alive through CLI invocation (`std::fs::write` opens with O_TRUNC,
   preserving the pre-set 0o644 mode). Avoids the v0.11.0 P2 `drop(f)`
   foot-gun where the temp file got deleted before CLI re-created under
   umask 022.

## Nice-to-have findings (non-blocking)

**N1.** Code clarity OK; helpers `entropy_bytes_to_word_count`,
`map_seed_xor_error`, `emit_world_readable_advisory`, `write_split_json` +
`write_combine_json` are right-sized. Minor duplication between
`write_split_json` and `write_combine_json` is acceptable (different
struct types).

**N2.** Test count: 11 happy-path + 8 JSON + 11 refusal + 9 advisory + 5
stdin = 44 CLI tests (slightly higher than the 42 cited in the commit
message; all passing). Coverage matches SPEC §4 acceptance gates.

**N3.** SPEC §2.5 row 5 (mixed-language shares refuse) is collapsed into
row 6 architecturally — the single `--language` flag makes per-share
language-divergence surface as a BIP-39 parse failure (row 6/7). If a
future revision wants per-share language detection, that'd be a follow-on
with a `bip39::Language::detect_per_share` path. SPEC narrative ambiguity
flagged but not blocking (the implementation correctly handles all
mixed-language inputs via the existing row-6/7 path).

## R1 LOCK

v0.12.0 P2 R1 LOCK round 1. Phase 3 (manual chapter + cli-subcommands
list + glossary count bump + R1) cleared to start.
