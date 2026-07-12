//! P2.2 (pathless/dead-card partial-decode) — `verify-bundle` explicit partial
//! gate (FUNDS-CRITICAL).
//!
//! Per `design/IMPLEMENTATION_PLAN_pathless_partial_decode.md` §P2.2 +
//! `design/SPEC_pathless_partial_decode.md`. A supplied md1 that decodes cleanly
//! EXCEPT for an elided-and-unresolvable origin (a "dead card") downgrades an
//! otherwise-`ok` verdict to `result: "partial"` / exit 4 — NEVER a pass.
//! Precedence: `mismatch > partial > ok` (a failed structural check WINS). The
//! cross-chunk content-id oracle stays enforced under partial (a doctored dead
//! card → `mismatch`, never `partial`).
//!
//! Fixtures are FROZEN bundle cards (minted by `mnemonic bundle`) — the md1 is
//! decoded and re-emitted with its origin ELIDED in-crate (`elide`) to produce
//! the keyed dead card. Only the SUPPLIED md1 is elided; the expected bundle is
//! synthesized fresh (WITH origin) from the same `--template`/`--slot` inputs.
//! I-2: all md1 fixtures are CHUNK-FORM.

#![allow(missing_docs)]

use assert_cmd::Command;
use md_codec::origin_path::{OriginPath, PathDeclPaths};

// ── FROZEN: `mnemonic bundle --template tr-sortedmulti-a --threshold 2` ──────
const P1: &str = "legal winner thank year wave sausage worth useful legal winner thank yellow";
const P2: &str = "letter advice cage absurd amount doctor acoustic avoid letter advice cage above";

const MULTI_MD1_ORIGIN: &[&str] = &[
    "md1fypsppspqfm67zzqqvpeyy9p0p9cdzxlr9pj9qps4cyst7a0v9dqxjg406uy6llxvywv2quayl75ggm8hqt",
    "md1fypsppsvlcu6jsa6jdz69hy75rzqkjnyqzvheytfy9mjcppayp9n2lz42cd7ju7kdqa6uzgezf9vwalc8nl",
    "md1fypspps3j7vgkacswhccv5tpx2s4feevxsd3mta6k0mpf2dw678vsdfa3pu0hswnvwhxsktnd8g86jpq9km",
    "md1fypsppsu5lw2qhcnezhhjv6cjmcqmu328ay376tgu7u2ty8elftecmq3qe7afhljyr8edgcxt4mqkq",
];
const MULTI_MK1: &[&str] = &[
    "mk1qpfw9tpqqspyhz46hd9c4w4mhp5gmug8qjyty85qm48l29lwhkzksrfy2hawzd0lnxz8x9qnlrn22rh2f5tgkun6svgz6ekwcd6phwst0a5u",
    "mk1qpfw9tpp9xgqye0jgkjgth9szr6gztx47924sma9eav6pm4cye0xytwug8tuvx29stjumlc4fwgpyujvu593",
    "mk1qpfw92pqqspyhz46hd9c4w4m9pj9qps8qjyty8k08pch6zv4p2nnjcdqmrkhm4vlkzj56a4uweq6nmzrcl0qaxcawdpw2er454nss4mwzzcr",
    "mk1qpfw92pp8mjs97y7g4aunxkyk7qxly23lfy0kj688hzjep70627wxcygx0h2dlu36g9z6qcur7zmg7rpd6h6",
];
const MULTI_MS1: &[&str] = &[
    "ms10entrsqplh7lml0alh7lml0alh7lml0als5cclar2zmksh6",
    "ms10entrsqzqgpqyqszqgpqyqszqgpqyqszqqlfm7mep84hunu",
];

// ── FROZEN: `mnemonic bundle --descriptor wsh(pk([b8688df1/48'/0'/0']XPUB/<0;1>/*))` ──
const SS_XPUB: &str = "xpub6CETL9tkmWBQkYmxxxGAEzVZNHCZLx24pj58FYqe41qhXgyAcabP9iyXaVCJXcZWcVbzttVPdoJpJvYNfBnQeFcunvuxcsKKAwxMw6S5S7s";
const SS_DESCRIPTOR: &str =
    "wsh(pk([b8688df1/48'/0'/0']xpub6CETL9tkmWBQkYmxxxGAEzVZNHCZLx24pj58FYqe41qhXgyAcabP9iyXaVCJXcZWcVbzttVPdoJpJvYNfBnQeFcunvuxcsKKAwxMw6S5S7s/<0;1>/*))";
const SS_MD1_ORIGIN: &[&str] = &[
    "md1f9xlxpqpqpmvyyyqqcy2pdqhp5gmug4gy80cpxatjnpdtxhjvyuds54ar44wuc0a34",
    "md1f9xlxpq036ekkrhtkv6grq7qcua7ej7xusqaaq2qptxulyg808qnqjq8s570kd4kkd",
    "md1f9xlxpqsz3h36nf43a3dytlcf6saj9lwz9gc9uag7ce95hlcqu95t5qpd0qs94",
];
const SS_MK1: &[&str] = &[
    "mk1qp0a7spqqsqh7lgtujux3r03lcpmpqyqsqygpqyqsqygpqyqsqyqfz9jre8sgup6wlszd6h9xz6kd0ycfcm78tx6cwawc4mc8w8mc22yh64y",
    "mk1qp0a7sppe5sxpup3eman9udeqpm6q5qzkde7gsw7wpxq2x782dxk8k9530lp82rkghacg4rqhn4rmrykjll8tvxr6cfeg0w6p8t55lmdv",
];

/// Decode a keyed card WITH origin and re-emit it with the origin ELIDED
/// (`path_decl → Shared([])`). For a `canonical_origin == None` tree this yields
/// a keyed DEAD card (`unresolved_origin_indices()` non-empty). Returns the
/// chunk-form strings.
fn elide(md1_with_origin: &[&str]) -> Vec<String> {
    let mut d = md_codec::chunk::reassemble(md1_with_origin).expect("decode keyed card WITH origin");
    d.path_decl.paths = PathDeclPaths::Shared(OriginPath { components: vec![] });
    md_codec::chunk::split(&d).expect("split elided (dead) card")
}

/// Elide + doctor: swap chunk[1] for one from a foreign descriptor (mutated
/// pubkey → different content-id) to trip the cross-chunk oracle.
fn elide_doctored(md1_with_origin: &[&str]) -> Vec<String> {
    let mut foreign = md_codec::chunk::reassemble(md1_with_origin).expect("decode");
    if let Some(pks) = foreign.tlv.pubkeys.as_mut() {
        pks[0].1[10] ^= 0x01;
    }
    foreign.path_decl.paths = PathDeclPaths::Shared(OriginPath { components: vec![] });
    let foreign_chunks = md_codec::chunk::split(&foreign).expect("split foreign");
    let mut clean = elide(md1_with_origin);
    assert!(clean.len() >= 2 && foreign_chunks.len() >= 2, "need multi-chunk");
    clean[1] = foreign_chunks[1].clone();
    clean
}

fn push_md1(args: &mut Vec<String>, md1: &[String]) {
    for c in md1 {
        args.push("--md1".into());
        args.push(c.clone());
    }
}
fn push_flag(args: &mut Vec<String>, flag: &str, vals: &[&str]) {
    for v in vals {
        args.push(flag.into());
        args.push((*v).into());
    }
}

/// Base args for the tr-sortedmulti-a full-multisig verify (ms1+mk1 supplied).
fn multi_base() -> Vec<String> {
    let mut a = vec![
        "verify-bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "tr-sortedmulti-a".into(),
        "--threshold".into(),
        "2".into(),
        "--slot".into(),
        format!("@0.phrase={P1}"),
        "--slot".into(),
        format!("@1.phrase={P2}"),
    ];
    push_flag(&mut a, "--ms1", MULTI_MS1);
    push_flag(&mut a, "--mk1", MULTI_MK1);
    a
}

fn result_line(stdout: &[u8]) -> String {
    String::from_utf8_lossy(stdout)
        .lines()
        .find_map(|l| l.strip_prefix("result: ").map(str::to_string))
        .unwrap_or_default()
}

// ── funds-critical negative: ok → partial, NEVER a false-pass (template/:2450) ──

#[test]
fn verify_bundle_multisig_elided_md1_is_partial_exit_4() {
    let dead = elide(MULTI_MD1_ORIGIN);
    // BOTH auto-repair modes: default (M-3b clean fall-through — no exit-5
    // short-circuit) AND --no-auto-repair.
    for extra in [vec![], vec!["--no-auto-repair".to_string()]] {
        let mut args = multi_base();
        push_md1(&mut args, &dead);
        args.extend(extra.clone());
        let out = Command::cargo_bin("mnemonic")
            .unwrap()
            .args(&args)
            .env("MNEMONIC_FORCE_TTY", "1")
            .output()
            .unwrap();
        assert_eq!(
            out.status.code(),
            Some(4),
            "elided (dead) supplied md1 must verdict `partial`/exit 4 (extra={extra:?}); \
             stderr={:?}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(
            result_line(&out.stdout),
            "partial",
            "verdict must be `partial` (extra={extra:?})"
        );
    }
}

// ── BOUNDARY (RED-proof): the SAME bundle with the ORIGINAL md1 stays ok/exit-0 ──

#[test]
fn verify_bundle_multisig_original_md1_stays_ok_exit_0() {
    let orig: Vec<String> = MULTI_MD1_ORIGIN.iter().map(|s| s.to_string()).collect();
    let mut args = multi_base();
    push_md1(&mut args, &orig);
    args.push("--no-auto-repair".into());
    let out = Command::cargo_bin("mnemonic").unwrap().args(&args).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "original bundle must stay ok/exit 0");
    assert_eq!(result_line(&out.stdout), "ok");
}

// ── mismatch beats partial (M-1 precedence) ─────────────────────────────────

#[test]
fn verify_bundle_mismatch_beats_partial() {
    // The dead md1 encodes a 2-of-2 policy; verify against --threshold 1 → the
    // expected tree (k=1) differs → md1_xpub_match fails → `mismatch` WINS even
    // though the supplied md1 is (also) partial.
    let dead = elide(MULTI_MD1_ORIGIN);
    let mut args = vec![
        "verify-bundle".to_string(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "tr-sortedmulti-a".into(),
        "--threshold".into(),
        "1".into(),
        "--slot".into(),
        format!("@0.phrase={P1}"),
        "--slot".into(),
        format!("@1.phrase={P2}"),
        "--no-auto-repair".into(),
    ];
    push_flag(&mut args, "--ms1", MULTI_MS1);
    push_flag(&mut args, "--mk1", MULTI_MK1);
    push_md1(&mut args, &dead);
    let out = Command::cargo_bin("mnemonic").unwrap().args(&args).output().unwrap();
    assert_eq!(
        result_line(&out.stdout),
        "mismatch",
        "a structurally-wrong dead card must read `mismatch`, not `partial`; stdout={}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert_eq!(out.status.code(), Some(4));
}

// ── funds-critical: doctored content-id dead card REJECTS (oracle intact) ────

#[test]
fn verify_bundle_doctored_content_id_dead_card_verdicts_mismatch_oracle_intact() {
    let doctored = elide_doctored(MULTI_MD1_ORIGIN);
    let mut args = multi_base();
    push_md1(&mut args, &doctored);
    args.push("--no-auto-repair".into());
    let out = Command::cargo_bin("mnemonic").unwrap().args(&args).output().unwrap();
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Funds-load-bearing: the cross-chunk content-id oracle stays enforced UNDER
    // partial. `elide_doctored` flips a pubkey byte in chunk[1] so its derived
    // content-id diverges from chunk[0] → `reassemble_with_opts(.., partial())`
    // rejects with `ChunkSetInconsistent` → the `md1_decode` check fails →
    // verdict `mismatch`/exit 4, NEVER a false `ok`/`partial` (the v0.86.0
    // aliasing funds-loss mode). Pin the EXACT funds-safe verdict — `!= "partial"`
    // alone would also pass on a false `ok`, the very mode we must exclude (R0
    // M-1). Asserting `md1_decode: fail` pins the ORACLE as the rejecting check,
    // so a regression that disabled the oracle but kept some other failing check
    // would not silently satisfy this test (R0 M-2; oracle-in-isolation is also
    // covered at the md-codec layer by descriptor-mnemonic's partial_decode tests).
    assert_eq!(
        result_line(&out.stdout),
        "mismatch",
        "a doctored-content-id dead card must verdict `mismatch` (oracle intact), never `ok`/`partial`; stdout={stdout}"
    );
    assert_eq!(
        out.status.code(),
        Some(4),
        "doctored dead card must exit 4 (mismatch verdict); stdout={stdout}"
    );
    assert!(
        stdout
            .lines()
            .any(|l| l.starts_with("md1_decode:") && l.contains("fail")),
        "the content-id oracle must surface as a failing `md1_decode` check; stdout={stdout}"
    );
}

// ── --json partial field ────────────────────────────────────────────────────

#[test]
fn verify_bundle_json_partial_field_present_on_partial() {
    let dead = elide(MULTI_MD1_ORIGIN);
    let mut args = multi_base();
    args.push("--json".into());
    push_md1(&mut args, &dead);
    args.push("--no-auto-repair".into());
    let out = Command::cargo_bin("mnemonic").unwrap().args(&args).output().unwrap();
    assert_eq!(out.status.code(), Some(4));
    let stdout = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["schema_version"], "4");
    assert_eq!(v["result"], "partial");
    assert_eq!(v["partial"]["reason"], "missing_explicit_origin");
    assert!(!v["partial"]["unresolved_indices"].as_array().unwrap().is_empty());
}

#[test]
fn verify_bundle_json_no_partial_field_on_ok() {
    let orig: Vec<String> = MULTI_MD1_ORIGIN.iter().map(|s| s.to_string()).collect();
    let mut args = multi_base();
    args.push("--json".into());
    push_md1(&mut args, &orig);
    args.push("--no-auto-repair".into());
    let out = Command::cargo_bin("mnemonic").unwrap().args(&args).output().unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(v["result"], "ok");
    assert!(v.get("partial").is_none(), "ok verdict must NOT carry `partial`; got {v}");
}

// ── descriptor mode (single-sig, :3045 / emit_md1_checks + verify_emit_from_expected) ──

#[test]
fn verify_bundle_descriptor_singlesig_elided_md1_is_partial_exit_4() {
    let dead = elide(SS_MD1_ORIGIN);
    let mut args = vec![
        "verify-bundle".to_string(),
        "--network".into(),
        "mainnet".into(),
        "--descriptor".into(),
        SS_DESCRIPTOR.into(),
        "--slot".into(),
        format!("@0.xpub={SS_XPUB}"),
        "--no-auto-repair".into(),
    ];
    push_flag(&mut args, "--mk1", SS_MK1);
    push_md1(&mut args, &dead);
    let out = Command::cargo_bin("mnemonic").unwrap().args(&args).output().unwrap();
    assert_eq!(
        out.status.code(),
        Some(4),
        "descriptor-mode elided single-sig md1 → partial/exit 4; stderr={:?}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(result_line(&out.stdout), "partial");
}

#[test]
fn verify_bundle_descriptor_singlesig_original_md1_stays_ok() {
    let orig: Vec<String> = SS_MD1_ORIGIN.iter().map(|s| s.to_string()).collect();
    let mut args = vec![
        "verify-bundle".to_string(),
        "--network".into(),
        "mainnet".into(),
        "--descriptor".into(),
        SS_DESCRIPTOR.into(),
        "--slot".into(),
        format!("@0.xpub={SS_XPUB}"),
        "--no-auto-repair".into(),
    ];
    push_flag(&mut args, "--mk1", SS_MK1);
    push_md1(&mut args, &orig);
    let out = Command::cargo_bin("mnemonic").unwrap().args(&args).output().unwrap();
    assert_eq!(out.status.code(), Some(0), "descriptor-mode original → ok/exit 0");
    assert_eq!(result_line(&out.stdout), "ok");
}

// ── verify↔restore parity: the same dead card `restore --md1` refuses ───────

#[test]
fn restore_md1_refuses_the_same_dead_card() {
    let dead = elide(SS_MD1_ORIGIN);
    let mut args = vec!["restore".to_string()];
    push_md1(&mut args, &dead);
    args.push("--no-auto-repair".into());
    let out = Command::cargo_bin("mnemonic").unwrap().args(&args).output().unwrap();
    assert_ne!(
        out.status.code(),
        Some(0),
        "restore --md1 must REFUSE a dead card (strict; no partial); stderr={:?}",
        String::from_utf8_lossy(&out.stderr)
    );
}
