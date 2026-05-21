# v0.32.2 end-of-cycle architect review

**Reviewer:** opus (feature-dev:code-reviewer)
**Round:** end-of-cycle
**Cycle:** Cycle 16 (bsms-encryption-per-signer-tokens)
**Date:** 2026-05-21
**Pre-tag SHA:** `21105a6` (Phase 2-4; Phase 5 uncommitted)

## Verdict

**GREEN.** All 10 verification items pass. 0 Critical / 0 Important / 1 sub-80 non-blocking note. (Security-adjacent: BIP-129 per-token MAC verify.)

## Checks

1. **Pairing logic**: `verify_bsms_round1_files` validates `tokens.len() == paths.len()` + all-records-encrypted UPFRONT, so `tokens[i]` is index-safe. Per-record selection: 0→error, 1→tokens[0], _→tokens[i]. stdin records skipped in the probe then refused in the main loop.
2. **Gap-h guard** (N>1 + 0 records): before token read; reachable (clap forces a blob present; guard runs before blob processing). Test supplies a blob to confirm runtime-guard reach.
3. **Multi-token + encrypted-blob refusal**: Round-1 verify runs before Round-2 block → per-record MAC failures fire first (precedence holds).
4. **Single-token backward-compat**: `tokens[0]` / `1 => &tokens[0]` reproduce v0.32.1 byte-for-byte.
5. **stdin generalization**: `>1` token `-` refused; `blob=- AND any token=-` refused; one `-` read once.
6. **MAC verify per token**: `decrypt_bsms_record` uses the paired `token.hex` only; per-record-i MAC-mismatch cell confirms token[1]↔record[1] isolation (exit 2). No cross-token confusion.
7. **No new ToolkitError variants / no flag-NAME change**: reuses BadInput + BsmsMacMismatch; flag name unchanged (only cardinality). PATCH + GUI-optional rationale holds.
8. **Test coverage**: 8 cells cover every pairing combo + gap-h + per-record MAC + count-mismatch + mixed-mode + multi-token-blob + two-stdin. Adequate.
9-10. **Release tooling + regression**: Cargo.toml/install.sh/CHANGELOG all 0.32.2; single-token suite (17 encrypted + 15 round1) green.

## Non-blocking note (sub-80; not gating)

The per-Signer all-encrypted probe reads each non-stdin record file once, then the main loop re-reads it — a benign double file-read (small local files; no stdin contention since `-` is skipped). Optional future micro-opt; intentionally NOT fixed (avoids added complexity beyond the task).

## Cleared for tag.
