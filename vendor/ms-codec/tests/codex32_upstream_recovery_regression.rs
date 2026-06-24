//! Named regression anchor for upstream `apoelstra/rust-codex32` PR #2 (open,
//! Dec 2025; updated Apr 2026; unmerged): *"Serialization to seed & subsequent
//! re-serialization to shares breaks shamir recover result."*
//!
//! The upstream bug: take a share, store only its `data` bytes + metadata,
//! reconstruct it via `Codex32String::from_seed`, then recover — the recovered
//! secret DIFFERS from the original (a padding bug; the last nibble flips on a
//! 16-byte / 128-bit secret, e.g. `…4979 9` vs `…4979 f`).
//!
//! **Our pipeline is NOT exposed.** `combine_shares` recovers via
//! `Codex32String::interpolate_at` over the parsed share STRINGS
//! (`ms_codec::shares::combine_shares`), never the decompose-to-`data` →
//! `from_seed` reload that the bug requires — we carry the full codex32 share
//! string end-to-end. The BROAD cross-length guard already lives in
//! `tests/spike_kofn.rs` (claim b) and `shares.rs::combine_round_trip_entr_and_mnem_all_lengths`.
//!
//! This file is the NAMED anchor for the *specific* upstream bug: it pins that
//! the bug's EXACT 16-byte secret round-trips correctly through our
//! split→combine across every 2-of-3 share pair. If a future `codex32` bump (or
//! the long-slated "rewrite") reintroduces the padding bug on our path, this
//! fails loudly with a pointer to the upstream PR.
//!
//! See `design/FOLLOWUPS.md::rust-codex32-upstream-pr2-recovery-bug-not-exposed`.

use ms_codec::{combine_shares, encode_shares, Payload, Tag, Threshold};

/// The exact 16-byte secret from upstream PR #2's failing assertion.
const PR2_SECRET_HEX: &str = "10cbc41852b76438e5781f2cefb4979f";

fn hex_to_bytes(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).expect("valid hex"))
        .collect()
}

#[test]
fn upstream_pr2_exact_secret_round_trips_across_all_pairs() {
    let secret_bytes = hex_to_bytes(PR2_SECRET_HEX);
    assert_eq!(secret_bytes.len(), 16, "PR#2 secret is 16 bytes / 128-bit");
    let secret = Payload::Entr(secret_bytes);

    // 2-of-3 split.
    let shares = encode_shares(Tag::ENTR, Threshold::new(2).unwrap(), 3, &secret).unwrap();
    assert_eq!(shares.len(), 3);

    // Every 2-of-3 subset MUST recover the EXACT original secret — the upstream
    // PR#2 padding bug must never manifest on our interpolate-over-strings path.
    for (a, b) in [(0usize, 1usize), (1, 2), (0, 2)] {
        let subset = [shares[a].clone(), shares[b].clone()];
        let (tag, recovered) = combine_shares(&subset).unwrap();
        assert_eq!(tag, Tag::ENTR);
        assert_eq!(
            recovered, secret,
            "upstream rust-codex32 PR#2 padding bug manifested on our path: \
             pair ({a},{b}) recovered a different secret than the original"
        );
    }
}
