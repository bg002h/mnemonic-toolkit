//! v0.28.0 Phase P3 — Coldcard single-sig wallet.json parser.
//!
//! Per `design/SPEC_wallet_import_v0_28_0.md` §11.3.
//!
//! Coldcard's generic-wallet-export JSON shape (canonical authority:
//! `coldcard/firmware/shared/generic_wallet_export.py` +
//! `coldcard/firmware/docs/generic-wallet-export.md` at upstream master) is
//! a single JSON object with these top-level fields:
//!
//! ```json
//! {
//!   "chain": "BTC"   // or "XTN"
//!   "xfp": "<8-hex-uppercase-master-fingerprint>",
//!   "xpub": "<optional-master-xpub>",
//!   "account": <u32 account number>,
//!   "bip44": { "name": "p2pkh",        "deriv": "m/44'/<coin>'/<acct>'", "xfp": "<hex>", "xpub": "<account-xpub>", "first": "<addr>" },
//!   "bip49": { "name": "p2wpkh-p2sh",  "deriv": "m/49'/<coin>'/<acct>'", "xfp": "<hex>", "xpub": "<account-xpub>", "_pub": "<ypub>", "first": "<addr>" },
//!   "bip84": { "name": "p2wpkh",       "deriv": "m/84'/<coin>'/<acct>'", "xfp": "<hex>", "xpub": "<account-xpub>", "_pub": "<zpub>", "first": "<addr>" },
//!   "bip86": { "name": "p2tr",         "deriv": "m/86'/<coin>'/<acct>'", "xfp": "<hex>", "xpub": "<account-xpub>", "first": "<addr>" },
//!   "bip48_1": { ... },   // BIP-48 multisig hint (P2SH-P2WSH); IGNORED by single-sig parser
//!   "bip48_2": { ... }    // BIP-48 multisig hint (P2WSH); IGNORED by single-sig parser
//! }
//! ```
//!
//! Firmware-variance handling (SPEC §11.3 firmware-variance table):
//! - Mk1/Mk2 pre-2022: `xpub` top-level only (single BIP-44 wallet).
//! - Mk3+: per-bipNN sub-objects (`bip44`/`bip49`/`bip84`).
//! - Mk4+: adds `bip86` (taproot).
//! - Q (2024+): adds `bip48_1`/`bip48_2` multisig hints.
//!
//! Sniff signature (SPEC §11.3 Q3 lock, R0 I8 relaxed): top-level JSON
//! object with `chain ∈ {BTC, XTN}` + `xfp` + at-least-one-of
//! `{xpub, bip44, bip49, bip84, bip86, bip48_1, bip48_2}`.
//!
//! Parse contract:
//! 1. JSON-parse + top-level object check.
//! 2. Extract `chain` → network (BTC → mainnet, XTN → testnet).
//! 3. Extract `xfp` → master fingerprint.
//! 4. Dominant-BIP selection per SPEC §11.3.1: bip86 > bip84 > bip49 > bip44
//!    (with top-level `xpub` legacy-firmware fallback inferring BIP from
//!    SLIP-132 prefix).
//! 5. Build synthetic descriptor: `<wrapper>([xfp/deriv_no_m]xpub/<0;1>/*)#<csum>`
//!    where wrapper ∈ {pkh, sh(wpkh), wpkh, tr} per dominant BIP.
//! 6. Feed through `concrete_keys_to_placeholders` → `parse_descriptor`.
//! 7. Build single ResolvedSlot.
//! 8. Wrap in `ParsedImport` with `ImportProvenance::Coldcard(...)`.
//!
//! Phase P3A scope: parser skeleton + sniff impl + provenance metadata
//! struct decls + sniff unit tests. `parse()` returns
//! `Err(BadInput("P3B: parse not yet wired"))` — Phase P3B installs the
//! real body; Phase P3C flips the `cmd/import_wallet.rs` dispatch sites.

use super::{
    pipeline::concrete_keys_to_placeholders, validate_watch_only_resolved, ImportProvenance,
    ParsedImport, WalletFormatParser,
};
use crate::error::ToolkitError;
use crate::parse_descriptor;
use crate::synthesize::{xpub_to_65, ResolvedSlot};
use bitcoin::bip32::{ChildNumber, DerivationPath, Fingerprint, Xpub};
use serde_json::Value;
use std::io::Write;
use std::str::FromStr;

/// SPEC §11.3 — Coldcard single-sig wallet.json parser.
pub(crate) struct ColdcardParser;

/// SPEC §11.3 — `chain` field discriminator. `BTC` → mainnet, `XTN` → testnet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColdcardChain {
    Btc,
    Xtn,
}

impl ColdcardChain {
    /// Map to `bitcoin::Network`.
    pub(crate) fn to_network(self) -> bitcoin::Network {
        match self {
            ColdcardChain::Btc => bitcoin::Network::Bitcoin,
            ColdcardChain::Xtn => bitcoin::Network::Testnet,
        }
    }
}

/// SPEC §11.3.1 — Coldcard's dominant-BIP selection result. The single-sig
/// parser picks ONE of these per blob; bip48_* multisig hints are explicitly
/// excluded (multisig case is `--format coldcard-multisig`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColdcardBip {
    Bip44,
    Bip49,
    Bip84,
    Bip86,
}

impl ColdcardBip {
    /// JSON sub-object key for this BIP (e.g., `"bip44"`).
    pub(crate) fn as_json_key(self) -> &'static str {
        match self {
            ColdcardBip::Bip44 => "bip44",
            ColdcardBip::Bip49 => "bip49",
            ColdcardBip::Bip84 => "bip84",
            ColdcardBip::Bip86 => "bip86",
        }
    }
}

/// SPEC §11.3 — per-blob provenance metadata for a Coldcard single-sig parse.
/// Carried on `ImportProvenance::Coldcard(...)`; preserved for `--json`
/// envelope `coldcard_source_metadata` emit (P3C wiring).
#[derive(Debug, Clone)]
pub(crate) struct ColdcardSourceMetadata {
    /// Top-level `chain` (BTC | XTN).
    pub(crate) chain: ColdcardChain,
    /// Top-level `xfp` decoded to 4-byte master fingerprint.
    pub(crate) xfp: [u8; 4],
    /// Dominant-BIP block selected per SPEC §11.3.1 (bip86 > bip84 > bip49 > bip44).
    pub(crate) bip_derivation: ColdcardBip,
    /// Top-level `account` field (u32). Default 0 if absent (legacy firmware).
    pub(crate) raw_account: u32,
    /// Top-level fields encountered in the blob but not preserved on the
    /// import-side provenance (mirrors `CoreSourceMetadata.dropped_fields`).
    pub(crate) dropped_fields: Vec<String>,
}

/// Top-level keys preserved on the Coldcard envelope by the toolkit's parse.
/// Any other top-level field surfaces in `ColdcardSourceMetadata.dropped_fields`
/// and drives a stderr NOTICE per SPEC §2.4. Mirrors
/// `SPECTER_PRESERVED_TOP_LEVEL_KEYS`.
pub(crate) const COLDCARD_PRESERVED_TOP_LEVEL_KEYS: &[&str] = &[
    "chain", "xfp", "xpub", "account", "bip44", "bip49", "bip84", "bip86",
    "bip48_1", "bip48_2",
];

impl WalletFormatParser for ColdcardParser {
    /// SPEC §11.3 (Q3 lock) sniff: top-level JSON object containing ALL of:
    /// (1) `chain ∈ {"BTC", "XTN"}` as a string,
    /// (2) `xfp` as a string,
    /// (3) at-least-one-of `{xpub, bip44, bip49, bip84, bip86, bip48_1, bip48_2}`.
    ///
    /// The disjunction in (3) absorbs Coldcard firmware variance — different
    /// firmware versions emit different combinations of per-BIP derivation
    /// blocks.
    fn sniff(blob: &[u8]) -> bool {
        let trimmed = trim_leading_ws(blob);
        if !trimmed.starts_with(b"{") {
            return false;
        }
        let value: Value = match serde_json::from_slice(blob) {
            Ok(v) => v,
            Err(_) => return false,
        };
        let obj = match value.as_object() {
            Some(o) => o,
            None => return false,
        };
        // (1) chain: must be string "BTC" or "XTN".
        let chain_ok = obj
            .get("chain")
            .and_then(|v| v.as_str())
            .map(|s| s == "BTC" || s == "XTN")
            .unwrap_or(false);
        if !chain_ok {
            return false;
        }
        // (2) xfp: must be string.
        if obj.get("xfp").and_then(|v| v.as_str()).is_none() {
            return false;
        }
        // (3) at-least-one-of: xpub | bip44 | bip49 | bip84 | bip86 | bip48_1 | bip48_2.
        let has_derivation_marker = [
            "xpub", "bip44", "bip49", "bip84", "bip86", "bip48_1", "bip48_2",
        ]
        .iter()
        .any(|k| obj.contains_key(*k));
        if !has_derivation_marker {
            return false;
        }
        true
    }

    /// SPEC §11.3 — parse a Coldcard single-sig wallet JSON blob.
    ///
    /// Steps:
    /// 1. JSON-parse + top-level object check.
    /// 2. Extract `chain` → network (BTC → mainnet, XTN → testnet).
    /// 3. Decode `xfp` → 4-byte master fingerprint.
    /// 4. Extract optional `account` (default 0).
    /// 5. Dominant-BIP selection per SPEC §11.3.1:
    ///    bip86 > bip84 > bip49 > bip44 > legacy-top-level-xpub (SLIP-132 infer).
    /// 6. Build synthetic descriptor with `[xfp/deriv_no_m]xpub/<0;1>/*` shape,
    ///    wrapper per BIP (`pkh`, `sh(wpkh(...))`, `wpkh`, `tr`).
    /// 7. Feed through `concrete_keys_to_placeholders` → `parse_descriptor`.
    /// 8. Build single ResolvedSlot (single-sig: exactly one cosigner).
    /// 9. Emit stderr NOTICE per SPEC §2.4 listing dropped envelope fields.
    /// 10. Wrap in `ParsedImport` with `ImportProvenance::Coldcard(...)`.
    fn parse(blob: &[u8], stderr: &mut dyn Write) -> Result<Vec<ParsedImport>, ToolkitError> {
        // Step 1: JSON parse.
        let value: Value = serde_json::from_slice(blob).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: coldcard: parse error: invalid JSON: {e}"
            ))
        })?;
        let obj = value.as_object().ok_or_else(|| {
            ToolkitError::ImportWalletParse(
                "import-wallet: coldcard: parse error: top-level JSON value is not an object"
                    .to_string(),
            )
        })?;

        // Step 2: chain → network.
        let chain_str = obj
            .get("chain")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(
                    "import-wallet: coldcard: parse error: missing or non-string top-level `chain`"
                        .to_string(),
                )
            })?;
        let chain = match chain_str {
            "BTC" => ColdcardChain::Btc,
            "XTN" => ColdcardChain::Xtn,
            other => {
                return Err(ToolkitError::ImportWalletParse(format!(
                    "import-wallet: coldcard: parse error: `chain` must be \"BTC\" or \"XTN\", got {other:?}"
                )));
            }
        };
        let network = chain.to_network();

        // Step 3: decode xfp.
        let xfp_str = obj
            .get("xfp")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(
                    "import-wallet: coldcard: parse error: missing or non-string top-level `xfp`"
                        .to_string(),
                )
            })?;
        let xfp = parse_xfp_hex(xfp_str)?;

        // Step 4: optional account (default 0).
        let raw_account = obj
            .get("account")
            .and_then(|v| v.as_u64())
            .map(|n| n as u32)
            .unwrap_or(0);

        // Step 5: dominant-BIP selection per SPEC §11.3.1.
        let (bip_derivation, account_xpub_str, deriv_path_str_opt) =
            select_dominant_bip(obj, network)?;

        // Step 6: build synthetic descriptor.
        //
        // For per-bipN sub-objects: the sub-object's `deriv` field carries the
        // canonical BIP-44/49/84/86 origin path (`m/<purpose>'/<coin>'/<acct>'`).
        // For legacy top-level-xpub fallback: deriv is inferred from BIP purpose
        // + chain coin-type + account.
        let deriv_str = match deriv_path_str_opt {
            Some(s) => s,
            None => {
                let coin_type = match chain {
                    ColdcardChain::Btc => 0,
                    ColdcardChain::Xtn => 1,
                };
                let purpose = match bip_derivation {
                    ColdcardBip::Bip44 => 44,
                    ColdcardBip::Bip49 => 49,
                    ColdcardBip::Bip84 => 84,
                    ColdcardBip::Bip86 => 86,
                };
                format!("m/{purpose}'/{coin_type}'/{raw_account}'")
            }
        };
        // Strip leading `m/` for bracket form.
        let deriv_no_m = deriv_str
            .strip_prefix("m/")
            .unwrap_or(deriv_str.strip_prefix('m').unwrap_or(&deriv_str));

        // Normalize the account xpub (handles ypub/zpub/upub/vpub SLIP-132
        // variants by mapping to neutral xpub/tpub form). Coldcard's per-bipN
        // sub-object's `xpub` field IS already the neutral xpub/tpub form per
        // upstream sample (see `wallet_export/coldcard.rs:203` — toolkit's own
        // emitter does this); the legacy top-level `xpub` fallback may carry
        // a SLIP-132 variant prefix.
        let (neutral_xpub_str, _slip132_variant) =
            crate::slip0132::normalize_xpub_prefix(&account_xpub_str).map_err(|e| {
                ToolkitError::ImportWalletParse(format!(
                    "import-wallet: coldcard: parse error: xpub normalize: {}",
                    e.message()
                ))
            })?;

        // Build bracketed descriptor: `<wrapper>([xfp/deriv_no_m]xpub/<0;1>/*)`.
        let xfp_hex_lower = format!(
            "{:02x}{:02x}{:02x}{:02x}",
            xfp[0], xfp[1], xfp[2], xfp[3]
        );
        let bracketed = format!(
            "[{xfp_hex_lower}/{deriv_no_m}]{neutral_xpub_str}/<0;1>/*"
        );
        let wrapped = match bip_derivation {
            ColdcardBip::Bip44 => format!("pkh({bracketed})"),
            ColdcardBip::Bip49 => format!("sh(wpkh({bracketed}))"),
            ColdcardBip::Bip84 => format!("wpkh({bracketed})"),
            ColdcardBip::Bip86 => format!("tr({bracketed})"),
        };

        // Step 7: feed through the existing pipeline.
        let (placeholder_form, parsed_keys, parsed_fingerprints) =
            concrete_keys_to_placeholders(&wrapped).map_err(|e| {
                ToolkitError::ImportWalletParse(e.message().replacen(
                    "import-wallet: bsms:",
                    "import-wallet: coldcard:",
                    1,
                ))
            })?;

        let descriptor = parse_descriptor::parse_descriptor(
            &placeholder_form,
            &parsed_keys,
            &parsed_fingerprints,
        )
        .map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: coldcard: parse error: {}",
                e.message()
            ))
        })?;

        // Step 8: single ResolvedSlot.
        let (xpub, fp, path, path_raw) = build_slot_fields(&wrapped)?;
        debug_assert_eq!(xpub_to_65(&xpub), parsed_keys[0].payload);
        let cosigners: Vec<ResolvedSlot> = vec![ResolvedSlot {
            xpub,
            fingerprint: fp,
            path,
            path_raw,
            entropy: None,
            master_xpub: None,
            _entropy_pin: None,
        }];
        validate_watch_only_resolved(&cosigners)?;

        // Single-sig: threshold is None.
        let threshold: Option<u8> = None;

        // Step 9: dropped-field detection + stderr NOTICE.
        let mut dropped_fields: Vec<String> = Vec::new();
        for (k, _) in obj.iter() {
            if !COLDCARD_PRESERVED_TOP_LEVEL_KEYS.contains(&k.as_str()) {
                dropped_fields.push(k.clone());
            }
        }
        if !dropped_fields.is_empty() {
            writeln!(
                stderr,
                "notice: import-wallet: coldcard: dropped envelope fields {}: not preserved in bundle output (key-state only)",
                dropped_fields.join(", ")
            )
            .map_err(ToolkitError::Io)?;
        }

        let source_metadata = ColdcardSourceMetadata {
            chain,
            xfp,
            bip_derivation,
            raw_account,
            dropped_fields,
        };

        // Step 10: reconstruct original_descriptor with a freshly-computed
        // BIP-380 checksum. On checksum-engine failure (non-ASCII), fall back
        // to verbatim. Mirrors `wallet_import/specter.rs:301` discipline.
        let original_descriptor = match recompute_descriptor_checksum(&wrapped) {
            Ok(s) => s,
            Err(_) => wrapped.clone(),
        };

        Ok(vec![ParsedImport {
            descriptor,
            original_descriptor,
            cosigners,
            network,
            threshold,
            provenance: ImportProvenance::Coldcard(source_metadata),
        }])
    }
}

/// Decode an 8-hex-uppercase (or lowercase; we tolerate both) xfp string into
/// a 4-byte fingerprint array. Coldcard upstream emits uppercase per
/// `wallet_export/coldcard.rs:155`'s `to_uppercase()` step; users hand-editing
/// blobs might lowercase it — both forms accepted.
fn parse_xfp_hex(s: &str) -> Result<[u8; 4], ToolkitError> {
    if s.len() != 8 {
        return Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: coldcard: parse error: `xfp` must be 8 hex characters, got {} characters",
            s.len()
        )));
    }
    let mut out = [0u8; 4];
    for i in 0..4 {
        out[i] = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: coldcard: parse error: `xfp` hex decode: {e}"
            ))
        })?;
    }
    Ok(out)
}

/// SPEC §11.3.1 dominant-BIP selection. Returns `(bip, account_xpub_string,
/// derivation_path_string_or_none)`.
///
/// Order: bip86 > bip84 > bip49 > bip44 > legacy-top-level-xpub. bip48_*
/// blocks are ignored (multisig case is `--format coldcard-multisig`).
///
/// For per-bipN sub-objects: extracts `xpub` + `deriv` fields. Legacy fallback
/// infers BIP from the top-level xpub's SLIP-132 prefix (zpub → BIP-84,
/// ypub → BIP-49, xpub → BIP-44; tpub/upub/vpub on XTN map similarly).
fn select_dominant_bip(
    obj: &serde_json::Map<String, Value>,
    network: bitcoin::Network,
) -> Result<(ColdcardBip, String, Option<String>), ToolkitError> {
    // Order: 86 > 84 > 49 > 44.
    for bip in [
        ColdcardBip::Bip86,
        ColdcardBip::Bip84,
        ColdcardBip::Bip49,
        ColdcardBip::Bip44,
    ] {
        let key = bip.as_json_key();
        if let Some(sub) = obj.get(key).and_then(|v| v.as_object()) {
            let xpub = sub
                .get("xpub")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    ToolkitError::ImportWalletParse(format!(
                        "import-wallet: coldcard: parse error: `{key}.xpub` missing or not a string"
                    ))
                })?
                .to_string();
            let deriv = sub
                .get("deriv")
                .and_then(|v| v.as_str())
                .ok_or_else(|| {
                    ToolkitError::ImportWalletParse(format!(
                        "import-wallet: coldcard: parse error: `{key}.deriv` missing or not a string"
                    ))
                })?
                .to_string();
            return Ok((bip, xpub, Some(deriv)));
        }
    }
    // Fallback: legacy top-level xpub (Mk1/Mk2 firmware). Infer BIP from
    // SLIP-132 prefix.
    if let Some(xpub_str) = obj.get("xpub").and_then(|v| v.as_str()) {
        let bip = infer_bip_from_xpub_prefix(xpub_str, network)?;
        return Ok((bip, xpub_str.to_string(), None));
    }
    Err(ToolkitError::ImportWalletParse(
        "import-wallet: coldcard: parse error: no dominant derivation block found; \
         expected one of {bip86, bip84, bip49, bip44, xpub (legacy)} at top level"
            .to_string(),
    ))
}

/// Legacy-fallback BIP inference from top-level xpub's SLIP-132 prefix.
/// zpub/vpub → BIP-84, ypub/upub → BIP-49, xpub/tpub → BIP-44.
/// (BIP-86 has no SLIP-132 prefix variant; legacy firmware never emitted bip86.)
fn infer_bip_from_xpub_prefix(
    xpub_str: &str,
    _network: bitcoin::Network,
) -> Result<ColdcardBip, ToolkitError> {
    let prefix_4 = xpub_str
        .get(..4)
        .ok_or_else(|| {
            ToolkitError::ImportWalletParse(
                "import-wallet: coldcard: parse error: legacy top-level `xpub` is too short to identify SLIP-132 variant"
                    .to_string(),
            )
        })?;
    match prefix_4 {
        "xpub" | "tpub" => Ok(ColdcardBip::Bip44),
        "ypub" | "upub" => Ok(ColdcardBip::Bip49),
        "zpub" | "vpub" => Ok(ColdcardBip::Bip84),
        other => Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: coldcard: parse error: legacy top-level `xpub` has unrecognized SLIP-132 prefix {other:?}; expected one of xpub/tpub/ypub/upub/zpub/vpub"
        ))),
    }
}

/// Build the typed slot fields (xpub, fingerprint, path, path_raw) from the
/// synthesized descriptor body. Mirrors `wallet_import/specter.rs:398`
/// `build_slot_fields` — coldcard descriptors carry exactly one origin
/// annotation (single-sig: exactly one slot).
fn build_slot_fields(
    descriptor_body: &str,
) -> Result<(Xpub, Fingerprint, DerivationPath, String), ToolkitError> {
    use regex::Regex;
    use std::sync::OnceLock;
    static R: OnceLock<Regex> = OnceLock::new();
    let re = R.get_or_init(|| {
        Regex::new(r"\[([0-9a-fA-F]{8})((?:/\d+'?)+)\]([xtyzuvYZUV]pub[A-HJ-NP-Za-km-z1-9]+)")
            .expect("origin_capture_regex is a fixed string literal")
    });
    let cap = re.captures(descriptor_body).ok_or_else(|| {
        ToolkitError::ImportWalletParse(
            "import-wallet: coldcard: parse error: no origin annotation in synthesized descriptor (internal bug)"
                .to_string(),
        )
    })?;
    let fp_hex = cap.get(1).expect("group 1").as_str();
    let path_raw_inner = cap.get(2).expect("group 2").as_str();
    let xpub_str = cap.get(3).expect("group 3").as_str();

    let mut fp_bytes = [0u8; 4];
    for i in 0..4 {
        fp_bytes[i] = u8::from_str_radix(&fp_hex[i * 2..i * 2 + 2], 16).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: coldcard: parse error: fingerprint hex: {e}"
            ))
        })?;
    }
    let fp = Fingerprint::from(fp_bytes);
    let path = DerivationPath::from_str(&format!("m{path_raw_inner}")).map_err(|e| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: coldcard: parse error: derivation-path parse: {e}"
        ))
    })?;
    let path_raw = format!("[{fp_hex}{path_raw_inner}]");

    let (neutral, _variant) = crate::slip0132::normalize_xpub_prefix(xpub_str)?;
    let xpub = Xpub::from_str(&neutral).map_err(|e| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: coldcard: parse error: xpub decode: {e}"
        ))
    })?;
    Ok((xpub, fp, path, path_raw))
}

/// Recompute the BIP-380 checksum for the descriptor body. Mirrors
/// `wallet_import/specter.rs:484`'s `recompute_descriptor_checksum` shape.
fn recompute_descriptor_checksum(body: &str) -> Result<String, ToolkitError> {
    use miniscript::descriptor::checksum::Engine as ChecksumEngine;
    let body_no_csum = match body.rsplit_once('#') {
        Some((b, _)) => b,
        None => body,
    };
    let mut eng = ChecksumEngine::new();
    eng.input(body_no_csum).map_err(|e| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: coldcard: checksum engine input rejected: {e}"
        ))
    })?;
    let csum = eng.checksum();
    Ok(format!("{body_no_csum}#{csum}"))
}

/// Reference the `ChildNumber` import to suppress unused-warning when the
/// pattern is consumed via `DerivationPath::from_str` (full-path-string
/// roundtrip).
#[allow(dead_code)]
const _CHILD_NUMBER_USED: fn(ChildNumber) = |_| {};

/// Strip ASCII leading whitespace before checking for `{` prefix. Mirrors
/// the helper in `wallet_import/specter.rs:503` (`trim_leading_ws`).
fn trim_leading_ws(blob: &[u8]) -> &[u8] {
    let mut i = 0;
    while i < blob.len()
        && (blob[i] == b' ' || blob[i] == b'\t' || blob[i] == b'\n' || blob[i] == b'\r')
    {
        i += 1;
    }
    &blob[i..]
}


#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Sniff: positive cases (SPEC §11.3 Q3 lock — chain + xfp + ≥1 derivation marker)
    // -------------------------------------------------------------------------

    #[test]
    fn sniff_true_on_modern_bip84_blob() {
        let blob = br#"{
            "chain": "BTC",
            "xfp": "B8688DF1",
            "account": 0,
            "bip84": {
                "name": "p2wpkh",
                "deriv": "m/84'/0'/0'",
                "xfp": "B8688DF1",
                "xpub": "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX",
                "_pub": "zpubDFXrR8dxAH7gFqHkw9JvNXqVkPiTMfb4P4n2RvBT3PSnD3iJWHsodaR7g2ND2VPiR1iCqXcLqCCdKM7ZN3Hh3hQrFqdjsLkhBwYHbLAQt2T",
                "first": "bc1qjyf0xzn0eyl9d0glujytdq2t5kdq0u4lcj6xtg"
            }
        }"#;
        assert!(ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_true_on_legacy_xpub_only_blob() {
        // SPEC §11.3 firmware-variance table: Mk1/Mk2 firmware emitted only
        // a top-level `xpub` (no per-bipN blocks). Sniff still accepts via
        // the at-least-one-of disjunction.
        let blob = br#"{
            "chain": "BTC",
            "xfp": "B8688DF1",
            "xpub": "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX"
        }"#;
        assert!(ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_true_on_multi_bip_blob() {
        // Modern Coldcard firmware emits BOTH bip44, bip49, bip84 in one
        // envelope (and optionally bip86 / bip48_*). Sniff is shape-only —
        // dominant-BIP selection happens at parse time per SPEC §11.3.1.
        let blob = br#"{
            "chain": "BTC",
            "xfp": "B8688DF1",
            "account": 0,
            "bip44": {"name":"p2pkh","deriv":"m/44'/0'/0'","xfp":"B8688DF1","xpub":"xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX","first":"1FRBR4iY3XQhytKwgZmnjJyCSGsXHm9gBL"},
            "bip49": {"name":"p2wpkh-p2sh","deriv":"m/49'/0'/0'","xfp":"B8688DF1","xpub":"xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX","_pub":"ypubDExampleYpub","first":"3FZ..."},
            "bip84": {"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"B8688DF1","xpub":"xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX","_pub":"zpubDExampleZpub","first":"bc1q..."}
        }"#;
        assert!(ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_true_on_bip86_taproot_blob() {
        // Mk4+ firmware adds bip86 (taproot).
        let blob = br#"{
            "chain": "BTC",
            "xfp": "B8688DF1",
            "account": 0,
            "bip86": {"name":"p2tr","deriv":"m/86'/0'/0'","xfp":"B8688DF1","xpub":"xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX","first":"bc1p..."}
        }"#;
        assert!(ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_true_on_testnet_xtn_blob() {
        let blob = br#"{
            "chain": "XTN",
            "xfp": "704C7836",
            "account": 0,
            "bip84": {"name":"p2wpkh","deriv":"m/84'/1'/0'","xfp":"704C7836","xpub":"tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC","_pub":"vpubDExampleVpub","first":"tb1q..."}
        }"#;
        assert!(ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_true_on_bip48_only_blob() {
        // Q firmware adds bip48_1/bip48_2 multisig hints. They are sniff-positive
        // markers but the single-sig parser ignores them at parse time (multisig
        // case is `--format coldcard-multisig`). Sniff is shape-only — at-least-
        // one-of disjunction means bip48_* alone satisfies sniff.
        let blob = br#"{
            "chain": "BTC",
            "xfp": "B8688DF1",
            "account": 0,
            "bip48_2": {"name":"p2wsh","deriv":"m/48'/0'/0'/2'","xfp":"B8688DF1","xpub":"xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX"}
        }"#;
        assert!(ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_tolerates_leading_whitespace() {
        let blob = br#"
        {
            "chain": "BTC",
            "xfp": "B8688DF1",
            "xpub": "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX"
        }"#;
        assert!(ColdcardParser::sniff(blob));
    }

    // -------------------------------------------------------------------------
    // Sniff: negative cases
    // -------------------------------------------------------------------------

    #[test]
    fn sniff_false_on_missing_chain() {
        let blob = br#"{"xfp":"B8688DF1","xpub":"xpub..."}"#;
        assert!(!ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_chain_main_not_btc() {
        // Bitcoin Core uses `"chain": "main"` — Coldcard uses `"BTC"`. The
        // sniff must reject `main` to keep the format-disambiguation strict.
        let blob = br#"{"chain":"main","xfp":"B8688DF1","xpub":"xpub..."}"#;
        assert!(!ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_chain_test_not_xtn() {
        let blob = br#"{"chain":"test","xfp":"704C7836","xpub":"tpub..."}"#;
        assert!(!ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_missing_xfp() {
        let blob = br#"{"chain":"BTC","xpub":"xpub..."}"#;
        assert!(!ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_xfp_not_string() {
        // `xfp` is conventionally an uppercase hex string, NOT an integer.
        // Coldcard firmware emits it as a string; the sniff must reject the
        // wrong-type form to keep format disambiguation strict.
        let blob = br#"{"chain":"BTC","xfp":3094905841,"xpub":"xpub..."}"#;
        assert!(!ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_chain_only_no_derivation_marker() {
        // chain + xfp alone is not enough — need at least one of the
        // derivation markers per the Q3 disjunction.
        let blob = br#"{"chain":"BTC","xfp":"B8688DF1","account":0}"#;
        assert!(!ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_invalid_json() {
        let blob = br#"{"chain":"BTC","xfp":"B8688DF1","xpub":"xpub..."#;
        assert!(!ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_top_level_array() {
        let blob = br#"[{"chain":"BTC","xfp":"B8688DF1","xpub":"xpub..."}]"#;
        assert!(!ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_empty_blob() {
        assert!(!ColdcardParser::sniff(b""));
    }

    #[test]
    fn sniff_false_on_random_text() {
        assert!(!ColdcardParser::sniff(b"some random text\n"));
    }

    // -------------------------------------------------------------------------
    // Sniff: cross-format negative — must NOT match other vendor blobs
    // -------------------------------------------------------------------------

    #[test]
    fn sniff_false_on_bsms_blob() {
        let blob = b"BSMS 1.0\nwpkh([deadbeef/84'/0'/0']xpub.../0/*)#abcdefgh\n";
        assert!(!ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_bitcoin_core_listdescriptors() {
        // Bitcoin Core `listdescriptors` lacks `chain` and `xfp` keys; the
        // sniff must reject.
        let blob = br#"{"wallet_name":"a","descriptors":[{"desc":"wpkh(xpub.../<0;1>/*)#abcdefgh"}]}"#;
        assert!(!ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_specter_blob() {
        // Specter carries `label`+`blockheight`+`descriptor`+`devices`; lacks
        // Coldcard's `chain`+`xfp`.
        let blob = br#"{
            "label":"Daily","blockheight":800000,
            "descriptor":"wpkh([5436d724/84'/0'/0']xpub.../<0;1>/*)#abcdefgh",
            "devices":[{"type":"coldcard","label":"primary"}]
        }"#;
        assert!(!ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_sparrow_blob() {
        // Sparrow lacks `chain`+`xfp` at top level.
        let blob = br#"{
            "policyType":"SINGLE","scriptType":"P2WPKH",
            "defaultPolicy":{"miniscript":{"script":"wpkh(@0/**)"}},
            "keystores":[{"keyDerivation":{"masterFingerprint":"5436d724","derivation":"m/84'/0'/0'"},
                "extendedPublicKey":"xpub..."}]
        }"#;
        assert!(!ColdcardParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_coldcard_multisig_text() {
        // Coldcard multisig is text-shape (not JSON) — leads with `Name:`.
        let blob = b"Name: ms-2of3\nPolicy: 2 of 3\nDerivation: m/48'/0'/0'/2'\nFormat: P2WSH\n";
        assert!(!ColdcardParser::sniff(blob));
    }

    // ========================================================================
    // P3B PARSE cells — dominant-BIP selection + slot construction
    // ========================================================================

    fn parse(blob: &[u8]) -> Result<Vec<ParsedImport>, ToolkitError> {
        let mut stderr = Vec::new();
        ColdcardParser::parse(blob, &mut stderr)
    }

    fn parse_capturing_stderr(blob: &[u8]) -> (Result<Vec<ParsedImport>, ToolkitError>, String) {
        let mut stderr = Vec::new();
        let r = ColdcardParser::parse(blob, &mut stderr);
        (r, String::from_utf8(stderr).unwrap_or_default())
    }

    // Reusable test xpubs from `core-bip84-mainnet.json` / `core-bip49-mainnet.json`.
    const XPUB_84: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
    const XPUB_49: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";
    const TPUB_84_TESTNET: &str = "tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC";

    #[test]
    fn parse_bip84_mainnet_happy_path() {
        let blob = format!(
            r#"{{
                "chain":"BTC","xfp":"B8688DF1","account":0,
                "bip84": {{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"B8688DF1","xpub":"{XPUB_84}","first":"bc1q..."}}
            }}"#,
        );
        let parsed = parse(blob.as_bytes()).unwrap();
        assert_eq!(parsed.len(), 1);
        let p = &parsed[0];
        assert_eq!(p.cosigners.len(), 1);
        assert_eq!(p.network, bitcoin::Network::Bitcoin);
        assert_eq!(p.threshold, None);
        assert_eq!(p.cosigners[0].fingerprint.to_string(), "b8688df1");
        // Origin path components must match BIP-84 m/84'/0'/0'.
        let path_str = format!("{}", p.cosigners[0].path);
        assert!(
            path_str == "84'/0'/0'" || path_str == "m/84'/0'/0'",
            "expected BIP-84 path; got: {path_str:?}"
        );
        match &p.provenance {
            ImportProvenance::Coldcard(m) => {
                assert_eq!(m.chain, ColdcardChain::Btc);
                assert_eq!(m.bip_derivation, ColdcardBip::Bip84);
                assert_eq!(m.raw_account, 0);
                assert_eq!(m.xfp, [0xB8, 0x68, 0x8D, 0xF1]);
            }
            other => panic!("expected Coldcard provenance, got: {other:?}"),
        }
        // wpkh wrapper.
        assert!(
            p.original_descriptor.starts_with("wpkh("),
            "BIP-84 must emit wpkh(...); got: {}",
            p.original_descriptor
        );
    }

    #[test]
    fn parse_bip44_mainnet_happy_path() {
        let blob = format!(
            r#"{{
                "chain":"BTC","xfp":"B8688DF1","account":0,
                "bip44": {{"name":"p2pkh","deriv":"m/44'/0'/0'","xfp":"B8688DF1","xpub":"{XPUB_84}","first":"1..."}}
            }}"#,
        );
        let parsed = parse(blob.as_bytes()).unwrap();
        assert_eq!(parsed.len(), 1);
        let p = &parsed[0];
        assert_eq!(p.network, bitcoin::Network::Bitcoin);
        assert!(p.original_descriptor.starts_with("pkh("));
        match &p.provenance {
            ImportProvenance::Coldcard(m) => {
                assert_eq!(m.bip_derivation, ColdcardBip::Bip44);
            }
            _ => panic!("provenance"),
        }
    }

    #[test]
    fn parse_bip49_mainnet_happy_path() {
        let blob = format!(
            r#"{{
                "chain":"BTC","xfp":"28645006","account":0,
                "bip49": {{"name":"p2wpkh-p2sh","deriv":"m/49'/0'/0'","xfp":"28645006","xpub":"{XPUB_49}","_pub":"ypub...","first":"3..."}}
            }}"#,
        );
        let parsed = parse(blob.as_bytes()).unwrap();
        assert_eq!(parsed.len(), 1);
        let p = &parsed[0];
        assert!(
            p.original_descriptor.starts_with("sh(wpkh("),
            "BIP-49 must emit sh(wpkh(...)); got: {}",
            p.original_descriptor
        );
        match &p.provenance {
            ImportProvenance::Coldcard(m) => {
                assert_eq!(m.bip_derivation, ColdcardBip::Bip49);
                assert_eq!(m.xfp, [0x28, 0x64, 0x50, 0x06]);
            }
            _ => panic!("provenance"),
        }
    }

    #[test]
    fn parse_bip86_taproot_mainnet_happy_path() {
        let blob = format!(
            r#"{{
                "chain":"BTC","xfp":"B8688DF1","account":0,
                "bip86": {{"name":"p2tr","deriv":"m/86'/0'/0'","xfp":"B8688DF1","xpub":"{XPUB_84}","first":"bc1p..."}}
            }}"#,
        );
        let parsed = parse(blob.as_bytes()).unwrap();
        assert_eq!(parsed.len(), 1);
        let p = &parsed[0];
        assert!(
            p.original_descriptor.starts_with("tr("),
            "BIP-86 must emit tr(...); got: {}",
            p.original_descriptor
        );
        match &p.provenance {
            ImportProvenance::Coldcard(m) => {
                assert_eq!(m.bip_derivation, ColdcardBip::Bip86);
            }
            _ => panic!("provenance"),
        }
    }

    #[test]
    fn parse_xtn_testnet_inferred_network() {
        let blob = format!(
            r#"{{
                "chain":"XTN","xfp":"704C7836","account":0,
                "bip84": {{"name":"p2wpkh","deriv":"m/84'/1'/0'","xfp":"704C7836","xpub":"{TPUB_84_TESTNET}","_pub":"vpub...","first":"tb1q..."}}
            }}"#,
        );
        let parsed = parse(blob.as_bytes()).unwrap();
        assert_eq!(parsed[0].network, bitcoin::Network::Testnet);
        match &parsed[0].provenance {
            ImportProvenance::Coldcard(m) => assert_eq!(m.chain, ColdcardChain::Xtn),
            _ => panic!("provenance"),
        }
    }

    #[test]
    fn parse_dominant_bip_selection_bip86_wins_over_bip84() {
        // Modern firmware emits ALL of bip44/bip49/bip84/bip86 in one blob.
        // Dominant order is bip86 > bip84 > bip49 > bip44.
        let blob = format!(
            r#"{{
                "chain":"BTC","xfp":"B8688DF1","account":0,
                "bip44": {{"name":"p2pkh","deriv":"m/44'/0'/0'","xfp":"B8688DF1","xpub":"{XPUB_84}","first":"1..."}},
                "bip49": {{"name":"p2wpkh-p2sh","deriv":"m/49'/0'/0'","xfp":"B8688DF1","xpub":"{XPUB_49}","_pub":"ypub...","first":"3..."}},
                "bip84": {{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"B8688DF1","xpub":"{XPUB_84}","_pub":"zpub...","first":"bc1q..."}},
                "bip86": {{"name":"p2tr","deriv":"m/86'/0'/0'","xfp":"B8688DF1","xpub":"{XPUB_84}","first":"bc1p..."}}
            }}"#,
        );
        let parsed = parse(blob.as_bytes()).unwrap();
        match &parsed[0].provenance {
            ImportProvenance::Coldcard(m) => assert_eq!(m.bip_derivation, ColdcardBip::Bip86),
            _ => panic!("provenance"),
        }
        assert!(parsed[0].original_descriptor.starts_with("tr("));
    }

    #[test]
    fn parse_dominant_bip_selection_bip84_wins_over_bip49_and_bip44() {
        // No bip86 present — bip84 dominates.
        let blob = format!(
            r#"{{
                "chain":"BTC","xfp":"B8688DF1","account":0,
                "bip44": {{"name":"p2pkh","deriv":"m/44'/0'/0'","xfp":"B8688DF1","xpub":"{XPUB_84}","first":"1..."}},
                "bip49": {{"name":"p2wpkh-p2sh","deriv":"m/49'/0'/0'","xfp":"B8688DF1","xpub":"{XPUB_49}","_pub":"ypub...","first":"3..."}},
                "bip84": {{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"B8688DF1","xpub":"{XPUB_84}","_pub":"zpub...","first":"bc1q..."}}
            }}"#,
        );
        let parsed = parse(blob.as_bytes()).unwrap();
        match &parsed[0].provenance {
            ImportProvenance::Coldcard(m) => assert_eq!(m.bip_derivation, ColdcardBip::Bip84),
            _ => panic!("provenance"),
        }
    }

    #[test]
    fn parse_legacy_top_level_xpub_only_infers_bip44_from_xpub_prefix() {
        // Mk1/Mk2 firmware: only `xpub` at top level (no bipNN sub-objects).
        let blob = format!(
            r#"{{"chain":"BTC","xfp":"B8688DF1","xpub":"{XPUB_84}"}}"#,
        );
        let parsed = parse(blob.as_bytes()).unwrap();
        assert_eq!(parsed.len(), 1);
        match &parsed[0].provenance {
            ImportProvenance::Coldcard(m) => {
                // Top-level xpub starts with "xpub" → BIP-44 by SLIP-132 inference.
                assert_eq!(m.bip_derivation, ColdcardBip::Bip44);
                assert_eq!(m.raw_account, 0);
            }
            _ => panic!("provenance"),
        }
        // Legacy fallback synthesizes deriv path m/44'/0'/0' for BTC + account 0.
        assert!(parsed[0].original_descriptor.contains("44'/0'/0'"));
    }

    #[test]
    fn parse_bip48_only_blob_refused_no_dominant_singlesig_block() {
        // bip48_* keys satisfy sniff (Q3 disjunction includes them) but the
        // single-sig parser must refuse — bip48_* are multisig hints; the
        // multisig case is `--format coldcard-multisig` (SPEC §11.3.1 rule 6).
        let blob = format!(
            r#"{{
                "chain":"BTC","xfp":"B8688DF1","account":0,
                "bip48_2": {{"name":"p2wsh","deriv":"m/48'/0'/0'/2'","xfp":"B8688DF1","xpub":"{XPUB_84}"}}
            }}"#,
        );
        let err = parse(blob.as_bytes()).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("coldcard") && msg.contains("no dominant derivation block"),
            "expected coldcard 'no dominant block' refusal; got: {msg}"
        );
    }

    #[test]
    fn parse_emits_notice_for_dropped_fields() {
        let blob = format!(
            r#"{{
                "chain":"BTC","xfp":"B8688DF1","account":0,
                "bip84": {{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"B8688DF1","xpub":"{XPUB_84}","_pub":"zpub...","first":"bc1q..."}},
                "extra_field":"this should be in dropped_fields",
                "another":"too"
            }}"#,
        );
        let (res, stderr) = parse_capturing_stderr(blob.as_bytes());
        let parsed = res.unwrap();
        assert!(
            stderr.contains("notice: import-wallet: coldcard: dropped envelope fields")
                && stderr.contains("extra_field")
                && stderr.contains("another"),
            "expected NOTICE listing both dropped fields; got: {stderr}"
        );
        match &parsed[0].provenance {
            ImportProvenance::Coldcard(m) => {
                assert!(m.dropped_fields.iter().any(|f| f == "extra_field"));
                assert!(m.dropped_fields.iter().any(|f| f == "another"));
            }
            _ => panic!("provenance"),
        }
    }

    #[test]
    fn parse_malformed_json_refused() {
        let err = parse(b"{not json").unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("coldcard") && msg.contains("invalid JSON"),
            "expected coldcard invalid-JSON error; got: {msg}"
        );
    }

    #[test]
    fn parse_missing_chain_refused() {
        let blob = format!(
            r#"{{"xfp":"B8688DF1","bip84":{{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"B8688DF1","xpub":"{XPUB_84}","first":"bc1q..."}}}}"#,
        );
        let err = parse(blob.as_bytes()).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("coldcard") && msg.contains("`chain`"),
            "expected coldcard missing-chain error; got: {msg}"
        );
    }

    #[test]
    fn parse_unrecognized_chain_value_refused() {
        let blob = format!(
            r#"{{"chain":"main","xfp":"B8688DF1","bip84":{{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"B8688DF1","xpub":"{XPUB_84}","first":"bc1q..."}}}}"#,
        );
        let err = parse(blob.as_bytes()).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("coldcard") && msg.contains("`chain`"),
            "expected coldcard chain-not-BTC-or-XTN error; got: {msg}"
        );
    }

    #[test]
    fn parse_bad_xfp_length_refused() {
        let blob = format!(
            r#"{{"chain":"BTC","xfp":"DEAD","bip84":{{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"DEAD","xpub":"{XPUB_84}","first":"bc1q..."}}}}"#,
        );
        let err = parse(blob.as_bytes()).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("coldcard") && msg.contains("`xfp`") && msg.contains("8 hex"),
            "expected coldcard xfp-length error; got: {msg}"
        );
    }

    #[test]
    fn parse_bip84_sub_missing_xpub_refused() {
        let blob = r#"{"chain":"BTC","xfp":"B8688DF1","bip84":{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"B8688DF1"}}"#;
        let err = parse(blob.as_bytes()).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("coldcard") && msg.contains("`bip84.xpub`"),
            "expected coldcard bip84.xpub missing error; got: {msg}"
        );
    }

    #[test]
    fn parse_lowercase_xfp_tolerated() {
        // Coldcard upstream emits uppercase but lowercase is tolerated.
        let blob = format!(
            r#"{{"chain":"BTC","xfp":"b8688df1","bip84":{{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"b8688df1","xpub":"{XPUB_84}","first":"bc1q..."}}}}"#,
        );
        let parsed = parse(blob.as_bytes()).unwrap();
        match &parsed[0].provenance {
            ImportProvenance::Coldcard(m) => assert_eq!(m.xfp, [0xB8, 0x68, 0x8D, 0xF1]),
            _ => panic!("provenance"),
        }
    }

    // ========================================================================
    // Fixture-driven cells (load tests/fixtures/wallet_import/coldcard-*.json)
    // ========================================================================

    fn load_fixture(name: &str) -> Vec<u8> {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/wallet_import")
            .join(name);
        std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()))
    }

    #[test]
    fn fixture_singlesig_bip84_mainnet_parses_clean() {
        let blob = load_fixture("coldcard-singlesig-bip84-mainnet.json");
        let parsed = parse(&blob).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].cosigners.len(), 1);
        assert_eq!(parsed[0].network, bitcoin::Network::Bitcoin);
        match &parsed[0].provenance {
            ImportProvenance::Coldcard(m) => assert_eq!(m.bip_derivation, ColdcardBip::Bip84),
            _ => panic!("provenance"),
        }
    }

    #[test]
    fn fixture_singlesig_bip49_mainnet_parses_clean() {
        let blob = load_fixture("coldcard-singlesig-bip49-mainnet.json");
        let parsed = parse(&blob).unwrap();
        assert_eq!(parsed.len(), 1);
        match &parsed[0].provenance {
            ImportProvenance::Coldcard(m) => assert_eq!(m.bip_derivation, ColdcardBip::Bip49),
            _ => panic!("provenance"),
        }
    }

    #[test]
    fn fixture_singlesig_bip44_mainnet_parses_clean() {
        let blob = load_fixture("coldcard-singlesig-bip44-mainnet.json");
        let parsed = parse(&blob).unwrap();
        assert_eq!(parsed.len(), 1);
        match &parsed[0].provenance {
            ImportProvenance::Coldcard(m) => assert_eq!(m.bip_derivation, ColdcardBip::Bip44),
            _ => panic!("provenance"),
        }
    }

    #[test]
    fn fixture_singlesig_bip84_xtn_testnet_parses_clean() {
        let blob = load_fixture("coldcard-singlesig-bip84-xtn-testnet.json");
        let parsed = parse(&blob).unwrap();
        assert_eq!(parsed[0].network, bitcoin::Network::Testnet);
        match &parsed[0].provenance {
            ImportProvenance::Coldcard(m) => {
                assert_eq!(m.bip_derivation, ColdcardBip::Bip84);
                assert_eq!(m.chain, ColdcardChain::Xtn);
            }
            _ => panic!("provenance"),
        }
    }

    #[test]
    fn fixture_singlesig_bip86_taproot_mainnet_parses_clean() {
        let blob = load_fixture("coldcard-singlesig-bip86-mainnet.json");
        let parsed = parse(&blob).unwrap();
        assert_eq!(parsed.len(), 1);
        match &parsed[0].provenance {
            ImportProvenance::Coldcard(m) => assert_eq!(m.bip_derivation, ColdcardBip::Bip86),
            _ => panic!("provenance"),
        }
        assert!(parsed[0].original_descriptor.starts_with("tr("));
    }
}
