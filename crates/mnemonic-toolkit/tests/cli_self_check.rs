//! `--self-check` happy-path test (Phase E.2).
//!
//! Self-check on a freshly-emitted bundle MUST succeed; a failure indicates a
//! synthesis/verify inconsistency. Fixture: the single read golden
//! `tests/vectors/v0_2/bip84-mainnet-0-false-true.txt` (v0.53.2: the 25
//! orphaned multisig v0_2 goldens were DELETED — no test read them and the
//! v0.48.0 NUMS + v0.53.0 csi changes staled their wire bytes; see FOLLOWUPS
//! `orphaned-v0_2-md1-vectors-no-harness`).

use assert_cmd::Command;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

#[test]
fn bundle_self_check_passes_for_canonical_seed_singlesig() {
    let expected = std::fs::read_to_string("tests/vectors/v0_2/bip84-mainnet-0-false-true.txt")
        .expect("fixture");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("--allow-argv-secret")
        .args([
            "bundle",
            "--slot",
            &format!("@0.phrase={TREZOR_24}"),
            "--network",
            "mainnet",
            "--scheme",
            "bip84",
            "--self-check",
            "--no-engraving-card",
            // mstring-grouping P4: golden is the unbroken canonical form.
            "--group-size",
            "0",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout, expected, "self-check single-sig fixture mismatch");
}

// Deleted v0.4.2 cleanup: bundle_self_check_passes_for_canonical_seed_multisig
// exercised the v0.2 self-multisig pattern (--cosigner-count 3 with --phrase),
// which was hard-rejected by BIP-388 in v0.4.0 and has no migration path.

// G-B (FOLLOWUP self-check-ms1-iteration, R0 C1b) used a `--slot @N.wif=`
// slot to prove the self-check oracle keys off `entropy.is_some()`. Since bug
// hunt 2026-11-25 X4, `bundle --scheme` refuses a WIF slot at mint (exit 1)
// before the self-check runs; the refusal is pinned instead.
#[test]
fn bundle_wif_slot_refused_before_self_check() {
    const SAMPLE_WIF: &str = "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn";
    Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("--allow-argv-secret")
        .args([
            "bundle",
            "--scheme",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.wif={SAMPLE_WIF}"),
            "--self-check",
            "--no-engraving-card",
        ])
        .assert()
        .failure()
        .code(1);
}
