# mnemonic-toolkit Rust API

This chapter is the reference for the `mnemonic-toolkit`\index{mnemonic-toolkit} crate at v0.8.0\index{mnemonic-toolkit v0.8.0} (HEAD `4210b91c` in `bg002h/mnemonic-toolkit`). Unlike its three sibling chapters (§V.1, §V.2, §V.3), this chapter does **not** enumerate a public library API: `mnemonic-toolkit` v0.8.0 is a **binary-only crate** with no `[lib]` target and no `src/lib.rs`. The `[[bin]] name = "mnemonic"` declaration at `crates/mnemonic-toolkit/Cargo.toml:15-18` is the crate's only build target; every `mod foo;` in `src/main.rs` is private at crate root. External crates cannot `use mnemonic_toolkit::*` at v0.8.0. Per SPEC §4.2.5 Part V scope, the CLI surface (clap-derived `*Args` structs, `cmd::*::run` dispatch) is **out of scope** for this chapter and lives in the end-user manual instead. What Part V documents here is the consumer contract that an external program **can** target today without a library facade: the JSON envelope schema emitted by `mnemonic bundle` / `mnemonic verify-bundle` on stdout, the engraving-card layout emitted on stderr, the orchestration modules that produce them, and the `ToolkitError` taxonomy mapped to exit codes.

## V.4.1 Crate purpose

`mnemonic-toolkit`\index{mnemonic\_toolkit (crate)} is the orchestration crate of the m-format-star: it composes the three sibling codecs (`ms-codec`, `mk-codec`, `md-codec`) into end-to-end bundle creation (`bundle`), bundle verification (`verify-bundle`), seed-material conversion (`convert`), watch-only wallet export (`export-wallet`), and BIP-85 child-derivation (`derive-child`). The crate owns descriptor parsing (BIP-388 placeholder lexing plus `miniscript::Descriptor` walk), distinct-key enforcement, network/template/path-family selection, BIP-39 / BIP-38 / SLIP-0132 input plumbing, and the unified slot-input pipeline (`@N.subkey=value`).

The crate has **no library target**\index{binary-only crate} at v0.8.0. The `pub` items in `src/*.rs` are reachable only within this binary or its `tests/` integration suite — they form a candidate library facade that v0.9+ may extract, but no external crate compiles against them today. Sibling codecs are git-pinned (§V.4.8) rather than published to crates.io; downstream consumers integrate by spawning the `mnemonic` binary and consuming its JSON envelopes (§V.4.5) or its engraving cards (§V.4.6).

## V.4.2 Feature flags

**None.** The `[features]` table is **absent** from `crates/mnemonic-toolkit/Cargo.toml`; `grep -n '\[features\]'` on the manifest returns no match, and there are no `#[cfg(feature = ...)]` attributes anywhere under `src/`. All code is unconditionally compiled. The crate ships no optional `serde` impl, no optional vector-generator binary, and no optional `derive` tier. Test-only items use `#[cfg(test)]` only.

## V.4.3 Crate structure (orchestration modules)

What follows is a module-by-module walk of the binary's internal architecture as a documented surface. **None of these items are external API surface at v0.8.0** — every module is declared `mod foo;` (no `pub`) in `src/main.rs`. `pub` and `pub(crate)` markings on individual items reflect intra-crate visibility only, not crates.io-facing contract.

### V.4.3.1 `error`\index{error (module)} (`src/error.rs`)

Central error enum + `Result` alias. SPEC §6.1–§6.4 exit-code mapping lives in `ToolkitError::exit_code` (`error.rs:223`); JSON `kind` discriminant in `ToolkitError::kind` (`error.rs:254`); friendly-message dispatch in `ToolkitError::message` (`error.rs:288`); JSON `details` in `ToolkitError::details` (`error.rs:364`). Variant table in §V.4.4.

| Item | Visibility | Notes |
|---|---|---|
| `ToolkitError`\index{ToolkitError} | `pub` | `#[non_exhaustive]`; 26 variants; `error.rs:10` |
| `BitcoinErrorKind`\index{BitcoinErrorKind} | `pub` | bitcoin-crate-sourced error wrapper; `error.rs:119` |
| `Result<T>` | `pub` | `std::result::Result<T, ToolkitError>`; `error.rs:453` |
| `From<bip39::Error>` | impl | auto-lift via `?` |
| `From<bitcoin::bip32::Error>` | impl | wraps as `Bitcoin(Bip32(_))` |
| `From<ms_codec::Error>` | impl | folds `ReservedTagNotEmittedInV01` to `FutureFormat` |
| `From<mk_codec::Error>` | impl | folds `UnsupportedVersion` to `FutureFormat` |
| `From<md_codec::Error>` | impl | folds `UnsupportedVersion` to `FutureFormat` |

### V.4.3.2 `format`\index{format (module)} (`src/format.rs`)

JSON envelope structs (§V.4.5), the unified engraving-card renderer (§V.4.6), and 5-char chunking helpers. The serde-derived structs in this module are the **canonical source of truth** for the JSON schema: every field name and `Option` semantic is determined here.

| Item | Visibility | Notes |
|---|---|---|
| `MsField` (= `Vec<String>`) | `pub` type alias | SPEC §5.8 dense layout; `format.rs:54` |
| `MkField` | `pub enum` | `#[serde(untagged)]`; `Single` or `Multi`; `format.rs:66` |
| `CosignerEntry` | `pub struct` | `format.rs:94`; `Serialize` |
| `MultisigInfo` | `pub struct` | `format.rs:104`; `Serialize` |
| `BundleJson` | `pub struct` | `format.rs:120`; `Serialize` |
| `VerifyBundleJson` | `pub struct` | `format.rs:149`; `Serialize` |
| `VerifyCheck` | `pub struct` | `format.rs:166`; `Serialize, Clone` |
| `BundleInputForCard`\index{BundleInputForCard} | `pub struct` | engraving-card input (not serde); `format.rs:223` |
| `TemplateOrDescriptor`\index{TemplateOrDescriptor} | `pub enum` | `format.rs:236` |
| `SlotCardBlock`\index{SlotCardBlock} | `pub struct` | per-slot card block; `format.rs:242` |
| `chunk_5char`\index{chunk\_5char} | `pub fn` | 5-char groups, 10/line; `format.rs:10` |
| `chunk_md1`\index{chunk\_md1} | `pub fn` | delegates to `md_codec::encode::render_codex32_grouped(s, 5)`; `format.rs:38` |
| `engraving_card_unified`\index{engraving\_card\_unified} | `pub fn` | SPEC §5.5 sole card surface; `format.rs:259` |

### V.4.3.3 `synthesize`\index{synthesize (module)} (`src/synthesize.rs`)

Bundle construction. The current unified entrypoint is `synthesize_unified` (`synthesize.rs:593`), which dispatches across the five `BundleMode` cases (§V.4.3.7). Legacy entry points (`synthesize_full`, `synthesize_watch_only`, `synthesize_multisig_full`, `synthesize_multisig_watch_only`, `synthesize_descriptor`) are retained behind `#[allow(dead_code)]` for v0.9+ library extraction; the CLI no longer calls them directly. The `Bundle` struct carries the three sibling outputs (`ms1: MsField`, `mk1: MkField`, `md1: Vec<String>`). `ResolvedSlot` carries the per-slot binding produced by the unified slot pipeline; SPEC §4.11.b distinctness uses its **typed** `bitcoin::bip32::DerivationPath` field (which folds `h ↔ '`), not raw-string equality. `CosignerKeyInfo` is a `pub type` alias for `ResolvedSlot` retained for legacy callers.

| Item | Visibility | Notes |
|---|---|---|
| `Bundle`\index{Bundle (toolkit)} | `pub struct` | three-codec output; `synthesize.rs:20` |
| `Bundle::any_secret_bearing`\index{Bundle::any\_secret\_bearing} | `pub fn` | true iff any slot carries entropy; `synthesize.rs:33` |
| `ResolvedSlot`\index{ResolvedSlot} | `pub struct` | `xpub`, `fingerprint`, typed `path`, `path_raw`, `entropy`; `synthesize.rs:569` |
| `ResolvedSlot::is_secret_bearing`\index{ResolvedSlot::is\_secret\_bearing} | `pub fn` | `entropy.is_some()`; `synthesize.rs:579` |
| `CosignerKeyInfo`\index{CosignerKeyInfo} | `pub type` alias | `= ResolvedSlot` (legacy); `synthesize.rs:190` |
| `xpub_to_65`\index{xpub\_to\_65} | `pub fn` | SPEC §4.6.1 `chain_code \|\| pubkey` form; `synthesize.rs:69` |
| `build_descriptor`\index{build\_descriptor} | `pub fn` | template → `md_codec::Descriptor`; `synthesize.rs:80` |
| `synthesize_unified`\index{synthesize\_unified} | `pub fn` | the unified entrypoint; `synthesize.rs:593` |
| `synthesize_full` / `synthesize_watch_only` / `synthesize_multisig_*` / `synthesize_descriptor` | `pub fn` | legacy variants; all `#[allow(dead_code)]` |

### V.4.3.4 `parse_descriptor`\index{parse\_descriptor (module)} (`src/parse_descriptor.rs`)

The descriptor pipeline: BIP-388 placeholder lexing → key-vector resolution → synthetic-xpub substitution for parser conformance → `miniscript::Descriptor` walk to a `md_codec::Descriptor` AST → distinct-key enforcement. The **sole `pub` function on the BIP-388 distinct-key path** is `check_key_vector_distinctness` (`parse_descriptor.rs:1104`), which compares typed `DerivationPath == DerivationPath` at `parse_descriptor.rs:1108` (folding `h ↔ '` per `bitcoin::bip32`). A second distinct-key check at the CLI layer (`cmd::bundle::check_resolved_slots_distinctness`) is `pub(crate)` and uses raw-string equality — that mirror is **out of scope** for Part V (§V.4.8 flags the doc-comment drift on the CLI side).

| Item | Visibility | Notes |
|---|---|---|
| `ScriptCtx`\index{ScriptCtx} | `pub enum` | `SingleSig` or `MultiSig`; `parse_descriptor.rs:32` |
| `PlaceholderOccurrence`\index{PlaceholderOccurrence} | `pub struct` | `i`, `fingerprint_anno`, `origin_path_anno`, `multipath_alts`, `wildcard_hardened`; `parse_descriptor.rs:50` |
| `ResolvedPlaceholders`\index{ResolvedPlaceholders} | `pub struct` | `n`, `path_decl`, `fingerprint_annos`, `use_site_path`, `use_site_path_overrides`; `parse_descriptor.rs:145` |
| `ParsedKey` / `ParsedFingerprint`\index{ParsedKey}\index{ParsedFingerprint} | `pub struct` | `(i, payload [u8;65])` and `(i, fp [u8;4])`; `parse_descriptor.rs:673, 679` |
| `DescriptorMode`\index{DescriptorMode} | `pub enum` | `SingleSig` or `MultiSig`; `parse_descriptor.rs:751` |
| `DescriptorBinding`\index{DescriptorBinding} | `pub struct` | resolved keys + fingerprints + cosigners; `parse_descriptor.rs:790` |
| `lex_placeholders`\index{lex\_placeholders} | `pub fn` | placeholder enumeration; `parse_descriptor.rs:60` |
| `resolve_placeholders`\index{resolve\_placeholders} | `pub fn` | reconcile annotations across occurrences; `parse_descriptor.rs:156` |
| `substitute_synthetic`\index{substitute\_synthetic} | `pub fn` | replace `@i` with deterministic synthetic xpubs for parser; `parse_descriptor.rs:263` |
| `walk_root`\index{walk\_root} | `pub fn` | `miniscript::Descriptor` → `md_codec::tree::Node`; `parse_descriptor.rs:353` |
| `parse_descriptor`\index{parse\_descriptor (function)} | `pub fn` | top-level pipeline driver; `parse_descriptor.rs:687` |
| `synthetic_xpub_for`\index{synthetic\_xpub\_for} | `pub fn` | deterministic synthetic xpub for `@i`; seed prefix `b"toolkit-v0.3"`; `parse_descriptor.rs:769` |
| `bind_descriptor_keys`\index{bind\_descriptor\_keys} | `pub fn` | bind resolved placeholders to entropy + cosigners; `parse_descriptor.rs:815` |
| `check_key_vector_distinctness`\index{check\_key\_vector\_distinctness} | `pub fn` | SPEC §4.11.b typed-`DerivationPath` distinctness; `parse_descriptor.rs:1104` |

### V.4.3.5 `derive`\index{derive (module)} (`src/derive.rs`)

BIP-32 derivation from a BIP-39 mnemonic to a template-relative account xpub + xpriv + fingerprint. The `DerivedAccount`\index{DerivedAccount} struct collects all five outputs (`entropy: Vec<u8>`, `master_fingerprint: Fingerprint`, `account_xpub: Xpub`, `account_xpriv: Xpriv`, `account_path: DerivationPath`); `derive_full`\index{derive\_full} (`derive.rs:22`) is the single entry point and is parameterised by `(phrase, passphrase, CliLanguage, CliNetwork, CliTemplate, account)`.

### V.4.3.6 `parse`\index{parse (module)} (`src/parse.rs`)

Input helpers: `read_phrase_input` (stdin or argv), `parse_master_fingerprint`, `parse_cosigner_spec` (legacy `--cosigner=xpub:fp:path` form), `parse_cosigners_file` (legacy JSON form), `check_no_concurrent_stdin` (single-stdin invariant). Also exports `CosignerSpec` (struct) and `MultisigPathFamily` (enum, BIP-48 or BIP-87) for the legacy multisig path.

| Item | Visibility | Notes |
|---|---|---|
| `read_phrase_input`\index{read\_phrase\_input} | `pub fn` | argv-or-stdin read; `parse.rs:17` |
| `parse_master_fingerprint`\index{parse\_master\_fingerprint} | `pub fn` | 4-byte hex parse to `Fingerprint`; `parse.rs:38` |
| `parse_cosigner_spec`\index{parse\_cosigner\_spec} | `pub fn` | legacy `xpub:fp:path` parse; `parse.rs:111` |
| `parse_cosigners_file`\index{parse\_cosigners\_file} | `pub fn` | legacy `--cosigners-file` JSON parse; `parse.rs:173` |
| `check_no_concurrent_stdin`\index{check\_no\_concurrent\_stdin} | `pub fn` | reject simultaneous phrase + passphrase stdin reads; `parse.rs:224` |
| `CosignerSpec`\index{CosignerSpec} | `pub struct` | `xpub`, `master_fingerprint`, `path: Option<DerivationPath>`; `parse.rs:53` |
| `MultisigPathFamily`\index{MultisigPathFamily} | `pub enum` | `Bip48` or `Bip87` (Default); `parse.rs:63` |

### V.4.3.7 `template`\index{template (module)} (`src/template.rs`)

Template enum + derivation-path math. `CliTemplate` enumerates the ten supported templates (`Bip44`, `Bip49`, `Bip84`, `Bip86` single-sig; `WshMulti`, `WshSortedMulti`, `ShWshMulti`, `ShWshSortedMulti`, `TrMultiA`, `TrSortedMultiA` multisig). Methods produce the BIP-32 origin path (`origin_path_str`, `derivation_path`), the md-codec wire-form origin path (`md_origin_path`), the md-codec wrapper `Node` (`wrapper_node`), and the BIP-48 script-type discriminant (`bip48_script_type`).

| Method | Signature | Notes |
|---|---|---|
| `is_multisig`\index{CliTemplate::is\_multisig} | `fn (&self) -> bool` | matches `WshMulti`, `WshSortedMulti`, `ShWshMulti`, `ShWshSortedMulti`, `TrMultiA`, `TrSortedMultiA`; `template.rs:46` |
| `origin_path_str`\index{CliTemplate::origin\_path\_str} | `fn (&self, CliNetwork, u32) -> String` | BIP-44 `m/{purpose}'/{coin}'/{account}'` (single-sig) or BIP-48 `m/48'/.../{script_type}'`; `template.rs:61` |
| `derivation_path`\index{CliTemplate::derivation\_path} | `fn (&self, CliNetwork, u32) -> DerivationPath` | typed form of `origin_path_str`; `template.rs:75` |
| `md_origin_path`\index{CliTemplate::md\_origin\_path} | `fn (&self, CliNetwork, u32) -> md_codec::origin_path::OriginPath` | md1 wire-form path; `template.rs:82` |
| `wrapper_node`\index{CliTemplate::wrapper\_node} | `fn (&self, u8, usize) -> md_codec::tree::Node` | md1 wrapper for `(K, N)`; `template.rs:111` |
| `bip48_script_type`\index{CliTemplate::bip48\_script\_type} | `fn (&self) -> Option<u32>` | `Some(1)` for `WshMulti` / `WshSortedMulti`, `Some(2)` for `TrMultiA` / `TrSortedMultiA`, `None` otherwise; `template.rs:219` |
| `human_name`\index{CliTemplate::human\_name} | `fn (&self) -> &'static str` | human-readable; `template.rs:228` |

### V.4.3.8 Miscellaneous support modules

| Module | Items | Role |
|---|---|---|
| `language` (`src/language.rs`) | `CliLanguage`\index{CliLanguage} (enum) | 10 BIP-39 wordlists; `human_name`; `From<CliLanguage> for bip39::Language` |
| `network` (`src/network.rs`) | `CliNetwork`\index{CliNetwork} (enum) | mainnet / testnet / signet / regtest; `coin_type`, `network_kind`, `known_hrp`, `human_name` |
| `slip0132` (`src/slip0132.rs`) | `XpubPrefix`\index{XpubPrefix} (enum), `parse_xpub_prefix_arg`\index{parse\_xpub\_prefix\_arg} | xpub / ypub / Ypub / zpub / Zpub prefix selector |
| `slot_input` (`src/slot_input.rs`) | `SlotSubkey`\index{SlotSubkey}, `SlotInput`\index{SlotInput}, `ParseError`\index{ParseError (slot)}, `parse_slot_input`\index{parse\_slot\_input}, `validate_slot_set`\index{validate\_slot\_set} | unified `@N.subkey=value` parser (SPEC §6.6) |
| `bundle_unified` (`src/bundle_unified.rs`) | `BundleMode`\index{BundleMode} (enum), `detect_bundle_mode`\index{detect\_bundle\_mode}, `pre_check_threshold`\index{pre\_check\_threshold}, `pre_check_template\_n`\index{pre\_check\_template\_n} | five-way mode dispatch over `&[SlotInput]` |
| `friendly` (`src/friendly.rs`) | `friendly_bip39`, `friendly_bitcoin`, `friendly_ms_codec`, `friendly_mk_codec`, `friendly_md_codec` | human-readable error messages |
| `wallet_export` (`src/wallet_export/mod.rs` + 9 `pub(crate)` sub-modules) | `REFUSAL_SECRET_INPUT` (const), `format_stub_message`, `taproot_multisig_unsupported_message`, `TaprootInternalKey`\index{TaprootInternalKey} (enum), `ELECTRUM_SEED_VERSION_PIN`\index{ELECTRUM\_SEED\_VERSION\_PIN} (`wallet_export/electrum.rs:37`, `pub const`); `WalletFormatEmitter`\index{WalletFormatEmitter} (trait), `EmitInputs`\index{EmitInputs} (struct), `MissingField`\index{MissingField} (enum), `TimestampArg`\index{TimestampArg} (enum), and `build_missing_fields_refusal` are `pub(crate)` (not part of the `pub` surface) | watch-only refusal text + taproot internal-key selector + pinned Electrum seed-version. Eight vendor-emitter sub-modules — `bip388`\index{bip388 (format alias)}, `bitcoin_core`\index{Bitcoin Core (wallet-export format)}, `coldcard`\index{Coldcard (wallet-export format)}, `electrum`\index{Electrum (wallet-export format)}, `green`\index{Blockstream Green (wallet-export format)}, `jade`\index{Jade (wallet-export format)}, `sparrow`\index{Sparrow (wallet-export format)}, `specter`\index{Specter (wallet-export format)} — plus `pipeline` are `pub(crate)` orchestration plumbing. Each vendor emitter implements `WalletFormatEmitter`; the eight output shapes + the format × descriptor-shape compatibility matrix are documented in §V.4.5.9 and §V.4.5.10 |
| `bip85` (`src/bip85.rs`) | (all `pub(crate)`) | BIP-85 child derivation |
| `electrum` (`src/electrum.rs`) | (all `pub(crate)`) | Electrum-seed plumbing |
| `wordlists` (`src/wordlists/mod.rs`) | (all `pub(crate)`) | wordlist tables |
| `derive_slot` (`src/derive_slot.rs`) | (all `pub(crate)`) | per-slot derivation helper |

### V.4.3.9 `cmd::*` — OUT OF SCOPE

The submodules under `src/cmd/` (`bundle`, `verify_bundle`, `convert`, `export_wallet`, `derive_child`) hold the clap-derive `*Args` structs, their `run` dispatch functions, and CLI-only helpers (`SuppliedCards`, `emit_verify_checks`, `NodeType`, `FromInput`, `ScriptType`, `CliExportFormat`, `TimestampArgValue`, etc.). Per SPEC §4.2.5 these belong to the end-user manual (`docs/manual/src/40-cli-reference/`), not Part V. Part V cross-references them only when a `VerifyCheck.name` string literal (such as `md1_xpub_match`) is the load-bearing contract — and even then the contract is the JSON envelope, not the dispatch function.

## V.4.4 ToolkitError taxonomy

`ToolkitError`\index{ToolkitError} is `#[non_exhaustive]` (`error.rs:10`); the 26-row table below covers every variant at HEAD. The v0.8.1 vendor-emitter expansion (8 `WalletFormatEmitter` impls — §V.4.5.9) wired `ExportWalletMissingFields` to live construction sites: `Sparrow` returns it when `--threshold` is absent on a multisig template (`wallet_export/sparrow.rs:31-48`); `Specter` returns it when `--wallet-name` is absent (`wallet_export/specter.rs:31-38`); the dispatcher routes the variant through `build_missing_fields_refusal` (`wallet_export/mod.rs:279`) for byte-exact stderr. The variant is no longer reserved at HEAD. The `Exit` column maps to `ToolkitError::exit_code` (`error.rs:223`) per SPEC §6.1; the `kind()` column is the stable JSON discriminant emitted into `details` blocks (SPEC §6.4); the `Emitted by` column lists representative call sites (not exhaustive).

| Variant | Exit | `kind()` | Display summary | Emitted by |
|---|---|---|---|---|
| `BadInput(String)` | 1 | `BadInput` | generic exit-1 user-input failure | `parse::read_phrase_input`, `parse::parse_master_fingerprint`, `parse::check_no_concurrent_stdin` |
| `Bip39(bip39::Error)` | 1 | `Bip39` | BIP-39 mnemonic parse or validate failure | `derive_full`, `bind_full_mode` |
| `Bitcoin(BitcoinErrorKind)` | 1 | `Bitcoin` | bitcoin-crate wrapper | `parse::parse_master_fingerprint`, `synthesize::derive_xpub_at_path` |
| `MsCodec(ms_codec::Error)` | 1 or 2 | `MsCodec` | ms1 codec error | `synthesize_*`, `emit_verify_checks` |
| `MkCodec(mk_codec::Error)` | 1 or 2 | `MkCodec` | mk1 codec error | `synthesize_*` |
| `MdCodec(md_codec::Error)` | 1, 2 or 3 | `MdCodec` | md1 codec error | `synthesize_*` |
| `ModeViolation { mode, flag, message }` | 2 | `ModeViolation` | SPEC §5.5 flag-vs-mode violation | `cmd::bundle::run`, `parse_descriptor::bind_descriptor_keys` |
| `BundleMismatch { card, message }` | 4 | `BundleMismatch` | SPEC §6.1 verify-bundle string mismatch | `cmd::verify_bundle::run` |
| `NetworkMismatch { xpub_network, expected }` | 2 | `NetworkMismatch` | SPEC §4.3 xpub network mismatch | `synthesize_multisig_watch_only`, `synthesize_unified` |
| `FutureFormat { source, detail }` | 3 | `FutureFormat` | reserved-not-emitted-in-this-version | `From<ms_codec::Error>` / `mk_codec` / `md_codec` lifts |
| `MultisigConfig { message }` | 1 | `MultisigConfig` | SPEC §6.2 threshold or N-range | `synthesize_multisig_*`, `synthesize_unified` |
| `CosignerSpec { cosigner_idx, message }` | 1 | `CosignerSpec` | SPEC §6.2 cosigner spec parse | `parse::parse_cosigner_spec`, depth-check in `synthesize_*` |
| `CosignersFile { message }` | 1 | `CosignersFile` | SPEC §6.2 cosigners-file parse | `parse::parse_cosigners_file` |
| `DescriptorParse(String)` | 2 | `DescriptorParse` | SPEC §6.7 descriptor content parse | `lex_placeholders`, `resolve_placeholders`, `parse_descriptor`, `synthesize_descriptor` |
| `DescriptorReparseFailed { detail }` | 4 | `DescriptorReparseFailed` | SPEC §5.7 verify-bundle re-parse | `cmd::verify_bundle::run` |
| `Bip388Distinctness { i, j }` | 2 | `Bip388Distinctness` | SPEC §4.11.b distinct-key at bundle | `check_key_vector_distinctness`, `cmd::bundle::check_resolved_slots_distinctness` |
| `Bip388VerifyDistinctness` | 4 | `Bip388VerifyDistinctness` | SPEC §4.11.c distinct-key at verify | `cmd::verify_bundle::run` |
| `SlotInputViolation { kind, message }` | 2 | `SlotInputViolation` | SPEC §6.6 unified slot input | `slot_input::validate_slot_set`, `bundle_unified::detect_bundle_mode` |
| `ConvertRefusal(String)` | 2 | `ConvertRefusal` | SPEC\_convert §3 or §4 refusal | `cmd::convert::refusal_*` family |
| `ExportWalletSecretInput` | 2 | `ExportWalletSecretInput` | SPEC\_export\_wallet §3 watch-only refusal | `wallet_export::validate_watch_only` |
| `ExportWalletFormatStub(&'static str)` | 2 | `ExportWalletFormatStub` | SPEC\_export\_wallet §7 sparrow or specter stub | `cmd::export_wallet::run` |
| `ExportWalletTaprootMultisigUnsupported(&'static str)` | 2 | `ExportWalletTaprootMultisigUnsupported` | SPEC\_export\_wallet §4 (unreachable post-v0.8 NUMS) | (variant retained for back-compat) |
| `ExportWalletMissingFields { format, missing }` | 2 | `ExportWalletMissingFields` | SPEC\_export\_wallet §4 missing-fields refusal | `SparrowEmitter::collect_missing`, `SpecterEmitter::collect_missing` (re-emitted by `cmd::export_wallet::run` at `cmd/export_wallet.rs:392-397`) |
| `DeriveChildUnsupportedApp` | 2 | `DeriveChildUnsupportedApp` | SPEC\_derive\_child §7 rsa or rsa-gpg deferred | `cmd::derive_child::run` |
| `DeriveChildLengthOutOfRange { app, length, valid_text }` | 2 | `DeriveChildLengthOutOfRange` | SPEC\_derive\_child §7 length range | `cmd::derive_child::run` |
| `DeriveChildLengthNotApplicable` | 2 | `DeriveChildLengthNotApplicable` | SPEC\_derive\_child §4 / §7 length not applicable | `cmd::derive_child::run` |

(Variant count = 26; row count = 26.) The `From` impls for `ms_codec::Error`, `mk_codec::Error`, and `md_codec::Error` selectively fold version-future variants (`ReservedTagNotEmittedInV01`, `UnsupportedVersion`) into `FutureFormat` so that callers see exit code 3 on any forward-incompatible card. Every other sibling-codec variant passes through wrapped as `MsCodec(_)` / `MkCodec(_)` / `MdCodec(_)` and inherits the sibling's own exit-code dispatcher (`ms_codec_exit_code` / `mk_codec_exit_code` / `md_codec_exit_code`).

## V.4.5 JSON envelope schema

The `mnemonic bundle` and `mnemonic verify-bundle` subcommands emit a serde-serialised JSON envelope on stdout. The structs are defined in `crates/mnemonic-toolkit/src/format.rs` and are the canonical schema source: field names, field order, and `Option` semantics on the JSON wire are whatever serde produces from those structs at HEAD.

**Schema version.** `BundleJson.schema_version` and `VerifyBundleJson.schema_version` are both `&'static str` literals fixed at `"4"` at every construction site at HEAD (`synthesize.rs:1296`, `cmd/bundle.rs:572`, `cmd/verify_bundle.rs:182`, `cmd/verify_bundle.rs:498`). The `format.rs:114` module-level doc-comment still reads `v0.2: schema_version "2"`; that doc-comment is **stale** and persists at HEAD (§V.4.8). External consumers MUST pin against the constructor-site literal `"4"`, not the module-level doc-comment.

**Optionality.** Two distinct serde policies coexist:

- `BundleJson` `Option<T>` fields (`template`, `descriptor`, `origin_path`, `origin_paths`, `master_fingerprint`, `multisig`) carry no `skip_serializing_if` attribute and therefore serialise as JSON `null` when `None`. JSON readers MUST tolerate explicit nulls.
- `VerifyCheck` forensic fields (`expected`, `actual`, `diff_byte_offset`, `decode_error`) carry `#[serde(skip_serializing_if = "Option::is_none")]` (`format.rs:171,174,177,181`) and are therefore **omitted entirely** from passing checks. They appear only on `passed: false` rows (string-mismatch forensics) or as `decode_error: Some("skipped: <reason>")` on skipped checks. (Note: a transient earlier draft cited `#[serde(skip)]` here; at HEAD the four attributes are uniformly `skip_serializing_if = "Option::is_none"`, and the JSON envelope follows that semantic.)

### V.4.5.1 `BundleJson`\index{BundleJson}

```json
{
  "schema_version": "4",
  "mode": "full",
  "network": "mainnet",
  "template": "bip84",
  "descriptor": null,
  "account": 0,
  "origin_path": "m/84'/0'/0'",
  "origin_paths": null,
  "master_fingerprint": "5436d724",
  "ms1": ["ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f"],
  "mk1": ["mk1qprsqhpqqsq3c..."],
  "md1": ["md1zsxdspqqqpm6jzzq..."],
  "multisig": null,
  "privacy_preserving": false
}
```

| Field | Type | Semantics |
|---|---|---|
| `schema_version` | `&'static str` | always `"4"` at HEAD |
| `mode` | `&'static str` | `"full"` or `"watch-only"` |
| `network` | `&'static str` | `"mainnet"`, `"testnet"`, `"signet"`, `"regtest"` |
| `template` | `Option<&'static str>` | `Some` in template mode; `null` in descriptor mode |
| `descriptor` | `Option<String>` | user-supplied descriptor in descriptor mode; `null` otherwise |
| `account` | `u32` | BIP-32 account index |
| `origin_path` | `Option<String>` | single-sig or shared-path multisig; `null` for divergent-path |
| `origin_paths` | `Option<Vec<String>>` | divergent-path multisig; `null` otherwise |
| `master_fingerprint` | `Option<String>` | `null` for multisig or `--privacy-preserving` |
| `ms1` | `Vec<String>` | dense schema-4 layout; `""` watch-only sentinel |
| `mk1` | `MkField` | bare array (single-sig) or array-of-arrays (multi-source) |
| `md1` | `Vec<String>` | one or more chunked md1 strings |
| `multisig` | `Option<MultisigInfo>` | `null` for single-sig |
| `privacy_preserving` | `bool` | strips fingerprints when `true` |

(Source: `format.rs:120-145`; `#[derive(Serialize)]`. Field order is part of the schema.)

### V.4.5.2 `VerifyBundleJson`\index{VerifyBundleJson}

```json
{
  "schema_version": "4",
  "result": "ok",
  "checks": [
    {"name": "ms1_decode", "passed": true, "detail": "ms1[0] decoded as 16-byte entropy"},
    {"name": "mk1_decode", "passed": true, "detail": "mk1[0] decoded; xpub matches descriptor"},
    {"name": "md1_decode", "passed": true, "detail": "md1 reassembled and decoded"},
    {"name": "md1_xpub_match", "passed": true, "detail": "md1 xpub matches expected"}
  ]
}
```

| Field | Type | Semantics |
|---|---|---|
| `schema_version` | `&'static str` | always `"4"` at HEAD |
| `result` | `&'static str` | `"ok"` or `"mismatch"` |
| `checks` | `Vec<VerifyCheck>` | per-check rows (SPEC §5.7) |

(Source: `format.rs:149-153`.)

### V.4.5.3 `VerifyCheck`\index{VerifyCheck}

```json
{
  "name": "ms1_decode",
  "passed": false,
  "detail": "ms1[0] decoded as 20-byte entropy; expected 16",
  "expected": "ms10entrsqqq...cj9s",
  "actual": "ms10entrsqyq...rxz4",
  "diff_byte_offset": 11,
  "decode_error": null
}
```

| Field | Type | Semantics |
|---|---|---|
| `name` | `String` | check identifier; e.g. `ms1_decode`, `ms1_entropy_match`, `mk1_decode`, `mk1_xpub_match`, `mk1_fingerprint_match`, `mk1_path_match`, `md1_decode`, `md1_wallet_policy`, `md1_xpub_match` |
| `passed` | `bool` | check outcome |
| `detail` | `String` | one-line human summary |
| `expected` | `Option<String>` | string-mismatch expected (omitted if `None`) |
| `actual` | `Option<String>` | string-mismatch actual (omitted if `None`) |
| `diff_byte_offset` | `Option<usize>` | first differing UTF-8 byte (omitted if `None`) |
| `decode_error` | `Option<String>` | decode-failure text or `"skipped: <reason>"` (omitted if `None`) |

(Source: `format.rs:166-183`. All four forensic fields use `#[serde(skip_serializing_if = "Option::is_none")]`.)

### V.4.5.4 `MultisigInfo`\index{MultisigInfo}

```json
{
  "template": "wsh-sortedmulti",
  "threshold": 3,
  "cosigner_count": 5,
  "path_family": "bip48",
  "cosigners": [ /* CosignerEntry[] */ ]
}
```

| Field | Type | Semantics |
|---|---|---|
| `template` | `&'static str` | template name (multisig only) |
| `threshold` | `u8` | `K` in `K-of-N` |
| `cosigner_count` | `usize` | `N` |
| `path_family` | `&'static str` | `"bip48"` or `"bip87"` |
| `cosigners` | `Vec<CosignerEntry>` | per-cosigner descriptor |

(Source: `format.rs:104-111`.)

### V.4.5.5 `CosignerEntry`\index{CosignerEntry}

```json
{
  "index": 0,
  "master_fingerprint": "5436d724",
  "origin_path": "m/48'/0'/0'/2'",
  "xpub": "xpub6E..."
}
```

| Field | Type | Semantics |
|---|---|---|
| `index` | `usize` | slot index `@N` |
| `master_fingerprint` | `Option<String>` | `null` under `--privacy-preserving` |
| `origin_path` | `String` | BIP-32 origin path for this cosigner |
| `xpub` | `String` | account xpub |

(Source: `format.rs:94-100`.)

### V.4.5.6 `MkField`\index{MkField}

`MkField` is `#[serde(untagged)]` (`format.rs:66-92`), so the JSON wire is either a bare array (single-sig) or an array of arrays (multi-source multisig) — there is **no** discriminator key:

```json
"mk1": ["mk1qprsqhpqqsq3c..."]
```

```json
"mk1": [
  ["mk1qprsqhpqqsq3c..."],
  ["mk1qa9xxxxxxxxxx..."],
  ["mk1qb8yyyyyyyyyy..."]
]
```

| Variant | JSON shape |
|---|---|
| `Single(Vec<String>)` | flat array of chunked mk1 strings (one cosigner) |
| `Multi(Vec<Vec<String>>)` | array-of-arrays, one inner array per cosigner |

### V.4.5.7 `MsField`\index{MsField}

`MsField` is a `pub type` alias for `Vec<String>` (`format.rs:54`) and serialises as a bare JSON array. SPEC §5.8 dense-vec invariant: length `N` (one entry per slot), with the empty-string sentinel `""` marking watch-only slots. A single-sig watch-only bundle emits `["", "", ""]` for a triple-cosigner pure-watch-only setup; a full single-sig emits `["ms10entrsqqq..."]`.

### V.4.5.8 Consumer-side integration

External code consuming the `mnemonic bundle` stdout envelope only needs `serde` + `serde_json` — no dependency on `mnemonic-toolkit` itself. The worked example referenced under Cross-references defines a local `BundleJson` struct that mirrors the field names and `Option` semantics of `format::BundleJson`, then parses `mnemonic bundle --json` output:

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct BundleJson {
    schema_version: String,
    mode: String,
    network: String,
    template: Option<String>,
    origin_path: Option<String>,
    master_fingerprint: Option<String>,
    ms1: Vec<String>,
    mk1: Vec<String>,  // serde untagged: bare array fits Single variant
    md1: Vec<String>,
}

let bundle: BundleJson = serde_json::from_str(&stdout)?;
if bundle.schema_version != "4" {
    return Err(format!("unexpected schema_version: {}", bundle.schema_version).into());
}
```

The `mk1` field deserialises as `Vec<String>` for single-sig (the `MkField::Single` branch of the untagged enum). For multi-source multisig, redefine the local field as `Vec<Vec<String>>` (the `MkField::Multi` branch) — the on-the-wire JSON shape is the entire discriminator. Code expecting to handle both shapes can carry an untagged `enum` mirror of `MkField` directly.

### V.4.5.9 `export-wallet` output shapes

`mnemonic export-wallet` emits a vendor-specific watch-only wallet-import artifact selected by `--format <NAME>`. The shape is fully orchestrated through the `WalletFormatEmitter`\index{WalletFormatEmitter} trait (`wallet_export/mod.rs:316-320`) — one impl per vendor, all called from `cmd::export_wallet::run` (`cmd/export_wallet.rs:399-408`). The trait carries three methods: `collect_missing(&EmitInputs) -> Vec<MissingField>`\index{MissingField} (SPEC §4 deterministic-refusal predicate; non-empty result short-circuits to `ToolkitError::ExportWalletMissingFields` via `build_missing_fields_refusal` at `wallet_export/mod.rs:279`), `emit(&EmitInputs) -> Result<String, ToolkitError>` (final byte-exact output string), and `extension() -> &'static str` (file extension hint for `--output`). Every emitter borrows the same `EmitInputs`\index{EmitInputs} struct (`wallet_export/mod.rs:327-369`); construction at `cmd/export_wallet.rs:359-375` runs **after** watch-only validation (`validate_watch_only` pre-resolve at `wallet_export/mod.rs:92` and `validate_watch_only_resolved` post-resolve at `wallet_export/mod.rs:111`), so by the time an emitter sees `EmitInputs` no slot carries `Phrase` / `Entropy` / `Xprv` / `Wif` material. The `--output <path>` flag writes the emitted string with a single trailing newline (`cmd/export_wallet.rs:410-414`); `-` (default) writes to stdout. `extension()` is not enforced against the `--output` path at HEAD — it is documented as a suggestion.

Eight vendor formats are wired at HEAD; their `--format` selector values come from the `CliExportFormat` enum at `cmd/export_wallet.rs:21-39`. The eight sub-sub-sections below cover each format in selector-enum-declaration order. The matrix in §V.4.5.10 cross-tabulates accept / refuse by descriptor shape.

`EmitInputs` field reference (per `wallet_export/mod.rs:327-369`): `canonical_descriptor: &str` (BIP-380 form with `#checksum`); `resolved_slots: &[ResolvedSlot]` (slot-index order; empty when `--descriptor` was supplied without `--template`); `template: Option<CliTemplate>`; `script_type: WalletScriptType`\index{WalletScriptType} (`wallet_export/mod.rs:143`; eight variants — `P2pkh`, `P2shP2wpkh`, `P2wpkh`, `P2tr`, `P2shMulti`, `P2shP2wshMulti`, `P2wshMulti`, `P2trMulti`); `network: CliNetwork`; `account: u32`; `threshold: Option<u8>`; `threshold_user_supplied: bool` (Sparrow consults this); `master_xpub_at_0: Option<bitcoin::bip32::Xpub>` (`@0.master_xpub=<base58>` plumbing — Coldcard only); `wallet_name: &str`; `wallet_name_was_user_supplied: bool` (Specter consults this); `taproot_internal_key: Option<TaprootInternalKey>`\index{TaprootInternalKey} (`Nums` or `Cosigner(u8)`; `wallet_export/mod.rs:68-77`); `range: (u32, u32)`; `timestamp: TimestampArg`\index{TimestampArg} (`wallet_export/mod.rs:121-125`; `Now` or `Unix(i64)`); `bitcoin_core_version: u8`.

#### V.4.5.9.1 `--format bitcoin-core`

| Property | Value |
|---|---|
| Selector | `CliExportFormat::BitcoinCore` (`cmd/export_wallet.rs:23`) |
| Emitter | `BitcoinCoreEmitter` (`wallet_export/bitcoin_core.rs:14`) |
| Output | JSON array (Bitcoin Core `importdescriptors` argument) |
| `extension()` | `"json"` |
| Accepted shapes | every BIP-388-parseable descriptor (template path + descriptor passthrough); multipath `<0;1>/*` is split into receive (`internal: false`) and change (`internal: true`) entries |
| `collect_missing` | always empty (`wallet_export/bitcoin_core.rs:17-22`) — Bitcoin Core's `importdescriptors` consumes the canonical descriptor as-is |
| Vendor-specific flags | `--range <start>,<end>` (default `0,999`; `cmd/export_wallet.rs:83`), `--timestamp <now\|unix>` (default `now`; `cmd/export_wallet.rs:87`), `--bitcoin-core-version <24\|25>` (default `25`; `cmd/export_wallet.rs:91`) |

Representative output for a single-sig BIP-84 mainnet wallet (multipath split into two entries):

```json
[
  {
    "desc": "wpkh([5436d724/84'/0'/0']xpub6B.../0/*)#<checksum>",
    "active": true,
    "internal": false,
    "range": [0, 999],
    "timestamp": "now"
  },
  {
    "desc": "wpkh([5436d724/84'/0'/0']xpub6B.../1/*)#<checksum>",
    "active": true,
    "internal": true,
    "range": [0, 999],
    "timestamp": "now"
  }
]
```

(Source: `format_bitcoin_core_importdescriptors` at `wallet_export/bitcoin_core.rs:42-86`. Multipath form `<0;1>` is split via `parsed.into_single_descriptors()` and asserted to produce exactly 2 entries; non-multipath descriptors emit a single entry with `internal: false`.) `--bitcoin-core-version` is currently consumed but unused (parameter `_bitcoin_core_version` at `wallet_export/bitcoin_core.rs:46`; reserved for future shape divergence between Core 24 vs 25).

#### V.4.5.9.2 `--format bip388`

| Property | Value |
|---|---|
| Selector | `CliExportFormat::Bip388` (`cmd/export_wallet.rs:25`) |
| Emitter | `Bip388Emitter` (`wallet_export/bip388.rs:21`) |
| Output | JSON object with `name`, `description_template`, `keys_info` |
| `extension()` | `"json"` |
| Accepted shapes | every template + every BIP-388-parseable `--descriptor` with multipath `/<0;1>/*` on every key; non-multipath suffixes refused as `BadInput` from `descriptor_to_bip388_wallet_policy` (`wallet_export/pipeline.rs:210-220`) |
| `collect_missing` | always empty (`wallet_export/bip388.rs:24-30`) |
| Vendor-specific flags | none beyond shared (`--wallet-name`, `--taproot-internal-key`) |

Representative output for a `wsh(sortedmulti(2, @0/**, @1/**, @2/**))` template at BIP-48 mainnet account 0:

```json
{
  "name": "wsh-sortedmulti",
  "description_template": "wsh(sortedmulti(2,@0/**,@1/**,@2/**))",
  "keys_info": [
    "[b8688df1/48'/0'/0'/2']xpub6FQya...",
    "[28645006/48'/0'/0'/2']xpub6DnEB...",
    "[5436d724/48'/0'/0'/2']xpub6Buxw..."
  ]
}
```

(Source: `format_bip388_wallet_policy` at `wallet_export/bip388.rs:62-137`. The template path renders `@N/**` placeholders by direct enumeration over `CliTemplate`; the descriptor-passthrough path re-extracts the keys from a canonical descriptor via `descriptor_to_bip388_wallet_policy` and string-substitutes each key-expression with `@N/**` longest-first to avoid prefix collisions — `wallet_export/pipeline.rs:160-205`. For `tr-multi-a` / `tr-sortedmulti-a` with `--taproot-internal-key nums`, the literal `NUMS_XONLY_HEX` (`50929b74...803ac0`) is embedded as the internal key; with `--taproot-internal-key @N`, cosigner `N`'s placeholder is the internal key and the remaining cosigners form the `multi_a` / `sortedmulti_a` leaf.) The `name` field is `template.human_name()` for the template path or `"imported-descriptor"` for the descriptor-passthrough path.

#### V.4.5.9.3 `--format coldcard`

| Property | Value |
|---|---|
| Selector | `CliExportFormat::Coldcard` (`cmd/export_wallet.rs:27`) |
| Emitter | `ColdcardEmitter` (`wallet_export/coldcard.rs:21`) |
| Output | JSON (singlesig generic-wallet-export) or text (multisig) |
| `extension()` | `"json"` (returned uniformly; multisig output is text-bodied) |
| Accepted shapes | singlesig `bip44` / `bip49` / `bip84` (JSON); multisig `wsh-multi` / `wsh-sortedmulti` / `sh-wsh-multi` / `sh-wsh-sortedmulti` (text) |
| Refused shapes | `bip86` → `BadInput` (`wallet_export/coldcard.rs:120-124`); `tr-multi-a` / `tr-sortedmulti-a` → `BadInput` (`wallet_export/coldcard.rs:264-269`); descriptor passthrough without `--template` → `BadInput` (`wallet_export/coldcard.rs:111-115`) |
| `collect_missing` | always empty (`wallet_export/coldcard.rs:24-40`) — refusals surface as `BadInput` with byte-exact pointer text (more helpful than the generic §4 bullet) |
| Vendor-specific flags | `--slot @0.master_xpub=<base58>` (singlesig only; top-level `xpub` field) |

Representative singlesig BIP-84 output (`wallet_export/coldcard.rs:105-225`):

```json
{
  "chain": "BTC",
  "xfp": "5436D724",
  "account": 0,
  "bip84": {
    "name": "p2wpkh",
    "deriv": "m/84'/0'/0'",
    "xfp": "127EA0E6",
    "xpub": "xpub6Bner3L3tdQW...",
    "_pub": "zpub6qTBTNftBzVTj...",
    "first": "bc1qzmtrqsfuaf6l6kkcsseumq26ukaphfj9skkug6"
  }
}
```

When `--slot @0.master_xpub=<base58>` is supplied (`cmd/export_wallet.rs:357`), a top-level `xpub` field is emitted between `xfp` and `account` carrying the depth-0 master xpub; the field is otherwise omitted entirely (`wallet_export/coldcard.rs:74-75, 216`). Top-level `xfp` is the **master** fingerprint of slot `@0` in uppercase 8-hex; the `bipNN.xfp` field is the **parent** fingerprint of the account xpub (BIP-32 serialization bytes 5–8). `bipNN.first` is the address at `/0/0` relative to the account xpub. `bipNN._pub` carries the SLIP-132 prefix variant (`ypub` for `bip49`, `zpub` for `bip84`; absent for `bip44`). `chain` is `"BTC"` mainnet, `"XTN"` testnet+signet, `"XRT"` regtest (`wallet_export/coldcard.rs:369-376`).

Representative multisig output (`wallet_export/coldcard.rs:254-365`) is a 4-header-plus-N-cosigner text block:

```text
Name: <name truncated to 20 Unicode scalar values>
Policy: <K> of <N>
Derivation: m/48'/0'/0'/2'
Format: P2WSH | P2SH-P2WSH
<XFP>: xpub6...   (one line per cosigner)
```

Notes: `Format: P2WSH` for `wsh-multi` / `wsh-sortedmulti`; `Format: P2SH-P2WSH` for `sh-wsh-multi` / `sh-wsh-sortedmulti`. Cosigners are emitted in slot-index order for `multi`; lexicographically by xpub for `sortedmulti` (`wallet_export/coldcard.rs:333-341`). `Name` truncates by Unicode scalar value via `chars().take(20)` so multi-byte UTF-8 input cannot slice mid-character (`wallet_export/coldcard.rs:302`). `Derivation` is the shared per-slot origin path if all cosigners agree; otherwise the `m/0'/0'` Coldcard placeholder. Cosigner xpubs are emitted in BIP-32 base58 form, **not** SLIP-132 (per SPEC §5.2). Multisig text is the byte-identical input format accepted by Blockstream Jade (§V.4.5.9.4).

#### V.4.5.9.4 `--format jade`

| Property | Value |
|---|---|
| Selector | `CliExportFormat::Jade` (`cmd/export_wallet.rs:29`) |
| Emitter | `JadeEmitter` (`wallet_export/jade.rs:21`) |
| Output | text (delegates to `emit_coldcard_multisig_text`) |
| `extension()` | `"txt"` |
| Accepted shapes | `wsh-multi` / `wsh-sortedmulti` / `sh-wsh-multi` / `sh-wsh-sortedmulti` (delegates byte-identical to Coldcard's §5.2 multisig text — `wallet_export/jade.rs:41-46`) |
| Refused shapes | `bip44` / `bip49` / `bip84` / `bip86` → `BadInput` ("multisig wallet config only; for singlesig setups Jade reads the seed on-device"; `wallet_export/jade.rs:53-62`); `tr-multi-a` / `tr-sortedmulti-a` → `BadInput` (pending firmware support, FOLLOWUPS `jade-tr-multi-a-pending-firmware`; `wallet_export/jade.rs:48-51`); descriptor passthrough without `--template` → `BadInput` (`wallet_export/jade.rs:36-40`) |
| `collect_missing` | always empty (`wallet_export/jade.rs:24-32`) — refusals route through `BadInput` |
| Vendor-specific flags | none beyond `--wallet-name` |

The emitted text is byte-identical to the Coldcard `--format coldcard` multisig output for the same inputs; Jade's `register_multisig.multisig_file` accepts the same shape (SPEC v0.8 §6).

#### V.4.5.9.5 `--format sparrow`

| Property | Value |
|---|---|
| Selector | `CliExportFormat::Sparrow` (`cmd/export_wallet.rs:31`) |
| Emitter | `SparrowEmitter` (`wallet_export/sparrow.rs:28`) |
| Output | JSON object (`name`, `network`, `policyType`, `scriptType`, `defaultPolicy`, `keystores`) |
| `extension()` | `"json"` |
| Accepted shapes | every template (bip44 / bip49 / bip84 / bip86 / wsh-multi / wsh-sortedmulti / sh-wsh-multi / sh-wsh-sortedmulti / tr-multi-a / tr-sortedmulti-a) |
| Refused shapes | descriptor passthrough without `--template` → `BadInput` (`wallet_export/sparrow.rs:104-108`); multisig templates without `--threshold` → `ExportWalletMissingFields { format: "sparrow", missing: [Threshold] }` (`wallet_export/sparrow.rs:31-48`) |
| `collect_missing` | returns `[Threshold]` when `template.is_multisig() && !inputs.threshold_user_supplied` (`wallet_export/sparrow.rs:31-48`) |
| Vendor-specific flags | `--wallet-name <STRING>` (label); `--threshold <K>` (required for multisig — Sparrow publishes `K` in `defaultPolicy.miniscript.script`'s `multi(K,...)` / `sortedmulti(K,...)` argument, so silent K=N defaulting would emit a wallet that bypasses the K-of-N rule) |

Representative singlesig output (`emit_sparrow_wallet_json` at `wallet_export/sparrow.rs:103-150`):

```json
{
  "name": "bip84-0",
  "network": "mainnet",
  "policyType": "SINGLE",
  "scriptType": "P2WPKH",
  "defaultPolicy": {
    "name": "Default",
    "miniscript": { "script": "wpkh(@0/**)" }
  },
  "keystores": [
    {
      "label": "bip84-0",
      "source": "SW_WATCH",
      "walletModel": "SPARROW",
      "keyDerivation": {
        "masterFingerprint": "5436d724",
        "derivation": "m/84'/0'/0'"
      },
      "extendedPublicKey": "xpub6Bner3L3tdQW..."
    }
  ]
}
```

`policyType` is `"SINGLE"` / `"MULTI"`. `scriptType` is one of `P2PKH` / `P2SH_P2WPKH` / `P2WPKH` / `P2TR` / `P2SH` / `P2SH_P2WSH` / `P2WSH` (`sparrow_script_type` at `wallet_export/sparrow.rs:154-168`); taproot multisig keeps `P2TR` because Sparrow conveys the script-path `multi_a` / `sortedmulti_a` via the `defaultPolicy.miniscript.script` field, not via a separate discriminant. `masterFingerprint` is lowercase 8-hex; `extendedPublicKey` is BIP-32 base58 form (never SLIP-132). The `defaultPolicy.miniscript.script` field carries `@N/**` placeholder expressions (`pkh(@0/**)` / `sh(wpkh(@0/**))` / `wpkh(@0/**)` / `tr(@0/**)` / `wsh(multi(K,@0/**,...))` / etc.) for non-taproot-multisig templates, and the canonical descriptor (with `#checksum` stripped — see Note below) for `tr-multi-a` / `tr-sortedmulti-a`.

> **Note (Sparrow taproot-multisig checksum stripping).** Sparrow's `defaultPolicy.miniscript.script` is a bare miniscript policy expression, **not** a BIP-380 descriptor with checksum. For `tr-multi-a` / `tr-sortedmulti-a`, `build_miniscript_script` strips the trailing `#<8-char>` suffix from the canonical descriptor before emitting (`wallet_export/sparrow.rs:215-219`) — keeping the checksum breaks Sparrow's policy parser, which substring-matches on `script` for policy detection. (Folded in commit `a890881` per v0.8.1 Phase 2 R1 review.)

#### V.4.5.9.6 `--format specter`

| Property | Value |
|---|---|
| Selector | `CliExportFormat::Specter` (`cmd/export_wallet.rs:33`) |
| Emitter | `SpecterEmitter` (`wallet_export/specter.rs:28`) |
| Output | JSON object (`label`, `blockheight`, `descriptor`, `devices`) |
| `extension()` | `"json"` |
| Accepted shapes | every template + every BIP-388-parseable `--descriptor` |
| Refused shapes | missing `--wallet-name` → `ExportWalletMissingFields { format: "specter", missing: [WalletName] }` (`wallet_export/specter.rs:31-38`) |
| `collect_missing` | returns `[WalletName]` when `!inputs.wallet_name_was_user_supplied` (`wallet_export/specter.rs:31-38`) |
| Vendor-specific flags | `--wallet-name <STRING>` (REQUIRED; Specter's UX requires a wallet label) |

Representative 2-of-3 `wsh-sortedmulti` output (`emit_specter_wallet_json` at `wallet_export/specter.rs:58-74`):

```json
{
  "label": "VaultColdStorage",
  "blockheight": 0,
  "descriptor": "wsh(sortedmulti(2,[b8688df1/48'/0'/0'/2']xpub6FQya.../<0;1>/*,[28645006/48'/0'/0'/2']xpub6DnEB.../<0;1>/*,[5436d724/48'/0'/0'/2']xpub6Buxw.../<0;1>/*))#he0ej3xr",
  "devices": ["unknown", "unknown", "unknown"]
}
```

`label` is the user-supplied `--wallet-name` (required). `blockheight` is `0` by default (`--blockheight <N>` is deferred to FOLLOWUPS). `descriptor` is the canonical BIP-380 form **with** `#<checksum>` suffix — distinct from Sparrow, which strips it for taproot multisig. `devices` is a vendor-string array of length equal to the cosigner count (1 for singlesig); every entry is the placeholder `"unknown"` because cosigner-vendor metadata is not threaded through the codecs at HEAD.

#### V.4.5.9.7 `--format electrum`

| Property | Value |
|---|---|
| Selector | `CliExportFormat::Electrum` (`cmd/export_wallet.rs:35`) |
| Emitter | `ElectrumEmitter` (`wallet_export/electrum.rs:40`) |
| Output | JSON object (`seed_version`, `wallet_type`, `use_encryption`, `keystore` for singlesig; `seed_version`, `wallet_type`, `use_encryption`, `x1/`, `x2/`, ... for multisig) |
| `extension()` | `"json"` |
| Accepted shapes | every template **except** `tr-multi-a` / `tr-sortedmulti-a` (`bip44` / `bip49` / `bip84` / `bip86` / `wsh-multi` / `wsh-sortedmulti` / `sh-wsh-multi` / `sh-wsh-sortedmulti`) |
| Refused shapes | `tr-multi-a` / `tr-sortedmulti-a` → `BadInput` (pending Electrum libsecp-taproot support, FOLLOWUPS `electrum-tr-multi-a-pending-libsecp-taproot`; `wallet_export/electrum.rs:60-68`); descriptor passthrough without `--template` → `BadInput` (`wallet_export/electrum.rs:52-56`) |
| `collect_missing` | always empty (`wallet_export/electrum.rs:43-48`) — `keystore.label` defaults to `""` when `--wallet-name` is absent |
| Vendor-specific flags | `--wallet-name <STRING>` (label; defaults to `<template>-<account>`) |
| Schema-version pin | `ELECTRUM_SEED_VERSION_PIN: u32 = 17`\index{ELECTRUM\_SEED\_VERSION\_PIN} (`wallet_export/electrum.rs:37`) — empirically validated 2026-05-12 against Electrum 4.5.5; the loader walks `_convert_version_<N>` migrations forward to FINAL\_SEED\_VERSION on first save |

Representative singlesig BIP-84 output (`emit_electrum_standard_json` at `wallet_export/electrum.rs:101-128`):

```json
{
  "seed_version": 17,
  "wallet_type": "standard",
  "use_encryption": false,
  "keystore": {
    "type": "bip32",
    "xpub": "zpub6qTBTNftBzVTj...",
    "derivation": "m/84'/0'/0'",
    "root_fingerprint": "5436d724",
    "label": "Daily"
  }
}
```

`keystore.xpub` is rendered through `render_slip132_xpub` (`wallet_export/electrum.rs:194-215`): singlesig uses lowercase SLIP-132 variants (`ypub` / `zpub`); multisig uses uppercase (`Ypub` / `Zpub`); `P2pkh`, `P2tr`, `P2shMulti` have no SLIP-132 variant and emit the neutral xpub form. Taproot multisig is refused upstream by `emit()`.

Representative 2-of-4 `wsh-sortedmulti` output (`emit_electrum_multisig_json` at `wallet_export/electrum.rs:131-188`):

```json
{
  "seed_version": 17,
  "use_encryption": false,
  "wallet_type": "2of4",
  "x1/": {
    "derivation": "m/48'/0'/0'/2'",
    "label": "VaultCold-1",
    "root_fingerprint": "b8688df1",
    "type": "bip32",
    "xpub": "Zpub75ybJh4YZjnMs..."
  },
  "x2/": { /* ... */ },
  "x3/": { /* ... */ },
  "x4/": { /* ... */ }
}
```

Multisig `wallet_type` is the string `"<K>of<N>"`. Cosigner keys are `x1/`, `x2/`, ... in slot-index order; each carries `derivation`, `label = "<wallet_name>-<i+1>"`, `root_fingerprint` (lowercase 8-hex), `type = "bip32"`, and `xpub`. Field ordering in the multisig branch is `serde_json::Map` insertion-order-stable but alphabetical at serialization time because the multisig builder routes through `Value::Object(Map)` (`wallet_export/electrum.rs:152-187`); Electrum's loader does not depend on field order (Python `json`).

#### V.4.5.9.8 `--format green`

| Property | Value |
|---|---|
| Selector | `CliExportFormat::Green` (`cmd/export_wallet.rs:37`) |
| Emitter | `GreenEmitter` (`wallet_export/green.rs:23`) |
| Output | text (3 lines: 2 comment lines + canonical descriptor) |
| `extension()` | `"txt"` |
| Accepted shapes | singlesig `bip44` / `bip49` / `bip84` / `bip86`; descriptor passthrough for singlesig BIP-388 shapes |
| Refused shapes | every multisig template (`is_multisig()`) → `BadInput` ("Green's multisig setup is server-mediated (Green Multisig Shield) and not a file-import surface"; `wallet_export/green.rs:33-39`) |
| `collect_missing` | always empty (`wallet_export/green.rs:26-28`) |
| Vendor-specific flags | none beyond shared |

Representative output (`wallet_export/green.rs:40-43`):

```text
# Blockstream Green — Watch-only import (singlesig)
# Help: https://help.blockstream.com/hc/en-us/articles/19340800530713-Set-up-watch-only-wallet
wpkh([5436d724/84'/0'/0']xpub6Bner.../<0;1>/*)#00lx6ere
```

Green has no native descriptor-import file shape; the help article documents pasting the descriptor or xpub into Green's "Import from file" dialog. The toolkit emits a 3-line text file with two comment lines pointing at the help URL plus the canonical descriptor.

### V.4.5.10 `export-wallet` format × shape compatibility matrix

The matrix below cross-tabulates the 8 vendor formats against the 7 descriptor shapes the toolkit's `CliTemplate` enumerates. Cell legend: `✓` accepted; `R:<reason>` refused with the noted reason. Cells whose acceptance depends on a `--threshold` or `--wallet-name` flag carry an inline footnote.

| Shape | `bitcoin-core` | `bip388` | `coldcard` | `jade` | `sparrow` | `specter` | `electrum` | `green` |
|---|---|---|---|---|---|---|---|---|
| `wpkh` (bip84) | ✓ | ✓ | ✓ | R:singlesig | ✓ [c] | ✓ [d] | ✓ | ✓ |
| `pkh` (bip44) | ✓ | ✓ | ✓ | R:singlesig | ✓ [c] | ✓ [d] | ✓ | ✓ |
| `sh(wpkh)` (bip49) | ✓ | ✓ | ✓ | R:singlesig | ✓ [c] | ✓ [d] | ✓ | ✓ |
| `tr(xpub)` (bip86) | ✓ | ✓ | R:bip86 [a] | R:singlesig | ✓ [c] | ✓ [d] | ✓ | ✓ |
| `wsh(sortedmulti)` / `wsh(multi)` | ✓ | ✓ | ✓ | ✓ | ✓ [b], [c] | ✓ [d] | ✓ | R:multisig |
| `sh(wsh(sortedmulti))` / `sh(wsh(multi))` | ✓ | ✓ | ✓ | ✓ | ✓ [b], [c] | ✓ [d] | ✓ | R:multisig |
| `tr(NUMS, multi_a)` / `tr(NUMS, sortedmulti_a)` | ✓ | ✓ | R:firmware [e] | R:firmware [f] | ✓ [b], [c] | ✓ [d] | R:libsecp [g] | R:multisig |
| `tr(@N, multi_a)` / `tr(@N, sortedmulti_a)` (cosigner-internal) | ✓ | ✓ | R:firmware [e] | R:firmware [f] | ✓ [b], [c] | ✓ [d] | R:libsecp [g] | R:multisig |

Footnotes (each cell-level refusal text comes from `wallet_export/<file>.rs`):

- **[a]** Coldcard generic-wallet-export schema documents only `bip44`/`bip49`/`bip84`; BIP-86 refuses with byte-exact `BadInput` (`wallet_export/coldcard.rs:120-124`).
- **[b]** Sparrow refuses multisig templates without explicit `--threshold` — `collect_missing` returns `[MissingField::Threshold]` (`wallet_export/sparrow.rs:31-48`) → `ExportWalletMissingFields { format: "sparrow", missing: [Threshold] }` (exit 2).
- **[c]** Sparrow strips the BIP-380 `#checksum` from the embedded miniscript only for `tr-multi-a` / `tr-sortedmulti-a` (Note in §V.4.5.9.5).
- **[d]** Specter requires explicit `--wallet-name` — `collect_missing` returns `[MissingField::WalletName]` (`wallet_export/specter.rs:31-38`) → `ExportWalletMissingFields { format: "specter", missing: [WalletName] }` (exit 2).
- **[e]** Coldcard firmware does not ingest taproot multisig text exports (FOLLOWUPS `coldcard-tr-multi-a-pending-firmware`; `wallet_export/coldcard.rs:264-269`).
- **[f]** Blockstream Jade firmware does not ingest taproot multisig wallet config (FOLLOWUPS `jade-tr-multi-a-pending-firmware`; `wallet_export/jade.rs:48-51`).
- **[g]** Electrum's wallet-db does not ingest taproot multisig (FOLLOWUPS `electrum-tr-multi-a-pending-libsecp-taproot`; `wallet_export/electrum.rs:60-68`).
- Jade refuses every singlesig template ("multisig wallet config only; for singlesig setups Jade reads the seed on-device"; `wallet_export/jade.rs:53-62`).
- Green refuses every multisig template ("Green's multisig setup is server-mediated"; `wallet_export/green.rs:33-39`).
- For `tr-multi-a` / `tr-sortedmulti-a`, `cmd::export_wallet::run` requires `--taproot-internal-key <nums|@N>` (`cmd/export_wallet.rs:182-194`) before any emitter is reached; the matrix rows are conditioned on that flag being present.

All refusals route to exit code 2 (per `ToolkitError::exit_code` in §V.4.4). Refusals routed through `BadInput` produce a single-line `error: ...` stderr; refusals routed through `ExportWalletMissingFields` produce the multi-line `build_missing_fields_refusal` shape (`wallet_export/mod.rs:279-296`).

## V.4.6 Engraving-card layout

The `mnemonic bundle` subcommand emits a unified engraving card on **stderr** (the `--quiet` flag suppresses it). The card is a three-card mental model — ms1, mk1, md1 cards laid out alongside each other physically — rendered as a single human-readable text block. The sole producing function is `format::engraving_card_unified` (`format.rs:259-376`); `BundleJson.engraving_card` was removed in v0.5.0 Phase A.3.

The renderer accepts `BundleInputForCard` (header + threshold + N + language + passphrase-used + privacy + per-slot blocks + md1 chunk_set_id) and emits eight ordered sections: header line, threshold (multisig only), cosigners block (N ≥ 2) or single-slot summary (N == 1), template-or-descriptor line, md1 reference line, recovery hint (multisig only), language / passphrase footer, hardware-wallet caveat for `tr-multi-a` and `tr-sortedmulti-a`. Descriptor strings over 80 characters truncate to a 60-char prefix plus `... [md1: <id>] (<n> chars total)` annotation.

Cross-reference: §IV.1 (bundle anatomy), §IV.2 (anti-collision invariants), §IV.3 (future shares) hold the deep technical details (`chunk_set_id` derivation, ms1/mk1/md1 card-id collision invariants, recovery procedure). This chapter shows only the stderr-emitted text.

### V.4.6.1 Single-sig BIP-86 example

```text
# === Wallet bundle: bip86, mainnet ===
# ms1: c4f1
# mk1: 7a2b
# fingerprint: 5436d724
# origin path: m/86'/0'/0'
# Template: bip86
# md1: 9e03
# Language: english
```

### V.4.6.2 3-of-5 wsh-sortedmulti example

```text
# === Wallet bundle: wsh-sortedmulti, mainnet ===
# Threshold: 3 of 5
# Cosigners:
#   @0: ms1:c4f1,mk1:7a2b (5436d724 @ m/48'/0'/0'/2')
#   @1: ms1:8de2,mk1:b193 (a112f8e0 @ m/48'/0'/0'/2')
#   @2: ms1:30ac,mk1:9f5c (c739a04b @ m/48'/0'/0'/2')
#   @3: ms1:e811,mk1:2d4a (4b08e6c2 @ m/48'/0'/0'/2')
#   @4: ms1:6f53,mk1:5510 (e22d70a9 @ m/48'/0'/0'/2')
# Template: wsh-sortedmulti
# md1: 9e03
# Recovery: any 3 of 5 signing keys + md1 (template card).
# Language: english
```

Both examples are illustrative output shapes drawn from the renderer at `format.rs:259-376`; the `ms1:`, `mk1:`, `md1:` four-hex IDs are `chunk_set_id` short-forms (top 20 bits of the relevant codec's identity hash, hex-encoded). The ms1 short-form derives from `policy_id_stub`; the mk1 short-form derives from each cosigner's mk1 chunk header; the md1 short-form derives from `md_codec::derive_chunk_set_id` (§V.1.5.3).

The card is stderr-only and is **not** part of the JSON envelope — `BundleJson.engraving_card` was removed in v0.5.0 Phase A.3. Programs that need a machine-readable bundle should parse stdout (`BundleJson`); programs that need the human-readable engraving block should capture stderr separately. The `--quiet` flag suppresses the card; `--json` does not affect it (the card always emits on stderr unless `--quiet` is set). This mirrors the SPEC §5.5 invariant that stdout carries the JSON envelope alone and is therefore pipe-safe to `jq` or `serde_json::from_reader`.

## V.4.7 Versioning and library-extraction posture

- Crate version: **0.8.0** (HEAD `4210b91c`).
- Rust edition: **2021** (inherited from workspace `Cargo.toml`).
- MSRV: **1.85** (`rust-version` inherited from workspace).
- License: **MIT**.
- Binary target only: `[[bin]] name = "mnemonic" path = "src/main.rs"`. No `[lib]` target; no `src/lib.rs`.
- Public semver promise: **none**. Pre-1.0 reference implementation; any 0.X bump may break the CLI surface, the JSON envelope schema, the engraving-card layout, or the in-binary module structure. The `schema_version` field exists precisely so external JSON consumers can pin against a stable contract independent of the crate version: at v0.8.0 the contract is `"4"`.
- v0.9+ **library extraction is deferred.** The `pub` items enumerated in §V.4.3 are the candidate facade if and when a `src/lib.rs` ships; until then, external integration goes through the CLI + JSON envelopes. No promise is made that the v0.9 library surface will mirror v0.8.0's in-binary `pub` set verbatim.

## V.4.8 Notes for advanced users

- **Sibling codecs are git-pinned, not crates.io.** `crates/mnemonic-toolkit/Cargo.toml:20-22` pins `ms-codec` to `ms-codec-v0.1.0`, `mk-codec` to `mk-codec-v0.2.1`, `md-codec` to `md-codec-v0.16.1` — all via git tags against their respective repos. The workspace also carries a `[patch.crates-io]` pinning `miniscript` to a specific git rev (see workspace `Cargo.toml`). This is **pre-crates.io-publish** state: a crates.io-facing public Rust API contract for the m-format-star does not yet exist. Downstream consumers that want a stable contract today should target the CLI binary + JSON envelopes, not the in-binary library API.
- **`schema_version = "4"` is HEAD; the `format.rs:114` doc-comment is stale.** Every constructor site of `BundleJson` and `VerifyBundleJson` literal-encodes `schema_version: "4"` (`synthesize.rs:1296`, `cmd/bundle.rs:572`, `cmd/verify_bundle.rs:182,498`). The module-level doc-comment at `format.rs:114` still reads `v0.2: schema_version "2"`. The doc-comment drift was first reported in v0.3 Phase 3.1 and persists at HEAD; chapter authors and external consumers should treat the constructor-site literal as authoritative and ignore the doc-comment until it is reconciled.
- **`check_key_vector_distinctness` is the sole `pub` BIP-388 distinct-key entry point.** It lives at `parse_descriptor.rs:1104` and compares **typed** `bitcoin::bip32::DerivationPath` equality at `parse_descriptor.rs:1108` (which folds `h` and `'`). The CLI-layer mirror `cmd::bundle::check_resolved_slots_distinctness` is `pub(crate)` and uses raw-string equality on `(xpub.to_string(), path_raw)` — that mirror is **out of Part V scope**. The doc-comment lag on both layers (`bundle.rs:259-260` and `error.rs:68-71` describing the equality semantic as raw-string) persists at HEAD and is tracked as a known drift item.
- **`md1_xpub_match` is a `VerifyCheck.name` string literal, not a `pub` function.** It appears only inside `cmd::verify_bundle` (`verify_bundle.rs:1214, 1224, 1242, 1267, 1321, 1331, 1349, 1374, 1438, 1595`). The contract for external JSON consumers is the string itself (a row in `VerifyBundleJson.checks` with `name == "md1_xpub_match"`), not a callable. Coverage of SPEC §5.7 multisig-vs-single-sig path disclosure is therefore a property of the JSON envelope schema, not the orchestration-module surface.
- **`ToolkitError` is `#[non_exhaustive]`.** Any future library extraction (v0.9+) will require external matchers to include `_ => ...` arms. The annotation is at `error.rs:10`. The forward-compatibility implication mirrors `ms_codec::Error` and `mk_codec::Error` (both `#[non_exhaustive]`). **`md_codec::Error` is the exception:** it is NOT `#[non_exhaustive]` — it derives only `Debug, Error, PartialEq, Eq` (see §V.1.3.9); the toolkit's `md_codec_exit_code` match at `error.rs:174` is consequently exhaustive (no `_ =>` arm needed, and the compiler will warn if a new variant is added upstream — that's intentional).
- **JSON envelope optionality is mixed.** Some `Option<T>` fields serialise as JSON `null` (`BundleJson.template`, `descriptor`, `origin_path`, `origin_paths`, `master_fingerprint`, `multisig`); others are omitted entirely when `None` (every `VerifyCheck` forensic field, via `skip_serializing_if = "Option::is_none"` at `format.rs:171, 174, 177, 181`). External consumers MUST tolerate both shapes simultaneously. The mixed policy is intentional: top-level `null` is informative ("this bundle has no multisig metadata"), whereas forensic omission keeps passing-check rows compact.
- **`#[non_exhaustive]` discipline is uniform across the public types.** `ToolkitError` (`error.rs:10`) is the only top-level `#[non_exhaustive]` declaration at HEAD; the JSON envelope structs (`BundleJson`, `VerifyBundleJson`, `VerifyCheck`, `MultisigInfo`, `CosignerEntry`, `BundleInputForCard`, `SlotCardBlock`) are **not** marked `#[non_exhaustive]`. Brace-init from within the crate is permitted; brace-init from an extracted library would remain permitted at v0.9 unless the extraction adds the annotation. Chapter readers planning a v0.9 facade should treat the JSON-envelope structs as forward-stable in field count, since the wire schema is what `schema_version` versions.
- **MSRV 1.85 is high.** Most Bitcoin Rust libraries target older MSRV; downstream consumers building against an extracted v0.9 library should flag this in their own `rust-version`.
- **`BundleJson.descriptor` is preserved verbatim, not re-rendered.** When the user supplies `--descriptor=<text>` on `mnemonic bundle`, the field carries the user's literal input string. Verify-bundle round-tripping reparses through `parse_descriptor::parse_descriptor` and re-encodes through `synthesize::build_descriptor`; the `md1` re-encoding is compared as a string at SPEC §5.7 check `md1_decode`, not by re-emitting `descriptor`. The original descriptor text is therefore the canonical wire form for descriptor-mode bundles; any whitespace, placeholder annotation, or multipath ordering chosen by the user is preserved.
- **`CliTemplate::wrapper_node` and `template::md_origin_path` are the bridge into `md-codec`.** The md1 wire-format wrapper node and origin path are produced from `CliTemplate` directly without going through `miniscript::Descriptor`; the descriptor pipeline (§V.4.3.4) is engaged only in descriptor mode. In template mode, the md1 layer sees only the wrapper shape and the canonical origin path — no per-key annotations. This split is why the BIP-388 distinct-key check (§V.4.3.4) is only meaningful in descriptor mode: template mode synthesises each cosigner's path deterministically and cannot produce a colliding pair by construction.
- **The eight `WalletFormatEmitter` impls are a shipped CLI-level surface.** v0.8.1 promoted Sparrow + Specter from format stubs to real emitters, added Coldcard, Jade, Electrum, and Green, and wired `--slot @0.master_xpub=<base58>` into Coldcard. At HEAD the `wallet_export` module exposes eight `pub(crate)` emitter structs implementing the `WalletFormatEmitter` trait (§V.4.5.9) plus a `pub(crate) build_missing_fields_refusal` function that is the sole construction site of the SPEC §4 byte-exact missing-info refusal shape. The `ExportWalletFormatStub` variant of `ToolkitError` (`error.rs:99`) is `#[allow(dead_code)]` at HEAD — no construction site remains in the codebase; the variant is retained for future per-vendor stub introductions (removing it from a `#[non_exhaustive]` enum would otherwise be a breaking change). `ExportWalletTaprootMultisigUnsupported` (`error.rs:106`) is similarly retained for back-compat but is unreachable post-v0.8 NUMS lift; the per-vendor taproot-multisig refusals all surface as `BadInput` with byte-exact pointer text (per-vendor footnotes in §V.4.5.10).

## Cross-references

- §II.1 — md1 wire format.
- §II.2 — mk1 wire format.
- §II.3 — ms1 wire format.
- §IV.1 — bundle anatomy (the three-card physical layout rendered by `engraving_card_unified`).
- §IV.2 — anti-collision invariants (the `chunk_set_id` short-forms appearing on every card).
- §IV.3 — future shares (the v0.2-shares migration locked across md1 / mk1 / ms1).
- §V.1 — md-codec API (the codec wrapped by `synthesize_*` and decoded by `cmd::verify_bundle`).
- §V.2 — mk-codec API.
- §V.3 — ms-codec API.
- Worked example: `cargo run --quiet --manifest-path docs/technical-manual/examples/Cargo.toml --example mnemonic-toolkit-api-roundtrip` — source at `docs/technical-manual/examples/examples/mnemonic-toolkit-api-roundtrip.rs`; transcript pair at `docs/technical-manual/transcripts/mnemonic-toolkit-api-roundtrip.{cmd,out}`. The transcript's `.out` line is: `parsed BundleJson: schema_version=4 mode=full network=mainnet template=bip84 origin_path=m/84'/0'/0' fingerprint=5436d724 ms1_len=1 mk1_len=1 md1_len=1`.

<!-- cspell-additions: "Jade" (v0.8.1 wallet-export emitter); other vendor terms (Coldcard, Sparrow, Specter, Blockstream, Electrum, Bitcoin Core) already in the dictionary -->
