//! Archetype preset producers (presets SPEC `design/SPEC_descriptor_builder_presets.md`
//! §2–§3, Release B / v0.51.0) — 5 thin producers that lower flag-supplied
//! parameters into the FROZEN Release-A `PolicyNode` IR and flow through the
//! same validation gate (`gate::validate`). No second validation path: the
//! producer validates ONLY applicability / arity / decay-ordering (§3.1);
//! everything else (k≤n, dup keys, hex, timelocks, xprv screen, type, sanity,
//! cap) is the gate's (§3.2).
//!
//! Canon: each producer, fed its fixture's own parameter values, must
//! reproduce `tests/fixtures/descriptor_builder/<id>.json`'s `root` AST
//! exactly (layer-1 equivalence tests below).

use super::gate::{Diagnostic, DiagnosticKind};
use super::ir::{MultiSpec, PolicyNode, ThreshSpec, WrapSpec};

/// Flat clap-collected preset parameters (one struct for all archetypes).
#[derive(Debug, Clone, Default)]
pub struct ArchetypeParams {
    pub keys: Vec<String>,
    pub threshold: Option<u32>,
    pub recovery_keys: Vec<String>,
    pub recovery_threshold: Option<u32>,
    pub final_key: Option<String>,
    pub older: Option<u32>,
    pub recovery_older: Option<u32>,
    pub after: Option<u32>,
    pub hash: Option<String>,
}

/// Parameter-kind METADATA for the `--spec-schema` archetypes section + the
/// manual — no producer check keys off it (hex/timelock validation is the
/// gate's, SPEC §3.2). `AbsoluteLocktime` is deliberately locktime-neutral:
/// the decaying-multisig canon uses a block HEIGHT `after(4000000)`, not a
/// unix time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParamKind {
    Key,
    Threshold,
    Blocks,
    AbsoluteLocktime,
    HexDigest,
}

impl ParamKind {
    /// Stable snake_case wire string for the `--spec-schema` `archetypes`
    /// section (presets SPEC §5).
    pub fn as_str(self) -> &'static str {
        match self {
            ParamKind::Key => "key",
            ParamKind::Threshold => "threshold",
            ParamKind::Blocks => "blocks",
            ParamKind::AbsoluteLocktime => "absolute_locktime",
            ParamKind::HexDigest => "hex_digest",
        }
    }
}

/// One param's declaration — drives the generic applicability/arity
/// validation in [`validate_params`] AND the `--spec-schema` archetypes
/// section. `flag` is the literal clap long name (with leading `--`).
#[derive(Debug, Clone, Copy)]
pub struct ParamSpec {
    pub flag: &'static str,
    pub required: bool,
    pub repeatable: bool,
    /// 1 for scalars; ≥2 where a quorum needs it. Only meaningful when
    /// `repeatable`.
    pub min_count: usize,
    pub kind: ParamKind,
}

/// One archetype's registry entry. Adding archetype #6 = one additive entry
/// (engine-SPEC §5 seam 5; static table + fn-pointer lowering, no `dyn`).
pub struct ArchetypeDef {
    /// == the `CliArchetype` kebab name (drift self-test in `cmd`).
    pub id: &'static str,
    /// One-line human description (`--spec-schema` archetypes section).
    pub summary: &'static str,
    pub params: &'static [ParamSpec],
    /// Kind-aware provenance for gate-diagnostic annotation (presets SPEC
    /// §3.3, Phase 2): (node_path prefix, kind override, flag). Resolution =
    /// longest prefix; at equal prefix a `Some(kind)`-matching entry beats a
    /// `None` (catch-all) entry. Consumed by [`resolve_flag`].
    pub provenance: &'static [(&'static str, Option<DiagnosticKind>, &'static str)],
    /// Lower validated params to the IR. Infallible BY CONVENTION: callers
    /// run [`validate_params`] first; unwraps inside are `expect`-annotated
    /// against the registry declaration (presets SPEC §2, R0-r1 M1).
    pub lower: fn(&ArchetypeParams) -> PolicyNode,
}

const KEY: &str = "--key";
const THRESHOLD: &str = "--threshold";
const RECOVERY_KEY: &str = "--recovery-key";
const RECOVERY_THRESHOLD: &str = "--recovery-threshold";
const FINAL_KEY: &str = "--final-key";
const OLDER: &str = "--older";
const RECOVERY_OLDER: &str = "--recovery-older";
const AFTER: &str = "--after";
const HASH: &str = "--hash";

const fn p(
    flag: &'static str,
    required: bool,
    repeatable: bool,
    min_count: usize,
    kind: ParamKind,
) -> ParamSpec {
    ParamSpec {
        flag,
        required,
        repeatable,
        min_count,
        kind,
    }
}

/// The 5 canonical archetypes, alphabetical by id (matches the `CliArchetype`
/// declaration order — presets SPEC §2.1).
pub const ARCHETYPE_REGISTRY: &[ArchetypeDef] = &[
    ArchetypeDef {
        id: "decaying-multisig",
        summary: "k-of-n multisig that decays to a smaller recovery quorum and \
                  finally a single key as timelocks expire",
        params: &[
            p(KEY, true, true, 2, ParamKind::Key),
            p(THRESHOLD, true, false, 1, ParamKind::Threshold),
            p(OLDER, true, false, 1, ParamKind::Blocks),
            p(RECOVERY_KEY, true, true, 2, ParamKind::Key),
            p(RECOVERY_THRESHOLD, true, false, 1, ParamKind::Threshold),
            p(RECOVERY_OLDER, true, false, 1, ParamKind::Blocks),
            p(FINAL_KEY, true, false, 1, ParamKind::Key),
            p(AFTER, true, false, 1, ParamKind::AbsoluteLocktime),
        ],
        provenance: &[
            (
                "root.andor[0]",
                Some(DiagnosticKind::SchemaField),
                THRESHOLD,
            ),
            ("root.andor[0]", None, KEY),
            ("root.andor[1]", None, OLDER),
            (
                "root.andor[2].andor[0]",
                Some(DiagnosticKind::SchemaField),
                RECOVERY_THRESHOLD,
            ),
            ("root.andor[2].andor[0]", None, RECOVERY_KEY),
            ("root.andor[2].andor[1]", None, RECOVERY_OLDER),
            ("root.andor[2].andor[2].and_v[0]", None, FINAL_KEY),
            ("root.andor[2].andor[2].and_v[1]", None, AFTER),
        ],
        lower: lower_decaying_multisig,
    },
    ArchetypeDef {
        id: "hashlock-gated",
        summary: "primary key spends with a SHA-256 preimage; a recovery key \
                  takes over after a relative timelock",
        params: &[
            p(KEY, true, false, 1, ParamKind::Key),
            p(HASH, true, false, 1, ParamKind::HexDigest),
            p(RECOVERY_KEY, true, false, 1, ParamKind::Key),
            p(OLDER, true, false, 1, ParamKind::Blocks),
        ],
        provenance: &[
            ("root.andor[0]", None, KEY),
            ("root.andor[1]", None, HASH),
            ("root.andor[2].and_v[0]", None, RECOVERY_KEY),
            ("root.andor[2].and_v[1]", None, OLDER),
        ],
        lower: lower_hashlock_gated,
    },
    ArchetypeDef {
        id: "kofn-recovery",
        summary: "k-of-n multisig with a single timelocked recovery key",
        params: &[
            p(KEY, true, true, 2, ParamKind::Key),
            p(THRESHOLD, true, false, 1, ParamKind::Threshold),
            p(RECOVERY_KEY, true, false, 1, ParamKind::Key),
            p(OLDER, true, false, 1, ParamKind::Blocks),
        ],
        provenance: &[
            ("root.or_d[0]", Some(DiagnosticKind::SchemaField), THRESHOLD),
            ("root.or_d[0]", None, KEY),
            ("root.or_d[1].and_v[0]", None, RECOVERY_KEY),
            ("root.or_d[1].and_v[1]", None, OLDER),
        ],
        lower: lower_kofn_recovery,
    },
    ArchetypeDef {
        id: "simple-timelocked-inheritance",
        summary: "owner key spends anytime; an heir key unlocks after a \
                  relative timelock",
        params: &[
            p(KEY, true, false, 1, ParamKind::Key),
            p(RECOVERY_KEY, true, false, 1, ParamKind::Key),
            p(OLDER, true, false, 1, ParamKind::Blocks),
        ],
        provenance: &[
            ("root.or_d[0]", None, KEY),
            ("root.or_d[1].and_v[0]", None, RECOVERY_KEY),
            ("root.or_d[1].and_v[1]", None, OLDER),
        ],
        lower: lower_simple_timelocked_inheritance,
    },
    ArchetypeDef {
        id: "tiered-recovery",
        summary: "primary sorted multisig OR a timelocked recovery threshold \
                  of distinct keys",
        params: &[
            p(KEY, true, true, 2, ParamKind::Key),
            p(THRESHOLD, true, false, 1, ParamKind::Threshold),
            p(RECOVERY_KEY, true, true, 2, ParamKind::Key),
            p(RECOVERY_THRESHOLD, true, false, 1, ParamKind::Threshold),
            p(OLDER, true, false, 1, ParamKind::Blocks),
        ],
        provenance: &[
            ("root.or_i[0]", Some(DiagnosticKind::SchemaField), THRESHOLD),
            ("root.or_i[0]", None, KEY),
            ("root.or_i[1].and_v[0]", None, OLDER),
            (
                "root.or_i[1].and_v[1]",
                Some(DiagnosticKind::SchemaField),
                RECOVERY_THRESHOLD,
            ),
            ("root.or_i[1].and_v[1]", None, RECOVERY_KEY),
        ],
        lower: lower_tiered_recovery,
    },
];

/// Look up a registry entry by id. The `CliArchetype` ↔ registry id mapping
/// is pinned by a drift self-test in `cmd::build_descriptor`.
pub fn registry_get(id: &str) -> &'static ArchetypeDef {
    ARCHETYPE_REGISTRY
        .iter()
        .find(|d| d.id == id)
        .expect("CliArchetype variant present in ARCHETYPE_REGISTRY (drift self-test pins this)")
}

/// Producer-level validation — ONLY what the gate cannot know (presets SPEC
/// §3.1): applicability, presence/arity, and the decaying-multisig
/// decay-ordering rule. Gate rules (k≤n, dup keys, hex, timelocks, xprv) are
/// deliberately NOT checked here (§3.2 — no second validation path).
pub fn validate_params(
    def: &ArchetypeDef,
    params: &ArchetypeParams,
) -> Result<(), Vec<Diagnostic>> {
    let mut diags = Vec::new();
    let supplied: &[(&str, usize)] = &[
        (KEY, params.keys.len()),
        (THRESHOLD, params.threshold.iter().count()),
        (RECOVERY_KEY, params.recovery_keys.len()),
        (RECOVERY_THRESHOLD, params.recovery_threshold.iter().count()),
        (FINAL_KEY, params.final_key.iter().count()),
        (OLDER, params.older.iter().count()),
        (RECOVERY_OLDER, params.recovery_older.iter().count()),
        (AFTER, params.after.iter().count()),
        (HASH, params.hash.iter().count()),
    ];

    // Applicability: supplied but not declared for this archetype.
    for &(flag, count) in supplied {
        if count > 0 && !def.params.iter().any(|s| s.flag == flag) {
            diags.push(param_diag(
                flag,
                format!("{flag} is not a parameter of {}", def.id),
            ));
        }
    }

    // Presence / arity against each declaration.
    for spec in def.params {
        let count = supplied
            .iter()
            .find(|(flag, _)| *flag == spec.flag)
            .map(|&(_, c)| c)
            .expect("every ParamSpec.flag is one of the 9 ArchetypeParams fields");
        if count == 0 {
            if spec.required {
                diags.push(param_diag(
                    spec.flag,
                    format!("{} requires {} (missing)", def.id, spec.flag),
                ));
            }
        } else if !spec.repeatable && count > 1 {
            diags.push(param_diag(
                spec.flag,
                format!("{} takes exactly one {} (got {count})", def.id, spec.flag),
            ));
        } else if spec.repeatable && count < spec.min_count {
            diags.push(param_diag(
                spec.flag,
                format!(
                    "{} requires at least {} {} values (got {count})",
                    def.id, spec.min_count, spec.flag
                ),
            ));
        }
    }

    // Decay ordering (decaying-multisig only): tiers must unlock progressively
    // later. Both values are individually gate-valid and an inverted tree is
    // sane, yet inversion silently defeats the archetype (SPEC §3.1.3); a user
    // who genuinely wants inverted timelocks has `--spec`.
    if def.id == "decaying-multisig" {
        // D-decay-rel (cycle-6): the tier-1→tier-2 relative-timelock ordering
        // must be checked in a COMMON BIP-68 unit. `older_unit_value` classifies
        // each clean operand's unit (bit-22) + low-16 value; a masked operand is
        // refused independently downstream by the gate (`gate.rs` Older arm), so
        // no cleanliness precondition is needed here — we compare unit+value only.
        if let (Some(older), Some(recovery_older)) = (params.older, params.recovery_older) {
            let (u_p, v_p) = crate::timelock_advisory::older_unit_value(older);
            let (u_r, v_r) = crate::timelock_advisory::older_unit_value(recovery_older);
            if u_p != u_r {
                // CROSS-UNIT — a block delay and a 512-second delay cannot be
                // totally ordered without baking in a block-interval assumption.
                // Refuse fail-closed (R1); `--spec` is the escape hatch.
                diags.push(param_diag(
                    RECOVERY_OLDER,
                    format!(
                        "decaying-multisig --older ({older}) and --recovery-older \
                         ({recovery_older}) use different BIP-68 timelock units \
                         (one block-height, one 512-second) and cannot be ordered; \
                         express both in the same unit, or author the policy with --spec"
                    ),
                ));
            } else if v_r <= v_p {
                // SAME-UNIT but not strictly later — the unit-aware generalization
                // of the former raw `recovery_older <= older` compare (keeps the
                // 2000/2000-blocks negative test GREEN: v_r == v_p ⇒ refused).
                diags.push(param_diag(
                    RECOVERY_OLDER,
                    format!(
                        "decaying-multisig requires --recovery-older ({recovery_older}) > \
                         --older ({older}): tiers must unlock progressively later"
                    ),
                ));
            }
        }

        // D-decay-abs (cycle-6): tier-3's absolute `after(T)` must be FUTURE.
        // `after` is absolute (a wall-clock/height moment) and the `older` tiers
        // are relative (a delay per UTXO), so they live in different reference
        // frames and cannot be totally ordered offline — the decidable invariant
        // is future-ness. Classify via the BIP-65 500_000_000 height/time split,
        // refuse fail-closed below the conservative static past-floors. STRICT `<`
        // (monotone-safe: only false-NEGATIVE on a borderline-recent locktime,
        // never false-POSITIVE on a legit future one). Future values are silent.
        if let Some(after) = params.after {
            let is_height = after < 500_000_000;
            let past = if is_height {
                after < crate::timelock_advisory::ABS_HEIGHT_PAST_FLOOR
            } else {
                after < crate::timelock_advisory::ABS_TIME_PAST_FLOOR
            };
            if past {
                diags.push(param_diag(
                    AFTER,
                    format!(
                        "decaying-multisig --after ({after}) encodes an absolute locktime \
                         that is already in the past ({}), so the final-key tier would be \
                         spendable immediately and the decay ladder collapses; use a future \
                         {} value, or author the policy with --spec",
                        if is_height { "block height" } else { "Unix time" },
                        if is_height { "block height" } else { "Unix timestamp" },
                    ),
                ));
            }
        }
    }

    if diags.is_empty() {
        Ok(())
    } else {
        Err(diags)
    }
}

fn param_diag(flag: &str, message: String) -> Diagnostic {
    Diagnostic {
        node_path: "params".to_string(),
        kind: DiagnosticKind::Param,
        message,
        flag: Some(flag.to_string()),
    }
}

/// Resolve a gate diagnostic's `(node_path, kind)` to the clap flag it traces
/// back to, via the archetype's provenance table (presets SPEC §3.3).
/// Resolution = longest matching prefix (on a path-segment boundary); at equal
/// prefix length a kind-specific entry (`Some(kind)` == the diagnostic's kind)
/// beats the catch-all (`None`). Entries with a NON-matching `Some(kind)` are
/// ignored. No match ⇒ `None` ⇒ the `flag` key stays absent (contractual —
/// e.g. a cross-branch duplicate key localized to a combinator node).
pub fn resolve_flag(
    def: &ArchetypeDef,
    node_path: &str,
    kind: DiagnosticKind,
) -> Option<&'static str> {
    def.provenance
        .iter()
        .filter(|(prefix, k, _)| {
            let boundary_ok = node_path == *prefix
                || node_path
                    .strip_prefix(prefix)
                    .is_some_and(|rest| rest.starts_with('.') || rest.starts_with('['));
            boundary_ok && k.map_or(true, |k| k == kind)
        })
        .max_by_key(|(prefix, k, _)| (prefix.len(), k.is_some()))
        .map(|&(_, _, flag)| flag)
}

// ======================================================================
// Lowering helpers — structural shapes are the Release-A fixture canon
// (presets SPEC §6); key argv order is preserved untouched.
// ======================================================================

fn wrap(w: &str, sub: PolicyNode) -> PolicyNode {
    PolicyNode::Wrap(WrapSpec {
        w: w.to_string(),
        sub: Box::new(sub),
    })
}

fn v_pk(key: &str) -> PolicyNode {
    wrap("v", PolicyNode::Pk(key.to_string()))
}

fn and_v(a: PolicyNode, b: PolicyNode) -> PolicyNode {
    PolicyNode::AndV(Box::new([a, b]))
}

fn one_key<'a>(params: &'a ArchetypeParams, id: &str) -> &'a str {
    params
        .keys
        .first()
        .unwrap_or_else(|| panic!("--key declared required in ARCHETYPE_REGISTRY for {id}"))
}

fn one_recovery_key<'a>(params: &'a ArchetypeParams, id: &str) -> &'a str {
    params.recovery_keys.first().unwrap_or_else(|| {
        panic!("--recovery-key declared required in ARCHETYPE_REGISTRY for {id}")
    })
}

/// `andor(multi(k1,T1…), older(N1), andor(multi(k2,T2…), older(N2),
/// and_v(v:pk(F), after(T))))` — presets SPEC §6.
fn lower_decaying_multisig(params: &ArchetypeParams) -> PolicyNode {
    let id = "decaying-multisig";
    let tier3 = and_v(
        v_pk(params.final_key.as_deref().unwrap_or_else(|| {
            panic!("--final-key declared required in ARCHETYPE_REGISTRY for {id}")
        })),
        PolicyNode::After(req(params.after, "--after", id)),
    );
    let tier2 = PolicyNode::Andor(Box::new([
        PolicyNode::Multi(MultiSpec {
            k: req(params.recovery_threshold, "--recovery-threshold", id),
            keys: params.recovery_keys.clone(),
        }),
        PolicyNode::Older(req(params.recovery_older, "--recovery-older", id)),
        tier3,
    ]));
    PolicyNode::Andor(Box::new([
        PolicyNode::Multi(MultiSpec {
            k: req(params.threshold, "--threshold", id),
            keys: params.keys.clone(),
        }),
        PolicyNode::Older(req(params.older, "--older", id)),
        tier2,
    ]))
}

/// `andor(pk(A), sha256(H), and_v(v:pk(B), older(N)))` — presets SPEC §6.
fn lower_hashlock_gated(params: &ArchetypeParams) -> PolicyNode {
    let id = "hashlock-gated";
    PolicyNode::Andor(Box::new([
        PolicyNode::Pk(one_key(params, id).to_string()),
        PolicyNode::Sha256(
            params.hash.clone().unwrap_or_else(|| {
                panic!("--hash declared required in ARCHETYPE_REGISTRY for {id}")
            }),
        ),
        and_v(
            v_pk(one_recovery_key(params, id)),
            PolicyNode::Older(req(params.older, "--older", id)),
        ),
    ]))
}

/// `or_d(multi(k,K…), and_v(v:pk(R), older(N)))` — presets SPEC §6.
fn lower_kofn_recovery(params: &ArchetypeParams) -> PolicyNode {
    let id = "kofn-recovery";
    PolicyNode::OrD(Box::new([
        PolicyNode::Multi(MultiSpec {
            k: req(params.threshold, "--threshold", id),
            keys: params.keys.clone(),
        }),
        and_v(
            v_pk(one_recovery_key(params, id)),
            PolicyNode::Older(req(params.older, "--older", id)),
        ),
    ]))
}

/// `or_d(pk(P), and_v(v:pkh(H), older(N)))` — presets SPEC §6 (heir is `pkh`
/// under `v:` — the fixture canon).
fn lower_simple_timelocked_inheritance(params: &ArchetypeParams) -> PolicyNode {
    let id = "simple-timelocked-inheritance";
    PolicyNode::OrD(Box::new([
        PolicyNode::Pk(one_key(params, id).to_string()),
        and_v(
            wrap(
                "v",
                PolicyNode::Pkh(one_recovery_key(params, id).to_string()),
            ),
            PolicyNode::Older(req(params.older, "--older", id)),
        ),
    ]))
}

/// `or_i(sortedmulti(k1,P…), and_v(v:older(N), thresh(k2, pk, s:pk…)))` —
/// presets SPEC §6 (`s:` wraps on thresh subs 2..n — the fixture canon).
fn lower_tiered_recovery(params: &ArchetypeParams) -> PolicyNode {
    let id = "tiered-recovery";
    let subs: Vec<PolicyNode> = params
        .recovery_keys
        .iter()
        .enumerate()
        .map(|(i, k)| {
            let pk = PolicyNode::Pk(k.clone());
            if i == 0 {
                pk
            } else {
                wrap("s", pk)
            }
        })
        .collect();
    PolicyNode::OrI(Box::new([
        PolicyNode::Sortedmulti(MultiSpec {
            k: req(params.threshold, "--threshold", id),
            keys: params.keys.clone(),
        }),
        and_v(
            wrap("v", PolicyNode::Older(req(params.older, "--older", id))),
            PolicyNode::Thresh(ThreshSpec {
                k: req(params.recovery_threshold, "--recovery-threshold", id),
                subs,
            }),
        ),
    ]))
}

fn req(v: Option<u32>, flag: &str, id: &str) -> u32 {
    v.unwrap_or_else(|| panic!("{flag} declared required in ARCHETYPE_REGISTRY for {id}"))
}

// ======================================================================
// Layer-1 tests — producer-vs-fixture AST equivalence (the keystone) +
// validate_params unit cells. The fixtures are the IMMUTABLE canon.
// ======================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::descriptor_builder::ir::SpecDoc;

    fn fixture_root(json: &str) -> PolicyNode {
        SpecDoc::parse(json).expect("fixture parses").root
    }

    // The five fixtures' own parameter values (transcribed from
    // tests/fixtures/descriptor_builder/*.json — presets SPEC §6).
    const K1: &str = "[11111111/48h/0h/0h/2h]xpub661MyMwAqRbcEZVB4dScxMAdx6d4nFc9nvyvH3v4gJL378CSRZiYmhRoP7mBy6gSPSCYk6SzXPTf3ND1cZAceL7SfJ1Z3GC8vBgp2epUt13";
    const K2: &str = "[22222222/48h/0h/0h/2h]xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8";
    const K3: &str = "[33333333/48h/0h/0h/2h]xpub661MyMwAqRbcFW31YEwpkMuc5THy2PSt5bDMsktWQcFF8syAmRUapSCGu8ED9W6oDMSgv6Zz8idoc4a6mr8BDzTJY47LJhkJ8UB7WEGuduB";
    const K4: &str = "[44444444/48h/0h/0h/2h]xpub661MyMwAqRbcGczjuMoRm6dXaLDEhW1u34gKenbeYqAix21mdUKJyuyu5F1rzYGVxyL6tmgBUAEPrEz92mBXjByMRiJdba9wpnN37RLLAXa";
    const K5: &str = "[55555555/48h/0h/0h/2h]xpub68Gmy5EdvgibQVfPdqkBBCHxA5htiqg55crXYuXoQRKfDBFA1WEjWgP6LHhwBZeNK1VTsfTFUHCdrfp1bgwQ9xv5ski8PX9rL2dZXvgGDnw";
    const HASH_HEX: &str = "926a54995ca48600920a19bf7bc502ca5f2f7d07e6f804c4f00ebf0325084dbc";

    fn keys(ks: &[&str]) -> Vec<String> {
        ks.iter().map(|k| k.to_string()).collect()
    }

    fn fixture_params(id: &str) -> ArchetypeParams {
        match id {
            "decaying-multisig" => ArchetypeParams {
                keys: keys(&[K1, K2]),
                threshold: Some(2),
                older: Some(1000),
                recovery_keys: keys(&[K3, K4]),
                recovery_threshold: Some(2),
                recovery_older: Some(2000),
                final_key: Some(K5.to_string()),
                after: Some(4000000),
                ..Default::default()
            },
            "hashlock-gated" => ArchetypeParams {
                keys: keys(&[K1]),
                hash: Some(HASH_HEX.to_string()),
                recovery_keys: keys(&[K2]),
                older: Some(144),
                ..Default::default()
            },
            "kofn-recovery" => ArchetypeParams {
                keys: keys(&[K1, K2, K3]),
                threshold: Some(2),
                recovery_keys: keys(&[K4]),
                older: Some(52560),
                ..Default::default()
            },
            "simple-timelocked-inheritance" => ArchetypeParams {
                keys: keys(&[K1]),
                recovery_keys: keys(&[K2]),
                older: Some(65535),
                ..Default::default()
            },
            "tiered-recovery" => ArchetypeParams {
                keys: keys(&[K1, K2]),
                threshold: Some(2),
                recovery_keys: keys(&[K3, K4, K5]),
                recovery_threshold: Some(2),
                older: Some(4032),
                ..Default::default()
            },
            other => panic!("unknown archetype id {other}"),
        }
    }

    fn fixture_json(id: &str) -> &'static str {
        match id {
            "decaying-multisig" => {
                include_str!("../../tests/fixtures/descriptor_builder/decaying-multisig.json")
            }
            "hashlock-gated" => {
                include_str!("../../tests/fixtures/descriptor_builder/hashlock-gated.json")
            }
            "kofn-recovery" => {
                include_str!("../../tests/fixtures/descriptor_builder/kofn-recovery.json")
            }
            "simple-timelocked-inheritance" => include_str!(
                "../../tests/fixtures/descriptor_builder/simple-timelocked-inheritance.json"
            ),
            "tiered-recovery" => {
                include_str!("../../tests/fixtures/descriptor_builder/tiered-recovery.json")
            }
            other => panic!("unknown archetype id {other}"),
        }
    }

    /// Layer 1 (the keystone): each producer, fed its fixture's own values,
    /// reproduces the Release-A fixture AST exactly — pinned against the
    /// fixture JSON, never against captured producer output (SPEC §7/§11.2).
    #[test]
    fn producers_reproduce_fixture_asts() {
        for def in ARCHETYPE_REGISTRY {
            let params = fixture_params(def.id);
            validate_params(def, &params)
                .unwrap_or_else(|d| panic!("{}: fixture params must validate: {d:?}", def.id));
            let lowered = (def.lower)(&params);
            let canon = fixture_root(fixture_json(def.id));
            assert_eq!(lowered, canon, "{}: producer AST != fixture canon", def.id);
        }
    }

    /// Registry integrity: alphabetical unique ids; every provenance flag and
    /// every param flag is declared/known (reads `provenance` so the Phase-2
    /// table is integrity-pinned from day one).
    #[test]
    fn registry_table_integrity() {
        let ids: Vec<&str> = ARCHETYPE_REGISTRY.iter().map(|d| d.id).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(ids, sorted, "registry must be alphabetical by id, unique");
        for def in ARCHETYPE_REGISTRY {
            for (prefix, _kind, flag) in def.provenance {
                assert!(
                    def.params.iter().any(|s| s.flag == *flag),
                    "{}: provenance flag {flag} not in params",
                    def.id
                );
                assert!(
                    prefix.starts_with("root"),
                    "{}: provenance prefix {prefix}",
                    def.id
                );
            }
            for spec in def.params {
                assert!(
                    spec.flag.starts_with("--"),
                    "{}: flag {}",
                    def.id,
                    spec.flag
                );
                if !spec.repeatable {
                    assert_eq!(spec.min_count, 1, "{}: scalar min_count", def.id);
                }
            }
        }
    }

    #[test]
    fn validate_params_missing_required() {
        let def = registry_get("kofn-recovery");
        let mut params = fixture_params("kofn-recovery");
        params.recovery_keys.clear();
        params.older = None;
        let diags = validate_params(def, &params).unwrap_err();
        assert_eq!(diags.len(), 2);
        assert!(diags.iter().all(|d| d.kind == DiagnosticKind::Param));
        assert!(diags.iter().all(|d| d.node_path == "params"));
        assert!(diags.iter().any(|d| d.message.contains("--recovery-key")));
        assert!(diags.iter().any(|d| d.message.contains("--older")));
    }

    #[test]
    fn validate_params_inapplicable_flag() {
        let def = registry_get("kofn-recovery");
        let mut params = fixture_params("kofn-recovery");
        params.hash = Some(HASH_HEX.to_string());
        let diags = validate_params(def, &params).unwrap_err();
        assert_eq!(diags.len(), 1);
        assert_eq!(diags[0].kind, DiagnosticKind::Param);
        assert!(diags[0].message.contains("--hash"));
        assert!(diags[0].message.contains("kofn-recovery"));
    }

    #[test]
    fn validate_params_under_min_count() {
        let def = registry_get("kofn-recovery");
        let mut params = fixture_params("kofn-recovery");
        params.keys = keys(&[K1]);
        let diags = validate_params(def, &params).unwrap_err();
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("at least 2 --key"));
    }

    /// Kind-aware provenance resolution (presets SPEC §3.3): longest prefix
    /// on a segment boundary; kind-specific beats catch-all at equal prefix;
    /// non-matching `Some(kind)` entries are ignored; no match ⇒ `None`.
    #[test]
    fn resolve_flag_kind_aware_longest_prefix() {
        let def = registry_get("kofn-recovery");
        // SchemaField at the quorum node → the kind-override entry.
        assert_eq!(
            resolve_flag(def, "root.or_d[0]", DiagnosticKind::SchemaField),
            Some("--threshold")
        );
        // Any other kind at the same path → the catch-all.
        assert_eq!(
            resolve_flag(def, "root.or_d[0]", DiagnosticKind::RepeatedKeys),
            Some("--key")
        );
        // keys[i] path resolves via PREFIX semantics (P1-r1 M2).
        assert_eq!(
            resolve_flag(def, "root.or_d[0].multi.keys[1]", DiagnosticKind::SecretKey),
            Some("--key")
        );
        // Longest prefix wins: the recovery and_v arm, deeper path.
        assert_eq!(
            resolve_flag(
                def,
                "root.or_d[1].and_v[0].wrap.sub",
                DiagnosticKind::SecretKey
            ),
            Some("--recovery-key")
        );
        // No entry matches root (cross-branch dup) → None.
        assert_eq!(
            resolve_flag(def, "root", DiagnosticKind::RepeatedKeys),
            None
        );
        // Decaying intra-andor[2] cross-tier dup matches no entry (P1-r1 M3).
        let decaying = registry_get("decaying-multisig");
        assert_eq!(
            resolve_flag(decaying, "root.andor[2]", DiagnosticKind::RepeatedKeys),
            None
        );
    }

    #[test]
    fn validate_params_non_repeatable_repeated() {
        // Vec-typed flag on an exactly-one archetype (SPEC §7, R0-r1 M2).
        // (Scalar Option<_> flags repeated are rejected by clap itself as a
        // usage error and never reach validate_params — R0-r1 M2 / P1-r1 M4.)
        let def = registry_get("simple-timelocked-inheritance");
        let mut params = fixture_params("simple-timelocked-inheritance");
        params.keys = keys(&[K1, K3]);
        let diags = validate_params(def, &params).unwrap_err();
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("exactly one --key"));
    }

    #[test]
    fn validate_params_decay_ordering() {
        let def = registry_get("decaying-multisig");
        for bad_recovery_older in [1000, 999] {
            let mut params = fixture_params("decaying-multisig");
            params.recovery_older = Some(bad_recovery_older);
            let diags = validate_params(def, &params).unwrap_err();
            assert_eq!(diags.len(), 1);
            assert_eq!(diags[0].kind, DiagnosticKind::Param);
            assert!(diags[0].message.contains("--recovery-older"));
            assert!(diags[0].message.contains("--older"));
        }
    }

    /// The gate rules are NOT duplicated at the producer (SPEC §3.2): k>n and
    /// duplicate keys pass `validate_params` (the gate refuses them later).
    #[test]
    fn validate_params_does_not_duplicate_gate_rules() {
        let def = registry_get("kofn-recovery");
        let mut params = fixture_params("kofn-recovery");
        params.threshold = Some(5); // k > n — gate's SchemaField, not ours
        validate_params(def, &params).expect("k>n flows to the gate");
        let mut params = fixture_params("kofn-recovery");
        params.keys = keys(&[K1, K1, K2]); // dup key — gate's RepeatedKeys
        validate_params(def, &params).expect("dup keys flow to the gate");
    }
}
