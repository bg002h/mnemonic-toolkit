# R0 review — `SPEC_md1_bip_alignment_and_code_honesty.md` (round 5, final convergence gate)

**Reviewer:** Fable, adversarial, read-only. Persisted verbatim per CLAUDE.md. Prior round: `md1-bip-alignment-spec-r0-round-4.md` (0C/1I/1M).
**Dispatched:** 2026-07-10 (MD SPEC, R0 round 5). Scope: verify the round-4 folds only; rounds 1–4 verified all substantive mechanics.

## 1. Round-4 fold verification

**I-1(a) — Phase 3 surviving re-pin instruction: RESOLVED.** Line 133 now ends: "release ritual (md-codec + md-cli lockstep publish + tag) + bump `manual.yml`'s `MD_BIN` tag… **Toolkit re-pin is NOT in this cycle** (deferred…)."

**Full-spec grep (`re-pin|repin`) — 7 hits, ZERO surviving this-cycle instructions:** line 103 (descriptive, defer-framed), 105 (the DECISION anchor; imperative content scoped to the follow-up), 113 (descriptive of the deferred flip), 114 (follow-up-scoped), 124 (acceptance-#5 slug parenthetical), 125 (acceptance #6 "or explicitly deferred"), 133 (the folded NOT-in-this-cycle statement). All clean.

**I-1(b) — acceptance #5 missing slug: RESOLVED.** Line 124 lists `toolkit-repin-sh-wpkh-canonical-flip` with an accurate parenthetical, alongside DG-1…DG-4, `canonical-origin-sh-wpkh-toolkit-mirror-divergence`, and `pathless-wallet-backup-partial-decode`.

**M-D — manual.yml MD_BIN pin: FOLDED.** Line 103 carries the full content: `MD_BIN` tag-pinned at md-cli-v0.11.2 (`manual.yml:86`); new-behavior transcripts red the gate until the tag bumps; "doc-verification-only; independent of the deferred toolkit lib re-pin"; the frozen `install.sh:35` sibling-pin trap named with the v0.75.0 precedent; help-text-table-only updates un-gated by `lint.sh`. Phase 3 picks up the bump with an "(M-D)" back-cite.

## 2. No-new-contradiction check

- Phase 3 still ships everything this cycle: BIP §Test Vectors re-sync, independent-reader check, FOLLOWUP filing, md-codec + md-cli lockstep publish + tag, manual.yml `MD_BIN` bump. Only the toolkit **lib** re-pin is carved out.
- The M-D Phase-3 bump does not collide with the defer: line 103 resolves it — `MD_BIN` is doc-verification-only, orthogonal to the md-codec lib pin.
- Whole-spec defer coherence: ripple DECISION (114), ripple header (105), acceptance #5 (124), #6 (125), Phase 3 (133) all agree. No orphaned instruction.
- Everything outside the three folded sites is byte-consistent with the round-4 text.

## 3. Recovery-safety

Unchanged from rounds 1–4 (F-A1 additive, F-A2 dispatch-bit additive, F-A8 rejects only malformed non-zero pads, F-A3 exit≠0, F-A4 stderr-only, F-A5/A9 cosmetic; DEFER is the maximally-safe posture for the tier-(B) wallet-changing flip). The round-4 defect (phasing text lagging the DECISION) is closed.

---

**Critical: 0. Important: 0. Minor: 0.** The spec is internally consistent on the toolkit re-pin defer across all seven mention sites. Passes the final pre-implementation gate — implementation may begin (Phase 1 BIP prose first).

**VERDICT: GREEN (0C/0I)**
