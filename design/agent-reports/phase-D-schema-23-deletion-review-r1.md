# Phase D — schema-2/3 placeholder rejection deletion — code-reviewer r1 (2026-05-06)

## Findings

### Critical
None.

### Important
None.

### Low / Nit
None blocking.

## Implementation review

- `load_bundle_json_into_args` (cmd/verify_bundle.rs): the `serde_json::Value` peek + `schema_version` rejection branch deleted (~16 lines including the placeholder error message pointing at FOLLOWUP `bundle-json-schema-2-3-retro-compat`). Doc-comment updated to note v0.5 behavior: schema-mismatch envelopes fail at the underlying field extraction.
- Existing schema-3 fixture test (`verify_bundle_via_bundle_json_unsupported_schema_rejected`) renamed to `verify_bundle_via_bundle_json_schema3_envelope_fails_at_field_extraction_v0_5` and reasserted to expect failure at the `ms1 field is not an array` check (the schema-3 envelope has `ms1` as a flat string, not an array).

## Test status

236 lib + 22 integration suites pass.

## Outcome

Phase D APPROVED. Proceed to Phase E.
