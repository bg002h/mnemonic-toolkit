//! `mnemonic convert` subcommand — single-format conversion utility.
//!
//! Realizes `design/SPEC_convert_v0_6.md`.

use crate::derive_slot::{derive_bip32_at_path, derive_bip32_from_entropy};
use crate::electrum::{self, SeedVersion};
use crate::wordlists::ElectrumWordlist;
use crate::error::{BitcoinErrorKind, ToolkitError};
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use crate::slip0132::{
    apply_xpub_prefix, normalize_xpub_prefix, parse_xpub_prefix_arg, XpubPrefix,
};
use crate::template::CliTemplate;
use bip38::{Decrypt, EncryptWif};
use bip39::Mnemonic;
use bitcoin::bip32 as bip32;
use bitcoin::hashes::{sha256, Hash};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::{Address, PrivateKey};
use clap::Args;
use serde::Serialize;
use std::io::{Read, Write};
use std::str::FromStr;

// ============================================================================
// SPEC §1 nodes
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    Phrase,
    /// v0.31.6 — SeedQR digit-string (48/60/72/84/96 ASCII digits encoding
    /// a BIP-39 phrase per the SeedSigner SeedQR spec). Secret-bearing;
    /// decoded inline via `crate::seedqr::decode` at `convert::run`
    /// stdin-resolution time (L808+), then substituted as a `Phrase` node
    /// for the downstream conversion dispatch. Closes
    /// `seedqr-digits-from-input-unification` FOLLOWUP.
    Seedqr,
    Entropy,
    Xpub,
    Xprv,
    Wif,
    Fingerprint,
    Path,
    Ms1,
    Mk1,
    Bip38,
    MiniKey,
    ElectrumPhrase,
    Address,
}

impl NodeType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Phrase => "phrase",
            Self::Seedqr => "seedqr",
            Self::Entropy => "entropy",
            Self::Xpub => "xpub",
            Self::Xprv => "xprv",
            Self::Wif => "wif",
            Self::Fingerprint => "fingerprint",
            Self::Path => "path",
            Self::Ms1 => "ms1",
            Self::Mk1 => "mk1",
            Self::Bip38 => "bip38",
            Self::MiniKey => "minikey",
            Self::ElectrumPhrase => "electrum-phrase",
            Self::Address => "address",
        }
    }

    pub fn from_token(t: &str) -> Option<Self> {
        Some(match t {
            "phrase" => Self::Phrase,
            "seedqr" => Self::Seedqr,
            "entropy" => Self::Entropy,
            "xpub" => Self::Xpub,
            "xprv" => Self::Xprv,
            "wif" => Self::Wif,
            "fingerprint" => Self::Fingerprint,
            "path" => Self::Path,
            "ms1" => Self::Ms1,
            "mk1" => Self::Mk1,
            "bip38" => Self::Bip38,
            "minikey" => Self::MiniKey,
            "electrum-phrase" => Self::ElectrumPhrase,
            "address" => Self::Address,
            _ => return None,
        })
    }

    pub fn is_secret_bearing(self) -> bool {
        matches!(
            self,
            Self::Phrase
                | Self::Seedqr
                | Self::Entropy
                | Self::Xprv
                | Self::Wif
                | Self::Ms1
                | Self::Bip38
                | Self::ElectrumPhrase
        )
    }

    /// SPEC v0.9.0 §1 item 1 — superset of `is_secret_bearing()` that adds
    /// MiniKey (Casascius mini-key — a private-key encoding). This widens
    /// the secret-bearing tag specifically for the argv-leakage advisory:
    /// MiniKey is part of survey §5 toolkit row "convert --from minikey=".
    /// The narrower `is_secret_bearing()` predicate is preserved because
    /// it gates separate stdout-redaction / secret-on-stdout machinery
    /// (`convert.rs:769, 796`) whose MiniKey behavior is intentionally
    /// distinct (a separate `convert-minikey-stdout-redaction` follow-up
    /// covers widening THAT predicate).
    pub fn is_argv_secret_bearing(self) -> bool {
        self.is_secret_bearing() || matches!(self, Self::MiniKey)
    }

    pub fn is_side_input_only(self) -> bool {
        matches!(self, Self::Path | Self::Fingerprint)
    }
}

// ============================================================================
// SPEC §5 grammar — `--from <node>=<value>`
// ============================================================================

#[derive(Debug, Clone)]
pub struct FromInput {
    pub node: NodeType,
    pub value: String,
}

pub fn parse_from_input(s: &str) -> Result<FromInput, String> {
    let eq = s
        .find('=')
        .ok_or_else(|| format!("--from must have shape <node>=<value>; got {:?}", s))?;
    let (token, after) = s.split_at(eq);
    let value = &after[1..];
    if token.is_empty() {
        return Err(format!("--from missing node name before '='; got {:?}", s));
    }
    let node = NodeType::from_token(token).ok_or_else(|| {
        format!(
            "unknown --from node {:?}; expected one of: phrase, seedqr, entropy, xpub, xprv, wif, fingerprint, path, ms1, mk1, bip38, minikey, electrum-phrase, address",
            token
        )
    })?;
    if value.is_empty() {
        return Err(format!(
            "--from {} value is empty; supply a non-empty value (or '-' to read from stdin)",
            node.as_str()
        ));
    }
    Ok(FromInput {
        node,
        value: value.to_string(),
    })
}

// ============================================================================
// CLI args
// ============================================================================

#[derive(Args, Debug, Clone)]
pub struct ConvertArgs {
    /// Input node descriptor — shape `<node>=<value>`. Repeating.
    ///
    /// `<node>` is one of:
    ///   phrase           BIP-39 mnemonic (secret)
    ///   seedqr           SeedQR digit-string, 48/60/72/84/96 ASCII digits
    ///                    encoding a BIP-39 phrase (secret; input-only)
    ///   entropy          raw entropy hex (secret)
    ///   xpub             BIP-32 extended public key
    ///   xprv             BIP-32 extended private key (secret)
    ///   wif              Wallet Import Format private key (secret)
    ///   fingerprint      4-byte master fingerprint (hex)
    ///   path             BIP-32 derivation path
    ///   ms1              m-format constellation ms1 seed-card string (secret)
    ///   mk1              m-format constellation mk1 xpub-card string
    ///   bip38            BIP-38 encrypted private key (secret with passphrase)
    ///   minikey          Casascius mini-private-key (secret)
    ///   electrum-phrase  Electrum-format seed phrase (secret)
    ///   address          on-chain Bitcoin address
    ///
    /// `<value>` is the node's text form, or `-` to read from stdin
    /// (one stdin reader per invocation).
    #[arg(
        long = "from",
        action = clap::ArgAction::Append,
        value_parser = parse_from_input,
        required = true,
        verbatim_doc_comment,
    )]
    pub from: Vec<FromInput>,

    /// Output node type. Repeating — emit the same input across
    /// multiple targets in one invocation. Two equivalent forms:
    ///   --to xpub --to xprv
    ///   --to xpub,xprv
    /// Valid values mirror `--from` (see above).
    #[arg(
        long,
        action = clap::ArgAction::Append,
        required = true,
        value_delimiter = ',',
        value_parser = clap::builder::PossibleValuesParser::new([
            "phrase",
            "entropy",
            "xpub",
            "xprv",
            "wif",
            "fingerprint",
            "path",
            "ms1",
            "mk1",
            "bip38",
            "minikey",
            "electrum-phrase",
            "address",
        ]),
        verbatim_doc_comment,
    )]
    pub to: Vec<String>,

    #[arg(long)]
    pub network: Option<CliNetwork>,

    #[arg(long)]
    pub template: Option<CliTemplate>,

    /// BIP-32 derivation path. Accepts:
    ///   named:    `bip44` / `bip49` / `bip84` / `bip86`
    ///   hex:      `0xNN` (raw purpose byte)
    ///   literal:  `m/...` (full BIP-32 path string)
    ///
    /// Required for `--to wif` (no template-implied path) and for
    /// `(xpub, address)` derivation when no `--template` is supplied.
    #[arg(long, verbatim_doc_comment)]
    pub path: Option<String>,

    #[arg(long)]
    pub language: Option<CliLanguage>,

    /// BIP-39 mnemonic-extension passphrase. Empty (default) is the
    /// common case. Mutually exclusive with `--passphrase-stdin`. For
    /// BIP-38 encryption-key passphrase use `--bip38-passphrase`.
    #[arg(long)]
    pub passphrase: Option<String>,

    /// SPEC v0.8 §12.b — BIP-38 Scrypt passphrase, distinct from `--passphrase`.
    /// On composite `(phrase|entropy, bip38)` paths, `--passphrase` feeds BIP-39
    /// PBKDF2 and `--bip38-passphrase` feeds BIP-38 Scrypt independently; if
    /// `--bip38-passphrase` is unset on a composite path, BIP-38 encrypt uses
    /// `""` (BREAKING CHANGE from v0.7's dual-purpose dispatch). On direct
    /// `(wif, bip38)` and `(bip38, wif)` edges, `--bip38-passphrase` falls back
    /// to `--passphrase` when unset.
    #[arg(long = "bip38-passphrase")]
    pub bip38_passphrase: Option<String>,

    /// SPEC v0.8 §5.a — read the value of `--passphrase` from stdin (raw,
    /// preserving NULL bytes). Mutually exclusive with `--passphrase`. Closes
    /// the BIP-38 spec V3 NULL-byte passphrase gap (POSIX argv cannot carry
    /// U+0000). Mutually exclusive with any `--from <node>=-` (single stdin).
    #[arg(long = "passphrase-stdin", conflicts_with = "passphrase")]
    pub passphrase_stdin: bool,

    /// SPEC v0.9.0 §1 item 1 — read `--bip38-passphrase` from stdin (raw,
    /// preserving NULL bytes; strips a single trailing `\r?\n`). Closes
    /// the survey §5 argv-leakage gap for the BIP-38 Scrypt passphrase
    /// channel. Mutually exclusive with `--bip38-passphrase` AND with
    /// `--passphrase-stdin` AND with `--from <node>=-` (single stdin per
    /// invocation).
    #[arg(long = "bip38-passphrase-stdin", conflicts_with = "bip38_passphrase")]
    pub bip38_passphrase_stdin: bool,

    /// BIP-32 account index (default 0). Used to compute the
    /// template-derived path when `--path` is omitted.
    #[arg(long, default_value = "0")]
    pub account: u32,

    /// Master-key fingerprint (8 lowercase hex chars). Used by output
    /// targets that record origin metadata (e.g., `--to mk1`).
    #[arg(long)]
    pub fingerprint: Option<String>,

    /// Emit `xpub` targets with a SLIP-0132 prefix instead of the
    /// canonical `xpub` (mainnet) / `tpub` (testnet) prefixes.
    ///
    /// Accepted values:
    ///   xpub  canonical mainnet legacy/segwit prefix (default)
    ///   ypub  BIP-49 nested-segwit single-sig
    ///   Ypub  BIP-48 nested-segwit multisig
    ///   zpub  BIP-84 native-segwit single-sig
    ///   Zpub  BIP-48 native-segwit multisig
    ///
    /// Requires explicit `--network` when non-default. (SPEC v0.6.1 §11.a)
    #[arg(long = "xpub-prefix", value_parser = parse_xpub_prefix_arg, verbatim_doc_comment)]
    pub xpub_prefix: Option<XpubPrefix>,

    /// Electrum seed-version selector for `(entropy, electrum-phrase)`
    /// encode.
    ///
    /// Accepted values:
    ///   standard  v1 SegWit-v0 seed-version (default)
    ///   segwit    v2 native-SegWit seed-version
    ///
    /// 2FA versions (`standard-2fa`, `segwit-2fa`, `101`, `102`) are
    /// REFUSED at the encode layer (Electrum 2FA requires an
    /// out-of-band second factor). (SPEC v0.7 §14)
    #[arg(long = "electrum-version", value_parser = parse_electrum_version_arg, verbatim_doc_comment)]
    pub electrum_version: Option<SeedVersion>,

    /// Electrum wordlist for the `(entropy, electrum-phrase)` and
    /// `(electrum-phrase, entropy)` arms. Distinct from `--language`
    /// (BIP-39 wordlist set diverges from Electrum's). On Electrum
    /// arms, `--electrum-language` wins; `--language` is silently
    /// ignored.
    ///
    /// Accepted values:
    ///   english             default
    ///   spanish             also accepts `es`
    ///   japanese            also accepts `ja`
    ///   portuguese          also accepts `pt`
    ///   chinese-simplified  also accepts `zh-hans` / `zh`
    ///
    /// (SPEC v0.8 §14)
    #[arg(long = "electrum-language", value_parser = parse_electrum_language_arg, verbatim_doc_comment)]
    pub electrum_language: Option<ElectrumWordlist>,

    /// Script-type selector for `(xpub, address)` derivation.
    ///
    /// Accepted values:
    ///   p2pkh        legacy single-sig (BIP-44; mainnet `1...`, testnet `m/n...`)
    ///   p2wpkh       native-segwit single-sig
    ///   p2sh-p2wpkh  nested-segwit single-sig
    ///   p2tr         taproot single-sig
    ///
    /// If absent and `--template` is supplied, inferred from the
    /// template (`bip44` → p2pkh, `bip84` → p2wpkh, `bip49` → p2sh-p2wpkh,
    /// `bip86` → p2tr); else refused. (SPEC v0.7 §10.a; P2PKH added v0.26.0)
    #[arg(long = "script-type", value_parser = parse_script_type_arg, verbatim_doc_comment)]
    pub script_type: Option<ScriptType>,

    /// Emit a single JSON object on stdout instead of multi-line text.
    #[arg(long)]
    pub json: bool,
}

/// SPEC v0.7 §10.a — script-type selector for the `(Xpub, Address)` edge.
/// Single-sig variants only; multisig templates do not infer to a single-sig
/// script-type. P2PKH (BIP-44) added v0.26.0 to round out the four standard
/// single-sig address types (closes the v0.26.0 xpub-search P3 5-site gap).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptType {
    P2pkh,
    P2wpkh,
    P2shP2wpkh,
    P2tr,
}

impl ScriptType {
    /// Canonical lowercase tag (round-trips with `parse_script_type_arg`).
    pub fn as_str(self) -> &'static str {
        match self {
            ScriptType::P2pkh => "p2pkh",
            ScriptType::P2wpkh => "p2wpkh",
            ScriptType::P2shP2wpkh => "p2sh-p2wpkh",
            ScriptType::P2tr => "p2tr",
        }
    }
}

pub fn parse_script_type_arg(s: &str) -> Result<ScriptType, String> {
    match s {
        "p2pkh" => Ok(ScriptType::P2pkh),
        "p2wpkh" => Ok(ScriptType::P2wpkh),
        "p2sh-p2wpkh" => Ok(ScriptType::P2shP2wpkh),
        "p2tr" => Ok(ScriptType::P2tr),
        _ => Err(format!(
            "--script-type must be one of: p2pkh, p2wpkh, p2sh-p2wpkh, p2tr; got {:?}",
            s,
        )),
    }
}

/// SPEC v0.7 §10.a — infer script-type from `--template` when `--script-type`
/// is absent. Single-sig templates (`bip44` / `bip49` / `bip84` / `bip86`)
/// map cleanly; multisig templates return None (refused upstream as
/// `refusal_address_script_type_unknown_template`).
fn script_type_from_template(template: CliTemplate) -> Option<ScriptType> {
    match template {
        CliTemplate::Bip44 => Some(ScriptType::P2pkh),
        CliTemplate::Bip84 => Some(ScriptType::P2wpkh),
        CliTemplate::Bip49 => Some(ScriptType::P2shP2wpkh),
        CliTemplate::Bip86 => Some(ScriptType::P2tr),
        // Multisig templates don't reduce to a single-sig script-type.
        _ => None,
    }
}

fn parse_electrum_language_arg(s: &str) -> Result<ElectrumWordlist, String> {
    match s {
        "english" => Ok(ElectrumWordlist::English),
        "spanish" | "es" => Ok(ElectrumWordlist::Spanish),
        "japanese" | "ja" => Ok(ElectrumWordlist::Japanese),
        "portuguese" | "pt" => Ok(ElectrumWordlist::Portuguese),
        "chinese-simplified" | "zh-hans" | "zh" => Ok(ElectrumWordlist::ChineseSimplified),
        _ => Err(format!(
            "--electrum-language must be one of: english, spanish, japanese, portuguese, \
             chinese-simplified; got {:?}",
            s,
        )),
    }
}

fn parse_electrum_version_arg(s: &str) -> Result<SeedVersion, String> {
    match s {
        "standard" => Ok(SeedVersion::Standard),
        "segwit" => Ok(SeedVersion::Segwit),
        "standard-2fa" | "segwit-2fa" | "101" | "102" => Err(format!(
            "--electrum-version {:?} is refused (Electrum 2FA seeds require a second factor; \
             only 'standard' and 'segwit' are supported)",
            s,
        )),
        _ => Err(format!(
            "--electrum-version must be one of: standard, segwit; got {:?}",
            s,
        )),
    }
}

// ============================================================================
// SPEC §6 JSON envelope
// ============================================================================

#[derive(Serialize)]
struct ConvertJson<'a> {
    schema_version: &'a str,
    from_node: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    from_value: Option<&'a str>,
    to: Vec<ConvertJsonEntry<'a>>,
}

#[derive(Serialize)]
struct ConvertJsonEntry<'a> {
    node: &'a str,
    value: String,
}

// ============================================================================
// SPEC §3 / §4 refusal taxonomy
// ============================================================================

fn refusal_one_way(from: NodeType, to: NodeType) -> ToolkitError {
    ToolkitError::ConvertRefusal(format!(
        "--to {} is cryptographically unrecoverable from --from {} (one-way derivation barrier)",
        to.as_str(),
        from.as_str(),
    ))
}

fn refusal_sibling_pivot(from: NodeType, to: NodeType) -> ToolkitError {
    ToolkitError::ConvertRefusal(format!(
        "--from {} --to {} is a sibling-format pivot, not a single-format conversion. Use 'mnemonic bundle' instead.",
        from.as_str(),
        to.as_str(),
    ))
}

fn refusal_xpub_to_mk1() -> ToolkitError {
    ToolkitError::ConvertRefusal(
        "--to mk1 requires a policy descriptor binding (mk1 cards bind xpubs to specific policies via policy_id_stubs). Use 'mnemonic bundle --slot @0.xpub=... --template ...' to emit a complete bundle.".into(),
    )
}

fn refusal_phrase_entropy_to_wif_no_path() -> ToolkitError {
    ToolkitError::ConvertRefusal(
        "--to wif requires explicit --path; supply a BIP-32 path producing a leaf privkey (the toolkit does not auto-default a path from --template/--account).".into(),
    )
}

fn refusal_xpub_prefix_no_network() -> ToolkitError {
    ToolkitError::ConvertRefusal(
        "--xpub-prefix <variant> requires explicit --network (cannot infer mainnet vs. testnet swap from defaults).".into(),
    )
}

fn refusal_wif_with_path() -> ToolkitError {
    ToolkitError::ConvertRefusal(
        "--from wif does not retain a chain code; --path-driven derivation is impossible.".into(),
    )
}

// SPEC v0.7 §3.d, v0.8 §12.b
fn refusal_bip38_no_passphrase() -> ToolkitError {
    ToolkitError::ConvertRefusal(
        "--from <bip38|wif> --to <wif|bip38> requires --passphrase or --bip38-passphrase (BIP-38 encryption is passphrase-driven).".into(),
    )
}

fn refusal_bip38_passphrase_mismatch() -> ToolkitError {
    ToolkitError::ConvertRefusal(
        "BIP-38 decryption failed: passphrase does not match the encrypted key (per BIP-38 §\"Decryption\" address-hash check).".into(),
    )
}

fn refusal_bip38_identity() -> ToolkitError {
    ToolkitError::ConvertRefusal(
        "--from bip38 --to bip38 is an identity pivot. To re-encrypt with a different passphrase, decrypt to wif then re-encrypt.".into(),
    )
}

// SPEC v0.7 §3.d — Casascius mini-key refusals.
// `--to minikey`: generation requires brute-forcing the typo-checksum.
fn refusal_minikey_one_way() -> ToolkitError {
    ToolkitError::ConvertRefusal(
        "--to minikey is one-way (mini-key generation requires brute-force search for typo-checksum byte; no inverse derivation).".into(),
    )
}

// SPEC v0.7 §3.d — `minikey → non-wif`: decode-only contract; pivot via wif intermediate.
fn refusal_minikey_decode_only(to: NodeType) -> ToolkitError {
    ToolkitError::ConvertRefusal(format!(
        "--from minikey only supports --to wif (decode-only); cannot convert to {}.",
        to.as_str()
    ))
}

fn refusal_minikey_invalid_format() -> ToolkitError {
    ToolkitError::ConvertRefusal(
        "--from minikey requires a Casascius mini-key string (22/26/30 chars, starting with uppercase 'S'); supplied value does not match.".into(),
    )
}

fn refusal_minikey_invalid_checksum() -> ToolkitError {
    // SPEC §13 wording: "invalid Casascius mini-key checksum (SHA256(key + \"?\")[0] != 0x00)".
    ToolkitError::ConvertRefusal(
        "invalid Casascius mini-key checksum (SHA256(key + \"?\")[0] != 0x00); supplied string is not a valid Casascius mini-key.".into(),
    )
}

// SPEC v0.7 §3.d — Electrum refusals.
fn refusal_electrum_2fa_unsupported() -> ToolkitError {
    ToolkitError::ConvertRefusal(
        "Electrum 2FA seed (version 101 or 102) requires a second factor not present in the phrase alone; conversion not supported. Use Electrum directly for 2FA recovery.".into(),
    )
}

fn refusal_electrum_phrase_pivot() -> ToolkitError {
    ToolkitError::ConvertRefusal(
        "--from phrase --to electrum-phrase (or reverse) is a sibling-format pivot, not a single-format conversion. BIP-39 and Electrum native seeds are different artifact classes.".into(),
    )
}

fn refusal_electrum_invalid_format() -> ToolkitError {
    ToolkitError::ConvertRefusal(
        "--from electrum-phrase value is not a valid Electrum native seed (HMAC-SHA512 prefix did not match a known seed version, or contains words outside the wordlist).".into(),
    )
}

// SPEC v0.7 §10.a / §3.d — Address derivation refusals.
fn refusal_address_no_path() -> ToolkitError {
    ToolkitError::ConvertRefusal(
        "--to address requires --path (xpub does not carry an origin path; supply BIP-32 derivation explicitly).".into(),
    )
}

fn refusal_address_no_script_type() -> ToolkitError {
    ToolkitError::ConvertRefusal(
        "--to address requires --script-type <p2pkh|p2wpkh|p2sh-p2wpkh|p2tr> or --template (script-type inferred from template).".into(),
    )
}

fn refusal_address_script_type_unknown_template() -> ToolkitError {
    ToolkitError::ConvertRefusal(
        "--template does not infer a single-sig --script-type for --to address (bip44/bip49/bip84/bip86 supported; multisig templates require explicit --script-type).".into(),
    )
}

fn refusal_address_one_way() -> ToolkitError {
    ToolkitError::ConvertRefusal(
        "--from address is one-way (addresses are hashes; cannot recover xpub or any source material).".into(),
    )
}

/// Direct edges supported per SPEC §2.
/// Used as the negative-space check for the catch-all refusal: any (from, to)
/// NOT in this set is a one-way barrier.
fn is_supported_direct_edge(from: NodeType, to: NodeType) -> bool {
    use NodeType::*;
    matches!(
        (from, to),
        (Phrase, Entropy)
            | (Entropy, Phrase)
            | (Phrase, Xpub)
            | (Phrase, Xprv)
            | (Phrase, Fingerprint)
            | (Phrase, Ms1)
            | (Phrase, Wif)        // SPEC-A v0.6.1
            // v0.31.6 — SeedQR is a digit-encoded BIP-39 phrase. `seedqr`
            // decodes to a phrase then projects to any phrase-reachable
            // target. `(Seedqr, Phrase)` IS meaningful (the canonical
            // decode operation), distinguishing it from the
            // `(Phrase, Phrase)` identity barrier. Closes
            // `seedqr-digits-from-input-unification`.
            | (Seedqr, Phrase)
            | (Seedqr, Entropy)
            | (Seedqr, Xpub)
            | (Seedqr, Xprv)
            | (Seedqr, Fingerprint)
            | (Seedqr, Ms1)
            | (Seedqr, Wif)
            | (Seedqr, Bip38)
            | (Seedqr, Address)
            | (Entropy, Xpub)
            | (Entropy, Xprv)
            | (Entropy, Fingerprint)
            | (Entropy, Ms1)
            | (Entropy, Wif)       // SPEC-A v0.6.1
            | (Xprv, Xpub)
            | (Xprv, Fingerprint)
            | (Xpub, Fingerprint)
            | (Xpub, Xpub)         // SPEC v0.6.1 §2 — encoding-only normalization (§11/§11.a primitive)
            | (Wif, Xpub)
            | (Wif, Fingerprint)
            | (Ms1, Entropy)
            | (Ms1, Phrase)
            | (Mk1, Xpub)
            | (Mk1, Fingerprint)
            | (Mk1, Path)
            | (Wif, Bip38)         // SPEC v0.7 §12 — BIP-38 encrypt
            | (Bip38, Wif)         // SPEC v0.7 §12 — BIP-38 decrypt
            | (Phrase, Bip38)      // SPEC v0.7 §12 — composite via WIF intermediate
            | (Entropy, Bip38)     // SPEC v0.7 §12 — composite via WIF intermediate
            | (MiniKey, Wif)       // SPEC v0.7 §13 — Casascius mini-key decode (one-way)
            | (ElectrumPhrase, Entropy) // SPEC v0.7 §14 — Electrum seed decode
            | (Entropy, ElectrumPhrase) // SPEC v0.7 §14 — Electrum seed encode
            | (Xpub, Address)      // SPEC v0.7 §10.a — address derivation (one-way)
            | (Phrase, Address)    // SPEC v0.7 §10.a — composite via leaf xpriv
            | (Entropy, Address)   // SPEC v0.7 §10.a — composite via leaf xpriv
    )
}

/// Returns Some(refusal) for a refused (from, to) edge; None when permitted.
fn classify_edge(from: NodeType, to: NodeType) -> Option<ToolkitError> {
    use NodeType::*;

    // §3.d v0.7 — BIP-38 identity-pivot refusal.
    if from == Bip38 && to == Bip38 {
        return Some(refusal_bip38_identity());
    }

    // §3.d v0.7 — `address → *` is one-way (addresses are hashes; no preimage).
    if from == Address {
        return Some(refusal_address_one_way());
    }

    // §3.d v0.7 — `* → minikey` is one-way (typo-checksum requires brute-force).
    if to == MiniKey {
        return Some(refusal_minikey_one_way());
    }
    // §3.d v0.7 — `minikey → non-wif`: decode-only contract; the only supported
    // edge from `minikey` is `(MiniKey, Wif)`. Everything else surfaces with a
    // distinct refusal pointing at the supported target.
    if from == MiniKey && to != Wif {
        return Some(refusal_minikey_decode_only(to));
    }

    // §3.c distinct xpub→mk1 message.
    if from == Xpub && to == Mk1 {
        return Some(refusal_xpub_to_mk1());
    }

    // §3.d v0.7 — Phrase ↔ ElectrumPhrase sibling-pivot refusal (distinct from
    // the generic codec-set sibling pivot below; BIP-39 vs Electrum-native
    // are different artifact classes with different validation rules).
    if matches!(
        (from, to),
        (Phrase, ElectrumPhrase) | (ElectrumPhrase, Phrase),
    ) {
        return Some(refusal_electrum_phrase_pivot());
    }

    // §3.c sibling pivots between codec formats.
    let codec_set = [Ms1, Mk1];
    if codec_set.contains(&from) && codec_set.contains(&to) && from != to {
        return Some(refusal_sibling_pivot(from, to));
    }

    // §3.a/§4 catch-all: any non-supported edge is a one-way barrier.
    if !is_supported_direct_edge(from, to) {
        return Some(refusal_one_way(from, to));
    }

    None
}

// ============================================================================
// SPEC §5.a stdin
// ============================================================================

pub(crate) fn read_stdin_to_string<R: Read>(stdin: &mut R) -> Result<String, ToolkitError> {
    let mut buf = String::new();
    stdin
        .read_to_string(&mut buf)
        .map_err(|e| ToolkitError::BadInput(format!("stdin read: {e}")))?;
    Ok(buf.trim().to_string())
}

/// SPEC v0.8 §5.a — stdin reader for passphrase channels (`--passphrase-stdin`).
/// Strips a single trailing line-ending pair (`\r?\n`) so users can pipe via
/// `echo` or `printf '\n'`-terminated files, but preserves all other bytes —
/// including leading/trailing spaces, internal NULL (BIP-38 V3 spec passphrase),
/// and tabs that may be intentional in the user's passphrase.
pub(crate) fn read_stdin_passphrase<R: Read>(stdin: &mut R) -> Result<String, ToolkitError> {
    let mut buf = String::new();
    stdin
        .read_to_string(&mut buf)
        .map_err(|e| ToolkitError::BadInput(format!("stdin read: {e}")))?;
    if buf.ends_with('\n') {
        buf.pop();
        if buf.ends_with('\r') {
            buf.pop();
        }
    }
    Ok(buf)
}

// ============================================================================
// dispatch entry
// ============================================================================

pub fn run<R: Read, W: Write, E: Write>(
    args: &ConvertArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
    no_auto_repair: bool,
) -> Result<u8, ToolkitError> {
    // v0.25.0 §2.B — TTY-conditional auto-fire (mirrors verify_bundle's
    // v0.22.1 D18 contract). See `crate::repair::resolve_no_auto_repair` for
    // the full public-API contract: `MNEMONIC_FORCE_TTY={0,1}` forces the
    // gate; unset → runtime `is_terminal()` detection. Computed once at
    // function entry and threaded to the downstream auto-fire call so
    // piped consumers (no TTY) see the typed decode error instead of
    // exit 5 short-circuit.
    let effective_no_auto_repair = crate::repair::resolve_no_auto_repair(no_auto_repair);

    // SPEC v0.9.0 §1 item 1 — argv-leakage closure (advisories first).
    // v0.26.0 §I1 fold: emit BEFORE `@env:` sentinel resolution so
    // sentinel-bearing flag values are skipped.
    emit_secret_in_argv_advisories(args, stderr);

    // v0.26.0 §3 — resolve `@env:<VAR>` sentinels before downstream
    // consumption.
    let env_resolved_owned;
    let args: &ConvertArgs = if needs_env_sentinel_resolution(args) {
        env_resolved_owned = resolve_env_sentinels(args)?;
        &env_resolved_owned
    } else {
        args
    };

    // 1) Single-from-value constraint (§5).
    let mut primaries: Vec<&FromInput> = args
        .from
        .iter()
        .filter(|f| !f.node.is_side_input_only())
        .collect();
    if primaries.is_empty() {
        return Err(ToolkitError::BadInput(
            "--from requires at least one primary value-bearing node (phrase, entropy, xpub, xprv, wif, ms1, mk1)".into(),
        ));
    }
    if primaries.len() > 1 {
        return Err(ToolkitError::BadInput(format!(
            "--from accepts at most one primary value-bearing node in v0.6; got {} ({})",
            primaries.len(),
            primaries
                .iter()
                .map(|f| f.node.as_str())
                .collect::<Vec<_>>()
                .join(", "),
        )));
    }
    let primary = primaries.pop().unwrap();

    // 2.a) SPEC v0.8 §5.a + v0.9.0 §1 — single-stdin-per-invocation. The
    // three potential stdin consumers (`--passphrase-stdin`,
    // `--bip38-passphrase-stdin`, `--from <node>=-`) are pairwise mutually
    // exclusive; clap-level locks cover `--{,bip38-}passphrase-stdin`
    // against their inline counterparts, runtime locks cover the
    // stdin-vs-stdin cases below.
    let primary_uses_stdin = primary.value == "-";
    if args.passphrase_stdin && primary_uses_stdin {
        return Err(ToolkitError::BadInput(
            "--passphrase-stdin cannot coexist with --from <node>=- (a single stdin cannot serve both); supply the value-bearing source via argv".into(),
        ));
    }
    if args.bip38_passphrase_stdin && primary_uses_stdin {
        return Err(ToolkitError::BadInput(
            "--bip38-passphrase-stdin cannot coexist with --from <node>=- (a single stdin cannot serve both); supply the value-bearing source via argv".into(),
        ));
    }
    if args.passphrase_stdin && args.bip38_passphrase_stdin {
        return Err(ToolkitError::BadInput(
            "--passphrase-stdin and --bip38-passphrase-stdin cannot both be set (single stdin per invocation); pick the channel that needs the NULL-byte-preserving route".into(),
        ));
    }
    let effective_passphrase: Option<String> = if args.passphrase_stdin {
        Some(read_stdin_passphrase(stdin)?)
    } else {
        args.passphrase.clone()
    };

    // SPEC v0.9.0 §1 item 1 — `--bip38-passphrase-stdin` populates the
    // BIP-38 Scrypt passphrase channel from stdin (preserves NULLs).
    let effective_bip38_passphrase: Option<String> = if args.bip38_passphrase_stdin {
        Some(read_stdin_passphrase(stdin)?)
    } else {
        args.bip38_passphrase.clone()
    };

    // 2.b) Stdin if `--from <node>=-`.
    let primary_value = if primary.value == "-" {
        read_stdin_to_string(stdin)?
    } else {
        primary.value.clone()
    };

    // Cycle B Phase 3a Site 1 — pin secret-bearing heap pages for the
    // remainder of the handler scope. convert.rs has no apply_stdin_substitutions;
    // instead three local secret-bearing String bindings (effective_passphrase,
    // effective_bip38_passphrase, primary_value) are populated above. Pin
    // each post-binding so the pin covers the actual secret bytes consumed
    // downstream (per SPEC §4 P3a + per-handler anchor lock).
    let _pin_pp = effective_passphrase
        .as_ref()
        .map(|s| mnemonic_toolkit::mlock::pin_pages_for(s.as_bytes()));
    let _pin_bip38 = effective_bip38_passphrase
        .as_ref()
        .map(|s| mnemonic_toolkit::mlock::pin_pages_for(s.as_bytes()));
    let _pin_primary = mnemonic_toolkit::mlock::pin_pages_for(primary_value.as_bytes());

    // 3) Parse `--to`.
    let mut targets: Vec<NodeType> = Vec::new();
    for chunk in &args.to {
        for tok in chunk.split(',') {
            let t = tok.trim();
            if t.is_empty() {
                return Err(ToolkitError::BadInput(format!(
                    "--to value contains an empty token; got {:?}",
                    chunk
                )));
            }
            let n = NodeType::from_token(t).ok_or_else(|| {
                ToolkitError::BadInput(format!(
                    "unknown --to node {:?}; expected one of: phrase, entropy, xpub, xprv, wif, fingerprint, path, ms1, mk1, bip38, minikey, electrum-phrase, address",
                    t
                ))
            })?;
            // NB: `seedqr` is intentionally absent from the `--to`
            // PossibleValuesParser list (L207) — it is an INPUT-only node
            // (`--from seedqr=`), so clap rejects `--to seedqr` at parse-time
            // (clap's raw exit 2 is remapped to exit 64 / EX_USAGE by the
            // sysexits main wrapper). Emitting a SeedQR digit-string is the
            // job of `mnemonic seedqr encode`.
            targets.push(n);
        }
    }
    if targets.is_empty() {
        return Err(ToolkitError::BadInput(
            "--to requires at least one node".into(),
        ));
    }

    // 4) §3 refusal pre-check.
    for &t in &targets {
        if let Some(e) = classify_edge(primary.node, t) {
            return Err(e);
        }
    }

    // 5) §4 WIF + --path guard.
    if primary.node == NodeType::Wif && args.path.is_some() {
        return Err(refusal_wif_with_path());
    }

    // 5.a) SPEC §11.a — `--xpub-prefix` (non-default) requires explicit `--network`.
    if let Some(prefix) = args.xpub_prefix {
        if !prefix.is_default() && args.network.is_none() {
            return Err(refusal_xpub_prefix_no_network());
        }
    }

    // 5.b) SPEC v0.7 §12 + v0.8 §12.b — BIP-38 edges require some passphrase
    //      (`--passphrase`, `--passphrase-stdin`, or `--bip38-passphrase`).
    let bip38_edge = primary.node == NodeType::Bip38
        || targets.iter().any(|t| *t == NodeType::Bip38);
    if bip38_edge && effective_passphrase.is_none() && effective_bip38_passphrase.is_none() {
        return Err(refusal_bip38_no_passphrase());
    }

    // 6) §8 --passphrase warning when not on PBKDF2 edge.
    //    SPEC-A v0.6.1: `Wif` joins the PBKDF2-bearing target set so
    //    `--from phrase --to wif --passphrase x` does NOT spuriously
    //    fire the ignored-passphrase warning (phrase → seed → master
    //    → derive at path → leaf privkey → WIF traverses PBKDF2).
    // v0.31.6 — Seedqr decodes to a BIP-39 phrase, so it traverses the
    // same PBKDF2 (phrase → seed) path as Phrase for derivation targets.
    let edge_uses_pbkdf2 =
        matches!(primary.node, NodeType::Seedqr | NodeType::Phrase | NodeType::Entropy)
            && targets.iter().any(|t| {
                matches!(
                    t,
                    NodeType::Xpub | NodeType::Xprv | NodeType::Fingerprint | NodeType::Wif
                )
            });
    // SPEC v0.7 §12 — BIP-38 uses Scrypt (not PBKDF2) but the passphrase IS
    // meaningful; suppress the "ignored" warning for BIP-38 edges.
    let edge_uses_passphrase = edge_uses_pbkdf2 || bip38_edge;
    if effective_passphrase.is_some() && !edge_uses_passphrase {
        let _ = writeln!(
            stderr,
            "warning: --passphrase ignored on this edge (not a PBKDF2-bearing conversion)",
        );
    }
    // SPEC v0.8 §12.b — `--bip38-passphrase` is BIP-38-only; warn if supplied
    // on a non-BIP-38 edge.
    if effective_bip38_passphrase.is_some() && !bip38_edge {
        let _ = writeln!(
            stderr,
            "warning: --bip38-passphrase ignored on this edge (no BIP-38 source/target)",
        );
    }

    // 7) §2 wif→xpub sentinel warning (chain-code zeroed; not BIP-32 derivable).
    if primary.node == NodeType::Wif && targets.iter().any(|t| *t == NodeType::Xpub) {
        let _ = writeln!(
            stderr,
            "warning: wif → xpub emits a depth-0 sentinel with a zeroed chain code; this xpub is not BIP-32 derivable",
        );
    }

    // 8) Compute outputs.
    let pbkdf2_passphrase = effective_passphrase.as_deref().unwrap_or("");
    let bip38_passphrase = effective_bip38_passphrase.as_deref();
    let computed = compute_outputs(
        primary.node,
        &primary_value,
        &targets,
        args,
        pbkdf2_passphrase,
        bip38_passphrase,
    );
    let (mut outputs, input_variant, electrum_seed_version) = match computed {
        Ok(o) => o,
        Err(orig) => {
            // v0.22.0 auto-fire — on Ms1 / Mk1 sibling-codec decode failures,
            // attempt BCH correction and short-circuit with exit 5 on success.
            // Falls through to typed `orig` if repair fails or the original
            // error wasn't a decode-class failure. Per D6 / §2.5 standalone
            // `repair` subcommand ignores --no-auto-repair; here we honor it.
            // v0.25.0 §2.B — use `effective_no_auto_repair` (TTY-aware) so
            // piped invocations skip auto-fire by default; see top-of-fn.
            if !effective_no_auto_repair {
                let repair_kind = match primary.node {
                    NodeType::Ms1 => Some(crate::repair::CardKind::Ms1),
                    NodeType::Mk1 => Some(crate::repair::CardKind::Mk1),
                    _ => None,
                };
                let is_codec_decode_err = matches!(
                    &orig,
                    ToolkitError::MsCodec(_) | ToolkitError::MkCodec(_)
                );
                if let (Some(kind), true) = (repair_kind, is_codec_decode_err) {
                    let chunks: Vec<String> = if kind == crate::repair::CardKind::Mk1 {
                        primary_value
                            .split_whitespace()
                            .map(|s| s.to_string())
                            .collect()
                    } else {
                        vec![primary_value.clone()]
                    };
                    // try_repair_and_short_circuit is always-Err on
                    // repair-success; `?` propagates RepairShortCircuit
                    // (exit 5) up to main.rs's special-case. v0.22.1 D20:
                    // pass `args.json` so the auto-fire emits a JSON
                    // envelope on stdout when the caller expected JSON.
                    crate::repair::try_repair_and_short_circuit(
                        kind, &chunks, stdout, stderr, args.json,
                    )?;
                }
            }
            return Err(orig);
        }
    };

    // SPEC v0.6.1 §11 + v0.6.2 §5.5.a — informational note when SLIP-0132 input was normalized.
    if let Some(variant) = input_variant {
        let _ = writeln!(stderr, "{}", crate::slip0132::render_slip0132_info_line(variant));
    }

    // SPEC v0.8 §14 — Electrum decode emits detected SeedVersion to stderr.
    if let Some(version) = electrum_seed_version {
        let _ = writeln!(
            stderr,
            "note: detected Electrum SeedVersion {} ({})",
            version.label(),
            match version {
                SeedVersion::Standard => "standard",
                SeedVersion::Segwit => "segwit",
                SeedVersion::Standard2FA => "standard-2fa",
                SeedVersion::Segwit2FA => "segwit-2fa",
            },
        );
    }

    // 8.a) SPEC §11.a — apply --xpub-prefix to xpub-typed outputs. The flag
    //      is silently ignored when no xpub target is present (per §11.a).
    if let Some(prefix) = args.xpub_prefix {
        if !prefix.is_default() {
            // §5.a refusal already enforced --network presence above; safe to
            // unwrap_or default for the swap-target lookup.
            let network = args.network.unwrap_or(CliNetwork::Mainnet);
            for (node, value) in outputs.iter_mut() {
                if *node == NodeType::Xpub {
                    let xpub = bip32::Xpub::from_str(value)
                        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
                    *value = apply_xpub_prefix(&xpub, prefix, network);
                }
            }
        }
    }

    // 9) Emit.
    if args.json {
        // v0.34.5: redact via the WIDER `is_argv_secret_bearing` (adds MiniKey,
        // a Casascius private-key carrier) so `--from minikey= --json` does not
        // echo the private key in `from_value`. Closes `convert-minikey-stdout-redaction`.
        let from_value = if primary.node.is_argv_secret_bearing() {
            None
        } else {
            Some(primary_value.as_str())
        };
        let entries: Vec<ConvertJsonEntry> = outputs
            .iter()
            .map(|(node, value)| ConvertJsonEntry {
                node: node.as_str(),
                value: value.clone(),
            })
            .collect();
        let env = ConvertJson {
            schema_version: "1",
            from_node: primary.node.as_str(),
            from_value,
            to: entries,
        };
        serde_json::to_writer(&mut *stdout, &env).ok();
        writeln!(stdout).ok();
    } else {
        for (node, value) in &outputs {
            writeln!(stdout, "{}: {}", node.as_str(), value).ok();
        }
    }

    // 10) §7 secret-on-stdout warning.
    // v0.34.5: `is_argv_secret_bearing` widens to MiniKey for stdout-redaction
    // parity with the `from_value` path above. MiniKey is currently
    // output-unreachable (one-way `MiniKey→Wif`), so this is a no-op today —
    // it keeps both redaction pathways on a single predicate.
    if outputs.iter().any(|(n, _)| n.is_argv_secret_bearing()) {
        let _ = writeln!(
            stderr,
            "warning: secret material on stdout — consider redirecting (e.g., '> file.txt' or '| age -e ...')",
        );
    }

    Ok(0)
}

// ============================================================================
// edge dispatch
// ============================================================================

type Output = (NodeType, String);

// SPEC v0.8 §12.b — `pbkdf2_passphrase` feeds BIP-39 PBKDF2 (mnemonic extension);
// resolved upstream from `--passphrase` or stdin (`--passphrase-stdin`).
// `bip38_passphrase` is `Some` only when the user passed `--bip38-passphrase`.
// Edge-specific fallback rules: composite `(phrase|entropy, bip38)` does NOT
// fall back to `pbkdf2_passphrase` (BREAKING CHANGE); direct `(wif, bip38)` /
// `(bip38, wif)` falls back when unset.
//
// Return-shape tuple decoded as `ComputeOutputsResult` below.
type ComputeOutputsResult = (Vec<Output>, Option<&'static str>, Option<SeedVersion>);

fn compute_outputs(
    from: NodeType,
    value: &str,
    targets: &[NodeType],
    args: &ConvertArgs,
    pbkdf2_passphrase: &str,
    bip38_passphrase: Option<&str>,
) -> Result<ComputeOutputsResult, ToolkitError> {
    use NodeType::*;
    let language = args.language.unwrap_or_default();
    let network = args.network.unwrap_or(CliNetwork::Mainnet);
    let secp = Secp256k1::new();

    match from {
        Seedqr | Phrase | Entropy => {
            // BIP-39 source — derive once, project. v0.31.6: Seedqr decodes
            // its digit-string to a BIP-39 phrase, then folds into the same
            // entropy-projection path. Unlike (Phrase, Phrase), the
            // (Seedqr, Phrase) edge IS permitted (the canonical decode).
            // SAFETY: third-party-blocked — `bip39::Mnemonic` has no
            // Drop+Zeroize; tracked by FOLLOWUP
            // `rust-bip39-mnemonic-zeroize-upstream`.
            let entropy: zeroize::Zeroizing<Vec<u8>> = match from {
                Seedqr => {
                    let phrase =
                        mnemonic_toolkit::seedqr::decode(value).map_err(|e| {
                            ToolkitError::BadInput(format!("seedqr: convert: decode: {e}"))
                        })?;
                    let m = Mnemonic::parse_in(language.into(), &phrase)
                        .map_err(ToolkitError::Bip39)?;
                    zeroize::Zeroizing::new(m.to_entropy())
                }
                Phrase => {
                    let m = Mnemonic::parse_in(language.into(), value)
                        .map_err(ToolkitError::Bip39)?;
                    zeroize::Zeroizing::new(m.to_entropy())
                }
                _ => zeroize::Zeroizing::new(hex::decode(value).map_err(|e| {
                    ToolkitError::BadInput(format!("--from entropy hex-decode: {e}"))
                })?),
            };

            let needs_derive = targets
                .iter()
                .any(|t| matches!(t, Xpub | Xprv | Fingerprint));
            let derived = if needs_derive {
                let template = args.template.ok_or_else(|| {
                    ToolkitError::BadInput(
                        "--template is required for derivation targets (xpub/xprv/fingerprint)".into(),
                    )
                })?;
                Some(derive_bip32_from_entropy(
                    &entropy, pbkdf2_passphrase, language, network, template, args.account,
                )?)
            } else {
                None
            };

            let mut out = Vec::with_capacity(targets.len());
            for &t in targets {
                let v = match t {
                    Phrase => {
                        // SAFETY: third-party-blocked — `bip39::Mnemonic`
                        // has no Drop+Zeroize;
                        // FOLLOWUP `rust-bip39-mnemonic-zeroize-upstream`.
                        Mnemonic::from_entropy_in(language.into(), &entropy[..])
                            .map_err(ToolkitError::Bip39)?
                            .to_string()
                    }
                    Entropy => hex::encode(&entropy[..]),
                    Xpub => derived.as_ref().unwrap().account_xpub.to_string(),
                    Xprv => derived.as_ref().unwrap().account_xpriv.to_string(),
                    Fingerprint => derived
                        .as_ref()
                        .unwrap()
                        .master_fingerprint
                        .to_string()
                        .to_lowercase(),
                    Ms1 => ms_codec::encode(
                        ms_codec::Tag::ENTR,
                        &ms_codec::Payload::Entr((*entropy).clone()),
                    )
                    .map_err(ToolkitError::from)?,
                    Wif => {
                        // SPEC-A v0.6.1: phrase/entropy → wif requires explicit
                        // --path. `needs_derive` deliberately does NOT include
                        // Wif, so --template is not required for this edge.
                        let path_str = args.path.as_deref().ok_or_else(refusal_phrase_entropy_to_wif_no_path)?;
                        let path = bip32::DerivationPath::from_str(path_str)
                            .map_err(|e| ToolkitError::BadInput(format!("--path parse: {e}")))?;
                        let leaf_xpriv = derive_bip32_at_path(
                            &entropy, pbkdf2_passphrase, language, network, &path,
                        )?;
                        // BIP-32 §4 mandates compressed pubkeys for derived
                        // keys; WIF compression follows the BIP-32 contract.
                        let pk = PrivateKey {
                            compressed: true,
                            network: network.network_kind(),
                            inner: leaf_xpriv.private_key,
                        };
                        pk.to_wif()
                    }
                    Path => return Err(ToolkitError::BadInput(
                        "--to path is informational; not emitted as a value".into(),
                    )),
                    Seedqr => unreachable!("--to seedqr is refused at target-parse in convert::run"),
                    Mk1 => unreachable!("classify_edge intercepts (Phrase|Entropy, Mk1) as one-way barrier"),
                    Bip38 => {
                        // SPEC v0.7 §12 + v0.8 §12.b — composite phrase/entropy → wif → bip38.
                        // Same --path requirement as the direct phrase→wif edge.
                        // BREAKING (v0.8): on this composite arm `--passphrase`
                        // feeds BIP-39 PBKDF2 only; `--bip38-passphrase` feeds
                        // BIP-38 Scrypt independently. If the latter is unset,
                        // BIP-38 encrypt uses `""` (no fallback to --passphrase).
                        let path_str = args.path.as_deref().ok_or_else(refusal_phrase_entropy_to_wif_no_path)?;
                        let path = bip32::DerivationPath::from_str(path_str)
                            .map_err(|e| ToolkitError::BadInput(format!("--path parse: {e}")))?;
                        let leaf_xpriv = derive_bip32_at_path(
                            &entropy, pbkdf2_passphrase, language, network, &path,
                        )?;
                        let pk = PrivateKey {
                            compressed: true,
                            network: network.network_kind(),
                            inner: leaf_xpriv.private_key,
                        };
                        let wif = pk.to_wif();
                        let scrypt_pp = bip38_passphrase.unwrap_or("");
                        wif.as_str()
                            .encrypt_wif(scrypt_pp)
                            .map_err(map_bip38_error)?
                    }
                    MiniKey => unreachable!("classify_edge intercepts (*, MiniKey) as one-way"),
                    ElectrumPhrase => {
                        // SPEC v0.7 §14 — `(Entropy, ElectrumPhrase)` direct;
                        // `(Phrase, ElectrumPhrase)` is intercepted as sibling
                        // pivot in classify_edge, so this arm is reached only
                        // for from == Entropy.
                        debug_assert_eq!(from, Entropy);
                        let version = args
                            .electrum_version
                            .unwrap_or(SeedVersion::Standard);
                        let wl = args
                            .electrum_language
                            .unwrap_or(ElectrumWordlist::English);
                        electrum::entropy_to_phrase(&entropy, version, wl)
                            .map_err(map_electrum_error)?
                    }
                    Address => {
                        // SPEC v0.7 §10.a — composite phrase/entropy → address.
                        // `--path` is mandatory and applied from MASTER (NOT
                        // relative to a template-derived account xpub), matching
                        // the semantics of the `phrase|entropy → wif` edge:
                        // the user supplies a path that derives directly to the
                        // leaf pubkey. `--script-type` (or `--template`-inferred)
                        // selects the address dispatch.
                        let path_str = args
                            .path
                            .as_deref()
                            .ok_or_else(refusal_address_no_path)?;
                        let path = bip32::DerivationPath::from_str(path_str)
                            .map_err(|e| ToolkitError::BadInput(format!("--path parse: {e}")))?;
                        let script_type = resolve_script_type(args)?;
                        let leaf_xpriv = derive_bip32_at_path(
                            &entropy, pbkdf2_passphrase, language, network, &path,
                        )?;
                        let leaf_xpub = bip32::Xpub::from_priv(&secp, &leaf_xpriv);
                        build_address_from_xpub(&secp, &leaf_xpub, script_type, network)
                    }
                };
                out.push((t, v));
            }
            Ok((out, None, None))
        }
        Xprv => {
            let xprv = bip32::Xpriv::from_str(value)
                .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
            let xpub = bip32::Xpub::from_priv(&secp, &xprv);
            let mut out = Vec::with_capacity(targets.len());
            for &t in targets {
                let v = match t {
                    Xpub => xpub.to_string(),
                    Fingerprint => xpub.fingerprint().to_string().to_lowercase(),
                    _ => {
                        return Err(ToolkitError::BadInput(format!(
                            "--from xprv --to {} is not a defined edge",
                            t.as_str()
                        )))
                    }
                };
                out.push((t, v));
            }
            Ok((out, None, None))
        }
        Xpub => {
            // SPEC v0.6.1 §11 — accept SLIP-0132 prefix variants on input.
            let (value, input_variant) = normalize_xpub_prefix(value)?;
            let xpub = bip32::Xpub::from_str(&value)
                .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
            let mut out = Vec::with_capacity(targets.len());
            for &t in targets {
                let v = match t {
                    Fingerprint => xpub.fingerprint().to_string().to_lowercase(),
                    // SPEC v0.6.1 §2 — encoding-only normalization. Default
                    // emit is the neutral xpub/tpub; any --xpub-prefix swap
                    // happens in run() after compute_outputs.
                    Xpub => xpub.to_string(),
                    Address => {
                        // SPEC v0.7 §10.a — derive child xpub at --path,
                        // build address per --script-type. Network is
                        // inferred from the xpub when --network is absent.
                        let path_str = args.path.as_deref().ok_or_else(refusal_address_no_path)?;
                        let path = bip32::DerivationPath::from_str(path_str)
                            .map_err(|e| ToolkitError::BadInput(format!("--path parse: {e}")))?;
                        let script_type = resolve_script_type(args)?;
                        let child = xpub
                            .derive_pub(&secp, &path)
                            .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
                        let net = args.network.unwrap_or_else(|| network_from_xpub(&xpub));
                        build_address_from_xpub(&secp, &child, script_type, net)
                    }
                    _ => {
                        return Err(ToolkitError::BadInput(format!(
                            "--from xpub --to {} is not a defined edge",
                            t.as_str()
                        )))
                    }
                };
                out.push((t, v));
            }
            Ok((out, input_variant, None))
        }
        Wif => {
            let pk = PrivateKey::from_wif(value)
                .map_err(|e| ToolkitError::BadInput(format!("--from wif parse: {e}")))?;
            let pubkey = pk.public_key(&secp);
            let sentinel_xpub = bip32::Xpub {
                network: network.network_kind(),
                depth: 0,
                parent_fingerprint: bip32::Fingerprint::default(),
                child_number: bip32::ChildNumber::Normal { index: 0 },
                public_key: pubkey.inner,
                chain_code: bip32::ChainCode::from([0u8; 32]),
            };
            let mut out = Vec::with_capacity(targets.len());
            for &t in targets {
                let v = match t {
                    Xpub => sentinel_xpub.to_string(),
                    Fingerprint => sentinel_xpub.fingerprint().to_string().to_lowercase(),
                    Bip38 => {
                        // SPEC v0.7 §12 + v0.8 §12.b — direct (wif, bip38) Scrypt
                        // encrypt. Per v0.8 lock, `--bip38-passphrase` falls back
                        // to `--passphrase` (effective) on this direct edge.
                        // Passphrase presence is enforced earlier in run().
                        let scrypt_pp = bip38_passphrase.unwrap_or(pbkdf2_passphrase);
                        value
                            .encrypt_wif(scrypt_pp)
                            .map_err(map_bip38_error)?
                    }
                    _ => {
                        return Err(ToolkitError::BadInput(format!(
                            "--from wif --to {} is not a defined edge",
                            t.as_str()
                        )))
                    }
                };
                out.push((t, v));
            }
            Ok((out, None, None))
        }
        Bip38 => {
            // SPEC v0.7 §12 + v0.8 §12.b — decrypt to raw key + compress flag,
            // then build WIF with the user's --network (the crate's
            // decrypt_to_wif always emits mainnet; per Phase 1 security review
            // caveat 2). Per v0.8 lock, `--bip38-passphrase` falls back to
            // `--passphrase` (effective) on this direct edge.
            let scrypt_pp = bip38_passphrase.unwrap_or(pbkdf2_passphrase);
            let (raw, compressed) = <str as Decrypt>::decrypt(value, scrypt_pp)
                .map_err(map_bip38_error)?;
            // SAFETY: third-party-blocked — `secp256k1::SecretKey` is stack-
            // bound, no Drop+Zeroize; FOLLOWUP `rust-secp256k1-secretkey-zeroize-upstream`.
            let inner = bitcoin::secp256k1::SecretKey::from_slice(&raw)
                .map_err(|e| ToolkitError::BadInput(format!("BIP-38 decrypted key parse: {e}")))?;
            let pk = PrivateKey {
                compressed,
                network: network.network_kind(),
                inner,
            };
            let mut out = Vec::with_capacity(targets.len());
            for &t in targets {
                let v = match t {
                    Wif => pk.to_wif(),
                    _ => {
                        return Err(ToolkitError::BadInput(format!(
                            "--from bip38 --to {} is not a defined edge",
                            t.as_str()
                        )))
                    }
                };
                out.push((t, v));
            }
            Ok((out, None, None))
        }
        Ms1 => {
            let (_tag, payload) = ms_codec::decode(value).map_err(ToolkitError::from)?;
            // SAFETY: third-party-blocked — `bip39::Mnemonic` has no
            // Drop+Zeroize; FOLLOWUP `rust-bip39-mnemonic-zeroize-upstream`.
            // ms_codec::Payload::Entr ships a Vec<u8> the codec doesn't
            // scrub (tracked at sibling FOLLOWUPS `secret-memory-hygiene-v0_9-cycle-a`
            // ms-codec rows); the local wrap below protects the duplicate.
            let entropy: zeroize::Zeroizing<Vec<u8>> = match payload {
                ms_codec::Payload::Entr(bytes) => zeroize::Zeroizing::new(bytes),
                _ => {
                    return Err(ToolkitError::BadInput(
                        "ms1 decoded to a non-Entr payload; v0.1 ms-codec emits only Entr".into(),
                    ))
                }
            };
            let mut out = Vec::with_capacity(targets.len());
            for &t in targets {
                let v = match t {
                    Entropy => hex::encode(&entropy[..]),
                    Phrase => {
                        // SAFETY: third-party-blocked — `bip39::Mnemonic` has
                        // no Drop+Zeroize; FOLLOWUP
                        // `rust-bip39-mnemonic-zeroize-upstream`.
                        Mnemonic::from_entropy_in(language.into(), &entropy[..])
                            .map_err(ToolkitError::Bip39)?
                            .to_string()
                    }
                    _ => {
                        return Err(ToolkitError::BadInput(format!(
                            "--from ms1 --to {} is not a defined edge",
                            t.as_str()
                        )))
                    }
                };
                out.push((t, v));
            }
            Ok((out, None, None))
        }
        Mk1 => {
            let tokens: Vec<&str> = value.split_whitespace().collect();
            let card = mk_codec::decode(&tokens).map_err(ToolkitError::from)?;
            let mut out = Vec::with_capacity(targets.len());
            for &t in targets {
                let v = match t {
                    Xpub => card.xpub.to_string(),
                    Fingerprint => card
                        .origin_fingerprint
                        .map(|f| f.to_string().to_lowercase())
                        .ok_or_else(|| {
                            ToolkitError::BadInput(
                                "mk1 card has no origin_fingerprint; cannot project --to fingerprint".into(),
                            )
                        })?,
                    Path => card.origin_path.to_string(),
                    _ => {
                        return Err(ToolkitError::BadInput(format!(
                            "--from mk1 --to {} is not a defined edge",
                            t.as_str()
                        )))
                    }
                };
                out.push((t, v));
            }
            Ok((out, None, None))
        }
        Fingerprint | Path => Err(ToolkitError::BadInput(format!(
            "--from {} is not a primary value-bearing node",
            from.as_str()
        ))),
        MiniKey => {
            // SPEC v0.7 §13 — Casascius mini-private-key decode.
            // Format: 22/26/30 chars, starts with 'S'.
            // Self-checksum: SHA256(key + "?")[0] == 0x00 (typo detection).
            // Privkey: SHA256(key) (32-byte scalar). Compressed flag is false
            // (Casascius predates BIP-32 compressed-pubkey convention).
            let len = value.len();
            if !(matches!(len, 22 | 26 | 30) && value.starts_with('S')) {
                return Err(refusal_minikey_invalid_format());
            }
            let mut buf = Vec::with_capacity(len + 1);
            buf.extend_from_slice(value.as_bytes());
            buf.push(b'?');
            if sha256::Hash::hash(&buf).as_byte_array()[0] != 0x00 {
                return Err(refusal_minikey_invalid_checksum());
            }
            let raw = sha256::Hash::hash(value.as_bytes()).to_byte_array();
            // SAFETY: third-party-blocked — `secp256k1::SecretKey` is stack-
            // bound, no Drop+Zeroize; FOLLOWUP `rust-secp256k1-secretkey-zeroize-upstream`.
            let inner = bitcoin::secp256k1::SecretKey::from_slice(&raw)
                .map_err(|e| ToolkitError::BadInput(format!("Casascius decoded scalar parse: {e}")))?;
            let pk = PrivateKey {
                compressed: false,
                network: network.network_kind(),
                inner,
            };
            let mut out = Vec::with_capacity(targets.len());
            for &t in targets {
                let v = match t {
                    Wif => pk.to_wif(),
                    _ => unreachable!("classify_edge intercepts (MiniKey, !Wif)"),
                };
                out.push((t, v));
            }
            Ok((out, None, None))
        }
        ElectrumPhrase => {
            // SPEC v0.7 §14 + v0.8 §14 — validate via HMAC-SHA512 prefix;
            // refuse 2FA; decode via per-wordlist base-N mapping; surface the
            // detected SeedVersion to the caller for the §14 stderr info-line.
            let version =
                electrum::validate_seed_version(value).map_err(map_electrum_error)?;
            if version.is_2fa() {
                return Err(refusal_electrum_2fa_unsupported());
            }
            let detected_version = Some(version);
            let wl = args
                .electrum_language
                .unwrap_or(ElectrumWordlist::English);
            let entropy =
                electrum::phrase_to_entropy(value, wl).map_err(map_electrum_error)?;
            let mut out = Vec::with_capacity(targets.len());
            for &t in targets {
                let v = match t {
                    Entropy => hex::encode(&entropy),
                    _ => unreachable!("classify_edge intercepts (ElectrumPhrase, !Entropy)"),
                };
                out.push((t, v));
            }
            Ok((out, None, detected_version))
        }
        Address => unreachable!("classify_edge intercepts (Address, *) as one-way"),
    }
}

/// Map `bip38::Error` variants to ToolkitError per SPEC v0.7 §12.
fn map_bip38_error(e: bip38::Error) -> ToolkitError {
    match e {
        bip38::Error::Pass => refusal_bip38_passphrase_mismatch(),
        other => ToolkitError::BadInput(format!("bip38: {other}")),
    }
}

/// Map `electrum::ElectrumError` variants to ToolkitError per SPEC v0.7 §14.
/// `Empty`, `UnknownWord`, and `InvalidVersion` all surface as a single
/// invalid-format refusal — the user-facing distinction is "this isn't an
/// Electrum native seed."
fn map_electrum_error(_e: electrum::ElectrumError) -> ToolkitError {
    refusal_electrum_invalid_format()
}

/// SPEC v0.7 §10.a — resolve the `--script-type` from explicit flag, then
/// `--template` inference, then refuse. Callers reach this only on the
/// `(*, Address)` edge where the resolved type is mandatory.
fn resolve_script_type(args: &ConvertArgs) -> Result<ScriptType, ToolkitError> {
    if let Some(st) = args.script_type {
        return Ok(st);
    }
    if let Some(template) = args.template {
        return script_type_from_template(template)
            .ok_or_else(refusal_address_script_type_unknown_template);
    }
    Err(refusal_address_no_script_type())
}

/// SPEC v0.7 §10.a — render an address from a child xpub (already derived to
/// the leaf path) per the requested script-type and network.
fn build_address_from_xpub<C: bitcoin::secp256k1::Verification>(
    secp: &Secp256k1<C>,
    child: &bip32::Xpub,
    script_type: ScriptType,
    network: CliNetwork,
) -> String {
    match script_type {
        ScriptType::P2pkh => Address::p2pkh(child.to_pub(), network.network_kind()).to_string(),
        ScriptType::P2wpkh => Address::p2wpkh(&child.to_pub(), network.known_hrp()).to_string(),
        ScriptType::P2shP2wpkh => {
            Address::p2shwpkh(&child.to_pub(), network.network_kind()).to_string()
        }
        ScriptType::P2tr => {
            Address::p2tr(secp, child.to_x_only_pub(), None, network.known_hrp()).to_string()
        }
    }
}

/// SPEC v0.7 §10.a — infer `CliNetwork` from a parsed `Xpub` when `--network`
/// is absent. `NetworkKind::Test` collapses testnet / signet / regtest into
/// `Testnet` (the bech32 HRP `tb1...` is shared; signet/regtest disambiguation
/// is not encoded in the version-byte prefix). `--network` overrides this when
/// supplied.
fn network_from_xpub(xpub: &bip32::Xpub) -> CliNetwork {
    match xpub.network {
        bitcoin::NetworkKind::Main => CliNetwork::Mainnet,
        bitcoin::NetworkKind::Test => CliNetwork::Testnet,
    }
}

// ============================================================================
// SPEC v0.9.0 §1 item 1 — argv-leakage closure helpers
// ============================================================================

/// `convert`'s per-occurrence advisory emission. Iterates `args.from`
/// (a `Vec<FromInput>` via `ArgAction::Append`) and emits one advisory
/// per inline-secret `--from <node>=` occurrence, plus advisories for
/// `--passphrase <inline>` and `--bip38-passphrase <inline>`. The
/// argv-secret tag is provided by `NodeType::is_argv_secret_bearing()`
/// which widens `is_secret_bearing()` to include MiniKey (Casascius
/// mini-key encoding) per survey §5 row "convert --from minikey=".
fn emit_secret_in_argv_advisories<E: Write>(args: &ConvertArgs, stderr: &mut E) {
    use crate::secret_advisory::secret_in_argv_warning;
    for f in &args.from {
        if !f.node.is_argv_secret_bearing() {
            continue;
        }
        if f.value == "-" {
            continue;
        }
        // v0.26.0 §I1 fold: skip sentinel-bearing values; user opted into
        // the @env: leak-mitigation channel.
        if f.value.starts_with("@env:") {
            continue;
        }
        let node = f.node.as_str();
        let flag = format!("--from {node}=");
        let alt = format!("--from {node}=-");
        secret_in_argv_warning(stderr, &flag, &alt);
    }
    if let Some(pp) = args.passphrase.as_deref() {
        if !pp.starts_with("@env:") {
            secret_in_argv_warning(stderr, "--passphrase", "--passphrase-stdin");
        }
    }
    if let Some(bpp) = args.bip38_passphrase.as_deref() {
        if !bpp.starts_with("@env:") {
            secret_in_argv_warning(stderr, "--bip38-passphrase", "--bip38-passphrase-stdin");
        }
    }
}

/// v0.26.0 §3 — cheap pre-check for `@env:` sentinels on `convert`'s
/// secret-bearing flag surfaces (`--passphrase`, `--bip38-passphrase`, and
/// secret-bearing `--from <node>=` values).
fn needs_env_sentinel_resolution(args: &ConvertArgs) -> bool {
    let pp = args
        .passphrase
        .as_deref()
        .map(|v| v.starts_with("@env:"))
        .unwrap_or(false);
    let bip38_pp = args
        .bip38_passphrase
        .as_deref()
        .map(|v| v.starts_with("@env:"))
        .unwrap_or(false);
    let from = args
        .from
        .iter()
        .any(|f| f.node.is_secret_bearing() && f.value.starts_with("@env:"));
    pp || bip38_pp || from
}

/// v0.26.0 §3 — resolve `@env:<VAR>` sentinels across `convert`'s
/// secret-bearing flag surfaces. Per SPEC §3.2, resolution is opt-in
/// per-callsite: only secret-bearing flags are scanned.
fn resolve_env_sentinels(args: &ConvertArgs) -> Result<ConvertArgs, ToolkitError> {
    use crate::env_sentinel::resolve_env_var_sentinel;
    let mut owned = args.clone();
    if let Some(pp) = owned.passphrase.as_ref() {
        owned.passphrase = Some(resolve_env_var_sentinel(pp, "--passphrase")?);
    }
    if let Some(bp) = owned.bip38_passphrase.as_ref() {
        owned.bip38_passphrase = Some(resolve_env_var_sentinel(bp, "--bip38-passphrase")?);
    }
    for f in owned.from.iter_mut() {
        if f.node.is_secret_bearing() {
            let flag = format!("--from {}=", f.node.as_str());
            f.value = resolve_env_var_sentinel(&f.value, &flag)?;
        }
    }
    Ok(owned)
}

#[cfg(test)]
mod secret_taxonomy_parity_tests {
    use super::NodeType;
    use mnemonic_toolkit::secret_taxonomy::SECRET_NODE_TYPES;

    /// Declare the complete list of `NodeType` variants exactly once.
    /// This macro produces BOTH (a) the `ALL_NODE_TYPE_VARIANTS` const
    /// array and (b) a `#[cfg(test)] fn _exhaustiveness_check` that
    /// pattern-matches every variant via `|`-alternatives — no wildcard.
    ///
    /// **Why this shape**: the array literal and the match share a
    /// single source-of-truth list. Adding a new `NodeType::FooBar`
    /// variant to the enum makes `_exhaustiveness_check`'s match
    /// non-exhaustive → **compile error**. The contributor MUST
    /// extend the macro's input list to fix the compile error, which
    /// AUTOMATICALLY extends `ALL_NODE_TYPE_VARIANTS` too (same input
    /// expands to both outputs). The parity tests then iterate
    /// `ALL_NODE_TYPE_VARIANTS` — guaranteed to include every enum
    /// variant — and assert each variant's `as_str()` membership in
    /// `SECRET_NODE_TYPES` matches its `is_secret_bearing()` predicate.
    ///
    /// Net guarantee: a future contributor cannot add a secret-bearing
    /// `NodeType` variant without also updating `SECRET_NODE_TYPES` in
    /// `secret_taxonomy.rs`. The parity-test assertion fires.
    macro_rules! declare_node_type_variants {
        ( $( $variant:ident ),* $(,)? ) => {
            const ALL_NODE_TYPE_VARIANTS: &[NodeType] =
                &[ $( NodeType::$variant ),* ];

            #[allow(dead_code)]
            fn _exhaustiveness_check(v: NodeType) {
                match v {
                    $( NodeType::$variant )|* => (),
                }
            }
        };
    }

    declare_node_type_variants!(
        Phrase,
        Seedqr,
        Entropy,
        Xpub,
        Xprv,
        Wif,
        Fingerprint,
        Path,
        Ms1,
        Mk1,
        Bip38,
        MiniKey,
        ElectrumPhrase,
        Address,
    );

    #[test]
    fn secret_taxonomy_parity_with_is_secret_bearing() {
        for &v in ALL_NODE_TYPE_VARIANTS {
            let predicate = v.is_secret_bearing();
            let in_taxonomy = SECRET_NODE_TYPES.contains(&v.as_str());
            assert_eq!(
                predicate, in_taxonomy,
                "drift: NodeType::{:?}.is_secret_bearing()={} but \
                 secret_taxonomy::SECRET_NODE_TYPES.contains({:?})={}. \
                 If you added a NodeType variant, the macro expansion \
                 above means `ALL_NODE_TYPE_VARIANTS` already includes \
                 it — so this assertion is firing because the variant's \
                 secret-class status disagrees between \
                 `is_secret_bearing()` (`cmd/convert.rs`) and \
                 `secret_taxonomy::SECRET_NODE_TYPES` \
                 (`src/secret_taxonomy.rs`). Bring them into agreement.",
                v,
                predicate,
                v.as_str(),
                in_taxonomy,
            );
        }
    }

    #[test]
    fn secret_taxonomy_entries_round_trip_via_from_token() {
        for &token in SECRET_NODE_TYPES {
            let parsed = NodeType::from_token(token).unwrap_or_else(|| {
                panic!(
                    "secret_taxonomy::SECRET_NODE_TYPES entry {:?} does \
                     not parse as a NodeType via from_token — drift",
                    token
                )
            });
            assert_eq!(parsed.as_str(), token);
            assert!(
                parsed.is_secret_bearing(),
                "secret_taxonomy::SECRET_NODE_TYPES contains {:?} but \
                 NodeType::{:?}.is_secret_bearing()=false — drift",
                token,
                parsed,
            );
        }
    }

    /// MiniKey is intentionally excluded from `SECRET_NODE_TYPES` even
    /// though `is_argv_secret_bearing()` includes it — see
    /// SPEC_secret_memory_hygiene §1 and the
    /// `convert-minikey-stdout-redaction` FOLLOWUP. This test pins the
    /// intentional asymmetry so a future refactor that widens
    /// `is_secret_bearing()` to include MiniKey forces a coordinated
    /// update of the persistence-class consumers.
    #[test]
    fn minikey_intentionally_excluded_from_persistence_taxonomy() {
        assert!(!NodeType::MiniKey.is_secret_bearing());
        assert!(NodeType::MiniKey.is_argv_secret_bearing());
        assert!(!SECRET_NODE_TYPES.contains(&"minikey"));
    }
}

#[cfg(test)]
mod script_type_tests {
    use super::*;

    #[test]
    fn script_type_as_str_round_trips() {
        for st in [ScriptType::P2pkh, ScriptType::P2wpkh, ScriptType::P2shP2wpkh, ScriptType::P2tr] {
            assert_eq!(parse_script_type_arg(st.as_str()).unwrap(), st);
        }
    }
}
