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
    Bitcoin(BitcoinErrorKind),
    MsCodec(ms_codec::Error),
    MkCodec(mk_codec::Error),
    MdCodec(md_codec::Error),
    ModeViolation {
        // mode/flag are read by `details()` for SPEC §5.5 JSON output (wired in v0.1+ JSON path).
        #[allow(dead_code)]
        mode: &'static str,
        #[allow(dead_code)]
        flag: &'static str,
        message: &'static str,
    },
    /// SPEC §6.1 exit-4 verify-bundle mismatch variant. `card` identifies the
    /// mismatching card (e.g., "mk1", "md1", or "mk1[N]" for multisig cosigner N).
    #[allow(dead_code)]
    BundleMismatch {
        card: String,
        message: String,
    },
    #[allow(dead_code)]
    NetworkMismatch {
        xpub_network: &'static str,
        expected: &'static str,
    },
    FutureFormat {
        source: &'static str,
        detail: String,
    },
    /// SPEC §6.2 v0.2 multisig configuration error (threshold/cosigner-count
    /// out of range, k > n, etc.). Exit 1 (user-input).
    #[allow(dead_code)]
    MultisigConfig {
        message: String,
    },
    /// SPEC §6.2 v0.2 cosigner-spec parse error
    /// (`--cosigner=<xpub>:<fp>:<path>`). Exit 1.
    #[allow(dead_code)]
    CosignerSpec {
        cosigner_idx: usize,
        message: String,
    },
    /// SPEC §6.2 v0.2 cosigners-file (JSON) parse error. Exit 1.
    #[allow(dead_code)]
    CosignersFile {
        message: String,
    },
    /// SPEC §6.7 descriptor parse error (lex/resolve/walk failure). Exit 2.
    /// Distinct from `ModeViolation` (SPEC §6.9, flag-combination errors):
    /// `DescriptorParse` covers descriptor *content* failures.
    DescriptorParse(String),
    /// SPEC §5.7 verify-bundle: descriptor-derived bundle's preserved
    /// descriptor string fails to round-trip (corrupted JSON, manual edit,
    /// upstream library version mismatch). Exit 4 (BundleMismatch tier).
    DescriptorReparseFailed {
        detail: String,
    },
    /// SPEC §4.11.b BIP-388 distinct-key violation at bundle creation. Exit 2.
    /// `i` and `j` are the colliding slot indices (i < j) under
    /// `(xpub, derivation_path_string)` raw-string equality per §4.11.b
    /// normalization domain.
    Bip388Distinctness { i: u8, j: u8 },
    /// SPEC §4.11.c BIP-388 distinct-key violation at verify-bundle. Exit 4.
    /// Re-emitted from `check_key_vector_distinctness` post-binding under
    /// verify-bundle (different exit code + message vs `Bip388Distinctness`).
    Bip388VerifyDistinctness,
    /// SPEC §6.6 row 4 (conflict) / row 8 (gap) / §6.6.b (invalid subkey set)
    /// `--slot @N.<subkey>=<value>` validation violation at bundle creation.
    /// Exit 2. Wired into `bundle_run` in Phase C.
    #[allow(dead_code)]
    SlotInputViolation {
        /// "conflict" | "gap" | "invalid-set" | "duplicate-subkey".
        kind: &'static str,
        message: String,
    },
    /// SPEC_convert_v0_6.md §3 / §4 refusal — convert subcommand rejects
    /// a (from, to) pair as cryptographically unrecoverable, sibling-pivot,
    /// or otherwise invalid. Exit 2.
    ConvertRefusal(String),
    /// SPEC_export_wallet_v0_7.md §3 watch-only refusal — phrase / entropy /
    /// xprv / wif slot supplied to `export-wallet`. Exit 2.
    ExportWalletSecretInput,
    /// SPEC_export_wallet_v0_7.md §7 — sparrow / specter format stub. Exit 2.
    /// v0.8.1 Phase 2 + Phase 3 promoted Sparrow + Specter to real formats;
    /// no construction site remains in the codebase. Variant retained for
    /// future per-vendor stub introductions (would otherwise be a breaking
    /// removal from a `#[non_exhaustive]` enum).
    #[allow(dead_code)]
    ExportWalletFormatStub(&'static str),
    #[allow(dead_code)]
    /// SPEC_export_wallet_v0_7.md §4 — taproot multisig templates
    /// (`tr-multi-a`, `tr-sortedmulti-a`) are not yet supported by
    /// `mnemonic export-wallet` because constructing `tr(<internal-key>,
    /// multi_a(...))` requires picking an internal-key designation (NUMS vs
    /// key-path key); deferred to v0.8. Exit 2. The `&'static str` payload is
    /// the offending template name (`"tr-multi-a"` or `"tr-sortedmulti-a"`).
    ExportWalletTaprootMultisigUnsupported(&'static str),
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
    /// SPEC_derive_child_v0_7.md §7 — `--application rsa|rsa-gpg` deferred
    /// pending rsa-crate stability (RUSTSEC-2023-0071 unpatched as of v0.8.0).
    /// `dice` shipped in v0.8 Phase 7. Exit 2.
    DeriveChildUnsupportedApp,
    /// SPEC_derive_child_v0_7.md §7 — `--length <N>` falls outside the
    /// per-app valid range. Exit 2.
    DeriveChildLengthOutOfRange {
        app: &'static str,
        length: u32,
        valid_text: &'static str,
    },
    /// SPEC_derive_child_v0_7.md §4 / §7 — non-zero `--length` supplied to
    /// an app whose output is fixed-size (`hd-seed`, `xprv`). Exit 2.
    DeriveChildLengthNotApplicable,
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
    /// v0.24.0 §2.C.1 (D34/I5 fold) — a positional `<STRING>` argument did
    /// not begin with a recognized HRP prefix (`ms1` / `mk1` / `md1`). Exit 2.
    UnknownHrp {
        got: String,
        expected_one_of: Vec<&'static str>,
    },
    /// v0.22.0 repair feature — user-input class (exit 2). Wraps every
    /// `RepairError` variant (EmptyInput / HrpMismatch / TooManyErrors /
    /// UnparseableInput).
    Repair(crate::repair::RepairError),
    /// v0.26.0 compare-cost subcommand error. SPEC §9.
    CompareCost(crate::cost::CompareCostError),
    /// v0.22.0 repair feature — auto-fire short-circuit signal (exit 5).
    /// Synthesized by `repair::try_repair_and_short_circuit` on
    /// repair-success; `?`-propagated through the helper hierarchy up to
    /// the run() boundary; main.rs special-cases this variant to suppress
    /// the Display impl from writing to stderr (per plan-doc R2 I1 — the
    /// repair report already wrote a clean stderr summary; appending the
    /// Display text would be confusing noise).
    RepairShortCircuit { exit_code: u8 },
    /// v0.22.0 repair feature — std::io::Error from emit_repair_report
    /// writes to stdout/stderr. Exit 1 (generic toolkit failure).
    Io(std::io::Error),
    /// v0.26.0 wallet-import cycle — cross-cutting `@env:<VAR>` sentinel
    /// resolution failed. Either the env-var was unset (`Unset`) or the
    /// `<VAR>` token failed POSIX env-var-name validation (`InvalidName`).
    /// Exit 1 (Tier-1, user-input class) per SPEC_wallet_import_v0_26_0.md §2.3.
    /// Carries the offending `--flag` name for stderr disambiguation across
    /// the 6 secret-flag surfaces enumerated in SPEC §3.1.
    EnvVarMissing {
        flag: String,
        var: String,
        reason: EnvVarMissingReason,
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
    /// v0.26.0 `mnemonic xpub-search` — no match found in the searched
    /// candidate set. Exit 4 (sibling to `BundleMismatch` /
    /// `Bip388VerifyDistinctness` — search-target mismatch class).
    /// `mode` distinguishes which xpub-search mode emitted (one of
    /// `"path-of-xpub"`, `"account-of-descriptor"`, `"address-of-xpub"`,
    /// `"passphrase-of-xpub"`); `searched` is the candidate count exhausted
    /// (paths × accounts × cosigners for descriptor mode; addresses ×
    /// chains × gap-limit for address mode).
    XpubSearchNoMatch {
        mode: &'static str,
        searched: usize,
    },
    /// v0.27.0 — `mnemonic import-wallet --bsms-round1 <FILE>` parsed a
    /// blob that does not meet BIP-129 §Round 1 record syntax (line count
    /// != 5 after CRLF normalize, line 1 != `BSMS 1.0`, malformed line-3
    /// KEY field, malformed line-5 base64 SIG, line-4 description carries
    /// `\n` or `\r`, etc.). Exit 2 — parse-class error.
    BsmsRound1Malformed { reason: String },
    /// v0.27.0 — BIP-129 §Round 1 BIP-322 ECDSA recoverable signature
    /// verification failed: signature recovered to a different pubkey than
    /// the one declared on the record's line 3, OR recovery itself failed.
    /// Exit 2 — explicit failure (NOT a stderr NOTICE; the user opted into
    /// verify by supplying `--bsms-round1` + `--bsms-verify-strict`).
    BsmsSignatureMismatch {
        record_index: usize,
        signer_pubkey: String,
        reason: String,
    },
}

/// v0.26.0 — reason discriminant for `ToolkitError::EnvVarMissing`. Drives the
/// two distinct stderr message templates per SPEC_wallet_import_v0_26_0.md §2.4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvVarMissingReason {
    /// The `@env:VAR` syntax was well-formed but `std::env::var(VAR)` failed
    /// (variable is unset or not valid UTF-8).
    Unset,
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

/// SPEC §6.4.3 routing (delegates to ms-cli's §6.1.1 dispatch table).
/// `ReservedTagNotEmittedInV01` is intercepted by `From` to `FutureFormat` (exit 3).
fn ms_codec_exit_code(e: &ms_codec::Error) -> u8 {
    match e {
        ms_codec::Error::Codex32(_)
        | ms_codec::Error::UnexpectedStringLength { .. }
        | ms_codec::Error::PayloadLengthMismatch { .. } => 1,
        ms_codec::Error::WrongHrp { .. }
        | ms_codec::Error::ThresholdNotZero { .. }
        | ms_codec::Error::ShareIndexNotSecret { .. }
        | ms_codec::Error::TagInvalidAlphabet { .. }
        | ms_codec::Error::UnknownTag { .. }
        | ms_codec::Error::ReservedPrefixViolation { .. } => 2,
        // ReservedTagNotEmittedInV01 is intercepted by From → FutureFormat.
        _ => 1,
    }
}

/// SPEC §6.4.4 routing. `UnsupportedVersion` is intercepted by `From` to `FutureFormat`.
fn mk_codec_exit_code(e: &mk_codec::Error) -> u8 {
    match e {
        mk_codec::Error::InvalidStringLength(_)
        | mk_codec::Error::InvalidChar { .. }
        | mk_codec::Error::BchUncorrectable(_) => 1,
        mk_codec::Error::InvalidHrp(_)
        | mk_codec::Error::MixedCase
        | mk_codec::Error::UnsupportedCardType(_)
        | mk_codec::Error::MalformedPayloadPadding
        | mk_codec::Error::ChunkSetIdMismatch
        | mk_codec::Error::ChunkedHeaderMalformed(_)
        | mk_codec::Error::MixedHeaderTypes
        | mk_codec::Error::CrossChunkHashMismatch
        | mk_codec::Error::ReservedBitsSet
        | mk_codec::Error::InvalidPolicyIdStubCount
        | mk_codec::Error::InvalidPathIndicator(_)
        | mk_codec::Error::PathTooDeep(_)
        | mk_codec::Error::InvalidPathComponent(_)
        | mk_codec::Error::InvalidXpubVersion(_)
        | mk_codec::Error::InvalidXpubPublicKey(_)
        | mk_codec::Error::UnexpectedEnd
        | mk_codec::Error::TrailingBytes
        | mk_codec::Error::CardPayloadTooLarge { .. } => 2,
        // UnsupportedVersion is intercepted by From → FutureFormat.
        _ => 1,
    }
}

/// SPEC §6.4.5 routing. md_codec::Error is NOT `#[non_exhaustive]`; match is exhaustive.
/// `WireVersionMismatch` is intercepted by `From` to `FutureFormat` (exit 3).
///
/// v0.30 wire-format clean break (md-codec) removed `ReservedHeaderBitSet`,
/// `UnsupportedVersion`, `UnknownPrimaryTag`, `UnknownExtensionTag`. v0.32
/// removed `UnsupportedDerivationShape`. Replacements: `WireVersionMismatch`
/// (semantic replacement for UnsupportedVersion; same exit-3 routing),
/// `TagOutOfRange` (replaces both Unknown*Tag variants), `MalformedHeader`,
/// `NUMSSentinelConflict`, `OperatorContextViolation`,
/// `DecodeRecursionDepthExceeded`, `AddressDerivationFailed` (replaces
/// UnsupportedDerivationShape).
fn md_codec_exit_code(e: &md_codec::Error) -> u8 {
    match e {
        md_codec::Error::Codex32DecodeError(_) | md_codec::Error::Codex32EncodeError(_) => 1,
        md_codec::Error::BitStreamTruncated { .. }
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
        | md_codec::Error::PlaceholderIndexOutOfRange { .. }
        | md_codec::Error::OverrideOrderViolation { .. }
        | md_codec::Error::EmptyTlvEntry { .. }
        | md_codec::Error::TlvLengthExceedsRemaining { .. }
        | md_codec::Error::PlaceholderNotReferenced { .. }
        | md_codec::Error::PlaceholderFirstOccurrenceOutOfOrder { .. }
        | md_codec::Error::MultipathAltCountMismatch { .. }
        | md_codec::Error::ForbiddenTapTreeLeaf { .. }
        | md_codec::Error::OperatorContextViolation { .. }
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
        | md_codec::Error::InvalidPresenceByte { .. }
        | md_codec::Error::InvalidXpubBytes { .. }
        | md_codec::Error::MissingPubkey { .. }
        | md_codec::Error::ChainIndexOutOfRange { .. }
        | md_codec::Error::HardenedPublicDerivation
        | md_codec::Error::AddressDerivationFailed { .. }
        | md_codec::Error::NUMSSentinelConflict
        | md_codec::Error::DecodeRecursionDepthExceeded { .. }
        // v0.34.0 BCH-error-correction variant (Phase B.2): uncorrectable chunk
        // → exit 2 (Repair error class, matches RepairError::TooManyErrors).
        | md_codec::Error::TooManyErrors { .. } => 2,
        // WireVersionMismatch is intercepted by From → FutureFormat.
        md_codec::Error::WireVersionMismatch { .. } => 3,
    }
}

impl ToolkitError {
    /// SPEC §6.1 exit-code mapping; sibling-codec wrappers dispatch to per-variant
    /// helpers per SPEC §6.4.3 / §6.4.4 / §6.4.5 routing tables.
    pub fn exit_code(&self) -> u8 {
        match self {
            ToolkitError::BadInput(_) | ToolkitError::Bip39(_) | ToolkitError::Bitcoin(_) => 1,
            ToolkitError::MsCodec(e) => ms_codec_exit_code(e),
            ToolkitError::MkCodec(e) => mk_codec_exit_code(e),
            ToolkitError::MdCodec(e) => md_codec_exit_code(e),
            ToolkitError::ModeViolation { .. }
            | ToolkitError::NetworkMismatch { .. }
            | ToolkitError::DescriptorParse(_)
            | ToolkitError::Bip388Distinctness { .. }
            | ToolkitError::SlotInputViolation { .. }
            | ToolkitError::ConvertRefusal(_)
            | ToolkitError::ExportWalletSecretInput
            | ToolkitError::ExportWalletFormatStub(_)
            | ToolkitError::ExportWalletTaprootMultisigUnsupported(_)
            | ToolkitError::ExportWalletMissingFields { .. }
            | ToolkitError::DeriveChildUnsupportedApp
            | ToolkitError::DeriveChildLengthOutOfRange { .. }
            | ToolkitError::DeriveChildLengthNotApplicable
            | ToolkitError::HrpMismatch { .. }
            | ToolkitError::UnknownHrp { .. }
            | ToolkitError::Repair(_) => 2,
            ToolkitError::CompareCost(e) => e.exit_code(),
            ToolkitError::FutureFormat { .. } => 3,
            ToolkitError::BundleMismatch { .. }
            | ToolkitError::DescriptorReparseFailed { .. }
            | ToolkitError::Bip388VerifyDistinctness
            | ToolkitError::XpubSearchNoMatch { .. } => 4,
            ToolkitError::RepairShortCircuit { exit_code } => *exit_code,
            ToolkitError::Io(_) => 1,
            ToolkitError::MultisigConfig { .. }
            | ToolkitError::CosignerSpec { .. }
            | ToolkitError::CosignersFile { .. } => 1,
            ToolkitError::EnvVarMissing { .. } => 1,
            // v0.26.0 wallet-import tier table per SPEC §2.3.
            ToolkitError::ImportWalletAmbiguousFormat(_)
            | ToolkitError::ImportWalletFormatMismatch { .. } => 1,
            ToolkitError::ImportWalletParse(_)
            | ToolkitError::ImportWalletXprvForbidden
            | ToolkitError::ImportWalletWatchOnlyViolation(_) => 2,
            ToolkitError::ImportWalletSeedMismatch { .. } => 4,
            // v0.27.0 — BSMS Round-1 parse/verify failures. Both exit 2
            // (parse-class for Malformed; explicit-fail for SignatureMismatch
            // when --bsms-verify-strict is set; lenient-default path emits
            // stderr NOTICE + signature_verified: false instead of returning
            // SignatureMismatch — caller controls the strictness gate).
            ToolkitError::BsmsRound1Malformed { .. }
            | ToolkitError::BsmsSignatureMismatch { .. } => 2,
        }
    }

    /// Stable discriminant for JSON `kind` field (SPEC §5.5).
    /// Reserved for the §5.5 JSON-error envelope path (covered by tests in v0.1).
    #[allow(dead_code)]
    pub fn kind(&self) -> &'static str {
        match self {
            ToolkitError::BadInput(_) => "BadInput",
            ToolkitError::Bip39(_) => "Bip39",
            ToolkitError::Bitcoin(_) => "Bitcoin",
            ToolkitError::MsCodec(_) => "MsCodec",
            ToolkitError::MkCodec(_) => "MkCodec",
            ToolkitError::MdCodec(_) => "MdCodec",
            ToolkitError::ModeViolation { .. } => "ModeViolation",
            ToolkitError::NetworkMismatch { .. } => "NetworkMismatch",
            ToolkitError::BundleMismatch { .. } => "BundleMismatch",
            ToolkitError::FutureFormat { .. } => "FutureFormat",
            ToolkitError::MultisigConfig { .. } => "MultisigConfig",
            ToolkitError::CosignerSpec { .. } => "CosignerSpec",
            ToolkitError::CosignersFile { .. } => "CosignersFile",
            ToolkitError::DescriptorParse(_) => "DescriptorParse",
            ToolkitError::DescriptorReparseFailed { .. } => "DescriptorReparseFailed",
            ToolkitError::Bip388Distinctness { .. } => "Bip388Distinctness",
            ToolkitError::Bip388VerifyDistinctness => "Bip388VerifyDistinctness",
            ToolkitError::SlotInputViolation { .. } => "SlotInputViolation",
            ToolkitError::ConvertRefusal(_) => "ConvertRefusal",
            ToolkitError::ExportWalletSecretInput => "ExportWalletSecretInput",
            ToolkitError::ExportWalletFormatStub(_) => "ExportWalletFormatStub",
            ToolkitError::ExportWalletTaprootMultisigUnsupported(_) => {
                "ExportWalletTaprootMultisigUnsupported"
            }
            ToolkitError::ExportWalletMissingFields { .. } => "ExportWalletMissingFields",
            ToolkitError::DeriveChildUnsupportedApp => "DeriveChildUnsupportedApp",
            ToolkitError::DeriveChildLengthOutOfRange { .. } => "DeriveChildLengthOutOfRange",
            ToolkitError::DeriveChildLengthNotApplicable => "DeriveChildLengthNotApplicable",
            ToolkitError::HrpMismatch { .. } => "HrpMismatch",
            ToolkitError::UnknownHrp { .. } => "UnknownHrp",
            ToolkitError::Repair(_) => "Repair",
            ToolkitError::RepairShortCircuit { .. } => "RepairShortCircuit",
            ToolkitError::CompareCost(_) => "CompareCost",
            ToolkitError::Io(_) => "Io",
            ToolkitError::EnvVarMissing { .. } => "EnvVarMissing",
            ToolkitError::ImportWalletAmbiguousFormat(_) => "ImportWalletAmbiguousFormat",
            ToolkitError::ImportWalletFormatMismatch { .. } => "ImportWalletFormatMismatch",
            ToolkitError::ImportWalletParse(_) => "ImportWalletParse",
            ToolkitError::ImportWalletSeedMismatch { .. } => "ImportWalletSeedMismatch",
            ToolkitError::ImportWalletWatchOnlyViolation(_) => "ImportWalletWatchOnlyViolation",
            ToolkitError::ImportWalletXprvForbidden => "ImportWalletXprvForbidden",
            ToolkitError::XpubSearchNoMatch { .. } => "XpubSearchNoMatch",
            ToolkitError::BsmsRound1Malformed { .. } => "BsmsRound1Malformed",
            ToolkitError::BsmsSignatureMismatch { .. } => "BsmsSignatureMismatch",
        }
    }

    /// Friendly human-readable message. Five sibling-source mappers live in
    /// `friendly.rs` (Phase 3 task 3.3) and are dispatched here.
    pub fn message(&self) -> String {
        match self {
            ToolkitError::BadInput(m) => m.clone(),
            ToolkitError::Bip39(e) => crate::friendly::friendly_bip39(e),
            ToolkitError::Bitcoin(e) => crate::friendly::friendly_bitcoin(e),
            ToolkitError::MsCodec(e) => crate::friendly::friendly_ms_codec(e),
            ToolkitError::MkCodec(e) => crate::friendly::friendly_mk_codec(e),
            ToolkitError::MdCodec(e) => crate::friendly::friendly_md_codec(e),
            ToolkitError::ModeViolation { message, .. } => (*message).to_owned(),
            ToolkitError::NetworkMismatch {
                xpub_network,
                expected,
            } => format!(
                "xpub network {} does not match --network {}",
                xpub_network, expected,
            ),
            ToolkitError::BundleMismatch { card, message } => {
                format!("bundle mismatch on {}: {}; if the engraved bundle was produced at a non-zero BIP-32 account, pass --account <N> to match (default 0)",
                    card, message)
            }
            ToolkitError::FutureFormat { source, detail } => format!(
                "{} reserved-not-emitted: {}; deferred to v0.2+",
                source, detail,
            ),
            ToolkitError::MultisigConfig { message } => message.clone(),
            ToolkitError::CosignerSpec {
                cosigner_idx,
                message,
            } => format!("--cosigner[{}]: {}", cosigner_idx, message),
            ToolkitError::CosignersFile { message } => {
                format!("--cosigners-file: {}", message)
            }
            ToolkitError::DescriptorParse(m) => m.clone(),
            ToolkitError::DescriptorReparseFailed { detail } => {
                format!("descriptor re-parse failed during verify-bundle: {detail}")
            }
            ToolkitError::Bip388Distinctness { i, j } => {
                format!("BIP-388 distinct-key violation: slot @{i} and slot @{j} resolve to identical (xpub, path)")
            }
            ToolkitError::Bip388VerifyDistinctness => {
                "bundle violates BIP-388 distinct-key rule; regenerate with distinct keys".to_string()
            }
            ToolkitError::SlotInputViolation { message, .. } => message.clone(),
            ToolkitError::ConvertRefusal(m) => m.clone(),
            ToolkitError::ExportWalletSecretInput => crate::wallet_export::REFUSAL_SECRET_INPUT.to_string(),
            ToolkitError::ExportWalletFormatStub(name) => crate::wallet_export::format_stub_message(name),
            ToolkitError::ExportWalletTaprootMultisigUnsupported(name) => {
                crate::wallet_export::taproot_multisig_unsupported_message(name)
            }
            ToolkitError::ExportWalletMissingFields { format, missing } => {
                crate::wallet_export::build_missing_fields_refusal(format, missing)
            }
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
            ToolkitError::DeriveChildLengthOutOfRange {
                app,
                length,
                valid_text,
            } => format!(
                "--length {length} out of range for --application {app} (valid: {valid_text})",
            ),
            ToolkitError::DeriveChildLengthNotApplicable => {
                "--length not applicable for --application <hd-seed|xprv> (output is fixed-size)"
                    .to_string()
            }
            ToolkitError::HrpMismatch { flag, expected, got } => {
                format!(
                    "{flag} expects a value with HRP '{expected}', got '{got}' \
                     (HRP is not BCH-protected; re-type the prefix)"
                )
            }
            ToolkitError::UnknownHrp { got, expected_one_of } => {
                format!(
                    "positional argument '{got}' does not begin with a recognized \
                     HRP prefix (expected one of: {})",
                    expected_one_of.join(", ")
                )
            }
            ToolkitError::Repair(e) => format!("{e}"),
            ToolkitError::RepairShortCircuit { .. } => {
                // R2 I1: main.rs special-cases this variant to skip
                // writing this message to stderr (the repair report
                // already emitted its own clean stderr summary).
                String::new()
            }
            ToolkitError::Io(e) => format!("I/O error: {e}"),
            // v0.26.0 variants in alphabetical order (canonical ordering for
            // I2 multi-PR convergence).
            ToolkitError::CompareCost(e) => format!("{e}"),
            ToolkitError::EnvVarMissing { flag, var, reason } => match reason {
                EnvVarMissingReason::Unset => format!(
                    "{flag}: env-var {var} referenced by sentinel is not set"
                ),
                EnvVarMissingReason::InvalidName => {
                    format!("{flag}: invalid env-var name `{var}`")
                }
            },
            ToolkitError::ImportWalletAmbiguousFormat(detail) => detail.clone(),
            ToolkitError::ImportWalletFormatMismatch { supplied, sniffed } => format!(
                "import-wallet: --format {supplied} supplied but blob looks like {sniffed}"
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
            ToolkitError::XpubSearchNoMatch { mode, searched } => format!(
                "no match in searched set: mode={mode}, paths searched={searched}; \
                 widen the range with --max-account / --number-of-accounts, or supply \
                 additional templates via --add-path"
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
        }
    }

    /// JSON `details` field (SPEC §5.5).
    /// Reserved for the §5.5 JSON-error envelope path.
    #[allow(dead_code)]
    pub fn details(&self) -> Option<serde_json::Value> {
        match self {
            ToolkitError::ModeViolation { mode, flag, .. } => Some(json!({
                "mode": mode,
                "flag": flag,
            })),
            ToolkitError::NetworkMismatch {
                xpub_network,
                expected,
            } => Some(json!({
                "xpub_network": xpub_network,
                "expected": expected,
            })),
            ToolkitError::BundleMismatch { card, .. } => Some(json!({ "card": card })),
            ToolkitError::FutureFormat { source, detail } => Some(json!({
                "source": source,
                "detail": detail,
            })),
            ToolkitError::CosignerSpec { cosigner_idx, .. } => Some(json!({
                "cosigner_idx": cosigner_idx,
            })),
            ToolkitError::Bip388Distinctness { i, j } => Some(json!({ "i": i, "j": j })),
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
            // Same exit-3 routing; user-facing detail message kept as
            // "unsupported version N" for CLI message stability.
            md_codec::Error::WireVersionMismatch { got } => ToolkitError::FutureFormat {
                source: "md_codec",
                detail: format!("unsupported version {}", got),
            },
            other => ToolkitError::MdCodec(other),
        }
    }
}

/// Convenience alias; exported for downstream-crate use.
#[allow(dead_code)]
pub type Result<T> = std::result::Result<T, ToolkitError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exit_code_table_per_variant() {
        assert_eq!(ToolkitError::BadInput("x".into()).exit_code(), 1);
        assert_eq!(
            ToolkitError::DescriptorParse("descriptor parse failed: ...".into()).exit_code(),
            2,
        );
        assert_eq!(
            ToolkitError::ModeViolation {
                mode: "watch-only",
                flag: "--passphrase",
                message: "x",
            }
            .exit_code(),
            2,
        );
        assert_eq!(
            ToolkitError::NetworkMismatch {
                xpub_network: "main",
                expected: "test"
            }
            .exit_code(),
            2,
        );
        assert_eq!(
            ToolkitError::FutureFormat {
                source: "ms_codec",
                detail: "x".into()
            }
            .exit_code(),
            3,
        );
        assert_eq!(
            ToolkitError::BundleMismatch {
                card: "mk1".into(),
                message: "x".into()
            }
            .exit_code(),
            4,
        );
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
            1,
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
            1,
        );
        assert_eq!(
            ToolkitError::MkCodec(mk_codec::Error::BchUncorrectable("foo".into())).exit_code(),
            1,
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
            1,
        );
        assert_eq!(
            ToolkitError::MdCodec(md_codec::Error::Codex32EncodeError("bar".into())).exit_code(),
            1,
        );
    }

    /// v0.30/v0.31/v0.32 added 7 new md-codec variants. This cell pins
    /// the exit-code routing for each — 1 routes via From → FutureFormat
    /// (exit 3); the other 6 route to MdCodec → exit 2 per SPEC §6.4.5.
    #[test]
    fn md_codec_v0_30_new_variant_routing() {
        // exit-3: WireVersionMismatch is the v0.30 semantic replacement
        // for the v0.x UnsupportedVersion variant; both route via From →
        // FutureFormat.
        let tk: ToolkitError = md_codec::Error::WireVersionMismatch { got: 99 }.into();
        assert_eq!(tk.exit_code(), 3);
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
            ToolkitError::MdCodec(md_codec::Error::TagOutOfRange { primary: 0xAB })
                .exit_code(),
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

    #[test]
    fn v0_2_multisig_variants_exit_code_kind() {
        let e = ToolkitError::MultisigConfig {
            message: "k > n".into(),
        };
        assert_eq!(e.exit_code(), 1);
        assert_eq!(e.kind(), "MultisigConfig");

        let e = ToolkitError::CosignerSpec {
            cosigner_idx: 2,
            message: "fingerprint required".into(),
        };
        assert_eq!(e.exit_code(), 1);
        assert_eq!(e.kind(), "CosignerSpec");
        let det = e.details().unwrap();
        assert_eq!(det["cosigner_idx"], 2);

        let e = ToolkitError::CosignersFile {
            message: "json parse error".into(),
        };
        assert_eq!(e.exit_code(), 1);
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
        assert_eq!(v.exit_code(), 4);
        assert_eq!(v.kind(), "Bip388VerifyDistinctness");
        assert_eq!(
            v.message(),
            "bundle violates BIP-388 distinct-key rule; regenerate with distinct keys"
        );
        assert!(v.details().is_none());
    }

    #[test]
    fn kind_strings_stable() {
        assert_eq!(ToolkitError::BadInput("x".into()).kind(), "BadInput");
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
