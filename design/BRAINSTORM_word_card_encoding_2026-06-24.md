# BRAINSTORM / SPEC — Engravable Word-Card encoding for `mk1` / `md1`

- **Status:** Brainstorm spec — pre-R0. NOT approved for implementation.
- **Date:** 2026-06-24
- **Author:** brainstorm session (single author; awaiting mandatory opus R0 gate)
- **Working name:** **Word Card (WC)** — provisional, rename welcome.
- **Source SHAs (wire-format facts cited below were read at these revisions):**
  - `mnemonic-toolkit` @ `60af98dd`
  - `descriptor-mnemonic` (md-codec) @ `7764145d`
  - `mnemonic-key` (mk-codec) @ `46631c6`
  - `mnemonic-secret` (ms-codec) @ `5c0335c`

> This document is the output of a design dialogue. Per `CLAUDE.md` it MUST pass an
> opus architect **R0 review to 0 Critical / 0 Important BEFORE any implementation**,
> and the reviewer-loop continues after every fold. No code until GREEN.

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
   array-id, role}`. `array-id` and `role` are Layer-D fields (§7); for a standalone `md1`
   they are degenerate (`role = solo`).
3. **Regroup.** Concatenate header ∥ payload bits and regroup the bitstream **8→11**:
   `K = ceil(total_bits / 11)` **data words**. (This is exactly how BIP-39 maps entropy to
   words; here the "entropy" is the codec payload.)

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
  classic concatenation optimum). For `K=54`, `b≈7`; for `K=160`, `b≈12`.
- After every `b` payload words, insert one **checkpoint word** carrying a **running block
  index** + a **local parity** over its block. Count ≈ `K/b ≈ √K` checkpoints.
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
- A detected+localized deletion is reduced to a **known erasure** (re-insert a blank
  placeholder at the pinpointed slot → global synchronization restored).

### 6.2 Error-correction (Layer C): append-only systematic RS

- **Code:** systematic Reed–Solomon over GF(2048) in **evaluation form** (Reed's original
  construction): parity words are independent evaluations at a **spec-frozen canonical
  sequence** of points `α₁, α₂, …`. Consequence: **any prefix `P₁…Pₘ` is itself a valid
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

### 6.3 Stop-sign word (soft-terminal)

- The string ends in a **stop-sign word** = a terminal marker carrying a **total-word count
  + checksum**. Semantics:
  - Stop-sign present and consistent ⇒ the string is **whole**.
  - Stop-sign absent at the end ⇒ the tail was **truncated** (end-of-string deletions);
    flagged, never mistaken for an intentional stop.
  - **Append-only upgrade:** to harden later, append more parity words then write a **new**
    stop-sign after them. The decoder takes the **last** stop-sign as authoritative and
    treats any earlier (now mid-stream) stop-sign as an ordinary word. ("Soft" = lenient to
    leftover markers.)

### 6.4 Word-ladder (the per-string progressive UX)

The mandatory prefix is data + checkpoints (`K′ ≈ K + √K` words). Legal stop points are
**checkpoint boundaries**; each adds ≈ `b` words ≈ one tier. The toolkit prints a ladder at
generation; `mnemonic recover` reports achieved strength at read-back. Example for a `mk1`
xpub (`K≈54`, `K′≈61`):

```
MANDATORY  words 1–61   the xpub + sync. Fewer = data loss.
                        At 61: every missing/extra/swapped word DETECTED & PINPOINTED.
OPTIONAL ── stop at any ⟐ checkpoint:
  ⟐ 68  ( 7 check)  repair  7 missing OR 3 wrong
  ⟐ 81  (20 check)  repair 20 missing OR 10 wrong   ◀ ~50% "survive one lost line + typos"
  ⟐ … append-only, up to word ~2047 …
```

---

## 7. Layer D — cross-plate RAID (`mk1` arrays only)

A **second, orthogonal** redundancy axis: Layer C repairs a plate you still *have*; Layer D
reconstructs a plate you've *lost entirely*.

### 7.1 Construction (progressive r=1 → r=2)

`n` aligned xpub payloads are striped column-wise; add `r` parity stripes forming an
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

A single Recovery plate is one linear combination of `n` unknowns ⇒ **reveals nothing**
about any individual xpub until combined with `n−1` real plates. Safe to store off-site /
with a third party. Distribute so no location holds more than `r` plates to survive losing
a whole location.

### 7.4 Conditional emission (overlap with wallet-policy `md1`)

A **wallet-policy `md1` card already embeds all `n` xpubs** (TLV `0x02`, md-codec @
`7764145d`) — it is *already* a cross-plate xpub backup. Therefore:

- If a wallet-policy `md1` Word Card is part of the same bundle ⇒ **do not auto-emit RAID**;
  the `md1` card is the cross-plate recovery (toolkit says so).
- If `md1` is **keyless-template**, or `mk1` is backed up standalone ⇒ RAID is the only
  cross-plate recovery ⇒ **emit** (per chosen `r`).
- RAID remains available as an **explicit opt-in** even when it overlaps `md1`.

---

## 8. Decoder algorithm (per string)

1. Normalize case; map words → 11-bit symbols; locate the authoritative **stop-sign**
   (last one); flag truncation if absent.
2. **Sync pass (Layer B):** walk checkpoints; classify each block (clean / substitution /
   deletion / insertion, §6.1); rebuild the full-length grid; mark erasures at pinpointed
   indels.
3. **RS pass (Layer C):** decode the systematic RS codeword over the grid (Welch–Berlekamp
   / Gao), correcting substitutions and filling erasures within budget; **refuse** (no
   silent miscorrect) if the error weight exceeds `⌊m/2⌋` + erasure budget.
4. **Re-verify** the sync grid against the corrected symbols (catches a corrupted checkpoint
   that was repaired in step 3).
5. Strip checkpoints + header; **regroup 11→8** to recover the source payload bytes;
   re-encode through the codec and **assert the round-trip** matches the declared
   `array-id`/payload (defends against an undetected-but-decodable corruption).
6. **RAID pass (Layer D)**, if reconstructing an array: gather available plates by
   `array-id`; if `≤ r` plates are missing, solve the `[n+r, n]` system for the missing
   stripes; each reconstructed stripe is a full Word-Card string for that xpub.

---

## 9. Guarantees & bounds

- **Detection** of any missing / extra / swapped word: **always**, from the mandatory
  prefix, independent of recorded parity.
- **Per-string repair** (with `m` parity words): `2·(substitutions) + (erasures) ≤ m`;
  located runs cost 1/word. This is the **MDS ceiling** — no code at this overhead does
  better, and RS meets it.
- **Per-array survival** (`r` Recovery plates): lose any `r` of `n+r` plates.
- **Append-only** on both axes, up to the GF(2048) length cap (`n ≤ 2047` words/string;
  with `K′ ≈ 61` that is ~1980 appendable parity words — effectively unbounded).
- **Custody-safe:** the decoder never silently miscorrects; beyond budget it refuses and
  reports which words/plates are implicated.

### 9.1 Worked numbers — 3-key multisig (per-xpub `mk1`, `K≈54`)

| per-xpub overhead | sync | parity `m` | corrects | survives (single plate) |
|---|---|---|---|---|
| 25% | ~7 | ~7 | 3 wrong / 7 missing | scattered typos |
| **~50%** | ~7 | ~20 | 10 wrong / 20 missing | **one lost line + several typos** |
| 100% | ~7 | ~48 | 24 wrong / 48 missing | most of a plate |

Plate-level: `+ Recovery A` (r=1) ⇒ survive any 1 of 4 plates; `+ Recovery B` (r=2) ⇒
survive any 2 of 5 plates. Default recommendation: **~50% word tails + Recovery A**.

---

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
2. **Local-parity scheme** inside each checkpoint word — how many of the 11 bits are index
   vs parity; whether it can *pinpoint* a single intra-block deletion (reinsert-and-test)
   vs only flag the block.
3. **Crate boundary** — new sibling crate vs toolkit module vs per-codec extension. Who
   owns the RS/RAID engine (shared lib?).
4. **Wordlist** — confirm BIP-39 English vs a confusion-minimized 2048-list; if the latter,
   sourcing + edit-distance criteria. (Default: BIP-39 English.)
5. **Stripe alignment** for `mk1` arrays — canonical fixed-width per-xpub payload (padding
   rules) so column striping is well-defined across differing origin-path lengths.
6. **Soft-terminal exactness** — stop-sign encoding (count + which checksum), and the rule
   for choosing "the last" stop-sign under tail corruption.
7. **md1 size extremes** — fixed parity floor for tiny keyless templates; behavior of `√K`
   sync when `K` is very small (`<10`).
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
