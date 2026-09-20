//! A path the operator supplies must reach the CARD.
//!
//! `bundle --descriptor` accepted `--slot @N.path=`, derived with it, and then
//! dropped it: the emitted md1 carried one shared EMPTY origin, so the engraved
//! card recorded where NO key lived. Three guards keyed on `is_non_canonical`
//! did it between them -- `bind_descriptor_mode_paths` returned early, its
//! override loop skipped xpub-bearing slots, and the propagation back into the
//! parsed descriptor was gated. Each was written for default-INFERENCE, which
//! really is non-canonical-only; each also silently took path BINDING with it.
//!
//! WHY NOTHING CAUGHT IT, which is the reason this file exists. Every slot
//! dropped its path identically, so `verify-bundle` round-tripped the card to
//! itself and agreed. And addresses derive from the xpubs a card CARRIES, never
//! from the origin it DECLARES, so every address check passed either way. The
//! card was wrong in the one field no other check reads.
//!
//! So these assert on the ORIGIN as stored, not on exit codes and not on
//! addresses -- the two things that were already green while this was broken.

use assert_cmd::Command;

/// Two accounts of ONE master: distinct keys, distinct paths, shared
/// fingerprint. (Operator ruling 2026-09-19: reusing a seed for different keys
/// at different keypaths is fine; reusing a KEY is not.)
const FP: &str = "5436d724";
const X0: &str = "xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9";
const X1: &str = "xpub6Buxw9MmbkJr8dFGbbbjY46MzzbM8MCosN5AxgxVEstcQMYcAn7oV8DvwYouSbixK4zhej2oTUoMkFD6FaHu7tuZPLiGQ7VKcBcj8fmj4g9";

fn emit_md1(p0: &str, p1: &str) -> Vec<String> {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("--allow-argv-secret")
        .args([
            "bundle",
            "--network",
            "mainnet",
            // CANONICAL on purpose: wsh(sortedmulti) HAS a canonical origin, and
            // the canonical arm is the one that dropped the paths.
            "--descriptor",
            "wsh(sortedmulti(2,@0,@1))",
            "--slot",
            &format!("@0.xpub={X0}"),
            "--slot",
            &format!("@0.fingerprint={FP}"),
            "--slot",
            &format!("@0.path={p0}"),
            "--slot",
            &format!("@1.xpub={X1}"),
            "--slot",
            &format!("@1.fingerprint={FP}"),
            "--slot",
            &format!("@1.path={p1}"),
            "--no-engraving-card",
        ])
        .assert()
        .success();
    String::from_utf8(out.get_output().stdout.clone())
        .unwrap()
        .lines()
        .filter(|l| l.trim_start().starts_with("md1"))
        .map(|l| l.replace(' ', ""))
        .collect()
}

/// Read the per-`@N` origin paths back off the emitted card.
fn origins_on_card(md1: &[String]) -> Vec<String> {
    let refs: Vec<&str> = md1.iter().map(|s| s.as_str()).collect();
    let d = md_codec::chunk::reassemble(&refs).expect("emitted md1 must decode");
    let expanded = md_codec::canonicalize::expand_per_at_n(&d).expect("expand");
    expanded
        .iter()
        .map(|e| {
            e.origin_path
                .components
                .iter()
                .map(|c| format!("{}{}", c.value, if c.hardened { "'" } else { "" }))
                .collect::<Vec<_>>()
                .join("/")
        })
        .collect()
}

#[test]
fn per_slot_paths_reach_the_card_on_a_canonical_descriptor() {
    let md1 = emit_md1("m/48'/0'/0'/2'", "m/48'/0'/1'/2'");
    let got = origins_on_card(&md1);
    assert_eq!(
        got,
        vec!["48'/0'/0'/2'".to_string(), "48'/0'/1'/2'".to_string()],
        "the card must record BOTH supplied origins; an empty or shared origin \
         means the operator's path was accepted and discarded"
    );
}

/// The anti-tautology control. If the emitter ever collapses both slots to one
/// path again, the test above could still pass with a fixture whose two paths
/// happened to be equal -- so pin that the card DISTINGUISHES them.
#[test]
fn the_two_slots_do_not_share_one_origin() {
    let md1 = emit_md1("m/48'/0'/0'/2'", "m/48'/0'/1'/2'");
    let got = origins_on_card(&md1);
    assert_eq!(got.len(), 2, "two slots");
    assert_ne!(
        got[0], got[1],
        "the two slots declare DIFFERENT accounts, so the card must not collapse \
         them to one shared origin"
    );
    assert!(
        !got[0].is_empty() && !got[1].is_empty(),
        "neither origin may be empty, which is what the bug emitted: {got:?}"
    );
}

/// THE CRITICAL FOUND IN WHOLE-DIFF REVIEW (2026-09-19).
///
/// Binding `--slot @N.path=` on canonical descriptors meant this function stopped
/// returning early for them -- and default-INFERENCE lived past that early
/// return in the `Divergent` arm. A canonical descriptor with PARTIAL inline
/// origins reaches that arm with one empty entry, and the arm invented an
/// origin for it. For a phrase slot the key is then DERIVED at the invented
/// path, so the same command line produced a different wallet:
///
///   shipped: @1 [3f635a63/m]            id 7dd635d7...  bc1qq3p989...
///   broken:  @1 [3f635a63/48'/0'/0'/2']  id a28485fe...  bc1qt2yw8z4...
///
/// This asserts the shape a REVIEWER had to construct, because no existing
/// test covered a canonical descriptor with SOME slots annotated: an
/// un-annotated slot must be left exactly as the operator left it.
#[test]
fn a_canonical_descriptor_with_partial_inline_origins_invents_nothing() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .arg("--allow-argv-secret")
        .args([
            "bundle",
            "--network",
            "mainnet",
            // CANONICAL, and only @0 carries an inline origin.
            "--descriptor",
            "wsh(sortedmulti(2,[73c5da0a/48'/0'/0'/2']@0,@1))",
            "--slot",
            "@0.phrase=abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
            "--slot",
            "@1.phrase=zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong",
            "--no-engraving-card",
        ])
        .assert()
        .success();
    let md1: Vec<String> = String::from_utf8(out.get_output().stdout.clone())
        .unwrap()
        .lines()
        .filter(|l| l.trim_start().starts_with("md1"))
        .map(|l| l.replace(' ', ""))
        .collect();
    let got = origins_on_card(&md1);
    assert_eq!(got.len(), 2, "two slots");
    assert_eq!(got[0], "48'/0'/0'/2'", "@0's inline origin must survive");
    assert!(
        got[1].is_empty(),
        "@1 carried NO origin and supplied NO --slot path, so none may be \
         invented for it -- inventing one changes the derived key and the \
         wallet. got {:?}",
        got[1]
    );
}
