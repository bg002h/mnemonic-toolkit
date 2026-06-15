# mstring display grouping — P0 Foundation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Establish the constellation-wide contract for mstring display grouping — the canonical conformance-vector file plus the reference `render_grouped` / `strip_display_separators` implementation in the toolkit — so every sibling repo (P1–P3) and the toolkit wiring (P4) implement against an identical, CI-verified spec.

**Architecture:** Two pure, ASCII-safe, dependency-free functions added to the toolkit's `crates/mnemonic-toolkit/src/format.rs` (alongside the existing `chunk_*` fns, which stay until P4). A canonical tab-separated vector file at repo-root `design/display-grouping-vectors.tsv` encodes every render/strip case with keyword separators and `<…>` sentinels (no raw whitespace in any field). A new integration test drives both functions over every vector row. No CLI flags, no emit-site wiring, no release in P0 — this is the foundation only.

**Tech Stack:** Rust (edition 2021), the toolkit's existing `tests/` integration-test harness, std-only (no new deps).

**Spec:** `design/SPEC_mstring_display_grouping.md` (R0 GREEN, round 3). This plan implements SPEC §3 (algorithm + edge cases), §4 (charset safety), and the §8 vector-encoding convention.

**Branch:** `feature/mstring-display-grouping` (already checked out).

**Plan-R0 gate:** This plan MUST pass an opus architect plan-R0 to 0 Critical / 0 Important before any task is executed (CLAUDE.md §1).

---

## File Structure

- **Create** `design/display-grouping-vectors.tsv` — canonical conformance vectors (repo root, so siblings copy it verbatim + checksum in P1–P3).
- **Modify** `crates/mnemonic-toolkit/src/format.rs` — add `is_display_separator`, `render_grouped`, `strip_display_separators` (pure fns) near the existing `chunk_5char` (line 10). Do NOT touch `chunk_5char`/`chunk_mk1`/`chunk_md1` (P4 deletes them).
- **Create** `crates/mnemonic-toolkit/tests/display_grouping_conformance.rs` — reads the TSV, decodes sentinels/keywords, drives `render_grouped`/`strip_display_separators` over every row.

### Vector-encoding convention (SPEC §8, extended for whitespace inputs)
Tab-separated, 6 columns, header row first: `op`\t`input`\t`group_size`\t`separator`\t`expected`\t`note`.
- `op` ∈ {`render`, `strip`}.
- `separator` is a KEYWORD: `space` | `hyphen` | `comma` | `none` (for `op=render group_size=0` and all `op=strip` rows).
- For `op=strip` rows, `group_size` is `0` (ignored by strip).
- Sentinels in `input`/`expected` (never a raw whitespace char, so a plain tab-split parser is unambiguous): `<empty>` = empty string; `<sp>` = U+0020 space; `<tab>` = U+0009; `<lf>` = U+000A; `<cr>` = U+000D. (Keyword separators in OUTPUT of `render` are emitted as the literal char in `expected` — e.g. a space row's `expected` uses `<sp>` to keep the field whitespace-free.)
- The test driver maps keyword→char (`space`→`' '`, `hyphen`→`'-'`, `comma`→`','`) and decodes sentinels in both `input` and `expected`.

---

## Task 1: Canonical conformance-vector file

**Files:**
- Create: `design/display-grouping-vectors.tsv`

- [ ] **Step 1: Write the vector file**

Create `design/display-grouping-vectors.tsv` with EXACTLY this content (real tab characters between columns; one header line; no trailing blank line):

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

- [ ] **Step 2: Verify the file is tab-separated (not spaces)**

Run: `awk -F'\t' 'NF!=6{print "BAD LINE "NR": "$0}' design/display-grouping-vectors.tsv`
Expected: no output (every row has exactly 6 tab-delimited fields). If any line prints, a tab was written as spaces — fix it.

- [ ] **Step 3: Commit**

```bash
git add design/display-grouping-vectors.tsv
git commit -m "feat(mstring-grouping): canonical display-grouping conformance vectors (P0)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 2: Reference `render_grouped` + `strip_display_separators`

**Files:**
- Modify: `crates/mnemonic-toolkit/src/format.rs` (add after the `chunk_5char` fn, which ends at line 26)
- Test: unit tests appended to the existing `#[cfg(test)] mod tests` in `format.rs` (the module starts near line 404)

- [ ] **Step 1: Write the failing unit tests**

Append these tests inside the existing `#[cfg(test)] mod tests { ... }` block in `crates/mnemonic-toolkit/src/format.rs`:

```rust
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
```

- [ ] **Step 2: Run to verify failure (functions not defined)**

Run: `cargo test -p mnemonic-toolkit --lib render_grouped`
Expected: FAIL — `cannot find function render_grouped in this scope` (and similar for `strip_display_separators`).

- [ ] **Step 3: Add the implementation**

Insert this immediately AFTER the `chunk_5char` function (after its closing `}` at line 26) and BEFORE `chunk_mk1` in `crates/mnemonic-toolkit/src/format.rs`:

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
/// ignored). Single line always — no newline wrapping. ASCII-safe; works on
/// `char` boundaries (all m-format strings are ASCII).
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
/// separators; any other char (including codex32-alphabet chars) passes through,
/// so a malformed card is never silently "cleaned" into validity.
pub fn strip_display_separators(s: &str) -> String {
    s.chars().filter(|&c| !is_display_separator(c)).collect()
}
```

- [ ] **Step 4: Run to verify the tests pass**

Run: `cargo test -p mnemonic-toolkit --lib render_grouped && cargo test -p mnemonic-toolkit --lib strip_display_separators && cargo test -p mnemonic-toolkit --lib render_then_strip`
Expected: PASS (all the Task-2 tests green).

- [ ] **Step 5: Commit**

```bash
git add crates/mnemonic-toolkit/src/format.rs
git commit -m "feat(mstring-grouping): reference render_grouped + strip_display_separators (P0)

Pure fns per SPEC §3; coexist with chunk_* (deleted in P4). Unit tests cover
empty / group_size>=len / unbroken / trailing-partial / whitespace+hyphen+comma
strip / idempotence / codex32-passthrough / render-then-strip round-trip.

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: Vector-driven conformance test

**Files:**
- Create: `crates/mnemonic-toolkit/tests/display_grouping_conformance.rs`

- [ ] **Step 1: Write the conformance test (it will fail until it can read the TSV + call the fns)**

Create `crates/mnemonic-toolkit/tests/display_grouping_conformance.rs`:

```rust
//! Drives the canonical display-grouping conformance vectors
//! (`design/display-grouping-vectors.tsv`) through the toolkit's reference
//! `render_grouped` / `strip_display_separators`. This SAME vector file is
//! copied (verbatim, checksum-pinned) into each sibling repo in P1–P3, so all
//! four implementations are proven byte-identical. SPEC §8.

use mnemonic_toolkit::format::{render_grouped, strip_display_separators};

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
        "none" => ' ', // unused by render(gs=0) and by strip
        other => panic!("unknown separator keyword: {other}"),
    }
}

#[test]
fn conformance_vectors_pass() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../design/display-grouping-vectors.tsv"
    );
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("read {path}: {e}"));

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
        let gs: usize = gs.parse().unwrap_or_else(|_| panic!("row {}: bad group_size {gs:?}", i + 2));

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

- [ ] **Step 2: Confirm `format` is reachable as `mnemonic_toolkit::format` (already verified)**

Run: `grep -n "pub mod format" crates/mnemonic-toolkit/src/lib.rs`
Expected: `143:pub mod format;` (verified at plan-write time — the toolkit has a lib target `mnemonic-toolkit` plus `[[bin]] mnemonic`; `format.rs` is compiled into both via `lib.rs:143 pub mod format;` and `main.rs:16 mod format;`). Existing integration tests already `use mnemonic_toolkit::...` (e.g. `mlock`, `seed_xor`), so `use mnemonic_toolkit::format::{render_grouped, strip_display_separators}` resolves with NO lib.rs change. If for any reason the grep does not show `pub mod format`, STOP and reconcile before proceeding.

- [ ] **Step 3: Run to verify it passes**

Run: `cargo test -p mnemonic-toolkit --test display_grouping_conformance`
Expected: PASS — `conformance_vectors_pass` green, exercising every TSV row (≥20).

- [ ] **Step 4: Run the whole toolkit test suite (no regressions)**

Run: `cargo test -p mnemonic-toolkit`
Expected: PASS — all pre-existing tests still green (P0 only adds fns + a test; `chunk_*` untouched).

- [ ] **Step 5: Commit**

```bash
git add crates/mnemonic-toolkit/tests/display_grouping_conformance.rs
git commit -m "test(mstring-grouping): vector-driven conformance test over canonical TSV (P0)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Self-Review (completed at write time)

**Spec coverage:** SPEC §3.1 (render) → Task 2 fn + render vectors. §3.2 (strip = whitespace+`-`+`,`) → Task 2 fn + strip vectors (incl. tab/CRLF). §3.3 edge cases (empty, gs≥len, gs0+sep, consecutive seps, idempotence, ASCII) → vectors + unit tests. §4 charset safety → documented in `is_display_separator` doc + the codex32-passthrough vector. §8 vector-encoding convention → Task 1 file + Task 3 decoder. (Flags, emit/intake wiring, releases, sibling copies, manuals = P1–P5, out of P0 scope by design.)

**Placeholder scan:** none — all code/paths/commands are concrete.

**Type consistency:** `render_grouped(&str, usize, char) -> String` and `strip_display_separators(&str) -> String` and `is_display_separator(char) -> bool` are used identically in Task 2 tests, Task 3 driver, and the impl. Separator keyword set {space,hyphen,comma,none} consistent between Task 1 file and Task 3 `sep_char`.

**Crate-layout check (resolved at write time):** the toolkit has a lib target with `pub mod format;` (`lib.rs:143`) AND `[[bin]] mnemonic` (`main.rs:16 mod format;`), so the conformance test's `use mnemonic_toolkit::format::{…}` resolves with no lib change (existing integration tests already import `mnemonic_toolkit::…`). The Task-2 `#[cfg(test)]` unit tests run under `cargo test -p mnemonic-toolkit --lib`.
