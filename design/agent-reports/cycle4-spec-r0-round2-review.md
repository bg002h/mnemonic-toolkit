# Cycle-4 codec funds-safety spec — R0 review (Round 2)

**Reviewer role:** opus software architect, mandatory R0 gate (NO implementation until 0C/0I).
**Date:** 2026-06-21.
**Spec under review:** `design/BRAINSTORM_cycle4_codec_funds_fixes.md` (H6 + M4 in md-codec; M6 in ms-codec) — POST-FOLD of Round-1's 0C/2I.

**Source SHAs independently verified against (live `git show <origin-ref>:<path>`):**

| repo | branch | origin SHA (verified this round) |
|---|---|---|
| descriptor-mnemonic (md-codec / md-cli) | `main` | **`58cc9ec25b3d35120c8e785d3c2ce7f48322529b`** |
| mnemonic-secret (ms-codec / ms-cli) | `master` | **`6b289186c12380c228974ce919eb85f758348aca`** |
| mnemonic-toolkit (consumer) | `master` | **`c578e123466a61ae62db9c54769cc4928ed52aff`** |

**Verdict (jump):** `R0 ROUND 2: 0C / 0I` — **GREEN.** Both Round-1 Important findings (I1 = non-correcting decode path uncapped; I2 = drifted citations + under-stated compile gates) are RESOLVED. Every line citation in the folded sections (§5.2.3, §6.1, §6.3, §6.4, §7.2, §7.3, D17, D18) re-verified byte-exact against the three live SHAs. The variant count is now uniformly **three** (no lingering "two"). The folds introduced **no new Critical or Important**. One pre-existing **Minor** (M-min-1, the speculative own-account "0.63.0" figure) remains unfolded but is non-gating; carried forward.

---

## Method

Re-grepped every citation in the folded sections against the three live origin SHAs. Traced the I1 cap's interaction with the M4 boundary cap and the chunked (`reassemble`) path to test for fold-introduced double-gating/conflict and legitimate-path regression. Re-confirmed the D8/§6.2 M6 membership design is byte-identical to Round-1's PROVED-correct form. Grepped the whole doc for variant-count drift and residual open questions. Confirmed the own-account branch version on origin.

---

## CRITICAL

**None.**

---

## IMPORTANT

**None.** (Both Round-1 Importants resolved — see fold-verification below.)

---

## I1 — RESOLVED (non-correcting decode path now capped)

**Round-1 ask:** add a `symbols.len() > 93` cap to `unwrap_string` (the `decode_md1_string` primitive), with a RED test, and a distinct variant; OR document an argued carve-out. The spec took option (a) (preferred).

**Verified against live `58cc9ec`:**
- `decode.rs:86` `decode_md1_string(s)` → `unwrap_string(s)?` (call at `:87`) → `decode_payload`. Confirmed — the single-string non-correcting entry point. Spec §5.2.3 cites `decode.rs:86`. **Exact.**
- `codex32.rs:113` `unwrap_string`. Char-decode loop builds `symbols` (`:131-142`); `bch_verify_regular(HRP, &symbols)` at **`:144`**; too-SHORT floor `symbols.len() < REGULAR_CHECKSUM_SYMBOLS` at **`:151`**; checksum strip at `:156`. **NO upper cap today.** Spec §5.2.3 / D17 cite `codex32.rs:113`, `:144`, `:151` — **all exact.**
- `bch_verify_regular` (`bch.rs:89`) is `polymod_run(&input) == MD_REGULAR_CONST` with only a too-SHORT floor `< 13` (`:90`) — **length-agnostic**. Confirms a clean over-length md1 BCH-verifies and proceeds today. The fold's premise holds.
- New variant `Error::StringSymbolCountOutOfRange { symbols, max }` (no `chunk_index`) is coherent: md-codec `Error` is `#[derive(Debug, Error, PartialEq, Eq)]`, **NOT `#[non_exhaustive]`** (`error.rs:19`; grep for `non_exhaustive` is clean). `usize` fields keep `PartialEq/Eq`. Distinct from the chunk-indexed `ChunkSymbolCountOutOfRange`. Coherent.

**Boundary correctness (`>93` codeword vs H6's `>80` data):** at the `unwrap_string` cap point, `symbols` is the full char-decoded vector INCLUDING the 13-symbol checksum (stripped only at `:156`, after BCH-verify). So `symbols.len()` = data + 13 = full codeword; `> 93` rejects data > 80. **Symmetric with H6's data-only `> 80` cap (80 + 13 = 93) and with M4's `> 93` codeword cap.** All three converge on the same BCH(93,80,8) protocol limit. The §5.2.3 "Guarantee" paragraph (longest legal single-string md1 = 80 data + 13 = 93, so `> 93` never false-rejects) is correct.

**RED test #5 (`unwrap_string_rejects_clean_over_93_symbol_string`) is genuinely RED today and constructible:** the test reuses H6 RED-test-#3's oversize descriptor, which today's uncapped `wrap_payload` encodes into a **clean (residue 0) >93-symbol** md1 that even round-trip-verifies. Feeding that same string to `decode_md1_string` today: `unwrap_string` BCH-verifies (clean), strips checksum, and `decode_payload` decodes the descriptor from the full bit stream → ACCEPTS today (RED). After the fold's `> 93` cap → `Err(StringSymbolCountOutOfRange)` (GREEN). The 93-symbol (80-data) positive control still decodes. Constructible and correctly RED/GREEN — and it rides H6's already-confirmed "round-trip-verifies" fact, so it needs no new fixture machinery.

**Verdict: I1 RESOLVED.** The cap is now on BOTH decode entry points; the spec's M4 domain-cap rationale is internally consistent.

---

## I2 — RESOLVED (citations corrected; compile gates + pin-edits documented)

**Round-1 ask:** correct drifted line numbers at the funds-relevant lockstep sites; add the ms-codec exhaustive-`Display` intra-crate compile-gate note; add the explicit toolkit caret-pin-string-edit requirement; make the three-variant exit-2 mapping an explicit line-cited checklist.

**Re-verified against live `c578e123` / `6b28918` / `58cc9ec` — every corrected number is now exact:**

| spec citation | live | match |
|---|---|---|
| `ms_codec_exit_code` opens `error.rs:399` (§6.4) | `:399` | ✓ |
| exit-2 group ends `SecretShareSuppliedToCombine => 2` `:417` (§6.4) | `:417` | ✓ |
| `_ => 1` at `:419` (§6.4) | `:419` | ✓ |
| ms-cli `From<ms_codec::Error>` wildcard `other => CliError::BadInput("unhandled …")` `:246` (§6.4) | `:246` | ✓ |
| `combine_shares` `shares.rs:186` (§6.1) | `:186` | ✓ |
| C1 `return Err(SecretShareSuppliedToCombine)` `:235` (§6.1) | `:235` | ✓ (R1 corrected the wrong `:234`) |
| `k = (fields[0].0 - b'0')` `:242` (§6.1) | `:242` | ✓ |
| `parsed.len() < k → ThresholdNotPassed` `:243` (§6.1) | `:243` | ✓ |
| step-5 `interpolate_at(&parsed, Fe::S)` `:263` (§6.1) | `:263` | ✓ |
| existing primitive `interpolate_at(&defining, *pool_idx)` `:153` (§6.1) | `:153` | ✓ |
| md `md_codec_exit_code` exhaustive, opens `:464`, exit-2 group `:516`, `WireVersionMismatch => 3` tail (§7.3) | `:464` / `:516` / `:518` | ✓ |
| toolkit pins `md-codec = "0.37"` Cargo.toml `:36`, `ms-codec = "0.4.4"` `:29`, `codex32 = "=0.1.0"` `:34` (§7.2/D18) | `:36` / `:29` / `:34` | ✓ |
| toolkit `From<ms_codec>` wildcard `:939` / `From<md_codec>` wildcard `:966` (§6.4/§7.3) | `:939` / `:966` | ✓ |
| md-cli opaque `CliError::Codec(md_codec::Error)` `md-cli/src/error.rs:5`, `From` `:42` (§7.1) | `:5` / `:42` | ✓ |

**ms-codec exhaustive-`Display` intra-crate compile gate — ADDED and CORRECT (§6.3/D10):** `impl fmt::Display for Error` opens `error.rs:125`, `fn fmt` `:126`; the `match self` has **NO bare `_ =>` arm** (grep `^\s*_ =>` over the file returns nothing; the only `safe =>` at `:156` is an inner match on the wrapped codex32 error, not the outer enum), closing at `SecretShareSuppliedToCombine => write!(…)` `:221`. ⇒ adding `InconsistentShareSet` **compile-forces** a Display arm inside ms-codec — the GOOD intra-crate gate. `Debug` delegates to Display (`:237` `write!(f, "Error(\"{self}\")")`) so it inherits the arm automatically. The spec §6.3 now states this. **Resolved.**

**Caret-pin BLOCKING-edit requirement — ADDED and CORRECT (§7.2/D18):** `"0.37"` → `^0.37` → `<0.38.0`; `"0.4.4"` → `^0.4.4` → `<0.5.0` (0.x minor is the caret breaking digit). The 0.38.0 / 0.5.0 bumps are outside both ranges → hand-edit required, `cargo update` insufficient. `codex32 = "=0.1.0"` (`:34`) unchanged (ms-codec 0.5.0 keeps codex32 =0.1.0). md-cli/ms-cli exact pins (`md-codec = { …, version = "=0.37.0" }` `Cargo.toml:28`; `ms-codec = { …, version = "=0.4.4" }` `Cargo.toml:20`) also hand-edit. All correct. **Resolved.**

**Three-variant exit-2 checklist — ADDED (§7.3 lines 467-470):** `PayloadTooLongForSingleString` (encode → exit 2), `ChunkSymbolCountOutOfRange` (correcting-decode → exit 2), `StringSymbolCountOutOfRange` (non-correcting-decode → exit 2), all alongside the `TooManyErrors` exit-2 group at `error.rs:516`; compiler flags any miss (exhaustive `md_codec_exit_code`). ms side: explicit `InconsistentShareSet → exit 2` arm at `error.rs:417` (no compiler — paired-PR discipline). Line-cited and explicit. **Resolved.**

**Verdict: I2 RESOLVED.**

---

## Fold-introduced-drift checks (the dangerous part of a re-review)

1. **Variant-count consistency — CLEAN.** Grep for `two new|2 new|two additive|two variants|2 variants|two public` over the doc returns **nothing** (exit 1). The doc says "three"/"THREE"/"3 variants" at §5.2.3 (`:211`), §7.1 (`:407`), §7.3 (`:464`), and D17 (`:525`) — uniformly. §7.1, §7.3, §5.2.3, and D17 all agree on three. No place still says two. **Consistent everywhere.**

2. **I1 cap vs M4 boundary cap — DEFENSE-IN-DEPTH, NOT a conflict.** Traced the convergence: `decode_with_correction` (the M4 correcting path, `chunk.rs:502`) for a single non-chunked string ultimately calls `decode::decode_md1_string(&corrected_strings[0])` at `chunk.rs:613` → which calls `unwrap_string`. So the two paths DO converge on `unwrap_string` for the correcting path's *final* decode. BUT the M4 boundary guard (spec §5.2 bullet 2) sits at the TOP of `decode_with_correction`, BEFORE the residue/correction logic, so a >93 chunk is rejected with `ChunkSymbolCountOutOfRange` long before `:613`/`unwrap_string` is reached. The two are therefore **genuinely separate entry points** (`decode_md1_string` direct vs `decode_with_correction`), and the `unwrap_string` cap is redundant for the correcting path but load-bearing for the non-correcting direct path. No conflict; both fail-closed at the same `> 93` boundary. **`decode_with_correction` does call `unwrap_string` (transitively at `:613`) — the prompt's "verify it does NOT" resolves to: it DOES, but the M4 boundary cap pre-empts it, so the cap placement is correct and non-conflicting.**

3. **I1 cap vs the chunked (`reassemble`) path — NO legitimate-path regression.** `reassemble` (`chunk.rs:305`) calls `unwrap_string(s)` **per-chunk** at `:321` (each chunk individually ≤ 64 data + 13 = 77 ≤ 93), then concatenates per-chunk payload bytes and calls `decode_payload` on the FULL reassembled stream at `:376` (NOT via `unwrap_string`). So the I1 `> 93` cap (a) never fires on a legitimate per-chunk `unwrap_string` call, and (b) is never applied to the >93-symbol reassembled total. The fold STRENGTHENS coverage (the per-chunk codeword gate now also guards `reassemble`'s per-chunk decode) with zero false-reject on legitimate chunked input. **No regression introduced.**

4. **D8 / §6.2 M6 membership design — INTACT, unchanged by the folds.** §6.2 still reads: interpolate the secret from the **first k** shares (`interpolate_at(&k_set, Fe::S)`), then for each extra `parsed[k..]` assert `interpolate_at(&k_set, idx_j) == parsed[j]`, else `InconsistentShareSet`. The index-comparison detail (full canonical lowercased `Codex32String`), the cost argument ((n−k) interpolations, k≤9/n≤31), and the three resolved edge cases (n==k empty loop; first-k-consistent-extras-diverge caught; fully-internally-consistent-wrong-k-subset out-of-scope) are byte-identical to the form Round-1 PROVED correct and non-false-positive against codex32-0.1.0's `interpolate_at` (header-agreement + input-index short-circuit + Lagrange of all payload symbols) and BIP-93's linearity property. The I2 citation corrections (`:235`/`:242`/`:243`/`:263`) touch only LINE NUMBERS in §6.1's "locked facts", not the §6.2 design logic. **No weakening.**

5. **No remaining open question.** Grep for `open question|unresolved|TBD|TODO|\?\?\?|FIXME|undecided` returns only the two self-referential hits at `:17` and `:505` (both asserting there are NO open questions). The Resolved-decisions table D1–D18 is internally consistent: D17 (I1 fold) and D18 (I2 fold) agree with §5.2.3, §7.1, §7.3, §7.2; D14's md-compile-forced / ms-silent-fallthrough split is consistent with §6.4 + §7.3; D12 SemVer (md-codec MINOR, ms-codec MINOR, CLIs PATCH, toolkit PATCH) agrees with §7.1. No contradiction between any two decisions.

---

## MINOR

### M-min-1 (carried from Round 1) — own-account "0.63.0" figure still speculative; NOT folded.
The spec still carries "the unmerged `feature/own-account-subset-search` plans to renumber to 0.63.0" at `:427` and D16 (`:524`). **Live:** that branch's `crates/mnemonic-toolkit/Cargo.toml:3` is **`version = "0.60.0"`** (cut off v0.60.0, never renumbered); toolkit `origin/master` is 0.62.0. The "0.63.0" figure remains unsubstantiated, exactly as Round-1 flagged. This is **Minor, non-gating**: the mitigation (cut the pin-bump PATCH off `origin/master` 0.62.0 → 0.62.1, rebase own-account after) is sound regardless of own-account's eventual number. Recommend the plan-doc soften "0.63.0" to "the next available number ≥ 0.62.1 at its merge time," but this does NOT block GREEN. (Round-1 listed this as a recommended fold but correctly did not gate on it.)

### M-min-2 — phantom `§5.2.2` cross-reference anchor (new, trivial).
§5.2 bullet at `:179` references "the **`chunk.rs` boundary** (§5.2.2)", but there is no `### 5.2.2` header — the content it points to is the §5.2 body (bullet 2, `:182-188`). Harmless internal-anchor imprecision (the §5.2.3 header DOES exist for the I1 leg). Recommend renumbering the body bullet to a real §5.2.2 sub-header or dropping the "(§5.2.2)" pointer in the plan-doc. Non-gating cosmetic.

---

## Cross-checks carried forward (re-confirmed, not re-litigated)

- All M4 `bch_decode.rs` internal-floor citations re-verified exact: `chien_search` `:284`, loop `:293`, `decode_regular_errors` `:403`, deg gate `:416`, position map `k = data_with_checksum_len - 1 - d` `:437`, `const BETA` `:148` / order-93 comment `:146` / test `beta_has_order_93_regular` `:477`.
- All three findings still REPRODUCE on live origin (H6/M4/M6 unchanged by the citation folds).
- md `md_codec_exit_code` is the ONE compile-forced lockstep (exhaustive, NO `_ =>`, md-codec NOT `#[non_exhaustive]`); ms side is the SILENT exit-1 fallthrough (`_ => 1` at `:419`, ms-cli wildcard `:246`, `friendly_ms_codec` `_ =>` fallback `friendly.rs:147`). D14 split confirmed both halves.
- SemVer, publish→pin order, manual/GUI-schema non-obligation, FOLLOWUP-slug freshness: all unchanged and correct.

---

## Required folds before proceeding

**None gating.** Both Importants resolved; verdict is GREEN. Two non-gating Minors (M-min-1 own-account "0.63.0" soften; M-min-2 phantom §5.2.2 anchor) MAY be swept when the spec content is lifted into the per-track plan-docs, but neither blocks advancing to the plan-doc R0 phase.

---

## Verdict

`R0 ROUND 2: 0C / 0I` — **GREEN.**
