//! C4 — `bundle --descriptor "<keyless>"` gives an HONEST refusal.
//!
//! A keyless concrete descriptor (no pubkeys — hashlock/timelock only) cannot be
//! a coherent m-format bundle (no cosigner key to engrave as an mk1 card). Bundle
//! refuses it (exit 2), but now with a message that names the real reason and
//! routes to `export-wallet --descriptor … --format descriptor` (which emits it
//! as a watch-only descriptor file) — NOT the vacuous "must carry a key origin".

use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;

fn bin() -> Command {
    Command::cargo_bin("mnemonic").expect("binary built")
}

/// Keyless hashlock+timelock → honest export-wallet route.
#[test]
fn bundle_keyless_descriptor_routes_to_export_wallet() {
    bin()
        .args([
            "bundle",
            "--descriptor",
            "wsh(and_v(v:ripemd160(0000000000000000000000000000000000000000),older(1234567)))",
            "--network",
            "mainnet",
            "--no-engraving-card",
        ])
        .assert()
        .failure()
        .code(2)
        .stderr(
            predicates::str::contains("export-wallet --descriptor")
                .and(predicates::str::contains("no keys to engrave")),
        );
}

/// Contrast: a KEY-but-origin-less descriptor (raw pubkey, no `[fp/path]`) keeps
/// the existing "must carry a key origin" message — the C4 split is narrow.
#[test]
fn bundle_origin_less_key_keeps_origin_message() {
    bin()
        .args([
            "bundle",
            "--descriptor",
            "wpkh(0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798)",
            "--network",
            "mainnet",
            "--no-engraving-card",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("must carry a key origin"));
}

/// `export-wallet-bundle-descriptor-md1-clearer-error`: an md1 CARD passed to
/// `bundle --descriptor` gets a clear surface-pointing refusal, NOT the
/// misleading classify_descriptor_form "keyless script" message.
#[test]
fn bundle_md1_card_on_descriptor_clear_refusal() {
    let md1 = "md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np";
    let out = bin()
        .args(["bundle", "--descriptor", md1, "--network", "mainnet"])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("md1 descriptor-mnemonic CARD"),
        "stderr: {stderr:?}"
    );
    // Important R0 fold: gate the WORKING pointer (subcommand-qualified), not bare `xpub-search`.
    assert!(
        stderr.contains("xpub-search account-of-descriptor"),
        "stderr: {stderr:?}"
    );
    assert!(
        !stderr.contains("keyless script"),
        "must NOT be classify_descriptor_form's keyless msg: {stderr:?}"
    );
}

/// `wave4-L2` site sweep (2026-08-03 post-implementation audit): the L2 fix
/// wired `bundle` and `export-wallet` but MISSED `verify-bundle --descriptor`,
/// the wallet-VERIFICATION surface — the one where pasting the engraved card is
/// the natural mistake. It answered with classify's "keyless script
/// (hashlock/timelock only)" message and pointed at `export-wallet
/// --descriptor`, which refuses the same card, sending the user in a circle.
#[test]
fn verify_bundle_md1_card_on_descriptor_clear_refusal() {
    let md1 = "md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np";
    let mk1 = "mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4";
    let out = bin()
        .args([
            "verify-bundle",
            "--network",
            "mainnet",
            "--mk1",
            mk1,
            "--md1",
            md1,
            "--descriptor",
            md1,
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("md1 descriptor-mnemonic CARD"),
        "stderr: {stderr:?}"
    );
    assert!(
        !stderr.contains("keyless script"),
        "must NOT be classify_descriptor_form's keyless msg: {stderr:?}"
    );
}

/// Same sweep, fourth surface: `compare-cost --descriptor` previously echoed the
/// entire card back inside an opaque `unrecognized name '<card>'` parse error.
#[test]
fn compare_cost_md1_card_on_descriptor_clear_refusal() {
    let md1 = "md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np";
    let out = bin()
        .args(["compare-cost", "--descriptor", md1])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("md1 descriptor-mnemonic CARD"),
        "stderr: {stderr:?}"
    );
    assert!(
        !stderr.contains("unrecognized name"),
        "must NOT echo the card in an opaque parse error: {stderr:?}"
    );
}
