# Plan-doc R0 review — Engravable Word-Card encoding — ROUND 4 (convergence gate)

- **Artifact under review:** `design/IMPLEMENTATION_PLAN_word_card_encoding.md` (round-1 + round-2 + round-3 folds applied)
- **Authoritative spec (R0-GREEN):** `design/BRAINSTORM_word_card_encoding_2026-06-24.md` (commit `31109f8e`)
- **Round-3 review (folded):** `design/agent-reports/word-card-plan-r0-round-3.md` (RED 0C/1I/1n)
- **Reviewer:** opus architect (mandatory pre-implementation plan-doc R0 gate; 0C/0I required; reviewer-loop continues after every fold)
- **Date:** 2026-06-24
- **Repo HEAD at review:** `813c8949` ("docs(design): fold Word-Card plan-R0 round-3 (0C/1I/1n)")

---

## Verdict

**GREEN — 0 Critical / 0 Important.** (0 Minor that blocks; 1 benign documentation nit, non-blocking, noted below.)

The reviewer loop has converged: **R1 2C/3I → R2 0C/2I → R3 0C/1I → R4 0C/0I.** The single
round-3 Important (NEW-I3) is **genuinely and fully closed**, machine-re-verified below. The final
adversarial whole-plan consistency pass — small AND large cold-decode traced end-to-end, every
frozen constant re-checked, every cross-reference walked across four rounds of in-place edits — found
**no internal contradiction, no dangling cross-ref, and no number that fails to reconcile.** This
plan-doc clears the mandatory pre-implementation R0 gate.

I did not rubber-stamp: I re-derived the rounding rule's totality from first principles (proving no
tie can ever exist), swept `floor(√K + 0.5)` against the exact nearest-integer across the full
admissible `K` range including the float-precision boundary cases, traced both the brief's small case
and a fresh large case, and re-confirmed the field/RS/RAID/CRC-5 algebra was not perturbed by the
NEW-I3 fold. All checks pass.

### Math I re-verified myself this round (NOT carried over on trust)

| Claim | Method | Result |
|---|---|---|
| `floor(√K + 0.5)` is **total / tie-free** for all integer `K` | `√K` is integer (perfect square, frac 0) or irrational (frac ≠ .5); `√K = m+0.5 ⇔ 4K=(2m+1)²` is even=odd, **impossible** | ✅ NO tie ever — banker's-vs-half-up is moot |
| `floor(√K + 0.5)` == exact nearest-integer, **no float wobble**, `K ∈ 1..2099` + boundary stress `K=m²±m`, `m≤50` | swept vs exact `isqrt`-based nearest predicate (`m²−m+1 ≤ K ≤ m²+m`) | ✅ 0 mismatches over the whole RS-admissible range |
| Encoder and decoder derive the **same** `b` | both apply the identical closed-form `b = floor(√K + 0.5)` to the same `K = ceil((8·payload_len+t)/11)`; `b` is **never stored/read** (§4.2 L195, §4.3 L214, §5 L271 all say "DERIVED") | ✅ no consistency surface; cannot disagree |
| **No operative `stride b` field remains** | only occurrence of `stride b`/`b(4)` is L65 (round-3 fold-log history); GEOM word C (L191–192) = `U(3) │ reserved(8)` | ✅ field dropped; 4 freed bits absorbed into `reserved(4)→reserved(8)` |
| **GEOM still = 4 words** after the change | A+B `payload_len(16)+t(6)=22`=2w; C `U(3)+reserved(8)=11`=1w; D `CRC(11)`=1w | ✅ 4 words (plan says "fixed 4 words") |
| `K`/`b`/`checkpoints`/`|header|`/`K′`/`m_present` interlock with `b` **derived** | re-ran closed-form for solo + RAID + tiny; ledger-offset non-circularity holds | ✅ all reconcile (traces below) |
| **K≥241 boundary** correctly derived | `K=240→b=15` (last that fit old 4-bit), `K=241→b=16` (the exact NEW-I3 cliff), `K=368/732/1095→b=19/27/33` | ✅ matches §7 P4 KAT's stated `b≈19–33` |
| Field `0x805`/`α=x` ord 2047=23·89; CRC-5 `x⁵+x²+1`; RS eval-form append-only n≤2047; RAID `index(5)` distinct over 0..31, ord(α)>32 ⇒ r=2 MDS all n≤32; `t=44` residual 2⁻⁴⁴ | grep-diff vs round-3 machine-verified §3 | ✅ byte-identical — fold did not touch the math layers |
| H1 = `5+2+5+10 = 22` (2 words) | bit count | ✅ unchanged |
| Deps `sha2="0.10"` (Cargo.toml:47), `bip39 v2 all-languages` (:49), members `["crates/mnemonic-toolkit"]` (:2) | re-checked at HEAD | ✅ accurate |

---

## Closure of NEW-I3

Round-3 NEW-I3: the GEOM `stride b` was a **4-bit field (max 15)** but `b = round(√K)` reaches ~33
for the large `md1` wallet-policies (`K≥241`, payload ≳ 325 B) the format explicitly admits — a
frozen-constant width too narrow for an in-range input. Recommended fix (option 1): **drop the field
and derive `b`**, freezing the exact rounding rule.

The fold applied **exactly option 1**. I confirmed every sub-question the round-4 brief raised:

**(a) NO operative `stride b` field remains — ✅ verified.**
The only `stride b` / `b(4)` string in the document is L65, inside the **"Plan-R0 round-3 fold log"**
section — pure append-only history documenting the removal ("the M-3 fold pinned GEOM `stride b(4)`
… **`b` is now DERIVED, not stored**"). The operative GEOM definition (§4.2 L190–192) is now:
`word C = U(3: reserved ledger slots) │ reserved(8)`. No `b`. The L34 mention of GEOM carrying
"`payload_len`, `t`, `b`" is likewise inside the **round-1 fold log** (historical state when `b` was
stored) — not operative text. Both are correctly quarantined to fold-log history.

**(b) Encoder and decoder derive the SAME `b`; the rounding rule is unambiguous — ✅ verified.**
`b = floor(√K + 0.5)` is applied identically in both directions:
- §4.2 L195 (geometry recovery): "`b = floor(√K + 0.5)` (DERIVED, not stored — NEW-I3; frozen rounding)"
- §4.3 L214 (encoder insertion rule): "Inserted after every `b` payload-data words, `b = floor(√K + 0.5)`, DERIVED from `K`"
- §5 L271 (decoder): "derive `(K, t, K′, b, m_present)` in closed form"

`b` is **never read from the header** — it is computed from `K`, and `K` is itself closed-form from
the CRC'd GEOM (`payload_len`, `t`). There is therefore **no encode/decode consistency surface to
drift**; this is strictly stronger than a stored field.

On the tie/boundary concern the brief flagged: **`floor(√K + 0.5)` has no tie for any integer K.**
For `√K` to land exactly on a `.5` boundary we would need `√K = m + 0.5`, i.e. `4K = (2m+1)²` — but
`4K` is even and `(2m+1)²` is odd, a contradiction. So `√K` is either an integer (frac 0) or
irrational (frac ≠ .5); it can **never** have fractional part exactly 0.5. The rule is total and
deterministic, and banker's-vs-half-up tie-breaking is irrelevant (no tie exists). I additionally
swept the float implementation against an exact `isqrt`-based nearest-integer predicate across
`K ∈ 1..2099` plus the boundary-stress points `K = m²±m, m², m²±m+1` for `m ≤ 50`: **0 mismatches**,
so the float `floor(√K + 0.5)` does not wobble at any boundary in the admissible range either.
*(Plan-level: the §3 frozen-constants list does not yet enumerate the rounding rule among the pinned
constants, and the exact integer-only implementation of `floor(√K + 0.5)` — e.g. via `isqrt` — is an
implementation detail. This is a P1/P3 freeze-and-KAT item, NOT an R0 blocker: the rule is
mathematically unambiguous and the boundary KAT at §7 P4 pins it. Flag for the implementing phase to
pin "compute `b` via integer arithmetic, KAT the `K=240/241` boundary" — already implied by the P4
KAT.)*

**(c) K / b / checkpoints / |header| / K′ / m_present interlock with `b` derived — ✅ verified.**
Removing `b` from GEOM word C kept GEOM at exactly 4 words (the freed 4 bits went to
`reserved(4)→reserved(8)`), so `|header| = 1 + (4 if has-raid) + 4(GEOM) + 2U` is **unchanged** and
all downstream quantities (`payload_offset`, `K′`, `m_present`) still close. The ledger-offset
non-circularity (`|header|−2U == GEOM-end`) holds for all `(has_raid,U) ∈ {(0,3),(1,3),(0,1),(1,1)}`
→ `{5,9,5,9}` matching GEOM-end → `{5,9,5,9}`. Verified by computation. The `b`-derivation sits
entirely downstream of `K` and feeds only `checkpoints = ceil(K/b)` (K≥16) — never feeds back into
`K`, `|header|`, or the RS message length, so there is no circular dependency.

**(d) K≥241 boundary KAT present — ✅ verified.**
§7 P4 (L340–341): "large-md1 `K≥241` (derived `b≈19–33`) boundary round-trip (NEW-I3)." The stated
`b≈19–33` range is exactly correct: my computation gives `K=368→b=19` (≈500 B) up to `K=1095→b=33`
(≈1500 B). The KAT is at the right boundary (`K=241` is the precise cliff where the old 4-bit field
overflowed: `b(240)=15`, `b(241)=16`).

**SHA confirmation:** HEAD is `813c8949` (the round-3 fold commit). The plan header L7 cites toolkit
`352b1adf` (the round-2 commit) — stale by exactly one commit, the **same benign decay** flagged at
every prior round: a fold commit cannot cite its own not-yet-created SHA, so it records the prior
HEAD. Deps re-verified accurate at `813c8949`. This is the brief's anticipated "`352b1adf` /
`813c8949` (HEAD)" pattern; it is a documentation nit (Nit-1 below), not a gate blocker.

**NEW-I3: CLOSED.**

---

## Final consistency pass

### Small cold-decode trace — solo mk1 xpub (payload 73 B, t=44, U=3, has-raid=0)

A cold decoder, given only words-present + the positional CRC'd GEOM, recovers every quantity with
`b` **derived**:

```
K              = ceil((8·73 + 44)/11) = ceil(628/11) = ceil(57.09) = 58       ✓
b              = floor(√58 + 0.5) = floor(7.616 + 0.5) = floor(8.116) = 8     ✓ (DERIVED, not read)
checkpoints    = ceil(58/8) = 8         (K≥16, so not the degenerate-1 floor)  ✓
|header|       = 1(H0) + 0(no H1/array-id) + 4(GEOM) + 2·3(ledger) = 11        ✓
payload_offset = |header| = 11                                                ✓
K′             = |header| + K + checkpoints = 11 + 58 + 8 = 77                 ✓
   (with m=8 parity + 2-word stop-sign: words_present = 77 + 8 + 2 = 87)
m_present      = words_present − K′ − |stop-sign| = 87 − 77 − 2 = 8            ✓ (recovers m)
```

Every quantity derivable from `payload_len`+`t`+`U` (CRC'd GEOM) + `has-raid` (H0) + `words_present`;
`b` derived, no off-by-one, no post-RS dependency, no circularity. Matches round-3's verified trace.

### Large cold-decode trace — md1 wallet-policy (~500 B, t=44, U=3, has-raid=0)

A wallet-policy `md1` is a single per-card codeword (RAID striping is mk1-array-only; `has-raid=0` for
a standalone md1 — §2, §7), so:

```
K              = ceil((8·500 + 44)/11) = ceil(4044/11) = ceil(367.6) = 368    ✓
b              = floor(√368 + 0.5) = floor(19.18 + 0.5) = floor(19.68) = 19   ✓ (>15 — the old 4-bit field would have OVERFLOWED here; now fine)
checkpoints    = ceil(368/19) = ceil(19.4) = 20                               ✓
|header|       = 1 + 0 + 4 + 6 = 11                                           ✓
payload_offset = 11                                                          ✓
K′             = 11 + 368 + 20 = 399                                          ✓
RS-cap check   = K′ + parity ≤ 2047  ⇒  399 leaves 1648 appendable parity words ✓ (b≤44 < no field cap)
   (with m=50 parity + 2-word stop: words_present = 399 + 50 + 2 = 451)
m_present      = 451 − 399 − 2 = 50                                           ✓ (recovers m)
```

This is the exact regime NEW-I3 fixed: `b=19 > 15` would have been unrepresentable in the old 4-bit
field; with derivation it is recovered cleanly and the card stays well under the RS length cap. I also
spot-checked `K=732` (≈1000 B, `b=27`) and `K=1095` (≈1500 B, `b=33`) — both derive correctly and
remain under `n ≤ 2047`. The decoder recovers every quantity from words-present + GEOM with `b`
derived; no off-by-one, no circularity, no field-width cap.

### Section-by-section agreement (four rounds of in-place edits)

- **§1 resolution table** — all 10 spec open-Qs map to a §ref; Q1/Q2/Q5/Q6/Q7 entries match their
  bodies; no entry references a stored `b`. ✓
- **§3 frozen constants** — field `0x805`/`α=x`/ord 2047=23·89, CRC-5 `x⁵+x²+1`, RS eval-form
  append-only n≤2047, RAID `index-in-array(5)` = `P₂` exponent (NOT array-id), `t=44` residual 2⁻⁴⁴,
  three distinct markers (`0b101`/`0b1111`/`0b1110`). Byte-identical to round-3's machine-verified
  text; the NEW-I3 fold touched none of it. ✓
- **§4 wire layout** — H0(1)/H1(2)/array-id(2)/GEOM(4: A+B `payload_len(16)│t(6)`, C `U(3)│
  reserved(8)`, D `CRC(11)`)/ledger(2U). GEOM C no longer carries `b`; GEOM still totals 4 words;
  `|header|` formula consistent across §4.2, §5, and the M1 trace. ✓
- **§5 algorithms** — encode inserts checkpoints at derived `b`; decode step (1) derives
  `(K,t,K′,b,m_present)` in closed form (lists `b` among *derived*, correct); the global-tag step-5
  alignment oracle and post-correction SHA-256 equality preserved. ✓
- **§6 API** — `wc-codec` signatures, `WcError` alphabetical, no-zeroize rationale unchanged. ✓
- **§7 phases** — P4 carries the `K≥241` (`b≈19–33`) boundary KAT (NEW-I3); P0–P6 KAT coverage
  (CRC-5 floor, mod-8 aliasing, global-tag deletion, r=2 MDS n>8, cold-decode, truncation,
  header-CRC-refuse) intact. ✓
- **§8 lockstep** — schema_mirror, manual `40-cli-reference`, ToolkitError alphabetical,
  binary-identical docs, CHANGELOG/READMEs/fuzz-lock/install.sh/man-pages, wc-codec fuzz target,
  `recover --json` paired-PR coordination, post-impl whole-diff review. Complete; no regression. ✓

### Spec faithfulness & deferred-item audit

- **Non-linear integrity tag** (spec C1/NEW-C1): preserved (§3, §4.5) — linear in-codeword tag
  forbidden; post-correction SHA-256 equality is the C1 miscorrection guard AND the deletion-alignment
  oracle. ✓
- **Bounded-desync / whole-block-erasure** (spec C2): preserved (§4.3) — global-tag pinpoint,
  per-block-linear multi-deletion, mod-8 aliasing bound, refuse-on-ambiguity. ✓
- **Recorded-length ledger** (spec C3/I-A): faithful — exact 2047-cap counts, fixed-`U` positional
  ledger (NEW-I1), max-over-slots-and-stop-signs. ✓
- **RAID r=2 MDS** (spec §7.1): holds for all n≤32 (NEW-I2). ✓ **RAID privacy** (spec §7.3):
  lone-parity KAT preserved. ✓
- **Deferred §12 items correctly deferred, not silently assumed:** Q8 interleaving (§9 non-goal),
  r≥3 (construction supports, not surfaced — §4.6/§9), custom wordlist (BIP-39 English chosen),
  wc-codec extraction (deferred — §9). Each is an explicit deferral with rationale, not an
  unstated assumption. ✓
- **NEW risk from the round-3 fold:** none in the math layers (untouched). The fold only removed a
  field and added the derivation + boundary KAT — a strict simplification (one fewer stored constant,
  one fewer encode/decode consistency surface). The freed 4 bits go to `reserved`, leaving headroom.

The only number that did not reconcile in round 3 — `stride b(4)` vs `b = round(√K)` for large K — is
**gone** (the field is removed). Nothing else fails to reconcile.

---

## Minor / Nit (non-blocking — does NOT hold the gate)

### Nit-1 — stale source SHA in plan header
Plan L7 cites toolkit `352b1adf`; HEAD is `813c8949` (the round-3 fold commit, docs-only). Deps
re-verified accurate at HEAD, so no decay damage. Refresh L7 to `813c8949` and L68 ("SHA refreshed
`352b1adf`") to `813c8949` at the next touch. This is the standard one-commit-behind fold-log decay
seen every round and is **not an R0 blocker** — the brief explicitly anticipated it.

*(Optional, non-blocking polish for the implementing phase, not gate items: (i) add the rounding rule
`b = floor(√K + 0.5)` explicitly to the §3 frozen-constants list — it is currently frozen only in
§4.2/§4.3 bodies; (ii) the implementing phase should pin an integer-only `b` computation, e.g.
`b = isqrt(4K+1).div_ceil/...` or the proven `floor(√K+0.5)` via `isqrt`, and the §7 P4 boundary KAT
already covers `K=240/241`. Both are correctly plan-resolved; exact integer formula is an impl detail,
flagged for P1/P3, not assumed.)*

---

## Plan-resolved-but-impl-detail items (correctly NOT R0 blockers)

Per the brief, these are flagged for the implementing phase, not assumed, and do not hold the gate:
exact CRC-11 polynomial (§4.2 says "CRC-11" without the generator), header-CRC input byte-order, fuzz
harness specifics, and the integer-only `b` computation. All are named as P1–P4/P6 freeze-and-KAT
items in §7; none is silently assumed in a load-bearing claim.

---

## What turns GREEN

Nothing remains to fold for the gate. Both round-3 carry-overs are resolved: NEW-I3 closed (field
dropped, `b` derived, rounding rule proven tie-free and float-stable, boundary KAT present), and the
only open item is the benign one-commit SHA refresh (Nit-1, non-blocking). The math foundation
(field / RS / RAID / CRC-5 / global-tag C1 / fixed-`U` positional ledger / derived `b`) is solid and
machine-verified. The plan faithfully implements the GREEN spec and correctly defers the §12
non-goals.

**The reviewer loop has converged: 2C/3I → 0C/2I → 0C/1I → 0C/0I. This plan-doc is GREEN and clears
the mandatory pre-implementation R0 gate. Implementation may proceed (P0 first), per the phased TDD
plan in §7, with the standard per-phase R0 + post-impl whole-diff review.**
