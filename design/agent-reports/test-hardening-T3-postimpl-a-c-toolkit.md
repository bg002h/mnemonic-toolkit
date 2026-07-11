# Post-impl whole-diff R0 — T3-a + T3-c toolkit — Fable, adversarial

**Persisted verbatim per CLAUDE.md.** Scope: new `crates/wc-codec/tests/wire_golden.rs` (3 tests) + `crates/mnemonic-toolkit/src/word_card_adapter.rs` (+17 test-only, `assert_eq!(cp.payload_bits, 644)` at :235). The implementer under-reported (degenerate return) → every RED-proof re-reproduced by execution here.

## 1. Green
`cargo test --workspace` **3814/0** (217 binaries; cold baseline + post-revert). `wire_golden` 3/3. T3-c `md1_canonical_carries_bit_precise_total_bits` green.

## 2. Goldens correct + both sides frozen (false-oracle check)
Both input `(payload, payload_bits)` consts AND output word-list consts frozen with provenance (wc-codec 0.1.1, toolkit SHA c00ed813); RAID uses a FIXED `ARRAY_ID_SEED` literal. Input consts independently re-derived (temp test, deleted): `mk_codec::decode(MK1).canonical_payload_bytes()`==MK1_PAYLOAD (672b); `md_codec::reassemble(MD1)`==MD1_PAYLOAD+644b; RAID_PAYLOAD_1/2 == cli_word_card derivations; RAID_PAYLOAD_0==MK1_PAYLOAD. End-to-end: `mnemonic word-card --from <fixture> --json` reproduces MK1_WORDS (96/96) + MD1_WORDS (92/92); MD1_PAYLOAD matches `md bytecode --json`. No false oracle.

## 3. T3-a RED-proofs (all re-reproduced, reverted sha256-clean)
SRC_MK1↔SRC_MD1 swap (pipeline.rs:60,62) → all 3 goldens RED, round-trips (pipeline 24, raid 20, sync 23, rs/field/pad) all green. CRC5_POLY→another primitive (sync.rs:43) → all 3 RED, round-trips green (raid's 3 f2b_legacy failures are frozen-oracle KATs, not round-trips — Minor 1). H0 src↔has-raid swap (pipeline.rs:216-217 + :394-395) → md1+RAID RED, mk1 green (all-zero fields, Minor 2). RS-parity correctly OMITTED (a bare `rs_parity` reversal REDs the round-trip suite — verified). Each golden REDs under ≥1 class; acceptance #1 holds (round-trips survive).

## 4. T3-c RED-proof + value
644 independently confirmed via `md bytecode --json` (`"payload_bits":644,"payload_bytes":81`; 644%8==4, last byte 0xb0 zero low-4 pad). RED reproduced: `:108` → `bytes.len()*8` → exactly 1 failure at the new pin (:236, left=648 right=644); both range asserts (:217-218) passed under the regression (the gap); other adapter + cli_word_card e2e green. Reverted; `:108` = `total_bits`.

## 5. Gates + NO-BUMP
`git diff` = word_card_adapter.rs +17 only; mlock.rs 0 bytes; 4 mutated src files sha256-match pre-review; temp test deleted. clippy `--workspace --all-targets -D warnings` exit 0.

## Findings
Critical: none.
**Important 1 — CI fmt gate RED on `wire_golden.rs`** (`rust.yml:53-80` `cargo +1.95.0 fmt --all -- --check`; rustfmt reflows the byte/word arrays). Fix mechanical + semantics-free: `cargo +1.95.0 fmt -p wc-codec` (wc-codec has no mlock.rs → g6 safe). **[FOLDED — dispatcher ran `cargo +1.95.0 fmt -p wc-codec`; wire_golden.rs now fmt-clean, 3/3 still pass, only that file affected, mlock untouched; self-verified — pure reformat, no value change.]**
Minor (no action): 1 — the doc-comment "no frozen word sequence pre-exists" overstates (`f2b_legacy_*` plates raid.rs:698-702 are frozen word KATs; new goldens still add encode-side/solo/m=8/non-byte-aligned pinning). 2 — mk1 golden blind to H0-swap (covered by SRC/CRC5). 3 — RAID seed is deterministic-from-fingerprints, not entropy-drawn (fixed-literal oracle still valid; not CLI-cross-checkable).

## VERDICT: OPEN (0C / 1I) → GREEN after the fmt fold
The sole Important is the rustfmt gate; all substance GREEN with execution evidence. Fmt folded + self-verified → T3-a/c ready to ship.

---
**SHIP (opus, 2026-07-10):** fmt Important folded (`cargo +1.95.0 fmt -p wc-codec`; re-verified 3/3 + fmt-clean + mlock untouched). T3-a/c GREEN. Bundling with T3-b (md, already GREEN) → ship T3 = toolkit + md direct-FF NO-BUMP.