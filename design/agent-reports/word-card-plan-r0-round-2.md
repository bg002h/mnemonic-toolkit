# Plan-doc R0 review — Engravable Word-Card encoding — ROUND 2

- **Artifact under review:** `design/IMPLEMENTATION_PLAN_word_card_encoding.md` (round-1 folds applied)
- **Authoritative spec (R0-GREEN):** `design/BRAINSTORM_word_card_encoding_2026-06-24.md` (commit `31109f8e`)
- **Round-1 review (folded):** `design/agent-reports/word-card-plan-r0-round-1.md` (RED 2C/3I/5n)
- **Reviewer:** opus architect (mandatory pre-implementation plan-doc R0 gate; 0C/0I required; reviewer-loop continues after every fold)
- **Date:** 2026-06-24
- **Repo HEAD at review:** `a552a242` ("fold Word-Card plan-R0 round-1 (2C/3I/5n)")

---

## Verdict

**RED — 0 Critical / 2 Important / 4 Minor-Nit.**

Both round-1 **Criticals are genuinely closed.** I re-verified the load-bearing math by machine:
the CRC-5 generator `x⁵+x²+1` is irreducible+primitive and gives a **slot-independent** single-word
substitution miss of **63/2047 ≈ 3.08% (≤ 2⁻⁵)** — the old integer-mod-32 slot-7 25% blind spot is
gone (C2 ✅). The C1 deletion-pinpoint mechanism is now the **GLOBAL RS+non-linear-tag**, which is
the correct construction; the false-positive ambiguity is bounded by `(b−1)·2⁻ᵗ ≈ 2⁻⁴⁰`, negligible
(C1 ✅). The **field/RS/RAID algebra is byte-identical to round-1's verified §3** — the folds did not
perturb it (re-confirmed). Deps are accurate.

The block is that the **round-2 folds introduced/left two Important consistency defects in the
frozen wire layout**, both in §4.2/§3 (the header self-description the folds rewrote):

1. **NEW-I1** — the I3 GEOM fold closed the `K`/`payload_len` circularity but left the **ledger-length
   leg open**: the closed-form `K′` and `m_present` now *explicitly depend on* `|ledger|` (§4.2), yet
   the variable-length ledger has **no closed-form read** — its end is only marker-delimited, which
   collides with payload data at **1/16 per word ⇒ ~6% chance a pristine card mis-locates payload
   word 0.** That is unacceptable for a frozen format.
2. **NEW-I2** — an **H1 bit-budget contradiction**: `n−1` is 5 bits (`n_max = 32`) but `index-in-array`
   is only 3 bits (max 8), and the `P₂` α-exponent is bound to that 3-bit index. RAID r=2 MDS requires
   *distinct* exponents `i = 0..n−1`, so the construction **silently breaks for n > 8** (a 15-of-15
   multisig is realistic). The plan contradicts itself (L110 `n_max=32` vs L154 `index 3b`).

Neither is a funds-*loss* hole (RS+tag never silently miscorrect; the failures are mis-locate /
refuse / non-MDS-reconstruct), but both are **frozen-constant correctness defects** that contradict
the plan's own normative claims — which the gate treats as blocking. Both are localized header-layout
fixes; the math foundation is solid.

### Math I re-verified myself this round (NOT rubber-stamped)

| Claim (plan §) | Result |
|---|---|
| `x⁵+x²+1` (=`0b100101`) irreducible over GF(2): not divisible by `x`, `x+1`, `x²+x+1` | ✅ TRUE |
| `x⁵+x²+1` **primitive**: `ord(x) = 31 = 2⁵−1` (computed); automatic since 2⁵−1 prime | ✅ TRUE |
| CRC-5 single-**word** (11-bit) substitution miss = 63/2047 ≈ **3.08% ≤ 2⁻⁵**, **slot-independent** (all 8 slots identical) | ✅ TRUE — uniform, no blind spot |
| CRC-5 detects **all** single-bit errors; **all** double-bit errors within an 11-bit word (min missed span k=31 > 10) | ✅ TRUE |
| CRC-5 kernel for a value-free erased slot = exactly 2⁶ = 64 matching values ⇒ **CRC cannot prune a deletion candidate** (value is free) | ✅ TRUE (see C1 residual) |
| C1 false-positive ambiguity bound = `(b−1)·2⁻ᵗ` (union over ≤ b candidates), ≈ 2⁻⁴⁰ at b≈12, t=44 | ✅ negligible |
| GEOM closed form: `K=⌈(8·73+44)/11⌉=58`, `b=round√58=8`, `checkpoints=⌈58/8⌉=8` | ✅ reconciles |
| `payload_len` 2 words (≥2¹⁶ B, actually 22 bits) ≫ largest md1 (~1500 B) and ≫ RS cap (~2800 B) | ✅ sufficient |
| §3 field/RS/RAID constants (poly `0x805`, α=x ord 2047, systematic eval-form `βⱼ=αʲ`, RAID `P₁=Σxᵢ`/`P₂=Σαⁱxᵢ`) byte-identical to round-1's verified text | ✅ UNCHANGED |
| Deps: `sha2="0.10"` (Cargo.toml:47), `bip39 v2 all-languages` (:49), members=`["crates/mnemonic-toolkit"]` (:2) | ✅ accurate |

---

## Closure of round-1 findings

### C1 — deletion pinpoint — **CLOSED (substantively), with 2 minor residuals**

The fold replaces the impossible local-pinpoint with the right mechanism: a single in-block deletion
becomes one erasure, the ≤ b candidate gap positions are enumerated, and **each is validated by the
GLOBAL RS-decode + non-linear integrity tag** (§4.3 L186–190). This is sound — the global tag is the
only thing that can distinguish alignments when the value is a free unknown.

I probed the two specific questions:

- **"Could two gap alignments BOTH pass the integrity tag (collision at 2⁻ᵗ)?"** The correct
  alignment passes (~always); a *wrong* alignment passes only if RS miscorrects to a valid codeword
  AND its SHA tag matches, i.e. `≤ 2⁻ᵗ` each. Union over the ≤ b candidates ⇒ **`P(ambiguity) ≤
  (b−1)·2⁻ᵗ ≈ 12·2⁻⁴⁴ ≈ 2⁻⁴⁰`.** Negligible — the §9(b) honest-budget claim survives. *(Residual: the
  plan states the per-alignment `≤ 2⁻ᵗ` but not the `(b−1)·` union; state it so the bound is honest.)*
- **"Is the candidate-search bound well-defined for multiple deletions?"** Yes for the safety property,
  but the plan's framing is imprecise. The §4.3 "budget exceeded ⇒ refuse" cap *does* prevent
  exponential blowup. However, **d separate single-deletions in d distinct blocks** require a
  `b^d` cross-product of joint alignments **only if** you insist on cost-1 recovery for all of them
  simultaneously; the always-available linear escape is to **whole-block-erase each flagged block
  (d·b erasures, zero search).** The plan lists both fallbacks ("ambiguous/too-large search ⇒
  whole-block erasure" AND "global budget exceeded ⇒ refuse") but does **not say** that multi-block
  single-deletions should take the per-block-erase path rather than the cross-product (which would
  refuse perfectly-recoverable cards). See Minor-1.

**Verified claim "CRC-5 prunes the candidate gap positions" is WEAK for deletions:** because the
deleted value is a free unknown, for *every* candidate slot there are exactly 2⁶ = 64 erased-word
values that satisfy the local CRC-5 (kernel of the surjective 11→5 bit map). So **CRC-5 prunes
nothing when the value is free** — all b slots reach the global validation. The bound stays ≤ b
(correct), but the §4.3 phrase "candidate gap positions **pruned by the CRC-5**" (L188) overstates
CRC's role for the deletion case. See Minor-2.

Net: the **safety story holds** (refuse/erase fallback, no silent miscorrection); the capability
claim is sound up to the two wording refinements. **CLOSED** as a Critical.

### C2 — CRC-5 polynomial quality — **CLOSED (verified)**

`x⁵+x²+1` is irreducible and primitive (`ord(x)=31`). Single-word 11-bit substitution miss is a
**uniform 3.08% (≤ 2⁻⁵) at every slot** — I exhaustively swept all 8 slots and all 2047 nonzero
deltas; the old slot-7 25% hole is eliminated. It also detects all single-bit and all within-word
double-bit errors. This **strictly dominates** the integer weighted-sum it replaced. The polynomial
is a genuinely good degree-5 CRC for this use. **CLOSED.** *(One honest caveat worth a one-liner:
`x+1 ∤ g` (g(1)=1), so CRC-5 does NOT guarantee detection of all odd-weight bit errors — irrelevant
under the per-word-delta model and the global RS+tag net, but state the floor as "single-symbol-sub
≤ 2⁻⁵," not "all errors.")*

### I1 — ledger granularity reaches 2047 — **CLOSED**

§4.2 L168 ledger entry = `marker(4:0b1110) │ cumulative-count(11:0..2047) │ checksum(7)` = 22 bits =
2 words, an **exact 11-bit count reaching the `n ≤ 2047` cap**. The ×16/2032 shortfall is gone.
Consistent with the §4.4 stop-sign (also `4│11│7`, exact 11-bit). The truncation test (§4.4 L201–204)
uses `max(ledger entries, highest stop-sign)`, both exact — a near-cap top truncation is no longer
missed. **CLOSED.** *(Nit: the ledger entry's `checksum(7)` coverage is unspecified — the stop-sign
says `SHA-256(preceding)[0..7]`; say what the ledger checksum covers. Minor-4.)*

### I2 — codec round-trip — **CLOSED**

P0 (§7 L270–276) is now an explicit adversarial multi-vector KAT (multi-chunk mk1 cross-chunk-hash,
multi-`0x02`-TLV md1 ordering, keyless-template md1) asserting `assemble∘disassemble = id`
byte-identically, **with the NO-BUMP question resolved before P1** ("if the accessor must canonicalize
(re-sort TLV / recompute a hash) it is a PATCH, not NO-BUMP"). This directly addresses the two named
hazards. **CLOSED.**

### I3 — header circularity — **PARTIALLY closed → reopened as NEW-I1**

The K/`payload_len` leg is **closed**: GEOM stores `payload_len` explicitly (2 words), read
**positionally before RS** at a deterministic offset (`1 + has_raid·3` from H0), guarded by a
`header-CRC`, with the whole geometry in closed form (§4.2 L158–165). `header-CRC fail ⇒ refuse` is
acceptable — the header is a few words the human re-reads, and it is *also* inside the big RS for
correction on a clean re-read, so a single header typo does **not** brick an otherwise-recoverable
card (the refuse is a re-read prompt, not a permanent loss). `payload_len` 2 words is more than
enough for the largest md1 (~1500 B) and even the RS field cap (~2800 B). **Good.**

**But the ledger-length leg is still open** — and the fold made it *load-bearing* by writing
`K′ = |header| + K + checkpoints` and `m_present = words_present − K′ − |stop-sign| − |ledger|`
(§4.2 L162–163), both of which depend on `|ledger|`, a variable quantity with **no stated closed-form
read**. This is the round-2 regression — see **NEW-I1**.

### Minors / Nits from round-1

- **M1** (K reconciliation): **CLOSED** — §4.1 L141–148 pins payload 73 B ⇒ K=58, b=8, checkpoints=8,
  and explicitly supersedes the spec's illustrative K=54 ladder. *(Residual nit: §4.1 L147 writes
  `K′ = 58 + 8 (+ header)` — the `(+ header)` is vague; §4.2 gives the exact `K′ = |header| + K +
  checkpoints`, but `|header|` is never assigned a concrete number anywhere. Minor-3.)*
- **M2** (mod-8 aliasing bound): **CLOSED** — §4.3 L181–183 states the bound (`≥ 8·b` consecutive
  destroyed = whole-payload refuse) and adds the P3 KAT (§7 L287).
- **M3** (P₂ exponent source + array-id target): **INCONSISTENTLY closed** — §3 L120 adds the fix
  ("`i` = H1's `index-in-array`, NOT array-id") but L109 still reads "`i` = stripe index, **fixed by
  `array-id`**." §3 now contradicts itself in two adjacent bullets. See Minor-1-bit / actually this
  feeds NEW-I2. The array-id collision target (`≤ 2⁻²²`) is stated (L119). *(The contradiction is
  tracked under Minor-1 below; the bit-width problem it exposes is NEW-I2.)*
- **M4** (padding freeze at P1/P2): **CLOSED** — §3 L122–123 + §7 P1 L278–279 freeze the
  zero-pad-to-array-max rule in P1, so P5 RAID is self-contained.
- **N5** (SHA refresh): **PARTIALLY closed** — plan L7 now cites toolkit `d08b0d51` (was the spec's
  `31109f8e`), but HEAD is `a552a242`. `d08b0d51` was the round-1 *review* HEAD; the only commit since
  is the fold commit itself. Acceptable as "write-time SHA," but per CLAUDE.md it should track current
  origin/master. Minor refresh. The spec commit `31109f8e` (L5) is verified correct.
- **N6** (error.rs placement): **CLOSED** — §6.2 L259–262 places `ToolkitError::WordCard`
  alphabetically among the post-v0.27.2 sorted variants, explicitly NOT interleaving the unsorted
  pre-v0.27.2 block. Matches CLAUDE.md.

---

## New Critical

**None.** The field/RS/RAID algebra is unchanged and re-verified; C1/C2 are genuinely closed; no
new funds-loss or silent-miscorrection path was introduced. The two new defects below are Important
(frozen-format correctness contradicting the plan's own claims), not Critical (no silent fund loss —
they manifest as mis-locate/refuse/non-MDS, all loud or recoverable-by-re-read).

---

## New Important

### NEW-I1 — the GEOM closed-form `K′`/`m_present` depend on `|ledger|`, but the variable-length ledger has no closed-form read; the ledger→payload boundary is marker-ambiguous against payload data (~6% mis-locate on a pristine card)

**Citation:** plan §4.2 L162–163 — `K′ = |header| + K + checkpoints`; `m_present = words_present −
K′ − |stop-sign| − |ledger|`. §4.2 L166–170 — the `recorded-length LEDGER` is "append-only,
front-anchored," each entry 2 words, **variable count** ("a new 2-word entry is appended on each
stop/upgrade"). §3 L116–118 — ledger marker `0b1110`, distinct from checkpoint `0b101` / stop-sign
`0b1111`.

**Defect.** The I3 fold made `payload_len` explicit (good), but `K′` and `m_present` are *also*
written to depend on `|ledger|` — and **`|ledger|` (the front-header length) has no closed-form read.**
The only way the plan offers to find where the ledger ends and payload word 0 begins is the
self-identifying ledger marker `0b1110`. But a **payload data word can have leading bits `0b1110`
purely by chance** (the payload is arbitrary xpub/descriptor bytes, uniform over 11-bit symbols), so:

- `P(first payload word starts 0b1110) = 1/16 = 6.25%.` On a **pristine, undamaged** card, a cold
  decoder that ends the ledger by marker-scan will then **over-read one ledger entry into the payload
  ~6% of the time**, shifting payload-start by 2 words and desyncing the entire RS grid (and the
  checkpoint stride). The `header-CRC` does **not** cover the payload, so it cannot catch the over-read.
- Symmetrically, a corrupted genuine-ledger word-1 (marker no longer `0b1110`) truncates the ledger
  early ⇒ payload-start short by 2k. Again not caught by header-CRC.

This is a **direct residual of the I3 fold**: the fold closed the `K`/`payload_len` chicken-and-egg
but the *ledger length* — which the closed forms now lean on — was left implicit and ambiguous. A
6% structural mis-locate on a clean card is not acceptable in a 20-year-frozen format.

**Why Important not Critical:** it does not silently hand back a wrong xpub (the integrity tag still
fires on a mis-located decode, so the result is *refused/garbled*, not silently-wrong-funds). But it
is a frozen-format defect that **defeats the I3 cold-decode guarantee the fold was meant to
establish** (and would make the P4 "cold-decode-from-words-only via positional GEOM" KAT flaky / RED
~6% of the time on random payloads).

**Recommended fold:** make the front-header length **fully closed-form**. Add a small
**`ledger_entry_count`** field to GEOM (e.g. 4 bits, ≤ 15 upgrades; or fold into the spare bits of the
`t(6)│b(4)` word). Then `payload_offset = |H0| + has_raid·3 + |GEOM| + 2·ledger_entry_count` is exact,
the `header-CRC` covers a **deterministic** word set (currently it "covers all positional header
words" — but "all" is undefined while the ledger length is unknown; this also fixes the header-CRC
coverage), and `K′`/`m_present` become genuinely post-RS-independent. Add the boundary case to the P4
cold-decode KAT: a payload whose first word *does* begin `0b1110`, with k ∈ {0,1,2,3} ledger entries,
must still locate payload-start exactly.

### NEW-I2 — H1 `index-in-array` is 3 bits (max 8) but `n−1` is 5 bits (`n_max = 32`); the `P₂` α-exponent is bound to that 3-bit index, so RAID r=2 MDS silently breaks for n > 8

**Citation:** plan §4.2 L153–154 — `H1` = `n−1(5: 1..32) │ role(3) │ index-in-array(3: 0..n−1 or
parity index)`. §3 L109–110 — `P₂[c] = Σᵢ αⁱ·xᵢ[c]`; `ord(α)=2047 ≥ n_max=32`. §3 L120 — "the `P₂`
stripe exponent `i` = header `H1`'s `index-in-array` field."

**Defect.** Internal contradiction in the frozen header. `n−1` at **5 bits** declares `n_max = 32`
(plan says so explicitly, L110). But `index-in-array` is **3 bits = 0..7.** The `P₂` α-exponent is
`i = index-in-array` (L120). RAID-6 / r=2 MDS recovering *any 2 of n+2* requires the n+2 column
generators (here the α-exponents) to be **pairwise distinct** — `i = 0..n−1` must all be different.
With a 3-bit index, **plates 8..31 cannot be addressed and would collide modulo the field, so the
`[n+2, n]` code is NOT MDS for n > 8** — r=2 reconstruction can fail or refuse for a perfectly
recoverable array. n > 8 is realistic (15-of-15 P2WSH multisig; large `multi_a` taproot policies).

The arithmetic doesn't close even with effort: even shrinking `role` to 2 bits (4 values suffice),
`n−1(5) + role(2) + index(5) = 12 bits > 11` — **H1 cannot hold `n_max=32` with a 5-bit index in one
word.** Either `n_max` must be honestly reduced (and `n−1` narrowed), or H1 must spill the index to a
second word, or the α-exponent must be sourced from something with full range.

**Why Important not Critical:** a non-MDS r=2 array does not silently lose funds — reconstruction
either succeeds (when the lost plates happen to avoid the degeneracy) or **fails/refuses** loudly; and
the per-plate Layer-C word tail is independent. But it is a frozen-constant contradiction that
**breaks the headline r=2 "survive any 2 of n+2 plates" guarantee for common multisig sizes** — a real
availability defect.

**Recommended fold (pick one, then re-grep §3/§4.6 for consistency):**
1. **Honest cap:** declare `n_max = 8` (RAID arrays ≤ 8 plates), narrow `n−1` to 3 bits, keep
   `index-in-array` 3 bits — then 3+3+role(3)+spare. State the n≤8 limit in §3/§4.6/§9 and note that
   larger multisigs use per-key word tails without cross-plate RAID. (Simplest; but caps a real use case.)
2. **Spill H1 to 2 words** when `has_raid` (or always), giving `n−1(5) │ role(2) │ index-in-array(5)`
   = 12 bits across 2 words, restoring `n_max = 32` and distinct exponents `i = 0..31`. Adjust the
   `|header|` accounting (and NEW-I1's `payload_offset`) for the extra word.
3. **Decouple the exponent from the narrow index:** keep `index-in-array` for display but source the
   `P₂` α-exponent from a full-range canonical stripe order derived from the (frozen) array
   fingerprint sort — but then re-verify distinctness and freeze that rule (heavier; re-opens §3).

Whichever path, **reconcile L110 (`n_max=32`) with L154 (`index 3 bits`) and the §3 L120 exponent
binding** so they cannot contradict. Add a P5 KAT: r=2 reconstruct for `n = max_n` (whatever the
frozen cap is) proving distinct exponents / MDS at the boundary.

---

## Minor / Nit

### Minor-1 — §3 self-contradicts on the `P₂` exponent source (M3 fold left the old parenthetical)
**Citation:** §3 L109 "`i` = stripe index, **fixed by `array-id`**" vs §3 L120 "the `P₂` stripe
exponent `i` = header `H1`'s `index-in-array` field, **NOT array-id**." The M3 fold added the correct
L120 but did not delete the contradicting L109 clause. **Fold:** change L109's parenthetical to "(`i`
= the per-plate `index-in-array`; see below)" so §3 is internally consistent. (This is the same field
NEW-I2 widens — do both together.)

### Minor-2 — §4.3 "candidate gap positions pruned by the CRC-5" overstates CRC's role for deletions
**Citation:** §4.3 L188. I verified the CRC-5 kernel: with the deleted value a free unknown, exactly
2⁶ = 64 word values satisfy the local CRC at *every* candidate slot ⇒ **CRC-5 prunes nothing for the
value-free deletion case.** All ≤ b slots reach the global validation (the bound is still ≤ b, so the
safety argument is fine). **Fold:** reword to "enumerate the ≤ b candidate gap positions (the CRC-5
prunes *substitution* candidates where the value is known; for a deletion the value is free so all b
slots are validated by the GLOBAL RS+tag)." Also state the **`(b−1)·2⁻ᵗ` union bound** for the
false-positive-ambiguity probability (currently only the per-alignment `≤ 2⁻ᵗ` is given), and add a
sentence that **multiple single-deletions in distinct blocks** take the per-block-erasure path
(d·b erasures, linear), reserving the joint cross-product (and its budget cap) for the
recover-all-at-cost-1 attempt — so "budget exceeded ⇒ refuse" never refuses a card that per-block
erasure would recover.

### Minor-3 — `|header|` is used in the closed form but never given a concrete word count
**Citation:** §4.1 L147 "`K′ = 58 + 8 (+ header)`" (vague); §4.2 L162 "`K′ = |header| + K +
checkpoints`" (exact symbol, no value). **Fold:** tabulate `|header|` for the two cases — solo mk1/md1
(H0 + GEOM = 1 + 4 = 5, + ledger 2·k) and RAID mk1 (H0 + H1 + array-id + GEOM = 1 + 1 + 2 + 4 = 8,
+ ledger 2·k) — and make M1's worked `K′` use the concrete number. (Ties into NEW-I1: once
`ledger_entry_count` is in GEOM, `|header|` is fully closed-form.)

### Minor-4 — ledger-entry `checksum(7)` coverage unspecified; `payload_len` "2¹⁶" vs 2-word (22-bit) capacity mismatch
**Citation:** §4.2 L168 ledger `checksum(7)` (vs §4.4 L199 stop-sign `= SHA-256(all-preceding-words)
[0..7]`); §4.2 L160 "`payload_len` (2 words, up to **2¹⁶** B)" — 2 words hold **22 bits** (up to 2²²),
not 2¹⁶. **Fold:** (a) state what the ledger checksum covers (presumably `SHA-256(this entry's
preceding header words)[0..7]`, paralleling the stop-sign); (b) fix the "2¹⁶" to "2²² (capped well
above the RS field limit)" or explicitly cap `payload_len` at 2¹⁶ and note the spare bits — either is
fine, but the text and the width must agree.

### Nit-5 — stale source SHA in plan header
**Citation:** plan L7 cites toolkit `d08b0d51`; HEAD is `a552a242` (one fold commit since). Deps
re-verified accurate at HEAD, so no decay damage. **Fold:** refresh to `a552a242` (or note the delta
is the docs-only fold commit). Spec commit `31109f8e` (L5) is verified correct against `git log`.

---

## Faithfulness & new-risk audit

- **Non-linear tag (spec C1/NEW-C1):** ✅ preserved — §3 L112–113 keeps SHA-256 truncated, linear
  in-codeword tag forbidden.
- **Bounded-desync / whole-block-erasure fallback (spec C2):** ✅ preserved and now *correctly*
  realized — the C1 fold removes the un-deliverable local-pinpoint over-claim and routes through the
  global tag + erase/refuse fallback (faithful to spec §6.1).
- **Recorded-length ledger (spec C3/I-A):** ⚠️ the I1 granularity is now faithful (reaches 2047), but
  the **ledger-length read is ambiguous (NEW-I1)** — a faithfulness gap on the cold-decode geometry,
  not on the truncation *semantics*.
- **No-silent-miscorrect (spec C1):** ✅ post-correction SHA-256 equality (§5 step 5) preserved; it is
  also correctly reused as the C1 alignment oracle.
- **RAID privacy (spec §7.3):** ✅ preserved (§4.6/§8 lone-parity KAT).
- **RAID r=2 MDS (spec §7.1):** ⚠️ **broken for n > 8** by the H1 index width (NEW-I2) — the spec's
  "any r of n+r" headline does not hold for common multisig sizes under the frozen header.
- **Field/RS/RAID algebra:** ✅ byte-identical to round-1's machine-verified §3; folds did not perturb it.
- **NEW design risk introduced by the round-2 folds:** the two header-layout defects above (NEW-I1
  ledger-length ambiguity, NEW-I2 index-width). Both are in §4.2/§3 — the exact text the folds
  rewrote. No new risk in the math layers.

## Lockstep completeness

Unchanged from round-1 and complete: schema_mirror ✅, manual `40-cli-reference` ✅, ToolkitError
alphabetical ✅, binary-identical docs ✅, CHANGELOG/READMEs/fuzz-lock/install.sh/man-pages ✅,
`wc-codec` fuzz target ✅ (§7 P6 L298), `recover --json` paired-PR coordination ✅ (§7 L299–300),
post-impl whole-diff review ✅ (§7 L302). No regressions.

---

## What must turn GREEN before code

1. **NEW-I1** — give the front-header a **closed-form length** (add `ledger_entry_count` to GEOM);
   make `payload_offset`/`K′`/`m_present` post-RS-independent and the `header-CRC` cover a
   deterministic word set; add the `0b1110`-prefixed-payload P4 cold-decode KAT.
2. **NEW-I2** — resolve the H1 `n_max=32` vs `index-in-array(3 bits)` contradiction (honest n≤8 cap,
   or spill H1 to 2 words, or full-range exponent source); reconcile §3 L110/L120 and §4.2 L154; add a
   max-n r=2 MDS boundary KAT.
3. **Minor-1..4 / Nit-5** — fold for self-consistency (§3 exponent contradiction, CRC-pruning wording
   + union bound + multi-deletion path, `|header|` concretization, ledger-checksum coverage +
   `payload_len` width, SHA refresh).

The math foundation (field/RS/RAID + CRC-5 + the global-tag C1 mechanism) is **solid and verified.**
The remaining work is entirely in the **header self-description / wire-layout bit budgets** — the same
area the round-1 folds touched. Fold → persist this review → re-dispatch for round-3.
