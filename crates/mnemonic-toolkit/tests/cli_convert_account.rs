//! FIX_sec_convert_account_unused_refusal RED rows: a non-zero `convert --account`
//! is refused when no requested `--to` target reads it
//! (`design/FIX_sec_convert_account_unused_refusal.md` sections 2 and 6). Signet
//! test seed only (BIP-39 `abandon x11 about`, master fingerprint 73c5da0a).
use assert_cmd::Command;

const P12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const ENTROPY: &str = "00000000000000000000000000000000";
const SEEDQR: &str = "000000000000000000000000000000000000000000000003";
const WIF_87: &str = "cQSNG6HnKUGreXgRpAr8QV5k9fvZkgrQuKXmgFPFNypaWK3yGe1R";
const XPUB_84: &str = "tpubDC8msFGeGuwnKG9Upg7DM2b4DaRqg3CUZa5g8v2SRQ6K4NSkxUgd7HsL2XVWbVm39yBA4LAxysQAm397zwQSQoQgewGiYZqrA9DsP4zbQ1M";
const XPRV_84: &str = "tprv8fSjiqEQ8YG7Ro7gw2ScwcvweYuuWi1ZzGUtrPz918HvDtBzL5s2voFTrN4y3yUwj5cYD54pLhxk6NKCzHUjcka3zbKjbTEcsuAnkzbjhkL";
const MK1_84: &str = "mk1qpg397pqqsq5gjlze9eutks2zvzrtp70pm6trteu3ssr0mjvzcsa5rf53k63zcmsnf3z6rfg8rw7dkzpn3glvvquvgpms6mzamgctgratycf mk1qpg397pp3c8mu0myvvm7myaups8nhpplea7jtz09ajyyw48xgqsz02ysk3l6fyxg6m66tmvhzh70a";
const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
/// m/84'/1'/3' account key (= `convert --from P --to xpub --path m/84'/1'/3'`).
const XPUB_84_A3: &str = "tpubDC8msFGeGuwnSsiDodPKT5czjv1psTKmAnqF8tmt8cJ8Besyz2KEbSCkA5M2xdvVZfcTWpBohyLzKNVKhDR1ysXy9aoZMVrS56hjFNJCCNR";
const XPRV_84_A3: &str = "tprv8fSjiqEQ8YG7ZQgRuyij3fxtAtVti88rbVETrNjaiLVjMAdDMdVeQwasyvm2KrMXdtKKCB9jmp1jY3bgk9SWiEUccJgRYEcJsrubEBjqypZ";

const ACCT: &str = "error: --account is not used by any requested --to target from this --from source (only --to xpub or --to xprv from a phrase, seedqr or entropy reads it); drop --account.";
const R3_TEXT: &str = "error: --account has no effect when --path is given; put the account in --path (e.g. m/84'/1'/3') or drop --path.";
const R4_TEXT: &str =
    "error: --path is not used by any requested --to target from this --from source; drop --path.";
const ADDR_PATH_TEXT: &str = "error: --to address requires --path (xpub does not carry an origin path; supply BIP-32 derivation explicitly).";
const EDGE_XPRV_TEXT: &str =
    "error: --to xprv is cryptographically unrecoverable from --from xprv (one-way derivation barrier)";

struct Out {
    code: i32,
    stdout: String,
    stderr: String,
}

fn convert(args: &[&str]) -> Out {
    let o = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("--allow-argv-secret")
        .arg("convert")
        .args(args)
        .output()
        .unwrap();
    Out {
        code: o.status.code().unwrap(),
        stdout: String::from_utf8(o.stdout).unwrap(),
        stderr: String::from_utf8(o.stderr).unwrap(),
    }
}

fn phrase() -> String {
    format!("phrase={P12}")
}

fn assert_ok(o: &Out, lines: &[String]) {
    assert_eq!(o.code, 0, "stderr: {}", o.stderr);
    for l in lines {
        assert!(
            o.stdout.lines().any(|x| x == l),
            "missing `{l}` in stdout:\n{}",
            o.stdout
        );
    }
}

/// Exit code, stdout empty, and a stderr line equal to `text`.
fn assert_refused(o: &Out, code: i32, text: &str) {
    assert_eq!(o.code, code, "stdout: {}\nstderr: {}", o.stdout, o.stderr);
    assert!(o.stdout.is_empty(), "stdout not empty:\n{}", o.stdout);
    assert!(
        o.stderr.lines().any(|x| x == text),
        "missing `{text}` in stderr:\n{}",
        o.stderr
    );
}

fn assert_no_line(o: &Out, text: &str) {
    assert!(
        !o.stderr.lines().any(|x| x == text),
        "unexpected `{text}` in stderr:\n{}",
        o.stderr
    );
}

fn acct_refused(args: &[&str]) {
    let o = convert(args);
    assert_refused(&o, 64, ACCT);
}

// ---- 6.1 RED rows.

#[test]
fn acct_xpub_to_xpub_account_refused() {
    let from = format!("xpub={XPUB_84}");
    acct_refused(&[
        "--from",
        &from,
        "--network",
        "signet",
        "--to",
        "xpub",
        "--account",
        "3",
    ]);
    acct_refused(&[
        "--from",
        &from,
        "--network",
        "signet",
        "--to",
        "xpub",
        "--account",
        "1",
    ]);
}

#[test]
fn acct_xprv_to_xpub_account_refused() {
    let from = format!("xprv={XPRV_84}");
    acct_refused(&[
        "--from",
        &from,
        "--network",
        "signet",
        "--to",
        "xpub",
        "--account",
        "3",
    ]);
}

#[test]
fn acct_wif_to_xpub_account_refused() {
    let from = format!("wif={WIF_87}");
    acct_refused(&[
        "--from",
        &from,
        "--network",
        "signet",
        "--to",
        "xpub",
        "--account",
        "3",
    ]);
}

#[test]
fn acct_mk1_to_xpub_account_refused() {
    let from = format!("mk1={MK1_84}");
    acct_refused(&[
        "--from",
        &from,
        "--network",
        "signet",
        "--to",
        "xpub",
        "--account",
        "3",
    ]);
}

#[test]
fn acct_ms1_to_phrase_account_refused() {
    let from = format!("ms1={MS1}");
    acct_refused(&[
        "--from",
        &from,
        "--network",
        "signet",
        "--to",
        "phrase",
        "--account",
        "3",
    ]);
}

#[test]
fn acct_phrase_to_fingerprint_account_refused() {
    acct_refused(&[
        "--from",
        &phrase(),
        "--network",
        "signet",
        "--template",
        "bip84",
        "--to",
        "fingerprint",
        "--account",
        "3",
    ]);
}

#[test]
fn acct_phrase_to_entropy_account_refused() {
    acct_refused(&[
        "--from",
        &phrase(),
        "--network",
        "signet",
        "--to",
        "entropy",
        "--account",
        "3",
    ]);
}

#[test]
fn acct_phrase_to_address_account_refused() {
    let o = convert(&[
        "--from",
        &phrase(),
        "--network",
        "signet",
        "--to",
        "address",
        "--script-type",
        "p2wpkh",
        "--account",
        "3",
    ]);
    assert_refused(&o, 64, ACCT);
    assert_no_line(&o, ADDR_PATH_TEXT);
}

#[test]
fn acct_xpub_to_address_account_before_address_path() {
    let from = format!("xpub={XPUB_84}");
    let o = convert(&[
        "--from",
        &from,
        "--network",
        "signet",
        "--to",
        "address",
        "--script-type",
        "p2wpkh",
        "--account",
        "3",
    ]);
    assert_refused(&o, 64, ACCT);
    assert_no_line(&o, ADDR_PATH_TEXT);
}

#[test]
fn acct_json_refused_no_envelope() {
    let from = format!("xpub={XPUB_84}");
    acct_refused(&[
        "--from",
        &from,
        "--network",
        "signet",
        "--to",
        "xpub",
        "--account",
        "3",
        "--json",
    ]);
}

#[test]
fn acct_help_account_line() {
    let o = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["convert", "--help"])
        .output()
        .unwrap();
    assert!(o.status.success());
    let h = String::from_utf8(o.stdout).unwrap();
    assert!(
        h.lines().any(|l| l.trim()
            == "BIP-32 account index (default 0). Used to compute the template-derived path when `--path` is omitted. Refused (non-zero) together with --path, or where no requested target uses it"),
        "help:\n{h}"
    );
}

// ---- 6.2 guards (pass before and after the fix).

#[test]
fn acct_guard_phrase_template_xpub_account_honoured() {
    let o = convert(&[
        "--from",
        &phrase(),
        "--network",
        "signet",
        "--template",
        "bip84",
        "--to",
        "xpub",
        "--account",
        "3",
    ]);
    assert_ok(&o, &[format!("xpub: {XPUB_84_A3}")]);
}

#[test]
fn acct_guard_seedqr_template_xprv_account_honoured() {
    let from = format!("seedqr={SEEDQR}");
    let o = convert(&[
        "--from",
        &from,
        "--network",
        "signet",
        "--template",
        "bip84",
        "--to",
        "xprv",
        "--account",
        "3",
    ]);
    assert_ok(&o, &[format!("xprv: {XPRV_84_A3}")]);
}

#[test]
fn acct_guard_entropy_xpub_fingerprint_accepted() {
    let from = format!("entropy={ENTROPY}");
    let o = convert(&[
        "--from",
        &from,
        "--network",
        "signet",
        "--template",
        "bip84",
        "--to",
        "xpub,fingerprint",
        "--account",
        "3",
    ]);
    assert_ok(
        &o,
        &[
            format!("xpub: {XPUB_84_A3}"),
            "fingerprint: 73c5da0a".to_string(),
        ],
    );
}

#[test]
fn acct_guard_reader_not_first_accepted() {
    let o = convert(&[
        "--from",
        &phrase(),
        "--network",
        "signet",
        "--template",
        "bip84",
        "--to",
        "entropy,xpub",
        "--account",
        "3",
    ]);
    assert_ok(
        &o,
        &[format!("entropy: {ENTROPY}"), format!("xpub: {XPUB_84_A3}")],
    );
}

#[test]
fn acct_guard_xpub_account_zero_accepted() {
    let from = format!("xpub={XPUB_84}");
    let o = convert(&[
        "--from",
        &from,
        "--network",
        "signet",
        "--to",
        "xpub",
        "--account",
        "0",
    ]);
    assert_ok(&o, &[format!("xpub: {XPUB_84}")]);
}

#[test]
fn acct_order_r4_before_account_unused() {
    let from = format!("xpub={XPUB_84}");
    let o = convert(&[
        "--from",
        &from,
        "--network",
        "signet",
        "--to",
        "xpub",
        "--path",
        "m/0",
        "--account",
        "3",
    ]);
    assert_refused(&o, 64, R4_TEXT);
    assert_no_line(&o, ACCT);
}

#[test]
fn acct_order_r3_before_account_unused() {
    let from = format!("xpub={XPUB_84}");
    let o = convert(&[
        "--from",
        &from,
        "--network",
        "signet",
        "--to",
        "address",
        "--script-type",
        "p2wpkh",
        "--path",
        "m/0/0",
        "--account",
        "3",
    ]);
    assert_refused(&o, 64, R3_TEXT);
    assert_no_line(&o, ACCT);
}

#[test]
fn acct_order_edge_refusal_before_account_unused() {
    let from = format!("xprv={XPRV_84}");
    let o = convert(&[
        "--from",
        &from,
        "--network",
        "signet",
        "--to",
        "xprv",
        "--account",
        "3",
    ]);
    assert_refused(&o, 2, EDGE_XPRV_TEXT);
    assert_no_line(&o, ACCT);
}
