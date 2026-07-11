# R0 convergence review — `SPEC_test_hardening_T5_gui.md` (round 2) — Fable, adversarial

**Persisted verbatim per CLAUDE.md.** Verified against `mnemonic-gui` @ `5d88286` (HEAD, clean), the exact pinned binary `mnemonic 0.75.0` (= `Cargo.toml:76` tag `mnemonic-toolkit-v0.75.0`), `ms 0.13.2`. All wire-shape/classifier claims re-run empirically.

## Critical: none. Important: none.

### C1 fold — CLOSED (S3 oracle re-spec) — all three legs empirically exact
- **bundle:** `bundle --template bip84 --network mainnet --slot "@0.phrase=abandon…about" --json` → `"ms1":["ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f"]` (byte-exact) + `"master_fingerprint":"73c5da0a"`. Warnings→stderr; stdout pure JSON. Emitted `md1` = exactly 3 chunks.
- **restore:** ms1 + 3× `--md1` → exit 0, `wallets[0].first_addresses[0] == "bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu"` + `"verification":{"status":"verified"}`.
- **entropy drop correct:** `restore … --json | grep -c entropy` = 0 (top keys `{cosigners,mode,network,threshold,verification,wallets}`). Optional leg: `ms decode --json <ms1>` → `"entropy_hex":"00000000000000000000000000000000"`. GUI already pins both (`ui_harness_i4_realcli.rs:63-64`).

### I1 fold — CLOSED (S3 slot-input) — every API name compile-real
`FormState::with_slots` (`schema/mod.rs:361`); `SlotState{rows}`/`SlotRow{index,subkey,value}` (`form/slot_model.rs:103-105,:148`; tests import via `mnemonic_gui::form::slot_editor::{SlotRow,SlotState,SlotSubkey}`, `tests/argv_assembler_slot.rs:11`); `SlotSubkey::Phrase` exists. `assemble_argv` slot branch `invocation.rs:263-290`; `argv_assembler_slot.rs` cell_1 (:23-70) pins `"--slot","@0.phrase=…"` for `bundle` verbatim. bundle `allows_slots:true` (`schema/mnemonic.rs:4433`); no `--phrase`/`--phrase-stdin` under bundle; boolean-stdin no-emit `continue` (`invocation.rs:354`); `runner::run` = `runner.rs:147`; `"bip84"` a live dropdown choice. Repeating `--md1` supported (`invocation.rs:357`; restore `--md1` `repeating:true`); restore `--from`-based. Implementable end-to-end.

### I2 fold — CLOSED (S2 both-directions)
`non_canonical_descriptor_account_pin.rs` idiom (`state_with_descriptor`→`bundle`→assert PinValue(0) present/absent) extends to both h-directions with zero new machinery. Chain live: `if !is_descriptor_non_canonical(state){vis.push(("--account",PinValue{0}))}` (`conditional.rs:241-248`); `is_descriptor_non_canonical` private (`:139`). Direction (i) is the funds-relevant RED: under `'?h?`→`'?`, h-Canonical `wpkh([deadbeef/84h/0h/0h]@0/<0;1>/*)` (toolkit `canonical`) → GUI NonCanonical → pin mis-lifted → FIRES assertion REDs (L12 class, `:44-63`). M2 verdicts re-confirmed: pkh/wpkh/tr h-origins, mixed `44'/0h/0'`, suffix-origin-h, use-site `/*h`, wsh(multi h-origins) → `canonical`; `sh(multi …45h…)` → `non-canonical`; `44'h` → exit 2 ParseFails. Regexes 1-3 carry `'?h?` (`:99-122`); 4-5 wrapper-prefix only (`:116-118`) → wsh(multi) row NOT mutation-covered.

## MINOR (all folded into the SPEC in this pass)
- m1: `runner.rs` citation → `:147`/`:172` (folded).
- m2: S1 mutation-teeth attribution (wsh row exempt from RED-proof, Acceptance §1) + empirical-`Expect`-capture directive + `44'h`→ParseFails row (folded — the load-bearing Minor).
- m3: `canonicity_drift.rs` = 223 lines (folded).
- m4: `non_canonical_descriptor_account_pin.rs` = 8 tests not 7 (folded).

## VERDICT: GREEN (0C/0I)
All three blocking findings closed with empirical evidence against the live source + pinned 0.75.0 binary; folds introduced no new Critical/Important. T5 implementation may begin.

---
**FOLD STATUS (opus, 2026-07-10):** m1-m4 folded into the SPEC (this pass, verified). T5 GREEN — implementer dispatched (GUI repo, PR+CI-before-tag, no commit). BIP-84 + ms1 oracles externally verified. T5 runs fully parallel to T3/T4.