//! Origin-path codec — standard-table dictionary + `0xFE` explicit-path
//! escape hatch.
//!
//! Per `design/SPEC_mk_v0_1.md` §3.5 (closure Q-3: cap = 10).
//!
//! mk1-internal indicator-byte path dictionary. md1 v0.11+ encodes paths
//! explicitly via `OriginPath` and does not carry a path-dictionary
//! table; mk1's dictionary is therefore standalone, not a sibling
//! mirror. Historically (md-codec v0.10.x and earlier), md1 carried a
//! compatible table via `Tag::SharedPath` / `Tag::OriginPaths`; the
//! v0.11 architectural cleanup retired that table per
//! `descriptor-mnemonic/design/SPEC_v0_11_wire_format.md` §1.4. The
//! testnet companion `0x16` to mainnet `0x06` (BIP 48 nested-segwit
//! multisig) was added in mk-codec v0.2.0; the addition is wire-additive
//! (v0.1.x decoders reject `0x16` as `Error::InvalidPathIndicator(0x16)`,
//! v0.2+ decoders accept and resolve to `m/48'/1'/0'/1'`).
//!
//! Explicit-path encoding: indicator `0xFE`, 1-byte component count
//! (0..=10; 0 = no-path / depth-0 key, e.g. a WIF), then each component as
//! LEB128-encoded u32 with the BIP 32 hardened-bit in the high bit.

use bitcoin::bip32::{ChildNumber, DerivationPath};

use crate::consts::MAX_PATH_COMPONENTS;
use crate::error::{Error, Result};

/// Indicator byte for an explicit (non-standard-table) path.
pub const EXPLICIT_PATH_INDICATOR: u8 = 0xFE;

/// Standard-table dictionary entries — `(indicator_byte, path_string)`.
///
/// mk1-internal table — not a sibling mirror. md1 v0.11+ does not carry
/// a path-dictionary table (per
/// `descriptor-mnemonic/design/SPEC_v0_11_wire_format.md` §1.4). The 14
/// entries are: 7 mainnet (`0x01`..=`0x07`) and 7 testnet
/// (`0x11`..=`0x17`). `0x16` (BIP 48 testnet nested-segwit multisig) was
/// added in v0.2.0.
pub const STANDARD_PATHS: &[(u8, &str)] = &[
    // Mainnet
    (0x01, "m/44'/0'/0'"),    // BIP 44 mainnet
    (0x02, "m/49'/0'/0'"),    // BIP 49 mainnet
    (0x03, "m/84'/0'/0'"),    // BIP 84 mainnet
    (0x04, "m/86'/0'/0'"),    // BIP 86 mainnet
    (0x05, "m/48'/0'/0'/2'"), // BIP 48 segwit-v0 multisig mainnet
    (0x06, "m/48'/0'/0'/1'"), // BIP 48 nested-segwit multisig mainnet
    (0x07, "m/87'/0'/0'"),    // BIP 87 multisig mainnet
    // Testnet
    (0x11, "m/44'/1'/0'"),
    (0x12, "m/49'/1'/0'"),
    (0x13, "m/84'/1'/0'"),
    (0x14, "m/86'/1'/0'"),
    (0x15, "m/48'/1'/0'/2'"),
    (0x16, "m/48'/1'/0'/1'"), // v0.2.0+; was reserved-pending in v0.1.x
    (0x17, "m/87'/1'/0'"),
];

/// Look up a standard-table indicator → `DerivationPath`. Returns
/// `None` for indicators outside the dictionary (reserved values
/// `0x00`, `0x08`..=`0x10`, `0x18`..=`0xFD`, `0xFF`).
pub fn lookup_indicator(indicator: u8) -> Option<DerivationPath> {
    STANDARD_PATHS
        .iter()
        .find(|(b, _)| *b == indicator)
        .and_then(|(_, p)| p.parse().ok())
}

/// Look up `DerivationPath` → standard-table indicator. Returns `None`
/// if the path is not in the dictionary (encoder falls through to
/// explicit-path encoding). Comparison is structural (parses each
/// table entry to a `DerivationPath`); this avoids the `m/`-prefix
/// pitfall in `bitcoin::bip32::DerivationPath`'s Display.
pub fn lookup_path(path: &DerivationPath) -> Option<u8> {
    STANDARD_PATHS
        .iter()
        .find(|(_, p)| {
            p.parse::<DerivationPath>()
                .map(|table_path| &table_path == path)
                .unwrap_or(false)
        })
        .map(|(b, _)| *b)
}

/// Encode a path: 1-byte standard-table indicator if available, else
/// explicit-path escape hatch (`0xFE` + count + LEB128 components).
pub fn encode_path(path: &DerivationPath) -> Vec<u8> {
    if let Some(indicator) = lookup_path(path) {
        return vec![indicator];
    }
    let mut out = Vec::with_capacity(2 + 5 * MAX_PATH_COMPONENTS as usize);
    out.push(EXPLICIT_PATH_INDICATOR);
    let components: Vec<ChildNumber> = path.into_iter().copied().collect();
    out.push(components.len() as u8);
    for cn in components {
        let raw: u32 = u32::from(cn);
        leb128_encode(raw, &mut out);
    }
    out
}

/// Decode a path field starting at `*cursor` (advances the cursor).
pub fn decode_path(cursor: &mut &[u8]) -> Result<DerivationPath> {
    let indicator = read_u8(cursor)?;
    if indicator == EXPLICIT_PATH_INDICATOR {
        return decode_explicit_path(cursor);
    }
    if let Some(path) = lookup_indicator(indicator) {
        return Ok(path);
    }
    Err(Error::InvalidPathIndicator(indicator))
}

fn decode_explicit_path(cursor: &mut &[u8]) -> Result<DerivationPath> {
    let count = read_u8(cursor)?;
    if count > MAX_PATH_COMPONENTS {
        return Err(Error::PathTooDeep(count));
    }
    // count == 0 → no-path / depth-0 root key (e.g. a WIF). The component loop
    // below runs zero times → DerivationPath::from(vec![]) = empty path "m".
    let mut components: Vec<ChildNumber> = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let raw = leb128_decode_u32(cursor)?;
        let cn = if raw & 0x8000_0000 != 0 {
            ChildNumber::from_hardened_idx(raw & 0x7FFF_FFFF)
                .map_err(|e| Error::InvalidPathComponent(format!("{e}")))?
        } else {
            ChildNumber::from_normal_idx(raw)
                .map_err(|e| Error::InvalidPathComponent(format!("{e}")))?
        };
        components.push(cn);
    }
    Ok(DerivationPath::from(components))
}

fn leb128_encode(mut value: u32, out: &mut Vec<u8>) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
            out.push(byte);
        } else {
            out.push(byte);
            break;
        }
    }
}

fn leb128_decode_u32(cursor: &mut &[u8]) -> Result<u32> {
    let mut result: u64 = 0;
    let mut shift: u32 = 0;
    loop {
        let byte = read_u8(cursor)?;
        result |= ((byte & 0x7F) as u64) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
        // u32 max needs ⌈32/7⌉ = 5 bytes; bail at the 6th byte (shift=35).
        if shift >= 35 {
            return Err(Error::InvalidPathComponent(format!(
                "LEB128 overflow at shift {shift}"
            )));
        }
    }
    if result > u32::MAX as u64 {
        return Err(Error::InvalidPathComponent(format!(
            "LEB128 value {result} > u32::MAX"
        )));
    }
    Ok(result as u32)
}

fn read_u8(cursor: &mut &[u8]) -> Result<u8> {
    if cursor.is_empty() {
        return Err(Error::UnexpectedEnd);
    }
    let b = cursor[0];
    *cursor = &cursor[1..];
    Ok(b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn round_trip_all_standard_paths() {
        for (indicator, path_str) in STANDARD_PATHS {
            let path = DerivationPath::from_str(path_str).unwrap();
            let encoded = encode_path(&path);
            assert_eq!(encoded, vec![*indicator], "round-trip {path_str}");
            let mut cursor: &[u8] = &encoded;
            let decoded = decode_path(&mut cursor).unwrap();
            assert_eq!(decoded, path, "round-trip parsed {path_str}");
            assert!(cursor.is_empty());
        }
    }

    #[test]
    fn round_trip_explicit_path_simple() {
        let path = DerivationPath::from_str("m/0/1/2").unwrap();
        let encoded = encode_path(&path);
        // 0xFE + count(3) + leb128(0,1,2) = each fits in 1 byte
        assert_eq!(encoded[0], 0xFE);
        assert_eq!(encoded[1], 3);
        let mut cursor: &[u8] = &encoded;
        let decoded = decode_path(&mut cursor).unwrap();
        assert_eq!(decoded, path);
    }

    #[test]
    fn round_trip_explicit_path_all_hardened() {
        // m/9999'/1234'/56'/7' — hardened, requires 5 LEB128 bytes per component
        let path = DerivationPath::from_str("m/9999'/1234'/56'/7'").unwrap();
        let encoded = encode_path(&path);
        assert_eq!(encoded[0], 0xFE);
        assert_eq!(encoded[1], 4);
        // 0xFE + 1 (count) + 4 * 5 = 22 bytes
        assert_eq!(encoded.len(), 1 + 1 + 4 * 5);
        let mut cursor: &[u8] = &encoded;
        let decoded = decode_path(&mut cursor).unwrap();
        assert_eq!(decoded, path);
    }

    #[test]
    fn round_trip_explicit_path_at_cap() {
        // 10 components — cap exact
        let path = DerivationPath::from_str("m/0'/1'/2'/3'/4'/5'/6'/7'/8'/9'").unwrap();
        let encoded = encode_path(&path);
        let mut cursor: &[u8] = &encoded;
        let decoded = decode_path(&mut cursor).unwrap();
        assert_eq!(decoded, path);
    }

    #[test]
    fn rejects_path_too_deep() {
        // Construct an explicit-path encoding with count = 11
        let mut bytes = vec![0xFE, 11u8];
        for i in 0..11 {
            bytes.push(i); // single-byte LEB128
        }
        let mut cursor: &[u8] = &bytes;
        assert!(matches!(
            decode_path(&mut cursor),
            Err(Error::PathTooDeep(11)),
        ));
    }

    #[test]
    fn accepts_path_count_zero_as_empty_path() {
        // count = 0 is the no-path / depth-0 case (e.g. a WIF). v0.4.0+: decode
        // returns the empty path; older decoders rejected it as PathTooDeep(0).
        let bytes = vec![0xFE, 0u8];
        let mut cursor: &[u8] = &bytes;
        let decoded = decode_path(&mut cursor).unwrap();
        assert_eq!(decoded.into_iter().count(), 0, "empty path");
        assert!(cursor.is_empty());
    }

    #[test]
    fn round_trip_empty_path() {
        let path = DerivationPath::from_str("m").unwrap(); // empty
        let encoded = encode_path(&path);
        assert_eq!(encoded, vec![0xFE, 0x00]);
        let mut cursor: &[u8] = &encoded;
        let decoded = decode_path(&mut cursor).unwrap();
        assert_eq!(decoded, path);
        assert!(cursor.is_empty());
    }

    #[test]
    fn rejects_reserved_indicator_zero() {
        let bytes = vec![0x00];
        let mut cursor: &[u8] = &bytes;
        assert!(matches!(
            decode_path(&mut cursor),
            Err(Error::InvalidPathIndicator(0x00)),
        ));
    }

    #[test]
    fn round_trip_indicator_0x16_added_in_v0_2() {
        // 0x16 was reserved-pending in v0.1.x; added to STANDARD_PATHS
        // in v0.2.0. Resolves to BIP 48 testnet nested-segwit multisig
        // (`m/48'/1'/0'/1'`). Historical context: this entry tracked an
        // md1-side gap at the time the mk-codec v0.2.0 cycle ran;
        // md1 v0.11+ has since dropped path dictionaries entirely (the
        // mirror invariant is retired — see this module's rustdoc).
        let path = DerivationPath::from_str("m/48'/1'/0'/1'").unwrap();
        let encoded = encode_path(&path);
        assert_eq!(encoded, vec![0x16]);
        let mut cursor: &[u8] = &encoded;
        let decoded = decode_path(&mut cursor).unwrap();
        assert_eq!(decoded, path);
        assert!(cursor.is_empty());
    }

    #[test]
    fn rejects_reserved_indicator_high_range() {
        // 0xFD (just below 0xFE explicit) is reserved
        let bytes = vec![0xFD];
        let mut cursor: &[u8] = &bytes;
        assert!(matches!(
            decode_path(&mut cursor),
            Err(Error::InvalidPathIndicator(0xFD)),
        ));
        // 0xFF is reserved
        let bytes = vec![0xFF];
        let mut cursor: &[u8] = &bytes;
        assert!(matches!(
            decode_path(&mut cursor),
            Err(Error::InvalidPathIndicator(0xFF)),
        ));
    }

    #[test]
    fn rejects_truncated_explicit_path() {
        // 0xFE indicator + count(2) + only one component byte
        let bytes = vec![0xFE, 2u8, 0u8];
        let mut cursor: &[u8] = &bytes;
        assert!(matches!(
            decode_path(&mut cursor),
            Err(Error::UnexpectedEnd),
        ));
    }

    #[test]
    fn leb128_encode_examples() {
        // 0 → [0]
        let mut out = Vec::new();
        leb128_encode(0, &mut out);
        assert_eq!(out, vec![0]);
        // 127 → [0x7F]
        let mut out = Vec::new();
        leb128_encode(127, &mut out);
        assert_eq!(out, vec![0x7F]);
        // 128 → [0x80, 0x01]
        let mut out = Vec::new();
        leb128_encode(128, &mut out);
        assert_eq!(out, vec![0x80, 0x01]);
        // 0x80000000 (hardened bit set) → [0x80, 0x80, 0x80, 0x80, 0x08]
        let mut out = Vec::new();
        leb128_encode(0x8000_0000, &mut out);
        assert_eq!(out, vec![0x80, 0x80, 0x80, 0x80, 0x08]);
    }
}
