//! cycle-13 Lane C · M7 — `bundle … --json` must report the real multisig
//! threshold K in `multisig.threshold`, not the cosigner count N.
//!
//! In descriptor / `--import-json` / concrete-descriptor mode `args.threshold`
//! is `None`, so the `--json` emitter (`cmd/bundle.rs` JSON branch) fell back
//! to `threshold = args.threshold.unwrap_or(n as u8)` — the cosigner COUNT.
//! The engraving CARD path already computes K correctly via
//! `extract_multisig_threshold(&tree)`; the JSON branch never reused it.
//! md1 wire + embedded descriptor were CORRECT — only the JSON field was wrong.
//!
//! NOTE: this is a `--json` wire-VALUE change (not a flag), so schema_mirror
//! (flag-NAME parity) does not gate it — but it IS a GUI `--json` consumer
//! concern under the paired-PR discipline.

use assert_cmd::Command;

/// A 2-of-3 wsh(sortedmulti) descriptor (BSMS-derived cosigner xpubs). The
/// threshold K=2 lives inside the descriptor body; `--threshold` is NOT passed,
/// so `args.threshold` is None on the descriptor-mode bundle path.
const DESC_2_OF_3: &str = "wsh(sortedmulti(2,\
[b8688df1/48'/0'/0'/2']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*,\
[5436d724/48'/0'/0'/2']xpub6Buxw9MmbkJr4iAw8SACNci2hQNuPCMwt9P7HkK62ZQAW9UcJaQ2bc6ARD892TToQQ9Rp6AHujHxBLXqAsvn5fRnLfnhKSRfz8qtaoyKUYx/<0;1>/*,\
[28645006/48'/0'/0'/2']xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6/<0;1>/*))";

fn bundle_json(descriptor: &str) -> serde_json::Value {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--network",
            "mainnet",
            "--descriptor",
            descriptor,
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    serde_json::from_str(&stdout).unwrap()
}

// ============================================================================
// M7 — descriptor-mode 2-of-3 bundle --json must report threshold == 2.
// ============================================================================

#[test]
fn descriptor_mode_2_of_3_bundle_json_reports_real_threshold_k() {
    let v = bundle_json(DESC_2_OF_3);
    let m = &v["multisig"];
    assert_eq!(
        m["cosigner_count"].as_u64(),
        Some(3),
        "cosigner_count must be N=3; got {m}"
    );
    assert_eq!(
        m["threshold"].as_u64(),
        Some(2),
        "multisig.threshold must be the real K=2 (read from the descriptor \
         via extract_multisig_threshold), NOT the cosigner count N=3; got {m}"
    );
}
