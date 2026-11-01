//! Bug-hunt triage RED rows, mnemonic-toolkit slice S1 (`restore`, `verify-bundle`).
//!
//! Source: `design/agent-reports/bughunt-2026-10-31-tk.md` (hunt at c0a99df).
//! One test per reported bug, named `bughunt_tk_NN_<slug>`, asserting the
//! CORRECT behaviour. A test that fails on unmodified code confirms the bug;
//! the triage report records the unmodified-run failure and the scratch fix
//! that turns it green. Every test drives the built `mnemonic` binary the way
//! the existing `tests/cli_*.rs` files do.
//!
//! #55 is the hunt report's own duplicate of #14 and has no separate test.

use assert_cmd::Command;
use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use miniscript::{Descriptor, DescriptorPublicKey};
use serde_json::Value;
use std::str::FromStr;

const PHRASE_A: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const SEED_A: &str = "legal winner thank year wave sausage worth useful legal winner thank yellow";
const SEED_B: &str =
    "letter advice cage absurd amount doctor acoustic avoid letter advice cage above";
const SEED_C: &str = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";

/// bip84 account 1 xpub (m/84'/0'/1') of PHRASE_A: right seed, wrong account
/// against an `--expect-xpub` meant for account 0.
const ACCT1_XPUB_BIP84: &str = "xpub6CatWdiZiodmYVtWLtEQsAg1H9ooS1bmsJUBwQ83FE1Fyk386FWcyicJgEZv3quZSJKA5dh5Lo2PbubMGxCfZtRthV6ST2qquL9w3HSzcUn";

fn mnemonic() -> Command {
    let mut c = Command::cargo_bin("mnemonic").expect("mnemonic binary builds");
    c.arg("--allow-argv-secret");
    c
}

/// Run to completion; return (exit code, stdout, stderr).
fn run(args: &[String]) -> (Option<i32>, String, String) {
    let out = mnemonic().args(args).output().expect("spawn mnemonic");
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

// ---------------------------------------------------------------------------
// Shared bundle/restore helpers (copied from the existing cli_* test files).
// ---------------------------------------------------------------------------

fn section_lines(stdout: &str, header: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_sec = false;
    for line in stdout.lines() {
        if line.starts_with(header) {
            in_sec = true;
            continue;
        }
        if in_sec {
            if line.trim().is_empty() {
                in_sec = false;
                continue;
            }
            out.push(line.trim().to_string());
        }
    }
    out
}

fn md1_lines(stdout: &str) -> Vec<String> {
    section_lines(stdout, "# md1")
}

fn mk1_groups(stdout: &str) -> Vec<Vec<String>> {
    let mut groups: Vec<Vec<String>> = Vec::new();
    let mut cur: Option<Vec<String>> = None;
    for line in stdout.lines() {
        if line.starts_with("# mk1") {
            if let Some(g) = cur.take() {
                if !g.is_empty() {
                    groups.push(g);
                }
            }
            cur = Some(Vec::new());
            continue;
        }
        if let Some(g) = cur.as_mut() {
            let t = line.trim();
            if t.starts_with("mk1") {
                g.push(t.to_string());
            }
        }
    }
    if let Some(g) = cur.take() {
        if !g.is_empty() {
            groups.push(g);
        }
    }
    groups
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
    let md1 = md1_lines(&stdout);
    assert!(!md1.is_empty(), "{template}: template md1 emitted");
    md1
}

fn push_md1(args: &mut Vec<String>, md1: &[String]) {
    for c in md1 {
        args.push("--md1".into());
        args.push(c.clone());
    }
}

/// Derive a mainnet account xpub + master fingerprint at `path_str` from a
/// BIP-39 phrase.
fn xpub_at(phrase: &str, path_str: &str) -> (Xpub, String) {
    let secp = Secp256k1::new();
    let m = Mnemonic::parse_in(bip39::Language::English, phrase).unwrap();
    let seed = m.to_seed("");
    let master = Xpriv::new_master(bitcoin::NetworkKind::Main, &seed).unwrap();
    let fp = master.fingerprint(&secp);
    let path = DerivationPath::from_str(path_str).unwrap();
    let xpriv = master.derive_priv(&secp, &path).unwrap();
    let xpub = Xpub::from_priv(&secp, &xpriv);
    (xpub, fp.to_string().to_lowercase())
}

/// The canonical BIP-48 origin for a given wsh/sh-wsh multisig at `account`.
fn canonical_path(script: &str, account: u32) -> String {
    match script {
        "wsh-multi" | "wsh-sortedmulti" => format!("48'/0'/{account}'/2'"),
        "sh-wsh-multi" | "sh-wsh-sortedmulti" => format!("48'/0'/{account}'/1'"),
        other => panic!("unknown script {other}"),
    }
}

/// `bundle` argv for N cosigners (each at the canonical BIP-48 origin for
/// `script`/`account`) in `form` (`template` | `policy`).
fn bundle_args(
    script: &str,
    threshold: &str,
    form: &str,
    cosigners: &[(&str, u32)],
) -> Vec<String> {
    let mut args: Vec<String> = vec![
        "bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        script.into(),
        "--threshold".into(),
        threshold.into(),
        "--md1-form".into(),
        form.into(),
        "--group-size".into(),
        "0".into(),
        "--no-engraving-card".into(),
    ];
    for (idx, (phrase, account)) in cosigners.iter().enumerate() {
        let path = canonical_path(script, *account);
        let (xpub, fp) = xpub_at(phrase, &path);
        args.push("--slot".into());
        args.push(format!("@{idx}.xpub={xpub}"));
        args.push("--slot".into());
        args.push(format!("@{idx}.fingerprint={fp}"));
        args.push("--slot".into());
        args.push(format!("@{idx}.path={path}"));
    }
    args
}

fn emit_template_md1(script: &str, threshold: &str, cosigners: &[(&str, u32)]) -> Vec<String> {
    let out = mnemonic()
        .args(bundle_args(script, threshold, "template", cosigners))
        .assert()
        .success();
    md1_lines(&String::from_utf8(out.get_output().stdout.clone()).unwrap())
}

/// The full-hex WalletPolicyId the template emit advisory prints on stderr.
fn emit_template_wallet_id(script: &str, threshold: &str, cosigners: &[(&str, u32)]) -> String {
    let out = mnemonic()
        .args(bundle_args(script, threshold, "template", cosigners))
        .assert()
        .success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    let line = stderr
        .lines()
        .find(|l| l.contains("wallet-id (hex)"))
        .unwrap_or_else(|| panic!("no wallet-id (hex) line in: {stderr}"));
    line.split(':').next_back().unwrap().trim().to_string()
}

/// One cosigner's mk1 chunk(s) from a policy-form multisig bundle.
fn emit_cosigner_mk1(
    script: &str,
    threshold: &str,
    cosigners: &[(&str, u32)],
    which: usize,
) -> Vec<String> {
    let out = mnemonic()
        .args(bundle_args(script, threshold, "policy", cosigners))
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    mk1_groups(&stdout)
        .get(which)
        .unwrap_or_else(|| panic!("no mk1 group {which} in: {stdout}"))
        .clone()
}

/// Independent golden: first `count` receive addresses of the watch-only
/// wsh/sh-wsh multisig built directly from the cosigner account xpubs.
fn golden_addresses(
    script: &str,
    threshold: u32,
    cosigners: &[(&str, u32)],
    sorted: bool,
    count: u32,
) -> Vec<String> {
    let mut key_strs: Vec<String> = Vec::new();
    for (phrase, account) in cosigners {
        let path = canonical_path(script, *account);
        let (xpub, fp) = xpub_at(phrase, &path);
        let origin = path.replace('\'', "h");
        key_strs.push(format!("[{fp}/{origin}]{xpub}/<0;1>/*"));
    }
    let inner = if sorted {
        format!("sortedmulti({threshold},{})", key_strs.join(","))
    } else {
        format!("multi({threshold},{})", key_strs.join(","))
    };
    let desc_str = match script {
        "wsh-multi" | "wsh-sortedmulti" => format!("wsh({inner})"),
        "sh-wsh-multi" | "sh-wsh-sortedmulti" => format!("sh(wsh({inner}))"),
        other => panic!("unknown script {other}"),
    };
    let desc = Descriptor::<DescriptorPublicKey>::from_str(&desc_str)
        .unwrap_or_else(|e| panic!("golden descriptor parse {desc_str}: {e}"));
    let receive = desc.clone().into_single_descriptors().unwrap().remove(0);
    (0..count)
        .map(|i| {
            receive
                .derive_at_index(i)
                .unwrap()
                .address(bitcoin::Network::Bitcoin)
                .unwrap()
                .to_string()
        })
        .collect()
}

// ===========================================================================
// #1 — restore skips --expect-xpub when --expect-fingerprint is also given.
// ===========================================================================

#[test]
fn bughunt_tk_01_restore_checks_xpub_even_when_fingerprint_matches() {
    // Right seed (fingerprint 73c5da0a matches), but the reference xpub is the
    // ACCOUNT-1 xpub while restore derives account 0: the xpub reference is
    // wrong and must be reported.
    let (code, stdout, stderr) = run(&s(&[
        "restore",
        "--from",
        &format!("phrase={PHRASE_A}"),
        "--template",
        "bip84",
        "--expect-fingerprint",
        "73c5da0a",
        "--expect-xpub",
        ACCT1_XPUB_BIP84,
    ]));
    assert_eq!(
        code,
        Some(4),
        "a matching --expect-fingerprint must not mask a wrong --expect-xpub \
         (want exit 4 RestoreMismatch);\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stderr.contains("xpub mismatch"),
        "stderr must name the xpub mismatch:\n{stderr}"
    );
}

// ===========================================================================
// #2 — restore of a keyless single-sig template md1 ignores the reference
//      flags, --allow-mismatch and --format.
// ===========================================================================

#[test]
fn bughunt_tk_02_singlesig_template_md1_honours_reference_flags() {
    let md1 = template_md1("bip84", PHRASE_A, "0");
    let base = |extra: &[&str]| -> Vec<String> {
        let mut a = s(&[
            "restore",
            "--network",
            "mainnet",
            "--from",
            &format!("phrase={PHRASE_A}"),
        ]);
        push_md1(&mut a, &md1);
        a.extend(s(extra));
        a
    };
    let mut failures: Vec<String> = Vec::new();

    // Open product question: honour the flags in template mode (the report's
    // first fix: exit 4 on a wrong reference) or refuse them (its second).
    // Wrong = exit 0 with the flag silently ignored. Either fix passes below.
    //
    // (a) wrong --expect-fingerprint must not exit 0.
    let (code, stdout, _e) = run(&base(&["--expect-fingerprint", "deadbeef"]));
    if code == Some(0) {
        failures.push(format!(
            "(a) --expect-fingerprint deadbeef: want exit 4 (or a refusal), got exit 0; stdout: {stdout}"
        ));
    }
    // (b) wrong --expect-xpub must not exit 0.
    let (code, stdout, _e) = run(&base(&["--expect-xpub", ACCT1_XPUB_BIP84]));
    if code == Some(0) {
        failures.push(format!(
            "(b) --expect-xpub <account-1 xpub>: want exit 4 (or a refusal), got exit 0; stdout: {stdout}"
        ));
    }
    // (c) --allow-mismatch with a wrong reference: honoured = exit 0 AND a
    // loud MISMATCH banner; refused = non-zero. Wrong = exit 0 with no banner.
    let (code, _o, stderr) = run(&base(&[
        "--expect-fingerprint",
        "deadbeef",
        "--allow-mismatch",
    ]));
    if code == Some(0) && !stderr.contains("MISMATCH") {
        failures.push(format!(
            "(c) --allow-mismatch with wrong fingerprint: exit 0 and no MISMATCH banner; stderr: {stderr}"
        ));
    }
    // (d) --format must not be silently ignored (honour, or refuse). Open
    // product question: honour vs refuse. Wrong = exit 0 with the plain text doc.
    let (code, stdout, _e) = run(&base(&["--format", "bitcoin-core"]));
    let honoured = code == Some(0) && serde_json::from_str::<Value>(stdout.trim()).is_ok();
    let refused = code != Some(0);
    if !(honoured || refused) {
        failures.push(format!(
            "(d) --format bitcoin-core silently ignored: exit {code:?}, stdout not an import payload: {stdout}"
        ));
    }
    assert!(
        failures.is_empty(),
        "single-sig template md1 restore ignores flags:\n{}",
        failures.join("\n")
    );
}

// ===========================================================================
// #10 — restore with a keyless multisig template ignores --output FILE.
// ===========================================================================

#[test]
fn bughunt_tk_10_multisig_template_restore_honours_output_file() {
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-sortedmulti", "2", cos);
    let id = emit_template_wallet_id("wsh-sortedmulti", "2", cos);
    let mk1_b = emit_cosigner_mk1("wsh-sortedmulti", "2", cos, 1);
    let golden = golden_addresses("wsh-sortedmulti", 2, cos, true, 1);
    let dir = tempfile::tempdir().unwrap();
    let outfile = dir.path().join("restored.txt");

    let mut args = s(&["restore", "--network", "mainnet"]);
    push_md1(&mut args, &md1);
    args.extend(s(&[
        "--from",
        &format!("phrase={SEED_A}"),
        "--account",
        "0",
        "--expect-wallet-id",
        &id,
        "--output",
        outfile.to_str().unwrap(),
    ]));
    for c in &mk1_b {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    let (code, stdout, stderr) = run(&args);
    assert_eq!(code, Some(0), "restore must succeed; stderr: {stderr}");
    assert!(
        outfile.exists(),
        "--output {} was not created (the document went to stdout instead):\n{stdout}",
        outfile.display()
    );
    let written = std::fs::read_to_string(&outfile).unwrap_or_default();
    assert!(
        written.contains(&golden[0]),
        "--output file must hold the restored document (first recv {}); file:\n{written}",
        golden[0]
    );
    assert!(
        !stdout.contains(&golden[0]),
        "with --output FILE the document must not also go to stdout:\n{stdout}"
    );
}

// ===========================================================================
// Taproot multi_a helpers (#38, #53).
// ===========================================================================

const TK0: &str = "[73c5da0a/87'/0'/0']xpub6DBjiYnc4ewKti13Q1L35bqdodw5z3VGJnf516B3icHrEGEUcCuCG5GVQDZtH8Xmsyt3Fs9YDNwLaqjUbbRidwXZ6sxufZcr4VqqzrXvicM";
const TK1: &str = "[b8688df1/87'/0'/0']xpub6CbhrPzY2z7NcCGCGjLAJLq8iRyjUfwmdXQs66MxTVUReKqb9DpLnVJ5D1qpatZjUuPGTyxf5TYU1vA34YFE9FHB4TvfYmokYLVsyEFZFt9";
const TK2: &str = "[28645006/87'/0'/0']xpub6DB7HNqw6CZojxN85NuFTPWZhi2FagSnexPS1rv3nYQhngkmdHgb7iebYvTFmFKKDA3ozf5yezDsCH6cXAw3WZijviSZtZC2hjHn2uazz4z";
/// BIP-341 NUMS H-point, x-only hex (the reconstructed internal key).
const NUMS_HEX: &str = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

/// `bundle --descriptor <desc> --network mainnet --json` -> md1 chunks.
fn bundle_md1(desc: &str) -> Vec<String> {
    let out = mnemonic()
        .args([
            "bundle",
            "--descriptor",
            desc,
            "--network",
            "mainnet",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: Value = serde_json::from_slice(&out).expect("bundle --json output");
    v["md1"]
        .as_array()
        .expect("md1 array")
        .iter()
        .map(|c| c.as_str().expect("md1 chunk str").to_string())
        .collect()
}

fn restore_md1_args(md1: &[String]) -> Vec<String> {
    let mut a = s(&["restore", "--network", "mainnet"]);
    push_md1(&mut a, md1);
    a
}

/// First receive address (index 0 of the first branch) of a concrete descriptor.
fn first_address_of(desc_str: &str) -> String {
    let desc = Descriptor::<DescriptorPublicKey>::from_str(desc_str)
        .unwrap_or_else(|e| panic!("golden descriptor parse {desc_str}: {e}"));
    let receive = desc.into_single_descriptors().unwrap().remove(0);
    receive
        .derive_at_index(0)
        .unwrap()
        .address(bitcoin::Network::Bitcoin)
        .unwrap()
        .to_string()
}

/// `tr(<internal_key>, <tag>(k, <leaf_indices>))` wallet-policy `Descriptor`
/// built DIRECTLY via md-codec's tree types (copied from
/// `cli_restore_taproot.rs::build_tr_leaf_descriptor`). `bundle` refuses to mint
/// shapes with repeated keys, so these cards come from the codec.
fn build_tr_leaf_descriptor(
    internal_key: md_codec::tree::InternalKey,
    n: u8,
    k: u8,
    leaf_indices: Vec<u8>,
    tag: md_codec::Tag,
) -> md_codec::Descriptor {
    use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
    use md_codec::tree::{Body, Node};
    use md_codec::use_site_path::UseSitePath;
    use md_codec::{Descriptor as MdDescriptor, Tag, TlvSection};

    let tree = Node {
        tag: Tag::Tr,
        body: Body::Tr {
            internal_key,
            tree: Some(Box::new(Node {
                tag,
                body: Body::MultiKeys {
                    k,
                    indices: leaf_indices,
                },
            })),
        },
    };
    let path = OriginPath {
        components: [48u32, 0, 0, 2]
            .iter()
            .map(|&value| PathComponent {
                hardened: true,
                value,
            })
            .collect(),
    };
    let fingerprints: Vec<(u8, [u8; 4])> = (0..n).map(|i| (i, [i, 0xBB, 0xCC, 0xDD])).collect();
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let pubkeys: Vec<(u8, [u8; 65])> = (0..n)
        .map(|i| {
            let mut sk_bytes = [0u8; 32];
            sk_bytes[31] = i + 1;
            let sk = bitcoin::secp256k1::SecretKey::from_slice(&sk_bytes).unwrap();
            let pk = bitcoin::secp256k1::PublicKey::from_secret_key(&secp, &sk);
            let mut b = [0u8; 65];
            b[0..32].copy_from_slice(&[0x42; 32]);
            b[32..].copy_from_slice(&pk.serialize());
            (i, b)
        })
        .collect();
    MdDescriptor {
        n,
        path_decl: PathDecl {
            n,
            paths: PathDeclPaths::Shared(path),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree,
        tlv: TlvSection {
            use_site_path_overrides: None,
            fingerprints: Some(fingerprints),
            pubkeys: Some(pubkeys),
            origin_path_overrides: None,
            unknown: Vec::new(),
        },
    }
}

// ===========================================================================
// #38 — restore of a taproot multi_a template ignores a non-standard
//       baseline use-site path (hard-codes /<0;1>/*).
// ===========================================================================

#[test]
fn bughunt_tk_38_taproot_multi_a_nonstandard_usesite_not_silently_replaced() {
    let mut failures: Vec<String> = Vec::new();
    // Control first: the standard use-site must restore and match the golden,
    // so a "refuse every tr(NUMS,multi_a) card" fix cannot pass.
    // Then the two non-standard baselines: restore must either print the
    // faithful wallet (golden address) or refuse. Open product question:
    // faithful reconstruction vs refusal. Wrong = exit 0 with another wallet.
    for (us, must_succeed) in [("/<0;1>/*", true), ("/<2;3>/*", false), ("/*", false)] {
        let desc = format!("tr(NUMS,multi_a(2,{TK0}{us},{TK1}{us},{TK2}{us}))");
        let md1 = bundle_md1(&desc);
        let golden = first_address_of(&format!(
            "tr({NUMS_HEX},multi_a(2,{TK0}{us},{TK1}{us},{TK2}{us}))"
        ));
        let (code, stdout, stderr) = run(&restore_md1_args(&md1));
        match code {
            Some(0) => {
                if !stdout.contains(&golden) {
                    failures.push(format!(
                        "use-site {us}: restore exit 0 but first recv is NOT the card's wallet \
                         (want {golden}); stdout:\n{stdout}"
                    ));
                }
            }
            other => {
                if must_succeed {
                    failures.push(format!(
                        "use-site {us}: control must restore, got {other:?}; stderr: {stderr}"
                    ));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "taproot multi_a restore silently replaces the card's use-site path:\n{}",
        failures.join("\n")
    );
}

// ===========================================================================
// #53 — taproot multi_a with a repeated key index restores a different leaf.
// ===========================================================================

#[test]
fn bughunt_tk_53_taproot_multi_a_repeated_key_index_is_refused() {
    // tr(NUMS, multi_a(2, @0, @1, @1)) -- built via md-codec (bundle refuses it).
    let d = build_tr_leaf_descriptor(
        md_codec::tree::InternalKey::NumsPoint,
        2,
        2,
        vec![0, 1, 1],
        md_codec::Tag::MultiA,
    );
    let chunks = md_codec::chunk::split(&d).expect("split repeated-index tr md1");
    let (code, stdout, stderr) = run(&restore_md1_args(&chunks));
    assert_ne!(
        code,
        Some(0),
        "a card whose leaf repeats @1 cannot be restored faithfully (multi_a with a repeated \
         key is not a valid script); restore must refuse, not print a different leaf.\n\
         md1 chunks: {chunks:?}\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        !stdout.contains("descriptor:"),
        "no descriptor may be printed for it:\n{stdout}"
    );
}

// ===========================================================================
// #40 — restore multisig template: own account ignored (account 0 used) when
//       the first cosigner is a bare xpub.
// ===========================================================================

#[test]
fn bughunt_tk_40_template_restore_bare_xpub_cosigner_keeps_own_account() {
    // 2-of-2 BIP-48 sortedmulti: own seed A at ACCOUNT 1, cosigner B at account 0
    // supplied as a BARE xpub (no origin). The completion must derive A at
    // m/48'/0'/1'/2' (as --account 1 says) and reproduce the card's wallet.
    let cos = &[(SEED_A, 1u32), (SEED_B, 0u32)];
    let md1 = emit_template_md1("wsh-sortedmulti", "2", cos);
    let golden = golden_addresses("wsh-sortedmulti", 2, cos, true, 2);
    let (xpub_b, _fp) = xpub_at(SEED_B, &canonical_path("wsh-sortedmulti", 0));

    let mut args = s(&["restore", "--network", "mainnet"]);
    push_md1(&mut args, &md1);
    args.extend(s(&[
        "--from",
        &format!("phrase={SEED_A}"),
        "--account",
        "1",
        // A bare xpub carries no origin, so id-search cannot match; the address
        // is the collision-free target (and origin-independent).
        "--search-address",
        &golden[0],
        "--cosigner",
        &xpub_b.to_string(),
        "--count",
        "2",
        "--json",
    ]));
    let (code, stdout, stderr) = run(&args);
    assert_eq!(
        code,
        Some(0),
        "own seed at --account 1 must be derived at account 1 (m/48'/0'/1'/2'), not account 0;\n\
         stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let j: Value = serde_json::from_str(&stdout).expect("restore --json");
    let got: Vec<String> = j["wallets"][0]["first_addresses"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a.as_str().unwrap().to_string())
        .collect();
    assert_eq!(
        got, golden,
        "completed addresses must equal the card's wallet"
    );
}

// ===========================================================================
// #41 — restore with @N= placement silently ignores --search-address.
// ===========================================================================

#[test]
fn bughunt_tk_41_explicit_placement_does_not_ignore_search_address() {
    // Mirrors cli_restore_md1_template_multisig.rs::explicit_assignment_honours_expect_wallet_id
    // with the address instead of the id. UNSORTED wsh-multi: a swap changes the wallet.
    let cos = &[(SEED_A, 0u32), (SEED_B, 0u32), (SEED_C, 0u32)];
    let md1 = emit_template_md1("wsh-multi", "2", cos);
    let true_addr = golden_addresses("wsh-multi", 2, cos, false, 1).remove(0);
    let mk1_b = emit_cosigner_mk1("wsh-multi", "2", cos, 1);
    let mk1_c = emit_cosigner_mk1("wsh-multi", "2", cos, 2);

    let mut args = s(&["restore", "--network", "mainnet"]);
    push_md1(&mut args, &md1);
    args.extend(s(&[
        "--from",
        &format!("phrase={SEED_A}"),
        "--account",
        "0",
        "--search-address",
        &true_addr,
        "--count",
        "1",
    ]));
    // Truth is @1=B, @2=C (own A fills @0). Assert them SWAPPED.
    for c in &mk1_c {
        args.push("--cosigner".into());
        args.push(format!("@1={c}"));
    }
    for c in &mk1_b {
        args.push("--cosigner".into());
        args.push(format!("@2={c}"));
    }
    let (code, stdout, stderr) = run(&args);
    // Open product question: check the address (exit 4) vs refuse the
    // combination (exit 1/2). Wrong = exit 0 and a wallet for another placement.
    assert_ne!(
        code,
        Some(0),
        "a wrong @N= placement plus the wallet's TRUE address must not complete at exit 0;\n\
         stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        !stdout.contains("descriptor:"),
        "no completed wallet may be printed:\n{stdout}"
    );
}

// ===========================================================================
// #42 — restore labels the user's own slot "verified" without comparing the
//       engraved fingerprint.
// ===========================================================================

#[test]
fn bughunt_tk_42_own_slot_with_wrong_engraved_fingerprint_is_not_verified() {
    // 2-of-2 wallet-policy card whose @0 carries seed A's REAL account xpub at
    // m/87'/0'/0' but a WRONG master fingerprint (deadbeef).
    let (xpub_a, real_fp) = xpub_at(SEED_A, "87'/0'/0'");
    let (xpub_b, fp_b) = xpub_at(SEED_B, "87'/0'/0'");
    assert_ne!(real_fp, "deadbeef");
    let desc = format!(
        "wsh(sortedmulti(2,[deadbeef/87'/0'/0']{xpub_a}/<0;1>/*,[{fp_b}/87'/0'/0']{xpub_b}/<0;1>/*))"
    );
    let md1 = bundle_md1(&desc);
    let mut args = restore_md1_args(&md1);
    args.extend(s(&["--from", &format!("phrase={SEED_A}")]));
    let (code, stdout, stderr) = run(&args);
    let all = format!("{stdout}\n{stderr}");
    assert!(
        code != Some(0) || !all.contains("your seed (verified)"),
        "the own slot's engraved fingerprint deadbeef != the seed's real master fingerprint \
         {real_fp}, yet it is labelled verified (exit {code:?}):\n{all}"
    );
}

// ===========================================================================
// verify-bundle helpers.
// ===========================================================================

/// `bundle --json` for a FULL-mode (seed-bearing) multisig over `phrases`.
fn full_multisig_bundle(script: &str, k: &str, phrases: &[&str]) -> Value {
    let mut args = s(&[
        "bundle",
        "--network",
        "mainnet",
        "--template",
        script,
        "--threshold",
        k,
        "--multisig-path-family",
        "bip48",
        "--json",
    ]);
    for (i, p) in phrases.iter().enumerate() {
        args.push("--slot".into());
        args.push(format!("@{i}.phrase={p}"));
    }
    let (code, stdout, stderr) = run(&args);
    assert_eq!(code, Some(0), "bundle failed: {stderr}");
    serde_json::from_str(&stdout).expect("bundle --json")
}

/// Base `verify-bundle` argv for a FULL-mode multisig over `phrases` (no cards).
fn full_verify_base(script: &str, k: &str, phrases: &[&str]) -> Vec<String> {
    let mut args = s(&[
        "--no-auto-repair",
        "verify-bundle",
        "--network",
        "mainnet",
        "--template",
        script,
        "--threshold",
        k,
        "--multisig-path-family",
        "bip48",
        "--json",
    ]);
    for (i, p) in phrases.iter().enumerate() {
        args.push("--slot".into());
        args.push(format!("@{i}.phrase={p}"));
    }
    args
}

fn strs(v: &Value) -> Vec<String> {
    v.as_array()
        .expect("array")
        .iter()
        .map(|x| x.as_str().expect("string").to_string())
        .collect()
}

fn mk1_groups_of(v: &Value) -> Vec<Vec<String>> {
    v["mk1"]
        .as_array()
        .expect("mk1 groups")
        .iter()
        .map(strs)
        .collect()
}

/// Append card flags: one `--ms1` per entry (including `""` watch-only
/// sentinels), `--mk1` per chunk, `--md1` per chunk.
fn push_cards(args: &mut Vec<String>, ms1: &[String], mk1: &[Vec<String>], md1: &[String]) {
    for m in ms1 {
        args.push("--ms1".into());
        args.push(m.clone());
    }
    for g in mk1 {
        for c in g {
            args.push("--mk1".into());
            args.push(c.clone());
        }
    }
    push_md1(args, md1);
}

/// Look up a verify-bundle `--json` check by name.
fn check<'a>(j: &'a Value, name: &str) -> Option<&'a Value> {
    j["checks"]
        .as_array()?
        .iter()
        .find(|c| c["name"].as_str() == Some(name))
}

fn check_passed(j: &Value, name: &str) -> Option<bool> {
    check(j, name).and_then(|c| c["passed"].as_bool())
}

/// A deterministic, valid BIP-39 phrase for stray-key fixtures.
fn stray_phrase(k: u8) -> String {
    Mnemonic::from_entropy(&[k; 16]).unwrap().to_string()
}

// ===========================================================================
// #12 — verify_bundle: one stray mk1 card blames the wrong cosigner slots.
// ===========================================================================

#[test]
fn bughunt_tk_12_stray_mk1_does_not_blame_correct_cosigner_slots() {
    let script = "wsh-sortedmulti";
    let good = full_multisig_bundle(script, "2", &[SEED_A, SEED_B, SEED_C]);
    let ms1 = strs(&good["ms1"]);
    let md1 = strs(&good["md1"]);
    let groups = mk1_groups_of(&good);
    assert_eq!(groups.len(), 3);

    // Supplied mk1: A's card (slot 0), a STRAY card from an unrelated key (in
    // place of B's), C's card (slot 2). Slots 0 and 2 are correct and must be
    // reported so; only slot 1 is wrong. The groups are processed in
    // chunk_set_id order, so whether the stray is processed before A's card is
    // fixed by the (deterministic) hashes: try several strays.
    let mut failures: Vec<String> = Vec::new();
    for k in 1u8..=8 {
        let stray = mk1_groups_of(&full_multisig_bundle(
            script,
            "2",
            &[&stray_phrase(k), SEED_B, SEED_C],
        ))[0]
            .clone();
        let mut args = full_verify_base(script, "2", &[SEED_A, SEED_B, SEED_C]);
        push_cards(
            &mut args,
            &ms1,
            &[groups[0].clone(), stray, groups[2].clone()],
            &md1,
        );
        let (_code, stdout, stderr) = run(&args);
        let j: Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
            panic!("verify --json parse: {e}\nstdout:{stdout}\nstderr:{stderr}")
        });
        let p0 = check_passed(&j, "mk1_decode[0]");
        let p2 = check_passed(&j, "mk1_decode[2]");
        if p0 != Some(true) || p2 != Some(true) {
            failures.push(format!(
                "stray #{k}: mk1_decode[0]={p0:?} mk1_decode[2]={p2:?} (both cards are correct); \
                 slot-0 detail: {:?}",
                check(&j, "mk1_decode[0]").map(|c| c["detail"].clone())
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "a stray mk1 card made correct cosigner cards fail:\n{}",
        failures.join("\n")
    );
}

// ===========================================================================
// #13 — verify_bundle positional fallback orders cards by chunk_set_id.
// ===========================================================================

#[test]
fn bughunt_tk_13_positional_fallback_keeps_supplied_card_order() {
    let script = "wsh-sortedmulti";
    let mut failures: Vec<String> = Vec::new();
    // Several wallets: the chunk_set_id order of 3 cards equals the supplied
    // order for only 1 wallet in 6, so the bug cannot hide across 6 variants.
    for k in 1u8..=6 {
        let phrases_owned = [stray_phrase(k), SEED_B.to_string(), SEED_C.to_string()];
        let phrases: Vec<&str> = phrases_owned.iter().map(|x| x.as_str()).collect();
        let good = full_multisig_bundle(script, "2", &phrases);
        let ms1 = strs(&good["ms1"]);
        let groups = mk1_groups_of(&good); // already in slot order: the order the user supplies
                                           // An md1 that does not decode forces the positional fallback. (Drop the
                                           // last chunk: an incomplete set.)
        let mut md1 = strs(&good["md1"]);
        assert!(md1.len() > 1, "need a multi-chunk md1");
        md1.pop();
        let mut args = full_verify_base(script, "2", &phrases);
        push_cards(&mut args, &ms1, &groups, &md1);
        let (_code, stdout, stderr) = run(&args);
        let j: Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
            panic!("verify --json parse: {e}\nstdout:{stdout}\nstderr:{stderr}")
        });
        for i in 0..3 {
            for kind in ["mk1_xpub_match", "mk1_fingerprint_match", "mk1_path_match"] {
                let name = format!("{kind}[{i}]");
                if check_passed(&j, &name) != Some(true) {
                    failures.push(format!(
                        "variant {k}: {name} = {:?} although cards were supplied in slot order",
                        check_passed(&j, &name)
                    ));
                }
            }
        }
    }
    assert!(
        failures.is_empty(),
        "positional fallback re-ordered the supplied mk1 cards:\n{}",
        failures.join("\n")
    );
}

// ===========================================================================
// #14 — verify_bundle multisig --json lacks md1_origin_match when the md1
//       does not decode.
// ===========================================================================

#[test]
fn bughunt_tk_14_multisig_json_has_md1_origin_row_when_md1_does_not_decode() {
    let script = "wsh-sortedmulti";
    let good = full_multisig_bundle(script, "2", &[SEED_A, SEED_B, SEED_C]);
    let ms1 = strs(&good["ms1"]);
    let groups = mk1_groups_of(&good);
    let md1_good = strs(&good["md1"]);
    let mut md1_bad = md1_good.clone();
    md1_bad.pop(); // incomplete md1 set: does not decode

    let names = |md1: &[String]| -> Vec<String> {
        let mut args = full_verify_base(script, "2", &[SEED_A, SEED_B, SEED_C]);
        push_cards(&mut args, &ms1, &groups, md1);
        let (_c, stdout, stderr) = run(&args);
        let j: Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
            panic!("verify --json parse: {e}\nstdout:{stdout}\nstderr:{stderr}")
        });
        j["checks"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c["name"].as_str().unwrap().to_string())
            .collect()
    };
    let ok_names = names(&md1_good);
    assert!(
        ok_names.iter().any(|n| n == "md1_origin_match"),
        "control: the decodable path must carry md1_origin_match: {ok_names:?}"
    );
    let bad_names = names(&md1_bad);
    assert!(
        bad_names.iter().any(|n| n == "md1_decode"),
        "fixture must make the md1 undecodable: {bad_names:?}"
    );
    assert!(
        bad_names.iter().any(|n| n == "md1_origin_match"),
        "the --json check list is a FIXED schema; md1_origin_match is missing when the md1 \
         does not decode: {bad_names:?}"
    );
    assert_eq!(
        ok_names.len(),
        bad_names.len(),
        "the check list must have the same length on every branch:\nok:  {ok_names:?}\nbad: {bad_names:?}"
    );
}

// ===========================================================================
// #43 — verify-bundle panics on a supplied md1 with more cosigners than
//       expected.
// ===========================================================================

#[test]
fn bughunt_tk_43_supplied_md1_with_extra_cosigner_does_not_panic() {
    let script = "wsh-sortedmulti";
    // Supplied: a 3-cosigner wallet (cards for A, B, C). Expected: the 2-cosigner
    // wallet from phrases A, B only.
    let three = full_multisig_bundle(script, "2", &[SEED_A, SEED_B, SEED_C]);
    let ms1 = strs(&three["ms1"]);
    let md1 = strs(&three["md1"]);
    let groups = mk1_groups_of(&three);
    let mut args = full_verify_base(script, "2", &[SEED_A, SEED_B]);
    push_cards(&mut args, &ms1, &groups, &md1);
    let (code, stdout, stderr) = run(&args);
    assert!(
        !stderr.contains("panicked") && code != Some(101),
        "verify-bundle panicked on a supplied 3-cosigner md1 against a 2-cosigner expectation \
         (exit {code:?}):\n{stderr}"
    );
    assert_ne!(
        code,
        Some(0),
        "a 3-cosigner card set cannot verify a 2-cosigner wallet:\n{stdout}"
    );
}

// ===========================================================================
// Single-sig template verify-bundle helpers (#37, #39, #47).
// ===========================================================================

/// `bundle --template <t> --slot @0.phrase=<p> --json` -> (ms1, mk1 chunks, md1 chunks).
fn singlesig_cards(template: &str, phrase: &str) -> (Vec<String>, Vec<String>, Vec<String>) {
    let (code, stdout, stderr) = run(&s(&[
        "bundle",
        "--template",
        template,
        "--network",
        "mainnet",
        "--slot",
        &format!("@0.phrase={phrase}"),
        "--json",
    ]));
    assert_eq!(code, Some(0), "bundle failed: {stderr}");
    let v: Value = serde_json::from_str(&stdout).unwrap();
    (strs(&v["ms1"]), strs(&v["mk1"]), strs(&v["md1"]))
}

/// Single-sig TEMPLATE-form cards (keyless md1) for (template, phrase).
fn singlesig_template_cards(
    template: &str,
    phrase: &str,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let (code, stdout, stderr) = run(&s(&[
        "bundle",
        "--template",
        template,
        "--network",
        "mainnet",
        "--md1-form",
        "template",
        "--group-size",
        "0",
        "--slot",
        &format!("@0.phrase={phrase}"),
    ]));
    assert_eq!(code, Some(0), "bundle failed: {stderr}");
    (
        section_lines(&stdout, "# ms1"),
        section_lines(&stdout, "# mk1"),
        section_lines(&stdout, "# md1"),
    )
}

// ===========================================================================
// #37 — verify-bundle on a single-sig template never checks the --ms1 card.
// ===========================================================================

#[test]
fn bughunt_tk_37_singlesig_template_verify_checks_the_ms1_card() {
    let (ms1_good, mk1, md1) = singlesig_template_cards("bip84", PHRASE_A);
    let (ms1_other, _mk1_o, _md1_o) = singlesig_template_cards("bip84", SEED_A);
    assert_ne!(ms1_good, ms1_other, "fixture: two different seeds");
    let verify = |ms1: &[String]| -> (Option<i32>, String, String) {
        let mut args = s(&[
            "verify-bundle",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={PHRASE_A}"),
        ]);
        for m in ms1.iter().filter(|m| !m.is_empty()) {
            args.push("--ms1".into());
            args.push(m.clone());
        }
        for m in &mk1 {
            args.push("--mk1".into());
            args.push(m.clone());
        }
        push_md1(&mut args, &md1);
        run(&args)
    };
    let (code, stdout, stderr) = verify(&ms1_good);
    assert_eq!(
        code,
        Some(0),
        "control: the right ms1 verifies: {stdout}{stderr}"
    );
    let (code, stdout, stderr) = verify(&ms1_other);
    assert_eq!(
        code,
        Some(4),
        "an --ms1 card from a DIFFERENT seed must fail verification (exit 4);\n\
         stdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

// ===========================================================================
// #39 — verify-bundle single-sig md1_xpub_match compares only the first
//       pubkey (not the tree, not the use-site path).
// ===========================================================================

#[test]
fn bughunt_tk_39_singlesig_md1_with_other_usesite_or_script_does_not_pass() {
    let (ms1, mk1, md1_good) = singlesig_cards("bip84", PHRASE_A);
    let verify = |md1: &[String]| -> (Option<i32>, Value) {
        let mut args = s(&[
            "verify-bundle",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={PHRASE_A}"),
            "--json",
        ]);
        for m in &ms1 {
            args.push("--ms1".into());
            args.push(m.clone());
        }
        for m in &mk1 {
            args.push("--mk1".into());
            args.push(m.clone());
        }
        push_md1(&mut args, md1);
        let (code, stdout, stderr) = run(&args);
        let j: Value = serde_json::from_str(&stdout).unwrap_or_else(|e| {
            panic!("verify --json parse: {e}\nstdout:{stdout}\nstderr:{stderr}")
        });
        (code, j)
    };
    let (code, j) = verify(&md1_good);
    assert_eq!(code, Some(0), "control: the genuine md1 verifies: {j}");

    // The seed's real bip84 account xpub (m/84'/0'/0'), same origin, same key.
    let (acct, fp) = xpub_at(PHRASE_A, "84'/0'/0'");
    let mut failures: Vec<String> = Vec::new();
    for (label, desc) in [
        (
            "use-site <2;3>",
            format!("wpkh([{fp}/84'/0'/0']{acct}/<2;3>/*)"),
        ),
        (
            "script type pkh",
            format!("pkh([{fp}/84'/0'/0']{acct}/<0;1>/*)"),
        ),
    ] {
        let md1_alt = bundle_md1(&desc);
        let (code, j) = verify(&md1_alt);
        if code == Some(0) || j["result"] == "ok" {
            failures.push(format!(
                "{label}: supplied md1 encodes a DIFFERENT wallet but verify-bundle says result={} (exit {code:?})",
                j["result"]
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

// ===========================================================================
// #44 — verify-bundle in descriptor mode ignores the [fp/path]@N annotation
//       fingerprint.
// ===========================================================================

#[test]
fn bughunt_tk_44_descriptor_mode_verify_uses_annotation_fingerprint() {
    let (xpub_b, fp_b) = xpub_at(SEED_B, "48'/0'/0'/2'");
    let (_xa, fp_a) = xpub_at(SEED_A, "48'/0'/0'/2'");
    // @1 is a bare xpub slot with NO `.fingerprint=` overlay: its fingerprint
    // comes from the descriptor annotation, exactly as `bundle` takes it.
    let descriptor = format!(
        "wsh(sortedmulti(2,@0[{fp_a}/48'/0'/0'/2']/<0;1>/*,@1[{fp_b}/48'/0'/0'/2']/<0;1>/*))"
    );
    let slots = [format!("@0.phrase={SEED_A}"), format!("@1.xpub={xpub_b}")];
    let mut bundle_cmd = s(&[
        "bundle",
        "--descriptor",
        &descriptor,
        "--network",
        "mainnet",
        "--json",
    ]);
    for sl in &slots {
        bundle_cmd.push("--slot".into());
        bundle_cmd.push(sl.clone());
    }
    let (code, stdout, stderr) = run(&bundle_cmd);
    assert_eq!(code, Some(0), "bundle must accept the descriptor: {stderr}");
    let v: Value = serde_json::from_str(&stdout).unwrap();

    let mut verify_cmd = s(&[
        "--no-auto-repair",
        "verify-bundle",
        "--descriptor",
        &descriptor,
        "--network",
        "mainnet",
        "--json",
    ]);
    for sl in &slots {
        verify_cmd.push("--slot".into());
        verify_cmd.push(sl.clone());
    }
    push_cards(
        &mut verify_cmd,
        &strs(&v["ms1"]),
        &mk1_groups_of(&v),
        &strs(&v["md1"]),
    );
    let (code, stdout, stderr) = run(&verify_cmd);
    let failed: Vec<String> = serde_json::from_str::<Value>(&stdout)
        .ok()
        .and_then(|j| {
            j["checks"].as_array().map(|cs| {
                cs.iter()
                    .filter(|c| c["passed"] == false)
                    .map(|c| format!("{} ({})", c["name"], c["detail"]))
                    .collect()
            })
        })
        .unwrap_or_default();
    assert_eq!(
        code,
        Some(0),
        "verifying the bundle with the SAME descriptor and slots must pass; failed checks: {failed:?}\n\
         stdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

// ===========================================================================
// #47 — an empty --expect-wallet-id matches any wallet.
// ===========================================================================

#[test]
fn bughunt_tk_47_empty_expect_wallet_id_is_rejected_by_verify_bundle() {
    let (ms1, mk1, md1) = singlesig_template_cards("bip84", PHRASE_A);
    let mut args = s(&[
        "verify-bundle",
        "--network",
        "mainnet",
        "--slot",
        &format!("@0.phrase={PHRASE_A}"),
        "--expect-wallet-id",
        "",
    ]);
    for m in ms1.iter().filter(|m| !m.is_empty()) {
        args.push("--ms1".into());
        args.push(m.clone());
    }
    for m in &mk1 {
        args.push("--mk1".into());
        args.push(m.clone());
    }
    push_md1(&mut args, &md1);
    let (code, stdout, stderr) = run(&args);
    assert!(
        code != Some(0),
        "an empty --expect-wallet-id (e.g. an unset shell variable) must be rejected, \
         not reported as a passing wallet_id_match; exit {code:?}\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    // The restore path already refuses an empty prefix (decode_wallet_id_prefix):
    // pin that as the reference behaviour.
    let mut rargs = s(&[
        "restore",
        "--network",
        "mainnet",
        "--from",
        &format!("phrase={PHRASE_A}"),
        "--expect-wallet-id",
        "",
    ]);
    push_md1(&mut rargs, &md1);
    let (rcode, _o, rerr) = run(&rargs);
    assert_ne!(rcode, Some(0), "restore refuses an empty prefix: {rerr}");
}
