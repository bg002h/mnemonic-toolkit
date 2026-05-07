# v0.7 Phase 1 — Code Quality Review

**Date:** 2026-05-06
**Git range:** `35c230e..c3d0a85`
**Files reviewed:** `crates/mnemonic-toolkit/src/cmd/convert.rs`, `crates/mnemonic-toolkit/tests/cli_convert_bip38.rs`, `design/agent-reports/v0_7-phase-1-bip38-security-review.md`

## Strengths

1. **Test vectors DRY.** Nine constants (`V1/V2/V3 × PASS/WIF/BIP38`) defined once at `tests/cli_convert_bip38.rs:10-22`, reused across all six spec-vector tests. Zero copy-paste hazard.
2. **`map_bip38_error` is clean and complete.** `bip38::Error::Pass` → `refusal_bip38_passphrase_mismatch()`; all other variants fall to `BadInput(format!("bip38: {other}"))`. No gaps, no panics, no `unwrap` in any new arm.
3. **Caveat 2 correctly implemented.** The `Bip38` arm at `convert.rs:726-728` calls `<str as Decrypt>::decrypt` (returns `([u8; 32], bool)`) instead of `decrypt_to_wif`. The `PrivateKey` is assembled with `network: network.network_kind()` at line 733, correctly honoring `--network testnet` for WIF encoding. Exactly the pattern prescribed by the security review.
4. **`bip38_edge` passphrase guard is minimal.** Lines 425-429 compute `bip38_edge` off already-available state; line 445 ORs it into `edge_uses_passphrase`. No set mutation, no accidental broadening to unrelated edges.
5. **Security-review line citations all verified accurate** against the actual crate source.

## Issues

### Important

**[I1] Dual-passphrase semantic ambiguity — `(Phrase|Entropy, Bip38)` composite arm**

- `crates/mnemonic-toolkit/src/cmd/convert.rs:539,618-622,630-631`
- `passphrase` (line 539: `args.passphrase.as_deref().unwrap_or("")`) is passed to BOTH `derive_bip32_at_path` (as the BIP-39 mnemonic extension into PBKDF2) AND to `encrypt_wif` (as the BIP-38 Scrypt key). A user who supplies `--passphrase myp38pass` intending only to set the encryption passphrase will silently derive a key whose seed incorporated `myp38pass` as its mnemonic extension — a different key than the same phrase with no mnemonic passphrase.
- The round-trip test at `cli_convert_bip38.rs:226` passes because it uses the same passphrase for both legs.
- **Resolution (decided by Phase 1 review iteration):** Lock dual-passphrase as INTENTIONAL in v0.7. Documented in SPEC §12 + inline comment + cross-check test. File `bip38-distinct-passphrase-flag` FOLLOWUP for v0.8 to optionally split with a `--bip38-passphrase` flag.

### Minor

**[M1] No `(Entropy, Bip38)` integration test**

- Composite path tests only exercise `phrase→bip38`. The `Entropy` branch shares code but is untested end-to-end via the CLI.
- **Resolution:** added `composite_entropy_to_bip38_via_wif` test in fix follow-up.

## Security Review Quality

The `v0_7-phase-1-bip38-security-review.md` is well-structured and accurate. All line citations verified. Caveat 1 deferral (EC-multiplied form / SPEC §12 amendment to Phase 8) is correctly scoped. Caveat 2 integration is correct.

## Assessment

**Shippable.** I1 resolved by SPEC + comment + test (dual-passphrase locked as intentional). M1 resolved by added test. No blockers.
