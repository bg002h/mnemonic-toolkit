# v0.8.1 Phase 3 R1 — reviewer report

## Verdict
**0C / 0I — converge**

## Verification

### 1. SPEC §8 field order and types

`crates/mnemonic-toolkit/src/wallet_export/specter.rs:50-55` — `SpecterWallet` struct declares fields `label`, `blockheight`, `descriptor`, `devices` in that order. `#[derive(Serialize)]` emits struct fields in declaration order (serde guarantee for structs, not maps). Fixtures `specter_single_wpkh.json` and `specter_multi_2of3.json` confirm the byte shape. Field types: `label: &str`, `blockheight: u32` (hardcoded 0), `descriptor: &str`, `devices: Vec<&'static str>` — all correct per SPEC §8.

### 2. `descriptor` includes `#checksum` (unlike Sparrow)

`inputs.canonical_descriptor` is populated from `pipeline::build_descriptor_string` (template path) or `MsDescriptor::to_string()` (descriptor passthrough). Both paths use miniscript's `Display` impl which produces BIP-380 form with `#checksum` suffix. Fixture `specter_single_wpkh.json` shows `#00lx6ere`; `specter_multi_2of3.json` shows `#he0ej3xr`. Cell_4 asserts `descriptor.contains("#")` as a targeted regression guard. Correct and intentionally opposite to Sparrow (which strips the checksum per Phase 2 R1 C-1).

### 3. `MissingField::WalletName` channel and refusal text byte-exact

`collect_missing` at `specter.rs:31-38` fires `MissingField::WalletName` when `!inputs.wallet_name_was_user_supplied`. The flag is set via `args.wallet_name.is_some()`. Traced `build_missing_fields_refusal("specter", &[MissingField::WalletName])` through `wallet_export/mod.rs:275-291`: produces `"mnemonic export-wallet --format specter requires the following missing fields:\n  - wallet_name (supply --wallet-name <STRING>)\nRe-invoke with all missing fields supplied."`. With `writeln!(stderr, "{}", e)` in `main.rs:83` prepending `"error: "` via `Display` and appending `\n`, the emitted stderr matches `specter_missing_wallet_name_refusal.stderr` byte-exact. Cell_3 exit code 2 is correct.

### 4. `devices` count — template path correct; `.max(1)` inert there

Template path: `resolved_slots.len()` equals cosigner count (singlesig = 1, 2-of-3 = 3). `.max(1)` is inert because a `n == 0` guard at `export_wallet.rs:263-266` refuses invocations with zero slots before `EmitInputs` is built. Fixture `specter_multi_2of3.json` shows three `"unknown"` entries. Correct.

### 5. `ExportWalletFormatStub` retention with `#[allow(dead_code)]`

`error.rs:93-99` — variant retained. `ToolkitError` is `pub` with `#[non_exhaustive]`; external consumers may reference the variant in exhaustive-style code. Retaining it with `#[allow(dead_code)]` costs nothing and avoids future confusion when a stub phase is needed again. Reasoning is sound.

### 6. Both dispatch sites wired correctly

`collect_missing` arm: `CliExportFormat::Specter => (SpecterEmitter::collect_missing(&inputs), "specter")`. `emit` arm: `CliExportFormat::Specter => SpecterEmitter::emit(&inputs)`. No stub arm remains. Correct.

### 7. Manual update

`docs/manual/src/40-cli-reference/41-mnemonic.md:167` — `--wallet-name` REQUIRED note correctly added for `--format specter` with accurate UX rationale. Mirrors SPEC §13 R1-L1 hardening. Manual mirror invariant satisfied.

### 8. Old cell_4 deletion

`cli_export_wallet.rs:210-213` — v0.7 `cell_4_specter_stub_refusal_byte_exact` deleted; explanatory comment left in place. Replacement coverage in `cli_export_wallet_specter.rs` (four cells). Correct.

## Findings

None above confidence threshold (80).

## Confidence-filtered: omitted

- `.max(1)` for `--descriptor` multisig passthrough: `resolved_slots` is empty on the descriptor-passthrough path, so `.max(1)` would produce 1 device for any multisig descriptor. SPEC §8 says "length matches cosigner count for multisig." However, fetching upstream `wallet_importer.py` confirms Specter does NOT validate devices array length against descriptor cosigner count — import succeeds regardless; unmatched cosigners surface as `unknown_cosigners` in Specter's UI. This is a UX gap for the edge case of descriptor-passthrough multisig, not a functional import failure. No test covers this combination. Confidence 65 — omitted.
- Cell_4 `serde_json::from_str(&stdout).unwrap()` after the byte-exact `assert_eq` is redundant in the success path but harmless as a targeted invariant check. Confidence 30 — not a defect.
