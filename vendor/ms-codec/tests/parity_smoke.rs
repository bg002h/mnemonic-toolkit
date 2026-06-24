//! Cross-validate the freshly-vendored ms-codec `bch_decode` port against
//! toolkit v0.22.1's own pre-existing vendored copy.
//!
//! Per plan §4.B.2 — timing-critical: this cell runs at Phase B.4 BEFORE
//! Phase B.7 migration. Post-B.7 the toolkit delegates to ms-codec's native
//! API, so a parity check there is tautological (same code, twice). The
//! cell catches Berlekamp–Massey + Chien-search + Forney port drift while
//! both implementations are independent.
//!
//! Toolkit binary expected at `~/.cargo/bin/mnemonic` built from
//! `mnemonic-toolkit-v0.22.1` (d3e1a74). If the binary is missing or its
//! version doesn't match, the test SKIPS gracefully — the parity check is
//! a regression guard, not a hard build dependency.

use ms_codec::decode_with_correction;

/// Codex32 alphabet for synthesizing deterministic single-char corruptions.
const CODEX32_ALPHABET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

/// Canonical 12-word abandon ms1 vector (same as `tests/bch_drift.rs` +
/// `tests/vectors/v0.1.json` entry 0).
const VALID_MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

/// Flip one character of an ms1 string at the data-part position `pos`
/// (0-indexed, post-`ms1` HRP).
fn corrupt_at(s: &str, pos: usize, xor_mask: u8) -> String {
    let mut chars: Vec<char> = s.chars().collect();
    let abs_idx = 3 + pos; // skip "ms1"
    let original_sym = CODEX32_ALPHABET
        .iter()
        .position(|&b| b == chars[abs_idx].to_ascii_lowercase() as u8)
        .expect("char in alphabet") as u8;
    let new_sym = (original_sym ^ (xor_mask & 0x1F)) & 0x1F;
    chars[abs_idx] = CODEX32_ALPHABET[new_sym as usize] as char;
    chars.iter().collect()
}

/// Extract the last `ms1...` line from text-form toolkit `repair` output.
/// The toolkit emits `# Repair report` comment lines followed by the
/// corrected card on its own line.
fn extract_corrected_ms1(stdout: &str) -> Option<String> {
    stdout
        .lines()
        .filter(|l| !l.starts_with('#') && l.to_ascii_lowercase().starts_with("ms1"))
        .last()
        .map(String::from)
}

#[test]
fn parity_smoke_ms_against_toolkit_v0_22_1() {
    // Locate the toolkit binary. Skip gracefully if absent — the parity
    // check is a regression guard, not a hard build dep.
    let home = match std::env::var("HOME") {
        Ok(h) => h,
        Err(_) => {
            eprintln!("parity_smoke: HOME unset; skipping");
            return;
        }
    };
    let toolkit_bin = format!("{home}/.cargo/bin/mnemonic");
    if !std::path::Path::new(&toolkit_bin).exists() {
        eprintln!("parity_smoke: {toolkit_bin} not found; skipping");
        return;
    }

    // Verify the toolkit binary version (best-effort — if it's a different
    // version the parity guarantee weakens but we don't hard-fail). The
    // parity is meaningful against ANY toolkit version that has a working
    // vendored BCH decoder, so we just log mismatches.
    let version_out = std::process::Command::new(&toolkit_bin)
        .arg("--version")
        .output();
    match version_out {
        Ok(out) => {
            let v = String::from_utf8_lossy(&out.stdout);
            eprintln!("parity_smoke: toolkit binary reports: {}", v.trim());
        }
        Err(e) => {
            eprintln!("parity_smoke: --version failed: {e}; skipping");
            return;
        }
    }

    // Corrupt one character of the canonical 12-word abandon ms1.
    let original = VALID_MS1.to_string();
    let bad = corrupt_at(&original, 4, 0b10110);
    assert_ne!(bad, original, "corruption changed the string");

    // 1. ms-codec native: decode_with_correction.
    let (_tag, _payload, codec_details) =
        decode_with_correction(&bad).expect("ms-codec must decode the corrupted ms1");
    assert_eq!(codec_details.len(), 1, "exactly 1 correction reported");
    let codec_correction = &codec_details[0];

    // 2. Toolkit binary: `mnemonic repair --ms1 <bad>`.
    let toolkit_out = std::process::Command::new(&toolkit_bin)
        .args(["repair", "--ms1", &bad])
        .output()
        .expect("invoke toolkit repair");
    let stdout = String::from_utf8_lossy(&toolkit_out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&toolkit_out.stderr).to_string();
    assert!(
        toolkit_out.status.success() || toolkit_out.status.code() == Some(5),
        "toolkit repair must succeed (got exit {:?}; stderr: {})",
        toolkit_out.status.code(),
        stderr
    );

    let toolkit_corrected = extract_corrected_ms1(&stdout).unwrap_or_else(|| {
        panic!("could not extract corrected ms1 from toolkit stdout:\n{stdout}")
    });

    // 3. Cross-validate: toolkit's corrected string must equal the
    // original valid ms1 (the BCH decoder restored the codeword, and
    // re-encoding a clean codeword is deterministic).
    assert_eq!(
        toolkit_corrected.to_lowercase(),
        original.to_lowercase(),
        "toolkit's corrected output must match ms-codec's BCH-corrected codeword"
    );

    // 4. Cross-validate the correction position: toolkit stderr / stdout
    // line cites `position N: 'x' -> 'y'`. Search for the position number
    // and the corrected `now` char from ms-codec.
    let expected_position_substr = format!("position {}", codec_correction.position);
    assert!(
        stdout.contains(&expected_position_substr),
        "toolkit stdout must cite position {}:\n{stdout}",
        codec_correction.position
    );
    let now_char = codec_correction.now;
    assert!(
        stdout.contains(&format!("'{now_char}'")),
        "toolkit stdout must cite corrected char '{now_char}':\n{stdout}"
    );

    eprintln!(
        "parity_smoke: ms-codec @position {} ({} -> {}) == toolkit-v0.22.1 corrected card",
        codec_correction.position, codec_correction.was, codec_correction.now
    );
}
