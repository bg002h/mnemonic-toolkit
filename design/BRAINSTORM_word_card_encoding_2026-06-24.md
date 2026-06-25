# BRAINSTORM / SPEC — Engravable Word-Card encoding for `mk1` / `md1`

- **Status:** Brainstorm spec — **R0 round-3 folded (0C/1I addressed); round-4 re-dispatch pending.** NOT approved for implementation.
- **Date:** 2026-06-24 (R0 round-1 + round-2 + round-3 folds applied same day)
- **Author:** brainstorm session (single author; in the mandatory opus R0 loop)
- **Working name:** **Word Card (WC)** — provisional, rename welcome.
- **Source SHAs (wire-format facts cited below were read at these revisions):**
  - `mnemonic-toolkit` @ `60af98dd`
  - `descriptor-mnemonic` (md-codec) @ `7764145d`
  - `mnemonic-key` (mk-codec) @ `46631c6`
  - `mnemonic-secret` (ms-codec) @ `5c0335c`

> This document is the output of a design dialogue. Per `CLAUDE.md` it MUST pass an
> opus architect **R0 review to 0 Critical / 0 Important BEFORE any implementation**,
> and the reviewer-loop continues after every fold. No code until GREEN.

### R0 round-1 fold log (2026-06-24)

Round-1 verdict RED (3C/4I); full review at `design/agent-reports/word-card-r0-round-1.md`.
Wire-format citations all verified TRUE; core math (RS prefix-extensibility, RAID MDS,
privacy) confirmed sound. Folds applied:

- **C1** — removed the false "never silently miscorrects" claim + the self-referential
  re-encode check; added an **independent integrity tag outside the value-relation** (§5.3,
  §8 step 5, §9) with a numeric residual bound `≤ 2⁻ᵗ`.
- **C2** — promoted indel pinpointing to normative (§6.1); added the **bounded-desync
  invariant** + whole-block-erasure fallback (cost ≤ `b`); honest indel budget (§9).
- **C3** — stop-sign now **≥2 words** + a **monotone `declared-total-length`** header field
  so truncation/downgrade is always flagged (§5.2, §6.3).
- **I1** split §9 into value-layer (MDS) vs indel-layer (sync-bounded). **I2** flipped §7.4
  (RAID no longer auto-suppressed by an `md1`). **I3** elevated striping well-definedness to
  normative (§7.1). **I4** added the **Frozen Constants** section (§9.5).
- Nits: N1 RS attribution (§6.2), N2 privacy wording (§7.3), N3 `K′≈61–62`, N4 lockstep
  version-sites (§10), N6 pre-chunking payload (§5.4).

### R0 round-2 fold log (2026-06-24)

Round-2 verdict RED (2C/3I); full review at `design/agent-reports/word-card-r0-round-2.md`.
Round-1 folds C2/C3/I1/I2/I3/I4 + N1/N2/N4/N6 confirmed genuinely closed; citations and core
math re-verified TRUE. Round-2 folds:

- **NEW-C1** — integrity tag MUST be a **non-linear cryptographic hash**; a linear
  (BCH/CRC/XOR) tag inside the linear RS codeword is self-satisfied by a miscorrection and is
  **forbidden** in-codeword (§5.3, §9.5).
- **NEW-C2** — checkpoints gain a **self-identifying marker** + a normative
  **recognition/realignment** rule (bounded offset search validated by index-continuity;
  ambiguous ⇒ refuse-and-report) + a compound-case lemma (§6.1, §9.5).
- **I-A** — `declared-total-length` (double-meaning, false-flagged deliberate stops) replaced
  by a single-meaning **front-anchored append-only `recorded-length` ledger** (§5.2, §6.3, §8).
- **I-B/I-C** mirrored the two new primitives into §9.5. **Nit-1** stale `K′=61`→62 ladder
  numbers (§6.4, §9.1); **Nit-2** N5 detection-all-`K` + small-`K` parity floor (§6.1).

### R0 round-3 fold log (2026-06-24)

Round-3 verdict RED (0C/**1I**); full review at `design/agent-reports/word-card-r0-round-3.md`.
Both round-2 Criticals confirmed genuinely closed; all nits self-consistent; citations + core
math clean. Round-3 fold (a single propagation residual of the round-2 in-place I-A edit):

- **NEW-I-1** — synced the remaining `declared-total-length` sites to §6.3's
  **`recorded-length` ledger**: §8 step 1 (decoder truncation test), §8 step 2 + §6.1
  (bounded-desync invariant), and §9.5 (removed the double-listed dropped name).
- **Minor-1** — explicit ledger-durability note (RS-protected front header) in §6.3.
- **Nit-3** — fixed the `§8.5` label → `§8 step 5` (the load-bearing C1 anchor; §8 has no
  sub-section 8.5).
- **Minor-2** (plan-time) remains tracked as §12-Q2: exhibit a concrete `(marker|index|
  parity)` 11-bit split per K-class.

---

## 1. Motivation

The constellation's three cards (`ms1`, `mk1`, `md1`) are bech32-family strings over the
32-symbol alphabet `qpzry9x8gf2tvdw0s3jn54khce6mua7l` — **5 bits per character**. For a
steel-engravable self-custody backup that a human transcribes by hand, long runs of these
confusable glyphs are error-prone, and the existing BCH error-correction operates on
**5-bit symbols**, not on the **word-sized units humans actually mis-record**.

This spec defines an alternate, **human-writable** rendering of the *payloads* of `mk1`
and `md1` as **BIP-39 English words**, with error-correction and whole-plate redundancy
designed around the human error model (substitutions, deletions, insertions, runs, lost
plates), and with a **progressive** redundancy dial so a user can record more words for
more protection — append-only, even years later.

### 1.1 Why `ms1` is OUT of scope

`ms1` is a thin codex32 wrapper around a BIP-39 seed's **entropy** (`0x00` prefix byte +
16/20/24/28/32 B entropy; ms-codec `consts.rs:29`, verified @ `5c0335c`). Re-encoding it
to words is a **net loss**: a 12-word `ms1` is 47 data symbols ≈ 22 words to transcode the
whole string, but the underlying 128-bit secret **is already 12 BIP-39 words**. The
correct "word view" of an `ms1` is simply the original BIP-39 seed phrase. `ms1` secret
sharing/correction stays in codex32 (BIP-93). **Word Cards therefore cover only the
public-ish key/descriptor material (`mk1`, `md1`).**

### 1.2 Privacy / hygiene note

Word Cards encode **xpubs and descriptors** — public-key / policy material, NOT spending
secrets. The first-class secret-memory-hygiene bar (zeroize-on-drop, redacting `Debug`,
off-argv) that governs seed/entropy handling is **not triggered by key/descriptor bytes**,
but **xpubs are privacy-sensitive** (they reveal a wallet's addresses). The cross-plate
RAID layer (§7) is designed so a parity plate **alone leaks nothing** about any xpub.

---

## 2. Scope & granularity

| Source format | Codeword granularity | Rationale |
|---|---|---|
| `mk1` (one xpub/card) | **per-xpub** | Each cosigner key is independently recoverable; rides the k-of-n multisig redundancy already present. A 3-key set = 3 independent word-strings, each with its own ladder. |
| `md1` (descriptor) | **per-card** | A descriptor is one semantic unit (keyless template … wallet-policy with embedded xpubs). One RS codeword spans the whole `md1` string. |
| `ms1` | — | **excluded** (§1.1). |

---

## 3. Symbol domain

- **Wordlist:** **BIP-39 English**, 2048 words. Rationale: it is the dictionary users
  already know (this is a seed-phrase tool), and the error-correction (§6) already repairs
  look-alike confusions, so a bespoke confusion-minimized list is not required. (A custom
  confusion-minimized 2048-list is a documented **deferred alternative**, §12.)
- **Field:** **GF(2048) = GF(2¹¹)**; **one word = one symbol = 11 bits**. All RS / RAID
  arithmetic (§6, §7) is over this field. Field length limit: a single RS codeword is
  `n ≤ 2047` words.
- Word ⇄ 11-bit index is the canonical BIP-39 mapping. Case-insensitive; canonical output
  lowercase.

---

## 4. Architecture overview

Each Word-Card string is three stacked layers, then an optional cross-plate layer:

```
  ┌ Layer A: PAYLOAD ─ the source m*1 payload bytes, regrouped 8→11 bit into words
  │ Layer B: SYNC    ─ interspersed self-protected checkpoint words (position grid)
  │ Layer C: ECC     ─ append-only systematic RS parity tail  +  stop-sign word
  └ Layer D: RAID    ─ (mk1 only) progressive cross-plate parity plates (r=1, r=2)
```

Two **independent progressive dials** fall out:

| Dial | Unit added | Defends against | Stop points |
|---|---|---|---|
| **word tail** (Layer C, per string) | check **words** | typos / lost lines *on a present plate* | any checkpoint (printed word-ladder) |
| **recovery plates** (Layer D, per array) | whole **plates** | a *destroyed / lost* plate | r=0 → r=1 → r=2 |

Detection of any missing / extra / swapped word is **always on** (Layer B), independent of
how much of Layers C/D the user records. Only *repair* spends recorded budget.

---

## 5. Layer A — payload → words

1. **Source bytes.** Decode the `m*1` string via its codec to the canonical payload bytes:
   - `mk1`: the per-xpub bytecode core. The incompressible part is chain-code (32 B) +
     compressed pubkey (33 B) = 65 B; the mk1 "compact xpub" is 73 B (drops depth /
     child-number, reconstructs from origin path — mk-codec `xpub_compact.rs`, verified
     @ `46631c6`). Word Card encodes the **per-xpub canonical payload** (compact xpub +
     its origin framing), padded to a fixed array-wide width (§7 requires aligned stripes).
   - `md1`: the whole descriptor payload (header + paths + AST tree + TLV; md-codec
     `encode.rs:65-92`, verified @ `7764145d`). Wallet-policy mode embeds xpubs (65 B each,
     TLV tag `0x02`); keyless-template mode does not.
2. **Header word(s).** Prepend a small self-describing header (exact bit layout = OPEN,
   §12): `{format-version, source-kind (mk1|md1), K-class, checkpoint stride b,
   recorded-length ledger (append-only, front-anchored; §6.3), array-id, role}`.
   `array-id`/`role` are Layer-D
   fields (§7); for a standalone `md1` they are degenerate (`role = solo`). **`array-id` is
   a plate-MATCHING aid only — NOT the integrity check** (C1): it travels inside the
   codeword and would move with a miscorrection.
3. **Integrity tag (C1; NEW-C1 fold).** Append a dedicated **integrity tag** = a strong,
   **NON-LINEAR** function of the canonical payload: a truncated **cryptographic hash**
   (e.g. SHA-256 truncated to `t ≥ 32` bits). Recomputed and cross-checked after RS decode
   (§8 step 5); an RS *miscorrection* onto a valid-but-wrong payload survives only with
   probability `≤ 2⁻ᵗ`. **A LINEAR tag (BCH residue / CRC / XOR) is FORBIDDEN as the
   in-codeword integrity check** — it lives in the same linear RS image, so a miscorrection
   satisfies it by construction and the bound collapses. (A linear residue is admissible
   only if carried as a fully *independent, out-of-codeword* check; the non-linear
   in-codeword hash is the mandated default.) The tag words are themselves RS-protected and
   the check runs *post-correction*, so a mere tag typo is repaired, not falsely rejected.
   This replaces the old `decode(encode(B))=B` identity.
4. **Regroup (N6).** Concatenate header ∥ payload ∥ integrity-tag bits and regroup **8→11**:
   `K = ceil(total_bits / 11)` **data words** (same mapping BIP-39 uses for entropy). Layer
   A consumes the codec's **PRE-chunking canonical payload** (the single logical
   xpub/descriptor byte-string), NOT the chunked `mk1`/`md1` wire fragments.

Approximate `K` (data words):

| Source | bytes | K (data words) |
|---|---|---|
| `mk1` one xpub | ~73 B (+header) | **~54** (≈48 of which are irreducible chaincode+pubkey) |
| `md1` keyless template (e.g. BIP-84 single-sig) | ~8–20 B | **~6–15** |
| `md1` 2-of-3 wallet-policy (3 embedded xpubs) | ~200+ B | **~150** |

---

## 6. Layer B + C — sync and error-correction

### 6.1 Sync (Layer B): interspersed checkpoints

- **Block size `b ≈ √K`** (minimizes `checkpoint_overhead = K/b` + `run_slop ≈ b`; the
  classic concatenation optimum). For `K=54`, `b≈7` (⌈54/7⌉=8 checkpoints ⇒ `K′=62`); for
  `K=160`, `b≈13`.
- After every `b` payload words, insert one **checkpoint word** carrying a **self-identifying
  marker + running block index + local parity** over its block. Count ≈ `K/b ≈ √K`.
- **Checkpoint recognition / realignment (NEW-C2 — NORMATIVE).** The sync pass must
  *recognize* a checkpoint by content **before** RS runs; the marker bits provide that. After
  a desync (deleted checkpoint, or a run), the decoder re-finds the next checkpoint by a
  **bounded offset search** validated by (a) the marker and (b) **index-continuity across ≥2
  consecutive checkpoints** + their local parity. A unique consistent alignment is accepted;
  **ambiguous or no alignment ⇒ refuse-and-report** (custody-safe: never silently
  mis-align). This is what makes the C2 bounded-desync invariant *demonstrated*, not asserted.
- **Small-`K` (N5).** Detection is on for **any `K ≥ 1`** (≥1 checkpoint always present);
  tiny `md1` templates (`K<10`) use a **fixed parity floor** (§9.1) + a degenerate single
  checkpoint, so the √K rule never underflows.
- **Indel trichotomy.** Each checkpoint `Cᵢ` is expected after exactly `i·b` payload words.
  The count of words since the previous checkpoint classifies the error:

  | words since last checkpoint | local parity | verdict |
  |---|---|---|
  | `b` | ok | clean |
  | `b` | **fail** | **substitution** in block |
  | `b − 1` | — | **deletion** in block |
  | `b + 1` | — | **insertion** in block |

- **Checkpoints are themselves RS-coded symbols** (they sit *inside* the Layer-C codeword).
  A miswritten checkpoint is corrected by the global RS pass like any other word; its sync
  role is a *decode-time interpretation*, never an unprotected control channel. This closes
  the "what if the safety marker itself is wrong" hole.
- **Pinpointing is NORMATIVE (C2 — was open-Q2).** Each checkpoint's local parity MUST be
  strong enough to pinpoint a **single** intra-block indel to one slot (reinsert-and-test:
  try the `b` reinsertion positions, accept the one the local parity validates) → reduce it
  to **one known erasure**.
- **Fallback + bounded-desync invariant (C2).** When pinpointing fails — multiple indels in
  one block, or a **deleted checkpoint** (detected by index-discontinuity at the *next*
  checkpoint, whose declared index `i` arrives after `< i·b` words) — the affected block(s)
  are marked as a **whole-block erasure (cost ≤ `b`)**. The running indices + the
  recorded-length ledger (§6.3) guarantee every indel is localized to **at most block
  granularity**, so a
  single un-localized deletion can NEVER silently desync the whole codeword. This bound is
  what makes the two-pass decode (§8) well-founded.
- **Compound-case lemma (C2 / NEW-C2).** A deleted checkpoint `Cᵢ` AND a data-word deletion
  in block `i`: the merged span is detected when the next *recognizable* checkpoint arrives
  with an index/stride mismatch; the merged span is erased (cost ≤ `2b`). If that next
  checkpoint is itself unrecognizable, realignment fails ⇒ **refuse-and-report** (never a
  silent mis-decode).

### 6.2 Error-correction (Layer C): append-only systematic RS

- **Code:** systematic Reed–Solomon over GF(2048) in **evaluation form** (interpolation /
  extended-evaluation "Vandermonde" systematic RS — NOT Reed's original *coefficient* form,
  N1): parity words are independent evaluations at a **spec-frozen canonical sequence** of
  points `α₁, α₂, …`. Consequence: **any prefix `P₁…Pₘ` is itself a valid
  `[K′+m, K′]` RS code with minimum distance `m+1`** (`K′` = data + checkpoints). This is
  what makes the tail append-only and progressive. (Generator-polynomial form is NOT
  prefix-extensible and MUST NOT be used.)
- **The lever:** with `m` recorded parity words, the correction/detection budget splits as
  any `(t correct) + (s detect)` with `t + s = m`, `s ≥ t`. Equivalently:
  - repair up to **`m`** *erasures* (located damage — every deletion is located; cost 1 each),
  - or correct up to **`⌊m/2⌋`** *unlocated substitutions* (cost 2 each),
  - or any mix: `2·(substitutions) + (erasures) ≤ m`.
  - For custody safety the decoder corrects up to `⌊m/2⌋` and **refuses rather than
    silently miscorrects** beyond it.
- **Located runs are cheap.** Because §6.1 localizes bursts, a smudged/lost line becomes
  erasures (1 each), not unknown errors (2 each) — a located burst is the *easy* case.
- **Single codeword** per string (no interleaving needed at `n ≤ ~300`): RS is
  position-agnostic, so a burst up to the erasure budget is absorbed regardless of where it
  lands.

### 6.3 Stop-sign + front length-ledger (soft-terminal) — C3 / I-A

- A **stop-sign spans ≥2 words** (a single 11-bit word cannot hold a ~2047 word-count +
  marker + checksum). It carries the **cumulative total-word count** + a checksum.
- The **front header carries a `recorded-length` LEDGER (append-only, front-anchored)**: at
  creation and at *every* stop/upgrade the user appends the then-current cumulative word
  count as a new ledger entry. Authoritative recorded length = the **highest** ledger entry.
  There is exactly **ONE meaning** — "how many words were actually recorded" — which resolves
  I-A: the old `declared-total-length` double-meaning is dropped, there is no
  pre-committed-max, so a **deliberate early stop is NOT a false truncation**.
- **Truncation/downgrade test:** words physically present **<** highest ledger entry ⇒
  **truncation flag** (tail chipped/lost). Because the ledger is at the FRONT, it survives
  losing the back tail — so even a wholly-lost newest tail+stop-sign is flagged (closes C3).
  A deliberate stop wrote its own matching ledger entry ⇒ present == ledger ⇒ no false flag.
- **Append-only upgrade:** append parity at the back, write a new higher-count stop-sign, AND
  append the new cumulative count to the front ledger (the front grows one small entry per
  upgrade — acceptable steel cost). The decoder takes the highest stop-sign as authoritative
  and treats earlier mid-stream stop-signs as ordinary words.
- **Ledger durability (Minor-1).** The ledger lives in the **RS-protected front header**
  (§5.2), so a corrupted entry is repaired before the truncation test; losing an *older*
  entry is inert (authoritative = highest), and losing the *newest* requires front-header
  loss, which fails loudly rather than silently downgrading.

### 6.4 Word-ladder (the per-string progressive UX)

The mandatory prefix is data + checkpoints (`K′ ≈ K + √K` words). Legal stop points are
**checkpoint boundaries**; each adds ≈ `b` words ≈ one tier. The toolkit prints a ladder at
generation; `mnemonic recover` reports achieved strength at read-back. Example for a `mk1`
xpub (`K≈54`, 8 checkpoints ⇒ `K′≈62`):

```
MANDATORY  words 1–62   the xpub + sync. Fewer = data loss.
                        At 62: every missing/extra/swapped word DETECTED & PINPOINTED.
OPTIONAL ── stop at any ⟐ checkpoint:
  ⟐ 69  ( 7 check)  repair  7 missing OR 3 wrong
  ⟐ 82  (20 check)  repair 20 missing OR 10 wrong   ◀ ~50% "survive one lost line + typos"
  ⟐ … append-only, up to word ~2047 …
```

---

## 7. Layer D — cross-plate RAID (`mk1` arrays only)

A **second, orthogonal** redundancy axis: Layer C repairs a plate you still *have*; Layer D
reconstructs a plate you've *lost entirely*.

### 7.1 Construction (progressive r=1 → r=2)

**Prerequisite (I3 — NORMATIVE, was open-Q5):** each xpub is first normalized to a
**canonical fixed-width per-xpub payload** (pad to the array-wide maximum; exact padding
rule pinned in §9.5), so the column stripes are well-defined despite differing origin-path
framing. The r=1/r=2 MDS math below **depends on** this alignment — it is a requirement, not
an open question.

The `n` aligned xpub payloads are striped column-wise; add `r` parity stripes forming an
`[n+r, n]` MDS code at the plate level:

- **r=1 — "Recovery A"** (RAID-5 / XOR): `P₁[j] = Σᵢ xᵢ[j]` (XOR = GF(2048) addition).
  Recovers **any 1 of the `n+1`** plates.
- **r=2 — "Recovery B"** (RAID-6 / RS): `P₂[j] = Σᵢ αⁱ · xᵢ[j]`. With `P₁,P₂` (the first
  two RS syndromes) recovers **any 2 of the `n+2`** plates.
- `P₁` is **unchanged** when `P₂` is added (same fixed evaluation sequence as §6.2) ⇒ the
  RAID layer is **append-only at plate granularity**, the same primitive as the word tail.
- Each Recovery plate is **itself a full Word-Card string** (its own Layer A/B/C), so it
  self-heals typos like any data plate.
- Guarantee is stated honestly as "**lose any `r` of the `n+r` plates**" (parity plates
  count toward the budget), NOT "any `r` data plates."

### 7.2 Array identity & legibility (the user-facing stop points)

The RAID stop points are **whole labeled plates** — far easier to make legible than the
word-level ladder:

1. **Human title engraved on each plate:** `KEY 1/3`, `KEY 2/3`, `KEY 3/3`,
   `RECOVERY A — survive any 1 plate lost`, `RECOVERY B — survive any 2 plates lost`.
   The stop point *is* "how many Recovery plates exist," and each plate states its tolerance.
2. **Machine header (Layer A) on every plate:** `array-id` (a ~1–2 word hash of the `n`
   ordered xpub fingerprints) + `role` (`data i/n` | `parity 1|2`) + `n`. The `array-id`
   fixes **stripe order** (which `P₂`'s α-weighting depends on), lets a recovery tool match
   plates of one wallet, and prevents accidentally mixing plates across different multisigs.
3. **Read-back report:** `mnemonic recover` reads available plates and reports e.g.
   *"Array a3f… : found KEY 1, KEY 3, RECOVERY A (3 of 4) — reconstructing KEY 2."*

### 7.3 Privacy & distribution

Any set of **fewer than `n`** plates (data or parity) is information-theoretically
uninformative about the full xpub set — in particular a **lone Recovery plate reveals
nothing** (it is `r` linear combinations of `n` unknowns; for r=2 the two parity plates
*together* are still only 2 of the `n` equations needed, N2). Safe to store off-site / with
a third party. Distribute so no location holds more than `r` plates to survive losing a
whole location.

### 7.4 Conditional emission (overlap with wallet-policy `md1`)

A **wallet-policy `md1` card already embeds all `n` xpubs** (TLV `0x02`, md-codec @
`7764145d`) — it is *already* a cross-plate xpub backup. Therefore:

- **RAID is NOT auto-suppressed when a wallet-policy `md1` is present (I2 — flipped).** A
  single `md1` plate is itself one point of failure: losing it **plus** one `mk1` plate is
  unrecoverable, whereas Recovery-A survives **any** one plate lost. So RAID-A is retained
  by default; the `md1` is noted as *additional* coverage, not a replacement.
- Auto-suppression is offered only as an **explicit, coverage-verified opt-in** (the user
  affirms the `md1` is independently backed up).
- If `md1` is **keyless-template** or `mk1` is backed up standalone ⇒ RAID is the only
  cross-plate recovery ⇒ always emit.

---

## 8. Decoder algorithm (per string)

1. Normalize case; map words → 11-bit symbols; read the front header's **`recorded-length`
   ledger** and take its **highest entry** (= the highest-count stop-sign) as authoritative.
   **Flag truncation when words-present < highest ledger entry** (C3 / I-A) — not merely when
   a stop-sign is absent; a deliberate early stop wrote a matching ledger entry, so it is NOT
   a false truncation.
2. **Sync pass (Layer B):** walk checkpoints; classify each block (clean / substitution /
   deletion / insertion, §6.1); rebuild the full-length grid; mark erasures at pinpointed
   indels. **Bounded-desync invariant (C2):** every indel localizes to ≥ block granularity
   (running indices + the recorded-length ledger); an indel that cannot be pinpointed to one slot
   degrades to a **whole-block erasure (cost ≤ `b`)**, never an unbounded whole-codeword
   desync — which is what makes this two-pass decode well-founded.
3. **RS pass (Layer C):** decode the systematic RS codeword over the grid (Welch–Berlekamp
   / Gao), correcting substitutions + filling erasures within budget; **refuse** if the
   error weight exceeds `⌊m/2⌋` + erasure budget. (Refusal handles *failure*; a
   *miscorrection* within budget is handled by step 5, not here — see C1.)
4. **Re-verify** the sync grid against the corrected symbols (catches a corrupted checkpoint
   that was repaired in step 3).
5. Strip checkpoints + header; **regroup 11→8** to recover the source payload bytes.
   **Integrity cross-check (C1 — replaces the old round-trip):** recompute the independent
   integrity tag (§5.3) over the recovered payload and require equality with the stored tag.
   An RS *miscorrection* onto a valid-but-wrong payload survives this only with probability
   `≤ 2⁻ᵗ` (default ≤ 2⁻³²). The removed `decode(encode(B))=B` check was an identity that
   caught structural garbage but never a structurally-valid wrong xpub.
6. **RAID pass (Layer D)**, if reconstructing an array: gather available plates by
   `array-id`; if `≤ r` plates are missing, solve the `[n+r, n]` system for the missing
   stripes; each reconstructed stripe is a full Word-Card string for that xpub.

---

## 9. Guarantees & bounds

Two **DISTINCT** guarantees — do not conflate (I1):

**(a) Value layer — MDS-optimal (substitutions + located erasures).** With `m` parity
words: `2·(substitutions) + (erasures) ≤ m`. This is the **MDS ceiling** — no code at this
overhead does better, and RS meets it.

**(b) Indel layer — sync-bounded, NOT MDS.** Detection of any missing / extra / swapped word
is **always on** (mandatory prefix, independent of `m`). *Repair* of an indel reduces it to
erasure(s): **cost 1/word when pinpointed within its block, otherwise ≤ `b` per affected
block (C2).** "Located runs cost 1/word" holds only under successful per-slot pinpointing;
the honest worst case is `b` erasures per damaged block.

- **Per-array survival** (`r` Recovery plates): lose any `r` of `n+r` plates.
- **Append-only** on both axes, up to the GF(2048) length cap (`n ≤ 2047` words/string;
  with `K′ ≈ 62` that is ~1985 appendable parity words — effectively unbounded).
- **Custody safety (C1 — corrected from an over-claim):** the design does **not** claim the
  decoder *never* miscorrects — a bounded-distance RS decoder can land on a valid-but-wrong
  codeword within `⌊m/2⌋`. That event is caught by the independent integrity tag
  (§5.3 / §8 step 5) with residual `≤ 2⁻ᵗ` (default ≤ 2⁻³²); only then is the result trusted.
  Beyond the correction budget the decoder refuses and reports the implicated words/plates.

### 9.1 Worked numbers — 3-key multisig (per-xpub `mk1`, `K≈54`)

| per-xpub overhead | sync | parity `m` | corrects | survives (single plate) |
|---|---|---|---|---|
| 25% | ~8 | ~6 | 3 wrong / 6 missing | scattered typos |
| **~50%** | ~8 | ~20 | 10 wrong / 20 missing | **one lost line + several typos** |
| 100% | ~8 | ~46 | 23 wrong / 46 missing | most of a plate |

Plate-level: `+ Recovery A` (r=1) ⇒ survive any 1 of 4 plates; `+ Recovery B` (r=2) ⇒
survive any 2 of 5 plates. Default recommendation: **~50% word tails + Recovery A**.

---

## 9.5 Frozen constants (normative — pin for 20-year recoverability) — I4

These MUST be fixed by the spec and never changed (the top recoverability risk after C1);
the plan-doc assigns concrete values:

- **Field:** GF(2¹¹) with a named **primitive polynomial**.
- **Symbol map:** the canonical **BIP-39 English index map** (word ⇄ 11-bit value).
- **RS evaluation sequence:** the ordered points `α₁, α₂, …` for the append-only tail (§6.2).
- **RAID generator `α`:** with `ord(α) ≥ n_max` (REQUIRED for r=2 MDS; §7.1).
- **Integrity tag (NEW-C1):** the **non-linear cryptographic hash family** + truncation
  bit-width `t` (§5.3) — a linear tag is forbidden in-codeword.
- **Checkpoint self-identifying marker + local-parity (NEW-C2):** the marker pattern + the
  11-bit split (marker / index / parity) + the bounded-realignment search ceiling (§6.1).
- **Stop-sign + front length-ledger encodings:** field widths + ledger-entry size (§6.3).
- **Header bit-layout:** all fields (§5.2) — the `recorded-length` ledger is frozen by the
  stop-sign/ledger bullet above; do NOT re-introduce the removed `declared-total-length`.
- **Canonical fixed-width per-xpub payload** padding rule for RAID striping (§7.1).

## 10. Toolkit integration

- **New rendering**, NOT a replacement for `m*1`. Proposed surface (exact spelling = OPEN):
  a Word-Card output mode of `bundle` and/or a dedicated `mnemonic word-card` / `mnemonic
  recover` pair. Encoder is deterministic (fixed `array-id`, fixed RS points) ⇒
  binary-identical output for docs (`verify-examples` discipline applies).
- **Lockstep obligations (per `CLAUDE.md`):**
  - Any new flag/subcommand/dropdown **MUST** update `mnemonic-gui/src/schema/mnemonic.rs`
    in the same/paired PR (`schema_mirror` is a *lagging* gate).
  - Any CLI-surface change **MUST** update `docs/manual/src/40-cli-reference/` in lockstep
    (`docs/manual/tests/lint.sh`).
  - New `ToolkitError` variants + their exhaustive `match` arms: **alphabetical order**.
  - All doc CLI-output blocks must be **binary-generated/identical** (fixed seeds).
  - `schema_mirror` gates flag-**NAMES** + dropdown **VALUE** enums, **not `--json`
    wire-shape** (value-only adds ride the paired-PR rule).
  - **Release version-sites (N4 — many NOT gate-enforced):** `CHANGELOG.md` (tag-gated
    `changelog-check`), **both** READMEs, `fuzz/Cargo.lock`, `scripts/install.sh` sibling
    pins, generated **man-pages** (`gen-man`). Re-run full suite + fuzz build after any
    version bump, before tag.
- **Where the code lives = OPEN** (§12): leaning toward a **new sibling crate**
  (`word-card` / `wc-codec`) consumed by the toolkit, mirroring the codec-per-format
  pattern, so `md`/`mk`/`ms` CLIs could each gain a word view independently.

---

## 11. Non-goals

- Re-encoding `ms1` (use the BIP-39 seed phrase directly).
- Secret-sharing / privacy thresholds (that is codex32/`ms1`'s job; Word Cards are
  availability coding over public-ish material).
- Replacing the `m*1` wire format or its BCH layer.
- A general-purpose file ECC tool.

---

## 12. Open questions (for R0)

1. **Header bit layout** — exact fields/widths for `{version, source-kind, K-class, stride,
   array-id, role}`; how `array-id` is derived (hash of ordered fingerprints — which hash,
   truncated to how many words) and its collision target.
2. **Local-parity scheme** inside each checkpoint word — *pinpoint-or-block-erasure* is now
   **NORMATIVE** (§6.1, C2 fold); remaining open = the exact 11-bit split (index vs parity)
   and the reinsert-and-test cost ceiling.
3. **Crate boundary** — new sibling crate vs toolkit module vs per-codec extension. Who
   owns the RS/RAID engine (shared lib?).
4. **Wordlist** — confirm BIP-39 English vs a confusion-minimized 2048-list; if the latter,
   sourcing + edit-distance criteria. (Default: BIP-39 English.)
5. **Stripe alignment** for `mk1` arrays — canonical fixed-width per-xpub payload is now
   **NORMATIVE** (§7.1, §9.5, I3 fold); remaining open = the exact padding rule.
6. **Soft-terminal exactness** — stop-sign encoding (count + which checksum), and the rule
   for choosing "the last" stop-sign under tail corruption.
7. **md1 size extremes** — detection-all-`K` + a fixed parity floor + degenerate single
   checkpoint for tiny templates is now **NORMATIVE** (§6.1, N5 fold); remaining open = the
   exact floor value.
8. **Optional interleaving** — only if runs dominate beyond a single codeword's budget
   (probably unnecessary; documented lever).
9. **Beyond r=2** — construction already supports `r≥3`; confirm we cap surfaced stop
   points at r=2.

---

## 13. Constellation follow-ups (to mirror on approval)

- File `FOLLOWUPS.md` entries in `mnemonic-toolkit` and any owning sibling repo with
  cross-citing `Companion:` lines (per `CLAUDE.md` cross-repo rule).
- If a new sibling crate is chosen, register it in the `CLAUDE.md` constellation list.

---

## 14. Next step

**Mandatory opus R0 architect review** of this spec to 0C/0I before any plan-doc or code,
per the repo's pre-implementation gate. Persist the verbatim review to
`design/agent-reports/` and re-dispatch after each fold until GREEN.
