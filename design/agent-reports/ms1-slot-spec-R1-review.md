# ms1-slot — SPEC R0 Review (round 1)
**Verdict:** GREEN (0C/0I)

Both round-0 Importants are folded correctly AND the folds are independently verified against current branch source. Cleared to begin implementation per the mandatory-R0 standard.

## Critical (0) / Important (0) / Minor (2)

## Fold verification (each CONFIRMED-FIXED with source evidence)

### I1 — verify_bundle has no Entropy arm; helper called directly — CONFIRMED-FIXED
- SPEC §2 now drops "mirror the Entropy arm" and routes all three binding-loop Ms1 arms through `derive_slot::derive_bip32_from_entropy[_at_path]`. §5 site 15 + §9 carry the same framing.
- **Path in scope (verify_bundle site 15):** `verify_bundle.rs:766-774` binds `anno_path: bitcoin::bip32::DerivationPath` per-cosigner, immediately before the binding `if` at `:776`. The Phrase arm derives at exactly that path (`master.derive_priv(&secp, &anno_path)`, `:818`). `anno_path` is a plain `DerivationPath` — exactly what `derive_bip32_from_entropy_at_path(…, path:&DerivationPath)` accepts. Fully specified.
- **Comparison xpub:** Phrase arm produces `xpub: BipXpub` (`:821`), pushed at `:859`. `DerivedAccount::into_parts()` yields the xpub the same way the template Entropy arm uses (`bundle.rs:646`). Site 15's Ms1 arm extracts it identically.
- **Sites 10 + 13:** template Entropy arm already calls the helper (`bundle.rs:624-639`); `bundle_run_unified_descriptor` has `anno_path` in scope at `:1294-1301` (used by its Phrase arm `:1340` + Entropy arm `:1403`), so its Ms1 arm can call `derive_bip32_from_entropy_at_path(&entropy, pass, derive_language, network, &anno_path)`.
- `derive_slot.rs:42-71` signatures confirmed verbatim (`pub(crate)`, both `-> Result<DerivedAccount,…>`).

### I2 — Seedqr canonical-gate change reframed as error-class NORMALIZATION — CONFIRMED-FIXED
- Baseline verified: canonical gate `bundle.rs:1142-1162` (`if !is_non_canonical`, `has_phrase && has_path` `:1153`) → `[Seedqr, Path]` has `has_phrase=false` → PASSES → reaches binding loop `:1305-1430` (NO Seedqr arm) → `else→BadInput` (`:1408`), **exit 1** today. SPEC §4/§9/test-9 now state exactly this; widened gate makes it exit-2 `SlotInputViolation{kind:"conflict"}`.
- Test 9 documents the pre-fix exit-1 BadInput baseline (cites recapture-golden lesson). Correct.
- Ms1 load-bearing claim CONFIRMED (widening prevents a real silent mis-acceptance once site-13's Ms1 arm exists). CHANGELOG note truthful.

### Minor cite fixes — CONFIRMED correct
- §3 `bundle.rs:621-622` (`:621` unwrap_or_default, `:622` lang.into()). §2 cites wider `:621-637` for the helper-call block — also correct (`match &multisig_acct_path` spans `:624-639`).
- §3/§10 `synthesize.rs:299-306` (synthesize_descriptor collapse rule) + `:831-839` (synthesize_unified). Both confirmed; function labels correct.

## New-drift scan
- §2 reworded paragraph consistent with §3 + the helper return type (`derive_language: bip39::Language` + `emit_language: Option<bip39::Language>`); Entr→emit None, Mnem→emit Some(wire) after conflict gate — matches §3 verbatim.
- End-to-end hang-together CONFIRMED via `pub type CosignerKeyInfo = ResolvedSlot;` (`synthesize.rs:219`): the `language` field is the SAME field/type (`Option<bip39::Language>`, `:671`) for template (`ResolvedSlot`) and descriptor (`CosignerKeyInfo`) pushes. `synthesize_descriptor` reads `c.language.unwrap_or(run_language)` (`:298`), `synthesize_unified` reads `s.language.unwrap_or(run_language)` (`:831`); both English→Entr/else→Mnem. `language = emit_language` flows correctly in all three paths.
- Conflict-refusal wiring: `SlotInputViolation{kind:"language-conflict", message}` constructible (`error.rs:284-288`, kind:&'static str), reaches `--json` (`:797`), Display (`:755`), exit 2 (`:519`); "language-conflict" used consistently §3 line 74 + §8 test 5.
- ms-codec 0.4.0: `payload.rs:28-57` (`#[non_exhaustive]`, `Entr(Vec<u8>)` :44, `Mnem{language:u8 :53, entropy:Vec<u8> :55}`); `decode.rs:42`; share→IsShareNotSingleString→friendly.rs:110-114. All confirmed; `_ =>` arm requirement real.
- Surface/validation sites (`slot_input.rs` enum/from_token/as_str/is_secret_bearing/stdin-sentinel/exempted_v0_19_0/is_legal_set/macro/parity-test; `secret_taxonomy.rs:111`; `language.rs:96,120`; whole-card compare `verify_bundle.rs:1245,1639`) — all confirmed.
- No remaining unverified citation — every file:line in §§0-10 independently re-grepped.

## Minor (2) — non-blocking, optional polish
- **M1 (path prefix):** SPEC cites bare filenames; bundle/verify_bundle/convert live under `crates/mnemonic-toolkit/src/cmd/`; synthesize/slot_input/error/language/friendly/secret_taxonomy/derive_slot under `src/`. Pre-existing SPEC convention (round-0 same); §10 mandates re-grep at impl. Implementer should expect `src/cmd/` for bundle/verify/convert.
- **M2 (stale source doc-comment):** `error.rs:285` documents `SlotInputViolation.kind` as `"conflict" | "gap" | "invalid-set" | "duplicate-subkey"`. The new `"language-conflict"` value is constructible (&'static str, not an enum) needing NO error.rs edit, but an optional one-word doc append (`| "language-conflict"`) keeps the enumeration current. Not load-bearing.

## Verdict rationale
Both round-0 Importants folded correctly and verified against current branch source; both minor cite drifts corrected to the right ranges; fold introduced NO new drift (end-to-end confirmed incl. the CosignerKeyInfo=ResolvedSlot alias). Residue is two non-blocking Minors. **Gate: GREEN (0C/0I). Cleared to begin implementation.**
