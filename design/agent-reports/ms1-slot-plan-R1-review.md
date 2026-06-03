# ms1-slot — Plan R0 Review (round 1)
**Verdict:** GREEN (0C/0I)

Both round-0 Importants CONFIRMED-FIXED against source; fold introduced no scope drift. Three residual Minors, all compiler-caught within the per-task TDD loop and conceptually self-correcting — none rises to Important. Cleared to begin Phase-1 implementation.

## Critical (0) / Important (0) / Minor (3)

## Fold verification

### I1 — CONFIRMED-FIXED (correct + implementable)
- `bundle.rs:1305-1430`: loop binds an INFERRED 4-tuple `let (xpub, fingerprint, path, ent_opt) = if … else …;` (arms Phrase `:1344`, Xpub `:1374`, Entropy `:1407`, else→BadInput `:1409`); single `CosignerKeyInfo` push at `:1422-1430` (`language: None` `:1428`); `keys.push`/`fingerprints.push` (`:1432`,`:1436`) consume the tuple BY REFERENCE → widening to a 5-tuple does NOT break them. Plan's instruction (append `, None` to existing arms; Ms1 arm returns `…, res.emit_language`; `language: emit_lang` at the one push) is correct.
- `verify_bundle.rs:776-867`: SAME structure with an EXPLICIT annotation `(BipXpub, Fingerprint, DerivationPath, Option<Vec<u8>>)` `:776-780`; single push `:859-867` (`language: None` `:865`); arms Phrase||Seedqr `:781`, Xpub `:823`, else→DescriptorReparseFailed `:849`. Plan cites the widen site as `:776-780` (the annotated binding) → directs extending the annotation (M-C).
- `ent_opt` is `Option<Vec<u8>>` (PLAIN) in BOTH loops (`bundle.rs:1407`/`:1420`, `verify_bundle.rs:780`/`:857`); Ms1 arm's `res.entropy: Zeroizing<Vec<u8>>` → 4th element = `Some((*res.entropy).clone())`. Folded into Tasks 2.3/2.4.
- A `let mut emit_lang = None;` approach would also work (esp. for the inferred bundle tuple), but the 5-tuple is CORRECT and natural for the annotated verify tuple. Not a blocker.

### I2 — CONFIRMED-FIXED conceptually (line drift → M-B)
Fold correctly pins `mod slot_ms1;` to `main.rs` (binary crate). `main.rs` declares `mod error;` `:14`, `mod language;` `:18`, `mod slot_input;` `:27`, `mod synthesize;` `:28` — so `crate::error`/`crate::language` resolve and `crate::slot_ms1::resolve_ms1_slot` is reachable from `cmd/*`. (Plan's `:11` was wrong — `:11` is `mod derive_slot;`; corrected to `:27` in this round.)

### Minors M1/M2/M4/M5 — all CONFIRMED-FIXED
- M1: Task 2.2 cites Entropy arm `:608-657` ✓ (`else if …Entropy` `:608`, push closes `:657`).
- M2: Task 2.2 includes updating `synthesize.rs:660-671` `language`-field doc (currently "populated ONLY at import-json arm") ✓ appropriate.
- M4: Task 2.2 binds `entropy_pin` local before moving `res.entropy` ✓ matches Entropy arm `:647`.
- M5: canonical fixture `wsh(sortedmulti(2,@0,@1))` (`cli_non_canonical_descriptor.rs:121,201`) needs a well-formed `@1`; gate `:1142-1162` is per-`@i` before the binding loop so `@0` conflict fires exit 2 regardless of `@1` ✓. `[Seedqr,Path]` exit-1→exit-2 normalization claim ACCURATE.

## New-drift scan
- 5-tuple widening correctly scoped to the TWO descriptor loops ONLY; Task 2.2 (template resolve_slots) uses per-arm `out.push(ResolvedSlot{…})` and sets `language: res.emit_language` directly — no conflict.
- M4 pin-ordering consistent with the existing Entropy arm. Plan still complete (all 11 tests, 16 sites). No new non-implementable prose beyond the Minors.

## Minor (residual, non-blocking; compiler-caught)
- **M-A (inherited from SPEC, now clarified in plan):** Tasks 2.3/2.4 descriptor loops have NO bare `pass`/`network` locals (those exist only in `resolve_slots`, `:623` + param). Use `args.network` + the per-arm passphrase from `args.passphrase`. Folded as a parenthetical in Tasks 2.3/2.4. Compiler-caught regardless.
- **M-B (fixed this round):** `mod slot_input;` is `main.rs:27`, not `:11`. Corrected.
- **M-C (fixed this round):** verify_bundle's tuple is explicitly annotated at `:776-780`; widening must extend the annotation with `Option<bip39::Language>`. Noted in Task 2.4. Compiler-caught.

## Verdict rationale
Both Importants fixed against source (I1 tuple-widening models the single-shared-push reality; I2 main.rs registration resolves all `crate::` paths). All actionable Minors correct + implementable. No scope drift; widening confined to the two descriptor loops. The residual Minors (M-A accessor shorthand, M-B/M-C line/annotation, all corrected or compiler-caught) would not cost a failed cycle. **Gate: GREEN (0C/0I). Cleared to begin Phase-1 implementation.**
