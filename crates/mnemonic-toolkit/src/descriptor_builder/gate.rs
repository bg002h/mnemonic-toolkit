//! `descriptor-builder` validation gate — the funds-safety core (SPEC §3 + §3.4).
//!
//! A 4-step, fail-closed gate that turns a parsed [`SpecDoc`] into a validated
//! `wsh(M)` descriptor or a list of **node-addressed** diagnostics:
//!   1. schema field-validate (k≤n, hashlock hex, timelock ranges) — node path
//!      from the IR walk;
//!   2. type-check: render → `wsh(M)` → `Descriptor::from_str` (type-check ONLY;
//!      F1: `from_str` is LENIENT on the funds-footgun rules);
//!   3. `Miniscript::sanity_check()` (the SOLE funds-footgun gate) + §3.4
//!      per-subtree localization (NOT read off `sanity_check`);
//!   4. build-time complexity cap (the "always-previewable envelope").
//!
//! The former step-4 per-branch `plan()` satisfiability check was CUT: it is
//! tautological for any tree passing steps 2+3 (`AnalysisError` defines an
//! unspendable path as exactly resource-limits + timelock-mixing, both
//! `sanity_check` rules; empirically `plan(&maximal_assets)` is `Ok` for every
//! sane tree). See SPEC §3 "(CUT) former step 4".

use std::collections::BTreeSet;
use std::str::FromStr;

use miniscript::descriptor::DescriptorPublicKey;
use miniscript::miniscript::analyzable::{AnalysisError, ExtParams};
use miniscript::miniscript::decode::Terminal;
use miniscript::{Descriptor as MsDescriptor, ForEachKey, Miniscript, Segwitv0};
use serde::Serialize;

use super::ir::{PolicyNode, SpecDoc};

/// Default complexity cap — matches `compare-cost`'s default `--max-conditions`
/// (`cmd/compare_cost.rs`) so a policy that passes this gate also renders in the
/// Phase-3 cost preview without tripping `ConditionsTooMany`.
pub const DEFAULT_PREVIEW_CAP: usize = 4096;

/// The validated output of the gate — what Phase-3 emit consumes.
pub struct ValidatedPolicy {
    /// Parsed `wsh(M)` (multipath). Canonicalize (+ BIP-380 checksum) via
    /// `.to_string()` at emit.
    pub descriptor: MsDescriptor<DescriptorPublicKey>,
    /// Sanity rules that were `--allow`ed AND actually fired (allow SPEC §2).
    /// Empty when no allowance was requested or none was needed. Reuses the
    /// step-3 [`DiagnosticKind`]s 1:1; populated in `ext_check`'s check order.
    pub allowed_fired: Vec<DiagnosticKind>,
}

/// The reviewed sanity opt-outs (allow SPEC §1-§2) — gate-local so gate.rs
/// does not depend on the clap enum. Maps onto miniscript's `ExtParams`;
/// `raw_pkh` is deliberately not exposed (unreachable from IR-rendered
/// miniscript — the IR has no raw-pkh node and `render()` cannot emit the
/// `expr_raw_pkh` fragment).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AllowSet {
    pub sigless_branch: bool,
    pub malleable: bool,
    pub resource_limit: bool,
    pub repeated_keys: bool,
    pub mixed_timelock: bool,
}

impl AllowSet {
    fn to_ext_params(self) -> ExtParams {
        let mut p = ExtParams::new();
        p.top_unsafe = self.sigless_branch;
        p.malleability = self.malleable;
        p.resource_limitations = self.resource_limit;
        p.repeated_pk = self.repeated_keys;
        p.timelock_mixing = self.mixed_timelock;
        p
    }
}

/// A node-addressed structured diagnostic. `node_path` is a dotted/bracketed
/// path into the JSON tree (e.g. `root.or_d[1].and_v[0]`).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Diagnostic {
    pub node_path: String,
    pub kind: DiagnosticKind,
    pub message: String,
    /// Preset-mode param provenance (presets SPEC §3.3): the clap flag this
    /// diagnostic traces back to, resolved from the archetype's provenance
    /// table. `None` in spec mode — and skipped on the wire, so spec-mode
    /// `--json` output is byte-identical to pre-Release-B.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flag: Option<String>,
}

/// The kind of gate failure. `as_str` is the stable `--json` discriminant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticKind {
    /// Step 0 — a preset parameter error from the producer layer
    /// (`descriptor_builder::archetype::validate_params`): inapplicable flag,
    /// missing/under-count param, or a decay-ordering violation. `node_path`
    /// is the sentinel `"params"`; the offending flag(s) are named in
    /// `message`. (Placed first: this enum orders by gate step, and producer
    /// checks run before step 1 — presets SPEC §3.3.)
    Param,
    /// Step 1 — bad threshold / hex / timelock field.
    SchemaField,
    /// Step 2 — miniscript type error (e.g. missing `v:` wrapper).
    TypeError,
    /// Step 3 — an anyone-can-spend path (`SiglessBranch`).
    SiglessBranch,
    /// Step 3 — malleable satisfaction (`Malleable`).
    Malleable,
    /// Step 3 — exceeds script resource limits (`BranchExceedResouceLimits`).
    ResourceLimit,
    /// Step 3 — a key used more than once (`RepeatedPubkeys`).
    RepeatedKeys,
    /// Step 3 — an unspendable mixed height/time timelock path
    /// (`HeightTimelockCombination`) — the "wrong timelock loses money" guard.
    MixedTimelock,
    /// Step 4 — exceeds the always-previewable complexity envelope.
    OverEnvelope,
    /// Step 1 — a key node carries an extended PRIVATE key; build-descriptor is
    /// watch-only-out. Refused with a controlled message (never echoing the key).
    SecretKey,
}

impl DiagnosticKind {
    pub fn as_str(self) -> &'static str {
        match self {
            DiagnosticKind::Param => "param",
            DiagnosticKind::SchemaField => "schema_field",
            DiagnosticKind::TypeError => "type_error",
            DiagnosticKind::SiglessBranch => "sigless_branch",
            DiagnosticKind::Malleable => "malleable",
            DiagnosticKind::ResourceLimit => "resource_limit",
            DiagnosticKind::RepeatedKeys => "repeated_keys",
            DiagnosticKind::MixedTimelock => "mixed_timelock",
            DiagnosticKind::OverEnvelope => "over_envelope",
            DiagnosticKind::SecretKey => "secret_key",
        }
    }
}

/// Run the 4-step gate. `Ok` ⇒ the policy is emit-safe; `Err` ⇒ one or more
/// node-addressed diagnostics (step 1 returns ALL field errors; steps 2–4
/// short-circuit on first failure).
/// (cmd routes through [`validate_with_allow`]; this no-allowance form is the
/// stable gate API, delegation-pinned by the gate unit tests.)
#[allow(dead_code)]
pub fn validate(doc: &SpecDoc) -> Result<ValidatedPolicy, Vec<Diagnostic>> {
    validate_with_cap(doc, DEFAULT_PREVIEW_CAP)
}

/// `validate` with an explicit cap (for tests / a future `--max-conditions`).
#[allow(dead_code)]
pub fn validate_with_cap(doc: &SpecDoc, cap: usize) -> Result<ValidatedPolicy, Vec<Diagnostic>> {
    validate_with_allow(doc, cap, &AllowSet::default())
}

/// `validate` with reviewed sanity opt-outs (allow SPEC §2). With the default
/// (empty) [`AllowSet`] this is behavior-identical to [`validate_with_cap`]:
/// `ext_check(&ExtParams::new())` is `sanity_check()`'s five arms in the same
/// order plus a `raw_pkh` arm vacuous for IR-rendered input.
pub fn validate_with_allow(
    doc: &SpecDoc,
    cap: usize,
    allow: &AllowSet,
) -> Result<ValidatedPolicy, Vec<Diagnostic>> {
    // ---- Step 1: schema field-validation (collect ALL) -------------------
    let mut field_diags = Vec::new();
    validate_fields(&doc.root, "root", &mut field_diags);
    if !field_diags.is_empty() {
        return Err(field_diags);
    }

    // ---- Step 2: type-check (render → wsh(M) → Descriptor::from_str) ------
    let rendered = doc.render_descriptor();
    let descriptor = match MsDescriptor::<DescriptorPublicKey>::from_str(&rendered) {
        Ok(d) => d,
        Err(e) => return Err(vec![localize_type_error(doc, &e.to_string())]),
    };

    // ---- Step 3: sanity_check (the SOLE funds-footgun gate) + localize ----
    let inner = strip_wsh(&rendered);
    let inner_ms = match Miniscript::<DescriptorPublicKey, Segwitv0>::from_str_ext(
        &inner,
        &ExtParams::insane(),
    ) {
        Ok(ms) => ms,
        // Step 2 passed ⇒ inner is a type-correct B-typed miniscript, so the
        // insane parse must succeed. Defensive: never panic in a funds tool.
        Err(e) => return Err(vec![root_diag(DiagnosticKind::TypeError, format!("inner parse: {e}"))]),
    };
    if let Err(rule) = inner_ms.ext_check(&allow.to_ext_params()) {
        return Err(vec![localize_sanity(doc, rule)]);
    }

    // Fired-vs-requested (allow SPEC §2): for each REQUESTED allowance,
    // evaluate the per-rule predicate. Polarity pinned: three safety-positive
    // (fired iff NEGATED), two violation-positive — mirrors localize_sanity's
    // dispatch. Order = ext_check's check order.
    let mut allowed_fired = Vec::new();
    if allow.sigless_branch && !inner_ms.requires_sig() {
        allowed_fired.push(DiagnosticKind::SiglessBranch);
    }
    if allow.malleable && !inner_ms.is_non_malleable() {
        allowed_fired.push(DiagnosticKind::Malleable);
    }
    if allow.resource_limit && !inner_ms.within_resource_limits() {
        allowed_fired.push(DiagnosticKind::ResourceLimit);
    }
    if allow.repeated_keys && inner_ms.has_repeated_keys() {
        allowed_fired.push(DiagnosticKind::RepeatedKeys);
    }
    if allow.mixed_timelock && inner_ms.has_mixed_timelocks() {
        allowed_fired.push(DiagnosticKind::MixedTimelock);
    }

    // ---- Step 4: build-time complexity cap -------------------------------
    if let Some(diag) = check_cap(doc, &descriptor, &inner_ms, cap) {
        return Err(vec![diag]);
    }

    Ok(ValidatedPolicy { descriptor, allowed_fired })
}

// ======================================================================
// Step 1 — field validation
// ======================================================================

fn validate_fields(node: &PolicyNode, path: &str, out: &mut Vec<Diagnostic>) {
    match node {
        PolicyNode::Pk(k) | PolicyNode::Pkh(k) => {
            check_secret_key(k, path, node.kind(), out);
        }
        PolicyNode::Multi(m) | PolicyNode::Sortedmulti(m) => {
            check_threshold(m.k, m.keys.len(), path, node.kind(), out);
            for (i, key) in m.keys.iter().enumerate() {
                check_secret_key(key, &format!("{path}.{}.keys[{i}]", node.kind()), node.kind(), out);
            }
        }
        PolicyNode::Thresh(t) => {
            check_threshold(t.k, t.subs.len(), path, "thresh", out);
        }
        PolicyNode::Sha256(h) | PolicyNode::Hash256(h) => {
            check_hashlock(h, 64, path, node.kind(), out);
        }
        PolicyNode::Hash160(h) | PolicyNode::Ripemd160(h) => {
            check_hashlock(h, 40, path, node.kind(), out);
        }
        PolicyNode::Older(n) => {
            // BIP-68 relative timelock: only the low 16 bits are the value and
            // bit 22 (0x400000) selects 512-second units; consensus masks the
            // operand to 0x0040FFFF, so any other bit (incl. the bit-31 disable
            // flag) is silently dropped and a zero 16-bit value is a no-op lock.
            // Reject the lot — on an engraving surface a silently-weakened
            // timelock is a funds-safety bug, not a parse curiosity.
            if (*n & !0x0040_FFFFu32) != 0 || (*n & 0x0000_FFFFu32) == 0 {
                // The consequence clause MUST branch on the bit-31 disable flag:
                // a CSV operand with bit 31 set is a no-op (no timelock at all),
                // NOT a masked value — claiming "effective value of N" there would
                // be consensus-FALSE.
                let consequence = if *n & 0x8000_0000 != 0 {
                    "the bit-31 disable flag is set, so consensus would treat this \
                     CHECKSEQUENCEVERIFY as a no-op — no relative timelock at all"
                        .to_string()
                } else {
                    let unit = if *n & 0x0040_0000 != 0 {
                        " (512-second units)"
                    } else {
                        " blocks"
                    };
                    format!(
                        "consensus would silently mask this to an effective value of {}{}, \
                         weakening or nullifying the timelock",
                        *n & 0x0000_FFFF,
                        unit
                    )
                };
                out.push(field_diag(
                    path,
                    format!(
                        "older(N) encodes a BIP-68 relative timelock: only the low 16 bits are \
                         the value, and bit 22 (0x400000) selects 512-second units. All other \
                         bits — including the bit-31 disable flag — must be clear, and the 16-bit \
                         value must be non-zero. got {n} (0x{n:08x}); {consequence}. Use 1..=65535 \
                         (blocks) or 0x400000|(1..=65535) (512-second units)."
                    ),
                ));
            }
        }
        PolicyNode::After(n) => {
            if *n == 0 {
                out.push(field_diag(path, "after(N) requires N ≥ 1; got 0".to_string()));
            } else if *n > 0x7FFF_FFFF {
                // BIP-65 absolute locktimes are bounded [1, 0x7fffffff]. Step-2
                // from_str already rejects this; surfacing it here gives a
                // node-localized field diagnostic (parity with older()).
                out.push(field_diag(
                    path,
                    format!(
                        "after(N) encodes a BIP-65 absolute locktime; valid range is \
                         1..=0x7fffffff (2147483647). got {n} (0x{n:08x})"
                    ),
                ));
            }
        }
        // Wrap / combinators have no own field constraints; recurse below.
        PolicyNode::AndV(_)
        | PolicyNode::OrD(_)
        | PolicyNode::OrI(_)
        | PolicyNode::OrB(_)
        | PolicyNode::Andor(_)
        | PolicyNode::Wrap(_) => {}
    }
    for (cpath, child) in child_paths(node, path) {
        validate_fields(child, &cpath, out);
    }
}

/// Watch-only-out screen (SPEC §0): refuse a key node carrying an extended
/// PRIVATE key. Strips an optional `[origin]` prefix, then checks for an
/// extended-private prefix — `xprv`/`tprv`/`yprv`/`zprv`/`uprv`/`vprv` (+ capital
/// variants), all of which have `prv` at byte offset 1..4 (`xpub`/`tpub` have
/// `pub`). The diagnostic NEVER echoes the key (no leak surface), and fires at
/// step 1 — independent of the step-2 `from_str` error text. WIF / raw-hex
/// secrets are not prefix-detectable here; they are refused by the step-2
/// `from_str` type-check (which does not echo the key either — pinned by the
/// `cli_build_descriptor` no-leak test).
fn check_secret_key(key: &str, path: &str, kind: &str, out: &mut Vec<Diagnostic>) {
    let key_part = key.rsplit(']').next().unwrap_or(key);
    let is_xprv = key_part.is_char_boundary(4) && key_part.as_bytes().get(1..4) == Some(b"prv");
    if is_xprv {
        out.push(Diagnostic {
            node_path: path.to_string(),
            kind: DiagnosticKind::SecretKey,
            message: format!(
                "{kind} key is an extended PRIVATE key — build-descriptor is watch-only; supply an xpub cosigner key (no secret material)"
            ),
            flag: None,
        });
    }
}

fn check_threshold(k: u32, n: usize, path: &str, kind: &str, out: &mut Vec<Diagnostic>) {
    if n == 0 {
        out.push(field_diag(path, format!("{kind} has no keys/subs")));
    } else if k == 0 || (k as usize) > n {
        out.push(field_diag(
            path,
            format!("{kind} threshold k={k} must satisfy 1 ≤ k ≤ {n}"),
        ));
    }
}

fn check_hashlock(hex: &str, want_len: usize, path: &str, kind: &str, out: &mut Vec<Diagnostic>) {
    let ok_len = hex.len() == want_len;
    let ok_hex = hex.bytes().all(|b| b.is_ascii_hexdigit());
    if !ok_len || !ok_hex {
        out.push(field_diag(
            path,
            format!("{kind} expects a {want_len}-char hex digest; got {:?} (len {})", hex, hex.len()),
        ));
    }
}

// ======================================================================
// Step 3 — sanity localization dispatch
// ======================================================================

/// A per-subtree predicate that returns `true` when the subtree exhibits the
/// failing sanity property (used by §3.4 localization).
type SanityPredicate = fn(&Miniscript<DescriptorPublicKey, Segwitv0>) -> bool;

fn localize_sanity(doc: &SpecDoc, rule: AnalysisError) -> Diagnostic {
    // Map the first-failing sanity rule to its localizing predicate + kind.
    let (kind, predicate): (DiagnosticKind, SanityPredicate) =
        match rule {
            AnalysisError::SiglessBranch => {
                (DiagnosticKind::SiglessBranch, |ms| !ms.requires_sig())
            }
            AnalysisError::Malleable => (DiagnosticKind::Malleable, |ms| !ms.is_non_malleable()),
            AnalysisError::BranchExceedResouceLimits => {
                (DiagnosticKind::ResourceLimit, |ms| !ms.within_resource_limits())
            }
            AnalysisError::RepeatedPubkeys => {
                (DiagnosticKind::RepeatedKeys, |ms| ms.has_repeated_keys())
            }
            AnalysisError::HeightTimelockCombination => {
                (DiagnosticKind::MixedTimelock, |ms| ms.has_mixed_timelocks())
            }
            // The IR has no raw-pkh-without-key node, so ContainsRawPkh is
            // unreachable for builder-emitted trees. We still handle it (fail
            // closed) rather than panic. NB: this match is exhaustive over
            // miniscript's AnalysisError — a future variant breaks compilation
            // here, which is the intended forcing function.
            AnalysisError::ContainsRawPkh => {
                return root_diag(
                    DiagnosticKind::TypeError,
                    "unexpected raw-pkh fragment (unreachable for the builder IR)".to_string(),
                )
            }
        };
    let path = localize(&doc.root, "root", &|ms| predicate(ms)).unwrap_or_else(|| "root".to_string());
    Diagnostic {
        node_path: path,
        kind,
        // Refusal-message affordance (allow SPEC §1): name the exact --allow
        // token. Only step-3 kinds reach this fn, all of them allowable.
        message: format!(
            "{}; rerun with --allow {} after review",
            sanity_message(kind),
            kind.as_str().replace('_', "-")
        ),
        flag: None,
    }
}

fn sanity_message(kind: DiagnosticKind) -> String {
    match kind {
        DiagnosticKind::SiglessBranch => {
            "this subtree can be spent without any signature (anyone-can-spend path)".to_string()
        }
        DiagnosticKind::Malleable => {
            "this subtree has a malleable satisfaction".to_string()
        }
        DiagnosticKind::ResourceLimit => {
            "this subtree exceeds script resource limits".to_string()
        }
        DiagnosticKind::RepeatedKeys => {
            "this subtree reuses a public key (RepeatedPubkeys)".to_string()
        }
        DiagnosticKind::MixedTimelock => {
            "this subtree combines incompatible height/time timelocks → an unspendable path".to_string()
        }
        _ => "sanity_check failure".to_string(),
    }
}

// ======================================================================
// Step 2 — type-error localization
// ======================================================================

fn localize_type_error(doc: &SpecDoc, top_err: &str) -> Diagnostic {
    let path = localize_parse_failure(&doc.root, "root").unwrap_or_else(|| "root".to_string());
    Diagnostic {
        node_path: path,
        kind: DiagnosticKind::TypeError,
        message: format!("miniscript type/parse error: {top_err}"),
        flag: None,
    }
}

/// §3.4 post-order walk for a TYPE error: the deepest subtree whose standalone
/// render fails `from_str_ext(insane)` with a real (non-`NonTopLevel`) error.
fn localize_parse_failure(node: &PolicyNode, path: &str) -> Option<String> {
    for (cpath, child) in child_paths(node, path) {
        if let Some(p) = localize_parse_failure(child, &cpath) {
            return Some(p);
        }
    }
    match Miniscript::<DescriptorPublicKey, Segwitv0>::from_str_ext(&node.render(), &ExtParams::insane()) {
        Ok(_) => None,
        // Non-B subtree (e.g. an explicit `v:` child) — not standalone-testable;
        // defer to the nearest B-typed ancestor (B-type restriction, §3.4).
        Err(miniscript::Error::NonTopLevel(_)) => None,
        Err(_) => Some(path.to_string()),
    }
}

// ======================================================================
// §3.4 localization core (for sanity predicates)
// ======================================================================

/// Post-order walk returning the deepest subtree for which `defect` holds. Used
/// for step-3 funds-footgun localization: every subtree parses under `insane`
/// (the whole tree type-checked at step 2); a non-B subtree is skipped (defer to
/// nearest B ancestor); the deepest defective subtree is the diagnostic node.
fn localize(
    node: &PolicyNode,
    path: &str,
    defect: &dyn Fn(&Miniscript<DescriptorPublicKey, Segwitv0>) -> bool,
) -> Option<String> {
    for (cpath, child) in child_paths(node, path) {
        if let Some(p) = localize(child, &cpath, defect) {
            return Some(p);
        }
    }
    match Miniscript::<DescriptorPublicKey, Segwitv0>::from_str_ext(&node.render(), &ExtParams::insane()) {
        Ok(ms) => {
            if defect(&ms) {
                Some(path.to_string())
            } else {
                None
            }
        }
        // INVARIANT (M3, enforced since audit M2): this runs only after step 2
        // type-checked the WHOLE tree, so a subtree's only possible
        // `from_str_ext(insane)` failure is `NonTopLevel` (a non-B subtree,
        // e.g. an explicit `v:` child) — not a real parse/type error. A non-B
        // subtree is not standalone-testable → defer to the nearest B-typed
        // ancestor. Any OTHER variant means the invariant broke (step 2 was
        // relaxed): debug/test builds fail loudly via the debug_assert; release
        // stays fail-closed (None → caller's root fallback, behavior unchanged).
        Err(miniscript::Error::NonTopLevel(_)) => None,
        Err(e) => {
            debug_assert!(
                false,
                "localize: unexpected from_str_ext(insane) failure at {path} on a \
                 step-2-typechecked subtree (variant: {e:?})"
            );
            None
        }
    }
}

// ======================================================================
// Step 4 — build-time complexity cap
// ======================================================================

/// Compute `raw = 2^(n_keys + n_hashes) × n_tl_states` from the parsed AST (the
/// SAME `Miniscript` enumerate sees) and refuse if it exceeds `cap`.
///
/// Counts MUST agree with `cost::enumerate` (else a policy passes this cap and
/// still trips `ConditionsTooMany` in the Phase-3 preview, or vice versa). To be
/// drift-proof, on the same parsed AST we: classify timelocks via the same
/// `is_block_height`/`is_height_locked` methods; dedup keys via the same
/// `BTreeSet<DescriptorPublicKey>` over `for_each_key` (M1); and count hash
/// LEAVES (not distinct digests) to match enumerate's non-deduping hash walk
/// (Phase-2 review I1). Boundary agreement tests pin both the no-hash and the
/// repeated-digest cases against `run_compare_cost`.
fn check_cap(
    doc: &SpecDoc,
    descriptor: &MsDescriptor<DescriptorPublicKey>,
    inner_ms: &Miniscript<DescriptorPublicKey, Segwitv0>,
    cap: usize,
) -> Option<Diagnostic> {
    let n_keys = distinct_keys(descriptor);
    let (n_hashes, n_tl_states) = hash_and_timelock_counts(inner_ms);

    let over = match 2usize
        .checked_pow((n_keys + n_hashes) as u32)
        .and_then(|pow| pow.checked_mul(n_tl_states))
    {
        Some(raw) => raw > cap,
        None => true, // overflow ⇒ definitely over-envelope
    };

    if over {
        // Re-point at root: the cap is a whole-tree property.
        let _ = doc;
        Some(Diagnostic {
            node_path: "root".to_string(),
            kind: DiagnosticKind::OverEnvelope,
            message: format!(
                "policy exceeds the always-previewable envelope (2^({n_keys} keys + {n_hashes} hashes) × {n_tl_states} timelock-states > cap {cap}); use the raw `--descriptor` path for arbitrarily complex policies"
            ),
            flag: None,
        })
    } else {
        None
    }
}

fn distinct_keys(descriptor: &MsDescriptor<DescriptorPublicKey>) -> usize {
    let mut set: BTreeSet<DescriptorPublicKey> = BTreeSet::new();
    descriptor.for_each_key(|k| {
        set.insert(k.clone());
        true
    });
    set.len()
}

/// `(n_hash_leaves, n_tl_states)` from the parsed AST — matching
/// `cost::enumerate`'s `collect_ast_assets` classification EXACTLY.
///
/// **Hashes are counted as LEAVES, not distinct digests** — `enumerate`'s
/// `walk_segv0_for_hash_leaves` (`enumerate.rs:422-444`) unconditionally pushes
/// every hash leaf into a `Vec` (no dedup) and `n_hashes = hashes.len()`
/// (`enumerate.rs:90`). The same digest in two leaves counts as 2 on both sides,
/// so the gate cap and the Phase-3 preview agree (Phase-2 review I1). (Keys ARE
/// deduped — `BTreeSet<DescriptorPublicKey>` — on both sides; see
/// `distinct_keys`.) Timelock states: `n_abs × n_rel`, each `1..=3`, same
/// `is_block_height`/`is_height_locked` classification.
fn hash_and_timelock_counts(ms: &Miniscript<DescriptorPublicKey, Segwitv0>) -> (usize, usize) {
    let mut n_hash_leaves = 0usize;
    let (mut abs_h, mut abs_t, mut rel_b, mut rel_t) = (false, false, false, false);
    for sub in ms.iter() {
        match &sub.node {
            Terminal::Sha256(_)
            | Terminal::Hash256(_)
            | Terminal::Ripemd160(_)
            | Terminal::Hash160(_) => {
                n_hash_leaves += 1;
            }
            Terminal::After(abs) => {
                if abs.is_block_height() {
                    abs_h = true;
                } else if abs.is_block_time() {
                    abs_t = true;
                }
            }
            Terminal::Older(rel) => {
                if rel.is_height_locked() {
                    rel_b = true;
                } else if rel.is_time_locked() {
                    rel_t = true;
                }
            }
            _ => {}
        }
    }
    let n_abs = 1 + abs_h as usize + abs_t as usize;
    let n_rel = 1 + rel_b as usize + rel_t as usize;
    (n_hash_leaves, n_abs * n_rel)
}

// ======================================================================
// Shared helpers
// ======================================================================

/// The path-segment-bearing children of a node (mirrors
/// `PolicyNode::children()`, but yields the JSON path segment per child).
fn child_paths<'a>(node: &'a PolicyNode, path: &str) -> Vec<(String, &'a PolicyNode)> {
    match node {
        PolicyNode::AndV(s)
        | PolicyNode::OrD(s)
        | PolicyNode::OrI(s)
        | PolicyNode::OrB(s) => {
            let kind = node.kind();
            vec![
                (format!("{path}.{kind}[0]"), &s[0]),
                (format!("{path}.{kind}[1]"), &s[1]),
            ]
        }
        PolicyNode::Andor(s) => vec![
            (format!("{path}.andor[0]"), &s[0]),
            (format!("{path}.andor[1]"), &s[1]),
            (format!("{path}.andor[2]"), &s[2]),
        ],
        PolicyNode::Thresh(t) => t
            .subs
            .iter()
            .enumerate()
            .map(|(i, c)| (format!("{path}.thresh.subs[{i}]"), c))
            .collect(),
        PolicyNode::Wrap(w) => vec![(format!("{path}.wrap.sub"), &*w.sub)],
        _ => Vec::new(),
    }
}

fn strip_wsh(rendered: &str) -> String {
    rendered
        .strip_prefix("wsh(")
        .and_then(|s| s.strip_suffix(')'))
        .map(|s| s.to_string())
        .unwrap_or_else(|| rendered.to_string())
}

fn field_diag(path: &str, message: String) -> Diagnostic {
    Diagnostic {
        node_path: path.to_string(),
        kind: DiagnosticKind::SchemaField,
        message,
        flag: None,
    }
}

fn root_diag(kind: DiagnosticKind, message: String) -> Diagnostic {
    Diagnostic {
        node_path: "root".to_string(),
        kind,
        message,
        flag: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Bare valid xpubs (origin not needed for type-check / sanity).
    const A: &str = "xpub661MyMwAqRbcEZVB4dScxMAdx6d4nFc9nvyvH3v4gJL378CSRZiYmhRoP7mBy6gSPSCYk6SzXPTf3ND1cZAceL7SfJ1Z3GC8vBgp2epUt13";
    const B: &str = "xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8";
    const C: &str = "xpub661MyMwAqRbcFW31YEwpkMuc5THy2PSt5bDMsktWQcFF8syAmRUapSCGu8ED9W6oDMSgv6Zz8idoc4a6mr8BDzTJY47LJhkJ8UB7WEGuduB";
    const D: &str = "xpub661MyMwAqRbcGczjuMoRm6dXaLDEhW1u34gKenbeYqAix21mdUKJyuyu5F1rzYGVxyL6tmgBUAEPrEz92mBXjByMRiJdba9wpnN37RLLAXa";

    fn gate(json: &str) -> Result<ValidatedPolicy, Vec<Diagnostic>> {
        let doc = SpecDoc::parse(json).expect("fixture parses");
        validate(&doc)
    }

    fn errs(json: &str) -> Vec<Diagnostic> {
        gate(json).err().expect("expected gate failure")
    }

    fn doc(root_json: &str) -> String {
        format!(r#"{{"schema_version":1,"wrapper":"wsh","root":{root_json}}}"#)
    }

    // ---- --allow / AllowSet (allow SPEC §2) ------------------------------
    //
    // Coverage note (allow SPEC §5 best-effort posture, impl-r1 M2): fired
    // detection is exercised end-to-end for sigless-branch (unit + CLI),
    // repeated-keys and mixed-timelock (CLI banners). `malleable` and
    // `resource-limit` have no minimal in-envelope construction worth
    // authoring (see the constructibility rationale further down in this
    // module's M1 comment block); their coverage is the per-variant
    // AllowSet→ExtParams mapping cell below — the mapping is what needs
    // pinning, the ext_check mechanism is uniform across the five.

    fn sigless_doc() -> String {
        doc(&format!(r#"{{"or_d":[{{"pk":"{A}"}},{{"after":100}}]}}"#))
    }

    /// AllowSet → ExtParams mapping, one cell per variant (allow SPEC §5).
    #[test]
    fn allow_set_maps_each_variant_to_its_ext_params_field() {
        type FieldProbe = fn(&miniscript::miniscript::analyzable::ExtParams) -> bool;
        let cases: [(AllowSet, FieldProbe); 5] = [
            (AllowSet { sigless_branch: true, ..Default::default() }, |p| p.top_unsafe),
            (AllowSet { malleable: true, ..Default::default() }, |p| p.malleability),
            (AllowSet { resource_limit: true, ..Default::default() }, |p| p.resource_limitations),
            (AllowSet { repeated_keys: true, ..Default::default() }, |p| p.repeated_pk),
            (AllowSet { mixed_timelock: true, ..Default::default() }, |p| p.timelock_mixing),
        ];
        for (set, field) in cases {
            let params = set.to_ext_params();
            assert!(field(&params), "{set:?} must set its field");
            // raw_pkh is never exposed (allow SPEC §1).
            assert!(!params.raw_pkh, "{set:?} must not enable raw_pkh");
        }
        let none = AllowSet::default().to_ext_params();
        assert!(
            !none.top_unsafe && !none.malleability && !none.resource_limitations
                && !none.repeated_pk && !none.timelock_mixing && !none.raw_pkh,
            "empty AllowSet == ExtParams::new() baseline"
        );
    }

    /// Delegation: validate / validate_with_cap behave identically to the
    /// empty-AllowSet path (sigless refuses through every entry point).
    #[test]
    fn allow_default_baseline_identical_through_all_entry_points() {
        let json = sigless_doc();
        let parsed = SpecDoc::parse(&json).unwrap();
        for result in [
            validate(&parsed),
            validate_with_cap(&parsed, DEFAULT_PREVIEW_CAP),
            validate_with_allow(&parsed, DEFAULT_PREVIEW_CAP, &AllowSet::default()),
        ] {
            let diags = result.err().expect("sigless refuses");
            assert_eq!(diags[0].kind, DiagnosticKind::SiglessBranch);
        }
    }

    /// Fired detection (allow SPEC §2 polarity): an allowed rule that fires
    /// lands in allowed_fired; a sane tree under the same allowance fires
    /// nothing.
    #[test]
    fn allow_fired_detection_populates_allowed_fired() {
        let parsed = SpecDoc::parse(&sigless_doc()).unwrap();
        let allow = AllowSet { sigless_branch: true, ..Default::default() };
        let vp = validate_with_allow(&parsed, DEFAULT_PREVIEW_CAP, &allow)
            .expect("allowed sigless passes");
        assert_eq!(vp.allowed_fired, vec![DiagnosticKind::SiglessBranch]);

        let sane = doc(&format!(
            r#"{{"or_d":[{{"pk":"{A}"}},{{"and_v":[{{"wrap":{{"w":"v","sub":{{"pk":"{B}"}}}}}},{{"older":100}}]}}]}}"#
        ));
        let parsed = SpecDoc::parse(&sane).unwrap();
        let vp = validate_with_allow(&parsed, DEFAULT_PREVIEW_CAP, &allow)
            .expect("sane tree passes");
        assert!(vp.allowed_fired.is_empty(), "nothing fired on a sane tree");
    }

    /// The refusal hint names the exact --allow token (allow SPEC §1).
    #[test]
    fn sanity_refusal_carries_rerun_hint() {
        let diags = errs(&sigless_doc());
        assert!(
            diags[0].message.contains("rerun with --allow sigless-branch after review"),
            "hint: {}",
            diags[0].message
        );
    }

    // ---- localize() error-collapse narrowing (audit M2) -------------------

    /// Pins `localize`'s SANCTIONED error arm: a non-B subtree (here a root
    /// `v:` wrap) fails `from_str_ext(insane)` with `Error::NonTopLevel` and
    /// must defer to None — NOT trip the `debug_assert!` arm (this cell would
    /// panic in debug builds if the narrowing ever misrouted NonTopLevel).
    /// The debug_assert arm for any OTHER variant is deliberately unreachable
    /// today — step 2 type-checks the whole tree before any `localize` call —
    /// so no cell contorts to force it; its value is loud invariant
    /// enforcement the moment step 2 is ever relaxed.
    #[test]
    fn localize_non_top_level_subtree_defers_to_none() {
        let json = doc(&format!(r#"{{"wrap":{{"w":"v","sub":{{"pk":"{A}"}}}}}}"#));
        let parsed = SpecDoc::parse(&json).unwrap();
        // Fixture sanity: the root's standalone render IS NonTopLevel.
        assert!(
            matches!(
                Miniscript::<DescriptorPublicKey, Segwitv0>::from_str_ext(
                    &parsed.root.render(),
                    &ExtParams::insane(),
                ),
                Err(miniscript::Error::NonTopLevel(_))
            ),
            "fixture must render a non-B (NonTopLevel) subtree"
        );
        // Defect never holds for the B-typed child → the root's NonTopLevel
        // arm is what produces the final None.
        assert_eq!(localize(&parsed.root, "root", &|_| false), None);
    }

    // ---- the 5 archetypes all pass the gate (GREEN) ----------------------

    #[test]
    fn archetypes_pass_the_gate() {
        for f in [
            include_str!("../../tests/fixtures/descriptor_builder/simple-timelocked-inheritance.json"),
            include_str!("../../tests/fixtures/descriptor_builder/decaying-multisig.json"),
            include_str!("../../tests/fixtures/descriptor_builder/kofn-recovery.json"),
            include_str!("../../tests/fixtures/descriptor_builder/tiered-recovery.json"),
            include_str!("../../tests/fixtures/descriptor_builder/hashlock-gated.json"),
        ] {
            assert!(gate(f).is_ok(), "archetype should pass the gate: {f}");
        }
    }

    // ---- step 1: field validation ----------------------------------------

    #[test]
    fn rejects_threshold_exceeding_keys() {
        let d = doc(&format!(r#"{{"multi":{{"k":3,"keys":["{A}","{B}"]}}}}"#));
        let e = errs(&d);
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].kind, DiagnosticKind::SchemaField);
        assert_eq!(e[0].node_path, "root");
        assert!(e[0].message.contains("k=3"));
    }

    #[test]
    fn rejects_bad_hashlock_hex() {
        // 63-char (too short) + a non-hex char
        let d = doc(r#"{"andor":[{"pk":"X"},{"sha256":"zz6a54995ca48600920a19bf7bc502ca5f2f7d07e6f804c4f00ebf0325084db"},{"older":5}]}"#);
        let e = errs(&d);
        assert!(e.iter().any(|x| x.kind == DiagnosticKind::SchemaField && x.message.contains("sha256")));
    }

    #[test]
    fn rejects_zero_timelock() {
        let older0 = doc(&format!(r#"{{"and_v":[{{"wrap":{{"w":"v","sub":{{"pk":"{A}"}}}}}},{{"older":0}}]}}"#));
        let after0 = doc(&format!(r#"{{"and_v":[{{"wrap":{{"w":"v","sub":{{"pk":"{A}"}}}}}},{{"after":0}}]}}"#));
        assert!(errs(&older0).iter().any(|x| x.kind == DiagnosticKind::SchemaField && x.message.contains("older")));
        assert!(errs(&after0).iter().any(|x| x.kind == DiagnosticKind::SchemaField && x.message.contains("after")));
    }

    // ---- step 1: older() BIP-68 mask gate (funds-safety) -----------------
    // A single timelock per tree (M2): a height + a time older() in one tree
    // would trip step-3 HeightTimelockCombination and contaminate the assertion.
    // `gate(...)` (not `errs`, which panics on success) so accept cells and the
    // pre-fix RED state are clean assertions, not panics.
    fn older_tree(n: u32) -> String {
        doc(&format!(
            r#"{{"and_v":[{{"wrap":{{"w":"v","sub":{{"pk":"{A}"}}}}}},{{"older":{n}}}]}}"#
        ))
    }
    fn after_tree(n: u32) -> String {
        doc(&format!(
            r#"{{"and_v":[{{"wrap":{{"w":"v","sub":{{"pk":"{A}"}}}}}},{{"after":{n}}}]}}"#
        ))
    }
    /// Step-1 field diagnostics for `tree` (empty when the gate passes step 1
    /// or succeeds entirely) — never panics, unlike `errs`.
    fn field_diags(tree: &str) -> Vec<Diagnostic> {
        match gate(tree) {
            Ok(_) => Vec::new(),
            Err(ds) => ds
                .into_iter()
                .filter(|d| d.kind == DiagnosticKind::SchemaField)
                .collect(),
        }
    }

    #[test]
    fn rejects_masked_older_timelocks() {
        // Garbage outside the BIP-68 value(16) + type-flag(bit-22) field, or a
        // zero 16-bit value, would be silently masked by consensus to a weakened
        // or zero relative timelock. All must be rejected at step 1.
        // RED-proof asymmetry (M-A): 65536/105120/0x400000 produce NO field diag
        // pre-fix (RED = diag-absence); 0x80000090 is already rejected pre-fix by
        // the n>=2^31 check (RED = wording mismatch only).
        for n in [65536u32, 105120, 0x0040_0000, 0x8000_0090] {
            let fd = field_diags(&older_tree(n));
            assert!(
                fd.iter().any(|x| x.message.contains("older")),
                "older({n}) must be rejected at step 1: {fd:?}"
            );
        }
        // The bit-31-CLEAR masked case carries the "effective value" wording.
        let masked = field_diags(&older_tree(65536));
        assert!(
            masked.iter().any(|x| x.message.contains("effective value")),
            "older(65536) message must state the consensus-effective value: {masked:?}"
        );
        // The bit-31-SET case is a CSV no-op — must NOT claim a masked value.
        let nop = field_diags(&older_tree(0x8000_0090));
        assert!(
            nop.iter()
                .any(|x| x.message.contains("no-op") && x.message.contains("disable flag")),
            "older(0x80000090) message must state the bit-31 disable-flag no-op: {nop:?}"
        );
        assert!(
            !nop.iter().any(|x| x.message.contains("effective value")),
            "older(0x80000090) must NOT claim an effective masked value (it is a no-op)"
        );
    }

    #[test]
    fn accepts_valid_older_block_and_time() {
        // Each in its own single-timelock tree (M2). Block values 1..=65535 and
        // 512-second-unit values 0x400001..=0x40FFFF are valid BIP-68 encodings.
        for n in [1u32, 65535, 52560, 0x0040_0001, 0x0040_FFFF] {
            assert!(
                field_diags(&older_tree(n)).is_empty(),
                "older({n}) is a valid BIP-68 timelock and must not raise a field diag"
            );
        }
        // At least one builds end-to-end (exit-0 path intact).
        assert!(gate(&older_tree(52560)).is_ok());
    }

    #[test]
    fn rejects_after_above_max() {
        // BIP-65 absolute locktimes are bounded [1, 0x7fffffff]; step 2 already
        // rejects > max, but step 1 now gives a node-localized field diag.
        for n in [0x8000_0000u32, 0xFFFF_FFFF] {
            let fd = field_diags(&after_tree(n));
            assert!(
                fd.iter().any(|x| x.message.contains("after")),
                "after({n}) must be rejected at step 1 (SchemaField, not step-2): {fd:?}"
            );
        }
        // Valid absolute locktimes pass step 1 (own trees).
        for n in [1u32, 500_000_000, 0x7FFF_FFFF] {
            assert!(
                field_diags(&after_tree(n)).is_empty(),
                "after({n}) is a valid absolute locktime and must not raise a field diag"
            );
        }
    }

    #[test]
    fn field_errors_are_node_addressed_and_collected() {
        // two bad thresholds in different branches → two diagnostics, distinct paths
        let d = doc(&format!(
            r#"{{"or_d":[{{"multi":{{"k":5,"keys":["{A}","{B}"]}}}},{{"and_v":[{{"wrap":{{"w":"v","sub":{{"multi":{{"k":9,"keys":["{C}","{D}"]}}}}}}}},{{"older":5}}]}}]}}"#
        ));
        let e = errs(&d);
        assert_eq!(e.len(), 2, "both field errors collected: {e:?}");
        let paths: Vec<&str> = e.iter().map(|x| x.node_path.as_str()).collect();
        assert!(paths.contains(&"root.or_d[0]"));
        assert!(paths.contains(&"root.or_d[1].and_v[0].wrap.sub"));
    }

    // ---- step 2: type error, localized -----------------------------------

    #[test]
    fn type_error_missing_v_wrapper_localizes_to_and_v() {
        // and_v(pk, older) — left must be V; pk is B → type error at the and_v.
        let d = doc(&format!(r#"{{"or_d":[{{"pk":"{A}"}},{{"and_v":[{{"pk":"{B}"}},{{"older":5}}]}}]}}"#));
        let e = errs(&d);
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].kind, DiagnosticKind::TypeError);
        assert_eq!(e[0].node_path, "root.or_d[1]");
    }

    // ---- step 3: sanity rejections, each localized (oracle = sanity_check,
    //              NOT from_str — F1) -------------------------------------

    #[test]
    fn sigless_branch_localizes_to_the_sigless_node() {
        // or_d(pk, after) — the `after` branch needs no signature.
        let d = doc(&format!(r#"{{"or_d":[{{"pk":"{A}"}},{{"after":100}}]}}"#));
        let e = errs(&d);
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].kind, DiagnosticKind::SiglessBranch);
        assert_eq!(e[0].node_path, "root.or_d[1]");
    }

    #[test]
    fn repeated_key_passes_step2_but_step3_rejects() {
        // or_b(pk(A), s:pk(A)) — same key twice. from_str (step 2) is LENIENT
        // (F1); only sanity_check (step 3) rejects → kind is RepeatedKeys.
        let inner = format!(r#"{{"or_b":[{{"pk":"{A}"}},{{"wrap":{{"w":"s","sub":{{"pk":"{A}"}}}}}}]}}"#);
        let d = doc(&inner);
        // step 2 passes:
        let rendered = SpecDoc::parse(&d).unwrap().render_descriptor();
        assert!(MsDescriptor::<DescriptorPublicKey>::from_str(&rendered).is_ok(),
            "from_str must be lenient on repeated keys (F1)");
        // step 3 rejects:
        let e = errs(&d);
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].kind, DiagnosticKind::RepeatedKeys);
        assert_eq!(e[0].node_path, "root");
    }

    #[test]
    fn mixed_timelock_localizes_to_nearest_common_ancestor() {
        // and_v(v:after(height), and_v(v:after(time), pk)) — height+time absolute
        // mix in one branch → HeightTimelockCombination at the outer and_v (NCA).
        let d = doc(&format!(
            r#"{{"and_v":[{{"wrap":{{"w":"v","sub":{{"after":100}}}}}},{{"and_v":[{{"wrap":{{"w":"v","sub":{{"after":600000000}}}}}},{{"pk":"{A}"}}]}}]}}"#
        ));
        let e = errs(&d);
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].kind, DiagnosticKind::MixedTimelock);
        assert_eq!(e[0].node_path, "root");
    }

    // ---- step 4: build-time complexity cap --------------------------------

    #[test]
    fn over_envelope_refused_at_small_cap() {
        // multi(2,A,B,C) — 3 keys, no hashes/timelocks → raw = 2^3 × 1 = 8.
        let d = doc(&format!(r#"{{"multi":{{"k":2,"keys":["{A}","{B}","{C}"]}}}}"#));
        // cap 4 < 8 → refused
        let doc_parsed = SpecDoc::parse(&d).unwrap();
        let e = validate_with_cap(&doc_parsed, 4).err().expect("over envelope");
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].kind, DiagnosticKind::OverEnvelope);
        // cap 8 == raw → passes
        assert!(validate_with_cap(&doc_parsed, 8).is_ok());
    }

    /// The advisor's boundary agreement test: the gate's `raw` count MUST equal
    /// `cost::enumerate`'s, else a policy passes this cap but trips
    /// `ConditionsTooMany` in the Phase-3 preview (or vice versa). kofn-recovery
    /// = 4 keys, 0 hashes, one rel-block `older` → n_rel=2, n_abs=1 →
    /// raw = 2^4 × 2 = 32. We pin BOTH the gate (cap 32 ok / 31 refused) AND
    /// `run_compare_cost` (max 32 ok / 31 ConditionsTooMany) on the single-path
    /// projection of the same descriptor.
    #[test]
    fn cap_agrees_with_enumerate_at_boundary() {
        use crate::cost::{self, CompareCostArgs, InputForm};
        use crate::error::ToolkitError;

        let json = format!(
            r#"{{"or_d":[{{"multi":{{"k":2,"keys":["{A}","{B}","{C}"]}}}},{{"and_v":[{{"wrap":{{"w":"v","sub":{{"pk":"{D}"}}}}}},{{"older":52560}}]}}]}}"#
        );
        let d = doc(&json);
        let parsed = SpecDoc::parse(&d).unwrap();

        // Gate side: raw == 32.
        assert!(validate_with_cap(&parsed, 32).is_ok(), "gate raw must be ≤ 32");
        let over = validate_with_cap(&parsed, 31).err().expect("gate refuses at 31");
        assert_eq!(over[0].kind, DiagnosticKind::OverEnvelope);

        // Enumerate side: single-path projection (multipath errors in cost).
        let vp = validate_with_cap(&parsed, 32).unwrap();
        let single = vp.descriptor.into_single_descriptors().unwrap()[0].to_string();

        let run = |max: usize| -> Result<(), ToolkitError> {
            let mut sink = Vec::new();
            cost::run_compare_cost(
                &CompareCostArgs {
                    input: InputForm::Descriptor(single.clone()),
                    feerate_sat_per_vb: 1.0,
                    max_conditions: max,
                    json: false,
                },
                &mut sink,
            )
        };
        assert!(run(32).is_ok(), "enumerate raw must be ≤ 32");
        assert!(
            matches!(
                run(31),
                Err(ToolkitError::CompareCost(cost::CompareCostError::ConditionsTooMany { .. }))
            ),
            "enumerate must trip ConditionsTooMany at 31 → enumerate raw == 32 == gate raw"
        );
    }

    /// Phase-2 review M2 (carry-forward): the cap's key dedup is by full
    /// `DescriptorPublicKey` (origin-bearing), so the SAME xpub under two
    /// DIFFERENT origins counts as 2 distinct keys — matching enumerate. (At the
    /// abstract-key level they are distinct, so sanity_check does not flag
    /// RepeatedPubkeys.) `multi(2, xpubA@o1, xpubA@o2)` → 2 keys, raw = 2^2 = 4.
    #[test]
    fn cap_counts_same_xpub_two_origins_as_distinct() {
        use crate::cost::{self, CompareCostArgs, InputForm};
        use crate::error::ToolkitError;

        let json = format!(
            r#"{{"multi":{{"k":2,"keys":["[11111111/0h]{A}","[22222222/0h]{A}"]}}}}"#
        );
        let parsed = SpecDoc::parse(&doc(&json)).unwrap();

        // raw == 4 (2 distinct origin-keys), not 2 (xpub-deduped).
        assert!(validate_with_cap(&parsed, 4).is_ok(), "2 distinct origin-keys ⇒ raw 4");
        assert_eq!(
            validate_with_cap(&parsed, 3).err().unwrap()[0].kind,
            DiagnosticKind::OverEnvelope,
            "cap 3 < raw 4 ⇒ refused (proves 2 keys counted, not 1)"
        );

        // Enumerate agrees: single-path projection, raw == 4.
        let single = validate_with_cap(&parsed, 4)
            .unwrap()
            .descriptor
            .into_single_descriptors()
            .unwrap()[0]
            .to_string();
        let run = |max: usize| -> Result<(), ToolkitError> {
            let mut sink = Vec::new();
            cost::run_compare_cost(
                &CompareCostArgs {
                    input: InputForm::Descriptor(single.clone()),
                    feerate_sat_per_vb: 1.0,
                    max_conditions: max,
                    json: false,
                },
                &mut sink,
            )
        };
        assert!(run(4).is_ok());
        assert!(matches!(
            run(3),
            Err(ToolkitError::CompareCost(cost::CompareCostError::ConditionsTooMany { .. }))
        ));
    }

    /// Phase-2 review I1 regression: hashes are counted as LEAVES, not distinct
    /// digests (enumerate does not dedup). The SAME sha256 digest in two leaves
    /// must count as 2 on both sides. Shape: `and_v(v:sha256(H),and_v(v:sha256(H),pk(A)))`
    /// = 1 key, 2 hash leaves (same H), 0 timelocks → raw = 2^(1+2) × 1 = 8.
    /// (The old `BTreeSet` dedup gave n_hashes=1 → raw=4, which would pass a
    /// cap=4 the Phase-3 preview refuses.)
    #[test]
    fn repeated_digest_cap_agrees_with_enumerate() {
        use crate::cost::{self, CompareCostArgs, InputForm};
        use crate::error::ToolkitError;

        const H: &str = "926a54995ca48600920a19bf7bc502ca5f2f7d07e6f804c4f00ebf0325084dbc";
        let json = format!(
            r#"{{"and_v":[{{"wrap":{{"w":"v","sub":{{"sha256":"{H}"}}}}}},{{"and_v":[{{"wrap":{{"w":"v","sub":{{"sha256":"{H}"}}}}}},{{"pk":"{A}"}}]}}]}}"#
        );
        let d = doc(&json);
        let parsed = SpecDoc::parse(&d).unwrap();

        // Gate side: raw == 8 (would be 4 under the buggy dedup).
        assert!(validate_with_cap(&parsed, 8).is_ok(), "gate raw must be ≤ 8");
        let over = validate_with_cap(&parsed, 7).err().expect("gate refuses at 7");
        assert_eq!(over[0].kind, DiagnosticKind::OverEnvelope);
        // Crucially, the buggy dedup would have PASSED at cap 4:
        let over4 = validate_with_cap(&parsed, 4).err().expect("gate must refuse at 4 (leaf count)");
        assert_eq!(over4[0].kind, DiagnosticKind::OverEnvelope);

        // Enumerate side: single-path projection, raw == 8.
        let single = validate_with_cap(&parsed, 8)
            .unwrap()
            .descriptor
            .into_single_descriptors()
            .unwrap()[0]
            .to_string();
        let run = |max: usize| -> Result<(), ToolkitError> {
            let mut sink = Vec::new();
            cost::run_compare_cost(
                &CompareCostArgs {
                    input: InputForm::Descriptor(single.clone()),
                    feerate_sat_per_vb: 1.0,
                    max_conditions: max,
                    json: false,
                },
                &mut sink,
            )
        };
        assert!(run(8).is_ok(), "enumerate raw must be ≤ 8");
        assert!(
            matches!(
                run(7),
                Err(ToolkitError::CompareCost(cost::CompareCostError::ConditionsTooMany { .. }))
            ),
            "enumerate raw == 8 == gate raw (hash leaves counted, not deduped)"
        );
    }

    // ---- M1: the Malleable + ResourceLimit dispatch arms -----------------
    // miniscript's type system resists producing a malleable-but-otherwise-sane
    // tree via the builder IR (every natural shape is non-malleable), and a
    // resource-limit tree under a sane cap is impractical to construct. So
    // rather than an end-to-end RED (the 3 cross-branch kinds — sigless /
    // repeated / mixed-timelock — already prove the localize() path end-to-end),
    // this pins the AnalysisError→DiagnosticKind dispatch for the two untested
    // arms directly. On a clean tree the predicate never matches, so localize
    // falls back to "root" — which also exercises the fail-closed fallback.

    #[test]
    fn sanity_dispatch_maps_each_rule_to_its_kind() {
        use miniscript::miniscript::analyzable::AnalysisError;
        let clean = SpecDoc::parse(&doc(&format!(r#"{{"pk":"{A}"}}"#))).unwrap();
        let cases = [
            (AnalysisError::SiglessBranch, DiagnosticKind::SiglessBranch),
            (AnalysisError::Malleable, DiagnosticKind::Malleable),
            (AnalysisError::BranchExceedResouceLimits, DiagnosticKind::ResourceLimit),
            (AnalysisError::RepeatedPubkeys, DiagnosticKind::RepeatedKeys),
            (AnalysisError::HeightTimelockCombination, DiagnosticKind::MixedTimelock),
        ];
        for (rule, want_kind) in cases {
            let label = format!("{rule:?}");
            let d = localize_sanity(&clean, rule);
            assert_eq!(d.kind, want_kind, "dispatch for {label}");
            // clean tree ⇒ predicate matches nothing ⇒ fail-closed root fallback
            assert_eq!(d.node_path, "root");
        }
    }
}
