//! Deterministic, re-runnable seed-corpus generator for the `descriptor_parse`
//! fuzz target. Toolkit phase of the constellation stress-fuzz program.
//!
//! Run with (the `--cfg fuzzing` flag activates the `parse_descriptor` lib
//! mount — see `crates/mnemonic-toolkit/src/lib.rs`; plain `cargo test` cannot
//! see it because the mount is gated out):
//!
//!     cd fuzz && RUSTFLAGS="--cfg fuzzing" \
//!       cargo +nightly-2026-04-27 test --test gen_corpus
//!
//! It (1) writes a fixed set of descriptor-string seed files into the cargo-fuzz
//! default `corpus/descriptor_parse/` layout, and (2) — THE GATE — asserts every
//! `VALID` seed passes the EXACT call the target uses
//! (`parse_descriptor(seed, &[], &[])` → `Ok`). A "valid" seed that does not
//! parse is a generation bug and fails loudly. The `MALFORMED` seeds need only
//! NOT panic (the target's never-panic charter); the gate asserts they parse to
//! `Err` (panics would unwind the test).
//!
//! Determinism: `parse_descriptor` is a pure function of its string input, so
//! these seed files are byte-identical every run — re-running never churns the
//! committed corpus.
//!
//! NOTE on accepted shapes: with EMPTY key/fingerprint binding slices (what the
//! target passes), `parse_descriptor` accepts ONLY `@N`-template descriptors;
//! concrete-key forms (raw xpub / hex pubkey) are rejected up front with
//! "descriptor must contain at least one @N placeholder". So every VALID seed
//! below is an `@N` template.

use std::fs;
use std::path::{Path, PathBuf};

use mnemonic_toolkit::parse_descriptor::parse_descriptor;

const H32: &str = "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20";

/// VALID `@N`-template seeds: one per major descriptor shape the closure walks
/// (single-key script types, k-of-n multisig variants, taproot NUMS multi_a,
/// and miniscript timelock/hashlock fragments).
fn valid_seeds() -> Vec<(&'static str, String)> {
    vec![
        ("pkh", "pkh(@0/<0;1>/*)".to_string()),
        ("wpkh", "wpkh(@0/<0;1>/*)".to_string()),
        ("sh_wpkh", "sh(wpkh(@0/<0;1>/*))".to_string()),
        (
            "sh_wsh_multi_2of2",
            "sh(wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*)))".to_string(),
        ),
        (
            "wsh_multi_2of3",
            "wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))".to_string(),
        ),
        (
            "wsh_sortedmulti_2of2",
            "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))".to_string(),
        ),
        (
            "wsh_sortedmulti_annotated_2of2",
            "wsh(sortedmulti(2,@0[deadbeef/48'/0'/0'/2']/<0;1>/*,\
             @1[cafef00d/48'/0'/0'/2']/<0;1>/*))"
                .to_string(),
        ),
        ("tr_keyonly", "tr(@0/<0;1>/*)".to_string()),
        (
            "tr_nums_multi_a_2of2",
            "tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<0;1>/*))".to_string(),
        ),
        (
            "tr_nums_sortedmulti_a_2of2",
            "tr(NUMS,sortedmulti_a(2,@0/<0;1>/*,@1/<0;1>/*))".to_string(),
        ),
        (
            "wsh_ms_timelock",
            "wsh(and_v(v:pk(@0/<0;1>/*),older(144)))".to_string(),
        ),
        (
            "wsh_ms_hashlock",
            format!("wsh(and_v(v:pk(@0/<0;1>/*),sha256({H32})))"),
        ),
        (
            "wsh_ms_hybrid_or_d",
            format!("wsh(or_d(pk(@0/<0;1>/*),and_v(v:sha256({H32}),older(144))))"),
        ),
        (
            "wsh_ms_thresh",
            "wsh(thresh(2,pk(@0/<0;1>/*),s:pk(@1/<0;1>/*),sln:older(144)))".to_string(),
        ),
    ]
}

/// MALFORMED seeds: arbitrary near-grammar strings. The target's primary oracle
/// is never-panic; these exercise the error paths. The gate only asserts they
/// return `Err` (a panic would unwind this test and fail it).
fn malformed_seeds() -> Vec<(&'static str, &'static str)> {
    vec![
        ("empty", ""),
        ("unbalanced_wsh", "wsh("),
        ("garbage", "not a descriptor"),
        ("k_gt_n", "wsh(multi(99,@0/<0;1>/*))"),
        ("bare_tr_no_key", "tr()"),
        ("no_placeholder_concrete", "wpkh(0203deadbeef)"),
    ]
}

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("corpus")
        .join("descriptor_parse")
}

fn write_seed(dir: &Path, name: &str, bytes: &[u8]) {
    fs::create_dir_all(dir).expect("create corpus dir");
    fs::write(dir.join(name), bytes).expect("write seed");
}

#[test]
fn gen_corpus() {
    let dir = corpus_dir();

    let mut valid_count = 0usize;
    let mut malformed_count = 0usize;

    // --- VALID seeds: each must parse Ok via the SAME call the target uses. ---
    for (name, s) in valid_seeds() {
        parse_descriptor(&s, &[], &[]).unwrap_or_else(|e| {
            panic!("gen-corpus GATE: valid seed {name} ({s:?}) does not parse: {e}")
        });
        write_seed(&dir, &format!("{name}.desc"), s.as_bytes());
        valid_count += 1;
    }

    // --- MALFORMED seeds: must NOT panic; asserted to parse Err. ---
    for (name, s) in malformed_seeds() {
        assert!(
            parse_descriptor(s, &[], &[]).is_err(),
            "gen-corpus: malformed seed {name} ({s:?}) unexpectedly parsed Ok"
        );
        write_seed(&dir, &format!("{name}.bad"), s.as_bytes());
        malformed_count += 1;
    }

    assert!(valid_count >= 10, "expected at least 10 valid seeds");
    assert!(malformed_count >= 4, "expected several malformed seeds");

    eprintln!(
        "gen-corpus wrote: valid={valid_count}, malformed={malformed_count} \
         (total {})",
        valid_count + malformed_count
    );
}
