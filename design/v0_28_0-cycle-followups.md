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

### `sparrow-taproot-descriptor-passthrough-import-support` — Sparrow taproot import support

- **Surfaced:** 2026-05-19 during Phase P1B execution; SPEC §11.1 implementation discovery.
- **Where:** `crates/mnemonic-toolkit/src/wallet_import/sparrow.rs:280-285` (parse-step-6 taproot refusal) + `crates/mnemonic-toolkit/src/wallet_export/sparrow.rs:215-219` (emit-side taproot descriptor-passthrough).
- **What:** Sparrow's emit ships taproot wallets as DESCRIPTOR-PASSTHROUGH (concrete `[fp/path]xpub` keys embedded in `defaultPolicy.miniscript.script` instead of `@N/**` placeholders). The P1B parse path substitutes `@N/**` placeholders and refuses taproot scripts; full taproot import requires a parallel parse path that detects descriptor-passthrough shape via heuristic (e.g., `[fp/path]xpub` substring vs `@N/**`) and consumes the embedded concrete-keys descriptor verbatim via `concrete_keys_to_placeholders`.
- **Why deferred:** P1B is the first per-parser cycle; taproot import is a non-trivial second parse path with its own sniff/refusal matrix. Better to ship singlesig + sortedmulti coverage first and dedicate a follow-on cycle to taproot multisig + descriptor-passthrough support symmetric across all 6 new parsers (Sparrow/Specter/Coldcard/etc.).
- **Triage decision (post-P14A):** open in design/FOLLOWUPS.md as `sparrow-taproot-descriptor-passthrough-import-support`.
- **Tier:** v0.29+.

### `sparrow-descriptor-with-checksum-verify-fixture` — dedicated checksum-verify fixture

- **Surfaced:** 2026-05-19 during Phase P1B execution; plan-doc §S.1 fixture enumeration mentions "descriptor-with-checksum verify" as one of ~5 fixtures.
- **Where:** `crates/mnemonic-toolkit/tests/fixtures/wallet_import/sparrow-*.json` (5 fixtures created; no `*-checksum-verify` fixture).
- **What:** Add a dedicated `sparrow-checksum-verify-mainnet.json` fixture that round-trips through `canonicalize_sparrow` + the `recompute_descriptor_checksum` helper. Currently `parse_single_wpkh_mainnet_happy_path` indirectly exercises this; a fixture-level cell makes the contract explicit + survives `--ignored` filter usage.
- **Why deferred:** Sparrow's WIRE SHAPE has no BIP-380 checksum on `miniscript.script` (the script is a bare policy expression, not a BIP-380 descriptor). The "checksum verify" semantic only applies to the toolkit-side `original_descriptor` field on `ParsedImport`, which is implicit in P1B's parse cells. A dedicated fixture is scope-creep at P1B; could fold into a future hardening cycle if Sparrow ever adds a wire-level checksum.
- **Triage decision (post-P14A):** wontfix-with-rationale (Sparrow's wire shape does not carry a wire-level checksum); resurface only if Sparrow changes their JSON shape.
- **Tier:** v0.29+ (likely wontfix).

---

## Triage queue for Phase P14A

(none yet — populated at cycle close from the open-items list)
