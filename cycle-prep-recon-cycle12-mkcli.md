# cycle-prep recon — 2026-06-21 — cycle-12 mk-cli cluster (M12, L20)

**Repo:** `mnemonic-key` (`/scratch/code/shibboleth/mnemonic-key`)
**Origin/main SHA at recon time:** `3258271` (`32582718245735a26fb36787c5b56edcfa06d972`)
**Local branch:** `main`
**Sync state:** up-to-date with `origin/main` (last-known `3258271` confirmed unchanged)
**Untracked:** none relevant to mk-cli/mk-codec source

Findings source: `mnemonic-toolkit/design/agent-reports/constellation-bughunt-2026-06-20.md` (M12 §756-766, L20 §787-792).
Drift expectation: **citations use `mk-cli/src/...` paths but the workspace nests them under `crates/mk-cli/src/...`** — a uniform prefix-DRIFT, content otherwise ACCURATE. Both findings STILL-REPRODUCE.

All source verified against `origin/main` BYTES via `git show origin/main:<path>`.

---

## Per-finding verification

### M12 · `mk repair` emits an INVALID mixed-case mk1 string for all-uppercase input
- **WHAT (from bug-hunt report):** `mk repair`'s `reconstruct_corrected` splices the original-cased HRP prefix with lowercase data symbols from `ALPHABET`; input is never case-normalized. Uppercase card (`MK1…`, the canonical QR-friendly form) yields a **mixed-case** output (`MK1qpslfap…`) that is invalid per bech32/codex32 and fails re-decode with "mixed case" — defeating repair's entire purpose (a usable, re-ingestable string). Fires even at exit 0 (no correction needed).
- **Class:** mk-cli · other (broken artifact). Funds-relevance: NOT funds-loss; produces an un-ingestable recovery artifact (loud-fail on next decode, not silent).
- **Citations:**
  - `mk-cli/src/cmd/repair.rs:97-147` (`reconstruct_corrected`) — **DRIFTED (path prefix)**: actual file is `crates/mk-cli/src/cmd/repair.rs`; `fn reconstruct_corrected` is at **line 97**, body runs ~97-147. ACCURATE content.
  - `cmd/mod.rs:97` (no case-normalize) — **ACCURATE (path-prefixed)**: `crates/mk-cli/src/cmd/mod.rs:97` is `pub fn read_mk1_strings`, the input-read path; it calls `strip_display_separators` which (verified, `crates/mk-cli/src/format.rs:33`) only filters whitespace/`-`/`,` — **does NOT lowercase**. Confirms "input is never case-normalized."
  - "splices original-cased prefix with lowercase `ALPHABET`" — **ACCURATE**: `repair.rs:104` `let (prefix, rest) = original.split_at(sep_pos)`; `:132` `out.push_str(prefix)` (original case preserved); `:135` `out.push(ALPHABET[v as usize] as char)` where `ALPHABET` = `crates/mk-codec/src/string_layer/bch.rs:39` = `b"qpzry9x8gf2tvdw0s3jn54khce6mua7l"` (**lowercase**). Mixed-case output for uppercase input is structurally guaranteed.
  - "decode accepts all-uppercase" — **ACCURATE (verified vs codec)**: `mk-codec/src/string_layer/bch.rs:658` `decode_string` rejects only `CaseStatus::Mixed` (`:661-662 → Error::MixedCase`), then `:664 s.to_lowercase()` for internal decode. All-uppercase = `CaseStatus::Upper` → accepted. So `MK1…` decodes fine but `reconstruct_corrected` re-emits mixed case.
- **STILL-REPRODUCES: YES.** Mechanism fully present and structurally forced for any all-uppercase input (even clean, exit-0 path). Output `MK1<lowercase-data>` → feed back to `mk decode`/`mk repair` → `Error::MixedCase` (exit 2).
- **Fix-site (mk-cli ONLY):** `crates/mk-cli/src/cmd/repair.rs::reconstruct_corrected` — normalize the emitted string's case to match input case (lowercase the prefix; or, to preserve uppercase QR canonical form, uppercase the data symbols when `case_check(original) == Upper`). The cleanest funds-safe default per BIP-173/93 (entirely-lower XOR entirely-upper) is to emit all-lowercase always (the codec's canonical internal form) OR mirror input case. Add a re-decode round-trip test (existing `crates/mk-cli/tests/cli_repair.rs` has NO uppercase coverage — `repair_already_valid_input_exits_0`, `repair_one_substitution_exits_5`, `repair_long_code_happy_path` are all lowercase fixtures).
- **mk-codec needed?** NO. The codec is correct: it rejects mixed-case and accepts all-upper; `case_check`/`to_lowercase` already exist. M12 is a pure mk-cli re-emit bug.

### L20 · `classify_code_variant` off-by-one mislabels a 96-symbol long-code chunk as "regular"
- **WHAT:** `classify_code_variant` uses `s.len() <= 96 + "mk1".len()` (≤99) → "regular". A long-code minimum data-part of 96 symbols → total string length 99 → mislabeled "regular". Display/JSON only. Fix: threshold `≤ 93 + "mk1".len()` (≤96), or classify via `bch_code_for_length`.
- **Class:** mk-cli · other (display). NOT funds-affecting (cosmetic label in decode/encode/inspect output).
- **Citations:**
  - `mk-cli/src/cmd/mod.rs:131-140` — **ACCURATE (path-prefixed)**: `crates/mk-cli/src/cmd/mod.rs:131` `pub fn classify_code_variant`; `:135` `if s.len() <= 96 + "mk1".len()` → `:136 "regular"` else `:138 "long"`. Exactly as cited.
  - authoritative `mk-codec/src/string_layer/bch.rs:117-124` (`bch_code_for_length`) — **ACCURATE (path-prefixed)**: `crates/mk-codec/src/string_layer/bch.rs:112-117` define `bch_code_for_length`: `14..=93 → Regular`, `94..=95 → None` (reserved-invalid), `96..=108 → Long`. This is the ground truth.
  - "96-symbol long chunk gives total 99, mislabeled" — **ACCURATE (verified by arithmetic)**: HRP+sep `mk1` = 3 chars; data-part 96 → total 99; `99 <= 99` is true → "regular", but 96-symbol data-part = `BchCode::Long`. Off-by-one confirmed.
  - "spec: BIP-93 regular 14..=93, long 96..=108" — **ACCURATE (verified vs primary source)**: matches `bch_code_for_length` and the module doc (`bch.rs:24-25`: "regular for ≤93 chars, long for 96–108 chars; 94–95 reserved-invalid") + BchCode docstrings (Regular = BCH(93,80,8); Long = BCH(108,93,8)).
  - Proposed fix `≤ 93 + "mk1".len()` (≤96) — **CORRECT**: data-part ≤93 → total ≤96 → "regular"; ≥96 data-part → ≥99 total → "long". (Note: a 94–95-symbol data-part is reserved-invalid and never reaches `classify` post-decode, so the two-way threshold is safe; classifying via `bch_code_for_length` would be more robust/self-documenting but requires the data-part length, not the full-string length.)
- **STILL-REPRODUCES: YES.** Pure arithmetic off-by-one, structurally present. Surfaces in **3 subcommands** (more than the report's repair focus): `crates/mk-cli/src/cmd/decode.rs:30`, `encode.rs:119`, `inspect.rs:33` all map `classify_code_variant` over inputs for their `code_variant`/`chunk_variants` output fields.
- **Fix-site (mk-cli ONLY):** `crates/mk-cli/src/cmd/mod.rs:135` — change threshold to `93 + "mk1".len()`. Also fix the stale doc-comment at `:127-130` (claims "regular = 1+3+93 = 97 chars" and "90 (regular)… 108 (long)" — internally inconsistent and pre-dates the corrected boundary). Add a 96-symbol-data-part fixture test (existing `repair_long_code_happy_path` in `cli_repair.rs:209` already builds a long chunk and references `valid_long.len() - "mk1".len()`; mirror that for a `classify` unit/CLI test).
- **mk-codec needed?** NO. `bch_code_for_length` is already correct; this is a mk-cli display helper duplicating the boundary wrong.

---

## Cross-cutting observations
1. **Uniform path-prefix DRIFT, not structural error.** Every cited path drops the `crates/` workspace prefix (`mk-cli/src/...` should be `crates/mk-cli/src/...`, `mk-codec/src/...` → `crates/mk-codec/src/...`). Line numbers and symbols are otherwise ACCURATE/exact. Brainstorm spec must use the `crates/`-prefixed paths and cite SHA `3258271`.
2. **Both fixes are mk-cli-only.** mk-codec is the authoritative source for both boundaries and is **correct** (`bch_code_for_length`, `case_check`, mixed-case rejection, `to_lowercase`). **mk-codec NO-BUMP.** No mk-codec change, no new mk-codec API needed.
3. **L20 blast radius is wider than the report frames it.** Report cites only the repair-verify context, but `classify_code_variant` feeds `decode`, `encode`, AND `inspect` output (`code_variant` / `chunk_variants` JSON+text fields). The single-line fix corrects all three at once; tests should cover at least one of them.
4. **Toolkit is NOT in the publish chain.** `mnemonic-toolkit/crates/mnemonic-toolkit/Cargo.toml:35` deps `mk-codec = "0.4.0"` (library) — NOT mk-cli. Since neither fix touches mk-codec, **no toolkit pin bump, no toolkit change of any kind.** Confirmed.
5. **GUI schema_mirror: N/A.** That gate covers the toolkit's `mnemonic` clap surface only, and only fires on toolkit binary pins. mk-cli has no GUI schema mirror; both fixes are output-VALUE/behavior changes with **zero clap flag/subcommand/dropdown changes**. No GUI lockstep.
6. **Manual mirror: optional, not forced.** `docs/manual/src/40-cli-reference/44-mk-cli.md` documents the `regular|long` value set and example JSON (lines 68/72/137/153/191/196) but with regular-fixture examples and no flag/schema change — L20's fix makes the label correct without altering the documented schema. No mandatory manual update; an optional NOTE on the corrected 96-symbol boundary is polish. M12 changes no documented surface.
7. **Current versions:** mk-cli `0.10.0`, mk-codec `0.4.0`. mk-cli path-deps mk-codec (`version = "0.4.0"`).

---

## Recommended brainstorm-session scope
**Single combined cycle (cycle-12), mk-cli-only.** M12 + L20 are independent, both small, same crate, no shared state, no inter-finding dependency — fold into one brainstorm/SPEC/plan with two phases (or one phase, two fixes) under the standard R0 gate.

- **Rough LOC:** ~10-20 LOC production + ~40-60 LOC tests.
  - M12: 1 case-normalize branch in `reconstruct_corrected` (~5-8 LOC) + uppercase round-trip test (~20-30 LOC).
  - L20: 1-line threshold change at `mod.rs:135` + stale doc-comment fix at `:127-130` (~3 LOC) + 96-symbol-boundary classify test (~15-25 LOC).
- **SemVer (mk-cli):** **PATCH → `0.10.1`.** Both are bug fixes to existing behavior; no new flags/subcommands/values, no breaking change. (M12 changes the emitted string's case for uppercase input — a bug-fix toward a valid artifact, not a contract break. L20 corrects a mislabel — output-value fix. Neither adds public API.)
- **mk-codec:** **NO-BUMP.** Authoritative source already correct; no change.
- **Publish chain:** mk-cli → tag + publish `0.10.1` to crates.io. **No toolkit pin, no toolkit tag, no GUI change.** Toolkit deps mk-codec lib (unchanged) only.
- **Locksteps:** none mandatory. (No GUI schema_mirror; no forced manual update — optional polish only.)
- **Ordering:** L20 first (trivial 1-line + doc), then M12 (slightly more involved re-emit logic + round-trip test). Either order is fine; no dependency.
- **Test gap to close:** `crates/mk-cli/tests/cli_repair.rs` has zero uppercase coverage → M12 needs an all-uppercase repair round-trip (`MK1…` in → re-`mk decode` the output must succeed). L20 needs a 96-symbol-data-part `classify_code_variant` assertion + ideally a `decode`/`inspect` CLI-level `code_variant: "long"` check.

**Next gate:** R0 architect review on the brainstorm spec + plan-doc → converge 0C/0I BEFORE any code. (Recon only — no implementation here.)
