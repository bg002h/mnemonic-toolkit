# v0.31.1 plan-doc R0 review

**Reviewer:** opus
**Round:** R0
**Plan under review:** design/PLAN_mnemonic_toolkit_v0_31_1.md
**Date:** 2026-05-21
**Source SHA:** b29029f (master HEAD)

## Critical (C)

### C1 — Existing taproot-refusal tests will FAIL after path-split; Phase 2 Step 5 underspecifies the update.

Plan-doc Step 5 says "Update any `#[cfg(test)]` cells that asserted the L311 refusal" and runs only an in-file grep. But the refusal is asserted from TWO files:
- `crates/mnemonic-toolkit/src/wallet_import/sparrow.rs:892-910` — `parse_p2tr_singlesig_refused` (in-file unit test; uses `tr(@0/**)` SINGLESIG shape, which under the new heuristic still contains `@0/**` so falls into the substitution branch).
- `crates/mnemonic-toolkit/tests/cli_import_wallet_sparrow.rs:304-330` — `sparrow_taproot_singlesig_refused` (NOT in the file Step 5 greps; will silently fail until `cargo test --workspace` in Phase 3 Step 4).

Plus the module-docstring bullet at `tests/cli_import_wallet_sparrow.rs:13`: `refusal: taproot blob exits 2 (P1B descriptor-passthrough deferral)` is now stale.

**Fix:** plan-doc must enumerate BOTH tests + the docstring bullet, and explicitly direct conversion of `tr(@0/**)` singlesig → either success-path cell or a preserved narrow refusal. Otherwise Phase 3 Step 4 fails workspace-test count.

### C2 — Heuristic `!contains("@0/**")` does not cover taproot SINGLESIG (`tr(@0/**)`).

The plan claims "non-taproot wallets always have `@N/**` placeholders." TRUE. But `wallet_export/sparrow.rs:195` emits `CliTemplate::Bip86 => "tr(@0/**)"` — taproot SINGLESIG with `@0/**` placeholder. So `tr(`-containing inputs are not equivalent to descriptor-passthrough.

Under the proposed path-split, `tr(@0/**)` falls into the substitution branch; whether the resulting `tr([fp/path]xpub/<0;1>/*)` round-trips cleanly via `concrete_keys_to_placeholders` + `parse_descriptor` is UNVERIFIED.

**Fix:** either (a) add an explicit Bip86 happy-path cell + verify pipeline acceptance, or (b) preserve a narrow refusal for the `tr(@N/**)` singlesig case until coverage exists. Risk-register mentions only the converse (future emit-side change); does not flag the present-day Bip86 ambiguity.

**Recommended path: (b)** — preserve narrow refusal. Cycle 8 was scoped to descriptor-passthrough specifically; expanding to taproot singlesig is mid-cycle scope creep. File new FOLLOWUP `sparrow-taproot-singlesig-template-mode-import` at cycle close.

## Important (I)

### I1 — Source-SHA staleness in plan-doc.

Plan-doc cites SHA `4eb1fa8`; current master HEAD is `b29029f`. All line citations re-verified at `b29029f`: `wallet_import/sparrow.rs:304-315` refusal block, `:52-56` module docstring, `:209-212` parse-fn docstring Step 6, `wallet_export/sparrow.rs:215-219` — ALL match. Update plan-doc to `b29029f` per CLAUDE.md "Document the source SHA in the spec for future readers."

### I2 — Manual chapter §"Deferral — taproot import" location not pre-located.

Plan-doc Phase 4 Step 1 greps but the deferral section is at `docs/manual/src/45-foreign-formats.md:321-333` (`### Deferral — taproot import`) PLUS a deferrals-list bullet at `:813-815`. Both must convert to shipped-strikethrough; plan-doc Step 2 example shows only the bullet shape. Add explicit dual-update direction.

### I3 — Round-trip test cell does not actually round-trip.

The cell asserts only import-side success + envelope content substring; the docstring describes a full import→export round-trip but the implementation does not invoke `export-wallet`. Either re-author to actually round-trip via `--from-import-json`, or rename to `tr_multi_a_nums_2of3_envelope_carries_canonical_descriptor`.

### I4 — Integration cell `tr_multi_a_nums_2of3_sniffs_as_sparrow` provides redundant coverage.

Sniff is `policyType`-based (not script-content); identical to the existing template-mode sniff cells. Low value; consider dropping.

## Minor (M)

### M1 — Cell envelope-key trial chain over an underspecified envelope schema.
Pin the exact field per current `cmd/import_wallet/emit_json_envelope` rather than try-three-keys.

### M2 — Plan-doc "File structure" omits the integration-test file.
`tests/cli_import_wallet_sparrow.rs` carries `sparrow_taproot_singlesig_refused`. Add to modified list per C1.

### M3 — Risk register undercount-of-locations.
Risk register entry on existing unit tests only counts one file; expand to two per C1.

## Verdict

**YELLOW.** The plan-doc's structural design (path-split + reuse existing pipeline) is sound and primary-source-verified. However, C1+C2 are blocking: the heuristic does not classify taproot SINGLESIG correctly, AND the test-conversion direction omits the integration-test file. Both are recoverable with targeted fold (enumerate both refusal tests + explicit narrow refusal for `tr(@N/**)` template-mode + file follow-on FOLLOWUP). After fold, plan re-converges to GREEN.
