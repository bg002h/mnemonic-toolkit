# export-wallet --format descriptor — End-of-Cycle R0 Review

**Verdict: GREEN (0C / 0I).** Cleared for tag `mnemonic-toolkit-v0.42.0`.

Whole-cycle gate over `master..HEAD` (Phase 1 code + Phase 2 docs/version-bump + Phase-2-R0 I1 fold). Reviewer (opus, full shell) built the v0.42.0 binary + 3 siblings and ran everything end-to-end. This R0 is also the re-dispatch that re-reviews the complete diff including the I1 fold (the per-phase R0s having converged).

## Critical
None.

## Important
None.

## Minor
1. **Untracked scratch in the working tree** (`stderr.txt`, `stderr2.txt`, `.claude/`, `CONTINUITY.md`, `cycle-prep-recon-*.md`, `feature-coverage-survey-*.md`). None staged, none in `master..HEAD`; the HEAD-based tag is unaffected and untracked files don't block `checkout`/ff. `stderr.txt`/`stderr2.txt` are stale scratch redirects → deleted by the controller before ship.
2. **`CheckedDescriptor::as_str` `#[allow(dead_code)]`** (`wallet_export/mod.rs:443`) — pre-existing, not from this cycle; `DescriptorEmitter` uses `.to_string()` (via `Display`), consistent with `green.rs`. No action.

## Verification ledger (reviewer ran every command)

**Code correctness**
- `DescriptorEmitter` (`src/wallet_export/descriptor.rs:11-25`): all 3 trait methods; `emit → Ok(inputs.canonical_descriptor.to_string())` (NO trailing `\n`; `Display` at `mod.rs:457-461` delegates verbatim; dispatch tail appends exactly one `\n` — `writeln!` `:567`, file `format!("{emitted}\n")` `:575`). No double-newline (the exact + multisig tests assert single-`\n`).
- All 5 exhaustive `match CliExportFormat` sites carry an explicit `Descriptor` arm, no `_`: `format_requires_template` `Descriptor => false` (`:58`); `run()` collect_missing (`:518`) + emit (`:560`); `run_from_import_json()` collect_missing (`:774`) + emit (`:816`).
- No over-reach: secret slot refused (exit 2, watch-only); multisig allowed (exit 0); taproot refused only on the from-import-json leg (`:675`), works direct/template.

**Recipe (manual §round-trip, all EXIT=0)**
- IN `bundle` → md1 + 2× mk1, no ms1, watch-only advisory.
- OUT single-sig (test seed): `wpkh([73c5da0a/84'/0'/0']xpub6CatWdiZiodmU…/<0;1>/*)#hpg6d6w2` — matches `:282`.
- OUT multisig from-import-json (sparrow fixture): `wsh(sortedmulti(2,[b8688df1/…]…,[28645006/…]…,[5436d724/…]…))#he0ej3xr` — order + `#he0ej3xr` match `:299`.
- Taproot passthrough: `tr([73c5da0a/86'/0'/0']…/<0;1>/*)#5tp3cj93`, exit 0.
- Taproot from-import-json refusal confirmed (`sparrow-singlesig-p2tr.json` → exit 1, documented error).

**make audit:** all 4 binaries + FIXTURES_DIR → 6 stages pass, 20 transcripts pass, literal **EXIT=0**.

**Version-bump completeness (v0.42.0):** `Cargo.toml:3`, `Cargo.lock:706`, both README markers (`README.md:13`, crate `:9`), both README `Status:` = `v0.42.x` (I1 fold landed; `README.md:14`, crate `:10`), `CHANGELOG.md` v0.42.0 entry (2026-06-03, MINOR), `scripts/install.sh:32` self-pin, manual `--format` value list `41-mnemonic.md:700`. `readme_version_current` → 1 passed.

**Regression:** `--no-fail-fast` FAILED count = **0** (877 in the largest suite; `cli_export_wallet_descriptor.rs` 8 tests). `clippy --all-targets -D warnings` → exit 0.

**Lockstep + hygiene:** no `mnemonic-gui` file touched (GUI v0.23.0 deferred post-tag); `cli_gui_schema.rs` asserts 11 export formats incl `descriptor`; `git diff --stat master..HEAD` = exactly the expected 21 files (4 code, 2 test, 5 version, 2 manual, 8 design); no `git add -A` residue.
