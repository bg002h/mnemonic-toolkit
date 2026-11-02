//! FIX_convert_path RED rows: `convert --path` (ruling R1-R5,
//! `design/agent-reports/grade-convert-path.md`). Signet test seed only
//! (BIP-39 `abandon x11 about`, master fingerprint 73c5da0a).
use assert_cmd::Command;

const P12: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const ENTROPY: &str = "00000000000000000000000000000000";
const SEEDQR: &str = "000000000000000000000000000000000000000000000003";
/// Account key at m/87'/1'/0' (= `--template wsh-sortedmulti`, BIP-87 default).
const XPUB_87: &str = "tpubDCChhoz5Qdrkn7Z7KXawq6Ad6r3A4MUkCoVTqeWxfTkA6bHNJ3CHUEtALQdkNeixNz4446PcAmw4WKcj3mV2vb29H7sg9EPzbyCU1y2merw";
const XPRV_87: &str = "tprv8fWfZPwqGGB5teXKRsvMRgWWXpXDu2HqdVtgZ8UfFBwmG72bfeNhHkGJAHjJr9T11jjduBxH2fhi7rW6U2NBMiQcUgSGfc3y5p1JUFQA7gH";
const WIF_87: &str = "cQSNG6HnKUGreXgRpAr8QV5k9fvZkgrQuKXmgFPFNypaWK3yGe1R";
/// Account key at m/84'/1'/0' (= `--template bip84`).
const XPUB_84: &str = "tpubDC8msFGeGuwnKG9Upg7DM2b4DaRqg3CUZa5g8v2SRQ6K4NSkxUgd7HsL2XVWbVm39yBA4LAxysQAm397zwQSQoQgewGiYZqrA9DsP4zbQ1M";
const XPRV_84: &str = "tprv8fSjiqEQ8YG7Ro7gw2ScwcvweYuuWi1ZzGUtrPz918HvDtBzL5s2voFTrN4y3yUwj5cYD54pLhxk6NKCzHUjcka3zbKjbTEcsuAnkzbjhkL";
// m/48'/1'/0'/2' (BIP-48 signet P2WSH). XPUB_48's key equals the pubkey of
// WIF_48, which today's `--to wif --path m/48'/1'/0'/2'` prints (checked
// independently by secp256k1 point multiplication); XPRV_48's private key
// equals WIF_48's, and its depth, parent fingerprint, child number and chain
// code equal XPUB_48's (checked independently by base58check decoding).
const XPUB_48: &str = "tpubDFH9dgzveyD8zTbPUFuLrGmCydNvxehyNdUXKJAQN8x4aZ4j6UZqGfnqFrD4NqyaTVGKbvEW54tsvPTK2UoSbCC1PJY8iCNiwTL3RWZEheQ";
const XPRV_48: &str = "tprv8ib7VGxgWbXU6zZbacEkSs76QbrzoKX4oKsk2n86ws9fk4oxU5kF6BAy5iKYC2XM3X9GZApZM9B14B7JTXd77DXwmmNfh5cEWiw21CfyfPx";
const WIF_48: &str = "cUuVAfhtsAn4VtDzXdmQMQ8nJdXVsJXwCdhkL5RR4dvbcgK2Gvco";
/// Master key (path `m`).
const XPUB_M: &str = "tpubD6NzVbkrYhZ4XYa9MoLt4BiMZ4gkt2faZ4BcmKu2a9te4LDpQmvEz2L2yDERivHxFPnxXXhqDRkUNnQCpZggCyEZLBktV7VaSmwayqMJy1s";
/// mk1 card minted by `bundle` from XPUB_84 at m/84'/1'/0', fingerprint 73c5da0a, signet.
const MK1_84: &str = "mk1qpg397pqqsq5gjlze9eutks2zvzrtp70pm6trteu3ssr0mjvzcsa5rf53k63zcmsnf3z6rfg8rw7dkzpn3glvvquvgpms6mzamgctgratycf mk1qpg397pp3c8mu0myvvm7myaups8nhpplea7jtz09ajyyw48xgqsz02ysk3l6fyxg6m66tmvhzh70a";
/// ms1 card from `convert --from phrase=P12 --to ms1`.
const MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

const R3_TEXT: &str = "error: --account has no effect when --path is given; put the account in --path (e.g. m/84'/1'/3') or drop --path.";
const R4_TEXT: &str =
    "error: --path is not used by any requested --to target from this --from source; drop --path.";
const R5_TEXT: &str = "error: --path parse: invalid child number format";
const R5_FORM_TEXT: &str = "error: --path parse: invalid derivation path format";

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

fn assert_refused(o: &Out, code: i32, text: &str) {
    assert_eq!(o.code, code, "stdout: {}\nstderr: {}", o.stdout, o.stderr);
    assert!(
        o.stderr.lines().any(|x| x == text),
        "missing `{text}` in stderr:\n{}",
        o.stderr
    );
}

fn xpub_xprv_87() -> Vec<String> {
    vec![format!("xpub: {XPUB_87}"), format!("xprv: {XPRV_87}")]
}

// ---- R1: --path gives the key at P; no --template needed; --path wins; shared by all targets.

#[test]
fn r1_phrase_path_xpub_xprv_without_template() {
    let o = convert(&[
        "--from",
        &phrase(),
        "--network",
        "signet",
        "--to",
        "xpub,xprv",
        "--path",
        "m/87'/1'/0'",
    ]);
    assert_ok(&o, &xpub_xprv_87());
}

#[test]
fn r1_seedqr_path_xpub_xprv_without_template() {
    let from = format!("seedqr={SEEDQR}");
    let o = convert(&[
        "--from",
        &from,
        "--network",
        "signet",
        "--to",
        "xpub,xprv",
        "--path",
        "m/87'/1'/0'",
    ]);
    assert_ok(&o, &xpub_xprv_87());
}

#[test]
fn r1_entropy_path_xpub_xprv_without_template() {
    let from = format!("entropy={ENTROPY}");
    let o = convert(&[
        "--from",
        &from,
        "--network",
        "signet",
        "--to",
        "xpub,xprv",
        "--path",
        "m/87'/1'/0'",
    ]);
    assert_ok(&o, &xpub_xprv_87());
}

#[test]
fn r1_path_wins_over_template() {
    let o = convert(&[
        "--from",
        &phrase(),
        "--network",
        "signet",
        "--template",
        "bip84",
        "--to",
        "xpub,xprv",
        "--path",
        "m/87'/1'/0'",
    ]);
    assert_ok(&o, &xpub_xprv_87());
    assert!(
        !o.stdout.contains(XPUB_84) && !o.stdout.contains(XPRV_84),
        "template path used: {}",
        o.stdout
    );
}

#[test]
fn convert_template_xpub_path_not_silently_ignored() {
    let base = convert(&[
        "--from",
        &phrase(),
        "--template",
        "wsh-sortedmulti",
        "--network",
        "signet",
        "--to",
        "xpub",
    ]);
    assert_eq!(base.code, 0);
    let a = convert(&[
        "--from",
        &phrase(),
        "--template",
        "wsh-sortedmulti",
        "--network",
        "signet",
        "--to",
        "xpub",
        "--path",
        "m/99'/1'/5'",
    ]);
    // fail-closed refusal OR a different key; identical success output = ignored
    assert!(
        a.code != 0 || a.stdout != base.stdout,
        "--path silently ignored: {}",
        a.stdout
    );
}

#[test]
fn r1_path_shared_by_xpub_and_wif() {
    let o = convert(&[
        "--from",
        &phrase(),
        "--network",
        "signet",
        "--to",
        "xpub,wif",
        "--path",
        "m/87'/1'/0'",
    ]);
    assert_ok(&o, &[format!("xpub: {XPUB_87}"), format!("wif: {WIF_87}")]);
}

#[test]
fn r1_phrase_bip48_path_xpub_xprv_match_wif() {
    let o = convert(&[
        "--from",
        &phrase(),
        "--network",
        "signet",
        "--to",
        "xpub,xprv,wif",
        "--path",
        "m/48'/1'/0'/2'",
    ]);
    assert_ok(
        &o,
        &[
            format!("xpub: {XPUB_48}"),
            format!("xprv: {XPRV_48}"),
            format!("wif: {WIF_48}"),
        ],
    );
}

// ---- R2: fingerprint stays the master fingerprint; --path satisfies the requirement.

#[test]
fn r2_path_fingerprint_is_master() {
    let o = convert(&[
        "--from",
        &phrase(),
        "--network",
        "signet",
        "--to",
        "xpub,fingerprint",
        "--path",
        "m/87'/1'/0'",
    ]);
    assert_ok(
        &o,
        &[
            format!("xpub: {XPUB_87}"),
            "fingerprint: 73c5da0a".to_string(),
        ],
    );
}

#[test]
fn r2_neither_template_nor_path_names_both() {
    let o = convert(&["--from", &phrase(), "--network", "signet", "--to", "xpub"]);
    assert_refused(
        &o,
        1,
        "error: --template or --path is required for derivation targets (xpub/xprv/fingerprint)",
    );
}

// ---- R3: non-zero --account with --path is refused; explicit --account 0 is accepted.

#[test]
fn r3_account_nonzero_with_path_refused_wif() {
    let o = convert(&[
        "--from",
        &phrase(),
        "--network",
        "signet",
        "--to",
        "wif",
        "--path",
        "m/84'/1'/0'",
        "--account",
        "3",
    ]);
    assert_refused(&o, 2, R3_TEXT);
}

#[test]
fn r3_account_nonzero_with_path_refused_template_xpub() {
    let o = convert(&[
        "--from",
        &phrase(),
        "--network",
        "signet",
        "--template",
        "wsh-sortedmulti",
        "--account",
        "3",
        "--to",
        "xpub",
        "--path",
        "m/87'/1'/3'",
    ]);
    assert_refused(&o, 2, R3_TEXT);
}

#[test]
fn r3_guard_explicit_account_zero_with_path_accepted() {
    let o = convert(&[
        "--from",
        &phrase(),
        "--network",
        "signet",
        "--to",
        "wif",
        "--path",
        "m/84'/1'/0'",
        "--account",
        "0",
    ]);
    assert_ok(
        &o,
        &["wif: cRR3sz6fBG6cWeuJE8cRngfY4LGVzXvqxTSFqRPeak5k8beCGBX8".to_string()],
    );
}

// ---- R4: --path read by no requested target is refused (exit 2).

#[test]
fn r4_xprv_to_xpub_path_refused() {
    let from = format!("xprv={XPRV_84}");
    let o = convert(&[
        "--from",
        &from,
        "--network",
        "signet",
        "--to",
        "xpub",
        "--path",
        "m/0",
    ]);
    assert_refused(&o, 2, R4_TEXT);
}

#[test]
fn r4_xprv_to_fingerprint_path_refused() {
    let from = format!("xprv={XPRV_84}");
    let o = convert(&[
        "--from",
        &from,
        "--network",
        "signet",
        "--to",
        "fingerprint",
        "--path",
        "m/0",
    ]);
    assert_refused(&o, 2, R4_TEXT);
}

#[test]
fn r4_phrase_to_fingerprint_path_refused() {
    let o = convert(&[
        "--from",
        &phrase(),
        "--network",
        "signet",
        "--template",
        "bip84",
        "--to",
        "fingerprint",
        "--path",
        "m/84'/1'/0'",
    ]);
    assert_refused(&o, 2, R4_TEXT);
}

#[test]
fn r4_phrase_to_entropy_path_refused() {
    let o = convert(&["--from", &phrase(), "--to", "entropy", "--path", "m/0"]);
    assert_refused(&o, 2, R4_TEXT);
}

#[test]
fn r4_seedqr_to_phrase_path_refused() {
    let from = format!("seedqr={SEEDQR}");
    let o = convert(&["--from", &from, "--to", "phrase", "--path", "m/0"]);
    assert_refused(&o, 2, R4_TEXT);
}

#[test]
fn r4_entropy_to_phrase_path_refused() {
    let from = format!("entropy={ENTROPY}");
    let o = convert(&["--from", &from, "--to", "phrase", "--path", "m/0"]);
    assert_refused(&o, 2, R4_TEXT);
}

#[test]
fn r4_xpub_to_fingerprint_path_refused() {
    let from = format!("xpub={XPUB_84}");
    let o = convert(&[
        "--from",
        &from,
        "--network",
        "signet",
        "--to",
        "fingerprint",
        "--path",
        "m/0/0",
    ]);
    assert_refused(&o, 2, R4_TEXT);
}

#[test]
fn r4_mk1_to_xpub_path_refused() {
    let from = format!("mk1={MK1_84}");
    let o = convert(&["--from", &from, "--to", "xpub", "--path", "m/0"]);
    assert_refused(&o, 2, R4_TEXT);
}

#[test]
fn r4_ms1_to_phrase_path_refused() {
    let from = format!("ms1={MS1}");
    let o = convert(&["--from", &from, "--to", "phrase", "--path", "m/0"]);
    assert_refused(&o, 2, R4_TEXT);
}

#[test]
fn r4_guard_xpub_to_address_path_accepted() {
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
    ]);
    assert_ok(
        &o,
        &["address: tb1q6rz28mcfaxtmd6v789l9rrlrusdprr9pqcpvkl".to_string()],
    );
}

// ---- R5: literal `m/...` only (bare `m` included); named/hex forms are parse errors.

#[test]
fn r5_named_path_form_refused() {
    let o = convert(&[
        "--from",
        &phrase(),
        "--network",
        "signet",
        "--template",
        "bip84",
        "--to",
        "xpub",
        "--path",
        "bip84",
    ]);
    assert_refused(&o, 1, R5_TEXT);
}

#[test]
fn r5_hex_path_form_refused() {
    let o = convert(&[
        "--from",
        &phrase(),
        "--network",
        "signet",
        "--template",
        "bip84",
        "--to",
        "xpub",
        "--path",
        "0x54",
    ]);
    assert_refused(&o, 1, R5_TEXT);
}

#[test]
fn r5_bare_m_is_master() {
    let o = convert(&[
        "--from",
        &phrase(),
        "--network",
        "signet",
        "--to",
        "xpub",
        "--path",
        "m",
    ]);
    assert_ok(&o, &[format!("xpub: {XPUB_M}")]);
}

// ---- Help text (R5 drops the named/hex lines; R3 extends --account).

fn help() -> String {
    let o = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["convert", "--help"])
        .output()
        .unwrap();
    assert!(o.status.success());
    String::from_utf8(o.stdout).unwrap()
}

#[test]
fn r5_help_path_block_literal_only() {
    let h = help();
    let lines: Vec<&str> = h.lines().map(str::trim).collect();
    let want = [
        "--path <PATH>",
        "BIP-32 derivation path from the master key, literal form `m/...`. Overrides the",
        "--template/--account path for --to xpub/xprv; required for --to wif, --to address",
        "(from a seed) and composite --to bip38. Refused where no requested target uses it.",
    ];
    let at = lines
        .iter()
        .position(|l| *l == want[0])
        .expect("--path <PATH> line");
    assert_eq!(&lines[at..at + 4], &want, "help:\n{h}");
    assert!(
        !h.contains("named:") && !h.contains("hex:"),
        "named/hex forms still listed:\n{h}"
    );
}

#[test]
fn r3_help_account_refused_with_path() {
    let h = help();
    assert!(
        h.lines().any(|l| l.trim()
            == "BIP-32 account index (default 0). Used to compute the template-derived path when `--path` is omitted. Refused (non-zero) together with --path"),
        "help:\n{h}"
    );
}

// QT1-QT3 precedence guard rows (architect bughunt-fixdocs).
#[test]
fn r4_before_r3_xprv_to_xpub_path_and_account() {
    let from = format!("xprv={XPRV_84}");
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
    assert_refused(&o, 2, R4_TEXT);
}

#[test]
fn r4_before_xpub_prefix_network() {
    let o = convert(&[
        "--from",
        &phrase(),
        "--to",
        "fingerprint",
        "--path",
        "m/0",
        "--xpub-prefix",
        "zpub",
    ]);
    assert_refused(&o, 2, R4_TEXT);
}

#[test]
fn r3_before_r5_named_path_with_account() {
    let o = convert(&[
        "--from",
        &phrase(),
        "--network",
        "signet",
        "--to",
        "wif",
        "--path",
        "bip84",
        "--account",
        "3",
    ]);
    assert_refused(&o, 2, R3_TEXT);
}

#[test]
fn s2_path_alone_xprv_accepted() {
    let o = convert(&[
        "--from",
        &phrase(),
        "--network",
        "signet",
        "--to",
        "xprv",
        "--path",
        "m/84'/1'/0'",
    ]);
    assert_ok(&o, &[format!("xprv: {XPRV_84}")]);
}

#[test]
fn s2_path_alone_bip38_accepted() {
    let o = convert(&[
        "--from",
        &phrase(),
        "--to",
        "bip38",
        "--bip38-passphrase",
        "x",
        "--path",
        "m/84'/0'/0'/0/0",
    ]);
    assert_ok(
        &o,
        &["bip38: 6PYP8fdoWkbfb1nGtgRPgSRUsHK113HqGCvrY6YHWMm3BvnKGRnH564c8J".to_string()],
    );
}

#[test]
fn s2_path_alone_address_accepted() {
    let o = convert(&[
        "--from",
        &phrase(),
        "--network",
        "signet",
        "--to",
        "address",
        "--script-type",
        "p2wpkh",
        "--path",
        "m/84'/1'/0'/0/0",
    ]);
    assert_ok(
        &o,
        &["address: tb1q6rz28mcfaxtmd6v789l9rrlrusdprr9pqcpvkl".to_string()],
    );
}

#[test]
fn s2_account_one_with_path_refused() {
    let o = convert(&[
        "--from",
        &phrase(),
        "--network",
        "signet",
        "--to",
        "xpub",
        "--path",
        "m/84'/1'/0'",
        "--account",
        "1",
    ]);
    assert_refused(&o, 2, R3_TEXT);
}

#[test]
fn s2_path_honours_passphrase() {
    let o = convert(&[
        "--from",
        &phrase(),
        "--network",
        "signet",
        "--to",
        "xpub,xprv",
        "--passphrase",
        "TREZOR",
        "--path",
        "m/84'/1'/0'",
    ]);
    assert_ok(&o, &[
        "xpub: tpubDC3QUzyNJLFENW1qz75ATkef1wkGURb2bvPiBd7ty35wpR9oTzV12aHjzWFr9eZDnL5H9Wqbq6riaHcfa4FR4iv22uFLufe7Ldor9jr2MJF".to_string(),
        "xprv: tprv8fMNLaw89xZZV2z46TQa4LzYSvELK6Q82cnvu75bYmHYyvu2qbfQr5fspNnjRRF9D4Fte48koK2tWB6ya3xY43KdPvTmFiH76h8W48rLLRQ".to_string(),
    ]);
}

#[test]
fn s2_path_honours_language() {
    let o = convert(&[
        "--from",
        "phrase=ábaco ábaco ábaco ábaco ábaco ábaco ábaco ábaco ábaco ábaco ábaco abierto",
        "--language",
        "spanish",
        "--network",
        "signet",
        "--to",
        "xpub",
        "--path",
        "m/84'/1'/0'",
    ]);
    assert_ok(&o, &["xpub: tpubDCbgL9wqirauBJJQHwtmGoTJx7rqvTAJ1PabwjpnVCu2prdzKRseAyqBsDFpPjstykf4k5LCZpg69GaDD6pXuzddbphbZv6BRuczCnPyKvG".to_string()]);
}

// ---- whole-diff s1 additions: literal path form (I1) and fingerprint-only refusal (I2).

#[test]
fn r5_empty_path_refused() {
    let p = phrase();
    let o = convert(&[
        "--from",
        &p,
        "--network",
        "signet",
        "--to",
        "xpub",
        "--path",
        "",
    ]);
    assert_refused(&o, 1, R5_FORM_TEXT);
    assert!(o.stdout.is_empty(), "stdout: {}", o.stdout);
}

#[test]
fn r5_m_slash_path_refused() {
    let p = phrase();
    let o = convert(&[
        "--from",
        &p,
        "--network",
        "signet",
        "--to",
        "wif",
        "--path",
        "m/",
    ]);
    assert_refused(&o, 1, R5_FORM_TEXT);
    assert!(o.stdout.is_empty(), "stdout: {}", o.stdout);
}

#[test]
fn r5_unprefixed_path_refused() {
    let p = phrase();
    let o = convert(&[
        "--from",
        &p,
        "--network",
        "signet",
        "--to",
        "xpub",
        "--path",
        "84'/1'/0'",
    ]);
    assert_refused(&o, 1, R5_FORM_TEXT);
    assert!(o.stdout.is_empty(), "stdout: {}", o.stdout);
}

#[test]
fn r4_fingerprint_only_no_template_names_template_only() {
    let p = phrase();
    let o = convert(&["--from", &p, "--network", "signet", "--to", "fingerprint"]);
    assert_refused(
        &o,
        1,
        "error: --template is required for --to fingerprint (the master fingerprint; --path is not read)",
    );
    assert!(o.stdout.is_empty(), "stdout: {}", o.stdout);
}
