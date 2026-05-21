# Cycle 7 kickoff — BIP-129 encryption envelope (7a library + 7b CLI/ship)

**Created:** 2026-05-21 at the Cycle 7a/7b split point. Read this file first when resuming Cycle 7b in a fresh session.

## Where we are

- **Last shipped:** `mnemonic-toolkit-v0.30.1` (Cycle 6b; `11fd38f`) — encrypted Electrum watch-only passthrough. CI green.
- **Cycle 7a shipping THIS session:** BIP-129 encryption library (`bsms_crypto.rs`) + recon dossier + brainstorm + opus R0 + test vectors. No CLI surface change. No version bump.
- **Cycle 7b remains for next session:** Phase 2-7 (CLI flag `--bsms-encryption-token` + `bsms.rs` parser integration + manual chapter updates + ship + GUI lockstep + FOLLOWUP closure).

## Why split-cycle

Cycle 6 used the same 6a/6b split. The 6b R0 opus review of the 6a brainstorm caught a foundational design error (parser doesn't read encrypted fields → Path A pivot dropped `--decrypt-password*` entirely). That error would have wasted 6b's Phase 2-6 implementation. Splitting Cycle 7 gives Cycle 7b's R0 the same chance to catch errors in the CLI/parser integration design before implementation.

## Cycle 7 scope (locked at brainstorm)

- **Parent FOLLOWUP slug:** `bsms-bip129-encryption-envelope` (`design/FOLLOWUPS.md:2546`).
- **Scheme (verified vs BIP-129 primary source 2026-05-21):**
  - PBKDF2-SHA512(password=`"No SPOF"`, salt=TOKEN_raw_bytes, c=2048, dkLen=32) → ENCRYPTION_KEY.
  - HMAC_KEY = SHA256(ENCRYPTION_KEY).
  - MAC = HMAC-SHA256(HMAC_KEY, hex_ascii(TOKEN) || plaintext).
  - IV = first 16 bytes of MAC.
  - Ciphertext = AES-256-CTR-Encrypt(plaintext, ENCRYPTION_KEY, IV).
  - Wire = hex(MAC || ciphertext).
  - Encrypt-and-MAC ordering per BIP-129 line 165.
- **Critical footgun:** PBKDF2 salt = RAW bytes of TOKEN; HMAC input = ASCII-HEX of TOKEN. Same TOKEN, two byte representations.
- **TOKEN modes:** STANDARD (8 bytes raw / 16 hex chars) + EXTENDED (16 bytes raw / 32 hex chars). NO_ENCRYPTION (TOKEN=0x00) is handled by the existing plaintext path.

## Cycle 7a deliverables (THIS session, complete)

1. `design/CYCLE_7_KICKOFF.md` (this file).
2. `design/cycle-7-p0-recon.md` (primary-source recon dossier).
3. `design/BRAINSTORM_v0_31_0_bsms_bip129_encryption_v1_7a_library.md` (architectural locks for the 7a library scope; opus R0 dispatched before Phase 1).
4. `design/agent-reports/v0_31_0-bsms-crypto-brainstorm-r0-review.md` (opus R0 verbatim).
5. `crates/mnemonic-toolkit/src/bsms_crypto.rs` — library: `derive_encryption_key` + `derive_hmac_key` + `compute_mac` + `decrypt` + `encrypt` (symmetric helper) + library-local `BsmsCryptoError` enum + 18+ unit cells covering TV-3 (STANDARD; cross-validated values from BIP-129 + v0.27.0 dossier) + TV-4 (EXTENDED) + refusal classes.
6. `Cargo.toml`: new direct dep `ctr = "0.9"`.
7. `lib.rs`: `pub mod bsms_crypto;` + doc-comment bullet appended to the lib-local-error sibling list.

**NOT shipped in 7a:** version bump, install.sh self-pin, CHANGELOG entry, GUI changes, parser integration, CLI flags, manual chapter, tag, GH Release. All deferred to 7b.

## Cycle 7b resume prompt

After `/clear` (or in a fresh session), issue this prompt:

```
Read design/CYCLE_7_KICKOFF.md and proceed with Cycle 7b (BIP-129 encryption-envelope CLI + parser integration + ship). Use the same discipline as Cycle 6b (P0 STRICT-GATE recon refresh → plan-doc R0+R1 opus review BEFORE Phase 2 dispatch → subagent-driven implementation → opus end-of-cycle review → install-pin-check CI gate on tag push). Cycle 7a shipped bsms_crypto.rs + 18+ unit cells; 7b ships the parser integration + --bsms-encryption-token flag through to a MINOR tag + GUI lockstep + FOLLOWUP closure.

LESSON FROM CYCLE 6b: dispatch opus R0 review on the brainstorm + plan-doc BEFORE Phase 2 implementation. The Cycle 6a brainstorm shipped without R0 review; 6b R0 then caught a foundational design error that invalidated the entire --decrypt-password* design. Don't repeat that.
```

## Phase 2-7 detail (deferred to 7b)

### Phase 2 — CLI plumbing in `cmd/import_wallet.rs`

- Add `--bsms-encryption-token <FILE|->` flag (single-form per parent brainstorm; user-direction may revise to 3-form). Read raw token bytes from file or stdin.
- Mutex: at most ONE of file/stdin (if 3-form, mutex group).
- `secret_in_argv_warning` for inline form (if any).
- `warn_if_world_readable` for file form.
- `secrets.rs::flag_is_secret`: add `--bsms-encryption-token` (value-dependent secret).

### Phase 3 — parser integration in `wallet_import/bsms.rs`

- Detection: if `--bsms-encryption-token` supplied → orchestrator reads token, dispatches to encrypted-path; else → existing plaintext path.
- Encrypted path:
  - Parse wire: `hex_decode(blob)` → `MAC` (32 bytes) + `ciphertext` (remainder).
  - Call `bsms_crypto::decrypt(ciphertext, &dkey, &iv) -> plaintext`.
  - Compute expected MAC; verify; refuse on mismatch with typed error `BsmsCryptoError::MacMismatch` mapped to `ToolkitError::BadInput` (or new `ToolkitError::BsmsMacMismatch` per architect's "typed error variant" recommendation in the FOLLOWUP body).
  - Pass plaintext to existing 4-line/6-line parser.
- Stderr advisory for MAC verify path: "BIP-129 encrypted Round-2 envelope; decrypted + MAC-verified".

### Phase 4 — `tests/cli_import_wallet_bsms_encrypted.rs`

~20-30 cells covering: TV-3 happy path (cross-impl smoke against the cross-validated values from the recon dossier); TV-4 EXTENDED happy path; wrong-token refusal (MAC mismatch); malformed-hex refusal; MAC-too-short refusal; missing-token refusal (encrypted blob WITHOUT `--bsms-encryption-token`); etc.

### Phase 5 — Manual chapter

New section in chapter-45 §"BSMS" or chapter-41 §`mnemonic import-wallet` documenting the `--bsms-encryption-token` flag + the encrypted-Round-2 wire shape.

### Phase 6 — Cycle close (toolkit MINOR ship)

- `Cargo.toml`: v0.30.1 → v0.31.0 (MINOR per new secret-bearing flag pattern; same SemVer rule as Cycle 6's Path B would have used had it shipped).
- `install.sh:32` toolkit pin bump.
- `CHANGELOG.md` v0.31.0 section.
- Tag + push + GH Release + install-pin-check CI.

### Phase 7 — GUI lockstep + FOLLOWUP closure

- GUI v0.16.0: pin bump + new `SubcommandSchema` entries for `--bsms-encryption-token` on import-wallet schema.
- `schema_mirror` test verify with explicit `MNEMONIC_BIN=...`.
- FOLLOWUP closure:
  - Close `bsms-bip129-encryption-envelope` (resolved by v0.31.0).
  - Close (or cross-cite) `wallet-import-bsms-encrypted` (v0.27+ predecessor entry; reconcile bodies).
  - File new `bsms-bip129-encryption-cross-impl-coinkite-python-smoke` IF cross-impl smoke is deferred from 7b.

## Memory entries to consult on resume

- `project_v0_30_1_cycle_6b_shipped` — Cycle 6b context; specifically the R0 lesson.
- `project_v0_31_0_cycle_6a_shipped` — Cycle 6a library-ship pattern.
- `feedback_no_parallelism_for_code_generation` — subagent dispatch hygiene.
- `feedback_opus_primary_review_agent` — opus for substantive reviews.

## Repo state at session-end (post-Cycle-7a)

- mnemonic-toolkit master HEAD: TBD (post-7a commit; backfilled here at 7a close).
- mnemonic-gui master HEAD: `5582e22` (Cycle 5 paired GUI commit; unchanged since v0.30.0 ship).
- Last toolkit tag: `mnemonic-toolkit-v0.30.1` (Cycle 6b ship `11fd38f`).
