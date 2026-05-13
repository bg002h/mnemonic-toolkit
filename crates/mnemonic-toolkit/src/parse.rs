//! Input source resolution: argv vs stdin, phrase normalization,
//! fingerprint parsing.
//!
//! Realizes SPEC §3.2 (stdin uniform), §2.1.5 (--master-fingerprint
//! 8-hex case-insensitive), §2.1.6 (concurrent stdin guard).

use crate::error::{BitcoinErrorKind, ToolkitError};
use crate::network::CliNetwork;
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};
use clap::ValueEnum;
use std::io::Read;
use std::path::Path;
use std::str::FromStr;

#[allow(dead_code)]
/// Resolve a flag value: `Some(s)` literal, `Some("-")` stdin, `None` error.
/// Whitespace is collapsed via `normalize_phrase`.
pub fn read_phrase_input(arg: Option<&str>, stdin: &mut dyn Read) -> Result<String, ToolkitError> {
    match arg {
        Some("-") => {
            let mut buf = String::new();
            stdin
                .read_to_string(&mut buf)
                .map_err(|e| ToolkitError::BadInput(format!("stdin read failed: {}", e)))?;
            Ok(normalize_phrase(&buf))
        }
        Some(s) => Ok(normalize_phrase(s)),
        None => Err(ToolkitError::BadInput("missing argument".into())),
    }
}

#[allow(dead_code)]
/// Collapse runs of whitespace to single spaces; preserve word boundaries.
fn normalize_phrase(s: &str) -> String {
    s.split_whitespace().collect::<Vec<&str>>().join(" ")
}

/// Parse `--master-fingerprint`: 8 hex chars, case-insensitive, no `0x` prefix.
/// SPEC §2.1.5 byte-exact rejection message.
pub fn parse_master_fingerprint(s: &str) -> Result<Fingerprint, ToolkitError> {
    if s.len() != 8 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ToolkitError::BadInput(
            "--master-fingerprint must be 8 hex chars (e.g., deadbeef)".into(),
        ));
    }
    Fingerprint::from_str(s)
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::FingerprintParse(format!("{}", e))))
}

/// Per-cosigner spec for v0.2 multisig watch-only mode (SPEC §2.1.2).
/// Source: `--cosigner=<xpub>:<fp>:<path>` flag-repetition or per-entry
/// records in `--cosigners-file`.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CosignerSpec {
    pub xpub: Xpub,
    pub master_fingerprint: Fingerprint,
    /// `None` ⇒ use `--multisig-path-family` default for the cosigner.
    pub path: Option<DerivationPath>,
}

/// Path family for v0.2 multisig (SPEC §2.1.7). Default: `Bip87`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
#[allow(dead_code)]
pub enum MultisigPathFamily {
    Bip48,
    #[default]
    Bip87,
}

#[allow(dead_code)]
impl MultisigPathFamily {
    pub fn human_name(&self) -> &'static str {
        match self {
            MultisigPathFamily::Bip48 => "bip48",
            MultisigPathFamily::Bip87 => "bip87",
        }
    }

    /// Default origin path for this (family, network, account, script_type)
    /// per SPEC §4.2 (BIP-87: `m/87'/<coin>'/<account>'`; BIP-48:
    /// `m/48'/<coin>'/<account>'/<script_type>'`).
    ///
    /// `script_type` is the BIP-48 script-type component (1' = sh-wsh,
    /// 2' = wsh, 3' = tr-multi-a). Ignored for BIP-87.
    pub fn default_origin_path(
        &self,
        network: CliNetwork,
        account: u32,
        script_type: u32,
    ) -> String {
        match self {
            MultisigPathFamily::Bip87 => {
                format!("m/87'/{}'/{}'", network.coin_type(), account)
            }
            MultisigPathFamily::Bip48 => format!(
                "m/48'/{}'/{}'/{}'",
                network.coin_type(),
                account,
                script_type,
            ),
        }
    }
}

/// Parse a `--cosigner=<xpub>:<fp>:<path>` or `<xpub>:<fp>` argument value
/// (SPEC §2.1.2). The `cosigner_idx` is set by the caller (the bundle/
/// verify_bundle dispatcher iterates `--cosigner` flags with index).
///
/// Errors out with `ToolkitError::CosignerSpec { cosigner_idx, message }`
/// (exit 1) on malformed input.
#[allow(dead_code)]
pub fn parse_cosigner_spec(s: &str, cosigner_idx: usize) -> Result<CosignerSpec, ToolkitError> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() < 2 || parts.len() > 3 {
        return Err(ToolkitError::CosignerSpec {
            cosigner_idx,
            message: "expected <xpub>:<fp> or <xpub>:<fp>:<path>".into(),
        });
    }
    let xpub_str = parts[0];
    let fp_str = parts[1];
    let path_str = parts.get(2).copied();

    if fp_str.is_empty() {
        return Err(ToolkitError::CosignerSpec {
            cosigner_idx,
            message: "fingerprint required".into(),
        });
    }
    let xpub = Xpub::from_str(xpub_str).map_err(|e| ToolkitError::CosignerSpec {
        cosigner_idx,
        message: format!("xpub parse: {}", e),
    })?;
    let master_fingerprint =
        parse_master_fingerprint(fp_str).map_err(|e| ToolkitError::CosignerSpec {
            cosigner_idx,
            message: format!("fingerprint parse: {}", e.message()),
        })?;
    let path = if let Some(p) = path_str {
        if p.is_empty() {
            None
        } else {
            Some(
                DerivationPath::from_str(p).map_err(|e| ToolkitError::CosignerSpec {
                    cosigner_idx,
                    message: format!("path parse: {}", e),
                })?,
            )
        }
    } else {
        None
    };
    Ok(CosignerSpec {
        xpub,
        master_fingerprint,
        path,
    })
}

/// Per-entry shape of `--cosigners-file` JSON (SPEC §2.1.2.1). Mirrors
/// `CosignerSpec` but with optional fields (deserialized then validated).
#[derive(Debug, serde::Deserialize)]
#[allow(dead_code)]
struct CosignersFileEntry {
    xpub: Option<String>,
    master_fingerprint: Option<String>,
    path: Option<String>,
}

/// Parse a JSON `--cosigners-file <path>` into a list of `CosignerSpec`s
/// (SPEC §2.1.2.1). Errors with `ToolkitError::CosignersFile { message }`
/// (exit 1) for I/O, JSON parse, or schema-violation failures.
#[allow(dead_code)]
pub fn parse_cosigners_file(path: &Path) -> Result<Vec<CosignerSpec>, ToolkitError> {
    let text = std::fs::read_to_string(path).map_err(|e| ToolkitError::CosignersFile {
        message: format!("read {}: {}", path.display(), e),
    })?;
    let entries: Vec<CosignersFileEntry> =
        serde_json::from_str(&text).map_err(|e| ToolkitError::CosignersFile {
            message: format!("JSON parse: {}", e),
        })?;
    let mut out = Vec::with_capacity(entries.len());
    for (idx, entry) in entries.into_iter().enumerate() {
        let xpub_str = entry.xpub.ok_or_else(|| ToolkitError::CosignersFile {
            message: format!("cosigner index {}: xpub required", idx),
        })?;
        let fp_str = entry
            .master_fingerprint
            .ok_or_else(|| ToolkitError::CosignersFile {
                message: format!("cosigner index {}: master_fingerprint required", idx),
            })?;
        if fp_str.is_empty() {
            return Err(ToolkitError::CosignersFile {
                message: format!("cosigner index {}: master_fingerprint required", idx),
            });
        }
        let xpub = Xpub::from_str(&xpub_str).map_err(|e| ToolkitError::CosignersFile {
            message: format!("cosigner index {}: xpub parse: {}", idx, e),
        })?;
        let master_fingerprint =
            parse_master_fingerprint(&fp_str).map_err(|e| ToolkitError::CosignersFile {
                message: format!("cosigner index {}: fingerprint parse: {}", idx, e.message()),
            })?;
        let path = match entry.path {
            None => None,
            Some(p) if p.is_empty() => None,
            Some(p) => {
                Some(
                    DerivationPath::from_str(&p).map_err(|e| ToolkitError::CosignersFile {
                        message: format!("cosigner index {}: path parse: {}", idx, e),
                    })?,
                )
            }
        };
        out.push(CosignerSpec {
            xpub,
            master_fingerprint,
            path,
        });
    }
    Ok(out)
}

#[allow(dead_code)]
/// Reject concurrent stdin reads across phrase + passphrase.
pub fn check_no_concurrent_stdin(
    phrase: Option<&str>,
    passphrase: Option<&str>,
) -> Result<(), ToolkitError> {
    if phrase == Some("-") && passphrase == Some("-") {
        return Err(ToolkitError::BadInput(
            "only one of --phrase and --passphrase may read from stdin".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_phrase_collapses_whitespace() {
        assert_eq!(
            normalize_phrase("  word1   word2\nword3\t word4  "),
            "word1 word2 word3 word4"
        );
    }

    #[test]
    fn fp_lowercase_8hex_ok() {
        let fp = parse_master_fingerprint("deadbeef").unwrap();
        assert_eq!(fp.to_string().to_lowercase(), "deadbeef");
    }

    #[test]
    fn fp_uppercase_8hex_ok() {
        parse_master_fingerprint("DEADBEEF").unwrap();
    }

    #[test]
    fn fp_mixed_case_8hex_ok() {
        parse_master_fingerprint("DeAdBeEf").unwrap();
    }

    #[test]
    fn fp_short_rejected() {
        let e = parse_master_fingerprint("dead").unwrap_err();
        match e {
            ToolkitError::BadInput(m) => assert_eq!(
                m,
                "--master-fingerprint must be 8 hex chars (e.g., deadbeef)"
            ),
            _ => panic!("expected BadInput, got {:?}", e),
        }
    }

    #[test]
    fn fp_with_0x_prefix_rejected() {
        let e = parse_master_fingerprint("0xdeadbe").unwrap_err();
        assert!(matches!(e, ToolkitError::BadInput(_)));
    }

    #[test]
    fn fp_non_hex_char_rejected() {
        let e = parse_master_fingerprint("deadbeeg").unwrap_err();
        assert!(matches!(e, ToolkitError::BadInput(_)));
    }

    #[test]
    fn read_phrase_argv_normalizes() {
        let mut stdin = std::io::empty();
        let s = read_phrase_input(Some("  word1   word2  "), &mut stdin).unwrap();
        assert_eq!(s, "word1 word2");
    }

    #[test]
    fn read_phrase_stdin_normalizes() {
        let mut stdin = std::io::Cursor::new("  word1\n  word2\t\nword3\n  ");
        let s = read_phrase_input(Some("-"), &mut stdin).unwrap();
        assert_eq!(s, "word1 word2 word3");
    }

    #[test]
    fn concurrent_stdin_rejected() {
        let e = check_no_concurrent_stdin(Some("-"), Some("-")).unwrap_err();
        match e {
            ToolkitError::BadInput(m) => assert_eq!(
                m,
                "only one of --phrase and --passphrase may read from stdin"
            ),
            _ => panic!("expected BadInput"),
        }
    }

    #[test]
    fn one_stdin_ok() {
        check_no_concurrent_stdin(Some("-"), None).unwrap();
        check_no_concurrent_stdin(None, Some("-")).unwrap();
        check_no_concurrent_stdin(Some("words"), Some("-")).unwrap();
    }

    /// Real-world xpub for cosigner-spec parse tests. Mainnet, depth 3.
    /// (Comes from BIP-32 test vector 1; safe public test data.)
    const TEST_XPUB: &str = "xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj";

    #[test]
    fn cosigner_spec_xpub_fp_only_ok() {
        let spec = parse_cosigner_spec(&format!("{}:deadbeef", TEST_XPUB), 0).unwrap();
        assert_eq!(spec.path, None);
        assert_eq!(
            spec.master_fingerprint.to_string().to_lowercase(),
            "deadbeef"
        );
    }

    #[test]
    fn cosigner_spec_xpub_fp_path_ok() {
        let spec =
            parse_cosigner_spec(&format!("{}:deadbeef:m/48'/0'/0'/2'", TEST_XPUB), 1).unwrap();
        assert!(spec.path.is_some());
    }

    #[test]
    fn cosigner_spec_empty_fp_rejected() {
        let e = parse_cosigner_spec(&format!("{}::", TEST_XPUB), 0).unwrap_err();
        match e {
            ToolkitError::CosignerSpec {
                cosigner_idx,
                message,
            } => {
                assert_eq!(cosigner_idx, 0);
                assert!(message.contains("fingerprint required"));
            }
            other => panic!("expected CosignerSpec, got {:?}", other),
        }
    }

    #[test]
    fn cosigner_spec_malformed_xpub_rejected() {
        let e = parse_cosigner_spec("not-an-xpub:deadbeef", 2).unwrap_err();
        match e {
            ToolkitError::CosignerSpec {
                cosigner_idx,
                message,
            } => {
                assert_eq!(cosigner_idx, 2);
                assert!(message.contains("xpub parse"));
            }
            other => panic!("expected CosignerSpec, got {:?}", other),
        }
    }

    #[test]
    fn cosigner_spec_too_few_parts_rejected() {
        let e = parse_cosigner_spec("xpubonly", 0).unwrap_err();
        assert!(matches!(e, ToolkitError::CosignerSpec { .. }));
    }

    #[test]
    fn cosigners_file_round_trip_two_entries() {
        let json = format!(
            r#"[
                {{"xpub": "{x}", "master_fingerprint": "deadbeef"}},
                {{"xpub": "{x}", "master_fingerprint": "cafebabe", "path": "m/48'/0'/0'/2'"}}
            ]"#,
            x = TEST_XPUB,
        );
        let dir = std::env::temp_dir();
        let path = dir.join("toolkit_cosigners_file_test_two.json");
        std::fs::write(&path, &json).unwrap();
        let specs = parse_cosigners_file(&path).unwrap();
        assert_eq!(specs.len(), 2);
        assert!(specs[0].path.is_none());
        assert!(specs[1].path.is_some());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn cosigners_file_missing_fp_rejected() {
        let json = format!(r#"[{{"xpub": "{x}"}}]"#, x = TEST_XPUB);
        let dir = std::env::temp_dir();
        let path = dir.join("toolkit_cosigners_file_test_missing_fp.json");
        std::fs::write(&path, &json).unwrap();
        let e = parse_cosigners_file(&path).unwrap_err();
        match e {
            ToolkitError::CosignersFile { message } => {
                assert!(message.contains("master_fingerprint required"));
                assert!(message.contains("cosigner index 0"));
            }
            other => panic!("expected CosignersFile, got {:?}", other),
        }
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn multisig_path_family_default_origin_path_strings() {
        assert_eq!(
            MultisigPathFamily::Bip87.default_origin_path(CliNetwork::Mainnet, 0, 2),
            "m/87'/0'/0'"
        );
        assert_eq!(
            MultisigPathFamily::Bip48.default_origin_path(CliNetwork::Testnet, 5, 2),
            "m/48'/1'/5'/2'"
        );
        assert_eq!(MultisigPathFamily::default(), MultisigPathFamily::Bip87);
    }
}
