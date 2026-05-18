# Phase 5 R0 review — wallet-import v0.26.0

**Date:** 2026-05-18
**Reviewer:** opus architect
**Commit under review:** `ff1c85c` (HEAD; parent `b953745` = Phase 4 R0 folds GREEN)
**Worktree:** `.claude/worktrees/wallet-import-export-multiformat-brainstorm`

**Verdict:** YELLOW — 0 Critical, 3 Important, 5 Minor. Phase 5 wires sniff dispatch + seed overlay + `--json` envelope correctly end-to-end. The Phase 4 I2 contract IS partially met (the four `#[allow(dead_code)]` attributes on `canonicalize_bsms` / `canonicalize_bitcoin_core` / `unified_diff` / `recanonicalize_descriptor` in `roundtrip.rs` are REMOVED and the helpers are wired through `cmd::import_wallet::run`). However four `#[allow(dead_code)]` attributes survive in `wallet_import/mod.rs` (lines 57, 77, 96, 178) at items now actually consumed by Phase 5 — they are silent latent-bug class per the Phase 4 I2 lesson and should be removed in the fold pass. Other Importants: (a) `--json` envelope `bundle: {}` shape is a SUMMARY (cosigners/network/threshold), not the toolkit-native `BundleJson` SPEC §2.2 says verify-bundle consumes — divergence either needs SPEC amendment or FOLLOWUP for v0.27+; (b) the `--format` mismatch contract in `cmd/import_wallet.rs:137-179` is LOOSER than SPEC §6.2 ("supplied `<X>` AND `<X>`'s sniff returns false → exit 1 mismatch") — the code only mismatches when the OTHER format's sniff matches. The cell `sniff_explicit_format_honored_when_blob_has_vendor_markers` actually documents the looser behavior as desired (legit Core blob with vendor-marker → user-overrides → parse proceeds). That's a SPEC-vs-implementation drift that needs either SPEC clarification or implementation tightening.

## Critical

None.

## Important

### I1 — Four `#[allow(dead_code)]` attributes survive in `wallet_import/mod.rs` at items now actually consumed by Phase 5

**Sites:**
- `crates/mnemonic-toolkit/src/wallet_import/mod.rs:57` — `#[allow(dead_code)] // Phase 2 constructs; Phase 5 consumes descriptor + bsms_audit.`
- `crates/mnemonic-toolkit/src/wallet_import/mod.rs:77` — `#[allow(dead_code)] // Phase 5 consumes range + wallet_name for round-trip emit.`
- `crates/mnemonic-toolkit/src/wallet_import/mod.rs:96` — `#[allow(dead_code)] // Phase 5 wires the full clap parser.`
- `crates/mnemonic-toolkit/src/wallet_import/mod.rs:178` — `#[allow(dead_code)] // Phase 5 consumes for --json envelope emission.`
- `crates/mnemonic-toolkit/src/wallet_import/bitcoin_core.rs:487` — `#[allow(dead_code)] // consumed by sniff; same Phase 5 wiring rationale as VENDOR_MARKER_KEYS.`

The Phase 4 R0 I2 contract was: "remove `#[allow(dead_code)]` in the same commit that wires the helpers". The implementer correctly removed the four `roundtrip.rs` attributes (verified via `Grep`). But the parallel attributes in `mod.rs` survived — at items now demonstrably consumed by Phase 5:

- `ParsedImport` (line 57) — consumed at `cmd/import_wallet.rs:43`, `:197-205`, `:322-431`. Specifically `p.bsms_audit` at `:406`, `p.source_metadata` at `:418`, `p.network` at `:354`, `p.cosigners` at `:342`. The only field NOT consumed is `descriptor: md_codec::Descriptor` — see Important I2 below.
- `CoreSourceMetadata` (line 77) — consumed at `cmd/import_wallet.rs:418-426`, `:515-518`. `wallet_name` accessed at `:426`.
- `SelectDescriptor` (line 96) — consumed at `cmd/import_wallet.rs:43,236,247,266-271`.
- `BsmsAuditFields` (line 178) — consumed at `cmd/import_wallet.rs:407-415`.
- `trim_leading_ws` (bitcoin_core.rs:487) — called by `BitcoinCoreParser::sniff` at `bitcoin_core.rs:72`.

The `#[allow(dead_code)]` was the compiler-side gate that would catch a Phase 5 forgetting-to-wire bug; now it's just covering nothing. Worse, leaving it on `ParsedImport` (line 57) actively hides that `ParsedImport.descriptor` field is NEVER consumed in cmd/import_wallet.rs — see I2.

**Fix:** Remove all four `#[allow(dead_code)]` attributes in `mod.rs` (and the one in `bitcoin_core.rs:487` since `trim_leading_ws` IS called by `sniff`). Then compile — if anything fires dead-code, that surfaces the real I2-style gap (e.g. `ParsedImport.descriptor`).

**SPEC reference:** Phase 4 R0 review I2 (lines 33-41); `[[feedback-build-rs-stub-fallback-security-audit]]` latent-bug class.

### I2 — `ParsedImport.descriptor` is never consumed by `cmd::import_wallet::run`; `--json` envelope `bundle:` field diverges from SPEC §2.2

**Site:** `crates/mnemonic-toolkit/src/wallet_import/mod.rs:59` (`pub(crate) descriptor: md_codec::Descriptor`); `cmd/import_wallet.rs:336-359` (`emit_json_envelope` builds `bundle_view` summary).

The `ParsedImport.descriptor: md_codec::Descriptor` is constructed by the parsers (`bitcoin_core.rs:242-249`, BSMS path equivalent) but never read by `cmd::import_wallet::run`. The Phase 5 `--json` envelope's `bundle:` field is a hand-built summary at `:355-359`:

```rust
let bundle_view = json!({
    "cosigners": cosigners_json,    // [{fingerprint, path_raw, xpub, has_entropy}]
    "network": network_name,
    "threshold": p.threshold,
});
```

SPEC §2.2 says `bundle: {...}` is "toolkit-native bundle struct (same shape `verify-bundle --bundle-json` consumes)". The implementer comment at `cmd/import_wallet.rs:336-341` acknowledges this divergence: "The full BundleJson shape (with synthesized ms1/mk1/md1 cards) is NOT produced by import-wallet in v0.26.0 — synthesis happens in a separate `bundle` pipeline."

Risk:
- Downstream consumers (mnemonic-gui Phase 6, future automation, manual examples in Phase 6) will encode against the actual shipped shape. If v0.27 changes the envelope to honor SPEC §2.2, it's a wire-format break.
- The `descriptor: md_codec::Descriptor` field is parsed (CPU cost on every parse) but never used. Either it's dead-weight or it's the intended source of the full BundleJson that wasn't wired.
- The user-facing JSON contract is now diverged from SPEC.

**Fix:** EITHER (a) wire `p.descriptor` into the envelope via a `descriptor_md1: <md_codec::encode(p.descriptor)>` field (or full synthesis if accessible), AND/OR (b) amend SPEC §2.2 to reflect the summary shape as final for v0.26.0 + file FOLLOWUP `wallet-import-json-envelope-full-bundle` for v0.27+. Either way, surface the divergence explicitly — not silently as an in-source comment that will be missed at GUI integration time.

**SPEC reference:** SPEC §2.2 ("toolkit-native bundle struct (same shape `verify-bundle --bundle-json` consumes)") vs `cmd/import_wallet.rs:336-359`.

### I3 — `--format <X>` mismatch contract is looser than SPEC §6.2

**Site:** `crates/mnemonic-toolkit/src/cmd/import_wallet.rs:137-179` vs SPEC §6.2 lines 220-225.

SPEC §6.2: "If `--format <X>` is supplied AND `<X>`'s parser's `sniff` returns false: exit 1 `ImportWalletFormatMismatch`".

Implementation: only errors `ImportWalletFormatMismatch` when the OPPOSITE format's sniff matches. When sniff is `NoMatch` (e.g., vendor-marker rejection), `--format bsms` for random text proceeds to parse and surfaces `ImportWalletParse` (exit 2) — not exit 1.

This divergence is actually USER-PREFERRED — the cell `sniff_explicit_format_honored_when_blob_has_vendor_markers` (tests/cli_import_wallet_sniff.rs:142-160) DOCUMENTS the behavior that a Specter-shape blob with `chain` vendor marker + `--format bitcoin-core` parses successfully. Tightening to literal SPEC would break that legitimate override path.

The right resolution is to amend SPEC §6.2 to match the implementation: mismatch fires only when the SUPPLIED format directly contradicts a positive sniff verdict for a DIFFERENT format. NoMatch / Ambiguous + explicit `--format` honors the user's override.

**Fix:** Amend SPEC §6.2 to specify the mismatch fires only on positive-sniff-for-different-format. Add a sentence: "If sniff returns `NoMatch` or `Ambiguous`, the explicit `--format` is honored unconditionally and parse proceeds with the supplied format." Also add a third cell `sniff_format_override_with_no_match_honored_random_text` testing `--format bsms` on a non-BSMS blob → exit 2 `ImportWalletParse` (not exit 1) to lock the contract.

**SPEC reference:** SPEC §6.2 line 224 vs `cmd/import_wallet.rs:137-179`.

## Minor

### M1 — `emit_json_envelope` silently masks canonicalize failures with empty-string default

**Site:** `crates/mnemonic-toolkit/src/cmd/import_wallet.rs:329-333, 363-385`.

```rust
let canon_orig = match format_str {
    "bsms" => canonicalize_bsms(blob).ok(),         // Err → None
    "bitcoin-core" => canonicalize_bitcoin_core(blob).ok(),
    _ => None,
};
...
let canon = canon_orig.clone().unwrap_or_default();   // None → ""
let byte_exact = original_text == canon;
let diff_val = if byte_exact { Null } else { unified_diff(original_text, &canon) };
json!({
    "byte_exact": byte_exact,
    "semantic_match": true,   // <-- hardcoded true even if canonicalize failed
    "diff": diff_val,
    "status": "ok",
})
```

If `canonicalize_bitcoin_core` errors (post-JSON-parse-success — e.g., exotic descriptor that `BitcoinCoreParser::parse` accepts but `MsDescriptor::from_str` rejects in the canonicalize path), the envelope claims `semantic_match: true` + `status: "ok"` while the diff is `unified_diff(blob, "")` (massive). After successful parse this should be unreachable in practice, but the silent unwrap_or_default loses a real signal.

**Fix:** Replace `unwrap_or_default()` with an explicit `match canon_orig { Some(c) => ..., None => emit error_status_envelope }`. Set `status: "canonicalize_failed"` and `semantic_match: false` in the error branch.

### M2 — Doc-comment header for TREZOR_24 lists wrong xpub for m/48'/0'/0'/2'

**Site:** `crates/mnemonic-toolkit/tests/cli_import_wallet_seed_overlay.rs:17-18` vs `:34`.

Lines 17-18 doc-comment: "Known xpubs (verified live): `m/48'/0'/0'/2'` (BIP-48 multisig segwit, account 0): `xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx`"

Line 34: `const TREZOR_24_XPUB_BIP48: &str = "xpub6E79FaRWLSJCAgA2jDHRvyrWKwT6aSmR685zptzyYPvmUd44omcxZ1NAzDtbdFBvEADjcVbV4NzTDwQeU6oiSV9KGiMSWhjANZjbfUHkm3Y";`

Test uses the constant; doc-comment is misleading. The constant carries the line-33 "verified live" comment.

**Fix:** Update doc-comment lines 17-18 to match the actual constant on line 34, or delete the doc-comment xpub claim entirely.

### M3 — Plan §5.10 `seed_overlay_partial_watch_only` not covered (multi-cosigner skip-middle case)

**Site:** `crates/mnemonic-toolkit/tests/cli_import_wallet_seed_overlay.rs` (7 cells); plan §5.10 line 489.

Plan §5.10 calls for "3-cosigner blob; supply `--ms1` for cosigner 0 + 2 only (cosigner 1 skipped). Assert cosigner 1 has `entropy: None` (watch-only); cosigner 0 + 2 have entropy populated."

Closest shipped cell is `seed_overlay_empty_string_sentinel_preserves_watch_only` which uses a 1-of-1 single-cosigner blob — exercises the empty-string sentinel path but not the multi-cosigner-with-middle-skip case. The actual mechanism (`--ms1 X --ms1 "" --ms1 Z` on a 3-cosigner blob) is uncovered.

**Fix:** Add cell `seed_overlay_multi_cosigner_skip_middle` using a 2-of-3 BSMS multisig blob with known seeds for cosigners 0 and 2; assert `has_entropy: [true, false, true]` post-overlay.

### M4 — `--format` and `--select-descriptor` lack clap-side `PossibleValuesParser`

**Site:** `crates/mnemonic-toolkit/src/cmd/import_wallet.rs:59-71`.

`--format` accepts any string; bad values surface at runtime via `cmd/import_wallet.rs:156-159` as `BadInput`. Same for `--select-descriptor`.

Per `[[feedback-clap-derive-help-enumerations]]`: `PossibleValuesParser` adds dropdown validation + cleaner `--help` enumerations. Phase 6 GUI lockstep relies on the `gui-schema` JSON for the dropdown.

**Fix:** Apply `value_parser = clap::builder::PossibleValuesParser::new(["bsms", "bitcoin-core"])` to `--format`.

### M5 — `parse_select` function carries the wrong doc-comment

**Site:** `crates/mnemonic-toolkit/src/cmd/import_wallet.rs:262-264`.

The doc-comment describes `--ms1` positional vector construction, but the function parses `--select-descriptor`. Stale paste-over from an earlier draft.

**Fix:** Rewrite to "Parse the `--select-descriptor` flag value into a SelectDescriptor variant."

## Out of scope / observations

- `Vec<Option<String>>` for `ms1_args` is wrapping that's always `Some` — cosmetic dead-weight; defensive for sparse-array semantics. Leave.
- `let _ = crate::repair::resolve_no_auto_repair(no_auto_repair);` at `:121` is vestigial; not a bug.
- Cell count: 10 unit (sniff.rs) + 11 + 7 integration = 28 new Phase-5 cells.
- `derive_xpub_at_path` reuse: `overlay.rs:166-171` inlines `Xpriv::new_master + derive_priv + Xpub::from_priv` rather than calling `synthesize::derive_xpub_at_path`; comment correctly justifies the typed-DerivationPath route. Same crypto primitives.
- `mnemonic.to_seed("")` empty passphrase intentional (no `--passphrase`).
- Watch-only invariant correctly scoped to parse-time; overlay is explicit downstream step.
- `@env:VAR` resolution: `cmd/import_wallet.rs:282-307` matches `cmd/convert.rs:1571-1608` precedent.
- Stderr silence under `--json` honored.
- BSMS round-trip status `"blocked_no_emitter"` cell pinned; SPEC §7.4 doesn't forbid extensions.
- Cosigner-to-cosigner coin-type heterogeneity gate NOT weakened by overlay.
- Specter/Sparrow vendor markers — `"version"` is fragile; FOLLOWUP `wallet-import-sniff-bitcoin-core-tighten-heuristic` already noted.
- All flags clean for Phase 6 SubcommandSchema mirror.

## Cell-coverage assessment

| Plan §5.X | Coverage in commit `ff1c85c` | Verdict |
|---|---|---|
| §5.1 `sniff.rs::sniff_format` impl | `src/wallet_import/sniff.rs:43-52` + 10 unit cells | OK |
| §5.2 `sniff_bsms_2line_detected` | `tests/cli_import_wallet_sniff.rs:53-61` | OK |
| §5.3 `sniff_core_descriptors_detected` | `tests/cli_import_wallet_sniff.rs:64-72` | OK |
| §5.4 `sniff_ambiguous_with_specter_markers` | `tests/cli_import_wallet_sniff.rs:75-92` | OK |
| §5.5 `sniff_format_mismatch_explicit_override` | `tests/cli_import_wallet_sniff.rs:95-105` | OK |
| §5.6 `sniff_no_match_no_format_exit_1` | `tests/cli_import_wallet_sniff.rs:108-122` | OK |
| §5.7 seed-overlay impl | `src/wallet_import/overlay.rs:57-189` | OK |
| §5.8 `seed_overlay_ms1_match_success` | `tests/cli_import_wallet_seed_overlay.rs:60-92` | OK |
| §5.9 `seed_overlay_ms1_mismatch_exit_4` | `tests/cli_import_wallet_seed_overlay.rs:98-129` | OK |
| §5.10 `seed_overlay_partial_watch_only` (3-cosigner skip-middle) | **NOT covered** | MISSING (M3) |
| §5.11 `seed_overlay_via_slot_subkey_phrase` | `tests/cli_import_wallet_seed_overlay.rs:171-198` | OK |
| §5.12 `sniff_path_roundtrip` | `tests/cli_import_wallet_sniff.rs:125-139` | OK |

**Cell count:** 10 + 11 + 7 = 28 new Phase-5 cells. Plan target 8-10; coverage wider than plan.

## Verdict reasoning

- **I2 contract honored at the `roundtrip.rs` site** — Phase 4 R0's load-bearing fold is met.
- **I1 surfaces a related I2-style oversight** — five `#[allow(dead_code)]` attributes left on items now consumed; trivial fold.
- **I2 (envelope `bundle:` shape divergence)** is the substantive SPEC drift; either SPEC amendment or FOLLOWUP needed.
- **I3 (`--format` mismatch contract)** is a SPEC clarification; implementation is user-preferred.
- **Cell coverage BROAD** — 28 new cells, well over plan target. M3 (multi-cosigner skip-middle) is the only material gap.

Recommendation: fold I1 + I2 + I3 (SPEC amendments or FOLLOWUP filings) + M1/M2/M3/M5 inline before Phase 6 dispatch. M4 (`PossibleValuesParser`) can fold same-cycle. Then GREEN → proceed to Phase 6.
