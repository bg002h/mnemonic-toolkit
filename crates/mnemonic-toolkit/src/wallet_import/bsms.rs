//! BIP-129 BSMS Round-2 parser.
//!
//! Per `design/SPEC_wallet_import_v0_26_0.md` §4. Accepts two lenient shapes:
//! - 2-line: `BSMS 1.0\n<descriptor>#<checksum>` (kickoff seed-case form;
//!   stderr WARNING about reduced form).
//! - 6-line: `BSMS 1.0\n<TOKEN>\n<descriptor>#<checksum>\n<DERIVATION_PATH>\n
//!   <FIRST_ADDRESS>\n<SIGNATURE>` (full BIP-129 Round-2; audit fields
//!   preserved via `ParsedImport::bsms_audit()` accessor; backed by
//!   `ImportProvenance::Bsms(Some(...))`).
//!
//! Network detection (SPEC §4.2 step 8, §7.0.a locked): inspect the BIP-48
//! coin-type child number on the FIRST cosigner's origin path; hardened `0'`
//! → mainnet, hardened `1'` → testnet. Cosigner-to-cosigner heterogeneity →
//! `ImportWalletParse` (exit 2). Signet/regtest are not distinguishable from
//! testnet at the origin-path layer (both use coin-type 1); both are imported
//! as testnet (FOLLOWUP `wallet-import-signet-regtest-disambiguation`).
//!
//! BIP-380 checksum: validated up-front via
//! `miniscript::descriptor::checksum::verify_checksum` against the user's
//! ORIGINAL concrete-keys descriptor body. The downstream `parse_descriptor`
//! pipeline transforms the body (substituting `@N` placeholders with
//! synthetic xpubs) before passing it to `MsDescriptor::from_str`, so the
//! original checksum no longer applies at that layer. SPEC §4.4's "auto-
//! validated" language is implemented here as a direct verify_checksum call
//! on the user-supplied body — preserving the SPEC intent (any checksum
//! mismatch is rejected) while accommodating the substitution-based
//! pipeline. See FOLLOWUP `wallet-import-bsms-checksum-delegation-note` for
//! SPEC § rephrase if needed.

use super::{
    pipeline::concrete_keys_to_placeholders, validate_watch_only_resolved, BsmsAuditFields,
    ImportProvenance, ParsedImport, WalletFormatParser,
};
use crate::error::ToolkitError;
use crate::parse_descriptor;
use crate::synthesize::{xpub_to_65, ResolvedSlot};
use bitcoin::bip32::{ChildNumber, DerivationPath, Fingerprint, Xpub};
use regex::Regex;
use std::io::Write;
use std::str::FromStr;
use std::sync::OnceLock;

pub(crate) struct BsmsParser;

const BSMS_HEADER: &str = "BSMS 1.0";

impl WalletFormatParser for BsmsParser {
    fn sniff(blob: &[u8]) -> bool {
        // SPEC §6.1.1 — exact prefix `BSMS 1.0\n` (or `BSMS 1.0\r\n` after
        // CRLF normalize). Re-check post-normalize so the heuristic is
        // robust to CRLF blobs encountered in the wild.
        let s = match std::str::from_utf8(blob) {
            Ok(s) => s,
            Err(_) => return false,
        };
        let normalized = s.replace("\r\n", "\n");
        normalized.starts_with("BSMS 1.0\n")
    }

    fn parse(blob: &[u8], stderr: &mut dyn Write) -> Result<Vec<ParsedImport>, ToolkitError> {
        let text = std::str::from_utf8(blob).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: bsms: parse error: blob is not valid UTF-8: {e}"
            ))
        })?;

        // SPEC §4.2 step 1: CRLF → LF normalize before any line-based
        // processing.
        let normalized = text.replace("\r\n", "\n");
        let lines: Vec<&str> = normalized.split('\n').collect();
        if lines.is_empty() {
            return Err(ToolkitError::ImportWalletParse(
                "import-wallet: bsms: parse error: empty blob".to_string(),
            ));
        }

        // SPEC §4.2 step 2: verify line 1 == `BSMS 1.0`. Other versions →
        // FutureFormat (exit 3) per §2.3.
        let header = lines[0];
        if header != BSMS_HEADER {
            if let Some(version) = header.strip_prefix("BSMS ") {
                return Err(ToolkitError::FutureFormat {
                    source: "bsms",
                    detail: format!("version {:?}; toolkit supports \"1.0\"", version),
                });
            }
            return Err(ToolkitError::ImportWalletParse(format!(
                "import-wallet: bsms: parse error: expected header `BSMS 1.0` on line 1, got {:?}",
                header
            )));
        }

        // SPEC §4.2 step 3: count non-empty lines (trailing newline may
        // produce a single empty final element). Detect 2-line vs 6-line.
        let trimmed_count = strip_trailing_empty(&lines).len();
        let (descriptor_body, audit) = match trimmed_count {
            2 => {
                writeln!(
                    stderr,
                    "warning: import-wallet: bsms: 2-line excerpt; full BIP-129 Round-2 carries token + signature + first-address verification fields; accepting reduced form"
                )
                .map_err(ToolkitError::Io)?;
                (lines[1], None)
            }
            6 => {
                let token = lines[1].to_string();
                let body = lines[2];
                let derivation_path = lines[3].to_string();
                let first_address = lines[4].to_string();
                let signature = lines[5].to_string();
                writeln!(
                    stderr,
                    "notice: import-wallet: bsms: 6-line Round-1 signature in the blob \
                     is not verified inline by this 2/6-line parser; supply the same record \
                     via --bsms-round1 <FILE> to engage v0.27.0 BIP-322 verification"
                )
                .map_err(ToolkitError::Io)?;
                (
                    body,
                    Some(BsmsAuditFields {
                        token,
                        signature,
                        first_address,
                        derivation_path,
                        verification: crate::wallet_import::BsmsVerification::NotAttempted,
                    }),
                )
            }
            other => {
                return Err(ToolkitError::ImportWalletParse(format!(
                    "import-wallet: bsms: parse error: expected 2 or 6 lines, got {other}"
                )));
            }
        };

        // SPEC §4.4 + §4.2 step 4: validate the BIP-380 checksum on the
        // ORIGINAL descriptor body (concrete `[fp/path]xpub` keys present).
        // The downstream `parse_descriptor` pipeline runs on the placeholder
        // form (post-substitution) where the checksum no longer applies, so
        // the checksum must be verified up-front against the user's
        // concrete-keys body. Returns the descriptor body sans `#<checksum>`
        // suffix; the placeholder adapter consumes this stripped form.
        let descriptor_body_no_csum =
            miniscript::descriptor::checksum::verify_checksum(descriptor_body).map_err(|e| {
                ToolkitError::ImportWalletParse(format!(
                    "import-wallet: bsms: parse error: BIP-380 checksum validation failed: {e}"
                ))
            })?;

        // SPEC §4.2 step 5: run concrete-keys → @N adapter against the
        // checksum-stripped body.
        let (placeholder_form, parsed_keys, parsed_fingerprints) =
            concrete_keys_to_placeholders(descriptor_body_no_csum)?;

        // SPEC §4.2 step 7: parse_descriptor::parse_descriptor consumes the
        // placeholder form + key/fingerprint vectors. BIP-380 checksum
        // auto-validated by MsDescriptor::from_str inside parse_descriptor.
        let descriptor = parse_descriptor::parse_descriptor(
            &placeholder_form,
            &parsed_keys,
            &parsed_fingerprints,
        )
        .map_err(|e| {
            // Wrap downstream DescriptorParse messages with the
            // SPEC §2.4 template prefix so user-facing stderr is
            // tied to the import-wallet subcommand.
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: bsms: parse error: {}",
                e.message()
            ))
        })?;

        // SPEC §4.2 step 8: network detection via BIP-48 coin-type
        // (hardened component index 1) on the FIRST cosigner. Cosigner-to-
        // cosigner heterogeneity → ImportWalletParse.
        let origins = extract_origin_components(descriptor_body_no_csum)?;
        let network = network_from_origins(&origins)?;

        // SPEC §4.2 step 9: build ResolvedSlot vec per cosigner; entropy=None.
        let mut cosigners: Vec<ResolvedSlot> = Vec::with_capacity(parsed_keys.len());
        for (i, _) in parsed_keys.iter().enumerate() {
            let (xpub, fp, path, path_raw) = build_slot_fields(descriptor_body_no_csum, i)?;
            // Sanity: the slot's xpub matches the ParsedKey payload (the
            // ParsedKey payload was derived from the same xpub bytes).
            debug_assert_eq!(xpub_to_65(&xpub), parsed_keys[i].payload);
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

        let threshold = extract_threshold(descriptor_body_no_csum)?;

        // SPEC §4.1 — first-address verification. v0.27.0 wires in the
        // toolkit-side derivation at canonical /0/0 via
        // `crate::derive_address::derive_first_address` and compares against
        // `audit.first_address` (6-line shape only; the 2-line shape carries
        // no audit fields). Mismatch is informational (stderr WARNING; not
        // hard-error) per BIP-129 §6's coordinator-output self-consistency
        // intent. Closes FOLLOWUP `bsms-first-address-verify`.
        if let Some(ref a) = audit {
            // Re-parse the descriptor string so we can derive against a
            // miniscript `Descriptor` value. The earlier `parse_descriptor`
            // returns the toolkit's `md_codec::Descriptor`, which carries
            // the canonical-form bytes; for first-address derivation we
            // need a `miniscript::Descriptor` over `DescriptorPublicKey`.
            // We use the user-supplied checksum-stripped body — taproot
            // descriptors are accepted at parse time (network detection +
            // checksum + xpub parse are all script-type-agnostic), but
            // address derivation via `derive_first_address` is non-taproot
            // by contract; we therefore skip the WARNING for taproot.
            use miniscript::{Descriptor as MsDescriptor, DescriptorPublicKey};
            if let Ok(parsed) =
                MsDescriptor::<DescriptorPublicKey>::from_str(descriptor_body_no_csum)
            {
                let is_taproot = matches!(parsed, MsDescriptor::Tr(_));
                if !is_taproot {
                    match crate::derive_address::derive_first_address(&parsed, network) {
                        Ok(computed) if computed == a.first_address => {
                            // Match: silent. The audit field is consistent
                            // with the toolkit-derived address.
                        }
                        Ok(computed) => {
                            // SPEC §2.4 row 3 template (restored at v0.27.0
                            // Phase 3; FOLLOWUP `bsms-first-address-verify`
                            // body line 2091). `at path <P>` segment sources
                            // from the audit's declared derivation_path.
                            writeln!(
                                stderr,
                                "warning: import-wallet: bsms: first-address mismatch at path {path}: computed {computed}, blob declares {declared}",
                                path = a.derivation_path,
                                computed = computed,
                                declared = a.first_address,
                            )
                            .map_err(ToolkitError::Io)?;
                        }
                        Err(e) => {
                            // v0.27.0 Phase 6.5 PR-review I3 fold:
                            // surface derivation failure as a stderr
                            // NOTICE rather than silently conflating it
                            // with a match. The user opted into BSMS
                            // first-address verify by including the
                            // 6-line audit fields; they deserve to know
                            // it was skipped. Downstream parse already
                            // validated the descriptor structurally, so
                            // this remains non-fatal.
                            writeln!(
                                stderr,
                                "notice: import-wallet: bsms: first-address verify could not run: {e}; \
                                 audit.first_address={declared} preserved verbatim in envelope",
                                declared = a.first_address,
                            )
                            .map_err(ToolkitError::Io)?;
                        }
                    }
                }
            }
        }

        Ok(vec![ParsedImport {
            descriptor,
            original_descriptor: descriptor_body.to_string(),
            cosigners,
            network,
            threshold,
            provenance: ImportProvenance::Bsms(audit),
        }])
    }
}

/// Per SPEC §4.2 step 9: drop trailing empty entries produced by a trailing
/// newline. Does NOT collapse intra-blob blank lines (those are a parse
/// error caught by the 2-or-6 line-count check).
fn strip_trailing_empty<'a>(lines: &'a [&'a str]) -> Vec<&'a str> {
    let mut v: Vec<&str> = lines.to_vec();
    while matches!(v.last(), Some(&"")) {
        v.pop();
    }
    v
}

/// Per-cosigner origin tuple lifted out of the descriptor body via the
/// shared `key_regex` pattern. Returned in declaration order.
fn extract_origin_components(
    descriptor_body: &str,
) -> Result<Vec<(Fingerprint, DerivationPath, String, String)>, ToolkitError> {
    let re = origin_capture_regex();
    let mut out = Vec::new();
    for cap in re.captures_iter(descriptor_body) {
        let fp_hex = cap.get(1).expect("group 1").as_str();
        let path_raw_inner = cap.get(2).expect("group 2").as_str();
        let xpub_str = cap.get(3).expect("group 3").as_str();

        let mut fp_bytes = [0u8; 4];
        for i in 0..4 {
            fp_bytes[i] = u8::from_str_radix(&fp_hex[i * 2..i * 2 + 2], 16).map_err(|e| {
                ToolkitError::ImportWalletParse(format!(
                    "import-wallet: bsms: parse error: fingerprint hex: {e}"
                ))
            })?;
        }
        let fp = Fingerprint::from(fp_bytes);
        let path = DerivationPath::from_str(&format!("m{path_raw_inner}")).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: bsms: parse error: derivation-path parse: {e}"
            ))
        })?;
        // path_raw is the bracket-inner form `<fp>/<path>` (mirrors
        // ResolvedSlot.path_raw conventions established by bundle.rs).
        let path_raw = format!("[{fp_hex}{path_raw_inner}]");
        out.push((fp, path, path_raw, xpub_str.to_string()));
    }
    if out.is_empty() {
        return Err(ToolkitError::ImportWalletParse(
            "import-wallet: bsms: parse error: no origin annotations in descriptor".to_string(),
        ));
    }
    Ok(out)
}

/// Build per-slot ResolvedSlot fields by re-running the origin lex (the
/// concrete-keys adapter consumes the xpub bytes only; this helper extracts
/// the typed Xpub + Fingerprint + DerivationPath for the ResolvedSlot vec).
fn build_slot_fields(
    descriptor_body: &str,
    slot_idx: usize,
) -> Result<(Xpub, Fingerprint, DerivationPath, String), ToolkitError> {
    let origins = extract_origin_components(descriptor_body)?;
    let (fp, path, path_raw, xpub_str) = origins.into_iter().nth(slot_idx).ok_or_else(|| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: bsms: parse error: slot index {slot_idx} out of range"
        ))
    })?;
    let (neutral, _variant) = crate::slip0132::normalize_xpub_prefix(&xpub_str)?;
    let xpub = Xpub::from_str(&neutral).map_err(|e| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: bsms: parse error: xpub decode for slot {slot_idx}: {e}"
        ))
    })?;
    Ok((xpub, fp, path, path_raw))
}

/// SPEC §4.2 step 8 network detection. Inspects the BIP-48 coin-type child
/// number (path component index 1, hardened) on the first parsed origin.
/// Returns `bitcoin::Network::Bitcoin` for hardened `0'`, `bitcoin::Network::Testnet`
/// for hardened `1'`. Other coin-types → parse error. Cosigner-to-cosigner
/// heterogeneity → parse error (per SPEC §4.2 step 8 locked rule).
fn network_from_origins(
    origins: &[(Fingerprint, DerivationPath, String, String)],
) -> Result<bitcoin::Network, ToolkitError> {
    if origins.is_empty() {
        return Err(ToolkitError::ImportWalletParse(
            "import-wallet: bsms: parse error: no origins to infer network from".to_string(),
        ));
    }
    let coin_types: Vec<u32> = origins
        .iter()
        .map(|(_, p, _, _)| coin_type_from_path(p))
        .collect::<Result<Vec<_>, _>>()?;
    let first = coin_types[0];
    for (i, ct) in coin_types.iter().enumerate().skip(1) {
        if *ct != first {
            return Err(ToolkitError::ImportWalletParse(format!(
                "import-wallet: bsms: cosigner {i} has coin-type {ct}, cosigner 0 has coin-type {first}; all cosigners must share a coin-type"
            )));
        }
    }
    match first {
        0 => Ok(bitcoin::Network::Bitcoin),
        1 => Ok(bitcoin::Network::Testnet),
        other => Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: bsms: parse error: unsupported coin-type {other} on origin path; only 0 (mainnet) and 1 (testnet) supported per BIP-48"
        ))),
    }
}

fn coin_type_from_path(path: &DerivationPath) -> Result<u32, ToolkitError> {
    let comps: Vec<&ChildNumber> = path.into_iter().collect();
    if comps.len() < 2 {
        return Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: bsms: parse error: origin path has only {} components; need ≥2 for BIP-48 coin-type inference",
            comps.len()
        )));
    }
    // BIP-48 path: m / purpose' / coin_type' / ...
    // We accept any purpose value (44'/45'/48'/49'/84'/86') because v0.26.0
    // import accepts heterogeneous singlesig + multisig BIP families; the
    // network inference is gated only on coin-type (index 1).
    match comps[1] {
        ChildNumber::Hardened { index } => Ok(*index),
        ChildNumber::Normal { index } => Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: bsms: parse error: coin-type component {index} is not hardened; BIP-48 requires `<coin_type>'`"
        ))),
    }
}

/// Extract the K from `thresh(K, ...)` / `multi(K, ...)` / `sortedmulti(K, ...)`
/// at the top-level miniscript context. Returns `None` for single-key
/// descriptors (`wpkh(...)`, `pkh(...)`, etc.).
///
/// The match is a lexical scan for the first occurrence of `thresh(`,
/// `multi(`, or `sortedmulti(`. The numeric K is the first comma-delimited
/// integer. For decaying-multisig shapes (`thresh(K, pkh(@0), s:pk(@1),
/// sln:older(N))`) this returns K (the thresh threshold), not N (the
/// timelock). Per SPEC §4.1: BSMS Round-2 carries the multisig threshold
/// via the `thresh()` outer-K in the wsh body.
/// v0.27.1 Phase 2 I6 fold: returns `Ok(None)` when no thresh/multi token is
/// found; `Err` on u8 overflow (was: silently mapped overflow to `None`).
/// Mirrors `bitcoin_core::extract_threshold`.
pub(super) fn extract_threshold(descriptor_body: &str) -> Result<Option<u8>, ToolkitError> {
    static R: OnceLock<Regex> = OnceLock::new();
    let re = R.get_or_init(|| {
        Regex::new(r"(?:thresh|multi|sortedmulti)\((\d+)\s*,").expect("threshold regex is fixed")
    });
    let cap = match re.captures(descriptor_body) {
        Some(c) => c,
        None => return Ok(None),
    };
    let arg = cap.get(1).expect("regex has capture group 1").as_str();
    arg.parse::<u8>().map(Some).map_err(|e| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: bsms: parse error: thresh/multi argument `{arg}` exceeds u8 range (>255 cosigners not supported): {e}"
        ))
    })
}

/// Shared origin-capture regex. Mirrors `pipeline::key_regex` but with
/// the same capture-group indices so caller code can index uniformly.
fn origin_capture_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"\[([0-9a-fA-F]{8})((?:/\d+'?)+)\]([xtyzuvYZUV]pub[A-HJ-NP-Za-km-z1-9]+)")
            .expect("origin_capture_regex is a fixed string literal")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// v0.27.1 Phase 2 R0 M1 fold: guarantee coverage of the
    /// `extract_threshold` u8-overflow branch via a direct unit test. (The
    /// integration cell at `tests/cli_import_wallet_bsms.rs::bsms_thresh_
    /// overflow_errors_clearly` accepts multiple correct rejection paths
    /// — checksum, descriptor parse, or u8-overflow — which means the
    /// overflow code path itself may not be exercised in the integration
    /// suite. This unit test is the regression guard.)
    #[test]
    fn extract_threshold_u8_overflow_is_typed_error() {
        // Body without thresh/multi → Ok(None).
        let r = extract_threshold("wpkh(@0)").unwrap();
        assert_eq!(r, None);

        // Body with thresh(2,…) → Ok(Some(2)).
        let r = extract_threshold("wsh(thresh(2,pk(@0),s:pk(@1)))").unwrap();
        assert_eq!(r, Some(2));

        // Body with sortedmulti(256,…) → Err (u8 overflow). 256 > u8::MAX.
        let err = extract_threshold("wsh(sortedmulti(256,@0,@1))").unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("exceeds u8 range") && msg.contains("256"),
            "expected u8-overflow diagnostic naming 256; got: {msg}"
        );
    }
}
