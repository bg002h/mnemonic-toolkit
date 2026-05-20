# v0.28.0 Phase P4C — architect R0 self-review (GREEN)

**Sub-phase:** P4C — Flip 8 dispatch sites + integration tests + xfp-divergence
WARNING cell.

**Plan-doc anchors:** §S.4 + table-row P4C (plan-doc line 517).

**SPEC anchors:** SPEC_wallet_import_v0_28_0.md §11.4 + §6.1 + §6.2 + §12.

**Branch:** `v0.28.0/p4-coldcard-multisig` (continued from P4A + P4B).

**Predecessor commit:** P4B @ `508afa3`.

---

## Scope verification (P4C boundary)

Per plan-doc §B.2 #6 (the 8-site dispatch surface) + plan-doc P4C row:

| Site | File / line | Pre-P4C | Post-P4C |
|---|---|---|---|
| 1 (clap PossibleValues) | `cmd/import_wallet.rs:108` | `"coldcard-multisig"` listed | UNCHANGED (already wired at P0C) |
| 2 (explicit `--format` arm) | `cmd/import_wallet.rs:274-276` | `unimplemented!("P4C")` | Mismatch check against `SniffOutcome::Bsms` / `BitcoinCore` → return `"coldcard-multisig"` |
| 3 (auto-sniff `None =>` arm) | `cmd/import_wallet.rs:288` | `other => unreachable!()` catch-all | NEW `SniffOutcome::ColdcardMultisig => "coldcard-multisig"` arm |
| 4 (parse dispatch) | `cmd/import_wallet.rs:362` | `unimplemented!("P4C")` | `ColdcardMultisigParser::parse(&blob, stderr)?` |
| 5 (select-descriptor coerce) | `cmd/import_wallet.rs:412-425` | `_ => apply_select_descriptor(...)` default | UNCHANGED — coldcard-multisig falls through to default per plan-doc §B.2 #6 ("none identified at plan-time") |
| 6 (canonicalize dispatch) | `cmd/import_wallet.rs:540-542` | Already wired at P0C (calls `canonicalize_coldcard_multisig`) | UNCHANGED at P4C (the body became real at P4B; this site is body-swap-transparent) |
| 7 (JSON envelope round-trip) | `cmd/import_wallet.rs:696` | `"coldcard-multisig" => json!({})` (placeholder) | Real shape mirroring bitcoin-core: `{byte_exact, semantic_match, diff, status}` per canon_orig branch |
| 8 (provenance accessor) | `wallet_import/mod.rs` `bsms_audit()` + `source_metadata()` | Exhaustive arm `ColdcardMultisig => None` added at P4A | UNCHANGED at P4C |

All 8 dispatch sites are now coherent. Sites 5 + 6 + 8 were either already
wired at P0C/P4A or fall through to defaults that already handle the
new variant correctly.

NEW imports list update: `crates/mnemonic-toolkit/src/cmd/import_wallet.rs`
gains `coldcard_multisig::ColdcardMultisigParser` import line (alphabetical
position between `bsms::BsmsParser` and `overlay::apply_seed_overlay`).

## Integration tests

NEW file: `crates/mnemonic-toolkit/tests/cli_import_wallet_coldcard_multisig.rs`
with 12 cells covering:
- Happy-path explicit-`--format` parse on all 3 happy-path fixtures.
- Auto-sniff dispatch on fixtures (Site 3 wiring exercise).
- Refusal on malformed-missing-Format fixture (`ImportWalletParse`, exit 2).
- SPEC §11.4.1 row-2 WARNING byte-exact template check (xfp-header
  divergence + per-cosigner `<XFP>:` divergence).
- SPEC §6.1 format-mismatch dispatch (`--format coldcard-multisig` vs
  BSMS blob / Bitcoin Core blob).
- `--json` envelope shape verification (schema_version, source_format,
  roundtrip.status=="ok").
- Auto-sniff doesn't co-fire with BSMS / Bitcoin Core on the 3 happy-path
  fixtures (smoke for SPEC §6.2 consult-all-then-count discipline).

## P0C dispatch-test update (in-place fold)

`tests/cli_import_wallet_p0c_dispatch.rs::p0c_format_coldcard_multisig_panics_unimplemented`
RENAMED to `p0c_format_coldcard_multisig_dispatches_format_mismatch_post_p4c`
and updated to assert the post-P4C semantic (BSMS blob with
`--format coldcard-multisig` surfaces `ImportWalletFormatMismatch`).
This is the in-place fold per "skeleton-or-real" matrix-discipline: when
a sub-phase wires a previously-stubbed dispatch arm, the corresponding
P0C-stub regression cell must update in lockstep.

## SPEC fidelity

- **§6.1 format-mismatch**: Site 2 explicit arm rejects when sniff
  identifies a different format. Mirrors the existing BSMS / Bitcoin Core
  arms (lines 246-263). Verified by 2 integration cells.
- **§6.2 sniff dispatch**: Site 3 adds the `SniffOutcome::ColdcardMultisig`
  arm. Auto-sniff via `sniff_format` now returns this variant for
  text-shape blobs (the bool wiring was P4A). Verified by 3 integration cells.
- **§11.4 parse dispatch**: Site 4 invokes `ColdcardMultisigParser::parse`.
  The parser internally wraps `parse_text` (P4B) and returns
  `Vec<ParsedImport>` of length 1. Verified by 3 happy-path integration cells.
- **§11.4.1 xfp WARNING byte-exact template**: verified character-for-character
  by `coldcard_ms_xfp_header_divergence_warns_byte_exact_template` against
  the SPEC §11.4.1 template (template includes the literal substrings
  `warning: import-wallet: coldcard-multisig: xfp header`, the
  blob-supplied XFP value, the computed value, and the
  `using blob-supplied header value as authoritative` clause).
- **§7.4 round-trip envelope**: Site 7 emits `byte_exact` /
  `semantic_match` / `diff` / `status` matching the bitcoin-core
  shape. Verified by `coldcard_ms_json_envelope_emits_canonical_shape`.

## Cross-instance handoff (final state)

With P4C merged to `release/v0.28.0`, Phase E (Jade) is FULLY unblocked:
- E's P5A may have run parallel-to-P4A; can now finalize.
- E's P5B may delegate to `coldcard_multisig::parse_text` (P4B published `pub(super)`).
- E's P5C will follow the same 8-site dispatch pattern this P4C established.

The plan-doc Wave-1 cross-instance dependency D → E is satisfied.

## Verification

- `cargo clippy --all-targets -- -D warnings` — clean. ✓
- `cargo test --test cli_import_wallet_coldcard_multisig` — 12/12 pass. ✓
- `cargo test --test cli_import_wallet_p0c_dispatch` — 10/10 pass (the
  in-place renamed cell asserts the new post-P4C semantic). ✓
- Full `cargo test` workspace run — all suites pass; no regressions. ✓

## Findings

**Critical:** NONE.
**Important:** NONE.
**Minor:** NONE worth blocking on.

### Minor inline observations (not blocking; pinned for future cycles)

- (M1, no fold) The `p0c_format_coldcard_multisig_dispatches_format_mismatch_post_p4c`
  cell renaming + assertion update created a small file-level naming
  inconsistency: 4 sibling cells still carry the old `p0c_format_X_panics_unimplemented`
  shape (for the 4 not-yet-wired formats: coldcard / electrum / jade /
  sparrow / specter). Future P{N}C sub-phases (P1C/P2C/P3C/P5C/P6C)
  will each perform the same rename-and-flip on their respective cell,
  reaching uniform post-flip naming. The current half-state is
  expected during the Wave-1 fan-out.

- (M2, no fold) `coldcard-multisig` has NO `--select-descriptor` coerce
  override at Site 5 (it falls through to the default `apply_select_descriptor`
  arm). This is correct per plan-doc §B.2 #6 ("none identified at plan-time"):
  a Coldcard multisig text file describes a single multisig wallet
  (one descriptor); `apply_select_descriptor` on a single-entry parse
  trivially returns `SelectDescriptor::All` semantics. If a future user
  passes `--select-descriptor active-receive` against a coldcard-multisig
  blob, they'll get a "no active-receive descriptor found" error since
  ColdcardMultisig provenance lacks `source_metadata` (returns `None`).
  This is acceptable UX; future cycles MAY add a NOTICE-coerce arm
  mirroring BSMS at Site 5 if user reports surface.

## Overall R0 verdict

**GREEN.** P4C scope satisfied: all 8 dispatch sites coherent; SPEC §11.4
+ §11.4.1 + §6.1 + §6.2 fully exercised by 12 new integration cells;
P0C-stub regression cell updated in lockstep; cross-instance handoff to
Phase E complete; no regressions.

Recommendation: commit P4C, open PR to `release/v0.28.0` (do NOT self-merge
per task brief). Phase 4 (Instance D) work complete.
