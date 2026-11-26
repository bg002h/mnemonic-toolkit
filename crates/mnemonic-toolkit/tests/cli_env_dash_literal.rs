//! bughunt-1119 bh-tk-I5 (design/FIX-tk-env-dash-stdin-order.md): an `@env:VAR` value of `-` is
//! that literal value, never a request to read stdin. One test per FIX row (section 5/6).
//! Public test phrase and signet cards only; variables are set with `.env(...)` on the child.

use assert_cmd::Command;

const P: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const X: &str = "throw enter benefit planet indoor happy cheap exhibit rule aisle then average";
const M1: &str = "ms126mq5qqwxr08thfjy39dcehlnxfwf5fxhs8whrj2rsryask";
const M2: &str = "ms126mq5pq4fsz7v0tkwxe9d60npft4tltf0383kdy72303gps";
const L1: &str = "else garden academic acid crowd spew exchange thunder medal station galaxy cubic military cinema evaluate database saver petition unwrap racism";
const L2: &str = "else garden academic agency away oasis decorate swimming quarter parking roster sister energy crucial amount cluster segment estate prune spirit";
const MS: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
const K1: &str = "mk1qpg397pqqsq5gjlze9eutks2zvzrtp70pm6trteu3ssr0mjvzcsa5rf53k63zcmsnf3z6rfg8rw7dkzpn3glvvquvgpms6mzamgctgratycf";
const K2: &str = "mk1qpg397pp3c8mu0myvvm7myaups8nhpplea7jtz09ajyyw48xgqsz02ysk3l6fyxg6m66tmvhzh70a";
const D1: &str = "md1fkwsupqpqpm6j8pqqxqqks883w6pg25zpu3ssr0mjvzcsa5rf5sfn48qvmn2cutt";
const D2: &str = "md1fkwsupqtd4z93hpxnz95xjswxaumvyr8z37ccpccsrhz8ql03lv04y7x3v7nh0ze";
const D3: &str = "md1fkwsupq3rrxlke80qvpuacg07005jcne0v3pr4fejqyqn63y95qcngq49s9nrnck";

const WC: &str = "error: BIP-39 word count 1 invalid (must be 12, 15, 18, 21, or 24)";
const R6_ERR: &str = "error: seed-xor combine: share at position 0 has invalid BIP-39 checksum (not a parseable mnemonic in --language english): mnemonic has an invalid word count: 1. Word count must be 12, 15, 18, 21, or 24";
const SLIP_SPLIT_ERR: &str =
    "error: slip39 split: input phrase must be 12/15/18/21/24 words; got 1";
const SLIP_COMBINE_ERR: &str =
    "error: slip39 combine: share at position 0: word at index 0 not in SLIP-39 wordlist";
const ADV_STEM: &str = "secret material on argv";

struct Out {
    rc: i32,
    stdout: String,
    stderr: String,
}

fn run(args: &[&str], stdin: &str, env: &[(&str, &str)]) -> Out {
    let mut c = Command::cargo_bin("mnemonic").unwrap();
    c.args(args).write_stdin(format!("{stdin}\n"));
    for (k, v) in env {
        c.env(k, v);
    }
    let o = c.output().unwrap();
    Out {
        rc: o.status.code().unwrap(),
        stdout: String::from_utf8(o.stdout).unwrap(),
        stderr: String::from_utf8(o.stderr).unwrap(),
    }
}

fn dash(args: &[&str], stdin: &str) -> Out {
    run(args, stdin, &[("E", "-")])
}

/// rc=1, empty stdout, `line` present as a whole stderr line.
fn refused(o: &Out, line: &str) {
    assert_eq!(o.rc, 1, "stderr: {}", o.stderr);
    assert_eq!(o.stdout, "", "stdout must be empty");
    assert!(
        o.stderr.lines().any(|l| l == line),
        "expected stderr line {line:?}; got: {}",
        o.stderr
    );
}

fn verify_args<'a>(slot: &'a str, extra: &'a [&'a str]) -> Vec<&'a str> {
    let mut v = vec![
        "verify-bundle",
        "--allow-argv-secret",
        "--scheme",
        "bip84",
        "--network",
        "signet",
        "--ms1",
        MS,
        "--mk1",
        K1,
        K2,
        "--md1",
        D1,
        D2,
        D3,
        "--slot",
        slot,
    ];
    v.extend_from_slice(extra);
    v
}

// ---- R: an @env: value of `-` must not read stdin ----

#[test]
fn r1_convert_env_dash_is_literal() {
    refused(
        &dash(
            &["convert", "--from", "phrase=@env:E", "--to", "entropy"],
            P,
        ),
        WC,
    );
}

#[test]
fn r2_derive_child_env_dash_is_literal() {
    let a = [
        "derive-child",
        "--from",
        "phrase=@env:E",
        "--application",
        "hex",
        "--length",
        "16",
        "--index",
        "0",
    ];
    refused(&dash(&a, P), WC);
}

#[test]
fn r3_bundle_slot_env_dash_is_literal() {
    let a = [
        "bundle",
        "--scheme",
        "bip84",
        "--network",
        "signet",
        "--slot",
        "@0.phrase=@env:E",
    ];
    refused(&dash(&a, P), WC);
}

#[test]
fn r4_verify_bundle_slot_env_dash_is_literal() {
    refused(&dash(&verify_args("@0.phrase=@env:E", &[]), P), WC);
}

#[test]
fn r5_seed_xor_split_env_dash_is_literal() {
    let a = [
        "seed-xor",
        "split",
        "--from",
        "phrase=@env:E",
        "--shares",
        "2",
        "--deterministic-from-master",
    ];
    refused(&dash(&a, P), WC);
}

#[test]
fn r6_seed_xor_combine_env_dash_is_literal() {
    let sx = format!("phrase={X}");
    let a = [
        "seed-xor",
        "combine",
        "--allow-argv-secret",
        "--shares",
        "2",
        "--share",
        "phrase=@env:E",
        "--share",
        &sx,
    ];
    refused(&dash(&a, X), R6_ERR);
}

#[test]
fn r7_ms_shares_split_env_dash_is_literal() {
    let a = [
        "ms-shares",
        "split",
        "--from",
        "phrase=@env:E",
        "--threshold",
        "2",
        "--shares",
        "3",
    ];
    refused(
        &dash(&a, P),
        "error: ms-shares split: input phrase must be 12/15/18/21/24 words; got 1",
    );
}

#[test]
fn r8_ms_shares_combine_env_dash_is_literal() {
    let a = [
        "ms-shares",
        "combine",
        "--allow-argv-secret",
        "--share",
        "@env:E",
        "--share",
        M2,
    ];
    refused(
        &dash(&a, M1),
        "error: ms1 not enough shares: have 1, need 2",
    );
}

#[test]
fn r9_slip39_split_env_dash_is_literal() {
    let a = [
        "slip39",
        "split",
        "--from",
        "phrase=@env:E",
        "--group-threshold",
        "1",
        "--group",
        "3,2",
    ];
    refused(&dash(&a, P), SLIP_SPLIT_ERR);
}

#[test]
fn r10_slip39_combine_env_dash_is_literal() {
    let a = [
        "slip39",
        "combine",
        "--allow-argv-secret",
        "--share",
        "@env:E",
        "--share",
        L2,
    ];
    refused(&dash(&a, L1), SLIP_COMBINE_ERR);
}

// ---- S: single-stdin guards judge the value as written ----

#[test]
fn s1_derive_child_env_dash_not_a_stdin_reader() {
    let a = [
        "derive-child",
        "--from",
        "phrase=@env:E",
        "--passphrase-stdin",
        "--application",
        "hex",
        "--length",
        "16",
        "--index",
        "0",
    ];
    refused(&dash(&a, ""), WC);
}

#[test]
fn s2_convert_env_dash_not_a_stdin_reader() {
    let a = [
        "convert",
        "--from",
        "phrase=@env:E",
        "--passphrase-stdin",
        "--to",
        "entropy",
    ];
    refused(&dash(&a, ""), WC);
}

#[test]
fn s3_bundle_slot_env_dash_not_a_stdin_reader() {
    let a = [
        "bundle",
        "--scheme",
        "bip84",
        "--network",
        "signet",
        "--slot",
        "@0.phrase=@env:E",
        "--passphrase-stdin",
    ];
    refused(&dash(&a, ""), WC);
}

#[test]
fn s4_seed_xor_combine_two_env_dashes_not_stdin_readers() {
    let a = [
        "seed-xor",
        "combine",
        "--shares",
        "2",
        "--share",
        "phrase=@env:E",
        "--share",
        "phrase=@env:E",
    ];
    refused(&dash(&a, X), R6_ERR);
}

#[test]
fn s5_slip39_split_env_dash_not_a_stdin_reader() {
    let a = [
        "slip39",
        "split",
        "--from",
        "phrase=@env:E",
        "--passphrase-stdin",
        "--group-threshold",
        "1",
        "--group",
        "3,2",
    ];
    refused(&dash(&a, ""), SLIP_SPLIT_ERR);
}

#[test]
fn s6_slip39_combine_two_env_dashes_not_stdin_readers() {
    let a = [
        "slip39", "combine", "--share", "@env:E", "--share", "@env:E",
    ];
    refused(&dash(&a, L1), SLIP_COMBINE_ERR);
}

#[test]
fn s7_ms_shares_combine_two_env_dashes_not_stdin_readers() {
    let a = [
        "ms-shares",
        "combine",
        "--share",
        "@env:E",
        "--share",
        "@env:E",
    ];
    refused(
        &dash(&a, M1),
        "error: ms-shares combine: at least one --share required",
    );
}

#[test]
fn s8_bundle_literal_slot_and_env_dash_slot() {
    let a = [
        "bundle",
        "--scheme",
        "wsh-sortedmulti",
        "--threshold",
        "1",
        "--network",
        "signet",
        "--slot",
        "@0.phrase=-",
        "--slot",
        "@1.phrase=@env:E",
    ];
    refused(&dash(&a, P), WC);
}

#[test]
fn s9_verify_bundle_slot_env_dash_not_a_stdin_reader() {
    refused(
        &dash(
            &verify_args("@0.phrase=@env:E", &["--passphrase-stdin"]),
            "",
        ),
        WC,
    );
}

// ---- A: the split argv advisory judges the value as written ----

fn no_advisory(args: &[&str]) {
    let o = run(args, "", &[("G", P)]);
    assert_eq!(o.rc, 0, "stderr: {}", o.stderr);
    assert!(
        !o.stderr.lines().any(|l| l.contains(ADV_STEM)),
        "unexpected argv advisory: {}",
        o.stderr
    );
}

#[test]
fn a1_seed_xor_split_env_source_no_argv_advisory() {
    no_advisory(&[
        "seed-xor",
        "split",
        "--from",
        "phrase=@env:G",
        "--shares",
        "2",
        "--deterministic-from-master",
    ]);
}

#[test]
fn a2_ms_shares_split_env_source_no_argv_advisory() {
    no_advisory(&[
        "ms-shares",
        "split",
        "--from",
        "phrase=@env:G",
        "--threshold",
        "2",
        "--shares",
        "3",
    ]);
}

#[test]
fn a3_slip39_split_env_source_no_argv_advisory() {
    no_advisory(&[
        "slip39",
        "split",
        "--from",
        "phrase=@env:G",
        "--group-threshold",
        "1",
        "--group",
        "3,2",
    ]);
}

// ---- C: controls (pass at the base and after the fix) ----

#[test]
fn c1_convert_literal_dash_reads_stdin() {
    let o = run(
        &["convert", "--from", "phrase=-", "--to", "entropy"],
        P,
        &[("E", "-")],
    );
    assert_eq!(o.rc, 0, "stderr: {}", o.stderr);
    assert!(
        o.stdout
            .lines()
            .next()
            .unwrap()
            .starts_with("entropy: 0000"),
        "{}",
        o.stdout
    );
}

#[test]
fn c2_convert_env_phrase_works() {
    let o = run(
        &["convert", "--from", "phrase=@env:G", "--to", "entropy"],
        "",
        &[("G", P)],
    );
    assert_eq!(o.rc, 0, "stderr: {}", o.stderr);
    assert!(
        o.stdout
            .lines()
            .next()
            .unwrap()
            .starts_with("entropy: 0000"),
        "{}",
        o.stdout
    );
}

#[test]
fn c3_bundle_slot_literal_dash_reads_stdin() {
    let a = [
        "bundle",
        "--scheme",
        "bip84",
        "--network",
        "signet",
        "--slot",
        "@0.phrase=-",
    ];
    let o = run(&a, P, &[("E", "-")]);
    assert_eq!(o.rc, 0, "stderr: {}", o.stderr);
    assert_eq!(
        o.stdout.lines().next().unwrap(),
        "# ms1 (entropy, BCH-checksummed)"
    );
}

#[test]
fn c4_stdin_text_is_not_env_resolved() {
    let o = run(
        &["convert", "--from", "phrase=-", "--to", "entropy"],
        "@env:G",
        &[("G", P)],
    );
    refused(&o, WC);
}

#[test]
fn c5_bundle_stdin_text_is_not_env_resolved() {
    let a = [
        "bundle",
        "--scheme",
        "bip84",
        "--network",
        "signet",
        "--slot",
        "@0.phrase=-",
    ];
    refused(&run(&a, "@env:G", &[("G", P)]), WC);
}

#[test]
fn c6_ms_shares_split_inline_keeps_argv_advisory() {
    let from = format!("phrase={P}");
    let a = [
        "ms-shares",
        "split",
        "--allow-argv-secret",
        "--from",
        &from,
        "--threshold",
        "2",
        "--shares",
        "3",
    ];
    let o = run(&a, "", &[]);
    assert_eq!(o.rc, 0, "stderr: {}", o.stderr);
    assert!(
        o.stderr.lines().any(|l| l.contains(ADV_STEM)),
        "{}",
        o.stderr
    );
}

fn stdin_conflict_absent(o: &Out) {
    assert!(!o.stderr.contains("cannot coexist"), "{}", o.stderr);
    assert!(
        !o.stderr.contains("only one input can come from stdin"),
        "{}",
        o.stderr
    );
}

fn ok(o: &Out) {
    assert_eq!(o.rc, 0, "stderr: {}", o.stderr);
}
#[test]
fn n1_convert_bip38_env_dash_not_a_stdin_reader() {
    let o = dash(
        &[
            "convert",
            "--from",
            "wif=@env:E",
            "--to",
            "bip38",
            "--bip38-passphrase-stdin",
        ],
        "",
    );
    stdin_conflict_absent(&o);
    assert!(
        o.stderr
            .lines()
            .any(|l| l == "error: --from wif parse: invalid base58"),
        "{}",
        o.stderr
    );
}

#[test]
fn n3b_verify_bundle_literal_slot_and_env_dash_slot() {
    let a = [
        "verify-bundle",
        "--allow-argv-secret",
        "--scheme",
        "wsh-sortedmulti",
        "--threshold",
        "1",
        "--network",
        "signet",
        "--ms1",
        MS,
        "--mk1",
        K1,
        "--md1",
        D1,
        "--slot",
        "@0.phrase=-",
        "--slot",
        "@1.phrase=@env:E",
    ];
    refused(&dash(&a, P), WC);
}

#[test]
fn n10_bundle_env_slot_then_literal_dash_slot_reads_stdin_into_slot_1() {
    let o = run(
        &[
            "bundle",
            "--scheme",
            "wsh-sortedmulti",
            "--threshold",
            "1",
            "--multisig-path-family",
            "bip87",
            "--network",
            "signet",
            "--slot",
            "@0.phrase=@env:E",
            "--slot",
            "@1.phrase=-",
        ],
        X,
        &[("E", P)],
    );
    ok(&o);
    assert!(o.stdout.contains("# ms1[1]"), "{}", o.stdout);
}

#[test]
fn n11_convert_side_input_before_literal_dash_primary() {
    let o = dash(
        &[
            "convert",
            "--from",
            "fingerprint=73c5da0a",
            "--from",
            "phrase=-",
            "--to",
            "entropy",
        ],
        P,
    );
    ok(&o);
    assert!(
        o.stdout
            .lines()
            .next()
            .unwrap()
            .starts_with("entropy: 0000"),
        "{}",
        o.stdout
    );
}
