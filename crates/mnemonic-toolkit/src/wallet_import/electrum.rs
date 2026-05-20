//! Electrum 4.x wallet-file ingest (`--format electrum`).
//!
//! Per `design/SPEC_wallet_import_v0_28_0.md` §11.6. Inverse of
//! `wallet_export/electrum.rs` (the export-side emitter). Parses Electrum's
//! Python-dict-serialized JSON wallet file. Accepts the post-upgrade
//! canonical shape only; pre-Electrum-4.x wallets must be opened in Electrum
//! 4.x first so the loader's `_convert_*` migration chain rewrites legacy
//! values (see §11.6 Electrum-version scoping note).
//!
//! ## Namespace disambiguation (SPEC §1.4)
//!
//! Three `electrum`-named modules coexist in the toolkit:
//!
//! - `crate::electrum` — Electrum native-seed-format codec (HMAC-SHA512
//!   prefix dispatch). Unrelated to wallet-file ingest. UNCHANGED in v0.28.0.
//! - `crate::wallet_export::electrum` — Electrum wallet-file EMIT. UNCHANGED.
//! - `crate::wallet_import::electrum` — THIS MODULE. Wallet-file INGEST
//!   (inverse of wallet_export::electrum). NEW in v0.28.0 P6.
//!
//! ## Sniff signature (P6A)
//!
//! Sniff matches when ALL of:
//! 1. Blob parses as JSON.
//! 2. Top-level value is an object.
//! 3. Object has a `seed_version` integer field in {11..=71} (per Electrum
//!    upstream's `NEW_SEED_VERSION=11` floor + `FINAL_SEED_VERSION=71` ceiling
//!    at `electrum/wallet_db.py`).
//! 4. Object has a `wallet_type` string field in the accepted enumeration
//!    (`"standard"`, `"<k>of<n>"` pattern via regex, `"2fa"`, `"imported"`).
//!
//! Sniff returns `true` for the refused variants (`"2fa"` / `"imported"` /
//! encrypted) so the parser arm is reached and the user-facing refusal
//! template fires; sniff is a routing decision, not an admission decision.
//!
//! ## Parse contract (P6B — body lands in Phase P6B)
//!
//! See `parse()` doc-comment for the per-`wallet_type` dispatch table.

use super::{
    pipeline::concrete_keys_to_placeholders, validate_watch_only_resolved, ImportProvenance,
    ParsedImport, WalletFormatParser,
};
use crate::error::ToolkitError;
use crate::parse_descriptor;
use crate::slip0132::normalize_xpub_prefix;
use crate::synthesize::{xpub_to_65, ResolvedSlot};
use bitcoin::bip32::{ChildNumber, DerivationPath, Fingerprint, Xpub};
use miniscript::descriptor::checksum::Engine as ChecksumEngine;
use serde_json::Value;
use std::io::Write;
use std::str::FromStr;
use std::sync::OnceLock;

/// SPEC §11.6 — Electrum wallet-file parser.
pub(crate) struct ElectrumParser;

/// SPEC §11.6 — per-format provenance for Electrum-ingested bundles. Holds
/// non-bundle metadata (seed_version, wallet_type, wallet_name,
/// dropped_fields) preserved for the `--json` envelope's `source_metadata`
/// surface (parallel to v0.26.0's `CoreSourceMetadata`).
///
/// Wired into `ImportProvenance::Electrum(_)` at v0.28.0 Phase P6C;
/// consumed by `cmd::import_wallet::emit_json_envelope`'s
/// `source_metadata` block for envelope downstream consumers.
#[derive(Debug, Clone)]
pub(crate) struct ElectrumSourceMetadata {
    /// `seed_version` integer from the top-level Electrum wallet object.
    /// Range {11..=71} (Electrum 4.x current; per FOLLOWUP
    /// `electrum-final-seed-version-drift`, the upper bound drifts upstream).
    /// Widened from `u8` (P0A SPEC draft) to `u32` for consistency with
    /// `wallet_export/electrum.rs::ELECTRUM_SEED_VERSION_PIN: u32` and to
    /// absorb future upstream drift past 255 without a field-type churn.
    pub(crate) seed_version: u32,
    /// Parsed `wallet_type` discriminator. See enum doc.
    pub(crate) wallet_type: ElectrumWalletType,
    /// Optional human-readable wallet name. Electrum's wallet file does NOT
    /// carry a top-level `wallet_name` field (the value is implicit in the
    /// on-disk filename); this slot is reserved for future Electrum-version
    /// changes. P6 leaves it `None`.
    pub(crate) wallet_name: Option<String>,
    /// Electrum wallet-state field names present in the source that were
    /// dropped from the bundle output (e.g., `addr_history`, `addresses`,
    /// `channels`, `transactions` — runtime state not reconstructable from
    /// the key-material alone). Drives a single stderr NOTICE per SPEC §2.4
    /// (analogous to `CoreSourceMetadata::dropped_fields`).
    pub(crate) dropped_fields: Vec<String>,
}

/// SPEC §11.6 — accepted `wallet_type` value-shape discriminator.
///
/// The literal string `"multisig"` is NEVER stored as `wallet_type` by
/// Electrum 4.x (the `_convert_wallet_type` upgrade chain rewrites legacy
/// values to either `"standard"` or to the `<k>of<n>` pattern, validated by
/// `multisig_type()` at `electrum/util.py` via regex `r'(\d+)of(\d+)'`).
/// `ElectrumWalletType::Multisig` carries the parsed `(k, n)` directly to
/// faithfully mirror what's on disk.
///
/// Refused wallet-type variants (`"2fa"` / `"imported"`) do not reach this
/// enum — they error out at parse-time via SPEC §11.6.1 refusal templates
/// before a provenance is constructed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ElectrumWalletType {
    /// `wallet_type: "standard"` — singlesig wallet (single `keystore` field
    /// at top level). Covers post-upgrade `"old"`, `"xpub"`, `"bip44"`, and
    /// hardware-wallet legacy values that map to `"standard"` per
    /// `_convert_wallet_type`.
    Standard,
    /// `wallet_type: "<k>of<n>"` — multisig wallet (per-cosigner `x1/`,
    /// `x2/`, ... fields at top level). `k` ≥ 1, `n` ≤ 15 per Electrum's
    /// `multisig_type` regex; not bounds-validated here (caller may sanity-
    /// check separately at parse time).
    Multisig { k: u8, n: u8 },
}

impl WalletFormatParser for ElectrumParser {
    /// SPEC §11.6 sniff signature. Returns `true` when the blob looks like
    /// an Electrum 4.x wallet file (post-upgrade canonical shape). Does NOT
    /// discriminate between the parse-able (`standard` / `<k>of<n>`) and
    /// refused (`2fa` / `imported`) sub-shapes — that's a parse-arm
    /// distinction. Encrypted wallets are also sniff-positive (the
    /// `use_encryption: true` top-level field still lives alongside
    /// `seed_version` / `wallet_type`); the parse arm refuses them.
    fn sniff(blob: &[u8]) -> bool {
        let value: Value = match serde_json::from_slice(blob) {
            Ok(v) => v,
            Err(_) => return false,
        };
        let obj = match value.as_object() {
            Some(o) => o,
            None => return false,
        };

        // seed_version: integer in {11..=71}.
        let sv_ok = obj
            .get("seed_version")
            .and_then(Value::as_u64)
            .map(|n| (11..=71).contains(&n))
            .unwrap_or(false);
        if !sv_ok {
            return false;
        }

        // wallet_type: string in accepted set.
        let wt = match obj.get("wallet_type").and_then(Value::as_str) {
            Some(s) => s,
            None => return false,
        };
        match wt {
            "standard" | "2fa" | "imported" => true,
            other => multisig_type_regex().is_match(other),
        }
    }

    /// SPEC §11.6 parse contract — per-`wallet_type` dispatch:
    ///
    /// - `"standard"` → singlesig parse: extract `keystore.{xpub, derivation,
    ///   root_fingerprint}`, normalize SLIP-132 xpub, infer wrapper from
    ///   `(slip132_variant, derivation_purpose)`, synthesize concrete-keys
    ///   descriptor, run through the toolkit's `concrete_keys_to_placeholders`
    ///   + `parse_descriptor` pipeline.
    /// - `"<k>of<n>"` → multisig parse: iterate `x1/`..`xN/` per-cosigner
    ///   sub-objects; wrapper is `wsh(sortedmulti)` (Zpub) or
    ///   `sh(wsh(sortedmulti))` (Ypub) or `sh(sortedmulti)` (xpub legacy);
    ///   K and N parsed from the regex.
    /// - `"2fa"` / `"imported"` → REFUSE with SPEC §11.6.1 template.
    /// - `use_encryption: true` → REFUSE with SPEC §11.6.1 encrypted template.
    fn parse(blob: &[u8], stderr: &mut dyn Write) -> Result<Vec<ParsedImport>, ToolkitError> {
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

        // §11.6.1 — encrypted refusal precedes type-discrimination because
        // sensitive keystore fields are unreadable; we cannot extract xpub
        // from a base64-encrypted keystore blob.
        if obj.get("use_encryption").and_then(Value::as_bool) == Some(true) {
            return Err(ToolkitError::ImportWalletParse(
                "import-wallet: electrum: encrypted wallet files require decrypting via \
                 'electrum --decrypt-wallet' first; encrypted ingest not yet supported \
                 (FOLLOWUP wallet-import-electrum-encrypted)"
                    .to_string(),
            ));
        }

        let seed_version = obj
            .get("seed_version")
            .and_then(Value::as_u64)
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(
                    "import-wallet: electrum: parse error: missing or non-integer \
                     `seed_version` field"
                        .to_string(),
                )
            })?;
        if !(11..=71).contains(&seed_version) {
            return Err(ToolkitError::ImportWalletParse(format!(
                "import-wallet: electrum: parse error: seed_version {seed_version} outside \
                 supported range {{11..=71}} (Electrum 4.x post-upgrade canonical range)"
            )));
        }
        // Cast: range-checked above, fits in u32.
        let seed_version: u32 = seed_version as u32;

        let wallet_type = obj
            .get("wallet_type")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(
                    "import-wallet: electrum: parse error: missing or non-string \
                     `wallet_type` field"
                        .to_string(),
                )
            })?;

        // §11.6.1 — 2fa / imported refusal templates.
        match wallet_type {
            "2fa" => {
                return Err(ToolkitError::ImportWalletParse(
                    "import-wallet: electrum: 2fa wallets require TrustedCoin two-factor \
                     restoration; ingest not supported"
                        .to_string(),
                ))
            }
            "imported" => {
                return Err(ToolkitError::ImportWalletParse(
                    "import-wallet: electrum: imported-addresses wallets have no derivation \
                     chain to reconstruct; ingest not supported"
                        .to_string(),
                ))
            }
            _ => {}
        }

        let wt_enum: ElectrumWalletType = if wallet_type == "standard" {
            ElectrumWalletType::Standard
        } else if let Some((k, n)) = parse_multisig_wallet_type(wallet_type) {
            // SPEC §11.6: bounds-validate per Electrum's `multisig_type` upstream
            // (k >= 1, n <= 15, k <= n).
            if k < 1 || n < 2 || n > 15 || k > n {
                return Err(ToolkitError::ImportWalletParse(format!(
                    "import-wallet: electrum: parse error: wallet_type `{wallet_type}` has \
                     out-of-bounds (k, n) = ({k}, {n}); require 1 <= k <= n and 2 <= n <= 15"
                )));
            }
            ElectrumWalletType::Multisig { k, n }
        } else {
            return Err(ToolkitError::ImportWalletParse(format!(
                "import-wallet: electrum: parse error: unrecognized wallet_type `{wallet_type}`; \
                 accepted: \"standard\", \"<k>of<n>\" pattern, \"2fa\", \"imported\""
            )));
        };

        // SPEC §11.6 step 3 — dropped wallet-state fields. Electrum's wallet
        // file carries runtime state (address history, transactions, channel
        // state) that is not reconstructable from key material alone. Surface
        // a single stderr NOTICE listing what was present (per SPEC §2.4,
        // analogous to bitcoin-core's `aggregate_dropped` block).
        let dropped_fields = collect_dropped_fields(obj);
        if !dropped_fields.is_empty() {
            writeln!(
                stderr,
                "notice: import-wallet: electrum: dropped wallet-state fields {}: not preserved \
                 in bundle output (key-state only)",
                dropped_fields.join(", ")
            )
            .map_err(ToolkitError::Io)?;
        }

        // Per-wallet_type parse-arm. Both arms produce a descriptor body
        // bearing concrete `[fp/path]xpub` keys; the rest of the pipeline
        // (placeholder substitution + parse_descriptor) is shared.
        let (descriptor_body, network, threshold, original_descriptor) = match wt_enum {
            ElectrumWalletType::Standard => parse_standard(obj)?,
            ElectrumWalletType::Multisig { k, n } => parse_multisig(obj, k, n)?,
        };

        let descriptor_body_no_csum =
            miniscript::descriptor::checksum::verify_checksum(&descriptor_body).map_err(|e| {
                ToolkitError::ImportWalletParse(format!(
                    "import-wallet: electrum: parse error: synthesized BIP-380 checksum \
                     validation failed (internal bug; descriptor: {descriptor_body}): {e}"
                ))
            })?;

        let (placeholder_form, parsed_keys, parsed_fingerprints) =
            concrete_keys_to_placeholders(descriptor_body_no_csum).map_err(|e| {
                ToolkitError::ImportWalletParse(e.message().replacen(
                    "import-wallet: bsms:",
                    "import-wallet: electrum:",
                    1,
                ))
            })?;

        let descriptor =
            parse_descriptor::parse_descriptor(&placeholder_form, &parsed_keys, &parsed_fingerprints)
                .map_err(|e| {
                    ToolkitError::ImportWalletParse(format!(
                        "import-wallet: electrum: parse error: {}",
                        e.message()
                    ))
                })?;

        // Per-cosigner ResolvedSlot vector. Walk the synthesized descriptor's
        // origin annotations (which we just emitted from the parsed Electrum
        // fields) and pair each with the parsed xpub.
        let origins = extract_origin_components(descriptor_body_no_csum)?;
        let mut cosigners: Vec<ResolvedSlot> = Vec::with_capacity(parsed_keys.len());
        for (slot_idx, _) in parsed_keys.iter().enumerate() {
            let (fp, path, path_raw, xpub_str) = origins.get(slot_idx).cloned().ok_or_else(|| {
                ToolkitError::ImportWalletParse(format!(
                    "import-wallet: electrum: parse error: slot {slot_idx} out of range in \
                     synthesized descriptor origins (internal bug)"
                ))
            })?;
            let (neutral, _variant) = normalize_xpub_prefix(&xpub_str)?;
            let xpub = Xpub::from_str(&neutral).map_err(|e| {
                ToolkitError::ImportWalletParse(format!(
                    "import-wallet: electrum: parse error: xpub decode for slot {slot_idx}: {e}"
                ))
            })?;
            debug_assert_eq!(xpub_to_65(&xpub), parsed_keys[slot_idx].payload);
            cosigners.push(ResolvedSlot {
                xpub,
                fingerprint: fp,
                path,
                path_raw,
                entropy: None,
                master_xpub: None,
                _entropy_pin: None,
            });
        }

        validate_watch_only_resolved(&cosigners)?;

        // v0.28.0 P6C: ImportProvenance::Electrum(ElectrumSourceMetadata) is
        // now wired (was Bsms(None) placeholder under P6B). Per SPEC §11.6
        // Provenance section; populated from the per-blob fields captured
        // above (seed_version, wt_enum, dropped_fields, wallet_name=None
        // since Electrum's wallet file does not carry a top-level
        // wallet_name field).
        let provenance = ImportProvenance::Electrum(ElectrumSourceMetadata {
            seed_version,
            wallet_type: wt_enum,
            wallet_name: None,
            dropped_fields,
        });

        Ok(vec![ParsedImport {
            descriptor,
            original_descriptor,
            cosigners,
            network,
            threshold,
            provenance,
        }])
    }
}

// ============================================================================
// Per-wallet_type parse arms
// ============================================================================

/// SPEC §11.6 — singlesig parse. Returns `(descriptor_body_with_checksum,
/// network, threshold=None, original_descriptor)`.
fn parse_standard(
    obj: &serde_json::Map<String, Value>,
) -> Result<(String, bitcoin::Network, Option<u8>, String), ToolkitError> {
    let keystore = obj.get("keystore").and_then(Value::as_object).ok_or_else(|| {
        ToolkitError::ImportWalletParse(
            "import-wallet: electrum: parse error: missing or non-object `keystore` field for \
             wallet_type=standard"
                .to_string(),
        )
    })?;

    let xpub_str = keystore
        .get("xpub")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            ToolkitError::ImportWalletParse(
                "import-wallet: electrum: parse error: missing or non-string `keystore.xpub`"
                    .to_string(),
            )
        })?;

    let derivation = keystore
        .get("derivation")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            ToolkitError::ImportWalletParse(
                "import-wallet: electrum: parse error: missing or non-string \
                 `keystore.derivation`"
                    .to_string(),
            )
        })?;

    let root_fingerprint = keystore
        .get("root_fingerprint")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            ToolkitError::ImportWalletParse(
                "import-wallet: electrum: parse error: missing or non-string \
                 `keystore.root_fingerprint`"
                    .to_string(),
            )
        })?;

    // SLIP-132 variant + derivation purpose drive the wrapper choice.
    let (neutral_xpub, variant) = normalize_xpub_prefix(xpub_str)?;
    let xpub = Xpub::from_str(&neutral_xpub).map_err(|e| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: electrum: parse error: keystore.xpub decode failed: {e}"
        ))
    })?;

    let path = parse_derivation_path(derivation)?;
    let purpose = derivation_purpose(&path)?;
    let network = network_from_derivation(&path)?;
    let wrapper = singlesig_wrapper_from_variant_and_purpose(variant, purpose)?;

    let fp_hex = normalize_fp_hex(root_fingerprint)?;
    let path_no_m = derivation.trim_start_matches('m').trim_start_matches('/');
    // Inner is `[fp/path]xpub/<0;1>/*` (the multipath/ranged tail mirrors
    // Electrum's auto-derivation of `/0/*` receive + `/1/*` change).
    let inner = format!(
        "[{fp_hex}/{path_no_m}]{xpub}/<0;1>/*",
        fp_hex = fp_hex,
        path_no_m = path_no_m,
        xpub = xpub, // re-rendered neutral form via Display
    );
    let body = wrap_singlesig(wrapper, &inner);
    let checksum = render_checksum(&body)?;
    let original = format!("{body}#{checksum}");
    Ok((original.clone(), network, None, original))
}

/// SPEC §11.6 — multisig parse. Iterates `x1/`..`xN/` cosigner sub-objects;
/// returns `(descriptor_body_with_checksum, network, threshold=Some(K),
/// original_descriptor)`.
fn parse_multisig(
    obj: &serde_json::Map<String, Value>,
    k: u8,
    n: u8,
) -> Result<(String, bitcoin::Network, Option<u8>, String), ToolkitError> {
    let mut cosigners: Vec<(Fingerprint, DerivationPath, String, &'static str)> =
        Vec::with_capacity(n as usize);

    // Per-cosigner key is "x1/", "x2/", ... per Electrum's wallet_db.py
    // (note the trailing slash; per the upstream code "version 55 removes
    // trailing /" — pre-v55 wallets carry the slash, current Electrum
    // 4.x writes still emit it as of FINAL_SEED_VERSION=71 per the toolkit's
    // own electrum_multi_2of4.json fixture). We support both shapes
    // (with and without slash) defensively.
    for i in 0..n {
        let key_with_slash = format!("x{}/", i + 1);
        let key_no_slash = format!("x{}", i + 1);
        let ks = obj
            .get(&key_with_slash)
            .or_else(|| obj.get(&key_no_slash))
            .and_then(Value::as_object)
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(format!(
                    "import-wallet: electrum: parse error: missing or non-object cosigner key \
                     `{key_with_slash}` (also tried `{key_no_slash}`)"
                ))
            })?;

        let xpub_str = ks
            .get("xpub")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(format!(
                    "import-wallet: electrum: parse error: missing or non-string \
                     `{key_with_slash}.xpub`"
                ))
            })?;
        let derivation = ks
            .get("derivation")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(format!(
                    "import-wallet: electrum: parse error: missing or non-string \
                     `{key_with_slash}.derivation`"
                ))
            })?;
        let root_fingerprint = ks
            .get("root_fingerprint")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(format!(
                    "import-wallet: electrum: parse error: missing or non-string \
                     `{key_with_slash}.root_fingerprint`"
                ))
            })?;

        let (neutral_xpub, variant) = normalize_xpub_prefix(xpub_str)?;
        let _xpub = Xpub::from_str(&neutral_xpub).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: electrum: parse error: `{key_with_slash}.xpub` decode failed: {e}"
            ))
        })?;

        // Per-cosigner variant must agree across all cosigners (Electrum
        // emits a uniform SLIP-132 variant per multisig wallet — all Zpub
        // or all Ypub). Heterogeneous variants → parse error.
        let path = parse_derivation_path(derivation)?;
        let fp_hex = normalize_fp_hex(root_fingerprint)?;
        let fp = Fingerprint::from_str(&fp_hex).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: electrum: parse error: `{key_with_slash}.root_fingerprint` \
                 decode failed: {e}"
            ))
        })?;
        cosigners.push((
            fp,
            path,
            neutral_xpub,
            variant.unwrap_or("xpub"),
        ));
    }

    // Cosigner-variant agreement.
    let first_variant = cosigners[0].3;
    for (i, c) in cosigners.iter().enumerate().skip(1) {
        if c.3 != first_variant {
            return Err(ToolkitError::ImportWalletParse(format!(
                "import-wallet: electrum: parse error: cosigner x{slot}/ uses SLIP-132 variant \
                 `{ovariant}`, x1/ uses `{first_variant}`; all cosigners must share a variant",
                slot = i + 1,
                ovariant = c.3,
            )));
        }
    }

    // Cosigner-network agreement.
    let networks: Vec<bitcoin::Network> = cosigners
        .iter()
        .map(|(_, p, _, _)| network_from_derivation(p))
        .collect::<Result<Vec<_>, _>>()?;
    let first_network = networks[0];
    for (i, nw) in networks.iter().enumerate().skip(1) {
        if *nw != first_network {
            return Err(ToolkitError::ImportWalletParse(format!(
                "import-wallet: electrum: parse error: cosigner x{slot}/ has network {nw:?}, \
                 x1/ has {first_network:?}; all cosigners must share a network",
                slot = i + 1,
            )));
        }
    }

    let wrapper = multisig_wrapper_from_variant(first_variant)?;
    let inner: Vec<String> = cosigners
        .iter()
        .map(|(fp, path, xpub_str, _)| {
            format!(
                "[{fp}/{path}]{xpub_str}/<0;1>/*",
                fp = format_fp(fp),
                path = render_path_no_m(path),
                xpub_str = xpub_str,
            )
        })
        .collect();
    let body = match wrapper {
        MultisigWrapper::Wsh => format!("wsh(sortedmulti({k},{}))", inner.join(",")),
        MultisigWrapper::ShWsh => format!("sh(wsh(sortedmulti({k},{})))", inner.join(",")),
        MultisigWrapper::Sh => format!("sh(sortedmulti({k},{}))", inner.join(",")),
    };
    let checksum = render_checksum(&body)?;
    let original = format!("{body}#{checksum}");
    Ok((original.clone(), first_network, Some(k), original))
}

// ============================================================================
// Inference helpers (variant + derivation → wrapper)
// ============================================================================

/// Parse Electrum's `m/<purpose>'/...` derivation string into a typed
/// `DerivationPath`.
fn parse_derivation_path(s: &str) -> Result<DerivationPath, ToolkitError> {
    DerivationPath::from_str(s).map_err(|e| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: electrum: parse error: derivation `{s}` parse failed: {e}"
        ))
    })
}

/// BIP-43 purpose (first hardened component) from a derivation path. Must
/// be hardened.
fn derivation_purpose(p: &DerivationPath) -> Result<u32, ToolkitError> {
    let mut iter = p.into_iter();
    let comp = iter.next().ok_or_else(|| {
        ToolkitError::ImportWalletParse(
            "import-wallet: electrum: parse error: derivation path is empty".to_string(),
        )
    })?;
    match comp {
        ChildNumber::Hardened { index } => Ok(*index),
        ChildNumber::Normal { .. } => Err(ToolkitError::ImportWalletParse(
            "import-wallet: electrum: parse error: derivation purpose component must be \
             hardened (e.g., 84')"
                .to_string(),
        )),
    }
}

/// BIP-48 coin-type (second hardened component). 0' → mainnet, 1' → testnet.
fn network_from_derivation(p: &DerivationPath) -> Result<bitcoin::Network, ToolkitError> {
    let comps: Vec<&ChildNumber> = p.into_iter().collect();
    if comps.len() < 2 {
        return Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: electrum: parse error: derivation path has only {} components; \
             need >=2 for coin-type inference",
            comps.len()
        )));
    }
    match comps[1] {
        ChildNumber::Hardened { index: 0 } => Ok(bitcoin::Network::Bitcoin),
        ChildNumber::Hardened { index: 1 } => Ok(bitcoin::Network::Testnet),
        ChildNumber::Hardened { index } => Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: electrum: parse error: unsupported coin-type {index}; only 0 \
             (mainnet) and 1 (testnet) supported"
        ))),
        ChildNumber::Normal { .. } => Err(ToolkitError::ImportWalletParse(
            "import-wallet: electrum: parse error: coin-type component must be hardened"
                .to_string(),
        )),
    }
}

/// SPEC §11.6 — singlesig wrapper choice from `(SLIP-132 variant, purpose)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SinglesigWrapper {
    /// `pkh(<inner>)` — BIP-44.
    Pkh,
    /// `wpkh(<inner>)` — BIP-84 (or BIP-84 neutral-form hand-edit).
    Wpkh,
    /// `sh(wpkh(<inner>))` — BIP-49.
    ShWpkh,
    /// `tr(<inner>)` — BIP-86.
    Tr,
}

/// SPEC §11.6 — singlesig wrapper from `(SLIP-132 variant, purpose)`.
///
/// Variant table (per `wallet_export/electrum.rs::render_slip132_xpub`):
/// - xpub + purpose=44 → pkh
/// - xpub + purpose=86 → tr
/// - xpub + purpose=84 → wpkh (rare; Electrum emits zpub here, but tolerate
///   neutral form for hand-edited wallets)
/// - xpub + purpose=49 → sh(wpkh) (rare; Electrum emits ypub)
/// - ypub / upub → sh(wpkh)
/// - zpub / vpub → wpkh
fn singlesig_wrapper_from_variant_and_purpose(
    variant: Option<&'static str>,
    purpose: u32,
) -> Result<SinglesigWrapper, ToolkitError> {
    match (variant, purpose) {
        (Some("zpub"), _) | (Some("vpub"), _) => Ok(SinglesigWrapper::Wpkh),
        (Some("ypub"), _) | (Some("upub"), _) => Ok(SinglesigWrapper::ShWpkh),
        (None, 44) => Ok(SinglesigWrapper::Pkh),
        (None, 86) => Ok(SinglesigWrapper::Tr),
        (None, 84) => Ok(SinglesigWrapper::Wpkh),
        (None, 49) => Ok(SinglesigWrapper::ShWpkh),
        (None, other) => Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: electrum: parse error: cannot infer wrapper for neutral xpub at \
             purpose {other}'; supported singlesig purposes for neutral xpub are \
             44 (pkh), 49 (sh(wpkh)), 84 (wpkh), 86 (tr)"
        ))),
        (Some(other), _) => Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: electrum: parse error: unexpected SLIP-132 variant `{other}` on \
             singlesig keystore.xpub; expected xpub / ypub / zpub for mainnet or \
             tpub / upub / vpub for testnet"
        ))),
    }
}

fn wrap_singlesig(w: SinglesigWrapper, inner: &str) -> String {
    match w {
        SinglesigWrapper::Pkh => format!("pkh({inner})"),
        SinglesigWrapper::Wpkh => format!("wpkh({inner})"),
        SinglesigWrapper::ShWpkh => format!("sh(wpkh({inner}))"),
        SinglesigWrapper::Tr => format!("tr({inner})"),
    }
}

/// SPEC §11.6 — multisig wrapper choice from cosigner SLIP-132 variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MultisigWrapper {
    /// `wsh(sortedmulti(K, ...))` — Zpub cosigners (or Vpub testnet).
    Wsh,
    /// `sh(wsh(sortedmulti(K, ...)))` — Ypub cosigners (or Upub testnet).
    ShWsh,
    /// `sh(sortedmulti(K, ...))` — neutral xpub cosigners (legacy BIP-45 /
    /// hand-edited wallets). Not common but tolerated.
    Sh,
}

fn multisig_wrapper_from_variant(variant: &str) -> Result<MultisigWrapper, ToolkitError> {
    match variant {
        "Zpub" | "Vpub" => Ok(MultisigWrapper::Wsh),
        "Ypub" | "Upub" => Ok(MultisigWrapper::ShWsh),
        "xpub" | "tpub" => Ok(MultisigWrapper::Sh),
        other => Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: electrum: parse error: unsupported multisig SLIP-132 variant \
             `{other}`; expected Zpub / Ypub / Vpub / Upub or neutral xpub / tpub"
        ))),
    }
}

/// Normalize a `root_fingerprint` string (Electrum stores it lowercase,
/// 8 hex chars). Accepts upper/lower case, rejects non-hex / wrong length.
fn normalize_fp_hex(s: &str) -> Result<String, ToolkitError> {
    if s.len() != 8 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: electrum: parse error: root_fingerprint `{s}` must be 8 lowercase \
             hex characters"
        )));
    }
    Ok(s.to_ascii_lowercase())
}

fn format_fp(fp: &Fingerprint) -> String {
    fp.to_string().to_ascii_lowercase()
}

/// Render a `DerivationPath` as `84'/0'/0'` (no leading `m/`). Hardened
/// components use `'`.
fn render_path_no_m(p: &DerivationPath) -> String {
    let s = p.to_string();
    // bitcoin::bip32::DerivationPath::Display emits `m/84'/0'/0'`.
    s.trim_start_matches('m').trim_start_matches('/').to_string()
}

/// Re-emit a BIP-380 checksum for a descriptor body via miniscript's
/// `ChecksumEngine`. Used internally to synthesize a valid `#checksum`
/// trailer on the parsed-from-Electrum descriptor body.
fn render_checksum(body: &str) -> Result<String, ToolkitError> {
    let mut e = ChecksumEngine::new();
    e.input(body).map_err(|err| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: electrum: parse error: checksum input rejected: {err}"
        ))
    })?;
    Ok(e.checksum())
}

/// Per-cosigner origin tuple lifted out of the descriptor body via the
/// shared `[fp/path]xpub` regex pattern. Returned in declaration order.
/// Mirrors `bsms::extract_origin_components` semantics with electrum error
/// prefix.
fn extract_origin_components(
    descriptor_body: &str,
) -> Result<Vec<(Fingerprint, DerivationPath, String, String)>, ToolkitError> {
    let re = origin_capture_regex();
    let mut out = Vec::new();
    for cap in re.captures_iter(descriptor_body) {
        let fp_hex = cap.get(1).expect("group 1").as_str();
        let path_raw_inner = cap.get(2).expect("group 2").as_str();
        let xpub_str = cap.get(3).expect("group 3").as_str();

        let fp = Fingerprint::from_str(fp_hex).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: electrum: parse error: fingerprint hex: {e}"
            ))
        })?;
        let path = DerivationPath::from_str(&format!("m{path_raw_inner}")).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: electrum: parse error: derivation-path parse: {e}"
            ))
        })?;
        let path_raw = format!("[{fp_hex}{path_raw_inner}]");
        out.push((fp, path, path_raw, xpub_str.to_string()));
    }
    Ok(out)
}

fn origin_capture_regex() -> &'static regex::Regex {
    static R: OnceLock<regex::Regex> = OnceLock::new();
    R.get_or_init(|| {
        regex::Regex::new(
            r"\[([0-9a-fA-F]{8})((?:/\d+'?)+)\]([xtyzuvYZUV]pub[A-HJ-NP-Za-km-z1-9]+)",
        )
        .expect("origin_capture_regex is a fixed string literal")
    })
}

/// Collect Electrum wallet-state field names present at top level that are
/// not preserved in the bundle output. Drives the §2.4 stderr NOTICE.
/// The discriminator set covers the runtime-state fields Electrum 4.x
/// commonly writes (per `electrum/wallet_db.py`'s default keys).
fn collect_dropped_fields(obj: &serde_json::Map<String, Value>) -> Vec<String> {
    const STATE_FIELDS: &[&str] = &[
        "addr_history",
        "addresses",
        "channels",
        "channel_backups",
        "fiat_value",
        "labels",
        "spent_outpoints",
        "stored_height",
        "transactions",
        "tx_fees",
        "txi",
        "txo",
        "verified_tx3",
    ];
    let mut out: Vec<String> = Vec::new();
    for f in STATE_FIELDS {
        if obj.contains_key(*f) {
            out.push((*f).to_string());
        }
    }
    out
}

/// SPEC §11.6 — `wallet_type` multisig pattern regex. Mirrors Electrum's
/// `multisig_type()` at `electrum/util.py`: `r'(\d+)of(\d+)'`. Anchored
/// at both ends (the upstream regex is unanchored via `re.match` which
/// anchors leading-only; we anchor trailing too because Electrum's writer
/// only ever stores the bare pattern with no suffix).
fn multisig_type_regex() -> &'static regex::Regex {
    static R: OnceLock<regex::Regex> = OnceLock::new();
    R.get_or_init(|| regex::Regex::new(r"^(\d+)of(\d+)$").expect("static regex compiles"))
}

/// Parse a `<k>of<n>` `wallet_type` value into `(k, n)`. Returns `None` for
/// non-matching inputs (caller should check via `sniff` or fall through to
/// the SPEC §11.6.1 refusal-template arms first).
#[allow(dead_code)] // Consumed by Phase P6B parse-arm.
pub(crate) fn parse_multisig_wallet_type(s: &str) -> Option<(u8, u8)> {
    let caps = multisig_type_regex().captures(s)?;
    let k = caps.get(1)?.as_str().parse::<u8>().ok()?;
    let n = caps.get(2)?.as_str().parse::<u8>().ok()?;
    Some((k, n))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal `standard` wallet shape — mirrors the toolkit's own
    /// `wallet_export/electrum.rs` emit + matches Electrum 4.x on-disk.
    const STANDARD_BIP84: &str = r#"{
  "seed_version": 17,
  "wallet_type": "standard",
  "use_encryption": false,
  "keystore": {
    "type": "bip32",
    "xpub": "zpub6qTBTNftBzVTjgVcSUw7vW5N1KQbV93Jnrw314RHGkCkSx4vk6nEWH1MJfReXi2WThvuDRiRpyT7cDoakEcZMQ1iZPgfJgQrcVMR4aJWh6S",
    "derivation": "m/84'/0'/0'",
    "root_fingerprint": "5436d724",
    "label": ""
  }
}
"#;

    const MULTISIG_2OF3: &str = r#"{
  "seed_version": 17,
  "use_encryption": false,
  "wallet_type": "2of3",
  "x1/": {"type":"bip32","xpub":"Zpub75ybJh4YZjnMskAAUkpy6uLizWcTTRC91yDtz9RcRwtavi4wHpBPZDEYUu9LoAPb6NQZNqKd6eKqF4FhqgWSaWQdqSt4FmdQkQH9uMmHhSh","derivation":"m/48'/0'/0'/2'","root_fingerprint":"b8688df1","label":""},
  "x2/": {"type":"bip32","xpub":"Zpub74LquwpiAdpsXwRDJp46dQ9BhcoEhk3vPktqwMqGrQYmjRhYQi5mbemCRiHUXVh1Ypu5XRYzbbznqxodCwK5NPeVXAPVAuLGKrr1LUMFmPh","derivation":"m/48'/0'/0'/2'","root_fingerprint":"28645006","label":""},
  "x3/": {"type":"bip32","xpub":"Zpub72UafiS3U4xBBsiYjpCRcsEqm8i4Uo2Y2e5DmoNQALzLEXfyaJ7RvrGNGKznahzYT9T2BdMXiGPZ55NiuVukpcueupHwtfXeRKF3wyH3XDv","derivation":"m/48'/0'/0'/2'","root_fingerprint":"5436d724","label":""}
}
"#;

    // ====== sniff: positive cases ======

    #[test]
    fn sniff_standard_singlesig_matches() {
        assert!(ElectrumParser::sniff(STANDARD_BIP84.as_bytes()));
    }

    #[test]
    fn sniff_multisig_2of3_matches() {
        assert!(ElectrumParser::sniff(MULTISIG_2OF3.as_bytes()));
    }

    #[test]
    fn sniff_2fa_matches_for_routing_to_refusal_arm() {
        // SPEC §11.6: refused variants are sniff-positive so the parser arm
        // is reached and the §11.6.1 refusal template fires (vs falling
        // through to NoMatch with a generic "could not detect format"
        // error).
        let blob = br#"{"seed_version":17,"wallet_type":"2fa","use_encryption":false,"x1/":{}}"#;
        assert!(ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_imported_matches_for_routing_to_refusal_arm() {
        let blob = br#"{"seed_version":17,"wallet_type":"imported","use_encryption":false,"addresses":{}}"#;
        assert!(ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_encrypted_matches_for_routing_to_refusal_arm() {
        // use_encryption: true does NOT change sniff outcome; sniff
        // recognizes the wallet, parse arm refuses it.
        let blob = br#"{"seed_version":17,"wallet_type":"standard","use_encryption":true,"keystore":"base64-blob..."}"#;
        assert!(ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_3of5_multisig_matches() {
        let blob = br#"{"seed_version":17,"wallet_type":"3of5","use_encryption":false}"#;
        assert!(ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_seed_version_at_floor_11_matches() {
        let blob = br#"{"seed_version":11,"wallet_type":"standard","use_encryption":false,"keystore":{}}"#;
        assert!(ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_seed_version_at_ceiling_71_matches() {
        let blob = br#"{"seed_version":71,"wallet_type":"standard","use_encryption":false,"keystore":{}}"#;
        assert!(ElectrumParser::sniff(blob));
    }

    // ====== sniff: negative cases ======

    #[test]
    fn sniff_no_match_seed_version_below_floor_10() {
        let blob = br#"{"seed_version":10,"wallet_type":"standard"}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_no_match_seed_version_above_ceiling_72() {
        let blob = br#"{"seed_version":72,"wallet_type":"standard"}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_no_match_wallet_type_literal_multisig() {
        // SPEC §11.6: the literal string "multisig" is NEVER stored by
        // Electrum (only "<k>of<n>" patterns). A blob carrying it is NOT
        // an Electrum wallet.
        let blob = br#"{"seed_version":17,"wallet_type":"multisig"}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_no_match_wallet_type_unknown_string() {
        let blob = br#"{"seed_version":17,"wallet_type":"hd_wallet"}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_no_match_missing_seed_version() {
        let blob = br#"{"wallet_type":"standard","keystore":{}}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_no_match_missing_wallet_type() {
        let blob = br#"{"seed_version":17,"keystore":{}}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_no_match_seed_version_not_integer() {
        let blob = br#"{"seed_version":"17","wallet_type":"standard"}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_no_match_wallet_type_not_string() {
        let blob = br#"{"seed_version":17,"wallet_type":17}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_no_match_invalid_json() {
        assert!(!ElectrumParser::sniff(b"not json"));
    }

    #[test]
    fn sniff_no_match_empty() {
        assert!(!ElectrumParser::sniff(b""));
    }

    #[test]
    fn sniff_no_match_array_at_top_level() {
        assert!(!ElectrumParser::sniff(br#"[{"seed_version":17}]"#));
    }

    #[test]
    fn sniff_no_match_multisig_pattern_zero_k() {
        // Per Electrum semantics k >= 1; "0of3" is malformed. Our regex
        // matches any digits, so this is sniff-positive at the regex layer
        // — the bounds check is deferred to parse-time. Document that.
        // (If the SPEC tightens to k >= 1 at sniff-time, this test flips.)
        let blob = br#"{"seed_version":17,"wallet_type":"0of3"}"#;
        // Currently sniff-positive (regex matches). Parse will reject.
        assert!(ElectrumParser::sniff(blob));
    }

    #[test]
    fn sniff_no_match_multisig_pattern_with_suffix() {
        // Anchored trailing: `"2of3junk"` does NOT match.
        let blob = br#"{"seed_version":17,"wallet_type":"2of3junk"}"#;
        assert!(!ElectrumParser::sniff(blob));
    }

    // ====== parse: P6B happy paths ======

    #[test]
    fn parse_standard_bip84_produces_singlesig_bundle() {
        let mut stderr: Vec<u8> = Vec::new();
        let parsed = ElectrumParser::parse(STANDARD_BIP84.as_bytes(), &mut stderr)
            .expect("P6B standard parse must succeed");
        assert_eq!(parsed.len(), 1, "standard wallet emits 1 bundle");
        let p = &parsed[0];
        assert_eq!(p.cosigners.len(), 1, "singlesig has 1 cosigner");
        assert_eq!(p.threshold, None, "singlesig has no threshold");
        assert_eq!(p.network, bitcoin::Network::Bitcoin);
        // Cosigner xpub is the neutral form (SLIP-132 zpub → xpub).
        let xpub_str = p.cosigners[0].xpub.to_string();
        assert!(xpub_str.starts_with("xpub"), "neutral xpub form expected; got {xpub_str}");
        // Descriptor should be `wpkh([fp/84'/0'/0']xpub.../<0;1>/*)#csum`.
        assert!(
            p.original_descriptor.starts_with("wpkh(["),
            "wpkh wrapper expected; got {}",
            p.original_descriptor
        );
        assert!(
            p.original_descriptor.contains("/<0;1>/*"),
            "multipath suffix expected; got {}",
            p.original_descriptor
        );
        // Fingerprint matches the keystore.
        assert_eq!(
            p.cosigners[0].fingerprint.to_string(),
            "5436d724"
        );
    }

    #[test]
    fn parse_multisig_2of3_produces_wsh_sortedmulti_bundle() {
        let mut stderr: Vec<u8> = Vec::new();
        let parsed = ElectrumParser::parse(MULTISIG_2OF3.as_bytes(), &mut stderr)
            .expect("P6B multisig parse must succeed");
        assert_eq!(parsed.len(), 1, "multisig wallet emits 1 bundle");
        let p = &parsed[0];
        assert_eq!(p.cosigners.len(), 3, "2of3 has 3 cosigners");
        assert_eq!(p.threshold, Some(2), "2of3 has threshold 2");
        assert_eq!(p.network, bitcoin::Network::Bitcoin);
        assert!(
            p.original_descriptor.starts_with("wsh(sortedmulti(2,"),
            "wsh(sortedmulti(2,..)) expected; got {}",
            p.original_descriptor
        );
        // Three cosigner fingerprints in declaration order.
        assert_eq!(p.cosigners[0].fingerprint.to_string(), "b8688df1");
        assert_eq!(p.cosigners[1].fingerprint.to_string(), "28645006");
        assert_eq!(p.cosigners[2].fingerprint.to_string(), "5436d724");
    }

    #[test]
    fn parse_2fa_refuses_with_specific_template() {
        let blob = br#"{"seed_version":17,"wallet_type":"2fa","use_encryption":false,"x1/":{}}"#;
        let mut stderr: Vec<u8> = Vec::new();
        let err = ElectrumParser::parse(blob, &mut stderr).expect_err("2fa must refuse");
        let msg = err.to_string();
        assert!(
            msg.contains("2fa wallets require TrustedCoin two-factor restoration"),
            "expected 2fa-specific refusal template; got: {msg}"
        );
    }

    #[test]
    fn parse_imported_refuses_with_specific_template() {
        let blob =
            br#"{"seed_version":17,"wallet_type":"imported","use_encryption":false,"addresses":{}}"#;
        let mut stderr: Vec<u8> = Vec::new();
        let err = ElectrumParser::parse(blob, &mut stderr).expect_err("imported must refuse");
        let msg = err.to_string();
        assert!(
            msg.contains("imported-addresses wallets have no derivation chain"),
            "expected imported-specific refusal template; got: {msg}"
        );
    }

    #[test]
    fn parse_encrypted_refuses_with_specific_template_and_followup() {
        let blob = br#"{"seed_version":17,"wallet_type":"standard","use_encryption":true,"keystore":"blob"}"#;
        let mut stderr: Vec<u8> = Vec::new();
        let err = ElectrumParser::parse(blob, &mut stderr).expect_err("encrypted must refuse");
        let msg = err.to_string();
        assert!(
            msg.contains("encrypted wallet files require decrypting via")
                && msg.contains("wallet-import-electrum-encrypted"),
            "expected encrypted-specific refusal template + FOLLOWUP slug; got: {msg}"
        );
    }

    #[test]
    fn parse_drops_runtime_state_fields_with_stderr_notice() {
        // Inject several runtime-state fields. Parser should drop them
        // and emit ONE stderr NOTICE enumerating the dropped fields.
        let blob = r#"{
  "seed_version": 17,
  "wallet_type": "standard",
  "use_encryption": false,
  "keystore": {
    "type": "bip32",
    "xpub": "zpub6qTBTNftBzVTjgVcSUw7vW5N1KQbV93Jnrw314RHGkCkSx4vk6nEWH1MJfReXi2WThvuDRiRpyT7cDoakEcZMQ1iZPgfJgQrcVMR4aJWh6S",
    "derivation": "m/84'/0'/0'",
    "root_fingerprint": "5436d724",
    "label": ""
  },
  "addresses": {"receiving": [], "change": []},
  "labels": {},
  "transactions": {}
}
"#;
        let mut stderr: Vec<u8> = Vec::new();
        let _ = ElectrumParser::parse(blob.as_bytes(), &mut stderr).expect("parse with state");
        let stderr_str = String::from_utf8(stderr).unwrap();
        assert!(
            stderr_str.contains("notice:") && stderr_str.contains("dropped wallet-state fields"),
            "expected dropped-fields NOTICE; got: {stderr_str}"
        );
        assert!(
            stderr_str.contains("addresses")
                && stderr_str.contains("labels")
                && stderr_str.contains("transactions"),
            "NOTICE should enumerate present runtime-state fields; got: {stderr_str}"
        );
    }

    #[test]
    fn parse_rejects_multisig_with_out_of_bounds_n() {
        let blob =
            br#"{"seed_version":17,"wallet_type":"2of20","use_encryption":false,"x1/":{}}"#;
        let mut stderr: Vec<u8> = Vec::new();
        let err = ElectrumParser::parse(blob, &mut stderr).expect_err("n>15 must reject");
        let msg = err.to_string();
        assert!(
            msg.contains("out-of-bounds") && msg.contains("(2, 20)"),
            "expected out-of-bounds template; got: {msg}"
        );
    }

    #[test]
    fn parse_rejects_multisig_with_k_greater_than_n() {
        let blob = br#"{"seed_version":17,"wallet_type":"4of3","use_encryption":false,"x1/":{}}"#;
        let mut stderr: Vec<u8> = Vec::new();
        let err = ElectrumParser::parse(blob, &mut stderr).expect_err("k>n must reject");
        let msg = err.to_string();
        assert!(
            msg.contains("out-of-bounds") && msg.contains("(4, 3)"),
            "expected k>n out-of-bounds; got: {msg}"
        );
    }

    #[test]
    fn parse_rejects_missing_keystore_for_standard() {
        let blob = br#"{"seed_version":17,"wallet_type":"standard","use_encryption":false}"#;
        let mut stderr: Vec<u8> = Vec::new();
        let err =
            ElectrumParser::parse(blob, &mut stderr).expect_err("missing keystore must reject");
        assert!(err.to_string().contains("missing or non-object `keystore`"));
    }

    #[test]
    fn parse_rejects_missing_x_n_for_multisig() {
        let blob = r#"{
  "seed_version": 17,
  "wallet_type": "2of3",
  "use_encryption": false,
  "x1/": {"type":"bip32","xpub":"Zpub75ybJh4YZjnMskAAUkpy6uLizWcTTRC91yDtz9RcRwtavi4wHpBPZDEYUu9LoAPb6NQZNqKd6eKqF4FhqgWSaWQdqSt4FmdQkQH9uMmHhSh","derivation":"m/48'/0'/0'/2'","root_fingerprint":"b8688df1","label":""},
  "x2/": {"type":"bip32","xpub":"Zpub74LquwpiAdpsXwRDJp46dQ9BhcoEhk3vPktqwMqGrQYmjRhYQi5mbemCRiHUXVh1Ypu5XRYzbbznqxodCwK5NPeVXAPVAuLGKrr1LUMFmPh","derivation":"m/48'/0'/0'/2'","root_fingerprint":"28645006","label":""}
}
"#;
        let mut stderr: Vec<u8> = Vec::new();
        let err = ElectrumParser::parse(blob.as_bytes(), &mut stderr)
            .expect_err("missing x3/ must reject");
        let msg = err.to_string();
        assert!(
            msg.contains("missing or non-object cosigner key `x3/`"),
            "expected x3/ missing template; got: {msg}"
        );
    }

    // ====== fixture parse: round-trip against canonical fixtures ======

    fn read_fixture(name: &str) -> Vec<u8> {
        let path = format!("tests/fixtures/wallet_import/{name}");
        std::fs::read(&path).unwrap_or_else(|e| panic!("read fixture {path}: {e}"))
    }

    #[test]
    fn fixture_standard_bip84_mainnet_parses() {
        let blob = read_fixture("electrum-standard-bip84-mainnet.json");
        let mut stderr: Vec<u8> = Vec::new();
        let parsed = ElectrumParser::parse(&blob, &mut stderr).expect("fixture parse");
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].cosigners.len(), 1);
        assert_eq!(parsed[0].cosigners[0].fingerprint.to_string(), "5436d724");
        assert!(parsed[0].original_descriptor.starts_with("wpkh(["));
    }

    #[test]
    fn fixture_standard_bip49_mainnet_parses_as_sh_wpkh() {
        let blob = read_fixture("electrum-standard-bip49-mainnet.json");
        let mut stderr: Vec<u8> = Vec::new();
        let parsed = ElectrumParser::parse(&blob, &mut stderr).expect("fixture parse");
        assert_eq!(parsed[0].cosigners.len(), 1);
        assert!(
            parsed[0].original_descriptor.starts_with("sh(wpkh(["),
            "BIP-49 ypub must wrap as sh(wpkh(...)); got: {}",
            parsed[0].original_descriptor
        );
    }

    #[test]
    fn fixture_multisig_2of3_wsh_parses() {
        let blob = read_fixture("electrum-multisig-2of3-wsh.json");
        let mut stderr: Vec<u8> = Vec::new();
        let parsed = ElectrumParser::parse(&blob, &mut stderr).expect("fixture parse");
        assert_eq!(parsed[0].cosigners.len(), 3);
        assert_eq!(parsed[0].threshold, Some(2));
        assert!(parsed[0].original_descriptor.starts_with("wsh(sortedmulti(2,"));
    }

    #[test]
    fn fixture_2fa_refuses() {
        let blob = read_fixture("electrum-2fa-refused.json");
        let mut stderr: Vec<u8> = Vec::new();
        let err = ElectrumParser::parse(&blob, &mut stderr).expect_err("2fa fixture refuses");
        assert!(err.to_string().contains("TrustedCoin"));
    }

    #[test]
    fn fixture_imported_refuses() {
        let blob = read_fixture("electrum-imported-refused.json");
        let mut stderr: Vec<u8> = Vec::new();
        let err = ElectrumParser::parse(&blob, &mut stderr).expect_err("imported refuses");
        assert!(err.to_string().contains("imported-addresses"));
    }

    #[test]
    fn fixture_encrypted_refuses() {
        let blob = read_fixture("electrum-encrypted-refused.json");
        let mut stderr: Vec<u8> = Vec::new();
        let err = ElectrumParser::parse(&blob, &mut stderr).expect_err("encrypted refuses");
        assert!(err.to_string().contains("encrypted wallet files require decrypting"));
    }

    #[test]
    fn parse_rejects_heterogeneous_cosigner_variants() {
        // x1/ is Zpub (mainnet wsh multisig), x2/ is xpub (neutral) —
        // heterogeneous SLIP-132 variants must reject.
        let blob = r#"{
  "seed_version": 17,
  "wallet_type": "2of2",
  "use_encryption": false,
  "x1/": {"type":"bip32","xpub":"Zpub75ybJh4YZjnMskAAUkpy6uLizWcTTRC91yDtz9RcRwtavi4wHpBPZDEYUu9LoAPb6NQZNqKd6eKqF4FhqgWSaWQdqSt4FmdQkQH9uMmHhSh","derivation":"m/48'/0'/0'/2'","root_fingerprint":"b8688df1","label":""},
  "x2/": {"type":"bip32","xpub":"xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6","derivation":"m/48'/0'/0'/2'","root_fingerprint":"28645006","label":""}
}
"#;
        let mut stderr: Vec<u8> = Vec::new();
        let err = ElectrumParser::parse(blob.as_bytes(), &mut stderr)
            .expect_err("heterogeneous variants must reject");
        let msg = err.to_string();
        assert!(
            msg.contains("must share a variant"),
            "expected heterogeneous-variant rejection; got: {msg}"
        );
    }

    // ====== parse_multisig_wallet_type helper ======

    #[test]
    fn parse_multisig_wallet_type_2of3() {
        assert_eq!(parse_multisig_wallet_type("2of3"), Some((2, 3)));
    }

    #[test]
    fn parse_multisig_wallet_type_15of15() {
        assert_eq!(parse_multisig_wallet_type("15of15"), Some((15, 15)));
    }

    #[test]
    fn parse_multisig_wallet_type_rejects_non_pattern() {
        assert_eq!(parse_multisig_wallet_type("standard"), None);
        assert_eq!(parse_multisig_wallet_type(""), None);
        assert_eq!(parse_multisig_wallet_type("multisig"), None);
    }

    #[test]
    fn parse_multisig_wallet_type_rejects_overflow() {
        // u8 max is 255; "256of256" parses as numbers but overflows u8.
        assert_eq!(parse_multisig_wallet_type("256of256"), None);
    }

    // ====== ElectrumSourceMetadata construction sanity ======

    #[test]
    fn metadata_standard_construction() {
        let m = ElectrumSourceMetadata {
            seed_version: 17,
            wallet_type: ElectrumWalletType::Standard,
            wallet_name: None,
            dropped_fields: Vec::new(),
        };
        assert_eq!(m.seed_version, 17);
        assert!(matches!(m.wallet_type, ElectrumWalletType::Standard));
    }

    #[test]
    fn metadata_multisig_construction() {
        let m = ElectrumSourceMetadata {
            seed_version: 17,
            wallet_type: ElectrumWalletType::Multisig { k: 2, n: 3 },
            wallet_name: Some("Test".to_string()),
            dropped_fields: vec!["addr_history".to_string()],
        };
        match m.wallet_type {
            ElectrumWalletType::Multisig { k, n } => {
                assert_eq!(k, 2);
                assert_eq!(n, 3);
            }
            _ => panic!("expected Multisig variant"),
        }
    }
}
