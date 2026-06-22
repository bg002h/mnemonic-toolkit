//! SPEC cycle-2 H10 — `mnemonic export-wallet` refuses an UNSORTED
//! `wsh-multi` / `sh-wsh-multi` template to the field-less
//! electrum / coldcard(-multisig) / jade vendors.
//!
//! Those three vendor file formats are BIP-67 sortedmulti-only (no field to
//! express literal `multi(...)` key order), so exporting an unsorted multisig
//! to them would silently coerce to `sortedmulti` → different
//! witnessScript / address (oracle-proven by the `wsh-multi-2of3-divergent`
//! row in `tests/bitcoind_differential.rs`). The fix is a PURE REFUSAL (no
//! flag) at the shared `emit_payload` chokepoint, gated on a STRUCTURED
//! predicate over the resolved `CliTemplate`.
//!
//! These are the process-level behavioral tests (exit code 2 + a faithful-
//! format-pointing stderr message). The `kind()` / typed-vs-generic boundary
//! is pinned by the in-crate unit module `h10_unsorted_multi_refusal_tests`
//! in `src/cmd/export_wallet.rs`.

use assert_cmd::Command;

// Cosigner material reused from `cli_export_wallet_electrum.rs`.
const A_XPUB: &str = "xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX";
const A_FP: &str = "b8688df1";
const B_XPUB: &str = "xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6";
const B_FP: &str = "28645006";

/// `export-wallet --format <fmt> --template <tmpl>` with two cosigner slots.
/// `extra` carries per-case flags (e.g. `--multisig-path-family`).
fn export_template(fmt: &str, template: &str) -> assert_cmd::assert::Assert {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            fmt,
            "--template",
            template,
            "--threshold",
            "2",
            "--multisig-path-family",
            "bip48",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.xpub={A_XPUB}"),
            "--slot",
            &format!("@0.fingerprint={A_FP}"),
            "--slot",
            "@0.path=m/48'/0'/0'/2'",
            "--slot",
            &format!("@1.xpub={B_XPUB}"),
            "--slot",
            &format!("@1.fingerprint={B_FP}"),
            "--slot",
            "@1.path=m/48'/0'/0'/2'",
            "--output",
            "-",
        ])
        .assert()
}

const FIELDLESS: [&str; 3] = ["electrum", "coldcard-multisig", "jade"];

// ===========================================================================
// 1. `--template wsh-multi` / `sh-wsh-multi` → field-less vendor → exit 2.
// ===========================================================================

#[test]
fn template_wsh_multi_refused_exit2_for_each_fieldless_vendor() {
    for fmt in FIELDLESS {
        let out = export_template(fmt, "wsh-multi").failure().code(2);
        let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
        assert!(
            stderr.contains("UNSORTED") || stderr.contains("sortedmulti"),
            "{fmt}: stderr must explain the unsorted-multi refusal; got: {stderr}"
        );
        assert!(
            stderr.contains("descriptor"),
            "{fmt}: stderr must point to a faithful format; got: {stderr}"
        );
    }
}

#[test]
fn template_sh_wsh_multi_refused_exit2_for_each_fieldless_vendor() {
    for fmt in FIELDLESS {
        export_template(fmt, "sh-wsh-multi").failure().code(2);
    }
}

// ===========================================================================
// 2. `--from-import-json` of an unsorted wsh(multi)/sh(wsh(multi)) → exit 2.
//    The envelope is produced live via `import-wallet --format descriptor`.
// ===========================================================================

/// Round-trip an unsorted multisig descriptor through `import-wallet --json`
/// → `export-wallet --from-import-json -`. The exported leg must REFUSE for a
/// field-less vendor (exit 2).
fn import_then_export(descriptor: &str, export_fmt: &str) -> assert_cmd::assert::Assert {
    let imp = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--format",
            "descriptor",
            "--json",
            "--blob",
            "-",
        ])
        .write_stdin(descriptor.to_string())
        .assert()
        .success();
    let envelope = String::from_utf8(imp.get_output().stdout.clone()).unwrap();

    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--from-import-json",
            "-",
            "--format",
            export_fmt,
        ])
        .write_stdin(envelope)
        .assert()
}

#[test]
fn from_import_json_unsorted_wsh_multi_refused_exit2() {
    let desc = format!(
        "wsh(multi(2,[{A_FP}/48'/0'/0'/2']{A_XPUB}/<0;1>/*,[{B_FP}/48'/0'/0'/2']{B_XPUB}/<0;1>/*))"
    );
    for fmt in FIELDLESS {
        import_then_export(&desc, fmt).failure().code(2);
    }
}

#[test]
fn from_import_json_unsorted_sh_wsh_multi_refused_exit2() {
    let desc = format!(
        "sh(wsh(multi(2,[{A_FP}/48'/0'/0'/1']{A_XPUB}/<0;1>/*,[{B_FP}/48'/0'/0'/1']{B_XPUB}/<0;1>/*)))"
    );
    for fmt in FIELDLESS {
        import_then_export(&desc, fmt).failure().code(2);
    }
}

/// SORTED envelopes STILL export (BIP-67 is what these vendors implement).
#[test]
fn from_import_json_sorted_wsh_multi_still_exports() {
    let desc = format!(
        "wsh(sortedmulti(2,[{A_FP}/48'/0'/0'/2']{A_XPUB}/<0;1>/*,[{B_FP}/48'/0'/0'/2']{B_XPUB}/<0;1>/*))"
    );
    for fmt in FIELDLESS {
        import_then_export(&desc, fmt).success();
    }
}

// ===========================================================================
// 3. Direct `--descriptor 'wsh(multi(…))'` → refused with the TYPED H10
//    message (v0.70.1 Wave 1 — second arm of the guard now keys on the
//    descriptor `script_type` + unsorted `multi(` for the `template == None`
//    direct path, not just the resolved `--template`). Refused with exit 2,
//    stderr names sortedmulti / a faithful format, never silently coerced.
//    (The typed kind() boundary is also pinned in the unit module.)
// ===========================================================================

#[test]
fn direct_descriptor_unsorted_multi_refused_not_silently_coerced() {
    let desc = format!(
        "wsh(multi(2,[{A_FP}/48'/0'/0'/2']{A_XPUB}/<0;1>/*,[{B_FP}/48'/0'/0'/2']{B_XPUB}/<0;1>/*))"
    );
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "electrum",
            "--descriptor",
            &desc,
            "--output",
            "-",
        ])
        .assert()
        .failure()
        .code(2);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        stderr.contains("UNSORTED") || stderr.contains("sortedmulti"),
        "direct-descriptor unsorted multi must surface the typed unsorted-multi message; got: {stderr}"
    );
    assert!(
        stderr.contains("descriptor")
            || stderr.contains("bitcoin-core")
            || stderr.contains("sparrow"),
        "typed message must point to a faithful format; got: {stderr}"
    );
}

/// v0.70.1 (Wave 1) no-change guard — a SORTED `sortedmulti(` direct
/// descriptor to a field-less vendor is still refused (the descriptor path's
/// per-emitter `--template`-required refusal), but NOT by the typed
/// unsorted-multi message (which is specific to UNSORTED multi). Pins that the
/// new second arm does not over-refuse the BIP-67-faithful sorted case.
#[test]
fn direct_descriptor_sorted_multi_not_typed_unsorted_message() {
    let desc = format!(
        "wsh(sortedmulti(2,[{A_FP}/48'/0'/0'/2']{A_XPUB}/<0;1>/*,[{B_FP}/48'/0'/0'/2']{B_XPUB}/<0;1>/*))"
    );
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "electrum",
            "--descriptor",
            &desc,
            "--output",
            "-",
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains("UNSORTED multisig"),
        "sorted-multi direct descriptor must NOT surface the typed unsorted-multi refusal; got: {stderr}"
    );
}

// ===========================================================================
// 4. MANDATED `sortedmulti`-NOT-refused regression (false-refuse guard):
//    a SORTED template STILL exports to each field-less vendor (exit 0).
// ===========================================================================

#[test]
fn sorted_multi_template_still_exports_to_fieldless_vendors() {
    for fmt in FIELDLESS {
        export_template(fmt, "wsh-sortedmulti").success();
        export_template(fmt, "sh-wsh-sortedmulti").success();
    }
}

// ===========================================================================
// 5. `multi_a` / `sortedmulti_a` (taproot) → field-less vendor → hits the
//    EXISTING per-emitter taproot refusal, NOT the new H10 error. Disjoint
//    variant sets (§2.3 / §2.5). Asserted as a failure that does NOT mention
//    the H10 unsorted-multi wording (so a future predicate that mis-classified
//    taproot would RED here).
// ===========================================================================

#[test]
fn taproot_multi_a_hits_existing_taproot_refusal_not_h10() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "electrum",
            "--template",
            "tr-multi-a",
            "--threshold",
            "2",
            "--multisig-path-family",
            "bip87",
            "--network",
            "mainnet",
            "--taproot-internal-key",
            "nums",
            "--slot",
            &format!("@0.xpub={A_XPUB}"),
            "--slot",
            &format!("@0.fingerprint={A_FP}"),
            "--slot",
            &format!("@1.xpub={B_XPUB}"),
            "--slot",
            &format!("@1.fingerprint={B_FP}"),
        ])
        .assert()
        .failure();
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains("UNSORTED multisig"),
        "tr-multi-a must hit the taproot refusal, NOT the H10 unsorted-multi guard; got: {stderr}"
    );
}

// ===========================================================================
// 6. Single-sig + faithful formats must STILL work.
// ===========================================================================

#[test]
fn single_sig_to_fieldless_vendor_still_exports() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "export-wallet",
            "--format",
            "coldcard",
            "--template",
            "bip84",
            "--network",
            "mainnet",
            "--slot",
            "@0.xpub=zpub6qTBTNftBzVTjgVcSUw7vW5N1KQbV93Jnrw314RHGkCkSx4vk6nEWH1MJfReXi2WThvuDRiRpyT7cDoakEcZMQ1iZPgfJgQrcVMR4aJWh6S",
            "--slot",
            "@0.fingerprint=5436d724",
            "--output",
            "-",
        ])
        .assert()
        .success();
}

#[test]
fn faithful_formats_still_export_unsorted_multi() {
    // descriptor: emits the literal `multi(`.
    let out = export_template("descriptor", "wsh-multi").success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    assert!(
        stdout.contains("multi("),
        "--format descriptor must emit the literal unsorted multi(...); got: {stdout}"
    );
    // sparrow + bitcoin-core also faithful.
    export_template("sparrow", "wsh-multi").success();
    export_template("bitcoin-core", "wsh-multi").success();
}

// ===========================================================================
// 7. Restore-path regression (free consequence of the shared `emit_payload`
//    chokepoint): `restore --md1 --format electrum` of an md1 reconstructing
//    an UNSORTED wsh-multi is also refused (exit 2). `restore.rs` is NOT edited;
//    this asserts the chokepoint coverage. The md1 is produced by `bundle`.
// ===========================================================================

const C0: &str =
    "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
const C1: &str = "legal winner thank year wave sausage worth useful legal winner thank yellow";
const C2: &str = "letter advice cage absurd amount doctor acoustic avoid letter advice cage above";

/// `bundle --template <multisig> --json` → the md1 card chunks.
fn bundle_md1(template: &str) -> Vec<String> {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "bundle",
            "--template",
            template,
            "--threshold",
            "2",
            "--network",
            "mainnet",
            "--slot",
            &format!("@0.phrase={C0}"),
            "--slot",
            &format!("@1.phrase={C1}"),
            "--slot",
            &format!("@2.phrase={C2}"),
            "--json",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let v: serde_json::Value = serde_json::from_slice(&out.get_output().stdout).unwrap();
    v["md1"]
        .as_array()
        .expect("bundle --json md1 array")
        .iter()
        .map(|x| x.as_str().unwrap().to_string())
        .collect()
}

#[test]
fn restore_md1_unsorted_multi_to_fieldless_vendor_refused() {
    let md1 = bundle_md1("wsh-multi");
    for fmt in FIELDLESS {
        let mut args = vec!["restore".to_string(), "--network".into(), "mainnet".into()];
        for c in &md1 {
            args.push("--md1".into());
            args.push(c.clone());
        }
        args.push("--format".into());
        args.push(fmt.into());
        Command::cargo_bin("mnemonic")
            .unwrap()
            .args(&args)
            .assert()
            .failure()
            .code(2);
    }
}

/// Restore of a SORTED md1 still emits (BIP-67) — the restore-path guard does
/// NOT over-refuse (mirror of the export-path false-refuse guard).
#[test]
fn restore_md1_sorted_multi_to_fieldless_vendor_still_emits() {
    let md1 = bundle_md1("wsh-sortedmulti");
    for fmt in FIELDLESS {
        let mut args = vec!["restore".to_string(), "--network".into(), "mainnet".into()];
        for c in &md1 {
            args.push("--md1".into());
            args.push(c.clone());
        }
        args.push("--format".into());
        args.push(fmt.into());
        Command::cargo_bin("mnemonic")
            .unwrap()
            .args(&args)
            .assert()
            .success();
    }
}
