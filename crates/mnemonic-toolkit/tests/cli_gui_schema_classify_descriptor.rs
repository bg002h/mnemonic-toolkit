//! v0.20.0 F2 — `mnemonic gui-schema --classify-descriptor <STR>` diagnostic
//! flag. Emits `canonical\n` or `non-canonical\n` on stdout based on
//! `md_codec::canonical_origin::canonical_origin` over the parsed descriptor's
//! tree. Exit 0 on parse success; exit 2 on parse failure (DescriptorParse
//! error variant).
//!
//! Drives the v0.8.1 GUI canonicity-classifier drift gate (Phase 4 of the
//! v0.20.0 cycle).

use assert_cmd::Command;
use predicates::prelude::*;

/// Cell 6 — canonical `pkh(@0)` returns `canonical\n` on stdout, exit 0.
#[test]
fn canonical_pkh_returns_canonical_exit_0() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["gui-schema", "--classify-descriptor", "pkh(@0)"])
        .assert()
        .success()
        .stdout("canonical\n");
}

/// Cell 7 — canonical `wpkh(@0)` returns `canonical\n`.
#[test]
fn canonical_wpkh_returns_canonical() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["gui-schema", "--classify-descriptor", "wpkh(@0)"])
        .assert()
        .success()
        .stdout("canonical\n");
}

/// Cell 8 — canonical `tr(@0)` (keypath-only) returns `canonical\n`.
#[test]
fn canonical_tr_keypath_returns_canonical() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["gui-schema", "--classify-descriptor", "tr(@0)"])
        .assert()
        .success()
        .stdout("canonical\n");
}

/// Cell 9 — canonical `wsh(multi(2,@0,@1,@2))` returns `canonical\n`.
#[test]
fn canonical_wsh_multi_returns_canonical() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "gui-schema",
            "--classify-descriptor",
            "wsh(multi(2,@0,@1,@2))",
        ])
        .assert()
        .success()
        .stdout("canonical\n");
}

/// Cell 10 — canonical `sh(wsh(sortedmulti(2,@0,@1,@2)))` returns `canonical\n`.
#[test]
fn canonical_sh_wsh_sortedmulti_returns_canonical() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "gui-schema",
            "--classify-descriptor",
            "sh(wsh(sortedmulti(2,@0,@1,@2)))",
        ])
        .assert()
        .success()
        .stdout("canonical\n");
}

/// Cell 11 — non-canonical `wsh(andor(...))` returns `non-canonical\n`.
#[test]
fn non_canonical_wsh_andor_returns_non_canonical() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "gui-schema",
            "--classify-descriptor",
            "wsh(andor(pkh(@0),after(12000000),pk(@1)))",
        ])
        .assert()
        .success()
        .stdout("non-canonical\n");
}

/// Cell 12 — non-canonical `tr(NUMS,pk(@0))` (taptree) returns `non-canonical\n`.
#[test]
fn non_canonical_tr_with_taptree_returns_non_canonical() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "gui-schema",
            "--classify-descriptor",
            "tr(NUMS,pk(@0))",
        ])
        .assert()
        .success()
        .stdout("non-canonical\n");
}

/// Cell 13 — malformed descriptor returns exit 2 (DescriptorParse) with no stdout.
#[test]
fn malformed_descriptor_exits_2_with_no_stdout() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "gui-schema",
            "--classify-descriptor",
            "this is not a descriptor",
        ])
        .assert()
        .code(2)
        .stdout(predicate::str::is_empty());
}
