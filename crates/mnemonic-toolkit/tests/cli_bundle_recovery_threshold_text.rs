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
//! The replacement wording is a product decision (open question in the triage
//! report), so these rows assert only that the false claim is gone, not what
//! replaces it. The four plain shapes are GREEN controls.

use assert_cmd::Command;

const K0: &str = "[b8688df1/48'/0'/0'/2']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*";
const K1: &str = "[5436d724/48'/0'/0'/2']xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx/<0;1>/*";
const K2: &str = "[28645006/48'/0'/0'/2']xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6/<0;1>/*";
const K3: &str = "[16a93ed0/48'/0'/0'/2']xpub6Bv8ayijom26yJ1wZ62h4X1smfYBfNeNtGujxw6vaY4zq4Tw4cn2oV8qZmjnuVxh56oSe21r7V8r9LjZjArFh3QRZQbgzgLcfjVikZNa86W/<0;1>/*";
const NUMS: &str = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

/// Substitute `K0..K3` and `NUMS` in a template and run `bundle --descriptor`;
/// return stderr (where the card text goes). Asserts exit 0.
fn bundle_stderr(template: &str) -> String {
    let mut d = template.to_string();
    for (name, val) in [("K0", K0), ("K1", K1), ("K2", K2), ("K3", K3), ("NUMS", NUMS)] {
        d = d.replace(name, val);
    }
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
        assert!(e.contains("# Threshold: 2 of 3"), "{t}: missing Threshold line:\n{e}");
        assert!(
            e.contains("# Recovery: any 2 of 3 signing keys + md1"),
            "{t}: missing Recovery line:\n{e}"
        );
    }
}
