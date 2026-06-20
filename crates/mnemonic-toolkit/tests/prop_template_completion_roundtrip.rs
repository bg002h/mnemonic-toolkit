//! #28 phase 2, P5 — keyless multisig/general TEMPLATE COMPLETION round-trip
//! PROPERTY test.
//!
//! The P3a/P3b/P4 suites assert "completed addresses == independent
//! rust-miniscript golden" across a fixed set of canonical + general + verify
//! shapes. This property test adds what those fixed vectors do NOT: a
//! RANDOMIZED sweep over small (n ≤ 3) canonical AND general policies, each
//! exercising the keyless-template COMPLETION path end-to-end:
//!
//!   bundle --md1-form=template (keyless md1 + cosigner mk1s)
//!     → restore --md1 <template> --from <one seed> --account <acct>
//!         --cosigner <other mk1s> --expect-wallet-id <id> --json
//!     → assert completed first addresses == an INDEPENDENT rust-miniscript
//!       `derive_receive` from the ORIGINAL descriptor (NOT md-codec
//!       reconstruction).
//!
//! NON-VACUITY (the cardinal rule): the oracle is an independent
//! rust-miniscript derivation of the SAME wallet, asserted byte-equal to the
//! toolkit's completion output. The permanent `oracle_*` self-test cells prove
//! that oracle would FAIL for a WRONG key→slot assignment (a swapped-@N
//! descriptor derives a DIFFERENT address for an order-dependent shape), so a
//! degenerate/tautological oracle cannot pass vacuously. The
//! `swapped_assignment_no_match_refuses` cell proves the SEARCH itself refuses
//! a wrong key set (never a silent wrong wallet).
//!
//! FAILURE POLICY (mirrors `prop_backup_restore_roundtrip.rs`): a gate-accepted
//! policy MUST complete — a `.success()`-asserting helper panics otherwise. A
//! genuine reconstruction/completion bug surfaces in the oracle (the address
//! differential), NOT in an exit-0 unwrap.
//!
//! Case budget is modest: the n! permutation search (n ≤ 3 → ≤ 6) + ~4 CLI
//! spawns per case are not free; mirror the sibling prop test's ProptestConfig.

use assert_cmd::Command;
use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use miniscript::{Descriptor, DescriptorPublicKey};
use proptest::prelude::*;
use std::str::FromStr;

// ── Fixed seed pool (public BIP-39 vectors; same as the sibling suites). The
//    own seed for every generated wallet is SEED_A (slot @0); SEED_B/SEED_C are
//    externally-supplied cosigners. ──
const SEED_A: &str = "legal winner thank year wave sausage worth useful legal winner thank yellow";
const SEED_B: &str =
    "letter advice cage absurd amount doctor acoustic avoid letter advice cage above";
const SEED_C: &str = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";

fn bin() -> Command {
    Command::cargo_bin("mnemonic").expect("mnemonic binary builds")
}

// ── stdout/stderr section helpers (identical to the restore suite) ──
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

fn md1_lines(stdout: &str) -> Vec<String> {
    section_lines(stdout, "# md1")
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

/// Derive a mainnet account xpub + master fingerprint at `path_str`.
fn xpub_at(phrase: &str, path_str: &str) -> (Xpub, String) {
    let secp = Secp256k1::new();
    let m = Mnemonic::parse_in(bip39::Language::English, phrase).unwrap();
    let seed = m.to_seed("");
    let master = Xpriv::new_master(bitcoin::NetworkKind::Main, &seed).unwrap();
    let fp = master.fingerprint(&secp);
    let path = DerivationPath::from_str(path_str).unwrap();
    let xpriv = master.derive_priv(&secp, &path).unwrap();
    let xpub = Xpub::from_priv(&secp, &xpriv);
    (xpub, fp.to_string().to_lowercase())
}

/// A `[fp/origin]xpub/<0;1>/*` key string for `phrase` at `path` (no leading m/).
fn key_str(phrase: &str, path: &str) -> String {
    let (xpub, fp) = xpub_at(phrase, path);
    let origin = path.replace('\'', "h");
    format!("[{fp}/{origin}]{xpub}/<0;1>/*")
}

// ───────────────────────────────────────────────────────────────────────────
// Generated wallet model.
//
// Each case is ONE small wallet, with slot @0 = SEED_A (the OWN key, at
// `own_account`) and the remaining slots = SEED_B/SEED_C (external cosigners).
// We build the FULL concrete descriptor string (the independent golden source)
// AND the matching `bundle --md1-form=template` invocation.
//
// Two families:
//   • CANONICAL: wsh(multi/sortedmulti), sh(wsh(...)) at BIP-48 (so `--template`
//     can emit it) — covers the order-dependent vs order-independent legs.
//   • GENERAL: wsh(or_i(...)) / wsh(thresh(...)) at BIP-84/BIP-87 (non-canonical
//     origins) — emitted via `--descriptor`.
// ───────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct GenWallet {
    /// The full concrete descriptor (the independent rust-miniscript golden source).
    descriptor: String,
    /// `bundle` argv producing the keyless template md1 / per-cosigner mk1s.
    /// (form is substituted at call time: "template" → md1, "policy" → mk1s)
    bundle_args_template: Vec<String>,
    /// The OWN account (slot @0 = SEED_A at this account).
    own_account: u32,
    /// The N cosigner seeds in slot order (slot @0 is always SEED_A).
    slots: Vec<&'static str>,
}

/// Family A (canonical, `--template`): the per-slot path is BIP-48
/// `48'/0'/acct'/{2'|1'}`. The bundle emits via `--template <script>`.
fn gen_canonical(
    script: &str, // wsh-multi | wsh-sortedmulti | sh-wsh-multi | sh-wsh-sortedmulti
    threshold: u32,
    slots: &[(&'static str, u32)], // (seed, account)
) -> GenWallet {
    let sorted = script.contains("sortedmulti");
    let usesite = match script {
        "wsh-multi" | "wsh-sortedmulti" => "2'",
        "sh-wsh-multi" | "sh-wsh-sortedmulti" => "1'",
        other => panic!("unknown canonical script {other}"),
    };
    let mut keys: Vec<String> = Vec::new();
    let mut bundle_args: Vec<String> = vec![
        "bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        script.into(),
        "--threshold".into(),
        threshold.to_string(),
        "--md1-form".into(),
        "FORM".into(), // placeholder, replaced per call
        "--group-size".into(),
        "0".into(),
        "--no-engraving-card".into(),
    ];
    for (idx, (seed, acct)) in slots.iter().enumerate() {
        let path = format!("48'/0'/{acct}'/{usesite}");
        keys.push(key_str(seed, &path));
        let (xpub, fp) = xpub_at(seed, &path);
        bundle_args.push("--slot".into());
        bundle_args.push(format!("@{idx}.xpub={xpub}"));
        bundle_args.push("--slot".into());
        bundle_args.push(format!("@{idx}.fingerprint={fp}"));
        bundle_args.push("--slot".into());
        bundle_args.push(format!("@{idx}.path={path}"));
    }
    let inner = if sorted {
        format!("sortedmulti({threshold},{})", keys.join(","))
    } else {
        format!("multi({threshold},{})", keys.join(","))
    };
    let descriptor = match script {
        "wsh-multi" | "wsh-sortedmulti" => format!("wsh({inner})"),
        "sh-wsh-multi" | "sh-wsh-sortedmulti" => format!("sh(wsh({inner}))"),
        _ => unreachable!(),
    };
    GenWallet {
        descriptor,
        bundle_args_template: bundle_args,
        own_account: slots[0].1,
        slots: slots.iter().map(|(s, _)| *s).collect(),
    }
}

/// Family B (general, `--descriptor`): a `wsh(or_i(...))` or `wsh(thresh(...))`
/// at BIP-84 `84'/0'/acct'`. The full descriptor is bundled DIRECTLY.
fn gen_general(shape: usize, slots: &[(&'static str, u32)]) -> GenWallet {
    let k = |i: usize| -> String {
        let (seed, acct) = slots[i];
        key_str(seed, &format!("84'/0'/{acct}'"))
    };
    let descriptor = match shape {
        // 2 slots — order-DEPENDENT or_i with single-key branches.
        0 => format!("wsh(or_i(pk({}),pk({})))", k(0), k(1)),
        // 3 slots — or_i(pk, and_v(v:pk, pk)) (the P3b archetype: @0 alone, @1/@2 jointly).
        1 => format!("wsh(or_i(pk({}),and_v(v:pk({}),pk({}))))", k(0), k(1), k(2)),
        // 3 slots — thresh(2, pk, s:pk, s:pk) (the general thresh leg, order-dependent).
        _ => format!("wsh(thresh(2,pk({}),s:pk({}),s:pk({})))", k(0), k(1), k(2)),
    };
    let bundle_args = vec![
        "bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--md1-form".into(),
        "FORM".into(),
        "--group-size".into(),
        "0".into(),
        "--no-engraving-card".into(),
        "--descriptor".into(),
        descriptor.clone(),
    ];
    GenWallet {
        descriptor,
        bundle_args_template: bundle_args,
        own_account: slots[0].1,
        slots: slots.iter().map(|(s, _)| *s).collect(),
    }
}

// ── bundle / restore helpers (FAILURE POLICY: gate-accepted → MUST succeed) ──

/// Run a `bundle` with `--md1-form` set to `form`, returning stdout.
fn run_bundle(w: &GenWallet, form: &str) -> String {
    let mut args = w.bundle_args_template.clone();
    // replace the FORM placeholder
    for a in args.iter_mut() {
        if a == "FORM" {
            *a = form.to_string();
        }
    }
    let out = bin().args(&args).assert().success();
    String::from_utf8(out.get_output().stdout.clone()).unwrap()
}

/// The printed WalletPolicyId (full hex) from the template emit advisory (stderr).
fn template_wallet_id(w: &GenWallet) -> String {
    let mut args = w.bundle_args_template.clone();
    for a in args.iter_mut() {
        if a == "FORM" {
            *a = "template".to_string();
        }
    }
    let out = bin().args(&args).assert().success();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    let line = stderr
        .lines()
        .find(|l| l.contains("wallet-id (hex)"))
        .unwrap_or_else(|| panic!("no wallet-id (hex) line in: {stderr}"));
    line.split(':').next_back().unwrap().trim().to_string()
}

/// INDEPENDENT golden: first `count` receive addresses of the ORIGINAL concrete
/// descriptor via rust-miniscript (NOT md-codec reconstruction).
fn golden_addresses(descriptor: &str, count: u32) -> Vec<String> {
    let parsed = Descriptor::<DescriptorPublicKey>::from_str(descriptor)
        .unwrap_or_else(|e| panic!("golden descriptor parse {descriptor}: {e}"));
    let receive = parsed.into_single_descriptors().unwrap().remove(0);
    (0..count)
        .map(|i| {
            receive
                .derive_at_index(i)
                .unwrap()
                .address(bitcoin::Network::Bitcoin)
                .unwrap()
                .to_string()
        })
        .collect()
}

/// Complete the keyless template via id-search; return the reported receive
/// addresses. FAILURE POLICY: success is asserted (a gate-accepted wallet MUST
/// complete) — a genuine bug surfaces in the address differential, not here.
fn complete_id_search(w: &GenWallet, count: u32) -> Vec<String> {
    let template_md1 = md1_lines(&run_bundle(w, "template"));
    let policy_stdout = run_bundle(w, "policy");
    let cosigner_groups = mk1_groups(&policy_stdout);
    let id = template_wallet_id(w);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    for c in &template_md1 {
        args.push("--md1".into());
        args.push(c.clone());
    }
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        w.own_account.to_string(),
        "--expect-wallet-id".into(),
        id,
        "--count".into(),
        count.to_string(),
        "--json".into(),
    ]);
    // Supply every NON-own slot as an unassigned `--cosigner` mk1 (slot @0 is own).
    for (idx, _seed) in w.slots.iter().enumerate() {
        if idx == 0 {
            continue; // own slot, filled by --from
        }
        for c in &cosigner_groups[idx] {
            args.push("--cosigner".into());
            args.push(c.clone());
        }
    }
    let out = bin().args(&args).assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let j: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("restore --json parse: {e}; stdout: {stdout}"));
    j["wallets"][0]["first_addresses"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a.as_str().unwrap().to_string())
        .collect()
}

// ───────────────────────────────────────────────────────────────────────────
// The property: a generated small canonical/general wallet completes from its
// keyless template to the INDEPENDENT golden.
// ───────────────────────────────────────────────────────────────────────────

/// Materialize a (family, params) case into a `GenWallet`. `n` slots, own @0.
fn build_case(family: u8, variant: usize, own_acct: u32, b_acct: u32, c_acct: u32) -> GenWallet {
    match family {
        // Canonical 2-of-2 / 2-of-3 (sorted | unsorted | sh-wsh).
        0 => {
            let scripts = [
                "wsh-sortedmulti",
                "wsh-multi",
                "sh-wsh-sortedmulti",
                "sh-wsh-multi",
            ];
            let script = scripts[variant % scripts.len()];
            if variant % 2 == 0 {
                // 2-of-2 {A@own, B@b}
                gen_canonical(script, 2, &[(SEED_A, own_acct), (SEED_B, b_acct)])
            } else {
                // 2-of-3 {A@own, B@b, C@c}
                gen_canonical(
                    script,
                    2,
                    &[(SEED_A, own_acct), (SEED_B, b_acct), (SEED_C, c_acct)],
                )
            }
        }
        // General (or_i 2-key | or_i 3-key | thresh 3-key) at BIP-84.
        _ => {
            if variant % 3 == 0 {
                gen_general(0, &[(SEED_A, own_acct), (SEED_B, b_acct)])
            } else if variant % 3 == 1 {
                gen_general(1, &[(SEED_A, own_acct), (SEED_B, b_acct), (SEED_C, c_acct)])
            } else {
                gen_general(2, &[(SEED_A, own_acct), (SEED_B, b_acct), (SEED_C, c_acct)])
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: std::env::var("PROP_CASES").ok().and_then(|s| s.parse().ok()).unwrap_or(32),
        // A degenerate generator (always the same trivial case) fails loudly via
        // the coverage self-test, not silently.
        max_global_rejects: 4,
        // No on-disk regression file (integration crate). A counterexample prints
        // its (family, variant, accounts) — reproduce as an explicit #[test].
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    /// Headline: a randomly-generated small canonical/general wallet completes
    /// from its keyless template (id-search) to the INDEPENDENT golden.
    #[test]
    fn template_completion_matches_independent_golden(
        family in 0u8..2,
        variant in 0usize..4,
        own_acct in 0u32..3,
        b_acct in 0u32..3,
        c_acct in 0u32..3,
    ) {
        let w = build_case(family, variant, own_acct, b_acct, c_acct);
        // Sanity: the own slot is SEED_A (the property's invariant).
        prop_assert_eq!(w.slots[0], SEED_A);

        let golden = golden_addresses(&w.descriptor, 2);
        let got = complete_id_search(&w, 2);
        prop_assert_eq!(
            got, golden,
            "completion address differential failed for descriptor {}", w.descriptor
        );
    }
}

// ───────────────────────────────────────────────────────────────────────────
// Anti-vacuity self-test cells (PERMANENT — they keep the oracle honest).
// ───────────────────────────────────────────────────────────────────────────

/// The independent golden oracle is DISCRIMINATING: a wrong key→slot assignment
/// (swapping @0 and @1 in an ORDER-DEPENDENT shape) derives a DIFFERENT first
/// address. If this ever passed (equal addresses) the oracle would be vacuous —
/// the property could "match the golden" while completing the wrong wallet.
#[test]
fn oracle_swapped_assignment_changes_address_order_dependent() {
    // wsh-multi (order-dependent): {A@0, B@0} vs the SWAPPED {B@0, A@0}.
    let correct = gen_canonical("wsh-multi", 2, &[(SEED_A, 0), (SEED_B, 0)]);
    let swapped = gen_canonical("wsh-multi", 2, &[(SEED_B, 0), (SEED_A, 0)]);
    assert_ne!(
        golden_addresses(&correct.descriptor, 1),
        golden_addresses(&swapped.descriptor, 1),
        "an order-dependent multi MUST derive a different address under a swapped \
         @N assignment — else the golden oracle is vacuous"
    );

    // The general or_i archetype (order-dependent): @0 alone-spends, @1/@2
    // jointly — swapping @0↔@1 changes the spending roles AND the address.
    let g_correct = gen_general(1, &[(SEED_A, 0), (SEED_B, 1), (SEED_C, 2)]);
    let g_swapped = gen_general(1, &[(SEED_B, 1), (SEED_A, 0), (SEED_C, 2)]);
    assert_ne!(
        golden_addresses(&g_correct.descriptor, 1),
        golden_addresses(&g_swapped.descriptor, 1),
        "a general or_i MUST derive a different address under a swapped @N \
         assignment — else the golden oracle is vacuous"
    );
}

/// The oracle ACCEPTS a faithful (same-keys, same-order) reconstruction — it is
/// not so strict it false-fails a correct completion (the dual of the above).
#[test]
fn oracle_accepts_faithful_reconstruction() {
    let w = gen_canonical("wsh-multi", 2, &[(SEED_A, 0), (SEED_B, 0)]);
    assert_eq!(
        golden_addresses(&w.descriptor, 2),
        golden_addresses(&w.descriptor, 2)
    );
}

/// The SEARCH itself refuses a WRONG key set (never a silent wrong wallet): a
/// recorded id for {A,B} with a cosigner that is NOT B → no permutation matches
/// → REFUSE (exit ≠ 0). This is the runtime funds-safety counterpart of the
/// oracle self-test above (the property's `.success()` path only sees correct
/// key sets; this cell proves the failure direction is a loud refuse).
#[test]
fn swapped_assignment_no_match_refuses() {
    // The recorded id is for {A@0, B@0}; supply a cosigner C (NOT B) → REFUSE.
    let correct = gen_canonical("wsh-sortedmulti", 2, &[(SEED_A, 0), (SEED_B, 0)]);
    let id = template_wallet_id(&correct);
    let template_md1 = md1_lines(&run_bundle(&correct, "template"));

    // A wrong cosigner: emit a {A@0, C@0} wallet's @1 mk1 (carries C's key).
    let wrong = gen_canonical("wsh-sortedmulti", 2, &[(SEED_A, 0), (SEED_C, 0)]);
    let wrong_cosigner = mk1_groups(&run_bundle(&wrong, "policy"))[1].clone();

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    for c in &template_md1 {
        args.push("--md1".into());
        args.push(c.clone());
    }
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--account".into(),
        "0".into(),
        "--expect-wallet-id".into(),
        id,
    ]);
    for c in &wrong_cosigner {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    // No assignment of {A, C} reproduces the {A, B} id → loud refuse, never OK.
    bin().args(&args).assert().failure();
}

/// Coverage smoke (fast even at low PROP_CASES): EVERY family/variant the
/// generator can emit completes to its golden. Guards against a generator that
/// silently only ever produces one shape (the property would then prove little).
#[test]
fn every_generated_shape_completes() {
    // family 0: canonical (4 scripts × {2of2, 2of3}); family 1: general (3 shapes).
    for variant in 0..4usize {
        let w = build_case(0, variant, 0, 1, 2);
        let golden = golden_addresses(&w.descriptor, 2);
        let got = complete_id_search(&w, 2);
        assert_eq!(
            got, golden,
            "canonical variant {variant} ({}) must complete to golden",
            w.descriptor
        );
    }
    for variant in 0..3usize {
        let w = build_case(1, variant, 0, 1, 2);
        let golden = golden_addresses(&w.descriptor, 2);
        let got = complete_id_search(&w, 2);
        assert_eq!(
            got, golden,
            "general variant {variant} ({}) must complete to golden",
            w.descriptor
        );
    }
}
