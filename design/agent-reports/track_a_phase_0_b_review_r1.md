# Track A Phase 0.B review r1 — 42-md.md audit (commit 713178c)

Reviewed: `docs/manual/src/40-cli-reference/42-md.md`, 13 lines net.
Evidence: vector file, CHANGELOG, crates/md-cli/src/, appendix samples.

## Critical: 0
## Important: 0

## Low

### L1 — Stale "v0.19+" pointer in CLI error string (md1-repo issue, not a manual error)

The manual correctly says "caller-supplied internal-key support is deferred to
a future version" (lines 30, 156) without naming v0.19. That is accurate at
v0.32 HEAD.

However, `crates/md-cli/src/main.rs:224` still emits the user-visible error
string "track v0.19+ for caller-supplied internal-key support". v0.19 through
v0.32 shipped without that feature; the version reference is now stale. This
is an md1-repo issue that escaped the v0.31/v0.32 audit cycles, not a manual
bug. Filed as cross-repo FOLLOWUPS entry
`md-cli-unspendable-key-v0.19-error-string-stale` in
`descriptor-mnemonic/design/FOLLOWUPS.md` with companion
`md-cli-unspendable-key-v0.19-error-string-stale-companion` in this repo's
FOLLOWUPS.

### L2 — `(v0.18)` label on "Round-trip" section — defensible, no change needed

Line 199: `### Round-trip with explicit --path (v0.18)` was deliberately left
when `### Worked examples (v0.18)` was normalized. The round-trip section is
a historical narrative about a specific v0.18 bug fix (`--path` was silently
dropped pre-v0.18). The label serves as bug-fix provenance. The
worked-examples section is current reference content where a version tag is
noise. Asymmetry is intentional.

## Nit: 0

## Verification results

1. Encode phrase `md1yqpqqxqq8xtwhw4xwn4qh`: matches
   `bg002h/descriptor-mnemonic/crates/md-codec/tests/vectors/wpkh_basic.phrase.txt:1` exactly. PASS.

2. Segwitv0 compile example (`thresh(2,...) → wsh(multi(2,...))`): confirmed
   via `compile.rs:81` code path + CHANGELOG v0.17/v0.18 entry. PASS.

3. `--unspendable-key` "future version" wording: accurate at v0.32. The stale
   "v0.19+" is in the CLI error string (see L1), not the manual. Manual PASS.

4. No-edit decisions for `61-glossary.md` and `64-descriptors-primer.md`: both
   sampled; no md-codec-version-specific claims present. Decisions justified.

5. `(v0.18)` asymmetry: intentional historical marker, defensible (see L2).

## Disposition

0C / 0I / 1L (cross-repo, filed) / 0N. L1 filed as cross-repo FOLLOWUPS pair
(md1 + toolkit). L2 is informational, no action.
