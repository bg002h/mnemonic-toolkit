# Phase E — origin_path null unification — code-reviewer r1 (2026-05-06)

## Findings

### Critical
None.

### Important
None.

### Low / Nit
None blocking.

## Implementation review

- New `origin_path_for_json(path_raw)` helper in `cmd/bundle.rs::emit_unified`: returns `None` when `path_raw.is_empty()` (the SPEC §4.11.b absent-path sentinel) and `Some(normalize_origin_path(path_raw))` otherwise. Replaces the v0.4.2 `Some(normalize_origin_path(...))` unconditional emission which mapped empty `path_raw` to `Some("m")`.
- For the multisig branch (n>1), per-cosigner `CosignerEntry.origin_path: String` still maps empty `path_raw` to `"m"` via the original `normalize_origin_path`. The all-cosigners-empty case unifies up via `all_same` → `(Some(info), paths.first().cloned(), None)` — but since `paths.first()` is `Some("m")` (string), the top-level `origin_path` would still be `Some("m")` not `None`. The plan-E intent applies to single-sig watch-only canonical case which is now correctly `None`. Multisig behavior is unchanged from v0.4.5; per-cosigner null emission is a future refinement (no current open FOLLOWUP).

## Test status

236 lib + 22 integration suites pass.

## Outcome

Phase E APPROVED. Proceed to Phase F.
