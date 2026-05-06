# Phase S0 — SPEC v0.5 document copy + amendments — code-reviewer r1 (2026-05-06)

## Findings

### Critical
None.

### Important

**I-1: Carry-forward boilerplate references v0.3 instead of v0.4** (lines 17-23). Three instances: section heading, narrative, link, delta-section label. Copy-paste residue from v0.4 source.

**Status:** addressed inline. Heading rewritten to "Carry-forward from v0.4"; narrative + link updated; delta-section label rewritten to enumerate v0.5-specific sections only.

**I-2: §6.7 dangling forward-reference to §6.6.a** (line 236). §6.6.a in v0.5 is "Legacy flag deletion (table removed)"; the reference points to deleted content.

**Status:** addressed inline. Parenthetical reworded to "v0.5 sole input shape" without citing the deleted section.

### Low / Nit

**N-1: Plan changelog says "three-case" table; SPEC body correctly says "four-case".** Plan-side stale wording; SPEC body is correct. Phase R CHANGELOG drafting must use "four-case".

**Status:** noted for Phase R.

**N-2: §6.6 gappy row numbering** acceptable; cross-references intact; documented "Removed in v0.5" annotation is the safer choice.

**N-3: Crate README Quickstart uses legacy flags** — expected to be updated in Phase A.

## Outcome

S0 APPROVED with I-1 + I-2 addressed inline. Proceed to Phase B.
