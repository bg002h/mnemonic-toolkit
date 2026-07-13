//! #28 phase 1 — `mnemonic verify-bundle` on a keyless single-sig TEMPLATE
//! bundle: bind-via-template-id-stub + complete + recompose the watch-only
//! wallet, with `--expect-wallet-id`.

use assert_cmd::Command;

const PHRASE_A: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

fn mnemonic() -> Command {
    Command::cargo_bin("mnemonic").expect("mnemonic binary builds")
}

/// Emit a template bundle and return its (ms1, mk1, md1) unbroken card vectors.
fn template_cards(
    template: &str,
    phrase: &str,
    account: &str,
) -> (Vec<String>, Vec<String>, Vec<String>) {
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
    let mut ms1 = Vec::new();
    let mut mk1 = Vec::new();
    let mut md1 = Vec::new();
    let mut section = "";
    for line in stdout.lines() {
        if line.starts_with("# ms1") {
            section = "ms1";
            continue;
        }
        if line.starts_with("# mk1") {
            section = "mk1";
            continue;
        }
        if line.starts_with("# md1") {
            section = "md1";
            continue;
        }
        let t = line.trim();
        if t.is_empty() {
            section = "";
            continue;
        }
        match section {
            "ms1" => ms1.push(t.to_string()),
            "mk1" => mk1.push(t.to_string()),
            "md1" => md1.push(t.to_string()),
            _ => {}
        }
    }
    (ms1, mk1, md1)
}

fn verify_args(template: &str, phrase: &str, account: &str, expect: Option<&str>) -> Vec<String> {
    let (ms1, mk1, md1) = template_cards(template, phrase, account);
    let mut args = vec![
        "verify-bundle".to_string(),
        "--network".to_string(),
        "mainnet".to_string(),
        "--account".to_string(),
        account.to_string(),
        "--slot".to_string(),
        format!("@0.phrase={phrase}"),
    ];
    for m in &ms1 {
        if !m.is_empty() {
            args.push("--ms1".to_string());
            args.push(m.clone());
        }
    }
    for m in &mk1 {
        args.push("--mk1".to_string());
        args.push(m.clone());
    }
    for m in &md1 {
        args.push("--md1".to_string());
        args.push(m.clone());
    }
    if let Some(e) = expect {
        args.push("--expect-wallet-id".to_string());
        args.push(e.to_string());
    }
    args
}

/// Decode a chunk-form md1 set and RE-ENCODE it as a single NON-chunked md1
/// string (the bare `md encode` form). Defined per-file (integration test files
/// are separate crates — helpers cannot be shared).
fn to_nonchunked(chunk_form_md1: &[String]) -> String {
    let refs: Vec<&str> = chunk_form_md1.iter().map(String::as_str).collect();
    let d = md_codec::chunk::reassemble(&refs).expect("chunk-form md1 decodes");
    md_codec::encode_md1_string(&d).expect("re-encode as a single non-chunked md1")
}

/// Emit a KEYED (wallet-policy) bundle and return its (ms1, mk1, md1) cards.
/// Same as `template_cards` but WITHOUT `--md1-form template`, so the md1 is a
/// keyed policy card — naturally MULTI-chunk (a 65-byte pubkey = 520 bits > the
/// 400-bit single-string cap), exercising the classify `_ => reassemble` arm.
fn keyed_cards(
    template: &str,
    phrase: &str,
    account: &str,
) -> (Vec<String>, Vec<String>, Vec<String>) {
    let out = mnemonic()
        .args([
            "bundle",
            "--template",
            template,
            "--network",
            "mainnet",
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
    let (mut ms1, mut mk1, mut md1) = (Vec::new(), Vec::new(), Vec::new());
    let mut section = "";
    for line in stdout.lines() {
        if line.starts_with("# ms1") {
            section = "ms1";
            continue;
        }
        if line.starts_with("# mk1") {
            section = "mk1";
            continue;
        }
        if line.starts_with("# md1") {
            section = "md1";
            continue;
        }
        let t = line.trim();
        if t.is_empty() {
            section = "";
            continue;
        }
        match section {
            "ms1" => ms1.push(t.into()),
            "mk1" => mk1.push(t.into()),
            "md1" => md1.push(t.into()),
            _ => {}
        }
    }
    (ms1, mk1, md1)
}

#[test]
fn verify_bundle_keyed_multichunk_unchanged() {
    // A KEYED bip84 bundle md1 is multi-chunk. It enters the classify `match`'s
    // `_ => reassemble` arm (len>1, verbatim-unchanged by Facet 1), skips both
    // template branches (is_wallet_policy=true), and verifies via the general
    // path — GREEN before AND after Facet 1.
    let (ms1, mk1, md1) = keyed_cards("bip84", PHRASE_A, "0");
    assert!(md1.len() > 1, "keyed bip84 md1 must be multi-chunk: {md1:?}");
    // A KEYED wallet-policy md1 REQUIRES --template: it skips the keyless-template
    // short-circuit → verify_bundle.rs:435-443 ModeViolation without it (planr0 I-A).
    // The general/keyed path prints lowercase "result: ok" (:558-567), NOT the
    // template-path-only "OK (…recomposed)" string (:824). Mirror the proven keyed
    // verify pattern in cli_verify_bundle_full.rs:30-56.
    let mut args = vec![
        "verify-bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "bip84".into(),
        "--account".into(),
        "0".into(),
        "--slot".into(),
        format!("@0.phrase={PHRASE_A}"),
    ];
    for m in ms1.iter().filter(|m| !m.is_empty()) {
        args.push("--ms1".into());
        args.push(m.clone());
    }
    for m in &mk1 {
        args.push("--mk1".into());
        args.push(m.clone());
    }
    for m in &md1 {
        args.push("--md1".into());
        args.push(m.clone());
    }
    let out = mnemonic().args(&args).assert().success();
    assert!(String::from_utf8(out.get_output().stdout.clone())
        .unwrap()
        .contains("result: ok"));
}

#[test]
fn verify_bundle_nonchunked_dead_card_falls_through_strict() {
    // A NON-chunked KEYLESS DEAD card: take the frozen keyed wsh(pk) card
    // (`m/48'/0'/0'`, a NON-canonical wrapper), strip its keys → keyless template,
    // elide the origin → unresolvable. Keyless → re-encodes as a single non-chunked
    // string (mirrors the `dead()` helper in cli_repair_dead_card_strict.rs:32-36,
    // minus keys, plus encode_md1_string). Strict decode_md1_string rejects the
    // elided-unresolvable origin (MissingExplicitOrigin) → classify falls THROUGH →
    // never "OK" (SPEC INV-5), before AND after Facet 1.
    const SS_MD1_ORIGIN: &[&str] = &[
        "md1f9xlxpqpqpmvyyyqqcy2pdqhp5gmug4gy80cpxatjnpdtxhjvyuds54ar44wuc0a34",
        "md1f9xlxpq036ekkrhtkv6grq7qcua7ej7xusqaaq2qptxulyg808qnqjq8s570kd4kkd",
        "md1f9xlxpqsz3h36nf43a3dytlcf6saj9lwz9gc9uag7ce95hlcqu95t5qpd0qs94",
    ];
    let mut d = md_codec::chunk::reassemble(SS_MD1_ORIGIN).expect("decode keyed wsh(pk) card");
    d.tlv.pubkeys = None; // → keyless template (fits a single non-chunked string)
    d.tlv.fingerprints = None;
    d.path_decl.paths =
        md_codec::PathDeclPaths::Shared(md_codec::OriginPath { components: vec![] }); // elide → dead
    let dead_single =
        md_codec::encode_md1_string(&d).expect("re-encode keyless dead card non-chunked");
    // --mk1 is clap-required alongside --md1 (verify_bundle.rs:183); supply a real
    // template mk1 so the invocation PASSES clap and actually REACHES the classify
    // gate (without it, clap rejects at exit 64 and the strict-classify path is
    // never exercised — a vacuous lock). The dead md1 strict-decode-fails so it
    // falls THROUGH classify → "--template is required" (exit 2), never OK, before
    // AND after Facet 1. The mk1 content is irrelevant: the fall-through refusal
    // fires before mk1 is examined.
    let (_ms1, mk1, _md1) = template_cards("bip84", PHRASE_A, "0");
    let mut args = vec![
        "verify-bundle".to_string(),
        "--network".to_string(),
        "mainnet".to_string(),
        "--md1".to_string(),
        dead_single,
    ];
    for m in &mk1 {
        args.push("--mk1".to_string());
        args.push(m.clone());
    }
    let assert = mnemonic().args(&args).assert().failure();
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(
        !stdout.contains("OK"),
        "non-chunked dead card must never verify OK: {stdout}"
    );
}

#[test]
fn verify_template_bundle_recomposes_and_passes() {
    for template in ["bip44", "bip84", "bip86"] {
        let out = mnemonic()
            .args(verify_args(template, PHRASE_A, "0", None))
            .assert()
            .success();
        let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
        assert!(
            stdout.contains("OK") && stdout.contains("descriptor:"),
            "{template}: verify must recompose + report OK: {stdout}"
        );
    }
}

#[test]
fn verify_template_bundle_without_seed_is_refused() {
    let (_ms1, mk1, md1) = template_cards("bip84", PHRASE_A, "0");
    let mut args = vec![
        "verify-bundle".to_string(),
        "--network".to_string(),
        "mainnet".to_string(),
    ];
    for m in &mk1 {
        args.push("--mk1".to_string());
        args.push(m.clone());
    }
    for m in &md1 {
        args.push("--md1".to_string());
        args.push(m.clone());
    }
    // No --slot seed → refused (the template is keyless; nothing to recompose).
    let assert = mnemonic().args(&args).assert().failure();
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("seed") || stderr.contains("--slot"),
        "missing-seed refusal must name the seed requirement: {stderr}"
    );
}

#[test]
fn verify_template_bundle_expect_wallet_id_wrong_mismatch() {
    mnemonic()
        .args(verify_args(
            "bip84",
            PHRASE_A,
            "0",
            Some("deadbeefdeadbeef"),
        ))
        .assert()
        .failure()
        .code(4);
}

#[test]
fn verify_template_bundle_recomposes_under_origin_matches_restore() {
    // R0 I2 — verify-bundle's template completion supports `--origin` (mirroring
    // restore's `--origin`). A custom-origin template recomposes to the SAME
    // descriptor verify-bundle and restore each build at that origin, and the
    // verdict is OK (no spurious mismatch from a canonical-account fallback).
    //
    // The bundle is emitted at `--account 7`, whose CANONICAL origin is exactly
    // `m/84'/0'/7'`, so the explicit `--origin m/84'/0'/7'` re-derives the SAME
    // xpub → the supplied mk1 binds and the verdict is OK.
    const ORIGIN: &str = "m/84'/0'/7'";
    let (ms1, mk1, md1) = template_cards("bip84", PHRASE_A, "7");

    let mut vargs = vec![
        "verify-bundle".to_string(),
        "--network".to_string(),
        "mainnet".to_string(),
        "--origin".to_string(),
        ORIGIN.to_string(),
        "--slot".to_string(),
        format!("@0.phrase={PHRASE_A}"),
    ];
    for m in &ms1 {
        if !m.is_empty() {
            vargs.push("--ms1".to_string());
            vargs.push(m.clone());
        }
    }
    for m in &mk1 {
        vargs.push("--mk1".to_string());
        vargs.push(m.clone());
    }
    for m in &md1 {
        vargs.push("--md1".to_string());
        vargs.push(m.clone());
    }
    let out_v = mnemonic().args(&vargs).assert().success();
    let v_stdout = String::from_utf8(out_v.get_output().stdout.clone()).unwrap();
    assert!(
        v_stdout.contains("OK"),
        "verify under --origin must report OK: {v_stdout}"
    );
    let v_desc = v_stdout
        .lines()
        .find_map(|l| l.trim().strip_prefix("descriptor:"))
        .map(|s| s.trim().to_string())
        .expect("verify --origin descriptor");

    // The same template restored at the same --origin must yield the same desc.
    let mut rargs = vec![
        "restore".to_string(),
        "--from".to_string(),
        format!("phrase={PHRASE_A}"),
        "--network".to_string(),
        "mainnet".to_string(),
        "--origin".to_string(),
        ORIGIN.to_string(),
    ];
    for m in &md1 {
        rargs.push("--md1".to_string());
        rargs.push(m.clone());
    }
    let out_r = mnemonic().args(&rargs).assert().success();
    let r_stdout = String::from_utf8(out_r.get_output().stdout.clone()).unwrap();
    let r_desc = r_stdout
        .lines()
        .find_map(|l| l.trim().strip_prefix("descriptor:"))
        .map(|s| s.trim().to_string())
        .expect("restore --origin descriptor");

    assert_eq!(
        v_desc, r_desc,
        "verify-bundle --origin recompose must equal restore --origin completion"
    );
    // The custom origin must actually appear in the descriptor (proof it was used).
    assert!(
        v_desc.contains("/84'/0'/7'") || v_desc.contains("84h/0h/7h"),
        "recomposed descriptor must carry the --origin path: {v_desc}"
    );
}

#[test]
fn verify_template_bundle_expect_wallet_id_skipped_under_origin() {
    // R0 I2 — `--expect-wallet-id` is NOT checked under `--origin` (the wallet-id
    // was computed for the canonical origin; an override is a different preimage).
    // A deliberately-wrong id must NOT cause a failure when --origin is present.
    // Bundle at account 7 so the cards bind at the explicit `m/84'/0'/7'` origin.
    let (ms1, mk1, md1) = template_cards("bip84", PHRASE_A, "7");
    let mut args = vec![
        "verify-bundle".to_string(),
        "--network".to_string(),
        "mainnet".to_string(),
        "--origin".to_string(),
        "m/84'/0'/7'".to_string(),
        "--expect-wallet-id".to_string(),
        "deadbeefdeadbeef".to_string(),
        "--slot".to_string(),
        format!("@0.phrase={PHRASE_A}"),
    ];
    for m in &ms1 {
        if !m.is_empty() {
            args.push("--ms1".to_string());
            args.push(m.clone());
        }
    }
    for m in &mk1 {
        args.push("--mk1".to_string());
        args.push(m.clone());
    }
    for m in &md1 {
        args.push("--md1".to_string());
        args.push(m.clone());
    }
    let out = mnemonic().args(&args).assert().success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("--expect-wallet-id is not checked") || stderr.contains("--origin"),
        "skip-under-origin must emit a notice: {stderr}"
    );
}

#[test]
fn verify_template_bundle_recompose_matches_restore() {
    // The recomposed descriptor from verify-bundle must equal the restore
    // template-completion descriptor (cross-tool consistency).
    let out_v = mnemonic()
        .args(verify_args("bip84", PHRASE_A, "0", None))
        .assert()
        .success();
    let v_stdout = String::from_utf8(out_v.get_output().stdout.clone()).unwrap();
    let v_desc = v_stdout
        .lines()
        .find_map(|l| l.trim().strip_prefix("descriptor:"))
        .map(|s| s.trim().to_string())
        .expect("verify descriptor");

    let (_ms1, _mk1, md1) = template_cards("bip84", PHRASE_A, "0");
    let mut rargs = vec![
        "restore".to_string(),
        "--from".to_string(),
        format!("phrase={PHRASE_A}"),
        "--network".to_string(),
        "mainnet".to_string(),
    ];
    for m in &md1 {
        rargs.push("--md1".to_string());
        rargs.push(m.clone());
    }
    let out_r = mnemonic().args(&rargs).assert().success();
    let r_stdout = String::from_utf8(out_r.get_output().stdout.clone()).unwrap();
    let r_desc = r_stdout
        .lines()
        .find_map(|l| l.trim().strip_prefix("descriptor:"))
        .map(|s| s.trim().to_string())
        .expect("restore descriptor");

    assert_eq!(
        v_desc, r_desc,
        "verify recompose must equal restore completion"
    );
}

/// Like `verify_args`, but supply the md1 as a single NON-chunked string.
fn verify_args_nonchunked(template: &str, phrase: &str, account: &str) -> Vec<String> {
    let (ms1, mk1, md1) = template_cards(template, phrase, account);
    let single = to_nonchunked(&md1);
    let mut args = vec![
        "verify-bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--account".into(),
        account.into(),
        "--slot".into(),
        format!("@0.phrase={phrase}"),
    ];
    for m in ms1.iter().filter(|m| !m.is_empty()) {
        args.push("--ms1".into());
        args.push(m.clone());
    }
    for m in &mk1 {
        args.push("--mk1".into());
        args.push(m.clone());
    } // REQUIRED (SPEC M-2)
    args.push("--md1".into());
    args.push(single);
    args
}

#[test]
fn verify_bundle_nonchunked_singlesig_template_ok() {
    for template in ["bip44", "bip84", "bip86"] {
        let out = mnemonic()
            .args(verify_args_nonchunked(template, PHRASE_A, "0"))
            .assert()
            .success();
        let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
        assert!(
            stdout.contains("OK"),
            "{template} non-chunked must verify OK: {stdout}"
        );
    }
}

#[test]
fn verify_bundle_chunked_template_still_ok() {
    // No-regression (INV-3): byte-compare pass ⟹ same descriptor ⟹ id-compare pass.
    let out = mnemonic()
        .args(verify_args("bip84", PHRASE_A, "0", None))
        .assert()
        .success();
    assert!(String::from_utf8(out.get_output().stdout.clone())
        .unwrap()
        .contains("OK"));
}

#[test]
fn verify_bundle_nonchunked_singlesig_json_ok() {
    let mut args = verify_args_nonchunked("bip84", PHRASE_A, "0");
    args.push("--json".into());
    let out = mnemonic().args(&args).assert().success();
    let v: serde_json::Value =
        serde_json::from_slice(&out.get_output().stdout).expect("json");
    assert_eq!(v["result"], "ok");
    assert_eq!(v["mode"], "single-sig-template");
    let md1c = v["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "md1_template_match")
        .expect("md1_template_match check");
    assert_eq!(md1c["passed"], true);
}

#[test]
fn verify_bundle_form_equivalence_same_verdict() {
    // SPEC §6.2 #5: the SAME descriptor as chunk-form vs non-chunked yields the
    // identical verdict — stdout AND stderr (the ✓/✗ check lines print to stderr,
    // verify_bundle.rs:811-820) AND the --json shape (planr0 M-1).
    let chunked = mnemonic()
        .args(verify_args("bip84", PHRASE_A, "0", None))
        .assert()
        .success();
    let nonchunked = mnemonic()
        .args(verify_args_nonchunked("bip84", PHRASE_A, "0"))
        .assert()
        .success();
    assert_eq!(
        String::from_utf8(chunked.get_output().stdout.clone()).unwrap(),
        String::from_utf8(nonchunked.get_output().stdout.clone()).unwrap(),
        "stdout"
    );
    assert_eq!(
        String::from_utf8(chunked.get_output().stderr.clone()).unwrap(),
        String::from_utf8(nonchunked.get_output().stderr.clone()).unwrap(),
        "stderr checks"
    );
    let mut cj = verify_args("bip84", PHRASE_A, "0", None);
    cj.push("--json".into());
    let mut nj = verify_args_nonchunked("bip84", PHRASE_A, "0");
    nj.push("--json".into());
    let cjv: serde_json::Value =
        serde_json::from_slice(&mnemonic().args(&cj).assert().success().get_output().stdout)
            .unwrap();
    let njv: serde_json::Value =
        serde_json::from_slice(&mnemonic().args(&nj).assert().success().get_output().stdout)
            .unwrap();
    assert_eq!(cjv, njv, "--json shape must be identical across forms");
}

#[test]
fn verify_bundle_nonchunked_noncanonical_encoding_mismatch() {
    // PROBATIVE INV-4 anchor (SPEC §6.3 #7, construction b): inject a Fingerprints
    // TLV the template synthesis never carries. Same (tag,body) → still classifies
    // single-sig + re-derives the SAME (fingerprint-less) expected → encoding-id
    // DIFFERS → md1_template_match FALSE. Stays GREEN only if the compare is
    // content-sensitive (a broken md1_match=true regression FAILS it). Fingerprints
    // are WDT-id-EXCLUDED, so this also proves encoding-id > WDT-id.
    let (ms1, mk1, md1) = template_cards("bip84", PHRASE_A, "0");
    let refs: Vec<&str> = md1.iter().map(String::as_str).collect();
    let mut d = md_codec::chunk::reassemble(&refs).unwrap();
    d.tlv.fingerprints = Some(vec![(0u8, [0xABu8; 4])]);
    let doctored = md_codec::encode_md1_string(&d).unwrap();
    let mut args = vec![
        "verify-bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--account".into(),
        "0".into(),
        "--slot".into(),
        format!("@0.phrase={PHRASE_A}"),
        "--json".into(),
    ];
    for m in ms1.iter().filter(|m| !m.is_empty()) {
        args.push("--ms1".into());
        args.push(m.clone());
    }
    for m in &mk1 {
        args.push("--mk1".into());
        args.push(m.clone());
    }
    args.push("--md1".into());
    args.push(doctored);
    let assert = mnemonic().args(&args).assert().code(4);
    let v: serde_json::Value = serde_json::from_slice(&assert.get_output().stdout).unwrap();
    let md1c = v["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["name"] == "md1_template_match")
        .unwrap();
    assert_eq!(
        md1c["passed"], false,
        "non-canonical encoding must mismatch: {v}"
    );
}

#[test]
fn verify_bundle_nonchunked_doctored_origin_stricter_than_wdt() {
    // SPEC §6.3 #8: an EXPLICIT (non-elided) canonical origin. encode_payload writes
    // path_decl verbatim → the explicit form's id differs from the elided expected's;
    // WDT-id EXCLUDES origin-path-decl so it would MATCH — proving encoding-id is
    // strictly stronger. Tree stays canonical → strict-decodes + classifies.
    let (ms1, mk1, md1) = template_cards("bip84", PHRASE_A, "0");
    let refs: Vec<&str> = md1.iter().map(String::as_str).collect();
    let mut d = md_codec::chunk::reassemble(&refs).unwrap();
    d.path_decl.paths = md_codec::PathDeclPaths::Shared(md_codec::OriginPath {
        components: vec![
            md_codec::PathComponent {
                hardened: true,
                value: 84,
            },
            md_codec::PathComponent {
                hardened: true,
                value: 0,
            },
            md_codec::PathComponent {
                hardened: true,
                value: 0,
            },
        ],
    });
    let doctored = md_codec::encode_md1_string(&d).unwrap();
    let mut args = vec![
        "verify-bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--account".into(),
        "0".into(),
        "--slot".into(),
        format!("@0.phrase={PHRASE_A}"),
    ];
    for m in ms1.iter().filter(|m| !m.is_empty()) {
        args.push("--ms1".into());
        args.push(m.clone());
    }
    for m in &mk1 {
        args.push("--mk1".into());
        args.push(m.clone());
    }
    args.push("--md1".into());
    args.push(doctored);
    mnemonic().args(&args).assert().code(4); // md1_template_match mismatch → exit 4
}

#[test]
fn verify_bundle_mk1_tolerance_not_extended() {
    // SPEC §6.4 #10: md1 form-tolerance is md1-ONLY. A matching non-chunked md1 with
    // a case-variant mk1 still mismatches (mk1_template_stub_bind byte-compare :697-700).
    let (ms1, mk1, md1) = template_cards("bip84", PHRASE_A, "0");
    let single = to_nonchunked(&md1);
    let mk1_variant: Vec<String> = mk1.iter().map(|m| m.to_uppercase()).collect();
    let mut args = vec![
        "verify-bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--account".into(),
        "0".into(),
        "--slot".into(),
        format!("@0.phrase={PHRASE_A}"),
    ];
    for m in ms1.iter().filter(|m| !m.is_empty()) {
        args.push("--ms1".into());
        args.push(m.clone());
    }
    for m in &mk1_variant {
        args.push("--mk1".into());
        args.push(m.clone());
    }
    args.push("--md1".into());
    args.push(single);
    mnemonic().args(&args).assert().code(4); // mk1 stub-bind fails → mismatch
}
