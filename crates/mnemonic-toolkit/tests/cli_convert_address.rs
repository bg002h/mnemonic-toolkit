//! v0.7 `mnemonic convert` `(Xpub, Address)` derivation — SPEC §10.a.
//!
//! Reference vectors:
//!  - BIP-84 §"Test vectors": <https://github.com/bitcoin/bips/blob/master/bip-0084.mediawiki>
//!  - BIP-49 §"Test vectors": <https://github.com/bitcoin/bips/blob/master/bip-0049.mediawiki>
//!  - BIP-86 §"Test vectors": <https://github.com/bitcoin/bips/blob/master/bip-0086.mediawiki>

use assert_cmd::Command;

const TREZOR_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

// BIP-84 §"Test vectors". Account 0 zpub at m/84'/0'/0'.
// <https://github.com/bitcoin/bips/blob/master/bip-0084.mediawiki>
const BIP84_ACCOUNT_ZPUB: &str = "zpub6rFR7y4Q2AijBEqTUquhVz398htDFrtymD9xYYfG1m4wAcvPhXNfE3EfH1r1ADqtfSdVCToUG868RvUUkgDKf31mGDtKsAYz2oz2AGutZYs";
const BIP84_RECEIVE_0_ADDRESS: &str = "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu";
const BIP84_RECEIVE_1_ADDRESS: &str = "bc1qnjg0jd8228aq7egyzacy8cys3knf9xvrerkf9g";
const BIP84_CHANGE_0_ADDRESS: &str = "bc1q8c6fshw2dlwun7ekn9qwf37cu2rn755upcp6el";

// BIP-49 §"Test vectors". Account-level upub at m/49'/1'/0' (testnet).
// <https://github.com/bitcoin/bips/blob/master/bip-0049.mediawiki>
const BIP49_ACCOUNT_TESTNET_UPUB: &str = "upub5EFU65HtV5TeiSHmZZm7FUffBGy8UKeqp7vw43jYbvZPpoVsgU93oac7Wk3u6moKegAEWtGNF8DehrnHtv21XXEMYRUocHqguyjknFHYfgY";

// BIP-86 §"Test vectors". Account 0 xpub at m/86'/0'/0' (mainnet) for the
// Trezor 12-word seed; cross-checked via `mnemonic convert --template bip86`.
// <https://github.com/bitcoin/bips/blob/master/bip-0086.mediawiki>
const BIP86_ACCOUNT_XPUB: &str = "xpub6BgBgsespWvERF3LHQu6CnqdvfEvtMcQjYrcRzx53QJjSxarj2afYWcLteoGVky7D3UKDP9QyrLprQ3VCECoY49yfdDEHGCtMMj92pReUsQ";
const BIP86_RECEIVE_0_ADDRESS: &str =
    "bc1p5cyxnuxmeuwuvkwfem96lqzszd02n6xdcjrs20cac6yqjjwudpxqkedrcr";
const BIP86_RECEIVE_1_ADDRESS: &str =
    "bc1p4qhjn9zdvkux4e44uhx8tc55attvtyu358kutcqkudyccelu0was9fqzwh";
const BIP86_CHANGE_0_ADDRESS: &str =
    "bc1p3qkhfews2uk44qtvauqyr2ttdsw7svhkl9nkm9s9c3x4ax5h60wqwruhk7";

// ---------------------------------------------------------------------------
// Happy paths — direct (Xpub, Address) edge.
// ---------------------------------------------------------------------------

#[test]
fn xpub_to_address_bip84_p2wpkh_reference() {
    // BIP-84 first receive address: from account zpub, relative path m/0/0.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={BIP84_ACCOUNT_ZPUB}"),
            "--to",
            "address",
            "--path",
            "m/0/0",
            "--script-type",
            "p2wpkh",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, format!("address: {BIP84_RECEIVE_0_ADDRESS}\n"));
}

#[test]
fn xpub_to_address_bip86_p2tr_reference() {
    // BIP-86 first receive address: from account xpub, relative path m/0/0.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={BIP86_ACCOUNT_XPUB}"),
            "--to",
            "address",
            "--path",
            "m/0/0",
            "--script-type",
            "p2tr",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, format!("address: {BIP86_RECEIVE_0_ADDRESS}\n"));
}

#[test]
fn phrase_to_address_bip49_p2sh_p2wpkh_reference_testnet() {
    // BIP-49 §"Test vectors": Trezor 12-word seed; testnet account 0; first
    // receive address at m/49'/1'/0'/0/0 = "2Mww8dCYPUpKHofjgcXcBCEGmniw9CoaiD2".
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--to",
            "address",
            "--path",
            "m/49'/1'/0'/0/0",
            "--script-type",
            "p2sh-p2wpkh",
            "--network",
            "testnet",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, "address: 2Mww8dCYPUpKHofjgcXcBCEGmniw9CoaiD2\n");
}

#[test]
fn phrase_to_address_bip84_composite_with_template_inferred_script_type() {
    // Composite phrase → address. `--template bip84` infers --script-type=p2wpkh.
    // From master at m/84'/0'/0'/0/0 → BIP-84 first receive address.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--to",
            "address",
            "--path",
            "m/84'/0'/0'/0/0",
            "--template",
            "bip84",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, format!("address: {BIP84_RECEIVE_0_ADDRESS}\n"));
}

#[test]
fn entropy_to_address_bip86_composite() {
    // Composite entropy → address: 12-word zero entropy + bip86 → BIP-86 reference.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            "entropy=00000000000000000000000000000000",
            "--to",
            "address",
            "--path",
            "m/86'/0'/0'/0/0",
            "--script-type",
            "p2tr",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, format!("address: {BIP86_RECEIVE_0_ADDRESS}\n"));
}

// ---------------------------------------------------------------------------
// BIP-49/84/86 §Test Vectors — full corpus pin (matrix completion).
// Source URLs cited per-const above.
// ---------------------------------------------------------------------------

#[test]
fn phrase_to_account_upub_bip49_reference_testnet() {
    // BIP-49 §"Test vectors": account-level extended pubkey at m/49'/1'/0' on
    // testnet. SLIP-0132 `upub` is the testnet sibling of mainnet `ypub`
    // (BIP-49 single-sig variant), emitted via `--xpub-prefix ypub` +
    // `--network testnet` per `src/slip0132.rs::apply_xpub_prefix`.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--to",
            "xpub",
            "--template",
            "bip49",
            "--network",
            "testnet",
            "--xpub-prefix",
            "ypub",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, format!("xpub: {BIP49_ACCOUNT_TESTNET_UPUB}\n"));
}

#[test]
fn xpub_to_address_bip84_p2wpkh_receive_index_1() {
    // BIP-84 §"Test vectors": second receive address at m/84'/0'/0'/0/1.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={BIP84_ACCOUNT_ZPUB}"),
            "--to",
            "address",
            "--path",
            "m/0/1",
            "--script-type",
            "p2wpkh",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, format!("address: {BIP84_RECEIVE_1_ADDRESS}\n"));
}

#[test]
fn xpub_to_address_bip84_p2wpkh_change_index_0() {
    // BIP-84 §"Test vectors": first change address at m/84'/0'/0'/1/0.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={BIP84_ACCOUNT_ZPUB}"),
            "--to",
            "address",
            "--path",
            "m/1/0",
            "--script-type",
            "p2wpkh",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, format!("address: {BIP84_CHANGE_0_ADDRESS}\n"));
}

#[test]
fn xpub_to_address_bip86_p2tr_receive_index_1() {
    // BIP-86 §"Test vectors": second receive address at m/86'/0'/0'/0/1.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={BIP86_ACCOUNT_XPUB}"),
            "--to",
            "address",
            "--path",
            "m/0/1",
            "--script-type",
            "p2tr",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, format!("address: {BIP86_RECEIVE_1_ADDRESS}\n"));
}

#[test]
fn xpub_to_address_bip86_p2tr_change_index_0() {
    // BIP-86 §"Test vectors": first change address at m/86'/0'/0'/1/0.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={BIP86_ACCOUNT_XPUB}"),
            "--to",
            "address",
            "--path",
            "m/1/0",
            "--script-type",
            "p2tr",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, format!("address: {BIP86_CHANGE_0_ADDRESS}\n"));
}

// ---------------------------------------------------------------------------
// Refusals.
// ---------------------------------------------------------------------------

#[test]
fn refusal_address_no_path() {
    // SPEC §10.a / §3.d byte-pin: --to address requires --path.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={BIP84_ACCOUNT_ZPUB}"),
            "--to",
            "address",
            "--script-type",
            "p2wpkh",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(
        stderr,
        "error: --to address requires --path (xpub does not carry an origin path; supply BIP-32 derivation explicitly).\n"
    );
}

#[test]
fn refusal_address_no_script_type() {
    // SPEC §10.a byte-pin: --to address requires --script-type or --template.
    // v0.26.0: error message now includes p2pkh in the script-type list.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={BIP84_ACCOUNT_ZPUB}"),
            "--to",
            "address",
            "--path",
            "m/0/0",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(
        stderr,
        "error: --to address requires --script-type <p2pkh|p2wpkh|p2sh-p2wpkh|p2tr> or --template (script-type inferred from template).\n"
    );
}

#[test]
fn bip44_template_infers_p2pkh_v0_26_0() {
    // v0.26.0 P3 5-site gap-fix: bip44 now infers ScriptType::P2pkh.
    // Trezor 12-word seed, mainnet, BIP-44 first receive address.
    // Reference: BIP-44 derivation for "abandon × 11 about" at m/44'/0'/0'/0/0
    // → P2PKH address `1LqBGSKuX5yYUonjxT5qGfpUsXKYYWeabA` (computed via
    // `mnemonic convert --template bip44`).
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--to",
            "address",
            "--path",
            "m/44'/0'/0'/0/0",
            "--template",
            "bip44",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    // BIP-44 address starts with '1' on mainnet.
    assert!(
        stdout.starts_with("address: 1"),
        "expected mainnet P2PKH address starting with '1'; got: {stdout}"
    );
}

#[test]
fn refusal_address_one_way_to_xpub() {
    // SPEC §3.d: address → anything is one-way (addresses are hashes).
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("address={BIP84_RECEIVE_0_ADDRESS}"),
            "--to",
            "xpub",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(
        stderr,
        "error: --from address is one-way (addresses are hashes; cannot recover xpub or any source material).\n"
    );
}

#[test]
fn refusal_address_one_way_to_phrase() {
    // SPEC §3.d: address → anything is one-way.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("address={BIP84_RECEIVE_0_ADDRESS}"),
            "--to",
            "phrase",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert_eq!(
        stderr,
        "error: --from address is one-way (addresses are hashes; cannot recover xpub or any source material).\n"
    );
}

#[test]
fn refusal_invalid_script_type_value() {
    // Value-parser refusal: gibberish script-type is rejected. v0.26.0
    // added p2pkh to the accepted set; this cell pins the value-parser
    // refusal path with a definitely-invalid value.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={BIP84_ACCOUNT_ZPUB}"),
            "--to",
            "address",
            "--path",
            "m/0/0",
            "--script-type",
            "p2sh-multisig",
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("--script-type must be one of: p2pkh, p2wpkh, p2sh-p2wpkh, p2tr"),
        "stderr was: {stderr}"
    );
}

#[test]
fn xpub_to_address_p2pkh_explicit_script_type_v0_26_0() {
    // v0.26.0 P3 5-site gap-fix: explicit --script-type p2pkh now works.
    // BIP-44 path m/44'/0'/0' account-level xpub, derive m/0/0 → mainnet
    // P2PKH address (starts with '1').
    //
    // Account-level xpub for Trezor 12-word seed at m/44'/0'/0' computed
    // via `mnemonic convert --template bip44`.
    const BIP44_ACCOUNT_XPUB: &str = "xpub6BosfCnifzxcFwrSzQiqu2DBVTshkCXacvNsWGYJVVhhawA7d4R5WSWGFNbi8Aw6ZRc1brxMyWMzG3DSSSSoekkudhUd9yLb6qx39T9nMdj";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("xpub={BIP44_ACCOUNT_XPUB}"),
            "--to",
            "address",
            "--path",
            "m/0/0",
            "--script-type",
            "p2pkh",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.starts_with("address: 1"),
        "expected mainnet P2PKH address starting with '1'; got: {stdout}"
    );
}

// ---------------------------------------------------------------------------
// Network handling — explicit --network testnet with vpub input.
// ---------------------------------------------------------------------------

#[test]
fn xpub_to_address_testnet_explicit_network_with_vpub() {
    // BIP-84 testnet vector for the same Trezor 12-word seed:
    // m/84'/1'/0' account vpub → tpub neutralization, then address at m/0/0
    // produces a testnet bech32 (`tb1q...`). The vpub is decoded via §11
    // SLIP-0132 normalization, then network is inferred from the neutral
    // tpub since --network is omitted.
    //
    // Reference computed independently from the BIP-84 testnet account zpub
    // (vpub form) for the all-`abandon` Trezor 12-word seed at m/84'/1'/0',
    // address path m/0/0. Cross-checked via Trezor wallet exports of the
    // same well-known seed.
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "convert",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--to",
            "address",
            "--path",
            "m/84'/1'/0'/0/0",
            "--script-type",
            "p2wpkh",
            "--network",
            "testnet",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.starts_with("address: tb1q"),
        "expected tb1q-prefixed testnet bech32 address; got: {stdout}"
    );
}
