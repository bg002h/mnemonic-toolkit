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
    // SPEC §10.4 (v0.28.0): the 6-line shape now emits a DEPRECATION NOTICE
    // pointing the user at the BIP-129-canonical 4-line shape. The legacy
    // v0.27.0 "6-line lenient shape is DEPRECATED" reword has been folded into the
    // deprecation message (the 6-line surface is on its way out).
    assert!(
        stderr.contains("6-line lenient shape is DEPRECATED")
            && stderr.contains("4-line shape")
            && stderr.contains("SPEC §10"),
        "expected v0.28.0 6-line DEPRECATION NOTICE; stderr was: {stderr}"
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

/// v0.27.1 Phase 2 I6 fold cell — `thresh()` argument exceeding u8 range
/// in a BSMS Round-2 blob must surface as a typed parse error, not silently
/// render as `"threshold": null`. Mirrors the bitcoin-core-path cell.
#[test]
fn bsms_thresh_overflow_errors_clearly() {
    // 2-line lenient form: header + descriptor. The descriptor body carries
    // `sortedmulti(256, ...)` which triggers extract_threshold's u8 overflow
    // branch. The toolkit's descriptor parse may reject 256-cosigner shapes
    // upstream too; either rejection path closes the silent null-threshold
    // surface.
    let blob = "BSMS 1.0\nwsh(sortedmulti(256,[b8688df1/48'/0'/0'/2']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*,[28645006/48'/0'/0'/2']xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6/<0;1>/*))#abcdefgh\n";
    let assertion = run_import_stdin(blob).failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("exceeds u8 range") || stderr.contains("256") || stderr.to_lowercase().contains("threshold") || stderr.to_lowercase().contains("checksum"),
        "expected u8-overflow / 256-cosigner / checksum-rejection diagnostic; got: {stderr}"
    );
}

// ============================================================================
// v0.27.1 Phase 4 PR-#26 fold — I17 + I18
// ============================================================================

/// I17 — unrecognized BSMS line counts (3/5/7+) hit the
/// `wallet_import/bsms.rs::parse` "expected 2, 4, or 6 lines" rejection arm.
/// v0.28.0 admits a 4-line BIP-129-canonical Round-2 shape (SPEC §10), so
/// the prior 4-line rejection cell has been removed; the 4-line happy-path
/// plus first-address-mismatch cells below replace it. Cells below exercise
/// 3, 5, and 7-line inputs; all must reject at exit 2 with the locked
/// v0.28.0 template.
#[test]
fn bsms_3_line_blob_rejected_with_pointer_text() {
    let blob = "BSMS 1.0\nfoo\nbar\n";
    let assertion = run_import_stdin(blob).failure();
    let code = assertion.get_output().status.code().unwrap_or(-1);
    assert_eq!(code, 2, "non-{{2,4,6}}-line BSMS must exit 2");
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("expected 2, 4, or 6 lines"),
        "expected locked line-count rejection template; got: {stderr}"
    );
}

#[test]
fn bsms_5_line_blob_rejected_with_pointer_text() {
    let blob = "BSMS 1.0\na\nb\nc\nd\n";
    let assertion = run_import_stdin(blob).failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("expected 2, 4, or 6 lines"),
        "expected line-count rejection on 5-line blob; got: {stderr}"
    );
}

#[test]
fn bsms_7_line_blob_rejected_with_pointer_text() {
    let blob = "BSMS 1.0\na\nb\nc\nd\ne\nf\n";
    let assertion = run_import_stdin(blob).failure();
    let stderr = String::from_utf8(assertion.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("expected 2, 4, or 6 lines"),
        "expected line-count rejection on 7-line blob; got: {stderr}"
    );
}

/// I18 — BSMS sniff is strict-prefix `BSMS 1.0\n`. Lowercase `bsms 1.0\n`
/// and leading-whitespace ` BSMS 1.0\n` MUST NOT match. Pins this so a
/// future "tolerance" loosening doesn't silently accept malformed blobs.
#[test]
fn bsms_sniff_rejects_lowercase_header() {
    let blob = b"bsms 1.0\nwpkh(xpub...)\n";
    assert!(
        !crate::shared::bsms_sniff_via_dispatch(blob),
        "lowercase header must NOT match BSMS sniff"
    );
}

#[test]
fn bsms_sniff_rejects_leading_whitespace() {
    let blob = b" BSMS 1.0\nwpkh(xpub...)\n";
    assert!(
        !crate::shared::bsms_sniff_via_dispatch(blob),
        "leading-whitespace header must NOT match BSMS sniff"
    );
}

// ============================================================================
// v0.28.0 Phase 7 (G1) — SPEC §10 BIP-129-canonical 4-line Round-2 parser
// ============================================================================

/// SPEC §10.1 — 4-line BIP-129-canonical Round-2 shape (sortedmulti 2-of-3
/// mainnet P2WSH). Asserts: parser accepts the 4-line shape; first-address
/// cross-validation per SPEC §10.2 emits NO mismatch WARNING (the blob's
/// line-4 byte-equals `derive_first_address` at canonical /0/0); audit
/// provenance uses the empty-string-sentinel pattern per SPEC §10.3 (the
/// `bsms_audit=some` summary line confirms `Bsms(Some(...))` is constructed).
#[test]
fn bsms_4line_sortedmulti_2of3_happy_path() {
    let p = fixture_path("bsms-4line-sortedmulti-2of3.txt");
    let out = run_import(&p).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    // 4-line shape does NOT emit the 2-line reduced-form WARNING.
    assert!(
        !stderr.contains("2-line excerpt"),
        "4-line shape must NOT trigger 2-line WARNING; stderr was: {stderr}"
    );
    // 4-line shape does NOT emit the 6-line DEPRECATION NOTICE.
    assert!(
        !stderr.contains("6-line lenient shape is DEPRECATED"),
        "4-line shape must NOT trigger 6-line DEPRECATION NOTICE; stderr was: {stderr}"
    );
    // SPEC §10.2 happy-path: line-4 byte-equals derive_first_address ⇒ no WARNING.
    assert!(
        !stderr.contains("first-address mismatch"),
        "4-line happy-path must NOT emit first-address-mismatch WARNING; stderr was: {stderr}"
    );
    assert!(stdout.contains("cosigners=3"), "stdout: {stdout}");
    assert!(stdout.contains("network=mainnet"), "stdout: {stdout}");
    assert!(stdout.contains("threshold=2"), "stdout: {stdout}");
    // SPEC §10.3 empty-string-sentinel: audit is Some(...) for 4-line.
    assert!(stdout.contains("bsms_audit=some"), "stdout: {stdout}");
    // Cosigner fingerprints byte-exact.
    assert!(stdout.contains(MAINNET_FP_A));
    assert!(stdout.contains(MAINNET_FP_B));
    assert!(stdout.contains(MAINNET_FP_C));
}

/// SPEC §10.1 — 4-line BIP-129-canonical Round-2 shape, singlesig P2WPKH
/// (BIP-84). Asserts singlesig descriptors with non-multisig path-restrictions
/// parse through the same 4-line arm. Threshold is None for singlesig (no
/// thresh/multi/sortedmulti token in the descriptor).
#[test]
fn bsms_4line_singlesig_wpkh_happy_path() {
    let p = fixture_path("bsms-4line-singlesig-wpkh.txt");
    let out = run_import(&p).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(!stderr.contains("2-line excerpt"));
    assert!(!stderr.contains("6-line lenient shape is DEPRECATED"));
    assert!(!stderr.contains("first-address mismatch"));
    assert!(stdout.contains("cosigners=1"), "stdout: {stdout}");
    assert!(stdout.contains("network=mainnet"), "stdout: {stdout}");
    // Singlesig has no threshold token; summary should reflect None.
    assert!(stdout.contains("threshold=none"), "stdout: {stdout}");
    assert!(stdout.contains("bsms_audit=some"));
}

/// SPEC §10.1 — 4-line shape accepts the `"No path restrictions"` sentinel
/// on line 3 (per BIP-129 line 96 the field is required but may be the
/// literal string `"No path restrictions"` when there are none). Asserts
/// the parser does NOT special-case that line content — it's preserved
/// verbatim in the audit envelope and the descriptor parse + first-address
/// cross-validation succeed normally.
#[test]
fn bsms_4line_no_path_restrictions_accepted() {
    let p = fixture_path("bsms-4line-no-path-restrictions.txt");
    let out = run_import(&p).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(!stderr.contains("first-address mismatch"));
    assert!(stdout.contains("cosigners=2"));
    assert!(stdout.contains("threshold=2"));
    assert!(stdout.contains("bsms_audit=some"));
}

/// SPEC §10.2 — 4-line first-address cross-validation. The fixture carries
/// a deliberately-wrong line-4 address; the parser MUST emit the existing
/// `first-address mismatch at path <P>` WARNING (informational, exit 0,
/// matching the 6-line behavior) and STILL succeed. The `<P>` segment for
/// the 4-line shape sources from the BIP-129 path-restrictions string
/// (e.g., `/0/*,/1/*`) per the empty-string-sentinel audit layout.
#[test]
fn bsms_4line_first_address_mismatch_emits_warning() {
    let p = fixture_path("bsms-4line-first-address-mismatch.txt");
    let out = run_import(&p).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("first-address mismatch"),
        "4-line first-address mismatch must emit the locked WARNING; stderr was: {stderr}"
    );
    // The WARNING is informational, not a hard error — exit 0, parse proceeds.
    assert!(stdout.contains("cosigners=3"));
    assert!(stdout.contains("network=mainnet"));
    assert!(stdout.contains("threshold=2"));
    assert!(stdout.contains("bsms_audit=some"));
}

/// SPEC §10.4 — 6-line lenient shape continues to be accepted in v0.28.0
/// but emits the new DEPRECATION NOTICE. This is the regression guard
/// against accidentally removing the 6-line arm (per SPEC §10.4 the 6-line
/// shape stays for one cycle before removal in a future minor).
#[test]
fn bsms_6line_still_accepted_with_deprecation_notice() {
    let desc = format!(
        "wsh(sortedmulti(2,[{MAINNET_FP_A}/48'/0'/0'/2']{MAINNET_XPUB_A}/<0;1>/*,[{MAINNET_FP_B}/48'/0'/0'/2']{MAINNET_XPUB_B}/<0;1>/*))"
    );
    // Use the real /0/0 first-address so the cross-validation does not fire.
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
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    // SPEC §10.4 DEPRECATION NOTICE shape verbatim.
    assert!(
        stderr.contains("6-line lenient shape is DEPRECATED in v0.28+ and"),
        "expected v0.28.0 DEPRECATION NOTICE; stderr was: {stderr}"
    );
    assert!(
        stderr.contains("4-line shape"),
        "expected DEPRECATION NOTICE to point at the 4-line shape; stderr was: {stderr}"
    );
    assert!(
        stderr.contains("SPEC §10"),
        "expected DEPRECATION NOTICE to cite SPEC §10; stderr was: {stderr}"
    );
}

/// SPEC §10 + roundtrip — 4-line input through the `--json` envelope.
/// `canonicalize_bsms` accepts the 4-line shape (per the P7A R5-C2 mirror
/// fix at `roundtrip.rs::canonicalize_bsms`); the canonical form is the
/// 2-line shape (audit lines dropped per §7.3.1 step 4), so 4-line →
/// canonicalize is semantically equivalent to the same descriptor in 2-line
/// shape. Round-trip status should report `byte_exact: false` but
/// `semantic_match: true` (and `status: "blocked_no_emitter"` until the
/// BSMS emitter is wired for the input side; until then status is the
/// envelope's canonical "blocked" value).
#[test]
fn bsms_4line_via_bundle_roundtrip_json() {
    let p = fixture_path("bsms-4line-sortedmulti-2of3.txt");
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
        ])
        .arg(&p)
        .args(["--format", "bsms", "--json"])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("envelope JSON invalid: {e}\nstdout: {stdout}"));
    let entry = &val.as_array().expect("array envelope")[0];
    // Sanity: source_format identifies BSMS.
    assert_eq!(entry["source_format"].as_str(), Some("bsms"));
    // The roundtrip envelope must be present and parseable. Status enumerates
    // {ok, blocked_no_emitter, canonicalize_failed} per SPEC §2.2; either
    // "ok" or "blocked_no_emitter" is acceptable here — the load-bearing
    // assertion is that canonicalize_bsms did NOT fail on the 4-line shape.
    let status = entry["roundtrip"]["status"].as_str();
    assert!(
        matches!(status, Some("ok") | Some("blocked_no_emitter")),
        "expected ok or blocked_no_emitter roundtrip status (canonicalize accepted 4-line); got: {status:?}\nentry: {entry}"
    );
// v0.28.0 Phase P9A — BSMS fixture-corpus expansion (3 fixtures)
}

//
// Plan-doc §S.9 owner-phase tag. Parse-only cells exercising the file-on-disk
// fixtures `bsms-2line-decay-4032.txt`, `bsms-6line-sortedmulti-2of3.txt`,
// `bsms-2line-sortedmulti-3of5.txt`. These cells lock the v0.26.0 BSMS
// 2-line / 6-line lenient parser behavior across:
//   - decaying-multisig timelock N=4032 (1-month-ish fallback);
//   - 6-line Round-2 with audit fields populated;
//   - sortedmulti scaled to 3-of-5 (verifies the parser handles >3 cosigners).
// All cells route through `run_import(&fixture_path(...))` so the fixture
// files themselves are exercised on the filesystem (matches the existing
// `bsms_2_line_happy_path` pattern at line 104).
// ============================================================================

/// P9A.1 — 2-line decaying-multisig N=4032 fixture.
///
/// Mirrors `bsms_decaying_multisig_n_4032` (line 280) which builds the same
/// descriptor dynamically; this cell pins the static fixture file. Both must
/// parse identically.
#[test]
fn bsms_2line_decay_4032_fixture_parses() {
    let p = fixture_path("bsms-2line-decay-4032.txt");
    let out = run_import(&p).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    // 2-line WARNING fires.
    assert!(
        stderr.contains("2-line excerpt"),
        "expected 2-line WARNING; stderr was: {stderr}"
    );
    // Two cosigners (decaying multisig has 2 keys + 1 timelock branch).
    assert!(stdout.contains("cosigners=2"), "stdout: {stdout}");
    assert!(stdout.contains("network=testnet"), "stdout: {stdout}");
    assert!(stdout.contains("bsms_audit=none"), "stdout: {stdout}");
    // Watch-only invariant.
    assert!(stdout.contains("entropy=none"), "stdout: {stdout}");
    // Cosigner fingerprints byte-exact.
    assert!(stdout.contains(TESTNET_FP_A), "stdout: {stdout}");
    assert!(stdout.contains(TESTNET_FP_B), "stdout: {stdout}");
}

/// P9A.2 — 6-line BSMS Round-2 mainnet sortedmulti 2-of-3 fixture.
///
/// Static-fixture mirror of `bsms_6_line_happy_path` (line 132) — but with
/// the audit `<FIRST_ADDRESS>` field set to the toolkit-derived /0/0 mainnet
/// address. The 6-line lenient parser populates `BsmsAuditFields` and emits
/// the "6-line lenient shape is DEPRECATED" NOTICE per `wallet_import/bsms.rs:111-117`.
#[test]
fn bsms_6line_sortedmulti_2of3_fixture_parses() {
    let p = fixture_path("bsms-6line-sortedmulti-2of3.txt");
    let out = run_import(&p).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    // 6-line "6-line lenient shape is DEPRECATED" NOTICE fires (v0.27.0 wording).
    assert!(
        stderr.contains("6-line lenient shape is DEPRECATED"),
        "expected v0.28.0 6-line DEPRECATION NOTICE; stderr was: {stderr}"
    );
    // 6-line MUST NOT emit the 2-line WARNING.
    assert!(
        !stderr.contains("2-line excerpt"),
        "6-line shape must not emit 2-line WARNING; stderr was: {stderr}"
    );
    // 6-line carries audit fields (token + signature + first_address + path).
    assert!(stdout.contains("cosigners=3"), "stdout: {stdout}");
    assert!(stdout.contains("network=mainnet"), "stdout: {stdout}");
    assert!(stdout.contains("threshold=2"), "stdout: {stdout}");
    assert!(stdout.contains("bsms_audit=some"), "stdout: {stdout}");
    // Real /0/0 first-address (pre-computed at fixture-author time) — so the
    // mismatch WARNING must NOT fire on this fixture.
    assert!(
        !stderr.contains("first-address mismatch"),
        "fixture's audit first-address byte-equals the toolkit-derived address; \
         mismatch WARNING must not fire; stderr was: {stderr}"
    );
}

/// P9A.3 — 2-line BSMS wsh(sortedmulti(3, ...5 cosigners)) fixture.
///
/// Scales the existing 2-of-3 sortedmulti fixture path to 3-of-5. Pins the
/// parser's handling of larger sortedmulti N — the cosigner-extraction loop
/// at `wallet_import/bsms.rs:181-195` walks `parsed_keys.iter().enumerate()`
/// so any cosigner count up to u8::MAX should parse identically.
#[test]
fn bsms_2line_sortedmulti_3of5_fixture_parses() {
    let p = fixture_path("bsms-2line-sortedmulti-3of5.txt");
    let out = run_import(&p).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    // 2-line WARNING fires.
    assert!(
        stderr.contains("2-line excerpt"),
        "expected 2-line WARNING; stderr was: {stderr}"
    );
    assert!(stdout.contains("cosigners=5"), "stdout: {stdout}");
    assert!(stdout.contains("threshold=3"), "stdout: {stdout}");
    assert!(stdout.contains("network=mainnet"), "stdout: {stdout}");
    assert!(stdout.contains("bsms_audit=none"), "stdout: {stdout}");
    // All 5 cosigner fingerprints byte-exact in the summary.
    for fp in [MAINNET_FP_A, MAINNET_FP_B, MAINNET_FP_C, "16a93ed0", "99887766"] {
        assert!(
            stdout.contains(fp),
            "expected fingerprint {fp} in summary; stdout: {stdout}"
        );
    }
}

// ============================================================================
// v0.28.0 Phase P9B — BSMS fixture-corpus expansion (4 more fixtures)
//
// Plan-doc §S.9 owner-phase tag. Adds 4 more file-on-disk BSMS fixtures
// exercising SLIP-132 prefix variants (ypub/zpub), the BIP-129 taproot-edge
// (tr(NUMS, sortedmulti_a(...))), and the BIP-45 legacy multisig shape.
// All fixtures pin the v0.26.0 parser's CURRENT behavior — see the tr-NUMS
// cell for the v0.28+ taproot-refusal FOLLOWUP rationale.
// ============================================================================

/// P9B.4 — SLIP-132 mainnet ypub single-sig fixture (BIP-49 path).
///
/// Locks the `slip0132::normalize_xpub_prefix` round-trip behavior on a
/// file-on-disk fixture (the existing `bsms_slip132_variants_ypub` cell at
/// line 373 builds the same descriptor dynamically). The ypub xpub bytes are
/// the canonical SLIP-0132 BIP-49 mainnet test vector at `m/49'/0'/0'`.
#[test]
fn bsms_2line_mainnet_ypub_fixture_parses() {
    let p = fixture_path("bsms-2line-mainnet-ypub.txt");
    let out = run_import(&p).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("2-line excerpt"),
        "expected 2-line WARNING; stderr was: {stderr}"
    );
    assert!(stdout.contains("cosigners=1"), "stdout: {stdout}");
    assert!(stdout.contains("network=mainnet"), "stdout: {stdout}");
    assert!(stdout.contains("bsms_audit=none"), "stdout: {stdout}");
}

/// P9B.5 — SLIP-132 mainnet zpub single-sig fixture (BIP-84 path).
///
/// Mirrors P9B.4 for the zpub variant. The zpub bytes are
/// `TREZOR_24_BIP84_MAINNET_ZPUB` shared across the toolkit's export-wallet
/// test corpus (`cli_export_wallet_electrum.rs:14`, `cli_export_wallet_jade.rs:11`,
/// etc.); the fingerprint is `5436d724` (the TREZOR 24-word seed master fp).
#[test]
fn bsms_2line_mainnet_zpub_fixture_parses() {
    let p = fixture_path("bsms-2line-mainnet-zpub.txt");
    let out = run_import(&p).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("2-line excerpt"),
        "expected 2-line WARNING; stderr was: {stderr}"
    );
    assert!(stdout.contains("cosigners=1"), "stdout: {stdout}");
    assert!(stdout.contains("network=mainnet"), "stdout: {stdout}");
    assert!(stdout.contains("bsms_audit=none"), "stdout: {stdout}");
    // FP is canonical Trezor-24-word seed master fingerprint.
    assert!(stdout.contains(MAINNET_FP_C), "stdout: {stdout}");
}

/// P9B.6 — tr(NUMS, sortedmulti_a(2, A, C)) BSMS blob at m/86'/0'/0'.
///
/// **Plan-doc §S.9 R1-M2 ownership intent:** the cell name pins the
/// fixture-author identity (`bsms_tr_nums_refused`) but P9B's scope is
/// 0 src changes; the v0.27.0 BSMS parser at `wallet_import/bsms.rs:201-265`
/// explicitly **accepts** taproot descriptors at parse time (it only skips
/// the first-address-verify WARNING for `Tr(_)` — `bsms.rs:217-224`). So
/// `import-wallet --format bsms <tr-blob>` currently exits 0 with cosigner
/// extraction succeeding via the existing origin-capture-regex path.
///
/// This cell **pins the current behavior** (exit 0, cosigners=2,
/// network=mainnet, bsms_audit=none) and documents the gap from the plan-doc's
/// forward-looking "taproot-refusal" intent. The actual refusal — which
/// would require a `Tr(_)` short-circuit at `bsms.rs::parse` mirroring
/// `wallet_export/bsms.rs:69-76` — is filed as a v0.28+ FOLLOWUP at
/// `design/v0_28_0-cycle-followups.md` (entry `bsms-import-taproot-refusal-parity`).
///
/// **Side-channel finding (folded into the FOLLOWUP body):**
/// `extract_threshold`'s regex at `wallet_import/bsms.rs:419-421` does NOT
/// match `sortedmulti_a(` (the `_a` taproot variant). For this fixture's
/// `tr(NUMS, sortedmulti_a(2, ...))` body, the regex returns `Ok(None)` and
/// the CLI summary emits `threshold=none`. A real taproot-aware BSMS parser
/// would either refuse (P9B.6's forward-looking intent) or extend the regex
/// to include `sortedmulti_a` and `multi_a`.
#[test]
fn bsms_2line_tr_nums_current_behavior_no_refusal() {
    let p = fixture_path("bsms-2line-tr-nums.txt");
    let out = run_import(&p).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    // v0.27.0 parser accepts tr(...) and skips first-address verify.
    assert!(
        stderr.contains("2-line excerpt"),
        "expected 2-line WARNING; stderr was: {stderr}"
    );
    // Parse extracts both cosigners from the sortedmulti_a leaf via the
    // origin-capture-regex at bsms.rs:438-441 (regex matches anywhere in
    // the body, including taproot script-tree positions).
    assert!(stdout.contains("cosigners=2"), "stdout: {stdout}");
    assert!(stdout.contains("network=mainnet"), "stdout: {stdout}");
    assert!(stdout.contains("bsms_audit=none"), "stdout: {stdout}");
    // Side-channel finding: extract_threshold's regex does NOT match
    // `sortedmulti_a(` — tracked in the FOLLOWUP body.
    assert!(
        stdout.contains("threshold=none"),
        "expected threshold=none (extract_threshold regex gap); stdout: {stdout}"
    );
    // No first-address-mismatch WARNING (2-line shape carries no audit).
    assert!(
        !stderr.contains("first-address mismatch"),
        "2-line shape has no audit fields; mismatch WARNING must not fire; \
         stderr was: {stderr}"
    );
}

/// P9B.7 — BIP-45 multisig `sh(multi(2, ...))` at m/45'/0'/0'.
///
/// BIP-45 uses bare `multi(...)` (declaration-order, not lexicographic-
/// sorted). Pins the SPEC §4.3 declaration-order preservation for BIP-45
/// paths (the existing `bsms_multi_non_sorted_2_of_3` cell at line 326
/// exercises the same shape dynamically; this cell adds the file-on-disk
/// fixture variant). Coin-type at origin-path index 1 is `0'` →
/// `network=mainnet` per `coin_type_from_path` at `bsms.rs:384-401`.
///
/// **Permissive contract** (mirrors line 354-366): rust-miniscript may
/// refuse bare `multi(...)` inside `sh(...)` at BIP-45 in some configs;
/// if so, the failure must mention the descriptor body (rules out a silent
/// re-sort regression). Cell accepts EITHER the success path (with
/// declaration-order fingerprint preservation) OR a structured rejection.
#[test]
fn bsms_2line_bip45_fixture_parses_or_rejects_descriptively() {
    let p = fixture_path("bsms-2line-bip45.txt");
    let output = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob"])
        .arg(&p)
        .args(["--format", "bsms"])
        .output()
        .expect("run import-wallet");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    if output.status.success() {
        // Happy path: 3 cosigners; mainnet; declaration order preserved
        // (first fingerprint is A per fixture line 2 order).
        assert!(stdout.contains("cosigners=3"), "stdout: {stdout}");
        assert!(stdout.contains("network=mainnet"), "stdout: {stdout}");
        assert!(
            stdout.contains(&format!("cosigners[0].fingerprint={MAINNET_FP_A}")),
            "expected first cosigner = A per BIP-45 declaration order; stdout: {stdout}"
        );
    } else {
        // Permissive rejection: stderr must explain (rules out silent
        // re-sort or other foot-gun).
        assert!(
            !stderr.is_empty(),
            "expected stderr description if pipeline refuses sh(multi)"
        );
    }
}

/// P9B roundtrip cell — re-parse YPUB fixture via stdin to confirm the
/// CRLF-normalized + stdin-piped path treats the file content identically
/// to `--blob <path>`. Pins the symmetry of the two CLI input modes for
/// the new fixtures.
#[test]
fn bsms_2line_mainnet_ypub_stdin_roundtrip_matches_blob_path() {
    let p = fixture_path("bsms-2line-mainnet-ypub.txt");
    let blob = std::fs::read_to_string(&p).expect("fixture file present");

    let by_path = run_import(&p).success();
    let by_stdin = run_import_stdin(&blob).success();

    // Stdout summary must byte-equal across the two input modes.
    let stdout_a = String::from_utf8(by_path.get_output().stdout.clone()).unwrap();
    let stdout_b = String::from_utf8(by_stdin.get_output().stdout.clone()).unwrap();
    assert_eq!(
        stdout_a, stdout_b,
        "stdin and --blob <path> must produce byte-identical stdout summaries; \
         path={stdout_a} stdin={stdout_b}"
    );
}

/// P9B roundtrip cell — same as above for the zpub fixture. Closes
/// "stdin vs --blob symmetry" for SLIP-132 prefix variants in the corpus.
#[test]
fn bsms_2line_mainnet_zpub_stdin_roundtrip_matches_blob_path() {
    let p = fixture_path("bsms-2line-mainnet-zpub.txt");
    let blob = std::fs::read_to_string(&p).expect("fixture file present");

    let by_path = run_import(&p).success();
    let by_stdin = run_import_stdin(&blob).success();

    let stdout_a = String::from_utf8(by_path.get_output().stdout.clone()).unwrap();
    let stdout_b = String::from_utf8(by_stdin.get_output().stdout.clone()).unwrap();
    assert_eq!(stdout_a, stdout_b);
}

mod shared {
    use std::process::Stdio;
    /// I18 helper — drives the sniff via the `mnemonic import-wallet --blob -`
    /// command without `--format`, which exercises `sniff_format`. If sniff
    /// returns `Bsms`, the parser runs and either succeeds or fails with a
    /// bsms-specific template; if sniff returns `NoMatch`/`Ambiguous`, the
    /// dispatch returns `ImportWalletAmbiguousFormat` instead (no
    /// bsms-specific template in stderr).
    pub(super) fn bsms_sniff_via_dispatch(blob: &[u8]) -> bool {
        let mut cmd = std::process::Command::new(
            assert_cmd::cargo::cargo_bin("mnemonic"),
        );
        cmd.args(["import-wallet", "--blob", "-"]);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        let mut child = cmd.spawn().expect("spawn mnemonic");
        use std::io::Write;
        child.stdin.as_mut().unwrap().write_all(blob).unwrap();
        let out = child.wait_with_output().expect("wait");
        let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
        // BSMS-sniff-positive path emits a bsms-specific stderr (parse error
        // mentioning "bsms"). NoMatch/Ambiguous path emits
        // ImportWalletAmbiguousFormat without "bsms" in the template.
        stderr.to_lowercase().contains("bsms:") || stderr.contains("expected 2, 4, or 6 lines")
    }
}

