# v0.13.0 Phase 1b — SLIP-39 Feistel + PBKDF2 R1 reviewer report

**Phase:** P1b — 4-round Feistel encryption + PBKDF2-HMAC-SHA-256 round-key derivation
**Round:** R1 round 1 (clean LOCK)
**Reviewer:** Opus (`feature-dev:code-reviewer`)
**Date:** 2026-05-14
**Commits under review:**
- `fea0339` (P1b RED: stub + 19 RED tests + sha2/hmac/pbkdf2 deps)
- `55f04a9` (P1b GREEN: Feistel impl + output-order fix + clippy fix)
**Predecessor:** `1e7f912` (P1a R1 LOCK)

## Verdict

**0 Critical / 0 Important / 1 Nice-to-have — R1 LOCK round 1.**

Phase 1c (RS1024 + wordlist + share encoding + 45-vector harness) cleared to start.

## Scope reviewed

All 12 mandatory reviewer checks per the dispatch:
- Critical: algorithm parity with python-shamir-mnemonic (L/R split, loop body,
  output ordering `R || L`, F function PBKDF2 params, identifier 2-byte BE,
  decrypt reversed-rounds); PBKDF2 iteration formula; round-key single-buffer
  reuse; Zeroize discipline.
- Important: n length validation deferred to P1c; iteration_exponent ≤ 15
  assert; output-ordering bug fix correctness; pbkdf2 dep features minimal;
  password/salt capacity-zero gap; encrypt/decrypt factoring via `feistel_run`.

## Key validations

1. **Algorithm matches python-shamir-mnemonic byte-for-byte.** Verified against
   `shamir_mnemonic/cipher.py` + `shamir_mnemonic/constants.py` at HEAD:
   - L/R split: `master_secret[:n/2] + master_secret[n/2:]` — matches.
   - Loop: `(L, R) = (R, L XOR F(i, R))` — impl uses `l[j] ^= round_key[j]` then `swap(l, r)`, equivalent.
   - Output: `R || L` (halves swapped at output boundary) — matches.
   - F function: `PBKDF2-HMAC-SHA-256(password = [i] || passphrase, salt = b"shamir" || identifier_be(2) || R, iters = (10000 << e) / 4, dkLen = n/2)` — matches line-for-line.
   - Identifier: `identifier.to_be_bytes()` = 2-byte big-endian — matches.
   - decrypt: rounds run in reverse (3, 2, 1, 0) via `feistel_run(reverse=true)` — matches.

2. **Output-order fix verified by hand-trace.** 1-round trace with half=1, L=a, R=b: after the loop `l=b, r=a XOR F`; output `r || l = (a XOR F) || b` — matches python's `r + l`. Round-trip via decrypt with reversed schedule recovers `a || b`. Fix correct.

3. **PBKDF2 iters formula correct.** Per round: `(10000 << e) / 4`. At E=0: 2500/round × 4 rounds = 10000 total. At E=15: 81,920,000/round × 4 = 327,680,000 total. Matches spec.

4. **Round-key single-buffer reuse SPEC-compliant.** Line 90 allocates `round_key` ONCE outside the loop; lines 100-115 refill it via `round_function_into(..., &mut round_key)`. ONE heap allocation per encryption pass, not four. Matches SPEC §2.1 explicit mandate.

5. **Zeroize discipline complete.** `l`, `r`, `round_key`, `out` wrapped in `Zeroizing<Vec<u8>>`. `password` and `salt` are plain `Vec<u8>` but explicitly `Zeroize::zeroize`'d before scope exit. `salt_prefix` is non-secret (deterministic from public identifier).

6. **password/salt capacity gap analyzed (check #9).** Both built via `Vec::with_capacity(exact)` + `extend_from_slice(slice_of_exact_len)`. No reallocation, capacity == len. `zeroize`-on-len == zeroize-on-capacity here. No gap.

7. **n length validation deferred to P1c (check #5).** Module doc explicitly states "Public `slip39_split` / `slip39_combine` at P1c validate inputs before reaching this layer". The current `n >= 16 && n % 2 == 0 && n <= 32` assert is intentionally permissive at the cryptographic-primitive layer; share-encoding-layer adds the strict 5-length whitelist.

8. **pbkdf2 dep features minimal.** `pbkdf2 = "0.12", default-features = false, features = ["hmac"]` — `hmac` feature exposes the generic `pbkdf2::<Hmac<H>>` API. Correct + minimal (no `simple` / `parallel` features pulled in).

9. **Encrypt/decrypt factoring clean.** Both delegate to `feistel_run(reverse: bool)`. Single source of truth.

## Nice-to-have findings (non-blocking)

**N1.** `round_order: Vec<u8>` allocates a tiny 4-byte heap vec per call (could be `[u8; 4]` with conditional reverse, avoiding the alloc). Cosmetic; sub-threshold for blocking. Not flagged in SPEC.

## Byte-anchor coverage deferred

The current 19 tests prove round-trip consistency but NOT byte-equivalence with the python-shamir-mnemonic reference. P1c's `tests/lib_slip39_vectors.rs` harness (45 vectors from `python-shamir-mnemonic/vectors.json`, 15 positive) will exercise the full pipeline (Feistel + share encoding + RS1024) against the authoritative reference — that's the byte-anchor layer.

## R1 LOCK

v0.13.0 P1b R1 LOCK round 1. Phase 1c (RS1024 Reed-Solomon-1024 checksum + 1024-word SLIP-39 wordlist + `Share` struct with bit-packing + parse/render mnemonic encoding + library-local `Slip39Error` + public `slip39_split` / `slip39_combine` entry points + `tests/fixtures/slip39_vectors.json` vendored + `tests/lib_slip39_vectors.rs` exercising all 45 spec vectors) cleared to start.

P1c is the largest sub-phase: ~750 LOC of code + ~400 LOC of tests. RED-first TDD per the plan; vectors.json fixture SHA-pinned against the upstream commit at fetch time.
