# R0 review — `SPEC_mk1_bip_alignment.md` — round 2 (Fable, adversarial, read-only)

**Reviewer:** Fable. Persisted verbatim per CLAUDE.md. Target: round-2 fold vs `mnemonic-key @ origin/main 1c9fbf7` (confirmed live) and toolkit master. Round-1 report: `mk1-bip-alignment-spec-r0-round-1.md` (0C/3I/4M).
**Dispatched:** 2026-07-10 (MK SPEC, R0 round 2).

## Fold verification (each round-1 finding, against ground truth)

**I-1 (F-A7 token) — FOLDED, one completeness gap (→ I-B).** SPEC :26-29 rules ROLL → `"mk-codec 0.4"` inside the Phase-3 regen; the false "churn every pinned SHA" rationale is gone (`tests/vectors.rs:41 V0_1_SHA256` is the only pin); Q-10 convention kept with the honesty note; the three doc-site rewrites are mandatory + byte-consistent with an explicit "no 'optional comment'" prohibition. No stable-anchor residue. Grep-complete sweep of `"mk-codec 0.2"`: `consts.rs:50` ✓; `tests/vectors.rs:52` (comment) ✓; `v0.1.json:2` auto-rolls (gen writes `GENERATOR_FAMILY`, `gen_mk_vectors.rs:1095`) ✓; **`tests/vectors.rs:131`** — `schema_metadata_pinned` asserts the hardcoded literal `"mk-codec 0.2"`, NOT the const — **missing from F-A7's site list** (→ I-B).

**I-2 (scope/acceptance) — FOLDED at 2 of 3 sites.** Goal :10 enumerates the Phase-3 churn + per-phase scope ✓; acceptance-1 :63 scoped to "V1–V18 byte-identical" + sanctioned-churn ✓. **But Ripple :56's parenthetical "(Confirm: comments + BIP + one added vector only.)" survives verbatim** — the third cited site — contradicting the churn set. → I-B.

**I-3 (sibling-pin) — FOLDED, verified.** `mnemonic-toolkit/scripts/install.sh:41` pins `mk-cli-v0.12.0` (frozen; `sibling-pin-check.yml` live); `crates/mnemonic-toolkit/Cargo.toml:33` caret `mk-codec = "0.4.1"` → 0.4.2 zero-edit compatible; root `Cargo.toml` has no mk-codec dep. NO ACTION correct. Nit M-i: cite `crates/mnemonic-toolkit/Cargo.toml:33`.

**PATCH ruling — folded** at Ripple :57, acceptance-5 :67, Phasing :73 (0.4.2/0.12.1 lockstep, corpus `include_str!`-published).

**M-1 — folded** (:45 wire-scope; `mk-cli/src/slip132.rs` exists; FOLLOWUP `FOLLOWUPS.md:392`). **M-3 — folded** (:22; re-verified: BIP-93 init IS `0x23181b3`, recomputed fold of `hrp_expand("ms")=[3,3,0,13,19]` from 1 → exactly `0x23181b3`; equivalence note kept). **M-4 — folded** (`error.rs:114-117`; `FOLLOWUPS.md:365`; :263-269 source exact). **M-2 — folded, but half is WRONG → I-A.** Extraction-order half verifies (`bch.rs:312` `(polymod >> (5*(12-i))) & 0x1F` regular, `:344` long — cite :310-312 correct).

**Core algorithm drift check — EXACT.** SPEC :37 re-checked vs `bch.rs` @1c9fbf7: `POLYMOD_INIT 0x23181b3` (:287-293); create `hrp_expand ‖ data ‖ [0;13]/[0;15]` XOR `MK_*_CONST` (:304-345); verify `polymod(hrp_expand ‖ data_with_checksum)==MK_*_CONST` (:317-331); `hrp_expand("mk")=[3,3,0,13,11]` recomputed. No drift.

## CRITICAL — none

## IMPORTANT — 2 (both introduced/left by the fold; fail-closed)

### I-A. The M-2 code-selection threshold as folded is factually wrong — V1 chunk 0 disproves it (SPEC :37)
":37 …≤93 data symbols → regular; 94–108 → long…" is wrong under both readings. `bch_code_for_length` (`bch.rs:111-124`) dispatches on **TOTAL data-part length**: `14..=93` → Regular, **`94..=95` → reserved-invalid** (`InvalidStringLength`), `96..=108` → Long. The encoder (`encode_5bit_to_string`, `:525-545`) tries `data+13` regular else `data+15` long — pre-checksum: **data ≤80 → regular; 81–93 → long**. Counter-example: **V1 chunk 0 = 93 pre-checksum data symbols (8 header + 85 payload) → LONG (108 total)** (`bch.rs:1224-1225` + round-1 recompute), which the SPEC's rule classifies regular. This text is destined for the BIP's headline normative §Checksum — wrong-worked-example class (fail-closed, but the SPEC must be right before it's treated as authority). **Fix:** state both sides — encoder: pre-checksum ≤80 → regular (append 13), 81–93 → long (append 15), >93 invalid; decoder: total 14–93 → regular, 94–95 reserved-invalid, 96–108 → long — and add the 94–95 gap to C-M1's decoder-validity list. **Provenance: the "≤93/94–108" phrasing originated in my own round-1 M-2; the fold transcribed it faithfully. The error is mine, caught on re-verification.**

### I-B. I-2 fold incomplete: the "exactly"-billed churn enumeration misses a mandatory edit, and the Ripple :56 over-claim survives
(a) Acceptance-1's "exactly: …" churn set and F-A7's site list both omit **`tests/vectors.rs:129-133`** — `schema_metadata_pinned` asserts the hardcoded literal `"mk-codec 0.2"` (not the const), so Phase-3 regen hard-fails that test until edited; a strict reviewer applying the "exactly" list flags that edit as unsanctioned — the wedging mode round-1 I-2 identified. (b) Ripple :56's "(Confirm: comments + BIP + one added vector only.)" survives and contradicts the enumeration. **Fix (two phrases):** add `tests/vectors.rs:129-133` (family-token assert literal) to both the F-A7 site list and acceptance-1's churn set; replace the :56 parenthetical with "comments + BIP + Phase-3 vector-data churn per acceptance-1" (or delete it).

## MINOR — 1

- **M-i:** I-3 cite → `crates/mnemonic-toolkit/Cargo.toml:33`. C-I4's `chunk.rs::split_into_chunks` → `crates/mk-codec/src/string_layer/chunk.rs:50` (full path preferred).

**VERDICT: NOT GREEN — 0 Critical / 2 Important open** (I-A threshold restatement — funds-adjacent normative text; I-B churn-enumeration completeness + :56 residue). Both small surgical edits; fold and re-dispatch.
