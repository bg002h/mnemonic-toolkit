//! T4 + T9 (SPEC_path_raw_bracketed_bare_unification.md §8) — origin-path
//! rendering on `bundle --json` after the `ResolvedSlot.path_raw` deletion.
//!
//! - T4: the pathless `--slot @N.wif=` slot emits `origin_path: null` (the
//!   default-path → `""` → `None` sentinel, formerly `path_raw.is_empty()`).
//! - T9: descriptor/template-mode `--slot @N.path=<non-canonical>` is rendered
//!   CANONICAL (`48h` → `48'`) in `bundle --json` `origin_path` (Amendment A3 —
//!   the intentional wire-value change from deriving the origin from the typed
//!   `DerivationPath` instead of the raw user string).

use assert_cmd::Command;
use serde_json::Value;

const SAMPLE_WIF: &str = "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn";
// An arbitrary valid mainnet xpub (reused from the fixture corpus); content is
// irrelevant — only the slot's `--slot @N.path=` value drives this assertion.
const SAMPLE_XPUB: &str = "xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9";

fn bundle_json(args: &[&str]) -> Value {
    let out = Command::cargo_bin("mnemonic").unwrap().args(args).assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("bundle JSON invalid: {e}\nstdout:\n{stdout}"))
}

// T4 — pathless WIF slot → origin_path: null (default-path sentinel).
#[test]
fn bundle_wif_slot_origin_path_is_null() {
    let v = bundle_json(&[
        "bundle",
        "--template",
        "bip84",
        "--network",
        "mainnet",
        "--slot",
        &format!("@0.wif={SAMPLE_WIF}"),
        "--json",
        "--no-engraving-card",
    ]);
    assert!(
        v["origin_path"].is_null(),
        "pathless WIF slot must emit origin_path: null; got {:?}",
        v["origin_path"]
    );
}

// T9 — descriptor-mode non-canonical `--slot @N.path=48h/...` → canonical
// origin_path in `bundle --json` (A3).
#[test]
fn bundle_descriptor_mode_noncanonical_path_renders_canonical() {
    let v = bundle_json(&[
        "bundle",
        "--slot",
        &format!("@0.xpub={SAMPLE_XPUB}"),
        "--slot",
        "@0.fingerprint=deadbeef",
        "--slot",
        "@0.path=48h/0h/0h/2h",
        "--template",
        "bip84",
        "--network",
        "mainnet",
        "--json",
        "--no-engraving-card",
    ]);
    assert_eq!(
        v["origin_path"].as_str(),
        Some("m/48'/0'/0'/2'"),
        "non-canonical @N.path=48h must canonicalize to '-notation (A3); got {:?}",
        v["origin_path"]
    );
}

// T10 (Amendment A4) — `export-wallet` with a non-canonical `--slot @N.path=`
// renders the origin CANONICAL (`84h` → `84'`) in the emitted wallet file,
// since emitters now derive the origin from the typed DerivationPath rather
// than the raw user string. Exercised via the electrum `derivation` field
// (the consumer that previously echoed `path_raw` verbatim).
#[test]
fn export_wallet_noncanonical_path_renders_canonical() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "electrum",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={SAMPLE_XPUB}"),
            "--slot",
            "@0.fingerprint=deadbeef",
            "--slot",
            "@0.path=84h/0h/0h",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&stdout).expect("electrum JSON");
    assert_eq!(
        v["keystore"]["derivation"].as_str(),
        Some("m/84'/0'/0'"),
        "non-canonical @N.path=84h must canonicalize in export output (A4); got {:?}",
        v["keystore"]["derivation"]
    );
}

// T6 (Amendment A2) — `check_resolved_slots_distinctness` now compares the
// TYPED DerivationPath (h folds to '), converging with the descriptor-mode
// twin `check_key_vector_distinctness`. Two cosigners with the same xpub and
// paths differing only in `h`-vs-`'` notation must COLLIDE (BIP-388, exit 2);
// pre-A2 the raw-string compare would have let them through.
#[test]
fn bundle_distinctness_h_vs_apostrophe_paths_collide() {
    let assert = Command::cargo_bin("mnemonic")
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
            &format!("@0.xpub={SAMPLE_XPUB}"),
            "--slot",
            "@0.fingerprint=deadbeef",
            "--slot",
            "@0.path=48h/0h/0h/2h",
            "--slot",
            &format!("@1.xpub={SAMPLE_XPUB}"),
            "--slot",
            "@1.fingerprint=deadbeef",
            "--slot",
            "@1.path=48'/0'/0'/2'",
            "--no-engraving-card",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("BIP-388 distinct-key violation"),
        "h-vs-' same-xpub paths must collide under typed-path distinctness; got stderr:\n{stderr}"
    );
}
