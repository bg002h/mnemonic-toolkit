# v0.27.0 Phase 1 R0 review — InspectEnvelope + runbook move

**Phase:** 1 (per `design/PLAN_v0_27_0_bsms_round_trip_and_wallet_import_handoff.md` §4.1)
**Reviewer:** opus / feature-dev:code-reviewer
**Verdict:** GREEN (0 Critical / 0 Important / 2 Minor / 8 Praise)
**Date:** 2026-05-18

---

## Scope reviewed

1. `InspectEnvelope` wrapper struct in `crates/mnemonic-toolkit/src/cmd/inspect.rs` with `INSPECT_SCHEMA_VERSION = "1"` constant + 3 unit cells (ms1/mk1/md1). Mirrors `XpubSearchEnvelope` precedent at `src/cmd/xpub_search/mod.rs:111-116`.
2. RepairEnvelope intentionally NOT shipped — `RepairJson` at `cmd/repair.rs:155+178` already carries `schema_version: "1"` inline (plan §3.3 R3 fold).
3. Integration cell `inspect_json_envelope_schema_version_v_0_27_0` in `tests/cli_inspect.rs` covering all three kind variants on real `mnemonic inspect --json`.
4. Relocated `.v0_26_0-merge-plan.md` → `design/PLAN_v0_26_0_three_way_merge.md` (plain `mv`; file was untracked) with canonical-record header.
5. `CLAUDE.md` Conventions cross-cite added at line 33.
6. Presence-smoke `tests/design_artifacts_presence.rs::three_way_merge_runbook_lives_in_design_dir`.
7. Both FOLLOWUPS `Status: open` → `Status: resolved` flipped in-commit (`inspect-json-schema-version-backfill` + `coordinator-runbook-into-design-dir`).

---

## Files examined

- `crates/mnemonic-toolkit/src/cmd/inspect.rs` (modified)
- `crates/mnemonic-toolkit/src/cmd/xpub_search/mod.rs` (precedent, unchanged)
- `crates/mnemonic-toolkit/src/cmd/repair.rs` (precedent, unchanged — confirms RepairJson already done)
- `crates/mnemonic-toolkit/tests/cli_inspect.rs` (new cell added)
- `crates/mnemonic-toolkit/tests/cli_auto_repair.rs` (verified no breakage to cell 25)
- `crates/mnemonic-toolkit/tests/design_artifacts_presence.rs` (new file)
- `design/PLAN_v0_26_0_three_way_merge.md` (relocated, header verified)
- `CLAUDE.md` (cross-cite added at line 33)
- `design/FOLLOWUPS.md` (Status flips verified at lines 54 + 2306)

---

## Critical: 0

None.

## Important: 0

None.

## Minor (informational, not blocking)

**m1. `InspectEnvelope` is private vs `XpubSearchEnvelope` public — divergence from plan-stated "mirror exactly", but justified.** `inspect.rs:252` declares `struct InspectEnvelope<'a>` (private); `xpub_search/mod.rs:112` declares `pub struct XpubSearchEnvelope` (public). Functionally identical for both use cases, but the divergence is justified: `XpubSearchEnvelope` is constructed across four sibling submodule files (`path_of_xpub.rs`, `account_of_descriptor.rs`, etc.), so it must be `pub`. `InspectEnvelope` is constructed only in `emit_inspect_json` (same file), so private is correct and tighter. Test still works because `mod inspect_envelope_tests` uses `super::*`. No action needed.

**m2. Test name `inspect_json_envelope_schema_version_v_0_27_0` uses underscores in the version segment** (`tests/cli_inspect.rs:132`). Slightly unusual naming; the codebase elsewhere prefers `v0_27_0`. Not load-bearing; passes rustc. Optional cleanup at next touch.

## Praise

**p1. FOLLOWUPS closure narrative is precise and honest.** The `inspect-json-schema-version-backfill` Status flip (line 54) explicitly calls out the latent FOLLOWUP-body inaccuracy (RepairJson already had `schema_version`) and ships `InspectEnvelope` only — matches plan §3.3 R3 fold verbatim. No false closure.

**p2. Per-phase FOLLOWUPS Status flip discipline executed in-commit.** Both flips (lines 54 + 2306) are in this commit, not deferred to Phase 6 — directly addresses memory `feedback-per-phase-agents-forget-followup-status-flip` and plan §4.0 generic instruction.

**p3. Presence-smoke is the right minimum.** `tests/design_artifacts_presence.rs:13` resolves path via `CARGO_MANIFEST_DIR/../../design` — correctly anchored against the workspace-crate manifest dir; survives future workspace reorgs. Helpful error message cites the FOLLOWUP slug + CLAUDE.md cross-cite for any future investigator.

**p4. Three serde unit cells (ms1/mk1/md1) at `inspect.rs:334-396` pin the `#[serde(flatten)]` + `tag = "kind"` shape comprehensively per kind variant.** Cheap regression guard against accidental shape breakage. Matches `XpubSearchEnvelope` precedent cells (`xpub_search/mod.rs:140-190`) in discipline.

**p5. Integration cell at `cli_inspect.rs:131-191` correctly asserts top-level `schema_version` + flattened `kind` for all three real-CLI paths** without disturbing cells 15-17 (text-form). Cell 25 in `cli_auto_repair.rs:254-290` continues to test the separate `RepairJson` envelope path (via `try_repair_and_short_circuit`) unchanged.

**p6. Relocated `PLAN_v0_26_0_three_way_merge.md` header is correctly canonical-record-shaped** (line 3 cites the closing FOLLOWUP slug + CLAUDE.md cross-cite + R3 reviewer-loop lineage). Plain `mv` was correct since the file was untracked; content preserved.

**p7. CLAUDE.md cross-cite (line 33) placed in the Conventions section** — correct location; matches plan §4.1 spec.

**p8. Downstream-consumer impact analysis is sound.** `#[serde(tag)]` internal-tagged enums in serde_json ignore unknown top-level fields by default during deserialization (only fail under `deny_unknown_fields`). An external parser of the v0.26.0 `InspectJson` shape will silently ignore the new `schema_version` field. mnemonic-gui has no current dependency on inspect's JSON envelope (confirmed by Grep across the repo). Safe additive change.

---

## Verdict

**GREEN — ship Phase 1 commit immediately.**

The InspectEnvelope shape mirrors the XpubSearchEnvelope precedent in all load-bearing ways (top-level `schema_version` + `#[serde(flatten)]` + inner `tag` discriminator), the schema_version literal `"1"` is consistent with both `XpubSearchEnvelope` and the already-shipped `RepairJson` field, FOLLOWUPS Status flips are honest and in-commit per the discipline lesson, the presence-smoke is correctly anchored, and the CLAUDE.md cross-cite landed in the right section. Test coverage is comprehensive (3 unit cells + 1 integration cell + 1 presence cell — 5 new cells total, all passing). No false positives in the closure narrative (the RepairJson-already-done callout is correctly preserved). Two minor stylistic nits noted (private-vs-pub envelope, test name underscore spelling) but neither is plan-violating or load-bearing.
