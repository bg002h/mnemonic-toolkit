//! Phase-0 guard (R0 C1): the byte-aligned mnem payload [0x02][lang][entropy]
//! constructs through codex32 from_seed for ALL 5 entropy lengths. The earlier
//! bit-aligned 4-bit layout failed sanity_check for N=20/24/32; this pins that
//! the byte-aligned layout does not.
use ms_codec::codex32::{Codex32String, Fe};

#[test]
fn mnem_byte_aligned_constructs_for_all_five_lengths() {
    let mut lengths = vec![];
    for n in [16usize, 20, 24, 28, 32] {
        let mut data = vec![0x02u8, 0x00u8]; // mnem prefix + language English(0)
        data.extend(std::iter::repeat(0xABu8).take(n));
        let c = Codex32String::from_seed("ms", 0, "entr", Fe::S, &data)
            .unwrap_or_else(|e| panic!("N={n}: from_seed failed (sanity_check?): {e:?}"));
        let s = c.to_string();
        // round-trips byte-aligned via the public data() path
        let back = Codex32String::from_string(s.clone())
            .unwrap()
            .parts()
            .data();
        assert_eq!(back[0], 0x02, "N={n} prefix byte");
        assert_eq!(back[1], 0x00, "N={n} language byte");
        assert_eq!(&back[2..2 + n], &data[2..], "N={n} entropy");
        lengths.push(s.len());
    }
    assert_eq!(
        lengths,
        vec![51, 58, 64, 70, 77],
        "VALID_MNEM_STR_LENGTHS (byte-aligned)"
    );
}
