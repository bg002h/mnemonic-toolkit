# v0.11.0 P1 R1 — final-word library impl reviewer-loop (Opus)

**Date:** 2026-05-13.
**Reviewer:** Opus (per `feedback_opus_primary_review_agent`).
**Scope:** R1 impl-review on the v0.11.0 Phase 1 library deliverable.
**Verdict:** LOCK (round 1, clean). 0 critical / 0 important / 4 nits (all deferred or low-confidence).

## Artifacts reviewed

- Implementation: `crates/mnemonic-toolkit/src/final_word.rs`
- Library shape change: `crates/mnemonic-toolkit/src/lib.rs`
- Tests: `crates/mnemonic-toolkit/tests/lib_final_word.rs`
- Commits: `9a6cd76` (RED) → `b3ced29` (GREEN)

## Design pivot endorsed

P1 GREEN pivoted from R0's "library returns `Result<Vec<&'static str>, ToolkitError>` consuming `CliLanguage`" to a self-contained library surface with library-local `FinalWordLanguage` + `FinalWordError` enums. Rationale was captured in the GREEN commit message: exposing `error`/`language`/`friendly` from `lib.rs` would create either E0428 conflicts with main.rs's `mod error;` declarations OR type-mismatch between `mnemonic_toolkit::error::ToolkitError` and `crate::error::ToolkitError` paths throughout the binary's modules. The cleaner alternative (move the modules to lib.rs entirely) is an out-of-scope crate-shape refactor.

Reviewer endorsement (verbatim from R1):

> The pivot (introduce `FinalWordLanguage` + `FinalWordError` rather than re-using `crate::language::CliLanguage` + `crate::error::ToolkitError`) is **sound**.
>
> 1. R0 SPEC §2.1 was written assuming `error`/`language` lived in lib.rs. They don't.
> 2. Cycle B SPEC §4 P2 Option C explicitly carved out `mlock` as the FIRST exposed lib module on the principle that lib types should be self-contained and not pull in binary-private error routing. `final_word` correctly follows that precedent.
> 3. Duplication cost is bounded (10 frozen BIP-39 languages, 2 error variants).
> 4. P2's CLI-boundary wrapper will be ~20 LOC of mechanical mapping.

A FOLLOWUP entry `library-error-and-language-surface-promotion` filed at `design/FOLLOWUPS.md` for the future cleaner lib-shape refactor — explicitly future-cycle scope, NOT a P1 blocker.

## Verification

R1 independently grep-verified:

- **bip39 v2 API**: `Language::word_list() -> &'static [&'static str; 2048]` confirmed (docs.rs/bip39/2.1.0). `Error` variants `BadEntropyBitCount`, `BadWordCount`, `UnknownWord(usize)`, `InvalidChecksum`, `AmbiguousLanguages` all confirmed. `parse_in` (language-pinned) cannot return `AmbiguousLanguages` — only `parse` (auto-detect) can. The impl correctly uses `parse_in`.
- **Algorithm correctness**: matches SPEC §2.1 verbatim. Pure naïve enumeration over 2048 entries via `parse_in` as oracle. Output deterministic by N (128/64/32/16/8) — verified against 5 happy-path tests + 2 anchor SHA pins. Math independently verified at R0 round 1.
- **Test coverage**: all 13 prescribed tests present (3 abandon anchors + 3 beef anchors + 5 per-N happy-paths + 4 refusals + 1 determinism + 1 cross-language = 17 total). Sortedness invariant asserted. SHA-pin regression backstops in place.
- **Doc-comment accuracy**: module + function docs accurately describe inputs/outputs/errors. No drift from impl.
- **FinalWordError completeness**: 2 variants suffice. `BadWordCount`/`InvalidChecksum` from bip39 cannot escape (impl pre-validates count + filters checksum failures silently). `AmbiguousLanguages` cannot arise from `parse_in`. `BadEntropyBitCount` only from entropy-construction path, never from `parse_in`.
- **Anchor SHA pin durability**: durable. Wordlist BIP-39-frozen; SHA-256 unchanged; joined-by-`\n` byte representation has no pointer-level concerns. Test file documents the pin-update protocol for hypothetical wordlist normalization fixes.

## Nits (all deferred or low-confidence)

| ID | Title | Action |
|---|---|---|
| N1 | `Vec::with_capacity(128)` over-allocates for N≥15 | No action — ~1.9KB worst case; allocator paging dominates. |
| N2 | `wordlist.contains(w)` O(n) per word (~47K cmps worst case) | No action — microseconds; dwarfed by the 2048 SHA-256 ops in the main loop. |
| N3 | Anchor SHA durability conditional on bip39 wordlist stability | No action — documented in test file (pin-update protocol). |
| N4 | `From<CliLanguage> for FinalWordLanguage` not pre-provided in P1 | Deferred to P2 — the conversion site is closest to its call. |

## Ship gate

R1 LOCK round-1 clean → P1 ship gate satisfied. Convergence took 1 round (R0 took 3 rounds; the R0 effort paid off in R1 round-1 success).

Commit: `docs(design): v0.11.0 P1 R1 LOCK report + lib-shape FOLLOWUP`.

Next phase: P2 (CLI subcommand + lint anchors + R1 reviewer-loop). The R0 SPEC §2.2 narrative is locked; P2 implements verbatim with the `FinalWordLanguage`/`FinalWordError` boundary conversion documented above.
