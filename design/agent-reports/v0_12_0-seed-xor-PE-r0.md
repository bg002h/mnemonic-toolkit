# v0.12.0 PE — release rollup reviewer report

**Phase:** PE — release rollup (toolkit v0.11.0 → v0.12.0)
**Round:** R0 (single-pass per plan §Phase E)
**Reviewer:** Opus (`feature-dev:code-reviewer`)
**Date:** 2026-05-14
**Commit under review:** `8836c2a` (PE rollup)
**Predecessor tag:** `mnemonic-toolkit-v0.11.0` (final-word completer at `f6c036a`, 2026-05-14)

## Verdict

**0 Critical / 0 Important / 0 Nice-to-have — PE LOCK.**

(The reviewer's first-pass flagged 1 Critical on `cli_gui_schema.rs`
appearing untracked; verified post-review as a false alarm — reviewer
was reading a stale session-start `git status` snapshot. `git ls-files
crates/mnemonic-toolkit/tests/cli_gui_schema.rs` returns the file as
tracked; `git log` shows it was added in v0.11.0 P2 commit `48b4488`
and modified in v0.12.0 P2 commit `48241e7`. Current `git status
--short` shows only `.claude/` + `.v0_2-plan-stash.md` (out-of-scope
local artifacts), no other untracked files. C-1 discounted to
verified-non-issue, same pattern as v0.11.0 PE.)

Tag `mnemonic-toolkit-v0.12.0` cleared to push pending user authorization.

## Scope reviewed

All 11 mandatory PE checks:
- Critical: version-bump consistency; CHANGELOG enumeration accuracy;
  SemVer correctness; FOLLOWUPS closure correctness.
- Important: no premature SHA citation; no untracked work; tests +
  linters green; R1 reports persisted; cross-repo coherence; CI matrix
  drift.
- Nice-to-have: CHANGELOG header date.

Files reviewed:
- `crates/mnemonic-toolkit/Cargo.toml:3` (0.11.0 → 0.12.0)
- `Cargo.lock:592-593` (matches)
- `CHANGELOG.md:309+` (new v0.12.0 section)
- `design/FOLLOWUPS.md:71` (seed-xor-coldcard-compat resolved)
- 4 per-phase R1 LOCK reports

## Key validations

1. **Version bump consistent across Cargo.toml + Cargo.lock.** Both
   show `0.12.0` under the `mnemonic-toolkit` package block.

2. **CHANGELOG enumerates every shipped artifact.** Library module +
   CLI subcommand (split + combine) + new advisory class (multi-secret-
   on-stdout) + 15/21-toolkit-only advisory + permission-mode advisory
   + manual chapter + cli-subcommands.list rows + glossary count fix +
   lint anchor deltas (21→23 + 1 zeroize row) + cli_gui_schema bump +
   JSON envelope SHA pins + deps (rand_core + rand_chacha) + resolved
   FOLLOWUP.

3. **SemVer minor bump justified.** Purely additive: new `Command`
   variant, new library module, new advisory class. No breaking changes
   to existing surfaces; no flag renames; no library re-exports removed.

4. **FOLLOWUPS closure properly cites the 4 R0/R1 LOCK reports.**
   Companion `slip39-shamir-secret-sharing` correctly remains `open`
   (v0.13.0 hasn't started).

5. **No premature SHA citation.** FOLLOWUPS closure cites the tag name
   but no SHA — follows v0.10.1/v0.11.0 precedent of adding the SHA in
   a post-tag follow-on commit.

6. **CI workflow drift.** `.github/workflows/rust.yml` retains 5
   top-level jobs (test × {ubuntu, macos} = 2 matrix entries + miri +
   clippy + test-release-mlock-einval + g6-invariant) = 6 effective CI
   runs + manual.yml. Matches v0.11.0 baseline.

7. **All R1 LOCK reports persisted at cited paths** with 0C/0I clean
   round 1 verdicts.

## R0 single-pass LOCK

v0.12.0 PE R0 LOCK. Tag `mnemonic-toolkit-v0.12.0` cleared to push
pending user authorization. Post-tag actions per v0.11.0 precedent:
1. After tag push, commit a FOLLOWUPS-SHA refresh citing the actual
   tag-commit SHA.
2. Smoke-test commands from plan §A verification (Step 1-6) post-tag.
3. CI verification on tagged commit.
