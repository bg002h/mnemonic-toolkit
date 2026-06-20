//! C1 — bundle/import-wallet unrestorable-shape advisory cross-surface contract.
//!
//! `bundle` (and `import-wallet`) engrave a wire-faithful md1 card for descriptor
//! shapes that `restore --md1` then REFUSES to mechanically reconstruct. The
//! refusals are LOUD (not silent funds-loss — `restore` exits non-zero and the
//! card is a faithful backup), but the user got no warning at engrave time. This
//! advisory closes that gap with a NON-BLOCKING stderr note that fires IFF
//! restore would refuse, across three shapes:
//!   1. sortedmulti() inside a combinator (not the sole wsh/sh child),
//!   2. a HARDENED use-site path anywhere (`/*h` wildcard or a hardened alt,
//!      baseline OR per-cosigner override),
//!   3. a TAPROOT (`tr`) root carrying per-cosigner use-site overrides (deferred).
//!
//! P2.4 UPDATE: non-taproot, non-hardened per-cosigner use-site overrides are now
//! RESTORABLE (faithful per-`@N` reconstruction) — the old blanket overrides
//! advisory was DROPPED, and a `shared_suffix`-style divergent card now bundles
//! WITHOUT an advisory and restores faithfully.
//!
//! Mirrors the v0.55.2 `older()` advisory (`cli_older_advisory.rs`). The CORE
//! correctness property is PARITY: every positive cell feeds ONE descriptor to
//! BOTH `bundle` (advisory fires, exit 0) AND `restore --md1` of the emitted card
//! (refusal) — proving the advisory predicate matches restore's refusal predicate.
//!
//! FOLLOWUP `bundle-unrestorable-shape-advisory`.

use assert_cmd::Command;
use serde_json::Value;

const C0: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const C1: &str = "legal winner thank year wave sausage worth useful legal winner thank yellow";

/// Surface-stable advisory prefix shared by all three shape messages.
const ADVISORY_PREFIX: &str = "advisory: restore --md1 cannot reconstruct";

fn bin() -> Command {
    Command::cargo_bin("mnemonic").expect("binary built")
}

/// `bundle --descriptor <desc>` with two phrase-slots, `--json --no-engraving-card`.
/// Returns (exit-0 assert output, stdout string).
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

// ── Positive cells: advisory fires + non-blocking + restore-refuse parity ─────

/// Shape 1 — sortedmulti() inside a combinator. The GAP-3 descriptor
/// (`prop_backup_restore_roundtrip.rs:684`). bundle exit 0 + advisory; restore
/// refuses ("sole child" … "faithful backup").
#[test]
fn shape1_sortedmulti_in_combinator_fires_and_restore_refuses() {
    let out = bundle_two_cosigner("wsh(or_d(pk(@1),sortedmulti(2,@0,@1)))").success();
    let o = out.get_output();
    let stderr = String::from_utf8_lossy(&o.stderr);
    assert!(
        stderr.contains(ADVISORY_PREFIX) && stderr.contains("sortedmulti"),
        "shape-1 advisory must fire on stderr; got: {stderr}"
    );
    // Parity: restore of the emitted card refuses loudly.
    let cards = md1_cards(&o.stdout);
    let r = restore_md1(&cards).failure();
    let rerr = String::from_utf8_lossy(&r.get_output().stderr);
    assert!(
        rerr.contains("sole child") && rerr.contains("faithful backup"),
        "shape-1 restore must refuse loudly; got: {rerr}"
    );
}

/// Shape 2 — a HARDENED use-site path (`/*h`). bundle exit 0 + advisory; restore
/// refuses (shared `has_hardened_use_site` predicate).
#[test]
fn shape2_hardened_wildcard_fires_and_restore_refuses() {
    let out = bundle_two_cosigner("wsh(multi(2,@0/*h,@1/*h))").success();
    let o = out.get_output();
    let stderr = String::from_utf8_lossy(&o.stderr);
    assert!(
        stderr.contains(ADVISORY_PREFIX) && stderr.contains("hardened use-site path"),
        "shape-2 advisory must fire on stderr; got: {stderr}"
    );
    let cards = md1_cards(&o.stdout);
    let r = restore_md1(&cards).failure();
    let rerr = String::from_utf8_lossy(&r.get_output().stderr);
    assert!(
        rerr.contains("hardened use-site path"),
        "shape-2 restore must refuse loudly; got: {rerr}"
    );
}

/// Shape 2b — an OVERRIDE-only hardened wildcard (`@0` clean, `@1/<2;3>/*h`):
/// exercises the override-aware leg of `has_hardened_use_site` (not just the
/// baseline scan). bundle exit 0 + advisory; restore refuses.
#[test]
fn shape2b_override_hardened_wildcard_fires_and_restore_refuses() {
    let out = bundle_two_cosigner("wsh(multi(2,@0/<0;1>/*,@1/<2;3>/*h))").success();
    let o = out.get_output();
    let stderr = String::from_utf8_lossy(&o.stderr);
    assert!(
        stderr.contains(ADVISORY_PREFIX) && stderr.contains("hardened use-site path"),
        "shape-2b advisory must fire on stderr; got: {stderr}"
    );
    let cards = md1_cards(&o.stdout);
    let r = restore_md1(&cards).failure();
    let rerr = String::from_utf8_lossy(&r.get_output().stderr);
    assert!(
        rerr.contains("hardened use-site path"),
        "shape-2b restore must refuse loudly; got: {rerr}"
    );
}

/// Shape 3 — a TAPROOT root carrying per-cosigner use-site overrides
/// (`tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<2;3>/*))`). bundle exit 0 + advisory;
/// restore refuses (shared `taproot_override_card` predicate; taproot leg
/// deferred to FOLLOWUP `restore-md1-taproot-use-site-override-arm`).
#[test]
fn shape3_taproot_use_site_override_fires_and_restore_refuses() {
    let out = bundle_two_cosigner("tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<2;3>/*))").success();
    let o = out.get_output();
    let stderr = String::from_utf8_lossy(&o.stderr);
    assert!(
        stderr.contains(ADVISORY_PREFIX) && stderr.contains("taproot"),
        "shape-3 advisory must fire on stderr; got: {stderr}"
    );
    let cards = md1_cards(&o.stdout);
    let r = restore_md1(&cards).failure();
    let rerr = String::from_utf8_lossy(&r.get_output().stderr);
    assert!(
        rerr.contains("taproot") && rerr.contains("restore-md1-taproot-use-site-override-arm"),
        "shape-3 restore must refuse loudly; got: {rerr}"
    );
}

/// P2.4 PARITY (the flip): a NON-taproot, non-hardened per-cosigner use-site
/// override card (`@0/<0;1>/*` vs DIVERGENT `@1/<2;3>/*`) is now RESTORABLE —
/// bundle fires NO advisory, and restore SUCCEEDS faithfully (the divergent
/// suffix survives). Proves the dropped overrides-advisory matches the narrowed
/// guard.
#[test]
fn nontaproot_nonhardened_override_no_advisory_and_restores() {
    let out = bundle_two_cosigner("wsh(multi(2,@0/<0;1>/*,@1/<2;3>/*))").success();
    let o = out.get_output();
    let stderr = String::from_utf8_lossy(&o.stderr);
    assert!(
        !stderr.contains(ADVISORY_PREFIX),
        "non-taproot non-hardened override must NOT fire the advisory; got: {stderr}"
    );
    // Parity: restore reconstructs it faithfully (exit 0) with the divergent suffix.
    let cards = md1_cards(&o.stdout);
    let r = restore_md1(&cards).success();
    let rout = String::from_utf8_lossy(&r.get_output().stdout);
    assert!(
        rout.contains("<2;3>/*"),
        "restore must reconstruct @1's divergent <2;3> suffix; got: {rout}"
    );
}

// ── import-wallet parity (the second engrave surface) ────────────────────────

/// import-wallet of a foreign (Bitcoin Core) wallet carrying a hardened wildcard
/// emits an md1 the same way bundle does → the advisory must fire there too.
#[test]
fn import_wallet_hardened_wildcard_fires_advisory() {
    // Two account xpubs from cli_bip388_policy_intake.rs (origin-annotated).
    let a = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
    let b = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";
    // Checksum ry7qflrd computed via `bitcoin-cli getdescriptorinfo` (Core v27).
    let desc = format!(
        "wsh(sortedmulti(2,[704c7836/48'/0'/0'/2']{a}/*h,[97139860/48'/0'/0'/2']{b}/*h))#ry7qflrd"
    );
    let envelope = format!(
        r#"{{"wallet_name":"x","descriptors":[{{"desc":"{desc}","active":true,"internal":false}}]}}"#
    );
    let out = bin()
        .args([
            "import-wallet",
            "--format",
            "bitcoin-core",
            "--blob",
            "-",
            "--network",
            "mainnet",
        ])
        .write_stdin(envelope)
        .assert()
        .success();
    let stderr = String::from_utf8_lossy(&out.get_output().stderr);
    assert!(
        stderr.contains(ADVISORY_PREFIX) && stderr.contains("hardened use-site path"),
        "import-wallet must surface the unrestorable-shape advisory; got: {stderr}"
    );
}

// ── Clean negatives: NO false positives (the predicate-parity guard) ──────────

/// All THREE restorable sole-child sortedmulti positions must NOT fire (R0-r1 I1):
/// wsh(sortedmulti), sh(wsh(sortedmulti)), AND bare sh(sortedmulti) — each bundles
/// exit 0 with NO advisory, and restore exits 0 (faithful).
#[test]
fn sole_child_sortedmulti_no_advisory_and_restores() {
    for desc in [
        "wsh(sortedmulti(2,@0,@1))",
        "sh(wsh(sortedmulti(2,@0,@1)))",
        "sh(sortedmulti(2,@0,@1))",
    ] {
        let out = bundle_two_cosigner(desc).success();
        let o = out.get_output();
        let stderr = String::from_utf8_lossy(&o.stderr);
        assert!(
            !stderr.contains(ADVISORY_PREFIX),
            "sole-child `{desc}` must NOT fire the advisory; got: {stderr}"
        );
        // Parity: restore reconstructs it (exit 0).
        let cards = md1_cards(&o.stdout);
        restore_md1(&cards).success();
    }
}

/// `multi()` (a real miniscript Terminal, NOT sortedmulti) inside a combinator
/// restores fine → NO advisory (guards the shape-1 predicate against keying on
/// the wrong tag).
#[test]
fn multi_in_combinator_no_advisory() {
    let out = bundle_two_cosigner("wsh(or_d(pk(@1),multi(2,@0,@1)))").success();
    let stderr = String::from_utf8_lossy(&out.get_output().stderr);
    assert!(
        !stderr.contains(ADVISORY_PREFIX),
        "multi-in-combinator restores fine and must NOT fire; got: {stderr}"
    );
}

/// A shared-suffix multisig (one multipath for all cosigners, unhardened
/// wildcard) → NO advisory (shapes 2 and 3 both absent).
#[test]
fn shared_suffix_unhardened_no_advisory() {
    let out = bundle_two_cosigner("wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))").success();
    let stderr = String::from_utf8_lossy(&out.get_output().stderr);
    assert!(
        !stderr.contains(ADVISORY_PREFIX),
        "shared-suffix unhardened multisig must NOT fire; got: {stderr}"
    );
}

// ── --json stdout cleanliness ────────────────────────────────────────────────

/// The advisory is on stderr; the `--json` stdout payload never contains the
/// advisory text (mirror `cli_older_advisory.rs` invariant 4).
#[test]
fn json_stdout_clean_advisory_on_stderr() {
    let out = bundle_two_cosigner("wsh(or_d(pk(@1),sortedmulti(2,@0,@1)))").success();
    let o = out.get_output();
    let stdout = String::from_utf8_lossy(&o.stdout);
    let stderr = String::from_utf8_lossy(&o.stderr);
    // stdout is valid JSON and carries NO advisory text.
    let _: Value = serde_json::from_slice(&o.stdout).expect("bundle --json valid JSON on stdout");
    assert!(
        !stdout.contains(ADVISORY_PREFIX),
        "advisory must NOT leak into stdout JSON; stdout: {stdout}"
    );
    assert!(
        stderr.contains(ADVISORY_PREFIX),
        "advisory must be on stderr under --json; stderr: {stderr}"
    );
}
