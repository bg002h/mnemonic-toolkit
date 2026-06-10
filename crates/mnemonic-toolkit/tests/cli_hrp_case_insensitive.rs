//! v0.53.3 (audit M11) — case-insensitive HRP PROBES across the toolkit's
//! card-intake surface. BIP-173 uppercase-QR cards exist in the wild; the
//! toolkit's prefix probes lowercase a COPY for routing only and pass the
//! ORIGINAL string to the sibling codecs, which remain the authority on
//! case (mk-codec rejects mixed; ms-codec 0.4.0's envelope layer rejects
//! uppercase past codex32 — companion `ms1-envelope-uppercase-bip173`).
//!
//! Per `design/PLAN_hrp_case_insensitive_probes.md` (contract = the ratified
//! recon `cycle-prep-recon-hrp-case-insensitive.md` §Recommended scope):
//!   - uppercase MK1 end-to-end: inspect + restore --cosigner + --target-xpub
//!   - uppercase MD1 end-to-end: inspect + xpub-search --descriptor
//!   - uppercase MS1 positional: M3 advisory FIRES + ms-codec-attributed
//!     error (WrongHrp "MS") with NO full-string echo
//!   - verify-bundle uppercase positionals round-trip
//!   - decision-2 RIDER: UnknownHrp Display truncates `got` to its first 12
//!     chars + `…` so a near-miss secret-ish positional never echoes in full
//!   - silent-payment --secret <UPPER ms1> → ms-codec-attributed error
//!   - mixed-case ms1/mk1 → clean codec-attributed errors; md1 mixed-case
//!     ACCEPTED (characterization of md-codec BIP-173 leniency — not a fix)
//!   - typed flags accept consistent-case values post-relaxation (the
//!     v0.24.0 I5 case-mismatch rejection is relaxed: codecs own case)
//!
//! Fixtures: uppercase forms are derived by `.to_uppercase()` on the same
//! lowercase fixtures used by `cli_positional_hrp_autodetect.rs` (canonical
//! `abandon × 11 about` toolkit-emitted bundle; `VALID_MS1` mirrors
//! `src/repair.rs` tests — in-crate const, integration tests carry their
//! own copy of the literal).

use assert_cmd::Command;
use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use bitcoin::NetworkKind;
use predicates::prelude::*;
use serde_json::Value;
use std::str::FromStr;

const VALID_MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
const VALID_MK1_CHUNK0: &str = "mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4";
const VALID_MK1_CHUNK1: &str = "mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh";
const VALID_MD1_CHUNK0: &str = "md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np";
const VALID_MD1_CHUNK1: &str = "md1fgdxlpqf2zcgefcpupmel75q5435j7seugaj5jr7qyur6vt76es5cdeyrq7zdy0d";
const VALID_MD1_CHUNK2: &str = "md1fgdxlpq3xa2dk8vwpj7gx74hwqxqdp083jehp5tdrfa0n5zdfkqcdlrvnh5r62jn";

const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const C1_PHRASE: &str =
    "legal winner thank year wave sausage worth useful legal winner thank yellow";
const C2_PHRASE: &str =
    "letter advice cage absurd amount doctor acoustic avoid letter advice cage above";

/// Compute the xpub at `path` for `phrase` (no passphrase).
fn xpub_at(phrase: &str, path: &str) -> Xpub {
    let mnemonic = Mnemonic::parse_in(bip39::Language::English, phrase).unwrap();
    let seed = mnemonic.to_seed("");
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(NetworkKind::Main, &seed).unwrap();
    let dp = DerivationPath::from_str(path).unwrap();
    let xpriv = master.derive_priv(&secp, &dp).unwrap();
    Xpub::from_priv(&secp, &xpriv)
}

/// Master fingerprint of `phrase`.
fn master_fp(phrase: &str) -> bitcoin::bip32::Fingerprint {
    let mnemonic = Mnemonic::parse_in(bip39::Language::English, phrase).unwrap();
    let seed = mnemonic.to_seed("");
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(NetworkKind::Main, &seed).unwrap();
    Xpub::from_priv(&secp, &master).fingerprint()
}

/// Bundle a 2-of-3 wsh-sortedmulti and return (md1 chunks, per-cosigner mk1
/// chunks). Mirrors `cli_restore_multisig.rs::bundle_multisig`.
fn bundle_multisig() -> (Vec<String>, Vec<Vec<String>>) {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={PHRASE}"),
            "--slot",
            &format!("@1.phrase={C1_PHRASE}"),
            "--slot",
            &format!("@2.phrase={C2_PHRASE}"),
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).expect("bundle JSON");
    let md1: Vec<String> = v["md1"]
        .as_array()
        .expect("md1 array")
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect();
    let mk1_per: Vec<Vec<String>> = v["mk1"]
        .as_array()
        .expect("mk1 array")
        .iter()
        .map(|el| match el {
            Value::String(s) => vec![s.clone()],
            Value::Array(inner) => {
                inner.iter().map(|c| c.as_str().unwrap().to_string()).collect()
            }
            other => panic!("unexpected mk1 element: {other:?}"),
        })
        .collect();
    (md1, mk1_per)
}

// ============================================================================
// uppercase MK1 — decodes end-to-end (mk-codec self-normalizes)
// ============================================================================

/// Uppercase MK1 positionals through inspect: the case-insensitive probe
/// routes them to the mk1 bucket; mk-codec lowercase-normalizes and decodes.
#[test]
fn inspect_positional_uppercase_mk1_decodes() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "inspect",
            &VALID_MK1_CHUNK0.to_uppercase(),
            &VALID_MK1_CHUNK1.to_uppercase(),
        ])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("kind: mk1"))
        .stdout(predicate::str::contains("xpub: xpub"));
}

/// Typed `--mk1 MK1…` accepted post-relaxation (the v0.24.0 I5 case-mismatch
/// rejection in `validate_flag_hrp` is relaxed; codecs are the case
/// authority and mk-codec accepts consistent uppercase).
#[test]
fn inspect_typed_flag_uppercase_mk1_accepted() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "inspect",
            "--mk1",
            &VALID_MK1_CHUNK0.to_uppercase(),
            "--mk1",
            &VALID_MK1_CHUNK1.to_uppercase(),
        ])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("kind: mk1"));
}

/// Uppercase MK1 through `restore --cosigner @1=<MK1…>`: the restore.rs
/// all-chunks-mk1 probe is case-insensitive; mk-codec decodes and the
/// cosigner cross-check succeeds (PARTIAL — only @1 verified).
#[test]
fn restore_cosigner_uppercase_mk1_cross_checked() {
    let (md1, mk1_per) = bundle_multisig();
    let mut a = vec!["restore".to_string(), "--network".into(), "mainnet".into()];
    for c in &md1 {
        a.push("--md1".into());
        a.push(c.clone());
    }
    for chunk in &mk1_per[1] {
        a.push("--cosigner".into());
        a.push(format!("@1={}", chunk.to_uppercase()));
    }
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&a)
        .assert()
        .code(0)
        .stdout(predicate::str::contains("cross-checked"))
        .stderr(predicate::str::contains("PARTIAL"));
}

/// Uppercase MK1 through `xpub-search path-of-xpub --target-xpub`: the
/// target-intake mk1-vs-SLIP-0132 dispatch probe is case-insensitive.
#[test]
fn xpub_search_target_xpub_uppercase_mk1_accepted() {
    let xpub = xpub_at(PHRASE, "m/84'/0'/0'");
    let origin_path = DerivationPath::from_str("m/84'/0'/0'").unwrap();
    let card = mk_codec::KeyCard::new(vec![[0u8; 4]], None, origin_path, xpub);
    let chunks = mk_codec::encode(&card).expect("mk_codec::encode");
    let mk1_upper = chunks.join(" ").to_uppercase();

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "path-of-xpub",
            "--phrase-stdin",
            "--target-xpub",
            &mk1_upper,
            "--json",
        ])
        .write_stdin(PHRASE)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_str(&String::from_utf8(out).unwrap()).unwrap();
    assert_eq!(v["path"], "m/84'/0'/0'");
}

// ============================================================================
// uppercase MD1 — decodes end-to-end (md-codec self-normalizes)
// ============================================================================

/// Uppercase MD1 positionals through inspect.
#[test]
fn inspect_positional_uppercase_md1_decodes() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "inspect",
            &VALID_MD1_CHUNK0.to_uppercase(),
            &VALID_MD1_CHUNK1.to_uppercase(),
            &VALID_MD1_CHUNK2.to_uppercase(),
        ])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("kind: md1"));
}

/// Uppercase MD1 through `xpub-search account-of-descriptor --descriptor`:
/// `detect_shape`'s md1 probe is case-insensitive, so an uppercase card is
/// routed to the md1 shape (NOT mis-detected as a literal-xpub descriptor)
/// and md-codec decodes it.
#[test]
fn xpub_search_descriptor_uppercase_md1_routes_md1_shape() {
    let xpub = xpub_at(PHRASE, "m/84'/0'/0'");
    let fp_hex = master_fp(PHRASE).to_string();
    let descriptor_template = format!("wpkh(@0[{fp_hex}/84'/0'/0']/<0;1>/*)");
    // Emit a bundle (full single-sig) to obtain the md1 card.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            &descriptor_template,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={xpub}"),
            "--slot",
            &format!("@0.fingerprint={fp_hex}"),
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&stdout).unwrap();
    let md1_strs: Vec<String> = v["md1"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect();
    // Inline `--descriptor` value: one chunk per line (parse_md1's shape),
    // whole value uppercased.
    let descriptor_upper = md1_strs.join("\n").to_uppercase();

    let xs_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "xpub-search",
            "account-of-descriptor",
            "--phrase",
            PHRASE,
            "--descriptor",
            &descriptor_upper,
            "--json",
        ])
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_str(&String::from_utf8(xs_out).unwrap()).unwrap();
    assert_eq!(v["descriptor_shape"], "md1");
    assert_eq!(v["result"], "match");
}

// ============================================================================
// uppercase MS1 — routes correctly; ms-codec is the (case-sensitive) authority
// ============================================================================

/// Uppercase MS1 positional through inspect: the M3 secret-in-argv advisory
/// FIRES (the probe is no longer case-gated), the error is ms-codec-
/// attributed (`WrongHrp { got: "MS" }` via the friendly mapper — uppercase
/// ms1 cannot decode until the ms-codec envelope companion ships), and the
/// FULL secret string is NEVER echoed to stderr (pre-fix, `UnknownHrp`
/// echoed it verbatim).
#[test]
fn inspect_positional_uppercase_ms1_advisory_fires_ms_codec_attributed_no_echo() {
    let upper = VALID_MS1.to_uppercase();
    let assert = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["inspect", &upper])
        .assert()
        .code(2);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("warning: secret material on argv (positional ms1)"),
        "M3 positional-ms1 advisory must fire for an uppercase card; got stderr: {stderr}"
    );
    assert!(
        stderr.contains(r#"ms1 wrong HRP: got "MS", expected "ms""#),
        "error must be ms-codec-attributed (WrongHrp); got stderr: {stderr}"
    );
    assert!(
        !stderr.contains(&upper),
        "the full secret string must never be echoed to stderr; got stderr: {stderr}"
    );
}

/// Uppercase MS1 positional through verify-bundle rides the same shared
/// classifier: no `UnknownHrp` full-string echo; the failure is ms-codec-
/// attributed.
#[test]
fn verify_bundle_positional_uppercase_ms1_no_echo_ms_codec_attributed() {
    let upper = VALID_MS1.to_uppercase();
    let assert = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-bundle",
            "--network",
            "mainnet",
            "--template",
            "bip84",
            "--slot",
            "@0.xpub=xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V",
            &upper,
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains("does not begin with a recognized HRP prefix"),
        "uppercase ms1 must HRP-classify (not UnknownHrp); got stderr: {stderr}"
    );
    assert!(
        !stderr.contains(&upper),
        "the full secret string must never be echoed to stderr; got stderr: {stderr}"
    );
}

/// `--ms1 <UPPER full-length>` typed flag: post-relaxation the value passes
/// `validate_flag_hrp` and the repair path's own `parse_chunk` pre-gate
/// (which lowercases), then reaches ms-codec — which rejects the uppercase
/// envelope HRP. Pins the REPAIR-path translation marker
/// (`RepairError::HrpMismatch` from `MsErr::WrongHrp`).
#[test]
fn repair_ms1_flag_uppercase_reaches_ms_codec_repair_marker() {
    let upper = VALID_MS1.to_uppercase();
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", "--ms1", &upper])
        .assert()
        .code(2)
        .stderr(predicate::str::contains(
            "repair: chunk 0 HRP mismatch — expected 'ms', found 'MS'",
        ));
}

/// `silent-payment --secret <UPPER full ms1>`: the secret-kind dispatch
/// probe is case-insensitive, so the ms-codec-attributed error replaces the
/// pre-fix generic "expected a seed-bearing secret" refusal.
#[test]
fn silent_payment_uppercase_ms1_ms_codec_attributed() {
    let upper = VALID_MS1.to_uppercase();
    let assert = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["silent-payment", "--secret", &upper])
        .assert()
        .failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains(r#"ms1 wrong HRP: got "MS", expected "ms""#),
        "error must be ms-codec-attributed; got stderr: {stderr}"
    );
    assert!(
        !stderr.contains("seed-bearing"),
        "the generic refusal must be replaced by the ms-codec-attributed error; \
         got stderr: {stderr}"
    );
}

// ============================================================================
// verify-bundle — uppercase mk1/md1 positionals round-trip
// ============================================================================

/// Uppercase mk1 + md1 positionals through verify-bundle (watch-only
/// bip84-mainnet fixture): the shared classifier routes them and both codecs
/// decode → `result: ok`.
#[test]
fn verify_bundle_positional_uppercase_mk1_md1_round_trip() {
    let fixture =
        std::fs::read_to_string("tests/vectors/v0_1/bip84-mainnet.txt").expect("fixture exists");
    let mk1: Vec<String> = fixture
        .lines()
        .filter(|l| l.starts_with("mk1") && !l.contains(' ') && !l.contains('-'))
        .map(String::from)
        .collect();
    let md1: Vec<String> = fixture
        .lines()
        .filter(|l| l.starts_with("md1") && !l.contains(' ') && !l.contains('-'))
        .map(String::from)
        .collect();
    assert!(!mk1.is_empty() && !md1.is_empty(), "fixture must yield mk1+md1");

    let mk_refs: Vec<&str> = mk1.iter().map(|s| s.as_str()).collect();
    let card = mk_codec::decode(&mk_refs).expect("mk1 decodes");
    let xpub_str = card.xpub.to_string();
    let fp_str = card
        .origin_fingerprint
        .expect("fingerprint present")
        .to_string()
        .to_lowercase();

    let mut argv: Vec<String> = vec![
        "verify-bundle".into(),
        "--slot".into(),
        format!("@0.xpub={xpub_str}"),
        "--slot".into(),
        format!("@0.fingerprint={fp_str}"),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "bip84".into(),
    ];
    for s in &mk1 {
        argv.push(s.to_uppercase());
    }
    for s in &md1 {
        argv.push(s.to_uppercase());
    }

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&argv)
        .assert()
        .success()
        .stdout(predicate::str::contains("result: ok"));
}

// ============================================================================
// mixed-case — codecs are the authority (clean attributed errors; md1 leniency)
// ============================================================================

/// Mixed-case ms1: the probe routes it; ms-codec/codex32 rejects mixed case
/// with a clean attributed error (no panic, no UnknownHrp echo).
#[test]
fn inspect_mixed_case_ms1_codec_attributed() {
    // Capitalize the first letter only → mixed case.
    let mixed = format!("Ms{}", &VALID_MS1[2..]);
    let assert = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["inspect", &mixed])
        .assert()
        .failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("ms1 codex32:") && stderr.contains("InvalidCase"),
        "mixed-case ms1 must produce the codex32 InvalidCase attributed error; \
         got stderr: {stderr}"
    );
    assert!(
        !stderr.contains("does not begin with a recognized HRP prefix"),
        "mixed-case ms1 must HRP-classify (not UnknownHrp); got stderr: {stderr}"
    );
}

/// Mixed-case mk1: mk-codec rejects with its dedicated MixedCase error.
#[test]
fn inspect_mixed_case_mk1_codec_attributed() {
    let mixed = format!("Mk{}", &VALID_MK1_CHUNK0[2..]);
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["inspect", &mixed])
        .assert()
        .failure()
        .stderr(predicate::str::contains("mk1 mixed case in input string"));
}

/// Mixed-case md1 is ACCEPTED — characterization of md-codec's BIP-173
/// leniency (per-char lowercase, no mixed-case rejection). Codec-side
/// observation (`md-codec-accepts-mixed-case-bip173-leniency`), NOT a
/// toolkit bug; do not "fix" toolkit-side.
#[test]
fn inspect_mixed_case_md1_accepted_characterization() {
    let mixed0 = format!("Md{}", &VALID_MD1_CHUNK0[2..]);
    let mixed1 = format!("Md{}", &VALID_MD1_CHUNK1[2..]);
    let mixed2 = format!("Md{}", &VALID_MD1_CHUNK2[2..]);
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["inspect", &mixed0, &mixed1, &mixed2])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("kind: md1"));
}

// ============================================================================
// decision-2 RIDER — UnknownHrp echo truncation
// ============================================================================

/// A LONG unknown-HRP positional (51-char `xs1…`, secret-shaped) must NOT be
/// echoed in full by `UnknownHrp`'s Display: only the first 12 chars + `…`
/// appear, alongside the existing "does not begin with a recognized HRP
/// prefix" prose. (Pre-fix, error.rs formatted the full `{got}` — a
/// near-miss secret-ish positional leaked verbatim to stderr.)
#[test]
fn unknown_hrp_long_positional_echo_truncated() {
    let long = format!("xs1{}", "q".repeat(48)); // 51 chars, secret-shaped
    let truncated_head: String = long.chars().take(12).collect();
    let assert = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", &long])
        .assert()
        .code(2);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains(&long),
        "the full unknown-HRP string must not be echoed; got stderr: {stderr}"
    );
    assert!(
        stderr.contains(&format!("{truncated_head}…")),
        "expected the truncated head + ellipsis ({truncated_head}…); got stderr: {stderr}"
    );
    assert!(
        stderr.contains("does not begin with a recognized HRP prefix"),
        "UnknownHrp prose must be preserved; got stderr: {stderr}"
    );
}
