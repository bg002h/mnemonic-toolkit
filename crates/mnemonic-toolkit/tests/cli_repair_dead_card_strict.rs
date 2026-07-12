//! P2.4 (pathless/dead-card partial-decode) — `repair` STAYS STRICT.
//!
//! Per `design/IMPLEMENTATION_PLAN_pathless_partial_decode.md` §P2.4 +
//! `design/SPEC_pathless_partial_decode.md` (per-command table): `repair` does
//! NOT opt into partial-decode. A `canonical_origin == None` md1 card whose
//! origin is elided-and-unresolvable (a "dead card") is UN-repairable — its
//! strict `decode_with_correction` oracle rejects the post-correction decode →
//! exit 2 (fail-closed, UNCHANGED). Partial-repair of a corrupted pathless card
//! is the explicit FOLLOWUP `repair-corrupted-pathless-card-partial`, NOT built
//! here.

#![allow(missing_docs)]

use assert_cmd::Command;
use md_codec::origin_path::{OriginPath, PathDeclPaths};

/// FROZEN keyed cards WITH origin (from `mnemonic bundle`), elided in-crate to
/// dead cards. Single-sig `wsh(pk)` (`m/48'/0'/0'`) + multisig
/// `tr(sortedmulti_a)` (`m/87'/0'/0'`).
const SS_MD1_ORIGIN: &[&str] = &[
    "md1f9xlxpqpqpmvyyyqqcy2pdqhp5gmug4gy80cpxatjnpdtxhjvyuds54ar44wuc0a34",
    "md1f9xlxpq036ekkrhtkv6grq7qcua7ej7xusqaaq2qptxulyg808qnqjq8s570kd4kkd",
    "md1f9xlxpqsz3h36nf43a3dytlcf6saj9lwz9gc9uag7ce95hlcqu95t5qpd0qs94",
];
const MULTI_MD1_ORIGIN: &[&str] = &[
    "md1fypsppspqfm67zzqqvpeyy9p0p9cdzxlr9pj9qps4cyst7a0v9dqxjg406uy6llxvywv2quayl75ggm8hqt",
    "md1fypsppsvlcu6jsa6jdz69hy75rzqkjnyqzvheytfy9mjcppayp9n2lz42cd7ju7kdqa6uzgezf9vwalc8nl",
    "md1fypspps3j7vgkacswhccv5tpx2s4feevxsd3mta6k0mpf2dw678vsdfa3pu0hswnvwhxsktnd8g86jpq9km",
    "md1fypsppsu5lw2qhcnezhhjv6cjmcqmu328ay376tgu7u2ty8elftecmq3qe7afhljyr8edgcxt4mqkq",
];

fn dead(md1_with_origin: &[&str]) -> Vec<String> {
    let mut d = md_codec::chunk::reassemble(md1_with_origin).expect("decode keyed card");
    d.path_decl.paths = PathDeclPaths::Shared(OriginPath { components: vec![] });
    md_codec::chunk::split(&d).expect("split dead card")
}

fn repair_exit(dead_chunks: &[String]) -> Option<i32> {
    let mut args = vec!["repair".to_string()];
    for c in dead_chunks {
        args.push("--md1".into());
        args.push(c.clone());
    }
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .output()
        .unwrap()
        .status
        .code()
}

#[test]
fn repair_untouched_singlesig_dead_card_exits_2() {
    assert_eq!(
        repair_exit(&dead(SS_MD1_ORIGIN)),
        Some(2),
        "an untouched single-sig dead card is UN-repairable → exit 2 (strict, unchanged)"
    );
}

#[test]
fn repair_untouched_multisig_dead_card_exits_2() {
    assert_eq!(
        repair_exit(&dead(MULTI_MD1_ORIGIN)),
        Some(2),
        "an untouched multisig dead card is UN-repairable → exit 2 (strict, unchanged)"
    );
}
