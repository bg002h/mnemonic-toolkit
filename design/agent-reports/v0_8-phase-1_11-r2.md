# v0.8.1 Phase 1.11 R2 — reviewer report

## Convergence verdict

**0C / 0I — converge**

One new L-level finding (R2-L1). No critical or important new issues. Per the CLAUDE.md rule, 0C/0I is reached and the cycle may converge.

---

## R1 fold verification

**C-1 (doubled `error:` prefix — `build_missing_fields_refusal`):**
Verified. `crates/mnemonic-toolkit/src/wallet_export/mod.rs:274` has an explicit comment: `"NOTE: no leading 'error: ' — ToolkitError::Display (error.rs:410) prepends that prefix uniformly..."`. The function body begins with `format!("mnemonic export-wallet --format {format} requires...")` — no prefix. Grep for `"error: error:"` across the crate returns only those two comments (mod.rs:277, jade.rs:59); no literal occurrences in emitted strings. `ToolkitError::Display` at `error.rs:410` (`write!(f, "error: {}", self.message())`) is confirmed as the sole prefix site.

**I-1 (doubled prefix — `JadeEmitter::emit` singlesig refusal):**
Verified co-jointly with C-1. `jade.rs:61` string literal begins with `"mnemonic export-wallet --format jade emits multisig wallet config only..."`. The inline comment at jade.rs:57-60 explicitly notes the no-prefix invariant. Fixture `jade_refusal_singlesig.stderr` contains exactly `error: mnemonic export-wallet...` — single prefix, matching `Display` output.

**I-2 (refuse-on-supply guard for `master_xpub`):**
Verified. Guard at `export_wallet.rs:183-197` fires when all three conditions are true: `has_master_xpub_slot && matches!(args.format, CliExportFormat::Coldcard) && matches!(args.template, Some(Bip44) | Some(Bip49) | Some(Bip84))`. FOLLOWUPS slug `coldcard-master-xpub-plumbing-pending` is present at `design/FOLLOWUPS.md:873` with correct `Where:`, `What:`, `Status:`, `Tier:` fields. Guard is unreachable for other formats (silent ignore) and for coldcard+multisig (master_xpub not relevant there).

**I-3 (`collect_missing` rationale):**
Verified. `ColdcardEmitter::collect_missing` (`coldcard.rs:24-39`) returns `Vec::new()` with a 7-line comment explaining the design choice: per-template incompat refusals surface as `ToolkitError::BadInput` with pointer text, more helpful than the generic `MissingField::IncompatibleFormatForTemplate` bullet. `JadeEmitter::collect_missing` (`jade.rs:24-31`) returns `Vec::new()` with a matching 4-line rationale comment referencing the Coldcard rationale. Both are coherent and cross-consistent.

**I-4 (byte-exact stderr fixtures):**
Verified.
- All 4 `.stderr` files exist at `crates/mnemonic-toolkit/tests/export_wallet/`: `coldcard_refusal_bip86.stderr`, `coldcard_refusal_tr_multi_a.stderr`, `jade_refusal_singlesig.stderr`, `jade_refusal_tr_multi_a.stderr`.
- All 4 refusal tests (`cell_4_coldcard_bip86_refuses_byte_exact`, `cell_6_coldcard_tr_multi_a_refuses`, `cell_4_jade_singlesig_refuses_byte_exact`, `cell_5_jade_tr_multi_a_refuses`) use `assert_eq!(stderr, expected)` — no `.contains()`, no `.trim_end()`.
- Each fixture contains exactly one line of content plus a trailing `\n` (Read tool shows line 1 = message text, line 2 = blank/EOF). This matches `writeln!(stdout, "{emitted}")` in `cmd::export_wallet::run:400` which writes one trailing newline.

**L-1 (JSON field-order comment on `ColdcardGenericJson`):**
Verified. `coldcard.rs:62-69` doc comment states: "Field order matches the canonical upstream sample (`firmware/docs/generic-wallet-export.md`)... SPEC v0.8 §5.1 pins this order intentionally to mirror upstream byte-for-byte... Using `#[derive(Serialize)]` (not `serde_json::Map`) so the output order is guaranteed regardless of whether the crate-level `preserve_order` feature is enabled." Fully covers the §5.1 pinning intent.

**L-2 (manual prose — `### Notes` subsection):**
Verified. `docs/manual/src/40-cli-reference/41-mnemonic.md:162-165` contains both required notes:
1. `--wallet-name` length cap: documents 20 Unicode scalar values, first 20 characters (not bytes), codepoint-granularity truncation, emoji example.
2. `@N.master_xpub=` parse vs emit: documents the refuse-on-supply behavior for coldcard+singlesig, cites FOLLOWUPS slug `coldcard-master-xpub-plumbing-pending`, v0.8.2 scheduling, and the silent-ignore contract for other formats.

**L-3 (dead re-exports removed from `wallet_export/mod.rs`):**
Verified. `mod.rs:20-24` re-exports exactly five symbols: `Bip388Emitter`, `BitcoinCoreEmitter`, `ColdcardEmitter`, `JadeEmitter`, `build_descriptor_string`. All five are consumed in `cmd/export_wallet.rs`. No dead re-export lines are present.

**N-1 (char-boundary-safe wallet-name truncation):**
Verified. `coldcard.rs:302`: `let name: String = inputs.wallet_name.chars().take(20).collect();`. No `.truncate(20)` call anywhere in wallet_export. Regression test `cell_7_coldcard_wallet_name_non_ascii_truncation_no_panic` (`cli_export_wallet_coldcard.rs:299-348`) supplies 25 × U+1F910 (4 bytes each = 100 bytes total), asserts `.success()`, and checks `stdout.lines().next() == Some("Name: " + 20 emoji)`.

---

## New findings

### R2-L1 — Stale inline comment at `coldcard.rs:213-215` misrepresents the I-2 invariant

**File:** `crates/mnemonic-toolkit/src/wallet_export/coldcard.rs:212-216`

**Confidence:** 82

**What:** The inline comment reads:
```
// Phase 1.2: synthesize.rs does not yet forward MasterXpub slot inputs
// into ResolvedSlot, so this is unconditionally None for now. A
// follow-on commit will plumb the field through.
xpub: None,
```
After the I-2 fold, `emit_coldcard_generic_json` is unreachable from the coldcard+singlesig path when `master_xpub` is supplied (the guard in `export_wallet.rs:187-197` refuses before `emit()` is called). The phrase "for now" and "A follow-on commit will plumb the field through" imply this `None` is a temporary placeholder that will be corrected in-place, when actually the follow-on commit (v0.8.2) must lift the guard and add plumbing, not change the `None` alone. A future developer reading only this comment might conclude the `xpub: None` is a bug to fix without understanding the guard.

The function-level doc comment at `coldcard.rs:101-104` correctly states "Top-level `xpub` is emitted iff `@0.master_xpub=` was supplied" — but readers of the implementation body see the stale inline comment first.

**Fix:** Replace the inline comment to acknowledge the guard:
```rust
// SPEC §5.1: top-level xpub emitted iff @0.master_xpub= was supplied.
// The cmd::export_wallet::run guard (FOLLOWUPS `coldcard-master-xpub-plumbing-pending`)
// refuses before this point when master_xpub is supplied for singlesig templates,
// so this None is correct under the current dispatch invariant. v0.8.2 will lift
// the guard and plumb MasterXpub through ResolvedSlot/EmitInputs.
xpub: None,
```

---

## Confidence-filtered: omitted findings

- `REFUSAL_SECRET_INPUT`, `format_stub_message`, `taproot_multisig_unsupported_message` are `pub` rather than `pub(crate)` — binary crate, no external exposure possible; stylistic nit, not a real issue.
- I-2 guard scope excludes `bip86` from the singlesig-template match — bip86 is refused anyway by the emitter; user receives an error either way; harmless gap, no actionable fix.
- I-2 guard scope excludes coldcard+multisig+master_xpub — master_xpub is silently ignored on multisig path; the SPEC comment "Other formats SPEC-IGNORE the slot" applies within-format to non-singlesig paths; pre-existing design decision.
- `cell_7` uses `stdout.lines().next()` rather than asserting the full output — the test's stated purpose is "no panic" regression guard, not full fixture pinning; adequate for the scope.
