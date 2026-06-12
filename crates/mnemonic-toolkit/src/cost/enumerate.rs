//! Minimal-satisfying-configuration enumeration over a translated miniscript.
//! SPEC §3.
//!
//! Substrate: rust-miniscript v13's `Descriptor<DefiniteDescriptorKey>::plan(
//! self, &Assets)` API. `Assets` is a builder-style `AssetProvider` impl
//! provided by miniscript; we construct one per candidate configuration and
//! feed it to `plan()`. `plan` consumes `self`, so we clone the descriptor
//! per iteration (and per shrinkage check in the minimality pass).
//!
//! Configuration = (signing_keys subset × known_preimages subset × per-kind
//! abs/rel timelock-state). The eager precheck (SPEC §3.3 step 1) refuses
//! enumeration up-front when
//! `n_abs × n_rel × 2^(|signers| + |preimages|) > hard_cap`, where `n_abs` /
//! `n_rel` are 1..=3 depending on which timelock kinds (height vs MTP-time
//! abs, blocks vs 512s rel) appear in the AST.

use std::collections::BTreeSet;

use miniscript::bitcoin::hashes::{hash160, ripemd160, sha256};
use miniscript::bitcoin::{absolute, relative};
use miniscript::descriptor::{DefiniteDescriptorKey, DescriptorPublicKey};
use miniscript::plan::Assets;
use miniscript::{Descriptor, ForEachKey, Miniscript, Segwitv0};

use super::translate::{pubkey_to_label_segv0, Translated};
use super::CompareCostError;

/// Per-condition row produced by enumeration.
#[derive(Debug, Clone)]
pub struct Row {
    pub label: String,
    pub wsh_witness_bytes: usize,
    pub tr_witness_bytes: usize,
}

/// Result of enumeration.
pub struct EnumerationReport {
    pub rows: Vec<Row>,
    pub soft_cap_reached: bool,
    /// `true` if the input policy contained hash fragments
    /// (sha256/hash256/ripemd160/hash160). Drives a `notes[]` advisory.
    pub has_hash_fragments: bool,
}

/// Soft warn-trail threshold; see SPEC §3.3 step 7.
const SOFT_THRESHOLD: usize = 256;

/// SegWit per-input base weight (constant across wrappers, per SPEC §4).
/// = 36-byte outpoint + 1-byte scriptSig-length-zero + 4-byte sequence, ×4 wu/B.
pub const SEGWIT_INPUT_BASE_WU: usize = 164;

/// Hash-leaf assets the user "knows preimages of". Each variant is enumerated
/// over the powerset of hashes-known.
#[derive(Debug, Clone)]
enum HashAsset {
    Sha256(sha256::Hash),
    Hash256(miniscript::hash256::Hash),
    Ripemd160(ripemd160::Hash),
    Hash160(hash160::Hash),
}

/// Collected AST assets — keys (per-context), hashes, timelock fragments.
struct AstAssets {
    /// Compressed-secp keys for the Segwitv0 descriptor's Assets, in AST
    /// left-to-right order of first occurrence.
    segv0_keys: Vec<DescriptorPublicKey>,
    /// x-only keys for the Tap descriptor's Assets, same indexing as
    /// `segv0_keys` (label[i] → segv0_keys[i] and tap_keys[i]).
    tap_keys: Vec<DescriptorPublicKey>,
    /// Hashes, in AST left-to-right order of first occurrence.
    hashes: Vec<HashAsset>,
    /// `true` if the AST has any block-height-kind `after(N<500_000_000)`.
    has_abs_height: bool,
    /// `true` if the AST has any MTP-time-kind `after(N≥500_000_000)`.
    has_abs_time: bool,
    /// `true` if the AST has any block-height-kind `older(N)` (TIME_LOCK_FLAG clear).
    has_rel_blocks: bool,
    /// `true` if the AST has any 512s-time-kind `older(N|TIME_LOCK_FLAG)`.
    has_rel_512s: bool,
}

pub fn enumerate_minimal_conditions(
    translated: &Translated,
    wsh_desc: &Descriptor<DefiniteDescriptorKey>,
    tr_desc: &Descriptor<DefiniteDescriptorKey>,
    hard_cap: usize,
) -> Result<EnumerationReport, CompareCostError> {
    let assets = collect_ast_assets(translated);
    let n_keys = assets.segv0_keys.len();
    let n_hashes = assets.hashes.len();
    debug_assert_eq!(
        n_keys,
        assets.tap_keys.len(),
        "segv0 and tap key counts must match (post-translation)"
    );

    // Per-axis timelock states. Each axis enumerates: (None) plus
    // (height-saturated) iff that kind is in the AST plus (time-saturated)
    // iff that kind is in the AST. rust-miniscript's `is_implied_by` returns
    // false on mismatched units (block-height vs MTP-time, or rel-blocks vs
    // rel-512s), so we must saturate per-kind.
    let abs_states: Vec<AbsState> = std::iter::once(AbsState::None)
        .chain(assets.has_abs_height.then_some(AbsState::HeightSaturated))
        .chain(assets.has_abs_time.then_some(AbsState::TimeSaturated))
        .collect();
    let rel_states: Vec<RelState> = std::iter::once(RelState::None)
        .chain(assets.has_rel_blocks.then_some(RelState::BlocksSaturated))
        .chain(assets.has_rel_512s.then_some(RelState::Time512sSaturated))
        .collect();

    // SPEC §3.3 step 1 — eager combinatorial precheck.
    let n_tl_states: usize = abs_states.len() * rel_states.len();
    let pow = 2_usize.checked_pow((n_keys + n_hashes) as u32).ok_or(
        CompareCostError::ConditionsTooMany {
            raw: usize::MAX,
            cap: hard_cap,
        },
    )?;
    let raw = n_tl_states
        .checked_mul(pow)
        .ok_or(CompareCostError::ConditionsTooMany {
            raw: usize::MAX,
            cap: hard_cap,
        })?;
    if raw > hard_cap {
        return Err(CompareCostError::ConditionsTooMany { raw, cap: hard_cap });
    }

    let mut rows: Vec<Row> = Vec::new();
    let mut soft_reached = false;
    // SPEC §3.3 step 7: soft warn-trail fires at `min(SOFT_THRESHOLD, hard_cap)`
    // — when the user sets --max-conditions below the soft threshold, the
    // advisory shouldn't be unreachable.
    let soft_threshold = std::cmp::min(SOFT_THRESHOLD, hard_cap);

    'outer: for key_mask in 0..(1u64 << n_keys) {
        for hash_mask in 0..(1u64 << n_hashes) {
            for &abs_state in &abs_states {
                for &rel_state in &rel_states {
                    let cfg = Config {
                        key_mask,
                        hash_mask,
                        abs_state,
                        rel_state,
                    };

                    let wsh_ws = build_and_plan(wsh_desc, &assets, &assets.segv0_keys, &cfg);
                    let tr_ws = build_and_plan(tr_desc, &assets, &assets.tap_keys, &cfg);
                    let (Some(wb), Some(tb)) = (wsh_ws, tr_ws) else {
                        continue;
                    };

                    if is_minimal(wsh_desc, tr_desc, &assets, &cfg) {
                        let label = label_config(&cfg, &assets, translated);
                        rows.push(Row {
                            label,
                            wsh_witness_bytes: wb,
                            tr_witness_bytes: tb,
                        });
                        if rows.len() >= soft_threshold {
                            soft_reached = true;
                        }
                        if rows.len() >= hard_cap {
                            break 'outer;
                        }
                    }
                }
            }
        }
    }

    if rows.is_empty() {
        return Err(CompareCostError::NoSatisfyingConditions);
    }

    Ok(EnumerationReport {
        rows,
        soft_cap_reached: soft_reached,
        has_hash_fragments: n_hashes > 0,
    })
}

/// Absolute-locktime saturation kind for one configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AbsState {
    /// No `after()` is satisfied (or the script has no abs timelock).
    None,
    /// Saturated block-height — satisfies `after(N<500_000_000)`.
    HeightSaturated,
    /// Saturated MTP-time — satisfies `after(N≥500_000_000)`.
    TimeSaturated,
}

/// Relative-locktime saturation kind for one configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RelState {
    /// No `older()` is satisfied (or the script has no rel timelock).
    None,
    /// Saturated rel-blocks — satisfies `older(N)` where TIME_LOCK_FLAG clear.
    BlocksSaturated,
    /// Saturated rel-512s-intervals — satisfies `older(N|TIME_LOCK_FLAG)`.
    Time512sSaturated,
}

#[derive(Debug, Clone, Copy)]
struct Config {
    key_mask: u64,
    hash_mask: u64,
    abs_state: AbsState,
    rel_state: RelState,
}

fn build_and_plan(
    desc: &Descriptor<DefiniteDescriptorKey>,
    assets: &AstAssets,
    keys_for_ctx: &[DescriptorPublicKey],
    cfg: &Config,
) -> Option<usize> {
    let mut a = Assets::new();
    for (i, key) in keys_for_ctx.iter().enumerate() {
        if cfg.key_mask & (1u64 << i) != 0 {
            a = a.add(key.clone());
        }
    }
    for (i, hash) in assets.hashes.iter().enumerate() {
        if cfg.hash_mask & (1u64 << i) != 0 {
            a = match hash {
                HashAsset::Sha256(h) => a.add(*h),
                HashAsset::Hash256(h) => a.add(*h),
                HashAsset::Ripemd160(h) => a.add(*h),
                HashAsset::Hash160(h) => a.add(*h),
            };
        }
    }
    match cfg.abs_state {
        AbsState::None => {}
        AbsState::HeightSaturated => {
            // Saturated block-height — satisfies `after(N<500_000_000)`.
            // `absolute::LockTime::from_height` requires a valid block height
            // (i.e., < THRESHOLD = 500_000_000). The largest valid height is
            // `499_999_999`. `from_height(u32)` takes a u32, returns Result.
            if let Ok(lt) = absolute::LockTime::from_height(499_999_999) {
                a = a.after(lt);
            }
        }
        AbsState::TimeSaturated => {
            // Saturated MTP-time — satisfies `after(N≥500_000_000)`.
            // `absolute::LockTime::from_time` requires `n >= 500_000_000`.
            if let Ok(lt) = absolute::LockTime::from_time(u32::MAX) {
                a = a.after(lt);
            }
        }
    }
    match cfg.rel_state {
        RelState::None => {}
        RelState::BlocksSaturated => {
            // Saturated rel-blocks — satisfies any `older(N)` with
            // TIME_LOCK_FLAG (bit 22) clear.
            a = a.older(relative::LockTime::from_height(0xFFFF));
        }
        RelState::Time512sSaturated => {
            // Saturated rel-512s — satisfies any `older(N|TIME_LOCK_FLAG)`.
            a = a.older(relative::LockTime::from_512_second_intervals(0xFFFF));
        }
    }
    match desc.clone().plan(&a) {
        Ok(plan) => Some(plan.witness_size()),
        Err(_) => None,
    }
}

fn is_minimal(
    wsh_desc: &Descriptor<DefiniteDescriptorKey>,
    tr_desc: &Descriptor<DefiniteDescriptorKey>,
    assets: &AstAssets,
    cfg: &Config,
) -> bool {
    // Drop each key in turn.
    let n_keys = assets.segv0_keys.len();
    for i in 0..n_keys {
        if cfg.key_mask & (1u64 << i) == 0 {
            continue;
        }
        let smaller = Config {
            key_mask: cfg.key_mask & !(1u64 << i),
            ..*cfg
        };
        if both_plans_ok(wsh_desc, tr_desc, assets, &smaller) {
            return false;
        }
    }
    // Drop each hash.
    let n_hashes = assets.hashes.len();
    for i in 0..n_hashes {
        if cfg.hash_mask & (1u64 << i) == 0 {
            continue;
        }
        let smaller = Config {
            hash_mask: cfg.hash_mask & !(1u64 << i),
            ..*cfg
        };
        if both_plans_ok(wsh_desc, tr_desc, assets, &smaller) {
            return false;
        }
    }
    // Drop abs-timelock saturation (if currently active).
    if cfg.abs_state != AbsState::None {
        let smaller = Config {
            abs_state: AbsState::None,
            ..*cfg
        };
        if both_plans_ok(wsh_desc, tr_desc, assets, &smaller) {
            return false;
        }
    }
    // Drop rel-timelock saturation (if currently active).
    if cfg.rel_state != RelState::None {
        let smaller = Config {
            rel_state: RelState::None,
            ..*cfg
        };
        if both_plans_ok(wsh_desc, tr_desc, assets, &smaller) {
            return false;
        }
    }
    true
}

fn both_plans_ok(
    wsh_desc: &Descriptor<DefiniteDescriptorKey>,
    tr_desc: &Descriptor<DefiniteDescriptorKey>,
    assets: &AstAssets,
    cfg: &Config,
) -> bool {
    build_and_plan(wsh_desc, assets, &assets.segv0_keys, cfg).is_some()
        && build_and_plan(tr_desc, assets, &assets.tap_keys, cfg).is_some()
}

fn label_config(cfg: &Config, assets: &AstAssets, translated: &Translated) -> String {
    let mut parts: Vec<String> = Vec::new();
    let n_keys = assets.segv0_keys.len();
    for i in 0..n_keys {
        if cfg.key_mask & (1u64 << i) != 0 {
            let pk = &assets.segv0_keys[i];
            // Try to map back to user label (Segwitv0 form). If not found
            // (concrete key input), use a stable identifier.
            if let Some(lbl) = pubkey_to_label_segv0(pk, translated) {
                parts.push(lbl.to_string());
            } else {
                parts.push(format!("key[{i}]"));
            }
        }
    }
    let n_hashes = assets.hashes.len();
    for i in 0..n_hashes {
        if cfg.hash_mask & (1u64 << i) != 0 {
            parts.push(format!("preimage(h{i})"));
        }
    }
    match cfg.abs_state {
        AbsState::None => {}
        AbsState::HeightSaturated => parts.push("after(height)".to_string()),
        AbsState::TimeSaturated => parts.push("after(time)".to_string()),
    }
    match cfg.rel_state {
        RelState::None => {}
        RelState::BlocksSaturated => parts.push("older(blocks)".to_string()),
        RelState::Time512sSaturated => parts.push("older(512s)".to_string()),
    }
    if parts.is_empty() {
        "(none)".to_string()
    } else {
        parts.join(" + ")
    }
}

fn collect_ast_assets(translated: &Translated) -> AstAssets {
    // Walk both descriptor variants since they carry context-specific pubkey
    // serializations (Segwitv0 = compressed 33B; Tap = x-only 32B). The key
    // ORDER is identical post-translation since both come from the same
    // user-label-substituted AST shape.
    let mut segv0_set: BTreeSet<DescriptorPublicKey> = BTreeSet::new();
    let mut segv0_order: Vec<DescriptorPublicKey> = Vec::new();
    translated.segv0.for_each_key(|k| {
        let dpk: DescriptorPublicKey = k.clone().into();
        if segv0_set.insert(dpk.clone()) {
            segv0_order.push(dpk);
        }
        true
    });
    let mut tap_set: BTreeSet<DescriptorPublicKey> = BTreeSet::new();
    let mut tap_order: Vec<DescriptorPublicKey> = Vec::new();
    translated.tap.for_each_key(|k| {
        let dpk: DescriptorPublicKey = k.clone().into();
        if tap_set.insert(dpk.clone()) {
            tap_order.push(dpk);
        }
        true
    });

    // I3 fold: explicit order-stability check — keys at the same index in
    // both walks must correspond to the same user label. This holds when
    // for_each_key walks the AST in the same order across contexts (the AST
    // shape is identical post-translation; only key serializations differ).
    debug_assert_eq!(segv0_order.len(), tap_order.len());
    debug_assert!(
        (0..segv0_order.len()).all(|i| {
            super::translate::pubkey_to_label_segv0(&segv0_order[i], translated)
                == super::translate::pubkey_to_label_tap(&tap_order[i], translated)
        }),
        "segv0 + tap key walks must yield labels in the same order"
    );

    let mut hashes: Vec<HashAsset> = Vec::new();
    walk_segv0_for_hash_leaves(&translated.segv0, &mut hashes);
    let (has_abs_h, has_abs_t) = walk_absolute_timelock_kinds(&translated.segv0);
    let (has_rel_b, has_rel_t) = walk_relative_timelock_kinds(&translated.segv0);

    AstAssets {
        segv0_keys: segv0_order,
        tap_keys: tap_order,
        hashes,
        has_abs_height: has_abs_h,
        has_abs_time: has_abs_t,
        has_rel_blocks: has_rel_b,
        has_rel_512s: has_rel_t,
    }
}

fn walk_segv0_for_hash_leaves(
    m: &Miniscript<DefiniteDescriptorKey, Segwitv0>,
    out: &mut Vec<HashAsset>,
) {
    use miniscript::miniscript::decode::Terminal;
    fn visit<Pk, Ctx: miniscript::ScriptContext>(t: &Terminal<Pk, Ctx>, out: &mut Vec<HashAsset>)
    where
        Pk: miniscript::MiniscriptKey<Sha256 = sha256::Hash>
            + miniscript::MiniscriptKey<Hash256 = miniscript::hash256::Hash>
            + miniscript::MiniscriptKey<Ripemd160 = ripemd160::Hash>
            + miniscript::MiniscriptKey<Hash160 = hash160::Hash>,
    {
        match t {
            Terminal::Sha256(h) => out.push(HashAsset::Sha256(*h)),
            Terminal::Hash256(h) => out.push(HashAsset::Hash256(*h)),
            Terminal::Ripemd160(h) => out.push(HashAsset::Ripemd160(*h)),
            Terminal::Hash160(h) => out.push(HashAsset::Hash160(*h)),
            _ => {}
        }
    }
    for sub in m.iter() {
        visit::<DefiniteDescriptorKey, Segwitv0>(&sub.node, out);
    }
}

/// Walk the AST and report `(has_block_height_absolute, has_mtp_time_absolute)`
/// for `after(...)` fragments. The two kinds use different units and cannot
/// be co-satisfied by a single `Assets::after(_)` value, so the enumeration
/// must consider them separately.
fn walk_absolute_timelock_kinds(m: &Miniscript<DefiniteDescriptorKey, Segwitv0>) -> (bool, bool) {
    use miniscript::miniscript::decode::Terminal;
    let mut h = false;
    let mut t = false;
    for sub in m.iter() {
        if let Terminal::After(abs) = sub.node {
            if abs.is_block_height() {
                h = true;
            } else if abs.is_block_time() {
                t = true;
            }
        }
    }
    (h, t)
}

/// Walk the AST and report `(has_rel_blocks, has_rel_512s)` for `older(...)`
/// fragments. Same per-kind separation rationale as abs.
fn walk_relative_timelock_kinds(m: &Miniscript<DefiniteDescriptorKey, Segwitv0>) -> (bool, bool) {
    use miniscript::miniscript::decode::Terminal;
    let mut b = false;
    let mut t = false;
    for sub in m.iter() {
        if let Terminal::Older(rel) = sub.node {
            if rel.is_height_locked() {
                b = true;
            } else if rel.is_time_locked() {
                t = true;
            }
        }
    }
    (b, t)
}
