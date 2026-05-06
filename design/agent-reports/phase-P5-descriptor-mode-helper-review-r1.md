# Phase P.5 descriptor-mode helper rewrite — code-reviewer r1 (2026-05-06)

## Findings

### Critical
None.

### Important

**I-1: Plain-text output format diverges between descriptor-mode and template-mode (pre-P.5 inheritance).**
Descriptor-mode previously emitted `verify-bundle: {result}` header + `  - {name} [{ok|fail}]: {detail}` per check. Template-mode emits `{name}: {ok|fail} {detail}` per check + `result: {result}` trailer. SPEC §5.7 mandates same check schema but is silent on plain-text wire format. Divergence existed before P.5 (predates this cycle).

**Status:** addressed inline. Descriptor-mode plain-text aligned to template-mode's format.

### Low / Nit

**N-1: Stale comment at line 280 referencing "v0.2: schema_version 2; multisig array shape comes in Phase C".** Schema is "4" and Phase C shipped. Cleanup-only.

**Status:** addressed inline. Comment removed.

## Confirmed correct

1. `descriptor.n > 1` is a reliable multisig signal: `resolve_placeholders` enforces dense `0..n` placeholder set + non-empty input, so `n >= 1` always after parse_descriptor returns.
2. JSON envelope shape (`schema_version: "4"`, `result`, `checks`) byte-identical to template-mode emission.
3. SPEC §5.7 schema parity: both modes call `emit_verify_checks` with `is_multisig` derived identically. 9 / 3+6N schema unified.

## Outcome

P.5 APPROVED with I-1 + N-1 addressed inline (descriptor-mode plain-text aligned + stale comment removed). Closes FOLLOWUP `verify-bundle-9-3plus6n-descriptor-mode-parity`.
