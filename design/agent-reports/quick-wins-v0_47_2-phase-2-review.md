# Phase 2 (GREEN) Review — quick-wins PATCH v0.47.2

> Persisted verbatim from the opus `feature-dev:code-reviewer` agent
> (`agentId: a22301864de4eb880`). The review environment exposed no Bash, so the
> four executable gates were marked NOT RUN; the **operator addendum** at the
> bottom records those gates run GREEN plus the M-1 fold.

---

## VERDICT: 0 Critical / 0 Important on code correctness (+ 1 Minor)

**Important caveat on the gates:** This review environment exposes **no command-execution (Bash) tool** — my function set is Read/Glob/Grep/WebFetch/WebSearch/advisor only. The task's "You have Bash" is not honored by the harness. I therefore **cannot run** `cargo test`, `cargo clippy`, or `make -C docs/manual audit`, and I will not fabricate numbers. The static correctness review is complete and clean; the empirical gate pass/fail is **NOT RUN** and must be confirmed by the caller (commands below).

---

## Statically verified clean (I stand behind these)

### Slug 1 — manual reword (TRUE, not a new falsehood)
All 6 edits present and consistent in `docs/manual/src/40-cli-reference/41-mnemonic.md`:
- Repair synopsis `:2744` and inspect synopsis `:3016` are the SPEC-curated independently-optional form (`[--ms1 <MS1>] [--mk1 <MK1> [--mk1 <MK1>...]] [--md1 <MD1> [--md1 <MD1>...]] [--json]`, inspect adds `[--reveal-secret]`). No false "pick exactly one" constraint.
- Repair rows `:2751/:2752/:2753` and inspect `--ms1` row `:3023` all reworded to "may be combined with ... (one HRP per card; per D35)".
- **Ground truth confirms the reword is TRUE:** `repair.rs:30-53` and `inspect.rs:23-36` both define `ArgGroup::new("kind").args(["ms1","mk1","md1"]).required(false).multiple(true)` with **no `conflicts_with_all`**; `inspect.rs:24-26` explicitly comments "mixed-HRP invocations are valid (`mnemonic inspect ms1xxx mk1yyy md1zzz`)".
- **No stray "mutually exclusive" left on these flags.** The other 6 hits (`:59/:60/:774/:1006/:1270/:1274`) are genuinely-mutex unrelated flags (`--template`/`--descriptor-file`, `--passphrase-stdin`, `--slot` vs `--ms1[N]` same-N, `--ciphertext`, password forms) — correctly untouched.

### Slug 2 — import-wallet argv advisory (the I2 point is correct)
`import_wallet.rs:288-295`:
- **The @env: skip reads RAW args BEFORE the rebind (I2):** advisory at `:288/:291` runs before the `env_resolved_owned` shadowing rebind at `:297-304`. It iterates the original `args.ms1` / `args.slot`. Skip conditions `!v.is_empty() && !v.starts_with("@env:")` (ms1) and `s.subkey == SlotSubkey::Phrase && !s.value.is_empty() && !s.value.starts_with("@env:")` (slot) are correct.
- **The negative cell is genuinely discriminating, not vacuous:** `ms1_env_sentinel_no_argv_advisory` (`cli_import_wallet_seed_overlay.rs:534-554`) sets `MNEMONIC_TEST_MS1_ENV` to a real ms1, passes `--ms1 @env:...`, asserts `.success()` AND no advisory. A read-after-rebind bug would see the resolved secret (not `@env:`-prefixed) and fire — failing this test. Real guard.
- Slot label uses the ACTUAL index (`format!("--slot @{}.phrase=", s.index)`); `--ms1` fires at most once (single `if … any()`); flag label has no trailing space (`"--ms1"`).
- `""` watch-only sentinel correctly does NOT fire (`!v.is_empty()`). Inline `--ms1 <value>` fires even on later-erroring paths (advisory is at top-of-run, before all early returns).
- Faithful to the established `bundle.rs:2115-2128` and `verify_bundle.rs:1061-1076` precedent; `bundle`'s slot-@env: skip is independently proven by `cli_env_var_sentinel.rs:833-856` (those tests are `bundle`, unaffected by this change). No double-fire.

### Slug 3 — convert electrum→address redirect
- Helper `refusal_electrum_phrase_to_address()` at `convert.rs:570-574` returns `ConvertRefusal` (exit code preserved — same variant as `refusal_one_way`).
- Arm at `:703-705` placed **before** the `:714` one-way catch-all and **after** the `(Phrase, ElectrumPhrase)|(ElectrumPhrase, Phrase)` pivot arm `:692-697` (which does not match `(ElectrumPhrase, Address)`). No earlier arm intercepts it. Correct placement.
- Redirect message contains `addresses --from electrum-phrase` (the real invocation). No OTHER electrum edge changed: `(ElectrumPhrase, Entropy)` / `(Entropy, ElectrumPhrase)` (`:651-652`) and the pivot arm are intact.
- `unreachable!("classify_edge intercepts (ElectrumPhrase, !Entropy)")` at `:1615` invariant still holds — `classify_edge` gates before execution dispatch, so `(ElectrumPhrase, Address)` never reaches it. No new panic risk.
- M1 stale comment reworded (`cli_convert_electrum.rs:556-563`); test strengthened to assert both `.failure()` and `contains("addresses --from electrum-phrase")` (`:578-585`).

### Scope (partially verified)
The 3 expected source files carry exactly the SPEC'd edits with **no clap-surface change** (no `#[arg]`/`Args`-struct/subcommand/value-enum change) — confirming "no GUI `schema_mirror`". Slug 2 reuses existing fields; slug 3 adds only a helper fn + a match arm. **Caveat:** without `git diff --stat` I cannot rule out stray edits in files I did not inspect; the cross-file scope check is Bash-gated.

---

## Minor (does not block)
**M-1 — missing dedicated import-wallet slot-`@env:` negative cell.** SPEC §5(b) called for `--slot @0.phrase=@env:VAR` → no advisory, but `cli_import_wallet_seed_overlay.rs` has no such cell (grep-confirmed absent). The slot `@env:` skip at `import_wallet.rs:292` is proven only by inference from the ms1 negative cell (different field) plus the `bundle` twin's passing `cli_env_var_sentinel.rs:833-856`. Code is correct; coverage is slightly thinner than spec. Suggested fix (optional): add `slot_phrase_env_sentinel_no_argv_advisory` mirroring the ms1 negative.

---

## NOT RUN — caller must confirm before declaring GREEN
I could not execute these. Run them and confirm the expected outcomes:

```
cargo test -p mnemonic-toolkit --no-fail-fast 2>&1 | grep -E 'test result|FAILED' | grep -v '0 failed'   # expect EMPTY
cargo clippy -p mnemonic-toolkit --all-targets 2>&1 | grep -c warning                                     # expect 0
# 4 named cells pass: ms1_inline_value_fires_argv_advisory, ms1_env_sentinel_no_argv_advisory,
#   slot_phrase_inline_fires_argv_advisory_with_actual_index, electrum_phrase_to_address_is_refused
MNEMONIC_BIN=/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic \
  MD_BIN=/scratch/code/shibboleth/descriptor-mnemonic/target/debug/md \
  MS_BIN=/scratch/code/shibboleth/mnemonic-secret/target/debug/ms \
  MK_BIN=/scratch/code/shibboleth/mnemonic-key/target/debug/mk \
  make -C docs/manual audit                                                                               # expect GREEN
git show 1ae6586 --stat ; git diff 73173bc~1..HEAD --stat                                                 # expect only 3 src + 2 test + design files
```
(The four CLI bins all exist at the paths above — audit precondition met. `make audit` = `lint verify-examples anchor-check`, a real target.)

---

## Bottom line
On code correctness against SPEC + source: **0 Critical / 0 Important / 1 Minor**. Every substantive correctness question (I2 raw-args read, non-vacuous negative cell, arm placement, exit-code preservation, manual truthfulness, no stray mutex, no clap-surface drift) is verified clean by static analysis.

**I cannot issue an unconditional "GREEN — proceed to Phase 3" because that assertion is defined by the executable gates passing, which I am not equipped to confirm.** Conditional verdict: **once the four NOT-RUN gates pass as specified above, Phase 2 is GREEN and may proceed to Phase 3 (release).** If any gate is red, re-engage. The relevant files are:
- `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` (`:288-295`)
- `crates/mnemonic-toolkit/src/cmd/convert.rs` (`:570-574`, `:703-705`)
- `docs/manual/src/40-cli-reference/41-mnemonic.md` (`:2744`, `:2751-2753`, `:3016`, `:3023`)
- `crates/mnemonic-toolkit/tests/cli_import_wallet_seed_overlay.rs` (`:516-577`)
- `crates/mnemonic-toolkit/tests/cli_convert_electrum.rs` (`:553-586`)

---

## Operator addendum (gates run + M-1 fold) — 2026-06-06

The four NOT-RUN gates were run by the operator and all pass GREEN:

1. **Full suite** — `cargo test -p mnemonic-toolkit --no-fail-fast`: **0 failed** (lib 878 passed; every integration bin GREEN; 0 unexpected ignored). The failure-grep returned EMPTY.
2. **Clippy** — `cargo clippy -p mnemonic-toolkit --all-targets`: 0 warnings (confirmed in the Phase-2 pre-review gate run).
3. **Manual audit** — `make -C docs/manual audit` with the four CLI bins pinned: GREEN (lint + verify-examples + anchor-check).
4. **Scope** — Phase-2 diff = exactly the 3 SPEC'd source files (convert.rs, import_wallet.rs, 41-mnemonic.md) + the Phase-1 test cells; no stray edits, no clap-surface drift.

**M-1 fold applied (Minor → resolved):** added the dedicated negative cell
`slot_phrase_env_sentinel_no_argv_advisory` to
`crates/mnemonic-toolkit/tests/cli_import_wallet_seed_overlay.rs`
(`--slot @0.phrase=@env:MNEMONIC_TEST_SLOT_PHRASE` → asserts no argv advisory).
Cell run GREEN (1 passed); full suite re-run GREEN after the fold.

**Phase 2 is GREEN (0C/0I, M-1 folded) — cleared to proceed to Phase 3 (release).**
