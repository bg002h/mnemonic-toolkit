//! Cross-validate the freshly-vendored md-codec `bch_decode` port against
//! toolkit v0.22.1's own pre-existing vendored copy.
//!
//! Per plan §4.B.2 — timing-critical: this cell runs at Phase B.2 BEFORE
//! Phase B.7 migration. Post-B.7 the toolkit delegates to md-codec's native
//! API, so a parity check there is tautological (same code, twice). The
//! cell catches Berlekamp–Massey + Chien-search + Forney port drift while
//! both implementations are independent.
//!
//! Toolkit binary expected at `~/.cargo/bin/mnemonic` built from
//! `mnemonic-toolkit-v0.22.1` (d3e1a74). If the binary is missing or its
//! version doesn't match, the test SKIPS gracefully — the parity check is
//! a regression guard, not a hard build dependency.

use md_codec::chunk::split;
use md_codec::decode_with_correction;
use md_codec::encode::Descriptor;
use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
use md_codec::tag::Tag;
use md_codec::tlv::TlvSection;
use md_codec::tree::{Body, Node};
use md_codec::use_site_path::UseSitePath;

/// Codex32 alphabet for synthesizing deterministic single-char corruptions.
const CODEX32_ALPHABET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

fn small_descriptor() -> Descriptor {
    Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(OriginPath {
                components: vec![
                    PathComponent {
                        hardened: true,
                        value: 84,
                    },
                    PathComponent {
                        hardened: true,
                        value: 0,
                    },
                    PathComponent {
                        hardened: true,
                        value: 0,
                    },
                ],
            }),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node {
            tag: Tag::Wpkh,
            body: Body::KeyArg { index: 0 },
        },
        tlv: TlvSection::new_empty(),
    }
}

fn corrupt_at(chunk: &str, pos: usize, xor_mask: u8) -> String {
    let mut chars: Vec<char> = chunk.chars().collect();
    let abs_idx = 3 + pos; // skip "md1"
    let original_sym = CODEX32_ALPHABET
        .iter()
        .position(|&b| b == chars[abs_idx].to_ascii_lowercase() as u8)
        .expect("char in alphabet") as u8;
    let new_sym = (original_sym ^ (xor_mask & 0x1F)) & 0x1F;
    chars[abs_idx] = CODEX32_ALPHABET[new_sym as usize] as char;
    chars.iter().collect()
}

/// Extract the last `md1...` line from text-form toolkit `repair` output.
/// The toolkit emits `# Repair report` comment lines followed by one chunk
/// per line.
fn extract_corrected_md1(stdout: &str) -> Option<String> {
    stdout
        .lines()
        .rfind(|l| !l.starts_with('#') && l.to_ascii_lowercase().starts_with("md1"))
        .map(String::from)
}

#[test]
fn parity_smoke_md_against_toolkit_v0_22_1() {
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

    // Generate a known-valid md1 chunk and corrupt one character.
    let d = small_descriptor();
    let chunks = split(&d).expect("split");
    assert_eq!(chunks.len(), 1, "small descriptor must fit in one chunk");
    let original = chunks[0].clone();
    let bad = corrupt_at(&original, 4, 0b10110);
    assert_ne!(bad, original, "corruption changed the chunk");

    // 1. md-codec native: decode_with_correction.
    let (codec_decoded, codec_details) =
        decode_with_correction(&[bad.as_str()]).expect("md-codec must decode");
    assert_eq!(
        codec_decoded, d,
        "md-codec correction restores original descriptor"
    );
    assert_eq!(codec_details.len(), 1, "exactly 1 correction reported");
    let codec_correction = &codec_details[0];

    // 2. Toolkit binary: `mnemonic repair --md1 <bad>`.
    let toolkit_out = std::process::Command::new(&toolkit_bin)
        .args(["repair", "--md1", &bad])
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

    let toolkit_corrected = extract_corrected_md1(&stdout).unwrap_or_else(|| {
        panic!("could not extract corrected md1 from toolkit stdout:\n{stdout}")
    });

    // 3. Cross-validate: toolkit's corrected chunk must equal md-codec's
    // re-encoded version. md-codec doesn't expose the corrected-string
    // directly — it returns the parsed Descriptor + corrections — so
    // re-derive: the corrected chunk must equal `original` (the BCH
    // decoder restored the codeword, and re-encoding a clean codeword is
    // deterministic).
    assert_eq!(
        toolkit_corrected.to_lowercase(),
        original.to_lowercase(),
        "toolkit's corrected output must match md-codec's BCH-corrected codeword"
    );

    // 4. Cross-validate the correction position: toolkit stderr / stdout
    // line cites `position N: 'x' -> 'y'`. Search for the position number
    // and the corrected `now` char from md-codec.
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
        "parity_smoke: md-codec @position {} ({} -> {}) == toolkit-v0.22.1 corrected chunk",
        codec_correction.position, codec_correction.was, codec_correction.now
    );
}
