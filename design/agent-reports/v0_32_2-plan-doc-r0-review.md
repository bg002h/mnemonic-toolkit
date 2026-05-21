# v0.32.2 plan-doc R0 review (Cycle 16 — bsms-encryption-per-signer-tokens)

**Reviewer:** opus (feature-dev:code-reviewer)
**Round:** R0
**Plan:** `design/PLAN_mnemonic_toolkit_v0_32_2.md`
**Date:** 2026-05-21
**Source SHA:** `c25b272`

## Verdict

**YELLOW.** 0 Critical / 2 Important / 2 Minor. Pairing semantics sound + well-specified; citations hold; two real gaps.

## Citations

All 6 ranges verified. Round-2 block is L860-882 (plan said L861-879 — trivial). read_and_validate L1908-1926.

## Important (I)

**I1 — Gap (h): N>1 tokens + 0 `--bsms-round1` records is silently accepted.** `verify_bsms_round1_files` is only called when `!args.bsms_round1.is_empty()` (L277), so the `tokens.len() == records.len()` count check never fires for 0 records. If no Round-2 blob either, N>1 tokens are read + discarded silently — contradicting the risk register's "refuse." **Fold:** add an explicit early guard — if `args.bsms_encryption_token.len() > 1` AND `args.bsms_round1.is_empty()` → `BadInput("per-Signer tokens (N>1 --bsms-encryption-token) require N matching --bsms-round1 records; none supplied")`. Add a test cell.

**I2 — Round-2 vs Round-1 error precedence undocumented.** `verify_bsms_round1_files` (L277) runs BEFORE the Round-2 block (L860). So in N>1 + encrypted-blob, the Round-1 positional verify (incl. any per-record MAC failure) fires FIRST; the Round-2 multi-token refusal is only reached if Round-1 passes. The plan §40 doesn't state this. **Fold:** document "Round-1 errors take precedence; the multi-token-Round-2 refusal is reached only after Round-1 records verify."

## Minor (M)

**M1 — Append idiom inconsistency.** Plan specs `action = clap::ArgAction::Append`, but the cited mirror `--bsms-round1: Vec<PathBuf>` (L191-192) uses NO `action` (clap-derive auto-infers Append for `Vec`). Both work; for sibling consistency, OMIT the explicit `action` to match `--bsms-round1`.

**M2 — Missing test cells:** (a) gap (h) N>1 + 0 records refusal (per I1); (b) explicit per-record-i MAC-mismatch attribution (token[i] fails record[i] → error cites index i).

## Verified clear

- stdin with Vec (only one `-`; loop reads stdin once): sound.
- Backward-compat single-token path (`tokens.first()` / `tokens[0]`): byte-identical.
- MAC attribution: `decrypt_bsms_record` ctx `"--bsms-round1: encrypted record {i}"` cites index.
- SemVer PATCH (strictly more permissive): correct.
- GUI lockstep optional (schema_mirror.rs:52-53 flag-name-only): confirmed.

## Recommendation

Fold I1 (gap-h guard + cell) + I2 (precedence doc) + M1 (drop explicit action) + M2 (2 cells), then Phase 2.
