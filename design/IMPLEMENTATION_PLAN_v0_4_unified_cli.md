# mnemonic-toolkit v0.4 implementation plan — unified `@N`-pattern CLI

**Cycle scope:** unified `bundle` command with `@N`-pattern slot inputs; BIP-388 distinct-key conformance enforced symmetrically; multi-source full multisig (N seeds → N (ms1, mk1) pairs); multi-leaf taproot; verify-bundle 9 / 3+6N descriptor-mode parity; schema 4 dispatch.

**Companion artifacts:**
- SPEC: [SPEC_mnemonic_toolkit_v0_4.md](SPEC_mnemonic_toolkit_v0_4.md)
- Cycle plan (audit + brainstorm record): `/home/bcg/.claude/plans/read-the-memory-you-snoopy-pearl.md`
- v0.3 SPEC (carry-forward base): [SPEC_mnemonic_toolkit_v0_3.md](SPEC_mnemonic_toolkit_v0_3.md)

**Discipline:**
- TDD-first: tests written before impl per phase (per `feedback_iterative_review_every_phase`).
- Per-phase architect review: mid-phase + end-of-phase rounds for non-trivial phases (B, C, D, G); end-of-phase only for smaller phases (A, E, F).
- Iterate to 0C/0I per round; max r4 then escalate.
- Low / nit findings route to `design/FOLLOWUPS.md` at appropriate tier.
- Per-implementation-phase reports persist to `design/agent-reports/phase-<X>-<slug>-review-r<N>.md`.
- All v0.4-tier follow-ups close in-cycle per the cycle plan exit criterion 10 (except L-10, L-11 deferred to v0.5+ as new follow-ups).

## Phase 2 SPIKE (mandatory, pre-Phase A)

Two mandatory SPIKE items. Skip ONLY if SPIKE conclusively resolves both with no architect findings.

**SPIKE-1:** Multi-leaf tap encoding round-trip via md-codec `Tag::TapTree`. Throwaway crate exercising:
- 2-leaf and 3-leaf TapTree shapes
- md-codec encode/decode round-trip preserving tree topology
- Walker descent into branches via existing `walk_miniscript_node`

**SPIKE-2:** Clap impl-1 `--slot @N.<subkey>=<value>` value-parser + bundle-multisig-full removed-subcommand error path. Throwaway crate exercising:
- `parse_slot_input(s: &str) -> Result<SlotInput, ParseError>` value-parser shape
- Clap `Arg::new("slot").long("slot").action(ArgAction::Append).value_parser(parse_slot_input)` shape
- Edge cases: `@<bad-index>`, `@N.<unknown-subkey>`, `@N.subkey=<empty>`, `<no-@>`, `<no-equals>`
- Removed-subcommand trap: `mnemonic bundle multisig-full <flags>` → byte-exact §6.6 row 1 error (clap interprets `multisig-full` as positional arg by default; SPIKE locks the trap mechanism, e.g., custom positional handler or pre-clap argv inspection)

**SPIKE deliverable:** `design/agent-reports/spike-toolkit-v0_4-pre-phaseA.md` with confirmed API shapes for `walk_tap_tree` (md-codec API surface) and `parse_slot_input` (clap value-parser surface). Architect review iterates to 0C/0I before Phase A starts.

## Phase A: Audit-fixups + BIP-388 conformance

**Goal:** delete dead/legacy code; introduce hard-reject conformance check; enumerate fixture exclusions; clean up audit-flagged comment/annotation rot.

**Tasks:**

**A.1** Delete `SELF_MULTISIG_WARNING` (both consts at `cmd/bundle.rs:639-644` and `parse_descriptor.rs:1054-1055`); delete `check_self_multisig_warning` function at `parse_descriptor.rs:1041-1052`. Tests: existing tests at `parse_descriptor.rs:1641` and `:1676` referencing the function are deleted; no replacement (the conformance check is added in A.2).

**A.2** Add `check_key_vector_distinctness(&binding: &DescriptorBinding) -> Result<(), ToolkitError>`. Pairwise scan over `binding.cosigners`; collision detected by `(xpub, derivation_path_string)` tuple equality across all slot pairs per SPEC §4.11.b normalization domain. Returns `Err(ToolkitError::Bip388Distinctness { i: u8, j: u8 })`. Tests: ≥6 cases per SPEC §4.11.b's tuple-equality definition:
- All-distinct N=3 passes
- `@0` xpub == `@1` xpub AND `@0` path == `@1` path → collision (exit 2)
- `@0` xpub == `@2` xpub AND `@1` xpub == `@3` xpub (two collisions; first-detected reported)
- **`@0` xpub == `@1` xpub BUT paths differ → ACCEPTED** (different tuples per §4.11.b; BIP-388-letter interpretation locked)
- `@0` xpub differs from `@1` xpub BUT paths identical → ACCEPTED (different tuples)
- `@0` and `@1` both with empty path strings AND identical xpubs → collision (per §4.11.b normalization: absent paths treated as `""` for collision)
- Degenerate N=1 passes (no pairs to compare)

**A.3** Wire `check_key_vector_distinctness` into both `bundle_run` (post-binding, pre-synthesis) and `verify_bundle::run_*` (post-binding, pre-comparison). Tests: bundle-time rejection emits SPEC §6.6 row 13 stderr text; verify-bundle-time rejection emits exit 4 + `error: bundle violates BIP-388 distinct-key rule; regenerate with distinct keys`.

**A.4** Audit-flagged cleanups:
- L-3: simplify `parse_descriptor.rs:725-727` stale `ctx_for_descriptor` comment to one-liner.
- L-4: remove `#[allow(dead_code)]` at `synthesize.rs:171-176`.
- L-7: drop `let _ = SELF_MULTISIG_WARNING;` and the `use crate::cmd::bundle::SELF_MULTISIG_WARNING;` import from `verify_bundle.rs:1284,1342-1343` (now dead).

**A.5** v0.2 fixture corpus enumeration: grep `tests/vectors/v0_2/` for cells produced by `multisig-full` invocations; produce SPEC §10 exclusion list. v0.3 corpus audit: grep for descriptor cells with potential BIP-388 violations; produce SPEC §10 v0.3 exclusion list. Tests: existing v0.2 multisig-full integration tests are now expected to FAIL with row-13 error; mark `#[ignore]` with comment "deprecated v0.2 pattern; remove after v0.4 release".

**A.6** Verify SPEC §4.11 in-repo file matches the implementation. (§4.11 is already fully drafted in `design/SPEC_mnemonic_toolkit_v0_4.md`; A.6 is a cross-check / faithfulness verification, not new content authoring. Run `git diff` against the SPEC file after A.1–A.5 land; lock the SPEC text if the impl drifted from the SPEC.)

**Phase A architect review checkpoints:** mid-phase after A.3 (verify wire-up); end-of-phase after A.6.

## Phase B: `@N`-pattern CLI + slot input parsing

**Goal:** introduce `--slot @N.<subkey>=<value>` flag with custom value parser; deprecation aliases for v0.2 flags; per-slot subkey-set validation.

**Tasks:**

**B.1** New module `crates/mnemonic-toolkit/src/slot_input.rs`. Define `SlotInput { index: u8, subkey: SlotSubkey, value: SlotValue }`; `SlotSubkey` enum (Phrase, Entropy, Xpub, Fingerprint, Path, Wif, Xprv). `SlotValue` is the parsed-and-typed value (each subkey has its own type validation).

**B.2** Custom clap value parser: `parse_slot_input(s: &str) -> Result<SlotInput, ParseError>`. Accepts `@N.<subkey>=<value>` strings; returns typed SlotInput. **Consumes SPIKE-2 confirmed API surface** — read `design/agent-reports/spike-toolkit-v0_4-pre-phaseA.md` for the locked function signature and error type shape before writing tests. Tests: ≥10 parse cases (each subkey's happy path; malformed `@`; invalid index; unknown subkey; missing `=`; empty value).

**B.3** Wire `--slot` repeating flag into clap: `Arg::new("slot").long("slot").action(ArgAction::Append).value_parser(parse_slot_input)`. Replaces the v0.2 multi-flag CLI surface inside `cli.rs`.

**B.4** Per-slot subkey-set validator: `validate_slot_set(slots: &[SlotInput]) -> Result<(), ToolkitError>` returning errors per SPEC §6.6 row 4 (conflict cases) and row 8 (gap detection). Tests: ≥10 validation cases (legal sets per the validity matrix; each conflict case from §6.6.b; gap detection at `@2` without `@1`).

**B.5** Deprecation aliases per SPEC §6.6.a alias-mapping table: when `--phrase` is supplied without any `--slot`, expand to `--slot @0.phrase=value` virtually (post-clap normalization). Same for `--cosigner` (both bare and annotated forms). Same for `--cosigner-file`. Same conflict-detection per rows 5/6/7. Tests: each alias row exercised with byte-exact error text on conflicts.

**B.6** Verify SPEC §6.6.a alias-mapping section matches the implementation. (§6.6.a is already fully drafted in `design/SPEC_mnemonic_toolkit_v0_4.md`; B.6 is cross-check, not new content. Confirm impl alias handling matches the table verbatim; lock the SPEC if drift is found.)

**Phase B architect review checkpoints:** mid-phase after B.4 (parser + validator); end-of-phase after B.6.

## Phase C: Single `bundle` command + auto-detection

**Goal:** remove `multisig-full` / `multisig-watch-only` subcommands; unified `bundle` command; mode auto-detection from per-slot subkeys.

**Tasks:**

**C.1** Remove `multisig-full` and `multisig-watch-only` Subcommand variants from `cli.rs`. Custom positional-argument trap (per SPIKE-2 confirmed mechanism) emits SPEC §6.6 row 1 error when these tokens appear under `bundle`.

**C.2** Refactor `cmd/bundle.rs::bundle_run` to a unified entry point: takes `slots: Vec<SlotInput>` + optional `--template` + optional `--descriptor` + threshold; auto-detects mode from per-slot subkeys.

**C.3** Mode dispatch:
```rust
enum BundleMode {
    SingleSigFull,        // 1 slot, secret-bearing
    SingleSigWatchOnly,   // 1 slot, xpub-only
    MultisigMultiSource,  // N>1 slots, all secret-bearing
    MultisigWatchOnly,    // N>1 slots, all xpub-only
    MultisigHybrid,       // N>1 slots, mixed
}
```

**Descriptor mode dispatch:** `--descriptor` presence does NOT introduce a separate `BundleMode::DescriptorMode` variant. Instead, descriptor mode is orthogonal: `bundle_run` first parses the descriptor (if present) into a `Descriptor` AST + binding sources, then computes `BundleMode` from the same per-slot rules above (single vs multi; secret-bearing vs xpub-only). The combination `(BundleMode, Option<Descriptor>)` drives downstream phases:

- Card rendering (Phase E): `BundleInputForCard` carries either a template name OR a descriptor string + the same per-slot blocks regardless of which.
- Synthesis (Phase D): `synthesize_*` functions take `(BundleMode, Option<&Descriptor>)`; descriptor presence drives the descriptor-aware code path; mode drives single-source-vs-multi-source synthesis.
- Verification (Phase G): `emit_verify_checks(expected: &Bundle, supplied: &SuppliedCards, mode: BundleMode)` — descriptor presence is implicit via `expected: &Bundle`'s template_or_descriptor field.

**C.4** Pre-check ladder per SPEC §6.6 rows 1-12: `pre_check_mode(args, slots)` returning Result. Tests: each row exercised with byte-exact stderr.

**C.5** Verify SPEC §6.6 row table matches the implementation. (§6.6 is already fully drafted in `design/SPEC_mnemonic_toolkit_v0_4.md` with all 14 rows + sub-tables §6.6.a, §6.6.b; C.5 is cross-check, not new content. Confirm impl error texts match the SPEC verbatim; lock the SPEC if drift is found.)

**Phase C architect review checkpoints:** mid-phase after C.3 (dispatch shapes); end-of-phase after C.5.

## Phase D: Multi-source secrets + MsField + schema 4 + multisig reshape

**Goal:** implement multi-source bundle synthesis; `BundleJson` migration; schema 4 dispatch.

**Tasks:**

**D.1** `format.rs::BundleJson`: change `ms1: Option<String>` → `ms1: MsField` where `MsField = Vec<String>` (per SPEC §5.8). Update `schema_version: &'static str = "4"`. v0.2/v0.3 verify-bundle paths read schema_version FIRST and pick the correct shape.

**D.2** `synthesize.rs::synthesize_multisig_multisource`: new function for `BundleMode::MultisigMultiSource`. Iterates over slots; for each secret-bearing slot, derives (ms1, mk1) pair; aggregates into `Vec<String>`; combines per template into single md1.

**D.3** `synthesize.rs::synthesize_multisig_hybrid`: hybrid mode. Some slots produce ms1+mk1 (secret-bearing); others produce mk1 only (watch-only). Per the locked SPEC §5.8 rule: `MsField` is **dense-with-empty-string-placeholders, length-N invariant** — `len(ms1) == N`; `ms1[i] == "<ms1-string>"` for secret-bearing slot @i; `ms1[i] == ""` (empty-string sentinel) for watch-only slot @i. Tests assert the length invariant and the sentinel positions per slot subkey set.

**D.4** verify-bundle schema-4 dispatch: new `verify_bundle::run_schema_4` handler. Reads `MsField` array; for each non-empty element, runs ms1_decode + ms1_entropy_match check. Tests: schema-4 multi-source verification round-trip; per-cell forensic diagnostic on mismatch.

**D.5** Verify SPEC §5.8 (`MsField`) and §5.6 (schema-4 carry rules) match implementation. Both are already drafted in the in-repo SPEC; D.5 is cross-check.

**Phase D architect review checkpoints:** mid-phase after D.3 (synthesis shapes); end-of-phase after D.5.

## Phase E: Engraving card under unified bundle (1 master)

**Goal:** unified `BundleInputForCard` shape; deprecate per-mode `EngravingMode` variants; render 1 master card per bundle.

**Tasks:**

**E.1** New `format.rs::BundleInputForCard` struct: shared header (network, template_or_descriptor, threshold, N, language, passphrase_used) + per-slot blocks (slot index, ms1+mk1 chunk_set_ids if present, fingerprint, origin path, xpub).

**E.2** Unified `engraving_card_unified(input: &BundleInputForCard) -> String` function. Renders header + per-slot block list + descriptor (truncated if > 80 chars per SPEC §5.5; full descriptor lives in md1; chunk_set_id reference on card).

**E.3** Wire from `bundle_run` (template mode) and `descriptor_mode_run` (descriptor mode) to `engraving_card_unified`. `--no-engraving-card` flag plumbed across both paths (closes audit follow-up L-8).

**E.4** Deprecate old `EngravingMode::*` variants. Tests: byte-identical regression for v0.2 single-sig full + multisig-watch-only card text; new tests for hybrid mode + multi-source + descriptor mode card content.

**E.5** Verify SPEC §5.5 (card layout) matches implementation. §5.5 is already drafted in the in-repo SPEC; E.5 is cross-check.

**Phase E architect review checkpoints:** mid-phase after E.2 (review `BundleInputForCard` struct + `engraving_card_unified` render-function API shape before E.3 wires them in); end-of-phase after E.5. Mid-phase rationale: E.1 + E.2 introduce a new data struct + render function with no SPIKE coverage (SPIKE-1 and SPIKE-2 do not exercise the engraving card surface); locking the API shape before E.3 wiring prevents API errors from propagating into bundle/descriptor call sites.

## Phase F: Multi-leaf taproot walker

**Goal:** generalize `walk_tap_tree_singleleaf` → `walk_tap_tree`; descend `Tag::TapTree` branches.

**Tasks:**

**F.1** New `parse_descriptor.rs::walk_tap_tree(tree: &TapTree) -> Result<Layer1Node>`. Recurses into branch nodes (each branch has 2 children); leaves call into existing `walk_miniscript_node`. No 0-leaf guard (per SPEC §4.9.a invariant). Consumes the SPIKE-1-confirmed md-codec `Tag::TapTree` API shape (the SPIKE report at `design/agent-reports/spike-toolkit-v0_4-pre-phaseA.md` is the authoritative reference).

**F.2** Replace call site `walk_tap_tree_singleleaf` → `walk_tap_tree`. Delete the singleleaf function (no longer needed).

**F.3** Round-trip tests: `tr(K, {pk(@1), pk(@2)})` (2-leaf), `tr(K, {pk(@1), {pk(@2), pk(@3)}})` (3-leaf, asymmetric tree), `tr(K, {{pk(@1), pk(@2)}, {pk(@3), pk(@4)}})` (4-leaf, balanced tree). Each round-trips via md-codec's `Tag::TapTree` encoding.

**F.4** Opportunistic cleanups (audit follow-ups L-1, L-2): walk_wsh isolation tests; SortedMulti byte-equiv matrix — fold during walker work since file is already being touched.

**F.5** Verify SPEC §4.9.a (Tr-multileaf) matches implementation. §4.9.a is already drafted in the in-repo SPEC; F.5 is cross-check.

**Phase F architect review checkpoint:** end-of-phase only.

## Phase G: Verify-bundle 9 / 3+6N ladder + emit_verify_checks helper + schema-4 dispatch

**Goal:** factor shared check helper; descriptor-mode 9 / 3+6N parity; schema-4 dispatch; per-cell forensic diagnostics.

**Tasks:**

**G.1** New `verify_bundle::emit_verify_checks(expected: &Bundle, supplied: &SuppliedCards, mode: ChecksMode) -> Vec<VerifyCheck>` helper. Templates the 9 / 3+6N check schema with mode-specific expected sources.

**G.2** Refactor `run_full` / `run_watch_only` / `run_multisig` to call the helper instead of inline check generation. Cross-phase invariant: emitted check ordering and names UNCHANGED.

**G.3** Refactor `descriptor_mode_verify_run`: replace 3-element coarse ladder with `emit_verify_checks` call. Descriptor-mode now emits the same schema as template-mode.

**G.4** Per-cell forensic diagnostics. **`VerifyCheck` struct full v0.4 definition:**
```rust
pub struct VerifyCheck {
    /// e.g., "ms1_decode[0]", "mk1_xpub_match[1]", "md1_wallet_policy"
    pub name: String,
    /// true = check passed; false = failed
    pub passed: bool,
    /// Expected encoded string (for string-mismatch checks); null otherwise
    pub expected: Option<String>,
    /// Actual encoded string (for string-mismatch checks); null otherwise
    pub actual: Option<String>,
    /// First UTF-8 byte position where expected and actual differ; null if either side is null
    pub diff_byte_offset: Option<usize>,
    /// Decode-error message text for decode-failure checks; null otherwise
    pub decode_error: Option<String>,
}
```
v0.3 shipped `VerifyCheck { name: String, passed: bool }`; v0.4 adds the four forensic fields. `expected` / `actual` / `diff_byte_offset` populated per SPEC §5.7 forensic-field rules; `decode_error` populated for decode-failure checks. All four new fields default to `None` for `passed: true` checks.

**G.5** Schema-4 dispatch in `verify_bundle::run_*`: read `schema_version` from BundleJson; route to schema-2 / schema-3 / schema-4 handler. Schema-4 handler iterates the `MsField` array.

**G.6** Stderr warnings parity: descriptor-mode emits the same warnings as template-mode (closes audit follow-up L-9).

**G.7** Fixture additions per Phase D: divergent-path multisig fixture (closes audit follow-up L-5); `--privacy-preserving` descriptor-mode fixture (closes L-6).

**G.8** Verify SPEC §5.7 (verify-bundle conditional guarantee) matches implementation. §5.7 is already fully drafted in the in-repo SPEC file; G.8 is cross-check, not new authoring. (Note: prior drafts referenced §5.4; v0.4 SPEC does not introduce §5.4 deltas — the 9 / 3+6N check schema lives entirely in §5.7.)

**Phase G architect review checkpoints:** mid-phase after G.4 (forensic shape); end-of-phase after G.8.

## Release (post-Phase-G)

Final architect review across all phases (transcript-only). CHANGELOG with prominent breaking-changes section per SPEC §9. ≥40 v0.4 fixtures + selected v0.2/v0.3 carries (excluding v0.2 multisig-full and any v0.3 cells that now violate BIP-388 distinctness). v0.4 SHA pin computed and recorded. Tag `mnemonic-toolkit-v0.4.0` (gated on user approval per `feedback_iterative_review_every_phase`). GitHub release.

`cargo publish` for the toolkit remains gated on ms-codec / mk-codec / md-codec landing on crates.io. v0.4.0 distributed via GitHub tag only.
