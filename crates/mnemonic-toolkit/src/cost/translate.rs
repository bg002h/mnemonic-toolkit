//! Substitute user-supplied abstract labels (`A`, `B`, …) in a miniscript
//! string with deterministic `DefiniteDescriptorKey` hex; parse the result in
//! both Segwitv0 and Tap script contexts. SPEC §2.1 + §2.2.
//!
//! Whitespace inside the substituted miniscript is stripped before parsing —
//! miniscript's parser is strict and rejects intra-argument whitespace, but
//! users naturally write `multi(2, A, B, C)` with spaces.

use std::str::FromStr;
use std::sync::Arc;

use miniscript::descriptor::{DefiniteDescriptorKey, DescriptorPublicKey, TapTree, Tr, Wsh};
use miniscript::{Descriptor, Miniscript, Segwitv0, Tap};

use super::dummy_keys::{dummy_compressed, dummy_xonly, nums_xonly_definite};
use super::CompareCostError;

/// Translation product — parsed in both contexts, ready to build descriptors.
pub struct Translated {
    /// Echoed back to the user in §5 header; same string they passed in.
    pub extracted: String,
    pub segv0: Miniscript<DefiniteDescriptorKey, Segwitv0>,
    pub tap: Miniscript<DefiniteDescriptorKey, Tap>,
    /// `true` if the input had at least one concrete hex pubkey; `false` if
    /// all keys were abstract labels.
    pub concrete_keys: bool,
    /// Ordered list of user-supplied abstract labels in their AST-left-to-
    /// right order — used to label rows in the output table (SPEC §3.5).
    /// Empty when input had only concrete hex keys.
    pub labels: Vec<String>,
    /// (segwit-compressed pubkey hex, x-only-tap pubkey hex) per label, same
    /// index as `labels`. Used to map post-substitution pubkey strings back
    /// to user labels.
    pub label_pubkeys: Vec<(String, String)>,
    /// SPEC §11 (v0.28.0) — single-leaf `tr(IK, {M})` input only: x-only hex
    /// of the user-supplied internal key (`None` for `--miniscript`,
    /// `--descriptor wsh(...)`, `--descriptor sh(wsh(...))`, or when the IK
    /// equals BIP-341 NUMS). When `Some`, [`run_compare_cost`] surfaces a
    /// keypath-spend-cost column / advisory.
    ///
    /// [`run_compare_cost`]: super::run_compare_cost
    pub tr_non_nums_internal_key_xonly_hex: Option<String>,
}

/// Substitute abstract labels and parse in both contexts.
pub fn translate_miniscript(input: &str) -> Result<Translated, CompareCostError> {
    let labels = collect_abstract_labels(input);
    let concrete_keys = labels.is_empty();

    let label_pubkeys: Vec<(String, String)> = labels
        .iter()
        .map(|lbl| {
            (
                dummy_compressed(lbl).to_string(),
                dummy_xonly(lbl).to_string(),
            )
        })
        .collect();

    let segv0_input = if labels.is_empty() {
        input.to_string()
    } else {
        rewrite_labels_with(&labels, input, |lbl_idx| label_pubkeys[lbl_idx].0.clone())
    };
    let tap_input_raw = if labels.is_empty() {
        input.to_string()
    } else {
        rewrite_labels_with(&labels, input, |lbl_idx| label_pubkeys[lbl_idx].1.clone())
    };

    // §2.1 rewriting: Segwitv0 keeps `multi`/`sortedmulti`; Tap needs the
    // `_a` variants. We apply this textually before parsing.
    let tap_input = rewrite_multi_to_multi_a(&tap_input_raw);
    let segv0_input = rewrite_multi_a_to_multi(&segv0_input);

    // Strip whitespace — miniscript's parser is strict and does not accept
    // intra-argument whitespace, but users naturally write `multi(2, A, B, C)`.
    let segv0_input: String = segv0_input.chars().filter(|c| !c.is_whitespace()).collect();
    let tap_input: String = tap_input.chars().filter(|c| !c.is_whitespace()).collect();

    let segv0: Miniscript<DefiniteDescriptorKey, Segwitv0> =
        Miniscript::from_str(&segv0_input).map_err(|e| CompareCostError::Parse(format!("{e}")))?;
    let tap: Miniscript<DefiniteDescriptorKey, Tap> =
        Miniscript::from_str(&tap_input).map_err(|e| CompareCostError::ContextIncompat {
            valid_in: "Segwitv0",
            invalid_in: "Tap",
            detail: format!("{e}"),
        })?;

    Ok(Translated {
        extracted: input.to_string(),
        segv0,
        tap,
        concrete_keys,
        labels,
        label_pubkeys,
        tr_non_nums_internal_key_xonly_hex: None,
    })
}

/// Build `wsh(M)` from the Segwitv0-context miniscript.
pub fn build_wsh_descriptor(
    m: Miniscript<DefiniteDescriptorKey, Segwitv0>,
) -> Result<Descriptor<DefiniteDescriptorKey>, CompareCostError> {
    let wsh = Wsh::new(m).map_err(|e| CompareCostError::Parse(format!("wsh: {e}")))?;
    Ok(Descriptor::Wsh(wsh))
}

/// Build `tr(NUMS, {M})` from the Tap-context miniscript.
pub fn build_tr_descriptor(
    m: Miniscript<DefiniteDescriptorKey, Tap>,
) -> Result<Descriptor<DefiniteDescriptorKey>, CompareCostError> {
    let leaf = TapTree::leaf(Arc::new(m));
    let tr = Tr::new(nums_xonly_definite(), Some(leaf))
        .map_err(|e| CompareCostError::Parse(format!("tr: {e}")))?;
    Ok(Descriptor::Tr(tr))
}

/// Map a post-substitution `DescriptorPublicKey` (segwit-compressed form, as
/// rendered by `to_string()`) back to its user-supplied label, or `None` if
/// not found.
pub fn pubkey_to_label_segv0<'a>(
    pk: &DescriptorPublicKey,
    translated: &'a Translated,
) -> Option<&'a str> {
    let pk_str = pk.to_string();
    translated
        .label_pubkeys
        .iter()
        .enumerate()
        .find(|(_, (segv0_hex, _))| *segv0_hex == pk_str)
        .map(|(i, _)| translated.labels[i].as_str())
}

/// Same as `pubkey_to_label_segv0` but for x-only Tap-context pubkeys.
/// Used by the I3 order-stability debug_assert in `cost::enumerate`.
/// `#[allow(dead_code)]` survives because release builds strip the assertion.
#[allow(dead_code)]
pub fn pubkey_to_label_tap<'a>(
    pk: &DescriptorPublicKey,
    translated: &'a Translated,
) -> Option<&'a str> {
    let pk_str = pk.to_string();
    translated
        .label_pubkeys
        .iter()
        .enumerate()
        .find(|(_, (_, tap_hex))| *tap_hex == pk_str)
        .map(|(i, _)| translated.labels[i].as_str())
}

// ─── private helpers ────────────────────────────────────────────────────────

/// Scan the input for abstract labels in AST-left-to-right order. Each
/// distinct label is recorded once (in first-seen order).
fn collect_abstract_labels(input: &str) -> Vec<String> {
    let mut seen: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut labels: Vec<String> = Vec::new();
    let mut chars = input.char_indices().peekable();
    while let Some((_i, c)) = chars.next() {
        if c == '(' || c == ',' {
            let mut buf = String::new();
            while let Some(&(_j, nc)) = chars.peek() {
                if nc.is_whitespace() {
                    chars.next();
                    continue;
                }
                if nc.is_ascii_alphanumeric() || nc == '_' {
                    buf.push(nc);
                    chars.next();
                } else {
                    break;
                }
            }
            if buf.is_empty() {
                continue;
            }
            let is_hex_pubkey =
                (buf.len() == 64 || buf.len() == 66) && buf.chars().all(|c| c.is_ascii_hexdigit());
            let is_numeric = buf.chars().all(|c| c.is_ascii_digit());
            const KEYWORDS: &[&str] = &[
                "pk",
                "pk_k",
                "pk_h",
                "older",
                "after",
                "sha256",
                "hash256",
                "ripemd160",
                "hash160",
                "andor",
                "and_v",
                "and_b",
                "and_n",
                "or_b",
                "or_c",
                "or_d",
                "or_i",
                "thresh",
                "multi",
                "multi_a",
                "sortedmulti",
                "sortedmulti_a",
                "wsh",
                "sh",
                "tr",
                "wpkh",
                "pkh",
                "raw",
                "addr",
                "combo",
                "rawtr",
                "a",
                "s",
                "c",
                "t",
                "d",
                "v",
                "j",
                "n",
                "l",
                "u",
                "true",
                "false",
                "0",
                "1",
            ];
            let is_keyword = KEYWORDS.contains(&buf.as_str());
            let first_is_alpha = buf.chars().next().is_some_and(|c| c.is_ascii_alphabetic());
            if first_is_alpha
                && !is_hex_pubkey
                && !is_numeric
                && !is_keyword
                && seen.insert(buf.clone())
            {
                labels.push(buf);
            }
        }
    }
    labels
}

/// Replace each label in `input` with `f(label_index)`, in-place across all
/// occurrences. Longer labels are replaced first to avoid prefix-clobber.
fn rewrite_labels_with<F: Fn(usize) -> String>(labels: &[String], input: &str, f: F) -> String {
    // Sort by descending length so longer labels get replaced first.
    let mut sorted: Vec<(usize, &String)> = labels.iter().enumerate().collect();
    sorted.sort_by(|a, b| b.1.len().cmp(&a.1.len()));

    let mut out = input.to_string();
    for (idx, lbl) in sorted {
        let replacement = f(idx);
        out = replace_whole_word(&out, lbl, &replacement);
    }
    out
}

fn replace_whole_word(haystack: &str, needle: &str, replacement: &str) -> String {
    let mut result = String::with_capacity(haystack.len());
    let bytes = haystack.as_bytes();
    let needle_b = needle.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + needle_b.len() <= bytes.len() && &bytes[i..i + needle_b.len()] == needle_b {
            let before_ok = i == 0 || !is_ident_char(bytes[i - 1]);
            let after_ok =
                i + needle_b.len() == bytes.len() || !is_ident_char(bytes[i + needle_b.len()]);
            if before_ok && after_ok {
                result.push_str(replacement);
                i += needle_b.len();
                continue;
            }
        }
        result.push(bytes[i] as char);
        i += 1;
    }
    result
}

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn rewrite_multi_to_multi_a(input: &str) -> String {
    let s = replace_fragment(input, "sortedmulti(", "sortedmulti_a(");
    replace_fragment(&s, "multi(", "multi_a(")
}

fn rewrite_multi_a_to_multi(input: &str) -> String {
    let s = replace_fragment(input, "sortedmulti_a(", "sortedmulti(");
    replace_fragment(&s, "multi_a(", "multi(")
}

fn replace_fragment(haystack: &str, find: &str, repl: &str) -> String {
    let mut result = String::with_capacity(haystack.len());
    let bytes = haystack.as_bytes();
    let find_b = find.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + find_b.len() <= bytes.len() && &bytes[i..i + find_b.len()] == find_b {
            let before_ok = i == 0 || !is_ident_char(bytes[i - 1]);
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
    fn collects_abstract_labels() {
        let labels = collect_abstract_labels("or_b(pk(A),s:pk(B))");
        assert_eq!(labels, vec!["A".to_string(), "B".to_string()]);
    }

    #[test]
    fn collects_in_ast_left_to_right_order() {
        // Labels collected in left-to-right order of first appearance.
        let labels = collect_abstract_labels("or_b(pk(Z),s:pk(A))");
        assert_eq!(
            labels,
            vec!["Z".to_string(), "A".to_string()],
            "Z appears first, so should be label[0]"
        );
    }

    #[test]
    fn skips_hex_pubkeys() {
        let input = "pk(02ababababababababababababababababababababababababababababababababab)";
        let labels = collect_abstract_labels(input);
        assert!(labels.is_empty());
    }

    #[test]
    fn skips_numeric_thresholds() {
        let labels = collect_abstract_labels("thresh(2, pk(A), pk(B), pk(C))");
        assert_eq!(
            labels,
            vec!["A".to_string(), "B".to_string(), "C".to_string()]
        );
    }

    #[test]
    fn collects_alice_bob_carol() {
        let labels = collect_abstract_labels("multi(2, Alice, Bob, Carol)");
        assert_eq!(
            labels,
            vec!["Alice".to_string(), "Bob".to_string(), "Carol".to_string()]
        );
    }

    #[test]
    fn rewrite_multi_to_multi_a_basic() {
        assert_eq!(
            rewrite_multi_to_multi_a("multi(2,A,B,C)"),
            "multi_a(2,A,B,C)"
        );
        assert_eq!(
            rewrite_multi_to_multi_a("sortedmulti(2,A,B)"),
            "sortedmulti_a(2,A,B)"
        );
    }

    #[test]
    fn rewrite_multi_a_to_multi_basic() {
        assert_eq!(
            rewrite_multi_a_to_multi("multi_a(2,A,B,C)"),
            "multi(2,A,B,C)"
        );
        assert_eq!(
            rewrite_multi_a_to_multi("sortedmulti_a(2,A,B)"),
            "sortedmulti(2,A,B)"
        );
    }

    #[test]
    fn translate_simple_pk() {
        let t = translate_miniscript("pk(A)").expect("simple pk parses");
        assert!(!t.concrete_keys);
        assert_eq!(t.labels, vec!["A".to_string()]);
        assert_eq!(t.label_pubkeys.len(), 1);
    }

    #[test]
    fn translate_multi_rewrites_for_tap() {
        let t = translate_miniscript("multi(2, A, B, C)").expect("multi parses on both sides");
        assert!(!t.concrete_keys);
        assert_eq!(t.labels.len(), 3);
    }
}
