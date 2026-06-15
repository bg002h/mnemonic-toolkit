//! Drives the canonical display-grouping conformance vectors
//! (`design/display-grouping-vectors.tsv`) through the toolkit's reference
//! `render_grouped` / `strip_display_separators`. This SAME vector file is
//! copied (verbatim, checksum-pinned) into each sibling repo in P1–P3, so all
//! four implementations are proven byte-identical. SPEC §8.

use mnemonic_toolkit::display_grouping::{render_grouped, strip_display_separators};

/// Decode the field sentinels defined by the vector-encoding convention.
fn decode(field: &str) -> String {
    if field == "<empty>" {
        return String::new();
    }
    field
        .replace("<sp>", " ")
        .replace("<tab>", "\t")
        .replace("<lf>", "\n")
        .replace("<cr>", "\r")
}

fn sep_char(keyword: &str) -> char {
    match keyword {
        "space" => ' ',
        "hyphen" => '-',
        "comma" => ',',
        // returned value ignored by render_grouped when group_size==0; never used by strip
        "none" => ' ',
        other => panic!("unknown separator keyword: {other}"),
    }
}

#[test]
fn conformance_vectors_pass() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../design/display-grouping-vectors.tsv"
    );
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));

    let mut lines = text.lines();
    let header = lines.next().expect("header row");
    assert_eq!(
        header, "op\tinput\tgroup_size\tseparator\texpected\tnote",
        "vector header drift"
    );

    let mut count = 0usize;
    for (i, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(cols.len(), 6, "row {} not 6 tab-fields: {line:?}", i + 2);
        let (op, input, gs, sep, expected, note) =
            (cols[0], cols[1], cols[2], cols[3], cols[4], cols[5]);
        let input = decode(input);
        let expected = decode(expected);
        let gs: usize = gs
            .parse()
            .unwrap_or_else(|_| panic!("row {}: bad group_size {gs:?}", i + 2));

        let got = match op {
            "render" => render_grouped(&input, gs, sep_char(sep)),
            "strip" => strip_display_separators(&input),
            other => panic!("row {}: unknown op {other:?}", i + 2),
        };
        assert_eq!(
            got,
            expected,
            "row {} ({note}): {op}({input:?}, {gs}, {sep})",
            i + 2
        );
        count += 1;
    }
    assert!(count >= 20, "expected >=20 vector rows, got {count}");
}
