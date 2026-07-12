# VERDICT — `repair-corrupted-pathless-card-partial`

**Disposition: (A) WONTFIX-AS-CARD.** Keep `repair` STRICT on dead cards; do NOT open a partial-repair R0 cycle.

- **Type:** read-only funds-safety analysis + reasoned verdict for the toolkit FOLLOWUP
  `repair-corrupted-pathless-card-partial` (`design/FOLLOWUPS.md:37-39`).
- **Source SHA:** `a528eba561d47a7f9ceb34b2523e3a74e04ab117` (master; `mnemonic-toolkit-v0.88.0` +
  the pinned-rustfmt fmt commit). All line numbers below are live against this SHA.
- **Scope:** `crates/mnemonic-toolkit/src/repair.rs`, `crates/mnemonic-toolkit/src/cmd/repair.rs`,
  `crates/mnemonic-toolkit/src/error.rs`, and the vendored `vendor/md-codec/src/{chunk,decode,error}.rs`.
- **No source was modified.** This document is the only artifact produced.

---

## 1. What happens today (recon — grep-verified current line numbers)

### 1.1 The md1 repair path is STRICT-decode-gated end to end

`repair_card(CardKind::Md1, chunks)` (`repair.rs:1144`, Md1 arm `repair.rs:1232-1250`) pre-gates
each chunk through `parse_chunk` (BCH structural/length checks, long-code reject) and then
delegates the whole set atomically to `repair_via_md_codec(chunks)` (`repair.rs:1249`).

`repair_via_md_codec` (`repair.rs:1638`) calls `md_codec::decode_with_correction(&refs)`
(`repair.rs:1641`). That sibling entry point (`vendor/md-codec/src/chunk.rs:531`) does two things in
order:

1. **BCH-corrects each chunk** to a valid codeword (`chunk.rs:564-629`) — up to `t = 4`
   substitutions, with a defensive residue re-verify (`chunk.rs:613-623`). This is the
   funds-critical correction step: a >4-error pattern can alias to a *different* valid codeword
   (the F4 class).
2. **Then runs a STRICT full decode** of the corrected bytes:
   - single non-chunked string → `crate::decode::decode_md1_string(&corrected_strings[0])`
     (`chunk.rs:657`), which is `decode_md1_string_with_opts(s, DecodeOpts::default())`
     (`vendor/md-codec/src/decode.rs:179`) — i.e. `allow_unresolved_origin: false`;
   - chunked (multi-chunk or chunked-of-1) → `reassemble(&corrected_refs)` (`chunk.rs:665`),
     which is `reassemble_with_opts(strings, DecodeOpts::default())` (`chunk.rs:312`) — also strict.

`decode_with_correction` is deliberately given **no** partial variant. The partial-decode cycle
(v0.88.0) added `DecodeOpts::partial()` (`decode.rs:55`) and threaded it through
`decode_payload` / `decode_md1_string` / `reassemble`, but the SPEC per-command table explicitly
carves repair out: *"`decode_with_correction` (repair) is NOT given a partial variant (repair stays
strict)"* (`design/SPEC_pathless_partial_decode.md:26`, and the per-command row `:39`).

### 1.2 A dead card fails the strict post-correction decode → exit 2

A **dead card** = an md1 whose `canonical_origin(tree)` is `None` **and** whose per-`@N` origin is
elided-and-unresolvable (tr+tree, `sh(sortedmulti)`, `sh(multi)`, bare `wsh`, raw miniscript). The
template is fully on the card; only the derivation ORIGIN is absent with no canonical default. Under
a strict decode it hits `Error::MissingExplicitOrigin` (`vendor/md-codec/src/validate.rs:~221-246`,
routed through `decode_payload`).

Tracing that error back through `repair_via_md_codec`: `MissingExplicitOrigin` is not
`TooManyErrors` / `ChunkSetEmpty` / `Codex32DecodeError`, so it falls to the
`Err(other) => Err(RepairError::PostCorrectionDecodeFailed { chunk_index: None, detail })` arm
(`repair.rs:1710-1713`). `PostCorrectionDecodeFailed` → `ToolkitError::Repair(_)` (`error.rs:420-423`)
→ `exit_code() == 2` (`error.rs:643`). (For completeness: had md-codec's own
`md_codec_exit_code` mapped it, `MissingExplicitOrigin` is also exit 2 — `error.rs:550` — so the
class is consistent either way.)

**Therefore, TODAY:**

| Input | Path | Result |
|---|---|---|
| **Intact** dead card | residue==0, no correction, strict decode → `MissingExplicitOrigin` | **exit 2** (un-repairable) |
| **Corrupted** dead card (1–4 sub errors + elided-unresolvable origin) | BCH-corrects internally, then strict decode → `MissingExplicitOrigin` | **exit 2**; the computed correction is **discarded** (atomic, nothing returned) |

The corrupted case is the FOLLOWUP's subject: the toolkit *does* compute a BCH correction but then
prunes it fail-closed because the corrected card still can't strict-decode. Nothing is blessed,
nothing leaked. This matches the SPEC's stated repair contract verbatim (`SPEC:39`, `:63`, `:80`).

### 1.3 Auto-repair and `--max-indel` compose the same way (also fail-closed)

- **Auto-repair** (`try_repair_and_short_circuit`, `repair.rs:1786`): `repair_card` returning `Err`
  → `return Ok(())` fall-through (`repair.rs:1795`); the caller's original typed error surfaces.
  A dead card is never auto-repaired. (And per SPEC `:63`, an *intact* dead card in
  `verify-bundle`/`inspect` now partial-decodes rather than decode-errs, so the `is_err()`
  auto-repair trigger doesn't even fire.)
- **`--max-indel ≥ 1`**: `PostCorrectionDecodeFailed` IS an indel trigger (`repair.rs:1523-1535`),
  so a corrupted dead card enters `recover_indel_card` (`cmd/repair.rs:178-233`). The md1 indel
  oracle verifies candidates through the **strict** `md_codec::chunk::reassemble` — so a dead-card
  candidate fails the oracle → `IndelOutcome::Unrecoverable` → `IndelUnrecoverable` → exit 2. Still
  fail-closed.

### 1.4 The oracle geometry that makes the corrupted-dead-card case special

Two independent oracles could catch a BCH miscorrection; a dead card defeats both for its
dominant shape:

1. **Cross-chunk content-id oracle** — `chunk.rs:406-415` (`compute_md1_encoding_id` →
   `derive_chunk_set_id` → `ChunkSetIdMismatch`). This is UNCONDITIONAL even under partial opts
   (`chunk.rs:321-327`, `:406-407`; SPEC funds-load-bearing invariant `:27`). **BUT it only runs in
   the `reassemble` path — i.e. for CHUNKED cards.** A single **non-chunked** md1 routes via
   `decode_md1_string` (`chunk.rs:657`), which SKIPS reassemble entirely → **no content-id oracle**.
   This is exactly the gap the v0.86.0 non-chunked demote (`is_non_chunked_md1`, `repair.rs:736`;
   demote block `repair.rs:1660-1676`) already documents.
2. **Derivable-wallet cross-check** — the user re-derives an address/xpub from the recovered card
   and confirms it controls funds (the standard "verify a corrected card" remediation, e.g. the
   ms1/md1 Unverified advisories `repair.rs:1219-1222`, `:1667-1672`). **A dead card has no derivable
   wallet** — the origin is unspecified, so no address can be derived at all.

The dominant dead-card shape (a pathless single-string md1) is **non-chunked**, so it has neither
oracle.

---

## 2. Funds / value tradeoff analysis

### 2.1 Co-occurrence is vanishingly rare — and inversely correlated with safety

The two conditions must both hold: **(a)** a dead-card shape (pathless / non-canonical, elided
unresolvable origin — a shape the toolkit's own `bundle` never mints, since it builds origins fresh,
SPEC `:42`) AND **(b)** BCH-recoverable corruption (1–4 substitution errors confined to one chunk).
Each is already a minority; their intersection is a corner of a corner.

Worse, the intersection splits by shape in the *wrong* direction for value:

- **Chunked dead card** (multi-chunk or chunked-of-1): the content-id oracle (`chunk.rs:406-415`)
  runs even under partial decode, so a miscorrection would be *caught*. This is the only subset
  where partial-repair could be safe — and it is the **rarest** sub-sub-case (dead AND chunked AND
  corrupted). Even here the payoff is thin: the product is a template with an unspecified origin, so
  the user *still* cannot derive a wallet from it; it is only useful if they can supply the origin
  out-of-band on restore — the same recoverable state partial-DECODE already gives for an *intact*
  card, now with an added "do you trust the BCH guess?" question.
- **Non-chunked single-string dead card** (the DOMINANT dead-card shape): no content-id oracle AND
  no derivable wallet. This is the **common** case and the one where partial-repair adds pure risk.

So the subset where partial-repair helps is the rarest; the subset where it hurts is the common one.

### 2.2 What partial-repair would actually PRODUCE

To make repair emit anything for a dead card you would swap the internal strict decode for a
partial one (a `decode_with_correction_with_opts { allow_unresolved_origin: true }`). The output
would be: the BCH-corrected chunk(s) + a template rendered with `origin: «unspecified»`, surfaced as
an **exit-4 VERIFY-ME candidate**.

For the non-chunked dominant shape that candidate is **un-verifiable by construction**:

- the correction could be a >4-error alias to a DIFFERENT valid codeword (F4;
  `bch-repair-miscorrection-set-level-reverify`);
- there is no content-id oracle to catch it (non-chunked bypass, `chunk.rs:657`);
- there is no derivable wallet to sanity-check it (origin unspecified);
- the standard remediation the advisory would print ("re-derive the wallet/address and confirm it
  controls your funds") is **impossible to follow** on a dead card — there is no address to derive.

So the "candidate" would be a corrected card the user has literally no way to validate — an exit-4
that dresses up an un-checkable guess as a recoverable result.

### 2.3 Does partial-repair make the miscorrection surface WORSE? — YES, for the common shape

`bch-repair-miscorrection-set-level-reverify` (F4, `FOLLOWUPS.md:41-46`) established that BCH beyond
`t = 4` can alias to a different valid codeword, and the whole constellation response (v0.80.0/v0.81.0
+ v0.86.0) was to make every correction that spends the checksum's error-detection budget WITHOUT a
surviving oracle either a hard `Reject` (mk1 complete-set mismatch, `repair.rs:1107-1112`) or an
`Unverified`/exit-4 demote (ms1 always; md1 non-chunked). A dead card is the extreme point of that
program: it has **neither** the mk1 set oracle **nor** the md1 content-id oracle (non-chunked) **nor**
the universal derivable-wallet backstop. Partial-repair would therefore convert today's honest
fail-closed **exit-2 prune** into an **exit-4 blessing of an unvalidatable correction** — strictly
*enlarging* the miscorrection/misrepresentation surface on precisely the class of card where the user
has the LEAST ability to notice a wrong result. It only shrinks the surface for the rarest (chunked)
subset, where the oracle already fires and the current strict prune loses only a marginal,
still-origin-dead recovery.

### 2.4 The guiding principle — the funds bound argues for staying strict

Standing principle: *"maximally expressive on output, permissive on input — Postel's law — BOUNDED
by never silently misrepresent; the fail-closed REJECT stays the FLOOR until the wire can hold the
input faithfully"* (`feedback_guiding_principle_expressive_output_permissive_input`).

Partial-**decode** (v0.88.0) is the principle's clean embodiment: it renders a template that is
*provably, byte-for-byte on the card* and only marks the elided origin unspecified — nothing is
invented, no checksum budget is spent, zero funds risk. Partial-**repair** is categorically
different: it **invents** chunk content via BCH correction, and for the dominant dead-card shape
there is no oracle to validate the invention. Blessing that as an exit-4 candidate the user cannot
check is exactly the "silently misrepresent" outcome the bound forbids — the advisory is technically
"loud," but its remediation is un-followable, so it functionally launders a guess into a "recovered"
card. This is the direct analog of the `descriptor-use-site-collapse` / `concrete-nonranged-xpub`
precedent the memory calls out: the fail-closed reject/prune is the correct floor until there is a
faithful representation (here, a surviving oracle) — which for a non-chunked dead card does not
exist. The bound points at **strict / wontfix**, not partial-repair.

---

## 3. Verdict and rationale

**(A) WONTFIX-AS-CARD.** The two triggering conditions co-occur only in a corner of a corner; the
common (non-chunked) dead-card shape has no content-id oracle and no derivable wallet, so any
partial-repair output is an unvalidatable exit-4 candidate whose only offered remediation is
impossible to follow — strictly enlarging the F4 miscorrection surface on the least-checkable class
of card. The narrow safe subset (chunked dead card, content-id oracle intact) is the rarest
sub-sub-case and yields only a still-origin-dead template the user already gets from partial-DECODE
of an intact card. The value does not justify a funds-critical R0 cycle, and the funds bound (never
bless a wrong wallet) affirmatively argues for keeping the strict exit-2 prune. Repair staying strict
is the CORRECT, honest, fail-closed disposition — not a gap.

**What the user loses by wontfix (stated honestly):** a user holding a *corrupted* dead card cannot
recover it via `mnemonic repair` (exit 2, "un-repairable"). But for the dominant non-chunked shape
there is no *safe* recovery to offer — any BCH correction is un-validatable, so returning it would be
blessing a guess. Exit 2 is the truthful answer. The only case with a real (if thin) safe recovery is
the chunked dead card; that alone does not warrant the cycle.

**Reopen criteria (documented so a future reader can trust the wontfix):** revisit only if (a) a
concrete user surfaces a *chunked* dead card needing recovery, AND (b) md-codec grows a
`decode_with_correction_with_opts` whose partial path keeps the content-id oracle unconditional and
demotes every non-chunked/oracle-less correction to a plainly-un-followable-advisory exit-4 (or
reject). Absent both, strict stays.

### If (B) were ever pursued — contract sketch (recorded, NOT recommended)

For completeness only. A future cycle would: add `md_codec::decode_with_correction_with_opts(opts)`;
in `repair_via_md_codec` opt into `DecodeOpts::partial()`; compose with the v0.86.0 non-chunked
demote (`repair.rs:1660-1676`) so a non-chunked dead-card correction is `Unverified`/exit-4 with an
HONEST advisory that says the card cannot be verified (no address to derive); keep the content-id
oracle unconditional (`chunk.rs:406-415`) so chunked dead cards get real protection; keep
`MissingExplicitOrigin`-swallow scoped so `EmptyOriginOverride` and every non-origin reject stay
fatal-in-partial (`decode.rs:146`, `error.rs:558`); and RED-prove that a non-chunked dead-card >4-error
miscorrection never surfaces as anything a user could mistake for a confirmed recovery. That is a
full funds-critical R0 pipeline (SPEC + plan + per-phase + whole-diff) for value that reduces to "the
rarest chunked dead card gets an exit-4 candidate it still can't fully use." The cost/value/risk math
is why the recommendation is (A).

---

## 4. FOLLOWUPS.md status line to apply (exact text)

Append this `**Status:**` line to the `repair-corrupted-pathless-card-partial` entry
(`design/FOLLOWUPS.md`, immediately after the existing bullet at `:39`):

```
- **Status:** ✓ **WONTFIX-AS-CARD (2026-07-11, `design/VERDICT_repair_corrupted_pathless_card.md`, SHA a528eba5).** Repair STAYS STRICT on dead cards by design — NOT a gap. Recon: a corrupted dead card is already fully fail-closed today — `repair_via_md_codec` (`repair.rs:1638`) BCH-corrects internally then runs md-codec's STRICT `decode_with_correction` (`repair.rs:1641`; `vendor/md-codec/src/chunk.rs:531`, whose post-correction decode is `DecodeOpts::default()` at `chunk.rs:657`/`:665`), which hits `MissingExplicitOrigin` → `PostCorrectionDecodeFailed` → `ToolkitError::Repair` → exit 2 (`repair.rs:1710-1713`, `error.rs:643`); the computed correction is discarded (atomic). Intact AND corrupted dead cards both → exit 2 (auto-repair falls through `repair.rs:1795`; `--max-indel` fails the strict reassemble oracle → exit 2). Funds analysis: partial-repair would ENLARGE the F4 miscorrection surface for the DOMINANT (non-chunked, single-string) dead-card shape — it has NO content-id oracle (the non-chunked bypass `chunk.rs:657` skips the unconditional `chunk.rs:406-415` check) AND no derivable wallet (origin unspecified), so an exit-4 candidate would be un-verifiable by construction and its standard "re-derive the address" remediation is impossible to follow. Only the rarest subset (chunked dead card, content-id oracle intact) could partial-repair safely, and it yields only a still-origin-dead template partial-DECODE already provides for an intact card. Funds bound ("never bless a wrong wallet; fail-closed reject stays the floor until a faithful representation exists") + the F4 program (`bch-repair-miscorrection-set-level-reverify`) both point at strict. **Reopen only if** a concrete CHUNKED dead-card recovery need appears AND md-codec grows a `decode_with_correction_with_opts` that keeps the content-id oracle unconditional and demotes every oracle-less correction. **Severity:** LOW. **Tier:** toolkit.
```
