//! #28 phase 1 — `mnemonic restore --md1 <keyless-template>` single-sig
//! template-completion integration tests.
//!
//! Funds-safety class. The make-or-break gates:
//!  - completion ADDRESS-EQUIVALENCE: template + seed + account → the SAME
//!    descriptor + addresses as the INDEPENDENT full-policy single-sig restore
//!    (`restore --from <seed> --template …`, which does NOT touch the new code
//!    path) — the independent golden.
//!  - the C2 funds-safety hole: a no-`--from` template restore is REJECTED.
//!  - keyless MULTISIG template at ingest → refusal (carve-out fall-through).
//!  - `--expect-wallet-id`: correct→pass, wrong→loud refuse, short→advisory.

use assert_cmd::Command;

const PHRASE_A: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fn mnemonic() -> Command {
    Command::cargo_bin("mnemonic").expect("mnemonic binary builds")
}

/// Emit a keyless single-sig template md1 (unbroken) for (template, phrase).
fn template_md1(template: &str, phrase: &str, account: &str) -> Vec<String> {
    let out = mnemonic()
        .args([
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
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    // md1 line(s) under `# md1`.
    let mut md1 = Vec::new();
    let mut in_md1 = false;
    for line in stdout.lines() {
        if line.starts_with("# md1") {
            in_md1 = true;
            continue;
        }
        if in_md1 {
            if line.trim().is_empty() {
                break;
            }
            md1.push(line.trim().to_string());
        }
    }
    assert!(!md1.is_empty(), "{template}: template md1 emitted");
    md1
}

/// Extract the `descriptor:` line from a restore TEXT document.
fn descriptor_line(stdout: &str) -> String {
    stdout
        .lines()
        .find_map(|l| l.trim().strip_prefix("descriptor: "))
        .expect("descriptor line present")
        .to_string()
}

/// The INDEPENDENT golden: the full-policy single-sig restore (NOT the template
/// completion path) for (template, phrase, account).
fn golden_restore(template: &str, phrase: &str, account: &str) -> String {
    let out = mnemonic()
        .args([
            "restore",
            "--from",
            &format!("phrase={phrase}"),
            "--template",
            template,
            "--network",
            "mainnet",
            "--account",
            account,
        ])
        .assert()
        .success();
    String::from_utf8(out.get_output().stdout.clone()).unwrap()
}

// ============================================================================
// Funds-safety test 3 (the make-or-break) — completion address-equivalence
// against an INDEPENDENT golden.
// ============================================================================

#[test]
fn template_completion_equals_independent_full_restore() {
    for template in ["bip44", "bip84", "bip86"] {
        for account in ["0", "5"] {
            let md1 = template_md1(template, PHRASE_A, account);
            // Restore via template-completion.
            let mut args = vec![
                "restore".to_string(),
                "--from".to_string(),
                format!("phrase={PHRASE_A}"),
                "--network".to_string(),
                "mainnet".to_string(),
                "--account".to_string(),
                account.to_string(),
            ];
            for chunk in &md1 {
                args.push("--md1".to_string());
                args.push(chunk.clone());
            }
            let out = mnemonic().args(&args).assert().success();
            let completed = String::from_utf8(out.get_output().stdout.clone()).unwrap();
            let completed_desc = descriptor_line(&completed);

            // Golden: the independent full-policy restore.
            let golden = golden_restore(template, PHRASE_A, account);
            let golden_desc = descriptor_line(&golden);

            assert_eq!(
                completed_desc, golden_desc,
                "{template} acct {account}: template completion descriptor must equal the independent full restore"
            );
            // Addresses too (the funds-safety oracle).
            let completed_recv: Vec<&str> = completed
                .lines()
                .filter_map(|l| l.trim().strip_prefix("first recv: "))
                .collect();
            let golden_recv: Vec<&str> = golden
                .lines()
                .filter_map(|l| l.trim().strip_prefix("first recv: "))
                .collect();
            assert_eq!(
                completed_recv, golden_recv,
                "{template} acct {account}: first-receive addresses must match the golden"
            );
        }
    }
}

// ============================================================================
// Funds-safety test (C2) — a no-`--from` template restore is REJECTED.
// ============================================================================

#[test]
fn template_restore_without_from_is_refused() {
    let md1 = template_md1("bip84", PHRASE_A, "0");
    let mut args = vec![
        "restore".to_string(),
        "--network".to_string(),
        "mainnet".to_string(),
    ];
    for chunk in &md1 {
        args.push("--md1".to_string());
        args.push(chunk.clone());
    }
    let assert = mnemonic().args(&args).assert().failure().code(2);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("--from"),
        "no-seed template restore must name --from as required: {stderr}"
    );
}

// ============================================================================
// Funds-safety test 5 (restore end) — a keyless MULTISIG template at ingest
// falls through to run_multisig's keyless refusal.
// ============================================================================

#[test]
fn keyless_multisig_md1_refused_at_restore() {
    // A keyless wsh-sortedmulti template md1 (built directly via md_codec so the
    // test does not depend on bundle emitting one — bundle refuses multisig
    // template form, which is the OTHER guard). Here we exercise the restore
    // carve-out fall-through: keyless + n>1 → run_multisig → ModeViolation.
    use md_codec::origin_path::{OriginPath, PathDecl, PathDeclPaths};
    use md_codec::tag::Tag;
    use md_codec::tree::{Body, Node};
    use md_codec::use_site_path::UseSitePath;
    use md_codec::{Descriptor, TlvSection};

    let tree = Node {
        tag: Tag::Wsh,
        body: Body::Children(vec![Node {
            tag: Tag::SortedMulti,
            body: Body::MultiKeys {
                k: 2,
                indices: vec![0, 1, 2],
            },
        }]),
    };
    let desc = Descriptor {
        n: 3,
        path_decl: PathDecl {
            n: 3,
            paths: PathDeclPaths::Shared(OriginPath { components: vec![] }),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree,
        tlv: TlvSection {
            use_site_path_overrides: None,
            fingerprints: None,
            pubkeys: None,
            origin_path_overrides: None,
            unknown: Vec::new(),
        },
    };
    let md1 = md_codec::chunk::split(&desc).expect("keyless multisig md1 encodes");
    let mut args = vec![
        "restore".to_string(),
        "--network".to_string(),
        "mainnet".to_string(),
    ];
    for chunk in &md1 {
        args.push("--md1".to_string());
        args.push(chunk.clone());
    }
    mnemonic().args(&args).assert().failure().code(2);
}

// ============================================================================
// D7 round-trip + --expect-wallet-id.
// ============================================================================

/// Recover the printed wallet-id prefix from the bundle stderr advisory.
fn bundle_wallet_id_prefix(template: &str, phrase: &str, account: &str) -> String {
    let out = mnemonic()
        .args([
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
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    stderr
        .lines()
        .find_map(|l| l.trim().strip_prefix("wallet-id (prefix): "))
        .map(|s| s.split_whitespace().next().unwrap().to_string())
        .expect("D7 prefix printed")
}

fn restore_with_expect(
    template: &str,
    phrase: &str,
    account: &str,
    expect: &str,
) -> assert_cmd::assert::Assert {
    let md1 = template_md1(template, phrase, account);
    let mut args = vec![
        "restore".to_string(),
        "--from".to_string(),
        format!("phrase={phrase}"),
        "--network".to_string(),
        "mainnet".to_string(),
        "--account".to_string(),
        account.to_string(),
        "--expect-wallet-id".to_string(),
        expect.to_string(),
    ];
    for chunk in &md1 {
        args.push("--md1".to_string());
        args.push(chunk.clone());
    }
    mnemonic().args(&args).assert()
}

#[test]
fn d7_round_trip_expect_wallet_id_correct_passes() {
    let prefix = bundle_wallet_id_prefix("bip84", PHRASE_A, "3");
    restore_with_expect("bip84", PHRASE_A, "3", &prefix).success();
}

#[test]
fn expect_wallet_id_wrong_refuses_loudly() {
    let assert = restore_with_expect("bip84", PHRASE_A, "0", "deadbeefdeadbeef")
        .failure()
        .code(4);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("MISMATCH"),
        "wrong wallet-id must refuse loudly: {stderr}"
    );
}

#[test]
fn expect_wallet_id_short_prefix_advises() {
    // 1-byte prefix matching the real id → passes but advises.
    let full = bundle_wallet_id_prefix("bip84", PHRASE_A, "0");
    let one_byte = &full[..2]; // first byte hex
    let assert = restore_with_expect("bip84", PHRASE_A, "0", one_byte).success();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("advisory") && stderr.contains("byte"),
        "short prefix must emit the ≥4-byte advisory: {stderr}"
    );
}
