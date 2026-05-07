# v0.8 Phase 2 Review — Electrum i18n + safety + UX

**Scope:** `wordlists/`, `electrum.rs`, `cmd/convert.rs`, `tests/cli_convert_electrum.rs`. 3 items: non-Latin wordlists, encode iteration bound, version info-line stderr.

**Verdict:** No critical bugs. Two Important findings, both narrow.

---

## Critical

None.

---

## Important

### I1 — Stale Portuguese word count `1654` in `electrum.rs` (confidence 95)

**File:** `crates/mnemonic-toolkit/src/electrum.rs` lines 6 (module doc), 394 (test name), 395 (inline comment).

The authoritative count in `wordlists/mod.rs` is 1626 (1654-line file − 27 copyright-header lines − 1 blank). Three occurrences of `"1654"` survived in `electrum.rs`. Doc-only bug — arithmetic is correct (`wordlist.base()` returns the actual length).

**Fix:** rewrite the three occurrences to `1626`.

### I2 — Missing CLI integration test for `--electrum-language portuguese` (confidence 82)

**File:** `crates/mnemonic-toolkit/tests/cli_convert_electrum.rs`.

Phase 2 ships per-language CLI tests for Spanish, Japanese, and Chinese-Simplified but skips Portuguese entirely on the CLI path. Portuguese is the only non-2048-base wordlist, making it the highest-value smoke test for the parameterized base-N arithmetic. Synthetic-entropy unit test exists in `electrum.rs` but doesn't exercise `parse_electrum_language_arg` or `--electrum-language portuguese`.

**Fix:** add a `--electrum-language portuguese` decode test (using the synthetic entropy from the unit test, since upstream `SEED_TEST_CASES` lacks a Portuguese vector).

---

## Verified-correct items

1. **Wordlist counts and ordering.** Exact-count asserts at init for all 5 wordlists; blob SHAs pinned in `wordlists/mod.rs` header; `cross_language_decode_rejected` guards against accidental intersection.

2. **`MAX_ENCODE_ITERATIONS = 1<<20` cap.** Checked before each loop iteration; `Err(EncodeIterationBoundExceeded)` arm exists; mapped to user-visible refusal via `map_electrum_error`. No bound-exceeded test (acceptable: practically unreachable; flagged in briefing).

3. **R2-L2 silent-ignore.** `electrum_arm_silently_ignores_language_flag` pins `--electrum-language` winning over `--language` with no stderr mention of `--language`. Silence is structural: `args.language` is never read on the `ElectrumPhrase` arm.

4. **SeedVersion info-line only on decode.** `detected_version = Some(version)` set exclusively inside the `ElectrumPhrase` match arm; all other arms return `(out, None, None)`. Two CLI tests (Standard + Segwit) pin the info-line strings.

5. **`normalize_phrase_for_hmac` vs per-word split.** Necessary and correct: HMAC dispatch needs fully-collapsed CJK whitespace; per-word lookup splits on whitespace first, so CJK-internal stripping would collapse multi-word CJK phrases. `decode_chinese_simplified_vector` (unit) + `decode_chinese_simplified_phrase_to_entropy` (CLI) cross-validate against upstream `SEED_TEST_CASES` hex.

6. **Plan deviation correctly applied.** Plan claimed 8 wordlists; upstream Electrum has 5. Phase 2 ships the 4 non-English ones (zh-Hans, ja, pt, es). zh-Hant, German, French, Italian are NOT in upstream. Implementation matches reality.

7. **`is_combining_mark` switch-over.** Initial hand-rolled range table missed Japanese U+3099 / U+309A (dakuten / handakuten) used in voiced kana like `ぶ`/`ぷ`. Switched to `unicode_normalization::char::is_combining_mark` mid-implementation; matches Python `unicodedata.combining(c) != 0` across all scripts. Cross-validated by the Japanese decode CLI test.

---

## Resolution actions applied

- **I1:** `1654` → `1626` in 3 places in `electrum.rs` (module doc + test name + inline comment).
- **I2:** added `decode_portuguese_phrase_to_entropy` CLI integration test using the synthetic round-trip pattern.
