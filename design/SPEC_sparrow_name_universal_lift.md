# SPEC — universal source-wallet-name lift on `export-wallet --from-import-json`

- **Date:** 2026-05-28
- **Source SHA:** `2a36ee6` (toolkit `master`)
- **Status:** approved design (brainstorm + per-format provenance audit); **pending R0 to 0C/0I before any implementation.**
- **Type:** product behavior change (SemVer-**PATCH** — default-shift on a re-emit, no flag change). Resolves FOLLOWUP `sparrow-from-import-json-wallet-name-preservation`. No GUI lockstep (no clap surface change). Manual lockstep: chapter-45 prose + 6 transcript re-captures.

## Purpose
`export-wallet --from-import-json` without `--wallet-name` defaults to the static string `"imported-descriptor"` (`cmd/export_wallet.rs:693-696`), so a `format X → envelope → format X` round-trip silently loses the wallet's original name/label. Surfaced as the non-empty `diff` in the v0.37.7 manual-prose-gate cycle's sparrow transcript. The fix lifts the original wallet name from the envelope's already-emitted per-format source-metadata field whenever the user did not supply `--wallet-name` explicitly.

## Per-format audit (R0-folded against `origin/master` `2a36ee6`)
The envelope already carries 6 per-format wallet-metadata fields — emitter at `cmd/import_wallet.rs:1736-1882` writes them; deserializer (`wallet_import/json_envelope.rs:62-71`) currently drops them via the project-wide "Phase 5 does NOT need source_metadata" comment. **R0 C1 fold:** `coldcard_multisig_source_metadata` is NOT among the 6 today (no `ImportProvenance::ColdcardMultisig` accessor; never enters emit chain); this cycle ADDS that emit-side accessor + field so the lift is universal across all multisig formats. **R0 C2 fold:** Jade's wallet name lives at the *nested* `coldcard_compat.name` (jade delegates to coldcard-multisig parser); accessor must walk a path. **R0 C3 fold:** coldcard-singlesig has no source-side wallet name (`"name": "p2wpkh"` in the fixture is a BIP-derivation label, not a wallet name) — drop from lift scope, same disposition as BSMS.

| `source_format` | Envelope field | Source-side field (parser type + cite) | Carries usable name |
|---|---|---|---|
| `sparrow` | `sparrow_source_metadata` | `SparrowSourceMetadata.label: Option<String>` (`wallet_import/sparrow.rs:103-108, 466`) | yes (Optional) |
| `specter` | `specter_source_metadata` | `SpecterSourceMetadata.label: String` (`wallet_import/specter.rs:59-72`) | yes (mandatory) |
| `jade` | `jade_source_metadata.coldcard_compat` | `JadeSourceMetadata.coldcard_compat: ColdcardMultisigSourceMetadata` (`wallet_import/jade.rs:65-79`); read `.name` from the nested struct | yes (nested) |
| `electrum` | `electrum_source_metadata` | `ElectrumSourceMetadata.wallet_name: Option<String>` (`wallet_import/electrum.rs:100-112`) | yes (Optional) |
| `bitcoin-core` | `source_metadata` | `CoreSourceMetadata.wallet_name: Option<String>` (`wallet_import/mod.rs:318-329` — **R0 I2 fold**: spec previously cited `bitcoin_core.rs:220`, which is a function param) | yes (Optional) |
| `coldcard-multisig` | `coldcard_multisig_source_metadata` **(NEW — this cycle adds the emit-side + accessor; R0 C1 fold option (a))** | `ColdcardMultisigSourceMetadata.name: String` (`wallet_import/coldcard_multisig.rs:96-111`) | yes (mandatory) |
| `coldcard` (singlesig) | (none — `coldcard_source_metadata` exists but has no `name` field) | `ColdcardSourceMetadata` fields = `chain/xfp/bip_derivation/raw_account/dropped_fields` (`wallet_import/coldcard.rs:108-124`); no source-side wallet name | **no — falls through** (R0 C3 fold) |
| `bsms` | (none) | descriptor-only, no wallet name in source | no — falls through |

## Scope (decided — universal lift across 6 name-carrying formats)
Single clean pattern; "fix the class, not the instance." Coldcard-singlesig + BSMS fall through to the static `"imported-descriptor"` default (their sources don't carry wallet names — not asymmetry, just absence). Coldcard-multisig adds a new per-format envelope field this cycle (R0 C1 fold option (a)) so the lift covers every name-carrying multisig format uniformly.

## Architecture

### 1. Add coldcard-multisig provenance emit-side (R0 C1 fold)
**New work this cycle** — before the deserializer can lift, the emitter must produce the field:
- `wallet_import/mod.rs` — add `coldcard_multisig_source_metadata(&self) -> Option<&ColdcardMultisigSourceMetadata>` accessor on `ImportProvenance`, mirroring the existing per-format accessors (`coldcard_source_metadata`, `electrum_source_metadata`, `jade_source_metadata`, `sparrow_source_metadata`, `specter_source_metadata`).
- `cmd/import_wallet.rs:1736+` — add a `if let Some(meta) = p.provenance.coldcard_multisig_source_metadata() { ... }` block emitting `coldcard_multisig_source_metadata: { name, policy_k, policy_n, script_format, … }`. Mirrors the existing jade/coldcard/electrum/sparrow/specter blocks.

Additive wire-shape change — back-compat with existing consumers via serde drop-unknown.

### 2. Deserializer — extend `ImportJsonEnvelope`
`wallet_import/json_envelope.rs:62-71` — add 6 optional per-format `source_metadata` fields. Cheapest implementation uses `serde_json::Value` (avoid pulling in serde structs for each source-metadata shape, which the deser side doesn't need to read deeply):

```rust
pub(crate) struct ImportJsonEnvelope {
    pub(crate) schema_version: String,
    pub(crate) source_format: String,
    pub(crate) bundle: BundleJsonView,
    #[serde(default)] pub(crate) source_metadata: Option<serde_json::Value>,             // bitcoin-core
    #[serde(default)] pub(crate) sparrow_source_metadata: Option<serde_json::Value>,
    #[serde(default)] pub(crate) specter_source_metadata: Option<serde_json::Value>,
    #[serde(default)] pub(crate) jade_source_metadata: Option<serde_json::Value>,
    #[serde(default)] pub(crate) electrum_source_metadata: Option<serde_json::Value>,
    #[serde(default)] pub(crate) coldcard_multisig_source_metadata: Option<serde_json::Value>,
}
```

Note: NO `coldcard_source_metadata` field — coldcard singlesig has no source-side wallet name (R0 C3 fold). Drop the `// Phase 5 does NOT need source_metadata` comment — it's now wrong.

### 3. Uniform accessor — `ImportJsonEnvelope::resolved_wallet_name()`
Path-walking helper (R0 C2 fold — jade needs nested access):
```rust
impl ImportJsonEnvelope {
    /// The user-facing wallet name carried in the source's per-format metadata,
    /// dispatched on `source_format`. `None` when the source has no name field
    /// (BSMS, coldcard-singlesig) OR the field exists but is empty/null (treat
    /// `""` as None — we don't propagate an empty name through round-trip).
    pub(crate) fn resolved_wallet_name(&self) -> Option<String> {
        let (meta, path): (Option<&serde_json::Value>, &[&str]) = match self.source_format.as_str() {
            "sparrow"           => (self.sparrow_source_metadata.as_ref(),           &["label"]),
            "specter"           => (self.specter_source_metadata.as_ref(),           &["label"]),
            "jade"              => (self.jade_source_metadata.as_ref(),              &["coldcard_compat", "name"]),  // R0 C2
            "electrum"          => (self.electrum_source_metadata.as_ref(),          &["wallet_name"]),
            "bitcoin-core"      => (self.source_metadata.as_ref(),                   &["wallet_name"]),
            "coldcard-multisig" => (self.coldcard_multisig_source_metadata.as_ref(), &["name"]),
            _ => return None,  // bsms + coldcard (singlesig) + any future format default to no-lift
        };
        let mut cur = meta?;
        for k in path {
            cur = cur.get(k)?;
        }
        cur.as_str().filter(|s| !s.is_empty()).map(String::from)
    }
}
```

Treats empty string as None (don't lift `""`). Unknown `source_format` falls through to None.

### 4. Use site — `cmd/export_wallet.rs:693-696` + flag-semantics update (R0 I1 fold)
```rust
let lifted = envelope.resolved_wallet_name();
let wallet_name_resolved: String = args
    .wallet_name
    .clone()
    .or_else(|| lifted.clone())
    .unwrap_or_else(|| "imported-descriptor".to_string());
// EmitInputs:
wallet_name_is_non_default: args.wallet_name.is_some() || lifted.is_some(),
```

**R0 I1 + I3 fold — rename `wallet_name_was_user_supplied` → `wallet_name_is_non_default`** throughout. The field's purpose (prevent silent `"imported-descriptor"` slipping into Specter exports per `wallet_export/specter.rs:31-38`) is satisfied by either explicit user supply OR source-lift — the renamed field captures the actual semantics. Mechanical refactor: rename in the struct + one consumer at `wallet_export/specter.rs:34`. Update doc-comment at `cmd/export_wallet.rs:453-454` accordingly.

Explicit `--wallet-name` still wins. Source-name is the new mid-tier default; `"imported-descriptor"` remains the final fallback for sources without a name (coldcard-singlesig + BSMS).

## Wire compatibility (R0 M2 fold)
Envelope `schema_version` stays `"1"`. The added deserialize fields are `#[serde(default)]` Optional — strictly back-compat for readers; existing emitters already produce all 5 of the pre-existing fields. The new `coldcard_multisig_source_metadata` is additive (older readers ignore unknown fields). No consumer regression.

## Tests

### Unit (TDD — RED first)
Parametric set locked as `&[(format, path, expected_name)]` slice literals (R0 M4 fold) so any drift between match-arms and dispatch surfaces as test failure:
1. **`json_envelope_resolves_per_format_dispatch`** — parametric over the **6** name-carrying formats (sparrow/specter/jade/electrum/bitcoin-core/coldcard-multisig — R0 M3 fold), each with its respective field populated → all resolve correctly. Jade cell exercises the nested `coldcard_compat.name` path.
2. **`json_envelope_empty_string_returns_none`** — `sparrow_source_metadata: { label: "" }` → `None`.
3. **`json_envelope_electrum_multisig_empty_x1_label_returns_none`** (R0 I4 fold) — electrum multisig path where `wallet_name: ""` (parser path may emit empty) → `None`, not `Some("")`.
4. **`json_envelope_bsms_returns_none`** — `source_format: "bsms"` → `None`.
5. **`json_envelope_coldcard_singlesig_returns_none`** (R0 C3 fold) — `source_format: "coldcard"` with `coldcard_source_metadata` present but no `name` field → `None`.
6. **`json_envelope_missing_metadata_returns_none`** — sparrow with no `sparrow_source_metadata` field → `None`.
7. **`json_envelope_unknown_format_returns_none`** — defensive.

### Integration (per-format round-trip — 6 cells)
For each of 6 name-carrying formats, capture `import → --from-import-json → re-import` and assert round-trip wallet name equals source's name. New file `crates/mnemonic-toolkit/tests/cli_export_wallet_from_import_json_name_preservation.rs`. Jade cell verifies the nested-path accessor works end-to-end through real fixtures (not just unit tests).

### Explicit-override
`explicit_wallet_name_wins` — `--wallet-name "Custom" --from-import-json <env-with-sparrow-label-Daily>` → exported file's name field == `"Custom"`.

### Specter target — flag rename verification (R0 I1)
`specter_target_with_source_lift_no_explicit_flag_succeeds` — re-emit `--format specter` with `args.wallet_name == None` and source carrying a name → success (NOT `ExportWalletMissingFields`). Without the I1 fold, this cell fails.

### Existing-test regression risk
The v0.37.6 hygiene cycle adds the `--locked` CI guard; this cycle bumps Cargo.toml so `Cargo.lock` must regenerate. Snapshot tests that pinned the old `"imported-descriptor"` default for `--from-import-json` re-exports may need updating — surface in the regression run, fold inline.

## Manual lockstep
- **Re-capture all 6 chapter-45 transcripts** (`docs/manual/transcripts/foreign-formats/` under `docs/manual/src/` — R0 M1 fold: file is at `docs/manual/src/45-foreign-formats.md`, NOT `40-cli-reference/`). After the fix:
  - sparrow `roundtrip-sparrow-singlesig.out`: 146 bytes → empty (round-trip clean).
  - coldcard-multisig `roundtrip-coldcard-multisig.out`: name lift removes the `Name: TestMs2of3` divergence; text-format reordering portion of the diff may shrink or persist (re-capture; whatever the live output is becomes the new pinned expectation).
  - specter/jade/electrum/coldcard-SS (recipes have no terminal `diff`): unchanged (empty `.out`, exit 0).
- **Rewrite chapter-45 prose addendum** at `docs/manual/src/45-foreign-formats.md:~322` (post-v0.37.7 chapter line; re-verify at implement time). Old text: "`diff` is non-empty because `--wallet-name` omitted; tracked at FOLLOWUP `sparrow-from-import-json-wallet-name-preservation`." Replace: lift is now automatic (cite v0.37.8); `--wallet-name` overrides; FOLLOWUP cite becomes CHANGELOG pointer.
- **Chapter-45 coldcard-multisig addendum** at `~:582+9 = :591` (shifted by sparrow addendum + coldcard-MS addendum offsets): keep — text reordering portion is unrelated to name, but reword to remove the implicit "ALL non-empty diff is reordering" framing.
- **Verify `make audit` green** after re-capture (20 transcripts pass).

## Verification & ship
- Full crate suite green (`cargo test -p mnemonic-toolkit`) — surface + fold any snapshot drift inline.
- `cargo clippy --all-targets -D warnings` clean.
- `make -C docs/manual audit ...` green (20 transcripts).
- `cargo metadata --locked` succeeds (v0.37.6 guard).
- Phase-6 release: Cargo.toml/lock 0.37.8, both README markers, install.sh pin, CHANGELOG `[0.37.8]`, FOLLOWUP flip.
- Ship as `mnemonic-toolkit-v0.37.8`.

## R0 history
R0 (`design/agent-reports/sparrow-name-universal-lift-R0-review.md`): RED 3C/4I/4M → folded. C1 (coldcard-multisig: chose option (a) — add emit-side this cycle), C2 (jade nested path), C3 (drop coldcard-singlesig — no source-side name), I1+I3 (rename `wallet_name_was_user_supplied` → `wallet_name_is_non_default`; flip on lift), I2 (audit citations re-greped), I4 (electrum-multisig empty-x1 test cell), M1-M4 inline. Format count corrected 7 → 6. R1 re-dispatch pending.
