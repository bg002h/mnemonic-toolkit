# cycle-prep recon — 2026-05-24 — export-wallet-from-import-json-template-format-reemit + manual-prose-command-execution-gate

**Origin/master SHA at recon time:** `36e6bfa`
**Local branch:** `master`
**Sync state:** up-to-date (0 ahead / 0 behind)
**Untracked:** `.claude/`

Slug(s) verified: `export-wallet-from-import-json-template-format-reemit`, `manual-prose-command-execution-gate`. Both clean on cited paths/lines; TWO substantive findings extend the FOLLOWUP bodies (a third multisig-ambiguity pair + a stronger ambiguity-free fix source). Coupled: slug 1 unblocks 5/6 broken recipes that slug 2 then gates.

---

## Per-slug verification

### `export-wallet-from-import-json-template-format-reemit`
- **WHAT:** `export-wallet --from-import-json <env>` re-emits via the envelope descriptor for descriptor-passthrough formats (bitcoin-core/bip388/bsms) but REFUSES template-requiring formats (sparrow/coldcard/jade/electrum), and `--from-import-json conflicts_with --template`, so those formats cannot round-trip at all. Fix: auto-derive the template so they re-emit. MINOR.
- **Citations:**
  - `crates/mnemonic-toolkit/src/cmd/export_wallet.rs::run_from_import_json` — **ACCURATE**. Defined at `:540` (FOLLOWUP gave no line). The crux is its `EmitInputs { … template: None, … }` construction at **`:678`** (comment: "template is always None for descriptor-mode") — this hardcoded `None` is what denies the template-requiring emitters their required input.
  - `wallet_export/mod.rs:211 script_type_from_descriptor` — **ACCURATE**. `pub(crate) fn script_type_from_descriptor(` at `mod.rs:211`.
  - `script_type_from_template:191-203` (the multisig-ambiguity source) — **ACCURATE**. Lives at `mod.rs:191-203`; confirms singlesig 1:1 (Bip44→P2pkh `:193`, Bip49→P2shP2wpkh `:194`, Bip84→P2wpkh `:195`, Bip86→P2tr `:196`).
  - Refusal message "descriptor passthrough is not supported by <format>'s file-import surface" — **ACCURATE** (per-format, not a single string): `electrum.rs:54` ("…by Electrum's wallet-db schema"), `jade.rs:38` ("…by Jade's file-import surface"), `sparrow.rs:106` ("…by Sparrow's file-import surface"), `coldcard.rs:113` ("requires --template (bip44 / bip49 / bip84)…").
  - `--from-import-json conflicts_with --template` — **ACCURATE**. `conflicts_with_all = ["template", "descriptor"]` at `export_wallet.rs:171`.
  - Multisig inverse-ambiguity "P2wshMulti ← WshMulti|WshSortedMulti; P2trMulti ← TrMultiA|TrSortedMultiA" — **ACCURATE-BUT-INCOMPLETE**. There is a THIRD ambiguous pair the FOLLOWUP omits: **`P2shP2wshMulti ← ShWshMulti | ShWshSortedMulti`** (`mod.rs:198-200`). All three multisig script-types are non-invertible through `WalletScriptType`.
- **Action for brainstorm spec:**
  1. **Don't derive the template from the lossy `WalletScriptType` enum — derive it from the parsed descriptor.** The descriptor string is *unambiguous*: it literally distinguishes `multi` vs `sortedmulti` and `wsh(…)` vs `sh(wsh(…))`, whereas `script_type_from_descriptor` collapses both sorted/unsorted to one `WalletScriptType` (`mod.rs:229 Wsh(_) => P2wshMulti`). A new `descriptor → CliTemplate` mapping reading the miniscript structure dissolves the entire inverse-ambiguity the FOLLOWUP flags as the headline R0 risk. (The envelope already parses the descriptor at `export_wallet.rs:614` `parsed_ms`.)
  2. **Taproot is already walled off upstream of the gap.** `run_from_import_json` hard-refuses `P2tr | P2trMulti` at `export_wallet.rs:629-642` (BadInput, FOLLOWUP `wallet-import-taproot-internal-key`) *before* `EmitInputs` is built — so the `P2trMulti` ambiguity (and bip86 singlesig auto-derive) is **currently unreachable**. Live reachable surface = bip44/bip49/bip84 singlesig + wsh-multi|wsh-sortedmulti + sh-wsh-multi|sh-wsh-sortedmulti multisig. Scope the fix to non-taproot; note P2tr* re-emit stays blocked behind the separate taproot FOLLOWUP.
  3. The fix sets `EmitInputs.template = Some(derived)` on the from-import-json path; keep `conflicts_with_all` as-is (user omits `--template`; it's auto-derived — no override needed since the descriptor is authoritative). YAGNI on an override flag.
  4. **Lockstep:** chapter-45 recipes (slug 2's subject) must DROP the now-conflicting `--template` from the 5 broken recipes in the SAME PR. No clap flag-NAME change → **no GUI `schema_mirror` lockstep**. Cite source SHA `36e6bfa`.

### `manual-prose-command-execution-gate`
- **WHAT:** the manual lint validates flag NAMES + spelling + links + glossary + index but NEVER RUNS the documented commands; build a lint stage / integration test that extracts the round-trip recipes and runs them against the pinned binary.
- **Citations:**
  - `docs/manual/tests/lint.sh` "6 stages" — **ACCURATE**. Uses an `N/6` step convention (e.g. `step "3/6 lychee"` at `:55`). Executable, 5237 bytes.
  - lychee `--include-fragments` suggestion — **ACCURATE (absent, as the FOLLOWUP implies)**. lint.sh runs `lychee --offline --no-progress` at `:57` with NO `--include-fragments` → intra-doc `#anchor` links are unchecked today; adding the flag is net-new.
  - `docs/manual/src/45-foreign-formats.md` round-trip recipes — **ACCURATE**. 875 lines. All 6 cited recipe lines are `mnemonic export-wallet --from-import-json envelope.json` invocations, verbatim:
    - `:405` specter — `--wallet-name "Specter re-export"` (no `--template`) → **WORKS**.
    - `:313` sparrow `--template bip84`; `:481` coldcard `--template bip84`; `:752` electrum `--template bip84` → **BROKEN (singlesig)**.
    - `:564` coldcard `--template wsh-sortedmulti --threshold 2`; `:639` jade `--template wsh-sortedmulti --threshold 2` → **BROKEN (sorted multisig)**.
  - `design/AUDIT_FINDINGS_manual_v0_28_0_content.md` (the v0.28.1 prior breakage record) — **ACCURATE** (exists, 22630 bytes).
- **Action for brainstorm spec:** the broken set spans BOTH singlesig (`bip84`) and sorted-multisig (`wsh-sortedmulti`) — the gate's expected-success assertions must cover both once slug 1 lands. The gate can land covering the WORKING recipes first (specter via `--wallet-name`; descriptor-passthrough re-emits to bitcoin-core/bip388/bsms), then expand to all 6 after slug 1. Consider folding the `lychee --include-fragments` add into this cycle (cheap, same lint.sh). Cite source SHA `36e6bfa`.

---

## Cross-cutting observations
1. **The FOLLOWUP's headline R0 risk (multisig inverse-ambiguity) is dissolvable, not just budget-able.** Deriving `CliTemplate` from the parsed descriptor (which retains multi/sortedmulti + wsh/sh-wsh distinctions) is unambiguous; the ambiguity only exists if you route through the lossy `WalletScriptType`. This reframes the fix from "scope to singlesig first / carry the precise template in the envelope" (FOLLOWUP's two escape hatches) to "read the descriptor, which already carries everything." Surfacing this at recon saves an R0 fold round.
2. **Third omitted ambiguity pair:** `P2shP2wshMulti ← ShWshMulti|ShWshSortedMulti` (`mod.rs:198-200`) — the FOLLOWUP lists only 2 of the 3 non-invertible multisig script-types. Moot under the descriptor-driven approach, but record it so a `WalletScriptType`-based implementer doesn't ship a 2-of-3 fix.
3. **Taproot is pre-walled:** `P2tr|P2trMulti` are refused at `export_wallet.rs:629-642` before `EmitInputs`, so bip86 + tr-multi-a re-emit stays blocked on the separate `wallet-import-taproot-internal-key` FOLLOWUP regardless of this fix. The cycle must NOT claim to fix taproot round-trips.
4. **Coupling / ordering is hard:** the 5 broken recipes can only be un-broken by slug 1; slug 2's gate can't assert their success until then. Slug 1 first.
5. **Sync clean**, no DRIFTED-by-N findings; both slugs' line citations are exact at `36e6bfa`.

---

## Recommended brainstorm-session scope
- **Cycle A — slug 1 (`export-wallet-from-import-json-template-format-reemit`), MINOR.** New `descriptor → CliTemplate` derivation (non-taproot) wired into `run_from_import_json`'s `EmitInputs.template` (currently `None` at `:678`). ~50–90 LOC core + per-format round-trip tests (BIN-target, per memory) for bip44/49/84 + wsh-multi/wsh-sortedmulti + sh-wsh-multi/sh-wsh-sortedmulti. **Lockstep IN-PR:** strip `--template` from the 5 chapter-45 recipes (`:313/:481/:564/:639/:752`). **No GUI schema_mirror** (no flag-name change). SemVer MINOR (new export-wallet behavior). Own opus R0.
- **Cycle B — slug 2 (`manual-prose-command-execution-gate`), PATCH (test/CI infra).** New lint stage / integration test running the chapter-45 recipes against the pinned binary; optionally add `lychee --include-fragments`. Land AFTER Cycle A so it can assert all 6 recipes succeed (else it ships asserting only the 4 working ones and must be revisited). Own opus R0.
- **Ordering:** A → B, sequential (not parallel — slug 2 depends on slug 1's behavior). Recommend two separate cycles/PRs over one combined, since A is feature code under SemVer-MINOR and B is test infra under PATCH; bisect hygiene + independent R0 scopes favor the split.
- **No sibling-codec companions** (toolkit + manual only; both live in this repo).
