# Review: toolkit bundle slots 0-31 docs (PR #163), class D

Reviewer: Claude Sonnet 5.5 (claude-sonnet-5-5). Range b7b248dc..120f81b2. Re-review: no (first review; 0 findings from a prior fix commit).
Method: read the diff; built the worktree binary; ran signet probes (inputs /scratch/cache/tk-slots-logs/d16.txt, d20m.txt, /scratch/cache/grade-mk-chunk-set-id-bundle/run/d32.txt); grepped technical-manual, manual and crates/*/src; read mnemonic-gui v0.63.0 (tag checkout at /scratch/cache/tk-slots-gui-pinned).

**⚠ Two Important findings (I1: "only the template paths" wrong for `--md1-form template`; I2: stale leading-16-bit claims).**

## Verified true
- `derive_mk1_chunk_set_id_for_slot` = base ^ slot (synthesize.rs:97). Bits 4..0, leading 15 bits (19..5) fixed: correct. Worked examples checked by hand: 29bdc^0x10=29bcc, 18ed4^0x1f=18ecb. Re-ran `--md1-form template` on d20m: md1 `505b`, @15 `505b0`, @16 `505af` (4th char differs by lowest bit only), as the new text says.
- 41-bundle-anatomy.md:140 and :195 edits, 42-anti-collision-invariants.md:17 and :19 numbers (15 bits, 2^-15, 32,768) are consistent with the code.
- Grade item 1a, `cmd/bundle.rs` ~:1397 comment: ACCEPT. "slot (0..=31) XORs into bits 4..0 ... for slots 16..=31 so does the lowest bit of the 4th" is true.
- Grade item 1b, `cmd/bundle.rs` :1215 `n!` comment: ACCEPT. 20! = 2432902008176640000 < u64::MAX, 21! overflows; run: 20-key `--md1-form template` prints `20! = 2432902008176640000`; 32-key d32.txt prints `32! (astronomically many)`. (21..=32 needs a non-`multi` shape, since miniscript refuses `multi` thresholds above 20; the d32 shape does reach it.)
- Grade item 2, manual-gui 42-bundle.md:567 "drag-value 0..15": the sentence is TRUE to pinned v0.63.0: `src/form/slot_editor.rs:36` is `egui::DragValue::new(&mut row.index).range(0u8..=15)`. Not a doc defect. The widget cannot reach slots 16..31, which is a mnemonic-gui item (proposed entry below).

## Findings

### I1 (Important): "only the template paths stop at 16 cosigners" is false for `--descriptor --md1-form template`
- Where: `docs/technical-manual/src/40-bundle-formation/42-anti-collision-invariants.md:17` ("only the template paths stop at 16 cosigners") and `crates/mnemonic-toolkit/src/synthesize.rs:89-90` ("only the template paths cap at 16 cosigners").
- Failure: run as printed, `mnemonic bundle --network signet --descriptor-file /scratch/cache/tk-slots-logs/d20m.txt --md1-form template` mints 20 slots (`@16 mk1:505af` ... `@19 mk1:505ac`; md1 `505b`). That is a "template" path (`--md1-form template`, `synthesize_template_descriptor`) with no 16 cap. The same PR's bundle.rs:1215 comment says exactly this (n up to 32 via `--descriptor --md1-form template`), so the PR contradicts itself. The 16 cap (`synthesize.rs:621`, `:876`) applies only to `--template wsh-multi...tr-sortedmulti-a` (the `--template` multisig paths).
- Fix text (both places): replace "only the template paths stop at 16 cosigners" with "only the `--template` multisig names (`wsh-multi` ... `tr-sortedmulti-a`) stop at 16 cosigners; `--descriptor` bundles, with either `--md1-form`, mint every slot up to 32". In synthesize.rs: "only `--template` multisig caps at 16 cosigners".

### I2 (Important): leading-16-bit claims left in the technical manual and in descriptor-path comments
- `docs/technical-manual/src/60-back-matter/61-glossary.md:89`: "The leading 16 bits agree across all three cards from one bundle." False for slots 16..31 (the md1 `29bd` vs `@16 mk1:29bcc` case). Fix: "The leading 15 bits agree across all the cards of one bundle (all 16 for single-sig and for multisig slots 0..15; a slot 16..31 card flips the lowest bit of the fourth hex char). Discussed §IV.2."
- `crates/mnemonic-toolkit/src/synthesize.rs:545-546` (inside `synthesize_descriptor`, cap 32): "preserves the leading-16-bit bundle-binding prefix". Fix: "preserves the leading-15-bit bundle-binding prefix".
- `crates/mnemonic-toolkit/src/synthesize.rs:1139-1140` (inside `synthesize_template_descriptor`, same uncapped n; run above): same wording; same fix.
- `synthesize.rs:736` is in `synthesize_multisig_watch_only` (cap 16 at :621) and `:2065` is a 2-cosigner test, so both are true; left alone (optional: say "15" there too for consistency).

## Minor / Nit
- N1: `crates/mnemonic-toolkit/src/synthesize.rs:94` doc line is 109 columns (the other new lines wrap near 80); rewrap the last paragraph.
- N2: `41-bundle-anatomy.md:195` says "For a multisig bundle the shared prefix is the leading 15 bits"; for slots 0..15 it is 16. Say "at least the leading 15 bits".
- M1: `docs/manual-gui/src/40-mnemonic/42-bundle.md:567` is true but silent that the CLI takes `@0..@31`; optional addition: "(the CLI accepts up to `@31` for descriptor bundles; see the GUI follow-up)". Do not add until the mnemonic-gui item is decided.

## Proposed mnemonic-gui FOLLOWUPS entry (not graded C/I; mnemonic-gui parked 2026-11-09)
### `gui-slot-editor-index-0-15-vs-31` — slot-editor index spinner stops at @15; `mnemonic bundle --descriptor` takes @0..@31
- Surfaced: toolkit PR #163 review (review-tk-bundle-slots-docs).
- What: mnemonic-gui v0.63.0 `src/form/slot_editor.rs:36` uses `egui::DragValue::new(&mut row.index).range(0u8..=15)`, so the GUI cannot enter `--slot @16..@31` for a 17..32-key descriptor bundle that the CLI accepts. Fix: widen to `0u8..=31` (check `SlotState.rows` u8 type and the render_emit/validation path) and update manual-gui `42-bundle.md:567` ("drag-value 0..31") in lockstep.
- Status: `deferred` (gui parked). Tier: `mnemonic-gui`.

C / I / M / N: 0 / 2 / 1 / 2
