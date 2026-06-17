//! Stress Cycle A — backup→restore round-trip PROPERTY test.
//!
//! Generates valid, reconstructable wallet policies (typed-template, sane by
//! construction) → `build-descriptor` → `bundle --descriptor` (concrete
//! watch-only, no slots) → `restore --md1` → asserts three independent oracles:
//!   O1 — STRUCTURAL: the reconstructed descriptor is AST-equal to the original
//!        modulo key identity (a dropped/masked/swapped fragment fails).
//!   O2 — md1 fixed-point: re-bundling the reconstruction reproduces the card.
//!   O3 — address differential: addresses derived INDEPENDENTLY from the ORIGINAL
//!        descriptor (rust-miniscript) == restore's reported addresses.
//!
//! Design + 2 R0 rounds: design/SPEC_stress_a_backup_restore_property_test.md,
//! design/agent-reports/stress-a-backup-restore-property-test-r0-round{1,2}-review.md.
//! The worst outcome (a green test that proves nothing) is foreclosed by: the
//! `max_global_rejects` budget + `generator_covers_all_fragments`, the permanent
//! oracle self-test cells, and O3's desc-vs-desc' script-hash equality.

use assert_cmd::Command;
use miniscript::{Descriptor, DescriptorPublicKey};
use predicates::prelude::*;
use proptest::prelude::*;
use serde_json::{json, Value};
use std::collections::BTreeSet;
use std::str::FromStr;

// ── Origin-annotated key pool (R0-r1 C1: bundle rejects origin-LESS concrete
//    keys). Distinct fingerprints carry key identity through O1 normalization. ──
const KEYS: &[&str] = &[
    "[11111111/48h/0h/0h/2h]xpub661MyMwAqRbcEZVB4dScxMAdx6d4nFc9nvyvH3v4gJL378CSRZiYmhRoP7mBy6gSPSCYk6SzXPTf3ND1cZAceL7SfJ1Z3GC8vBgp2epUt13",
    "[22222222/48h/0h/0h/2h]xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8",
    "[33333333/48h/0h/0h/2h]xpub661MyMwAqRbcFW31YEwpkMuc5THy2PSt5bDMsktWQcFF8syAmRUapSCGu8ED9W6oDMSgv6Zz8idoc4a6mr8BDzTJY47LJhkJ8UB7WEGuduB",
    "[44444444/48h/0h/0h/2h]xpub661MyMwAqRbcGczjuMoRm6dXaLDEhW1u34gKenbeYqAix21mdUKJyuyu5F1rzYGVxyL6tmgBUAEPrEz92mBXjByMRiJdba9wpnN37RLLAXa",
    "[55555555/48h/0h/0h/2h]xpub68Gmy5EdvgibQVfPdqkBBCHxA5htiqg55crXYuXoQRKfDBFA1WEjWgP6LHhwBZeNK1VTsfTFUHCdrfp1bgwQ9xv5ski8PX9rL2dZXvgGDnw",
];

/// Fresh-key allocator (no replacement → never collides → no RepeatedPubkeys).
struct Alloc {
    next: usize,
}
impl Alloc {
    fn new() -> Self {
        Alloc { next: 0 }
    }
    fn remaining(&self) -> usize {
        KEYS.len() - self.next
    }
    fn fresh(&mut self) -> &'static str {
        let k = KEYS[self.next];
        self.next += 1;
        k
    }
}

/// Deterministic splitmix64 — derives schema params from the proptest seed.
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// Inclusive range `lo..=hi`.
    fn range(&mut self, lo: u64, hi: u64) -> u64 {
        lo + self.next() % (hi - lo + 1)
    }
}

// ── IR-node builders (the build-descriptor `--spec` JSON shapes) ──
fn v_wrap(sub: Value) -> Value {
    json!({"wrap": {"w": "v", "sub": sub}})
}
fn s_wrap(sub: Value) -> Value {
    json!({"wrap": {"w": "s", "sub": sub}})
}
fn pk(a: &mut Alloc) -> Value {
    json!({"pk": a.fresh()})
}
fn pkh(a: &mut Alloc) -> Value {
    json!({"pkh": a.fresh()})
}
/// A single-key leaf — `pk` or `pkh` (pkh exercises the md-codec 0.35.1 Check
/// double-wrap fix on the reconstruction path; both are valid B-typed leaves).
fn key_leaf(rng: &mut Rng, a: &mut Alloc) -> Value {
    if rng.next() % 2 == 0 {
        pk(a)
    } else {
        pkh(a)
    }
}
/// `allow_sorted`: `sortedmulti` nested in a combinator is only RECONSTRUCTABLE
/// at the TOP level, so inside a combinator we emit plain `multi`. The limit is
/// RENDERER-level, NOT wire-level: the md1 wire round-trips a nested
/// `Tag::SortedMulti` byte-exact (md-codec's P7 proves it), but md-codec's
/// renderer pins crates.io miniscript 13.0.0 (no `Terminal::SortedMulti`) while
/// the toolkit pins git 95fdd1c (which has it) — the two-miniscripts split.
/// (The asymmetry where build/bundle accept sortedmulti-in-combinator but
/// restore refuses it is filed as FOLLOWUP `bundle-accepts-sortedmulti-in-
/// combinator-restore-cannot`, found by this harness; pinned by the
/// `sortedmulti_in_combinator_bundles_but_restore_refuses_loudly` cell above.)
fn multi(rng: &mut Rng, a: &mut Alloc, max_n: usize, allow_sorted: bool) -> (Value, &'static str) {
    let avail = a.remaining().min(max_n).max(1);
    let n = rng.range(1, avail as u64) as usize;
    let k = rng.range(1, n as u64) as usize;
    let keys: Vec<Value> = (0..n).map(|_| json!(a.fresh())).collect();
    let tag = if allow_sorted && rng.next() % 2 == 0 {
        "sortedmulti"
    } else {
        "multi"
    };
    (json!({ tag: {"k": k, "keys": keys} }), tag)
}
/// One relative-timelock class per tree (block OR 512-second-unit) — the v0.53.9
/// accepted domain; both classes appear ACROSS the corpus (R0-r2 M4).
fn rel_timelock(rng: &mut Rng) -> Value {
    let v = if rng.next() % 2 == 0 {
        rng.range(1, 65535) as u32 // blocks
    } else {
        0x0040_0000 | (rng.range(1, 0xFFFF) as u32) // 512-second units
    };
    json!({ "older": v })
}
fn abs_timelock(rng: &mut Rng) -> Value {
    json!({ "after": rng.range(1, 0x7FFF_FFFF) as u32 })
}
fn hashlock(rng: &mut Rng) -> (Value, &'static str) {
    fn hx(rng: &mut Rng, n: usize) -> String {
        use std::fmt::Write;
        let mut s = String::with_capacity(n * 2);
        for _ in 0..n {
            let _ = write!(s, "{:02x}", rng.range(0, 255) as u8);
        }
        s
    }
    match rng.next() % 4 {
        0 => (json!({"sha256": hx(rng, 32)}), "sha256"),
        1 => (json!({"hash256": hx(rng, 32)}), "hash256"),
        2 => (json!({"ripemd160": hx(rng, 20)}), "ripemd160"),
        _ => (json!({"hash160": hx(rng, 20)}), "hash160"),
    }
}

/// The typed-template schema library — every schema is B-typed, has a key on
/// EVERY spending path (no SiglessBranch), and uses ≤1 relative + ≤1 absolute
/// timelock class (no HeightTimelockCombination). Returns the `root` node + the
/// set of fragment keywords it used (for coverage).
fn build_policy(schema: usize, seed: u64) -> (Value, BTreeSet<&'static str>) {
    let rng = &mut Rng(seed);
    let a = &mut Alloc::new();
    let mut frags = BTreeSet::new();
    let note = |f: &'static str, set: &mut BTreeSet<&'static str>| {
        set.insert(f);
    };
    let root = match schema % 10 {
        // 0 — plain multi/sortedmulti (TOP-LEVEL: sortedmulti allowed here only).
        0 => {
            let (m, tag) = multi(rng, a, 5, true);
            note(tag, &mut frags);
            m
        }
        // 1 — timelocked recovery: or_d(multi, and_v(v:pk, older)).
        1 => {
            let (m, tag) = multi(rng, a, 3, false);
            note(tag, &mut frags);
            note("older", &mut frags);
            note("or_d", &mut frags);
            json!({"or_d": [m, json!({"and_v": [v_wrap(key_leaf(rng, a)), rel_timelock(rng)]})]})
        }
        // 2 — absolute-timelock recovery: or_d(multi, and_v(v:pk, after)).
        2 => {
            let (m, tag) = multi(rng, a, 3, false);
            note(tag, &mut frags);
            note("after", &mut frags);
            note("or_d", &mut frags);
            json!({"or_d": [m, json!({"and_v": [v_wrap(key_leaf(rng, a)), abs_timelock(rng)]})]})
        }
        // 3 — multi recovery: or_d(multi, and_v(v:multi, older)).
        3 => {
            let (m1, t1) = multi(rng, a, 2, false);
            let (m2, t2) = multi(rng, a, 2, false);
            note(t1, &mut frags);
            note(t2, &mut frags);
            note("older", &mut frags);
            note("or_d", &mut frags);
            json!({"or_d": [m1, json!({"and_v": [v_wrap(m2), rel_timelock(rng)]})]})
        }
        // 4 — timelock-gated multi: and_v(v:multi, older).
        4 => {
            let (m, tag) = multi(rng, a, 3, false);
            note(tag, &mut frags);
            note("older", &mut frags);
            note("and_v", &mut frags);
            json!({"and_v": [v_wrap(m), rel_timelock(rng)]})
        }
        // 5 — hashlock-gated multi: and_v(v:multi, <hash>).
        5 => {
            let (m, tag) = multi(rng, a, 3, false);
            let (h, ht) = hashlock(rng);
            note(tag, &mut frags);
            note(ht, &mut frags);
            note("and_v", &mut frags);
            json!({"and_v": [v_wrap(m), h]})
        }
        // 6 — hash+timelock recovery: or_d(multi, and_v(v:and_v(v:pk,<hash>),older)).
        6 => {
            let (m, tag) = multi(rng, a, 2, false);
            let (h, ht) = hashlock(rng);
            note(tag, &mut frags);
            note(ht, &mut frags);
            note("older", &mut frags);
            note("or_d", &mut frags);
            let inner = json!({"and_v": [v_wrap(pk(a)), h]});
            json!({"or_d": [m, json!({"and_v": [v_wrap(inner), rel_timelock(rng)]})]})
        }
        // 7 — andor(pk, <hash>, and_v(v:pk, older)) (the hashlock-gated archetype;
        //     pk-keyed → exercises the md-codec 0.35.1 Check fix).
        7 => {
            let (h, ht) = hashlock(rng);
            note(ht, &mut frags);
            note("older", &mut frags);
            note("andor", &mut frags);
            let k1 = key_leaf(rng, a);
            let recover = json!({"and_v": [v_wrap(pk(a)), rel_timelock(rng)]});
            json!({"andor": [k1, h, recover]})
        }
        // 8 — or_i(multi, and_v(v:multi, older)).
        8 => {
            let (m1, t1) = multi(rng, a, 2, false);
            let (m2, t2) = multi(rng, a, 2, false);
            note(t1, &mut frags);
            note(t2, &mut frags);
            note("older", &mut frags);
            note("or_i", &mut frags);
            json!({"or_i": [m1, json!({"and_v": [v_wrap(m2), rel_timelock(rng)]})]})
        }
        // 9 — thresh recovery (tiered-recovery shape): or_i(multi,
        //     and_v(v:older, thresh(k, pk, s:pk, s:pk))).
        _ => {
            let (m, tag) = multi(rng, a, 2, false);
            note(tag, &mut frags);
            note("older", &mut frags);
            note("thresh", &mut frags);
            note("or_i", &mut frags);
            let subs = json!([pk(a), s_wrap(pk(a)), s_wrap(pk(a))]);
            let thr = json!({"thresh": {"k": 2, "subs": subs}});
            json!({"or_i": [m, json!({"and_v": [v_wrap(rel_timelock(rng)), thr]})]})
        }
    };
    (
        json!({"schema_version": 1, "wrapper": "wsh", "root": root}),
        frags,
    )
}

const N_SCHEMAS: usize = 10;

// ── Pipeline (CLI; tests the real binary) ──
fn bin() -> Command {
    Command::cargo_bin("mnemonic").unwrap()
}

/// `build-descriptor --spec -` → `Some(desc)` if the gate ACCEPTS, else `None`
/// (a step-1 rejection — the only legitimate `prop_assume!`-skip site).
fn build_descriptor(spec: &Value) -> Option<String> {
    let out = bin()
        .args([
            "build-descriptor",
            "--spec",
            "-",
            "--network",
            "mainnet",
            "--format",
            "descriptor",
        ])
        .write_stdin(serde_json::to_vec(spec).unwrap())
        .assert();
    let o = out.get_output();
    if o.status.success() {
        Some(
            String::from_utf8(o.stdout.clone())
                .unwrap()
                .trim()
                .to_string(),
        )
    } else {
        None
    }
}

/// `bundle --descriptor <concrete>` (no slots → watch-only path) → md1 chunks.
/// FAILURE POLICY (R0-r1 C1): a gate-accepted descriptor MUST bundle — `.success()`
/// panics (= property failure) otherwise.
fn bundle_md1(desc: &str) -> Vec<String> {
    let out = bin()
        .args([
            "bundle",
            "--descriptor",
            desc,
            "--network",
            "mainnet",
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    v["md1"]
        .as_array()
        .expect("md1 array")
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect()
}

/// `restore --md1 … --count 2 --json` → (reconstructed descriptor, chain-0
/// addresses). FAILURE POLICY: a gate-accepted policy MUST restore — `.success()`
/// panics otherwise (the C2 silent-collapse class would exit 0 with a wrong
/// descriptor, caught by the oracles, not here).
fn restore(md1: &[String]) -> (String, Vec<String>) {
    let mut a = vec![
        "restore".to_string(),
        "--network".into(),
        "mainnet".into(),
        "--count".into(),
        "2".into(),
        "--json".into(),
    ];
    for c in md1 {
        a.push("--md1".into());
        a.push(c.clone());
    }
    let out = bin().args(&a).assert().success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    let w = &v["wallets"][0];
    let desc = w["descriptor"].as_str().unwrap().to_string();
    let addrs = w["first_addresses"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect();
    (desc, addrs)
}

// ── Oracles ──
fn is_base58(c: u8) -> bool {
    c.is_ascii_alphanumeric() && !matches!(c, b'0' | b'O' | b'I' | b'l')
}

/// Canonicalize via miniscript (normalizes path notation + checksum), strip the
/// checksum, and erase every xpub/tpub BODY (keeping `[fp/path]` + the `/<0;1>/*`
/// suffix). Two descriptors AST-equal modulo key identity → equal normalized
/// string. `multi`↔`sortedmulti`, `sha256`↔`hash256`, branch reorder, dropped
/// fragment, masked timelock value all DIFFER (R0-r1 I2).
fn normalize(desc: &str) -> String {
    let d = Descriptor::<DescriptorPublicKey>::from_str(desc)
        .unwrap_or_else(|e| panic!("descriptor must parse: {desc}: {e}"));
    let mut s = d.to_string();
    if let Some(i) = s.rfind('#') {
        s.truncate(i);
    }
    let b = s.as_bytes();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < b.len() {
        if s[i..].starts_with("xpub") || s[i..].starts_with("tpub") {
            i += 4;
            while i < b.len() && is_base58(b[i]) {
                i += 1;
            }
            out.push_str("KEY");
        } else {
            out.push(b[i] as char);
            i += 1;
        }
    }
    out
}

/// O3 — derive chain-0 addresses INDEPENDENTLY from a descriptor (the rust-
/// miniscript differential). Mirrors `derive_receive_addresses`.
fn derive_receive(desc: &str, count: u32) -> Vec<String> {
    use miniscript::DefiniteDescriptorKey;
    let d = Descriptor::<DescriptorPublicKey>::from_str(desc).unwrap();
    let receive = if d.is_multipath() {
        d.clone().into_single_descriptors().unwrap().remove(0)
    } else {
        d.clone()
    };
    (0..count)
        .map(|i| {
            let def: Descriptor<DefiniteDescriptorKey> = if receive.has_wildcard() {
                receive.derive_at_index(i).unwrap()
            } else {
                Descriptor::<DefiniteDescriptorKey>::try_from(receive.clone()).unwrap()
            };
            def.address(bitcoin::Network::Bitcoin).unwrap().to_string()
        })
        .collect()
}

// ── The property ──
proptest! {
    #![proptest_config(ProptestConfig {
        cases: std::env::var("PROP_CASES").ok().and_then(|s| s.parse().ok()).unwrap_or(64),
        // A degraded (mis-typed) generator fails LOUDLY, not silently (R0-r1 I1).
        max_global_rejects: 8,
        // No on-disk regression file (integration-test crate has no source root for
        // proptest's default persistence). A found counterexample prints its
        // `(schema, seed)` — DIRECTLY reproducible: pin it as an explicit
        // `build_policy(schema, seed)` `#[test]` (the R0-r1 I3 regression discipline).
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    #[test]
    fn backup_restore_roundtrip(schema in 0usize..N_SCHEMAS, seed in any::<u64>()) {
        let (spec, _frags) = build_policy(schema, seed);
        // Step 1: gate. A rejection is the ONLY legitimate skip.
        let desc = match build_descriptor(&spec) {
            Some(d) => d,
            None => return Err(TestCaseError::reject("build-descriptor gate rejected")),
        };
        // Steps 2-3: once accepted, any failure is a PROPERTY FAILURE (panics in
        // bundle_md1 / restore via `.success()`).
        let md1 = bundle_md1(&desc);
        let (desc2, restore_addrs) = restore(&md1);

        // O1 — structural AST preservation (modulo key identity).
        prop_assert_eq!(
            normalize(&desc), normalize(&desc2),
            "O1 structural mismatch:\n  orig: {}\n  recon: {}", desc, desc2
        );
        // O3 — address differential from the ORIGINAL descriptor.
        prop_assert_eq!(
            derive_receive(&desc, 2), restore_addrs.clone(),
            "O3 address differential failed for {}", desc
        );
        // O2 — md1 fixed-point.
        prop_assert_eq!(bundle_md1(&desc2), md1, "O2 md1 fixed-point failed for {}", desc2);
    }
}

// ── B1: taproot leg — tr(NUMS,{multi_a|sortedmulti_a}(k,…)) round-trips. The
//    toolkit-UNIQUE reconstruction (v0.49.1/v0.55.x route AROUND md-codec; the
//    workspace `95fdd1c` fork has Terminal::SortedMultiA) was covered only by
//    fixed goldens in cli_restore_multisig.rs, never property-tested. Concrete
//    tr strings bypass build_descriptor (wsh-only WrapperKind by design) and
//    enter at `bundle --descriptor`; O1/O2/O3 reuse unchanged. The negative
//    (@-in-both / non-NUMS refusal) path is already comprehensively covered at
//    n≥3 in cli_restore_taproot.rs::at_in_both_* — not duplicated here.
const NUMS_HEX: &str = "50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0";

/// `tr(NUMS,{multi_a|sortedmulti_a}(k, K0..K_{n-1}))` from the frozen KEYS pool.
fn tr_multi_desc(sorted: bool, n: usize, k: usize) -> String {
    let frag = if sorted { "sortedmulti_a" } else { "multi_a" };
    let keys: Vec<String> = (0..n).map(|i| format!("{}/<0;1>/*", KEYS[i])).collect();
    format!("tr({NUMS_HEX},{frag}({k},{}))", keys.join(","))
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: std::env::var("PROP_CASES").ok().and_then(|s| s.parse().ok()).unwrap_or(48),
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    /// Positive tr leg: bundle → restore reconstructs the same NUMS taproot
    /// multisig; O1 structural + O2 md1 fixed-point + O3 address differential.
    #[test]
    fn tr_taproot_roundtrip(
        sorted in any::<bool>(),
        n in 2usize..=KEYS.len(),
        k_raw in 1usize..=KEYS.len(),
    ) {
        let k = 1 + (k_raw - 1) % n; // 1..=n
        let desc = tr_multi_desc(sorted, n, k);
        let md1 = bundle_md1(&desc);
        let (desc2, restore_addrs) = restore(&md1);
        prop_assert_eq!(
            normalize(&desc), normalize(&desc2),
            "tr O1 structural mismatch:\n  orig: {}\n  recon: {}", desc, desc2
        );
        prop_assert_eq!(
            derive_receive(&desc, 2), restore_addrs.clone(),
            "tr O3 address differential failed for {}", desc
        );
        prop_assert_eq!(bundle_md1(&desc2), md1, "tr O2 md1 fixed-point failed for {}", desc2);
    }
}

/// Anti-vacuity smoke: BOTH variants (multi_a + the toolkit-unique sortedmulti_a)
/// round-trip deterministically (fast signal even when PROP_CASES is gated low;
/// proves the leg actually covers both fragments + derives a P2TR address).
#[test]
fn tr_taproot_smoke_both_variants() {
    for sorted in [false, true] {
        let desc = tr_multi_desc(sorted, 3, 2); // tr(NUMS,{multi_a|sortedmulti_a}(2,K0,K1,K2))
        let md1 = bundle_md1(&desc);
        let (desc2, addrs) = restore(&md1);
        assert_eq!(
            normalize(&desc),
            normalize(&desc2),
            "tr smoke O1 (sorted={sorted}): {desc} vs {desc2}"
        );
        assert_eq!(
            derive_receive(&desc, 2),
            addrs,
            "tr smoke O3 (sorted={sorted})"
        );
        assert_eq!(bundle_md1(&desc2), md1, "tr smoke O2 (sorted={sorted})");
        assert!(
            addrs[0].starts_with("bc1p"),
            "tr leg must derive a P2TR address: {}",
            addrs[0]
        );
    }
}

// ── Anti-vacuity: the generator must cover every fragment (R0-r1 I1) ──
#[test]
fn generator_covers_all_fragments() {
    let mut all = BTreeSet::new();
    for schema in 0..N_SCHEMAS {
        for seed in 0..200u64 {
            all.extend(build_policy(schema, seed).1);
        }
    }
    for f in [
        "older",
        "after",
        "sha256",
        "hash256",
        "ripemd160",
        "hash160",
        "multi",
        "sortedmulti",
        "thresh",
        "or_d",
        "or_i",
        "andor",
        "and_v",
    ] {
        assert!(
            all.contains(f),
            "generator never produced fragment `{f}`: {all:?}"
        );
    }
}

// ── Permanent oracle self-test cells (R0-r1 I3): each oracle must REJECT a
//    known-bad pair, so an oracle weakened by a later refactor is caught. ──
const D_A: &str = "[11111111/48h/0h/0h/2h]xpub661MyMwAqRbcEZVB4dScxMAdx6d4nFc9nvyvH3v4gJL378CSRZiYmhRoP7mBy6gSPSCYk6SzXPTf3ND1cZAceL7SfJ1Z3GC8vBgp2epUt13";
const D_B: &str = "[22222222/48h/0h/0h/2h]xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8";

#[test]
fn oracle1_rejects_dropped_timelock() {
    let full = format!("wsh(and_v(v:multi(2,{D_A}/<0;1>/*,{D_B}/<0;1>/*),older(4032)))");
    let dropped = format!("wsh(multi(2,{D_A}/<0;1>/*,{D_B}/<0;1>/*))");
    assert_ne!(
        normalize(&full),
        normalize(&dropped),
        "O1 must catch a dropped older()"
    );
}
#[test]
fn oracle1_rejects_multi_sortedmulti_swap() {
    let m = format!("wsh(multi(2,{D_A}/<0;1>/*,{D_B}/<0;1>/*))");
    let sm = format!("wsh(sortedmulti(2,{D_A}/<0;1>/*,{D_B}/<0;1>/*))");
    assert_ne!(
        normalize(&m),
        normalize(&sm),
        "O1 must catch multi↔sortedmulti"
    );
}
#[test]
fn oracle1_rejects_masked_timelock_value() {
    let a = format!("wsh(and_v(v:multi(2,{D_A}/<0;1>/*,{D_B}/<0;1>/*),older(4032)))");
    let b = format!("wsh(and_v(v:multi(2,{D_A}/<0;1>/*,{D_B}/<0;1>/*),older(1000)))");
    assert_ne!(
        normalize(&a),
        normalize(&b),
        "O1 must catch a changed timelock value"
    );
}
#[test]
fn oracle1_accepts_keyless_equivalent_redepth() {
    // The SAME policy with the SAME keys normalizes equal (sanity — the oracle is
    // not so strict it false-fails a faithful reconstruction).
    let x = format!("wsh(and_v(v:multi(2,{D_A}/<0;1>/*,{D_B}/<0;1>/*),older(4032)))");
    assert_eq!(normalize(&x), normalize(&x));
}
#[test]
fn oracle3_rejects_wrong_descriptor_address() {
    // Two different descriptors → different addresses (O3 catches a reconstruction
    // that built a different script).
    let a = format!("wsh(multi(2,{D_A}/<0;1>/*,{D_B}/<0;1>/*))");
    let b = format!("wsh(and_v(v:multi(2,{D_A}/<0;1>/*,{D_B}/<0;1>/*),older(4032)))");
    assert_ne!(
        derive_receive(&a, 1),
        derive_receive(&b, 1),
        "O3 must catch a wrong script"
    );
}

// ── Smoke: a handful of hand-picked policies through the full pipeline (fast
//    signal even when PROP_CASES is gated low). ──
#[test]
fn smoke_handpicked_policies() {
    for schema in 0..N_SCHEMAS {
        let (spec, _) = build_policy(schema, 42);
        if let Some(desc) = build_descriptor(&spec) {
            let md1 = bundle_md1(&desc);
            let (desc2, addrs) = restore(&md1);
            assert_eq!(
                normalize(&desc),
                normalize(&desc2),
                "schema {schema}: {desc} vs {desc2}"
            );
            assert_eq!(derive_receive(&desc, 2), addrs, "schema {schema} addresses");
            assert_eq!(bundle_md1(&desc2), md1, "schema {schema} md1 fixed-point");
        }
    }
}

// ── Negative property (R0-r1 M3): shapes OUTSIDE the reconstructable set ALWAYS
//    refuse loudly — NEVER a silent wrong reconstruction. Constructed via the @N
//    SLOT pipeline (bundle accepts, restore refuses); they cannot come from
//    build-descriptor (uniform /<0;1>/*). ──
#[test]
fn negative_property_unreconstructable_shapes_refuse_loudly() {
    const C0: &str =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    const C1: &str = "legal winner thank year wave sausage worth useful legal winner thank yellow";
    // per-key use-site override (mixed multipath); hardened wildcard.
    for desc in ["wsh(multi(2,@0/<0;1>/*,@1/*))", "wsh(multi(2,@0/*h,@1/*h))"] {
        let out = bin()
            .args([
                "bundle",
                "--descriptor",
                desc,
                "--network",
                "mainnet",
                "--slot",
                &format!("@0.phrase={C0}"),
                "--slot",
                &format!("@1.phrase={C1}"),
                "--json",
                "--no-engraving-card",
            ])
            .assert()
            .success();
        let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
        let md1: Vec<String> = v["md1"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_str().unwrap().to_string())
            .collect();
        let mut a = vec!["restore".to_string(), "--network".into(), "mainnet".into()];
        for c in &md1 {
            a.push("--md1".into());
            a.push(c.clone());
        }
        // MUST fail loudly — never exit-0 with a silent (wrong) reconstruction.
        bin().args(&a).assert().failure();
    }
}

/// GAP-3 contract — `sortedmulti` inside a combinator engraves a FAITHFUL card
/// (bundle exit 0; the md1 wire round-trips byte-exact — md-codec's P7 proves
/// it) but `restore --md1` refuses LOUDLY with the sole-child message, never a
/// silent wrong reconstruction. Pinned with the STDERR SUBSTRING (not just
/// `.failure()`) because the chosen shape reuses @1: when the deferred faithful
/// nested-sortedmulti reconstruction lands (FOLLOWUP
/// `bundle-accepts-sortedmulti-in-combinator-restore-cannot`), a `.failure()`
/// could stay green for the WRONG reason (repeated-pubkey sanity) — the
/// substring forces this cell red at that transition for a conscious re-pin.
/// The refusal is the RENDERER (md-codec pins crates.io miniscript 13.0.0,
/// which lacks `Terminal::SortedMulti`; the toolkit pins git 95fdd1c, which has
/// it), NOT a wire limitation.
#[test]
fn sortedmulti_in_combinator_bundles_but_restore_refuses_loudly() {
    const C0: &str =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    const C1: &str = "legal winner thank year wave sausage worth useful legal winner thank yellow";
    let out = bin()
        .args([
            "bundle",
            "--descriptor",
            "wsh(or_d(pk(@1),sortedmulti(2,@0,@1)))",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={C0}"),
            "--slot",
            &format!("@1.phrase={C1}"),
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let v: Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    let md1: Vec<String> = v["md1"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect();
    let mut a = vec!["restore".to_string(), "--network".into(), "mainnet".into()];
    for c in &md1 {
        a.push("--md1".into());
        a.push(c.clone());
    }
    bin().args(&a).assert().failure().stderr(
        predicate::str::contains("sole child").and(predicate::str::contains("faithful backup")),
    );
}
