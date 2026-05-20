# v0.28.0 P2A-v2 — R0 architect review

**Branch:** `v0.28.0/p2-specter-v2-bg` (suffixed because `v2` claimed by sibling worktree scaffold)
**Base:** `release/v0.28.0` @ `b20a357`
**Scope:** Specter-DIY skeleton + sniff + `SpecterSourceMetadata` + `SpecterDeviceMarker` + `ImportProvenance::Specter` variant + sniff wiring in `wallet_import/sniff.rs::sniff_format` + auto-sniff arm `SniffOutcome::Specter => "specter"` in `cmd/import_wallet.rs` (BEFORE the unreachable catch-all, per learned-best-practice).

## Verdict

GREEN — 0 Critical / 0 Important / 0 Minor.

## Critical

(none)

## Important

(none)

## Minor

(none)

## Verification

- `cargo build -p mnemonic-toolkit`: clean
- `cargo clippy -p mnemonic-toolkit --all-targets`: clean
- 24 new specter unit tests (sniff positive/negative/cross-format + skeleton parse) pass
- Existing tests (Sparrow + BSMS + Bitcoin Core + ColdcardMultisig) all green
- ImportProvenance accessor matrix tests pass (Specter variant added to all 3 exhaustive matches + new `specter_source_metadata()` accessor)

## Notes

- `ImportProvenance::Specter` carries `#[allow(dead_code)]` because P2A scaffolds the variant but P2C wires the only constructor at the envelope-emit dispatch. Pattern mirrors P1A → P1C lift.
- `SpecterSourceMetadata` + `SpecterDeviceMarker` fields carry `#[allow(dead_code)]` for the same P2A → P2C interim. Lift comes at P2C when the envelope-emit reads the fields.
- Auto-sniff arm `SniffOutcome::Specter => "specter"` added BEFORE the `other => unreachable!()` catch-all (per orchestrator's "CRITICAL — do NOT defer to P2C" directive; matches Sparrow P1A precedent in current code at `cmd/import_wallet.rs`).
- Variant-ordering across enum + 3 accessors stays alphabetical (BitcoinCore, Bsms, ColdcardMultisig, Sparrow, Specter) per CLAUDE.md drift-avoidance discipline.
- Sniff is positive-marker on `{label (string), blockheight (integer), descriptor (string), devices (array)}`; integer-shape check on `blockheight` is the load-bearing disambiguator (rejects string-blockheight blobs that might collide with future formats).
- `bitcoin_core.rs::VENDOR_MARKER_KEYS` already lists `blockheight` + `devices` at lines 94-95 → Bitcoin Core sniff rejects Specter blobs (consult-all-then-count produces unambiguous `Specter` verdict).
- New `specter_source_metadata()` accessor on `ImportProvenance` mirrors `sparrow_source_metadata` (consumed by `--json` envelope at P2C).
- Inlined `trim_leading_ws` helper in specter.rs (sibling to bitcoin_core's private one); the codebase already has multiple such inline duplicates and keeping `bitcoin_core.rs::trim_leading_ws` `pub(super)`-free was preferred.
