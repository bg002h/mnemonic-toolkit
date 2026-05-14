# v0.13.0 Phase 1c (triad) — SLIP-39 error + wordlist + rs1024 R1 reviewer report

**Phase:** P1c (triad) — `Slip39Error` enum + 1024-word wordlist embedding + RS1024 BCH checksum
**Round:** R1 round 2 (verify-the-fix)
**Reviewer:** Opus (`feature-dev:code-reviewer`)
**Date:** 2026-05-14

## Round 2 verdict

**0 Critical / 0 Important / 0 Nice-to-have — R1 LOCK round 2.**

Doc-only fold commit `0586fa2` lands the three accepted findings verbatim against the round-1 recommendation. Cryptographic substance and data integrity already validated in round 1; no code paths touched in this fold.

## Round 1 recap

- 0 Critical / 1 Important (I-1: `error.rs:10-13` misattributed one of the 15 library variants as a "CLI-only omission") / 3 Nice-to-have (N-1 stale upstream path, N-2 intentional forward reference, N-3 undocumented defensive mask).
- I-1 + N-1 + N-3 folded inline at `0586fa2`. N-2 left unchanged as intentional doc-as-contract (next-phase reader hint, not staleness).
- All cryptographic primitives (GEN constants, polymod, customization-string handling, checksum unpack order, wordlist data) verified against SLIP-0039 §3.5 spec and python-shamir-mnemonic `17fcce14`.

## Fold verification

**I-1 — `error.rs:10-19` — VERIFIED.**

Compared `error.rs:10-19` against the round-1 report's recommended rewrite block. Substantively identical, including: "15 library variants spanning 16 of the 18 SPEC §B.2.5 refusal classes"; explicit identification of rows 17 and 18 as the CLI-only pair (`--from` syntactically invalid; multi-stdin contention across `--share` / `--from` / `--passphrase-stdin`); explicit fold explanation (rows 4 and 5 → `BadGroupSpec`); and the (n, t) distinguisher (`row 5 = n == 1 && t == 1`). The off-by-N narrative pattern is corrected; the doc no longer counts the row-4/5 fold as a "CLI-only omission."

**N-1 — `wordlist.rs:3` — VERIFIED.**

Path corrected from `python-shamir-mnemonic/wordlists/wordlist.txt` to `python-shamir-mnemonic/shamir_mnemonic/wordlist.txt`. The SHA-pin (`17fcce14`) and content-hash anchor are unchanged. Path now matches upstream tree exactly.

**N-3 — `rs1024.rs:60-65` — VERIFIED.**

The new 5-line comment explains: (a) what the mask does (defensive 10-bit clamp), (b) why Python omits it (arbitrary-precision ints), (c) why Rust needs it (`u16` allows 0..=0xFFFF), and (d) the failure-mode rationale (truncate vs silently corrupt the LFSR's subsequent rounds). Accurate against the spec and the Rust idiom; matches the round-1 recommendation.

**N-2 — left unchanged — accepted as intentional doc-as-contract.**

The `parse_slip39_share` forward reference in `error.rs:27` (slightly renumbered after the rewrite) was correctly left as a forward-looking contract for P1c-D.

## No regressions

Doc-only fold; no source-of-truth code paths or test surfaces touched. The three edited regions are `//!` module-level comments and a `//` inline comment. They contain no doc-tests (no fenced code blocks) and cannot affect `cargo test` or `cargo clippy`. Test counts (20 / 11 / 15) and clippy cleanliness are preserved by construction; the main-session ran the triad tests post-fold and confirmed 46/46 green + clippy clean.

## No new findings

Re-skimmed all three files (`error.rs`, `wordlist.rs`, `rs1024.rs`) end-to-end. The doc-only edits introduced no new accuracies-to-verify, no semver risk, no test gap. Variant data shapes, Display branches, GEN constants, polymod arithmetic, wordlist invariants — all unchanged from the round-1 LOCK-eligible state.

## R1 LOCK

The P1c (triad) module is ready for P1c-D (`share.rs` + parse harness) dispatch.

v0.13.0 P1c (triad) R1 LOCK round 2.

## References

- Round 1 report (overwritten by this file): preserved at `0586fa2^:design/agent-reports/v0_13_0-slip39-triad-r1.md`
- Doc-fold commit: `0586fa2`
- [SLIP-0039 specification (satoshilabs/slips)](https://github.com/satoshilabs/slips/blob/master/slip-0039.md)
- [python-shamir-mnemonic commit 17fcce14](https://github.com/trezor/python-shamir-mnemonic/commit/17fcce14)
