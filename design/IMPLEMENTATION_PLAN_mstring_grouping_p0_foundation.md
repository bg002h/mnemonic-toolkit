# mstring display grouping — P0 Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the constellation-wide contract for mstring display grouping — the canonical conformance-vector file plus the reference `render_grouped` / `strip_display_separators` implementation in the toolkit — so every sibling repo (P1–P3) and the toolkit wiring (P4) implement against an identical, CI-verified spec.

**Architecture:** Three pure, ASCII-safe, dependency-free functions in a NEW dedicated lib module `crates/mnemonic-toolkit/src/display_grouping.rs`, exposed via an UNCONDITIONAL `pub mod display_grouping;` in `lib.rs`. (NOT `format.rs` — its `pub mod format` is `#[cfg(fuzzing)]`-gated in the lib, unreachable in normal builds, so `--lib` runs none of its tests; plan-R0-r1 C1/C2. A dedicated module also keeps the bin-private heavy API — `BundleJson`/`engraving_card_unified`/… — out of the public lib surface per the lib.rs:12 crate-shape policy.) A canonical tab-separated vector file at repo-root `design/display-grouping-vectors.tsv` encodes every render/strip case with keyword separators and `<…>` sentinels (no raw whitespace in any field). A new integration test drives both functions over every vector row. No CLI flags, no emit-site wiring, no release in P0 — foundation only.

**Tech Stack:** Rust (edition 2021), the toolkit's `tests/` integration-test harness, std-only (no new deps).

**Spec:** `design/SPEC_mstring_display_grouping.md` (R0 GREEN, round 3) — implements §3 (algorithm + edge cases), §4 (charset safety), §8 (vector encoding).

**Branch:** `feature/mstring-display-grouping` (already checked out).

**Plan-R0 gate:** PASSED — round 1 NOT GREEN (2C/1I/2m) → round 2 **GREEN (0C/0I)**. Reviews at `design/agent-reports/mstring-display-grouping-plan-r0-p0-round{1,2}-review.md`. Cleared for execution.

---

## File Structure

- **Create** `design/display-grouping-vectors.tsv` — canonical conformance vectors (repo root, so siblings copy it verbatim + checksum in P1–P3).
- **Create** `crates/mnemonic-toolkit/src/display_grouping.rs` — the three pure fns + their `#[cfg(test)]` unit tests. NEW dedicated lib module (NOT `format.rs`).
- **Modify** `crates/mnemonic-toolkit/src/lib.rs` — add `pub mod display_grouping;` UNCONDITIONALLY (no `#[cfg]`), with the always-on `pub mod` declarations (NOT the `#[cfg(fuzzing)]` block at lines 138-145). Do NOT touch `format.rs` / `chunk_5char` / `chunk_mk1` / `chunk_md1` (P4 deletes those and routes emit sites through `display_grouping::render_grouped`).
- **Create** `crates/mnemonic-toolkit/tests/display_grouping_conformance.rs` — reads the TSV, decodes sentinels/keywords, drives `mnemonic_toolkit::display_grouping::{render_grouped, strip_display_separators}` over every row.

### Vector-encoding convention (SPEC §8, extended for whitespace inputs)
Tab-separated, 6 columns, header row first: `op`\t`input`\t`group_size`\t`separator`\t`expected`\t`note`.
- `op` ∈ {`render`, `strip`}.
- `separator` is a KEYWORD: `space` | `hyphen` | `comma` | `none` (for `op=render group_size=0` and all `op=strip` rows).
- For `op=strip` rows, `group_size` is `0` (ignored by strip).
- Sentinels in `input`/`expected` (never a raw whitespace char, so a plain tab-split parser is unambiguous): `<empty>` = empty string; `<sp>` = U+0020; `<tab>` = U+0009; `<lf>` = U+000A; `<cr>` = U+000D. A `render` space-separated `expected` writes `<sp>` to keep the field whitespace-free.
- The driver maps keyword→char (`space`→`' '`, `hyphen`→`'-'`, `comma`→`','`) and decodes sentinels in both `input` and `expected`.

---

## Task 1: Canonical conformance-vector file

**Files:**
- Create: `design/display-grouping-vectors.tsv`

- [ ] **Step 1: Write the vector file**

Create `design/display-grouping-vectors.tsv` with EXACTLY this content (real TAB characters between columns; one header line; no trailing blank line):

```
op	input	group_size	separator	expected	note
render	abcdefghij	5	space	abcde<sp>fghij	basic space/5
render	abcdefghij	5	hyphen	abcde-fghij	basic hyphen/5
render	abcdefghij	5	comma	abcde,fghij	basic comma/5
render	abcdefghij	0	none	abcdefghij	group_size 0 = unbroken
render	abcdefghij	0	space	abcdefghij	group_size 0 ignores separator
render	abc	5	space	abc	group_size > len
render	abcde	5	space	abcde	group_size == len (no trailing sep)
render	abcdef	3	space	abc<sp>def	period 3
render	abcdefg	3	hyphen	abc-def-g	trailing partial group
render	ab	1	comma	a,b	group_size 1
render	<empty>	5	space	<empty>	empty input
render	ms1qpzry9x8gf	4	comma	ms1q,pzry,9x8g,f	codex32-shaped, comma/4
strip	abcde<sp>fghij	0	none	abcdefghij	strip space
strip	abcde-fghij	0	none	abcdefghij	strip hyphen
strip	abcde,fghij	0	none	abcdefghij	strip comma
strip	ab<sp>cd-ef,gh	0	none	abcdefgh	strip mixed separators
strip	ab--cd	0	none	abcd	consecutive separators
strip	ab<tab>cd	0	none	abcd	strip tab (whitespace)
strip	ab<cr><lf>cd	0	none	abcd	strip CRLF (whitespace)
strip	abcdefgh	0	none	abcdefgh	no separators (idempotent shape)
strip	ms1qpzry9x8	0	none	ms1qpzry9x8	codex32 chars pass through
strip	<empty>	0	none	<empty>	empty input
```

- [ ] **Step 2: Verify the file is tab-separated (not spaces) with exactly 6 fields/row**

Run: `awk -F'\t' 'NF!=6{print "BAD LINE "NR": "$0}' design/display-grouping-vectors.tsv`
Expected: no output (every row has exactly 6 tab-delimited fields). Any printed line means a tab was written as spaces — fix it.

- [ ] **Step 3: Commit**

```bash
git add design/display-grouping-vectors.tsv
git commit -m "feat(mstring-grouping): canonical display-grouping conformance vectors (P0)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 2: Reference `render_grouped` + `strip_display_separators` (new lib module)

**Files:**
- Create: `crates/mnemonic-toolkit/src/display_grouping.rs`
- Modify: `crates/mnemonic-toolkit/src/lib.rs` (one `pub mod display_grouping;` line)

- [ ] **Step 1: Create the module with STUB impls + full unit tests (red-first)**

Create `crates/mnemonic-toolkit/src/display_grouping.rs` with stub bodies so the tests compile but FAIL:

```rust
//! Canonical mstring DISPLAY-GROUPING layer (SPEC §3). Pure, ASCII-safe,
//! dependency-free. A dedicated lib module (NOT bin-private `format.rs`) so the
//! conformance test and `--lib` unit tests reach it, and so the bin-private
//! heavy API stays out of the public lib surface. P4 routes the toolkit's emit
//! sites through `render_grouped` and deletes `format.rs::chunk_*`.

pub fn is_display_separator(_c: char) -> bool {
    unimplemented!("stub")
}

pub fn render_grouped(_s: &str, _group_size: usize, _separator: char) -> String {
    unimplemented!("stub")
}

pub fn strip_display_separators(_s: &str) -> String {
    unimplemented!("stub")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_grouped_basic_space() {
        assert_eq!(render_grouped("abcdefghij", 5, ' '), "abcde fghij");
    }

    #[test]
    fn render_grouped_zero_is_unbroken() {
        assert_eq!(render_grouped("abcdefghij", 0, ' '), "abcdefghij");
        assert_eq!(render_grouped("abcdefghij", 0, '-'), "abcdefghij");
    }

    #[test]
    fn render_grouped_group_size_ge_len_unchanged() {
        assert_eq!(render_grouped("abc", 5, ' '), "abc");
        assert_eq!(render_grouped("abcde", 5, ' '), "abcde"); // no trailing sep
    }

    #[test]
    fn render_grouped_trailing_partial() {
        assert_eq!(render_grouped("abcdefg", 3, '-'), "abc-def-g");
    }

    #[test]
    fn render_grouped_empty() {
        assert_eq!(render_grouped("", 5, ' '), "");
    }

    #[test]
    fn strip_display_separators_all_kinds() {
        assert_eq!(strip_display_separators("abcde fghij"), "abcdefghij");
        assert_eq!(strip_display_separators("abcde-fghij"), "abcdefghij");
        assert_eq!(strip_display_separators("abcde,fghij"), "abcdefghij");
        assert_eq!(strip_display_separators("ab cd-ef,gh"), "abcdefgh");
    }

    #[test]
    fn strip_display_separators_whitespace_kinds() {
        assert_eq!(strip_display_separators("ab\tcd"), "abcd");
        assert_eq!(strip_display_separators("ab\r\ncd"), "abcd");
    }

    #[test]
    fn strip_display_separators_idempotent() {
        let once = strip_display_separators("ab cd-ef");
        assert_eq!(strip_display_separators(&once), once);
    }

    #[test]
    fn strip_display_separators_passes_codex32_chars() {
        assert_eq!(strip_display_separators("ms1qpzry9x8"), "ms1qpzry9x8");
    }

    #[test]
    fn render_then_strip_round_trips() {
        let s = "ms1qpzry9x8gf2tvdw";
        for gs in [0usize, 1, 4, 5, 100] {
            for sep in [' ', '-', ','] {
                assert_eq!(strip_display_separators(&render_grouped(s, gs, sep)), s);
            }
        }
    }
}
```

Then add the module declaration to `crates/mnemonic-toolkit/src/lib.rs` — insert `pub mod display_grouping;` among the UNCONDITIONAL `pub mod` lines (NOT inside the `#[cfg(fuzzing)]` block at lines 138-145). For example immediately before the first `#[cfg(fuzzing)]` declaration, or alphabetically among the existing always-on `pub mod` entries:

```rust
pub mod display_grouping;
```

- [ ] **Step 2: Run to verify the unit tests FAIL (stub panics)**

Run: `cargo test -p mnemonic-toolkit --lib display_grouping`
Expected: the module COMPILES (so `--lib` selects the tests — confirms the module is genuinely in the lib, not bin-only) and the tests FAIL with `not implemented: stub` panics. If `0 tests run`, the module is not wired into the lib — fix the `lib.rs` declaration before continuing.

- [ ] **Step 3: Replace the stub bodies with the real implementation**

In `crates/mnemonic-toolkit/src/display_grouping.rs`, replace the three stub fns with:

```rust
/// True for any character treated as a display separator on intake: ALL Unicode
/// whitespace plus `-` and `,`. SPEC §3.2. The OUTPUT separator set is the
/// subset {space, '-', ','}; every emitted grouped form therefore re-ingests.
/// None of these chars appear in the codex32 alphabet
/// (`qpzry9x8gf2tvdw0s3jn54khce6mua7l`) or the `ms`/`mk`/`md`/`1` structural
/// chars (SPEC §4), so stripping is unambiguous.
pub fn is_display_separator(c: char) -> bool {
    c.is_whitespace() || c == '-' || c == ','
}

/// Insert `separator` after every `group_size` characters (SPEC §3.1).
/// `group_size == 0` returns the input unchanged (unbroken; `separator`
/// ignored). Single line always — no newline wrapping. ASCII-safe.
pub fn render_grouped(s: &str, group_size: usize, separator: char) -> String {
    if group_size == 0 {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len() + s.len() / group_size);
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && i % group_size == 0 {
            out.push(separator);
        }
        out.push(ch);
    }
    out
}

/// Strip every display separator (SPEC §3.2) — used on intake before decode so
/// grouped and unbroken forms both re-ingest. Idempotent. Strips ONLY
/// separators; any other char (incl. codex32-alphabet chars) passes through, so
/// a malformed card is never silently "cleaned" into validity.
pub fn strip_display_separators(s: &str) -> String {
    s.chars().filter(|&c| !is_display_separator(c)).collect()
}
```

- [ ] **Step 4: Run to verify the unit tests PASS**

Run: `cargo test -p mnemonic-toolkit --lib display_grouping`
Expected: PASS — all 10 Task-2 unit tests green.

- [ ] **Step 5: Commit**

```bash
git add crates/mnemonic-toolkit/src/display_grouping.rs crates/mnemonic-toolkit/src/lib.rs
git commit -m "feat(mstring-grouping): reference render_grouped + strip_display_separators (P0)

New lib module display_grouping (NOT fuzzing-gated format.rs); pure fns per
SPEC §3. Unit tests: empty / group_size>=len / unbroken / trailing-partial /
whitespace+hyphen+comma strip / idempotence / codex32-passthrough / round-trip.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: Vector-driven conformance test

**Files:**
- Create: `crates/mnemonic-toolkit/tests/display_grouping_conformance.rs`

- [ ] **Step 1: Write the conformance test**

Create `crates/mnemonic-toolkit/tests/display_grouping_conformance.rs`:

```rust
//! Drives the canonical display-grouping conformance vectors
//! (`design/display-grouping-vectors.tsv`) through the toolkit's reference
//! `render_grouped` / `strip_display_separators`. This SAME vector file is
//! copied (verbatim, checksum-pinned) into each sibling repo in P1–P3, so all
//! four implementations are proven byte-identical. SPEC §8.

use mnemonic_toolkit::display_grouping::{render_grouped, strip_display_separators};

/// Decode the field sentinels defined by the vector-encoding convention.
fn decode(field: &str) -> String {
    if field == "<empty>" {
        return String::new();
    }
    field
        .replace("<sp>", " ")
        .replace("<tab>", "\t")
        .replace("<lf>", "\n")
        .replace("<cr>", "\r")
}

fn sep_char(keyword: &str) -> char {
    match keyword {
        "space" => ' ',
        "hyphen" => '-',
        "comma" => ',',
        "none" => ' ', // returned value ignored by render_grouped when group_size==0; never used by strip
        other => panic!("unknown separator keyword: {other}"),
    }
}

#[test]
fn conformance_vectors_pass() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../design/display-grouping-vectors.tsv"
    );
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));

    let mut lines = text.lines();
    let header = lines.next().expect("header row");
    assert_eq!(
        header, "op\tinput\tgroup_size\tseparator\texpected\tnote",
        "vector header drift"
    );

    let mut count = 0usize;
    for (i, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(cols.len(), 6, "row {} not 6 tab-fields: {line:?}", i + 2);
        let (op, input, gs, sep, expected, note) =
            (cols[0], cols[1], cols[2], cols[3], cols[4], cols[5]);
        let input = decode(input);
        let expected = decode(expected);
        let gs: usize = gs
            .parse()
            .unwrap_or_else(|_| panic!("row {}: bad group_size {gs:?}", i + 2));

        let got = match op {
            "render" => render_grouped(&input, gs, sep_char(sep)),
            "strip" => strip_display_separators(&input),
            other => panic!("row {}: unknown op {other:?}", i + 2),
        };
        assert_eq!(got, expected, "row {} ({note}): {op}({input:?}, {gs}, {sep})", i + 2);
        count += 1;
    }
    assert!(count >= 20, "expected >=20 vector rows, got {count}");
}
```

- [ ] **Step 2: Confirm `display_grouping` is reachable from an integration test**

Run: `grep -n -B1 "pub mod display_grouping" crates/mnemonic-toolkit/src/lib.rs`
Expected: the `pub mod display_grouping;` line with **NO `#[cfg(fuzzing)]` (or any `#[cfg]`) on the line immediately above it**. (This is the real reachability check — an unconditional `pub mod` is what lets `use mnemonic_toolkit::display_grouping::…` resolve in a normal `cargo test`. Contrast `format` at lib.rs:142-143, which IS `#[cfg(fuzzing)]`-gated and would NOT resolve.) If a `#[cfg]` appears above it, move the declaration out of the cfg block.

- [ ] **Step 3: Run to verify it passes**

Run: `cargo test -p mnemonic-toolkit --test display_grouping_conformance`
Expected: PASS — `conformance_vectors_pass` green, exercising every TSV row (≥20).

- [ ] **Step 4: Run the whole toolkit test suite (no regressions)**

Run: `cargo test -p mnemonic-toolkit`
Expected: PASS — all pre-existing tests still green (P0 only adds a module + a test; `format.rs`/`chunk_*` untouched).

- [ ] **Step 5: Commit**

```bash
git add crates/mnemonic-toolkit/tests/display_grouping_conformance.rs
git commit -m "test(mstring-grouping): vector-driven conformance test over canonical TSV (P0)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Self-Review (completed at write time)

**Spec coverage:** SPEC §3.1 (render) → Task 2 fn + render vectors. §3.2 (strip = whitespace+`-`+`,`) → Task 2 fn + strip vectors (incl. tab/CRLF). §3.3 edge cases (empty, gs≥len, gs0+sep, consecutive seps, idempotence, ASCII) → vectors + unit tests. §4 charset safety → `is_display_separator` doc + the codex32-passthrough vector. §8 vector-encoding convention → Task 1 file + Task 3 decoder. (Flags, emit/intake wiring, releases, sibling copies, manuals = P1–P5, out of P0 scope by design.)

**Placeholder scan:** none — all code/paths/commands concrete.

**Type consistency:** `render_grouped(&str, usize, char) -> String`, `strip_display_separators(&str) -> String`, `is_display_separator(char) -> bool` used identically in stub, real impl, unit tests, and the conformance driver. Separator keyword set {space,hyphen,comma,none} consistent between the TSV (Task 1) and `sep_char` (Task 3).

**Crate-layout (plan-R0-r1 C1/C2 resolved):** the fns live in a NEW UNCONDITIONAL lib module `display_grouping` (NOT the `#[cfg(fuzzing)]`-gated `format.rs`). Therefore `cargo test --lib display_grouping` selects the unit tests (not zero), and the integration test's `use mnemonic_toolkit::display_grouping::…` resolves in normal builds. Task 2 uses a stub→red→impl→green TDD cycle; Task 3 Step 2 verifies the `pub mod` is genuinely uncgated.
