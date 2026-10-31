# Constellation architecture and agent onboarding plan

Status: R0 reviewed (0 Critical / 0 Important); minor clarifications folded for re-review. Scope approved in the 2026-10-30 Codex conversation.
This task produces documentation and agent entry points; it does not change runtime code,
dependencies, firmware, wallet behavior, release configuration, or existing process policies.

## Outcome and scope

Maintain a compact developer guide that tells a new contributor where behavior lives,
what a change affects, which specifications constrain it, and how to find validation evidence.
Cover mnemonic-secret (ms), mnemonic-key (mk), descriptor-mnemonic (md), mnemonic-toolkit
(mnemonic), mnemonic-transaction (mt), mnemonic-engrave (me and shared host libraries),
and mnemonic-gui at lower priority. Inventory shared packages and dependency forks.
Refugium appears only at dependency/API boundaries. Gemini owns SeedHammer firmware
mapping and its separate documentation under mnemonic-engrave/design/firmware-architecture/.

## Baseline and ownership

Create isolated, sibling documentation worktrees for the seven repositories, each on
docs/constellation-architecture, from its refreshed origin/main or origin/master.
Record the full source SHA, branch, manifests, lockfile resolutions, patches, GUI pins,
and relevant source paths. Keep checkout revisions distinct from resolved dependencies.
Do not change another agent's checkout or include its untracked files.
The primary shared guide belongs in mnemonic-toolkit/docs/architecture/.
The unversioned parent workspace receives only a small navigation AGENTS.md with a
tracked template in the toolkit. Each repository receives its own tracked AGENTS.md.
Do not edit Gemini's firmware documents or its GEMINI.md.

## Phases and evidence

1. Inventory and reconciliation.
   Read current manifests, lockfiles, source entry points, specifications, decisions,
   relevant tests, and existing guidance. Deduplicate worktrees using Git common
   directories and distinguish active dependencies from historical clones.
   Identify stale statements as evidence-backed discrepancies, not new requirements.

2. Component and contract mapping.
   Trace the codec/CLI separation; toolkit bundle, recovery and identity operations;
   signed-transaction backup; host delivery and preview; shared I/O and wc-codec;
   and the GUI's library versus subprocess boundaries.
   Locate normative rules, implementation, test vectors, and cross-repo update owners.
   Inspect resolved dependency code when a consumer uses a different codec revision.
   Distinguish documented intent, observed implementation, and outstanding questions.
   Do not infer hardware validation or cryptographic correctness from a round trip.

3. Documentation.
   Write docs/architecture/README.md (target 1,500–2,500 words), SOURCE_MAP.md,
   OPEN_QUESTIONS.md, and a small machine-readable baseline inventory.
   The main guide has a repository/package map, a dependency diagram, representative
   flows, compatibility boundaries, and a change-to-owner table.
   SOURCE_MAP provides exact inspected revisions, key paths/symbols, dependency pins,
   authoritative documents, relevant checks, and the interface handoff to Gemini.
   Prefer links to existing manuals and specifications over duplicated explanations.
   Record unresolved conflicts with concrete evidence and resolution ownership.

4. Initialization.
   Add concise AGENTS.md files to all seven repositories and a tracked workspace
   template. Files state scope, reading routes, important invariants, existing
   cross-repo documentation obligations, and where project validation/workflow lives.
   Preserve applicable existing CLAUDE.md rules; do not silently replace required
   reviewers or invent blanket workflow policies. Historical status is evidence to
   recheck. Do not embed volatile commit pins or whole design histories in AGENTS.md.
   Support sibling checkouts and isolated worktrees; provide repository identity and
   a durable remote fallback when shared documentation is absent locally.
   Explicitly instruct the agent to read adjacent CLAUDE.md and applicable ancestor
   guidance; neither is automatically loaded by adding AGENTS.md. Provide a compact
   tracked WORKFLOW.md for inherited requirements, including mt without CLAUDE.md.
   Check reading routes from repo roots, shared-package directories, sibling
   worktrees, and standalone checkouts.

5. Verification and independent review.
   Validate new Markdown links, source paths, diagram relationships, baseline hashes,
   package names and resolved pins against the worktrees. Trace substantive claims
   to code and existing tests. Keep the guide navigation-oriented; use references
   to existing validation scripts instead of adding unexecuted executable recipes.
   Add a small standard-library documentation checker in docs/architecture/check.py.
   It checks all seven AGENTS files, shared Markdown navigation, baseline Git objects,
   manifests, resolved dependencies, and immutable source references against local
   repositories; explicitly distinguish this from remote HTTP and runtime checks.
   Exercise its failure reporting with a deliberately invalid temporary fixture.
   Persist its coverage/results in a validation record. Confirm delivered branch
   references after pushing; absent or skipped CI is not validation.
   Run documentation checks appropriate to changed files. No runtime changes are
   planned, so runtime suite results are not claimed by this task.
   A separate Codex context reviews the plan and later the whole documentation diff.
   Preserve reports verbatim before applying findings, in separate commits; resolve
   Critical/Important findings and re-review changed artifacts. Do not label this
   Codex review an Opus review or a replacement for code-release gates.
   A fresh-reader review must answer: where to edit a codec rule, which consumer pin
   is actually in use, which manual/GUI mirror must move, how mt/me meet, and where
   firmware ownership begins.

6. Delivery and maintenance.
   Commit only task-owned files, using each repository's configured identity.
   Push the seven documentation branches under existing protections; do not merge
   default branches or publish releases. Record branch URLs, commits, checks,
   pending questions, and how the guide is updated alongside contract changes.
   Install the small workspace navigation file only after its content is reviewed.
   Preserve a resumable task handoff. Source baselines stay dated snapshots; update
   them deliberately when dependencies or ownership changes, rather than pretending
   the guide automatically tracks moving branches.

## Acceptance

All seven repositories and discovered shared packages are accounted for. The guide
describes current implementation at recorded revisions, with clear boundaries for
Refugium and firmware. Important architectural claims have code/specification anchors.
Unresolved discrepancies are explicit. Entry instructions work from individual
repositories and the parent workspace without assuming automatic parent discovery.
Independent review has no open Critical/Important findings for the documentation.
No existing tracked changes or untracked files are absorbed into task commits.

## Review limitations

Source inspection and document validation do not establish Bitcoin consensus
conformance, secret erasure, hardware behavior, or a successful full build/test run.
Existing project-specific implementation/release requirements remain applicable to
future code work. This task uses the user-approved Codex documentation workflow;
any model-specific formal gate that remains unmet is identified as such.
