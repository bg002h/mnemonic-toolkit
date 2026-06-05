# mnemonic restore — SPEC R0 Review (round 1)

**Verdict: RED (0C / 3I).** Descope + all six round-0 folds (C1/I1/I2/I3/I5/M1-M6) landed correctly; single-sig design end-to-end-proven with ZERO private-key-leakage path. RED only on 3 under-specifications surfaced in the now-single-sig surface. Fold + re-dispatch.

## Critical
None.

## Important

**I-A — `--format` + no-`--template` exit code is self-contradictory (§2, §1).** SPEC says "`BadInput`/usage, exit 2", but `BadInput` = exit **1** (`error.rs exit_code()`; `addresses.rs:104 bad()`), and the I2 fold uses `BadInput` exit 1 too. Only clap-usage or `ModeViolation` → exit 2. **Fold:** `--template` becomes `Option<CliTemplate>` (None = all-4 default); `--format` set + `--template` None → `ModeViolation` (exit 2). Pin the code so the P2 test asserts the right number. (`format_requires_template` at `export_wallet.rs:54` is a different concept — correctly NOT cited; leave it.)

**I-B — multisig `--template` values not rejected (§2, §3.2).** `CliTemplate` (`template.rs:16`) is a `ValueEnum` with all 10 variants incl. 6 multisig; `--template wsh-sortedmulti` would silently emit a degenerate 1-of-1 + BIP-87 path. Also there is NO `ScriptType` for multisig → `render_address_from_xpub` can't be called. **Fold:** reject multisig templates via `CliTemplate::is_multisig()` (`template.rs:47`, pub) → `BadInput` exit 1 ("restore is single-sig only; --template ∈ {bip44,bip49,bip84,bip86}").

**I-C — ms1 decode drops the `mnem` wire-language (§3.1, §4).** ms1 has `Payload::Entr` (no lang) and `Payload::Mnem { language: wire, entropy }` (wordlist language ON WIRE → drives PBKDF2). A bare `ms_codec::decode` ignoring it derives the WRONG seed for non-English `mnem` cards. **Fold:** route ms1 through `slot_ms1::resolve_ms1_slot(value, flag_language, idx) -> Ms1SlotResolution { entropy, derive_language, emit_language }` (`slot_ms1.rs:37`, pub; `mod slot_ms1` at `main.rs:28`) and use `res.derive_language` for `derive_bip32_from_entropy`. This applies the wire-wins / refuse-on-`--language`-conflict policy (`SlotInputViolation` exit 2) used everywhere else (`convert.rs:1463-1495`). Builds on `project_ms_mnem_v0_2_shipped` / `project_ms1_slot_v0_41_0_shipped`.

## Minor (fold)
- **M-a** error.rs anchors off-by-1: `exit_code:471 / kind:529 / message:588 / details:775` (SPEC said 472/530/589/776). Alpha slot + `details() _=>None` correct.
- **M-b** `DerivedAccount` struct is `derive.rs:23-39` (SPEC said `:24-37`). Fields/types accurate.
- **M-c** No `CliTemplate→ScriptType` helper in-tree (only forward `template_for(ScriptType)` `addresses.rs:95`). Restore hand-writes the 4-way inverse map (bip44→P2pkh, bip49→P2shP2wpkh, bip84→P2wpkh, bip86→P2tr). One-line note.
- **M-d** `main.rs` `enum Command` (`:90`) + dispatch (`:153`) are feature-clustered, NOT alpha — line anchors approximate; extend the "don't re-sort" caveat to main.rs.

## Verification ledger (highlights — all RAN/checked against source)
- **`b4e3f5ed` CONFIRMED** (RAN): `convert --from phrase=@env:P --to fingerprint --template bip84 --passphrase @env:PP` (PP=TREZOR) → `b4e3f5ed`, path-independent; NOT yet in-tree (I5 re-derive obligation correct). `73c5da0a` (no-pp) matches `cli_export_wallet.rs:27`.
- **§3.2 step-3 origin-rendering PROVEN** (RAN): hand-built slot equiv (`export-wallet --slot @0.xpub= --slot @0.fingerprint=73c5da0a --template bip84 --format descriptor`) → `wpkh([73c5da0a/84'/0'/0']xpub6CatW…/<0;1>/*)#hpg6d6w2` — REAL fingerprint origin, not `[00000000/…]`. Secret slots set `ResolvedSlot.path = account_path` (full `m/84'/0'/0'`) via `into_parts` (`bundle.rs:528,649`); `key_origin_str` renders `[fp/path]` (`pipeline.rs:33`). Export-wallet-leakage class does NOT bite.
- C1 descope CLEAN: multisig APIs appear only in §11 + §4 NB; P1/P2/P3 reach only single-sig + `build_descriptor_string`.
- All P1 load-bearing cites ACCURATE: `derive_bip32_from_entropy`, `DerivedAccount` (+`account_xpriv` do-not-emit), `build_descriptor_string`, `render_address_from_xpub`, `resolve_env_var_sentinel`, `pin_pages_for`, `secret_advisory::*`, `is_argv_secret_bearing`, `CliExportFormat`+dispatch, error exit-4 tier + alpha slot, cli_gui_schema 28@:74/:108, `flag_is_secret`, GUI/manual lockstep, sibling pins ms/mk 0.4.0/md 0.35. `CliTemplate::is_multisig()` pub `template.rs:47`; `slot_ms1::resolve_ms1_slot` pub `slot_ms1.rs:37`.
- **Phasing:** 3-phase single-sig shape appropriately scoped (round-0 4-phase over-bound resolved). P1 heaviest (error variant + gui-schema fix + core) but coherent.

**Bottom line:** zero leakage, design proven. Fold I-A (exit-code pin via Option<template>+ModeViolation), I-B (reject multisig --template), I-C (route ms1 via resolve_ms1_slot) + M-a..M-d, re-dispatch.
