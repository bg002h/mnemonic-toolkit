# R0 architect review — Word-Card encoding brainstorm spec (round 4)

- **Reviewer:** opus architect (mandatory pre-implementation R0 gate, round 4 of the loop — convergence round)
- **Spec under review:** `design/BRAINSTORM_word_card_encoding_2026-06-24.md` (R0 round-1 + round-2 + round-3 folds applied in-place)
- **Round-3 review:** `design/agent-reports/word-card-r0-round-3.md` (verdict RED, 0C/1I)
- **Date:** 2026-06-24
- **Spec source SHAs (unchanged this round):** mk-codec @ `46631c6`, md-codec @ `7764145d`, ms-codec @ `5c0335c`, toolkit @ `60af98dd`
- **Scope:** funds-safety / custody-safety first-class. Adversarial. This gate blocks all implementation.

---

## Verdict

**GREEN — 0 Critical / 0 Important.**

The single round-3 blocker (NEW-I-1, the incomplete I-A propagation fold) is **genuinely closed**, and so are
both round-3 minors (Minor-1, Nit-3). The round-4 fold synced **every** operative `declared-total-length`
site to §6.3's `recorded-length` ledger; the decoder algorithm (§8) is now internally consistent with the
mechanism §6.3 specifies; the §9.5 freeze list lists the field exactly once under the correct name; and the
only surviving mentions of the dropped name are the three historical fold-logs plus the §6.3/§9.5 negative
"do-NOT-reintroduce" guards. The final adversarial whole-spec consistency pass found **no remaining internal
contradiction, no dangling cross-reference, and no number that fails to reconcile.** All wire-format
citations are untouched at the stated SHAs and the core coding-theory claims (RS evaluation-form
prefix-extensibility, RAID r=1/r=2 MDS, parity-privacy, the `2·subs + erasures ≤ m` lever, the non-linear
in-codeword integrity tag) survive the round-3+round-4 edits intact and remain sound.

This is the convergence round, and the spec is clean. Per `CLAUDE.md`, **0C/0I clears the pre-implementation
R0 gate for this brainstorm spec.** The next step is the plan-doc, which carries its own mandatory R0 loop;
the §12 open questions are correctly *deferred* plan-time parameters (verified below), not silent assumptions,
and are therefore NOT R0 blockers.

---

## Closure of round-3 findings

| Finding | Status | Verification |
|---|---|---|
| **NEW-I-1** (incomplete I-A fold: §8 + §9.5 + §6.1 still cited the removed `declared-total-length` / "fixed declared length") | **CLOSED** | Every operative consumer is now synced to the `recorded-length` ledger. **§8 step 1** (lines 378-382): "read the front header's `recorded-length` ledger and take its highest entry … as authoritative. **Flag truncation when words-present < highest ledger entry** (C3 / I-A) — … a deliberate early stop wrote a matching ledger entry, so it is NOT a false truncation." This now carries §6.3's exact semantics (highest-entry authority + deliberate-stop exemption) verbatim. **§8 step 2** (lines 385-388) + **§6.1** (line 236): the bounded-desync anchor reads "running indices + the recorded-length ledger (§6.3)" — the offending "fixed declared length" is gone, and the phrasing correctly attributes localization primarily to the *running checkpoint indices* with the ledger bounding total length. **§9.5** (lines 456-458): the field is listed exactly once, under the correct name ("front length-ledger encodings: field widths + ledger-entry size"); the former double-listed line now reads "the `recorded-length` ledger is frozen by the stop-sign/ledger bullet above; **do NOT re-introduce the removed `declared-total-length`**" — a negative guard, not an operative freeze. Grep confirms the **only** surviving `declared-total-length` occurrences are lines 28/48/59 (the three historical fold-logs) and lines 276/458 (the §6.3/§9.5 negative guards). **Zero operative use survives. §8 is now internally consistent with §6.3.** |
| **Minor-1** (ledger-durability reasoning explicit) | **CLOSED** | §6.3 lines 286-289 add a dedicated bullet: "**Ledger durability (Minor-1).** The ledger lives in the **RS-protected front header** (§5.2), so a corrupted entry is repaired before the truncation test; losing an *older* entry is inert (authoritative = highest), and losing the *newest* requires front-header loss, which fails loudly rather than silently downgrading." This closes the "does a corrupted/lost ledger entry create a new failure mode" probe affirmatively in-text. Correct on all three sub-cases (corrupted → RS-repaired; older lost → inert; newest lost → loud front-header loss, not silent downgrade). |
| **Nit-3** (`§8.5` → `§8 step 5`) | **CLOSED** | All three operative `§8.5` labels are now `§8 step 5` (lines 25, 172, 427 — the load-bearing C1 integrity-cross-check anchor, which resolves to §8 step 5 at lines 395-400). Grep for `§8\.5` returns exactly one line: line 63, the round-3 fold-log entry *documenting* the fix ("fixed the `§8.5` label → `§8 step 5`"). Zero operative `§8.5` remains. |
| **Minor-2** (plan-time per-K-class 11-bit split) | **CORRECTLY DEFERRED (not a fold blocker; carried to the plan's R0)** | §12-Q2 states pinpoint-or-block-erasure is NORMATIVE (§6.1) and scopes the *remaining* open piece to "the exact 11-bit split (index vs parity) and the reinsert-and-test cost ceiling." §6.1 lines 228-231 keep the normative pinpoint requirement, and lines 232-239 keep the safe whole-block-erasure / refuse fallback, so no K-class can silently mis-pinpoint. The round-3 reviewer explicitly classified this as a watch item, NOT a fold blocker; it is a plan-time R0 obligation, correctly deferred. |

---

## Final consistency pass

I ran a full adversarial whole-spec sweep — cross-references, every operative number, the wire-format
citations, and each core coding-theory / custody claim — specifically hunting for anything the three prior
rounds' in-place folds may have inadvertently broken.

**Cross-references — all resolve, none dangle.**
- Every `§N` / `§N.M` / `§N step M` pointer used in the body (§1.1, §5.2/5.3/5.4, §6.x, §7.x, §8 step 1/2/5,
  §9, §9.1, §9.5, §10, §12) maps to a real section or to a stable list-item convention. §5.2/§5.3/§5.4 refer
  to the numbered items 2/3/4 of §5 (header / integrity tag / regroup) — a convention used consistently
  throughout (every §5.3 points to the integrity tag, every §5.2 to the header). No broken pointer.
- The only `declared-total-length` mentions are the 3 fold-logs + the 2 negative guards (grep-confirmed);
  the only `§8.5` mention is the fold-log entry documenting its removal. Both removals are complete.

**Numbers — all reconcile (recomputed independently).**
- `b = round(√54) = 7`; checkpoints `= ⌈54/7⌉ = 8`; `K′ = 54 + 8 = 62`. ✓
- Ladder: mandatory 62 → `62+7 = 69` (7 check) → `62+20 = 82` (20 check). ✓
- Corrects `= ⌊m/2⌋`: `m=6→3`, `m=20→10`, `m=46→23`. ✓ (matches §6.4 ladder and §9.1 table)
- Appendable `= 2047 − 62 = 1985` (§9 "~1985"). ✓
- `K=160 → b = round(√160) = 13` (§6.1 "b≈13"). ✓
- §9.1 overhead column reconciles as `(sync + parity)/K`: `(8+6)/54 ≈ 26%`→labeled 25%; `(8+20)/54 ≈ 52%`
  →labeled ~50%; `(8+46)/54 = 100%`→labeled 100%. Internally consistent. ✓
- The K-table's `~150` (2-of-3 wallet-policy estimate, line 191) vs §6.1's round `K=160` worked b-example
  (line 201) is a benign rounded illustrative value, **not** a contradiction: `round(√150)=12` and
  `round(√160)=13` both sit in the same 12–13 band; the table is a byte→word estimate, the example picks a
  round number to show `b≈13`. No number fails to reconcile.

**Wire-format citations — untouched at the stated SHAs.** Same SHAs (mk `46631c6`, md `7764145d`, ms
`5c0335c`, toolkit `60af98dd`); same in-text citations (`consts.rs:29` 0x00+16/20/24/28/32 B entropy;
`xpub_compact.rs` 65 B incompressible / 73 B compact; `encode.rs:65-92`; TLV `0x02` 65 B). The round-4 fold
touched only §6.1/§6.3/§8/§9.5 ledger language — none of these wire-format lines. Round-3 already
grep-verified all citations TRUE at these SHAs; nothing this round could have disturbed them. Zero drift.

**Core coding-theory / custody claims — re-confirmed sound, unchanged by the folds.**
- **RS evaluation-form prefix-extensibility (§6.2):** any prefix `P₁…Pₘ` is a valid `[K′+m, K′]` MDS code of
  minimum distance `m+1`; append-only/progressive; generator-poly form correctly excluded as
  non-prefix-extensible. Correct and load-bearing for both progressive dials.
- **The `2·subs + erasures ≤ m` lever + `⌊m/2⌋`-correct + refuse-beyond-budget (§6.2 / §9a):** correct RS
  arithmetic; the miscorrection-within-budget residual is handled by the non-linear integrity tag (C1).
- **Non-linear in-codeword integrity tag (§5.3 / §8 step 5):** the `≤ 2⁻ᵗ` bound holds for the *only*
  permitted construction (a non-linear hash cannot be forced by a linear-RS miscorrection); the linear-tag
  forbiddance is intact. NEW-I-1's §8-step-5 label fix preserves the correct anchor.
- **RAID r=1/r=2 MDS (§7.1):** `P₁ = Σxᵢ`, `P₂ = Σαⁱxᵢ`; `[n+r,n]` recovers any `r` of `n+r`; `P₁` unchanged
  when `P₂` is appended (append-only at plate granularity); `ord(α) ≥ n_max` frozen. Correct.
- **Parity-privacy (§7.3):** a lone parity plate is `r` linear combinations of `n` unknowns and leaks nothing
  below the legitimate threshold; the r=2 "two parity plates together are still only 2 of n equations"
  framing is correct.
- **Two-guarantee split (§9):** value-layer MDS vs indel-layer sync-bounded remains honest; the indel layer
  correctly states "cost 1/word only under successful per-slot pinpointing; honest worst case `b` erasures
  per damaged block." No over-claim.
- **Custody-safety posture (C1/C2):** the decoder never claims to never-miscorrect; it refuses beyond budget,
  and a within-budget miscorrection is caught by the independent non-linear tag (residual ≤ 2⁻ᵗ); ambiguous
  realignment ⇒ refuse-and-report. No silent-mis-decode path survives.

**Open-questions hygiene (§12) — deferred, not assumed.** Each item promoted to normative in prior rounds
(Q2 pinpoint-or-block-erasure, Q5 stripe alignment, Q7 detection-all-K + parity floor) explicitly says "is now
**NORMATIVE**" and scopes only the remaining *parameter* (exact 11-bit split / exact padding rule / exact
floor value) as open. These are correctly deferred plan-time constants, each with a safe normative fallback
already specified in the body — none is silently assumed. Per the round-4 mandate, deferred §12 items are not
R0 blockers.

---

## New Critical / Important / Minor

**New Critical:** None.

**New Important:** None.

**New Minor / Nit (non-blocking, optional polish for the plan author — do NOT gate on these):**

- **N-polish-1 (imprecise cross-ref, cosmetic).** §6.1 line 212 cites the small-`K` "fixed parity floor
  (§9.1)", but §9.1 is the K≈54 worked-numbers table and does not itself define the small-`K` floor; the
  floor *value* is correctly deferred in §12-Q7 ("remaining open = the exact floor value"). The pointer would
  read more precisely as §12-Q7 (or no section ref). This is pre-existing (round-2 Nit-2 fold, accepted by the
  round-3 reviewer), does not mislead an implementer (§6.1 states the floor concept; §12-Q7 holds the open
  value), and is **well below the I/C threshold**. Optional fix at plan time; NOT a gate blocker.

These are below the bar that blocks the gate. I am explicitly NOT manufacturing a finding to look diligent:
the spec is genuinely clean, and the one cosmetic pointer above is logged only for completeness.

---

## Gate decision

**GREEN. 0 Critical / 0 Important.** The round-3 blocker (NEW-I-1) and both round-3 minors (Minor-1, Nit-3)
are genuinely closed; the decoder algorithm (§8) is now internally consistent with §6.3; the §9.5 freeze
lists the ledger once under the correct name; the only `declared-total-length` / `§8.5` survivors are
historical fold-logs and negative guards. The final adversarial whole-spec pass found no internal
contradiction, no dangling reference, and no number that fails to reconcile; wire-format citations and core
math (RS prefix-extensibility, RAID r=1/r=2 MDS, parity-privacy, non-linear integrity tag, the lever
arithmetic) are all intact and sound. The §12 open questions are correctly deferred plan-time parameters,
each with a safe normative fallback, and are not R0 blockers.

**The reviewer loop has converged.** Per `CLAUDE.md`, this brainstorm spec **clears the mandatory
pre-implementation R0 gate (0C/0I)**. The next artifact is the plan-doc, which begins its own R0 loop —
the plan's R0 should, at minimum: (1) exhibit a concrete `(marker | index | parity)` 11-bit checkpoint split
that meets the normative per-slot-pinpoint requirement for every surfaced K-class, or document the K-classes
that fall back to block-erasure (carried Minor-2 / §12-Q2); (2) resolve §12-Q1 (header bit layout + array-id
hash/width), §12-Q5 (exact RAID padding rule), §12-Q6 (stop-sign encoding), and §12-Q7 (parity-floor value);
and (3) fix the constant values pinned in §9.5. No code until the plan-doc is itself R0-GREEN.
