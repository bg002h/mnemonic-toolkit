# Phase 4.4 review — r1

Date: 2026-05-11
Reviewer: feature-dev:code-reviewer (r1)

## Summary

- Chapter (54-mnemonic-toolkit-api.md): 1C / 1I / 0L / 0N
- Index-table accretion: 0C / 0I / 0L / 0N
- cspell additions: 0C / 0I / 0L / 0N
- Worked example: 0C / 0I / 0L / 0N
- Transcript pair: 0C / 0I / 0L / 0N
- Cross-cutting: 0C / 0I / 0L / 0N

Total: 1C / 1I / 0L / 0N

---

## Findings — chapter

### C-1 — §V.4.4 ToolkitError taxonomy missing `ExportWalletMissingFields` — HEAD has 26 variants, not 25

**Location:** `docs/technical-manual/src/50-rust-api/54-mnemonic-toolkit-api.md:145,175`

**Evidence.** HEAD `crates/mnemonic-toolkit/src/error.rs:109-112`:

```rust
ExportWalletMissingFields {
    format: &'static str,
    missing: Vec<&'static str>,
},
```

The variant carries `exit_code() → 2` (`error.rs:249`), `kind() → "ExportWalletMissingFields"` (`error.rs:292`), and `message()` dispatches to `crate::wallet_export::build_missing_fields_refusal` (`error.rs:350`). `#[allow(dead_code)]` reserves it for Phase-1+ emitters. The variant is part of the v0.8.1 phase-0 work that landed on master after the Phase 4.0 harvest was generated; the chapter inherited the harvest's 25-variant count.

`awk '/^pub enum ToolkitError/,/^}/' error.rs | grep -cE '^    [A-Z]'` yields **26**.

**Fix.** Add the missing row to the §V.4.4 table, update preamble + closing count from 25 to 26, and note the v0.8.1 phase-0 reservation in the preamble.

---

### I-1 — §V.4.3.8 `wallet_export` row omits public functions

**Location:** `docs/technical-manual/src/50-rust-api/54-mnemonic-toolkit-api.md:133`

**Evidence.** The row enumerated `REFUSAL_SECRET_INPUT`, `format_stub_message`, `TaprootInternalKey`. HEAD `wallet_export/mod.rs` declares additional `pub` items:
- `pub fn taproot_multisig_unsupported_message(name: &str) -> String` at `:47` (called by `ToolkitError::ExportWalletTaprootMultisigUnsupported.message()` at `error.rs:348`).
- `pub fn build_missing_fields_refusal(...)` (`#[allow(dead_code)]`, reserved for v0.8.1 phase-0 — called by `ToolkitError::ExportWalletMissingFields.message()` at `error.rs:350`).

Both are `pub` (not `pub(crate)`), so they belong in the orchestration-module reference. The `wallet_export.rs` path also drifted to `wallet_export/mod.rs` post v0.8.1 phase-0 (module turned into a directory).

**Fix.** Update the row to list all 5 public items + correct the module path to `src/wallet_export/mod.rs`.

---

## Findings — index-table accretion

None. 15 new rows (413 → 428). All anchor to `#mnemonic-toolkit-rust-api`.

## Findings — cspell additions

None. `impls`, `serialise`, `serialised` confirmed.

## Findings — worked example

None. Standalone consumer (no `mnemonic-toolkit` dep); fixture is valid schema-4 `BundleJson`; output deterministic.

## Findings — transcript pair

None. `.cmd` correct format; `.out` matches example output.

## Findings — cross-cutting

None. Full pass:

- **Binary-only constraint surfaced.** §V.4.1, §V.4.2, §V.4.7, §V.4.5.8 all independently state the constraint.
- **`schema_version = "4"`** at every cited location; stale `format.rs:114` doc-comment correctly flagged in §V.4.5 and §V.4.8.
- **`VerifyCheck` serde semantics.** All four forensic fields confirmed `#[serde(skip_serializing_if = "Option::is_none")]` at `format.rs:171,174,177,181` — chapter is correct (correcting the stale harvest finding).
- **`#[non_exhaustive]` attribution exact.** Only `ToolkitError` at `error.rs:9`. No false attributions.
- **BIP-388 distinct-key attribution.** Correctly cites `parse_descriptor::check_key_vector_distinctness` at `parse_descriptor.rs:1104`.
- **Line-cite spot-checks (10):** `error.rs:10,119,223,254,288,364,453`; `format.rs:120,149,166` — all confirmed.
- **CLI surface excluded.** §V.4.3.9 marks `cmd::*` OUT OF SCOPE.
- **Cross-references.** §II.1, §II.2, §II.3, §IV.1, §IV.2, §IV.3, §V.1, §V.2, §V.3 all valid.
- **Style alignment.** Subsection numbering, citation format, `\index{}` convention, code-block language tags all match §V.1–§V.3.

---

## Verdict

- [ ] 0 C / 0 I — Phase 4.4 ready to close
- [x] Findings present — iterate r2

C-1: `ExportWalletMissingFields` missing from §V.4.4 taxonomy (HEAD has 26 variants; chapter says 25 — v0.8.1 phase-0 work landed after the harvest was generated). I-1: §V.4.3.8 `wallet_export` row omits two `pub fn`s and the module path is stale (`.rs` → `mod.rs` post phase-0).

Both fixes are local chapter edits. All other deliverables clean.
