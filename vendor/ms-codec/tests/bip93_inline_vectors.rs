//! BIP-93 §Test Vectors — full inline corpus pin.
//!
//! ms-codec consumes the BIP-93 codex32 wire format via Andrew
//! Poelstra's `codex32 = "=0.1.0"` crate (exact-pinned). The
//! existing `bip93_cross_format.rs` byte-pins §93.4 only; everything
//! else (§93.1–.3, §93.5, the 64-entry invalid corpus) was
//! transitively delegated to `rust-codex32`. This file closes the
//! upstream-drift surface for all of §Test Vectors.
//!
//! For each valid vector (§93.1–.5), this file asserts:
//!
//!   1. `Codex32String::from_string(spec_string)` returns `Ok(_)`.
//!   2. The parsed string's payload bytes (`data()`), truncated to
//!      the master-seed length the BIP specifies, equal the
//!      spec-published master-seed hex.
//!   3. The parsed value `Display`-formats back to the input string
//!      verbatim (case-preserving; the codex32 spec allows all-upper
//!      and all-lower).
//!
//! For each invalid vector (64 entries), this file asserts:
//!
//!   1. `Codex32String::from_string(spec_string)` returns `Err(_)`.
//!
//! Granular error-variant classification per invalid vector
//! (`InvalidChecksum` / `InvalidLength` / `InvalidChar` / etc.) is
//! deferred to a future cycle — `rust-codex32 =0.1.0`'s error enum
//! is granular enough but the BIP-93 §Invalid section doesn't
//! categorize each entry, and pinning the bucket would amount to
//! pinning `rust-codex32`'s internal classification rather than a
//! BIP-published claim. The coarse `is_err()` assertion is the
//! spec-published claim.
//!
//! BIP-93 spec: <https://github.com/bitcoin/bips/blob/master/bip-0093.mediawiki>.
//! Counts verified at Phase 0 close via `gh api`:
//! `scriptPubKey`-equivalent valid count = 5, invalid `<code>`-bullets = 64.
//!
//! Cycle: v0.8.0 BIP-vector adoption.
//! SPEC: `mnemonic-toolkit/design/SPEC_test_vector_audit_v0_8_0.md` §2.
//! Phase: 2.

use ms_codec::codex32::Codex32String;

// ─── Valid vectors ────────────────────────────────────────────────────────

/// BIP-93 §Test vector 1 — 16-byte master seed, k=0 (no splitting).
const V1_STRING: &str = "ms10testsxxxxxxxxxxxxxxxxxxxxxxxxxx4nzvca9cmczlw";
const V1_MASTER_SEED_HEX: &str = "318c6318c6318c6318c6318c6318c631";

#[test]
fn vector_1_no_split_16_byte_secret() {
    let parsed = Codex32String::from_string(V1_STRING.to_string()).expect("§93.1 parses");
    assert_eq!(
        parsed.to_string(),
        V1_STRING,
        "§93.1 should Display-round-trip case-preservingly"
    );
    let data = parsed.parts().data();
    let expected = hex_to_bytes(V1_MASTER_SEED_HEX);
    assert_eq!(
        &data[..expected.len()],
        &expected[..],
        "§93.1 first 16 bytes of payload should equal the BIP master seed"
    );
}

/// BIP-93 §Test vector 2 — 16-byte master seed, k=2, share index `S`
/// (the recovered secret). The vector publishes shares `A` and `C` plus the
/// computed-by-interpolation share `S`; we pin the `S` share's payload
/// equals the master seed.
const V2_RECOVERED_S_SHARE: &str = "MS12NAMES6XQGUZTTXKEQNJSJZV4JV3NZ5K3KWGSPHUH6EVW";
const V2_MASTER_SEED_HEX: &str = "d1808e096b35b209ca12132b264662a5";

#[test]
fn vector_2_k_of_2_share_s_recovers_secret() {
    let parsed = Codex32String::from_string(V2_RECOVERED_S_SHARE.to_string())
        .expect("§93.2 recovered S share parses");
    assert_eq!(
        parsed.to_string(),
        V2_RECOVERED_S_SHARE,
        "§93.2 should Display-round-trip case-preservingly (uppercase form)"
    );
    let data = parsed.parts().data();
    let expected = hex_to_bytes(V2_MASTER_SEED_HEX);
    assert_eq!(
        &data[..expected.len()],
        &expected[..],
        "§93.2 first 16 bytes of S-share payload should equal the BIP master seed"
    );
}

/// BIP-93 §Test vector 3 — 16-byte master seed, k=3, encoded as the
/// canonical `s` share. The vector also publishes 4 alternative
/// canonical encodings (different last-two-bit choices); we pin the
/// canonical one. Round-tripping all 4 alternates would be redundant
/// (`rust-codex32 =0.1.0`'s decode is many-to-one onto the same
/// underlying secret).
const V3_S_SHARE: &str = "ms13cashsllhdmn9m42vcsamx24zrxgs3qqjzqud4m0d6nln";
const V3_MASTER_SEED_HEX: &str = "ffeeddccbbaa99887766554433221100";

#[test]
fn vector_3_k_of_3_share_s_canonical() {
    let parsed = Codex32String::from_string(V3_S_SHARE.to_string()).expect("§93.3 parses");
    assert_eq!(parsed.to_string(), V3_S_SHARE, "§93.3 round-trips");
    let data = parsed.parts().data();
    let expected = hex_to_bytes(V3_MASTER_SEED_HEX);
    assert_eq!(
        &data[..expected.len()],
        &expected[..],
        "§93.3 first 16 bytes of S-share payload should equal the BIP master seed"
    );
}

/// BIP-93 §Test vector 4 — 32-byte master seed, k=0, identifier `leet`.
/// Already cross-format-pinned at byte level in `bip93_cross_format.rs`; this
/// file pins it again at the parse + master-seed level so a future
/// regression in either path surfaces here too.
const V4_STRING: &str =
    "ms10leetsllhdmn9m42vcsamx24zrxgs3qrl7ahwvhw4fnzrhve25gvezzyqqtum9pgv99ycma";
const V4_MASTER_SEED_HEX: &str = "ffeeddccbbaa99887766554433221100ffeeddccbbaa99887766554433221100";

#[test]
fn vector_4_no_split_32_byte_secret() {
    let parsed = Codex32String::from_string(V4_STRING.to_string()).expect("§93.4 parses");
    assert_eq!(parsed.to_string(), V4_STRING, "§93.4 round-trips");
    let data = parsed.parts().data();
    let expected = hex_to_bytes(V4_MASTER_SEED_HEX);
    assert_eq!(
        &data[..expected.len()],
        &expected[..],
        "§93.4 first 32 bytes of payload should equal the BIP master seed"
    );
}

/// BIP-93 §Test vector 5 — long-codex32 form, 64-byte master seed.
/// All-uppercase input; `Codex32String::from_string` preserves casing in
/// `Display`.
const V5_STRING: &str =
    "MS100C8VSM32ZXFGUHPCHTLUPZRY9X8GF2TVDW0S3JN54KHCE6MUA7LQPZYGSFJD6AN074RXVCEMLH8WU3TK925ACDEFGHJKLMNPQRSTUVWXY06FHPV80UNDVARHRAK";
const V5_MASTER_SEED_HEX: &str =
    "dc5423251cb87175ff8110c8531d0952d8d73e1194e95b5f19d6f9df7c01111104c9baecdfea8cccc677fb9ddc8aec5553b86e528bcadfdcc201c17c638c47e9";

#[test]
fn vector_5_long_codex32_512_bit_secret() {
    let parsed = Codex32String::from_string(V5_STRING.to_string()).expect("§93.5 parses");
    assert_eq!(
        parsed.to_string(),
        V5_STRING,
        "§93.5 (all-uppercase) round-trips"
    );
    let data = parsed.parts().data();
    let expected = hex_to_bytes(V5_MASTER_SEED_HEX);
    assert_eq!(
        &data[..expected.len()],
        &expected[..],
        "§93.5 first 64 bytes of payload should equal the BIP master seed"
    );
}

// ─── Invalid vectors ──────────────────────────────────────────────────────
//
// 64 entries from BIP-93 §Invalid test vectors. The BIP-93 §Invalid prose
// only says "These examples have incorrect checksums" but the inline list
// actually spans multiple failure modes: bad-checksum on `ms10faux...`
// payloads, length-violation variants, truncated-HRP (`0faux...`, `10faux...`,
// `m10faux...`, `s10faux...`), mixed-case (`Ms...`, `mS...`, `MS...`,
// `ms10FAUX...`), and k-digit issues (`ms12faux...`, `ms1faux...`).
//
// The shared invariant: `Codex32String::from_string` MUST return `Err(_)`
// for every entry. Granular error-variant classification is deferred per
// the file-level doc-comment.

const INVALID_VECTORS: &[&str] = &[
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxve740yyge2ghq",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxve740yyge2ghp",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxlk3yepcstwr",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxx6pgnv7jnpcsp",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxx0cpvr7n4geq",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxm5252y7d3lr",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxrd9sukzl05ej",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxc55srw5jrm0",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxgc7rwhtudwc",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxx4gy22afwghvs",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxe8yfm0",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxvm597d",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxme084q0vpht7pe0",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxme084q0vpht7pew",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxqyadsp3nywm8a",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxzvg7ar4hgaejk",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxcznau0advgxqe",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxch3jrc6j5040j",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx52gxl6ppv40mcv",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx7g4g2nhhle8fk",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx63m45uj8ss4x8",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxy4r708q7kg65x",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxurfvwmdcmymdufv",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxcsyppjkd8lz4hx3",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxu6hwvl5p0l9xf3c",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxwqey9rfs6smenxa",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxv70wkzrjr4ntqet",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx3hmlrmpa4zl0v",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxrfggf88znkaup",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxpt7l4aycv9qzj",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxus27z9xtyxyw3",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxcwm4re8fs78vn",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxw0a4c70rfefn4",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxk4pavy5n46nea",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxx9lrwar5zwng4w",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxr335l5tv88js3",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxvu7q9nz8p7dj68v",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxpq6k542scdxndq3",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxkmfw6jm270mz6ej",
    "ms12fauxxxxxxxxxxxxxxxxxxxxxxxxxxzhddxw99w7xws",
    "ms12fauxxxxxxxxxxxxxxxxxxxxxxxxxxxx42cux6um92rz",
    "ms12fauxxxxxxxxxxxxxxxxxxxxxxxxxxxxxarja5kqukdhy9",
    "ms12fauxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxky0ua3ha84qk8",
    "ms12fauxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx9eheesxadh2n2n9",
    "ms12fauxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx9llwmgesfulcj2z",
    "ms12fauxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx02ev7caq6n9fgkf",
    "ms10fauxxxxxxxxxxxxxxxxxxxxxxxxxxxx0z26tfn0ulw3p",
    "ms1fauxxxxxxxxxxxxxxxxxxxxxxxxxxxxxda3kr3s0s2swg",
    "0fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxuqxkk05lyf3x2",
    "10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxuqxkk05lyf3x2",
    "ms0fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxuqxkk05lyf3x2",
    "m10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxuqxkk05lyf3x2",
    "s10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxuqxkk05lyf3x2",
    "0fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxhkd4f70m8lgws",
    "10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxhkd4f70m8lgws",
    "m10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxx8t28z74x8hs4l",
    "s10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxh9d0fhnvfyx3x",
    "Ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxuqxkk05lyf3x2",
    "mS10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxuqxkk05lyf3x2",
    "MS10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxuqxkk05lyf3x2",
    "ms10FAUXsxxxxxxxxxxxxxxxxxxxxxxxxxxuqxkk05lyf3x2",
    "ms10fauxSxxxxxxxxxxxxxxxxxxxxxxxxxxuqxkk05lyf3x2",
    "ms10fauxsXXXXXXXXXXXXXXXXXXXXXXXXXXuqxkk05lyf3x2",
    "ms10fauxsxxxxxxxxxxxxxxxxxxxxxxxxxxUQXKK05LYF3X2",
];

#[test]
fn invalid_corpus_length_is_64() {
    // SPEC §2 invariant: 64 invalid vectors. Guards against silent
    // upstream BIP edits adding/removing entries while this file is
    // out-of-sync.
    assert_eq!(INVALID_VECTORS.len(), 64);
}

#[test]
fn all_invalid_vectors_rejected_by_codex32() {
    let mut failures = Vec::new();
    for (i, s) in INVALID_VECTORS.iter().enumerate() {
        match Codex32String::from_string(s.to_string()) {
            Ok(parsed) => failures.push(format!(
                "vector[{i}] should be rejected but parsed successfully: input={s:?} parsed={parsed:?}"
            )),
            Err(_e) => {
                // Coarse `is_err()` matches the BIP-93 §Invalid claim.
                // Granular variant pinning is deferred (see file
                // doc-comment).
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} of 64 invalid vectors leaked through `Codex32String::from_string`:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// ─── Helpers ──────────────────────────────────────────────────────────────

fn hex_to_bytes(s: &str) -> Vec<u8> {
    assert!(s.len() % 2 == 0, "hex must be even-length: {s}");
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("valid hex"))
        .collect()
}
