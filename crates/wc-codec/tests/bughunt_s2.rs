//! Bug-hunt triage RED rows, wc-codec, slice S2: bugs 7 and 8 of
//! `design/agent-reports/bughunt-2026-10-31-tk.md`. Each test asserts the
//! CORRECT behaviour, so it FAILS on unmodified code if the bug is real.

use wc_codec::sync::{interleave, sync_classify, SyncOutcome};
use wc_codec::{decode, encode, EncodeOpts, SourceKind, WcError};

// ---------------------------------------------------------------------------
// #7 -- `classify_single_insertion` H1 branch accepts the first checkpoint
// whose marker and index parse (no CRC check, no check of later checkpoints),
// so a data word that merely LOOKS like a checkpoint yields a misaligned grid
// labelled `Aligned` whose wrong positions are not erased.
//
// Input (report): k = 16, data[i] = (i*37+3) & 0xFF, data[4] = 0x500; take
// interleave(data), flip bit 0 of word 0, insert the word 7 at index 12.
// ---------------------------------------------------------------------------

#[test]
#[ignore = "deferred-minor: wc-codec-sync-h1-unverified-later-checkpoints"]
fn bughunt_tk_07_single_insertion_never_aligned_with_unerased_wrong_positions() {
    let k = 16usize;
    let mut data: Vec<u16> = (0..k).map(|i| ((i * 37 + 3) & 0xFF) as u16).collect();
    data[4] = 0x500;
    let clean = interleave(&data);
    let mut recv = clean.clone();
    recv[0] ^= 1;
    recv.insert(12, 7);
    match sync_classify(&recv, k) {
        SyncOutcome::Aligned { grid, erasures } => {
            let bad: Vec<usize> = (0..grid.len())
                .filter(|&i| grid[i] != clean[i] && !erasures.contains(&i))
                .collect();
            assert!(
                bad.is_empty(),
                "Aligned grid has wrong positions {bad:?} that are not in erasures {erasures:?} \
                 (grid {grid:?} vs clean {clean:?})"
            );
        }
        // Refusing, or reporting candidates, is acceptable behaviour.
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// #8 -- `encode` accepts `u_slots >= 8`, but the GEOM header stores U in 3
// bits, so the card is written with U wrapped (8 -> 0) and 16 ledger words,
// and can never be decoded. Correct: reject, or produce a card that decodes.
// ---------------------------------------------------------------------------

#[test]
fn bughunt_tk_08_u_slots_8_rejected_or_roundtrips() {
    let payload = [0x5Au8; 33];
    let opts = EncodeOpts {
        u_slots: 8,
        ..Default::default()
    };
    assert!(
        matches!(
            encode(SourceKind::Mk1Xpub, &payload, 264, &opts),
            Err(WcError::InvalidParams)
        ),
        "encode must refuse u_slots=8 with InvalidParams"
    );
    // Control: u_slots=7 (the 3-bit maximum) encodes and decodes back.
    let opts7 = EncodeOpts {
        u_slots: 7,
        ..Default::default()
    };
    let words = encode(SourceKind::Mk1Xpub, &payload, 264, &opts7).expect("u_slots=7 encodes");
    let refs: Vec<&str> = words.to_vec();
    let d = decode(&refs).expect("u_slots=7 decodes");
    assert_eq!(d.payload, payload.to_vec());
}
