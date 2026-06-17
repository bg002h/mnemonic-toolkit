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
