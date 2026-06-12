//! `descriptor-builder` IR — the versioned `PolicyNode` fragment tree.
//!
//! SPEC `design/SPEC_descriptor_builder_engine.md` §1. The IR is a fragment-
//! level mirror of the miniscript AST, serialized as **versioned JSON** and
//! rendered to a byte-stable miniscript string (the engine's own emitter — NOT
//! `wallet_export::pipeline::build_descriptor_string`, which is template-coupled
//! per R0-r1 C1).
//!
//! **Encoding (R0-r1 fold / Phase-1 decision):** each [`PolicyNode`] is an
//! externally-tagged serde enum (`{"<frag>": <payload>}`, exactly one key) with
//! `deny_unknown_fields` on every struct payload. serde's `deny_unknown_fields`
//! is incompatible with `#[serde(flatten)]` and with internally/adjacently-
//! tagged enums, so the wrapper is its own node `{"wrap":{"w":..,"sub":..}}`
//! rather than an adjacent `"w"` field. Same expressiveness, but typo'd fields
//! are rejected loudly (the funds-safety property).

use std::fmt;

use serde::{Deserialize, Serialize};

/// Multipath receive+change suffix appended to every key by the renderer
/// (BIP-388 `/<0;1>/*`). Inputs are account-level `[fp/path]xpub` strings.
pub const MULTIPATH_SUFFIX: &str = "/<0;1>/*";

/// The only schema version v1 understands.
pub const SUPPORTED_SCHEMA_VERSION: u32 = 1;

/// Generates `NODE_KINDS` (every `PolicyNode` external tag) AND
/// `PolicyNode::kind()` from a SINGLE `(Variant => "kind")` list — so they can
/// never drift, and a NEW variant is FORCED into both: the generated `kind()`
/// `match self` is exhaustive, so a variant missing from the macro list is a
/// COMPILE error; extending the list automatically grows `NODE_KINDS`, and
/// `node_kinds_cover_enum` (samples == NODE_KINDS) then FAILS until the sample
/// is added — no longer vacuous (closes `policynode-grammar-coverage-vacuous-
/// on-joint-omission`). `grammar_matches_node_kinds_hand_list` is likewise
/// de-vacuified.
///
/// Precedent: `cmd::convert::declare_node_type_variants!` builds a const VALUE
/// array (possible only for unit variants); this builds a `match self` METHOD
/// because `PolicyNode` carries data. All 17 variants are single-field tuple
/// variants, so `PolicyNode::$variant(..)` matches each (a future UNIT variant
/// would fail the `(..)` pattern — an acceptable extra forcing function).
macro_rules! declare_policy_node_kinds {
    ( $( $variant:ident => $kind:literal ),* $(,)? ) => {
        /// Every `PolicyNode` external tag (the single JSON key). Macro-generated
        /// (complete-by-construction); consumed by `super::schema`.
        pub const NODE_KINDS: &[&str] = &[ $( $kind ),* ];

        impl PolicyNode {
            /// The external tag (single JSON key) for this node — also the
            /// diagnostic fragment label. Macro-generated alongside `NODE_KINDS`.
            pub fn kind(&self) -> &'static str {
                match self { $( PolicyNode::$variant(..) => $kind ),* }
            }
        }
    };
}

declare_policy_node_kinds!(
    Pk => "pk",
    Pkh => "pkh",
    Multi => "multi",
    Sortedmulti => "sortedmulti",
    Older => "older",
    After => "after",
    Sha256 => "sha256",
    Hash256 => "hash256",
    Hash160 => "hash160",
    Ripemd160 => "ripemd160",
    AndV => "and_v",
    OrD => "or_d",
    OrI => "or_i",
    OrB => "or_b",
    Andor => "andor",
    Thresh => "thresh",
    Wrap => "wrap",
);

/// The top-level versioned spec document (`--spec` input).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecDoc {
    /// Spec schema version. v1 only (checked by [`SpecDoc::parse`]).
    pub schema_version: u32,
    /// Output wrapper. v1 = `wsh` only.
    pub wrapper: WrapperKind,
    /// The policy fragment tree.
    pub root: PolicyNode,
}

/// Output script wrapper. v1 ships `wsh` only; `tr` is the deferred
/// wrapper-strategy seam (SPEC §5.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WrapperKind {
    Wsh,
}

impl WrapperKind {
    fn as_str(self) -> &'static str {
        match self {
            WrapperKind::Wsh => "wsh",
        }
    }
}

/// A fragment-level miniscript node. Externally tagged.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyNode {
    /// `pk(<key>)`
    Pk(String),
    /// `pkh(<key>)`
    Pkh(String),
    /// `multi(K, …)` — wsh unsorted
    Multi(MultiSpec),
    /// `sortedmulti(K, …)` — wsh sorted
    Sortedmulti(MultiSpec),
    /// `older(N)` — relative timelock
    Older(u32),
    /// `after(N)` — absolute timelock
    After(u32),
    /// `sha256(<hex>)`
    Sha256(String),
    /// `hash256(<hex>)`
    Hash256(String),
    /// `hash160(<hex>)`
    Hash160(String),
    /// `ripemd160(<hex>)`
    Ripemd160(String),
    /// `and_v(A,B)`
    AndV(Box<[PolicyNode; 2]>),
    /// `or_d(A,B)`
    OrD(Box<[PolicyNode; 2]>),
    /// `or_i(A,B)`
    OrI(Box<[PolicyNode; 2]>),
    /// `or_b(A,B)`
    OrB(Box<[PolicyNode; 2]>),
    /// `andor(A,B,C)`
    Andor(Box<[PolicyNode; 3]>),
    /// `thresh(K, …)`
    Thresh(ThreshSpec),
    /// `<w>:<sub>` — explicit miniscript wrapper(s)
    Wrap(WrapSpec),
}

/// `multi` / `sortedmulti` payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MultiSpec {
    /// Threshold.
    pub k: u32,
    /// Cosigner keys (account-level `[fp/path]xpub`).
    pub keys: Vec<String>,
}

/// `thresh` payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThreshSpec {
    /// Threshold.
    pub k: u32,
    /// Sub-policies.
    pub subs: Vec<PolicyNode>,
}

/// `wrap` payload — `<w>:<sub>`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WrapSpec {
    /// Wrapper string (e.g. `"v"`, `"sv"`) emitted verbatim as a prefix.
    pub w: String,
    /// The wrapped node.
    pub sub: Box<PolicyNode>,
}

/// Failure parsing a `--spec` document into a [`SpecDoc`].
#[derive(Debug)]
pub enum SpecParseError {
    /// serde/JSON structural failure (unknown field, bad arity, missing key…).
    Json(serde_json::Error),
    /// `schema_version` is not [`SUPPORTED_SCHEMA_VERSION`].
    UnsupportedVersion(u32),
}

impl fmt::Display for SpecParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpecParseError::Json(e) => write!(f, "spec JSON parse error: {e}"),
            SpecParseError::UnsupportedVersion(v) => write!(
                f,
                "unsupported spec schema_version {v} (this build supports {SUPPORTED_SCHEMA_VERSION})"
            ),
        }
    }
}

impl std::error::Error for SpecParseError {}

impl SpecDoc {
    /// Parse + version-check a `--spec` JSON document.
    pub fn parse(json: &str) -> Result<SpecDoc, SpecParseError> {
        let doc: SpecDoc = serde_json::from_str(json).map_err(SpecParseError::Json)?;
        if doc.schema_version != SUPPORTED_SCHEMA_VERSION {
            return Err(SpecParseError::UnsupportedVersion(doc.schema_version));
        }
        Ok(doc)
    }

    /// Render the full descriptor body (`wsh(<inner>)`), pre-canonicalization.
    /// Canonicalization + BIP-380 checksum happen at emit (Phase 3) via the
    /// `from_str(&s)?.to_string()` round-trip idiom.
    pub fn render_descriptor(&self) -> String {
        format!("{}({})", self.wrapper.as_str(), self.root.render())
    }
}

fn with_multipath(key: &str) -> String {
    format!("{key}{MULTIPATH_SUFFIX}")
}

fn render_keys(keys: &[String]) -> String {
    keys.iter()
        .map(|k| with_multipath(k))
        .collect::<Vec<_>>()
        .join(",")
}

impl PolicyNode {
    // `kind()` is macro-generated by `declare_policy_node_kinds!` above (it +
    // `NODE_KINDS` share a single variant list — see that macro's doc).

    /// Render this node to its miniscript text (recursive `Display`). Pure
    /// string concatenation — infallible; type-correctness is the Phase-2 gate.
    pub fn render(&self) -> String {
        match self {
            PolicyNode::Pk(k) => format!("pk({})", with_multipath(k)),
            PolicyNode::Pkh(k) => format!("pkh({})", with_multipath(k)),
            PolicyNode::Multi(m) => format!("multi({},{})", m.k, render_keys(&m.keys)),
            PolicyNode::Sortedmulti(m) => {
                format!("sortedmulti({},{})", m.k, render_keys(&m.keys))
            }
            PolicyNode::Older(n) => format!("older({n})"),
            PolicyNode::After(n) => format!("after({n})"),
            PolicyNode::Sha256(h) => format!("sha256({h})"),
            PolicyNode::Hash256(h) => format!("hash256({h})"),
            PolicyNode::Hash160(h) => format!("hash160({h})"),
            PolicyNode::Ripemd160(h) => format!("ripemd160({h})"),
            PolicyNode::AndV(s) => format!("and_v({},{})", s[0].render(), s[1].render()),
            PolicyNode::OrD(s) => format!("or_d({},{})", s[0].render(), s[1].render()),
            PolicyNode::OrI(s) => format!("or_i({},{})", s[0].render(), s[1].render()),
            PolicyNode::OrB(s) => format!("or_b({},{})", s[0].render(), s[1].render()),
            PolicyNode::Andor(s) => {
                format!(
                    "andor({},{},{})",
                    s[0].render(),
                    s[1].render(),
                    s[2].render()
                )
            }
            PolicyNode::Thresh(t) => {
                let subs = t
                    .subs
                    .iter()
                    .map(|s| s.render())
                    .collect::<Vec<_>>()
                    .join(",");
                format!("thresh({},{})", t.k, subs)
            }
            PolicyNode::Wrap(w) => format!("{}:{}", w.w, w.sub.render()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn k() -> String {
        "KEY".to_string()
    }
    fn pk() -> PolicyNode {
        PolicyNode::Pk(k())
    }

    /// One hand-maintained sample per enum variant. The exhaustive `match`
    /// below breaks compilation when a `PolicyNode` variant is added — a
    /// REMINDER that forces an author to visit this helper — but it does NOT
    /// force the new variant into the `samples` vec (E0004 is a property of the
    /// match arms, not of the vec's contents). Closing the loop is author
    /// discipline: add the variant to `samples`, [`NODE_KINDS`], and
    /// `schema::NODE_GRAMMAR`. `node_kinds_cover_enum` then cross-checks
    /// `samples`'s tag set == `NODE_KINDS`.
    fn all_variant_samples() -> Vec<PolicyNode> {
        let samples = vec![
            PolicyNode::Pk(k()),
            PolicyNode::Pkh(k()),
            PolicyNode::Multi(MultiSpec {
                k: 1,
                keys: vec![k()],
            }),
            PolicyNode::Sortedmulti(MultiSpec {
                k: 1,
                keys: vec![k()],
            }),
            PolicyNode::Older(1),
            PolicyNode::After(1),
            PolicyNode::Sha256("aa".into()),
            PolicyNode::Hash256("aa".into()),
            PolicyNode::Hash160("aa".into()),
            PolicyNode::Ripemd160("aa".into()),
            PolicyNode::AndV(Box::new([pk(), pk()])),
            PolicyNode::OrD(Box::new([pk(), pk()])),
            PolicyNode::OrI(Box::new([pk(), pk()])),
            PolicyNode::OrB(Box::new([pk(), pk()])),
            PolicyNode::Andor(Box::new([pk(), pk(), pk()])),
            PolicyNode::Thresh(ThreshSpec {
                k: 1,
                subs: vec![pk()],
            }),
            PolicyNode::Wrap(WrapSpec {
                w: "v".into(),
                sub: Box::new(pk()),
            }),
        ];
        for n in &samples {
            // Exhaustiveness REMINDER — a new variant makes this match
            // non-exhaustive (compile error), forcing an author to visit here.
            // It does not by itself add the variant to `samples` (see fn doc).
            #[allow(clippy::match_like_matches_macro)]
            match n {
                PolicyNode::Pk(_)
                | PolicyNode::Pkh(_)
                | PolicyNode::Multi(_)
                | PolicyNode::Sortedmulti(_)
                | PolicyNode::Older(_)
                | PolicyNode::After(_)
                | PolicyNode::Sha256(_)
                | PolicyNode::Hash256(_)
                | PolicyNode::Hash160(_)
                | PolicyNode::Ripemd160(_)
                | PolicyNode::AndV(_)
                | PolicyNode::OrD(_)
                | PolicyNode::OrI(_)
                | PolicyNode::OrB(_)
                | PolicyNode::Andor(_)
                | PolicyNode::Thresh(_)
                | PolicyNode::Wrap(_) => {}
            }
        }
        samples
    }

    /// Risk #2 cross-check (one link of the freeze): asserts the hand-maintained
    /// `all_variant_samples()` tag set == [`NODE_KINDS`]
    /// (`grammar_matches_node_kinds_hand_list` extends it to
    /// `schema::NODE_GRAMMAR`). `kind()` is exhaustive (compile-forced) so each
    /// sample's tag is well-defined. This catches a sample/`NODE_KINDS`/grammar
    /// *drift*; it does NOT catch a variant omitted from all three together (see
    /// `all_variant_samples` — that would need a variant-enumerator macro).
    #[test]
    fn node_kinds_cover_enum() {
        let produced: BTreeSet<&str> = all_variant_samples().iter().map(|n| n.kind()).collect();
        let expected: BTreeSet<&str> = NODE_KINDS.iter().copied().collect();
        assert_eq!(
            produced, expected,
            "every PolicyNode variant's kind() must appear in NODE_KINDS and vice-versa"
        );
        // One distinct sample per NODE_KINDS entry (no duplicate tags).
        assert_eq!(produced.len(), NODE_KINDS.len());
        assert_eq!(all_variant_samples().len(), NODE_KINDS.len());
    }

    // ---- render unit cells (exact, hand-verifiable) -----------------------

    fn doc(root: PolicyNode) -> SpecDoc {
        SpecDoc {
            schema_version: 1,
            wrapper: WrapperKind::Wsh,
            root,
        }
    }

    #[test]
    fn render_leaf_nodes_exact() {
        assert_eq!(PolicyNode::Pk("A".into()).render(), "pk(A/<0;1>/*)");
        assert_eq!(PolicyNode::Pkh("A".into()).render(), "pkh(A/<0;1>/*)");
        assert_eq!(PolicyNode::Older(144).render(), "older(144)");
        assert_eq!(PolicyNode::After(500000).render(), "after(500000)");
        assert_eq!(PolicyNode::Sha256("ab".into()).render(), "sha256(ab)");
        assert_eq!(PolicyNode::Hash256("ab".into()).render(), "hash256(ab)");
        assert_eq!(PolicyNode::Hash160("ab".into()).render(), "hash160(ab)");
        assert_eq!(PolicyNode::Ripemd160("ab".into()).render(), "ripemd160(ab)");
    }

    #[test]
    fn render_multi_applies_suffix_per_key() {
        let m = PolicyNode::Multi(MultiSpec {
            k: 2,
            keys: vec!["A".into(), "B".into()],
        });
        assert_eq!(m.render(), "multi(2,A/<0;1>/*,B/<0;1>/*)");
        let s = PolicyNode::Sortedmulti(MultiSpec {
            k: 1,
            keys: vec!["A".into()],
        });
        assert_eq!(s.render(), "sortedmulti(1,A/<0;1>/*)");
    }

    #[test]
    fn render_combinators_and_wrapper_exact() {
        let andv = PolicyNode::AndV(Box::new([
            PolicyNode::Wrap(WrapSpec {
                w: "v".into(),
                sub: Box::new(PolicyNode::Pk("A".into())),
            }),
            PolicyNode::Older(5),
        ]));
        assert_eq!(andv.render(), "and_v(v:pk(A/<0;1>/*),older(5))");

        let andor = PolicyNode::Andor(Box::new([
            PolicyNode::Pk("A".into()),
            PolicyNode::Sha256("ab".into()),
            PolicyNode::Older(7),
        ]));
        assert_eq!(andor.render(), "andor(pk(A/<0;1>/*),sha256(ab),older(7))");

        let thr = PolicyNode::Thresh(ThreshSpec {
            k: 2,
            subs: vec![
                PolicyNode::Pk("A".into()),
                PolicyNode::Wrap(WrapSpec {
                    w: "s".into(),
                    sub: Box::new(PolicyNode::Pk("B".into())),
                }),
            ],
        });
        assert_eq!(thr.render(), "thresh(2,pk(A/<0;1>/*),s:pk(B/<0;1>/*))");

        let orb = PolicyNode::OrB(Box::new([
            PolicyNode::Pk("A".into()),
            PolicyNode::Wrap(WrapSpec {
                w: "s".into(),
                sub: Box::new(PolicyNode::Pk("B".into())),
            }),
        ]));
        assert_eq!(orb.render(), "or_b(pk(A/<0;1>/*),s:pk(B/<0;1>/*))");
    }

    #[test]
    fn render_descriptor_wraps_in_wsh() {
        let d = doc(PolicyNode::Pk("A".into()));
        assert_eq!(d.render_descriptor(), "wsh(pk(A/<0;1>/*))");
    }

    // ---- serde / gate cells ----------------------------------------------

    #[test]
    fn parses_minimal_doc() {
        let j = r#"{"schema_version":1,"wrapper":"wsh","root":{"pk":"A"}}"#;
        let d = SpecDoc::parse(j).expect("valid");
        assert_eq!(d.render_descriptor(), "wsh(pk(A/<0;1>/*))");
    }

    #[test]
    fn rejects_unknown_top_level_field() {
        let j = r#"{"schema_version":1,"wrapper":"wsh","root":{"pk":"A"},"bogus":1}"#;
        assert!(matches!(SpecDoc::parse(j), Err(SpecParseError::Json(_))));
    }

    #[test]
    fn rejects_unknown_field_in_struct_payload() {
        // typo'd inner field on MultiSpec
        let j = r#"{"schema_version":1,"wrapper":"wsh","root":{"multi":{"k":2,"keys":["A","B"],"x":1}}}"#;
        assert!(matches!(SpecDoc::parse(j), Err(SpecParseError::Json(_))));
    }

    #[test]
    fn rejects_sibling_key_on_leaf_node() {
        // externally-tagged single-key rule: a 2-key node object is rejected.
        let j = r#"{"schema_version":1,"wrapper":"wsh","root":{"pk":"A","w":"v"}}"#;
        assert!(matches!(SpecDoc::parse(j), Err(SpecParseError::Json(_))));
    }

    #[test]
    fn rejects_version_mismatch() {
        let j = r#"{"schema_version":2,"wrapper":"wsh","root":{"pk":"A"}}"#;
        assert!(matches!(
            SpecDoc::parse(j),
            Err(SpecParseError::UnsupportedVersion(2))
        ));
    }

    #[test]
    fn rejects_wrong_arity_binary_combinator() {
        let one = r#"{"schema_version":1,"wrapper":"wsh","root":{"and_v":[{"pk":"A"}]}}"#;
        let three = r#"{"schema_version":1,"wrapper":"wsh","root":{"and_v":[{"pk":"A"},{"pk":"B"},{"pk":"C"}]}}"#;
        assert!(matches!(SpecDoc::parse(one), Err(SpecParseError::Json(_))));
        assert!(matches!(
            SpecDoc::parse(three),
            Err(SpecParseError::Json(_))
        ));
    }

    #[test]
    fn rejects_wrong_arity_andor() {
        let two =
            r#"{"schema_version":1,"wrapper":"wsh","root":{"andor":[{"pk":"A"},{"pk":"B"}]}}"#;
        assert!(matches!(SpecDoc::parse(two), Err(SpecParseError::Json(_))));
    }

    #[test]
    fn round_trips_through_serde() {
        let j = r#"{"schema_version":1,"wrapper":"wsh","root":{"or_d":[{"pk":"A"},{"and_v":[{"wrap":{"w":"v","sub":{"pkh":"B"}}},{"older":65535}]}]}}"#;
        let d = SpecDoc::parse(j).expect("valid");
        let reser = serde_json::to_string(&d).expect("serialize");
        let d2 = SpecDoc::parse(&reser).expect("reparse");
        assert_eq!(d, d2);
        assert_eq!(
            d.render_descriptor(),
            "wsh(or_d(pk(A/<0;1>/*),and_v(v:pkh(B/<0;1>/*),older(65535))))"
        );
    }
}
