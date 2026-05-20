# v0.28.0 — Phase P1C (Sparrow CLI dispatch flip + envelope wiring + integration cells) — R0 architect review

**Branch:** `v0.28.0/p1-sparrow-v2`
**Base:** `release/v0.28.0` @ `d7a2859` (post-Wave-1-cascade)
**Scope:** Flip the remaining 7 cmd/import_wallet.rs dispatch sites for Sparrow + add `sparrow_source_metadata` JSON envelope field + integration cells in `tests/cli_import_wallet_sparrow.rs` + update the P0C regression cell.
**Date:** 2026-05-19
**Reviewer:** self-architect-review (R0)

## Status: GREEN

## Methodology

Plan-doc anchor: `/home/bcg/.claude/plans/unified-meandering-sundae.md:493` —
> **P1C** | Flip the 8 `cmd/import_wallet.rs` dispatch sites from `unimplemented!()` → real `SparrowParser::parse(...)` / `canonicalize_sparrow(...)`. Integration cells in `tests/cli_import_wallet_sparrow.rs` (parse + sniff + roundtrip + envelope-shape cells). | `cmd/import_wallet.rs`, `tests/cli_import_wallet_sparrow.rs` (new) | ~50 src + ~250 tests

§B.2 #6 (orchestrator dispatch-shape lock) — Site 1 (PossibleValuesParser), Site 2 (explicit-format mismatch arm), Site 3 (auto-sniff arm — already wired at P1A), Site 4 (parse dispatch), Site 5 (select-descriptor coerce — default), Site 6 (canonicalize dispatch — already wired at P0C via skeleton-then-real-at-P1B), Site 7 (roundtrip envelope), Site 8 (envelope source-metadata).

## Source-grep verification (current branch HEAD)

### Site 1 — PossibleValuesParser

`cmd/import_wallet.rs:112` includes `"sparrow"` since P0C; unchanged at P1C.

### Site 2 — explicit `--format sparrow` arm (mismatch-check)

`cmd/import_wallet.rs:301-335` (post-P1C) — flipped from `unimplemented!("P1C: format sparrow not yet wired")` to mismatch-check + `"sparrow"` format-str selection. Mismatch matrix at P1C covers BSMS + BitcoinCore + ColdcardMultisig (the 3 wired parsers as of d7a2859). Cycle-followup `wallet-import-format-mismatch-matrix-completion` documents the N×N completion deferral.

### Site 3 — auto-sniff arm

`cmd/import_wallet.rs:336` (post-P1A): `SniffOutcome::Sparrow => "sparrow"`. Unchanged at P1C.

### Site 4 — parse-dispatch arm

`cmd/import_wallet.rs:432`: flipped from `unimplemented!("P1C: parse not yet wired")` to `SparrowParser::parse(&blob, stderr)?`.

### Site 5 — select-descriptor coerce

`cmd/import_wallet.rs:478-481`: Sparrow falls through to the default `_ => apply_select_descriptor(...)` arm. No format-specific coerce needed (Sparrow has no equivalent of BSMS's "single-descriptor" coerce).

### Site 6 — canonicalize dispatch

`cmd/import_wallet.rs:611`: `"sparrow" => Some(canonicalize_sparrow(blob).map_err(|e| e.to_string()))` — already calls the real `canonicalize_sparrow` body since P1B (the import + arm shape was pre-wired at P0C via the skeleton; P1B replaced the body).

### Site 7 — roundtrip envelope

`cmd/import_wallet.rs:801-823` (post-P1C): flipped from `"sparrow" => json!({})` to the full byte_exact / semantic_match / diff / status envelope mirroring the BitcoinCore + ColdcardMultisig shape.

### Site 8 — envelope source-metadata

`cmd/import_wallet.rs:869-887` (post-P1C): NEW `sparrow_source_metadata` field wire-up. Mirrors `source_metadata` (Core) + `bsms_audit` (BSMS) — surfaces ONLY when the parse was Sparrow-shaped. Field name `sparrow_source_metadata` is per-format-distinct to avoid wire-shape conflict with `source_metadata` (Core's).

### `ImportProvenance::sparrow_source_metadata` accessor

`wallet_import/mod.rs:128-137`: P1B added the accessor with `#[allow(dead_code)]` (P1B → P1C bridging); P1C removes the `#[allow]` since the envelope-emit site now calls it. Likewise the `#[allow(dead_code)]` on `SparrowSourceMetadata` struct (sparrow.rs) is removed.

### P0C regression cell update

`tests/cli_import_wallet_p0c_dispatch.rs:56-72`: renamed `p0c_format_sparrow_panics_unimplemented` → `p0c_format_sparrow_dispatches_format_mismatch_post_p1c`. New assertion: stderr cites `sparrow` + `bsms` (format-mismatch shape). Mirrors the P4C precedent for `p0c_format_coldcard_multisig_dispatches_format_mismatch_post_p4c`.

## Test surface (P1C cells)

`crates/mnemonic-toolkit/tests/cli_import_wallet_sparrow.rs` (NEW; 14 cells):

- 4 parse happy-path (singlesig p2wpkh / multi 2-of-3 sortedmulti / multi 2-of-3 multi-ordered / singlesig p2sh-p2wpkh).
- 3 sniff cells (singlesig auto-route / multisig auto-route / sparrow-with-`--format bsms` refusal).
- 4 `--json` envelope cells:
  - `includes_source_metadata_and_roundtrip` — asserts all 4 metadata fields + roundtrip.status=ok.
  - `no_sparrow_source_metadata_on_bsms` — cross-format negative check.
  - `roundtrip_status_ok_on_well_formed_fixture` — pins byte_exact=false + semantic_match=true + status=ok contract.
  - `dropped_fields_surface_in_metadata` — stdin path; asserts birthDate surfaces in dropped_fields array.
- 2 refusal cells (malformed-missing-script exit 2; taproot exit 2).
- 1 canonicalize semantic-drop cell (`sparrow_canonicalize_drops_extra_top_level_fields` — round-trip on non-canonical input).

P0C regression cell `p0c_format_sparrow_dispatches_format_mismatch_post_p1c` updated in place.

Bin target: 570 cells (unchanged from P1B; P1C does not add bin-target unit cells). Integration target adds 14 new sparrow cells in a new file.

`cargo test -p mnemonic-toolkit` full suite: 0 FAILED across all suites.
`cargo clippy --all-targets -- -D warnings` clean.

## Findings

### Critical

**None.**

### Important

**None.**

### Minor

#### M1 — `sparrow_with_bsms_format_refused` cell tolerates either refusal exit code

The cell asserts non-zero exit + stderr contains `bsms`/`format` — does NOT pin a specific `ImportWalletFormatMismatch` vs `ImportWalletParse` exit code. Rationale: the BSMS arm's Site-2 mismatch-check at `cmd/import_wallet.rs:248-262` only catches `SniffOutcome::BitcoinCore` as a mismatch (the matrix isn't yet symmetric per cycle-followup `wallet-import-format-mismatch-matrix-completion`). When sniff says Sparrow, the BSMS arm proceeds and `BsmsParser::parse` fails on the JSON shape → `ImportWalletParse` exit 2. Both code paths surface "bsms didn't work" stderr, which is sufficient for the regression-guard intent. Adding a precise exit-code assertion would couple P1C to the BSMS-side mismatch matrix completion which is deferred.

#### M2 — `sparrow_source_metadata` field name is per-format-distinct

The envelope's `sparrow_source_metadata` field could in principle have been collapsed into the existing `source_metadata` field by relaxing the latter's `Option<&CoreSourceMetadata>` accessor to a wider enum. The chosen approach (per-format-distinct field) is the lowest-risk wire-shape extension; consumers that don't expect `sparrow_source_metadata` simply ignore it. Per-format-distinct is also the convention `bsms_audit` already uses. Not a defect — a deliberate API choice.

### Folds applied this round

None (R0 GREEN).

## Sign-off

P1C scope per plan-doc row 493 is fully implemented:
- ✓ Site 2 + Site 4 + Site 7 dispatch arms flipped from `unimplemented!()` to real dispatch.
- ✓ Site 8 NEW: `sparrow_source_metadata` JSON envelope field via the `sparrow_source_metadata()` accessor.
- ✓ Site 3 auto-sniff arm pre-wired at P1A (`SniffOutcome::Sparrow => "sparrow"`).
- ✓ P0C regression cell updated in place (P4C precedent).
- ✓ Integration cells in new `tests/cli_import_wallet_sparrow.rs` (14 cells: parse + sniff + roundtrip + envelope shape + refusals).

After this commit Sparrow joins ColdcardMultisig as "fully wired" in the v0.28.0 cycle's 8-format dispatch matrix; the remaining 4 (Coldcard, Electrum, Jade, Specter) stay in their per-parser P{N}C placeholders.

Per CLAUDE.md "Per-phase reviewer-loop until 0 critical / 0 important" — R0 hits 0C/0I. P1 cycle ready to push & PR.
