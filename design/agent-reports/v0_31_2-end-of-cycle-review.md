# v0.31.2 end-of-cycle architect review

**Reviewer:** opus (feature-dev:code-reviewer)
**Round:** end-of-cycle
**Cycle:** Cycle 9 (sparrow-taproot-singlesig template-mode import)
**Date:** 2026-05-21
**Pre-tag SHAs reviewed:**
- Phase 2 (code): `8d67bdc`
- Phase 3 (integration tests + fixture): `8d9fe4b`
- Phase 4 (manual): `55aaaa6`
- Phase 5 (uncommitted on disk at review time): Cargo.toml 0.31.1→0.31.2 + install.sh self-pin + CHANGELOG entry + Cargo.lock

## Verdict

**GREEN.** The v0.31.2 Cycle 9 changes are coherent, minimal, and ready to tag. Behavior-expansion-only PATCH bump verified end-to-end against source.

## Critical (C)

None.

## Important (I)

None.

## Minor (M)

**M1 — `has_at_placeholder` substring is narrow but sound, undocumented edge-case.** `crates/mnemonic-toolkit/src/wallet_import/sparrow.rs:338` matches only the literal `@0/**`. For a hypothetical Sparrow MULTISIG blob that emits `@N/**` with `N ≥ 1` and no `@0/**` (e.g. a 2-of-2 starting key at index 1), `is_descriptor_passthrough` would mis-classify as passthrough and skip Step 5 substitution. This is hypothetical — Sparrow's emit always starts cosigner indexing at 0 (`wallet_export/sparrow.rs` builds placeholders from `(0..n)`), and the leftover-placeholder regex at sparrow.rs:383-389 would catch any stray `@N/**` and surface as a parse error rather than feeding garbage downstream. Confidence the current behavior is correct: high. Suggest filing a defensive FOLLOWUP for v0.32+ to widen detection to `Regex(r"@\d+/\*\*")` for robustness against future emit-side drift. Informational.

**M2 — `tr(` substring overlap with `addr(` / `pkh(` / etc. is non-existent in practice** but technically a `multi_a(...)` wrapping inside `wsh(...)` could not collide. Not actionable.

## Verifications passed

- **Step 5 substitution correctness** (`sparrow.rs:341-378`): for Bip86 `tr(@0/**)` + `derivation=m/86'/0'/0'`, `path_no_m="86'/0'/0'"`, bracketed form is `[5436d724/86'/0'/0']xpub.../<0;1>/*`, substituted body `tr([5436d724/86'/0'/0']xpub.../<0;1>/*)`. Boundary cell at `tests/cli_import_wallet_sparrow_taproot.rs:122-129` asserts exactly this.
- **Path-split discriminator preservation** (`sparrow.rs:337-339`): `is_descriptor_passthrough = has_tr && !has_at_placeholder` unchanged; Cycle 8's multisig branch at `sparrow.rs:350-352` (`script_template.clone()`) preserved verbatim. `tr_multi_a_nums_2of3_imports_successfully` regression cell intact.
- **Test coverage**: 1 in-file unit cell (`sparrow.rs:931`) + 1 integration cell `taproot_singlesig_template_imports_via_substitution` (`cli_import_wallet_sparrow_taproot.rs:105`) + 1 fixture-driven cell + boundary cell + fixture file. Adequate. Round-trip narrowing to orthogonal-boundary cell is sound: the export-side gate at `cmd/export_wallet.rs:622-632` empirically refuses ALL taproot envelopes — round-trip is blocked by a separate FOLLOWUP, not swept under the rug.
- **Manual chapter** (`45-foreign-formats.md:321-357`): anchor `#taproot-import-shipped-v0311` preserved at line 321; describes both branches; deferrals-list bullet at line 837-842 correctly strikethroughs and cross-references. No stale v0.31.1-only narrowing prose found.
- **CHANGELOG** (`CHANGELOG.md:9-38`): cites correct FOLLOWUP slug + correct cell counts + R0 GREEN. Accurate.
- **SemVer**: behavior-expansion-only with no clap/schema/wire change → PATCH correct. `Cargo.toml:3` + `Cargo.lock:637` + `install.sh:32` all `0.31.2`.
- **Orthogonal taproot-refusal logic**: grepped all 13 `wallet_import` files. Only stale paths are `bsms.rs::BsmsTaprootImportRefused` (intentional BIP-129 gate, unchanged) and `cmd/export_wallet.rs:622` (export-side; intentionally still refuses — exactly the gap documented by boundary cell). No missed parser updates.
- **GUI lockstep**: no clap surface change (no new flags / no subcommand / no enum value change); schema_mirror unaffected. Toolkit-only correct.

## Action items

- File a follow-on FOLLOWUP `sparrow-import-detection-regex-defensive-widening` capturing M1 for v0.32+ scope (treat as defensive hardening, not load-bearing). Filed at cycle close.
