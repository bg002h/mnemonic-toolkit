//! v0.7 Phase 5 — `mnemonic export-wallet` integration tests.
//!
//! SPEC `design/SPEC_export_wallet_v0_7.md` §9: 5 mandatory + 1 conditional cell.
//! Test vectors derive from the well-known Trezor 12-word seed
//! ("abandon ... about") + a sibling "letter advice ... above" wallet to keep
//! cosigner xpubs distinct without leaking real keys.

use assert_cmd::Command;

// Trezor 12-word seed → BIP-84 mainnet account 0.
const TREZOR_BIP84_XPUB: &str = "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V";
const TREZOR_BIP84_FP: &str = "73c5da0a";

// Two BIP-48 mainnet xpubs (path m/48'/0'/0'/2') for wsh-sortedmulti tests.
const COSIGNER_A_XPUB: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
const COSIGNER_A_FP: &str = "b8688df1";
const COSIGNER_B_XPUB: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";
const COSIGNER_B_FP: &str = "28645006";

// Trezor 12-word "abandon ... about" reference seed used by BIP-388 fixtures
// below. Master fingerprint `73c5da0a` is invariant across paths (it's the
// master xpub fingerprint, not a derived-account fingerprint). The
// account-level xpubs are derived at runtime via `mnemonic convert` from this
// phrase rather than hardcoded — see `derive_xpub_via_cli` helper.
const TREZOR_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const TREZOR_12_MASTER_FINGERPRINT: &str = "73c5da0a";

// BIP-49 mainnet account 0 xpub at m/49'/0'/0' for the Trezor 12-word seed.
// Derived by Phase-2 BIP-49 vector test path; cross-checked at runtime in
// `cell_8_*` below by re-deriving via `mnemonic convert --template bip49
// --network mainnet`. BIP-49's published vector covers testnet only, so
// this mainnet value is toolkit-derived (not spec-published) — what's
// pinned is the BIP-388 `description_template` shape, per matrix decision.
const TREZOR_12_BIP49_MAINNET_ACCOUNT_XPUB: &str = "xpub6C6nQwHaWbSrzs5tZ1q7m5R9cPK9eYpNMFesiXsYrgc1P8bvLLAet9JfHjYXKjToD8cBRswJXXbbFpXgwsswVPAZzKMa1jUp2kVkGVUaJa7";

// BIP-86 mainnet account 0 xpub at m/86'/0'/0' for the Trezor 12-word seed.
// This value IS the BIP-86 §"Test vectors" reference (cross-checked in
// `tests/cli_convert_address.rs::BIP86_ACCOUNT_XPUB`).
// <https://github.com/bitcoin/bips/blob/master/bip-0086.mediawiki>
const TREZOR_12_BIP86_MAINNET_ACCOUNT_XPUB: &str = "xpub6BgBgsespWvERF3LHQu6CnqdvfEvtMcQjYrcRzx53QJjSxarj2afYWcLteoGVky7D3UKDP9QyrLprQ3VCECoY49yfdDEHGCtMMj92pReUsQ";

/// Helper: derive `(xpub, fingerprint)` for a `(template, network)` from
/// `TREZOR_12` via `mnemonic convert`. Used to confirm the pinned constants
/// above remain in sync with the toolkit's derivation rather than drifting
/// silently.
fn derive_xpub_via_cli(template: &str, network: &str) -> (String, String) {
    let xpub_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--to",
            "xpub",
            "--template",
            template,
            "--network",
            network,
        ])
        .assert()
        .success();
    let xpub_line = String::from_utf8(xpub_out.get_output().stdout.clone()).unwrap();
    let xpub = xpub_line
        .strip_prefix("xpub: ")
        .and_then(|s| s.strip_suffix('\n'))
        .expect("`mnemonic convert --to xpub` emits `xpub: <value>\\n`")
        .to_string();

    let fp_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--to",
            "fingerprint",
            "--template",
            template,
            "--network",
            network,
        ])
        .assert()
        .success();
    let fp_line = String::from_utf8(fp_out.get_output().stdout.clone()).unwrap();
    let fp = fp_line
        .strip_prefix("fingerprint: ")
        .and_then(|s| s.strip_suffix('\n'))
        .expect("`mnemonic convert --to fingerprint` emits `fingerprint: <value>\\n`")
        .to_string();
    (xpub, fp)
}

/// SPEC §9 cell 1: Bitcoin Core importdescriptors round-trip with single-sig wpkh.
#[test]
fn cell_1_bitcoin_core_single_sig_wpkh_round_trip() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={TREZOR_BIP84_XPUB}"),
            "--slot",
            &format!("@0.fingerprint={TREZOR_BIP84_FP}"),
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let value: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let arr = value.as_array().expect("Bitcoin Core output is JSON array");
    assert_eq!(arr.len(), 2, "multipath splits into 2 entries");

    // Receive entry — checksum byte-pinned.
    assert_eq!(
        arr[0]["desc"].as_str().unwrap(),
        format!("wpkh([73c5da0a/84'/0'/0']{TREZOR_BIP84_XPUB}/0/*)#wc3n3van"),
    );
    assert!(!arr[0]["internal"].as_bool().unwrap());
    assert!(arr[0]["active"].as_bool().unwrap());
    assert_eq!(arr[0]["range"][0].as_u64().unwrap(), 0);
    assert_eq!(arr[0]["range"][1].as_u64().unwrap(), 999);
    assert_eq!(arr[0]["timestamp"].as_str().unwrap(), "now");

    // Change entry.
    assert_eq!(
        arr[1]["desc"].as_str().unwrap(),
        format!("wpkh([73c5da0a/84'/0'/0']{TREZOR_BIP84_XPUB}/1/*)#lv5jvedt"),
    );
    assert!(arr[1]["internal"].as_bool().unwrap());
}

/// SPEC §9 cell 2: BIP-388 wallet_policy round-trip with multisig wsh-sortedmulti.
#[test]
fn cell_2_bip388_wallet_policy_multisig_wsh_sortedmulti() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "bip388",
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "2",
            "--multisig-path-family",
            "bip48",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={COSIGNER_A_XPUB}"),
            "--slot",
            &format!("@0.fingerprint={COSIGNER_A_FP}"),
            "--slot",
            "@0.path=m/48'/0'/0'/2'",
            "--slot",
            &format!("@1.xpub={COSIGNER_B_XPUB}"),
            "--slot",
            &format!("@1.fingerprint={COSIGNER_B_FP}"),
            "--slot",
            "@1.path=m/48'/0'/0'/2'",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let value: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(value["name"].as_str().unwrap(), "wsh-sortedmulti");
    assert_eq!(
        value["description_template"].as_str().unwrap(),
        "wsh(sortedmulti(2,@0/**,@1/**))",
    );
    let keys = value["keys_info"].as_array().unwrap();
    assert_eq!(keys.len(), 2);
    assert_eq!(
        keys[0].as_str().unwrap(),
        format!("[{COSIGNER_A_FP}/48'/0'/0'/2']{COSIGNER_A_XPUB}"),
    );
    assert_eq!(
        keys[1].as_str().unwrap(),
        format!("[{COSIGNER_B_FP}/48'/0'/0'/2']{COSIGNER_B_XPUB}"),
    );
}

/// SPEC §9 cell 3: refusal stderr for `phrase=` slot input. Byte-exact per §3.
#[test]
fn cell_3_phrase_slot_refusal_byte_exact() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
            "@0.phrase=abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    let expected = "error: mnemonic export-wallet is watch-only by definition; supply only xpub/fingerprint/path slots. To produce an artifact that includes secret material, use 'mnemonic bundle'.\n";
    assert_eq!(stderr, expected);
}

/// SPEC §9 cell 4: Sparrow stub refusal stderr. Byte-exact per §7.
#[test]
fn cell_4_sparrow_stub_refusal_byte_exact() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "sparrow",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={TREZOR_BIP84_XPUB}"),
            "--slot",
            &format!("@0.fingerprint={TREZOR_BIP84_FP}"),
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    let expected_sparrow = "error: --format <sparrow> is deferred to v0.8 if user demand surfaces; use --format bitcoin-core or --format bip388 instead.\n";
    assert_eq!(stderr, expected_sparrow);

    // Also Specter.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "specter",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={TREZOR_BIP84_XPUB}"),
            "--slot",
            &format!("@0.fingerprint={TREZOR_BIP84_FP}"),
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    let expected_specter = "error: --format <specter> is deferred to v0.8 if user demand surfaces; use --format bitcoin-core or --format bip388 instead.\n";
    assert_eq!(stderr, expected_specter);
}

/// SPEC §9 cell 5: `--range 0,4999` override exercised in Bitcoin Core format.
#[test]
fn cell_5_range_override() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--range",
            "0,4999",
            "--slot",
            &format!("@0.xpub={TREZOR_BIP84_XPUB}"),
            "--slot",
            &format!("@0.fingerprint={TREZOR_BIP84_FP}"),
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let value: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let arr = value.as_array().unwrap();
    assert_eq!(arr[0]["range"][0].as_u64().unwrap(), 0);
    assert_eq!(arr[0]["range"][1].as_u64().unwrap(), 4999);
    assert_eq!(arr[1]["range"][0].as_u64().unwrap(), 0);
    assert_eq!(arr[1]["range"][1].as_u64().unwrap(), 4999);
}

/// SPEC §9 cell 6 (CONDITIONAL): `--bitcoin-core-version 24` shape diff vs. 25.
///
/// PER SPEC §9: "if version 24 differs from 25 materially — confirm during
/// impl; if no diff, document and reduce to a single-version test." For the
/// fields the toolkit emits (`desc` / `active` / `internal` / `range` /
/// `timestamp`), Bitcoin Core 24 and 25 are wire-identical — both versions
/// accept and require this same JSON. The `--bitcoin-core-version` flag is
/// retained for future-proofing (24 vs 25 may diverge in fields the toolkit
/// does not yet emit, e.g. `next_index`); v0.7 emits the byte-identical shape
/// for both. This cell asserts that.
#[test]
fn cell_6_bitcoin_core_version_24_matches_25_for_emitted_fields() {
    let mk_args = |ver: &str| {
        vec![
            "export-wallet".to_string(),
            "--template".into(),
            "bip84".into(),
            "--network".into(),
            "mainnet".into(),
            "--bitcoin-core-version".into(),
            ver.to_string(),
            "--slot".into(),
            format!("@0.xpub={TREZOR_BIP84_XPUB}"),
            "--slot".into(),
            format!("@0.fingerprint={TREZOR_BIP84_FP}"),
        ]
    };
    let out_24 = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(mk_args("24"))
        .assert()
        .success();
    let stdout_24 = String::from_utf8(out_24.get_output().stdout.clone()).unwrap();

    let out_25 = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(mk_args("25"))
        .assert()
        .success();
    let stdout_25 = String::from_utf8(out_25.get_output().stdout.clone()).unwrap();

    assert_eq!(
        stdout_24, stdout_25,
        "Bitcoin Core 24 and 25 emit byte-identical JSON for the toolkit's \
        importdescriptors field set (desc / active / internal / range / timestamp). \
        SPEC §9 cell 6 reduces to documentation per the conditional clause."
    );
}

/// Phase-5 post-review: `--template tr-multi-a` and `--template tr-sortedmulti-a`
/// emit a clean refusal at exit 2. Constructing `tr(<internal-key>,
/// multi_a(...))` requires picking a NUMS point or designating a key-path key;
/// deferred to v0.8.
#[test]
fn taproot_multisig_template_refusal_byte_exact() {
    for (template_name, expected) in [
        (
            "tr-multi-a",
            "error: --template <tr-multi-a> is not yet supported by 'mnemonic export-wallet' (taproot internal-key designation deferred to v0.8); use 'mnemonic bundle' for taproot multisig artifacts.\n",
        ),
        (
            "tr-sortedmulti-a",
            "error: --template <tr-sortedmulti-a> is not yet supported by 'mnemonic export-wallet' (taproot internal-key designation deferred to v0.8); use 'mnemonic bundle' for taproot multisig artifacts.\n",
        ),
    ] {
        let out = Command::cargo_bin("mnemonic")
            .unwrap()
            .args([
                "export-wallet",
                "--template",
                template_name,
                "--threshold",
                "2",
                "--multisig-path-family",
                "bip48",
                "--network",
                "mainnet",
                "--slot",
                &format!("@0.xpub={COSIGNER_A_XPUB}"),
                "--slot",
                &format!("@0.fingerprint={COSIGNER_A_FP}"),
                "--slot",
                "@0.path=m/48'/0'/0'/2'",
                "--slot",
                &format!("@1.xpub={COSIGNER_B_XPUB}"),
                "--slot",
                &format!("@1.fingerprint={COSIGNER_B_FP}"),
                "--slot",
                "@1.path=m/48'/0'/0'/2'",
            ])
            .assert()
            .failure()
            .code(2);
        let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
        assert_eq!(stderr, expected);
    }
}

// ---------------------------------------------------------------------------
// v0.7.1 Phase 4 — BIP-380 + BIP-388 §Test Vectors / §Reference Wallet Policies.
// Audit matrix: `design/agent-reports/v0_7_1-bip-test-vector-audit-matrix.md`
// §BIP-380 (380.1) + §BIP-388 (388.2 + 388.4).
// ---------------------------------------------------------------------------

/// BIP-380 §Test Vectors row 380.1 — `raw(deadbeef)#89f8spxm` is the spec's
/// only valid-checksum vector. The toolkit emits descriptors with their
/// canonical `#checksum` via `miniscript::Descriptor::Display` (see
/// `wallet_export.rs::build_descriptor_string`); pin BIP-380 conformance by
/// (a) verifying the spec's `raw(deadbeef)#89f8spxm` passes
/// `miniscript::descriptor::checksum::verify_checksum` (the same checksum
/// algorithm the toolkit's Display impl uses), and (b) verifying the
/// toolkit's Phase-5 wpkh export-wallet emits a well-formed
/// `#<8-char-bech32-checksum>` suffix that round-trips through both
/// `verify_checksum` and a full `Descriptor` parse + re-serialize.
///
/// Note: `miniscript::Descriptor` (the typed enum) does not include the
/// `raw(<hex>)` form (a Bitcoin-Core-only extension for arbitrary
/// scriptPubKey), so the spec vector is exercised against
/// `verify_checksum` directly — which is the layer of miniscript that
/// implements BIP-380's checksum spec.
///
/// <https://github.com/bitcoin/bips/blob/master/bip-0380.mediawiki>
#[test]
fn bip380_valid_checksum_round_trip_via_miniscript() {
    use miniscript::descriptor::checksum::verify_checksum;
    use miniscript::{Descriptor, DescriptorPublicKey};
    use std::str::FromStr;

    // BIP-380 §Test Vectors 380.1: `raw(deadbeef)#89f8spxm` is valid.
    let spec_vector = "raw(deadbeef)#89f8spxm";
    let stripped = verify_checksum(spec_vector)
        .expect("BIP-380 §Test Vectors 380.1 must pass miniscript verify_checksum");
    assert_eq!(
        stripped, "raw(deadbeef)",
        "verify_checksum must return the body without the `#checksum` suffix"
    );

    // Toolkit-side end-to-end: emit a wpkh export-wallet, extract the
    // emitted descriptor, assert it ends in `#<8 chars>` and that re-parsing
    // it yields the same canonical form. Reuses
    // `cell_1_bitcoin_core_single_sig_wpkh_round_trip` inputs.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={TREZOR_BIP84_XPUB}"),
            "--slot",
            &format!("@0.fingerprint={TREZOR_BIP84_FP}"),
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let value: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let arr = value.as_array().unwrap();
    let desc = arr[0]["desc"].as_str().unwrap();

    // BIP-380 §Checksum: `#` separator + exactly 8 bech32 chars.
    let (_body, checksum) = desc
        .rsplit_once('#')
        .expect("emitted descriptor must carry a `#checksum` suffix per BIP-380");
    assert_eq!(
        checksum.len(),
        8,
        "BIP-380 mandates 8-character checksum; got {checksum:?} (len {})",
        checksum.len()
    );
    assert!(
        checksum.chars().all(|c| "qpzry9x8gf2tvdw0s3jn54khce6mua7l".contains(c)),
        "BIP-380 checksum must be bech32 charset; got {checksum:?}"
    );

    // BIP-380 checksum-spec conformance: verify_checksum accepts the
    // toolkit-emitted descriptor (same algorithm the spec mandates).
    verify_checksum(desc).expect("toolkit-emitted descriptor must pass verify_checksum");

    // Round-trip property: re-parse + re-display must yield the canonical form.
    let reparsed = Descriptor::<DescriptorPublicKey>::from_str(desc)
        .expect("toolkit-emitted descriptor must parse via miniscript");
    assert_eq!(
        reparsed.to_string(),
        desc,
        "toolkit-emitted descriptor must round-trip byte-identical through \
         miniscript (BIP-380 checksum end-to-end conformance)"
    );
}

/// BIP-388 §Reference Wallet Policies row 388.2 — `sh(wpkh(@0/**))` (BIP-49
/// nested segwit). Pin the `description_template` shape from
/// `mnemonic export-wallet --format bip388 --template bip49` and the
/// `keys_info[0]` `[fingerprint/49h/0h/0h]xpub` shape. The xpub is
/// toolkit-derived from the Trezor 12-word "abandon ... about" reference
/// seed (BIP-388's 388.2 spec value uses an unspecified seed, so the spec's
/// concrete xpub is not byte-pinnable; the matrix records this as
/// COVERED-TEMPLATE-SHAPE-ONLY).
///
/// <https://github.com/bitcoin/bips/blob/master/bip-0388.mediawiki>
#[test]
fn cell_8_bip388_sh_wpkh_bip49_template_shape() {
    // Confirm the pinned constant matches the toolkit's runtime derivation
    // (guards against silent drift).
    let (derived_xpub, derived_fp) = derive_xpub_via_cli("bip49", "mainnet");
    assert_eq!(
        derived_xpub, TREZOR_12_BIP49_MAINNET_ACCOUNT_XPUB,
        "TREZOR_12_BIP49_MAINNET_ACCOUNT_XPUB drifted from runtime derivation"
    );
    assert_eq!(
        derived_fp, TREZOR_12_MASTER_FINGERPRINT,
        "TREZOR_12_MASTER_FINGERPRINT drifted from runtime derivation"
    );

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "bip388",
            "--template",
            "bip49",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={TREZOR_12_BIP49_MAINNET_ACCOUNT_XPUB}"),
            "--slot",
            &format!("@0.fingerprint={TREZOR_12_MASTER_FINGERPRINT}"),
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let value: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(value["name"].as_str().unwrap(), "bip49");
    // BIP-388 §Reference Wallet Policies 388.2 template shape.
    assert_eq!(
        value["description_template"].as_str().unwrap(),
        "sh(wpkh(@0/**))",
    );
    let keys = value["keys_info"].as_array().unwrap();
    assert_eq!(keys.len(), 1);
    assert_eq!(
        keys[0].as_str().unwrap(),
        format!(
            "[{TREZOR_12_MASTER_FINGERPRINT}/49'/0'/0']{TREZOR_12_BIP49_MAINNET_ACCOUNT_XPUB}"
        ),
    );
}

/// BIP-388 §Reference Wallet Policies row 388.4 — `tr(@0/**)` (BIP-86
/// taproot). Same pattern as 388.2 but for the taproot single-key template.
/// The xpub here IS the BIP-86 §"Test vectors" reference value (the Trezor
/// 12-word seed produces the BIP-86 spec xpub at m/86'/0'/0').
///
/// <https://github.com/bitcoin/bips/blob/master/bip-0388.mediawiki>
#[test]
fn cell_9_bip388_tr_bip86_template_shape() {
    let (derived_xpub, derived_fp) = derive_xpub_via_cli("bip86", "mainnet");
    assert_eq!(
        derived_xpub, TREZOR_12_BIP86_MAINNET_ACCOUNT_XPUB,
        "TREZOR_12_BIP86_MAINNET_ACCOUNT_XPUB drifted from runtime derivation"
    );
    assert_eq!(
        derived_fp, TREZOR_12_MASTER_FINGERPRINT,
        "TREZOR_12_MASTER_FINGERPRINT drifted from runtime derivation"
    );

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "bip388",
            "--template",
            "bip86",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={TREZOR_12_BIP86_MAINNET_ACCOUNT_XPUB}"),
            "--slot",
            &format!("@0.fingerprint={TREZOR_12_MASTER_FINGERPRINT}"),
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let value: serde_json::Value = serde_json::from_str(&stdout).unwrap();

    assert_eq!(value["name"].as_str().unwrap(), "bip86");
    // BIP-388 §Reference Wallet Policies 388.4 template shape.
    assert_eq!(
        value["description_template"].as_str().unwrap(),
        "tr(@0/**)",
    );
    let keys = value["keys_info"].as_array().unwrap();
    assert_eq!(keys.len(), 1);
    assert_eq!(
        keys[0].as_str().unwrap(),
        format!(
            "[{TREZOR_12_MASTER_FINGERPRINT}/86'/0'/0']{TREZOR_12_BIP86_MAINNET_ACCOUNT_XPUB}"
        ),
    );
}

/// Phase-5 post-review: `--threshold N > cosigner_count` returns a clean
/// refusal at exit 1 (BadInput) rather than a miniscript parse error.
#[test]
fn threshold_greater_than_cosigner_count_refusal() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--template",
            "wsh-sortedmulti",
            "--threshold",
            "5",
            "--multisig-path-family",
            "bip48",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={COSIGNER_A_XPUB}"),
            "--slot",
            &format!("@0.fingerprint={COSIGNER_A_FP}"),
            "--slot",
            "@0.path=m/48'/0'/0'/2'",
            "--slot",
            &format!("@1.xpub={COSIGNER_B_XPUB}"),
            "--slot",
            &format!("@1.fingerprint={COSIGNER_B_FP}"),
            "--slot",
            "@1.path=m/48'/0'/0'/2'",
        ])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("--threshold 5 exceeds cosigner count 2"),
        "stderr did not contain k>n refusal: {stderr:?}",
    );
}
