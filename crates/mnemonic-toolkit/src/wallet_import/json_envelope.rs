//! v0.27.0 Phase 5 — `import-wallet --json` envelope CONSUMER.
//!
//! This module is the inverse of `cmd::import_wallet::emit_json_envelope`
//! (the Phase 4 emitter). It parses a v0.27.0 envelope JSON blob into
//! typed Rust structs (`ImportJsonEnvelope` + the deserialization-friendly
//! `BundleJsonView` mirror struct), decodes per-cosigner mk1 chunks back
//! into `ResolvedSlot` values per SPEC §3.6.1, and constructs the
//! 16-field `EmitInputs` contract per SPEC §3.7.1.
//!
//! Why a mirror struct (Phase 4 holistic review I1 fold). `crate::format::
//! BundleJson` is `#[derive(Serialize)]` only at `format.rs:119` and
//! carries `&'static str` fields (`schema_version`, `mode`, `network`,
//! `Option<&'static str>` for `template`, plus `multisig.template`).
//! `serde` cannot deserialize into `&'static str`. The wire-shape is
//! also union-of-shapes for `mk1` (single-cosigner = flat `Vec<String>`,
//! multi-cosigner = `Vec<Vec<String>>`) which `#[serde(untagged)]`
//! handles at serialize time but requires explicit handling at deser.
//! Rather than inject a `Deserialize` impl on `BundleJson` (which would
//! force a String-flavor of every static-str field across the wire-
//! emission codebase), we ship a `BundleJsonView` here whose fields are
//! `String` / `Option<String>` and `Vec` / nested-Vec for the union case.
//! Plan-doc §4.5 names this option (a) and recommends it for Phase 5
//! over the alternative `serde_json::Value` traversal at
//! `verify_bundle.rs:980-1010`.
//!
//! Public surface (`pub(crate)`):
//! - `ImportJsonEnvelope` — typed wrapper around one envelope-array element.
//! - `BundleJsonView` — deserialization mirror of `BundleJson`.
//! - `parse_import_json_envelopes(raw, index) -> ImportJsonEnvelope` —
//!   load a JSON array; pick the entry at `index` (with multi-entry
//!   semantics + `BadInput` exit 2 on ambiguity / out-of-range).
//! - `envelope_to_resolved_slots(envelope, stderr) -> Vec<ResolvedSlot>` —
//!   decode mk1 chunks (single or multi) into `ResolvedSlot` values
//!   per §3.6.1. `stderr` carries the per-cosigner origin_fingerprint
//!   substitution NOTICE when any mk1 card omits the master fingerprint
//!   (v0.27.1 Phase 2 I5 fold).
//! - `mk1_card_to_resolved_slot(card, index, stderr) -> ResolvedSlot` —
//!   per-cosigner decode helper. Emits a NOTICE on the substitution
//!   fallback (see `envelope_to_resolved_slots` above).
//! - `cli_network_from_bitcoin_network(n) -> CliNetwork` — inverse of
//!   `network_human_name`; covers all 4 variants per §4.5 R0 scope.
//! - `cli_network_from_str(s) -> CliNetwork` — inverse of
//!   `CliNetwork::human_name()`; parses the envelope's `network` field.

use crate::error::ToolkitError;
use crate::network::CliNetwork;
use crate::synthesize::ResolvedSlot;
use bitcoin::bip32::DerivationPath;
use serde::Deserialize;
use std::str::FromStr;

/// SPEC §3.2 outer envelope — one element of the JSON array emitted by
/// `import-wallet --json`. Deserialization-friendly mirror of the
/// `serde_json::Map` Phase 4 hand-builds.
///
/// `#[allow(dead_code)]` on the whole struct: most fields are wire-shape
/// carry consumed by serde but not read by Phase 5 consumer code (which
/// only reaches into `bundle`); the dead-code analysis can't see the
/// serde deserialization use.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub(crate) struct ImportJsonEnvelope {
    /// `"1"` for v0.27.0; future versions bump.
    pub(crate) schema_version: String,
    /// `"bsms"` or `"bitcoin-core"`.
    pub(crate) source_format: String,
    pub(crate) bundle: BundleJsonView,
    // v0.37.8 — per-format source-metadata fields, deserialized as
    // opaque `serde_json::Value` (each format's projection-shape lives in
    // `cmd::import_wallet::emit_json_envelope`; the consumer here only
    // path-walks them via `resolved_wallet_name`). All optional + serde
    // `default` so envelopes from format families not carrying a
    // wallet-name (or older toolkits) deserialize unchanged. Only the
    // wallet-name lift consumes these; future consumers may grow.
    #[serde(default)]
    pub(crate) source_metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub(crate) sparrow_source_metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub(crate) specter_source_metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub(crate) jade_source_metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub(crate) electrum_source_metadata: Option<serde_json::Value>,
    #[serde(default)]
    pub(crate) coldcard_multisig_source_metadata: Option<serde_json::Value>,
    // Phase 5 still does not need `bsms_audit`, `roundtrip`, or
    // `bsms_round1_verifications` for the consumer paths; serde drops
    // unknown fields by default.
}

impl ImportJsonEnvelope {
    /// v0.37.8 — universal source-name lift. Returns the wallet name carried
    /// in the envelope's per-format source-metadata projection, if any of the
    /// six name-carrying formats parsed populated it. Probed in import-order
    /// priority (bitcoin-core → sparrow → specter → jade → electrum →
    /// coldcard-multisig); per the spec, at most one of the six is populated
    /// per envelope (each is emitted ONLY when the matching parser claimed
    /// the input), so the order is a defensive tie-break, not a precedence
    /// statement.
    ///
    /// Consumed by `cmd::export_wallet::run_from_import_json` to flow the
    /// lifted name into `EmitInputs.wallet_name` AND the
    /// `wallet_name_is_non_default` flag so the Specter `MissingField::
    /// WalletName` path doesn't fire on a lifted name (SPEC §13 R1-L1).
    pub(crate) fn resolved_wallet_name(&self) -> Option<String> {
        // (probe_root, json_path) — `&["a","b"]` walks `obj["a"]["b"]`.
        let probes: &[(&Option<serde_json::Value>, &[&str])] = &[
            (&self.source_metadata, &["wallet_name"]),
            (&self.sparrow_source_metadata, &["label"]),
            (&self.specter_source_metadata, &["label"]),
            (&self.jade_source_metadata, &["coldcard_compat", "name"]),
            (&self.electrum_source_metadata, &["wallet_name"]),
            (&self.coldcard_multisig_source_metadata, &["name"]),
        ];
        for (root, path) in probes {
            if let Some(v) = root.as_ref() {
                if let Some(name) = walk_str(v, path) {
                    if !name.is_empty() {
                        return Some(name.to_string());
                    }
                }
            }
        }
        None
    }
}

/// Walk a nested `serde_json::Value` along `path`, returning the leaf as
/// `&str` if it terminates at a non-null string. Designed for the universal
/// source-name lift (`ImportJsonEnvelope::resolved_wallet_name`); kept
/// general so future per-format probes can reuse it.
fn walk_str<'a>(v: &'a serde_json::Value, path: &[&str]) -> Option<&'a str> {
    let mut cur = v;
    for key in path {
        cur = cur.get(*key)?;
    }
    cur.as_str()
}

/// Deserialization mirror of `crate::format::BundleJson`. Field order
/// matches `format.rs:119-145`. String / owned types throughout.
///
/// `#[allow(dead_code)]` on the whole struct — same reasoning as
/// `ImportJsonEnvelope`: wire-shape carry consumed by serde + by selected
/// Phase 5 consumer paths.
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub(crate) struct BundleJsonView {
    pub(crate) schema_version: String,
    pub(crate) mode: String,
    pub(crate) network: String,
    pub(crate) template: Option<String>,
    pub(crate) descriptor: Option<String>,
    pub(crate) account: u32,
    pub(crate) origin_path: Option<String>,
    pub(crate) origin_paths: Option<Vec<String>>,
    pub(crate) master_fingerprint: Option<String>,
    pub(crate) ms1: Vec<String>,
    /// Union shape: flat `Vec<String>` for single-cosigner emission
    /// (`MkField::Single`), nested `Vec<Vec<String>>` for multi-cosigner
    /// (`MkField::Multi`). The custom deserializer normalizes both to
    /// `Vec<Vec<String>>` (one chunk-vec per cosigner) so downstream
    /// per-cosigner iteration is uniform.
    #[serde(deserialize_with = "deserialize_mk_field_normalized")]
    pub(crate) mk1: Vec<Vec<String>>,
    pub(crate) md1: Vec<String>,
    pub(crate) multisig: Option<MultisigInfoView>,
    pub(crate) privacy_preserving: bool,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub(crate) struct MultisigInfoView {
    pub(crate) template: String,
    pub(crate) threshold: u8,
    pub(crate) cosigner_count: usize,
    pub(crate) path_family: String,
    pub(crate) cosigners: Vec<CosignerEntryView>,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub(crate) struct CosignerEntryView {
    pub(crate) index: usize,
    pub(crate) master_fingerprint: Option<String>,
    pub(crate) origin_path: String,
    pub(crate) xpub: String,
}

/// `MkField` deserializer: accepts both `["chunk1", "chunk2", ...]` (flat
/// — `MkField::Single`) and `[["c1","c2"],["c3","c4"], ...]` (nested —
/// `MkField::Multi`). Normalizes flat to a length-1 outer with the flat
/// list as its single inner. Mirrors the union-handling at
/// `verify_bundle.rs:989-1010`.
fn deserialize_mk_field_normalized<'de, D>(de: D) -> Result<Vec<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    let value = serde_json::Value::deserialize(de)?;
    let arr = value
        .as_array()
        .ok_or_else(|| D::Error::custom("mk1 must be an array"))?;
    if arr.is_empty() {
        return Ok(Vec::new());
    }
    // Probe first element to disambiguate single vs multi shape.
    match &arr[0] {
        serde_json::Value::String(_) => {
            // Flat: Vec<String> → wrap into Vec<Vec<String>> with one outer.
            let inner: Vec<String> = arr
                .iter()
                .map(|v| {
                    v.as_str()
                        .map(String::from)
                        .ok_or_else(|| D::Error::custom("mk1 flat-form must contain only strings"))
                })
                .collect::<Result<_, _>>()?;
            Ok(vec![inner])
        }
        serde_json::Value::Array(_) => {
            // Nested: Vec<Vec<String>>.
            arr.iter()
                .map(|outer_v| {
                    let inner_arr = outer_v.as_array().ok_or_else(|| {
                        D::Error::custom("mk1 nested-form outer element must be array")
                    })?;
                    inner_arr
                        .iter()
                        .map(|s| {
                            s.as_str().map(String::from).ok_or_else(|| {
                                D::Error::custom("mk1 nested-form inner element must be string")
                            })
                        })
                        .collect::<Result<_, _>>()
                })
                .collect()
        }
        _ => Err(D::Error::custom(
            "mk1 must be Vec<String> (single) or Vec<Vec<String>> (multi)",
        )),
    }
}

/// Multi-entry-envelope semantics. The envelope wire-shape is always a
/// top-level JSON array (length 1 for BSMS — single descriptor per blob;
/// length-N for Bitcoin Core `listdescriptors` with N entries).
///
/// `index` semantics:
/// - `Some(n)` — return entry `n`; out-of-range is `BadInput` exit 2.
/// - `None` — accept length-1 arrays implicitly; ambiguous (length > 1)
///   is `BadInput` exit 2 per opus R0 D8 lock (multi-entry without
///   `--import-json-index` is a footgun: silent N=0 selection would
///   discard descriptors 1+).
pub(crate) fn parse_import_json_envelopes(
    raw: &str,
    index: Option<usize>,
    flag_label: &str,
) -> Result<ImportJsonEnvelope, ToolkitError> {
    let envelopes: Vec<ImportJsonEnvelope> = serde_json::from_str(raw)
        .map_err(|e| ToolkitError::BadInput(format!("{flag_label}: envelope JSON parse: {e}")))?;
    if envelopes.is_empty() {
        return Err(ToolkitError::BadInput(format!(
            "{flag_label}: envelope array is empty"
        )));
    }
    let selected = match index {
        Some(n) => {
            if n >= envelopes.len() {
                return Err(ToolkitError::BadInput(format!(
                    "{flag_label}-index {n} out of range; envelope array has {} entries",
                    envelopes.len()
                )));
            }
            envelopes.into_iter().nth(n).unwrap()
        }
        None => {
            if envelopes.len() > 1 {
                return Err(ToolkitError::BadInput(format!(
                    "{flag_label}: envelope array has {} entries; supply --import-json-index <N> \
                     (or --from-import-json-index <N> on export-wallet) to pick one",
                    envelopes.len()
                )));
            }
            envelopes.into_iter().next().unwrap()
        }
    };
    // Gate the SELECTED envelope's schema versions — fail closed on an
    // unrecognized/future version rather than silently mis-parsing (serde
    // drops unknown fields). Only the chosen entry is consumed, so only it is
    // validated.
    validate_schema_versions(&selected, flag_label)?;
    Ok(selected)
}

/// The import-json envelope schema versions this toolkit understands. These
/// mirror the EMIT side — outer `"1"` (`cmd::import_wallet::emit_json_envelope`)
/// and inner bundle `"4"` (`format.rs::BundleJson::schema_version`); an
/// emit-side bump must update these in lockstep.
const SUPPORTED_ENVELOPE_SCHEMA: &str = "1";
const SUPPORTED_BUNDLE_SCHEMA: &str = "4";

/// Reject an import-json envelope whose schema_version (outer or inner bundle)
/// is not exactly what this toolkit supports. Strict-equal / fail-closed: a
/// future version with changed semantics must NOT be silently parsed by an
/// older binary.
fn validate_schema_versions(
    env: &ImportJsonEnvelope,
    flag_label: &str,
) -> Result<(), ToolkitError> {
    if env.schema_version != SUPPORTED_ENVELOPE_SCHEMA {
        return Err(ToolkitError::BadInput(format!(
            "{flag_label}: unsupported import-json envelope schema_version {:?} \
             (this toolkit supports {SUPPORTED_ENVELOPE_SCHEMA:?}); upgrade the toolkit",
            env.schema_version
        )));
    }
    if env.bundle.schema_version != SUPPORTED_BUNDLE_SCHEMA {
        return Err(ToolkitError::BadInput(format!(
            "{flag_label}: unsupported import-json bundle schema_version {:?} \
             (this toolkit supports {SUPPORTED_BUNDLE_SCHEMA:?}); upgrade the toolkit",
            env.bundle.schema_version
        )));
    }
    Ok(())
}

/// SPEC §3.6.1 — decode every mk1 chunk-vector in the envelope's
/// `bundle.mk1` field into a `ResolvedSlot`. Slot order is the declaration
/// order from the source descriptor (preserved by Phase 4's emit via
/// synthesize_descriptor; preserved at deser by `deserialize_mk_field_normalized`).
///
/// `entropy` is always `None` for v0.27.0 wallet-import-derived envelopes
/// (the envelope's `bundle.ms1[i] == ""` sentinel marks watch-only; if any
/// `ms1[i] != ""`, the caller is responsible for decoding it and overlaying
/// entropy at the corresponding slot — see `bundle --import-json` consumer
/// at `cmd::bundle::run_from_import_json`).
pub(crate) fn envelope_to_resolved_slots<E: std::io::Write>(
    envelope: &ImportJsonEnvelope,
    stderr: &mut E,
) -> Result<Vec<ResolvedSlot>, ToolkitError> {
    let mk1_outer = &envelope.bundle.mk1;
    let mut out = Vec::with_capacity(mk1_outer.len());
    for (i, chunks) in mk1_outer.iter().enumerate() {
        let chunk_refs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
        let card = mk_codec::decode(&chunk_refs).map_err(|e| {
            ToolkitError::BadInput(format!("--import-json: mk1[{i}] decode failed: {e}"))
        })?;
        let mut slot = mk1_card_to_resolved_slot(&card, i, stderr)?;
        // v0.37.10: the mk1 card's origin_path is the account-consistent path (it
        // round-trips the xpub it carries — a depth-3 account xpub annotated with a
        // depth-4 BIP-48 descriptor origin yields a depth-3 mk1 path). The FULL
        // descriptor origin lives in the envelope's bundle.origin_path[s] (md1's
        // path_decl). Prefer it so the re-imported cosigner origin matches the source
        // descriptor; the `mk1_origin_path` helper re-truncates for the emitted card.
        let full_origin: Option<&str> = match &envelope.bundle.origin_paths {
            Some(paths) => paths.get(i).map(|s| s.as_str()),
            None => envelope.bundle.origin_path.as_deref(),
        };
        if let Some(s) = full_origin {
            slot.path = derivation_path_from_envelope(s, "--import-json")?;
        }
        out.push(slot);
    }
    Ok(out)
}

/// Per-cosigner `mk_codec::KeyCard → ResolvedSlot` decode (§3.6.1).
///
/// `card.origin_fingerprint` falls back to `card.xpub.fingerprint()` when
/// missing (privacy-preserving envelopes). v0.27.0 wallet-import-emitted
/// envelopes always carry the master fingerprint (Phase 4 synthesizes
/// with `privacy_preserving: false`), but hand-crafted / intermediate
/// envelopes might omit it; mirror sortedmulti's xpub-fingerprint
/// fallback so the consumer is robust.
///
/// v0.27.1 Phase 2 I5 fold: emit a stderr NOTICE when the fallback fires.
/// Master-fp and current-xpub-fp are semantically distinct — substituting
/// silently produces wallets with mismatched origin annotations downstream.
/// Closes the self-confessed `let _ = slot_idx; // reserved` gap by wiring
/// `slot_idx` through to the NOTICE template.
pub(crate) fn mk1_card_to_resolved_slot<E: std::io::Write>(
    card: &mk_codec::KeyCard,
    slot_idx: usize,
    stderr: &mut E,
) -> Result<ResolvedSlot, ToolkitError> {
    let fingerprint = match card.origin_fingerprint {
        Some(fp) => fp,
        None => {
            let substituted = card.xpub.fingerprint();
            writeln!(
                stderr,
                "notice: import-wallet: mk1[{slot_idx}]: origin_fingerprint absent; substituting xpub-derived fingerprint {} (master-fp and current-xpub-fp may differ; downstream wallets may show mismatched origins)",
                substituted.to_string().to_lowercase()
            )
            .map_err(ToolkitError::Io)?;
            substituted
        }
    };
    Ok(ResolvedSlot {
        xpub: card.xpub,
        fingerprint,
        path: card.origin_path.clone(),
        entropy: None,
        master_xpub: None,
        language: None,
        _entropy_pin: None,
    })
}

/// Inverse of `cmd::import_wallet::network_human_name`. Phase 4's emitter
/// writes `bundle.network` as one of "mainnet" / "testnet" / "signet" /
/// "regtest" / "unknown" (the "unknown" case never round-trips). Phase 5
/// rejects unknown strings as `BadInput`.
pub(crate) fn cli_network_from_str(s: &str) -> Result<CliNetwork, ToolkitError> {
    match s {
        "mainnet" => Ok(CliNetwork::Mainnet),
        "testnet" => Ok(CliNetwork::Testnet),
        "signet" => Ok(CliNetwork::Signet),
        "regtest" => Ok(CliNetwork::Regtest),
        other => Err(ToolkitError::BadInput(format!(
            "--import-json: unrecognized envelope.bundle.network {other:?}; \
             expected one of mainnet|testnet|signet|regtest"
        ))),
    }
}

/// Convert `bitcoin::Network → CliNetwork`. v0.27.0 Phase 5 helper per
/// plan §4.5 R0 scope item ("confirm helper covers all 4 variants").
/// Rejects unknown variants (`bitcoin::Network` is `#[non_exhaustive]`
/// upstream) with `BadInput`.
///
/// Currently unused by Phase 5 (the envelope's `bundle.network` is a
/// `&'static str` form — consumers use `cli_network_from_str` instead).
/// Kept here as the symmetric helper for any future consumer that has
/// a typed `bitcoin::Network` value.
#[allow(dead_code)]
pub(crate) fn cli_network_from_bitcoin_network(
    n: bitcoin::Network,
) -> Result<CliNetwork, ToolkitError> {
    match n {
        bitcoin::Network::Bitcoin => Ok(CliNetwork::Mainnet),
        bitcoin::Network::Testnet => Ok(CliNetwork::Testnet),
        bitcoin::Network::Signet => Ok(CliNetwork::Signet),
        bitcoin::Network::Regtest => Ok(CliNetwork::Regtest),
        other => Err(ToolkitError::BadInput(format!(
            "--import-json: bitcoin::Network::{other:?} not representable as CliNetwork"
        ))),
    }
}

/// Read `--import-json` / `--from-import-json` value: either a file path
/// or `-` (stdin). Mirrors the `read_blob` precedent at
/// `cmd::import_wallet::read_blob` for the `--blob` flag.
pub(crate) fn read_import_json_arg<R: std::io::Read + ?Sized>(
    value: &str,
    stdin: &mut R,
    flag_label: &str,
) -> Result<String, ToolkitError> {
    if value == "-" {
        let mut buf = String::new();
        stdin.read_to_string(&mut buf).map_err(ToolkitError::Io)?;
        return Ok(buf);
    }
    std::fs::read_to_string(value)
        .map_err(|e| ToolkitError::BadInput(format!("{flag_label}: read {value}: {e}")))
}

/// Parse the envelope's `descriptor` field, stripping the BIP-380
/// `#<checksum>` suffix for downstream `concrete_keys_to_placeholders`
/// consumption (which expects the body sans checksum, matching the
/// `wallet_import::bsms` step at line 141).
///
/// The Phase 4 wire-shape preserves `#<csum>` verbatim per §3.2.1 row
/// `descriptor`; the strip happens at consumer time.
///
/// **Phase 5 R0 I1 fold:** previously silently fell back to the
/// checksum-bearing form on validation failure, masking a hand-crafted /
/// edited envelope where the descriptor body was mutated but the
/// checksum kept. Now propagates a clean BIP-380 checksum error matching
/// the sibling convention at `wallet_import/bsms.rs:141-145`.
pub(crate) fn descriptor_body_no_csum<'a>(
    descriptor_with_csum: &'a str,
    flag_label: &str,
) -> Result<&'a str, ToolkitError> {
    miniscript::descriptor::checksum::verify_checksum(descriptor_with_csum).map_err(|e| {
        ToolkitError::BadInput(format!(
            "{flag_label}: BIP-380 checksum validation failed for envelope.bundle.descriptor: {e}"
        ))
    })
}

/// Parse a `DerivationPath` from an envelope `origin_path` string
/// (form `m/48'/0'/0'/2'`). v0.37.10: live — `envelope_to_resolved_slots` uses it
/// to source the full descriptor origin from the envelope metadata (the mk1 card
/// now carries the account-consistent, possibly-shorter path).
pub(crate) fn derivation_path_from_envelope(
    s: &str,
    flag_label: &str,
) -> Result<DerivationPath, ToolkitError> {
    DerivationPath::from_str(s)
        .map_err(|e| ToolkitError::BadInput(format!("{flag_label}: origin_path parse {s:?}: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Single-cosigner mk1 deserializes as `Vec<Vec<String>>` with one
    /// outer entry; flat input.
    #[test]
    fn mk_field_deser_single_form_normalizes_to_outer_length_1() {
        let bundle_json = r#"{
            "schema_version": "4",
            "mode": "watch-only",
            "network": "mainnet",
            "template": null,
            "descriptor": "wpkh(xpub.../<0;1>/*)",
            "account": 0,
            "origin_path": "m/84'/0'/0'",
            "origin_paths": null,
            "master_fingerprint": "deadbeef",
            "ms1": [""],
            "mk1": ["mk1qpchunk0", "mk1qpchunk1"],
            "md1": ["md1qpchunk0"],
            "multisig": null,
            "privacy_preserving": false
        }"#;
        let v: BundleJsonView = serde_json::from_str(bundle_json).expect("parse");
        assert_eq!(v.mk1.len(), 1, "flat-form must normalize to outer-len-1");
        assert_eq!(v.mk1[0].len(), 2);
    }

    /// Multi-cosigner mk1 deserializes as `Vec<Vec<String>>` directly.
    #[test]
    fn mk_field_deser_multi_form_preserves_outer_length_n() {
        let bundle_json = r#"{
            "schema_version": "4",
            "mode": "watch-only",
            "network": "mainnet",
            "template": null,
            "descriptor": "wsh(sortedmulti(2,@0,@1,@2))",
            "account": 0,
            "origin_path": "m/48'/0'/0'/2'",
            "origin_paths": null,
            "master_fingerprint": null,
            "ms1": ["", "", ""],
            "mk1": [["a0","a1"],["b0","b1"],["c0","c1"]],
            "md1": ["md1qpchunk0"],
            "multisig": null,
            "privacy_preserving": false
        }"#;
        let v: BundleJsonView = serde_json::from_str(bundle_json).expect("parse");
        assert_eq!(v.mk1.len(), 3, "nested-form must preserve outer length");
        for inner in &v.mk1 {
            assert_eq!(inner.len(), 2);
        }
    }

    /// `cli_network_from_str` covers all 4 variants + rejects unknowns.
    #[test]
    fn cli_network_from_str_covers_all_four_and_rejects_unknown() {
        assert_eq!(
            cli_network_from_str("mainnet").unwrap(),
            CliNetwork::Mainnet
        );
        assert_eq!(
            cli_network_from_str("testnet").unwrap(),
            CliNetwork::Testnet
        );
        assert_eq!(cli_network_from_str("signet").unwrap(), CliNetwork::Signet);
        assert_eq!(
            cli_network_from_str("regtest").unwrap(),
            CliNetwork::Regtest
        );
        assert!(cli_network_from_str("bogus").is_err());
        assert!(cli_network_from_str("unknown").is_err());
    }

    /// `cli_network_from_bitcoin_network` covers all 4 variants per §4.5
    /// R0 scope.
    #[test]
    fn cli_network_from_bitcoin_network_covers_all_four() {
        assert_eq!(
            cli_network_from_bitcoin_network(bitcoin::Network::Bitcoin).unwrap(),
            CliNetwork::Mainnet
        );
        assert_eq!(
            cli_network_from_bitcoin_network(bitcoin::Network::Testnet).unwrap(),
            CliNetwork::Testnet
        );
        assert_eq!(
            cli_network_from_bitcoin_network(bitcoin::Network::Signet).unwrap(),
            CliNetwork::Signet
        );
        assert_eq!(
            cli_network_from_bitcoin_network(bitcoin::Network::Regtest).unwrap(),
            CliNetwork::Regtest
        );
    }

    /// `parse_import_json_envelopes` errors on ambiguous multi-entry
    /// without `--import-json-index`.
    #[test]
    fn parse_import_json_envelopes_multi_entry_without_index_errors() {
        let raw = r#"[
            {"schema_version":"1","source_format":"bitcoin-core","bundle":{
                "schema_version":"4","mode":"watch-only","network":"mainnet",
                "template":null,"descriptor":"wpkh(@0)","account":0,
                "origin_path":"m","origin_paths":null,"master_fingerprint":null,
                "ms1":[""],"mk1":["a"],"md1":["m1"],"multisig":null,
                "privacy_preserving":false}},
            {"schema_version":"1","source_format":"bitcoin-core","bundle":{
                "schema_version":"4","mode":"watch-only","network":"mainnet",
                "template":null,"descriptor":"wpkh(@0)","account":0,
                "origin_path":"m","origin_paths":null,"master_fingerprint":null,
                "ms1":[""],"mk1":["a"],"md1":["m1"],"multisig":null,
                "privacy_preserving":false}}
        ]"#;
        let err = parse_import_json_envelopes(raw, None, "--import-json").unwrap_err();
        assert!(format!("{err:?}").contains("envelope array has 2 entries"));
    }

    // ── schema-version gate (`import-json-schema-version-unchecked`) ──────────

    /// A single-entry valid `"1"`/`"4"` envelope parses Ok (no-regression).
    /// RED-equivalent baseline for the two reject cells below.
    #[test]
    fn parse_import_json_accepts_supported_schema_versions() {
        let raw = r#"[
            {"schema_version":"1","source_format":"bitcoin-core","bundle":{
                "schema_version":"4","mode":"watch-only","network":"mainnet",
                "template":null,"descriptor":"wpkh(@0)","account":0,
                "origin_path":"m","origin_paths":null,"master_fingerprint":null,
                "ms1":[""],"mk1":["a"],"md1":["m1"],"multisig":null,
                "privacy_preserving":false}}
        ]"#;
        let env = parse_import_json_envelopes(raw, None, "--import-json")
            .expect("valid 1/4 envelope must parse");
        assert_eq!(env.schema_version, "1");
        assert_eq!(env.bundle.schema_version, "4");
    }

    /// An unsupported OUTER envelope schema_version is rejected (not silently
    /// mis-parsed). Without the gate this parses Ok → the test catches it.
    #[test]
    fn parse_import_json_rejects_unsupported_envelope_schema_version() {
        let raw = r#"[
            {"schema_version":"2","source_format":"bitcoin-core","bundle":{
                "schema_version":"4","mode":"watch-only","network":"mainnet",
                "template":null,"descriptor":"wpkh(@0)","account":0,
                "origin_path":"m","origin_paths":null,"master_fingerprint":null,
                "ms1":[""],"mk1":["a"],"md1":["m1"],"multisig":null,
                "privacy_preserving":false}}
        ]"#;
        let err = parse_import_json_envelopes(raw, None, "--import-json").unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("unsupported import-json envelope schema_version")
                && msg.contains("\"2\""),
            "expected an envelope-version rejection naming \"2\"; got {msg}"
        );
    }

    /// An unsupported INNER bundle schema_version is rejected.
    #[test]
    fn parse_import_json_rejects_unsupported_bundle_schema_version() {
        let raw = r#"[
            {"schema_version":"1","source_format":"bitcoin-core","bundle":{
                "schema_version":"5","mode":"watch-only","network":"mainnet",
                "template":null,"descriptor":"wpkh(@0)","account":0,
                "origin_path":"m","origin_paths":null,"master_fingerprint":null,
                "ms1":[""],"mk1":["a"],"md1":["m1"],"multisig":null,
                "privacy_preserving":false}}
        ]"#;
        let err = parse_import_json_envelopes(raw, None, "--import-json").unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("unsupported import-json bundle schema_version") && msg.contains("\"5\""),
            "expected a bundle-version rejection naming \"5\"; got {msg}"
        );
    }

    /// v0.27.0 Phase 6.5 PR-review I8: drift regression — serialize a fully
    /// populated `BundleJson` and re-parse it via `BundleJsonView`. The
    /// assertion that the parse succeeds + each typed field round-trips is
    /// what catches drift if `BundleJson` gains, renames, or retypes a field
    /// without a matching `BundleJsonView` update. (Compile alone is not
    /// enough — `BundleJson` is Serialize-only and Serde tolerates unknown
    /// fields by default on `BundleJsonView`'s side.)
    #[test]
    fn bundle_json_view_round_trips_every_field_of_bundle_json() {
        use crate::format::{BundleJson, CosignerEntry, MkField, MultisigInfo};

        let src = BundleJson {
            schema_version: "4",
            mode: "watch-only",
            network: "mainnet",
            template: Some("multisig"),
            descriptor: Some("wsh(sortedmulti(2,@0,@1))#csum".to_string()),
            account: 7,
            origin_path: Some("m/48'/0'/7'/2'".to_string()),
            origin_paths: Some(vec![
                "m/48'/0'/0'/2'".to_string(),
                "m/48'/0'/1'/2'".to_string(),
            ]),
            master_fingerprint: Some("deadbeef".to_string()),
            ms1: vec!["ms1abc".to_string(), "".to_string()],
            mk1: MkField::Multi(vec![
                vec!["a0".to_string(), "a1".to_string()],
                vec!["b0".to_string(), "b1".to_string()],
            ]),
            md1: vec!["md1xyz".to_string()],
            multisig: Some(MultisigInfo {
                template: "sortedmulti",
                threshold: 2,
                cosigner_count: 2,
                path_family: "bip48",
                cosigners: vec![
                    CosignerEntry {
                        index: 0,
                        master_fingerprint: Some("11111111".to_string()),
                        origin_path: "m/48'/0'/0'/2'".to_string(),
                        xpub: "xpub6A".to_string(),
                    },
                    CosignerEntry {
                        index: 1,
                        master_fingerprint: Some("22222222".to_string()),
                        origin_path: "m/48'/0'/1'/2'".to_string(),
                        xpub: "xpub6B".to_string(),
                    },
                ],
            }),
            privacy_preserving: false,
        };

        let wire = serde_json::to_string(&src).expect("BundleJson serialize");
        let v: BundleJsonView = serde_json::from_str(&wire).expect("BundleJsonView re-parse");

        assert_eq!(v.schema_version, "4");
        assert_eq!(v.mode, "watch-only");
        assert_eq!(v.network, "mainnet");
        assert_eq!(v.template.as_deref(), Some("multisig"));
        assert_eq!(
            v.descriptor.as_deref(),
            Some("wsh(sortedmulti(2,@0,@1))#csum")
        );
        assert_eq!(v.account, 7);
        assert_eq!(v.origin_path.as_deref(), Some("m/48'/0'/7'/2'"));
        assert_eq!(
            v.origin_paths.as_deref(),
            Some(["m/48'/0'/0'/2'".to_string(), "m/48'/0'/1'/2'".to_string()].as_slice())
        );
        assert_eq!(v.master_fingerprint.as_deref(), Some("deadbeef"));
        assert_eq!(v.ms1, vec!["ms1abc".to_string(), "".to_string()]);
        assert_eq!(v.mk1.len(), 2, "Multi-form outer length");
        assert_eq!(v.mk1[0], vec!["a0".to_string(), "a1".to_string()]);
        assert_eq!(v.mk1[1], vec!["b0".to_string(), "b1".to_string()]);
        assert_eq!(v.md1, vec!["md1xyz".to_string()]);
        let m = v.multisig.expect("multisig view present");
        assert_eq!(m.template, "sortedmulti");
        assert_eq!(m.threshold, 2);
        assert_eq!(m.cosigner_count, 2);
        assert_eq!(m.path_family, "bip48");
        assert_eq!(m.cosigners.len(), 2);
        assert_eq!(m.cosigners[0].index, 0);
        assert_eq!(
            m.cosigners[0].master_fingerprint.as_deref(),
            Some("11111111")
        );
        assert_eq!(m.cosigners[0].origin_path, "m/48'/0'/0'/2'");
        assert_eq!(m.cosigners[0].xpub, "xpub6A");
        assert_eq!(m.cosigners[1].index, 1);
        assert!(!v.privacy_preserving);
    }

    /// `parse_import_json_envelopes` errors on out-of-range index.
    #[test]
    fn parse_import_json_envelopes_out_of_range_index_errors() {
        let raw = r#"[
            {"schema_version":"1","source_format":"bsms","bundle":{
                "schema_version":"4","mode":"watch-only","network":"mainnet",
                "template":null,"descriptor":"wpkh(@0)","account":0,
                "origin_path":"m","origin_paths":null,"master_fingerprint":null,
                "ms1":[""],"mk1":["a"],"md1":["m1"],"multisig":null,
                "privacy_preserving":false}}
        ]"#;
        let err = parse_import_json_envelopes(raw, Some(5), "--import-json").unwrap_err();
        assert!(format!("{err:?}").contains("out of range"));
    }

    /// v0.27.1 Phase 2 I5 fold — when `KeyCard.origin_fingerprint == None`,
    /// `mk1_card_to_resolved_slot` substitutes `card.xpub.fingerprint()` BUT
    /// emits a stderr NOTICE naming the slot index + the substituted hex.
    /// Closes the prior `let _ = slot_idx; // reserved` silent-substitution
    /// gap. Master-fp and current-xpub-fp are semantically distinct.
    #[test]
    fn mk1_card_to_resolved_slot_missing_origin_fingerprint_emits_notice() {
        use std::str::FromStr;
        // Synthetic KeyCard with origin_fingerprint = None — exercises the
        // fallback arm.
        let xpub = bitcoin::bip32::Xpub::from_str("xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX").unwrap();
        let card = mk_codec::KeyCard::new(
            vec![[0u8; 4]],
            None, // <-- origin_fingerprint absent
            bitcoin::bip32::DerivationPath::from_str("m/48'/0'/0'/2'").unwrap(),
            xpub,
        );
        let mut stderr: Vec<u8> = Vec::new();
        let slot = mk1_card_to_resolved_slot(&card, 7, &mut stderr).unwrap();
        let stderr_str = String::from_utf8(stderr).unwrap();
        assert!(
            stderr_str.contains("mk1[7]:"),
            "NOTICE must name the slot index; got: {stderr_str}"
        );
        assert!(
            stderr_str.contains("origin_fingerprint absent"),
            "NOTICE must explain the substitution; got: {stderr_str}"
        );
        assert!(
            stderr_str.contains("mismatched origins"),
            "NOTICE must warn about downstream wallets; got: {stderr_str}"
        );
        // Resolved fingerprint equals xpub-derived.
        assert_eq!(slot.fingerprint, xpub.fingerprint());
    }

    /// Regression guard: present `origin_fingerprint` does NOT emit a NOTICE.
    #[test]
    fn mk1_card_to_resolved_slot_present_origin_fingerprint_silent() {
        use std::str::FromStr;
        let xpub = bitcoin::bip32::Xpub::from_str("xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX").unwrap();
        let fp = bitcoin::bip32::Fingerprint::from([0xde, 0xad, 0xbe, 0xef]);
        let card = mk_codec::KeyCard::new(
            vec![[0u8; 4]],
            Some(fp),
            bitcoin::bip32::DerivationPath::from_str("m/48'/0'/0'/2'").unwrap(),
            xpub,
        );
        let mut stderr: Vec<u8> = Vec::new();
        let slot = mk1_card_to_resolved_slot(&card, 0, &mut stderr).unwrap();
        let stderr_str = String::from_utf8(stderr).unwrap();
        assert!(
            stderr_str.is_empty(),
            "present fingerprint must emit no NOTICE; got: {stderr_str}"
        );
        assert_eq!(slot.fingerprint, fp);
    }

    // ========================================================================
    // v0.37.8 — universal source-name lift unit tests. 7 cells total: 6
    // per-format positive cells (one per name-carrying format) + 1 negative
    // cell (no metadata → None). Each builds a minimal `ImportJsonEnvelope`
    // with the one populated metadata field that exercises the
    // `resolved_wallet_name` walker for its format. The 8th name-carrying
    // CLI format (coldcard singlesig) has no `name` field — explicitly
    // omitted from scope per SPEC §C3.
    // ========================================================================

    /// Build a minimal `ImportJsonEnvelope` with no per-format metadata
    /// populated. Bundle is a placeholder — `resolved_wallet_name` only
    /// inspects the six `*_source_metadata` fields.
    fn empty_envelope_for_name_lift_test() -> ImportJsonEnvelope {
        ImportJsonEnvelope {
            schema_version: "1".to_string(),
            source_format: "test".to_string(),
            bundle: BundleJsonView {
                schema_version: "4".to_string(),
                mode: "watch-only".to_string(),
                network: "mainnet".to_string(),
                template: None,
                descriptor: None,
                account: 0,
                origin_path: None,
                origin_paths: None,
                master_fingerprint: None,
                ms1: vec![],
                mk1: vec![],
                md1: vec![],
                multisig: None,
                privacy_preserving: false,
            },
            source_metadata: None,
            sparrow_source_metadata: None,
            specter_source_metadata: None,
            jade_source_metadata: None,
            electrum_source_metadata: None,
            coldcard_multisig_source_metadata: None,
        }
    }

    /// Unit cell 1/7 — no per-format metadata populated ⇒ no name lift.
    /// Guards against accidental fall-through fabrication.
    #[test]
    fn resolved_wallet_name_returns_none_when_no_source_metadata_populated() {
        let env = empty_envelope_for_name_lift_test();
        assert_eq!(env.resolved_wallet_name(), None);
    }

    /// Unit cell 2/7 — sparrow projection lifts the top-level `label` key.
    #[test]
    fn resolved_wallet_name_lifts_sparrow_label() {
        let mut env = empty_envelope_for_name_lift_test();
        env.sparrow_source_metadata = Some(serde_json::json!({
            "label": "wsh-sortedmulti-0",
            "policy_type": "MULTI",
            "script_type": "P2WSH",
            "dropped_fields": []
        }));
        assert_eq!(
            env.resolved_wallet_name(),
            Some("wsh-sortedmulti-0".to_string())
        );
    }

    /// Unit cell 3/7 — specter projection lifts the top-level `label` key.
    #[test]
    fn resolved_wallet_name_lifts_specter_label() {
        let mut env = empty_envelope_for_name_lift_test();
        env.specter_source_metadata = Some(serde_json::json!({
            "label": "VaultColdStorage",
            "blockheight": 750000,
            "devices": [],
            "dropped_fields": []
        }));
        assert_eq!(
            env.resolved_wallet_name(),
            Some("VaultColdStorage".to_string())
        );
    }

    /// Unit cell 4/7 — jade projection lifts the NESTED
    /// `coldcard_compat.name` key (not the top-level `name`). This cell
    /// guards `walk_str`'s multi-step path traversal end-to-end.
    #[test]
    fn resolved_wallet_name_lifts_jade_nested_coldcard_compat_name() {
        let mut env = empty_envelope_for_name_lift_test();
        env.jade_source_metadata = Some(serde_json::json!({
            "coldcard_compat": {
                "name": "TestMs2of3",
                "policy_k": 2,
                "policy_n": 3,
                "script_format": "P2WSH",
                "xfp_was_blob_supplied": true,
                "xfp_header_disagreed": false,
                "dropped_fields": []
            },
            "jade_specific_fields": []
        }));
        assert_eq!(env.resolved_wallet_name(), Some("TestMs2of3".to_string()));
    }

    /// Unit cell 5/7 — electrum projection lifts the top-level
    /// `wallet_name` key.
    #[test]
    fn resolved_wallet_name_lifts_electrum_wallet_name() {
        let mut env = empty_envelope_for_name_lift_test();
        env.electrum_source_metadata = Some(serde_json::json!({
            "seed_version": 17,
            "wallet_type": "standard",
            "wallet_name": "Daily",
            "dropped_fields": []
        }));
        assert_eq!(env.resolved_wallet_name(), Some("Daily".to_string()));
    }

    /// Unit cell 6/7 — bitcoin-core projection (the `source_metadata` key
    /// without a per-format prefix) lifts the top-level `wallet_name`.
    #[test]
    fn resolved_wallet_name_lifts_bitcoin_core_wallet_name() {
        let mut env = empty_envelope_for_name_lift_test();
        env.source_metadata = Some(serde_json::json!({
            "wallet_name": "bip84_mainnet",
            "active": true,
            "internal": false,
            "range": [0, 1000],
            "dropped_fields": []
        }));
        assert_eq!(
            env.resolved_wallet_name(),
            Some("bip84_mainnet".to_string())
        );
    }

    /// Unit cell 7/7 — coldcard-multisig projection lifts top-level `name`.
    /// Mirrors `coldcard_compat.name` shape minus the wrapper key (Jade
    /// nests it; coldcard-multisig is direct).
    #[test]
    fn resolved_wallet_name_lifts_coldcard_multisig_name() {
        let mut env = empty_envelope_for_name_lift_test();
        env.coldcard_multisig_source_metadata = Some(serde_json::json!({
            "name": "TestMs2of3",
            "policy_k": 2,
            "policy_n": 3,
            "script_format": "P2WSH",
            "xfp_was_blob_supplied": true,
            "xfp_header_disagreed": false,
            "dropped_fields": []
        }));
        assert_eq!(env.resolved_wallet_name(), Some("TestMs2of3".to_string()));
    }

    /// Sub-cell — `walk_str` returns None when an intermediate key is
    /// missing. Guards against fabricated leaves on partial envelopes
    /// (e.g., a jade envelope that omits `coldcard_compat` wrapper).
    #[test]
    fn walk_str_returns_none_on_missing_intermediate_key() {
        let v = serde_json::json!({"a": {"b": "leaf"}});
        assert_eq!(walk_str(&v, &["a", "b"]), Some("leaf"));
        assert_eq!(walk_str(&v, &["a", "c"]), None);
        assert_eq!(walk_str(&v, &["missing"]), None);
        // Non-string leaf returns None (matches the `walk_str` contract).
        let v2 = serde_json::json!({"a": {"b": 42}});
        assert_eq!(walk_str(&v2, &["a", "b"]), None);
    }

    /// End-of-cycle R0 M1 fold — defensive cell pinning the `!name.
    /// is_empty()` filter at `resolved_wallet_name`. If a future
    /// emitter (or a hand-crafted envelope) populates a per-format
    /// metadata field with a literal empty string, the lift must
    /// behave as if the field is absent — falling back to the
    /// `imported-descriptor` default rather than emitting an
    /// empty-string wallet name.
    #[test]
    fn resolved_wallet_name_returns_none_on_empty_string_leaf() {
        let mut env = empty_envelope_for_name_lift_test();
        env.sparrow_source_metadata = Some(serde_json::json!({
            "label": "",
            "policy_type": "MULTI",
            "script_type": "P2WSH",
            "dropped_fields": []
        }));
        assert_eq!(
            env.resolved_wallet_name(),
            None,
            "empty-string leaf must NOT lift; falls through to default"
        );
    }
}
