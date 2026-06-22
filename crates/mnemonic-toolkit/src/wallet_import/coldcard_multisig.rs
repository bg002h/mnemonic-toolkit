//! Coldcard multisig text-file parser (`--format coldcard-multisig`).
//!
//! Per `design/SPEC_wallet_import_v0_28_0.md` §11.4. This format is a
//! line-oriented TEXT shape (NOT JSON) produced by Coldcard firmware when
//! the user exports a multisig wallet (`Settings → Multisig Wallets →
//! Export`). The shape is also accepted byte-identically by Blockstream
//! Jade via its `register_multisig` RPC `multisig_file` reply field —
//! `wallet_import/jade.rs` (Phase P5B) delegates here for the inner text.
//!
//! ## Sniff signature
//!
//! Top-of-blob lines (UTF-8; CRLF normalized to LF) contain ALL of:
//! - `Name:` line-prefix
//! - `Policy:` line-prefix
//! - `Format:` line-prefix
//!
//! Any leading `# …` comment lines and blank lines are tolerated. The
//! `XFP:` header line is OPTIONAL (firmware-variance). Sniff scans the
//! first ~20 lines of the blob (well above the maximum header-block size)
//! for these markers.
//!
//! ## On-disk shape
//!
//! Two firmware-variance shapes are accepted at parse time:
//!
//! 1. **Shared-derivation shape** (matches `wallet_export/coldcard.rs:254
//!    emit_coldcard_multisig_text` output — the toolkit's own emit form):
//!    ```text
//!    Name: <wallet-name>
//!    Policy: <K> of <N>
//!    Derivation: m/...
//!    Format: P2WSH | P2SH-P2WSH | P2SH
//!    <XFP>: <xpub>
//!    <XFP>: <xpub>
//!    ...
//!    ```
//!
//! 2. **Per-cosigner shape** (older Coldcard firmware + several third-party
//!    coordinators emit this form):
//!    ```text
//!    Name: <wallet-name>
//!    Policy: <K>-of-<N>
//!    Format: P2WSH
//!    Derivation: m/...
//!    <xpub>
//!    Derivation: m/...
//!    <xpub>
//!    ...
//!    ```
//!
//! Also accepted: an optional leading `XFP: <hex>` line carrying the master
//! fingerprint (Coldcard variant). When present, it OVERRIDES the
//! computed-from-xpub fingerprint per SPEC §11.4.1 (cycle-13a DEPTH-GATED
//! truth table). The computed `Xpub::fingerprint()` is the master fingerprint
//! ONLY when `xpub.depth == 0`; at depth>0 a NO-XFP blob is REFUSED (the
//! master fp is unrecoverable from an account xpub) and a supplied XFP is
//! accepted silently (no disagreement warning).
//!
//! ## Provenance
//!
//! `ImportProvenance::ColdcardMultisig(ColdcardMultisigSourceMetadata)`.
//! `xfp_was_blob_supplied` / `xfp_header_disagreed` flags are populated per
//! the SPEC §11.4.1 truth table; the WARNING stderr message is emitted
//! during `parse` (not `sniff`).

use super::{
    pipeline::concrete_keys_to_placeholders, validate_watch_only_resolved, ImportProvenance,
    ParsedImport, WalletFormatParser,
};
use crate::error::ToolkitError;
use crate::parse_descriptor;
use crate::synthesize::{xpub_to_65, ResolvedSlot};
use bitcoin::bip32::{ChildNumber, DerivationPath, Fingerprint, Xpub};
use miniscript::descriptor::checksum::Engine as ChecksumEngine;
use std::io::Write;
use std::str::FromStr;

pub(crate) struct ColdcardMultisigParser;

/// SPEC §11.4 — line-oriented Coldcard multisig text format provenance.
///
/// Carries the parsed header fields + the xfp-policy telemetry flags
/// per SPEC §11.4.1 (5-row truth table). `xfp_was_blob_supplied` is `true`
/// when the blob carried an `XFP:` header line OR any cosigner line used
/// the `<XFP>: <xpub>` shared-derivation form (both are "blob-supplied"
/// fingerprint sources per SPEC §11.4.1 semantic); `xfp_header_disagreed`
/// is `true` only when both the header AND a computed fingerprint were
/// available AND they did NOT byte-match (the WARNING-fire row of the
/// truth table).
///
/// `#[allow(dead_code)]` covers the P4B → P4C interim: P4B publishes the
/// type and its fields, but per-field consumption by downstream emitters
/// (envelope JSON, manual-chapter helpers, etc.) lands at P4C + later
/// phases. The `name` and `policy` fields are consumed by `Display` /
/// canonicalize helpers within P4B; `script_format` drives synthesis;
/// `xfp_was_blob_supplied` / `xfp_header_disagreed` / `dropped_fields`
/// surface in P4C integration cells.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct ColdcardMultisigSourceMetadata {
    pub(crate) name: String,
    pub(crate) policy: PolicyKOfN,
    pub(crate) script_format: ColdcardMsFormat,
    /// SPEC §11.4.1 telemetry: blob carried an `XFP:` header line OR any
    /// cosigner line used the `<XFP>: <xpub>` shared-derivation form.
    pub(crate) xfp_was_blob_supplied: bool,
    /// SPEC §11.4.1 telemetry: header present AND computed available AND
    /// the two disagreed (WARNING surfaced via stderr). `false` for the
    /// silent-match row and for rows where `xfp_was_blob_supplied=false`.
    pub(crate) xfp_header_disagreed: bool,
    /// Future-proof: parser-encountered field names that were NOT consumed
    /// into typed metadata fields. Currently empty (header schema is
    /// closed); reserved for forward-compat with firmware extensions.
    pub(crate) dropped_fields: Vec<String>,
}

/// SPEC §11.4 — K-of-N policy as parsed from the `Policy:` header line.
/// Both `K of N` (space form, the toolkit's own emit) and `K-of-N` (dash
/// form, third-party variant) accepted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PolicyKOfN {
    pub(crate) k: u8,
    pub(crate) n: u8,
}

/// SPEC §11.4 — `Format:` header script-type discriminator. Maps to the
/// descriptor synthesis wrapper (`wsh(...)` vs `sh(wsh(...))` vs `sh(...)`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColdcardMsFormat {
    P2wsh,
    P2shP2wsh,
    P2sh,
}

impl WalletFormatParser for ColdcardMultisigParser {
    fn sniff(blob: &[u8]) -> bool {
        // SPEC §11.4 sniff: must be valid UTF-8 (Coldcard text export is
        // ASCII-only in practice; UTF-8 superset accepted for tolerance);
        // must contain `Name:` + `Policy:` + `Format:` line-prefixes
        // within the first ~20 lines of the blob (the header block is
        // ~5 lines + optional XFP/Derivation; 20 is far above any
        // plausible header-block size).
        let text = match std::str::from_utf8(blob) {
            Ok(s) => s,
            Err(_) => return false,
        };
        let normalized = text.replace("\r\n", "\n");
        let header_lines: Vec<&str> = normalized.lines().take(20).collect();
        let has_name = header_lines.iter().any(|l| line_key(l) == Some("Name"));
        let has_policy = header_lines.iter().any(|l| line_key(l) == Some("Policy"));
        let has_format = header_lines.iter().any(|l| line_key(l) == Some("Format"));
        has_name && has_policy && has_format
    }

    fn parse(blob: &[u8], stderr: &mut dyn Write) -> Result<Vec<ParsedImport>, ToolkitError> {
        let parsed_text = parse_text(blob, stderr)?;
        Ok(vec![parsed_text])
    }
}

/// SPEC §11.4 — line-oriented Coldcard multisig text parser. Returns ONE
/// `ParsedImport` (the format is single-descriptor by construction — N
/// cosigners form one multisig wallet).
///
/// Exposed as `pub(super)` so `wallet_import/jade.rs` (Phase P5B) can
/// delegate to this helper for the inner Coldcard text wrapped inside
/// Jade's `multisig_file` field.
///
/// The returned `ImportProvenance::ColdcardMultisig(metadata)` carries
/// the SPEC §11.4.1 truth-table telemetry flags
/// (`xfp_was_blob_supplied`, `xfp_header_disagreed`).
pub(super) fn parse_text(
    blob: &[u8],
    stderr: &mut dyn Write,
) -> Result<ParsedImport, ToolkitError> {
    let text = std::str::from_utf8(blob).map_err(|e| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: coldcard-multisig: parse error: blob is not valid UTF-8: {e}"
        ))
    })?;

    let normalized = text.replace("\r\n", "\n");

    // Single-pass header + body walk. Classify each line into one of:
    //   - blank line (skip)
    //   - comment line `# ...` (skip)
    //   - `XFP: <hex>` (optional master fingerprint header)
    //   - `Name: <value>` (wallet name)
    //   - `Policy: <K> of <N>` or `<K>-of-<N>` (multisig policy)
    //   - `Derivation: m/...` (shared OR per-cosigner derivation path)
    //   - `Format: <P2WSH | P2SH-P2WSH | P2SH>` (script-type)
    //   - `<XFP_hex>: <xpub>` (cosigner entry — shared-derivation shape)
    //   - bare `<xpub>` (cosigner entry — per-cosigner shape)
    //   - anything else (unrecognized → preserve in dropped_fields for diagnostics)
    let mut name: Option<String> = None;
    let mut policy: Option<PolicyKOfN> = None;
    let mut script_format: Option<ColdcardMsFormat> = None;
    let mut header_xfp: Option<Fingerprint> = None;
    // For shared-derivation shape: the path is set ONCE before the cosigner
    // block. For per-cosigner shape: the path immediately precedes each xpub.
    let mut shared_derivation: Option<String> = None;
    let mut cosigners_raw: Vec<RawCosigner> = Vec::new();
    let mut dropped_fields: Vec<String> = Vec::new();

    // Per-cosigner-shape staging: when we see a `Derivation:` AFTER a
    // cosigner has been recorded (or AFTER `Format:` when no cosigner yet),
    // we treat it as a per-cosigner derivation that pairs with the NEXT
    // bare-xpub line.
    let mut pending_per_cosigner_path: Option<String> = None;

    for (line_idx, raw_line) in normalized.lines().enumerate() {
        let line_no = line_idx + 1; // 1-based for user-facing errors.
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some(key) = line_key(raw_line) {
            // `Key: value` shape. Extract value after the FIRST `:`.
            let value = match raw_line.find(':') {
                Some(i) => raw_line[i + 1..].trim().to_string(),
                None => continue, // unreachable — line_key returned Some
            };
            match key {
                "Name" => {
                    name = Some(value);
                }
                "Policy" => {
                    policy = Some(parse_policy(&value, line_no)?);
                }
                "Format" => {
                    script_format = Some(parse_script_format(&value, line_no)?);
                }
                "XFP" => {
                    header_xfp = Some(parse_fingerprint_hex(&value, line_no, "XFP header")?);
                }
                "Derivation" => {
                    // First `Derivation:` BEFORE any cosigner → shared.
                    // Subsequent `Derivation:` → per-cosigner staging.
                    if shared_derivation.is_none() && cosigners_raw.is_empty() {
                        shared_derivation = Some(value.clone());
                    }
                    // Always stage as per-cosigner pending too — the next
                    // bare-xpub line will consume it. This handles the case
                    // where the shared-derivation file format is used AND
                    // also pairs subsequent `Derivation:`s with cosigners.
                    pending_per_cosigner_path = Some(value);
                }
                cosigner_xfp_key if is_xfp_hex(cosigner_xfp_key) => {
                    // `<XFP_hex>: <xpub>` cosigner line. cycle-13a Q1: ALSO
                    // consume any pending per-line `Derivation:` path so the
                    // interleaved shape-2 form (`Derivation: <path>` then
                    // `<XFP_master>: <xpub>`, which H11's divergent export
                    // emits) round-trips the per-cosigner path. `.take()`
                    // empties `pending_per_cosigner_path` (no separate clear).
                    // This consumes ONLY the per-line pending — it MUST NOT
                    // disturb `shared_derivation`, so cosigners with no
                    // per-line `Derivation:` still fall back to the shared
                    // path at the effective-path resolution step.
                    let fp =
                        parse_fingerprint_hex(cosigner_xfp_key, line_no, "cosigner XFP prefix")?;
                    cosigners_raw.push(RawCosigner {
                        xpub_str: value,
                        per_line_xfp: Some(fp),
                        per_line_path: pending_per_cosigner_path.take(),
                    });
                }
                unknown => {
                    dropped_fields.push(unknown.to_string());
                }
            }
        } else {
            // Bare-value line. Treat as a per-cosigner bare xpub IF we have
            // a pending per-cosigner derivation; else if `Derivation:` was
            // NEVER seen, refuse with a clear error. Otherwise (no pending
            // path but past derivation) treat as malformed cosigner.
            let xpub_str = trimmed.to_string();
            if pending_per_cosigner_path.is_some() {
                let path = pending_per_cosigner_path.take();
                cosigners_raw.push(RawCosigner {
                    xpub_str,
                    per_line_xfp: None,
                    per_line_path: path,
                });
            } else if shared_derivation.is_some() && header_xfp.is_some() {
                // Some firmware variants emit `<xpub>\n` lines without a
                // preceding `Derivation:` AND without a `<XFP>:` prefix when
                // the top-level `XFP:` header carries the master and the
                // shared `Derivation:` applies to all cosigners. Treat as
                // bare-xpub cosigner with no per-line XFP/path.
                cosigners_raw.push(RawCosigner {
                    xpub_str,
                    per_line_xfp: None,
                    per_line_path: None,
                });
            } else {
                return Err(ToolkitError::ImportWalletParse(format!(
                    "import-wallet: coldcard-multisig: parse error: line {line_no}: \
                     bare xpub `{xpub_str}` has no associated derivation path \
                     (expected either a preceding `Derivation: m/...` line or a \
                     `<XFP>: <xpub>` form per SPEC §11.4 shape variants)"
                )));
            }
        }
    }

    // Header completeness checks.
    let name = name.ok_or_else(|| {
        ToolkitError::ImportWalletParse(
            "import-wallet: coldcard-multisig: parse error: missing `Name:` header".to_string(),
        )
    })?;
    let policy = policy.ok_or_else(|| {
        ToolkitError::ImportWalletParse(
            "import-wallet: coldcard-multisig: parse error: missing `Policy:` header".to_string(),
        )
    })?;
    let script_format = script_format.ok_or_else(|| {
        ToolkitError::ImportWalletParse(
            "import-wallet: coldcard-multisig: parse error: missing `Format:` header".to_string(),
        )
    })?;

    if cosigners_raw.len() as u8 != policy.n {
        return Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: coldcard-multisig: parse error: Policy declared {n} cosigners \
             but {found} cosigner entries were parsed",
            n = policy.n,
            found = cosigners_raw.len(),
        )));
    }
    if policy.k == 0 || policy.k > policy.n {
        return Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: coldcard-multisig: parse error: Policy threshold {k} out of \
             range for {n} cosigners (must be 1..=n)",
            k = policy.k,
            n = policy.n,
        )));
    }

    // Resolve each cosigner to (Xpub, Fingerprint, DerivationPath, path_raw).
    // SPEC §11.4.1 xfp-policy 5-row truth table is applied per-cosigner here.
    let mut xfp_was_blob_supplied = header_xfp.is_some();
    let mut xfp_header_disagreed = false;
    let mut resolved: Vec<ResolvedCosigner> = Vec::with_capacity(cosigners_raw.len());
    for (i, raw) in cosigners_raw.iter().enumerate() {
        // Resolve effective derivation path. Per-cosigner overrides shared.
        let path_str = raw
            .per_line_path
            .as_deref()
            .or(shared_derivation.as_deref())
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(format!(
                    "import-wallet: coldcard-multisig: parse error: cosigner {i}: \
                     no derivation path (missing both shared `Derivation:` and \
                     per-cosigner `Derivation:`)"
                ))
            })?;
        let path = DerivationPath::from_str(path_str).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: coldcard-multisig: parse error: cosigner {i}: \
                 derivation path `{path_str}` parse failed: {e}"
            ))
        })?;
        let path_components_str = derivation_path_components(path_str);

        // Resolve effective XFP per SPEC §11.4.1 — DEPTH-GATED (cycle-13a H14).
        //
        // `Xpub::fingerprint()` is the HASH160 of the CURRENT key (rust-bitcoin
        // `bip32.rs`), so it equals the BIP-380 *master* fingerprint ONLY when
        // the xpub IS the master, i.e. `xpub.depth == 0`. At depth>0 it is the
        // account-key's own id, NOT the master — and the master fp is
        // unrecoverable from a child xpub (child→parent is one-way). So the
        // depth byte of the decoded xpub is the authoritative discriminator
        // (independent of the declared `Derivation:` path).
        let xpub_parse_result = Xpub::from_str(&raw.xpub_str);
        let computed_fp: Option<Fingerprint> =
            xpub_parse_result.as_ref().ok().map(|x| x.fingerprint());
        let computed_depth: Option<u8> = xpub_parse_result.as_ref().ok().map(|x| x.depth);
        let xpub_is_master = computed_depth == Some(0);
        let supplied_fp: Option<Fingerprint> = raw.per_line_xfp.or(header_xfp); // per-line shared form OR top-level XFP header.

        let effective_fp: Fingerprint = match (supplied_fp, computed_fp) {
            // H14-c: supplied XFP at depth>0 → the supplied value is the only
            // signal for the master fp; comparing it to the account-key id
            // (which can NEVER equal the master at depth>0) would emit a
            // guaranteed-spurious warning. Accept supplied silently; do NOT
            // compute/compare; do NOT set xfp_header_disagreed.
            (Some(supplied), _) if !xpub_is_master => supplied,
            // Row 1 (H14-d at depth 0): supplied + computed available + match.
            (Some(supplied), Some(computed)) if supplied == computed => supplied,
            // Row 2 (H14-d at depth 0): supplied + computed available +
            // MISMATCH → WARNING + use supplied (per SPEC §11.4.1 byte-exact
            // template). Reachable only at depth 0 (depth>0 took H14-c above),
            // where the computed fp IS comparable to the master XFP.
            (Some(supplied), Some(computed)) => {
                xfp_header_disagreed = true;
                writeln!(
                    stderr,
                    "warning: import-wallet: coldcard-multisig: xfp header `XFP: {supplied}` \
                     disagrees with computed fingerprint `{computed}` from cosigner xpub; \
                     using blob-supplied header value as authoritative",
                    supplied = supplied.to_string().to_uppercase(),
                    computed = computed.to_string().to_uppercase(),
                )
                .map_err(ToolkitError::Io)?;
                supplied
            }
            // Row 3 (H14-e): supplied + xpub malformed (computed/depth
            // unavailable) → use supplied silently; the xpub-parse error
            // surfaces below when we try to build the descriptor.
            (Some(supplied), None) => supplied,
            // H14-a: no supplied XFP + computed available + xpub IS the master
            // (depth 0) → `xpub.fingerprint()` IS the master fp → use it
            // silently.
            (None, Some(computed)) if xpub_is_master => computed,
            // H14-b: no supplied XFP + computed available BUT xpub is depth>0
            // → the master fp is UNRECOVERABLE from an account xpub → REFUSE
            // (was the Row-4 silent-substitute bug). Exit 2 (ImportWalletParse).
            (None, Some(_)) => {
                let depth = computed_depth.unwrap_or_default();
                return Err(ToolkitError::ImportWalletParse(format!(
                    "import-wallet: coldcard-multisig: parse error: cosigner {i}: cannot \
                     determine master fingerprint — the cosigner xpub is at depth {depth} \
                     (an account xpub), so its own fingerprint is NOT the master fingerprint \
                     a BIP-380 key-origin requires, and the master fingerprint is unrecoverable \
                     from an account xpub. Re-export with the device's XFP (a top-level `XFP:` \
                     header or a per-cosigner `<XFP>: <xpub>` line carrying the master \
                     fingerprint)."
                )));
            }
            // Row 5: no supplied XFP + no computed (xpub malformed) → hard error.
            (None, None) => {
                let xpub_err = xpub_parse_result
                    .as_ref()
                    .err()
                    .map(|e| e.to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                return Err(ToolkitError::ImportWalletParse(format!(
                    "coldcard-multisig: cannot compute xfp: no XFP header and xpub parse \
                     failed: {xpub_err}"
                )));
            }
        };
        // Mark the telemetry flag when the per-line `<XFP>:` form was used
        // (this is also a "blob-supplied" XFP per SPEC §11.4.1 semantic,
        // even if the top-level `XFP:` header was absent).
        if raw.per_line_xfp.is_some() {
            xfp_was_blob_supplied = true;
        }

        let xpub = xpub_parse_result.map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: coldcard-multisig: parse error: cosigner {i}: \
                 xpub `{}` parse failed: {e}",
                raw.xpub_str,
            ))
        })?;

        let path_raw = format!("[{}{}]", effective_fp, path_components_str);
        resolved.push(ResolvedCosigner {
            xpub,
            fingerprint: effective_fp,
            path,
            path_raw,
        });
    }

    // Synthesize the descriptor body using the `[fp/path]xpub` bracket form
    // + `/<0;1>/*` multipath suffix (SPEC §11.4 — the multipath suffix is
    // implied by the multisig context). Wrap per script_format.
    let descriptor_body_raw = build_descriptor_body(policy.k, &resolved, script_format)?;

    // Re-checksum via miniscript engine for canonical BIP-380 form.
    let csum = compute_bip380_checksum(&descriptor_body_raw)?;
    let descriptor_with_csum = format!("{descriptor_body_raw}#{csum}");

    // Network detection: BIP-48 coin-type from the first cosigner's
    // derivation path (component index 1, hardened).
    let network = network_from_path(&resolved[0].path)?;

    // Heterogeneity check: all cosigners must share the same coin-type.
    let first_ct = coin_type_index(&resolved[0].path)?;
    for (i, cs) in resolved.iter().enumerate().skip(1) {
        let ct = coin_type_index(&cs.path)?;
        if ct != first_ct {
            return Err(ToolkitError::ImportWalletParse(format!(
                "import-wallet: coldcard-multisig: parse error: cosigner {i} has coin-type \
                 {ct}, cosigner 0 has coin-type {first_ct}; all cosigners must share a coin-type"
            )));
        }
    }

    // Feed the descriptor body through the standard concrete-keys →
    // @N-placeholder pipeline so the toolkit's descriptor representation
    // is the same shape as BSMS / Bitcoin Core parses produce.
    let (placeholder_form, parsed_keys, parsed_fingerprints) =
        concrete_keys_to_placeholders(&descriptor_body_raw).map_err(|e| {
            // Re-tag the BSMS pipeline's error prefix as coldcard-multisig.
            ToolkitError::ImportWalletParse(e.message().replacen(
                "import-wallet: bsms:",
                "import-wallet: coldcard-multisig:",
                1,
            ))
        })?;

    let descriptor =
        parse_descriptor::parse_descriptor(&placeholder_form, &parsed_keys, &parsed_fingerprints)
            .map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: coldcard-multisig: parse error: {}",
                e.message()
            ))
        })?;

    let cosigners_slots: Vec<ResolvedSlot> = resolved
        .iter()
        .enumerate()
        .map(|(i, c)| {
            // Sanity: the slot's xpub matches the ParsedKey payload (we
            // built parsed_keys from the same descriptor body bytes).
            debug_assert_eq!(xpub_to_65(&c.xpub), parsed_keys[i].payload);
            ResolvedSlot {
                xpub: c.xpub,
                fingerprint: c.fingerprint,
                path: c.path.clone(),
                entropy: None,
                master_xpub: None,
                language: None,
                _entropy_pin: None,
            }
        })
        .collect();

    validate_watch_only_resolved(&cosigners_slots)?;

    // cycle-5 S-NET (axis 2 / H15): xpub-version vs coin-type cross-check.
    crate::wallet_import::pipeline::assert_slots_network_agrees(
        &cosigners_slots,
        network,
        "import: coldcard-multisig",
    )?;

    let metadata = ColdcardMultisigSourceMetadata {
        name,
        policy,
        script_format,
        xfp_was_blob_supplied,
        xfp_header_disagreed,
        dropped_fields,
    };

    Ok(ParsedImport {
        descriptor,
        original_descriptor: descriptor_with_csum,
        cosigners: cosigners_slots,
        network,
        threshold: Some(policy.k),
        provenance: ImportProvenance::ColdcardMultisig(metadata),
    })
}

/// Raw cosigner entry as lifted from the text blob — pre-resolution.
/// `per_line_xfp` is `Some` for shared-derivation shape's `<XFP>: <xpub>`
/// line; `None` for per-cosigner shape's bare-xpub line. Same for
/// `per_line_path`.
struct RawCosigner {
    xpub_str: String,
    per_line_xfp: Option<Fingerprint>,
    per_line_path: Option<String>,
}

/// Post-resolution cosigner: typed xpub + effective fingerprint + path +
/// canonical path_raw bracket-form for the descriptor body.
struct ResolvedCosigner {
    xpub: Xpub,
    fingerprint: Fingerprint,
    path: DerivationPath,
    path_raw: String,
}

/// Parse the `Policy:` value (e.g., `2 of 3` OR `2-of-3`) into a typed
/// PolicyKOfN.
fn parse_policy(value: &str, line_no: usize) -> Result<PolicyKOfN, ToolkitError> {
    // Accept both `K of N` and `K-of-N` forms (and the lowercase `k of n` for
    // robustness, though the canonical Coldcard emit is `K of N`).
    let cleaned = value
        .replace("of", " of ")
        .replace('-', " ")
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ");
    // Now cleaned looks like `K of N` or `K N` (after dash-removal).
    // Parse out two consecutive integers separated by `of` or whitespace.
    let parts: Vec<&str> = cleaned.split_whitespace().collect();
    let nums: Vec<&&str> = parts
        .iter()
        .filter(|p| !p.eq_ignore_ascii_case("of"))
        .collect();
    if nums.len() != 2 {
        return Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: coldcard-multisig: parse error: line {line_no}: \
             Policy `{value}` does not match `K of N` or `K-of-N` shape \
             (got {n} numeric token(s))",
            n = nums.len(),
        )));
    }
    let k: u8 = nums[0].parse().map_err(|e| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: coldcard-multisig: parse error: line {line_no}: \
             Policy K `{}` is not a u8 integer: {e}",
            nums[0]
        ))
    })?;
    let n: u8 = nums[1].parse().map_err(|e| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: coldcard-multisig: parse error: line {line_no}: \
             Policy N `{}` is not a u8 integer: {e}",
            nums[1]
        ))
    })?;
    Ok(PolicyKOfN { k, n })
}

/// Parse the `Format:` value into a typed ColdcardMsFormat.
fn parse_script_format(value: &str, line_no: usize) -> Result<ColdcardMsFormat, ToolkitError> {
    // Case-insensitive match per Coldcard convention.
    let upper = value.trim().to_ascii_uppercase();
    match upper.as_str() {
        "P2WSH" => Ok(ColdcardMsFormat::P2wsh),
        "P2SH-P2WSH" | "P2SH_P2WSH" | "P2WSH-P2SH" => Ok(ColdcardMsFormat::P2shP2wsh),
        "P2SH" => Ok(ColdcardMsFormat::P2sh),
        other => Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: coldcard-multisig: parse error: line {line_no}: \
             Format `{other}` is not one of `P2WSH`, `P2SH-P2WSH`, `P2SH`"
        ))),
    }
}

/// Parse an 8-hex-char fingerprint (case-insensitive). Used for both the
/// top-level `XFP:` header AND each cosigner's `<XFP>:` prefix.
fn parse_fingerprint_hex(
    s: &str,
    line_no: usize,
    context: &str,
) -> Result<Fingerprint, ToolkitError> {
    let trimmed = s.trim();
    if trimmed.len() != 8 || !trimmed.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: coldcard-multisig: parse error: line {line_no}: \
             {context} `{trimmed}` is not 8 hex characters"
        )));
    }
    let mut bytes = [0u8; 4];
    for i in 0..4 {
        bytes[i] = u8::from_str_radix(&trimmed[i * 2..i * 2 + 2], 16).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: coldcard-multisig: parse error: line {line_no}: \
                 {context} hex parse: {e}"
            ))
        })?;
    }
    Ok(Fingerprint::from(bytes))
}

/// Predicate: is the given line-key purely 8 hex characters (cosigner XFP
/// prefix per shared-derivation shape)? Used to distinguish `<XFP>:` lines
/// from `Name:` / `Policy:` / etc.
fn is_xfp_hex(s: &str) -> bool {
    s.len() == 8 && s.chars().all(|c| c.is_ascii_hexdigit())
}

/// Strip the leading `m` from a derivation path string and return the
/// `/N'/N'/...` component suffix for use in the `[fp<components>]` bracket
/// form of the descriptor body. E.g., `m/48'/0'/0'/2'` → `/48'/0'/0'/2'`.
/// Accepts both `m/...` and bare-`m`.
fn derivation_path_components(s: &str) -> String {
    let trimmed = s.trim();
    if let Some(stripped) = trimmed.strip_prefix('m') {
        stripped.to_string()
    } else if !trimmed.is_empty() && !trimmed.starts_with('/') {
        // No leading `m/`; assume the user provided `48'/0'/0'/2'` (rare).
        format!("/{trimmed}")
    } else {
        trimmed.to_string()
    }
}

/// Build the descriptor body string (sans `#<csum>`) given the script
/// format + cosigners. Uses `sortedmulti` per Coldcard convention (every
/// Coldcard multisig export emits sortedmulti — the lexicographic key sort
/// is part of the script-image canonical form).
fn build_descriptor_body(
    k: u8,
    cosigners: &[ResolvedCosigner],
    script_format: ColdcardMsFormat,
) -> Result<String, ToolkitError> {
    if cosigners.is_empty() {
        return Err(ToolkitError::ImportWalletParse(
            "import-wallet: coldcard-multisig: parse error: zero cosigners after parse".to_string(),
        ));
    }
    let key_parts: Vec<String> = cosigners
        .iter()
        .map(|c| format!("{}{}/<0;1>/*", c.path_raw, c.xpub))
        .collect();
    let inner = format!("sortedmulti({k},{})", key_parts.join(","));
    let wrapped = match script_format {
        ColdcardMsFormat::P2wsh => format!("wsh({inner})"),
        ColdcardMsFormat::P2shP2wsh => format!("sh(wsh({inner}))"),
        ColdcardMsFormat::P2sh => format!("sh({inner})"),
    };
    Ok(wrapped)
}

/// Compute BIP-380 checksum (8-char bech32-style suffix) for a descriptor
/// body using miniscript's `ChecksumEngine`. Wraps any error as
/// `ImportWalletParse`.
fn compute_bip380_checksum(body: &str) -> Result<String, ToolkitError> {
    let mut eng = ChecksumEngine::new();
    eng.input(body).map_err(|e| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: coldcard-multisig: parse error: BIP-380 checksum engine: {e}"
        ))
    })?;
    Ok(eng.checksum())
}

/// Network detection per BIP-48 coin-type child number at path component
/// index 1 (hardened). Mirrors `bsms::network_from_origins` semantics.
fn network_from_path(path: &DerivationPath) -> Result<bitcoin::Network, ToolkitError> {
    let ct = coin_type_index(path)?;
    match ct {
        0 => Ok(bitcoin::Network::Bitcoin),
        1 => Ok(bitcoin::Network::Testnet),
        other => Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: coldcard-multisig: parse error: unsupported coin-type \
             {other} on derivation path; only 0 (mainnet) and 1 (testnet) supported per BIP-48"
        ))),
    }
}

/// Extract the coin-type index from a BIP-48 derivation path (component
/// index 1, must be hardened). Returns the raw u32 index.
fn coin_type_index(path: &DerivationPath) -> Result<u32, ToolkitError> {
    let comps: Vec<&ChildNumber> = path.into_iter().collect();
    if comps.len() < 2 {
        return Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: coldcard-multisig: parse error: derivation path has only {} \
             components; need ≥2 for BIP-48 coin-type inference",
            comps.len()
        )));
    }
    match comps[1] {
        ChildNumber::Hardened { index } => Ok(*index),
        ChildNumber::Normal { index } => Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: coldcard-multisig: parse error: coin-type component {index} is \
             not hardened; BIP-48 requires `<coin_type>'`"
        ))),
    }
}

/// Extract the "key" portion of a line of the form `Key: value`. Returns
/// `Some("Key")` (trimmed of surrounding whitespace, case-preserved) when
/// the line matches `^<word>:<rest>`; returns `None` for blank lines,
/// comment lines (`# …`), or lines without a `:` separator.
///
/// Used by both sniff (header-presence check) and parse (line classification).
pub(super) fn line_key(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return None;
    }
    let colon = trimmed.find(':')?;
    let key = trimmed[..colon].trim();
    if key.is_empty() {
        return None;
    }
    Some(key)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    /// One-off helper to compute XFP for fixture xpubs. Run via
    /// `cargo test --bin mnemonic xfp_compute -- --nocapture`. Output is
    /// then hardcoded into the .txt fixture files.
    /// Documentation regression: compute and pin the
    /// `bitcoin::bip32::Xpub::fingerprint()` value for each fixture xpub.
    /// SPEC §11.4.1's truth table relies on this exact formula; if the
    /// upstream `bitcoin` crate ever changes the fingerprint algorithm,
    /// this test will fail and surface the wire-shape break to the
    /// fixture authors. Pinned values are also referenced in the
    /// integration-test cells at `tests/cli_import_wallet_coldcard_multisig.rs`
    /// (Phase P4C) for byte-exact stderr assertions on the xfp-divergence
    /// WARNING path.
    #[test]
    fn xfp_fixture_xpubs_pinned_fingerprints() {
        let pairs: &[(&str, &str)] = &[
            (
                "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX",
                "34a3a4f1",
            ),
            (
                "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6",
                "ff9dfbcf",
            ),
            (
                "xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx",
                "b7f7dfea",
            ),
            (
                "xpub68Gmy5EdvgibQVfPdqkBBCHxA5htiqg55crXYuXoQRKfDBFA1WEjWgP6LHhwBZeNK1VTsfTFUHCdrfp1bgwQ9xv5ski8PX9rL2dZXvgGDnw",
                "5c1bd648",
            ),
            (
                "xpub6BgBgsespWvERF3LHQu6CnqdvfEvtMcQjYrcRzx53QJjSxarj2afYWcLteoGVky7D3UKDP9QyrLprQ3VCECoY49yfdDEHGCtMMj92pReUsQ",
                "a7bea80d",
            ),
            (
                "tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC",
                "8e3836c1",
            ),
            (
                "tpubDFiXyf7zmBhQrSHoAQB6SmMpF3rfSihAxQGMdQUtZfE8HWHkWLLNLTiYpMzvHnFiTmuUSYieHUYv4tFguzmiHeDrYV8TtWGCWt5qpqox4w3",
                "1dd94239",
            ),
        ];
        for (x, expected_fp) in pairs {
            let xpub = bitcoin::bip32::Xpub::from_str(x).unwrap();
            assert_eq!(
                xpub.fingerprint().to_string(),
                *expected_fp,
                "fixture xpub {x} fingerprint must match pinned value (SPEC §11.4.1)"
            );
        }
    }

    /// SPEC §11.4 sniff: shared-derivation shape (toolkit's own emit form).
    /// Headers in order: Name / Policy / Derivation / Format.
    #[test]
    fn sniff_true_on_shared_derivation_shape() {
        let blob = b"Name: testwallet\n\
Policy: 2 of 3\n\
Derivation: m/48'/0'/0'/2'\n\
Format: P2WSH\n\
\n\
B8688DF1: xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX\n";
        assert!(ColdcardMultisigParser::sniff(blob));
    }

    /// SPEC §11.4 sniff: per-cosigner-derivation shape (older firmware).
    #[test]
    fn sniff_true_on_per_cosigner_shape() {
        let blob = b"Name: testwallet\n\
Policy: 2-of-3\n\
Format: P2WSH\n\
Derivation: m/48'/0'/0'/2'\n\
xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX\n";
        assert!(ColdcardMultisigParser::sniff(blob));
    }

    /// SPEC §11.4 sniff: optional `XFP:` header line accepted (firmware-variance).
    #[test]
    fn sniff_true_with_xfp_header() {
        let blob = b"XFP: B8688DF1\n\
Name: testwallet\n\
Policy: 2 of 3\n\
Format: P2WSH\n\
Derivation: m/48'/0'/0'/2'\n";
        assert!(ColdcardMultisigParser::sniff(blob));
    }

    /// SPEC §11.4 sniff: CRLF blobs (Windows line endings) accepted.
    #[test]
    fn sniff_true_on_crlf() {
        let blob = b"Name: t\r\nPolicy: 2 of 3\r\nFormat: P2WSH\r\nDerivation: m/0\r\n";
        assert!(ColdcardMultisigParser::sniff(blob));
    }

    /// SPEC §11.4 sniff: missing `Format:` → false.
    #[test]
    fn sniff_false_on_missing_format() {
        let blob = b"Name: t\nPolicy: 2 of 3\nDerivation: m/0\n";
        assert!(!ColdcardMultisigParser::sniff(blob));
    }

    /// SPEC §11.4 sniff: missing `Name:` → false.
    #[test]
    fn sniff_false_on_missing_name() {
        let blob = b"Policy: 2 of 3\nFormat: P2WSH\nDerivation: m/0\n";
        assert!(!ColdcardMultisigParser::sniff(blob));
    }

    /// SPEC §11.4 sniff: missing `Policy:` → false.
    #[test]
    fn sniff_false_on_missing_policy() {
        let blob = b"Name: t\nFormat: P2WSH\nDerivation: m/0\n";
        assert!(!ColdcardMultisigParser::sniff(blob));
    }

    /// SPEC §11.4 sniff: BSMS blob → false (no Name/Policy/Format headers).
    /// Critical — BSMS parser owns this shape; ColdcardMultisig must not
    /// co-fire with BSMS.
    #[test]
    fn sniff_false_on_bsms_blob() {
        let blob = b"BSMS 1.0\nwsh(sortedmulti(2,...))#abcdefgh\n";
        assert!(!ColdcardMultisigParser::sniff(blob));
    }

    /// SPEC §11.4 sniff: Bitcoin Core JSON blob → false (JSON shape rejected
    /// at line-key extraction; `{` is not a key).
    #[test]
    fn sniff_false_on_bitcoin_core_json() {
        let blob = br#"{"wallet_name":"x","descriptors":[{"desc":"wpkh(xpub...)#abcdefgh"}]}"#;
        assert!(!ColdcardMultisigParser::sniff(blob));
    }

    /// SPEC §11.4 sniff: empty blob → false.
    #[test]
    fn sniff_false_on_empty_blob() {
        assert!(!ColdcardMultisigParser::sniff(b""));
    }

    /// SPEC §11.4 sniff: non-UTF-8 blob → false (Coldcard text export is
    /// ASCII; non-UTF-8 input cannot be a valid Coldcard multisig export).
    #[test]
    fn sniff_false_on_non_utf8() {
        let blob = &[0xFF, 0xFE, 0xFD, b'\n'];
        assert!(!ColdcardMultisigParser::sniff(blob));
    }

    /// SPEC §11.4 sniff: random text without Name/Policy/Format → false.
    #[test]
    fn sniff_false_on_random_text() {
        let blob = b"hello world\nlorem ipsum\n";
        assert!(!ColdcardMultisigParser::sniff(blob));
    }

    /// SPEC §11.4 sniff: comment lines + blank lines tolerated.
    #[test]
    fn sniff_true_with_leading_comments() {
        let blob = b"# exported from Coldcard\n\
\n\
Name: t\n\
Policy: 2 of 3\n\
Format: P2WSH\n\
Derivation: m/0\n";
        assert!(ColdcardMultisigParser::sniff(blob));
    }

    /// `line_key` helper: well-formed key:value line → Some(key).
    #[test]
    fn line_key_extracts_key_for_wellformed_line() {
        assert_eq!(line_key("Name: testwallet"), Some("Name"));
        assert_eq!(line_key("Policy: 2 of 3"), Some("Policy"));
        assert_eq!(line_key("  Format: P2WSH"), Some("Format"));
        assert_eq!(line_key("XFP: DEADBEEF"), Some("XFP"));
    }

    /// `line_key` helper: blank/comment/no-colon lines → None.
    #[test]
    fn line_key_rejects_blank_or_comment_or_keyless_lines() {
        assert_eq!(line_key(""), None);
        assert_eq!(line_key("   "), None);
        assert_eq!(line_key("# a comment"), None);
        assert_eq!(line_key("just a single line"), None);
        assert_eq!(line_key(":no key"), None);
    }

    /// `line_key` helper: per-cosigner xpub line (single base58 token) → None.
    /// The xpub itself contains no colon, so it routes to the "value" arm at
    /// parse time, not the header arm.
    #[test]
    fn line_key_rejects_bare_xpub_line() {
        let xpub = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
        assert_eq!(line_key(xpub), None);
    }

    // ---- P4B parser body tests ----

    const XPUB_A: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
    const XPUB_B: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";
    const XPUB_C: &str = "xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx";

    // Computed (xpub.fingerprint()) values for the above; pinned at
    // `xfp_fixture_xpubs_pinned_fingerprints` regression cell above.
    const FP_A: &str = "34A3A4F1";
    const FP_B: &str = "FF9DFBCF";
    const FP_C: &str = "B7F7DFEA";

    // cycle-13a (H14 depth-gated truth table): genuine DEPTH-0 master xpubs.
    // The account xpubs above (XPUB_A/B depth 4, XPUB_C depth 3 — all >0) can
    // no longer feed the Row-2/Row-4 warn/silent-computed fixtures, because the
    // H14 matrix gates `xpub.fingerprint()`-as-master-fp on `xpub.depth == 0`
    // (a depth>0 xpub's own fingerprint is NOT the master fp, so the disagree
    // warning would be meaningless / the no-XFP case now REFUSES). These three
    // are deterministically generated BIP-32 master (depth-0) xpubs; their
    // `FP_D0_*` values are `Xpub::fingerprint()` (= master id at depth 0),
    // verified at `depth0_const_fingerprints_pinned` below.
    const XPUB_D0_A: &str = "xpub661MyMwAqRbcGQ5dEWgzwBWpcFA5Uc2TKjZy6gqBoHgMGBKn91Q7ooXXCk2cdjU6nh1GW5tF7ttjKiYg2RJ5ybBZscgMqLE7RevfHn4J1jS";
    const FP_D0_A: &str = "57ACB302";
    const XPUB_D0_B: &str = "xpub661MyMwAqRbcF4AYTpoZvFuiLUyBtTmtVhoUVutzfJzeCFvNFjRcdtLLaWgDb7gwHmLBTV6gZf4T9rqSnxcu8hcmLigphSiFioYqVFcRSEZ";
    const FP_D0_B: &str = "0734F923";
    const XPUB_D0_C: &str = "xpub661MyMwAqRbcFCaP8gCBrgGNeaypTxHjEXKShDXjuEMf33MdFcJvStdLeHJjXygp28GU1fTNTTsRmJVRnm3RyQ2S2BFFaT8btZLTwWupf3a";
    const FP_D0_C: &str = "689B0FA9";

    /// cycle-13a: pin the depth byte == 0 AND `Xpub::fingerprint()` of the
    /// new depth-0 master-xpub consts against the vendored bitcoin crate (the
    /// authoritative source for the H14 discriminator). Guards the generated
    /// constants from silently drifting wrong.
    #[test]
    fn depth0_const_fingerprints_pinned() {
        for (xpub, want_fp) in [
            (XPUB_D0_A, FP_D0_A),
            (XPUB_D0_B, FP_D0_B),
            (XPUB_D0_C, FP_D0_C),
        ] {
            let x = Xpub::from_str(xpub).expect("depth-0 const xpub must parse");
            assert_eq!(x.depth, 0, "const must be a depth-0 master xpub");
            assert_eq!(
                x.fingerprint().to_string().to_uppercase(),
                want_fp,
                "pinned FP_D0_* must equal Xpub::fingerprint()"
            );
        }
    }

    // ====================================================================
    // cycle-13a H14 — depth-gated master-fingerprint truth table (P1)
    // SPEC §11.4.1. The discriminator is `xpub.depth == 0`: only a depth-0
    // master xpub's own `fingerprint()` IS the BIP-380 master fp.
    // ====================================================================

    /// #6 (H14-b): depth>0 account xpub, shared `Derivation:`, NO `XFP:`
    /// header, bare xpub → the master fingerprint is UNRECOVERABLE from an
    /// account xpub → REFUSE (`ImportWalletParse`, exit 2). RED today
    /// (silently substitutes the account-key id as the master fp, Row 4).
    #[test]
    fn import_coldcard_multisig_depth_gt0_no_xfp_refuses() {
        // XPUB_A is depth 4 (m/48'/0'/0'/2' account xpub), no XFP anywhere.
        let blob = format!(
            "Name: T\n\
Policy: 1 of 1\n\
Derivation: m/48'/0'/0'/2'\n\
Format: P2WSH\n\
{XPUB_A}\n"
        );
        let mut stderr = Vec::new();
        let err = parse_text(blob.as_bytes(), &mut stderr).unwrap_err();
        match &err {
            ToolkitError::ImportWalletParse(msg) => {
                assert!(
                    msg.contains("depth") && msg.contains("master fingerprint"),
                    "H14-b refusal must cite depth + master fingerprint; got: {msg}"
                );
            }
            other => panic!("expected ImportWalletParse (exit 2), got: {other:?}"),
        }
    }

    /// #7 (H14-c): depth>0 account xpub WITH a top-level `XFP:` header that
    /// differs from `xpub.fingerprint()` → use the SUPPLIED master fp,
    /// stderr SILENT (no disagreement warning — the account-key id can never
    /// equal the master fp at depth>0, so the comparison is meaningless).
    /// RED today (Row 2 fires a spurious warning).
    #[test]
    fn import_coldcard_multisig_depth_gt0_with_header_xfp_no_warning() {
        // XPUB_A depth 4; header XFP DEADBEEF (synthetic master) != FP_A.
        let blob = format!(
            "Name: T\n\
Policy: 1 of 1\n\
Derivation: m/48'/0'/0'/2'\n\
Format: P2WSH\n\
XFP: DEADBEEF\n\
{XPUB_A}\n"
        );
        let mut stderr = Vec::new();
        let p = parse_text(blob.as_bytes(), &mut stderr).unwrap();
        let meta = match &p.provenance {
            ImportProvenance::ColdcardMultisig(m) => m,
            other => panic!("expected ColdcardMultisig provenance, got {other:?}"),
        };
        assert!(
            !meta.xfp_header_disagreed,
            "depth>0 + supplied XFP must NOT set xfp_header_disagreed (H14-c)"
        );
        assert!(
            stderr.is_empty(),
            "depth>0 + supplied XFP must be SILENT (H14-c); got: {:?}",
            String::from_utf8_lossy(&stderr)
        );
        assert_eq!(
            p.cosigners[0].fingerprint.to_string().to_uppercase(),
            "DEADBEEF",
            "must use the supplied master fp at depth>0"
        );
    }

    /// #8 (H14-c): depth>0 account xpub with a PER-LINE `<XFP>: <xpub>`
    /// master fp != computed → use supplied, SILENT, exit 0. RED today.
    #[test]
    fn import_coldcard_multisig_depth_gt0_per_line_xfp_no_warning() {
        // CAFEBABE per-line master fp on a depth-4 xpub.
        let blob = format!(
            "Name: T\n\
Policy: 1 of 1\n\
Derivation: m/48'/0'/0'/2'\n\
Format: P2WSH\n\
\n\
CAFEBABE: {XPUB_A}\n"
        );
        let mut stderr = Vec::new();
        let p = parse_text(blob.as_bytes(), &mut stderr).unwrap();
        let meta = match &p.provenance {
            ImportProvenance::ColdcardMultisig(m) => m,
            other => panic!("expected ColdcardMultisig provenance, got {other:?}"),
        };
        assert!(
            !meta.xfp_header_disagreed,
            "depth>0 + per-line XFP must NOT set xfp_header_disagreed (H14-c)"
        );
        assert!(
            stderr.is_empty(),
            "depth>0 + per-line XFP must be SILENT (H14-c); got: {:?}",
            String::from_utf8_lossy(&stderr)
        );
        assert_eq!(
            p.cosigners[0].fingerprint.to_string().to_uppercase(),
            "CAFEBABE"
        );
    }

    /// #9 (H14-a): genuine DEPTH-0 master xpub, NO XFP → use
    /// `xpub.fingerprint()` (= master fp at depth 0) SILENTLY, exit 0.
    /// Guards against over-refusing (the H14-b refusal must NOT fire at
    /// depth 0). GREEN both before and after the fix at depth 0.
    #[test]
    fn import_coldcard_multisig_depth0_no_xfp_uses_computed() {
        let blob = format!(
            "Name: T\n\
Policy: 1 of 1\n\
Derivation: m/48'/0'/0'/2'\n\
Format: P2WSH\n\
{XPUB_D0_A}\n"
        );
        let mut stderr = Vec::new();
        let p = parse_text(blob.as_bytes(), &mut stderr).unwrap();
        assert!(
            stderr.is_empty(),
            "depth-0 + no-XFP must be silent (H14-a); got: {:?}",
            String::from_utf8_lossy(&stderr)
        );
        assert_eq!(
            p.cosigners[0].fingerprint.to_string().to_uppercase(),
            FP_D0_A,
            "depth-0 uses computed master fp"
        );
    }

    /// #10 (H14-d): DEPTH-0 master xpub, supplied XFP != computed → WARNING +
    /// use supplied (Row 2 stays meaningful at depth 0 — the computed fp IS
    /// comparable to the master XFP). Guards against under-warning.
    #[test]
    fn import_coldcard_multisig_depth0_xfp_mismatch_warns() {
        // XFP DEADBEEF != FP_D0_A computed at depth 0.
        let blob = format!(
            "Name: T\n\
Policy: 1 of 1\n\
Derivation: m/48'/0'/0'/2'\n\
Format: P2WSH\n\
XFP: DEADBEEF\n\
{XPUB_D0_A}\n"
        );
        let mut stderr = Vec::new();
        let p = parse_text(blob.as_bytes(), &mut stderr).unwrap();
        let meta = match &p.provenance {
            ImportProvenance::ColdcardMultisig(m) => m,
            other => panic!("expected ColdcardMultisig provenance, got {other:?}"),
        };
        assert!(
            meta.xfp_header_disagreed,
            "depth-0 XFP mismatch must set xfp_header_disagreed (H14-d)"
        );
        let s = String::from_utf8_lossy(&stderr);
        assert!(
            s.contains("warning: import-wallet: coldcard-multisig: xfp header"),
            "depth-0 mismatch must WARN (H14-d); got: {s}"
        );
        assert!(s.contains("DEADBEEF"), "warning must cite header; got: {s}");
        assert!(s.contains(FP_D0_A), "warning must cite computed; got: {s}");
        assert_eq!(
            p.cosigners[0].fingerprint.to_string().to_uppercase(),
            "DEADBEEF",
            "depth-0 mismatch uses supplied (header authoritative)"
        );
    }

    // ====================================================================
    // cycle-13a P2 (Q1) — `<XFP>:` arm consumes the pending per-line path
    // without clearing `shared_derivation`.
    // ====================================================================

    /// #13 (Q1 condition ii, regression guard): one shared `Derivation:` +
    /// 3 `<XFP>: <xpub>` lines (NO per-line `Derivation:`) → ALL 3 cosigners
    /// resolve to the SHARED path. Proves the extended `<XFP>:` arm does NOT
    /// clear `shared_derivation`, so cosigners 2..N still fall back to it.
    /// GREEN both BEFORE and AFTER the arm change (it asserts an invariant the
    /// change must preserve).
    #[test]
    fn import_coldcard_multisig_shared_derivation_3_cosigners_all_resolve_shared() {
        // Depth-0 master xpubs with matching per-line XFPs → silent (H14-a/d
        // not triggered; per-line XFP == computed). Shared Derivation precedes
        // the 3 cosigner lines; none carries its own per-line `Derivation:`.
        let blob = format!(
            "Name: T\n\
Policy: 2 of 3\n\
Derivation: m/48'/0'/0'/2'\n\
Format: P2WSH\n\
\n\
{FP_D0_A}: {XPUB_D0_A}\n\
{FP_D0_B}: {XPUB_D0_B}\n\
{FP_D0_C}: {XPUB_D0_C}\n"
        );
        let mut stderr = Vec::new();
        let p = parse_text(blob.as_bytes(), &mut stderr).unwrap();
        for (i, cs) in p.cosigners.iter().enumerate() {
            assert_eq!(
                cs.path.to_string(),
                "48'/0'/0'/2'",
                "cosigner {i} must resolve to the SHARED derivation path"
            );
        }
        assert!(
            stderr.is_empty(),
            "matching per-line XFPs at depth 0 must be silent; got: {:?}",
            String::from_utf8_lossy(&stderr)
        );
    }

    /// #13b (Q1 positive RED): interleaved per-line `Derivation:` + `<XFP>:`
    /// pairs with DISTINCT paths → each cosigner resolves to its OWN per-line
    /// path. RED before the arm change (the `<XFP>:` arm set `per_line_path:
    /// None` + cleared pending → both fell back to the shared/first path).
    /// GREEN after the arm consumes the pending per-line path. Depth-0 xpubs +
    /// matching XFPs (M-3) so no incidental Row-2 warning muddies RED→GREEN.
    #[test]
    fn import_coldcard_multisig_per_line_derivation_plus_xfp_roundtrips_path() {
        let blob = format!(
            "Name: T\n\
Policy: 1 of 2\n\
Format: P2WSH\n\
Derivation: m/48'/0'/0'/2'\n\
{FP_D0_A}: {XPUB_D0_A}\n\
Derivation: m/48'/0'/1'/2'\n\
{FP_D0_B}: {XPUB_D0_B}\n"
        );
        let mut stderr = Vec::new();
        let p = parse_text(blob.as_bytes(), &mut stderr).unwrap();
        assert_eq!(
            p.cosigners[0].path.to_string(),
            "48'/0'/0'/2'",
            "cosigner 0 must keep its per-line path m/48'/0'/0'/2'"
        );
        assert_eq!(
            p.cosigners[1].path.to_string(),
            "48'/0'/1'/2'",
            "cosigner 1 must keep its DISTINCT per-line path m/48'/0'/1'/2'"
        );
    }

    /// SPEC §11.4 happy-path: shared-derivation shape, no XFP header, all
    /// per-cosigner `<XFP>: <xpub>` fingerprints match xpub.fingerprint().
    /// Row 1 of the truth table (silent). Telemetry: xfp_was_blob_supplied=
    /// true (via the per-cosigner `<XFP>:` form), xfp_header_disagreed=false.
    #[test]
    fn parse_shared_derivation_no_xfp_header_silent() {
        let blob = format!(
            "Name: T\n\
Policy: 2 of 3\n\
Derivation: m/48'/0'/0'/2'\n\
Format: P2WSH\n\
\n\
{FP_A}: {XPUB_A}\n\
{FP_B}: {XPUB_B}\n\
{FP_C}: {XPUB_C}\n"
        );
        let mut stderr = Vec::new();
        let p = parse_text(blob.as_bytes(), &mut stderr).unwrap();
        let meta = match &p.provenance {
            ImportProvenance::ColdcardMultisig(m) => m,
            other => panic!("expected ColdcardMultisig provenance, got {other:?}"),
        };
        assert_eq!(meta.name, "T");
        assert_eq!(meta.policy, PolicyKOfN { k: 2, n: 3 });
        assert_eq!(meta.script_format, ColdcardMsFormat::P2wsh);
        assert!(meta.xfp_was_blob_supplied);
        assert!(!meta.xfp_header_disagreed);
        assert_eq!(p.network, bitcoin::Network::Bitcoin);
        assert_eq!(p.threshold, Some(2));
        assert_eq!(p.cosigners.len(), 3);
        // Stderr should be silent on row 1.
        assert!(
            stderr.is_empty(),
            "stderr must be silent on row-1 match; got {:?}",
            String::from_utf8_lossy(&stderr)
        );
    }

    /// SPEC §11.4.1 row 2 (H14-d at depth 0): header present + computed
    /// available + MISMATCH. WARNING fires + header is authoritative.
    /// `xfp_header_disagreed=true`. cycle-13a: the cosigner xpub is a genuine
    /// DEPTH-0 master xpub (`XPUB_D0_A`) — the Row-2 disagreement warning is
    /// only meaningful at depth 0, where `xpub.fingerprint()` IS comparable to
    /// the supplied master XFP. (Pre-cycle-13a this fixture used a depth-4
    /// account xpub, which now takes the silent H14-c arm.)
    #[test]
    fn parse_xfp_header_mismatch_warns_uses_header() {
        // Top-level XFP header DEADBEEF disagrees with the depth-0 master
        // xpub's computed fingerprint FP_D0_A.
        let blob = format!(
            "Name: T\n\
Policy: 1 of 1\n\
Derivation: m/48'/0'/0'/2'\n\
Format: P2WSH\n\
XFP: DEADBEEF\n\
\n\
{XPUB_D0_A}\n" // bare xpub → supplied = header DEADBEEF, computed = FP_D0_A (mismatch).
        );
        let mut stderr = Vec::new();
        let p = parse_text(blob.as_bytes(), &mut stderr).unwrap();
        let meta = match &p.provenance {
            ImportProvenance::ColdcardMultisig(m) => m,
            other => panic!("expected ColdcardMultisig provenance, got {other:?}"),
        };
        assert!(meta.xfp_was_blob_supplied);
        assert!(meta.xfp_header_disagreed, "row-2 must set disagreed flag");
        let s = String::from_utf8_lossy(&stderr);
        assert!(
            s.contains("warning: import-wallet: coldcard-multisig: xfp header"),
            "row-2 WARNING template missing; got: {s}"
        );
        assert!(
            s.contains("DEADBEEF"),
            "row-2 WARNING must cite header value DEADBEEF; got: {s}"
        );
        assert!(
            s.contains(FP_D0_A),
            "row-2 WARNING must cite computed value {FP_D0_A}; got: {s}"
        );
        assert!(
            s.contains("using blob-supplied header value as authoritative"),
            "row-2 WARNING must cite 'authoritative' clause; got: {s}"
        );
        // Cosigner fingerprint is the header value (DEADBEEF) per SPEC §11.4.1.
        assert_eq!(
            p.cosigners[0].fingerprint.to_string().to_uppercase(),
            "DEADBEEF"
        );
    }

    /// SPEC §11.4.1 H14-a (formerly "row 4"): no header + computed available
    /// AND the xpub IS the DEPTH-0 master → `xpub.fingerprint()` IS the master
    /// fp → use it SILENTLY. cycle-13a SPLIT (a): re-pointed to genuine depth-0
    /// master xpubs (`XPUB_D0_*`) so the "uses computed silently" assertion
    /// stays valid — at depth 0 the computed fp legitimately IS the master fp.
    /// (The depth>0/no-XFP companion now REFUSES — see the SPLIT (b) test
    /// `parse_no_header_depth_gt0_refuses` directly below.)
    #[test]
    fn parse_no_header_no_per_cosigner_xfp_uses_computed_silent() {
        // Per-cosigner shape: `Derivation:` before each bare xpub; no XFP
        // anywhere; all three cosigners are DEPTH-0 masters → computed-from-
        // xpub path (H14-a). `xfp_was_blob_supplied` telemetry stays `false`.
        let blob = format!(
            "Name: T\n\
Policy: 2 of 3\n\
Format: P2WSH\n\
Derivation: m/48'/0'/0'/2'\n\
{XPUB_D0_A}\n\
Derivation: m/48'/0'/0'/2'\n\
{XPUB_D0_B}\n\
Derivation: m/48'/0'/0'/2'\n\
{XPUB_D0_C}\n"
        );
        let mut stderr = Vec::new();
        let p = parse_text(blob.as_bytes(), &mut stderr).unwrap();
        let meta = match &p.provenance {
            ImportProvenance::ColdcardMultisig(m) => m,
            other => panic!("expected ColdcardMultisig provenance, got {other:?}"),
        };
        assert!(
            !meta.xfp_was_blob_supplied,
            "H14-a (no XFP anywhere) must set xfp_was_blob_supplied=false"
        );
        assert!(!meta.xfp_header_disagreed);
        assert!(
            stderr.is_empty(),
            "H14-a (depth-0, no XFP) must be silent; got: {:?}",
            String::from_utf8_lossy(&stderr)
        );
        // All cosigners use computed (= master at depth 0) fingerprints.
        assert_eq!(
            p.cosigners[0].fingerprint.to_string().to_uppercase(),
            FP_D0_A
        );
        assert_eq!(
            p.cosigners[1].fingerprint.to_string().to_uppercase(),
            FP_D0_B
        );
        assert_eq!(
            p.cosigners[2].fingerprint.to_string().to_uppercase(),
            FP_D0_C
        );
    }

    /// SPEC §11.4.1 H14-b: no header + computed available BUT the cosigner
    /// xpub is at depth>0 (an account xpub) → the master fp is UNRECOVERABLE
    /// → REFUSE (`ImportWalletParse`, exit 2). cycle-13a SPLIT (b): this is
    /// the silent-substitute → refuse flip that the lane is built on (the
    /// former `parse_no_header_no_per_cosigner_xfp_uses_computed_silent` used
    /// depth-4 xpubs with NO XFP and silently substituted the account-key id).
    #[test]
    fn parse_no_header_depth_gt0_refuses() {
        // Same shape as the H14-a test but with DEPTH-4 account xpubs and no
        // XFP anywhere → must refuse, not silently substitute.
        let blob = format!(
            "Name: T\n\
Policy: 2 of 3\n\
Format: P2WSH\n\
Derivation: m/48'/0'/0'/2'\n\
{XPUB_A}\n\
Derivation: m/48'/0'/0'/2'\n\
{XPUB_B}\n\
Derivation: m/48'/0'/0'/2'\n\
{XPUB_C}\n"
        );
        let mut stderr = Vec::new();
        let err = parse_text(blob.as_bytes(), &mut stderr).unwrap_err();
        match &err {
            ToolkitError::ImportWalletParse(msg) => {
                assert!(
                    msg.contains("depth") && msg.contains("master fingerprint"),
                    "H14-b refusal must cite depth + master fingerprint; got: {msg}"
                );
            }
            other => panic!("expected ImportWalletParse (exit 2), got: {other:?}"),
        }
    }

    /// SPEC §11.4.1 row 5: no header + xpub malformed → hard error citing
    /// "cannot compute xfp".
    #[test]
    fn parse_no_header_malformed_xpub_hard_errors() {
        let blob = "Name: T\n\
Policy: 1 of 1\n\
Format: P2WSH\n\
Derivation: m/48'/0'/0'/2'\n\
not-a-valid-base58-xpub\n";
        let mut stderr = Vec::new();
        let err = parse_text(blob.as_bytes(), &mut stderr).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("cannot compute xfp"),
            "row-5 error must cite `cannot compute xfp`; got: {msg}"
        );
    }

    /// SPEC §11.4 happy-path: P2SH-P2WSH wrapper.
    #[test]
    fn parse_p2sh_p2wsh_format_wraps_descriptor() {
        let blob = format!(
            "Name: T\n\
Policy: 2 of 3\n\
Derivation: m/48'/0'/0'/1'\n\
Format: P2SH-P2WSH\n\
\n\
{FP_A}: {XPUB_A}\n\
{FP_B}: {XPUB_B}\n\
{FP_C}: {XPUB_C}\n"
        );
        let mut stderr = Vec::new();
        let p = parse_text(blob.as_bytes(), &mut stderr).unwrap();
        assert!(
            p.original_descriptor.starts_with("sh(wsh(sortedmulti(2,"),
            "P2SH-P2WSH must wrap with sh(wsh(...)); got: {}",
            p.original_descriptor
        );
        let meta = match &p.provenance {
            ImportProvenance::ColdcardMultisig(m) => m,
            other => panic!("expected ColdcardMultisig provenance, got {other:?}"),
        };
        assert_eq!(meta.script_format, ColdcardMsFormat::P2shP2wsh);
    }

    /// SPEC §11.4 refusal: missing `Format:` header.
    #[test]
    fn parse_missing_format_errors() {
        let blob = format!(
            "Name: T\n\
Policy: 2 of 3\n\
Derivation: m/48'/0'/0'/2'\n\
{FP_A}: {XPUB_A}\n"
        );
        let mut stderr = Vec::new();
        let err = parse_text(blob.as_bytes(), &mut stderr).unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("missing `Format:` header"), "got: {msg}");
    }

    /// SPEC §11.4 refusal: Policy mismatch (N declared 3 but 2 cosigners
    /// listed).
    #[test]
    fn parse_policy_n_mismatch_errors() {
        let blob = format!(
            "Name: T\n\
Policy: 2 of 3\n\
Derivation: m/48'/0'/0'/2'\n\
Format: P2WSH\n\
\n\
{FP_A}: {XPUB_A}\n\
{FP_B}: {XPUB_B}\n"
        );
        let mut stderr = Vec::new();
        let err = parse_text(blob.as_bytes(), &mut stderr).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("Policy declared 3 cosigners but 2 cosigner entries"),
            "got: {msg}"
        );
    }

    /// SPEC §11.4 refusal: Policy K out of range (K > N).
    #[test]
    fn parse_policy_k_out_of_range_errors() {
        let blob = format!(
            "Name: T\n\
Policy: 4 of 3\n\
Derivation: m/48'/0'/0'/2'\n\
Format: P2WSH\n\
\n\
{FP_A}: {XPUB_A}\n\
{FP_B}: {XPUB_B}\n\
{FP_C}: {XPUB_C}\n"
        );
        let mut stderr = Vec::new();
        let err = parse_text(blob.as_bytes(), &mut stderr).unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("threshold 4 out of range"), "got: {msg}");
    }

    /// SPEC §11.4 dash-form Policy accepted (`K-of-N`).
    #[test]
    fn parse_policy_dash_form_accepted() {
        let blob = format!(
            "Name: T\n\
Policy: 2-of-3\n\
Derivation: m/48'/0'/0'/2'\n\
Format: P2WSH\n\
\n\
{FP_A}: {XPUB_A}\n\
{FP_B}: {XPUB_B}\n\
{FP_C}: {XPUB_C}\n"
        );
        let mut stderr = Vec::new();
        let _p = parse_text(blob.as_bytes(), &mut stderr).unwrap();
    }

    /// Helper `parse_policy`: accepts `K of N` and `K-of-N`; rejects junk.
    #[test]
    fn parse_policy_helper_accepts_both_forms_rejects_junk() {
        assert_eq!(
            parse_policy("2 of 3", 1).unwrap(),
            PolicyKOfN { k: 2, n: 3 }
        );
        assert_eq!(
            parse_policy("2-of-3", 1).unwrap(),
            PolicyKOfN { k: 2, n: 3 }
        );
        assert!(parse_policy("malformed", 1).is_err());
        assert!(parse_policy("256 of 1", 1).is_err()); // u8 overflow
    }

    /// Helper `parse_script_format`: case-insensitive match; alias variants
    /// accepted for P2SH-P2WSH; junk rejected.
    #[test]
    fn parse_script_format_helper_accepts_aliases() {
        assert_eq!(
            parse_script_format("p2wsh", 1).unwrap(),
            ColdcardMsFormat::P2wsh
        );
        assert_eq!(
            parse_script_format("P2WSH", 1).unwrap(),
            ColdcardMsFormat::P2wsh
        );
        assert_eq!(
            parse_script_format("P2SH-P2WSH", 1).unwrap(),
            ColdcardMsFormat::P2shP2wsh
        );
        assert_eq!(
            parse_script_format("P2WSH-P2SH", 1).unwrap(),
            ColdcardMsFormat::P2shP2wsh
        );
        assert_eq!(
            parse_script_format("p2sh", 1).unwrap(),
            ColdcardMsFormat::P2sh
        );
        assert!(parse_script_format("P2TR", 1).is_err());
    }

    /// Helper `is_xfp_hex`: 8-hex-char predicate.
    #[test]
    fn is_xfp_hex_predicate() {
        assert!(is_xfp_hex("DEADBEEF"));
        assert!(is_xfp_hex("deadbeef"));
        assert!(is_xfp_hex("12345678"));
        assert!(!is_xfp_hex("DEADBEEFEXTRA"));
        assert!(!is_xfp_hex("DEAD"));
        assert!(!is_xfp_hex("ZZZZZZZZ")); // non-hex
        assert!(!is_xfp_hex("Name"));
    }

    /// Helper `derivation_path_components`: strip leading `m` correctly.
    #[test]
    fn derivation_path_components_strips_leading_m() {
        assert_eq!(
            derivation_path_components("m/48'/0'/0'/2'"),
            "/48'/0'/0'/2'"
        );
        assert_eq!(derivation_path_components("m"), "");
        assert_eq!(derivation_path_components("/48'/0'/0'/2'"), "/48'/0'/0'/2'");
    }

    /// Per-cosigner shape mixed with `<XFP>:` for a single cosigner where
    /// the per-line XFP DIVERGES from the computed value → row 2 WARNING.
    /// cycle-13a: re-pointed to a DEPTH-0 master xpub (`XPUB_D0_A`) — the
    /// Row-2/H14-d disagreement warning is only meaningful at depth 0 (at
    /// depth>0 a supplied per-line XFP takes the silent H14-c arm).
    #[test]
    fn parse_per_cosigner_xfp_divergence_warns() {
        let blob = format!(
            "Name: T\n\
Policy: 1 of 1\n\
Derivation: m/48'/0'/0'/2'\n\
Format: P2WSH\n\
\n\
CAFEBABE: {XPUB_D0_A}\n" // per-line XFP CAFEBABE vs computed FP_D0_A (depth 0)
        );
        let mut stderr = Vec::new();
        let p = parse_text(blob.as_bytes(), &mut stderr).unwrap();
        let meta = match &p.provenance {
            ImportProvenance::ColdcardMultisig(m) => m,
            other => panic!("expected ColdcardMultisig provenance, got {other:?}"),
        };
        assert!(meta.xfp_was_blob_supplied);
        assert!(meta.xfp_header_disagreed);
        let s = String::from_utf8_lossy(&stderr);
        assert!(s.contains("CAFEBABE"), "got: {s}");
        assert!(s.contains(FP_D0_A), "got: {s}");
        assert_eq!(
            p.cosigners[0].fingerprint.to_string().to_uppercase(),
            "CAFEBABE",
            "row-2: header value (per-line `<XFP>:`) wins"
        );
    }

    /// Fixture cell: `coldcard-ms-2of3-p2wsh-with-xfp.txt` parses cleanly +
    /// silently (row 1 across all cosigners).
    #[test]
    fn fixture_2of3_with_xfp_parses_silent() {
        let blob =
            std::fs::read("tests/fixtures/wallet_import/coldcard-ms-2of3-p2wsh-with-xfp.txt")
                .expect("fixture file readable");
        let mut stderr = Vec::new();
        let p = parse_text(&blob, &mut stderr).unwrap();
        let meta = match &p.provenance {
            ImportProvenance::ColdcardMultisig(m) => m,
            other => panic!("expected ColdcardMultisig provenance, got {other:?}"),
        };
        assert_eq!(meta.name, "TestMs2of3");
        assert_eq!(meta.policy, PolicyKOfN { k: 2, n: 3 });
        assert_eq!(meta.script_format, ColdcardMsFormat::P2wsh);
        assert!(meta.xfp_was_blob_supplied);
        assert!(!meta.xfp_header_disagreed);
        assert!(
            stderr.is_empty(),
            "stderr must be silent on the with-xfp fixture; got: {:?}",
            String::from_utf8_lossy(&stderr)
        );
    }

    /// Fixture cell: `coldcard-ms-2of3-p2wsh-no-xfp.txt` parses cleanly +
    /// silently (row 1 per-cosigner via `<XFP>: <xpub>` form, no top-level
    /// XFP header).
    #[test]
    fn fixture_2of3_no_xfp_header_parses_silent() {
        let blob = std::fs::read("tests/fixtures/wallet_import/coldcard-ms-2of3-p2wsh-no-xfp.txt")
            .expect("fixture file readable");
        let mut stderr = Vec::new();
        let p = parse_text(&blob, &mut stderr).unwrap();
        let meta = match &p.provenance {
            ImportProvenance::ColdcardMultisig(m) => m,
            other => panic!("expected ColdcardMultisig provenance, got {other:?}"),
        };
        assert_eq!(meta.policy, PolicyKOfN { k: 2, n: 3 });
        assert!(meta.xfp_was_blob_supplied);
        assert!(!meta.xfp_header_disagreed);
        assert!(stderr.is_empty());
    }

    /// Fixture cell: `coldcard-ms-3of5-p2wsh.txt` parses cleanly.
    #[test]
    fn fixture_3of5_parses_silent() {
        let blob = std::fs::read("tests/fixtures/wallet_import/coldcard-ms-3of5-p2wsh.txt")
            .expect("fixture file readable");
        let mut stderr = Vec::new();
        let p = parse_text(&blob, &mut stderr).unwrap();
        let meta = match &p.provenance {
            ImportProvenance::ColdcardMultisig(m) => m,
            other => panic!("expected ColdcardMultisig provenance, got {other:?}"),
        };
        assert_eq!(meta.policy, PolicyKOfN { k: 3, n: 5 });
        assert_eq!(p.cosigners.len(), 5);
        assert_eq!(p.threshold, Some(3));
        assert!(stderr.is_empty());
    }

    /// Fixture cell: `coldcard-ms-malformed-missing-format.txt` refuses
    /// citing the missing `Format:` header.
    #[test]
    fn fixture_malformed_missing_format_refuses() {
        let blob =
            std::fs::read("tests/fixtures/wallet_import/coldcard-ms-malformed-missing-format.txt")
                .expect("fixture file readable");
        let mut stderr = Vec::new();
        let err = parse_text(&blob, &mut stderr).unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("missing `Format:` header"), "got: {msg}");
    }

    /// Verify the synthesized descriptor body for the 2-of-3 P2WSH fixture
    /// matches the BIP-380-checksum-prefix-byte-exact form we expect:
    /// `wsh(sortedmulti(2,[fp/path]xpub/<0;1>/*,...))`.
    /// Compares against the same xpubs in the BSMS fixture
    /// `bsms-2line-sortedmulti-2of3.txt`. The fingerprints DIFFER (Coldcard
    /// XFP=xpub.fingerprint() vs BSMS bracket-fp=master), so we only assert
    /// the structure + presence of the xpubs.
    #[test]
    fn fixture_2of3_descriptor_structure() {
        let blob =
            std::fs::read("tests/fixtures/wallet_import/coldcard-ms-2of3-p2wsh-with-xfp.txt")
                .expect("fixture file readable");
        let mut stderr = Vec::new();
        let p = parse_text(&blob, &mut stderr).unwrap();
        let body = &p.original_descriptor;
        assert!(body.starts_with("wsh(sortedmulti(2,"), "got: {body}");
        assert!(body.contains(XPUB_A), "got: {body}");
        assert!(body.contains(XPUB_B), "got: {body}");
        assert!(body.contains(XPUB_C), "got: {body}");
        assert!(body.contains("/<0;1>/*"), "got: {body}");
        assert!(
            body.contains('#'),
            "must have a BIP-380 checksum; got: {body}"
        );
        // Bracket-form per-cosigner uses xpub.fingerprint(), not the bracket fp from BSMS.
        assert!(
            body.contains(&format!("[{}/48'/0'/0'/2']", FP_A.to_lowercase())),
            "got: {body}"
        );
    }

    /// Testnet parse: BIP-48 path `m/48'/1'/...'` → coin-type 1 → Network::Testnet.
    #[test]
    fn parse_testnet_path_sets_network_testnet() {
        let tpub_a = "tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC";
        let tpub_b = "tpubDFiXyf7zmBhQrSHoAQB6SmMpF3rfSihAxQGMdQUtZfE8HWHkWLLNLTiYpMzvHnFiTmuUSYieHUYv4tFguzmiHeDrYV8TtWGCWt5qpqox4w3";
        let blob = format!(
            "Name: T\n\
Policy: 2 of 2\n\
Derivation: m/48'/1'/0'/2'\n\
Format: P2WSH\n\
\n\
8E3836C1: {tpub_a}\n\
1DD94239: {tpub_b}\n"
        );
        let mut stderr = Vec::new();
        let p = parse_text(blob.as_bytes(), &mut stderr).unwrap();
        assert_eq!(p.network, bitcoin::Network::Testnet);
    }

    /// Cross-cosigner coin-type heterogeneity rejected.
    /// cycle-13a: the per-cosigner truth-table loop runs BEFORE the coin-type
    /// heterogeneity check, so a NO-XFP depth>0 blob would now refuse with the
    /// H14-b master-fp message before the coin-type validation is reached. We
    /// supply a top-level `XFP:` master fp (depth>0 → H14-c silent accept) so
    /// the truth-table loop passes and the coin-type cross-check stays the
    /// reached/exercised refusal point. The per-line `Derivation:` paths still
    /// diverge in coin-type (mainnet 0 vs testnet 1).
    #[test]
    fn parse_heterogeneous_coin_type_rejected() {
        let tpub_a = "tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC";
        // Mix mainnet xpub (coin-type 0) with testnet-pathed cosigner (coin-type 1).
        let blob = format!(
            "Name: T\n\
Policy: 2 of 2\n\
Format: P2WSH\n\
XFP: DEADBEEF\n\
Derivation: m/48'/0'/0'/2'\n\
{XPUB_A}\n\
Derivation: m/48'/1'/0'/2'\n\
{tpub_a}\n"
        );
        let mut stderr = Vec::new();
        let err = parse_text(blob.as_bytes(), &mut stderr).unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("coin-type") && msg.contains("must share a coin-type"),
            "got: {msg}"
        );
    }

    /// `parse()` (the `WalletFormatParser::parse` impl) wraps the
    /// `parse_text` helper and returns a 1-element Vec.
    #[test]
    fn parse_via_wallet_format_parser_returns_single_element_vec() {
        let blob = format!(
            "Name: T\n\
Policy: 2 of 3\n\
Derivation: m/48'/0'/0'/2'\n\
Format: P2WSH\n\
\n\
{FP_A}: {XPUB_A}\n\
{FP_B}: {XPUB_B}\n\
{FP_C}: {XPUB_C}\n"
        );
        let mut stderr = Vec::new();
        let v = ColdcardMultisigParser::parse(blob.as_bytes(), &mut stderr).unwrap();
        assert_eq!(v.len(), 1);
    }

    /// CRLF-normalized parse: a Windows-emitted blob with `\r\n` line
    /// endings parses identically to LF.
    #[test]
    fn parse_crlf_normalized() {
        let lf = format!(
            "Name: T\n\
Policy: 2 of 3\n\
Derivation: m/48'/0'/0'/2'\n\
Format: P2WSH\n\
\n\
{FP_A}: {XPUB_A}\n\
{FP_B}: {XPUB_B}\n\
{FP_C}: {XPUB_C}\n"
        );
        let crlf = lf.replace('\n', "\r\n");
        let mut e1 = Vec::new();
        let mut e2 = Vec::new();
        let p_lf = parse_text(lf.as_bytes(), &mut e1).unwrap();
        let p_crlf = parse_text(crlf.as_bytes(), &mut e2).unwrap();
        assert_eq!(p_lf.original_descriptor, p_crlf.original_descriptor);
    }
}
