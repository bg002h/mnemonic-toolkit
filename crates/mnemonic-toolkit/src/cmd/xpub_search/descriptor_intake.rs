//! Descriptor intake helper for `xpub-search account-of-descriptor`.
//!
//! Polymorphic descriptor shape parser per plan §4.2 + §4.2.1. Implements the
//! two-funnel approach because `md_codec::Descriptor → String` is not a
//! confirmed-existing API:
//!
//! - **String funnel** (literal-xpub + BIP-388 JSON): the canonical descriptor
//!   string is parsed via `rust_miniscript::Descriptor::from_str` and walked
//!   via `iter_pk` to extract `CosignerExtract` entries.
//! - **Tree-walk funnel** (md1 card(s)): chunks are reassembled via
//!   `md_codec::chunk::reassemble`, and the resulting `md_codec::Descriptor`'s
//!   `tlv.pubkeys` + `tlv.fingerprints` + `tlv.origin_path_overrides` +
//!   `path_decl.paths` are walked directly to produce `CosignerExtract`
//!   entries carrying the 65-byte form.
//!
//! Both funnels feed `account_search::match_descriptor_against_seed`.
//!
//! Tie-break order (plan §4.2 R1 I-5 lock, checked top-to-bottom):
//!   1. `trim_start().starts_with('{')` → BIP-388 JSON.
//!   2. all-bech32 tokens with `md1` HRP → md1 cards.
//!   3. contains `@\d+` outside string-literal context → toolkit-@N (refused).
//!   4. else → external literal-xpub.

use crate::error::ToolkitError;
use bitcoin::bip32::{DerivationPath, Fingerprint};
use miniscript::{Descriptor as MsDescriptor, DescriptorPublicKey};
use std::str::FromStr;

/// Canonical-form payload describing one cosigner position. Both funnels
/// produce a `Vec<CosignerExtract>` keyed by position so downstream
/// `match_descriptor_against_seed` is funnel-agnostic.
///
/// The `fingerprint_anno` / `derivation_path_anno` fields are not consumed by
/// `match_descriptor_against_seed` in C2 (the candidate-path enumeration
/// drives the search), but they're preserved for future use (e.g. early-prune
/// optimization when the descriptor annotation matches a candidate path).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CosignerExtract {
    /// Cosigner position (`@0`, `@1`, …) in source order.
    pub idx: usize,
    /// 65-byte canonical xpub payload (`[0..32] = chain_code, [32..65] =
    /// compressed pubkey`) per SPEC §4.6.1. None for NUMS sentinel.
    pub xpub_65: Option<[u8; 65]>,
    /// Origin fingerprint annotation when present.
    pub fingerprint_anno: Option<Fingerprint>,
    /// Derivation-path annotation when present.
    pub derivation_path_anno: Option<DerivationPath>,
    /// Flag set for taproot internal keys recognized as the NUMS sentinel.
    /// When `true`, the cosigner is skipped by the search and reported as
    /// `unspendable_internal_key: true` in the result.
    pub is_nums: bool,
}

/// Auto-detected descriptor shape. Drives the `descriptor_shape` field in
/// the JSON envelope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DescriptorShape {
    LiteralXpub,
    Md1,
    Bip388Json,
}

/// Result of descriptor intake: the cosigner extract list + the detected
/// shape (preserved for the JSON envelope).
#[derive(Debug, Clone)]
pub struct DescriptorIntake {
    pub cosigners: Vec<CosignerExtract>,
    pub shape: DescriptorShape,
    /// `true` when one or more cosigners had a missing path-anno (literal-xpub
    /// path only). Drives the v0.19.0 default-path-inference stderr notice.
    /// Indices reference the position in `cosigners`. Currently informational
    /// after the notice is emitted; preserved for future telemetry / JSON
    /// envelope extension.
    #[allow(dead_code)]
    pub defaulted_indices: Vec<u8>,
}

/// NUMS H-point x-only key (mirrors `parse_descriptor::NUMS_H_POINT_X_ONLY_HEX`).
const NUMS_H_POINT_X_ONLY_HEX: &str =
    "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

/// Parse the `--descriptor <value>` form with auto-detected shape.
pub fn intake_from_descriptor_value(
    value: &str,
    network: crate::network::CliNetwork,
    account: u32,
    stderr: &mut impl std::io::Write,
) -> Result<DescriptorIntake, ToolkitError> {
    let shape = detect_shape(value)?;
    intake_from_shape(value, shape, network, account, stderr)
}

/// Parse the `--descriptor-from <node>=<value>` explicit-form. The caller
/// has already split on `=`. Stdin sentinel `-` (e.g. `md1=-`) is handled by
/// reading from `stdin` (one chunk per line for md1; single string otherwise).
pub fn intake_from_explicit_form<R: std::io::Read>(
    node: &str,
    value: &str,
    network: crate::network::CliNetwork,
    account: u32,
    stdin: &mut R,
    stderr: &mut impl std::io::Write,
) -> Result<DescriptorIntake, ToolkitError> {
    let shape = match node {
        "literal" => DescriptorShape::LiteralXpub,
        "md1" => DescriptorShape::Md1,
        "bip388" => DescriptorShape::Bip388Json,
        other => {
            return Err(ToolkitError::BadInput(format!(
                "--descriptor-from <node>=<value>: unknown node `{other}`; expected one of `literal`, `md1`, `bip388`"
            )));
        }
    };
    // Stdin sentinel `-`: read all stdin. For md1 we split on newlines; for
    // literal/bip388 we use the full payload.
    let resolved: String = if value == "-" {
        let mut buf = String::new();
        stdin
            .read_to_string(&mut buf)
            .map_err(|e| ToolkitError::BadInput(format!("stdin read: {e}")))?;
        buf
    } else {
        value.to_string()
    };
    intake_from_shape(&resolved, shape, network, account, stderr)
}

/// Dispatch on detected shape.
fn intake_from_shape(
    payload: &str,
    shape: DescriptorShape,
    network: crate::network::CliNetwork,
    account: u32,
    stderr: &mut impl std::io::Write,
) -> Result<DescriptorIntake, ToolkitError> {
    match shape {
        DescriptorShape::Bip388Json => parse_bip388_json(payload, network, account, stderr),
        DescriptorShape::Md1 => parse_md1(payload, stderr),
        DescriptorShape::LiteralXpub => parse_literal_xpub(payload, network, account, stderr),
    }
}

/// Auto-detect descriptor shape per plan §4.2 tie-break order.
pub fn detect_shape(value: &str) -> Result<DescriptorShape, ToolkitError> {
    let trimmed = value.trim_start();
    // 1) BIP-388 JSON: starts with `{` after trim.
    if trimmed.starts_with('{') {
        return Ok(DescriptorShape::Bip388Json);
    }
    // 2) md1 HRP: all whitespace-separated tokens start with `md1`. Single
    //    token is the inline single-chunk case. Case-insensitive PROBE
    //    (v0.53.3 audit M11); originals pass to md-codec, the case authority.
    let tokens: Vec<&str> = trimmed.split_whitespace().collect();
    if !tokens.is_empty() && tokens.iter().all(|t| t.to_lowercase().starts_with("md1")) {
        return Ok(DescriptorShape::Md1);
    }
    // 3) toolkit `@N`-placeholder: contains `@<digit>+`. Refuse (synthetic
    //    xpubs are non-searchable).
    if contains_at_n_placeholder(trimmed) {
        return Err(ToolkitError::BadInput(
            "toolkit @N descriptors carry synthetic xpubs; supply a literal-xpub descriptor, md1 card, or BIP-388 wallet-policy JSON instead".into(),
        ));
    }
    // 4) Default: external literal-xpub.
    Ok(DescriptorShape::LiteralXpub)
}

/// Test `value` for an `@\d+` token. Lightweight: byte-walk seeking `@`
/// followed by ASCII digit. Mirrors `parse_descriptor.rs:60-127` lex rules
/// to a first approximation; this is a detection heuristic only.
fn contains_at_n_placeholder(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'@' && bytes[i + 1].is_ascii_digit() {
            return true;
        }
        i += 1;
    }
    false
}

/// Parse a BIP-388 wallet-policy JSON by delegating to the shared expander
/// `wallet_import::pipeline::expand_bip388_policy` (the single-sourced exact
/// inverse of the emitter), then parsing the reconstructed concrete descriptor
/// via the literal-xpub funnel.
fn parse_bip388_json(
    payload: &str,
    network: crate::network::CliNetwork,
    account: u32,
    stderr: &mut impl std::io::Write,
) -> Result<DescriptorIntake, ToolkitError> {
    let template = crate::wallet_import::pipeline::expand_bip388_policy(payload)?;
    let mut intake = parse_literal_xpub(&template, network, account, stderr)?;
    intake.shape = DescriptorShape::Bip388Json;
    Ok(intake)
}

/// Parse md1 card(s) into a `Vec<CosignerExtract>` via tree-walk of
/// `md_codec::Descriptor`. Mirrors `cmd/verify_bundle.rs:1934-1944` for
/// path resolution and `cmd/verify_bundle.rs:1971-1985` for xpub lookup.
///
/// Multi-chunk handling (plan §4.2 R2 I-R2-1 + R3 I-R3-2 lock — NO whitespace
/// or comma split anywhere): newline-separated payload comes from the
/// `--descriptor-from md1=-` stdin sentinel (each chunk on its own line,
/// mirrors `repair.rs:7` precedent). Inline `--descriptor <md1...>` is
/// single-chunk only; payloads with multiple tokens are refused.
fn parse_md1(
    payload: &str,
    stderr: &mut impl std::io::Write,
) -> Result<DescriptorIntake, ToolkitError> {
    // Split on newlines only (stdin one-chunk-per-line shape). Inline
    // single-chunk arrives as a single string with no newlines.
    let chunks: Vec<&str> = payload
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    if chunks.is_empty() {
        return Err(ToolkitError::BadInput("md1 payload is empty".into()));
    }
    let desc = md_codec::chunk::reassemble(&chunks).map_err(ToolkitError::MdCodec)?;
    let n = desc.n as usize;
    if n == 0 {
        return Err(ToolkitError::BadInput(
            "md1 descriptor declares zero cosigners (n=0)".into(),
        ));
    }
    // Non-blocking consensus-masked older() advisory (Adapter A, A-raw-card:
    // bit-31 REACHABLE — older_advisories_tree has NO debug_assert).
    // SPEC_older_timelock_advisory §4 / PLAN Task 10 3c.
    let adv = crate::timelock_advisory::older_advisories_tree(&desc);
    crate::timelock_advisory::emit_advisories(&adv, stderr);
    // Per-slot path resolver. Mirrors `verify_bundle::md_path_for`.
    let md_path_for = |idx: usize| -> Option<md_codec::origin_path::OriginPath> {
        if let Some(overrides) = &desc.tlv.origin_path_overrides {
            if let Some((_, op)) = overrides.iter().find(|(i, _)| *i as usize == idx) {
                return Some(op.clone());
            }
        }
        match &desc.path_decl.paths {
            md_codec::origin_path::PathDeclPaths::Shared(op) => Some(op.clone()),
            md_codec::origin_path::PathDeclPaths::Divergent(v) => v.get(idx).cloned(),
        }
    };
    // Per-slot fingerprint resolver. Mirrors `verify_bundle::md_fp_for`.
    let md_fp_for = |idx: usize| -> Option<[u8; 4]> {
        desc.tlv.fingerprints.as_ref().and_then(|v| {
            v.iter()
                .find(|(i, _)| *i as usize == idx)
                .map(|(_, fp)| *fp)
        })
    };
    // Zero-xpub guard (plan §4.3 step 3 + R2 m-R2-1 + R3 I-R3-4 lock).
    let pubkeys = desc.tlv.pubkeys.as_ref();
    if pubkeys.map_or(0, Vec::len) == 0 {
        return Err(ToolkitError::BadInput(
            "descriptor contains no extended keys; xpub-search requires xpub-shaped cosigners"
                .into(),
        ));
    }
    let mut cosigners: Vec<CosignerExtract> = Vec::with_capacity(n);
    for idx in 0..n {
        let xpub_65 = pubkeys
            .and_then(|v| v.iter().find(|(slot, _)| *slot as usize == idx))
            .map(|(_, b)| *b);
        let fp = md_fp_for(idx).map(Fingerprint::from);
        let path = match md_path_for(idx) {
            Some(op) => Some(crate::cmd::bundle::origin_to_derivation_path(&op)?),
            None => None,
        };
        cosigners.push(CosignerExtract {
            idx,
            xpub_65,
            fingerprint_anno: fp,
            derivation_path_anno: path,
            is_nums: false,
        });
    }
    Ok(DescriptorIntake {
        cosigners,
        shape: DescriptorShape::Md1,
        defaulted_indices: Vec::new(),
    })
}

/// Parse a literal-xpub descriptor via rust-miniscript. Walks `iter_pk()`
/// collecting each xpub-shaped key into a `CosignerExtract`.
fn parse_literal_xpub(
    descriptor_str: &str,
    network: crate::network::CliNetwork,
    account: u32,
    stderr: &mut impl std::io::Write,
) -> Result<DescriptorIntake, ToolkitError> {
    // SPEC bip388-double-star-shorthand-support §0/§5 — xpub-search is a
    // structurally separate parser (bypasses `parse_descriptor`'s lexer
    // entirely), so it needs its own expansion of a literal `/**` before
    // `miniscript::Descriptor::from_str`.
    let expanded = crate::parse_descriptor::expand_literal_double_star(descriptor_str);
    let parsed = MsDescriptor::<DescriptorPublicKey>::from_str(expanded.as_ref())
        .map_err(|e| ToolkitError::DescriptorParse(format!("--descriptor parse: {e}")))?;
    // H12 — taproot-aware default-origin script-type. `parsed` is a
    // rust-miniscript `Descriptor` (NOT an md-codec `Tag`), so detect taproot
    // directly (precedent: `wallet_import/bsms.rs`): Tr → 3' (P2TR), else 2'.
    // Pre-H12 the literal `2` put taproot cosigner keys in the wrong subtree.
    let default_script_type = if matches!(parsed, MsDescriptor::Tr(_)) {
        3
    } else {
        2
    };
    // Non-blocking consensus-masked older() advisory (Adapter B; post-from_str,
    // bit-31 unreachable). SPEC_older_timelock_advisory §4 / PLAN Task 10 3a.
    let adv = crate::timelock_advisory::older_advisories_descriptor(&parsed);
    crate::timelock_advisory::emit_advisories(&adv, stderr);
    // For a top-level `Tr` descriptor, the internal key may be the NUMS
    // sentinel (a `Single(SinglePubKey::XOnly(<NUMS>))`). We mark that
    // cosigner `is_nums = true`; other DescriptorPublicKey::Single entries
    // are not xpub-shaped and are NOT searched, but they're still positions
    // in the cosigner index sequence.
    let mut cosigners: Vec<CosignerExtract> = Vec::new();
    let mut xpub_count: usize = 0;
    let mut defaulted_indices: Vec<u8> = Vec::new();
    for (idx, pk) in parsed.iter_pk().enumerate() {
        match &pk {
            DescriptorPublicKey::XPub(x) => {
                let xpub_65 = crate::synthesize::xpub_to_65(&x.xkey);
                let (fp_anno, path_anno) = match &x.origin {
                    Some((fp, path)) => (Some(*fp), Some(path.clone())),
                    None => (None, None),
                };
                let mut effective_path = path_anno.clone();
                if effective_path.is_none() {
                    // v0.19.0 silent-default-path inference: assign BIP-48
                    // m/48'/coin'/account'/<script_type>' as the cosigner path
                    // (H12 — 3' for taproot). Mirrors `cmd/bundle.rs` notice +
                    // plan §4.3 step 4.
                    let default = bip48_default_path(network, account, default_script_type);
                    effective_path = Some(default);
                    defaulted_indices.push(idx as u8);
                }
                cosigners.push(CosignerExtract {
                    idx,
                    xpub_65: Some(xpub_65),
                    fingerprint_anno: fp_anno,
                    derivation_path_anno: effective_path,
                    is_nums: false,
                });
                xpub_count += 1;
            }
            DescriptorPublicKey::MultiXPub(x) => {
                let xpub_65 = crate::synthesize::xpub_to_65(&x.xkey);
                let (fp_anno, path_anno) = match &x.origin {
                    Some((fp, path)) => (Some(*fp), Some(path.clone())),
                    None => (None, None),
                };
                let mut effective_path = path_anno.clone();
                if effective_path.is_none() {
                    // H12 — taproot-aware default (3' for taproot, else 2').
                    let default = bip48_default_path(network, account, default_script_type);
                    effective_path = Some(default);
                    defaulted_indices.push(idx as u8);
                }
                cosigners.push(CosignerExtract {
                    idx,
                    xpub_65: Some(xpub_65),
                    fingerprint_anno: fp_anno,
                    derivation_path_anno: effective_path,
                    is_nums: false,
                });
                xpub_count += 1;
            }
            DescriptorPublicKey::Single(single) => {
                // Non-xpub key. Common case: taproot NUMS internal key.
                // Detect via x-only pubkey hex equal to NUMS_H_POINT_X_ONLY_HEX.
                let is_nums = match &single.key {
                    miniscript::descriptor::SinglePubKey::XOnly(xonly) => {
                        let serialized = xonly.serialize();
                        let hex_str = hex_encode_lower(&serialized);
                        hex_str == NUMS_H_POINT_X_ONLY_HEX
                    }
                    _ => false,
                };
                cosigners.push(CosignerExtract {
                    idx,
                    xpub_65: None,
                    fingerprint_anno: None,
                    derivation_path_anno: None,
                    is_nums,
                });
            }
        }
    }
    // Zero-xpub guard (R2 m-R2-1 + R3 I-R3-4 lock).
    if xpub_count == 0 {
        return Err(ToolkitError::BadInput(
            "descriptor contains no extended keys; xpub-search requires xpub-shaped cosigners"
                .into(),
        ));
    }
    // Emit the v0.19.0 default-path notice (mirrors `cmd/bundle.rs:1367-1388`
    // ~6 LOC inline per plan §4.3 step 4 + R0 I7 lock).
    if !defaulted_indices.is_empty() {
        let idx_list = defaulted_indices
            .iter()
            .map(|i| format!("@{i}"))
            .collect::<Vec<_>>()
            .join(",");
        let coin = network.coin_type();
        // H12 — render the ACTUAL inferred script-type leaf (3' for taproot).
        writeln!(
            stderr,
            "info: non-canonical descriptor; defaulting origin path for {idx_list} to m/48'/{coin}'/{account}'/{default_script_type}' (BIP-48 cosigner path). Override per-placeholder with [fp/path] in the descriptor."
        )
        .map_err(|e| ToolkitError::BadInput(format!("stderr write: {e}")))?;
    }
    Ok(DescriptorIntake {
        cosigners,
        shape: DescriptorShape::LiteralXpub,
        defaulted_indices,
    })
}

/// Build a BIP-48 path `m/48'/coin'/account'/script_type'` for default-path
/// inference (mirrors `parse::MultisigPathFamily::default_origin_path`).
fn bip48_default_path(
    network: crate::network::CliNetwork,
    account: u32,
    script_type: u32,
) -> DerivationPath {
    use crate::parse::MultisigPathFamily;
    let s = MultisigPathFamily::Bip48.default_origin_path(network, account, script_type);
    DerivationPath::from_str(&s).expect("BIP-48 default path well-formed")
}

/// Lowercase-hex encode. Tiny helper; avoids the bitcoin::hex direct import.
fn hex_encode_lower(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        out.push_str(&format!("{b:02x}"));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_shape_json_object_returns_bip388() {
        assert_eq!(
            detect_shape("  {\"name\":\"x\"}").unwrap(),
            DescriptorShape::Bip388Json
        );
    }

    #[test]
    fn detect_shape_md1_token_returns_md1() {
        assert_eq!(detect_shape("md1abcde").unwrap(), DescriptorShape::Md1);
    }

    #[test]
    fn detect_shape_at_n_placeholder_refused() {
        let err = detect_shape("wpkh(@0/<0;1>/*)").unwrap_err();
        let msg = format!("{err:?}");
        assert!(msg.contains("toolkit @N"));
    }

    #[test]
    fn detect_shape_default_literal() {
        let xpub = "xpub6Cuvy7w8aDxakdjsxFq8M2NbXdZHghkpAcKvqzh4WUR8FBxXVKkjjsedX9yzeYZPjVx3vrwJxYqLmnfvSdyXxztnUMpsiE7Q1wPwhP3DmFy";
        assert_eq!(
            detect_shape(&format!("wpkh({xpub}/<0;1>/*)")).unwrap(),
            DescriptorShape::LiteralXpub
        );
    }

    #[test]
    fn contains_at_n_walker() {
        assert!(contains_at_n_placeholder("wpkh(@0)"));
        assert!(contains_at_n_placeholder(
            "wsh(sortedmulti(2,@0[fp/x'],@10[fp/y']))"
        ));
        assert!(!contains_at_n_placeholder("wpkh(xpub6Cuvy7w8aDxakdjsxFq8M2NbXdZHghkpAcKvqzh4WUR8FBxXVKkjjsedX9yzeYZPjVx3vrwJxYqLmnfvSdyXxztnUMpsiE7Q1wPwhP3DmFy)"));
        // Edge: `@a` should NOT trigger.
        assert!(!contains_at_n_placeholder("foo@a"));
    }

    // ── H12 (cycle-1): xpub-search literal-xpub default-origin is taproot-aware ──
    // `parse_literal_xpub` operates on a rust-miniscript `Descriptor` (not an
    // md-codec `Tag`), so taproot detection is `matches!(parsed, Tr(_))` →
    // script-type 3', else 2'. Pre-H12 the literal `2` hardcode put taproot
    // cosigner keys in the wrong (P2WSH) subtree on the xpub-search intake path.
    const H12_XPUB_A: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
    const H12_XPUB_B: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";

    fn last_component_hardened(p: &DerivationPath) -> Option<(u32, bool)> {
        use bitcoin::bip32::ChildNumber;
        p.into_iter().last().map(|c| match c {
            ChildNumber::Hardened { index } => (*index, true),
            ChildNumber::Normal { index } => (*index, false),
        })
    }

    #[test]
    fn literal_xpub_taproot_defaults_origin_to_3prime() {
        let mut sink = Vec::new();
        // Origin-elided taproot multisig → default-inference fires; must land
        // in the 3' (P2TR) subtree.
        let desc = format!(
            "tr({NUMS_H_POINT_X_ONLY_HEX},multi_a(2,{H12_XPUB_A}/<0;1>/*,{H12_XPUB_B}/<0;1>/*))"
        );
        let intake = parse_literal_xpub(&desc, crate::network::CliNetwork::Mainnet, 0, &mut sink)
            .expect("origin-elided taproot multisig must parse");
        let xpub_cosigners: Vec<&CosignerExtract> = intake
            .cosigners
            .iter()
            .filter(|c| c.xpub_65.is_some())
            .collect();
        assert_eq!(xpub_cosigners.len(), 2, "two xpub cosigners");
        for c in xpub_cosigners {
            let p = c
                .derivation_path_anno
                .as_ref()
                .expect("defaulted cosigner must carry an inferred path");
            assert_eq!(
                last_component_hardened(p),
                Some((3, true)),
                "taproot cosigner default origin must end in 3' (P2TR); got {p}"
            );
        }
    }

    #[test]
    fn literal_xpub_wsh_defaults_origin_to_2prime() {
        let mut sink = Vec::new();
        // Clean-negative: origin-elided wsh multisig still defaults to 2'.
        let desc = format!("wsh(sortedmulti(2,{H12_XPUB_A}/<0;1>/*,{H12_XPUB_B}/<0;1>/*))");
        let intake = parse_literal_xpub(&desc, crate::network::CliNetwork::Mainnet, 0, &mut sink)
            .expect("origin-elided wsh multisig must parse");
        for c in intake.cosigners.iter().filter(|c| c.xpub_65.is_some()) {
            let p = c.derivation_path_anno.as_ref().expect("inferred path");
            assert_eq!(
                last_component_hardened(p),
                Some((2, true)),
                "wsh cosigner default origin must end in 2' (P2WSH); got {p}"
            );
        }
    }
}
