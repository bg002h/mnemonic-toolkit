//! H7 (cycle-2) — BIP-380-canonical PREFIX-form `[fp/path]@N` key-origin
//! annotation is ACCEPTED (was silently ignored, dropping the origin AND
//! bypassing the per-`@N` master-fingerprint cross-check).
//!
//! BIP-380 defines key-origin as a PREFIX `[fingerprint/path]KEY`. The toolkit
//! historically accepted ONLY the non-canonical suffix form `@N[fp/path]`, so a
//! user/tool following the standard who wrote `[fp/path]@N` had their origin
//! path silently dropped (the slot xpub was built at the default/master path →
//! the backup watches a DIFFERENT address set) AND the funds-safety master-fp
//! cross-check was bypassed. H7 ACCEPTs the prefix form (converting
//! `lex_placeholders` to all-NAMED capture groups so the H13 hardened-multipath
//! validator can never be shifted) and adds the explicit xpub-slot
//! `prefix-anno-fp vs --slot @N.fingerprint=` equality check.
//!
//! These tests prove (CLI-level, exit-code + byte-identity):
//!   (a) a WRONG prefix fp now FIRES the master-fp cross-check (exit ≠ 0);
//!       a correct prefix fp still succeeds and carries the origin;
//!   (b) prefix ≡ suffix → byte-identical md1/mk1 for the same wallet;
//!   (c) prefix-fp vs `--slot @N.fingerprint=` on an xpub slot: mismatch
//!       refuses, agreement succeeds;
//!   (d) both-positions (prefix AND suffix bracket on the same `@N`) refuses.

use assert_cmd::Command;
use serde_json::Value;

// All-zero-entropy 12-word seed (its mainnet/regtest master fingerprint is
// `73c5da0a`, read from the binary's own `master_fingerprint` JSON field — not
// hard-coded as a load-bearing literal).
const SEED_A: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
// A wrong fingerprint that does NOT match SEED_A's master fp.
const WRONG_FP: &str = "deadbeef";

/// Run `bundle --network regtest --account 0 --no-engraving-card <extra...>` and
/// return (exit-success?, stdout, stderr).
fn run_bundle(args: &[&str]) -> (bool, String, String) {
    let mut full: Vec<String> = vec![
        "bundle".into(),
        "--network".into(),
        "regtest".into(),
        "--account".into(),
        "0".into(),
        "--no-engraving-card".into(),
    ];
    for a in args {
        full.push((*a).to_string());
    }
    let assert = Command::cargo_bin("mnemonic").unwrap().args(&full).assert();
    let out = assert.get_output();
    let success = out.status.success();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.stderr.clone()).unwrap();
    (success, stdout, stderr)
}

/// Run a successful phrase-slot bundle and return its parsed `--json`.
fn phrase_bundle_json(descriptor: &str) -> Value {
    let (ok, stdout, stderr) = run_bundle(&[
        "--descriptor",
        descriptor,
        "--slot",
        &format!("@0.phrase={SEED_A}"),
        "--json",
    ]);
    assert!(ok, "phrase bundle must succeed; stderr:\n{stderr}");
    serde_json::from_str(&stdout).expect("bundle JSON valid")
}

/// SEED_A's real master fingerprint, read from the binary (compute, don't
/// hard-code the load-bearing value).
fn real_master_fp() -> String {
    let v = phrase_bundle_json("wpkh(@0[73c5da0a/84'/0'/0']/<0;1>/*)");
    v["master_fingerprint"]
        .as_str()
        .expect("master_fingerprint present")
        .to_string()
}

/// (a) A WRONG prefix-form fingerprint must FIRE the master-fp cross-check
/// (exit ≠ 0) — identical to the suffix form. Before H7 the prefix form exited
/// 0 (origin dropped, cross-check bypassed).
#[test]
fn prefix_wrong_fingerprint_fires_master_fp_cross_check() {
    let desc = format!("wpkh([{WRONG_FP}/84'/0'/0']@0/<0;1>/*)");
    let (success, _stdout, stderr) = run_bundle(&[
        "--descriptor",
        &desc,
        "--slot",
        &format!("@0.phrase={SEED_A}"),
        "--json",
    ]);
    assert!(
        !success,
        "a WRONG prefix-form fingerprint must FIRE the master-fp cross-check (exit != 0); stderr:\n{stderr}"
    );
    assert!(
        stderr.contains("master fingerprint") && stderr.contains(WRONG_FP),
        "stderr must name the fp-mismatch and the wrong annotation; got:\n{stderr}"
    );

    // Suffix-form oracle: same wrong fp, suffix position → same refusal.
    let suffix_desc = format!("wpkh(@0[{WRONG_FP}/84'/0'/0']/<0;1>/*)");
    let (suffix_success, _, suffix_stderr) = run_bundle(&[
        "--descriptor",
        &suffix_desc,
        "--slot",
        &format!("@0.phrase={SEED_A}"),
        "--json",
    ]);
    assert!(
        !suffix_success,
        "suffix-form wrong fp must also refuse; got stderr:\n{suffix_stderr}"
    );
}

/// (a, cont.) A CORRECT prefix-form fingerprint must SUCCEED (exit 0) and carry
/// the origin path through to the emitted bundle (it was dropped before H7).
#[test]
fn prefix_correct_fingerprint_succeeds_and_carries_origin() {
    let real_fp = real_master_fp();
    let desc = format!("wpkh([{real_fp}/84'/0'/0']@0/<0;1>/*)");
    let v = phrase_bundle_json(&desc);
    assert_eq!(
        v["origin_path"].as_str(),
        Some("m/84'/0'/0'"),
        "the prefix-form origin path must be carried into the bundle (not dropped)"
    );
}

/// (b) prefix ≡ suffix → byte-identical md1 + mk1 for the same wallet.
#[test]
fn prefix_equals_suffix_byte_identical_md1_mk1() {
    let real_fp = real_master_fp();
    let prefix_desc = format!("wpkh([{real_fp}/84'/0'/0']@0/<0;1>/*)");
    let suffix_desc = format!("wpkh(@0[{real_fp}/84'/0'/0']/<0;1>/*)");

    let pv = phrase_bundle_json(&prefix_desc);
    let sv = phrase_bundle_json(&suffix_desc);
    assert_eq!(
        pv["md1"], sv["md1"],
        "prefix-form md1 must be byte-identical to the suffix-form md1"
    );
    assert_eq!(
        pv["mk1"], sv["mk1"],
        "prefix-form mk1 must be byte-identical to the suffix-form mk1"
    );
    // And both must agree on the origin (not the dropped default).
    assert_eq!(pv["origin_path"], sv["origin_path"]);
    assert_eq!(pv["origin_path"].as_str(), Some("m/84'/0'/0'"));
}

/// (c) xpub-slot composition: a prefix-fp annotation that DISAGREES with an
/// explicit `--slot @N.fingerprint=` must REFUSE (the ADDED equality check at the
/// xpub-slot `.or(anno_fp)` site); the agreeing case must SUCCEED. Before H7 the
/// prefix-anno fp was silently overridden by the `--slot` value (no equality
/// check on the xpub-slot arm).
#[test]
fn prefix_fp_vs_slot_fingerprint_mismatch_refuses_agreement_ok() {
    let real_fp = real_master_fp();

    // Recover the account-level xpub for SEED_A @ m/84'/0'/0' by decoding the
    // mk1 card (the `mk1` JSON array is the chunked form of ONE card — decode all
    // chunks together) of a phrase-slot bundle.
    let v = phrase_bundle_json(&format!("wpkh([{real_fp}/84'/0'/0']@0/<0;1>/*)"));
    let mk1_chunks: Vec<String> = v["mk1"]
        .as_array()
        .expect("mk1 array")
        .iter()
        .map(|s| s.as_str().expect("mk1 chunk str").to_string())
        .collect();
    let mk1_refs: Vec<&str> = mk1_chunks.iter().map(|s| s.as_str()).collect();
    let card = mk_codec::decode(&mk1_refs).expect("mk1 decodes");
    let xpub = card.xpub.to_string();

    let desc = format!("wpkh([{real_fp}/84'/0'/0']@0/<0;1>/*)");

    // MISMATCH: prefix fp = real_fp vs explicit --slot @0.fingerprint=WRONG_FP.
    let (mm_ok, _mm_out, mm_err) = run_bundle(&[
        "--descriptor",
        &desc,
        "--slot",
        &format!("@0.xpub={xpub}"),
        "--slot",
        &format!("@0.fingerprint={WRONG_FP}"),
        "--json",
    ]);
    assert!(
        !mm_ok,
        "prefix-fp vs --slot @0.fingerprint= mismatch must REFUSE; stderr:\n{mm_err}"
    );

    // AGREEMENT: prefix fp = real_fp and --slot @0.fingerprint= real_fp → ok.
    let (ag_ok, _ag_out, ag_err) = run_bundle(&[
        "--descriptor",
        &desc,
        "--slot",
        &format!("@0.xpub={xpub}"),
        "--slot",
        &format!("@0.fingerprint={real_fp}"),
        "--json",
    ]);
    assert!(
        ag_ok,
        "prefix-fp == --slot @0.fingerprint= agreement must SUCCEED; stderr:\n{ag_err}"
    );
}

/// verify-bundle shares `lex_placeholders`, so the prefix-form origin must carry
/// through its reparse: a bundle generated from the prefix-form descriptor must
/// verify OK when re-presented with the SAME prefix-form descriptor (and the
/// origin path is honored, not dropped). This pins the shared-lexer inheritance
/// (verify-bundle has no compensating per-`@N` guard, so the lexer fix is its
/// only protection).
#[test]
fn verify_bundle_carries_prefix_origin_through_reparse() {
    let real_fp = real_master_fp();
    let prefix_desc = format!("wpkh([{real_fp}/84'/0'/0']@0/<0;1>/*)");
    let v = phrase_bundle_json(&prefix_desc);

    let ms1 = v["ms1"][0].as_str().expect("ms1[0]").to_string();
    let mk1: Vec<String> = v["mk1"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect();
    let md1: Vec<String> = v["md1"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect();

    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--network".into(),
        "regtest".into(),
        "--account".into(),
        "0".into(),
        "--descriptor".into(),
        prefix_desc.clone(),
        "--slot".into(),
        format!("@0.phrase={SEED_A}"),
        "--ms1".into(),
        ms1,
    ];
    for s in &mk1 {
        args.push("--mk1".into());
        args.push(s.clone());
    }
    for s in &md1 {
        args.push("--md1".into());
        args.push(s.clone());
    }
    let arg_refs: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let assert = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&arg_refs)
        .assert();
    let out = assert.get_output();
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.stderr.clone()).unwrap();
    assert!(
        out.status.success(),
        "verify-bundle must accept the prefix-form descriptor reparse; stdout:\n{stdout}\nstderr:\n{stderr}"
    );
    assert!(
        stdout.contains("result: ok"),
        "verify-bundle of the prefix form must report ok; stdout:\n{stdout}"
    );
}

/// (d) Both-positions: a prefix AND a suffix origin bracket on the same `@N` is
/// an ambiguous double-origin → REFUSE.
#[test]
fn both_positions_origin_brackets_refused() {
    let desc = "wpkh([deadbeef/84'/0'/0']@0[cafef00d/84'/0'/0']/<0;1>/*)";
    let (success, _stdout, stderr) = run_bundle(&[
        "--descriptor",
        desc,
        "--slot",
        &format!("@0.phrase={SEED_A}"),
        "--json",
    ]);
    assert!(
        !success,
        "both-positions origin brackets must REFUSE; stderr:\n{stderr}"
    );
}
