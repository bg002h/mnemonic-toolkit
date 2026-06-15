# Plan-R0 (P2 ms) round 2 — architect review (verbatim) — GREEN

> Persisted verbatim per CLAUDE.md. Dispatched via Agent (feature-dev:code-architect,
> Opus 4.8). **Verdict: GREEN — 0 Critical / 0 Important.** Plan SHA at review:
> toolkit `5d14d84`; ms repo `b616530`. Verification round after the r1 fold;
> swept all 48 test files + src `#[cfg(test)]` blocks. Includes the agent's
> analysis prefix + structured review.

---

### Round-1 fold verification
- **I1 `encode_no_engraving_card.rs:19`** — RESOLVED (named in corrections block + Task 3 fix list; space-strip-then-prefix or `--group-size 0`).
- **I2 `encode_hex_input.rs:13`** — RESOLVED (same).
- **I3 `encode_mnem_japanese.rs` (four length tests `:30`/`:74`/`:89`/round-trip)** — RESOLVED (`--group-size 0` keeps length unchanged).
- **I4 `cli_split.rs`** — RESOLVED (`--group-size 0` fix + concrete `split_grouped_default_labels_on_stderr` test spec).

### Completeness sweep — no missed breaking tests
Verified COVERED (scheduled): `encode_canonical_12_word.rs` (\n\n + prefix), `encode_canonical_24_word.rs` (prefix), `encode_output_unchanged_after_split_refactor.rs` (3 text `assert_eq!`; json unaffected), `format.rs` 3 `chunked_*` unit tests, `parse.rs::strip_whitespace_dedupes_doubled_content`.
Verified NOT breaking: `back_typed_chunked_form_decodes.rs` (space-strip survives), `encode_pipe_to_decode.rs` (EMIT-first ordering keeps green; spaces stripped by both strip_whitespace and strip_display_separators), `decode_routes_share_to_is_share_not_single_string.rs` (exit/stderr only), `cli_combine.rs` (`combine --to ms1` stays unbroken → `:94` filter `!contains(' ')` still finds it; other combine tests use `--json`/unbroken positionals), `gui_schema_emits_spec_v7_json.rs` (`find_flag` by name; no exhaustive count → new flags don't break it), `cli_output_class.rs` (stderr only), all `decode_*`/`inspect_*`/`verify_*`/`vectors_*`/`cli_repair`/`cli_derive`/`exit_codes_table`/`encode_arg_group_violations`/`encode_emits_passphrase_warning`/`encode_rejects_*`/`json_error_envelope`/`lint_zeroize`/`mlock_g6` (none assert encode/split stdout), `parse.rs::read_input_with_explicit_arg_returns_stripped` (`"  ms10  "`→`"ms10"` survives). **48 test files + src cfg(test) swept; no additional breaking tests.**

### Contradiction check
- I3 `--group-size 0` → `render_grouped` returns input unchanged → first-line length = 50/51 → length asserts pass. Sound.
- I1/I2 space-strip-then-prefix recovers canonical prefix after a space at idx 5. Sound (or `--group-size 0`).
- I4 new test achievable: ms1 shares ≫ 5 chars → space/5 always inserts a space → `contains(' ')` guaranteed; stdout has no "share "; stderr has labels. Sound.
- EMIT-first/INTAKE-second ordering intact (Task 3 before Task 4; no correction moved a dependency).

### Minor
- **m_new_1:** spec §5 says `--separator bogus` → "exit 2", but ms's `main.rs:169` maps ALL clap parse errors to **64**. The plan correctly uses 64. Spec text is stale; no plan action. (Spec already R0-GREEN; does not affect impl.)
- **m_new_2 (execution-accuracy clarification):** `parse.rs::strip_whitespace_handles_all_three_workflows` has NO doubling clause — all three sub-cases use non-doubled inputs and SURVIVE the body replacement unchanged. The plan's phrase "the doubling clause of `strip_whitespace_handles_all_three_workflows`" is mislabeled. **Implementer action: DELETE only `strip_whitespace_dedupes_doubled_content`; leave `strip_whitespace_handles_all_three_workflows` INTACT.**

### Verdict
**GREEN — 0 Critical / 0 Important.** All four r1 Importants resolved; full-tree sweep found no additional breaking tests; corrections internally consistent; ordering preserved. Implementation may proceed.
