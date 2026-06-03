# ms1-slot — Plan R0 Review (round 0)
**Verdict:** RED (0C / 2I)

The plan is architecturally faithful to the R0-GREEN SPEC, every cited type/API exists with the assumed signature, the helper snippet compiles against real types, and the test set is constructible (the mnem-English encode hedge is unnecessary — `ms_codec::encode` accepts `Mnem{language:0}`). Two Important gaps, both in Phase-2 descriptor-loop task prose; both foldable, neither relitigates a fixed decision.

## Critical (0) / Important (2) / Minor (5)

### Important

**I1 — Site-13 + site-15 descriptor loops use a SINGLE shared `CosignerKeyInfo` push fed by a 4-tuple; the plan's "push at :1422-1430/:859-865, currently language: None" wording implies a per-arm push that does not exist — non-implementable as written.** `bundle.rs:1305-1430` binds `let (xpub, fingerprint, path, ent_opt) = if … else …;` (each arm RETURNS a 4-tuple: Phrase `:1344`, Xpub, Entropy `:1407`, else `:1409`); `CosignerKeyInfo` is built ONCE at `:1422-1430` (`language: None` `:1428`), then `keys.push`/`fingerprints.push` at `:1432-1439` consume the tuple. `verify_bundle.rs:776-855` is structurally identical (tuple `:776-780`, single push `:859-867`, `language: None` `:865`). To make the Ms1 arm emit `language: res.emit_language` while the other arms stay `None`, the implementer must WIDEN each loop's tuple to a 5-tuple carrying `emit_lang: Option<bip39::Language>` (None in existing arms, `res.emit_language` in the Ms1 arm), then write `language: emit_lang` at the single push. The plan must state this for BOTH site 13 (Task 2.3) and site 15 (Task 2.4). (SPEC §5/§2 are decision-correct; only the PLAN task prose omitted the tuple mechanic.)

**I2 — Module registration must be `main.rs`, not "lib.rs/main.rs".** The crate is lib+bin: `main.rs` declares private `mod slot_input; mod derive_slot; mod language; …` (`main.rs:11,16,18,…`); `lib.rs` declares `pub mod secret_taxonomy;` etc. `cmd/bundle.rs` reaches siblings via `crate::slot_input`/`crate::derive_slot`/`crate::language` (binary-internal) and lib items via `mnemonic_toolkit::…`. The `slot_ms1` helper uses `use crate::error::ToolkitError; use crate::language::{…};` and is consumed as `crate::slot_ms1::resolve_ms1_slot` — those `crate::` paths exist ONLY in the binary crate, so `mod slot_ms1;` MUST go in `main.rs` (declaring it in lib.rs fails to compile).

### Minor
- **M1** — resolve_slots Entropy-arm cite `:606-655` → actual `:608-657`.
- **M2** — `synthesize.rs:671` doc-comment says `language` is populated ONLY at the import-json arm; this cycle adds 3 more sites — update it.
- **M3** — R0-M2 optional `error.rs:285` doc-append (`| "language-conflict"`) already in Task 3.1 Step 2; accurate.
- **M4** — Task 2.2 struct-literal: bind `_entropy_pin` from `&res.entropy[..]` to a LOCAL before moving `res.entropy` into `entropy:` (left-to-right field eval). Existing arms do this (`bundle.rs:527/647`).
- **M5** — Task 1.4 canonical test: supply a well-formed `@1` (`@1.xpub=`) so the cmd isn't rejected for missing `@1`; gate fires on `@0` first.

## SPEC→plan coverage matrix
All §-requirements, all 16 sites, all 11 tests map to tasks. T1→1.1; T2→1.3; T3(byte-id)→2.2a; T4(mnem)→2.2b; T5(conflict both)→2.2c+2.4b; T6(share)→2.5a; T7(round-trip)→2.4a; T8(mnem-English)→2.5b; T9(`[Ms1,Path]`+`[Seedqr,Path]`)→1.4; T10(descriptor multisig)→2.3; T11(`--self-check`)→2.5c. No GAP.

## Citation verification
ALL ACCURATE except M1 (`:606-655`→`:608-657`) and minor ≤2-line range drifts (immaterial; §10 re-grep mandate). Confirmed: slot_input enum/from_token/as_str/is_secret_bearing/error-string/exempted_v0_19_0/is_legal_set(`[Phrase,Fp,Path]`/`[Phrase,Path]` at `:347-348`)/macro/parity-test; secret_taxonomy.rs:111 (+ SECRET_NODE_TYPES_ARGV already has "ms1" at :100, unrelated); bundle.rs catch-all `:709`, canonical gate `:1151-1160`, default-path-override `:1222-1232` (`!Phrase&&!Seedqr` `:1228-1230`), descriptor loop `:1305-1430` (anno_path `:1294-1301`, push `:1422-1430`, `language:None` `:1428`), `--slot` doc `:94-113` (entropy line `:102`); verify_bundle default-path `:715-723`, descriptor loop `:776-849` (tuple `:776-780`, anno_path `:766-774`, push `:859-865`, else→DescriptorReparseFailed `:849`), whole-card compares `:1245`/`:1639`; synthesize emit `:298-306`/`:831-839`, ResolvedSlot.language `:671`, CosignerKeyInfo alias `:219`; derive_slot.rs:42-71 (both `pub(crate) -> Result<DerivedAccount,ToolkitError>`); error.rs SlotInputViolation `:284-288` (kind `&'static str`), exit-2 `:519`, `--json` `:797`, `From<ms_codec::Error>` `:823`, `ms_codec_exit_code IsShareNotSingleString=>2` `:369-370`; language.rs wire_code_to_bip39 `:96` (fallible), bip39_to_wire_code `:120`, `From<CliLanguage>` `:153`, `CliLanguage: Default(#[default] English)`; friendly.rs:110-114; convert.rs:1464 (ToolkitError::from); ms-codec decode.rs:42, payload.rs:28-57. Cargo.toml:3=0.40.0; FOLLOWUP not yet present.

## Code-correctness findings
- Helper `resolve_ms1_slot` COMPILES against real types (decode→map_err(From), wire_code_to_bip39?, CliLanguage Default+Into, SlotInputViolation literal, bip39::Language PartialEq, `_=>` arm).
- resolve_slots Ms1 arm: `ResolvedSlot` fields match the plan literal; `into_parts()` 5-tuple `(entropy, fingerprint, xpub, xpriv, path)` destructured as Entropy arm does (`bundle.rs:646`); `pin_pages_for` in scope (`bundle.rs:14`); ordering hazard → M4.
- Module registration → I2 (main.rs).

## TDD + phasing findings
- Test 2.2(a) byte-identity ACHIEVABLE (both arms language:None → identical stdout, 5 lengths). 2.2(b) mnem-japanese ACHIEVABLE (`encode(ENTR, Mnem{language:1,…})`). 2.5(b) mnem-English ACHIEVABLE — `ms_codec::encode` accepts `Mnem{language:0}` (`envelope.rs:447-464` proves it); the plan's "or document if refused" hedge is moot; assert toolkit emits the entr card (English→Entr collapse). 2.5(a) share→exit 2 + friendly prose. 2.4(a) round-trip is exactly what makes I1's tuple-threading load-bearing (leaving site-15 `language:None` → mnem round-trip card mismatch → test FAILS).
- Phasing SOUND/severable: P1.4 canonical-gate test passes at exit 2 via the widened gate WITHOUT the P2 arm (gate fires before binding loop); after P1, bare `[Ms1]`/`[Ms1,Path]`-non-canonical fall to `else→BadInput` (exit 1) — acceptable intermediate. Widening does NOT break existing seedqr tests (their `code(1)` cases are template-mode decode/stdin errors, not canonical `[Seedqr,Path]`).

## Verdict rationale
Citations ACCURATE (≤2-line drifts immaterial); helper + template-arm literal compile against confirmed types; all 11 tests map + constructible; phasing severable, P1 breaks nothing. Blockers are Phase-2 task-prose defects: I1 (shared single-push tuple misdescribed as per-arm → un-stated tuple-widening) + I2 (slot_ms1 must register in main.rs). Both would cost an implementer a failed cycle. **Gate: RED. Fold I1 into Tasks 2.3/2.4 (5-tuple + emit_language + single push), I2 into Phase-2 Files + Task 2.1 (main.rs); re-dispatch.**
