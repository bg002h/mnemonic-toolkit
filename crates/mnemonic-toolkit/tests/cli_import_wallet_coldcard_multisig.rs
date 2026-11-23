//! v0.28.0 Phase P4C — Coldcard multisig text-file parser integration tests.
//!
//! Per `design/SPEC_wallet_import_v0_28_0.md` §11.4. Tests the library
//! boundary via the CLI scaffold (`cmd/import_wallet.rs`) extended in P4C
//! to dispatch `--format coldcard-multisig` to `ColdcardMultisigParser`
//! and to wire the auto-sniff path for the text-shape blob.
//!
//! Self-contained: no dependency on adjacent repos or external network.
//! Uses the 4 fixtures introduced in P4B:
//! - `coldcard-ms-2of3-p2wsh-with-xfp.txt` — happy-path with leading XFP header.
//! - `coldcard-ms-2of3-p2wsh-no-xfp.txt` — happy-path without leading XFP header.
//! - `coldcard-ms-3of5-p2wsh.txt` — 3-of-5 multisig.
//! - `coldcard-ms-malformed-missing-format.txt` — refused with diagnostic.

use assert_cmd::Command;

const FIX_2OF3_WITH_XFP: &str = "tests/fixtures/wallet_import/coldcard-ms-2of3-p2wsh-with-xfp.txt";
const FIX_2OF3_NO_XFP: &str = "tests/fixtures/wallet_import/coldcard-ms-2of3-p2wsh-no-xfp.txt";
const FIX_3OF5: &str = "tests/fixtures/wallet_import/coldcard-ms-3of5-p2wsh.txt";
const FIX_MALFORMED: &str = "tests/fixtures/wallet_import/coldcard-ms-malformed-missing-format.txt";

/// cycle-13a (H14 depth-gated truth table): a genuine DEPTH-0 master xpub.
/// The Row-2 disagreement WARNING is only meaningful at depth 0, where the
/// xpub's own `fingerprint()` IS the master fp and is comparable to a supplied
/// XFP. At depth>0 a supplied XFP is accepted silently (H14-c), so the prior
/// depth-4 blobs in the two divergence-warning cells below would go silent.
/// `XPUB_D0_A.fingerprint() == 57ACB302` (pinned against the bitcoin crate in
/// the `coldcard_multisig::tests::depth0_const_fingerprints_pinned` unit test).
const XPUB_D0_A: &str = "xpub661MyMwAqRbcGQ5dEWgzwBWpcFA5Uc2TKjZy6gqBoHgMGBKn91Q7ooXXCk2cdjU6nh1GW5tF7ttjKiYg2RJ5ybBZscgMqLE7RevfHn4J1jS";
const FP_D0_A: &str = "57ACB302";
/// Second depth-0 master xpub (for divergent-path round-trip tests).
const XPUB_D0_B: &str = "xpub661MyMwAqRbcF4AYTpoZvFuiLUyBtTmtVhoUVutzfJzeCFvNFjRcdtLLaWgDb7gwHmLBTV6gZf4T9rqSnxcu8hcmLigphSiFioYqVFcRSEZ";

/// Run `mnemonic import-wallet --blob <path> --format coldcard-multisig`.
fn run_explicit(path: &str) -> assert_cmd::assert::Assert {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            path,
            "--format",
            "coldcard-multisig",
        ])
        .assert()
}

/// Run `mnemonic import-wallet --blob <path>` (no `--format`); auto-sniff path.
fn run_autosniff(path: &str) -> assert_cmd::assert::Assert {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", path])
        .assert()
}

/// Run with a synthetic blob piped on stdin.
fn run_stdin(blob: &str) -> assert_cmd::assert::Assert {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "coldcard-multisig",
        ])
        .write_stdin(blob.to_string())
        .assert()
}

// ============================================================================
// §11.4 — happy-path parse via explicit --format coldcard-multisig
// ============================================================================

#[test]
fn coldcard_ms_2of3_with_xfp_happy_path() {
    let out = run_explicit(FIX_2OF3_WITH_XFP).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stdout.contains("bundles=1"), "stdout: {stdout}");
    assert!(
        stdout.contains("bundles[0].cosigners=3"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("bundles[0].network=mainnet"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("bundles[0].threshold=2"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("bundles[0].bsms_audit=none"),
        "stdout: {stdout}"
    );
    assert!(
        stdout.contains("bundles[0].entropy=none"),
        "stdout: {stdout}"
    );
    // Coldcard-multisig provenance is neither bsms nor bitcoin-core; the
    // `source_metadata` accessor returns None per ImportProvenance enum semantics.
    assert!(
        stdout.contains("bundles[0].source_metadata=none"),
        "stdout: {stdout}"
    );
    // Silent stderr on the with-xfp fixture (row 1 of truth table for all cosigners).
    assert!(
        !stderr.contains("warning:"),
        "stderr must be silent on with-xfp happy-path; got: {stderr}"
    );
}

#[test]
fn coldcard_ms_2of3_no_xfp_happy_path() {
    let out = run_explicit(FIX_2OF3_NO_XFP).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stdout.contains("bundles[0].cosigners=3"));
    assert!(stdout.contains("bundles[0].threshold=2"));
    assert!(!stderr.contains("warning:"));
}

#[test]
fn coldcard_ms_3of5_happy_path() {
    let out = run_explicit(FIX_3OF5).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stdout.contains("bundles[0].cosigners=5"));
    assert!(stdout.contains("bundles[0].threshold=3"));
    assert!(!stderr.contains("warning:"));
}

// ============================================================================
// §11.4 — auto-sniff: blob with Name/Policy/Format text-shape → ColdcardMultisig
// ============================================================================

#[test]
fn coldcard_ms_autosniff_2of3_with_xfp_dispatches_correctly() {
    let out = run_autosniff(FIX_2OF3_WITH_XFP).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=1"), "stdout: {stdout}");
}

#[test]
fn coldcard_ms_autosniff_3of5_dispatches_correctly() {
    let out = run_autosniff(FIX_3OF5).success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles[0].cosigners=5"));
}

// ============================================================================
// §11.4 — refusal on malformed input
// ============================================================================

#[test]
fn coldcard_ms_malformed_missing_format_refused() {
    let out = run_explicit(FIX_MALFORMED).failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("coldcard-multisig"),
        "stderr must cite format; got: {stderr}"
    );
    assert!(
        stderr.contains("missing `Format:` header"),
        "stderr must cite missing field; got: {stderr}"
    );
    // SPEC §2.4 row: ImportWalletParse → exit code 2.
    let code = out.get_output().status.code().expect("exit code present");
    assert_eq!(code, 2, "ImportWalletParse must exit 2; got {code}");
}

// ============================================================================
// SPEC §11.4.1 — xfp-divergence WARNING cell (row 2 of the truth table)
// ============================================================================

#[test]
fn coldcard_ms_xfp_header_divergence_warns_byte_exact_template() {
    // Synthetic blob: top-level `XFP: DEADBEEF` disagrees with computed
    // fingerprint `57ACB302` from the DEPTH-0 master cosigner xpub. The parser
    // must emit the SPEC §11.4.1 row-2 WARNING (byte-exact template) AND
    // succeed (header value is authoritative per the truth table).
    // cycle-13a: re-pointed to a depth-0 master xpub so the disagreement
    // warning stays meaningful (depth>0 + supplied XFP is silent under H14-c).
    let blob = format!(
        "Name: T\n\
Policy: 1 of 1\n\
Derivation: m/48'/0'/0'/2'\n\
Format: P2WSH\n\
XFP: DEADBEEF\n\
\n\
{XPUB_D0_A}\n"
    );
    let out = run_stdin(&blob).success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    // SPEC §11.4.1 row-2 WARNING template (verbatim):
    //   warning: import-wallet: coldcard-multisig: xfp header `XFP: <hex>`
    //   disagrees with computed fingerprint `<hex>` from cosigner xpub;
    //   using blob-supplied header value as authoritative
    assert!(
        stderr.contains("warning: import-wallet: coldcard-multisig: xfp header"),
        "row-2 WARNING template missing; got: {stderr}"
    );
    assert!(
        stderr.contains("`XFP: DEADBEEF`"),
        "WARNING must cite blob's XFP header value; got: {stderr}"
    );
    assert!(
        stderr.contains(&format!("`{FP_D0_A}`")),
        "WARNING must cite computed fingerprint; got: {stderr}"
    );
    assert!(
        stderr.contains("using blob-supplied header value as authoritative"),
        "WARNING must cite authoritative clause; got: {stderr}"
    );
    // Parse still succeeds (row 2 uses header value).
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(stdout.contains("bundles=1"), "stdout: {stdout}");
}

#[test]
fn coldcard_ms_per_cosigner_xfp_divergence_warns_per_cosigner() {
    // Synthetic blob: per-cosigner `<XFP>: <xpub>` form where the XFP
    // prefix CAFEBABE disagrees with computed `57ACB302`. Same WARNING
    // template applies (the truth table is per-cosigner, not just per-header).
    // cycle-13a: re-pointed to a depth-0 master xpub (depth>0 + per-line XFP
    // is silent under H14-c).
    let blob = format!(
        "Name: T\n\
Policy: 1 of 1\n\
Derivation: m/48'/0'/0'/2'\n\
Format: P2WSH\n\
\n\
CAFEBABE: {XPUB_D0_A}\n"
    );
    let out = run_stdin(&blob).success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(stderr.contains("warning: import-wallet: coldcard-multisig: xfp header"));
    assert!(stderr.contains("CAFEBABE"));
    assert!(stderr.contains(FP_D0_A));
}

// ============================================================================
// SPEC §6.1 — format mismatch (explicit --format vs sniffed)
// ============================================================================

#[test]
fn coldcard_ms_format_mismatch_against_bsms_blob_rejected() {
    // BSMS blob with `--format coldcard-multisig` must surface
    // ImportWalletFormatMismatch (exit 1) per SPEC §6.1.
    let bsms_blob = "BSMS 1.0\nwsh(sortedmulti(2,[deadbeef/48'/0'/0'/2']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*,[cafebabe/48'/0'/0'/2']xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6/<0;1>/*))#00000000\n";
    let out = run_stdin(bsms_blob).failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("coldcard-multisig") && stderr.contains("bsms"),
        "stderr must cite mismatch between coldcard-multisig (supplied) and bsms (sniffed); \
         got: {stderr}"
    );
}

#[test]
fn coldcard_ms_format_mismatch_against_bitcoin_core_blob_rejected() {
    let core_blob = r#"{"wallet_name":"x","descriptors":[{"desc":"wpkh([deadbeef/84'/0'/0']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*)#00000000"}]}"#;
    let out = run_stdin(core_blob).failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("coldcard-multisig") && stderr.contains("bitcoin-core"),
        "stderr must cite mismatch; got: {stderr}"
    );
}

// ============================================================================
// --json envelope shape for coldcard-multisig
// ============================================================================

#[test]
fn coldcard_ms_json_envelope_emits_canonical_shape() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            FIX_2OF3_NO_XFP,
            "--format",
            "coldcard-multisig",
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("envelope is JSON");
    let arr = v.as_array().expect("top-level envelope is an array");
    assert_eq!(arr.len(), 1, "single multisig wallet → single envelope");
    let env = &arr[0];
    assert_eq!(env["schema_version"], "1");
    assert_eq!(env["source_format"], "coldcard-multisig");
    let roundtrip = &env["roundtrip"];
    assert_eq!(
        roundtrip["status"], "ok",
        "canonicalize succeeds on the no-xfp-header fixture"
    );
    assert_eq!(
        roundtrip["semantic_match"], true,
        "semantic_match always true on canonicalize-ok"
    );
}

// ============================================================================
// SPEC §6.2 — auto-sniff dispatch via sniff_format → SniffOutcome::ColdcardMultisig
// ============================================================================

#[test]
fn coldcard_ms_autosniff_does_not_co_fire_with_bsms_or_core() {
    // Sanity: the 4 fixtures all auto-sniff to `coldcard-multisig` (not
    // `Ambiguous`, not `NoMatch`). This is a per-parser sniff signature
    // smoke; the cross-format-matrix Phase P11 will exhaustively verify.
    for fix in [FIX_2OF3_WITH_XFP, FIX_2OF3_NO_XFP, FIX_3OF5] {
        let _out = run_autosniff(fix).success();
    }
}

// ============================================================================
// cycle-13a #16 (P4 / I-1) — round-trip-verify on a DIVERGENT-path blob passes
// (the canonicalizer preserves per-cosigner paths instead of re-collapsing
// onto cosigner-0's path → no spurious mismatch / false pass). The roundtrip
// canonicalizer is invoked unconditionally; the `--json` envelope's
// `roundtrip.semantic_match` reflects it. Depth-0 master xpubs carry no XFP
// (no refusal at re-parse, M-1) under DISTINCT per-cosigner `Derivation:` lines.
// ============================================================================

#[test]
fn roundtrip_verify_divergent_coldcard_multisig_passes() {
    let blob = format!(
        "Name: T\n\
Policy: 1 of 2\n\
Format: P2WSH\n\
Derivation: m/48'/0'/0'/2'\n\
{XPUB_D0_A}\n\
Derivation: m/48'/0'/1'/2'\n\
{XPUB_D0_B}\n"
    );
    // The divergent blob imports cleanly via the LIVE round-trip canonicalize
    // dispatch (cmd/import_wallet.rs).
    let json_out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            "-",
            "--format",
            "coldcard-multisig",
            "--json",
        ])
        .write_stdin(blob.clone())
        .assert()
        .success();
    let jstdout = String::from_utf8(json_out.get_output().stdout.clone()).unwrap();
    let v: serde_json::Value = serde_json::from_str(&jstdout).expect("envelope is JSON");
    let roundtrip = &v[0]["roundtrip"];
    assert_eq!(
        roundtrip["status"], "ok",
        "divergent canonicalize must succeed; envelope: {jstdout}"
    );

    // CANONICAL-FORM fidelity: the canonical re-emit (the `output` side of the
    // round-trip diff) must carry BOTH divergent `Derivation:` paths. Pre-P4
    // the canonicalizer stamped cosigner-0's path on all cosigners, so the
    // output side DROPPED `m/48'/0'/1'/2'` entirely → this is the RED. Parse
    // the unified diff and reconstruct the output side (context ` ` lines +
    // `+` additions, minus `-` removals).
    let diff = roundtrip["diff"].as_str().expect("roundtrip diff present");
    let output_side: String = diff
        .lines()
        .filter(|l| !l.starts_with("---") && !l.starts_with("+++") && !l.starts_with("@@"))
        .filter(|l| !l.starts_with('-'))
        .map(|l| l.strip_prefix(['+', ' ']).unwrap_or(l))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        output_side.contains("Derivation: m/48'/0'/0'/2'"),
        "canonical output must preserve cosigner path m/48'/0'/0'/2'; output side:\n{output_side}"
    );
    assert!(
        output_side.contains("Derivation: m/48'/0'/1'/2'"),
        "canonical output must preserve the DISTINCT path m/48'/0'/1'/2' (NOT collapse to \
         cosigner-0's path); output side:\n{output_side}"
    );

    // The resolved descriptor must likewise carry both per-cosigner origins.
    let descriptor = v[0]["bundle"]["descriptor"]
        .as_str()
        .expect("descriptor present");
    assert!(
        descriptor.contains("/48'/0'/0'/2']") && descriptor.contains("/48'/0'/1'/2']"),
        "both divergent cosigner paths must survive into the descriptor; got:\n{descriptor}"
    );
}

// ---------------------------------------------------------------------------
// FIX-tk-coldcard-multisig-duplicate-headers (bughunt-1119 bh-tk-I1): rows R1-R18.
// Blobs are built from the committed export fixture, as
// `design/fix-tk-coldcard-dup-headers-red.sh` does.
// ---------------------------------------------------------------------------

const DUP_FX: &str = "tests/export_wallet/coldcard_multisig_2of3_wsh.txt";
const DUP_JADE: &str = "tests/fixtures/wallet_import/jade-multisig-2of3-p2wsh.json";
const DUP_X1: &str = "xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx";
const DUP_X2: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";
const DUP_X3: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";

fn dup_base() -> String {
    std::fs::read_to_string(DUP_FX).expect("fixture")
}

/// Per-cosigner shape with a bare first xpub; `xfp_lines` sits before the first `Derivation:`.
fn dup_bare(xfp_lines: &str) -> String {
    format!(
        "Name: bare-xfp\nPolicy: 2 of 3\nFormat: P2WSH\n{xfp_lines}Derivation: m/48'/0'/0'/2'\n{DUP_X1}\nDerivation: m/48'/0'/1'/2'\n28645006: {DUP_X2}\nDerivation: m/48'/0'/2'/2'\nB8688DF1: {DUP_X3}\n"
    )
}

fn dup_percos() -> String {
    format!(
        "Name: per-cosigner\nPolicy: 2-of-3\nFormat: P2WSH\nDerivation: m/48'/0'/0'/2'\n5436D724: {DUP_X1}\nDerivation: m/48'/0'/1'/2'\n28645006: {DUP_X2}\nDerivation: m/48'/0'/2'/2'\nB8688DF1: {DUP_X3}\n"
    )
}

/// Assert exit 2, empty stdout, and `detail` in stderr behind the standard prefix.
fn dup_refused(assert: assert_cmd::assert::Assert, detail: &str) {
    let out = assert.code(2).get_output().clone();
    assert!(
        out.stdout.is_empty(),
        "stdout must be empty, got: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let err = String::from_utf8_lossy(&out.stderr);
    let want = format!("error: import-wallet: coldcard-multisig: parse error: {detail}");
    assert!(
        err.contains(&want),
        "stderr must contain `{want}`; got:\n{err}"
    );
}

/// Origin paths (`48'/0'/N'/2'`) of the descriptor of a successful `--json` import, in key order.
fn dup_paths(assert: assert_cmd::assert::Assert) -> Vec<String> {
    let out = assert.success().get_output().clone();
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json");
    let d = v[0]["bundle"]["descriptor"].as_str().expect("descriptor");
    d.split('[')
        .skip(1)
        .map(|s| {
            let o = &s[..s.find(']').unwrap()];
            o[o.find('/').unwrap() + 1..].to_string()
        })
        .collect()
}

fn dup_json(blob: &str) -> assert_cmd::assert::Assert {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--format",
            "coldcard-multisig",
            "--blob",
            "-",
            "--json",
        ])
        .write_stdin(blob.to_string())
        .assert()
}

const P2: &str = "repeated `Policy:` header (first given at line 2)";

#[test]
fn dup_headers_r1_policy_conflict_appended() {
    let blob = format!("{}Policy: 1 of 3\n", dup_base());
    dup_refused(run_stdin(&blob), &format!("line 8: {P2}"));
}

#[test]
fn dup_headers_r2_policy_conflict_prepended() {
    let blob = format!("Policy: 1 of 3\n{}", dup_base());
    dup_refused(
        run_stdin(&blob),
        "line 3: repeated `Policy:` header (first given at line 1)",
    );
}

#[test]
fn dup_headers_r3_policy_conflict_autosniff() {
    let blob = format!("{}Policy: 1 of 3\n", dup_base());
    let a = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", "-", "--json"])
        .write_stdin(blob)
        .assert();
    dup_refused(a, &format!("line 8: {P2}"));
}

#[test]
fn dup_headers_r4_policy_conflict_jade() {
    let inner = format!("{}Policy: 1 of 3\n", dup_base());
    let mut j: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(DUP_JADE).unwrap()).unwrap();
    j["multisig_file"] = serde_json::Value::String(inner);
    let dir = tempfile::tempdir().unwrap();
    let p = dir.path().join("jade.json");
    std::fs::write(&p, j.to_string()).unwrap();
    let a = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--format", "jade", "--blob"])
        .arg(&p)
        .arg("--json")
        .assert();
    dup_refused(a, &format!("line 8: {P2}"));
}

#[test]
fn dup_headers_r5_format_conflict_appended() {
    let blob = format!("{}Format: P2SH-P2WSH\n", dup_base());
    dup_refused(
        run_stdin(&blob),
        "line 8: repeated `Format:` header (first given at line 4)",
    );
}

#[test]
fn dup_headers_r6_format_conflict_prepended() {
    let blob = format!("Format: P2SH-P2WSH\n{}", dup_base());
    dup_refused(
        run_stdin(&blob),
        "line 5: repeated `Format:` header (first given at line 1)",
    );
}

#[test]
fn dup_headers_r7_xfp_conflict() {
    let blob = dup_bare("XFP: 0000AAAA\nXFP: 0000BBBB\n");
    dup_refused(
        run_stdin(&blob),
        "line 5: repeated `XFP:` header (first given at line 4)",
    );
}

#[test]
fn dup_headers_r8_name_conflict() {
    let blob = format!("{}Name: other-wallet\n", dup_base());
    dup_refused(
        run_stdin(&blob),
        "line 8: repeated `Name:` header (first given at line 1)",
    );
}

#[test]
fn dup_headers_r9_policy_identical_repeat() {
    let blob = format!("{}Policy: 2 of 3\n", dup_base());
    dup_refused(run_stdin(&blob), &format!("line 8: {P2}"));
}

#[test]
fn dup_headers_r10_policy_repeat_other_spelling() {
    let blob = format!("{}Policy: 2-of-3\n", dup_base());
    dup_refused(run_stdin(&blob), &format!("line 8: {P2}"));
}

#[test]
fn dup_headers_r11_policy_lowercase_key_repeat() {
    let blob = format!("{}policy: 1 of 3\n", dup_base());
    dup_refused(run_stdin(&blob), &format!("line 8: {P2}"));
}

#[test]
fn dup_headers_r12_control_original_fixture() {
    let out = dup_json(&dup_base()).success().get_output().clone();
    assert!(String::from_utf8_lossy(&out.stdout).contains("sortedmulti(2,"));
}

#[test]
fn dup_headers_r13_control_per_cosigner_derivation() {
    assert_eq!(
        dup_paths(dup_json(&dup_percos())),
        ["48'/0'/0'/2'", "48'/0'/1'/2'", "48'/0'/2'/2'"]
    );
}

#[test]
fn dup_headers_r14_control_xfp_single() {
    let out = dup_json(&dup_bare("XFP: 0000AAAA\n"))
        .success()
        .get_output()
        .clone();
    assert!(String::from_utf8_lossy(&out.stdout).contains("[0000aaaa/48'/0'/0'/2']"));
}

#[test]
fn dup_headers_r15_derivation_twice_before_cosigners() {
    let base = dup_base();
    let mut lines: Vec<&str> = base.lines().collect();
    lines.insert(3, "Derivation: m/48'/0'/9'/2'");
    let blob = format!("{}\n", lines.join("\n"));
    dup_refused(
        run_stdin(&blob),
        "line 4: `Derivation:` line follows the one at line 3 with no cosigner line between them",
    );
}

#[test]
fn dup_headers_r16_derivation_lowercase_after_shared() {
    let base = dup_base();
    let mut lines: Vec<&str> = base.lines().collect();
    lines.insert(3, "derivation: m/48'/0'/9'/2'");
    let blob = format!("{}\n", lines.join("\n"));
    dup_refused(
        run_stdin(&blob),
        "line 4: `derivation:` must be spelled `Derivation:`",
    );
}

#[test]
fn dup_headers_r17_derivation_lowercase_per_cosigner() {
    let blob = dup_percos().replacen(
        "\nDerivation: m/48'/0'/1'/2'",
        "\nderivation: m/48'/0'/1'/2'",
        1,
    );
    dup_refused(
        run_stdin(&blob),
        "line 6: `derivation:` must be spelled `Derivation:`",
    );
}

#[test]
fn dup_headers_r18_render_export_path_change_carries_forward() {
    let blob = format!(
        "Name: render-export\nPolicy: 2 of 3\nFormat: P2WSH\nDerivation: m/48'/0'/0'/2'\n5436D724: {DUP_X1}\nDerivation: m/48'/0'/1'/2'\n28645006: {DUP_X2}\nB8688DF1: {DUP_X3}\n"
    );
    assert_eq!(
        dup_paths(dup_json(&blob)),
        ["48'/0'/0'/2'", "48'/0'/1'/2'", "48'/0'/1'/2'"]
    );
}

#[test]
fn dup_headers_x1_case_variants_other_headers() {
    for (k, canon, first) in [("name", "Name", 1), ("format", "Format", 4)] {
        let blob = format!("{}{k}: z\n", dup_base());
        dup_refused(
            run_stdin(&blob),
            &format!("line 8: repeated `{canon}:` header (first given at line {first})"),
        );
    }
}

#[test]
fn dup_headers_x2_xfp_lowercase_repeat() {
    let blob = dup_bare("XFP: 0000AAAA\nxfp: 0000BBBB\n");
    dup_refused(
        run_stdin(&blob),
        "line 5: repeated `XFP:` header (first given at line 4)",
    );
}

#[test]
fn dup_headers_x4_bare_xpub_takes_current_path() {
    let blob = format!(
        "Name: b\nPolicy: 2 of 3\nFormat: P2WSH\nXFP: 0000AAAA\nDerivation: m/48'/0'/0'/2'\n5436D724: {DUP_X1}\nDerivation: m/48'/0'/1'/2'\n28645006: {DUP_X2}\n{DUP_X3}\n"
    );
    assert_eq!(
        dup_paths(dup_json(&blob)),
        ["48'/0'/0'/2'", "48'/0'/1'/2'", "48'/0'/1'/2'"]
    );
}
