//! Bug hunt 2026-11-25 (toolkit findings). One test group per finding; each
//! names its finding ID. These drive the real binary.
//!
//! - toolkit#1: `bundle --descriptor` on a canonical shape with a bare `@N`
//!   seed slot derived at the MASTER node and minted an empty origin.
//! - toolkit#2: `restore --md1` invented the template default origin for a key
//!   whose card origin is empty (and, with the seed, never searched for it).
//! - toolkit#4: template mode silently dropped `@N.path=` / `@N.fingerprint=`
//!   beside a seed.
//! - toolkit#5: an xpub slot got the template default origin even when its
//!   own header contradicts it.
//! - toolkit#6: `--account >= 2^31` panicked.
//! - toolkit#7: the legacy `sh(sortedmulti)` default (superseded: BIP-87 + warning).
//! - X4: a WIF slot minted a depth-0, zero-chain-code "xpub" under a ranged
//!   use-site.

use assert_cmd::Command;
use serde_json::Value;

const A: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const B: &str = "legal winner thank year wave sausage worth useful legal winner thank yellow";
const FP_A: &str = "73c5da0a";
const FP_B: &str = "b8688df1";
/// BIP-84 test vector: first receive address of `A` at m/84'/0'/0'/0/0.
const BIP84_A_FIRST: &str = "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu";
/// The address the old (master-key) wallet of `A` produced.
const MASTER_A_FIRST: &str = "bc1qrh98qvlnec9k9au5auntfj3y2tmmw9w0353lqd";
const MAINNET_WIF: &str = "KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn";

struct Out {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run(args: &[&str]) -> Out {
    let o = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("--allow-argv-secret")
        .args(args)
        .output()
        .unwrap();
    Out {
        code: o.status.code().unwrap_or(-1),
        stdout: String::from_utf8(o.stdout).unwrap(),
        stderr: String::from_utf8(o.stderr).unwrap(),
    }
}

fn no_panic(o: &Out) {
    assert!(
        !o.stderr.contains("panicked"),
        "must not panic; stderr: {}",
        o.stderr
    );
}

fn xpub_at(phrase: &str, path: &str, network: &str) -> String {
    let o = run(&[
        "convert",
        "--network",
        network,
        "--from",
        &format!("phrase={phrase}"),
        "--to",
        "xpub",
        "--path",
        path,
    ]);
    assert_eq!(o.code, 0, "convert: {}", o.stderr);
    o.stdout
        .split_whitespace()
        .find(|w| (w.starts_with("xpub") || w.starts_with("tpub")) && w.len() > 100)
        .expect("an xpub in convert output")
        .to_string()
}

fn md1_of(json: &str) -> Vec<String> {
    let v: Value = serde_json::from_str(json).expect("bundle --json");
    match &v["md1"] {
        Value::Array(a) => a.iter().map(|s| s.as_str().unwrap().to_string()).collect(),
        Value::String(s) => vec![s.clone()],
        other => panic!("md1 field: {other:?}"),
    }
}

/// Per-`@N` origin paths stored on the card, as `84'/0'/0'`-style strings.
fn origins_on_card(md1: &[String]) -> Vec<String> {
    let refs: Vec<&str> = md1.iter().map(|s| s.as_str()).collect();
    let d = md_codec::chunk::reassemble(&refs).expect("md1 decodes");
    let expanded = md_codec::canonicalize::expand_per_at_n(&d).expect("expand");
    expanded
        .iter()
        .map(|e| {
            e.origin_path
                .components
                .iter()
                .map(|c| format!("{}{}", c.value, if c.hardened { "'" } else { "" }))
                .collect::<Vec<_>>()
                .join("/")
        })
        .collect()
}

/// Re-split `md1` with every origin EMPTIED: a keyed card as the old minting
/// produced it (the case `restore` must now handle without guessing).
fn with_empty_origins(md1: &[String]) -> Vec<String> {
    use md_codec::origin_path::{OriginPath, PathDeclPaths};
    let refs: Vec<&str> = md1.iter().map(|s| s.as_str()).collect();
    let mut d = md_codec::chunk::reassemble(&refs).expect("md1 decodes");
    d.path_decl.paths = PathDeclPaths::Shared(OriginPath {
        components: Vec::new(),
    });
    md_codec::chunk::split(&d).expect("re-split")
}

fn md1_args(md1: &[String]) -> Vec<String> {
    md1.iter()
        .flat_map(|s| ["--md1".to_string(), s.clone()])
        .collect()
}

fn restore(md1: &[String], extra: &[&str]) -> Out {
    let mut args: Vec<String> = vec!["restore".into()];
    args.extend(md1_args(md1));
    args.extend(extra.iter().map(|s| s.to_string()));
    let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    run(&refs)
}

// ---------------------------------------------------------------------------
// toolkit#1 — descriptor mode, canonical shape, bare @N.
// ---------------------------------------------------------------------------

#[test]
fn tk1_wpkh_seed_slot_derives_at_canonical_origin_not_master() {
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        "wpkh(@0/<0;1>/*)",
        "--slot",
        &format!("@0.phrase={A}"),
        "--json",
    ]);
    // Brian's ruling of 2026-11-26: an origin the toolkit chose exits 4.
    assert_eq!(o.code, 4, "{}", o.stderr);
    assert!(
        o.stderr
            .contains("warning: no origin was given for @0; the toolkit chose m/84'/0'/0' (this descriptor shape's canonical origin)"),
        "warning naming the path; stderr: {}",
        o.stderr
    );
    let md1 = md1_of(&o.stdout);
    assert_eq!(
        origins_on_card(&md1),
        vec!["84'/0'/0'"],
        "origin written explicitly"
    );
    // The wallet is the BIP-84 wallet, not the master-key one.
    let r = restore(&md1, &[]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    assert!(r.stdout.contains(BIP84_A_FIRST), "{}", r.stdout);
    assert!(!r.stdout.contains(MASTER_A_FIRST));
    assert!(r.stdout.contains("[73c5da0a/84'/0'/0']"), "{}", r.stdout);
}

#[test]
fn tk1_multisig_seed_slots_get_bip48_canonical_origin() {
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        "wsh(sortedmulti(2,@0/**,@1/**))",
        "--slot",
        &format!("@0.phrase={A}"),
        "--slot",
        &format!("@1.phrase={B}"),
        "--json",
    ]);
    assert_eq!(o.code, 4, "{}", o.stderr);
    assert!(
        o.stderr.contains(
            "@0,@1; the toolkit chose m/48'/0'/0'/2' (this descriptor shape's canonical origin)"
        ),
        "{}",
        o.stderr
    );
    let md1 = md1_of(&o.stdout);
    assert_eq!(origins_on_card(&md1), vec!["48'/0'/0'/2'", "48'/0'/0'/2'"]);
    // The keys are the seeds' BIP-48 keys, not their master keys.
    let r = restore(&md1, &[]);
    assert_eq!(r.code, 0, "{}", r.stderr);
    // The card carries chain code + key only (no parent fingerprint), so
    // compare the key material, not the serialised string.
    use bitcoin::bip32::Xpub;
    use std::str::FromStr;
    let want = Xpub::from_str(&xpub_at(A, "m/48'/0'/0'/2'", "mainnet")).unwrap();
    let tail = r
        .stdout
        .split("[73c5da0a/48'/0'/0'/2']")
        .nth(1)
        .expect("@0 key in descriptor");
    let got = Xpub::from_str(&tail[..tail.find('/').unwrap()]).unwrap();
    assert_eq!(
        (got.chain_code, got.public_key, got.depth),
        (want.chain_code, want.public_key, want.depth),
        "{}",
        r.stdout
    );
    assert!(
        !r.stdout.contains("xpub661MyMwAqRbc"),
        "no master xpub: {}",
        r.stdout
    );
}

#[test]
fn tk1_testnet_canonical_origin_uses_coin_1() {
    let o = run(&[
        "bundle",
        "--network",
        "testnet",
        "--descriptor",
        "wpkh(@0/<0;1>/*)",
        "--slot",
        &format!("@0.phrase={A}"),
        "--json",
    ]);
    assert_eq!(o.code, 4, "{}", o.stderr);
    assert_eq!(origins_on_card(&md1_of(&o.stdout)), vec!["84'/1'/0'"]);
}

#[test]
fn tk1_xpub_slot_uses_canonical_origin_only_when_header_agrees() {
    let acct = xpub_at(A, "m/84'/0'/0'", "mainnet");
    let ok = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        "wpkh(@0/<0;1>/*)",
        "--slot",
        &format!("@0.xpub={acct}"),
        "--slot",
        &format!("@0.fingerprint={FP_A}"),
        "--json",
    ]);
    assert_eq!(ok.code, 4, "{}", ok.stderr);
    assert_eq!(origins_on_card(&md1_of(&ok.stdout)), vec!["84'/0'/0'"]);

    // A BIP-48 (depth 4) xpub under the canonical wpkh origin: refused.
    let x48 = xpub_at(A, "m/48'/0'/0'/2'", "mainnet");
    let bad = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        "wpkh(@0/<0;1>/*)",
        "--slot",
        &format!("@0.xpub={x48}"),
        "--json",
    ]);
    assert_eq!(bad.code, 2, "{}", bad.stderr);
    assert!(bad.stderr.contains("@0.path="), "{}", bad.stderr);
    assert!(bad.stdout.is_empty());
}

#[test]
fn tk1_explicit_master_path_on_canonical_shape_is_refused() {
    // An empty origin on a canonical card means "the canonical origin", so the
    // master node cannot be written there: refused, nothing minted.
    let master = xpub_at(A, "m", "mainnet");
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        "wpkh(@0/<0;1>/*)",
        "--slot",
        &format!("@0.xpub={master}"),
        "--slot",
        &format!("@0.fingerprint={FP_A}"),
        "--slot",
        "@0.path=m",
        "--json",
    ]);
    assert_eq!(o.code, 2, "{}", o.stderr);
    assert!(o.stderr.contains("cannot be recorded"), "{}", o.stderr);
    assert!(o.stdout.is_empty());
}

/// Fable review I1: template mode refused `@N.path=m` nowhere, so a keyed
/// card was minted with an EMPTY origin (the descriptor-mode twin above is
/// refused). Refused at mint for single-sig and multisig templates.
#[test]
fn tk1_explicit_master_path_in_template_mode_is_refused() {
    let master = xpub_at(A, "m", "mainnet");
    let b87 = xpub_at(B, "m/87'/0'/0'", "mainnet");
    let s0x = format!("@0.xpub={master}");
    let s0f = format!("@0.fingerprint={FP_A}");
    let s1x = format!("@1.xpub={b87}");
    let mut cases: Vec<Vec<&str>> = vec![vec!["--template", "bip84"]];
    cases.push(vec![
        "--template",
        "wsh-sortedmulti",
        "--threshold",
        "1",
        "--slot",
        &s1x,
        "--slot",
        "@1.fingerprint=b8688df1",
    ]);
    for extra in cases {
        let mut args = vec![
            "bundle",
            "--network",
            "mainnet",
            "--slot",
            &s0x,
            "--slot",
            &s0f,
            "--slot",
            "@0.path=m",
            "--json",
        ];
        args.extend(extra.iter().copied());
        let o = run(&args);
        assert_eq!(o.code, 2, "{extra:?}: {}", o.stderr);
        assert!(o.stdout.is_empty(), "{extra:?}: no card: {}", o.stdout);
        assert!(
            o.stderr
                .contains("slot @0: `--slot @0.path=m` (the master node) cannot be recorded"),
            "{extra:?}: {}",
            o.stderr
        );
    }
}

// ---------------------------------------------------------------------------
// toolkit#2 — restore of a keyed card whose origin is EMPTY.
// ---------------------------------------------------------------------------

/// A keyed 2-of-2 whose keys live at m/48'/0'/0'/2', with the origins emptied.
fn empty_origin_multisig() -> Vec<String> {
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        "wsh(sortedmulti(2,@0/**,@1/**))",
        "--slot",
        &format!("@0.phrase={A}"),
        "--slot",
        &format!("@1.phrase={B}"),
        "--json",
    ]);
    assert_eq!(o.code, 4, "{}", o.stderr);
    with_empty_origins(&md1_of(&o.stdout))
}

#[test]
fn tk2_restore_without_seed_never_guesses_a_path() {
    // Brian's ruling of 2026-11-26 (tk-bh1125-r1-m2-no-seed-depth0-render):
    // no seed and no stated path refuses; nothing is rendered or guessed.
    let md1 = empty_origin_multisig();
    let r = restore(&md1, &[]);
    assert_eq!(r.code, 2, "{}", r.stderr);
    assert!(r.stdout.is_empty(), "{}", r.stdout);
    assert!(
        !r.stderr.contains("87'"),
        "no template default: {}",
        r.stderr
    );
    assert!(
        r.stderr
            .contains("@0, @1 carries NO origin path on the card"),
        "{}",
        r.stderr
    );
}

#[test]
fn tk2_restore_with_seed_finds_the_path_and_warns() {
    let md1 = empty_origin_multisig();
    // The foreign @1 has no seed here, so its path is stated (2026-11-26
    // ruling); that assertion makes the result unverified (exit 4).
    let r = restore(
        &md1,
        &[
            "--from",
            &format!("phrase={A}"),
            "--origin",
            "@1=m/48'/0'/0'/2'",
        ],
    );
    assert_eq!(r.code, 4, "{}", r.stderr);
    assert!(
        r.stderr
            .contains("@0 carries NO origin path on the card. Its key was FOUND")
            && r.stderr.contains("m/48'/0'/0'/2'"),
        "{}",
        r.stderr
    );
    assert!(
        r.stdout.contains(&format!("[{FP_A}/48'/0'/0'/2']")),
        "{}",
        r.stdout
    );
    // The foreign cosigner is never guessed: it carries the stated path,
    // marked as asserted.
    assert!(
        r.stdout.contains(&format!("[{FP_B}/48'/0'/0'/2']")),
        "{}",
        r.stdout
    );
    assert!(
        r.stderr.contains("is asserted by you, not on the card"),
        "{}",
        r.stderr
    );
}

#[test]
fn tk2_restore_with_unrelated_seed_refuses() {
    let md1 = empty_origin_multisig();
    let other = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";
    let r = restore(&md1, &["--from", &format!("phrase={other}")]);
    assert_ne!(r.code, 0, "{}", r.stdout);
    assert!(
        r.stderr
            .contains("carries NO origin path on the card, and no common path"),
        "{}",
        r.stderr
    );
    assert!(r.stdout.is_empty(), "{}", r.stdout);
}

#[test]
fn tk2_master_key_card_is_found_at_m_with_master_warning() {
    // A card holding A's MASTER key with an empty origin: what the old
    // descriptor-mode mint produced for a bare `@0` + `@0.phrase=`. Built by
    // swapping @0's key bytes on a freshly minted card.
    use bitcoin::bip32::Xpub;
    use std::str::FromStr;
    let md1 = empty_origin_multisig();
    let refs: Vec<&str> = md1.iter().map(|s| s.as_str()).collect();
    let mut d = md_codec::chunk::reassemble(&refs).expect("decode");
    let master = Xpub::from_str(&xpub_at(A, "m", "mainnet")).unwrap();
    let mut k65 = [0u8; 65];
    k65[..32].copy_from_slice(master.chain_code.as_bytes());
    k65[32..].copy_from_slice(&master.public_key.serialize());
    let pks = d.tlv.pubkeys.as_mut().expect("keyed card");
    for (idx, key) in pks.iter_mut() {
        if *idx == 0 {
            *key = k65;
        }
    }
    let md1 = md_codec::chunk::split(&d).expect("re-split");
    let r = restore(
        &md1,
        &[
            "--from",
            &format!("phrase={A}"),
            "--origin",
            "@1=m/48'/0'/0'/2'",
        ],
    );
    assert_eq!(r.code, 4, "{}", r.stderr);
    assert!(r.stderr.contains("at m."), "{}", r.stderr);
    assert!(r.stderr.contains("MASTER key"), "{}", r.stderr);
    // Review r1 M6 (FOLLOWUP fable-m1): the key found at m is never also
    // reported as carrying no path, and only @1 is asserted.
    assert!(!r.stderr.contains("Its key is shown"), "{}", r.stderr);
    assert!(!r.stderr.contains("no path is guessed"), "{}", r.stderr);
    assert_eq!(
        r.stderr.matches("@0 carries NO origin").count(),
        1,
        "{}",
        r.stderr
    );
    assert!(
        !r.stderr.contains("md1 cosigner @0's origin"),
        "{}",
        r.stderr
    );
}

#[test]
fn tk2_refugium_path_is_found_on_testnet() {
    // A key at the Refugium seat path m/270028'/1'/34' (round 1, seat 2).
    let x = xpub_at(A, "m/270028'/1'/34'", "testnet");
    let o = run(&[
        "bundle",
        "--network",
        "testnet",
        "--descriptor",
        "wsh(sortedmulti(2,@0/**,@1/**))",
        "--slot",
        &format!("@0.xpub={x}"),
        "--slot",
        &format!("@0.fingerprint={FP_A}"),
        "--slot",
        "@0.path=m/270028'/1'/34'",
        "--slot",
        &format!("@1.phrase={B}"),
        "--json",
    ]);
    assert_eq!(o.code, 4, "{}", o.stderr);
    let md1 = with_empty_origins(&md1_of(&o.stdout));
    let r = restore(
        &md1,
        &[
            "--network",
            "testnet",
            "--from",
            &format!("phrase={A}"),
            "--origin",
            "@1=m/48'/1'/0'/2'",
        ],
    );
    assert_eq!(r.code, 4, "{}", r.stderr);
    assert!(r.stderr.contains("m/270028'/1'/34'"), "{}", r.stderr);
    assert!(
        r.stdout.contains(&format!("[{FP_A}/270028'/1'/34']")),
        "{}",
        r.stdout
    );
}

/// Review r1 B5: BIP-45's shared key is the purpose key m/45' ("Only the
/// hardened purpose extended public key is shared"), so a card whose @N
/// came from a BIP-45 wallet and lost its origin is found there. (The card
/// is a canonical wsh shape because a non-canonical one cannot carry an empty
/// origin; the search does not depend on the shape.)
#[test]
fn b5_bip45_purpose_key_is_found_by_the_empty_origin_search() {
    let x = xpub_at(A, "m/45'", "mainnet");
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        "wsh(sortedmulti(2,@0/**,@1/**))",
        "--slot",
        &format!("@0.xpub={x}"),
        "--slot",
        &format!("@0.fingerprint={FP_A}"),
        "--slot",
        "@0.path=m/45'",
        "--slot",
        &format!("@1.phrase={B}"),
        "--json",
    ]);
    assert_eq!(o.code, 4, "{}", o.stderr);
    let md1 = with_empty_origins(&md1_of(&o.stdout));
    let r = restore(
        &md1,
        &[
            "--from",
            &format!("phrase={A}"),
            "--origin",
            "@1=m/48'/0'/0'/2'",
        ],
    );
    assert_eq!(r.code, 4, "{}", r.stderr);
    assert!(r.stderr.contains("at m/45'."), "{}", r.stderr);
    assert!(r.stdout.contains(&format!("[{FP_A}/45']")), "{}", r.stdout);
}

// ---------------------------------------------------------------------------
// toolkit#4 — template mode refuses a path / fingerprint beside a seed.
// ---------------------------------------------------------------------------

#[test]
fn tk4_template_mode_refuses_path_beside_seed() {
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "bip84",
        "--slot",
        &format!("@0.phrase={A}"),
        "--slot",
        "@0.path=m/84'/0'/5'",
        "--slot",
        "@0.fingerprint=deadbeef",
        "--json",
    ]);
    assert_eq!(o.code, 2, "{}", o.stderr);
    assert!(o.stderr.contains("--template mode ignores"), "{}", o.stderr);
    assert!(o.stdout.is_empty());
}

#[test]
fn tk4_template_mode_refuses_path_beside_ms1_and_seedqr_too() {
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "bip84",
        "--slot",
        "@0.ms1=ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f",
        "--slot",
        "@0.path=m/84'/0'/5'",
    ]);
    assert_eq!(o.code, 2, "{}", o.stderr);
    assert!(o.stderr.contains("--template mode ignores"), "{}", o.stderr);
}

// ---------------------------------------------------------------------------
// toolkit#5 — defaulted origin contradicted by the xpub header.
// ---------------------------------------------------------------------------

#[test]
fn tk5_bip48_xpub_under_bip87_default_is_refused() {
    let x0 = xpub_at(A, "m/48'/0'/0'/2'", "mainnet");
    let x1 = xpub_at(B, "m/48'/0'/0'/2'", "mainnet");
    let base = [
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "wsh-sortedmulti",
        "--threshold",
        "2",
    ];
    let mut args: Vec<String> = base.iter().map(|s| s.to_string()).collect();
    for (i, (x, fp)) in [(&x0, FP_A), (&x1, FP_B)].iter().enumerate() {
        args.push("--slot".into());
        args.push(format!("@{i}.xpub={x}"));
        args.push("--slot".into());
        args.push(format!("@{i}.fingerprint={fp}"));
    }
    let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let o = run(&refs);
    assert_eq!(o.code, 2, "{}", o.stderr);
    assert!(o.stderr.contains("@0.path="), "{}", o.stderr);
    assert!(o.stdout.is_empty());

    // Stating the real path keeps working.
    for i in 0..2 {
        args.push("--slot".into());
        args.push(format!("@{i}.path=m/48'/0'/0'/2'"));
    }
    args.push("--json".into());
    let refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let o = run(&refs);
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert!(o.stdout.contains("m/48'/0'/0'/2'"), "{}", o.stdout);
}

#[test]
fn tk5_xpub_whose_header_matches_the_default_is_unchanged() {
    let acct = xpub_at(A, "m/84'/0'/0'", "mainnet");
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "bip84",
        "--slot",
        &format!("@0.xpub={acct}"),
        "--json",
    ]);
    assert_eq!(o.code, 0, "{}", o.stderr);
    assert!(
        o.stdout.contains("\"origin_path\":\"m/84'/0'/0'\""),
        "{}",
        o.stdout
    );
}

// ---------------------------------------------------------------------------
// toolkit#6 — `--account >= 2^31` is a clean usage error, not a panic.
// ---------------------------------------------------------------------------

#[test]
fn tk6_account_at_2p31_is_refused_in_every_command() {
    let xpub = xpub_at(A, "m/84'/0'/0'", "mainnet");
    let phrase = format!("phrase={A}");
    let seed_slot = format!("@0.phrase={A}");
    let xpub_slot = format!("@0.xpub={xpub}");
    let cases: Vec<Vec<&str>> = vec![
        vec!["bundle", "--template", "bip84", "--slot", &seed_slot],
        vec!["bundle", "--template", "bip84", "--slot", &xpub_slot],
        vec!["verify-bundle", "--template", "bip84", "--slot", &seed_slot],
        vec!["export-wallet", "--template", "bip84", "--slot", &xpub_slot],
        vec!["addresses", "--from", &phrase, "--address-type", "p2wpkh"],
        vec!["restore", "--from", &phrase, "--template", "bip84"],
        vec![
            "convert",
            "--from",
            &phrase,
            "--to",
            "xpub",
            "--template",
            "bip84",
        ],
        vec!["silent-payment", "--secret", A],
    ];
    for mut c in cases {
        c.extend(["--account", "2147483648"]);
        let o = run(&c);
        no_panic(&o);
        assert_eq!(o.code, 64, "{c:?}: clap usage error: {}", o.stderr);
        assert!(o.stderr.contains("2^31"), "{c:?}: {}", o.stderr);
    }
}

#[test]
fn tk6_largest_hardened_account_still_parses() {
    let o = run(&[
        "addresses",
        "--from",
        &format!("phrase={A}"),
        "--address-type",
        "p2wpkh",
        "--account",
        "2147483647",
    ]);
    no_panic(&o);
    assert_eq!(o.code, 0, "{}", o.stderr);
}

// ---------------------------------------------------------------------------
// toolkit#7 — legacy sh(sortedmulti) default. Superseded by the 2026-11-25
// BIP-defaults ruling: BIP-48 assigns no path to bare P2SH, so the default is
// BIP-87 with a warning naming m/45' (see bughunt_1125_r1 b2_*).
// ---------------------------------------------------------------------------

#[test]
fn tk7_legacy_sh_sortedmulti_defaults_to_bip87_with_a_warning() {
    let o = run(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        "sh(sortedmulti(2,@0/**,@1/**))",
        "--slot",
        &format!("@0.phrase={A}"),
        "--slot",
        &format!("@1.phrase={B}"),
        "--json",
    ]);
    assert_eq!(o.code, 4, "{}", o.stderr);
    assert!(
        o.stderr.contains("chose m/87'/0'/0' (BIP-87)") && o.stderr.contains("m/45'"),
        "bare sh(...) defaults to BIP-87 and names m/45'; {}",
        o.stderr
    );
    assert!(
        !o.stderr.contains("m/48'/0'/0'/1'"),
        "BIP-48 1' is P2SH-P2WSH, not bare P2SH; {}",
        o.stderr
    );
}

// ---------------------------------------------------------------------------
// X4 — a WIF slot is refused at mint.
// ---------------------------------------------------------------------------

#[test]
fn x4_wif_slot_is_refused_at_mint() {
    for (template, extra) in [
        ("bip84", vec![]),
        ("wsh-sortedmulti", vec!["--threshold", "2"]),
    ] {
        let seed_slot = format!("@1.phrase={A}");
        let wif_slot = format!("@0.wif={MAINNET_WIF}");
        let mut args = vec!["bundle", "--network", "mainnet", "--template", template];
        args.extend(extra.iter().copied());
        args.extend(["--slot", &wif_slot]);
        if template != "bip84" {
            args.extend(["--slot", &seed_slot]);
        }
        let o = run(&args);
        assert_eq!(o.code, 1, "{template}: {}", o.stderr);
        assert!(o.stderr.contains("@0.wif is refused"), "{}", o.stderr);
        assert!(o.stdout.is_empty(), "nothing minted");
    }
}
