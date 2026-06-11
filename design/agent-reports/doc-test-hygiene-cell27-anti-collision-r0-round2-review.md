# R0 Review — cell-27 temp race + anti-collision terminology — ROUND 2 (GREEN)

**Source SHA:** `be1a581`. Re-review after folding all round-1 findings.

**Verdict: 🟢 GREEN — 0 Critical / 0 Important.** Ready.

## Confirmations
- **I1** — `42:19` replacement prose present + internally consistent with `21-md1-wire-format.md:50` (md1 wire csi = Md1EncodingId) + `bundle.rs::build_unified_card:1071-1078` (engraved = policy_id[0..2]); dangling FOLLOWUPS forward-ref removed; no new anchors.
- **I2** — CHANGELOG `:2009` is the real §IV.2 line; parenthetical (engraved vs wire) accurate, no reword.
- **m1** — `Relaxed` comment correct (distinctness only).
- **m2** — `41:193` double `build_unified_card` anchor → collapse correct.
- **m3** — `glossary.md:57` "cross-prefix agreement" gains "engraved"; `:89`/`:221` cross-ref correct.
- No fold-introduced inconsistency; symbol-ref-check won't break (no new `file.rs::symbol` anchors).

## Pre-existing out-of-scope note (architect)
`42:21` says the §II.1 chunk_set_id is "derived from policy_id" — imprecise for md1 (wire csi is Md1EncodingId-derived). NOT introduced by this cycle. **Decision: fold it in** — same terminology clarification, same file, cheap; correct `:21` to distinguish md1 wire csi (Md1EncodingId) from the policy-id engraved id.
