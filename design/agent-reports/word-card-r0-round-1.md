# R0 architect review — Word-Card encoding brainstorm spec (round 1)

- **Reviewer:** opus architect (mandatory pre-implementation R0 gate)
- **Spec under review:** `design/BRAINSTORM_word_card_encoding_2026-06-24.md`
- **Date:** 2026-06-24
- **Spec source SHAs verified:** mk-codec @ `46631c6`, md-codec @ `7764145d`, ms-codec @ `5c0335c`, toolkit @ `60af98dd`
- **Scope:** funds-safety/custody-safety first-class. Adversarial. This gate blocks all implementation.

---

## Verdict

**RED — 3 Critical / 4 Important.**

The wire-format facts are accurate (all 3 codecs verified TRUE against source — see §A below).
The core coding-theory primitives (evaluation-form systematic RS prefix-extensibility; RAID r=1/r=2
MDS recovery; the parity-privacy claim) are **fundamentally sound**. The design is promising.

But three things must close before a plan-doc:

1. **C1 — the "never silently miscorrects" custody guarantee (§9) is FALSE as written.** The §8-step-5
   round-trip is *self-referential* and cannot catch an RS miscorrection onto a structurally-valid
   wrong xpub. For a funds tool this is the load-bearing custody claim and it does not hold.
2. **C2 — the indel→erasure reduction (the entire cheap-located-runs story) depends on intra-block
   *pinpointing* that §12 open-question 2 admits is unresolved.** The headline guarantee
   (§6.2 "located runs cost 1/word", §9 worked budgets) is built on a capability the spec itself
   lists as OPEN. Worse, a compound *checkpoint-deletion + same-block data-deletion* defeats
   localization entirely and desyncs the whole RS codeword (a single un-localized deletion is up to
   `n` symbol errors, NOT a bounded-weight error).
3. **C3 — the soft-terminal stop-sign cannot fit in one word and the §9 anti-truncation claim is
   wrong in the append-only case** (silent tier-downgrade onto a stale-but-valid shorter codeword).

None of these are fatal to the *concept*; all are fixable. But each is an open Critical, so the gate
is RED and implementation MUST NOT begin.

---

## A. Wire-format fact verification (§1.1, §5, §7.4) — all TRUE, two clarifications

Verified against source (parallel sub-agent reads, citations confirmed):

| Spec claim | Verdict | Source |
|---|---|---|
| mk1 compact xpub = 73 B | **TRUE** | `mk-codec/src/consts.rs:53` `XPUB_COMPACT_BYTES=73`; `consts.rs:113-115` asserts `73 = 4+4+32+33` |
| incompressible = chaincode 32 + pubkey 33 = 65 B | **TRUE** | `mk-codec/src/bytecode/xpub_compact.rs:32-40` struct `chain_code:[u8;32]`, `public_key:[u8;33]` |
| drops depth/child-number, reconstructs from origin path | **TRUE** | `xpub_compact.rs:4-6`, `:71-108` `reconstruct_xpub()` (depth = path-len, child = last component) |
| md1 payload layout @ `encode.rs:65-92` | **TRUE** | `md-codec/src/encode.rs:65-92` `encode_payload` = header∥path∥use-site∥tree∥TLV |
| wallet-policy xpubs TLV tag `0x02`, 65 B each | **TRUE (w/ clarification)** | `md-codec/src/tlv.rs:16` `TLV_PUBKEYS=0x02`; `:29-32` value = 32 chaincode ∥ 33 pubkey = 65 |
| ms1 = `0x00` prefix + entropy 16/20/24/28/32 B, `consts.rs:29` | **TRUE** | `ms-codec/src/consts.rs:29` `VALID_ENTR_LENGTHS=&[16,20,24,28,32]`; `envelope.rs:235` `RESERVED_PREFIX` |
| keyless-template md1 embeds no xpubs | **TRUE** | `md-codec/src/encode.rs:43-51` `is_wallet_policy()` (false when `Pubkeys` absent/empty) |

**Clarification 1 (informational, not a finding by itself but feeds I3):** the md1 "65 B each" is the
TLV *value* (32+33). On the wire each xpub is additionally framed with a 5-bit tag + a **varint
length** (`md-codec/src/tlv.rs:203-206`), so the *framed* per-xpub size is variable and >65 B. Layer A
operates on the decoded payload, so this does not break the design — but §7's column-striping needs
the *aligned* per-xpub stripe width, and origin-path framing is variable across cosigners (see I3).

**Clarification 2 (informational):** mk-codec also defines a `cross_chunk_hash` + 53-byte stream
chunking for multi-string mk1 cards (`mk-codec/src/string_layer/chunk.rs:50-94`). Word Card encodes
the *payload* (73 B compact xpub), not the chunked string, so this is correctly out of scope — but
the spec should state explicitly that it consumes the pre-chunking canonical payload, to avoid a
future implementer re-deriving from the chunked wire form.

**Conclusion:** zero wire-format errors. The spec's citations are honest and current.

---

## Critical

### C1 — "the decoder never silently miscorrects" (§9, §6.2, §8-step-5) is FALSE; the round-trip is self-referential

**Where:** §9 bullet "Custody-safe: the decoder never silently miscorrects"; §6.2 "refuses rather than
silently miscorrects"; §8 step 5 "re-encode through the codec and assert the round-trip matches the
declared `array-id`/payload".

**The hole.** A bounded-distance RS decoder with `d = m+1` corrects `t ≤ ⌊m/2⌋` errors. When the true
error weight `e` exceeds `⌊m/2⌋` but the received word lies within `⌊m/2⌋` of a **different** valid
codeword, the decoder *miscorrects to that wrong codeword and reports success*. The decoder cannot
self-detect this — "refuse beyond `⌊m/2⌋`" only fires on decoder *failure* (no codeword within `t`),
not on miscorrection. So §6.2's refusal does **not** prevent miscorrection.

The only remaining defense is §8 step 5's round-trip. But step 5 re-encodes the **decoded payload
bytes `B`** and compares — and `decode(encode(B)) = B` is an **identity** for any structurally-valid
`B`. It catches an RS result that decodes to structural garbage; it does **not** catch an RS result
that decodes to a *structurally-valid-but-wrong* xpub. And "assert matches the declared `array-id`":
if `array-id` is a header field *inside the same RS codeword* (the natural reading of §5.2 / §7.2,
where `array-id` is in Layer A), a full-codeword miscorrection moves the `array-id` field too, so
recomputing it from the wrong `B` self-consistently matches. **The check is circular.**

**Quantified residual.** A miscorrected codeword passes step 5 whenever its 65-B payload is a
structurally-valid compact xpub: ~`(2/256)` for the `0x02/0x03` prefix byte × ~`0.5` on-curve x-coord
≈ **~0.4%** of random wrong codewords, before even considering origin-framing parse. That is **not**
negligible for a steel-backup custody tool. §9's "never" is unjustified.

**Recommended fold (must close before plan):**
- State explicitly that no-silent-miscorrection requires an **integrity tag carried OUTSIDE the RS
  codeword** — one that a wrong-but-valid codeword fails with overwhelming probability and that is
  verified against something *not reconstructed from the same codeword*. Concretely, mandate **at
  least one** of:
  (a) cross-check the recovered xpub's **master fingerprint** against the value **engraved in the
      plate header text / the human-readable plate title** (external to the codeword) — a wrong xpub
      matches a 4-B fingerprint with prob `2⁻³²`;
  (b) carry the **source `m*1` string's own BCH residue / a ≥32-bit SHA digest of the true payload**
      in a position that is verified but is *not itself an RS-recoverable symbol of the same
      codeword* (e.g. a separately-engraved short checksum the user also records, or derive it so a
      miscorrection cannot self-satisfy it).
- Re-word §9 to "the decoder never silently miscorrects **given the external integrity cross-check
  passes**; absent it, residual miscorrection probability is bounded by `P(wrong codeword is a valid
  xpub with matching external tag)`," and state the numeric bound.
- §8 step 5 must name *where* `array-id`/the integrity tag lives (inside vs outside the codeword) and
  why the chosen check is not self-referential.

This is the single most important finding. A self-custody tool's headline custody guarantee currently
does not hold.

---

### C2 — the indel→erasure reduction (and thus every §9 budget number) depends on intra-block pinpointing that §12-Q2 lists as UNRESOLVED; compound checkpoint+data deletion defeats it entirely

**Where:** §6.1 ("A detected+localized deletion is reduced to a **known erasure** … re-insert a blank
placeholder at the **pinpointed slot** → global synchronization restored"); §6.2 ("Located runs are
cheap … a located burst is the *easy* case", "every deletion is located; cost 1 each"); §9 worked
budgets all assume `erasure = 1 word`. Contradicted by §12 open-question 2: "whether it can *pinpoint*
a single intra-block deletion … **vs only flag the block**."

**The gap.** RS-with-erasures requires the erasure **positions** to be known. The sync trichotomy
(§6.1) localizes an indel to a **block of `b` words**, not to a position *within* the block. If the
decoder can only *flag the block*, then a single deletion forces it to treat the entire block as up to
`b` erasures (or `b` reinsert-and-test trials), so each indel costs up to `b`, **not 1**. That breaks
"located runs cost 1/word" (§6.2/§9) and invalidates the worked tables in §6.4 and §9.1 (which
advertise "repair 7 missing" / "20 missing" at face value). The spec's headline guarantee is built on
a capability it lists as OPEN.

**The deeper hole — unlocalizable deletion desyncs the whole codeword.** A single deletion that the
sync layer *fails to localize* is not a bounded-weight substitution: it shifts every subsequent symbol
by one position, i.e. up to `n` symbol errors. RS cannot correct that within any reasonable budget.
The whole design leans on §6.1 **always** localizing deletions to within the global erasure budget.
Probe the compound case the spec does not address:

> **Checkpoint `Cᵢ` deleted AND a data word in block `i` deleted (one steel-corrosion run hits
> both).** `Cᵢ` vanishing merges blocks `i` and `i+1`; the decoder sees ~`2b−2` words before the next
> readable checkpoint `Cᵢ₊₁` (running index +1, total span short by 2). It can bound *that two
> deletions occurred in the merged 2b-span* but cannot pinpoint **which two of ~2b slots**. The
> deletions become an **un-localized shift** over the merged span → RS desyncs → unrecoverable even
> though only 2 words were lost and `m` may be large.

The two-pass decode (§8 step 2→3→4) is **not circular** in the normal case (sync produces a
*hypothesized* full-length grid; RS corrects it; step 4 re-verifies) — that part is well-founded. But
it is well-founded **only when sync localizes every indel to a single slot**. The chicken-and-egg the
prompt flags is real precisely in the compound-deletion-of-a-checkpoint case: the checkpoint you need
to localize the nearby deletion is the one that's gone.

**Recommended fold (must close before plan):**
- Resolve §12-Q2 *in the spec*, not as an open question, because the §9 guarantees depend on it.
  Specify the intra-block pinpointing mechanism (e.g. per-block local parity strong enough to identify
  the deleted slot by reinsert-and-test, with the cost stated) **or** down-state every budget to the
  *block-granular* cost (`indel = up to b erasures`) and re-derive §6.4/§9.1 honestly.
- Add an explicit **guarantee statement for compound checkpoint+intra-block indel**: either prove the
  running-index + local-parity machinery still localizes both, or document the failure mode and the
  recovery path (e.g. "such a double-loss in one block exceeds per-string repair → falls through to
  Layer-D RAID / refuses"). Right now §6.1's "this closes the safety-marker-itself-is-wrong hole" is
  asserted but not demonstrated for the *deleted* (not just *miswritten*) checkpoint.
- State the **per-string repair guarantee as conditional on successful localization**, and define
  decoder behavior when localization is ambiguous (must be *refuse*, never *guess-and-decode*).

---

### C3 — soft-terminal: the stop-sign can't fit one word, and §6.3/§9's anti-truncation claim is wrong in the append-only case (silent tier-downgrade)

**Where:** §6.3 (stop-sign "carrying a total-word count + checksum"; "absent ⇒ truncated; flagged,
never mistaken for an intentional stop"; "decoder takes the **last** stop-sign as authoritative");
§9 "Append-only … up to ~1980 appendable parity words"; §12-Q6 (only asks *how to choose* the last
stop-sign).

**Sub-issue (a) — sizing.** A single word is 11 bits. A total-word count up to ~2047 already needs 11
bits, leaving **zero** bits for a marker class or checksum. So the stop-sign **cannot be one word** —
it must span ≥2–3 words, which §6.3 never states. §12-Q6 frames stop-sign encoding as "which
checksum," missing that the field simply does not fit. This must be sized before a plan, because the
mid-stream-forgery resistance (a single substitution must not be able to forge a marker *and* a
consistent count *and* checksum) depends on the stop-sign being wider than one word.

**Sub-issue (b) — the anti-truncation guarantee is false under append-only.** §6.3 claims a missing
trailing stop-sign is "flagged, never mistaken for an intentional stop." But the append-only ladder
(§6.4) means **earlier stop-signs are physically engraved and never erased**. If the user appended a
higher tier and then the last `k` words (incl. the newest stop-sign) are lost to tail corrosion, the
decoder finds the **previous tier's stop-sign**, whose count field is internally consistent for that
shorter prefix, and decodes a **valid shorter codeword with no truncation flag**. A stop-sign's count
proves the words *before* it; it cannot prove *no words came after it*.

**Custody assessment of (b):** the recovered shorter codeword still round-trips to the **correct
xpub** (it's a valid prefix), so this is **not a wrong-funds bug** — it is a **silent
protection-downgrade**: the user believes they have tier-2 redundancy but the decoder silently used
tier-1, and won't warn them. For a tool whose entire value proposition is *graduated, reported
redundancy* (§6.4 "reports achieved strength at read-back"), silently under-reporting strength after
partial loss is a real defect, and it **contradicts the spec's own "never mistaken for an intentional
stop" claim.**

**Recommended fold (must close before plan):**
- Size the stop-sign explicitly (≥2 words: marker ∥ count ∥ checksum) and state its forgery
  resistance against a single/double substitution.
- Add a **monotone tier-id / max-tier-ever field** to the header (Layer A, RS-protected) so the
  decoder knows the highest tier that *once existed* and can flag "newest tier appears truncated —
  achieved strength is X but Y was once recorded" instead of silently falling back. Reconcile §6.3's
  "never mistaken for an intentional stop" with the append-only reality.

---

## Important

### I1 — §9's "never silently miscorrects" and "MDS ceiling" are stated as unconditional but both have material caveats

Beyond C1: §9 bullet 2 says per-string repair "is the **MDS ceiling** — no code at this overhead does
better." True for the RS code *as an erasure/substitution code over correctly-positioned symbols*, but
the **system** also fights indels (synchronization errors), and MDS optimality says nothing about
synchronization. The honest framing: "RS is MDS-optimal for substitutions/erasures; indel resilience
comes from Layer B and is bounded separately by localization success (C2)." As written, §9 conflates
the two and over-claims. **Fold:** split the guarantee into (i) substitution/erasure (MDS, RS) and
(ii) indel (sync-localization-bounded), with the conditionality from C2.

### I2 — §7.4 conditional RAID suppression is a protection-downgrade footgun

**Where:** §7.4 "If a wallet-policy `md1` Word Card is part of the same bundle ⇒ **do not auto-emit
RAID**; the `md1` card is the cross-plate recovery."

A wallet-policy `md1` *contains* all `n` xpubs but is **a single plate** — it provides redundancy for a
lost `mk1` **only if the `md1` plate itself survives**. Suppressing RAID conflates "md1 *contains* the
data" with "md1 *provides redundancy*." Concretely: with RAID-A suppressed, losing the `md1` plate +
any one `mk1` plate is **unrecoverable**, whereas RAID-A alone survives any 1 loss and RAID-B any 2.
So the default suppression can **downgrade** the array's loss tolerance while the toolkit tells the
user "the md1 card is your cross-plate recovery" — a false sense of safety. Also (footgun 2): for a
`sortedmulti` policy the md1's xpub order differs from the array stripe order, and the "covers the
mk1s" claim silently assumes a verified 1:1 coverage that isn't specified.

**Fold:** make RAID suppression **opt-in, not default**, OR gate it on the md1 redundancy being at
least as strong as the RAID it replaces (i.e. require ≥2 independent md1 copies before suppressing
RAID-A), AND require the toolkit to *verify* the md1 actually embeds all `n` array xpubs (coverage
check) before claiming coverage. State the residual single-point-of-failure plainly.

### I3 — §7 RAID striping well-definedness is a *prerequisite* for the r=1/r=2 MDS math, but is filed as open-question Q5

**Where:** §7.1 striping; §5.1 "padded to a fixed array-wide width"; §12-Q5 "canonical fixed-width
per-xpub payload (padding rules) so column striping is well-defined."

The r=1/r=2 MDS recovery (§7.1) is *correct* **only if the columns are aligned** — every plate's stripe
`j` must hold the same semantic slot of each xpub's payload. The 73-B compact xpub is fixed-width
(good), but §5.1 stripes "compact xpub **+ its origin framing**," and origin/derivation-path framing is
**variable-length across cosigners** (different path depths). So the striped unit is *not* fixed width
without a padding convention, which §12-Q5 admits is unresolved. This isn't a future nicety — it's a
**precondition for the RAID linear algebra to even be defined**. **Fold:** elevate Q5 from
open-question to a must-resolve-before-plan item; specify the canonical fixed-width per-xpub stripe
(pad-to-array-max with an unambiguous, RS-protected length field) so the `[n+r,n]` system is
well-posed. (A brainstorm may legitimately defer *some* open questions, but not ones the headline
math depends on.)

### I4 — the §2/§7.1 "α^i" RAID weighting is under-specified; r=2 recovery requires a stated generator with `ord(α) ≥ n`

**Where:** §7.1 r=2 `P₂[j] = Σᵢ αⁱ·xᵢ[j]`; §6.2 "spec-frozen canonical sequence of points."

r=2 recovers any 2 erasures **iff** the `αⁱ` are distinct and nonzero for `i ∈ {1..n}` (the 2×2
Vandermonde minor `det = α^q − α^p ≠ 0` requires `α^p ≠ α^q`, i.e. `ord(α) ≥ n`), and the indexing
convention (does `i` start at 0 or 1? is `P₁`'s implicit weight `α⁰=1`?) must be pinned so it doesn't
collide with `P₁`'s all-ones row. Over GF(2¹¹) this is trivially satisfiable (`α` a generator, `n ≤ ~15
cosigners ≪ 2047`), so it is **not** a soundness risk — but the **spec must name `α`, the field's
reduction polynomial, the index base, and the canonical RS evaluation sequence** as frozen constants,
exactly as it (correctly) insists on for the word-tail in §6.2. Without that, two implementations (or a
re-implementation years later for recovery) will disagree and recovery will silently fail. **Fold:**
add a "frozen constants" subsection: field polynomial, `α`, eval-point sequence for the word tail,
α-weighting/index-base for RAID, and the BIP-39 index↔word map endianness. This is the single highest
"recover-in-20-years" risk after C1.

---

## Minor / Nit

- **N1 — §6.2 "Reed's original construction" is a misattribution.** Reed's 1960 original was the
  **coefficient** evaluation form (message symbols as polynomial *coefficients*), which is **neither
  systematic nor prefix-extensible**. The construction the spec actually needs and describes (data =
  evaluations at the first `K′` fixed points, parity = evaluations at further fixed points) is the
  **interpolation / extended-evaluation systematic RS**. The underlying claim — *any prefix `P₁…Pₘ` is
  a valid `[K′+m, K′]` MDS code, append-only* — is **correct**; only the name is wrong, and the
  "generator-polynomial form is NOT prefix-extensible / MUST NOT be used" parenthetical is **correct**.
  Naming WB/Gao decoders (§8 step 3) is consistent with the evaluation form. **Fold:** drop "Reed's
  original construction," call it "systematic Reed–Solomon in evaluation/interpolation form."

- **N2 — §7.3 privacy threshold wording is off-by-one for r=2.** "reveals nothing … until combined
  with `n−1` real plates" is exact for r=1 (XOR). For r=2 the last **two** xpubs are recoverable from
  both parity plates + `n−2` reals. Still safe (that's the recovery threshold = exactly who is supposed
  to recover), but the stated threshold understates r=2 exposure by one plate. **Fold:** "r=1: needs
  `n−1` reals; r=2: needs `n−2` reals + both parity," and note this equals the recovery threshold (no
  privacy loss).

- **N3 — §6.1 block-size arithmetic rounds inconsistently.** `K=54, b=7 ⇒ ⌈54/7⌉ = 8` checkpoints ⇒
  `K′ = 62`, but the spec uses `K′ ≈ 61` (and §9.1 "~1980" vs computed 1985–1986 appendable). All
  within rounding tolerance and harmless, but pick one and be consistent so §6.4's ladder ("words
  1–61") matches. Also §6.1 says `K=160 ⇒ b≈12` but `√160 = 12.65 ⇒ 13`; trivial.

- **N4 — §10 lockstep list is missing several version-sites that MEMORY flags as silent-drift traps.**
  Listed: schema_mirror, manual mirror, ToolkitError alphabetical, binary-identical docs. **Missing**
  (per project release-ritual memory): **`CHANGELOG.md`** (gated by `changelog-check` *on the tag* —
  "easy to omit," caught v0.70.1); **BOTH READMEs** version sites (silent drift); **fuzz/Cargo.lock +
  `cfg(fuzzing)` dual-home**; **`install.sh` self-pin / sibling pins** (load-bearing if §10/§12-Q3
  picks the *new sibling crate* option — a new crate adds a whole pin-staleness surface, cf. the open
  `install-sh-sibling-pin-staleness` followup); **man-pages** (`gen-man`: a new `word-card`/`recover`
  subcommand adds man pages across the affected CLIs, per the man-pages cycle); and the nuance that
  schema_mirror gates flag **names** not dropdown **values** (value additions are paired-PR
  discipline, not gated). **Fold:** expand §10 to the full version-site checklist, and explicitly note
  the new-sibling-crate option multiplies the lockstep surface (CLAUDE.md constellation list +
  install.sh pins + its own release ritual).

- **N5 — header `K-class` + variable `b` interact with very small `md1` (§12-Q7) in a way that should
  be a guarantee, not an open question, for funds-safety.** A keyless template with `K < 10` has
  `√K < 3.2`; sync degenerates (checkpoints nearly every other word) and a fixed parity floor is
  needed. Fine to keep the *tuning* open, but the spec should commit that **detection-always (§9
  bullet 1) holds for all `K ≥ 1`**, since that is the one guarantee that must never degrade. **Fold:**
  one sentence pinning the small-`K` floor and asserting detection-always survives it.

- **N6 — §5.1 should state it consumes the pre-chunking canonical payload** (Clarification 2 above), so
  a future implementer doesn't re-derive Layer A from mk1's chunked wire form (with its
  `cross_chunk_hash`).

---

## What is SOUND (so the next round doesn't re-litigate it)

- **Wire-format facts:** all TRUE (§A). No drift.
- **Evaluation-form systematic RS prefix-extensibility:** **correct** (modulo the N1 naming). Any
  prefix `P₁…Pₘ` *is* a valid `[K′+m, K′]` MDS code, distance `m+1`, and appending parity is genuinely
  append-only because each parity word is an independent evaluation at a new fixed point. Generator-poly
  form is correctly excluded. Two-pass decode (sync→RS→re-verify) is **well-founded in the
  single-localized-indel case** — the circularity only bites in the compound case (C2).
- **RAID r=1/r=2 MDS:** **correct.** `P₁=Σxᵢ` and `P₂=Σαⁱxᵢ` are the first two RS syndromes; the
  `[n+r,n]` code recovers any `r` erasures of the `n+r` plates (including parity plates), `P₁` is
  unchanged when `P₂` is appended, and there is **no** case where r=2 fails to recover 2 erasures
  given `ord(α) ≥ n` (I4). The "lose any r of n+r" honesty (parity counts toward budget) is right.
- **RAID privacy:** **correct** for both r=1 and r=2 — a lone parity plate is one (resp. two) linear
  combination(s) of `n` unknowns and leaks no individual xpub below the recovery threshold (modulo the
  N2 off-by-one wording). An adversary holding parity + reals only reaches an individual xpub at
  exactly the legitimate recovery threshold.
- **The "parity = correct + detect" lever** (`2·subs + erasures ≤ m`, `⌊m/2⌋` corrections): **correct**
  RS arithmetic. The *refuse-beyond-budget* posture is right; it just doesn't cover miscorrection
  *within* budget (C1).
- **Field/arithmetic:** GF(2¹¹)=2048, `n ≤ 2047`, K≈54 for 73 B, K′≈61, ~1986 appendable — all check
  out within rounding (N3).
- **`ms1` exclusion (§1.1):** correct and well-argued (re-encoding entropy to words is a net loss; the
  word view of `ms1` *is* the BIP-39 seed phrase).

---

## Gate decision

**RED. 3 Critical, 4 Important.** Per `CLAUDE.md`, no implementation, no plan-doc, until this converges
to 0C/0I. Fold C1–C3 + I1–I4 (N-items at author's discretion but N4 is cheap and high-value), persist
the revised spec, and **re-dispatch this R0 review** — the reviewer-loop continues after every fold,
because folds (especially the C1 external-integrity-tag design and the C2 pinpoint mechanism) can
introduce their own drift.

Priority order for the fold: **C1 (custody) > C2 (custody/correctness) > I4 (frozen constants /
recover-in-20-years) > C3 > I3 > I2 > I1 > N4**.
