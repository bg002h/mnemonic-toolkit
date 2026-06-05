//! v0.43.0 — `mnemonic restore` (single-sig core, Phase 1).
//!
//! Watch-only restore document: master fingerprint + CONFIRM line, then per-type
//! concrete descriptor + first receive address(es) for bip44/49/84/86 (or a
//! single `--template`). Optional `--expect-fingerprint`/`--expect-xpub`
//! reference → mismatch is exit 4 `RestoreMismatch` (no descriptors) unless
//! `--allow-mismatch`; no reference → UNVERIFIED banner.
//!
//! NEVER emits private key material (watch-only-out): a negative test greps the
//! whole output for `xprv`/`tprv` and asserts absence.

use assert_cmd::Command;

// Trezor 12-word "abandon ... about" reference seed. Master fingerprint
// `73c5da0a` is path-independent (master xpub fingerprint, not a derived-account
// fingerprint) — asserted in-tree at `cli_export_wallet.rs:27`.
const TREZOR_12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const FP_NO_PP: &str = "73c5da0a";

// bip84 account 0 xpub (m/84'/0'/0') for the no-passphrase seed.
const ACCT_XPUB_BIP84: &str = "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V";
// bip84 single-sig multipath descriptor (with #checksum) for the no-pp seed.
const DESC_BIP84: &str = "wpkh([73c5da0a/84'/0'/0']xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V/<0;1>/*)#hpg6d6w2";
// bip84 first receive address (m/84'/0'/0'/0/0) — the canonical BIP-84 vector.
const FIRST_RECV_BIP84: &str = "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu";

// ms1 entr card for the no-pp seed (generated via `convert --to ms1 --template
// bip84` at write time; restore decodes it back to the same entropy/fingerprint).
const MS1_NO_PP: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
// SeedQR digit-string for the no-pp seed (generated via `seedqr encode`).
const SEEDQR_NO_PP: &str = "000000000000000000000000000000000000000000000003";

// Japanese `mnem` ms1 card (carries the wire language on-chain). Its entropy is
// 16 zero bytes; derived as a JAPANESE phrase the fingerprint is `0ed2c5a4`
// (NOT `73c5da0a`), proving the wire language — not English — drives PBKDF2.
const MS1_MNEM_JP: &str = "ms10entrsqgqsqqqqqqqqqqqqqqqqqqqqqqqqqj9tawneveyd9j";

fn bin() -> Command {
    Command::cargo_bin("mnemonic").unwrap()
}

/// Re-derive a master fingerprint independently via `convert --to fingerprint`,
/// so the restore expected value is proven from source (per
/// `feedback_recapture_golden_only_when_current_correct`), not asserted from
/// memory.
fn fingerprint_via_convert(phrase: &str, passphrase: Option<&str>) -> String {
    let mut cmd = bin();
    cmd.args([
        "convert",
        "--from",
        &format!("phrase={phrase}"),
        "--to",
        "fingerprint",
        "--template",
        "bip84",
    ]);
    if let Some(pp) = passphrase {
        cmd.args(["--passphrase", pp]);
    }
    let out = cmd.output().expect("convert spawn");
    assert!(out.status.success(), "convert --to fingerprint failed");
    String::from_utf8(out.stdout)
        .unwrap()
        .trim()
        .trim_start_matches("fingerprint:")
        .trim()
        .to_string()
}

// ---------------------------------------------------------------------------
// 1.2 smoke + 1.4 exact descriptor/address
// ---------------------------------------------------------------------------

#[test]
fn restore_phrase_bip84_smoke_and_exact() {
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--template",
            "bip84",
        ])
        .output()
        .expect("spawn");
    assert!(out.status.success(), "exit {:?}", out.status.code());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("master fingerprint:"), "stdout:\n{stdout}");
    assert!(stdout.contains(FP_NO_PP), "stdout:\n{stdout}");
    assert!(stdout.contains("CONFIRM"), "stdout:\n{stdout}");
    // Exact descriptor + first recv address.
    assert!(
        stdout.contains(DESC_BIP84),
        "expected descriptor {DESC_BIP84}\ngot:\n{stdout}"
    );
    assert!(stdout.contains(FIRST_RECV_BIP84), "stdout:\n{stdout}");
    // multipath `<0;1>` token present.
    assert!(stdout.contains("<0;1>"), "stdout:\n{stdout}");
}

#[test]
fn restore_all_four_default() {
    let out = bin()
        .args(["restore", "--from", &format!("phrase={TREZOR_12}")])
        .output()
        .expect("spawn");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    // All four single-sig script-type prefixes appear.
    assert!(stdout.contains("pkh(["), "stdout:\n{stdout}"); // bip44
    assert!(stdout.contains("sh(wpkh(["), "stdout:\n{stdout}"); // bip49
    assert!(stdout.contains("wpkh(["), "stdout:\n{stdout}"); // bip84
    assert!(stdout.contains("tr(["), "stdout:\n{stdout}"); // bip86
    // Fingerprint is path-independent — identical across all four (header + 4
    // descriptor origins = at least 5 occurrences).
    assert!(stdout.matches(FP_NO_PP).count() >= 5, "stdout:\n{stdout}");
}

#[test]
fn restore_template_single_only_bip84() {
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--template",
            "bip84",
        ])
        .output()
        .expect("spawn");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    // Only bip84 — no bip44 legacy pkh, no taproot tr.
    assert!(stdout.contains("wpkh(["), "stdout:\n{stdout}");
    assert!(!stdout.contains("pkh([73c5da0a/44"), "stdout:\n{stdout}");
    assert!(!stdout.contains("tr([73c5da0a/86"), "stdout:\n{stdout}");
}

// ---------------------------------------------------------------------------
// 1.3 input channels: ms1 / entropy / seedqr / passphrase / stdin-mutex
// ---------------------------------------------------------------------------

#[test]
fn restore_from_ms1_same_fingerprint() {
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("ms1={MS1_NO_PP}"),
            "--template",
            "bip84",
        ])
        .output()
        .expect("spawn");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains(FP_NO_PP), "stdout:\n{stdout}");
    assert!(stdout.contains(DESC_BIP84), "stdout:\n{stdout}");
}

#[test]
fn restore_from_entropy_same_fingerprint() {
    // abandon×11+about == 16 zero entropy bytes.
    let out = bin()
        .args([
            "restore",
            "--from",
            "entropy=00000000000000000000000000000000",
            "--template",
            "bip84",
        ])
        .output()
        .expect("spawn");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains(FP_NO_PP), "stdout:\n{stdout}");
    assert!(stdout.contains(DESC_BIP84), "stdout:\n{stdout}");
}

#[test]
fn restore_from_seedqr_same_fingerprint() {
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("seedqr={SEEDQR_NO_PP}"),
            "--template",
            "bip84",
        ])
        .output()
        .expect("spawn");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains(FP_NO_PP), "stdout:\n{stdout}");
    assert!(stdout.contains(DESC_BIP84), "stdout:\n{stdout}");
}

#[test]
fn restore_passphrase_via_env_and_stdin_derives_trezor_pp_fingerprint() {
    // Re-derive the TREZOR-pp fingerprint independently (do NOT hardcode from
    // the brief): convert --to fingerprint with the same passphrase.
    let expected = fingerprint_via_convert(TREZOR_12, Some("TREZOR"));
    assert_eq!(expected, "b4e3f5ed", "TREZOR-pp fingerprint drifted");

    // Channel: seed via @env:, passphrase via stdin (no secret on argv).
    let out = bin()
        .args([
            "restore",
            "--from",
            "phrase=@env:RESTORE_SEED",
            "--passphrase-stdin",
            "--template",
            "bip84",
        ])
        .env("RESTORE_SEED", TREZOR_12)
        .write_stdin("TREZOR")
        .output()
        .expect("spawn");
    assert!(out.status.success(), "exit {:?}", out.status.code());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains(&expected), "expected {expected}\nstdout:\n{stdout}");
    assert!(stdout.contains("passphrase: applied"), "stdout:\n{stdout}");
}

#[test]
fn restore_stdin_mutex_rejected() {
    // --passphrase-stdin AND --from phrase=- both want stdin → exit 1.
    let out = bin()
        .args([
            "restore",
            "--from",
            "phrase=-",
            "--passphrase-stdin",
            "--template",
            "bip84",
        ])
        .write_stdin(TREZOR_12)
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(1), "expected exit 1 stdin-mutex");
}

#[test]
fn restore_non_seed_from_rejected() {
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("xpub={ACCT_XPUB_BIP84}"),
        ])
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(1), "non-seed --from must be exit 1");
}

#[test]
fn restore_ms1_mnem_uses_wire_language_not_english() {
    // Japanese mnem card: deriving as English would give a DIFFERENT fingerprint.
    // The wire language (Japanese) must win → fingerprint 0ed2c5a4.
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("ms1={MS1_MNEM_JP}"),
            "--template",
            "bip84",
        ])
        .output()
        .expect("spawn");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("0ed2c5a4"), "wire-language seed expected;\n{stdout}");
    // The same 16-zero entropy derived as ENGLISH gives `73c5da0a`; the wire
    // language (Japanese) must override, so that fingerprint must NOT appear.
    assert!(!stdout.contains(FP_NO_PP), "must not be english-derived;\n{stdout}");
}

#[test]
fn restore_ms1_mnem_language_conflict_exit_2() {
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("ms1={MS1_MNEM_JP}"),
            "--template",
            "bip84",
            "--language",
            "english",
        ])
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(2), "language-conflict must be exit 2");
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("language") && stderr.contains("Japanese"),
        "stderr:\n{stderr}"
    );
}

// ---------------------------------------------------------------------------
// 1.4 watch-only-out negative
// ---------------------------------------------------------------------------

#[test]
fn restore_emits_no_private_key_material() {
    // Cover both the all-4 default (every script type) AND a passphrase run.
    for args in [
        vec!["restore", "--from", &format!("phrase={TREZOR_12}")],
        vec![
            "restore",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--passphrase",
            "TREZOR",
        ],
    ] {
        let out = bin().args(&args).output().expect("spawn");
        assert!(out.status.success());
        let stdout = String::from_utf8(out.stdout).unwrap();
        let stderr = String::from_utf8(out.stderr).unwrap();
        for stream in [&stdout, &stderr] {
            assert!(!stream.contains("xprv"), "private xprv leaked:\n{stream}");
            assert!(!stream.contains("tprv"), "private tprv leaked:\n{stream}");
        }
    }
}

// ---------------------------------------------------------------------------
// 1.5 verify gate: expect-fingerprint / expect-xpub / allow-mismatch / UNVERIFIED
// ---------------------------------------------------------------------------

#[test]
fn restore_expect_fingerprint_match_exit_0() {
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--template",
            "bip84",
            "--expect-fingerprint",
            FP_NO_PP,
        ])
        .output()
        .expect("spawn");
    assert!(out.status.success(), "match must be exit 0");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains(DESC_BIP84), "stdout:\n{stdout}");
    // A matched reference suppresses the UNVERIFIED banner.
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(!stderr.contains("UNVERIFIED"), "stderr:\n{stderr}");
}

#[test]
fn restore_expect_fingerprint_mismatch_exit_4_no_descriptors() {
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--template",
            "bip84",
            "--expect-fingerprint",
            "deadbeef",
        ])
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(4), "mismatch must be exit 4");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(!stdout.contains("wpkh("), "no descriptors on mismatch;\n{stdout}");
}

#[test]
fn restore_mismatch_allow_override_exit_0_banner() {
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--template",
            "bip84",
            "--expect-fingerprint",
            "deadbeef",
            "--allow-mismatch",
        ])
        .output()
        .expect("spawn");
    assert!(out.status.success(), "allow-mismatch must be exit 0");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("wpkh("), "descriptors emitted on override;\n{stdout}");
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("MISMATCH (overridden)"), "stderr:\n{stderr}");
}

#[test]
fn restore_no_reference_unverified_banner() {
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--template",
            "bip84",
        ])
        .output()
        .expect("spawn");
    assert!(out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("UNVERIFIED"), "stderr:\n{stderr}");
}

#[test]
fn restore_expect_xpub_match_exit_0() {
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--template",
            "bip84",
            "--expect-xpub",
            ACCT_XPUB_BIP84,
        ])
        .output()
        .expect("spawn");
    assert!(out.status.success(), "xpub match must be exit 0");
}

#[test]
fn restore_expect_xpub_without_template_exit_2() {
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--expect-xpub",
            ACCT_XPUB_BIP84,
        ])
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(2), "expect-xpub w/o template = exit 2");
}

#[test]
fn restore_multisig_template_rejected_exit_1() {
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--template",
            "wsh-sortedmulti",
        ])
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(1), "multisig template = exit 1");
}

#[test]
fn restore_watch_only_advisory_present() {
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--template",
            "bip84",
        ])
        .output()
        .expect("spawn");
    assert!(out.status.success());
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("watch-only"), "advisory missing:\n{stderr}");
}

// ---------------------------------------------------------------------------
// 2.1 --format importable payload (requires single --template)
// ---------------------------------------------------------------------------

#[test]
fn restore_format_descriptor_payload_to_stdout() {
    // `--format descriptor` emits the bare concrete descriptor on stdout; the
    // verify block (fingerprint / CONFIRM / first recv) goes to stderr so the
    // payload pipes cleanly into wallet software.
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--template",
            "bip84",
            "--format",
            "descriptor",
        ])
        .output()
        .expect("spawn");
    assert!(out.status.success(), "exit 0; got {:?}", out.status.code());
    let stdout = String::from_utf8(out.stdout).unwrap();
    // stdout is EXACTLY the descriptor payload (single trailing newline).
    assert_eq!(stdout.trim_end(), DESC_BIP84, "stdout payload:\n{stdout}");
    // The human verify doc is on stderr, not stdout.
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains(FP_NO_PP), "fingerprint on stderr:\n{stderr}");
    assert!(stderr.contains("CONFIRM"), "CONFIRM on stderr:\n{stderr}");
    assert!(stderr.contains(FIRST_RECV_BIP84), "first recv on stderr:\n{stderr}");
    // The descriptor MUST NOT also be duplicated as a text doc on stdout.
    assert!(
        !stdout.contains("master fingerprint:"),
        "verify doc must be on stderr, not stdout:\n{stdout}"
    );
}

#[test]
fn restore_format_bitcoin_core_payload() {
    // bitcoin-core → an importdescriptors JSON array (receive + change descriptors).
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--template",
            "bip84",
            "--format",
            "bitcoin-core",
        ])
        .output()
        .expect("spawn");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value =
        serde_json::from_str(&stdout).expect("bitcoin-core payload must be a JSON array");
    let arr = v.as_array().expect("importdescriptors array");
    assert_eq!(arr.len(), 2, "receive + change descriptor:\n{stdout}");
    // The receive descriptor carries the bip84 origin + xpub (single-chain `/0/*`).
    let desc0 = arr[0]["desc"].as_str().unwrap();
    assert!(
        desc0.contains("[73c5da0a/84'/0'/0']") && desc0.contains(ACCT_XPUB_BIP84),
        "receive desc:\n{desc0}"
    );
}

#[test]
fn restore_format_bip388_payload() {
    // bip388 → a wallet-policy JSON object (`description_template` + `keys_info`).
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--template",
            "bip84",
            "--format",
            "bip388",
        ])
        .output()
        .expect("spawn");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value =
        serde_json::from_str(&stdout).expect("bip388 payload must be a JSON object");
    assert_eq!(
        v["description_template"].as_str().unwrap(),
        "wpkh(@0/**)",
        "bip388 template:\n{stdout}"
    );
    let keys = v["keys_info"].as_array().expect("keys_info array");
    assert_eq!(keys.len(), 1, "single-sig has one key:\n{stdout}");
    assert!(
        keys[0].as_str().unwrap().contains(ACCT_XPUB_BIP84),
        "keys_info xpub:\n{stdout}"
    );
}

#[test]
fn restore_format_without_template_exit_2() {
    // `--format` requires a single `--template`; with the all-4 default → exit 2
    // (ModeViolation, NOT BadInput exit 1).
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--format",
            "descriptor",
        ])
        .output()
        .expect("spawn");
    assert_eq!(
        out.status.code(),
        Some(2),
        "--format + all-4 default = ModeViolation exit 2"
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.is_empty(), "no payload on the gate:\n{stdout}");
}

#[test]
fn restore_format_mismatch_no_allow_exit_4_no_payload() {
    // The verify gate runs BEFORE the payload: a mismatch without --allow-mismatch
    // → exit 4 and NO payload on stdout.
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--template",
            "bip84",
            "--format",
            "descriptor",
            "--expect-fingerprint",
            "deadbeef",
        ])
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(4), "mismatch must be exit 4");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.is_empty(),
        "no payload emitted on mismatch:\n{stdout}"
    );
}

#[test]
fn restore_format_path_emits_no_private_key_material() {
    // Watch-only-out across the --format path (incl. a passphrase run).
    for fmt in ["descriptor", "bitcoin-core", "bip388", "sparrow"] {
        let out = bin()
            .args([
                "restore",
                "--from",
                &format!("phrase={TREZOR_12}"),
                "--template",
                "bip84",
                "--passphrase",
                "TREZOR",
                "--format",
                fmt,
            ])
            .output()
            .expect("spawn");
        assert!(out.status.success(), "{fmt} exit 0");
        let stdout = String::from_utf8(out.stdout).unwrap();
        let stderr = String::from_utf8(out.stderr).unwrap();
        for stream in [&stdout, &stderr] {
            assert!(!stream.contains("xprv"), "{fmt}: xprv leaked:\n{stream}");
            assert!(!stream.contains("tprv"), "{fmt}: tprv leaked:\n{stream}");
            assert!(!stream.contains("WIF"), "{fmt}: WIF leaked:\n{stream}");
            assert!(
                !stream.contains("account_xpriv"),
                "{fmt}: account_xpriv leaked:\n{stream}"
            );
        }
    }
}

// ---------------------------------------------------------------------------
// 2.2 --json structured output + --output + seed redaction
// ---------------------------------------------------------------------------

#[test]
fn restore_json_shape_single_template() {
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--template",
            "bip84",
            "--json",
        ])
        .output()
        .expect("spawn");
    assert!(out.status.success(), "exit 0; got {:?}", out.status.code());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value =
        serde_json::from_str(&stdout).expect("--json stdout must parse as JSON");
    assert_eq!(v["master_fingerprint"].as_str().unwrap(), FP_NO_PP);
    assert!(!v["passphrase_applied"].as_bool().unwrap());
    assert_eq!(v["network"].as_str().unwrap(), "mainnet");
    assert_eq!(v["verification"]["status"].as_str().unwrap(), "unverified");
    let wallets = v["wallets"].as_array().expect("wallets array");
    assert_eq!(wallets.len(), 1, "single --template = 1 wallet");
    assert_eq!(wallets[0]["wallet_type"].as_str().unwrap(), "bip84");
    assert_eq!(wallets[0]["descriptor"].as_str().unwrap(), DESC_BIP84);
    let addrs = wallets[0]["first_addresses"].as_array().unwrap();
    assert_eq!(addrs[0].as_str().unwrap(), FIRST_RECV_BIP84);
    // No `--format` → no import_payload field.
    assert!(
        v.get("import_payload").is_none(),
        "import_payload absent without --format:\n{stdout}"
    );
}

#[test]
fn restore_json_all_four_default_lists_four_wallets() {
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--json",
        ])
        .output()
        .expect("spawn");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("parse");
    let types: Vec<&str> = v["wallets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w["wallet_type"].as_str().unwrap())
        .collect();
    assert_eq!(types, ["bip44", "bip49", "bip84", "bip86"], "all-4:\n{stdout}");
}

#[test]
fn restore_json_with_format_embeds_import_payload() {
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--template",
            "bip84",
            "--json",
            "--format",
            "descriptor",
        ])
        .output()
        .expect("spawn");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("parse");
    assert_eq!(
        v["import_payload"].as_str().unwrap(),
        DESC_BIP84,
        "import_payload embeds the --format payload:\n{stdout}"
    );
}

#[test]
fn restore_json_verification_verified_status() {
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--template",
            "bip84",
            "--json",
            "--expect-fingerprint",
            FP_NO_PP,
        ])
        .output()
        .expect("spawn");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("parse");
    assert_eq!(v["verification"]["status"].as_str().unwrap(), "verified");
}

#[test]
fn restore_json_mismatch_exits_4_not_json_success_body() {
    // A reference mismatch surfaces via ToolkitError::message() (exit 4), NOT a
    // json-success body with verification.status="mismatch".
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--template",
            "bip84",
            "--json",
            "--expect-fingerprint",
            "deadbeef",
        ])
        .output()
        .expect("spawn");
    assert_eq!(out.status.code(), Some(4), "mismatch in --json must be exit 4");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        stdout.is_empty(),
        "no json success body on mismatch:\n{stdout}"
    );
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(
        stderr.contains("restore: fingerprint mismatch"),
        "message() error on stderr:\n{stderr}"
    );
}

#[test]
fn restore_json_no_seed_material_redacted_non_vacuous() {
    // NON-VACUOUS: run with a REAL passphrase + a REAL ms1 seed. The passphrase
    // must actually be applied (it changes the fingerprint vs the no-pp seed),
    // yet NONE of the seed phrase / ms1 string / passphrase / xprv / tprv may
    // appear anywhere in the --json output.
    const PASSPHRASE: &str = "correct-horse-battery-staple-9417";
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("ms1={MS1_NO_PP}"),
            "--passphrase",
            PASSPHRASE,
            "--template",
            "bip84",
            "--json",
        ])
        .output()
        .expect("spawn");
    assert!(out.status.success(), "exit 0; got {:?}", out.status.code());
    let stdout = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).expect("parse");
    // Proof the passphrase was applied: passphrase_applied=true AND the
    // fingerprint is NOT the no-passphrase fingerprint (so PBKDF2 used it).
    assert!(v["passphrase_applied"].as_bool().unwrap());
    assert_ne!(
        v["master_fingerprint"].as_str().unwrap(),
        FP_NO_PP,
        "passphrase must change the fingerprint (non-vacuous):\n{stdout}"
    );
    // Redaction: no secret token appears anywhere in stdout.
    for tok in [PASSPHRASE, MS1_NO_PP, "xprv", "tprv", "abandon"] {
        assert!(
            !stdout.contains(tok),
            "secret `{tok}` leaked into --json:\n{stdout}"
        );
    }
}

#[test]
fn restore_output_file_writes_content_stdout_clean() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("restore_out_{}.txt", std::process::id()));
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--template",
            "bip84",
            "--output",
            path.to_str().unwrap(),
        ])
        .output()
        .expect("spawn");
    assert!(out.status.success());
    // stdout is empty/clean — the document went to the file.
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.is_empty(), "stdout must be clean with --output:\n{stdout}");
    let file = std::fs::read_to_string(&path).expect("output file written");
    assert!(file.contains("master fingerprint:"), "doc in file:\n{file}");
    assert!(file.contains(DESC_BIP84), "descriptor in file:\n{file}");
    std::fs::remove_file(&path).ok();
}

#[test]
fn restore_output_file_with_json_writes_json() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!("restore_out_json_{}.txt", std::process::id()));
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--template",
            "bip84",
            "--json",
            "--output",
            path.to_str().unwrap(),
        ])
        .output()
        .expect("spawn");
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.is_empty(), "stdout clean with --json --output:\n{stdout}");
    let file = std::fs::read_to_string(&path).expect("output file");
    let v: serde_json::Value =
        serde_json::from_str(&file).expect("file content is valid JSON");
    assert_eq!(v["master_fingerprint"].as_str().unwrap(), FP_NO_PP);
    std::fs::remove_file(&path).ok();
}

#[test]
fn restore_json_and_output_paths_emit_no_private_key_material() {
    // Watch-only-out absolute across the --json and --output paths (with pp).
    let dir = std::env::temp_dir();
    let path = dir.join(format!("restore_wo_{}.txt", std::process::id()));
    // --json path.
    let out = bin()
        .args([
            "restore",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--passphrase",
            "TREZOR",
            "--json",
        ])
        .output()
        .expect("spawn");
    assert!(out.status.success());
    let json_stdout = String::from_utf8(out.stdout).unwrap();
    // --output path (file).
    let out2 = bin()
        .args([
            "restore",
            "--from",
            &format!("phrase={TREZOR_12}"),
            "--passphrase",
            "TREZOR",
            "--output",
            path.to_str().unwrap(),
        ])
        .output()
        .expect("spawn");
    assert!(out2.status.success());
    let file = std::fs::read_to_string(&path).expect("output file");
    std::fs::remove_file(&path).ok();
    for stream in [&json_stdout, &file] {
        assert!(!stream.contains("xprv"), "xprv leaked:\n{stream}");
        assert!(!stream.contains("tprv"), "tprv leaked:\n{stream}");
        assert!(
            !stream.contains("account_xpriv"),
            "account_xpriv leaked:\n{stream}"
        );
    }
}
