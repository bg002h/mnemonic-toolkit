//! v0.6 `mnemonic convert` SPEC §3 + §4 refusal taxonomy.
//! Byte-exact stderr; exit 2.

use assert_cmd::Command;

const TREZOR_BIP84_MAINNET_XPUB: &str = "xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9";
const TREZOR_12_ZERO_MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
const SAMPLE_WIF: &str = "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn";

#[test]
fn refusal_xpub_to_entropy_one_way_barrier() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={TREZOR_BIP84_MAINNET_XPUB}"),
            "--to",
            "entropy",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.ends_with(        "error: --to entropy is cryptographically unrecoverable from --from xpub (one-way derivation barrier)\n"),
        "stderr must end with byte-exact SPEC error text; got {:?}",
        stderr,
    )
}

#[test]
fn refusal_xpub_to_xprv_one_way_barrier() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={TREZOR_BIP84_MAINNET_XPUB}"),
            "--to",
            "xprv",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.ends_with(        "error: --to xprv is cryptographically unrecoverable from --from xpub (one-way derivation barrier)\n"),
        "stderr must end with byte-exact SPEC error text; got {:?}",
        stderr,
    )
}

#[test]
fn refusal_ms1_to_mk1_sibling_pivot() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("ms1={TREZOR_12_ZERO_MS1}"),
            "--to",
            "mk1",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.ends_with(        "error: --from ms1 --to mk1 is a sibling-format pivot, not a single-format conversion. Use 'mnemonic bundle' instead.\n"),
        "stderr must end with byte-exact SPEC error text; got {:?}",
        stderr,
    )
}

#[test]
fn refusal_xpub_to_mk1_distinct_message() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={TREZOR_BIP84_MAINNET_XPUB}"),
            "--to",
            "mk1",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.ends_with(        "error: --to mk1 requires a policy descriptor binding (mk1 cards bind xpubs to specific policies via policy_id_stubs). Use 'mnemonic bundle --slot @0.xpub=... --template ...' to emit a complete bundle.\n"),
        "stderr must end with byte-exact SPEC error text; got {:?}",
        stderr,
    )
}

#[test]
fn refusal_wif_with_path_chain_code_destroyed() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("wif={SAMPLE_WIF}"),
            "--to",
            "xpub",
            "--path",
            "m/84'/0'/0'",
            "--network",
            "mainnet",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.ends_with(        "error: --from wif does not retain a chain code; --path-driven derivation is impossible.\n"),
        "stderr must end with byte-exact SPEC error text; got {:?}",
        stderr,
    )
}

#[test]
fn refusal_wif_to_entropy_one_way() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("wif={SAMPLE_WIF}"),
            "--to",
            "entropy",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.ends_with(        "error: --to entropy is cryptographically unrecoverable from --from wif (one-way derivation barrier)\n"),
        "stderr must end with byte-exact SPEC error text; got {:?}",
        stderr,
    )
}

// SPEC-A v0.6.1 — phrase/entropy → wif requires explicit --path.

const TREZOR_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

#[test]
fn refusal_phrase_to_wif_missing_path() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--to",
            "wif",
            "--network",
            "mainnet",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.ends_with(        "error: --to wif requires explicit --path; supply a BIP-32 path producing a leaf privkey (the toolkit does not auto-default a path from --template/--account).\n"),
        "stderr must end with byte-exact SPEC error text; got {:?}",
        stderr,
    )
}

#[test]
fn refusal_entropy_to_wif_missing_path() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            "entropy=00000000000000000000000000000000",
            "--to",
            "wif",
            "--network",
            "mainnet",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.ends_with(        "error: --to wif requires explicit --path; supply a BIP-32 path producing a leaf privkey (the toolkit does not auto-default a path from --template/--account).\n"),
        "stderr must end with byte-exact SPEC error text; got {:?}",
        stderr,
    )
}

#[test]
fn refusal_fingerprint_as_source() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            "fingerprint=5436d724",
            "--to",
            "xpub",
        ])
        .assert()
        .failure();
    // fingerprint is side-input-only; pre-empted before edge-classification by
    // the §5 single-from-value check (no primary value-bearing --from).
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("primary value-bearing"),
        "stderr should reject fingerprint as primary source; got: {stderr}"
    );
}
