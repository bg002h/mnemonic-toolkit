# v0.32.3 plan-doc R0 review (Cycle 17 — bsms-encryption-cross-impl-coinkite-python-smoke)

**Reviewer:** opus (feature-dev:code-reviewer)
**Round:** R0
**Plan:** `design/PLAN_mnemonic_toolkit_v0_32_3.md`
**Date:** 2026-05-21
**Source SHA:** `5cf6e1c`

## Verdict

**GREEN.** 0 Critical / 1 Important / 2 Minor + 1 scope-audit note. All foldable; nothing blocks Phase 2.

## Recipe alignment (verified)

Coinkite `m_a_c(key, token, data)` concatenates the HEX token string with plaintext, matching toolkit `compute_mac(hmac_key, token_hex, data)`. IV=MAC[:16], Ctr128BE, PBKDF2-SHA512/2048/32, salt=raw-token-bytes — all match. The descriptor-byte-equal invariant is non-circular (independent ciphertext → identical toolkit output).

## Important (I)

**I1 — descriptor-only equality under-covers the keystream.** `bsms-2line-multi-2of3.txt` is a multi-field record (~460 bytes); asserting only `json["descriptor"]` equality does NOT exercise the keystream over the trailing bytes — a counter-rollover/keystream-length bug past byte ~430 would survive. **Fold:** add a cell asserting FULL decrypted-plaintext byte-equality — decrypt the vendored wire via `mnemonic_toolkit::bsms_crypto::decrypt` (as `tv3_decrypted_plaintext()` already does) and compare to the `bsms-2line-multi-2of3.txt` fixture bytes exactly. This is the strong cross-impl pin; the CLI-import + descriptor-equality cell stays as the end-to-end check.

## Minor (M)

**M1 — hardcoded-descriptor pin.** A `const` expected descriptor gives a self-describing diff. Largely subsumed by I1's full-plaintext-equality; optional.

**M2 — regen-script newline foot-gun.** The plaintext fixture ends with `\n`; the script MUST read it as exact bytes (no `.strip()`/`.rstrip()`), and read the TOKEN file with `.strip()` before `bytes.fromhex`. The script should self-verify: re-decrypt its own output + assert byte-equality before writing. Document in the README.

## Scope-audit note (Q10)

The FOLLOWUP body literally specified (a) clone + (b) run `python3 test.py` + (c) CI-gating. The user-locked vendored-only scope DROPS (b)+(c). The closure note MUST explicitly record this intentional narrowing (cite the user lock) so the audit trail shows it was deliberate — and either file a residual "optional opt-in CI cross-impl smoke" slug or explicitly waive it (frozen repo + vendored-output pin + existing TV-3 byte-exact ⇒ waive is defensible).

## Verified clear

- Determinism (deterministic IV + fixed key/plaintext → fixed ciphertext; pure-Python pyaes; no platform RNG): reproducible byte-for-byte.
- Staleness-detection logic: sound (stale wire → decrypt yields old descriptor ≠ new plaintext import → equality fails).
- No-CI-dependency: script in `tests/external/`, never referenced by workflows.
- Wrong-token cell: sound (mirrors existing exit-2 pattern).
- Token-width coverage: STANDARD (TV-3) + EXTENDED (new) — and this is the FIRST EXTENDED wire that actually DECRYPTS (the existing `extended_mode_32_hex_token_passes_width_check` only exercised width-acceptance against a MAC-failing wire). Worth a plan note.
- SemVer PATCH / no GUI lockstep: correct.
- Arc closure: this retires the last of the 3 Cycle-7 BIP-129 child slugs; confirm no sibling-repo companion entry needs lockstep (toolkit-internal → none).

## Recommendation

Fold I1 (full-plaintext-equality cell) + M2 (regen newline + self-verify) + scope-audit closure note, then Phase 2.
