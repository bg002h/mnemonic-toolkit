# PLAN R0 review — ms1-repair-demote-to-candidate — round 3

**Verdict: GREEN (0 Critical / 0 Important / 0 Minor)**
**Reviewer:** Fable architect (funds-weighted, read-only), per user directive. Plan rev-3 @ toolkit `f7d011eb`.
**Dispatched:** 2026-07-09 (Cycle F, plan-R0 loop round 3 — convergence). Persisted verbatim per CLAUDE.md.

The sole open Important from round 2 is folded correctly, no regression elsewhere.

## Exit-1 fold confirmation
- **cell_19** (`:52`): now **exit 1** with the correct citation (`Codex32` invalid-checksum → `ms_codec_exit_code`⇒1, `error.rs:434-436`), tagged "NOT 4, NOT 2" — matches the live-run ground truth (`error: ms1 codex32: invalid short checksum` → exit 1).
- **cell_18b** (`:53`): now **exit 1** "(same `Codex32`⇒1 path)".
- **§5.3** (`:28-30`): parenthetical corrected to "exit 1 … NOT exit 4" — consistent with the inventory.
- **cell_24** (`:54`): correctly pins no exit (no-short-circuit + advisory).
- cell_9=exit 4 (`:51`), cell_27=exit 0 (`:55`), cell_30=exit 0 (`:56`) — unchanged, re-confirmed. No other stale number.

## No-regression scan
Re-checked all round-2-clean items intact in rev-3: I1 phase-merge (P0 rewires both verify-bundle ms1 sites via pure `repair_card`, advisory kind-gated Ms1 @`repair.rs:1701`, standalone-inline-only, mk1 preserved — G4); §5.6/5.7/5.8 homing; M1 (redaction `diff_byte_offset:None` + Zeroizing + both-substring §8.6 scan); M3 (CHANGELOG head `[0.80.0]`); M4/G9 (ms-cli `RepairJson.verdict` at D27-identical position); 4-site ms-cli sibling-pin advance (`manual-gui.yml:165` excluded); G1-G9. All unaffected.

## Gate status
Plan-R0 converged 0C/0I over three rounds (r1 0C/2I/4M → r2 0C/1I → r3 0C/0I). The plan faithfully executes the R0-GREEN SPEC and is implementable without drift. **CLEARED to begin P0 implementation** under per-phase TDD + per-phase R0 (FULL `cargo test -p`) + post-impl whole-diff.
