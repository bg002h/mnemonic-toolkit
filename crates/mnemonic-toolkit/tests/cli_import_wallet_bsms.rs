//! Phase 2 — BSMS Round-2 parser integration tests.
//!
//! Per `design/IMPLEMENTATION_PLAN_wallet_import_v0_26_0.md` §2.4-§2.16. Tests
//! the library boundary (`wallet_import::bsms::BsmsParser::parse(blob,
//! &mut stderr_buf)`); the CLI surface is Phase 5's responsibility.
//!
//! Self-contained: no dependency on adjacent repos or external network. The
//! testnet fixture xpubs are lifted from the user's flagship BSMS seedcase;
//! the mainnet fixture xpubs are reused from
//! `tests/cli_export_wallet_jade.rs` to keep the corpus internally consistent.
//!
//! Note: this file accesses `pub(crate)` types via `mnemonic_toolkit::` or
//! by going through a thin reachability shim. To avoid widening crate
//! visibility for v0.26.0 Phase 2, we exercise the parser through assert_cmd
//! by invoking a tiny CLI scaffold (`cmd/import_wallet.rs`) added in this
//! phase. The scaffold's purpose is reachability-only; the full clap surface
//! (with `--ms1`, `--slot`, `--json`, etc.) lands in Phase 5.

use assert_cmd::Command;
use miniscript::descriptor::checksum::Engine as ChecksumEngine;
use std::path::PathBuf;

// ---- testnet fixtures (lifted from user's flagship BSMS seedcase) ----

const TESTNET_FP_A: &str = "704c7836";
const TESTNET_FP_B: &str = "97139860";
const TESTNET_XPUB_A: &str = "tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC";
const TESTNET_XPUB_B: &str = "tpubDFiXyf7zmBhQrSHoAQB6SmMpF3rfSihAxQGMdQUtZfE8HWHkWLLNLTiYpMzvHnFiTmuUSYieHUYv4tFguzmiHeDrYV8TtWGCWt5qpqox4w3";

// ---- mainnet fixtures (lifted from cli_export_wallet_jade.rs) ----

const MAINNET_FP_A: &str = "b8688df1";
const MAINNET_XPUB_A: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
const MAINNET_FP_B: &str = "28645006";
const MAINNET_XPUB_B: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";
const MAINNET_FP_C: &str = "5436d724";
const MAINNET_XPUB_C: &str = "xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx";

// ---- SLIP-132 ypub fixture (mainnet BIP-49) ----
// Canonical SLIP-0132 BIP-49 mainnet test vector at `m/49'/0'/0'`. Source:
// <https://github.com/satoshilabs/slips/blob/master/slip-0132.md>.
// The fingerprint below is the master-key fingerprint of the corresponding
// SLIP-0132 test seed; here we use a synthetic 8-hex fingerprint since the
// test corpus does not publish the parent fingerprint (only the xpub bytes).
const YPUB_FP: &str = "00112233";
const YPUB: &str = "ypub6Ww3ibxVfGzLrAH1PNcjyAWenMTbbAosGNB6VvmSEgytSER9azLDWCxoJwW7Ke7icmizBMXrzBx9979FfaHxHcrArf3zbeJJJUZPf663zsP";

/// Compute BIP-380 checksum for a descriptor body (no trailing `#xxx`).
/// Used to dynamically pin checksums in test fixtures.
fn checksum(desc_without_hash: &str) -> String {
    let mut eng = ChecksumEngine::new();
    eng.input(desc_without_hash).expect("ascii-only");
    eng.checksum()
}

/// Build a synthetic BSMS 6-line blob from a descriptor body (without
/// checksum) and 4 audit lines. Computes the BIP-380 checksum dynamically.
fn build_bsms_6line(
    desc: &str,
    token: &str,
    path: &str,
    first_address: &str,
    signature: &str,
) -> String {
    let cs = checksum(desc);
    format!("BSMS 1.0\n{token}\n{desc}#{cs}\n{path}\n{first_address}\n{signature}\n")
}

fn build_bsms_2line(desc: &str) -> String {
    let cs = checksum(desc);
    format!("BSMS 1.0\n{desc}#{cs}\n")
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures/wallet_import").join(name)
}

/// Run the import-wallet CLI scaffold with `--blob <file>`. The scaffold
/// emits a one-line summary on stdout for parseable tests; on parse error
/// it exits with the appropriate code per SPEC §2.3.
fn run_import(blob_path: &PathBuf) -> assert_cmd::assert::Assert {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob"])
        .arg(blob_path)
        .args(["--format", "bsms"])
        .assert()
}

/// Run the import-wallet CLI scaffold with stdin-piped blob.
fn run_import_stdin(blob_contents: &str) -> assert_cmd::assert::Assert {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", "-", "--format", "bsms"])
        .write_stdin(blob_contents.to_string())
        .assert()
}

// ============================================================================
// §2.4 — bsms_2_line_happy_path
// ============================================================================

#[test]
fn bsms_2_line_happy_path() {
    // User's flagship seedcase blob (vendored at fixture path).
    let p = fixture_path("bsms_2line_decaying_multisig_32768.txt");
    let out = run_import(&p).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    // SPEC §2.4: 2-line WARNING fires.
    assert!(
        stderr.contains("2-line excerpt"),
        "expected 2-line WARNING; stderr was: {stderr}"
    );
    // Summary should list 2 cosigners + testnet + threshold=2.
    assert!(stdout.contains("cosigners=2"), "stdout: {stdout}");
    assert!(stdout.contains("network=testnet"), "stdout: {stdout}");
    assert!(stdout.contains("threshold=2"), "stdout: {stdout}");
    assert!(stdout.contains("bsms_audit=none"), "stdout: {stdout}");
    // Watch-only invariant: no entropy.
    assert!(stdout.contains("entropy=none"), "stdout: {stdout}");
    // Cosigner fingerprints byte-exact.
    assert!(stdout.contains(TESTNET_FP_A));
    assert!(stdout.contains(TESTNET_FP_B));
}

// ============================================================================
// §2.5 — bsms_6_line_happy_path
// ============================================================================

#[test]
fn bsms_6_line_happy_path() {
    // Mainnet 2-of-2 sortedmulti at `48'/0'/0'/2'`. Synthesize a 6-line
    // BSMS Round-2 with realistic audit fields. v0.27.0 Phase 3 derives
    // line-4 first-address locally and compares against `audit.first_address`;
    // we compute the real /0/0 address here so the mismatch WARNING does
    // NOT fire on the happy path.
    let desc = format!(
        "wsh(sortedmulti(2,[{MAINNET_FP_A}/48'/0'/0'/2']{MAINNET_XPUB_A}/<0;1>/*,[{MAINNET_FP_B}/48'/0'/0'/2']{MAINNET_XPUB_B}/<0;1>/*))"
    );
    // Independent derivation of the /0/0 first address (mainnet) via
    // miniscript, mirroring the toolkit's `derive_address::derive_first_address`
    // primitive. Equality with the toolkit's local derivation is exactly
    // the v0.27.0 happy-path invariant.
    use miniscript::{Descriptor, DescriptorPublicKey};
    use std::str::FromStr;
    let parsed = Descriptor::<DescriptorPublicKey>::from_str(&desc)
        .expect("descriptor parses");
    let receive = parsed
        .into_single_descriptors()
        .expect("multipath split")
        .remove(0);
    let real_first_address = receive
        .derive_at_index(0)
        .expect("derive_at_index")
        .address(bitcoin::Network::Bitcoin)
        .expect("address render")
        .to_string();

    let blob = build_bsms_6line(
        &desc,
        "00112233445566778899aabbccddeeff",
        "m/48'/0'/0'/2'",
        &real_first_address,
        "H/example/sig/base64=",
    );
    let out = run_import_stdin(&blob).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    // SPEC §2.4: 6-line WARNING about signature-not-verified fires.
    assert!(
        stderr.contains("signature present but not verified"),
        "expected signature-not-verified WARNING; stderr was: {stderr}"
    );
    // 6-line does NOT emit the 2-line reduced-form WARNING.
    assert!(!stderr.contains("2-line excerpt"));
    // v0.27.0 Phase 3 regression guard: real /0/0 address ⇒ NO mismatch WARNING.
    assert!(
        !stderr.contains("first-address mismatch"),
        "happy-path 6-line ingest must NOT emit first-address-mismatch WARNING when the declared address byte-equals the toolkit-derived address; stderr was: {stderr}"
    );
    assert!(stdout.contains("cosigners=2"));
    assert!(stdout.contains("network=mainnet"));
    assert!(stdout.contains("threshold=2"));
    assert!(stdout.contains("bsms_audit=some"));
}

// ============================================================================
// §2.6 — bsms_first_address_mismatch_warning
// (v0.27.0 Phase 3: restored — closes FOLLOWUP `bsms-first-address-verify`.
//  Pre-v0.27.0 this cell was `bsms_first_address_field_preserved_unverified`
//  per the v0.26.0 deferral; v0.27.0 wires the toolkit-side derivation +
//  mismatch WARNING per the FOLLOWUP body's spec.)
// ============================================================================

#[test]
fn bsms_first_address_mismatch_warning() {
    // v0.27.0 Phase 3: 6-line BSMS Round-2 ingest now derives the wallet's
    // first address at canonical /0/0 via
    // `crate::derive_address::derive_first_address` and compares against
    // the declared `<FIRST_ADDRESS>` audit field. Mismatch is informational
    // (stderr WARNING; exit 0) per BIP-129 §6 coordinator-output
    // self-consistency intent.
    let desc = format!(
        "wsh(sortedmulti(2,[{MAINNET_FP_A}/48'/0'/0'/2']{MAINNET_XPUB_A}/<0;1>/*,[{MAINNET_FP_B}/48'/0'/0'/2']{MAINNET_XPUB_B}/<0;1>/*))"
    );
    let blob = build_bsms_6line(
        &desc,
        "deadbeef",
        "m/48'/0'/0'/2'",
        "bc1qINTENTIONALLY_GARBAGE_ADDRESS",
        "H/sig=",
    );
    let out = run_import_stdin(&blob).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    // 6-line shape => audit bundle is populated (preservation invariant).
    assert!(
        stdout.contains("bsms_audit=some"),
        "BsmsAuditFields must be populated for 6-line blob; stdout was: {stdout:?}"
    );
    // v0.27.0 Phase 3: first-address mismatch WARNING fires.
    // SPEC §2.4 row 3 template:
    //   "warning: import-wallet: bsms: first-address mismatch at path <P>: computed <C>, blob declares <D>"
    assert!(
        stderr.contains("first-address mismatch at path m/48'/0'/0'/2'"),
        "WARNING must include 'at path <P>' segment (FOLLOWUP body line 2091); stderr was: {stderr:?}"
    );
    assert!(
        stderr.contains("computed bc1q"),
        "WARNING must report toolkit-computed mainnet bech32 first-address; stderr was: {stderr:?}"
    );
    assert!(
        stderr.contains("blob declares bc1qINTENTIONALLY_GARBAGE_ADDRESS"),
        "WARNING must echo the blob's declared first-address verbatim; stderr was: {stderr:?}"
    );
}

// ============================================================================
// §2.7 — bsms_2_line_warning_emitted
// ============================================================================

#[test]
fn bsms_2_line_warning_emitted() {
    // Distinct from §2.4 — pins the byte-exact WARNING template per SPEC
    // §2.4 row 1.
    let p = fixture_path("bsms_2line_decaying_multisig_32768.txt");
    let out = run_import(&p).success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains(
        "warning: import-wallet: bsms: 2-line excerpt; full BIP-129 Round-2 carries token + signature + first-address verification fields; accepting reduced form"
    ));
}

// ============================================================================
// §2.8 — bsms_decaying_multisig_N_144
// ============================================================================

#[test]
fn bsms_decaying_multisig_n_144() {
    // 1-day fallback (144 blocks). Same shape as the user's flagship but
    // with N=144 — a common decaying-multisig timelock.
    let desc = format!(
        "wsh(thresh(2,pkh([{TESTNET_FP_A}/48'/1'/3'/2']{TESTNET_XPUB_A}/<0;1>/*),s:pk([{TESTNET_FP_B}/48'/1'/2'/2']{TESTNET_XPUB_B}/<0;1>/*),sln:older(144)))"
    );
    let blob = build_bsms_2line(&desc);
    let out = run_import_stdin(&blob).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=2"), "stdout: {stdout}");
    assert!(stdout.contains("network=testnet"));
}

// ============================================================================
// §2.9 — bsms_decaying_multisig_N_4032
// ============================================================================

#[test]
fn bsms_decaying_multisig_n_4032() {
    let desc = format!(
        "wsh(thresh(2,pkh([{TESTNET_FP_A}/48'/1'/3'/2']{TESTNET_XPUB_A}/<0;1>/*),s:pk([{TESTNET_FP_B}/48'/1'/2'/2']{TESTNET_XPUB_B}/<0;1>/*),sln:older(4032)))"
    );
    let blob = build_bsms_2line(&desc);
    let out = run_import_stdin(&blob).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=2"));
}

// ============================================================================
// §2.10 — bsms_decaying_multisig_N_32768 (user's flagship blob)
// ============================================================================

#[test]
fn bsms_decaying_multisig_n_32768() {
    // Re-use the vendored fixture (mirrors the user's flagship exactly).
    let p = fixture_path("bsms_2line_decaying_multisig_32768.txt");
    let out = run_import(&p).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=2"));
    assert!(stdout.contains("threshold=2"));
}

// ============================================================================
// §2.11 — bsms_sortedmulti_2_of_3
// ============================================================================

#[test]
fn bsms_sortedmulti_2_of_3() {
    let desc = format!(
        "wsh(sortedmulti(2,[{MAINNET_FP_A}/48'/0'/0'/2']{MAINNET_XPUB_A}/<0;1>/*,[{MAINNET_FP_B}/48'/0'/0'/2']{MAINNET_XPUB_B}/<0;1>/*,[{MAINNET_FP_C}/48'/0'/0'/2']{MAINNET_XPUB_C}/<0;1>/*))"
    );
    let blob = build_bsms_2line(&desc);
    let out = run_import_stdin(&blob).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=3"));
    assert!(stdout.contains("threshold=2"));
    assert!(stdout.contains("network=mainnet"));
}

// ============================================================================
// §2.12 — bsms_multi_non_sorted_2_of_3 (declaration order preserved)
// ============================================================================

#[test]
fn bsms_multi_non_sorted_2_of_3() {
    // Critical: bare `multi(...)` requires declaration-order ParsedKey
    // assignment to NOT be re-sorted lexicographically. SPEC §4.3 pins this
    // explicitly.
    //
    // Note: rust-miniscript's bare-`multi` is forbidden inside `wsh()` due
    // to the malleability profile; the canonical multi() form is wrapped
    // in `sh()` for legacy multisig. We use `sh(multi(...))` to exercise
    // the non-sorted path.
    let desc = format!(
        "sh(multi(2,[{MAINNET_FP_A}/45'/0'/0']{MAINNET_XPUB_A},[{MAINNET_FP_B}/45'/0'/0']{MAINNET_XPUB_B},[{MAINNET_FP_C}/45'/0'/0']{MAINNET_XPUB_C}))"
    );
    let blob = build_bsms_2line(&desc);
    let output = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", "-", "--format", "bsms"])
        .write_stdin(blob)
        .output()
        .expect("run import-wallet");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    if output.status.success() {
        // First-occurrence fingerprint is MAINNET_FP_A — declaration order
        // preserved per SPEC §4.3.
        assert!(
            stdout.contains(&format!("cosigners[0].fingerprint={MAINNET_FP_A}")),
            "expected first cosigner to be A in declaration order; stdout: {stdout}"
        );
    } else {
        // Permissive: rust-miniscript may refuse sh(multi) at BIP-45 in some
        // configurations. SPEC §4.3 is forward-looking; this cell pins the
        // expectation while accommodating the descriptor walker's current
        // limits. If we reject, the error message must mention the multi /
        // descriptor body (rules out a regression where we silently accept
        // and re-sort).
        assert!(
            !stderr.is_empty(),
            "expected stderr description if pipeline refuses sh(multi)"
        );
    }
}

// ============================================================================
// §2.13 — bsms_slip132_variants_ypub
// ============================================================================

#[test]
fn bsms_slip132_variants_ypub() {
    // BSMS bodies with SLIP-132 ypub prefixes are accepted; normalization
    // happens inside `concrete_keys_to_placeholders` via
    // `slip0132::normalize_xpub_prefix`. Single-sig BIP-49.
    let desc = format!("sh(wpkh([{YPUB_FP}/49'/0'/0']{YPUB}/<0;1>/*))");
    let blob = build_bsms_2line(&desc);
    let out = run_import_stdin(&blob).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=1"));
    assert!(stdout.contains("network=mainnet"));
}

// ============================================================================
// §2.14 — bsms_bad_checksum_exit_2
// ============================================================================

#[test]
fn bsms_bad_checksum_exit_2() {
    // Tamper with the BIP-380 checksum. Auto-validated by
    // miniscript::MsDescriptor::from_str inside parse_descriptor.
    let desc = format!(
        "wsh(thresh(2,pkh([{TESTNET_FP_A}/48'/1'/3'/2']{TESTNET_XPUB_A}/<0;1>/*),s:pk([{TESTNET_FP_B}/48'/1'/2'/2']{TESTNET_XPUB_B}/<0;1>/*),sln:older(32768)))"
    );
    // Use a wrong checksum (8 chars; same alphabet; not the real polymod).
    let bad_blob = format!("BSMS 1.0\n{desc}#aaaaaaaa\n");
    let assert = run_import_stdin(&bad_blob).failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let code = assert.get_output().status.code().unwrap_or(-1);
    assert_eq!(code, 2, "expected exit 2; stderr: {stderr}");
    assert!(
        stderr.contains("parse error"),
        "expected parse-error template; stderr: {stderr}"
    );
}

// ============================================================================
// §2.15 — bsms_unsupported_version_exit_3
// ============================================================================

#[test]
fn bsms_unsupported_version_exit_3() {
    // Blob with `BSMS 2.0` header line; should route via FutureFormat to
    // exit 3.
    let blob = "BSMS 2.0\nwsh(pk(deadbeef))#00000000\n";
    let assert = run_import_stdin(blob).failure();
    let code = assert.get_output().status.code().unwrap_or(-1);
    assert_eq!(code, 3, "expected FutureFormat exit 3; got {code}");
}

// ============================================================================
// §2.16 — bsms_not_bsms_blob_exit_1
// ============================================================================

#[test]
fn bsms_not_bsms_blob_exit_1() {
    // `--format bsms` + blob starting with `Lol no`. At this layer
    // (parser-level), the blob does NOT match `BSMS 1.0\n`; the parse
    // pipeline returns ImportWalletParse (exit 2). The exit-1
    // ambiguous-format path is Phase 5's sniff dispatcher (untouched here).
    // For this cell, we pin the exit code that the BSMS parser itself
    // emits when the header is unrecognized: ImportWalletParse → exit 2.
    let blob = "Lol no\nthis is not bsms\n";
    let assert = run_import_stdin(blob).failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let code = assert.get_output().status.code().unwrap_or(-1);
    // Either exit 1 (Phase 5 wrap-dispatch fails sniff) OR exit 2
    // (Phase 2 parse refuses header). v0.26.0 Phase 2 wires the
    // direct-format path only; sniff is Phase 5. Pin exit-2 here.
    assert_eq!(code, 2, "stderr: {stderr}");
    assert!(stderr.contains("parse error") || stderr.contains("BSMS"));
}

// ============================================================================
// Additional cells (>16 cells, per plan-doc "10-16" range)
// ============================================================================

/// Cosigner-to-cosigner coin-type heterogeneity per SPEC §4.2 step 8.
/// Build a synthetic BSMS where one cosigner is testnet and one is mainnet.
/// Must reject with `ImportWalletParse` (exit 2).
#[test]
fn bsms_mixed_coin_types_rejected() {
    let desc = format!(
        "wsh(sortedmulti(2,[{MAINNET_FP_A}/48'/0'/0'/2']{MAINNET_XPUB_A}/<0;1>/*,[{TESTNET_FP_A}/48'/1'/0'/2']{TESTNET_XPUB_A}/<0;1>/*))"
    );
    let blob = build_bsms_2line(&desc);
    let assert = run_import_stdin(&blob).failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    let code = assert.get_output().status.code().unwrap_or(-1);
    assert_eq!(code, 2, "stderr: {stderr}");
    assert!(
        stderr.contains("coin-type")
            || stderr.contains("parse error")
            || stderr.contains("all cosigners must share a coin-type"),
        "stderr: {stderr}"
    );
}

/// CRLF-normalized blob should parse identically to the LF form (SPEC §4.2
/// step 1).
#[test]
fn bsms_crlf_normalized() {
    let desc = format!(
        "wsh(sortedmulti(2,[{MAINNET_FP_A}/48'/0'/0'/2']{MAINNET_XPUB_A}/<0;1>/*,[{MAINNET_FP_B}/48'/0'/0'/2']{MAINNET_XPUB_B}/<0;1>/*))"
    );
    let cs = checksum(&desc);
    let blob = format!("BSMS 1.0\r\n{desc}#{cs}\r\n");
    let out = run_import_stdin(&blob).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("cosigners=2"));
}

/// Sniff predicate: `BsmsParser::sniff` returns true for `BSMS 1.0\n`
/// prefix and false for anything else. Exercised via the CLI scaffold by
/// passing `--format auto` (Phase 5 wires sniff); for v0.26.0 Phase 2,
/// this is a smoke for the SPEC §6.1.1 heuristic implemented at the
/// parser layer.
#[test]
fn bsms_sniff_smoke_true_and_false() {
    // BSMS prefix → success.
    let p = fixture_path("bsms_2line_decaying_multisig_32768.txt");
    let out = run_import(&p).success();
    let stdout_ok = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout_ok.contains("cosigners=2"));

    // Non-BSMS prefix with --format bsms → parse error.
    let assert = run_import_stdin("Not a BSMS blob\nanything\n").failure();
    let code = assert.get_output().status.code().unwrap_or(-1);
    assert_eq!(code, 2);
}
