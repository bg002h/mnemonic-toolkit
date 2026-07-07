//! Bitcoin Core `listdescriptors` parser.
//!
//! Per `design/SPEC_wallet_import_v0_26_0.md` §5. Accepts the JSON shape:
//!
//! ```json
//! {
//!   "wallet_name": "<name>",
//!   "descriptors": [
//!     {
//!       "desc": "<descriptor>#<checksum>",
//!       "timestamp": <int|"now">,
//!       "active": <bool>,
//!       "internal": <bool>,
//!       "range": [<int>, <int>],
//!       "next": <int>,
//!       "next_index": <int>
//!     }, ...
//!   ]
//! }
//! ```
//!
//! Each `descriptors[i]` is parsed via the same adapter + `parse_descriptor`
//! pipeline as BSMS (`pipeline::concrete_keys_to_placeholders` →
//! `parse_descriptor::parse_descriptor`). Per-entry metadata (`active`,
//! `internal`, `range`) is preserved via `ParsedImport::source_metadata()`
//! accessor; backed by `ImportProvenance::BitcoinCore(...)`;
//! wallet-state fields (`timestamp`, `next`, `next_index`) are dropped from
//! the bundle output with a single stderr NOTICE per SPEC §2.4.
//!
//! Per SPEC §5.2 step 2.a: `desc` containing the literal substring `xprv`
//! is refused with `ImportWalletXprvForbidden` (exit 2) — Bitcoin Core's
//! `listdescriptors true` form returns xprv-bearing entries that the
//! toolkit must not consume.
//!
//! Network detection mirrors BSMS (§4.2 step 8 = §7.0.a locked): inspect
//! the BIP-48 coin-type child number on the FIRST cosigner's origin path.
//! Per-entry coin-type heterogeneity within a single `desc` body is rejected
//! (same rule as BSMS); cross-entry coin-type heterogeneity (e.g.,
//! descriptors[0] mainnet, descriptors[1] testnet) is NOT enforced at the
//! parser level — each `ParsedImport` carries its own `network` field per
//! SPEC §8.1, and the CLI dispatch may emit per-bundle network metadata.

use super::{
    pipeline::concrete_keys_to_placeholders, validate_watch_only_resolved, CoreSourceMetadata,
    ImportProvenance, ParsedImport, WalletFormatParser,
};
use crate::error::ToolkitError;
use crate::parse_descriptor;
use crate::synthesize::{xpub_to_65, ResolvedSlot};
use bitcoin::bip32::{ChildNumber, DerivationPath, Fingerprint, Xpub};
use miniscript::descriptor::{DescriptorType, Wildcard};
use miniscript::{Descriptor as MsDescriptor, DescriptorPublicKey};
use regex::Regex;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::str::FromStr;
use std::sync::OnceLock;

pub(crate) struct BitcoinCoreParser;

/// Vendor-marker keys that ALSO appear at the top level of competing wallet
/// vendor blobs (Specter, Sparrow, Coldcard, Jade, Electrum, etc.). Their
/// presence at top level overrides any Core match in `sniff` — keeps `sniff`
/// conservative per `SPEC_wallet_import_v0_26_0.md` §6.1.2 lock and the
/// v0.28.0 amendment at `SPEC_wallet_import_v0_28_0.md` §6.1.1 (Q4 lock).
///
/// v0.28.0 P0A additions absorb markers for Phases P1-P6 parsers:
/// - `seed_version`, `wallet_type` — Electrum wallet (SPEC §11.6)
/// - `policyType`, `defaultPolicy`, `keystores` — Sparrow Wallet (SPEC §11.1)
/// - `devices`, `blockheight` — Specter (SPEC §11.2; `label` deliberately
///   omitted per R0 I3 fold — Specter positive sniff uses `blockheight` +
///   `devices` + `descriptor` + `label`, but `label` is generic enough that
///   a legitimate Core blob carrying a top-level `label` key should not be
///   excluded; Specter is still strongly disambiguated by `blockheight`)
/// - `multisig_file` — Blockstream Jade (SPEC §11.5; the top-level reply
///   field of Jade's `get_registered_multisig` RPC. R0 I4 fold removed
///   `register_multisig` from this list — that's the RPC command name,
///   not an on-disk JSON field, verified via Blockstream/Jade docs)
///
/// Note: Coldcard generic-JSON (`chain`, `xfp`, `bipN`) is already covered
/// by the `chain` exclusion (v0.26.0 original); ColdcardMultisig is a text
/// format (NOT JSON) and never reaches this JSON-sniff path.
const VENDOR_MARKER_KEYS: &[&str] = &[
    // v0.26.0 originals (Bitcoin Core / generic-vendor exclusion):
    "chain",
    "policy",
    "version",
    "bipname",
    "extendedPublicKey",
    // v0.28.0 P0A additions (per-format vendor markers; R1 fold):
    "seed_version",
    "wallet_type",
    "policyType",
    "defaultPolicy",
    "keystores",
    "devices",
    "blockheight",
    "multisig_file",
];

impl WalletFormatParser for BitcoinCoreParser {
    fn sniff(blob: &[u8]) -> bool {
        // SPEC §6.1 item 2:
        // 1. Trimmed-leading-whitespace starts with `{`.
        // 2. `serde_json::from_slice::<Value>` succeeds.
        // 3. Top-level value is an object with a `descriptors` key whose
        //    value is a non-empty array.
        // 4. Each `descriptors[i]` is an object with a `desc: String` field.
        // 5. No vendor-specific marker keys present at top level.
        let trimmed = trim_leading_ws(blob);
        if !trimmed.starts_with(b"{") {
            return false;
        }
        let value: Value = match serde_json::from_slice(blob) {
            Ok(v) => v,
            Err(_) => return false,
        };
        let obj = match value.as_object() {
            Some(o) => o,
            None => return false,
        };
        // Conservative absence-check against competing vendor markers.
        for marker in VENDOR_MARKER_KEYS {
            if obj.contains_key(*marker) {
                return false;
            }
        }
        let descriptors = match obj.get("descriptors").and_then(|v| v.as_array()) {
            Some(a) => a,
            None => return false,
        };
        if descriptors.is_empty() {
            return false;
        }
        // Every entry must be an object with a `desc: String`.
        descriptors.iter().all(|entry| {
            entry
                .as_object()
                .and_then(|o| o.get("desc"))
                .and_then(|d| d.as_str())
                .is_some()
        })
    }

    fn parse(blob: &[u8], stderr: &mut dyn Write) -> Result<Vec<ParsedImport>, ToolkitError> {
        // SPEC §5.2 step 1: JSON-parse.
        let value: Value = serde_json::from_slice(blob).map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: bitcoin-core: parse error: invalid JSON: {e}"
            ))
        })?;
        let obj = value.as_object().ok_or_else(|| {
            ToolkitError::ImportWalletParse(
                "import-wallet: bitcoin-core: parse error: top-level JSON value is not an object"
                    .to_string(),
            )
        })?;
        // SPEC §5.1 + Phase 3 R0 I2 fold: extract `wallet_name` from envelope
        // (metadata-only; preserved for Phase 4 canonicalize + Phase 5 --json
        // envelope). Absent / non-string → None.
        let wallet_name = obj
            .get("wallet_name")
            .and_then(|v| v.as_str())
            .map(str::to_string);

        let descriptors = obj
            .get("descriptors")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                ToolkitError::ImportWalletParse(
                    "import-wallet: bitcoin-core: parse error: missing or non-array top-level `descriptors` key"
                        .to_string(),
                )
            })?;
        if descriptors.is_empty() {
            return Err(ToolkitError::ImportWalletParse(
                "import-wallet: bitcoin-core: parse error: top-level `descriptors` array is empty; no bundles to emit"
                    .to_string(),
            ));
        }

        // SPEC §5.2 step 2.d: aggregate dropped-field names across all entries
        // and emit ONE stderr NOTICE if any are present (avoids N notices for
        // an N-entry blob; the field-set is uniform per Core output anyway).
        // Phase 3 R0 M2 fold: join(", ") instead of {:?} Debug for clean
        // user-facing stderr (no brackets/double-quotes).
        let mut aggregate_dropped: Vec<&'static str> = Vec::new();
        for entry in descriptors {
            let eobj = entry.as_object().ok_or_else(|| {
                ToolkitError::ImportWalletParse(
                    "import-wallet: bitcoin-core: parse error: descriptors[i] is not an object"
                        .to_string(),
                )
            })?;
            for f in ["timestamp", "next", "next_index"] {
                if eobj.contains_key(f) && !aggregate_dropped.contains(&f) {
                    aggregate_dropped.push(f);
                }
            }
        }
        if !aggregate_dropped.is_empty() {
            writeln!(
                stderr,
                "notice: import-wallet: bitcoin-core: dropped wallet-state fields {}: not preserved in bundle output (key-state only)",
                aggregate_dropped.join(", ")
            )
            .map_err(ToolkitError::Io)?;
        }

        // Parse-time pre-pass (SPEC_bitcoin_core_receive_change_pair_merge.md
        // §4): recombine a same-key receive/change split pair into one
        // `<a;b>/*` multipath entry BEFORE the per-entry parse loop. Must run
        // AFTER the aggregate dropped-fields NOTICE above (§4.5 — that NOTICE
        // needs to see the ORIGINAL per-entry field set) and BEFORE the
        // per-entry parse loop below.
        let prepared = merge_receive_change_pairs(descriptors.clone(), stderr)?;

        // SPEC §5.2 step 2: per-entry parse loop.
        //
        // `internal` provenance (SPEC_bitcoin_core_receive_change_pair_merge.md
        // §5) is threaded EXPLICITLY per entry: a passthrough entry always
        // carries `Some(bool)` (computed inside `merge_receive_change_pairs`);
        // a pre-pass-merged entry carries `None`.
        let mut out: Vec<ParsedImport> = Vec::with_capacity(prepared.len());
        for (i, p) in prepared.iter().enumerate() {
            out.push(parse_entry(i, &p.value, wallet_name.clone(), p.internal)?);
        }
        Ok(out)
    }
}

/// One key's grouping-relevant signature (SPEC §4.1), EXCLUDING the final
/// use-site step. Positional/ordered equality via `PartialEq` on the
/// enclosing `Vec` — NOT set-based (a swapped-order `multi(...)` differs).
#[derive(Debug, Clone, PartialEq, Eq)]
struct MergeKeySig {
    fingerprint: Fingerprint,
    origin_path: DerivationPath,
    xpub: String,
}

/// Grouping key for the merge pre-pass (§4.1) — everything about a candidate
/// descriptor EXCEPT its final use-site step. Two entries merge only if this
/// is equal (`PartialEq`) AND the §4.2 guard matrix holds.
#[derive(Debug, Clone, PartialEq, Eq)]
struct MergeGroupKey {
    desc_type: DescriptorType,
    multi_keyword: Option<&'static str>,
    threshold: Option<u8>,
    keys: Vec<MergeKeySig>,
}

/// A `descriptors[i]` entry that qualifies as a merge CANDIDATE per §4.2
/// cond. 3 (every key: fixed single unhardened wildcard final step) and
/// cond. 7 (that step is uniform across all keys). Does NOT imply a partner
/// exists — pairing/guard-matrix application happens in
/// `merge_receive_change_pairs`.
struct MergeCandidate {
    idx: usize,
    group: MergeGroupKey,
    step: u32,
    body_no_csum: String,
    active: bool,
    internal: bool,
    range: Option<(u64, u64)>,
}

/// Output of the merge pre-pass: the (possibly-synthesized) entry `Value`
/// plus its explicit `internal` provenance (SPEC §5) — `Some(bool)` for a
/// passthrough entry, `None` for a pre-pass-merged entry.
struct PreparedEntry {
    value: Value,
    internal: Option<bool>,
}

/// Grouping-key discriminant only (SPEC §4.1: `sortedmulti` vs `multi` are
/// different template kinds and never group together). Pure text
/// classification of the checksum-stripped body — NOT used to extract any
/// security-relevant value (steps/origins/xpubs come from the rust-
/// miniscript parse per §4.1). Longest-prefix-first ordering avoids
/// "multi(" false-matching inside "sortedmulti(" / "multi_a(".
fn multi_keyword_of(body_no_csum: &str) -> Option<&'static str> {
    if body_no_csum.contains("sortedmulti_a(") {
        Some("sortedmulti_a")
    } else if body_no_csum.contains("sortedmulti(") {
        Some("sortedmulti")
    } else if body_no_csum.contains("multi_a(") {
        Some("multi_a")
    } else if body_no_csum.contains("multi(") {
        Some("multi")
    } else {
        None
    }
}

/// Re-render with a fresh BIP-380 checksum. 6th local copy per SPEC §6
/// (mirrors `descriptor.rs:246`, `electrum.rs:1027`, `coldcard.rs:515`,
/// `specter.rs:444`, `sparrow.rs:668`).
fn recompute_descriptor_checksum(body_no_csum: &str) -> Result<String, ToolkitError> {
    use miniscript::descriptor::checksum::Engine as ChecksumEngine;
    let mut eng = ChecksumEngine::new();
    eng.input(body_no_csum).map_err(|e| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: bitcoin-core: checksum engine input rejected: {e}"
        ))
    })?;
    let csum = eng.checksum();
    Ok(format!("{body_no_csum}#{csum}"))
}

/// §4.3 — merged `range` is the union/widening of the two entries' ranges; a
/// range difference never blocks the merge (receive/change legitimately carry
/// different scan state). Absent+absent -> absent.
fn union_range(a: Option<(u64, u64)>, b: Option<(u64, u64)>) -> Option<(u64, u64)> {
    match (a, b) {
        (None, None) => None,
        (Some(x), None) => Some(x),
        (None, Some(y)) => Some(y),
        (Some((alo, ahi)), Some((blo, bhi))) => Some((alo.min(blo), ahi.max(bhi))),
    }
}

/// Attempt to classify `body_no_csum` (already checksum-validated,
/// checksum-stripped) as a merge CANDIDATE per §4.2 cond. 3 + cond. 7.
/// Returns `None` for anything that doesn't qualify — including parse
/// failure, script-path `tr` (§0 / cond. 7 taproot lock), non-xpub keys,
/// already-multipath keys, hardened wildcards, non-single-component final
/// steps, hardened final steps, or non-uniform final steps across multiple
/// keys. A `None` here means "leave this entry alone" — it flows through
/// unchanged and (if it truly carries a fixed step) hits the existing
/// `lex_placeholders` floor reject downstream. NEVER an ad-hoc regex, and
/// NEVER `lex_placeholders`/`concrete_keys_to_placeholders` for the final
/// step (they reject the input) — extraction is via rust-miniscript's own
/// parsed key fields (`derivation_path()`/`wildcard()`/`tap_tree()`).
fn analyze_merge_candidate(body_no_csum: &str) -> Option<(MergeGroupKey, u32)> {
    let d = MsDescriptor::<DescriptorPublicKey>::from_str(body_no_csum).ok()?;
    // §4.2 cond. 7 / §0 taproot lock: a script-path `tr` (tapscript leaves
    // present) is OUT of scope — never a merge candidate, always floor-reject.
    if d.tap_tree().is_some() {
        return None;
    }
    let desc_type = d.desc_type();
    let multi_keyword = multi_keyword_of(body_no_csum);
    let threshold = extract_threshold(body_no_csum).ok()?;

    let mut keys: Vec<MergeKeySig> = Vec::new();
    let mut steps: Vec<u32> = Vec::new();
    for key in d.iter_pk() {
        let DescriptorPublicKey::XPub(xk) = &key else {
            // `Single` (raw pubkey, no wildcard) or `MultiXPub` (already
            // multipath) — neither is a fixed-single-step candidate.
            return None;
        };
        if xk.wildcard != Wildcard::Unhardened {
            return None;
        }
        let comps: Vec<&ChildNumber> = (&xk.derivation_path).into_iter().collect();
        if comps.len() != 1 {
            return None;
        }
        let step = match comps[0] {
            ChildNumber::Normal { index } => *index,
            ChildNumber::Hardened { .. } => return None,
        };
        let (fingerprint, origin_path) = xk
            .origin
            .clone()
            .unwrap_or_else(|| (key.master_fingerprint(), DerivationPath::from(vec![])));
        keys.push(MergeKeySig {
            fingerprint,
            origin_path,
            xpub: xk.xkey.to_string(),
        });
        steps.push(step);
    }
    if keys.is_empty() {
        return None;
    }
    let first_step = steps[0];
    if !steps.iter().all(|s| *s == first_step) {
        // §4.2 cond. 7 uniformity violated (e.g. a partial per-key split
        // WITHIN one entry) — not a candidate; the entry's own fixed step(s)
        // will hit the existing floor reject downstream (§8.11).
        return None;
    }
    Some((
        MergeGroupKey {
            desc_type,
            multi_keyword,
            threshold,
            keys,
        },
        first_step,
    ))
}

/// The parse-time pre-pass (SPEC_bitcoin_core_receive_change_pair_merge.md
/// §4): recombine a same-key receive/change split pair (`.../0/*` +
/// `.../1/*`) into one `<a;b>/*` multipath entry, restoring standard Bitcoin
/// Core `listdescriptors` import.
///
/// Maximally-strict guard matrix (§4.2) — merges iff ALL of: identical
/// grouping key (script/threshold/ordered per-key fp+origin+xpub, EXCLUDING
/// the final step); each side a fixed single unhardened wildcard final step;
/// steps differ; `internal` flags disagree; exactly two share the grouping
/// key; multi-key uniformity. ANY deviation -> do not merge (leave both
/// entries -> they hit the existing fixed-step floor reject downstream).
///
/// A receive/change-SHAPED near-miss (fixed-step, steps differ, internal
/// flags disagree) whose grouping keys differ (distinct keys/scripts) is
/// refused directly with a differentiated message (§7) rather than the
/// generic per-entry floor reject — distinct keys are different wallets.
fn merge_receive_change_pairs(
    descriptors: Vec<Value>,
    stderr: &mut dyn Write,
) -> Result<Vec<PreparedEntry>, ToolkitError> {
    // Pass 1: classify every entry that COULD be a merge candidate. Parse /
    // shape / checksum failures here are swallowed (`None`) -- they simply
    // opt the entry out of merge candidacy; the SAME failure surfaces with
    // its normal diagnostic when the entry is later parsed for real via
    // `parse_entry` (fail-closed: never silently "fix" a bad entry here).
    let mut candidates: Vec<MergeCandidate> = Vec::new();
    for (idx, entry) in descriptors.iter().enumerate() {
        let Some(eobj) = entry.as_object() else {
            continue;
        };
        let Some(desc_with_csum) = eobj.get("desc").and_then(|d| d.as_str()) else {
            continue;
        };
        // §4.4 / M9 — validate the candidate's OWN BIP-380 checksum BEFORE
        // consuming it; a corrupt checksum is simply not merge-eligible
        // (§8.17) -- `parse_entry`'s own re-validation raises the real error
        // downstream when this entry is (necessarily) left unmerged.
        let Ok(body_no_csum) = miniscript::descriptor::checksum::verify_checksum(desc_with_csum)
        else {
            continue;
        };
        let Some((group, step)) = analyze_merge_candidate(body_no_csum) else {
            continue;
        };
        let Ok(active) = parse_bool_field(eobj, "active") else {
            continue;
        };
        let Ok(internal) = parse_bool_field(eobj, "internal") else {
            continue;
        };
        let Ok(range) = parse_range_field(eobj.get("range")) else {
            continue;
        };
        candidates.push(MergeCandidate {
            idx,
            group,
            step,
            body_no_csum: body_no_csum.to_string(),
            active,
            internal,
            range,
        });
    }

    // Bucket candidates by grouping key, first-seen order (linear scan --
    // candidate counts are small; a HashMap would need `Hash` on
    // `DerivationPath`/`Fingerprint`, not worth the API risk here).
    let mut buckets: Vec<(MergeGroupKey, Vec<usize>)> = Vec::new();
    for (ci, c) in candidates.iter().enumerate() {
        if let Some(b) = buckets.iter_mut().find(|(g, _)| *g == c.group) {
            b.1.push(ci);
        } else {
            buckets.push((c.group.clone(), vec![ci]));
        }
    }

    // For each 2-member bucket satisfying cond. 4 (steps differ) + cond. 5
    // (internal flags disagree), build the merged entry (§4.3). 3+-member
    // buckets are ambiguous (cond. 6) -> NOTICE, no merge. 1-member buckets
    // have no partner -> left alone.
    struct Merge {
        first_idx: usize,
        second_idx: usize,
        desc_with_csum: String,
        active: bool,
        range: Option<(u64, u64)>,
    }
    let mut merges: Vec<Merge> = Vec::new();
    let mut merged_candidate_idx: HashSet<usize> = HashSet::new();

    for (_group, member_cis) in &buckets {
        if member_cis.len() >= 3 {
            writeln!(
                stderr,
                "notice: import-wallet: bitcoin-core: {} descriptors share identical script/key material with differing use-site steps — ambiguous receive/change pairing, not merged",
                member_cis.len()
            )
            .map_err(ToolkitError::Io)?;
            continue;
        }
        if member_cis.len() != 2 {
            continue;
        }
        let a = &candidates[member_cis[0]];
        let b = &candidates[member_cis[1]];
        if a.step == b.step || a.internal == b.internal {
            // cond. 4 or cond. 5 failed -- ordinary (non-differentiated)
            // reject via the existing floor once these are left unmerged.
            continue;
        }
        let (recv, chg) = if !a.internal { (a, b) } else { (b, a) };
        let recv_step_pat = format!("/{}/*", recv.step);
        let merged_pat = format!("/<{};{}>/*", recv.step, chg.step);
        // §4.3 uniformity precondition (verified by `analyze_merge_candidate`
        // cond. 7) guarantees every key in `recv.body_no_csum` carries this
        // EXACT suffix; the global replace is therefore all-keys-uniform by
        // construction. Defensive: if the pattern is somehow absent, do NOT
        // merge (fail closed) rather than emit a no-op / wrong descriptor.
        if !recv.body_no_csum.contains(&recv_step_pat) {
            continue;
        }
        let merged_body_no_csum = recv.body_no_csum.replace(&recv_step_pat, &merged_pat);
        let merged_desc_with_csum = recompute_descriptor_checksum(&merged_body_no_csum)?;
        merges.push(Merge {
            first_idx: recv.idx.min(chg.idx),
            second_idx: recv.idx.max(chg.idx),
            desc_with_csum: merged_desc_with_csum,
            active: recv.active || chg.active,
            range: union_range(recv.range, chg.range),
        });
        merged_candidate_idx.insert(member_cis[0]);
        merged_candidate_idx.insert(member_cis[1]);
    }

    // §7 differentiated near-miss: among candidates NOT consumed by a
    // successful merge, any CROSS-GROUP pair with differing steps +
    // disagreeing internal flags is receive/change-SHAPED but for DIFFERENT
    // keys/scripts -- distinct keys are different wallets. Refuse the whole
    // parse loudly rather than let it fall through to the generic per-entry
    // floor reject.
    for i in 0..candidates.len() {
        if merged_candidate_idx.contains(&i) {
            continue;
        }
        for j in (i + 1)..candidates.len() {
            if merged_candidate_idx.contains(&j) {
                continue;
            }
            let a = &candidates[i];
            let b = &candidates[j];
            if a.group == b.group {
                continue; // same-group near-misses are ordinary cond.4/5 rejects
            }
            if a.step != b.step && a.internal != b.internal {
                let lo = a.idx.min(b.idx);
                let hi = a.idx.max(b.idx);
                return Err(ToolkitError::ImportWalletParse(format!(
                    "import-wallet: bitcoin-core: parse error: descriptors[{lo}]/[{hi}] look like a receive/change pair but their keys/origins differ — not merged (distinct keys are different wallets); a fixed single step like /{}/* is un-representable. If these ARE one wallet, combine them by hand to /<a;b>/* and import with --format descriptor.",
                    a.step
                )));
            }
        }
    }

    // Assembly (§4.4 Vec invariant): walk the ORIGINAL descriptors in order;
    // a merged pair emits ONE synthesized entry at the first member's index
    // and the second member's index is skipped entirely. Unpaired entries
    // keep their relative order and an explicit `Some(bool)` internal
    // provenance; a merged entry carries `None`.
    let mut skip_second: HashSet<usize> = HashSet::new();
    let mut merged_values: HashMap<usize, Value> = HashMap::new();
    for m in &merges {
        skip_second.insert(m.second_idx);
        // §4.5 — union the `timestamp`/`next`/`next_index` dropped-field
        // presence from BOTH original members into the synthesized entry so
        // the per-entry `dropped_fields` provenance (computed later, inside
        // `parse_entry`, by scanning `contains_key`) reflects the union.
        let recv_eobj = descriptors[m.first_idx].as_object();
        let chg_eobj = descriptors[m.second_idx].as_object();
        let mut obj = serde_json::Map::new();
        obj.insert("desc".to_string(), json!(m.desc_with_csum));
        obj.insert("active".to_string(), json!(m.active));
        if let Some((lo, hi)) = m.range {
            obj.insert("range".to_string(), json!([lo, hi]));
        }
        for f in ["timestamp", "next", "next_index"] {
            let v = recv_eobj
                .and_then(|o| o.get(f))
                .or_else(|| chg_eobj.and_then(|o| o.get(f)))
                .cloned();
            if let Some(v) = v {
                obj.insert(f.to_string(), v);
            }
        }
        merged_values.insert(m.first_idx, Value::Object(obj));
    }

    let mut out: Vec<PreparedEntry> = Vec::with_capacity(descriptors.len());
    for (idx, entry) in descriptors.into_iter().enumerate() {
        if skip_second.contains(&idx) {
            continue;
        }
        if let Some(merged_value) = merged_values.remove(&idx) {
            out.push(PreparedEntry {
                value: merged_value,
                internal: None,
            });
            continue;
        }
        let internal = match entry.as_object() {
            Some(eobj) => Some(parse_bool_field(eobj, "internal")?),
            None => None, // `parse_entry` raises "is not an object" downstream.
        };
        out.push(PreparedEntry {
            value: entry,
            internal,
        });
    }
    Ok(out)
}

fn parse_entry(
    idx: usize,
    entry: &Value,
    wallet_name: Option<String>,
    internal: Option<bool>,
) -> Result<ParsedImport, ToolkitError> {
    let eobj = entry.as_object().ok_or_else(|| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: bitcoin-core: parse error: descriptors[{idx}] is not an object"
        ))
    })?;

    let desc_with_csum = eobj
        .get("desc")
        .and_then(|d| d.as_str())
        .ok_or_else(|| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: bitcoin-core: parse error: descriptors[{idx}].desc is missing or not a string"
            ))
        })?;

    // SPEC §5.2 step 2.a (Phase 3 R0 architect C1+I1 folds): refuse any
    // extended-private-key prefix, not just literal "xprv". Bitcoin Core's
    // `listdescriptors true` on testnet/signet/regtest emits `tprv`; SLIP-132
    // defines `yprv|Yprv|zprv|Zprv|uprv|Uprv|vprv|Vprv` private-key prefix
    // variants. None were caught by the prior `contains("xprv")` check.
    // Strip the BIP-380 `#<csum>` trailer before the substring scan so the
    // checksum's bech32-style alphabet (which can contain the 4-char run
    // `xprv` stochastically at probability ~5e-6 per descriptor) cannot
    // false-positive a benign xpub descriptor.
    let body_for_xprv_check = match desc_with_csum.rsplit_once('#') {
        Some((body, _csum)) => body,
        None => desc_with_csum,
    };
    if xprv_prefix_regex().is_match(body_for_xprv_check) {
        return Err(ToolkitError::ImportWalletXprvForbidden);
    }

    // SPEC §5.2 step 2.b: same adapter + parse_descriptor pipeline as BSMS.
    // Validate the BIP-380 checksum up-front via miniscript so a bad
    // checksum surfaces as ImportWalletParse rather than a downstream
    // DescriptorParse (consistent with BSMS error template).
    let descriptor_body_no_csum = miniscript::descriptor::checksum::verify_checksum(
        desc_with_csum,
    )
    .map_err(|e| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: bitcoin-core: parse error: descriptors[{idx}]: BIP-380 checksum validation failed: {e}"
        ))
    })?;

    let (placeholder_form, parsed_keys, parsed_fingerprints) =
        concrete_keys_to_placeholders(descriptor_body_no_csum).map_err(|e| {
            // Re-tag the BSMS error template prefix as bitcoin-core for the
            // user-facing message.
            ToolkitError::ImportWalletParse(e.message().replacen(
                "import-wallet: bsms:",
                "import-wallet: bitcoin-core:",
                1,
            ))
        })?;

    let descriptor =
        parse_descriptor::parse_descriptor(&placeholder_form, &parsed_keys, &parsed_fingerprints)
            .map_err(|e| {
            ToolkitError::ImportWalletParse(format!(
                "import-wallet: bitcoin-core: parse error: descriptors[{idx}]: {}",
                e.message()
            ))
        })?;

    let origins = crate::wallet_import::pipeline::extract_origin_components(
        descriptor_body_no_csum,
        "bitcoin-core",
    )?;
    let network = network_from_origins(&origins, idx)?;

    let mut cosigners: Vec<ResolvedSlot> = Vec::with_capacity(parsed_keys.len());
    for (slot_idx, _) in parsed_keys.iter().enumerate() {
        let (xpub, fp, path) = build_slot_fields(descriptor_body_no_csum, slot_idx, idx)?;
        debug_assert_eq!(xpub_to_65(&xpub), parsed_keys[slot_idx].payload);
        cosigners.push(ResolvedSlot {
            xpub,
            fingerprint: fp,
            path,
            entropy: None,
            master_xpub: None,
            language: None,
            _entropy_pin: None,
        });
    }

    validate_watch_only_resolved(&cosigners)?;

    // cycle-5 S-NET (axis 2 / H15): per-entry, each decoded xpub's NetworkKind
    // must agree with this entry's coin-type-derived network.
    crate::wallet_import::pipeline::assert_slots_network_agrees(
        &cosigners,
        network,
        "import: bitcoin-core",
    )?;

    let threshold = extract_threshold(descriptor_body_no_csum)?;

    // v0.27.1 Phase 2 I4 fold: distinguish "absent" (default false) from
    // "shape-wrong" (typed parse error). The prior pattern
    // `.and_then(.as_bool).unwrap_or(false)` silently flipped non-bool inputs
    // ("active": "true", `1`, etc.) to false, which downstream
    // `--select-descriptor active-*` reported as "no active-* descriptor
    // found" — a misleading user-facing error. Mirrors `parse_range_field`'s
    // shape-strictness precedent.
    let active = parse_bool_field(eobj, "active")?;
    // `internal` is now threaded explicitly by the caller (see `parse`'s loop
    // and, from P1, `merge_receive_change_pairs`) rather than read here —
    // `Some(bool)` for a passthrough entry, `None` for a pre-pass-merged
    // entry. NEVER inferred from the multipath shape of `desc` itself.
    let range = parse_range_field(eobj.get("range"))?;

    let mut dropped_fields: Vec<String> = Vec::new();
    for f in ["timestamp", "next", "next_index"] {
        if eobj.contains_key(f) {
            dropped_fields.push(f.to_string());
        }
    }

    let source_metadata = CoreSourceMetadata {
        active,
        internal,
        range,
        dropped_fields,
        wallet_name,
    };

    Ok(ParsedImport {
        descriptor,
        original_descriptor: desc_with_csum.to_string(),
        cosigners,
        network,
        threshold,
        provenance: ImportProvenance::BitcoinCore(source_metadata),
    })
}

/// Decode the optional `range` field — Bitcoin Core emits a 2-element integer
/// array `[lo, hi]`. Returns `Ok(None)` if absent (Core may omit `range` for
/// non-ranged descriptors); errors if the shape is unexpected.
/// v0.27.1 Phase 2 I4 helper. Mirrors `parse_range_field`'s shape-strictness:
/// absent or `null` → `Ok(false)` (default); present + non-bool → `Err` with
/// pointer text naming the field.
fn parse_bool_field(
    eobj: &serde_json::Map<String, Value>,
    field: &str,
) -> Result<bool, ToolkitError> {
    match eobj.get(field) {
        None => Ok(false),
        Some(Value::Null) => Ok(false),
        Some(Value::Bool(b)) => Ok(*b),
        Some(other) => Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: bitcoin-core: parse error: `{field}` must be boolean, got {}",
            kind_of(other)
        ))),
    }
}

/// Compact JSON type label used by `parse_bool_field` error templates.
fn kind_of(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn parse_range_field(v: Option<&Value>) -> Result<Option<(u64, u64)>, ToolkitError> {
    let v = match v {
        Some(v) => v,
        None => return Ok(None),
    };
    if v.is_null() {
        return Ok(None);
    }
    let arr = v.as_array().ok_or_else(|| {
        ToolkitError::ImportWalletParse(
            "import-wallet: bitcoin-core: parse error: `range` must be a [lo, hi] array"
                .to_string(),
        )
    })?;
    if arr.len() != 2 {
        return Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: bitcoin-core: parse error: `range` must have exactly 2 elements, got {}",
            arr.len()
        )));
    }
    let lo = arr[0].as_u64().ok_or_else(|| {
        ToolkitError::ImportWalletParse(
            "import-wallet: bitcoin-core: parse error: `range[0]` must be a non-negative integer"
                .to_string(),
        )
    })?;
    let hi = arr[1].as_u64().ok_or_else(|| {
        ToolkitError::ImportWalletParse(
            "import-wallet: bitcoin-core: parse error: `range[1]` must be a non-negative integer"
                .to_string(),
        )
    })?;
    Ok(Some((lo, hi)))
}

fn build_slot_fields(
    descriptor_body: &str,
    slot_idx: usize,
    entry_idx: usize,
) -> Result<(Xpub, Fingerprint, DerivationPath), ToolkitError> {
    let origins =
        crate::wallet_import::pipeline::extract_origin_components(descriptor_body, "bitcoin-core")?;
    let (fp, path, xpub_str) = origins.into_iter().nth(slot_idx).ok_or_else(|| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: bitcoin-core: parse error: descriptors[{entry_idx}]: slot index {slot_idx} out of range"
        ))
    })?;
    crate::wallet_import::pipeline::finalize_slot_fields(fp, path, &xpub_str, "bitcoin-core")
}

fn network_from_origins(
    origins: &[(Fingerprint, DerivationPath, String)],
    entry_idx: usize,
) -> Result<bitcoin::Network, ToolkitError> {
    if origins.is_empty() {
        return Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: bitcoin-core: parse error: descriptors[{entry_idx}]: no origins to infer network from"
        )));
    }
    let coin_types: Vec<u32> = origins
        .iter()
        .map(|(_, p, _)| coin_type_from_path(p, entry_idx))
        .collect::<Result<Vec<_>, _>>()?;
    let first = coin_types[0];
    for (i, ct) in coin_types.iter().enumerate().skip(1) {
        if *ct != first {
            return Err(ToolkitError::ImportWalletParse(format!(
                "import-wallet: bitcoin-core: descriptors[{entry_idx}]: cosigner {i} has coin-type {ct}, cosigner 0 has coin-type {first}; all cosigners must share a coin-type"
            )));
        }
    }
    match first {
        0 => Ok(bitcoin::Network::Bitcoin),
        1 => Ok(bitcoin::Network::Testnet),
        other => Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: bitcoin-core: parse error: descriptors[{entry_idx}]: unsupported coin-type {other} on origin path; only 0 (mainnet) and 1 (testnet) supported per BIP-48"
        ))),
    }
}

fn coin_type_from_path(path: &DerivationPath, entry_idx: usize) -> Result<u32, ToolkitError> {
    let comps: Vec<&ChildNumber> = path.into_iter().collect();
    if comps.len() < 2 {
        return Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: bitcoin-core: parse error: descriptors[{entry_idx}]: origin path has only {} components; need ≥2 for BIP-48 coin-type inference",
            comps.len()
        )));
    }
    match comps[1] {
        ChildNumber::Hardened { index } => Ok(*index),
        ChildNumber::Normal { index } => Err(ToolkitError::ImportWalletParse(format!(
            "import-wallet: bitcoin-core: parse error: descriptors[{entry_idx}]: coin-type component {index} is not hardened; BIP-48 requires `<coin_type>'`"
        ))),
    }
}

/// Extract K from `thresh(K, ...)` / `multi(K, ...)` / `sortedmulti(K, ...)`
/// at the top-level miniscript context. Returns `Ok(None)` for single-key
/// shapes (no thresh/multi token found); `Err` for u8 overflow.
///
/// v0.27.1 Phase 2 I6 fold: previously returned `Option<u8>`, silently
/// mapping u8 overflow (e.g. `thresh(256, …)`) to `None` — which downstream
/// rendered as `"threshold": null`, presenting a "no-threshold" descriptor
/// when the input was actually malformed. Now distinguishes "no thresh token"
/// from "thresh argument failed u8 parse" via the typed Result.
///
/// Mirrors `bsms::extract_threshold`.
pub(super) fn extract_threshold(descriptor_body: &str) -> Result<Option<u8>, ToolkitError> {
    static R: OnceLock<Regex> = OnceLock::new();
    let re = R.get_or_init(|| {
        Regex::new(r"(?:thresh|multi|sortedmulti)\((\d+)\s*,").expect("threshold regex is fixed")
    });
    let cap = match re.captures(descriptor_body) {
        Some(c) => c,
        None => return Ok(None),
    };
    let arg = cap.get(1).expect("regex has capture group 1").as_str();
    arg.parse::<u8>().map(Some).map_err(|e| {
        ToolkitError::ImportWalletParse(format!(
            "import-wallet: bitcoin-core: parse error: thresh/multi argument `{arg}` exceeds u8 range (>255 cosigners not supported): {e}"
        ))
    })
}

/// Match any extended-private-key prefix per BIP-32 + SLIP-132 (Phase 3 R0
/// architect C1 fold). Mainnet `xprv`, testnet `tprv`, SLIP-132
/// `yprv|Yprv|zprv|Zprv|uprv|Uprv|vprv|Vprv`. The trailing
/// `[A-HJ-NP-Za-km-z1-9]+` ensures we match an actual base58check key body
/// rather than the literal 4-char prefix substring (BIP-380 checksum
/// false-positive guard, I1 fold).
fn xprv_prefix_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"[xtyzuvYZUV]prv[A-HJ-NP-Za-km-z1-9]+")
            .expect("xprv_prefix_regex is a fixed string literal")
    })
}

fn trim_leading_ws(blob: &[u8]) -> &[u8] {
    let mut i = 0;
    while i < blob.len()
        && (blob[i] == b' ' || blob[i] == b'\t' || blob[i] == b'\n' || blob[i] == b'\r')
    {
        i += 1;
    }
    &blob[i..]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// v0.27.1 Phase 2 R0 M1 fold: guarantee coverage of the
    /// `extract_threshold` u8-overflow branch. Mirrors `bsms::tests` cell.
    #[test]
    fn extract_threshold_u8_overflow_is_typed_error() {
        // Body without thresh/multi → Ok(None).
        let r = extract_threshold("wpkh(@0)").unwrap();
        assert_eq!(r, None);

        // Body with multi(2,…) → Ok(Some(2)).
        let r = extract_threshold("sh(multi(2,@0,@1,@2))").unwrap();
        assert_eq!(r, Some(2));

        // Body with sortedmulti(256,…) → Err (u8 overflow).
        let err = extract_threshold("wsh(sortedmulti(256,@0,@1))").unwrap_err();
        let msg = format!("{err:?}");
        assert!(
            msg.contains("exceeds u8 range") && msg.contains("256"),
            "expected u8-overflow diagnostic naming 256; got: {msg}"
        );
    }

    /// SPEC §6.1 item 2: sniff predicate smoke. Pins behavior for the cases
    /// where sniff must return true vs. false. Used by Phase 5's sniff
    /// dispatcher.
    #[test]
    fn sniff_true_on_minimal_core_blob() {
        let blob = br#"{"wallet_name":"x","descriptors":[{"desc":"wpkh(xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/<0;1>/*)#00000000"}]}"#;
        assert!(BitcoinCoreParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_bsms_blob() {
        let blob = b"BSMS 1.0\nwsh(pk(deadbeef))#00000000\n";
        assert!(!BitcoinCoreParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_specter_blob() {
        // Top-level `chain` is a Specter vendor-marker key per VENDOR_MARKER_KEYS.
        let blob = br#"{"chain":"main","descriptor":"wpkh(xpub...)","label":"daily","devices":["unknown"]}"#;
        assert!(!BitcoinCoreParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_empty_descriptors_array() {
        let blob = br#"{"descriptors":[]}"#;
        assert!(!BitcoinCoreParser::sniff(blob));
    }

    #[test]
    fn sniff_false_on_non_object_top_level() {
        assert!(!BitcoinCoreParser::sniff(b"[1, 2, 3]"));
    }

    #[test]
    fn sniff_false_on_entry_missing_desc() {
        let blob = br#"{"descriptors":[{"timestamp":42}]}"#;
        assert!(!BitcoinCoreParser::sniff(blob));
    }

    /// PLAN P1 "cheap insurance" — `Descriptor::<DescriptorPublicKey>::
    /// from_str` must succeed on each in-scope merge-candidate shape (wpkh /
    /// wsh(sortedmulti) / sh(wsh) / single-key bip86 tr) BEFORE
    /// `analyze_merge_candidate` relies on parsed fields. A parse failure
    /// here would silently degrade `analyze_merge_candidate` to `None`
    /// (via `.ok()?`) for an entire in-scope shape class, making the merge
    /// pre-pass vacuous for that shape without any test ever going RED.
    #[test]
    fn in_scope_merge_candidate_shapes_parse_via_rust_miniscript() {
        let wpkh = "wpkh([b8688df1/84'/0'/0']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/0/*)";
        let wsh_sortedmulti = "wsh(sortedmulti(2,[b8688df1/48'/0'/0'/2']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/0/*,[28645006/48'/0'/0'/2']xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6/0/*))";
        let sh_wsh = "sh(wsh(sortedmulti(2,[b8688df1/48'/0'/0'/1']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/0/*,[28645006/48'/0'/0'/1']xpub6DnEBNkSJKBYQmsbhS1sP9cNdtU5c9PLFGCjTJmxicxc13WB8zNNGQazabQpyFAGW5bV9tMko4uBxDxjUKL6dSAcx1tEbgEHtgSqyRsekh6/0/*)))";
        let tr_bip86 = "tr([b8688df1/86'/0'/0']xpub6FQya7zGhR92kacYsNnjreouvnHJMpXYsUXnW6NJJAJRCKsa26TzDy4LdnGhEurr3d6y1J8PJ7EEMKQp74XTqYvmGJNogYXSKDszYHtF8mX/0/*)";
        for (label, body) in [
            ("wpkh", wpkh),
            ("wsh(sortedmulti)", wsh_sortedmulti),
            ("sh(wsh(sortedmulti))", sh_wsh),
            ("tr bip86", tr_bip86),
        ] {
            MsDescriptor::<DescriptorPublicKey>::from_str(body)
                .unwrap_or_else(|e| panic!("{label} must parse via rust-miniscript: {e}"));
            assert!(
                analyze_merge_candidate(body).is_some(),
                "{label} must be classified as a merge candidate: {body}"
            );
        }
    }
}
