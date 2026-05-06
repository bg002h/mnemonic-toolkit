# Phase F — text-mode trailing-space fix — code-reviewer r1 (2026-05-06)

## Findings

### Critical
None.

### Important
None.

### Low / Nit
None blocking.

## Implementation review

Three identical `writeln!(stdout, "{}: {} {}", c.name, status, c.detail)` emit sites in `cmd/verify_bundle.rs` (template-mode single-sig at line 255, template-mode multisig at line 289, descriptor-mode at line ~607) all rewritten to branch on `c.detail.is_empty()`:

```rust
let status = if c.passed { "ok" } else { "fail" };
if c.detail.is_empty() {
    writeln!(stdout, "{}: {}", c.name, status).ok();
} else {
    writeln!(stdout, "{}: {} {}", c.name, status, c.detail).ok();
}
```

This produces `"md1_xpub_match: skipped"` (no trailing space) when detail is empty, and `"md1_decode: ok decoded successfully"` when populated.

## Test status

236 lib + 22 integration suites pass.

## Outcome

Phase F APPROVED. Proceed to Phase A.
