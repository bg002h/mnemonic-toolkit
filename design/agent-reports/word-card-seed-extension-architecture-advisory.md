# Architecture advisory — Word-Card "seed-phrase parity extension"

- **Type:** opus architect **design advisory** (NOT a formal R0 gate — the spec for this
  extension does not yet exist). Adversarial; load-bearing claims verified independently.
- **Date:** 2026-06-25
- **Author:** architect advisory (single author)
- **Subject:** appending RS "parity words" to a verbatim BIP-39 seed phrase for future
  error recovery, reusing the GREEN Word-Card RS/tag/stop-sign engine.
- **Inputs read:**
  `design/BRAINSTORM_word_card_encoding_2026-06-24.md` (R0-GREEN, round 4),
  `design/IMPLEMENTATION_PLAN_word_card_encoding.md` (plan-R0-GREEN, round 4).
- **Facts verified against:** `bip39 = "2"` → registry crate **2.2.2**
  (`~/.cargo/registry/.../bip39-2.2.2/src/lib.rs`); toolkit lint surface under
  `crates/mnemonic-toolkit/tests/`.

---

## Recommendation (sibling vs SourceKind, one line)

**(a) A separate sibling spec `SPEC_seed_phrase_parity_extension`** — reuse the `wc-codec`
RS/tag/stop-sign *value engine* as a library primitive, but give the seed path its own
secret posture, its own decode/augment surface, **NO RAID**, a larger integrity tag, a
mandatory native-BIP-39-checksum second oracle, and its own R0 loop. Do **NOT** fold a
third `SourceKind::Bip39Seed` into the GREEN public-path spec — the secret-vs-public
posture split is a hard architectural boundary, not a payload variant. **And before any of
that: this feature is of dubious value and partially duplicates `ms1`/codex32 — see Risks.
My honest recommendation is to NOT build it, or to build it only as a strictly
co-located, single-plate, no-distribution "typo insurance" with loud secrecy framing.**

---

## BIP-39 fact-check

Verified against the actual resolved crate `bip39-2.2.2` (the toolkit's `bip39 = "2"`
resolves here, not the vendored copy the brainstorm cites — same facts, noted for the
spec to pin the exact version).

| Claim in the prompt/spec | Verdict | Evidence |
|---|---|---|
| Word counts {12,15,18,21,24} | **TRUE** | `is_invalid_word_count`: `wc < 12 \|\| wc % 3 != 0 \|\| wc > 24` (`lib.rs:682-684`); `MIN_NB_WORDS=12`, `MAX_NB_WORDS=24` (`:83,:86`). The `%3==0 ∧ 12..24` set is exactly {12,15,18,21,24}. |
| 11 bits per word | **TRUE** | `for j in 0..11 { bits[i*11+j] = idx >> (10-j) & 1 }` (`:455-456`); MSB-first. |
| 2048-word English list | **TRUE** | `english.rs` has exactly 2048 entries (`grep -c` = 2048), `abandon`…`zoo`. |
| ENT+CS layout, CS = ENT/32, in the final word | **TRUE** | entropy bytes `=(wc/3)*4` (`:282`), so ENT∈{128,160,192,224,256}; checksum = top `ENT/32` bits of `SHA-256(entropy)` appended after the entropy bits, occupying the tail of the final 11-bit word (`:471-474`). For 12w: 128 ENT + 4 CS = 132 = 12·11. |
| "a BIP-39 word is already an 11-bit GF(2048) symbol, no Layer-A byte-regroup" | **TRUE** | The word index *is* the field element directly (matches the plan's symbol map, §3). The seed path genuinely skips the 8→11 regroup the public path needs. |
| Augment-time "verify BIP-39 checksum, refuse invalid seed" | **CORRECT and necessary** | The native checksum is SHA-256-based (`:471`), only ENT/32 ≈ 4–8 bits, so it catches ~15/16…255/256 of random corruptions — weak, but the only pre-existing guard. Refusing an invalid seed at augment time is the right rule; the tool can only protect *future* damage. **Add:** also refuse the (rare, harmless-looking) case where the input is a valid phrase in a *non-English* wordlist — the symbol map is English-only. |

**No BIP-39 fact in the framing is wrong.** One precision note: the spec/plan should pin
`bip39 2.2.x` explicitly, since the live dep is the registry 2.2.2, not the vendored tree
the brainstorm SHA-cites.

---

## Security crux verification (property B)

**The claim is correct, and stronger than "~": the leak is exact.** I verified it from
first principles (`scratchpad/verify.py`).

- The seed is N symbols over GF(2¹¹). Systematic evaluation-form RS makes each parity word
  a **fixed GF(2¹¹)-linear functional** of the message: `pⱼ = Σᵢ c_{j,i}·msgᵢ`. The parity
  rows of the RS generator are Vandermonde rows ⇒ **any m ≤ N of them are linearly
  independent over GF(q)**.
- An attacker holding the `m` parity words **alone** therefore pins down exactly `m`
  independent linear equations on the N unknowns ⇒ the consistent-seed set is an affine
  subspace of dimension `N − m` ⇒ **residual entropy = (N − m)·11 bits, a drop of exactly
  `m·11` bits.** Not approximate — exact, for any uniform seed.
- Worked: 12-word/128-bit seed, m=8 parity words ⇒ residual `(12−8)·11 = 44` bits ⇒
  GPU-brute-forceable in hours. The prompt's "~8 parity words → brute-forceable" is
  **confirmed** (the framing slightly *under*-counts: residual is 44 bits, even softer than
  "~8·11=88 bits dropped" suggests once you add the tag oracle below).

**"Parity must be as-secret-as-seed / cannot be stored off-site" — CORRECT.** A lone
parity plate is `m` linear combinations of the seed's own symbols; with `m ≥ N/2` it is
already catastrophic, and even small `m` erodes the security margin. Off-site storage of
*any* parity is a partial seed disclosure to that location.

**Is there ANY construction giving error-correction whose redundancy does NOT leak (so it
could be stored apart)? — NO, not without a key. Verified.**

- Information-theoretic core: to *correct* a damaged seed, the redundancy `r=f(seed)` must
  disambiguate among seed-consistent candidates, i.e. it must carry `≥ log₂|candidate set|`
  bits **about** the seed. A redundancy block statistically independent of the seed
  (`I(seed;r)=0`) cannot shift the posterior over seeds at all ⇒ cannot correct a single
  error. "Corrects errors" and "leaks nothing" are **mutually exclusive for any keyless
  scheme.** This is just the contrapositive of Slepian–Wolf/syndrome coding: the minimum
  keyless redundancy equals the error entropy and is by construction a function of the seed.
- The only escape is a **secret key**: `r = MAC_k(seed)` or encrypted parity is
  pseudo-independent of the seed without `k` and useful with it — but then `k` is a *new*
  secret you must back up. You move the secret, you don't remove it.

**Conclusion:** the spec's posture (first-class zeroize/redacting-Debug/off-argv on the
whole seed-parity path, co-located only) is the **only sound** posture. Good — but see the
tag paradox in Risks, which the framing *under*-states.

---

## Search / property-A verification

**Property A ("unknown-position deletion, once located, costs the same 1 parity word as a
known erasure; only decode compute grows ~C(L,d)") is sound** — with three caveats that
must be normative.

- The arithmetic of "unknown→known conversion": once the search fixes a candidate gap-set,
  RS sees a located erasure (cost 1), not an unknown error (cost 2). The parity-cost claim
  is correct. Compute grows as `Σ_{d≤D} C(L,d)`; I tabulated it (`scratchpad/verify.py`):

  | L | D | hypotheses | tag-union FP @t=44 |
  |---|---|---|---|
  | 40 | 2 | 821 | 2⁻³⁴·³ |
  | 40 | 3 | 10,701 | 2⁻³⁰·⁶ |
  | 40 | 4 | 102,091 | 2⁻²⁷·⁴ |
  | 50 | 4 | 251,176 | 2⁻²⁶·¹ |
  | 60 | 6 | 56M | — |
  | 80 | 8 | 32.5B | — |

  For realistic `D ≤ 4` on `L ≈ 40–50` the search is milliseconds-to-seconds — **feasible.**
  But the cost is super-polynomial in D: at D=8/L=80 it is 2³⁵ hypotheses × an RS decode
  each. **A small, frozen, compile-time D cap (D ≤ 3–4) is mandatory** and must **never** be
  a flag the user can raise — otherwise a garbled input is a CPU **DoS** (Risk 4).

- **CAN the search silently return a WRONG, tag-passing seed? YES — and on the seed path
  that is funds-loss, so the bound must be far tighter than the public path's.** Two cases:
  1. **Wrong RS codeword within bounded distance that also passes the t-bit tag.** Per
     candidate `≤2⁻ᵗ`; union over the search `≤ (Σ C(L,d))·2⁻ᵗ`. At `t=44`, D=4, L=50 that
     is `≈ 2⁻²⁶` — **~1 in 67 million.** On the *public* path a miscorrect is caught at
     next use (address mismatch); on the *seed* path a tag-passing wrong seed is a silently
     wrong wallet the user engraves and trusts ⇒ **silent funds loss.** `2⁻²⁶` is **not**
     acceptable for a custody secret. The seed path must use a **larger tag** (`t ≥ 64`,
     i.e. 6 words) to push the union below ~2⁻⁴⁶, *and even then* this enlarges the engraved
     secret + the oracle (Risk 1). Tension is real and unavoidable.
  2. **Transpositions / mixed indel+substitution.** A transposition handled as 2
     substitutions is fine *only* if it stays inside the `2·subs + erasures ≤ m` budget; a
     transposition that straddles a checkpoint boundary perturbs the block-length
     trichotomy (`b∓1`) and can masquerade as an indel, multiplying the candidate space.
     The interaction "deleted-checkpoint + adjacent data deletion" is already a compound
     case the plan handles by whole-block erasure + refuse-on-ambiguity — that discipline
     must extend to seed-path transpositions, with the **refuse path strongly preferred
     over a low-confidence repair** (custody asymmetry: a refused recover is recoverable by
     a human re-read; a wrong-but-confident recover is not).

- **MANDATORY free win the spec/plan omits on the seed path: use the native BIP-39 checksum
  as a SECOND decode oracle.** After RS+tag, the recovered seed *must also* satisfy its own
  BIP-39 checksum (ENT/32 bits, already on the phrase, **secret-free**). This is an
  independent 4–8-bit filter that tightens every union bound above by `2⁻ᴱᴺᵀ/³²` at zero
  footprint cost. It does not replace the larger tag, but it is strictly additive and must
  be normative.

**Net:** property A holds; the search is feasible at small D; **but the silent-wrong-seed
risk forces a larger tag + the native-checksum second oracle + refuse-biased ambiguity
handling** on the seed path. These are seed-specific deltas the public-path engine does
*not* carry — direct evidence for the sibling-spec recommendation.

---

## RAID applicability

**Exclude RAID from the seed path entirely. It is unsound here and duplicative.** Verified.

- RAID's privacy property ("a lone parity plate leaks nothing") holds on the public path
  **only because** there are `n` *independent* xpubs as the `n` message stripes, so `r < n`
  parity = `r` of `n` equations = underdetermined. That structure **does not exist for a
  seed**: there is **one** secret. "Striping" a single seed's symbols and adding XOR/RS
  parity is *exactly* the property-B leak — `r` parity stripes are `r` linear combinations
  of the seed's **own** symbols ⇒ leak `r·11` bits. There is no independent-secret cover to
  hide behind. A lone seed-RAID parity plate leaks; it cannot be distributed.
- The **only** sound way to distribute *one* secret across locations with a privacy
  threshold is a **keyed/randomized threshold scheme** — Shamir / SLIP-39 / codex32. The
  constellation **already has this**: `ms1` *is* codex32 (BIP-93) over the seed entropy,
  with real `k`-of-`n` shares whose lone share leaks nothing. Re-implementing
  availability-RAID on a raw seed would be both **insecure to distribute** and a
  **reinvention of `ms1`'s job**.
- Safe redundancy-distribution option for a single secret = **point users at `ms1`/codex32
  shares**, full stop. The seed-parity extension, if built, must be single-location,
  co-located "typo insurance" only — never a distribution mechanism.

---

## Prior-art & advisability

**Relation to existing schemes:**

- **vs BIP-39 itself:** BIP-39 already carries a (weak, 4–8-bit) checksum. The extension is
  "bolt a strong RS tail onto the existing phrase." Mechanically novel-ish, but see below.
- **vs SLIP-39 (Shamir over a BIP-39-like wordlist):** SLIP-39 *is* the standard answer to
  "I want my seed backup to survive damage/loss across shares," with a real threshold and a
  3-bit-per-word-derived stronger checksum (RS over GF(1024)). SLIP-39 already uses
  **Reed–Solomon over a word field** for its share checksum. The extension's "RS over a
  word field for a seed" is **the same primitive SLIP-39 already standardized** — but
  applied to a *single non-threshold* phrase, which is the part SLIP-39 deliberately does
  **not** do (because non-threshold redundancy leaks, exactly property B).
- **vs codex32 / `ms1`:** codex32 (BIP-93) is RS-over-GF(32) on the seed *entropy*, with a
  long checksum that *does* error-correct, and a threshold-share mode. The constellation
  ships it as `ms1`. The extension overlaps codex32's "error-correctable seed backup" goal
  almost completely; the brainstorm's own §1.1 already excluded `ms1` from Word-Cards
  precisely because **the word view of a seed is the seed phrase, and its
  sharing/correction belongs to codex32.** The seed-parity extension *re-opens exactly the
  door §1.1 closed.*

**Is appended-RS-parity-on-raw-BIP-39 novel? Marginally — and that is a warning, not a
selling point.** The novelty is "keep the phrase verbatim and append leaky parity for
single-copy typo recovery." Nobody standardizes this because:

1. The redundancy leaks the seed (property B), so it can't be distributed — removing the
   main reason you'd want backup redundancy.
2. For single-copy typo insurance, the **dominant real-world failure is total plate
   loss/destruction**, which co-located parity does **nothing** for — you need SLIP-39 /
   codex32 / a second plate.
3. It **trains users to engrave seed-derived secret material in a new place and shape**,
   enlarging the attack surface and the "what is this string?" confusion, for a payoff
   (recover from hand-copy typos on a phrase that *already has a checksum*) that is small.

**Reasons NOT to do it (advisability):** the feature mostly duplicates `ms1`/codex32 for
the one case where redundancy is sound (distribution → use shares), and for the case it
uniquely addresses (single-copy typos) the BIP-39 checksum + a careful re-read already
covers most of it, while the new secret words enlarge the footprint and add an
offline-confirmation oracle. **My recommendation is to decline, or to ship only a loudly
secrecy-framed, single-plate, no-distribution "typo insurance" with the larger tag + native
checksum oracle + refuse-biased decode — and to first ask whether `ms1` already serves the
user's actual need.**

---

## Risks (including ones not in the framing)

1. **The tag is secret-correlated on the seed path — the framing under-states this
   (NEW).** The integrity tag is `SHA-256(entropy)[0..t]`. BIP-39's own checksum is *also*
   the top bits of `SHA-256(entropy)`, so the first 4–8 of the tag's `t` bits **are** the
   native checksum (redundant), and the rest are **new** leaked bits of a one-way commitment
   to the entropy. By itself a `t`-bit one-way commitment doesn't reveal entropy bits — but
   it is a `t`-bit **offline confirmation oracle**: combined with the property-B parity leak
   (residual 44 bits at m=8), the tag *confirms the unique survivor* of the residual space,
   collapsing brute-force verification. **The tag must be inside the secret boundary** (it
   is, if the whole path is) **and the seed-path tag is NOT free public metadata** — every
   extra tag bit both improves miscorrection safety *and* strengthens the oracle. The `t≥64`
   needed for funds-safety (search section) directly worsens the oracle. This trade-off must
   be stated explicitly in the spec; the public-path "tag is just a public checksum"
   framing does not carry over.

2. **Enlarged engraved secret footprint (NEW).** A 12-word seed gains `m` parity words +
   ~6 tag words + checkpoints/header/stop-sign ⇒ a 12-word secret balloons to ~30–50
   *engraved* words, **all of which are now secret** (property B). That is more steel
   surface holding seed-correlated material, more transcription to get wrong, and more for
   a thief to photograph. The "small append" framing hides a 2.5–4× growth of the secret
   footprint.

3. **Silent-wrong-seed on miscorrection = funds loss (verified, §search).** Requires a
   larger tag + native-checksum second oracle + refuse-biased ambiguity handling. The
   public path's "miscorrect caught at next use" safety net does **not** exist for a seed.

4. **Search DoS (verified).** `Σ C(L,d)` is super-polynomial in D; an adversarial/garbled
   input at a high D cap is a CPU exhaustion vector. D must be a small frozen constant,
   never user-raisable, with a hard hypothesis-count ceiling → refuse.

5. **Lint/test blast radius is large and seed-specific (NEW — verified against the live
   lint surface).** The toolkit already enforces a serious secret-hygiene gate that a seed
   path *will* trip:
   - `tests/lint_argv_secret_flags.rs` is a **completeness closure** that set-equals the
     secret-argv route set against live `gui-schema` + `src/secret_taxonomy.rs`
     (`SECRET_NODE_TYPES_ARGV`, `SECRET_SLOT_SUBKEYS`). A new `augment`/`recover`
     subcommand that ingests a seed is a **new secret-argv route** — the closure will
     **fail until the seed-input flag is enrolled** with a non-argv (`*-stdin` / `@env:` /
     refusal) channel anchor. This is the *opposite* of the public path, whose plan §8
     explicitly says "no secret material ⇒ no zeroize requirement" — that sentence is
     **false for the seed path** and would make the lint RED.
   - `tests/lint_zeroize_discipline.rs` enumerates every owned-secret site with a
     `Zeroizing`/`SecretString` evidence anchor. Any seed bytes the codec touches become
     new rows. **Implication:** if `wc-codec` stays "codec-AGNOSTIC, no zeroize" (plan §2),
     it **must not see seed bytes** — the seed must be zeroize-wrapped *before* entering the
     engine and the engine must operate on already-wrapped buffers, or `wc-codec` itself
     must take on the first-class hygiene bar. Either way this is a **structural decision
     the public-path plan explicitly punted** ("zeroize not required") and cannot be
     inherited.
   - Also implicated: `cli_secret_in_argv_warning.rs`, `cli_argv_leakage.rs`,
     `lint_world_readable_helper.rs` — all keyed on the secret-flag taxonomy.

6. **`wc-codec`'s "no-zeroize, codec-agnostic" identity is incompatible with a seed payload
   (NEW).** Plan §2/§6.1/§8 thrice assert `wc-codec` handles "no secret material" and needs
   no `zeroize`. A `SourceKind::Bip39Seed` would **invalidate that invariant** and force a
   zeroize refactor of the shared engine — a re-open of the public plan's settled posture.
   This is the single strongest reason the seed path must be a **sibling that consumes the
   value engine on already-zeroized buffers**, not a third SourceKind in the same crate.

7. **User confusion / footgun (NEW).** Two near-identical word strings — a real seed phrase
   and a "seed + parity" string — that *look* alike but have wildly different secrecy and
   wallet-derivation semantics. A user who imports the augmented string into a stock BIP-39
   wallet gets a wrong/invalid seed; one who treats parity words as "extra seed words" is
   catastrophically confused. The verbatim-prefix property (a selling point) is *also* the
   footgun (the augmented string is indistinguishable at a glance from a phrase).

---

## Decision rationale (sibling spec vs third SourceKind)

**Recommend (a) sibling spec `SPEC_seed_phrase_parity_extension`, consuming `wc-codec`'s
value engine — if the feature is built at all.**

- **Secret-vs-public posture is a hard boundary, not a payload variant.** The public plan
  *thrice* commits to "no secret material ⇒ no zeroize" and builds `wc-codec` as a
  zeroize-free, codec-agnostic library. The seed path is the exact opposite: property-B
  proves the parity *is* the seed, the tag is secret-correlated (Risk 1), and the live
  toolkit lints (Risk 5/6) will force enrollment in the argv-secret closure + zeroize
  discipline. Folding `Bip39Seed` into the same SourceKind **invalidates the public crate's
  settled invariant** and drags the entire public spec/plan back through R0 over a
  posture-flip delta — large blast radius, high regression risk to a just-converged
  artifact.
- **Code reuse is preserved without coupling.** The *value layer* (eval-form RS over
  GF(2¹¹), the non-linear SHA tag, the stop-sign, the bounded indel search) is genuinely
  reusable. A sibling spec can depend on `wc-codec` as a **pure-function library** (encode
  parity over a `&[symbol]`, decode/search over `&[symbol]`), passing **already-zeroize-
  wrapped** seed buffers in and out, so the engine never owns plaintext secret allocations
  and keeps its no-zeroize identity. The seed-specific deltas — larger tag (`t≥64`),
  mandatory native-BIP-39-checksum second oracle, refuse-biased ambiguity, **no RAID**,
  augment-time checksum/English-wordlist gate, the secret posture, and the
  argv/zeroize/redacting-Debug bar — all live in the sibling, where they belong.
- **R0 blast radius:** sibling = one fresh R0 loop over a self-contained delta; SourceKind =
  re-open two GREEN artifacts plus the crate-posture invariant. Sibling is strictly smaller
  and isolates the funds-safety-critical seed logic for focused adversarial review.
- **Long-term maintainability:** the constellation's pattern is one concern per artifact
  (`md`/`mk`/`ms` codecs; public Word-Cards). A seed-secret availability layer is a
  distinct concern with a distinct threat model; a sibling keeps the "public availability
  coding" and "secret typo-insurance" stories from contaminating each other's invariants
  and lints.

**But the prior gate is advisability (Prior-art):** the feature substantially duplicates
`ms1`/codex32 (the sound, threshold, distributable answer) and BIP-39's own checksum (the
typo answer), while uniquely adding only co-located single-copy typo insurance at the cost
of a 2.5–4× larger, fully-secret engraved footprint and a new offline-confirmation oracle.
**Recommend declining, or scoping to a loudly-framed single-plate no-distribution typo
insurance only, and first confirming `ms1` doesn't already serve the user's real need.** If
it *is* built, build it as the sibling spec, never the third SourceKind.
