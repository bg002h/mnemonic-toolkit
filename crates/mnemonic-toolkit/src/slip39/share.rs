//! SLIP-39 share — bit-packing, parse, render.
//!
//! Per SLIP-0039 §3.1 "Format of the share mnemonic" + reference impl
//! `python-shamir-mnemonic/shamir_mnemonic/share.py` @ commit
//! `17fcce14`. A SLIP-39 mnemonic is a sequence of 10-bit words drawn
//! from the 1024-word wordlist (`wordlist.rs`); each share encodes:
//!
//! ```text
//!   id_exp       : 2 words = 20 bits — identifier(15) | extendable(1) | iteration_exponent(4)
//!   share_params : 2 words = 20 bits — group_index(4) | (group_threshold − 1)(4)
//!                                    | (group_count − 1)(4) | member_index(4)
//!                                    | (member_threshold − 1)(4)
//!   value        : variable — share value bytes, LEFT-padded with
//!                  0..=8 zero bits so the padded length is a multiple
//!                  of 10
//!   checksum     : 3 words = 30 bits — RS1024 BCH over
//!                  `cs || (id_exp .. value)`
//! ```
//!
//! Thresholds are stored as `T − 1` on the wire (4-bit field 0..=15 ↔
//! threshold 1..=16); indices are stored as-is (already in 0..=15).
//! Bit order is big-endian; the MSB of the first 10-bit word holds the
//! MSB of the bit stream.
//!
//! The customization string fed to RS1024 derives from the `ext` bit:
//!   ext = 0 ⇒ cs = b"shamir"
//!   ext = 1 ⇒ cs = b"shamir_extendable"
//!
//! Parse-error ordering (matches the reference impl):
//!   1. unknown word ⇒ [`Slip39Error::UnknownWord`]
//!   2. word-count gate (`< MIN_MNEMONIC_LENGTH_WORDS`) ⇒
//!      [`Slip39Error::InvalidPadding`]
//!   3. pre-checksum padding gate (`padding_bits > 8`) ⇒
//!      [`Slip39Error::InvalidPadding`]
//!   4. RS1024 checksum ⇒ [`Slip39Error::InvalidChecksum`]
//!   5. non-zero leading padding bits in the value field ⇒
//!      [`Slip39Error::InvalidPadding`]
//!
//! [`parse_slip39_share`] is a SINGLE-share parser; the `share_idx`
//! carried by the three position-bearing variants is always `0`.
//! [`crate::slip39::slip39_combine`] (P1c-E) remaps `share_idx` to the
//! position within its input vector.

use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::slip39::error::Slip39Error;
use crate::slip39::rs1024;
use crate::slip39::wordlist;

/// Words before the value field: 2 id_exp + 2 share_params.
const PREFIX_WORDS: usize = 4;

/// Words after the value field: 3 RS1024 checksum.
const CHECKSUM_WORDS: usize = 3;

/// Total non-value words.
const METADATA_LENGTH_WORDS: usize = PREFIX_WORDS + CHECKSUM_WORDS;

/// Minimum mnemonic word count: 7 metadata + 13 value words (the
/// smallest valid SLIP-39 value field is 13 words = 130 bits, holding a
/// 16-byte master-secret share with 2 padding bits).
const MIN_MNEMONIC_LENGTH_WORDS: usize = 20;

/// Customization string for RS1024 on non-extendable shares.
const CS_NON_EXTENDABLE: &[u8] = b"shamir";

/// Customization string for RS1024 on extendable shares.
const CS_EXTENDABLE: &[u8] = b"shamir_extendable";

/// A SLIP-39 share — metadata + share-value bytes.
///
/// `value` is private and zeroized on drop; the public metadata fields
/// are skipped (non-secret; on the wire in the encoded mnemonic).
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct Share {
    /// Share-value bytes — secret-bearing. For a non-trivial Shamir
    /// split, this is one share of the EMS (encrypted master secret).
    #[zeroize]
    value: Vec<u8>,

    /// 15-bit identifier shared across all shares of one master secret.
    #[zeroize(skip)]
    pub identifier: u16,

    /// SLIP-0039 `ext` bit: false ⇒ non-extendable (identifier used as
    /// Feistel salt); true ⇒ extendable (identifier NOT in salt).
    #[zeroize(skip)]
    pub extendable: bool,

    /// 4-bit PBKDF2 cost exponent (0..=15); iterations = 10000 · 2^E.
    #[zeroize(skip)]
    pub iteration_exponent: u8,

    /// 4-bit group index (0..=15) — group's Shamir x-coordinate.
    #[zeroize(skip)]
    pub group_index: u8,

    /// Group threshold (1..=16) — groups required to combine.
    /// Stored on the wire as `group_threshold − 1`.
    #[zeroize(skip)]
    pub group_threshold: u8,

    /// Group count (1..=16) — total groups.
    /// Stored on the wire as `group_count − 1`.
    #[zeroize(skip)]
    pub group_count: u8,

    /// 4-bit member index (0..=15) — member's Shamir x-coordinate
    /// within its group.
    #[zeroize(skip)]
    pub member_index: u8,

    /// Member threshold (1..=16) — members required within this
    /// share's group. Stored on the wire as `member_threshold − 1`.
    #[zeroize(skip)]
    pub member_threshold: u8,
}

impl Share {
    /// Construct a `Share` from explicit field values. Crate-internal;
    /// external callers reach `Share` via [`parse_slip39_share`] or
    /// [`crate::slip39::slip39_split`] (P1c-E).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_parts(
        value: Vec<u8>,
        identifier: u16,
        extendable: bool,
        iteration_exponent: u8,
        group_index: u8,
        group_threshold: u8,
        group_count: u8,
        member_index: u8,
        member_threshold: u8,
    ) -> Self {
        Self {
            value,
            identifier,
            extendable,
            iteration_exponent,
            group_index,
            group_threshold,
            group_count,
            member_index,
            member_threshold,
        }
    }

}

/// Manually-implemented Debug that REDACTS the share-value bytes —
/// `value` is secret-bearing per SPEC §2.1 and must never leak via
/// log/print paths. Length is non-secret (derivable from the mnemonic
/// word count anyway) so it's shown as a length hint.
impl std::fmt::Debug for Share {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Share")
            .field("identifier", &self.identifier)
            .field("extendable", &self.extendable)
            .field("iteration_exponent", &self.iteration_exponent)
            .field("group_index", &self.group_index)
            .field("group_threshold", &self.group_threshold)
            .field("group_count", &self.group_count)
            .field("member_index", &self.member_index)
            .field("member_threshold", &self.member_threshold)
            .field(
                "value",
                &format_args!("<{} bytes redacted>", self.value.len()),
            )
            .finish()
    }
}

/// Customization string for RS1024 based on the `ext` bit.
fn cs_for(extendable: bool) -> &'static [u8] {
    if extendable {
        CS_EXTENDABLE
    } else {
        CS_NON_EXTENDABLE
    }
}

/// Parse a single SLIP-39 share mnemonic into a [`Share`].
///
/// Whitespace-tolerant: any run of ASCII whitespace separates words.
///
/// Errors carry `share_idx: 0`; the combine driver remaps this to
/// position-within-input.
pub fn parse_slip39_share(s: &str) -> Result<Share, Slip39Error> {
    // 1. Word validity.
    let words: Vec<&str> = s.split_whitespace().collect();
    let mut indices: Vec<u16> = Vec::with_capacity(words.len());
    for (word_idx, w) in words.iter().enumerate() {
        match wordlist::word_to_index(w) {
            Some(idx) => indices.push(idx),
            None => {
                return Err(Slip39Error::UnknownWord {
                    share_idx: 0,
                    word_idx,
                })
            }
        }
    }

    // 2. Word-count gate.
    if indices.len() < MIN_MNEMONIC_LENGTH_WORDS {
        return Err(Slip39Error::InvalidPadding { share_idx: 0 });
    }

    // 3. Pre-checksum padding gate.
    let value_data_word_count = indices.len() - METADATA_LENGTH_WORDS;
    let total_value_bits = value_data_word_count * 10;
    let padding_bits = total_value_bits % 16;
    if padding_bits > 8 {
        return Err(Slip39Error::InvalidPadding { share_idx: 0 });
    }
    let value_byte_count = (total_value_bits - padding_bits) / 8;

    // Decode id_exp (used to derive cs before the checksum step).
    let id_exp_int = (u32::from(indices[0]) << 10) | u32::from(indices[1]);
    let identifier = (id_exp_int >> 5) as u16;
    let extendable = ((id_exp_int >> 4) & 1) != 0;
    let iteration_exponent = (id_exp_int & 0xF) as u8;

    // 4. RS1024 checksum.
    let cs = cs_for(extendable);
    if !rs1024::verify_checksum(cs, &indices) {
        return Err(Slip39Error::InvalidChecksum { share_idx: 0 });
    }

    // Decode share_params (2 words).
    let share_params_int =
        (u32::from(indices[2]) << 10) | u32::from(indices[3]);
    let member_threshold = ((share_params_int & 0xF) as u8) + 1;
    let member_index = ((share_params_int >> 4) & 0xF) as u8;
    let group_count = (((share_params_int >> 8) & 0xF) as u8) + 1;
    let group_threshold = (((share_params_int >> 12) & 0xF) as u8) + 1;
    let group_index = ((share_params_int >> 16) & 0xF) as u8;

    // 5. Decode value bytes — fails if the leading padding bits are
    //    not all zero.
    let value_words = &indices[PREFIX_WORDS..indices.len() - CHECKSUM_WORDS];
    let value = decode_value(value_words, padding_bits, value_byte_count)
        .ok_or(Slip39Error::InvalidPadding { share_idx: 0 })?;

    Ok(Share::from_parts(
        value,
        identifier,
        extendable,
        iteration_exponent,
        group_index,
        group_threshold,
        group_count,
        member_index,
        member_threshold,
    ))
}

/// Render a [`Share`] as a SLIP-39 mnemonic string (single-space-joined).
///
/// Inverse of [`parse_slip39_share`]: `parse(render(s))` is `s` for any
/// `s` constructed via the public surface.
pub fn render_slip39_share(s: &Share) -> String {
    // Encode id_exp into 2 × 10-bit words.
    let id_exp_int = (u32::from(s.identifier) << 5)
        | (u32::from(s.extendable) << 4)
        | (u32::from(s.iteration_exponent) & 0xF);
    let id_exp_words = [
        ((id_exp_int >> 10) & 0x3FF) as u16,
        (id_exp_int & 0x3FF) as u16,
    ];

    // Encode share_params into 2 × 10-bit words. Per SPEC §2.1
    // invariant the thresholds are 1..=16, so the subtractions cannot
    // underflow.
    let share_params_int = (u32::from(s.group_index) << 16)
        | (u32::from(s.group_threshold - 1) << 12)
        | (u32::from(s.group_count - 1) << 8)
        | (u32::from(s.member_index) << 4)
        | u32::from(s.member_threshold - 1);
    let share_params_words = [
        ((share_params_int >> 10) & 0x3FF) as u16,
        (share_params_int & 0x3FF) as u16,
    ];

    // Encode value bytes. `value_word_count` is the smallest count
    // such that 10 · value_word_count ≥ 8 · value.len(); the resulting
    // padding (≤ 8 bits) is prepended as zeros.
    let value_bits = s.value.len() * 8;
    let value_word_count = value_bits.div_ceil(10);
    let padding_bits = value_word_count * 10 - value_bits;
    let value_words = encode_value(&s.value, padding_bits, value_word_count);

    // Concatenate id_exp || share_params || value.
    let mut data: Vec<u16> =
        Vec::with_capacity(PREFIX_WORDS + value_word_count + CHECKSUM_WORDS);
    data.extend_from_slice(&id_exp_words);
    data.extend_from_slice(&share_params_words);
    data.extend_from_slice(&value_words);

    // Append RS1024 checksum.
    let cs = cs_for(s.extendable);
    let checksum = rs1024::create_checksum(cs, &data);
    data.extend_from_slice(&checksum);

    // Map each 10-bit index to its wordlist word, join with one space.
    data.iter()
        .map(|&i| {
            wordlist::index_to_word(i)
                .expect("rs1024 + bit-pack invariants keep all words in 0..1024")
        })
        .collect::<Vec<&str>>()
        .join(" ")
}

/// Decode `value_words` (10-bit big-endian, left-padded with
/// `padding_bits` zeros) into the `value_byte_count` original bytes.
///
/// Returns `None` if any of the leading `padding_bits` bits is non-zero
/// (caller surfaces this as [`Slip39Error::InvalidPadding`]).
fn decode_value(
    value_words: &[u16],
    padding_bits: usize,
    value_byte_count: usize,
) -> Option<Vec<u8>> {
    debug_assert_eq!(
        value_words.len() * 10,
        padding_bits + value_byte_count * 8
    );

    let get_bit = |i: usize| -> u8 {
        let word = value_words[i / 10] & 0x3FF;
        let bit_in_word = i % 10;
        // The MSB of a 10-bit word is bit 9 (big-endian).
        ((word >> (9 - bit_in_word)) & 1) as u8
    };

    // Padding-bit check: all leading bits must be zero.
    for i in 0..padding_bits {
        if get_bit(i) != 0 {
            return None;
        }
    }

    // Pack the remaining bits into bytes MSB-first.
    let mut bytes = vec![0u8; value_byte_count];
    for (byte_idx, byte) in bytes.iter_mut().enumerate() {
        let mut b = 0u8;
        for j in 0..8 {
            b = (b << 1) | get_bit(padding_bits + byte_idx * 8 + j);
        }
        *byte = b;
    }
    Some(bytes)
}

/// Encode `value` bytes into 10-bit big-endian words, left-padding the
/// bit stream with `padding_bits` zeros.
fn encode_value(value: &[u8], padding_bits: usize, word_count: usize) -> Vec<u16> {
    debug_assert_eq!(word_count * 10, padding_bits + value.len() * 8);

    let get_bit = |i: usize| -> u8 {
        if i < padding_bits {
            0
        } else {
            let value_bit = i - padding_bits;
            (value[value_bit / 8] >> (7 - value_bit % 8)) & 1
        }
    };

    let mut words = Vec::with_capacity(word_count);
    for word_idx in 0..word_count {
        let mut w = 0u16;
        for j in 0..10 {
            w = (w << 1) | u16::from(get_bit(word_idx * 10 + j));
        }
        words.push(w);
    }
    words
}
