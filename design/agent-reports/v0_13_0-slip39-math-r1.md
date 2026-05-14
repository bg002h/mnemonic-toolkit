# v0.13.0 Phase 1a — SLIP-39 math primitives R1 reviewer report

**Phase:** P1a — GF(256) Rijndael field arithmetic + Lagrange interpolation
**Round:** R1 round 1 (clean LOCK)
**Reviewer:** Opus (`feature-dev:code-reviewer`)
**Date:** 2026-05-14
**Commits under review:**
- `919f3a2` (P1a RED: stub + 23 RED tests)
- `e0d21c2` (P1a GREEN: gf256 + lagrange impl + clippy fix)
**Predecessor:** `351db15` (P0 SPEC R0 LOCK)

## Verdict

**0 Critical / 0 Important / 2 Nice-to-have — R1 LOCK round 1.**

Phase 1b (Feistel + PBKDF2 encryption pipeline) cleared to start.

## Scope reviewed

All 12 mandatory reviewer checks per the dispatch:
- Critical: GF(256) reduction poly correctness (0x11b); generator
  correctness (g=3); `mul` correctness across all 256×256 pairs
  (covered by axiom tests + AES Rijndael spot-check); `inv` correctness
  (covered by `mul_by_inv_yields_one` over 255 values); Lagrange formula
  correctness at x*=0 in characteristic 2; per-byte independence in
  multi-byte interpolation.
- Important: `exp[255] = 1` defensive set; constant-time discipline
  (zero-short-circuit acceptable for SLIP-39 threat model); test
  coverage completeness; duplicate-x panic vs SPEC §2.5 contract.

## Key validations

1. **GF(256) table generation correct.** Hand-trace: `x = 1 → 3 → 5 →
   15 → 17 → ...` matches `g^0, g^1, g^2, g^3, g^4 = 1, 3, 5, 15, 17`
   for g=3 under reduction poly 0x11b. Matches Trezor
   `python-shamir-mnemonic`'s precomputed EXP_TABLE byte-for-byte.

2. **`mul` correctness validated transitively.** `mul_by_inv_yields_one`
   asserts `mul(a, inv(a)) == 1` for all 255 non-zero values. Any bad
   entry in the exp/log tables breaks this for many `a`. Combined with
   `distributivity_sample` (a × (b XOR c) = (a×b) XOR (a×c)) the field
   axioms are exercised over a wide value range.

3. **`inv` correctness.** `inv(1) = 1` (verified: log[1]=0, neg_log=0,
   exp[0]=1). `inv(2) = 0x8d` (canonical AES Rijndael inverse; matches
   `mul(2, 0x8d) = 1` which is asserted in `mul_by_inv_yields_one`).

4. **Lagrange formula matches characteristic-2 derivation exactly.**
   - `num = Π_{j≠i} x_j`
   - `den = Π_{j≠i} (x_i XOR x_j)` (XOR = subtraction in char 2)
   - `basis = num / den`
   - `result ^= y_i * basis` (XOR-accumulate)

   Matches `python-shamir-mnemonic/shamir_mnemonic/shamir.py::_interpolate`
   semantically.

5. **Per-byte independence correct.** `interpolate_secret_at_zero`
   destructures `(x, y)` per-share then rebuilds `(x, y[byte_idx])` with
   the x-coordinate shared across byte slots and y varying per slot.

6. **Constant-time discipline acceptable.** `mul` short-circuits on
   `a == 0 || b == 0`. SPEC §2.1 "no branches on field values" refers
   to the value-dependent computation path (log/exp lookups are
   constant-time table indexes). Zero handling is structural and matches
   the Trezor reference impl.

## Nice-to-have findings (non-blocking)

**N1.** `exp[255] = 1` explicit set at `gf256.rs:46-47` is defensive
dead code (the `mul` impl uses `if log_sum >= 255 { log_sum - 255 }
else { log_sum }` which never accesses index 255). Acknowledged in
source comment; defensible as future-proofing if a different `mul` impl
ever needs cyclic access.

**N2.** Clippy `needless_range_loop` fix at `gf256.rs:35` (the GREEN
commit's last edit) correctly transitions to `enumerate().take(255)`
pattern; clippy clean.

## Coverage of acceptance-gate prerequisites

- SPEC §2.1 algorithm contract (Rijndael poly + g=3 + log/exp tables):
  satisfied.
- SPEC §4 G1 (vectors.json round-trip): NOT exercised at this layer —
  deferred to P1c when share encoding lands; the math primitives here
  are necessary-but-not-sufficient (correct Lagrange depends on correct
  GF mul/inv).
- SPEC §4 G6 Cycle A/B discipline: NOT applicable at this layer (no
  secret-bearing locals in the math primitives; bytes flow through
  caller-supplied `&[u8]` and return-by-value `Vec<u8>` that callers
  wrap in `Zeroizing`). The Cycle B pin discipline lands at P1c when
  the caller's `Vec<Zeroizing<...>>` shape is finalized.

## R1 LOCK

v0.13.0 P1a R1 LOCK round 1. Phase 1b (`feistel.rs` 4-round Feistel
network + PBKDF2-HMAC-SHA256 round-key derivation, single-buffer
round-key reuse per SPEC §2.1) cleared to start.
