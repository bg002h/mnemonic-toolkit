# mnemonic restore — SPEC R0 Review (round 2)

**Verdict: RED (0C / 1I).** Descope + all round-0 and round-1 folds (I-A/I-B/I-C/M-a/M-b/M-d) landed and are ACCURATE; zero private-key-leakage path; full P1 input→derive→verify→descriptor→address→output chain end-to-end-implementable from verified APIs. RED on ONE Important — the round-1 M-c fold codified a factually-wrong "no helper exists" claim.

## Critical
None. Address rendering uses `Secp256k1::verification_only()` + `Xpub::derive_pub` (watch-only by construction, `addresses.rs:232,242`); `account_xpriv` never emitted; `--json` redacts via `is_argv_secret_bearing`; negative test asserts no `xprv`/`tprv`.

## Important

**I-Mc — "no in-tree `CliTemplate→ScriptType` helper" is WRONG; reuse the existing one, don't hand-write a 4th copy (§3.2 step 2, §4 M-c).** `convert::script_type_from_template(template: CliTemplate) -> Option<ScriptType>` already exists at **`convert.rs:393-402`** mapping exactly bip44→P2pkh / bip49→P2shP2wpkh / bip84→P2wpkh / bip86→P2tr (multisig → `None`), returning the same `convert::ScriptType` that `render_address_from_xpub` consumes (`address_render.rs:18`). It is **private** (`fn`) → restore needs a one-line bump to `pub(crate)`. The mapping *values* the SPEC lists are correct, but the "hand-write the inverse" instruction is a `feedback_fix_the_class_hunt_for_second_instance` violation (the tree already has `convert::script_type_from_template`→`ScriptType`, `wallet_export::script_type_from_template`→`WalletScriptType` at `mod.rs:193`, and reverse `addresses::template_for`). **Fold:** §3.2/§4 reuse `convert::script_type_from_template` (bump `pub(crate)`); drop the hand-write instruction. The multisig→`None` arm is unreachable after I-B rejection (or an internal-invariant guard).

## Minor (do not block GREEN)
- **M-idx** — `resolve_ms1_slot`'s `slot_index` param is used ONLY in the `SlotInputViolation` message string ("slot @{idx}.ms1=…", `slot_ms1.rs:63`); it does NOT touch derivation, so passing a fixed `idx=0` (§3.1) is functionally correct. Only consequence: a restore-time ms1 language-conflict error reads "slot @0.ms1=" — slightly odd for a slot-less subcommand. Acceptable; optionally note in §3.1 / confirm phrasing in the I-C test.
- **M-b precision** — `DerivedAccount` struct fields are `derive.rs:23-36` (impl opens :38); SPEC's "23-39" is close enough. Fields/types ACCURATE.

## Verification ledger (highlights — all RAN against `edd58f6`, base `6566941`)
- **I-A** `ModeViolation` = exit 2 (`error.rs:511`); `BadInput` = exit 1 (`:473`); no residual "BadInput exit 2" contradiction in the SPEC. ✓
- **I-B** `CliTemplate::is_multisig()` pub, true for exactly the 6 multisig variants (`template.rs:47-57`; in-tree test `:394-400`; called cross-module `electrum.rs:70`). ✓
- **I-C** `slot_ms1::resolve_ms1_slot(value, flag_language: Option<CliLanguage>, slot_index: u8) -> Result<Ms1SlotResolution,_>` pub; `Ms1SlotResolution { entropy:Zeroizing<Vec<u8>>, derive_language: bip39::Language, emit_language: Option<_> }` (`slot_ms1.rs:15-41`); `derive_language` is the field fed to derivation (real caller `bundle.rs:672,682`); wire-wins/refuse-on-conflict → `SlotInputViolation` exit 2 (`slot_ms1.rs:52-68`; `error.rs:519`). ✓
- **M-a** error blocks 471/529/588 forced-exhaustive + `details()` `_=>None` @775; `RestoreMismatch` alpha slot RepairShortCircuit→SilentPayment in all. ✓
- **M-d** `main.rs` enum `:90` + dispatch feature-clustered (not alpha). ✓
- Internal consistency: `--template Option`; `--expect-fingerprint` works with all-4 (path-independent); `--expect-xpub`/`--format` require `--template Some` (path-dependent / one-descriptor-emit) → no contradiction. `RestoreMismatch` fields consistent §3.4↔§5. Watch-only holds across text/json/format/address. ✓
- Phasing: 3-phase single-sig coherent.

**Bottom line:** fix the one citation (reuse `convert::script_type_from_template`, bump pub(crate); drop "hand-write"), re-dispatch. Everything else GREEN.
