# cycle-prep recon — 2026-07-07 — F4 (BCH "repair" blesses wrong-fit miscorrections)

**Source finding:** `design/agent-reports/constellation-eval-2026-07-06.md:101-124` (+ remediation
line 308-310), constellation-eval 2026-07-06.

## Per-repo sync state

| Repo | Default branch | origin SHA | Local branch | Ahead/behind | Untracked (code-relevant?) |
|---|---|---|---|---|---|
| `mnemonic-toolkit` | master | `9866acc7` (tag `mnemonic-toolkit-v0.79.0`) | master | 0/0 | many `cycle-prep-recon-*.md` / `design/*` scratch docs — no source |
| `mnemonic-key` (mk) | main | `85bca69` | main | 0/0 | none |
| `descriptor-mnemonic` (md) | main | `ef1f3e71` | main | 0/0 | none |
| `mnemonic-secret` (ms) | master | `c2fd4eb` | master | 0/0 | `.claude/worktrees/repro-p3b-ms/`, 3 scratch `cycle-prep-recon-*.md`/`design/*` — no source |

All four repos are byte-identical to their remote default branch at recon time. **Note:** the
session's carried-over `MEMORY.md` states the last-shipped toolkit tag was `v0.76.0` — that is
**stale**; `origin/master` is actually at `v0.79.0` (`9866acc7`, "reject concrete non-ranged xpub"
— Cycle D, unrelated to F4). This recon cites current source, not the memory snapshot.

---

## Per-citation verification

### 1. `mk-cli/src/cmd/repair.rs:63-90` — no set-level re-verify

**ACCURATE — verbatim structural match.** Current file, lines 63-90:

```rust
for (idx, original) in strings.iter().enumerate() {
    let decoded = mk_codec::string_layer::decode_string(original)?;
    let (corrected_chunk, corrected_positions) = reconstruct_corrected(original, &decoded);
    reports.push(RepairDetail { ... });
    corrected_chunks.push(corrected_chunk);
}
let any_correction = reports.iter().any(|r| !r.corrected_positions.is_empty());
...
Ok(if any_correction { 5 } else { 0 })
```

Confirmed: the loop calls `mk_codec::string_layer::decode_string` **per string** (BCH-corrects
each chunk independently) and returns exit 5 the moment any chunk needed correction. There is
**no call anywhere in this file** to `mk_codec::string_layer::reassemble_from_chunks` or any
cross-chunk-hash check. Grep confirms: `reassemble` appears nowhere in `mk-cli/src/`. By contrast,
the *normal* (non-repair) `mk decode` command (`mk-cli/src/cmd/decode.rs:27`, `mk_codec::decode`)
routes chunked input through `string_layer::pipeline::decode` (`pipeline.rs:150`,
`reassemble_from_chunks(chunks)?`), which enforces the cross-chunk SHA-256 hash
(`chunk.rs:105-106`, `Error::CrossChunkHashMismatch`). **`mk repair` is a strictly weaker check
than `mk decode`** — repair explicitly bypasses the net decode already has.

### 2. mk-codec `bch.rs:442` / `:495` — `bch_verify_*` passes for ANY valid codeword

**ACCURATE — exact line match, no drift.** `crates/mk-codec/src/string_layer/bch.rs`:

- Line 442: `if bch_verify_regular(hrp, &corrected) {` (inside `bch_correct_regular`, the
  "Defensive: re-verify. Catches the 5+-error edge case" step, line 441 comment).
- Line 495: `if bch_verify_long(hrp, &corrected) {` (identical pattern inside `bch_correct_long`).

`bch_verify_regular`/`bch_verify_long` (lines 322-329, 353-360) compute the BCH polymod residue
over `hrp_expand(hrp) || data_with_checksum` and return `true` iff it equals the target constant
(`MK_REGULAR_CONST`/`MK_LONG_CONST`). This is true for **any** valid codeword — the function has
no way to distinguish "the original codeword before corruption" from "a different codeword the
bounded-distance decoder aliased onto." Confirmed by construction: this check is a checksum
membership test, not an identity test.

### 3. toolkit `src/repair.rs` mk1 arm — same skip; auto-repair fires by default

**ACCURATE, with a materially useful refinement found in the same file.**

- `repair_card`'s `CardKind::Mk1` branch (`src/repair.rs:766-783`) loops
  `repair_chunk_one(kind, i, chunk)` per chunk and assembles `corrected_chunks` directly — **no
  call to `mk_codec::decode(&refs)` or any reassembly**. `repair_chunk_one`'s own defensive
  re-verify (`src/repair.rs:731-739`) is `polymod_residue(...) != 0` — i.e. the exact same
  "is-it-*a*-valid-codeword" check as citation 2, not a "is-it-*the*-original" check.
- **Found in the same file — a ready-made fix template already exists.** The toolkit's *indel*
  recovery path already does what F4 asks for: `Mk1IndelOracle::validate`
  (`src/repair.rs:1036-1056`) calls `mk_codec::decode(&refs)` (full multi-chunk reassembly +
  cross-chunk-hash) on the candidate-corrected chunk set and only accepts the candidate if
  `decode` succeeds. `Md1IndelOracle` (`:1063-1083`) does the analogous thing via
  `md_codec::chunk::reassemble`. **This means the substitution-repair path (the one with the F4
  gap) and the indel-repair path (which already has the fix) sit side-by-side in the same file** —
  the fix is to route the substitution path's already-in-hand `corrected_chunks` through the same
  reassembly call the indel oracles already use, not to invent a new mechanism.
- Auto-repair default: `resolve_no_auto_repair` (`src/repair.rs:411-418`) returns
  `no_auto_repair || !tty` — i.e. auto-repair fires **by default whenever stdout is a TTY** (unless
  `--no-auto-repair` is passed), and is off-by-default under a pipe/redirect/CI. `convert.rs:790-1051`
  and `inspect.rs:93-133` both wire `effective_no_auto_repair` from this resolver and call
  `crate::repair::try_repair_and_short_circuit` unconditionally when it's false. **Refinement over
  the finding's phrasing:** "fires by default" is accurate but TTY-conditional, not unconditional —
  worth stating precisely in the fix spec (matches the manual's own documented behavior, see below).
  `verify_bundle.rs` (6 call sites) and `xpub_search/seed_intake.rs:184` also call the same
  short-circuit helper.

### 4. ms1 single-string path — no cross-chunk hash net, no payload-derived oracle

**ACCURATE — confirmed absent, worst case in the cluster.**

- ms1 is single-string per codex32 spec (`ms-cli/src/cmd/repair.rs:9`, "ms1 is single-chunk per
  codex32 spec"); there is no second string to cross-check against.
- `ms_codec::decode_with_correction` (`ms-codec/src/decode.rs:242-298`) BCH-corrects, re-verifies
  the *codeword* (`decode.rs:285-292`, same class of check as citation 2), then calls `decode(s)`
  (`decode.rs:44` in the underlying non-correcting entry point) which layers only: string-length
  membership in a small allowed set, tag-alphabet/reserved-tag checks (`§4` rules 6/7/9). None of
  these validate the **entropy payload itself** — `Payload::Entr(Vec<u8>)` is raw BIP-39 entropy
  with no internal redundancy of its own. A miscorrected-but-valid ms1 codeword decodes to a
  plausible `(Tag::Entr, Payload)` with **no downstream signal that it's wrong**.
- `ms repair` (`ms-cli/src/cmd/repair.rs:89`) is a thin wrapper over `decode_with_correction` —
  confirms the same no-second-oracle gap at the CLI layer.
- Toolkit auto-repair on ms1 intake: `repair_card`'s `CardKind::Ms1` branch
  (`src/repair.rs:784-818`) dispatches through the same `try_repair_and_short_circuit` /
  `resolve_no_auto_repair` machinery as mk1/md1 (kind-agnostic caller in `convert.rs`/`inspect.rs`)
  — **ms1 auto-repairs by default under a TTY, identically to mk1**. This is the finding's
  "highest-consequence" claim, confirmed: wrong-fit ms1 repair → silently wrong seed, auto-fired,
  with the only textual signal a Zeroizing-secret advisory (`OutputClass::PrivateKeyMaterial`) that
  says nothing about correction confidence.

### 5. `GEN_REGULAR` shared across the 3 codecs

**ACCURATE — byte-identical across all three codecs AND the BIP-93 primary source.**

| Source | `GEN_REGULAR` (5×u128) |
|---|---|
| mk-codec `string_layer/bch.rs:173-179` | `0x19dc500ce73fde210, 0x1bfae00def77fe529, 0x1fbd920fffe7bee52, 0x1739640bdeee3fdad, 0x07729a039cfc75f5a` |
| md-codec `bch.rs:7-13` | identical |
| ms-codec `bch.rs:23-29` | identical |
| BIP-93 (`bitcoin/bips` master, `bip-0093.mediawiki`, `ms32_polymod` GEN array) | identical |

Also cross-checked: mk1's `MK_REGULAR_CONST`/ms1's `MS_REGULAR_CONST = 0x10ce0795c2fd1e62a`
("SECRETSHARE32", `ms-codec/bch.rs` doc comment) — the ms1 target matches BIP-93's stated
`MS32_CONST = 0x10ce0795c2fd1e62a` exactly (md1/mk1 use NUMS-derived per-HRP targets that
deliberately differ from BIP-93's, by design — see mk-codec `bch.rs:181-197` doc comment and the
repo's pinned `reference_mformat_bch_residue_architecture` memory card; this is intentional
domain-separation, not drift). Since the generator polynomial (which determines the
codeword-difference structure the miscorrection analysis depends on) is identical across all
three codecs, **the "rate transfers to md1/ms1" claim is mechanically sound for the *raw BCH
math*** — though (per the primary-source section below) the *downstream* consequence differs
sharply per codec because md1 has an independent content-derived second oracle that mk1/ms1 lack.

---

## Primary-source crypto verification (BIP-93 / codex32)

Fetched `bitcoin/bips` master `bip-0093.mediawiki` directly (not the eval report, not repo doc
comments). Confirmed:

- **Regular code (13-char checksum, ≤93-symbol data-part):** "guarantees detection of any error
  affecting at most 8 characters" and "the 13 character checksum is adequate to correct 4 errors
  in up to 93 characters." This is the `t = 4` correction bound the codebase implements
  (`bch_correct_regular` doc, mk-codec `bch.rs:383-396`). The eval's phrase **"designed distance
  9"** is a reasonable restatement (a code that guarantees detecting `d-1` errors has minimum
  distance `≥ d`; BIP-93's "detects ≤8" ⇒ distance ≥9) but is **not verbatim BIP-93 language** —
  the BIP never uses the phrase "distance 9." Tag: **ACCURATE (restated, not verbatim)**.
- **Long code (15-char checksum, 96-108 symbols):** "can correct up to 4 character substitutions"
  — same `t=4` bound, confirmed for the long code too (matches mk-codec `bch_correct_long` doc).
- **GEN arrays:** BIP-93's `GEN` (short) and `GEN` (long, `ms32_long_polymod`) match all three
  codecs byte-for-byte (see table above). `MS32_CONST = 0x10ce0795c2fd1e62a` matches ms-codec's
  `MS_REGULAR_CONST` exactly.
- **"Beyond-t aliasing to a different valid codeword" — NOT explicitly stated by BIP-93 as a
  named risk, but implicitly acknowledged and consistent with standard bounded-distance-decoder
  theory.** BIP-93 does NOT contain language like "a >4-error input may decode to a different valid
  string." What it DOES contain, directly on point: **"implementations SHOULD NOT automatically
  proceed with a corrected codex32 string without user confirmation."** This is a normative BIP-93
  admonition against exactly the failure mode the toolkit's default (TTY) auto-repair commits —
  auto-firing without any confirmation step, on mk1 and ms1 alike. This BIP-93 clause is
  independent, primary-source support for the F4 fix direction that is *stronger* than the eval
  report's own framing (the eval didn't cite it). The "aliasing to a different codeword beyond t"
  property itself is not spec text but a standard fact about bounded-distance decoding for any
  block code with minimum distance `d`: patterns of weight `> t = ⌊(d-1)/2⌋` can fall within the
  radius-`t` ball of a *different* codeword. The codebase's own comments already acknowledge this
  exact category (mk-codec `bch.rs:441` "Defensive: re-verify. Catches the 5+-error edge case";
  `:450-452` "more than 4 substitutions or pathological pattern") — i.e., the original implementers
  already knew a >t input could produce a spuriously-valid result and added the codeword-membership
  re-verify for it, but that re-verify is (as citation 2 shows) insufficient to catch a
  miscorrection landing on a genuinely different valid codeword. **Verdict: the underlying
  mechanism the finding describes is real and well-founded; it is not a BIP-93-stated fact but a
  standard, uncontroversial coding-theory consequence of the stated `t=4`/distance-9 parameters,
  and the code's own comments show the implementers already anticipated the general shape of the
  risk (just didn't close the gap with a set-level/second-oracle re-check).**

### The specific numeric rate (~2⁻¹³·⁹, "1800× worse than assumed", "3.5×10⁸ trials")

**UNVERIFIABLE FROM REPO STATE — flag, do not treat as pinned.** Searched all four repos for any
test, script, or fixture backing "2⁻¹³·⁹", "6.5×10⁻⁵", "3.5×10⁸", or "1.15e-4": **none exist.**
The eval report's own "test-improvement program" (§2, items 6-8) *proposes* — as NOT-YET-WRITTEN
tests — `mnemonic-toolkit/tests/prop_repair_never_wrong.rs`, `md/tests/bch_exhaustive_sweep.rs`
(with "a seeded 5-8-error acceptance-rate cell pinned below a bound (~1.15e-4 measured)"), and an
`mk` `bch_correct_ok_implies_valid_codeword` proptest — confirmed absent by `find`/`grep` across
all four repos. So the eval's quantitative claims are the result of an **ad-hoc measurement run
during the eval's own investigation, not committed anywhere as a reproducible artifact.**

Two internal-consistency notes for the fix-cycle author to resolve (not blocking, but should be
re-derived/cited precisely rather than carried forward as-is):
- The finding's own numbers imply a ratio of `6.5×10⁻⁵ / 2^-23.7 ≈ 6.5×10⁻⁵ / 7.34×10⁻⁸ ≈ 886×`,
  not the headlined **"~1800×"**. Either "assumed" refers to a different baseline than the
  in-paragraph `ecc-4` figure, or the headline multiplier needs recomputation.
- The magnitude itself (a 5-error, one-past-threshold pattern miscorrecting at roughly
  `10⁻⁵`–`10⁻⁴`) is **plausible order-of-magnitude** for a bounded-distance BCH/BM-Chien-Forney
  decoder one symbol past `t` (a sphere-packing-style collision argument over the `GF(32)`-symbol,
  93/108-length codeword space lands in this range) — this recon does not certify the exact digits,
  but the shape of the claim is credible and matches the "5+-error edge case" framing already
  present in the shipped code comments.

**Recommendation:** the fix cycle should commit the measurement as a real, seeded, reproducible
test (exactly what eval items 6-8 propose) rather than re-citing the eval's un-reproducible number
in a spec. Do not let a SPEC hard-code "2⁻¹³·⁹" as an established fact; cite it as "the eval's own
ad-hoc estimate, order-of-magnitude 10⁻⁵–10⁻⁴, to be re-measured and pinned in-repo."

---

## Cross-cutting observations

1. **The three codecs are NOT equally exposed, despite sharing `GEN_REGULAR`.** md1 already has a
   real (if narrower-than-ideal) second oracle: `md_codec::chunk::reassemble`
   (`md-codec/src/chunk.rs:306-386`) performs a **"Cross-chunk integrity check"**
   (`chunk.rs:379-386`) that re-derives a 20-bit `chunk_set_id` from the *decoded descriptor's
   content* (`compute_md1_encoding_id` → `derive_chunk_set_id`, `chunk.rs:176-179`) and rejects if
   it disagrees with the header's embedded id — and this check runs **unconditionally, even for a
   single-chunk (`count=1`) md1 card**, because it's part of `reassemble`, which
   `decode_with_correction` (`chunk.rs:503-...`, step 5 of its own doc comment) already calls at
   the end of every repair. `md repair` (`md-cli/src/cmd/repair.rs:118`) delegates directly to
   `decode_with_correction` — so **`md repair` is NOT vulnerable to the F4 gap today**; it already
   does what the fix asks for (weaker than a full 32-bit hash — 20 bits of content-derived id — but
   present and wired in). The toolkit's `Md1` arm (`repair_via_md_codec`, `src/repair.rs:1219-1262`)
   inherits this for free. **This means the finding's file list (`md-codec/src/chunk.rs (md1
   single-string)`) is over-broad**: md1 is not equivalently exposed to mk1/ms1; it is already
   substantially (not perfectly) mitigated. The fix's actual scope is **mk1 (both `mk-cli` and
   toolkit) and ms1 (both `ms-cli` and toolkit)** — md1 needs, at most, hardening (the 20-bit id
   check is weaker than mk1's design intent of a 32-bit SHA-256-truncated cross-chunk hash) or a
   documentation-only note, not a structural fix.
2. **A working fix template already exists in the toolkit codebase.** `Mk1IndelOracle` /
   `Md1IndelOracle` (`src/repair.rs:1036-1083`) already implement "solve the chunk, then confirm
   full-card reassembly via the real decode/reassemble call" for the *indel* recovery path. The
   substitution-repair path (`repair_card`, `src/repair.rs:760-838`) is the one place in the
   toolkit that skips this pattern. This substantially de-risks the fix: it is largely "apply the
   already-proven oracle-gating idiom to a sibling code path," not a novel design.
3. **BIP-93's own "SHOULD NOT auto-proceed without user confirmation" is a stronger primary-source
   argument for urgency than the specific miscorrection-rate number** — it means the current
   default-TTY auto-repair behavior (documented in `docs/manual/src/40-cli-reference/41-mnemonic.md:739-751`)
   arguably already sits in tension with the BIP's own recommendation, independent of exactly how
   rare a >4-error miscorrection is.
4. **Existing test coverage is happy-path only.** `crates/mnemonic-toolkit/tests/cli_auto_repair.rs`
   and `cli_repair.rs` (1050 lines combined) cover 1-substitution auto-fire scenarios and the
   TTY/`--no-auto-repair` matrix; none exercise a >4-error / miscorrection scenario. No fuzz target
   in any of the 4 repos targets "does a >t correction ever silently differ from the injected
   original" (mk-codec's `fuzz/fuzz_targets/mk1_decode.rs` / `mk1_decode_single.rs` exist but only
   check panic-freedom, per the eval's own item 8 recommending a NEW proptest).
5. **Manual has no caveat today.** `docs/manual/src/40-cli-reference/41-mnemonic.md:2990-3037`
   (`## mnemonic repair`) documents exit 5 as an unqualified "corrected" outcome with no >4-error
   caveat and no "then independently verify" instruction — confirmed, matches the finding.
6. **No claim-counting ambiguity found** in the file-list/citation set itself (each citation named
   a single concrete file:line and checked out structurally); the only numeric discrepancy is the
   "~1800×" vs. the finding's own "~886×"-implying numbers (see crypto section above).

---

## Recommended scope + blast-radius map

### Blast radius (which repos/functions actually need to change)

| Repo | Change needed | Why |
|---|---|---|
| **mk-codec** (`mnemonic-key`) | None required — `reassemble_from_chunks` + cross-chunk hash already exist (`string_layer/chunk.rs:109`, `mod.rs:38` `pub use`). Just needs a **caller**. | Primitive already public. |
| **mk-cli** (`mnemonic-key`) | `mk-cli/src/cmd/repair.rs` — after the per-string loop, build `refs: Vec<&str>` from `corrected_chunks` and call `mk_codec::decode(&refs)` (or `reassemble_from_chunks` directly); on failure, degrade exit 5 → an advisory/non-5 outcome. | The actual gap. |
| **mk-codec** | Possibly expose a "verify only" variant of `decode`/`reassemble_from_chunks` if CLI wants to avoid discarding a fully-parsed `KeyCard` (minor ergonomics, not required). | Optional. |
| **toolkit** | `src/repair.rs`'s `repair_card`'s `CardKind::Mk1` branch — after building `corrected_chunks`, call `mk_codec::decode(&refs)` (mirror `Mk1IndelOracle::validate`'s existing pattern at `:1051`) before returning `Ok(RepairOutcome)`. | The actual gap; template exists in the same file. |
| **ms-codec** (`mnemonic-secret`) | New: some payload-derived second oracle does not exist and arguably *cannot* be invented for raw BIP-39 entropy (there is nothing else to check against) — per the eval's own fix language, the realistic option is **demotion**: a `decode_with_correction` result requiring `> some conservative threshold` (or simply: any correction at all, if the risk is judged unacceptable for a bearer-secret) returns an advisory/candidate status rather than a silent `Ok`. This is a **spec decision**, not a mechanical fix — needs the R0 gate to pick the threshold/exit-code semantics. | Highest-consequence, no existing net to wire up — genuinely new design work. |
| **ms-cli** | `ms-cli/src/cmd/repair.rs` — reflect whatever ms-codec decides (e.g., a new exit code / "VERIFY-ME" framing, mirroring `mnemonic repair`'s existing `--max-subst` "candidate" convention at exit 4). | Follows ms-codec's decision. |
| **toolkit** | `repair_card`'s `CardKind::Ms1` branch + `try_repair_and_short_circuit` — must not silently auto-fire exit 5 for an ms1 correction beyond whatever the new ms-codec threshold is; needs to either suppress auto-fire or downgrade to the existing exit-4 "VERIFY-ME" convention already used for indel+substitution combos (`RepairError`/`indel_exit_code`, `:1118-1136`). | Same auto-repair-default exposure as mk1. |
| **md-codec / md-cli / toolkit md1 arm** | No structural fix required (already gated by the cross-chunk content-id check). Optional hardening: widen the 20-bit `derive_chunk_set_id` check or add an explicit doc/test asserting this already covers the F4 risk class, so a future reader doesn't rediscover "does md1 need the F4 fix too?" from scratch. | Already largely mitigated; lowest priority in this cluster. |
| **All 3 CLIs + toolkit** | Manual (`docs/manual/src/40-cli-reference/41-mnemonic.md`, `42-md.md`, `43-ms.md`, `44-mk-cli.md`) — add the >4-error / miscorrection caveat to the `repair` chapters and the auto-fire sections; mirror invariant per `CLAUDE.md` requires this in lockstep with any CLI-surface/exit-code semantics change. | Mandatory per repo convention if exit-code contracts change. |
| **GUI** (`mnemonic-gui`) | Only if an exit-code semantics change alters the schema (unlikely — exit codes aren't part of `schema_mirror`'s flag-name surface) or if new flags are added (e.g., a `--repair-confidence` style flag). Check at plan time. | Likely no-op, verify. |

### SemVer

- mk-codec / mk-cli: behavior change (repair set-level re-verify can newly reject/downgrade a
  previously-"successful" repair) — this is a **breaking behavior change to an existing exit-code
  contract**, warranting a **MINOR** bump pre-1.0 per repo convention (breaking = MINOR pre-1.0).
- ms-codec / ms-cli: new advisory/exit-code semantics for >t corrections — **MINOR**.
- md-codec / md-cli: likely **PATCH** (hardening only) or no bump if scoped out of this cycle.
- toolkit: **MINOR** (repair/auto-repair behavior change across all three arms + manual mirror).
- Codecs publish to crates.io in lockstep (per `CLAUDE.md`); toolkit's git-deps must re-pin to the
  new codec versions in the same cycle.

### Split recommendation

**Split into two cycles, not one:**

- **Cycle 1 (mechanical, template exists, lower risk): mk1 + md1-hardening set-level re-verify.**
  Wire `mk-cli`'s and toolkit's Mk1 substitution-repair path through the already-public
  `mk_codec::decode`/`reassemble_from_chunks`, mirroring the existing `Mk1IndelOracle` idiom
  byte-for-byte. Optionally hardens md1's existing check / adds an explicit regression test
  documenting md1 is already covered. Add the manual caveat for mk1 (and md1 if touched). This is
  well-scoped, low-design-risk, and should ship first — it's the "prove the pattern, low blast
  radius" leg.
- **Cycle 2 (design-heavy, ms1 advisory semantics): ms1 single-string demotion.** Requires an
  actual design decision (what exit code, what UX, does auto-repair suppress entirely for ms1
  beyond some correction count, does `--max-subst`'s existing "VERIFY-ME" exit-4 convention already
  give a ready-made answer?) — this needs its own brainstorm + R0 spec, not a copy of Cycle 1's
  mechanics. **ms1 is the single highest-funds-risk item in this whole cluster** (bearer secret,
  zero downstream oracle, auto-fires by default) and should not be delayed behind Cycle 1, but it
  is a different *kind* of work (product/security-UX decision vs. mechanical wiring) and shouldn't
  block Cycle 1's ship.

Rough sizing: Cycle 1 ≈ 150-300 LOC across mk-codec (maybe 0, if `decode` is reused as-is)/mk-cli
(~30 LOC)/toolkit (~30-50 LOC + tests)/manual (~1 chapter section); Cycle 2 ≈ similar code size but
materially more design/spec time (the R0 gate will need to converge on the exit-code semantics
before any code is written) plus the ms-codec crate change + ms-cli + toolkit Ms1 arm + manual.

**Confirmed as the next-highest funds risk:** yes. This sits alongside (not below) the constellation
eval's other open Critical/Important items; ms1's exposure in particular — a bearer secret,
auto-repaired by default, with literally zero corroborating oracle if the BCH decoder aliases past
t=4 — is a plausible "wrong seed, confidently reported as recovered" outcome, the same class of
severity as the already-shipped Cycle A (use-site collapse) fix.
