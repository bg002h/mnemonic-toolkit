//! Trezor SLIP-0039 hierarchical K-of-N Shamir Secret Sharing.
//!
//! See `design/SPEC_slip39_v0_13_0.md` for the contract. Phase 1a lands
//! the math primitives only (GF(256) field arithmetic + Lagrange
//! interpolation). Encryption pipeline (Feistel + PBKDF2) ships at
//! P1b. Share encoding (RS1024 + wordlist + bit-packing + parse/render)
//! ships at P1c. The public `slip39_split` / `slip39_combine` surface
//! follows in P1c-E.
//!
//! Library-local `Slip39Error` per the v0.11.0 / v0.12.0 precedent;
//! tracked under FOLLOWUP `library-error-and-language-surface-promotion`
//! for the future crate-shape unification with `ToolkitError`.

pub mod error;
pub mod feistel;
pub mod gf256;
pub mod lagrange;
pub mod rs1024;
pub mod share;
pub mod wordlist;

pub use error::Slip39Error;
pub use share::{parse_slip39_share, render_slip39_share, Share};

use std::collections::BTreeMap;

use hmac::{Hmac, Mac};
use rand_core::{CryptoRng, RngCore};
use sha2::Sha256;
use zeroize::Zeroizing;

// ============================================================================
// SLIP-0039 algorithm constants
// ============================================================================

/// Shamir x-coordinate where the master secret is stored
/// (`SECRET_INDEX` per SLIP-0039 §"Sharing a secret").
const SECRET_INDEX: u8 = 255;

/// Shamir x-coordinate where the digest payload is stored
/// (`DIGEST_INDEX` per SLIP-0039 §"Sharing a secret").
const DIGEST_INDEX: u8 = 254;

/// Length in bytes of the HMAC-SHA-256-derived digest prefix.
const DIGEST_LENGTH: usize = 4;

/// Allowed SLIP-39 master-secret / share-value byte lengths.
const VALID_SECRET_LENGTHS: &[usize] = &[16, 20, 24, 28, 32];

/// Sentinel `group_idx` value carried by `InsufficientShares` when the
/// failure is at the GROUP level (too few distinct groups provided),
/// distinct from the member-level failure where `group_idx` is the real
/// 0..=15 group index. 255 is outside the 4-bit group-index range so it
/// can never collide with a real group.
const GROUP_LEVEL_SENTINEL: u8 = 255;

// ============================================================================
// Public surface
// ============================================================================

/// Member-level Shamir configuration for one group of a SLIP-0039
/// hierarchical share set.
///
/// - `member_threshold`: shares required to recover the group's share.
/// - `member_count`: total shares emitted for the group.
///
/// Invariant: `1 <= member_threshold <= member_count <= 16`. Additional
/// toolkit policy refuses `member_threshold == 1 && member_count > 1`
/// (matches python `split_ems`'s "use a 1-of-1 group instead" rule);
/// both refusals surface as [`Slip39Error::BadGroupSpec`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GroupSpec {
    pub member_count: u8,
    pub member_threshold: u8,
}

/// Split a master secret into a hierarchical SLIP-0039 share set.
///
/// Per SPEC §2.1 + plan §3.1. Returns `Vec<Vec<Share>>` where the outer
/// vector indexes groups (in input order) and each inner vector holds
/// the group's `member_count` shares.
///
/// - `master_secret`: 16/20/24/28/32 bytes (BIP-39 entropy sizes).
/// - `passphrase`: arbitrary bytes; must be re-supplied at combine time.
/// - `group_threshold`: groups required to reconstruct.
/// - `groups`: per-group member configurations.
/// - `iteration_exponent`: PBKDF2 cost (0..=15; iterations = 10000 · 2^E).
/// - `extendable`: SLIP-0039 ext bit; when true, salt_prefix is empty.
/// - `identifier`: optional 15-bit identifier; derived from `rng` if `None`.
/// - `rng`: cryptographically-secure RNG for identifier (if needed) +
///   random shares + digest random_part.
#[allow(clippy::too_many_arguments)]
pub fn slip39_split<R: CryptoRng + RngCore>(
    master_secret: &[u8],
    passphrase: &[u8],
    group_threshold: u8,
    groups: &[GroupSpec],
    iteration_exponent: u8,
    extendable: bool,
    identifier: Option<u16>,
    rng: &mut R,
) -> Result<Vec<Vec<Share>>, Slip39Error> {
    // ----- Validate -----
    if !VALID_SECRET_LENGTHS.contains(&master_secret.len()) {
        return Err(Slip39Error::BadEntropyByteLength(master_secret.len()));
    }
    if iteration_exponent > 15 {
        return Err(Slip39Error::BadIterationExponent(iteration_exponent));
    }
    let group_count_u = groups.len();
    if group_count_u == 0 || group_count_u > 16 {
        return Err(Slip39Error::BadGroupThreshold {
            got: group_threshold,
            group_count: group_count_u as u8,
        });
    }
    if group_threshold < 1 || group_threshold as usize > group_count_u {
        return Err(Slip39Error::BadGroupThreshold {
            got: group_threshold,
            group_count: group_count_u as u8,
        });
    }
    for (i, g) in groups.iter().enumerate() {
        if g.member_threshold < 1 || g.member_threshold > g.member_count || g.member_count > 16 {
            return Err(Slip39Error::BadGroupSpec {
                group_idx: i,
                n: g.member_count,
                t: g.member_threshold,
            });
        }
        // Python `split_ems` rule: refuse member_threshold=1 with
        // member_count>1; the algorithm replicates the group share to
        // all N members, so the policy is "use a 1-of-1 group instead".
        if g.member_threshold == 1 && g.member_count > 1 {
            return Err(Slip39Error::BadGroupSpec {
                group_idx: i,
                n: g.member_count,
                t: g.member_threshold,
            });
        }
    }

    // ----- Identifier -----
    let identifier = identifier.unwrap_or_else(|| (rng.next_u32() as u16) & 0x7FFF) & 0x7FFF;

    // ----- EMS via Feistel encrypt -----
    let ems = feistel::encrypt(
        master_secret,
        passphrase,
        iteration_exponent,
        identifier,
        extendable,
    );
    // Pin EMS heap pages so the encrypted master cannot be swapped to disk
    // during the (potentially nested) Shamir split below. The pin drops at
    // function exit, after `ems` itself is no longer referenced. v0.14.1:
    // `mlock` is `#[cfg(unix)]` (POSIX syscall; no Windows equivalent in
    // libc-rs); on non-unix the pin is a no-op. The slip39 algorithm
    // itself is platform-uniform — only the swap-protection sidecar is
    // unix-only.
    #[cfg(unix)]
    let _ems_pin = crate::mlock::pin_pages_for(&ems[..]);

    // ----- Group-level Shamir split -----
    let group_shares = split_secret(group_threshold, group_count_u as u8, &ems, rng);

    // ----- Per-group member-level split + Share construction -----
    let mut result: Vec<Vec<Share>> = Vec::with_capacity(group_count_u);
    for (g_idx, group) in groups.iter().enumerate() {
        let (gx, gval) = &group_shares[g_idx];
        let member_shares = split_secret(group.member_threshold, group.member_count, gval, rng);
        let mut group_out: Vec<Share> = Vec::with_capacity(group.member_count as usize);
        for (m_idx, mut mval) in member_shares {
            // Move the Vec<u8> out of the Zeroizing wrapper into the Share.
            // The drained Zeroizing wrapper then drops, scrubbing the empty
            // Vec (no-op); Share's `#[derive(Zeroize, ZeroizeOnDrop)]` on
            // the `value` field takes over Drop-time scrubbing of the
            // moved-out bytes.
            let inner: Vec<u8> = std::mem::take(&mut *mval);
            let share = Share::from_parts(
                inner,
                identifier,
                extendable,
                iteration_exponent,
                *gx,
                group_threshold,
                group_count_u as u8,
                m_idx,
                group.member_threshold,
            );
            group_out.push(share);
        }
        result.push(group_out);
    }

    Ok(result)
}

/// Combine a SLIP-0039 share set back to the master secret.
///
/// Per SPEC §2.1 + plan §3.4. All shares must come from the same master
/// (matching identifier, extendable, iteration_exponent, group_threshold,
/// group_count, and value byte-length); exactly `group_threshold` groups
/// must be represented, each with exactly its `member_threshold` shares
/// at distinct member indices.
pub fn slip39_combine(
    shares: &[Share],
    passphrase: &[u8],
) -> Result<Zeroizing<Vec<u8>>, Slip39Error> {
    // Step 1: empty list refusal.
    if shares.is_empty() {
        return Err(Slip39Error::EmptyShares);
    }

    // Step 3: per-share value-length sanity. Reports input-position
    // share_idx, NOT the parser's 0 (per plan §3.4 + pre-GREEN I2).
    for (idx, s) in shares.iter().enumerate() {
        let len = s.value().len();
        if !VALID_SECRET_LENGTHS.contains(&len) {
            return Err(Slip39Error::InvalidShareValueLength {
                share_idx: idx,
                got: len,
            });
        }
    }

    // Step 4: cross-share consistency. Six invariants per R0 I1.
    let first = &shares[0];
    for s in &shares[1..] {
        if s.identifier != first.identifier {
            return Err(Slip39Error::IdentifierMismatch);
        }
        if s.extendable != first.extendable {
            return Err(Slip39Error::ExtendableMismatch);
        }
        if s.iteration_exponent != first.iteration_exponent {
            return Err(Slip39Error::IterationExponentMismatch);
        }
        if s.group_threshold != first.group_threshold {
            return Err(Slip39Error::GroupThresholdMismatch);
        }
        if s.group_count != first.group_count {
            return Err(Slip39Error::GroupCountMismatch);
        }
        if s.value().len() != first.value().len() {
            return Err(Slip39Error::ShareValueLengthMismatch);
        }
    }

    // Step 5: group by group_index. BTreeMap gives deterministic
    // ordering for error reporting.
    let mut by_group: BTreeMap<u8, Vec<&Share>> = BTreeMap::new();
    for s in shares {
        by_group.entry(s.group_index).or_default().push(s);
    }

    // Per-group: member_threshold uniformity, distinct member indices,
    // exact threshold count. Run group-level Shamir recovery on each.
    let mut group_shares: Vec<(u8, Zeroizing<Vec<u8>>)> = Vec::with_capacity(by_group.len());
    for (&group_idx, gs) in &by_group {
        let mt = gs[0].member_threshold;
        for s in gs.iter().skip(1) {
            if s.member_threshold != mt {
                return Err(Slip39Error::MemberThresholdMismatch);
            }
        }
        // Distinct member indices.
        let mut indices: Vec<u8> = gs.iter().map(|s| s.member_index).collect();
        indices.sort_unstable();
        for w in indices.windows(2) {
            if w[0] == w[1] {
                return Err(Slip39Error::DuplicateMemberIndex {
                    group_idx,
                    member_idx: w[0],
                });
            }
        }
        // Strict-equal count check. Plan §3.4 step 5: BOTH too few AND
        // too many shares trigger InsufficientShares (the algorithm
        // needs == threshold).
        if gs.len() != mt as usize {
            return Err(Slip39Error::InsufficientShares {
                group_idx,
                needed: mt,
                got: gs.len() as u8,
            });
        }
        // Member-level Shamir recovery. Per-share value clones land in
        // Zeroizing<Vec<u8>> so the intermediate share-value copies
        // scrub on Drop, mirroring the Share.value field's own
        // ZeroizeOnDrop discipline.
        let pts: Vec<(u8, Zeroizing<Vec<u8>>)> = gs
            .iter()
            .map(|s| (s.member_index, Zeroizing::new(s.value().to_vec())))
            .collect();
        let gv = recover_secret(mt, &pts)?;
        group_shares.push((group_idx, gv));
    }

    // Group-level threshold check. Strict-equal (same justification as
    // member-level above).
    let gt = first.group_threshold;
    if group_shares.len() != gt as usize {
        return Err(Slip39Error::InsufficientShares {
            group_idx: GROUP_LEVEL_SENTINEL,
            needed: gt,
            got: group_shares.len() as u8,
        });
    }

    // Group-level Shamir recovery → EMS.
    let ems = recover_secret(gt, &group_shares)?;

    // Pin EMS pages so the encrypted master is not swapped to disk during
    // the PBKDF2-heavy Feistel decrypt below. The pin drops at function
    // exit after `master` is moved into the return value. v0.14.1:
    // unix-only (see split-side comment above).
    #[cfg(unix)]
    let _ems_pin = crate::mlock::pin_pages_for(&ems[..]);

    // Feistel decrypt → master secret.
    let master = feistel::decrypt(
        &ems,
        passphrase,
        first.iteration_exponent,
        first.identifier,
        first.extendable,
    );

    Ok(master)
}

// ============================================================================
// Private helpers
// ============================================================================

/// Shamir share generation for a single layer (either group-level over
/// the EMS, or member-level over a group's share value).
///
/// Per plan §3.3 + python `_split_secret` @ 17fcce14:
/// - `threshold == 1`: replicate secret to all `share_count` members.
/// - `threshold >= 2`: generate `T - 2` random shares at indices
///   `0..T-2`; compute `digest = HMAC-SHA256(R, secret)[..4]` where `R`
///   is `len(secret) - 4` random bytes; base shares are
///   `[random_0..T-2, (DIGEST_INDEX, digest || R), (SECRET_INDEX, secret)]`;
///   emit each `0..share_count` share via Lagrange interpolation
///   over the base set.
fn split_secret(
    threshold: u8,
    share_count: u8,
    secret: &[u8],
    rng: &mut (impl CryptoRng + RngCore),
) -> Vec<(u8, Zeroizing<Vec<u8>>)> {
    debug_assert!(
        threshold >= 1 && threshold <= share_count,
        "split_secret invariant"
    );
    debug_assert!(
        VALID_SECRET_LENGTHS.contains(&secret.len()),
        "secret length must be valid"
    );

    if threshold == 1 {
        // Trivial replication. NO digest path; no RNG draws.
        return (0..share_count)
            .map(|i| (i, Zeroizing::new(secret.to_vec())))
            .collect();
    }

    let n = secret.len();
    let random_len = n - DIGEST_LENGTH;

    // T - 2 random shares at indices 0..T-2 — each wrapped in Zeroizing.
    let mut random_shares: Vec<(u8, Zeroizing<Vec<u8>>)> =
        Vec::with_capacity((threshold - 2) as usize);
    for i in 0..(threshold - 2) {
        let mut val = Zeroizing::new(vec![0u8; n]);
        rng.fill_bytes(&mut val);
        random_shares.push((i, val));
    }

    // R = random_len random bytes; digest = HMAC-SHA256(R, secret)[..4].
    let mut r = Zeroizing::new(vec![0u8; random_len]);
    rng.fill_bytes(&mut r);
    let mut mac =
        <Hmac<Sha256> as Mac>::new_from_slice(&r).expect("HMAC-SHA-256 accepts any key length");
    mac.update(secret);
    let digest_full = mac.finalize().into_bytes();

    // digest_payload = digest (4 bytes) || R (n - 4 bytes).
    // R0 C1: order is `digest || R`, NOT `R || digest`.
    let mut digest_payload = Zeroizing::new(Vec::with_capacity(n));
    digest_payload.extend_from_slice(&digest_full[..DIGEST_LENGTH]);
    digest_payload.extend_from_slice(&r);

    // Base shares: random_0..T-2, then (DIGEST_INDEX, digest_payload),
    // then (SECRET_INDEX, secret).
    let mut base_shares: Vec<(u8, Zeroizing<Vec<u8>>)> = random_shares.clone();
    base_shares.push((DIGEST_INDEX, digest_payload));
    base_shares.push((SECRET_INDEX, Zeroizing::new(secret.to_vec())));

    // Emit N shares at indices 0..share_count. For i < T-2 reuse the
    // already-generated random share; for i >= T-2 interpolate over
    // base_shares at x = i.
    let mut out: Vec<(u8, Zeroizing<Vec<u8>>)> = Vec::with_capacity(share_count as usize);
    let base_pts: Vec<(u8, &[u8])> = base_shares
        .iter()
        .map(|(x, v)| (*x, v.as_slice()))
        .collect();
    for i in 0..share_count {
        if i < threshold - 2 {
            out.push(random_shares[i as usize].clone());
        } else {
            let v = Zeroizing::new(lagrange::interpolate_secret_at(&base_pts, i));
            out.push((i, v));
        }
    }
    out
}

/// Shamir recovery for a single layer.
///
/// Per plan §3.5 + python `_recover_secret` @ 17fcce14:
/// - `threshold == 1`: return the single share's value directly (NO
///   digest verification — the digest is not computed in this branch).
/// - `threshold >= 2`: interpolate at `SECRET_INDEX` and `DIGEST_INDEX`,
///   recompute `HMAC-SHA-256(R, recovered_secret)[..4]`, and verify
///   it equals the recovered digest prefix. On mismatch, refuse with
///   [`Slip39Error::DigestVerificationFailed`].
fn recover_secret(
    threshold: u8,
    shares: &[(u8, Zeroizing<Vec<u8>>)],
) -> Result<Zeroizing<Vec<u8>>, Slip39Error> {
    debug_assert!(
        !shares.is_empty(),
        "recover_secret invariant: non-empty shares"
    );

    if threshold == 1 {
        return Ok(shares[0].1.clone());
    }

    let pts: Vec<(u8, &[u8])> = shares.iter().map(|(x, v)| (*x, v.as_slice())).collect();
    let secret = Zeroizing::new(lagrange::interpolate_secret_at(&pts, SECRET_INDEX));
    let digest_payload = Zeroizing::new(lagrange::interpolate_secret_at(&pts, DIGEST_INDEX));

    let (digest, random_part) = digest_payload.split_at(DIGEST_LENGTH);
    let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(random_part)
        .expect("HMAC-SHA-256 accepts any key length");
    mac.update(&secret);
    let computed = mac.finalize().into_bytes();

    if digest != &computed[..DIGEST_LENGTH] {
        return Err(Slip39Error::DigestVerificationFailed);
    }

    Ok(secret)
}

// ============================================================================
// P1c-E.2 R0 C1 — `split_secret(T=2, N=3)` pinned unit test.
//
// Per the R0 architect report and plan §3.3: for T == 2 (the boundary
// case between T == 1 trivial-replication and T >= 3 random-share
// loop), the algorithm constructs `base_shares = [(254, digest_payload),
// (255, secret)]` where `digest_payload = digest (4 bytes) || R (n-4
// bytes)` and `R` is RNG-derived. The N emitted shares are computed
// via `interpolate_secret_at(base_shares, i)` for `i in 0..N`.
//
// This unit test pins three foot-guns simultaneously before the G1
// vectors harness runs:
//   1. random-share-loop bound: for T == 2, the loop must iterate 0
//      times (T - 2 = 0).
//   2. digest_payload byte order: `digest || R`, NOT `R || digest`
//      (verified against python `RawShare(DIGEST_INDEX, digest +
//      random_part)` @ commit 17fcce14).
//   3. emit indices: shares are emitted at `x = 0, 1, ..., N-1`, NOT
//      starting at `x = 1`.
//
// The cross-check uses the LOCKed P1c-E.1 primitives (`lagrange`,
// `hmac::Hmac<sha2::Sha256>`) at the test layer rather than
// pre-computed reference bytes — this isolates the test from RNG-seed
// brittleness while still pinning the structural invariants.
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slip39::lagrange::interpolate_secret_at;
    use hmac::{Hmac, Mac};
    use rand_chacha::ChaCha20Rng;
    use rand_core::SeedableRng;
    use sha2::Sha256;

    #[test]
    fn split_secret_t2_n3_pins_base_share_layout_and_digest_payload_order() {
        let secret = [0xA5u8; 16];
        let mut rng = ChaCha20Rng::seed_from_u64(0xC0FFEE_u64);

        let shares = split_secret(2, 3, &secret, &mut rng);

        // (a) Shape: 3 shares emitted, at x = 0, 1, 2; each share has
        //     the same length as the secret.
        assert_eq!(shares.len(), 3, "T=2 N=3 must emit 3 shares");
        for (i, (x, v)) in shares.iter().enumerate() {
            assert_eq!(
                *x, i as u8,
                "share {i} emitted at x={x}, expected x={i} (emit indices start at 0)",
            );
            assert_eq!(
                v.len(),
                secret.len(),
                "share {i} length {} does not match secret length {}",
                v.len(),
                secret.len(),
            );
        }

        // (b) Recover secret via lagrange interpolation at x = 255
        //     (SECRET_INDEX) from any 2 of the 3 emitted shares. Pins
        //     that the secret IS embedded at x = 255 in the underlying
        //     polynomial (which is the only way the SLIP-39 combine
        //     path can recover it).
        let pts_for_secret: Vec<(u8, &[u8])> = shares
            .iter()
            .take(2)
            .map(|(x, v)| (*x, v.as_slice()))
            .collect();
        let recovered_secret = interpolate_secret_at(&pts_for_secret, 255);
        assert_eq!(
            recovered_secret.as_slice(),
            &secret,
            "interpolation at x=255 from any 2 emitted shares must recover the secret",
        );

        // (c) Recover digest_payload at x = 254 (DIGEST_INDEX) and
        //     pin its byte order: digest_payload[0..4] must equal
        //     HMAC-SHA256(key=digest_payload[4..], msg=secret)[0..4].
        //     A transposed concatenation (R || digest) would fail this
        //     check.
        let digest_payload = interpolate_secret_at(&pts_for_secret, 254);
        assert_eq!(
            digest_payload.len(),
            secret.len(),
            "digest_payload length must match secret length (4-byte digest \
             || (n-4)-byte random_part)",
        );
        let digest_part = &digest_payload[..4];
        let random_part = &digest_payload[4..];
        let mut mac = <Hmac<Sha256> as Mac>::new_from_slice(random_part)
            .expect("Hmac<Sha256> accepts any key length");
        mac.update(&secret);
        let computed = mac.finalize().into_bytes();
        assert_eq!(
            digest_part,
            &computed[..4],
            "digest_payload byte order must be `digest || R`, NOT `R || digest`",
        );
    }

    #[test]
    fn combine_invalid_share_value_length_remaps_share_idx_to_input_position() {
        // Pre-GREEN I2 fold: pin that `slip39_combine`'s per-share
        // value-length check (plan §3.4 step 3) emits the INPUT POSITION
        // as `share_idx`, not the parser's default 0. After the C1 fold
        // (vector #40 now folds to InvalidPadding at parse), no G1
        // fixture vector exercises this combine-time path — this
        // synthetic forged-share test closes the gap and pins the
        // `InvalidShareValueLength` variant's defense-in-depth role.
        //
        // Construct two shares with identical metadata except value
        // length: share[0] has a valid 16-byte value; share[1] has an
        // out-of-set 19-byte value. The per-share check must fire on
        // share[1] BEFORE the cross-share consistency check at step 4
        // (which would otherwise emit `ShareValueLengthMismatch`).
        let share_0 = Share::from_parts(
            vec![0u8; 16],
            /* identifier */ 0x1234,
            /* extendable */ false,
            /* iteration_exponent */ 0,
            /* group_index */ 0,
            /* group_threshold */ 1,
            /* group_count */ 1,
            /* member_index */ 0,
            /* member_threshold */ 1,
        );
        let share_1 = Share::from_parts(vec![0u8; 19], 0x1234, false, 0, 0, 1, 1, 1, 1);
        let err = slip39_combine(&[share_0, share_1], b"").unwrap_err();
        assert_eq!(
            err,
            Slip39Error::InvalidShareValueLength {
                share_idx: 1,
                got: 19,
            },
            "combine must report share_idx=1 (input position), not 0 (parser default)",
        );
    }

    // ========================================================================
    // P1c-E.3 G6 hygiene — mlock-attempts invariant tests.
    //
    // Mirror pattern of `derive::tests::site_3_derive_full_invokes_pin_at_*`
    // and `bip85::tests::format_bip39_phrase_invokes_pin_pages_for`. Asserts
    // that the public SLIP-39 driver functions invoke `pin_pages_for` on at
    // least one secret-bearing buffer (the EMS in both directions) by
    // observing the process-static `attempts_for_test()` counter. The
    // counter increments on every `pin_pages_for` call regardless of mlock
    // success/failure (record_attempt fires before sys_mlock_attempt), so
    // this observer works uniformly across cfg(test) and production builds.
    // ========================================================================

    #[cfg(unix)]
    #[test]
    fn slip39_split_invokes_pin_pages_for_on_ems() {
        let baseline = crate::mlock::attempts_for_test();
        let master = [0xA5u8; 16];
        let mut rng = ChaCha20Rng::seed_from_u64(0x1CE3_5117_u64);
        let shares = slip39_split(
            &master,
            b"",
            1,
            &[GroupSpec {
                member_count: 1,
                member_threshold: 1,
            }],
            0,
            false,
            None,
            &mut rng,
        )
        .expect("1-of-1 split must succeed for 16-byte master");
        assert_eq!(shares.len(), 1, "1 group requested");
        assert_eq!(shares[0].len(), 1, "1 share in the group");
        assert!(
            crate::mlock::attempts_for_test() > baseline,
            "slip39_split must invoke mlock::pin_pages_for on the EMS buffer; \
             attempts counter did not increment from baseline = {baseline}",
        );
    }

    #[cfg(unix)]
    #[test]
    fn slip39_combine_invokes_pin_pages_for() {
        let master = [0xB4u8; 16];
        let mut rng = ChaCha20Rng::seed_from_u64(0x1CE3_C0B1_u64);
        let shares = slip39_split(
            &master,
            b"",
            1,
            &[GroupSpec {
                member_count: 1,
                member_threshold: 1,
            }],
            0,
            false,
            None,
            &mut rng,
        )
        .expect("1-of-1 split must succeed for 16-byte master");
        let flat: Vec<Share> = shares.into_iter().flatten().collect();
        let baseline = crate::mlock::attempts_for_test();
        let recovered = slip39_combine(&flat, b"").expect("1-of-1 combine must succeed");
        assert_eq!(
            &recovered[..],
            &master,
            "1-of-1 round-trip must recover the master",
        );
        assert!(
            crate::mlock::attempts_for_test() > baseline,
            "slip39_combine must invoke mlock::pin_pages_for at least once; \
             attempts counter did not increment from baseline = {baseline}",
        );
    }

    #[test]
    fn split_secret_t1_n5_replicates_secret_without_digest() {
        // T == 1 special case: ALL N shares equal the secret directly.
        // No digest computed; no random bytes consumed beyond what the
        // caller drives (the algorithm doesn't read from the RNG in
        // this branch per python `_split_secret(threshold=1, ...)`).
        let secret = [0xB4u8; 16];
        let mut rng = ChaCha20Rng::seed_from_u64(0xFEEDFACE_u64);

        let shares = split_secret(1, 5, &secret, &mut rng);

        assert_eq!(shares.len(), 5, "T=1 N=5 must emit 5 shares");
        for (i, (x, v)) in shares.iter().enumerate() {
            assert_eq!(*x, i as u8);
            assert_eq!(
                v.as_slice(),
                &secret,
                "T=1 share {i} must be a direct replica of the secret (no digest path)",
            );
        }
    }
}
