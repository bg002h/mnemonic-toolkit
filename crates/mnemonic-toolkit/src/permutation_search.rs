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

// ===========================================================================
// P1 — own-account subset-search combinatorics (SPEC §3/§4).
//
// Pool-index CONVENTION (P2 MUST follow this): the search pool is laid out
// OWN-FIRST — own-account candidates occupy indices `0..k_own`, cosigner cards
// occupy indices `k_own..k_own+m`. Every generator below returns a
// `Vec<usize>` whose i-th element is the FULL-pool index assigned to slot `i`
// (slot → pool-index), in this same own-first layout. P2 builds its candidate
// `key65`/origin pool in exactly this order so the returned indices address the
// right keys.
//
// All cardinality helpers are overflow-checked → `Option<u128>`; `None` means
// REFUSE (never panic — the #28 M1 lesson; e.g. `C(256,128)` is a 252-bit
// number that cannot fit `u128`).
// ===========================================================================

/// `C(k, r)` (combinations / the binomial coefficient), overflow-checked. The
/// combinatorial-number-system (CNS) stratum count. Returns:
/// - `Some(0)` when `r > k` (no such subset),
/// - `Some(1)` when `r == 0`,
/// - `None` on `u128` overflow (e.g. `C(256, 128)` is 252-bit → refuse).
///
/// Computed via the multiplicative formula on the smaller of `r` / `k-r`
/// (numerically exact, no factorial blow-up): each step multiplies then divides
/// exactly (the running product `C(k, i)` is always integral), with a
/// `checked_mul` so an overflow at any step refuses.
pub fn c_choose(k: usize, r: usize) -> Option<u128> {
    if r > k {
        return Some(0);
    }
    // C(k,r) == C(k,k-r); pick the smaller to minimize the number of steps and
    // keep intermediates small.
    let r = r.min(k - r);
    let mut acc: u128 = 1;
    // acc holds C(k, i) after step i; C(k,i) = C(k,i-1) * (k-i+1) / i.
    for i in 1..=r {
        acc = acc.checked_mul((k - i + 1) as u128)?;
        // Division is exact: acc was C(k,i-1)*(k-i+1) which is divisible by i.
        acc /= i as u128;
    }
    Some(acc)
}

/// `P(pool, n)` — the count of injective placements of `n` of `pool` items into
/// `n` ordered slots: `pool · (pool−1) · … · (pool−n+1)`. Overflow-checked
/// (`None` on `u128` overflow). Returns `Some(0)` when `n > pool` (no injective
/// placement exists). This is the cardinality of [`unrank_kperm`]'s domain.
pub fn p_count(pool: usize, n: usize) -> Option<u128> {
    if n > pool {
        return Some(0);
    }
    let mut acc: u128 = 1;
    for i in 0..n {
        acc = acc.checked_mul((pool - i) as u128)?;
    }
    Some(acc)
}

/// The own-only enumerated count `S_own` (SPEC §3):
/// - non-sorted: `C(K_own, j) · N!` where `N = j + m`,
/// - sorted shape (order-independent): `C(K_own, j)` (drop the `N!` factor —
///   one identity-ordered placement per subset).
///
/// Overflow at ANY step (`c_choose` or `factorial`) → `None` → the caller
/// refuses. Collapses to `N!` (non-sorted) / `1` (sorted) at `K_own == j`
/// (no over-supply; byte-identical to the v0.60.0 exact path's `N!`).
pub fn s_own(k_own: usize, j: usize, m: usize, sorted: bool) -> Option<u128> {
    let n = j.checked_add(m)?;
    let combos = c_choose(k_own, j)?;
    if sorted {
        Some(combos)
    } else {
        combos.checked_mul(factorial(n)?)
    }
}

/// The opt-in enumerated count `S_opt` (SPEC §4.3): the search ranges over
/// `(own-subset, cosigner-subset, ordering)` for the supplied `K_own` own +
/// `M_sup` cosigner candidates filling `N` slots with `j` own + `(N−j)`
/// cosigner, summed over the valid `j`-strata:
/// - non-sorted: `Σ_j C(K_own, j) · C(M_sup, N−j) · N!`,
/// - sorted shape: `Σ_j C(K_own, j) · C(M_sup, N−j)`.
///
/// `j ∈ [1, min(K_own, N−1)]` (own ≥1 via `--from`, ≥1 cosigner). The strata are
/// disjoint by own-slot-count `j` ⇒ no double-count. Every term AND the running
/// sum is overflow-checked → `None` refuses.
pub fn s_opt(k_own: usize, m_sup: usize, n: usize, sorted: bool) -> Option<u128> {
    let nfact = if sorted { 1u128 } else { factorial(n)? };
    let j_max = k_own.min(n.saturating_sub(1));
    let mut sum: u128 = 0;
    for j in 1..=j_max {
        let need_cos = n - j; // n - j ≥ 1 since j ≤ n-1
        if need_cos > m_sup {
            continue; // not enough cosigner candidates for this stratum
        }
        let own = c_choose(k_own, j)?;
        let cos = c_choose(m_sup, need_cos)?;
        let term = own.checked_mul(cos)?.checked_mul(nfact)?;
        sum = sum.checked_add(term)?;
    }
    Some(sum)
}

/// The total candidate count for a SUBSET search that drives exactly `s`
/// candidate assignments (`s = S_own` / `S_own_sorted` / `S_opt`, per §3) —
/// `s` for id-search, `s × outer_count` for address-search. `None` on overflow.
/// This is the subset-mode analogue of [`total_candidates`] (which assumes the
/// `n!` exact-pool count); P2 passes the realized `s` here.
pub fn total_candidates_subset(mode: SearchMode, s: u128) -> Option<u128> {
    match mode {
        SearchMode::Id => Some(s),
        SearchMode::Address(range) => s.checked_mul(u128::from(range.outer_count())),
    }
}

/// Unrank a lexicographic INJECTIVE k-permutation: place `n` of `pool` items
/// into `n` ordered slots, returning the `pool`-indices (slot → pool-index).
/// `rank ∈ [0, P(pool, n))`. The order is lexicographic over the chosen
/// pool-index tuples (rank 0 = `[0, 1, …, n−1]`).
///
/// Lehmer/factorial-number-system style: at slot `i` the remaining-pool size is
/// `pool − i`, and there are `P(pool−1−i, n−1−i)` arrangements per choice of the
/// current slot, so `digit = rank / that`, then reduce. Used by the OPT-IN /
/// uniform path; the own-anchored generator composes `c_choose`-unrank with
/// [`unrank_permutation`] instead (it must EXCLUDE cosigner-dropping
/// placements, which a plain `unrank_kperm` over the whole pool would include).
pub fn unrank_kperm(mut rank: u128, pool: usize, n: usize) -> Vec<usize> {
    let mut elems: Vec<usize> = (0..pool).collect();
    let mut out = Vec::with_capacity(n);
    for i in 0..n {
        let remaining = n - 1 - i;
        // Number of ways to fill the REMAINING slots from the REMAINING pool
        // after this pick: P(elems.len()-1, remaining).
        let block = p_count(elems.len() - 1, remaining)
            .expect("subset search domain is small; P(pool,n) fits u128");
        let digit = (rank / block) as usize;
        rank %= block;
        out.push(elems.remove(digit));
    }
    out
}

/// Unrank a `combo_rank ∈ [0, C(k, r))` into the `rank`-th `r`-subset of
/// `0..k`, as an ASCENDING `Vec<usize>` (the combinatorial-number-system
/// unrank). Lexicographic over ascending subsets (rank 0 = `[0,1,…,r−1]`).
///
/// At each position we pick the smallest next element `x ≥ start` such that the
/// number of completions with a SMALLER `x` has been skipped: the count of
/// `(r−1−i)`-subsets of the elements above `x` is `C(k−1−x, r−1−i)`.
fn unrank_combination(mut combo_rank: u128, k: usize, r: usize) -> Vec<usize> {
    let mut out = Vec::with_capacity(r);
    let mut x = 0usize;
    let mut remaining = r;
    while remaining > 0 {
        // For candidate element `x`, completions choosing `x` then `remaining-1`
        // from `x+1..k` number C(k-1-x, remaining-1).
        let with_x =
            c_choose(k - 1 - x, remaining - 1).expect("subset search domain is small; C fits u128");
        if combo_rank < with_x {
            out.push(x);
            remaining -= 1;
            x += 1;
        } else {
            combo_rank -= with_x;
            x += 1;
        }
    }
    out
}

/// The OWN-ANCHORED composed-rank generator (SPEC §3/§4.1). Returns the
/// `rank`-th assignment in the `S_own` space as `Vec<slot→pool-index>` in the
/// own-first pool layout (own `0..k_own`, cosigners `k_own..k_own+m`).
///
/// `rank ∈ [0, S_own)` where `S_own = s_own(k_own, j, m, sorted)`.
///
/// - **non-sorted:** `combo_rank = rank / N!`, `perm_rank = rank % N!`
///   (`N = j+m`). `combo_rank` → the `j`-subset of own indices via
///   [`unrank_combination`]; the `N` selected pool entries are `(j chosen own)
///   ++ (all m cosigners, ascending k_own..k_own+m)`; `perm_rank →
///   unrank_permutation(perm_rank, N)` orders the selected entries into the N
///   slots. **Bijective onto exactly `S_own`** — every assignment uses exactly
///   `j` own + ALL `m` cosigners, each once, nothing else (no cosigner-dropping
///   placement).
/// - **sorted:** drop `perm_rank` — emit each of the `C(k_own, j)` subsets ONCE
///   in canonical identity order (selected entries as-is).
pub fn own_anchored_unrank(
    rank: u128,
    k_own: usize,
    j: usize,
    m: usize,
    sorted: bool,
) -> Vec<usize> {
    let n = j + m;
    let (combo_rank, perm_rank) = if sorted {
        (rank, 0u128)
    } else {
        let nfact = factorial(n).expect("subset search domain is small; N! fits u128");
        (rank / nfact, rank % nfact)
    };
    let own_subset = unrank_combination(combo_rank, k_own, j);
    // The N selected pool entries: j chosen own ++ all m cosigners (ascending).
    let mut selected: Vec<usize> = own_subset;
    for c in 0..m {
        selected.push(k_own + c);
    }
    if sorted {
        // Identity order — the selected entries as-is.
        selected
    } else {
        // Order the N selected entries into the N slots by perm_rank.
        let order = unrank_permutation(perm_rank, n);
        order.into_iter().map(|p| selected[p]).collect()
    }
}

/// The OPT-IN STRATIFIED generator (SPEC §4.3). Returns the `rank`-th assignment
/// in the `S_opt` space as `Vec<slot→pool-index>` in the own-first pool layout
/// (own `0..k_own`, cosigners `k_own..k_own+m_sup`).
///
/// `rank ∈ [0, S_opt)` where `S_opt = s_opt(k_own, m_sup, n, sorted)`.
///
/// The space is partitioned into disjoint `j`-strata (`j` = own-slot-count,
/// `j ∈ [1, min(k_own, n−1)]`), each of size
/// `C(k_own,j)·C(m_sup,n−j)·N!` (non-sorted) / `C(k_own,j)·C(m_sup,n−j)`
/// (sorted). We locate `rank`'s stratum by cumulative size, then within the
/// stratum compose `(own-combo CNS-unrank, cosigner-combo CNS-unrank,
/// perm-unrank)`. Bijective onto `S_opt` by the §4.1 argument applied per
/// stratum (disjoint ⇒ no double-count).
pub fn opt_in_unrank(
    mut rank: u128,
    k_own: usize,
    m_sup: usize,
    n: usize,
    sorted: bool,
) -> Vec<usize> {
    let nfact = if sorted {
        1u128
    } else {
        factorial(n).expect("subset search domain is small; N! fits u128")
    };
    let j_max = k_own.min(n.saturating_sub(1));
    for j in 1..=j_max {
        let need_cos = n - j;
        if need_cos > m_sup {
            continue;
        }
        let own_combos = c_choose(k_own, j).expect("small domain; C fits u128");
        let cos_combos = c_choose(m_sup, need_cos).expect("small domain; C fits u128");
        // Stratum size: own_combos · cos_combos · nfact.
        let stratum = own_combos * cos_combos * nfact;
        if rank < stratum {
            // Decompose the in-stratum rank as
            // own_rank · (cos_combos·nfact) + cos_rank · nfact + perm_rank.
            let perm_rank = rank % nfact;
            let rest = rank / nfact; // ∈ [0, own_combos·cos_combos)
            let cos_rank = rest % cos_combos;
            let own_rank = rest / cos_combos;
            let own_subset = unrank_combination(own_rank, k_own, j);
            let cos_subset = unrank_combination(cos_rank, m_sup, need_cos);
            let mut selected: Vec<usize> = own_subset;
            for &c in &cos_subset {
                selected.push(k_own + c);
            }
            return if sorted {
                selected
            } else {
                let order = unrank_permutation(perm_rank, n);
                order.into_iter().map(|p| selected[p]).collect()
            };
        }
        rank -= stratum;
    }
    // rank ≥ S_opt — out of domain. Callers stay within [0, S_opt); a stray
    // rank yields an empty assignment rather than a panic.
    Vec::new()
}

/// The enumeration the [`search_enumerated`] engine ranks over (SPEC §4). It
/// abstracts the per-rank assignment generator + its cardinality so the SAME
/// sharded engine drives the v0.60.0 exact-pool space AND the new over-supply
/// subset spaces — the only difference is the cardinality `S` the rank range
/// `[0, S)` spans and the `unrank(rank) -> Vec<slot→pool-index>` map.
///
/// Pool-index CONVENTION (own-first; see the module header at the P1 banner):
/// own candidates occupy pool indices `0..k_own`, cosigner cards
/// `k_own..k_own+m` (`+m_sup` for the opt-in form). Every variant's `unrank`
/// returns the full-pool index assigned to each slot, in this layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Enumeration {
    /// The v0.60.0 EXACT-pool space: all `n!` full permutations of the `n` pool
    /// entries (`pool.len() == n`). Cardinality `n!`; `unrank` is the legacy
    /// [`unrank_permutation`]. The `search(n, …)` wrapper builds this so the
    /// exact path stays BYTE-IDENTICAL.
    FullPermutation {
        /// The slot count (== the exact pool size).
        n: usize,
    },
    /// The own-anchored over-supply space (SPEC §3): `k_own` own candidates,
    /// `j = N − M` own slots, `m` exact cosigners, `N = j + m` slots.
    /// Cardinality `s_own(k_own, j, m, sorted)`; `unrank` is
    /// [`own_anchored_unrank`].
    OwnAnchored {
        /// Number of own candidates (pool `0..k_own`).
        k_own: usize,
        /// Number of own slots (`= N − M`).
        j: usize,
        /// Number of exact cosigner cards (pool `k_own..k_own+m`).
        m: usize,
        /// Order-independent shape (drop the `N!` ordering factor).
        sorted: bool,
    },
    /// The opt-in over-supply space (SPEC §4.3): `k_own` own + `m_sup` cosigner
    /// candidates filling `n` slots over the valid `j`-strata. Cardinality
    /// `s_opt(k_own, m_sup, n, sorted)`; `unrank` is [`opt_in_unrank`]. (Wired by
    /// P3; the engine already drives it.)
    OptIn {
        /// Number of own candidates (pool `0..k_own`).
        k_own: usize,
        /// Number of supplied cosigner candidates (pool `k_own..k_own+m_sup`).
        m_sup: usize,
        /// The slot count.
        n: usize,
        /// Order-independent shape (drop the `N!` ordering factor).
        sorted: bool,
    },
}

impl Enumeration {
    /// The slot count `n` (the width of every `unrank` assignment / the
    /// evaluator's expected assignment length).
    pub fn n(&self) -> usize {
        match *self {
            Enumeration::FullPermutation { n } => n,
            Enumeration::OwnAnchored { j, m, .. } => j + m,
            Enumeration::OptIn { n, .. } => n,
        }
    }

    /// The number of assignments this enumeration ranks over (`S`), i.e. the
    /// PERMUTATION-space cardinality BEFORE the address-range multiplier.
    /// Overflow-checked → `None` (the caller refuses; never panic — the #28 M1
    /// lesson). **P1 M-1:** the per-rank `unrank` generators below carry internal
    /// `.expect()`s that divide this same cardinality, so `unrank` MUST be
    /// reached only after this returned `Some(_)` (the engine guards it).
    pub fn cardinality(&self) -> Option<u128> {
        match *self {
            Enumeration::FullPermutation { n } => factorial(n),
            Enumeration::OwnAnchored {
                k_own,
                j,
                m,
                sorted,
            } => s_own(k_own, j, m, sorted),
            Enumeration::OptIn {
                k_own,
                m_sup,
                n,
                sorted,
            } => s_opt(k_own, m_sup, n, sorted),
        }
    }

    /// Unrank `rank ∈ [0, cardinality())` into a `Vec<slot→pool-index>`
    /// assignment (own-first pool layout). PRECONDITION: `cardinality()` was
    /// proven `Some(_)` (the engine guards this — the generators' internal
    /// `.expect()`s divide that already-validated cardinality, so they cannot
    /// fire here; P1 M-1).
    fn unrank(&self, rank: u128) -> Vec<usize> {
        match *self {
            Enumeration::FullPermutation { n } => unrank_permutation(rank, n),
            Enumeration::OwnAnchored {
                k_own,
                j,
                m,
                sorted,
            } => own_anchored_unrank(rank, k_own, j, m, sorted),
            Enumeration::OptIn {
                k_own,
                m_sup,
                n,
                sorted,
            } => opt_in_unrank(rank, k_own, m_sup, n, sorted),
        }
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
///
/// **`early_exit` (SPEC §4.4 contract).** When `false` (the DEFAULT / every
/// v0.60.0 call site, every id-search and prefix-id path) the engine retains
/// the full-scan-with-2nd-match-ambiguity behavior described above — BYTE
/// IDENTICAL to the pre-subset-search engine: it scans the whole space to
/// certify uniqueness and only short-circuits at the SECOND match (→
/// `Ambiguous`). When `true` the engine MAY stop at the FIRST match and return
/// it as `Unique` (it sets the stop flag at match #1). This is ONLY funds-safe
/// for a COLLISION-FREE address-search (full scriptPubKey ⇒ a match is provably
/// unique — SPEC §4.4 §I-2); the over-supply address-search path opts in.
/// Prefix-id NEVER gets `early_exit=true` (it would miss a 2nd-match ambiguity →
/// silent-wrong-wallet). The caller is responsible for the gating; the engine
/// just honors the flag.
pub fn search<E: CandidateEvaluator>(
    n: usize,
    evaluator: &E,
    mode: SearchMode,
    early_exit: bool,
) -> Result<SearchOutcome, SearchError> {
    // The v0.60.0 EXACT-pool path: rank the `n!` full permutations. This is a
    // thin wrapper over [`search_enumerated`] driven by
    // `Enumeration::FullPermutation { n }` — whose cardinality is `factorial(n)`
    // and whose `unrank` is the legacy `unrank_permutation(perm_rank, n)`, so the
    // exact path is BYTE-IDENTICAL to the pre-subset-search engine (the
    // regression guard in §7 pins this).
    search_enumerated(
        &Enumeration::FullPermutation { n },
        evaluator,
        mode,
        early_exit,
    )
}

/// Search a candidate space defined by an [`Enumeration`] and return the
/// funds-safe [`SearchOutcome`]. This is the generalized engine the over-supply
/// subset-search (SPEC §4) drives: the rank space is `[0, S)` where
/// `S = enumeration.cardinality()` (the SUBSET count `s_own`/`s_opt`, NOT `n!`),
/// and the per-rank assignment comes from `enumeration.unrank(rank)` instead of
/// the hardcoded `unrank_permutation`. The sharding, stop-flag, ambiguity
/// certification, and `early_exit` semantics are IDENTICAL across all
/// enumerations — only the cardinality + per-rank generator differ (P1 confirmed
/// the shard logic is structure-agnostic: it needs only `S` + a bijective
/// `unrank`). The `search(n, …)` wrapper drives the `FullPermutation` variant so
/// every v0.60.0 call site is byte-unchanged.
///
/// **Funds-safety / `early_exit`:** see the [`search`] doc — the contract is
/// unchanged. `early_exit=true` (first-match) is ONLY funds-safe for a
/// collision-free address-search over a subset enumeration; prefix-id + the
/// exact path always pass `false`.
///
/// **P1 M-1 (overflow → refuse):** the cardinality is computed up-front via
/// `enumeration.cardinality()` and an overflow (`None`) REFUSES with
/// [`SearchError::SearchSpaceTooLarge`] BEFORE any `unrank` runs — so the
/// generators' internal `.expect()`s (which divide that same already-validated
/// cardinality) can never fire on an unguarded overflow.
pub fn search_enumerated<E: CandidateEvaluator>(
    enumeration: &Enumeration,
    evaluator: &E,
    mode: SearchMode,
    early_exit: bool,
) -> Result<SearchOutcome, SearchError> {
    let n = enumeration.n();
    if n == 0 {
        return Err(SearchError::EmptySearchSpace);
    }
    // The global match count at which every thread stops scanning: 1 for
    // first-match early-exit (collision-free address-search), 2 for the
    // full-scan-with-2nd-match ambiguity certification (the v0.60.0 default).
    let stop_at: usize = if early_exit { 1 } else { 2 };
    // M1 / P1 M-1: a cardinality that overflows `u128` must REFUSE, not panic.
    // `cardinality()` (factorial / s_own / s_opt) returns `None` on overflow; we
    // propagate it as a typed error BEFORE any `unrank` (whose internal
    // `.expect()`s divide this validated cardinality, so they cannot fire). For
    // `FullPermutation` this is `factorial(n)`, identical to the legacy engine.
    let perms = enumeration
        .cardinality()
        .ok_or(SearchError::SearchSpaceTooLarge { n })?;
    let outer = match mode {
        SearchMode::Id => 1u128,
        SearchMode::Address(range) => u128::from(range.outer_count()),
    };
    let total = perms
        .checked_mul(outer)
        .ok_or(SearchError::SearchSpaceTooLarge { n })?;
    if total == 0 {
        // Empty address range (or empty subset) → no candidates → no match.
        return Ok(SearchOutcome::None);
    }

    let nthreads = search_threads().min(usize_from_u128_clamped(total));
    let matches: Mutex<Vec<Match>> = Mutex::new(Vec::new());
    let global_matches = AtomicUsize::new(0);
    let stop = AtomicBool::new(false);

    // Shard the contiguous index space [0, total) into `nthreads` near-equal
    // chunks. Each thread unranks its slice (outer = idx / perms, perm_rank =
    // idx % perms — address index OUTER) and evaluates. `perms` is the
    // PERMUTATION-space cardinality `S` (subset count for the over-supply modes,
    // `n!` for the exact path).
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
                    let assignment = enumeration.unrank(perm_rank);
                    let address_index = match mode {
                        SearchMode::Id => 0,
                        SearchMode::Address(range) => range.flatten(outer_k),
                    };
                    if evaluator_ref.matches(&assignment, address_index) {
                        local.push(Match {
                            perm_rank,
                            address_index,
                        });
                        // Bump the global counter; once it reaches the stop
                        // threshold (2 = ambiguity certification, the default;
                        // 1 = first-match early-exit), signal stop.
                        let prior = global_matches.fetch_add(1, Ordering::Relaxed);
                        if prior + 1 >= stop_at {
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
    if early_exit {
        // First-match early-exit (collision-free address-search per the §4.4
        // contract): the caller guarantees ≤1 distinct match exists, so any
        // match certifies the unique wallet. Because the threads race, several
        // may each record the (same-wallet) hit before the stop flag
        // propagates; we deterministically return the LOWEST-rank match as
        // `Unique` (a pure function of the input → parallel == reference).
        return match found.iter().min_by(|a, b| {
            a.address_index
                .cmp(&b.address_index)
                .then(a.perm_rank.cmp(&b.perm_rank))
        }) {
            None => Ok(SearchOutcome::None),
            Some(m) => Ok(SearchOutcome::Unique {
                assignment: enumeration.unrank(m.perm_rank),
                address_index: m.address_index,
            }),
        };
    }
    match found.len() {
        0 => Ok(SearchOutcome::None),
        1 => {
            let m = found[0];
            Ok(SearchOutcome::Unique {
                assignment: enumeration.unrank(m.perm_rank),
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
        let outcome = search(5, &target_eval(target.clone()), SearchMode::Id, false).unwrap();
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
        let outcome = search(6, &target_eval(target.clone()), SearchMode::Id, false).unwrap();
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
        assert_eq!(
            search(6, &never, SearchMode::Id, false).unwrap(),
            SearchOutcome::None
        );
    }

    #[test]
    fn engine_two_targets_is_ambiguous() {
        let t1 = vec![0, 1, 2, 3, 4];
        let t2 = vec![4, 3, 2, 1, 0];
        let two = move |a: &[usize], _idx: u64| a == t1.as_slice() || a == t2.as_slice();
        assert_eq!(
            search(5, &two, SearchMode::Id, false).unwrap(),
            SearchOutcome::Ambiguous
        );
    }

    #[test]
    fn engine_n1_trivial_unique() {
        // n == 1: a single permutation [0]; matching evaluator → Unique.
        let always = |_a: &[usize], _idx: u64| true;
        assert_eq!(
            search(1, &always, SearchMode::Id, false).unwrap(),
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
            search(0, &always, SearchMode::Id, false),
            Err(SearchError::EmptySearchSpace)
        );
    }

    // -- Determinism: parallel == single-threaded reference. ----------------

    #[test]
    fn parallel_matches_reference_unique() {
        let target = vec![4, 2, 6, 0, 5, 1, 3];
        let eval = target_eval(target.clone());
        let par = search(7, &eval, SearchMode::Id, false).unwrap();
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
            search(7, &never, SearchMode::Id, false).unwrap(),
            search_reference(7, &never, SearchMode::Id).unwrap()
        );

        let t1 = vec![0, 1, 2, 3, 4, 5];
        let t2 = vec![5, 4, 3, 2, 1, 0];
        let two = move |a: &[usize], _idx: u64| a == t1.as_slice() || a == t2.as_slice();
        assert_eq!(
            search(6, &two, SearchMode::Id, false).unwrap(),
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
        let eval =
            move |a: &[usize], idx: u64| a == target_perm.as_slice() && idx == target_addr_idx;
        let range = AddressRange {
            min: 0,
            max: 20,
            chains: ChainScope::Receive,
        };
        let outcome = search(3, &eval, SearchMode::Address(range), false).unwrap();
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
            search(3, &always, SearchMode::Address(range), false).unwrap(),
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
        let d2 = cap_decision(
            7200,
            Duration::from_secs(1),
            Some(Duration::from_secs(10_000)),
        )
        .unwrap();
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
            search(35, &always, SearchMode::Id, false),
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
            search(34, &always, SearchMode::Address(range), false),
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

    // =======================================================================
    // P1 — own-account subset-search engine (SPEC §3/§4, the combinatorics
    // core). The make-or-break gate is the BIJECTION: every generator below is
    // verified by an independent brute-force reference that builds the valid
    // set from first principles, and we assert the generator's enumerated set
    // EQUALS that reference set EXACTLY (each member once, no dup, no miss) and
    // that the closed-form cardinality EQUALS the enumerated count.
    // =======================================================================

    // ---- Brute-force reference builders (first-principles oracles). --------

    /// All injective placements of `n` of `pool` items into `n` ordered slots,
    /// as `Vec<slot→pool-index>`. Independent of `unrank_kperm` — built by a
    /// recursive choose-and-place so it is a true oracle.
    fn bf_injective_placements(pool: usize, n: usize) -> Vec<Vec<usize>> {
        fn go(
            pool: usize,
            n: usize,
            used: &mut Vec<bool>,
            cur: &mut Vec<usize>,
            out: &mut Vec<Vec<usize>>,
        ) {
            if cur.len() == n {
                out.push(cur.clone());
                return;
            }
            for p in 0..pool {
                if !used[p] {
                    used[p] = true;
                    cur.push(p);
                    go(pool, n, used, cur, out);
                    cur.pop();
                    used[p] = false;
                }
            }
        }
        let mut out = Vec::new();
        if n <= pool {
            let mut used = vec![false; pool];
            let mut cur = Vec::new();
            go(pool, n, &mut used, &mut cur, &mut out);
        }
        out
    }

    /// All `r`-subsets of `0..k` (each an ascending Vec). Independent oracle.
    fn bf_combinations(k: usize, r: usize) -> Vec<Vec<usize>> {
        fn go(k: usize, r: usize, start: usize, cur: &mut Vec<usize>, out: &mut Vec<Vec<usize>>) {
            if cur.len() == r {
                out.push(cur.clone());
                return;
            }
            for x in start..k {
                cur.push(x);
                go(k, r, x + 1, cur, out);
                cur.pop();
            }
        }
        let mut out = Vec::new();
        if r <= k {
            let mut cur = Vec::new();
            go(k, r, 0, &mut cur, &mut out);
        }
        out
    }

    /// All permutations of the given `items` slice. Independent oracle (Heap-ish
    /// recursive).
    fn bf_permute(items: &[usize]) -> Vec<Vec<usize>> {
        if items.is_empty() {
            return vec![vec![]];
        }
        let mut out = Vec::new();
        for i in 0..items.len() {
            let mut rest: Vec<usize> = items.to_vec();
            let x = rest.remove(i);
            for mut p in bf_permute(&rest) {
                let mut v = vec![x];
                v.append(&mut p);
                out.push(v);
            }
        }
        out
    }

    /// Own-anchored valid-assignment ORACLE (SPEC §3). Pool-index convention:
    /// own candidates are `0..k_own`, cosigners are `k_own..k_own+m`. A valid
    /// assignment uses EXACTLY `j` own indices + ALL `m` cosigner indices, each
    /// once, ordered into `N = j+m` slots (sorted ⇒ identity order only).
    fn bf_own_anchored(k_own: usize, j: usize, m: usize, sorted: bool) -> Vec<Vec<usize>> {
        let n = j + m;
        let mut out = Vec::new();
        for own_subset in bf_combinations(k_own, j) {
            // The N selected pool indices: j chosen own ++ all m cosigners.
            let mut selected: Vec<usize> = own_subset.clone();
            for c in 0..m {
                selected.push(k_own + c);
            }
            debug_assert_eq!(selected.len(), n);
            if sorted {
                // identity order: the selected set in its canonical (here:
                // own-ascending then cosigners) order, exactly one placement.
                out.push(selected);
            } else {
                for p in bf_permute(&selected) {
                    out.push(p);
                }
            }
        }
        out
    }

    /// Opt-in valid-assignment ORACLE (SPEC §4.3). Pool: own `0..k_own`,
    /// cosigners `k_own..k_own+m_sup`. For every valid `j ∈ [1, min(k_own,n-1)]`
    /// choose `j` own + `(n-j)` cosigners, order into `n` slots (sorted ⇒
    /// identity). Strata are disjoint by own-slot-count `j`.
    fn bf_opt_in(k_own: usize, m_sup: usize, n: usize, sorted: bool) -> Vec<Vec<usize>> {
        let mut out = Vec::new();
        let j_min = 1usize;
        let j_max = k_own.min(n.saturating_sub(1));
        for j in j_min..=j_max {
            let need_cos = n - j;
            if need_cos > m_sup {
                continue;
            }
            for own_subset in bf_combinations(k_own, j) {
                for cos_subset in bf_combinations(m_sup, need_cos) {
                    let mut selected: Vec<usize> = own_subset.clone();
                    for &c in &cos_subset {
                        selected.push(k_own + c);
                    }
                    debug_assert_eq!(selected.len(), n);
                    if sorted {
                        out.push(selected);
                    } else {
                        for p in bf_permute(&selected) {
                            out.push(p);
                        }
                    }
                }
            }
        }
        out
    }

    /// Assert the generated set EQUALS the reference set EXACTLY (each member
    /// once, no dup, no miss) and the closed-form `card` EQUALS both counts.
    fn assert_bijects(generated: &[Vec<usize>], reference: &[Vec<usize>], card: u128, ctx: &str) {
        use std::collections::HashSet;
        let gen_set: HashSet<&Vec<usize>> = generated.iter().collect();
        assert_eq!(
            gen_set.len(),
            generated.len(),
            "{ctx}: generator emitted a DUPLICATE (|set| {} < |list| {})",
            gen_set.len(),
            generated.len()
        );
        let ref_set: HashSet<&Vec<usize>> = reference.iter().collect();
        assert_eq!(
            gen_set, ref_set,
            "{ctx}: generated set != reference set (missing or extra members)"
        );
        assert_eq!(
            generated.len() as u128,
            card,
            "{ctx}: enumerated count {} != closed-form cardinality {card}",
            generated.len()
        );
        assert_eq!(
            reference.len() as u128,
            card,
            "{ctx}: reference count {} != closed-form cardinality {card}",
            reference.len()
        );
    }

    // ---- c_choose / p_count / s_own / s_opt cardinality helpers. -----------

    #[test]
    fn c_choose_matches_known_values() {
        assert_eq!(c_choose(0, 0), Some(1));
        assert_eq!(c_choose(5, 0), Some(1));
        assert_eq!(c_choose(5, 5), Some(1));
        assert_eq!(c_choose(5, 6), Some(0)); // r > k → 0
        assert_eq!(c_choose(5, 2), Some(10));
        assert_eq!(c_choose(32, 4), Some(35_960));
        assert_eq!(c_choose(52, 5), Some(2_598_960));
        // symmetry C(n,r) == C(n,n-r)
        assert_eq!(c_choose(40, 13), c_choose(40, 27));
    }

    #[test]
    fn c_choose_overflows_to_none() {
        // C(256,128) is a 252-bit number → cannot fit u128 → None (REFUSE,
        // not panic). This is the §6-ceiling backstop (the #28 M1 lesson).
        assert_eq!(c_choose(256, 128), None);
    }

    #[test]
    fn p_count_matches_perm_count() {
        // P(pool,n) = pool!/(pool-n)!. Cross-check against factorial ratios.
        assert_eq!(p_count(5, 0), Some(1));
        assert_eq!(p_count(5, 5), Some(120));
        assert_eq!(p_count(7, 3), Some(210)); // 7·6·5
        assert_eq!(p_count(39, 11), {
            // P(39,11) = 39·38·…·29
            let mut p = 1u128;
            for x in 29..=39u128 {
                p *= x;
            }
            Some(p)
        });
        assert_eq!(p_count(3, 5), Some(0)); // n > pool → 0 injective placements
    }

    #[test]
    fn p_count_overflows_to_none() {
        // A huge pool with a large n overflows u128 → None.
        assert_eq!(p_count(usize::MAX, 40), None);
    }

    #[test]
    fn s_own_closed_form() {
        // Non-sorted: C(K_own,j)·N!.  Sorted: C(K_own,j).
        // K_own=32, j=4, M=7 → N=11 → C(32,4)·11! = 35960·39916800.
        let expect = 35_960u128 * 39_916_800u128;
        assert_eq!(s_own(32, 4, 7, false), Some(expect));
        assert_eq!(s_own(32, 4, 7, true), Some(35_960));
        // Collapse: K_own == j (no over-supply) → C(j,j)·N! = N! (byte-identical
        // to the v0.60.0 exact path).
        assert_eq!(s_own(4, 4, 7, false), factorial(11));
        assert_eq!(s_own(4, 4, 7, true), Some(1));
    }

    #[test]
    fn s_own_overflows_to_none() {
        // Force the factorial leg to overflow (N=35 → 35! overflows).
        assert_eq!(s_own(40, 1, 34, false), None); // N = 35
                                                   // Force the c_choose leg to overflow.
        assert_eq!(s_own(256, 128, 0, false), None);
    }

    #[test]
    fn s_opt_closed_form_matches_strata_sum() {
        // S_opt = Σ_j C(K_own,j)·C(M_sup,N-j)·N!  (non-sorted)
        //        = Σ_j C(K_own,j)·C(M_sup,N-j)     (sorted)
        // small: K_own=4, M_sup=4, N=3 → j ∈ {1,2} (j_max = min(4,2)=2).
        let nfact = factorial(3).unwrap();
        let mut expect_ns = 0u128;
        let mut expect_s = 0u128;
        for j in 1..=2usize {
            let term = c_choose(4, j).unwrap() * c_choose(4, 3 - j).unwrap();
            expect_s += term;
            expect_ns += term * nfact;
        }
        assert_eq!(s_opt(4, 4, 3, false), Some(expect_ns));
        assert_eq!(s_opt(4, 4, 3, true), Some(expect_s));
    }

    // ---- unrank_kperm bijection. -------------------------------------------

    #[test]
    fn unrank_kperm_bijects_injective_placements() {
        // For several small (pool, n) the SET of unrank_kperm(r,pool,n) over
        // r ∈ [0, P(pool,n)) EQUALS the brute-force injective-placement set,
        // each EXACTLY once.
        for (pool, n) in [
            (3usize, 2usize),
            (4, 2),
            (4, 3),
            (5, 3),
            (5, 5),
            (1, 1),
            (6, 1),
        ] {
            let card = p_count(pool, n).unwrap();
            let generated: Vec<Vec<usize>> = (0..card).map(|r| unrank_kperm(r, pool, n)).collect();
            let reference = bf_injective_placements(pool, n);
            assert_bijects(
                &generated,
                &reference,
                card,
                &format!("unrank_kperm(pool={pool}, n={n})"),
            );
        }
    }

    #[test]
    fn unrank_kperm_is_lexicographic() {
        // The unrank order is lexicographic over the chosen pool-index tuples.
        // P(4,2) = 12; rank 0 = [0,1], rank 1 = [0,2], …, rank 11 = [3,2].
        assert_eq!(unrank_kperm(0, 4, 2), vec![0, 1]);
        assert_eq!(unrank_kperm(1, 4, 2), vec![0, 2]);
        assert_eq!(unrank_kperm(2, 4, 2), vec![0, 3]);
        assert_eq!(unrank_kperm(3, 4, 2), vec![1, 0]);
        assert_eq!(unrank_kperm(11, 4, 2), vec![3, 2]);
    }

    // ---- own-anchored generator bijection over S_own. ----------------------

    #[test]
    fn own_anchored_bijects_s_own_nonsorted() {
        // Exhaustive small (k_own, j, m). The generated set EQUALS the oracle,
        // count == C(k_own,j)·N!, and CRUCIALLY no cosigner-dropping placement
        // (every assignment uses ALL m cosigner indices).
        for (k_own, j, m) in [
            (3usize, 1usize, 1usize),
            (3, 2, 1),
            (4, 2, 2),
            (4, 1, 2),
            (5, 2, 1),
            (2, 2, 1), // collapse-ish: C(2,2)=1
        ] {
            let n = j + m;
            let card = s_own(k_own, j, m, false).unwrap();
            let generated: Vec<Vec<usize>> = (0..card)
                .map(|r| own_anchored_unrank(r, k_own, j, m, false))
                .collect();
            let reference = bf_own_anchored(k_own, j, m, false);
            let ctx = format!("own_anchored(k_own={k_own}, j={j}, m={m}, sorted=false)");
            assert_bijects(&generated, &reference, card, &ctx);
            // No cosigner-dropping: every assignment contains all m cosigner
            // indices {k_own..k_own+m}.
            for a in &generated {
                for c in 0..m {
                    assert!(
                        a.contains(&(k_own + c)),
                        "{ctx}: assignment {a:?} DROPS cosigner index {}",
                        k_own + c
                    );
                }
                assert_eq!(a.len(), n, "{ctx}: assignment {a:?} has wrong width");
            }
        }
    }

    #[test]
    fn own_anchored_bijects_s_own_sorted() {
        // Sorted shape: drop the perm_rank factor → C(k_own,j) identity-ordered
        // subsets.
        for (k_own, j, m) in [(4usize, 2usize, 2usize), (5, 3, 1), (3, 1, 2), (4, 1, 1)] {
            let card = s_own(k_own, j, m, true).unwrap();
            let generated: Vec<Vec<usize>> = (0..card)
                .map(|r| own_anchored_unrank(r, k_own, j, m, true))
                .collect();
            let reference = bf_own_anchored(k_own, j, m, true);
            let ctx = format!("own_anchored(k_own={k_own}, j={j}, m={m}, sorted=true)");
            assert_bijects(&generated, &reference, card, &ctx);
            // Each sorted placement is in identity (ascending own ++ cosigner)
            // order — i.e. own portion ascending, cosigners in fixed order.
            for a in &generated {
                // own indices come first and are ascending; cosigners follow in
                // ascending order k_own..k_own+m.
                let own_part: Vec<usize> = a.iter().copied().filter(|&x| x < k_own).collect();
                let mut own_sorted = own_part.clone();
                own_sorted.sort_unstable();
                assert_eq!(
                    own_part, own_sorted,
                    "{ctx}: own part not ascending in {a:?}"
                );
            }
        }
    }

    #[test]
    fn own_anchored_collapses_to_nfact_at_k_own_eq_j() {
        // When K_own == j (no over-supply) the own-anchored generator is exactly
        // the v0.60.0 unrank_permutation(N): C(j,j)=1 subset, N! orderings, and
        // the pool indices are 0..N (j own 0..j ++ m cosigners j..j+m == 0..N).
        let (k_own, j, m) = (3usize, 3usize, 2usize);
        let n = j + m;
        let card = s_own(k_own, j, m, false).unwrap();
        assert_eq!(card, factorial(n).unwrap());
        for r in 0..card {
            let got = own_anchored_unrank(r, k_own, j, m, false);
            let want = unrank_permutation(r, n);
            assert_eq!(
                got, want,
                "rank {r}: own-anchored != plain unrank_permutation"
            );
        }
    }

    // ---- opt-in stratified generator bijection over S_opt. -----------------

    #[test]
    fn opt_in_bijects_s_opt_nonsorted() {
        for (k_own, m_sup, n) in [
            (3usize, 3usize, 3usize),
            (4, 4, 3),
            (3, 2, 3),
            (4, 3, 4),
            (2, 3, 3),
        ] {
            let card = s_opt(k_own, m_sup, n, false).unwrap();
            let generated: Vec<Vec<usize>> = (0..card)
                .map(|r| opt_in_unrank(r, k_own, m_sup, n, false))
                .collect();
            let reference = bf_opt_in(k_own, m_sup, n, false);
            let ctx = format!("opt_in(k_own={k_own}, m_sup={m_sup}, n={n}, sorted=false)");
            assert_bijects(&generated, &reference, card, &ctx);
            // Every assignment uses ≥1 own and ≥1 cosigner (j_min=1, ≥1 cosigner).
            for a in &generated {
                assert!(a.iter().any(|&x| x < k_own), "{ctx}: {a:?} has no own");
                assert!(
                    a.iter().any(|&x| x >= k_own),
                    "{ctx}: {a:?} has no cosigner"
                );
                assert_eq!(a.len(), n, "{ctx}: {a:?} wrong width");
            }
        }
    }

    #[test]
    fn opt_in_bijects_s_opt_sorted() {
        for (k_own, m_sup, n) in [(4usize, 4usize, 3usize), (3, 3, 3), (4, 3, 4)] {
            let card = s_opt(k_own, m_sup, n, true).unwrap();
            let generated: Vec<Vec<usize>> = (0..card)
                .map(|r| opt_in_unrank(r, k_own, m_sup, n, true))
                .collect();
            let reference = bf_opt_in(k_own, m_sup, n, true);
            let ctx = format!("opt_in(k_own={k_own}, m_sup={m_sup}, n={n}, sorted=true)");
            assert_bijects(&generated, &reference, card, &ctx);
        }
    }

    // ---- total_candidates_subset. ------------------------------------------

    #[test]
    fn total_candidates_subset_passes_s_through() {
        // Id-mode: drives exactly S candidates.
        assert_eq!(total_candidates_subset(SearchMode::Id, 1000), Some(1000));
        // Address-mode: S × outer_count.
        let range = AddressRange {
            min: 0,
            max: 20,
            chains: ChainScope::Both,
        };
        assert_eq!(
            total_candidates_subset(SearchMode::Address(range), 1000),
            Some(1000 * 40)
        );
        // Overflow → None.
        assert_eq!(
            total_candidates_subset(SearchMode::Address(range), u128::MAX),
            None
        );
    }

    // ---- early_exit knob: byte-invariance + first-match semantics. ----------

    #[test]
    fn early_exit_false_reproduces_v060_full_scan_outcomes() {
        // early_exit=false MUST reproduce today's full-scan-with-2nd-match
        // behavior IDENTICALLY (the v0.60.0 anchor): Unique / None / Ambiguous.
        let target = vec![3, 1, 4, 0, 2];
        assert_eq!(
            search(5, &target_eval(target.clone()), SearchMode::Id, false).unwrap(),
            SearchOutcome::Unique {
                assignment: target,
                address_index: 0
            }
        );
        let never = |_a: &[usize], _idx: u64| false;
        assert_eq!(
            search(6, &never, SearchMode::Id, false).unwrap(),
            SearchOutcome::None
        );
        let t1 = vec![0, 1, 2, 3, 4];
        let t2 = vec![4, 3, 2, 1, 0];
        let two = move |a: &[usize], _idx: u64| a == t1.as_slice() || a == t2.as_slice();
        assert_eq!(
            search(5, &two, SearchMode::Id, false).unwrap(),
            SearchOutcome::Ambiguous
        );
        // And it must AGREE with the (unchanged) reference oracle.
        assert_eq!(
            search(5, &two, SearchMode::Id, false).unwrap(),
            search_reference(5, &two, SearchMode::Id).unwrap()
        );

        // STRONGER ambiguity anchor (defeats the parallel race + perturbation):
        // a MANY-match evaluator over a large space (every perm whose slot 0 maps
        // to candidate 0 — there are 7!/7 = 720 such matches across 7! = 5040).
        // With early_exit=false the engine MUST certify Ambiguous (≥2). A broken
        // engine that stopped at the FIRST match (stop_at=1) would return Unique
        // for at least one run; we assert Ambiguous across many runs so the race
        // cannot mask the break, and that it always matches the full-scan oracle.
        let many = |a: &[usize], _idx: u64| a.first() == Some(&0);
        for _ in 0..64 {
            assert_eq!(
                search(7, &many, SearchMode::Id, false).unwrap(),
                SearchOutcome::Ambiguous,
                "early_exit=false must full-scan-certify Ambiguous over a many-match space"
            );
            assert_eq!(
                search(7, &many, SearchMode::Id, false).unwrap(),
                search_reference(7, &many, SearchMode::Id).unwrap()
            );
        }
    }

    #[test]
    fn early_exit_true_two_matches_may_report_unique_or_first() {
        // With early_exit=true the engine MAY stop at the first match. Over a
        // collision-free address-search the contract guarantees a single match,
        // but if a synthetic evaluator DOES have 2 matches, early-exit is
        // permitted to return one of them as Unique (it stops at the first).
        // The funds-safety contract only USES early_exit where collisions are
        // impossible — here we just assert it does not PANIC and returns a
        // Unique carrying a real match (never Ambiguous, since it short-circuits).
        let range = AddressRange {
            min: 0,
            max: 8,
            chains: ChainScope::Receive,
        };
        let always = |_a: &[usize], _idx: u64| true;
        let outcome = search(3, &always, SearchMode::Address(range), true).unwrap();
        match outcome {
            SearchOutcome::Unique { .. } => {}
            other => panic!(
                "early_exit=true over a matching space: expected Unique-on-first, got {other:?}"
            ),
        }
    }

    #[test]
    fn early_exit_false_unique_still_full_scans_for_a_single_match() {
        // A SINGLE match with early_exit=false is Unique (the full scan finds
        // no 2nd match). Address mode, low-index target.
        let range = AddressRange {
            min: 0,
            max: 20,
            chains: ChainScope::Receive,
        };
        let target_perm = vec![2, 0, 1];
        let target_addr_idx = range.flatten(3);
        let eval =
            move |a: &[usize], idx: u64| a == target_perm.as_slice() && idx == target_addr_idx;
        let outcome = search(3, &eval, SearchMode::Address(range), false).unwrap();
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

    // ---- search_enumerated: drive the over-supply spaces (P2-A). -------------

    #[test]
    fn search_full_permutation_equals_legacy_search() {
        // `search` (the n! wrapper) MUST be byte-identical to driving
        // `search_enumerated` with `Enumeration::FullPermutation { n }` — the
        // wrapper is just the latter with the full-perm enumeration. Verified
        // across Unique / None / Ambiguous.
        let target = vec![3, 1, 4, 0, 2];
        let enum_full = Enumeration::FullPermutation { n: 5 };
        assert_eq!(
            search_enumerated(
                &enum_full,
                &target_eval(target.clone()),
                SearchMode::Id,
                false
            )
            .unwrap(),
            search(5, &target_eval(target.clone()), SearchMode::Id, false).unwrap(),
        );
        let never = |_a: &[usize], _idx: u64| false;
        assert_eq!(
            search_enumerated(&enum_full, &never, SearchMode::Id, false).unwrap(),
            SearchOutcome::None
        );
        let t1 = vec![0, 1, 2, 3, 4];
        let t2 = vec![4, 3, 2, 1, 0];
        let two = move |a: &[usize], _idx: u64| a == t1.as_slice() || a == t2.as_slice();
        assert_eq!(
            search_enumerated(&enum_full, &two, SearchMode::Id, false).unwrap(),
            SearchOutcome::Ambiguous
        );
    }

    #[test]
    fn search_own_anchored_resolves_a_nonzero_subset_assignment() {
        // The OVER-SUPPLY own-anchored space: k_own=4 own candidates (indices
        // 0..4), m=1 cosigner (index 4), j=1 own slot → N=2 slots. Only ONE own
        // candidate (say index 2) is the real one. The evaluator matches the
        // unique assignment that places own-index-2 + cosigner-index-4. The
        // search MUST drive the s_own space (NOT n!) and resolve it.
        let k_own = 4usize;
        let j = 1usize;
        let m = 1usize;
        let enumeration = Enumeration::OwnAnchored {
            k_own,
            j,
            m,
            sorted: false,
        };
        // The target assignment: slot 0 → pool index 2 (the real own key), slot
        // 1 → pool index 4 (the cosigner). (This is a valid own-anchored
        // assignment: exactly j=1 own index <4 + all m=1 cosigner.)
        let target = vec![2usize, 4usize];
        let eval = move |a: &[usize], _idx: u64| a == target.as_slice();
        let outcome = search_enumerated(&enumeration, &eval, SearchMode::Id, false).unwrap();
        match outcome {
            SearchOutcome::Unique { assignment, .. } => {
                assert_eq!(assignment, vec![2, 4]);
            }
            other => panic!("expected Unique over the own-anchored space, got {other:?}"),
        }
        // The reference exactly enumerates s_own candidates, so a never-match is
        // None over the SAME (subset) cardinality, not n!.
        let never = |_a: &[usize], _idx: u64| false;
        assert_eq!(
            search_enumerated(&enumeration, &never, SearchMode::Id, false).unwrap(),
            SearchOutcome::None
        );
    }

    #[test]
    fn search_own_anchored_only_enumerates_s_own_candidates() {
        // The engine must enumerate EXACTLY the s_own set — never an n!-over-the-
        // whole-pool placement that drops a cosigner. We assert this by counting
        // the DISTINCT assignments the engine visits: a synthetic evaluator
        // records every assignment it is handed; the recorded SET must equal the
        // independently-generated own_anchored_unrank set (size s_own).
        let k_own = 4usize;
        let j = 2usize;
        let m = 1usize;
        let sorted = false;
        let enumeration = Enumeration::OwnAnchored {
            k_own,
            j,
            m,
            sorted,
        };
        let s = s_own(k_own, j, m, sorted).unwrap();
        let seen = std::sync::Mutex::new(std::collections::HashSet::new());
        let recorder = |a: &[usize], _idx: u64| {
            seen.lock().unwrap().insert(a.to_vec());
            false // never match → full scan
        };
        let outcome = search_enumerated(&enumeration, &recorder, SearchMode::Id, false).unwrap();
        assert_eq!(outcome, SearchOutcome::None);
        let visited = seen.into_inner().unwrap();
        // Independent expected set.
        let expected: std::collections::HashSet<Vec<usize>> = (0..s)
            .map(|r| own_anchored_unrank(r, k_own, j, m, sorted))
            .collect();
        assert_eq!(
            visited, expected,
            "the engine must visit EXACTLY the s_own assignment set (no cosigner-dropping placements)"
        );
        assert_eq!(visited.len() as u128, s);
    }

    #[test]
    fn search_subset_overflow_refuses_not_panics() {
        // A subset cardinality that overflows u128 → REFUSE (typed error), never
        // panic. `OwnAnchored { k_own: 256, j: 128, .. }` → C(256,128) is 252-bit.
        let enumeration = Enumeration::OwnAnchored {
            k_own: 256,
            j: 128,
            m: 0,
            sorted: false,
        };
        let never = |_a: &[usize], _idx: u64| false;
        assert!(matches!(
            search_enumerated(&enumeration, &never, SearchMode::Id, false),
            Err(SearchError::SearchSpaceTooLarge { .. })
        ));
    }

    #[test]
    fn search_own_anchored_address_early_exit_resolves_unique() {
        // Address-mode over the own-anchored space with early_exit=true: a unique
        // (assignment, address_index) is found and returned as Unique-on-first.
        let k_own = 3usize;
        let j = 1usize;
        let m = 1usize;
        let enumeration = Enumeration::OwnAnchored {
            k_own,
            j,
            m,
            sorted: false,
        };
        let range = AddressRange {
            min: 0,
            max: 10,
            chains: ChainScope::Receive,
        };
        let target = vec![1usize, 3usize]; // own index 1 + cosigner index 3
        let target_addr = range.flatten(2);
        let eval = move |a: &[usize], idx: u64| a == target.as_slice() && idx == target_addr;
        let outcome =
            search_enumerated(&enumeration, &eval, SearchMode::Address(range), true).unwrap();
        match outcome {
            SearchOutcome::Unique {
                assignment,
                address_index,
            } => {
                assert_eq!(assignment, vec![1, 3]);
                assert_eq!(address_index, target_addr);
            }
            other => panic!("expected Unique over own-anchored address space, got {other:?}"),
        }
    }
}
