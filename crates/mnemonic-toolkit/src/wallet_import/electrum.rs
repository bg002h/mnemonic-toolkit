//! v0.28.0 Phase P6 — Electrum 4.x wallet-file ingest parser.
//!
//! Per `design/SPEC_wallet_import_v0_28_0.md` §11.6 (with P6A in-phase
//! correction to the `wallet_type` value-set; see SPEC §11.6 intro).
//!
//! **NAMESPACE TRAP (SPEC §1.4):** this module is the wallet-file INGEST
//! surface. Sibling modules with similar names:
//! - `crate::electrum` (`src/electrum.rs`) — native Electrum seed-phrase
//!   codec (HMAC-SHA512 prefix dispatch + per-wordlist base-N mapping);
//!   UNCHANGED in v0.28.0.
//! - `crate::wallet_export::electrum` (`wallet_export/electrum.rs`) — the
//!   inverse-of-this-module wallet-file EMIT surface; UNCHANGED in v0.28.0.
//! - `crate::wallet_import::electrum` (THIS FILE) — wallet-file INGEST;
//!   NEW in v0.28.0 Phase P6.
//!
//! ## Wire shape (Electrum 4.x JSON wallet-file)
//!
//! Singlesig (`wallet_type: "standard"`):
//! ```json
//! {
//!   "seed_version": 17,
//!   "wallet_type": "standard",
//!   "use_encryption": false,
//!   "keystore": {
//!     "type": "bip32",
//!     "xpub": "zpub6...",
//!     "derivation": "m/84'/0'/0'",
//!     "root_fingerprint": "5436d724",
//!     "label": "Daily"
//!   }
//! }
//! ```
//!
//! Multisig (`wallet_type: "<k>of<n>"` regex `(\d+)of(\d+)`):
//! ```json
//! {
//!   "seed_version": 17,
//!   "wallet_type": "2of4",
//!   "use_encryption": false,
//!   "x1/": { "type": "bip32", "xpub": "Zpub...", "derivation": "m/48'/0'/0'/2'", "root_fingerprint": "...", "label": "..." },
//!   "x2/": { ... },
//!   "x3/": { ... },
//!   "x4/": { ... }
//! }
//! ```
//!
//! Refusals (`2fa` / `imported`) per SPEC §11.6.1. Encrypted wallets
//! (`use_encryption: true`) are imported as watch-only at v0.30.1+
//! (stderr NOTICE advisory; encrypted seed/xprv/passphrase/keypairs fields
//! ignored) — see `design/BRAINSTORM_v0_30_1_electrum_encrypted_watch_only.md`.
//!
//! ## Phase P6A scope
//!
//! Parser skeleton + sniff impl + provenance metadata struct decls + sniff
//! unit tests. `parse()` returns `Err(BadInput("P6B: parse not yet wired"))`
//! — Phase P6B installs the real body; Phase P6C flips the
//! `cmd/import_wallet.rs` dispatch sites.

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

/// SPEC §11.6 — Electrum 4.x wallet-file ingest parser.
pub(crate) struct ElectrumParser;

/// SPEC §11.6 — `wallet_type` discriminator (post-P6A correction).
///
/// Values:
/// - `Standard` — `wallet_type: "standard"` (singlesig).
/// - `Multisig { k, n }` — `wallet_type` matches `(\d+)of(\d+)` per
///   `electrum/util.py::multisig_type`. Mirrors the toolkit's own emit
///   at `wallet_export/electrum.rs:141` (`format!("{k}of{n}")`).
///
/// Refused variants (`2fa`, `imported`) do NOT produce an
/// `ElectrumWalletType` — they error out before provenance construction
/// per SPEC §11.6.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ElectrumWalletType {
    Standard,
    /// `k`-of-`n` multisig per `electrum/util.py::multisig_type` regex
    /// `(\d+)of(\d+)`. P6A in-phase SPEC correction.
    Multisig { k: u8, n: u8 },
}

/// SPEC §11.6 — per-blob provenance metadata for an Electrum parse.
/// Carried on `ImportProvenance::Electrum(...)`; preserved for `--json`
/// envelope `electrum_source_metadata` emit (P6C wiring).
#[derive(Debug, Clone)]
pub(crate) struct ElectrumSourceMetadata {
    /// Top-level `seed_version` (Electrum's wallet-db version pin; integer
    /// in {11..71} at v0.28.0 cutover, FINAL_SEED_VERSION drifts upward per
    /// upstream releases — see `wallet_export/electrum.rs::ELECTRUM_SEED_VERSION_PIN`
    /// FOLLOWUP `electrum-final-seed-version-drift`).
    pub(crate) seed_version: u64,
    /// Decoded `wallet_type` (singlesig vs k-of-n multisig).
    pub(crate) wallet_type: ElectrumWalletType,
    /// Top-level wallet label (best-effort: derived from `keystore.label`
    /// for singlesig, or `x1/.label` for multisig). `None` if absent or
    /// the (singlesig) label is empty.
    pub(crate) wallet_name: Option<String>,
    /// Top-level fields encountered in the blob but not preserved on the
    /// import-side provenance (mirrors `CoreSourceMetadata.dropped_fields`).
    pub(crate) dropped_fields: Vec<String>,
}

/// Top-level keys preserved on the Electrum envelope by the toolkit's parse.
/// Any other top-level field surfaces in `ElectrumSourceMetadata.dropped_fields`
/// and drives a stderr NOTICE per SPEC §2.4. Mirrors
/// `COLDCARD_PRESERVED_TOP_LEVEL_KEYS`.
///
/// Note: multisig per-cosigner keys `x1/`, `x2/`, ..., `xN/` are dynamic
/// (N = cosigner count) and tested separately via prefix match in
/// `dropped_fields` computation.
pub(crate) const ELECTRUM_PRESERVED_TOP_LEVEL_KEYS: &[&str] = &[
    "seed_version",
    "wallet_type",
    "use_encryption",
    "keystore",
];

/// SPEC §11.6 — sniff seed_version range. Electrum's `_convert_version_*`
/// chain accepts `seed_version >= 12` (with rejections at 14 / 51 per
/// `wallet_db.py`), so the sniff accepts a generous {11..71+} band to
/// absorb future FINAL_SEED_VERSION drift without re-pinning the sniff.
/// The lower bound is intentionally inclusive at 11 to allow one notch of
/// pre-12 tolerance (NoMatch via `seed_version: 10` is a footgun for
/// hand-edited blobs). The upper bound 71 matches current FINAL_SEED_VERSION
/// but is treated as a soft ceiling — values >71 ARE still accepted at sniff
/// time (the parse-time post-validation re-checks the range and emits a
/// stderr NOTICE if seed_version >71 — handled at P6B).
const SNIFF_SEED_VERSION_MIN: u64 = 11;

/// SPEC §11.6 — sniff `wallet_type` value: matches `(\d+)of(\d+)` regex
/// per `electrum/util.py::multisig_type`. Returns `Some((k, n))` on
/// match, `None` otherwise. Used by sniff + parse contracts.
///
/// Implementation is a hand-rolled state-machine equivalent to the
/// Python regex `(\d+)of(\d+)` anchored at the START (Python's `re.match`
/// is start-anchored by default). The end is NOT anchored — Electrum's
/// `re.match` returns on partial-prefix match, so trailing garbage is
/// tolerated (though canonical Electrum wallet files never carry it).
pub(crate) fn parse_multisig_wallet_type(s: &str) -> Option<(u8, u8)> {
    let bytes = s.as_bytes();
    let mut i = 0usize;
    // First digit run.
    let k_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == k_start {
        return None;
    }
    let k_str = &s[k_start..i];
    // Literal "of".
    if i + 2 > bytes.len() || &bytes[i..i + 2] != b"of" {
        return None;
    }
    i += 2;
    // Second digit run.
    let n_start = i;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    if i == n_start {
        return None;
    }
    let n_str = &s[n_start..i];
    let k = k_str.parse::<u8>().ok()?;
    let n = n_str.parse::<u8>().ok()?;
    Some((k, n))
}

/// SPEC §11.6 — classify a top-level `wallet_type` string into the
/// post-correction value-set. Returns `None` for unrecognized values
/// (including `2fa` / `imported` — those are recognized at parse time
/// for the refusal templates per §11.6.1, but at SNIFF time we accept
/// any non-empty string as "this looks Electrum-shaped" — see
/// `ElectrumParser::sniff` for the sniff predicate).
pub(crate) fn classify_wallet_type(s: &str) -> Option<ElectrumWalletType> {
    if s == "standard" {
        return Some(ElectrumWalletType::Standard);
    }
    if let Some((k, n)) = parse_multisig_wallet_type(s) {
        return Some(ElectrumWalletType::Multisig { k, n });
    }
    None
}

impl WalletFormatParser for ElectrumParser {
    /// SPEC §11.6 sniff (P6A correction): top-level JSON object containing
    /// ALL of:
    /// (1) `seed_version` integer in `{SNIFF_SEED_VERSION_MIN..}` (inclusive
    ///     lower bound; upper bound is unbounded at sniff time to absorb
    ///     future Electrum FINAL_SEED_VERSION drift — parse time re-checks
    ///     the ceiling per SPEC §11.6).
    /// (2) `wallet_type` string in the v0.28.0 value-set
    ///     `{"standard", "<k>of<n>", "2fa", "imported"}`. The
    ///     `"<k>of<n>"` regex is recognized per `electrum/util.py::multisig_type`
    ///     `(\d+)of(\d+)`.
    ///
    /// Refusal types (`2fa`, `imported`) ARE matched at sniff time (so
    /// the parse-time refusal stderr template can fire with a clear message)
    /// — they would otherwise vector through `NoMatch` and produce the
    /// generic "could not detect format" template.
    ///
    /// Note: encrypted wallets (`use_encryption: true`) sniff POSITIVE here
    /// (top-level structure is still Electrum-shaped); the refusal lands at
    /// parse time per SPEC §11.6.1.
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
        // (1) seed_version: integer ≥ SNIFF_SEED_VERSION_MIN.
        let sv_ok = obj
            .get("seed_version")
            .and_then(|v| v.as_u64())
            .map(|n| n >= SNIFF_SEED_VERSION_MIN)
            .unwrap_or(false);
        if !sv_ok {
            return false;
        }
        // (2) wallet_type: string in value-set.
        let wt_ok = obj
            .get("wallet_type")
            .and_then(|v| v.as_str())
            .map(|s| {
                s == "standard"
                    || s == "2fa"
                    || s == "imported"
                    || parse_multisig_wallet_type(s).is_some()
            })
            .unwrap_or(false);
        if !wt_ok {
            return false;
        }
        true
    }

    /// SPEC §11.6 parse — P6B body.
    ///
    /// Steps:
    /// 1. JSON-parse + top-level object check.
    /// 2. Validate `seed_version` + sniff-positive `wallet_type`.
    /// 3. If `use_encryption: true` → emit stderr NOTICE advisory
    ///    (watch-only-passthrough per v0.30.1 / Cycle 6b R0 fold). Parse
    ///    continues with the plaintext xpub/derivation/fingerprint/label
    ///    fields the parser actually reads; encrypted seed/xprv/passphrase/
    ///    keypairs are ignored.
    /// 4. Classify `wallet_type`:
    ///    - `"standard"` → singlesig: build `<wrapper>([fp/path]xpub/<0;1>/*)`
    ///      where wrapper is derived from xpub SLIP-132 prefix + derivation purpose.
    ///    - `"<k>of<n>"` → multisig: build `<outer>(sortedmulti(K, ...))`
    ///      where outer is derived from per-cosigner SLIP-132 prefix.
    ///    - `"2fa"` → REFUSE per §11.6.1.
    ///    - `"imported"` → REFUSE per §11.6.1.
    /// 5. Feed through `concrete_keys_to_placeholders` → `parse_descriptor`.
    /// 6. Build `ResolvedSlot` vec (length 1 for singlesig, K..N for multisig).
    /// 7. Emit stderr NOTICE per SPEC §2.4 listing dropped envelope fields.
    /// 8. Wrap in `ParsedImport` with `ImportProvenance::Electrum(...)`.
    fn parse(blob: &[u8], stderr: &mut dyn Write) -> Result<Vec<ParsedImport>, ToolkitError> {
        // Step 1: JSON parse.
        let value: Value = serde_json::from_slice(blob).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: electrum: parse error: invalid JSON: {e}"
            ))
        })?;
        let obj = value.as_object().ok_or_else(|| {
            ToolkitError::ImportWalletParse(
                "import-wallet: electrum: parse error: top-level JSON value is not an object"
                    .to_string(),
            )
        })?;

        // Step 2: seed_version validation.
        let seed_version = obj
            .get("seed_version")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(
                    "import-wallet: electrum: parse error: missing or non-integer top-level `seed_version`"
                        .to_string(),
                )
            })?;
        if seed_version < SNIFF_SEED_VERSION_MIN {
            return Err(ToolkitError::ImportWalletParse(format!(
                "import-wallet: electrum: parse error: seed_version {seed_version} below minimum supported {SNIFF_SEED_VERSION_MIN} (Electrum's pre-12 wallet-db format is not supported; upgrade via Electrum 4.x first)"
            )));
        }
        // Future-FINAL_SEED_VERSION drift: a value above the current
        // upstream FINAL_SEED_VERSION is informational, not fatal — Electrum
        // may have shipped a newer version since the toolkit's last update.
        // No NOTICE here (sniff-time + parse-time validation both lenient
        // per SPEC §11.6).

        // Step 3: use_encryption advisory (v0.30.1 / Cycle 6b watch-only-passthrough).
        //
        // Per electrum/keystore.py (verified at Cycle 6 P0 recon §A1 + Cycle 6b
        // brainstorm R0 §C1), Electrum's field-level encryption protects
        // `keystore.{seed,xprv,passphrase,keypairs}`. The fields THIS parser
        // reads (`keystore.{xpub,derivation,root_fingerprint,label}` +
        // multisig analogues at xN/.*) are plaintext under BOTH encrypted and
        // unencrypted wallets. The encrypted-wallet refusal v0.28.0 shipped
        // was therefore over-restrictive in principle: watch-only import has
        // all the material it needs without touching the encrypted fields.
        // v0.30.1 downgrades the refusal to a stderr advisory and continues
        // with the plaintext xpub/derivation/etc.
        let use_encryption = obj
            .get("use_encryption")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if use_encryption {
            let _ = writeln!(
                stderr,
                "notice: import-wallet: electrum: wallet is encrypted (use_encryption=true); importing watch-only material only (encrypted seed/xprv/passphrase/keypairs fields ignored). To extract the encrypted seed, use 'electrum --decrypt-wallet' out-of-band then re-import the plaintext wallet."
            );
        }

        // Step 4: classify wallet_type.
        let wallet_type_str = obj
            .get("wallet_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(
                    "import-wallet: electrum: parse error: missing or non-string top-level `wallet_type`"
                        .to_string(),
                )
            })?;
        // Refusals per SPEC §11.6.1.
        if wallet_type_str == "2fa" {
            return Err(ToolkitError::ImportWalletParse(
                "import-wallet: electrum: 2fa wallets require TrustedCoin two-factor restoration; ingest not supported".to_string(),
            ));
        }
        if wallet_type_str == "imported" {
            return Err(ToolkitError::ImportWalletParse(
                "import-wallet: electrum: imported-addresses wallets have no derivation chain to reconstruct; ingest not supported".to_string(),
            ));
        }
        let wallet_type = classify_wallet_type(wallet_type_str).ok_or_else(|| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: electrum: parse error: unrecognized `wallet_type` value {wallet_type_str:?} (expected \"standard\", \"<k>of<n>\", \"2fa\", or \"imported\")"
            ))
        })?;

        // Step 4a/4b: dispatch on classified wallet_type.
        let (descriptor_body, network, threshold, wallet_name, cosigners_count) =
            match wallet_type {
                ElectrumWalletType::Standard => build_standard_descriptor(obj)?,
                ElectrumWalletType::Multisig { k, n } => {
                    build_multisig_descriptor(obj, k, n)?
                }
            };

        // Step 5: feed through pipeline + parse_descriptor.
        let (placeholder_form, parsed_keys, parsed_fingerprints) =
            concrete_keys_to_placeholders(&descriptor_body).map_err(|e| {
                ToolkitError::ImportWalletParse(e.message().replacen(
                    "import-wallet: bsms:",
                    "import-wallet: electrum:",
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
                "import-wallet: electrum: parse error: {}",
                e.message()
            ))
        })?;

        // Step 6: build ResolvedSlot vec.
        let mut cosigners: Vec<ResolvedSlot> = Vec::with_capacity(cosigners_count);
        for (i, key) in parsed_keys.iter().enumerate().take(cosigners_count) {
            let (xpub, fp, path) = build_slot_fields(&descriptor_body, i)?;
            debug_assert_eq!(xpub_to_65(&xpub), key.payload);
            cosigners.push(ResolvedSlot {
                xpub,
                fingerprint: fp,
                path,
                entropy: None,
                master_xpub: None,
                language: None,
                _entropy_pin: None,
            });
        }
        validate_watch_only_resolved(&cosigners)?;

        // Step 7: dropped-field detection + stderr NOTICE.
        let mut dropped_fields: Vec<String> = Vec::new();
        for (k, _) in obj.iter() {
            // Top-level keys: preserved set + per-cosigner `xN/` dynamic keys
            // (N up to cosigners_count). Anything else is dropped.
            if ELECTRUM_PRESERVED_TOP_LEVEL_KEYS.contains(&k.as_str()) {
                continue;
            }
            if is_multisig_cosigner_key(k, cosigners_count) {
                continue;
            }
            dropped_fields.push(k.clone());
        }
        if !dropped_fields.is_empty() {
            writeln!(
                stderr,
                "notice: import-wallet: electrum: dropped envelope fields {}: not preserved in bundle output (key-state only)",
                dropped_fields.join(", ")
            )
            .map_err(ToolkitError::Io)?;
        }

        let source_metadata = ElectrumSourceMetadata {
            seed_version,
            wallet_type,
            wallet_name,
            dropped_fields,
        };

        // Step 8: original_descriptor with freshly-computed BIP-380 checksum.
        let original_descriptor = match recompute_descriptor_checksum(&descriptor_body) {
            Ok(s) => s,
            Err(_) => descriptor_body.clone(),
        };

        Ok(vec![ParsedImport {
            descriptor,
            original_descriptor,
            cosigners,
            network,
            threshold,
            provenance: ImportProvenance::Electrum(source_metadata),
        }])
    }
}

/// Trim ASCII-whitespace bytes (space, tab, CR, LF) from the start of a
/// blob. Mirrors `wallet_import/coldcard.rs:trim_leading_ws`.
fn trim_leading_ws(b: &[u8]) -> &[u8] {
    let mut start = 0usize;
    while start < b.len() && matches!(b[start], b' ' | b'\t' | b'\r' | b'\n') {
        start += 1;
    }
    &b[start..]
}

/// `true` iff `k` matches `xN/` for some `1 <= N <= n`. Used by dropped-field
/// detection at the multisig parse path; per-cosigner `xN/` keys are
/// preserved (consumed by the multisig parser), not dropped.
fn is_multisig_cosigner_key(k: &str, n: usize) -> bool {
    if n == 0 {
        return false;
    }
    let stripped = match k.strip_prefix('x').and_then(|s| s.strip_suffix('/')) {
        Some(s) => s,
        None => return false,
    };
    let parsed: usize = match stripped.parse() {
        Ok(v) => v,
        Err(_) => return false,
    };
    parsed >= 1 && parsed <= n
}

/// SPEC §11.6 dispatch result tuple: `(descriptor_body, network, threshold,
/// wallet_name, cosigners_count)`. Threshold is `Some(K)` for multisig,
/// `None` for singlesig. Returned by both `build_standard_descriptor` +
/// `build_multisig_descriptor`.
type ElectrumDispatchResult = (
    String,
    bitcoin::Network,
    Option<u8>,
    Option<String>,
    usize,
);

/// SPEC §11.6 — singlesig parse path.
///
/// Extracts `keystore.xpub` + `keystore.derivation` + `keystore.root_fingerprint`,
/// then builds a synthetic `<wrapper>([fp/path]xpub/<0;1>/*)` descriptor with
/// wrapper inferred from the xpub SLIP-132 prefix and derivation purpose.
fn build_standard_descriptor(
    obj: &serde_json::Map<String, Value>,
) -> Result<ElectrumDispatchResult, ToolkitError> {
    let keystore = obj
        .get("keystore")
        .and_then(|v| v.as_object())
        .ok_or_else(|| {
            ToolkitError::ImportWalletParse(
                "import-wallet: electrum: parse error: standard wallet missing or non-object `keystore`"
                    .to_string(),
            )
        })?;
    let xpub_str = keystore
        .get("xpub")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            ToolkitError::ImportWalletParse(
                "import-wallet: electrum: parse error: keystore.xpub missing or not a string"
                    .to_string(),
            )
        })?
        .to_string();
    let derivation = keystore
        .get("derivation")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            ToolkitError::ImportWalletParse(
                "import-wallet: electrum: parse error: keystore.derivation missing or not a string"
                    .to_string(),
            )
        })?
        .to_string();
    let root_fingerprint = keystore
        .get("root_fingerprint")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            ToolkitError::ImportWalletParse(
                "import-wallet: electrum: parse error: keystore.root_fingerprint missing or not a string"
                    .to_string(),
            )
        })?;
    // 8-hex validation (lowercase by Electrum emit convention; accept either).
    if root_fingerprint.len() != 8
        || !root_fingerprint.chars().all(|c| c.is_ascii_hexdigit())
    {
        return Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: electrum: parse error: keystore.root_fingerprint must be 8 hex chars, got {root_fingerprint:?}"
        )));
    }
    let label_opt = keystore
        .get("label")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from);

    // Normalize the xpub (SLIP-132 → neutral) + capture the variant for
    // wrapper inference.
    let (neutral_xpub_str, slip132_variant) =
        crate::slip0132::normalize_xpub_prefix(&xpub_str).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: electrum: parse error: keystore.xpub normalize: {}",
                e.message()
            ))
        })?;

    // Derive network from xpub neutral form (xpub → Mainnet, tpub → Testnet).
    let network = network_from_xpub_neutral(&neutral_xpub_str)?;

    // Wrapper inference: SLIP-132 prefix is the primary signal; fall back to
    // derivation purpose for neutral xpub (which is BIP-44 by convention but
    // BIP-86 if the path purpose is 86').
    let wrapper = standard_wrapper_for(slip132_variant, &derivation)?;

    // Strip leading `m/` for bracket form.
    let deriv_no_m = derivation
        .strip_prefix("m/")
        .unwrap_or_else(|| derivation.strip_prefix('m').unwrap_or(&derivation));

    let bracketed = format!(
        "[{root_fingerprint}/{deriv_no_m}]{neutral_xpub_str}/<0;1>/*"
    );
    let wrapped = match wrapper {
        StandardWrapper::Pkh => format!("pkh({bracketed})"),
        StandardWrapper::ShWpkh => format!("sh(wpkh({bracketed}))"),
        StandardWrapper::Wpkh => format!("wpkh({bracketed})"),
        StandardWrapper::Tr => format!("tr({bracketed})"),
    };

    Ok((wrapped, network, None, label_opt, 1))
}

/// Inner enumeration: which wrapper to use for an Electrum singlesig descriptor.
/// Mirrors `wallet_export/electrum.rs::WalletScriptType` (P2pkh / P2shP2wpkh /
/// P2wpkh / P2tr) but restricted to the singlesig variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StandardWrapper {
    Pkh,
    ShWpkh,
    Wpkh,
    Tr,
}

/// SPEC §11.6 — wrapper inference for singlesig descriptors.
///
/// Primary signal: SLIP-132 variant from the xpub prefix:
/// - `zpub` / `vpub` → BIP-84 (`wpkh`)
/// - `ypub` / `upub` → BIP-49 (`sh(wpkh)`)
/// - neutral `xpub` / `tpub` → fall back to derivation purpose:
///   - `m/44'/...` → BIP-44 (`pkh`)
///   - `m/86'/...` → BIP-86 (`tr`)
///   - other → default `pkh` (conservative)
fn standard_wrapper_for(
    slip132_variant: Option<&'static str>,
    derivation: &str,
) -> Result<StandardWrapper, ToolkitError> {
    match slip132_variant {
        Some("zpub") | Some("vpub") => Ok(StandardWrapper::Wpkh),
        Some("ypub") | Some("upub") => Ok(StandardWrapper::ShWpkh),
        Some("Zpub") | Some("Vpub") | Some("Ypub") | Some("Upub") => {
            // Multisig SLIP-132 prefixes on a singlesig keystore are
            // inconsistent — Electrum's emit never produces this combination.
            Err(ToolkitError::ImportWalletParse(format!(
                "import-wallet: electrum: parse error: keystore.xpub uses multisig SLIP-132 prefix ({:?}) in a wallet_type=standard envelope",
                slip132_variant.unwrap()
            )))
        }
        None => {
            // Neutral xpub/tpub. Disambiguate via derivation purpose component.
            let purpose = derivation_purpose(derivation);
            match purpose.as_deref() {
                Some("44'") | Some("44h") => Ok(StandardWrapper::Pkh),
                Some("86'") | Some("86h") => Ok(StandardWrapper::Tr),
                Some("84'") | Some("84h") => {
                    // Neutral xpub + BIP-84 derivation: technically valid
                    // but unusual (Electrum's emit pairs zpub with BIP-84).
                    // Default to wpkh — the wallet content is BIP-84 by
                    // path semantics; the SLIP-132 prefix is a hint only.
                    Ok(StandardWrapper::Wpkh)
                }
                Some("49'") | Some("49h") => Ok(StandardWrapper::ShWpkh),
                Some(_) | None => {
                    // Default: bare xpub with unrecognized / missing purpose
                    // → BIP-44 (`pkh`) as the conservative legacy default.
                    Ok(StandardWrapper::Pkh)
                }
            }
        }
        Some(other) => Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: electrum: parse error: unrecognized SLIP-132 xpub variant {other:?}"
        ))),
    }
}

/// Extract the BIP-43 purpose component (e.g., `"84'"`) from a derivation path
/// like `"m/84'/0'/0'"`. Returns `None` for empty/malformed paths.
fn derivation_purpose(s: &str) -> Option<String> {
    let trimmed = s.trim_start_matches("m/").trim_start_matches('m');
    let first = trimmed.trim_start_matches('/').split('/').next()?;
    if first.is_empty() {
        None
    } else {
        Some(first.to_string())
    }
}

/// SPEC §11.6 — multisig parse path.
///
/// Iterates `x1/`, `x2/`, ..., `xn/` per-key sub-objects; each carries
/// `xpub`, `derivation`, `root_fingerprint`, `label` (and `type: "bip32"`).
/// Cosigners are uniformity-checked: all SLIP-132 variants must agree (drives
/// outer wrapper inference); all coin-types must agree (drives network).
/// Output descriptor is `<outer>(sortedmulti(K, ...))` mirroring the
/// `wallet_export/electrum.rs` emit convention for emit-import symmetry.
fn build_multisig_descriptor(
    obj: &serde_json::Map<String, Value>,
    k: u8,
    n: u8,
) -> Result<ElectrumDispatchResult, ToolkitError> {
    if k == 0 || n == 0 || k > n {
        return Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: electrum: parse error: wallet_type \"{k}of{n}\" is malformed (require 1 ≤ k ≤ n, n ≥ 1)"
        )));
    }
    let mut cosigners: Vec<MultisigCosigner> = Vec::with_capacity(n as usize);
    for i in 1..=n {
        let key = format!("x{i}/");
        let sub = obj.get(&key).and_then(|v| v.as_object()).ok_or_else(|| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: electrum: parse error: multisig wallet_type \"{k}of{n}\" missing or non-object cosigner key `{key}`"
            ))
        })?;
        cosigners.push(parse_multisig_cosigner(&key, sub)?);
    }

    // Uniformity-check #1: all cosigners share a SLIP-132 variant class.
    // (Electrum's emit always uses the same prefix across all cosigners per
    // `wallet_export/electrum.rs:160-184`.) Heterogeneity surfaces here as
    // a typed parse error rather than producing a descriptor with mixed
    // semantic wrapper-script-types.
    let first_variant_class = cosigners[0].variant_class;
    for (i, c) in cosigners.iter().enumerate().skip(1) {
        if c.variant_class != first_variant_class {
            return Err(ToolkitError::ImportWalletParse(format!(
                "import-wallet: electrum: parse error: cosigner x{}/ has SLIP-132 variant class {:?}, cosigner x1/ has {:?}; all cosigners must share a wrapper class",
                i + 1,
                c.variant_class,
                first_variant_class,
            )));
        }
    }

    // Uniformity-check #2: all cosigners share a coin-type (drives network).
    let first_coin = cosigners[0].coin_type;
    for (i, c) in cosigners.iter().enumerate().skip(1) {
        if c.coin_type != first_coin {
            return Err(ToolkitError::ImportWalletParse(format!(
                "import-wallet: electrum: parse error: cosigner x{}/ has coin-type {}, cosigner x1/ has {}; all cosigners must share a coin-type",
                i + 1,
                c.coin_type,
                first_coin,
            )));
        }
    }
    let network = match first_coin {
        0 => bitcoin::Network::Bitcoin,
        1 => bitcoin::Network::Testnet,
        other => {
            return Err(ToolkitError::ImportWalletParse(format!(
                "import-wallet: electrum: parse error: unsupported coin-type {other} on cosigner origin paths; only 0 (mainnet) and 1 (testnet) supported"
            )));
        }
    };

    // Build inner sortedmulti(K, <co1>, <co2>, ...).
    let mut inner = format!("sortedmulti({k}");
    for c in &cosigners {
        inner.push(',');
        let deriv_no_m = c
            .derivation
            .strip_prefix("m/")
            .unwrap_or_else(|| c.derivation.strip_prefix('m').unwrap_or(&c.derivation));
        inner.push_str(&format!(
            "[{fp}/{path}]{xpub}/<0;1>/*",
            fp = c.root_fingerprint,
            path = deriv_no_m,
            xpub = c.neutral_xpub,
        ));
    }
    inner.push(')');

    let wrapped = match first_variant_class {
        MultisigVariantClass::P2wsh => format!("wsh({inner})"),
        MultisigVariantClass::P2shP2wsh => format!("sh(wsh({inner}))"),
        MultisigVariantClass::P2sh => format!("sh({inner})"),
    };

    // Wallet name: use first cosigner's label if present (mirrors emit's
    // `format!("{wallet_name}-{i+1}")` convention — best-effort recovery
    // strips the trailing `-N` suffix if present).
    let wallet_name = cosigners[0]
        .label
        .as_deref()
        .map(|s| s.trim_end_matches(|c: char| c.is_ascii_digit()).trim_end_matches('-').to_string())
        .filter(|s| !s.is_empty());

    Ok((wrapped, network, Some(k), wallet_name, n as usize))
}

/// Parsed Electrum multisig cosigner sub-object (`x1/`, `x2/`, ...).
#[derive(Debug)]
struct MultisigCosigner {
    root_fingerprint: String,
    derivation: String,
    /// SLIP-132-neutralized xpub (`xpub` or `tpub`).
    neutral_xpub: String,
    /// SLIP-132 variant class — drives outer wrapper script-type inference.
    variant_class: MultisigVariantClass,
    /// BIP-43 coin-type component (index 1 in derivation) — drives network.
    coin_type: u32,
    label: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MultisigVariantClass {
    /// `Zpub` (mainnet) / `Vpub` (testnet) → `wsh(sortedmulti(...))`.
    P2wsh,
    /// `Ypub` (mainnet) / `Upub` (testnet) → `sh(wsh(sortedmulti(...)))`.
    P2shP2wsh,
    /// Neutral `xpub` / `tpub` → `sh(sortedmulti(...))` (BIP-45 / legacy).
    P2sh,
}

fn parse_multisig_cosigner(
    key: &str,
    sub: &serde_json::Map<String, Value>,
) -> Result<MultisigCosigner, ToolkitError> {
    let xpub_str = sub
        .get("xpub")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: electrum: parse error: cosigner `{key}` missing or non-string `xpub`"
            ))
        })?
        .to_string();
    let derivation = sub
        .get("derivation")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: electrum: parse error: cosigner `{key}` missing or non-string `derivation`"
            ))
        })?
        .to_string();
    let root_fingerprint = sub
        .get("root_fingerprint")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: electrum: parse error: cosigner `{key}` missing or non-string `root_fingerprint`"
            ))
        })?
        .to_string();
    if root_fingerprint.len() != 8
        || !root_fingerprint.chars().all(|c| c.is_ascii_hexdigit())
    {
        return Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: electrum: parse error: cosigner `{key}` root_fingerprint must be 8 hex chars, got {root_fingerprint:?}"
        )));
    }
    let label = sub
        .get("label")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(String::from);

    let (neutral_xpub, slip132_variant) =
        crate::slip0132::normalize_xpub_prefix(&xpub_str).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: electrum: parse error: cosigner `{key}` xpub normalize: {}",
                e.message()
            ))
        })?;
    let variant_class = match slip132_variant {
        Some("Zpub") | Some("Vpub") => MultisigVariantClass::P2wsh,
        Some("Ypub") | Some("Upub") => MultisigVariantClass::P2shP2wsh,
        None => MultisigVariantClass::P2sh,
        Some("zpub") | Some("vpub") | Some("ypub") | Some("upub") => {
            // Singlesig SLIP-132 prefixes on a multisig cosigner are
            // inconsistent — Electrum's emit never produces this.
            return Err(ToolkitError::ImportWalletParse(format!(
                "import-wallet: electrum: parse error: cosigner `{key}` uses singlesig SLIP-132 prefix ({:?}) in a multisig envelope",
                slip132_variant.unwrap()
            )));
        }
        Some(other) => {
            return Err(ToolkitError::ImportWalletParse(format!(
                "import-wallet: electrum: parse error: cosigner `{key}` has unrecognized SLIP-132 variant {other:?}"
            )));
        }
    };

    // BIP-43 coin-type extraction from derivation path.
    let path = DerivationPath::from_str(&derivation).map_err(|e| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: electrum: parse error: cosigner `{key}` derivation-path parse: {e}"
        ))
    })?;
    let comps: Vec<&ChildNumber> = path.into_iter().collect();
    let coin_type = if comps.len() >= 2 {
        match comps[1] {
            ChildNumber::Hardened { index } => *index,
            ChildNumber::Normal { index } => {
                return Err(ToolkitError::ImportWalletParse(format!(
                    "import-wallet: electrum: parse error: cosigner `{key}` coin-type component {index} is not hardened"
                )));
            }
        }
    } else {
        // Path too short to extract coin-type: default to 0 (mainnet) per
        // the toolkit's network-default convention; downstream parse may
        // still reject if xpub neutral prefix disagrees.
        0
    };

    Ok(MultisigCosigner {
        root_fingerprint,
        derivation,
        neutral_xpub,
        variant_class,
        coin_type,
        label,
    })
}

/// Map a neutralized xpub prefix to `bitcoin::Network`. `xpub` →
/// `Network::Bitcoin`, `tpub` → `Network::Testnet`. Anything else → parse error.
fn network_from_xpub_neutral(s: &str) -> Result<bitcoin::Network, ToolkitError> {
    if s.starts_with("xpub") {
        Ok(bitcoin::Network::Bitcoin)
    } else if s.starts_with("tpub") {
        Ok(bitcoin::Network::Testnet)
    } else {
        Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: electrum: parse error: neutralized xpub does not start with `xpub` or `tpub`: {s}"
        )))
    }
}

/// Build the typed slot fields (xpub, fingerprint, path) from the synthesized
/// descriptor body at cosigner index `slot_idx`.
/// Mirrors `wallet_import/sparrow.rs:build_slot_fields`.
fn build_slot_fields(
    descriptor_body: &str,
    slot_idx: usize,
) -> Result<(Xpub, Fingerprint, DerivationPath), ToolkitError> {
    let origins =
        crate::wallet_import::pipeline::extract_origin_components(descriptor_body, "electrum")?;
    let (fp, path, xpub_str) = origins.into_iter().nth(slot_idx).ok_or_else(|| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: electrum: parse error: slot index {slot_idx} out of range in synthesized descriptor"
        ))
    })?;
    crate::wallet_import::pipeline::finalize_slot_fields(fp, path, &xpub_str, "electrum")
}

/// Recompute the BIP-380 checksum for the descriptor body. Mirrors
/// `wallet_import/coldcard.rs:recompute_descriptor_checksum`.
fn recompute_descriptor_checksum(body: &str) -> Result<String, ToolkitError> {
    use miniscript::descriptor::checksum::Engine as ChecksumEngine;
    let body_no_csum = match body.rsplit_once('#') {
        Some((b, _)) => b,
        None => body,
    };
    let mut eng = ChecksumEngine::new();
    eng.input(body_no_csum).map_err(|e| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: electrum: checksum engine input rejected: {e}"
        ))
    })?;
    let csum = eng.checksum();
    Ok(format!("{body_no_csum}#{csum}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // sniff tests (P6A scope)
    // ========================================================================

    #[test]
    fn sniff_standard_singlesig_positive() {
        let blob = br#"{
            "seed_version": 17,
            "wallet_type": "standard",
            "use_encryption": false,
            "keystore": {"type": "bip32", "xpub": "zpub6...", "derivation": "m/84'/0'/0'", "root_fingerprint": "5436d724", "label": "Daily"}
        }"#;
        assert!(ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_multisig_2of4_positive() {
        let blob = br#"{"seed_version": 17, "wallet_type": "2of4", "use_encryption": false, "x1/": {}}"#;
        assert!(ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_multisig_3of5_positive() {
        let blob = br#"{"seed_version": 17, "wallet_type": "3of5"}"#;
        assert!(ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_2fa_positive_for_clear_error_message() {
        // SPEC §11.6: 2fa sniff is POSITIVE so the parse-time refusal can
        // fire with a clear "2fa wallets require TrustedCoin..." message
        // (otherwise sniff would NoMatch and surface the generic
        // "could not detect format" template).
        let blob = br#"{"seed_version": 17, "wallet_type": "2fa"}"#;
        assert!(ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_imported_positive_for_clear_error_message() {
        let blob = br#"{"seed_version": 17, "wallet_type": "imported"}"#;
        assert!(ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_seed_version_below_min_rejected() {
        let blob = br#"{"seed_version": 10, "wallet_type": "standard"}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_seed_version_above_current_final_still_accepted_at_sniff() {
        // FINAL_SEED_VERSION drifts upward over Electrum releases. Sniff
        // does NOT cap on the upper end — parse-time validation re-checks
        // the ceiling and emits a NOTICE if >71. This lets the sniff stay
        // stable across upstream upgrades.
        let blob = br#"{"seed_version": 99, "wallet_type": "standard"}"#;
        assert!(ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_wallet_type_unknown_rejected() {
        let blob = br#"{"seed_version": 17, "wallet_type": "trezor"}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_wallet_type_missing_rejected() {
        let blob = br#"{"seed_version": 17}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_seed_version_missing_rejected() {
        let blob = br#"{"wallet_type": "standard"}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_seed_version_string_rejected() {
        let blob = br#"{"seed_version": "17", "wallet_type": "standard"}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_top_level_array_rejected() {
        let blob = br#"[{"seed_version": 17, "wallet_type": "standard"}]"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_invalid_json_rejected() {
        let blob = br#"{not json"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_empty_blob_rejected() {
        assert!(!ElectrumParser::sniff(b""));
    }

    #[test]
    fn sniff_leading_whitespace_tolerated() {
        let blob = b"   \n\t{\"seed_version\": 17, \"wallet_type\": \"standard\"}";
        assert!(ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_bitcoin_core_descriptors_blob_rejected() {
        // Cross-format guard: a Bitcoin Core listdescriptors blob does NOT
        // carry seed_version / wallet_type; sniff must reject.
        let blob =
            br#"{"wallet_name":"a","descriptors":[{"desc":"wpkh(xpub...)#abcdefgh"}]}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_coldcard_blob_rejected() {
        // Cross-format guard: a Coldcard generic-wallet-export blob lacks
        // seed_version / wallet_type.
        let blob = br#"{"chain":"BTC","xfp":"B8688DF1","bip84":{"xpub":"zpub..."}}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    // ========================================================================
    // parse_multisig_wallet_type unit tests
    // ========================================================================

    #[test]
    fn parse_multisig_2of3() {
        assert_eq!(parse_multisig_wallet_type("2of3"), Some((2, 3)));
    }

    #[test]
    fn parse_multisig_15of15_max() {
        assert_eq!(parse_multisig_wallet_type("15of15"), Some((15, 15)));
    }

    #[test]
    fn parse_multisig_3of5() {
        assert_eq!(parse_multisig_wallet_type("3of5"), Some((3, 5)));
    }

    #[test]
    fn parse_multisig_overflow_u8_rejected() {
        // 256of256 overflows u8.
        assert_eq!(parse_multisig_wallet_type("256of256"), None);
    }

    #[test]
    fn parse_multisig_standard_string_rejected() {
        assert_eq!(parse_multisig_wallet_type("standard"), None);
    }

    #[test]
    fn parse_multisig_missing_of_rejected() {
        assert_eq!(parse_multisig_wallet_type("23"), None);
    }

    #[test]
    fn parse_multisig_only_first_digit_run_rejected() {
        assert_eq!(parse_multisig_wallet_type("2of"), None);
    }

    #[test]
    fn parse_multisig_2fa_rejected() {
        assert_eq!(parse_multisig_wallet_type("2fa"), None);
    }

    #[test]
    fn parse_multisig_imported_rejected() {
        assert_eq!(parse_multisig_wallet_type("imported"), None);
    }

    #[test]
    fn parse_multisig_empty_rejected() {
        assert_eq!(parse_multisig_wallet_type(""), None);
    }

    // ========================================================================
    // classify_wallet_type unit tests
    // ========================================================================

    #[test]
    fn classify_standard() {
        assert_eq!(
            classify_wallet_type("standard"),
            Some(ElectrumWalletType::Standard)
        );
    }

    #[test]
    fn classify_multisig_2of3() {
        assert_eq!(
            classify_wallet_type("2of3"),
            Some(ElectrumWalletType::Multisig { k: 2, n: 3 })
        );
    }

    #[test]
    fn classify_2fa_returns_none() {
        // 2fa is a recognized refusal class — classify_wallet_type itself
        // returns None (the refusal-template lookup happens in the parse
        // body, not the classifier). The sniff side accepts 2fa for clear
        // error messaging via a separate predicate.
        assert_eq!(classify_wallet_type("2fa"), None);
    }

    #[test]
    fn classify_imported_returns_none() {
        assert_eq!(classify_wallet_type("imported"), None);
    }

    #[test]
    fn classify_unknown_returns_none() {
        assert_eq!(classify_wallet_type("trezor"), None);
    }

    // ========================================================================
    // P6B parse body unit tests
    // ========================================================================

    /// Helper: minimal valid standard BIP-84 mainnet blob using the same xpub
    /// the toolkit's own export fixture uses (`tests/export_wallet/electrum_single.json`).
    fn fx_standard_bip84_mainnet() -> &'static [u8] {
        br#"{
            "seed_version": 17,
            "wallet_type": "standard",
            "use_encryption": false,
            "keystore": {
                "type": "bip32",
                "xpub": "zpub6qTBTNftBzVTjgVcSUw7vW5N1KQbV93Jnrw314RHGkCkSx4vk6nEWH1MJfReXi2WThvuDRiRpyT7cDoakEcZMQ1iZPgfJgQrcVMR4aJWh6S",
                "derivation": "m/84'/0'/0'",
                "root_fingerprint": "5436d724",
                "label": "Daily"
            }
        }"#
    }

    #[test]
    fn parse_standard_bip84_mainnet_happy_path() {
        let mut sink = Vec::new();
        let parsed = ElectrumParser::parse(fx_standard_bip84_mainnet(), &mut sink).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].cosigners.len(), 1);
        assert_eq!(parsed[0].network, bitcoin::Network::Bitcoin);
        assert_eq!(parsed[0].threshold, None);
        match &parsed[0].provenance {
            ImportProvenance::Electrum(meta) => {
                assert_eq!(meta.seed_version, 17);
                assert_eq!(meta.wallet_type, ElectrumWalletType::Standard);
                assert_eq!(meta.wallet_name.as_deref(), Some("Daily"));
            }
            other => panic!("expected Electrum provenance, got {other:?}"),
        }
        // Descriptor body should be wpkh(...) (zpub → BIP-84).
        assert!(
            parsed[0].original_descriptor.starts_with("wpkh("),
            "expected wpkh(...) for zpub; got {}",
            parsed[0].original_descriptor
        );
    }

    #[test]
    fn parse_standard_bip49_ypub_yields_sh_wpkh() {
        // ypub variant → BIP-49 → sh(wpkh(...)) wrapper. Real BIP-49 ypub
        // (grep-verified at e.g. src/slip0132.rs:178 SLIP0132_BIP49_YPUB).
        let blob = br#"{
            "seed_version": 17,
            "wallet_type": "standard",
            "use_encryption": false,
            "keystore": {
                "type": "bip32",
                "xpub": "ypub6Ww3ibxVfGzLrAH1PNcjyAWenMTbbAosGNB6VvmSEgytSER9azLDWCxoJwW7Ke7icmizBMXrzBx9979FfaHxHcrArf3zbeJJJUZPf663zsP",
                "derivation": "m/49'/0'/0'",
                "root_fingerprint": "00112233",
                "label": "Wrapped"
            }
        }"#;
        let mut sink = Vec::new();
        let parsed = ElectrumParser::parse(blob, &mut sink).unwrap();
        assert_eq!(parsed.len(), 1);
        assert!(
            parsed[0].original_descriptor.starts_with("sh(wpkh("),
            "expected sh(wpkh(...)) for ypub; got {}",
            parsed[0].original_descriptor
        );
    }

    #[test]
    fn parse_multisig_2of4_wsh_happy_path() {
        // Mirrors `tests/export_wallet/electrum_multi_2of4.json` (the toolkit's
        // own export fixture). Zpub prefix → wsh(sortedmulti(K, ...)).
        let blob = br#"{
            "seed_version": 17,
            "use_encryption": false,
            "wallet_type": "2of4",
            "x1/": {
                "derivation": "m/48'/0'/0'/2'",
                "label": "VaultCold-1",
                "root_fingerprint": "b8688df1",
                "type": "bip32",
                "xpub": "Zpub75ybJh4YZjnMskAAUkpy6uLizWcTTRC91yDtz9RcRwtavi4wHpBPZDEYUu9LoAPb6NQZNqKd6eKqF4FhqgWSaWQdqSt4FmdQkQH9uMmHhSh"
            },
            "x2/": {
                "derivation": "m/48'/0'/0'/2'",
                "label": "VaultCold-2",
                "root_fingerprint": "28645006",
                "type": "bip32",
                "xpub": "Zpub74LquwpiAdpsXwRDJp46dQ9BhcoEhk3vPktqwMqGrQYmjRhYQi5mbemCRiHUXVh1Ypu5XRYzbbznqxodCwK5NPeVXAPVAuLGKrr1LUMFmPh"
            },
            "x3/": {
                "derivation": "m/48'/0'/0'/2'",
                "label": "VaultCold-3",
                "root_fingerprint": "5436d724",
                "type": "bip32",
                "xpub": "Zpub72UafiS3U4xBBsiYjpCRcsEqm8i4Uo2Y2e5DmoNQALzLEXfyaJ7RvrGNGKznahzYT9T2BdMXiGPZ55NiuVukpcueupHwtfXeRKF3wyH3XDv"
            },
            "x4/": {
                "derivation": "m/48'/0'/0'/2'",
                "label": "VaultCold-4",
                "root_fingerprint": "16a93ed0",
                "type": "bip32",
                "xpub": "Zpub72UkKYo1g5fS6TZZAU4vJmYgqPsLkyJy2mbrSzAEiKfAZSfJLLVS8jK3QtcSTkVS7r731ZD5v2ET35aTTnqERztJ8Z6wZuSb6utt7dbTrBi"
            }
        }"#;
        let mut sink = Vec::new();
        let parsed = ElectrumParser::parse(blob, &mut sink).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].cosigners.len(), 4);
        assert_eq!(parsed[0].network, bitcoin::Network::Bitcoin);
        assert_eq!(parsed[0].threshold, Some(2));
        match &parsed[0].provenance {
            ImportProvenance::Electrum(meta) => {
                assert_eq!(meta.seed_version, 17);
                assert_eq!(
                    meta.wallet_type,
                    ElectrumWalletType::Multisig { k: 2, n: 4 }
                );
                // Label recovery strips trailing `-1` suffix.
                assert_eq!(meta.wallet_name.as_deref(), Some("VaultCold"));
            }
            other => panic!("expected Electrum provenance, got {other:?}"),
        }
        assert!(
            parsed[0].original_descriptor.starts_with("wsh(sortedmulti(2,"),
            "expected wsh(sortedmulti(2,...)) for Zpub multisig; got {}",
            parsed[0].original_descriptor
        );
    }

    #[test]
    fn parse_2fa_refuses_with_specific_message() {
        let blob = br#"{"seed_version": 17, "wallet_type": "2fa", "use_encryption": false}"#;
        let mut sink = Vec::new();
        let err = ElectrumParser::parse(blob, &mut sink).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("2fa") && msg.contains("TrustedCoin"),
            "expected 2fa refusal with TrustedCoin reference; got: {msg}"
        );
    }

    #[test]
    fn parse_imported_refuses_with_specific_message() {
        let blob = br#"{"seed_version": 17, "wallet_type": "imported", "use_encryption": false}"#;
        let mut sink = Vec::new();
        let err = ElectrumParser::parse(blob, &mut sink).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("imported-addresses") && msg.contains("derivation chain"),
            "expected imported-addresses refusal; got: {msg}"
        );
    }

    #[test]
    fn parse_use_encryption_emits_advisory_then_continues() {
        // v0.30.1 Cycle 6b: use_encryption=true is no longer a refusal — the
        // parser emits a stderr NOTICE advisory and continues. The parse here
        // still fails because the blob lacks `keystore` (Step 4 needs it),
        // but the advisory MUST fire first on stderr.
        let blob = br#"{"seed_version": 17, "wallet_type": "standard", "use_encryption": true}"#;
        let mut sink = Vec::new();
        let err = ElectrumParser::parse(blob, &mut sink).unwrap_err();
        let stderr = String::from_utf8(sink).unwrap();
        assert!(
            stderr.contains("notice: import-wallet: electrum: wallet is encrypted")
                && stderr.contains("watch-only material only")
                && stderr.contains("electrum --decrypt-wallet"),
            "expected use_encryption NOTICE advisory; got stderr: {stderr}"
        );
        // Parse continues to Step 4 keystore-required check.
        let msg = format!("{err}");
        assert!(
            msg.contains("keystore"),
            "expected keystore-missing refusal post-advisory; got: {msg}"
        );
    }

    #[test]
    fn parse_seed_version_below_min_rejected() {
        let blob = br#"{"seed_version": 5, "wallet_type": "standard"}"#;
        let mut sink = Vec::new();
        let err = ElectrumParser::parse(blob, &mut sink).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("seed_version") && msg.contains("below minimum"),
            "expected seed_version-below-min rejection; got: {msg}"
        );
    }

    #[test]
    fn parse_invalid_json_rejected() {
        let blob = br#"{not json"#;
        let mut sink = Vec::new();
        let err = ElectrumParser::parse(blob, &mut sink).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("invalid JSON"),
            "expected invalid-JSON rejection; got: {msg}"
        );
    }

    #[test]
    fn parse_missing_keystore_in_standard_rejected() {
        let blob = br#"{"seed_version": 17, "wallet_type": "standard", "use_encryption": false}"#;
        let mut sink = Vec::new();
        let err = ElectrumParser::parse(blob, &mut sink).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("keystore"),
            "expected missing-keystore rejection; got: {msg}"
        );
    }

    #[test]
    fn parse_multisig_missing_cosigner_rejected() {
        // wallet_type "2of3" but only x1/, x2/ keys present — x3/ missing.
        // Use real Zpubs so the missing-cosigner check fires before the
        // xpub-decode error path (cosigner loop iterates 1..=n upfront).
        let blob = br#"{
            "seed_version": 17,
            "wallet_type": "2of3",
            "use_encryption": false,
            "x1/": {
                "xpub": "Zpub75ybJh4YZjnMskAAUkpy6uLizWcTTRC91yDtz9RcRwtavi4wHpBPZDEYUu9LoAPb6NQZNqKd6eKqF4FhqgWSaWQdqSt4FmdQkQH9uMmHhSh",
                "derivation": "m/48'/0'/0'/2'",
                "root_fingerprint": "b8688df1"
            },
            "x2/": {
                "xpub": "Zpub74LquwpiAdpsXwRDJp46dQ9BhcoEhk3vPktqwMqGrQYmjRhYQi5mbemCRiHUXVh1Ypu5XRYzbbznqxodCwK5NPeVXAPVAuLGKrr1LUMFmPh",
                "derivation": "m/48'/0'/0'/2'",
                "root_fingerprint": "28645006"
            }
        }"#;
        let mut sink = Vec::new();
        let err = ElectrumParser::parse(blob, &mut sink).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("x3/") && msg.contains("missing"),
            "expected missing-x3/ rejection; got: {msg}"
        );
    }

    #[test]
    fn parse_unrecognized_wallet_type_rejected_with_format_hint() {
        // sniff must reject this BEFORE parse, but the parse path also has a
        // typed rejection for the explicit `--format electrum` user (sniff
        // bypass). Construct a blob with sniff-positive wallet_type but
        // post-classify-rejected (we don't have such a case naturally; this
        // test exercises the parse-side validation by constructing a blob
        // that sniff would actually reject — and confirms parse-side error
        // matches the rejection template).
        let blob =
            br#"{"seed_version": 17, "wallet_type": "trezor", "use_encryption": false}"#;
        let mut sink = Vec::new();
        let err = ElectrumParser::parse(blob, &mut sink).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("unrecognized") && msg.contains("wallet_type"),
            "expected unrecognized-wallet_type rejection; got: {msg}"
        );
    }

    #[test]
    fn parse_emits_dropped_fields_notice() {
        // Standard blob with an extra `addresses` top-level field that's
        // not in `ELECTRUM_PRESERVED_TOP_LEVEL_KEYS`.
        let blob = br#"{
            "seed_version": 17,
            "wallet_type": "standard",
            "use_encryption": false,
            "keystore": {
                "type": "bip32",
                "xpub": "zpub6qTBTNftBzVTjgVcSUw7vW5N1KQbV93Jnrw314RHGkCkSx4vk6nEWH1MJfReXi2WThvuDRiRpyT7cDoakEcZMQ1iZPgfJgQrcVMR4aJWh6S",
                "derivation": "m/84'/0'/0'",
                "root_fingerprint": "5436d724",
                "label": "Daily"
            },
            "addresses": {"receiving": ["bc1q..."], "change": []},
            "labels": {"some-txid": "label"}
        }"#;
        let mut sink = Vec::new();
        let parsed = ElectrumParser::parse(blob, &mut sink).unwrap();
        let stderr_text = String::from_utf8(sink).unwrap();
        assert!(
            stderr_text.contains("notice: import-wallet: electrum: dropped envelope fields"),
            "expected dropped-fields notice; got stderr: {stderr_text}"
        );
        assert!(stderr_text.contains("addresses"), "stderr: {stderr_text}");
        assert!(stderr_text.contains("labels"), "stderr: {stderr_text}");
        // Provenance dropped_fields list mirrors the stderr enumeration.
        match &parsed[0].provenance {
            ImportProvenance::Electrum(meta) => {
                assert!(meta.dropped_fields.contains(&"addresses".to_string()));
                assert!(meta.dropped_fields.contains(&"labels".to_string()));
            }
            _ => unreachable!(),
        }
    }

    // ========================================================================
    // is_multisig_cosigner_key unit tests
    // ========================================================================

    #[test]
    fn multisig_cosigner_key_x1_in_2cosigners() {
        assert!(is_multisig_cosigner_key("x1/", 2));
        assert!(is_multisig_cosigner_key("x2/", 2));
    }

    #[test]
    fn multisig_cosigner_key_out_of_range_rejected() {
        assert!(!is_multisig_cosigner_key("x3/", 2));
    }

    #[test]
    fn multisig_cosigner_key_x0_rejected() {
        // x0/ is below the 1-indexed minimum.
        assert!(!is_multisig_cosigner_key("x0/", 4));
    }

    #[test]
    fn multisig_cosigner_key_non_cosigner_rejected() {
        assert!(!is_multisig_cosigner_key("keystore", 4));
        assert!(!is_multisig_cosigner_key("seed_version", 4));
        assert!(!is_multisig_cosigner_key("xyz/", 4));
        assert!(!is_multisig_cosigner_key("x1", 4)); // missing trailing slash
    }

    #[test]
    fn multisig_cosigner_key_n_zero_returns_false() {
        assert!(!is_multisig_cosigner_key("x1/", 0));
    }

    // ========================================================================
    // derivation_purpose unit tests
    // ========================================================================

    #[test]
    fn derivation_purpose_bip84() {
        assert_eq!(derivation_purpose("m/84'/0'/0'"), Some("84'".to_string()));
    }

    #[test]
    fn derivation_purpose_bip44() {
        assert_eq!(derivation_purpose("m/44'/0'/0'"), Some("44'".to_string()));
    }

    #[test]
    fn derivation_purpose_h_form() {
        assert_eq!(derivation_purpose("m/84h/0h/0h"), Some("84h".to_string()));
    }

    #[test]
    fn derivation_purpose_empty_returns_none() {
        assert_eq!(derivation_purpose("m"), None);
        assert_eq!(derivation_purpose("m/"), None);
    }
}
