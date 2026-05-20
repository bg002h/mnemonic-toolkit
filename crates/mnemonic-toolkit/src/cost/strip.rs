//! Descriptor wrapper-stripping. SPEC §2 + §11 (v0.28.0 single-leaf-tr).
//!
//! Supports `wsh(M)`, `sh(wsh(M))`, and single-leaf `tr(IK, M)`. Multi-leaf
//! `tr(IK, {M1, M2, ...})` is rejected. Closes FOLLOWUP
//! `compare-cost-single-leaf-tr-input` (filed in v0.27.0 cycle close).

use std::str::FromStr;

use miniscript::descriptor::{DefiniteDescriptorKey, DescriptorPublicKey, ShInner};
use miniscript::{Descriptor, Miniscript, Segwitv0, Tap};

use super::translate::Translated;
use super::CompareCostError;
use super::NUMS_XONLY_HEX;

/// Strip the wrapper from a user-supplied descriptor string; return a
/// `Translated` ready for the enumerator (plus any advisory note). Concrete
/// hex pubkeys are preserved; no abstract labels are detected (those only
/// appear in `--miniscript` input).
pub fn translate_descriptor(input: &str) -> Result<(Translated, Option<String>), CompareCostError> {
    let desc = Descriptor::<DescriptorPublicKey>::from_str(input)
        .map_err(|e| CompareCostError::Parse(format!("descriptor parse: {e}")))?;

    // Materialize wildcards to a concrete derivation index (cost is identical
    // across child indices for a given descriptor shape; index 0 is fine).
    let definite: Descriptor<DefiniteDescriptorKey> = if desc.has_wildcard() {
        desc.derive_at_index(0)
            .map_err(|e| CompareCostError::Parse(format!("derivation: {e}")))?
    } else {
        // No wildcards: TryFrom converts cleanly without "index" indirection.
        Descriptor::<DefiniteDescriptorKey>::try_from(desc)
            .map_err(|e| CompareCostError::Parse(format!("definite-key conversion: {e}")))?
    };

    match definite {
        Descriptor::Wsh(wsh) => {
            let m_segv0 = wsh.into_inner();
            translated_from_segv0(input, m_segv0).map(|t| (t, None))
        }
        Descriptor::Sh(sh) => match sh.into_inner() {
            ShInner::Wsh(wsh) => {
                let m_segv0 = wsh.into_inner();
                translated_from_segv0(input, m_segv0).map(|t| (t, None))
            }
            ShInner::Wpkh(_) | ShInner::Ms(_) => Err(
                CompareCostError::UnsupportedWrapper("sh(non-wsh)".to_string()),
            ),
        },
        Descriptor::Tr(tr) => translate_descriptor_tr_single_leaf(input, tr),
        Descriptor::Bare(_) | Descriptor::Pkh(_) | Descriptor::Wpkh(_) => Err(
            CompareCostError::UnsupportedWrapper("pkh / wpkh / bare not a miniscript-wrapping question".to_string()),
        ),
    }
}

/// Build a Translated from an already-parsed Segwitv0 miniscript by
/// re-serializing the keys for Tap context (compressed-secp 33B → x-only 32B
/// via the standard "drop the 02/03 parity byte" projection).
fn translated_from_segv0(
    _original_input: &str,
    m_segv0: Miniscript<DefiniteDescriptorKey, Segwitv0>,
) -> Result<Translated, CompareCostError> {
    let segv0_str = m_segv0.to_string();
    let tap_str = segv0_string_to_tap_string(&segv0_str);
    let m_tap: Miniscript<DefiniteDescriptorKey, Tap> = Miniscript::from_str(&tap_str)
        .map_err(|e| CompareCostError::ContextIncompat {
            valid_in: "Segwitv0",
            invalid_in: "Tap",
            detail: format!("{e}"),
        })?;

    // SPEC §5: `extracted_miniscript` is the inner M (post-wrapper-strip),
    // not the original full descriptor string. Use the segv0-context
    // canonical serialization.
    Ok(Translated {
        extracted: segv0_str,
        segv0: m_segv0,
        tap: m_tap,
        concrete_keys: true,
        labels: Vec::new(),
        label_pubkeys: Vec::new(),
        tr_non_nums_internal_key_xonly_hex: None,
    })
}

/// SPEC §11 (v0.28.0) — strip a single-leaf `tr(IK, {M})` descriptor:
/// extract the internal key + Tap-context miniscript M, reverse-project
/// M into Segwitv0 context for the wsh comparison, and record the IK so
/// the dispatcher can surface keypath-spend cost.
///
/// Reverse projection: x-only Tap key `<32B>` → compressed Segwitv0 key
/// `02<32B>` (BIP-340 lift-x even-y LOCK per SPEC §11). Cost is
/// parity-invariant — the choice of `02` over `03` does not affect any
/// vbyte count, only the locked-in convention.
///
/// Multi-leaf TapTree is refused with `MultiLeafTr`; the user must supply
/// one leaf at a time via `--miniscript`.
fn translate_descriptor_tr_single_leaf(
    _original_input: &str,
    tr: miniscript::descriptor::Tr<DefiniteDescriptorKey>,
) -> Result<(Translated, Option<String>), CompareCostError> {
    // Walk the tap_tree leaves; reject multi-leaf. `li.miniscript()` returns
    // `&Arc<Miniscript<_, Tap>>` — deref-clone into an owned value so the
    // borrow on `tr` is dropped before the IK read below.
    let leaves: Vec<Miniscript<DefiniteDescriptorKey, Tap>> = match tr.tap_tree() {
        None => {
            return Err(CompareCostError::UnsupportedWrapper(
                "tr(IK) keypath-only descriptor has no script; supply --miniscript with a script body".to_string(),
            ));
        }
        Some(tt) => tt.leaves().map(|li| (**li.miniscript()).clone()).collect(),
    };
    if leaves.len() != 1 {
        return Err(CompareCostError::MultiLeafTr);
    }
    let m_tap: Miniscript<DefiniteDescriptorKey, Tap> =
        leaves.into_iter().next().expect("len==1 just checked");

    // Reverse-project Tap → Segwitv0 via the existing tap-string rewriter
    // (multi_a → multi + x-only-32B → compressed-33B with 02 prefix).
    let tap_str = m_tap.to_string();
    let segv0_str = tap_string_to_segv0_string(&tap_str);
    let m_segv0: Miniscript<DefiniteDescriptorKey, Segwitv0> = Miniscript::from_str(&segv0_str)
        .map_err(|e| CompareCostError::ContextIncompat {
            valid_in: "Tap",
            invalid_in: "Segwitv0",
            detail: format!("{e}"),
        })?;

    // Record internal-key for non-NUMS detection by the dispatcher.
    let internal_key_hex = tr.internal_key().to_string();
    let tr_non_nums_internal_key_xonly_hex = if internal_key_hex == NUMS_XONLY_HEX {
        None
    } else {
        Some(internal_key_hex)
    };

    Ok((
        Translated {
            // SPEC §5: extracted_miniscript is the inner M.
            extracted: segv0_str,
            segv0: m_segv0,
            tap: m_tap,
            concrete_keys: true,
            labels: Vec::new(),
            label_pubkeys: Vec::new(),
            tr_non_nums_internal_key_xonly_hex,
        },
        None,
    ))
}

/// Inverse of [`segv0_string_to_tap_string`] (within parity convention):
/// rewrite `multi_a`/`sortedmulti_a` → `multi`/`sortedmulti`, and
/// re-inflate x-only 64-char hex pubkeys to compressed 66-char hex by
/// prepending `02` (BIP-340 lift-x even-y LOCK per SPEC §11).
///
/// Cost is parity-invariant; the `02` prefix is a fixed convention so
/// the parsed Segwitv0 miniscript is deterministic.
fn tap_string_to_segv0_string(tap: &str) -> String {
    // Run multi-rewrite first so the subsequent key-scan does not
    // confuse `multi_a` digits with anything else.
    let after_multi = {
        let s = replace_fragment(tap, "sortedmulti_a(", "sortedmulti(");
        replace_fragment(&s, "multi_a(", "multi(")
    };
    inflate_xonly_to_compressed_even_y(&after_multi)
}

/// Scan for 64-char x-only hex runs (NOT inside a longer identifier or
/// hex run) and prepend `02` to produce a 66-char compressed hex. The
/// scan boundary discipline mirrors [`segv0_string_to_tap_string`]'s
/// 66-char scan: a candidate must be flanked by non-identifier characters
/// on both sides.
fn inflate_xonly_to_compressed_even_y(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + input.len() / 32);
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 64 <= bytes.len() {
            let candidate = &bytes[i..i + 64];
            let all_hex = candidate.iter().all(|b| b.is_ascii_hexdigit());
            let prev_is_ident = i > 0 && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_');
            let next_is_ident = i + 64 < bytes.len()
                && (bytes[i + 64].is_ascii_alphanumeric() || bytes[i + 64] == b'_');
            if all_hex && !prev_is_ident && !next_is_ident {
                // SPEC §11 LOCK: lift-x with even-y → prepend `02`.
                out.push('0');
                out.push('2');
                out.push_str(std::str::from_utf8(candidate).unwrap());
                i += 64;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

/// Convert a Segwitv0-context miniscript string (compressed-secp 66-char hex
/// pubkeys, `multi`/`sortedmulti`) into a Tap-context miniscript string
/// (x-only 64-char hex, `multi_a`/`sortedmulti_a`).
///
/// Compressed pubkey → x-only: drop the leading 2-hex-char parity byte
/// (`02ab…cd` → `ab…cd`). This is the standard BIP-340 projection (x-only is
/// just the x-coordinate of the secp point).
fn segv0_string_to_tap_string(segv0: &str) -> String {
    let mut out = String::with_capacity(segv0.len());
    let bytes = segv0.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        // Detect a 66-char compressed-pubkey hex run starting with 02/03.
        if i + 66 <= bytes.len() {
            let candidate = &bytes[i..i + 66];
            let starts_with_parity = matches!(candidate[0..2], [b'0', b'2'] | [b'0', b'3']);
            let all_hex = candidate.iter().all(|b| b.is_ascii_hexdigit());
            let prev_is_ident = i > 0 && (bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_');
            let next_is_ident = i + 66 < bytes.len()
                && (bytes[i + 66].is_ascii_alphanumeric() || bytes[i + 66] == b'_');
            if starts_with_parity && all_hex && !prev_is_ident && !next_is_ident {
                // Append the x-only form (drop first 2 hex chars).
                out.push_str(std::str::from_utf8(&candidate[2..]).unwrap());
                i += 66;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    // Now rewrite multi/sortedmulti → multi_a/sortedmulti_a.
    let out = replace_fragment(&out, "sortedmulti(", "sortedmulti_a(");
    replace_fragment(&out, "multi(", "multi_a(")
}

fn replace_fragment(haystack: &str, find: &str, repl: &str) -> String {
    let mut result = String::with_capacity(haystack.len());
    let bytes = haystack.as_bytes();
    let find_b = find.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + find_b.len() <= bytes.len() && &bytes[i..i + find_b.len()] == find_b {
            let before_ok = i == 0
                || !(bytes[i - 1].is_ascii_alphanumeric() || bytes[i - 1] == b'_');
            if before_ok {
                result.push_str(repl);
                i += find_b.len();
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_parity_byte_on_compressed_pubkey() {
        let segv0 = "pk(0288facd06da4a3b8b6c7e0a8f6c8c6e0a8f6c8c6e0a8f6c8c6e0a8f6c8c6e0a8f)";
        let tap = segv0_string_to_tap_string(segv0);
        assert_eq!(
            tap,
            "pk(88facd06da4a3b8b6c7e0a8f6c8c6e0a8f6c8c6e0a8f6c8c6e0a8f6c8c6e0a8f)"
        );
    }

    #[test]
    fn rewrites_multi_to_multi_a() {
        let input = "multi(2,02aa00aa00aa00aa00aa00aa00aa00aa00aa00aa00aa00aa00aa00aa00aa00aaaa,03bb00bb00bb00bb00bb00bb00bb00bb00bb00bb00bb00bb00bb00bb00bb00bbbb)";
        let out = segv0_string_to_tap_string(input);
        assert!(out.starts_with("multi_a("), "expected multi_a prefix: {out}");
    }

    // ── R1-I4 (a) lift-x parity-prefix assertion (SPEC §11 LOCK) ───────────
    //
    // The reverse-projection x-only `<32B>` → compressed `02<32B>` MUST
    // prepend exactly the byte 0x02 (BIP-340 lift-x even-y LOCK). This
    // cell pins the convention so future refactors of
    // `inflate_xonly_to_compressed_even_y` cannot silently swap to 0x03.
    #[test]
    fn lift_x_prefix_is_exactly_02() {
        let xonly = "ab".to_string().repeat(32); // 64 hex chars
        let input = format!("pk({xonly})");
        let out = tap_string_to_segv0_string(&input);
        let expected = format!("pk(02{xonly})");
        assert_eq!(out, expected, "lift-x LOCK: prefix MUST be exactly '02', not '03'");
        // Belt-and-suspenders: assert the inflated key starts with '02'
        // and has length 66.
        let body_start = "pk(".len();
        let body_end = out.len() - ")".len();
        let pk_hex = &out[body_start..body_end];
        assert_eq!(pk_hex.len(), 66, "compressed pubkey is 33B / 66 hex chars");
        assert!(pk_hex.starts_with("02"), "prefix MUST be exactly '02'");
    }

    #[test]
    fn lift_x_inflates_multi_a_to_multi_with_02_prefix() {
        let k1 = "aa".to_string().repeat(32);
        let k2 = "bb".to_string().repeat(32);
        let input = format!("multi_a(2,{k1},{k2})");
        let out = tap_string_to_segv0_string(&input);
        let expected = format!("multi(2,02{k1},02{k2})");
        assert_eq!(out, expected);
    }
}

// R1-I4 (b) cost-domain parity-invariance smoke lives in
// `tests/cli_compare_cost.rs::cost_is_parity_invariant_02_vs_03`
// (integration test, since `CARGO_BIN_EXE_mnemonic` is only set there).
