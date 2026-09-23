//! F-642 piece 3: `mnemonic repair` converges on `md repair` (md-cli 0.19.0,
//! descriptor-mnemonic `cf35d61a`, `crates/md-cli/src/cmd/repair.rs`) for an
//! md1 card whose WIRE VERSION this build cannot read.
//!
//! BCH correction is version-agnostic (`md_codec::correct_chunks`), so a
//! correctable SINGLE-STRING card at an unreadable version keeps its
//! correction. RULING 8: it exits 4 (VERIFY-ME), NOT 5 -- the toolkit gives 5
//! only to a correction something verified, and nothing past BCH checks this
//! one. This is the one deliberate divergence from `md repair`, which exits 5.
//! Ruling 7 (descriptor-mnemonic
//! `23203195`): a MULTI-string set at such a version still exits 2 with empty
//! stdout, since a build cannot read the chunk-header layout of a version it
//! does not support. A CLEAN card at an unreadable version is not "already
//! valid": exit 2, never 0.
//!
//! Before F-642 (measured at toolkit `0f92682f`) all three exited 2: the
//! correction was computed inside `decode_with_correction` and discarded.
//!
//! Fixtures are md-cli's own (`tests/cli_repair_unsupported_version.rs` at
//! `cf35d61a`), which mirror md-codec's `tests/correct_chunks.rs`.

#![allow(missing_docs)]

use std::process::Command as StdCommand;

fn mnemonic(args: &[&str]) -> (String, String, i32) {
    let out = StdCommand::new(assert_cmd::cargo::cargo_bin("mnemonic"))
        .args(args)
        .output()
        .expect("invoke mnemonic");
    (
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
        out.status.code().expect("mnemonic exited normally"),
    )
}

/// Version 12, one correctable error at data position 0.
const V12_ONE_ERROR: &str = "md1qzfdsssjjtvyyw2fdssj54qqxppcgscu5e7m9jgawlhg";
/// The same card, corrected: BCH-valid, zero errors, version 12.
const V12_CLEAN: &str = "md1uzfdsssjjtvyyw2fdssj54qqxppcgscu5e7m9jgawlhg";
/// A `md-codec-v0.16.2` string (pre-v0.30, header reads as version 0),
/// BCH-clean.
const LEGACY_CLEAN: &str = "md1qppqqxzxpp29gtcfh4dhmh72l6atuttfxe3cw2xenm";
/// A chunked card whose header version reads 9 (odd, above 8), one error.
const V9_ONE_ERROR: &str = "md1q4frpqq9q2tvyyy5jmpprj5qqcyxppgqwcudeey7atgd5";
const V9_CLEAN: &str = "md1n4frpqq9q2tvyyy5jmpprj5qqcyxppgqwcudeey7atgd5";

/// `LEGACY_CLEAN` with one substitution at data position 7 (`x` -> `q`).
fn legacy_one_error() -> String {
    let mut s: Vec<char> = LEGACY_CLEAN.chars().collect();
    assert_eq!(s[3 + 7], 'x', "fixture moved");
    s[3 + 7] = 'q';
    s.into_iter().collect()
}

#[test]
fn a_single_string_correction_on_an_unsupported_version_is_kept_and_exits_4() {
    let (out, err, code) = mnemonic(&["repair", "--md1", V12_ONE_ERROR]);
    assert_eq!(
        code, 4,
        "VERIFY-ME (ruling 8): not the atomic-fail 2, not the verified 5: {err}"
    );
    assert!(
        out.lines().any(|l| l == V12_CLEAN),
        "the corrected card must reach stdout: {out}"
    );
    assert!(out.contains("position 0: 'q' -> 'u'"), "the report: {out}");
    assert!(
        err.contains("wire version 12") && err.contains("accepted: 4, 8"),
        "stderr names the version and the accepted set: {err}"
    );
    assert!(err.contains("newer md"), "v12 is even and above 8: {err}");
    // No output-class advisory: there is no descriptor to classify.
    assert!(!err.contains("note: stdout is"), "{err}");
}

#[test]
fn the_json_report_carries_the_unreadable_version_verdict() {
    let (out, err, code) = mnemonic(&["repair", "--json", "--md1", V12_ONE_ERROR]);
    assert_eq!(code, 4, "{err}");
    let v: serde_json::Value =
        serde_json::from_str(&out).unwrap_or_else(|e| panic!("not JSON ({e}): {out}"));
    assert_eq!(v["kind"], "md1", "{v}");
    assert_eq!(v["verdict"], "unreadable_version", "{v}");
    assert_eq!(v["corrected_chunks"][0], V12_CLEAN, "{v}");
    assert_eq!(v["repairs"].as_array().map(Vec::len), Some(1), "{v}");
}

#[test]
fn a_clean_v12_card_exits_2_with_empty_stdout() {
    // Nothing to correct, and a version this build cannot read: NOT "already
    // valid" (never the success path's `repairs ? 5 : 0`).
    let (out, err, code) = mnemonic(&["repair", "--md1", V12_CLEAN]);
    assert_eq!(code, 2, "{err}");
    assert!(out.is_empty(), "{out}");
    assert!(err.contains("got 12"), "{err}");
}

/// Ruling 7: a genuine multi-chunk set at an unsupported version (all three
/// chunk headers rewritten to 12, BCH re-wrapped; one correctable error in
/// chunk 1) exits 2 with empty stdout.
#[test]
fn a_multi_chunk_set_at_an_unsupported_version_exits_2() {
    // `md encode` of an 8-key wsh multi with 8 fingerprints: three v4 chunks.
    let v4 = [
        "md1fsyk8pq9p6tvyyy5jmpprjjtvyy49ykcgfw2fdssnj2fdssnk2gh20njr9zxysyn",
        "md1fsyk8pq2mpp855jmpp8u4qqxppsfc989mse3sq3zyg3zfzyg3zywcpgwuu5knxhy",
        "md1fsyk8pq3rxvenxd5g3zygj924242kkvenxvm8wamhwlc3zyg3qqlprv3746tu2us",
    ];
    let mut damaged: Vec<String> = v4
        .iter()
        .map(|c| {
            let (mut bytes, bits) = md_codec::codex32::unwrap_string(c).unwrap();
            bytes[0] = (12 << 4) | (bytes[0] & 0x0f);
            md_codec::codex32::wrap_payload(&bytes, bits).unwrap()
        })
        .collect();
    let mut chars: Vec<char> = damaged[1].chars().collect();
    chars[10] = if chars[10] == 'q' { 'p' } else { 'q' };
    damaged[1] = chars.into_iter().collect();
    let mut args = vec!["repair"];
    for d in &damaged {
        args.push("--md1");
        args.push(d.as_str());
    }
    let (out, err, code) = mnemonic(&args);
    assert_eq!(code, 2, "{err}");
    assert!(out.is_empty(), "D28: nothing on stdout: {out}");
    assert!(!err.contains("newer md"), "{err}");
}

#[test]
fn a_legacy_card_is_corrected_without_the_newer_md_advice() {
    let bad = legacy_one_error();
    let (out, err, code) = mnemonic(&["repair", "--md1", &bad]);
    assert_eq!(code, 4, "{err}");
    assert!(out.contains("position 7: 'q' -> 'x'"), "{out}");
    assert!(out.lines().any(|l| l == LEGACY_CLEAN), "{out}");
    assert!(err.contains("wire version 0"), "{err}");
    assert!(
        !err.contains("newer md"),
        "version 0 is not from the future: {err}"
    );
    assert!(err.contains("pre-v0.30"), "{err}");
}

#[test]
fn an_odd_version_above_8_is_not_sent_to_a_newer_md() {
    let (out, err, code) = mnemonic(&["repair", "--md1", V9_ONE_ERROR]);
    assert_eq!(code, 4, "{err}");
    assert!(out.lines().any(|l| l == V9_CLEAN), "{out}");
    assert!(err.contains("wire version 9"), "{err}");
    assert!(!err.contains("newer md"), "9 is odd: {err}");
    assert!(err.contains("pre-v0.30 or misread"), "{err}");
}
