//! BIP-129 BSMS Round-2 parser.
//!
//! Per `design/SPEC_wallet_import_v0_26_0.md` §4 + `SPEC_wallet_import_v0_28_0.md`
//! §10. Accepts three shapes:
//! - 2-line: `BSMS 1.0\n<descriptor>#<checksum>` (kickoff seed-case form;
//!   stderr WARNING about reduced form).
//! - 4-line (v0.28.0; BIP-129-canonical Round-2 per BIP-129 line 96):
//!   `BSMS 1.0\n<descriptor>#<checksum>\n<path-restrictions>\n<FIRST_ADDRESS>`.
//!   The 4-line shape is symmetric with `wallet_export/bsms.rs`'s
//!   `BsmsForm::FourLine` emit (the canonical round-trip pair). Cross-validates
//!   line 4 against `derive_first_address` per SPEC §10.2. Audit provenance
//!   uses the empty-string-sentinel pattern (token / signature empty;
//!   first_address + derivation_path populated; verification = NotAttempted)
//!   per SPEC §10.3. Closes FOLLOWUP `bsms-bip129-full-cutover`.
//! - 6-line: `BSMS 1.0\n<TOKEN>\n<descriptor>#<checksum>\n<DERIVATION_PATH>\n
//!   <FIRST_ADDRESS>\n<SIGNATURE>` (legacy lenient 6-line shape; audit fields
//!   preserved via `ParsedImport::bsms_audit()` accessor; backed by
//!   `ImportProvenance::BsmsSixLine(...)`). DEPRECATED in v0.28.0; will be
//!   removed in a future minor version. See SPEC §10.4.
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

        // SPEC §4.2 step 3 + SPEC §10.1: count non-empty lines (trailing
        // newline may produce a single empty final element). Detect 2-line,
        // 4-line (BIP-129-canonical Round-2), or 6-line (DEPRECATED) shape.
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
            4 => {
                // SPEC §10.1 — BIP-129-canonical 4-line Round-2 shape per
                // BIP-129 line 96: header / descriptor#checksum /
                // path-restrictions / FIRST_ADDRESS. Audit provenance uses
                // the empty-string-sentinel pattern per SPEC §10.3:
                //   token = ""
                //   signature = ""
                //   first_address = lines[3]
                //   derivation_path = lines[2]   (the path-restrictions field)
                //   verification = NotAttempted
                // Cross-validation against `derive_first_address` runs in
                // the shared post-parse block below (SPEC §10.2); BIP-129
                // path-restrictions string is preserved verbatim in the
                // audit envelope's `derivation_path` slot for round-trip
                // (the field name is a legacy mismatch but the wire shape
                // is stable; see SPEC §10.3).
                let body = lines[1];
                let derivation_path = lines[2].to_string();
                let first_address = lines[3].to_string();
                (
                    body,
                    Some(BsmsAuditFields {
                        token: String::new(),
                        signature: String::new(),
                        first_address,
                        derivation_path,
                        verification: crate::wallet_import::BsmsVerification::NotAttempted,
                    }),
                )
            }
            6 => {
                let token = lines[1].to_string();
                let body = lines[2];
                let derivation_path = lines[3].to_string();
                let first_address = lines[4].to_string();
                let signature = lines[5].to_string();
                // SPEC §10.4 — DEPRECATION notice for the 6-line lenient
                // shape. v0.28.0 makes the 4-line BIP-129-canonical shape
                // the preferred ingest form; the 6-line lenient shape is
                // retained for one cycle to avoid breaking active flows but
                // will be removed in a future minor version.
                writeln!(
                    stderr,
                    "notice: import-wallet: bsms: 6-line lenient shape is DEPRECATED in v0.28+ and"
                )
                .map_err(ToolkitError::Io)?;
                writeln!(
                    stderr,
                    "will be removed in a future minor version; convert your blob to the BIP-129-"
                )
                .map_err(ToolkitError::Io)?;
                writeln!(
                    stderr,
                    "canonical 4-line shape (BSMS_VERSION + DESCRIPTOR + path-restrictions +"
                )
                .map_err(ToolkitError::Io)?;
                writeln!(
                    stderr,
                    "FIRST_ADDRESS) for forward compatibility. See SPEC §10 for the canonical shape."
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
                    "import-wallet: bsms: parse error: expected 2, 4, or 6 lines, got {other}"
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

        // v0.28.7 — Slug 1: refuse taproot at parse entry, mirroring emit-side
        // BsmsTaprootRefused. BIP-129 §1 prerequisites do not yet include BIP-386.
        // Detection mode: cheap textual sniff on `tr(` substring in the descriptor
        // block content. Authoritative parse-side detection happens later in this
        // fn via `MsDescriptor::Tr(_)`, but we want to refuse before doing the
        // (expensive) full descriptor-parse + first-address verify.
        if descriptor_body_no_csum.contains("tr(") {
            return Err(ToolkitError::BsmsTaprootImportRefused);
        }

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
        let origins = crate::wallet_import::pipeline::extract_origin_components(
            descriptor_body_no_csum,
            "bsms",
        )?;
        let network = network_from_origins(&origins)?;

        // SPEC §4.2 step 9: build ResolvedSlot vec per cosigner; entropy=None.
        let mut cosigners: Vec<ResolvedSlot> = Vec::with_capacity(parsed_keys.len());
        for (i, _) in parsed_keys.iter().enumerate() {
            let (xpub, fp, path) = build_slot_fields(descriptor_body_no_csum, i)?;
            // Sanity: the slot's xpub matches the ParsedKey payload (the
            // ParsedKey payload was derived from the same xpub bytes).
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

        // cycle-5 S-NET (axis 2 / H15 = L10): xpub-version vs coin-type cross-check.
        crate::wallet_import::pipeline::assert_slots_network_agrees(
            &cosigners,
            network,
            "import: bsms",
        )?;

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
            // SPEC bip388-double-star-shorthand-support §0 item 4 (soft gap):
            // this `from_str` re-parses the RAW concrete body directly (not
            // the `@N`-placeholder form the main parse used), so a literal
            // `/**` needs its own expansion here too — otherwise the `if let
            // Ok` silently SKIPS the first-address check on `/**` input
            // (the main parse above already accepts it via
            // `parse_descriptor`'s chokepoint).
            let expanded_body =
                crate::parse_descriptor::expand_literal_double_star(descriptor_body_no_csum);
            if let Ok(parsed) =
                MsDescriptor::<DescriptorPublicKey>::from_str(expanded_body.as_ref())
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
            provenance: match audit {
                Some(a) => ImportProvenance::BsmsSixLine(a),
                None => ImportProvenance::BsmsTwoLine,
            },
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

/// Build per-slot ResolvedSlot fields by re-running the origin lex (the
/// concrete-keys adapter consumes the xpub bytes only; this helper extracts
/// the typed Xpub + Fingerprint + DerivationPath for the ResolvedSlot vec).
fn build_slot_fields(
    descriptor_body: &str,
    slot_idx: usize,
) -> Result<(Xpub, Fingerprint, DerivationPath), ToolkitError> {
    let origins =
        crate::wallet_import::pipeline::extract_origin_components(descriptor_body, "bsms")?;
    let (fp, path, xpub_str) = origins.into_iter().nth(slot_idx).ok_or_else(|| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: bsms: parse error: slot index {slot_idx} out of range"
        ))
    })?;
    crate::wallet_import::pipeline::finalize_slot_fields(fp, path, &xpub_str, "bsms")
}

/// SPEC §4.2 step 8 network detection. Inspects the BIP-48 coin-type child
/// number (path component index 1, hardened) on the first parsed origin.
/// Returns `bitcoin::Network::Bitcoin` for hardened `0'`, `bitcoin::Network::Testnet`
/// for hardened `1'`. Other coin-types → parse error. Cosigner-to-cosigner
/// heterogeneity → parse error (per SPEC §4.2 step 8 locked rule).
fn network_from_origins(
    origins: &[(Fingerprint, DerivationPath, String)],
) -> Result<bitcoin::Network, ToolkitError> {
    if origins.is_empty() {
        return Err(ToolkitError::ImportWalletParse(
            "import-wallet: bsms: parse error: no origins to infer network from".to_string(),
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
    // v0.28.7 defense-in-depth: after Slug 1's BsmsTaprootImportRefused at
    // parse-entry, taproot blobs cannot reach this fn legitimately. But if
    // a future code path bypasses the parse-entry refusal, the existing
    // regex would return Ok(None) on `sortedmulti_a(...)` — silently emitting
    // `threshold=none` rather than refusing. Convert that silent miss into
    // an explicit refusal.
    if descriptor_body.contains("sortedmulti_a(") || descriptor_body.contains("multi_a(") {
        return Err(ToolkitError::BsmsTaprootImportRefused);
    }
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

    #[test]
    fn extract_threshold_refuses_taproot_multi_a_directly() {
        // v0.34.3: direct unit coverage for the v0.28.7 defense-in-depth guard
        // at `extract_threshold` (bsms.rs:496-497). The integration path can't
        // reach it — parse-entry refuses the `tr(` substring first (bsms.rs:215)
        // — so this asserts the guard directly on the multi_a / sortedmulti_a
        // bodies. Closes FOLLOWUP `bsms-extract-threshold-defense-in-depth-direct-unit-test`.
        assert!(matches!(
            extract_threshold("tr(NUMS,sortedmulti_a(2,@0,@1))"),
            Err(ToolkitError::BsmsTaprootImportRefused)
        ));
        assert!(matches!(
            extract_threshold("tr(NUMS,multi_a(2,@0,@1))"),
            Err(ToolkitError::BsmsTaprootImportRefused)
        ));
    }

    // ============================================================================
    // v0.28.0 Phase 7 (G1) — SPEC §10 4-line BIP-129-canonical parser units
    // ============================================================================

    /// Build a synthetic 4-line BIP-129-canonical Round-2 blob from
    /// (descriptor body without `#csum`, path-restrictions, first-address).
    /// Computes the BIP-380 checksum dynamically (mirrors the integration
    /// test helper `build_bsms_2line`).
    fn build_4line(desc: &str, path_restrictions: &str, first_address: &str) -> String {
        use miniscript::descriptor::checksum::Engine;
        let mut eng = Engine::new();
        eng.input(desc).expect("ascii-only descriptor body");
        let cs = eng.checksum();
        format!("BSMS 1.0\n{desc}#{cs}\n{path_restrictions}\n{first_address}\n")
    }

    /// SPEC §10.1 happy-path: 4-line parse succeeds; audit envelope uses
    /// the empty-string-sentinel pattern per SPEC §10.3 (token + signature
    /// empty; first_address + derivation_path populated; verification =
    /// NotAttempted). Replaces the deleted `bsms_4_line_blob_rejected_with_
    /// pointer_text` integration cell.
    #[test]
    fn parse_4line_happy_path_populates_audit_with_empty_sentinels() {
        // Real /0/0 first-address for the canonical mainnet sortedmulti-2of2
        // (re-using the fixture xpubs that vendor at tests/fixtures/
        // wallet_import/bsms-4line-no-path-restrictions.txt). Pre-derived
        // via `wallet_export::bsms::BsmsForm::FourLine` semantics so the
        // first-address cross-validation does NOT fire.
        let desc = "wsh(sortedmulti(2,[b8688df1/48'/0'/0'/2']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*,[28645006/48'/0'/0'/2']xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6/<0;1>/*))";
        let blob = build_4line(
            desc,
            "/0/*,/1/*",
            "bc1q2a4vm7ww7v0c02qerrgg0znr4ck4kez2la82kzpm9gturx68q4nsfdl000",
        );
        let mut stderr = Vec::new();
        let parsed = BsmsParser::parse(blob.as_bytes(), &mut stderr).expect("4-line parses");
        assert_eq!(parsed.len(), 1, "BSMS parse returns single ParsedImport");
        let p = &parsed[0];
        let audit = p.bsms_audit().expect("4-line populates audit (SPEC §10.3)");
        // SPEC §10.3 empty-string sentinels for token + signature.
        assert_eq!(
            audit.token, "",
            "4-line token must be empty-string sentinel"
        );
        assert_eq!(
            audit.signature, "",
            "4-line signature must be empty-string sentinel"
        );
        // SPEC §10.3 populated fields: first_address + derivation_path
        // (the path-restrictions string in 4-line semantics).
        assert_eq!(
            audit.first_address,
            "bc1q2a4vm7ww7v0c02qerrgg0znr4ck4kez2la82kzpm9gturx68q4nsfdl000"
        );
        assert_eq!(audit.derivation_path, "/0/*,/1/*");
        // verification = NotAttempted per SPEC §10.3.
        assert!(
            matches!(
                audit.verification,
                crate::wallet_import::BsmsVerification::NotAttempted
            ),
            "4-line verification field must be NotAttempted"
        );
        // SPEC §10.2 happy-path: no first-address mismatch WARNING on stderr.
        let stderr_str = String::from_utf8_lossy(&stderr);
        assert!(
            !stderr_str.contains("first-address mismatch"),
            "happy-path 4-line must NOT emit mismatch WARNING; stderr was: {stderr_str}"
        );
        // 4-line shape does NOT emit the 2-line WARNING or 6-line NOTICE.
        assert!(!stderr_str.contains("2-line excerpt"));
        assert!(!stderr_str.contains("DEPRECATED"));
    }

    /// SPEC §10.2 — 4-line first-address cross-validation. Wrong line-4
    /// address must trigger the existing `first-address mismatch` WARNING
    /// (informational; parse still succeeds — matches 6-line behavior).
    #[test]
    fn parse_4line_first_address_mismatch_emits_warning() {
        let desc = "wsh(sortedmulti(2,[b8688df1/48'/0'/0'/2']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*,[28645006/48'/0'/0'/2']xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6/<0;1>/*))";
        let blob = build_4line(
            desc,
            "/0/*,/1/*",
            // Deliberately-wrong first-address.
            "bc1qwrongwrongwrongwrongwrongwrongwrongwrongwrongwronguakw50",
        );
        let mut stderr = Vec::new();
        let parsed = BsmsParser::parse(blob.as_bytes(), &mut stderr).expect(
            "4-line parse succeeds even with first-address mismatch (SPEC §10.2 informational)",
        );
        assert_eq!(parsed.len(), 1);
        let stderr_str = String::from_utf8_lossy(&stderr);
        assert!(
            stderr_str.contains("first-address mismatch"),
            "expected first-address mismatch WARNING; stderr was: {stderr_str}"
        );
    }

    /// SPEC §10.1 — line-3 (path-restrictions) is preserved verbatim into
    /// the audit envelope's `derivation_path` slot. Non-canonical line-3
    /// strings such as `"No path restrictions"` survive into the audit
    /// envelope without rewriting.
    #[test]
    fn parse_4line_line3_preserved_verbatim_in_audit() {
        let desc = "wsh(sortedmulti(2,[b8688df1/48'/0'/0'/2']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*,[28645006/48'/0'/0'/2']xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6/<0;1>/*))";
        let blob = build_4line(
            desc,
            "No path restrictions",
            "bc1q2a4vm7ww7v0c02qerrgg0znr4ck4kez2la82kzpm9gturx68q4nsfdl000",
        );
        let mut stderr = Vec::new();
        let parsed = BsmsParser::parse(blob.as_bytes(), &mut stderr).expect("4-line parses");
        let audit = parsed[0].bsms_audit().expect("audit populated");
        assert_eq!(
            audit.derivation_path, "No path restrictions",
            "line-3 must be preserved verbatim in audit envelope"
        );
    }

    /// SPEC §10.4 — 6-line shape continues to parse; emits the new
    /// DEPRECATION NOTICE shape (4 lines of stderr text per the P7B
    /// `writeln!` block). Regression guard against accidental removal
    /// of any of the 4 stderr lines.
    #[test]
    fn parse_6line_emits_deprecation_notice_shape() {
        let desc = "wsh(sortedmulti(2,[b8688df1/48'/0'/0'/2']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*,[28645006/48'/0'/0'/2']xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6/<0;1>/*))";
        use miniscript::descriptor::checksum::Engine;
        let mut eng = Engine::new();
        eng.input(desc).expect("ascii-only");
        let cs = eng.checksum();
        let blob = format!(
            "BSMS 1.0\n00112233445566778899aabbccddeeff\n{desc}#{cs}\nm/48'/0'/0'/2'\nbc1q2a4vm7ww7v0c02qerrgg0znr4ck4kez2la82kzpm9gturx68q4nsfdl000\nH/example/sig/base64=\n"
        );
        let mut stderr = Vec::new();
        let parsed = BsmsParser::parse(blob.as_bytes(), &mut stderr).expect("6-line parses");
        assert_eq!(parsed.len(), 1);
        let stderr_str = String::from_utf8_lossy(&stderr);
        // Each of the 4 substrings must appear in the rendered stderr.
        assert!(
            stderr_str.contains("6-line lenient shape is DEPRECATED in v0.28+ and"),
            "missing DEPRECATION line 1; stderr was: {stderr_str}"
        );
        assert!(
            stderr_str.contains("will be removed in a future minor version"),
            "missing DEPRECATION line 2; stderr was: {stderr_str}"
        );
        assert!(
            stderr_str.contains("canonical 4-line shape"),
            "missing DEPRECATION line 3; stderr was: {stderr_str}"
        );
        assert!(
            stderr_str.contains("SPEC §10 for the canonical shape"),
            "missing DEPRECATION line 4; stderr was: {stderr_str}"
        );
    }
}
