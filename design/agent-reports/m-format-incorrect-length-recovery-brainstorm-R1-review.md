# R1 Architect Review (post-fold) — BRAINSTORM_m_format_incorrect_length_recovery.md

**Round:** R1 (brainstorm-spec gate; folds of R0's 2 Important applied)
**Reviewer:** feature-dev:code-reviewer (opus)
**Date:** 2026-05-24
**Reviewed against:** `origin/master` = `925f5ed`, ms-codec 0.2.0, mk-codec 0.3.1.
**Persisted verbatim per CLAUDE.md.**

---

## Verdict: GREEN (0 Critical / 0 Important)

All four folds (I1, I2, M1, M2) are RESOLVED and verified correct against `origin/master` source. The folds introduced no new inconsistencies, and all previously-SOUND claims still hold.

## Fold verification

**I1 — mk1 multi-string per-chunk model — RESOLVED.**
- "single string only when bytecode ≤ 56 bytes": `mk-codec/src/consts.rs:33` `pub const SINGLE_STRING_LONG_BYTES: usize = 56;` confirmed. Spec cite `consts.rs:33` exact.
- "≈84 bytes → fragments of 53 + 35 bytes = two strings": `mk-codec/src/string_layer/pipeline.rs:19-20` verbatim. Confirmed.
- `MAX_CHUNKS = 32`: `consts.rs:42`. Confirmed.
- "108 symbols per chunk cap": `mk-codec/src/string_layer/bch.rs:24-31` long code 96–108 chars; `BCH(108,93,8)` `bch.rs:30`. Confirmed.
- Per-chunk atomic (D8): `repair.rs:690` `repair_card` `CardKind::Mk1` arm iterates chunks calling `repair_chunk_one(kind, i, chunk)?` at `repair.rs:700` — the `?` propagation IS the atomic-fail. Confirmed real. Spec cites `:690-708`, `:700` exact.
- `--mk1` repeating: `cmd/repair.rs:46-47` `#[arg(long, value_name = "MK1")] pub mk1: Vec<String>` — confirmed repeating (spec's `repeating: true` is a loose paraphrase; substance correct).
- Per-chunk validator `decode_string` (`bch.rs:650`) + reassembly `decode(&[&str])` (`key_card.rs:114`). Confirmed.
- Completeness: per-chunk model correct and complete for scope. A prefix-region mk1 indel (P1) operates per-chunk (each chunk is independently `mk1`-prefixed, validated by `decode_string`). The chunk-count is the number of supplied strings, not a within-chunk header for mk1 (unlike md1, whose count header IS corruptible — correctly deferred to FOLLOWUP(b) per `chunk.rs:77`). No residual Important gap.

**I2 — exit-code remap — RESOLVED.**
- Existing contract `cmd/repair.rs:122` `Ok(if total_repairs == 0 { 0 } else { 5 })` confirmed; doc `:12-14` "0 — all chunks already valid / 5 — at least one correction applied (REPAIR_APPLIED)". So 5 = correction applied, 0 = already valid. Exact.
- Routing unique indel recovery → 5 reuses the `total_repairs > 0 ⇒ 5` path with no override: consistent and override-free.
- `Ok(4)` ambiguous: sound, non-colliding. 4-family in `error.rs` (`BundleMismatch` `:474`, `ImportWalletSeedMismatch` `:496`, `XpubSearchNoMatch` `:513`) = "human-review / no single answer"; `Ok(4)` success-with-candidates is the same family semantically.
- `Repair(_) => 2` at `error.rs:507`; new unrecoverable variant maps there.
- Internal consistency across §6 table / §6 prose / §7 / §10 / §12: ALL agree on 0/5/4/2 with identical meanings. No contradiction left by the fold.
- §6 note that "5 is also the auto-fire short-circuit value at `repair.rs:982`" is accurate (`Err(ToolkitError::RepairShortCircuit { exit_code: 5 })` at `:982`) and correctly disambiguated.

**M1 — RepairError source order — RESOLVED.**
- `repair.rs:388+` source order: `EmptyInput` `:389`, `HrpMismatch` `:390`, `TooManyErrors` `:395`, `UnparseableInput` `:399`, `ReservedInvalidLength` `:406`, `UnsupportedCodeVariant` `:414`, `PostCorrectionDecodeFailed` `:426`. Matches §7 exactly. Not-yet-alphabetized note + `error-rs-retroactive-alphabetical-sort` reference accurate.

**M2 — ms1 sparse length set — RESOLVED.**
- `ms-codec/src/consts.rs:33` `pub const VALID_STR_LENGTHS: &[usize] = &[50, 56, 62, 69, 75];` confirmed exact, gaps of 6–7. mk1 dense `[14,93] ∪ [96,108]` via `mk-codec/.../bch.rs:24-25`.

## New issues introduced by the folds
None. The folds are additive and introduced no internal contradiction or citation drift.

## Regression check (previously-SOUND claims re-confirmed)
- t = 4 capacity: `ms-codec/bch_decode.rs:416` `if deg == 0 || deg > 4`; `mk-codec/bch_decode.rs:22` "O(t²) for t = 4"; `md-codec/chunk.rs:12`. 8 syndromes: `ms-codec/bch_decode.rs:190` `-> [Gf1024; 8]`. Confirmed.
- placeholder-then-decode reduces omission to one substitution: rests on `deg > 4 → None` (`ms-codec/bch_decode.rs:416`). Sound.
- delete-and-validate residue==0: `repair.rs:559` dispatch + residue path present.
- no erasure primitive: grep "erasure" (case-insensitive) in ms-codec/src and mk-codec/src → zero matches. Confirmed by grepping the term itself.
- FP floor ~32⁻¹³: 13-symbol regular checksum (`bch.rs:28`), 15-symbol long (`bch.rs:30`). Sound.
- default --max-indel 0 leaves auto-fire untouched: auto-fire `try_repair_and_short_circuit` (`repair.rs:962`) calls `repair_card` with NO budget parameter — indel search is a separate standalone-subcommand path. Sound.
- SemVer PATCH + mandatory GUI schema_mirror + manual lockstep: consistent with CLAUDE.md mirror invariants.
- --max-indel non-secret: `secrets::flag_is_secret` (`secrets.rs:49-64`) is a closed allowlist; `--max-indel` absent → non-secret. Confirmed.
- Oracles: `ms_codec::decode_with_correction` (`decode.rs:188`, ascending corrections, empty vec = valid); `mk_codec::string_layer::bch::decode_string` (`bch.rs:650`); `mk_codec::decode(&[&str])` (`key_card.rs:114`). Confirmed.
- Ancillary: `inspect.rs:195` byte_length; `repair.rs:28` ALPHABET; `install.sh:32` self-pin (v0.37.0); md count header `chunk.rs:77` `(r.read_bits(6)? + 1)`. Confirmed.

## Residual Minor (optional, non-blocking)
- §3 / §11 use the loose phrase `--mk1` "`repeating: true`" — actual clap mechanism is `Vec<String>` with `#[arg(long)]`. Substance correct. Cosmetic; do not block.
- §12 item 5 already mandates re-grepping all `:line` cites at plan-doc write time (correct CLAUDE.md discipline); citations accurate as of `925f5ed`.

The spec is cleared to proceed to the implementation plan-doc (which then faces its own mandatory R0).
