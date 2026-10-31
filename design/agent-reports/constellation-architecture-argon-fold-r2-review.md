# Argon fold R2: correction review

**Verdict: corrections required — 0 Critical / 1 Important / 0 Minor.** I2 and I3 are closed. I1 remains open for uppercase commit IDs. No additional defect was found in the scoped correction diff.

Reviewed only the response to the findings preserved in `e0dae384`, the correction snapshot `a2e633b5384c6c61b3d9394379818b5ecd7d376d`, and subsequent fallback updates. Settled M1/M4 architecture and provenance facts were not reopened. All checked worktrees were clean on `docs/constellation-architecture`.

| Repository | Exact checked tip |
| --- | --- |
| mnemonic-secret | d46a23377a4f7cbe0a889dedd73ab4a6a0412168 |
| mnemonic-key | e04f9785328cb3b32bf2f84008aeb0e922697b9b |
| descriptor-mnemonic | 34b7c815355cd62602a1472217585051e6b023bd |
| mnemonic-toolkit | f635878083d490a188f617afcc98a22182e8a9b7 |
| mnemonic-transaction | 7a546fb9ef1dcd81c8286968bb07965a377b29c0 |
| mnemonic-engrave | 12a8cee0d70646d788b8718ca6deae7e8c6de8af |
| mnemonic-gui | c20b90677818ad39d9e732be5f4126d1bcc33c6a |

**I1, residual — Commit-shaped guards are case-sensitive.** `docs/architecture/check.py:199,272–274`; regression at `docs/architecture/test_check.py:143`.

Both the full-SHA dispatch guard and the configured-branch rejection use `[0-9a-f]`. Git also resolves uppercase commit IDs: a disposable fixture's uppercase documentation commit returned `commit` from `git cat-file -t`. With that uppercase value configured as `documentation_branch`, a source link outside the baseline still returned no checker errors. The existing new test only exercises lowercase values.

The CLI reproduction is also confirmed. A temporary inventory set `documentation_branch` to `0123456789ABCDEF0123456789ABCDEF01234567`, and an extra Markdown file linked that revision's `crates/mnemonic-toolkit/src/lib.rs`. The commit does not exist, but the checker exited **0** and reported **PASS: 7 repositories / 35 hashes / 1 gitlink / 4 documentation refs / 295 links / 14 Markdown files**. The uppercase spelling escapes both guards and reaches the tracked-working-tree shortcut. This is the original I1 bypass remaining open, not a new architecture concern.

**Correction:** recognize ASCII hexadecimal letters case-insensitively in both guards, including the SHA-leading branch-prefix case. Retaining lowercase-only canonical inventory/source-link spelling is compatible with rejecting ambiguous uppercase branch values. Add uppercase and mixed-case cases to the existing collision regression and assert a nonzero CLI result for the uppercase nonexistent-commit fixture.

**Closed findings and navigation:**

- **I2:** literal pathspec handling plus exact stage-0 regular-file membership closes the wildcard alias. The added test checks rejection of the untracked literal `[ab].md` and acceptance after staging that exact filename.
- **I3:** explicit symlink rejection, regular-file modes, and directory-descendant checks close the reported symlink/type mismatches. Tests cover symlink files and directories, rejection before staging a file-to-directory replacement, and acceptance afterward.
- All six sibling AGENTS files and the current `WORKSPACE_AGENTS.md` template point to `a2e633b5384c6c61b3d9394379818b5ecd7d376d` for both README and WORKFLOW. Current inventory metadata uses the same toolkit documentation pin. Toolkit's own AGENTS retains local links.
- The pinned README, source map, workflow, open questions, checker, and tests are byte-identical to those at the reviewed toolkit HEAD. The inventory change is solely the toolkit documentation pin; other provenance remains settled. The six sibling diffs are solely the fallback reference update.
- Pending workspace-root installation remains intentional and is not a finding.

Read the updated validation record and inspected the four added regressions. Already-passed suites were not repeated. Independent execution was limited to the unresolved case-sensitivity probe and its temporary-inventory CLI reproduction; temporary fixtures were cleaned up. No implementation edits, commits, full suites, firmware audit, or publication operations were performed. This report is the only persisted review change.
