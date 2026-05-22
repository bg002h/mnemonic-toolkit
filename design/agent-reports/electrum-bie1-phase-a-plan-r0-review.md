# Electrum BIE1 Phase-A plan-doc — opus R0 + R1 review (verbatim)

Reviews of `design/PLAN_electrum_bie1_storage.md` (Cycle 19, feature-dev:code-reviewer, opus). Persisted per CLAUDE.md ("persist review-agent output verbatim before fold-and-commit"). All findings folded into the plan-doc; R1 GREEN.

---

## R0 — VERDICT: YELLOW (0 Critical / 4 Important / 3 Minor)

Crypto core verified correct against pinned `spesmilo/electrum @ 2e640c83` and in-tree source: endianness, slice offsets, HMAC-before-decrypt ordering, AES-128, the pw123 scalar, layered-helper shape, dep choices all check out. No silent wrong-key or oracle bug. YELLOW = spec gaps + one factual error, no crypto error.

**I1 — Storage-layer verification gap; wiring round-trip can mask a zlib-direction bug.** The 3 Electrum KATs test `ecies_decrypt_message` only (raw plaintexts, not zlib). `ecies_decrypt_storage`'s zlib leg was guarded only by a self-authored round-trip (self-consistent even with wrong wbits). Fix: pin the §3 zlib hex as Python-stdlib `zlib.compress` output (the `789c` header = zlib-wrapped, what Electrum emits) → true cross-impl oracle; round-trip becomes wiring-coverage; defer a full vendored Electrum storage-file fixture to Phase B.

**I2 — Factual: "secp256k1 0.29" vs `bitcoin = "0.32"`.** secp256k1 is NOT a direct dep; `bitcoin` 0.32.8 re-exports secp256k1 0.29.1 (Cargo.lock:893). Fix: `use bitcoin::secp256k1::{...}`; `secp256k1::…` at crate root won't compile.

**I3 — `mul_tweak` Err mapping + zero/range edges.** `Scalar::from_be_bytes` rejects ≥ n (unreachable post-reduction); the real edge is scalar==0 (caught by `InvalidScalar` before `mul_tweak`). `mul_tweak` Err must NOT map to `InvalidEphemeralPubkey` (that's for `PublicKey::from_slice`). Fix: prove `mul_tweak` cannot fail (valid point × nonzero scalar < n ≠ infinity) and `.expect()` with the proof-comment.

**I4 — `sha2::Sha512` already available** (`bsms_crypto.rs:51`); §4 should list it under no-new-feature so the implementer doesn't add a redundant feature.

**M1** — `crypto-bigint` is the right call over `num-bigint`/`k256` (reject k256 — duplicates secp256k1). Order constant zero-LEFT-padded to 64B for U512; low 32 BE bytes. KAT to pw123 scalar.
**M2** — Error unification (`HmacMismatch | AesDecryptFailure` → one message) is a Phase-B CLI-boundary concern; Phase A keeps variants distinct (KATs assert on them); a pure library is not an oracle. State explicitly. Variants correctly alphabetical.
**M3** — Also `Zeroizing` the 64-byte PBKDF2 output and the compressed ecdh point bytes feeding sha512.

Scope: Phase A (library + KATs + deps, no CLI/sniff/version/tag) is a clean independently-testable cut; only the I1 zlib-oracle belongs in A; `looks_like_bie1`/`flag_is_secret`/GUI/manual stay in B.

---

## R1 — VERDICT: GREEN

All seven folds present and consistent. I1 (§3 + §6.4), I2 (§5), I3 (§5), I4 (§4), M1 (§4), M2 (§5), M3 (§7) resolved.

**I3 `.expect()` proof confirmed sound:** secp256k1 is prime-order `n`; every non-identity point has order exactly `n`; for `P ≠ O`, `s·P = O` iff `s ≡ 0 (mod n)`, impossible for `s ∈ [1, n-1]` (zero rejected by `InvalidScalar`; `< n` by reduction). `from_slice` success gives `P ≠ O`. `.expect()` unreachable.

**No fold contradicts another:** I3's `InvalidEphemeralPubkey`-for-`from_slice`-only is consistent with M2's distinct-variants stance + the §5 enum; I2's re-export path and I4's direct-dep list don't overlap.
