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

/// Restore the known `{hrp}1` prefix within exactly `j` edits and validate
/// the reassembled candidate. For split point `p` in the clamped range
/// `[(3-j).., (3+j)]`, if `levenshtein(&input[..p], "{hrp}1") == j` (exactly
/// j — the outer `1..=max_indel` loop assigns the precise `indel_count`),
/// reconstruct `cand = "{hrp}1" + input[p..]` and validate via the oracle.
/// `direction` = Inserted if `p < 3` (chars were dropped from prefix) else
/// Deleted (an extra char was present in the prefix).
fn collect_prefix(
    input: &str,
    hrp: &str,
    j: usize,
    oracle: &dyn IndelOracle,
    out: &mut Vec<IndelCandidate>,
) {
    let k = format!("{hrp}1"); // known 3-char prefix, e.g. "ms1"
    let chars: Vec<char> = input.chars().collect();
    let lo = 3usize.saturating_sub(j);
    let hi = (3 + j).min(chars.len());
    for p in lo..=hi {
        let head: String = chars[..p].iter().collect();
        if levenshtein(&head, &k) != j {
            continue; // exactly j edits in the prefix region
        }
        let tail: String = chars[p..].iter().collect();
        let cand = format!("{k}{tail}");
        if let Some(rec) = oracle.validate(&cand, &BTreeSet::new()) {
            let direction = if p < 3 {
                IndelDirection::Inserted
            } else {
                IndelDirection::Deleted
            };
            out.push(IndelCandidate {
                recovered: rec,
                indel_count: j,
                region: IndelRegion::Prefix,
                direction,
            });
        }
    }
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

fn collect_data_insert(
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
    let slots = data.len() + j; // post-insertion length
    for combo in combinations(slots, j) {
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
        let cand = format!("{hrp}1{}", built.iter().collect::<String>());
        if let Some(rec) = oracle.validate(&cand, &allowed) {
            out.push(IndelCandidate {
                recovered: rec,
                indel_count: j,
                region: IndelRegion::DataPart,
                direction: IndelDirection::Inserted,
            });
        }
    }
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
            },
            IndelCandidate {
                recovered: "ms1xyz".into(),
                indel_count: 1,
                region: IndelRegion::DataPart,
                direction: IndelDirection::Deleted,
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
            fn validate(&self, candidate: &str, _allowed: &BTreeSet<usize>) -> Option<String> {
                Some(candidate.to_string())
            }
        }
        let outcome = recover_indel("ms1qpzr", "ms", 1, &AcceptAll);
        assert!(
            matches!(outcome, IndelOutcome::Ambiguous(ref v) if v.len() >= 2),
            "got {outcome:?}"
        );
    }
}
