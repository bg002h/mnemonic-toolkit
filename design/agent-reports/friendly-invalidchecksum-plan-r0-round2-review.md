# R0 round-2 architect review — PLAN_friendly_invalidchecksum_redaction (2026-06-10)

Reviewer: Fable 5 architect agent (R0 round 2, post-fold). master @ f6023b1. Verdict: GREEN (0 Critical / 0 Important / 2 carry-over Minors, no action). Review verbatim below.

---

## Critical
None.
## Important
None.

## Minor

**M1 (carry-over, informational)** — under `MNEMONIC_FORCE_TTY=1` (repo-suite default) most single-char corruptions trigger the repair short-circuit (exit 5); T1's mandated `--no-auto-repair`/`FORCE_TTY=0` gate sidesteps it (any single-char data corruption renders the leak at exit 1 with repair disabled — verified live, both gate variants). Strengthens I1; no plan change.

**M2 (carry-over from r1 M5, no action)** — repair.rs:862-865 latent Debug-dump confirmed present; `InvalidChecksum` unreachable there (post-correction decode runs only after residue==0 ⇒ valid checksum). Non-goals correctly exclude it.

## Fold-verification

**I1 — FOLDED CORRECTLY.** Plan line 16 mandates `--no-auto-repair` OR `env("MNEMONIC_FORCE_TTY","0")`, asserts exit 1, explains the FORCE_TTY=1 hazard. `--no-auto-repair` confirmed a real global clap flag (main.rs:86-87 `#[arg(long, global = true)]`); `resolve_no_auto_repair` = `no_auto_repair || !tty` (repair.rs:401-407) forces repair-off regardless of FORCE_TTY; convert honors it at convert.rs:989. Both gate variants render the leak at exit 1 (pre-fix, live).

**I2 — FOLDED CORRECTLY.** Plan line 7: 16 variants, 6 explicit → TEN catch-all, listed exactly. Cross-checked codex32-0.1.0 lib.rs:42-83; the 6 explicit (friendly.rs:58-78) = ThresholdNotPassed/RepeatedIndex/Mismatched{Length,Hrp,Threshold,Id}; **only InvalidChecksum embeds a String** (field::Error carries ExtraChar(char)/NotAByte/InvalidByte; Fe=Fe(u8); Case fieldless; rest char/usize). One arm suffices.

**M1(r1)/M2(r1)/M3(r1)/M4(r1) — all FOLDED** (broader leak-path note; T2 asserts string-absent + standalone cell + !contains discipline; full-withholding divergence note; 5 version sites verified at cited lines).

## Verdict
**GREEN — 0 Critical / 0 Important.** No fold-drift; the 10-variant enumeration is exact, `--no-auto-repair` renders the leak at exit 1, the FORCE_TTY=1 repair-diversion hazard validated. Cleared to implementation (TDD red-first).
