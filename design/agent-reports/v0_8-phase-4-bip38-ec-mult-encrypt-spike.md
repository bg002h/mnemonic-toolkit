# v0.8 Phase 4 SPIKE — BIP-38 EC-multiplied encrypt API coverage

**Status:** SPIKE COMPLETE — verdict **DEFER to v0.8.1 / v0.9** (not blocking v0.8.0).

**Question 1 (plan):** Does `str::generate(compress)` from `bip38 v1.1.1` cover the
**full intermediate-code → encrypted privkey + confirmation code + address**
output per BIP-38 §"Encryption when EC multiply mode is used" steps 2–7?

**Answer: No.** `str::generate(compress)` covers only the simplest owner-only path
(no lot/sequence, no intermediate code, randomly-chosen `seed_b`). Specifically:

- **API surface:** `fn generate(&self, compress: bool) -> Result<String, Error>`.
  Takes ONE input (the passphrase as `&str`); returns one output (the encrypted
  WIF as a `String`). All entropy sources are sampled internally via
  `rand::thread_rng().fill_bytes(...)` (lines 788, 802 of `lib.rs`).

- **What it produces:** a 39-byte buffer of shape
  `[PRE_EC || flag || address_hash || owner_salt || encrypted_part1[..8] || encrypted_part2]`
  base58check-encoded. This corresponds to BIP-38 EC-multiplied steps **2–7
  fused into one call, with no intermediate code observable**.

- **What it does NOT produce:**
  1. **Intermediate code** (the BIP-38 "passphrase code" that the passphrase
     owner generates and hands to a third-party encryptor). Path B step.
  2. **Lot/sequence support.** The crate's `pass_factor = scrypt(passphrase, owner_salt, ...)`
     skips the lot/sequence pre-factor pathway entirely (BIP-38 §"Generation of
     intermediate code" alternate path with `magic[7] = 0x51`).
  3. **Confirmation code** (the value that lets the passphrase owner verify
     the encrypted privkey was generated correctly without revealing it).
  4. **Deterministic encryption from a known `owner_salt` + `seed_b`** — the
     two RNG sites at line 788 (`owner_salt`) and line 802 (`seed_b`) make
     pinning the 4 BIP-38 spec EC-multiplied test vectors (EC1–EC4)
     impossible without monkey-patching the crate.

**Question 2 (plan):** If gaps exist, identify the minimum delta against the spec.

**Answer:** The minimum delta is a **full hand-roll** of the EC-multiplied
encrypt-side surface: the crate exposes neither the underlying primitives
(scrypt with chosen salt, AES-256 with chosen key, secp256k1 scalar
multiplication with chosen factor) nor a deterministic encryption hook. A
wrapper around `Generate` cannot reach the spec test vectors because the
RNG calls are inside the trait method.

A clean implementation requires:

| Component                        | Already in tree?                                  | LOC est. |
|----------------------------------|---------------------------------------------------|----------|
| Scrypt(N=14, r=8, p=8, dkLen=32) | yes (`scrypt` via `bip38`)                        | 0        |
| Scrypt(N=10, r=1, p=1, dkLen=64) | yes (same)                                        | 0        |
| AES-256 ECB                      | yes (`aes` via `bip38`)                           | 0        |
| secp256k1 scalar mult            | yes (`bitcoin::secp256k1`)                        | 0        |
| HMAC-SHA256 (BIP-38 doesn't use HMAC) | n/a                                          | 0        |
| Address + hash160 + base58check  | yes (`bitcoin`)                                   | 0        |
| Owner-only EC-mult encrypt       | new (steps 2–7 fused, deterministic salts)       | ~40      |
| Intermediate-code generation     | new (Path B owner step + lot/sequence pre-factor) | ~30      |
| Third-party encrypt-from-intermediate | new (steps 4–7 keyed by intermediate code)   | ~40      |
| Confirmation code generation     | new (step 8 of "Encryption when EC multiply ...") | ~25      |
| Confirmation code verification   | new (round-trip of step 8)                        | ~20      |
| **Total**                        |                                                   | **~155** |

This exceeds the plan's revised estimate (10–30 wrap / 30–60 wrap+delta /
80–100 hand-roll); the upper end was sized for the owner-only path. The
intermediate-code workflow is approximately the same LOC again because it
introduces an entirely separate code path with its own SPEC § and refusal
taxonomy.

## Verdict: DEFER

**Reasoning:**

1. **High implementation cost** — ~155 LOC of hand-rolled cryptographic code
   touches AES, scrypt parameter selection, secp256k1 scalar arithmetic,
   and Unicode normalization. Each is a known correctness/security risk
   surface.
2. **No upstream wrappable primitive** — the `bip38` crate's internal
   primitives are `pub(crate)` / `trait` private; we cannot avoid
   re-implementing them.
3. **Marginal user value** — BIP-38 EC-multiplied encrypt is the niche
   third-party paper-wallet generation flow. v0.7.1's audit cycle already
   pins all 4 BIP-38 spec EC-multiplied DECRYPT vectors (EC1–EC4 →
   COVERED via the bip38 crate's `Decrypt` trait). The encrypt-side gap
   is real but addresses a specific historical use case (BitAddress.org-
   style key generation services) that modern wallets don't use.
4. **Plan's natural seam supports deferral.** From the plan's "Natural seam
   (escape hatch)" clause:
   > "If Phase 4 or 6 spikes return blockers, ship interim **v0.8.0** at
   > end of Phase 3 ... Carry Phase 5 (BIP-38 EC-mult encrypt) to
   > **v0.8.1**; carry Phase 7 (BIP-85 RSA/RSA-GPG/DICE) to **v0.8.2** (or
   > **v0.9.0** if RSA crate audit defers indefinitely)."

## Phase 5 disposition

**Phase 5 (BIP-38 EC-mult encrypt):** **DEFERRED** to v0.8.1 (or v0.9 if
the implementation cost still feels disproportionate to user demand at
the v0.8.1 ship-cycle decision point).

The v0.7.1 FOLLOWUP `bip38-ec-multiplied-encrypt-mode-support` remains
**OPEN** at tier `v0.8.1` (was `v0.8`); update on Phase 9 close-out.

## Cite-only test corpus

The 4 BIP-38 spec EC-multiplied vectors stay COVERED on the **decrypt
side** (pinned in v0.7.1 Phase 3) and OUT-OF-SCOPE on the encrypt side.
Audit matrix entries unchanged — encrypt rows still marked
`OUT-OF-SCOPE-PER-USER (encrypt-mode deferred)` with the cross-reference
updated from `v0.8` to `v0.8.1+`.

## Spike duration

~30 minutes (no code written; source-level audit of `bip38 v1.1.1`
`Generate` impl + cross-reference against BIP-38 spec).
