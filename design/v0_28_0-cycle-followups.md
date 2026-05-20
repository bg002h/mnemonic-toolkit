# v0.28.0 cycle in-progress FOLLOWUPS tracker

**Purpose:** scratchpad for out-of-scope items, deferred decisions, and surface-discovered work that emerges DURING v0.28.0 cycle execution. Per the plan-doc's scope-creep defense, new work is logged HERE (not folded mid-cycle), then triaged into `design/FOLLOWUPS.md` at Phase P14A (cycle close).

**Authoritative scope:** `/home/bcg/.claude/plans/unified-meandering-sundae.md` (R6 GREEN). Any work item NOT in the plan-doc's sub-phase rows is OOS by default.

**Cycle status:** Wave 0 in progress (P0A active 2026-05-19).

---

## Format

Each entry:

```markdown
### `<short-slug>` — <one-line title>

- **Surfaced:** YYYY-MM-DD during Phase P{N}{X} execution; brief context.
- **Where:** file:line citations (re-grep at write-time per plan-doc verification discipline).
- **What:** what the work would be.
- **Why deferred:** explicit scope-creep-defense reasoning.
- **Triage decision (post-P14A):** open in design/FOLLOWUPS.md / wontfix / fold-into-existing-FOLLOWUP / fold-into-v0.28.0 (rare; requires user lift).
- **Tier:** `v0.28+` / `v0.29+` / etc.
```

---

## Open items (cycle-internal)

### `wallet-import-cross-format-symmetric-mismatch` — extend `--format <X>` mismatch checks to all N+1 cross-format pairs

- **Surfaced:** 2026-05-19 during Phase P2C (specter dispatch wiring).
- **Where:** `crates/mnemonic-toolkit/src/cmd/import_wallet.rs:246-263` — only the `Some("bsms")` ↔ `Some("bitcoin-core")` arms cross-check the SniffOutcome for mismatch. P2C added `Some("specter")` with arms that reject `SniffOutcome::{Bsms,BitcoinCore}` but the inverse — `Some("bsms")` rejecting `SniffOutcome::Specter`, `Some("bitcoin-core")` rejecting `SniffOutcome::Specter`, etc. — is NOT covered. Today the existing `tests/cli_import_wallet_bitcoin_core.rs:548` `specter_like` test runs `--format bitcoin-core` against a Specter-shaped blob and the Core parser fails with `ImportWalletParse` (exit 2). The user gets a less-precise error than the `ImportWalletFormatMismatch` they'd get if the symmetric mismatch fired earlier.
- **What:** decide between (a) extend each `Some("<format>")` arm with an exhaustive sniff-outcome match rejecting all non-matching outcomes, or (b) introduce a generalized `check_format_mismatch(supplied, sniffed) -> Result<...>` helper consulted before the per-format arms. With 8 formats post-cycle, the matrix is 8×7=56 mismatch pairs (each direction); a generalized helper is preferable.
- **Why deferred:** the plan-doc P{N}C row literally enumerates "Flip 8 dispatch sites" — not "extend symmetric N+1 mismatch coverage." Cross-format mismatch matrix work fits the Phase P11 (cross-format-conversion-matrix-expansion) scope rather than the per-parser P{N}C scope. The existing test still passes because Core parser produces a typed `ImportWalletParse` instead of the more-precise `ImportWalletFormatMismatch`; behavior is correct (just less specific).
- **Triage decision (post-P14A):** open in design/FOLLOWUPS.md.
- **Tier:** v0.28+ (could land in Phase P11 of v0.28.0 if the matrix expansion adopts the generalized helper).

---

## Triage queue for Phase P14A

(none yet — populated at cycle close from the open-items list)
