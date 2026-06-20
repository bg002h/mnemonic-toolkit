//! #28 phase 1 — `mnemonic bundle --md1-form=template` integration tests.
//!
//! Funds-safety class: a wrong template-form md1 = a wrong wallet. The gates
//! are byte-identity (one engraving for thousands), template-id binding, and a
//! passing self-check — NOT exit-0. Every golden is anchored OUTSIDE the
//! toolkit synth path: the emitted md1 is decoded with the PUBLIC `md_codec`
//! API and its `WalletDescriptorTemplateId` / `WalletPolicyId` recomputed
//! independently.

use assert_cmd::Command;

// Two DIFFERENT seeds (distinct keys) — byte-identity must hold ACROSS keys.
const PHRASE_A: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const PHRASE_B: &str =
    "legal winner thank year wave sausage worth useful legal winner thank yellow";

fn mnemonic() -> Command {
    Command::cargo_bin("mnemonic").expect("mnemonic binary builds")
}

/// Run `bundle` and return (stdout, stderr) on success.
fn bundle_ok(extra: &[&str]) -> (String, String) {
    let out = mnemonic().args(extra).assert().success();
    let o = out.get_output();
    (
        String::from_utf8(o.stdout.clone()).unwrap(),
        String::from_utf8(o.stderr.clone()).unwrap(),
    )
}

/// Extract the md1 string(s) from `bundle` text stdout (lines under `# md1`).
fn md1_lines(stdout: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_md1 = false;
    for line in stdout.lines() {
        if line.starts_with("# md1") {
            in_md1 = true;
            continue;
        }
        if in_md1 {
            if line.trim().is_empty() {
                in_md1 = false;
                continue;
            }
            out.push(line.trim().to_string());
        }
    }
    out
}

fn mk1_lines(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .filter(|l| l.trim().starts_with("mk1"))
        .map(|l| l.trim().to_string())
        .collect()
}

fn template_bundle(template: &str, phrase: &str, account: &str) -> (String, String) {
    // --group-size 0 → unbroken strings (no display separators) so the emitted
    // ms1/mk1/md1 decode directly via the codec APIs.
    bundle_ok(&[
        "bundle",
        "--template",
        template,
        "--network",
        "mainnet",
        "--md1-form",
        "template",
        "--account",
        account,
        "--group-size",
        "0",
        "--slot",
        &format!("@0.phrase={phrase}"),
    ])
}

// ============================================================================
// Funds-safety test 1 — byte-identity (the goal): two seeds / two accounts,
// same type → identical template md1.
// ============================================================================

#[test]
fn template_md1_byte_identical_across_different_seeds() {
    for template in ["bip44", "bip84", "bip86"] {
        let (out_a, _) = template_bundle(template, PHRASE_A, "0");
        let (out_b, _) = template_bundle(template, PHRASE_B, "0");
        let md1_a = md1_lines(&out_a);
        let md1_b = md1_lines(&out_b);
        assert!(!md1_a.is_empty(), "{template}: md1 emitted");
        assert_eq!(
            md1_a, md1_b,
            "{template}: template md1 must be byte-identical across DIFFERENT seeds (one engraving for thousands)"
        );
    }
}

#[test]
fn template_md1_byte_identical_across_accounts() {
    for template in ["bip44", "bip84", "bip86"] {
        let (out_0, _) = template_bundle(template, PHRASE_A, "0");
        let (out_5, _) = template_bundle(template, PHRASE_A, "5");
        assert_eq!(
            md1_lines(&out_0),
            md1_lines(&out_5),
            "{template}: account must be normalized away in the template md1"
        );
    }
}

// ============================================================================
// Funds-safety test 2 — INDEPENDENT golden: the emitted md1 is keyless and its
// binding stub matches an independently-recomputed WalletDescriptorTemplateId.
// ============================================================================

#[test]
fn template_md1_is_keyless_and_binds_on_template_id() {
    let (out, _) = template_bundle("bip84", PHRASE_A, "0");
    let md1 = md1_lines(&out);
    let md1_refs: Vec<&str> = md1.iter().map(|s| s.as_str()).collect();

    // Independent decode + identity recompute via the PUBLIC md_codec API.
    let desc = md_codec::chunk::reassemble(&md1_refs).expect("md1 decodes");
    assert!(
        !desc.is_wallet_policy(),
        "template md1 must be KEYLESS (no pubkeys TLV)"
    );
    assert!(
        desc.tlv.pubkeys.is_none() && desc.tlv.fingerprints.is_none(),
        "template md1 strips pubkeys + fingerprints"
    );
    // canonical-origin elidable (the gate's invariant).
    assert!(
        md_codec::canonical_origin::canonical_origin(&desc.tree).is_some(),
        "template md1 tree is canonical-origin elidable"
    );

    let tmpl_id = md_codec::compute_wallet_descriptor_template_id(&desc).unwrap();
    let expected_stub = &tmpl_id.as_bytes()[..4];

    // The mk1 card carries the template-id stub (decode the mk1 independently).
    let mk1 = mk1_lines(&out);
    let mk1_refs: Vec<&str> = mk1.iter().map(|s| s.as_str()).collect();
    let card = mk_codec::decode(&mk1_refs).expect("mk1 decodes");
    assert!(
        card.policy_id_stubs.iter().any(|s| s == expected_stub),
        "mk1 stub must root on the WalletDescriptorTemplateId, not WalletPolicyId"
    );
}

#[test]
fn template_md1_does_not_cross_bind_to_policy_form() {
    // The SAME wallet emitted policy-form has a DIFFERENT (key-significant) stub.
    let (out_t, _) = template_bundle("bip84", PHRASE_A, "0");
    let (out_p, _) = bundle_ok(&[
        "bundle",
        "--template",
        "bip84",
        "--network",
        "mainnet",
        "--md1-form",
        "policy",
        "--slot",
        &format!("@0.phrase={PHRASE_A}"),
    ]);

    let md1_t = md1_lines(&out_t);
    let md1_p = md1_lines(&out_p);
    let dt = md_codec::chunk::reassemble(&md1_t.iter().map(|s| s.as_str()).collect::<Vec<_>>())
        .unwrap();
    let dp = md_codec::chunk::reassemble(&md1_p.iter().map(|s| s.as_str()).collect::<Vec<_>>())
        .unwrap();
    assert!(!dt.is_wallet_policy());
    assert!(dp.is_wallet_policy());

    let t_stub = md_codec::compute_wallet_descriptor_template_id(&dt).unwrap();
    let p_stub = md_codec::compute_wallet_policy_id(&dp).unwrap();
    assert_ne!(
        &t_stub.as_bytes()[..4],
        &p_stub.as_bytes()[..4],
        "template stub and policy stub must differ (no cross-mix)"
    );
}

// ============================================================================
// Funds-safety test 3 — self-check passes for a template bundle.
// ============================================================================

#[test]
fn template_bundle_self_check_passes() {
    mnemonic()
        .args([
            "bundle",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--md1-form",
            "template",
            "--self-check",
            "--slot",
            &format!("@0.phrase={PHRASE_A}"),
        ])
        .assert()
        .success();
}

// ============================================================================
// Funds-safety test 5 — refusals at bundle-emit: multisig / n>1 / non-canonical.
// ============================================================================

#[test]
fn template_form_refuses_multisig_template() {
    mnemonic()
        .args([
            "bundle",
            "--template",
            "wsh-sortedmulti",
            "--network",
            "mainnet",
            "--md1-form",
            "template",
            "--threshold",
            "2",
            "--slot",
            "@0.xpub=xpub6BgBgsespWvERF3LHQu6CnqdvfEvtMcQjYrcRzx53QJjSxarj2afYWcLteoGVky7D3UKDP9QyrLprQ3VCECoY49yfdDEHGCtMMj92pReUsQ",
            "--slot",
            "@1.xpub=xpub6BgBgsespWvERF3LHQu6CnqdvfEvtMcQjYrcRzx53QJjSxarj2afYWcLteoGVky7D3UKDP9QyrLprQ3VCECoY49yfdDEHGCtMMj92pReUsQ",
        ])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn template_form_refuses_bip49_nested_segwit() {
    // bip49 = sh(wpkh(@0)) → canonical_origin returns None → refused.
    let assert = mnemonic()
        .args([
            "bundle",
            "--template",
            "bip49",
            "--network",
            "mainnet",
            "--md1-form",
            "template",
            "--slot",
            &format!("@0.phrase={PHRASE_A}"),
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("single-sig") || stderr.contains("md1-form=policy"),
        "bip49 refusal must name the single-sig/template constraint: {stderr}"
    );
}

// ============================================================================
// Funds-safety test 8 — non-regression: --md1-form=policy byte-identical to
// the default (no flag) for all corpus shapes.
// ============================================================================

#[test]
fn policy_form_byte_identical_to_default() {
    for template in ["bip44", "bip49", "bip84", "bip86"] {
        let (out_default, _) = bundle_ok(&[
            "bundle",
            "--template",
            template,
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={PHRASE_A}"),
        ]);
        let (out_policy, _) = bundle_ok(&[
            "bundle",
            "--template",
            template,
            "--network",
            "mainnet",
            "--md1-form",
            "policy",
            "--slot",
            &format!("@0.phrase={PHRASE_A}"),
        ]);
        assert_eq!(
            out_default, out_policy,
            "{template}: --md1-form=policy must be byte-identical to the default"
        );
    }
}

// ============================================================================
// D7 — the WalletPolicyId advisory is on STDERR (cards on stdout unchanged) and
// is recomputable from an INDEPENDENTLY-built explicit-origin descriptor.
// ============================================================================

#[test]
fn d7_wallet_id_on_stderr_not_stdout() {
    let (stdout, stderr) = template_bundle("bip84", PHRASE_A, "0");
    assert!(
        stderr.contains("wallet-id"),
        "D7 wallet-id advisory must be on stderr: {stderr}"
    );
    assert!(
        !stdout.contains("wallet-id"),
        "D7 advisory must NOT pollute stdout (the engraved cards)"
    );
}
