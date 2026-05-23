//! v0.7 Phase 2 — `mnemonic convert` Casascius mini-key decode.
//! Reference: <https://en.bitcoin.it/wiki/Mini_private_key_format>.
//! Decode-only contract (SPEC §13). Compressed flag = false (Casascius
//! predates BIP-32 compressed-pubkey convention).

use assert_cmd::Command;

// --- Reference vectors ---
//
// 22-char canonical (Casascius wiki): SzavMBLoXU6kDrqtUVmffv
//   privkey hex = SHA256("SzavMBLoXU6kDrqtUVmffv")
//                = e9873d79c6d87dc0fb6a5778633389f4453213303da61f20bd67fc233aa33262
//   <https://en.bitcoin.it/wiki/Mini_private_key_format>
const VEC22_KEY: &str = "SzavMBLoXU6kDrqtUVmffv";
const VEC22_WIF_MAINNET: &str = "5Kb8kLf9zgWQnogidDA76MzPL6TsZZY36hWXMssSzNydYXYB9KF";

// 30-char canonical (Casascius wiki): S6c56bnXQiBjk9mqSYE7ykVQ7NzrRy
//   privkey hex = SHA256("S6c56bnXQiBjk9mqSYE7ykVQ7NzrRy")
//                = 4c7a9640c72dc2099f23715d0c8a0d8a35f8906e3cab61dd3f78b67bf887c9ab
const VEC30_KEY: &str = "S6c56bnXQiBjk9mqSYE7ykVQ7NzrRy";
const VEC30_WIF_MAINNET: &str = "5JPy8Zg7z4P7RSLsiqcqyeAF1935zjNUdMxcDeVrtU1oarrgnB7";
const VEC30_WIF_TESTNET: &str = "92AbiJVfaHTFPVrAMBWkrEiCeoPo9tufyJpZJGrNECkrMrR4VGx";

// 26-char fixture: brute-forced for typo-checksum compliance.
// Generator: Python random.Random(seed=1) selecting from base58 alphabet
// after literal 'S'; first candidate satisfying SHA256(key + "?")[0] == 0x00.
// Public canonical 26-char Casascius vectors are not widely cataloged; this is
// a test-only fixture asserting the 26-char length class round-trips. Privkey
// hex = SHA256("S2WSthnpsFbmS1btGUBjCNjG5r")
//      = 0a6c707fb693d8e080a9ab2c4650882c84c2d13a2a14603b2d5b6018a0cbecf7
const VEC26_KEY: &str = "S2WSthnpsFbmS1btGUBjCNjG5r";
const VEC26_WIF_MAINNET: &str = "5HtsqZr2VMtZnUjHfJ2dKDUgd8beQTnEvUW1w1YnQfercBLdQUH";

/// Helper: extract value from `<node>: <value>\n` stdout.
fn convert_value(args: &[&str]) -> String {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(args)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let line = stdout.trim();
    let colon = line.find(": ").expect("convert output must be '<node>: <value>'");
    line[colon + 2..].to_string()
}

// ============================================================================
// Decode happy paths — one per length class
// ============================================================================

#[test]
fn decode_minikey_22char_to_wif_mainnet() {
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("minikey={VEC22_KEY}"),
        "--to",
        "wif",
    ]);
    assert_eq!(out, VEC22_WIF_MAINNET);
    // Mainnet uncompressed WIFs start with '5'.
    assert!(out.starts_with('5'), "mainnet uncompressed WIF must start with '5'; got {out:?}");
}

#[test]
fn decode_minikey_26char_to_wif_mainnet() {
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("minikey={VEC26_KEY}"),
        "--to",
        "wif",
    ]);
    assert_eq!(out, VEC26_WIF_MAINNET);
    assert!(out.starts_with('5'));
}

#[test]
fn decode_minikey_30char_to_wif_mainnet() {
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("minikey={VEC30_KEY}"),
        "--to",
        "wif",
    ]);
    assert_eq!(out, VEC30_WIF_MAINNET);
    assert!(out.starts_with('5'));
}

#[test]
fn decode_minikey_30char_to_wif_testnet() {
    // Testnet uncompressed WIFs start with '9'.
    let out = convert_value(&[
        "convert",
        "--from",
        &format!("minikey={VEC30_KEY}"),
        "--to",
        "wif",
        "--network",
        "testnet",
    ]);
    assert_eq!(out, VEC30_WIF_TESTNET);
    assert!(out.starts_with('9'), "testnet uncompressed WIF must start with '9'; got {out:?}");
}

// ============================================================================
// Refusals — SPEC §3.d / §13
// ============================================================================

#[test]
fn refusal_minikey_invalid_format_no_s_prefix() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            "minikey=NotS22Charsxxxxxxxxxx",
            "--to",
            "wif",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.ends_with(        "error: --from minikey requires a Casascius mini-key string (22/26/30 chars, starting with uppercase 'S'); supplied value does not match.\n"),
        "stderr must end with byte-exact SPEC error text; got {:?}",
        stderr,
    )
}

#[test]
fn refusal_minikey_invalid_format_wrong_length() {
    // 23 chars, S-prefixed — wrong length class.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            "minikey=Sxxxxxxxxxxxxxxxxxxxxxx",
            "--to",
            "wif",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.ends_with(        "error: --from minikey requires a Casascius mini-key string (22/26/30 chars, starting with uppercase 'S'); supplied value does not match.\n"),
        "stderr must end with byte-exact SPEC error text; got {:?}",
        stderr,
    )
}

#[test]
fn refusal_minikey_invalid_checksum() {
    // 22-char S-prefixed, but SHA256("Sxxxxxxxxxxxxxxxxxxxxx" + "?")[0] != 0x00.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            "minikey=Sxxxxxxxxxxxxxxxxxxxxx",
            "--to",
            "wif",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.ends_with(        "error: invalid Casascius mini-key checksum (SHA256(key + \"?\")[0] != 0x00); supplied string is not a valid Casascius mini-key.\n"),
        "stderr must end with byte-exact SPEC error text; got {:?}",
        stderr,
    )
}

#[test]
fn refusal_minikey_to_xpub_decode_only() {
    // §3.d: minikey → non-wif surfaces with distinct decode-only refusal pointing at wif.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("minikey={VEC22_KEY}"),
            "--to",
            "xpub",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.ends_with(        "error: --from minikey only supports --to wif (decode-only); cannot convert to xpub.\n"),
        "stderr must end with byte-exact SPEC error text; got {:?}",
        stderr,
    )
}

#[test]
fn refusal_minikey_to_phrase_decode_only() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("minikey={VEC30_KEY}"),
            "--to",
            "phrase",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.ends_with(        "error: --from minikey only supports --to wif (decode-only); cannot convert to phrase.\n"),
        "stderr must end with byte-exact SPEC error text; got {:?}",
        stderr,
    )
}

#[test]
fn refusal_wif_to_minikey_one_way() {
    // SPEC §13: no `(*, MiniKey)` edge — generation requires brute-force.
    const SAMPLE_WIF: &str = "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("wif={SAMPLE_WIF}"),
            "--to",
            "minikey",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.ends_with(        "error: --to minikey is one-way (mini-key generation requires brute-force search for typo-checksum byte; no inverse derivation).\n"),
        "stderr must end with byte-exact SPEC error text; got {:?}",
        stderr,
    )
}

// ============================================================================
// v0.34.5 — MiniKey stdout-redaction: the echoed `from_value` in --json must
// be redacted (MiniKey is a private-key carrier). Closes
// `convert-minikey-stdout-redaction`.
// ============================================================================

#[test]
fn minikey_input_redacted_in_json_from_value() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["convert", "--from", &format!("minikey={VEC22_KEY}"), "--to", "wif", "--json"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(stdout.trim()).expect("valid convert JSON");
    assert_eq!(v["from_node"], "minikey");
    assert!(
        v["from_value"].is_null(),
        "minikey from_value must be redacted in --json; got: {}",
        v["from_value"]
    );
    // The decoded WIF output is itself secret-bearing and still appears in `to`.
    assert!(v["to"][0]["value"].as_str().unwrap().starts_with('5'));
    // The minikey private key must NOT leak anywhere in the JSON.
    assert!(!stdout.contains(VEC22_KEY), "minikey input leaked into JSON: {stdout}");
}
