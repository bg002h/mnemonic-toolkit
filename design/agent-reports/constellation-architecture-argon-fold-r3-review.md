# Argon fold R3: closure review

**Verdict: approved — 0 Critical / 0 Important / 0 Minor.** The residual I1 case-sensitivity finding is closed. No new defect was found in this scoped correction.

Reviewed only the uppercase/mixed-case and SHA-leading branch guards, their regressions, and mechanical fallback updates following the R2 report preserved in `52bf0c69`. The corrected shared-guide snapshot is `a619f6eefdc8a2c58c9ff8f290376ed371a1c677`. Closed I2/I3 and settled architecture/firmware findings were not reopened.

All seven worktrees were clean on `docs/constellation-architecture` at these exact checked tips:

| Repository | Exact checked tip |
| --- | --- |
| mnemonic-secret | fe70f0d2aaa01d5f77ad069a697cbf7c492676c2 |
| mnemonic-key | 65e6ffbcd3e58f2e6b3702a25dfb20182a8d309c |
| descriptor-mnemonic | c46c63b19d6618a7f9321155009fcc651ca3e23f |
| mnemonic-toolkit | cda3b0a485c7c0de99ab4d246aa5e28c59f6caf0 |
| mnemonic-transaction | 66a000195c36436ca244109de58876b76bb5284d |
| mnemonic-engrave | 8a0ffa3ebbe37438d6cebc2e36705dd50261a071 |
| mnemonic-gui | 68e8778d0a3ce1f33b9f0d8631d1166c37ec3398 |

Closure evidence:

- `check.py` now uses the same `COMMIT_REF` compiled with `re.IGNORECASE` in both URL dispatch and documentation-branch validation. Both examine the first path component. Uppercase/mixed-case SHA URLs cannot take the tracked-branch shortcut, and SHA-leading branch names are rejected before link checking. Canonical source/inventory SHA requirements remain lowercase.
- Literal `refs/heads/...` validation precedes `check-ref-format --branch`, preserving branch-name restrictions without accepting reflog shorthand through expansion. The updated collision regression includes an actual uppercase commit ID and nonexistent uppercase/mixed-case values, with documentation allowances removed so the negative assertions exercise the source-baseline boundary.
- All six sibling AGENTS files and the current workspace template pin `a619f6eefdc8a2c58c9ff8f290376ed371a1c677` for both README and WORKFLOW. Current inventory metadata records that same toolkit documentation snapshot. Toolkit's own AGENTS continues to use local navigation.
- The pinned README, WORKFLOW, checker, and regression tests are byte-identical to the reviewed toolkit HEAD. The mechanical reference update therefore points to the reviewed correction.

Read the appended R2-correction validation record: it records the failing uppercase/mixed-case cases before the fix, 19 passing regressions afterward, CLI rejection of lower/upper/mixed SHA names and SHA-leading branches, rejection of HEAD/reflog shorthand, and acceptance of master. The recorded architecture check remains 7 repositories / 35 hashes / 1 gitlink / 4 documentation refs / 294 links / 14 Markdown files. Already-passed suites were not repeated; no unresolved question required another execution probe.

The scoped review gate is closed. Pending workspace-template installation and publication remain delivery steps, not review defects. No implementation edits, commits, runtime suites, or fresh firmware/architecture audit were performed. This report is the only persisted review change.
