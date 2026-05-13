# v0.8.0 Phase 4 — cross-repo audit-matrix lift — R1 architect review + disposition

**Date:** 2026-05-13
**Reviewer:** `feature-dev:code-reviewer` (Sonnet 4.6), dispatched
per plan Phase 4 reviewer-loop discipline.
**Phase commits reviewed (one per repo):**

| Repo | Commit | Branch |
|---|---|---|
| `mnemonic-toolkit` | `03b6e8d` | `v0_8_0-bip39-bip85-fill` |
| `descriptor-mnemonic` (md-codec) | `e6e0b5d` | `v0_8_0-bip341-wallet-vectors` |
| `mnemonic-secret` (ms-codec) | `dbce078` | `v0_8_0-bip93-inline-vectors` |
| `mnemonic-key` (mk-codec) | `9605513` | `v0_8_0-bip-vector-adoption-companion` |

## R1 verdict

**0C / 1I / 2N.** The single Important finding (I-1) verified at-source
as a **false positive**; rejecting with `git show --stat` evidence.
Both nits non-blocking.

## R1 findings + disposition

### I-1 — §4 cites wrong commit shas for FOLLOWUPS landing: REJECT (false positive)

The architect read commit messages alone (e.g., "test(cli_derive_child):
BIP-85 v85.3" → inferred "pure test-file addition; no FOLLOWUPS touched")
without verifying via `git show --stat`. Evidence:

```
$ git show d269dda --stat --format=
 crates/mnemonic-toolkit/tests/cli_derive_child.rs | 35 +++++
 design/FOLLOWUPS.md                               | 27 +++++  ← 27 line insertions
 design/SPEC_test_vector_audit_v0_8_0.md           |  2 +-
 3 files changed, 63 insertions(+), 1 deletion(-)
```

```
$ git -C /scratch/code/shibboleth/mnemonic-secret show 7101c16 --stat --format=
 crates/ms-codec/tests/bip93_inline_vectors.rs | 285 +++++
 design/FOLLOWUPS.md                           |  18 ++   ← 18 line insertions
 2 files changed, 303 insertions(+)
```

Both commits include the `bip-vector-adoption-v0_8` FOLLOWUPS entry
addition. The §4 sha citations are correct as written. `d0e6afc`
(toolkit Phase 0) does NOT touch `design/FOLLOWUPS.md` per
`git show d0e6afc --stat` — the architect's proposed correction
would itself be wrong.

**Disposition:** Reject. No change to toolkit matrix §4.

### N-1 — ms-codec §3 asymmetry with md-codec §3 (non-blocking)

ms-codec §3 cross-cites toolkit + mk-codec but not BIP-340 OOS;
md-codec §3 cross-cites ms-codec's BIP-93. Architect confidence 40 —
below threshold, framing inconsistency only. **Disposition:** carry
forward unchanged; both framings are defensible.

### N-2 — BIP-380 §380.2–.8 not yet filed as FOLLOWUP (non-blocking)

Toolkit §5 explicitly acknowledges "Survey-noted, not yet filed."
The deferral is defensible since BIP-380 checksum vectors are an
md-codec/miniscript-layer concern and the cycle plan explicitly
names it as a survey note pending filing. **Disposition:** carry
forward; file in a future cycle if/when md-cli exercises descriptor
checksum independently of `rust-miniscript`.

## R1 focus-area verdicts (paraphrased from review)

All 8 focus-area items returned PASS or non-blocking notes:

1. Cross-repo consistency — §0 counts match per-sibling matrices.
2. SUPERSEDED headers — all 4 v0.7.1 files have correct forward-pointer.
3. OOS FOLLOWUPS — all 3 exist at cited paths with cited short-ids.
4. Cycle FOLLOWUPS commit shas — verified correct (see I-1 rejection above).
5. Per-sibling matrix completeness — both md-codec and ms-codec §1 tables full.
6. Carry-forward accuracy — BIP-32 TV1 spot-check passed (no regression).
7. v0.9.0 carry-overs (§5) — 4 items, all defensible deferrals.
8. mk-codec no-scope pattern — correct delegation; v0.7.1 substantive matrix remains authoritative.

## R2 self-clear

R1's only Important finding is rejected with deterministic evidence
(git show --stat output captured above). The two nits are
non-blocking. All cross-repo invariants verified:

- 4 v0.8.0 matrix files exist on disk, one per sibling repo.
- 4 v0.7.1 matrix files carry SUPERSEDED headers in lockstep.
- Cross-citation graph closes: each matrix names the others.
- Cell-count net: +94 vectors across the constellation vs v0.7.1.
- FOLLOWUPS state across 4 repos: `bip-vector-adoption-v0_8` entry
  present in each at the cited commit sha.

**Phase 4 close gate: CLEAR.** Phase E (release rollup) authorized.

## Methodological note for future reviewers

R1 reviews of multi-file commits should verify file lists via
`git show <sha> --stat` rather than inferring from commit messages
alone. Commit messages name the headline change but multi-file
commits routinely include adjacent FOLLOWUPS / CHANGELOG / SPEC
edits that the headline doesn't summarize.
