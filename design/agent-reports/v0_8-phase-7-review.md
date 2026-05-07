# v0.8 Phase 7 Review — BIP-85 DICE only

**Scope:** `bip85.rs` (`format_dice_rolls`), `cmd/derive_child.rs` (dice arm + `--dice-sides`), `tests/cli_derive_child.rs`, `error.rs` (`DeriveChildUnsupportedApp` message), `Cargo.toml` (`sha3 = "0.10"` added). RSA + RSA-GPG deferred per Phase 6 spike (RUSTSEC-2023-0071 unpatched).

**Verdict:** No critical or important findings above threshold. Implementation is correct and ready to ship.

---

## Critical

None.

---

## Important

None above threshold.

---

## Verified-correct items

1. **Bit-extraction.** `bits_per_roll = u32::BITS - (sides - 1).leading_zeros()`. For sides=6: `leading_zeros(5) = 29` → 3 bits. For sides=2: 1 bit. For sides=256: 8 bits, no rejection. For exact powers of 2 the formula gives minimal width with zero wasted trials. Shift `total_bits - bits_per_roll` is always non-negative given `bytes_per_roll = ceil(bits_per_roll / 8)`.

2. **Spec vector pin.** `dice_d6_10_rolls_matches_spec` pins `1,0,0,2,0,1,5,5,2,4` against BIP-85 v1.3.0 §"DICE" using the canonical master xprv. Unit + CLI tests pass.

3. **`sides = u32::MAX` safety.** No overflow in `sides - 1`. `bits_per_roll = 32`, `bytes_per_roll = 4`, `shift = 0`. Rejection sampling rate ≈ 1 in 4 billion — finite. No panic.

4. **`rolls < 1` double-guard.** `format_dice_rolls` returns `Err`; dispatcher returns `DeriveChildLengthOutOfRange`. Belt-and-suspenders.

5. **`cell_7` substring assertion.** Asserts both `"--application <rsa|rsa-gpg> is out-of-scope"` AND `"RUSTSEC-2023-0071"`. Conjunction is stronger than single byte-exact match; intentional rework given Phase 6 deferral.

6. **`--dice-sides` silently ignored for non-dice apps.** Per `DeriveChildArgs` doc-comment ("ignored otherwise"). Not a bug.

7. **BIP-85 path construction.** `derive_entropy(master, 89_101, &[sides, rolls], index)` produces path `m/83696968'/89101'/<sides>'/<rolls>'/<index>'` exactly as BIP-85 v1.3.0 §"DICE" specifies.

8. **SHAKE256 seeding.** `Shake256::default()` → `update(&entropy)` → `finalize_xof()` is the canonical BIP85-DRNG-SHAKE256 construction. 64-byte entropy consumed in one `update` call; no partial-state risk.

9. **Big-endian assembly.** `trial = (trial << 8) | (b as u32)` over `bytes_per_roll` bytes is correct big-endian; no off-by-one. With `bytes_per_roll = 1` reduces to `trial = buf[0] as u32`.

10. **`sha3 = "0.10"` dependency.** Standard RustCrypto crate; no security advisories in force.

11. **Exit code.** `DeriveChildUnsupportedApp` → exit 2. `cell_7` asserts `.code(2)`. Correct.
