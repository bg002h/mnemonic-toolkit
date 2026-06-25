//! KATs for the frozen stripe zero-padding rule (plan §4.1 / M4).

use wc_codec::pad::pad_payload_to;

#[test]
fn pads_shorter_on_the_right() {
    assert_eq!(pad_payload_to(&[1, 2, 3], 6), vec![1, 2, 3, 0, 0, 0]);
    assert_eq!(pad_payload_to(&[0xFF], 4), vec![0xFF, 0, 0, 0]);
}

#[test]
fn already_at_length_unchanged() {
    assert_eq!(pad_payload_to(&[1, 2, 3], 3), vec![1, 2, 3]);
    assert_eq!(pad_payload_to(&[], 0), Vec::<u8>::new());
}

#[test]
fn empty_padded_to_n() {
    assert_eq!(pad_payload_to(&[], 3), vec![0, 0, 0]);
}

#[test]
fn array_wide_max_example() {
    // Simulate three xpub payloads of different lengths padded to the array-wide
    // max (5 here); each becomes exactly 5 bytes, zero-extended on the right.
    let payloads: [&[u8]; 3] = [&[1, 2], &[3, 4, 5, 6, 7], &[8, 9, 10]];
    let max = payloads.iter().map(|p| p.len()).max().unwrap();
    assert_eq!(max, 5);
    let padded: Vec<Vec<u8>> = payloads.iter().map(|p| pad_payload_to(p, max)).collect();
    assert_eq!(padded[0], vec![1, 2, 0, 0, 0]);
    assert_eq!(padded[1], vec![3, 4, 5, 6, 7]);
    assert_eq!(padded[2], vec![8, 9, 10, 0, 0]);
    for p in &padded {
        assert_eq!(p.len(), max);
    }
}

#[test]
#[should_panic(expected = "target_len")]
fn panics_when_target_below_input() {
    let _ = pad_payload_to(&[1, 2, 3], 2);
}
