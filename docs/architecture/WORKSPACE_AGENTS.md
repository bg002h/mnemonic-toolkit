# Constellation and Refugium workspace

This directory contains independent Git repositories and scratch worktrees.
Before editing, identify the target Git root and branch, check its status, and read
its AGENTS.md plus CLAUDE.md and applicable ancestor guidance explicitly.
Do not treat duplicate worktrees or scratch clones as additional product repositories.

The constellation entry point is `mnemonic-toolkit/docs/architecture/README.md`.
While the documentation branches are awaiting integration, use
`architecture-worktrees/mnemonic-toolkit/docs/architecture/README.md`.
A standalone fallback is the [shared guide](https://github.com/bg002h/mnemonic-toolkit/blob/a97b6176646baee1824e7cade2c11b2a17bcbb8d/docs/architecture/README.md).
Read its [workflow](https://github.com/bg002h/mnemonic-toolkit/blob/a97b6176646baee1824e7cade2c11b2a17bcbb8d/docs/architecture/WORKFLOW.md) and the relevant source-map section.

The separate Refugium entry point is `mnemonic-refugium/docs/architecture/README.md`.
Until integration, use
`refugium-architecture-worktrees/mnemonic-refugium/docs/architecture/README.md`.
Standalone fallbacks: [Refugium guide](https://github.com/bg002h/mnemonic-refugium/blob/cdf7649c93f27b2bb9c6d03670c960b072ae5cdc/docs/architecture/README.md),
[workflow](https://github.com/bg002h/mnemonic-refugium/blob/cdf7649c93f27b2bb9c6d03670c960b072ae5cdc/docs/architecture/WORKFLOW.md)
and [onboarding prompt](https://github.com/bg002h/mnemonic-refugium/blob/cdf7649c93f27b2bb9c6d03670c960b072ae5cdc/docs/architecture/ONBOARDING.md).
The dated source maps distinguish repository HEADs from consumer lockfile pins.

The maintained maps cover:
- Constellation: mnemonic-secret (ms), mnemonic-key (mk), descriptor-mnemonic (md),
  mnemonic-toolkit, mnemonic-transaction (mt), mnemonic-engrave host/shared packages.
- Lower priority: mnemonic-gui, the utility GUI.
- Refugium: mnemonic-refugium (mr CLI and mr-codec), refugium-wallet
  (mr-gui, refugium-view and refugium-node). mr-gui is not a third repository.
- SeedHammer firmware is a separate handoff. Gemini owns
  mnemonic-engrave/design/firmware-architecture/ and the firmware GEMINI.md;
  read the recorded review status before relying on the pinned submission.

Constellation documentation is isolated under `architecture-worktrees/<repository>`
on `docs/constellation-architecture`; Refugium documentation is under
`refugium-architecture-worktrees/<repository>` on `docs/refugium-architecture`.
Read each worktree's AGENTS.md and the corresponding delivery receipt.
These instructions are not automatically installed into the original checkouts;
after integration, use each repository's tracked root entry point.
Preserve other agents' work, unrelated untracked files, default branches and releases.
Do not read real wallet secrets or SSH key contents for documentation work.
