//! Wallet-file cross-format convergence + hop idempotence (metamorphic).
//!
//! Design: `design/SPEC_wallet_cross_format_convergence_tests.md`
//! (R0 RED 0C/2I → folded → R1 GREEN; reviews in
//! `design/agent-reports/wallet-convergence-R{0,1}-review.md`).
//!
//! Property: the SAME wallet, expressed in different wallet-file formats, must
//! import to the SAME canonical key-material. Byte-/descriptor-identity is
//! impossible across formats by design (finding F1: bitcoin-core splits the
//! `<0;1>` multipath, formats differ on metadata), so convergence is asserted
//! on the DECODED key-material: xpub multiset + per-cosigner (xpub, fingerprint,
//! origin-path) triples + fingerprint set + threshold/N + md1 policy facts +
//! network. EXCLUDED: raw descriptor string, ms1 sentinels, format metadata.
//!
//! Construction: export-generate ONE in-test wallet to each round-trippable
//! format, re-import each, compare envelopes (F2-safe; same-wallet-by-
//! construction). Self-contained: only the `mnemonic` binary + md_codec/mk_codec
//! libs; no sibling binary, no network → default `cargo test`, no `#[ignore]`.

use assert_cmd::Command;
use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use serde_json::Value;
use std::collections::BTreeSet;
use std::str::FromStr;

const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";
const MS_PHRASES: [&str; 3] = [
    TREZOR_24,
    "legal winner thank year wave sausage worth useful legal winner thank yellow",
    "letter advice cage absurd amount doctor acoustic avoid letter advice cage above",
];

/// (xpub, master_fingerprint_hex) derived in-test from `phrase` at `path` on
/// mainnet (empty passphrase). Pattern: `cli_cross_start_convergence.rs:33`.
fn derive(phrase: &str, path: &str) -> (String, String) {
    let secp = Secp256k1::new();
    let m = Mnemonic::parse_in(bip39::Language::English, phrase).unwrap();
    let seed = m.to_seed("");
    let master = Xpriv::new_master(bitcoin::NetworkKind::Main, &seed).unwrap();
    let fp = master.fingerprint(&secp).to_string().to_lowercase();
    let dp = DerivationPath::from_str(path).unwrap();
    let xpub = Xpub::from_priv(&secp, &master.derive_priv(&secp, &dp).unwrap()).to_string();
    (xpub, fp)
}

fn run(args: &[String], stdin: Option<&str>) -> (bool, String, String) {
    let mut cmd = Command::cargo_bin("mnemonic").unwrap();
    cmd.args(args);
    if let Some(s) = stdin {
        cmd.write_stdin(s.to_string());
    }
    let out = cmd.assert();
    let o = out.get_output();
    (
        o.status.success(),
        String::from_utf8(o.stdout.clone()).unwrap(),
        String::from_utf8(o.stderr.clone()).unwrap(),
    )
}

/// Per-format required-field extras (e.g. specter/jade need a wallet name).
fn format_extras(format: &str) -> Vec<String> {
    match format {
        "specter" | "jade" | "sparrow" => vec!["--wallet-name".into(), "conv".into()],
        _ => vec![],
    }
}

/// Export the multisig wallet (2-of-3) to `format` via `export-wallet`.
fn export_multisig(format: &str, template: &str, cosigners: &[(String, String)]) -> String {
    let mut a: Vec<String> = [
        "export-wallet",
        "--format",
        format,
        "--template",
        template,
        "--threshold",
        "2",
        "--multisig-path-family",
        "bip48",
        "--network",
        "mainnet",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    a.extend(format_extras(format));
    for (i, (xpub, fp)) in cosigners.iter().enumerate() {
        a.push("--slot".into());
        a.push(format!("@{i}.xpub={xpub}"));
        a.push("--slot".into());
        a.push(format!("@{i}.fingerprint={fp}"));
        a.push("--slot".into());
        a.push(format!("@{i}.path=m/48'/0'/0'/2'"));
    }
    let (ok, stdout, stderr) = run(&a, None);
    assert!(
        ok,
        "export-wallet --format {format} ({template}) failed: {stderr}"
    );
    stdout
}

fn export_singlesig(format: &str, xpub: &str, fp: &str) -> String {
    let mut a: Vec<String> = [
        "export-wallet",
        "--format",
        format,
        "--template",
        "bip84",
        "--network",
        "mainnet",
        "--slot",
        &format!("@0.xpub={xpub}"),
        "--slot",
        &format!("@0.fingerprint={fp}"),
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    a.extend(format_extras(format));
    let (ok, stdout, stderr) = run(&a, None);
    assert!(
        ok,
        "export-wallet --format {format} (bip84) failed: {stderr}"
    );
    stdout
}

/// Apply the per-format export→import massage (R0 I1/§1.2): bitcoin-core needs
/// the export's bare array wrapped in a listdescriptors object; jade needs the
/// bare coldcard-text body wrapped as `{"multisig_file": "<text>"}`.
fn wrap_for_import(format: &str, exported: &str) -> String {
    match format {
        "bitcoin-core" => {
            let arr: Value = serde_json::from_str(exported).expect("core export is JSON array");
            serde_json::json!({ "wallet_name": "conv", "descriptors": arr }).to_string()
        }
        "jade" => {
            serde_json::json!({ "multisig_name": "conv", "multisig_file": exported }).to_string()
        }
        _ => exported.to_string(),
    }
}

/// export → (wrap) → import → return the `bundle` object of the single envelope entry.
fn export_then_import_bundle(format: &str, exported: &str) -> Value {
    let blob = wrap_for_import(format, exported);
    let a: Vec<String> = ["import-wallet", "--format", format, "--blob", "-", "--json"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let (ok, stdout, stderr) = run(&a, Some(&blob));
    assert!(
        ok,
        "import-wallet --format {format} failed: {stderr}\nblob:\n{blob}"
    );
    let env: Value = serde_json::from_str(&stdout).expect("import --json is valid JSON");
    env.as_array().expect("import --json is an array")[0]["bundle"].clone()
}

#[derive(Debug, PartialEq, Eq)]
struct KeyMaterial {
    triples: BTreeSet<(String, String, String)>, // (xpub, fingerprint_lc, origin_path)
    fingerprints: BTreeSet<String>,
    threshold: Option<u64>,
    cosigner_count: usize,
    md1_wallet_policy: bool,
    md1_tree_tag: String,          // top wrapper tag (Wsh / Sh / Wpkh / …)
    md1_multi_tag: Option<String>, // inner Multi / SortedMulti / MultiA / SortedMultiA, if any
    md1_n: u64,
    network: String,
}

/// First multi-family tag (Multi/SortedMulti/MultiA/SortedMultiA) in the tree.
fn find_multi_tag(node: &md_codec::tree::Node) -> Option<String> {
    use md_codec::tag::Tag;
    if matches!(
        node.tag,
        Tag::Multi | Tag::SortedMulti | Tag::MultiA | Tag::SortedMultiA
    ) {
        return Some(format!("{:?}", node.tag));
    }
    if let md_codec::tree::Body::Children(children) = &node.body {
        for c in children {
            if let Some(t) = find_multi_tag(c) {
                return Some(t);
            }
        }
    }
    None
}

/// Cosigner xpubs in mk1 declaration order (Vec, order-significant — for C4's
/// unsorted-multi positional check). Set-based comparison hides reordering.
fn ordered_xpubs(bundle: &Value) -> Vec<String> {
    let mk1 = bundle["mk1"].as_array().unwrap();
    let sets: Vec<Vec<String>> = if mk1.first().map(|v| v.is_string()).unwrap_or(false) {
        vec![mk1
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect()]
    } else {
        mk1.iter()
            .map(|c| {
                c.as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_string())
                    .collect()
            })
            .collect()
    };
    sets.iter()
        .map(|chunks| {
            let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
            mk_codec::decode(&refs).unwrap().xpub.to_string()
        })
        .collect()
}

/// Extract the canonical key-material from an import envelope `bundle` object.
/// Handles single-sig (`mk1` flat array = one card) and multisig (`mk1` nested
/// = one card per cosigner). Compares decoded key-material only (NOT descriptor
/// string / ms1 / metadata).
fn key_material(bundle: &Value) -> KeyMaterial {
    // mk1: flat Vec<String> (single-sig) OR Vec<Vec<String>> (multisig).
    let mk1 = bundle["mk1"].as_array().expect("mk1 array");
    let card_chunk_sets: Vec<Vec<String>> = if mk1.first().map(|v| v.is_string()).unwrap_or(false) {
        vec![mk1
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect()]
    } else {
        mk1.iter()
            .map(|c| {
                c.as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_string())
                    .collect()
            })
            .collect()
    };
    let mut triples = BTreeSet::new();
    let mut fingerprints = BTreeSet::new();
    for chunks in &card_chunk_sets {
        let refs: Vec<&str> = chunks.iter().map(String::as_str).collect();
        let card = mk_codec::decode(&refs).expect("mk1 decode");
        let fp = card
            .origin_fingerprint
            .map(|f| f.to_string().to_lowercase())
            .unwrap_or_default();
        let path = card.origin_path.to_string();
        triples.insert((card.xpub.to_string(), fp.clone(), path));
        if !fp.is_empty() {
            fingerprints.insert(fp);
        }
    }

    let md1: Vec<String> = bundle["md1"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    let md1_refs: Vec<&str> = md1.iter().map(String::as_str).collect();
    let desc = md_codec::chunk::reassemble(&md1_refs).expect("md1 reassemble");

    let (threshold, cosigner_count) = match bundle["multisig"].as_object() {
        Some(m) => (
            m["threshold"].as_u64(),
            m["cosigner_count"].as_u64().unwrap_or(1) as usize,
        ),
        None => (None, 1),
    };

    KeyMaterial {
        triples,
        fingerprints,
        threshold,
        cosigner_count,
        md1_wallet_policy: desc.is_wallet_policy(),
        md1_tree_tag: format!("{:?}", desc.tree.tag),
        md1_multi_tag: find_multi_tag(&desc.tree),
        md1_n: desc.n as u64,
        network: bundle["network"].as_str().unwrap_or("").to_string(),
    }
}

// ===========================================================================
// C2 (Phase-1 probe) — wsh-sortedmulti 2-of-3 converges across 7 formats.
// ===========================================================================
#[test]
fn c2_multisig_sortedmulti_converges_across_formats() {
    let cosigners: Vec<(String, String)> = MS_PHRASES
        .iter()
        .map(|p| derive(p, "m/48'/0'/0'/2'"))
        .collect();
    let formats = [
        "bitcoin-core",
        "bsms",
        "coldcard-multisig",
        "electrum",
        "jade",
        "sparrow",
        "specter",
    ];

    let anchor = key_material(&export_then_import_bundle(
        formats[0],
        &export_multisig(formats[0], "wsh-sortedmulti", &cosigners),
    ));
    // Sanity: the anchor really is the 3-cosigner wallet.
    assert_eq!(anchor.cosigner_count, 3, "C2 anchor cosigner count");
    assert_eq!(anchor.threshold, Some(2), "C2 anchor threshold");
    assert_eq!(anchor.triples.len(), 3, "C2 anchor distinct cosigners");

    for f in &formats[1..] {
        let km = key_material(&export_then_import_bundle(
            f,
            &export_multisig(f, "wsh-sortedmulti", &cosigners),
        ));
        assert_eq!(
            km, anchor,
            "C2: {f} must converge with {} on decoded key-material",
            formats[0]
        );
    }
}

// ===========================================================================
// C1 — single-sig bip84 wpkh converges across 5 formats.
// ===========================================================================
#[test]
fn c1_singlesig_wpkh_converges_across_formats() {
    let (xpub, fp) = derive(TREZOR_24, "m/84'/0'/0'");
    let formats = ["bitcoin-core", "coldcard", "electrum", "sparrow", "specter"];
    let anchor = key_material(&export_then_import_bundle(
        formats[0],
        &export_singlesig(formats[0], &xpub, &fp),
    ));
    assert_eq!(anchor.cosigner_count, 1, "C1 single-sig");
    for f in &formats[1..] {
        let km = key_material(&export_then_import_bundle(
            f,
            &export_singlesig(f, &xpub, &fp),
        ));
        assert_eq!(km, anchor, "C1: {f} must converge with {}", formats[0]);
    }
}

// ===========================================================================
// C-neg — a genuinely different wallet must NOT converge (anti-vacuity).
// ===========================================================================
#[test]
fn c_neg_different_wallet_does_not_converge() {
    let cosigners: Vec<(String, String)> = MS_PHRASES
        .iter()
        .map(|p| derive(p, "m/48'/0'/0'/2'"))
        .collect();
    // Swap cosigner 2 for a different (valid) key: same seed, different account
    // path → distinct xpub → genuinely different wallet.
    let mut other = cosigners.clone();
    other[2] = derive(MS_PHRASES[2], "m/48'/0'/1'/2'");
    assert_ne!(
        other[2].0, cosigners[2].0,
        "C-neg: swapped cosigner xpub must differ"
    );
    let a = key_material(&export_then_import_bundle(
        "bsms",
        &export_multisig("bsms", "wsh-sortedmulti", &cosigners),
    ));
    let b = key_material(&export_then_import_bundle(
        "bsms",
        &export_multisig("bsms", "wsh-sortedmulti", &other),
    ));
    assert_ne!(a, b, "C-neg: different wallet must not converge");
}

// ===========================================================================
// C3 — sh-wsh-sortedmulti 2-of-3 converges across 7 formats (P2SH-P2WSH path).
// ===========================================================================
#[test]
fn c3_multisig_sh_wsh_sortedmulti_converges_across_formats() {
    let cosigners: Vec<(String, String)> = MS_PHRASES
        .iter()
        .map(|p| derive(p, "m/48'/0'/0'/1'"))
        .collect();
    let formats = [
        "bitcoin-core",
        "bsms",
        "coldcard-multisig",
        "electrum",
        "jade",
        "sparrow",
        "specter",
    ];
    let anchor = key_material(&export_then_import_bundle(
        formats[0],
        &export_multisig(formats[0], "sh-wsh-sortedmulti", &cosigners),
    ));
    assert_eq!(anchor.md1_tree_tag, "Sh", "C3 anchor is P2SH-wrapped");
    for f in &formats[1..] {
        let km = key_material(&export_then_import_bundle(
            f,
            &export_multisig(f, "sh-wsh-sortedmulti", &cosigners),
        ));
        assert_eq!(km, anchor, "C3: {f} must converge with {}", formats[0]);
    }
}

// ===========================================================================
// C4 — wsh-MULTI (UNSORTED) 2-of-3: order-preserving formats converge WITH
// declaration order; coldcard-multisig is probed for reorder/coercion.
// ===========================================================================
#[test]
fn c4_unsorted_multi_order_preservation() {
    let cosigners: Vec<(String, String)> = MS_PHRASES
        .iter()
        .map(|p| derive(p, "m/48'/0'/0'/2'"))
        .collect();
    let order_preserving = ["bitcoin-core", "bsms", "sparrow", "specter"];

    let anchor_b = export_then_import_bundle(
        order_preserving[0],
        &export_multisig(order_preserving[0], "wsh-multi", &cosigners),
    );
    let anchor_km = key_material(&anchor_b);
    let anchor_order = ordered_xpubs(&anchor_b);
    assert_eq!(
        anchor_km.md1_multi_tag.as_deref(),
        Some("Multi"),
        "C4 anchor is UNSORTED multi"
    );
    assert_eq!(anchor_order.len(), 3, "C4 anchor 3 cosigners");

    for f in &order_preserving[1..] {
        let b = export_then_import_bundle(f, &export_multisig(f, "wsh-multi", &cosigners));
        assert_eq!(
            key_material(&b),
            anchor_km,
            "C4: {f} key-material must converge"
        );
        assert_eq!(
            ordered_xpubs(&b),
            anchor_order,
            "C4: {f} must PRESERVE declaration order (unsorted multi)"
        );
    }

    // Probe coldcard-multisig: its file format is BIP-67 sortedmulti-only (no
    // field to express literal `multi(...)` key order), so an unsorted
    // `wsh-multi` export would silently coerce to sortedmulti → different
    // witnessScript/address. As of cycle-2 H10 the export now REFUSES (exit 2,
    // typed `ExportWalletUnsortedMultisigUnsupported`) rather than emitting the
    // silently-reordered file. This was previously a documented by-design
    // coercion; it is now a funds-safety refusal.
    let mut probe: Vec<String> = [
        "export-wallet",
        "--format",
        "coldcard-multisig",
        "--template",
        "wsh-multi",
        "--threshold",
        "2",
        "--multisig-path-family",
        "bip48",
        "--network",
        "mainnet",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    for (i, (xpub, fp)) in cosigners.iter().enumerate() {
        probe.push("--slot".into());
        probe.push(format!("@{i}.xpub={xpub}"));
        probe.push("--slot".into());
        probe.push(format!("@{i}.fingerprint={fp}"));
        probe.push("--slot".into());
        probe.push(format!("@{i}.path=m/48'/0'/0'/2'"));
    }
    let (ok, _stdout, stderr) = run(&probe, None);
    assert!(
        !ok,
        "C4 (H10): coldcard-multisig must REFUSE an unsorted wsh-multi (no silent sortedmulti coercion); got success"
    );
    assert!(
        stderr.contains("UNSORTED multisig") && stderr.contains("sortedmulti-only"),
        "C4 (H10): refusal must explain the unsorted→sortedmulti hazard; got: {stderr}"
    );
}

// ===========================================================================
// Part 2 — hop idempotence: import A → (envelope) → export B → import B,
// assert key-material == the direct import-A envelope. The cross-format hop
// chains THROUGH the envelope via `export-wallet --from-import-json -` (reads
// stdin), so it genuinely exercises the A→B conversion (distinct from Part 1,
// which export-generates each format independently from raw slots).
// ===========================================================================

/// import format A's file → return (full envelope JSON string, bundle Value).
fn export_then_import_full(format: &str, exported: &str) -> (String, Value) {
    let blob = wrap_for_import(format, exported);
    let a: Vec<String> = ["import-wallet", "--format", format, "--blob", "-", "--json"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let (ok, stdout, stderr) = run(&a, Some(&blob));
    assert!(ok, "import-wallet --format {format} failed: {stderr}");
    let env: Value = serde_json::from_str(&stdout).unwrap();
    (stdout.clone(), env.as_array().unwrap()[0]["bundle"].clone())
}

/// Hop A→B: import A (E_A) → `export-wallet --from-import-json -` to format B
/// → import B (E_AB) → assert key-material(E_A) == key-material(E_AB).
fn assert_hop(a_fmt: &str, b_fmt: &str, a_file: &str) {
    let (env_a_json, e_a) = export_then_import_full(a_fmt, a_file);
    let mut ex: Vec<String> = [
        "export-wallet",
        "--from-import-json",
        "-",
        "--format",
        b_fmt,
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    ex.extend(format_extras(b_fmt));
    let (ok, b_file, stderr) = run(&ex, Some(&env_a_json));
    assert!(
        ok,
        "hop {a_fmt}→{b_fmt}: export-wallet --from-import-json --format {b_fmt} failed: {stderr}"
    );
    let e_ab = export_then_import_bundle(b_fmt, &b_file);
    assert_eq!(
        key_material(&e_a),
        key_material(&e_ab),
        "hop {a_fmt}→{b_fmt}: key-material must survive the cross-format conversion"
    );
}

#[test]
fn h_hop_idempotence_multisig_pairs() {
    let cosigners: Vec<(String, String)> = MS_PHRASES
        .iter()
        .map(|p| derive(p, "m/48'/0'/0'/2'"))
        .collect();
    let file = |f: &str| export_multisig(f, "wsh-sortedmulti", &cosigners);
    assert_hop("bsms", "bitcoin-core", &file("bsms")); // H1 multipath-split seam (core as target)
    assert_hop("sparrow", "coldcard-multisig", &file("sparrow")); // H2 order/fp seam
                                                                  // H3 SLIP-132 seam: base58 xpub source → electrum Zpub. Uses sparrow (single
                                                                  // import entry); bitcoin-core can't be the SOURCE here because it splits the
                                                                  // <0;1> multipath into 2 descriptors (F1) → a 2-entry envelope that
                                                                  // --from-import-json correctly requires an index to disambiguate.
    assert_hop("sparrow", "electrum", &file("sparrow")); // H3 SLIP-132 seam
    assert_hop("specter", "jade", &file("specter")); // H4 descriptor-JSON → jade text
    assert_hop("coldcard-multisig", "bsms", &file("coldcard-multisig")); // H6 text → bsms
}

#[test]
fn h5_hop_idempotence_singlesig_electrum_sparrow() {
    let (xpub, fp) = derive(TREZOR_24, "m/84'/0'/0'");
    assert_hop(
        "electrum",
        "sparrow",
        &export_singlesig("electrum", &xpub, &fp),
    ); // H5 SS SLIP-132
}

// ===========================================================================
// A1 — concrete↔@N descriptor convergence (metamorphic: both forms must emit
// the same md1 + mk1). Uses testnet, out-of-lex-order sortedmulti, both
// cosigners origin-bearing.
// ===========================================================================
#[test]
fn concrete_vs_atn_descriptor_converge_md1_mk1() {
    use assert_cmd::Command;
    let mnem = || Command::cargo_bin("mnemonic").unwrap();
    // Out-of-lexicographic-order sortedmulti (97139860 before 704c7836), both
    // inputs explicitly origin-bearing.
    let concrete = "wsh(sortedmulti(2,[97139860/48'/1'/2'/2']tpubDFiXyf7zmBhQrSHoAQB6SmMpF3rfSihAxQGMdQUtZfE8HWHkWLLNLTiYpMzvHnFiTmuUSYieHUYv4tFguzmiHeDrYV8TtWGCWt5qpqox4w3/<0;1>/*,[704c7836/48'/1'/3'/2']tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC/<0;1>/*))";
    let atn =
        "wsh(sortedmulti(2,@0[97139860/48'/1'/2'/2']/<0;1>/*,@1[704c7836/48'/1'/3'/2']/<0;1>/*))";

    let c = mnem()
        .args([
            "bundle",
            "--descriptor",
            concrete,
            "--network",
            "testnet",
            "--json",
        ])
        .output()
        .unwrap();
    let a = mnem().args(["bundle", "--descriptor", atn, "--network", "testnet",
        "--slot", "@0.xpub=tpubDFiXyf7zmBhQrSHoAQB6SmMpF3rfSihAxQGMdQUtZfE8HWHkWLLNLTiYpMzvHnFiTmuUSYieHUYv4tFguzmiHeDrYV8TtWGCWt5qpqox4w3",
        "--slot", "@1.xpub=tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC",
        "--json"]).output().unwrap();
    assert!(
        c.status.success() && a.status.success(),
        "c={} a={}",
        String::from_utf8_lossy(&c.stderr),
        String::from_utf8_lossy(&a.stderr)
    );
    let cv: serde_json::Value = serde_json::from_slice(&c.stdout).unwrap();
    let av: serde_json::Value = serde_json::from_slice(&a.stdout).unwrap();
    assert_eq!(cv["md1"], av["md1"], "md1 diverged");
    assert_eq!(cv["mk1"], av["mk1"], "mk1 diverged");
}

#[test]
fn concrete_duplicate_cosigner_rejected_bip388() {
    use assert_cmd::Command;
    let dup = "wsh(sortedmulti(2,[704c7836/48'/1'/3'/2']tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC/<0;1>/*,[704c7836/48'/1'/3'/2']tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC/<0;1>/*))";
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["bundle", "--descriptor", dup, "--network", "testnet"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(
        String::from_utf8_lossy(&out.stderr)
            .to_lowercase()
            .contains("distinct"),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}
