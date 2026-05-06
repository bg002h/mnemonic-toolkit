# mnemonic-toolkit v0.4.2 implementation plan — unified-path consolidation

**Cycle scope:** close all 7 v0.4.2 FOLLOWUPS + 1 promoted-from-v0.5 (legacy-flag-deprecation) per the user's "no users yet → ignore migration work" license. Theme: delete the dual-path baggage; finish the unified path; ship a single coherent CLI + library surface.

**Authoritative SPEC:** `design/SPEC_mnemonic_toolkit_v0_4.md` (cycle delta from v0.3 SPEC). v0.4.2 amends in-place via revision-history block.

**Discipline:**
- Per-phase architect review at end-of-phase; iterate to 0C/0I.
- Per-implementation-phase reports persist to `design/agent-reports/phase-<id>-<slug>-review-r<N>.md`.
- L/nit findings → `design/FOLLOWUPS.md` at `v0.4.3-nice-to-have`.
- TDD-first per phase where practical; rip-and-replace deletions skip the red phase.

## Locked decisions (user-confirmed defaults)

- **Q1 — legacy CLI deletion aggressiveness: (a) inputs preserved, dispatch unified.** `--phrase` / `--xpub` / `--cosigner` / `--cosigner-count` / `--cosigners-file` continue as accepted CLI flags, expanded into `Vec<SlotInput>` via the existing (and extended) `expand_legacy_to_slots`. The legacy parallel `bundle::run` dispatch (steps 2–6 in the v0.4.0 audit) is DELETED; `bundle::run` becomes a thin wrapper around `bundle_run_unified` after the trap + descriptor pre-check rows. Internal architecture is single-path; CLI is backward-compatible.
- **Q2 — 5 ignored v0.2 multisig-full integration tests: deleted.** They exercise a BIP-388-violating pattern with no migration path; their `#[ignore]` markers were placeholders documenting the v0.4.0 hard-reject. v0.4.2 deletes them outright.
- **SPEC versioning:** v0.4.2 amends `SPEC_mnemonic_toolkit_v0_4.md` inline with a revision-history "v0.4.2 amendments" block, NOT a new SPEC file.
- **Fixture regeneration:** none. v0.2 fixture files (`tests/vectors/v0_2/`) are text-mode-only; no JSON envelopes; pass byte-identically under schema-4.
- **`MultisigInfo.path_family: &'static str` — not enumified.** The v0.3 FOLLOWUP `multisiginfo-magic-strings-enumify` stays open; v0.4.2 delete-and-replace work doesn't compound the existing string-typing issue.

## Phase ordering rationale

Strict dependency chain — each phase consumes the prior phase's output:

```
Phase K (subkey shapes) ─┐
                         ├─→ Phase M (legacy deprecation: needs full slot vocab)
Phase L (descriptor)  ───┘                      │
                                                ▼
                                        Phase N (binding-type merge)
                                                │
                                                ▼
                                        Phase O (engraving migration)
                                                │
                                                ▼
                                        Phase P (verify-bundle helper + forensics)
                                                │
                                                ▼
                                        Phase Q (--bundle-json CLI + dispatch)
                                                │
                                                ▼
                                        Cleanup + Release
```

K and L are independent and could parallelize but I'll do K first; both must land before M.

## Phase K — additional slot subkey shapes

**Goal:** `resolve_slots` (cmd/bundle.rs) handles every subkey shape from SPEC §6.6.b validity matrix (closes FOLLOWUP `unified-slot-additional-subkey-shapes`).

### K.1 — `{entropy}` resolution

Hex-decode the entropy → `Mnemonic::from_entropy` → seed → derive at template path. Produces a ResolvedSlot with `entropy: Some(<bytes>)`. Tests: TREZOR_24's known 32-byte entropy `[0u8; 32]` round-trips byte-identically to `--slot @0.phrase=abandon...art`.

### K.2 — `{xprv}` resolution — DEFERRED to v0.5+ per r1 review C-1

Per r1 review C-1: xprv resolution introduces an incoherence between `SlotSubkey::is_secret_bearing` (returns true for Xprv) and `ResolvedSlot::is_secret_bearing` (returns `entropy.is_some()`). Resolving via "entropy: None + secret-bearing-by-subkey" creates two-truths semantics. Resolving via "entropy: Some(<xpriv bytes>) + ms-codec XPRV tag" requires ms-codec extension (not in scope for v0.4.2; cross-repo cycle).

**Locked decision:** v0.4.2 REJECTS `{xprv}` slots at `resolve_slots` with a BadInput error pointing at NEW FOLLOWUP `unified-slot-xprv-resolution-needs-ms-codec-extension` (v0.5+). `SlotSubkey::is_secret_bearing` for Xprv stays true (semantically correct for the underlying material) but `resolve_slots` short-circuits before that semantic matters. Tests: `unified_slot_xprv_rejected_with_followup_pointer`.

The single source of truth for ms1-emission is `ResolvedSlot::is_secret_bearing` (= `entropy.is_some()`); `SlotSubkey::is_secret_bearing` is only consulted for shape validation in `validate_slot_set`. Document the two methods' distinct roles in slot_input.rs doc-comments.

### K.3 — `{wif}` resolution

Parse WIF → degenerate single-key (no BIP-32 derivation). For multisig contexts, this is a "cold key" cosigner. Produces ResolvedSlot with synthetic `xpub` constructed from the WIF's pubkey (depth=0, fingerprint=0, no chain code → use [0u8; 32] chain code; BIP-32 framing accepts depth-0 with zero chain code). v0.4.2 minimum supports wif slots only in n=1 single-sig contexts; multisig with wif slots returns "wif-in-multisig deferred to v0.4.3" error pointing at FOLLOWUP `wif-multisig-resolution`.

### K.4 — Partial `{xpub}` shapes

`{xpub}` alone, `{xpub, fingerprint}`, `{xpub, path}` already pass `validate_slot_set`. `resolve_slots` already handles `{xpub, fingerprint, path}`; extend to default the missing fields (fingerprint defaults to `0x00000000`, path defaults to empty `m`).

### K.5 — Tests

Per-shape integration tests in `tests/cli_unified_slot.rs`:
- `unified_slot_entropy_singlesig_full_round_trips_against_phrase` (entropy emits byte-identical bundle to phrase form for the same seed).
- `unified_slot_xprv_singlesig_full` (xprv slot produces a valid bundle; ms1 is empty-string sentinel).
- `unified_slot_wif_singlesig_emits_valid_bundle` (degenerate single-key bundle).
- `unified_slot_xpub_alone_emits_partial_origin` (xpub-only watch-only slot with default fingerprint).
- `unified_slot_xpub_with_fingerprint_no_path` (partial origin).
- `unified_slot_wif_in_multisig_rejected_with_followup_pointer`.

**Phase K architect review:** end-of-phase only.

## Phase L — descriptor mode under unified --slot

**Goal:** `bundle_run_unified` accepts `--descriptor` / `--descriptor-file` alongside `--slot @N.<subkey>=<value>` (closes FOLLOWUP `unified-slot-descriptor-mode-support`). Per-`@N` slot binding from descriptor placeholders.

### L.1 — Descriptor + slot intake

Replace the v0.4.1 BadInput error in `bundle_run_unified` with: parse descriptor → lex placeholders → derive N from `max(@i)+1` → cross-check vs slot count → resolve each slot per K.1–K.4 → build descriptor-mode synthesis input.

### L.2 — Descriptor-mode synthesis through synthesize_unified

`synthesize_unified` already takes `template: CliTemplate` — extend signature to `template_or_descriptor: TemplateOrDescriptor` (matching the BundleInputForCard enum). Descriptor synthesis branches into the existing `synthesize_descriptor` path internally (now a private helper) but the entry point is unified.

**Alternative locked-out:** keep `synthesize_unified` template-only and route descriptor-mode through a parallel `synthesize_unified_descriptor`. Rejected because it perpetuates the dual-path problem v0.4.2 is meant to delete.

### L.3 — emit_unified descriptor-mode JSON envelope

When TemplateOrDescriptor::Descriptor, emit `template: null` + `descriptor: Some(...)`. MultisigInfo `template` field becomes `"descriptor"` literal.

### L.4 — Tests

- `unified_slot_descriptor_singlesig_phrase_full` (descriptor + --slot @0.phrase=).
- `unified_slot_descriptor_multisig_multisource` (3-cosigner descriptor + 3× --slot @N.phrase=).
- `unified_slot_descriptor_multi_leaf_taproot` (Phase F walker integrates).

**Phase L architect review:** end-of-phase only.

## Phase M — legacy flag deprecation (delete parallel dispatch)

**Goal:** delete the legacy `bundle::run` dispatch path (audit §6 steps 2–6). Closes promoted FOLLOWUP `legacy-flag-deprecation`. Single dispatch through `bundle_run_unified`.

### M.1 — Extend `expand_legacy_to_slots` for full legacy-flag vocabulary

Currently handles `--phrase` and `--cosigner-count`. Extend for:
- `--xpub X` → `--slot @0.xpub=X`.
- `--master-fingerprint X` → fold into the @0 slot when `--xpub` present (`--slot @0.fingerprint=X`).
- `--cosigner xpub:fp:path` (already-parsed CosignerSpec vec) → per-cosigner SlotInput triples.
- `--cosigners-file path` → parse + same expansion.

**Per r1 review C-2: cosigner offset rule LOCKED:**
- If `phrase.is_some()` (full multisig): cosigners occupy `@1..=N-1` (the `--phrase` slot is `@0`).
- If `phrase.is_none()` (pure watch-only multisig): cosigners occupy `@0..=N-1`.
- If both `phrase` AND `xpub` are supplied: error (cannot combine secret-bearing `@0` from phrase with watch-only `@0` from xpub).

The offset rule is fully deterministic from inputs; `expand_legacy_to_slots` makes the decision internally without "caller specifies" deferral. The legacy `--cosigner-count K` consistency check applies to `K == max(@i)+1` over the FINAL expanded vec (so `--phrase` + `--cosigner` × 2 + `--cosigner-count 3` is consistent: derived N = 3, K = 3, OK).

Full SPEC §6.6.a alias-mapping table now alive. Add to slot_input.rs unit tests: ≥6 expansion cases covering each legacy-flag combination.

### M.2 — Rewrite `bundle::run` as thin wrapper

Replace bundle::run body with:
1. Pre-clap trap (already in main.rs).
2. **SURVIVING pre-checks (per r1 review I-3 enumeration):** these v0.3 mode-violation pre-checks must remain in `bundle::run`'s thin wrapper because they fire on flag-combinations BEFORE slot expansion (and the existing `cli_mode_violations*.rs` tests pin their byte-exact stderr):
   - row 2: `--descriptor` + `--template` (descriptor-mode-and-template; `mode_text::DESCRIPTOR_AND_TEMPLATE`).
   - row 2.5: `--descriptor` + `--descriptor-file` mutual exclusion (`mode_text::DESCRIPTOR_AND_DESCRIPTOR_FILE`).
   - row 12 + 12.5/12.6/12.7: `--descriptor` with `--threshold` / `--cosigner-count` / `--multisig-path-family` / non-zero `--account` (`mode_text::DESCRIPTOR_WITH_*`).
   - row 6: `--xpub + --passphrase` (`mode_text::PASSPHRASE_WITH_XPUB`).
   - row 6.1: `--xpub + --language` (`mode_text::LANGUAGE_WITH_XPUB`).
   - row 6.2: `--xpub` requires `--master-fingerprint` (`mode_text::XPUB_NEEDS_FINGERPRINT`).
   - row 6.3: `--master-fingerprint` requires `--xpub` (`mode_text::FINGERPRINT_WITHOUT_XPUB`).
   - row 6.4: `--xpub + --cosigner` mutual exclusion (`mode_text::XPUB_AND_COSIGNER`).
   - row 6.5: `--cosigner + --cosigners-file` mutual exclusion (`mode_text::COSIGNER_AND_COSIGNERS_FILE`).
   - row 6.6: `--threshold` requires multisig template (`mode_text::THRESHOLD_WITHOUT_MULTISIG`).
   - row 6.7: `--cosigner-count` requires multisig template (`mode_text::COSIGNER_COUNT_WITHOUT_MULTISIG`).
   - row 6.8: `--multisig-path-family` requires multisig template (`mode_text::PATH_FAMILY_WITHOUT_MULTISIG`).
   - row 6.9: `--privacy-preserving + --xpub` (`mode_text::PRIVACY_WITH_XPUB`).
   - row 6.10: stdin-style `--xpub -` rejection (`mode_text::XPUB_STDIN`).
3. Compute slot vec via `expand_legacy_to_slots(args.slot, args.phrase, args.xpub, args.master_fingerprint, parsed_cosigners, args.cosigner_count)`.
4. Dispatch into `bundle_run_unified` regardless of whether `args.slot` is empty or not.

**Pre-checks NOT in surviving list** (absorbed into unified path's validate_slot_set or pre_check_threshold/template_n):
   - threshold range / template-N compatibility / contiguity (already in bundle_unified).
   - per-slot subkey-set validity matrix (already in slot_input::validate_slot_set).
   - BIP-388 distinctness (already in check_resolved_slots_distinctness).

**Delete:**
- `bundle_full` (synthesize.rs::synthesize_full kept; only the CLI-dispatch helper is removed).
- `bundle_watch_only` (CLI-dispatch helper).
- `bundle_multisig_full` (CLI-dispatch helper; the BIP-388 hard-reject moves into pre-check).
- `bundle_multisig_watch_only` (CLI-dispatch helper).
- `descriptor_mode_run` (replaced by unified path's L branch).
- `descriptor_mode_emit` (replaced by emit_unified).
- The stand-alone `emit` and `emit_multisig` text-mode renderers (replaced by emit_unified now handling all modes).

### M.3 — Synthesize.rs cleanup

- `synthesize_full` / `synthesize_watch_only` / `synthesize_multisig_full` / `synthesize_multisig_watch_only`: keep as primitives if `synthesize_unified` calls them; OR delete and have synthesize_unified do everything inline. **Decision: delete the four legacy functions; synthesize_unified handles all variants directly.**
- Remove the `bundle_multisig_full` BIP-388 hard-reject at the function entry (now caught upstream by `check_resolved_slots_distinctness`).

### M.4 — CLI integration test rewrite

Existing tests use `--phrase` / `--xpub` / `--cosigner` flags — they continue to work via `expand_legacy_to_slots`. Most tests should pass unchanged. Some assertions may need updates for:
- Engraving card text (Phase O migration changes byte-exact text).
- Stderr ordering (warnings now emitted in `bundle_run_unified`'s standard order).

**Phase M architect review:** mid-phase after M.2 (verify dispatch consolidation correct); end-of-phase after M.4.

## Phase N — CosignerKeyInfo → ResolvedSlot merge

**Goal:** retire `CosignerKeyInfo`; sole binding shape is `ResolvedSlot`. Closes FOLLOWUP `cosigner-keyinfo-resolved-slot-merge`.

### N.1 — Refactor parse_descriptor.rs::bind_descriptor_keys

`bind_descriptor_keys` returns `Vec<ResolvedSlot>` instead of `DescriptorBinding { cosigners: Vec<CosignerKeyInfo>, ... }`. Per-`@N` binding produces ResolvedSlot directly with entropy-bearing field set.

### N.2 — Update DescriptorBinding shape

`DescriptorBinding` retains `keys` + `fingerprints` (used by md-codec construction); `cosigners` becomes `Vec<ResolvedSlot>`. Or just delete `DescriptorBinding` and return tuple `(ResolvedPlaceholders, Vec<ResolvedSlot>)`.

### N.3 — Update verify_bundle.rs caller (per r1 review I-2)

Per r1 review I-2: `verify_bundle.rs::descriptor_mode_verify_run` calls `bind_descriptor_keys` (verify_bundle.rs:1331) and consumes `binding.cosigners: Vec<CosignerKeyInfo>`. Phase N MUST update this caller in lockstep with the bind_descriptor_keys signature change. The verify_bundle.rs consumer chain to update:
- Line 1331-1340: bind_descriptor_keys call site.
- Line 1349 + downstream: `synthesize_descriptor(&descriptor, &binding.cosigners, ...)` — switch to ResolvedSlot vec.
- Any other CosignerKeyInfo consumer in verify_bundle.rs (grep `binding.cosigners` + `cosigner.xpub` + `cosigner.fingerprint` + `cosigner.path`).

This is non-trivial scope (verify_bundle.rs is 1760 lines). Audit all sites + update mechanically.

### N.4 — Update check_key_vector_distinctness call sites

Already operates on `Vec<ResolvedSlot>` indirectly via the unified path; v0.4.1 had two parallel implementations (`check_key_vector_distinctness` for legacy + `check_resolved_slots_distinctness` for unified). v0.4.2 deletes one; keep `check_resolved_slots_distinctness` and rename to `check_key_vector_distinctness` (single name).

**Phase N architect review:** end-of-phase only.

## Phase O — engraving card legacy migration

**Goal:** delete the 4 legacy `engraving_card(...)` call sites (already gone in M.2 via emit_unified). Delete `EngravingMode` enum + old `engraving_card` function. Closes FOLLOWUP `engraving-card-unified-legacy-migration`.

### O.1 — Delete dead code

- `format.rs::engraving_card` function.
- `format.rs::EngravingMode` enum.
- 3 byte-exact format.rs unit tests for `EngravingMode::*` variants.

### O.2 — Update integration tests

Most integration tests use `predicate::str::contains(...)` against engraving card text — the new card layout has different field names (e.g., `# === Wallet bundle: ...` instead of `network:`). Audit + update each affected test. Per the v0.4.1 r1 review of Phase H, no integration tests use `contains` against old field names — but verify.

**Phase O architect review:** end-of-phase only.

## Phase P — verify-bundle helper + full forensics + descriptor 9/3+6N parity

**Goal:** introduce `emit_verify_checks` helper; refactor `run_full` / `run_multisig` / `descriptor_mode_verify_run` to share it; populate forensic fields at all ~78 push sites; descriptor-mode emits the same 9/3+6N schema as template-mode. Closes FOLLOWUPS `verify-bundle-emit-checks-helper-and-full-forensics-rollout` + `verify-bundle-9-3plus6n-descriptor-mode-parity`.

### P.1 — `emit_verify_checks(expected, supplied, is_multisig) -> Vec<VerifyCheck>` helper

In `cmd/verify_bundle.rs`. Takes the expected bundle + supplied cards + bool for multisig classification; emits the 9 / 3+6N check schema with shared logic for the per-cell forensic field population.

**Per r1 review I-3 — signature decision LOCKED:** signature uses `is_multisig: bool` (NOT `BundleMode`). Per-slot watch-only inference comes from the bundle's own ms1 sentinels (`expected.ms1[i].is_empty()` ↔ slot @i is watch-only). This avoids needing `verify_bundle.rs` to introduce BundleMode classification at its dispatch level (which would be a Phase-Q-equivalent surgery on the verify-bundle entry points). The bundle-data-driven inference is sufficient for the 9/3+6N schema's per-slot ms1 check skip ("skipped: watch-only slot" when `expected.ms1[i].is_empty()`).

### P.2 — Refactor 3 run_* entry points

`run_full`, `run_multisig`, `descriptor_mode_verify_run` all call `emit_verify_checks` instead of inline check generation. ~78 push sites collapse into ~10 helper-driven sites.

### P.3 — Descriptor-mode 9/3+6N

Drop the v0.3 3-element coarse ladder in `descriptor_mode_verify_run`; use emit_verify_checks. The existing shim from v0.4.1 H.1 Phase J (single-line ms1 lookup) is fully replaced.

### P.4 — Forensic field population

Every `result == "fail"` check populates `expected` / `actual` / `diff_byte_offset` (string-mismatch) or `decode_error` (decode-failure). emit_verify_checks helper centralizes the population.

### P.5 — Tests

- 3+6N count assertion for descriptor-mode multisig (`cli_descriptor_mode.rs`).
- Tampered-mk1 detection in descriptor mode emits forensic fields per SPEC §5.7 (`cli_descriptor_mode.rs`).
- Watch-only slot ms1 check is "skipped" with `decode_error: "skipped: watch-only slot"` (per SPEC §5.7).

**Phase P architect review:** mid-phase after P.2 (helper API shape) + end-of-phase after P.5.

## Phase Q — `--bundle-json` CLI + schema-version dispatch

**Goal:** `mnemonic verify-bundle --bundle-json <file>` reads a JSON-envelope bundle (output of `bundle --json`) + dispatches on `schema_version` for schema 2 / 3 / 4 intake. Closes FOLLOWUP `bundle-json-cli-flag-and-dispatch`.

### Q.1 — `--bundle-json <path>` CLI flag

`VerifyBundleArgs` gains `pub bundle_json: Option<PathBuf>`. Mutually exclusive with the explicit `--ms1` / `--mk1` / `--md1` flag triplet (clap conflicts_with).

### Q.2 — `serde_json::Value` peek + schema-4 typed dispatch (per r1 review N-1)

Per r1 review N-1: schema-2/3 retro-compat intake is speculative (no real-world schema-2/3 bundles exist; no users yet). v0.4.2 ships schema-4-only intake; schema-2/3 intake routes to a NEW v0.4.3 FOLLOWUP `bundle-json-schema-2-3-retro-compat` if a real need surfaces.

Read file → `serde_json::from_str::<Value>` → inspect `["schema_version"]` → branch:
- `"4"`: deserialize as the v0.4.1 BundleJson (with MsField).
- `"2"` / `"3"` / other: error with `"--bundle-json schema_version {got} not supported in v0.4.2; this toolkit emits and reads schema_version \"4\" only. Schema-2/3 retro-compat intake tracked at FOLLOWUP \`bundle-json-schema-2-3-retro-compat\`."`.

### Q.3 — Tests

- `verify_bundle_via_bundle_json_schema_4_round_trip` (run `bundle --json` → write JSON to tmp file → `verify-bundle --bundle-json tmp` → exit 0).
- `verify_bundle_via_bundle_json_unsupported_schema_rejected` (schema "3" or "99" → exit 4 with the byte-exact error pointing at the v0.4.3 retro-compat FOLLOWUP).

**Phase Q architect review:** end-of-phase only.

## Cleanup — delete 5 ignored v0.2 multisig-full integration tests

`cli_account_flag.rs`, `cli_privacy_preserving.rs`, `cli_bundle_multisig_full.rs`, `cli_self_check.rs::bundle_self_check_passes_for_canonical_seed_multisig`, `cli_bundle_multisig.rs::self_multisig_full_emits_warning_and_n_card_sets`. Per Q2 user-confirmed default. Files containing only-the-deleted-test get removed entirely; files containing other live tests have only the ignored function deleted.

## Release (post-Phase Q + cleanup)

Final architect review across all phases (transcript-only). CHANGELOG v0.4.2 entry. Tag `mnemonic-toolkit-v0.4.2`. GitHub release.

`cargo publish` for the toolkit remains gated on ms-codec / mk-codec / md-codec landing on crates.io. v0.4.2 distributed via GitHub tag only.

## Test impact summary

- **Phase K:** +6 new CLI integration tests for additional subkey shapes.
- **Phase L:** +3 new descriptor-mode tests under unified --slot.
- **Phase M:** existing ~25 integration tests should pass unchanged (legacy flags expand to slots transparently); engraving-card-text assertions may need updates in Phase O.
- **Phase N:** binding refactor; all callers update mechanically.
- **Phase O:** -3 EngravingMode unit tests deleted; +3 new ones for migrated call sites.
- **Phase P:** +3 descriptor-mode 9/3+6N + forensic tests.
- **Phase Q:** +3 --bundle-json tests.
- **Cleanup:** -5 deprecated integration tests deleted.

Estimated post-v0.4.2 test count: ~250 lib + ~40 integration (was 246 + integration suites).

## Out of scope (deferred to v0.4.3+ or v0.5+)

- `wif-multisig-resolution` (NEW v0.4.3 FOLLOWUP — multisig contexts with wif slots; v0.4.2 single-sig only).
- `unified-slot-xprv-resolution-needs-ms-codec-extension` (NEW v0.5+ FOLLOWUP — `{xprv}` slot resolution rejected in v0.4.2 per r1 review C-1; gated on ms-codec XPRV-tag support).
- `bundle-json-schema-2-3-retro-compat` (NEW v0.4.3 FOLLOWUP — `--bundle-json` schema 2/3 intake; gated on a real need surfacing).
- `multisiginfo-magic-strings-enumify` (existing v0.3-nice-to-have; not addressed in v0.4.2).
- `descriptor-string-normalization-policy` (existing v0.3-nice-to-have; not addressed).
