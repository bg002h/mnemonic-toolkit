//! Incorrect-length (indel) recovery for m-format strings — enumerate-and-
//! validate around the existing BCH decode. SPEC:
//! design/BRAINSTORM_m_format_incorrect_length_recovery.md.
//!
//! A two-level (prefix × data) search feeds one per-kind validator
//! (`IndelOracle`), allocating the indel budget across the two regions:
//!   - prefix-region restore to the known `ms1`/`mk1`/`md1` prefix
//!     (`prefix_restorations`);
//!   - data-region — delete-and-validate (too long) / placeholder-then-decode
//!     (too short, BCH solves the missing symbol) (`data_variants`).
//!
//! An edit split that touches both regions is tagged `CrossRegion`.
//! Pure-indel only (when `e_subst == 0`): a candidate's BCH corrections must
//! be ⊆ the placeholder positions we inserted (∅ for delete/prefix); with
//! `e_subst ≥ 1` up to `e_subst` out-of-placeholder corrections are tolerated.

use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndelRegion {
    Prefix,
    DataPart,
    /// Edits spanned BOTH the prefix and the data region (j_prefix ≥ 1 AND
    /// j_data ≥ 1). Only producible by the two-level cross-region search.
    CrossRegion,
}

/// The repair OPERATION applied to the corrupted input to recover the original.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndelDirection {
    /// Restored dropped char(s) — input was too short.
    Inserted,
    /// Removed added char(s) — input was too long.
    Deleted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndelCandidate {
    /// Full m*1 string (canonical, post-solve).
    pub recovered: String,
    /// j — number of indels applied.
    pub indel_count: usize,
    pub region: IndelRegion,
    pub direction: IndelDirection,
    /// Number of substitutions used beyond the inserted placeholders, i.e.
    /// `|corrections \ placeholders|`. `0` for a pure-indel recovery.
    pub subst_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndelOutcome {
    Unique(IndelCandidate),
    /// ≥2 candidates with DISTINCT `recovered`.
    Ambiguous(Vec<IndelCandidate>),
    Unrecoverable,
}

/// Per-kind single-string validator. `allowed` are the data-part indices of
/// placeholders we inserted (∅ for delete/prefix producers). `e_subst` is the
/// substitution-tolerance budget. Returns `Some((recovered, subst_count))` iff
/// the candidate decodes cleanly AND `|corrections \ allowed| ≤ e_subst`, where
/// `subst_count = |corrections \ allowed|` (`0` for a pure-indel recovery).
pub trait IndelOracle {
    fn validate(
        &self,
        candidate: &str,
        allowed: &BTreeSet<usize>,
        e_subst: usize,
    ) -> Option<(String, usize)>;
}

/// ALPHABET[0]; any fixed symbol works — the BCH decoder solves the true
/// value, and the subset-check tolerates a placeholder==true-symbol collision.
pub(crate) const PLACEHOLDER_CHAR: char = 'q';

/// Engine entry point. `input` is one full m*1 string (one ms1, or ONE mk1
/// or md1 chunk). `hrp` ∈ {"ms","mk","md"}. Produces the dedup'd outcome.
pub fn recover_indel(
    input: &str,
    hrp: &str,
    max_indel: usize,
    e_subst: usize,
    oracle: &dyn IndelOracle,
) -> IndelOutcome {
    let mut hits: Vec<IndelCandidate> = Vec::new();
    let k = format!("{hrp}1");
    for (data, j_prefix, pfx_dir) in prefix_restorations(input, hrp, &k, max_indel) {
        let data_budget = max_indel - j_prefix;
        for j_data in 0..=data_budget {
            if j_prefix == 0 && j_data == 0 {
                continue; // un-edited input is not a recovery
            }
            for (cand, allowed, data_dir) in data_variants(&k, &data, j_data) {
                if let Some((rec, sc)) = oracle.validate(&cand, &allowed, e_subst) {
                    let region = match (j_prefix > 0, j_data > 0) {
                        (true, true) => IndelRegion::CrossRegion,
                        (true, false) => IndelRegion::Prefix,
                        (false, true) => IndelRegion::DataPart,
                        (false, false) => unreachable!(),
                    };
                    // `direction` is data-region when any data edit was made,
                    // else the prefix direction. Metadata-only: dedup keys on
                    // `recovered`, not on region/direction.
                    let direction = if j_data > 0 { data_dir } else { pfx_dir };
                    hits.push(IndelCandidate {
                        recovered: rec,
                        indel_count: j_prefix + j_data,
                        region,
                        direction,
                        subst_count: sc,
                    });
                }
            }
        }
    }
    dedup_by_recovered(&mut hits);
    match hits.len() {
        0 => IndelOutcome::Unrecoverable,
        1 => IndelOutcome::Unique(hits.into_iter().next().unwrap()),
        _ => IndelOutcome::Ambiguous(hits),
    }
}

/// Restore the known `k = "{hrp}1"` prefix within exactly `j_prefix` edits,
/// yielding `(data_part_string, j_prefix, prefix_direction)` per restoration.
///
/// - `j_prefix == 0` yields the input's own data-part iff `input` already
///   starts with `k` (prefix intact). The yielded direction is a placeholder
///   (`Deleted`) — it is unused whenever a data edit is also applied, and the
///   `(false, false)` case is skipped by the caller.
/// - For each `j_prefix ∈ 1..=max_indel`, enumerate split points `p` in the
///   clamped range `[(3 - j_prefix).., (3 + j_prefix)]`, keep those with
///   `levenshtein(&chars[..p], k) == j_prefix` (exactly `j_prefix` edits), and
///   yield `(chars[p..], j_prefix, Inserted if p < 3 else Deleted)`.
///
/// This subsumes the old `collect_prefix` window/levenshtein logic verbatim;
/// the only change is yielding the data-part + `j_prefix` rather than
/// validating in place (validation moved to the two-level caller).
fn prefix_restorations(
    input: &str,
    hrp: &str,
    k: &str,
    max_indel: usize,
) -> Vec<(String, usize, IndelDirection)> {
    let mut out: Vec<(String, usize, IndelDirection)> = Vec::new();
    // j_prefix == 0: prefix intact ⇒ the input's own data-part.
    if let Some(dstart) = data_part_bounds(input, hrp) {
        out.push((input[dstart..].to_string(), 0, IndelDirection::Deleted));
    }
    let chars: Vec<char> = input.chars().collect();
    for j in 1..=max_indel {
        let lo = 3usize.saturating_sub(j);
        let hi = (3 + j).min(chars.len());
        for p in lo..=hi {
            let head: String = chars[..p].iter().collect();
            if levenshtein(&head, k) != j {
                continue; // exactly j edits in the prefix region
            }
            let tail: String = chars[p..].iter().collect();
            let direction = if p < 3 {
                IndelDirection::Inserted
            } else {
                IndelDirection::Deleted
            };
            out.push((tail, j, direction));
        }
    }
    out
}

/// All k-element subsets of indices [0, n), each a sorted Vec<usize> (lexicographic).
pub(crate) fn combinations(n: usize, k: usize) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    if k == 0 {
        out.push(Vec::new());
        return out;
    }
    if k > n {
        return out;
    }
    let mut idx: Vec<usize> = (0..k).collect();
    loop {
        out.push(idx.clone());
        let mut i = k;
        loop {
            if i == 0 {
                return out;
            }
            i -= 1;
            if idx[i] != i + n - k {
                break;
            }
        }
        idx[i] += 1;
        for j in (i + 1)..k {
            idx[j] = idx[j - 1] + 1;
        }
    }
}

/// Data-region candidates for exactly `j_data` edits, yielding
/// `(full_candidate, allowed_placeholders, direction)`. `data` is the
/// data-part chars (post-prefix-restoration). `k = "{hrp}1"`.
///
/// - `j_data == 0`: a single candidate `k + data` (data intact, `allowed = ∅`,
///   placeholder `Deleted` direction — unused, the caller picks the prefix
///   direction in that case).
/// - `j_data ≥ 1`: the delete variants (subsumes `collect_data_delete`:
///   `combinations(data.len(), j_data)`, skipped when `data.len() <= j_data`;
///   each yields `(k + kept, ∅, Deleted)`) AND the insert variants (subsumes
///   `collect_data_insert`: `slots = data.len() + j_data` placeholder
///   positions, each yields `(k + built, combo-as-set, Inserted)`).
fn data_variants(
    k: &str,
    data: &str,
    j_data: usize,
) -> Vec<(String, BTreeSet<usize>, IndelDirection)> {
    let mut out: Vec<(String, BTreeSet<usize>, IndelDirection)> = Vec::new();
    let data: Vec<char> = data.chars().collect();
    if j_data == 0 {
        let cand = format!("{k}{}", data.iter().collect::<String>());
        out.push((cand, BTreeSet::new(), IndelDirection::Deleted));
        return out;
    }
    // Delete variants (too-long): remove `j_data` data chars.
    if data.len() > j_data {
        for combo in combinations(data.len(), j_data) {
            let kept: String = data
                .iter()
                .enumerate()
                .filter(|(i, _)| !combo.contains(i))
                .map(|(_, c)| *c)
                .collect();
            let cand = format!("{k}{kept}");
            out.push((cand, BTreeSet::new(), IndelDirection::Deleted));
        }
    }
    // Insert variants (too-short): place `j_data` placeholders into the
    // `data.len() + j_data` post-insertion slots; the BCH decoder solves them.
    let slots = data.len() + j_data; // post-insertion length
    for combo in combinations(slots, j_data) {
        // `combo` = the post-insertion indices that are placeholders.
        let mut built: Vec<char> = Vec::with_capacity(slots);
        let mut src = data.iter();
        for i in 0..slots {
            if combo.contains(&i) {
                built.push(PLACEHOLDER_CHAR);
            } else if let Some(c) = src.next() {
                built.push(*c);
            }
        }
        if built.len() != slots {
            continue; // ran out of source chars (shouldn't happen for valid combos)
        }
        let allowed: BTreeSet<usize> = combo.iter().copied().collect();
        let cand = format!("{k}{}", built.iter().collect::<String>());
        out.push((cand, allowed, IndelDirection::Inserted));
    }
    out
}

fn dedup_by_recovered(hits: &mut Vec<IndelCandidate>) {
    hits.sort_by(|a, b| a.recovered.cmp(&b.recovered));
    hits.dedup_by(|a, b| a.recovered == b.recovered);
}

/// Returns `Some(hrp.len()+1)` iff `input` starts with `"{hrp}1"`, else `None`.
fn data_part_bounds(input: &str, hrp: &str) -> Option<usize> {
    if input.starts_with(&format!("{hrp}1")) {
        Some(hrp.len() + 1)
    } else {
        None
    }
}

/// Standard DP edit distance (small inputs — prefix region ≤ ~7 chars).
#[allow(clippy::needless_range_loop)]
fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let m = a.len();
    let n = b.len();
    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 0..=m {
        dp[i][0] = i;
    }
    for j in 0..=n {
        dp[0][j] = j;
    }
    for i in 1..=m {
        for j in 1..=n {
            dp[i][j] = if a[i - 1] == b[j - 1] {
                dp[i - 1][j - 1]
            } else {
                1 + dp[i - 1][j - 1].min(dp[i - 1][j]).min(dp[i][j - 1])
            };
        }
    }
    dp[m][n]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    struct NoOracle;
    impl IndelOracle for NoOracle {
        fn validate(&self, _: &str, _: &BTreeSet<usize>, _e_subst: usize) -> Option<(String, usize)> {
            None
        }
    }

    #[test]
    fn recover_indel_empty_budget_is_unrecoverable() {
        assert_eq!(
            recover_indel("ms1qqqq", "ms", 0, 0, &NoOracle),
            IndelOutcome::Unrecoverable
        );
    }

    /// R0 I2 load-bearing dedup test: P1 and P2 can recover the SAME string
    /// with different region/direction. dedup_by_recovered must collapse them
    /// (keyed on `recovered` only), NOT the derived PartialEq (which would
    /// leave both → false Ambiguous).
    #[test]
    fn dedup_collapses_same_recovered_with_differing_metadata() {
        let mut hits = vec![
            IndelCandidate {
                recovered: "ms1xyz".into(),
                indel_count: 1,
                region: IndelRegion::Prefix,
                direction: IndelDirection::Inserted,
                subst_count: 0,
            },
            IndelCandidate {
                recovered: "ms1xyz".into(),
                indel_count: 1,
                region: IndelRegion::DataPart,
                direction: IndelDirection::Deleted,
                subst_count: 0,
            },
        ];
        dedup_by_recovered(&mut hits);
        assert_eq!(hits.len(), 1);
    }

    /// Ambiguity contract: a mock oracle that accepts every candidate (recovered
    /// = the candidate string itself) produces ≥2 DISTINCT recovered strings
    /// from a typical input → Ambiguous.
    #[test]
    fn recover_indel_reports_ambiguous_on_multiple_distinct_recovered() {
        struct AcceptAll;
        impl IndelOracle for AcceptAll {
            fn validate(
                &self,
                candidate: &str,
                _allowed: &BTreeSet<usize>,
                _e_subst: usize,
            ) -> Option<(String, usize)> {
                Some((candidate.to_string(), 0))
            }
        }
        let outcome = recover_indel("ms1qpzr", "ms", 1, 0, &AcceptAll);
        assert!(
            matches!(outcome, IndelOutcome::Ambiguous(ref v) if v.len() >= 2),
            "got {outcome:?}"
        );
    }
}
