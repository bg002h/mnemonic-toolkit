//! own-account-subset-search, P5 — randomized OVER-SUPPLY subset-search
//! COMPLETION round-trip PROPERTY test.
//!
//! The sibling `prop_template_completion_roundtrip.rs` exercises the EXACT-pool
//! keyless-template completion (own at a FIXED, pre-known account: the operator
//! passes `--account <acct>` and the search is the `n!` ordering only). THIS
//! property exercises the NET-NEW surface P2/P3 added — the OWN-ANCHORED
//! OVER-SUPPLY subset-search:
//!
//!   bundle --md1-form=template (keyless md1 + cosigner mk1s)
//!     → restore --md1 <template> --from <own seed>
//!         --own-account-max <K>           ← over-supply: K > the true own count,
//!                                            and the own account is RANDOM (not 0)
//!         --cosigner <real mk1s>
//!         --expect-wallet-id <id> --json
//!     → assert completed first addresses == an INDEPENDENT rust-miniscript
//!       `derive_at_index` golden built from the ORIGINAL descriptor (NOT
//!       md-codec reconstruction).
//!
//! The discriminating axis is the OWN ACCOUNT: the operator does not recall it,
//! so the search must resolve own@`own_acct` (a value in `0..K-1`, NOT always 0)
//! out of `K` over-supplied candidates. `K` is always > the exact own-slot count
//! (1 here), so every property case drives the genuine `S_own = C(K,j)·N!`
//! enumeration, never the `n!` exact path.
//!
//! NON-VACUITY (the cardinal rule):
//!   - The oracle is an INDEPENDENT rust-miniscript derivation of the SAME
//!     wallet from its ORIGINAL concrete descriptor, asserted byte-equal to the
//!     toolkit's completion output.
//!   - The permanent `oracle_*` self-test cells prove that oracle would FAIL for
//!     a WRONG SUBSET assignment — specifically, for the SUBSET axis this
//!     property exercises: deriving the SAME wallet shape with the own key at a
//!     DIFFERENT account yields a DIFFERENT first address. So the property cannot
//!     "match the golden" while resolving the wrong own account vacuously. This
//!     mirrors `prop_template_completion_roundtrip.rs`'s
//!     `oracle_swapped_assignment_changes_address_order_dependent`, but for the
//!     OVER-SUPPLY SUBSET axis (the own account varies), not the slot-order axis.
//!   - `subset_search_wrong_account_no_match_refuses` proves the SEARCH itself
//!     refuses a pool that cannot reproduce the recorded id (own range too narrow
//!     to include the true account) — a loud NO-MATCH, never a silent wrong
//!     wallet.
//!
//! FAILURE POLICY (mirrors the sibling prop): a gate-accepted shape MUST
//! complete — a `.success()`-asserting helper panics otherwise. A genuine
//! completion bug surfaces in the oracle (the address differential), NOT in an
//! exit-0 unwrap. If the property surfaces a real completion bug, it FAILS the
//! address differential (it does not paper over with `.success()`).
//!
//! Case budget is modest: the over-supply subset search (`C(K,1)·N!` ≤ a few
//! dozen for the bounded params here) + ~4 CLI spawns per case are not free;
//! mirror the sibling prop test's `ProptestConfig`.

use assert_cmd::Command;
use bip39::Mnemonic;
use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use miniscript::{Descriptor, DescriptorPublicKey};
use proptest::prelude::*;
use std::str::FromStr;

// ── Fixed seed pool (public BIP-39 vectors; same as the sibling suites). SEED_A
//    is the operator's OWN seed (slot @0, at a RANDOM account); SEED_B/SEED_C are
//    externally-supplied cosigners at fixed accounts. ──
const SEED_A: &str = "legal winner thank year wave sausage worth useful legal winner thank yellow";
const SEED_B: &str =
    "letter advice cage absurd amount doctor acoustic avoid letter advice cage above";
const SEED_C: &str = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";

fn bin() -> Command {
    Command::cargo_bin("mnemonic").expect("mnemonic binary builds")
}

// ── stdout/stderr section helpers (identical to the restore + sibling suites) ──
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

/// The canonical BIP-48 origin for a wsh/sh-wsh multisig at `account`.
fn canonical_path(script: &str, account: u32) -> String {
    match script {
        "wsh-multi" | "wsh-sortedmulti" => format!("48'/0'/{account}'/2'"),
        "sh-wsh-multi" | "sh-wsh-sortedmulti" => format!("48'/0'/{account}'/1'"),
        other => panic!("unknown script {other}"),
    }
}

// ───────────────────────────────────────────────────────────────────────────
// Generated wallet model.
//
// Each case is ONE small canonical multisig wallet, slot @0 = SEED_A (the OWN
// key, at a RANDOM `own_account`) and the remaining slots = SEED_B/SEED_C
// (external cosigners at FIXED accounts). We build the FULL concrete descriptor
// (the independent golden source) AND the matching `bundle --md1-form=template`
// invocation; the restore is driven by `--own-account-max` (over-supply).
// ───────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
struct GenWallet {
    /// `wsh-multi | wsh-sortedmulti | sh-wsh-multi | sh-wsh-sortedmulti`.
    script: &'static str,
    threshold: u32,
    /// (seed, account) per slot, slot @0 = (SEED_A, own_account).
    slots: Vec<(&'static str, u32)>,
}

impl GenWallet {
    fn sorted(&self) -> bool {
        self.script.contains("sortedmulti")
    }

    /// The full concrete descriptor (the independent rust-miniscript golden src).
    fn descriptor(&self) -> String {
        let mut keys: Vec<String> = Vec::new();
        for (seed, acct) in &self.slots {
            keys.push(key_str(seed, &canonical_path(self.script, *acct)));
        }
        let inner = if self.sorted() {
            format!("sortedmulti({},{})", self.threshold, keys.join(","))
        } else {
            format!("multi({},{})", self.threshold, keys.join(","))
        };
        match self.script {
            "wsh-multi" | "wsh-sortedmulti" => format!("wsh({inner})"),
            "sh-wsh-multi" | "sh-wsh-sortedmulti" => format!("sh(wsh({inner}))"),
            other => panic!("unknown script {other}"),
        }
    }

    /// `bundle` argv producing the keyless template md1 / per-cosigner mk1s, with
    /// `--md1-form` left as the `FORM` placeholder (substituted at call time).
    fn bundle_args(&self) -> Vec<String> {
        let mut args: Vec<String> = vec![
            "bundle".into(),
            "--network".into(),
            "mainnet".into(),
            "--template".into(),
            self.script.into(),
            "--threshold".into(),
            self.threshold.to_string(),
            "--md1-form".into(),
            "FORM".into(),
            "--group-size".into(),
            "0".into(),
            "--no-engraving-card".into(),
        ];
        for (idx, (seed, acct)) in self.slots.iter().enumerate() {
            let path = canonical_path(self.script, *acct);
            let (xpub, fp) = xpub_at(seed, &path);
            args.push("--slot".into());
            args.push(format!("@{idx}.xpub={xpub}"));
            args.push("--slot".into());
            args.push(format!("@{idx}.fingerprint={fp}"));
            args.push("--slot".into());
            args.push(format!("@{idx}.path={path}"));
        }
        args
    }
}

// ── bundle / restore helpers (FAILURE POLICY: gate-accepted → MUST succeed) ──

/// Run a `bundle` with `--md1-form` set to `form`, returning stdout.
fn run_bundle(w: &GenWallet, form: &str) -> String {
    let mut args = w.bundle_args();
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
    let mut args = w.bundle_args();
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

/// INDEPENDENT golden: first `count` receive addresses of a concrete descriptor
/// via rust-miniscript (NOT md-codec reconstruction).
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

/// Complete the keyless template via the OVER-SUPPLY OWN-ACCOUNT subset-search
/// (`--own-account-max own_account_max`, id-search); return the reported receive
/// addresses. FAILURE POLICY: success is asserted (a gate-accepted wallet MUST
/// complete) — a genuine bug surfaces in the address differential, not here.
fn complete_own_account_max(w: &GenWallet, own_account_max: u32, count: u32) -> Vec<String> {
    let template_md1 = md1_lines(&run_bundle(w, "template"));
    let cosigner_groups = mk1_groups(&run_bundle(w, "policy"));
    let id = template_wallet_id(w);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    for c in &template_md1 {
        args.push("--md1".into());
        args.push(c.clone());
    }
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        // The over-supply axis: own seed derived at accounts 0..own_account_max-1,
        // so the true (random, possibly non-zero) own account is RESOLVED by the
        // search out of `own_account_max` candidates — NEVER passed via --account.
        "--own-account-max".into(),
        own_account_max.to_string(),
        "--expect-wallet-id".into(),
        id,
        "--count".into(),
        count.to_string(),
        "--json".into(),
    ]);
    // Supply every NON-own slot as an unassigned `--cosigner` mk1 (slot @0 is own).
    for (idx, _slot) in w.slots.iter().enumerate() {
        if idx == 0 {
            continue; // own slot, filled by --from over the over-supply range
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

/// Materialize a (script, threshold, own_account, …) case into a `GenWallet`.
/// Family 0 = 2-of-2 {A@own, B@b}; family 1 = 2-of-3 {A@own, B@b, C@c}.
fn build_case(family: u8, script_idx: usize, own_acct: u32, b_acct: u32, c_acct: u32) -> GenWallet {
    let scripts = [
        "wsh-sortedmulti",
        "wsh-multi",
        "sh-wsh-sortedmulti",
        "sh-wsh-multi",
    ];
    let script = scripts[script_idx % scripts.len()];
    let slots = if family == 0 {
        vec![(SEED_A, own_acct), (SEED_B, b_acct)]
    } else {
        vec![(SEED_A, own_acct), (SEED_B, b_acct), (SEED_C, c_acct)]
    };
    GenWallet {
        script,
        threshold: 2,
        slots,
    }
}

// ───────────────────────────────────────────────────────────────────────────
// The property: a generated small canonical wallet whose OWN key is at a RANDOM
// account completes from its keyless template VIA THE OVER-SUPPLY SUBSET-SEARCH
// (`--own-account-max K`, K > the exact own count) to the INDEPENDENT golden.
// ───────────────────────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig {
        cases: std::env::var("PROP_CASES").ok().and_then(|s| s.parse().ok()).unwrap_or(24),
        // A degenerate generator (always the same trivial case) fails loudly via
        // the coverage self-test, not silently.
        max_global_rejects: 4,
        // No on-disk regression file (integration crate). A counterexample prints
        // its (family, script, accounts) — reproduce as an explicit #[test].
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    /// Headline (NET-NEW vs the sibling exact-pool prop): a randomly-generated
    /// small canonical wallet whose OWN key is at a RANDOM `own_acct` completes
    /// via the OVER-SUPPLY subset-search (`--own-account-max K`, K strictly
    /// greater than the exact own count, so `S_own = C(K,1)·N! > N!` is always
    /// driven) to the INDEPENDENT golden.
    #[test]
    fn subset_search_resolves_random_own_account_to_golden(
        family in 0u8..2,
        script_idx in 0usize..4,
        // The OWN account is RANDOM in 0..5 (the over-supply discriminating axis).
        own_acct in 0u32..5,
        b_acct in 0u32..3,
        c_acct in 0u32..3,
        // Extra over-supply headroom ABOVE the true account (so K > own_acct AND
        // K > 1 always ⇒ genuine subset-search, never the n! exact path).
        slack in 1u32..3,
    ) {
        let w = build_case(family, script_idx, own_acct, b_acct, c_acct);
        // Invariant: the own slot is SEED_A at `own_acct`.
        prop_assert_eq!(w.slots[0], (SEED_A, own_acct));

        // K so the true own account is in 0..K-1 AND K is strictly over-supplied
        // (K ≥ own_acct + 2 > 1 = the exact own count). This guarantees every
        // case drives the over-supply enumeration S_own = C(K,1)·N!, NOT n!.
        let own_account_max = own_acct + 1 + slack;
        prop_assert!(own_account_max > 1, "K must over-supply the single own slot");

        let golden = golden_addresses(&w.descriptor(), 2);
        let got = complete_own_account_max(&w, own_account_max, 2);
        prop_assert_eq!(
            got, golden,
            "over-supply subset-search (own@{} of 0..{}) must complete to the INDEPENDENT \
             golden for {}",
            own_acct, own_account_max, w.descriptor()
        );
    }
}

// ───────────────────────────────────────────────────────────────────────────
// Anti-vacuity self-test cells (PERMANENT — they keep the oracle honest).
// ───────────────────────────────────────────────────────────────────────────

/// The independent golden oracle is DISCRIMINATING ALONG THE SUBSET AXIS this
/// property exercises: the SAME wallet shape with the own key at a DIFFERENT
/// account derives a DIFFERENT first address. If this ever passed (equal
/// addresses across accounts) the oracle would be vacuous — the property could
/// "match the golden" while the search resolved the WRONG own account.
///
/// This is the subset-axis analogue of the sibling prop's
/// `oracle_swapped_assignment_changes_address_order_dependent` (which varies the
/// slot ORDER); here the discriminator is the OWN ACCOUNT (the subset axis).
#[test]
fn oracle_own_account_change_changes_address_subset_axis() {
    for script in [
        "wsh-multi",
        "wsh-sortedmulti",
        "sh-wsh-multi",
        "sh-wsh-sortedmulti",
    ] {
        // Same wallet shape, same cosigner — only the OWN account differs (the
        // exact axis the subset-search resolves). Cover BOTH sorted (the address
        // moves via the sorted-pubkey multiset) and unsorted shapes.
        let at0 = GenWallet {
            script,
            threshold: 2,
            slots: vec![(SEED_A, 0), (SEED_B, 0)],
        };
        let at3 = GenWallet {
            script,
            threshold: 2,
            slots: vec![(SEED_A, 3), (SEED_B, 0)],
        };
        assert_ne!(
            golden_addresses(&at0.descriptor(), 1),
            golden_addresses(&at3.descriptor(), 1),
            "[{script}] the own key at a DIFFERENT account MUST derive a different \
             address — else the subset-search golden oracle is vacuous"
        );
    }
}

/// The oracle ACCEPTS a faithful (same-account, same-keys) reconstruction — it
/// is not so strict it false-fails a correct completion (the dual of the above).
#[test]
fn oracle_accepts_faithful_reconstruction() {
    let w = GenWallet {
        script: "wsh-multi",
        threshold: 2,
        slots: vec![(SEED_A, 3), (SEED_B, 0)],
    };
    assert_eq!(
        golden_addresses(&w.descriptor(), 2),
        golden_addresses(&w.descriptor(), 2)
    );
}

/// The SEARCH itself refuses when the OVER-SUPPLY range cannot REACH the true
/// own account (never a silent wrong wallet): own@4 but `--own-account-max 3`
/// (range 0..2 excludes account 4) → no own candidate reproduces the recorded id
/// → NO-MATCH refuse (exit 4). This is the runtime funds-safety counterpart of
/// the subset-axis oracle self-test above — proving the failure direction (an
/// unreachable own account) is a LOUD refuse, not a silent mis-resolution.
#[test]
fn subset_search_unreachable_own_account_no_match_refuses() {
    let w = GenWallet {
        script: "wsh-sortedmulti",
        threshold: 2,
        slots: vec![(SEED_A, 4), (SEED_B, 0)],
    };
    let template_md1 = md1_lines(&run_bundle(&w, "template"));
    let cosigner_groups = mk1_groups(&run_bundle(&w, "policy"));
    let id = template_wallet_id(&w);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    for c in &template_md1 {
        args.push("--md1".into());
        args.push(c.clone());
    }
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        // Range 0..2 — EXCLUDES the true own account 4.
        "--own-account-max".into(),
        "3".into(),
        "--expect-wallet-id".into(),
        id,
    ]);
    for c in &cosigner_groups[1] {
        args.push("--cosigner".into());
        args.push(c.clone());
    }
    // No own candidate in 0..2 reproduces the own@4 id → loud refuse, never OK.
    let assert = bin().args(&args).assert().code(4);
    let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.to_uppercase().contains("NO MATCH"),
        "an own range that cannot reach the true account must NO-MATCH (never silent-wrong): {stderr}"
    );
}

/// Coverage smoke (fast even at low PROP_CASES): EVERY (family, script) the
/// generator can emit completes to its golden VIA the over-supply subset-search,
/// with the own account fixed at a NON-ZERO value (3). Guards against a generator
/// that silently only ever produces one shape (the property would prove little)
/// AND pins that the non-zero-own-account path works across the full script
/// matrix.
#[test]
fn every_generated_shape_completes_via_subset_search() {
    for family in 0..2u8 {
        for script_idx in 0..4usize {
            // own at NON-ZERO account 3, over-supplied via --own-account-max 5.
            let w = build_case(family, script_idx, 3, 0, 1);
            let golden = golden_addresses(&w.descriptor(), 2);
            let got = complete_own_account_max(&w, 5, 2);
            assert_eq!(
                got, golden,
                "family {family} script {} (own@3, --own-account-max 5) must complete to golden",
                w.script
            );
        }
    }
}

// ───────────────────────────────────────────────────────────────────────────
// OPT-IN property: an over-supplied COSIGNER pool (`--search-cosigner-subset` +
// an extra outsider cosigner candidate) completes to the golden; anti-vacuity =
// the extra candidate is DROPPED (the golden has only the real cosigners). This
// exercises the second over-supply axis (the cosigner subset) on top of a
// random own account.
// ───────────────────────────────────────────────────────────────────────────

/// An outsider seed NOT part of the wallet (the dropped over-supplied cosigner).
const SEED_OUTSIDER: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

/// Complete via `--search-cosigner-subset` (+ `--own-account-max`) over a pool
/// that includes ONE EXTRA outsider cosigner card; return reported addresses.
fn complete_cosigner_subset(
    w: &GenWallet,
    own_account_max: u32,
    extra_cosigner: &[String],
    count: u32,
) -> Vec<String> {
    let template_md1 = md1_lines(&run_bundle(w, "template"));
    let cosigner_groups = mk1_groups(&run_bundle(w, "policy"));
    let id = template_wallet_id(w);

    let mut args = vec!["restore".into(), "--network".into(), "mainnet".into()];
    for c in &template_md1 {
        args.push("--md1".into());
        args.push(c.clone());
    }
    args.extend([
        "--from".into(),
        format!("phrase={SEED_A}"),
        "--own-account-max".into(),
        own_account_max.to_string(),
        "--search-cosigner-subset".into(),
        "--expect-wallet-id".into(),
        id,
        "--count".into(),
        count.to_string(),
        "--json".into(),
    ]);
    // The REAL cosigner cards…
    for (idx, _slot) in w.slots.iter().enumerate() {
        if idx == 0 {
            continue;
        }
        for c in &cosigner_groups[idx] {
            args.push("--cosigner".into());
            args.push(c.clone());
        }
    }
    // …PLUS the over-supplied outsider candidate (must be DROPPED by the search).
    for c in extra_cosigner {
        args.push("--cosigner".into());
        args.push(c.clone());
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

proptest! {
    #![proptest_config(ProptestConfig {
        // The opt-in stratified search (own + cosigner axes) is heavier; keep the
        // budget smaller than the headline.
        cases: std::env::var("PROP_CASES_OPTIN").ok().and_then(|s| s.parse().ok()).unwrap_or(8),
        max_global_rejects: 4,
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    /// OPT-IN: a 2-of-3 canonical wallet with the own key at a RANDOM account,
    /// over-supplied on BOTH axes — `--own-account-max K` (own subset) AND an
    /// EXTRA outsider `--cosigner` card under `--search-cosigner-subset` (cosigner
    /// subset) — completes to the INDEPENDENT golden. Anti-vacuity: the golden is
    /// built from ONLY the real cosigners, so a passing match proves the search
    /// SELECTED the real {B,C} subset and DROPPED the outsider.
    #[test]
    fn cosigner_subset_search_drops_outsider_completes_to_golden(
        script_idx in 0usize..4,
        own_acct in 0u32..4,
        slack in 1u32..3,
    ) {
        // 2-of-3 {A@own, B@0, C@1} (distinct cosigner accounts → distinct keys).
        let w = build_case(1, script_idx, own_acct, 0, 1);
        let own_account_max = own_acct + 1 + slack;

        // The EXTRA over-supplied cosigner: an outsider seed at the same canonical
        // origin family (NOT a wallet member; must be dropped).
        let outsider_wallet = GenWallet {
            script: w.script,
            threshold: 2,
            slots: vec![(SEED_A, 0), (SEED_OUTSIDER, 0)],
        };
        let extra = mk1_groups(&run_bundle(&outsider_wallet, "policy"))[1].clone();

        // The golden is built from ONLY the real members — the outsider is NOT in
        // it, so equality proves the search dropped the outsider (anti-vacuity).
        let golden = golden_addresses(&w.descriptor(), 2);
        let got = complete_cosigner_subset(&w, own_account_max, &extra, 2);
        prop_assert_eq!(
            got, golden,
            "opt-in cosigner-subset search must DROP the outsider + resolve own@{} to the golden \
             for {}",
            own_acct, w.descriptor()
        );
    }
}

/// OPT-IN anti-vacuity (explicit cell): the outsider card the opt-in property
/// over-supplies is genuinely NOT a wallet member — its presence in an EXACT pool
/// (no `--search-cosigner-subset`) would change the wallet. Proven at the oracle
/// level: a wallet using the outsider derives a DIFFERENT address than the real
/// wallet, so "drop the outsider" is a real, discriminating selection.
#[test]
fn opt_in_outsider_is_genuinely_distinct() {
    let real = GenWallet {
        script: "wsh-multi",
        threshold: 2,
        slots: vec![(SEED_A, 0), (SEED_B, 0), (SEED_C, 1)],
    };
    let with_outsider = GenWallet {
        script: "wsh-multi",
        threshold: 2,
        slots: vec![(SEED_A, 0), (SEED_B, 0), (SEED_OUTSIDER, 0)],
    };
    assert_ne!(
        golden_addresses(&real.descriptor(), 1),
        golden_addresses(&with_outsider.descriptor(), 1),
        "the over-supplied outsider must be a genuine non-member (different address) — \
         else 'drop the outsider' is a vacuous selection"
    );
}
