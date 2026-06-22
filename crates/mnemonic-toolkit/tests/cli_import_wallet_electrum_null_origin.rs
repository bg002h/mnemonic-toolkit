//! cycle-13 Lane C · L18 — Electrum import must NOT hard-refuse valid wallets
//! with null `root_fingerprint` / `derivation`.
//!
//! PROTOCOL FACT (verified against electrum/keystore.py, spesmilo/electrum):
//! `BIP32_KeyStore.dump()` writes `derivation = self.get_derivation_prefix()`
//! and `root_fingerprint = self.get_root_fingerprint()`, both `Optional[str]`.
//! `from_xpub()` constructs `BIP32_KeyStore({})` with an empty dict, so
//! `Xpub.__init__(derivation_prefix=None, root_fingerprint=None)` leaves both
//! `None` for the "use a master key" / restore-from-master-public-key watch-only
//! flow. Python `None` serializes to JSON `null`. The toolkit's prior
//! `.and_then(|v| v.as_str()).ok_or_else(...)` turned that null into a hard
//! `ImportWalletParse` refusal — a false-reject.
//!
//! FIX: treat null `root_fingerprint` as unknown-origin (`00000000` fp) + NOTICE;
//! treat null `derivation` by inferring the script-type from the SLIP-132 xpub
//! prefix and synthesizing the canonical origin (the toolkit's origin regex
//! requires a path component, so a bare `[fp]xpub` would not parse) + NOTICE.
//!
//! Pre-fix both fixtures hard-refused with exit 2 (empirically captured in the
//! implementing commit message); after, they import watch-only with NOTICEs.

use assert_cmd::Command;
use std::path::PathBuf;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures/wallet_import").join(name)
}

struct ImportResult {
    stdout: String,
    stderr: String,
    code: i32,
}

fn import(fixture: &str) -> ImportResult {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            fixture_path(fixture).to_str().unwrap(),
            "--format",
            "electrum",
            "--json",
        ])
        .output()
        .unwrap();
    ImportResult {
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        code: out.status.code().unwrap_or(-1),
    }
}

// ============================================================================
// L18 singlesig — null root_fingerprint + null derivation "use a master key"
// watch-only wallet imports (was hard-refused) with NOTICEs.
// ============================================================================

#[test]
fn electrum_singlesig_null_origin_imports_with_notice() {
    let res = import("electrum-standard-master-pubkey-null-origin.json");
    assert_eq!(
        res.code, 0,
        "null-origin Electrum singlesig must import (not hard-refuse); stderr={}",
        res.stderr
    );

    let env: serde_json::Value = serde_json::from_str(&res.stdout).unwrap();
    let desc = env[0]["bundle"]["descriptor"].as_str().unwrap();

    // zpub prefix → wpkh wrapper; null derivation → canonical m/84'/0'/0';
    // null fp → unknown-origin 00000000.
    assert!(
        desc.starts_with("wpkh(") && desc.contains("[00000000/84'/0'/0']"),
        "descriptor must wrap as wpkh with synthesized unknown-origin \
         [00000000/84'/0'/0']; got: {desc}"
    );

    // NOTICEs for both the null derivation and the null fingerprint.
    assert!(
        res.stderr.contains("keystore.derivation is null"),
        "expected null-derivation NOTICE; stderr={}",
        res.stderr
    );
    assert!(
        res.stderr.contains("keystore.root_fingerprint is null"),
        "expected null-root_fingerprint NOTICE; stderr={}",
        res.stderr
    );
}

// ============================================================================
// L18 multisig — a cosigner with null derivation + null root_fingerprint
// imports (was hard-refused) with NOTICEs; other cosigners keep real origins.
// ============================================================================

#[test]
fn electrum_multisig_null_cosigner_origin_imports_with_notice() {
    let res = import("electrum-multisig-2of3-wsh-null-cosigner-origin.json");
    assert_eq!(
        res.code, 0,
        "null-origin Electrum multisig cosigner must import (not hard-refuse); stderr={}",
        res.stderr
    );

    let env: serde_json::Value = serde_json::from_str(&res.stdout).unwrap();
    let desc = env[0]["bundle"]["descriptor"].as_str().unwrap();

    // 2-of-3 wsh(sortedmulti(...)); the null cosigner (x2/, Zpub → P2WSH)
    // gets the synthesized unknown-origin [00000000/48'/0'/0'/2']; the two
    // real cosigners keep their declared origins.
    assert!(
        desc.starts_with("wsh(sortedmulti(2,"),
        "descriptor must be 2-of-3 wsh(sortedmulti); got: {desc}"
    );
    assert!(
        desc.contains("[00000000/48'/0'/0'/2']"),
        "null cosigner must carry the synthesized unknown-origin \
         [00000000/48'/0'/0'/2']; got: {desc}"
    );
    assert!(
        desc.contains("[b8688df1/48'/0'/0'/2']") && desc.contains("[5436d724/48'/0'/0'/2']"),
        "real cosigners must keep their declared origins; got: {desc}"
    );

    assert!(
        res.stderr.contains("cosigner `x2/` derivation is null"),
        "expected null-derivation cosigner NOTICE; stderr={}",
        res.stderr
    );
    assert!(
        res.stderr
            .contains("cosigner `x2/` root_fingerprint is null"),
        "expected null-root_fingerprint cosigner NOTICE; stderr={}",
        res.stderr
    );
}

// ============================================================================
// Regression guard — a fully-populated (non-null) Electrum wallet still
// imports clean with NO null-origin NOTICEs.
// ============================================================================

#[test]
fn electrum_singlesig_populated_origin_no_null_notice() {
    let res = import("electrum-standard-bip84-mainnet.json");
    assert_eq!(
        res.code, 0,
        "populated wallet must import; stderr={}",
        res.stderr
    );
    assert!(
        !res.stderr.contains("is null"),
        "fully-populated wallet must NOT emit a null-origin NOTICE; stderr={}",
        res.stderr
    );
    let env: serde_json::Value = serde_json::from_str(&res.stdout).unwrap();
    let desc = env[0]["bundle"]["descriptor"].as_str().unwrap();
    assert!(
        desc.contains("[5436d724/84'/0'/0']"),
        "populated wallet keeps its real [fp/path]; got: {desc}"
    );
}
