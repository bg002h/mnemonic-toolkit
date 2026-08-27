//! Canonical mstring DISPLAY-GROUPING layer (SPEC §3). Pure, ASCII-safe,
//! dependency-free. A dedicated lib module (NOT bin-private `format.rs`) so the
//! conformance test and `--lib` unit tests reach it, and so the bin-private
//! heavy API stays out of the public lib surface. P4 routes the toolkit's emit
//! sites through `render_grouped` and deletes `format.rs::chunk_*`.

/// True for any character treated as a display separator on intake: ALL Unicode
/// whitespace plus `-` and `,`. SPEC §3.2. The OUTPUT separator set is the
/// subset {space, '-', ','}; every emitted grouped form therefore re-ingests.
/// None of these chars appear in the codex32 alphabet
/// (`qpzry9x8gf2tvdw0s3jn54khce6mua7l`) or the `ms`/`mk`/`md`/`1` structural
/// chars (SPEC §4), so stripping is unambiguous.
pub fn is_display_separator(c: char) -> bool {
    c.is_whitespace() || c == '-' || c == ','
}

/// Insert `separator` after every `group_size` characters (SPEC §3.1).
/// `group_size == 0` returns the input unchanged (unbroken; `separator`
/// ignored). Single line always — no newline wrapping. ASCII-safe.
pub fn render_grouped(s: &str, group_size: usize, separator: char) -> String {
    if group_size == 0 {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len() + s.len() / group_size);
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && i % group_size == 0 {
            out.push(separator);
        }
        out.push(ch);
    }
    out
}

/// Strip every display separator (SPEC §3.2) — used on intake before decode so
/// grouped and unbroken forms both re-ingest. Idempotent. Strips ONLY
/// separators; any other char (incl. codex32-alphabet chars) passes through, so
/// a malformed card is never silently "cleaned" into validity.
pub fn strip_display_separators(s: &str) -> String {
    s.chars().filter(|&c| !is_display_separator(c)).collect()
}

/// Parse `--separator`: the keyword `space`, or the literal `" "`. Returns the
/// separator char. SPEC §5. clap value-parser; rejection surfaces as a clap
/// parse error (before command dispatch).
///
/// **`hyphen` and `comma` were RETIRED by `SPEC_constellation_cli_uniformity`
/// 6c (P3), and the reason is cross-tool rather than cosmetic.** A grouped card
/// is what a human types back into *another* tool, and `mt`'s decoder strips
/// whitespace and nothing else — so a hyphen-grouped string round-trips here
/// and is refused there, after the plates are cut. A rule that is safe per-tool
/// and unsafe across tools is exactly the kind an operator carries between
/// tools. The cost is two cosmetic options; the cost of getting it wrong is a
/// plate.
///
/// **The narrowing is at the CLI's vocabulary and NOWHERE ELSE.** The four-repo
/// display-grouping corpus (`design/display-grouping-vectors.tsv`, sha256
/// `7147b0ec…`) still carries `hyphen` and `comma` rows and still passes: its
/// consumers map the keyword to a `char` inside the test, and [`render_grouped`]
/// takes a `char` and has no keyword vocabulary at all. Likewise
/// [`is_display_separator`] still strips `-` and `,` on **intake** (SPEC §3.2),
/// so a card grouped by an older build — or by hand — still re-ingests.
/// Narrowing either of those would be applying 6c one layer too deep.
pub fn parse_separator(s: &str) -> Result<char, String> {
    match s {
        "space" | " " => Ok(' '),
        other => Err(format!(
            "invalid separator {other:?}; the display separator is whitespace only \
             -- pass `space` or the literal \" \". `hyphen` and `comma` were retired \
             (SPEC_constellation_cli_uniformity 6c): mt's decoder strips whitespace \
             and nothing else, so a hyphen-grouped card round-trips here and is \
             refused there. Cards you already hold still re-ingest -- intake strips \
             '-' and ',' unchanged."
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_grouped_basic_space() {
        assert_eq!(render_grouped("abcdefghij", 5, ' '), "abcde fghij");
    }

    #[test]
    fn render_grouped_zero_is_unbroken() {
        assert_eq!(render_grouped("abcdefghij", 0, ' '), "abcdefghij");
        assert_eq!(render_grouped("abcdefghij", 0, '-'), "abcdefghij");
    }

    #[test]
    fn render_grouped_group_size_ge_len_unchanged() {
        assert_eq!(render_grouped("abc", 5, ' '), "abc");
        assert_eq!(render_grouped("abcde", 5, ' '), "abcde"); // no trailing sep
    }

    #[test]
    fn render_grouped_trailing_partial() {
        assert_eq!(render_grouped("abcdefg", 3, '-'), "abc-def-g");
    }

    #[test]
    fn render_grouped_empty() {
        assert_eq!(render_grouped("", 5, ' '), "");
    }

    #[test]
    fn strip_display_separators_all_kinds() {
        assert_eq!(strip_display_separators("abcde fghij"), "abcdefghij");
        assert_eq!(strip_display_separators("abcde-fghij"), "abcdefghij");
        assert_eq!(strip_display_separators("abcde,fghij"), "abcdefghij");
        assert_eq!(strip_display_separators("ab cd-ef,gh"), "abcdefgh");
    }

    #[test]
    fn strip_display_separators_whitespace_kinds() {
        assert_eq!(strip_display_separators("ab\tcd"), "abcd");
        assert_eq!(strip_display_separators("ab\r\ncd"), "abcd");
    }

    #[test]
    fn strip_display_separators_idempotent() {
        let once = strip_display_separators("ab cd-ef");
        assert_eq!(strip_display_separators(&once), once);
    }

    #[test]
    fn strip_display_separators_passes_codex32_chars() {
        assert_eq!(strip_display_separators("ms1qpzry9x8"), "ms1qpzry9x8");
    }

    #[test]
    fn parse_separator_keyword_and_literal() {
        assert_eq!(parse_separator("space").unwrap(), ' ');
        assert_eq!(parse_separator(" ").unwrap(), ' ');
        assert!(parse_separator("bogus").is_err());
    }

    /// SPEC_constellation_cli_uniformity 6c (P3): the CLI's separator vocabulary
    /// is whitespace only. Both retired spellings AND both retired literals are
    /// pinned, because a narrowing that dropped only the keywords would leave
    /// `--separator -` producing exactly the card `mt` refuses.
    #[test]
    fn parse_separator_rejects_the_retired_hyphen_and_comma() {
        for retired in ["hyphen", "-", "comma", ","] {
            let err = parse_separator(retired).expect_err("6c retired this value");
            assert!(
                err.contains("whitespace only"),
                "the refusal must name what replaced it; got {err}"
            );
            assert!(
                err.contains("re-ingest"),
                "the refusal must say that cards already grouped still decode; got {err}"
            );
        }
    }

    /// The layer control. 6c narrows the CLI's INPUT vocabulary; INTAKE keeps
    /// stripping `-` and `,` so a card grouped by an older build still decodes.
    /// A fix applied one layer too deep reds here.
    #[test]
    fn intake_still_strips_the_retired_separators() {
        assert!(is_display_separator('-'));
        assert!(is_display_separator(','));
        assert_eq!(strip_display_separators("ms1qp-zry9,x8"), "ms1qpzry9x8");
        assert_eq!(render_grouped("abcdefg", 3, '-'), "abc-def-g");
    }

    #[test]
    fn render_then_strip_round_trips() {
        let s = "ms1qpzry9x8gf2tvdw";
        for gs in [0usize, 1, 4, 5, 100] {
            for sep in [' ', '-', ','] {
                assert_eq!(strip_display_separators(&render_grouped(s, gs, sep)), s);
            }
        }
    }
}
