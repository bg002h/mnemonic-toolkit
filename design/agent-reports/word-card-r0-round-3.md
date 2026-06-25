# R0 architect review — Word-Card encoding brainstorm spec (round 3)

- **Reviewer:** opus architect (mandatory pre-implementation R0 gate, round 3 of the loop)
- **Spec under review:** `design/BRAINSTORM_word_card_encoding_2026-06-24.md` (R0 round-1 + round-2 folds applied in-place)
- **Round-2 review:** `design/agent-reports/word-card-r0-round-2.md` (verdict RED, 2C/3I)
- **Date:** 2026-06-24
- **Spec source SHAs (unchanged this round):** mk-codec @ `46631c6`, md-codec @ `7764145d`, ms-codec @ `5c0335c`, toolkit @ `60af98dd`
- **Scope:** funds-safety / custody-safety first-class. Adversarial. This gate blocks all implementation.

---

## Verdict

**RED — 0 Critical / 1 Important.**

Both round-2 Criticals are **genuinely closed**: NEW-C1 (the integrity tag now mandates a non-linear
cryptographic hash, forbids any linear in-codeword tag, and the post-correction-check reasoning is
sound) and NEW-C2 (checkpoints now carry a self-identifying marker, the realignment is validated by
index-continuity across ≥2 checkpoints with an explicit refuse-on-ambiguity rule that also covers the
coincidental "data word parses as a marker" false-positive, and the compound `cost ≤ 2b` lemma is
arithmetically correct). The two §9.5 freeze-gap Importants that mirrored them (I-B, I-C) are closed.
The Nits (ladder arithmetic 62/69/82, small-`K` floor, `~1985` appendable) are all closed and
arithmetically self-consistent. Wire-format citations are **untouched** (same SHAs, same line numbers as
round-2 table §A — verified by grep, zero drift). The core math (RS prefix-extensibility, RAID r=1/r=2
MDS, parity-privacy) survives the edits unchanged and remains sound.

**The single blocker is that the I-A fold is INCOMPLETE.** The round-2 I-A finding was folded *correctly
in §6.3* — `declared-total-length` (double-meaning) was replaced by a single-meaning front-anchored
append-only `recorded-length` ledger, and that section is now clean and unambiguous. But the in-place
edit did **not** propagate to the three other live sites that consume the field: **§8 step 1 + step 2
(the decoder algorithm) and §9.5 (the frozen-constants freeze) and §6.1 still reference the removed
`declared-total-length` / "fixed declared length."** The decoder algorithm — the operational heart of the
spec — therefore still implements the *rejected* primitive and the *rejected* truncation test, and §9.5
would freeze the dropped field's name into the 20-year constant set (double-listed alongside the new
ledger). This is a self-contradiction introduced by the round-2 fold, exactly the "folds themselves can
introduce drift" / "internal numbering/consistency from in-place edits" class this round was told to hunt.

This is **Important, not Critical**: the correct mechanism is fully and unambiguously specified in §6.3,
the residual is a documentation/propagation inconsistency (not a silent custody failure), and the
underlying semantics defect I-A flagged (false-truncation on a deliberate stop) is itself already
resolved in §6.3. But a plan author who lifts §8/§9.5 verbatim would implement the wrong, already-rejected
behavior and freeze the wrong constant — so per `CLAUDE.md` it must close before the plan-doc. One
mechanical fold (sync §8/§9.5/§6.1 to §6.3's ledger) converges this to GREEN.

Per `CLAUDE.md`, no plan-doc and no code until 0C/0I. Re-dispatch after the fold — the loop continues.

---

## Closure of round-2 findings

| Finding | Status | Notes |
|---|---|---|
| **NEW-C1** (linear tag void) | **CLOSED** | §5.3 (lines 154-164) now mandates a **non-linear** truncated cryptographic hash (`SHA-256[0..t]`, `t ≥ 32`), and **explicitly FORBIDS a linear (BCH/CRC/XOR) tag in-codeword** with the correct reason ("lives in the same linear RS image, so a miscorrection satisfies it by construction and the bound collapses"). The `≤ 2⁻ᵗ` bound is now sound *for the mandated construction*: a linear RS codeword has no mechanism to force `tag == hash(payload)`, so a wrong-but-valid codeword satisfies the non-linear tag only with prob `≈ 2⁻ᵗ`. The "RS-protected, post-correction ⇒ no false reject on tag typo" reasoning holds (line 162-163: tag words are RS symbols, check runs post-correction → a tag typo is repaired by RS before the cross-check). **No residual ambiguity that would let a plan freeze a linear tag** — the BCH-residue *option* is gone (only survives as the negative "FORBIDDEN" statement + an "admissible only if fully out-of-codeword" carve-out, which is correct). §9.5 (lines 431-432) pins "the non-linear cryptographic hash family + truncation bit-width `t` — a linear tag is forbidden in-codeword," so I-B is closed inside this. |
| **NEW-C2** (checkpoint recognition) | **CLOSED** | §6.1 (lines 187-195) now adds a **self-identifying marker** in each checkpoint word, and a NORMATIVE recognition/realignment rule: re-find the next checkpoint by a bounded offset search validated by (a) the marker and (b) **index-continuity across ≥2 consecutive checkpoints + local parity**; "**ambiguous or no alignment ⇒ refuse-and-report.**" This makes recognition **well-founded**. The coincidental-false-positive case the prompt names (a *data* word parsing as a valid marker at a shifted offset) is covered: a lone spurious marker-match cannot satisfy index-continuity *across ≥2* checkpoints with consistent stride, and any *ambiguity* between two candidate alignments triggers refuse (lines 193-195) — so a false-positive realignment is either rejected by the 2-checkpoint cross-check or surfaced as refuse, never silently accepted. The compound lemma (lines 224-228) is correct: a deleted `Cᵢ` + a data deletion in block `i` → merged span detected at the next recognizable checkpoint's index/stride mismatch, erased at **cost ≤ 2b** (two adjacent blocks merged), and if the next checkpoint is itself unrecognizable → refuse. `cost ≤ 2b` is right (the span can extend across at most the two blocks flanking the deleted checkpoint). §9.5 (lines 433-434) freezes the marker pattern + 11-bit split + realignment ceiling, so I-C is closed inside this. |
| **I-A** (ledger) | **MECHANISM CLOSED in §6.3, but FOLD INCOMPLETE** | §6.3 (lines 252-269) correctly replaces the field with a front-anchored append-only `recorded-length` ledger, single meaning, deliberate-stop no longer false-flags, lost-tail still flagged. **But §8 (decoder), §9.5 (freeze), and §6.1 line 220 were not synced** and still cite the removed `declared-total-length`. → **NEW-I-1** below. |
| **I-B** (§9.5 pin tag function) | **CLOSED** | §9.5 line 431-432 names the non-linear hash family + width; the freezable-linear-option is gone. |
| **I-C** (§9.5 pin checkpoint recognition) | **CLOSED** | §9.5 line 433-434 freezes marker + 11-bit split + realignment ceiling. |
| **Nit-1** (K′=62 ladder) | **CLOSED** | §6.4 ladder now reads `K=54, 8 checkpoints ⇒ K′≈62`, "words 1–62", "⟐ 69 (7 check) → 3 wrong", "⟐ 82 (20 check) → 10 wrong". Verified: `⌈54/7⌉=8`, `54+8=62`, `62+7=69`, `62+20=82`, `⌊6/2⌋=3`, `⌊20/2⌋=10`, `⌊46/2⌋=23`. §6.1 line 186 now `K=160 ⇒ b≈13` (was 12; `round(√160)=13`). §9.1 table (sync `~8`; parity `6/20/46` → corrects `3/10/23`) is internally consistent with the ladder. **Arithmetically self-consistent.** |
| **Nit-2** (N5 small-`K`) | **CLOSED** | §6.1 lines 196-198: detection on for **any `K ≥ 1`**, ≥1 checkpoint always, fixed parity floor for `K<10`, degenerate single checkpoint so √K never underflows. Committed, not just open-Q. |
| **Nit-3** (~1980 vs ~1985) | **CLOSED** | §9 line 402 now reads `~1985` (`2047 − 62 = 1985`). |

---

## New Critical

**None.** Both round-2 Criticals are genuinely closed (above). I specifically adversarially probed the
two highest-risk fold interactions the round-2 prompt flagged:

- **Does the marker steal the 11 bits NEW-C2 and C2 both need?** Yes, they compete for the same 11 bits
  (marker + running index + local parity, §6.1 line 187), and that competition is real: at `K=54` the
  index needs ~3 bits (8 checkpoints), at `K=2047` ~6 bits (≈46 checkpoints), leaving the marker + local
  parity to share the remaining 8→5 bits. **This is NOT a Critical** because (a) the spec correctly
  defers the *exact* split to §12-Q2 / §9.5 as a plan-assigned frozen constant (it does not over-claim a
  specific split works), and (b) pinpointing is stated as a *normative requirement on that constant*
  ("local parity MUST be strong enough to pinpoint a single intra-block indel," §6.1 lines 213-216) with
  a **safe fallback** when it can't be met (whole-block erasure cost ≤ b, or refuse). So if a given
  K-class's 11-bit budget proves too tight for both a discriminating marker and per-slot pinpointing, the
  decoder degrades to block-erasure / refuse — never to a silent mis-pinpoint. The competition is a
  plan-time constraint to satisfy (or prove unsatisfiable for some K-class and fall back), not an
  unproven correctness claim. Flagged as a non-blocking watch item (Minor-2) so the plan's R0 verifies a
  concrete split exists for every K-class before freezing.
- **Corrupted / lost ledger entry — new failure mode?** No. The ledger lives in the front header, which
  is part of the `K` data words → RS-protected; a corrupted entry is repaired by the RS pass like any
  data word. A *lost* (deleted) entry is an indel in the header region, caught by the sync/indel layer.
  Authoritative = the *highest* entry, so losing an older/lower entry is inert; losing the *newest*
  (highest) entry can only happen by losing the front, which also loses the header/array-id → the decode
  fails loudly, not silently. No silent under-protection. (Worth one explicit sentence — Minor-1.)

---

## New / promoted Important

### NEW-I-1 — the I-A fold is incomplete: §8 (decoder) + §9.5 (freeze) + §6.1 still reference the removed `declared-total-length`; the decoder algorithm still implements the rejected truncation primitive

**Where:**
- **§8 step 1 (lines 358-361):** "read the front header's `declared-total-length`; … **Flag truncation
  when words-present < `declared-total-length`** (C3)."
- **§8 step 2 (line 365):** "every indel localizes to ≥ block granularity (running indices + **declared
  length**)."
- **§6.1 line 220:** "The running indices + **fixed declared length** (§6.3) guarantee every indel is
  localized…"
- **§9.5 line 436:** "**Header bit-layout:** all fields, incl. **`declared-total-length`** (§5.2)."

**The contradiction.** The I-A fold (§6.3 lines 252-269) *removed* `declared-total-length` and *replaced*
it with the front-anchored append-only `recorded-length` ledger whose authoritative value is the
**highest ledger entry**, with the truncation test redefined as "words physically present **<** highest
ledger entry ⇒ truncation flag" (line 262). §5.2 (header, line 149) was correctly updated to carry
"`recorded-length` ledger (append-only, front-anchored; §6.3)". **But §8 — the normative decoder
algorithm — was never updated.** It still:
- reads a field named `declared-total-length` that no longer exists in the header (§5.2 renamed it),
- runs the *old* truncation test `words-present < declared-total-length` instead of the ledger test
  `words-present < highest-ledger-entry`, and
- (step 2 / §6.1 line 220) anchors the bounded-desync invariant on a "fixed declared length," whereas
  §6.3's whole point is that there is **no fixed/pre-committed length** — only an append-only ledger that
  *grows*. "fixed declared length" directly contradicts §6.3 lines 260-261 ("there is no
  pre-committed-max").

And §9.5 — the 20-year freeze list — now **double-lists** the same primitive under both names: line 435
freezes "**front length-ledger** encodings: field widths + ledger-entry size (§6.3)" (the new, correct
name) *and* line 436 freezes "all fields, incl. **`declared-total-length`** (§5.2)" (the removed name).
An implementer freezing §9.5 verbatim would either freeze a non-existent field or, worse, resurrect the
rejected double-meaning field that I-A was raised to kill.

**Why this is the operative defect, not cosmetic.** §8 is the *executable* contract — a plan-doc derives
the decoder from it. The semantics that §6.3 carefully fixed (deliberate early-stop ⇒ present == ledger
⇒ no false flag) are **absent from §8**; §8 still says "flag truncation when words-present <
declared-total-length," which, read with the (now-deleted) pre-commit meaning, is exactly the
false-truncation-on-deliberate-stop behavior I-A removed. The fold fixed the *explanation* (§6.3) but not
the *algorithm* (§8). A future reader has two live, contradictory descriptions of the same test.

**Why Important not Critical.** The correct mechanism is fully and unambiguously specified in §6.3; this
is a propagation/consistency gap, not a missing or silently-wrong primitive — and the underlying defect
I-A named (false-positive truncation, a usability/trust issue) is itself not a funds-safety hole. But it
must close before the plan-doc freezes the header-field semantics and the §9.5 constants, because §8 and
§9.5 are precisely what the plan lifts.

**Recommended fold (mechanical):**
- **§8 step 1:** replace "read the front header's `declared-total-length`; … Flag truncation when
  words-present < `declared-total-length`" with the §6.3 ledger language: "read the front header's
  `recorded-length` ledger; the authoritative recorded length is the **highest ledger entry**; take the
  highest-count stop-sign as authoritative. **Flag truncation when words-present < highest ledger
  entry** (C3/I-A) — a deliberate early-stop wrote its own matching ledger entry, so present == ledger ⇒
  no false flag."
- **§8 step 2 / §6.1 line 220:** replace "fixed declared length" / "declared length" with "the
  append-only `recorded-length` ledger (§6.3)". (The bounded-desync anchor is actually the *running
  checkpoint indices*; the ledger only bounds total length, so the phrase can simply drop "fixed.")
- **§9.5:** delete the redundant/stale line 436 clause "incl. `declared-total-length`" (the ledger is
  already frozen by line 435); replace with "incl. the `recorded-length` ledger fields (§5.2/§6.3)" or
  merge the two bullets so the field is listed once, under the correct name.

---

## Minor / Nit

- **Minor-1 (ledger durability — make the reasoning explicit).** §6.3 should state in one sentence that
  the ledger is RS-protected (it is part of the `K` header data words), so a *corrupted* ledger entry is
  repaired by the RS pass and a *lost* newest entry can only occur with front-header loss (which fails
  the decode loudly, not silently). This closes the "does a corrupted/lost ledger entry create a new
  failure mode" probe affirmatively in-text rather than leaving it to the reader. No correctness gap;
  documentation completeness only.

- **Minor-2 (NEW-C2 ↔ C2 11-bit budget — verify a concrete split exists per K-class at plan time).** The
  marker (NEW-C2) and the index + per-slot-pinpoint local parity (C2) share one 11-bit checkpoint word.
  The spec correctly defers the exact split (§12-Q2, §9.5) and has a safe fallback, so this is **not** a
  spec blocker. But the plan-doc's R0 MUST exhibit a concrete `(marker | index | parity)` split that (a)
  fits 11 bits and (b) meets the normative "pinpoint a single intra-block indel" requirement for the
  largest surfaced K-class (≈46 checkpoints ⇒ ~6 index bits, leaving ≤5 bits for marker+parity), OR
  explicitly accept block-erasure/refuse for the K-classes where 11 bits can't do both. Recommend adding
  one line to §12-Q2: "the plan must demonstrate the split satisfies per-slot pinpointing for every
  surfaced K-class, or document the K-classes that fall back to block-erasure." Watch item, not a fold
  blocker.

- **Nit-3 (citation label) — "§8.5" is referenced 3× but §8 has no labeled sub-section 8.5.** Lines 25,
  157, 406 cite "§8.5" for the integrity cross-check; the cross-check is actually **§8 step 5** (line
  374-379). Harmless, but since this is the load-bearing C1 anchor, change "§8.5" → "§8 step 5" so the
  pointer resolves. (Pre-existing from round-1; surfaced now only because it co-locates with the §8 edit
  the I-A fold needs.)

---

## What is SOUND (re-confirmed this round — do not re-litigate)

- **Wire-format facts:** **untouched** by the round-2 folds. Same SHAs (mk `46631c6`, md `7764145d`, ms
  `5c0335c`), same in-text citations (`consts.rs:29`, `xpub_compact.rs`, `encode.rs:65-92`, TLV `0x02`
  65-B, `0x00` prefix, 73-B compact / 65-B incompressible). Round-2 table §A re-verified all TRUE at
  these SHAs; the folds touched none of these lines (grep-confirmed). Zero drift.
- **NEW-C1 hash mandate (§5.3):** the in-codeword non-linear tag is correct; the linearity argument for
  forbidding BCH/CRC/XOR is correct; the `≤ 2⁻ᵗ` bound now holds for the *only* permitted construction.
- **NEW-C2 recognition (§6.1):** marker + index-continuity-across-≥2 + refuse-on-ambiguity is a
  well-founded recognition primitive that covers the coincidental-data-word-as-marker false positive; the
  `cost ≤ 2b` compound lemma is arithmetically correct.
- **RS evaluation-form prefix-extensibility (§6.2):** correct and unchanged. Any prefix `P₁…Pₘ` is a
  valid `[K′+m, K′]` MDS code, distance `m+1`; append-only; generator-poly form correctly excluded.
- **RAID r=1/r=2 MDS (§7.1):** correct and unchanged. `P₁=Σxᵢ`, `P₂=Σαⁱxᵢ`; `[n+r,n]` recovers any `r`
  of `n+r`; `P₁` unchanged when `P₂` appended; `ord(α) ≥ n_max` frozen.
- **RAID privacy (§7.3):** correct; lone parity plate leaks nothing below the legitimate threshold; the
  r=2 off-by-one fix is intact.
- **`2·subs + erasures ≤ m`, `⌊m/2⌋`-correct, refuse-beyond-budget lever (§6.2, §9a):** correct RS
  arithmetic; the miscorrection-within-budget gap is now handled by the (mandated non-linear) integrity
  tag.
- **Ladder arithmetic (§6.4/§9.1):** 62/69/82, parity 6/20/46 → corrects 3/10/23, appendable ~1985 — all
  internally self-consistent.
- **I-A semantics (§6.3 only):** the *mechanism* is correct and unambiguous — single meaning,
  deliberate-stop no false flag, lost-tail still flagged because the ledger is front-anchored. The defect
  is purely that §8/§9.5/§6.1 were not synced to it.

---

## Gate decision

**RED. 0 Critical / 1 Important (NEW-I-1) + 3 non-blocking Minor/Nit.** Both round-2 Criticals (NEW-C1,
NEW-C2) and both mirror Importants (I-B, I-C) and all Nits are genuinely closed; wire-format citations
and core math are intact. The lone blocker is the **incomplete I-A fold**: §6.3 was reworked to the
`recorded-length` ledger but §8 (decoder), §9.5 (freeze), and §6.1 still reference the removed
`declared-total-length` / "fixed declared length," so the decoder algorithm still encodes the rejected
truncation primitive and §9.5 double-lists/resurrects the dropped field.

This is one mechanical propagation fold (sync §8/§9.5/§6.1 to §6.3). Once applied, I expect this spec to
be **GREEN (0C/0I)** — there are no remaining substantive design gaps; NEW-I-1 is a consistency residual
of the round-2 in-place edit, not a new design problem. Per `CLAUDE.md`, fold and **re-dispatch** (the
loop continues after every fold, and this fold edits the load-bearing §8 decoder + §9.5 freeze, which can
themselves drift).

Priority for the fold: **NEW-I-1 (§8 + §9.5 + §6.1 ledger sync)** > Minor-1 (ledger durability sentence)
> Nit-3 (§8.5 → §8 step 5 label) > Minor-2 (plan-time per-K-class split — defer to the plan's R0).
