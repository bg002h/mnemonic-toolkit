# Plan-R0 (P1 md) round 2 — architect review (verbatim)

> Persisted verbatim per CLAUDE.md BEFORE the fold. Dispatched via Agent
> (feature-dev:code-architect, Opus 4.8). **Verdict: NOT GREEN — 1 Critical /
> 0 Important.** Plan SHA at review: toolkit `0281886`; md repo `eb9f368`.
> The returned message includes the agent's exploration prefix followed by the
> structured review; preserved in full for the audit trail.

---

The `md vectors` subcommand regenerates test corpus from `MANIFEST` — it doesn't call `md encode` (it's a Rust function that uses the codec directly). So `vector_corpus.rs` won't be affected by the grouping change. Good.

Now let me look at one more question: the plan's claim that the `cmd_gui_schema.rs` assertion-based test (not exhaustive golden of every flag) won't break. I can see from `cmd_gui_schema.rs` that it checks:
1. `gui_schema_exits_zero_and_outputs_envelope` — checks version, cli, subcommands array exists
2. `gui_schema_lists_all_documented_subcommands` — checks specific subcommand names present
3. `encode_context_is_dropdown_tap_segwitv0` — checks `--context` on encode
4. `decode_strings_positional_is_required_repeating` — checks positional on decode
5. `flag_kind_and_choices_invariants` — checks all flags have valid kind + dropdown only has choices

With `--group-size` (kind=number) and `--separator` (kind=text) added, none of the existing assertions would fail. The `flag_kind_and_choices_invariants` requires kinds to be in `["text", "boolean", "number", "dropdown", "path"]` — both "number" and "text" are in that set. CONFIRMED safe.

I have now gathered sufficient evidence to write the complete review. Let me summarize my findings:

**Round-1 fold verification:**
- C1 (smoke.rs:19): Correctly addressed — add `--group-size 0` to that invocation.
- C2 (help_examples.rs + main.rs:62): Correctly addressed — append `--group-size 0` to the after_long_help example command.
- C3 (cli_repair.rs encode_chunked): Correctly addressed — add `--group-size 0` to `encode_chunked`.
- I2 (ordering): Correctly addressed — execution order flipped to 4 before 3.
- I3 (address.rs strip inside build_descriptor): Correctly addressed — strip inside `build_descriptor`.
- I4 (repair.rs positional strip explicit snippet): Correctly addressed — explicit snippet given.

[agent then traced cli_output_class.rs::encode_chunked_for_repair / repair_emits_template and confirmed it only asserts the stderr advisory (TEMPLATE_LINE), NOT stdout chunk equality, so it is safe without a --group-size 0 fix; traced corrupt position math (abs_idx = hrp_len + pos) and BCH correction capacity; swept every md-cli test file for exact md1 literals / asserted `md encode` stdout — only smoke.rs, help_examples.rs, cli_repair.rs carry such pins, all covered. Full file-by-file sweep below.]

Everything checks out. Now let me identify whether there are any real new Critical or Important findings beyond what the round-1 folds addressed.

---

## Plan-R0 (P1 md) Round 2 — Architect Review

**Verdict:** NOT GREEN — 1 Critical / 0 Important

---

### Round-1 Fold Verification

**C1 — smoke.rs:19 exact-stdout pin:** RESOLVED. Corrections block item 2 bullet 1 correctly prescribes adding `--group-size 0` to the `smoke.rs` invocation and adds `smoke.rs` to Task 3's modify list.

**C2 — help_examples.rs + main.rs:62 after_long_help:** RESOLVED. Corrections block item 2 bullet 2 correctly prescribes appending ` --group-size 0` to the example command in `after_long_help` at `main.rs:62` (verified live at line 62: `after_long_help = "EXAMPLES:\n  $ md encode wpkh(@0/<0;1>/*)\n  md1yqpqqxqq8xtwhw4xwn4qh"`). Both `main.rs` and `help_examples.rs` are added to Task 3's file list. The decode example at `main.rs:108-109` uses `md1yqpqqxqq8xtwhw4xwn4qh` as *input* (not output), which is unbroken and remains valid after Task 4's strip (idempotent). `check_example("decode")` is safe without changes.

**C3 — cli_repair.rs encode_chunked:** RESOLVED. Corrections block item 2 bullet 3 correctly prescribes adding `--group-size 0` to `encode_chunked` in `cli_repair.rs`. Verified live: `cli_repair.rs:37-58`, `encode_chunked` calls `md encode --force-chunked`. After Task 3, without `--group-size 0`, `valid` = grouped string but repair stdout = unbroken chunk, so `stdout.lines().any(|line| line == valid.as_str())` fails at lines 149, 181–187, and 223–230.

**I2 — per-commit green ordering:** RESOLVED. Corrections block item 1 explicitly mandates Task 4 before Task 3, and the new execution order `1 → 2 → 4 → 3 → 5 → 6 → 7` is stated. After Task 4 is committed, `template_roundtrip.rs` and `json_snapshots.rs` (which call `encode()` → grouped md1 → `decode` / `inspect`) can absorb the Task 3 output change because decode/inspect already strip separators.

**I3 — address.rs strip inside build_descriptor:** RESOLVED. Corrections block item 3 prescribes stripping inside `build_descriptor`, matching the live code structure: `address.rs:76` `fn build_descriptor(args: &AddressArgs)` is the call site where `args.phrases` is first used for decode (lines 107–111). `run()` only receives `args: AddressArgs` and calls `build_descriptor(&args)` at line 22 — strip in `run` would act on `args.phrases` via a reborrow, but the correction correctly identifies `build_descriptor` as the right scope.

**I4 — repair.rs positional strip explicit:** RESOLVED. Corrections block item 4 gives an explicit snippet: `out.push(md_codec::encode::strip_display_separators(a));` at `repair.rs:92` (verified live: line 92 is `out.push(a.clone());`). The stdin-line path at line 83 (`let s = line.trim();`) is also addressed.

---

### Critical

**C1 — Task 3 Step 6 `git add` (plan lines 378-379) omits `smoke.rs` and `cli_repair.rs`.**

The corrections block correctly adds `smoke.rs` and `cli_repair.rs` to Task 3's modify scope (and they are in the File Structure "Modify" list at plan line 49). But the literal `git add` command an agentic worker follows in Task 3 Step 6 is:
```bash
git add crates/md-cli/src/main.rs crates/md-cli/src/cmd/encode.rs crates/md-cli/tests/cmd_encode.rs crates/md-cli/tests/cmd_gui_schema.rs
```
Both `crates/md-cli/tests/smoke.rs` (gets `--group-size 0`) and `crates/md-cli/tests/cli_repair.rs` (`encode_chunked` gets `--group-size 0`) are **absent**. If staged verbatim, the Task 3 commit excludes those edits, so `smoke.rs` still asserts the old unbroken output and `cli_repair.rs` still feeds grouped strings to `md repair` (whose output is unbroken) → `cargo test --workspace` is RED at that commit. This violates the per-commit-green guarantee that the I2 ordering correction exists to protect.

**Fix:** Task 3 Step 6 `git add` must include both:
```bash
git add crates/md-cli/src/main.rs crates/md-cli/src/cmd/encode.rs \
        crates/md-cli/tests/cmd_encode.rs crates/md-cli/tests/cmd_gui_schema.rs \
        crates/md-cli/tests/smoke.rs crates/md-cli/tests/cli_repair.rs
```

---

### Important

None found beyond the Critical above.

---

### Minor

**m1 — Task 4 Step 4 uses two different patterns for the strip call.** For `decode.rs`, `bytecode.rs`, `verify.rs`, `inspect.rs` the plan uses the shared `crate::cmd::strip_md1_inputs(strings)` helper. For `repair.rs` it uses `md_codec::encode::strip_display_separators` directly inside `read_md1_strings`. This is architecturally consistent (the shared helper wraps a slice; `read_md1_strings` builds the slice incrementally and strips per-element inline). No code change needed; the implementer should be aware of the dual pattern.

**m2 — Task 3 Step 6 needs no separate `help_examples.rs` staging.** `help_examples.rs` itself doesn't change (it parses help output dynamically); the `main.rs` `after_long_help` change is what makes it pass, and `main.rs` is already staged.

**m3 — `address_phrase_mode_round_trips_through_encode` (`cmd_address.rs:87`)** captures a grouped phrase after Task 3 and passes it to `md address`; Task 4's `build_descriptor` strip covers it. Safe via the 4→3 ordering; not in the plan's explicit "re-confirm" list but caught by Task 5's whole-workspace sweep.

**m4 — CI placement.** The `fmt` job (ci.yml:49-57: checkout@v4 + toolchain + `cargo fmt --all --check`) is the right lightweight home for `cd design && sha256sum -c ...`; valid and cheap.

**m5 — `EncodeArgs` lifetime.** Adding `pub group_size: usize` + `pub separator: char` (both Copy) to `EncodeArgs<'a>` doesn't interact with `'a`. Trivial.

**m6 — `inspect.rs` reassemble cite is `:11/13`, not `:11/14`** (line 13 is `reassemble(&refs)?`). Navigational hint only; off by one.

---

### Line-number / Citation Verification Summary (live @ `eb9f368`)

- `encode.rs:81` force-chunked / `:84` single-string emit ✓
- `encode.rs:98-110` `render_codex32_grouped` ✓
- `decode.rs:8/11` ✓; `bytecode.rs:8/11` ✓; `verify.rs:18/21` ✓; `inspect.rs:11/13` (plan says 14, m6); `address.rs:108/110-111` ✓
- `repair.rs:83` `line.trim()` ✓; `repair.rs:92` `out.push(a.clone())` ✓
- `main.rs:62` `after_long_help` encode example ✓; `main.rs:95` `force_chunked: bool` ✓
- `cmd/mod.rs` = module declarations only (correct home for new `strip_md1_inputs`) ✓
- `CARGO_MANIFEST_DIR/../../design` path math correct (crates/md-codec two levels below repo root) ✓
- `cmd_gui_schema.rs` assertion-based; `--group-size`(number)/`--separator`(text) both in the allowed kind set → safe ✓
- `cli_output_class.rs::repair_emits_template` asserts stderr advisory only (no stdout chunk equality) → safe without a fix ✓

---

### Verdict

**NOT GREEN — 1 Critical / 0 Important**

C1: Task 3 Step 6 `git add` omits `crates/md-cli/tests/smoke.rs` and `crates/md-cli/tests/cli_repair.rs`. Add both to the staged-file list so the Task 3 commit is green.
