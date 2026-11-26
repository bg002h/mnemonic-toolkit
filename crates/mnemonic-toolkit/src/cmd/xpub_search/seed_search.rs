//! Search engine for `xpub-search address-of-seed`
//! (`design/SPEC_xpub_search_address_of_seed.md` §4, §4.1, §5).
//!
//! - [`Template`]: a `@N` template parsed ONCE per script form (structure
//!   once); per candidate the cached child keys are translated into it and
//!   the scriptPubKey is looked up in the target set. No descriptor string is
//!   built or parsed per candidate (§4.1 item 3).
//! - [`Space`]: the (path tuple, slot order) enumeration of §4.1 item 2. Rank
//!   `r ∈ [0, T × S)`: `t = r div S`, `σ = unrank_permutation(r mod S, f)`;
//!   segments are concatenated (the shared or independent tuples, then the
//!   Refugium seat tuples).
//! - [`scan`]: collect-all, no early exit (§4.1 item 1). Every hit is
//!   counted; the kept list is capped. [`scan_reference`] is the
//!   single-threaded reference the parallel scan is tested against.
//! - [`Expr`]: a small expression tree over the template text, used for the
//!   sorted collapse test (pruning 4), the twin (§5) and the symmetry merge.

use crate::parse_descriptor::{
    expand_literal_double_star, lex_placeholders, parse_descriptor_lenient, resolve_placeholders,
    substitute_nums_sentinel, substitute_synthetic, ScriptCtx, NUMS_H_POINT_X_ONLY_HEX,
};
use crate::permutation_search::{factorial, unrank_permutation};
use bitcoin::bip32::{ChildNumber, Xpub};
use bitcoin::secp256k1::{Secp256k1, Verification};
use bitcoin::{PublicKey, Script, ScriptBuf};
use miniscript::descriptor::SinglePubKey;
use miniscript::{
    translate_hash_clone, Descriptor as MsDescriptor, DescriptorPublicKey, ToPublicKey, Translator,
};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Output kinds (pruning 3)
// ---------------------------------------------------------------------------

/// The scriptPubKey class a target decodes to and a template produces.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum SpkKind {
    P2pkh,
    P2sh,
    P2wpkh,
    P2wsh,
    P2tr,
}

impl SpkKind {
    /// Classify by the decoded scriptPubKey (witness version and program
    /// length, or the base58 form), never by an address prefix (§2).
    pub fn of_script(s: &Script) -> Option<Self> {
        if s.is_p2pkh() {
            Some(Self::P2pkh)
        } else if s.is_p2sh() {
            Some(Self::P2sh)
        } else if s.is_p2wpkh() {
            Some(Self::P2wpkh)
        } else if s.is_p2wsh() {
            Some(Self::P2wsh)
        } else if s.is_p2tr() {
            Some(Self::P2tr)
        } else {
            None
        }
    }
}

// ---------------------------------------------------------------------------
// Template
// ---------------------------------------------------------------------------

/// A `@N` template, parsed once.
#[derive(Debug, Clone)]
pub struct Template {
    /// The template as written (after the `/**` expansion), `NUMS` kept.
    pub text: String,
    desc: MsDescriptor<DescriptorPublicKey>,
    /// Synthetic key per slot (index = slot).
    slot_keys: Vec<DescriptorPublicKey>,
    /// Number of slots.
    pub n: usize,
    /// Per slot, the use-site step of each alternative (`None` for a bare
    /// `/*`). Every slot has the same number of alternatives (BIP-389).
    pub slot_alts: Vec<Vec<Option<u32>>>,
    /// Per slot, a stated `[fp…]` fingerprint and origin path.
    pub slot_fp: Vec<Option<bitcoin::bip32::Fingerprint>>,
    pub slot_origin: Vec<Option<bitcoin::bip32::DerivationPath>>,
    /// The output kind this template produces.
    pub kind: SpkKind,
    /// `p2wpkh`, `p2wsh`, `p2sh-p2wsh`, ...
    pub script_type: &'static str,
}

/// Why a template is refused at intake (exit 2, `bundle`'s wording where
/// `bundle` refuses the same thing).
pub type TemplateError = String;

impl Template {
    /// Parse a template. Refuses what `bundle` refuses at mint: a bare `@N`,
    /// a hardened wildcard `/*h`, a fixed single step (the lexer's refusal),
    /// slots with different numbers of alternatives, and a bare script.
    pub fn parse(text: &str) -> Result<Self, TemplateError> {
        let expanded = expand_literal_double_star(text).into_owned();
        let nums = substitute_nums_sentinel(&expanded);
        let occs = lex_placeholders(&nums).map_err(|e| e.to_string())?;
        if let Some(occ) = occs.iter().find(|o| !o.wildcard) {
            let i = occ.i;
            return Err(format!(
                "@{i} has no derivation after it: a bare `@{i}` would be recorded as `@{i}/*`, \
                 which names different keys from the ones typed. Write `@{i}/<0;1>/*` (receive \
                 addresses on 0, change on 1, as most wallets do), or `@{i}/*` if this wallet \
                 really uses a single chain."
            ));
        }
        if let Some(occ) = occs.iter().find(|o| o.wildcard_hardened) {
            return Err(format!(
                "@{} has a hardened wildcard (`/*h`): hardened leaf steps cannot be derived from \
                 a public key and are not searched (as `bundle` refuses them)",
                occ.i
            ));
        }
        let resolved = resolve_placeholders(&occs).map_err(|e| e.to_string())?;
        let n = resolved.n as usize;
        let mut slot_alts: Vec<Vec<Option<u32>>> = vec![Vec::new(); n];
        let mut slot_fp = vec![None; n];
        let mut slot_origin = vec![None; n];
        for occ in &occs {
            let i = occ.i as usize;
            slot_fp[i] = occ.fingerprint_anno;
            slot_origin[i] = occ.origin_path_anno.clone();
            if slot_alts[i].is_empty() {
                slot_alts[i] = if occ.multipath_alts.is_empty() {
                    vec![None]
                } else {
                    occ.multipath_alts.iter().map(|v| Some(*v)).collect()
                };
            }
        }
        let a0 = slot_alts[0].len();
        if slot_alts.iter().any(|a| a.len() != a0) {
            return Err(
                "every @N of one descriptor must have the same number of use-site alternatives \
                 (BIP-389)"
                    .into(),
            );
        }
        let ctx = if n == 1 {
            ScriptCtx::SingleSig
        } else {
            ScriptCtx::MultiSig
        };
        let (substituted, key_map) = substitute_synthetic(&nums, ctx).map_err(|e| e.to_string())?;
        let desc = parse_descriptor_lenient(&substituted)
            .map_err(|e| format!("descriptor parse failed: {e}"))?;
        let mut slot_keys: Vec<Option<DescriptorPublicKey>> = vec![None; n];
        for (s, i) in &key_map {
            let k = DescriptorPublicKey::from_str(s)
                .map_err(|e| format!("internal: synthetic key: {e}"))?;
            slot_keys[*i as usize] = Some(k);
        }
        let slot_keys: Vec<DescriptorPublicKey> = slot_keys
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .ok_or_else(|| "internal: a slot has no synthetic key".to_string())?;
        let t = expanded.trim();
        let (kind, script_type) = if t.starts_with("pkh(") {
            (SpkKind::P2pkh, "p2pkh")
        } else if t.starts_with("wpkh(") {
            (SpkKind::P2wpkh, "p2wpkh")
        } else if t.starts_with("sh(wpkh(") {
            (SpkKind::P2sh, "p2sh-p2wpkh")
        } else if t.starts_with("sh(wsh(") {
            (SpkKind::P2sh, "p2sh-p2wsh")
        } else if t.starts_with("sh(") {
            (SpkKind::P2sh, "p2sh")
        } else if t.starts_with("wsh(") {
            (SpkKind::P2wsh, "p2wsh")
        } else if t.starts_with("tr(") {
            (SpkKind::P2tr, "p2tr")
        } else {
            return Err(
                "only pkh, wpkh, sh, wsh and tr descriptors are searched (a bare script has no \
                 address)"
                    .into(),
            );
        };
        Ok(Self {
            text: expanded,
            desc,
            slot_keys,
            n,
            slot_alts,
            slot_fp,
            slot_origin,
            kind,
            script_type,
        })
    }

    /// Number of use-site alternatives per slot.
    pub fn alternatives(&self) -> usize {
        self.slot_alts[0].len()
    }

    /// The scriptPubKey with `keys[slot]` in each slot.
    pub fn eval(&self, keys: &[PublicKey]) -> Option<ScriptBuf> {
        let mut t = KeyTranslator {
            slot_keys: &self.slot_keys,
            keys,
        };
        let d = self.desc.translate_pk(&mut t).ok()?;
        Some(d.script_pubkey())
    }
}

struct KeyTranslator<'a> {
    slot_keys: &'a [DescriptorPublicKey],
    keys: &'a [PublicKey],
}

impl Translator<DescriptorPublicKey> for KeyTranslator<'_> {
    type TargetPk = PublicKey;
    type Error = ();

    fn pk(&mut self, pk: &DescriptorPublicKey) -> Result<PublicKey, ()> {
        if let Some(i) = self.slot_keys.iter().position(|k| k == pk) {
            return Ok(self.keys[i]);
        }
        match pk {
            DescriptorPublicKey::Single(s) => match s.key {
                SinglePubKey::FullKey(k) => Ok(k),
                SinglePubKey::XOnly(x) => Ok(x.to_public_key()),
            },
            _ => Err(()),
        }
    }

    translate_hash_clone!(DescriptorPublicKey);
}

/// The child key at `step/idx` under an account xpub (`idx` alone for a bare
/// `/*`).
pub fn derive_child<C: Verification>(
    secp: &Secp256k1<C>,
    account: &Xpub,
    step: Option<u32>,
    idx: u32,
) -> Option<PublicKey> {
    let chain = match step {
        Some(s) => account
            .ckd_pub(secp, ChildNumber::from_normal_idx(s).ok()?)
            .ok()?,
        None => *account,
    };
    let c = chain
        .ckd_pub(secp, ChildNumber::from_normal_idx(idx).ok()?)
        .ok()?;
    Some(PublicKey::new(c.public_key))
}

// ---------------------------------------------------------------------------
// Child-key cache (pruning 2)
// ---------------------------------------------------------------------------

/// `children[src][((path * nsteps) + step) * gap + idx]`.
pub struct ChildCache {
    pub steps: Vec<Option<u32>>,
    pub gap: u32,
    children: Vec<Vec<PublicKey>>,
}

impl ChildCache {
    /// Derive every (source, path, step, index) child key once, in parallel.
    /// `accounts[src]` is the source's account xpub per path entry.
    pub fn build(accounts: &[Vec<Xpub>], steps: &[Option<u32>], gap: u32, threads: usize) -> Self {
        let secp = Secp256k1::verification_only();
        let nsteps = steps.len();
        let units: Vec<(usize, usize)> = accounts
            .iter()
            .enumerate()
            .flat_map(|(s, v)| (0..v.len()).map(move |p| (s, p)))
            .collect();
        let results: Mutex<Vec<(usize, usize, Vec<PublicKey>)>> = Mutex::new(Vec::new());
        let cursor = AtomicU64::new(0);
        let threads = threads.max(1).min(units.len().max(1));
        std::thread::scope(|scope| {
            for _ in 0..threads {
                scope.spawn(|| loop {
                    let u = cursor.fetch_add(1, Ordering::Relaxed) as usize;
                    if u >= units.len() {
                        break;
                    }
                    let (s, p) = units[u];
                    let acct = &accounts[s][p];
                    let mut v = Vec::with_capacity(nsteps * gap as usize);
                    for st in steps {
                        let chain = match st {
                            Some(x) => acct.ckd_pub(&secp, ChildNumber::Normal { index: *x }).ok(),
                            None => Some(*acct),
                        };
                        for i in 0..gap {
                            let k = chain
                                .as_ref()
                                .and_then(|c| {
                                    c.ckd_pub(&secp, ChildNumber::Normal { index: i }).ok()
                                })
                                .map(|c| PublicKey::new(c.public_key))
                                // An invalid child (probability ~2^-127) gets
                                // the generator-free placeholder of the
                                // account key itself; it can match nothing a
                                // real wallet produces at that index.
                                .unwrap_or_else(|| PublicKey::new(acct.public_key));
                            v.push(k);
                        }
                    }
                    results.lock().expect("cache mutex").push((s, p, v));
                });
            }
        });
        let mut children: Vec<Vec<PublicKey>> = accounts
            .iter()
            .map(|v| Vec::with_capacity(v.len() * nsteps * gap as usize))
            .collect();
        let mut res = results.into_inner().expect("cache mutex");
        res.sort_by_key(|(s, p, _)| (*s, *p));
        for (s, _p, v) in res {
            children[s].extend(v);
        }
        Self {
            steps: steps.to_vec(),
            gap,
            children,
        }
    }

    #[inline]
    pub fn get(&self, src: usize, path: usize, step: usize, idx: u32) -> PublicKey {
        let nsteps = self.steps.len();
        self.children[src][((path * nsteps) + step) * self.gap as usize + idx as usize]
    }

    /// Total cached child keys.
    pub fn len(&self) -> usize {
        self.children.iter().map(|v| v.len()).sum()
    }

    /// True when nothing is cached.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

// ---------------------------------------------------------------------------
// The (path tuple, slot order) space (§4.1 item 2)
// ---------------------------------------------------------------------------

/// How one segment's path tuple `t` maps to each source's path index.
#[derive(Debug, Clone)]
pub enum PathTuples {
    /// Every searched seed at path `t` of the shared list (`T = C`).
    Shared { c: u64 },
    /// `t` read mixed-radix over the searched seeds, in seed order
    /// (`T = Π Cᵢ`); `seeds[i]` is a source id.
    Independent { seeds: Vec<usize>, c: u64 },
    /// Refugium seats: a digit `r ∈ 0..8` per seed, in seed order; the seed
    /// in slot `i` is at seat `32 r + i` (`T = 8^k`).
    Refugium { seeds: Vec<usize> },
}

impl PathTuples {
    pub fn count(&self) -> Option<u64> {
        match self {
            Self::Shared { c } => Some(*c),
            Self::Independent { seeds, c } => {
                let mut acc: u64 = 1;
                for _ in seeds {
                    acc = acc.checked_mul(*c)?;
                }
                Some(acc)
            }
            Self::Refugium { seeds } => {
                let mut acc: u64 = 1;
                for _ in seeds {
                    acc = acc.checked_mul(8)?;
                }
                Some(acc)
            }
        }
    }
}

/// Per source, where its path entries sit.
#[derive(Debug, Clone)]
pub struct SourcePaths {
    /// Length of the standard (shared) list at the start (1 for a cosigner).
    pub std_len: usize,
    /// Start of the 256 Refugium seat entries, when present.
    pub seat_base: Option<usize>,
    /// `pin[slot]`: this source's path index when it sits in a slot whose
    /// path is pinned.
    pub pin: Vec<Option<usize>>,
}

/// The whole enumeration for one template.
#[derive(Debug, Clone)]
pub struct Space {
    pub n: usize,
    /// `pinned_key[slot]`: the source that must sit in that slot.
    pub pinned_key: Vec<Option<usize>>,
    /// Slots without a pinned key, ascending.
    pub free_slots: Vec<usize>,
    /// Sources without a pinned slot, in seed-then-cosigner order.
    pub free_keys: Vec<usize>,
    /// Slot orders per tuple: 1 under the sorted collapse, else `f!`.
    pub orders: u64,
    pub segments: Vec<PathTuples>,
    pub sources: Vec<SourcePaths>,
}

/// Slot → (source, path index).
pub type Assignment = Vec<(usize, usize)>;

impl Space {
    /// Candidate count `Σ Tₛ × S`, or `None` past `u64`.
    pub fn candidates(&self) -> Option<u64> {
        let mut total: u64 = 0;
        for seg in &self.segments {
            total = total.checked_add(seg.count()?.checked_mul(self.orders)?)?;
        }
        Some(total)
    }

    /// Decode a rank into the slot assignment.
    pub fn decode(&self, mut rank: u64) -> Option<Assignment> {
        let mut seg_i = 0;
        loop {
            let seg = self.segments.get(seg_i)?;
            let size = seg.count()?.checked_mul(self.orders)?;
            if rank < size {
                break;
            }
            rank -= size;
            seg_i += 1;
        }
        let seg = &self.segments[seg_i];
        let t = rank / self.orders;
        let sigma_rank = rank % self.orders;
        let f = self.free_slots.len();
        let sigma: Vec<usize> = if self.orders == 1 {
            (0..f).collect()
        } else {
            unrank_permutation(u128::from(sigma_rank), f)
        };
        let mut slot_src: Vec<usize> = vec![usize::MAX; self.n];
        for (slot, p) in self.pinned_key.iter().enumerate() {
            if let Some(s) = p {
                slot_src[slot] = *s;
            }
        }
        for (j, &slot) in self.free_slots.iter().enumerate() {
            slot_src[slot] = self.free_keys[sigma[j]];
        }
        let mut out: Assignment = Vec::with_capacity(self.n);
        for (slot, &src) in slot_src.iter().enumerate() {
            let sp = &self.sources[src];
            let path = if let Some(p) = sp.pin[slot] {
                p
            } else if let (PathTuples::Refugium { seeds }, Some(base)) = (seg, sp.seat_base) {
                let pos = seeds.iter().position(|s| *s == src)?;
                let mut x = t;
                for _ in (pos + 1)..seeds.len() {
                    x /= 8;
                }
                let seat = 32 * (x % 8) as usize + slot;
                if seat > 255 {
                    return None;
                }
                base + seat
            } else if sp.std_len <= 1 {
                0
            } else {
                match seg {
                    PathTuples::Shared { .. } => t as usize,
                    PathTuples::Independent { seeds, c } => {
                        let pos = seeds.iter().position(|s| *s == src)?;
                        // Digit for seeds[pos]: most significant first.
                        let mut x = t;
                        for _ in (pos + 1)..seeds.len() {
                            x /= c;
                        }
                        (x % c) as usize
                    }
                    PathTuples::Refugium { .. } => 0,
                }
            };
            out.push((src, path));
        }
        Some(out)
    }
}

/// `f!` as `u64`, or `None`.
pub fn orders_for(f: usize) -> Option<u64> {
    factorial(f).and_then(|v| u64::try_from(v).ok())
}

/// Slot orders searched per path tuple: 1 under the sorted collapse, `f!`
/// otherwise, or `None` past `u64`.
///
/// The collapse is sound only when a slot contributes nothing but its
/// use-site path (SPEC §4 pruning 4): the template passes
/// [`sorted_collapse_applies`], AND every free key has the same candidate
/// path set in every free slot. Two things make a slot select a path, and
/// either one disables the collapse (review r1 C1, I1):
/// - a Refugium seat segment (the seat is `32 r + slot`, so the path fixes
///   the slot);
/// - a path pin that differs between free slots (a keyless md1's per-slot
///   origins, which bind the path to the slot, not to a seed).
pub fn slot_orders(
    tpl: &Template,
    free_slots: &[usize],
    free_keys: &[usize],
    segments: &[PathTuples],
    sources: &[SourcePaths],
) -> Option<u64> {
    let same_paths_in_every_free_slot = free_keys.iter().all(|k| {
        let pin = &sources[*k].pin;
        free_slots
            .iter()
            .all(|s| free_slots.first().is_none_or(|s0| pin[*s] == pin[*s0]))
    });
    let seat_segment = segments
        .iter()
        .any(|s| matches!(s, PathTuples::Refugium { .. }));
    if sorted_collapse_applies(tpl) && !seat_segment && same_paths_in_every_free_slot {
        Some(1)
    } else {
        orders_for(free_slots.len())
    }
}

/// True when the template evaluates at the space's first candidate (a
/// pre-scan guard: a template whose keys the evaluator cannot translate
/// would otherwise match nothing and print a false no-match).
pub fn probe_eval(space: &Space, tpl: &Template, cache: &ChildCache, alts: usize) -> bool {
    if space.candidates().unwrap_or(0) == 0 {
        return true;
    }
    let targets = HashMap::new();
    let ctx = ScanCtx::new(space, tpl, cache, &targets, alts);
    let Some(assign) = space.decode(0) else {
        return false;
    };
    let keys: Vec<PublicKey> = assign
        .iter()
        .enumerate()
        .map(|(slot, (src, path))| cache.get(*src, *path, ctx.step_idx[slot][0], 0))
        .collect();
    tpl.eval(&keys).is_some()
}

// ---------------------------------------------------------------------------
// Scan (§4.1 item 1)
// ---------------------------------------------------------------------------

/// One matched (candidate, alternative, index, target).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hit {
    pub rank: u64,
    pub alt: u32,
    pub index: u32,
    pub target: usize,
}

/// Every hit counted, the kept list capped and sorted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScanResult {
    pub hits: Vec<Hit>,
    pub total_hits: u64,
    pub capped: bool,
}

struct ScanCtx<'a> {
    space: &'a Space,
    tpl: &'a Template,
    cache: &'a ChildCache,
    targets: &'a HashMap<ScriptBuf, usize>,
    alts: usize,
    /// `step_idx[slot][alt]`: index into `cache.steps`.
    step_idx: Vec<Vec<usize>>,
}

impl<'a> ScanCtx<'a> {
    fn new(
        space: &'a Space,
        tpl: &'a Template,
        cache: &'a ChildCache,
        targets: &'a HashMap<ScriptBuf, usize>,
        alts: usize,
    ) -> Self {
        let step_idx = tpl
            .slot_alts
            .iter()
            .map(|a| {
                a.iter()
                    .take(alts)
                    .map(|st| {
                        cache
                            .steps
                            .iter()
                            .position(|x| x == st)
                            .expect("every template step is cached")
                    })
                    .collect()
            })
            .collect();
        Self {
            space,
            tpl,
            cache,
            targets,
            alts,
            step_idx,
        }
    }

    fn eval_rank(&self, rank: u64, keys: &mut [PublicKey], out: &mut impl FnMut(Hit)) {
        let Some(assign) = self.space.decode(rank) else {
            return;
        };
        for a in 0..self.alts {
            for idx in 0..self.cache.gap {
                for (slot, (src, path)) in assign.iter().enumerate() {
                    keys[slot] = self.cache.get(*src, *path, self.step_idx[slot][a], idx);
                }
                if let Some(spk) = self.tpl.eval(keys) {
                    if let Some(t) = self.targets.get(&spk) {
                        out(Hit {
                            rank,
                            alt: a as u32,
                            index: idx,
                            target: *t,
                        });
                    }
                }
            }
        }
    }
}

fn dummy_key() -> PublicKey {
    // The secp256k1 generator, a valid point; overwritten before any use.
    PublicKey::from_str("0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798")
        .expect("static key")
}

/// The single-threaded reference over the same space (§4.1 last line).
#[cfg(test)]
pub fn scan_reference(
    space: &Space,
    tpl: &Template,
    cache: &ChildCache,
    targets: &HashMap<ScriptBuf, usize>,
    alts: usize,
    cap: usize,
) -> ScanResult {
    let ctx = ScanCtx::new(space, tpl, cache, targets, alts);
    let total = space.candidates().unwrap_or(0);
    let mut keys = vec![dummy_key(); space.n];
    let mut hits = Vec::new();
    let mut count: u64 = 0;
    for r in 0..total {
        ctx.eval_rank(r, &mut keys, &mut |h| {
            count += 1;
            if hits.len() < cap {
                hits.push(h);
            }
        });
    }
    hits.sort();
    ScanResult {
        capped: count as usize > hits.len(),
        hits,
        total_hits: count,
    }
}

/// Per-candidate cost, measured on `samples` ranks strided over the space.
pub fn calibrate(
    space: &Space,
    tpl: &Template,
    cache: &ChildCache,
    targets: &HashMap<ScriptBuf, usize>,
    alts: usize,
    samples: u64,
) -> Duration {
    let ctx = ScanCtx::new(space, tpl, cache, targets, alts);
    let total = space.candidates().unwrap_or(0);
    let samples = samples.min(total).max(1);
    if total == 0 {
        return Duration::ZERO;
    }
    let stride = (total / samples).max(1);
    let mut keys = vec![dummy_key(); space.n];
    let start = Instant::now();
    for i in 0..samples {
        ctx.eval_rank((i * stride) % total, &mut keys, &mut |h| {
            std::hint::black_box(h);
        });
    }
    start.elapsed() / u32::try_from(samples).unwrap_or(u32::MAX)
}

/// The parallel collect-all scan. `progress` is called on the calling thread
/// about every `progress_every` with (done, total).
#[allow(clippy::too_many_arguments)]
pub fn scan(
    space: &Space,
    tpl: &Template,
    cache: &ChildCache,
    targets: &HashMap<ScriptBuf, usize>,
    alts: usize,
    threads: usize,
    cap: usize,
    progress_every: Option<Duration>,
    progress: &mut dyn FnMut(u64, u64),
) -> ScanResult {
    let ctx = ScanCtx::new(space, tpl, cache, targets, alts);
    let total = space.candidates().unwrap_or(0);
    let threads = threads.max(1);
    // Each candidate costs alternatives × gap evaluations; claim small blocks
    // so the tail stays balanced.
    let block: u64 = (total / (threads as u64 * 64)).clamp(1, 4096);
    let cursor = AtomicU64::new(0);
    let done = AtomicU64::new(0);
    let count = AtomicU64::new(0);
    let kept: Mutex<Vec<Hit>> = Mutex::new(Vec::new());
    std::thread::scope(|scope| {
        let mut handles = Vec::with_capacity(threads);
        for _ in 0..threads {
            let ctx = &ctx;
            let cursor = &cursor;
            let done = &done;
            let count = &count;
            let kept = &kept;
            handles.push(scope.spawn(move || {
                let mut keys = vec![dummy_key(); ctx.space.n];
                let mut local: Vec<Hit> = Vec::new();
                loop {
                    let start = cursor.fetch_add(block, Ordering::Relaxed);
                    if start >= total {
                        break;
                    }
                    let end = (start + block).min(total);
                    for r in start..end {
                        ctx.eval_rank(r, &mut keys, &mut |h| {
                            count.fetch_add(1, Ordering::Relaxed);
                            local.push(h);
                        });
                    }
                    if local.len() > cap {
                        let mut k = kept.lock().expect("kept mutex");
                        let room = cap.saturating_sub(k.len());
                        k.extend(local.drain(..).take(room));
                    }
                    done.fetch_add(end - start, Ordering::Relaxed);
                }
                let mut k = kept.lock().expect("kept mutex");
                let room = cap.saturating_sub(k.len());
                k.extend(local.drain(..).take(room));
            }));
        }
        if let Some(every) = progress_every {
            let mut last = Instant::now();
            while handles.iter().any(|h| !h.is_finished()) {
                std::thread::sleep(Duration::from_millis(200));
                if last.elapsed() >= every {
                    last = Instant::now();
                    progress(done.load(Ordering::Relaxed), total);
                }
            }
        }
    });
    let mut hits = kept.into_inner().expect("kept mutex");
    hits.sort();
    let total_hits = count.load(Ordering::Relaxed);
    ScanResult {
        capped: total_hits as usize > hits.len(),
        hits,
        total_hits,
    }
}

// ---------------------------------------------------------------------------
// Expression tree over the template text (pruning 4, twin, merge)
// ---------------------------------------------------------------------------

/// A descriptor expression: `name(args)`, a tap branch `{a,b}`, or a leaf
/// token (a key expression or a number).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Node { name: String, args: Vec<Expr> },
    Branch(Vec<Expr>),
    Leaf(String),
}

impl Expr {
    pub fn parse(s: &str) -> Result<Expr, String> {
        let b = s.trim().as_bytes();
        let mut pos = 0;
        let e = parse_at(b, &mut pos)?;
        if pos != b.len() {
            return Err(format!("unexpected text at byte {pos}"));
        }
        Ok(e)
    }

    pub fn render(&self) -> String {
        let mut out = String::new();
        self.render_into(&mut out);
        out
    }

    fn render_into(&self, out: &mut String) {
        match self {
            Expr::Leaf(s) => out.push_str(s),
            Expr::Node { name, args } => {
                out.push_str(name);
                out.push('(');
                for (i, a) in args.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    a.render_into(out);
                }
                out.push(')');
            }
            Expr::Branch(c) => {
                out.push('{');
                for (i, a) in c.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    a.render_into(out);
                }
                out.push('}');
            }
        }
    }

    /// Apply `f` to every leaf.
    pub fn map_leaves(&self, f: &dyn Fn(&str) -> String) -> Expr {
        match self {
            Expr::Leaf(s) => Expr::Leaf(f(s)),
            Expr::Node { name, args } => Expr::Node {
                name: name.clone(),
                args: args.iter().map(|a| a.map_leaves(f)).collect(),
            },
            Expr::Branch(c) => Expr::Branch(c.iter().map(|a| a.map_leaves(f)).collect()),
        }
    }

    /// Paths (child-index lists) of every multi-family fragment.
    pub fn multi_fragments(&self) -> Vec<Vec<usize>> {
        let mut out = Vec::new();
        self.collect_frags(&mut Vec::new(), &mut out);
        out
    }

    fn collect_frags(&self, here: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
        match self {
            Expr::Leaf(_) => {}
            Expr::Node { name, args } => {
                if multi_family(name).is_some() {
                    out.push(here.clone());
                }
                for (i, a) in args.iter().enumerate() {
                    here.push(i);
                    a.collect_frags(here, out);
                    here.pop();
                }
            }
            Expr::Branch(c) => {
                for (i, a) in c.iter().enumerate() {
                    here.push(i);
                    a.collect_frags(here, out);
                    here.pop();
                }
            }
        }
    }

    pub fn at(&self, path: &[usize]) -> &Expr {
        let mut e = self;
        for &i in path {
            e = match e {
                Expr::Node { args, .. } => &args[i],
                Expr::Branch(c) => &c[i],
                Expr::Leaf(_) => return e,
            };
        }
        e
    }

    pub fn at_mut(&mut self, path: &[usize]) -> &mut Expr {
        let mut e = self;
        for &i in path {
            e = match e {
                Expr::Node { args, .. } => &mut args[i],
                Expr::Branch(c) => &mut c[i],
                Expr::Leaf(_) => return e,
            };
        }
        e
    }

    /// The canonical form under the known symmetries (§5): the arguments of
    /// every sorted fragment sorted, the two children of every tap branch
    /// sorted. Applied to an instantiated template (leaves labelled by key
    /// and use-site path) this is the merge key.
    pub fn canonical(&self) -> Expr {
        match self {
            Expr::Leaf(s) => Expr::Leaf(s.clone()),
            Expr::Node { name, args } => {
                let mut args: Vec<Expr> = args.iter().map(|a| a.canonical()).collect();
                if matches!(multi_family(name), Some((true, _))) && args.len() > 1 {
                    let mut rest: Vec<Expr> = args.split_off(1);
                    rest.sort_by_key(|a| a.render());
                    args.extend(rest);
                }
                Expr::Node {
                    name: name.clone(),
                    args,
                }
            }
            Expr::Branch(c) => {
                let mut c: Vec<Expr> = c.iter().map(|a| a.canonical()).collect();
                c.sort_by_key(|a| a.render());
                Expr::Branch(c)
            }
        }
    }
}

/// `Some((sorted, tap))` for `multi`, `sortedmulti`, `multi_a`,
/// `sortedmulti_a` (with or without a wrapper prefix like `v:`).
pub fn multi_family(name: &str) -> Option<(bool, bool)> {
    let base = name.rsplit(':').next().unwrap_or(name);
    match base {
        "multi" => Some((false, false)),
        "sortedmulti" => Some((true, false)),
        "multi_a" => Some((false, true)),
        "sortedmulti_a" => Some((true, true)),
        _ => None,
    }
}

fn parse_at(b: &[u8], pos: &mut usize) -> Result<Expr, String> {
    if *pos < b.len() && b[*pos] == b'{' {
        *pos += 1;
        let mut c = Vec::new();
        loop {
            c.push(parse_at(b, pos)?);
            match b.get(*pos) {
                Some(b',') => *pos += 1,
                Some(b'}') => {
                    *pos += 1;
                    return Ok(Expr::Branch(c));
                }
                _ => return Err("unterminated `{`".into()),
            }
        }
    }
    let start = *pos;
    while *pos < b.len() && !matches!(b[*pos], b'(' | b')' | b',' | b'{' | b'}') {
        *pos += 1;
    }
    let tok = std::str::from_utf8(&b[start..*pos])
        .map_err(|_| "non-UTF-8 descriptor".to_string())?
        .trim()
        .to_string();
    if *pos < b.len() && b[*pos] == b'(' {
        *pos += 1;
        let mut args = Vec::new();
        if b.get(*pos) == Some(&b')') {
            *pos += 1;
            return Ok(Expr::Node { name: tok, args });
        }
        loop {
            args.push(parse_at(b, pos)?);
            match b.get(*pos) {
                Some(b',') => *pos += 1,
                Some(b')') => {
                    *pos += 1;
                    return Ok(Expr::Node { name: tok, args });
                }
                _ => return Err("unterminated `(`".into()),
            }
        }
    }
    if tok.is_empty() {
        return Err(format!("empty expression at byte {start}"));
    }
    Ok(Expr::Leaf(tok))
}

/// The slot number of a key-expression leaf (`[fp/path]@N/<..>/*`).
pub fn leaf_slot(leaf: &str) -> Option<usize> {
    let at = leaf.find('@')?;
    let digits: String = leaf[at + 1..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse().ok()
}

/// The use-site suffix of a key-expression leaf (everything after `@N` and
/// any suffix origin bracket), e.g. `/<0;1>/*`.
pub fn leaf_use_site(leaf: &str) -> String {
    let Some(at) = leaf.find('@') else {
        return String::new();
    };
    let rest = &leaf[at + 1..];
    let rest = rest.trim_start_matches(|c: char| c.is_ascii_digit());
    let rest = if rest.starts_with('[') {
        rest.find(']').map_or(rest, |e| &rest[e + 1..])
    } else {
        rest
    };
    rest.to_string()
}

/// Whether the sorted collapse (pruning 4) applies: `wsh(F)`, `sh(F)`,
/// `sh(wsh(F))` or `tr(NUMS,F)` with F one `sortedmulti`/`sortedmulti_a`
/// that is the whole script, every slot appearing in F exactly once and
/// nowhere else, and every slot with the same use-site alternatives.
pub fn sorted_collapse_applies(tpl: &Template) -> bool {
    let Ok(e) = Expr::parse(&tpl.text) else {
        return false;
    };
    let frag = match &e {
        Expr::Node { name, args } if (name == "wsh" || name == "sh") && args.len() == 1 => {
            match &args[0] {
                Expr::Node { name: n2, args: a2 } if n2 == "wsh" && a2.len() == 1 => &a2[0],
                other => other,
            }
        }
        Expr::Node { name, args } if name == "tr" && args.len() == 2 => match &args[0] {
            Expr::Leaf(k) if k == "NUMS" || k == NUMS_H_POINT_X_ONLY_HEX => &args[1],
            _ => return false,
        },
        _ => return false,
    };
    let Expr::Node { name, args } = frag else {
        return false;
    };
    if name != "sortedmulti" && name != "sortedmulti_a" {
        return false;
    }
    let mut seen = vec![0usize; tpl.n];
    for a in args.iter().skip(1) {
        match a {
            Expr::Leaf(l) => match leaf_slot(l) {
                Some(i) if i < tpl.n => seen[i] += 1,
                _ => return false,
            },
            _ => return false,
        }
    }
    // Every slot exactly once in F; F is the whole script, so nowhere else.
    if seen.iter().any(|c| *c != 1) {
        return false;
    }
    tpl.slot_alts.iter().all(|a| *a == tpl.slot_alts[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn xpubs(phrase: &str, paths: &[&str]) -> Vec<Xpub> {
        use bitcoin::bip32::{DerivationPath, Xpriv};
        let seed = bip39::Mnemonic::parse_in(bip39::Language::English, phrase)
            .unwrap()
            .to_seed("");
        let secp = Secp256k1::new();
        let m = Xpriv::new_master(bitcoin::NetworkKind::Main, &seed).unwrap();
        paths
            .iter()
            .map(|p| {
                Xpub::from_priv(
                    &secp,
                    &m.derive_priv(&secp, &DerivationPath::from_str(p).unwrap())
                        .unwrap(),
                )
            })
            .collect()
    }

    /// The parallel collect-all scan equals the single-threaded reference
    /// over the same space, including the collected match list (§4.1).
    #[test]
    fn parallel_scan_equals_reference() {
        let paths = ["m/48'/0'/0'/2'", "m/48'/0'/1'/2'", "m/87'/0'/0'"];
        let a = xpubs(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
            &paths,
        );
        let z = xpubs("zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong", &paths);
        for text in [
            "wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))",
            "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))",
            "tr(NUMS,multi_a(1,@0/<0;1>/*,@1/<0;1>/*))",
        ] {
            let tpl = Template::parse(text).unwrap();
            let steps = vec![Some(0), Some(1)];
            let cache = ChildCache::build(&[a.clone(), z.clone()], &steps, 6, 4);
            let mk = |seg: PathTuples| Space {
                n: 2,
                pinned_key: vec![None, None],
                free_slots: vec![0, 1],
                free_keys: vec![0, 1],
                orders: 2,
                segments: vec![seg],
                sources: vec![
                    SourcePaths {
                        std_len: 3,
                        seat_base: None,
                        pin: vec![None, None]
                    };
                    2
                ],
            };
            for space in [
                mk(PathTuples::Shared { c: 3 }),
                mk(PathTuples::Independent {
                    seeds: vec![0, 1],
                    c: 3,
                }),
            ] {
                // Targets: a few scripts from inside the space, one outside.
                let mut targets = HashMap::new();
                for (i, (r, alt, idx)) in [(0u64, 0usize, 0u32), (5, 1, 3), (7, 0, 5)]
                    .iter()
                    .enumerate()
                {
                    if *r >= space.candidates().unwrap() {
                        continue;
                    }
                    let asg = space.decode(*r).unwrap();
                    let keys: Vec<PublicKey> = asg
                        .iter()
                        .enumerate()
                        .map(|(slot, (s, p))| {
                            cache.get(*s, *p, tpl.slot_alts[slot][*alt].unwrap() as usize, *idx)
                        })
                        .collect();
                    targets.insert(tpl.eval(&keys).unwrap(), i);
                }
                let reference = scan_reference(&space, &tpl, &cache, &targets, 2, 1000);
                let parallel = scan(
                    &space,
                    &tpl,
                    &cache,
                    &targets,
                    2,
                    4,
                    1000,
                    None,
                    &mut |_, _| {},
                );
                assert!(
                    reference.total_hits as usize >= targets.len() && targets.len() >= 2,
                    "{text}"
                );
                assert_eq!(parallel, reference, "{text}");
                // The cap is counted, not silently dropped.
                let capped = scan(
                    &space,
                    &tpl,
                    &cache,
                    &targets,
                    2,
                    4,
                    1,
                    None,
                    &mut |_, _| {},
                );
                assert!(capped.capped && capped.total_hits == reference.total_hits);
            }
        }
    }

    #[test]
    fn expr_round_trips() {
        for s in [
            "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))",
            "tr(NUMS,{sortedmulti_a(1,@0/<0;1>/*,@1/<0;1>/*),sortedmulti_a(1,@2/<0;1>/*,@3/<0;1>/*)})",
            "wsh(and_v(v:pk(@0/<0;1>/*),older(144)))",
        ] {
            assert_eq!(Expr::parse(s).unwrap().render(), s);
        }
    }

    #[test]
    fn collapse_only_for_one_whole_sorted_fragment() {
        let yes = [
            "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))",
            "sh(wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*)))",
            "tr(NUMS,sortedmulti_a(2,@0/<0;1>/*,@1/<0;1>/*))",
        ];
        let no = [
            "wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))",
            "wsh(sortedmulti(2,@0/<0;1>/*,@1/<2;3>/*))",
            "tr(NUMS,{sortedmulti_a(1,@0/<0;1>/*,@1/<0;1>/*),sortedmulti_a(1,@2/<0;1>/*,@3/<0;1>/*)})",
            "tr(@0/<0;1>/*,sortedmulti_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))",
        ];
        for s in yes {
            assert!(sorted_collapse_applies(&Template::parse(s).unwrap()), "{s}");
        }
        for s in no {
            assert!(
                !sorted_collapse_applies(&Template::parse(s).unwrap()),
                "{s}"
            );
        }
    }

    #[test]
    fn canonical_sorts_sorted_fragments_and_tap_branches_only() {
        let a = Expr::parse("tr(N,{sortedmulti_a(1,kB,kA),multi_a(1,kD,kC)})").unwrap();
        let b = Expr::parse("tr(N,{multi_a(1,kD,kC),sortedmulti_a(1,kA,kB)})").unwrap();
        assert_eq!(a.canonical(), b.canonical());
        let c = Expr::parse("tr(N,{multi_a(1,kC,kD),sortedmulti_a(1,kA,kB)})").unwrap();
        assert_ne!(a.canonical(), c.canonical());
    }

    #[test]
    fn leaf_helpers() {
        assert_eq!(leaf_slot("[73c5da0a/84h]@12/<0;1>/*"), Some(12));
        assert_eq!(leaf_use_site("[73c5da0a/84h]@1/<2;3>/*"), "/<2;3>/*");
        assert_eq!(leaf_use_site("@0[73c5da0a/84h]/*"), "/*");
    }

    #[test]
    fn template_refusals() {
        assert!(Template::parse("wsh(sortedmulti(2,@0,@1))")
            .unwrap_err()
            .contains("would be recorded as"));
        assert!(Template::parse("wpkh(@0/*h)")
            .unwrap_err()
            .contains("hardened"));
        assert!(Template::parse("wpkh(@0/0/*)").is_err());
        let t = Template::parse("wsh(sortedmulti(2,@0/**,@1/**))").unwrap();
        assert_eq!(t.slot_alts, vec![vec![Some(0), Some(1)]; 2]);
        let t = Template::parse("wpkh(@0/*)").unwrap();
        assert_eq!(t.slot_alts, vec![vec![None]]);
    }
}
