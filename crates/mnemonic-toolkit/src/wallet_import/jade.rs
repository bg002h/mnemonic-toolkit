//! v0.28.0 Phase P5 — Blockstream Jade wallet-import parser.
//!
//! Per `design/SPEC_wallet_import_v0_28_0.md` §11.5. Jade's
//! `get_registered_multisig` RPC reply carries a top-level `multisig_file`
//! field whose value is the same flat-file text shape Coldcard's multisig
//! export produces (per Blockstream/Jade docs:
//! <https://github.com/Blockstream/Jade/blob/master/docs/index.rst>):
//!
//! ```json
//! {
//!   "id": "<request-id>",
//!   "multisig_name": "<wallet-name>",
//!   "multisig_file": "Name: …\nPolicy: …\nFormat: …\nDerivation: …\n\n<xfp>: <xpub>\n…"
//! }
//! ```
//!
//! The distinctive sniff marker is the top-level `multisig_file` field
//! (no other v0.28.0 ingest format uses it).
//!
//! ## Q1 lock (SeedQR deferred)
//!
//! v0.28.0 jade.rs handles ONLY the JSON shape with top-level `multisig_file`
//! field. SeedQR variant (`register_multisig` RPC + `seedqr` reply field) is
//! DEFERRED to a future cycle (FOLLOWUP `wallet-import-jade-seedqr` filed at
//! Phase P14A).
//!
//! ## Parse strategy (P5B)
//!
//! Extract the `multisig_file` field → delegate to
//! `super::coldcard_multisig::parse_text(&inner_text)` (SPEC §11.4 parser).
//! The Jade wrapper carries no extra parse semantics beyond the multisig
//! text; the delegation preserves the SPEC §11.4.1 5-row XFP truth-table
//! semantics byte-identical. Provenance is annotated as Jade (not Coldcard)
//! so downstream consumers can distinguish the source.
//!
//! ## Provenance
//!
//! `ImportProvenance::Jade(JadeSourceMetadata)`. The struct WRAPS the
//! delegated `ColdcardMultisigSourceMetadata` (cross-module `pub(crate)`
//! reference per plan-doc §S.5) plus a future-proof
//! `jade_specific_fields: Vec<String>` (currently empty; reserved for
//! SeedQR variant when the FOLLOWUP lands).

use super::{
    coldcard_multisig::{parse_text as parse_coldcard_multisig_text, ColdcardMultisigSourceMetadata},
    ImportProvenance, ParsedImport, WalletFormatParser,
};
use crate::error::ToolkitError;
use serde_json::Value;
use std::io::Write;

/// SPEC §11.5 — Blockstream Jade wallet-import parser.
pub(crate) struct JadeParser;

/// SPEC §11.5 — per-blob provenance metadata for a Jade parse.
///
/// Wraps the delegated `ColdcardMultisigSourceMetadata` (per plan-doc §S.5
/// — `multisig_file` body is byte-identical to Coldcard's multisig text
/// export) plus a future-proof `jade_specific_fields` vec (currently empty;
/// the SeedQR variant deferred by Q1 lock would populate it).
///
/// `#[allow(dead_code)]` covers the P5A → P5C interim: P5A publishes the
/// type, P5B constructs it from real parse output, and P5C plumbs per-field
/// consumption into the `--json` envelope (`jade_source_metadata` field).
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct JadeSourceMetadata {
    /// Delegated Coldcard-multisig metadata: extracted from the inner
    /// `multisig_file` text via `coldcard_multisig::parse_text`. Carries
    /// SPEC §11.4.1 truth-table telemetry (`xfp_was_blob_supplied`,
    /// `xfp_header_disagreed`) verbatim.
    pub(crate) coldcard_compat: ColdcardMultisigSourceMetadata,
    /// Future-proof: Jade-specific field names that were present in the
    /// JSON wrapper but not consumed into typed metadata. Currently empty
    /// (`multisig_name` / `id` are preserved-but-unused JSON envelope
    /// fields). Reserved for the SeedQR variant (Q1 lock; new FOLLOWUP
    /// `wallet-import-jade-seedqr` at Phase P14A).
    pub(crate) jade_specific_fields: Vec<String>,
}

impl WalletFormatParser for JadeParser {
    fn sniff(blob: &[u8]) -> bool {
        // SPEC §11.5 sniff: must be valid JSON with a top-level object
        // carrying a `multisig_file` field whose value is a non-empty
        // string. The `multisig_file` field is the load-bearing
        // distinctive marker — no other v0.28.0 format uses it.
        let value: Value = match serde_json::from_slice(blob) {
            Ok(v) => v,
            Err(_) => return false,
        };
        let obj = match value.as_object() {
            Some(o) => o,
            None => return false,
        };
        matches!(
            obj.get("multisig_file").and_then(|v| v.as_str()),
            Some(s) if !s.is_empty()
        )
    }

    fn parse(blob: &[u8], stderr: &mut dyn Write) -> Result<Vec<ParsedImport>, ToolkitError> {
        // SPEC §11.5 parse: extract `multisig_file` string; delegate to
        // `coldcard_multisig::parse_text` (SPEC §11.4 parser); re-annotate
        // the returned `ParsedImport.provenance` from `ColdcardMultisig`
        // to `Jade`. The delegation preserves SPEC §11.4.1 5-row XFP
        // truth-table semantics byte-identical — the Jade wrapper carries
        // no extra parse semantics beyond the multisig text.
        let value: Value = serde_json::from_slice(blob).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: jade: parse error: top-level blob is not valid JSON: {e}"
            ))
        })?;
        let obj = value.as_object().ok_or_else(|| {
            ToolkitError::ImportWalletParse(
                "import-wallet: jade: parse error: top-level JSON value is not an object"
                    .to_string(),
            )
        })?;
        let multisig_file = obj
            .get("multisig_file")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(
                    "import-wallet: jade: parse error: missing or non-string top-level \
                     `multisig_file` field"
                        .to_string(),
                )
            })?;

        // Delegate to Coldcard multisig text parser (SPEC §11.4).
        let mut parsed = parse_coldcard_multisig_text(multisig_file.as_bytes(), stderr)?;

        // Re-annotate provenance: ColdcardMultisig → Jade wrapper.
        let coldcard_compat = match parsed.provenance {
            ImportProvenance::ColdcardMultisig(meta) => meta,
            _ => {
                // Defensive: parse_coldcard_multisig_text only ever returns
                // ColdcardMultisig provenance. If this ever changes the
                // explicit error here is preferable to a silent mismatch.
                return Err(ToolkitError::ImportWalletParse(
                    "import-wallet: jade: internal: delegated parser returned \
                     non-ColdcardMultisig provenance"
                        .to_string(),
                ));
            }
        };
        parsed.provenance = ImportProvenance::Jade(JadeSourceMetadata {
            coldcard_compat,
            jade_specific_fields: Vec::new(),
        });

        Ok(vec![parsed])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Inner Coldcard-multisig text reused from the existing
    /// `coldcard-ms-2of3-p2wsh-with-xfp.txt` fixture — byte-identical text
    /// embedded inside Jade's `multisig_file` JSON field.
    const COLDCARD_INNER_2OF3: &str = "\
Name: TestMs2of3
Policy: 2 of 3
Derivation: m/48'/0'/0'/2'
Format: P2WSH
XFP: 34A3A4F1

34A3A4F1: xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX
FF9DFBCF: xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6
B7F7DFEA: xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx
";

    fn jade_wrapper_2of3() -> String {
        // Real-shape Jade `get_registered_multisig` reply.
        serde_json::to_string_pretty(&serde_json::json!({
            "id": "test-request-id-1",
            "multisig_name": "TestMs2of3",
            "multisig_file": COLDCARD_INNER_2OF3,
        }))
        .unwrap()
    }

    // ========================================================================
    // P5A sniff cells
    // ========================================================================

    #[test]
    fn sniff_jade_wrapper_2of3_positive() {
        let blob = jade_wrapper_2of3();
        assert!(JadeParser::sniff(blob.as_bytes()));
    }

    #[test]
    fn sniff_rejects_bare_coldcard_multisig_text() {
        // Bare Coldcard-multisig text (not wrapped in JSON) MUST NOT
        // sniff-positive as Jade — it sniffs-positive as
        // ColdcardMultisig instead. Disambiguation rule.
        let blob = COLDCARD_INNER_2OF3.as_bytes();
        assert!(!JadeParser::sniff(blob));
    }

    #[test]
    fn sniff_rejects_missing_multisig_file_field() {
        // Top-level JSON object WITHOUT the load-bearing `multisig_file`
        // field → sniff-negative.
        let blob = br#"{"id":"x","multisig_name":"y"}"#;
        assert!(!JadeParser::sniff(blob));
    }

    #[test]
    fn sniff_rejects_empty_multisig_file_string() {
        // Empty string in `multisig_file` → sniff-negative (defensive: an
        // empty wrapper carries no useful payload).
        let blob = br#"{"multisig_file":""}"#;
        assert!(!JadeParser::sniff(blob));
    }

    #[test]
    fn sniff_rejects_non_string_multisig_file_value() {
        // `multisig_file` present but non-string (e.g., object / array) →
        // sniff-negative.
        let blob = br#"{"multisig_file":{}}"#;
        assert!(!JadeParser::sniff(blob));
        let blob = br#"{"multisig_file":[]}"#;
        assert!(!JadeParser::sniff(blob));
        let blob = br#"{"multisig_file":null}"#;
        assert!(!JadeParser::sniff(blob));
    }

    #[test]
    fn sniff_rejects_top_level_array() {
        // Top-level JSON array → sniff-negative (must be an object).
        let blob = br#"[{"multisig_file":"x"}]"#;
        assert!(!JadeParser::sniff(blob));
    }

    #[test]
    fn sniff_rejects_malformed_json() {
        let blob = b"{not-valid-json";
        assert!(!JadeParser::sniff(blob));
    }

    #[test]
    fn sniff_rejects_empty_blob() {
        assert!(!JadeParser::sniff(b""));
    }

    #[test]
    fn sniff_rejects_bsms_blob() {
        // BSMS 2-line shape (text, not JSON) → sniff-negative.
        let blob = b"BSMS 1.0\nwpkh([deadbeef/84'/0'/0']xpub6...)\n";
        assert!(!JadeParser::sniff(blob));
    }

    // ========================================================================
    // P5B parse cells
    // ========================================================================

    #[test]
    fn parse_jade_wrapper_2of3_happy_path() {
        let blob = jade_wrapper_2of3();
        let mut stderr: Vec<u8> = Vec::new();
        let parsed = JadeParser::parse(blob.as_bytes(), &mut stderr).expect("parse must succeed");
        assert_eq!(parsed.len(), 1, "Jade emits exactly one ParsedImport");
        let p = &parsed[0];
        assert_eq!(p.cosigners.len(), 3);
        assert_eq!(p.threshold, Some(2));
        // Provenance: Jade (NOT ColdcardMultisig — the re-annotation
        // is the load-bearing distinction per SPEC §11.5).
        assert!(
            matches!(p.provenance, ImportProvenance::Jade(_)),
            "provenance must be Jade after delegation"
        );
        // The delegated ColdcardMultisig metadata is preserved inside.
        if let ImportProvenance::Jade(meta) = &p.provenance {
            assert_eq!(meta.coldcard_compat.name, "TestMs2of3");
            assert_eq!(meta.coldcard_compat.policy.k, 2);
            assert_eq!(meta.coldcard_compat.policy.n, 3);
            // jade_specific_fields is empty at v0.28.0 (SeedQR deferred per
            // Q1 lock).
            assert!(meta.jade_specific_fields.is_empty());
        }
    }

    #[test]
    fn parse_rejects_missing_multisig_file_field() {
        let blob = br#"{"id":"x","multisig_name":"y"}"#;
        let mut stderr: Vec<u8> = Vec::new();
        let err = JadeParser::parse(blob, &mut stderr).expect_err("parse must fail");
        let err_str = err.to_string();
        assert!(
            err_str.contains("jade") && err_str.contains("multisig_file"),
            "error must cite jade format + missing field; got: {err_str}"
        );
    }

    #[test]
    fn parse_rejects_malformed_json() {
        let blob = b"{not-valid-json";
        let mut stderr: Vec<u8> = Vec::new();
        let err = JadeParser::parse(blob, &mut stderr).expect_err("parse must fail");
        let err_str = err.to_string();
        assert!(
            err_str.contains("jade") && err_str.contains("not valid JSON"),
            "error must cite jade format + JSON validity; got: {err_str}"
        );
    }

    #[test]
    fn parse_rejects_inner_text_missing_format_header() {
        // Inner text lacks the `Format:` header → delegated parser surfaces
        // a ColdcardMultisig error (we do not re-wrap or relabel; the user
        // sees the underlying SPEC §11.4 diagnostic).
        let inner = "Name: x\nPolicy: 2 of 3\nDerivation: m/48'/0'/0'/2'\n\nABCDEF01: xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX\n";
        let blob = serde_json::to_string(&serde_json::json!({
            "multisig_file": inner,
        }))
        .unwrap();
        let mut stderr: Vec<u8> = Vec::new();
        let err = JadeParser::parse(blob.as_bytes(), &mut stderr).expect_err("parse must fail");
        // Delegated error surfaces — the format string is `coldcard-multisig`
        // since we delegate to that parser; this is by design (the user
        // sees the underlying §11.4 diagnostic verbatim).
        let err_str = err.to_string();
        assert!(
            err_str.contains("coldcard-multisig") || err_str.contains("Format"),
            "delegated coldcard-multisig error must surface; got: {err_str}"
        );
    }

    // ========================================================================
    // P5B fixture-backed parse cells
    // ========================================================================

    fn read_fixture(name: &str) -> Vec<u8> {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("wallet_import")
            .join(name);
        std::fs::read(&path).unwrap_or_else(|e| panic!("read fixture {path:?}: {e}"))
    }

    #[test]
    fn fixture_jade_multisig_2of3_p2wsh_parse_ok() {
        let blob = read_fixture("jade-multisig-2of3-p2wsh.json");
        // Sniff must positive on the wire-shape blob.
        assert!(JadeParser::sniff(&blob), "fixture must sniff-positive");
        let mut stderr: Vec<u8> = Vec::new();
        let parsed = JadeParser::parse(&blob, &mut stderr).expect("fixture parse must succeed");
        assert_eq!(parsed.len(), 1);
        let p = &parsed[0];
        assert_eq!(p.cosigners.len(), 3);
        assert_eq!(p.threshold, Some(2));
        assert!(matches!(p.provenance, ImportProvenance::Jade(_)));
    }

    #[test]
    fn fixture_jade_singlesig_refused() {
        let blob = read_fixture("jade-singlesig-refused.json");
        // Sniff is still positive (top-level `multisig_file` field exists);
        // the refusal surfaces at parse time via the delegated
        // coldcard-multisig parser (missing `Policy:` header).
        assert!(JadeParser::sniff(&blob), "fixture must sniff-positive");
        let mut stderr: Vec<u8> = Vec::new();
        let err = JadeParser::parse(&blob, &mut stderr)
            .expect_err("singlesig-shaped fixture must refuse");
        let err_str = err.to_string();
        assert!(
            err_str.contains("coldcard-multisig") || err_str.contains("Policy") || err_str.contains("Format"),
            "delegated parser error must surface; got: {err_str}"
        );
    }

    #[test]
    fn fixture_jade_malformed_json_refused() {
        let blob = read_fixture("jade-malformed-json.json");
        // Sniff returns false (not valid JSON).
        assert!(!JadeParser::sniff(&blob), "malformed JSON must sniff-negative");
        let mut stderr: Vec<u8> = Vec::new();
        let err = JadeParser::parse(&blob, &mut stderr)
            .expect_err("malformed-JSON fixture must refuse");
        let err_str = err.to_string();
        assert!(
            err_str.contains("jade") && err_str.contains("not valid JSON"),
            "jade error must cite format + JSON validity; got: {err_str}"
        );
    }
}
