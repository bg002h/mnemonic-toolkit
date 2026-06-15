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
    fn render_then_strip_round_trips() {
        let s = "ms1qpzry9x8gf2tvdw";
        for gs in [0usize, 1, 4, 5, 100] {
            for sep in [' ', '-', ','] {
                assert_eq!(strip_display_separators(&render_grouped(s, gs, sep)), s);
            }
        }
    }
}
