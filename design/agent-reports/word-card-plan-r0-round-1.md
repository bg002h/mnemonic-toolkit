# Plan-doc R0 review — Engravable Word-Card encoding — ROUND 1

- **Artifact under review:** `design/IMPLEMENTATION_PLAN_word_card_encoding.md`
- **Authoritative spec (R0-GREEN):** `design/BRAINSTORM_word_card_encoding_2026-06-24.md`
- **Reviewer:** opus architect (mandatory pre-implementation plan-doc R0 gate; 0C/0I required)
- **Date:** 2026-06-24
- **Repo HEAD at review:** `d08b0d51` (plan cites `31109f8e`; see Minor-5)

---

## Verdict

**RED — 2 Critical / 3 Important / 5 Minor-Nit.**

The plan's *frozen field/RS/RAID math is independently verified CORRECT* — the primitive
polynomial, systematic evaluation-form RS, and RAID MDS construction all pass machine
verification (details below). The blocking problems are in the **concrete checkpoint
local-parity formula (§4.3)** — which, as written, neither *pinpoints* a deletion nor
reliably *detects* a substitution, contradicting two NORMATIVE GREEN-spec guarantees
(spec §6.1, §6.1 trichotomy table) — and in the **ledger granularity (§4.2)**, which cannot
represent the maximum legal codeword length. These are frozen-for-20-years constants, so a
defect here is high-severity.

### Math I verified myself (NOT rubber-stamped)

| Claim (plan §) | Result |
|---|---|
| `p(x)=x¹¹+x²+1` = `0x805` is the bit pattern `x¹¹,x²,1` | ✅ TRUE |
| `p(x)` **primitive** over GF(2); `α=x` has multiplicative order **exactly 2047** (brute-forced) | ✅ TRUE |
| KAT `α^2047=1`, `α^23≠1` (=0x22), `α^89≠1` (=0x142); `2047=23·89`, only proper divisors 23,89 | ✅ TRUE & sufficient |
| Systematic eval-form RS: data verbatim at `β₀..β_{K′−1}`, parity at further `βⱼ` → (a) systematic, (b) prefix-extensible/append-only, (c) recovers **any m erasures** (MDS, d=m+1), corrects ⌊m/2⌋ subs | ✅ ALL TRUE (200 random erasure trials, exact) |
| RAID `P₁=Σxᵢ`, `P₂=Σαⁱxᵢ` = `[n+r,n]` MDS recovering **any r of n+r**; no singular erasure-pair for n≤32 | ✅ TRUE (n=10,20,32 exhaustive pair scan + 300 random recover trials) |
| Bit budgets: H0/H1/Kdesc/ledger/checkpoint each =11 bits; stop-sign & array-id =22 bits (2 words) | ✅ all sum correctly |
| Ladder 62→69(+7)→82(+20); b=round√54=7, ⌈54/7⌉=8, K′=62 | ✅ reconciles with spec |
| Deps: `sha2="0.10"` (Cargo.toml:47), `bip39 v2 all-languages` (:49), members=`["crates/mnemonic-toolkit"]` (root :2) | ✅ accurate |

So **Frozen-constant correctness (review item 1) is GREEN** — the load-bearing field/code
algebra is sound. The Criticals are elsewhere.

---

## Critical

### C1 — §4.3 checkpoint local-parity does NOT pinpoint a deletion; it cannot, under the plan's own "reduce-to-one-known-erasure" mechanism

**Citation:** plan §4.3 lines 130–139 — `local-parity(5) = Σ_{k}(k+1)·word_k mod 32`,
"reinsert-test the `b≤15` candidate slots, accept the unique slot that revalidates";
**KAT (P3):** "every single intra-block deletion is uniquely pinpointed." Spec §6.1 line 239
makes pinpointing **NORMATIVE** ("reinsert-and-test: try the `b` reinsertion positions,
accept the one the local parity validates → reduce it to **one known erasure**").

**Defect.** The plan reduces a deletion to *one known erasure* and lets the RS layer fill the
*value*. That means the checkpoint's only job is to find the **slot** `d`. But "reinsert and
test" with the value left as an unknown erasure makes the parity equation
`S ≡ (c+1)·e + Σ_known mod 32` have a **free unknown `e` at every candidate slot `c`** — so a
consistent `e` exists for (almost) every slot. I measured this directly:

```
b= 4: uniquely-pinpointed-to-correct-slot   0.0% | multi-validate 100.0%
b= 7: uniquely-pinpointed-to-correct-slot   0.0% | multi-validate 100.0%
b=13: uniquely-pinpointed-to-correct-slot   0.0% | multi-validate 100.0%
b=15: uniquely-pinpointed-to-correct-slot   0.0% | multi-validate 100.0%
```

**The pinpoint KAT (P3) as written is unprovable** — it will be RED forever, because the
mechanism cannot uniquely identify the slot. The slot is fundamentally under-determined: a
single deletion in a length-`b` block, with the value unknown, leaves `b` candidate
(slot, value) pairs all consistent with one scalar parity check. A 5-bit local parity simply
does not carry enough information to localize *and* leave the value free.

**Why this is Critical, not Important:** (a) it is a **frozen 20-year constant**; (b) it
directly contradicts a NORMATIVE GREEN-spec guarantee (spec §6.1 "Pinpointing is NORMATIVE").
The spec's safety story does survive — the §6.1 *fallback* degrades an un-pinpointed indel to
a **whole-block erasure (cost ≤ b)**, which is custody-safe — but the plan **promised
per-slot pinpointing (cost 1) as the default and froze a formula that delivers it ~0% of the
time.** That is a silent capacity over-claim baked into a frozen constant.

**Recommended fold (pick one, then re-grep the spec to confirm faithfulness):**
1. **Honest-downgrade (smallest change):** drop the "uniquely pinpointed" promise for
   deletions; state that the checkpoint **detects** a per-block desync and the §6.1
   whole-block-erasure fallback (cost ≤ b) is the *normal* deletion path, with per-slot
   pinpointing only for **substitutions** (where the value is known, see C2). Rewrite the P3
   KAT to assert *block-localization* (the indel is confined to ≤ b erasures and never
   desyncs the codeword), which IS provable. This stays faithful to spec §6.1's fallback.
2. **Make pinpointing real:** carry a **per-position value-derived** parity that pins the slot
   even with the value erased — e.g. a checkpoint that stores `Σ_k g^k · word_k` over a field
   where the *positional weights are distinct and invertible*, AND additionally a
   running-checksum that lets reinsert-and-test discriminate slots by the **known survivors'
   re-indexing**, not by solving for the unknown. This needs a real construction + a proof
   that the b reinsertions give b *distinct* parities; it is more than a constant tweak and
   would re-open the §4.3 design. If chosen, exhibit the discriminating KAT before P3.

Either way the **frozen constant in §3/§4.3 must change or the promise must change** before
any code. (Note C2 is a *distinct* defect in the *same* formula.)

### C2 — the concrete `mod 32` integer weighting has badly non-uniform, and in places ~25%-blind, substitution detection — contradicting the trichotomy "detected" guarantee

**Citation:** plan §4.3 line 133 `local-parity(5) = Σ_{k}(k+1)·word_k mod 32`; spec §6.1
trichotomy table line 232 (`b` words since last checkpoint + **local parity fail ⇒
substitution**) treats a substitution as *detected*; spec §6.1 line 224 / N5 "Detection is on
for **any K ≥ 1**".

**Defect.** Because the weight `(k+1)` is applied as **integer multiplication mod 32**, slots
whose weight shares factors of 2 with 32 *discard low bits of the changed word*, so a
substitution at those slots is frequently invisible. Per-slot undetected-substitution rate
(b=8, measured 20 000 trials/slot):

```
slot 0 (weight 1, gcd 1): miss 3.1%     slot 4 (weight 5, gcd 1): miss 3.1%
slot 1 (weight 2, gcd 2): miss 6.2%     slot 5 (weight 6, gcd 2): miss 6.0%
slot 2 (weight 3, gcd 1): miss 2.9%     slot 6 (weight 7, gcd 1): miss 3.0%
slot 3 (weight 4, gcd 4): miss 12.8%    slot 7 (weight 8, gcd 8): miss 25.4%
```

Aggregate single-substitution detection is only **~93–95%**, and **slot 7 misses 25% of
substitutions.** The trichotomy table presents "substitution → parity fail" as deterministic;
the frozen formula makes it probabilistic and slot-dependent, with a one-in-four blind spot.

This interacts with C1: even the *detection floor* the spec guarantees "for any K ≥ 1" is
weakened by the concrete constant.

**Why Critical:** same frozen-constant + NORMATIVE-spec-contradiction logic as C1; a 25%
detection hole at a fixed slot is a real, exploitable-by-bad-luck miss on a custody backup.
(The RS layer and the post-correction SHA-256 tag are the ultimate safety net — a missed
checkpoint detection does not silently corrupt funds, it just spends RS budget or refuses —
so this is "weakens a guarantee," not "loses funds." But it IS a contradiction of a frozen
NORMATIVE claim, which the gate treats as blocking.)

**Recommended fold:** define the 5-bit local parity over **GF(2⁵)** (or as 5 independent
GF(2) linear checks / a CRC-5 over the block's 11-bit symbols) with **invertible, distinct
nonzero coefficients per slot** — i.e. an algebraic checksum, not integer-mod-32 — so every
single-symbol substitution flips the parity with the uniform `2⁻⁵ ≈ 3.1%` miss floor and no
slot is privileged. State the floor honestly (5 bits ⇒ ~1/32 per-block undetected-sub
residual; the global RS+tag catches the rest). Then C1's reinsert-test should be re-derived
over the SAME corrected primitive. Re-freeze the constant in §3 + §4.3 + spec §9.5.

---

## Important

### I1 — recorded-length ledger ×16 granularity cannot represent the maximum legal codeword length (2032 < 2047)

**Citation:** plan §4.2 lines 126–128 — ledger word `marker(4: 0b1110) │ cumulative-count(7 →
×16 granularity to reach 2047)`; "×16 granularity keeps a ledger entry to 1 word."

**Defect.** `7-bit count × 16 = 127 × 16 = 2032`, which is **< 2047**, the field/codeword
length cap (`n ≤ 2047`, spec §3 / plan §3 "Length cap n = K′+m ≤ 2047"). A maximally-appended
card (the explicit append-only design goal, spec §9 "~1985 appendable parity words … up to
the GF(2048) length cap") cannot have its true length recorded by the ledger — the truncation
test (plan §4.4, spec §6.3/§8 step 1) then under-counts and can MISS a real truncation of the
top ~15 words. For a custody truncation-safety primitive this is a silent-failure window at
exactly the highest-redundancy configuration users are encouraged toward.

**Recommended fold:** either (a) widen the ledger cumulative field (e.g. 2 words / a 2-word
ledger entry with full 11-bit count), or (b) keep ×16 but make the granularity ceil so the
*last* bucket covers `[2032, 2047]` AND have the truncation test treat "present < (ledger
bucket lower bound)" conservatively, AND/OR (c) rely on the **stop-sign's exact 11-bit count**
(plan §4.4) as the authoritative length and demote the ledger to a coarse *lower-bound* — but
then state explicitly that a truncation losing both the stop-sign AND landing in the top
2033–2047 window is only caught by the ledger bucket, and prove that window is covered.
Whichever path, the cap arithmetic must reconcile to **≥ 2047** and a KAT must exercise a
near-2047-length truncation.

### I2 — `canonical_payload_bytes()` round-trip determinism is asserted, not demonstrated; mk1 cross-chunk-hash / md1 TLV-ordering are exactly the places it can fail

**Citation:** plan §2 lines 50–59 (P0 cross-repo accessor + byte-identical round-trip
invariant); spec §5.4 / N6 (pre-chunking canonical payload).

**Gap.** The plan asserts the assembled bytecode is "a deterministic `Vec<u8>`" and the
round-trip `m*1 → payload → words → payload → m*1` is byte-identical, but does NOT establish
the two known hazards from the codec internals the spec itself flagged:
- **mk1** uses per-xpub chunking **plus a cross-chunk hash** (spec §5.1 / mk-codec
  `xpub_compact.rs`). If the canonical payload is "pre-chunking bytecode," re-encoding must
  re-derive identical chunk boundaries AND the cross-chunk hash from the payload alone. If any
  chunk-framing or hash input is *not* a pure function of the pre-chunking payload (e.g. it
  folds in a length or a checksum computed differently on re-encode), the round-trip breaks.
- **md1** is a TLV tree (spec §5.1 / md-codec `encode.rs:65-92`). TLV **ordering** and any
  canonicalization (sorted vs insertion order, optional-field presence) must be deterministic
  from the payload bytes; otherwise `re-encode(decode(words))` differs from the original.

Because P0 is the **first phase** and the toolkit's whole correctness rests on this
invariant, "additive, NO-BUMP-eligible accessor" is an *unproven assumption* about sibling
internals. If the accessor must canonicalize (re-sort TLV, recompute a hash), it may NOT be a
trivial getter and may NOT be NO-BUMP.

**Recommended fold:** make P0's KAT explicit and adversarial — assert
`encode(canonical_payload_bytes(decode(s))) == s` for a **vector set** spanning: mk1
multi-chunk xpub with origin framing; md1 wallet-policy (multiple `0x02` TLVs, to exercise
ordering); md1 keyless template. Add a stated **precondition** that the accessor returns the
*canonicalized* payload (and if canonicalization touches a hash/length, flag that the
accessor may be PATCH not NO-BUMP, and that the sibling FOLLOWUP must say so). If any codec
cannot guarantee `assemble∘disassemble = id`, that is a blocking dependency to resolve *before*
P1, not during P6.

### I3 — `K-descriptor` / header self-description is circular: the decoder needs `payload_len` (hence `K`) to run the 8→11 regroup and the RS geometry, but the plan derives `K` *from* `payload_len` which is itself only "recoverable from the regroup + tag width"

**Citation:** plan §4.2 line 124 — "full `K` derived as `payload_len` is recoverable from the
regroup + tag width (decoder cross-checks)"; §4.1 line 110 `K = ceil((8·payload_len + t)/11)`;
§4.4 stop-sign carries `cumulative-word-count`, NOT `K` or `payload_len`.

**Gap (chicken-and-egg, review item 3).** A *cold* decoder reading a possibly-truncated,
possibly-corrupted card must establish the RS geometry — where data ends, where parity begins,
where checkpoints sit, the value of `K′` — **before** it can run Gao decode. The plan's header
fields give `stride b` and a 3-bit `K-class`, but:
- 3-bit `K-class` = only **8 classes** for K ranging 1..~2047 — what is the class→K map, and
  does it pin `K` exactly or only a range? If a range, the decoder cannot place the
  data/parity boundary exactly, and "decoder cross-checks `payload_len`" has nothing
  authoritative to cross-check against.
- `payload_len` is never stored explicitly; it is "recoverable from the regroup" — but the
  regroup requires knowing where the payload ends, which requires `K`. Circular.
- The number of **parity words present** (`m`) is needed to set the RS budget; the plan does
  not name a header field that records it (the ledger/stop-sign record *total* word count, not
  the K′/m split).

**Recommended fold:** specify, field-by-field, the **exact closed-form** by which a cold
decoder recovers `(payload_len or K, t, K′, b, m_present)` from header words that are
themselves inside the front (RS-protected) region — with NO field that depends on a quantity
only knowable post-RS. Concretely: store `K` (or `payload_len`) explicitly in a header word
(11 bits covers 0..2047), make `K-class` purely an index into the frozen
parity-floor/stride table (not the K source-of-truth), and define `m_present = (total words
present) − (header+ledger+K′+stop-sign overhead)` with each overhead term computable from the
already-read header. Add a KAT: cold-decode a card given ONLY the word list (no side
metadata), for min-K, mid-K, max-K.

---

## Minor / Nit

### Minor-1 — spec/plan `K` reconciliation for mk1 (~54 vs computed 58)
Spec §5 table line 201 says mk1 xpub `K≈54`; plan §4.1 formula with 73 B + 44-bit tag yields
`K=58`; the plan's worked ladder (§4.3 / spec §6.4) is built on `K≈54, b=7, K′=62`. The
"+header" wording (spec line 201) hides the discrepancy. Not load-bearing (b=round√58=8 vs 7
shifts the ladder slightly) but the frozen worked example should state the EXACT payload byte
count it assumes and recompute K/b/K′ from the §4.1 formula so the KAT and the printed ladder
agree. **Fold:** pin one canonical mk1 payload length, recompute, and use it consistently in
§3/§4.1/§4.3 and the P-phase KATs.

### Minor-2 — mod-8 block-index aliasing under long bursts is defensible but unstated
3-bit `index(mod 8)` (§4.3) wraps every 8 checkpoints = `8·b` words. For small K (K=64, b=8 ⇒
64 words = the whole payload) an 8-checkpoint-destroying burst aliases the index; I verified
this only happens when essentially the entire payload is destroyed (already a refuse case), so
it is safe — but the plan should *state* the bound ("mod-8 realignment is safe because
aliasing requires destroying ≥ `8b` consecutive words, which for all K exceeds the
whole-payload-erasure refuse threshold") and add it to the realignment KAT (P3). Without the
stated bound a future reader can't tell 3 bits is enough.

### Minor-3 — RAID `array-id` collision target and `P₂` α-weight index source are under-pinned
§4.2 line 120: `array-id = top 22 bits of SHA-256(concat ordered cosigner fingerprints)`.
22-bit truncation ⇒ ~`2⁻²²` collision; with a handful of arrays per user this is fine, but the
plan should *state* the collision target (the spec §7.2 left it open). Separately, §3 line 86
and §4.6 say the `P₂` α-exponent `i` is "the stripe index, fixed by array-id" — the binding
"array-id → canonical stripe order → α-exponent per plate" must be deterministic and frozen
(the role/index in H1 §4.2 line 118 is the actual source). Confirm H1's `index-in-array`
(not array-id) supplies `i`, and freeze the rule. **Fold:** one sentence pinning the
α-exponent source + the array-id collision target.

### Minor-4 — phasing: RAID (P5) depends on the canonical fixed-width padding rule (§4.6, Q5) which is only finalized inside P6's adapter
P5 strips and reconstructs aligned stripes; the array-wide-max padding (§4.6 / §4.2 Q5) is a
property of the **canonical payload** produced by the P0/P6 adapter. P5 can use synthetic
fixed-width payloads for its KATs, but the plan should note that P5's "recover any r of n+r"
KAT must use payloads padded by the SAME rule P6 will use, else P5 GREEN doesn't imply the
integrated path works. **Fold:** state P5 uses the frozen §4.6 padding rule directly (move the
padding-rule freeze to P1/P2 constants, not P6), so P5 is self-contained.

### Nit-5 — stale source SHA in plan header
Plan line 7 cites toolkit source SHA `31109f8e`; repo HEAD at review is `d08b0d51`. Deps were
re-verified accurate at HEAD (so no decay damage), but per CLAUDE.md "document the source SHA …
re-grep against current origin/master," refresh the cited SHA (or note the delta is
docs-only). Also confirm the spec's commit `31109f8e` (plan line 5) is the actually-committed
GREEN spec — the working tree shows the spec file as committed; just re-verify the SHA matches
`git log` for that path.

### Nit-6 — `WcError`/`ToolkitError` alphabetical ordering is stated but not exhibited
Plan §6.1 line 191 "WcError variants **alphabetical**" and §6.2 line 201 `ToolkitError::WordCard`
"alphabetical placement … + Display/exit_code/kind arms." Good — matches CLAUDE.md. Nit: list
the actual variant names and confirm `WordCard` sorts correctly between its neighbors in the
*current* `error.rs` (the pre-v0.27.2 variants are NOT yet sorted per CLAUDE.md, so "insert
alphabetically" needs a target position). **Fold:** name the insertion point in `error.rs` so
P6 has a concrete target.

---

## Faithfulness & new-risk audit (review item 6)

- **Non-linear tag (spec C1/NEW-C1):** ✅ preserved — plan §3 line 88-90 keeps SHA-256
  truncated, forbids linear in-codeword tag. Good.
- **Bounded-desync (spec C2):** ⚠️ the *invariant* survives via the §6.1 whole-block-erasure
  fallback, BUT the plan's §4.3 over-promises per-slot pinpointing on top of it — see C1. The
  fallback itself is faithful; the *added* promise is not deliverable.
- **Recorded-length ledger (spec C3/I-A):** ⚠️ preserved in structure, but the ×16 granularity
  (I1) breaks it at max length — a faithfulness gap on the truncation guarantee.
- **No-silent-miscorrect (spec C1):** ✅ post-correction SHA-256 equality (§5 step 5) preserved.
- **RAID privacy (spec §7.3):** ✅ preserved — plan §4.6/§8 keep lone-parity-leaks-nothing;
  P5 has a "lone-parity-plate privacy KAT."
- **NEW design risk introduced by the plan:** the concrete §4.3 integer-mod-32 parity formula
  (C1+C2) and the ×16 ledger (I1) are NEW constant choices the spec did not contain; both are
  defective. No other new risk spotted.

## Lockstep completeness (review item 5)

Plan §7 P6 + §8 list: schema_mirror ✅, manual `40-cli-reference` ✅, ToolkitError alphabetical
✅, binary-identical docs (fixed seeds) ✅, CHANGELOG/both-READMEs/fuzz-lock/install.sh
sibling-pins/man-pages ✅, post-impl whole-diff review ✅ (§7 line 229). **Complete vs
CLAUDE.md.** Two additions to consider (not blocking): (a) **fuzz targets** — a new codec with
a sync/RS state machine is a strong fuzz candidate (decode-never-panics, decode∘encode=id);
the plan mentions `fuzz/Cargo.lock` as a version-site but does not add a *fuzz target* for
`wc-codec` — recommend filing it as a P5/P6 sub-item or explicit FOLLOWUP. (b) the GUI
`schema_mirror` is flag-NAME only — the new `--json` wire-shape of `word-card`/`recover` rides
the manual paired-PR rule (correctly noted in CLAUDE.md); the plan should name that the
`recover` extension's `--json` shape change needs the manual GUI coordination.

---

## What must turn GREEN before code

1. **C1** — drop/redo the §4.3 deletion-pinpoint promise; the frozen formula cannot localize a
   deletion-as-erasure. Either honest-downgrade to block-erasure-default (faithful to spec
   §6.1 fallback) or supply a real pinpointing construction with a discriminating KAT.
2. **C2** — replace integer-mod-32 weighting with an algebraic (GF(2⁵)/CRC-5) local parity so
   every single substitution misses at the uniform 2⁻⁵ floor; re-freeze the constant.
3. **I1** — fix the ledger granularity so it reaches ≥ 2047; add a near-max-length truncation KAT.
4. **I2** — turn the P0 round-trip invariant into an adversarial multi-vector KAT and resolve
   whether the codec accessor is truly NO-BUMP (canonicalization hazard).
5. **I3** — specify the exact cold-decoder header-field closed-form for `(K/payload_len, t, K′,
   b, m_present)` with no post-RS dependency; add a cold-decode KAT.

Fold → persist this review → re-dispatch. Math foundation (field/RS/RAID) is solid; the work
is in the concrete sync constants and the header/ledger self-description.
