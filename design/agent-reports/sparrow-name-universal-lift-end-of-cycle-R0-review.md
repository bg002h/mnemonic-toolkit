# End-of-cycle R0 review — v0.37.8 universal source-name lift

Reviewer: feature-dev:code-reviewer (opus). Spec was already R0→R1 GREEN
(`SPEC_sparrow_name_universal_lift.md` + `sparrow-name-universal-lift-R{0,1}-review.md`). This review covers the IMPLEMENTATION + tests + docs + Phase-6 release prep against the working-tree diff before tag/ship. Note: reviewer subagent lacked Write tool; this body returned verbatim and persisted by the parent.

## VERDICT: GREEN — 0 Critical / 0 Important / 3 Minor

The v0.37.8 implementation is **ship-ready**. Every spec invariant in scope passes verification:

- SemVer-PATCH disposition holds (additive Optional wire-shape, no clap surface change).
- Specter `MissingField::WalletName` dissolution verified: `wallet_export/specter.rs:34` reads the renamed `wallet_name_is_non_default`, which flips on either `args.wallet_name.is_some()` OR `lifted_wallet_name.is_some()` at `cmd/export_wallet.rs:741`.
- Coldcard singlesig correctly omitted: `wallet_import/coldcard.rs:111-124` confirms `ColdcardSourceMetadata` has no `name` field; never emitted; impl's iter-over-probes design then defaults to None.
- BSMS correctly left in `SOURCES_LACKING_WALLET_NAME = ["bsms"]`: `wallet_import/bsms.rs` has zero `wallet_name`/`label`/`name` references in its wire shape.
- 6 envelope projections cross-checked against emit-blocks at `cmd/import_wallet.rs:1736-1909` (jade nested-under-`coldcard_compat`, others flat). All probe paths in `resolved_wallet_name` (`json_envelope.rs:108-115`) match the emit-shapes.
- `walk_str` nested-key traversal correctly implemented; jade `["coldcard_compat", "name"]` case covered by unit-cell 4/7 + integration cell 3/6.
- Explicit `--wallet-name` precedence preserved via `or_else` chain at `cmd/export_wallet.rs:707-711`; integration cell `explicit_wallet_name_overrides_envelope_lifted_name` pins it.
- Direct (non-from-import-json) path unchanged: `cmd/export_wallet.rs:482` retains `args.wallet_name.is_some()` only; `lifted_wallet_name` not introduced into that path.

Phase-6 release-prep complete and consistent:
- `crates/mnemonic-toolkit/Cargo.toml:3` = `0.37.8`
- `Cargo.lock:694` = `0.37.8`
- `README.md:13` + `crates/mnemonic-toolkit/README.md:9` = `<!-- toolkit-version: 0.37.8 -->`
- `scripts/install.sh:32` = `mnemonic-toolkit-v0.37.8`
- `CHANGELOG.md:9-16` substantive `[0.37.8]` section
- `design/FOLLOWUPS.md:85-87` `sparrow-from-import-json-wallet-name-preservation` flipped to `resolved (v0.37.8 — 2026-05-28)`

Manual lockstep complete: chapter-45 sparrow note (`docs/manual/src/45-foreign-formats.md:322-332`) + coldcard-multisig note (`:586-593`); sparrow transcript empty; coldcard-multisig transcript no longer carries `Name:` line.

## Per-section findings

### Code changes (items 1-6) — verified
- `wallet_import/json_envelope.rs:62-127` — 6 Optional fields + `resolved_wallet_name()` + `walk_str` walker correctly wired. Impl diverges from spec design by iterating-all-probes instead of `match self.source_format.as_str()`; doc-comment at lines 96-101 acknowledges the design choice and tie-break semantics. Functionally equivalent in canonical case. Below threshold (see M2).
- `wallet_import/mod.rs:198-212` — `coldcard_multisig_source_metadata()` accessor alphabetical-by-variant match. Compliant.
- `cmd/import_wallet.rs:1779-1807` — coldcard-multisig emit-block. Projection shape matches the unit-cell expectation exactly.
- `wallet_export/mod.rs:507` + `wallet_export/specter.rs:34` — field rename complete; doc-comment at `mod.rs:501-506` captures new semantics.
- `cmd/export_wallet.rs:706-741` — lift + `or_else` precedence + flag flip. Correct.

### Tests (items 7-10) — verified with one minor gap (M1)
- `wallet_import/json_envelope.rs` mod tests — 7 cells + 1 walker sub-cell. Covers the 6 per-format positive cases + missing-metadata negative + walker negative branches.
- `tests/cli_export_wallet_universal_name_lift.rs` — 8 cells, all fixtures present.
- `tests/cli_export_wallet_from_import_json.rs:909` — `SOURCES_LACKING_WALLET_NAME = ["bsms"]` narrowing correct + commented.
- `tests/cli_export_wallet_specter.rs:114-116` — doc comment updated.

### Docs (items 11-13) — verified
### Release prep (items 14-20) — verified

## Minor findings (below threshold)

**M1 — Missing dedicated empty-string regression cell.** The `if !name.is_empty()` filter at `json_envelope.rs:119` has no unit test. SPEC §6 listed `json_envelope_empty_string_returns_none` as cell #2; impl omitted it. Source-parsers already filter empty strings (e.g. `electrum.rs:759-763`), so this gap protects a defense-in-depth path. Confidence ~45. Suggested fold: 3-line cell with `sparrow_source_metadata: {"label": ""}` asserting `None`. **Folded inline as `resolved_wallet_name_returns_none_on_empty_string_leaf`.**

**M2 — Impl diverges from spec's `match on source_format` design.** Spec proposed dispatch-on-source-format; impl iterates all 6 probes. Equivalent on canonical envelopes; spec's `json_envelope_unknown_format_returns_none` cell #7 is moot under iter-all. Doc-comment at lines 96-101 captures the rationale. No correctness impact. Confidence ~30. Not folded.

**M3 — `SOURCES_LACKING_WALLET_NAME` narrowing prose is technically correct but understated.** Coldcard singlesig remains untested for specter target (same as pre-v0.37.8 — pre-existing scope, not a v0.37.8 regression). Confidence ~25. Not folded.

## Ship recommendation

Proceed to tag `mnemonic-toolkit-v0.37.8`. Cycle meets the CLAUDE.md GREEN gate (0C/0I).
