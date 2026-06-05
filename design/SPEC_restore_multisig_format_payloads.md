# SPEC — `mnemonic restore --md1 --format <export-format>` (multisig importable payloads)

**Status:** R0 gate (pre-implementation). MUST converge to 0 Critical / 0 Important before any code.
**Resolves:** FOLLOWUP `restore-multisig-format-payloads`.
**Source SHA:** branch `restore-multisig-format-payloads` off master `9bb80a0` (toolkit v0.44.0); md-codec pinned `0.35.0`.
**SemVer:** MINOR — additive output capability on the existing `restore --format` flag (a previously-refused `--md1 --format` invocation now emits). No new clap surface.

---

## 1. Summary

`mnemonic restore --md1` (multisig-cosigner restore, v0.44.0) reconstructs a watch-only multisig descriptor from the shared wallet-policy md1 and currently **refuses `--format`** (`run_multisig` gate at `restore.rs:735-741`, exit 2). Single-sig restore's `--format` emits an importable wallet payload via the `export-wallet` `WalletFormatEmitter` dispatch but requires one `--template` (doesn't fit multisig). This cycle un-refuses multisig `--format`: post-reconstruction the data is all in hand (`template`, `slots`, `k`, `descriptor`, `network`, `account`), so we build a **multisig `EmitInputs`** and run the same emitter dispatch — `restore --md1 --format X` then emits the same payload class as `export-wallet --template <multisig> --format X`.

No new CLI flag (reuses `--format`; `k` from the md1, not `--threshold`). Toolkit-only; no sibling-codec change.

## 2. Empirical ground truth (captured pre-implementation, v0.44.0 binary)

**Per-format multisig outcome** — `export-wallet --template wsh-sortedmulti --threshold 2 --slot @0..2.xpub=… --format X` (the reference for what restore's template-mode dispatch must do):

| EMIT (9) | REFUSE (2) |
|---|---|
| `bitcoin-core`, `bip388`, `coldcard`, `coldcard-multisig`, `jade`, `sparrow`, `electrum`, `bsms`, `descriptor` | `specter` (`ExportWalletMissingFields`: needs `--wallet-name`), `green` (exit 1: "does not support multisig") |

- **`coldcard` emits multisig text for a multisig template** (byte-identical to `coldcard-multisig`, 442b) — the *template*, not the format name, gates. So multisig restore does NOT refuse `--format coldcard` (mirrors export-wallet). The `coldcard-multisig` dispatch arm (`export_wallet.rs:539-552`) is a six-variant `CliTemplate` match (`WshMulti | WshSortedMulti | ShWshMulti | ShWshSortedMulti | TrMultiA | TrSortedMultiA` → emit multisig text, else `Err(BadInput)`) — NOT an `is_multisig()` call (R0-r1 M1); behaviorally it emits for any multisig template.
- **`collect_missing` is empty** for `bitcoin-core`/`bip388`/`coldcard`/`bsms`/`descriptor` (no `--wallet-name` needed) → restore's synthesized default name works. `specter` genuinely needs a name → refuses (restore has no `--wallet-name`; consistent with single-sig restore's specter refusal).
- **Cross-tool byte-parity vs export-wallet is not cleanly reproducible and is unnecessary:** restore's `--md1` EmitInputs are a unique provenance (template-mode + the md1's **real** master fingerprints `73c5da0a`/`b8688df1`/`28645006` + the md1's xpub serialization). Matching it via `export-wallet --slot @N.xpub=` would require hand-supplying each cosigner's `[Xpub, Fingerprint, Path]` form (`slot_input.rs:354-359` accepts it, so it is *possible* but laborious + fragile, R0-r1 M2); `export-wallet --descriptor` uses `template:None` (descriptor-mode) + supports only 4 formats. The cycle instead uses the strictly stronger, self-contained **`--format descriptor` exact-equality against the same run's `--json` descriptor** (genuine byte-parity for that format) + **multisig-fidelity containment** for the rest (§6). No fragile cross-tool reconstruction.

## 3. EmitInputs construction (byte-identical-to-export-wallet's-multisig-fields)

New `build_multisig_import_payload(format, template, slots: &[ResolvedSlot], k, descriptor: &str, network, account) -> Result<String>` in `restore.rs`, building:

```rust
let script_type = wallet_export::script_type_from_template(&template);
let wallet_name = format!("{}-{}", template.human_name(), account); // == export_wallet.rs:472 default
let inputs = EmitInputs {
    canonical_descriptor: CheckedDescriptor::new(descriptor)?,
    resolved_slots: slots,                 // @0..N order from expand_per_at_n
    template: Some(template),
    script_type,
    network, account,
    threshold: Some(k),
    threshold_user_supplied: true,         // k from md1 is AUTHORITATIVE — matches export-wallet's
                                           // emit AND is required: sparrow.rs:43 refuses multisig
                                           // (MissingField::Threshold) when this is false.
    master_xpub_at_0: slots.first().and_then(|s| s.master_xpub), // None for md1 cosigners (parity)
    wallet_name: &wallet_name,
    wallet_name_is_non_default: false,
    taproot_internal_key: None,            // wsh/sh-wsh only; taproot md1 refused upstream (restore.rs:766)
    range: (0, 999),                       // == export-wallet --range default
    timestamp: TimestampArg::Now,          // literal "now" sentinel (not wall-clock) == export-wallet default
    bitcoin_core_version: 25,              // == export-wallet --bitcoin-core-version default
    bsms_form: BsmsForm::default(),
};
```

Then the dispatch (`collect_missing`-first → `emit`) is written **arm-for-arm byte-identical to `export_wallet.rs:507-560`**, INCLUDING the `coldcard-multisig` arm's six-variant `CliTemplate` match (`:531-553` — emit multisig text for any of the 6 multisig templates, else `Err(BadInput)`; NOT an `is_multisig()` call) verbatim. (Deliberate copy — see §7 de-dup FOLLOWUP.) This is the 3rd copy of the dispatch (export_wallet, restore single-sig, restore multisig); keeping it byte-identical makes the eventual consolidation mechanical.

## 4. `run_multisig` integration (mirror single-sig restore's `--format` weave)

Replace the refusal gate (`restore.rs:735-741`) with the single-sig pattern (`restore.rs:452-543`):

- Compute `import_payload: Option<String>` = `args.format.map(|f| build_multisig_import_payload(f, template, &slots, k, &descriptor, network, account)).transpose()?` — **after** reconstruction + cross-check (so a `--from`/`--cosigner` MISMATCH still hard-fails exit 4 BEFORE emitting a payload).
- `stdout_content`: when `--json`, add `envelope["import_payload"] = payload` (mirror single-sig `restore.rs:490-492`); else when `import_payload.is_some()`, the payload alone is stdout (pipes cleanly); else the existing text doc.
- When `import_payload.is_some() && !args.json`: write the human verification doc (descriptor + cosigner table + UNVERIFIED/PARTIAL banner) to **stderr** (mirror single-sig `restore.rs:521-543`), so the payload pipes onward.
- `--output <FILE>` routes the stdout_content as today.

## 5. Scope / refusals

- **EMIT:** `bitcoin-core`, `bip388`, `coldcard`, `coldcard-multisig`, `jade`, `sparrow`, `electrum`, `bsms`, `descriptor` (the emitter internally handles multisig — reused verbatim).
- **REFUSE (unchanged, via the reused emitter logic):** `specter` (`ExportWalletMissingFields` — no `--wallet-name`), `green` (exit 1 — no multisig support). These refuse identically to `export-wallet`; no restore-specific message.
- **taproot** multisig md1 already refused upstream at `restore.rs:766` (`Tag::Tr` → exit 2) before reconstruction — out of scope (FOLLOWUP `restore-multisig-taproot-reconstruction`); no taproot `--format` cells.
- **Watch-only-out preserved:** the payloads are public-only (cosigner xpubs); no `xprv`/WIF/seed reaches stdout/stderr/`--json` (test-enforced, §6).

## 6. Tests — `tests/cli_restore_multisig_format.rs` (new)

Fixture: bundle the 2-of-3 `wsh-sortedmulti` from C0/C1/C2 (reuse `cli_restore_multisig.rs` pattern) → md1; master fps `73c5da0a` / `b8688df1` / `28645006`.

- **EMIT × multisig-fidelity (9 cells, the primary correctness check — catches silent single-sig-ify):** for each EMIT format, `restore --md1 --format X` exits 0 AND the payload contains the **exact per-format threshold token** below (NOT a bare `"2"` — that is vacuous: the digit appears in xpubs/paths/dates; R0-r1 I1 / [[feedback_ci_snapshot_test_substring_vacuity]]). A K=1 / single-sig-ified payload lacks the `2`-threshold token. Tokens (empirically pinned against the v0.44.0 binary):

  | threshold token | formats |
  |---|---|
  | `sortedmulti(2,` | `descriptor`, `bitcoin-core`, `bip388`, `sparrow`, `bsms` |
  | `Policy: 2 of` | `coldcard`, `coldcard-multisig`, `jade` |
  | `2of3` (`"wallet_type":"2of3"`) | `electrum` |

  PLUS, for the formats that embed `[fp/…]` hex key-origins (`descriptor`, `bitcoin-core`, `bsms`), assert **all three** real md1 cosigner fingerprints `73c5da0a` / `b8688df1` / `28645006` appear (proves the RIGHT 3 cosigners + the drop-a-cosigner case; restore embeds the md1's real fps, unlike export-wallet's placeholder `00000000`). For the non-fp-embedding formats the threshold token + `--format descriptor` equality below carry the fidelity guarantee.
- **`--format descriptor` exact-equality:** `restore --md1 --format descriptor` stdout == the `restore --md1 --json` `wallets[0].descriptor` (the bare canonical descriptor; clean strongest check).
- **Refusal cells (2):** `--format specter` → `ExportWalletMissingFields` (exit 2); `--format green` → exit 1 "does not support multisig". (Match export-wallet.)
- **Watch-only-out:** for `bitcoin-core` + `descriptor` + `bsms`, assert no `xprv`/`tprv`/`xprv`/WIF (`L`/`K`/`5` priv heuristics are noisy — assert NOT contains `"xprv"`/`"tprv"`) in stdout AND stderr AND `--json`.
- **`--json` envelope:** `--md1 --format bitcoin-core --json` → `import_payload` field present + still `mode:"multisig"` + `threshold:2` + 3 cosigners.
- **Mismatch precedence:** `--md1 --from <FOREIGN> --format descriptor` → exit 4 `RestoreMismatch` (NO payload emitted), unless `--allow-mismatch`.
- **`sh-wsh-sortedmulti`:** one EMIT cell (e.g. `--format descriptor`) on a `sh(wsh(sortedmulti))` md1 → payload contains `sh(wsh(sortedmulti(2,`.
- **`--output FILE`:** `--format descriptor --output <tmp>` writes the payload to the file; stderr carries the verification doc.

All run via the bundled md1 at runtime (no pinned goldens). Toolchain: toolkit CI pins 1.85.0; run `cargo test` workspace-wide per phase (`--no-fail-fast`).

## 7. Lockstep / FOLLOWUPs

- **GUI `schema_mirror`:** NO change — no clap flag/option/value-enum add/remove/rename (reuses `--format`; its `EXPORT_FORMATS` dropdown already lists all 11). Confirm at R0 by diffing `gui-schema` (restore flag set + EXPORT_FORMATS unchanged).
- **Manual mirror:** `docs/manual/src/40-cli-reference/41-mnemonic.md` — the `### Multisig-cosigner restore` section gains a `--format` paragraph (supported formats; specter/green refusals; watch-only). Apply the anchor discipline ([[project_manual_anchor_dangler_cleanup_shipped]]): run **`make audit`** (not just `make lint`) before declaring the manual green. No new cross-file anchor expected (no new heading), so no baseline entry anticipated.
- **NEW FOLLOWUP `restore-emit-dispatch-3way-dedup`:** the 11-arm `collect_missing`→`emit` dispatch now exists in 3 byte-identical copies (`export_wallet.rs:507-560`, `restore.rs` single-sig `build_import_payload:587-665`, the new multisig copy). Consolidate into one `wallet_export::emit_payload(inputs, format)` helper consumed by all three (same species as `descriptor-origin-extraction-dedup`). Deferred to keep this cycle additive + avoid perturbing two shipped/tested paths.

## 8. Phased plan

- **Phase 1 (RED):** `tests/cli_restore_multisig_format.rs` — all cells above. They fail (multisig `--format` refuses at `:735`). Verify RED-for-the-right-reason (the exit-2 refusal) at runtime.
- **Phase 2 (GREEN):** `build_multisig_import_payload` (§3, dispatch byte-identical to export-wallet) + `run_multisig` `--format` weave (§4) − remove the refusal gate. Workspace `cargo test --no-fail-fast` GREEN. Per-phase opus review → persist to `design/agent-reports/`.
- **Phase 3 (docs + release):** manual `--format` paragraph (§7) + `make audit` green; CHANGELOG; version bump (PATCH→ no, MINOR: v0.44.0 → **v0.45.0**); README version markers; `scripts/install.sh` self-pin; FOLLOWUP `restore-multisig-format-payloads` → resolved + file `restore-emit-dispatch-3way-dedup`. Per-phase review.
- **Phase 4 (ship):** clean-tree check → `git checkout master && ff-merge` → tag `mnemonic-toolkit-v0.45.0` → push master + tag → watch CI (rust, install-pin-check, sibling-pin-check, manual).

## 9. Risk

Low — additive, reuses the proven emitter set verbatim; the only new logic is the multisig `EmitInputs` construction (field values empirically pinned to export-wallet's) + the stdout/stderr weave (mirrors single-sig). The `threshold_user_supplied: true` value is load-bearing (sparrow refuses otherwise) and tested. No new CLI surface → no GUI lockstep.
