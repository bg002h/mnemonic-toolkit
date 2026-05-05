//! Input source resolution: argv vs stdin, phrase normalization,
//! fingerprint parsing.
//!
//! Realizes SPEC §3.2 (stdin uniform), §2.1.5 (--master-fingerprint
//! 8-hex case-insensitive), §2.1.6 (concurrent stdin guard).

use crate::error::{BitcoinErrorKind, ToolkitError};
use bitcoin::bip32::Fingerprint;
use std::io::Read;
use std::str::FromStr;

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
}
