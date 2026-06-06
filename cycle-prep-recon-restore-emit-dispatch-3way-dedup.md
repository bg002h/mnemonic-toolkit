# cycle-prep recon — 2026-06-05 — restore-emit-dispatch-3way-dedup

**Origin/master SHA at recon time:** `33db764`
**Local branch:** `master`
**Sync state:** `up-to-date (0 ahead / 0 behind)`
**Untracked:** recon/survey scratch + `.claude/` (none load-bearing).

Slug verified: `restore-emit-dispatch-3way-dedup`. **HAS A STRUCTURAL COUNT ERROR** — there are **4** dispatch copies, not 3, and they are NOT all byte-identical (the single-sig restore copy diverges in one arm).

---

## Per-slug verification
### restore-emit-dispatch-3way-dedup
- **WHAT (from FOLLOWUPS.md):** the 11-arm `collect_missing`→refuse→`emit` `WalletFormatEmitter` dispatch exists in "3 byte-identical copies"; consolidate into one `wallet_export::emit_payload(inputs: &EmitInputs, format: CliExportFormat) -> Result<String>` consumed by all sites.
- **Citations:**
  - `export_wallet.rs:506-560` (`run`) — **DRIFTED + UNDERCOUNT.** `run`'s dispatch is `collect_missing` at `:506` + `emit` at `:527` (coldcard-multisig 6-variant template match at `:531`). ACCURATE as a site, but it is only **1 of 2** dispatches in this file.
  - **UNCITED 4th copy:** `export_wallet.rs:760` (`collect_missing`) + `:783` (`emit`, coldcard-multisig at `:787`) lives in **`run_from_import_json`** (`:584`). The FOLLOWUP missed it entirely. **STRUCTURALLY-WRONG count: 3 → 4.**
  - `restore.rs` single-sig `build_import_payload` (~`:587-660`) — **DRIFTED + NOT byte-identical.** `build_import_payload` at `:587`; `collect_missing` at `:624`; `emit` at `:645`. Its coldcard-multisig emit arm (`:649`) is `Err(bad("--format coldcard-multisig requires a multisig wallet; restore is single-sig — use --format coldcard"))` — **DIFFERENT** from the 6-variant `CliTemplate` template match in the other three. So "byte-identical" is FALSE for this copy's emit half.
  - `restore.rs` multisig `build_multisig_import_payload` (~`:662-760`) — **DRIFTED-by-~11.** Actually `:673` (fn); `collect_missing` `:705`; `emit` `:728` (coldcard-multisig 6-variant match at `:732`, byte-identical to export-wallet). Cited range `~:662-760` is loose but in the right region.
  - **`collect_missing` half IS byte-identical** across all 4 (spot-checked `BitcoinCore` arm: `export_wallet.rs:508` == `restore.rs:626` == `restore.rs:707`; the from-import-json copy matches). Only the **emit** half diverges, and only for single-sig restore's coldcard-multisig arm.
  - **No test pins the single-sig coldcard-multisig message** — grep for "restore is single-sig — use --format coldcard" across `tests/` + `src/` (excluding the definition) returns nothing. So changing that arm's message is low-risk (R0/Phase-1 must still confirm no cli_restore cell exercises `--format coldcard-multisig` on a single-sig restore and asserts the exit/message).
- **Action for brainstorm spec:** correct the count to **4 sites** (`export_wallet.rs` `run`:527 + `run_from_import_json`:783; `restore.rs` `build_import_payload`:645 + `build_multisig_import_payload`:728). Extract `wallet_export::emit_payload(inputs: &EmitInputs, format: CliExportFormat) -> Result<String>` = the byte-identical `collect_missing`-first + the 6-variant-coldcard-multisig `emit` dispatch. Each call site keeps its own `EmitInputs` construction (the genuinely-differing part) and calls `emit_payload(&inputs, format)`. **Resolve the single-sig restore coldcard-multisig divergence explicitly:** (a) accept the unified message (single-sig template → the 6-variant `_ => BadInput("requires a multisig --template …")`; same exit 1, reworded — no test breaks found), OR (b) keep a single-sig pre-check in `build_import_payload` to preserve the restore-specific message. Recommend (a) (simpler; the unified refusal is still clear) + note the user-visible string change. Cite source SHA `33db764`.

---

## Cross-cutting observations
1. **The FOLLOWUP undercounts (3 vs 4)** and overstates "byte-identical" (true for 3 of 4; the single-sig restore emit arm differs). Both must be corrected in the SPEC — this is exactly the [[feedback_r0_must_read_source_off_by_n]] / count-ambiguity class.
2. **Two distinct factorings, don't conflate.** The `collect_missing` half is universally identical → trivially shared. The `emit` half is identical for 3 and divergent for 1 → the shared helper uses the 6-variant branch and the single-sig restore site either accepts the reworded refusal (a) or pre-checks (b). The **`EmitInputs` construction is NOT shared** (each site builds it differently — that's the legitimate per-site code).
3. **Behavior change is contained to one message** (`restore --format coldcard-multisig` on a single-sig template). No exit-code change, no other format affected. Ungated by any test (grep-confirmed) — but R0/Phase-1 must re-confirm against the cli_restore cells.
4. The companion `descriptor-origin-extraction-dedup` is a SEPARATE, larger dedup (6 import parsers + regex unification) — NOT in this cycle.

---

## Recommended brainstorm-session scope
**Single tight refactor cycle.** `restore-emit-dispatch-3way-dedup` (corrected: **4-way**). **SemVer: PATCH** (pure refactor; the lone user-visible change is the single-sig `restore --format coldcard-multisig` refusal wording — behavior-equivalent, reworded; note it in the CHANGELOG). **Size: net-negative LOC** — remove 4× (~20-line collect_missing + ~35-line emit) dispatch ≈ −180, add one ~70-line `emit_payload` helper + 4 call-site replacements (~5 lines each) + possibly 1 test/CHANGELOG note. **Locksteps: NONE** — no clap surface change (no flag/value/subcommand), so **no GUI `schema_mirror`**, **no manual mirror** (the `--format` value set is unchanged), no sibling-codec change. **TDD:** Phase 1 RED is light (the existing export-wallet + restore + from-import-json `--format` cells ALREADY cover the dispatch behavior — a green-stays-green refactor; add one cell pinning the chosen single-sig coldcard-multisig message so the rewrite is intentional). Per-phase opus review. **No inter-slug dependency** (descriptor-origin-extraction-dedup is independent + larger → separate cycle). R0 must lock the (a)-vs-(b) single-sig-message decision.
