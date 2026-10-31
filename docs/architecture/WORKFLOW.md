# Workflow and instruction loading

This is a portable reading route and compact summary of inherited constellation
workflow. It does not replace a repository's detailed rules, expand the task's scope,
or grant publication authority. Current user instructions and existing task
authorization take precedence over local guidance.

## Load the applicable instructions

From the target repository root, explicitly read its CLAUDE.md when present and any
applicable workspace/ancestor guidance. If the repository has STANDARD_WORKFLOW.md,
read that detailed policy. Merely adding AGENTS.md does not load other file formats,
and an instruction above a separate Git root cannot be assumed to load automatically.
Paths in a repository's AGENTS file are relative to that repository root, including
when starting inside a shared crate or using a worktree.

In this workspace the inherited policy is `/scratch/code/CLAUDE.md`; read it when
available. A standalone clone must not depend on that absolute path existing.
The portable essentials below cover that case. mnemonic-transaction currently has
no repository CLAUDE file; also read its design/PROVENANCE.md for spec/vector ownership.

The [architecture guide](README.md) is shared in mnemonic-toolkit. Try the sibling
checkout first; repository AGENTS files also provide a remote fallback. Documentation
branches keep that fallback on `docs/constellation-architecture` until integration.
When merging the guide into the default branch, update the fallback links together
before deleting its documentation branch.

## Inherited essentials

For substantial development, follow brainstorm → specification → plan → phased
implementation → whole-diff review → delivery. Scope ceremony to the work, but
preserve applicable gates. Independent review means author and reviewer are different
contexts for the artifact under review. Resolve Critical/Important findings before
advancing the relevant gate; required validation must also pass.

Persist a review verbatim under `design/agent-reports/` **before** folding it.
Persist and fold are separate commits. Record what the check actually covered.
Run machine-checkable executable material before review; repeat the relevant checks
after a fold and follow the repository's re-review policy. Do not silently replace a
required reviewer/model or call self-review independent. Report unavailable formal
gates accurately.

Existing policy applies independent orchestration to funds/keys/addresses, normative
codec behavior, irreversible actions and work spanning many files/repos; the later
risk-scoped policy is recorded in mnemonic-engrave/CLAUDE.md. Routine work can stay
solo where applicable. The 2026-10-30 architecture task specifically uses independent
Codex documentation review, without asserting an Opus or runtime-release approval.

Assign follow-ups to the phase that owns them and resolve gating/phase-owned work
before that phase closes. The operator's inherited severity ruling treats secret
leakage findings as non-gating follow-ups; it does not stop finding or recording them.
Wrong results, data loss, funds-safety regressions, unmet guarantees and false claims
of validation remain blocking. This guide introduces no new severity policy.

For shared normative Rust/Go codec changes, Rust is primary and supplies vectors.
For a suspected Go-only convergence defect, check the Rust counterpart before making
that classification. Firmware/device work remains outside this architecture task.

## Validation and delivery

Use current CI/workflow files for actual suites and toolchains; historical prose and
test counts can be stale. Preserve independent vectors/oracles. Do not substitute
self-generated codec vectors for their independent source, or a round trip for
identity/address evidence. Follow repository test parallelism and debug-assertion
requirements. Avoid duplicating a passed suite without a new reason.

Use isolated worktrees when concurrent work warrants them; preserve unrelated files.
Stage explicit task-owned paths. Check the target repository's current branch policy
and prescribed staging script before a default-branch push; one repository's policy
does not establish another's. Respect existing authorization without repeatedly
asking for actions already covered. Tags, releases, merges and hardware operations
must stay within the authorized task.

For this documentation change, the [checker](check.py) covers links and the recorded
source inventory; independent review covers the architectural claims. No runtime
suite, hardware, consensus, or erasure result is implied. See the
[plan](../../design/PLAN_constellation_architecture_guide.md) for its acceptance scope.
