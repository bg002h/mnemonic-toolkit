# v0.34.0 nostr-key-wrappers — Phase A opus code review (verbatim)

**Date:** 2026-05-22
**Reviewer:** opus `feature-dev:code-reviewer` (agent `a5a1ecd970e2e1c0e`)
**Scope:** Phase-A code delta `23534ae..79a0cd3` — `error.rs` (NostrKeyParse), `cmd/convert.rs` (`ScriptType::as_str`), `nostr.rs` (whole module + tests), `tests/external/regen_nostr_vectors.{py,md}` — vs `BRAINSTORM_v0_34_0` §3/§5 + `IMPLEMENTATION_PLAN_v0_34_0` Phase A0+A.
**Verdict:** **GREEN** — 0 Critical / 0 Important, 3 Minor.

---

## Crypto correctness — verified sound
- `normalize_to_even_y` (`nostr.rs:24-30`): `secret.negate()` = consuming `SecretKey::negate(self)` (secp256k1 0.29). Odd-y `d` → `n−d`; x-only of `(n−d)·G` == x-only of `d·G` (parity-independent), so npub preserved while WIF controls the even-y key. Edge cases handled in libsecp scalar arithmetic.
- `even_y_compressed` (`nostr.rs:94-96`): `PublicKey::from_x_only_public_key(xonly, Parity::Even)` wrapped in `CompressedPublicKey(pub PublicKey)` — correct BIP-340 even-y `02‖x`.
- `address_for` (`nostr.rs:99-112`): four call forms byte-for-byte match in-tree precedent `build_address_from_xpub` (`convert.rs:1570-1576`). `Address::p2tr(secp, xonly, None, hrp)` is correct BIP-86 key-path.
- `descriptor_for` (`nostr.rs:115-125`): `Descriptor::from_str(body).to_string()` correctly appends BIP-380 `#checksum`; round-trip parse validates it.
- `wif_for` (`nostr.rs:128-130`): mirrors `convert.rs:1190-1195`; version byte via `network_kind()` correct.
- decode (`nostr.rs:33-69`): HRP check, 32-byte length, hex-vs-bech32 disambiguation, `Zeroizing` on nsec — all correct; `Hrp::parse(static).expect` safe.

## Test / oracle adequacy — sound
`regen_nostr_vectors.py` is GENUINELY independent: pure-Python affine secp256k1, self-implemented BIP-340 `lift_x`, BIP-341/86 taptweak, self-implemented bech32/bech32m/base58check — NO rust-bitcoin. P2SH-P2WPKH redeemscript hashing correct (BIP-141). `EXPECTED_*` are real (no placeholder); fixture asserts `address_for == EXPECTED_*` for all four types.

## Spec compliance — sound
`NostrKeyParse(String)` alphabetical (between `NetworkMismatch`/`Repair`) in enum + `exit_code`(:480→1) + `kind`(:536) + `message`(:698→"nostr: {msg}"); NO phantom Display arm; no `details()` arm needed (`_ => None`). `ScriptType::as_str` round-trips with `parse_script_type_arg`. No over/under-build.

---

## Critical — None
## Important — None

## Minor
1. **Crux test's address-equality is a tautology** (`nostr.rs:164-168`): `xonly` and `xonly_from_secret` are always equal (parity-independent), so `a_pub == a_sec` holds even if `normalize_to_even_y` were a no-op. The genuine guarantee comes from (a) the final `parity == Even` assertion on the WIF-decoded key and (b) `normalize_tests` (seeds 1..=20). Suggest asserting the WIF key's x-only == the original npub x-only to make the claim non-trivial. Confidence 85.
2. **Dead `wpkh(03` branch** (`nostr.rs:182`): `even_y_compressed` always forces even, so body is always `02…`; `starts_with("wpkh(03")` unreachable. Cosmetic. Confidence 90.
3. **Plain-build `dead_code` warnings until Phase B**: the `pub fn`s are consumed only by `#[cfg(test)]` in Phase A; non-test consumers arrive in Phase B (`cmd/nostr.rs`). NOT a `clippy --all-targets -- -D warnings` failure (test cfg references them); self-resolves in Phase B. Confidence 80.

`electrum_prefix` (`p2sh-p2wpkh → "p2wpkh-p2sh:"`) correctly carried as OPEN plan item O2 (verify vs Electrum source before C4); not a Phase-A blocker.

---

## Verdict: GREEN — cleared to proceed to Phase B
0C/0I. Crypto correct, oracle genuinely independent + real constants, error wiring exhaustive + alphabetical + no phantom arms, `ScriptType::as_str` round-trips. 3 Minors non-blocking (Minors 1+2 folded post-review for test clarity; Minor 3 self-resolves in Phase B).
