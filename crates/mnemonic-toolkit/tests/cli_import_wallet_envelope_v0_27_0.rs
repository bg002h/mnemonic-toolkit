//! v0.27.0 Phase 4 — `import-wallet --json` envelope rewrite (full
//! `BundleJson` shape; SPEC §3.2 + §3.2.1).
//!
//! Per `design/PLAN_v0_27_0_bsms_round_trip_and_wallet_import_handoff.md`
//! §4.4. Eight cells:
//!
//! 1. BSMS 2-line input → `bundle.mode == "watch-only"`.
//! 2. BSMS 2-line input → `bundle.ms1 == ["", "", ...]` (length N sentinel).
//! 3. BSMS 2-line input → `bundle.mk1` decodes back to original cosigner xpubs.
//! 4. BSMS 2-line input → `bundle.descriptor.is_some()` (descriptor-mode).
//! 5. Bitcoin Core multi-descriptor → array of envelopes, one per descriptor.
//! 6. `bundle_account_hardcoded_zero_for_v0_27_0_wallet_import`.
//! 7. verify-bundle round-trip via `--bundle-json <FILE>`.
//! 8. Envelope wire-shape fixture comparison.

use assert_cmd::Command;
use std::io::Write;
use std::path::{Path, PathBuf};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from("tests/fixtures/wallet_import").join(name)
}

fn run_import_file_json(path: &Path, format: &str) -> serde_json::Value {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "import-wallet",
            "--blob",
            path.to_str().unwrap(),
            "--format",
            format,
            "--json",
        ])
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("envelope JSON invalid: {e}\nstdout was:\n{stdout}"))
}

// ============================================================================
// Cell 1 — BSMS 2-line input → bundle.mode == "watch-only".
// ============================================================================

#[test]
fn bsms_2line_envelope_bundle_mode_watch_only() {
    let p = fixture_path("bsms-2line-multi-2of3.txt");
    let val = run_import_file_json(&p, "bsms");
    let bundle = &val.as_array().unwrap()[0]["bundle"];
    assert_eq!(
        bundle["mode"].as_str(),
        Some("watch-only"),
        "BSMS 2-line input is always watch-only at v0.27.0 (no seed overlay supplied); env: {val}"
    );
}

// ============================================================================
// Cell 2 — BSMS 2-line input → bundle.ms1 == ["", "", ""] (length-N sentinel).
// ============================================================================

#[test]
fn bsms_2line_envelope_ms1_length_n_empty_string_sentinel() {
    let p = fixture_path("bsms-2line-multi-2of3.txt");
    let val = run_import_file_json(&p, "bsms");
    let ms1 = val.as_array().unwrap()[0]["bundle"]["ms1"]
        .as_array()
        .expect("bundle.ms1 must be array");
    assert_eq!(ms1.len(), 3, "2-of-3 → length-3 ms1 array");
    for (i, entry) in ms1.iter().enumerate() {
        assert_eq!(
            entry.as_str(),
            Some(""),
            "ms1[{i}] must be SPEC §5.8 \"\" watch-only sentinel"
        );
    }
}

// ============================================================================
// Cell 3 — bundle.mk1 decodes back to original cosigner xpubs.
// ============================================================================

#[test]
fn bsms_envelope_mk1_decodes_back_to_original_cosigner_identity() {
    // 2-of-3 BSMS fixture: each cosigner's mk1 chunk array decodes back to
    // a KeyCard whose `origin_fingerprint` and chain-code+pubkey segments
    // match the corresponding cosigner from the source descriptor.
    //
    // NOTE: mk-codec's xpub_compact form drops the `depth` and `child_number`
    // fields and reconstructs them from `origin_path` at decode time
    // (xpub_compact.rs:85). When a fixture's source xpub has a depth or
    // child_number that doesn't match the declared origin_path (common in
    // hand-crafted test BSMS bodies where the xpub is illustrative rather
    // than a real BIP-32 derivation), the serialized `xpub.to_string()`
    // differs by those reconstructed fields. The semantically-meaningful
    // identity is the (parent_fingerprint, chain_code, public_key) triple —
    // i.e., `xpub.encode()[..]` excluding the depth/child_number bytes —
    // plus the `origin_fingerprint` (which carries the BIP-380
    // `[master_fp/path]` master fingerprint).
    let p = fixture_path("bsms-2line-multi-2of3.txt");
    let val = run_import_file_json(&p, "bsms");
    let bundle = &val.as_array().unwrap()[0]["bundle"];
    let envelope_cosigners = bundle["multisig"]["cosigners"]
        .as_array()
        .expect("bundle.multisig.cosigners");

    let mk1_outer = bundle["mk1"]
        .as_array()
        .expect("bundle.mk1 must be outer array for multisig");
    assert_eq!(
        mk1_outer.len(),
        envelope_cosigners.len(),
        "mk1 outer length must equal cosigner count"
    );

    for (i, mk1_chunks) in mk1_outer.iter().enumerate() {
        let chunk_strs: Vec<&str> = mk1_chunks
            .as_array()
            .expect("mk1[i] must be array")
            .iter()
            .map(|c| c.as_str().expect("chunk is string"))
            .collect();
        let card = mk_codec::decode(&chunk_strs)
            .unwrap_or_else(|e| panic!("mk1[{i}] decode: {e:?}"));
        // origin_fingerprint check: the BIP-380 [master_fp/path] master
        // fingerprint is preserved verbatim and is the most reliable
        // per-cosigner identity carried by mk1.
        let expected_fp = envelope_cosigners[i]["master_fingerprint"]
            .as_str()
            .expect("envelope.cosigners[i].master_fingerprint");
        let got_fp = format!(
            "{}",
            card.origin_fingerprint.expect("mk1 must carry origin_fingerprint")
        );
        assert_eq!(
            got_fp, expected_fp,
            "decoded mk1[{i}].origin_fingerprint must match envelope.cosigners[{i}].master_fingerprint"
        );

        // public_key + chain_code + parent_fingerprint identity check: these
        // are the fields actually carried in xpub_compact (xpub.serialize()
        // bytes [0..4] = version, [4] = depth, [5..9] = parent_fingerprint,
        // [9..13] = child_number, [13..45] = chain_code, [45..78] = pubkey).
        // We compare bytes [0..4], [5..9], [13..78] (skipping depth + child).
        use bitcoin::bip32::Xpub;
        use std::str::FromStr;
        let expected_xpub_str = envelope_cosigners[i]["xpub"]
            .as_str()
            .expect("envelope.cosigners[i].xpub");
        let expected_xpub = Xpub::from_str(expected_xpub_str)
            .unwrap_or_else(|e| panic!("envelope xpub[{i}] parse: {e}"));
        assert_eq!(
            card.xpub.network, expected_xpub.network,
            "mk1[{i}] xpub network mismatch"
        );
        assert_eq!(
            card.xpub.parent_fingerprint, expected_xpub.parent_fingerprint,
            "mk1[{i}] xpub parent_fingerprint mismatch"
        );
        assert_eq!(
            card.xpub.chain_code, expected_xpub.chain_code,
            "mk1[{i}] xpub chain_code mismatch"
        );
        assert_eq!(
            card.xpub.public_key, expected_xpub.public_key,
            "mk1[{i}] xpub public_key mismatch"
        );
    }
}

// ============================================================================
// Cell 4 — bundle.descriptor.is_some() (descriptor-mode).
// ============================================================================

#[test]
fn bsms_envelope_descriptor_field_is_some_descriptor_mode() {
    let p = fixture_path("bsms-2line-multi-2of3.txt");
    let val = run_import_file_json(&p, "bsms");
    let bundle = &val.as_array().unwrap()[0]["bundle"];
    let descriptor = bundle["descriptor"]
        .as_str()
        .expect("bundle.descriptor must be present (descriptor-mode)");
    assert!(
        descriptor.starts_with("sh(multi(2,"),
        "descriptor must be the source descriptor verbatim (incl `#<checksum>`); got: {descriptor}"
    );
    assert!(
        descriptor.contains("#ek6d38cp"),
        "descriptor must carry the BIP-380 #<checksum> suffix verbatim; got: {descriptor}"
    );
    // Descriptor-mode → template field is null.
    assert!(
        bundle["template"].is_null(),
        "descriptor-mode → bundle.template is null"
    );
}

// ============================================================================
// Cell 5 — Bitcoin Core multi-descriptor → array of envelopes (one per).
// ============================================================================

#[test]
fn bitcoin_core_multi_descriptor_yields_one_envelope_per_entry() {
    let p = fixture_path("core-multi-bip84.json");
    let val = run_import_file_json(&p, "bitcoin-core");
    let arr = val.as_array().expect("--json must emit array");
    // Fixture has 4 entries: receive + change for bip84 + bip49.
    assert_eq!(arr.len(), 4, "expected 4 envelope entries; got {}", arr.len());
    for env in arr {
        assert_eq!(env["source_format"].as_str(), Some("bitcoin-core"));
        assert!(
            env["bundle"]["descriptor"].is_string(),
            "each entry must carry its descriptor verbatim"
        );
    }
}

// ============================================================================
// Cell 6 — bundle_account_hardcoded_zero_for_v0_27_0_wallet_import.
// Per §3.2.1 row `account` lock — emit `0` regardless of descriptor's
// BIP-48 account index.
// ============================================================================

#[test]
fn bundle_account_hardcoded_zero_for_v0_27_0_wallet_import() {
    // bsms-2line-multi-2of3.txt encodes BIP-48 account 0 explicitly; the
    // §3.2.1 lock requires emitting `0` regardless. This cell is the
    // change-witness that future contributors don't accidentally start
    // deriving an account index from the descriptor.
    let p = fixture_path("bsms-2line-multi-2of3.txt");
    let val = run_import_file_json(&p, "bsms");
    let bundle = &val.as_array().unwrap()[0]["bundle"];
    assert_eq!(
        bundle["account"].as_u64(),
        Some(0),
        "v0.27.0 wallet-import path emits account=0 hardcoded per §3.2.1; got {}",
        bundle["account"]
    );
}

// ============================================================================
// Cell 7 — verify-bundle round-trip via --bundle-json <FILE>.
// Per Phase 4 R0 scope (memory [[feedback-verify-bundle-round-trip-per-phase-r0-scope]]):
// import-wallet's emitted envelope must round-trip through verify-bundle
// without `mismatch`. Lossy synthesis would surface as a parse/verify error.
// ============================================================================

/// Build a BSMS 2-line blob from a descriptor body. Computes a fresh
/// BIP-380 checksum (mirrors the helper in `cli_import_wallet_seed_overlay.rs`).
fn bsms_2line_from_body(body: &str) -> String {
    use miniscript::descriptor::checksum::Engine as ChecksumEngine;
    let mut e = ChecksumEngine::new();
    e.input(body).expect("checksum input must be ASCII");
    let csum = e.checksum();
    format!("BSMS 1.0\n{body}#{csum}\n")
}

#[test]
fn bsms_envelope_verify_bundle_round_trip_via_bundle_json() {
    // R0-scope per memory [[feedback-verify-bundle-round-trip-per-phase-r0-scope]].
    // The synthesized BundleJson must verify against the source descriptor
    // via `verify-bundle --bundle-json <FILE>`. Lossy synthesis (mismatched
    // ms1/mk1/md1 vs descriptor) would surface as `result: "mismatch"`.
    //
    // The BSMS body below uses real BIP-32 derivations from
    // `cli_import_wallet_seed_overlay::skip_middle_3of3_blob` — xpubs derived
    // from canonical BIP-39 test vectors at m/87'/0'/0' (BIP-87 family).
    // Real derivations are required because mk-codec's xpub_compact form
    // reconstructs `depth` and `child_number` from `origin_path`; hand-rolled
    // xpubs (e.g., bsms-2line-multi-2of3.txt) have arbitrary depth and would
    // round-trip to a different xpub serialization, masking real synthesis
    // bugs behind a fixture-quality issue. verify-bundle's path-mode
    // template dispatch re-derives expected xpubs at `m/87'/0'/0'` and
    // matches against the decoded mk1; the round-trip succeeds iff the
    // emitted envelope is internally consistent.
    let body = "wsh(sortedmulti(2,\
[73c5da0a/87'/0'/0']xpub6DBjiYnc4ewKti13Q1L35bqdodw5z3VGJnf516B3icHrEGEUcCuCG5GVQDZtH8Xmsyt3Fs9YDNwLaqjUbbRidwXZ6sxufZcr4VqqzrXvicM/<0;1>/*,\
[b8688df1/87'/0'/0']xpub6CbhrPzY2z7NcCGCGjLAJLq8iRyjUfwmdXQs66MxTVUReKqb9DpLnVJ5D1qpatZjUuPGTyxf5TYU1vA34YFE9FHB4TvfYmokYLVsyEFZFt9/<0;1>/*,\
[28645006/87'/0'/0']xpub6DB7HNqw6CZojxN85NuFTPWZhi2FagSnexPS1rv3nYQhngkmdHgb7iebYvTFmFKKDA3ozf5yezDsCH6cXAw3WZijviSZtZC2hjHn2uazz4z/<0;1>/*))";
    let blob = bsms_2line_from_body(body);

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["import-wallet", "--blob", "-", "--format", "bsms", "--json"])
        .write_stdin(blob)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let val: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let bundle = &val.as_array().unwrap()[0]["bundle"];
    let cosigners = bundle["multisig"]["cosigners"]
        .as_array()
        .expect("bundle.multisig.cosigners must be array");

    let tmpdir = tempfile::tempdir().expect("tempdir");
    let bundle_path = tmpdir.path().join("bundle.json");
    let mut f = std::fs::File::create(&bundle_path).expect("create temp file");
    f.write_all(
        serde_json::to_string_pretty(bundle)
            .expect("bundle reserialize")
            .as_bytes(),
    )
    .expect("write bundle");
    drop(f);

    let mut args: Vec<String> = vec![
        "verify-bundle".into(),
        "--network".into(),
        "mainnet".into(),
        "--template".into(),
        "wsh-sortedmulti".into(),
        "--multisig-path-family".into(),
        "bip87".into(),
        "--threshold".into(),
        "2".into(),
        "--account".into(),
        "0".into(),
        "--bundle-json".into(),
        bundle_path.to_str().unwrap().to_string(),
        "--json".into(),
    ];
    for (i, c) in cosigners.iter().enumerate() {
        let xpub = c["xpub"].as_str().expect("cosigner xpub");
        let fp = c["master_fingerprint"]
            .as_str()
            .expect("cosigner master_fingerprint");
        args.push("--slot".into());
        args.push(format!("@{i}.xpub={xpub}"));
        args.push("--slot".into());
        args.push(format!("@{i}.fingerprint={fp}"));
    }

    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args(&args)
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let verify_val: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("verify-bundle JSON invalid: {e}\nstdout was:\n{stdout}"));
    assert_eq!(
        verify_val["result"].as_str(),
        Some("ok"),
        "verify-bundle on synthesized envelope must return result=\"ok\"; \
         lossy synthesis would surface as result=\"mismatch\"; got: {verify_val}"
    );
}

// ============================================================================
// Cell 8 — envelope wire-shape fixture byte-exact comparison.
// Pins the v0.27.0 envelope wire shape (single-source-of-truth artifact for
// downstream consumers). Future schema changes update fixture in lockstep.
// ============================================================================

#[test]
fn envelope_v0_27_0_wire_shape_fixture_byte_exact() {
    let p = fixture_path("bsms-2line-multi-2of3.txt");
    let val = run_import_file_json(&p, "bsms");
    let actual = serde_json::to_string_pretty(&val)
        .expect("re-serialize envelope")
        + "\n";

    let fixture = fixture_path("envelope_v0_27_0.json");
    if std::env::var_os("UPDATE_FIXTURES").is_some() {
        std::fs::write(&fixture, &actual).expect("write fixture");
        return;
    }

    let expected =
        std::fs::read_to_string(&fixture).expect("envelope_v0_27_0.json fixture must exist");
    assert_eq!(
        actual, expected,
        "envelope wire shape drift detected — regenerate with `UPDATE_FIXTURES=1 cargo test \
         envelope_v0_27_0_wire_shape_fixture_byte_exact` after intended schema changes"
    );
}
