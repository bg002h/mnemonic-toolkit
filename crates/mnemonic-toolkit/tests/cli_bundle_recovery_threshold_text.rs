//! Triage V7 (2026-11-03): `mnemonic bundle --descriptor` prints a false
//! `# Threshold: K of N` header line and a false `# Recovery: any K of N
//! signing keys + md1` line for any descriptor whose spend condition is not a
//! single plain `multi`/`sortedmulti`/`multi_a`/`sortedmulti_a` root.
//!
//! Cause: `cmd/bundle.rs::build_unified_card` takes K from
//! `extract_multisig_threshold(&tree)`, which returns the FIRST multi-family or
//! `thresh` node found in a depth-first walk (or `None`), and then falls back
//! to N. It never asks whether that node is the whole spend condition.
//! (`restore --md1` already knows this: it prints a threshold only for the
//! plain arm, `restore.rs` `threshold_field`.)
//!
//! Oracle: Bitcoin Core 31.1 on regtest. For every shape below, every subset of
//! the keys was tried as the only private keys in a wallet that imported the
//! descriptor; the subset spends iff `walletprocesspsbt` + `finalizepsbt` is
//! `complete`. Minimal spending key sets found:
//!
//! | descriptor                                         | minimal spending sets | printed (02052a17)  |
//! |----------------------------------------------------|-----------------------|---------------------|
//! | wsh(sortedmulti(2,K0,K1,K2))                       | any 2 of 3            | 2 of 3 (correct)    |
//! | wsh(multi(2,K0,K1,K2))                             | any 2 of 3            | 2 of 3 (correct)    |
//! | tr(NUMS,sortedmulti_a(2,K0,K1,K2))                 | any 2 of 3            | 2 of 3 (correct)    |
//! | tr(NUMS,multi_a(2,K0,K1,K2))                       | any 2 of 3            | 2 of 3 (correct)    |
//! | tr(NUMS,{pk(K0),pk(K1)})                           | {0} {1}               | 2 of 2              |
//! | tr(NUMS,{pk(K0),{pk(K1),pk(K2)}})                  | {0} {1} {2}           | 3 of 3              |
//! | wsh(or_d(pk(K0),pk(K1)))                           | {0} {1}               | 2 of 2              |
//! | wsh(or_d(pk(K2),multi(2,K0,K1)))                   | {2} {0,1}             | 2 of 3              |
//! | tr(NUMS,{multi_a(2,K0,K1),pk(K2)})                 | {2} {0,1}             | 2 of 3              |
//! | tr(NUMS,{sortedmulti_a(2,K0,K1),pk(K2)})           | {2} {0,1}             | 2 of 3              |
//! | wsh(or_i(multi(2,K0,K1),pk(K2)))                   | {2} {0,1}             | 2 of 3              |
//! | tr(K0,multi_a(2,K1,K2))  (non-NUMS internal key)   | {0} {1,2}             | 2 of 3              |
//! | wsh(and_v(v:pk(K0),or_d(pk(K1),pk(K2))))           | {0,1} {0,2}           | 3 of 3              |
//! | tr(NUMS,{multi_a(2,K0,K1),multi_a(2,K2,K3)})       | {0,1} {2,3}           | 2 of 4 "any 2"      |
//! | wsh(or_d(multi(2,K0,K1),multi(2,K2,K3)))           | {0,1} {2,3}           | 2 of 4 "any 2"      |
//! | wsh(and_v(v:multi(2,K0,K1),multi(2,K2,K3)))        | {0,1,2,3} only        | 2 of 4 (UNDERSTATED)|
//!
//! The rows above assert only that the false claim is gone. FIX-V7
//! (`design/FIX_v7_bundle_recovery.md`, grade rulings 1-3) decides what
//! replaces it: a descriptor is plain when its only child under `wsh`, `sh` or
//! `sh(wsh)`, or the only leaf of a `tr` whose internal key is the NUMS point,
//! is a multi-family node or a pure-key `thresh` (every argument `pk`/`pkh`
//! with only `s:`/`a:` wrappers). Plain keeps both lines. Any other descriptor
//! with n > 1 prints no `# Threshold` line and the general Recovery line
//! (`general_line`); its JSON `multisig.threshold` is null, in `bundle` and in
//! `import-wallet` alike, and a null-threshold envelope is accepted. n == 1
//! prints neither line. The FIX rows below pin those texts exactly.

use assert_cmd::Command;
use serde_json::Value;
use std::path::{Path, PathBuf};

const K0: &str = "[b8688df1/48'/0'/0'/2']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*";
const K1: &str = "[5436d724/48'/0'/0'/2']xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx/<0;1>/*";
const K2: &str = "[28645006/48'/0'/0'/2']xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6/<0;1>/*";
const K3: &str = "[16a93ed0/48'/0'/0'/2']xpub6Bv8ayijom26yJ1wZ62h4X1smfYBfNeNtGujxw6vaY4zq4Tw4cn2oV8qZmjnuVxh56oSe21r7V8r9LjZjArFh3QRZQbgzgLcfjVikZNa86W/<0;1>/*";
const NUMS: &str = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";
const K4: &str = "[1cf0bf7e/48'/0'/0'/2']xpub6FL8FhxNNUVnG64YurPd16AfGyvFLhh7S2uSsDqR3Qfcm6o9jtcMYwh6DvmcBF9qozxNQmTCVvWtxLpKTnhVLN3Pgnu2D3pAoXYFgVyd8Yz/<0;1>/*";
const HH: &str = "0000000000000000000000000000000000000000000000000000000000000001";

/// Substitute `K0..K4`, `NUMS` and `HH` in a template.
fn subst(template: &str) -> String {
    let mut d = template.to_string();
    for (name, val) in [
        ("K0", K0),
        ("K1", K1),
        ("K2", K2),
        ("K3", K3),
        ("K4", K4),
        ("NUMS", NUMS),
        ("HH", HH),
    ] {
        d = d.replace(name, val);
    }
    d
}

/// Substitute the keys in a template and run `bundle --descriptor`; return
/// stderr (where the card text goes). Asserts exit 0.
fn bundle_stderr(template: &str) -> String {
    let d = subst(template);
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["bundle", "--network", "mainnet", "--descriptor", &d])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "bundle {template} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stderr).unwrap()
}

fn assert_no_false_claim(template: &str, false_kn: &str) {
    let e = bundle_stderr(template);
    assert!(
        !e.contains(&format!("# Threshold: {false_kn}")),
        "{template}: card prints the false '# Threshold: {false_kn}':\n{e}"
    );
    assert!(
        !e.contains(&format!("# Recovery: any {false_kn} ")),
        "{template}: card prints the false '# Recovery: any {false_kn} signing keys':\n{e}"
    );
}

// ---- RED rows: the printed claim is false ---------------------------------

#[test]
fn v7_tr_two_pk_leaves_not_2_of_2() {
    // Core: either key alone spends.
    assert_no_false_claim("tr(NUMS,{pk(K0),pk(K1)})", "2 of 2");
}

#[test]
fn v7_tr_three_pk_leaves_depth2_not_3_of_3() {
    assert_no_false_claim("tr(NUMS,{pk(K0),{pk(K1),pk(K2)}})", "3 of 3");
}

#[test]
fn v7_wsh_or_d_two_keys_not_2_of_2() {
    assert_no_false_claim("wsh(or_d(pk(K0),pk(K1)))", "2 of 2");
}

#[test]
fn v7_wsh_or_d_key_or_multi_not_2_of_3() {
    // Core: K2 alone spends.
    assert_no_false_claim("wsh(or_d(pk(K2),multi(2,K0,K1)))", "2 of 3");
}

#[test]
fn v7_wsh_or_i_multi_or_key_not_2_of_3() {
    assert_no_false_claim("wsh(or_i(multi(2,K0,K1),pk(K2)))", "2 of 3");
}

#[test]
fn v7_tr_multi_a_leaf_plus_pk_leaf_not_2_of_3() {
    // Core: K2 alone spends.
    assert_no_false_claim("tr(NUMS,{multi_a(2,K0,K1),pk(K2)})", "2 of 3");
}

#[test]
fn v7_tr_sortedmulti_a_leaf_plus_pk_leaf_not_2_of_3() {
    assert_no_false_claim("tr(NUMS,{sortedmulti_a(2,K0,K1),pk(K2)})", "2 of 3");
}

#[test]
fn v7_tr_non_nums_internal_key_plus_multi_a_not_2_of_3() {
    // Core: the internal key K0 alone spends (key path).
    assert_no_false_claim("tr(K0,multi_a(2,K1,K2))", "2 of 3");
}

#[test]
fn v7_wsh_and_v_with_or_branch_not_3_of_3() {
    // Core: {K0,K1} or {K0,K2} (two keys), not all three.
    assert_no_false_claim("wsh(and_v(v:pk(K0),or_d(pk(K1),pk(K2))))", "3 of 3");
}

#[test]
fn v7_two_disjoint_multi_leaves_not_any_2_of_4() {
    // Core: {K0,K1} and {K2,K3} spend; {K0,K2} does not. "any 2 of 4" is false
    // even though the minimum K is 2.
    for t in [
        "tr(NUMS,{multi_a(2,K0,K1),multi_a(2,K2,K3)})",
        "wsh(or_d(multi(2,K0,K1),multi(2,K2,K3)))",
    ] {
        let e = bundle_stderr(t);
        assert!(
            !e.contains("# Recovery: any 2 of 4"),
            "{t}: card claims any 2 of 4 keys recover, but {{K0,K2}} cannot spend:\n{e}"
        );
    }
}

#[test]
fn v7_and_of_two_multis_not_2_of_4_understated() {
    // The dangerous direction: Core needs all four keys, the card says
    // "any 2 of 4".
    assert_no_false_claim("wsh(and_v(v:multi(2,K0,K1),multi(2,K2,K3)))", "2 of 4");
}

// ---- GREEN controls: plain shapes keep their (correct) lines --------------

#[test]
fn v7_control_plain_shapes_keep_threshold_and_recovery_lines() {
    for t in [
        "wsh(sortedmulti(2,K0,K1,K2))",
        "wsh(multi(2,K0,K1,K2))",
        "tr(NUMS,sortedmulti_a(2,K0,K1,K2))",
        "tr(NUMS,multi_a(2,K0,K1,K2))",
    ] {
        let e = bundle_stderr(t);
        assert!(
            e.contains("# Threshold: 2 of 3"),
            "{t}: missing Threshold line:\n{e}"
        );
        assert!(
            e.contains("# Recovery: any 2 of 3 signing keys + md1"),
            "{t}: missing Recovery line:\n{e}"
        );
    }
}

// ---- FIX-V7 rows (design/FIX_v7_bundle_recovery.md) -----------------------

/// The general Recovery line, exactly (grade ruling 1).
fn general_line(n: usize) -> String {
    format!(
        "# Recovery: general policy, not k-of-n (the md1 policy decides which key \
         sets can spend); keep all {n} signing keys, any hash preimages it names, \
         and md1 (template card)."
    )
}

/// A general card: no `# Threshold:` line, no `# Recovery: any` line, and
/// exactly one line equal to `general_line(n)`.
fn assert_general_card(card: &str, what: &str, n: usize) {
    let lines: Vec<&str> = card.lines().collect();
    assert!(
        !lines.iter().any(|l| l.starts_with("# Threshold:")),
        "{what}: a general policy must print no '# Threshold:' line:\n{card}"
    );
    assert!(
        !lines.iter().any(|l| l.starts_with("# Recovery: any")),
        "{what}: a general policy must print no '# Recovery: any' line:\n{card}"
    );
    let want = general_line(n);
    let hits = lines.iter().filter(|l| **l == want).count();
    assert_eq!(
        hits, 1,
        "{what}: want exactly one line\n{want}\ngot:\n{card}"
    );
}

/// A plain card: exactly one `# Threshold: k of n` line, exactly one
/// `# Recovery: any k of n ...` line, and no general line.
fn assert_plain_card(card: &str, what: &str, k: usize, n: usize) {
    let lines: Vec<&str> = card.lines().collect();
    let th = format!("# Threshold: {k} of {n}");
    let rec = format!("# Recovery: any {k} of {n} signing keys + md1 (template card).");
    assert_eq!(
        lines.iter().filter(|l| **l == th).count(),
        1,
        "{what}: want exactly one line '{th}':\n{card}"
    );
    assert_eq!(
        lines.iter().filter(|l| **l == rec).count(),
        1,
        "{what}: want exactly one line '{rec}':\n{card}"
    );
    assert!(
        !lines
            .iter()
            .any(|l| l.starts_with("# Recovery: general policy")),
        "{what}: a plain descriptor must not print the general line:\n{card}"
    );
}

/// `bundle --descriptor D --json`: the value of `multisig.threshold`, which
/// must be present (null is a value, not an omitted key).
fn bundle_json_threshold(template: &str) -> Value {
    let d = subst(template);
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--descriptor",
            &d,
            "--json",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "bundle --json {template} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    v["multisig"]
        .get("threshold")
        .unwrap_or_else(|| panic!("{template}: multisig.threshold key missing: {v}"))
        .clone()
}

/// `import-wallet --format descriptor --blob -` with D on stdin (plus
/// `--json` when asked). Asserts exit 0; returns stdout.
fn import_wallet_stdout(template: &str, json: bool) -> String {
    let d = subst(template);
    let mut args = vec!["import-wallet", "--format", "descriptor", "--blob", "-"];
    if json {
        args.push("--json");
    }
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .write_stdin(format!("{d}\n"))
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "import-wallet {template} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).unwrap()
}

/// Write an `import-wallet --json` envelope for D into `dir`, with
/// `[0].bundle.multisig.threshold` set to `threshold`; return its path.
fn envelope_file(dir: &Path, name: &str, template: &str, threshold: Value) -> PathBuf {
    let mut v: Value = serde_json::from_str(&import_wallet_stdout(template, true)).unwrap();
    v[0]["bundle"]["multisig"]["threshold"] = threshold;
    let p = dir.join(name);
    std::fs::write(&p, serde_json::to_string(&v).unwrap()).unwrap();
    p
}

/// `bundle --network mainnet --import-json F [extra..]`.
fn bundle_import_json(env: &Path, extra: &[&str]) -> std::process::Output {
    let mut args = vec![
        "bundle".to_string(),
        "--network".to_string(),
        "mainnet".to_string(),
        "--import-json".to_string(),
        env.to_str().unwrap().to_string(),
    ];
    args.extend(extra.iter().map(|s| s.to_string()));
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .output()
        .unwrap()
}

/// `export-wallet --from-import-json F --format X`.
fn export_from_envelope(env: &Path, format: &str) -> std::process::Output {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--from-import-json",
            env.to_str().unwrap(),
            "--format",
            format,
        ])
        .output()
        .unwrap()
}

/// The twelve triage shapes (the eleven rows above) with their key counts.
const GENERAL_SHAPES: [(&str, usize); 12] = [
    ("tr(NUMS,{pk(K0),pk(K1)})", 2),
    ("tr(NUMS,{pk(K0),{pk(K1),pk(K2)}})", 3),
    ("wsh(or_d(pk(K0),pk(K1)))", 2),
    ("wsh(or_d(pk(K2),multi(2,K0,K1)))", 3),
    ("wsh(or_i(multi(2,K0,K1),pk(K2)))", 3),
    ("tr(NUMS,{multi_a(2,K0,K1),pk(K2)})", 3),
    ("tr(NUMS,{sortedmulti_a(2,K0,K1),pk(K2)})", 3),
    ("tr(K0,multi_a(2,K1,K2))", 3),
    ("wsh(and_v(v:pk(K0),or_d(pk(K1),pk(K2))))", 3),
    ("tr(NUMS,{multi_a(2,K0,K1),multi_a(2,K2,K3)})", 4),
    ("wsh(or_d(multi(2,K0,K1),multi(2,K2,K3)))", 4),
    ("wsh(and_v(v:multi(2,K0,K1),multi(2,K2,K3)))", 4),
];

#[test]
fn v7_general_shapes_print_exact_general_recovery_line() {
    for (t, n) in GENERAL_SHAPES {
        assert_general_card(&bundle_stderr(t), t, n);
    }
}

#[test]
fn v7_non_pure_thresh_gets_general_line() {
    // Ruling 3: a thresh with a multi, timelock or hash argument, or a key
    // argument carrying a wrapper other than s:/a:, is general.
    for (t, n) in [
        ("wsh(thresh(1,multi(2,K0,K1,K2),s:pk(K3)))", 4),
        ("wsh(thresh(2,pk(K0),s:pk(K1),sln:older(1000)))", 2),
        ("wsh(thresh(2,pk(K0),s:pk(K1),a:sha256(HH)))", 2),
        ("wsh(thresh(2,pk(K0),sn:pk(K1),s:pk(K2)))", 3),
    ] {
        assert_general_card(&bundle_stderr(t), t, n);
    }
}

#[test]
fn v7_archetypes_get_general_line() {
    for (t, n) in [
        // tiered
        (
            "wsh(or_i(sortedmulti(2,K0,K1),and_v(v:older(1000),thresh(2,pk(K2),s:pk(K3)))))",
            4,
        ),
        // decaying
        (
            "wsh(andor(multi(2,K0,K1),older(1000),andor(multi(2,K2,K3),older(2000),and_v(v:pk(K4),after(900000)))))",
            5,
        ),
        // k-of-n with a recovery key
        ("wsh(or_d(multi(2,K0,K1,K2),and_v(v:pk(K3),older(1000))))", 4),
        // inheritance
        ("wsh(or_d(pk(K0),and_v(v:pkh(K1),older(1000))))", 2),
    ] {
        assert_general_card(&bundle_stderr(t), t, n);
    }
}

#[test]
fn v7_e3_timelocked_inheritance_gets_general_line() {
    // The manual's inheritance example (41-mnemonic.md, transcript
    // 41-bundle-inheritance-cards.err), which prints "3 of 3" today.
    let t = "wsh(andor(pkh(K0),after(12000000),or_i(and_v(v:pkh(K1),older(4032)),and_v(v:pkh(K2),older(32768)))))";
    assert_general_card(&bundle_stderr(t), t, 3);
}

#[test]
fn v7_bundle_json_threshold_null_for_general_policy() {
    for t in [
        "wsh(or_d(pk(K2),multi(2,K0,K1)))",
        "tr(NUMS,{pk(K0),pk(K1)})",
        "wsh(andor(pkh(K0),after(12000000),or_i(and_v(v:pkh(K1),older(4032)),and_v(v:pkh(K2),older(32768)))))",
    ] {
        assert_eq!(bundle_json_threshold(t), Value::Null, "{t}");
    }
}

#[test]
fn v7_import_wallet_text_threshold_none_for_general() {
    let t = "wsh(or_d(pk(K2),multi(2,K0,K1)))";
    let out = import_wallet_stdout(t, false);
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines.contains(&"bundles[0].threshold=none"), "{t}:\n{out}");
    assert!(lines.contains(&"threshold=none"), "{t}:\n{out}");
    assert!(
        !lines
            .iter()
            .any(|l| l.starts_with("bundles[0].threshold=") && *l != "bundles[0].threshold=none"),
        "{t}:\n{out}"
    );
    assert!(
        !lines
            .iter()
            .any(|l| l.starts_with("threshold=") && *l != "threshold=none"),
        "{t}:\n{out}"
    );
}

#[test]
fn v7_import_wallet_json_threshold_null_for_general() {
    let t = "wsh(or_d(pk(K2),multi(2,K0,K1)))";
    let v: Value = serde_json::from_str(&import_wallet_stdout(t, true)).unwrap();
    assert_eq!(
        v[0]["bundle"]["multisig"].get("threshold"),
        Some(&Value::Null),
        "{t}: {v}"
    );
}

#[test]
fn v7_null_threshold_envelope_accepted_by_bundle_import_json() {
    let dir = tempfile::tempdir().unwrap();
    let t = "wsh(or_d(pk(K2),multi(2,K0,K1)))";
    let env = envelope_file(dir.path(), "null.json", t, Value::Null);
    let out = bundle_import_json(&env, &[]);
    let card = String::from_utf8_lossy(&out.stderr).into_owned();
    assert!(
        out.status.success(),
        "{t}: rc {:?}:\n{card}",
        out.status.code()
    );
    assert_general_card(&card, t, 3);
    let out = bundle_import_json(&env, &["--json"]);
    assert!(
        out.status.success(),
        "{t} --json: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        v["multisig"].get("threshold"),
        Some(&Value::Null),
        "{t}: {v}"
    );
}

#[test]
fn v7_null_threshold_envelope_accepted_by_export_wallet_descriptor() {
    let dir = tempfile::tempdir().unwrap();
    let t = "wsh(or_d(pk(K2),multi(2,K0,K1)))";
    let null_env = envelope_file(dir.path(), "null.json", t, Value::Null);
    let two_env = envelope_file(dir.path(), "two.json", t, Value::from(2));
    let a = export_from_envelope(&null_env, "descriptor");
    assert!(
        a.status.success(),
        "{t}: null envelope refused: {}",
        String::from_utf8_lossy(&a.stderr)
    );
    let b = export_from_envelope(&two_env, "descriptor");
    assert!(
        b.status.success(),
        "{t}: {}",
        String::from_utf8_lossy(&b.stderr)
    );
    assert_eq!(
        a.stdout, b.stdout,
        "{t}: null and 2 envelopes export differently"
    );
}

#[test]
fn v7_null_threshold_envelope_template_format_still_refuses_general_policy() {
    let dir = tempfile::tempdir().unwrap();
    let t = "wsh(or_d(pk(K2),multi(2,K0,K1)))";
    let env = envelope_file(dir.path(), "null.json", t, Value::Null);
    let out = export_from_envelope(&env, "sparrow");
    assert_eq!(out.status.code(), Some(1), "{t}");
    assert!(out.stdout.is_empty(), "{t}: stdout not empty");
    assert_eq!(
        String::from_utf8_lossy(&out.stderr),
        "error: --from-import-json: --format sparrow cannot represent a general wallet \
         policy (timelocks/hashlocks/non-multisig miniscript); it is a plain k-of-n \
         multisig format. Use --format descriptor / bitcoin-core / bip388 for faithful \
         descriptor passthrough.\n",
        "{t}"
    );
}

#[test]
fn v7_stale_envelope_threshold_recomputed_by_bundle_import_json() {
    // An envelope a pre-fix import-wallet wrote (threshold 2 for a general
    // descriptor): bundle recomputes from the descriptor.
    let dir = tempfile::tempdir().unwrap();
    let t = "wsh(or_d(pk(K2),multi(2,K0,K1)))";
    let env = envelope_file(dir.path(), "stale.json", t, Value::from(2));
    let out = bundle_import_json(&env, &[]);
    assert!(
        out.status.success(),
        "{t}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_general_card(&String::from_utf8_lossy(&out.stderr), t, 3);
    let out = bundle_import_json(&env, &["--json"]);
    assert!(
        out.status.success(),
        "{t}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(
        v["multisig"].get("threshold"),
        Some(&Value::Null),
        "{t}: {v}"
    );
}

#[test]
fn v7_import_json_threshold_flag_cannot_override_card_k() {
    // FIX-V7 STOP-2, ruled (b) (architect 2026-11-04): --threshold is ignored on the --import-json path. Exit 0; the card and the JSON take K from the envelope's descriptor.
    let dir = tempfile::tempdir().unwrap();
    let plain = envelope_file(
        dir.path(),
        "plain.json",
        "wsh(multi(2,K0,K1,K2))",
        Value::from(2),
    );
    let general = envelope_file(
        dir.path(),
        "general.json",
        "wsh(or_d(pk(K2),multi(2,K0,K1)))",
        Value::from(2),
    );
    for k in ["1", "9"] {
        let out = bundle_import_json(&plain, &["--threshold", k]);
        assert!(
            out.status.success(),
            "plain --threshold {k}: stderr:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_plain_card(
            &String::from_utf8_lossy(&out.stderr),
            &format!("plain --threshold {k}"),
            2,
            3,
        );
        let out = bundle_import_json(&plain, &["--threshold", k, "--json"]);
        assert!(
            out.status.success(),
            "plain --threshold {k} --json: stderr:\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let v: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(
            v["multisig"]["threshold"],
            Value::from(2),
            "plain --threshold {k} --json"
        );
    }
    let out = bundle_import_json(&general, &["--threshold", "2"]);
    assert!(
        out.status.success(),
        "general --threshold 2: stderr:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_general_card(
        &String::from_utf8_lossy(&out.stderr),
        "general --threshold 2",
        3,
    );
}

// ---- FIX-V7 controls: must pass before and after the fix ------------------

#[test]
fn v7_pure_key_thresh_keeps_any_k_of_n() {
    // Ruling 3 (triage V7-14): a pure-key thresh is exactly k-of-n.
    for t in [
        "wsh(thresh(2,pk(K0),s:pk(K1),s:pk(K2)))",
        "tr(NUMS,thresh(2,pk(K0),s:pk(K1),s:pk(K2)))",
        "wsh(thresh(2,pkh(K0),a:pkh(K1),a:pk(K2)))",
        "sh(wsh(thresh(2,pk(K0),s:pk(K1),s:pk(K2))))",
    ] {
        assert_plain_card(&bundle_stderr(t), t, 2, 3);
    }
}

#[test]
fn v7_sh_plain_shapes_keep_threshold_and_recovery_lines() {
    for t in [
        "sh(multi(2,K0,K1,K2))",
        "sh(sortedmulti(2,K0,K1,K2))",
        "sh(wsh(multi(2,K0,K1,K2)))",
        "sh(wsh(sortedmulti(2,K0,K1,K2)))",
    ] {
        assert_plain_card(&bundle_stderr(t), t, 2, 3);
    }
}

#[test]
fn v7_single_key_prints_no_threshold_or_recovery_line() {
    for t in [
        "wsh(pk(K0))",
        "tr(K0)",
        "wsh(and_v(v:pk(K0),older(1000)))",
        "wsh(multi(1,K0))",
    ] {
        let e = bundle_stderr(t);
        assert!(
            !e.lines()
                .any(|l| l.starts_with("# Threshold:") || l.starts_with("# Recovery:")),
            "{t}: n == 1 prints neither line:\n{e}"
        );
    }
}

#[test]
fn v7_bundle_json_threshold_k_for_plain() {
    for t in [
        "wsh(sortedmulti(2,K0,K1,K2))",
        "tr(NUMS,multi_a(2,K0,K1,K2))",
        "wsh(thresh(2,pk(K0),s:pk(K1),s:pk(K2)))",
    ] {
        assert_eq!(bundle_json_threshold(t), Value::from(2), "{t}");
    }
}

#[test]
fn v7_import_wallet_threshold_k_for_plain() {
    let t = "wsh(sortedmulti(2,K0,K1,K2))";
    let out = import_wallet_stdout(t, false);
    let lines: Vec<&str> = out.lines().collect();
    assert!(lines.contains(&"bundles[0].threshold=2"), "{t}:\n{out}");
    assert!(lines.contains(&"threshold=2"), "{t}:\n{out}");
    let v: Value = serde_json::from_str(&import_wallet_stdout(t, true)).unwrap();
    assert_eq!(
        v[0]["bundle"]["multisig"]["threshold"],
        Value::from(2),
        "{t}: {v}"
    );
}

// ---- Template mode keeps `--threshold` (FIX-V7 ruling 1 / 2.1) -------------

const TPL_XPUBS: [(&str, &str); 3] = [
    ("xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX", "b8688df1"),
    ("xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx", "5436d724"),
    ("xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6", "28645006"),
];

/// `bundle --template wsh-sortedmulti --threshold 2` over three xpub slots.
fn template_bundle(extra: &[&str]) -> std::process::Output {
    let mut args: Vec<String> = [
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "wsh-sortedmulti",
        "--threshold",
        "2",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    for (i, (xpub, fp)) in TPL_XPUBS.iter().enumerate() {
        for (field, val) in [
            ("xpub", *xpub),
            ("fingerprint", *fp),
            ("path", "48'/0'/0'/2'"),
        ] {
            args.push("--slot".into());
            args.push(format!("@{i}.{field}={val}"));
        }
    }
    args.extend(extra.iter().map(|s| s.to_string()));
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

#[test]
fn v7_template_mode_card_keeps_flag_threshold_and_recovery_lines() {
    let e = String::from_utf8(template_bundle(&[]).stderr).unwrap();
    assert!(e.lines().any(|l| l == "# Threshold: 2 of 3"), "{e}");
    assert!(
        e.lines()
            .any(|l| l == "# Recovery: any 2 of 3 signing keys + md1 (template card)."),
        "{e}"
    );
    assert!(!e.contains("general policy"), "{e}");
}

#[test]
fn v7_template_mode_json_threshold_is_the_flag() {
    let out = template_bundle(&["--json"]);
    let v: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(v["multisig"]["threshold"], Value::from(2), "{v}");
}

#[test]
fn v7_export_wallet_takes_k_from_envelope_descriptor() {
    // A plain 2-of-3 envelope whose threshold is null or stale: every
    // template format emits the same file as the honest envelope, because K
    // comes from the envelope's descriptor (the rule bundle --import-json uses).
    let dir = tempfile::tempdir().unwrap();
    let t = "wsh(sortedmulti(2,K0,K1,K2))";
    let good = envelope_file(dir.path(), "good.json", t, Value::from(2));
    for (name, th) in [("null.json", Value::Null), ("stale.json", Value::from(3))] {
        let env = envelope_file(dir.path(), name, t, th);
        for format in ["sparrow", "coldcard", "electrum"] {
            let want = export_from_envelope(&good, format);
            assert!(
                want.status.success(),
                "{format}: {}",
                String::from_utf8_lossy(&want.stderr)
            );
            let got = export_from_envelope(&env, format);
            assert!(
                got.status.success(),
                "{name} {format}: {}",
                String::from_utf8_lossy(&got.stderr)
            );
            assert_eq!(
                String::from_utf8_lossy(&got.stdout),
                String::from_utf8_lossy(&want.stdout),
                "{name} {format}"
            );
        }
    }
}
