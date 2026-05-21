# v0.32.1 end-of-cycle architect review

**Reviewer:** opus (feature-dev:code-reviewer)
**Round:** end-of-cycle
**Cycle:** Cycle 15 (bsms-encryption-round1-decrypt-then-verify)
**Date:** 2026-05-21
**Pre-tag SHA:** `ff8b2e2` (Phase 2-4; Phase 5 uncommitted)

## Verdict

**GREEN.** All 10 verification items pass. 0 Critical / 0 Important / 0 Minor. (Security-adjacent: BIP-129 encryption + MAC verify.)

## Check-by-check

1. **Token hoist + stdin guard ordering** (`cmd/import_wallet.rs:251-286`): guard (L258-264) fires before token read (L269-272) before `verify_bsms_round1_files` (L277); uses `&args.blob`/`&args.bsms_encryption_token` directly; standalone mode (no --blob) works (guard short-circuits, None-branch emits envelope + returns). Correct R0 I1 fold.
2. **MAC verify correctness** (`decrypt_bsms_record`): Encrypt-and-MAC ordering — decrypt FIRST, then `compute_mac(hmac_key, token.hex, plaintext)` over the DECRYPTED plaintext, byte-compared to `mac_recv`; IV = `mac_recv[..16]`; mismatch → `BsmsMacMismatch`. HMAC prefix = `token.hex` (lowercase ASCII hex), opposite representation from the PBKDF2 salt `token.raw` — correct per BIP-129.
3. **Round-2 byte-identity**: `ctx="bsms: encrypted Round-2 wire"` reproduces the "not valid hex"/"too short" strings exactly; NOTICE unchanged; UTF-8-hex check stays inline. Prior 12 encrypted-suite cells match.
4. **`is_encrypted_bsms_record`**: sound (header→false, raw hex→true, empty→false). Unit cell covers all branches.
5. **No-token refusal**: `BadInput` exit 1.
6. **decrypt-OK-but-sig-FAIL cell**: `reencrypt_with_tv3_token` computes MAC correctly → decrypt+MAC pass; corrupted SIG → lenient NOTICE/exit 0 + strict fatal.
7. **No secret leakage**: plaintext (pubkey/sig, non-secret) flows through normal Strings; `Zeroizing` decrypt output consumed into String (same as Round-2). No new disk/argv exposure.
8. **CHANGELOG / install.sh / Cargo.toml**: all 0.32.1.
9. **SemVer PATCH / no GUI lockstep**: no new flag.
10. **Test totals**: +6 (5 integration + 1 unit) → 2198.

## Cleared for tag.
