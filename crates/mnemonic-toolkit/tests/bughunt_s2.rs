//! Bug-hunt triage RED rows, mnemonic-toolkit, slice S2 (descriptor parsing,
//! `bundle`, Electrum seed + export). Source: `design/agent-reports/
//! bughunt-2026-10-31-tk.md` bugs 3, 4, 5, 6, 9, 11, 45, 46, 52, 54.
//!
//! Every test asserts the CORRECT behaviour, so it FAILS on unmodified code if
//! the reported bug is real. Where the report leaves the correct behaviour
//! open, the test asserts only that the reported wrong behaviour is absent
//! (the open product question is named in the triage report).
//!
//! The `wc-codec` bugs of this slice (7, 8) are in
//! `crates/wc-codec/tests/bughunt_s2.rs`.

use assert_cmd::Command;
use serde_json::Value;
use std::process::Output;

const XPUB_A: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
const FP_A: &str = "704c7836";
const XPUB_B: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";
const FP_B: &str = "97139860";

/// 16 zero bytes of entropy = the BIP-39 phrase "abandon x11 about"; its
/// master fingerprint is `73c5da0a` (mainnet, empty passphrase).
const ZERO_ENTROPY_HEX: &str = "00000000000000000000000000000000";
/// The `ms10entr` codex32 card for `ZERO_ENTROPY_HEX` (emitted by the binary).
const ZERO_ENTROPY_MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

fn mn(args: &[&str]) -> Output {
    Command::cargo_bin("mnemonic")
        .expect("mnemonic binary builds")
        .arg("--allow-argv-secret")
        .args(args)
        .output()
        .expect("mnemonic runs")
}

fn out_s(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn err_s(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

fn json(o: &Output) -> Value {
    serde_json::from_slice(&o.stdout)
        .unwrap_or_else(|e| panic!("stdout is not JSON ({e}):\n{}", out_s(o)))
}

// ===========================================================================
// #3 -- a use-site with a `/<a;b>` multipath but NO trailing `/*` is accepted
// and encoded as the ranged `/<a;b>/*` wallet.
//
// Scope note (triage): the report's second example, bare `wpkh(@0)`, is the
// documented D1-deferred keyless-template idiom (`lex_bare_at_zero`,
// `lex_residue_floor_accepts_bare_at_n_d1_deferred`,
// `bundle_atn_bare_template_unaffected_by_new_check`) and is NOT asserted
// here. Only the explicit multipath-without-wildcard shape is.
// ===========================================================================

#[test]
fn bughunt_tk_03_multipath_without_wildcard_rejected() {
    // `parse_descriptor` is `#[cfg(fuzzing)]`-gated (not public API), so both
    // forms go through the `bundle --descriptor` CLI.
    let slot_xpub = format!("@0.xpub={XPUB_A}");
    let slot_fp = format!("@0.fingerprint={FP_A}");
    let concrete = format!("wpkh([{FP_A}/84'/0'/0']{XPUB_A}/<0;1>)");
    let mut failures: Vec<String> = Vec::new();
    for (label, desc, slots) in [
        (
            "placeholder",
            "wpkh(@0/<0;1>)".to_string(),
            vec!["--slot", slot_xpub.as_str(), "--slot", slot_fp.as_str()],
        ),
        ("concrete-xpub", concrete, vec![]),
    ] {
        let mut args = vec![
            "bundle",
            "--network",
            "mainnet",
            "--descriptor",
            desc.as_str(),
        ];
        args.extend(slots);
        args.push("--json");
        let o = mn(&args);
        if o.status.success() {
            failures.push(format!(
                "{label}: `{desc}` exited 0 (md1 {})",
                json(&o)["md1"][0]
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "a multipath use-site with no trailing `/*` is not representable in md1 \
         (UseSitePath is always wildcarded) and must be refused, not encoded as \
         `/<0;1>/*`:\n{}",
        failures.join("\n")
    );
}

// ===========================================================================
// #4 -- Electrum normalize_text misses Hangul Jamo (and other CJK ranges), so
// the space between two Hangul syllables survives in the passphrase and the
// seed differs from Electrum's.
//
// Electrum (spesmilo/electrum e1099925, electrum/mnemonic.py:62):
//   (0x1100, 0x11FF, 'Hangul Jamo'),
// and normalize_text line 90 strips whitespace between two is_CJK chars, AFTER
// NFKD has split Hangul syllables into Jamo.
// ===========================================================================

#[test]
fn bughunt_tk_04_electrum_passphrase_strips_space_between_hangul() {
    // Electrum standard-segwit test seed (cli_addresses_electrum.rs SW_SEED).
    let seed = "bitter grass shiver impose acquire brush forget axis eager alone wine silver";
    let from = format!("electrum-phrase={seed}");
    let run = |pp: &str| {
        mn(&[
            "addresses",
            "--from",
            &from,
            "--passphrase",
            pp,
            "--address-type",
            "p2wpkh",
            "--chain",
            "both",
            "--count",
            "1",
        ])
    };
    let spaced = run("한 글");
    let joined = run("한글");
    assert!(
        spaced.status.success() && joined.status.success(),
        "both runs must succeed: spaced={} joined={}",
        err_s(&spaced),
        err_s(&joined)
    );
    assert_eq!(
        out_s(&spaced),
        out_s(&joined),
        "Electrum strips the space between CJK characters (Hangul Jamo U+1100-11FF after \
         NFKD), so passphrases \"한 글\" and \"한글\" must derive the SAME addresses"
    );
}

// ===========================================================================
// #5 -- `export-wallet --format electrum --template bip86` writes a plain
// "standard" xpub keystore, which Electrum reads as p2pkh
// (electrum/wallet.py:4241 `self.txin_type = 'p2pkh' if xtype == 'standard'
// else xtype`). Correct: refuse, as for taproot multisig.
// ===========================================================================

#[test]
fn bughunt_tk_05_electrum_export_bip86_refused() {
    let slot_xpub = format!("@0.xpub={XPUB_A}");
    let slot_fp = format!("@0.fingerprint={FP_A}");
    let o = mn(&[
        "export-wallet",
        "--format",
        "electrum",
        "--template",
        "bip86",
        "--network",
        "mainnet",
        "--wallet-name",
        "T",
        "--slot",
        &slot_xpub,
        "--slot",
        &slot_fp,
        "--output",
        "-",
    ]);
    assert!(
        !o.status.success(),
        "export-wallet --format electrum --template bip86 exited 0 and wrote a plain-xpub \
         \"standard\" wallet that Electrum reads as p2pkh (wrong addresses). stdout:\n{}",
        out_s(&o)
    );
}

// ===========================================================================
// #6 -- descriptor-mode `bundle`: the `[fp/path]@N` annotation fingerprint is
// not cross-checked for `entropy` and `ms1` slots (the Phrase arm does).
// ===========================================================================

#[test]
fn bughunt_tk_06_descriptor_anno_fp_checked_for_entropy_and_ms1_slots() {
    let desc = "wsh(sortedmulti(2,[deadbeef/48'/0'/0'/2']@0,@1))";
    let slot_b = format!("@1.xpub={XPUB_B}");
    let mut failures: Vec<String> = Vec::new();
    for (label, slot0) in [
        ("entropy", format!("@0.entropy={ZERO_ENTROPY_HEX}")),
        ("ms1", format!("@0.ms1={ZERO_ENTROPY_MS1}")),
    ] {
        let o = mn(&[
            "bundle",
            "--network",
            "mainnet",
            "--descriptor",
            desc,
            "--slot",
            &slot0,
            "--slot",
            &slot_b,
            "--json",
        ]);
        if o.status.success() {
            let v = json(&o);
            failures.push(format!(
                "@0.{label}: exit 0; annotation deadbeef silently ignored, bundle records \
                 master_fingerprint {}",
                v["multisig"]["cosigners"][0]["master_fingerprint"]
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "annotation fingerprint deadbeef disagrees with the slot's derived fingerprint \
         73c5da0a and must be refused (as the Phrase arm does):\n{}",
        failures.join("\n")
    );
}

// ===========================================================================
// #9 -- descriptor-mode `bundle` silently drops an unparseable
// `--slot @N.fingerprint=` (`.ok()` at bundle.rs:1618).
// ===========================================================================

#[test]
fn bughunt_tk_09_unparseable_slot_fingerprint_refused() {
    let slot_xpub = format!("@0.xpub={XPUB_A}");
    let o = mn(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        "wpkh(@0/<0;1>/*)",
        "--slot",
        &slot_xpub,
        "--slot",
        "@0.fingerprint=zzzz",
        "--json",
    ]);
    let detail = if o.status.success() {
        format!(
            "exit 0, master_fingerprint {}",
            json(&o)["master_fingerprint"]
        )
    } else {
        String::new()
    };
    assert!(
        !o.status.success(),
        "--slot @0.fingerprint=zzzz is not a fingerprint and must be refused (template mode \
         refuses it); descriptor mode instead: {detail}"
    );
}

// ===========================================================================
// #11 -- a single-sig descriptor with a hardened use-site passes the
// `--md1-form=template` gate (`template_admissible` returns at `n == 1`
// before the hardened-use-site check).
// ===========================================================================

#[test]
fn bughunt_tk_11_singlesig_hardened_usesite_template_form_refused() {
    let slot_xpub = format!("@0.xpub={XPUB_A}");
    let slot_fp = format!("@0.fingerprint={FP_A}");
    let o = mn(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        "wpkh(@0/<0;1>/*h)",
        "--md1-form",
        "template",
        "--slot",
        &slot_xpub,
        "--slot",
        &slot_fp,
        "--json",
    ]);
    assert!(
        !o.status.success(),
        "bundle --md1-form=template accepted a hardened use-site (`/*h`) single-sig \
         descriptor and emitted a template that restore cannot rebuild; it must fail with \
         TemplateFormUnsupportedShape. stderr:\n{}",
        err_s(&o)
    );
    assert!(
        err_s(&o).contains("cannot template this descriptor shape"),
        "refusal must be the TemplateFormUnsupportedShape message, got:\n{}",
        err_s(&o)
    );
}

// ===========================================================================
// #45 -- `bundle --import-json` labels testnet and signet bundles "mainnet"
// (emit_args.network is never set from the envelope).
//
// Correct behaviour is open (envelope wins, or refuse when --network
// disagrees); the test asserts only that the bundle is NOT labelled with the
// wrong network while exiting 0.
// ===========================================================================

#[test]
fn bughunt_tk_45_import_json_testnet_envelope_not_labelled_mainnet() {
    let imported = mn(&[
        "import-wallet",
        "--format",
        "bsms",
        "--blob",
        "tests/fixtures/wallet_import/bsms-testnet-2of2.txt",
        "--json",
    ]);
    assert!(
        imported.status.success(),
        "import-wallet: {}",
        err_s(&imported)
    );
    let dir = tempfile::tempdir().unwrap();
    let env_path = dir.path().join("env.json");
    std::fs::write(&env_path, &imported.stdout).unwrap();
    let env: Value = serde_json::from_slice(&imported.stdout).unwrap();
    assert_eq!(
        env[0]["bundle"]["network"], "testnet",
        "fixture precondition: the envelope itself says testnet"
    );

    let o = mn(&[
        "bundle",
        "--network",
        "mainnet",
        "--import-json",
        env_path.to_str().unwrap(),
        "--json",
    ]);
    if o.status.success() {
        let v = json(&o);
        assert_eq!(
            v["network"], "testnet",
            "bundle --import-json of a testnet envelope exited 0 but labels the bundle {}",
            v["network"]
        );
    }
}

// ===========================================================================
// #46 -- `bundle --import-json` silently drops every `--slot` overlay except
// `@N.phrase=`: the slot stays watch-only and the command exits 0.
//
// Correct behaviour is open (apply the overlay, or refuse); the test asserts
// only that a silent drop (exit 0, slot still watch-only) is absent.
// ===========================================================================

fn skip_middle_3of3_envelope() -> String {
    use miniscript::descriptor::checksum::Engine as ChecksumEngine;
    let body = "wsh(sortedmulti(2,\
[73c5da0a/87'/0'/0']xpub6DBjiYnc4ewKti13Q1L35bqdodw5z3VGJnf516B3icHrEGEUcCuCG5GVQDZtH8Xmsyt3Fs9YDNwLaqjUbbRidwXZ6sxufZcr4VqqzrXvicM/<0;1>/*,\
[b8688df1/87'/0'/0']xpub6CbhrPzY2z7NcCGCGjLAJLq8iRyjUfwmdXQs66MxTVUReKqb9DpLnVJ5D1qpatZjUuPGTyxf5TYU1vA34YFE9FHB4TvfYmokYLVsyEFZFt9/<0;1>/*,\
[28645006/87'/0'/0']xpub6DB7HNqw6CZojxN85NuFTPWZhi2FagSnexPS1rv3nYQhngkmdHgb7iebYvTFmFKKDA3ozf5yezDsCH6cXAw3WZijviSZtZC2hjHn2uazz4z/<0;1>/*))";
    let mut e = ChecksumEngine::new();
    e.input(body).expect("ascii");
    let blob = format!("BSMS 1.0\n{body}#{}\n", e.checksum());
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("--allow-argv-secret")
        .args(["import-wallet", "--blob", "-", "--format", "bsms", "--json"])
        .write_stdin(blob)
        .output()
        .unwrap();
    assert!(out.status.success(), "import-wallet: {}", err_s(&out));
    String::from_utf8(out.stdout).unwrap()
}

#[test]
fn bughunt_tk_46_import_json_slot_overlays_other_than_phrase_not_silently_dropped() {
    let dir = tempfile::tempdir().unwrap();
    let env_path = dir.path().join("env.json");
    std::fs::write(&env_path, skip_middle_3of3_envelope()).unwrap();
    let mut failures: Vec<String> = Vec::new();
    for (label, slot) in [
        ("entropy", format!("@0.entropy={ZERO_ENTROPY_HEX}")),
        ("ms1", format!("@0.ms1={ZERO_ENTROPY_MS1}")),
    ] {
        let o = mn(&[
            "bundle",
            "--network",
            "mainnet",
            "--import-json",
            env_path.to_str().unwrap(),
            "--slot",
            &slot,
            "--json",
        ]);
        if o.status.success() {
            let v = json(&o);
            let ms1_0 = v["ms1"][0].as_str().unwrap_or("<missing>");
            if ms1_0.is_empty() {
                failures.push(format!(
                    "--slot @0.{label}=...: exit 0 but ms1[0] is the empty watch-only sentinel \
                     (mode {})",
                    v["mode"]
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "a --slot overlay was silently dropped by bundle --import-json:\n{}",
        failures.join("\n")
    );
}

// ===========================================================================
// #52 -- the mk1 template-stub check ignores an unparseable `--mk1`
// (`filter_map(chunk_set_id_extract)` drops cards whose csi cannot be read).
//
// Flow copied from cli_verify_bundle_md1_template_multisig.rs
// `verify_bundle_canonical_multisig_template_id_search_ok`.
// ===========================================================================

const SEED_A: &str = "legal winner thank year wave sausage worth useful legal winner thank yellow";
const SEED_B: &str =
    "letter advice cage absurd amount doctor acoustic avoid letter advice cage above";

fn xpub_at(phrase: &str, path_str: &str) -> (bitcoin::bip32::Xpub, String) {
    use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
    use bitcoin::secp256k1::Secp256k1;
    use std::str::FromStr;
    let secp = Secp256k1::new();
    let m = bip39::Mnemonic::parse_in(bip39::Language::English, phrase).unwrap();
    let seed = m.to_seed("");
    let master = Xpriv::new_master(bitcoin::NetworkKind::Main, &seed).unwrap();
    let fp = master.fingerprint(&secp);
    let path = DerivationPath::from_str(path_str).unwrap();
    let xpriv = master.derive_priv(&secp, &path).unwrap();
    (
        Xpub::from_priv(&secp, &xpriv),
        fp.to_string().to_lowercase(),
    )
}

fn bundle_args(form: &str) -> Vec<String> {
    let path = "48'/0'/0'/2'";
    let mut args: Vec<String> = [
        "bundle",
        "--network",
        "mainnet",
        "--template",
        "wsh-sortedmulti",
        "--threshold",
        "2",
        "--md1-form",
        form,
        "--group-size",
        "0",
        "--no-engraving-card",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    for (idx, phrase) in [SEED_A, SEED_B].iter().enumerate() {
        let (xpub, fp) = xpub_at(phrase, path);
        args.push("--slot".into());
        args.push(format!("@{idx}.xpub={xpub}"));
        args.push("--slot".into());
        args.push(format!("@{idx}.fingerprint={fp}"));
        args.push("--slot".into());
        args.push(format!("@{idx}.path={path}"));
    }
    args
}

fn run_args(args: &[String]) -> Output {
    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
    mn(&refs)
}

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

#[test]
fn bughunt_tk_52_template_stub_check_not_passed_by_garbage_mk1() {
    let tmpl = run_args(&bundle_args("template"));
    assert!(tmpl.status.success(), "template bundle: {}", err_s(&tmpl));
    let tmpl_out = out_s(&tmpl);
    let md1 = section_lines(&tmpl_out, "# md1");
    let stubs = mk1_groups(&tmpl_out);
    let wallet_id = err_s(&tmpl)
        .lines()
        .find(|l| l.contains("wallet-id (hex)"))
        .expect("wallet-id (hex) line")
        .split(':')
        .next_back()
        .unwrap()
        .trim()
        .to_string();
    let pol = run_args(&bundle_args("policy"));
    assert!(pol.status.success(), "policy bundle: {}", err_s(&pol));
    let cosigner_b = mk1_groups(&out_s(&pol))[1].clone();

    let verify = |extra_mk1: Option<&str>| -> Output {
        let mut args: Vec<String> =
            vec!["verify-bundle".into(), "--network".into(), "mainnet".into()];
        for c in &md1 {
            args.push("--md1".into());
            args.push(c.clone());
        }
        for g in &stubs {
            for c in g {
                args.push("--mk1".into());
                args.push(c.clone());
            }
        }
        if let Some(x) = extra_mk1 {
            args.push("--mk1".into());
            args.push(x.to_string());
        }
        args.extend([
            "--from".into(),
            format!("phrase={SEED_A}"),
            "--account".into(),
            "0".into(),
            "--expect-wallet-id".into(),
            wallet_id.clone(),
            "--json".into(),
        ]);
        for c in &cosigner_b {
            args.push("--cosigner".into());
            args.push(c.clone());
        }
        run_args(&args)
    };

    // Control: the genuine stubs bind and the run is OK.
    let ok = verify(None);
    assert!(ok.status.success(), "control run failed: {}", err_s(&ok));
    assert_eq!(
        json(&ok)["result"],
        "ok",
        "control run must be ok: {}",
        out_s(&ok)
    );

    // Experiment: same cards plus one garbage `--mk1`.
    let bad = verify(Some(
        "mk1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq",
    ));
    let passed_ok = bad.status.success() && json(&bad)["result"] == "ok";
    assert!(
        !passed_ok,
        "verify-bundle reports result=ok with an unparseable extra --mk1 card; the \
         mk1_template_stub_bind check must fail (or the run must refuse). stdout:\n{}",
        out_s(&bad)
    );
}

// ===========================================================================
// #54 -- `bundle --json` multisig `path_family` is always "bip87" in
// descriptor mode, even for a BIP-48 descriptor.
//
// Correct value is open (`bip48` derived from the origin, per the report, or
// the literal "descriptor" per SPEC v0.3 s5.3); the test asserts only that the
// wrong label "bip87" is absent for a `48'` origin.
// ===========================================================================

#[test]
fn bughunt_tk_54_descriptor_mode_path_family_not_bip87_for_bip48_origin() {
    let desc = format!(
        "wsh(sortedmulti(2,[{FP_A}/48'/0'/0'/2']@0/<0;1>/*,[{FP_B}/48'/0'/0'/2']@1/<0;1>/*))"
    );
    let a = format!("@0.xpub={XPUB_A}");
    let b = format!("@1.xpub={XPUB_B}");
    let o = mn(&[
        "bundle",
        "--network",
        "mainnet",
        "--descriptor",
        &desc,
        "--slot",
        &a,
        "--slot",
        &b,
        "--json",
    ]);
    assert!(o.status.success(), "bundle: {}", err_s(&o));
    let v = json(&o);
    assert_eq!(
        v["multisig"]["cosigners"][0]["origin_path"], "m/48'/0'/0'/2'",
        "fixture precondition: BIP-48 origin"
    );
    assert_ne!(
        v["multisig"]["path_family"], "bip87",
        "a BIP-48 (m/48'/..) descriptor is reported with path_family \"bip87\""
    );
}
