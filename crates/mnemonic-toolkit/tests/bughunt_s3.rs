//! Bug-hunt triage rows, tk slice S3 (wallet import / export).
//!
//! One test per reported bug (report `design/agent-reports/bughunt-2026-10-31-tk.md`
//! numbers 16-23, 29-31, 49, 57, 58). Every test asserts the CORRECT behaviour,
//! so it FAILS on unmodified code when the bug is real. Tests drive the built
//! `mnemonic` binary the way the other `tests/cli_*.rs` files do.

use assert_cmd::Command;
use bitcoin::hashes::Hash;
use bitcoin::secp256k1::{Message, PublicKey, Secp256k1, SecretKey};
use bitcoin::sign_message::{signed_msg_hash, MessageSignature};
use miniscript::descriptor::checksum::Engine as ChecksumEngine;
use serde_json::Value;

// ---- shared fixtures (same xpubs the other CLI tests use) ----

const XPUB_A: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
const FP_A: &str = "b8688df1";
const XPUB_B: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";
const FP_B: &str = "28645006";
const XPUB_C: &str = "xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx";
const FP_C: &str = "5436d724";
const NUMS: &str = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

/// Legacy-firmware Coldcard account-level xpub (depth > 0), as in
/// `tests/fixtures/wallet_import/coldcard-mk1-legacy-bip44-mainnet.json`.
const CC_LEGACY_XPUB: &str = "xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj";

fn checksum(body: &str) -> String {
    let mut eng = ChecksumEngine::new();
    eng.input(body).expect("ascii descriptor body");
    eng.checksum()
}

fn mnemonic() -> Command {
    Command::cargo_bin("mnemonic").unwrap()
}

/// Run `mnemonic <args>` with optional stdin; return (exit code, stdout, stderr).
fn run(args: &[&str], stdin: Option<&str>) -> (i32, String, String) {
    let mut cmd = mnemonic();
    cmd.args(args);
    if let Some(s) = stdin {
        cmd.write_stdin(s.to_string());
    }
    let out = cmd.output().expect("spawn mnemonic");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// Build a Bitcoin Core `listdescriptors` blob; each entry is
/// (descriptor body without checksum, internal flag).
fn core_blob(entries: &[(&str, bool)]) -> String {
    let descs: Vec<Value> = entries
        .iter()
        .map(|(d, internal)| {
            serde_json::json!({
                "desc": format!("{d}#{}", checksum(d)),
                "active": true,
                "internal": internal,
                "range": [0, 999],
            })
        })
        .collect();
    serde_json::json!({"wallet_name": "w", "descriptors": descs}).to_string()
}

/// `import-wallet --json` over stdin; returns the first envelope's
/// `bundle.multisig.threshold` (panics with the raw output on failure).
fn import_json_threshold(format: &str, blob: &str) -> u64 {
    let (code, stdout, stderr) = run(
        &["import-wallet", "--blob", "-", "--format", format, "--json"],
        Some(blob),
    );
    assert_eq!(
        code, 0,
        "import-wallet must succeed\nstdout: {stdout}\nstderr: {stderr}"
    );
    let v: Value = serde_json::from_str(&stdout).expect("--json stdout is JSON");
    v[0]["bundle"]["multisig"]["threshold"]
        .as_u64()
        .unwrap_or_else(|| panic!("no bundle.multisig.threshold in {stdout}"))
}

// ===========================================================================
// 16 / 17: taproot multisig threshold is lost on Bitcoin Core / Sparrow import
// ===========================================================================

/// #16 - `tr(NUMS, sortedmulti_a(2, A, B, C))` / `multi_a(2, ...)` imported
/// from a Bitcoin Core blob must keep K = 2 (it is reported as 3, the N-of-N
/// default, because `extract_threshold` returns None for the `_a` fragments).
#[test]
fn bughunt_tk_16_bitcoin_core_import_taproot_multisig_threshold() {
    let mut wrong = Vec::new();
    for frag in ["sortedmulti_a", "multi_a"] {
        let desc = format!(
            "tr({NUMS},{frag}(2,[{FP_A}/87'/0'/0']{XPUB_A}/<0;1>/*,[{FP_B}/87'/0'/0']{XPUB_B}/<0;1>/*,[{FP_C}/87'/0'/0']{XPUB_C}/<0;1>/*))"
        );
        let k = import_json_threshold("bitcoin-core", &core_blob(&[(&desc, false)]));
        if k != 2 {
            wrong.push(format!("{frag}: bundle.multisig.threshold = {k}"));
        }
    }
    assert!(
        wrong.is_empty(),
        "2-of-3 taproot multisig must import with threshold 2; got {wrong:?}"
    );
}

/// #17 - same bug in the Sparrow importer, using the repo's own 2-of-3 taproot
/// fixture (`multi_a`) and a `sortedmulti_a` rewrite of it.
#[test]
fn bughunt_tk_17_sparrow_import_taproot_multisig_threshold() {
    let fixture =
        std::fs::read_to_string("tests/fixtures/wallet_import/sparrow-tr-multi-a-nums-2of3.json")
            .expect("fixture");
    assert!(
        fixture.contains("multi_a(2,"),
        "fixture is a 2-of-3 multi_a"
    );
    let mut wrong = Vec::new();
    for (name, blob) in [
        ("multi_a", fixture.clone()),
        (
            "sortedmulti_a",
            fixture.replace("multi_a(2,", "sortedmulti_a(2,"),
        ),
    ] {
        let k = import_json_threshold("sparrow", &blob);
        if k != 2 {
            wrong.push(format!("{name}: bundle.multisig.threshold = {k}"));
        }
    }
    assert!(
        wrong.is_empty(),
        "2-of-3 taproot multisig must import with threshold 2; got {wrong:?}"
    );
}

// ===========================================================================
// 18: BSMS Round-1 signature is checked against a rebuilt line 3
// ===========================================================================

/// Build a signed Round-1 record whose line 3 is exactly `line3`, signed over
/// the first four lines as written (BIP-129 line 81: "to sign the first four
/// lines"). The key signing is the raw pubkey on line 3.
fn signed_round1(line3_prefix: &str, sk: &SecretKey) -> String {
    let secp = Secp256k1::new();
    let pk = PublicKey::from_secret_key(&secp, sk);
    let line3 = format!("{line3_prefix}{}", hex::encode(pk.serialize()));
    let body = format!("BSMS 1.0\n00\n{line3}\nSigner 1 key");
    let digest = signed_msg_hash(&body);
    let rsig = secp.sign_ecdsa_recoverable(&Message::from_digest(digest.to_byte_array()), sk);
    let sig = MessageSignature::new(rsig, true).to_base64();
    format!("{body}\n{sig}\n")
}

fn round1_verified(record: &str) -> (bool, String) {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(tmp.path(), record).unwrap();
    let (code, stdout, stderr) = run(
        &[
            "import-wallet",
            "--bsms-round1",
            tmp.path().to_str().unwrap(),
            "--bsms-verify-strict",
            "--json",
        ],
        None,
    );
    (
        code == 0 && stdout.contains("\"signature_verified\":true"),
        format!("exit {code}; stderr: {stderr}"),
    )
}

/// #18 - a record whose line 3 is not byte-identical to the toolkit's
/// canonical rebuild (uppercase fingerprint, `h` hardened marker, empty path
/// `[fp/]`) is still correctly signed by its signer and must verify. The
/// canonical-form control must pass or the harness itself is broken.
#[test]
fn bughunt_tk_18_bsms_round1_signature_over_line3_as_written() {
    let sk = SecretKey::from_slice(&[0x42u8; 32]).unwrap();
    let (control_ok, why) = round1_verified(&signed_round1("[59865f44/48'/0'/0'/2']", &sk));
    assert!(control_ok, "control (canonical line 3) must verify: {why}");

    let mut failing = Vec::new();
    for (name, prefix) in [
        ("uppercase fingerprint", "[59865F44/48'/0'/0'/2']"),
        ("h hardened marker", "[59865f44/48h/0h/0h/2h]"),
        ("empty path [fp/]", "[59865f44/]"),
    ] {
        let (ok, why) = round1_verified(&signed_round1(prefix, &sk));
        if !ok {
            failing.push(format!("{name} -> {why}"));
        }
    }
    assert!(
        failing.is_empty(),
        "validly signed Round-1 records rejected: {failing:#?}"
    );
}

// ===========================================================================
// 19: Coldcard multisig import of a legacy BIP-45 P2SH export
// ===========================================================================

/// #19 - Coldcard's legacy P2SH multisig export uses `Derivation: m/45'` (no
/// coin-type component). It must import (network from the xpub version).
#[test]
fn bughunt_tk_19_coldcard_multisig_import_bip45_p2sh() {
    let blob = format!(
        "Name: LegacyMs\nPolicy: 2 of 3\nDerivation: m/45'\nFormat: P2SH\n\n\
         34A3A4F1: {XPUB_A}\nFF9DFBCF: {XPUB_B}\nB7F7DFEA: {XPUB_C}\n"
    );
    let (code, stdout, stderr) = run(
        &[
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "coldcard-multisig",
        ],
        Some(&blob),
    );
    assert_eq!(
        code, 0,
        "legacy m/45' P2SH export must import\nstdout: {stdout}\nstderr: {stderr}"
    );
    assert!(stdout.contains("threshold=2"), "stdout: {stdout}");
}

// ===========================================================================
// 20: Sparrow export gives every keystore the same label
// ===========================================================================

/// #20 - Sparrow's drongo `Wallet.checkWallet()` throws "Wallet keystores
/// have duplicate labels"; the export must give each keystore its own label.
#[test]
fn bughunt_tk_20_sparrow_export_keystore_labels_unique() {
    let (code, stdout, stderr) = run(
        &[
            "export-wallet",
            "--format",
            "sparrow",
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--multisig-path-family",
            "bip48",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={XPUB_A}"),
            "--slot",
            &format!("@0.fingerprint={FP_A}"),
            "--slot",
            "@0.path=m/48'/0'/0'/2'",
            "--slot",
            &format!("@1.xpub={XPUB_B}"),
            "--slot",
            &format!("@1.fingerprint={FP_B}"),
            "--slot",
            "@1.path=m/48'/0'/0'/2'",
            "--slot",
            &format!("@2.xpub={XPUB_C}"),
            "--slot",
            &format!("@2.fingerprint={FP_C}"),
            "--slot",
            "@2.path=m/48'/0'/0'/2'",
            "--output",
            "-",
        ],
        None,
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    let v: Value = serde_json::from_str(&stdout).expect("sparrow JSON");
    let labels: Vec<String> = v["keystores"]
        .as_array()
        .expect("keystores")
        .iter()
        .map(|k| k["label"].as_str().unwrap().to_string())
        .collect();
    let mut uniq = labels.clone();
    uniq.sort();
    uniq.dedup();
    assert_eq!(labels.len(), 3);
    assert_eq!(
        uniq.len(),
        labels.len(),
        "keystore labels must be pairwise distinct, got {labels:?}"
    );
}

// ===========================================================================
// 21: BIP-388 export of a taproot NUMS multisig from --descriptor
// ===========================================================================

fn nums_tr_template_args<'a>(format: &'a str, slots: &'a [String]) -> Vec<&'a str> {
    let mut a = vec![
        "export-wallet",
        "--format",
        format,
        "--template",
        "tr-sortedmulti-a",
        "--taproot-internal-key",
        "nums",
        "--threshold",
        "2",
        "--multisig-path-family",
        "bip48",
        "--network",
        "mainnet",
        "--output",
        "-",
    ];
    for s in slots {
        a.push("--slot");
        a.push(s);
    }
    a
}

/// #21 - the toolkit can emit a BIP-388 policy for its own NUMS taproot
/// multisig from `--template`; feeding the SAME wallet back as `--descriptor`
/// must produce the same policy, not refuse it.
#[test]
fn bughunt_tk_21_bip388_export_from_descriptor_taproot_nums_multisig() {
    let slots: Vec<String> = [(0, XPUB_A, FP_A), (1, XPUB_B, FP_B), (2, XPUB_C, FP_C)]
        .iter()
        .flat_map(|(i, x, f)| {
            vec![
                format!("@{i}.xpub={x}"),
                format!("@{i}.fingerprint={f}"),
                format!("@{i}.path=m/48'/0'/0'/2'"),
            ]
        })
        .collect();
    let (c1, via_template, e1) = run(&nums_tr_template_args("bip388", &slots), None);
    assert_eq!(c1, 0, "template path must work (control): {e1}");
    let (c2, descriptor, e2) = run(&nums_tr_template_args("descriptor", &slots), None);
    assert_eq!(c2, 0, "descriptor emit must work (control): {e2}");
    let descriptor = descriptor.trim();
    assert!(
        descriptor.starts_with(&format!("tr({NUMS},sortedmulti_a(")),
        "{descriptor}"
    );

    let (c3, via_descriptor, e3) = run(
        &[
            "export-wallet",
            "--format",
            "bip388",
            "--descriptor",
            descriptor,
            "--network",
            "mainnet",
            "--output",
            "-",
        ],
        None,
    );
    assert_eq!(
        c3, 0,
        "bip388 from the toolkit's own NUMS descriptor must not be refused\nstderr: {e3}"
    );
    let a: Value = serde_json::from_str(&via_template).unwrap();
    let b: Value = serde_json::from_str(&via_descriptor).unwrap();
    assert_eq!(a["description_template"], b["description_template"]);
    assert_eq!(a["keys_info"], b["keys_info"]);
}

// ===========================================================================
// 22: Specter export with --descriptor lists one device
// ===========================================================================

/// #22 - `devices` is documented as "length = cosigner count for multisig";
/// a `--descriptor` 2-of-3 must list 3, not 1.
#[test]
fn bughunt_tk_22_specter_export_descriptor_mode_device_count() {
    let desc = format!(
        "wsh(sortedmulti(2,[{FP_A}/48'/0'/0'/2']{XPUB_A}/<0;1>/*,[{FP_B}/48'/0'/0'/2']{XPUB_B}/<0;1>/*,[{FP_C}/48'/0'/0'/2']{XPUB_C}/<0;1>/*))"
    );
    let (code, stdout, stderr) = run(
        &[
            "export-wallet",
            "--format",
            "specter",
            "--descriptor",
            &desc,
            "--wallet-name",
            "w",
            "--network",
            "mainnet",
            "--output",
            "-",
        ],
        None,
    );
    assert_eq!(code, 0, "stderr: {stderr}");
    let v: Value = serde_json::from_str(&stdout).expect("specter JSON");
    let n = v["devices"].as_array().expect("devices").len();
    assert_eq!(n, 3, "2-of-3 wallet must list 3 devices, got {n}: {stdout}");
}

// ===========================================================================
// 23: Electrum import mangles wallet names that end in digits
// ===========================================================================

/// #23 - only the `-<N>` suffix the exporter adds may be stripped. Control:
/// `Vault-2` -> `Vault` (what the exporter writes) must keep working.
#[test]
fn bughunt_tk_23_electrum_import_wallet_name_digits_preserved() {
    let base =
        std::fs::read_to_string("tests/fixtures/wallet_import/electrum-multisig-2of3-wsh.json")
            .expect("fixture");
    let name_for = |label: &str| -> Value {
        let blob = base.replace("VaultCold-1", label);
        let (code, stdout, stderr) = run(
            &[
                "import-wallet",
                "--blob",
                "-",
                "--format",
                "electrum",
                "--json",
            ],
            Some(&blob),
        );
        assert_eq!(code, 0, "stderr: {stderr}");
        let v: Value = serde_json::from_str(&stdout).unwrap();
        v[0]["electrum_source_metadata"]["wallet_name"].clone()
    };
    assert_eq!(name_for("Vault-2"), Value::from("Vault"), "control");
    let mut wrong = Vec::new();
    for label in ["Savings 2024", "Vault2", "2024"] {
        let got = name_for(label);
        if got.as_str() != Some(label) {
            wrong.push(format!("{label:?} -> {got}"));
        }
    }
    assert!(
        wrong.is_empty(),
        "wallet names must not lose their own digits: {wrong:?}"
    );
}

// ===========================================================================
// 29 / 30 / 31 / 49: Coldcard single-sig import
// ===========================================================================

fn coldcard_import(blob: &str) -> (i32, String, String) {
    run(
        &[
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "coldcard",
            "--json",
        ],
        Some(blob),
    )
}

/// #29 - a present-but-non-integer `account` must never be silently rewritten
/// to 0 (origin `m/44'/0'/0'`). Refusing is correct; accepting the string "5"
/// as 5 would also be acceptable (open product question), accepting it as 0 is
/// not.
#[test]
fn bughunt_tk_29_coldcard_import_non_integer_account_not_rewritten_to_zero() {
    let mut wrong = Vec::new();
    for account in ["\"5\"", "-1", "1.5"] {
        let blob = format!(
            r#"{{"xpub":"{CC_LEGACY_XPUB}","xfp":"5436D724","chain":"BTC","account":{account}}}"#
        );
        let (code, stdout, _e) = coldcard_import(&blob);
        if code == 0 && stdout.contains("44'/0'/0']") {
            wrong.push(format!("account {account} accepted as account 0"));
        }
    }
    assert!(wrong.is_empty(), "{wrong:?}");
}

/// #30 - a multibyte `xfp` of byte length 8 must be a clean parse error
/// (exit 2), not a panic (exit 101) on a non-char-boundary slice.
#[test]
fn bughunt_tk_30_coldcard_import_multibyte_xfp_is_error_not_panic() {
    let blob = format!(r#"{{"xpub":"{CC_LEGACY_XPUB}","xfp":"aéééb","chain":"BTC"}}"#);
    let (code, _o, stderr) = coldcard_import(&blob);
    assert!(
        !stderr.contains("panicked") && code == 2,
        "expected clean exit 2, got exit {code}; stderr: {stderr}"
    );
}

/// #31 - `+` is not a hex digit; `xfp` "+a+b+c+d" must be refused, not read
/// as 0a0b0c0d (`u8::from_str_radix` accepts a leading `+`).
#[test]
fn bughunt_tk_31_coldcard_import_xfp_plus_signs_refused() {
    let blob = format!(r#"{{"xpub":"{CC_LEGACY_XPUB}","xfp":"+a+b+c+d","chain":"BTC"}}"#);
    let (code, stdout, _e) = coldcard_import(&blob);
    assert_ne!(
        code,
        0,
        "xfp with '+' accepted; stdout: {}",
        &stdout[..stdout.len().min(300)]
    );
}

/// #49 - Coldcard's top-level `xpub` is the MASTER xpub (firmware
/// docs/generic-wallet-export.md: "It contains master XPUB, XFP for that").
/// A blob with only `bip48_*` blocks must not have that master xpub emitted
/// as the account xpub of a `pkh([fp/44'/0'/N'] ...)` wallet.
#[test]
fn bughunt_tk_49_coldcard_import_master_xpub_not_used_as_account_xpub() {
    const MASTER: &str = "xpub661MyMwAqRbcGC9DmWbtbAmuUjpMYxw4BWE88NSDHB3jSjfUK7KtYJuKa52GbowD3DVLkgsxH9QwPnTx5mjdHykYFEncnmAsNsCTbWzBhA7";
    let blob = format!(
        r#"{{"chain":"BTC","xfp":"0F056943","account":123,"xpub":"{MASTER}",
            "bip48_2":{{"name":"p2wsh","xfp":"2A01C6B0","deriv":"m/48h/0h/123h/2h",
            "xpub":"xpub6EkcQSTygvxVneXmk3ywiS2PFhBdiPxeMxYf6RFxHCHH36NxdcN7DjUpudCppAAxs58CG6DQLjtqZNmyC3MpgVob6wpdeATjpZZ1woX92EF"}}}}"#
    );
    let (code, stdout, stderr) = coldcard_import(&blob);
    assert!(
        !(code == 0 && stdout.contains(MASTER)),
        "master xpub emitted as an account xpub (exit {code}):\n{}\nstderr: {stderr}",
        stdout
            .lines()
            .find(|l| l.contains("\"descriptor\""))
            .unwrap_or("")
    );
}

// ===========================================================================
// 57 / 58: Bitcoin Core receive/change merge
// ===========================================================================

/// #57 - a receive entry `wsh(pk(A/0/*))` and a change entry `wsh(pkh(A/1/*))`
/// differ in script structure, so they are not "the same descriptor except the
/// last step" and must not be merged into one `wsh(pk(A/<0;1>/*))` that drops
/// the change script. (Refusing, or importing them separately, is correct.)
#[test]
fn bughunt_tk_57_bitcoin_core_merge_requires_same_script() {
    let o = format!("[{FP_A}/84'/0'/0']{XPUB_A}");
    let blob = core_blob(&[
        (&format!("wsh(pk({o}/0/*))"), false),
        (&format!("wsh(pkh({o}/1/*))"), true),
    ]);
    let (code, stdout, stderr) = run(
        &[
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "bitcoin-core",
            "--json",
        ],
        Some(&blob),
    );
    let merged = code == 0 && stdout.contains("<0;1>");
    assert!(
        !merged,
        "receive wsh(pk()) and change wsh(pkh()) were merged into one multipath descriptor\nstdout: {}\nstderr: {stderr}",
        stdout
            .lines()
            .find(|l| l.contains("\"descriptor\""))
            .unwrap_or("")
    );
}

/// #58 - after a receive/change merge shrinks the list, a later bad entry
/// must be reported under its ORIGINAL index (`descriptors[2]`).
#[test]
fn bughunt_tk_58_bitcoin_core_error_names_original_descriptor_index() {
    let recv = format!("wpkh([{FP_A}/84'/0'/0']{XPUB_A}/0/*)");
    let chg = format!("wpkh([{FP_A}/84'/0'/0']{XPUB_A}/1/*)");
    let other = format!("wpkh([{FP_B}/84'/0'/0']{XPUB_B}/<0;1>/*)");
    let mut blob: Value =
        serde_json::from_str(&core_blob(&[(&recv, false), (&chg, true), (&other, false)])).unwrap();
    // Corrupt only the THIRD entry's checksum.
    let d2 = blob["descriptors"][2]["desc"].as_str().unwrap().to_string();
    let (body, _cs) = d2.rsplit_once('#').unwrap();
    blob["descriptors"][2]["desc"] = Value::from(format!("{body}#aaaaaaaa"));

    let (code, _o, stderr) = run(
        &["import-wallet", "--blob", "-", "--format", "bitcoin-core"],
        Some(&blob.to_string()),
    );
    assert_eq!(code, 2, "stderr: {stderr}");
    assert!(
        stderr.contains("descriptors[2]") && !stderr.contains("descriptors[1]"),
        "error must name descriptors[2] (the corrupted entry): {stderr}"
    );
}
