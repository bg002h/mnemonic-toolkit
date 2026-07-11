# SPEC Convergence R0 — SPEC_toolkit_repin_sh_wpkh.md (round 2) — Opus

**Persisted verbatim per CLAUDE.md** (+ opus fold at end). Verdict: **OPEN (0C / 1I + 3M)**. Diffed against the `md-codec-v0.41.0` TAG (`546346c1`), NOT the contaminated working tree.

## C1/M4 rejection UPHELD (the load-bearing adjudication — SOUND)
At the tag, `error.rs` = `MalformedPayloadPadding`×1, `EmptyOriginOverride`×**0**; `DecodeOpts`/`*_with_opts`/`unresolved_origin_indices` = **0**. Those are the concurrent Track-A P0's uncommitted 0.42.0 work. **No wrongly-rejected compile-blocker.** SPEC's single-`MalformedPayloadPadding`-arm S2 is correct.

## Important
**I1 (recurrence) — the S3.5 audit was itself incomplete: 8 `canonical_origin` consumers, not 6.** Two un-dispositioned: `verify_bundle.rs:391` (verify-path `is_singlesig_template` gate) + `restore.rs:319` (restore-path gate, covered only obliquely). Both flip None→Some for sh(wpkh) but are **provably SAFE** — `is_singlesig_template = ... && cli_template_from_tree(&tree).is_some()`, and `cli_template_from_tree` (`synthesize.rs:359-368`) has no `Sh` arm → FALSE for sh(wpkh) regardless of the flip. But the "EVERY consumer" completeness claim was falsifiable, and the VERIFY side (a verify false-pass = funds risk) deserves a symmetric guard mirroring the restore guard. Everything else in I1 verified sound (5 listed sites correct; `restore.rs:1645` SAFE-because arg sound — `canonical_fallback` is n≥2 multisig-completion, unreachable for n==1 sh(wpkh); guard test specified).

## Minor
- **M-a:** F-A9 (`TooManyErrors` Display message change) undispositioned. Genuine no-op (`friendly_md_codec` re-derives from `chunk_index`/`bound`, `friendly.rs:376-377`; no toolkit test pins md-codec Display). Name it.
- **M-b:** F-A2 rationale undercounts — 3 `decode_md1_string` calls (`prop_repair_never_wrong.rs:483`, `tests/cli_inspect.rs:234`, `tests/cli_repair_md1_non_chunked.rs:126`), all non-chunked; no-op holds (+ single-payload versions all-even ⇒ LSB=0 ⇒ never diverts). Fix count.
- **M-c:** S5 second manual site mischaracterized — only `41-mnemonic.md:418` hard-codes "five … wrapper shapes"; ~2989-3005 is a `--classify-descriptor` flag table + examples, no "five" string. Correct.

## Confirmed correct (no action)
M2 fold (`repair.rs:1710` wildcard absorbs `MalformedPayloadPadding`; direct-decode-cell rescope sound). M3 fold (`verify_bundle.rs:1414` probe; real refusal = md1 byte-mismatch per the code's own comment `1418-1420`). Cross-track (`IMPLEMENTATION_PLAN…P2.1b` carries the `EmptyOriginOverride` 0.42.0 arm, correctly attributing `MalformedPayloadPadding` to Track B). md-codec Error not `#[non_exhaustive]` → S2 genuine compile-blocker; exit-2 matches `mk_codec::Error::MalformedPayloadPadding`→2 sibling precedent (toolkit `error.rs:473`); grouped placement consistent.

## VERDICT: OPEN (0C / 1I + 3M) → fold + quick re-converge to GREEN.

---
**FOLD STATUS (opus, 2026-07-11):** ALL FOLDED. I1: added S3.5 rows for `restore.rs:319` + `verify_bundle.rs:391` (SAFE-via-`cli_template_from_tree` reasoning) + a `verify_bundle.rs:391` VERIFY-side guard assertion in acceptance #2 (8-consumer audit now complete). M-a (F-A9 named no-op), M-b (F-A2 count 1→3), M-c (S5 only `:418` is the five→six edit). Re-dispatch round-3 convergence (opus).
