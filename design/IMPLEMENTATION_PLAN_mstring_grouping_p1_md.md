# mstring display grouping — P1 (descriptor-mnemonic / `md`) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans. Steps use checkbox (`- [ ]`) syntax.

**Goal:** Bring the `md` CLI (descriptor-mnemonic) onto the standardized mstring display-grouping contract: `--group-size`/`--separator` flags on `md encode`, separator-stripping intake on `md decode`/`md repair`, the canonical conformance vectors copied + checksum-pinned, and a MINOR release of `md-codec` + `md-cli`.

**Architecture:** The two pure fns `render_grouped(s, group_size, separator)` + `strip_display_separators(s)` (+ `is_display_separator`) live in **md-codec** (the lib — md-cli is bin-only, so the lib is the only place a `tests/` conformance test can `use`; same lib-reachability lesson as P0). `md-codec::encode::render_codex32_grouped` is KEPT as a thin wrapper (`render_grouped(s, n, '-')`) so its public API + technical-manual entry survive (spec §8/§11, plan deviation noted below). `md-cli` calls `md_codec::encode::render_grouped` on its `encode` text emit and `md_codec::...::strip_display_separators` before decode in `decode`/`repair`. The canonical TSV is copied verbatim from the toolkit into `descriptor-mnemonic/design/` + a `.sha256`, CI-pinned.

**Tech Stack:** Rust 2021; md's CI builds/tests on 1.85.0, fmt on stable (`cargo fmt --all --check`); clap-derive (subcommands in `crates/md-cli/src/main.rs`); md-cli `gui-schema` auto-generated from clap with a golden test.

**Spec:** `design/SPEC_mstring_display_grouping.md` (R0 GREEN). Implements the P1 row of §12 + the md call-sites in §9.

**Source SHAs (grep-verified at write time):** descriptor-mnemonic `main` `eb9f368` (md-cli 0.6.2, md-codec 0.35.3, md-cli pins md-codec `=0.35.3`). Toolkit canonical vectors at `feature/mstring-display-grouping` (`design/display-grouping-vectors.tsv`, P0).

**Branch:** create `feature/mstring-display-grouping` in descriptor-mnemonic (parallel to the toolkit branch).

**Plan deviation from spec §8 (for plan-R0 to ratify):** spec §8 said "strip lives in md-cli." Because md-cli is **bin-only** (no lib target — confirmed: `crates/md-cli/src/` has `main.rs`, no `lib.rs`), a `tests/` integration test cannot `use md_cli::…`. Hosting BOTH pure fns in md-codec (a lib) keeps the conformance test trivial and the fns reusable; strip-before-decode is a natural pre-decode normalizer to sit beside the decoder. The decoder itself stays pure (no implicit stripping).

**Release boundary:** Tasks 1–6 are reversible branch work. Task 7 (version bump) PREPARES the release; the actual `git tag` + `cargo publish` (md-codec then md-cli) is a SEPARATE outward-facing step that REQUIRES explicit user authorization — do NOT tag/publish without it.

**Plan-R0 gate:** round 1 NOT GREEN (3C/2I) — folded in the corrections block below (review `…plan-r0-p1-round1-review.md`). round 2 NOT GREEN (1C/0I) — Task 3 Step 6 `git add` omitted `smoke.rs`+`cli_repair.rs`; folded (review `…plan-r0-p1-round2-review.md`). **round 3 GREEN (0C/0I)** (review `…plan-r0-p1-round3-review.md`). **GATE MET — cleared for execution in order `1 → 2 → 4 → 3 → 5 → 6 → 7`.**

---

## R0-r1 corrections (MUST APPLY — these override the task bodies where they conflict)

1. **Execution ORDER (I2 — per-commit green):** run the **intake-strip task (Task 4) BEFORE the encode-flags task (Task 3)**. Decode must accept grouped input before `md encode` starts emitting it, else `template_roundtrip.rs` + `json_snapshots.rs` (`md encode | md decode`) break in the intermediate commit. Execute: 1 → 2 → **4 → 3** → 5 → 6 → 7.
2. **Three additional pre-existing test sites the encode-flags task (Task 3) MUST fix** (default output becomes space/5):
   - `crates/md-cli/tests/smoke.rs:19` — `stdout("md1yqpqqxqq8xtwhw4xwn4qh\n")` exact pin → add `--group-size 0` to that `md encode` invocation (keeps the wire-canary exact-pin intact).
   - `crates/md-cli/tests/help_examples.rs` `check_example("encode")` exact-matches against the `Encode` `after_long_help` (`crates/md-cli/src/main.rs:62`). Append ` --group-size 0` to the example command in `after_long_help` so the printed example still reproduces the unbroken literal. Add BOTH `main.rs` and `help_examples.rs` to Task 3's file list.
   - `crates/md-cli/tests/cli_repair.rs` — its `encode_chunked` helper captures `md encode --force-chunked` (now grouped) and feeds `md repair` (output stays unbroken) → ≈5 assertions fail. Make `encode_chunked` pass `--group-size 0` so fixtures are unbroken. Add `cli_repair.rs` to Task 3's file list.
   - Re-confirm `template_roundtrip.rs` + `json_snapshots.rs` pass (they should once Task 4's intake-strip is in).
3. **`address.rs` strip site (I3):** the md1 decode is INSIDE `build_descriptor` (`address.rs:108/111` on `args.phrases`), NOT in `run`. Apply `let phrases = crate::cmd::strip_md1_inputs(args.phrases);` inside `build_descriptor` and use `phrases` for decode/reassemble.
4. **`repair.rs` positional strip (I4) — explicit:** in `read_md1_strings`, strip BOTH the stdin line (`strip_display_separators(line)`) AND each positional arg (replace `out.push(a.clone());` at `repair.rs:92` with `out.push(md_codec::encode::strip_display_separators(a));`).

---

## File Structure

- **Modify** `crates/md-codec/src/encode.rs` — add `render_grouped` + `strip_display_separators` + `is_display_separator`; rewrite `render_codex32_grouped` as a wrapper. (~encode.rs:98.)
- **Create** `descriptor-mnemonic/design/display-grouping-vectors.tsv` — verbatim copy of the toolkit canonical.
- **Create** `descriptor-mnemonic/design/display-grouping-vectors.tsv.sha256` — checksum pin.
- **Create** `crates/md-codec/tests/display_grouping_conformance.rs` — drives `md_codec::encode::{render_grouped, strip_display_separators}` over the TSV.
- **Modify** `.github/workflows/ci.yml` — add a `sha256sum -c` step on the vectors copy.
- **Modify** `crates/md-cli/src/main.rs` — add `--group-size`/`--separator` to the `Encode` subcommand + a shared `--separator` value parser; thread into `EncodeArgs`.
- **Modify** `crates/md-cli/src/cmd/encode.rs` — `EncodeArgs` gains `group_size`/`separator`; wrap the text emit (`:81`, `:84`) with `render_grouped`; `--json` stays unbroken.
- **Modify** ALL SIX md1-intake subcommands to strip separators before decode, via a shared md-cli helper `strip_md1_inputs`: `cmd/decode.rs` (`:8`,`:11`), `cmd/bytecode.rs` (`:8`,`:11`), `cmd/verify.rs` (`:18`,`:21`), `cmd/inspect.rs` (`:11`,`:13`), `cmd/address.rs` (`:108`,`:111`), and `cmd/repair.rs` `read_md1_strings` (`:83` per-line `.trim()` → interior strip). repair OUTPUT stays unbroken (no flags on repair).
- **Modify** `crates/md-cli/src/cmd/mod.rs` — add `pub fn strip_md1_inputs(strings: &[String]) -> Vec<String>` (maps `strip_display_separators` over each).
- **Modify** `crates/md-cli/tests/cmd_encode.rs`, `cmd_decode.rs`, `cli_repair.rs`, `cmd_gui_schema.rs` (golden) — add/refresh tests.
- **Modify** `crates/md-codec/Cargo.toml` (0.35.3 → 0.36.0) + `crates/md-cli/Cargo.toml` (0.6.2 → 0.7.0 + pin `=0.36.0`).

---

## Task 1: md-codec — `render_grouped` + `strip_display_separators` (+ wrapper)

**Files:**
- Modify: `crates/md-codec/src/encode.rs`

- [ ] **Step 1: Write failing unit tests**

Append to the `#[cfg(test)] mod tests` in `crates/md-codec/src/encode.rs` (find it with `grep -n '#\[cfg(test)\]' crates/md-codec/src/encode.rs`):

```rust
#[test]
fn render_grouped_separators_and_unbroken() {
    assert_eq!(render_grouped("abcdefghij", 5, ' '), "abcde fghij");
    assert_eq!(render_grouped("abcdefghij", 5, '-'), "abcde-fghij");
    assert_eq!(render_grouped("abcdefghij", 5, ','), "abcde,fghij");
    assert_eq!(render_grouped("abcdefghij", 0, ' '), "abcdefghij");
    assert_eq!(render_grouped("abcde", 5, ' '), "abcde");
    assert_eq!(render_grouped("abcdefg", 3, '-'), "abc-def-g");
    assert_eq!(render_grouped("", 5, ' '), "");
}

#[test]
fn render_codex32_grouped_still_hyphens() {
    // back-compat wrapper: unchanged behavior
    assert_eq!(render_codex32_grouped("abcdefghij", 5), "abcde-fghij");
    assert_eq!(render_codex32_grouped("abcde", 0), "abcde");
}

#[test]
fn strip_display_separators_whitespace_hyphen_comma() {
    assert_eq!(strip_display_separators("abcde fghij"), "abcdefghij");
    assert_eq!(strip_display_separators("ab-cd,ef gh"), "abcdefgh");
    assert_eq!(strip_display_separators("ab\tcd\r\nef"), "abcdef");
    assert_eq!(strip_display_separators("ms1qpzry9x8"), "ms1qpzry9x8");
    let once = strip_display_separators("a b-c,d");
    assert_eq!(strip_display_separators(&once), once);
}
```

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p md-codec --lib render_grouped`
Expected: FAIL — `cannot find function render_grouped`.

- [ ] **Step 3: Implement**

Replace the existing `render_codex32_grouped` (encode.rs ~:98-110) with the wrapper + the new fns:

```rust
/// True for any character treated as a display separator on intake: ALL Unicode
/// whitespace plus `-` and `,`. SPEC §3.2 (mstring display-grouping). None of
/// these appear in the codex32 alphabet (`qpzry9x8gf2tvdw0s3jn54khce6mua7l`) or
/// the `ms`/`mk`/`md`/`1` structural chars (SPEC §4), so stripping is unambiguous.
pub fn is_display_separator(c: char) -> bool {
    c.is_whitespace() || c == '-' || c == ','
}

/// Insert `separator` after every `group_size` characters (SPEC §3.1).
/// `group_size == 0` returns the input unchanged. Single line; ASCII-safe.
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

/// Strip every display separator (SPEC §3.2) — used on intake before decode.
/// Idempotent; strips ONLY separators (other chars pass through, so a malformed
/// card is never silently "cleaned" into validity).
pub fn strip_display_separators(s: &str) -> String {
    s.chars().filter(|&c| !is_display_separator(c)).collect()
}

/// Render a codex32 string with optional N-char HYPHEN grouping for
/// transcription aid (spec §10.2). `group_size = 0` returns the input unchanged.
/// Back-compat wrapper over `render_grouped` (hyphen separator). Retained as
/// public API (documented in the technical manual); new callers use
/// `render_grouped` with an explicit separator.
pub fn render_codex32_grouped(s: &str, group_size: usize) -> String {
    render_grouped(s, group_size, '-')
}
```

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p md-codec --lib render_grouped && cargo test -p md-codec --lib render_codex32_grouped && cargo test -p md-codec --lib strip_display_separators`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add crates/md-codec/src/encode.rs
git commit -m "feat(md-codec): render_grouped + strip_display_separators; render_codex32_grouped becomes a wrapper (mstring-grouping P1)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 2: Conformance vectors (copy + checksum + test + CI)

**Files:**
- Create: `descriptor-mnemonic/design/display-grouping-vectors.tsv` (+ `.sha256`)
- Create: `crates/md-codec/tests/display_grouping_conformance.rs`
- Modify: `.github/workflows/ci.yml`

- [ ] **Step 1: Copy the canonical vectors verbatim + generate the checksum**

```bash
mkdir -p design
cp /scratch/code/shibboleth/mnemonic-toolkit/design/display-grouping-vectors.tsv design/display-grouping-vectors.tsv
( cd design && sha256sum display-grouping-vectors.tsv > display-grouping-vectors.tsv.sha256 )
# verify it matches the toolkit canonical byte-for-byte
diff design/display-grouping-vectors.tsv /scratch/code/shibboleth/mnemonic-toolkit/design/display-grouping-vectors.tsv && echo IDENTICAL
( cd design && sha256sum -c display-grouping-vectors.tsv.sha256 )
```
Expected: `IDENTICAL` and `display-grouping-vectors.tsv: OK`.

- [ ] **Step 2: Write the conformance test (red until it can `use` the fns)**

Create `crates/md-codec/tests/display_grouping_conformance.rs`:

```rust
//! Same canonical display-grouping vectors as the toolkit + the other siblings
//! (copy is checksum-pinned in CI). Proves md-codec's render/strip match
//! byte-for-byte. SPEC §8.

use md_codec::encode::{render_grouped, strip_display_separators};

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
        "none" => ' ',
        other => panic!("unknown separator keyword: {other}"),
    }
}

#[test]
fn conformance_vectors_pass() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../design/display-grouping-vectors.tsv");
    let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    let mut lines = text.lines();
    assert_eq!(
        lines.next().expect("header"),
        "op\tinput\tgroup_size\tseparator\texpected\tnote",
        "vector header drift"
    );
    let mut count = 0usize;
    for (i, line) in lines.enumerate() {
        if line.is_empty() {
            continue;
        }
        let c: Vec<&str> = line.split('\t').collect();
        assert_eq!(c.len(), 6, "row {} not 6 fields: {line:?}", i + 2);
        let (op, input, gs, sep, expected, note) = (c[0], c[1], c[2], c[3], c[4], c[5]);
        let (input, expected) = (decode(input), decode(expected));
        let gs: usize = gs.parse().unwrap_or_else(|_| panic!("row {}: bad group_size", i + 2));
        let got = match op {
            "render" => render_grouped(&input, gs, sep_char(sep)),
            "strip" => strip_display_separators(&input),
            other => panic!("row {}: unknown op {other:?}", i + 2),
        };
        assert_eq!(got, expected, "row {} ({note})", i + 2);
        count += 1;
    }
    assert!(count >= 20, "expected >=20 rows, got {count}");
}
```

- [ ] **Step 3: Verify the path depth + run**

Run: `cargo test -p md-codec --test display_grouping_conformance`
Expected: PASS. (Path: `CARGO_MANIFEST_DIR` = `…/descriptor-mnemonic/crates/md-codec`, so `../../design` = repo-root `design/`. If the file isn't found, confirm the crate is two levels below repo root with `ls crates/md-codec/Cargo.toml`.)

- [ ] **Step 4: Add the CI checksum gate**

In `.github/workflows/ci.yml`, inside the existing `fmt` or `build`/`test` job (pick the job that does a plain `actions/checkout` + a shell step; the `fmt` job at lines ~49-57 is fine), add a step BEFORE the cargo invocation:

```yaml
      - name: conformance-vector checksum pin
        run: cd design && sha256sum -c display-grouping-vectors.tsv.sha256
```

(If no job has a convenient shell-step slot, add it to the `test` job after checkout.)

- [ ] **Step 5: Commit**

```bash
git add design/display-grouping-vectors.tsv design/display-grouping-vectors.tsv.sha256 crates/md-codec/tests/display_grouping_conformance.rs .github/workflows/ci.yml
git commit -m "test(md-codec): copy canonical display-grouping vectors + checksum-pin + conformance test (mstring-grouping P1)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 3: `md encode` — `--group-size` / `--separator` flags

**Files:**
- Modify: `crates/md-cli/src/main.rs` (Encode subcommand clap + a `parse_separator` value parser + EncodeArgs construction)
- Modify: `crates/md-cli/src/cmd/encode.rs`

- [ ] **Step 1: Write failing CLI tests**

Append to `crates/md-cli/tests/cmd_encode.rs` (use the file's existing helper to invoke `md`; mirror an existing encode test for the command scaffold). Behaviors:

```rust
// default = space/5 single line
#[test]
fn encode_default_groups_space_5() {
    // <build the same `md encode --template ... --key ...` invocation an existing
    // passing test uses>, then:
    // assert stdout's md1 line contains " " every 5 chars and equals the unbroken
    // string with spaces inserted. Concretely assert: the md1 line, with spaces
    // removed, parses; and the md1 line contains a ' ' at index 5.
}

#[test]
fn encode_unbroken_group_size_0() {
    // same invocation + `--group-size 0`; assert the md1 line contains NO ' '/'-'/','.
}

#[test]
fn encode_separator_hyphen() {
    // same invocation + `--separator hyphen`; assert the md1 line contains '-' at index 5.
}

#[test]
fn encode_rejects_bad_separator() {
    // `--separator bogus` exits non-zero (clap value_parser error, code 2).
}
```
(Fill the `<…>` invocation from the nearest existing `cmd_encode.rs` test that produces a single md1 string — copy its `Command::cargo_bin("md").args([...])` setup verbatim so the test is real, not a placeholder.)

- [ ] **Step 2: Run to verify failure**

Run: `cargo test -p md-cli --test cmd_encode encode_default_groups_space_5`
Expected: FAIL (flag unknown / output not grouped).

- [ ] **Step 3: Add the clap flags + value parser + wiring**

In `crates/md-cli/src/main.rs`: add a separator value-parser fn (shared by all md subcommands that will gain it) and two args on the `Encode` variant (after `force_chunked: bool,` ~line 95):

```rust
/// Parse `--separator`: accepts keyword (space|hyphen|comma) or literal (" "|-|,).
/// Returns the separator char. SPEC §5.
fn parse_separator(s: &str) -> Result<char, String> {
    match s {
        "space" | " " => Ok(' '),
        "hyphen" | "-" => Ok('-'),
        "comma" | "," => Ok(','),
        other => Err(format!(
            "invalid separator {other:?}; expected one of: space|hyphen|comma (or the literal char)"
        )),
    }
}
```
On the `Encode` variant:
```rust
        /// Insert a separator every N characters in the emitted md1 string
        /// (0 = unbroken). SPEC §3. Display only; --json stays unbroken.
        #[arg(long, default_value_t = 5)]
        group_size: u16,
        /// Separator: space|hyphen|comma (keyword) or the literal " "|-|, . SPEC §5.
        #[arg(long, default_value = "space", value_parser = parse_separator)]
        separator: char,
```
Thread both into the `EncodeArgs { … }` construction in the Encode dispatch arm (add `group_size: group_size as usize, separator,`).

In `crates/md-cli/src/cmd/encode.rs`: add to `EncodeArgs`:
```rust
    pub group_size: usize,
    pub separator: char,
```
and import `use md_codec::encode::{encode_md1_string, render_grouped};` (add `render_grouped`). Wrap the TEXT emit only (NOT the json branch). Replace the chunked-loop body (`:81`) and the single-string emit (`:84`):
```rust
    if args.force_chunked {
        let chunks = split(&descriptor)?;
        let csid = derive_chunk_set_id(&compute_md1_encoding_id(&descriptor)?);
        println!("chunk-set-id: 0x{csid:05x}");
        for s in &chunks {
            println!("{}", render_grouped(s, args.group_size, args.separator));
        }
    } else {
        println!(
            "{}",
            render_grouped(&encode_md1_string(&descriptor)?, args.group_size, args.separator)
        );
    }
```

- [ ] **Step 4: Run to verify pass**

Run: `cargo test -p md-cli --test cmd_encode`
Expected: PASS (new tests + pre-existing encode tests still green — pre-existing tests that asserted the unbroken output must be checked: any test asserting an exact md1 string now sees spaces. Update those to either pass `--group-size 0` or assert the space-grouped form. Enumerate and fix them in this step.)

- [ ] **Step 5: Confirm the gui-schema test still passes**

`md gui-schema` auto-generates from clap, so the new flags appear automatically. `cmd_gui_schema.rs` is ASSERTION-based (it checks subcommand names + that `--context` is a dropdown — NOT an exhaustive golden of every flag), so adding `--group-size`/`--separator` should NOT break it.
Run: `cargo test -p md-cli --test cmd_gui_schema`
Expected: PASS unchanged. If it does assert an exact encode flag set somewhere, update that assertion to include the two new flags (`--separator` renders as a text flag in the auto-gen schema — acceptable; not the toolkit's hand-mirror).

- [ ] **Step 6: Commit**

```bash
git add crates/md-cli/src/main.rs crates/md-cli/src/cmd/encode.rs \
        crates/md-cli/tests/cmd_encode.rs crates/md-cli/tests/cmd_gui_schema.rs \
        crates/md-cli/tests/smoke.rs crates/md-cli/tests/cli_repair.rs
git commit -m "feat(md-cli): md encode --group-size/--separator (default space/5 print-once); json stays unbroken (mstring-grouping P1)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 4: separator-stripping intake on ALL SIX md1-intake subcommands

Every md subcommand that decodes an md1 string is an intake site (verified against `main.rs` dispatch + each `cmd/*.rs`): `decode` (`decode.rs:8/11`), `bytecode` (`bytecode.rs:8/11`), `verify` (`verify.rs:18/21`), `inspect` (`inspect.rs:11/13`), `address` (`address.rs:108/111`), `repair` (`repair.rs` `read_md1_strings:83`). All decode/reassemble positionals (only `repair` has a `-`→stdin path). Use ONE shared helper so no site is missed.

**Files:**
- Modify: `crates/md-cli/src/cmd/mod.rs` (add `strip_md1_inputs`)
- Modify: `cmd/decode.rs`, `cmd/bytecode.rs`, `cmd/verify.rs`, `cmd/inspect.rs`, `cmd/address.rs`, `cmd/repair.rs`
- Test: `crates/md-cli/tests/cmd_decode.rs`, `cmd_verify.rs`, `cmd_inspect.rs`, `cmd_bytecode.rs`, `cmd_address.rs`, `cli_repair.rs`

- [ ] **Step 1: Add the shared helper**

In `crates/md-cli/src/cmd/mod.rs`:
```rust
use md_codec::encode::strip_display_separators;

/// Strip mstring display separators (SPEC §3.2) from each md1 input string so a
/// grouped or unbroken card both re-ingest. Applied at every md1-intake site.
pub fn strip_md1_inputs(strings: &[String]) -> Vec<String> {
    strings.iter().map(|s| strip_display_separators(s)).collect()
}
```

- [ ] **Step 2: Write failing tests (one per intake surface)**

For each of `cmd_decode.rs`, `cmd_verify.rs`, `cmd_inspect.rs`, `cmd_bytecode.rs`, `cmd_address.rs`, `cli_repair.rs`: add a test that takes a SPACE- or HYPHEN-grouped form of a known-good md1 (the grouped form of a fixture the file already uses) and asserts the command succeeds with the SAME result as the unbroken form. Copy each file's existing `Command::cargo_bin("md").args([...])` scaffold + md1 fixture verbatim; produce the grouped variant by inserting a space every 5 chars (or reuse `md encode` default output).

- [ ] **Step 3: Run to verify failure**

Run: `cargo test -p md-cli --test cmd_decode --test cmd_verify --test cmd_inspect --test cmd_bytecode --test cmd_address --test cli_repair 2>&1 | tail`
Expected: the new grouped-input tests FAIL (embedded separators rejected by decode).

- [ ] **Step 4: Apply the strip at each site**

In each of `decode.rs`, `bytecode.rs`, `verify.rs`, `inspect.rs`, `address.rs`: at the top of `run`, replace the raw `strings`/`phrases` slice with the stripped copy before the `decode_md1_string`/`reassemble` branch, e.g. in `decode.rs`:
```rust
pub fn run(strings: &[String], json: bool) -> Result<u8, CliError> {
    let strings = crate::cmd::strip_md1_inputs(strings);
    let descriptor = if strings.len() == 1 {
        decode_md1_string(&strings[0])?
    } else {
        let refs: Vec<&str> = strings.iter().map(String::as_str).collect();
        reassemble(&refs)?
    };
    // ...unchanged
```
Apply the analogous one-line `let strings = crate::cmd::strip_md1_inputs(strings);` (or `args.strings` / `args.phrases`) at the top of each of `bytecode.rs`, `verify.rs` (`args.strings`), `inspect.rs`, `address.rs` (`args.phrases`). In `repair.rs::read_md1_strings`, replace the per-line `let s = line.trim();` (`:83`) with `let s = md_codec::encode::strip_display_separators(line); let s = s.as_str();` AND apply `strip_display_separators` to each positional arg too. (repair OUTPUT stays unbroken; no `--group-size`/`--separator` on repair.)

- [ ] **Step 5: Run to verify pass**

Run: `cargo test -p md-cli --test cmd_decode --test cmd_verify --test cmd_inspect --test cmd_bytecode --test cmd_address --test cli_repair`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add crates/md-cli/src/cmd/mod.rs crates/md-cli/src/cmd/decode.rs crates/md-cli/src/cmd/bytecode.rs crates/md-cli/src/cmd/verify.rs crates/md-cli/src/cmd/inspect.rs crates/md-cli/src/cmd/address.rs crates/md-cli/src/cmd/repair.rs crates/md-cli/tests/cmd_decode.rs crates/md-cli/tests/cmd_verify.rs crates/md-cli/tests/cmd_inspect.rs crates/md-cli/tests/cmd_bytecode.rs crates/md-cli/tests/cmd_address.rs crates/md-cli/tests/cli_repair.rs
git commit -m "feat(md-cli): strip display separators on ALL six md1-intake surfaces (decode/bytecode/verify/inspect/address/repair) (mstring-grouping P1)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 5: Full suite + fmt gate

- [ ] **Step 1: Whole workspace test**

Run: `cargo test --workspace`
Expected: ALL green. Fix any pre-existing test that asserted unbroken md encode output (grep tests for exact md1 literals; add `--group-size 0` or update the expectation).

- [ ] **Step 2: fmt gate (md uses stable, not 1.95.0)**

Run: `cargo fmt --all --check`
Expected: no diff. If diff, run `cargo fmt --all` and re-commit (md has NO mlock.rs exemption — full fmt is fine here).

- [ ] **Step 3: Commit any test-fixup**

```bash
git add -p   # stage only the intended fixes (NOT git add -A)
git commit -m "test(md): fix vectors/goldens for default grouped encode output (mstring-grouping P1)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 6: Sibling FOLLOWUP companion

- [ ] **Step 1: File the companion entry**

Add to descriptor-mnemonic's `FOLLOWUPS.md` (find it: `git ls-files | grep -i followup`) an entry `display-grouping-render-strip-v1` with a `Companion:` line cross-citing the toolkit's entry (and add the matching entry in the toolkit's `design/FOLLOWUPS.md` in P4). Commit.

```bash
git add <followups path>
git commit -m "docs(followups): display-grouping-render-strip-v1 companion (mstring-grouping P1)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

---

## Task 7: Version bump (release PREP — do NOT tag/publish without user authorization)

**Files:**
- Modify: `crates/md-codec/Cargo.toml`, `crates/md-cli/Cargo.toml`, `Cargo.lock`, CHANGELOG(s) if present.

- [ ] **Step 1: Bump versions (MINOR, pre-1.0)**

- `crates/md-codec/Cargo.toml`: `version = "0.35.3"` → `"0.36.0"`.
- `crates/md-cli/Cargo.toml`: `version = "0.6.2"` → `"0.7.0"` AND the dep pin `md-codec = { path = "../md-codec", version = "=0.35.3" }` → `version = "=0.36.0"`.
- Update any CHANGELOG.md under the repo (check `git ls-files | grep -i changelog`).
- Run `cargo update -p md-codec -p md-cli` (or `cargo build`) to refresh `Cargo.lock`.

- [ ] **Step 2: Final green**

Run: `cargo test --workspace && cargo fmt --all --check`
Expected: all green.

- [ ] **Step 3: Commit the bump**

```bash
git add crates/md-codec/Cargo.toml crates/md-cli/Cargo.toml Cargo.lock <changelog if any>
git commit -m "release: md-codec 0.36.0 + md-cli 0.7.0 — mstring display grouping (P1)

Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>"
```

- [ ] **Step 4: STOP — request release authorization**

Do NOT `git tag`, `git push`, or `cargo publish`. Report to the user that P1 is code-complete + green on `feature/mstring-display-grouping`, and request authorization for the md-codec→md-cli crates.io publish + tags (md-codec must publish first since md-cli pins `=0.36.0`).

---

## Self-Review (completed at write time)

**Spec coverage:** §3 algorithm → Task 1 (md-codec fns) + Task 2 vectors. §4 charset → Task 1 doc. §5 separator set/parser → Task 3 `parse_separator` (keyword+literal). §6 print-once + json/repair invariants → Task 3 (text emit only; json untouched) + Task 4 (repair output unbroken, no flags). §8 vectors + checksum → Task 2. §9 md emit (encode) + intake (decode/repair) → Tasks 3/4. §9 NOTE: md `convert`/`ms-shares` etc. are toolkit-only; md-cli's emit surface is `encode` (+ chunked); intake is `decode`/`repair`. §11 technical-manual render_codex32_grouped → KEPT as wrapper (no manual break); the md flags' end-user-manual entry is in the TOOLKIT repo and lands in P4 with the toolkit pin bump (md has no own manual — confirmed). §12 P1 row → all tasks. SemVer MINOR → Task 7.

**Placeholder scan:** two intentional `<…>` fill-ins in Task 3 Step 1 / Task 4 Step 1 — the exact `Command::cargo_bin("md").args([...])` scaffold must be copied from the nearest existing passing test in the same test file (named explicitly: `cmd_encode.rs`, `cmd_decode.rs`, `cli_repair.rs`). This is a deliberate "use the repo's existing harness" instruction, not an unspecified behavior. All code steps otherwise show complete code.

**Type consistency:** `render_grouped(&str, usize, char) -> String`, `strip_display_separators(&str) -> String`, `parse_separator(&str) -> Result<char, String>`, `EncodeArgs.group_size: usize` / `.separator: char` (clap `u16` cast to `usize` at construction). Consistent across md-codec, md-cli, and the conformance test.

**Open items for plan-R0:** (1) ratify the §8 deviation (both pure fns in md-codec, not md-cli, because md-cli is bin-only). (2) RESOLVED — md1 intake is positional-only on `decode`/`bytecode`/`verify`/`inspect`/`address`; only `repair` has a `-`→stdin path; all six covered by Task 4's shared `strip_md1_inputs`. (3) RESOLVED — `cmd_gui_schema.rs` is assertion-based (subcommand names + `--context` dropdown), not an exhaustive golden, so the new flags should not break it; a `value_parser`-based `--separator` renders as a text flag in md's auto-gen schema, which is acceptable (the toolkit's hand-mirror is P5, separate). (4) grep found NO `cmd_encode.rs` test asserting an exact md1 literal; Task 3 Step 4 + Task 5 catch any pre-existing exact-output assertions at test-run time.
