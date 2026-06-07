# Phase 1 (applied-edits) Review — api-harvest-drift-fix

> Persisted verbatim from the opus `feature-dev:code-reviewer` agent
> (`agentId: a6608d432022aa12b`). Reviewed commit `5dfb981` (5 docs files).

---

## VERDICT: 0 Critical / 0 Important (+ 2 Minor) — cleared to ship.

## Verified Clean
1. **Distinctness rewrite behavior-accurate** (`42-anti-collision-invariants.md:115-119` + `54-…:72,738` + transcript `:469`): both layers typed (`bundle.rs:429` `slots[i].path==slots[j].path`; `parse_descriptor.rs:1208/1212` `cs[i].path==cs[j].path`); `path_raw` deletion framed as historical; lone residual lag = `error.rs:13-16` (verified still stale raw-string); dropped bifurcation example gone; no new falsehood.
2. **No `path_raw` as active field** — only 2 historical "deleted path_raw" mentions (`42-…:119`, `54-…:739`).
3. **Line refs** — spot-checked ~20 against source HEAD, all correct (synthesize_descriptor :229 4-arg, synthesize_unified :745 6-arg w/ run_language, ResolvedSlot :642, build_descriptor :109, xpub_to_65 :98, …, delegation :826, error.rs:13-16, verify_bundle.rs:143, parse_descriptor.rs:1208).
4. **Completeness** — `verify_bundle.rs:98` / `synthesize.rs:1296` / `cmd/bundle.rs:572` / `, entropy, privacy_preserving)` all gone; the ONLY remaining `:593` is the intentional mermaid node `41-…:55`.
5. **dead_code lift** — `54-…:68-69` `synthesize_descriptor` in its own row as the live v0.47.1 delegation target; legacy variants remain the dead group.
6. **schema_version 7-site list** all verified.
7. **Scope** — docs-only, 5 files; §2f code FOLLOWUPs correctly deferred.

## Minor
**M1 — MORE pre-existing stale refs (different symbol class):** `42-anti-collision-invariants.md:24,71,91,141-144` cite `verify_bundle.rs:831-836` (`MappingFailure`, actually `:1527`), `:838-1277` (`emit_multisig_checks`, actually `:1533-2025`), `:895-947`, `:1194-1232`. verify_bundle INTERNAL-HELPER refs — outside the synthesize/distinctness scope, analogous to the deferred `bundle.rs:707/:724`. → FOLLOWUP (extend the line-ref audit to verify_bundle helper refs).
**M2 — lint + git-stat must be run before ff-merge** (review env had no Bash). [Operator: ran `make -C docs/technical-manual lint` → `[lint] OK`; `git show --stat 5dfb981` = exactly 5 files, no `.rs`.]

**Phase 2 cleared:** file the §2f FOLLOWUPs + the M1 verify_bundle-ref FOLLOWUP + flip `api-harvest-drift-on-synthesize-descriptor-signature` resolved → ff-merge to master (no tag).

---

## Operator note
M1 surfaced a THIRD residual class (verify_bundle internal-helper line refs). Combined with the deferred mermaid `41:55` diagram regen + `bundle.rs:707/:724` chunk_set_id format refs, this confirms the technical-manual is BROADLY line-ref-drifted across many symbols (not just synthesize). Filing ONE comprehensive `technical-manual-residual-line-ref-drift` FOLLOWUP capturing all three deferred classes + recommending a CI-gated regeneration mechanism (the existing `api-surface-coverage.sh` is advisory-only and checks names, not line numbers). M2 gates run GREEN by operator.
