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
//! - `envelope_to_resolved_slots(envelope) -> Vec<ResolvedSlot>` —
//!   decode mk1 chunks (single or multi) into `ResolvedSlot` values
//!   per §3.6.1.
//! - `mk1_card_to_resolved_slot(card, index) -> ResolvedSlot` —
//!   per-cosigner decode helper.
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
    // Phase 5 does NOT need `bsms_audit`, `source_metadata`, `roundtrip`,
    // or `bsms_round1_verifications` for the consumer paths; serde drops
    // unknown fields by default.
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
                    v.as_str().map(String::from).ok_or_else(|| {
                        D::Error::custom("mk1 flat-form must contain only strings")
                    })
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
    let envelopes: Vec<ImportJsonEnvelope> = serde_json::from_str(raw).map_err(|e| {
        ToolkitError::BadInput(format!("{flag_label}: envelope JSON parse: {e}"))
    })?;
    if envelopes.is_empty() {
        return Err(ToolkitError::BadInput(format!(
            "{flag_label}: envelope array is empty"
        )));
    }
    match index {
        Some(n) => {
            if n >= envelopes.len() {
                return Err(ToolkitError::BadInput(format!(
                    "{flag_label}-index {n} out of range; envelope array has {} entries",
                    envelopes.len()
                )));
            }
            Ok(envelopes.into_iter().nth(n).unwrap())
        }
        None => {
            if envelopes.len() > 1 {
                return Err(ToolkitError::BadInput(format!(
                    "{flag_label}: envelope array has {} entries; supply --import-json-index <N> \
                     (or --from-import-json-index <N> on export-wallet) to pick one",
                    envelopes.len()
                )));
            }
            Ok(envelopes.into_iter().next().unwrap())
        }
    }
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
pub(crate) fn envelope_to_resolved_slots(
    envelope: &ImportJsonEnvelope,
) -> Result<Vec<ResolvedSlot>, ToolkitError> {
    let mk1_outer = &envelope.bundle.mk1;
    let mut out = Vec::with_capacity(mk1_outer.len());
    for (i, chunks) in mk1_outer.iter().enumerate() {
        let chunk_refs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
        let card = mk_codec::decode(&chunk_refs).map_err(|e| {
            ToolkitError::BadInput(format!("--import-json: mk1[{i}] decode failed: {e}"))
        })?;
        out.push(mk1_card_to_resolved_slot(&card, i)?);
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
pub(crate) fn mk1_card_to_resolved_slot(
    card: &mk_codec::KeyCard,
    slot_idx: usize,
) -> Result<ResolvedSlot, ToolkitError> {
    let fingerprint = card
        .origin_fingerprint
        .unwrap_or_else(|| card.xpub.fingerprint());
    let path_raw = format!(
        "[{}/{}]",
        fingerprint.to_string().to_lowercase(),
        card.origin_path
            .to_string()
            .trim_start_matches("m/")
            .trim_start_matches('m'),
    );
    let _ = slot_idx; // reserved for future error-context attribution
    Ok(ResolvedSlot {
        xpub: card.xpub,
        fingerprint,
        path: card.origin_path.clone(),
        path_raw,
        entropy: None,
        master_xpub: None,
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
        stdin
            .read_to_string(&mut buf)
            .map_err(ToolkitError::Io)?;
        return Ok(buf);
    }
    std::fs::read_to_string(value).map_err(|e| {
        ToolkitError::BadInput(format!("{flag_label}: read {value}: {e}"))
    })
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
/// (form `m/48'/0'/0'/2'`). Used by the consumer paths for slot
/// path-equivalence + the `--ms1`-on-non-empty-slot conflict check.
#[allow(dead_code)] // surfaced for consumer-side path-equivalence tests; v0.27.0 consumers source from mk1 decode
pub(crate) fn derivation_path_from_envelope(
    s: &str,
    flag_label: &str,
) -> Result<DerivationPath, ToolkitError> {
    DerivationPath::from_str(s).map_err(|e| {
        ToolkitError::BadInput(format!("{flag_label}: origin_path parse {s:?}: {e}"))
    })
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
        assert_eq!(cli_network_from_str("mainnet").unwrap(), CliNetwork::Mainnet);
        assert_eq!(cli_network_from_str("testnet").unwrap(), CliNetwork::Testnet);
        assert_eq!(cli_network_from_str("signet").unwrap(), CliNetwork::Signet);
        assert_eq!(cli_network_from_str("regtest").unwrap(), CliNetwork::Regtest);
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
            origin_paths: Some(vec!["m/48'/0'/0'/2'".to_string(), "m/48'/0'/1'/2'".to_string()]),
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
        assert_eq!(m.cosigners[0].master_fingerprint.as_deref(), Some("11111111"));
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
}
