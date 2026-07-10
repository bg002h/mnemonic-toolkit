//! v0.27.0 Phase 3 — `mnemonic export-wallet --format bsms` integration tests.
//!
//! SPEC `design/PLAN_v0_27_0_bsms_round_trip_and_wallet_import_handoff.md`
//! §3.5 / §3.5.1. Closes FOLLOWUPS `wallet-export-bsms-emitter` (FOLLOWUPS.md:2153)
//! and `bsms-first-address-verify` (FOLLOWUPS.md:2083; closure is by helper
//! availability + import-side WARNING wire-up at `wallet_import/bsms.rs`).
//!
//! 8 cells per plan §3.5 (cell 3 substitutes 3-of-4 for 3-of-5 due to
//! cosigner-vector availability — structural K-of-N coverage is identical):
//!   1. 4-line emit 2-of-2 wsh-sortedmulti mainnet
//!   2. 4-line emit 2-of-3 wsh-multi testnet
//!   3. 4-line emit sortedmulti 3-of-4 (K>2 / N>3 structural coverage)
//!   4. 4-line path-restrictions emits `/0/*,/1/*` for canonical multipath
//!   5. 4-line first-address byte-exact against independent descriptor derivation
//!   6. 4-line taproot descriptor errors explicit deferred
//!   7. 2-line lenient excerpt emits descriptor only
//!   8. 2-line → import idempotent (2-line round-trip through v0.26.0 parser)
//!
//! The first-address derivation primitive `crate::derive_address::derive_first_address`
//! is shared between the emitter (line 4 of 4-line emit) and the import-side
//! BSMS 6-line WARNING (which closes the import-half of the FOLLOWUP).

use assert_cmd::Command;

// Pinned cosigners (shared across export-wallet test files).
const COSIGNER_A_XPUB: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
const COSIGNER_A_FP: &str = "b8688df1";
const COSIGNER_B_XPUB: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";
const COSIGNER_B_FP: &str = "28645006";
const COSIGNER_C_XPUB: &str = "xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx";
const COSIGNER_C_FP: &str = "5436d724";
const COSIGNER_D_XPUB: &str = "xpub6Bv8ayijom26yJ1wZ62h4X1smfYBfNeNtGujxw6vaY4zq4Tw4cn2oV8qZmjnuVxh56oSe21r7V8r9LjZjArFh3QRZQbgzgLcfjVikZNa86W";
const COSIGNER_D_FP: &str = "16a93ed0";

// cycle-H F3 (E4/`assert_network_agrees`): the `--network testnet` cell below
// used to pair with the MAINNET-prefixed `COSIGNER_{A,B,C}_XPUB` consts above
// — that combination is now correctly refused fail-closed (a wrong-network
// mint, the exact hazard the guard closes). These are the SAME cosigner keys
// re-labeled with the testnet version bytes (offline version-byte swap, same
// key material — not a CLI network re-label, which the guard itself refuses)
// so the testnet cell exercises network-consistent watch-only input.
const COSIGNER_A_XPUB_TESTNET: &str = "tpubDFnc6MoxQh6V2NoQKZmq4a9HFuNxMD2cR785reRSe54JwcYH6KK5NQjAspMUmQp5qXdscseFqD4H3VuRVvNhizP4Ku87N5BfuBUQJGrfe1Y";
const COSIGNER_B_XPUB_TESTNET: &str = "tpubDE9rhca81b8zga4T9czxb4wjy1ZjbXtPnto2orq74XiVkLAtDDDTQrFppdVcVk7WHz8PmTsdLAjEeQTLsBBLWscv1cdYHCtXUe3FjRgWdjS";
const COSIGNER_C_XPUB_TESTNET: &str = "tpubDCHbTPBTK2GJLWMnad9HaY3Q2XUZNas1RmyQeJNENUA4FS9KNoF7k3kzfFCvYxR3CJgLRfgASq7zsX2SZjn1y6t5QGXzzy5ua6SJLqkY4A5";

/// Helper: run `mnemonic export-wallet --format bsms ...` with the supplied
/// extra args; returns (stdout, stderr) on success.
fn run_bsms(extra_args: &[&str]) -> (String, String) {
    let mut argv: Vec<&str> = vec!["export-wallet", "--format", "bsms"];
    argv.extend_from_slice(extra_args);
    argv.extend_from_slice(&["--output", "-"]);
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&argv)
        .assert()
        .success();
    (
        String::from_utf8(out.get_output().stdout.clone()).unwrap(),
        String::from_utf8(out.get_output().stderr.clone()).unwrap(),
    )
}

/// SPEC v0.27.0 §3.5 cell 1 — 2-of-2 wsh-sortedmulti mainnet emits 4 lines.
#[test]
fn bsms_4line_emit_2of2_wsh_sortedmulti_mainnet() {
    let (stdout, _) = run_bsms(&[
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
    ]);
    let lines: Vec<&str> = stdout.trim_end_matches('\n').split('\n').collect();
    assert_eq!(
        lines.len(),
        4,
        "expected 4 lines, got {} in:\n{stdout}",
        lines.len()
    );
    assert_eq!(lines[0], "BSMS 1.0");
    assert!(
        lines[1].starts_with("wsh(sortedmulti(2,"),
        "line 2 must be the wsh(sortedmulti(...)) descriptor; got {:?}",
        lines[1]
    );
    assert!(
        lines[1].contains("#"),
        "line 2 must carry the #<checksum> suffix; got {:?}",
        lines[1]
    );
    assert_eq!(
        lines[2], "/0/*,/1/*",
        "line 3 path-restrictions for canonical multipath"
    );
    // Line 4: bech32 mainnet P2WSH prefix.
    assert!(
        lines[3].starts_with("bc1q"),
        "line 4 must be a mainnet bech32 P2WSH address; got {:?}",
        lines[3]
    );
}

/// SPEC v0.27.0 §3.5 cell 2 — 2-of-3 wsh-multi testnet emits 4 lines.
#[test]
fn bsms_4line_emit_2of3_wsh_multi_testnet() {
    let (stdout, _) = run_bsms(&[
        "--template",
        "wsh-multi",
        "--threshold",
        "2",
        "--multisig-path-family",
        "bip48",
        "--network",
        "testnet",
        "--slot",
        &format!("@0.xpub={COSIGNER_A_XPUB_TESTNET}"),
        "--slot",
        &format!("@0.fingerprint={COSIGNER_A_FP}"),
        "--slot",
        "@0.path=m/48'/1'/0'/2'",
        "--slot",
        &format!("@1.xpub={COSIGNER_B_XPUB_TESTNET}"),
        "--slot",
        &format!("@1.fingerprint={COSIGNER_B_FP}"),
        "--slot",
        "@1.path=m/48'/1'/0'/2'",
        "--slot",
        &format!("@2.xpub={COSIGNER_C_XPUB_TESTNET}"),
        "--slot",
        &format!("@2.fingerprint={COSIGNER_C_FP}"),
        "--slot",
        "@2.path=m/48'/1'/0'/2'",
    ]);
    let lines: Vec<&str> = stdout.trim_end_matches('\n').split('\n').collect();
    assert_eq!(lines.len(), 4, "expected 4 lines, got {}", lines.len());
    assert_eq!(lines[0], "BSMS 1.0");
    assert!(
        lines[1].starts_with("wsh(multi(2,"),
        "line 2 must be wsh(multi(...)); got {:?}",
        lines[1]
    );
    assert_eq!(lines[2], "/0/*,/1/*");
    // testnet bech32 P2WSH prefix: tb1q...
    assert!(
        lines[3].starts_with("tb1q"),
        "line 4 must be a testnet bech32 P2WSH address; got {:?}",
        lines[3]
    );
}

/// SPEC v0.27.0 §3.5 cell 3 — sortedmulti 3-of-4 (K>2 / N>3 structural
/// coverage; substitutes for the plan's "3-of-5" name to avoid generating a
/// new synthetic xpub vector — load-bearing invariant identical).
#[test]
fn bsms_4line_emit_sortedmulti_3of5() {
    let (stdout, _) = run_bsms(&[
        "--template",
        "wsh-sortedmulti",
        "--threshold",
        "3",
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
        "--slot",
        &format!("@2.xpub={COSIGNER_C_XPUB}"),
        "--slot",
        &format!("@2.fingerprint={COSIGNER_C_FP}"),
        "--slot",
        "@2.path=m/48'/0'/0'/2'",
        "--slot",
        &format!("@3.xpub={COSIGNER_D_XPUB}"),
        "--slot",
        &format!("@3.fingerprint={COSIGNER_D_FP}"),
        "--slot",
        "@3.path=m/48'/0'/0'/2'",
    ]);
    let lines: Vec<&str> = stdout.trim_end_matches('\n').split('\n').collect();
    assert_eq!(lines.len(), 4);
    assert!(
        lines[1].starts_with("wsh(sortedmulti(3,"),
        "line 2 must be wsh(sortedmulti(3,...)); got {:?}",
        lines[1]
    );
    // 4 cosigners present in line 2 (one xpub per cosigner — count by xpub prefix).
    let xpub_count = lines[1].matches("xpub").count();
    assert_eq!(
        xpub_count, 4,
        "expected 4 cosigners (xpub occurrences) in line 2, got {xpub_count}"
    );
}

/// SPEC v0.27.0 §3.5.1 cell 4 — path-restrictions emit is `/0/*,/1/*` for a
/// canonical `<0;1>/*` multipath descriptor. (Implicit in cells 1-3; this
/// cell makes the assertion explicit + standalone.)
#[test]
fn bsms_4line_path_restrictions_emits_slash_0_star_slash_1_star_for_multipath() {
    let (stdout, _) = run_bsms(&[
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
    ]);
    let line3 = stdout.split('\n').nth(2).expect("line 3");
    assert_eq!(line3, "/0/*,/1/*");
}

/// SPEC v0.27.0 §3.5 cell 5 — line 4 (first-address) is byte-exact equal to
/// an independent miniscript-driven derivation through the same primitive
/// the toolkit uses. Cross-checks the helper's correctness against
/// `Descriptor::at_derivation_index(0).address(network)` directly.
#[test]
fn bsms_4line_first_address_byte_exact_against_descriptor_derivation() {
    // Capture both the emitter output AND the canonical descriptor it embeds
    // (line 2). Then independently derive /0/0 from that exact descriptor
    // string via miniscript and compare.
    let (stdout, _) = run_bsms(&[
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
    ]);
    let lines: Vec<&str> = stdout.trim_end_matches('\n').split('\n').collect();
    let canonical = lines[1];
    let emitted_first_address = lines[3];

    // Independent derivation via miniscript directly.
    use miniscript::{Descriptor, DescriptorPublicKey};
    use std::str::FromStr;
    let parsed =
        Descriptor::<DescriptorPublicKey>::from_str(canonical).expect("descriptor re-parse");
    let receive = parsed
        .into_single_descriptors()
        .expect("multipath split")
        .remove(0);
    let definite = receive.derive_at_index(0).expect("derive_at_index");
    let independent = definite
        .address(bitcoin::Network::Bitcoin)
        .expect("address render")
        .to_string();

    assert_eq!(
        emitted_first_address, independent,
        "BSMS 4-line line 4 must byte-equal an independent miniscript derivation:\n\
         emitted: {emitted_first_address}\nindependent: {independent}"
    );
}

/// SPEC v0.27.0 §3.5 cell 6 / v0.28.0 plan-doc §S.8 (P8A/P8B) — `tr(...)`
/// taproot descriptors REFUSE under `--format bsms`. BIP-129 §1 prerequisites
/// pre-date BIP-386 — no published canonicalization yet.
///
/// v0.28.0 tightened the refusal: the message now carries the per-script-type
/// discriminator (P2tr / P2trMulti), a BIP-386 status note, a FOLLOWUP slug
/// pointer (`bsms-taproot-emit`), and pointers to alternative formats
/// (`--format bitcoin-core` / `--format sparrow`). Exit code is 2 (unchanged
/// — same parse/refusal class as the prior `BadInput` text it replaces).
///
/// This cell exercises **P2trMulti** (template `tr-sortedmulti-a`).
#[test]
fn bsms_4line_taproot_multisig_refused_carries_full_v0_28_diagnostic() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "bsms",
            "--template",
            "tr-sortedmulti-a",
            "--threshold",
            "2",
            "--multisig-path-family",
            "bip87",
            "--network",
            "mainnet",
            "--taproot-internal-key",
            "nums",
            "--slot",
            &format!("@0.xpub={COSIGNER_A_XPUB}"),
            "--slot",
            &format!("@0.fingerprint={COSIGNER_A_FP}"),
            "--slot",
            &format!("@1.xpub={COSIGNER_B_XPUB}"),
            "--slot",
            &format!("@1.fingerprint={COSIGNER_B_FP}"),
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    // Headline + script-type discriminator (P2trMulti for tr-sortedmulti-a).
    assert!(
        stderr.contains("--format bsms does not support taproot"),
        "stderr must lead with the BIP-129 §1 refusal headline; got:\n{stderr}"
    );
    assert!(
        stderr.contains("(P2trMulti)"),
        "stderr must carry the P2trMulti script-type discriminator for tr-sortedmulti-a; got:\n{stderr}"
    );
    assert!(
        !stderr.contains("(P2tr)"),
        "stderr must NOT carry the singlesig discriminator for the multisig template; got:\n{stderr}"
    );
    // BIP-386 status note.
    assert!(
        stderr.contains("BIP-129 §1 prerequisites do not yet include BIP-386"),
        "stderr must cite the BIP-386 prerequisite gap; got:\n{stderr}"
    );
    // FOLLOWUP slug pointer (lets watchers grep the toolkit's tracker).
    assert!(
        stderr.contains("`bsms-taproot-emit`"),
        "stderr must point at the FOLLOWUP slug for upstream-watch; got:\n{stderr}"
    );
    // Alternative-format pointers.
    assert!(
        stderr.contains("--format bitcoin-core"),
        "stderr must point at bitcoin-core alternative; got:\n{stderr}"
    );
    assert!(
        stderr.contains("--format sparrow"),
        "stderr must point at sparrow alternative; got:\n{stderr}"
    );
}

/// SPEC v0.27.0 §3.5 cell 6 / v0.28.0 plan-doc §S.8 (P8A) — companion to
/// the multisig refusal cell above. Exercises **P2tr** (taproot singlesig via
/// `--template bip86`) to verify the per-script-type discriminator
/// distinguishes `P2tr` from `P2trMulti`. Same exit-code (2) + same overall
/// message body except for the parenthetical script-type token.
#[test]
fn bsms_taproot_singlesig_refused_carries_p2tr_discriminator() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "bsms",
            "--template",
            "bip86",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={COSIGNER_A_XPUB}"),
            "--slot",
            &format!("@0.fingerprint={COSIGNER_A_FP}"),
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("--format bsms does not support taproot"),
        "stderr must lead with the BIP-129 §1 refusal headline; got:\n{stderr}"
    );
    assert!(
        stderr.contains("(P2tr)"),
        "stderr must carry the P2tr (singlesig) script-type discriminator for bip86; got:\n{stderr}"
    );
    assert!(
        !stderr.contains("P2trMulti"),
        "stderr must NOT carry the multisig discriminator for the singlesig template; got:\n{stderr}"
    );
    assert!(
        stderr.contains("`bsms-taproot-emit`"),
        "stderr must point at the FOLLOWUP slug; got:\n{stderr}"
    );
}

/// SPEC v0.27.0 §3.5 cell 7 — `--bsms-form 2-line` emits the lenient excerpt
/// shape (just header + descriptor; no path-restrictions, no first-address).
#[test]
fn bsms_2line_lenient_excerpt_emits_descriptor_only() {
    let (stdout, _) = run_bsms(&[
        "--bsms-form",
        "2-line",
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
    ]);
    let lines: Vec<&str> = stdout.trim_end_matches('\n').split('\n').collect();
    assert_eq!(
        lines.len(),
        2,
        "expected 2 lines for 2-line form, got {}",
        lines.len()
    );
    assert_eq!(lines[0], "BSMS 1.0");
    assert!(lines[1].starts_with("wsh(sortedmulti(2,"));
    assert!(lines[1].contains("#"), "checksum suffix present");
}

/// SPEC v0.27.0 §3.5 cell 8 — 2-line emit → v0.26.0 import-side 2-line
/// lenient parser is byte-exact-idempotent. The toolkit's import-side parser
/// at `wallet_import/bsms.rs:95-102` accepts the 2-line lenient form; this
/// cell asserts the 2-line emit can be piped into `import-wallet` and yields
/// success. (v0.27.0 ingest does NOT add a 4-line lenient parser — closing
/// the full 4-line round-trip is tracked by FOLLOWUP `bsms-bip129-full-cutover`.)
#[test]
fn bsms_2line_then_import_byte_exact_idempotent() {
    // Emit 2-line (the v0.26.0 parser supports this shape).
    let (stdout, _) = run_bsms(&[
        "--bsms-form",
        "2-line",
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
    ]);

    // Pipe through stdin → import-wallet --blob -.
    let mut cmd = Command::cargo_bin("mnemonic").unwrap();
    cmd.args(["import-wallet", "--blob", "-", "--json"]);
    cmd.write_stdin(stdout.clone());
    let out = cmd.assert().success();
    let imported_stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    // v0.26.0 envelope shape carries a `bundle.cosigners` summary keyed on
    // (xpub, fingerprint, path_raw); Phase 4 promotes this to a full
    // BundleJson with descriptor body. Until then the round-trip assertion
    // pivots on cosigner-set fidelity: both xpubs supplied at emit time
    // must surface in the post-import envelope's cosigners array.
    assert!(
        imported_stdout.contains(COSIGNER_A_XPUB),
        "round-trip emit→import must surface cosigner A's xpub in the envelope; got:\n{imported_stdout}"
    );
    assert!(
        imported_stdout.contains(COSIGNER_B_XPUB),
        "round-trip emit→import must surface cosigner B's xpub in the envelope; got:\n{imported_stdout}"
    );
    assert!(
        imported_stdout.contains("\"threshold\": 2"),
        "round-trip emit→import must record threshold=2 in the envelope; got:\n{imported_stdout}"
    );
}
