# v0.32.1 plan-doc R0 review (Cycle 15 — bsms-encryption-round1-decrypt-then-verify)

**Reviewer:** opus (feature-dev:code-reviewer)
**Round:** R0
**Plan:** `design/PLAN_mnemonic_toolkit_v0_32_1.md`
**Date:** 2026-05-21
**Source SHA:** `1924e19`

## Verdict

**GREEN.** 0 Critical / 1 Important / 2 Minor — all foldable; none block Phase 2.

## Citations (all hold)

token field 204-205; bsms_round1 191-192; verify call 252-256; stdin guard 277-284; Round-2 decrypt 841-899 (recipe 862-890); read_bsms_token 1897; verify_bsms_round1_files 1952-2022; `BSMS_HEADER` bsms_round1.rs:26; bsms_crypto pub fns 98/114/136/160.

## Important (I)

**I1 — Hoist-site self-contradiction.** `verify_bsms_round1_files` (L252) runs BEFORE the standalone early-return (L268), BEFORE the stdin guard (L277-284), BEFORE the blob read (L287). So the token read MUST hoist to before L252 — NOT "≈L284, after the stdin-contention guard" as the file-structure bullet states (contradicting the risk-register's correct "before L252"). The guard at L277-284 references `blob_path` (only exists after the L260 match), so the guard cannot simply move above L252 unchanged. **Resolution:** (a) move the stdin-contention check above L252, rewritten to use `args.blob.as_ref()` + `args.bsms_encryption_token.as_ref()` directly (both in `args`; no dependency on the L260 `blob_path` binding); (b) hoist the token read immediately after that guard, before L252. In standalone mode `args.blob` is None → guard doesn't fire (same outcome). This ordering ensures the stdin-contention refusal happens BEFORE the token consumes stdin.

## Minor (M)

**M1 — Cite the existing TV-3 verify test.** `bsms_verify.rs:109 tv3_standard_encryption_xpub_signer1_verifies` already proves the decrypted TV-3 Round-1 plaintext passes `verify_round1_signature`. The plan's "confirm-or-escalate" risk entry is over-cautious — replace with a citation to this existing test.

**M2 — Add a decrypt-success + BIP-322-verify-FAIL cell.** The plan flags but doesn't add a cell for an encrypted record that decrypts + MAC-verifies OK but whose plaintext has a flipped/invalid BIP-322 signature. Add one to cover the lenient-NOTICE-vs-strict-fatal branch ON A DECRYPTED record (exercises the interaction of decrypt-then-verify with `--bsms-verify-strict`).

## Verified clear

- stdin (check 3): `--bsms-round1` rejects `-`; standalone `--token=-` read once; --blob dual-stdin still guarded. Safe after the I1 reordering.
- `is_encrypted_round1` (check 4): sound; no false pos/neg (plaintext always starts `BSMS 1.0`; encrypted is raw hex).
- decrypt recipe (check 5): mirrors 862-890 exactly.
- `BsmsMacMismatch` (check 6): carries `token_len_hex`.
- SemVer PATCH (8) + manual lint (10): correct; no new flag.

## Recommendation

Fold I1 (hoist-site + stdin-guard reorder) + M1 (cite existing test) + M2 (verify-fail cell), then Phase 2.
