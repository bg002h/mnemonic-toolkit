//! `--spec-schema` — the versioned node-tree grammar the GUI + Release-B
//! presets consume to render/validate specs generically.
//!
//! SPEC §2 (`--spec-schema` flag) + §5.3 + risk #2: this is a **separate
//! versioned contract** from the flat `gui-schema` flag projection. The
//! `spec_schema_version` is bumped independently when the node grammar changes.

use serde_json::{json, Value};

use super::ir::{MULTIPATH_SUFFIX, SUPPORTED_SCHEMA_VERSION};

/// The grammar's own version. Bump when the node set / field shapes change.
pub const SPEC_SCHEMA_VERSION: u32 = 1;

/// One node-grammar entry: external tag + payload shape + the miniscript it
/// renders to. The `kind`s here MUST equal [`NODE_KINDS`] (self-tested).
struct NodeGrammar {
    kind: &'static str,
    /// serde external-tag payload form.
    form: &'static str,
    /// human grammar hint.
    renders: &'static str,
}

const NODE_GRAMMAR: &[NodeGrammar] = &[
    NodeGrammar { kind: "pk", form: "string(key)", renders: "pk(<key>)" },
    NodeGrammar { kind: "pkh", form: "string(key)", renders: "pkh(<key>)" },
    NodeGrammar { kind: "multi", form: "{k:uint, keys:[key]}", renders: "multi(K, …)" },
    NodeGrammar { kind: "sortedmulti", form: "{k:uint, keys:[key]}", renders: "sortedmulti(K, …)" },
    NodeGrammar { kind: "older", form: "uint", renders: "older(N)" },
    NodeGrammar { kind: "after", form: "uint", renders: "after(N)" },
    NodeGrammar { kind: "sha256", form: "string(hex64)", renders: "sha256(<hex>)" },
    NodeGrammar { kind: "hash256", form: "string(hex64)", renders: "hash256(<hex>)" },
    NodeGrammar { kind: "hash160", form: "string(hex40)", renders: "hash160(<hex>)" },
    NodeGrammar { kind: "ripemd160", form: "string(hex40)", renders: "ripemd160(<hex>)" },
    NodeGrammar { kind: "and_v", form: "[node, node]", renders: "and_v(A,B)" },
    NodeGrammar { kind: "or_d", form: "[node, node]", renders: "or_d(A,B)" },
    NodeGrammar { kind: "or_i", form: "[node, node]", renders: "or_i(A,B)" },
    NodeGrammar { kind: "or_b", form: "[node, node]", renders: "or_b(A,B)" },
    NodeGrammar { kind: "andor", form: "[node, node, node]", renders: "andor(A,B,C)" },
    NodeGrammar { kind: "thresh", form: "{k:uint, subs:[node]}", renders: "thresh(K, …)" },
    NodeGrammar { kind: "wrap", form: "{w:string, sub:node}", renders: "<w>:<sub>" },
];

/// Build the `--spec-schema` JSON value.
pub fn spec_schema_json() -> Value {
    let nodes: Vec<Value> = NODE_GRAMMAR
        .iter()
        .map(|g| {
            json!({
                "kind": g.kind,
                "tag": format!("{{\"{}\": <payload>}}", g.kind),
                "payload": g.form,
                "renders": g.renders,
            })
        })
        .collect();
    json!({
        "spec_schema_version": SPEC_SCHEMA_VERSION,
        "supported_doc_schema_version": SUPPORTED_SCHEMA_VERSION,
        "doc_shape": "{schema_version: uint, wrapper: enum, root: node}",
        "wrapper": { "values": ["wsh"] },
        "multipath_suffix": MULTIPATH_SUFFIX,
        "node_tagging": "externally-tagged (exactly one key per node); unknown fields rejected",
        "nodes": nodes,
    })
}

/// Pretty-printed `--spec-schema` output (one trailing newline added by caller).
pub fn spec_schema_string() -> String {
    serde_json::to_string_pretty(&spec_schema_json()).expect("schema serializes")
}

#[cfg(test)]
mod tests {
    use super::super::ir::NODE_KINDS;
    use super::*;

    #[test]
    fn grammar_matches_node_kinds_hand_list() {
        // Hand-list vs hand-list: NODE_GRAMMAR (schema) == NODE_KINDS (ir).
        // Companion cross-check: `ir::tests::node_kinds_cover_enum` asserts the
        // hand-maintained `all_variant_samples` tag set == NODE_KINDS. Together
        // they catch *drift* across the three hand-lists — a partial (drift-only)
        // freeze. Neither catches a variant jointly omitted from all three
        // (enum→list is author discipline; an airtight guard would need a
        // variant-enumerator macro).
        let grammar: Vec<&str> = NODE_GRAMMAR.iter().map(|g| g.kind).collect();
        assert_eq!(
            grammar, NODE_KINDS,
            "--spec-schema node grammar must match ir::NODE_KINDS exactly (order + set)"
        );
    }

    #[test]
    fn schema_advertises_v1_and_is_valid_json() {
        let v = spec_schema_json();
        assert_eq!(v["spec_schema_version"], SPEC_SCHEMA_VERSION);
        assert_eq!(v["supported_doc_schema_version"], SUPPORTED_SCHEMA_VERSION);
        assert_eq!(v["multipath_suffix"], MULTIPATH_SUFFIX);
        assert_eq!(v["nodes"].as_array().unwrap().len(), NODE_KINDS.len());
        // round-trips as JSON
        let s = spec_schema_string();
        let _: Value = serde_json::from_str(&s).expect("schema string is valid JSON");
    }
}
