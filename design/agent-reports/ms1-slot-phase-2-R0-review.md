# ms1-slot — Phase 2 R0 Review
**Verdict:** GREEN (0C/0I)

Phase 2 diff `git diff 5c5dbc4..ef14fe3` (6 files: `slot_ms1.rs` NEW, `cmd/bundle.rs`, `cmd/verify_bundle.rs`, `main.rs`, `synthesize.rs`, `tests/cli_ms1_slot.rs`). Gate independently re-confirmed by the controller: `cargo test -p mnemonic-toolkit --no-fail-fast` → 0 FAILED; `cargo clippy --all-targets -- -D warnings` → exit 0.

## Critical (0) / Important (0) / Minor (2)

## A — `slot_ms1::resolve_ms1_slot` helper
- `Payload::Entr` (`slot_ms1.rs:46-50`): `derive_language = flag_language.unwrap_or_default().into()` (English default), `emit_language = None`. CORRECT.
- `Payload::Mnem{language: wire, entropy}` (`:52-75`): `wire_lang = wire_code_to_bip39(wire)?`; conflict gate `let flag_lang: bip39::Language = flag.into(); if flag_lang != wire_lang → Err(SlotInputViolation{kind:"language-conflict"})` — compares in `bip39::Language` space both sides. Else `derive_language=wire_lang, emit_language=Some(wire_lang)`. CORRECT.
- `#[non_exhaustive]` Payload → `_ =>` arm present (`:79-81`) → `BadInput` clean.
- Secret hygiene: `entropy: Zeroizing<Vec<u8>>`; NO `#[derive(Debug)]` (conflict test uses `match`, not `expect_err`); error message has only language names + slot index, no entropy. CONFIRMED.
- `mod slot_ms1;` in `main.rs:28` (binary crate); `crate::error`/`crate::language` resolve. CONFIRMED.

## B — template `resolve_slots` Ms1 arm
- `else if …Ms1` at `bundle.rs:658` BEFORE catch-all `else` (`:757`); mirrors Entropy arm incl. `multisig_acct_path` branch using the loop's real `language`/`passphrase`/`network`/`template`/`account`.
- **M4:** `let entropy_pin = Some(Rc::new(pin_pages_for(&res.entropy[..])));` (`:696`) bound BEFORE the ctor; `entropy: Some(res.entropy)` moves inside the ctor — no move-then-borrow, pin captures the surviving buffer (matches Entropy arm `:646-657`).
- `language: res.emit_language` (`:703`); `into_parts()` destructure matches `(entropy, master_fingerprint, account_xpub, account_xpriv, account_path)`. Byte-identity with `@N.entropy=` holds (Entr→English→same derive+card).

## C — descriptor-loop Ms1 arms (5-tuple widening, highest risk)
- **`bundle_run_unified_descriptor` (`:1362-1524`):** tuple widened to `(BipXpub, Fingerprint, DerivationPath, Option<Vec<u8>>, Option<bip39::Language>)` (`:1362-1367`). Every pre-existing arm got `, None` appended, other 4 elements UNCHANGED: Phrase `:1405`, Xpub `:1435`, Entropy `:1468`. New Ms1 arm `:1469-1498` returns `…, res.emit_language`. Shared push `:1513-1524` sets `language: emit_lang`. Derives via `derive_bip32_from_entropy_at_path(&res.entropy, &passphrase, res.derive_language, args.network, &anno_path)` — REAL accessors. `master_fp` consistent with Phrase/Entropy (no F-fp regression). No existing-arm behavior changed.
- **`verify_bundle.rs` (`:782-906`):** explicit annotation extended to 5 elements (`:787`); Phrase||Seedqr `:829`, Xpub `:854` got `, None`; Ms1 arm `:855-884` returns `…, res.emit_language`; push `:904` sets `language: emit_lang`. Uses `args.network`+`args.passphrase`. NO Entropy arm added (SPEC-R0-I1 honored).

## D — output-symmetry (load-bearing)
- All THREE binding sites set `ResolvedSlot.language = res.emit_language` (bundle `:703`/`:1522`; verify `:904`).
- Synth emit rule (`synthesize.rs:298-301` multisig, `:835-843` single-sig): `emit_lang = language.unwrap_or(run_language); English→Entr else→Mnem{bip39_to_wire_code(lang)}`. `bip39_to_wire_code ∘ wire_code_to_bip39 == id` (language.rs test `:197`) → a mnem ms1 with wire code `w` re-emits wire code `w` → byte-identical card; verify whole-card `==` (`verify_bundle.rs:1284`/`:1639`) closes the round-trip.
- Test 2.4a discriminates entr-vs-mnem by DECODING (`ms_codec::decode → match Payload`, `:484-488`), NOT a vacuous string prefix (the entr+mnem shared `ms10entr` TAG prefix is correct; kind = payload prefix byte). Re-feeds the emitted card, asserts `result: ok` for entr (English) + mnem (Japanese).

## E — synthesize.rs
Only the `ResolvedSlot.language` doc-comment (`:660-674`) updated (M2); emit rule unchanged. No behavioral change.

## F — test quality (cli_ms1_slot.rs)
Load-bearing, none vacuous: 2.2a byte-identity real-stdout across 5 lengths (`:137-181`); 2.2b mnem key-match + decode-assert `Mnem{language:WIRE_JAPANESE}` (`:186-255`); 2.2c+2.4b language-conflict exit 2 in BOTH binaries; 2.4a round-trip `result: ok` entr+mnem decode-discriminated; 2.5a share-rejection exit 2 + `"ms-shares combine"` from a real `encode_shares` 2-of-3 set (`:617-644`); 2.5b mnem-English edge (precondition `Mnem{language:0}` + emitted-card `Entr` assert); 2.5c `--self-check` Japanese mnem. Helper unit tests cover entr/mnem/match/conflict/share.

## G — regressions / clippy / secrets
- All ms-codec 0.4.0 APIs used exist (`encode`, `encode_shares`, `Threshold::new`, `Tag::ENTR`, `decode`, `Payload`, `MNEM_LANGUAGE_NAMES`).
- No production unwrap/expect on attacker input (`decode().map_err()?`; `.expect("contains() asserts presence")` guarded by the matching `subkeys.contains()`).
- `@N.ms1=` value secret + `Ms1.is_secret_bearing()==true` → argv-leak advisory + `@N.ms1=-` stdin sentinel inherit by construction.
- Transient `ent_opt` clone re-wrapped to `Zeroizing` (bundle `:1511`/verify `:894`) — identical to pre-existing Phrase/Entropy pattern (third-party-blocked, documented). Not new.

## Minor (non-blocking)
- **M1:** descriptor/verify `else→` error strings carry pre-existing "v0.4.2"/legacy phrasing, don't mention ms1-now-supported — pre-existing prose, not introduced here.
- **M2:** end-to-end coverage exercises only Japanese+English; language.rs round-trip covers all 10 wire codes at unit level → gap cosmetic.

## Verdict rationale
All Phase-2 risks verified: helper language policy correct (bip39 space), `_ =>` arm present, secret hygiene holds, M4 ordering correct; both descriptor 5-tuple widenings consistent (every arm `, None`, annotations extended, pushes flipped to `language: emit_lang`, no Entropy arm in verify); all three sites set `ResolvedSlot.language`; emit rule unchanged closes the byte-identical round-trip via the verified wire-code involution; tests load-bearing + decode-discriminated. Gate green (0 failed, clippy clean). **GREEN — proceed to Phase 3.**
