//! Coldcard-compatible BIP-39 ↔ BIP-39 all-or-nothing Seed XOR splitter.
//!
//! Given a single BIP-39 entropy of length N bytes (N ∈ {16, 20, 24, 28,
//! 32}), split into K BIP-39 entropies of the same length such that the
//! bytewise XOR of all K shares reconstitutes the master. Coldcard
//! interop is pinned at lengths {16, 24, 32} (= 12/18/24-word BIP-39);
//! 20/28-byte (= 15/21-word) are toolkit-only extensions.
//!
//! See `design/SPEC_seed_xor_v0_12_0.md` §2.1 for the algorithm contract.
//!
//! NOT a threshold scheme — ALL K shares are required to reconstruct.
//! For K-of-N use SLIP-39 via v0.13.0 `mnemonic slip39`.
//!
//! This module emits raw entropy bytes; per-share BIP-39 checksum
//! recomputation is the CLI-layer's responsibility (`src/cmd/seed_xor.rs`).
//!
//! Library-local `SeedXorError` — see [[library-error-and-language-surface-promotion]]
//! FOLLOWUP for the future crate-shape unification with `ToolkitError`.

// Phase 1 RED stub: type signatures + error variants only. Bodies land in P1 GREEN.

/// Accepted entropy lengths (bytes). Maps 1:1 onto BIP-39 word counts:
/// `{16,20,24,28,32}` = `{12,15,18,21,24}` words.
pub const VALID_ENTROPY_LENGTHS: &[usize] = &[16, 20, 24, 28, 32];

/// Minimum share count for a meaningful split.
pub const MIN_SHARES: usize = 2;

/// Errors returned by Seed XOR library functions. Mapped to
/// `ToolkitError::BadInput` at the CLI boundary (`cmd/seed_xor.rs`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeedXorError {
    /// Master entropy length is not in `VALID_ENTROPY_LENGTHS`.
    BadEntropyLength {
        got: usize,
        expected_one_of: &'static [usize],
    },
    /// Share count is < `MIN_SHARES` (= 2). Single-share "split" is a
    /// no-op; the toolkit refuses to surface this nonsense input.
    TooFewShares { got: usize, min: usize },
    /// Combine called with shares of differing lengths.
    MismatchedShareLengths { lengths: Vec<usize> },
}

impl std::fmt::Display for SeedXorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SeedXorError::BadEntropyLength {
                got,
                expected_one_of,
            } => write!(
                f,
                "seed-xor: entropy length {} bytes invalid; expected one of {:?}",
                got, expected_one_of,
            ),
            SeedXorError::TooFewShares { got, min } => write!(
                f,
                "seed-xor split: --shares must be >= {}; got {}",
                min, got,
            ),
            SeedXorError::MismatchedShareLengths { lengths } => write!(
                f,
                "seed-xor combine: all shares must be the same length; got mix of {:?} bytes",
                lengths,
            ),
        }
    }
}

impl std::error::Error for SeedXorError {}

/// Split `entropy` into `n_shares` byte-XOR shares using the supplied
/// CSPRNG to fill the first `n_shares - 1` shares; the Nth share is
/// `entropy XOR share[0] XOR ... XOR share[N-2]`.
///
/// Returns `Vec<Zeroizing<Vec<u8>>>` — each share's heap zeroizes on drop.
///
/// Errors:
/// - [`SeedXorError::BadEntropyLength`] if `entropy.len()` is not in
///   `VALID_ENTROPY_LENGTHS`.
/// - [`SeedXorError::TooFewShares`] if `n_shares < MIN_SHARES`.
pub fn seed_xor_split(
    entropy: &[u8],
    n_shares: usize,
    rng: &mut (impl rand_core::CryptoRng + rand_core::RngCore),
) -> Result<Vec<zeroize::Zeroizing<Vec<u8>>>, SeedXorError> {
    validate_entropy_len(entropy.len())?;
    validate_share_count(n_shares)?;

    let n = entropy.len();
    let mut shares: Vec<zeroize::Zeroizing<Vec<u8>>> = Vec::with_capacity(n_shares);

    // Last share starts as the master; we XOR each random mask into it
    // so the final state is `entropy XOR share[0] XOR ... XOR share[N-2]`.
    let mut last = zeroize::Zeroizing::new(entropy.to_vec());

    for _ in 0..n_shares - 1 {
        let mut mask = zeroize::Zeroizing::new(vec![0u8; n]);
        rng.fill_bytes(mask.as_mut_slice());
        for i in 0..n {
            last[i] ^= mask[i];
        }
        shares.push(mask);
    }
    shares.push(last);

    Ok(shares)
}

/// Deterministic split — mirrors Coldcard `shared/xor_seed.py`. For
/// share index `i` in `[0, n_shares - 1)`:
///
/// ```text
/// share[i] = sha256d(b'Batshitoshi ' || raw_secret || ('%d of %d parts' % (i, num_parts)).bytes)[:n]
/// ```
///
/// where `sha256d(x) = sha256(sha256(x))`. The last share is the
/// master XOR'd with shares 0..N-2.
///
/// Pinned against the Coldcard reference at the G1 acceptance gate.
pub fn seed_xor_split_deterministic(
    entropy: &[u8],
    n_shares: usize,
) -> Result<Vec<zeroize::Zeroizing<Vec<u8>>>, SeedXorError> {
    validate_entropy_len(entropy.len())?;
    validate_share_count(n_shares)?;

    use bitcoin::hashes::{sha256d, Hash};

    let n = entropy.len();
    let mut shares: Vec<zeroize::Zeroizing<Vec<u8>>> = Vec::with_capacity(n_shares);
    let mut last = zeroize::Zeroizing::new(entropy.to_vec());

    for i in 0..n_shares - 1 {
        let mut buf: Vec<u8> = Vec::with_capacity(12 + n + 16);
        buf.extend_from_slice(b"Batshitoshi ");
        buf.extend_from_slice(entropy);
        buf.extend_from_slice(format!("{} of {} parts", i, n_shares).as_bytes());
        let h = sha256d::Hash::hash(&buf);
        zeroize::Zeroize::zeroize(&mut buf);
        let share_bytes: Vec<u8> = h.as_byte_array()[..n].to_vec();
        let mask = zeroize::Zeroizing::new(share_bytes);
        for j in 0..n {
            last[j] ^= mask[j];
        }
        shares.push(mask);
    }
    shares.push(last);

    Ok(shares)
}

/// Combine `shares` via bytewise XOR. All shares must be the same
/// length; the recovered entropy is `share[0] XOR share[1] XOR ... XOR share[N-1]`.
///
/// Errors:
/// - [`SeedXorError::TooFewShares`] if `shares.len() < MIN_SHARES`.
/// - [`SeedXorError::MismatchedShareLengths`] if shares have differing lengths.
/// - [`SeedXorError::BadEntropyLength`] if the (uniform) share length is not in
///   `VALID_ENTROPY_LENGTHS`.
pub fn seed_xor_combine(shares: &[&[u8]]) -> Result<zeroize::Zeroizing<Vec<u8>>, SeedXorError> {
    validate_share_count(shares.len())?;

    let lengths: Vec<usize> = shares.iter().map(|s| s.len()).collect();
    let first_len = lengths[0];
    if lengths.iter().any(|&l| l != first_len) {
        return Err(SeedXorError::MismatchedShareLengths { lengths });
    }
    validate_entropy_len(first_len)?;

    let mut out = zeroize::Zeroizing::new(vec![0u8; first_len]);
    for share in shares {
        for i in 0..first_len {
            out[i] ^= share[i];
        }
    }
    Ok(out)
}

fn validate_entropy_len(got: usize) -> Result<(), SeedXorError> {
    if VALID_ENTROPY_LENGTHS.contains(&got) {
        Ok(())
    } else {
        Err(SeedXorError::BadEntropyLength {
            got,
            expected_one_of: VALID_ENTROPY_LENGTHS,
        })
    }
}

fn validate_share_count(got: usize) -> Result<(), SeedXorError> {
    if got >= MIN_SHARES {
        Ok(())
    } else {
        Err(SeedXorError::TooFewShares {
            got,
            min: MIN_SHARES,
        })
    }
}
