//! K-of-N codex32 Shamir share encoding (ms v0.2).
//!
//! A secret (`entr` or `mnem`) splits into N shares, any K of which recombine
//! to the original — using codex32's *native* threshold(k)+index Shamir
//! mechanism, NOT a payload byte (SPEC_ms_v0_2_kofn §1). The codex32 header
//! threshold char is the share-vs-single discriminator; the prefix byte
//! (`0x00`=entr / `0x02`=mnem) remains the payload-KIND discriminator, recovered
//! only on the secret-at-S after interpolation.
//!
//! v0.1/mnem single-strings stay byte-identical: `encode_shares(tag, ZERO, 1, &p)`
//! reduces to the exact `package()`/`encode()` construction (the Phase-0 gate).

use crate::codex32::{Codex32String, Fe};
use crate::consts::{HRP, RESERVED_ID_BLOCKLIST, SHARE_INDEX_V01};
use crate::envelope::{dispatch_payload, extract_wire_fields, payload_wire_bytes, wire_string};
use crate::error::{Error, Result};
use crate::payload::Payload;
use crate::tag::Tag;
use zeroize::Zeroizing;

/// The codex32 bech32 alphabet (32 chars). Index `s` (position 16) is the
/// secret-at-S index — never a distributed-share index.
const CODEX32_ALPHABET: &[u8; 32] = b"qpzry9x8gf2tvdw0s3jn54khce6mua7l";

/// The 31 valid non-`s` share indices, taken from the bech32 alphabet in its
/// own order with `s` removed (deterministic, front-to-back). `n <= 31` is
/// enforced by `encode_shares`, so this pool never runs out.
fn non_s_index_pool() -> Vec<Fe> {
    CODEX32_ALPHABET
        .iter()
        .filter(|&&b| b != b's')
        .map(|&b| Fe::from_char(b as char).expect("alphabet char is a valid Fe"))
        .collect()
}

/// Generate a random 4-char codex32-alphabet `id`, re-rolling while it lands in
/// `RESERVED_ID_BLOCKLIST` (a v0.1 type-tag-shaped value). Uses `getrandom`
/// (0.3.x `getrandom::fill`) — no injected-RNG param (the `mk_codec::encode`
/// precedent).
fn random_id() -> String {
    loop {
        let mut raw = [0u8; 4];
        getrandom::fill(&mut raw).expect("getrandom::fill must not fail");
        let id: [u8; 4] = [
            CODEX32_ALPHABET[(raw[0] & 0x1f) as usize],
            CODEX32_ALPHABET[(raw[1] & 0x1f) as usize],
            CODEX32_ALPHABET[(raw[2] & 0x1f) as usize],
            CODEX32_ALPHABET[(raw[3] & 0x1f) as usize],
        ];
        if !RESERVED_ID_BLOCKLIST.contains(&id) {
            // Every byte is a codex32-alphabet ASCII char → always valid UTF-8.
            return String::from_utf8(id.to_vec()).expect("codex32 alphabet is ASCII");
        }
    }
}

/// A codex32 share threshold.
///
/// `ZERO` is the unshared v0.1 single-string sentinel (codex32 threshold `0`,
/// share-index `s`); `new(k)` accepts a K-of-N share threshold `k in 2..=9`
/// (codex32 `from_seed` accepts threshold `0` or `2..=9` only — `1` is invalid).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Threshold(u8);

impl Threshold {
    /// The unshared single-string sentinel (threshold `0`). A const, NOT
    /// `new(0)` — `new` only admits the K-of-N share range `2..=9`.
    pub const ZERO: Threshold = Threshold(0);

    /// Construct a K-of-N share threshold. `k` MUST be in `2..=9`, else
    /// `Error::InvalidThreshold(k)`.
    pub fn new(k: u8) -> Result<Threshold> {
        if (2..=9).contains(&k) {
            Ok(Threshold(k))
        } else {
            Err(Error::InvalidThreshold(k))
        }
    }

    /// The threshold value (`0` for `ZERO`, `2..=9` for a share threshold).
    pub fn get(self) -> u8 {
        self.0
    }
}

/// Split a secret (`entr` or `mnem`) into `n` codex32 K-of-N shares.
///
/// - `threshold == ZERO`: `n` MUST be 1; returns a single string **byte-identical**
///   to `encode(tag, secret)` — the v0.1 single-string construction
///   (`from_seed(HRP, 0, tag, Fe::S, [prefix]||payload)`, deterministic). The
///   `id` stays the type `tag` (NOT random) — load-bearing for byte-identity.
/// - `threshold == k ∈ 2..=9`: validate `k <= n <= 31` (else `InvalidShareCount`).
///   A random 4-char `id` (not in `RESERVED_ID_BLOCKLIST`) keys the share-set.
///   The secret-at-S (`Fe::S`) holds the real payload; `k-1` random **defining
///   shares** at fixed canonical non-`s` indices + `interpolate_at` for the
///   remaining `n-(k-1)` indices produce the `n` **distributed** shares. The
///   secret-at-S is NEVER returned (it is the recovery target only).
///
/// Works identically for `entr` and `mnem` (byte-agnostic); language survives a
/// `mnem` split (it rides the secret-at-S wire bytes).
pub fn encode_shares(
    tag: Tag,
    threshold: Threshold,
    n: usize,
    secret: &Payload,
) -> Result<Vec<String>> {
    secret.validate()?;
    let bytes = payload_wire_bytes(secret);

    if threshold == Threshold::ZERO {
        // Unshared single-string: must be n==1; byte-identical to encode().
        if n != 1 {
            return Err(Error::InvalidShareCount { k: 0, n });
        }
        let single = Codex32String::from_seed(HRP, 0, tag.as_str(), Fe::S, &bytes[..])?;
        return Ok(vec![single.to_string()]);
    }

    let k = threshold.get();
    let k_usize = k as usize;
    // Bounds (SPEC §1): 2 <= k <= n <= 31 (31 valid non-`s` indices).
    if !(k_usize <= n && n <= 31) {
        return Err(Error::InvalidShareCount { k, n });
    }

    let id = random_id();
    let pool = non_s_index_pool();

    // cycle-15 Lane M (slug #3) — RESOLVED in Cycle-B: codex32 is now VENDORED
    // (crate::codex32, shape A) and `Codex32String` derives `ZeroizeOnDrop`, so
    // the secret-bearing `Codex32String` bindings below (`secret_s`, `defining`)
    // auto-scrub their inner String on drop — no `Zeroizing` wrapper, they own
    // their scrub. The `Vec<u8>` CSPRNG `filler` below also stays `Zeroizing`.
    // The IRREDUCIBLE residue is `distributed: Vec<String>` — it is the function
    // RETURN value (the wire form is `String`), so it MUST outlive the function
    // and cannot be wrapped without changing the public return type. Per the
    // caller-wrap contract (same discipline as `Payload::Entr(Vec<u8>)`,
    // documented in payload.rs + enforced by lint_zeroize_discipline), the
    // CALLER owns the scrub of the returned share strings. The `Codex32String`
    // SOURCE of each `.to_string()` copy IS drop-scrubbed; only the `String`
    // copy handed out is the caller's responsibility. (Honest, not papered over.)
    //
    // 1. secret-at-S carries the real payload at index `s`, threshold `k`.
    let secret_s = Codex32String::from_seed(HRP, k_usize, &id, Fe::S, &bytes[..])?;

    // 2. k-1 random DEFINING shares at the first k-1 pool indices. Each gets a
    //    CSPRNG payload of the SAME byte length as the secret (Zeroizing scrub).
    //    The defining set [secret_s, def_1..def_{k-1}] is k points → fully
    //    determines the Shamir polynomial.
    let mut defining: Vec<Codex32String> = Vec::with_capacity(k_usize);
    defining.push(secret_s);
    for pool_idx in pool.iter().take(k_usize - 1) {
        let mut filler: Zeroizing<Vec<u8>> = Zeroizing::new(vec![0u8; bytes.len()]);
        getrandom::fill(&mut filler[..]).expect("getrandom::fill must not fail");
        let share = Codex32String::from_seed(HRP, k_usize, &id, *pool_idx, &filler[..])?;
        defining.push(share);
    }

    // 3. The n DISTRIBUTED shares: the k-1 defining shares (indices 0..k-1) plus
    //    interpolation-derived shares at the remaining n-(k-1) pool indices.
    //    The secret-at-S (defining[0]) is NEVER distributed.
    let mut distributed: Vec<String> = Vec::with_capacity(n);
    for share in defining.iter().skip(1) {
        distributed.push(share.to_string());
    }
    for pool_idx in pool.iter().take(n).skip(k_usize - 1) {
        let derived = Codex32String::interpolate_at(&defining, *pool_idx)?;
        distributed.push(derived.to_string());
    }

    debug_assert_eq!(distributed.len(), n);
    Ok(distributed)
}

/// Recombine `k` (or more) distributed shares of a K-of-N share-set into the
/// original secret `(Tag, Payload)`.
///
/// Pre-validation runs BEFORE `interpolate_at` because codex32's
/// `interpolate_at` short-circuits when the target index (`s`) is among the
/// inputs (`lib.rs:262`) — bypassing its own payload validation. Order:
/// 1. parse each share (`Error::Codex32` on failure — preserves the
///    within-one-string mixed-case `InvalidCase` rejection), then re-parse the
///    lowercased copy into the CANONICAL vector (BIP-173 uppercase QR form
///    folds to canonical lowercase; codex32's `interpolate_at` does raw
///    case-sensitive cross-share hrp/id compares, so canonicalization here —
///    not field extraction — is what makes an uppercase or mixed-case SET
///    combine, and what lets the index-`s` guard below see `b's'`);
/// 2. **reject any share at index `s`** → `SecretShareSuppliedToCombine` (C1 —
///    the secret-at-S is the recovery target, never a combine input);
/// 3. `shares.len() >= k` (the first share's threshold) else surface
///    `ThresholdNotPassed`;
/// 4. distinct share indices else `RepeatedIndex` (codex32's own check is lazy);
/// 5. recover the secret-at-S from EXACTLY the first `k` shares (which define
///    the polynomial) via `interpolate_at(&parsed[..k], Fe::S)` (surfaces
///    `Mismatched{Hrp,Id,Threshold,Length}` on a header-inconsistent k-set),
///    then verify every EXTRA supplied share lies on that same polynomial
///    (`interpolate_at(k_set, idx)` re-derived value must equal the supplied
///    share) → `InconsistentShareSet` on any mismatch. (M6 — codex32 K-of-N
///    carries no digest share; a same-id but cross-polynomial set previously
///    combined to a SILENT WRONG secret. A valid exactly-k or n>k all-consistent
///    combine is bit-identical to the prior all-shares interpolation.)
///
/// Returns **`(Tag::ENTR, …)`** always: the recovered secret-at-S carries the
/// share-set's RANDOM `id` (NOT a type tag); the payload KIND is the prefix byte
/// (via `dispatch_payload`), so the random id is discarded. (We do NOT route
/// through `discriminate` — it would rebuild a `Tag` from the random id.)
pub fn combine_shares(shares: &[String]) -> Result<(Tag, Payload)> {
    // 1. Parse each share (map codex32 parse/checksum failure via Error::Codex32).
    let parsed: Vec<Codex32String> = shares
        .iter()
        .map(|s| Codex32String::from_string(s.clone()).map_err(Error::Codex32))
        .collect::<Result<Vec<_>>>()?;

    // 1b. Canonicalize: re-parse each share's lowercased wire copy (NEVER
    //     lowercase before the first parse above — that would launder the
    //     within-one-string mixed-case `InvalidCase` rejection). codex32's
    //     checksum engine case-folds, so this re-parse is infallible in
    //     practice (probe-proven byte-identical for lowercase input); still
    //     route the Result via `?`. The canonical vector feeds both the field
    //     extraction below AND `interpolate_at` (whose raw case-sensitive
    //     cross-share hrp/id compares are why extraction-side lowercasing
    //     alone cannot fix combine) — it also makes the recovered output
    //     lowercase.
    let parsed: Vec<Codex32String> = parsed
        .iter()
        .map(|c| {
            Codex32String::from_string(c.to_string().to_ascii_lowercase()).map_err(Error::Codex32)
        })
        .collect::<Result<Vec<_>>>()?;

    if parsed.is_empty() {
        // No shares → surface as below-threshold (k unknown; report 1/0).
        return Err(Error::Codex32(crate::codex32::Error::ThresholdNotPassed {
            threshold: 1,
            n_shares: 0,
        }));
    }

    // Re-parse wire fields for each → (threshold_byte, share_index_byte). Both
    // are `u8` (Copy), so this owns nothing that borrows the per-share string.
    // `wire_string` is subsumed by the canonical vector above (already
    // lowercase) — kept as harmless defense-in-depth; the canonical vector is
    // the load-bearing mechanism for combine.
    let fields: Vec<(u8, u8)> = parsed
        .iter()
        .map(|c| {
            let s = wire_string(c);
            extract_wire_fields(&s).map(|f| (f.threshold_byte, f.share_index_byte))
        })
        .collect::<Result<Vec<_>>>()?;

    // 2. C1: reject any input at index `s` BEFORE interpolate_at (the
    //    short-circuit at codex32 lib.rs:262 would otherwise bypass validation).
    if fields.iter().any(|&(_, idx)| idx == SHARE_INDEX_V01) {
        return Err(Error::SecretShareSuppliedToCombine);
    }

    // 3. count >= k (the first share's threshold char). codex32 thresholds are
    //    single ASCII digits ('2'..'9'); '0' (an unshared single) here means the
    //    caller passed a v0.1 single-string into combine — also below any share
    //    threshold, surfaced as ThresholdNotPassed.
    let k = (fields[0].0 - b'0') as usize;
    if parsed.len() < k {
        return Err(Error::Codex32(crate::codex32::Error::ThresholdNotPassed {
            threshold: k,
            n_shares: parsed.len(),
        }));
    }

    // 4. distinct share indices (codex32's RepeatedIndex check is lazy — only
    //    fires for the i==j Lagrange term — so pre-check exhaustively).
    for i in 0..fields.len() {
        for j in (i + 1)..fields.len() {
            if fields[i].1 == fields[j].1 {
                let idx = Fe::from_char(fields[i].1 as char).map_err(Error::Codex32)?;
                return Err(Error::Codex32(crate::codex32::Error::RepeatedIndex(idx)));
            }
        }
    }

    // 5. Recover the secret-at-S from EXACTLY k shares, then verify every
    //    EXTRA supplied share lies on that same polynomial (M6 — beyond-BIP-93
    //    defense-in-depth: codex32 K-of-N carries no digest share, so a same-id
    //    [same hrp/id/threshold/length] but cross-polynomial set would otherwise
    //    interpolate to a SILENT WRONG secret). The first k shares define the
    //    polynomial; recovery surfaces Mismatched{Hrp,Id,Threshold,Length} via
    //    Error::Codex32 on a header-inconsistent k-set, exactly as before.
    //
    //    Hard invariant (BRAINSTORM §6.0): a valid exactly-k combine is
    //    bit-identical to the prior `interpolate_at(&parsed, Fe::S)` (k == n →
    //    k_set == parsed, empty membership loop), and a valid n>k all-consistent
    //    combine recovers the same secret (every extra lies on the curve).
    let k_set = &parsed[..k];
    let secret = Codex32String::interpolate_at(k_set, Fe::S).map_err(Error::Codex32)?;

    // For each EXTRA supplied share, re-derive the polynomial's value at that
    // share's index from the k-set and require it to equal the supplied share
    // (full canonical lowercased Codex32String compare — header fields are
    // already cross-checked by interpolate_at; this adds the polynomial/data
    // dimension). The share-index char comes from the already-extracted `fields`
    // (codex32's `Parts::share_index` is private); reuse the same `Fe::from_char`
    // conversion as the distinct-index check above. Any mismatch ⇒ the set is
    // not all from one split.
    for j in k..parsed.len() {
        let idx = Fe::from_char(fields[j].1 as char).map_err(Error::Codex32)?;
        let derived = Codex32String::interpolate_at(k_set, idx).map_err(Error::Codex32)?;
        if derived != parsed[j] {
            return Err(Error::InconsistentShareSet);
        }
    }

    // Payload KIND is the recovered prefix byte; the id is random → discard it
    // and always return Tag::ENTR (the kind lives in the Payload, NOT the tag).
    //
    // cycle-15 Lane M (slug #3) — RESOLVED in Cycle-B: `parsed`/`k_set` and the
    // recovered `secret` are `Codex32String`, which now derives `ZeroizeOnDrop`
    // (vendored codex32, shape A) — each scrubs its inner String when the `Vec`
    // / binding drops at fn return. The recovered secret WIRE BYTES are also
    // scrubbed below via the `Zeroizing<Vec<u8>>` wrap (belt-and-suspenders).
    let data: Zeroizing<Vec<u8>> = Zeroizing::new(secret.parts().data());
    let payload = dispatch_payload(&data)?;
    Ok((Tag::ENTR, payload))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_accepts_2_through_9() {
        for k in 2u8..=9 {
            let t =
                Threshold::new(k).unwrap_or_else(|e| panic!("new({k}) should be Ok, got {e:?}"));
            assert_eq!(t.get(), k);
        }
    }

    #[test]
    fn new_rejects_zero() {
        assert!(matches!(Threshold::new(0), Err(Error::InvalidThreshold(0))));
    }

    #[test]
    fn new_rejects_one() {
        assert!(matches!(Threshold::new(1), Err(Error::InvalidThreshold(1))));
    }

    #[test]
    fn new_rejects_ten() {
        assert!(matches!(
            Threshold::new(10),
            Err(Error::InvalidThreshold(10))
        ));
    }

    #[test]
    fn zero_const_get_is_zero() {
        assert_eq!(Threshold::ZERO.get(), 0);
    }

    #[test]
    fn new_five_get_is_five() {
        assert_eq!(Threshold::new(5).unwrap().get(), 5);
    }

    // --- encode_shares tests (Task 1.3) ---

    use crate::codex32::{Codex32String, Fe};
    use crate::consts::RESERVED_PREFIX;
    use crate::encode::encode;
    use crate::payload::Payload;
    use crate::tag::Tag;

    fn entr_p() -> Payload {
        Payload::Entr(vec![0xCDu8; 16])
    }
    fn mnem_p() -> Payload {
        Payload::Mnem {
            language: 1,
            entropy: vec![0xCDu8; 16],
        }
    }

    /// Re-parse a share string and return (threshold_char, share_index_char, id).
    fn share_header(s: &str) -> (char, char, String) {
        let sep = s.rfind('1').unwrap();
        let b = s.as_bytes();
        let threshold = b[sep + 1] as char;
        let id: String = s[sep + 2..sep + 6].to_string();
        let index = b[sep + 6] as char;
        (threshold, index, id)
    }

    #[test]
    fn zero_share_is_byte_identical_to_encode_entr() {
        let p = entr_p();
        let shares = encode_shares(Tag::ENTR, Threshold::ZERO, 1, &p).unwrap();
        assert_eq!(shares, vec![encode(Tag::ENTR, &p).unwrap()]);
    }

    #[test]
    fn zero_share_is_byte_identical_to_encode_mnem() {
        let p = mnem_p();
        let shares = encode_shares(Tag::ENTR, Threshold::ZERO, 1, &p).unwrap();
        assert_eq!(shares, vec![encode(Tag::ENTR, &p).unwrap()]);
    }

    #[test]
    fn zero_share_requires_n_eq_1() {
        let p = entr_p();
        assert!(matches!(
            encode_shares(Tag::ENTR, Threshold::ZERO, 2, &p),
            Err(Error::InvalidShareCount { k: 0, n: 2 })
        ));
    }

    #[test]
    fn encode_shares_2_of_3_shape() {
        let p = entr_p();
        let shares = encode_shares(Tag::ENTR, Threshold::new(2).unwrap(), 3, &p).unwrap();
        assert_eq!(shares.len(), 3);
        // Each parses, threshold char '2', distinct non-`s` indices, same id.
        let mut indices = Vec::new();
        let mut ids = Vec::new();
        for s in &shares {
            Codex32String::from_string(s.clone()).expect("each share must parse");
            let (thr, idx, id) = share_header(s);
            assert_eq!(thr, '2', "threshold char");
            assert_ne!(idx, 's', "distributed share must not be index s");
            indices.push(idx);
            ids.push(id);
        }
        // Distinct indices.
        let mut sorted = indices.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), indices.len(), "indices must be distinct");
        // Same id across the set.
        assert!(ids.windows(2).all(|w| w[0] == w[1]), "id must be shared");
    }

    #[test]
    fn encode_shares_rejects_n_below_k() {
        let p = entr_p();
        assert!(matches!(
            encode_shares(Tag::ENTR, Threshold::new(2).unwrap(), 1, &p),
            Err(Error::InvalidShareCount { k: 2, n: 1 })
        ));
    }

    #[test]
    fn encode_shares_rejects_n_32() {
        let p = entr_p();
        assert!(matches!(
            encode_shares(Tag::ENTR, Threshold::new(2).unwrap(), 32, &p),
            Err(Error::InvalidShareCount { k: 2, n: 32 })
        ));
    }

    #[test]
    fn encode_shares_id_not_in_blocklist() {
        // Statistical: across many splits, the random id never lands in the blocklist.
        let p = entr_p();
        for _ in 0..64 {
            let shares = encode_shares(Tag::ENTR, Threshold::new(2).unwrap(), 2, &p).unwrap();
            let (_, _, id) = share_header(&shares[0]);
            let id_bytes: [u8; 4] = id.as_bytes().try_into().unwrap();
            assert!(
                !crate::consts::RESERVED_ID_BLOCKLIST.contains(&id_bytes),
                "id {id:?} must not be in RESERVED_ID_BLOCKLIST"
            );
        }
    }

    /// Inline round-trip (combine_shares lands in Task 1.4): any k of the n
    /// distributed shares, interpolated at S, recover the secret wire bytes.
    #[test]
    fn encode_shares_round_trip_via_interpolate_entr_and_mnem() {
        for p in [entr_p(), mnem_p()] {
            let secret_wire = crate::envelope::payload_wire_bytes(&p);
            for k in 2u8..=9 {
                let n = (k as usize) + 2; // exercise interpolation-derived shares
                let shares = encode_shares(Tag::ENTR, Threshold::new(k).unwrap(), n, &p).unwrap();
                assert_eq!(shares.len(), n);
                let parsed: Vec<Codex32String> = shares
                    .iter()
                    .map(|s| Codex32String::from_string(s.clone()).unwrap())
                    .collect();
                // First k and last k subsets both recover the secret.
                for subset in [&parsed[..k as usize], &parsed[n - k as usize..]] {
                    let recovered = Codex32String::interpolate_at(subset, Fe::S).unwrap();
                    assert_eq!(
                        recovered.parts().data(),
                        secret_wire[..],
                        "k={k} n={n} kind={:?} must recover secret wire bytes",
                        p.kind()
                    );
                }
            }
        }
    }

    // --- combine_shares tests (Task 1.4) ---

    #[test]
    fn combine_round_trip_entr_and_mnem_all_lengths() {
        for ent_len in [16usize, 20, 24, 28, 32] {
            let entr = Payload::Entr(vec![0x37u8; ent_len]);
            let mnem = Payload::Mnem {
                language: 7,
                entropy: vec![0x91u8; ent_len],
            };
            for p in [entr, mnem] {
                for k in 2u8..=9 {
                    let n = (k as usize) + 1;
                    let shares =
                        encode_shares(Tag::ENTR, Threshold::new(k).unwrap(), n, &p).unwrap();
                    // First k and last k subsets both combine back to the secret.
                    for subset in [&shares[..k as usize], &shares[n - k as usize..]] {
                        let (tag, recovered) = combine_shares(subset).unwrap();
                        assert_eq!(tag, Tag::ENTR, "combine always returns Tag::ENTR");
                        assert_eq!(
                            recovered, p,
                            "k={k} n={n} ent_len={ent_len} must recover the exact payload"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn combine_rejects_below_threshold() {
        let p = entr_p();
        let shares = encode_shares(Tag::ENTR, Threshold::new(3).unwrap(), 4, &p).unwrap();
        // Only 2 of a 3-of-4 set.
        let err = combine_shares(&shares[..2]).unwrap_err();
        assert!(
            matches!(
                err,
                Error::Codex32(crate::codex32::Error::ThresholdNotPassed { .. })
            ),
            "expected ThresholdNotPassed, got {err:?}"
        );
    }

    #[test]
    fn combine_rejects_duplicate_index() {
        let p = entr_p();
        let shares = encode_shares(Tag::ENTR, Threshold::new(2).unwrap(), 3, &p).unwrap();
        // Same share twice → duplicate index.
        let dup = vec![shares[0].clone(), shares[0].clone()];
        assert!(matches!(
            combine_shares(&dup),
            Err(Error::Codex32(crate::codex32::Error::RepeatedIndex(_)))
        ));
    }

    #[test]
    fn combine_rejects_secret_share_index_s() {
        // Hand-build the secret-at-S directly (index `s`, threshold 2). It must
        // be rejected BEFORE interpolate_at (C1 — the short-circuit would
        // otherwise bypass payload validation).
        let bytes = crate::envelope::payload_wire_bytes(&entr_p());
        let secret_s = Codex32String::from_seed(HRP, 2, "tst7", Fe::S, &bytes[..])
            .unwrap()
            .to_string();
        // Need >= k shares to get past the count check and reach the index check;
        // but the index-s check runs first regardless, so a single secret-s input
        // is rejected on the index axis.
        let p = entr_p();
        let shares = encode_shares(Tag::ENTR, Threshold::new(2).unwrap(), 2, &p).unwrap();
        let with_secret = vec![secret_s, shares[0].clone()];
        assert!(matches!(
            combine_shares(&with_secret),
            Err(Error::SecretShareSuppliedToCombine)
        ));
    }

    #[test]
    fn combine_rejects_mismatched_threshold() {
        // Two shares from different-threshold sets, at DISTINCT indices (so the
        // distinct-index pre-check passes and interpolate_at's eager
        // MismatchedThreshold check fires). set2[0]=index q; set3[1]=index p.
        let p = entr_p();
        let set2 = encode_shares(Tag::ENTR, Threshold::new(2).unwrap(), 2, &p).unwrap();
        let set3 = encode_shares(Tag::ENTR, Threshold::new(3).unwrap(), 3, &p).unwrap();
        let mixed = vec![set2[0].clone(), set3[1].clone()];
        let err = combine_shares(&mixed).unwrap_err();
        assert!(
            matches!(
                err,
                Error::Codex32(crate::codex32::Error::MismatchedThreshold(..))
            ),
            "expected MismatchedThreshold, got {err:?}"
        );
    }

    #[test]
    fn combine_rejects_unparseable() {
        let bad = vec!["not-an-ms1-string".to_string(), "also-bad".to_string()];
        assert!(matches!(combine_shares(&bad), Err(Error::Codex32(_))));
    }

    // --- audit I9: combine must REJECT (not panic on) a non-standard-length
    // Entr share set. The encode path validates length up front, but codex32
    // share strings are an open format — an externally-constructed valid-checksum
    // set with a non-standard payload length must surface a clean error, not abort.

    /// Build a valid-checksum K-of-N Entr share set whose recovered payload has a
    /// NON-STANDARD entropy length, bypassing `encode_shares`' `secret.validate()`
    /// guard (which would reject it). Mirrors `encode_shares`' codex32
    /// construction with a fixed id for determinism.
    fn nonstandard_entr_distributed(k: usize, n: usize, entropy_len: usize) -> Vec<String> {
        // wire payload = [RESERVED_PREFIX] || entropy
        let mut bytes = vec![RESERVED_PREFIX];
        bytes.extend(std::iter::repeat(0xCDu8).take(entropy_len));
        let id = "tst7";
        let secret_s = Codex32String::from_seed(HRP, k, id, Fe::S, &bytes[..]).unwrap();
        let pool = non_s_index_pool();
        let mut defining = vec![secret_s];
        for pidx in pool.iter().take(k - 1) {
            let filler = vec![0u8; bytes.len()];
            defining.push(Codex32String::from_seed(HRP, k, id, *pidx, &filler[..]).unwrap());
        }
        let mut out = Vec::new();
        for s in defining.iter().skip(1) {
            out.push(s.to_string());
        }
        for pidx in pool.iter().take(n).skip(k - 1) {
            out.push(
                Codex32String::interpolate_at(&defining, *pidx)
                    .unwrap()
                    .to_string(),
            );
        }
        out
    }

    #[test]
    fn combine_rejects_nonstandard_entr_length_not_panics() {
        // 17-byte entropy ∉ VALID_ENTR_LENGTHS. Pre-fix `combine_shares` returned
        // Ok(unvalidated Entr) and `ms combine`'s from_entropy_in panicked
        // (exit 101). Post-fix: a clean PayloadLengthMismatch, no panic.
        let shares = nonstandard_entr_distributed(2, 2, 17);
        let res = combine_shares(&shares);
        assert!(
            matches!(res, Err(Error::PayloadLengthMismatch { got: 17, .. })),
            "expected PayloadLengthMismatch{{got:17}}, got {res:?}"
        );
    }

    #[test]
    fn dispatch_payload_validates_entr_length() {
        // Unit-level: the Entr arm now validates length (parity with the Mnem arm
        // and this fn's doc contract). Audit I9.
        let mut bad = vec![RESERVED_PREFIX];
        bad.extend(std::iter::repeat(0xCDu8).take(17));
        assert!(
            matches!(
                dispatch_payload(&bad),
                Err(Error::PayloadLengthMismatch { got: 17, .. })
            ),
            "non-standard Entr length must Err"
        );
        // Positive control: a standard length (16) still decodes Ok — no over-rejection.
        let mut good = vec![RESERVED_PREFIX];
        good.extend(std::iter::repeat(0xCDu8).take(16));
        assert!(
            matches!(dispatch_payload(&good), Ok(Payload::Entr(_))),
            "standard Entr length must Ok"
        );
    }

    // --- M6: cross-share polynomial-consistency check in combine_shares ---
    //
    // Beyond-BIP-93 defense-in-depth (BRAINSTORM §6.0): codex32 K-of-N has no
    // digest share, so combining a same-id (same hrp/id/threshold/length) but
    // DIFFERENT-polynomial share set silently returns a WRONG secret. The check
    // truncates to the first k shares (which define the polynomial), recovers
    // the secret from them, then verifies every EXTRA supplied share lies on
    // that polynomial. Valid combines (exactly-k, or n>k all-consistent) MUST
    // stay bit-identical.

    /// Build a valid-checksum 2-of-`n` distributed share set carrying a STANDARD
    /// 16-byte Entr secret, with a CALLER-FIXED `id` and a caller-chosen secret
    /// entropy byte (→ a distinct Shamir polynomial). Two sets with the same
    /// `id` but different `secret_byte` are same-id-but-inconsistent: their
    /// shares pairwise lie on DIFFERENT polynomials. Mirrors `encode_shares`'
    /// codex32 construction (deterministic filler, no CSPRNG → reproducible).
    fn same_id_2_of_n(id: &str, secret_byte: u8, filler_byte: u8, n: usize) -> Vec<String> {
        let k = 2usize;
        // wire payload = [RESERVED_PREFIX] || 16-byte entropy (a STANDARD length,
        // so a clean combine recovers a valid Entr payload).
        let mut bytes = vec![RESERVED_PREFIX];
        bytes.extend(std::iter::repeat(secret_byte).take(16));
        let secret_s = Codex32String::from_seed(HRP, k, id, Fe::S, &bytes[..]).unwrap();
        let pool = non_s_index_pool();
        let mut defining = vec![secret_s];
        for pidx in pool.iter().take(k - 1) {
            let filler = vec![filler_byte; bytes.len()];
            defining.push(Codex32String::from_seed(HRP, k, id, *pidx, &filler[..]).unwrap());
        }
        let mut out = Vec::new();
        for s in defining.iter().skip(1) {
            out.push(s.to_string());
        }
        for pidx in pool.iter().take(n).skip(k - 1) {
            out.push(
                Codex32String::interpolate_at(&defining, *pidx)
                    .unwrap()
                    .to_string(),
            );
        }
        out
    }

    #[test]
    fn combine_inconsistent_same_id_set_rejected() {
        // Two DIFFERENT secrets A, B split 2-of-3 with the SAME id/threshold/
        // length. Supply an over-threshold (n>k) same-id set [A1, A2, B3]:
        // distinct indices, same header, but B3 is NOT on A's polynomial. RED
        // today: combine interpolates over all three and returns a WRONG
        // (garbage) secret with no error. Post-fix: the membership check derives
        // A's value at B3's index from {A1,A2} and finds it ≠ B3 →
        // Error::InconsistentShareSet. (BRAINSTORM §6.5 test #1, n>k extras form.)
        //
        // NOTE the spec's documented irreducible limit (§6.2 edge cases): an
        // EXACTLY-k mixed pair [A1, B2] is NOT detectable — any k points define
        // *a* polynomial, so there is no extra share to cross-check. M6 closes
        // only the detectable case (any over-threshold set not all-on-one-curve).
        let set_a = same_id_2_of_n("aaaa", 0x11, 0x22, 3);
        let set_b = same_id_2_of_n("aaaa", 0x33, 0x44, 3);
        // A's first two distributed shares (the consistent k-set) + B's third.
        let mixed = vec![set_a[0].clone(), set_a[1].clone(), set_b[2].clone()];
        let res = combine_shares(&mixed);
        assert!(
            matches!(res, Err(Error::InconsistentShareSet)),
            "expected InconsistentShareSet for a same-id mixed-polynomial set, got {res:?}"
        );
    }

    #[test]
    fn combine_valid_exactly_k_unchanged() {
        // Positive control (BRAINSTORM §6.0 hard invariant): a clean 2-of-3,
        // supply exactly k=2 consistent shares → recovers the correct secret A,
        // byte-identical to the current behavior. MUST stay GREEN.
        let p = Payload::Entr(vec![0xCDu8; 16]);
        let shares = encode_shares(Tag::ENTR, Threshold::new(2).unwrap(), 3, &p).unwrap();
        let (tag, recovered) = combine_shares(&shares[..2]).unwrap();
        assert_eq!(tag, Tag::ENTR);
        assert_eq!(
            recovered, p,
            "exactly-k combine must recover the exact payload"
        );
    }

    #[test]
    fn combine_valid_n_gt_k_all_consistent() {
        // Positive control: supply all 3 consistent shares of A (n > k) → the
        // extra share passes the membership check → recovers A unchanged. MUST
        // stay GREEN (no regression on the over-supplied legitimate case).
        let p = Payload::Entr(vec![0xCDu8; 16]);
        let shares = encode_shares(Tag::ENTR, Threshold::new(2).unwrap(), 3, &p).unwrap();
        let (tag, recovered) = combine_shares(&shares).unwrap();
        assert_eq!(tag, Tag::ENTR);
        assert_eq!(
            recovered, p,
            "n>k all-consistent combine must recover the exact payload"
        );
    }

    #[test]
    fn combine_inconsistent_extra_share_rejected() {
        // 2 consistent A-shares (the k-set) + a consistent A-extra + a same-id
        // B-extra, with the INCONSISTENT extra in a NON-terminal position
        // [A1, A2, B3, A4]: the first k recover A and the membership loop must
        // catch the B-share even though it is not the last extra. RED today
        // (combine interpolates over all 4 → garbage). Post-fix:
        // Error::InconsistentShareSet.
        // id chars must be in the codex32 (bech32) alphabet — 'b'/'i'/'o'/'1'
        // are excluded, so use 'cqcq'.
        let set_a = same_id_2_of_n("cqcq", 0x55, 0x66, 4);
        let set_b = same_id_2_of_n("cqcq", 0x77, 0x88, 4);
        // k-set [A1, A2] (pool indices 0,1) + B's index-2 share (inconsistent,
        // a non-terminal extra) + A's index-3 share (consistent, terminal).
        let mixed = vec![
            set_a[0].clone(),
            set_a[1].clone(),
            set_b[2].clone(),
            set_a[3].clone(),
        ];
        let res = combine_shares(&mixed);
        assert!(
            matches!(res, Err(Error::InconsistentShareSet)),
            "expected InconsistentShareSet for a consistent-k + inconsistent-extra set, got {res:?}"
        );
    }
}
