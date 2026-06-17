# cycle-prep recon — 2026-06-10 — repeating-secret-flags-never-reach-argv (GUI)

**Origin/master SHA at recon time:** GUI `dabbdfe` (= tag mnemonic-gui-v0.31.0); sync 0/0
**Registry:** `mnemonic-gui/FOLLOWUPS.md` (repo root)

Slug verified: `repeating-secret-flags-never-reach-argv`. Entry filed 2026-06-09 (v0.30.0 cycle) with the impl-review-corrected census — expected fresh; verified fresh, but the recon OVERTURNS the entry's fix direction.

---

## Per-slug verification

### `repeating-secret-flags-never-reach-argv`

- **WHAT:** secret+repeating+Text flags render ONE `SecretLineEdit` into `state.secret_widgets[name]` (widget.rs secret branch fires before the repeating branch) while `assemble_argv`'s repeating-secret arm reads rows from `state.values` → live forms emit NOTHING for them.
- **Citations:**
  - Widget secret branch single-entry (`src/form/widget.rs:78` `entry(flag.name).or_default()`) — **ACCURATE**.
  - Assembler repeating-secret arm reads `state.values` (`src/form/invocation.rs:238-243`; scalar arm reads `secret_widgets.get(name)` `:243`) — **ACCURATE**.
  - Census `--ms1` ×2 (VERIFY_BUNDLE + IMPORT_WALLET, optional) + `--share` ×2 Text (SLIP39_COMBINE + MS_SHARES_COMBINE, **required:true**) — **ACCURATE** (re-censused; seed-xor `--share` NodeValueComposite counter-example confirmed).
  - **Fix direction "route repeating secrets through `state.values` (the v0.3 fold comment's design)" — STRUCTURALLY-UNSAFE, recon overturns it:** `redact_for_persistence` (`src/persistence.rs:67-105`) drops ONLY `SECRET_FLAG_NAMES` (the 3 passphrase flags) + secret NodeValueComposite node-types + secret slot subkeys. Schema-`secret: true` Text flags (`--ms1`, `--share`) are NOT in any drop list — they are protected today solely by never entering `state.values`. Routing them through `state.values` would **persist seed material to disk** under the current filter. The v0.3 fold comment predates the persistence layer's reliance on that invariant.
- **Action for SPEC:** invert the fix — make `secret_widgets` per-row (`BTreeMap<String, Vec<SecretLineEdit>>`) and point the assembler's repeating-secret arm at it; the never-persist invariant stays **type-level** (`secret_widgets` is `#[serde(skip)]` + freshly-defaulted in redaction, persistence.rs:99-103) and zeroize-per-row is preserved. Belt-and-suspenders: ALSO extend `redact_for_persistence` to drop schema-secret flag names from `values` (closes the whole future-drift class) + a drift test. Cite GUI SHA `dabbdfe`.

---

## Cross-cutting observations

1. ~4 `secret_widgets` call sites total (widget.rs:78, invocation.rs:243, mod.rs:295/:308, persistence.rs:103) — the Vec migration is contained.
2. `SecretLineEdit::show` uses a positional `TextEdit` (no explicit id) — N sequential rows get distinct positional auto-IDs (same safety basis as the v0.30.0 Text rows); no salt work needed, note it.
3. `--share` Text sites are `required: true` → the v0.30.0 required-seed rule applies (one row always); `--ms1` optional → zero-seed.
4. The existing kittest/unit cells that "synthesize state.values directly" (e.g. `cell_import_wallet_repeating_ms1_argv`) pin the CURRENT assembler contract — the fix CHANGES that contract (repeating secrets stop reading values) → those cells must be MIGRATED to the new secret_rows source, not deleted (they become the regression pins for the fixed path).

---

## Recommended brainstorm-session scope

One GUI cycle, **PATCH v0.31.1** (bug fix; no flag-name change → no schema_mirror delta; no toolkit involvement). ~150-250 src LOC + tests. SPEC → R0 → implement → ship (CHANGELOG + tag + toolkit install.sh GUI pin follow-up).
