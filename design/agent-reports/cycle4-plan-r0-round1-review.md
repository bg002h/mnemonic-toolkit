# cycle-4 PLAN R0 — round 1 review

**Artifact:** `design/IMPLEMENTATION_PLAN_cycle4_codec_funds_fixes.md`
**Implements (GREEN, 0C/0I):** `design/BRAINSTORM_cycle4_codec_funds_fixes.md` (spec-R0 rounds 1–2)
**Source-of-truth SHAs (verified live this review):**
- md-codec / md-cli `descriptor-mnemonic` `origin/main` **`58cc9ec`** (md-codec 0.37.0 · md-cli 0.8.0, pin `md-codec =0.37.0`)
- ms-codec / ms-cli `mnemonic-secret` `origin/master` **`6b28918`** (ms-codec 0.4.4 · ms-cli 0.8.0, pin `ms-codec =0.4.4`)
- mnemonic-toolkit `origin/master` **`c578e123`** (toolkit 0.62.0; pins `md-codec = "0.37"`, `ms-codec = "0.4.4"`, `codex32 = "=0.1.0"`)

**Reviewer:** opus software architect (adversarial). **Date:** 2026-06-21. **Gate:** HARD R0 — no code until 0C/0I.

---

## Method

Every plan claim was re-verified against live origin (`git show <ref>:<path>`), not taken from the plan
or the spec. Edit sites, exhaustiveness of match blocks, `#[non_exhaustive]` status, caret semantics, the
length-agnosticism of `bch_verify_regular`, and the toolkit decode-path reachability of the I1 fix were all
independently confirmed. Findings below cite the live line I read.

---

## Critical

**None.**

All four fixes target the correct, currently-vulnerable code paths; all three new-variant lockstep sites are
correctly identified (one compile-forced, two silent); the publish→pin order is sound and the caret-pin
blocking-edit claim is correct. No funds-safety regression, no broken-build, no vacuous-RED issue rises to
Critical.

---

## Important

**None.**

The plan faithfully executes the GREEN spec. The items below are Minor (precision / coverage-rounding /
out-of-scope residue), none gate the start of code.

---

## Minor

### m1 — `shares.rs:235` is the C1 *reject site*, not the variant *definition* site (citation imprecision)
Plan Phase B1: "New unit variant `Error::InconsistentShareSet` near `SecretShareSuppliedToCombine`
(`:235`)." Live: `shares.rs:235` is `return Err(Error::SecretShareSuppliedToCombine);` (the C1 reject in
`combine_shares`), not the enum-variant declaration. The variant is **declared** at `error.rs:122`
(`SecretShareSuppliedToCombine,`), which is the actual placement target ("near the combine-family
variants"). The brainstorm §6.3 gets this right (`error.rs:122`); the plan's `shares.rs:235` is a stray.
**Fix:** in the plan, change the variant-placement citation to `error.rs:122` (the `:235` reference is fine
only as "the reject-site neighbor for context").

### m2 — md-codec new-variant placement lines have drifted; re-grep at write time
Plan A2 cites `error.rs:262-294` for the `ChunkSetEmpty`/`ChunkCountExceedsMax` family and A1 implies
`TooManyErrors` is the H6 neighbor. Live (md-codec `error.rs`): `ChunkCountExceedsMax` is at `:262`,
`ChunkSetEmpty` at `:277`, the chunk family runs `:232`–`:307`, and `TooManyErrors` is at `:422`. The
ranges are close but already off by a few lines from the spec's snapshot. Per CLAUDE.md "citations are
grep-verified at write time", the per-track plan-doc / per-phase TDD MUST re-grep against the worktree HEAD
before placing variants. **Fix:** add an explicit "re-grep variant-placement lines on the worktree branch
before editing `error.rs`" note (the plan already says this generally; make it variant-specific for the 3
md-codec adds).

### m3 — I1's toolkit-observable surface is `reassemble`, not just `repair`; add a restore/inspect characterization
The plan's Phase-C characterization tests cover only the **correcting** path (`mnemonic repair --md1
<over-93>`) and M6 combine. But I verified the I1 cap at `unwrap_string` (`codex32.rs:113`) is reached by
the toolkit's dominant **non-correcting** decode, `md_codec::chunk::reassemble` (`chunk.rs:305` → `:321`
calls `unwrap_string` per string; the single-string `decode_with_correction` branch also routes via
`decode_md1_string` → `unwrap_string`, `chunk.rs:613`/`decode.rs:87`). `reassemble` is what
`restore`/`import`/`bundle`/`inspect` call (e.g. `cmd/*` lines 1079/1250/2357/3059/3269, inspect `:175`).
So a clean over-93 single md1 fed to `mnemonic restore --md1` / inspect would now reject — a real
behavior change with **no** toolkit characterization test in the plan. **Fix (optional, hardening):** add a
toolkit characterization `mnemonic restore --md1 <clean over-93 string>` (or inspect) → exit 2, alongside
the existing `repair --md1` test, so the I1 routing through `reassemble` is guarded, not only the md-codec
unit test + the compile-forced exit-code arm.

### m4 — M4 RED test #1 must assert the *specific* post-fix variant, not merely "not clean-reject"
Plan A2 / spec §5.5.1 frame the pre-fix assertion as "decode does NOT cleanly reject" / "enters the
aliasing path". For a dirty (`residue != 0`) over-93 word, today's
`decode_regular_errors(residue, symbols.len()>93)` either aliases to a wrong-position "correction" (Ok) OR
returns `None` → `Error::TooManyErrors`. Both are "not a typed length reject", so the soft RED passes today
either way — fine. But the **post-fix** assertion must pin
`Err(Error::ChunkSymbolCountOutOfRange { .. })` exactly (not just "is Err"), else the test would still pass
if the guard mistakenly produced `TooManyErrors`. The plan does say "the post-fix assertion is the typed
reject"; make it an exact-variant `assert!(matches!(err, Error::ChunkSymbolCountOutOfRange { .. }))` so the
guard's *correct* error path is what is locked.

### m5 — indel md1 repair path (`--max-indel ≥ 1`) is an out-of-scope residual over-length surface (note only)
I confirmed `repair --max-indel` defaults to 0 (`cmd/repair.rs:65`), so the plan's default `repair --md1`
characterization correctly exercises the guarded `decode_with_correction` path. However, under
`--max-indel ≥ 1` the md1 branch routes to the toolkit-local indel solver (`md1_chunk_solve` /
`repair_chunk_one(Md1, …)`, `repair.rs:74`), which does **not** go through md-codec's
`decode_with_correction` and therefore does **not** inherit the M4 cap. This is a separate toolkit-side
surface, correctly **out of cycle-4 scope** (cycle-4 caps the codec). **Fix:** none required for cycle-4;
add a one-line FOLLOWUP-or-note that the toolkit indel path's over-93 handling is unaddressed by the codec
cap, so a future cycle doesn't assume the codec fix covers it.

### m6 — md-cli new-error exit code: confirm "exit 1" is the intended class for H6/M4/I1 on the `md` CLI
Plan A-ship step 2 / spec §7.1: md-cli inherits the new rejects via the opaque `CliError::Codec(_)` wrapper
(`md-cli/src/error.rs:42`), which prints `md: codec error: {e}` and exits **1** (`main.rs:257`,
non-`Repair` arm). Verified. The toolkit routes the *same* three variants to **exit 2** (decode/format
class). So the `md` CLI and the `mnemonic` CLI will disagree on exit code for an identical over-93 input
(md → 1, mnemonic → 2). This is consistent with the existing pattern (md-cli already collapses most codec
rejects to exit 1 via the opaque wrapper; only `Repair` gets a bespoke code), so it is **not a defect** —
but the plan should state the deliberate divergence so a reviewer doesn't read the md-cli "exit 1" as a
lockstep miss. **Fix:** add one sentence to A-ship step 2 noting md-cli's exit-1 collapse is intentional
(opaque wrapper, LEAN-PATCH) and differs from the toolkit's exit-2 routing by design.

---

## Targeted answers to the dispatch questions

1. **Edit sites exist as cited (re-grepped live):**
   - A1: `wrap_payload` **codex32.rs:67** ✓ (data_symbols at `:68`, no cap); `encode_md1_string`
     **encode.rs:136** ✓ (`wrap_payload` call `:138`).
   - A2: `decode_with_correction` **chunk.rs:502** ✓; `residue == 0` pass-through **chunk.rs:525** ✓ (guard
     must sit after `parse_chunk_symbols` `:518`, before `:525` — plan's placement is correct);
     `decode_regular_errors` **bch_decode.rs:403** ✓ (no length gate, `deg` gate `:416`); `chien_search`
     **bch_decode.rs:284** ✓ (unbounded `for d in 0..L` `:293`). The plan's `:403`/`:284` are right; the
     literal `chunk.rs:536` `decode_regular_errors` call is at `:536` ✓.
   - A3: `unwrap_string` **codex32.rs:113** ✓; `bch_verify_regular` **codex32.rs:144** ✓ (verified
     length-agnostic: `bch.rs:25` doc "`polymod(valid codeword) == MD_REGULAR_CONST` holds at every
     length" — so a CLEAN >93 string verifies today); too-short floor `:151` ✓ (no upper cap → I1
     reproduces).
   - B1: step-5 `interpolate_at(&parsed, Fe::S)` **shares.rs:263** ✓; arbitrary-index primitive
     `interpolate_at(&defining, *pool_idx)` **shares.rs:153** ✓; exhaustive `Display`
     **error.rs:125/:127** ✓ (NO `_ =>`; last arm `SecretShareSuppliedToCombine` `:221`); the `:235`
     "variant neighbor" is the *reject* site, variant decl is `:122` (see m1).
   - Phase-C toolkit: `md_codec_exit_code` **error.rs:464** ✓ **EXHAUSTIVE** (ends
     `WireVersionMismatch => 3` `:518`, NO `_ =>`); `TooManyErrors`-group exit-2 at **:516** ✓;
     `ms_codec_exit_code` `_ => 1` at **:419** ✓; `SecretShareSuppliedToCombine => 2` at **:417** ✓;
     `friendly_ms_codec` **friendly.rs:45** ✓ (has `_ =>` fallback `:147` → a miss is silent-generic);
     pins `Cargo.toml:36` (`md-codec = "0.37"`) / `:29` (`ms-codec = "0.4.4"`) / `:34` (`codex32 =
     "=0.1.0"`) ✓; `From<ms_codec::Error>` `:929` wildcard `:939` ✓; combine call site
     `cmd/ms_shares.rs:409` ✓. ms-cli `From` wildcard **error.rs:246** ✓ (`other => CliError::BadInput`).

2. **TDD integrity — all four RED tests are genuinely RED-first:**
   - **H6** (`wrap_payload_rejects_over_80_data_symbols`): RED — no cap exists in `wrap_payload`; returns
     `Ok` today. Positive control (exactly-80) is a genuine off-by-one guard.
   - **M4** (`decode_with_correction_rejects_over_93_symbol_chunk`): RED — `decode_with_correction` passes
     `symbols.len()` uncapped into `decode_regular_errors`/`chien_search` (unbounded loop), so today it
     aliases or `TooManyErrors`-rejects, never the typed length reject. Constructible/deterministic via the
     331-symbol/pos-100 aliasing fixture. (See m4: pin the exact post-fix variant.) Positive control
     `valid_chunked_md1_still_repairs` is a real regression guard (each chunk ≤ 93).
   - **I1** (`unwrap_string_rejects_clean_over_93_symbol_string`): RED — `unwrap_string` has only a
     too-SHORT floor (`:151`); `bch_verify_regular` is length-agnostic (`bch.rs:25`), so a clean >93 word
     decodes out-of-domain today. The RED is constructible: build >93 data + valid
     `bch_create_checksum_regular` checksum. Positive control (93-symbol legal) genuine.
   - **M6** (`combine_inconsistent_same_id_set_rejected`): RED — step-5 interpolates over **all** `&parsed`
     (`shares.rs:263`) with no truncation/consistency gate; a same-id `[A1,B2]` set returns *a* (wrong)
     secret with no error today (not pre-empted by another error: distinct-index, threshold, and C1 checks
     all pass for a well-formed same-id mixed set). Positive controls `combine_valid_exactly_k_unchanged`
     (`n==k` → membership loop empty → bit-identical) and `combine_valid_n_gt_k_all_consistent` are genuine
     regression guards for §6.0's hard invariant.

3. **Publish→pin order — SAFE.** md-codec→md-cli, ms-codec→ms-cli (parallel tracks), THEN one toolkit
   PATCH. The toolkit pin-bump **cannot compile** before both codecs are on crates.io: bumping the caret
   strings to `"0.38"`/`"0.5"` makes `cargo`/`cargo update` resolve the registry crate, and (independently)
   the 3 new md-codec variants are referenced by the toolkit's new exhaustive-match arms — both require the
   published codec. The plan enforces "C is the single join … needs both on crates.io." SemVer calls
   correct: md-codec MINOR (3 additive variants, behavior-tighten on never-contracted out-of-code input),
   ms-codec MINOR (additive variant on a `#[non_exhaustive]` enum), both CLIs PATCH, toolkit PATCH.

4. **Caret-pin BLOCKING claim — CORRECT.** Live pins are caret (bare `"0.37"` / `"0.4.4"` = `^`). Cargo
   0.x caret semantics: leftmost non-zero is the breaking digit → `^0.37` resolves `>=0.37.0,<0.38.0` and
   `^0.4.4` resolves `>=0.4.4,<0.5.0`. The 0.38.0 / 0.5.0 targets are OUTSIDE both ranges, so
   `cargo update -p md-codec -p ms-codec` cannot cross them — a hand-edit of the pin STRINGS is mandatory.
   The CLI exact pins are `md-codec =0.37.0` (md-cli `Cargo.toml:28`) and `ms-codec =0.4.4` (ms-cli
   `Cargo.toml:20`); these too must be hand-edited to `=0.38.0` / `=0.5.0`. All confirmed live.

5. **Lockstep completeness — COMPLETE on both sides:**
   - **md side (compiler is the backstop):** `md_codec_exit_code` (`error.rs:464`) is exhaustive (ends
     `WireVersionMismatch => 3`, no `_ =>`). md-codec `Error` is **NOT `#[non_exhaustive]`** (verified
     `error.rs:19` — only `#[derive(Debug, Error, PartialEq, Eq)]`). So the 3 new variants force a compile
     error until all 3 arms are added. The plan adds all 3 → exit 2, alongside the `TooManyErrors`/chunk-shape
     exit-2 group (`:516`). **The compiler catches a miss — confirmed.**
   - **ms side (no compiler — explicit arms required):** the plan adds the explicit
     `InconsistentShareSet => 2` arm to `ms_codec_exit_code` (whose `_ => 1` at `:419` would otherwise
     silently route exit 1) **AND** a `friendly_ms_codec` prose arm (whose `_ =>` at `:147` would otherwise
     emit the generic "unhandled variant" string). Both are in the plan (§C "ms side"). ms-cli also gets an
     explicit `From<ms_codec::Error>` arm (its `:246` wildcard otherwise maps BadInput/exit 1). **All three
     ms-side silent sites covered.**
   - **Exit-2 routing is correct:** all four new rejects are funds/format-reject class (over-domain
     encode/decode + inconsistent-share-set) → exit 2 matches the existing decode-reject / FormatViolation
     convention (md `TooManyErrors => 2`; ms `SecretShareSuppliedToCombine => 2`).

6. **Version-site completeness — COMPLETE.** md-codec/ms-codec `Cargo.toml` + `CHANGELOG.md` (both present:
   `descriptor-mnemonic/CHANGELOG.md`, `mnemonic-secret/crates/ms-codec/CHANGELOG.md`); CLI `Cargo.toml`
   exact-pins + CHANGELOGs; toolkit `Cargo.toml` + **root** `Cargo.lock` + **root** `fuzz/Cargo.lock`
   (NOT `crates/mnemonic-toolkit/fuzz/…` — the lock lives at repo-root `fuzz/Cargo.lock`; the plan's
   bare "`fuzz/Cargo.lock`" is correct) + BOTH READMEs (`README.md` + `crates/mnemonic-toolkit/README.md`,
   each carrying `<!-- toolkit-version: 0.62.0 -->`, gated by `tests/readme_version_current.rs` — must bump
   to `0.62.1`) + toolkit `CHANGELOG.md` (gated by `.github/workflows/changelog-check.yml`). "toolkit is
   tag-only, no publish" is CORRECT (not a registry crate). "codecs DO publish" is CORRECT (registry
   crates). No site missed.

7. **Multi-instance safety — SAFE.** The plan mandates worktrees off each repo's `origin` default and
   explicitly forbids committing on the parked `feature/own-account-subset-search` (toolkit main checkout
   is on it at version 0.60.0 — verified). The 0.62.1-vs-own-account handling is sound: cut the PATCH off
   `origin/master` (0.62.0 → 0.62.1) and treat own-account's renumber as that cycle's concern; the plan
   explicitly flags "0.63.0" as speculative and does not hard-depend on it. Correct.

---

## Spec-faithfulness check (does the PLAN execute the GREEN spec?)

Yes. Every locked decision D1–D18 is reflected: REJECT-not-auto-chunk in `wrap_payload` (D1/D2/D3),
`PayloadTooLongForSingleString` (D4), two-layer M4 guard + typed `ChunkSymbolCountOutOfRange` (D5/D6),
two-guards-not-one (D7), truncate-to-k + membership via existing `interpolate_at` primitive (D8/D11),
beyond-BIP-93 framing + hard invariant preserved by positive controls (D9), `InconsistentShareSet` unit
variant + compile-forced Display arm (D10), SemVer (D12), publish order (D13), md-side compile-forced /
ms-side silent lockstep (D14), no manual/GUI flag change (D15), version-collision handling (D16), the
I1 non-correcting cap at `unwrap_string` + `StringSymbolCountOutOfRange` (D17), and the caret-pin
blocking hand-edits (D18). No spec decision is dropped, weakened, or contradicted.

---

## Verdict

**PLAN R0 ROUND 1: 0C / 0I — GREEN (0C/0I).**

The plan faithfully executes the GREEN brainstorm spec; all edit sites, RED-test mechanisms, lockstep
sites (one compile-forced md side, two silent ms sites), publish→pin order, caret-pin blocking edits,
and version-site coverage are independently verified against live origin. The six Minor items are
citation-precision / coverage-rounding / out-of-scope-residue notes — fold them opportunistically; none
gates the start of Phase-A1/B1 code.
