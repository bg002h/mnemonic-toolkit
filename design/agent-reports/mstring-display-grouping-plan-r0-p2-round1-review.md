# Plan-R0 (P2 ms) round 1 — architect review (verbatim)

> Persisted verbatim per CLAUDE.md BEFORE the fold. Dispatched via Agent
> (feature-dev:code-architect, Opus 4.8). **Verdict: NOT GREEN — 0 Critical /
> 4 Important** (all missing breaking tests). Plan SHA at review: toolkit `000b6d4`;
> ms repo `b616530`. Includes the agent's analysis prefix + structured review.

---

Good - ms-codec has no display-separator stripping. The plan's claim is confirmed: ms-codec's decode does NOT tolerate display separators (space/hyphen/comma), so intake stripping at the ms-cli layer is genuinely required. ✓

Now I have the complete picture. Let me write the final review:

---

## Plan-R0 Architect Review — P2 ms-cli Implementation Plan

**Verdict: NOT GREEN — 0 Critical / 4 Important**

[The reviewer confirmed: all cited file/line numbers (format.rs chunked :14-32, parse.rs strip_whitespace :97-110 + tests :122/:138, encode.rs emit_text :198-201, split.rs emit_text :147-164, combine.rs, main.rs clap→64 :165-170); the EMIT-first/INTAKE-second ordering (removing the dedup heuristic while encode prints twice WOULD break encode_pipe_to_decode.rs — CONFIRMED); the architecture deviation (ms-cli bin-only; #[cfg(test)] mod in format.rs reaches the pub fns; path math resolves to repo-root design/; ms-codec needs NO change); the intake-coverage claim (ms-codec decode does NOT tolerate separators → comma/hyphen genuinely net-new); the spec-§13 correction (encode_canonical_24_word DOES break on starts_with); scope (combine intake-only; split labels→stderr; --separator bogus → exit 64). The four Importants below are breaking tests the plan's enumeration MISSED.]

---

### Critical
None.

---

### Important

**I1: `encode_no_engraving_card.rs::encode_no_engraving_card_suppresses_engraving_block` is a missing breaking test.**
File: `crates/ms-cli/tests/encode_no_engraving_card.rs:19` asserts `.stdout(starts_with("ms10entrsqqqq"))`. Default space/5 grouping makes stdout `"ms10e ntrsq..."` (space at index 5) → `starts_with` fails at position 5. Fix: add to Task 3's fix list; either assert `starts_with("ms10e ")` + a stripped-equals check, or drop `starts_with` for a contains-canonical-after-strip check (as with `encode_canonical_12_word`).

**I2: `encode_hex_input.rs::encode_hex_zeros_16_bytes` is a missing breaking test.**
File: `crates/ms-cli/tests/encode_hex_input.rs:13` asserts `.stdout(starts_with("ms10entrsqqqq"))`. Zero-hex encodes to the same ms1 as all-zeros 12-word; grouped → `"ms10e ntrsq..."` → fails at char 5. Fix: add to Task 3's fix list; repair analogously. (`encode_hex_omits_language_in_engraving_card` in the same file checks only stderr — unaffected.)

**I3: Four tests in `encode_mnem_japanese.rs` are missing breaking tests.**
- `:30` `encode_japanese_phrase_produces_mnem_ms1_of_expected_length` — asserts `first_line.len() == MNEM_12_WORD_LEN` (51). Grouped 51-char ms1 = 51 + 10 spaces = 61 ≠ 51 → FAILS.
- `:74` `encode_english_phrase_stays_entr_payload_length` — `first_line.len() == 50`; grouped = 59 → FAILS.
- `:89` `encode_hex_stays_entr_payload_length` — `first_line.len() == 50`; grouped = 59 → FAILS.
- `encode_japanese_phrase_decode_round_trip` — extracts `lines().next()` then asserts `ms1.len() == MNEM_12_WORD_LEN`; grouped first line = 61 → FAILS.
Fix: add `encode_mnem_japanese.rs` to Task 3's fix list. For pure-length tests use `--group-size 0` (length unchanged); or strip separators from `first_line` before the length check.

**I4: `cli_split.rs` — the `--group-size 0` fix for `split_english_phrase_emits_n_shares_text` is correct, but the NEW grouped/labels-on-stderr test is unspecified.**
File: `crates/ms-cli/tests/cli_split.rs:44` filter `!l.contains(' ')` breaks under grouping; the plan's `--group-size 0` fix is right (the 3 unbroken share lines ARE the entire stdout post-restructure). But the plan says "add a NEW test" without naming it or giving assertions. Fix: in Task 3, add a concrete `split_grouped_default_labels_on_stderr` test: stdout = exactly 3 lines, each `starts_with("ms1")` AND contains a space; stdout contains NO "share N of M"; stderr contains "share 1 of 3".

---

### Minor

**m1:** Task 3 `split.rs` `emit_text` restructure doesn't specify stderr-labels-vs-stdout-shares ordering. OS-buffered, so no correctness impact for `ms split | ms combine -`, but pick one (simplest: all stdout shares, then all stderr labels). Document the choice.

**m2:** Task 2 conformance `assert!(n >= 20)` is a loose floor (P0 TSV has 22 rows). Consider `assert_eq!(n, EXPECTED)` to catch silent row deletion. Low priority (matches P1's floor).

**m3:** Task 4 `parse.rs::strip_whitespace` doc update is mentioned but replacement text unspecified — confirm the new doc references `format::strip_display_separators` and drops the §3.2 doubling rationale.

**m4:** Task 7 publish: confirm ms-cli is a crates.io crate via dry-run + prior `ms-cli-v*` tags before `cargo publish`. Process note, not a finding.

---

### Summary of Required Actions Before Implementation
1. (I1) Add `encode_no_engraving_card.rs` to Task 3's fix list.
2. (I2) Add `encode_hex_input.rs` to Task 3's fix list.
3. (I3) Add all four `encode_mnem_japanese.rs` tests to Task 3's fix list (`--group-size 0` or strip-then-check).
4. (I4) Add a concrete named `split_grouped_default_labels_on_stderr` test spec to Task 3.
