//! Wire-byte-identity parity gate for the vendored codex32 (Cycle-B, shape A).
//!
//! The single most load-bearing invariant of the codex32 inline-vendor is that
//! the encoding paths (`from_seed` / `from_string` / `interpolate_at` /
//! `Parts::data` / the BCH `checksum` engine / the GF(32) `field` tables) are
//! copied BYTE-FOR-BYTE from `codex32 = "=0.1.0"` and never touched.
//!
//! This test pins the inlined `ms_codec::codex32` surface against:
//!   1. The BIP-93-published share/seed strings (upstream `bip_vector_2/3/4/5`),
//!      hard-coded as literals — so it pins to the BIP, NOT to itself.
//!   2. A `from_seed` golden corpus captured ONCE from the PRE-vendor
//!      `codex32 = "=0.1.0"` build (all five entropy lengths 16/20/24/28/32),
//!      pasted as literals — the "fixed seed → identical string pre/post-vendor"
//!      KAT.
//!
//! If ANY assertion here fails the vendor copy DIVERGED from upstream encoding —
//! STOP, do not patch around it (the spike_kofn "STOP, do not patch" rule).

use ms_codec::codex32::{Codex32String, Fe};

fn hex(data: &[u8]) -> String {
    let mut ret = String::new();
    for byte in data {
        ret.push_str(&format!("{byte:02x}"));
    }
    ret
}

/// BIP-93 §2: interpolate_at over the two published shares reproduces the
/// published share-D and the recovered secret-at-S byte-identically.
#[test]
fn bip_vector_2_interpolate() {
    let share_ac = [
        Codex32String::from_string("MS12NAMEA320ZYXWVUTSRQPNMLKJHGFEDCAXRPP870HKKQRM".into())
            .unwrap(),
        Codex32String::from_string("MS12NAMECACDEFGHJKLMNPQRSTUVWXYZ023FTR2GDZMPY6PN".into())
            .unwrap(),
    ];

    let share_d = Codex32String::interpolate_at(&share_ac, Fe::D).unwrap();
    assert_eq!(
        share_d.to_string(),
        "MS12NAMEDLL4F8JLH4E5VDVULDLFXU2JHDNLSM97XVENRXEG"
    );

    let seed = Codex32String::interpolate_at(&share_ac, Fe::S).unwrap();
    assert_eq!(
        seed.to_string(),
        "MS12NAMES6XQGUZTTXKEQNJSJZV4JV3NZ5K3KWGSPHUH6EVW"
    );
    assert_eq!(
        hex(&seed.parts().data()),
        "d1808e096b35b209ca12132b264662a5"
    );
}

/// BIP-93 §3: three shares interpolate to the three published D/E/F shares.
#[test]
fn bip_vector_3_interpolate() {
    let share_sac = [
        Codex32String::from_string("ms13cashsllhdmn9m42vcsamx24zrxgs3qqjzqud4m0d6nln".into())
            .unwrap(),
        Codex32String::from_string("ms13casha320zyxwvutsrqpnmlkjhgfedca2a8d0zehn8a0t".into())
            .unwrap(),
        Codex32String::from_string("ms13cashcacdefghjklmnpqrstuvwxyz023949xq35my48dr".into())
            .unwrap(),
    ];

    let share_def = [
        Codex32String::interpolate_at(&share_sac, Fe::D).unwrap(),
        Codex32String::interpolate_at(&share_sac, Fe::E).unwrap(),
        Codex32String::interpolate_at(&share_sac, Fe::F).unwrap(),
    ];
    assert_eq!(
        share_def[0].to_string(),
        "ms13cashd0wsedstcdcts64cd7wvy4m90lm28w4ffupqs7rm"
    );
    assert_eq!(
        share_def[1].to_string(),
        "ms13casheekgpemxzshcrmqhaydlp6yhms3ws7320xyxsar9"
    );
    assert_eq!(
        share_def[2].to_string(),
        "ms13cashf8jh6sdrkpyrsp5ut94pj8ktehhw2hfvyrj48704"
    );
}

/// BIP-93 §4: `from_seed` of the 32-byte "leet" seed reproduces the published
/// long-seed string byte-identically (the exact upstream `bip_vector_4` golden).
#[test]
fn bip_vector_4_from_seed() {
    #[rustfmt::skip]
    let seed_b = [
        0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99, 0x88,
        0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x00,
        0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99, 0x88,
        0x77, 0x66, 0x55, 0x44, 0x33, 0x22, 0x11, 0x00,
    ];
    let seed = Codex32String::from_seed("ms", 0, "leet", Fe::S, &seed_b).unwrap();
    assert_eq!(
        seed.to_string(),
        "ms10leetsllhdmn9m42vcsamx24zrxgs3qrl7ahwvhw4fnzrhve25gvezzyqqtum9pgv99ycma"
    );
    assert_eq!(seed.parts().data(), seed_b);
}

/// BIP-93 §5: the published long string decodes to the published hex data.
#[test]
fn bip_vector_5_long_decode() {
    let long_seed = Codex32String::from_string(
        "MS100C8VSM32ZXFGUHPCHTLUPZRY9X8GF2TVDW0S3JN54KHCE6MUA7LQPZYGSFJD6AN074RXVCEMLH8WU3TK925ACDEFGHJKLMNPQRSTUVWXY06FHPV80UNDVARHRAK".into()
    ).unwrap();
    assert_eq!(
        hex(&long_seed.parts().data()),
        "dc5423251cb87175ff8110c8531d0952d8d73e1194e95b5f19d6f9df7c01111104c9baecdfea8cccc677fb9ddc8aec5553b86e528bcadfdcc201c17c638c47e9"
    );
}

/// `from_seed` golden corpus captured ONCE from the PRE-vendor
/// `codex32 = "=0.1.0"` build. HRP="ms", threshold=0, id="test", index Fe::S.
/// Each data buffer is the deterministic pattern `b[i] = (i*7 + 3) mod 256`.
/// These literals are the proof that the vendored `from_seed` produces the same
/// wire bytes as the external crate did, for every supported entropy length.
#[test]
fn from_seed_golden_all_entropy_lengths() {
    fn pattern(len: usize) -> Vec<u8> {
        (0..len)
            .map(|i| (i as u8).wrapping_mul(7).wrapping_add(3))
            .collect()
    }

    let cases: &[(usize, &str)] = &[
        (16, "ms10testsqv9pzxqlyckngw6zf9g9whn9dstenv96hhvvflp"),
        (20, "ms10testsqv9pzxqlyckngw6zf9g9whn9d3eh4qvg4a6a9f2fsr3r2"),
        (
            24,
            "ms10testsqv9pzxqlyckngw6zf9g9whn9d3eh4qvg37tfmfqvazhe5382e4vw",
        ),
        (
            28,
            "ms10testsqv9pzxqlyckngw6zf9g9whn9d3eh4qvg37tfmf9tk2uuqru75puvle6emd",
        ),
        (
            32,
            "ms10testsqv9pzxqlyckngw6zf9g9whn9d3eh4qvg37tfmf9tk2uup37w6hwqgtmddfdndzx5f",
        ),
    ];

    for &(len, expected) in cases {
        let data = pattern(len);
        let s = Codex32String::from_seed("ms", 0, "test", Fe::S, &data).unwrap();
        assert_eq!(
            s.to_string(),
            expected,
            "from_seed parity drift at entropy length {len}"
        );
        // And the round-trip back to bytes is byte-identical.
        assert_eq!(
            s.parts().data(),
            data,
            "data round-trip drift at length {len}"
        );
    }
}
