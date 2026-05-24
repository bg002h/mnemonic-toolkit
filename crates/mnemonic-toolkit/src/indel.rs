//! Incorrect-length (indel) recovery for m-format strings — enumerate-and-
//! validate around the existing BCH decode. SPEC:
//! design/BRAINSTORM_m_format_incorrect_length_recovery.md.
//!
//! Two candidate producers feed one per-kind validator (`IndelOracle`):
//!   P1 prefix-region restore to the known `ms1`/`mk1` prefix;
//!   P2 data-region — delete-and-validate (too long) / placeholder-then-decode
//!      (too short, BCH solves the missing symbol).
//! Pure-indel only: a candidate's BCH corrections must be ⊆ the placeholder
//! positions we inserted (∅ for delete/prefix).

use std::collections::BTreeSet;

// used from Phase 1+
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IndelRegion {
    Prefix,
    DataPart,
}

/// The repair OPERATION applied to the corrupted input to recover the original.
// used from Phase 1+
#[allow(dead_code)]
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
}

// used from Phase 1+
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndelOutcome {
    Unique(IndelCandidate),
    /// ≥2 candidates with DISTINCT `recovered`.
    Ambiguous(Vec<IndelCandidate>),
    Unrecoverable,
}

/// Per-kind single-string validator. `allowed` are the data-part indices of
/// placeholders we inserted (∅ for delete/prefix producers). Returns the
/// canonical recovered full-string iff the candidate decodes cleanly AND its
/// BCH corrections ⊆ `allowed`.
// used from Phase 1+
#[allow(dead_code)]
pub trait IndelOracle {
    fn validate(&self, candidate: &str, allowed: &BTreeSet<usize>) -> Option<String>;
}

/// ALPHABET[0]; any fixed symbol works — the BCH decoder solves the true
/// value, and the subset-check tolerates a placeholder==true-symbol collision.
// used from Phase 1+
#[allow(dead_code)]
pub(crate) const PLACEHOLDER_CHAR: char = 'q';

/// Engine entry point. `input` is one full m*1 string (one ms1, or ONE mk1
/// chunk). `hrp` ∈ {"ms","mk"}. Produces the dedup'd outcome.
// used from Phase 5+ (CLI wiring)
#[allow(dead_code)]
pub fn recover_indel(
    input: &str,
    hrp: &str,
    max_indel: usize,
    oracle: &dyn IndelOracle,
) -> IndelOutcome {
    let mut hits: Vec<IndelCandidate> = Vec::new();
    for j in 1..=max_indel {
        collect_prefix(input, hrp, j, oracle, &mut hits); // P1
        collect_data_delete(input, hrp, j, oracle, &mut hits); // P2 too-long
        collect_data_insert(input, hrp, j, oracle, &mut hits); // P2 too-short
    }
    dedup_by_recovered(&mut hits);
    match hits.len() {
        0 => IndelOutcome::Unrecoverable,
        1 => IndelOutcome::Unique(hits.into_iter().next().unwrap()),
        _ => IndelOutcome::Ambiguous(hits),
    }
}

// Phase 3+ fills the body; keep Vec (not slice) so push works without signature churn.
#[allow(dead_code, clippy::ptr_arg)]
fn collect_prefix(
    _input: &str,
    _hrp: &str,
    _j: usize,
    _oracle: &dyn IndelOracle,
    _out: &mut Vec<IndelCandidate>,
) {
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

fn collect_data_delete(
    input: &str,
    hrp: &str,
    j: usize,
    oracle: &dyn IndelOracle,
    out: &mut Vec<IndelCandidate>,
) {
    let Some(dstart) = data_part_bounds(input, hrp) else {
        return;
    };
    let data: Vec<char> = input[dstart..].chars().collect();
    if data.len() <= j {
        return;
    }
    let allowed = BTreeSet::new();
    for combo in combinations(data.len(), j) {
        let kept: String = data
            .iter()
            .enumerate()
            .filter(|(i, _)| !combo.contains(i))
            .map(|(_, c)| *c)
            .collect();
        let cand = format!("{hrp}1{kept}");
        if let Some(rec) = oracle.validate(&cand, &allowed) {
            out.push(IndelCandidate {
                recovered: rec,
                indel_count: j,
                region: IndelRegion::DataPart,
                direction: IndelDirection::Deleted,
            });
        }
    }
}

// Phase 2+ fills the body; keep Vec (not slice) so push works without signature churn.
#[allow(dead_code, clippy::ptr_arg)]
fn collect_data_insert(
    _input: &str,
    _hrp: &str,
    _j: usize,
    _oracle: &dyn IndelOracle,
    _out: &mut Vec<IndelCandidate>,
) {
}

// used from recover_indel (Phase 1+); dead_code because recover_indel itself is
// dead until Phase 5.
#[allow(dead_code)]
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
// used from Phase 3+
#[allow(dead_code, clippy::needless_range_loop)]
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
        fn validate(&self, _: &str, _: &BTreeSet<usize>) -> Option<String> {
            None
        }
    }

    #[test]
    fn recover_indel_empty_budget_is_unrecoverable() {
        assert_eq!(
            recover_indel("ms1qqqq", "ms", 0, &NoOracle),
            IndelOutcome::Unrecoverable
        );
    }
}
