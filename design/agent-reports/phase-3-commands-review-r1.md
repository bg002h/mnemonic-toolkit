# Phase 3 Commands Review — r1

**Date:** 2026-05-04
**Commit under review:** `e92b3a9` (parent: `f3dc44a`)
**Reviewer:** opus phase-review

## Verdict

1 critical / 0 important / 3 low / 0 nits

NOT cleared to proceed to Phase 4 until C-1 is resolved.

## Critical

### C-1: `verify_bundle::run_watch_only` emits 5 checks in wrong order — SPEC §5.4 requires fixed 9-element `checks` array

**File:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:run_watch_only`

SPEC §5.4 lines 538-548 specify a fixed 9-element `checks` array with byte-exact field order:

```
ms1_entropy_match, mk1_decode, mk1_xpub_match, mk1_fingerprint_match,
mk1_path_match, md1_decode, md1_wallet_policy, md1_xpub_match, stub_linkage
```

Line 552: "`skipped` covers checks not applicable in watch-only mode (entropy/path-rederivation)."

`run_watch_only` currently emits 5 checks in order: `mk1_decode`, `md1_decode`, `stub_linkage`, `mk1_xpub_match`, `mk1_fingerprint_match`. Two substantive watch-only checks (`md1_wallet_policy`, `md1_xpub_match` against `xpub_to_65(&supplied_xpub)`) are entirely missing.

Impact: (a) JSON schema mismatch — consumers iterating by index get wrong data; (b) correctness gap — `md1_wallet_policy` and `md1_xpub_match` ARE substantive in watch-only.

**Fix:** restructure to push all 9 checks in SPEC order; use `skipped` for `ms1_entropy_match` and `mk1_path_match`; compute `md1_wallet_policy` and `md1_xpub_match` from decoded md1 + supplied --xpub via `xpub_to_65`.

## Important

(none)

## Low / Nit (defer to design/FOLLOWUPS.md)

- **L-1:** `friendly_mk_codec` `MixedCase` message is `"mk1 mixed case in input string"` — SPEC §6.4.4 has `"mixed case in mk1 input string"` (different word order). Phase 5 byte-exact fixtures will pin; minor textual diff.
- **L-2:** `bundle.rs::emit()` calls `chunk_5char` directly for mk1 instead of the `chunk_mk1` named alias. Functionally identical (chunk_mk1 delegates). Nit.
- **L-3:** `§5.2` stderr ordering — depth advisory emitted before watch-only account-index hazard in `bundle_watch_only`. Sub-order unspecified by SPEC; defer until Phase 5 fixtures pin.

## Verified

- **§6.6 byte-exact `mode_text::*` constants:** all 5 match SPEC verbatim. `XPUB_STDIN` correctly routes via `BadInput` (exit 1), not `ModeViolation`.
- **§5.3 `BundleJson` field order:** matches SPEC §5.3 exactly. `account: 0` hardcoded.
- **§5.4 `VerifyBundleJson` envelope shape:** `{schema_version, result, checks}` with `VerifyCheck {name, result, detail}`. Correct.
- **§5.5 routing rule:** `verify_bundle::run` returns `Result<u8, ToolkitError>`; mismatch is `Ok(4)`; pre-decode failures (`ModeViolation`, `NetworkMismatch`, `BadInput`) leave via `Err`. Correct.
- **§4.3 network/xpub cross-check:** present in both `bundle_watch_only` and `run_watch_only`. Message format matches SPEC §4.3.
- **§4.8 depth advisory + account-index hazard:** conditional + unconditional respectively in `bundle_watch_only`. No double-printing.
- **§6.4.1 `friendly_bip39`:** all 5 variants match SPEC.
- **§6.4.4 `friendly_mk_codec`:** 22 concrete arms + wildcard. Routing matches SPEC §6.4.4.
- **§6.4.5 `friendly_md_codec`:** exhaustive 41-variant match (correct since md_codec::Error is NOT non_exhaustive).
- **§2.2.1 `run_full` check ordering:** correctly emits 9 checks per SPEC §5.4 order across the success and mk1-decode-fail branches.
- **47 unit tests passing, clippy + fmt clean.**
