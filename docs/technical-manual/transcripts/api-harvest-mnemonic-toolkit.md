# mnemonic-toolkit API surface harvest

| Field | Value |
|---|---|
| Crate | mnemonic-toolkit |
| Version | 0.8.0 |
| Source root | /scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit |
| HEAD commit | 4210b91c9a858ea0586f58a407a6a69b616abf07 |
| Rust edition | 2021 (workspace) |
| MSRV | 1.85 (workspace `rust-version`) |
| Crate kind | **Binary only** — no `[lib]` target, no `src/lib.rs`. The crate is a Cargo `[[bin]] name = "mnemonic"` with `path = "src/main.rs"`. Library re-use by external crates is therefore impossible at v0.8.0 via `extern crate`; all `pub` items below are reachable only within this binary or via in-crate integration tests in `tests/`. Phase 4.4 must call out this dividing line explicitly — Part V documents what an end-user would consume **if** the crate ever ships a library facade. |

## Feature flags

`[features]` section is **absent** from `crates/mnemonic-toolkit/Cargo.toml`. No optional features; no default feature set. All code is unconditionally compiled.

## Dependencies (public-facing types appearing in `pub` signatures)

All from `[dependencies]` in `Cargo.toml`. None are re-exported, but the following sibling/upstream types leak into the in-crate `pub` API surface (function arg types, return types, struct fields):

- `md-codec` (git tag `md-codec-v0.16.1`): `md_codec::Descriptor`, `md_codec::Error`, `md_codec::TlvSection`, `md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths}`, `md_codec::tag::Tag`, `md_codec::tree::{Body, Node}`, `md_codec::use_site_path::{Alternative, UseSitePath}`.
- `mk-codec` (git tag `mk-codec-v0.2.1`): `mk_codec::Error`, `mk_codec::KeyCard` (used internally, not in public field types).
- `ms-codec` (git tag `ms-codec-v0.1.0`): `ms_codec::Error`, `ms_codec::Tag`, `ms_codec::Payload` (internal; only `ms_codec::Error` leaks via `ToolkitError::MsCodec`).
- `bip39` (v2): `bip39::Error`, `bip39::Language` (via `From<CliLanguage>`), `bip39::Mnemonic` (in `synthesize_multisig_full` signature).
- `bitcoin` (v0.32): `bitcoin::NetworkKind`, `bitcoin::address::KnownHrp`, `bitcoin::bip32::{DerivationPath, Fingerprint, Xpriv, Xpub}`.
- `miniscript` (v13): only inside private helpers; not in any `pub` signature.
- `clap` (v4): every `*Args` struct derives `Args`; enums derive `ValueEnum`.
- `serde` (v1): `Serialize`/`Deserialize` derives on JSON envelope structs.

## Public modules (top-level, declared in `src/main.rs`)

All modules in `main.rs` are declared with bare `mod NAME;` (i.e., **private** at crate root from an external-API standpoint, since the crate has no `lib.rs`). Treating them as "modules the chapter walks through" rather than externally `pub mod`:

- `bip85` (`src/bip85.rs`) — all items `pub(crate)`. **No `pub` items.**
- `bundle_unified` (`src/bundle_unified.rs`) — `BundleMode`, `detect_bundle_mode`, `pre_check_threshold`, `pre_check_template_n`.
- `cmd` (`src/cmd/mod.rs`) — submodules `bundle`, `convert`, `derive_child`, `export_wallet`, `verify_bundle`. **OUT OF SCOPE for Part V** (CLI dispatch surface).
- `derive` (`src/derive.rs`) — `DerivedAccount`, `derive_full`.
- `derive_slot` (`src/derive_slot.rs`) — no `pub` (only `pub(crate)`).
- `electrum` (`src/electrum.rs`) — all `pub(crate)`. **No `pub` items.**
- `error` (`src/error.rs`) — `ToolkitError`, `BitcoinErrorKind`, `Result`.
- `format` (`src/format.rs`) — formatting + JSON envelope types.
- `friendly` (`src/friendly.rs`) — `friendly_bip39`, `friendly_bitcoin`, `friendly_ms_codec`, `friendly_mk_codec`, `friendly_md_codec`.
- `language` (`src/language.rs`) — `CliLanguage`.
- `network` (`src/network.rs`) — `CliNetwork`.
- `parse` (`src/parse.rs`) — input parsing helpers + `CosignerSpec`, `MultisigPathFamily`.
- `parse_descriptor` (`src/parse_descriptor.rs`) — descriptor pipeline + binding.
- `slip0132` (`src/slip0132.rs`) — `XpubPrefix`, `parse_xpub_prefix_arg`.
- `slot_input` (`src/slot_input.rs`) — `SlotSubkey`, `SlotInput`, `ParseError`, `parse_slot_input`, `validate_slot_set`.
- `synthesize` (`src/synthesize.rs`) — `Bundle`, `ResolvedSlot`, `CosignerKeyInfo`, synthesize-* family.
- `template` (`src/template.rs`) — `CliTemplate`.
- `wallet_export` (`src/wallet_export.rs`) — `REFUSAL_SECRET_INPUT`, `format_stub_message`, `taproot_multisig_unsupported_message`, `TaprootInternalKey`.
- `wordlists` (`src/wordlists/mod.rs`) — all `pub(crate)`. **No `pub` items.**

## Public surface by module

### `mnemonic_toolkit::error` (`src/error.rs`)

#### Enums

- `pub enum ToolkitError` (`#[non_exhaustive]`) — central error enum. SPEC §6.1–§6.4. `src/error.rs:10`. Variants:
  - `BadInput(String)` — generic exit-1 (user input).
  - `Bip39(bip39::Error)` — BIP-39 mnemonic parse/validate failure.
  - `Bitcoin(BitcoinErrorKind)` — bitcoin-crate-sourced error wrapper.
  - `MsCodec(ms_codec::Error)` — ms1 codec wrapper.
  - `MkCodec(mk_codec::Error)` — mk1 codec wrapper.
  - `MdCodec(md_codec::Error)` — md1 codec wrapper.
  - `ModeViolation { mode, flag, message }` — SPEC §5.5 flag-vs-mode violation; exit 2.
  - `BundleMismatch { card: String, message: String }` — SPEC §6.1 exit-4 verify-bundle mismatch.
  - `NetworkMismatch { xpub_network, expected }` — SPEC §4.3 xpub network does not match `--network`.
  - `FutureFormat { source, detail }` — exit 3; emitted when a sibling codec reports a reserved-not-yet-emitted tag / unsupported version.
  - `MultisigConfig { message }` — SPEC §6.2 v0.2 threshold/cosigner-range; exit 1.
  - `CosignerSpec { cosigner_idx, message }` — SPEC §6.2 `--cosigner=<xpub>:<fp>:<path>` parse.
  - `CosignersFile { message }` — SPEC §6.2 `--cosigners-file` JSON parse.
  - `DescriptorParse(String)` — SPEC §6.7 descriptor content parse failure; exit 2.
  - `DescriptorReparseFailed { detail }` — SPEC §5.7 verify-bundle descriptor reparse failure; exit 4.
  - `Bip388Distinctness { i: u8, j: u8 }` — SPEC §4.11.b distinct-key violation at bundle creation; exit 2.
  - `Bip388VerifyDistinctness` — SPEC §4.11.c distinct-key violation at verify-bundle; exit 4.
  - `SlotInputViolation { kind, message }` — SPEC §6.6 / §6.6.b unified slot input violation; exit 2. `kind` ∈ `"conflict"|"gap"|"invalid-set"|"duplicate-subkey"|"empty"|"threshold-range"|"missing-threshold"|"single-sig-multi-slot"|"multisig-single-slot"`.
  - `ConvertRefusal(String)` — SPEC_convert §3/§4 refusal; exit 2.
  - `ExportWalletSecretInput` — SPEC_export_wallet §3 watch-only refusal; exit 2.
  - `ExportWalletFormatStub(&'static str)` — SPEC_export_wallet §7 sparrow/specter stub; exit 2.
  - `ExportWalletTaprootMultisigUnsupported(&'static str)` — SPEC_export_wallet §4 (now unreachable post-v0.8 NUMS support, but variant retained for back-compat).
  - `DeriveChildUnsupportedApp` — SPEC_derive_child §7 rsa/rsa-gpg deferred; exit 2.
  - `DeriveChildLengthOutOfRange { app, length, valid_text }` — SPEC_derive_child §7 length range.
  - `DeriveChildLengthNotApplicable` — SPEC_derive_child §4/§7 length-not-applicable for fixed-size apps.

- `pub enum BitcoinErrorKind` (`src/error.rs:119`):
  - `Bip32(bitcoin::bip32::Error)`
  - `XpubParse(String)`
  - `FingerprintParse(String)`

#### Impls on `ToolkitError`

- `pub fn exit_code(&self) -> u8` — SPEC §6.1 exit-code mapping (`src/error.rs:223`).
- `pub fn kind(&self) -> &'static str` (`#[allow(dead_code)]`) — stable JSON `kind` discriminant (`src/error.rs:254`).
- `pub fn message(&self) -> String` — friendly message dispatch (`src/error.rs:288`).
- `pub fn details(&self) -> Option<serde_json::Value>` (`#[allow(dead_code)]`) — JSON `details` field (`src/error.rs:364`).
- `impl std::fmt::Display`, `impl std::error::Error`, `From<bip39::Error>`, `From<bitcoin::bip32::Error>`, `From<ms_codec::Error>` (with `ReservedTagNotEmittedInV01` → `FutureFormat`), `From<mk_codec::Error>` (with `UnsupportedVersion` → `FutureFormat`), `From<md_codec::Error>` (with `UnsupportedVersion` → `FutureFormat`).

#### Type aliases

- `pub type Result<T> = std::result::Result<T, ToolkitError>` (`src/error.rs:453`).

### `mnemonic_toolkit::format` (`src/format.rs`)

#### Functions

- `pub fn chunk_5char(s: &str) -> String` — render in 5-char groups, 10 groups/line max (`src/format.rs:10`).
- `pub fn chunk_mk1(s: &str) -> String` (`#[allow(dead_code)]`) — currently delegates to `chunk_5char`; reserved for mk-codec chunked-form swap (`src/format.rs:33`).
- `pub fn chunk_md1(s: &str) -> String` — delegates to `md_codec::encode::render_codex32_grouped(s, 5)` (`src/format.rs:38`).
- `pub fn engraving_card_unified(input: &BundleInputForCard) -> String` — SPEC §5.5 unified-card render (`src/format.rs:259`).
- `pub fn chunk_set_id_extract(s: &str) -> Option<u32>` (`#[allow(dead_code)]`) — extract chunk_set_id from mk1 chunked header (`src/format.rs:385`).

#### Types (JSON envelope shapes + card layout)

- `pub type MsField = Vec<String>` — SPEC §5.8 (v0.4) dense ms1 layout; `""` sentinel = watch-only slot (`src/format.rs:54`).
- `pub enum MkField` (`#[serde(untagged)]`) — discriminated union for `BundleJson.mk1` (`src/format.rs:66`):
  - `Single(Vec<String>)` — flat single-sig.
  - `Multi(Vec<Vec<String>>)` — per-cosigner nested.
  - methods: `pub fn as_single(&self) -> Option<&Vec<String>>` (`#[allow(dead_code)]`), `pub fn as_multi(&self) -> Option<&Vec<Vec<String>>>` (`#[allow(dead_code)]`).
- `pub struct CosignerEntry` (`#[derive(Serialize)]`, `src/format.rs:94`) — per-cosigner descriptor in `MultisigInfo.cosigners`:
  - `pub index: usize`
  - `pub master_fingerprint: Option<String>` — None when `--privacy-preserving`.
  - `pub origin_path: String`
  - `pub xpub: String`
- `pub struct MultisigInfo` (`#[derive(Serialize)]`, `src/format.rs:104`):
  - `pub template: &'static str`
  - `pub threshold: u8`
  - `pub cosigner_count: usize`
  - `pub path_family: &'static str` — `"bip48"|"bip87"`.
  - `pub cosigners: Vec<CosignerEntry>`
- `pub struct BundleJson` (`#[derive(Serialize)]`, `src/format.rs:120`) — Bundle JSON output schema:
  - `pub schema_version: &'static str` — value at HEAD is `"4"` (per `synthesize.rs:1732`, `cmd/bundle.rs:906`, and 5 further sites in `cmd/import_wallet.rs`, `cmd/verify_bundle.rs` (×2), `wallet_import/json_envelope.rs` (×2) — 7 sites total; the doc-comment at `src/format.rs:114` still says `"v0.2: schema_version "2"";` — stale).
  - `pub mode: &'static str` — `"full"|"watch-only"`.
  - `pub network: &'static str`
  - `pub template: Option<&'static str>`
  - `pub descriptor: Option<String>`
  - `pub account: u32`
  - `pub origin_path: Option<String>`
  - `pub origin_paths: Option<Vec<String>>`
  - `pub master_fingerprint: Option<String>`
  - `pub ms1: MsField`
  - `pub mk1: MkField`
  - `pub md1: Vec<String>`
  - `pub multisig: Option<MultisigInfo>`
  - `pub privacy_preserving: bool`
- `pub struct VerifyBundleJson` (`#[derive(Serialize)]`, `src/format.rs:149`):
  - `pub schema_version: &'static str` — value `"4"` (`cmd/verify_bundle.rs:329, 1017`).
  - `pub result: &'static str` — `"ok"|"mismatch"`.
  - `pub checks: Vec<VerifyCheck>`
- `pub struct VerifyCheck` (`#[derive(Serialize, Clone)]`, `src/format.rs:166`) — SPEC §5.7:
  - `pub name: String`
  - `pub passed: bool`
  - `pub detail: String`
  - `pub expected: Option<String>` (`#[serde(skip_serializing_if = "Option::is_none")]`)
  - `pub actual: Option<String>` (`#[serde(skip)]`)
  - `pub diff_byte_offset: Option<usize>` (`#[serde(skip)]`)
  - `pub decode_error: Option<String>` (`#[serde(skip)]`)
  - methods: `impl Default for VerifyCheck`; `pub fn diff_offset(a: &str, b: &str) -> usize` (`src/format.rs:202`).
- `pub struct BundleInputForCard` (`src/format.rs:223`) — engraving-card input (not serde):
  - `pub network: &'static str`
  - `pub template_or_descriptor: TemplateOrDescriptor`
  - `pub threshold: Option<u8>`
  - `pub n: u8`
  - `pub language: Option<&'static str>`
  - `pub passphrase_used: bool`
  - `pub privacy_preserving: bool`
  - `pub per_slot: Vec<SlotCardBlock>`
  - `pub md1_chunk_set_id: String`
- `pub enum TemplateOrDescriptor` (`src/format.rs:236`):
  - `Template(&'static str)`
  - `Descriptor(String)`
- `pub struct SlotCardBlock` (`src/format.rs:242`):
  - `pub index: u8`
  - `pub ms1_card_id: Option<String>`
  - `pub mk1_card_id: String`
  - `pub fingerprint: Option<String>`
  - `pub origin_path: Option<String>`

### `mnemonic_toolkit::language` (`src/language.rs`)

#### Enums

- `pub enum CliLanguage` (`#[derive(ValueEnum, Default)]`, `#[clap(rename_all = "lower")]`, `src/language.rs:10`):
  - `English` (Default), `SimplifiedChinese`, `TraditionalChinese`, `Czech`, `French`, `Italian`, `Japanese`, `Korean`, `Portuguese`, `Spanish`.
  - methods: `pub fn human_name(&self) -> &'static str` (`src/language.rs:26`).
  - `impl From<CliLanguage> for bip39::Language`.

### `mnemonic_toolkit::network` (`src/network.rs`)

#### Enums

- `pub enum CliNetwork` (`#[derive(ValueEnum)]`, `#[clap(rename_all = "lower")]`, `src/network.rs:12`):
  - `Mainnet`, `Testnet`, `Signet`, `Regtest`.
  - methods:
    - `pub fn coin_type(&self) -> u32` — BIP-32 coin-type (mainnet=0, others=1) (`:22`).
    - `pub fn network_kind(&self) -> bitcoin::NetworkKind` — mainnet→Main, others→Test (`:30`).
    - `pub fn known_hrp(&self) -> bitcoin::address::KnownHrp` — bech32 HRP selector (`:40`).
    - `pub fn human_name(&self) -> &'static str` (`:49`).

### `mnemonic_toolkit::template` (`src/template.rs`)

#### Enums

- `pub enum CliTemplate` (`#[derive(ValueEnum)]`, `src/template.rs:15`) — 10 templates:
  - `Bip44`, `Bip49`, `Bip84`, `Bip86` (single-sig).
  - `WshMulti` (`wsh-multi`), `WshSortedMulti` (`wsh-sortedmulti`), `ShWshMulti` (`sh-wsh-multi`), `ShWshSortedMulti` (`sh-wsh-sortedmulti`), `TrMultiA` (`tr-multi-a`), `TrSortedMultiA` (`tr-sortedmulti-a`).
  - methods:
    - `pub fn is_multisig(&self) -> bool` (`:46`).
    - `pub fn origin_path_str(&self, network: CliNetwork, account: u32) -> String` (`:61`).
    - `pub fn derivation_path(&self, network: CliNetwork, account: u32) -> bitcoin::bip32::DerivationPath` (`:75`).
    - `pub fn md_origin_path(&self, network: CliNetwork, account: u32) -> md_codec::origin_path::OriginPath` (`:82`).
    - `pub fn wrapper_node(&self, k: u8, n: usize) -> md_codec::tree::Node` (`:111`).
    - `pub fn bip48_script_type(&self) -> Option<u32>` (`:219`).
    - `pub fn human_name(&self) -> &'static str` (`:228`).

### `mnemonic_toolkit::derive` (`src/derive.rs`)

#### Structs

- `pub struct DerivedAccount` (`src/derive.rs:14`):
  - `pub entropy: Vec<u8>`
  - `pub master_fingerprint: bitcoin::bip32::Fingerprint`
  - `pub account_xpub: bitcoin::bip32::Xpub`
  - `pub account_xpriv: bitcoin::bip32::Xpriv`
  - `pub account_path: bitcoin::bip32::DerivationPath`

#### Functions

- `pub fn derive_full(phrase: &str, passphrase: &str, language: CliLanguage, network: CliNetwork, template: CliTemplate, account: u32) -> Result<DerivedAccount, ToolkitError>` (`src/derive.rs:22`).

### `mnemonic_toolkit::parse` (`src/parse.rs`)

#### Functions

- `pub fn read_phrase_input(arg: Option<&str>, stdin: &mut dyn Read) -> Result<String, ToolkitError>` (`src/parse.rs:17`).
- `pub fn parse_master_fingerprint(s: &str) -> Result<Fingerprint, ToolkitError>` (`src/parse.rs:38`).
- `pub fn parse_cosigner_spec(s: &str, cosigner_idx: usize) -> Result<CosignerSpec, ToolkitError>` (`#[allow(dead_code)]`, `:111`).
- `pub fn parse_cosigners_file(path: &Path) -> Result<Vec<CosignerSpec>, ToolkitError>` (`#[allow(dead_code)]`, `:173`).
- `pub fn check_no_concurrent_stdin(phrase: Option<&str>, passphrase: Option<&str>) -> Result<(), ToolkitError>` (`:224`).

#### Types

- `pub struct CosignerSpec` (`#[allow(dead_code)]`, `src/parse.rs:53`):
  - `pub xpub: bitcoin::bip32::Xpub`
  - `pub master_fingerprint: bitcoin::bip32::Fingerprint`
  - `pub path: Option<bitcoin::bip32::DerivationPath>`
- `pub enum MultisigPathFamily` (`#[derive(ValueEnum, Default)]`, `#[allow(dead_code)]`, `src/parse.rs:63`):
  - `Bip48`
  - `Bip87` (Default)
  - methods: `pub fn human_name(&self) -> &'static str` (`:71`); `pub fn default_origin_path(&self, network: CliNetwork, account: u32, script_type: u32) -> String` (`:84`).

### `mnemonic_toolkit::synthesize` (`src/synthesize.rs`)

#### Functions

- `pub fn xpub_to_65(xpub: &Xpub) -> [u8; 65]` — SPEC §4.6.1 chain_code||pubkey form (`:98`).
- `pub fn build_descriptor(template, network, xpub, fingerprint, account) -> Descriptor` (`#[allow(dead_code)]`, `:109`).
- `pub fn synthesize_full(entropy, fingerprint, xpub, template, network, account) -> Result<Bundle, ToolkitError>` (`#[allow(dead_code)]`, `:142`).
- `pub fn synthesize_watch_only(fingerprint, xpub, template, network, account) -> Result<Bundle, ToolkitError>` (`#[allow(dead_code)]`, `:181`).
- `pub fn synthesize_descriptor(descriptor, cosigners, privacy_preserving, run_language) -> Result<Bundle, ToolkitError>` (`:229`).
- `pub fn synthesize_multisig_full(seed_mnemonic, passphrase, network, template, threshold, cosigner_count, account, path_family, privacy_preserving) -> Result<Bundle, ToolkitError>` (`:344`).
- `pub fn synthesize_multisig_watch_only(cosigners, network, template, threshold, account, path_family, privacy_preserving) -> Result<Bundle, ToolkitError>` (`:489`).
- `pub fn synthesize_unified(slots, template, threshold, network, privacy_preserving, run_language) -> Result<Bundle, ToolkitError>` (`:745`) — current dispatch entrypoint for unified slot pipeline.

#### Types

- `pub struct Bundle` (`src/synthesize.rs:22`):
  - `pub ms1: MsField` — SPEC §5.8 dense layout.
  - `pub mk1: MkField`
  - `pub md1: Vec<String>`
  - method: `pub fn any_secret_bearing(&self) -> bool` (`:35`).
- `pub struct ResolvedSlot` (`src/synthesize.rs:642`):
  - `pub xpub: Xpub`
  - `pub fingerprint: Fingerprint`
  - `pub path: DerivationPath` — typed; SPEC §4.11.b uses this for distinctness equality with `h ↔ '` folding.
  - `pub entropy: Option<zeroize::Zeroizing<Vec<u8>>>` — None = watch-only slot.
  - method: `pub fn is_secret_bearing(&self) -> bool` (`:690`).
- `pub type CosignerKeyInfo = ResolvedSlot` (`#[allow(dead_code)]`, `:219`) — legacy alias.

### `mnemonic_toolkit::parse_descriptor` (`src/parse_descriptor.rs`)

#### Functions

- `pub fn lex_placeholders(descriptor: &str) -> Result<Vec<PlaceholderOccurrence>, ToolkitError>` (`:60`).
- `pub fn resolve_placeholders(occs: &[PlaceholderOccurrence]) -> Result<ResolvedPlaceholders, ToolkitError>` (`:156`).
- `pub fn substitute_synthetic(descriptor: &str, ctx: ScriptCtx) -> Result<(String, BTreeMap<String, u8>), ToolkitError>` (`:263`).
- `pub fn walk_root(desc: &MsDescriptor<DescriptorPublicKey>, km: &BTreeMap<String, u8>) -> Result<Node, ToolkitError>` (`:353`).
- `pub fn parse_descriptor(input: &str, keys: &[ParsedKey], fingerprints: &[ParsedFingerprint]) -> Result<MdDescriptor, ToolkitError>` (`:687`).
- `pub fn determine_mode(d: &MdDescriptor) -> DescriptorMode` (`#[allow(dead_code)]`, `:757`).
- `pub fn synthetic_xpub_for(i: u8, ctx: ScriptCtx) -> String` (`:769`) — deterministic synthetic xpub for `@i`; seed prefix `b"toolkit-v0.3"`.
- `pub fn bind_descriptor_keys(resolved, network, phrase, passphrase, language, xpub_arg, master_fp_arg, cosigner_specs) -> Result<DescriptorBinding, ToolkitError>` (`:815`).
- `pub fn check_key_vector_distinctness(binding: &DescriptorBinding) -> Result<(), ToolkitError>` (`:1208`) — SPEC §4.11.b typed-DerivationPath distinctness.

#### Types

- `pub enum ScriptCtx` (`src/parse_descriptor.rs:32`): `SingleSig`, `MultiSig`.
- `pub struct PlaceholderOccurrence` (`:50`):
  - `pub i: u8`, `pub fingerprint_anno: Option<Fingerprint>`, `pub origin_path_anno: Option<DerivationPath>`, `pub multipath_alts: Vec<u32>`, `pub wildcard_hardened: bool`.
- `pub struct ResolvedPlaceholders` (`:145`):
  - `pub n: u8`, `pub path_decl: PathDecl`, `pub fingerprint_annos: Vec<Option<Fingerprint>>`, `pub use_site_path: UseSitePath`, `pub use_site_path_overrides: Vec<(u8, UseSitePath)>`.
- `pub struct ParsedKey` (`:673`): `pub i: u8`, `pub payload: [u8; 65]`.
- `pub struct ParsedFingerprint` (`:679`): `pub i: u8`, `pub fp: [u8; 4]`.
- `pub enum DescriptorMode` (`#[allow(dead_code)]`, `:751`): `SingleSig`, `MultiSig`.
- `pub struct DescriptorBinding` (`:790`):
  - `pub keys: Vec<ParsedKey>`, `pub fingerprints: Vec<ParsedFingerprint>`, `pub cosigners: Vec<CosignerKeyInfo>`.
  - method: `pub fn entropy_at_0(&self) -> Option<&[u8]>` (`:805`).

### `mnemonic_toolkit::slot_input` (`src/slot_input.rs`)

(Module carries `#![allow(dead_code)]` because items are wired as clap value-parser callbacks.)

#### Functions

- `pub fn parse_slot_input(s: &str) -> Result<SlotInput, ParseError>` (`:77`).
- `pub fn validate_slot_set(slots: &[SlotInput]) -> Result<(), ToolkitError>` (`:152`).

#### Types

- `pub enum SlotSubkey` (`src/slot_input.rs:13`): `Phrase`, `Entropy`, `Xpub`, `Fingerprint`, `Path`, `Wif`, `Xprv`.
  - methods: `pub fn as_str(self) -> &'static str` (`:36`); `pub fn is_secret_bearing(self) -> bool` (`:47`); `pub fn is_watch_only(self) -> bool` (`:50`).
- `pub struct SlotInput` (`:56`):
  - `pub index: u8`, `pub subkey: SlotSubkey`, `pub value: String`.
- `pub struct ParseError(pub String)` (`:65`) — clap value-parser error wrapper.

### `mnemonic_toolkit::bundle_unified` (`src/bundle_unified.rs`)

(Module carries `#![allow(dead_code)]`.)

#### Functions

- `pub fn detect_bundle_mode(slots: &[SlotInput]) -> Result<BundleMode, ToolkitError>` (`:34`).
- `pub fn pre_check_threshold(threshold: Option<u8>, n: usize, multisig_template: Option<&str>) -> Result<(), ToolkitError>` (`:67`).
- `pub fn pre_check_template_n(template: &str, is_multisig_template: bool, n: usize) -> Result<(), ToolkitError>` (`:94`).

#### Enums

- `pub enum BundleMode` (`:15`): `SingleSigFull`, `SingleSigWatchOnly`, `MultisigMultiSource`, `MultisigWatchOnly`, `MultisigHybrid`.

### `mnemonic_toolkit::slip0132` (`src/slip0132.rs`)

#### Functions

- `pub fn parse_xpub_prefix_arg(s: &str) -> Result<XpubPrefix, String>` (`:38`).

#### Enums

- `pub enum XpubPrefix` (`:17`): `Xpub`, `Ypub`, `YpubMultisig`, `Zpub`, `ZpubMultisig`.
  - method: `pub fn is_default(self) -> bool` (`:31`).

(The actual decode/swap/encode functions — `normalize_xpub_prefix`, `apply_xpub_prefix`, `neutral_for`, `render_slip0132_info_line` — are `pub(crate)`, so they are **not part of the in-crate `pub` surface** but are crate-internal helpers.)

### `mnemonic_toolkit::friendly` (`src/friendly.rs`)

#### Functions

- `pub fn friendly_bip39(e: &bip39::Error) -> String` (`:10`).
- `pub fn friendly_bitcoin(e: &BitcoinErrorKind) -> String` (`:34`).
- `pub fn friendly_ms_codec(e: &ms_codec::Error) -> String` (`:42`).
- `pub fn friendly_mk_codec(e: &mk_codec::Error) -> String` (`:84`).
- `pub fn friendly_md_codec(e: &md_codec::Error) -> String` (`:128`).

### `mnemonic_toolkit::wallet_export` (`src/wallet_export.rs`)

#### Constants

- `pub const REFUSAL_SECRET_INPUT: &str` (`:17`) — SPEC §3 byte-exact stderr text for secret-input refusal under `export-wallet`.

#### Functions

- `pub fn format_stub_message(name: &str) -> String` (`:21`) — sparrow/specter stub refusal text.
- `pub fn taproot_multisig_unsupported_message(name: &str) -> String` (`#[allow(dead_code)]`, `:33`) — pre-v0.8 refusal text, retained for the (unreachable) error variant message.

#### Enums

- `pub enum TaprootInternalKey` (`:42`): `Nums`, `Cosigner(u8)`.

(Helpers `build_descriptor_string`, `validate_watch_only`, `validate_watch_only_resolved`, `format_bitcoin_core_importdescriptors`, `format_bip388_wallet_policy`, `descriptor_to_bip388_wallet_policy`, `TimestampArg` enum, `NUMS_XONLY_HEX` const are all `pub(crate)` — not part of the `pub` surface.)

### CLI command modules (OUT OF SCOPE for Part V)

The following items live under `cmd::*` and are CLI dispatch surface (consumed only by `src/main.rs::main`). Part V chapter SHOULD NOT document them; they belong to the end-user manual.

- `cmd::bundle::BundleArgs` (clap-derive Args struct), `cmd::bundle::run`, `cmd::bundle::self_check_bundle`, `cmd::bundle::mode_text` (pub-mod nested constants).
- `cmd::verify_bundle::VerifyBundleArgs`, `cmd::verify_bundle::run`, `cmd::verify_bundle::SuppliedCards<'a>`, `cmd::verify_bundle::emit_verify_checks`.
- `cmd::convert::ConvertArgs`, `cmd::convert::run`, `cmd::convert::NodeType`, `cmd::convert::FromInput`, `cmd::convert::parse_from_input`, `cmd::convert::ScriptType`, `cmd::convert::parse_script_type_arg`.
- `cmd::export_wallet::ExportWalletArgs`, `cmd::export_wallet::run`, `cmd::export_wallet::CliExportFormat`, `cmd::export_wallet::TimestampArgValue`.
- `cmd::derive_child::DeriveChildArgs`, `cmd::derive_child::run`.

These are the entrypoints `main.rs` calls; their public fields are clap argument plumbing.

## Error taxonomy

| Variant | Doc-comment summary | Exit | `kind()` | Emitted by (sample call sites — non-exhaustive) |
|---|---|---|---|---|
| `BadInput(String)` | generic exit-1 user-input failure | 1 | `BadInput` | `parse::read_phrase_input`, `parse::parse_master_fingerprint`, `parse::check_no_concurrent_stdin`, `slip0132::normalize_xpub_prefix` |
| `Bip39(bip39::Error)` | BIP-39 mnemonic parse/validate | 1 | `Bip39` | `derive_full` (via `Mnemonic::parse_in`), `bind_full_mode` |
| `Bitcoin(BitcoinErrorKind)` | bitcoin-crate wrapper | 1 | `Bitcoin` | `parse::parse_master_fingerprint`, `synthesize::derive_xpub_at_path`, `parse_descriptor::bind_*` |
| `MsCodec(ms_codec::Error)` | ms1 codec error | per `ms_codec_exit_code` (1\|2) | `MsCodec` | `synthesize_*` (via `ms_codec::encode`), `emit_verify_checks` (via `ms_codec::decode`) |
| `MkCodec(mk_codec::Error)` | mk1 codec error | per `mk_codec_exit_code` (1\|2) | `MkCodec` | `synthesize_*` (via `mk_codec::encode_with_chunk_set_id`) |
| `MdCodec(md_codec::Error)` | md1 codec error | per `md_codec_exit_code` (1\|2\|3) | `MdCodec` | `synthesize_*` (via `md_codec::chunk::split` / `compute_wallet_policy_id`) |
| `ModeViolation { mode, flag, message }` | SPEC §5.5 flag-vs-mode | 2 | `ModeViolation` | `cmd::bundle::run`, `parse_descriptor::bind_descriptor_keys` |
| `BundleMismatch { card, message }` | verify-bundle string mismatch | 4 | `BundleMismatch` | `cmd::verify_bundle::run` (constructed on §5.7 check failure) |
| `NetworkMismatch { xpub_network, expected }` | SPEC §4.3 | 2 | `NetworkMismatch` | `synthesize_multisig_watch_only`, `synthesize_unified` |
| `FutureFormat { source, detail }` | reserved-not-emitted-in-this-version | 3 | `FutureFormat` | `From<ms_codec::Error>`, `From<mk_codec::Error>`, `From<md_codec::Error>` for `UnsupportedVersion`/`ReservedTagNotEmittedInV01` |
| `MultisigConfig { message }` | threshold/N-range | 1 | `MultisigConfig` | `synthesize_multisig_*`, `synthesize_unified` |
| `CosignerSpec { cosigner_idx, message }` | `--cosigner=` parse | 1 | `CosignerSpec` | `parse::parse_cosigner_spec`, depth-check in `synthesize_multisig_watch_only` and `synthesize_unified` |
| `CosignersFile { message }` | `--cosigners-file` parse | 1 | `CosignersFile` | `parse::parse_cosigners_file` |
| `DescriptorParse(String)` | SPEC §6.7 descriptor content | 2 | `DescriptorParse` | `lex_placeholders`, `resolve_placeholders`, `substitute_synthetic`, `walk_root`, `parse_descriptor`, `bind_*` family, `synthesize_descriptor` (n-cosigner-count mismatch) |
| `DescriptorReparseFailed { detail }` | SPEC §5.7 verify-bundle re-parse | 4 | `DescriptorReparseFailed` | `cmd::verify_bundle::run` (descriptor mode round-trip check) |
| `Bip388Distinctness { i, j }` | bundle-creation §4.11.b | 2 | `Bip388Distinctness` | `check_key_vector_distinctness`, `cmd::bundle::check_resolved_slots_distinctness` |
| `Bip388VerifyDistinctness` | verify-bundle §4.11.c | 4 | `Bip388VerifyDistinctness` | `cmd::verify_bundle::run` (re-wraps `Bip388Distinctness` post-binding) |
| `SlotInputViolation { kind, message }` | unified slot input | 2 | `SlotInputViolation` | `slot_input::validate_slot_set`, `bundle_unified::detect_bundle_mode`, `bundle_unified::pre_check_threshold`, `bundle_unified::pre_check_template_n` |
| `ConvertRefusal(String)` | convert §3/§4 refusal | 2 | `ConvertRefusal` | `cmd::convert::refusal_one_way`, `cmd::convert::refusal_sibling_pivot`, etc. |
| `ExportWalletSecretInput` | export-wallet §3 watch-only | 2 | `ExportWalletSecretInput` | `wallet_export::validate_watch_only`, `wallet_export::validate_watch_only_resolved` |
| `ExportWalletFormatStub(&'static str)` | sparrow/specter | 2 | `ExportWalletFormatStub` | `cmd::export_wallet::run` |
| `ExportWalletTaprootMultisigUnsupported(&'static str)` | pre-v0.8 tr-multisig | 2 | `ExportWalletTaprootMultisigUnsupported` | (variant retained; now-unreachable at runtime post-v0.8 NUMS shipment) |
| `DeriveChildUnsupportedApp` | rsa/rsa-gpg deferred | 2 | `DeriveChildUnsupportedApp` | `cmd::derive_child::run` |
| `DeriveChildLengthOutOfRange { app, length, valid_text }` | length per-app range | 2 | `DeriveChildLengthOutOfRange` | `cmd::derive_child::run` |
| `DeriveChildLengthNotApplicable` | length not applicable | 2 | `DeriveChildLengthNotApplicable` | `cmd::derive_child::run` |

## JSON envelope schema items

| Type | Path | Top-level fields | Serde derives | Notes |
|---|---|---|---|---|
| `BundleJson` | `format::BundleJson` | `schema_version, mode, network, template, descriptor, account, origin_path, origin_paths, master_fingerprint, ms1, mk1, md1, multisig, privacy_preserving` | `Serialize` | SPEC §5.3. Schema 4 layout. `schema_version` is `&'static str` and is set to `"4"` at every construction site — 7 total: `synthesize.rs:1732`, `cmd/bundle.rs:906`, `cmd/import_wallet.rs:1499`, `cmd/verify_bundle.rs:329`, `cmd/verify_bundle.rs:1017`, `wallet_import/json_envelope.rs:595`, `wallet_import/json_envelope.rs:761`. Field order is part of the schema. |
| `VerifyBundleJson` | `format::VerifyBundleJson` | `schema_version, result, checks` | `Serialize` | SPEC §5.4. `schema_version = "4"` at construction (`cmd/verify_bundle.rs:329,1017`). `result ∈ {"ok","mismatch"}`. |
| `VerifyCheck` | `format::VerifyCheck` | `name, passed, detail, expected, actual, diff_byte_offset, decode_error` | `Serialize, Clone` | SPEC §5.7. Optional forensic fields use `#[serde(skip_serializing_if = "Option::is_none")]`. |
| `MultisigInfo` | `format::MultisigInfo` | `template, threshold, cosigner_count, path_family, cosigners` | `Serialize` | Embedded in `BundleJson.multisig`. |
| `CosignerEntry` | `format::CosignerEntry` | `index, master_fingerprint, origin_path, xpub` | `Serialize, Clone` | Per-cosigner entry in `MultisigInfo.cosigners`. |
| `MkField` | `format::MkField` | `Single(Vec<String>) \| Multi(Vec<Vec<String>>)` | `Serialize, Clone, #[serde(untagged)]` | JSON-on-wire is bare array or array-of-arrays (no discriminator). |
| `MsField` | `format::MsField` (type alias = `Vec<String>`) | — | (delegates to `Vec<String>`) | SPEC §5.8 schema-4 dense vec; `""` = watch-only sentinel. |

The `cmd::convert` JSON envelope (`ConvertJson<'a>` / `ConvertJsonEntry<'a>` in `src/cmd/convert.rs:293,302`) is **non-`pub`** (struct-level visibility, no `pub`), and lives under the CLI dispatch surface. Schema version `"1"` is set at `cmd/convert.rs:782`.

## Engraving-card layout items

The `format::engraving_card_unified` function (`src/format.rs:259`) is the sole engraving-card-producing surface in v0.5+ (`BundleJson.engraving_card` field was removed in v0.5.0 Phase A.3). Its layout takes the following `pub` types:

- `BundleInputForCard` (header / threshold / N / language / passphrase_used / privacy_preserving / per_slot / md1_chunk_set_id).
- `TemplateOrDescriptor` (`Template(&'static str)` vs `Descriptor(String)`; descriptor truncates at 80 chars with `[md1: <id>]` annotation).
- `SlotCardBlock` (per-slot `index`, `ms1_card_id`, `mk1_card_id`, `fingerprint`, `origin_path`).

The 8-section render order is in `src/format.rs:259-376`:
1. Header line — `# === Wallet bundle: {summary}, {network} ===`.
2. Threshold (multisig only).
3. Cosigners block (multisig N≥2) OR single-slot summary (N==1).
4. Template OR Descriptor line.
5. md1 reference line.
6. Recovery hint (multisig only).
7. Language / Passphrase footer.
8. Hardware-wallet caveat for `tr-multi-a` / `tr-sortedmulti-a`.

## Feature-gated items

| Item | Feature | Path |
|---|---|---|
| — | — | — |

No `#[cfg(feature = ...)]` gates exist in the crate. The crate has no feature flags. Test-only items use `#[cfg(test)]` only.

## Notes for chapter author (Phase 4.4)

- **Crate has no library target.** `src/main.rs` declares every module as bare `mod NAME;` (private at crate root). External crates cannot `use mnemonic_toolkit::*` at v0.8.0. Phase 4.4 must frame Part V's coverage of this crate as "the binary's internal architecture as a documented surface" rather than "a public Rust API for downstream consumption." If/when a `lib.rs` is added, the items above are the candidate surface.
- **CLI surface is OUT OF SCOPE for Part V.** Items under `src/cmd/*.rs` (BundleArgs, VerifyBundleArgs, ConvertArgs, ExportWalletArgs, DeriveChildArgs and their `run` functions, plus `cmd::bundle::mode_text` constants, `cmd::verify_bundle::SuppliedCards`/`emit_verify_checks`, `cmd::convert::NodeType`/`FromInput`/`ScriptType`/`parse_*`) are clap-derived CLI dispatch — they belong to the end-user manual, not Part V.
- **`schema_version` confirmed at `"4"` at HEAD.** Every construction site of `BundleJson`/`VerifyBundleJson` literal-encodes `schema_version: "4"`. The `format::BundleJson` doc-comment at `src/format.rs:114` still reads `"v0.2: schema_version "2""` — this is a stale doc-comment (v0.3 Phase 3.1's documented gap **persists at HEAD**). Phase 4.4 should document the current value `"4"` and not echo the stale module-level comment.
- **`md1_xpub_match` exists only inside `cmd::verify_bundle` private helpers.** No `pub` API surface emits a check named `md1_xpub_match` — that name appears only as a `VerifyCheck.name` string literal at `src/cmd/verify_bundle.rs:1214,1224,1242,1267,1321,1331,1349,1374,1438,1595`. Coverage of the v0.3 Phase 3.5 multisig-vs-single-sig path-disclosure gap is therefore a property of the CLI-surface implementation, not the library API — Phase 4.4 should either skip the topic or document it via the JSON envelope's `VerifyCheck` type (mention that `name == "md1_xpub_match"` checks fall under SPEC §5.7).
- **Both BIP-388 distinctness layers now use TYPED `DerivationPath` equality at HEAD — no bifurcation.** The CLI-layer mirror `cmd::bundle::check_resolved_slots_distinctness` (a `pub(crate)` helper, out of Part V scope) compares `slots[i].xpub.to_string() == slots[j].xpub.to_string() && slots[i].path == slots[j].path` — the **typed** `DerivationPath` (`bundle.rs:429`), and its source doc-comment (`bundle.rs:423-429`) has already been updated to the typed framing (v0.5 §4.11.b deliberate reversal; the former raw-string `path_raw` field was deleted in v0.37.9). The **sole `pub` function** enforcing BIP-388 distinct-key semantics is `parse_descriptor::check_key_vector_distinctness` at `parse_descriptor.rs:1208`, which compares **typed** `DerivationPath` equality (`cs[i].path == cs[j].path` at `parse_descriptor.rs:1212`, which folds `h ↔ '`). Both layers therefore agree: `48h/..` and `48'/..` collide at synthesis AND verify. The **only remaining source-comment lag** is `error.rs:13-16` — the `Bip388Distinctness` doc still says "`(xpub, derivation_path_string)` raw-string equality," which now mis-describes the typed behavior. Phase 4.4 must document the typed-`DerivationPath ==` semantics accurately and may flag the lone `error.rs` doc-comment lag as a known drift item. (Verified: `src/synthesize.rs` contains no function named `check_key_vector_distinctness` — the only `pub` `check_*` function on the BIP-388 path lives in `parse_descriptor`.)
- **`DerivationPath` surface.** Public `pub` API exposes `bitcoin::bip32::DerivationPath` in: `derive::DerivedAccount.account_path`, `template::CliTemplate::derivation_path`, `parse::CosignerSpec.path`, `parse::MultisigPathFamily::default_origin_path` (returns `String`, not typed — flag this asymmetry), `synthesize::ResolvedSlot.path`, `parse_descriptor::PlaceholderOccurrence.origin_path_anno`. The BIP-388 distinct-key enforcement on the `pub` surface uses **typed** `DerivationPath == DerivationPath` (folds `h ↔ '`) per `check_key_vector_distinctness` at `parse_descriptor.rs:1212`.
- **Optional fields in JSON envelopes use `#[serde(skip_serializing_if = "Option::is_none")]` only on `VerifyCheck` forensic fields** (`expected`, `actual`, `diff_byte_offset`, `decode_error`). Other `Option<T>` fields on `BundleJson` (e.g., `template`, `descriptor`, `origin_path`, `origin_paths`, `master_fingerprint`, `multisig`) serialize as `null` when None — JSON readers must tolerate explicit nulls. Verify this in the chapter draft against actual `bundle --json` output before claiming behavior.
- **MSRV 1.85 is high.** Most Bitcoin Rust libraries target older MSRV; downstream consumers depending on this MSRV should be flagged.
- **`#[non_exhaustive] ToolkitError`** — external callers (when a library facade ships) must handle `_ => …` arms; chapter should call out the forward-compatibility implication.
- **Sibling-codec versions are git-pinned, not crates.io.** `Cargo.toml` pins `md-codec-v0.16.1`, `mk-codec-v0.2.1`, `ms-codec-v0.1.0` via git tags. Workspace also carries a `[patch.crates-io]` pinning `miniscript` to a git rev. Phase 4.4 should note this is **pre-crates.io-publish** state — the public Rust API contract via crates.io is not yet established.
