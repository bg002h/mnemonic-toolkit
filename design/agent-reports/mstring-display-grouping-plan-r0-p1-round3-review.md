# Plan-R0 (P1 md) round 3 — architect review (verbatim) — GREEN

> Persisted verbatim per CLAUDE.md. Dispatched via Agent (feature-dev:code-architect,
> Opus 4.8). **Verdict: GREEN — 0 Critical / 0 Important.** Plan SHA at review:
> toolkit `44087f0`; md repo `eb9f368`. Verification round after the r2 fold;
> includes the agent's analysis prefix followed by the structured review.

---

Now I have a complete picture. Let me work through all the key findings:

**Round-2 C1 verification:**
The plan at lines 378-381 now reads:
```
git add crates/md-cli/src/main.rs crates/md-cli/src/cmd/encode.rs \
        crates/md-cli/tests/cmd_encode.rs crates/md-cli/tests/cmd_gui_schema.rs \
        crates/md-cli/tests/smoke.rs crates/md-cli/tests/cli_repair.rs
```
Both `smoke.rs` and `cli_repair.rs` are present. C1 is resolved.

**`help_examples.rs` in Task 3's `git add`:** Task 3 modifies `main.rs:62` (the `after_long_help` string). `help_examples.rs` itself doesn't change (it parses help output dynamically); the `main.rs` change (already staged) is sufficient. After Task 3 changes `after_long_help` to include `--group-size 0`, the example command becomes `md encode wpkh(@0/<0;1>/*) --group-size 0`, whose output is the unbroken wire-canary `md1yqpqqxqq8xtwhw4xwn4qh\n` — matching the parsed-expected. Stays green.

**Task 4 `git add` completeness (plan line 444):** all 7 source files (`cmd/mod.rs`, `decode.rs`, `bytecode.rs`, `verify.rs`, `inspect.rs`, `address.rs`, `repair.rs`) + all 6 test files (`cmd_decode.rs`, `cmd_verify.rs`, `cmd_inspect.rs`, `cmd_bytecode.rs`, `cmd_address.rs`, `cli_repair.rs`) present. `cli_repair.rs` in both Task 3 and Task 4 is not a conflict — execution order `1→2→4→3` makes them sequential independent commits.

**inspect.rs citation (round-2 m6):** plan now says `:11/:13`; live source line 13 is `reassemble(&refs)?`. Correct.

**Execution order intermediate-green:** `1→2→4→3→5→6→7`. After Task 4 (strip on all intake sites, encode still unbroken): `md encode | md decode` round-trips green (idempotent strip on clean input). After Task 3 (encode emits grouped): decode strips before decoding → green. `encode_chunked` with `--group-size 0` produces unbroken fixtures so `corrupt_at` never hits a space at `chars[abs_idx]` (no panic). Confirmed.

**Task 1 `git add`:** only `crates/md-codec/src/encode.rs` — the only file Task 1 edits. Correct.
**Task 2 `git add`:** TSV + `.sha256` + conformance test + ci.yml — all four. Correct.
**Task 6/7 `git add`:** `<followups path>` / `<changelog if any>` are intentional discovery-time placeholders guarded by preceding `git ls-files | grep` steps. Acceptable.

---

## Plan-R0 (P1 md) Round 3 — Architect Review

**Verdict: GREEN — 0 Critical / 0 Important**

---

### Round-2 Fold Verification

**C1 RESOLVED.** Task 3 Step 6 `git add` (plan lines 378-381) now stages both `crates/md-cli/tests/smoke.rs` and `crates/md-cli/tests/cli_repair.rs`. The fold is correct and complete.

---

### Critical

None.

### Important

None.

### Minor

**m1 — `help_examples.rs` does not need to be staged in Task 3 (confirmed).** `check_example` dynamically parses `md encode --help` at test runtime and replays the example verbatim. Only `main.rs:62` (`after_long_help`) changes — already staged. No content change to `help_examples.rs`.

**m2 — Task 4 `git add` (plan line 444) is complete.** All 7 source + 6 test files staged. `cli_repair.rs` in both Task 3 and Task 4 is fine: order `1→2→4→3` makes Task 4 commit its grouped-intake test first, Task 3 commit the `encode_chunked --group-size 0` fix next — independent, sequential.

**m3 — Intermediate commit greenness verified at every transition.** At Task 4's commit (intake strip landed, encode still unbroken): `encode_chunked` produces unbroken strings; `valid` unbroken matches repair's unbroken stdout. `template_roundtrip.rs`/`json_snapshots.rs` see unbroken encode → stripped decode (idempotent). GREEN. At Task 3's commit (encode now grouped): `encode_chunked` has `--group-size 0` (unbroken fixtures); `corrupt_at` never sees a space at `chars[abs_idx]`; decode + all intake sites strip. GREEN.

**m4 — `inspect.rs` cite `:11/:13` is correct** in the plan body (line 47) and Task 4 body. Live `inspect.rs:13` is `reassemble(&refs)?`. Round-2 m6 corrected.

**m5 — `address.rs` strip inside `build_descriptor` is architecturally sound.** Live `address.rs:76-113`: `build_descriptor` is where `args.phrases` is first consumed (line 108 `decode_md1_string(&args.phrases[0])`, line 110 `args.phrases.iter()`). `run()` (line 21) only calls `build_descriptor(&args)` — stripping in `run` would not reach the `phrases` field used inside `build_descriptor`. Correction item 3 is correct.

**m6 — Task 6/7 `git add` placeholders** (`<followups path>`, `<changelog if any>`) are intentional, guarded by `git ls-files | grep` discovery. Acceptable.

---

### Full `git add` Audit Summary

| Task | Staged files | Complete? |
|---|---|---|
| 1 | `crates/md-codec/src/encode.rs` | Yes (only file) |
| 2 | TSV, `.sha256`, `display_grouping_conformance.rs`, `ci.yml` | Yes (all 4) |
| 3 | `main.rs`, `cmd/encode.rs`, `cmd_encode.rs`, `cmd_gui_schema.rs`, `smoke.rs`, `cli_repair.rs` | Yes (all 6; `help_examples.rs` has no content change) |
| 4 | `cmd/mod.rs` + 6 intake `cmd/*.rs` + 6 test files | Yes (all 13) |
| 5 | `git add -p` (interactive) | Appropriate catch-all |
| 6 | `<followups path>` (discovery placeholder) | Appropriate |
| 7 | both `Cargo.toml`s + `Cargo.lock` + `<changelog if any>` | Appropriate |

**GREEN — 0 Critical / 0 Important.** Round-2 C1 fold confirmed. No new drift. All per-commit `git add` sets complete. Plan cleared for execution in order `1 → 2 → 4 → 3 → 5 → 6 → 7`.
