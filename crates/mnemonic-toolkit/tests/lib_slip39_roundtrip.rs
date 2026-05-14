//! v0.13.0 P1c-E.2 G2 — SLIP-0039 split/combine round-trip property test.
//!
//! For each `(entropy_size, group_config, extendable, trial)` tuple in
//! the test matrix:
//!
//! 1. Derive a master secret from the trial seed (deterministic).
//! 2. `slip39_split(secret, b"", group_threshold, groups, iter_exp=0,
//!    extendable, identifier=None, rng)` → `Vec<Vec<Share>>`.
//! 3. Render each share to its mnemonic; pick `member_threshold` from
//!    each of `group_threshold` groups; re-parse to fresh `Share`s.
//! 4. Shuffle the selected share set deterministically.
//! 5. `slip39_combine(shares, b"")` → recovered master secret bytes.
//! 6. Assert byte-equal to the original master secret.
//!
//! The render→parse roundtrip on each selected share doubles as
//! coverage of the wire-format encoding pathway: any encoding bug
//! that affects metadata configurations not covered by G1's vectors
//! would surface here as a combine-time refusal or recovered-secret
//! mismatch.
//!
//! Matrix shape:
//!
//! - 5 entropy sizes: {16, 20, 24, 28, 32} bytes.
//! - 4 group configs:
//!   - `(1, [(1, 1)])` — 1-of-1 trivial.
//!   - `(1, [(2, 3)])` — single group, 2-of-3.
//!   - `(1, [(2, 3), (3, 5)])` — 1-of-2 groups (either group reconstructs).
//!   - `(2, [(3, 3), (3, 5), (2, 5)])` — 2-of-3 groups, varied member configs.
//!
//!   Note: plan §4.2 lists the second config as `(2, [(2, 3)])`; that's
//!   a plan typo (group_threshold=2 with 1 group violates the
//!   `group_threshold <= groups.len()` invariant). Corrected to
//!   `(1, [(2, 3)])` here matching the description "single group 2-of-3".
//! - 2 extendable axes: {false, true}.
//! - Default `cargo test`: 5 trials per shape → **200 trials total**
//!   (target ≤ 5 seconds at iter_exp=0).
//! - `#[ignore]`-gated extensive run: 50 trials per shape → **2000
//!   trials total** (CI-only via `--include-ignored`). See
//!   `feedback_default_cargo_test_runs_sibling_dependent_tests` for
//!   the gating rationale.
//!
//! Deterministic seeding via `ChaCha20Rng::seed_from_u64(trial_seed)`
//! where `trial_seed = SEED_BASE + per_shape_offset`. The same seed
//! produces the same secret + share bytes across runs, so a regression
//! caught by a specific trial is reproducible from the test name and
//! seed-base alone.

use mnemonic_toolkit::slip39::{
    parse_slip39_share, render_slip39_share, slip39_combine, slip39_split,
    GroupSpec, Share,
};
use rand_chacha::ChaCha20Rng;
use rand_core::{RngCore, SeedableRng};

const ENTROPY_SIZES: &[usize] = &[16, 20, 24, 28, 32];

/// `(group_threshold, &[(member_threshold, member_count), ...])`.
const GROUP_CONFIGS: &[(u8, &[(u8, u8)])] = &[
    (1, &[(1, 1)]),
    (1, &[(2, 3)]),
    (1, &[(2, 3), (3, 5)]),
    (2, &[(3, 3), (3, 5), (2, 5)]),
];

const EXT_AXES: &[bool] = &[false, true];

/// Seed base for deterministic trial RNG. Bump on any matrix shape
/// change so old saved-state tests don't false-pass against the new
/// shape via collision.
const SEED_BASE: u64 = 0xD3AD_BEEF_CAFE_0001;

/// Fisher-Yates shuffle using `rand_core::RngCore` (avoids a `rand`
/// crate dependency — we already use `rand_chacha` for seeded RNG).
fn shuffle<T>(slice: &mut [T], rng: &mut impl RngCore) {
    for i in (1..slice.len()).rev() {
        let j = (rng.next_u32() as usize) % (i + 1);
        slice.swap(i, j);
    }
}

/// Run one round-trip trial. Panics on any mismatch — the test name
/// + the printed shape parameters identify which trial.
fn run_trial(
    entropy_size: usize,
    group_threshold: u8,
    groups: &[(u8, u8)],
    extendable: bool,
    trial_idx: u64,
) {
    let seed = SEED_BASE.wrapping_add(trial_idx);
    let mut rng = ChaCha20Rng::seed_from_u64(seed);

    // Derive a deterministic master secret from the trial RNG.
    let mut secret = vec![0u8; entropy_size];
    rng.fill_bytes(&mut secret);

    let group_specs: Vec<GroupSpec> = groups
        .iter()
        .map(|&(t, n)| GroupSpec {
            member_threshold: t,
            member_count: n,
        })
        .collect();

    let split_shares = slip39_split(
        &secret,
        b"",
        group_threshold,
        &group_specs,
        0, // iteration_exponent (low for test perf)
        extendable,
        None, // identifier — derive from RNG
        &mut rng,
    )
    .unwrap_or_else(|e| {
        panic!(
            "G2 trial (entropy={entropy_size}, gt={group_threshold}, groups={groups:?}, \
             ext={extendable}, seed={seed:#x}): split must succeed, got {e:?}"
        )
    });

    // Sanity: split returned one Vec<Share> per group, each of length
    // member_count.
    assert_eq!(
        split_shares.len(),
        groups.len(),
        "split returned {} group-vectors but config has {} groups",
        split_shares.len(),
        groups.len(),
    );
    for (g_idx, group) in split_shares.iter().enumerate() {
        assert_eq!(
            group.len(),
            groups[g_idx].1 as usize,
            "group {g_idx} has {} shares but member_count is {}",
            group.len(),
            groups[g_idx].1,
        );
    }

    // Select `member_threshold` shares from the first `group_threshold`
    // groups via the render → parse path (doubles as wire-format
    // encoding coverage; avoids needing Clone on Share).
    let mut selected: Vec<Share> = Vec::new();
    for g_idx in 0..group_threshold as usize {
        let needed = groups[g_idx].0 as usize;
        for share in split_shares[g_idx].iter().take(needed) {
            let mnemonic = render_slip39_share(share);
            let parsed = parse_slip39_share(&mnemonic).unwrap_or_else(|e| {
                panic!(
                    "G2 trial (entropy={entropy_size}, gt={group_threshold}, \
                     groups={groups:?}, ext={extendable}, seed={seed:#x}): \
                     render/parse roundtrip on emitted share failed: {e:?}"
                )
            });
            selected.push(parsed);
        }
    }

    // Shuffle the selected share set — combine must not depend on
    // input order (shares carry their group_index + member_index
    // internally).
    shuffle(&mut selected, &mut rng);

    let recovered = slip39_combine(&selected, b"").unwrap_or_else(|e| {
        panic!(
            "G2 trial (entropy={entropy_size}, gt={group_threshold}, groups={groups:?}, \
             ext={extendable}, seed={seed:#x}): combine must succeed, got {e:?}"
        )
    });

    assert_eq!(
        recovered.as_slice(),
        secret.as_slice(),
        "G2 trial (entropy={entropy_size}, gt={group_threshold}, groups={groups:?}, \
         ext={extendable}, seed={seed:#x}): recovered secret mismatch",
    );
}

/// Run the full matrix at `trials_per_shape`. `entropy x config x ext`
/// = 5 × 4 × 2 = 40 shapes; total trials = 40 × `trials_per_shape`.
fn run_matrix(trials_per_shape: u64) {
    let mut trial_counter: u64 = 0;
    for &entropy in ENTROPY_SIZES {
        for &(group_threshold, groups) in GROUP_CONFIGS {
            for &ext in EXT_AXES {
                for trial in 0..trials_per_shape {
                    let trial_idx = trial_counter * 1009 + trial; // 1009 prime salt
                    run_trial(entropy, group_threshold, groups, ext, trial_idx);
                }
                trial_counter += 1;
            }
        }
    }
}

#[test]
fn roundtrip_default_matrix_200_trials() {
    // 5 entropies × 4 configs × 2 ext-axes × 5 trials = 200 trials.
    run_matrix(5);
}

#[test]
#[ignore = "G2 extensive 2000-trial run; opt in via `cargo test -- --include-ignored` or the dedicated CI job."]
fn roundtrip_extensive_matrix_2000_trials() {
    // 5 entropies × 4 configs × 2 ext-axes × 50 trials = 2000 trials.
    run_matrix(50);
}
