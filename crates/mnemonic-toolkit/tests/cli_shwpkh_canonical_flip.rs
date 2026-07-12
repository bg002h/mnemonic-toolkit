//! sh(wpkh) canonical re-pin cycle (md-codec 0.40→0.41, F-A1) — Phase B1.
//!
//! md-codec 0.41.0 flips `canonical_origin(sh(wpkh(@N)))` from `None` to
//! `Some(m/49'/0'/0')`. The toolkit's descriptor-mode probes consume
//! `is_non_canonical = canonical_origin(tree).is_none()` directly, so `sh(wpkh)`
//! moves from the NON-canonical branch to the CANONICAL branch — four
//! funds-visible behaviour flips, previously with ZERO test coverage.
//!
//! **THE FUNDS-LOAD-BEARING CORRECTION (plan-R0 C1):** post-flip,
//! `bundle --descriptor "sh(wpkh(@0))"` with no origin does NOT default to
//! `m/49'/0'/0'`. Because `is_non_canonical` is now false,
//! `bind_descriptor_mode_paths` EARLY-RETURNS (canonical: no default-inference,
//! `bundle.rs:2262-2266`) → the origin stays EMPTY → the card derives at the
//! MASTER key (`m/`), and the default-path notice is ABSENT. This is wpkh-parity
//! (identical to `wpkh(@0)`/`pkh(@0)` no-origin). The `49'` value exists ONLY in
//! md-codec's policy-id hashing + the n≥2 template-completion `canonical_fallback`
//! — NEVER on the toolkit's bundle EMIT path.
//!
//! Pre-flip (md-codec 0.40.0, empirically captured at toolkit `0287ce09`):
//! `bundle --descriptor "sh(wpkh(@0))"` no-origin emitted an mk1 with
//! `origin_path = 48'/0'/0'/1'` (a BIP-48 MULTISIG cosigner leaf, nonsensical
//! for single-key sh(wpkh)) + the default-path notice on stderr. SAME COMMAND,
//! DIFFERENT WALLET: `48'/0'/0'/1'`+notice → empty/master.

use assert_cmd::Command;
use serde_json::Value;

/// Standard BIP-39 12-word test seed ("abandon…about"); master fp 73c5da0a.
const SEED: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
/// Distinct second cosigner seed (for the non-canonical contrast).
const SEED_B: &str = "legal winner thank year wave sausage worth useful legal winner thank yellow";

/// The MASTER (depth-0) mainnet xpub for `SEED`, verified by decoding the
/// emitted card (`mk_codec::decode`). A no-origin CANONICAL single-sig card
/// encodes exactly this key. `xpub661My…` is the depth-0 mainnet version prefix.
const SEED_MASTER_XPUB: &str = "xpub661MyMwAqRbcFkPHucMnrGNzDwb6teAX1RbKQmqtEF8kK3Z7LZ59qafCjB9eCRLiTVG3uxBxgKvRgbubRhqSKXnGGb1aoaqLrpMBDrVxga8";

/// Substring of the stderr default-path notice emitted for NON-canonical
/// descriptors only (`emit_default_path_notice`, `bundle.rs`).
const DEFAULT_PATH_NOTICE: &str = "info: non-canonical descriptor; defaulting origin path";

/// `bundle --descriptor <d> --slot <s>… --json` → (parsed JSON, raw stderr).
/// Always `--no-engraving-card`, mainnet.
fn bundle(descriptor: &str, slots: &[&str]) -> (Value, String) {
    let mut args: Vec<String> = vec![
        "bundle".into(),
        "--descriptor".into(),
        descriptor.into(),
        "--network".into(),
        "mainnet".into(),
        "--no-engraving-card".into(),
        "--json".into(),
    ];
    for s in slots {
        args.push("--slot".into());
        args.push((*s).into());
    }
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    let v: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("bundle JSON invalid: {e}\nstdout:\n{stdout}"));
    (v, stderr)
}

/// Collect the single-sig mk1 chunk-vector from a `bundle --json` payload.
fn mk1_chunks(v: &Value) -> Vec<String> {
    v["mk1"]
        .as_array()
        .expect("mk1 array")
        .iter()
        .map(|c| c.as_str().expect("mk1 chunk str").to_string())
        .collect()
}

/// Decode a single-sig mk1 chunk-vector → (origin_path display, xpub, depth).
fn decode_mk1(chunks: &[String]) -> (String, String, u8) {
    let refs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
    let card = mk_codec::decode(&refs).expect("mk1 decode");
    (
        card.origin_path.to_string(),
        card.xpub.to_string(),
        card.xpub.depth,
    )
}

// ============================================================================
// FLIP #1 (funds-load-bearing) — no-origin sh(wpkh) now emits an
// EMPTY-ORIGIN / MASTER-KEY card with NO default-path notice (wpkh-parity),
// NOT the pre-flip 48'/0'/0'/1' + notice. (plan-R0 C1 — assert empty/master.)
// ============================================================================

#[test]
fn flip1_shwpkh_no_origin_emits_master_key_card_no_notice() {
    let slot = format!("@0.phrase={SEED}");
    let (v, stderr) = bundle("sh(wpkh(@0))", &[slot.as_str()]);

    // Top-level JSON origin is the empty/default sentinel (null), NOT 48'/0'.
    assert!(
        v["origin_path"].is_null(),
        "post-flip sh(wpkh) no-origin: top-level origin_path must be null (empty/master); got {:?}",
        v["origin_path"]
    );
    assert_eq!(v["mode"], "full");

    // The emitted mk1 encodes the MASTER key: empty origin path, depth 0.
    let (origin, xpub, depth) = decode_mk1(&mk1_chunks(&v));
    assert!(
        origin.is_empty(),
        "post-flip sh(wpkh) no-origin: mk1 origin_path must be empty/master (m/), NOT 48'/0'/0'/1' \
         and NOT 49'; got {origin:?}"
    );
    assert_eq!(
        depth, 0,
        "master key is depth-0; got depth {depth} (a defaulted origin would be depth-4)"
    );
    assert!(
        xpub.starts_with("xpub661My"),
        "depth-0 mainnet master xpub must carry the xpub661My… prefix; got {xpub}"
    );
    assert_eq!(
        xpub, SEED_MASTER_XPUB,
        "sh(wpkh) no-origin must encode the SEED master xpub (empty-origin derivation)"
    );

    // The default-path notice is ABSENT (canonical → no default-inference).
    assert!(
        !stderr.contains(DEFAULT_PATH_NOTICE),
        "post-flip sh(wpkh) is canonical: the default-path notice must be ABSENT; got stderr:\n{stderr}"
    );
    assert!(
        !stderr.contains("48'/0'/0'/1'"),
        "post-flip must NOT mention the pre-flip 48'/0'/0'/1' default; got stderr:\n{stderr}"
    );
}

/// RED-proof-equivalent (B0 already flipped, so this proves flip #1 genuinely
/// exercises the flip): a CANONICAL `wpkh(@0)` no-origin emits the SAME
/// empty-origin / master-key card, and post-flip `sh(wpkh(@0))` now MATCHES it
/// in the decoded key. Pre-flip these diverged (wpkh=master, sh(wpkh)=48'/0'/0'/1').
#[test]
fn flip1_shwpkh_now_matches_canonical_wpkh_master_parity() {
    let slot = format!("@0.phrase={SEED}");
    let (vw, stderr_w) = bundle("wpkh(@0)", &[slot.as_str()]);
    let (vs, stderr_s) = bundle("sh(wpkh(@0))", &[slot.as_str()]);

    let (ow, xw, dw) = decode_mk1(&mk1_chunks(&vw));
    let (os, xs, ds) = decode_mk1(&mk1_chunks(&vs));

    // wpkh(@0) is (and always was) canonical → master/empty.
    assert!(
        ow.is_empty() && dw == 0,
        "wpkh(@0) no-origin must be master/empty"
    );
    // sh(wpkh(@0)) now has PARITY with wpkh(@0): identical empty origin, depth,
    // and xpub.
    assert_eq!(
        os, ow,
        "sh(wpkh) origin must now match canonical wpkh (both empty)"
    );
    assert_eq!(
        ds, dw,
        "sh(wpkh) depth must now match canonical wpkh (both depth-0)"
    );
    assert_eq!(xs, xw, "sh(wpkh) master xpub must now match canonical wpkh's");
    // Neither canonical shape emits the default-path notice.
    assert!(!stderr_w.contains(DEFAULT_PATH_NOTICE));
    assert!(!stderr_s.contains(DEFAULT_PATH_NOTICE));
}

/// The flip-#1 "notice absent" assertion is non-vacuous: a genuinely
/// NON-canonical descriptor STILL receives the default-path notice + a
/// non-empty (48'/0'/0'/2') origin. Confirms the notice machinery is live and
/// that sh(wpkh) is on the canonical side of it.
#[test]
fn non_canonical_descriptor_still_gets_default_path_notice() {
    let s0 = format!("@0.phrase={SEED}");
    let s1 = format!("@1.phrase={SEED_B}");
    let (v, stderr) = bundle(
        "wsh(andor(pkh(@0),after(12000000),pk(@1)))",
        &[s0.as_str(), s1.as_str()],
    );
    assert_eq!(
        v["origin_path"].as_str(),
        Some("m/48'/0'/0'/2'"),
        "a non-canonical descriptor still default-infers a non-empty origin"
    );
    assert!(
        stderr.contains(DEFAULT_PATH_NOTICE),
        "a non-canonical descriptor must still emit the default-path notice; got:\n{stderr}"
    );
}

// ============================================================================
// FLIP #2 — `--account != 0` on canonical sh(wpkh) now REFUSES (ModeViolation,
// bundle.rs:1421-1427). Pre-flip (non-canonical) it was accepted.
// ============================================================================

#[test]
fn flip2_shwpkh_nonzero_account_refuses_mode_violation() {
    let assert = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            "sh(wpkh(@0))",
            "--account",
            "1",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={SEED}"),
            "--no-engraving-card",
            "--json",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("--account") && stderr.contains("descriptor mode"),
        "sh(wpkh) --account!=0 must refuse with the descriptor-mode account ModeViolation; got:\n{stderr}"
    );
}

/// Companion: `--account 0` on canonical sh(wpkh) is NOT a violation (only the
/// non-zero account is meaningless in descriptor mode).
#[test]
fn flip2_shwpkh_account_zero_still_succeeds() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            "sh(wpkh(@0))",
            "--account",
            "0",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={SEED}"),
            "--no-engraving-card",
            "--json",
        ])
        .assert()
        .success();
}

// ============================================================================
// FLIP #3 — a per-slot `--slot @N.path=` override on canonical sh(wpkh) now
// REFUSES, but ONLY for a SECRET-BEARING slot set `[Phrase|Seedqr|Ms1, Path]`
// (bundle.rs:1429-1453 §6.6 row 4). An `[Xpub, Path]` (watch-only) slot set
// STILL SUCCEEDS and honours the path (plan-R0 I1). Pre-flip (non-canonical)
// both honoured the path.
// ============================================================================

#[test]
fn flip3_shwpkh_secret_bearing_path_override_refuses() {
    let assert = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            "sh(wpkh(@0))",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={SEED}"),
            "--slot",
            "@0.path=m/48'/0'/0'/1'",
            "--no-engraving-card",
            "--json",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("secret-bearing input and watch-only input"),
        "sh(wpkh) [Phrase,Path] must refuse as a §6.6 row-4 slot conflict; got:\n{stderr}"
    );
}

/// Companion (plan-R0 I1) — an `[Xpub, Path]` (watch-only) slot set on canonical
/// sh(wpkh) does NOT refuse: it succeeds and honours the supplied path.
#[test]
fn flip3_shwpkh_watch_only_xpub_path_override_still_succeeds() {
    // An arbitrary valid mainnet xpub (fixture corpus); content is irrelevant —
    // only the routing/honouring is under test.
    const XPUB: &str = "xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9";
    let s0 = format!("@0.xpub={XPUB}");
    let (v, _stderr) = bundle(
        "sh(wpkh(@0))",
        &[
            s0.as_str(),
            "@0.fingerprint=deadbeef",
            "@0.path=m/48'/0'/0'/1'",
        ],
    );
    assert_eq!(v["mode"], "watch-only");
    assert_eq!(
        v["origin_path"].as_str(),
        Some("m/48'/0'/0'/1'"),
        "[Xpub,Path] on canonical sh(wpkh) must succeed and honour the override path"
    );
}

// ============================================================================
// FLIP #4 — a PRE-BUMP elided-sh(wpkh) bundle (a hardcoded fixture: it cannot
// be regenerated post-flip because the command now emits an empty-origin card)
// fails `verify-bundle` post-repin, fail-LOUD as a md1 BYTE-MISMATCH — NOT a
// silent false-pass, and NOT a probe-level error (R0-R1 M3). Post-flip verify
// re-derives an EMPTY-origin md1/mk1 that cannot match the stored explicit-48'
// cards.
//
// Fixture provenance: captured 2026-07-11 from the PRE-flip toolkit binary
// (`0287ce09`, md-codec 0.40.0) via:
//   mnemonic bundle --descriptor "sh(wpkh(@0))" --network mainnet \
//     --slot "@0.phrase=<SEED>" --no-engraving-card --json
// It verified `result: ok` against that pre-flip binary (origin defaulted to
// 48'/0'/0'/1', matching); the CURRENT (post-flip) binary re-derives empty/
// master and MISMATCHES.
// ============================================================================

// Pre-flip sh(wpkh) no-origin bundle cards (origin = m/48'/0'/0'/1').
const PREFLIP_MK1: &[&str] = &[
    "mk1qpg2shpqqsq59gt543eutks2qczg3vs7rnefw94w9ta9h0xfepwzexh0kz5f7fmpfputrh78437r3krpf3j0l07wwqpk6k40tpdjm4rfxtrs",
    "mk1qpg2shpp5nykk78g44wvx4ttk3jwrtg9zvkvevqf3mdfxd7nlzpzg9uk4f0m4kfyvy3du74ewl27m",
];
const PREFLIP_MD1: &[&str] = &[
    "md1fr7t0pqpqztvyyyvpsqtgrnchdq592pzhz47jmhnyushpvnthmqfuygcj9luxuqx",
    "md1fr7t0pq25f7fmpfputrh78437r3krpf3j0l07wwqpkmfxfdduws60j3lk52ux2tp",
    "md1fr7t0pqjk4es64dw6xfcddq5fjen9spx8d4yeh60ugyfqhj64qqf7w73e4vp2f7x",
];
const PREFLIP_MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

#[test]
fn flip4_preflip_elided_shwpkh_bundle_fails_verify_as_byte_mismatch() {
    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--descriptor".into(),
        "sh(wpkh(@0))".into(),
        "--slot".into(),
        format!("@0.phrase={SEED}"),
        "--json".into(),
    ];
    for c in PREFLIP_MK1 {
        args.push((*c).into());
    }
    for c in PREFLIP_MD1 {
        args.push((*c).into());
    }
    args.push(PREFLIP_MS1.into());

    let assert = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .assert()
        .failure(); // fail-LOUD (exit 4 = mismatch), NOT a false-pass
    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let v: Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("verify-bundle JSON invalid: {e}\nstdout:\n{stdout}"));

    // Overall verdict: mismatch (fail-loud), not "ok" and not a probe error.
    assert_eq!(
        v["result"], "mismatch",
        "post-flip verify of a pre-flip elided sh(wpkh) bundle must fail as a MISMATCH; got {:?}",
        v["result"]
    );

    let checks = v["checks"].as_array().expect("checks array");
    let find = |name: &str| -> &Value {
        checks
            .iter()
            .find(|c| c["name"] == name)
            .unwrap_or_else(|| panic!("check {name} missing from {checks:?}"))
    };

    // ms1 (seed) is fine — the divergence is purely the origin/xpub re-derivation.
    assert_eq!(find("ms1_entropy_match")["passed"], true);

    // The mk1 path re-derives to EMPTY (post-flip canonical) vs the stored
    // 48'/0'/0'/1' — this is the direct byte-level manifestation of the flip.
    let path_check = find("mk1_path_match");
    assert_eq!(
        path_check["passed"], false,
        "mk1 path must mismatch post-flip"
    );
    assert_eq!(
        path_check["expected"].as_str(),
        Some(""),
        "post-flip re-derived origin path is EMPTY (master); got {:?}",
        path_check["expected"]
    );
    assert_eq!(
        path_check["actual"].as_str(),
        Some("48'/0'/0'/1'"),
        "the STORED pre-flip card carries the explicit 48'/0'/0'/1' origin"
    );
    assert_eq!(
        find("mk1_xpub_match")["passed"],
        false,
        "mk1 xpub must mismatch"
    );

    // The md1 BYTE-MISMATCH (the funds-load-bearing assertion): the re-derived
    // empty-origin md1 wallet-policy xpub differs from the stored explicit-48' one.
    assert_eq!(
        find("md1_xpub_match")["passed"],
        false,
        "the stored explicit-48' md1 must NOT match the re-derived empty-origin md1 (byte-mismatch)"
    );
}

// ============================================================================
// B1.2 — S3.5 audit GUARD tests: the two funds-load-bearing `canonical_origin`
// consumers. A KEYLESS sh(wpkh) template md1 (n==1) — which the toolkit REFUSES
// to emit but a future bug / hand-craft could produce — must NEVER route into
// the single-sig-template completion (restore) or verify path, despite the
// canonical_origin flip. The `&& cli_template_from_tree(&tree).is_some()`
// conjunct (no-`Sh`-arm → FALSE for sh(wpkh)) neutralises the flip at both sites
// (restore.rs:317-320 / verify_bundle.rs:389-392).
//
// Fixture provenance: forged 2026-07-11 with md_codec only — decode a real KEYED
// sh(wpkh) policy md1, strip pubkeys/fingerprints, elide origin, re-`chunk::split`
// → a keyless template md1. Confirmed at forge time: reassembles to n==1,
// is_wallet_policy()==false, canonical_origin().is_some()==TRUE (the flip is
// live) — so ONLY cli_template_from_tree stops it.
// ============================================================================

const FORGED_KEYLESS_SHWPKH_TEMPLATE_MD1: &str = "md1fn3lnqqpqqqxqqzltp2wcfn0fqa";

/// restore.rs:1645 guard — a keyless sh(wpkh) template md1 does NOT route to
/// `run_singlesig_template_completion` and NEVER reaches `canonical_fallback`
/// with a 49' derivation. It refuses via the multisig keyless-md1 gate.
#[test]
fn guard_restore_keyless_shwpkh_template_refuses_never_49prime() {
    let assert = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "restore",
            "--md1",
            FORGED_KEYLESS_SHWPKH_TEMPLATE_MD1,
            "--from",
            SEED,
            "--network",
            "mainnet",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    // The refusal is the MULTISIG keyless-md1 gate — NOT a single-sig template
    // completion, and NOT any origin derivation (49' or otherwise).
    assert!(
        stderr.contains("template-only") && stderr.contains("wallet-policy md1"),
        "keyless sh(wpkh) template restore must refuse via the multisig keyless-md1 gate, \
         not route to single-sig template completion; got:\n{stderr}"
    );
    assert!(
        !stderr.contains("49'"),
        "restore must NEVER derive a 49' origin for a keyless sh(wpkh) template; got:\n{stderr}"
    );
}

/// verify_bundle.rs:391 VERIFY-side guard (mirror of the restore guard) — the
/// same keyless sh(wpkh) template md1 does NOT route into
/// `verify_singlesig_template`. With no `--template`/`--descriptor` it falls
/// through and demands one (proving it was NOT recognised as a keyless
/// single-sig template).
#[test]
fn guard_verify_keyless_shwpkh_template_never_singlesig_template_path() {
    // Any mk1 chunk-vector satisfies clap's `--mk1` requirement; routing is
    // decided from the md1 alone (before the cards are matched).
    let assert = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "verify-bundle",
            "--md1",
            FORGED_KEYLESS_SHWPKH_TEMPLATE_MD1,
            "--mk1",
            PREFLIP_MK1[0],
            "--mk1",
            PREFLIP_MK1[1],
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={SEED}"),
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("--template is required")
            && stderr.contains("not a keyless single-sig template"),
        "keyless sh(wpkh) template verify must fall through (NOT route into the single-sig \
         template verify path) and demand --template/--descriptor; got:\n{stderr}"
    );
}

/// Companion (mirror UNCHANGED) — the toolkit still REFUSES to EMIT a keyless
/// sh(wpkh) template (`--md1-form=template`); only the stale COMMENT reasoning
/// (canonical_origin==None) changed, not the refusal itself
/// (`cli_template_from_tree` stays no-`Sh`-arm). Guards the S4 "do not fix the
/// mirror" invariant.
#[test]
fn mirror_unchanged_shwpkh_template_emit_still_refused() {
    let assert = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--descriptor",
            "sh(wpkh(@0))",
            "--network",
            "mainnet",
            "--md1-form",
            "template",
            "--slot",
            &format!("@0.phrase={SEED}"),
            "--no-engraving-card",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("md1-form=policy"),
        "sh(wpkh) template emission must still refuse and route to --md1-form=policy; got:\n{stderr}"
    );
}
