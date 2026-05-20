//! Coldcard single-sig `wallet.json` parser (Phase P3).
//!
//! Per `design/SPEC_wallet_import_v0_28_0.md` §11.3. Coldcard's generic-
//! wallet-export `wallet.json` is the singlesig surface; the multisig
//! text-format export lands in §11.4 (`coldcard_multisig.rs`, Phase P4).
//!
//! ## On-disk shape (multiple firmware variants — see SPEC §11.3.1 table)
//!
//! ```json
//! {
//!   "chain": "BTC" | "XTN",
//!   "xfp": "<8-char uppercase hex master fingerprint>",
//!   "account": 0,
//!   "xpub": "<top-level account xpub — legacy Mk1/Mk2 firmware>",
//!   "bip44": { "name": "p2pkh",      "deriv": "m/44'/<coin>'/<acc>'", "xfp": "<parent>", "xpub": "...", "first": "..." },
//!   "bip49": { "name": "p2wpkh-p2sh","deriv": "m/49'/<coin>'/<acc>'", "xfp": "<parent>", "xpub": "...", "_pub": "ypub...", "first": "..." },
//!   "bip84": { "name": "p2wpkh",     "deriv": "m/84'/<coin>'/<acc>'", "xfp": "<parent>", "xpub": "...", "_pub": "zpub...", "first": "..." },
//!   "bip86": { "name": "p2tr",       "deriv": "m/86'/<coin>'/<acc>'", "xfp": "<parent>", "xpub": "...", "first": "..." },
//!   "bip48_1": { ... multisig hint — IGNORED by single-sig parser ... },
//!   "bip48_2": { ... multisig hint — IGNORED by single-sig parser ... }
//! }
//! ```
//!
//! ## Sniff signature (SPEC §11.3, Q3-lock relaxed per R0 I8)
//!
//! Top-level JSON object containing ALL of:
//! - `chain` ∈ {"BTC", "XTN"}
//! - `xfp` (string)
//! - At-least-one-of: `xpub`, `bip44`, `bip49`, `bip84`, `bip86`, `bip48_1`,
//!   `bip48_2`
//!
//! The disjunction in the third clause absorbs Coldcard firmware variance
//! (different firmware versions emit different combinations of per-BIP
//! derivation blocks). See SPEC §11.3.1 for the firmware-variance table.
//!
//! ## Parse contract (SPEC §11.3 + §11.3.1)
//!
//! Phase P3A is **skeleton + sniff only**; `parse` returns `unimplemented!()`.
//! Phase P3B implements the real parse:
//! 1. Extract `chain` → network (BTC → mainnet, XTN → testnet).
//! 2. Extract `xfp` (string) → master fingerprint (`[u8; 4]`).
//! 3. Pick dominant BIP block per SPEC §11.3.1 dominance order:
//!    BIP-86 > BIP-84 > BIP-49 > BIP-44, falling back to top-level `xpub` +
//!    SLIP-132 prefix inference for legacy Mk1/Mk2 firmware.
//! 4. Build a descriptor body from the selected block's `deriv`, parent
//!    `xfp`, and `xpub`; route through the same `parse_descriptor` pipeline
//!    as BSMS / Bitcoin Core.
//! 5. `bip48_1` / `bip48_2` are silently IGNORED — they are multisig-context
//!    hints; the authoritative multisig surface is Phase P4
//!    (`coldcard_multisig.rs`).

use super::{
    pipeline::concrete_keys_to_placeholders, validate_watch_only_resolved, ImportProvenance,
    ParsedImport, WalletFormatParser,
};
use crate::error::ToolkitError;
use crate::parse_descriptor;
use crate::synthesize::{xpub_to_65, ResolvedSlot};
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};
use serde_json::Value;
use std::io::Write;
use std::str::FromStr;

pub(crate) struct ColdcardParser;

/// Sniff-time marker set for the third clause of the SPEC §11.3 sniff
/// predicate: presence of ANY of these top-level keys (alongside `chain` +
/// `xfp`) classifies the blob as Coldcard.
///
/// Listed in alphabetical order to match the SPEC §11.3.1 firmware-variance
/// table reading order; ordering is not load-bearing for the sniff (the
/// predicate is a logical OR), only for human-readable diff-stability.
const COLDCARD_PER_BIP_MARKERS: &[&str] = &[
    "bip44", "bip48_1", "bip48_2", "bip49", "bip84", "bip86", "xpub",
];

/// SPEC §11.3 — `chain` field domain. BTC → mainnet, XTN → testnet
/// (signet/regtest absent from Coldcard's schema; the export side maps
/// signet → "XTN" by convention but the import side accepts only the two
/// canonical values per SPEC).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColdcardChain {
    Btc,
    Xtn,
}

impl ColdcardChain {
    /// SPEC §11.3 — map the `chain` field value to a bitcoin network.
    /// XTN → testnet (Coldcard's schema does not distinguish testnet /
    /// signet / regtest; testnet is the canonical mapping per SPEC).
    fn to_network(self) -> bitcoin::Network {
        match self {
            ColdcardChain::Btc => bitcoin::Network::Bitcoin,
            ColdcardChain::Xtn => bitcoin::Network::Testnet,
        }
    }
}

/// SPEC §11.3 — dominant-BIP selection result. P3B's parse impl picks ONE
/// variant per `ColdcardSourceMetadata` per the §11.3.1 dominance order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColdcardBip {
    Bip44,
    Bip49,
    Bip84,
    Bip86,
}

impl ColdcardBip {
    /// SPEC §11.3 — descriptor wrapper around the bracketed key for each
    /// BIP-derivation variant. Output template uses `{KEY}` as a substitution
    /// marker for `[xfp/path]xpub/<0;1>/*`.
    fn descriptor_template(self) -> &'static str {
        match self {
            ColdcardBip::Bip44 => "pkh({KEY})",
            ColdcardBip::Bip49 => "sh(wpkh({KEY}))",
            ColdcardBip::Bip84 => "wpkh({KEY})",
            ColdcardBip::Bip86 => "tr({KEY})",
        }
    }

    /// SPEC §11.3.1 — name of the corresponding top-level JSON field that
    /// contains this BIP's derivation block.
    fn json_field_name(self) -> &'static str {
        match self {
            ColdcardBip::Bip44 => "bip44",
            ColdcardBip::Bip49 => "bip49",
            ColdcardBip::Bip84 => "bip84",
            ColdcardBip::Bip86 => "bip86",
        }
    }
}

/// SPEC §11.3 — Coldcard single-sig parser provenance.
///
/// Carried inside `ImportProvenance::Coldcard(...)` and surfaced via the
/// `--json` envelope's `source_metadata` field (Phase P3C wire-up).
#[derive(Debug, Clone)]
#[allow(dead_code)] // P3B: fields populated by parse-impl; P3C surfaces them
                    // via the --json envelope's `source_metadata` field +
                    // `ImportProvenance::Coldcard` variant.
pub(crate) struct ColdcardSourceMetadata {
    /// SPEC §11.3 — `chain` field value (BTC / XTN).
    pub chain: ColdcardChain,
    /// SPEC §11.3 — master fingerprint extracted from top-level `xfp`.
    pub xfp: [u8; 4],
    /// SPEC §11.3.1 — which BIP block the parser selected via dominance order.
    pub bip_derivation: ColdcardBip,
    /// SPEC §11.3 — `account` field value (defaults to 0 if absent per
    /// Coldcard schema).
    pub raw_account: u32,
    /// SPEC §11.3 — per-entry fields that appeared in the source blob but
    /// are not preserved in the toolkit's parsed bundle. Populated with the
    /// selected BIP block's `name` + `first` + `_pub` (SLIP-132 alternate)
    /// when present, plus competing-BIP block names that were superseded by
    /// the dominance order.
    pub dropped_fields: Vec<String>,
}

impl WalletFormatParser for ColdcardParser {
    /// SPEC §11.3 sniff predicate (Q3-lock relaxed per R0 I8).
    ///
    /// Returns `true` if the blob is a JSON object containing all of:
    /// - `chain` ∈ {"BTC", "XTN"}
    /// - `xfp` (any string value)
    /// - At least one of: `xpub`, `bip44`, `bip49`, `bip84`, `bip86`,
    ///   `bip48_1`, `bip48_2`
    ///
    /// The third clause is a disjunction — it absorbs firmware-variant
    /// shapes (Mk1/Mk2 emit only `xpub`; Mk3 adds `bip44`/`bip49`/`bip84`
    /// blocks; Mk4 adds `bip86`; Q adds `bip48_1`/`bip48_2`). See SPEC
    /// §11.3.1 firmware-variance table.
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

        // Clause 1: `chain` ∈ {BTC, XTN}.
        let chain_ok = obj
            .get("chain")
            .and_then(|v| v.as_str())
            .map(|s| s == "BTC" || s == "XTN")
            .unwrap_or(false);
        if !chain_ok {
            return false;
        }

        // Clause 2: `xfp` is present as a string.
        if !obj.get("xfp").map(|v| v.is_string()).unwrap_or(false) {
            return false;
        }

        // Clause 3: at-least-one-of the per-BIP markers.
        COLDCARD_PER_BIP_MARKERS
            .iter()
            .any(|m| obj.contains_key(*m))
    }

    /// SPEC §11.3 + §11.3.1 — parse a Coldcard single-sig wallet.json blob.
    ///
    /// Steps:
    /// 1. JSON-parse + extract `chain` → `ColdcardChain` → network.
    /// 2. Extract top-level `xfp` → master fingerprint (`[u8; 4]`).
    /// 3. Extract `account` (default 0).
    /// 4. Select dominant BIP block per SPEC §11.3.1:
    ///    `bip86 > bip84 > bip49 > bip44`, falling back to top-level `xpub`
    ///    (with SLIP-132 prefix inference) for Mk1/Mk2 legacy firmware.
    ///    Note `bip48_1` / `bip48_2` are silently IGNORED — multisig hints;
    ///    the authoritative multisig surface is Phase P4.
    /// 5. From the selected block: extract `deriv`, parent `xfp`, `xpub`.
    /// 6. Build a `[xfp/deriv]xpub/<0;1>/*` bracket-form key + wrap in
    ///    `pkh / sh(wpkh) / wpkh / tr` per BIP.
    /// 7. Route through `concrete_keys_to_placeholders` +
    ///    `parse_descriptor::parse_descriptor` pipeline (same as
    ///    BSMS / Bitcoin Core).
    /// 8. Emit ONE `ParsedImport` (single-sig is single-cosigner; bundle
    ///    output is length-1 per blob).
    fn parse(blob: &[u8], stderr: &mut dyn Write) -> Result<Vec<ParsedImport>, ToolkitError> {
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

        // Step 1: chain → network.
        let chain_str = obj
            .get("chain")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(
                    "import-wallet: coldcard: parse error: missing or non-string top-level `chain` field"
                        .to_string(),
                )
            })?;
        let chain = match chain_str {
            "BTC" => ColdcardChain::Btc,
            "XTN" => ColdcardChain::Xtn,
            other => {
                return Err(ToolkitError::ImportWalletParse(format!(
                    "import-wallet: coldcard: parse error: `chain` must be `BTC` or `XTN`, got {other:?}"
                )));
            }
        };
        let network = chain.to_network();

        // Step 2: xfp → master fingerprint.
        let xfp_str = obj.get("xfp").and_then(|v| v.as_str()).ok_or_else(|| {
            ToolkitError::ImportWalletParse(
                "import-wallet: coldcard: parse error: missing or non-string top-level `xfp` field"
                    .to_string(),
            )
        })?;
        let xfp = parse_xfp_hex(xfp_str, "xfp")?;

        // Step 3: account.
        let raw_account: u32 = match obj.get("account") {
            None => 0,
            Some(Value::Null) => 0,
            Some(Value::Number(n)) => n.as_u64().and_then(|v| u32::try_from(v).ok()).ok_or_else(|| {
                ToolkitError::ImportWalletParse(
                    "import-wallet: coldcard: parse error: `account` must be a non-negative integer ≤ u32::MAX"
                        .to_string(),
                )
            })?,
            Some(other) => {
                return Err(ToolkitError::ImportWalletParse(format!(
                    "import-wallet: coldcard: parse error: `account` must be a number, got {}",
                    kind_of(other)
                )));
            }
        };

        // Step 4: dominant-BIP selection.
        let (bip_derivation, selected_block) = select_dominant_bip(obj)?;

        // Step 5: extract per-block fields.
        let block_obj = selected_block.as_object().ok_or_else(|| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: coldcard: parse error: `{}` block is not an object",
                bip_derivation.json_field_name()
            ))
        })?;
        let deriv = block_obj
            .get("deriv")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(format!(
                    "import-wallet: coldcard: parse error: `{}.deriv` is missing or not a string",
                    bip_derivation.json_field_name()
                ))
            })?;
        let block_xfp_str = block_obj
            .get("xfp")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(format!(
                    "import-wallet: coldcard: parse error: `{}.xfp` is missing or not a string",
                    bip_derivation.json_field_name()
                ))
            })?;
        // The per-block `xfp` is the PARENT fingerprint of the account xpub
        // (BIP-32 serialization bytes 5..9) — distinct from the top-level
        // master `xfp`. Coldcard's wallet.json includes both; we parse it
        // for shape-validation (8-char uppercase hex) but the bracketed key
        // form below uses the TOP-LEVEL master `xfp` per BIP-380 origin-
        // annotation semantics.
        let _block_xfp = parse_xfp_hex(block_xfp_str, &format!("{}.xfp", bip_derivation.json_field_name()))?;

        let xpub_str = block_obj
            .get("xpub")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(format!(
                    "import-wallet: coldcard: parse error: `{}.xpub` is missing or not a string",
                    bip_derivation.json_field_name()
                ))
            })?;

        // Normalize any SLIP-132 prefix to the neutral xpub/tpub form;
        // mirrors `bitcoin_core::build_slot_fields`.
        let (neutral_xpub_str, _variant) = crate::slip0132::normalize_xpub_prefix(xpub_str)
            .map_err(|e| {
                ToolkitError::ImportWalletParse(format!(
                    "import-wallet: coldcard: parse error: `{}.xpub` slip132 normalize: {}",
                    bip_derivation.json_field_name(),
                    e
                ))
            })?;
        let xpub = Xpub::from_str(&neutral_xpub_str).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: coldcard: parse error: `{}.xpub` parse: {e}",
                bip_derivation.json_field_name()
            ))
        })?;

        // Step 6: build descriptor body.
        // - Strip leading `m/` from `deriv` to produce the bracket-inner
        //   form `[xfp/<path>]xpub/<0;1>/*`.
        let path_inner = deriv.strip_prefix("m/").ok_or_else(|| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: coldcard: parse error: `{}.deriv` does not start with `m/`: {deriv:?}",
                bip_derivation.json_field_name()
            ))
        })?;
        let xfp_hex = bytes_to_hex_lower(&xfp);
        let bracket_key = format!(
            "[{xfp_hex}/{path_inner}]{neutral_xpub_str}/<0;1>/*"
        );
        let descriptor_body =
            bip_derivation.descriptor_template().replace("{KEY}", &bracket_key);

        // Build DerivationPath from deriv (relative form) for the
        // ResolvedSlot.path field.
        let path = DerivationPath::from_str(deriv).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: coldcard: parse error: `{}.deriv` parse: {e}",
                bip_derivation.json_field_name()
            ))
        })?;
        let fp = Fingerprint::from(xfp);

        // Step 7: route through concrete_keys_to_placeholders +
        // parse_descriptor pipeline.
        let (placeholder_form, parsed_keys, parsed_fingerprints) =
            concrete_keys_to_placeholders(&descriptor_body).map_err(|e| {
                // Re-tag the BSMS error template prefix as coldcard for the
                // user-facing message.
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
                "import-wallet: coldcard: parse error: descriptor parse: {}",
                e.message()
            ))
        })?;

        // Step 8: build ResolvedSlot + ParsedImport.
        let path_raw = format!("[{xfp_hex}/{path_inner}]");
        debug_assert_eq!(parsed_keys.len(), 1, "single-sig coldcard always has 1 key");
        debug_assert_eq!(xpub_to_65(&xpub), parsed_keys[0].payload);

        let cosigner = ResolvedSlot {
            xpub,
            fingerprint: fp,
            path,
            path_raw,
            entropy: None,
            master_xpub: None,
            _entropy_pin: None,
        };
        let cosigners = vec![cosigner];
        validate_watch_only_resolved(&cosigners)?;

        // Step 8b: build dropped_fields telemetry. Includes the block's
        // `name` + `first` + `_pub` (SLIP-132 alternate, when present)
        // because those source-blob fields do not survive into the
        // toolkit's parsed bundle. Also includes any competing-BIP block
        // names that were superseded by the dominance order.
        let mut dropped_fields: Vec<String> = Vec::new();
        for f in ["name", "first", "_pub"] {
            if block_obj.contains_key(f) {
                dropped_fields.push(format!("{}.{}", bip_derivation.json_field_name(), f));
            }
        }
        for competing in ["bip44", "bip49", "bip84", "bip86", "bip48_1", "bip48_2", "xpub"] {
            if competing == bip_derivation.json_field_name() {
                continue;
            }
            if obj.contains_key(competing) {
                dropped_fields.push(competing.to_string());
            }
        }
        if !dropped_fields.is_empty() {
            writeln!(
                stderr,
                "notice: import-wallet: coldcard: dominant-BIP {} selected; dropped fields {}: not preserved in bundle output (key-state only)",
                bip_derivation.json_field_name(),
                dropped_fields.join(", ")
            )
            .map_err(ToolkitError::Io)?;
        }

        // SPEC §11.3 P3C wire-up: the `ImportProvenance::Coldcard` variant
        // does not yet exist on the enum (lands at P3C). At P3B, we
        // construct the metadata struct for use post-P3C; for now, route
        // through the BSMS(None) provenance placeholder so the type
        // compiles. P3C replaces this with `ImportProvenance::Coldcard(meta)`.
        let _provenance_pending_p3c = ColdcardSourceMetadata {
            chain,
            xfp,
            bip_derivation,
            raw_account,
            dropped_fields,
        };
        let provenance = ImportProvenance::Bsms(None); // P3C-replace

        Ok(vec![ParsedImport {
            descriptor,
            original_descriptor: descriptor_body.clone(),
            cosigners,
            network,
            threshold: None,
            provenance,
        }])
    }
}

/// Strict 8-char uppercase hex → `[u8; 4]`. Tagged with `field_name` for
/// user-facing error messages.
fn parse_xfp_hex(s: &str, field_name: &str) -> Result<[u8; 4], ToolkitError> {
    if s.len() != 8 {
        return Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: coldcard: parse error: `{field_name}` must be 8-char hex, got {} chars",
            s.len()
        )));
    }
    let mut bytes = [0u8; 4];
    for i in 0..4 {
        bytes[i] = u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: coldcard: parse error: `{field_name}` hex decode: {e}"
            ))
        })?;
    }
    Ok(bytes)
}

fn bytes_to_hex_lower(bytes: &[u8; 4]) -> String {
    format!("{:02x}{:02x}{:02x}{:02x}", bytes[0], bytes[1], bytes[2], bytes[3])
}

/// SPEC §11.3.1 dominant-BIP selection. Returns the selected BIP-variant
/// AND a borrowed reference to its top-level JSON sub-block. Falls back to
/// the legacy Mk1/Mk2 top-level `xpub` via SLIP-132 prefix inference when
/// no `bip*` block is present.
fn select_dominant_bip(
    obj: &serde_json::Map<String, Value>,
) -> Result<(ColdcardBip, &Value), ToolkitError> {
    // §11.3.1 step 1-4: dominance order BIP-86 > BIP-84 > BIP-49 > BIP-44.
    for (variant, key) in [
        (ColdcardBip::Bip86, "bip86"),
        (ColdcardBip::Bip84, "bip84"),
        (ColdcardBip::Bip49, "bip49"),
        (ColdcardBip::Bip44, "bip44"),
    ] {
        if let Some(block) = obj.get(key) {
            return Ok((variant, block));
        }
    }

    // §11.3.1 step 5: legacy Mk1/Mk2 fallback. Top-level `xpub` only;
    // infer BIP from SLIP-132 prefix.
    if obj.contains_key("xpub") {
        return Err(ToolkitError::ImportWalletParse(
            "import-wallet: coldcard: parse error: legacy Mk1/Mk2 firmware variant \
             (top-level `xpub` only, no `bip*` sub-blocks) is recognized but not yet \
             supported by P3B's parse impl — the SLIP-132 prefix inference path \
             is a FOLLOWUP item. Use a Coldcard `Mk3+` firmware export that emits \
             explicit `bip44`/`bip49`/`bip84`/`bip86` sub-blocks."
                .to_string(),
        ));
    }

    // §11.3.1 step 6 + Q3 disjunction: bip48_* only (no singlesig blocks
    // alongside) → this is a multisig-context export; the singlesig parser
    // refuses with a pointer to the multisig surface (Phase P4).
    if obj.contains_key("bip48_1") || obj.contains_key("bip48_2") {
        return Err(ToolkitError::ImportWalletParse(
            "import-wallet: coldcard: parse error: blob contains only `bip48_*` blocks \
             (multisig-context hint); use `--format coldcard-multisig` against the \
             corresponding multisig text export file instead"
                .to_string(),
        ));
    }

    Err(ToolkitError::ImportWalletParse(
        "import-wallet: coldcard: parse error: no recognized BIP-derivation block \
         (`bip44`/`bip49`/`bip84`/`bip86`) or legacy top-level `xpub` found"
            .to_string(),
    ))
}

/// Compact JSON type label used by parse-error templates.
fn kind_of(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

/// Strip leading ASCII whitespace before the JSON parse for sniff
/// robustness. Mirrors `bitcoin_core::trim_leading_ws`.
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

    // ---------------------------------------------------------------------
    // Sniff predicate — SPEC §11.3 clause coverage
    // ---------------------------------------------------------------------

    /// SPEC §11.3 happy path: BTC + xfp + bip84 → sniff TRUE.
    #[test]
    fn sniff_true_on_mk3_bip84_btc() {
        let blob = br#"{"chain":"BTC","xfp":"5436D724","account":0,"bip84":{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"127EA0E6","xpub":"xpub...","first":"bc1q..."}}"#;
        assert!(ColdcardParser::sniff(blob), "BTC + xfp + bip84 must sniff true");
    }

    /// SPEC §11.3 happy path: XTN + xfp + bip49 → sniff TRUE.
    #[test]
    fn sniff_true_on_mk3_bip49_xtn() {
        let blob = br#"{"chain":"XTN","xfp":"5436D724","account":0,"bip49":{"name":"p2wpkh-p2sh","deriv":"m/49'/1'/0'","xfp":"CF1D3830","xpub":"tpub...","first":"2N..."}}"#;
        assert!(ColdcardParser::sniff(blob), "XTN + xfp + bip49 must sniff true");
    }

    /// SPEC §11.3 happy path: BTC + xfp + bip44 → sniff TRUE.
    #[test]
    fn sniff_true_on_mk3_bip44_btc() {
        let blob = br#"{"chain":"BTC","xfp":"5436D724","account":0,"bip44":{"name":"p2pkh","deriv":"m/44'/0'/0'","xfp":"ABCDEF01","xpub":"xpub..."}}"#;
        assert!(ColdcardParser::sniff(blob), "BTC + xfp + bip44 must sniff true");
    }

    /// SPEC §11.3.1 Mk4-era variance: BTC + xfp + bip86 (taproot) → sniff TRUE.
    #[test]
    fn sniff_true_on_mk4_bip86_btc() {
        let blob = br#"{"chain":"BTC","xfp":"5436D724","account":0,"bip86":{"name":"p2tr","deriv":"m/86'/0'/0'","xfp":"ABCDEF01","xpub":"xpub..."}}"#;
        assert!(ColdcardParser::sniff(blob), "BTC + xfp + bip86 must sniff true");
    }

    /// SPEC §11.3.1 Q-era variance: BTC + xfp + bip48_2 (multisig hint, no
    /// singlesig BIP block) → sniff TRUE per Q3-lock disjunction. The
    /// parser is still classifies the blob as Coldcard at sniff time;
    /// Phase P3B's dominant-BIP selection chooses among `bip44/49/84/86`
    /// (or top-level `xpub`) — falling back to a parse error if none of
    /// those is present alongside `bip48_*`.
    #[test]
    fn sniff_true_on_q_bip48_only() {
        let blob = br#"{"chain":"BTC","xfp":"5436D724","account":0,"bip48_2":{"name":"p2wsh","deriv":"m/48'/0'/0'/2'","xpub":"xpub..."}}"#;
        assert!(
            ColdcardParser::sniff(blob),
            "Q-era bip48_* only must sniff true per SPEC §11.3 Q3-lock disjunction"
        );
    }

    /// SPEC §11.3.1 Mk1/Mk2 legacy: BTC + xfp + top-level `xpub` (no bip*
    /// blocks at all) → sniff TRUE per Q3-lock disjunction (third clause
    /// absorbs legacy firmware variance).
    #[test]
    fn sniff_true_on_mk1_legacy_top_level_xpub() {
        let blob = br#"{"chain":"BTC","xfp":"5436D724","account":0,"xpub":"xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX"}"#;
        assert!(
            ColdcardParser::sniff(blob),
            "Mk1/Mk2 legacy top-level xpub-only must sniff true per SPEC §11.3 Q3-lock disjunction"
        );
    }

    // ---------------------------------------------------------------------
    // Sniff predicate — refusal cases
    // ---------------------------------------------------------------------

    /// SPEC §11.3 clause 1 violation: `chain` value outside {BTC, XTN} →
    /// sniff FALSE. Defends against Specter/Sparrow blobs that happen to
    /// carry a `chain` field with `"main"` / `"test"` values.
    #[test]
    fn sniff_false_on_chain_value_main_specter_style() {
        let blob = br#"{"chain":"main","xfp":"5436D724","bip84":{"name":"p2wpkh","xpub":"xpub..."}}"#;
        assert!(
            !ColdcardParser::sniff(blob),
            "chain=`main` (Specter convention, not Coldcard) must sniff false"
        );
    }

    /// SPEC §11.3 clause 1 violation: `chain` absent → sniff FALSE.
    #[test]
    fn sniff_false_on_missing_chain() {
        let blob = br#"{"xfp":"5436D724","bip84":{"name":"p2wpkh","xpub":"xpub..."}}"#;
        assert!(!ColdcardParser::sniff(blob), "missing chain must sniff false");
    }

    /// SPEC §11.3 clause 2 violation: `xfp` absent → sniff FALSE.
    #[test]
    fn sniff_false_on_missing_xfp() {
        let blob = br#"{"chain":"BTC","bip84":{"name":"p2wpkh","xpub":"xpub..."}}"#;
        assert!(!ColdcardParser::sniff(blob), "missing xfp must sniff false");
    }

    /// SPEC §11.3 clause 2 violation: `xfp` present but not a string
    /// (e.g. an integer) → sniff FALSE.
    #[test]
    fn sniff_false_on_xfp_non_string() {
        let blob = br#"{"chain":"BTC","xfp":12345,"bip84":{"name":"p2wpkh","xpub":"xpub..."}}"#;
        assert!(!ColdcardParser::sniff(blob), "xfp non-string must sniff false");
    }

    /// SPEC §11.3 clause 3 violation: chain + xfp present but no per-BIP
    /// markers → sniff FALSE.
    #[test]
    fn sniff_false_on_chain_and_xfp_alone() {
        let blob = br#"{"chain":"BTC","xfp":"5436D724","account":0}"#;
        assert!(
            !ColdcardParser::sniff(blob),
            "chain + xfp without per-BIP markers must sniff false (clause 3)"
        );
    }

    /// Robustness: invalid JSON → sniff FALSE.
    #[test]
    fn sniff_false_on_invalid_json() {
        let blob = b"{not json}";
        assert!(!ColdcardParser::sniff(blob), "invalid JSON must sniff false");
    }

    /// Robustness: BSMS text blob → sniff FALSE (BSMS leads with `BSMS 1.0`,
    /// not `{`).
    #[test]
    fn sniff_false_on_bsms_text_blob() {
        let blob = b"BSMS 1.0\nwpkh(xpub...)\n";
        assert!(!ColdcardParser::sniff(blob), "BSMS text blob must sniff false");
    }

    /// Robustness: bare JSON array (Bitcoin Core's bare-array shape) →
    /// sniff FALSE.
    #[test]
    fn sniff_false_on_bare_array() {
        let blob = br#"[{"desc":"wpkh(xpub...)"}]"#;
        assert!(!ColdcardParser::sniff(blob), "bare array must sniff false");
    }

    /// Robustness: empty blob → sniff FALSE.
    #[test]
    fn sniff_false_on_empty() {
        assert!(!ColdcardParser::sniff(b""), "empty blob must sniff false");
    }

    /// Robustness: leading whitespace before `{` → sniff TRUE (per
    /// `trim_leading_ws` discipline mirroring `bitcoin_core.rs`).
    #[test]
    fn sniff_true_on_leading_whitespace() {
        let blob = b"  \n\t{\"chain\":\"BTC\",\"xfp\":\"5436D724\",\"bip84\":{\"xpub\":\"xpub...\"}}";
        assert!(
            ColdcardParser::sniff(blob),
            "leading whitespace must not block sniff TRUE"
        );
    }

    // ---------------------------------------------------------------------
    // Format-disambiguation — sniff must NOT claim other vendors' blobs
    // ---------------------------------------------------------------------

    /// Bitcoin Core's `listdescriptors` JSON has `descriptors` array, NO
    /// `chain` key (and NO `xfp`) → Coldcard sniff FALSE.
    #[test]
    fn sniff_false_on_bitcoin_core_listdescriptors() {
        let blob = br#"{"wallet_name":"x","descriptors":[{"desc":"wpkh(xpub...)#00000000"}]}"#;
        assert!(
            !ColdcardParser::sniff(blob),
            "Bitcoin Core listdescriptors blob must sniff false on Coldcard"
        );
    }

    /// Specter's wallet JSON carries `blockheight`, `devices`, `descriptor`,
    /// `label` keys but not the Coldcard's `xfp` + `chain ∈ {BTC, XTN}`
    /// combination (Specter uses lowercase `"main"`/`"test"` for chain).
    #[test]
    fn sniff_false_on_specter_blob() {
        let blob = br#"{"chain":"main","label":"daily","blockheight":700000,"devices":["unknown"],"descriptor":"wpkh(xpub...)"}"#;
        assert!(
            !ColdcardParser::sniff(blob),
            "Specter blob (chain=main) must sniff false on Coldcard"
        );
    }

    /// Electrum wallet JSON has `seed_version` + `wallet_type`, no `chain`
    /// — Coldcard sniff FALSE.
    #[test]
    fn sniff_false_on_electrum_blob() {
        let blob = br#"{"seed_version":42,"wallet_type":"standard","keystore":{"xpub":"xpub..."}}"#;
        assert!(
            !ColdcardParser::sniff(blob),
            "Electrum blob must sniff false on Coldcard"
        );
    }

    /// Jade's `multisig_file`-shape JSON wrapper does not carry a top-level
    /// `chain` field — Coldcard sniff FALSE.
    #[test]
    fn sniff_false_on_jade_multisig_file_blob() {
        let blob = br#"{"multisig_file":"Name: foo\nPolicy: 2-of-3\n..."}"#;
        assert!(
            !ColdcardParser::sniff(blob),
            "Jade multisig_file blob must sniff false on Coldcard"
        );
    }

    // ---------------------------------------------------------------------
    // P3A skeleton invariants
    // ---------------------------------------------------------------------

    /// Provenance type-level invariants — ensure ColdcardSourceMetadata
    /// fields are stable across P3B → P3C. Constructed inline (no Default
    /// impl) so any field-shape drift surfaces here at compile time.
    #[test]
    fn provenance_struct_is_constructible_p3b_shape_lock() {
        let _meta = ColdcardSourceMetadata {
            chain: ColdcardChain::Btc,
            xfp: [0x54, 0x36, 0xD7, 0x24],
            bip_derivation: ColdcardBip::Bip84,
            raw_account: 0,
            dropped_fields: Vec::new(),
        };
    }

    // ---------------------------------------------------------------------
    // Parse impl — happy path per BIP variant (P3B)
    // ---------------------------------------------------------------------

    /// BIP-84 BTC mainnet: real Coldcard export-side fixture (lifted from
    /// `tests/export_wallet/coldcard_generic_bip84_mainnet.json`); ensures
    /// the parse pipeline accepts the toolkit's own emit shape.
    #[test]
    fn parse_bip84_btc_mainnet_happy_path() {
        let blob = br#"{
  "chain": "BTC",
  "xfp": "5436D724",
  "account": 0,
  "bip84": {
    "name": "p2wpkh",
    "deriv": "m/84'/0'/0'",
    "xfp": "127EA0E6",
    "xpub": "xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9",
    "_pub": "zpub6qTBTNftBzVTjgVcSUw7vW5N1KQbV93Jnrw314RHGkCkSx4vk6nEWH1MJfReXi2WThvuDRiRpyT7cDoakEcZMQ1iZPgfJgQrcVMR4aJWh6S",
    "first": "bc1qzmtrqsfuaf6l6kkcsseumq26ukaphfj9skkug6"
  }
}
"#;
        let mut stderr = Vec::new();
        let parsed = ColdcardParser::parse(blob, &mut stderr).expect("BIP-84 BTC must parse");
        assert_eq!(parsed.len(), 1, "single-sig must emit exactly one ParsedImport");
        let p = &parsed[0];
        assert_eq!(p.network, bitcoin::Network::Bitcoin, "BTC → mainnet");
        assert_eq!(p.cosigners.len(), 1, "single-sig → 1 cosigner");
        assert_eq!(p.threshold, None, "single-sig → no threshold");
        // Master fingerprint must come from top-level `xfp` (5436D724).
        assert_eq!(
            p.cosigners[0].fingerprint.to_string().to_lowercase(),
            "5436d724"
        );
        // Path must be the BIP-84 mainnet account path.
        assert_eq!(p.cosigners[0].path.to_string(), "84'/0'/0'");
        // Descriptor body must wrap the xpub in `wpkh(...)`.
        assert!(
            p.original_descriptor.starts_with("wpkh("),
            "BIP-84 → wpkh wrapper; got: {}",
            p.original_descriptor
        );
        // Multipath suffix preserved.
        assert!(
            p.original_descriptor.contains("/<0;1>/*"),
            "multipath suffix preserved; got: {}",
            p.original_descriptor
        );
        // Stderr notice mentions dropped fields.
        let stderr_str = String::from_utf8(stderr).unwrap();
        assert!(
            stderr_str.contains("dominant-BIP bip84"),
            "stderr must mention dominant-BIP: {stderr_str}"
        );
    }

    /// BIP-49 XTN testnet: tpub via SLIP-132 upub normalize. Lifted from
    /// `tests/export_wallet/coldcard_generic_bip49_testnet.json`.
    #[test]
    fn parse_bip49_xtn_testnet_happy_path() {
        let blob = br#"{
  "chain": "XTN",
  "xfp": "5436D724",
  "account": 0,
  "bip49": {
    "name": "p2wpkh-p2sh",
    "deriv": "m/49'/1'/0'",
    "xfp": "CF1D3830",
    "xpub": "tpubDDYhB7EGtNkJdeaPTacttc9jZ6aq7NWHiYy21ACcFx8g2zs9HNpQDondF7HQfemghZSEimBPHPRfs93UehvbFHZyHgWDBrY4KSCC183DAFw",
    "_pub": "upub5EgGjsPqj4AQoxpBWKGw4RXri53qJ82jRap4JRTon2F98QkkTBUdDB9NwTKs9RrggjYurksqNJXyX6iB6kqJy94sd4fdkbRMvJQr5MbJnEP",
    "first": "2NC5mrnP6XgKDASpbDjRhovygtF5jEpteiU"
  }
}
"#;
        let mut stderr = Vec::new();
        let parsed = ColdcardParser::parse(blob, &mut stderr).expect("BIP-49 XTN must parse");
        let p = &parsed[0];
        assert_eq!(p.network, bitcoin::Network::Testnet, "XTN → testnet");
        assert_eq!(p.cosigners[0].path.to_string(), "49'/1'/0'");
        assert!(
            p.original_descriptor.starts_with("sh(wpkh("),
            "BIP-49 → sh(wpkh(...)); got: {}",
            p.original_descriptor
        );
    }

    /// BIP-44 BTC mainnet: pkh wrapper (legacy P2PKH).
    #[test]
    fn parse_bip44_btc_mainnet_happy_path() {
        let blob = br#"{
  "chain": "BTC",
  "xfp": "5436D724",
  "account": 0,
  "bip44": {
    "name": "p2pkh",
    "deriv": "m/44'/0'/0'",
    "xfp": "127EA0E6",
    "xpub": "xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9",
    "first": "1Foo..."
  }
}
"#;
        let mut stderr = Vec::new();
        let parsed = ColdcardParser::parse(blob, &mut stderr).expect("BIP-44 BTC must parse");
        let p = &parsed[0];
        assert_eq!(p.network, bitcoin::Network::Bitcoin);
        assert_eq!(p.cosigners[0].path.to_string(), "44'/0'/0'");
        assert!(
            p.original_descriptor.starts_with("pkh("),
            "BIP-44 → pkh wrapper; got: {}",
            p.original_descriptor
        );
    }

    /// BIP-86 BTC mainnet taproot: `tr()` wrapper. Coldcard Mk4-era export
    /// (firmware-variance table at SPEC §11.3.1).
    #[test]
    fn parse_bip86_btc_mainnet_happy_path() {
        let blob = br#"{
  "chain": "BTC",
  "xfp": "5436D724",
  "account": 0,
  "bip86": {
    "name": "p2tr",
    "deriv": "m/86'/0'/0'",
    "xfp": "127EA0E6",
    "xpub": "xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9",
    "first": "bc1p..."
  }
}
"#;
        let mut stderr = Vec::new();
        let parsed = ColdcardParser::parse(blob, &mut stderr).expect("BIP-86 BTC must parse");
        let p = &parsed[0];
        assert_eq!(p.network, bitcoin::Network::Bitcoin);
        assert_eq!(p.cosigners[0].path.to_string(), "86'/0'/0'");
        assert!(
            p.original_descriptor.starts_with("tr("),
            "BIP-86 → tr wrapper; got: {}",
            p.original_descriptor
        );
    }

    // ---------------------------------------------------------------------
    // Parse impl — dominance order (SPEC §11.3.1)
    // ---------------------------------------------------------------------

    /// When bip84 + bip49 + bip44 are all present, dominance picks bip84
    /// (§11.3.1: bip86 > bip84 > bip49 > bip44).
    #[test]
    fn parse_dominance_picks_bip84_over_bip49_and_bip44() {
        let blob = br#"{
  "chain": "BTC",
  "xfp": "5436D724",
  "account": 0,
  "bip44": {"name":"p2pkh","deriv":"m/44'/0'/0'","xfp":"127EA0E6","xpub":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9","first":"1Foo"},
  "bip49": {"name":"p2wpkh-p2sh","deriv":"m/49'/0'/0'","xfp":"127EA0E6","xpub":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9","first":"3Foo"},
  "bip84": {"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"127EA0E6","xpub":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9","first":"bc1qFoo"}
}
"#;
        let mut stderr = Vec::new();
        let parsed = ColdcardParser::parse(blob, &mut stderr).unwrap();
        assert_eq!(parsed[0].cosigners[0].path.to_string(), "84'/0'/0'");
        let stderr_str = String::from_utf8(stderr).unwrap();
        assert!(
            stderr_str.contains("dominant-BIP bip84"),
            "stderr must name bip84 as dominant"
        );
        assert!(
            stderr_str.contains("bip44") && stderr_str.contains("bip49"),
            "stderr must enumerate dropped competing blocks bip44 + bip49"
        );
    }

    /// When bip86 is present alongside bip84/bip49/bip44, dominance picks
    /// bip86.
    #[test]
    fn parse_dominance_picks_bip86_when_present() {
        let blob = br#"{
  "chain": "BTC",
  "xfp": "5436D724",
  "account": 0,
  "bip84": {"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"127EA0E6","xpub":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9"},
  "bip86": {"name":"p2tr","deriv":"m/86'/0'/0'","xfp":"127EA0E6","xpub":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9"}
}
"#;
        let mut stderr = Vec::new();
        let parsed = ColdcardParser::parse(blob, &mut stderr).unwrap();
        assert_eq!(parsed[0].cosigners[0].path.to_string(), "86'/0'/0'");
    }

    // ---------------------------------------------------------------------
    // Parse impl — refusal cases
    // ---------------------------------------------------------------------

    /// Mk1/Mk2 legacy top-level `xpub` only → refused per SPEC §11.3.1 step
    /// 5 + FOLLOWUP pointer. The sniff predicate at P3A accepts these
    /// blobs (clause 3 disjunction allows top-level xpub), so the refusal
    /// must surface at parse time.
    #[test]
    fn parse_legacy_mk1_top_level_xpub_only_refused() {
        let blob = br#"{
  "chain":"BTC","xfp":"5436D724","account":0,
  "xpub":"xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX"
}
"#;
        let mut stderr = Vec::new();
        let err = ColdcardParser::parse(blob, &mut stderr).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("legacy Mk1/Mk2") && msg.contains("FOLLOWUP"),
            "Mk1/Mk2 fallback must surface clear refusal with FOLLOWUP pointer; got: {msg}"
        );
    }

    /// Multisig-only export (bip48_* present, no singlesig blocks) → refused
    /// with pointer to `--format coldcard-multisig` (Phase P4 surface).
    #[test]
    fn parse_bip48_only_refused_with_multisig_pointer() {
        let blob = br#"{
  "chain":"BTC","xfp":"5436D724","account":0,
  "bip48_2":{"name":"p2wsh","deriv":"m/48'/0'/0'/2'","xfp":"127EA0E6","xpub":"xpub..."}
}
"#;
        let mut stderr = Vec::new();
        let err = ColdcardParser::parse(blob, &mut stderr).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("bip48_") && msg.contains("coldcard-multisig"),
            "bip48_*-only blob must point user at --format coldcard-multisig; got: {msg}"
        );
    }

    /// `chain` value outside {BTC, XTN} → ImportWalletParse error (not
    /// sniff-time; sniff already rejected this, but defensive parse-time
    /// validation guards against explicit `--format coldcard` overrides).
    #[test]
    fn parse_chain_invalid_value_refused() {
        let blob = br#"{"chain":"REGTEST","xfp":"5436D724","bip84":{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"127EA0E6","xpub":"xpub..."}}"#;
        let mut stderr = Vec::new();
        let err = ColdcardParser::parse(blob, &mut stderr).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("chain") && (msg.contains("BTC") || msg.contains("XTN")),
            "invalid chain must surface BTC/XTN enumeration in error; got: {msg}"
        );
    }

    /// `xfp` non-8-char-hex → typed parse error.
    #[test]
    fn parse_xfp_wrong_length_refused() {
        let blob = br#"{"chain":"BTC","xfp":"DEAD","bip84":{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"127EA0E6","xpub":"xpub..."}}"#;
        let mut stderr = Vec::new();
        let err = ColdcardParser::parse(blob, &mut stderr).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("xfp") && msg.contains("8-char hex"),
            "wrong-length xfp must surface 8-char hex error; got: {msg}"
        );
    }

    /// `xfp` non-hex chars → typed parse error from hex decode.
    #[test]
    fn parse_xfp_non_hex_chars_refused() {
        let blob = br#"{"chain":"BTC","xfp":"NOTHEXZZ","bip84":{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"127EA0E6","xpub":"xpub..."}}"#;
        let mut stderr = Vec::new();
        let err = ColdcardParser::parse(blob, &mut stderr).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("hex decode"),
            "non-hex xfp must surface hex decode error; got: {msg}"
        );
    }

    /// Selected BIP block missing `deriv` → typed parse error naming the
    /// field.
    #[test]
    fn parse_block_missing_deriv_refused() {
        let blob = br#"{"chain":"BTC","xfp":"5436D724","bip84":{"name":"p2wpkh","xfp":"127EA0E6","xpub":"xpub..."}}"#;
        let mut stderr = Vec::new();
        let err = ColdcardParser::parse(blob, &mut stderr).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("bip84.deriv") && msg.contains("missing"),
            "missing deriv must name bip84.deriv; got: {msg}"
        );
    }

    /// Selected BIP block `deriv` does not start with `m/` → typed parse
    /// error.
    #[test]
    fn parse_block_deriv_without_m_prefix_refused() {
        let blob = br#"{"chain":"BTC","xfp":"5436D724","bip84":{"name":"p2wpkh","deriv":"84'/0'/0'","xfp":"127EA0E6","xpub":"xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9"}}"#;
        let mut stderr = Vec::new();
        let err = ColdcardParser::parse(blob, &mut stderr).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("does not start with `m/`"),
            "missing-m-prefix deriv must surface clear error; got: {msg}"
        );
    }

    /// Selected BIP block `xpub` unparsable → typed parse error.
    #[test]
    fn parse_block_xpub_unparsable_refused() {
        let blob = br#"{"chain":"BTC","xfp":"5436D724","bip84":{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"127EA0E6","xpub":"not-a-real-xpub"}}"#;
        let mut stderr = Vec::new();
        let err = ColdcardParser::parse(blob, &mut stderr).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("xpub") && (msg.contains("slip132") || msg.contains("parse")),
            "unparsable xpub must surface clear error; got: {msg}"
        );
    }

    /// `account` non-integer → typed parse error.
    #[test]
    fn parse_account_non_integer_refused() {
        let blob = br#"{"chain":"BTC","xfp":"5436D724","account":"not_a_number","bip84":{"name":"p2wpkh","deriv":"m/84'/0'/0'","xfp":"127EA0E6","xpub":"xpub..."}}"#;
        let mut stderr = Vec::new();
        let err = ColdcardParser::parse(blob, &mut stderr).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("account") && msg.contains("number"),
            "non-number account must surface typed error; got: {msg}"
        );
    }
}
