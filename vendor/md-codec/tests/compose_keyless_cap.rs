//! A policy holds AT MOST ONE key-less spend path (spec §4e).
//!
//! WHY THE RULE LIVES HERE AND NOT ONLY IN `md compose`. Paths chain
//! right-leaning as `or_i(P, rest)` (`or_d` under a bare-multi head), and
//! `or_i(l, r)` is non-malleable only if `l.safe || r.safe` — `safe` meaning
//! "every satisfaction needs a signature"
//! (`vendor/miniscript/src/miniscript/types/malleability.rs`). A key-less path
//! is never safe, so with two of them SOME `or_i` on the spine has two unsafe
//! arms and the whole script is malleable. Bitcoin Core says the same thing
//! (`… is not sane: malleable witnesses exist`) and refuses the import, so the
//! wallet is unusable by any coordinator even though its keyed path looks fine.
//!
//! Until this rule existed the primary reached that verdict only by re-parsing
//! its own output in `md compose` (md-cli F-600). The device's Go port has no
//! rust-miniscript and cannot read back, so a template the primary would never
//! mint reached STEEL — the composer fable review r0 lens 1, C-1. The rule
//! belongs where the knowledge is: in `validate()`, which `md/compose.go`
//! ports.
//!
//! A TIMELOCK DOES NOT HELP. `and_v(v:sha256(H),older(5))` is still unsafe:
//! `older` needs no signature either. Measured over 859 wsh path lists of 2 to
//! 4 paths built from five keyed and four key-less atoms (`md compose
//! --experimental`, md-cli 0.16.2): every one of the 504 lists with two or more
//! key-less paths was refused after lowering, and all 355 with at most one were
//! emitted. Zero counterexamples — the rule is exact, not a heuristic.

use md_codec::compose::UnspendableKind;
use md_codec::compose::{
    ComposeError, HashKind, HashLock, KeySet, Lock, PathList, SpendPath, Wrapper, compose,
    template_with_origins, validate,
};

/// The two digests the C-1 report drove through the device path.
const H1: HashLock = HashLock::new(
    HashKind::Sha256,
    hexlit(b"cc6a74520f526a6135a4eae180547ae73648254ad1ae90bad93520402b0a123d"),
);
const H2: HashLock = HashLock::new(
    HashKind::Sha256,
    hexlit(b"5a23c169bc7e6f2569a3d74f58118576f005118603473a7069e22ab544484a3d"),
);

/// `const`-usable hex: the vector file stores these as text and the test must
/// not drift from it by hand-transcribing bytes.
const fn hexlit(s: &[u8; 64]) -> [u8; 32] {
    const fn nib(c: u8) -> u8 {
        match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            _ => panic!("digest hex is lowercase 0-9a-f"),
        }
    }
    let mut out = [0u8; 32];
    let mut i = 0;
    while i < 32 {
        out[i] = (nib(s[2 * i]) << 4) | nib(s[2 * i + 1]);
        i += 1;
    }
    out
}

fn keyed(k: u8, n: u8) -> SpendPath {
    SpendPath {
        keys: Some(KeySet { k, n, sorted: true }),
        hash: None,
        lock: None,
    }
}

fn keyless(h: HashLock, lock: Option<Lock>) -> SpendPath {
    SpendPath {
        keys: None,
        hash: Some(h),
        lock,
    }
}

fn wsh(paths: Vec<SpendPath>) -> PathList {
    PathList {
        wrapper: Wrapper::Wsh,
        paths,
    }
}

#[test]
fn validate_refuses_a_second_keyless_path() {
    let err = validate(&wsh(vec![
        keyed(1, 1),
        keyless(H1, None),
        keyless(H2, None),
    ]))
    .unwrap_err();
    assert_eq!(
        err,
        ComposeError::TooManyKeylessPaths {
            first: 1,
            second: 2
        }
    );
    // `compose` is the entry point the port calls; it must refuse too, not just
    // the validator underneath it.
    assert_eq!(
        compose(
            &wsh(vec![keyed(1, 1), keyless(H1, None), keyless(H2, None)]),
            UnspendableKind::Nums
        )
        .unwrap_err(),
        err
    );
}

#[test]
fn a_timelock_on_the_second_keyless_path_does_not_rescue_it() {
    let err = validate(&wsh(vec![
        keyed(1, 1),
        keyless(H1, None),
        keyless(H2, Some(Lock::OlderBlocks(5))),
    ]))
    .unwrap_err();
    assert_eq!(
        err,
        ComposeError::TooManyKeylessPaths {
            first: 1,
            second: 2
        }
    );
}

#[test]
fn the_two_keyless_paths_need_not_be_adjacent() {
    // `or_i(K1, or_i(keyed, K3))`: the inner arm is non-malleable, but it is
    // not SAFE, so the outer `or_i` has two unsafe arms all the same.
    let err = validate(&wsh(vec![
        keyless(H1, None),
        keyed(2, 3),
        keyless(H2, None),
    ]))
    .unwrap_err();
    assert_eq!(
        err,
        ComposeError::TooManyKeylessPaths {
            first: 0,
            second: 2
        }
    );
}

#[test]
fn the_refusal_says_a_timelock_does_not_help() {
    let err = validate(&wsh(vec![
        keyed(1, 1),
        keyless(H1, None),
        keyless(H2, None),
    ]))
    .unwrap_err();
    let msg = err.to_string();
    // 1-based path numbers, as every other compose message uses.
    assert!(msg.contains("paths 2 and 3"), "{msg}");
    assert!(msg.contains("malleable"), "{msg}");
    // The F-600 hint said "give one of them a key, a timelock, or fold them
    // into one path" -- and a timelock is exactly what does NOT work. A remedy
    // that does not work is worse than no remedy.
    assert!(msg.contains("a timelock does not help"), "{msg}");
    assert!(msg.contains("key"), "{msg}");
}

#[test]
fn one_keyless_path_is_still_admitted() {
    // The control. A rule that refused every key-less path would pass every
    // assertion above and delete the EXPERIMENTAL feature.
    let list = wsh(vec![keyless(H1, Some(Lock::OlderBlocks(5))), keyed(2, 3)]);
    assert_eq!(validate(&list).unwrap(), 3);
    let c = compose(&list, UnspendableKind::Nums).expect("one key-less path composes");
    assert_eq!(
        template_with_origins(&c).unwrap(),
        "wsh(or_i(and_v(v:sha256(cc6a74520f526a6135a4eae180547ae73648254ad1ae90bad93520402b0a123d),older(5)),multi(2,@0/48'/0'/0'/2'/<0;1>/*,@1/48'/0'/1'/2'/<0;1>/*,@2/48'/0'/2'/2'/<0;1>/*)))"
    );
}

#[test]
fn a_keyless_path_under_tr_still_reports_the_tr_rule() {
    // Order matters for the operator: under `tr` the FIRST key-less path is
    // already the defect, and "use wsh, or add a key" is the actionable line.
    // Reporting the cap instead would name a rule that is not the one in force.
    let err = validate(&PathList {
        wrapper: Wrapper::Tr,
        paths: vec![keyed(2, 3), keyless(H1, None), keyless(H2, None)],
    })
    .unwrap_err();
    assert_eq!(err, ComposeError::KeylessUnderTr { path: 1 });
}

#[test]
fn a_policy_with_no_keyed_path_still_reports_that() {
    // `[K, K]` breaks two rules. BIP-388 l.191 -- a wallet needs a key -- is
    // the more fundamental one and stays first.
    let err = validate(&wsh(vec![keyless(H1, None), keyless(H2, None)])).unwrap_err();
    assert_eq!(err, ComposeError::NoKeyedPath);
}

#[test]
fn a_legacy_wrapper_with_two_keyless_paths_still_reports_the_legacy_rule() {
    // Fold-A review M-1: the cap's remedy ("fold them into one path") leaves
    // `sh` with two paths, which is still not one sorted multisig, so the
    // legacy rule -- whose remedy "use wsh or tr" works -- must come first.
    let err = validate(&PathList {
        wrapper: Wrapper::Sh,
        paths: vec![keyed(2, 3), keyless(H1, None), keyless(H2, None)],
    })
    .unwrap_err();
    assert_eq!(err, ComposeError::LegacyWrapperShape);
}

#[test]
fn a_policy_over_the_slot_cap_still_reports_the_slot_cap() {
    // Same argument: 36 slots folded into fewer paths are still 36 slots.
    let mut paths = vec![keyed(9, 9), keyed(9, 9), keyed(9, 9), keyed(9, 9)];
    paths.push(keyless(H1, None));
    paths.push(keyless(H2, None));
    let err = validate(&wsh(paths)).unwrap_err();
    assert!(
        matches!(err, ComposeError::TooManySlots { got: 36, .. }),
        "{err:?}"
    );
}

// ---- the conformance vector ------------------------------------------------
//
// `vectors/compose_refusal_keyless_cap.json` is what the Go port vendors and
// conforms to (`scripts/vendor-compose-vectors.sh` in the seedhammer fork picks
// up every `compose_*` file in this directory). It is HAND-WRITTEN -- `md
// vectors` exports admitted policies, and a refusal has no card, no template
// and no addresses to export -- so this test is what keeps it from drifting
// away from the code it describes: every case below is DRIVEN, not read.

const VECTOR: &str = include_str!("vectors/compose_refusal_keyless_cap.json");

fn hash_from(v: &serde_json::Value) -> HashLock {
    let kind = match v["kind"].as_str().expect("hash.kind") {
        "sha256" => HashKind::Sha256,
        "hash256" => HashKind::Hash256,
        "ripemd160" => HashKind::Ripemd160,
        "hash160" => HashKind::Hash160,
        other => panic!("unknown hash kind {other}"),
    };
    let hex = v["digest"].as_str().expect("hash.digest");
    assert_eq!(
        hex.len(),
        kind.digest_len() * 2,
        "digest width must match the kind"
    );
    let mut digest = [0u8; 32];
    for (i, b) in digest.iter_mut().take(kind.digest_len()).enumerate() {
        *b = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).expect("lowercase hex");
    }
    HashLock::new(kind, digest)
}

fn lock_from(v: &serde_json::Value) -> Lock {
    let n = v["value"].as_u64().expect("lock.value");
    match v["kind"].as_str().expect("lock.kind") {
        "older_blocks" => Lock::OlderBlocks(u16::try_from(n).expect("u16")),
        "older_units" => Lock::OlderUnits(u16::try_from(n).expect("u16")),
        "after_height" => Lock::AfterHeight(u32::try_from(n).expect("u32")),
        "after_time" => Lock::AfterTime(u32::try_from(n).expect("u32")),
        other => panic!("unknown lock kind {other}"),
    }
}

fn list_from(case: &serde_json::Value) -> PathList {
    let wrapper = match case["wrapper"].as_str().expect("wrapper") {
        "wsh" => Wrapper::Wsh,
        "tr" => Wrapper::Tr,
        "sh-wsh" => Wrapper::ShWsh,
        "sh" => Wrapper::Sh,
        other => panic!("unknown wrapper {other}"),
    };
    let paths = case["paths"]
        .as_array()
        .expect("paths")
        .iter()
        .map(|p| SpendPath {
            keys: p["keys"].as_object().map(|k| KeySet {
                k: u8::try_from(k["k"].as_u64().expect("k")).expect("u8"),
                n: u8::try_from(k["n"].as_u64().expect("n")).expect("u8"),
                sorted: k["sorted"].as_bool().expect("sorted"),
            }),
            hash: p.get("hash").filter(|h| !h.is_null()).map(hash_from),
            lock: p.get("lock").filter(|l| !l.is_null()).map(lock_from),
        })
        .collect();
    PathList { wrapper, paths }
}

#[test]
fn every_case_in_the_conformance_vector_behaves_as_recorded() {
    let doc: serde_json::Value = serde_json::from_str(VECTOR).expect("the vector file parses");
    let cases = doc["cases"].as_array().expect("cases");
    let (mut refused, mut admitted) = (0usize, 0usize);
    for case in cases {
        let name = case["name"].as_str().expect("name");
        let list = list_from(case);
        match case["expect"].as_str().expect("expect") {
            "refused" => {
                refused += 1;
                let err = validate(&list)
                    .map(|n| format!("{n} slots"))
                    .expect_err(name);
                let want = &case["error"];
                let idx = |k: &str| usize::try_from(want[k].as_u64().expect(k)).unwrap();
                let expected = match want["kind"].as_str().expect("kind") {
                    "TooManyKeylessPaths" => ComposeError::TooManyKeylessPaths {
                        first: idx("first"),
                        second: idx("second"),
                    },
                    // The precedence cases (fold-A review N-1): a port that
                    // orders the cap ahead of these rules still refuses, but
                    // names a rule whose remedy does not work.
                    "KeylessUnderTr" => ComposeError::KeylessUnderTr { path: idx("path") },
                    "NoKeyedPath" => ComposeError::NoKeyedPath,
                    "LegacyWrapperShape" => ComposeError::LegacyWrapperShape,
                    "TooManySlots" => ComposeError::TooManySlots {
                        got: idx("got"),
                        max: u8::try_from(idx("max")).unwrap(),
                    },
                    other => panic!("{name}: unknown error kind {other}"),
                };
                assert_eq!(err, expected, "{name}");
                if let Some(msg) = case["message"].as_str() {
                    assert_eq!(
                        err.to_string(),
                        msg,
                        "{name}: the operator-facing wording is normative too"
                    );
                }
            }
            "admitted" => {
                admitted += 1;
                let slots = validate(&list).unwrap_or_else(|e| panic!("{name}: {e}"));
                assert_eq!(
                    u64::try_from(slots).unwrap(),
                    case["slots"].as_u64().expect("slots"),
                    "{name}"
                );
                let c =
                    compose(&list, UnspendableKind::Nums).unwrap_or_else(|e| panic!("{name}: {e}"));
                assert_eq!(
                    template_with_origins(&c).unwrap(),
                    case["template"].as_str().expect("template"),
                    "{name}"
                );
            }
            other => panic!("{name}: unknown expect {other}"),
        }
    }
    // A vector file that lost its cases would pass every assertion above.
    assert!(
        refused >= 3,
        "the vector must carry the refusals: {refused}"
    );
    assert!(
        admitted >= 1,
        "the vector must carry an ADMITTED control, or a port that refuses every \
         key-less path conforms to it: {admitted}"
    );
}
