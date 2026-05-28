# v0.37.7 — F5 fix (export-wallet --from-import-json path_raw bracket → m/path normalization) — architect review (retroactive R0)

Reviewer: feature-dev:code-reviewer (opus). Base `9a88a46`. F5 surfaced by cell H2 (sparrow→coldcard-multisig hop) in the new wallet-file cross-format convergence suite. User chose "investigate deeper first → fix → ship as PATCH v0.37.7."

## Verdict: GREEN — 0 Critical / 0 Important / 3 Minor (M1, M2 folded; M3 advisory)
Fix correct, complete, ripple-free for the stated bug class. Two minors folded (comment reframing + electrum cosmetic-shape unification via `format!("m/{}", s.path)`); one FOLLOWUP filed (path_raw convention overloading).

## Bug + fix
**F5:** `export-wallet --from-import-json --format {coldcard-multisig, jade, electrum, sparrow}` corrupted the cosigner derivation: coldcard/jade wrote the `Derivation: m/0'/0'` placeholder; electrum wrote an invalid bracketed `derivation` field its own importer rejected. **Root cause:** `mk1_card_to_resolved_slot` (`wallet_import/json_envelope.rs:282`) populates `ResolvedSlot.path_raw` as bracketed `[fp/path]` (overloaded convention), but the export emitters consume `path_raw` expecting a bare `m/...` path (`resolve_slots` convention). **Fix:** in `run_from_import_json` (`cmd/export_wallet.rs:629-640`), after `envelope_to_resolved_slots`, normalize each slot's `path_raw` to `format!("m/{}", s.path)` — boundary fix, scoped to export-wallet's from-import-json.

## Confirmations (reviewer-verified)
- **Correctness:** `s.path = card.origin_path` (`json_envelope.rs:293`) — correct cosigner origin. `DerivationPath` Display = bare (no m/); `format!("m/{}", s.path)` = canonical m/-prefixed.
- **No ripple:** the fix is in `run_from_import_json`'s body, NOT in shared `mk1_card_to_resolved_slot`. `bundle --import-json` (`bundle.rs:1534`) calls `envelope_to_resolved_slots` separately and is untouched. Direct path (`run` via `resolve_slots`) untouched. Byte-exact `cell_5_..._byte_exact` (`cli_export_wallet_coldcard.rs:206`) is the direct path — unaffected. Full suite 2457/0.
- **Completeness:** all 4 export `path_raw` consumers (`coldcard.rs:321`, `sparrow.rs:130`, `pipeline.rs:37`, `electrum.rs:163`) read AFTER the normalization. No emitter genuinely needs the bracket.
- **Test soundness:** convergence assertions on decoded key-material (triples + fp set + md1 tree-tag + multi-tag + threshold + network), C-neg guards non-vacuity, H3 uses sparrow (single-entry source, avoids F1 bitcoin-core split), coldcard unsorted→sorted documented via eprintln (by-design coercion). The triple `(xpub, fp, path)` is the load-bearing F5 detection vector via coldcard-multisig's `Derivation:` parser.

## Minor (folded)
- **M1 — Comment reframing (folded).** Reviewer noted the "would ripple into bundle --import-json" reasoning was slightly misframed — bundle's `path_raw` consumers are cosmetic JSON envelope fields, not card-emission. Comment rewritten to "scoped to this fn; bundle's consumers are cosmetic and unaffected." FOLLOWUP filed for the underlying convention overloading.
- **M2 — Electrum cosmetic-shape unification (folded).** Electrum emits `derivation` verbatim (no m/ normalization), so the original `s.path.to_string()` form (`48'/0'/0'/2'`) diverged from the direct-path fixture's `m/...` shape. Tightened to `format!("m/{}", s.path)` at the F5 site → all paths emit consistent m/-prefixed shape.
- **M3 — Convergence detection vector** (advisory, no change). Confirmed F5 detection flows through coldcard-multisig's `Derivation:` parser into the re-imported descriptor → mk1 origin_path → triples mismatch. Sound.

## FOLLOWUP
`path-raw-bracketed-vs-bare-convention-unification` filed: `mk1_card_to_resolved_slot` produces bracketed `[fp/path]` while `resolve_slots` produces bare; consumers split (wallet-import + `origin_path_from_bracket` expect bracketed; wallet-export + bundle-emit cosmetic JSON fields expect bare; bundle's `origin_path` field is latently polluted today). Future cleanup: pick one convention, source-fix `mk1_card_to_resolved_slot`, and update bracket-consumers.
