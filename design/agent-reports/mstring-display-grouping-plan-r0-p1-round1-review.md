# Plan-R0 (P1 md) round 1 — architect review (verbatim)

> Persisted verbatim per CLAUDE.md BEFORE the fold. Dispatched via Agent
> (feature-dev:code-architect, Opus 4.8). **Verdict: NOT GREEN — 3 Critical /
> 2 Important / (minors).** Plan SHA at review: toolkit `aeb74b9`; md repo `eb9f368`.

---

## Critical

**C1 — `crates/md-cli/tests/smoke.rs:19` pins exact unbroken md1.** `cmd.assert().success().stdout("md1yqpqqxqq8xtwhw4xwn4qh\n");` — a full-stdout equality. Default `--group-size 5` makes `md encode` emit `"md1yq pqqxq q8xtw hw4xw n4qh\n"` → RED. The plan's grep only scanned `cmd_encode.rs`, missing `smoke.rs`. **Fix:** add `smoke.rs` to the fixup sweep; add `--group-size 0` to its invocation (keeps the wire-canary pin) or update the expected string to grouped.

**C2 — `crates/md-cli/tests/help_examples.rs` `check_example("encode")` does exact-match** between the `after_long_help` embedded example output (`main.rs:62`: `"…md encode wpkh(@0/<0;1>/*)\n  md1yqpqqxqq8xtwhw4xwn4qh"`) and the live command output. Default grouping makes them mismatch. **Fix:** Task 3 must update `main.rs:62` `after_long_help` (grouped form, or run the example with `--group-size 0`) — file not currently in the plan.

**C3 — `crates/md-cli/tests/cli_repair.rs` fixtures.** `encode_chunked` captures `--force-chunked` output (now GROUPED) and feeds it to `md repair`, whose OUTPUT stays UNBROKEN (plan guarantee). Assertions like `stdout.lines().any(|line| line == valid.as_str())` (valid=grouped) vs unbroken repair output → fail (≈5 repair assertions). **Fix:** `encode_chunked` helper must use `--group-size 0` so fixtures are unbroken; add `cli_repair.rs` to Task 3/5 modify list.

## Important

**I2 — per-commit green ordering.** Task 3 (encode default grouping) commits before Task 4 (intake strip). Between them, `template_roundtrip.rs` + `json_snapshots.rs` (which `md encode | md decode`) break because decode rejects grouped input. **Fix:** land intake-strip (Task 4) BEFORE encode-default-grouping (Task 3) — reorder so decode accepts grouped before encode emits it — or combine; declare per-commit-green either way.

**I3 — `address.rs` strip site.** `run` calls `build_descriptor(args)`; the md1 decode is INSIDE `build_descriptor` (`address.rs:108/111` on `args.phrases`), not in `run`. **Fix:** strip inside `build_descriptor` (`let stripped = strip_md1_inputs(args.phrases);` used for the decode), not "top of run".

**I4 — `repair.rs` positional strip shown only in prose.** Step 4 gives the stdin-line strip snippet but only prose-mentions the positional path (`out.push(a.clone())` at `repair.rs:92`). **Fix:** explicit snippet `out.push(strip_display_separators(a));` for the positional case so stdin + positional strip consistently.

## Minor
m1 `cmd/mod.rs` gains its first `use` import (trivial). m2 `render_grouped("abcdefg",3,'-')→"abc-def-g"` confirmed correct. m3 `sha256sum -c` in the `fmt` CI job is valid (job does checkout). m7 descriptor-mnemonic CHANGELOG not CI-gated (courtesy only). The wrapper preserves `render_codex32_grouped` exact behavior so `vector_corpus.rs` corpus does not drift (confirmed). `--separator` as `char` + free-fn `value_parser` returning `Result<char,String>` is valid clap-4.5 (String: Into<Box<dyn Error…>>); classifies as "text" kind → gui-schema invariant passes.

## Verdict
NOT GREEN — 3 Critical / 2 Important. All mechanical: C1 smoke.rs, C2 help_examples.rs + main.rs:62 after_long_help, C3 cli_repair.rs encode_chunked → --group-size 0; I2 reorder Task4-before-Task3; I3 address strip inside build_descriptor; I4 explicit repair positional-strip snippet.
