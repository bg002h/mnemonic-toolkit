# Electrum BIE1 Phase-B implementation — opus end-of-cycle review (verbatim)

Review of the uncommitted Phase-B working tree (import-wallet `--decrypt-password*` + `detect_storage_magic` + orchestrator decrypt + tests + manual + v0.33.2 bump), feature-dev:code-reviewer (opus). Persisted per CLAUDE.md. **VERDICT: GREEN (0 Critical / 1 Important / 3 Minor).** All folded.

## Important
**I1 — `import-wallet-blob-zeroizing` FOLLOWUP was promised by the plan but not filed.** `blob = plaintext.to_vec()` drops the `Zeroizing` wrapper; the recovered wallet JSON (may carry seed/xprv) lands in a plain `Vec<u8>` (mlock-pinned, not scrubbed). Plan-sanctioned deferral, but the tracking entry didn't exist. **FOLDED:** filed `import-wallet-blob-zeroizing` in `design/FOLLOWUPS.md`.

## Minor
- **M1** `.expect("detect_storage_magic confirmed valid UTF-8")` is genuinely unreachable (detection utf8-validated the same un-mutated `&blob`); the comment documents the invariant. No action.
- **M2** inline `--decrypt-password` on a plaintext wallet leaked via argv but got no advisory (resolution, hence the advisory, only ran on the BIE1 path). **FOLDED:** the `None` arm now fires `secret_in_argv_warning` when an inline password is present; the `password_supplied_for_plaintext_wallet_is_ignored` cell asserts it.
- **M3** `bsms_token_stdin_plus_password_stdin_refused` passes a real fixture as `--blob` but the guard fires before `read_blob` — cosmetic; the fixed-string assertion is drift-catching. No action.

## Verified clean
- Insertion order: decrypt after `read_blob`, before `sniff_format`; replacement feeds unchanged `--format`/sniff/BSMS arms. `--format bsms`+BIE1 decrypts-then-mismatches (`ImportWalletFormatMismatch` Display contains both "bsms"+"electrum").
- 3-way stdin guard hoisted before the token read; both new pairs have dedicated cells; existing guards untouched.
- Detection: `len>=85`+magic, same `BASE64`+`trim` as decrypt; JSON `{` non-base64 (no false-positive); trailing-newline tolerated; 6 unit cells.
- Oracle: `HmacMismatch|AesDecryptFailure` unified non-leaky; BIE2 refused before key derivation; encrypt-then-MAC verify-before-AES (no padding oracle).
- Secret hygiene: password `Zeroizing`+mlock; plaintext mlock-pinned; inline advisory fires; no password in any error/notice.
- `flag_is_secret` covers `--decrypt-password`/`-stdin` (v0.33.1); `gui-schema` derives `secret` dynamically by name → GUI secret projection satisfied.
- Fixture authority: `regen_electrum_bie1_storage.py` imports only `ecdsa`/`cryptography`/stdlib (independent); committed fixture decrypts via the toolkit (cross-impl).
- SemVer PATCH v0.33.2 correct; manual mirror + GUI lockstep addressed.
