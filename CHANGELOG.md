# Changelog

All notable changes to `mnemonic-toolkit` are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project follows [SemVer](https://semver.org/spec/v2.0.0.html) with the pre-1.0 convention that the second component (`0.X`) is the breaking-change axis.

## mnemonic-toolkit [0.4.0] — 2026-05-05

### What's new (v0.4.0 foundation release)

v0.4.0 is the foundation release for the v0.4 cycle. It ships:

- **BIP-388 distinct-key conformance (SPEC §4.11).** The toolkit now hard-rejects any descriptor binding whose `@N` slots resolve to identical `(xpub, derivation_path)` tuples. Symmetric across bundle creation (exit 2 + SPEC §6.6 row 13 byte-exact stderr) and verify-bundle (exit 4 + SPEC §4.11.c stderr). The legacy `bundle multisig-full --cosigner-count > 1` self-multisig path now hard-rejects at the entry point — all v0.2 self-multisig fixtures are excluded from the byte-identical regression matrix per SPEC §10 and the affected integration tests are marked `#[ignore = "deprecated v0.2 pattern; remove after v0.4 release"]`.
- **`--slot @N.<subkey>=<value>` CLI surface (SPEC §6.6.b).** New repeating clap flag with closed subkey vocabulary `phrase | entropy | xpub | fingerprint | path | wif | xprv`. Includes `parse_slot_input` value-parser (SPIKE-2 locked grammar; empty value rejected at parser), `validate_slot_set` (per-slot validity matrix + contiguity check), and `expand_legacy_to_slots` for SPEC §6.6.a deprecation alias mapping.
- **`bundle multisig-full` / `bundle multisig-watch-only` removed-subcommand trap (SPEC §6.6 row 1).** Pre-clap argv inspection emits the byte-exact migration error before clap parses. Two CLI integration tests assert byte-exact stderr from a live binary.
- **`BundleMode` mode-detection foundation (impl plan Phase C.3).** `detect_bundle_mode(slots)` classifier + `pre_check_threshold` / `pre_check_template_n` helpers (SPEC §6.6 rows 9, 9.5, 10, 11). Wired in v0.4.1 follow-on per `bundle-json-schema-4-cutover`.
- **`MsField = Vec<String>` type alias (SPEC §5.8).** Foundation for the schema-4 ms1 dense layout. Live wire-up deferred to v0.4.1.
- **Multi-leaf taproot walker (SPEC §4.9.a).** `walk_tap_tree` generalizes v0.3's single-leaf-only walker via depth-stack folding of miniscript's flat DFS-preorder leaf list. Algorithm transcribed verbatim from Phase 2 SPIKE-1 deliverable. Validated against 6 round-trip probe shapes (1/2/3/4-leaf incl. asymmetric and right-spine) at SPIKE time and 4 in-tree unit tests.

### Out of scope (deferred to v0.4.1)

The following SPEC §9 v0.4 deliverables are deferred to a v0.4.1 follow-on patch to keep the v0.4.0 release scope-safe under autonomous execution. See `design/FOLLOWUPS.md` entries at tier `v0.4.1`:

- **`bundle-json-schema-4-cutover`** — full `BundleJson.ms1: Option<String>` → `MsField` migration + `schema_version: "3" → "4"` bump + verify-bundle schema-4 dispatch + integration test JSON assertion updates + fixture envelope regeneration. v0.4.0 retains the schema-3 envelope; multi-source synthesis primitives sit ready in `format.rs` + `bundle_unified.rs` for v0.4.1 wire-up.
- **`engraving-card-unified-1-master-card`** — Phase E unified `BundleInputForCard` + `engraving_card_unified` per SPEC §5.5. Tightly coupled to schema-4 cutover.
- **`verify-bundle-9-3plus6n-forensics`** — Phase G descriptor-mode parity to template-mode 9 / 3+6N check ladder + per-cell forensic `VerifyCheck` fields per SPEC §5.7.

### Breaking changes

- **`bundle multisig-full --cosigner-count > 1`** hard-rejects (exit 2 + SPEC §6.6 row 13 stderr) per BIP-388 distinct-key rule. The legacy v0.2 self-multisig pattern is no longer producible. Migration: use `--cosigner` triples for watch-only multisig (still works), or wait for v0.4.1's multi-source synthesis (N distinct seeds → N (ms1, mk1) pairs).

### Wire-bit-identical guarantee

v0.2 single-sig + multisig-watch-only fixtures continue to pass byte-identically. v0.2 self-multisig fixtures (33 cells under `wsh-multi`/`sortedmulti`, `sh-wsh-multi`/`sortedmulti`, `tr-multi-a`/`sortedmulti-a` × 4 networks; plus 0/5/0-true variants of `wsh-sortedmulti`) are EXCLUDED from the byte-identical regression matrix per BIP-388 violation. v0.3 fixtures continue to pass byte-identically.

### Test corpus

227 lib unit tests + integration test suites pass; 5 v0.2 multisig-full integration tests are `#[ignore]`d per SPEC §10 fixture exclusions. Tests added in v0.4.0:
- 7 BIP-388 distinct-key unit tests (`parse_descriptor::tests::bip388_*`).
- 1 BIP-388 byte-exact CLI stderr integration test (`cli_bip388_distinctness`).
- 34 slot-input parser/validator/alias-expander unit tests (`slot_input::tests`).
- 24 bundle_unified mode-detection + pre-check + trap unit tests.
- 2 removed-subcommand trap CLI integration tests.
- 4 multi-leaf taproot walker unit tests.

### Cycle artifacts

- SPEC: `design/SPEC_mnemonic_toolkit_v0_4.md` (309 lines; delta over v0.3 SPEC).
- Implementation plan: `design/IMPLEMENTATION_PLAN_v0_4_unified_cli.md` (217 lines; 7 phases A-G + pre-Phase-A SPIKE).
- SPIKE deliverable: `design/agent-reports/spike-toolkit-v0_4-pre-phaseA.md` (architect-cleared at r2 0C/0I).
- Phase reviews: `design/agent-reports/phase-A-bip388-conformance-review-r1.md` (APPROVE WITH NITS), `phase-B-slot-input-review-r1.md` (APPROVE), `phase-C-bundle-unified-review-r1.md` (APPROVE WITH NITS).

### Architect-review history

- Brainstorm convergence: 6 plan-mode rounds (r1 0C/1I/4L → r6 0C/0I/2L APPROVE).
- SPEC + implementation plan: 2 rounds in-toolkit-repo (r2 APPROVE).
- Phase 2 SPIKE: 2 rounds (r2 0C/0I).
- Per-phase: A r1 APPROVE WITH NITS (1L+2N), B r1 APPROVE (0L+2N — L-1 fixed inline + 1 fix during r1 round), C r1 APPROVE WITH NITS (1L+3N), F skipped review (algorithm SPIKE-cleared, in-tree tests are direct SPIKE transcription), D/E/G — explicit deferral to v0.4.1 documented in FOLLOWUPS.

---

## mnemonic-toolkit [0.3.1] — 2026-05-05

### What's new

- `tr(K, sortedmulti_a(...))` in tap-leaves now parses and emits valid bundles. Closes the v0.3.0 deferral — rust-miniscript v13.0.0 had no parser for `sortedmulti_a`, but PR #910 ("Add support for sortedmulti_a") merged 2026-04-03 added it, and PR #915 (2026-04-04) refactored `SortedMultiVec` away.

### Mechanism

- Workspace `Cargo.toml` adds `[patch.crates-io] miniscript = { git = "https://github.com/rust-bitcoin/rust-miniscript", rev = "95fdd1c5773bd918c574d2225787973f63e16a66" }` — pinned to rust-miniscript master post-#910 + #915.
- `parse_descriptor.rs` walker refactored for the post-#915 API: `WshInner` enum removed (Wsh wraps Miniscript directly via `as_inner()`); `ShInner::SortedMulti` removed; `Terminal::SortedMulti` + `Terminal::SortedMultiA` arms added in `walk_miniscript_node`. Wire output unchanged for the `wsh(sortedmulti(...))` path; new `Tag::SortedMultiA` path added for tap-leaf `sortedmulti_a`.
- SPEC `design/SPEC_mnemonic_toolkit_v0_3.md` §4.9.a Layer 1 + Layer 2 patched in lockstep; revision Round 8.

### Future cleanup (v0.3.2)

When a miniscript crates.io release publishes containing PR #910 + #915, v0.3.2 drops the `[patch]` entry and bumps the version. Mechanical; no API or feature changes. Tracked in FOLLOWUP `tr-sortedmulti-a-via-upstream` (tier `v0.3.2`).

### Wire-bit-identical guarantee

v0.2 + v0.3.0 fixture matrices continue to validate byte-identically. New regression test confirms descriptor-mode `tr(@0, sortedmulti_a(2, @0, @1))` produces md1 byte-identical to template-mode `--template tr-sortedmulti-a` for matching keys/cosigners (`descriptor_tr_sortedmulti_a_matches_template_tr_sortedmulti_a_md1` in `parse_descriptor::tests`). This is the strongest correctness signal: the new walker arm produces the same `Tag::SortedMultiA` tree the template encoder has been producing since v0.3.0.

### Test corpus

159 unit tests + 2 ignored (was 156 + 2 in v0.3.0; +3 sortedmulti_a tests: `arm_sorted_multi_via_wsh` regression for the post-#915 `Terminal::SortedMulti` Layer-2 routing, `arm_sorted_multi_a_via_tap` for the v0.3.1 unblock target, `descriptor_tr_sortedmulti_a_matches_template_tr_sortedmulti_a_md1` for wire-bit-identical equivalence). Integration test count unchanged.

### Out of scope (still v0.4)

- Multi-leaf taproot trees (`tr(K, {A,B})` with N≥2 leaves).
- Engraving card in descriptor mode.
- Full 9 / 3+6N descriptor-aware verify-bundle check ladder (v0.3.x ships 3-element direct byte-equality ladder).
- `walker-backport-to-md-cli` — md-cli still rejects all v0.3-NEW miniscript fragments AND `sortedmulti_a` post-v0.3.1; cross-repo coordination cycle pending.

### Architect-review history

- Sketch r1: 0C / 3I / 4L → 5 action items folded into formal plan.
- Formal plan r2: 0C / 1I / 2L → 3 doc-fixes folded inline.
- End-of-phase r3: see `design/agent-reports/v0_3_1-end-of-phase-review-r1.md`.

---

## mnemonic-toolkit [0.3.0] — 2026-05-05

### What's new

- **`--descriptor "<string>"` and `--descriptor-file <path>`** flags accept any BIP-388 descriptor whose miniscript AST is supported by the v0.3 walker. Toolkit synthesizes md1 + mk1 + ms1 bundles for any combination of full / watch-only × single-sig / multisig modes detected from the descriptor's `@N` placeholder count (n=1 → single-sig regardless of outer wrapper; n≥2 → multisig).
- **Walker covers the BIP-388 surface:** all v0.2 wrappers (`wpkh`, `pkh`, `wsh+(Ms|SortedMulti)`, `sh+(Wpkh|Wsh|Ms|SortedMulti)`, `tr` keypath + single-leaf miniscript), plus 23 v0.3-NEW miniscript fragments — hash terminals (`sha256`, `hash256`, `hash160`, `ripemd160`), timelocks (`after`, `older`), wrappers (`v:`, `s:`, `a:`, `j:`, `n:`, `c:`), boolean ops (`and_v`, `and_b`, `andor`, `or_b`, `or_c`, `or_d`, `or_i`), and `thresh()`.
- **`@N[fp/path]/<multipath>/*` annotation syntax.** Full-mode `@0` requires the `[fp/path]` annotation; toolkit derives the xpub at the annotated path and cross-checks the fingerprint against the seed-derived master fp. Multi-cosigner `@N≥1` annotations are cross-checked against `--cosigner` triples.
- **`verify-bundle --descriptor`** mirror of the bundle path. Re-runs the descriptor pipeline, builds the expected ms1/mk1/md1, and compares byte-equality to the supplied cards. New `DescriptorReparseFailed` error variant (exit 4) for re-parse failures.
- **`SELF-MULTISIG WARNING`** detection extended to descriptor mode (fires when full-mode multisig descriptor has any cosigner xpub equal to the seed-derived `@0` xpub).
- **Bundle JSON schema bumped to `"3"`.** `template` field becomes nullable; new top-level `descriptor` field carries the user-supplied descriptor verbatim. Both fields ALWAYS emit (`null` when not set).

### Breaking changes (callers)

- `BundleArgs::template`: `CliTemplate` → `Option<CliTemplate>`. Clap attr `required_unless_present_any = ["descriptor", "descriptor_file"]`. Same change applied to `VerifyBundleArgs::template`.
- `BundleJson::template`: `&'static str` → `Option<&'static str>`. New `descriptor: Option<String>` field.
- `VerifyBundleJson::schema_version` and `BundleJson::schema_version`: `"2"` → `"3"`.

### Wire-bit-identical guarantee

Encoded card strings (ms1 / mk1 / md1) for any v0.2 invocation under the v0.3 binary remain byte-identical. Only the JSON envelope differs: `schema_version "2"→"3"` and a new `"descriptor": null` field appears. The v0.2 fixture corpus is preserved verbatim and continues to validate.

For descriptor-mode invocations that exactly express a v0.2 template (canonical `[fp/path]` annotation matching the BIP-44/49/84/86 paths), the resulting md1 is byte-identical to template-mode emission. Three regression tests confirm this for bip44 / bip84 / bip86 (`descriptor_bipXX_matches_template_bipXX_md1` in `parse_descriptor::tests`).

### Out of scope (deferred to v0.4)

- `tr(@0, sortedmulti_a(...))` — rust-miniscript v13.0.0 has no parser for `sortedmulti_a` in tap-leaves. Tracked in `design/FOLLOWUPS.md` (`tr-sortedmulti-a-via-upstream`); v0.4 gates on upstream parser support.
- Multi-leaf taproot trees (`tr(K, {A,B})` with N≥2 leaves). Deferred per SPEC §6.8 (Merkle-root logic).
- Engraving card in descriptor mode. Existing card builder is template-coupled; v0.4 will add a descriptor-aware card. Tracked in FOLLOWUPS (`descriptor-mode-engraving-card`).
- Full v0.4-style 9 / 3+6N descriptor-aware verify-bundle check ladder. v0.3 ships a 3-element direct-byte-equality ladder (ms1_match, mk1_match, md1_match). Functional but coarser than template-mode's 9-check schema.
- `RawPkH` and `DupIf` `Terminal` arms — descriptor-unreachable in rust-miniscript v13.0.0 (RawPkH only via raw script decode; DupIf type-restrictive). Walker handles them for completeness; tests `#[ignore]`.

### Test corpus

156 unit tests + 9 v0.3 mode-violation integration tests + all v0.2 integration tests (cli_bundle_*, cli_verify_bundle_*, cli_mode_violations_v0_2, cli_json_envelopes, etc.) green; v0.2 fixture matrix continues to pass byte-identically.

### Reproduction

Build: `cargo build --release`. Test: `cargo test --package mnemonic-toolkit`.

The v0.3 SPEC at `design/SPEC_mnemonic_toolkit_v0_3.md` (rounds 1-7, architect-reviewed 0C/0I) is normative for all descriptor-mode behavior. The implementation plan at `design/IMPLEMENTATION_PLAN_v0_3_descriptor_passthrough.md` records phase-by-phase architect-review verdicts (mid-phase + end-of-phase per phase, all addressed to 0C/0I).

---

## mnemonic-toolkit [0.2.0] — 2026-05-05

### What's new

- **Multisig templates (6 BIP-388 wrappers):** `wsh-multi`, `wsh-sortedmulti`, `sh-wsh-multi`, `sh-wsh-sortedmulti`, `tr-multi-a`, `tr-sortedmulti-a`. Threshold `1 ≤ K ≤ N ≤ 16`.
- **`--account <u32>`:** non-zero account index threading; replaces v0.1's hardcoded `account=0`.
- **`--xpub-input` multisig (watch-only):** `--cosigner <xpub>:<fp>:<path>` (repeatable) + `--cosigners-file <path>` for bulk JSON ingestion. Per-cosigner path overrides supported; `--multisig-path-family {bip48,bip87}` selects the global default (default `bip87`).
- **`--privacy-preserving`:** whole-bundle privacy boolean. Suppresses `master_fingerprint` from mk1 origins (multisig only); single-sig watch-only with `--xpub` rejects the flag (would produce inconsistent bundle vs. md1's `tlv.fingerprints`).
- **`--self-check`:** post-emit synthesize-then-verify pass on the bundle just produced. Catches synthesis/verify drift before the user engraves.

### Wire-bit-identical guarantee

Encoded card strings (ms1 / mk1 / md1) are byte-identical to v0.1's output for any v0.1-equivalent invocation (single-sig, account=0, no `--privacy-preserving`, no `--self-check`). v0.1 decoders consuming v0.2-emitted encoded strings work unchanged. The 16-cell v0.1 fixture corpus at `tests/vectors/v0_1/` is preserved verbatim and gated by `cli_bundle_full.rs` as a regression set; SHA-256 pin `81828299c927783d915108f32c9752b3dbf815c1caba5b6f6e7ce7b810ddcbf6` continues to hold for that subdirectory.

### JSON envelope evolution

- `schema_version` bumps `"1"` → `"2"`.
- New `bundle` fields: `multisig` (discriminated-union: `null` for single-sig; `{ k, n, template, path_family, cosigners: [...] }` for multisig), `privacy_preserving` (bool), `origin_paths` (per-cosigner path list when divergent from family default).
- `mk1` field becomes a `oneOf` shape: flat object for single-sig, array of N grouped chunk-set objects for multisig.

### v0.1 SHA pin retired; v0.2 SHA pin

The v0.1 fixture pin (`81828299...`) is retired as the active regression baseline (it remains as the `tests/vectors/v0_1/` byte-identity check). The v0.2 corpus adds 34 new multisig + axis cells under `tests/vectors/v0_2/`. Reproduction command (resolves v0.1 FOLLOWUPS N-1, the missing SHA-reproduction recipe):

```bash
shasum -a 256 crates/mnemonic-toolkit/tests/vectors/v0_2/*.txt | sort | shasum -a 256
# a381761656fd165e8e5af28574a5baaa55973e562c610254ae6f31d6b1f06171
```

### Tests

76 unit + 31 integration test functions = 107 total (`cargo test --workspace`). The 31 integration functions cover ~54 parametric cells across 13 test binaries. New v0.2 integration tests:
- `cli_bundle_multisig_full.rs` — 24-cell multisig fixture parametric (6 templates × 4 networks).
- `cli_account_flag.rs` — 4-cell `--account 5` parametric.
- `cli_privacy_preserving.rs` — 4-cell `--privacy-preserving` parametric.
- `cli_self_check.rs` — 2 happy-path self-check fixtures (single-sig + multisig).
- `cli_mode_violations_v0_2.rs` — 7 v0.2 NEW SPEC §6.6 mode-violation rows (byte-exact text + exit-2 contract).

### Known limitations (v0.3+ deferred)

- K-of-N share encoding (split mk1 / split ms1 / split md1) deferred — ms1 first per BIP-93.
- `--cosigners-file` user-supplied file output / multi-file output deferred.
- Hash-locks / timelocks / advanced descriptor variants deferred.
- `cargo publish` of the toolkit still gated on `ms-codec` / `mk-codec` / `md-codec` reaching crates.io. v0.2.0 distributed via GitHub tag `mnemonic-toolkit-v0.2.0`.

### Wire-format SHA pin

```text
sha256(crates/mnemonic-toolkit/tests/vectors/v0_2/) = a381761656fd165e8e5af28574a5baaa55973e562c610254ae6f31d6b1f06171
```

## mnemonic-toolkit [0.1.0] — 2026-05-04

### What's new

- Initial release. Top-level integration crate of the m-format star.
- 2 subcommands: `bundle` (encode-side: emit 3-card engraving bundle) and `verify-bundle` (round-trip integrity check).
- 2 input modes per command: full (`--phrase`) and watch-only / key-only (`--xpub --master-fingerprint`).
- 4 single-sig wallet templates: BIP-44 (pkh), BIP-49 (sh-wpkh), BIP-84 (wpkh), BIP-86 (tr).
- 4 networks: mainnet / testnet / signet / regtest.
- Account hardcoded `0` in v0.1; `--account` flag deferred to v0.2.
- All 10 BIP-39 wordlists supported via `--language`.
- Multi-section stdout (`# ms1` / `# mk1` / `# md1` headers + chunked engraving form).
- Byte-exact engraving-card stderr per SPEC §5.2.
- `--json` envelope schemas for both subcommands.
- Exit codes 0 / 1 / 2 / 3 / 4 / 64 per SPEC §6.
- Byte-deterministic mk1 `chunk_set_id` derived from the 4-byte `policy_id_stub` (mirrors md-codec's deterministic CSI derivation), so toolkit output is byte-reproducible across runs and the SHA-pinned regression corpus is meaningful.

### Tests

17 integration tests (assert_cmd) + 54 unit tests. Trezor 24-word zero-entropy vector pinned across 16 (template × network) cells.

### Known limitations

- Multisig templates, non-zero account, file output, recovery flow: deferred to v0.2+.
- `cargo publish` blocked until ms-codec / mk-codec / md-codec hit crates.io. v0.1.0 distributed via GitHub tag `mnemonic-toolkit-v0.1.0`.

### Wire-format SHA pin

The 16 fixture files at `crates/mnemonic-toolkit/tests/vectors/v0_1/*.txt` are SHA-256-pinned at this release. Subsequent corpus changes that alter the SHA require a SemVer minor bump per the pre-1.0 breaking-change-axis convention.

```text
sha256(crates/mnemonic-toolkit/tests/vectors/v0_1/) = 81828299c927783d915108f32c9752b3dbf815c1caba5b6f6e7ce7b810ddcbf6
```
