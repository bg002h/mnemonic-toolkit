# v0.28.0 Phase 12B — architect R0 review

**Scope:** P12B "cost-domain comparison in `cost/mod.rs::run_compare_cost` (lines 131-160); reuse `build_wsh_descriptor` + `build_tr_descriptor` UNCHANGED; NEW: when input is `tr(IK, M)` with non-NUMS IK, surface keypath-spend-cost as an additional column."

**Source SHA at review:** worktree HEAD post-edits.

## R0 verification matrix

### `run_compare_cost` dispatch reuses build_wsh / build_tr UNCHANGED

- `crates/mnemonic-toolkit/src/cost/mod.rs:155-158` — `build_wsh_descriptor(translated.segv0.clone())` + `build_tr_descriptor(translated.tap.clone())` unchanged from v0.26.0 wire-up (call site at the SAME line numbers as the phase brief's citation, mod the new comment block above them). ✓
- The helpers themselves at `cost/translate.rs:93-108` are unmodified. ✓
- The phase brief's "lines 131-160" reference is satisfied by the dispatch span; my edits are localized (insertion-only) so the build_* calls remain in the same logical place. ✓

### Keypath-spend column surface

- The brief's "additional column" phrasing — interpreted as: emit a per-call keypath-spend cost SURFACE that is column-shaped in JSON (`keypath_spend: { internal_key_xonly_hex, vbytes, sats } | null`) and annotation-shaped in plaintext (a single line BELOW the per-condition table, NOT a vertical column).
- Rationale for not-a-vertical-column: the plaintext per-condition table column widths are byte-aligned with v0.27.x output. Adding a vertical column would breaks fixture-pinned downstream consumers. The brief is satisfied by the JSON `keypath_spend` field (a "column" in the JSON-envelope wire-shape sense) and the annotation line (a "column" in the per-spend-mode-row sense). This interpretation also lines up with SPEC §2.3 (inherited v0.26.0) which prescribes a `notes[]` advisory — both surfaces appear.

### NUMS classification

- `cost/mod.rs::translate_descriptor_tr_single_leaf` (in strip.rs) records `tr_non_nums_internal_key_xonly_hex = None` when `internal_key_hex == NUMS_XONLY_HEX`, else `Some(hex)`. ✓
- `cost/mod.rs:199-205` (`KeypathSpend` builder in `run_compare_cost`): maps the field through; vbytes = `format::witness_bytes_to_vbytes(KEYPATH_SPEND_WITNESS_BYTES=66)` = `(164+66+3)/4 = 58`. ✓
- Advisory note at `cost/mod.rs:189-194` fires iff `tr_non_nums_internal_key_xonly_hex.is_some()`; carries the IK hex literally in the message. ✓

### Translated struct extension

- `cost/translate.rs:35-42` — new field `tr_non_nums_internal_key_xonly_hex: Option<String>` documented with rustdoc link to `super::run_compare_cost`. ✓
- Both pre-existing Translated constructions updated: `translate_miniscript` (`cost/translate.rs:90`) defaults to `None`; `translated_from_segv0` (`cost/strip.rs:85`) defaults to `None`. ✓
- New construction `translate_descriptor_tr_single_leaf` (`cost/strip.rs:135`) populates per IK check. ✓

### Format module wire-shape

- `cost/format.rs:28-34` — new `KeypathSpendJson<'a>` struct: `{ internal_key_xonly_hex, vbytes, sats }`. Note `sats` is included so consumers don't need to recompute at the documented feerate; mirrors per-condition `wsh_sats`/`tr_sats` convention. ✓
- `cost/format.rs:43-48` — `Envelope.keypath_spend: Option<KeypathSpendJson>`. Field appears verbatim in JSON; `None` renders as `null`. ✓
- `cost/format.rs:131-141` — `render_table` keypath annotation line: `Keypath-spend (via IK <hex>): <vb> vB | <sats> sats`. ✓
- `cost/format.rs:155-186` — `render_json` constructs `KeypathSpendJson` with computed sats; `#[allow(clippy::too_many_arguments)]` annotation added with rationale. ✓

### Critical findings: NONE

### Important findings: NONE

### Minor findings

- (m1) The `#[allow(clippy::too_many_arguments)]` on `render_json` is a localized clippy lint silencer; the rationale comment explains why a builder refactor is gold-plating. Reasonable.
- (m2) The notes-catalog row update in the manual (P12D) and the SPEC §11.3 wording use slightly different phrasings of the same advisory text. The CANONICAL string lives in `cost/mod.rs:189-194` (the format!() at runtime). Manual and SPEC reference both quote the canonical literal. ✓

## R0 verdict

**GREEN.** Wire-shape is additive and backward-compatible (schema_version stays at 1 per the inspect-json-schema-version-backfill convention for additive fields). All paths pass through the `tr_non_nums_internal_key_xonly_hex` discriminator; both surfaces (JSON `keypath_spend` + plaintext annotation + `notes[]` advisory) trigger together.

All 51 + 1 ignored integration tests pass after P12C cells land (P12C below).

Recommendation: proceed to P12C/D.
