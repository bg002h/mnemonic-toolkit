# Independent execution review: Argon architecture fold

**Verdict: corrections required — 0 Critical / 3 Important / 0 Minor.**

Reviewed only the M1–M4 fold against the original delivered tips, including the checker, tests, metadata, navigation, firmware status wording, and validation record. All seven worktrees were clean on `docs/constellation-architecture`. Toolkit HEAD remained `26a3472cf98746191e5d44ceaa69963c3967b25c` through the final check.

| Repository | Original delivery | Reviewed HEAD |
| --- | --- | --- |
| mnemonic-secret | 54a903d4 | 9f697876 |
| mnemonic-key | 79d02465 | 34633058 |
| descriptor-mnemonic | 983ccff0 | 109a6aff |
| mnemonic-toolkit | 6bc1c940 | 26a3472c |
| mnemonic-transaction | 3a46dd78 | a5d70908 |
| mnemonic-engrave | 00fde290 | 77300485 |
| mnemonic-gui | f09047b7 | 94ad1a27 |

**I1 — A SHA-shaped documentation branch bypasses immutable source validation.** `docs/architecture/check.py:196–212,254`.

The configurable branch is matched before the immutable revision is parsed. Git accepts a 40-hex string in `check-ref-format --branch`, so setting `documentation_branch` to a commit-shaped value sends matching URLs through working-tree validation and skips both the recorded-source constraint and Git-object existence check.

Confirmed with a temporary inventory setting `documentation_branch` to `0123456789abcdef0123456789abcdef01234567` and an extra Markdown link to that revision's `crates/mnemonic-toolkit/src/lib.rs`. The commit does not exist locally, but the actual checker CLI exited **0**, reporting **PASS: 7 repositories / 35 hashes / 1 gitlink / 4 documentation refs / 295 links / 14 Markdown files**. In the synthetic fixture, an undeclared documentation revision's `src/lib.rs` likewise changed from rejection to acceptance solely by changing this field.

**Correction:** reject commit-shaped documentation branch values, or classify full-SHA URLs before considering the branch shortcut. Add a regression asserting nonzero CLI status for the nonexistent-commit case and rejection of a nonbaseline source path, while retaining ordinary configurable branches such as `master`.

**I2 — Pathspec expansion lets an untracked literal file pass the tracking check.** `docs/architecture/check.py:207`.

`git ls-files --error-unmatch -- relative` interprets `relative` as a pathspec; `--` does not make it literal. In a disposable fixture, staging `docs/a.md` and leaving the literal file `docs/[ab].md` untracked caused a branch blob link to `docs/[ab].md` to return no checker errors. The Git query returned `docs/a.md`, not the requested filename. The existing existence/kind checks inspect the untracked literal file, so the two checks jointly accept a target that is not in the deliverable.

**Correction:** use literal pathspec semantics and verify the exact indexed pathname for blob targets. Keep directory membership component-aware. Add the tracked `a.md` versus untracked `[ab].md` negative case, followed by a positive case after staging the literal filename.

**I3 — Filesystem kind and index membership can refer to different Git object kinds.** `docs/architecture/check.py:201–211`.

The kind check follows filesystem links, while the tracking check only asks whether any index entry matches. Two targeted fixtures returned no checker errors for invalid tree links:

- A staged `docs/alias` symlink to `docs/architecture` passes as a tree because `Path.is_dir()` follows it; its index mode is **120000**, a symlink blob, not a directory tree.
- Replacing tracked `docs/architecture/README.md` in the working tree with a directory containing only untracked `scratch.md` also passes a tree link. The index still contains the original **100644** file and has no descendants under `README.md/`.

**Correction:** reconcile kind with literal index entries as well as filesystem state. A tree target needs indexed descendants under its directory prefix, not an exact file/symlink entry. Reject unsupported symlink targets or validate them with explicit Git semantics; do not follow them implicitly for kind or fragment validation. Add both negative fixtures and retain the existing staged-file/containing-directory positive test.

These are false acceptance paths in the guarantees this fold adds, so they require correction before the execution gate closes. They do not establish runtime or firmware defects.

The remaining scoped checks were satisfactory:

- The R0 minor is folded: `test_untracked_then_staged_file_and_directory` checks rejection before staging and acceptance of both links without a commit.
- Outside I1's branch-dispatch bypass, documentation exceptions remain restricted by repository, full commit, and component-bounded path. The entire preexisting source/boundary inventory is unchanged, including the preview gitlink.
- All six sibling fallbacks and the workspace template pin `99b4801cb9fa14b17d48d7b11dc793112eb2bafd`. Its README, source map, workflow, open questions, checker, and tests match the reviewed HEAD. The two-commit bootstrap and subsequent reference update are coherent. Retain/repin guidance and offline local-object prerequisites are documented without a GitHub retention guarantee.
- The me implementation links are added. Firmware submission and review links remain pinned separately, with an explicit corrections-required status and no adoption of the disputed Go-port summary. No firmware audit was repeated.
- The workspace root installation is intentionally pending review; its current difference from the template is not a finding.

Read `constellation-architecture-argon-fold-validation.txt` and inspected its test coverage; did not rerun the completed suites. Independent execution was limited to disposable Git probes for the findings above and one temporary-inventory CLI reproduction for I1. No implementation edits, commits, network/signature preflight, runtime suites, or physical operations were performed. Temporary fixtures were cleaned up. This report is the reviewer's only persisted change.
