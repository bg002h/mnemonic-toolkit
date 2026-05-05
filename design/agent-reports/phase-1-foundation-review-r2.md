# Phase 1 Foundation Review — r2

**Date:** 2026-05-04
**Commits under review:** `6a64331` (r1 fixup), parent `a654a18` (Phase 1 feature)
**Reviewer:** opus phase-review

## Verdict

0 critical / 0 important / 3 low / 2 nits

✅ **Phase 1 r2 terminator reached — cleared to proceed to Phase 2.**

## Critical

(none)

## Important

(none)

## Low / Nit (unchanged from r1; deferred to design/FOLLOWUPS.md)

- **L-1:** SPEC §5.5 `kind` enum list omits `NetworkMismatch` and `FutureFormat`. Code is correct; SPEC prose needs updating.
- **L-2:** `format::chunk_mk1` falls back to `chunk_5char` (space-separated). FOLLOWUPS entry: `mk-codec-chunked-visual-grouping-helper`.
- **L-3:** Phase 5 fixture-author note: md1 hyphens vs mk1 spaces.
- **Nit-1:** Implementation Plan's `spike_md_codec.rs` snippet still uses `[0x42; 65]` filler (panics). FOLLOWUPS: `toolkit-plan-spike-filler-bug`.
- **Nit-2:** Spike memo reviewer byline self-referential. No action.

## Spot-checks performed

- `ms_codec::Error::WrongHrp` → exit 2 ✓
- `ms_codec::Error::UnexpectedStringLength` → exit 1 ✓
- `ms_codec::Error::ReservedPrefixViolation` → exit 2 ✓
- `mk_codec::Error::InvalidHrp` → exit 2 ✓
- `mk_codec::Error::BchUncorrectable` → exit 1 ✓
- `mk_codec::Error::MixedHeaderTypes` → exit 2 ✓
- `mk_codec::Error::CardPayloadTooLarge` → exit 2 ✓
- `md_codec::Error::Codex32DecodeError` → exit 1 ✓
- `md_codec::Error::HardenedPublicDerivation` → exit 2 ✓
- `md_codec::Error::UnsupportedVersion` → exit 3 (explicit arm; required for exhaustiveness since md_codec is NOT non_exhaustive) ✓
- md_codec exhaustive match: 2 exit-1 + 38 exit-2 + 1 exit-3 = 41 total; zero `_ =>` wildcard ✓
- `ModeViolation.message: &'static str` (not `String`) ✓; `message()` arm `(*message).to_owned()` ✓; test fixtures pass `"x"` literal ✓

## Smoke checks

- `cargo test -p mnemonic-toolkit`: 32 passed (29 prior + 3 new routing tests)
- `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings`: clean
- `cargo fmt --check -p mnemonic-toolkit`: clean
