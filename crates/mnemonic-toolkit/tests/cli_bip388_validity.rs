//! Sweep unit `export-wallet-bip388-validity` (FIX_export_wallet_bip388_validity.md):
//! `--format bip388` refuses a policy BIP-388 calls invalid.
//! (1) A literal (non-`@N`) key: the NUMS taproot internal key, on the
//!     template arm (export-wallet, restore of a NUMS tr-multisig md1) and the
//!     descriptor arm. BIP-388 lines 149-162 and 310; ruled text (FOLLOWUPS
//!     `bip388-literal-nums-key-refusal-text`, architect P25 option (a)).
//! (2) `keys_info` whose public keys are not pairwise distinct (BIP-388
//!     line 193: "The public keys obtained by deserializing elements of the
//!     key information vector must be pairwise distinct").
//! Signet keys only.

use assert_cmd::Command;

const NUMS: &str = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";
const C0: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const C1: &str = "legal winner thank year wave sausage worth useful legal winner thank yellow";
const C2: &str = "letter advice cage absurd amount doctor acoustic avoid letter advice cage above";
const FP: [&str; 3] = ["73c5da0a", "b8688df1", "28645006"];
const TPUB: [&str; 3] = [
    "tpubDCChhoz5Qdrkn7Z7KXawq6Ad6r3A4MUkCoVTqeWxfTkA6bHNJ3CHUEtALQdkNeixNz4446PcAmw4WKcj3mV2vb29H7sg9EPzbyCU1y2merw",
    "tpubDC44ZFebbFm5ZXnG5Xt8k9PpdSZXUQ4HyuU8e8u5zfutYuu8EW4MLKS54EFTfGYgeqt7P2jEEd1iCF5kSPi7Xrj6EZcsStCcAjPy97DSiLb",
    "tpubDCs6kWyAq2CsViTz5YR3tKpCQ4kq2xwDmERmF5cGet45CsHAqR8wbQGRin6fzLJgCdrKfYts4ddyTJ7Zo4ocmG1zHK3vw71mh32wj25WmsT",
];

fn refusal() -> String {
    format!(
        "error: --format bip388 cannot express the literal key {NUMS} (a taproot internal \
         key with no origin, e.g. the BIP-341 NUMS key): a BIP-388 wallet policy allows only \
         @N key placeholders; use --format descriptor or --format bitcoin-core for this \
         wallet\n"
    )
}

fn run(args: &[String]) -> (i32, String, String) {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(args)
        .output()
        .expect("spawn mnemonic");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn export_args(format: &str, internal: &str, output: &str) -> Vec<String> {
    let mut a: Vec<String> = [
        "export-wallet",
        "--format",
        format,
        "--template",
        "tr-sortedmulti-a",
        "--taproot-internal-key",
        internal,
        "--threshold",
        "2",
        "--network",
        "signet",
        "--output",
        output,
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    for i in 0..3 {
        a.push("--slot".into());
        a.push(format!("@{i}.xpub={}", TPUB[i]));
        a.push("--slot".into());
        a.push(format!("@{i}.fingerprint={}", FP[i]));
        a.push("--slot".into());
        a.push(format!("@{i}.path=m/87'/1'/0'"));
    }
    a
}

/// RED: export-wallet template arm refuses (exit 1, empty stdout, ruled text).
#[test]
fn p25_export_wallet_template_nums_bip388_refuses() {
    let (code, stdout, stderr) = run(&export_args("bip388", "nums", "-"));
    assert_eq!(
        (code, stdout.as_str(), stderr.as_str()),
        (2, "", refusal().as_str())
    );
}

/// RED: with `--output <file>` the refusal writes no file.
#[test]
fn p25_export_wallet_template_nums_bip388_writes_no_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("policy.json");
    let (code, _stdout, stderr) = run(&export_args("bip388", "nums", path.to_str().unwrap()));
    assert_eq!((code, stderr.as_str()), (2, refusal().as_str()));
    assert!(!path.exists(), "refusal must not create {}", path.display());
}

/// RED: restore of a NUMS tr-sortedmulti-a md1 shares the template emitter.
#[test]
fn p25_restore_nums_tr_multisig_bip388_refuses() {
    let mut b: Vec<String> = [
        "--allow-argv-secret",
        "bundle",
        "--template",
        "tr-sortedmulti-a",
        "--threshold",
        "2",
        "--network",
        "signet",
        "--json",
        "--no-engraving-card",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    for (i, p) in [C0, C1, C2].iter().enumerate() {
        b.push("--slot".into());
        b.push(format!("@{i}.phrase={p}"));
    }
    let (code, stdout, stderr) = run(&b);
    assert_eq!(code, 4, "bundle: {stderr}");
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let mut r: Vec<String> = vec!["restore".into(), "--network".into(), "signet".into()];
    for c in v["md1"].as_array().unwrap() {
        r.push("--md1".into());
        r.push(c.as_str().unwrap().into());
    }
    r.push("--format".into());
    r.push("bip388".into());
    let (code, stdout, stderr) = run(&r);
    assert_eq!((code, stdout.as_str()), (2, ""), "stderr: {stderr}");
    assert_eq!(
        stderr,
        refusal(),
        "restore prints no preamble before a refusal"
    );
}

/// Controls (GREEN before and after): a cosigner internal key still emits a
/// placeholder policy, and `--format descriptor` / `bitcoin-core` (the forms
/// the refusal names) still emit the NUMS wallet.
#[test]
fn p25_controls_cosigner_internal_key_and_named_alternatives() {
    let (code, stdout, stderr) = run(&export_args("bip388", "@0", "-"));
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.contains("\"tr(@0/**,sortedmulti_a(2,@1/**,@2/**))\""),
        "{stdout}"
    );
    for f in ["descriptor", "bitcoin-core"] {
        let (code, stdout, stderr) = run(&export_args(f, "nums", "-"));
        assert_eq!(code, 0, "{f} stderr: {stderr}");
        assert!(
            stdout.contains(&format!("tr({NUMS},sortedmulti_a(2,")),
            "{f}: {stdout}"
        );
    }
}

// ===========================================================================
// FIX rows (FIX_export_wallet_bip388_validity.md section 6)
// ===========================================================================

/// BIP-48 signet account keys of the BIP-39 test phrases C0 and C1
/// (`convert --to xpub --network signet --path m/48'/1'/0'/2'`).
const XA: &str = "tpubDFH9dgzveyD8zTbPUFuLrGmCydNvxehyNdUXKJAQN8x4aZ4j6UZqGfnqFrD4NqyaTVGKbvEW54tsvPTK2UoSbCC1PJY8iCNiwTL3RWZEheQ";
const XB: &str = "tpubDEfobrrtptRTbKf4gysDhoabneABDTAcdj3Vbn4XwPsLE2pmqpizSPRG6zHsbAMuiSgWmWPsYCLHTKTPpyrGJ5rAoTpKoQNZcxodiPf2tSJ";
/// `bundle --descriptor <D_DUP_AT0> --slot @0.phrase=C0 --network signet`
/// derives @0 at the BIP-48 default m/48'/1'/0'/2' with this account key.
const XA_BUNDLE: &str = "tpubDDuXvjq5jan2D61HJDa31HeizPJdArvkNnax3csWXAmrggizBTFT5opWe81xufQv33vJUbzYaCEtPA3qJQY9hvQrMFckTU7zKvB3wHAnLHe";

fn ka() -> String {
    format!("[73c5da0a/48'/1'/0'/2']{XA}")
}
fn kb() -> String {
    format!("[b8688df1/48'/1'/0'/2']{XB}")
}
/// The same extended key as `ka()` under another origin.
fn ka_other_origin() -> String {
    format!("[b8688df1/48'/1'/0'/2']{XA}")
}
fn or_d(k0: &str, k1: &str) -> String {
    format!("wsh(or_d(pk({k0}/<0;1>/*),and_v(v:pk({k1}/<0;1>/*),older(100))))")
}

/// DUP text: the non-distinct keys_info refusal, export-wallet / restore form.
fn dup_text(i: usize, j: usize, xpub: &str) -> String {
    format!(
        "error: --format bip388 cannot express a wallet that uses one public key more than \
         once (keys_info @{i} and @{j}: {xpub}): a BIP-388 wallet policy requires pairwise \
         distinct keys; use --format descriptor or --format bitcoin-core for this wallet\n"
    )
}
/// DUP text, build-descriptor form (that command has no bitcoin-core format).
fn dup_text_build(i: usize, j: usize, xpub: &str) -> String {
    format!(
        "error: --format bip388 cannot express a wallet that uses one public key more than \
         once (keys_info @{i} and @{j}: {xpub}): a BIP-388 wallet policy requires pairwise \
         distinct keys; use --format descriptor for this wallet\n"
    )
}
/// LIT text: the ruled literal-key refusal for a given key string.
fn lit_text(hex: &str) -> String {
    format!(
        "error: --format bip388 cannot express the literal key {hex} (a taproot internal \
         key with no origin, e.g. the BIP-341 NUMS key): a BIP-388 wallet policy allows only \
         @N key placeholders; use --format descriptor or --format bitcoin-core for this \
         wallet\n"
    )
}

fn sv(a: &[&str]) -> Vec<String> {
    a.iter().map(|s| s.to_string()).collect()
}

fn export_desc(format: &str, descriptor: &str) -> Vec<String> {
    sv(&[
        "export-wallet",
        "--format",
        format,
        "--descriptor",
        descriptor,
        "--network",
        "signet",
        "--output",
        "-",
    ])
}

fn export_template_dup(format: &str, x1: &str, fp1: &str, p1: &str, output: &str) -> Vec<String> {
    let mut a = sv(&[
        "export-wallet",
        "--format",
        format,
        "--template",
        "wsh-sortedmulti",
        "--threshold",
        "1",
        "--network",
        "signet",
        "--output",
        output,
    ]);
    for s in [
        format!("@0.xpub={XA}"),
        "@0.fingerprint=73c5da0a".to_string(),
        "@0.path=m/48'/1'/0'/2'".to_string(),
        format!("@1.xpub={x1}"),
        format!("@1.fingerprint={fp1}"),
        format!("@1.path={p1}"),
    ] {
        a.push("--slot".into());
        a.push(s);
    }
    a
}

fn refused(r: (i32, String, String), stderr: &str) {
    assert_eq!((r.0, r.1.as_str(), r.2.as_str()), (2, "", stderr));
}

/// RED: descriptor arm, the same key twice (the grade's probe shape).
#[test]
fn bip388v_export_descriptor_repeated_key_refused() {
    let r = run(&export_desc("bip388", &or_d(&ka(), &ka())));
    refused(r, &dup_text(0, 1, XA));
}

/// RED: descriptor arm, the same extended key under two origins. BIP-388
/// compares the deserialized public keys, not the origins.
#[test]
fn bip388v_export_descriptor_same_xpub_other_origin_refused() {
    let r = run(&export_desc("bip388", &or_d(&ka(), &ka_other_origin())));
    refused(r, &dup_text(0, 1, XA));
}

/// `XA` with chain-code byte 0 XOR 0x01, re-encoded: the same public key
/// under another extended key (a different chain code).
const XA_OTHER_CHAINCODE: &str = "tpubDFH9dgzveyD8zT1tvsUvpENmA4ap3U5kyXv4qKQGYqRbXvoGcJfGqSExEeptW6f7HbrfyKS1rrSTGmXAyCjZYf5rceNxnQ7pNBo8adwpjEd";

/// RED (S3): two extended keys with one public key (different chain codes).
/// The rule compares the deserialized public key, not the extended key.
#[test]
fn bip388v_export_descriptor_same_pubkey_other_chaincode_refused() {
    let other = format!("[b8688df1/48'/1'/0'/2']{XA_OTHER_CHAINCODE}");
    let r = run(&export_desc("bip388", &or_d(&ka(), &other)));
    refused(r, &dup_text(0, 1, XA));
}

/// RED: a sortedmulti repeating a key (the pair is reported, not a leaf path).
#[test]
fn bip388v_export_descriptor_sortedmulti_repeated_key_refused() {
    let d = format!("wsh(sortedmulti(1,{}/<0;1>/*,{}/<0;1>/*))", ka(), ka());
    let r = run(&export_desc("bip388", &d));
    refused(r, &dup_text(0, 1, XA));
}

/// RED: a BIP-388 policy JSON given to --descriptor whose keys_info repeats.
#[test]
fn bip388v_export_descriptor_policy_json_repeated_key_refused() {
    let p = format!(
        "{{\"name\":\"dup\",\"description_template\":\"wsh(or_d(pk(@0/**),and_v(v:pk(@1/**),older(100))))\",\"keys_info\":[\"{}\",\"{}\"]}}",
        ka(),
        ka()
    );
    let r = run(&export_desc("bip388", &p));
    refused(r, &dup_text(0, 1, XA));
}

/// RED: template arm, two slots with the same key, and no file written.
#[test]
fn bip388v_export_template_repeated_slot_refused() {
    let r = run(&export_template_dup(
        "bip388",
        XA,
        "73c5da0a",
        "m/48'/1'/0'/2'",
        "-",
    ));
    refused(r, &dup_text(0, 1, XA));
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("policy.json");
    let r = run(&export_template_dup(
        "bip388",
        XA,
        "73c5da0a",
        "m/48'/1'/0'/2'",
        path.to_str().unwrap(),
    ));
    assert_eq!((r.0, r.2.as_str()), (2, dup_text(0, 1, XA).as_str()));
    assert!(!path.exists(), "refusal must not create {}", path.display());
}

/// RED: template arm, the same key under another fingerprint and path.
#[test]
fn bip388v_export_template_same_xpub_other_path_refused() {
    let r = run(&export_template_dup(
        "bip388",
        XA,
        "b8688df1",
        "m/48'/1'/1'/2'",
        "-",
    ));
    refused(r, &dup_text(0, 1, XA));
}

/// RED: restore of a policy md1 that uses @0 twice (general wsh arm).
#[test]
fn bip388v_restore_repeated_key_bip388_refused() {
    let b = sv(&[
        "--allow-argv-secret",
        "bundle",
        "--descriptor",
        "wsh(or_d(pk(@0/<0;1>/*),and_v(v:pk(@0/<0;1>/*),older(100))))",
        "--slot",
        &format!("@0.phrase={C0}"),
        "--network",
        "signet",
        "--json",
        "--no-engraving-card",
    ]);
    let (code, stdout, stderr) = run(&b);
    assert_eq!(code, 4, "bundle: {stderr}");
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let mut r = sv(&["restore", "--network", "signet", "--format", "bip388"]);
    for c in v["md1"].as_array().unwrap() {
        r.push("--md1".into());
        r.push(c.as_str().unwrap().into());
    }
    refused(run(&r), &dup_text(0, 1, XA_BUNDLE));
}

fn build_dup(extra: &[&str]) -> Vec<String> {
    let mut a = sv(&[
        "build-descriptor",
        "--archetype",
        "simple-timelocked-inheritance",
        "--key",
        &ka(),
        "--recovery-key",
        &ka(),
        "--older",
        "100",
        "--network",
        "signet",
        "--allow",
        "repeated-keys",
    ]);
    a.extend(extra.iter().map(|s| s.to_string()));
    a
}

const ALLOW_BANNER: &str = "WARNING: sanity rules OVERRIDDEN by --allow and FIRED: repeated-keys. This descriptor failed miniscript's funds-safety analysis; you have accepted that risk after review.\n";

/// RED: build-descriptor `--allow repeated-keys --format bip388` refuses
/// (after the --allow banner) with the build-descriptor DUP text.
#[test]
fn bip388v_build_descriptor_repeated_key_bip388_refused() {
    let (code, stdout, stderr) = run(&build_dup(&["--format", "bip388"]));
    let want = format!("{ALLOW_BANNER}{}", dup_text_build(0, 1, XA));
    assert_eq!(
        (code, stdout.as_str(), stderr.as_str()),
        (2, "", want.as_str())
    );
}

/// RED (S1): build-descriptor `--json` keeps exit 0 and the descriptor, and
/// writes `"bip388": null` for a policy BIP-388 cannot express.
#[test]
fn bip388v_build_descriptor_repeated_key_json_bip388_null() {
    let (code, stdout, stderr) = run(&build_dup(&["--json"]));
    assert_eq!(code, 0, "stderr: {stderr}");
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(v["bip388"].is_null(), "bip388: {}", v["bip388"]);
    assert!(v["descriptor"]
        .as_str()
        .unwrap()
        .starts_with("wsh(or_d(pk("));
    assert_eq!(stderr, ALLOW_BANNER);
}

/// RED: descriptor arm, a NUMS internal key in a non-multipath descriptor
/// gets the literal-key text (checked before the multipath rule).
#[test]
fn bip388v_export_descriptor_nums_literal_before_multipath() {
    let d = format!("tr({NUMS},sortedmulti_a(1,{}/0/*,{}/0/*))", ka(), kb());
    refused(run(&export_desc("bip388", &d)), &lit_text(NUMS));
}

/// RED: descriptor arm, a NUMS internal key (multipath leaves), signet.
#[test]
fn bip388v_export_descriptor_nums_refused() {
    let d = format!(
        "tr({NUMS},sortedmulti_a(1,{}/<0;1>/*,{}/<0;1>/*))",
        ka(),
        kb()
    );
    refused(run(&export_desc("bip388", &d)), &lit_text(NUMS));
}

/// GUARD: template arm, a taproot multisig with two equal slots is refused
/// upstream by the strict descriptor parse (exit 2, unchanged); neither
/// bip388 refusal is reached.
#[test]
fn bip388v_guard_template_tr_repeated_slot_refused_upstream() {
    let mut a = sv(&[
        "export-wallet",
        "--format",
        "bip388",
        "--template",
        "tr-sortedmulti-a",
        "--taproot-internal-key",
        "nums",
        "--threshold",
        "1",
        "--network",
        "signet",
        "--output",
        "-",
    ]);
    for i in 0..2 {
        for s in [
            format!("@{i}.xpub={XA}"),
            format!("@{i}.fingerprint=73c5da0a"),
            format!("@{i}.path=m/48'/1'/0'/2'"),
        ] {
            a.push("--slot".into());
            a.push(s);
        }
    }
    let r = run(&a);
    assert_eq!(
        (r.0, r.1.as_str(), r.2.as_str()),
        (
            2,
            "",
            "error: export-wallet descriptor parse: Miniscript contains repeated pubkeys or pubkeyhashes\n"
        )
    );
}

// ---- guards (pass before and after) ----

/// GUARD: distinct keys still emit, descriptor arm and template arm.
#[test]
fn bip388v_guard_distinct_keys_emit() {
    let (code, stdout, stderr) = run(&export_desc("bip388", &or_d(&ka(), &kb())));
    assert_eq!(code, 0, "stderr: {stderr}");
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["keys_info"].as_array().unwrap().len(), 2);
    let (code, stdout, stderr) = run(&export_template_dup(
        "bip388",
        XB,
        "b8688df1",
        "m/48'/1'/0'/2'",
        "-",
    ));
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(
        stdout.contains("\"wsh(sortedmulti(1,@0/**,@1/**))\""),
        "{stdout}"
    );
}

/// GUARD: the alternatives the DUP text names still emit the wallet.
#[test]
fn bip388v_guard_named_alternatives_emit_repeated_key() {
    for f in ["descriptor", "bitcoin-core"] {
        let (code, stdout, stderr) = run(&export_desc(f, &or_d(&ka(), &ka())));
        assert_eq!(code, 0, "{f} stderr: {stderr}");
        assert!(stdout.contains(XA), "{f}: {stdout}");
        let (code, _, stderr) = run(&export_template_dup(
            f,
            XA,
            "73c5da0a",
            "m/48'/1'/0'/2'",
            "-",
        ));
        assert_eq!(code, 0, "{f} template stderr: {stderr}");
    }
}

/// GUARD: build-descriptor `--format descriptor` and the human view are not
/// affected by the bip388 rule.
#[test]
fn bip388v_guard_build_descriptor_other_outputs_emit() {
    let (code, stdout, stderr) = run(&build_dup(&["--format", "descriptor"]));
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stdout.starts_with("wsh(or_d(pk("), "{stdout}");
    let (code, stdout, stderr) = run(&build_dup(&[]));
    assert_eq!(code, 0, "stderr: {stderr}");
    assert!(stdout.starts_with("descriptor:\n"), "{stdout}");
}

/// GUARD: one key under disjoint multipaths keeps today's suffix refusal
/// (the suffix rule runs before the distinct-key rule).
#[test]
fn bip388v_guard_disjoint_multipath_keeps_suffix_text() {
    let d = format!(
        "wsh(or_d(pk({}/<0;1>/*),and_v(v:pk({}/<2;3>/*),older(100))))",
        ka(),
        ka()
    );
    let want = format!(
        "error: --format bip388 requires every descriptor key to end in `/<0;1>/*` for the receive/change pair; got key \"{}/<2;3>/*\"\n",
        ka()
    );
    refused(run(&export_desc("bip388", &d)), &want);
}

/// GUARD: a literal key in a wsh leaf (not a taproot internal key) keeps
/// today's suffix refusal; the literal-key text names only the internal key.
#[test]
fn bip388v_guard_literal_leaf_key_keeps_suffix_text() {
    let d = format!(
        "wsh(or_d(pk({}/<0;1>/*),and_v(v:pk(02{NUMS}),older(100))))",
        ka()
    );
    let want = format!(
        "error: --format bip388 requires every descriptor key to end in `/<0;1>/*` for the receive/change pair; got key \"02{NUMS}\"\n"
    );
    refused(run(&export_desc("bip388", &d)), &want);
}

// ===========================================================================
// Architect fold (architect-bip388-validity-s1s3-1124): a derivation step
// between the extended key and `/<0;1>/*` has no BIP-388 form (bip-0388
// L172 "no derivation steps are allowed in a KP expression"; L180-187 a
// KEY_INFO is an optional origin then a serialized extended public key).
// ===========================================================================

/// EXT text: the extra-derivation-step refusal for a given descriptor key.
fn ext_text(key: &str) -> String {
    format!(
        "error: --format bip388 cannot express the key {key} (a derivation step between the \
         extended public key and `/<0;1>/*`): a BIP-388 key information entry is an extended \
         public key with no path after it; use --format descriptor or --format bitcoin-core \
         for this wallet\n"
    )
}

/// RED: descriptor arm, one key with a step `/5` before `/<0;1>/*`.
#[test]
fn bip388v_export_descriptor_extra_step_refused() {
    let d = format!(
        "wsh(or_d(pk({}/5/<0;1>/*),and_v(v:pk({}/<0;1>/*),older(100))))",
        ka(),
        kb()
    );
    refused(
        run(&export_desc("bip388", &d)),
        &ext_text(&format!("{}/5/<0;1>/*", ka())),
    );
}

/// RED: a BIP-388 policy JSON given to --descriptor whose keys_info entry
/// carries a trailing path (`…/5`).
#[test]
fn bip388v_export_descriptor_policy_json_extra_step_refused() {
    let p = format!(
        "{{\"name\":\"ext\",\"description_template\":\"wsh(or_d(pk(@0/**),and_v(v:pk(@1/**),older(100))))\",\"keys_info\":[\"{}/5\",\"{}\"]}}",
        ka(),
        kb()
    );
    refused(
        run(&export_desc("bip388", &p)),
        &ext_text(&format!("{}/5/<0;1>/*", ka())),
    );
}

/// RED: the extra-step rule runs per key, before the distinct-key rule
/// (one xpub under `/5` and `/6`: EXT, not DUP).
#[test]
fn bip388v_export_descriptor_extra_step_before_dup() {
    let d = format!(
        "wsh(or_d(pk({}/5/<0;1>/*),and_v(v:pk({}/6/<0;1>/*),older(100))))",
        ka(),
        ka()
    );
    refused(
        run(&export_desc("bip388", &d)),
        &ext_text(&format!("{}/5/<0;1>/*", ka())),
    );
}

/// GUARD: the alternatives the EXT text names still emit that wallet.
#[test]
fn bip388v_guard_extra_step_alternatives_emit() {
    let d = format!(
        "wsh(or_d(pk({}/5/<0;1>/*),and_v(v:pk({}/<0;1>/*),older(100))))",
        ka(),
        kb()
    );
    for f in ["descriptor", "bitcoin-core"] {
        let (code, stdout, stderr) = run(&export_desc(f, &d));
        assert_eq!(code, 0, "{f} stderr: {stderr}");
        assert!(stdout.contains(&format!("{XA}/5/")), "{f}: {stdout}");
    }
}
