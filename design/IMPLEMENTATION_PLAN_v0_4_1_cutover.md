# mnemonic-toolkit v0.4.1 implementation plan — schema-4 cutover + Phase E + Phase G

**Cycle scope:** land the three v0.4.0 deferrals from `design/FOLLOWUPS.md`:
- `bundle-json-schema-4-cutover` (Phase D-cutover; sub-deliverables a–g).
- `engraving-card-unified-1-master-card` (Phase E).
- `verify-bundle-9-3plus6n-forensics` (Phase G).

**Authoritative SPEC:** `design/SPEC_mnemonic_toolkit_v0_4.md` §5.5, §5.6, §5.7, §5.8, §6.6, §6.7.
**Pre-Phase audit:** transcript-only; key findings inlined into phase tasks below.

**Discipline:**
- TDD-first per phase; tests written before impl (per `feedback_iterative_review_every_phase`).
- Per-phase architect review at end-of-phase (mid-phase only for Phase H since it's cross-cutting).
- Iterate to 0C/0I per round; max r4.
- Per-implementation-phase reports persist to `design/agent-reports/phase-<id>-<slug>-review-r<N>.md`.
- L/nit findings route to `design/FOLLOWUPS.md` at `v0.4.2-nice-to-have` (new tier).

## Locked decisions (carried into Phase H)

- **BIP-388 path normalization (FOLLOWUP `bip388-distinctness-path-normalization-phase-b-decision`):** RAW-STRING comparison per SPEC §4.11.b literal text. Switch `check_key_vector_distinctness` to compare the raw user-supplied path string (preserved as `String` on `CosignerKeyInfo`). For legacy v0.2/v0.3 paths the raw string is the parsed/canonical form; for v0.4.1 slot-driven paths the raw string is the `--slot @N.path=` value. SPEC §4.11.b prose updated in lockstep to clarify `h`-vs-`'` notation behavior.
- **VerifyBundleJson.schema_version (Finding F from audit):** the verify-bundle response envelope tracks the v0.4.1 tool version (always emit `"4"` from v0.4.1 onward); does NOT mirror the input bundle's schema_version. Rationale: the verify-bundle response envelope is the tool's contract with the user, not a passthrough of the input. Schema-4 verify response includes the new forensic fields uniformly across schema 2/3/4 inputs.
- **`MsField` for single-sig watch-only:** `[""]` (one empty-string sentinel), per SPEC §5.8 example. Pure watch-only multisig N=3: `["", "", ""]`. NOT `[]`.
- **`mode_str` derivation post-cutover:** `"full"` iff `bundle.ms1.iter().any(|s| !s.is_empty())`. The presence of any non-empty ms1 element means at least one slot is secret-bearing.
- **`--ms1` CLI clap flag in verify-bundle:** migrate `Option<String>` → `Vec<String>` with `ArgAction::Append`. Existing single-value `--ms1 <s>` invocations continue to work (clap accepts the single occurrence as a 1-element vec). For schema-4 multi-source verification, repeating `--ms1` per slot in slot-index order; `--ms1 ""` for watch-only slot positions.

## Phase H (D-cutover): BundleJson schema-4 + multi-source synthesis + bundle::run rewiring

**Goal:** land sub-deliverables (a)-(g) of `bundle-json-schema-4-cutover` atomically. Tests-then-impl per task.

### H.1 — `Bundle.ms1` and `BundleJson.ms1` migration to `Vec<String>`

`crates/mnemonic-toolkit/src/synthesize.rs:20-24` — change `ms1: Option<String>` → `ms1: Vec<String>`. Update each of the 5 Bundle producers (audit §1):
- `synthesize_full` (synthesize.rs:128): `vec![ms1]`.
- `synthesize_watch_only` (synthesize.rs:160): `vec!["".into()]` (single-sig watch-only, N=1).
- `synthesize_descriptor` (synthesize.rs:245): `vec![ms1]` if entropy present else `vec!["".into()]`.
- `synthesize_multisig_full` (synthesize.rs:375): hard-rejected for cosigner_count > 1; with cosigner_count==1 (degenerate), `vec![ms1]`.
- `synthesize_multisig_watch_only` (synthesize.rs:527): `vec!["".into(); cosigner_count]`.

`crates/mnemonic-toolkit/src/format.rs:124-146` — change `BundleJson.ms1: Option<String>` → `ms1: MsField`. Bump `schema_version: &'static str = "3"` → `"4"` at all 3 hardcoded sites (format.rs:124, verify_bundle.rs:224, 260). Remove `#[allow(dead_code)]` from `MsField` at format.rs:57.

Update derived sites that pattern-match `Option`:
- `cmd/bundle.rs::descriptor_mode_emit:1162` `bundle.ms1.is_some()` → `.iter().any(|s| !s.is_empty())`.
- `cmd/bundle.rs::descriptor_mode_emit:1230` same `mode_str` derivation in text-mode branch.
- `cmd/bundle.rs::emit:583` text-mode `if let Some(ms1) = bundle.ms1.as_deref()` → iterate Vec, skip empties.
- `cmd/bundle.rs::emit_multisig:950` same.
- `cmd/bundle.rs::descriptor_mode_emit:1230` text-mode rendering — same iteration.
- `cmd/verify_bundle.rs::descriptor_mode_verify_run:1367` `expected.ms1.as_deref()` — REPLACE WITH SHIM in Phase H to keep build green: `expected.ms1.first().map(|s| s.as_str()).filter(|s| !s.is_empty())`. Phase J's full refactor of `descriptor_mode_verify_run` then supersedes the shim with the proper per-slot ms1 check. (Per r1 review B1: shim is required so H.1 lands without a compile break against the existing Phase J-deferred verify-bundle code. Per r2 nit: shim diverges from v0.4.0 semantics for the impossible `Some("")` case — `Some("")` becomes `None` instead of triggering a "fail" check; harmless in practice since synthesis never produced `Some("")` and the shim is short-lived. Add an inline `// shim: Some("") routes to "skipped" not "fail"; impossible under v0.4.0 producers; superseded in Phase J.` at the call site.)

Update synthesize.rs unit tests that assert `bundle.ms1.is_some()/is_none()/as_ref().unwrap()` (audit §1 lists 6 sites: synthesize.rs:567,569,582,855,870,883,896).

**Tests (TDD red-then-green):**
- ≥8 unit tests in synthesize.rs covering each producer's new ms1 shape.
- 1 unit test pinning `BundleJson.schema_version == "4"`.

### H.2 — JSON integration test assertion updates

Per audit §4, 8 specific assertions across 2 files break under schema 4:
- `cli_json_envelopes.rs:26` `schema_version "3"` → `"4"`.
- `cli_json_envelopes.rs:33` `v["ms1"].as_str()` → `v["ms1"][0].as_str()` (length-1 array).
- `cli_json_envelopes.rs:86` `schema_version "3"` → `"4"`.
- `cli_descriptor_mode.rs:33` `schema_version "3"` → `"4"`.
- `cli_descriptor_mode.rs:37` `v["ms1"].as_str()` → `v["ms1"][0].as_str()`.
- `cli_descriptor_mode.rs:67` `assert_eq!(v["ms1"], Value::Null)` → `assert_eq!(v["ms1"], json!([""]))`.
- `cli_descriptor_mode.rs:92` `bundle["ms1"].as_str()` → `bundle["ms1"][0].as_str()`.
- `cli_descriptor_mode.rs:134` `verify["schema_version"] "3"` → `"4"`.
- `cli_descriptor_mode.rs:162` same `bundle["ms1"]` extraction.

### H.3 — `synthesize_multisig_multisource` (new function)

`crates/mnemonic-toolkit/src/synthesize.rs::synthesize_multisig_multisource` — for `BundleMode::MultisigMultiSource` (N≥2, every slot secret-bearing).

**Signature:**
```rust
pub fn synthesize_multisig_multisource(
    slots: &[ResolvedSlot],     // length N; each carries (xpub, fingerprint, path, entropy)
    template: CliTemplate,
    threshold: u8,
    network: CliNetwork,
    account: u32,
    path_family: MultisigPathFamily,
    privacy_preserving: bool,
) -> Result<Bundle, ToolkitError>
```

**`ResolvedSlot` struct (new):** the post-binding shape carrying all per-slot keymaterial:
```rust
pub struct ResolvedSlot {
    pub xpub: Xpub,
    pub fingerprint: Fingerprint,
    pub path: DerivationPath,
    /// Raw user-supplied path string (for SPEC §4.11.b BIP-388 raw-equality).
    pub path_raw: String,
    /// `Some(entropy_bytes)` for secret-bearing slots; `None` for watch-only.
    pub entropy: Option<Vec<u8>>,
}
```

**Algorithm:**
1. Validate N == slots.len(); each slot has Some(entropy).
2. Compute md1 from descriptor (template-driven; reuse `template.wrapper_node(threshold, N)`).
3. For each slot @i: derive (xpub, fingerprint, path) per slot's source (already done by binding); encode ms1 per-slot from slots[i].entropy.unwrap(); encode mk1 per-slot.
4. Output: `Bundle { ms1: vec![ms1_0, ms1_1, ...], mk1: MkField::Multi(per_cosigner_chunks), md1 }`.

### H.4 — `synthesize_multisig_hybrid` (new function)

Same signature as H.3 but accepts mixed slots (some with entropy, some without). Per SPEC §5.8, ms1[i] = encoded ms1 for secret-bearing slot, `""` for watch-only.

### H.5 — `bundle::run` top-level dispatch rewiring

`crates/mnemonic-toolkit/src/cmd/bundle.rs::run` — add a new early branch (per r1 review B2: gate is `--slot` non-empty ONLY; legacy `--phrase`/`--xpub`/`--cosigner` retain legacy dispatch unchanged in v0.4.1; full SPEC §6.6.a alias migration deferred to v0.5+ via FOLLOWUP `legacy-flag-deprecation`):

```rust
// v0.4.1 unified slot-driven dispatch. Activates when --slot is supplied.
// Legacy --phrase / --xpub / --cosigner remain on the v0.3 dispatch path
// for v0.4.1 (deprecation deferred to v0.5+).
if !args.slot.is_empty() {
    return bundle_run_unified(args, stdin, stdout, stderr);
}
// Else fall through to existing legacy dispatch (unchanged).
```

A user supplying BOTH `--slot @0.phrase=X` AND legacy `--phrase Y` enters the unified dispatch (because `args.slot` is non-empty); inside `bundle_run_unified`, `expand_legacy_to_slots` fires the SPEC §6.6 row-6 conflict (`--phrase deprecated; cannot combine with --slot @0.phrase=`). Behavior is unambiguous in this case.

**`bundle_run_unified`:** new function that:
1. `expand_legacy_to_slots(args.slot, args.phrase, ...)` → effective slots.
2. `validate_slot_set(&slots)?`.
3. `detect_bundle_mode(&slots)?` → `BundleMode`.
4. Pre-checks: `pre_check_threshold(args.threshold, n, template.map(|t| t.human_name()))?`; `pre_check_template_n(template.human_name(), template.is_multisig(), n)?` (when template is Some).
5. Bind slots → ResolvedSlot vector.
6. Dispatch on BundleMode:
   - `SingleSigFull` → call adapter that produces a Bundle equivalent to legacy `bundle_full`.
   - `SingleSigWatchOnly` → adapter to `bundle_watch_only`.
   - `MultisigMultiSource` → `synthesize_multisig_multisource`.
   - `MultisigWatchOnly` → adapter to legacy `synthesize_multisig_watch_only`.
   - `MultisigHybrid` → `synthesize_multisig_hybrid`.

Legacy paths (steps 2-6 in audit §6) remain intact; the unified branch fires only when `args.slot` is non-empty.

**Critical scope note:** the SPEC §9 v0.4.0 release goal of "deprecating legacy --phrase/--xpub aliases entirely" is too risky for v0.4.1; v0.4.1 introduces the unified path as opt-in via `--slot`. Full legacy-flag deprecation deferred to v0.5+ via a new FOLLOWUP `legacy-flag-deprecation`.

### H.6 — BIP-388 path-normalization switch to raw-string

Change `check_key_vector_distinctness` (parse_descriptor.rs:1041): comparison key from `(xpub.to_string(), path.to_string())` to `(xpub.to_string(), path_raw.clone())`. Per r1 review I1: **`CosignerKeyInfo` ITSELF gains `path_raw: String`** (not just `ResolvedSlot`), since `synthesize_descriptor` and the existing legacy path also flow CosignerKeyInfo through `check_key_vector_distinctness`. The `path_raw` source-of-truth at `bind_descriptor_keys` (parse_descriptor.rs::bind_descriptor_keys) is the descriptor placeholder's `@N.path` annotation as parsed (string available before DerivationPath conversion); for slot-driven paths it's the `--slot @N.path=` value verbatim; for cases with no explicit path annotation the fallback is `path.to_string()` (canonical form — matches v0.4.0 behavior so no regression). Update SPEC §4.11.b prose to clarify raw-string equality means "the user's literal path-syntax bytes as supplied; `48h/0h` and `48'/0'` compare unequal under raw-string equality." Add a `CosignerKeyInfo → ResolvedSlot` retirement task to FOLLOWUP `cosigner-keyinfo-resolved-slot-merge` at tier `v0.4.2`.

**Tests:** add 2 new unit cases:
- Same xpub, paths `48h/0h/0h/2h` vs `48'/0'/0'/2'` → ACCEPTED (different raw strings under raw-string equality).
- Same xpub, paths both `48'/0'/0'/2'` → COLLISION.

### H.7 — SPEC cross-check (Phase H end-of-phase task)

Verify SPEC §5.6, §5.8, §4.11.b match the implementation. Update §4.11.b for the raw-string normalization decision.

**Phase H architect review checkpoints:** mid-phase after H.5 (rewiring is the highest-risk surgery); end-of-phase after H.7.

## Phase I (E): unified engraving card

**Goal:** SPEC §5.5 — single master card per bundle; `BundleInputForCard` shape; `engraving_card_unified` render function.

### I.1 — `BundleInputForCard` struct (new)

`crates/mnemonic-toolkit/src/format.rs::BundleInputForCard`:

```rust
pub struct BundleInputForCard {
    pub network: &'static str,
    pub template_or_descriptor: TemplateOrDescriptor,
    pub threshold: Option<u8>,             // Some for multisig, None for single-sig
    pub n: u8,
    pub language: Option<CliLanguage>,     // None for watch-only
    pub passphrase_used: bool,
    pub privacy_preserving: bool,
    pub per_slot: Vec<SlotCardBlock>,
    pub md1_chunk_set_id: String,          // hex of first 4 chunk_set_id bytes
}

pub enum TemplateOrDescriptor {
    Template(&'static str),
    Descriptor(String),
}

pub struct SlotCardBlock {
    pub index: u8,
    pub ms1_card_id: Option<String>,       // 4-hex chunk_set_id for ms1 (None if watch-only)
    pub mk1_card_id: String,               // 4-hex chunk_set_id for mk1
    pub fingerprint: Option<String>,       // None under privacy_preserving
    pub origin_path: Option<String>,       // None if absent
}
```

**Per r1 review B3 — `ms1_card_id` extraction:** ms-codec encoded strings do not embed a chunk_set_id in a separately parseable header (no equivalent of `mk_codec::string_layer::StringLayerHeader`). Therefore `SlotCardBlock.ms1_card_id` is computed from the same `policy_id_stub` derivation already used for `mk1_card_id`: `derive_mk1_chunk_set_id(&policy_id_stub)` → 20-bit value → render as 4-hex. Both ms1 and mk1 card_ids for slot @i thus share the same 4-hex prefix per BIP-93 — consistent, identifiable, no ms-codec API gap. Watch-only slots (ms1[i] == "") get `ms1_card_id: None`. Per r2 nit: `derive_mk1_chunk_set_id` is `pub(crate)` in `synthesize.rs`, accessible from `format.rs` via `use crate::synthesize::derive_mk1_chunk_set_id;` (sibling crate-root modules). Add this import explicitly when Phase I lands.

### I.2 — `engraving_card_unified` render function (new)

Per SPEC §5.5 layout:
1. Header line with template/descriptor summary + network.
2. Threshold line (multisig only).
3. Cosigners block: N indented lines.
4. Template OR descriptor line (truncate descriptor at 80 chars per SPEC §5.5).
5. md1 reference line.
6. Recovery hint line.

Privacy-preserving rendering: `anon` for fingerprints.

### I.3 — Wire from bundle/descriptor paths

Replace each of the 4 `engraving_card(...)` call sites (audit §5) with `engraving_card_unified(BundleInputForCard { ... })`.

### I.4 — Deprecate `EngravingMode::*` variants + `engraving_card` function

Mark old function and enum with `#[deprecated]` + comment "removed in v0.5+ once all call sites migrated"; remove old format.rs unit tests (3 sites: `engraving_card_full_no_passphrase_byte_exact`, `engraving_card_with_passphrase_uses_uppercase_USED`, `engraving_card_watch_only_omits_ms1`). Add new unit tests for the unified card layout.

### I.5 — SPEC cross-check.

**Phase I architect review:** end-of-phase only.

## Phase J (G): verify-bundle 9 / 3+6N parity + per-cell forensics + schema-4 dispatch

**Goal:** SPEC §5.7. Replace v0.3's 3-element coarse ladder for descriptor mode with full 9 / 3+6N schema; add per-cell forensic fields; schema-4 dispatch.

### J.1 — `VerifyCheck` struct gains forensic fields

`crates/mnemonic-toolkit/src/format.rs:156-161`:
```rust
pub struct VerifyCheck {
    pub name: String,
    /// "ok" | "fail" | "skipped"
    pub result: &'static str,
    pub detail: String,
    /// Forensic fields (SPEC §5.7); all None for "ok" / "skipped" checks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff_byte_offset: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decode_error: Option<String>,
}
```

`#[serde(skip_serializing_if = "Option::is_none")]` keeps the JSON envelope clean for "ok"/"skipped" checks (forensic fields omitted entirely; only present on "fail" checks).

### J.2 — `emit_verify_checks` helper (new)

`crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::emit_verify_checks` factors the 9 / 3+6N check-emission logic shared across template-mode and descriptor-mode. Signature:

```rust
pub fn emit_verify_checks(
    expected: &Bundle,
    supplied: &SuppliedCards,
    mode: BundleMode,
) -> Vec<VerifyCheck>
```

Where `SuppliedCards` is a new struct wrapping the user-supplied `--ms1`/`--mk1`/`--md1` argument vectors.

### J.3 — Refactor `run_full` / `run_multisig` / `descriptor_mode_verify_run` to call helper

Cross-phase invariant: emitted check ordering and names UNCHANGED for surviving checks. New checks added (e.g., `mk1_path_match[i]`) appear at the SPEC-mandated positions.

### J.4 — Schema-4 dispatch in verify-bundle JSON intake (REMOVED from v0.4.1 per r1 review I2)

Per r1 review I2: shipping the schema-4 dispatch infrastructure WITHOUT the corresponding `--bundle-json <file>` CLI flag would be untestable dead code violating the TDD-first discipline. v0.4.1 ships the verify-bundle CLI args migration (`--ms1` repeating per J.5) and the per-cell forensic fields (J.1) but DEFERS both the JSON-bundle intake mechanism AND its schema-version dispatch to v0.4.2.

New FOLLOWUP `bundle-json-cli-flag-and-dispatch` filed at tier `v0.4.2` for the atomic landing.

### J.5 — `--ms1` CLI repeating-flag migration

`crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::VerifyBundleArgs.ms1`:
- v0.4.0: `pub ms1: Option<String>`.
- v0.4.1: `pub ms1: Vec<String>` with `#[arg(long, action = clap::ArgAction::Append)]`.

All 4 existing integration tests (audit §8) continue to work — single `--ms1 <s>` invocation produces a 1-element vec. New schema-4 multi-source verification supplies repeating `--ms1` per slot.

### J.6 — Stderr warnings parity (closes audit FOLLOWUP L-9)

Descriptor-mode verify-bundle emits the same warnings as template-mode.

### J.7 — Per-cell forensic diagnostics integration

Mismatch identifies the failing field within a card per SPEC §5.7 forensic-field rules.

### J.8 — SPEC cross-check.

**Phase J architect review:** end-of-phase only.

## Release process (post-Phase J)

Final architect review across all phases (transcript). CHANGELOG v0.4.1 entry. Tag `mnemonic-toolkit-v0.4.1`. GitHub release.

`cargo publish` for the toolkit remains gated on ms-codec / mk-codec / md-codec landing on crates.io. v0.4.1 distributed via GitHub tag only.

## Test impact summary

- Bundle struct changes touch 6 unit tests in synthesize.rs.
- BundleJson schema-4 cutover updates 8 JSON assertions across 2 integration test files.
- `--ms1` clap migration: 4 existing tests continue to work via single-element vec; multi-source tests new in Phase H/J.
- Engraving card: removes 3 v0.2 byte-exact format.rs unit tests; adds new unified-card tests.
- VerifyCheck struct: ~70 push sites in verify_bundle.rs gain `expected: None, actual: None, diff_byte_offset: None, decode_error: None` defaults (or use `..Default::default()` after `Default` impl).
- Multi-source / hybrid synthesis: new unit + integration tests added in Phase H.
- Verify-bundle 9/3+6N descriptor parity: new test fixtures for descriptor-mode verify under schema 4.

## Estimated test counts post-v0.4.1

- v0.4.0: 227 lib + integration suites passing, 5 ignored.
- v0.4.1 target: ≥260 lib tests + new integration coverage for multi-source/hybrid + schema-4 round-trips. The 5 v0.2 self-multisig integration tests remain `#[ignore]`d (the multi-source path produces different fixtures, not a v0.2-fixture-equivalent).
