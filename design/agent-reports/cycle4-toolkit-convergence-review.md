# Cycle-4 toolkit convergence review — independent adversarial execution review

- **Scope:** toolkit convergence PATCH 0.62.0 → 0.62.1 (pins md-codec 0.38.0 + ms-codec 0.5.0; downstream lockstep exit-code/prose arms + 2 characterization tests).
- **Worktree HEAD:** `e3e7255d` (branch `chore/cycle4-toolkit-pin-bump`, one commit)
- **Base:** `c578e123` (`origin/master`)
- **Date:** 2026-06-21
- **Reviewer:** opus software architect (mandatory independent adversarial execution review)
- **Funds-safety property at stake:** fail-closed on out-of-domain md1 input; correct rejection of inconsistent Shamir share sets.

---

## Method

Every claim was verified empirically, not from the doc text:

1. Read the full diff (`git diff origin/master..HEAD`): Cargo.toml pins, both locks, `error.rs`, `friendly.rs`, both new tests, both READMEs, `install.sh`, `CHANGELOG.md`.
2. Confirmed the pre-existing `bad()`→`BadInput` wrapping at `cmd/restore.rs:2751` is **byte-identical on `origin/master`** (`git show origin/master:.../restore.rs`).
3. Confirmed both `md_codec_exit_code` and `friendly_md_codec` are **exhaustive matches with no `_ =>` wildcard for `md_codec::Error`** (so the 3 new md arms were genuinely compile-forced).
4. **Built the HEAD binary** and ran `restore --md1`, `inspect`, `repair --md1`, and `ms-shares combine` against live over-93 / inconsistent-set fixtures.
5. **Built a base binary against the OLD codecs (md 0.37.0 + ms 0.4.4)** and re-ran the identical fixtures — the definitive non-vacuity / mutation check.
6. Ran the two new test files (9 tests), the full workspace suite (`cargo test`), and `cargo clippy --all-targets -- -D warnings`.
7. Verified lock surgery (only 3 packages changed; codex32 stays =0.1.0), all 5 release version sites, and that the fuzz plain-build break is identical on base.

---

## PRIMARY focus 1 — `restore --md1 <over-93>` fail-closed (the funds-critical property)

**VERDICT: FAIL-CLOSED CONFIRMED. No funds hole.**

Empirical (HEAD binary, 94-symbol clean md1):

```
$ mnemonic restore --md1 "md1qqq…(94 q's)"
error: --md1 decode: string has 94 symbols; the codex32 regular code caps a string at 93
EXIT=1   STDOUT length = 0
```

- The codec **rejects** (returns `Err`); the toolkit propagates via `?` at `cmd/restore.rs:2751`, so the function returns early — **no descriptor / wallet is ever emitted**. STDOUT is empty (verified, len=0).
- Exit is **1 (BadInput)**, not 2, because the pre-existing `.map_err(|e| bad(format!("--md1 decode: {e}")))` wrapper at line 2751 down-classifies every md1-reassemble failure to `BadInput`. This line is **byte-identical on `origin/master`** (verified via `git show`) — the exit-1 behavior is pre-existing, NOT introduced by this PATCH.
- I built the base binary (md-codec 0.37.0) and ran the same input: **exit 1, STDOUT len 0** — confirming the fail-closed-at-exit-1 contract predates cycle-4. The ONLY change 0.38.0 brings is the error *message* (`caps a string at 93` vs the old `BCH checksum verification failed`); the reject + zero-output behavior is unchanged.
- Exit 1 vs 2 here is purely cosmetic. The funds-critical distinction — SILENT-ACCEPT vs REJECT — lands firmly on REJECT. The original I1 bug (silently decoding an out-of-domain descriptor) is **closed**: the codec now refuses the over-93 word before any descriptor is produced.
- The exit-2 surface of the I1 cap is asserted honestly via `inspect` (which routes through `md_codec_exit_code`): `inspect <over-93>` → exit 2 with `caps a string at 93` prose (verified). The `restore` exit-1 test (`restore_over_93_symbol_md1_exits_1_pre_existing_bad_input`) asserts the **actual current behavior** (code 1 + the cap message rendering) — it is a true characterization test, not vacuous; it passed on HEAD and asserts code(1) which is the real observed value.

This drift finding is **sound and not a funds hole.**

## PRIMARY focus 2 — `repair --md1 <over-93>` → exit 2 via toolkit's own band classifier

**VERDICT: SOUND. Repair fails closed; codec M4 cap is defense-in-depth, not relied upon here.**

Empirical (HEAD):

```
$ mnemonic repair --md1 "md1qqq…(94)"  → EXIT 2  "…reserved-invalid band [94, 95]…"
$ mnemonic repair --md1 "md1qqq…(100)" → EXIT 2  "…long BCH code…"
```

- The reject originates in the toolkit's OWN `repair.rs` length-band classifier (BIP-93 [94,95] reserved-invalid / [96,108] long-code-undefined), which sits *in front of* the codec's `decode_with_correction` path. So `repair` never reaches the codec M4 `ChunkSymbolCountOutOfRange` cap for these inputs.
- Is the codec M4 cap therefore dead code for repair? It is **not reached** on the repair path for over-93 input (the toolkit band-rejects first), but it remains a legitimate **defense-in-depth floor** behind `decode_with_correction` for any caller that bypasses the band classifier (e.g. the chunk-correction path on a malformed multi-chunk set). Not a defect; both tests (`repair_md1_reserved_invalid_band_exits_2`, `repair_md1_long_code_band_exits_2`) assert a non-zero exit-2 fail-closed outcome and passed. This is a **Minor note**, not a finding.

---

## Standard verification

### Lockstep arm correctness — PASS

- **md side (compile-forced, both sites exhaustive):**
  - `md_codec_exit_code` (`error.rs:531-533`): `PayloadTooLongForSingleString | ChunkSymbolCountOutOfRange | StringSymbolCountOutOfRange => 2`, grouped with `TooManyErrors`. No `_ =>` wildcard for `md_codec::Error`; the match ends `WireVersionMismatch { .. } => 3`. Exhaustive — confirmed by grep (no wildcard) and by the fact the build only succeeds with all three arms present.
  - `friendly_md_codec` (`friendly.rs:380,388,398`): all three variants have prose arms; the match has no wildcard (ends with `StringSymbolCountOutOfRange`). Prose is accurate and sane (renders symbol counts + the 93/80 caps).
  - All three route to **exit 2** (decode/format-reject class). None routes to exit 1 or success.
- **ms side (silent — `#[non_exhaustive]` + `_ => 1` wildcard, so arms are explicit by design):**
  - `ms_codec_exit_code` (`error.rs:425`): `InconsistentShareSet => 2`, placed in the exit-2 group **before** the terminal `_ => 1`. Correct — without it the wildcard would mis-route this funds error to exit 1. Verified empirically: combine of an inconsistent set exits 2.
  - `friendly_ms_codec` (`friendly.rs:142`): explicit prose arm ("ms1 inconsistent share set: … Combining them would recover the WRONG secret — supply only shares from a single split"). Clear and funds-aware.
- No arm routes a funds/format error to exit 1 or "success." Confirmed.

### Test non-vacuity (mutation check against OLD codecs) — PASS, with one prose-accuracy Minor

Built a base binary (md-codec 0.37.0 + ms-codec 0.4.4) and ran the fixtures:

| Fixture | OLD codec | NEW codec | Test assertion | Vacuous on old? |
|---|---|---|---|---|
| `inspect <over-93>` | exit **1**, "BCH checksum verification failed" | exit **2**, "caps a string at 93" | `.code(2)` + "caps a string at 93" | **NO** — fails on old (wrong exit + wrong message) |
| `ms-shares combine [A1,A2,B3]` | exit **2**, "reserved-prefix byte was 0x12" | exit **2**, "inconsistent share set" | `exit==2` **AND** `stderr.contains("inconsistent share set")` | **NO** — the prose assertion fails on old (old says "reserved-prefix byte", not "inconsistent share set") |
| `ms-shares combine [A1,A2]` (valid) | recovers A, exit 0 | recovers A, exit 0 | recovers A, exit 0 | positive control (same both) — correct |

- The ms test IS non-vacuous: although the `exit==2` half coincides between old and new, the **prose half (`contains("inconsistent share set")`) distinguishes them** — the old codec rejects via a *different, incidental* check and never emits that string. A run against the old codec FAILS the assertion. Mutation-check satisfied.
- The fixture is a genuinely INCONSISTENT same-id n>k set: A1/A2/B3 all carry id "aaaa"/threshold 2/length, B3 is off A's polynomial (secret B = 0x33×16 vs A = 0x11×16). The positive controls (exactly-k `[A1,A2]` and over-threshold all-consistent `[A1,A2,A3]`) both recover A at exit 0 (verified empirically).
- **Minor (prose-accuracy, not a funds issue):** The test's doc comment and the CHANGELOG state this *specific* fixture "previously combined to a SILENT WRONG secret … at exit 0" pre-0.5.0. Empirically that is **false for this exact fixture**: the OLD ms-codec 0.4.4 rejected `[A1,A2,B3]` at **exit 2** with `ms1 reserved-prefix byte was 0x12, expected 0x00` — i.e. interpolating B3 against A's polynomial corrupted the recovered secret-share's reserved header byte, which an *incidental* pre-existing validation caught. So for THIS fixture the old behavior was reject-by-accident, not silent-wrong-secret-leak. The M6 *general* property (the silent-wrong-secret class exists for inconsistent sets whose corrupted recovery happens to pass the reserved-byte check) is real and is what 0.5.0's membership check closes principledly; but the narrative attached to this particular fixture overstates the pre-fix leak. This does NOT weaken the test (it still fails on old via the prose assertion and asserts the correct new behavior + no-leak), and does NOT affect funds-safety (the new codec rejects principledly; the no-stdout-leak assertion passed). It is a documentation/provenance imprecision only. **No required code change.** Optional: soften the doc/CHANGELOG wording to "would not be reliably caught pre-0.5.0 (this fixture was rejected only incidentally by the reserved-byte check; other inconsistent sets leaked silently)."

### Pin correctness — PASS

- `Cargo.lock` + `fuzz/Cargo.lock`: only `md-codec` 0.37.0→0.38.0 (new checksum `e131406…`), `ms-codec` 0.4.4→0.5.0 (new checksum `fde9b45…`), and `mnemonic-toolkit` 0.62.0→0.62.1 changed. **No other package versions moved.**
- `codex32` stays `=0.1.0` in both locks (verified).
- Cargo.toml caret pins hand-edited to `ms-codec = "0.5"`, `md-codec = "0.38"` (cannot be reached by `cargo update` across the `^0.37`/`^0.4.4` bounds) — correct.

### Release-ritual completeness — PASS

All 5 version sites bumped to 0.62.1: `crates/mnemonic-toolkit/Cargo.toml`, top-level `README.md` (`toolkit-version` marker), `crates/mnemonic-toolkit/README.md` (marker), `scripts/install.sh` (self-pin tag `mnemonic-toolkit-v0.62.1`), and a complete `CHANGELOG.md` 0.62.1 entry. `fuzz/Cargo.toml` is version `0.0.0` (path-dep workspace — no version site; its lock is updated). Lingering `0.62.0` strings are all historical design-doc / FOLLOWUPS / agent-report references to the prior cycle-2 release (correct). No version site missed.

### No regression — PASS

- **`cargo test` (workspace): 3306 passed, 0 failed.** Matches the implementer's claim exactly.
- The two new test files: 6/0 (md cap) + 3/0 (ms inconsistent) = 9 passed.
- **`cargo clippy --all-targets -- -D warnings`: clean** (finished, no warnings).
- The fuzz plain-build break (`could not find parse_descriptor` — the `cfg(fuzzing)`-gated module referenced by a plain fuzz build) is **byte-for-byte identical on base** (verified by building base fuzz) → pre-existing, out of scope, not introduced here. Not a blocker per the review brief.

---

## Findings

### Critical
None.

### Important
None.

### Minor
1. **ms inconsistent-set fixture narrative overstates the pre-fix leak** (`tests/cli_ms_shares_inconsistent.rs` header lines 5-11, 58-60; `CHANGELOG.md` 0.62.1 "Funds-safety notes"). The specific `[A1,A2,B3]` fixture was rejected at exit 2 by the OLD codec via an incidental `reserved-prefix byte was 0x12` check — NOT a silent-wrong-secret-at-exit-0. The general M6 silent-leak class is real and is what 0.5.0 closes principledly, but this fixture is not an instance of the silent leak. The test remains non-vacuous (the "inconsistent share set" prose assertion fails on old) and funds-correct. **No code change required;** optional doc/CHANGELOG wording softening for provenance accuracy. (Documentation/provenance only — does not block the tag.)
2. **Codec M4 `ChunkSymbolCountOutOfRange` cap is not reached on the `repair --md1` path** for over-93 input (the toolkit's `repair.rs` band classifier rejects first). It remains a legitimate defense-in-depth floor behind `decode_with_correction` for other callers. Note only; not a defect — repair fails closed at exit 2 either way.

---

## VERDICT

**CONVERGENCE: 0C / 0I**

**GREEN (0C / 0I) — cleared to tag.**

Two Minor items (one documentation-provenance imprecision in the ms fixture narrative, one informational note on the repair/M4 path), neither funds-affecting nor tag-blocking.

### Explicit answers to the brief's three questions

- **(a) Does `restore --md1` over-93 FAIL CLOSED (reject, no wrong wallet)?** **YES.** Empirically: exit non-zero (1), STDOUT length 0, no descriptor/wallet emitted, codec rejects the over-93 word before any output. Exit 1 (vs 2) is the pre-existing `bad()`→`BadInput` wrapper at `restore.rs:2751`, byte-identical on `origin/master` — cosmetic, not a funds hole.
- **(b) Are both md exhaustive sites + the ms arms complete and correctly exit-2?** **YES.** All three md variants have arms in BOTH `md_codec_exit_code` (`error.rs:531-533`, exit 2) and `friendly_md_codec` (`friendly.rs:380/388/398`, sane prose); both matches are wildcard-free exhaustive and compile-complete (build forced them). The ms `InconsistentShareSet => 2` arm sits before the `_ => 1` wildcard in `ms_codec_exit_code` (`error.rs:425`) with a matching `friendly_ms_codec` prose arm (`friendly.rs:142`). No funds/format error routes to exit 1 or success.
- **(c) Is the ms inconsistent-set test non-vacuous (would fail against old codecs)?** **YES.** Run against md-codec 0.37.0 / ms-codec 0.4.4, the test FAILS: its required `stderr.contains("inconsistent share set")` assertion does not hold (the old codec emits "reserved-prefix byte was 0x12" instead). The fixture is a genuine same-id/different-secret inconsistent set with a passing positive control. (Caveat — Minor 1: the *narrative* that this specific fixture silently leaked a wrong secret pre-0.5.0 is inaccurate; the old codec rejected it incidentally. The test itself is non-vacuous and funds-correct regardless.)
