//! ToolkitError + exit_code() + per-source From impls.
//!
//! Realizes SPEC §6.1 (exit-code table), §6.2 (ToolkitError enum),
//! §6.3 (exit-code mapping), §6.4.0 (routing principle).

use serde_json::json;

#[derive(Debug)]
#[non_exhaustive]
pub enum ToolkitError {
    BadInput(String),
    Bip39(bip39::Error),
    /// SPEC §4.11.b BIP-388 distinct-key violation at bundle creation. Exit 2.
    /// `i` and `j` are the colliding slot indices (i < j) under
    /// `(xpub.to_string(), path)` typed-`DerivationPath` equality per §4.11.b
    /// (`h`/`'`-notation folds; mirrors the `cmd::bundle` twin comment).
    Bip388Distinctness {
        i: u8,
        j: u8,
    },
    /// SPEC §4.11.c BIP-388 distinct-key violation at verify-bundle. Exit 2 (exit-code spec §3).
    /// Re-emitted from `check_key_vector_distinctness` post-binding under
    /// verify-bundle (different exit code + message vs `Bip388Distinctness`).
    Bip388VerifyDistinctness,
    Bitcoin(BitcoinErrorKind),
    /// v0.31.0 — `mnemonic import-wallet --format bsms --bsms-encryption-token <FILE|->`
    /// decrypted a BIP-129 encryption-envelope wire blob but the HMAC-SHA256
    /// MAC verify failed: either the supplied TOKEN is wrong, or the wire
    /// blob has been tampered with. Per BIP-129 §Encryption (Encrypt-and-MAC
    /// ordering), the MAC is recomputed over decrypted plaintext and
    /// compared to the received MAC (first 32 bytes of wire). Exit 1 (a comparison that came out no) —
    /// authentication-class refusal (typed per FOLLOWUP body recommendation).
    BsmsMacMismatch {
        token_len_hex: usize,
    },
    /// v0.27.0 — `mnemonic import-wallet --bsms-round1 <FILE>` parsed a
    /// blob that does not meet BIP-129 §Round 1 record syntax (line count
    /// != 5 after CRLF normalize, line 1 != `BSMS 1.0`, malformed line-3
    /// KEY field, malformed line-5 base64 SIG, line-4 description carries
    /// `\n` or `\r`, etc.). Exit 2 — parse-class error.
    BsmsRound1Malformed {
        reason: String,
    },
    /// v0.27.0 — BIP-129 §Round 1 BIP-322 ECDSA recoverable signature
    /// verification failed: signature recovered to a different pubkey than
    /// the one declared on the record's line 3, OR recovery itself failed.
    /// Exit 1 — explicit failure (NOT a stderr NOTICE; the user opted into
    /// verify by supplying `--bsms-round1` + `--bsms-verify-strict`).
    BsmsSignatureMismatch {
        record_index: usize,
        signer_pubkey: String,
        reason: String,
    },
    /// Import-side parity of `BsmsTaprootRefused`. v0.28.7+: refused at
    /// `BsmsParser::parse` entry. No `script_type` field — the import parser
    /// has no `WalletScriptType` in scope at parse time (see
    /// `design/cycle-3-p0-recon.md` Slug 1 lock α).
    BsmsTaprootImportRefused,
    /// v0.28.0 P8B (plan-doc §S.8) — `mnemonic export-wallet --format bsms`
    /// refused because the requested template / descriptor resolves to a
    /// taproot script-type (`P2tr` / `P2trMulti`). BIP-129 §1 prerequisites
    /// pre-date BIP-386 — there is no published canonicalization for taproot
    /// descriptors in the BSMS Round-2 wire shape. The carried `script_type`
    /// drives the per-script-type discriminator in the rendered message
    /// (P2tr → singlesig hint; P2trMulti → multisig hint). Real emit is
    /// tracked at FOLLOWUP `bsms-taproot-emit` (upstream-blocked).
    ///
    /// Exit 2 (parse / refusal class) — same routing as the prior
    /// `ToolkitError::BadInput` text this variant replaces at
    /// `wallet_export/bsms.rs:emit`.
    BsmsTaprootRefused {
        script_type: crate::wallet_export::WalletScriptType,
    },
    /// `build-descriptor` spec parse / input error (bad JSON, unknown field,
    /// unsupported schema_version, stdin/file read failure). Exit 2.
    BuildDescriptorSpec(String),
    /// SPEC §6.1 exit-4 verify-bundle mismatch variant. `card` identifies the
    /// mismatching card (e.g., "mk1", "md1", or "mk1[N]" for multisig cosigner N).
    #[allow(dead_code)]
    BundleMismatch {
        card: String,
        message: String,
    },
    /// v0.26.0 compare-cost subcommand error. SPEC §9.
    CompareCost(crate::cost::CompareCostError),
    /// SPEC_convert_v0_6.md §3 / §4 refusal — convert subcommand rejects
    /// a (from, to) pair as cryptographically unrecoverable, sibling-pivot,
    /// or otherwise invalid. Exit 2.
    ConvertRefusal(String),
    /// SPEC §6.2 v0.2 cosigner-spec parse error
    /// (`--cosigner=<xpub>:<fp>:<path>`). Exit 2.
    #[allow(dead_code)]
    CosignerSpec {
        cosigner_idx: usize,
        message: String,
    },
    /// SPEC §6.2 v0.2 cosigners-file (JSON) parse error. Exit 2.
    #[allow(dead_code)]
    CosignersFile {
        message: String,
    },
    /// v0.36.0 `mnemonic decode-address` — the supplied string is not a valid
    /// Bitcoin address (bad prefix/HRP/checksum). Exit 2. Public-data utility,
    /// no secrets.
    DecodeAddress(String),
    /// SPEC_derive_child_v0_7.md §4 / §7 — non-zero `--length` supplied to
    /// an app whose output is fixed-size (`hd-seed`, `xprv`). Exit 64 (`--length` is a setting).
    DeriveChildLengthNotApplicable,
    /// SPEC_derive_child_v0_7.md §7 — `--length <N>` falls outside the
    /// per-app valid range. Exit 64.
    DeriveChildLengthOutOfRange {
        app: &'static str,
        length: u32,
        valid_text: &'static str,
    },
    /// SPEC_derive_child_v0_7.md §7 — `--application rsa|rsa-gpg` deferred
    /// pending rsa-crate stability (RUSTSEC-2023-0071 unpatched as of v0.8.0).
    /// `dice` shipped in v0.8 Phase 7. Exit 2.
    DeriveChildUnsupportedApp,
    /// SPEC §6.7 descriptor parse error (lex/resolve/walk failure). Exit 2.
    /// Distinct from `ModeViolation` (SPEC §6.9, flag-combination errors):
    /// `DescriptorParse` covers descriptor *content* failures.
    DescriptorParse(String),
    /// verify-bundle descriptor mode: the completed keyed wallet failed to
    /// re-parse. Exit 101 (a bug).
    ///
    /// **v0.97.0: believed UNREACHABLE, retained defensively.** Descriptor
    /// INPUT errors (lex/resolve/parse, unreadable `--descriptor-file`,
    /// unsupported `--slot` subkeys) now exit 2 (`DescriptorParse`), matching
    /// `bundle`. `parse_descriptor` is deterministic in its string argument, so
    /// the post-binding re-parse cannot fail once the canonicity probe has
    /// succeeded on the identical string; a failure would indicate
    /// toolkit-internal parse non-determinism. The earlier doc scoped this to a
    /// "preserved descriptor" from an artifact — no such descriptor ever reaches
    /// these sites (`verify-bundle` has no `--from-import-json`, and
    /// `--bundle-json` never populates `descriptor`). Not presented in
    /// user-facing docs as a live exit-4 meaning.
    DescriptorReparseFailed {
        detail: String,
    },
    /// v0.26.0 wallet-import cycle — cross-cutting `@env:<VAR>` sentinel
    /// resolution failed. Either the env-var was unset (`Unset`) or the
    /// `<VAR>` token failed POSIX env-var-name validation (`InvalidName`).
    /// Exit 32 when unset, 64 for an invalid NAME, 2 for a non-UTF-8 value (exit-code spec §3).
    /// Carries the offending `--flag` name for stderr disambiguation across
    /// the 6 secret-flag surfaces enumerated in SPEC §3.1.
    EnvVarMissing {
        flag: String,
        var: String,
        reason: EnvVarMissingReason,
    },
    /// SPEC_export_wallet_v0_7.md §7 — sparrow / specter format stub. Exit 2.
    /// v0.8.1 Phase 2 + Phase 3 promoted Sparrow + Specter to real formats;
    /// no construction site remains in the codebase. Variant retained for
    /// future per-vendor stub introductions (would otherwise be a breaking
    /// removal from a `#[non_exhaustive]` enum).
    #[allow(dead_code)]
    ExportWalletFormatStub(&'static str),
    /// SPEC_export_wallet_v0_8.md §4 — missing-info refusal. Each per-format
    /// emitter's `collect_missing` returns the set of `MissingField` entries
    /// it cannot synthesize from the supplied slots/descriptor; this variant
    /// transports them to the `user_text()` arm which routes through
    /// `crate::wallet_export::build_missing_fields_refusal` (the sole site of
    /// message construction per §4). Exit 2.
    #[allow(dead_code)] // Phase 0 adds the variant; Phase 1+ emitters return it.
    ExportWalletMissingFields {
        format: &'static str,
        missing: Vec<crate::wallet_export::MissingField>,
    },
    /// SPEC_export_wallet_v0_7.md §3 watch-only refusal — phrase / entropy /
    /// xprv / wif slot supplied to `export-wallet`. Exit 2.
    ExportWalletSecretInput,
    #[allow(dead_code)]
    /// SPEC_export_wallet_v0_7.md §4 — taproot multisig templates
    /// (`tr-multi-a`, `tr-sortedmulti-a`) are not yet supported by
    /// `mnemonic export-wallet` because constructing `tr(<internal-key>,
    /// multi_a(...))` requires picking an internal-key designation (NUMS vs
    /// key-path key); deferred to v0.8. Exit 2. The `&'static str` payload is
    /// the offending template name (`"tr-multi-a"` or `"tr-sortedmulti-a"`).
    ExportWalletTaprootMultisigUnsupported(&'static str),
    /// SPEC cycle-2 H10 — the electrum / coldcard / jade multisig file formats
    /// are BIP-67 sortedmulti-only (no field to express literal `multi(...)`
    /// key order), so exporting an UNSORTED `wsh-multi` / `sh-wsh-multi` to them
    /// would silently coerce to sortedmulti → different witnessScript/address.
    /// Refuse, pointing to a faithful format (descriptor / bitcoin-core /
    /// sparrow). Exit 2. The `format` payload is the offending format name
    /// (`"electrum"` / `"coldcard"` / `"coldcard-multisig"` / `"jade"`).
    ExportWalletUnsortedMultisigUnsupported {
        format: &'static str,
    },
    FutureFormat {
        source: &'static str,
        detail: String,
    },
    /// v0.24.0 §2.C.1 (D34/I5 fold) — a typed `--ms1` / `--mk1` / `--md1`
    /// flag was supplied a value whose HRP prefix does not match the flag's
    /// expected codec. Toolkit-internal validation (not a clap parser
    /// callback) executed at the top of `repair::run` / `inspect::run`.
    /// Distinct from `RepairError::HrpMismatch` (which reports a chunk index
    /// after entering the repair pipeline); this variant reports the
    /// offending flag name so the user sees `--ms1` vs `--mk1` in the error.
    /// Exit 2 (user-input class).
    HrpMismatch {
        flag: &'static str,
        expected: &'static str,
        got: String,
    },
    /// v0.26.0 — sniff returned 0 or ≥2 format matches; user must supply
    /// `--format`. Tier-1 (exit 1). Emitted by `cmd::import_wallet::run` via
    /// `wallet_import::sniff::sniff_format` dispatch (`SniffOutcome::Ambiguous`
    /// / `NoMatch` arms).
    #[allow(dead_code)]
    ImportWalletAmbiguousFormat(String),
    /// v0.26.0 — `--format <X>` supplied but `<X>::sniff` returned false (and
    /// some other parser's sniff matched the blob). Tier-1 (exit 1). Emitted
    /// by `cmd::import_wallet::run` when the user-supplied `--format` conflicts
    /// with the sniff outcome.
    #[allow(dead_code)]
    ImportWalletFormatMismatch {
        supplied: String,
        sniffed: String,
    },
    /// v0.34.6 — `import-wallet --network <X>` requested a network in a
    /// different coin-type class than the imported blob's coin-type-derived
    /// network. The blob's xpub prefix is coin-type-bound (coin-type-1 ↔
    /// testnet/signet/regtest; coin-type-0 ↔ mainnet), so cross-class
    /// re-binding would contradict the key material. Tier-1 (exit 1).
    ImportWalletNetworkClassMismatch {
        requested: String,
        parsed_coin_type: u32,
    },
    /// v0.26.0 wallet-import cycle — blob parse failed (BIP-380 checksum,
    /// header line, descriptor body, JSON shape, etc.). Tier-2 (exit 2) per
    /// SPEC_wallet_import_v0_26_0.md §2.3. Carries an opaque detail message
    /// constructed at the parser site; the rendered template prepends
    /// `error: import-wallet: <format>: parse error:` via `message()`.
    /// Emitted by `wallet_import::bsms::BsmsParser::parse` and
    /// `wallet_import::bitcoin_core::BitcoinCoreParser::parse`.
    #[allow(dead_code)]
    ImportWalletParse(String),
    /// v0.26.0 — `--ms1` / `--slot @N.phrase=` seed overlay supplied entropy
    /// whose derived xpub at the blob-declared origin path does not match the
    /// blob's xpub for the same cosigner. Tier-4 (exit 4); mirrors
    /// `BundleMismatch` semantics. Emitted by `wallet_import::overlay::apply_seed_overlay`.
    #[allow(dead_code)]
    ImportWalletSeedMismatch {
        cosigner_index: usize,
        derived_xpub: String,
        blob_xpub: String,
        path: String,
    },
    /// v0.26.0 — post-parse watch-only invariant violation: a parser produced
    /// a `ParsedImport` whose cosigner at index `usize` carries `Some(entropy)`.
    /// This is an internal-bug guard (every `WalletFormatParser::parse` impl
    /// constructs watch-only cosigners). Tier-2 (exit 2). Mirrors
    /// `ExportWalletSecretInput` discipline. Emitted by
    /// `wallet_import::mod::validate_watch_only_resolved`.
    #[allow(dead_code)]
    ImportWalletWatchOnlyViolation(usize),
    /// v0.26.0 — Bitcoin Core `listdescriptors` returned an xprv-bearing
    /// descriptor (called with the `true` argument). Refuse: re-run with the
    /// xpub-only variant. Tier-2 (exit 2). Emitted by
    /// `wallet_import::bitcoin_core::BitcoinCoreParser::parse`.
    #[allow(dead_code)]
    ImportWalletXprvForbidden,
    /// Exit-code spec R13/§3 test 7: a failure no class covers — a toolkit bug
    /// (a serializer that cannot fail failing, an invariant the code
    /// guarantees broken). Exit 101.
    Internal(String),
    /// v0.22.0 repair feature — std::io::Error from emit_repair_report
    /// writes to stdout/stderr. Exit 32 (2 for bytes that are not UTF-8).
    Io(std::io::Error),
    /// Exit-code spec §3 test 2: something the tool needed from the system
    /// failed or is missing — a file it reads or writes, stdin, stdout, an
    /// unset environment variable, a feature this build lacks. The message
    /// names the flag and the OS error. Exit 32.
    IoFailure(String),
    /// An md1 `descriptor-mnemonic` CARD was passed where a raw descriptor is
    /// expected (`export-wallet`/`bundle` `--descriptor`). md1 is an engraved
    /// card, not a descriptor — decode it first. `surface` names the CLI context.
    Md1CardNotADescriptor {
        surface: &'static str,
    },
    MdCodec(md_codec::Error),
    MkCodec(mk_codec::Error),
    /// A `ModeViolation`-shaped refusal whose decision READ an input (the md1
    /// card, the descriptor), so it is not a usage error (exit-code spec
    /// decision 2): the input is wrong or insufficient for this verb. Same
    /// message as `ModeViolation`. Exit 2.
    ModeInputViolation {
        // mode/flag are read by `details()` for SPEC §5.5 JSON output.
        #[allow(dead_code)]
        mode: &'static str,
        #[allow(dead_code)]
        flag: &'static str,
        message: &'static str,
    },
    ModeViolation {
        // mode/flag are read by `details()` for SPEC §5.5 JSON output (wired in v0.1+ JSON path).
        #[allow(dead_code)]
        mode: &'static str,
        #[allow(dead_code)]
        flag: &'static str,
        message: &'static str,
    },
    MsCodec(ms_codec::Error),
    /// SPEC §6.2 v0.2 multisig configuration error (threshold/cosigner-count
    /// out of range, k > n, etc.). Exit 2.
    #[allow(dead_code)]
    MultisigConfig {
        message: String,
    },
    /// A decoded key's NetworkKind (mainnet vs testnet/signet/regtest)
    /// disagrees with the asserted network (coin-type-derived, `--network`, or
    /// a WIF's own network byte). cycle-5 S-NET: fail-closed network-provenance
    /// invariant, wired at import/convert/export sites via
    /// `network::assert_network_agrees`. Exit 8 (a network mismatch, exit-code spec decision 1).
    NetworkMismatch {
        decoded_network: &'static str,
        expected_network: &'static str,
        context: &'static str,
    },
    /// A nostr key (`npub`/`nsec` NIP-19 bech32 or 64-hex) failed to decode or
    /// validate (bad bech32/HRP/length, not-on-curve x-only, out-of-range scalar).
    NostrKeyParse(String),
    /// Exit-code spec §3 tests 3 and 5: the tool declined under a safety or
    /// policy rule that a flag, network, channel or different destination
    /// would waive, or a network mismatch. Exit 8.
    Refused(String),
    /// v0.75.0 `mnemonic inspect <md1>` — the canonical
    /// `md_codec::descriptor_to_template` renderer (relocated into md-codec
    /// 0.40.0) refused to render the keyless `@N` wallet-policy template for a
    /// decoded md1 `Descriptor`. A SEPARATE type from `md_codec::Error` (a
    /// text-render failure is not a wire/decode error), so it does NOT touch the
    /// exhaustive `md_codec_exit_code` match. The render `Err` arms are
    /// structural guards that never fire on a decoder-produced `Descriptor`;
    /// kept fail-closed. Display-only of PUBLIC descriptor material — no secret
    /// surface. Exit 2 (render/format-refusal class, alongside the codec
    /// decode-rejects). `details()` adds NO arm (`_ => None`).
    Render(md_codec::RenderError),
    /// v0.22.0 repair feature — user-input class (exit 2). Wraps every
    /// `RepairError` variant (EmptyInput / HrpMismatch / TooManyErrors /
    /// UnparseableInput).
    Repair(crate::repair::RepairError),
    /// v0.22.0 repair feature — auto-fire short-circuit signal (exit 5).
    /// Synthesized by `repair::try_repair_and_short_circuit` on
    /// repair-success; `?`-propagated through the helper hierarchy up to
    /// the run() boundary; main.rs special-cases this variant to suppress
    /// the Display impl from writing to stderr (per plan-doc R2 I1 — the
    /// repair report already wrote a clean stderr summary; appending the
    /// Display text would be confusing noise).
    RepairShortCircuit {
        exit_code: u8,
    },
    /// `restore` reference cross-check failed: derived material ≠ supplied
    /// `--expect-*` (or, future, cosigner slot). Exit 1 (a comparison that came out no,
    /// alongside `BundleMismatch`/`ImportWalletSeedMismatch`). Surfaced via
    /// `message()` only (no `details()` JSON-error envelope — SPEC I3).
    RestoreMismatch {
        reference: &'static str,
        derived: String,
        expected: String,
        slot: Option<u8>,
    },
    /// v0.35.0 — `mnemonic silent-payment` BIP-352 receiver-address derivation
    /// failure: a non-seed-bearing secret (WIF/minikey), the reserved `m=0`
    /// change label, or a derivation/label-tweak error. Exit 2 (parse
    /// class, like `NostrKeyParse`).
    SilentPayment(String),
    /// SPEC §6.6 row 4 (conflict) / row 8 (gap) / §6.6.b (invalid subkey set)
    /// `--slot @N.<subkey>=<value>` validation violation at bundle creation.
    /// Exit 2. Wired into `bundle_run` in Phase C.
    #[allow(dead_code)]
    SlotInputViolation {
        /// "conflict" | "gap" | "invalid-set" | "duplicate-subkey" | "language-conflict".
        kind: &'static str,
        message: String,
    },
    /// #28 phase 1 — `bundle --md1-form=template` was requested for a
    /// descriptor shape the keyless single-sig template form does not support:
    /// either `descriptor.n > 1` (multisig) or the shape is not one of the three
    /// origin-elidable single-sig template types (`cli_template_from_tree(&tree)`
    /// is `None` — a non-canonical wrapper, a custom origin, bare wsh, or bip49
    /// nested segwit `sh(wpkh)`). The keyless template MUST be origin-elidable and
    /// single-sig to be byte-shareable. NB (md-codec 0.41.0+): bip49 `sh(wpkh)` is
    /// now `canonical_origin=Some`, so `canonical_origin` alone no longer
    /// classifies it — `cli_template_from_tree` (no `Sh` arm) is the gate that
    /// still refuses it (intentional divergence). Use `--md1-form=policy` for
    /// these shapes. Exit 2.
    TemplateFormUnsupportedShape {
        message: String,
    },
    /// v0.24.0 §2.C.1 (D34/I5 fold) — a positional `<STRING>` argument did
    /// not begin with a recognized HRP prefix (`ms1` / `mk1` / `md1`). Exit 2.
    UnknownHrp {
        got: String,
        expected_one_of: Vec<&'static str>,
    },
    /// Exit-code spec decision 2: the command line alone is wrong (a flag
    /// combination, a missing argument, a bad SETTING such as a count or a
    /// range), judged without reading any input. Nothing ran. Exit 64.
    Usage(String),
    /// v0.36.0 `mnemonic verify-message` — message-signature verification
    /// could not be performed (bad address, undecodable signature, or
    /// `--format legacy` on a non-P2PKH address). Exit 2. A cleanly-decoded
    /// signature that simply does NOT verify is NOT this error — that path
    /// emits a structured `valid:false` result. Public-data, no secrets.
    VerifyMessage(String),
    /// P6 `mnemonic word-card` — the Word-Card value engine (`wc-codec`)
    /// refused to encode or decode. Wraps every [`wc_codec::WcError`] variant
    /// (header-CRC / integrity-tag / RS-budget / sync / RAID refusals, unknown
    /// word, truncation-beyond-repair, invalid params). Word Cards carry PUBLIC
    /// material (xpub / descriptor) — no secrets. Exit 2: every `WcError` is a
    /// format/structural refusal in the same class as the codec decode-rejects
    /// (and the funds-safety nets — `IntegrityMismatch` / `Uncorrectable` —
    /// MUST refuse with a non-zero exit, NEVER return a wrong payload).
    WordCard(wc_codec::WcError),
    /// v0.26.0 `mnemonic xpub-search` — no match found in the searched
    /// candidate set. Exit 1 (sibling to `BundleMismatch` /
    /// `Bip388VerifyDistinctness` — search-target mismatch class).
    /// `mode` distinguishes which xpub-search mode emitted (one of
    /// `"path-of-xpub"`, `"account-of-descriptor"`, `"address-of-xpub"`,
    /// `"passphrase-of-xpub"`); `searched` reports the count of
    /// **candidate-comparisons performed** (work done), not unique
    /// child-addresses derived. Formula:
    ///   - descriptor modes: `paths × accounts × cosigners`
    ///   - address mode: `n_targets × gap_limit × chains` (per-target
    ///     scan over the shared rendered-address Vec; one comparison per
    ///     (target, address) pair)
    ///
    /// The per-target JSON envelope fields `scanned_external` /
    /// `scanned_internal` (on `AddressResultJson::NoMatch` entries inside
    /// `AddressOfXpubResult.results`) report unique child-addresses derived
    /// per-target (i.e., `gap_limit × chains`).
    XpubSearchNoMatch {
        mode: &'static str,
        searched: usize,
    },
    /// `xpub-search passphrase-of-xpub --passphrase-candidates-file`: no
    /// candidate passphrase in the file produced the target xpub. Distinct
    /// from `XpubSearchNoMatch` so the message advises "add more candidates",
    /// not "widen --max-account". `candidates_tried` = #non-blank lines tried.
    XpubSearchPassphraseCandidatesExhausted {
        candidates_tried: usize,
    },
}

/// v0.26.0 — reason discriminant for `ToolkitError::EnvVarMissing`. Drives the
/// two distinct stderr message templates per SPEC_wallet_import_v0_26_0.md §2.4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvVarMissingReason {
    /// The `@env:VAR` syntax was well-formed but the variable is unset.
    Unset,
    /// The variable is set but its value is not valid UTF-8 (F-687 fold 1,
    /// review N1: it used to be reported as "not set").
    NotUnicode,
    /// The `<VAR>` token failed the POSIX env-var-name regex
    /// `[A-Z_][A-Z0-9_]*` (e.g., `@env:foo bar`, `@env:1FOO`, `@env:lowercase`).
    InvalidName,
}

impl From<crate::repair::RepairError> for ToolkitError {
    fn from(e: crate::repair::RepairError) -> Self {
        ToolkitError::Repair(e)
    }
}

#[derive(Debug)]
pub enum BitcoinErrorKind {
    Bip32(bitcoin::bip32::Error),
    #[allow(dead_code)]
    XpubParse(String),
    FingerprintParse(String),
}

/// ms-codec errors by the exit-code spec's §3 tests. `ReservedTagNotEmittedInV01`
/// is intercepted by `From` to `FutureFormat` (2). Every variant this crate's
/// pinned ms-codec has is named; the enum is `#[non_exhaustive]`, so a variant
/// a later ms-codec adds is unclassified until it is named here: 101 (R13).
fn ms_codec_exit_code(e: &ms_codec::Error) -> u8 {
    use crate::exit_code::{INTERNAL, INVALID, USAGE};
    match e {
        // Every decode, format and share-set reject is an input wrong for the
        // verb (test 6), including a combine over an INCONSISTENT share set
        // (cycle-4 M6), which pre-0.5.0 silently produced a wrong secret.
        ms_codec::Error::Codex32(_)
        | ms_codec::Error::MnemUnknownLanguage(_)
        | ms_codec::Error::UnexpectedStringLength { .. }
        | ms_codec::Error::PayloadLengthMismatch { .. }
        | ms_codec::Error::TooManyErrors { .. }
        | ms_codec::Error::WrongHrp { .. }
        | ms_codec::Error::ThresholdNotZero { .. }
        | ms_codec::Error::ShareIndexNotSecret { .. }
        | ms_codec::Error::TagInvalidAlphabet { .. }
        | ms_codec::Error::UnknownTag { .. }
        | ms_codec::Error::ReservedTagNotEmittedInV01 { .. }
        | ms_codec::Error::ReservedPrefixViolation { .. }
        | ms_codec::Error::IsShareNotSingleString { .. }
        | ms_codec::Error::SecretShareSuppliedToCombine
        | ms_codec::Error::InconsistentShareSet => INVALID,
        // A split's K and N are SETTINGS (decision 2): judged from the command
        // line alone, so a bad one is a usage error.
        ms_codec::Error::InvalidThreshold(_) | ms_codec::Error::InvalidShareCount { .. } => USAGE,
        _ => INTERNAL,
    }
}

/// mk-codec errors by §3. `UnsupportedVersion` is intercepted by `From` to
/// `FutureFormat` (2). Every variant is an input wrong for the verb (test 6);
/// a variant a later mk-codec adds is 101 until it is named here (R13).
fn mk_codec_exit_code(e: &mk_codec::Error) -> u8 {
    use crate::exit_code::{INTERNAL, INVALID};
    match e {
        mk_codec::Error::InvalidStringLength(_)
        | mk_codec::Error::InvalidChar { .. }
        | mk_codec::Error::BchUncorrectable(_)
        | mk_codec::Error::InvalidHrp(_)
        | mk_codec::Error::MixedCase
        | mk_codec::Error::UnsupportedCardType(_)
        | mk_codec::Error::MalformedPayloadPadding
        | mk_codec::Error::ChunkSetIdMismatch
        | mk_codec::Error::ChunkedHeaderMalformed(_)
        | mk_codec::Error::MixedHeaderTypes
        | mk_codec::Error::CrossChunkHashMismatch
        | mk_codec::Error::UnsupportedVersion(_)
        | mk_codec::Error::ReservedBitsSet
        | mk_codec::Error::InvalidPolicyIdStubCount
        | mk_codec::Error::InvalidPathIndicator(_)
        | mk_codec::Error::PathTooDeep(_)
        | mk_codec::Error::InvalidPathComponent(_)
        | mk_codec::Error::InvalidXpubVersion(_)
        | mk_codec::Error::InvalidXpubPublicKey(_)
        | mk_codec::Error::UnexpectedEnd
        | mk_codec::Error::TrailingBytes
        | mk_codec::Error::CardPayloadTooLarge { .. }
        | mk_codec::Error::XpubOriginPathMismatch { .. } => INVALID,
        _ => INTERNAL,
    }
}

/// md-codec errors by §3. md_codec::Error is NOT `#[non_exhaustive]`, so this
/// match is exhaustive and a new md-codec variant fails the build until it is
/// classified. Every variant is a card, descriptor or value wrong for the verb
/// (test 6): checksum and BCH failures, structural and TLV rejects, the
/// funds-safety rejects detectable from the card alone (DuplicateKeySlots,
/// OriginKeyContradiction, RelativeTimelockTruncated, the Unspendable*
/// refusals), chunk-set rejects, the encode-side length caps, and a newer wire
/// version (intercepted by `From` to `FutureFormat`, also 2).
fn md_codec_exit_code(e: &md_codec::Error) -> u8 {
    use crate::exit_code::INVALID;
    match e {
        md_codec::Error::Codex32DecodeError(_)
        | md_codec::Error::Codex32EncodeError(_)
        | md_codec::Error::BitStreamTruncated { .. }
        | md_codec::Error::MalformedHeader { .. }
        | md_codec::Error::PathDepthExceeded { .. }
        | md_codec::Error::KeyCountOutOfRange { .. }
        | md_codec::Error::DivergentPathCountMismatch { .. }
        | md_codec::Error::AltCountOutOfRange { .. }
        | md_codec::Error::TagOutOfRange { .. }
        | md_codec::Error::ThresholdOutOfRange { .. }
        | md_codec::Error::ChildCountOutOfRange { .. }
        | md_codec::Error::KGreaterThanN { .. }
        | md_codec::Error::TlvOrderingViolation { .. }
        | md_codec::Error::TlvUnknownTagOutOfRange { .. }
        | md_codec::Error::PlaceholderIndexOutOfRange { .. }
        | md_codec::Error::OverrideOrderViolation { .. }
        | md_codec::Error::BaselineUseSiteOverride { .. }
        | md_codec::Error::RedundantUseSiteOverride { .. }
        | md_codec::Error::EmptyTlvEntry { .. }
        | md_codec::Error::TlvLengthExceedsRemaining { .. }
        | md_codec::Error::PlaceholderNotReferenced { .. }
        | md_codec::Error::PlaceholderFirstOccurrenceOutOfOrder { .. }
        | md_codec::Error::MultipathAltCountMismatch { .. }
        | md_codec::Error::ForbiddenTapTreeLeaf { .. }
        | md_codec::Error::OperatorContextViolation { .. }
        | md_codec::Error::DuplicateKeySlots { .. }
        | md_codec::Error::OriginKeyContradiction { .. }
        | md_codec::Error::RelativeTimelockTruncated { .. }
        | md_codec::Error::ChunkCountOutOfRange { .. }
        | md_codec::Error::ChunkIndexOutOfRange { .. }
        | md_codec::Error::ChunkSetIdOutOfRange { .. }
        | md_codec::Error::ChunkHeaderChunkedFlagMissing
        | md_codec::Error::ChunkCountExceedsMax { .. }
        | md_codec::Error::ChunkSetEmpty
        | md_codec::Error::ChunkSetInconsistent
        | md_codec::Error::ChunkSetIncomplete { .. }
        | md_codec::Error::ChunkIndexGap { .. }
        | md_codec::Error::ChunkSetIdMismatch { .. }
        | md_codec::Error::VarintOverflow { .. }
        | md_codec::Error::MissingExplicitOrigin { .. }
        | md_codec::Error::EmptyOriginOverride { .. }
        | md_codec::Error::InvalidPresenceByte { .. }
        | md_codec::Error::InvalidXpubBytes { .. }
        | md_codec::Error::MissingPubkey { .. }
        | md_codec::Error::ChainIndexOutOfRange { .. }
        | md_codec::Error::HardenedPublicDerivation
        | md_codec::Error::AddressDerivationFailed { .. }
        | md_codec::Error::NUMSSentinelConflict
        | md_codec::Error::DecodeRecursionDepthExceeded { .. }
        | md_codec::Error::TooManyErrors { .. }
        | md_codec::Error::MalformedPayloadPadding { .. }
        | md_codec::Error::PayloadTooLongForSingleString { .. }
        | md_codec::Error::ChunkSymbolCountOutOfRange { .. }
        | md_codec::Error::StringSymbolCountOutOfRange { .. }
        | md_codec::Error::NetworkRequiredForUnspendable
        | md_codec::Error::NonMinimalWireVersion { .. }
        | md_codec::Error::UnspendableNotRootTr
        | md_codec::Error::UnspendableUseSiteNotCanonical { .. }
        | md_codec::Error::UnspendableWithSortedMultiA
        | md_codec::Error::WireVersionMismatch { .. } => INVALID,
    }
}

impl ToolkitError {
    /// The process exit status for this error, by the ordered tests of
    /// `SPEC_constellation_exit_codes.md` §3 (mnemonic-engrave `design/`).
    /// Every variant is classified here and pinned by the R13 table test
    /// `exit_code_every_variant_has_its_class`. A verb that prints a result
    /// despite a failure returns its bit sum as `Ok(code)` instead.
    pub fn exit_code(&self) -> u8 {
        use crate::exit_code::{INTERNAL, INVALID, IO, NO, REFUSED, USAGE};
        match self {
            // Mostly a malformed or insufficient input value (test 6). The
            // flag-combination, file and network cases that were BadInput
            // moved to `Usage`, `IoFailure` and `Refused`.
            ToolkitError::BadInput(_) => INVALID,
            ToolkitError::Bip39(_) => INVALID,
            ToolkitError::Bip388Distinctness { .. } => INVALID,
            // §3 worked table: "a BIP-388 distinctness violation in verify" is 2.
            ToolkitError::Bip388VerifyDistinctness => INVALID,
            ToolkitError::Bitcoin(_) => INVALID,
            // Decision 11 / §7f: the strict BSMS MAC and signature mismatches
            // are comparisons that came out no (test 4).
            ToolkitError::BsmsMacMismatch { .. } => NO,
            ToolkitError::BsmsRound1Malformed { .. } => INVALID,
            ToolkitError::BsmsSignatureMismatch { .. } => NO,
            ToolkitError::BsmsTaprootImportRefused => INVALID,
            ToolkitError::BsmsTaprootRefused { .. } => INVALID,
            ToolkitError::BuildDescriptorSpec(_) => INVALID,
            ToolkitError::BundleMismatch { .. } => NO,
            ToolkitError::CompareCost(e) => e.exit_code(),
            ToolkitError::ConvertRefusal(_) => INVALID,
            ToolkitError::CosignerSpec { .. } => INVALID,
            ToolkitError::CosignersFile { .. } => INVALID,
            ToolkitError::DecodeAddress(_) => INVALID,
            // `--length` is a SETTING (the output's size), judged from argv.
            ToolkitError::DeriveChildLengthNotApplicable => USAGE,
            ToolkitError::DeriveChildLengthOutOfRange { .. } => USAGE,
            // §3 worked table: a valid name the tool will not do; no flag waives it.
            ToolkitError::DeriveChildUnsupportedApp => INVALID,
            ToolkitError::DescriptorParse(_) => INVALID,
            // Believed unreachable: a failure would be toolkit-internal parse
            // non-determinism, a bug.
            ToolkitError::DescriptorReparseFailed { .. } => INTERNAL,
            // §3 worked table: `@env:NAME` with NAME unset is 32. A NAME that is
            // not a valid variable name is judged from argv (64); a value that
            // is not UTF-8 is a malformed input (2).
            ToolkitError::EnvVarMissing { reason, .. } => match reason {
                EnvVarMissingReason::Unset => IO,
                EnvVarMissingReason::InvalidName => USAGE,
                EnvVarMissingReason::NotUnicode => INVALID,
            },
            ToolkitError::ExportWalletFormatStub(_) => INVALID,
            ToolkitError::ExportWalletMissingFields { .. } => INVALID,
            // §7f: export is watch-only by definition, so no flag clears it.
            ToolkitError::ExportWalletSecretInput => INVALID,
            ToolkitError::ExportWalletTaprootMultisigUnsupported(_) => INVALID,
            ToolkitError::ExportWalletUnsortedMultisigUnsupported { .. } => INVALID,
            // §7f: a newer wire version is an invalid input (was 3).
            ToolkitError::FutureFormat { .. } => INVALID,
            ToolkitError::HrpMismatch { .. } => INVALID,
            ToolkitError::ImportWalletAmbiguousFormat(_) => INVALID,
            ToolkitError::ImportWalletFormatMismatch { .. } => INVALID,
            // A network mismatch (test 3).
            ToolkitError::ImportWalletNetworkClassMismatch { .. } => REFUSED,
            ToolkitError::ImportWalletParse(_) => INVALID,
            ToolkitError::ImportWalletSeedMismatch { .. } => NO,
            // "watch-only invariant violated (internal bug)".
            ToolkitError::ImportWalletWatchOnlyViolation(_) => INTERNAL,
            ToolkitError::ImportWalletXprvForbidden => INVALID,
            ToolkitError::Internal(_) => INTERNAL,
            // A read whose bytes are not UTF-8 is a malformed input, not a
            // system failure.
            ToolkitError::Io(e) if e.kind() == std::io::ErrorKind::InvalidData => INVALID,
            ToolkitError::Io(_) => IO,
            ToolkitError::IoFailure(_) => IO,
            ToolkitError::Md1CardNotADescriptor { .. } => INVALID,
            ToolkitError::MdCodec(e) => md_codec_exit_code(e),
            ToolkitError::MkCodec(e) => mk_codec_exit_code(e),
            ToolkitError::ModeInputViolation { .. } => INVALID,
            ToolkitError::ModeViolation { .. } => USAGE,
            ToolkitError::MsCodec(e) => ms_codec_exit_code(e),
            ToolkitError::MultisigConfig { .. } => INVALID,
            ToolkitError::NetworkMismatch { .. } => REFUSED,
            ToolkitError::NostrKeyParse(_) => INVALID,
            ToolkitError::Refused(_) => REFUSED,
            ToolkitError::Render(_) => INVALID,
            ToolkitError::Repair(_) => INVALID,
            ToolkitError::RepairShortCircuit { exit_code } => *exit_code,
            ToolkitError::RestoreMismatch { .. } => NO,
            ToolkitError::SilentPayment(_) => INVALID,
            ToolkitError::SlotInputViolation { .. } => INVALID,
            ToolkitError::TemplateFormUnsupportedShape { .. } => INVALID,
            ToolkitError::UnknownHrp { .. } => INVALID,
            ToolkitError::Usage(_) => USAGE,
            ToolkitError::VerifyMessage(_) => INVALID,
            ToolkitError::WordCard(_) => INVALID,
            ToolkitError::XpubSearchNoMatch { .. } => NO,
            ToolkitError::XpubSearchPassphraseCandidatesExhausted { .. } => NO,
        }
    }

    /// Stable discriminant for JSON `kind` field (SPEC §5.5).
    /// Reserved for the §5.5 JSON-error envelope path (covered by tests in v0.1).
    #[allow(dead_code)]
    pub fn kind(&self) -> &'static str {
        match self {
            ToolkitError::BadInput(_) => "BadInput",
            ToolkitError::Bip39(_) => "Bip39",
            ToolkitError::Bip388Distinctness { .. } => "Bip388Distinctness",
            ToolkitError::Bip388VerifyDistinctness => "Bip388VerifyDistinctness",
            ToolkitError::Bitcoin(_) => "Bitcoin",
            ToolkitError::BsmsMacMismatch { .. } => "BsmsMacMismatch",
            ToolkitError::BsmsRound1Malformed { .. } => "BsmsRound1Malformed",
            ToolkitError::BsmsSignatureMismatch { .. } => "BsmsSignatureMismatch",
            ToolkitError::BsmsTaprootImportRefused => "BsmsTaprootImportRefused",
            ToolkitError::BsmsTaprootRefused { .. } => "BsmsTaprootRefused",
            ToolkitError::BuildDescriptorSpec(_) => "BuildDescriptorSpec",
            ToolkitError::BundleMismatch { .. } => "BundleMismatch",
            ToolkitError::CompareCost(_) => "CompareCost",
            ToolkitError::ConvertRefusal(_) => "ConvertRefusal",
            ToolkitError::CosignerSpec { .. } => "CosignerSpec",
            ToolkitError::CosignersFile { .. } => "CosignersFile",
            ToolkitError::DecodeAddress(_) => "DecodeAddress",
            ToolkitError::DeriveChildLengthNotApplicable => "DeriveChildLengthNotApplicable",
            ToolkitError::DeriveChildLengthOutOfRange { .. } => "DeriveChildLengthOutOfRange",
            ToolkitError::DeriveChildUnsupportedApp => "DeriveChildUnsupportedApp",
            ToolkitError::DescriptorParse(_) => "DescriptorParse",
            ToolkitError::DescriptorReparseFailed { .. } => "DescriptorReparseFailed",
            ToolkitError::EnvVarMissing { .. } => "EnvVarMissing",
            ToolkitError::ExportWalletFormatStub(_) => "ExportWalletFormatStub",
            ToolkitError::ExportWalletMissingFields { .. } => "ExportWalletMissingFields",
            ToolkitError::ExportWalletSecretInput => "ExportWalletSecretInput",
            ToolkitError::ExportWalletTaprootMultisigUnsupported(_) => {
                "ExportWalletTaprootMultisigUnsupported"
            }
            ToolkitError::ExportWalletUnsortedMultisigUnsupported { .. } => {
                "ExportWalletUnsortedMultisigUnsupported"
            }
            ToolkitError::FutureFormat { .. } => "FutureFormat",
            ToolkitError::HrpMismatch { .. } => "HrpMismatch",
            ToolkitError::ImportWalletAmbiguousFormat(_) => "ImportWalletAmbiguousFormat",
            ToolkitError::ImportWalletFormatMismatch { .. } => "ImportWalletFormatMismatch",
            ToolkitError::ImportWalletNetworkClassMismatch { .. } => {
                "ImportWalletNetworkClassMismatch"
            }
            ToolkitError::ImportWalletParse(_) => "ImportWalletParse",
            ToolkitError::ImportWalletSeedMismatch { .. } => "ImportWalletSeedMismatch",
            ToolkitError::ImportWalletWatchOnlyViolation(_) => "ImportWalletWatchOnlyViolation",
            ToolkitError::ImportWalletXprvForbidden => "ImportWalletXprvForbidden",
            ToolkitError::Internal(_) => "Internal",
            ToolkitError::Io(_) => "Io",
            ToolkitError::IoFailure(_) => "IoFailure",
            ToolkitError::Md1CardNotADescriptor { .. } => "Md1CardNotADescriptor",
            ToolkitError::MdCodec(_) => "MdCodec",
            ToolkitError::MkCodec(_) => "MkCodec",
            ToolkitError::ModeInputViolation { .. } => "ModeInputViolation",
            ToolkitError::ModeViolation { .. } => "ModeViolation",
            ToolkitError::MsCodec(_) => "MsCodec",
            ToolkitError::MultisigConfig { .. } => "MultisigConfig",
            ToolkitError::NetworkMismatch { .. } => "NetworkMismatch",
            ToolkitError::NostrKeyParse(_) => "NostrKeyParse",
            ToolkitError::Refused(_) => "Refused",
            ToolkitError::Render(_) => "Render",
            ToolkitError::Repair(_) => "Repair",
            ToolkitError::RepairShortCircuit { .. } => "RepairShortCircuit",
            ToolkitError::RestoreMismatch { .. } => "RestoreMismatch",
            ToolkitError::SilentPayment(_) => "SilentPayment",
            ToolkitError::SlotInputViolation { .. } => "SlotInputViolation",
            ToolkitError::TemplateFormUnsupportedShape { .. } => "TemplateFormUnsupportedShape",
            ToolkitError::UnknownHrp { .. } => "UnknownHrp",
            ToolkitError::Usage(_) => "Usage",
            ToolkitError::VerifyMessage(_) => "VerifyMessage",
            ToolkitError::WordCard(_) => "WordCard",
            ToolkitError::XpubSearchNoMatch { .. } => "XpubSearchNoMatch",
            ToolkitError::XpubSearchPassphraseCandidatesExhausted { .. } => {
                "XpubSearchPassphraseCandidatesExhausted"
            }
        }
    }

    /// Friendly human-readable message. Five sibling-source mappers live in
    /// `friendly.rs` (Phase 3 task 3.3) and are dispatched here.
    pub fn message(&self) -> String {
        match self {
            ToolkitError::BadInput(m) => m.clone(),
            ToolkitError::Bip39(e) => crate::friendly::friendly_bip39(e),
            ToolkitError::Bip388Distinctness { i, j } => {
                format!("BIP-388 distinct-key violation: slot @{i} and slot @{j} resolve to identical (xpub, path)")
            }
            ToolkitError::Bip388VerifyDistinctness => {
                "bundle violates BIP-388 distinct-key rule; regenerate with distinct keys".to_string()
            }
            ToolkitError::Bitcoin(e) => crate::friendly::friendly_bitcoin(e),
            ToolkitError::BsmsMacMismatch { token_len_hex } => format!(
                "import-wallet: bsms: BIP-129 MAC verification failed (token width {token_len_hex} hex chars; wrong token or tampered ciphertext)"
            ),
            ToolkitError::BsmsRound1Malformed { reason } => format!(
                "import-wallet: --bsms-round1: BIP-129 Round-1 record malformed: {reason}"
            ),
            ToolkitError::BsmsSignatureMismatch {
                record_index,
                signer_pubkey,
                reason,
            } => format!(
                "import-wallet: --bsms-round1: BIP-129 signature verification failed for \
                 record {record_index} (signer pubkey {signer_pubkey}): {reason}"
            ),
            // v0.28.7 — import-side taproot refusal parity with BsmsTaprootRefused.
            ToolkitError::BsmsTaprootImportRefused => {
                "--format bsms does not support taproot import; BIP-129 §1 prerequisites \
                 do not yet include BIP-386. Real import support is tracked at FOLLOWUP \
                 `bsms-import-taproot-refusal-parity` (resolved v0.28.7). Use \
                 --format bitcoin-core (Core-importable) or --format sparrow \
                 (Sparrow JSON, taproot-capable) for taproot watch-only setup."
                    .to_string()
            }
            // v0.28.0 P8B (plan-doc §S.8) — tightened BSMS taproot refusal.
            // Replaces the v0.27.0 `ToolkitError::BadInput("--format bsms does
            // not support taproot descriptors; ...")` text. The per-script-type
            // discriminator (P2tr / P2trMulti) lets users see which template
            // class their input resolved to; the FOLLOWUP slug pointer
            // (`bsms-taproot-emit`) lets users tracking upstream BIP-129 work
            // grep the toolkit's tracker quickly; the alternative-format
            // pointers ({bitcoin-core, sparrow}) match the prior text's hints.
            ToolkitError::BsmsTaprootRefused { script_type } => format!(
                "--format bsms does not support taproot ({}); BIP-129 §1 prerequisites \
                 do not yet include BIP-386. Real emit support is tracked at FOLLOWUP \
                 `bsms-taproot-emit` and depends on a BIP-129 spec update. Use \
                 --format bitcoin-core (Core-importable) or --format sparrow \
                 (Sparrow JSON, taproot-capable) for taproot watch-only setup.",
                crate::wallet_export::script_type_short_name(script_type)
            ),
            ToolkitError::BuildDescriptorSpec(m) => m.clone(),
            ToolkitError::BundleMismatch { card, message } => {
                format!("bundle mismatch on {}: {}; if the engraved bundle was produced at a non-zero BIP-32 account, pass --account <N> to match (default 0)",
                    card, message)
            }
            ToolkitError::CompareCost(e) => format!("{e}"),
            ToolkitError::ConvertRefusal(m) => m.clone(),
            ToolkitError::CosignerSpec {
                cosigner_idx,
                message,
            } => format!("--cosigner[{}]: {}", cosigner_idx, message),
            ToolkitError::CosignersFile { message } => {
                format!("--cosigners-file: {}", message)
            }
            ToolkitError::DecodeAddress(m) => format!("decode-address: {m}"),
            ToolkitError::DeriveChildLengthNotApplicable => {
                "--length not applicable for --application <hd-seed|xprv> (output is fixed-size)"
                    .to_string()
            }
            ToolkitError::DeriveChildLengthOutOfRange {
                app,
                length,
                valid_text,
            } => format!(
                "--length {length} out of range for --application {app} (valid: {valid_text})",
            ),
            ToolkitError::DeriveChildUnsupportedApp => {
                // SPEC_derive_child_v0_8.md §7 byte-exact stderr text. v0.8
                // lifts `dice` to in-scope; `rsa` and `rsa-gpg` remain deferred
                // per Phase 6 RSA-crate security spike (RUSTSEC-2023-0071
                // unpatched as of 2026-05-07).
                "--application <rsa|rsa-gpg> is out-of-scope: the rsa crate has unpatched \
                 timing-attack advisory RUSTSEC-2023-0071 and BIP-85 RSA / RSA-GPG demand is \
                 limited; deferred pending crate stability + user demand."
                    .to_string()
            }
            ToolkitError::DescriptorParse(m) => m.clone(),
            ToolkitError::DescriptorReparseFailed { detail } => {
                format!("descriptor re-parse failed during verify-bundle: {detail}")
            }
            ToolkitError::EnvVarMissing { flag, var, reason } => match reason {
                EnvVarMissingReason::Unset => format!(
                    "{flag}: env-var {var} referenced by sentinel is not set"
                ),
                EnvVarMissingReason::NotUnicode => format!(
                    "{flag}: env-var {var} referenced by sentinel is set but not valid UTF-8"
                ),
                EnvVarMissingReason::InvalidName => {
                    format!("{flag}: invalid env-var name `{var}`")
                }
            },
            ToolkitError::ExportWalletFormatStub(name) => crate::wallet_export::format_stub_message(name),
            ToolkitError::ExportWalletMissingFields { format, missing } => {
                crate::wallet_export::build_missing_fields_refusal(format, missing)
            }
            ToolkitError::ExportWalletSecretInput => crate::wallet_export::REFUSAL_SECRET_INPUT.to_string(),
            ToolkitError::ExportWalletTaprootMultisigUnsupported(name) => {
                crate::wallet_export::taproot_multisig_unsupported_message(name)
            }
            ToolkitError::ExportWalletUnsortedMultisigUnsupported { format } => format!(
                "--format {format} cannot faithfully export an UNSORTED multisig (wsh-multi / sh-wsh-multi): the {format} multisig file format is BIP-67 sortedmulti-only and would silently reorder the keys, changing the witnessScript and every address. Use --format descriptor, --format bitcoin-core, or --format sparrow (which preserve literal multi(...) key order), or use a sortedmulti template if BIP-67 ordering is intended."
            ),
            ToolkitError::FutureFormat { source, detail } => format!(
                "{} reserved-not-emitted: {}; deferred to v0.2+",
                source, detail,
            ),
            ToolkitError::HrpMismatch { flag, expected, got } => {
                format!(
                    "{flag} expects a value with HRP '{expected}', got '{got}' \
                     (HRP is not BCH-protected; re-type the prefix)"
                )
            }
            ToolkitError::ImportWalletAmbiguousFormat(detail) => detail.clone(),
            ToolkitError::ImportWalletFormatMismatch { supplied, sniffed } => format!(
                "import-wallet: --format {supplied} supplied but blob looks like {sniffed}"
            ),
            ToolkitError::ImportWalletNetworkClassMismatch { requested, parsed_coin_type } => format!(
                "import-wallet: --network {requested} is incompatible with the imported \
                 wallet's coin-type-{parsed_coin_type} network. The blob's xpub prefix is \
                 coin-type-bound (coin-type-1 ↔ testnet/signet/regtest; coin-type-0 ↔ mainnet); \
                 omit --network to use the coin-type-derived network."
            ),
            ToolkitError::ImportWalletParse(detail) => detail.clone(),
            ToolkitError::ImportWalletSeedMismatch {
                cosigner_index,
                derived_xpub,
                blob_xpub,
                path,
            } => format!(
                "import-wallet: cosigner {cosigner_index}: supplied seed produces \
                 xpub {derived_xpub} at path {path}; blob declares {blob_xpub}"
            ),
            ToolkitError::ImportWalletWatchOnlyViolation(i) => format!(
                "import-wallet: cosigner {i} has entropy populated post-parse; \
                 watch-only invariant violated (internal bug)"
            ),
            ToolkitError::ImportWalletXprvForbidden => {
                "import-wallet: bitcoin-core: xprv-bearing descriptor refused; \
                 re-run `bitcoin-cli listdescriptors` without `true` to get \
                 xpub-only output"
                    .to_string()
            }
            ToolkitError::Internal(m) => format!("internal error (a bug; please report it): {m}"),
            ToolkitError::Io(e) => format!("I/O error: {e}"),
            ToolkitError::IoFailure(m) => m.clone(),
            ToolkitError::Md1CardNotADescriptor { surface } => format!(
                "{surface}: this is an md1 descriptor-mnemonic CARD, not a raw descriptor. \
                 md1 cards are not accepted on --descriptor. Decode the card first \
                 (`md decode <md1…>` / `mnemonic restore --md1 <md1…>`), or to search an \
                 xpub from the card use \
                 `mnemonic xpub-search account-of-descriptor --descriptor <md1…>`."
            ),
            ToolkitError::MdCodec(e) => crate::friendly::friendly_md_codec(e),
            ToolkitError::MkCodec(e) => crate::friendly::friendly_mk_codec(e),
            ToolkitError::ModeInputViolation { message, .. } => (*message).to_owned(),
            ToolkitError::ModeViolation { message, .. } => (*message).to_owned(),
            ToolkitError::MsCodec(e) => crate::friendly::friendly_ms_codec(e),
            ToolkitError::MultisigConfig { message } => message.clone(),
            ToolkitError::NetworkMismatch {
                decoded_network,
                expected_network,
                context,
            } => format!(
                "network mismatch: {context}: key encodes {decoded_network} but \
                 {expected_network} was asserted (xpub version bytes distinguish \
                 mainnet vs testnet/signet/regtest only)"
            ),
            ToolkitError::NostrKeyParse(msg) => format!("nostr: {msg}"),
            ToolkitError::Refused(m) => m.clone(),
            // `RenderError`'s Display already carries a structural-guard message
            // ("malformed descriptor tree: …"); surface it via the source.
            ToolkitError::Render(e) => format!("inspect: template render failed: {e}"),
            ToolkitError::Repair(e) => format!("{e}"),
            ToolkitError::RepairShortCircuit { .. } => {
                // R2 I1: main.rs special-cases this variant to skip
                // writing this message to stderr (the repair report
                // already emitted its own clean stderr summary).
                String::new()
            }
            ToolkitError::RestoreMismatch {
                reference,
                derived,
                expected,
                slot,
            } => format!(
                "restore: {reference} mismatch{} — derived {derived}, expected {expected}",
                slot.map(|s| format!(" at slot @{s}")).unwrap_or_default()
            ),
            ToolkitError::SilentPayment(msg) => format!("silent-payment: {msg}"),
            ToolkitError::SlotInputViolation { message, .. } => message.clone(),
            ToolkitError::TemplateFormUnsupportedShape { message } => message.clone(),
            ToolkitError::UnknownHrp { got, expected_one_of } => {
                // v0.53.3 (audit M11 rider): truncate the echo to the FIRST
                // 12 chars + `…` (only when longer) — a near-miss secret-ish
                // positional must never appear in full on stderr.
                let shown = if got.chars().count() > 12 {
                    let head: String = got.chars().take(12).collect();
                    format!("{head}…")
                } else {
                    got.clone()
                };
                format!(
                    "positional argument '{shown}' does not begin with a recognized \
                     HRP prefix (expected one of: {})",
                    expected_one_of.join(", ")
                )
            }
            ToolkitError::Usage(m) => m.clone(),
            ToolkitError::VerifyMessage(m) => format!("verify-message: {m}"),
            // `WcError`'s own Display already carries the `word-card: …` prefix
            // and a refuse/repair-aware message; surface it verbatim.
            ToolkitError::WordCard(e) => format!("{e}"),
            ToolkitError::XpubSearchNoMatch { mode, searched } => format!(
                "no match in searched set: mode={mode}, paths searched={searched}; \
                 widen the range with --max-account / --number-of-accounts, or supply \
                 additional templates via --add-path"
            ),
            ToolkitError::XpubSearchPassphraseCandidatesExhausted { candidates_tried } => {
                if *candidates_tried == 0 {
                    "no candidates in --passphrase-candidates-file (all lines blank); \
                     supply one passphrase candidate per line"
                        .to_string()
                } else {
                    format!(
                        "no candidate in --passphrase-candidates-file produced the target \
                         xpub ({candidates_tried} candidate(s) tried); verify the seed and \
                         --target-xpub, or add more candidates"
                    )
                }
            }
        }
    }

    /// JSON `details` field (SPEC §5.5).
    /// Reserved for the §5.5 JSON-error envelope path.
    #[allow(dead_code)]
    pub fn details(&self) -> Option<serde_json::Value> {
        match self {
            ToolkitError::Bip388Distinctness { i, j } => Some(json!({ "i": i, "j": j })),
            ToolkitError::BundleMismatch { card, .. } => Some(json!({ "card": card })),
            ToolkitError::CosignerSpec { cosigner_idx, .. } => Some(json!({
                "cosigner_idx": cosigner_idx,
            })),
            ToolkitError::FutureFormat { source, detail } => Some(json!({
                "source": source,
                "detail": detail,
            })),
            ToolkitError::ModeInputViolation { mode, flag, .. }
            | ToolkitError::ModeViolation { mode, flag, .. } => Some(json!({
                "mode": mode,
                "flag": flag,
            })),
            ToolkitError::NetworkMismatch {
                decoded_network,
                expected_network,
                context,
            } => Some(json!({
                "decoded_network": decoded_network,
                "expected_network": expected_network,
                "context": context,
            })),
            ToolkitError::SlotInputViolation { kind, .. } => Some(json!({ "kind": kind })),
            _ => None,
        }
    }
}

impl std::fmt::Display for ToolkitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "error: {}", self.message())
    }
}

impl std::error::Error for ToolkitError {}

impl From<bip39::Error> for ToolkitError {
    fn from(e: bip39::Error) -> Self {
        ToolkitError::Bip39(e)
    }
}

impl From<bitcoin::bip32::Error> for ToolkitError {
    fn from(e: bitcoin::bip32::Error) -> Self {
        ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e))
    }
}

impl From<ms_codec::Error> for ToolkitError {
    fn from(e: ms_codec::Error) -> Self {
        match e {
            ms_codec::Error::ReservedTagNotEmittedInV01 { got } => ToolkitError::FutureFormat {
                source: "ms_codec",
                detail: format!(
                    "reserved tag {:?}",
                    std::str::from_utf8(&got).unwrap_or("<non-utf8>")
                ),
            },
            other => ToolkitError::MsCodec(other),
        }
    }
}

impl From<mk_codec::Error> for ToolkitError {
    fn from(e: mk_codec::Error) -> Self {
        match e {
            mk_codec::Error::UnsupportedVersion(v) => ToolkitError::FutureFormat {
                source: "mk_codec",
                detail: format!("unsupported version {}", v),
            },
            other => ToolkitError::MkCodec(other),
        }
    }
}

impl From<md_codec::Error> for ToolkitError {
    fn from(e: md_codec::Error) -> Self {
        match e {
            // v0.30 renamed UnsupportedVersion -> WireVersionMismatch.
            // Routed to FutureFormat (exit 2); user-facing detail message kept as
            // "unsupported version N" for CLI message stability.
            md_codec::Error::WireVersionMismatch { got } => ToolkitError::FutureFormat {
                source: "md_codec",
                detail: format!("unsupported version {}", got),
            },
            other => ToolkitError::MdCodec(other),
        }
    }
}

impl From<md_codec::RenderError> for ToolkitError {
    fn from(e: md_codec::RenderError) -> Self {
        ToolkitError::Render(e)
    }
}

impl From<wc_codec::WcError> for ToolkitError {
    fn from(e: wc_codec::WcError) -> Self {
        ToolkitError::WordCard(e)
    }
}

/// Convenience alias; exported for downstream-crate use.
#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, ToolkitError>;

#[cfg(test)]
mod tests {
    use super::*;

    /// R13 (exit-code spec): every variant maps to its §3 class. The
    /// exhaustive `match` in `sample_count_is_every_variant` fails to compile
    /// when a variant is added, so a new one cannot skip this table.
    fn class_table() -> Vec<(ToolkitError, u8)> {
        use crate::exit_code::{INTERNAL, INVALID, IO, NO, REFUSED, USAGE};
        vec![
            (ToolkitError::BadInput("x".into()), INVALID),
            (ToolkitError::Bip39(bip39::Error::BadWordCount(5)), INVALID),
            (ToolkitError::Bip388Distinctness { i: 0, j: 1 }, INVALID),
            (ToolkitError::Bip388VerifyDistinctness, INVALID),
            (
                ToolkitError::Bitcoin(BitcoinErrorKind::FingerprintParse("zz".into())),
                INVALID,
            ),
            (ToolkitError::BsmsMacMismatch { token_len_hex: 16 }, NO),
            (
                ToolkitError::BsmsRound1Malformed { reason: "x".into() },
                INVALID,
            ),
            (
                ToolkitError::BsmsSignatureMismatch {
                    record_index: 0,
                    signer_pubkey: "x".into(),
                    reason: "x".into(),
                },
                NO,
            ),
            (ToolkitError::BsmsTaprootImportRefused, INVALID),
            (
                ToolkitError::BsmsTaprootRefused {
                    script_type: crate::wallet_export::WalletScriptType::P2tr,
                },
                INVALID,
            ),
            (ToolkitError::BuildDescriptorSpec("x".into()), INVALID),
            (
                ToolkitError::BundleMismatch {
                    card: "mk1".into(),
                    message: "x".into(),
                },
                NO,
            ),
            (
                ToolkitError::CompareCost(crate::cost::CompareCostError::Parse("x".into())),
                INVALID,
            ),
            (
                ToolkitError::CompareCost(crate::cost::CompareCostError::ConditionsTooMany {
                    raw: 9,
                    cap: 1,
                }),
                REFUSED,
            ),
            (ToolkitError::ConvertRefusal("x".into()), INVALID),
            (
                ToolkitError::CosignerSpec {
                    cosigner_idx: 0,
                    message: "x".into(),
                },
                INVALID,
            ),
            (
                ToolkitError::CosignersFile {
                    message: "x".into(),
                },
                INVALID,
            ),
            (ToolkitError::DecodeAddress("x".into()), INVALID),
            (ToolkitError::DeriveChildLengthNotApplicable, USAGE),
            (
                ToolkitError::DeriveChildLengthOutOfRange {
                    app: "x",
                    length: 1,
                    valid_text: "x",
                },
                USAGE,
            ),
            (ToolkitError::DeriveChildUnsupportedApp, INVALID),
            (ToolkitError::DescriptorParse("x".into()), INVALID),
            (
                ToolkitError::DescriptorReparseFailed { detail: "x".into() },
                INTERNAL,
            ),
            (
                ToolkitError::EnvVarMissing {
                    flag: "--from".into(),
                    var: "X".into(),
                    reason: EnvVarMissingReason::Unset,
                },
                IO,
            ),
            (
                ToolkitError::EnvVarMissing {
                    flag: "--from".into(),
                    var: "x y".into(),
                    reason: EnvVarMissingReason::InvalidName,
                },
                USAGE,
            ),
            (
                ToolkitError::EnvVarMissing {
                    flag: "--from".into(),
                    var: "X".into(),
                    reason: EnvVarMissingReason::NotUnicode,
                },
                INVALID,
            ),
            (ToolkitError::ExportWalletFormatStub("x"), INVALID),
            (
                ToolkitError::ExportWalletMissingFields {
                    format: "x",
                    missing: vec![],
                },
                INVALID,
            ),
            (ToolkitError::ExportWalletSecretInput, INVALID),
            (
                ToolkitError::ExportWalletTaprootMultisigUnsupported("x"),
                INVALID,
            ),
            (
                ToolkitError::ExportWalletUnsortedMultisigUnsupported { format: "x" },
                INVALID,
            ),
            (
                ToolkitError::FutureFormat {
                    source: "ms_codec",
                    detail: "x".into(),
                },
                INVALID,
            ),
            (
                ToolkitError::HrpMismatch {
                    flag: "--md1",
                    expected: "md",
                    got: "mk".into(),
                },
                INVALID,
            ),
            (
                ToolkitError::ImportWalletAmbiguousFormat("x".into()),
                INVALID,
            ),
            (
                ToolkitError::ImportWalletFormatMismatch {
                    supplied: "x".into(),
                    sniffed: "y".into(),
                },
                INVALID,
            ),
            (
                ToolkitError::ImportWalletNetworkClassMismatch {
                    requested: "mainnet".into(),
                    parsed_coin_type: 1,
                },
                REFUSED,
            ),
            (ToolkitError::ImportWalletParse("x".into()), INVALID),
            (
                ToolkitError::ImportWalletSeedMismatch {
                    cosigner_index: 0,
                    derived_xpub: "x".into(),
                    blob_xpub: "y".into(),
                    path: "m".into(),
                },
                NO,
            ),
            (ToolkitError::ImportWalletWatchOnlyViolation(0), INTERNAL),
            (ToolkitError::ImportWalletXprvForbidden, INVALID),
            (ToolkitError::Internal("x".into()), INTERNAL),
            (
                ToolkitError::Io(std::io::Error::from(std::io::ErrorKind::NotFound)),
                IO,
            ),
            (
                ToolkitError::Io(std::io::Error::from(std::io::ErrorKind::InvalidData)),
                INVALID,
            ),
            (ToolkitError::IoFailure("x".into()), IO),
            (
                ToolkitError::Md1CardNotADescriptor { surface: "x" },
                INVALID,
            ),
            (
                ToolkitError::MdCodec(md_codec::Error::Codex32DecodeError("x".into())),
                INVALID,
            ),
            (
                ToolkitError::MkCodec(mk_codec::Error::BchUncorrectable("x".into())),
                INVALID,
            ),
            (
                ToolkitError::ModeInputViolation {
                    mode: "x",
                    flag: "--md1",
                    message: "x",
                },
                INVALID,
            ),
            (
                ToolkitError::ModeViolation {
                    mode: "x",
                    flag: "--template",
                    message: "x",
                },
                USAGE,
            ),
            (
                ToolkitError::MsCodec(ms_codec::Error::InvalidThreshold(1)),
                USAGE,
            ),
            (
                ToolkitError::MultisigConfig {
                    message: "x".into(),
                },
                INVALID,
            ),
            (
                ToolkitError::NetworkMismatch {
                    decoded_network: "mainnet",
                    expected_network: "testnet",
                    context: "x",
                },
                REFUSED,
            ),
            (ToolkitError::NostrKeyParse("x".into()), INVALID),
            (ToolkitError::Refused("x".into()), REFUSED),
            (
                ToolkitError::Render(md_codec::RenderError::MalformedTree("x".into())),
                INVALID,
            ),
            (
                ToolkitError::Repair(crate::repair::RepairError::EmptyInput),
                INVALID,
            ),
            (ToolkitError::RepairShortCircuit { exit_code: 2 }, INVALID),
            (
                ToolkitError::RestoreMismatch {
                    reference: "fingerprint",
                    derived: "x".into(),
                    expected: "y".into(),
                    slot: None,
                },
                NO,
            ),
            (ToolkitError::SilentPayment("x".into()), INVALID),
            (
                ToolkitError::SlotInputViolation {
                    kind: "x",
                    message: "x".into(),
                },
                INVALID,
            ),
            (
                ToolkitError::TemplateFormUnsupportedShape {
                    message: "x".into(),
                },
                INVALID,
            ),
            (
                ToolkitError::UnknownHrp {
                    got: "x".into(),
                    expected_one_of: vec!["md"],
                },
                INVALID,
            ),
            (ToolkitError::Usage("x".into()), USAGE),
            (ToolkitError::VerifyMessage("x".into()), INVALID),
            (
                ToolkitError::WordCard(wc_codec::WcError::HeaderCrcMismatch),
                INVALID,
            ),
            (
                ToolkitError::XpubSearchNoMatch {
                    mode: "x",
                    searched: 1,
                },
                NO,
            ),
            (
                ToolkitError::XpubSearchPassphraseCandidatesExhausted {
                    candidates_tried: 1,
                },
                NO,
            ),
        ]
    }

    /// Compile-time completeness of `class_table`: adding a variant breaks this
    /// match until the variant is given an arm, and its arm's comment points at
    /// the table.
    fn variant_index(e: &ToolkitError) -> usize {
        match e {
            ToolkitError::BadInput(_) => 0,
            ToolkitError::Bip39(_) => 1,
            ToolkitError::Bip388Distinctness { .. } => 2,
            ToolkitError::Bip388VerifyDistinctness => 3,
            ToolkitError::Bitcoin(_) => 4,
            ToolkitError::BsmsMacMismatch { .. } => 5,
            ToolkitError::BsmsRound1Malformed { .. } => 6,
            ToolkitError::BsmsSignatureMismatch { .. } => 7,
            ToolkitError::BsmsTaprootImportRefused => 8,
            ToolkitError::BsmsTaprootRefused { .. } => 9,
            ToolkitError::BuildDescriptorSpec(_) => 10,
            ToolkitError::BundleMismatch { .. } => 11,
            ToolkitError::CompareCost(_) => 12,
            ToolkitError::ConvertRefusal(_) => 13,
            ToolkitError::CosignerSpec { .. } => 14,
            ToolkitError::CosignersFile { .. } => 15,
            ToolkitError::DecodeAddress(_) => 16,
            ToolkitError::DeriveChildLengthNotApplicable => 17,
            ToolkitError::DeriveChildLengthOutOfRange { .. } => 18,
            ToolkitError::DeriveChildUnsupportedApp => 19,
            ToolkitError::DescriptorParse(_) => 20,
            ToolkitError::DescriptorReparseFailed { .. } => 21,
            ToolkitError::EnvVarMissing { .. } => 22,
            ToolkitError::ExportWalletFormatStub(_) => 23,
            ToolkitError::ExportWalletMissingFields { .. } => 24,
            ToolkitError::ExportWalletSecretInput => 25,
            ToolkitError::ExportWalletTaprootMultisigUnsupported(_) => 26,
            ToolkitError::ExportWalletUnsortedMultisigUnsupported { .. } => 27,
            ToolkitError::FutureFormat { .. } => 28,
            ToolkitError::HrpMismatch { .. } => 29,
            ToolkitError::ImportWalletAmbiguousFormat(_) => 30,
            ToolkitError::ImportWalletFormatMismatch { .. } => 31,
            ToolkitError::ImportWalletNetworkClassMismatch { .. } => 32,
            ToolkitError::ImportWalletParse(_) => 33,
            ToolkitError::ImportWalletSeedMismatch { .. } => 34,
            ToolkitError::ImportWalletWatchOnlyViolation(_) => 35,
            ToolkitError::ImportWalletXprvForbidden => 36,
            ToolkitError::Internal(_) => 37,
            ToolkitError::Io(_) => 38,
            ToolkitError::IoFailure(_) => 39,
            ToolkitError::Md1CardNotADescriptor { .. } => 40,
            ToolkitError::MdCodec(_) => 41,
            ToolkitError::MkCodec(_) => 42,
            ToolkitError::ModeInputViolation { .. } => 43,
            ToolkitError::ModeViolation { .. } => 44,
            ToolkitError::MsCodec(_) => 45,
            ToolkitError::MultisigConfig { .. } => 46,
            ToolkitError::NetworkMismatch { .. } => 47,
            ToolkitError::NostrKeyParse(_) => 48,
            ToolkitError::Refused(_) => 49,
            ToolkitError::Render(_) => 50,
            ToolkitError::Repair(_) => 51,
            ToolkitError::RepairShortCircuit { .. } => 52,
            ToolkitError::RestoreMismatch { .. } => 53,
            ToolkitError::SilentPayment(_) => 54,
            ToolkitError::SlotInputViolation { .. } => 55,
            ToolkitError::TemplateFormUnsupportedShape { .. } => 56,
            ToolkitError::UnknownHrp { .. } => 57,
            ToolkitError::Usage(_) => 58,
            ToolkitError::VerifyMessage(_) => 59,
            ToolkitError::WordCard(_) => 60,
            ToolkitError::XpubSearchNoMatch { .. } => 61,
            ToolkitError::XpubSearchPassphraseCandidatesExhausted { .. } => 62,
        }
    }
    const VARIANT_COUNT: usize = 63;

    #[test]
    fn exit_code_every_variant_has_its_class() {
        let table = class_table();
        let mut seen = [false; VARIANT_COUNT];
        for (e, class) in &table {
            seen[variant_index(e)] = true;
            assert_eq!(e.exit_code(), *class, "{} ({e:?})", e.kind());
            // No error maps to 0, and every code is a §3 code.
            let c = e.exit_code();
            assert!(
                matches!(c, 1..=63 | 64 | 101),
                "{}: {c} is not a §3 code",
                e.kind()
            );
        }
        let missing: Vec<usize> = (0..VARIANT_COUNT).filter(|i| !seen[*i]).collect();
        assert!(
            missing.is_empty(),
            "variants with no class row: {missing:?}"
        );
    }

    /// Every ms-codec and mk-codec variant this crate's pin has is classified
    /// (the enums are `#[non_exhaustive]`; an unnamed one would be 101).
    #[test]
    fn codec_variants_have_their_class() {
        use crate::exit_code::{INVALID, USAGE};
        let ms = [
            ms_codec::Error::MnemUnknownLanguage(9),
            ms_codec::Error::WrongHrp { got: "mq".into() },
            ms_codec::Error::ReservedPrefixViolation { got: 1 },
            ms_codec::Error::SecretShareSuppliedToCombine,
            ms_codec::Error::InconsistentShareSet,
        ];
        for e in ms {
            assert_eq!(ToolkitError::MsCodec(e).exit_code(), INVALID);
        }
        assert_eq!(
            ToolkitError::MsCodec(ms_codec::Error::InvalidShareCount { k: 3, n: 2 }).exit_code(),
            USAGE
        );
        let mk = [
            mk_codec::Error::InvalidHrp("x".into()),
            mk_codec::Error::MixedCase,
            mk_codec::Error::InvalidStringLength(5),
            mk_codec::Error::UnsupportedVersion(1),
            mk_codec::Error::TrailingBytes,
        ];
        for e in mk {
            assert_eq!(ToolkitError::MkCodec(e).exit_code(), INVALID);
        }
    }

    #[test]
    fn ms_codec_inner_variant_routing() {
        // Exit-2 (format-violation) variants per SPEC §6.4.3.
        assert_eq!(
            ToolkitError::MsCodec(ms_codec::Error::WrongHrp { got: "mq".into() }).exit_code(),
            2,
        );
        assert_eq!(
            ToolkitError::MsCodec(ms_codec::Error::ReservedPrefixViolation { got: 0x01 })
                .exit_code(),
            2,
        );
        // Exit-1 (user-input) variants.
        assert_eq!(
            ToolkitError::MsCodec(ms_codec::Error::UnexpectedStringLength {
                got: 51,
                allowed: &[],
            })
            .exit_code(),
            2,
        );
        // v0.2 K-of-N: a share handed to a single-string consume path, or the
        // secret-at-S handed to combine, is a format violation (exit 2).
        assert_eq!(
            ToolkitError::MsCodec(ms_codec::Error::IsShareNotSingleString {
                threshold: '2',
                index: 'a',
            })
            .exit_code(),
            2,
        );
        assert_eq!(
            ToolkitError::MsCodec(ms_codec::Error::SecretShareSuppliedToCombine).exit_code(),
            2,
        );
        // Split-side K and N are settings: usage (64).
        assert_eq!(
            ToolkitError::MsCodec(ms_codec::Error::InvalidThreshold(1)).exit_code(),
            64,
        );
        assert_eq!(
            ToolkitError::MsCodec(ms_codec::Error::InvalidShareCount { k: 3, n: 2 }).exit_code(),
            64,
        );
    }

    #[test]
    fn mk_codec_inner_variant_routing() {
        // Exit-2 (format-violation) variants per SPEC §6.4.4.
        assert_eq!(
            ToolkitError::MkCodec(mk_codec::Error::InvalidHrp("foo".into())).exit_code(),
            2,
        );
        assert_eq!(
            ToolkitError::MkCodec(mk_codec::Error::ReservedBitsSet).exit_code(),
            2,
        );
        assert_eq!(
            ToolkitError::MkCodec(mk_codec::Error::MalformedPayloadPadding).exit_code(),
            2,
        );
        // Exit-1 (user-input) variants.
        assert_eq!(
            ToolkitError::MkCodec(mk_codec::Error::InvalidStringLength(50)).exit_code(),
            2,
        );
        assert_eq!(
            ToolkitError::MkCodec(mk_codec::Error::BchUncorrectable("foo".into())).exit_code(),
            2,
        );
    }

    #[test]
    fn md_codec_inner_variant_routing() {
        // Exit-2 (format-violation) variants per SPEC §6.4.5.
        // Note: ReservedHeaderBitSet was removed in md-codec v0.30; the
        // semantic replacement for malformed-header conditions is
        // `MalformedHeader { detail }`.
        assert_eq!(
            ToolkitError::MdCodec(md_codec::Error::MalformedHeader {
                detail: "test".into()
            })
            .exit_code(),
            2,
        );
        assert_eq!(
            ToolkitError::MdCodec(md_codec::Error::ChunkSetEmpty).exit_code(),
            2,
        );
        assert_eq!(
            ToolkitError::MdCodec(md_codec::Error::HardenedPublicDerivation).exit_code(),
            2,
        );
        // Exit-1 (user-input) variants.
        assert_eq!(
            ToolkitError::MdCodec(md_codec::Error::Codex32DecodeError("foo".into())).exit_code(),
            2,
        );
        assert_eq!(
            ToolkitError::MdCodec(md_codec::Error::Codex32EncodeError("bar".into())).exit_code(),
            2,
        );
    }

    /// v0.30/v0.31/v0.32 added 7 new md-codec variants. This cell pins
    /// the exit-code routing for each — 1 routes via From → FutureFormat
    /// (exit 2 since the exit-code spec; was 3); the other 6 route to MdCodec → exit 2 per SPEC §6.4.5.
    #[test]
    fn md_codec_v0_30_new_variant_routing() {
        // exit 2 (was 3): WireVersionMismatch is the v0.30 semantic replacement
        // for the v0.x UnsupportedVersion variant; both route via From →
        // FutureFormat.
        let tk: ToolkitError = md_codec::Error::WireVersionMismatch { got: 99 }.into();
        assert_eq!(tk.exit_code(), 2);
        assert_eq!(tk.kind(), "FutureFormat");

        // exit-2: the six remaining new variants must route to MdCodec.
        assert_eq!(
            ToolkitError::MdCodec(md_codec::Error::MalformedHeader {
                detail: "test".into()
            })
            .exit_code(),
            2,
        );
        assert_eq!(
            ToolkitError::MdCodec(md_codec::Error::TagOutOfRange { primary: 0xAB }).exit_code(),
            2,
        );
        assert_eq!(
            ToolkitError::MdCodec(md_codec::Error::NUMSSentinelConflict).exit_code(),
            2,
        );
        assert_eq!(
            ToolkitError::MdCodec(md_codec::Error::DecodeRecursionDepthExceeded {
                depth: 33,
                max: 32
            })
            .exit_code(),
            2,
        );
        assert_eq!(
            ToolkitError::MdCodec(md_codec::Error::AddressDerivationFailed {
                detail: "test".into()
            })
            .exit_code(),
            2,
        );
        assert_eq!(
            ToolkitError::MdCodec(md_codec::Error::OperatorContextViolation {
                tag: md_codec::tag::Tag::Multi,
                context: md_codec::error::ContextKind::TopLevel,
            })
            .exit_code(),
            2,
        );
    }

    /// md-codec 0.41.0 (F-A8) added ONE new `Error` variant —
    /// `MalformedPayloadPadding { bits }`, the non-zero trailing-pad decode
    /// reject fired by the shared TLV parser (`reject_non_zero_pad`). Pin its
    /// routing: it is a decode-reject-class error → MdCodec → exit 2 +
    /// kind() == "MdCodec" (NOT the FutureFormat interception, which is
    /// reserved for WireVersionMismatch). Mirrors
    /// `md_codec_v0_30_new_variant_routing`.
    #[test]
    fn md_codec_v0_41_new_variant_routing() {
        // Direct MdCodec wrap → exit 2 + "MdCodec".
        let e = ToolkitError::MdCodec(md_codec::Error::MalformedPayloadPadding { bits: 3 });
        assert_eq!(e.exit_code(), 2);
        assert_eq!(e.kind(), "MdCodec");

        // Via `From` (the production path): a non-WireVersionMismatch variant
        // routes to MdCodec, NOT FutureFormat — so still exit 2 / "MdCodec".
        let tk: ToolkitError = md_codec::Error::MalformedPayloadPadding { bits: 7 }.into();
        assert_eq!(tk.exit_code(), 2);
        assert_eq!(tk.kind(), "MdCodec");
    }

    /// md-codec 0.42.0 (partial-decode P0.3) added ONE new `Error` variant —
    /// `EmptyOriginOverride { idx }`, the unconditional pre-canonical reject of a
    /// use-site override that resolves to an empty origin. Pin its routing: it is
    /// a decode-reject-class error → MdCodec → exit 2 + kind() == "MdCodec" (NOT
    /// the FutureFormat interception). It is deliberately NOT swallowed by
    /// partial-decode's `allow_unresolved_origin` (only `MissingExplicitOrigin`
    /// is fatal-in-partial-suppressible). Mirrors `md_codec_v0_41_new_variant_routing`.
    #[test]
    fn md_codec_v0_42_new_variant_routing() {
        // Direct MdCodec wrap → exit 2 + "MdCodec".
        let e = ToolkitError::MdCodec(md_codec::Error::EmptyOriginOverride { idx: 1 });
        assert_eq!(e.exit_code(), 2);
        assert_eq!(e.kind(), "MdCodec");

        // Via `From` (the production path): a non-WireVersionMismatch variant
        // routes to MdCodec, NOT FutureFormat — so still exit 2 / "MdCodec".
        let tk: ToolkitError = md_codec::Error::EmptyOriginOverride { idx: 3 }.into();
        assert_eq!(tk.exit_code(), 2);
        assert_eq!(tk.kind(), "MdCodec");
    }

    #[test]
    fn v0_2_multisig_variants_exit_code_kind() {
        let e = ToolkitError::MultisigConfig {
            message: "k > n".into(),
        };
        assert_eq!(e.exit_code(), 2);
        assert_eq!(e.kind(), "MultisigConfig");

        let e = ToolkitError::CosignerSpec {
            cosigner_idx: 2,
            message: "fingerprint required".into(),
        };
        assert_eq!(e.exit_code(), 2);
        assert_eq!(e.kind(), "CosignerSpec");
        let det = e.details().unwrap();
        assert_eq!(det["cosigner_idx"], 2);

        let e = ToolkitError::CosignersFile {
            message: "json parse error".into(),
        };
        assert_eq!(e.exit_code(), 2);
        assert_eq!(e.kind(), "CosignersFile");
    }

    #[test]
    fn bip388_variants_exit_code_kind_message() {
        let e = ToolkitError::Bip388Distinctness { i: 0, j: 1 };
        assert_eq!(e.exit_code(), 2);
        assert_eq!(e.kind(), "Bip388Distinctness");
        assert_eq!(
            e.message(),
            "BIP-388 distinct-key violation: slot @0 and slot @1 resolve to identical (xpub, path)"
        );
        let det = e.details().unwrap();
        assert_eq!(det["i"], 0);
        assert_eq!(det["j"], 1);

        let v = ToolkitError::Bip388VerifyDistinctness;
        assert_eq!(v.exit_code(), 2);
        assert_eq!(v.kind(), "Bip388VerifyDistinctness");
        assert_eq!(
            v.message(),
            "bundle violates BIP-388 distinct-key rule; regenerate with distinct keys"
        );
        assert!(v.details().is_none());
    }

    /// v0.28.0 P8B (plan-doc §S.8) — pin `BsmsTaprootRefused`'s tri-arm
    /// behavior: exit_code = 2 (parse/refusal class — same as the
    /// `BadInput` text it replaces); kind = `"BsmsTaprootRefused"`; message
    /// carries the per-script-type discriminator + BIP-386 status note +
    /// FOLLOWUP slug pointer + alternative-format pointers.
    ///
    /// Two cells (P2tr + P2trMulti) verify the message format-arg
    /// substitution distinguishes singlesig (`bip86`/`tr(K)`) from multisig
    /// (`tr-multi-a` / `tr-sortedmulti-a`).
    #[test]
    fn bsms_taproot_refused_variant_p2tr_singlesig() {
        let e = ToolkitError::BsmsTaprootRefused {
            script_type: crate::wallet_export::WalletScriptType::P2tr,
        };
        assert_eq!(e.exit_code(), 2);
        assert_eq!(e.kind(), "BsmsTaprootRefused");
        assert!(e.details().is_none());
        let msg = e.message();
        assert!(
            msg.contains("--format bsms does not support taproot (P2tr)"),
            "P2tr message must include the P2tr discriminator; got:\n{msg}"
        );
        assert!(
            !msg.contains("P2trMulti"),
            "P2tr message must NOT include the multisig discriminator; got:\n{msg}"
        );
        assert!(
            msg.contains("BIP-129 §1 prerequisites do not yet include BIP-386"),
            "message must cite BIP-386 prerequisite gap; got:\n{msg}"
        );
        assert!(
            msg.contains("`bsms-taproot-emit`"),
            "message must point at FOLLOWUP slug; got:\n{msg}"
        );
        assert!(
            msg.contains("--format bitcoin-core") && msg.contains("--format sparrow"),
            "message must list both alternative formats; got:\n{msg}"
        );
    }

    #[test]
    fn bsms_taproot_refused_variant_p2tr_multisig() {
        let e = ToolkitError::BsmsTaprootRefused {
            script_type: crate::wallet_export::WalletScriptType::P2trMulti,
        };
        assert_eq!(e.exit_code(), 2);
        assert_eq!(e.kind(), "BsmsTaprootRefused");
        let msg = e.message();
        assert!(
            msg.contains("--format bsms does not support taproot (P2trMulti)"),
            "P2trMulti message must include the P2trMulti discriminator; got:\n{msg}"
        );
        // Substring discipline: `(P2tr)` alone would falsely match
        // `(P2trMulti)` — assert the non-multisig token does NOT appear by
        // bracketing the suffix character to disambiguate from the prefix.
        assert!(
            !msg.contains("(P2tr)"),
            "P2trMulti message must NOT include the bare-P2tr token; got:\n{msg}"
        );
    }

    #[test]
    fn kind_strings_stable() {
        assert_eq!(ToolkitError::BadInput("x".into()).kind(), "BadInput");
        assert_eq!(
            ToolkitError::Md1CardNotADescriptor { surface: "x" }.kind(),
            "Md1CardNotADescriptor",
        );
        assert_eq!(
            ToolkitError::BundleMismatch {
                card: "ms1".into(),
                message: "".into()
            }
            .kind(),
            "BundleMismatch",
        );
        assert_eq!(
            ToolkitError::FutureFormat {
                source: "ms_codec",
                detail: "".into()
            }
            .kind(),
            "FutureFormat",
        );
    }
}
