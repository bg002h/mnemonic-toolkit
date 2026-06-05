# mnemonic restore — SPEC R0 Review (round 3)

**Verdict: GREEN (0 Critical / 0 Important).** The single-sig `mnemonic restore` SPEC has converged. Implementation may begin (→ implementation plan-doc, which itself takes an R0 pass).

Rounds 0/1/2 (RED 1C/5I → descope; 0C/3I; 0C/1I) each had folds verified accurate by the next round. Round-2's I-Mc (reuse `convert::script_type_from_template` instead of hand-writing the map) landed cleanly.

## Critical
None. Address derivation is watch-only by construction: `Secp256k1::verification_only()` + `account_xpub.derive_pub` (public→public, `addresses.rs:232,241-243`); `account_xpriv` never read; `--json` redaction via `is_argv_secret_bearing` (`convert.rs:117`); `WatchOnly` advisory; negative test asserts no `xprv`/`tprv`. No private-material flow path in text/json/format/address output.

## Important
None.

## Minor (do not block GREEN; fold at lift-time — DONE)
- **M-b** — §4 table cited `DerivedAccount` as `derive.rs:23-39`; struct body is `23-36` (impl opens :38). **Folded → `23-36`.** Field names/types accurate.
- **M-gui-cite** — §7 cited `flag_is_secret` at `secrets.rs:49-64`; actual `src/secrets.rs:149` over `SECRET_FLAG_NAMES` `:141`. Substantive claim correct (set already covers the `--passphrase` class; restore adds no new literal secret flag). **Folded → `:149`/`:141` + re-grep-at-P3 note.** Cross-repo GUI anchor re-grepped at pairing time per standing decay convention.

## Verification ledger (RAN against base `6566941`, branch @ `80ee7be`)
- **I-Mc LANDED:** `script_type_from_template(CliTemplate) -> Option<ScriptType>` private at `convert.rs:393`, maps bip44→P2pkh/bip49→P2shP2wpkh/bip84→P2wpkh/bip86→P2tr, multisig→None; returns the `ScriptType` `render_address_from_xpub` consumes (`address_render.rs:18`). SPEC §3.2/§4 reuse it (pub(crate) bump); "hand-write" instruction gone (survives only as a corrected historical note in the fold log). ✓
- All load-bearing cites ACCURATE: `derive_bip32_from_entropy` (`derive_slot.rs:42`), `DerivedAccount` (`derive.rs:23-36`, `account_xpriv` do-not-emit), `slot_ms1::resolve_ms1_slot`+`Ms1SlotResolution.derive_language` (`slot_ms1.rs:37/15`; wire-wins refuse-on-conflict `SlotInputViolation` exit 2 `:60-68`), `build_descriptor_string` (`pipeline.rs:18`), `ResolvedSlot` (`synthesize.rs:642`), `render_address_from_xpub` (`address_render.rs:18`), `CliTemplate::is_multisig` (`template.rs:47`, 6-true), `RestoreMismatch` exit-4 + alpha slot RepairShortCircuit→SilentPayment (`error.rs:471/529/588`; `details() _=>None` :775; ModeViolation→2 :511; BadInput→1 :473), cli_gui_schema 28-vec + "restore" alpha slot (`cli_gui_schema.rs:74,93-94,108`), advisory/env/mlock/redaction, `convert::run` 5-arg + stdin mutex (`:737,798-813`), `CliExportFormat` 11 + dispatch, GUI `SUBCOMMANDS` `mnemonic.rs:3191`. (Sub-Minor drift noted: `CheckedDescriptor` at `mod.rs:414` not `:420` — functional.)
- Internal consistency: `--template Option`; `--expect-fingerprint` valid all-4 (master fp path-independent); `--expect-xpub`/`--format` require `--template Some`; `RestoreMismatch` fields consistent §3.4↔§5; watch-only across all output. ✓
- Phasing/tests coherent: P1 core + exit-4 + I-B/I-C; P2 formats + I-A + redaction; P3 docs/GUI/release.

**Bottom line:** GREEN at 0C/0I. Cleared for the implementation plan-doc.
