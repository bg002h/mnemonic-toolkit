//! Permutation-search engine — the funds-safety core of the multisig-template
//! completion (#28 phase 2).
//!
//! SPEC: `design/SPEC_bundle_md1_template_multisig_2026-06-20.md` §6 (the
//! engine) + §7 floors 2/5. Plan: `design/IMPLEMENTATION_PLAN_bundle_md1_…`
//! §2 (P1). This module is **standalone** — the actual id/address computation
//! is injected as a [`CandidateEvaluator`] predicate, so P1 carries no
//! md-codec completion code. The real `WalletPolicyId`-prefix + scriptPubKey
//! evaluators are wired in P3 (`cmd/restore.rs`) and P4
//! (`cmd/verify_bundle.rs`).
//!
//! # What it does
//! Given `n` distinct `@N` slots and a set of supplied candidate keys, the
//! engine searches the `n!` bijections (id-search) — or `range × n!`
//! (address-search, ascending-index OUTER) — for assignments the injected
//! predicate accepts, and returns [`SearchOutcome::Unique`] /
//! [`SearchOutcome::None`] / [`SearchOutcome::Ambiguous`].
//!
//! # Funds-safety contract (SPEC §6.2 / §7)
//! - **No silent wrong assembly.** A `Unique` outcome is returned ONLY after
//!   the engine has proven there is no SECOND match. The engine therefore
//!   does NOT early-terminate on the first match for the `Unique` decision; it
//!   scans until either the whole space is exhausted (→ exactly-one →
//!   `Unique`) or a **second** match is found (→ short-circuit → `Ambiguous`).
//!   See [`search`] and the `unique_vs_ambiguous_full_scan` reasoning in the
//!   tests.
//! - **0 matches → `None`; ≥2 matches → `Ambiguous`; exactly 1 → `Unique`.**
//! - **Distinct keys** ([`reject_duplicate_keys`]) — floor 2 — and
//!   **realized-S strong-prefix sizing** ([`required_prefix_bytes`]) — floor 5
//!   / I_new — are caller-facing primitives the id-search caller invokes before
//!   the search.
//!
//! # Parallelism
//! `std::thread` (no rayon in deps), sharded across
//! `min(20, available_parallelism())` threads (the benchmark cap reused from
//! the `idsearch`/`addrsearch` cost-model prior art). The candidate space is a
//! contiguous index range; each thread unranks its slice into a permutation
//! (and address index, in address mode), evaluates, and reports matches. The
//! outcome is a pure function of the collected match COUNT, so the result is
//! **identical to a single-threaded reference** regardless of thread
//! interleaving (determinism asserted in the tests).

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Hard ceiling on the search threads — the benchmark cap reused from the
/// `idsearch_bench` / `addrsearch_bench` cost-model prior art. The realized
/// thread count is `min(MAX_SEARCH_THREADS, available_parallelism())`.
pub const MAX_SEARCH_THREADS: usize = 20;

/// SPEC §6.4 — searches whose estimated EXHAUSTIVE time is below this run
/// silently (no progress UI).
pub const SILENT_THRESHOLD: Duration = Duration::from_secs(30);

/// SPEC §6.4 — the 1-hour ceiling. A search whose estimate exceeds this is
/// REFUSED unless the operator passes an explicit `accept_search_time` ≥ the
/// estimate (forced acknowledgment).
pub const SEARCH_CEILING: Duration = Duration::from_secs(3600);

// ---------------------------------------------------------------------------
// Error surface (library-local, per the `final_word` / `seed_xor` pattern —
// the lib surface stays self-contained; the CLI handler converts to
// `ToolkitError` at the binary boundary in P3/P4).
// ---------------------------------------------------------------------------

/// Errors surfaced by the permutation-search engine + its funds-safety
/// primitives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchError {
    /// Two or more supplied keys are byte-identical (floor 2). Carries the
    /// 0-based indices of the first colliding pair.
    DuplicateKeys { a: usize, b: usize },
    /// `--expect-wallet-id` prefix is shorter than the realized-S strong
    /// prefix (floor 5 / I_new). `required` / `supplied` are byte counts.
    PrefixTooShort { required: usize, supplied: usize },
    /// The estimated exhaustive search time exceeds [`SEARCH_CEILING`] and no
    /// sufficient `accept_search_time` was supplied (SPEC §6.4).
    SearchTimeExceedsCeiling {
        estimate: Duration,
        ceiling: Duration,
    },
    /// `accept_search_time` was supplied but is below the estimated exhaustive
    /// time — the forced-acknowledgment must restate (≥) the estimate.
    AcceptSearchTimeTooLow {
        estimate: Duration,
        supplied: Duration,
    },
    /// `n == 0` — there are no slots to permute.
    EmptySearchSpace,
    /// `n!` (the permutation count) overflows `u128` (`n > 34`) — the candidate
    /// space cannot be addressed. A realized multisig N never approaches this,
    /// but a hostile/garbage slot count must be REFUSED rather than panic (M1).
    SearchSpaceTooLarge { n: usize },
}

impl std::fmt::Display for SearchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchError::DuplicateKeys { a, b } => write!(
                f,
                "duplicate cosigner keys: supplied keys at positions {a} and {b} are identical \
                 (two slots given the same key collide on both address and id)"
            ),
            SearchError::PrefixTooShort { required, supplied } => write!(
                f,
                "--expect-wallet-id prefix too weak for this search: need ≥{required} bytes \
                 (sized to the realized search space), got {supplied}"
            ),
            SearchError::SearchTimeExceedsCeiling { estimate, ceiling } => write!(
                f,
                "estimated exhaustive search time {estimate:?} exceeds the {ceiling:?} ceiling; \
                 re-run with --accept-search-time ≥{estimate:?} to acknowledge"
            ),
            SearchError::AcceptSearchTimeTooLow { estimate, supplied } => write!(
                f,
                "--accept-search-time {supplied:?} is below the estimated exhaustive time \
                 {estimate:?}; restate a duration ≥ the estimate"
            ),
            SearchError::EmptySearchSpace => {
                write!(f, "empty search space: no slots to permute (n == 0)")
            }
            SearchError::SearchSpaceTooLarge { n } => write!(
                f,
                "search space too large: {n} slots → {n}! overflows the candidate index space \
                 (the realized multisig N is a handful of cosigners — this looks malformed)"
            ),
        }
    }
}

impl std::error::Error for SearchError {}

// ---------------------------------------------------------------------------
// The injected predicate.
// ---------------------------------------------------------------------------

/// The injected match predicate — P1's seam. The engine knows nothing about
/// keys, origins, descriptors, ids, or addresses: it hands an `assignment`
/// (slot index → candidate index, a permutation of `0..n`) to `matches` and
/// acts on the boolean.
///
/// In P3/P4 the concrete evaluators build a fresh descriptor under the
/// candidate assignment and:
/// - **id-search:** `compute_wallet_policy_id(candidate)` has the
///   `--expect-wallet-id` strong-prefix (SPEC §6.2).
/// - **address-search:** the candidate's scriptPubKey at the supplied
///   `(chain, idx)` equals the target's (SPEC §6.2). The `(chain, idx)` is the
///   address-mode outer coordinate (see [`search`]).
///
/// `Sync` is required because the engine shares `&self` across the search
/// threads. P1 tests use SYNTHETIC evaluators (e.g. "matches iff
/// `assignment == target`").
pub trait CandidateEvaluator: Sync {
    /// Returns `true` iff `assignment` (a permutation of `0..n`, slot →
    /// candidate) reproduces the target wallet under this evaluator.
    ///
    /// `address_index` is the address-search OUTER coordinate the engine is
    /// currently iterating (`(chain, idx)` flattened — see
    /// [`AddressRange::flatten`]); for id-search it is always `0` and
    /// evaluators ignore it.
    fn matches(&self, assignment: &[usize], address_index: u64) -> bool;
}

/// Blanket impl so a plain closure `Fn(&[usize], u64) -> bool` is a
/// [`CandidateEvaluator`] without a named type — convenient for the synthetic
/// P1 tests and for thin CLI call sites.
impl<F> CandidateEvaluator for F
where
    F: Fn(&[usize], u64) -> bool + Sync,
{
    fn matches(&self, assignment: &[usize], address_index: u64) -> bool {
        self(assignment, address_index)
    }
}

// ---------------------------------------------------------------------------
// Search mode + outcome.
// ---------------------------------------------------------------------------

/// The address-search index range (SPEC §6.3): `[min, max)` over address
/// indices, iterated for one or both chains, ascending-index OUTER.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddressRange {
    /// Inclusive lower address index.
    pub min: u32,
    /// Exclusive upper address index (default range is `0..20`).
    pub max: u32,
    /// Which chain(s) to scan.
    pub chains: ChainScope,
}

/// Which BIP-32 change-chain branch(es) the address-search covers (SPEC §6.3,
/// `--search-chain`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainScope {
    /// External / receive chain only (chain 0) — the default.
    Receive,
    /// Internal / change chain only (chain 1). Constructed by the
    /// `--search-chain change` CLI value wired in P3 (`cmd/restore.rs`).
    Change,
    /// Both chains (doubles the per-index cost).
    Both,
}

impl ChainScope {
    /// Number of chains this scope covers (1 or 2).
    pub fn count(self) -> u32 {
        match self {
            ChainScope::Receive | ChainScope::Change => 1,
            ChainScope::Both => 2,
        }
    }
}

impl AddressRange {
    /// The number of distinct outer `(chain, idx)` coordinates: `span ×
    /// chains`. `span` is `max - min` (saturating; an inverted range is
    /// empty).
    pub fn outer_count(&self) -> u64 {
        let span = u64::from(self.max.saturating_sub(self.min));
        span * u64::from(self.chains.count())
    }

    /// Flatten the `k`-th outer coordinate (`0 <= k < outer_count()`) into the
    /// `address_index` handed to the evaluator. The flattening is
    /// **ascending-index OUTER** (SPEC §6.3): index advances slowest, chain
    /// fastest — so the first coordinates scanned are the lowest indices, and
    /// a low-index target is found before high indices are reached. The
    /// concrete encoding (chain bit + idx) is an evaluator/engine contract;
    /// P1 keeps it opaque and only guarantees ascending-index ordering.
    pub fn flatten(&self, k: u64) -> u64 {
        let chains = u64::from(self.chains.count());
        let idx_offset = k / chains;
        let chain_step = k % chains;
        let idx = u64::from(self.min) + idx_offset;
        // chain occupies the low bit; idx is shifted up by 1. Opaque to the
        // engine; the concrete evaluator decodes it in P3/P4.
        (idx << 1) | chain_step
    }
}

/// The search-space shape (SPEC §6.1): id-search is just the `n!`
/// permutations; address-search multiplies by the `(chain, idx)` range,
/// iterated index-OUTER.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    /// id-search: enumerate the `n!` permutations only. `address_index` handed
    /// to the evaluator is always `0`.
    Id,
    /// address-search: ascending-index OUTER, permutations INNER.
    Address(AddressRange),
}

/// The engine's verdict (SPEC §6.2 refusal floors).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchOutcome {
    /// Exactly one assignment matched — safe to complete. Carries the matching
    /// permutation (slot → candidate index). For address-search, also carries
    /// the `address_index` it matched at.
    Unique {
        /// The matching permutation (slot → candidate index).
        assignment: Vec<usize>,
        /// The address-mode outer coordinate it matched at (`0` for
        /// id-search).
        address_index: u64,
    },
    /// No assignment matched — refuse.
    None,
    /// Two or more assignments matched — refuse (the wallet is not uniquely
    /// determined by this predicate over this space).
    Ambiguous,
}

// ---------------------------------------------------------------------------
// Funds-safety primitives (SPEC §7 floors 2 + 5).
// ---------------------------------------------------------------------------

/// FLOOR 2 — reject duplicate supplied keys (pairwise byte-compare BEFORE the
/// search). Two slots given the same key collide on BOTH address AND id (a
/// "2-of-3" that is secretly a 2-of-2), so the search would either find a
/// spurious match or hide a real one. `keys` are the raw supplied key blobs
/// (e.g. 65-byte `Key65` in P3); P1 is generic over any `&[T: PartialEq]`.
///
/// Returns the first colliding pair's indices in `SearchError::DuplicateKeys`.
/// Note: this is the SUPPLIED-key list (one entry per distinct candidate); it
/// does NOT over-reject legitimate same-`@N` multi-leaf reuse, which is one
/// slot / one key (SPEC §7 floor 2).
pub fn reject_duplicate_keys<T: PartialEq>(keys: &[T]) -> Result<(), SearchError> {
    for a in 0..keys.len() {
        for b in (a + 1)..keys.len() {
            if keys[a] == keys[b] {
                return Err(SearchError::DuplicateKeys { a, b });
            }
        }
    }
    Ok(())
}

/// FLOOR 5 / I_new — the required `--expect-wallet-id` prefix length in BYTES
/// for a search over `search_space` candidates:
///
/// ```text
/// required_prefix_bytes(S) = ceil((log2(S) + 32) / 8)
/// ```
///
/// `S` is the **realized** candidate count: `n!` for an explicit `--account`
/// LIST, or the larger `P((n−own)+K, n)` subset×permutation count for
/// `--own-account-max K`. The caller computes `S` from its enumeration and
/// passes it here. The `+32` term keeps the false-positive probability of a
/// lone spurious match ≤ ~2e-10 across the realized range; a fixed 8-byte
/// prefix would hit ~1-in-275 at `K=32` — so the prefix MUST size from `S`
/// (SPEC §6.2).
///
/// Sized-byte ladder (n=11, own=4) — pinned in the tests:
/// `S = 11!`→8 · `K=8`→9 · `K=16`→10 · `K=32`→11 · `K=64`→13.
///
/// `S = 0` and `S = 1` are clamped to a 0-bit information content (the
/// `log2` floor), giving `ceil(32/8) = 4` bytes — a single-candidate recompute
/// (still the full 4-byte minimum). The result never exceeds the full 16-byte
/// id; callers may cap at 16.
pub fn required_prefix_bytes(search_space: u128) -> usize {
    // log2(S) as an exact integer-ish bit count: use the bit length of the
    // largest value < S that information theory needs to distinguish, i.e.
    // ceil(log2(S)). For S <= 1 there is no choice to distinguish → 0 bits.
    let log2_s_bits: u32 = if search_space <= 1 {
        0
    } else {
        // ceil(log2(S)) = bits needed to represent S-1's full range of
        // outcomes = 128 - leading_zeros(S - 1).
        128 - (search_space - 1).leading_zeros()
    };
    let total_bits = u128::from(log2_s_bits) + 32;
    // ceil(total_bits / 8)
    let bytes = total_bits.div_ceil(8);
    bytes as usize
}

/// Validate that a supplied `--expect-wallet-id` prefix is long enough for the
/// realized search space (floor 5). Convenience wrapper over
/// [`required_prefix_bytes`] the id-search caller invokes before the search.
pub fn validate_prefix_strength(
    supplied_bytes: usize,
    search_space: u128,
) -> Result<(), SearchError> {
    let required = required_prefix_bytes(search_space);
    if supplied_bytes < required {
        return Err(SearchError::PrefixTooShort {
            required,
            supplied: supplied_bytes,
        });
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Adaptive cap (SPEC §6.4).
// ---------------------------------------------------------------------------

/// The cap decision (SPEC §6.4) after calibrating per-candidate cost against
/// the realized search space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapDecision {
    /// Estimated exhaustive time `< 30s` → run silently (no progress UI).
    RunSilent { estimate: Duration },
    /// `30s ≤ estimate ≤ 1h` → run with a progress bar + ETA (the rendering
    /// is CLI-wired later; P1 exposes the decision + the estimate).
    RunWithProgress { estimate: Duration },
}

/// SPEC §6.4 — the adaptive-cap DECISION.
///
/// Given the realized total candidate count and a measured/estimated
/// per-candidate cost, compute the estimated EXHAUSTIVE time `total × cost`
/// and apply the policy:
/// - `< 30s` → [`CapDecision::RunSilent`].
/// - `30s ≤ est ≤ 1h` → [`CapDecision::RunWithProgress`] (progress bar + ETA;
///   rendered by the CLI layer later).
/// - `> 1h` → REFUSE unless `accept_search_time` is `Some(d)` with `d ≥ est`
///   (the forced acknowledgment), in which case [`CapDecision::RunWithProgress`].
///
/// `accept_search_time` below the estimate → [`SearchError::AcceptSearchTimeTooLow`];
/// no override above the ceiling → [`SearchError::SearchTimeExceedsCeiling`].
///
/// The exhaustive time is over the FULL space (no early-terminate credit) —
/// the operator is being asked to accept the worst case (the no-match scan,
/// which is also exactly the scan an `Ambiguous`/`None` outcome performs).
pub fn cap_decision(
    total_candidates: u64,
    per_candidate: Duration,
    accept_search_time: Option<Duration>,
) -> Result<CapDecision, SearchError> {
    let estimate = per_candidate
        .checked_mul_u64(total_candidates)
        .unwrap_or(Duration::MAX);

    if estimate < SILENT_THRESHOLD {
        return Ok(CapDecision::RunSilent { estimate });
    }
    if estimate <= SEARCH_CEILING {
        return Ok(CapDecision::RunWithProgress { estimate });
    }
    // Above the ceiling — require the forced acknowledgment.
    match accept_search_time {
        Some(accepted) if accepted >= estimate => Ok(CapDecision::RunWithProgress { estimate }),
        Some(accepted) => Err(SearchError::AcceptSearchTimeTooLow {
            estimate,
            supplied: accepted,
        }),
        None => Err(SearchError::SearchTimeExceedsCeiling {
            estimate,
            ceiling: SEARCH_CEILING,
        }),
    }
}

/// `Duration::checked_mul` only takes a `u32`; the candidate counts here are
/// `u64`. Small helper for the saturating `u64` multiply used by
/// [`cap_decision`].
trait DurationMulU64 {
    fn checked_mul_u64(self, rhs: u64) -> Option<Duration>;
}

impl DurationMulU64 for Duration {
    fn checked_mul_u64(self, rhs: u64) -> Option<Duration> {
        let nanos = self.as_nanos().checked_mul(u128::from(rhs))?;
        // Duration's full range is ~u64::MAX seconds; cap conservatively at
        // Duration::MAX when the product overflows what `from_nanos`-equivalent
        // construction can hold.
        let secs = nanos / 1_000_000_000;
        let sub = (nanos % 1_000_000_000) as u32;
        let secs_u64 = u64::try_from(secs).ok()?;
        Some(Duration::new(secs_u64, sub))
    }
}

/// Micro-calibrate the per-candidate cost (SPEC §6.4) by timing `samples`
/// evaluations on this machine, then return the per-candidate `Duration`.
/// `n` sets the assignment width passed to the evaluator (the identity
/// permutation `0..n` is reused for every sample — calibration measures cost,
/// not correctness). `address_index` is `0` (id-mode) or a representative
/// outer coordinate.
///
/// Returns `Duration::ZERO` if `samples == 0`. The CLI calls this at search
/// start, then feeds the result + the realized total to [`cap_decision`].
pub fn calibrate_per_candidate<E: CandidateEvaluator>(
    evaluator: &E,
    n: usize,
    samples: u32,
    address_index: u64,
) -> Duration {
    if samples == 0 {
        return Duration::ZERO;
    }
    let assignment: Vec<usize> = (0..n).collect();
    let start = Instant::now();
    for _ in 0..samples {
        // `std::hint::black_box` keeps the call from being optimized away.
        std::hint::black_box(evaluator.matches(std::hint::black_box(&assignment), address_index));
    }
    let elapsed = start.elapsed();
    elapsed / samples
}

// ---------------------------------------------------------------------------
// The search engine.
// ---------------------------------------------------------------------------

/// Realized thread count: `min(MAX_SEARCH_THREADS, available_parallelism())`,
/// clamped to ≥1.
pub fn search_threads() -> usize {
    let ncpu = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    ncpu.clamp(1, MAX_SEARCH_THREADS)
}

/// `n!` as a `u128` (the id-search candidate count). `None` on overflow
/// (`n > 34`), which the realized multisig N (≤ a handful of cosigners) never
/// approaches — but callers must handle it rather than panic.
pub fn factorial(n: usize) -> Option<u128> {
    let mut acc: u128 = 1;
    for k in 2..=n {
        acc = acc.checked_mul(k as u128)?;
    }
    Some(acc)
}

/// Unrank a lexicographic permutation index into the permutation of `0..n`
/// (Lehmer-code / factorial-number-system decode). Deterministic and
/// O(n²); `rank ∈ [0, n!)`. This is the per-candidate index → assignment map
/// that lets the parallel shards address the space by contiguous integer
/// ranges (no shared enumeration state).
fn unrank_permutation(mut rank: u128, n: usize) -> Vec<usize> {
    // Lehmer code: digit i is rank / (n-1-i)! , then reduce.
    let mut elems: Vec<usize> = (0..n).collect();
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let f = factorial(n - 1 - i).expect("n is small (search domain); n! fits u128");
        let digit = (rank / f) as usize;
        rank %= f;
        out.push(elems.remove(digit));
    }
    out
}

/// The total candidate count for a mode over `n` slots: `n!` for id-search,
/// `n! × outer_count` for address-search. `None` on `n!` overflow.
pub fn total_candidates(n: usize, mode: SearchMode) -> Option<u128> {
    let perms = factorial(n)?;
    match mode {
        SearchMode::Id => Some(perms),
        SearchMode::Address(range) => perms.checked_mul(u128::from(range.outer_count())),
    }
}

/// A single match record (internal): the permutation rank + the outer
/// address coordinate it matched at. Kept compact so the per-thread match
/// buffers stay small.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Match {
    perm_rank: u128,
    address_index: u64,
}

/// Search the candidate space and return the funds-safe [`SearchOutcome`].
///
/// **Unique-vs-Ambiguous correctness (the load-bearing decision).** To return
/// [`SearchOutcome::Unique`] the engine MUST know there is no SECOND match.
/// It therefore does NOT stop at the first match: it scans until either the
/// whole space is exhausted (→ count is 0 or 1) or a SECOND distinct match is
/// found (→ short-circuit → `Ambiguous`). Concretely each thread reports its
/// matches; a shared atomic counts global matches and, once it reaches 2, sets
/// a stop flag every thread polls — so the worst case (no match, or unique
/// match) is a full scan, while ambiguity short-circuits at the 2nd hit. This
/// is the funds-safe resolution of the "full-scan vs short-circuit" trade-off:
/// we favor full-scan for ambiguity DETECTION and never report `Unique`
/// without having ruled out a 2nd match.
///
/// **Ordering.** For [`SearchMode::Address`] the engine iterates address index
/// OUTER, permutations INNER (SPEC §6.3): the flattened index space is laid
/// out as `outer * n! + perm_rank`, so contiguous low integers are the lowest
/// address indices across all permutations. A low-index target is therefore
/// found while the high indices are still unscanned — though the engine still
/// completes (or short-circuits at the 2nd match) to honor the funds-safe
/// uniqueness guarantee.
///
/// Determinism: the outcome is a pure function of the collected match COUNT
/// (and, for `Unique`, the single match record), so parallel and
/// single-threaded runs over the same input return identical outcomes.
pub fn search<E: CandidateEvaluator>(
    n: usize,
    evaluator: &E,
    mode: SearchMode,
) -> Result<SearchOutcome, SearchError> {
    if n == 0 {
        return Err(SearchError::EmptySearchSpace);
    }
    // M1: a hostile/garbage slot count whose `n!` overflows `u128` must REFUSE,
    // not panic. `factorial`/`total_candidates` return `None` on overflow; we
    // propagate it as a typed error (the realized multisig N never approaches
    // `n > 34`). `unrank_permutation` below is only reached for the validated
    // small `n`, so its internal `factorial(..).expect(..)` cannot fire.
    let perms = factorial(n).ok_or(SearchError::SearchSpaceTooLarge { n })?;
    let outer = match mode {
        SearchMode::Id => 1u128,
        SearchMode::Address(range) => u128::from(range.outer_count()),
    };
    let total = perms
        .checked_mul(outer)
        .ok_or(SearchError::SearchSpaceTooLarge { n })?;
    if total == 0 {
        // Empty address range → no candidates → no match.
        return Ok(SearchOutcome::None);
    }

    let nthreads = search_threads().min(usize_from_u128_clamped(total));
    let matches: Mutex<Vec<Match>> = Mutex::new(Vec::new());
    let global_matches = AtomicUsize::new(0);
    let stop = AtomicBool::new(false);

    // Shard the contiguous index space [0, total) into `nthreads` near-equal
    // chunks. Each thread unranks its slice (outer = idx / perms, perm_rank =
    // idx % perms — address index OUTER) and evaluates.
    let chunk = total.div_ceil(nthreads as u128);
    std::thread::scope(|scope| {
        for t in 0..nthreads {
            let start = (t as u128) * chunk;
            if start >= total {
                break;
            }
            let end = (start + chunk).min(total);
            let matches = &matches;
            let global_matches = &global_matches;
            let stop = &stop;
            let evaluator_ref = evaluator;
            scope.spawn(move || {
                let mut local: Vec<Match> = Vec::new();
                let mut since_check: u64 = 0;
                for idx in start..end {
                    // Poll the stop flag every 1024 candidates (cheap; bounds
                    // the over-scan past a discovered 2nd match).
                    since_check += 1;
                    if since_check >= 1024 {
                        since_check = 0;
                        if stop.load(Ordering::Relaxed) {
                            break;
                        }
                    }
                    let outer_k = (idx / perms) as u64;
                    let perm_rank = idx % perms;
                    let assignment = unrank_permutation(perm_rank, n);
                    let address_index = match mode {
                        SearchMode::Id => 0,
                        SearchMode::Address(range) => range.flatten(outer_k),
                    };
                    if evaluator_ref.matches(&assignment, address_index) {
                        local.push(Match {
                            perm_rank,
                            address_index,
                        });
                        // Bump the global counter; once ≥2 globally, signal stop.
                        let prior = global_matches.fetch_add(1, Ordering::Relaxed);
                        if prior + 1 >= 2 {
                            stop.store(true, Ordering::Relaxed);
                            break;
                        }
                    }
                }
                if !local.is_empty() {
                    matches.lock().unwrap().extend(local);
                }
            });
        }
    });

    let found = matches.into_inner().unwrap();
    match found.len() {
        0 => Ok(SearchOutcome::None),
        1 => {
            let m = found[0];
            Ok(SearchOutcome::Unique {
                assignment: unrank_permutation(m.perm_rank, n),
                address_index: m.address_index,
            })
        }
        _ => Ok(SearchOutcome::Ambiguous),
    }
}

/// Single-threaded reference search — the determinism oracle for the tests and
/// a deterministic fallback. Full-scan (no 2nd-match short-circuit) so it is a
/// pure reference; the parallel [`search`] must agree with it on the outcome
/// for every input.
pub fn search_reference<E: CandidateEvaluator>(
    n: usize,
    evaluator: &E,
    mode: SearchMode,
) -> Result<SearchOutcome, SearchError> {
    if n == 0 {
        return Err(SearchError::EmptySearchSpace);
    }
    // M1: same overflow refusal as `search` (keep the reference oracle total).
    let perms = factorial(n).ok_or(SearchError::SearchSpaceTooLarge { n })?;
    let outer = match mode {
        SearchMode::Id => 1u128,
        SearchMode::Address(range) => u128::from(range.outer_count()),
    };
    let total = perms
        .checked_mul(outer)
        .ok_or(SearchError::SearchSpaceTooLarge { n })?;
    if total == 0 {
        return Ok(SearchOutcome::None);
    }
    let mut found: Vec<Match> = Vec::new();
    for idx in 0..total {
        let outer_k = (idx / perms) as u64;
        let perm_rank = idx % perms;
        let assignment = unrank_permutation(perm_rank, n);
        let address_index = match mode {
            SearchMode::Id => 0,
            SearchMode::Address(range) => range.flatten(outer_k),
        };
        if evaluator.matches(&assignment, address_index) {
            found.push(Match {
                perm_rank,
                address_index,
            });
            if found.len() >= 2 {
                // Two matches is enough to decide Ambiguous; stop (matches the
                // parallel engine's semantics — outcome, not match list).
                return Ok(SearchOutcome::Ambiguous);
            }
        }
    }
    match found.len() {
        0 => Ok(SearchOutcome::None),
        1 => {
            let m = found[0];
            Ok(SearchOutcome::Unique {
                assignment: unrank_permutation(m.perm_rank, n),
                address_index: m.address_index,
            })
        }
        _ => Ok(SearchOutcome::Ambiguous),
    }
}

/// Clamp a `u128` to `usize` (saturating) — used to cap the thread count at
/// the candidate count so we never spawn more threads than candidates.
fn usize_from_u128_clamped(v: u128) -> usize {
    usize::try_from(v).unwrap_or(usize::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- required_prefix_bytes: the realized-S byte ladder (SPEC §6.2). -----

    /// Helper mirroring the realized-S enumeration: the subset×permutation
    /// count `P((n-own)+K, n)` for `--own-account-max K`.
    fn perm_count(n: u128, r: u128) -> u128 {
        let mut p = 1u128;
        for i in 0..r {
            p *= n - i;
        }
        p
    }

    #[test]
    fn prefix_ladder_n11_account_list() {
        // Explicit --account LIST → S = n! (n = 11).
        let s = factorial(11).unwrap();
        assert_eq!(s, 39_916_800);
        assert_eq!(required_prefix_bytes(s), 8);
    }

    #[test]
    fn prefix_ladder_own_account_max_subset_space() {
        // n = 11, own = 4 → cosigner candidate pool = (11-4)+K = 7+K,
        // arranged into the 11 slots: S = P(7+K, 11). SPEC §6.2 ladder.
        let n = 11u128;
        let own = 4u128;
        let cases = [(8u128, 9usize), (16, 10), (32, 11), (64, 13)];
        for (k, expected) in cases {
            let s = perm_count((n - own) + k, n);
            assert_eq!(
                required_prefix_bytes(s),
                expected,
                "K={k}: S=P({}, {n})={s} expected {expected}B",
                (n - own) + k
            );
        }
    }

    #[test]
    fn prefix_floor_small_spaces() {
        // S <= 1 → 0 bits of choice → ceil(32/8) = 4 bytes (the 4-byte
        // minimum, a single-candidate recompute).
        assert_eq!(required_prefix_bytes(0), 4);
        assert_eq!(required_prefix_bytes(1), 4);
        // S = 2 → 1 bit → ceil(33/8) = 5 bytes.
        assert_eq!(required_prefix_bytes(2), 5);
    }

    #[test]
    fn validate_prefix_strength_rejects_short_accepts_long() {
        let s = factorial(11).unwrap(); // needs 8 bytes
                                        // a 4-byte prefix is too weak.
        assert_eq!(
            validate_prefix_strength(4, s),
            Err(SearchError::PrefixTooShort {
                required: 8,
                supplied: 4
            })
        );
        // 8 bytes exactly is accepted; 16 (full id) is accepted.
        assert!(validate_prefix_strength(8, s).is_ok());
        assert!(validate_prefix_strength(16, s).is_ok());
    }

    // -- reject_duplicate_keys: floor 2. ------------------------------------

    #[test]
    fn duplicate_keys_rejected() {
        // Simulate 65-byte key blobs with byte vectors.
        let k0 = vec![1u8; 65];
        let k1 = vec![2u8; 65];
        let k2 = vec![1u8; 65]; // duplicate of k0
        let keys = vec![k0, k1, k2];
        assert_eq!(
            reject_duplicate_keys(&keys),
            Err(SearchError::DuplicateKeys { a: 0, b: 2 })
        );
    }

    #[test]
    fn distinct_keys_ok() {
        let keys = vec![vec![1u8; 65], vec![2u8; 65], vec![3u8; 65]];
        assert!(reject_duplicate_keys(&keys).is_ok());
    }

    // -- engine over small N with synthetic evaluators. ---------------------

    /// Evaluator that matches iff the assignment equals a known target perm.
    fn target_eval(target: Vec<usize>) -> impl CandidateEvaluator {
        move |a: &[usize], _idx: u64| a == target.as_slice()
    }

    #[test]
    fn engine_resolves_unique_target_n5() {
        let target = vec![3, 1, 4, 0, 2];
        let outcome = search(5, &target_eval(target.clone()), SearchMode::Id).unwrap();
        match outcome {
            SearchOutcome::Unique {
                assignment,
                address_index,
            } => {
                assert_eq!(assignment, target);
                assert_eq!(address_index, 0);
            }
            other => panic!("expected Unique, got {other:?}"),
        }
    }

    #[test]
    fn engine_resolves_unique_target_n6() {
        let target = vec![5, 0, 3, 2, 4, 1];
        let outcome = search(6, &target_eval(target.clone()), SearchMode::Id).unwrap();
        assert_eq!(
            outcome,
            SearchOutcome::Unique {
                assignment: target,
                address_index: 0
            }
        );
    }

    #[test]
    fn engine_no_match_is_none() {
        // Evaluator that never matches.
        let never = |_a: &[usize], _idx: u64| false;
        assert_eq!(search(6, &never, SearchMode::Id).unwrap(), SearchOutcome::None);
    }

    #[test]
    fn engine_two_targets_is_ambiguous() {
        let t1 = vec![0, 1, 2, 3, 4];
        let t2 = vec![4, 3, 2, 1, 0];
        let two = move |a: &[usize], _idx: u64| a == t1.as_slice() || a == t2.as_slice();
        assert_eq!(search(5, &two, SearchMode::Id).unwrap(), SearchOutcome::Ambiguous);
    }

    #[test]
    fn engine_n1_trivial_unique() {
        // n == 1: a single permutation [0]; matching evaluator → Unique.
        let always = |_a: &[usize], _idx: u64| true;
        assert_eq!(
            search(1, &always, SearchMode::Id).unwrap(),
            SearchOutcome::Unique {
                assignment: vec![0],
                address_index: 0
            }
        );
    }

    #[test]
    fn engine_n0_is_empty_search_space() {
        let always = |_a: &[usize], _idx: u64| true;
        assert_eq!(
            search(0, &always, SearchMode::Id),
            Err(SearchError::EmptySearchSpace)
        );
    }

    // -- Determinism: parallel == single-threaded reference. ----------------

    #[test]
    fn parallel_matches_reference_unique() {
        let target = vec![4, 2, 6, 0, 5, 1, 3];
        let eval = target_eval(target.clone());
        let par = search(7, &eval, SearchMode::Id).unwrap();
        let single = search_reference(7, &eval, SearchMode::Id).unwrap();
        assert_eq!(par, single);
        assert_eq!(
            par,
            SearchOutcome::Unique {
                assignment: target,
                address_index: 0
            }
        );
    }

    #[test]
    fn parallel_matches_reference_none_and_ambiguous() {
        let never = |_a: &[usize], _idx: u64| false;
        assert_eq!(
            search(7, &never, SearchMode::Id).unwrap(),
            search_reference(7, &never, SearchMode::Id).unwrap()
        );

        let t1 = vec![0, 1, 2, 3, 4, 5];
        let t2 = vec![5, 4, 3, 2, 1, 0];
        let two = move |a: &[usize], _idx: u64| a == t1.as_slice() || a == t2.as_slice();
        assert_eq!(
            search(6, &two, SearchMode::Id).unwrap(),
            SearchOutcome::Ambiguous
        );
        assert_eq!(
            search_reference(6, &two, SearchMode::Id).unwrap(),
            SearchOutcome::Ambiguous
        );
    }

    #[test]
    fn thread_count_capped_at_20_and_ncpu() {
        let t = search_threads();
        assert!(t >= 1);
        assert!(t <= MAX_SEARCH_THREADS);
        let ncpu = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        assert_eq!(t, ncpu.clamp(1, MAX_SEARCH_THREADS));
    }

    // -- Permutation unranking sanity (the parallel addressing primitive). --

    #[test]
    fn unrank_covers_all_permutations_bijectively() {
        // Every rank in [0, n!) yields a distinct permutation; all are valid.
        let n = 5;
        let total = factorial(n).unwrap();
        let mut seen = std::collections::HashSet::new();
        for r in 0..total {
            let p = unrank_permutation(r, n);
            // valid permutation of 0..n
            let mut sorted = p.clone();
            sorted.sort_unstable();
            assert_eq!(sorted, (0..n).collect::<Vec<_>>());
            assert!(seen.insert(p), "rank {r} collided");
        }
        assert_eq!(seen.len() as u128, total);
    }

    // -- Address-search: ascending-index OUTER. -----------------------------

    #[test]
    fn address_search_finds_low_index_match() {
        // n = 3, range [0, 20), receive only. The target matches at a SPECIFIC
        // address_index (low) for a SPECIFIC permutation. Ascending-index
        // OUTER means a low index is reached early; we assert the engine
        // resolves the unique (perm, idx).
        let target_perm = vec![2, 0, 1];
        let target_addr_idx = AddressRange {
            min: 0,
            max: 20,
            chains: ChainScope::Receive,
        }
        .flatten(3); // the 4th outer coordinate (index 3, receive).
        let eval = move |a: &[usize], idx: u64| {
            a == target_perm.as_slice() && idx == target_addr_idx
        };
        let range = AddressRange {
            min: 0,
            max: 20,
            chains: ChainScope::Receive,
        };
        let outcome = search(3, &eval, SearchMode::Address(range)).unwrap();
        match outcome {
            SearchOutcome::Unique {
                assignment,
                address_index,
            } => {
                assert_eq!(assignment, vec![2, 0, 1]);
                assert_eq!(address_index, target_addr_idx);
            }
            other => panic!("expected Unique, got {other:?}"),
        }
    }

    #[test]
    fn address_range_flatten_is_ascending_index_outer() {
        // With Both chains, the first two outer coords are index `min` (both
        // chains), then index min+1 (both chains), … — index OUTER, chain
        // INNER. The decoded idx must be monotonically non-decreasing in k.
        let range = AddressRange {
            min: 5,
            max: 8,
            chains: ChainScope::Both,
        };
        assert_eq!(range.outer_count(), 6); // 3 indices × 2 chains
        let mut last_idx = 0u64;
        for k in 0..range.outer_count() {
            let flat = range.flatten(k);
            let idx = flat >> 1;
            assert!(idx >= last_idx, "k={k}: idx {idx} < last {last_idx}");
            last_idx = idx;
        }
        // First coordinate is the lowest index (min = 5).
        assert_eq!(range.flatten(0) >> 1, 5);
        // Last coordinate is the highest index (max-1 = 7).
        assert_eq!(range.flatten(5) >> 1, 7);
    }

    #[test]
    fn address_range_empty_is_none() {
        // Inverted / empty range → no candidates → None.
        let always = |_a: &[usize], _idx: u64| true;
        let range = AddressRange {
            min: 10,
            max: 10,
            chains: ChainScope::Receive,
        };
        assert_eq!(range.outer_count(), 0);
        assert_eq!(
            search(3, &always, SearchMode::Address(range)).unwrap(),
            SearchOutcome::None
        );
    }

    // -- total_candidates / factorial. --------------------------------------

    #[test]
    fn total_candidates_id_and_address() {
        assert_eq!(total_candidates(5, SearchMode::Id), Some(120));
        let range = AddressRange {
            min: 0,
            max: 20,
            chains: ChainScope::Both,
        };
        // 5! × (20 × 2) = 120 × 40 = 4800
        assert_eq!(total_candidates(5, SearchMode::Address(range)), Some(4800));
    }

    #[test]
    fn factorial_overflow_is_none() {
        assert_eq!(factorial(0), Some(1));
        assert_eq!(factorial(11), Some(39_916_800));
        // 35! overflows u128.
        assert_eq!(factorial(35), None);
    }

    // -- Adaptive cap (SPEC §6.4). ------------------------------------------

    #[test]
    fn cap_decision_silent_below_30s() {
        // 1000 candidates × 1µs = 1ms < 30s → silent.
        let d = cap_decision(1000, Duration::from_micros(1), None).unwrap();
        assert_eq!(
            d,
            CapDecision::RunSilent {
                estimate: Duration::from_millis(1)
            }
        );
    }

    #[test]
    fn cap_decision_progress_between_30s_and_1h() {
        // 600 candidates × 1s = 600s (10 min): 30s ≤ est ≤ 1h → progress.
        let d = cap_decision(600, Duration::from_secs(1), None).unwrap();
        assert_eq!(
            d,
            CapDecision::RunWithProgress {
                estimate: Duration::from_secs(600)
            }
        );
    }

    #[test]
    fn cap_decision_above_ceiling_refuses_without_accept() {
        // 7200 candidates × 1s = 7200s (2h) > 1h ceiling, no override → refuse.
        let est = Duration::from_secs(7200);
        assert_eq!(
            cap_decision(7200, Duration::from_secs(1), None),
            Err(SearchError::SearchTimeExceedsCeiling {
                estimate: est,
                ceiling: SEARCH_CEILING
            })
        );
    }

    #[test]
    fn cap_decision_above_ceiling_accepts_with_sufficient_override() {
        // Same 2h estimate, override ≥ estimate → run with progress.
        let est = Duration::from_secs(7200);
        let d = cap_decision(7200, Duration::from_secs(1), Some(est)).unwrap();
        assert_eq!(d, CapDecision::RunWithProgress { estimate: est });
        // A larger override is also fine.
        let d2 =
            cap_decision(7200, Duration::from_secs(1), Some(Duration::from_secs(10_000))).unwrap();
        assert_eq!(d2, CapDecision::RunWithProgress { estimate: est });
    }

    #[test]
    fn cap_decision_above_ceiling_rejects_insufficient_override() {
        // Override below the estimate → AcceptSearchTimeTooLow.
        let est = Duration::from_secs(7200);
        assert_eq!(
            cap_decision(7200, Duration::from_secs(1), Some(Duration::from_secs(60))),
            Err(SearchError::AcceptSearchTimeTooLow {
                estimate: est,
                supplied: Duration::from_secs(60)
            })
        );
    }

    #[test]
    fn cap_estimate_with_synthetic_slow_evaluator_exceeds_ceiling() {
        // A synthetic SLOW evaluator (~each call costs a measurable amount):
        // calibrate, then estimate over a large realized space → exceeds the
        // ceiling → requires accept_search_time (forced acknowledgment).
        let slow = |_a: &[usize], _idx: u64| {
            // ~a few µs of work per candidate.
            let mut x = 0u64;
            for i in 0..2000u64 {
                x = x.wrapping_add(i).wrapping_mul(2654435761);
            }
            std::hint::black_box(x);
            false
        };
        let per = calibrate_per_candidate(&slow, 11, 64, 0);
        assert!(per > Duration::ZERO, "calibration measured non-zero cost");
        // A huge realized space (e.g. a 13-slot search ≈ 6.2e9 perms) at any
        // measurable per-candidate cost blows past 1h.
        let huge_total: u64 = 6_227_020_800; // 13!
        let res = cap_decision(huge_total, per, None);
        assert!(
            matches!(res, Err(SearchError::SearchTimeExceedsCeiling { .. })),
            "expected ceiling refusal, got {res:?}"
        );
        // With the forced acknowledgment (≥ estimate) it proceeds.
        if let Err(SearchError::SearchTimeExceedsCeiling { estimate, .. }) = res {
            let ok = cap_decision(huge_total, per, Some(estimate)).unwrap();
            assert_eq!(ok, CapDecision::RunWithProgress { estimate });
        }
    }

    #[test]
    fn calibrate_zero_samples_is_zero() {
        let eval = |_a: &[usize], _idx: u64| true;
        assert_eq!(calibrate_per_candidate(&eval, 5, 0, 0), Duration::ZERO);
    }

    // -- M1: hostile slot count refuses (no panic). -------------------------

    #[test]
    fn search_refuses_overflowing_slot_count_without_panic() {
        // n = 35 → 35! overflows u128. Both the parallel engine and the
        // single-threaded reference must return a typed REFUSAL, never panic
        // (M1: `factorial(n)` is `?`-propagated, not `.expect()`-unwrapped).
        let always = |_a: &[usize], _idx: u64| true;
        assert_eq!(
            search(35, &always, SearchMode::Id),
            Err(SearchError::SearchSpaceTooLarge { n: 35 })
        );
        assert_eq!(
            search_reference(35, &always, SearchMode::Id),
            Err(SearchError::SearchSpaceTooLarge { n: 35 })
        );
        // The address-mode multiply can also overflow even when n! fits; a huge
        // range over a moderate n is refused too rather than panicking.
        let range = AddressRange {
            min: 0,
            max: u32::MAX,
            chains: ChainScope::Both,
        };
        // n = 34 → 34! is the largest factorial that fits u128; × the range
        // overflows the product → SearchSpaceTooLarge (the checked_mul leg).
        assert_eq!(
            search(34, &always, SearchMode::Address(range)),
            Err(SearchError::SearchSpaceTooLarge { n: 34 })
        );
    }

    // -- M2: cross-module address-index encoding round-trip. -----------------

    /// The address-mode `address_index` handed to the evaluator is the
    /// engine↔evaluator contract: `(idx << 1) | chain_bit`. P3/P4 evaluators
    /// DECODE it as `idx = address_index >> 1`, `chain = address_index & 1`.
    /// This pins that exact codec so a future change to `AddressRange::flatten`
    /// cannot silently desync the two sides (the evaluator would derive at the
    /// wrong (chain, idx) and silently mis-match scriptPubKeys — funds-safety).
    #[test]
    fn address_index_encoding_round_trips_engine_to_evaluator() {
        for &chains in &[ChainScope::Receive, ChainScope::Change, ChainScope::Both] {
            let range = AddressRange {
                min: 3,
                max: 9,
                chains,
            };
            for k in 0..range.outer_count() {
                let flat = range.flatten(k);
                // The evaluator-side decode (the contract P3/P4 implement).
                let decoded_idx = (flat >> 1) as u32;
                let decoded_chain_bit = (flat & 1) as u32;
                // The engine-side ground truth this k corresponds to.
                let chain_count = chains.count();
                let expected_idx = range.min + (k / u64::from(chain_count)) as u32;
                let expected_chain = (k % u64::from(chain_count)) as u32;
                assert_eq!(
                    decoded_idx, expected_idx,
                    "k={k} chains={chains:?}: decoded idx {decoded_idx} != {expected_idx}"
                );
                assert_eq!(
                    decoded_chain_bit, expected_chain,
                    "k={k} chains={chains:?}: decoded chain {decoded_chain_bit} != {expected_chain}"
                );
                // And the round-trip is total: re-encoding the decoded (idx,chain)
                // reproduces the same flattened value.
                assert_eq!(
                    flat,
                    (u64::from(decoded_idx) << 1) | u64::from(decoded_chain_bit),
                    "k={k}: re-encode mismatch"
                );
            }
        }
    }
}
