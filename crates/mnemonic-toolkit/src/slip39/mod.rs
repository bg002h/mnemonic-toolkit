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
        let pts_for_secret: Vec<(u8, &[u8])> =
            shares.iter().take(2).map(|(x, v)| (*x, v.as_slice())).collect();
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
        let share_1 = Share::from_parts(
            vec![0u8; 19],
            0x1234,
            false,
            0,
            0,
            1,
            1,
            1,
            1,
        );
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
