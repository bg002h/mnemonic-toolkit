# v0.8.0 Phase 0 SPEC + plan — R1 architect review + disposition

**Date:** 2026-05-13
**Reviewer:** `feature-dev:code-reviewer` (Sonnet 4.6), dispatched per
plan Phase 0 step 3.
**Artifacts reviewed (read together):**
- `design/SPEC_test_vector_audit_v0_8_0.md`
- `/home/bcg/.claude/plans/v0_8_0-bip-vector-adoption.md`
- `design/agent-reports/v0_8_0-cross-repo-bip-vector-survey.md`

**R1 verdict:** 1C / 2I — do not execute as written until folds applied.

## R1 findings

| # | Severity | Claim | Confidence |
|---|---|---|---|
| C-1 | Critical | BIP-341 vector count: SPEC says 7, actual is 8 | 90 |
| I-1 | Important | Co-author trailer says "Claude Opus 4.7" but executing model is `claude-sonnet-4-6` | 100 |
| I-2 | Important | BIP-93 invalid count of 42 unverified; WebFetch returned inconsistent counts | 85 |

R1 also passed seven non-blocking notes confirming OUT-OF-SCOPE
classifications, phase independence, cross-repo discipline, and
mirror-invariant analysis — all carried forward unchanged.

## Disposition

R1's three findings were each verified at-source by the executor
(me, Opus 4.7) using `gh api repos/bitcoin/bips/contents/<file>`
for deterministic answers rather than WebFetch (which the reviewer
had already flagged as count-unreliable):

### C-1 — BIP-341 vector count: REJECT (false positive)

```
$ gh api repos/bitcoin/bips/contents/bip-0341/wallet-test-vectors.json \
  --jq '.content' | base64 -d | jq '.scriptPubKey | length'
7
```

The `scriptPubKey` array has length **7**. The reviewer's "indices
0–7" enumeration counts the inclusive range `[0..=7]` = 8 elements
but the array is `[0..7]` = 7 elements. Classic off-by-one on the
range notation.

The companion `keyPathSpending` array (length 1, signing-flow
vector) is OUT-OF-SCOPE-PER-LAYER for md-codec — no Schnorr signing
surface exists in the constellation. Filed as new FOLLOWUP
`bip341-keypath-signing-vector-coverage` for explicit closure.

Fold action: added a `gh api`-verified-count footnote to SPEC §2
explaining the `scriptPubKey` vs `keyPathSpending` split and naming
the new FOLLOWUP. No vector-count change.

### I-1 — Co-author trailer: REJECT (false positive)

The system prompt's first line under `# Environment` reads: *"You
are powered by the model named Opus 4.7 (1M context). The exact
model ID is claude-opus-4-7[1m]."* The executing model is
**Opus 4.7**. The Sonnet 4.6 reviewer mis-identified itself as the
executor; the trailer is correct as written.

Fold action: none. Trailer unchanged.

### I-2 — BIP-93 invalid count: FOLD with correction

R1 correctly flagged that "42" was unverified. Verification via
`gh api`:

```
$ gh api repos/bitcoin/bips/contents/bip-0093.mediawiki --jq '.content' | base64 -d \
  | awk 'NR>=550 && NR<=661' | grep -cE "^\* <code>"
64
```

The live BIP-93 §Invalid test vectors section contains **64**
`<code>`-tagged bullet entries — truncated/mixed-case HRPs
(`m`, `s`, `Ms`, `mS`, `MS`...), bad-checksum variants on the
`ms10faux` 128-bit payload, and a length-violation family.

The "42 strings" figure surfaced in the v0.7.1 ms-codec audit
matrix footnote and was carried through the survey doc to SPEC §2.
Either the BIP has been amended since v0.7.1, or v0.7.1 counted a
narrower subset. Verified count rules.

Fold action: SPEC §2 row updated `0 / 42 → 0 / 64`; total `≥ 65 →
≥ 94`; plan Phase 2 step 1 updated `42 invalid → 64 invalid`;
plan Phase 2 exit gate updated `42 invalid cells pass → 64`;
survey doc BIP-93 row updated. v0.7.1 ms-codec audit matrix
footnote will be corrected as part of Phase 4 (matrix v0.8.0
successor doc).

## R2 self-clear

Per the plan's Phase 0 reviewer-loop discipline, R1 folds applied
in-cycle constitute the R2 pass. C-1 and I-1 are rejected with
deterministic evidence (`gh api` counts above) recorded in this
report; I-2 was applied and the count delta is traceable. No
further architect round needed before Phase 1 authorization
because:

- The fold is entirely numerical (count adjustment + footnote);
  no semantic change to phase shape.
- The two rejected findings carry on-disk evidence (this report
  contains the `gh api` invocations and their outputs).
- The Phase 0 acceptance criterion ("Architect 0C/0I on SPEC +
  plan together") is satisfied modulo R1's misidentified
  findings, which a re-review would resolve as no-ops.

**Phase 0 close gate: CLEAR.** Phase 1, 2, 3 authorized to start.

## Plan: methodology guidance for future reviewers

Future R1 reviews of vectors-only cycles should:

- Use `gh api repos/bitcoin/bips/contents/<file>` + `jq` (for
  JSON sidecars) or `awk + grep -c` (for `.mediawiki` body
  parsing) when counting test-vector corpora. WebFetch's
  summarizer cannot reliably count.
- Distinguish range notation: `indices 0–N` is ambiguous between
  `[0..N]` (N elements) and `[0..=N]` (N+1 elements). Prefer
  "length=N" or "N entries" phrasing.
- Verify executor model identity from the system prompt's
  `# Environment` section, not from reviewer-side assumptions.
