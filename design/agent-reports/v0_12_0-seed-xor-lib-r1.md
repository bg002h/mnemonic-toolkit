# v0.12.0 Phase 1 — Seed XOR library R1 reviewer report

**Phase:** P1 — library impl (`crates/mnemonic-toolkit/src/seed_xor.rs`)
**Round:** R1 round 1 (clean LOCK)
**Reviewer:** Opus (`feature-dev:code-reviewer`)
**Date:** 2026-05-14
**Commit under review:** `4fe3a25` (P1 GREEN)
**Predecessor:** `ea63fa6` (P0 SPEC R0 LOCK)

## Verdict

**0 Critical / 0 Important / 2 Nice-to-have — R1 LOCK round 1.**

Phase 2 (CLI surface) cleared to start.

## Scope reviewed

All 12 mandatory reviewer checks per the dispatch:
- Critical: Coldcard algorithm byte-correctness (prefix, share-index format,
  sha256d, slice width, last-share construction); Zeroize discipline; test
  correctness for anchor + property tests.
- Important: SPEC alignment (3 error variants, MIN_SHARES, VALID_ENTROPY_LENGTHS,
  rand_core features, doc-comment cross-refs); single-pass length validation.
- Nice-to-have: regression-catching test structure; rand trait imports.

## Key validations (Coldcard byte-correctness)

Verified line-by-line against `https://github.com/Coldcard/firmware/blob/master/shared/xor_seed.py`:

| Element | Coldcard | Toolkit | Match? |
|---|---|---|---|
| Prefix | `b'Batshitoshi '` (12 bytes incl. trailing space) | `b"Batshitoshi "` at `seed_xor.rs:133` | ✓ |
| Index format | `b'%d of %d parts' % (i, num_parts)`, 0-indexed | `format!("{} of {} parts", i, n_shares)`, `for i in 0..n_shares-1` | ✓ |
| Hash function | `ngu.hash.sha256d` (double-SHA-256) | `bitcoin::hashes::sha256d::Hash::hash` | ✓ |
| Output slice | full secret width (for entropy lengths ≤ 32) | `h.as_byte_array()[..n]` at `seed_xor.rs:138` | ✓ |
| Last share | `xor(raw_secret, *parts)` | Accumulated XOR into `last` (Zeroizing<Vec<u8>>) | ✓ (equivalent) |

The byte-pin anchor test at `lib_seed_xor.rs:196-217` reconstructs share[0] from first principles and asserts byte-equality — genuine regression anchor (would catch e.g., a prefix mutation that dropped the trailing space).

## Zeroize discipline

- `bitcoin::hashes::sha256d::Hash::hash(&buf)` returns by value; post-hash `Zeroize::zeroize(&mut buf)` is safe.
- All secret-bearing locals (`last`, `mask`, `out`) are `Zeroizing<Vec<u8>>` from initialization.
- Return type `Vec<Zeroizing<Vec<u8>>>` zeroizes each share's heap on drop; outer Vec stores only fat pointers (no secret material).
- Stack residue of `sha256d::Hash` newtype is per-iteration and matches the `final_word.rs` precedent.

## Test coverage

- **G1 anchor:** byte-pinned share[0] for `abandon × 12` deterministic split + multi-size round-trip (12/18/24-word).
- **G2 property tests:** 5 sizes × 4 share counts × 100 seeds = 2000 deterministic round-trip pairs (exceeds SPEC §4 G2's ≥100/size requirement).
- **Length-validation refusals:** 5 tests covering all 3 `SeedXorError` variants.
- **Zeroize-discipline type-binding check.**
- **RNG determinism** (same seed → same shares) + **seed-separation** (different seeds → different shares).

## Nice-to-have findings (non-blocking)

**N1.** Anchor test would catch byte-prefix mutations (e.g., dropping the trailing space in `b"Batshitoshi "`) because it reconstructs share[0] from first principles. Solid regression coverage.

**N2.** Test file imports `rand_core::{RngCore, SeedableRng}` — both are the right traits for the `DeterministicRng` ChaCha20-backed CryptoRng wrapper.

## R1 LOCK

v0.12.0 P1 R1 LOCK round 1. Phase 2 (CLI surface + lint anchors) cleared to start.

## SPEC alignment summary

- 3 `SeedXorError` variants match SPEC §A.2.1 enumeration.
- `MIN_SHARES = 2`, `VALID_ENTROPY_LENGTHS = &[16,20,24,28,32]` byte-exact.
- `rand_core = "0.6" features = ["std", "getrandom"]` — sufficient for P2's CLI-boundary `OsRng` construction (verification deferred to P2).
- Library-local error pattern matches v0.11.0 final-word precedent; FOLLOWUP `library-error-and-language-surface-promotion` correctly cited.
