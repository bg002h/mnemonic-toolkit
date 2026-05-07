# Changelog

All notable changes to `mnemonic-toolkit` are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project follows [SemVer](https://semver.org/spec/v2.0.0.html) with the pre-1.0 convention that the second component (`0.X`) is the breaking-change axis.

## mnemonic-toolkit [0.7.0] — 2026-05-06

### Added

- `mnemonic convert` gains 4 new `NodeType` targets: `bip38`, `minikey`, `electrum-phrase`, `address`.
- **BIP-38 encrypt/decrypt** edges (`Wif↔Bip38`) plus composite paths (`phrase|entropy → bip38` via the `wif` intermediate). New crate dependency `bip38 = "1.1"` (Apache-2.0). SPEC §12.
- **Casascius mini-private-key** decode (`MiniKey → Wif`); SHA256 self-checksum rule per Casascius's typo-check. One-way edge (no encode direction; key search is non-deterministic). SPEC §13.
- **Electrum native seed format** (`ElectrumPhrase ↔ Entropy`); 4 SeedVersion dispatch (`01` standard, `100` segwit, `101`/`102` 2FA) via HMAC-SHA512 prefix; 2FA versions refused. Composite paths via `entropy` reach `phrase`/`xprv`/`xpub`/`wif`/etc. SPEC §14.
- **Address derivation** (`Xpub → Address`); `--script-type` flag with inference from `--template` for BIP-44/49/84/86 → P2PKH/P2SH-P2WPKH/P2WPKH/P2TR. SPEC §10.a.
- New subcommand **`mnemonic export-wallet`** — Bitcoin Core `importdescriptors` JSON (default) + BIP-388 `wallet_policy` JSON. Sparrow / Specter formats refuse with v0.8 deferral stubs. `--range` / `--timestamp` / `--bitcoin-core-version` overrides. Watch-only by definition (refuses entropy/phrase slot input). New SPEC `design/SPEC_export_wallet_v0_7.md`.
- New subcommand **`mnemonic derive-child`** — BIP-85 deterministic entropy via HMAC-SHA512 at `m/83696968'/<application>'/<index>'`. 6 in-scope applications: `bip39`, `hd-seed`, `xprv`, `hex`, `password-base64`, `password-base85`. RSA / RSA-GPG / DICE applications refused with v0.8 deferral stubs. New SPEC `design/SPEC_derive_child_v0_7.md`.

### Changed

- `NodeType` enum extended with 4 variants (`Bip38`, `MiniKey`, `ElectrumPhrase`, `Address`). `is_secret_bearing` extended for `Bip38` + `ElectrumPhrase`.

### Internal

- SPEC §11 carry-over: new `slip0132::tests::spec_info_line_template_matches_production_render` reads `SPEC_convert_v0_6.md` text via `include_str!` and asserts byte-equality against `render_slip0132_info_line` for all 8 SLIP-0132 variants. Closes the SPEC↔production drift hazard.
- `verify_bundle.rs` callsite-comments at `:208/:261/:336/:406` gain a SPEC §11 v0.7 amendment cross-pointer (Option B per architect R1-I8 — verify-bundle remains silent on SLIP-0132 input-normalization signals; documented as intentional checker semantics).
- New module `bip85.rs` — BIP-85 derivation primitive + 6 application dispatchers.
- New module `electrum.rs` — `SeedVersion` enum + HMAC-SHA512 prefix dispatch + entropy↔phrase encode/decode.
- New module `wallet_export.rs` — descriptor pipeline + Bitcoin Core / BIP-388 formatters + watch-only validator.
- 3 new error variants on `ToolkitError`: `ExportWalletSecretInput`, `ExportWalletFormatStub(&'static str)`, `ExportWalletTaprootMultisigUnsupported(&'static str)` (all exit 2). Plus 3 new derive-child variants: `DeriveChildUnsupportedApp`, `DeriveChildLengthOutOfRange`, `DeriveChildLengthNotApplicable` (all exit 2).

### Fixed

- `convert.rs:565` — `--to` unknown-node hint string was stale since v0.6 (omitted `bip38`, `minikey`, `electrum-phrase`, `address`); now enumerates all 13 NodeType tokens.

### FOLLOWUPS resolved

- `slip0132-info-line-spec-text-not-byte-pinned` — SPEC §11 byte-pin test shipped (Phase 7, `354c945`).
- `verify-bundle-discards-slip0132-input-variant-asymmetry` — Option B locked: 4 callsite-comments cross-pointed to SPEC §11 v0.7 amendment; intentional checker semantics (Phase 7, `354c945`).
- `bip38-encrypted-wif` — `Wif↔Bip38` edges + composite paths via `bip38 = "1.1"` (Phase 1, `c3d0a85`).
- `casascius-mini-private-key` — `MiniKey → Wif` decode-only edge with SHA256 self-checksum (Phase 2, `89d29ab`).
- `bip85-deterministic-entropy` — `mnemonic derive-child` subcommand with 6 in-scope apps (Phase 6, `965cc3e`).
- `electrum-native-seed-format` — `ElectrumPhrase ↔ Entropy` edges with 4-version dispatch + 2FA refusal (Phase 3, `892139c`).
- `address-derivation-from-xpub-path` — `(Xpub, Address)` edge with `--path` mandatory + `--script-type` template-inferred (Phase 4, `940ec0b`).
- `wallet-export-industry-formats` — `mnemonic export-wallet` subcommand with Bitcoin Core importdescriptors + BIP-388 wallet_policy (Phase 5, `3821f66`).

### Test corpus

363 lib + integration tests at v0.6.2 → 444 at v0.7.0 (2 ignored, pre-existing).

## mnemonic-toolkit [0.6.2] — 2026-05-06

### Added

- `mnemonic convert` and `mnemonic bundle` now emit a stderr informational line when a SLIP-0132 input prefix (`ypub | Ypub | zpub | Zpub` mainnet; `upub | Upub | vpub | Vpub` testnet) is silently normalized to its BIP-32 neutral form (`xpub` / `tpub`). Closes the v0.6.1 UX gap where intent signals were lost without trace. Emission is independent of `--json` and `--no-engraving-card`. Multi-slot bundles emit one note per slot in slot-index ascending order.

### Changed

- SPEC §5.5.a relaxed: the secret-on-stdout warning is the last stderr write *when it fires*; informational notes precede the engraving-card block. Deterministic stderr ordering: `informational notes → engraving card → secret-on-stdout warning (conditional)`. See `design/SPEC_mnemonic_toolkit_v0_5.md` §5.5.a (v0.6.2 amendment block).

### Internal

- `slip0132::normalize_xpub_prefix` return type changed from `Result<String, ToolkitError>` to `Result<(String, Option<&'static str>), ToolkitError>` to thread the variant-name signal up to the emission layer. `pub(crate)` API only — no impact on external consumers.
- `bundle::resolve_slots` return type extended with a `Vec<(u8, &'static str)>` slot-index→variant-name signal channel. `pub(crate)` API only.

### Fixed

- `cmd::convert::run` had duplicate `// 8)` step-label comments (`8) Compute outputs.` and `8) Emit.`). Renumbered for sequence clarity. Closes FOLLOWUPS `convert-run-step-numbering-duplicate-8`.

### FOLLOWUPS resolved

- `slip0132-input-normalization-stderr-info` — SLIP-0132 input-normalization stderr info-line shipped (this release).
- `convert-run-step-numbering-duplicate-8` — duplicate `// 8)` step labels in `cmd::convert::run` renumbered (this release).

## mnemonic-toolkit [0.6.1] — 2026-05-06

### What's new (v0.6.1 — `convert` polish + `bundle` retrofit)

A patch release bundling four small additive items consolidated under a single SPEC-amendment cycle (`SPEC_convert_v0_6.md` v0.6.1 + `SPEC_mnemonic_toolkit_v0_5.md` §5.5.a). All four items are additive — no breaking changes; no wire-format change to existing bundles or convert outputs.

- **`phrase`/`entropy` → `wif` edge** (SPEC-A) — previously deferred-in-code (BadInput at `convert.rs:482-484`); now a fully supported edge that derives a leaf privkey at an explicit `--path` and serializes via `bitcoin::PrivateKey::to_wif()` with `compressed: true` (BIP-32 §4 mandate). `--path` is REQUIRED — the toolkit does NOT auto-default a path from `--template`/`--account`. Refusal exits 2 (`ToolkitError::ConvertRefusal`) when `--path` is absent. SPEC §8 invariant: `--passphrase` is meaningful for this edge (the PBKDF2 pipeline is traversed).
- **SLIP-0132 prefix-tolerant input** (SPEC-B / new §11) — `convert --from xpub=...`, `bundle --slot @0.xpub=...`, and `verify-bundle --slot @0.xpub=...` accept SLIP-0132 prefix variants in addition to BIP-32 neutral `xpub`/`tpub`. 8 prefixes recognized: `ypub`/`Ypub`/`zpub`/`Zpub` (mainnet → swap to `xpub`); `upub`/`Upub`/`vpub`/`Vpub` (testnet → swap to `tpub`). Implementation in new `src/slip0132.rs` via base58check decode → version-byte swap → re-encode (key material is unchanged; encoding-only normalization). Unknown prefix exits 1 with byte-exact stderr. Spike: `design/agent-reports/spike-slip0132-v0_6_1-pre-spec.md`.
- **`--xpub-prefix <variant>` output flag** (SPEC-C / new §11.a) — emit `xpub`-typed targets with a SLIP-0132 prefix. 5 flag values (`xpub` default / `ypub` / `Ypub` / `zpub` / `Zpub`); testnet variants are network-context-derived via `--network`, not separate flag values. **`--network` REQUIRED when `--xpub-prefix` is non-default** (refuses with byte-exact stderr; eliminates a "testnet user gets mainnet zpub" bug class). Silent no-op on non-xpub targets. New `(xpub, xpub)` edge in §2 supports the round-trip primitive cited in §11.a.
- **`bundle` secret-on-stdout warning** (SPEC-D / new §5.5.a) — `bundle.rs::emit_unified` now emits the same byte-exact stderr warning as `convert` §7 when `Bundle::any_secret_bearing()` returns true. Watch-only invocations (all `ms1[i] == ""` sentinel per §5.8) suppress it. Wif-only-bundle limitation per SPEC: WIF slots produce empty-string ms1, so the warning is silently suppressed even when WIF is supplied as input — the warning's scope is BIP-39 entropy emission, not WIF.

### Test corpus

- **239 lib + 100 integration tests** at v0.6.1 (was 230 lib + 67 integration at v0.6.0). Net +9 lib unit tests (all in new `slip0132.rs`) + 33 integration tests:
  - `cli_convert_slip0132.rs` (NEW, 15 tests).
  - `cli_convert_round_trips.rs` (NEW, 3 tests).
  - `cli_convert_happy_paths.rs` (+9: 3 from Phase B `phrase/entropy → wif`, 6 from Phase E coverage tightening).
  - `cli_convert_refusals.rs` (+2: Phase B no-`--path` refusal for both phrase and entropy sources).
  - `cli_bundle_full.rs` (+2: Phase D text-mode + JSON-mode positive warning assertions).
  - `cli_bundle_watch_only.rs` (+1: Phase C zpub cross-cut, plus an in-place stderr negative assertion).
  - `cli_descriptor_mode.rs` (+1: Phase C descriptor-mode zpub cross-cut).
  - `cli_bundle_multisig.rs` (in-place stderr negative assertion only; no new test function).
- 16-cell parametric `bundle_full_16_cells_byte_exact_against_pinned_vectors` continues to pass — the new bundle stderr warning does not perturb the wire-format byte-identity invariant.

### FOLLOWUPS resolved

- `secret-on-stdout-warning-bundle-retrofit` (resolved Phase D, commit `66ff7c0`).
- `convert-phrase-to-leaf-wif` (resolved Phase B, commit `62b4f23`).
- `convert-test-coverage-tightening` (resolved Phase E, commit `59140c5`).
- `convert-slip0132-prefix-support` (resolved Phase C, commit `bb77164`).

### Internal

- New module `src/slip0132.rs` with `XpubPrefix` enum + `normalize_xpub_prefix` + `apply_xpub_prefix` + clap value-parser. 9 inline unit tests pin the byte-level swap mechanics against the BIP-84 reference vector.
- `derive_slot::derive_bip32_at_path` — sibling helper to `derive_bip32_from_entropy` for path-driven leaf derivation (used by the `phrase/entropy → wif` edge).
- `convert.rs::edge_uses_pbkdf2` extended to include `Wif` per SPEC §8 v0.6.1 invariant.

## mnemonic-toolkit [0.6.0] — 2026-05-06

### What's new (v0.6.0 — `mnemonic convert` subcommand)

A new orthogonal subcommand for single-format conversions between BIP-39 phrase, BIP-39 entropy, BIP-32 xpriv/xpub, WIF, fingerprint, path, and the codex32 codec encodings ms1 and mk1. The subcommand makes conversions a first-class CLI operation rather than a side-effect of bundle synthesis.

- **New subcommand `mnemonic convert`**, governed by the new `design/SPEC_convert_v0_6.md` (architect-approved 0C/0I at r3).
- **9-node typed conversion graph.** `phrase`, `entropy`, `xpub`, `xprv`, `wif`, `fingerprint`, `path`, `ms1`, `mk1`. Direct edges enumerated in `is_supported_direct_edge`; any (from, to) NOT in the set is auto-refused as a one-way barrier (exit 2). Deferred nodes (`seed`, `raw_privkey`) are documented but not yet emit/accept-supported (gated on ms-codec v0.2). `md1` is deliberately excluded (descriptors are bundle artifacts).
- **Three refusal classes** (one-way cryptographic barrier / lossy compression / cross-format pivot) with byte-exact stderr templates. `xpub → mk1` has a distinct refusal redirecting to `mnemonic bundle` (mk1 cards bind xpubs to specific policies via `policy_id_stubs`; standalone encoding is meaningless).
- **`--from`/`--to` grammar.** Single-from-value v0.6 constraint (one primary value-bearing `--from` plus optional side-input `--from path=...` / `--from fingerprint=...`); multi-value `--from` reserved for future `--slot @N` indexing.
- **`--from <node>=-` stdin convention** for any single-line node; `mk1` reads whitespace-separated tokens from stdin.
- **ConvertJson schema-1 envelope** independent of `BundleJson`. `from_value` omitted when `from_node` is secret-bearing (privacy hygiene); `to` array preserves `--to` argument order.
- **Side-channel hygiene:** stderr warning when secret material is on stdout. New convention in v0.6; bundle retrofit tracked at FOLLOWUP `secret-on-stdout-warning-bundle-retrofit`.
- **`--passphrase` ignored-on-non-PBKDF2-edge stderr warning** — explicit (higher-stakes than other ignored side-inputs).
- **`wif → xpub` sentinel stderr warning** — emits depth-0 sentinel xpub with zeroed chain code; warns the resulting xpub is not BIP-32 derivable. Refuses `wif → xpub --path m/...` (chain code destroyed).
- **`derive::DerivedAccount` extended** with `account_xpriv: Xpriv` field to support the `phrase/entropy → xprv` edge. Both `derive::derive_full` and `derive_slot::derive_bip32_from_entropy` populate it.
- **New error variant** `ToolkitError::ConvertRefusal(String)`; exit code 2.

### Test corpus

230 lib + 67 integration tests pass (was 230 lib + 44 integration in v0.5.2). 23 new convert tests across 4 files: `cli_convert_happy_paths.rs` (11 edges + mk1→xpub decode), `cli_convert_refusals.rs` (7 refusal classes, byte-exact stderr), `cli_convert_json.rs` (3 envelope shape tests), `cli_convert_help_fixtures.rs` (2 help-text smoke tests).

### FOLLOWUPS

- New: `secret-on-stdout-warning-bundle-retrofit` — apply v0.6 §7 secret-on-stdout warning to `bundle` for cross-tool consistency.
- New: `convert-seed-and-raw-privkey-nodes` — add `seed`, `raw_privkey`, `xprv`-via-ms1, `seed`-via-ms1 nodes when ms-codec v0.2 ships.
- New: `convert-phrase-to-leaf-wif` — implement `phrase/entropy → wif` (path-to-leaf-WIF derivation; deferred from v0.6).

### Wire format

Bundle/verify-bundle wire format unchanged. Convert subcommand is additive.

### Architect review reports

- `design/agent-reports/spike-convert-v0_6_0-pre-spec.md` — Phase 0 codec call-shape spike.
- `design/agent-reports/v0_6_0_phase_spec_r3.md` — SPEC 0C/0I at r3.
- `design/agent-reports/v0_6_0_phase_impl_r1.md` — implementation review (0C/2I/2L/1N → 0C/0I after foldings).

## mnemonic-toolkit [0.5.2] — 2026-05-06

### What's new (v0.5.2 — derive_slot helper extraction)

Pure refactor patch. Sets up a shared call site for the upcoming v0.6.0 `mnemonic convert` subcommand without conflating refactor risk with new-feature risk.

- **`derive_slot.rs` (NEW).** `derive_bip32_from_entropy(entropy, passphrase, language, network, template, account) -> Result<DerivedAccount>` consolidates the BIP-39 + BIP-32 derivation spine that was duplicated between `bundle::resolve_slots`'s phrase and entropy branches.
- **`derive::DerivedAccount` extended.** New field `account_path: DerivationPath` populated via the helper. `derive_full` is now a thin wrapper that parses the phrase to entropy and delegates.
- **`bundle::resolve_slots` simplified.** Phrase + entropy branches each shrink from ~22 LOC to ~10 LOC, calling the shared helper. The xpub / wif / xprv-rejected branches stay unchanged.

### Wire format

Byte-identical to v0.5.1. 230 lib + 44 integration tests pass (2 lib ignored, pre-existing). The pre-shipped 16-cell parametric fixture in `cli_bundle_full.rs` continues to match.

### Architect review report

- `design/agent-reports/v0_5_2_phase_extract_r1.md` (0C/0I — APPROVED; 1 unused-import nit folded inline).

## mnemonic-toolkit [0.5.1] — 2026-05-06

### What's new (v0.5.1 — close the v0.5.0 partial-delivery deferrals)

v0.5.1 closes the 2 FOLLOWUPS deferred from v0.5.0 (`legacy-cli-flag-deletion` + `legacy-flag-deprecation`). The unified `--slot @N.<subkey>=<value>` syntax is now the sole input shape for slot-bearing data; the v0.4-era legacy CLI flags are deleted entirely from `BundleArgs` + `VerifyBundleArgs` along with their alias plumbing.

- **Phase A.1a — source-side deletions.** 6 legacy fields (`--phrase`, `--xpub`, `--master-fingerprint`, `--cosigner`, `--cosigners-file`, `--cosigner-count`) deleted from both `BundleArgs` and `VerifyBundleArgs`. `bundle::bundle_args_to_slots` and `slot_input::expand_legacy_to_slots` shims (+ 5 unit tests) deleted entirely. 9 mode-violation guards swept from `bundle.rs::run`; 11 mode-text consts removed (`PASSPHRASE_WITH_XPUB`, `LANGUAGE_WITH_XPUB`, `XPUB_NEEDS_FINGERPRINT`, `FINGERPRINT_WITHOUT_XPUB`, `XPUB_STDIN`, `XPUB_AND_COSIGNER`, `COSIGNER_AND_COSIGNERS_FILE`, `COSIGNER_COUNT_WITHOUT_MULTISIG`, `PRIVACY_WITH_XPUB`, `ACCOUNT_INCOMPATIBLE_TEMPLATE`, `DESCRIPTOR_WITH_COSIGNER_COUNT`); 3 retained guards: `THRESHOLD_WITHOUT_MULTISIG`, `PATH_FAMILY_WITHOUT_MULTISIG`, `DESCRIPTOR_AND_TEMPLATE` (plus the v0.3 retained descriptor-mode set).
- **Phase A.1d — verify-bundle slot dispatch refactor.** `VerifyBundleArgs` gains a `pub slot: Vec<SlotInput>` field with parity to `BundleArgs::slot`. `bundle::resolve_slots` refactored to take an explicit args-tuple `(template, network, account, language, passphrase)` and promoted to `pub(crate)`; both `bundle.rs` and `verify_bundle.rs` share the helper. `verify_bundle::run` reshaped to dispatch via slot-shape detection; `run_full` / `run_watch_only` / `run_multisig` / `descriptor_mode_verify_run` rewired to consume slots through `synthesize_unified` (template mode) or `synthesize_descriptor` (descriptor mode).
- **Phase A.1b/c — test corpus migration.** 3 `cli_mode_violations*.rs` files deleted (~584 lines, 61 legacy-flag references). New `cli_mode_violations_v0_5.rs` (6 tests; byte-exact stderr) covers the 3 retained guards.
- **Phase A.2 — consumer test rewrites.** 13 `cli_*.rs` integration test files rewritten per the v0.5.0 mapping table. Special handling: `cli_unified_slot.rs` row-6 collision test + dead `TREZOR_BIP84_XPUB` const deleted; `cli_bip388_distinctness.rs` row-5-conflict test deleted (trap unreachable post-`--cosigner-count` deletion).
- **Phase A.3 — SPEC §6.6 partial-delivery note removal.** The v0.5.0 SPEC paragraph acknowledging the deferral is deleted; the §6.6 table now reflects shipped state.
- **Path-defaulting refinement.** `bundle::resolve_slots` Xpub branch now defaults the path from `template.derivation_path(network, account)` when the slot lacks an explicit `Path` subkey. Preserves v0.4 watch-only path-default semantics; required for verify-bundle round-trip on bip84/etc account-paths.

### Breaking changes

Per "no users yet → break anything" license:

- **6 legacy CLI flags deleted entirely.** `--phrase`, `--xpub`, `--master-fingerprint`, `--cosigner`, `--cosigners-file`, `--cosigner-count` are now unknown to clap (exit 2 unknown-arg). Use `--slot @N.<subkey>=<value>` instead.
- **Mode-violation pre-check ladder reduced.** 9 guard branches removed; 3 retained. Stderr text for the 3 retained guards is unchanged byte-for-byte.

### Test corpus

230 lib + 44 integration tests pass (2 lib ignored, pre-existing). Net delta: -6 lib (5 expand-legacy unit tests + 1 watch-only-stderr test), -3 integration files (cli_mode_violations*.rs), +1 integration file (cli_mode_violations_v0_5.rs), -2 integration tests within rewritten files.

### Carry-forward

v0.5.0 schema-4 `bundle --json` envelopes continue to emit byte-identically. The legacy-flag → `--slot` rewrite is wire-format-neutral.

### Architect review reports

- `design/agent-reports/v0_5_1_phase_atomic_r1.md` (Commit 1, 0C/0I/0L/2N).
- `design/agent-reports/v0_5_1_phase_spec_r1.md` (Commit 2, 0C/0I).

## mnemonic-toolkit [0.5.0] — 2026-05-06

### What's new (v0.5.0 — bundle the v0.4.5-nice-to-have + open `*-nice-to-have` deferrals)

v0.5.0 closes 13 open FOLLOWUPS across 6 of the 7 planned phases. The user's strongest "no users yet → break anything" license is exercised: a deliberate SPEC §4.11.b reversal (typed-DerivationPath equality), a JSON envelope `engraving_card` field deletion, a four-case ms1 short-circuit table with byte-exact `decode_error` strings, and a `MappingFailure` enum for mk1 cosigner-mapping diagnostics.

A new SPEC document `design/SPEC_mnemonic_toolkit_v0_5.md` is created (v0.4 retained for historical reference). Cycle artifacts: `/home/bcg/.claude/plans/robust-cooking-kazoo.md` (in-plan-mode brainstorm + SPEC + plan all converged 0C/0I across multiple architect rounds).

- **Phase S0 — SPEC v0.5 document.** New `SPEC_mnemonic_toolkit_v0_5.md` with 6 normative amendments: §4.11.b deliberate reversal, §5.7 line 103 multiset semantics for `md1_xpub_match`, §5.7 line 104 four-case ms1 table, §5.7 NEW mk1-mapping-diagnostic paragraph, §5.5 `engraving_card` field deletion, §6.6 legacy-flag-deletion sketch (full deletion deferred to v0.5.1).
- **Phase B — multisig helper polish (5 items).** B.1 new `helper_multisig_full_emits_3plus6n_checks_in_spec_order` unit test. B.2 positional-fallback condition refactored to `match`. B.3 `md1_xpub_match` now multiset (sort-then-compare with multiplicity). B.4 `MappingFailure` enum (`NotSupplied` / `DecodeFailed(String)` / `XpubNotInPolicy`) replaces `Vec<Option<&KeyCard>>`; precedence `XpubNotInPolicy > DecodeFailed > NotSupplied`. B.5 four-case ms1 emission per SPEC §5.7 line 104 — full-mode supplied-absent case now `passed: false` (was `passed: true` in v0.4.5) with byte-exact `decode_error: "error: ms1[{i}] expected (full-mode bundle) but not supplied"`.
- **Phase C — SPEC reversals (3 items).** C.1 `check_key_vector_distinctness` switches from raw-string `path_raw == path_raw` to typed `path == path` (folds `h` → `'`). v0.4.1 `bip388_h_vs_apostrophe_paths_distinct_under_raw_string` test migrated to `bip388_h_vs_apostrophe_paths_collide_under_typed_equality_v0_5`. C.2 SPEC-only codification of watch-only spurious-`--ms1` short-circuit + new integration test. C.3+C.4 `detect_removed_subcommand` trap deleted entirely (~80 lines including 5 inline tests); 2 byte-exact-stderr tests migrated to clap-fallback exit-64 assertions.
- **Phase D — schema-2/3 placeholder rejection deletion.** `load_bundle_json_into_args`'s peek-and-reject `schema_version` branch deleted (~16 lines including the FOLLOWUP placeholder pointer). Schema-mismatch envelopes now fail at the underlying field extraction.
- **Phase E — `origin_path` null unification (single-sig).** New `origin_path_for_json(path_raw)` helper returns `None` when `path_raw.is_empty()` (was `Some("m")` via the v0.4.2 normalize fallback).
- **Phase F — text-mode trailing-space fix.** Three identical `writeln!` emit sites in `cmd/verify_bundle.rs` rewritten to branch on `c.detail.is_empty()` (no more `"md1_xpub_match: skipped "` trailing space).
- **Phase A.3 — engraving-card dead-field cleanup.** `BundleJson.engraving_card: Option<String>` field DELETED + 2 always-`None` initializers DELETED + stale doc-comment rewritten. Active stderr emission path (`build_unified_card` + `engraving_card_unified`) and `--no-engraving-card` CLI flag both preserved.

### Deferred to v0.5.1 (Phase A scope reduction)

- **`legacy-cli-flag-deletion`** — Delete `--phrase`, `--xpub`, `--cosigner`, `--master-fingerprint`, `--cosigner-count`, `--cosigners-file` from `BundleArgs` + `VerifyBundleArgs`. Rewrite ~25 integration tests (~1500 LOC churn) to use `--slot @N.<subkey>=<value>` syntax exclusively.
- **`legacy-flag-deprecation`** — superseded by the deletion above.
- **Mode-violation guard sweep + new `cli_mode_violations_v0_5.rs`** — 9 guards delete; 3 retain (`THRESHOLD_WITHOUT_MULTISIG`, `PATH_FAMILY_WITHOUT_MULTISIG`, `DESCRIPTOR_AND_TEMPLATE`). New test file pinning the 3 retained guards.

Per the plan's explicit scope-reduction trigger, the ~2500 LOC of mechanical-but-error-prone churn is deferred to its own cycle, matching the v0.4.4→v0.4.5 helper-foundation-then-rollout pattern.

### Breaking changes

Per "no users yet → break anything" license:

- **JSON envelope `BundleJson.engraving_card` field DELETED.**
- **JSON envelope `verify-bundle` `mk1_decode[i]` `decode_error` strings changed** (per SPEC §5.7 mk1-mapping diagnostic; was conflated as "skipped: mk1[i] not supplied or decode failed"; now distinguishes 3 modes).
- **JSON envelope `verify-bundle` multisig `ms1_decode[i]` / `ms1_entropy_match[i]` semantics changed** (case 4: `passed: false` for full-mode supplied-absent; was `passed: true` in v0.4.5).
- **JSON envelope `verify-bundle` `md1_xpub_match` is now multiset-equality** (was ordered Vec equality).
- **JSON envelope `bundle` `origin_path` field is `null` for absent paths** (was `"m"` in v0.4.2 unified-slot watch-only).
- **BIP-388 distinctness now treats `48h/0h` and `48'/0'` as the same path** (v0.4 raw-string equality REVERSED). Existing tests using `h`/`'` notation differences as a distinctness lever migrated.
- **`detect_removed_subcommand` trap deleted** — `mnemonic bundle multisig-full` now rejected by clap fallback (exit 64) instead of the byte-exact pre-clap stderr.
- **`--bundle-json` schema-2/3 rejection deleted** — schema-mismatch envelopes fail at field extraction (no more placeholder error pointer).
- **Plain-text `verify-bundle` output no longer has trailing spaces** when `detail` is empty.

### Wire-bit-identical guarantee

v0.4.5 schema-4 `bundle --json` envelopes continue to emit byte-identically EXCEPT for the deleted `engraving_card: null` field and `origin_path: null` (was `"m"`) for unified-slot single-sig watch-only.

### Test corpus

236 lib unit tests + 22 integration suites pass (was 243+22 in v0.4.5; net -7 lib over the cycle from C.3+C.4 trap deletion offsetting B+C+F additions).

### Cycle artifacts

- Plan: `/home/bcg/.claude/plans/robust-cooking-kazoo.md` (in-plan-mode brainstorm + SPEC + plan all converged 0C/0I).
- SPEC: new `design/SPEC_mnemonic_toolkit_v0_5.md`.
- Per-phase reports: `design/agent-reports/phase-{S0,B,C,D,E,F,A}-*-review-r1.md`.

### Architect-review history

- Brainstorm: 2 rounds (r1 0C/2I/3L → addressed; r2 0C/0I/2L → addressed).
- SPEC: 3 rounds (r1 0C/2I/2L → addressed; r2 0C/1I/1L → addressed; r3 0C/0I → APPROVE).
- Implementation plan: 2 rounds (r1 0C/3I/3L → addressed; r2 0C/0I/2L → addressed).
- Per-phase reviews: S0 0C/2I addressed; B-F 0C/0I; A 0C/0I (scope-reduced).
- Final cross-phase review: APPROVED 2026-05-06 (2 Important re: CHANGELOG arithmetic + SPEC §6.6 partial-delivery note both addressed inline; 2 Low/Nit deferred).

---

## mnemonic-toolkit [0.4.5] — 2026-05-06

### What's new (v0.4.5 helper call-site rollout + 9/3+6N descriptor-mode parity)

v0.4.5 finishes the v0.4.4 helper-foundation work by wiring `emit_verify_checks` into all four production verify-bundle dispatch paths (`run_full`, `run_watch_only`, `run_multisig`, `descriptor_mode_verify_run`), expanding the helper to emit the SPEC §5.7 3+6N multisig schema, dropping the legacy `stub_linkage` v0.1 leftover, and adding forensic-field integration tests. Per the user's "no users yet → ignore migration" license, the JSON envelope check-array shape changes are taken directly without compatibility shims.

- **Phase P.3+P.6 — `run_full` + `run_watch_only` via helper.** Replaced ~270 lines of duplicated push-site logic with helper-routed shapes (~50 lines each). Deleted `verify_md1_and_stub` (~107 lines), `verify_md1_only` (~58 lines), `watch_only_checks` (~210 lines), and 5 obsolete unit tests (~165 lines). The single-sig 9-check JSON envelope shape changes: `stub_linkage` is dropped (was a v0.1 leftover with no SPEC §5.7 equivalent); `ms1_decode` joins at position 0 (canonical SPEC §5.7 ordering). `cli_json_envelopes.rs` test pin migrated in lockstep. Runs cmd/verify_bundle.rs from 2365 → 1707 lines (-658 net).
- **Phase P.4 — multisig 3+6N helper expansion.** New `emit_multisig_checks` (~280 lines) implements SPEC §5.7 line 103 multisig schema: 6N per-cosigner [i]-indexed checks (`ms1_decode[i]`, `ms1_entropy_match[i]`, `mk1_decode[i]`, `mk1_xpub_match[i]`, `mk1_fingerprint_match[i]`, `mk1_path_match[i]`) interleaved by cosigner, then 3 shared md1 checks (`md1_decode`, `md1_wallet_policy`, `md1_xpub_match`). Watch-only / wif slots short-circuit ms1 checks per SPEC §5.7 lines 104-106. `run_multisig` body collapses from ~450 lines to ~85 lines via synthesize → SuppliedCards → helper. JSON envelope shape change: per-cosigner `md1_xpub_match[i]` (×N) replaced by single shared `md1_xpub_match`; per-cosigner `stub_linkage[i]` (×N) dropped entirely (no SPEC §5.7 equivalent). New helper unit test `helper_multisig_watch_only_emits_3plus6n_checks_in_spec_order` pins the 3+6N name vec via the watch-only synthesis path; full-mode multisig 3+6N unit-level coverage is open as FOLLOWUP `verify-bundle-multisig-helper-full-mode-unit-test` (covered end-to-end by `cli_bundle_multisig.rs`).
- **Phase P.5 — descriptor-mode rewrite (closes 9/3+6N parity).** `descriptor_mode_verify_run` body's v0.3 3-element coarse ladder (`ms1_entropy_match`, `mk1_match`, `md1_match`) replaced with `emit_verify_checks(&expected, &supplied, descriptor.n > 1)` — yields the same SPEC §5.7 9 / 3+6N schema as template-mode. Plain-text output format also aligned to template-mode (`{name}: ok|fail {detail}` per check + `result: {result}` trailer). Closes FOLLOWUP `verify-bundle-9-3plus6n-descriptor-mode-parity`.
- **Phase L — helper foundation cleanup.** L-1: `emit_verify_checks` doc-comment §5.8 → §5.7 (watch-only short-circuit semantics live in §5.7; §5.8 is the MsField wire format). L-2: `MkField::Multi` early-return arm in the single-sig branch replaced with `unreachable!()` — converts silent data truncation into loud invariant violation now that the helper is live. Closes FOLLOWUP `verify-bundle-helper-foundation-cleanup-v0.4.5`.
- **Phase P.7 — forensic-field integration tests.** New `cli_verify_bundle_forensics.rs` (3 tests): pass-checks omit forensic fields per `#[serde(skip_serializing_if = "Option::is_none")]`; garbage-payload tamper exercises `decode_error` population on `ms1_decode`; watch-only mode emits `decode_error: "skipped: watch-only slot"` on `ms1_decode` + `ms1_entropy_match`.

### Deferred to v0.4.5-nice-to-have / v0.4.6+

- **`verify-bundle-multisig-md1-xpub-match-set-equality`** — `md1_xpub_match` uses ordered Vec equality. SPEC §5.7 "all N pubkeys match" arguably implies set semantics. Triggered only by descriptor-mode where user provides non-canonical slot order. Re-evaluate after descriptor-mode use cases surface.
- **`verify-bundle-multisig-cosigner-mapping-diagnostic`** — distinguish "card not supplied" from "xpub not in policy" failure modes (currently conflated as "skipped: mk1[i] not supplied or decode failed").
- **`verify-bundle-multisig-missing-ms1-passes-true`** — full-mode multisig with no `--ms1` supplied reports `passed: true` for `ms1_decode[i]`/`ms1_entropy_match[i]`. SPEC §5.7 doesn't address this case.
- **`verify-bundle-watch-only-spurious-ms1-handling`** — watch-only with user-supplied `--ms1` produces `ms1_entropy_match: fail` (was silently passed-vacuously pre-v0.4.5). Behavior change; SPEC clarification pending.

### Breaking changes

JSON envelope `verify-bundle --json` check-array shape — internal-only break per "no users yet" license; no consumers to migrate:

- **Single-sig (template-mode + descriptor-mode + watch-only):** `[ms1_entropy_match, mk1_decode, ..., stub_linkage]` (9 names with stub_linkage) → `[ms1_decode, ms1_entropy_match, mk1_decode, ..., md1_xpub_match]` (9 names per SPEC §5.7).
- **Multisig (template-mode + descriptor-mode):** old per-cell shape (`[ms1_entropy_match, mk1_decode[0..N], mk1_xpub_match[0..N], ..., md1_xpub_match[0..N], stub_linkage[0..N]]`) → SPEC §5.7 3+6N (`[ms1_decode[0], ms1_entropy_match[0], mk1_decode[0], ..., mk1_path_match[N-1], md1_decode, md1_wallet_policy, md1_xpub_match]`).
- **Descriptor-mode plain-text output** also aligned to template-mode format (`{name}: ok|fail {detail}` per check + `result: {result}` trailer; was `verify-bundle: {result}` header + `  - {name} [ok|fail]: {detail}`).

### Wire-bit-identical guarantee

v0.4.0 / v0.4.1 / v0.4.2 / v0.4.3 / v0.4.4 schema-4 `bundle --json` envelopes continue to emit byte-identically. The shape changes are confined to `verify-bundle --json` and `verify-bundle` plain-text output.

### Test corpus

243 lib unit tests + 22 integration suites pass (was 244 lib in v0.4.4; -1 from `helper_multisig_returns_todo_stub` deletion replaced by `helper_multisig_full_emits_3plus6n_checks_in_spec_order`; +3 forensic integration tests).

### Cycle artifacts

- Implementation plan: `design/IMPLEMENTATION_PLAN_v0_4_5_helper_call_sites.md` (r2 APPROVE 0C/0I post-r1 fix).
- Phase reports: `design/agent-reports/phase-P3-helper-wire-up-review-r1.md`, `design/agent-reports/phase-P4-multisig-helper-review-r1.md`, `design/agent-reports/phase-P5-descriptor-mode-helper-review-r1.md`.

### Architect-review history

- v0.4.5 impl plan: 2 in-cycle rounds (r1 BLOCK 2I → 0C/0I r2; multisig check-name bracket notation + shared/per-cosigner grouping corrections inline).
- Phase P.3+P.6: 1 review round (1 Important re: stale `#[allow(dead_code)]` attrs addressed inline; 1 Low re: watch-only spurious --ms1 deferred to FOLLOWUP).
- Phase P.4: 1 review round (1 Critical re: stale doc-comment + 2 nits addressed inline; 2 Important + 1 Low deferred via 3 FOLLOWUPS at v0.4.5-nice-to-have).
- Phase P.5: 1 review round (1 Important re: plain-text format divergence + 1 nit addressed inline).
- Final cross-phase review: APPROVED 2026-05-06 (1 Important re: multisig helper test name/fixture mismatch addressed via rename + FOLLOWUP for full-mode unit coverage; 3 Low/Nit deferred via FOLLOWUPS at v0.4.5-nice-to-have tier).

---

## mnemonic-toolkit [0.4.4] — 2026-05-06

### What's new (v0.4.4 verify-bundle helper foundation + DescriptorBinding cleanup)

v0.4.4 closes the 2 v0.4.4-tier FOLLOWUPS from v0.4.3 deferral. Per the user's "no users yet → ignore migration" license, the DescriptorBinding.entropy field is deleted outright (no shim period). The Phase P scope was reduced from "helper + full ~78-site forensic rollout + descriptor-mode 9/3+6N parity" to "helper foundation only"; call-site rollout (P.3-P.7) deferred to v0.4.5.

- **Phase P.1+P.2 — `emit_verify_checks` helper foundation.** New `#[allow(dead_code)]` helper in `cmd/verify_bundle.rs` with the canonical SPEC §5.7 9-check ordering for single-sig template-mode (ms1_decode, ms1_entropy_match, mk1_decode, mk1_xpub_match, mk1_fingerprint_match, mk1_path_match, md1_decode, md1_wallet_policy, md1_xpub_match). New `SuppliedCards<'a>` struct (`{ms1, mk1, md1}` slice triplet — mk1 indexed by cosigner position with placeholder strings for absent slots; documented). New `emit_md1_checks` shared helper. Multisig path returns a TODO stub: `[VerifyCheck { name: "TODO_multisig_v0_4_5", passed: false, decode_error: Some("multisig helper rollout deferred to v0.4.5") }]`. Watch-only short-circuit: ms1[i].is_empty() → `passed: true + decode_error: Some("skipped: watch-only slot")`. 4 unit tests pin: `helper_singlesig_full_emits_9_checks_in_spec_order`, `helper_singlesig_tampered_mk1_populates_forensics`, `helper_singlesig_watch_only_short_circuits_ms1`, `helper_multisig_returns_todo_stub`. Helper landed but not yet wired to run_full / run_multisig / descriptor_mode_verify_run; that consolidation deferred to v0.4.5 (FOLLOWUP `verify-bundle-helper-call-sites-rollout-v0.4.5`). Closes structural piece of FOLLOWUP `verify-bundle-helper-and-full-forensics-rollout-v0.4.4` (superseded by v0.4.5 successor).
- **Phase S — `DescriptorBinding.entropy` field retired.** Bundle-level `entropy: Option<Vec<u8>>` field deleted from `parse_descriptor.rs::DescriptorBinding`; per-slot entropy lives on `binding.cosigners[i].entropy` (post v0.4.3 N's CosignerKeyInfo→ResolvedSlot type alias merge). New `entropy_at_0()` compatibility shim method returns `Option<&[u8]>` reading `cosigners[0].entropy`. `bind_full_mode` sets `cosigners[0].entropy = Some(entropy)` before constructing the binding. `bind_watch_only_singlesig` and `bind_watch_only_multisig` drop the field initializer. ~10 readers (parse_descriptor.rs tests, cmd/verify_bundle.rs, cmd/bundle.rs::bundle_run_unified_descriptor) migrated from `binding.entropy.as_deref()` / `binding.entropy.is_some()` / `binding.entropy.is_none()` to the helper. Closes FOLLOWUP `descriptor-binding-entropy-field-redundant`.

### Deferred to v0.4.5

- **`verify-bundle-helper-call-sites-rollout-v0.4.5`** — Phase P.3-P.7. Wire `emit_verify_checks` into run_full (P.3), run_multisig (P.4 — replace TODO stub with real 3-shared+6N-per-cosigner emission), descriptor_mode_verify_run (P.5 — closes `verify-bundle-9-3plus6n-descriptor-mode-parity` simultaneously), migrate watch_only_tests (P.6), add forensic-field integration tests (P.7).

### Breaking changes

None at the CLI surface or JSON envelope level. Internal Rust API broke: `DescriptorBinding.entropy: Option<Vec<u8>>` field deleted. Per "no users yet" license — no external Rust consumers to migrate. The `entropy_at_0()` helper method is the new accessor.

### Wire-bit-identical guarantee

v0.4.0 / v0.4.1 / v0.4.2 / v0.4.3 schema-4 bundles continue to emit byte-identically. The `bundle --json` and `verify-bundle --json` envelope shapes are unchanged from v0.4.3. The new `emit_verify_checks` helper is `#[allow(dead_code)]` in v0.4.4 — production code paths still emit the v0.4.3 P.0 shape (passed: bool with forensic fields populated only at the v0.4.1 J.7 proof-of-shape site).

### Test corpus

244 lib unit tests pass (was 240 in v0.4.3; +4 from new emit_verify_checks helper unit tests). Integration suites unchanged.

### Cycle artifacts

- Implementation plan: `design/IMPLEMENTATION_PLAN_v0_4_4_verify_bundle_finish_for_real.md` (r1 APPROVE WITH NITS; 2 LOW findings addressed inline before execution).

### Architect-review history

- v0.4.4 impl plan: 1 in-cycle round (r1 APPROVE WITH NITS — 2 LOW addressed: wif-slot handling clarified; SuppliedCards.mk1 indexing convention documented).
- Phase P.1+P.2: scope-reduced to helper foundation only; 244 tests pass post-helper.
- Phase S: scope-minimized field deletion; 244 tests pass post-migration.
- Final cross-phase review: APPROVED 2026-05-06 (1 Important re: stale CHANGELOG check-names addressed inline; 2 Low/Nit deferred via FOLLOWUP `verify-bundle-helper-foundation-cleanup-v0.4.5`).

---

## mnemonic-toolkit [0.4.3] — 2026-05-06

### What's new (v0.4.3 verify-bundle finish + unified-path edges)

v0.4.3 closes 4 of 5 v0.4.3-tagged FOLLOWUPS plus 1 NEW (`wif-multisig-resolution`). Theme: **finish verify-bundle (struct-shape correction + JSON intake) and close the unified-path edges (binding-type merge + wif multisig)**. Per the user's "no users yet → ignore migration" license, the v0.4.1-introduced VerifyCheck struct drift from SPEC §5.7 is corrected directly.

- **Phase N — `CosignerKeyInfo` → `ResolvedSlot` merge.** Sole binding type is now `ResolvedSlot`; `CosignerKeyInfo` retained as a `#[allow(dead_code)]` type alias for source-compat. Per-slot `entropy: Option<Vec<u8>>` lives on every `ResolvedSlot`. Closes FOLLOWUP `cosigner-keyinfo-resolved-slot-merge`. Bundle-level `DescriptorBinding.entropy` field retained for now (semantically redundant; tracked at NEW v0.4.4 FOLLOWUP `descriptor-binding-entropy-field-redundant`).
- **Phase R — wif slots in multisig contexts.** `resolve_slots` (cmd/bundle.rs) lifted the v0.4.2 single-sig-only guard. Wif slots produce ResolvedSlots with the wif's pubkey + zero chain code + empty path. BIP-388 distinctness applies — same WIF twice → SPEC §6.6 row 13 collision (verified by new test). Closes FOLLOWUP `wif-multisig-resolution`. 3 new integration tests in `cli_unified_slot.rs`: hybrid 2-of-3 (phrase + wif + xpub), pure wif 2-of-2 (two distinct WIFs), same-WIF-twice → row 13.
- **Phase P.0 — VerifyCheck struct shape correction.** Long-standing v0.4.1 J.1 drift from SPEC §5.7: `result: &'static str` ("ok"|"fail"|"skipped") → `passed: bool`. Skipped checks: `passed: true` (decode_error population deferred to v0.4.4 with the helper rollout). Mechanical migration of ~78 push sites in `cmd/verify_bundle.rs` + ~30 test assertions. JSON envelope: `"result": "ok"|"fail"` → `"passed": true|false`.
- **Phase Q — `--bundle-json <file>` verify-bundle JSON intake (SPEC §6.7 amended).** New CLI flag mutually exclusive with `--ms1`/`--mk1`/`--md1` triplet via clap `conflicts_with_all`. Reads a `bundle --json` envelope file, peeks `schema_version`, validates `"4"`, extracts `ms1`/`mk1`/`md1` arrays into a synthetic VerifyBundleArgs, then continues dispatch as if user had supplied the explicit triplet. Re-derivation flags (`--slot`/`--phrase`/etc.) are STILL required for expected-bundle computation. Schema-2/3 envelopes rejected with byte-exact stderr pointing at NEW v0.4.4-nice-to-have FOLLOWUP `bundle-json-schema-2-3-retro-compat`. SPEC §6.7 amended in lockstep with v0.4.3 amendment paragraph. Closes FOLLOWUP `bundle-json-cli-flag-and-dispatch`. 3 new integration tests in `cli_bundle_json_intake.rs` (round-trip, unsupported schema, conflicts_with).

### Deferred to v0.4.4

- **`verify-bundle-helper-and-full-forensics-rollout-v0.4.4`** — full Phase P (P.1 emit_verify_checks helper + P.2-P.5 ~78-site forensic rollout + descriptor-mode 9/3+6N parity refactor). Estimated ~800-1000 lines deleted in verify_bundle.rs. v0.4.3 ships the structural pieces (P.0); the heavy refactor lands in v0.4.4. Bundles `verify-bundle-9-3plus6n-descriptor-mode-parity` from v0.4.2 deferral.
- **`descriptor-binding-entropy-field-redundant`** — retire `DescriptorBinding.entropy` field after v0.4.3 N's per-slot ResolvedSlot.entropy. Cleanup-only; no behavior change.

### Breaking changes

- **JSON envelope `VerifyCheck`**: `"result": "ok"|"fail"|"skipped"` → `"passed": true|false` (skipped: `"passed": true`, `decode_error` population in v0.4.4). Per "no users yet" license — internal-only break; no existing JSON consumers to migrate. SPEC §5.7 was always specified this way; v0.4.1 had implementation drift.

### Wire-bit-identical guarantee

v0.4.0 / v0.4.1 / v0.4.2 schema-4 bundles continue to emit byte-identically. The VerifyCheck struct change affects only `verify-bundle --json` output, not `bundle --json` output.

### Test corpus

240 lib + integration suites pass (was 240 in v0.4.2; net 0 — additions: 3 wif-multisig + 3 bundle-json + struct-shape correction touched ~30 test sites; no test count delta because the v0.4.2 wif-multisig-rejected test was replaced by 3 new wif-multisig-supported tests).

### Cycle artifacts

- Implementation plan: `design/IMPLEMENTATION_PLAN_v0_4_3_verify_bundle_finish.md` (r2 APPROVE WITH NITS; nits applied).
- SPEC: `design/SPEC_mnemonic_toolkit_v0_4.md` §6.7 amended in lockstep with Phase Q.

### Architect-review history

- v0.4.3 impl plan + SPEC: 2 in-cycle rounds (r1 BLOCK 2C/3N → r2 APPROVE WITH NITS 0C/0I/1N; SPEC §6.7 amendment for `--bundle-json` landed before execution).
- Phase N: scope-minimized type alias merge; 240 tests pass post-migration.
- Phase R: scope-minimized guard lift; 3 new tests including BIP-388 collision.
- Phase P.0: SPEC §5.7 drift correction (~78-site mechanical migration); P.1-P.5 deferred to v0.4.4 atomic refactor.
- Phase Q: scope-minimized JSON intake (load + dispatch + 3 tests); helper landed without rewriting run() entry.
- Final cross-phase review: pending (this CHANGELOG entry is the gate).

---

## mnemonic-toolkit [0.4.2] — 2026-05-06

### What's new (v0.4.2 unified-path consolidation)

v0.4.2 closes the v0.4 cycle's "delete the dual-path baggage" theme. Per the user's "no users yet → ignore migration work" license, this release deletes the legacy parallel CLI dispatch path and lands the unified `--slot @N.<subkey>=<value>` path as the sole architectural shape, plus extends slot-input support and removes deprecated test patterns.

- **Phase K — additional slot subkey shapes.** `resolve_slots` (cmd/bundle.rs) now handles `{entropy}` (hex-decode → BIP-39 mnemonic → derive at template path), `{wif}` (degenerate single-key in single-sig contexts), and partial `{xpub}` shapes (`{xpub}` alone, `{xpub, fingerprint}`, `{xpub, path}`). `{xprv}` REJECTED with v0.5+ deferral pointer (FOLLOWUP `unified-slot-xprv-resolution-needs-ms-codec-extension`); `{wif}` in multisig contexts REJECTED with v0.4.3 deferral pointer (FOLLOWUP `wif-multisig-resolution`). Per-shape integration tests in `cli_unified_slot.rs`.
- **Phase L — descriptor mode under unified `--slot`.** `bundle_run_unified_descriptor` resolves each `@i` slot against the per-`@i` annotation path from the parsed descriptor (NOT template's path). Cross-checks fingerprint annotation against phrase-derived master fingerprint. Constructs CosignerKeyInfo bridge + ParsedKey + ParsedFingerprint vecs → existing synthesize_descriptor pipeline. 3 new integration tests.
- **Phase M — legacy flag deprecation (delete parallel dispatch).** `bundle::run` rewritten as a thin ~140-line wrapper holding only the SPEC §6.6 v0.2 + v0.3 mode-violation pre-checks (cli_mode_violations*.rs byte-exact pins). All synthesis and emit goes through `bundle_run_unified` regardless of whether `--slot` or legacy `--phrase` / `--xpub` / `--cosigner` was supplied. New `bundle_args_to_slots` helper folds ALL legacy flags into a unified `Vec<SlotInput>` with the locked cosigner offset rule (phrase present → cosigners @1+; phrase absent → cosigners @0+). Deleted ~990 lines: `bundle_full`, `bundle_watch_only`, `bundle_multisig_full`, `bundle_multisig_watch_only`, `emit`, `emit_multisig`, `descriptor_mode_run`, `descriptor_mode_emit`, `derive_threshold_from_descriptor_tree`, `BundleArgs::template_unchecked`. `emit_unified` text-mode preserves v0.3 UX (ms1-omitted markers, "multisig wallet policy" md1 header, "m/" prefix on origin_path).
- **Phase O — engraving card legacy migration.** Deleted `format.rs::engraving_card` function + `EngravingMode` enum + 3 byte-exact unit tests. Sole engraving card surface is now `engraving_card_unified` (Phase I, v0.4.1). ~140 lines removed.
- **Cleanup — deleted 5 v0.2 multisig-full integration tests.** `cli_account_flag.rs`, `cli_privacy_preserving.rs`, `cli_bundle_multisig_full.rs` (whole-file deletes); 2 `#[ignore]`-marked test functions inside `cli_self_check.rs` and `cli_bundle_multisig.rs` deleted in-place. These exercised the v0.2 self-multisig pattern (BIP-388 violating, no migration path).

### Deferred to v0.4.3

Three v0.4.2 FOLLOWUPS are deferred to v0.4.3 to keep the v0.4.2 release window scope-safe:

- `cosigner-keyinfo-resolved-slot-merge` — Phase N. Retire `CosignerKeyInfo` into `ResolvedSlot`. Cleanup-only; no user-visible behavior change.
- `verify-bundle-emit-checks-helper-and-full-forensics-rollout` — Phase P. `emit_verify_checks` helper + full ~78-site forensic field population + descriptor-mode 9/3+6N parity (FOLLOWUP `verify-bundle-9-3plus6n-descriptor-mode-parity`).
- `bundle-json-cli-flag-and-dispatch` — Phase Q. `--bundle-json <file>` verify-bundle intake + schema-version dispatch.

### Breaking changes

None at the CLI level — legacy `--phrase` / `--xpub` / `--cosigner` flags continue to accept the same inputs (they're parsed and folded into `Vec<SlotInput>` internally). Some byte-exact stderr text shifted as a consequence of the dispatch consolidation:

- `bundle --phrase X --template wsh-sortedmulti --threshold 2 --cosigner-count 3` (no actual cosigners) now emits `error: --cosigner-count deprecated and inconsistent with slot indices (declared N=3, derived N=1)` (SPEC §6.6 row 5) instead of v0.4.0's BIP-388 row-13 hard-reject. The architectural diagnosis is more accurate (no actual cosigners → declared/derived N mismatch).
- `bundle --descriptor 'wsh(sortedmulti(2,@0/...,@1/...))' --phrase X` (descriptor with no cosigner specs) now emits `error: descriptor has n=2 placeholders but --slot vec covers 1 slots` instead of v0.3's "requires explicit [fp/path] origin annotation" — fires earlier in the pipeline.

Both shifts are tracked by updated integration tests pinning the new byte-exact stderr.

Promoted to v0.5: FOLLOWUP `legacy-cli-flag-deletion` covers eventually deleting `--phrase` / `--xpub` / `--cosigner` flags entirely (option (b) from the v0.4.2 brainstorm). v0.4.2 ships option (a): inputs preserved, dispatch unified.

### Wire-bit-identical guarantee

v0.4.0 / v0.4.1 schema-4 bundles continue to emit byte-identically. v0.2 watch-only multisig fixtures pass byte-identically (text-mode, no JSON envelope). v0.2 self-multisig fixtures remain BIP-388-rejected (no integration coverage now since the 5 ignored tests are deleted).

### Test corpus

240 lib unit tests + integration suites pass (was 246 in v0.4.1; net -6 after cleanup: -3 deleted EngravingMode unit tests, -3 deleted v0.2 multisig-full whole-file integration tests, +5 new K + L tests, ~- 5 net via direct delete).

### Cycle artifacts

- Implementation plan: `design/IMPLEMENTATION_PLAN_v0_4_2_unified_consolidation.md` (r2 APPROVE WITH NITS; nits applied).

### Architect-review history

- v0.4.2 impl plan: 2 in-cycle rounds (r1 BLOCK 2C/3I/2N → r2 APPROVE WITH NITS 0C/0I/1N; nits applied inline before execution).
- Phase K: scope-minimized; per-shape integration tests directly validate.
- Phase L: scope-minimized; descriptor-mode integration tests + fingerprint cross-check.
- Phase M: substantive cleanup (~990 lines deleted); test reconciliation surfaced 6 regressions, all closed via 3 emit_unified UX-preserving fixes (ms1 omitted marker, md1 multisig header, "m/" path prefix) + 3 test updates (BIP-388 row-13 → row-5; new explicit row-13 test; descriptor missing-annotation → slot-count-gap).
- Phase O: trivial deletion; 240 tests pass after.
- Final cross-phase review: pending (this CHANGELOG entry is the gate).

---

## mnemonic-toolkit [0.4.1] — 2026-05-05

### What's new (v0.4.1 schema-4 cutover + multi-source synthesis + foundations for unified card and forensics)

v0.4.1 lands the three v0.4.0 deferrals:

- **`bundle-json-schema-4-cutover` (Phase H, complete).** `Bundle.ms1` and `BundleJson.ms1` migrate from `Option<String>` to `MsField` (= `Vec<String>`). `schema_version` bumps `"3"` → `"4"`. All 5 producers + 4 emit sites updated. SPEC §5.8 dense-with-empty-string-sentinel layout: single-sig watch-only is `[""]`; pure watch-only multisig N=3 is `["", "", ""]`; multi-source full N=3 is `["ms1...", "ms1...", "ms1..."]`; hybrid is mixed. `mode_str` derivation switches to `bundle.any_secret_bearing()`.
- **Multi-source synthesis (Phase H).** `synthesize_unified(slots, template, threshold, network, privacy)` is the new universal synthesis entry handling all five `BundleMode` variants (SingleSigFull / SingleSigWatchOnly / MultisigMultiSource / MultisigWatchOnly / MultisigHybrid). `ResolvedSlot` carries per-slot xpub + fingerprint + path + path_raw + optional entropy.
- **`bundle::run` unified dispatch (Phase H).** When `--slot @N.<subkey>=<value>` is supplied, `bundle::run` routes through `bundle_run_unified`: `expand_legacy_to_slots → validate_slot_set → detect_bundle_mode → resolve_slots → check_resolved_slots_distinctness → synthesize_unified → emit_unified`. Legacy `--phrase` / `--xpub` / `--cosigner` retain v0.3 dispatch (full deprecation deferred to v0.5+).
- **BIP-388 raw-string path normalization (Phase H.6).** `check_key_vector_distinctness` switches to raw-string `(xpub.to_string(), path_raw)` equality per SPEC §4.11.b literal text. `CosignerKeyInfo` and `ResolvedSlot` both carry `path_raw: String`. Legacy descriptor-placeholder paths preserve the parser's canonical `'`-form; `--slot @N.path=<value>` preserves the user's literal byte sequence end-to-end (so `48h/0h` and `48'/0'` compare unequal under raw-string equality on the slot path).
- **Unified engraving card foundation (Phase I, additive).** `BundleInputForCard` struct + `engraving_card_unified` function per SPEC §5.5. Wired into `bundle_run_unified`'s emit_unified path. The 4 legacy `engraving_card(...)` call sites retain v0.3 behavior (full migration deferred to v0.4.2 per FOLLOWUP `engraving-card-unified-legacy-migration`). Card layout: header / threshold / cosigners block / template OR descriptor (truncation at 80 chars) / md1 reference / recovery hint / language+passphrase footer / hardware caveat for tap-multisig.
- **Verify-bundle forensic-field foundation (Phase J, additive).** `VerifyCheck` gains 4 forensic fields (`expected`, `actual`, `diff_byte_offset`, `decode_error`) per SPEC §5.7, with `#[serde(skip_serializing_if = "Option::is_none")]` so JSON envelopes stay clean for "ok"/"skipped" checks. `VerifyCheck::diff_offset(a, b)` helper. Per-cell forensic field POPULATION is wired at one proof-of-shape site (descriptor-mode `ms1_entropy_match` mismatch); full ~78-site rollout deferred to v0.4.2 alongside the `emit_verify_checks` helper refactor (FOLLOWUP `verify-bundle-emit-checks-helper-and-full-forensics-rollout`).
- **`--ms1` CLI repeating-flag migration (Phase J.5).** `VerifyBundleArgs.ms1: Option<String>` → `Vec<String>` with `ArgAction::Append`. Existing single-value invocations continue to work (clap accepts the single occurrence as a 1-element vec). Multi-source schema-4 verification supplies `--ms1` per slot (`--ms1 "" --ms1 <s>` for hybrid-shaped vectors).

### Deferred to v0.4.2

The following SPEC-mandated v0.4 deliverables are deferred to v0.4.2 to preserve v0.4.1 release-window scope-safety. See `design/FOLLOWUPS.md` entries at tier `v0.4.2`:

- `unified-slot-additional-subkey-shapes` — `entropy` / `xprv` / `wif` / partial-xpub-only resolution under `--slot` (v0.4.1 supports `{phrase}` and `{xpub, fingerprint, path}` shapes).
- `unified-slot-descriptor-mode-support` — descriptor mode under unified `--slot` dispatch.
- `bundle-json-cli-flag-and-dispatch` — `--bundle-json <file>` verify-bundle JSON intake + schema-version dispatch (Phase J.4).
- `cosigner-keyinfo-resolved-slot-merge` — retire `CosignerKeyInfo` into `ResolvedSlot`.
- `engraving-card-unified-legacy-migration` — migrate the 4 legacy `engraving_card()` call sites (Phase I migration tail).
- `verify-bundle-emit-checks-helper-and-full-forensics-rollout` — Phase J.2 + J.3 + ~78-site forensic field population.
- `verify-bundle-9-3plus6n-descriptor-mode-parity` — descriptor-mode 9/3+6N parity (depends on the helper).

### Versioning rationale

v0.4.1 is a patch bump (not a 0.5.0 minor bump) under the framing established in v0.4.0's CHANGELOG: v0.4.0 explicitly deferred these breaking changes "to v0.4.1" with full FOLLOWUPS pointers, designating the v0.4 cycle as the breaking-change unit landing in two releases (v0.4.0 ships the BIP-388 enforcement + CLI surface foundation; v0.4.1 completes the schema-4 wire migration + multi-source synthesis + foundations for the unified card and forensics). Consumers reading either v0.4.x release's CHANGELOG are explicitly warned of the schema-4 cutover. Per the repo's pre-1.0 SemVer convention, the breaking changes WOULD justify 0.5.0; the deliberate choice to land them within 0.4.x is an internal-cycle accounting decision documented at v0.4.0.

### Breaking changes

- **`BundleJson.schema_version`** bumps `"3"` → `"4"` for all bundles emitted by v0.4.1. Consumers that assert `schema_version == "3"` will break; update to `"4"` or to schema-aware dispatch.
- **`BundleJson.ms1`** type changes from `string | null` to `array<string>`. Consumers that read `.ms1` as a string break. Migration: read `.ms1` as an array; for single-sig full, use `.ms1[0]`; for watch-only, the array contains an empty-string sentinel `[""]`.
- **`Bundle.ms1`** (Rust API) type changes from `Option<String>` to `Vec<String>`. Direct consumers of the toolkit's library API need to update their pattern matching.
- **`VerifyBundleArgs.ms1`** (CLI flag) accepts `--ms1` multiple times (`Vec<String>`). Single `--ms1 <s>` invocations continue to work as 1-element vec. **Note for multi-slot verification:** v0.4.1's verify-bundle path compares only the FIRST `--ms1` value against the bundle's slot 0; full per-slot multi-source verification (all elements of `--ms1` checked against all slots) is deferred to v0.4.2 alongside `--bundle-json` intake (FOLLOWUP `bundle-json-cli-flag-and-dispatch`).
- **BIP-388 raw-string path equality** for `--slot @N.path=` paths preserves the user's literal byte sequence; `48h/0h` and `48'/0'` are now treated as distinct paths under the slot-driven path. Legacy descriptor paths continue to use the parser's canonical form.

### Wire-bit-identical guarantee

v0.4.0 v0.2/v0.3 single-sig + watch-only multisig fixtures continue to pass byte-identically (text-mode output for these cases is unchanged; only the JSON envelope shape changes). The 5 v0.2 self-multisig integration tests remain `#[ignore]`d per BIP-388 hard-reject (introduced in v0.4.0).

### Test corpus

246 lib unit tests + integration suites pass (was 227 in v0.4.0; +19). New tests added in v0.4.1:
- 2 BIP-388 raw-string distinctness unit tests.
- 7 `synthesize_unified` shape tests (each BundleMode + threshold-out-of-range + schema-version pin).
- 4 unified `--slot` CLI integration tests (happy path + missing-template/descriptor + unsupported-subkey-shape + row-6 conflict).
- 6 unified engraving card unit tests (single-sig full / watch-only / multisig / privacy-preserving / descriptor truncation / tap caveat).
- 4 VerifyCheck forensic field unit tests.

### Cycle artifacts

- Implementation plan: `design/IMPLEMENTATION_PLAN_v0_4_1_cutover.md` (r2 APPROVE WITH NITS; nits applied).
- Per-phase reviews: `design/agent-reports/phase-H-schema-4-cutover-review-r1.md` (r1 BLOCK 0C/2I/1L → r2 APPROVE 0C/0I/0L).

### Architect-review history

- v0.4.1 impl plan: 2 in-cycle rounds (r1 BLOCK 3C/2I → r2 APPROVE WITH NITS 0C/0I/2N + nits applied inline).
- Phase H: 2 rounds (r1 BLOCK 0C/2I/1L → r2 APPROVE 0C/0I/0L).
- Phase I: scope-minimized to additive only; format.rs unit tests (6) directly cover the new function; per-phase review skipped.
- Phase J: scope-minimized to additive only (J.1 + J.5 + one J.7 proof-of-shape); format.rs unit tests (4) directly cover the new VerifyCheck behavior; per-phase review skipped.
- Final cross-phase review: pending (this CHANGELOG entry).

---

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
