**Independent Codex documentation fold review R1 — 2026-10-30**

**Verdict: approve the fold and documentation deliverables. Critical: 0 · Important: 0 · Minor: 0.**

Reviewed toolkit `1f20c230641117ce4f2e637e20107f4ba91ee445..ce9a914859a4923089a75bbf2e7e19bed084ecae` and descriptor-mnemonic `4681e40e98f293804b9110485040dd33135f7a71..983ccff06ff3f010883098c3e98b3b81662b5db1` against the persisted R0 findings. The other five repositories remain at their R0-reviewed commits. Review was limited to finding resolution and defects introduced by the fold; settled architecture and scope were not re-audited.

- **I1 resolved.** [WORKFLOW.md:30](/scratch/code/shibboleth/architecture-worktrees/mnemonic-toolkit/docs/architecture/WORKFLOW.md:30) restores test-first implementation, independent review of every written artifact from specification onward, and re-review after every fold including the last. Lines 41–43 retain explicit repository overrides and expressly assign the inherited default to standalone mt.
- **I2 resolved.** [check.py:117](/scratch/code/shibboleth/architecture-worktrees/mnemonic-toolkit/docs/architecture/check.py:117) requires exactly the four boundary records and compares the SeedHammer SHA with both the recorded me baseline and HEAD gitlinks. The main routine invokes this check and records missing-boundary errors. Independent read-only probes passed the valid control and rejected a fabricated SHA, missing seed boundary, duplicate boundary, and simulated HEAD gitlink drift. Firmware contents and a populated SeedHammer checkout are unnecessary.
- **M1 resolved.** SOURCE_MAP.md lines 85–87 and OPEN_QUESTIONS.md line 11 now link the me-hosted wire-8 design at the recorded immutable revision, distinguish md implementation ownership, and avoid inferring historical gate completion. The md AGENTS reading route is updated.
- **M2 resolved.** README.md line 21, SOURCE_MAP.md line 90, and md AGENTS.md lines 34–36 correctly describe xtask generating the verdict table from recorded coordinator evidence.
- **M3 resolved.** All package rows and checker expectations consistently use `explicit_binary_declarations`. SOURCE_MAP.md lines 7–9 explicitly exclude automatically discovered targets and disclaim an exhaustive executable inventory.
- **M4 resolved.** [SOURCE_MAP.md:170](/scratch/code/shibboleth/architecture-worktrees/mnemonic-toolkit/docs/architecture/SOURCE_MAP.md:170) documents both boundary repositories, exact required revisions, supported sibling/parent layouts, and failure when required objects are unavailable. It clarifies that their HEADs need not match and that SeedHammer need not be checked out.

No fold-introduced defects found. The plan status accurately reflects the previously recorded plan R1 approval. The persisted documentation R0 report remains byte-for-byte unchanged.

Validation comprised diff/source inspection, consistency checks for the renamed inventory field, and targeted in-memory probes of the changed boundary validator. The committed validation record documents the complete checker PASS of 7 repositories / 35 hashes / 1 gitlink / 278 links / 13 Markdown files and the coordinator's negative fixtures, syntax and whitespace checks. Runtime suites were not run.

Post-push remote-ref verification remains the coordinator's delivery step. This approval is independent Codex documentation review, not Opus or runtime-release certification. Only this R1 report is written; no other files, commits, or remote refs are changed.
