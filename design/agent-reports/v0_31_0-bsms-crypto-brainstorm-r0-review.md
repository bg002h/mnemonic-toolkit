# v0.31.0 Cycle 7a brainstorm R0 review

**Reviewer:** opus
**Round:** R0
**Spec under review:** design/BRAINSTORM_v0_31_0_bsms_bip129_encryption_v1_7a_library.md
**Date:** 2026-05-21
**Source SHA:** b03cac3 (master HEAD)

## Critical (C)

### C1. Architecture code-block (line 55) declares the WRONG Ctr variant: `Ctr64BE` instead of `Ctr128BE`.

**Citation:** brainstorm L55: `type Aes256Ctr = ctr::Ctr64BE<Aes256>;`. Section "AES-CTR variant choice" (L104-116) correctly hypothesizes `Ctr128BE` and defers empirical confirmation to Phase 1 TV-3 cell — but the architecture skeleton block itself hard-codes `Ctr64BE`.

**Evidence the correct variant is Ctr128BE:**
- Coinkite Python `bsms/encryption.py:34` (verified verbatim 2026-05-21): `pyaes.AESModeOfOperationCTR(key, pyaes.Counter(int(iv.hex(), 16)))` — initializes a SINGLE 128-bit integer counter from the full 16-byte IV (no nonce/counter split).
- `ctr` crate docs: `Ctr128BE` "treats the entire 16-byte IV as a single 128-bit big-endian counter"; `Ctr64BE` treats first 8 bytes as nonce + last 8 bytes as 64-bit BE counter (RFC 3686 nonce/IV/counter framing).
- TV-3 will fail under `Ctr64BE` because the keystream block-index counter increments at byte-15 only; Coinkite's full-128-bit counter increments roll into byte-7 etc. after 2^64 blocks (irrelevant for short records BUT the initial counter value as interpreted differs immediately because no nonce-prefix is reserved).

**Fix:** change L55 to `type Aes256Ctr = ctr::Ctr128BE<Aes256>;` and update the "AES-CTR variant choice" subsection to lock `Ctr128BE` as Decision-of-Record (with the Coinkite Python citation supporting), reframing the Phase 1 TV-3 cell as confirmation-not-discovery. The "if fails switch and retry" framing leaves uncertainty in a Phase 1 plan that should be locked.

## Important (I)

### I1. `derive_encryption_key` `.expect()` on `pbkdf2(...)` — verify call shape mirrors SLIP-39 precedent

**Citation:** brainstorm L68-69:
```rust
pbkdf2::<Hmac<Sha512>>(b"No SPOF", token_raw, 2048, out.as_mut_slice())
    .expect("pbkdf2 fill must succeed (...)");
```

The `pbkdf2::pbkdf2::<F>(...)` free function in the `0.12` crate returns `()` (with the `simple` feature off) — there is no `Result` to `.expect()`. The fallible variant is `pbkdf2_hmac` vs `pbkdf2_hmac_array` returning `Result<_, InvalidLength>`. Phase 1 will get a compile error.

**Fix:** drop the `.expect(...)` chain — write `pbkdf2::<Hmac<Sha512>>(b"No SPOF", token_raw, 2048, out.as_mut_slice());` as a statement — OR switch to `pbkdf2_hmac::<Sha512>(...)` and keep the `.expect()`. The brainstorm should pick one and lock the signature so Phase 1 isn't fixing this mid-stream. (SLIP-39 at `crates/mnemonic-toolkit/src/slip39/` already uses this crate; mirror that call shape.)

### I2. `compute_mac` accepts `token_hex: &str` but the type does not enforce hex-validity. A caller could pass a raw-byte hex string OR a malformed string; both type-check.

**Citation:** brainstorm L25, L81-83. Per BIP-129 line 152 and Coinkite Python `m_a_c` (`(token + data).encode()`), the literal ASCII bytes of the line-2 hex string are what hash; if a caller pre-validated raw bytes and accidentally passes those, MAC will silently mismatch every TV. The library cannot enforce hex-shape without parsing, but the doc-comment must call out the contract explicitly.

**Fix:** add a `#[must_use]` doc-comment to `compute_mac` stating: "`token_hex` MUST be the lowercase ASCII hex representation of TOKEN, NOT the raw bytes. This is the foot-gun BIP-129 §Encryption + the v0.27.0 recon dossier §Critical Asymmetry both call out." Same advisory in module-level `//!` doc-comment around L42.

### I3. Test plan §4 "decrypt — variant determination + TV-3 cross-validation" frames Phase 1 as discovery, but the recon already determines it.

**Citation:** brainstorm L147-149. "If fails, switch to Ctr64BE and retry" is exploratory. Phase 1 should be deterministic: write the test asserting `Ctr128BE` produces the TV-3 plaintext on the first try; if it fails, that's a Phase-1 RED diagnosis, not a plan-doc-mandated fallback. This couples with C1.

**Fix:** rewrite §4 to read "TV-3 decrypt under `Ctr128BE<Aes256>` MUST recover the BIP-129 Round-1 5-line plaintext byte-identical (per Coinkite Python ref + ctr-crate counter-width semantics; see Decision-of-Record above)."

### I4. Module skeleton lacks the brainstorm-promised "hand-rolled `impl Display`" body (L63 shows only a `/* hand-rolled */` placeholder).

**Citation:** brainstorm L63. Per the `seed_xor.rs:45-65` precedent the Phase 1 implementer will need exact Display strings for `InvalidWireFormat { reason }` + `MacMismatch`. Locking the strings in the brainstorm avoids a Phase-1 implementer-discretion drift that downstream CLI mapper (Cycle 7b) will then have to chase if it tries to do substring matches.

**Fix:** add explicit Display strings — e.g.,  `BsmsCryptoError::InvalidWireFormat { reason } => write!(f, "bsms-crypto: invalid wire format: {reason}")`, `BsmsCryptoError::MacMismatch => write!(f, "bsms-crypto: MAC verification failed (wrong token or tampered ciphertext)")`. Cycle 7b CLI boundary mapper then knows the exact text.

## Minor (M)

### M1. TV-3 ciphertext is "extracted at Phase 1 implementation time" (L171) — would be cleaner to lock it in the brainstorm.

The full TV-3 wire blob is available from the BIP-129 source — I retrieved it during this review:
```
fbdbdb64e6a8231c342131d9f13dcd5a954b4c5021658fa5afcb3fc74dc8270653f491cfd1431c292d922ea5a5dec3eb8ddaa6ed38ae109e7b040f0f23013e89a89b4d27476761a01197a3277850b2bc1621ae626efe65f2081eec6eb571c4f787bf1c49d061b43f70fd73cb3f37fa591d2400973ac0644c8941a83f1d4155e98f01fa2fdeb9f86c2e2413154fd18566a28fb0d9d8bd6172efabcfa6dab09ee7029bf3dd43376df52c118a6d291ec168f4ec7f7df951dfc6135fd8cb4b234da62eaea6017dfe5ca418f083e02e3aba2962ba313ba17b6468c7672fb218329a9f3fe4e4887fb87dac57c63ebff0e715a44498d18de8afc10e1cfeb46a1fc65ce871fef8a43b289305433a90c342d025aa4c19454fcfbcf911e9e2f928d5affd0536a6ddc2e816
```
First 64 hex chars = MAC (matches the recon-dossier MAC `fbdbdb64...c82706`); remainder = ciphertext. Lock this in the brainstorm test-plan §4-5 so Phase 1 reads the brainstorm, not the BIP source.

### M2. `encrypt` symmetric helper as `pub` (vs `#[cfg(test)]`-gated) is fine — open question 3 (L177) self-resolves via the `electrum_crypto.rs::encrypt_field` precedent.

`electrum_crypto.rs` ships `pub fn encrypt_field` (verified at L154 of that file). Following Cycle 6a precedent is correct; no change needed. Document the rationale in the module `//!` block (one bullet referencing the Cycle 6a precedent).

### M3. `derive_hmac_key` non-Zeroizing return (open question 2, L176) is acceptable but should be documented.

The HMAC_KEY is short-lived stack data derived from ENCRYPTION_KEY (which IS zeroized). Threat model: an attacker who can read process stack already has access to ENCRYPTION_KEY too. Cycle 6a's `derive_key` returns `Zeroizing<[u8; 32]>` but its consumers immediately move ownership into AES-CBC contexts; the BIP-129 HMAC_KEY is consumed similarly. Acceptable — add a doc-comment note: "Returned `[u8; 32]` is intentionally NOT zeroized because (a) the upstream ENCRYPTION_KEY is zeroized, (b) the HMAC_KEY lifetime is bounded by the immediately-following `compute_mac` call. A caller that retains HMAC_KEY beyond the immediate MAC compute should wrap manually."

### M4. Brainstorm "Open questions for opus R0" §3 (L177) self-resolves; §1 closes via C1 above; §2 closes via M3 above. After fold all three OPEN-QUESTIONS items are resolved.

Update the brainstorm to mark each open-question RESOLVED with citation (C1 / M3 / M2 of this review) and a one-line decision.

## Verdict

**YELLOW — fold then proceed.** One critical (C1: wrong Ctr variant in code skeleton) plus four importants; C1 alone would burn Phase 1 in a confused TV-3 fallback path. After folding C1 + I1-I4 (Ctr128BE locked; pbkdf2 call shape locked to either `()`-return or `_array` variant; foot-gun doc-comment for `compute_mac`; explicit Display strings; TV-3 framed as confirmation) the brainstorm is GREEN to seed Phase 1. Recommend a quick R1 dispatch on the folded brainstorm per CLAUDE.md "reviewer-loop continues after every fold" convention.

**Summary:** Brainstorm scheme citations are byte-exact against BIP-129 primary source (verified via `bitcoin/bips/bip-0129.mediawiki` 2026-05-21) and against Coinkite Python `bsms/encryption.py` (verified verbatim 2026-05-21). TV-3 values (ENCRYPTION_KEY / HMAC_KEY / MAC / IV) match the recon-dossier values. PBKDF2-salt-raw vs HMAC-input-ASCII-hex asymmetry is correctly captured. The library-local error pattern matches `seed_xor.rs:31-67` + `electrum_crypto.rs:46-90` precedent. The one critical issue is mechanical (one-line type-alias swap); the importants are spec-tightening so Phase 1 has zero discretion left to drift.
