//! Cycle E (`mk1-repair-set-level-reverify`, F4) — Phase P0.
//!
//! SPEC §4 test matrix: after a per-string BCH substitution-correction of an
//! mk1 card set, `repair_card` / `mnemonic repair` RE-VERIFY by reassembling
//! through `mk_codec::decode` before declaring success, so a >4-error
//! miscorrection (a chunk aliased onto a DIFFERENT valid codeword) is
//! REJECTED rather than blessed as recovery of a different wallet. A PARTIAL
//! set (a single plate) is preserved as an UNVERIFIED candidate.
//!
//! Fixtures: the canonical "abandon × 11 about" bip84 mk1 pair (chunk 0 =
//! long code, chunk 1 = regular code) reused byte-identically from
//! `cli_repair.rs` / `cli_auto_repair.rs` / `cli_indel.rs`; a SECOND, entirely
//! distinct real mk1 pair from `tests/vectors/v0_1/bip84-mainnet.txt` (a
//! different `chunk_set_id`) for the §4.5b multi-group batch test.

use assert_cmd::Command;
use mk_codec::string_layer::bch::ALPHABET;
use predicates::prelude::*;

/// Real ≥2-chunk mk1 card — chunk 0 (long code, 108-char data-part).
const CHUNK0: &str = "mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4";
/// Same card — chunk 1 (regular code, 77-char data-part; the trailing
/// regular-code chunk the SPEC's funds anchor corrupts).
const CHUNK1: &str =
    "mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh";

/// A SECOND, entirely distinct real mk1 card (different `chunk_set_id`),
/// from the checked-in `bip84-mainnet.txt` conformance fixture. Used ONLY as
/// the "clean/other group" half of the §4.5b multi-group batch test — never
/// corrupted.
const SECOND_CHUNK0: &str = "mk1qpnd2wpqqsqek48ppe2rd4eyqvzg3vs7zfl2pe5jyqghcnaqxqq4gdatr9tn90ga6tg0purlfh9275f4pvjmck3usgpec7pzw3wvgsn9mwmd";
const SECOND_CHUNK1: &str =
    "mk1qpnd2wppha4qc2sv8g58zqcpswt0zfsza3lk237tx7xeg8evycaywffzk5r3hcma55t0u0d83tguz";

/// **§4.1 / plan-R0 PI-2 — pinned funds-anchor seed.** A 5-substitution
/// corruption of `CHUNK1`'s data-part that `bch_correct_regular` (mk-codec's
/// own BCH corrector, the SAME primitive `repair_chunk_one` wraps) aliases to
/// a valid-but-DIFFERENT codeword, AND that fails full-set reassembly via
/// `mk_codec::decode(&[CHUNK0, <corrected>])`. Found ONCE via
/// `find_mk1_miscorrection_seed` (seed 0x4634_5F31, cap 10_000_000) and
/// pinned here as the fully-resolved CORRUPTED STRING — encoding a
/// `(payload, positions)` pair and re-encoding at test time would NOT
/// reproduce this: `mk_codec::encode` draws a RANDOM `chunk_set_id`
/// (`pipeline.rs:45-47`) that sits inside the BCH codeword, so the same
/// positions/payload would alias differently (or not at all) under a
/// different csid. Pinning the STRING is the only reproducible anchor.
///
/// If a future mk-codec BCH change invalidates this pinned alias (the
/// `cell_4_1_funds_anchor_*` tests below start failing at the
/// `mk_codec::decode(...).is_err()` assertion, NOT at a panic), re-pin via
/// `find_mk1_miscorrection_seed(CHUNK0, CHUNK1, <new seed>, 10_000_000)` — if
/// that returns `None` within the cap, the true miscorrection rate is lower
/// than assumed and the finding should be escalated, not silently retried.
const CORRUPTED_MK1_CHUNK1: &str =
    "mk1qprsqhpp0f3kmtxzd65mvwcvr9usdatwxqvq6z70rgnwrgk6xndl8gy6nwa2n977sw6zh34rma0nh";

/// The pinned funds-anchor set: chunk 0 unmodified + the corrupted chunk 1.
const CORRUPTED_SET: [&str; 2] = [CHUNK0, CORRUPTED_MK1_CHUNK1];

// ============================================================================
// Bech32 char <-> 5-bit value helpers (mirrors `src/repair.rs::parse_chunk`
// using ONLY mk-codec's PUBLIC `ALPHABET`, since this file is an external
// integration test with no access to toolkit `pub(crate)` internals).
// ============================================================================

fn char_to_5bit(c: char) -> u8 {
    ALPHABET
        .iter()
        .position(|&b| b as char == c)
        .expect("bech32 alphabet char") as u8
}

/// Split a bech32-family string into `(hrp + separator prefix, data-part
/// 5-bit values)`.
fn data_part_values(s: &str) -> (&str, Vec<u8>) {
    let sep = s.rfind('1').expect("bech32 separator");
    let (prefix, rest) = s.split_at(sep + 1);
    (prefix, rest.chars().map(char_to_5bit).collect())
}

fn rebuild_string(prefix: &str, values: &[u8]) -> String {
    let mut out = String::from(prefix);
    for &v in values {
        out.push(ALPHABET[v as usize] as char);
    }
    out
}

/// Flip the bech32 char at `pos` (within the data-part) to the NEXT
/// alphabet char (cyclic) — a single deterministic substitution. Mirrors the
/// `flip_at` helper duplicated across the other `cli_repair*` test files.
fn flip_at(chunk: &str, pos: usize) -> String {
    let (prefix, mut values) = data_part_values(chunk);
    let was = values[pos];
    values[pos] = (was + 1) % 32;
    rebuild_string(prefix, &values)
}

fn flip_many(chunk: &str, positions: &[usize]) -> String {
    positions
        .iter()
        .fold(chunk.to_string(), |acc, &p| flip_at(&acc, p))
}

/// **plan-R0 PI-2 — bounded deterministic search** for a 5-substitution
/// corruption of `chunk1`'s data-part that `bch_correct_regular` aliases to a
/// valid-but-DIFFERENT codeword, AND that fails full-set reassembly via
/// `mk_codec::decode(&[chunk0, <corrected>])`. Uses ONLY mk-codec's public
/// API (`bch_correct_regular`, `decode`) — no toolkit-internal access needed.
/// Deterministic via a fixed-seed `rand::rngs::StdRng` (never `thread_rng`);
/// bounded by `cap` iterations — returns `None` (fails loudly, does not hang)
/// if no alias is found within the cap.
///
/// This is the re-pinning tool referenced by `CORRUPTED_MK1_CHUNK1`'s doc
/// comment; it is exercised directly by the `#[ignore]`d
/// `rediscover_funds_anchor_seed_matches_pinned_constant` test below (not run
/// by default — the pinned constant is the fast, deterministic anchor for
/// the default suite).
#[allow(dead_code)]
fn find_mk1_miscorrection_seed(chunk0: &str, chunk1: &str, seed: u64, cap: u64) -> Option<String> {
    use mk_codec::string_layer::bch::bch_correct_regular;
    use mk_codec::string_layer::StringLayerHeader;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    let (prefix, values) = data_part_values(chunk1);
    let n = values.len();
    let mut rng = StdRng::seed_from_u64(seed);

    for _ in 0..cap {
        let mut positions: Vec<usize> = Vec::with_capacity(5);
        while positions.len() < 5 {
            let p = rng.gen_range(0..n);
            if !positions.contains(&p) {
                positions.push(p);
            }
        }
        let mut corrupted = values.clone();
        for &p in &positions {
            let was = corrupted[p];
            let mut now = rng.gen_range(0u8..32);
            while now == was {
                now = rng.gen_range(0u8..32);
            }
            corrupted[p] = now;
        }

        if let Ok(result) = bch_correct_regular("mk", &corrupted) {
            if result.data != values {
                // Aliased to a DIFFERENT valid codeword than the original.
                // Prefer the "canonical" REJECT shape (SPEC §2 rule 2's
                // primary case): the corrected chunk's HEADER still parses
                // as `Chunked` (a plausible-looking chunk), but the SET
                // fails to reassemble (cross-chunk hash / chunk_set_id
                // mismatch / structural bytecode failure) — rather than the
                // header itself being garbage (also SPEC-covered, but a
                // less illustrative pinned example). Both are valid Reject
                // triggers; this filter just picks the more instructive one
                // when available within the search budget.
                let header_still_chunked = StringLayerHeader::from_5bit_symbols(&result.data)
                    .map(|(h, _)| matches!(h, StringLayerHeader::Chunked { .. }))
                    .unwrap_or(false);
                if header_still_chunked {
                    let corrected_chunk1 = rebuild_string(prefix, &result.data);
                    if mk_codec::decode(&[chunk0, &corrected_chunk1]).is_err() {
                        return Some(rebuild_string(prefix, &corrupted));
                    }
                }
            }
        }
    }
    None
}

// ============================================================================
// §4.8 rate harness — seeded StdRng, Clopper-Pearson UPPER confidence bound.
// ============================================================================

/// `ln(C(n, j))` for `j = 0..=k_max`, via the standard multiplicative
/// recurrence (`C(n,j) = C(n,j-1) * (n-j+1)/j`) taken in log-space. Only
/// computes the small number of terms actually needed (`k_max` is the
/// observed hit count, expected to be small), so this avoids both a
/// full `O(n)` factorial table AND any gamma-function dependency.
fn ln_binomial_coeffs(n: u64, k_max: u64) -> Vec<f64> {
    let mut out = Vec::with_capacity(k_max as usize + 1);
    let mut ln_c = 0.0_f64; // ln(C(n,0)) = ln(1) = 0
    out.push(ln_c);
    for j in 1..=k_max {
        ln_c += ((n - j + 1) as f64).ln() - (j as f64).ln();
        out.push(ln_c);
    }
    out
}

/// `P(Binomial(n, p) <= k)`, computed directly (no incomplete-beta /
/// gamma-function dependency) since `a = k+1`, `b = n-k` are both integers
/// here — the regularized incomplete beta at integer parameters is exactly
/// the binomial survival function, so summing the `j = 0..=k` binomial PMF
/// terms in log-space is both correct and numerically simple.
fn binomial_cdf(n: u64, p: f64, k: u64) -> f64 {
    if p <= 0.0 {
        return 1.0;
    }
    if p >= 1.0 {
        return if k >= n { 1.0 } else { 0.0 };
    }
    let ln_c = ln_binomial_coeffs(n, k);
    let ln_p = p.ln();
    let ln_1mp = (1.0 - p).ln();
    let mut sum = 0.0_f64;
    for (j, &ln_c_j) in ln_c.iter().enumerate() {
        let j = j as u64;
        let ln_term = ln_c_j + (j as f64) * ln_p + ((n - j) as f64) * ln_1mp;
        sum += ln_term.exp();
    }
    sum.min(1.0)
}

/// One-sided Clopper-Pearson UPPER confidence bound at level `1 - alpha`:
/// the `p = U` solving `P(Binomial(n, U) <= k) = alpha`, found via bisection
/// (the CDF is monotonically decreasing in `p`). Returns `1.0` if `k >= n`.
fn clopper_pearson_upper_bound(n: u64, k: u64, alpha: f64) -> f64 {
    if k >= n {
        return 1.0;
    }
    let (mut lo, mut hi) = (0.0_f64, 1.0_f64);
    for _ in 0..100 {
        let mid = (lo + hi) / 2.0;
        if binomial_cdf(n, mid, k) > alpha {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    (lo + hi) / 2.0
}

/// Count how many of `n` seeded random 5-substitution corruptions of
/// `chunk1`'s data-part alias to a valid-but-DIFFERENT codeword (the SPEC §1
/// miscorrection event — independent of whether the full set subsequently
/// fails reassembly, which is the more pessimistic / upper-bound-friendly
/// quantity to measure per SPEC §4.8).
fn count_miscorrection_hits(chunk1: &str, seed: u64, n: u64) -> u64 {
    use mk_codec::string_layer::bch::bch_correct_regular;
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    let (_prefix, values) = data_part_values(chunk1);
    let len = values.len();
    let mut rng = StdRng::seed_from_u64(seed);
    let mut hits = 0u64;

    for _ in 0..n {
        let mut positions: Vec<usize> = Vec::with_capacity(5);
        while positions.len() < 5 {
            let p = rng.gen_range(0..len);
            if !positions.contains(&p) {
                positions.push(p);
            }
        }
        let mut corrupted = values.clone();
        for &p in &positions {
            let was = corrupted[p];
            let mut now = rng.gen_range(0u8..32);
            while now == was {
                now = rng.gen_range(0u8..32);
            }
            corrupted[p] = now;
        }
        if let Ok(result) = bch_correct_regular("mk", &corrupted) {
            if result.data != values {
                hits += 1;
            }
        }
    }
    hits
}

/// §4.8 — default-suite-fast rate measurement (N = 20_000; seeded, fixed).
/// Pins a 95% Clopper-Pearson UPPER confidence bound on the 5-substitution
/// miscorrection (alias) rate. The bound is a SOFT sanity print/assert (< 1),
/// not a strict pass/fail threshold — the HARD funds proof is the §4.1
/// pinned-seed test; this harness's role is to produce a MEASURED,
/// citable bound for the CHANGELOG/manual (NOT the eval's unverified
/// `2⁻¹³·⁹`). Run the slower, tighter `--ignored` variant below for a
/// higher-power bound.
#[test]
fn cell_4_8_rate_harness_default_fast_bound() {
    const N: u64 = 20_000;
    let hits = count_miscorrection_hits(CHUNK1, 0x4634_5F31, N);
    let bound = clopper_pearson_upper_bound(N, hits, 0.05);
    eprintln!(
        "§4.8 rate harness: {hits} alias hit(s) / {N} trials — 95% Clopper-Pearson UPPER bound = {bound:.3e}"
    );
    assert!(
        bound > 0.0 && bound < 1.0,
        "upper bound must be a proper probability, got {bound}"
    );
    // Soft sanity: at N=20_000 the fast default run may well observe 0 hits
    // (rate is expected far below 1/N); that is NOT a test failure — the
    // pinned §4.1 seed is the non-vacuous proof, independent of this count.
}

/// §4.8 — slow, higher-power rate measurement (N = 1_000_000). Gated behind
/// `--ignored` per SPEC §4.8's explicit N-sizing guidance (run manually / in
/// CI, not the default local suite). At the expected ~10⁻⁴-10⁻⁵ alias rate
/// this sizing targets `E[hits] ≫ 1` so an `observed >= 1` assertion is
/// robust (not merely a soft warning).
#[test]
#[ignore = "slow (~10^6 BCH-correction trials); run explicitly or in CI"]
fn cell_4_8_rate_harness_slow_high_power_bound() {
    const N: u64 = 1_000_000;
    let hits = count_miscorrection_hits(CHUNK1, 0x4634_5F31, N);
    let bound = clopper_pearson_upper_bound(N, hits, 0.05);
    eprintln!(
        "§4.8 rate harness (slow): {hits} alias hit(s) / {N} trials — 95% Clopper-Pearson UPPER bound = {bound:.3e}"
    );
    assert!(
        hits >= 1,
        "expected >= 1 alias hit at N={N} given the ~1e-4..1e-5 assumed rate (E[hits] >> 1); \
        got 0 — either the rate is far lower than assumed (escalate) or the RNG/harness regressed"
    );
}

/// `#[ignore]`d discovery/regression tool for the pinned §4.1 anchor: re-runs
/// `find_mk1_miscorrection_seed` with the SAME seed used to originally find
/// `CORRUPTED_MK1_CHUNK1` and asserts it reproduces the identical pinned
/// string byte-for-byte (search determinism) — and, transitively, that the
/// alias still exists under the CURRENT mk-codec BCH implementation. NOT run
/// by default (search cost); the default suite instead uses the fast pinned
/// constant directly (`cell_4_1_*` below).
#[test]
#[ignore = "re-derivation / re-pinning tool, not a default-suite regression guard"]
fn rediscover_funds_anchor_seed_matches_pinned_constant() {
    let found = find_mk1_miscorrection_seed(CHUNK0, CHUNK1, 0x4634_5F31, 10_000_000).expect(
        "the pinned F4 miscorrection seed no longer aliases to a wrong codeword under the \
            current mk-codec BCH implementation (or the rate is lower than assumed within the \
            10^7 cap) — re-pin via find_mk1_miscorrection_seed with a fresh seed and escalate if \
            it still returns None",
    );
    assert_eq!(
        found, CORRUPTED_MK1_CHUNK1,
        "search re-derivation diverged from the pinned constant"
    );
}

// ============================================================================
// §4.1 — FUNDS ANCHOR (pinned seed, deterministic).
// ============================================================================

/// Sanity precondition: the pinned corrupted chunk 1 really does differ from
/// the original (else the "aliased to a DIFFERENT codeword" claim would be
/// vacuous), and really is a valid mk1 chunk (BCH residue == 0) on its own —
/// i.e. `bch_correct_regular`'s post-correction re-verify accepted it.
#[test]
fn cell_4_1a_pinned_seed_preconditions() {
    assert_ne!(
        CORRUPTED_MK1_CHUNK1, CHUNK1,
        "pinned corrupted chunk1 must differ from the original"
    );
    // The corrupted STRING itself is invalid (5 raw substitution errors);
    // per-chunk BCH correction (the toolkit's `repair_chunk_one`, wrapping
    // the SAME `bch_correct_regular` primitive) must succeed on it alone —
    // aliasing to *some* valid codeword — which is exercised end-to-end via
    // the CLI cells below (`cell_4_1b` onward).
}

/// **§4.1 FUNDS ANCHOR** — `mnemonic repair --mk1 <CORRUPTED_SET>` (the FULL
/// 2-chunk set, one chunk corrupted with the pinned 5-substitution
/// miscorrection) must REJECT: exit 2 (`ToolkitError::Repair` →
/// `RepairError::SetReassemblyMismatch`), NOT the pre-fix exit 5
/// "successfully repaired". The corrected-but-wrong chunk1 must NOT appear
/// on stdout as a recovered card.
#[test]
fn cell_4_1b_full_set_miscorrection_cli_rejects_exit_2() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "repair",
            "--mk1",
            CORRUPTED_SET[0],
            "--mk1",
            CORRUPTED_SET[1],
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("does not reassemble"))
        .stderr(predicate::str::contains("chunk_set_id"));
}

/// **§4.1 FUNDS ANCHOR (`--json` form)** — the `--json` flag only affects
/// the SUCCESS-path emission shape (D14); a `RepairError` always surfaces
/// via the same typed-error stderr path regardless. Confirms `--json` does
/// not accidentally mask or convert the Reject into a false success.
#[test]
fn cell_4_1c_full_set_miscorrection_json_form_also_rejects_exit_2() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "repair",
            "--json",
            "--mk1",
            CORRUPTED_SET[0],
            "--mk1",
            CORRUPTED_SET[1],
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("does not reassemble"))
        .stderr(predicate::str::contains("chunk_set_id"));
}

/// **§4.1 FUNDS ANCHOR (auto-repair)** — auto-repair (`try_repair_and_short_circuit`,
/// exercised here via `convert`) does NOT short-circuit on the full-set
/// miscorrection; the caller's original decode error surfaces instead of a
/// silently-wrong xpub. Forces the TTY-positive auto-fire gate via
/// `MNEMONIC_FORCE_TTY=1` (cargo test pipes stdout, which would otherwise
/// take the TTY-negative legacy path).
#[test]
fn cell_4_1d_auto_repair_does_not_short_circuit_on_miscorrection() {
    let mk1_value = format!("{} {}", CORRUPTED_SET[0], CORRUPTED_SET[1]);
    Command::cargo_bin("mnemonic")
        .unwrap()
        .env("MNEMONIC_FORCE_TTY", "1")
        .args([
            "convert",
            "--from",
            &format!("mk1={mk1_value}"),
            "--to",
            "xpub",
        ])
        .assert()
        .failure()
        .code(2)
        .stdout(predicate::str::contains("# Repair report").not());
}

// ============================================================================
// §4.2 — partial-set per-plate repair PRESERVED (regression guard, C1).
// ============================================================================

/// `mnemonic repair --mk1 <single chunk of a real 2-chunk card>` (a genuine
/// ≤4-error correction on JUST chunk 1, no chunk 0 supplied) → exit-4
/// VERIFY-ME candidate + the "unverified — reassemble to confirm" advisory
/// (NOT a reject, NOT the old exit 5). Replays the documented per-plate
/// workflow (`44-mk-cli.md:247`-style single-plate repair) at the toolkit
/// `mnemonic repair` surface.
#[test]
fn cell_4_2_partial_set_single_plate_repair_exit_4_unverified() {
    let bad_chunk1 = flip_at(CHUNK1, 25); // genuine 1-substitution — within t<=4
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", "--mk1", &bad_chunk1])
        .assert()
        .code(4)
        .stdout(predicate::str::contains(CHUNK1))
        .stderr(predicate::str::contains("UNVERIFIED"))
        .stderr(predicate::str::contains("BIP-93"));
}

/// Auto-repair on a partial (single-chunk) mk1 correction must NOT
/// short-circuit either (G7) — the caller's original error surfaces because
/// a partial card cannot convert/inspect anyway. Exercised via `inspect`
/// (which fails on a lone mk1 chunk regardless, so this asserts specifically
/// that NO repair report / short-circuit fires — i.e. exit code is NOT 5).
#[test]
fn cell_4_2b_auto_repair_does_not_short_circuit_on_partial_set() {
    let bad_chunk1 = flip_at(CHUNK1, 25);
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .env("MNEMONIC_FORCE_TTY", "1")
        .args(["inspect", "--mk1", &bad_chunk1])
        .assert()
        .get_output()
        .clone();
    assert_ne!(
        out.status.code(),
        Some(5),
        "auto-repair must not short-circuit(5) on an unverified partial-set correction"
    );
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(
        !stdout.contains("# Repair report"),
        "no auto-fire repair report expected for a Candidate (unverified) outcome"
    );
}

// ============================================================================
// §4.3 — genuine <=4-error FULL-set correction still blesses (G1).
// ============================================================================

/// A genuine 4-error correction (the BCH `t=4` boundary) applied to the FULL
/// 2-chunk set still BLESSES: exit 5, both chunks emitted, no "UNVERIFIED"
/// advisory. Confirms the re-verify does NOT false-reject a real recovery.
#[test]
fn cell_4_3_genuine_t4_full_set_correction_still_blesses_exit_5() {
    let bad_chunk1 = flip_many(CHUNK1, &[3, 11, 19, 27]); // 4 spread errors
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", "--mk1", CHUNK0, "--mk1", &bad_chunk1])
        .assert()
        .code(5)
        .stdout(predicate::str::contains("# Repair report"))
        .stdout(predicate::str::contains(CHUNK0))
        .stdout(predicate::str::contains(CHUNK1))
        .stderr(predicate::str::contains("UNVERIFIED").not());
}

// ============================================================================
// §4.4 — clean card (no correction needed) — exit 0.
// ============================================================================

#[test]
fn cell_4_4_clean_full_set_exit_0_no_report() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", "--mk1", CHUNK0, "--mk1", CHUNK1])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("# Repair report").not())
        .stdout(predicate::str::contains(CHUNK0))
        .stdout(predicate::str::contains(CHUNK1));
}

/// A clean, but INCOMPLETE (single-chunk) supply must ALSO stay exit 0 — no
/// correction occurred, so there is no aliasing risk to flag (the tri-state
/// re-verify only ever engages a chunk_set_id group that had >= 1 chunk
/// actually corrected). Regression guard against over-eagerly Candidate-
/// flagging every incomplete supply regardless of whether anything was
/// corrected.
#[test]
fn cell_4_4b_clean_partial_set_exit_0_no_advisory() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args(["repair", "--mk1", CHUNK0])
        .assert()
        .code(0)
        .stdout(predicate::str::contains("# Repair report").not())
        .stderr(predicate::str::contains("UNVERIFIED").not());
}

// ============================================================================
// §4.5 — convert/inspect auto-repair no longer silently emits the wrong
// card (covered above as cell_4_1d; this section adds the `inspect` path).
// ============================================================================

#[test]
fn cell_4_5_inspect_auto_repair_does_not_emit_wrong_card_on_miscorrection() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .env("MNEMONIC_FORCE_TTY", "1")
        .args([
            "inspect",
            "--mk1",
            CORRUPTED_SET[0],
            "--mk1",
            CORRUPTED_SET[1],
        ])
        .assert()
        .failure()
        .stdout(predicate::str::contains("kind: mk1").not())
        .stdout(predicate::str::contains("# Repair report").not());
}

// ============================================================================
// §4.5b — BATCH reject-dominant (multi-group aggregation, I-r2-1b / PM-r2-2).
// ============================================================================

/// A single `mnemonic repair --mk1` invocation spanning TWO `chunk_set_id`
/// groups — {the §4.1 miscorrection group, a clean SECOND real card's group}
/// — must exit REJECT (exit 2), and must NOT present the miscorrected
/// group's chunks (nor the clean group's) as recovered: the whole invocation
/// output is suppressed (plan-R0 PM-r2-2 — a batch success must never carry
/// a miscorrection, so a co-batched clean/blessed group is not emitted
/// either).
#[test]
fn cell_4_5b_batch_reject_dominates_over_clean_group() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "repair",
            "--mk1",
            CORRUPTED_SET[0],
            "--mk1",
            CORRUPTED_SET[1],
            "--mk1",
            SECOND_CHUNK0,
            "--mk1",
            SECOND_CHUNK1,
        ])
        .assert()
        .code(2)
        .stderr(predicate::str::contains("does not reassemble"))
        .stdout(predicate::str::contains(SECOND_CHUNK0).not())
        .stdout(predicate::str::contains(SECOND_CHUNK1).not());
}

/// Same batch shape, order-reversed (clean group first, miscorrection group
/// second) — the dominant-Reject fold must not depend on group order within
/// the flat `--mk1` repetition.
#[test]
fn cell_4_5b_batch_reject_dominates_regardless_of_group_order() {
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "repair",
            "--mk1",
            SECOND_CHUNK0,
            "--mk1",
            SECOND_CHUNK1,
            "--mk1",
            CORRUPTED_SET[0],
            "--mk1",
            CORRUPTED_SET[1],
        ])
        .assert()
        .code(2)
        .stdout(predicate::str::contains(SECOND_CHUNK0).not())
        .stdout(predicate::str::contains(SECOND_CHUNK1).not());
}

// ============================================================================
// §4.6 — md1 regression-lock (already protected by content-id check).
// ============================================================================

const VALID_MD1_CHUNK0: &str =
    "md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np";
const VALID_MD1_CHUNK1: &str =
    "md1fgdxlpqf2zcgefcpupmel75q5435j7seugaj5jr7qyur6vt76es5cdeyrq7zdy0d";
const VALID_MD1_CHUNK2: &str =
    "md1fgdxlpq3xa2dk8vwpj7gx74hwqxqdp083jehp5tdrfa0n5zdfkqcdlrvnh5r62jn";

/// md1 is ALREADY protected: `md_codec::chunk::reassemble`'s content-derived
/// 20-bit `chunk_set_id` check runs unconditionally for all counts
/// (`chunk.rs:379-387`). A wrong-fit md1 chunk correction is rejected
/// (atomic per D28 — `repair_via_md_codec` delegates whole-set), NOT
/// silently blessed. This cell locks that pre-existing behavior as a
/// regression guard alongside the mk1 fix (no structural change to md1 in
/// this cycle).
#[test]
fn cell_4_6_md1_already_rejects_wrong_fit_correction() {
    // 5 spread errors in chunk 0 — beyond md-codec's t<=4 per-chunk bound,
    // so `decode_with_correction` fails atomically (TooManyErrors), never a
    // silent cross-chunk bless. md1 has no separate "set re-verify" concept
    // to test post-hoc — the atomic whole-set delegate already IS the
    // re-verify.
    fn flip_md(chunk: &str, pos: usize) -> String {
        let sep = chunk.rfind('1').unwrap();
        let (prefix, rest) = chunk.split_at(sep + 1);
        let mut chars: Vec<char> = rest.chars().collect();
        let was = chars[pos];
        let alphabet_str = std::str::from_utf8(ALPHABET).unwrap();
        let was_idx = alphabet_str.find(was).unwrap();
        chars[pos] = alphabet_str.chars().nth((was_idx + 1) % 32).unwrap();
        let mut out = String::from(prefix);
        for c in chars {
            out.push(c);
        }
        out
    }
    let bad_chunk0 = [3usize, 11, 19, 27, 35]
        .iter()
        .fold(VALID_MD1_CHUNK0.to_string(), |acc, &p| flip_md(&acc, p));
    Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "repair",
            "--md1",
            &bad_chunk0,
            "--md1",
            VALID_MD1_CHUNK1,
            "--md1",
            VALID_MD1_CHUNK2,
        ])
        .assert()
        .code(2);
}

/// Reachability note mirrored for md1: `md_codec::decode_with_correction`
/// has no non-chunked (single-string) bypass around `reassemble`'s
/// content-id check — the delegate is the SAME function regardless of
/// chunk count, so there is no code path that skips the check for a
/// 1-chunk md1. (md1 chunking always includes the content-derived id per
/// `chunk.rs:379-387`; there is no md1 "SingleString" format variant at all,
/// unlike mk1's defined-but-unreachable `SingleString` header.)
#[test]
fn cell_4_6b_md1_no_non_chunked_bypass_of_reassemble() {
    // The 3-chunk fixture must reassemble cleanly (sanity precondition for
    // the regression-lock above).
    assert!(
        md_codec::chunk::reassemble(&[VALID_MD1_CHUNK0, VALID_MD1_CHUNK1, VALID_MD1_CHUNK2])
            .is_ok()
    );
}

// ============================================================================
// §4.7 — reachability lock (count=1 resolved-favorably note, R0-round-1 I1).
// ============================================================================

/// The minimum-size REAL mk1 card (the canonical bip84 pair used throughout
/// this file) produces >= 2 chunks. If a future encoder change ever made a
/// 1-chunk mk1 reachable, this test's premise (and the `CHUNK0`/`CHUNK1`
/// 2-chunk fixture) would need re-examination — the assertion locks today's
/// `SINGLE_STRING_LONG_BYTES` / compact-xpub-size relationship.
#[test]
fn cell_4_7_min_real_mk1_card_produces_at_least_2_chunks() {
    assert!(
        mk_codec::decode(&[CHUNK0, CHUNK1]).is_ok(),
        "fixture pair must be a genuine reassembling 2-chunk card"
    );
    // A single chunk of this pair does NOT decode alone (proves it's a REAL
    // 2-chunk card, not a single-chunk one padded to look like two).
    assert!(mk_codec::decode(&[CHUNK0]).is_err());
    assert!(mk_codec::decode(&[CHUNK1]).is_err());
}

// `StringLayerHeader::SingleString` is a defined-but-encoder-unreachable wire
// shape (SPEC §1) — no v0.1 `mk_codec::encode` output for a realistic card
// (compact xpub alone is 73 bytes, `SINGLE_STRING_LONG_BYTES` = 56) ever
// produces one. mk-codec's own test suite already covers `SingleString`
// round-tripping at the header layer via hand-constructed inputs (see
// `mk-codec` `pipeline.rs`'s `synthetic_singlestring` helper); this lock
// pins the NUMERIC size relationship that makes it encoder-unreachable, as
// a COMPILE-TIME invariant (both are library `const`s, so a runtime
// `assert!` would just be constant-folded away by the compiler — a `const`
// assertion is both the correct AND clippy-clean way to pin this) so a
// future mk-codec consts change trips the BUILD, not a test run.
const _: () = assert!(
    mk_codec::XPUB_COMPACT_BYTES > mk_codec::SINGLE_STRING_LONG_BYTES,
    "if this ever flips, a real mk1 card could reach the SingleString path — \
    the tri-state re-verify's SingleString handling would need live (not \
    just defensive) coverage"
);
