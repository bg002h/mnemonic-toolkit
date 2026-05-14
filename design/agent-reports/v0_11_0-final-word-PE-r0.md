# v0.11.0 PE — release rollup reviewer report

**Phase:** PE — release rollup (toolkit v0.10.1 → v0.11.0)
**Round:** R0 (single-pass per plan §Phase E)
**Reviewer:** Opus (`feature-dev:code-reviewer`)
**Date:** 2026-05-14
**Commit under review:** `11d0d49` (PE rollup)
**Predecessor tag:** `mnemonic-toolkit-v0.10.1` (Cycle B Path B-lite carve-out completion)

## Verdict

**0 Critical / 0 Important / 1 Nice-to-have (CHANGELOG editorial nit) — PE LOCK.**

Pre-tag gate clears. Tag `mnemonic-toolkit-v0.11.0` may be cut pending user authorization.

## Scope reviewed

All 12 mandatory PE checks: version-bump consistency, CHANGELOG
enumeration accuracy, SemVer correctness, SPEC narrative fix
completeness, FOLLOWUPS closure correctness, glossary count fix,
no-premature-SHA, no-untracked-work, tests + linters green, R1 reports
persisted, verification commands, lockstep coherence.

Files reviewed:
- `crates/mnemonic-toolkit/Cargo.toml:3` (0.10.1 → 0.11.0)
- `Cargo.lock:593` (matches)
- `CHANGELOG.md:309-391` (new v0.11.0 section)
- `design/SPEC_final_word_v0_11_0.md` §2.4 + §2.5 + §4 G4 (64 → 1)
- `design/FOLLOWUPS.md:57-64` (bip39-final-word-completer resolved)
- `docs/manual/src/60-appendices/61-glossary.md:149-156` (Five → Seven)

## Reviewer findings detail

### Critical: 0

### Important: 0

Raised in the reviewer's first-pass: an `Important I1` flagging
`crates/mnemonic-toolkit/tests/cli_gui_schema.rs` as untracked. This
was a false alarm caused by the reviewer reading a stale session-start
`git status` snapshot. Verified post-review by
`git ls-files crates/mnemonic-toolkit/tests/cli_gui_schema.rs` (tracked)
and `git log --oneline -- crates/mnemonic-toolkit/tests/cli_gui_schema.rs`
(added in commit `48b4488`, the P2 GREEN ship). File is included in
the test suite (P2 R1 LOCK already verified all tests green). I1
discounted to verified-non-issue.

### Nice-to-have: 1

**N1** — `CHANGELOG.md:377-380` glossary parenthetical is slightly
ambiguous. Reads "(also adds `gui-schema` to the previous pre-existing
drift — was actually six before this cycle)". A clearer phrasing
would be "(this cycle bumps the count by two: the pre-existing
`gui-schema` fold-in plus the new `final-word`)". Optional polish;
not blocking the tag.

## Key validations

1. **Version bump consistent across Cargo.toml + Cargo.lock.** Both
   files show `0.11.0` under the `mnemonic-toolkit` package block.

2. **CHANGELOG enumerates every shipped artifact.** Cross-checked
   against `git diff v0.10.1..HEAD --stat` and the 4 per-phase R1
   reports. Library, CLI, lint anchors, advisories, JSON envelope,
   GUI-schema test bump, manual chapter, SPEC narrative fix, glossary
   fix — all present.

3. **SemVer minor bump justified.** Purely additive: new `Command`
   variant, new library module, new advisory class. No breaking
   changes to existing surfaces; no existing flags renamed/removed;
   no library re-exports removed.

4. **SPEC narrative fix complete + grep-clean.** All §2.5 refusal
   rows, §2.4 exit-code table, and §4 G4 acceptance gate consistently
   say exit code `1` for `BadInput` refusals (with `64` reserved for
   clap parse errors). No stray `64` references in refusal context.

5. **FOLLOWUPS closure properly cites the 4 R1 LOCK reports.**
   Companion `library-error-and-language-surface-promotion` remains
   correctly `open` as a future-refactor candidate.

6. **Glossary count matches `Command` enum.** Both = 7.

7. **All R1 LOCK reports persisted at cited paths.** This PE pass is
   the 5th (spec-r0 + lib-r1 + cli-r1 + manual-r1 + PE-r0).

## Post-tag actions (note for user)

1. After `git push origin master` and `git tag mnemonic-toolkit-v0.11.0`
   + `git push origin mnemonic-toolkit-v0.11.0`, refresh the FOLLOWUPS
   closure to cite the actual tag-commit SHA (per the v0.10.1
   `resolved ed5a1d9` precedent).

2. Smoke-test commands from plan §"Verification" (Step 1-4) should run
   post-tag to confirm the binary behaves at the tagged version. Not
   gating for the tag push itself.

3. CI verification: `.github/workflows/rust.yml` matrix (test ubuntu +
   test macos + miri + clippy + test-release-mlock-einval + g6-invariant)
   + `.github/workflows/manual.yml` will run on the tag push. All should
   turn green based on pre-tag verification (full suite green, clippy
   clean, manual lint 6/6 OK).

## PE LOCK

0 Critical / 0 Important / 1 Nice-to-have. PE LOCK. Tag `mnemonic-toolkit-v0.11.0` cleared to push pending user authorization.
