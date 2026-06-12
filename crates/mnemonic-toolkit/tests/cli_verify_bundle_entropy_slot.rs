//! v0.43.1 — `verify-bundle --descriptor … --slot @N.entropy=<hex>` binding arm.
//!
//! FOLLOWUP `verify-bundle-descriptor-entropy-slot-gap`: the descriptor-mode
//! binding loop in `verify_bundle.rs` had arms for Phrase/Seedqr, Xpub, and Ms1
//! but NO `Entropy` arm, so a raw-`entropy` cosigner fell to the catch-all
//! (`DescriptorReparseFailed`, exit 4). This suite round-trips raw-entropy
//! cosigners through the new arm — the `bundle` Entropy arm (`bundle.rs:1438`)
//! and the new `verify_bundle` Entropy arm are DISTINCT code paths that must
//! agree, which is what the round-trips assert. See
//! design/SPEC_verify_bundle_entropy_slot.md §5.

use assert_cmd::Command;
use predicates::prelude::*;

/// Single-`@0` non-canonical descriptor (from `cli_ms1_slot.rs:294`).
const NONCANONICAL_DESC: &str = "tr(NUMS,and_v(v:pk(@0),after(12000000)))";
/// Proven non-canonical 3-cosigner fixture that bundles secret slots with
/// `.success()` (from `cli_non_canonical_descriptor.rs:22`).
const ANDOR3_DESC: &str =
    "wsh(andor(pkh(@0),after(12000000),or_i(and_v(v:pkh(@1),older(4032)),and_v(v:pkh(@2),older(32768)))))";

const TREZOR_12_ZERO: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const BIP39_TEST_3: &str =
    "letter advice cage absurd amount doctor acoustic avoid letter advice cage above";

/// Extract (ms1, mk1-flat, md1) from a `bundle --json` envelope. `mk1` may be a
/// flat array (single placeholder) or nested per-cosigner; both flatten to one
/// chunk list. Clone of `cli_ms1_slot.rs::extract_cards`.
fn extract_cards(v: &serde_json::Value) -> (Vec<String>, Vec<String>, Vec<String>) {
    let ms1: Vec<String> = v["ms1"]
        .as_array()
        .expect("ms1 array")
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect();
    let mut mk1: Vec<String> = Vec::new();
    for el in v["mk1"].as_array().expect("mk1 array") {
        match el {
            serde_json::Value::String(s) => mk1.push(s.clone()),
            serde_json::Value::Array(inner) => {
                for chunk in inner {
                    mk1.push(chunk.as_str().unwrap().to_string());
                }
            }
            other => panic!("unexpected mk1 element shape: {other:?}"),
        }
    }
    let md1: Vec<String> = v["md1"]
        .as_array()
        .expect("md1 array")
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect();
    (ms1, mk1, md1)
}

/// Run a descriptor-mode `bundle --json` with the given slot args and return the
/// extracted (ms1, mk1, md1) cards. `extra` carries `--language`/`--account`/
/// `--passphrase` etc.
fn bundle_cards(
    desc: &str,
    slots: &[String],
    extra: &[&str],
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let mut args: Vec<String> = vec![
        "bundle".into(),
        "--descriptor".into(),
        desc.into(),
        "--network".into(),
        "mainnet".into(),
    ];
    for e in extra {
        args.push((*e).into());
    }
    for s in slots {
        args.push("--slot".into());
        args.push(s.clone());
    }
    args.push("--json".into());
    args.push("--no-engraving-card".into());

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .assert()
        .success();
    let v: serde_json::Value =
        serde_json::from_slice(&out.get_output().stdout).expect("descriptor bundle JSON");
    extract_cards(&v)
}

/// Assemble verify-bundle args: descriptor + network + the slot re-specs + all
/// emitted cards + any `extra` (language/account/passphrase).
fn verify_args(
    desc: &str,
    slots: &[String],
    cards: &(Vec<String>, Vec<String>, Vec<String>),
    extra: &[&str],
) -> Vec<String> {
    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--descriptor".into(),
        desc.into(),
        "--network".into(),
        "mainnet".into(),
    ];
    for e in extra {
        args.push((*e).into());
    }
    for s in slots {
        args.push("--slot".into());
        args.push(s.clone());
    }
    for s in &cards.0 {
        args.push("--ms1".into());
        args.push(s.clone());
    }
    for s in &cards.1 {
        args.push("--mk1".into());
        args.push(s.clone());
    }
    for s in &cards.2 {
        args.push("--md1".into());
        args.push(s.clone());
    }
    args
}

/// 32-byte (24-word) raw entropy at `@0` on a single-`@0` descriptor round-trips
/// through the new Entropy arm → `result: ok`.
#[test]
fn round_trip_len32() {
    let hex = hex::encode([0x07u8; 32]);
    let slots = vec![format!("@0.entropy={hex}")];
    let cards = bundle_cards(NONCANONICAL_DESC, &slots, &[]);

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(verify_args(NONCANONICAL_DESC, &slots, &cards, &[]))
        .assert()
        .code(0)
        .stdout(predicate::str::contains("result: ok"));
}

/// 16-byte (12-word) raw entropy — guards `Mnemonic::from_entropy_in` length
/// acceptance distinct from the 32-byte case.
#[test]
fn round_trip_len16() {
    let hex = hex::encode([0x42u8; 16]);
    let slots = vec![format!("@0.entropy={hex}")];
    let cards = bundle_cards(NONCANONICAL_DESC, &slots, &[]);

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(verify_args(NONCANONICAL_DESC, &slots, &cards, &[]))
        .assert()
        .code(0)
        .stdout(predicate::str::contains("result: ok"));
}

/// Entropy arm at a NON-`@0` position in a multi-`@N` descriptor, composing with
/// the Phrase arm on the sibling slots. Uses the proven-bundlable ANDOR3 fixture.
#[test]
fn nonzero_slot_multi_n() {
    let hex = hex::encode([0x11u8; 16]);
    let slots = vec![
        format!("@0.phrase={TREZOR_12_ZERO}"),
        format!("@1.entropy={hex}"),
        format!("@2.phrase={BIP39_TEST_3}"),
    ];
    let extra = ["--language", "english", "--account", "0"];
    let cards = bundle_cards(ANDOR3_DESC, &slots, &extra);

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(verify_args(ANDOR3_DESC, &slots, &cards, &extra))
        .assert()
        .code(0)
        .stdout(predicate::str::contains("result: ok"));
}

/// The new arm honors `--passphrase` (positive): same passphrase on both sides
/// → `result: ok`.
#[test]
fn passphrase_round_trip() {
    let hex = hex::encode([0x5au8; 32]);
    let slots = vec![format!("@0.entropy={hex}")];
    let cards = bundle_cards(
        NONCANONICAL_DESC,
        &slots,
        &["--passphrase", "correct horse"],
    );

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(verify_args(
            NONCANONICAL_DESC,
            &slots,
            &cards,
            &["--passphrase", "correct horse"],
        ))
        .assert()
        .code(0)
        .stdout(predicate::str::contains("result: ok"));
}

/// The arm derives an INPUT-DEPENDENT key and the verify comparison is LIVE:
/// same entropy, different passphrase on verify → `result: mismatch` (exit 4).
/// (ms1 is entropy-only so it matches; the mk1/md1 checks fail on the differing
/// derived xpub.) Replaces the originally-specified `self_check` test —
/// `--self-check` is a `bundle`-only flag and does not route through the
/// verify-bundle descriptor loop. GREEN assertion keys on stdout `result:
/// mismatch` because both RED (catch-all) and GREEN (BundleMismatch) are exit 4.
#[test]
fn passphrase_mismatch_detected() {
    let hex = hex::encode([0x5au8; 32]);
    let slots = vec![format!("@0.entropy={hex}")];
    let cards = bundle_cards(NONCANONICAL_DESC, &slots, &["--passphrase", "right"]);

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(verify_args(
            NONCANONICAL_DESC,
            &slots,
            &cards,
            &["--passphrase", "wrong"],
        ))
        .assert()
        .code(4)
        .stdout(predicate::str::contains("result: mismatch"));
}
