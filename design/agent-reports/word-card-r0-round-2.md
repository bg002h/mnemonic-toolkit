# R0 architect review — Word-Card encoding brainstorm spec (round 2)

- **Reviewer:** opus architect (mandatory pre-implementation R0 gate, round 2 of the loop)
- **Spec under review:** `design/BRAINSTORM_word_card_encoding_2026-06-24.md` (R0 round-1 folds applied in-place)
- **Round-1 review:** `design/agent-reports/word-card-r0-round-1.md` (verdict RED, 3C/4I)
- **Date:** 2026-06-24
- **Spec source SHAs re-verified this round:** mk-codec @ `46631c6`, md-codec @ `7764145d`, ms-codec @ `5c0335c`, toolkit @ `60af98dd`
- **Scope:** funds-safety / custody-safety first-class. Adversarial. This gate blocks all implementation.

---

## Verdict

**RED — 2 Critical / 3 Important.**

The folds materially improved the spec and **two of the three round-1 Criticals (C2, C3) and three of the
four round-1 Importants (I2, I3, I4) are genuinely closed.** Wire-format citations were independently
re-verified against source at the cited SHAs by two parallel sub-agents and **all remain TRUE, with
exact line numbers** (table §A). The core math — RS evaluation-form prefix-extensibility, RAID r=1/r=2
MDS, parity-privacy — survives the edits unchanged and remains **sound**.

But the C1 fold introduced a new soundness gap, and two folds left an internal contradiction and an
unproven primitive:

1. **NEW-C1 (residual of round-1 C1) — the integrity tag now lives INSIDE the RS codeword (§5 step 4),
   and the spec offers a LINEAR tag option (codec BCH residue) for which the claimed `≤ 2⁻ᵗ`
   miscorrection bound is NOT established.** The hash option is sound; the BCH-residue option is
   presented as interchangeable but is not proven independent of a full-codeword RS miscorrection. The
   load-bearing custody bound is overclaimed for one of the two frozen-constant choices.
2. **NEW-C2 — the C2 bounded-desync invariant depends on the decoder being able to RE-RECOGNIZE a
   checkpoint after a desync, but the checkpoint word carries no self-identifying marker** (§6.1 packs
   the full 11 bits with running-index + local-parity, no marker class). Post-desync re-synchronization
   at `Cᵢ₊₁` — the anchor the whole bounded-desync proof rests on — is asserted but not demonstrated to
   be achievable when checkpoints are positionally indistinguishable from data words.

Two Importants: the C3 `declared-total-length` field has contradictory semantics (intended-max vs
actually-present) that make a deliberate early-stop indistinguishable from corrosion-truncation; and the
N3 ladder arithmetic the fold-log claims fixed is still internally inconsistent (`K′=62` computed vs
`61` hard-coded in the §6.4 ladder).

Per `CLAUDE.md`, no plan-doc and no code until this converges to 0C/0I. Re-dispatch after the fold.

---

## Closure of round-1 findings

| Finding | Status | Notes |
|---|---|---|
| **C1** (silent miscorrection) | **PARTIALLY** | Self-referential round-trip correctly removed; independent-tag concept + numeric bound added. BUT the tag is now a data symbol *inside* the same RS codeword (§5 step 4 `header ∥ payload ∥ integrity-tag`), and the spec offers a *linear* BCH-residue option whose `≤ 2⁻ᵗ` bound is unproven. → **NEW-C1** below. |
| **C2** (indel→erasure / compound-deletion desync) | **MOSTLY CLOSED** — one residual | Pinpointing now NORMATIVE (§6.1); honest block-granular fallback (cost ≤ `b`); §9(b) now consistent with §6.4/§9.1; the compound checkpoint+data deletion is named and bounded to ≤ 2b erasures via running-index anchors. The *reasoning* is sound IF checkpoints are re-recognizable post-desync — which is the new gap. → **NEW-C2** below. The math of the bound itself is correct. |
| **C3** (stop-sign / lost-tail downgrade) | **CLOSED (concept)** — one semantic ambiguity | ≥2-word stop-sign sized correctly; monotone `declared-total-length` + `words-present < declared ⇒ truncation flag` genuinely flags the lost-newest-tail downgrade that a stop-sign alone cannot. The steel-medium pre-commit-vs-appendable choice is acceptably left to the plan. BUT the pre-commit option collides with deliberate early-stop. → **I-A** below (Important, not Critical: downgrade is now *flaggable*; the residual is a false-positive, not a silent downgrade). |
| **I1** (split guarantee) | **CLOSED** | §9 now cleanly splits (a) value-layer MDS vs (b) indel-layer sync-bounded, with the C2 conditionality stated. |
| **I2** (RAID auto-suppression) | **CLOSED** | §7.4 flipped: RAID retained by default; suppression is explicit, coverage-verified opt-in; single-`md1`-plate SPOF stated plainly. |
| **I3** (normative striping width) | **CLOSED** | §7.1 "Prerequisite (NORMATIVE)" + §9.5 frozen padding rule; Q5 demoted to "exact padding rule only." The MDS-depends-on-alignment dependency is now stated as a requirement. |
| **I4** (frozen constants) | **CLOSED** | §9.5 added with field poly, symbol map, RS eval sequence, RAID `α` with `ord(α) ≥ n_max`, tag width, header layout, padding rule. Comprehensive. |
| Nits N1/N2/N4/N6 | **CLOSED** | N1 (RS attribution) §6.2 corrected; N2 (r=2 privacy off-by-one) §7.3 corrected; N4 (version-sites) §10 expanded; N6 (pre-chunking) §5 step 4 added. |
| Nit N3 | **NOT CLOSED** | Fold-log claims "N3 `K′≈61–62`" but only §9 prose was softened; the §6.4 ladder still hard-codes `K′≈61` / "words 1–61" / "⟐ 68" / "⟐ 81" while `ceil(54/7)=8 ⇒ K′=62`. → **Nit-1** below. |
| Nit N5 (small-`K` detection-always floor) | **NOT ADDRESSED** | Round-1 N5 (commit detection-always for all `K≥1`, pin small-`K` floor) was not folded and is not in the fold-log. Still only §12-Q7. → **Nit-2** (was Minor in round-1; restating, author discretion). |

---

## New Critical

### NEW-C1 — the integrity tag is now a data symbol INSIDE the RS codeword, and the offered BCH-residue tag option does not establish the `≤ 2⁻ᵗ` bound

**Where:** §5 step 3 (line 136-141, "the source `m*1` codec BCH residue, **or** a ≥32-bit truncated
hash"); §5 step 4 (line 142, "Concatenate **header ∥ payload ∥ integrity-tag bits** and regroup 8→11");
§8 step 5 (line 335-340); §9 custody bullet (line 364-368); §9.5 ("Integrity tag: function + bit-width").

**The structure.** The fold made the tag part of the `K` data words, which are RS-encoded inside the
*same* codeword as the payload. Round-1's C1 hole was a *full-codeword* RS miscorrection: the decoder
lands on a different valid codeword, which moves **every** data symbol — payload *and* the in-codeword
tag — coherently. So the round-1 "the array-id moves with the miscorrection, the check is circular"
critique applies verbatim to an in-codeword tag *unless* the tag's relationship to the payload is one
the RS codeword cannot self-satisfy.

**Why the HASH option is sound (and salvages the design).** The RS code is *linear* over GF(2¹¹). A
truncated cryptographic hash is *non-linear*. A valid RS codeword constrains its data symbols only
through the linear evaluation/interpolation relation; it has **no** mechanism to force `tag-symbols ==
hash(payload-symbols)`. So for a wrong-but-valid codeword `B'` with in-codeword tag `T'`, the event
`hash(B') == T'` is ≈ `2⁻ᵗ` (the hash output is effectively uniform and independent of the linear
constraint). **The in-codeword placement is fine for a non-linear tag.** Good.

**Why the BCH-RESIDUE option is NOT established.** The spec offers, as an equal alternative, "the source
`m*1` codec BCH residue." The BCH residue is a **linear** function of the payload (over GF(2⁵) bech32
symbols). Composing one linear code (the m*1 BCH check) with another linear code (the GF(2¹¹) RS) does
**not** give an independent `2⁻ᵗ` guarantee the way a hash does — linear/linear compositions can have
correlated codewords, and the spec presents zero argument that the BCH-residue tag is uncorrelated with
RS miscorrections. The `≤ 2⁻ᵗ` bound in §5/§8.5/§9 is asserted for *both* options indiscriminately. For
a steel-backup custody tool this is exactly the over-claim class round-1 C1 was raised to kill: the
headline custody bound holds for one frozen choice and is unproven for the other, and §9.5 lets the
plan-doc freeze *either*.

**Severity = Critical.** This is the load-bearing no-silent-miscorrection guarantee, it is overclaimed,
and §9.5 would let an implementer freeze the unproven (linear) construction for 20 years. The design is
salvageable (the hash option works), so the fix is a constraint, not a redesign — but it must close
before a plan freezes the constant.

**Recommended fold:**
- **Mandate a non-linear (cryptographic-hash) integrity tag and DROP the "codec BCH residue" option**
  (or, if the BCH-residue option is kept, *prove* its independence from a full-codeword RS miscorrection;
  absent a proof, it must go). State explicitly that the `≤ 2⁻ᵗ` bound relies on the tag being a
  non-linear function the linear RS codeword cannot self-satisfy.
- In §9.5 ("Integrity tag: function + bit-width"), pin the function family as **a truncated
  cryptographic hash (e.g. SHA-256[0..t])**, not "function = OPEN," so the unproven option can't be
  frozen later.
- One sentence in §8 step 5 noting the tag is in-codeword and *why* that is safe for a non-linear tag
  (so a future reader doesn't "optimize" it into a linear residue).

---

### NEW-C2 — the bounded-desync invariant requires post-desync checkpoint RE-RECOGNITION, but checkpoints carry no self-identifying marker

**Where:** §6.1 (line 175-178 "Checkpoints are themselves RS-coded symbols … inside the Layer-C
codeword"; line 183-189 the bounded-desync invariant; line 163-164 "checkpoint word carrying a running
block index + a local parity"); §8 step 2 ("walk checkpoints"); §12-Q2 (remaining open = "the exact
11-bit split (index vs parity)").

**The invariant's load-bearing primitive.** The C2 fold bounds every indel — including the compound
"`Cᵢ` deleted AND a data word in block `i` deleted" case round-1 raised — to ≤ block (or ≤ 2b for the
merged span) erasures. The proof structure is: anchor the desynced span between two **validated
checkpoints** `Cᵢ₋₁` (before) and `Cᵢ₊₁` (after), mark the merged span as erasures, and **re-synchronize
at `Cᵢ₊₁`** because its running index pins the absolute block number. This re-synchronization at `Cᵢ₊₁`
is the single fact that converts "unbounded whole-codeword desync" into "≤ 2b bounded erasure." It is
correct **only if the decoder can positively identify that a given word IS `Cᵢ₊₁`** during the sync pass
— *before* RS correction (step 2 precedes step 3).

**The gap.** A checkpoint is one 11-bit word holding "running block index + local parity" (§6.1) — and
§12-Q2 confirms the *entire* 11 bits are consumed by the index/parity split, leaving **no bits for a
'this word is a checkpoint' marker class.** So a checkpoint is **not structurally distinguishable from a
data word** by inspection. In the no-indel case this is fine: checkpoints are at *known positions*
(every `b+1`th slot). But the whole point of the sync pass is the *indel* case, where positions have
shifted — and there the decoder cannot use position to find `Cᵢ₊₁` (that's circular: it's looking for
the checkpoint precisely because the count is off), and it cannot use structure (no marker). The
reinsert-and-test machinery (§6.1) probes candidate positions, but the spec never demonstrates that a
data word cannot masquerade as a valid-index-valid-parity checkpoint at a shifted position, nor that the
true `Cᵢ₊₁` is uniquely recoverable among the `O(b)` candidate alignments. The bounded-desync invariant
is therefore **asserted, not demonstrated**, for exactly the case it was added to handle.

I confirmed by grep that the spec contains no language about checkpoint markers, self-identification, or
how checkpoints are recognized post-desync (only the unrelated "safety marker itself is wrong" / stop-sign
uses of the word "marker").

**Severity = Critical.** This is the same class as round-1 C2 (a custody/correctness invariant whose
load-bearing primitive is unproven), re-surfaced one level deeper by the fold. If checkpoint
re-recognition can fail or be ambiguous, a single deleted checkpoint can still desync the codeword — the
exact failure round-1 C2 demanded be closed. The fold closed the *budget accounting* but not the
*recognizability* it depends on.

**Recommended fold:**
- Specify **how a checkpoint is recognized during the sync pass under desync.** Options to pin: (a) a
  dedicated marker bit-field in the checkpoint word (which trades index/parity bits — re-derive the
  §12-Q2 11-bit split to fit it, and re-cost the local-parity strength); (b) a global "expected
  checkpoint positions from `declared-total-length`" grid that the reinsert-and-test search realigns
  against, with a stated argument that the realignment is **unique** (the true `Cᵢ₊₁` validates and no
  data word at a shifted position does, to probability ≥ `1 − 2⁻something`); or (c) accept that an
  unrecognizable checkpoint degrades to refuse-and-report (never guess), and state that path.
- Add an explicit **lemma** (even informal) for the compound case: "given validated anchors `Cᵢ₋₁` and
  `Cᵢ₊₁`, the merged span is ≤ 2b erasures AND re-sync at `Cᵢ₊₁` is achievable because [recognition
  mechanism]." The §6.1 invariant currently states the conclusion without the recognition premise.
- Restate §6.1's "this closes the safety-marker-itself-is-wrong hole" honestly: it closes the
  *miswritten* checkpoint (RS repairs it), but the *deleted* checkpoint requires the recognition
  mechanism above, which is currently missing.

---

## New / promoted Important

### I-A — `declared-total-length` has two contradictory meanings (intended-max vs actually-present); a deliberate early-stop trips a FALSE truncation flag

**Where:** §6.3 (line 217 "MONOTONE `declared-total-length`, bumped on every upgrade"; line 219-221
"words physically present **<** `declared-total-length` ⇒ truncation flag"; line 226-230 steel-medium
"either a **pre-committed** `declared-total-length` at creation (the user declares the **max tier they
intend to reach**) or a header region designed to be appended"); §8 step 1 (line 320-322).

**The contradiction.** The truncation test is `words-present < declared-total-length ⇒ truncation`. That
is correct **only if** `declared-total-length` means "the number of words that SHOULD currently be
present." But §6.3's own steel-medium discussion offers the **pre-commit** option, under which
`declared-total-length` = "the *max tier the user intends to eventually reach*." Under pre-commit, a user
who legitimately stops at tier 1 (the word-ladder §6.4 *explicitly invites* stopping at any checkpoint)
has `words-present < declared-total-length` **by design** — and the decoder raises a **truncation flag on
a perfectly intact plate**. The two readings give opposite decoder verdicts on the same plate. A
false-truncation-on-every-read for the common "I stopped early on purpose" case is a real defect for a
tool whose UX is graduated voluntary stop points.

**Why Important not Critical:** the *silent-downgrade* hole round-1 C3 raised IS closed — the worst case
is now a false *positive* (warns when it shouldn't), not a false negative (silently under-protects). A
spurious warning is a usability/trust defect, not a funds-safety defect. But it directly contradicts the
ladder's "stop at any checkpoint" promise and must be reconciled before the plan freezes the header
field semantics (it's in §9.5 frozen constants).

**Recommended fold:** pin **one** semantics. Cleanest: `declared-total-length` = "words that should be
present *at the tier this plate was last written to*," updated in lockstep with each append (the
append-only / appendable-header mechanism), so present<declared ⇒ genuine truncation. If the
pre-commit-max mechanism is kept, **separate the two fields** — a `max-intended-tier` (advisory, never
trips truncation) vs a `committed-length` (the actual current length, used for the truncation test) —
and state that a deliberate early-stop sets `committed-length = present` and trips nothing. Either way,
reconcile §6.3 lines 219-221 with lines 226-230 so they cannot be read to contradict.

### I-B — §9.5 frozen-constants list does not pin the integrity-tag FUNCTION (only "function + bit-width"), leaving the NEW-C1 unproven option freezable

**Where:** §9.5 line 392 ("Integrity tag: function + bit-width `t` (§5.3)").

This is the §9.5 side of NEW-C1, called out separately because the round-2 prompt asks whether §9.5
contradicts §5–§8. It does not *contradict* them, but it *under-pins* exactly the constant whose value
determines whether the custody bound holds. "function = (BCH residue OR hash), pick later" is precisely
what must NOT be deferred, because freezing the linear option silently voids §9's `≤ 2⁻ᵗ` claim 20 years
after anyone can reason about it. **Recommended fold:** change §9.5 line 392 to name the function family
(non-linear truncated hash, specific algorithm + truncation length), consistent with the NEW-C1 fold.
(If NEW-C1 is folded as recommended, this collapses into it — listing it ensures the §9.5 line is
actually edited, not just §5.)

### I-C — checkpoint-recognition cost/uniqueness is not in §9.5 frozen constants either, so two implementations could disagree on the sync primitive

**Where:** §9.5 line 393 ("Stop-sign + checkpoint local-parity encodings: field widths"); §12-Q2.

§9.5 freezes the checkpoint *field widths* but not the **recognition / realignment rule** (the NEW-C2
primitive). For 20-year recoverability the decoder's checkpoint-realignment algorithm under desync is as
load-bearing as the RS eval sequence — two implementations that realign differently will recover
different grids from the same damaged plate. **Recommended fold:** once NEW-C2 pins the recognition
mechanism, add it to §9.5 (the realignment rule / marker-field layout / uniqueness criterion), so the
sync primitive is frozen alongside the RS and RAID primitives.

---

## Minor / Nit

- **Nit-1 (N3 not fully folded) — §6.4 ladder hard-codes `K′≈61` and derived stop points while the
  arithmetic gives `K′=62`.** `K=54, b=7 ⇒ ⌈54/7⌉ = 8` checkpoints ⇒ `K′ = 62`. The fold-log claims N3
  fixed via "`K′≈61–62`," but only §9 (line 363 "`K′ ≈ 61–62`") was softened. §6.4 still reads
  "`K≈54`, `K′≈61`", "MANDATORY words 1–61", "At 61: …", "⟐ 68 (7 check)", "⟐ 81 (20 check)". With
  `K′=62` those become 62 / 1–62 / 69 / 82. Also §6.1 line 162 still says `K=160 ⇒ b≈12` where
  `√160=12.65 ⇒ 13` (round-1 N3, untouched). Harmless to correctness, but a doc-example that won't match
  a deterministic generator's real output — and `verify-examples` discipline (§10) will eventually CI-gate
  these blocks against the binary, so fix the ladder to the real numbers now. **Fold:** recompute the
  §6.4 ladder for `K′=62` (or whatever the plan's exact `K′` is) and reconcile §6.1's `b≈12` vs `13`.

- **Nit-2 (round-1 N5, unaddressed) — small-`K` detection-always floor still only an open question.**
  Round-1 N5 asked the spec to *commit* that detection-always (§9(b) line 355) holds for all `K ≥ 1` and
  to pin a fixed parity/sync floor for tiny keyless templates (`K<10`, where `√K<3.2` degenerates sync).
  Not folded; still only §12-Q7. This is the one guarantee that must never degrade with size. **Fold
  (author discretion, cheap):** one sentence committing detection-always for all `K≥1` and a fixed
  minimum checkpoint count / parity floor for small `K`.

- **Nit-3 (informational) — §9.1/§9 "~1980 appendable" vs computed ~1985.** `n_max − K′ = 2047 − 62 =
  1985`. "~1980 … effectively unbounded" is within tolerance; no action needed beyond consistency with
  whatever `K′` the Nit-1 fix lands on.

---

## What is SOUND (re-confirmed this round — do not re-litigate)

- **Wire-format facts:** independently re-verified at the cited SHAs by two parallel sub-agents — **all
  TRUE, line numbers match** (table §A). mk1 73-B compact xpub / 65-B incompressible / depth-child
  reconstruction / 53-B chunking; md1 `encode_payload` order / TLV `0x02` 65-B value / 5-bit-tag+varint
  framing / `is_wallet_policy`; ms1 `0x00` prefix + 16/20/24/28/32-B entropy. Zero drift after the edits.
- **RS evaluation-form prefix-extensibility (§6.2):** correct and unchanged. Any prefix `P₁…Pₘ` is a
  valid `[K′+m, K′]` MDS code, distance `m+1`; append-only because each parity word is an evaluation at a
  new fixed point. Generator-poly form correctly excluded. N1 naming now correct.
- **RAID r=1/r=2 MDS (§7.1):** correct and unchanged. `P₁=Σxᵢ`, `P₂=Σαⁱxᵢ` are the first two syndromes;
  `[n+r,n]` recovers any `r` of `n+r` plates; `P₁` unchanged when `P₂` appended (append-only at plate
  granularity); `ord(α) ≥ n_max` now frozen in §9.5 (I4). "Lose any `r` of `n+r`" honesty intact.
- **RAID privacy (§7.3):** correct, and the N2 r=2 off-by-one is now fixed ("r=2 needs `n−2` reals +
  both parity"). A lone parity plate leaks nothing below the legitimate recovery threshold.
- **The `2·subs + erasures ≤ m`, `⌊m/2⌋`-correct, refuse-beyond-budget lever (§6.2, §9a):** correct RS
  arithmetic. The miscorrection-within-budget gap is now (modulo NEW-C1) handled by the integrity tag.
- **`ms1` exclusion (§1.1):** correct and well-argued.
- **C2 budget accounting and C3 monotone-length downgrade detection:** the *math/logic* of both folds is
  correct; the residual issues (NEW-C2 recognition, I-A semantics) are about unproven/under-specified
  primitives the folds rest on, not arithmetic errors.

---

## A. Wire-format re-verification (parallel sub-agents, this round)

All claims TRUE at cited SHAs; line numbers confirmed (minor ±1 doc-comment-vs-code-line notes only):

| Claim | Verdict | Source |
|---|---|---|
| mk1 compact xpub = 73 B | TRUE | `mnemonic-key/crates/mk-codec/src/consts.rs:53`; assertion `:115` `73=4+4+32+33` |
| incompressible 32+33=65 B | TRUE | `…/bytecode/xpub_compact.rs:32` struct; `:38` chain_code[32], `:40` public_key[33] |
| drops depth/child, reconstructs from path | TRUE | `xpub_compact.rs:4-6`, `reconstruct_xpub :86-108` (depth=path-len `:89`, child=last `:94-97`) |
| 53-B chunking + cross_chunk_hash | TRUE | `…/string_layer/chunk.rs:50-94` `split_into_chunks`; `CHUNKED_FRAGMENT_LONG_BYTES=53` |
| md1 `encode_payload` header∥path∥use-site∥tree∥TLV | TRUE | `descriptor-mnemonic/crates/md-codec/src/encode.rs:65-92` (order `:84-89`) |
| md1 TLV `0x02`, 65-B value | TRUE | `…/tlv.rs:16` `TLV_PUBKEYS=0x02`; value `[u8;65]` `:29-32`, `:166-168` |
| 5-bit tag + varint framing ⇒ framed >65 B | TRUE | `…/tlv.rs:202-206` (`write_bits(tag,5)`, `write_varint`) |
| keyless template embeds no xpubs | TRUE | `encode.rs:50-52` `is_wallet_policy` false when pubkeys None/empty |
| ms1 `0x00` prefix + 16/20/24/28/32-B entropy | TRUE | `mnemonic-secret/crates/ms-codec/src/consts.rs:29` `VALID_ENTR_LENGTHS`, `:17` `RESERVED_PREFIX=0x00`; `envelope.rs:236` push |

---

## Gate decision

**RED. 2 Critical (NEW-C1, NEW-C2) / 3 Important (I-A, I-B, I-C).** Round-1 C2, C3, I1, I2, I3, I4 and
nits N1/N2/N4/N6 are closed. The two new Criticals are residuals the folds introduced (tag-inside-codeword
with a freezable linear option; bounded-desync resting on un-specified checkpoint recognition); the three
Importants are an unreconciled `declared-total-length` semantics contradiction and two §9.5 freeze gaps
that mirror the Criticals.

Per `CLAUDE.md`, no plan-doc and no implementation until 0C/0I. Fold and **re-dispatch** — the
loop continues because these folds (the non-linear-tag mandate and the checkpoint-recognition mechanism)
are themselves design decisions that can drift.

Priority order for the fold: **NEW-C1 (custody bound) > NEW-C2 (sync correctness) > I-A (downgrade-flag
semantics) > I-B / I-C (§9.5 freeze gaps, collapse into the Criticals' folds) > Nit-1 (ladder
arithmetic) > Nit-2 (small-K floor)**.
