//! Cycle Y (toolkit v0.73.3) — LOUD funds-safety advisory for a CUSTOM use-site
//! on a NUMS-taproot card, cross-surface contract.
//!
//! A `tr(NUMS, multi_a)` multisig with CUSTOM (divergent per-cosigner) use-site
//! derivation paths RESTORES FAITHFULLY since #26 (v0.59.1) — but no known wallet
//! produces this shape (every standard wallet uses one uniform `<0;1>/*` suffix
//! across all cosigners). A misconfigured user would silently get non-matching
//! addresses and risk PERMANENT LOSS OF FUNDS. Cycle Y KEEPS the reconstruction
//! (proceed-and-warn, not refuse) but emits a LOUD `WARNING (funds-safety): …`
//! line at BOTH engrave (`bundle` ×3 + `import-wallet`) AND restore
//! (`fn run_multisig`).
//!
//! The CORE properties:
//!   - the warning FIRES for a CUSTOM (divergent) `tr(NUMS,multi_a)` override at
//!     engrave AND restore;
//!   - BASELINE (uniform `<0;1>/*`) does NOT warn;
//!   - non-taproot / refused shapes do NOT warn;
//!   - the message carries the loud funds-safety phrasing;
//!   - restore STILL SUCCEEDS (exit 0) with the warning on stderr and the
//!     reconstructed addresses still on stdout (proceed-and-warn, not refuse).
//!
//! FOLLOWUP `restore-md1-taproot-use-site-override-arm` (PARTIALLY RESOLVED).

use assert_cmd::Command;
use serde_json::Value;

const C0: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const C1: &str = "legal winner thank year wave sausage worth useful legal winner thank yellow";

/// The LOUD funds-safety prefix (the distinctive marker; never shared with the
/// calm `advisory:` siblings).
const FS_PREFIX: &str = "WARNING (funds-safety):";

/// The CUSTOM (divergent per-cosigner) restorable `tr(NUMS,multi_a)` override —
/// the one shape that fires the loud warning.
const CUSTOM_OVERRIDE: &str = "tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<2;3>/*))";
/// BASELINE — uniform `<0;1>/*` across all cosigners → no use-site overrides →
/// must NOT warn.
const BASELINE_UNIFORM: &str = "tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<0;1>/*))";
/// A `sortedmulti_a` override — un-restorable; fires the CALM advisory, NOT the
/// loud funds-safety one.
const SORTEDMULTI_A_OVERRIDE: &str = "tr(NUMS,sortedmulti_a(2,@0/<0;1>/*,@1/<2;3>/*))";
/// A non-taproot divergent override — restorable, but NOT taproot → must NOT
/// fire the loud warning.
const NONTAPROOT_OVERRIDE: &str = "wsh(multi(2,@0/<0;1>/*,@1/<2;3>/*))";

fn bin() -> Command {
    Command::cargo_bin("mnemonic").expect("binary built")
}

/// `bundle --descriptor <desc>` with two phrase-slots, `--json --no-engraving-card`.
fn bundle_two_cosigner(descriptor: &str) -> assert_cmd::assert::Assert {
    bin()
        .args([
            "bundle",
            "--descriptor",
            descriptor,
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
}

/// Extract the md1 card array from a `bundle --json` stdout.
fn md1_cards(stdout: &[u8]) -> Vec<String> {
    let v: Value = serde_json::from_slice(stdout).expect("bundle --json valid JSON");
    v["md1"]
        .as_array()
        .expect("md1 array")
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect()
}

/// Feed md1 cards to `restore --md1` and return the Assert (caller asserts).
fn restore_md1(cards: &[String]) -> assert_cmd::assert::Assert {
    let mut a = vec!["restore".to_string(), "--network".into(), "mainnet".into()];
    for c in cards {
        a.push("--md1".into());
        a.push(c.clone());
    }
    bin().args(&a).assert()
}

// ── Engrave (bundle) ─────────────────────────────────────────────────────────

/// CUSTOM override fires the loud funds-safety warning at bundle (exit 0).
#[test]
fn bundle_custom_override_fires_funds_safety_warning() {
    let out = bundle_two_cosigner(CUSTOM_OVERRIDE).success();
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(
        stderr.contains(FS_PREFIX) && stderr.contains("PERMANENT LOSS OF FUNDS"),
        "custom override must fire the loud funds-safety warning at bundle; got: {stderr}"
    );
}

/// BASELINE (uniform) does NOT fire the loud warning at bundle.
#[test]
fn bundle_baseline_uniform_no_funds_safety_warning() {
    let out = bundle_two_cosigner(BASELINE_UNIFORM).success();
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(
        !stderr.contains(FS_PREFIX),
        "BASELINE uniform tr(NUMS,multi_a) must NOT fire the loud warning; got: {stderr}"
    );
}

/// A `sortedmulti_a` override (un-restorable) fires the CALM advisory, NOT the
/// loud funds-safety one (mutual exclusion at the engrave surface).
#[test]
fn bundle_sortedmulti_a_override_no_funds_safety_warning() {
    let out = bundle_two_cosigner(SORTEDMULTI_A_OVERRIDE).success();
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(
        !stderr.contains(FS_PREFIX),
        "un-restorable sortedmulti_a override fires the CALM advisory, not the loud one; got: {stderr}"
    );
    // Sanity: the calm advisory DID fire (it is the un-restorable register).
    assert!(
        stderr.contains("advisory: restore --md1 cannot reconstruct"),
        "sortedmulti_a override must fire the calm advisory; got: {stderr}"
    );
}

/// A non-taproot divergent override (restorable, but not taproot) does NOT fire
/// the loud warning.
#[test]
fn bundle_nontaproot_override_no_funds_safety_warning() {
    let out = bundle_two_cosigner(NONTAPROOT_OVERRIDE).success();
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(
        !stderr.contains(FS_PREFIX),
        "non-taproot override must NOT fire the loud warning; got: {stderr}"
    );
}

/// The `--json` stdout payload never carries the warning (it is on stderr).
#[test]
fn bundle_json_stdout_clean_warning_on_stderr() {
    let out = bundle_two_cosigner(CUSTOM_OVERRIDE).success();
    let o = out.get_output();
    let stdout = String::from_utf8_lossy(&o.stdout);
    let stderr = String::from_utf8_lossy(&o.stderr);
    let _: Value = serde_json::from_slice(&o.stdout).expect("bundle --json valid JSON on stdout");
    assert!(
        !stdout.contains(FS_PREFIX),
        "funds-safety warning must NOT leak into stdout JSON; stdout: {stdout}"
    );
    assert!(
        stderr.contains(FS_PREFIX),
        "funds-safety warning must be on stderr; stderr: {stderr}"
    );
}

// ── Restore (`fn run_multisig`) — the mandatory wrong-function-edit guard ─────

/// CUSTOM override: restore SUCCEEDS (exit 0), the loud warning is on stderr, AND
/// the reconstructed addresses are STILL emitted on stdout (the divergent `<2;3>`
/// suffix survives — proceed-and-warn, NOT refuse). If the emit lands in the
/// wrong restore function, assertion (1) fails RED.
#[test]
fn restore_custom_override_warns_loudly_and_still_succeeds() {
    let bundled = bundle_two_cosigner(CUSTOM_OVERRIDE).success();
    let cards = md1_cards(&bundled.get_output().stdout);

    let r = restore_md1(&cards).success(); // (2) exit 0 (Ok(0))
    let o = r.get_output();
    let stderr = String::from_utf8_lossy(&o.stderr).into_owned();
    let stdout = String::from_utf8_lossy(&o.stdout).into_owned();

    // (1) the loud warning fires on the restore surface (run_multisig).
    assert!(
        stderr.contains(FS_PREFIX) && stderr.contains("PERMANENT LOSS OF FUNDS"),
        "restore of a custom override must fire the loud warning on stderr; got: {stderr}"
    );
    // (3) reconstruction unchanged — @1's divergent <2;3>/* suffix still emitted.
    assert!(
        stdout.contains("<2;3>/*"),
        "restore must STILL reconstruct @1's divergent <2;3> suffix (proceed-and-warn); got: {stdout}"
    );
}

/// BASELINE (uniform) restore: SUCCEEDS, NO loud warning.
#[test]
fn restore_baseline_uniform_no_warning_and_succeeds() {
    let bundled = bundle_two_cosigner(BASELINE_UNIFORM).success();
    let cards = md1_cards(&bundled.get_output().stdout);
    let r = restore_md1(&cards).success();
    let stderr = String::from_utf8_lossy(&r.get_output().stderr).into_owned();
    assert!(
        !stderr.contains(FS_PREFIX),
        "BASELINE uniform restore must NOT fire the loud warning; got: {stderr}"
    );
}

// ── import-wallet (the second engrave surface) ───────────────────────────────

/// Two account xpubs (origin-annotated) carrying DIVERGENT per-cosigner suffixes
/// under `tr(NUMS,multi_a)` — the CUSTOM use-site override case as a CONCRETE
/// descriptor (xpubs, not `@N` placeholders) suitable for import-wallet intake.
const IMP_DIVERGENT_TR: &str = "tr(NUMS,multi_a(2,\
    [73c5da0a/87'/0'/0']xpub6DBjiYnc4ewKti13Q1L35bqdodw5z3VGJnf516B3icHrEGEUcCuCG5GVQDZtH8Xmsyt3Fs9YDNwLaqjUbbRidwXZ6sxufZcr4VqqzrXvicM/<0;1>/*,\
    [b8688df1/87'/0'/0']xpub6CbhrPzY2z7NcCGCGjLAJLq8iRyjUfwmdXQs66MxTVUReKqb9DpLnVJ5D1qpatZjUuPGTyxf5TYU1vA34YFE9FHB4TvfYmokYLVsyEFZFt9/<2;3>/*))";

/// import-wallet of a CUSTOM `tr(NUMS,multi_a)` divergent override (bare
/// descriptor blob) engraves an md1 the same way bundle does → the loud
/// funds-safety warning must fire there too.
#[test]
fn import_wallet_custom_override_fires_funds_safety_warning() {
    let out = bin()
        .args([
            "import-wallet",
            "--format",
            "descriptor",
            "--blob",
            "-",
            "--network",
            "mainnet",
        ])
        .write_stdin(IMP_DIVERGENT_TR)
        .assert()
        .success();
    let stderr = String::from_utf8_lossy(&out.get_output().stderr).into_owned();
    assert!(
        stderr.contains(FS_PREFIX) && stderr.contains("PERMANENT LOSS OF FUNDS"),
        "import-wallet must surface the loud funds-safety warning; got: {stderr}"
    );
}
