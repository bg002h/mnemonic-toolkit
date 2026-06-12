//! `--spec-schema` — the versioned node-tree grammar the GUI + Release-B
//! presets consume to render/validate specs generically.
//!
//! SPEC §2 (`--spec-schema` flag) + §5.3 + risk #2: this is a **separate
//! versioned contract** from the flat `gui-schema` flag projection. The
//! `spec_schema_version` is bumped independently when the node grammar changes.

use serde_json::{json, Value};

use super::archetype::ARCHETYPE_REGISTRY;
use super::ir::{MULTIPATH_SUFFIX, NODE_KINDS, SUPPORTED_SCHEMA_VERSION};

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
    NodeGrammar {
        kind: "pk",
        form: "string(key)",
        renders: "pk(<key>)",
    },
    NodeGrammar {
        kind: "pkh",
        form: "string(key)",
        renders: "pkh(<key>)",
    },
    NodeGrammar {
        kind: "multi",
        form: "{k:uint, keys:[key]}",
        renders: "multi(K, …)",
    },
    NodeGrammar {
        kind: "sortedmulti",
        form: "{k:uint, keys:[key]}",
        renders: "sortedmulti(K, …)",
    },
    NodeGrammar {
        kind: "older",
        form: "uint",
        renders: "older(N)",
    },
    NodeGrammar {
        kind: "after",
        form: "uint",
        renders: "after(N)",
    },
    NodeGrammar {
        kind: "sha256",
        form: "string(hex64)",
        renders: "sha256(<hex>)",
    },
    NodeGrammar {
        kind: "hash256",
        form: "string(hex64)",
        renders: "hash256(<hex>)",
    },
    NodeGrammar {
        kind: "hash160",
        form: "string(hex40)",
        renders: "hash160(<hex>)",
    },
    NodeGrammar {
        kind: "ripemd160",
        form: "string(hex40)",
        renders: "ripemd160(<hex>)",
    },
    NodeGrammar {
        kind: "and_v",
        form: "[node, node]",
        renders: "and_v(A,B)",
    },
    NodeGrammar {
        kind: "or_d",
        form: "[node, node]",
        renders: "or_d(A,B)",
    },
    NodeGrammar {
        kind: "or_i",
        form: "[node, node]",
        renders: "or_i(A,B)",
    },
    NodeGrammar {
        kind: "or_b",
        form: "[node, node]",
        renders: "or_b(A,B)",
    },
    NodeGrammar {
        kind: "andor",
        form: "[node, node, node]",
        renders: "andor(A,B,C)",
    },
    NodeGrammar {
        kind: "thresh",
        form: "{k:uint, subs:[node]}",
        renders: "thresh(K, …)",
    },
    NodeGrammar {
        kind: "wrap",
        form: "{w:string, sub:node}",
        renders: "<w>:<sub>",
    },
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
    // Release-B archetype field-specs (presets SPEC §5) — generated FROM the
    // registry (no hand-maintained copy). Additive sibling key: the node
    // grammar is untouched, so SPEC_SCHEMA_VERSION stays 1. Wire projection:
    // `min_count` → `min`; `ParamKind` → snake_case `as_str`.
    let archetypes: Vec<Value> = ARCHETYPE_REGISTRY
        .iter()
        .map(|d| {
            let params: Vec<Value> = d
                .params
                .iter()
                .map(|p| {
                    json!({
                        "flag": p.flag,
                        "kind": p.kind.as_str(),
                        "required": p.required,
                        "repeatable": p.repeatable,
                        "min": p.min_count,
                    })
                })
                .collect();
            json!({ "id": d.id, "summary": d.summary, "params": params })
        })
        .collect();
    json!({
        "spec_schema_version": SPEC_SCHEMA_VERSION,
        "supported_doc_schema_version": SUPPORTED_SCHEMA_VERSION,
        "doc_shape": "{schema_version: uint, wrapper: enum, root: node}",
        "wrapper": { "values": ["wsh"] },
        "multipath_suffix": MULTIPATH_SUFFIX,
        "node_tagging": "externally-tagged (exactly one key per node); unknown fields rejected",
        "node_kinds": NODE_KINDS,
        "nodes": nodes,
        "archetypes": archetypes,
    })
}

/// Pretty-printed `--spec-schema` output (one trailing newline added by caller).
pub fn spec_schema_string() -> String {
    serde_json::to_string_pretty(&spec_schema_json()).expect("schema serializes")
}

#[cfg(test)]
mod tests {
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

    /// Drift self-test (a) extended to the schema (presets SPEC §7/§9): the
    /// `archetypes` section ids == the registry ids, one entry per archetype,
    /// every param projected with the pinned wire keys.
    #[test]
    fn schema_archetypes_match_registry() {
        let v = spec_schema_json();
        let schema_ids: Vec<&str> = v["archetypes"]
            .as_array()
            .expect("archetypes array")
            .iter()
            .map(|a| a["id"].as_str().unwrap())
            .collect();
        let registry_ids: Vec<&str> = ARCHETYPE_REGISTRY.iter().map(|d| d.id).collect();
        assert_eq!(schema_ids, registry_ids);
        for (a, def) in v["archetypes"]
            .as_array()
            .unwrap()
            .iter()
            .zip(ARCHETYPE_REGISTRY)
        {
            let params = a["params"].as_array().unwrap();
            assert_eq!(params.len(), def.params.len(), "{}", def.id);
            for (p, spec) in params.iter().zip(def.params) {
                assert_eq!(p["flag"], spec.flag);
                assert_eq!(p["kind"], spec.kind.as_str());
                assert_eq!(p["required"], spec.required);
                assert_eq!(p["repeatable"], spec.repeatable);
                assert_eq!(p["min"], spec.min_count);
            }
        }
    }
}
