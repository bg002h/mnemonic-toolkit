//! F-642 review I-1: a KEYLESS template md1 carrying a wire-kind-1 (Liana
//! unspendable) internal key is REFUSED by the shared template-completion
//! engine (`complete_multisig_template`), which both `restore` and
//! `verify-bundle` run. It is refused with the SAME exit 2 and operator wording
//! as the keyed route (`classify_taproot_restore`), from ONE rule.
//!
//! Before the fix (branch `0a3c629f`) the engine had no Liana gate. Every
//! candidate render hit md-codec's network-less `NetworkRequiredForUnspendable`,
//! which address-search swallows as "not this assignment". So `--search-address`
//! with the TRUE Liana address and the CORRECT cosigner cards reported a false
//! "✗ NO MATCH" (exit 4), in restore and verify-bundle alike. The explicit
//! `@N=` and `--expect-wallet-id` modes exited 1 with md-codec's internal text.
//!
//! Each Liana case has a NUMS control: the same wallet with the BIP-341 NUMS
//! internal key, the same keys and cards, completing at exit 0. That proves the
//! fixtures are right, so a Liana failure cannot be a fixture error.
//!
//! FIXTURES (derive-once-then-pin, minted 2026-09-23 with md-cli 0.19.0 at
//! descriptor-mnemonic `cf35d61a` and this toolkit). The keys are md-codec's
//! vendored Liana fixture `preset-kofn-recovery-tr` @0..@2 (73c5da0a /
//! 3f635a63 / 66d455ea, all at m/48'/0'/0'/3'); @0 is `abandon x11 about`.
//! - `TPL_*`: `md encode --force-chunked "tr(<IK>,{multi_a(2,@0/48'/0'/0'/3'/<0;1>/*,
//!   @1/48'/0'/0'/3'/<0;1>/*),pk(@2/48'/0'/0'/3'/<0;1>/*)})"`, with IK =
//!   `UNSPENDABLE(liana)` or the NUMS hex. `TPL_NUMS` is byte-identical to
//!   `mnemonic bundle --md1-form template`'s md1 for the same wallet.
//! - `ADDR_*` / `WPID_*`: `md address --network mainnet` / `md inspect`
//!   (`wallet-policy-id`) of the KEYED encode (the same template plus
//!   `--key @i=<xpub> --fingerprint @i=<fp>`).
//! - `COSIGNER_1` / `COSIGNER_2`: `.mk1[1]` / `.mk1[2]` of `mnemonic bundle
//!   --network mainnet --descriptor <the keyed NUMS descriptor> --json`.
//! - `STUBS`: `.mk1` (all slots) of the same with `--md1-form template`.

#![allow(missing_docs)]

use std::process::Command as StdCommand;

const PHRASE: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const ORIGIN: &str = "m/48'/0'/0'/3'";

const TPL_LIANA: &str = "md13s9pvqqzqjtvyyyhqqxq79yqs39gq6vdk9zapley66";
const TPL_NUMS: &str = "md1frvw5qqpqjtvyyyhqqxqu2gppz2sq2wk5j2qhr0e8w";
const ADDR_LIANA: &str = "bc1pv7xd8qs3lfvvrarulyehpzleuyj0qsnrsm00xsftdv2pt2fl03xq5yy4k3";
const ADDR_NUMS: &str = "bc1p4wtccfsr47cslecug6tqu934hdt6rfrr70mmjzmld075pje86lasggy3x7";
const WPID_LIANA: &str = "468e0a1e8d1c8ce01f2eaa984eae9bdc";
const WPID_NUMS: &str = "9817986e66962e355e2d5e3fae5f449f";

const COSIGNER_1: [&str; 3] = [
    "mk1qpnqtczqqspes9ucd6vp0xrwnqtesm3lvddx8lsykzqgpqqgszqgpqqgszqgpqqgswqgpqqgqjyty8sqqqqqp4lqndxkyp6purepwq5rhtyx",
    "mk1qpnqtczp9ggrcvyqq3yzg54z6jkmhnk3p90x3k4segdgmdmuudds9cpv5jpagvs2adwksdmh86xacvut7xuevj0vaw666zx2gcwjcjsxq4ly",
    "mk1qpnqtczzjgzp56thkxu8jj5rnqkjc7c2re266v3",
];
const COSIGNER_2: [&str; 3] = [
    "mk1qpnqtmzqqspes9ucd6vp0xrwnqtesmnx63274lsykzqgpqqgszqgpqqgszqgpqqgswqgpqqgqjyty8sqqqqqqg5qf9x6qlnudh3k96jqymx3",
    "mk1qpnqtmzp5m68s648lvhmc65v2x8vgr0nr56c096r0trlvknfsl3qywvlenufc32nkmcug98y43sjevctgx3q0qyatn4sxg6ky8etcfm5pk5v",
    "mk1qpnqtmzzs076v4jr23dx5va4gcwzz2glc7h5n4n",
];
const STUBS: [&str; 9] = [
    "mk1qp8t5fzqqspn46y76uaw38kh8t5fa4mnchdq4lsykzqgpqqgszqgpqqgszqgpqqgswqgpqqgqjyty8sqqqqqqfakzhkc5wfh77ph0lclvfp0",
    "mk1qp8t5fzp7r09awfl80jfm09a65602n9z3re9d0nzhss9ls7rx8uq89332vmmdv5lrhjcesgm3gx9f8lnrphzqtlt9x79y9jyr9l6ftp6xjg3",
    "mk1qp8t5fzzvyy3he8envsk2wp4uc0lq809mafulcc",
    "mk1qp8t5gzqqspn46y76uaw38kh8t5fa4elvddx8lsykzqgpqqgszqgpqqgszqgpqqgswqgpqqgqjyty8sqqqqqp4lqndxkymzwjq44muk08qw3",
    "mk1qp8t5gzp9ggrcvyqq3yzg54z6jkmhnk3p90x3k4segdgmdmuudds9cpv5jpagvs2adwksdmh86xacvut7xuevj0vaw666d8pq3ex9q8ecc52",
    "mk1qp8t5gzzjgzp56thkxuwpfgytuzvdfgvwqd34gh",
    "mk1qp8t5tzqqspn46y76uaw38kh8t5fa4mx63274lsykzqgpqqgszqgpqqgszqgpqqgswqgpqqgqjyty8sqqqqqqg5qf9x6q9tnr5azsxsv5svx",
    "mk1qp8t5tzp5m68s648lvhmc65v2x8vgr0nr56c096r0trlvknfsl3qywvlenufc32nkmcug98y43sjevctgx3q0qyatn4sx8mavwwl9mvtemlz",
    "mk1qp8t5tzzs076v4jr23ddf5mjrqk5s33k6c0l2c6",
];

fn run(args: &[String]) -> (String, String, i32) {
    let out = StdCommand::new(assert_cmd::cargo::cargo_bin("mnemonic"))
        .args(args)
        .output()
        .expect("invoke mnemonic");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().expect("mnemonic exited normally"),
    )
}

/// `<sub> --network mainnet --md1 <tpl> --from phrase=… --origin …`
fn base(sub: &str, tpl: &str) -> Vec<String> {
    let mut a: Vec<String> = vec![
        sub.into(),
        "--network".into(),
        "mainnet".into(),
        "--md1".into(),
        tpl.into(),
        "--from".into(),
        format!("phrase={PHRASE}"),
        "--allow-argv-secret".into(),
        "--origin".into(),
        ORIGIN.into(),
    ];
    if sub == "verify-bundle" {
        for s in STUBS {
            a.push("--mk1".into());
            a.push(s.into());
        }
    }
    a
}

/// Unassigned cosigner cards (the search places them).
fn cosigners_unassigned(a: &mut Vec<String>) {
    for c in COSIGNER_1.iter().chain(COSIGNER_2.iter()) {
        a.push("--cosigner".into());
        a.push((*c).into());
    }
}

/// Explicitly assigned cosigner cards (`@1=`, `@2=`).
fn cosigners_assigned(a: &mut Vec<String>) {
    for c in COSIGNER_1 {
        a.push("--cosigner".into());
        a.push(format!("@1={c}"));
    }
    for c in COSIGNER_2 {
        a.push("--cosigner".into());
        a.push(format!("@2={c}"));
    }
}

/// The operator-facing refusal: exit 2, the keyed route's wording, and
/// never the false search verdict.
fn assert_liana_refusal(out: &str, err: &str, code: i32) {
    assert_eq!(
        code, 2,
        "a refusal (exit 2), not NO MATCH (4) or exit 1: {err}"
    );
    assert!(
        err.contains("Liana unspendable internal key"),
        "the keyed route's wording: {err}"
    );
    assert!(err.contains("md descriptor --network"), "the recipe: {err}");
    assert!(!err.contains("NO MATCH"), "never the false verdict: {err}");
    assert!(
        !err.contains("_with_network entry point"),
        "never md-codec's internal text: {err}"
    );
    assert!(!out.contains("descriptor:"), "nothing rendered: {out}");
}

// ---- restore, three modes ---------------------------------------------------

#[test]
fn restore_liana_template_search_address_refuses_not_no_match() {
    let mut a = base("restore", TPL_LIANA);
    cosigners_unassigned(&mut a);
    a.push("--search-address".into());
    a.push(ADDR_LIANA.into());
    let (out, err, code) = run(&a);
    assert_liana_refusal(&out, &err, code);
}

#[test]
fn restore_liana_template_explicit_cosigners_refuses() {
    let mut a = base("restore", TPL_LIANA);
    cosigners_assigned(&mut a);
    let (out, err, code) = run(&a);
    assert_liana_refusal(&out, &err, code);
}

#[test]
fn restore_liana_template_expect_wallet_id_refuses() {
    let mut a = base("restore", TPL_LIANA);
    cosigners_unassigned(&mut a);
    a.push("--expect-wallet-id".into());
    a.push(WPID_LIANA.into());
    let (out, err, code) = run(&a);
    assert_liana_refusal(&out, &err, code);
}

#[test]
fn restore_nums_control_search_address_completes() {
    let mut a = base("restore", TPL_NUMS);
    cosigners_unassigned(&mut a);
    a.push("--search-address".into());
    a.push(ADDR_NUMS.into());
    let (out, err, code) = run(&a);
    assert_eq!(code, 0, "{err}");
    assert!(
        err.contains(&format!("wallet-id (completed): {WPID_NUMS}")),
        "{err}"
    );
    assert!(out.contains(ADDR_NUMS), "{out}");
}

#[test]
fn restore_nums_control_explicit_cosigners_completes() {
    let mut a = base("restore", TPL_NUMS);
    cosigners_assigned(&mut a);
    let (out, err, code) = run(&a);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains(ADDR_NUMS), "{out}");
}

#[test]
fn restore_nums_control_expect_wallet_id_completes() {
    let mut a = base("restore", TPL_NUMS);
    cosigners_unassigned(&mut a);
    a.push("--expect-wallet-id".into());
    a.push(WPID_NUMS.into());
    let (out, err, code) = run(&a);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains(ADDR_NUMS), "{out}");
}

// ---- verify-bundle (the same engine) ----------------------------------------

#[test]
fn verify_bundle_liana_template_refuses_not_mismatch() {
    let mut a = base("verify-bundle", TPL_LIANA);
    cosigners_unassigned(&mut a);
    a.push("--search-address".into());
    a.push(ADDR_LIANA.into());
    let (out, err, code) = run(&a);
    assert_liana_refusal(&out, &err, code);
}

#[test]
fn verify_bundle_nums_control_verifies() {
    let mut a = base("verify-bundle", TPL_NUMS);
    cosigners_unassigned(&mut a);
    a.push("--search-address".into());
    a.push(ADDR_NUMS.into());
    let (out, err, code) = run(&a);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("OK (multisig template recomposed)"), "{out}");
    assert!(out.contains(WPID_NUMS), "{out}");
}
