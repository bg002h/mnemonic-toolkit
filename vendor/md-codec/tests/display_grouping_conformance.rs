//! Same canonical display-grouping vectors as the toolkit + the other siblings
//! (copy is checksum-pinned in CI). Proves md-codec's render/strip match
//! byte-for-byte. SPEC §8.

use md_codec::encode::{render_grouped, strip_display_separators};

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
    assert_eq!(
        lines.next().expect("header"),
        "op\tinput\tgroup_size\tseparator\texpected\tnote",
        "vector header drift"
    );
    let mut count = 0usize;
    for (i, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let c: Vec<&str> = line.split('\t').collect();
        assert_eq!(c.len(), 6, "row {} not 6 fields: {line:?}", i + 2);
        let (op, input, gs, sep, expected, note) = (c[0], c[1], c[2], c[3], c[4], c[5]);
        let (input, expected) = (decode(input), decode(expected));
        let gs: usize = gs
            .parse()
            .unwrap_or_else(|_| panic!("row {}: bad group_size", i + 2));
        let got = match op {
            "render" => render_grouped(&input, gs, sep_char(sep)),
            "strip" => strip_display_separators(&input),
            other => panic!("row {}: unknown op {other:?}", i + 2),
        };
        assert_eq!(got, expected, "row {} ({note})", i + 2);
        count += 1;
    }
    assert!(count >= 20, "expected >=20 rows, got {count}");
}
