# Phase 1 Foundation Review — r1

**Date:** 2026-05-04
**Commit under review:** `a654a18` (parent: `fda8bae`)
**Reviewer:** opus phase-review

## Verdict

0 critical / 2 important / 3 low / 2 nits

NOT cleared to proceed to Phase 2 until I-1 and I-2 are resolved.

## Critical

(none)

## Important

### I-1: `exit_code()` returns 1 for all `MsCodec` / `MkCodec` / `MdCodec` wrappers — SPEC §6.3 format-violation sub-variants require exit 2

**File:** `crates/mnemonic-toolkit/src/error.rs:46–56`

SPEC §6.3 line 611: "`ModeViolation`, `NetworkMismatch`, `MsCodec::WrongHrp`/etc. format-violation variants → 2". The current `exit_code()` returns 1 unconditionally for `MsCodec(_)`, `MkCodec(_)`, `MdCodec(_)`. The `From` impls intercept only the three exit-3 variants (`ms_codec::Error::ReservedTagNotEmittedInV01`, `mk_codec::Error::UnsupportedVersion`, `md_codec::Error::UnsupportedVersion`); all format-violation inner variants — `mk_codec::Error::InvalidHrp`, `MixedCase`, `UnsupportedCardType`, `MalformedPayloadPadding`, etc. and the 38 Exit-2 md_codec variants enumerated in §6.4.5 — land in `MkCodec(_)` / `MsCodec(_)` / `MdCodec(_)` and currently produce exit 1.

SPEC §6.2 does not add a separate `FormatViolation` variant; SPEC §6.4.4 and §6.4.5 specify per-inner-variant routing tables. The intended architecture is that `exit_code()` inspects the inner sibling error to distinguish exit 1 from exit 2.

**Fix:** Rewrite `exit_code()` arms for `MsCodec(_)`, `MkCodec(_)`, `MdCodec(_)` to match on the inner variant per the SPEC §6.4.4 / §6.4.5 routing tables. ms_codec routing comes from ms-cli's existing dispatch table (§6.4.3 delegates to it).

### I-2: `ModeViolation.message` field is `String` in code vs. `&'static str` in SPEC §6.2

**File:** `crates/mnemonic-toolkit/src/error.rs:17–21`

SPEC §6.2 line 599: `ModeViolation { mode: &'static str, flag: &'static str, message: &'static str }`. Code has `message: String`. SPEC §6.6 pins mode-violation messages as byte-exact `pub const` strings; using `String` weakens the pinning guarantee — Phase 3 `cmd/bundle.rs` call sites could inadvertently format a dynamic message.

**Fix:** `message: &'static str`. Update `message()` arm `ToolkitError::ModeViolation { message, .. } => (*message).to_owned()`. Update test fixtures at `error.rs:24-29` to pass `"x"` (a `&'static str` literal works directly without `.into()`).

## Low / Nit (defer to design/FOLLOWUPS.md)

- **L-1 (SPEC gap, no code fix):** SPEC §5.5 `kind` enum list omits `NetworkMismatch` and `FutureFormat`. Code correctly returns those. Add FOLLOWUPS entry to update SPEC §5.5.
- **L-2:** `format::chunk_mk1` falls back to `chunk_5char` (space-separated 5-char groups) because mk-codec exposes no per-string visual grouping helper. Acceptable for v0.1; Phase 5 fixtures must pin against this space-separated behavior. FOLLOWUPS entry: `mk-codec-chunked-visual-grouping-helper`.
- **L-3:** Phase 5 fixture-author note: md1 uses hyphens (via `render_codex32_grouped`); mk1 uses spaces (via `chunk_5char` fallback).
- **Nit-1:** Implementation Plan's `spike_md_codec.rs` snippet still uses `[0x42; 65]` filler (panics; fix documented in spike memo). Add FOLLOWUPS entry `toolkit-plan-spike-filler-bug`.
- **Nit-2:** Spike memo reviewer byline self-referential ("the spike runner (and Phase 1 reviewer at task 1.10)"). No action.

## Verified

- **SPEC §4.2 origin paths** — `origin_path_str()` produces all 8 (template × network) cells correctly: 4 templates × mainnet(coin=0)/non-mainnet(coin=1); account hardcoded 0. Tests cover 4 representative cells.
- **SPEC §4.6.3 wrapper nodes** — bip44=`Tag::Pkh/Body::KeyArg{0}`, bip49=`Tag::Sh/Body::Children([Tag::Wpkh/Body::KeyArg{0}])`, bip84=`Tag::Wpkh/Body::KeyArg{0}`, bip86=`Tag::Tr/Body::Tr{key_index:0, tree:None}`. All four match SPEC and confirmed against `md_codec::tree::Body` actual variants.
- **SPEC §2.1.5 fingerprint parse** — byte-exact rejection message verified; case-insensitive accept confirmed; `0x`-prefix reject confirmed.
- **SPEC §3.2 stdin uniform** — `normalize_phrase` uses `split_whitespace().join(" ")`. Test exercises tabs, newlines, multiple spaces.
- **SPEC §5.2 engraving card** — byte-exact 8-line test fixture matches SPEC template.
- **SPEC §6.4.0 FutureFormat routing** — three `From` impls intercept the three reserved-not-emitted variants and route to `FutureFormat` (exit 3). Correct.
- **md_codec::Error closure status** — confirmed NOT `#[non_exhaustive]`; 41 variants matching spike memo and §6.4.5 (2 Exit-1 + 38 Exit-2 + 1 Exit-3).
- **Sibling APIs** — `render_codex32_grouped`, `Body::{KeyArg, Tr, Children}`, `Tag::{Sh, Wpkh, Pkh, Tr}` all confirmed.

## Notes carried from Task 1.1

- Plan spike-snippet pubkey filler bug — FOLLOWUPS candidate.
- Spike memo reviewer byline self-referential — no action.
- mk1 chunked output (now tracked as L-2) — FOLLOWUPS candidate.
