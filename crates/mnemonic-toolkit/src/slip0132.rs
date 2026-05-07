//! SPEC v0.6.1 §11 / §11.a — SLIP-0132 prefix-tolerant input + `--xpub-prefix` output.
//!
//! Decode-swap-reencode at the `bitcoin::base58::decode_check` layer; key
//! material is unchanged. See `design/agent-reports/spike-slip0132-v0_6_1-pre-spec.md`
//! for the bitcoin-0.32 surface verification.

use crate::error::ToolkitError;
use crate::network::CliNetwork;
use bitcoin::base58;
use bitcoin::bip32::Xpub;

/// SPEC §11.a flag values: 5-variant SLIP-0132 *semantic class* selector
/// (BIP-49 single, BIP-49 multisig, BIP-84 single, BIP-84 multisig, neutral).
/// The actual prefix bytes are network-dependent; mainnet vs testnet is
/// resolved via `--network` at swap time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XpubPrefix {
    /// BIP-32 neutral — `xpub` (mainnet) / `tpub` (testnet). Default.
    Xpub,
    /// BIP-49 single-sig — `ypub` (mainnet) / `upub` (testnet).
    Ypub,
    /// BIP-49 multisig (P2SH-P2WSH) — `Ypub` (mainnet) / `Upub` (testnet).
    YpubMultisig,
    /// BIP-84 single-sig — `zpub` (mainnet) / `vpub` (testnet).
    Zpub,
    /// BIP-84 multisig (P2WSH) — `Zpub` (mainnet) / `Vpub` (testnet).
    ZpubMultisig,
}

impl XpubPrefix {
    pub fn is_default(self) -> bool {
        matches!(self, XpubPrefix::Xpub)
    }
}

/// Custom clap `value_parser` accepting the 5 SPEC §11.a flag-value strings
/// case-sensitively (`xpub`, `ypub`, `Ypub`, `zpub`, `Zpub`).
pub fn parse_xpub_prefix_arg(s: &str) -> Result<XpubPrefix, String> {
    Ok(match s {
        "xpub" => XpubPrefix::Xpub,
        "ypub" => XpubPrefix::Ypub,
        "Ypub" => XpubPrefix::YpubMultisig,
        "zpub" => XpubPrefix::Zpub,
        "Zpub" => XpubPrefix::ZpubMultisig,
        _ => {
            return Err(format!(
                "--xpub-prefix value {s:?} not in {{xpub, ypub, Ypub, zpub, Zpub}}"
            ))
        }
    })
}

/// Mainnet → swap target table for `Xpub::decode` neutralization.
const SWAP_TO_XPUB_MAINNET: [u8; 4] = [0x04, 0x88, 0xB2, 0x1E];
/// Testnet → swap target table for `Xpub::decode` neutralization.
const SWAP_TO_TPUB_TESTNET: [u8; 4] = [0x04, 0x35, 0x87, 0xCF];

/// SPEC §11 — accept SLIP-0132 prefix variants on input by decode-swap-reencode
/// down to BIP-32 neutral `xpub` / `tpub`. Returns the input unchanged when it
/// is already neutral. Returns `BadInput` (exit 1) for unknown prefixes.
///
/// The `Option<&'static str>` second element is the variant-name signal: `None`
/// when the input was already neutral, otherwise the SLIP-0132 prefix string
/// (`"ypub"`, `"Ypub"`, `"zpub"`, `"Zpub"`, `"upub"`, `"Upub"`, `"vpub"`, `"Vpub"`)
/// that was swapped out. Phase 3 surfaces this on stderr.
pub(crate) fn normalize_xpub_prefix(
    s: &str,
) -> Result<(String, Option<&'static str>), ToolkitError> {
    let raw = base58::decode_check(s)
        .map_err(|e| ToolkitError::BadInput(format!("base58check decode: {e}")))?;
    if raw.len() != 78 {
        return Err(ToolkitError::BadInput(format!(
            "extended-key serialization is 78 bytes; got {}",
            raw.len()
        )));
    }
    let prefix: [u8; 4] = raw[0..4].try_into().expect("78 bytes guarantees 4-byte prefix");
    let (neutral, variant): ([u8; 4], &'static str) = match prefix {
        // already neutral — pass through with None signal
        SWAP_TO_XPUB_MAINNET | SWAP_TO_TPUB_TESTNET => return Ok((s.to_string(), None)),
        // SLIP-0132 mainnet → xpub
        [0x04, 0x9D, 0x7C, 0xB2] => (SWAP_TO_XPUB_MAINNET, "ypub"),
        [0x02, 0x95, 0xB4, 0x3F] => (SWAP_TO_XPUB_MAINNET, "Ypub"),
        [0x04, 0xB2, 0x47, 0x46] => (SWAP_TO_XPUB_MAINNET, "zpub"),
        [0x02, 0xAA, 0x7E, 0xD3] => (SWAP_TO_XPUB_MAINNET, "Zpub"),
        // SLIP-0132 testnet → tpub
        [0x04, 0x4A, 0x52, 0x62] => (SWAP_TO_TPUB_TESTNET, "upub"),
        [0x02, 0x42, 0x89, 0xEF] => (SWAP_TO_TPUB_TESTNET, "Upub"),
        [0x04, 0x5F, 0x1C, 0xF6] => (SWAP_TO_TPUB_TESTNET, "vpub"),
        [0x02, 0x57, 0x54, 0x83] => (SWAP_TO_TPUB_TESTNET, "Vpub"),
        _ => {
            return Err(ToolkitError::BadInput(format!(
                "unknown extended-key version prefix: {:02x}{:02x}{:02x}{:02x}",
                prefix[0], prefix[1], prefix[2], prefix[3]
            )))
        }
    };
    let mut swapped = raw.clone();
    swapped[0..4].copy_from_slice(&neutral);
    Ok((base58::encode_check(&swapped), Some(variant)))
}

/// SPEC §11.a — emit `xpub` with a SLIP-0132 (or neutral) version prefix
/// selected by `variant` + `network`. Operates on the 78-byte raw
/// serialization (same primitive as `normalize_xpub_prefix`).
pub(crate) fn apply_xpub_prefix(
    xpub: &Xpub,
    variant: XpubPrefix,
    network: CliNetwork,
) -> String {
    let mut raw = xpub.encode();
    raw[0..4].copy_from_slice(&swap_target_for(variant, network));
    base58::encode_check(&raw)
}

/// Map a SLIP-0132 variant name (as produced by `normalize_xpub_prefix`'s
/// `Option<&'static str>` channel) to its BIP-32 neutral counterpart.
///
/// Variant determines neutral per SPEC §11/§11.b: mainnet variants
/// (`ypub | Ypub | zpub | Zpub`) → `xpub`; testnet variants
/// (`upub | Upub | vpub | Vpub`) → `tpub`.
pub(crate) fn neutral_for(variant: &'static str) -> &'static str {
    match variant {
        "ypub" | "Ypub" | "zpub" | "Zpub" => "xpub",
        "upub" | "Upub" | "vpub" | "Vpub" => "tpub",
        _ => unreachable!("neutral_for: unknown variant {variant:?} — must be one of the 8 produced by normalize_xpub_prefix"),
    }
}

/// Render the canonical SPEC §5.5.a SLIP-0132 input-normalization info-line
/// for a recognized variant. The variant determines the neutral form via
/// `neutral_for`. Returns the line text WITHOUT a trailing newline (callers
/// add one via `writeln!`).
pub(crate) fn render_slip0132_info_line(variant: &'static str) -> String {
    format!(
        "info: normalized {variant} input to neutral {neutral} (encoding-only; no key change). Re-emit with --xpub-prefix {variant} if you need the SLIP-0132 form.",
        neutral = neutral_for(variant),
    )
}

fn swap_target_for(variant: XpubPrefix, network: CliNetwork) -> [u8; 4] {
    let mainnet = matches!(network, CliNetwork::Mainnet);
    match (variant, mainnet) {
        (XpubPrefix::Xpub, true) => SWAP_TO_XPUB_MAINNET,
        (XpubPrefix::Xpub, false) => SWAP_TO_TPUB_TESTNET,
        (XpubPrefix::Ypub, true) => [0x04, 0x9D, 0x7C, 0xB2],
        (XpubPrefix::Ypub, false) => [0x04, 0x4A, 0x52, 0x62],
        (XpubPrefix::YpubMultisig, true) => [0x02, 0x95, 0xB4, 0x3F],
        (XpubPrefix::YpubMultisig, false) => [0x02, 0x42, 0x89, 0xEF],
        (XpubPrefix::Zpub, true) => [0x04, 0xB2, 0x47, 0x46],
        (XpubPrefix::Zpub, false) => [0x04, 0x5F, 0x1C, 0xF6],
        (XpubPrefix::ZpubMultisig, true) => [0x02, 0xAA, 0x7E, 0xD3],
        (XpubPrefix::ZpubMultisig, false) => [0x02, 0x57, 0x54, 0x83],
    }
}

// ============================================================================
// Tests
// ============================================================================
#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    /// BIP-84 reference vector (https://github.com/bitcoin/bips/blob/master/bip-0084.mediawiki):
    /// mnemonic "abandon abandon abandon abandon abandon abandon abandon abandon abandon
    /// abandon abandon about" + account 0 + m/84'/0'/0' →
    const BIP84_REF_ZPUB: &str = "zpub6rFR7y4Q2AijBEqTUquhVz398htDFrtymD9xYYfG1m4wAcvPhXNfE3EfH1r1ADqtfSdVCToUG868RvUUkgDKf31mGDtKsAYz2oz2AGutZYs";
    /// Equivalent neutral xpub for the same account-level key (computed by
    /// decode-swap-reencode against the spec's zpub).
    const BIP84_REF_XPUB: &str = "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V";

    #[test]
    fn normalize_passes_neutral_xpub_through_unchanged() {
        let (out, sig) = normalize_xpub_prefix(BIP84_REF_XPUB).unwrap();
        assert_eq!(out, BIP84_REF_XPUB);
        assert!(sig.is_none());
    }

    #[test]
    fn normalize_swaps_zpub_to_xpub() {
        let (out, sig) = normalize_xpub_prefix(BIP84_REF_ZPUB).unwrap();
        assert_eq!(out, BIP84_REF_XPUB);
        assert_eq!(sig, Some("zpub"));
    }

    #[test]
    fn normalize_round_trip_xpub_to_zpub_to_xpub() {
        let xpub = Xpub::from_str(BIP84_REF_XPUB).unwrap();
        let zpub_out = apply_xpub_prefix(&xpub, XpubPrefix::Zpub, CliNetwork::Mainnet);
        assert_eq!(zpub_out, BIP84_REF_ZPUB);
        let (neutral, sig) = normalize_xpub_prefix(&zpub_out).unwrap();
        assert_eq!(neutral, BIP84_REF_XPUB);
        assert_eq!(sig, Some("zpub"));
    }

    #[test]
    fn normalize_rejects_unknown_prefix() {
        // Construct a random base58check-valid blob with a bogus version prefix.
        let mut raw = [0u8; 78];
        raw[0..4].copy_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF]);
        let bogus = base58::encode_check(&raw);
        let err = normalize_xpub_prefix(&bogus).unwrap_err();
        assert!(
            matches!(&err, ToolkitError::BadInput(m) if m.contains("unknown extended-key version prefix: ffffffff")),
            "got {err:?}"
        );
    }

    #[test]
    fn normalize_rejects_short_payload() {
        // base58check-valid but only 10 bytes (not a 78-byte extended key).
        let short = base58::encode_check(&[0u8; 10]);
        let err = normalize_xpub_prefix(&short).unwrap_err();
        assert!(
            matches!(&err, ToolkitError::BadInput(m) if m.contains("78 bytes")),
            "got {err:?}"
        );
    }

    #[test]
    fn apply_emits_all_5_mainnet_variants() {
        let xpub = Xpub::from_str(BIP84_REF_XPUB).unwrap();
        let xpub_out = apply_xpub_prefix(&xpub, XpubPrefix::Xpub, CliNetwork::Mainnet);
        let ypub_out = apply_xpub_prefix(&xpub, XpubPrefix::Ypub, CliNetwork::Mainnet);
        let big_y_out = apply_xpub_prefix(&xpub, XpubPrefix::YpubMultisig, CliNetwork::Mainnet);
        let zpub_out = apply_xpub_prefix(&xpub, XpubPrefix::Zpub, CliNetwork::Mainnet);
        let big_z_out = apply_xpub_prefix(&xpub, XpubPrefix::ZpubMultisig, CliNetwork::Mainnet);
        assert!(xpub_out.starts_with("xpub"));
        assert!(ypub_out.starts_with("ypub"));
        assert!(big_y_out.starts_with("Ypub"));
        assert!(zpub_out.starts_with("zpub"));
        assert!(big_z_out.starts_with("Zpub"));
        // All decode back to the same neutral xpub, with variant-name signal
        // matching the prefix that was swapped out.
        for (variant_out, expected_sig) in &[
            (ypub_out, "ypub"),
            (big_y_out, "Ypub"),
            (zpub_out, "zpub"),
            (big_z_out, "Zpub"),
        ] {
            let (neutral, sig) = normalize_xpub_prefix(variant_out).unwrap();
            assert_eq!(neutral, BIP84_REF_XPUB);
            assert_eq!(sig, Some(*expected_sig));
        }
    }

    #[test]
    fn apply_testnet_variants_swap_to_lowercase_t_class_prefixes() {
        let xpub = Xpub::from_str(BIP84_REF_XPUB).unwrap();
        // Round-trip via testnet swap: even with a mainnet-derived xpub, the
        // version-byte swap is purely encoding; produces wire-valid testnet-prefixed
        // strings (the network-mismatch is the user's responsibility per §11).
        let vpub_out = apply_xpub_prefix(&xpub, XpubPrefix::Zpub, CliNetwork::Testnet);
        assert!(vpub_out.starts_with("vpub"));
        let big_v_out = apply_xpub_prefix(&xpub, XpubPrefix::ZpubMultisig, CliNetwork::Testnet);
        assert!(big_v_out.starts_with("Vpub"));
        let upub_out = apply_xpub_prefix(&xpub, XpubPrefix::Ypub, CliNetwork::Testnet);
        assert!(upub_out.starts_with("upub"));
        let big_u_out = apply_xpub_prefix(&xpub, XpubPrefix::YpubMultisig, CliNetwork::Testnet);
        assert!(big_u_out.starts_with("Upub"));
    }

    #[test]
    fn parse_xpub_prefix_arg_accepts_5_documented_values() {
        assert_eq!(parse_xpub_prefix_arg("xpub").unwrap(), XpubPrefix::Xpub);
        assert_eq!(parse_xpub_prefix_arg("ypub").unwrap(), XpubPrefix::Ypub);
        assert_eq!(parse_xpub_prefix_arg("Ypub").unwrap(), XpubPrefix::YpubMultisig);
        assert_eq!(parse_xpub_prefix_arg("zpub").unwrap(), XpubPrefix::Zpub);
        assert_eq!(parse_xpub_prefix_arg("Zpub").unwrap(), XpubPrefix::ZpubMultisig);
    }

    #[test]
    fn render_slip0132_info_line_pins_canonical_text() {
        // Pin the exact byte sequence the SPEC §5.5.a / SPEC convert §11 mandates.
        // If this test changes, both production sites and the test-side info_line
        // helpers in tests/cli_*_slip0132_info.rs must update in lockstep.
        assert_eq!(
            render_slip0132_info_line("zpub"),
            "info: normalized zpub input to neutral xpub (encoding-only; no key change). Re-emit with --xpub-prefix zpub if you need the SLIP-0132 form.",
        );
        assert_eq!(
            render_slip0132_info_line("Vpub"),
            "info: normalized Vpub input to neutral tpub (encoding-only; no key change). Re-emit with --xpub-prefix Vpub if you need the SLIP-0132 form.",
        );
    }

    #[test]
    fn neutral_for_maps_all_8_variants() {
        assert_eq!(neutral_for("ypub"), "xpub");
        assert_eq!(neutral_for("Ypub"), "xpub");
        assert_eq!(neutral_for("zpub"), "xpub");
        assert_eq!(neutral_for("Zpub"), "xpub");
        assert_eq!(neutral_for("upub"), "tpub");
        assert_eq!(neutral_for("Upub"), "tpub");
        assert_eq!(neutral_for("vpub"), "tpub");
        assert_eq!(neutral_for("Vpub"), "tpub");
    }

    #[test]
    #[should_panic(expected = "unknown variant")]
    fn neutral_for_panics_on_unknown_variant() {
        let _ = neutral_for("xpub");
    }

    #[test]
    fn parse_xpub_prefix_arg_rejects_other_values_including_testnet_strings() {
        // Testnet variants are NOT exposed as flag values per SPEC §11.a.
        for bad in &["upub", "Upub", "vpub", "Vpub", "tpub", "XPUB", "", "y"] {
            let err = parse_xpub_prefix_arg(bad).unwrap_err();
            assert!(err.contains("not in"), "got {err:?} for {bad:?}");
        }
    }
}
