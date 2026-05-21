# Cycle 8 — P0 STRICT-GATE recon dossier

**Date:** 2026-05-21
**Cycle target:** `mnemonic-toolkit-v0.31.1` (PATCH; behavior expansion of previously-refused inputs).
**Source SHA at recon time:** master HEAD `4eb1fa8` (post-Cycle-7-close; in sync with origin/master).
**FOLLOWUP slug:** `sparrow-taproot-descriptor-passthrough-import-support` (`design/FOLLOWUPS.md:2619`).

## A1 — Current refusal site (primary-source verified)

`crates/mnemonic-toolkit/src/wallet_import/sparrow.rs:304-315` (at HEAD `4eb1fa8`):

```rust
// Step 6 (early): refuse taproot scripts. Sparrow's emit at
// `wallet_export/sparrow.rs:215-219` ships taproot as descriptor-
// passthrough (no `@N/**` placeholder shape — the canonical
// descriptor with `[fp/path]xpub` keys is embedded directly). The
// P1B `@N/**` substitution path does not handle that shape; taproot
// import lands in a future cycle (cycle-followup
// `sparrow-taproot-descriptor-passthrough-import-support`).
if script_template.contains("tr(") {
    return Err(ToolkitError::ImportWalletParse(
        "import-wallet: sparrow: parse error: taproot scripts are not yet supported (Sparrow's taproot emit uses descriptor-passthrough; P1B's @N/** substitution path does not cover it)".to_string(),
    ));
}
```

**Cycle 8's job: convert this refusal into a path-split.** When `script_template` contains `tr(` AND has descriptor-passthrough shape, skip Step 5 substitution and feed directly through the existing `concrete_keys_to_placeholders` → `parse_descriptor` pipeline.

## A2 — Detection heuristic

Per the existing `wallet_export/sparrow.rs:215-219` emit-side comment + the locked fixture at `tests/export_wallet/sparrow_tr_multi_a_nums_2of3.json`:

- **Template mode** (current happy path): `script_template` contains `@N/**` placeholders. Step 5 substitutes them with `[fp/path]xpub/<0;1>/*` derived from each `keystores[i]`.
- **Descriptor-passthrough mode** (Cycle 8 new path): `script_template` contains concrete `[fp/path]xpub` keys directly — NO `@N/**` placeholders. Step 5 skipped; feed directly into `concrete_keys_to_placeholders`.

**Detection heuristic:** `!script_template.contains("@0/**")`. Equivalently: presence of `[` (origin bracket) AND absence of `@0/**`.

Robustness check: what if a non-taproot wallet is also descriptor-passthrough? Per `wallet_export/sparrow.rs:215-219`, only `CliTemplate::TrMultiA | TrSortedMultiA` ship as descriptor-passthrough; all other templates ship with `@N/**` placeholders. So absence of `@0/**` is a reliable marker for descriptor-passthrough (currently only emitted by taproot multisig).

## A3 — Round-trip fixture available

`crates/mnemonic-toolkit/tests/export_wallet/sparrow_tr_multi_a_nums_2of3.json` (verified at HEAD `4eb1fa8`):
- `policyType: "MULTI"`, `scriptType: "P2TR"`.
- `defaultPolicy.miniscript.script`: `tr(50929b74...(NUMS),multi_a(2,[b8688df1/87'/0'/0']xpub6FQya...,[28645006/87'/0'/0']xpub6DnEBN...,[5436d724/87'/0'/0']xpub6Buxw9...))` — full descriptor-passthrough.
- `keystores[]`: 3 cosigners with `masterFingerprint` + `derivation` + `extendedPublicKey`.

This fixture is the canonical SPEC §7 cell 5 emit-side byte-exact test. **Import-side reuse:** rename + relocate to `tests/fixtures/wallet_import/sparrow-tr-multi-a-nums-2of3.json` OR symlink (latter avoids byte drift between emit + import sides).

Existing emit test at `tests/cli_export_wallet_sparrow.rs:200`: `cell_5_sparrow_tr_multi_a_nums_2of3_byte_exact` validates the emit path. Cycle 8 adds the parallel import-side cell.

## A4 — `concrete_keys_to_placeholders` signature (primary-source verified)

`crates/mnemonic-toolkit/src/wallet_import/pipeline.rs:52`:
```rust
pub(crate) fn concrete_keys_to_placeholders(
    descriptor: &str,
) -> Result<(String, Vec<ParsedKey>, Vec<ParsedFingerprint>), ToolkitError>
```

Takes a descriptor with concrete `[fp/path]xpub` keys, returns a placeholder-form descriptor + parsed keys + fingerprints. **Already handles taproot descriptors per its regex** (the function is used by all parsers post-substitution; it doesn't care about taproot specifically).

The Cycle 8 fork point: feed `script_template` (already concrete-keys-form for descriptor-passthrough) directly into this function, skipping the `@N/**` substitution loop.

## A5 — Cargo deps + lib.rs

Zero net-new deps. Zero `ToolkitError` variants. Zero `lib.rs` changes. Cycle 8 is a pure parser refactor in `wallet_import/sparrow.rs`.

## A6 — Test coverage scope

Per Cycle 7 lessons + the existing sparrow.rs test suite at `crates/mnemonic-toolkit/tests/cli_import_wallet_sparrow.rs`:

- **Happy path cells:** TrMultiA (2-of-3 NUMS) + TrSortedMultiA — both via the existing emit-side fixture round-trip.
- **Sniff-positive sanity:** descriptor-passthrough wallet still sniffs as `sparrow` format (no sniff change needed; sniff is policyType-based).
- **No regression:** existing non-taproot template-mode wallets (wpkh, wsh-sortedmulti) still parse correctly.

## A7 — GUI lockstep status

NOT REQUIRED. No clap surface change → no `schema_mirror` delta → no GUI tag bump. Documented per Cycle 6 Path-A precedent.

## A8 — Manual chapter update scope

`docs/manual/src/45-foreign-formats.md` — current §"Sparrow Wallet" subsection cites the taproot refusal as a deferred behavior. Convert to v0.31.1-shipped strikethrough analogous to Cycle 6 + Cycle 7 closure precedents (`~~taproot scripts are not yet supported~~ — shipped in v0.31.1 via descriptor-passthrough import; reuses `concrete_keys_to_placeholders`...`).

## Recon verdict

**GREEN.** All decisions have primary-source backing:

- Refusal site identified at `sparrow.rs:311` (exact line; not stale-trusted).
- Detection heuristic verified against `wallet_export/sparrow.rs:215-219` emit-side.
- Round-trip fixture available (no fixture authoring needed; reuse existing emit-side fixture).
- `concrete_keys_to_placeholders` already supports taproot descriptors per its regex.
- Zero new deps / variants / lib.rs / CLI surface.

**SemVer-PATCH** (behavior-expansion of previously-refused inputs; no breaking change).
