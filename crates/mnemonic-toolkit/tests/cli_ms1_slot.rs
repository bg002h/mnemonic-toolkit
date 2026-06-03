//! v0.41.0 — `--slot @N.ms1=` integration tests.
//!
//! Phase 1 scope: the canonical-mode descriptor gate. A secret-bearing slot
//! (`ms1` / `seedqr` / `phrase`) carrying an explicit `@N.path=` against a
//! CANONICAL descriptor (`wsh(sortedmulti(...))`, whose `canonical_origin`
//! supplies the per-shape default path) is refused with a
//! `SlotInputViolation{kind:"conflict"}` — exit 2 — because the canonical
//! descriptor already pins the origin path; an explicit per-`@N` path would
//! conflict with it.
//!
//! These fire on the `@0.<secret> + @0.path` subkey set BEFORE the per-cosigner
//! binding loop, so `@1` only needs to be well-formed (a valid xpub) to clear
//! the missing-`@1` check.

use assert_cmd::Command;
use predicates::prelude::*;

// ─────────────────────────────────────────────────────────────────────────────
// Shared fixture builders (Phase 2). `ms_codec` is a direct toolkit dependency,
// so integration tests can construct entr/mnem ms1 cards directly.
// ─────────────────────────────────────────────────────────────────────────────

/// Wire code: Japanese = 1 (indexes `ms_codec::consts::MNEM_LANGUAGE_NAMES`).
#[allow(dead_code)]
const WIRE_JAPANESE: u8 = 1;
/// Wire code: English = 0.
#[allow(dead_code)]
const WIRE_ENGLISH: u8 = 0;

/// Encode raw entropy as an `entr` ms1 card (no intrinsic language).
#[allow(dead_code)]
fn entr_ms1(entropy: &[u8]) -> String {
    ms_codec::encode(ms_codec::Tag::ENTR, &ms_codec::Payload::Entr(entropy.to_vec()))
        .expect("ms_codec::encode entr")
}

/// Encode entropy as a `mnem` ms1 card carrying `wire_lang`.
#[allow(dead_code)]
fn mnem_ms1(entropy: &[u8], wire_lang: u8) -> String {
    ms_codec::encode(
        ms_codec::Tag::ENTR,
        &ms_codec::Payload::Mnem {
            language: wire_lang,
            entropy: entropy.to_vec(),
        },
    )
    .expect("ms_codec::encode mnem")
}

/// The BIP-39 phrase for `entropy` under `lang` (used to cross-check that a
/// mnem ms1 derives the same key as the equivalent `--slot @0.phrase=`).
#[allow(dead_code)]
fn phrase_in(entropy: &[u8], lang: bip39::Language) -> String {
    bip39::Mnemonic::from_entropy_in(lang, entropy)
        .expect("from_entropy_in")
        .to_string()
}

/// Canonical 2-of-2 sorted-multisig descriptor (canonical_origin maps it, so
/// it is NOT treated as non-canonical / explicit-origin).
const CANONICAL_DESC: &str = "wsh(sortedmulti(2,@0,@1))";

/// A well-formed mainnet xpub for the `@1` cosigner (from
/// `cli_export_wallet_coldcard.rs`). Keeps the command from being rejected for
/// a missing `@1` so the canonical gate on `@0` is what fires.
const VALID_XPUB: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";

/// The canonical-mode conflict message (verbatim from
/// `cmd::bundle.rs` / `slot_input.rs`).
const CONFLICT_MSG: &str =
    "has both secret-bearing input and watch-only input; pick one per slot.";

/// `@0.ms1=<...> + @0.path=<...>` against a CANONICAL descriptor → exit 2,
/// SlotInputViolation conflict. The gate fires on the subkey set; the ms1
/// value need not decode (the gate precedes the binding loop).
#[test]
fn ms1_plus_path_canonical_descriptor_refused_exit2() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            CANONICAL_DESC,
            "--network",
            "mainnet",
            "--slot",
            "@0.ms1=ms1stubvalue",
            "--slot",
            "@0.path=48'/0'/0'/2'",
            "--slot",
            &format!("@1.xpub={VALID_XPUB}"),
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(CONFLICT_MSG));
}

/// `@0.seedqr=<...> + @0.path=<...>` against a CANONICAL descriptor → exit 2,
/// SlotInputViolation conflict.
///
/// Baseline note (plan Task 1.4 R0-I2): pre-fix the canonical gate only matched
/// `has_phrase && has_path`, so a `[Seedqr, Path]` set fell through to the
/// per-cosigner binding loop and surfaced as an exit-1 BadInput. The widened
/// `(has_phrase || has_seedqr || has_ms1) && has_path` gate normalizes it to
/// the exit-2 SlotInputViolation. Assert exit 2.
#[test]
fn seedqr_plus_path_canonical_descriptor_refused_exit2() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            CANONICAL_DESC,
            "--network",
            "mainnet",
            "--slot",
            "@0.seedqr=000100020003000400050006000700080009001000110012",
            "--slot",
            "@0.path=48'/0'/0'/2'",
            "--slot",
            &format!("@1.xpub={VALID_XPUB}"),
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(CONFLICT_MSG));
}

// ─────────────────────────────────────────────────────────────────────────────
// Task 2.2 — template `resolve_slots` Ms1 arm (single-sig bundle).
// ─────────────────────────────────────────────────────────────────────────────

/// (a) entr-ms1 byte-identity: `--slot @0.ms1=<entr-ms1 of E>` produces a
/// stdout byte-identical to `--slot @0.entropy=<hex E>` for every valid
/// BIP-39 entropy length {16,20,24,28,32}. (entr ms1 has no intrinsic
/// language → derives identically to a raw-hex entropy slot, SPEC §3.)
#[test]
fn ms1_entr_byte_identical_to_entropy_slot_all_lengths() {
    for len in [16usize, 20, 24, 28, 32] {
        let entropy: Vec<u8> = (0..len).map(|i| (i as u8).wrapping_add(1)).collect();
        let hex = hex::encode(&entropy);
        let ms1 = entr_ms1(&entropy);

        let via_entropy = Command::cargo_bin("mnemonic")
            .unwrap()
            .args([
                "bundle",
                "--template",
                "bip84",
                "--network",
                "mainnet",
                "--slot",
                &format!("@0.entropy={hex}"),
                "--json",
                "--no-engraving-card",
            ])
            .assert()
            .success();
        let via_ms1 = Command::cargo_bin("mnemonic")
            .unwrap()
            .args([
                "bundle",
                "--template",
                "bip84",
                "--network",
                "mainnet",
                "--slot",
                &format!("@0.ms1={ms1}"),
                "--json",
                "--no-engraving-card",
            ])
            .assert()
            .success();

        assert_eq!(
            via_entropy.get_output().stdout,
            via_ms1.get_output().stdout,
            "entr-ms1 slot must produce a byte-identical bundle to the hex-entropy \
             slot for the same entropy (len={len})"
        );
    }
}

/// (b) mnem-japanese ms1: derives the SAME key as the equivalent Japanese
/// phrase under `--language japanese`, AND the emitted card is a `mnem` ms1
/// preserving the language (NOT collapsed to entr).
#[test]
fn ms1_mnem_japanese_matches_phrase_and_emits_mnem_card() {
    let entropy = [0x01u8; 16];
    let ms1 = mnem_ms1(&entropy, WIRE_JAPANESE);
    let ja_phrase = phrase_in(&entropy, bip39::Language::Japanese);

    let via_phrase = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--language",
            "japanese",
            "--slot",
            &format!("@0.phrase={ja_phrase}"),
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let phrase_json: serde_json::Value =
        serde_json::from_slice(&via_phrase.get_output().stdout).expect("phrase JSON");

    let via_ms1 = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.ms1={ms1}"),
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let ms1_json: serde_json::Value =
        serde_json::from_slice(&via_ms1.get_output().stdout).expect("ms1 JSON");

    // Same derived account material (mk1 xpub card(s) carry the derived key).
    assert_eq!(
        phrase_json["mk1"], ms1_json["mk1"],
        "mnem-japanese ms1 must derive the same key(s) as the Japanese phrase"
    );
    assert_eq!(
        phrase_json["master_fingerprint"], ms1_json["master_fingerprint"],
        "mnem-japanese ms1 master fingerprint must match the Japanese phrase"
    );

    // The emitted ms1 card must be a `mnem` PAYLOAD form (the wire kind is in
    // the payload prefix byte, NOT the tag — both entr and mnem cards share the
    // `ms10entr` tag prefix), preserving the Japanese wire language. Decode
    // structurally rather than string-matching.
    let emitted = ms1_json["ms1"][0].as_str().expect("ms1 card present");
    let (_tag, payload) = ms_codec::decode(emitted).expect("emitted ms1 decodes");
    match payload {
        ms_codec::Payload::Mnem { language, .. } => {
            assert_eq!(language, WIRE_JAPANESE, "emitted mnem card must carry the Japanese wire code");
        }
        other => panic!(
            "mnem-japanese ms1 must re-emit a mnem card (language-preserving); \
             got payload {other:?} from card {emitted}"
        ),
    }
}

/// (c) mnem ms1 + a conflicting `--language english` → exit 2,
/// `kind:"language-conflict"` (the helper's refuse-on-conflict policy,
/// SPEC §3).
#[test]
fn ms1_mnem_language_conflict_exit2() {
    let entropy = [0x01u8; 16];
    let ms1 = mnem_ms1(&entropy, WIRE_JAPANESE);

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--language",
            "english",
            "--slot",
            &format!("@0.ms1={ms1}"),
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("language"));
}

// ─────────────────────────────────────────────────────────────────────────────
// Task 2.3 — `bundle_run_unified_descriptor` Ms1 arm (non-canonical /
// explicit-origin descriptor mode). A single-`@0` non-canonical descriptor
// routes the binding through `bundle_run_unified_descriptor`, NOT the template
// `resolve_slots` path.
// ─────────────────────────────────────────────────────────────────────────────

/// A non-canonical single-`@0` descriptor (tr() with TapTree is non-canonical
/// per md-codec's table → default-path inference → descriptor binding loop).
const NONCANONICAL_DESC: &str = "tr(NUMS,and_v(v:pk(@0),after(12000000)))";

/// entr ms1 cosigner derives in descriptor mode (the bundle succeeds and emits
/// an ms1 card for `@0`).
#[test]
fn ms1_entr_descriptor_mode_derives_cosigner() {
    let entropy = [0x07u8; 32];
    let ms1 = entr_ms1(&entropy);

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            NONCANONICAL_DESC,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.ms1={ms1}"),
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let v: serde_json::Value =
        serde_json::from_slice(&out.get_output().stdout).expect("descriptor bundle JSON");
    let card = v["ms1"][0].as_str().expect("ms1 card present");
    // entr ms1 has no intrinsic language → re-emits an entr card.
    let (_t, payload) = ms_codec::decode(card).expect("emitted ms1 decodes");
    assert!(
        matches!(payload, ms_codec::Payload::Entr(_)),
        "entr ms1 descriptor cosigner must re-emit an entr card; got {payload:?}"
    );
}

/// mnem-japanese ms1 cosigner in descriptor mode emits a `mnem` card (the
/// 5-tuple `emit_lang` widening flows the wire language to the single push).
#[test]
fn ms1_mnem_japanese_descriptor_mode_emits_mnem_card() {
    let entropy = [0x01u8; 16];
    let ms1 = mnem_ms1(&entropy, WIRE_JAPANESE);

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            NONCANONICAL_DESC,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.ms1={ms1}"),
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let v: serde_json::Value =
        serde_json::from_slice(&out.get_output().stdout).expect("descriptor bundle JSON");
    let card = v["ms1"][0].as_str().expect("ms1 card present");
    let (_t, payload) = ms_codec::decode(card).expect("emitted ms1 decodes");
    match payload {
        ms_codec::Payload::Mnem { language, .. } => {
            assert_eq!(language, WIRE_JAPANESE, "descriptor mnem card must carry Japanese wire code");
        }
        other => panic!(
            "mnem-japanese ms1 descriptor cosigner must re-emit a mnem card; got {other:?}"
        ),
    }
}
