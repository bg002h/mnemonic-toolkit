# R0 review — `SPEC_mk1_bip_alignment.md` — round 3 (Fable, adversarial, read-only, convergence)

**Reviewer:** Fable. Persisted verbatim per CLAUDE.md. Target: round-2 fold vs `mnemonic-key @ origin/main 1c9fbf7` (confirmed live). Round-2 report: `mk1-bip-alignment-spec-r0-round-2.md` (0C/2I/1M).
**Dispatched:** 2026-07-10 (MK SPEC, R0 round 3).

## Fold verification

### I-A (code-selection threshold) — FOLDED, NUMERICALLY CORRECT against code
SPEC :37 now states both sides. Every number re-verified against `crates/mk-codec/src/string_layer/bch.rs` @ 1c9fbf7:
- **Decoder:** `bch_code_for_length` (`bch.rs:117-124`) is verbatim `14..=93 → Regular`, `94..=95 → None`, `96..=108 → Long`; `None` → `Error::InvalidStringLength` at the decode call site (`bch.rs:677`). SPEC exact. Unit test `bch_code_for_length_rejects_94_and_95` (:785-787) pins the gap.
- **Encoder:** `encode_5bit_to_string` (`bch.rs:525-545`): `regular_total = data+13` / `long_total = data+15`; data 1–80 → regular, 81–93 → long, >93 → invalid. SPEC exact.
- **Cross-consistency:** 80+13=93 (regular ceiling), 81+15=96 (long floor), 93+15=108 (long ceiling); totals 94–95 encoder-unreachable, so the decoder's reserved gap is coherent.
- **V1 chunk 0 re-verified from the shipped corpus** (`v0.1.json`, `V1_bip48_mainnet_1_stub_with_fp`): chunk 0 = 111 chars = `mk1` + 108-symbol data → LONG, pre-checksum = 108−15 = 93 data symbols. The folded rule classifies it long (93 ∈ 81–93) — correct; the round-1/2 "≤93→regular" would have misclassified it. V1 chunk 1 = 77-symbol → Regular (confirms C-I4's mixed-code correction of BIP line 74).
- **C-M1 (:46)** now lists the 94–95 reserved-invalid gap with the `InvalidStringLength` mapping. ✓

### I-B (churn enumeration + Ripple :56 residue) — FOLDED COMPLETELY
- **F-A7 site list (:29)** includes `tests/vectors.rs:129-133`; ground truth confirmed `schema_metadata_pinned` asserts `doc["family_token"] == "mk-codec 0.2"` as a hardcoded literal.
- **Acceptance-1 (:63)** "exactly" churn set enumerates all five sanctioned edits incl. the literal update. `tests/vectors.rs:41 V0_1_SHA256`, `consts.rs:50 GENERATOR_FAMILY` cites live.
- **Ripple :56** parenthetical replaced; grep sweep for "only / exactly / no SHA / one vector" → no cycle-wide over-claim survives; Goal :10 scopes per-phase.

### M-i (cite precision) — PARTIALLY folded (Minor residual)
`crates/mnemonic-toolkit/Cargo.toml:33` at Ripple :58 ✓. C-I4 (:43) still cites bare `chunk.rs::split_into_chunks` — unambiguous, stays Minor.

## Post-fold drift check — core algorithm still EXACT
Re-walked SPEC :37 vs `bch.rs` @ 1c9fbf7: `POLYMOD_INIT = 0x23181b3` (:198); create = `polymod_run(hrp_expand(hrp) ‖ data ‖ [0;13])` XOR `MK_REGULAR_CONST` / `[0;15]` XOR `MK_LONG_CONST` (:305-345); verify = `polymod_run(hrp_expand ‖ data_with_checksum) == MK_*_CONST`; extraction big-endian top-5-first `(polymod >> (5*(12-i))) & 0x1F` regular, `(5*(14-i))` long; `hrp_expand("mk") = [3,3,0,13,11]` recomputed. I-A introduced no new contradiction.

## CRITICAL — none · IMPORTANT — none

## MINOR — 3 (none blocking; fold without re-dispatch)
- **M-1 (carried):** C-I4 :43 `chunk.rs` cite short-form → `crates/mk-codec/src/string_layer/chunk.rs:50`.
- **M-2 (new, phrasing):** encoder "pre-checksum data ≤80 → regular" nominally includes data=0, which the code rejects; write "1–80" or anchor the floor (decoder floor 14 is stated in the same sentence). Cosmetic parenthetical tangle at :37.
- **M-3 (new, housekeeping):** header :6 "R0 history: (round 1 pending)" stale — update.

**VERDICT: GREEN (0C/0I)** — I-A folded with numerically exact thresholds (V1 chunk 0 = 93 pre-checksum → LONG confirmed from the shipped corpus), I-B folded at all three sites, core §Checksum drift-free. The 3 Minors may be folded without re-dispatch. Implementation may begin.
