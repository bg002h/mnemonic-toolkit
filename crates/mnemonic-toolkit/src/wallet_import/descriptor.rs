//! C5 — generic commented-descriptor import parser (`import-wallet --format descriptor`).
//!
//! Reads a watch-only concrete descriptor from a text file, tolerating leading
//! `#`-comment lines + blank lines (so it subsumes `export-wallet --format green`'s
//! 3-line output AND `--format descriptor`'s bare line AND any hand-written /
//! foreign commented descriptor). The descriptor flows through the SAME
//! concrete-keys pipeline the other foreign-format parsers use
//! (`concrete_keys_to_placeholders` → `parse_descriptor` → origin slots), so
//! singlesig AND multisig are both supported (a descriptor carries everything).
//!
//! Explicit-only: `sniff` always returns `false` and the parser is NOT wired into
//! `sniff::sniff_format`'s votes — a bare descriptor is too generic to auto-detect
//! safely, so `--format descriptor` is REQUIRED (mirrors BSMS-encrypted).
//!
//! BIP-380 checksum is TOLERANT: validated if present (a bad checksum is refused),
//! tolerated if absent — matching `bundle --descriptor`/`verify-bundle`.

use super::{
    pipeline::concrete_keys_to_placeholders, validate_watch_only_resolved, ImportProvenance,
    ParsedImport, WalletFormatParser,
};
use crate::error::ToolkitError;
use crate::parse_descriptor;
use crate::synthesize::{xpub_to_65, ResolvedSlot};
use bitcoin::bip32::{ChildNumber, DerivationPath, Fingerprint, Xpub};
use regex::Regex;
use std::io::Write;
use std::sync::OnceLock;

/// C5 — `import-wallet --format descriptor` parser.
pub(crate) struct DescriptorParser;

impl WalletFormatParser for DescriptorParser {
    /// Explicit-only: a bare descriptor is too generic to auto-sniff (it would
    /// collide with any descriptor-bearing text). NEVER auto-detected; the
    /// parser is intentionally absent from `sniff::sniff_format`'s votes array.
    fn sniff(_blob: &[u8]) -> bool {
        false
    }

    fn parse(blob: &[u8], _stderr: &mut dyn Write) -> Result<Vec<ParsedImport>, ToolkitError> {
        // Step 1: strip leading `#`-comment lines + blanks → the single
        // descriptor line. The descriptor's own mid-line `#<checksum>` is NOT a
        // full-line comment (the line starts with `wsh`/`wpkh`/...), so it is
        // preserved.
        let descriptor_str = strip_comments(blob)?;

        // Step 2: validate BIP-380 checksum on the ORIGINAL body (concrete
        // `[fp/path]xpub` keys present). TOLERANT — `verify_checksum`
        // validates-if-present, tolerates-absence (mirrors `bundle --descriptor`).
        // Returns the body sans `#<checksum>` suffix for the placeholder adapter.
        let descriptor_body_no_csum =
            miniscript::descriptor::checksum::verify_checksum(&descriptor_str).map_err(|e| {
                ToolkitError::ImportWalletParse(format!(
                "import-wallet: descriptor: parse error: BIP-380 checksum validation failed: {e}"
            ))
            })?;

        // Step 3: concrete-keys pipeline (placeholder substitution → md1 tree).
        let (placeholder_form, parsed_keys, parsed_fingerprints) =
            concrete_keys_to_placeholders(descriptor_body_no_csum).map_err(|e| {
                ToolkitError::ImportWalletParse(e.message().replacen(
                    "import-wallet: bsms:",
                    "import-wallet: descriptor:",
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
                "import-wallet: descriptor: parse error: {}",
                e.message()
            ))
        })?;

        // Step 4: build ResolvedSlot vec from the descriptor's key origins.
        let origins = crate::wallet_import::pipeline::extract_origin_components(
            &descriptor_str,
            "descriptor",
        )?;
        let network = network_from_origins(&origins)?;
        let mut cosigners: Vec<ResolvedSlot> = Vec::with_capacity(parsed_keys.len());
        for (i, _) in parsed_keys.iter().enumerate() {
            let (xpub, fp, path) = build_slot_fields(&descriptor_str, i)?;
            debug_assert_eq!(xpub_to_65(&xpub), parsed_keys[i].payload);
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

        // cycle-5 S-NET (axis 2 / H15): each decoded xpub's NetworkKind must
        // agree with the coin-type-derived network — rejects a hand-edited blob
        // carrying a tpub on a mainnet coin-type path (or vice-versa).
        crate::wallet_import::pipeline::assert_slots_network_agrees(
            &cosigners,
            network,
            "import: descriptor",
        )?;

        // Step 5: threshold (multisig only; singlesig → None).
        let threshold = extract_threshold_local(&descriptor_str)?;

        // Step 6: rebuild original_descriptor with a fresh BIP-380 checksum (byte
        // determinism); fall back to verbatim on engine failure.
        let original_descriptor = match recompute_descriptor_checksum(&descriptor_str) {
            Ok(s) => s,
            Err(_) => descriptor_str.clone(),
        };

        Ok(vec![ParsedImport {
            descriptor,
            original_descriptor,
            cosigners,
            network,
            threshold,
            provenance: ImportProvenance::Descriptor,
        }])
    }
}

/// Strip leading full-line `#`-comments + blank lines; require EXACTLY ONE
/// remaining non-comment line = the descriptor. A mid-line `#<checksum>` on the
/// descriptor line is preserved (the line does not START with `#`).
fn strip_comments(blob: &[u8]) -> Result<String, ToolkitError> {
    let text = std::str::from_utf8(blob).map_err(|e| {
        ToolkitError::BadInput(format!(
            "import-wallet: descriptor: input is not valid UTF-8: {e}"
        ))
    })?;
    let lines: Vec<&str> = text
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();
    match lines.as_slice() {
        [] => Err(ToolkitError::BadInput(
            "import-wallet: descriptor: no descriptor line found (input is only comments/blanks)"
                .to_string(),
        )),
        [one] => Ok((*one).to_string()),
        _ => Err(ToolkitError::BadInput(format!(
            "import-wallet: descriptor: expected a single descriptor line, found {} non-comment lines",
            lines.len()
        ))),
    }
}

/// Build the (xpub, fingerprint, path) slot for cosigner `slot_idx`.
/// Mirrors `specter::build_slot_fields`.
fn build_slot_fields(
    descriptor_body: &str,
    slot_idx: usize,
) -> Result<(Xpub, Fingerprint, DerivationPath), ToolkitError> {
    let origins =
        crate::wallet_import::pipeline::extract_origin_components(descriptor_body, "descriptor")?;
    let (fp, path, xpub_str) = origins.into_iter().nth(slot_idx).ok_or_else(|| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: descriptor: parse error: slot index {slot_idx} out of range"
        ))
    })?;
    crate::wallet_import::pipeline::finalize_slot_fields(fp, path, &xpub_str, "descriptor")
}

/// Infer network from the cosigners' BIP-48 coin-type. Mirrors
/// `specter::network_from_origins`.
fn network_from_origins(
    origins: &[(Fingerprint, DerivationPath, String)],
) -> Result<bitcoin::Network, ToolkitError> {
    if origins.is_empty() {
        return Err(ToolkitError::ImportWalletParse(
            "import-wallet: descriptor: parse error: no origins to infer network from".to_string(),
        ));
    }
    let coin_types: Vec<u32> = origins
        .iter()
        .map(|(_, p, _)| coin_type_from_path(p))
        .collect::<Result<Vec<_>, _>>()?;
    let first = coin_types[0];
    for (i, ct) in coin_types.iter().enumerate().skip(1) {
        if *ct != first {
            return Err(ToolkitError::ImportWalletParse(format!(
                "import-wallet: descriptor: cosigner {i} has coin-type {ct}, cosigner 0 has coin-type {first}; all cosigners must share a coin-type"
            )));
        }
    }
    match first {
        0 => Ok(bitcoin::Network::Bitcoin),
        1 => Ok(bitcoin::Network::Testnet),
        other => Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: descriptor: parse error: unsupported coin-type {other} on origin path; only 0 (mainnet) and 1 (testnet) supported per BIP-48"
        ))),
    }
}

/// BIP-48 coin-type = the 2nd path component (hardened). Mirrors
/// `specter::coin_type_from_path`.
fn coin_type_from_path(path: &DerivationPath) -> Result<u32, ToolkitError> {
    let comps: Vec<&ChildNumber> = path.into_iter().collect();
    if comps.len() < 2 {
        return Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: descriptor: parse error: origin path has only {} components; need ≥2 for BIP-48 coin-type inference",
            comps.len()
        )));
    }
    match comps[1] {
        ChildNumber::Hardened { index } => Ok(*index),
        ChildNumber::Normal { index } => Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: descriptor: parse error: coin-type component {index} is not hardened; BIP-48 requires `<coin_type>'`"
        ))),
    }
}

/// Extract K from `multi(K, ...)` / `sortedmulti(K, ...)`; `Ok(None)` for
/// singlesig. Mirrors `specter::extract_threshold_local`.
fn extract_threshold_local(descriptor_body: &str) -> Result<Option<u8>, ToolkitError> {
    static R: OnceLock<Regex> = OnceLock::new();
    let re = R.get_or_init(|| {
        Regex::new(r"(?:thresh|multi|sortedmulti|multi_a|sortedmulti_a)\((\d+)\s*,")
            .expect("threshold regex is fixed")
    });
    let cap = match re.captures(descriptor_body) {
        Some(c) => c,
        None => return Ok(None),
    };
    let arg = cap.get(1).expect("regex has capture group 1").as_str();
    arg.parse::<u8>().map(Some).map_err(|e| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: descriptor: parse error: multi argument `{arg}` exceeds u8 range (>255 cosigners not supported): {e}"
        ))
    })
}

/// Re-render with a fresh BIP-380 checksum. Mirrors
/// `specter::recompute_descriptor_checksum`.
fn recompute_descriptor_checksum(body: &str) -> Result<String, ToolkitError> {
    use miniscript::descriptor::checksum::Engine as ChecksumEngine;
    let body_no_csum = match body.rsplit_once('#') {
        Some((b, _)) => b,
        None => body,
    };
    let mut eng = ChecksumEngine::new();
    eng.input(body_no_csum).map_err(|e| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: descriptor: checksum engine input rejected: {e}"
        ))
    })?;
    let csum = eng.checksum();
    Ok(format!("{body_no_csum}#{csum}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_comments_extracts_the_descriptor_line() {
        // The green 3-line export shape: 2 `#`-comments + a descriptor.
        let blob = b"# Blockstream Green - Watch-only import (singlesig)\n# Help: https://example\nwpkh([5436d724/84'/0'/0']xpub6Bner.../0/*)#00lx6ere\n";
        let got = strip_comments(blob).expect("one descriptor line");
        assert!(got.starts_with("wpkh(") && got.contains("#00lx6ere"));
    }

    #[test]
    fn strip_comments_tolerates_blank_lines_and_trailing_ws() {
        let blob = b"\n#c\n\n   wsh(sortedmulti(2,a,b))#abcd   \n\n";
        assert_eq!(
            strip_comments(blob).unwrap(),
            "wsh(sortedmulti(2,a,b))#abcd"
        );
    }

    #[test]
    fn strip_comments_refuses_no_descriptor() {
        let blob = b"# only a comment\n\n# another\n";
        let err = strip_comments(blob).unwrap_err();
        assert!(err.to_string().contains("no descriptor line"));
    }

    #[test]
    fn strip_comments_refuses_two_descriptors() {
        let blob = b"wpkh(a)#1\nwpkh(b)#2\n";
        let err = strip_comments(blob).unwrap_err();
        assert!(err.to_string().contains("expected a single descriptor"));
    }

    #[test]
    fn sniff_is_always_false_explicit_only() {
        assert!(!DescriptorParser::sniff(
            b"wpkh([5436d724/84'/0'/0']xpub6Bner.../0/*)#00lx6ere"
        ));
    }
}
