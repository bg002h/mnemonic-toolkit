# v0.35.0 — End-of-cycle architect review (opus) — MANDATORY pre-tag gate

**Date:** 2026-05-23
**Cycle:** v0.35.0 `mnemonic silent-payment` (BIP-352 receiver address)
**Branch:** `v0.35.0-silent-payment`
**Reviewer:** opus (feature-dev:code-reviewer), end-of-cycle gate (crypto cycle)
**Scope reviewed:** focused code+manual diff `/tmp/v0_35_0_code.diff` (commits e51c556..bf2e2a2) + live source + BIP-352 spec re-fetch

---

## Critical
(none)

Highest-risk legs verified: derivation paths match BIP-352 (`silent_payment.rs:423-424`); label tweak `B_m=B_spend+hash_BIP0352/Label(ser_256(b_scan)‖ser_32(m))·G` via `Scalar::from_be_bytes`+`add_exp_tweak` (`:366-389`); encode `once(Fe32::Q).chain(payload.bytes_to_fes()).with_checksum::<Bech32m>` over 66-byte `B_scan‖B_m`, NOT segwit (`:396-405`); 28-vector official oracle + seed→path pin both pass. `--label 0` refused as the FIRST statement in `run()` before any derivation (`cmd/silent_payment.rs:182-188`). No secret leaks without the advisory — `secret_on_stdout_warning_unconditional` fires at the single success tail (`:256`), no early return between privkey emission and it. Resolver sniff (xprv→ms1→phrase→hex→error) has no realistic cross-class capture; WIF/minikey → error arm; `Zeroizing` + `mlock::pin_pages_for` present.

## Important
(none)

## Minor
- **1 (FOLDED) — `cli-subcommands.list` omitted `mnemonic silent-payment`**, so the manual flag-coverage lint (`lint.sh` step 4) never actually diffed the new chapter's flags vs `--help` (the "6/6" = the 6 stages, not the SP flags). Gate-wiring gap (defeats the CLAUDE.md manual-mirror invariant for this subcommand). The chapter content is complete + correct. (seedqr-*/electrum-decrypt also absent — latent, pre-existing.) Fix: add `mnemonic silent-payment` to `docs/manual/tests/cli-subcommands.list`. **[FOLDED + re-ran lint to confirm flag-coverage now checks + passes.]**
- **2 (FOLDED) — chapter intro count.** `41-mnemonic.md:3` "Eleven subcommands" — the inline link list went from 11 → 12 with silent-payment added. Fix: "Eleven" → "Twelve". **[FOLDED.]**

## Lockstep obligation (flagged, not a finding)
The toolkit `gui-schema` (clap-derived) now emits `silent-payment` (the `gui_schema_lists_all_subcommands` test bumped 22→23). The paired GUI `mnemonic-gui/src/schema/mnemonic.rs` MUST add the `silent-payment` `SubcommandSchema` (Task 5). `flag_is_secret` already covers `--secret`/`--secret-stdin` (excludes `--secret-file`), so the GUI secret projection inherits. Expected cross-repo follow-on, correctly absent from this diff.

## Verification summary
Crypto correct (vs spec + 28 vectors); resolver sound; label-0 refused pre-derivation; secret-handling complete (privkeys advisory-gated, b_scan online / b_spend COLD labeled); error variant in all blocks (alphabetical, exit 1, not in `_=>None`); registration + gui-schema test updated; manual chapter complete (all 7 flags + --help); version consistent (Cargo.toml/lock/install.sh/CHANGELOG = 0.35.0; MINOR correct); secrets.rs covers the secret flags; receiver-only scope.

VERDICT: GREEN (0C/0I)

---

## Fold disposition (controller)
GREEN (0C/0I) → gate satisfied. Folded both Minors (doc/lint-wiring): Minor 1 (`cli-subcommands.list` += `mnemonic silent-payment`; re-ran lint → flag-coverage now actually checks the SP chapter + passes), Minor 2 (`41-mnemonic.md:3` "Eleven"→"Twelve"). Doc-only, no R2 re-dispatch (no Critical/Important).
