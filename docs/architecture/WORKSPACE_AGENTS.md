# Constellation workspace

This directory contains independent Git repositories and scratch worktrees.
Before editing, identify the target Git root and branch, check its status, and read
its AGENTS.md plus CLAUDE.md and applicable ancestor guidance explicitly.
Do not treat duplicate worktrees or scratch clones as additional product repositories.

The architecture entry point is `mnemonic-toolkit/docs/architecture/README.md`.
While the documentation branches are awaiting integration, use
`architecture-worktrees/mnemonic-toolkit/docs/architecture/README.md`.
A standalone fallback is the [shared guide](https://github.com/bg002h/mnemonic-toolkit/blob/docs/constellation-architecture/docs/architecture/README.md).
Read its [workflow](https://github.com/bg002h/mnemonic-toolkit/blob/docs/constellation-architecture/docs/architecture/WORKFLOW.md) and the relevant source-map section.

For the current architecture project:
- In scope: mnemonic-secret (ms), mnemonic-key (mk), descriptor-mnemonic (md),
  mnemonic-toolkit, mnemonic-transaction (mt), mnemonic-engrave host/shared packages.
- Lower priority: mnemonic-gui, the utility GUI.
- Refugium is included only at shared dependency/API boundaries.
- General mr/mr-gui and SeedHammer firmware implementation are outside scope.
  Gemini owns mnemonic-engrave/design/firmware-architecture/ and its GEMINI.md.

Architecture work is isolated under `architecture-worktrees/<repository>` on
`docs/constellation-architecture`. Read each worktree's AGENTS.md for this task.
These instructions are not automatically installed into the original checkouts;
after integration, use each repository's tracked root entry point.
Preserve other agents' work, unrelated untracked files, default branches and releases.
Do not read real wallet secrets or SSH key contents for documentation work.
