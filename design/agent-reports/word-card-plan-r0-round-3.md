# Plan-doc R0 review — Engravable Word-Card encoding — ROUND 3

- **Artifact under review:** `design/IMPLEMENTATION_PLAN_word_card_encoding.md` (round-1 + round-2 folds applied)
- **Authoritative spec (R0-GREEN):** `design/BRAINSTORM_word_card_encoding_2026-06-24.md` (commit `31109f8e`)
- **Round-2 review (folded):** `design/agent-reports/word-card-plan-r0-round-2.md` (RED 0C/2I/4n)
- **Reviewer:** opus architect (mandatory pre-implementation plan-doc R0 gate; 0C/0I required; reviewer-loop continues after every fold)
- **Date:** 2026-06-24
- **Repo HEAD at review:** `352b1adf` ("fold Word-Card plan-R0 round-2 (0C/2I/4n)")

---

## Verdict

**RED — 0 Critical / 1 Important / 1 Minor-Nit.**

Both round-2 **Importants are genuinely and fully closed** — I re-verified each by machine:

- **NEW-I1 (ledger closed-form):** CLOSED. The ledger is now `U` FIXED reserved 2-word slots at the
  deterministic offset `|header|−2U`, with `U` carried in the CRC'd GEOM. The boundary is read
  **positionally**, never by marker-scan, so the ~6% payload-word-0 mislocate is gone. The closed-form
  `m_present = words_present − K′ − |stop-sign|` **no longer contains the `−|ledger|` term** — the
  ledger is now inside `|header|` ⊂ `K′`, so every quantity is genuinely post-RS-independent.
- **NEW-I2 (RAID H1 / r=2 MDS):** CLOSED. H1 is now 2 words = 22 bits (`n−1(5) │ role(2) │
  index-in-array(5) │ reserved(10)` = exactly 22), the `P₂` α-exponent is the **full 5-bit
  index-in-array**, distinct over `0..n−1` for all `n ≤ 32`, and `ord(α)=2047 ≥ 32` ⇒ r=2 MDS holds
  for every realistic multisig size. The §3 self-contradiction (old "fixed by array-id" parenthetical)
  is deleted; §3 and §4.2 now agree. `role(2)` holds the 3 needed values; no "solo" is required (has-raid=0 ⇒ solo).
- All round-2 Minors (M-1 … M-4, N-5) are closed (details below).

The math foundation re-verified clean and **unchanged** (field `0x805`/α order 2047, CRC-5 `x⁵+x²+1`
primitive + slot-independent ≤2⁻⁵, RS eval-form append-only, RAID MDS). Deps accurate at HEAD.

**The single remaining block is a frozen bit-budget defect the round-2 M-3 fold pinned into normative
text** but which neither round-2 nor round-1 flagged: the **GEOM `stride b` field is 4 bits (max 15),
but `b = round(√K)` exceeds 15 for `K ≥ 241` (payload ≳ 325 B)** — a regime the format explicitly
admits (`payload_len` is 16 bits / ≤65535 B; the spec itself names `n ≤ ~300` and "200+ B" md1
wallet-policies). This is the **same class** of frozen-constant width contradiction the gate caught at
NEW-I2 (3-bit index) and I1 (×16 ledger): a stored field too narrow to hold the value it must store
for in-range inputs. It is **Important, not Critical** (no silent fund loss — the encoder can refuse;
no decode miscorrection), but it contradicts the plan's own normative payload range and so blocks the
gate. See **NEW-I3**. The fix is clean: `b` is **fully derivable** (`b = round(√K)`, K closed-form
from `payload_len`+`t`), so the field can simply be dropped (and the freed bits returned to GEOM
reserved) — or widened to 6 bits if a stored stride is preferred.

The trend remains strongly convergent (R1 2C/3I → R2 0C/2I → R3 0C/1I). One localized
header-layout fold should land GREEN.

### Math I re-verified myself this round (NOT rubber-stamped)

| Claim (plan §) | Result |
|---|---|
| `p(x)=x¹¹+x²+1` (`0x805`), `α=x` (`0x002`), `ord(α)=2047` (brute-forced); `α²³≠1` (=34), `α⁸⁹≠1` (=322) | ✅ TRUE (2047=23·89, only proper divisors) |
| CRC-5 `x⁵+x²+1` (`0x25`) primitive: `ord(x)=31=2⁵−1` (computed) | ✅ TRUE |
| CRC-5 single 11-bit-word substitution miss = exactly `0.0308 ≤ 2⁻⁵`, **identical at every slot** (b=7 and b=8 swept, all slots, all 2047 deltas) | ✅ TRUE — uniform, no blind spot |
| **H1 bit budget:** `5+2+5+10 = 22` (2 words); `index-in-array(5)` covers `0..31` (full range for n≤32); `role(2)` ≥ 3 values | ✅ TRUE |
| **r=2 MDS:** `α^i` pairwise distinct for `i=0..31` since `ord(α)=2047 > 32` ⇒ `[n+2,n]` MDS for all n≤32 | ✅ TRUE |
| **GEOM bit budget:** A+B `payload_len(16)+t(6)=22`; C `b(4)+U(3)+reserved(4)=11`; D `CRC(11)` ⇒ GEOM = 4 words | ✅ adds up |
| **|header| not circular:** ledger offset `|header|−2U` == GEOM-end for all (has_raid,U) ∈ {(0,3),(1,3),(0,1),(1,1)} → {(11,5)=(5),(15,9)=(9),(7,5)=(5),(11,9)=(9)} | ✅ TRUE — `|header|` depends only on `has_raid`+`U` (counts), not ledger contents |
| **End-to-end trace** (solo mk1, payload 73B, t=44, U=3): `K=58`, `b=8`, `checkpoints=8`, `|header|=11`, `payload_offset=11`, `K′=77`; with m=8 recorded, `m_present = 87−77−2 = 8` ✓ | ✅ every quantity recoverable from words-present + GEOM alone |
| **`t(6)`:** 0..63 holds default 44 and min 33; **`U(3)`:** 0..7; **`payload_len(16)`:** ≤65535 | ✅ (but see NEW-I3 for `b(4)`) |
| **`b(4)` capacity:** `b=round(√K)>15` for `K≥241` (payload ≳ 325 B), reachable within `payload_len≤65535` / RS cap n≤2047 (b up to ~44) | ❌ **field too narrow — NEW-I3** |
| Deps: `sha2="0.10"` (Cargo.toml:47), `bip39 v2 all-languages` (:49), members=`["crates/mnemonic-toolkit"]` (:2) | ✅ accurate at HEAD `352b1adf` |

---

## Closure of round-2 findings

### NEW-I1 — ledger closed-form / read-ambiguity — **CLOSED (verified)**

The fold (§4.2 L190–199) replaces the variable marker-delimited run with **`U` FIXED reserved
2-word slots at the known offset `|header|−2U`**, `U` carried in the CRC'd GEOM word C. I probed every
sub-question the round-3 brief raised:

- **Is `|header|` self-consistent (it lists `2U` as part of `|header|` while the ledger offset is
  `|header|−2U`)?** ✅ **Not circular.** `|header| = 1(H0) + (4 if has-raid) + 4(GEOM) + 2U(ledger)`
  is a closed integer the moment `has-raid` (from H0, word 0) and `U` (from GEOM) are known — it
  depends only on those two *counts*, never on ledger *contents*. The ledger offset `|header|−2U`
  then equals exactly where GEOM ends; I verified `|header|−2U == GEOM-end` for `(has_raid,U) ∈
  {(0,3)→5, (1,3)→9, (0,1)→5, (1,1)→9}`. Self-consistent.
- **Can the decoder tell a blank all-zero slot from a filled one, and does an all-zero slot collide
  with a valid `marker=0b1110` read?** ✅ **No collision.** Slots are read **positionally** (the
  decoder knows there are exactly `U` of them at `|header|−2U`), so the marker is no longer a
  *delimiter* — it is only an in-slot validity check. A blank slot is the all-zero pattern, whose top
  4 bits are `0b0000 ≠ 0b1110`, so it is unambiguously "empty"; a filled slot carries `0b1110`. The
  decoder never marker-scans payload to find the boundary, so the ~6% payload-word-0 mislocate that
  defined NEW-I1 cannot occur.
- **What happens at the `U+1`-th upgrade (all slots used)?** ✅ **Graceful.** "Authoritative recorded
  length = **max** over filled slots AND stop-signs" (L198–199), and the §4.4 truncation test uses
  `max(ledger entries, highest stop-sign count)`. So once all `U` ledger slots are exhausted, a further
  upgrade's new count is still carried by the back stop-sign (also an exact 11-bit count), and the
  truncation flag still fires correctly. `U` is a soft ceiling on the *front-anchored* durability, not
  a hard cap on upgrades. *(Optional nit, not blocking: a one-clause note "upgrades past `U` slots rely
  on the back stop-sign for the newest count" would make this explicit; the semantics are already
  correct via the `max(...)`.)*

The load-bearing consequence — `K′`/`m_present`/`payload_offset` being post-RS-independent — is now
real: `m_present = words_present − K′ − |stop-sign|` (L187) dropped the `−|ledger|` term, and
`payload_offset = |header|` is a closed integer. **CLOSED.**

### NEW-I2 — H1 index width / r=2 MDS — **CLOSED (verified)**

§4.2 L172–176: `H1 = 2 words = 22 bits = n−1(5) │ role(2) │ index-in-array(5) │ reserved(10)`. I
verified:

- **`5+2+5+10 = 22`** ✓ (exactly 2 words; the prior 1-word/3-bit index that capped the exponent at 8
  is gone).
- **The `P₂` exponent = full 5-bit `index-in-array`** (§3 L128–129, L140–141), distinct over `0..n−1`.
  For r=2 MDS recovering any 2 of n+2, the column generators must be pairwise distinct; with
  `α^i` for `i=0..n−1` and `ord(α)=2047 > n_max=32`, they are. **r=2 MDS holds for all n≤32** — I
  confirmed `α^i` distinct over `i=0..31`. The headline "survive any 2 of n+2 plates" now holds for
  15-of-15 multisigs.
- **`role(2: 0=data,1=parityA,2=parityB)` is enough** — 2 bits hold 3 values; "solo" is unneeded
  because `has-raid=0` already implies solo (H1/array-id absent entirely). ✓

The §3 contradiction round-2 flagged (Minor-1: old "i = stripe index, fixed by array-id" vs the
correct H1-index binding) is **resolved** — §3 L128–129 and L140–141 both state "NOT array-id," and no
contradicting parenthetical remains. **CLOSED.**

### Round-2 Minors

- **M-1** (§3 `P₂`-exponent consistency): **CLOSED** — see NEW-I2; §3 is now internally consistent.
- **M-2** (CRC cannot prune a value-free deletion): **CLOSED** — §4.3 L213–216 now states the CRC-5
  "**cannot prune** a value-free deletion, kernel `2⁶`," routes deletion recovery through the GLOBAL
  RS+tag, gives the `(b−1)·2⁻ᵗ ≈ 2⁻⁴⁰` union bound (L220), and adds the **per-block-linear** path for
  multi-block single-deletions ("Single deletions in DIFFERENT blocks are independent … per-block
  search (linear, not a cross-block product)," L221–223). The round-2 Minor-2 over-claim is gone.
- **M-3** (concrete `|header|` formula): **CLOSED arithmetically** — `|header| = 1 + (4 if has-raid)
  + 4 + 2U` (L185) is used consistently in `payload_offset`, `K′`, and the M1 trace; I verified it
  reconciles end-to-end. *(The very same fold pinned the `stride b(4)` width that is too narrow — see
  NEW-I3. The `|header|` arithmetic itself is correct.)*
- **M-4** (ledger checksum coverage + `payload_len` width): **CLOSED** — L192 now states the ledger
  slot checksum is "7 over the slot's marker+count" (coverage specified), and L180 reads
  `payload_len(16: ≤65535 B)` (the round-2 "2¹⁶ vs 22-bit" mismatch is fixed — it is now an honest
  16-bit field with the matching ≤65535 B cap).
- **N-5** (SHA refresh): plan L7 cites toolkit `a552a242`; HEAD is now `352b1adf` (the round-2 fold
  commit is the only commit since, docs-only). One stale by exactly the fold commit again — same
  benign decay pattern as round-2; refresh to `352b1adf`. Tracked as Nit-1 below. Spec commit
  `31109f8e` (L5) verified correct.

---

## Final consistency pass

### End-to-end |header| / K′ trace (solo mk1 xpub — the brief's worked example)

Payload 73 B, `t=44`, `U=3`, `has-raid=0`. A cold decoder, given only the words-present and the
positional GEOM, recovers every quantity:

```
K           = ceil((8·73 + 44)/11) = ceil(628/11) = ceil(57.09) = 58      ✓
b           = round(√58) = round(7.616) = 8                                ✓
checkpoints = ceil(58/8) = 8        (K≥16, so not the degenerate-1 floor)  ✓
|header|    = 1(H0) + 0(no H1/array-id, has-raid=0) + 4(GEOM) + 2·3(ledger) = 11   ✓
payload_offset = |header| = 11                                            ✓
K′          = |header| + K + checkpoints = 11 + 58 + 8 = 77               ✓
   (with m=8 parity recorded + 2-word stop-sign: words_present = 77+8+2 = 87)
m_present   = words_present − K′ − |stop-sign| = 87 − 77 − 2 = 8          ✓ (recovers m)
```

Every quantity is derivable from `payload_len`+`t`+`b`+`U` (all in CRC'd GEOM) + `has-raid` (H0) +
`words_present`. **No circular dependency, no off-by-one, no post-RS dependency.** The trace is
internally consistent. (The brief's hypothesized `|header|`=? resolves to **11** for this case.)

### header-CRC input is well-defined before knowing has-raid? — ✅

`has-raid` lives in **H0 (word 0, fixed position)**, read first. Once H0 is read, the decoder knows
whether H1(2w) + array-id(2w) are present, so the positional CRC input set `H0 │ H1? │ array-id? │
GEOM A–C` (L182) is fully determined **before** the CRC is computed. The ordering works; the CRC
covers a deterministic word set (this is also what NEW-I1 needed — and now has, since `U` and
`has-raid` fully fix the header length). No ambiguity.

### Tiny md1 template (K≈6–19, U=1) — does the ledger dominate? — acceptable

For a tiny template with `U=1`, `|header| = 1+0+4+2 = 7`. Worst case I computed (payload 8 B, K=10,
checkpoints=1): `K′=18`, header = 7/18 ≈ 39% of the mandatory prefix — but the **ledger portion is
only 2 words (≈11%)**; the header overhead is dominated by GEOM(4)+H0(1), not the ledger. By payload
20 B the ledger is ≈6%. `U=1` is specified for "never-upgrade / tiny templates" (L199) and is
consistent with the GEOM `U(3)` field. The ledger does **not** dominate; this is acceptable and
correctly specified. *(No finding.)*

### Other cross-checks

- `role(2)` is used consistently (no leftover `role(3)`).
- The three word-class markers — checkpoint `0b101` (3b), stop-sign `0b1111` (4b), ledger `0b1110`
  (4b) — are all distinct (§3 L135–136); since the ledger and stop-sign are read positionally /
  by-highest-count and not by free marker-scan over payload, the class separation is sound.
- Non-linear integrity tag (spec C1/NEW-C1) preserved (§3 L132–133; linear in-codeword tag forbidden).
- Post-correction SHA-256 equality (§5 step 5) preserved + reused as the C1 alignment oracle.
- RAID lone-parity privacy KAT preserved (§4.6/§8; P5 L329).
- Lockstep completeness unchanged and complete: schema_mirror, manual `40-cli-reference`, ToolkitError
  alphabetical, binary-identical docs, CHANGELOG/READMEs/fuzz-lock/install.sh/man-pages, `wc-codec`
  fuzz target (P6 L334), `recover --json` paired-PR coordination (L334), post-impl whole-diff review
  (L337). No regressions.

The only number that does **not** reconcile from three rounds of in-place edits is the `stride b(4)`
field width vs `b = round(√K)` for large K — **NEW-I3**.

---

## New Important

### NEW-I3 — GEOM `stride b` is a 4-bit field (max 15) but `b = round(√K)` exceeds 15 for `K ≥ 241` (payload ≳ 325 B), a regime the format explicitly admits — large `md1` descriptors cannot be encoded

**Citation:** plan §4.2 L180 — GEOM word C = `stride b(4) │ U(3) │ reserved(4)`. §4.3 L202 —
`b = round(√K)`. §4.2 L180 — `payload_len(16: ≤65535 B)`. Spec §6.2 L275 — "no interleaving needed at
**`n ≤ ~300`**"; spec §5 L202 — `md1` 2-of-3 wallet-policy "**~200+ B → ~150**" data words.

**Defect.** `b` is stored in a **4-bit field (0..15)**, but `b = round(√K)`:

| K (data words) | payload ≈ | `b = round(√K)` | fits 4-bit? |
|---|---|---|---|
| 58 (mk1 xpub) | 73 B | 8 | ✅ |
| 150 (2-of-3 md1) | ~200 B | 12 | ✅ |
| **241** | **~325 B** | **16** | ❌ overflow |
| 368 | ~500 B | 19 | ❌ |
| 732 | ~1000 B | 27 | ❌ |
| 1095 | ~1500 B | 33 | ❌ |
| ~1950 (RS cap) | — | 44 | ❌ |

`b` overflows the 4-bit field at `K ≥ 241` (i.e. `√K ≥ 15.5`, payload ≳ 325 B). This regime is **not
hypothetical**: (i) the `payload_len` field is **16 bits / ≤65535 B** (L180) — the plan's own declared
range; (ii) the spec explicitly names single codewords up to **`n ≤ ~300`** (→ K≈280, b=17) as the
no-interleaving operating point (spec L275); (iii) realistic large md1 wallet-policies (e.g. a 5-of-9
or 15-of-15 with many embedded 65 B xpubs, plus general-policy AST/TLV framing) land at 500–1500 B,
giving b ≈ 19–33. For any such card the encoder cannot write a correct `b` into the 4-bit field; it
would either silently truncate the stride (→ the decoder reconstructs the wrong checkpoint grid →
desync / refuse on a pristine card) or must refuse to encode an otherwise in-range descriptor.

This is the **same class** of frozen-bit-budget contradiction the gate has already caught twice — I1
(×16 ledger granularity capped below 2047) and NEW-I2 (3-bit `index-in-array` capping the exponent at
8). It was latent in round-1/round-2 (the worked examples only reached K=160 / b=13, under the cliff),
but the **round-2 M-3 fold pinned the exact `stride b(4)` width into normative GEOM text** (L180), so
it is now a frozen constant that is provably too narrow for in-range inputs.

**Why Important not Critical:** no silent fund loss and no decode miscorrection — the failure modes are
(a) encoder refuses a large descriptor, or (b) if it truncated, the resulting card fails its own
header-CRC / RS / integrity-tag checks loudly. But it **breaks the format for large `md1`
wallet-policies**, contradicting the plan's stated `payload_len ≤ 65535 B` and the spec's `n ≤ ~300`
operating range — a real availability/correctness defect in a 20-year-frozen format.

**Recommended fold (pick one, then re-grep §4.2/§4.3 for consistency):**

1. **Drop the field, derive `b`.** `b = round(√K)` is a **deterministic pure function of `K`**, and
   `K = ceil((8·payload_len + t)/11)` is already closed-form from CRC'd GEOM. So the stored `stride b`
   is **redundant** — the decoder can recompute it. Remove `b(4)` from GEOM word C, return the 4 bits
   to `reserved`, and have both encoder and decoder compute `b = round(√K)` from the frozen formula.
   *(Cleanest — eliminates the field rather than widening it, and removes an encode/decode consistency
   surface. Recommended.)* Freeze the exact rounding rule (`round`, ties — `round(√K)` with banker's vs
   half-up) as a frozen constant so encoder/decoder never disagree.
2. **Widen the field to hold `b` up to ~44.** A stored stride needs **6 bits** (0..63 ≥ 44). Word C
   becomes `stride b(6) │ U(3) │ reserved(2)`. State `b_max` and confirm it covers the RS cap
   (`K′ ≤ 2047 ⇒ b ≤ round(√~1950) ≈ 44 < 64`). *(Keeps a stored stride if there's a reason to prefer
   it over derivation; spends 2 more reserved bits.)*
3. **Honestly cap `payload_len`** so `b ≤ 15` always (`K ≤ 240 ⇒ payload ≤ ~324 B`), narrow
   `payload_len` accordingly, and document that descriptors above the cap are out of scope. *(Caps a
   real use case — large multisig md1 — so only acceptable if such descriptors are explicitly declared
   out of scope; otherwise inferior to (1)/(2).)*

Whichever path, add a **P3/P4 KAT at the boundary**: a payload with `K ≥ 241` (`b ≥ 16`) round-trips —
either by deriving `b` (option 1), or with `b` stored in the widened field (option 2), or refused with
a clear "payload exceeds Word-Card size limit" error (option 3). And re-grep L180 (`stride b(4)`) vs
L202 (`b = round(√K)`) so the stored width and the formula cannot contradict.

---

## Minor / Nit

### Nit-1 — stale source SHA in plan header
**Citation:** plan L7 cites toolkit `a552a242`; HEAD is `352b1adf` (the round-2 fold commit, docs-only,
is the only commit since). Deps re-verified accurate at HEAD, so no decay damage. **Fold:** refresh to
`352b1adf`. Spec commit `31109f8e` (L5) and the sibling SHAs are unchanged/correct.

*(Optional, non-blocking: a one-clause note in §4.2 that upgrades past the `U` ledger slots rely on the
back stop-sign for the newest count would make the NEW-I1 `U+1`-th-upgrade path explicit; the semantics
are already correct via `max(ledger, stop-sign)`.)*

---

## Faithfulness & new-risk audit

- **Non-linear tag (spec C1/NEW-C1):** ✅ preserved (§3 L132–133).
- **Bounded-desync / whole-block-erasure fallback (spec C2):** ✅ preserved + correctly realized
  (global-tag alignment oracle, erase/refuse fallback, per-block-linear multi-deletion path).
- **Recorded-length ledger (spec C3/I-A):** ✅ now fully faithful — granularity reaches 2047 (I1) AND
  the cold-decode geometry is unambiguous (NEW-I1 closed via the fixed-`U` positional ledger). The
  round-2 faithfulness gap is closed.
- **No-silent-miscorrect (spec C1):** ✅ post-correction SHA-256 equality (§5 step 5) preserved.
- **RAID privacy (spec §7.3):** ✅ preserved (lone-parity KAT).
- **RAID r=2 MDS (spec §7.1):** ✅ **now holds for all n≤32** (NEW-I2 closed) — the round-2 "broken for
  n>8" gap is gone.
- **Field/RS/RAID algebra + CRC-5:** ✅ byte-identical to the prior rounds' machine-verified §3; the
  folds did not perturb it.
- **NEW risk introduced by the round-2 folds:** the M-3 fold pinned the `stride b(4)` GEOM width
  (NEW-I3) — too narrow for `K ≥ 241`. This is the only new defect; it is in §4.2 (the exact text the
  fold rewrote). No new risk in the math layers.

## Lockstep completeness

Unchanged and complete (see Final consistency pass). No regressions.

---

## What must turn GREEN before code

1. **NEW-I3** — resolve the GEOM `stride b(4)` vs `b = round(√K)` overflow for `K ≥ 241`: **drop the
   field and derive `b`** (recommended), or widen to 6 bits, or honestly cap `payload_len`. Re-grep
   L180/L202 for consistency; add the boundary KAT (`K ≥ 241`, `b ≥ 16`).
2. **Nit-1** — refresh the plan's source SHA to `352b1adf`; optionally add the one-clause `U+1`-th
   upgrade note.

Both round-2 Importants (NEW-I1 ledger closed-form, NEW-I2 H1 index width / r=2 MDS) and all four
round-2 Minors are **genuinely closed and machine-re-verified**. The math foundation (field / RS /
RAID / CRC-5 / global-tag C1 mechanism / fixed-`U` positional ledger) is **solid and verified**. The
remaining block is one localized header bit-budget fold in §4.2 — the same wire-layout area the prior
folds touched. Fold → persist this review → re-dispatch for round-4. The loop is converging
(2C/3I → 0C/2I → **0C/1I**); a clean fold should land GREEN.
