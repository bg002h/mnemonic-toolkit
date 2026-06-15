# mstring display grouping — P2 (mnemonic-secret / `ms`) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:executing-plans. Steps use checkbox (`- [ ]`).

**Goal:** Bring the `ms` CLI onto the standardized mstring display-grouping contract: `--group-size`/`--separator` on `ms encode` + `ms split` (default **space/5 print-once**); `ms split` shares one-per-line on stdout with labels→stderr; `ms combine` gains `-`→stdin multiline share intake; separator-stripping intake everywhere (decode/inspect/repair/encode-hex/combine); the doubling-dedup heuristic removed; canonical conformance vectors copied + checksum-pinned; a MINOR release of `ms-cli`.

**Architecture (deviation from spec §8 — for plan-R0 to ratify):** spec §8 said the two pure fns live in `crates/ms-cli/src/format.rs`. `ms-cli` is **bin-only** (`[[bin]] name = "ms" path = "src/main.rs"`, no `lib.rs`), so a `tests/` integration test cannot `use ms_cli::…`. We host `render_grouped` + `strip_display_separators` + `is_display_separator` + the `parse_separator` value-parser **locally in `ms-cli` `format.rs`** (spec-§8-faithful) and write the conformance test as a **`#[cfg(test)] mod` INSIDE the bin crate** (a bin-crate unit test CAN reach `format`'s fns; runs under `cargo test -p ms-cli` / `--workspace`). **`ms-codec` is UNTOUCHED** → no ms-codec bump/publish. This matches the constellation design (drift control = copy-with-checksum vectors; each repo carries its OWN local impl — md was the exception because md-cli is bin-only AND md-codec already owned `render_codex32_grouped`).

**Source SHA (grep-verified at write time):** `mnemonic-secret` `master` `b616530` (ms-cli 0.7.0, ms-codec 0.4.4, ms-cli pins ms-codec `=0.4.4`). Toolkit canonical vectors at `feature/mstring-display-grouping` (`design/display-grouping-vectors.tsv`, P0).

**Branch:** create `feature/mstring-display-grouping` in mnemonic-secret.

**Spec:** `design/SPEC_mstring_display_grouping.md` (R0 GREEN). Implements the P2 rows of §9 + §10 + §12-Phase-1.

---

## KEY FINDINGS / DECISIONS (recon `b616530`; these drive the task design)

1. **ms-codec decode does NOT tolerate separators** (no "D11"-style strip — `grep` of `crates/ms-codec/src/` finds none). ms-cli's `parse.rs::strip_whitespace` does the stripping at the CLI layer **today, for whitespace only**. So the **net-new** intake coverage of `strip_display_separators` is **hyphen + comma** (whitespace already re-ingests). **Intake round-trip tests MUST use comma- or hyphen-grouped fixtures** to genuinely RED→GREEN the new behavior (a space-grouped fixture would false-pass via the existing whitespace strip — the P1 lesson, different root cause).

2. **Per-commit-green ORDERING is EMIT-FIRST, then INTAKE (OPPOSITE of P1).** Removing the doubling-dedup heuristic (`parse.rs::strip_whitespace`) while `ms encode` still prints twice would break `ms encode | ms decode -` (`encode_pipe_to_decode.rs`) — the un-deduped doubled string fails decode. So: make **emit print-once FIRST** (doubling stops, heuristic becomes unreachable, suite stays green with the heuristic still present), **THEN** remove the heuristic + add hyphen/comma stripping. Execute: 1 → 2 → **3 (emit) → 4 (intake)** → 5 → 6 → 7.

3. **Spec §13 note is WRONG about `encode_canonical_24_word.rs`.** The spec said it "does not assert `\n\n`, so it won't go RED." It DOES assert `stdout.starts_with("ms10entrsqqqq")` (`:14`), which **breaks** under default space/5 grouping (a space lands at index 5 → `"ms10e ntrs…"`). Both canonical tests need fixing.

4. **`ms combine` gets INTAKE only, NOT emit flags.** Spec §9.1's emit-with-flags list is `ms encode` + `ms split` (NOT combine). `ms combine --to ms1` is a recovery re-encode (like `repair`) → stays **unbroken**, no grouping flags. `ms combine` DOES gain `-`→stdin + per-share strip (spec §9.2/§15 C1+C3).

5. **No `cargo fmt` CI gate** in `mnemonic-secret` (`rust.yml` jobs: test matrix, mlock-einval, miri, clippy `-D warnings`, g6-invariant). The gate is **clippy `-D warnings`** + the **g6 mlock byte-equality invariant**. **NEVER touch `crates/ms-cli/src/mlock.rs`** (g6). Run `cargo fmt` only on changed non-mlock files for cleanliness; it is not a CI gate.

6. **ms-codec is NOT bumped.** Only `ms-cli` bumps `0.7.0 → 0.8.0` (MINOR — default-output change). Tag `ms-cli-v0.8.0`. ms-codec stays `0.4.4`.

---

## R0-r1 corrections (MUST APPLY — these override the task bodies where they conflict)

Plan-R0 round 1 = NOT GREEN (0C/4I; review `design/agent-reports/mstring-display-grouping-plan-r0-p2-round1-review.md`). All four are breaking tests the enumeration missed. **Task 3 MUST fix ALL of these** (they break on the default-grouping commit; staging/repairing them keeps the commit green):

- **(I1) `tests/encode_no_engraving_card.rs:19`** — `starts_with("ms10entrsqqqq")` breaks under grouping (`"ms10e ntrs…"`). Fix: assert single-line + space-stripped `starts_with("ms10entrsqqqq")` (drop the raw `starts_with`), OR add `--group-size 0` to that invocation.
- **(I2) `tests/encode_hex_input.rs:13`** (`encode_hex_zeros_16_bytes`) — same `starts_with("ms10entrsqqqq")` break. Fix analogously (space-strip then prefix-check, or `--group-size 0`). (`encode_hex_omits_language_in_engraving_card` checks only stderr — unaffected.)
- **(I3) `tests/encode_mnem_japanese.rs`** — FOUR tests assert exact first-line LENGTH (`:30` `== 51`, `:74` `== 50`, `:89` `== 50`, plus `encode_japanese_phrase_decode_round_trip`'s `ms1.len() == 51`). Grouping adds spaces → lengths grow (51→61, 50→59). Fix: add `--group-size 0` to those invocations (cleanest — pure length checks; length unchanged), OR `strip_display_separators(first_line)` before the length check.
- **(I4) `tests/cli_split.rs`** — the `--group-size 0` fix for `split_english_phrase_emits_n_shares_text:44` is correct. ALSO add a concrete NEW test `split_grouped_default_labels_on_stderr`: run `ms split --phrase ENGLISH_12 -k 2 -n 3` (default grouped); assert stdout = exactly 3 lines, each `starts_with("ms1")` AND `contains(' ')`; stdout does NOT contain `"share "`; stderr contains `"share 1 of 3"`.

Minor folds: (m1) `split.rs::emit_text` ordering — emit ALL stdout shares first, THEN all stderr labels. (m3) `parse.rs::strip_whitespace` new doc must reference `format::strip_display_separators` and drop the §3.2 doubling rationale.

---

## Call-site inventory (grep-verified `b616530`)

**Emit (apply `render_grouped` + flags):**
- `cmd/encode.rs::emit_text` (`:198-201`): `println!("{ms1}"); println!(); println!("{}", chunked(ms1));` → ONE `println!("{}", render_grouped(ms1, gs, sep))`. `EncodeArgs` += `group_size`/`separator`.
- `cmd/split.rs::emit_text` (`:147-164`): bare-shares loop (`:152-154`) + blank + per-share labeled `chunked` blocks (`:157-163`) → stdout = N `render_grouped(share, gs, sep)` lines; **labels ("share i of n:") → stderr**. `SplitArgs` += `group_size`/`separator`.

**Intake (separator strip):**
- `parse.rs::strip_whitespace` (`:97-110`) → reimplement as `format::strip_display_separators` (keeps whitespace, ADDS `-`/`,`; **REMOVE the doubling-dedup heuristic**). `read_input` (`:21-27`) is the shared reader → automatically covers `decode.rs:42`, `inspect.rs:33`, `repair.rs:75`, `encode.rs:89` (`--hex`).
- `cmd/combine.rs` (`CombineArgs.shares` positional, `:37-38`; consumed `:52/:58`): ADD a `read_shares` helper — `-`→stdin (one share/line) + `strip_display_separators` each positional/stdin share. (Parallel to mk's `read_mk1_strings`; absent today.)

**Delete:** `format.rs::chunked` (`:14-32`) + its 3 unit tests (`:154-192`). **Remove:** `parse.rs` dedup tests `strip_whitespace_dedupes_doubled_content` (`:138`) + the doubling clause of `strip_whitespace_handles_all_three_workflows` (`:122`).

**Pre-existing tests that BREAK under default grouping (fix in Task 3 unless noted):**
- `tests/encode_canonical_12_word.rs` — asserts `contains("\n\n")` + `starts_with("ms10entrsqqqq")` → **rewrite** (the print-once conversion test).
- `tests/encode_canonical_24_word.rs` — `starts_with("ms10entrsqqqq")` breaks → fix.
- `tests/encode_output_unchanged_after_split_refactor.rs` — 3 TEXT tests assert exact `<ms1>\n\n<chunked>` (incl. the japanese wrap@10 `l` on its own line) → update to print-once grouped; the 3 `*_json_*` tests are unaffected (json unbroken).
- `tests/cli_split.rs::split_english_phrase_emits_n_shares_text` — filters share lines by `!l.contains(' ')` (`:44`) → grouped shares contain spaces; add `--group-size 0` to that invocation (keeps the bare-share parse) + add a NEW test for grouped default + labels-on-stderr **(I4 — concrete spec in the corrections block).**
- `src/format.rs` 3 `chunked_*` unit tests — `chunked` deleted → replaced by `render_grouped` unit tests (Task 1).
- **(I1) `tests/encode_no_engraving_card.rs:19`** — `starts_with("ms10entrsqqqq")` breaks under grouping.
- **(I2) `tests/encode_hex_input.rs:13`** — same `starts_with` break.
- **(I3) `tests/encode_mnem_japanese.rs`** — FOUR first-line-LENGTH assertions (`:30`/`:74`/`:89` + round-trip) break as grouping adds spaces.
- **Suite-sweep (Task 5) must also re-confirm:** `encode_pipe_to_decode.rs`, `decode_routes_share_to_is_share_not_single_string.rs`, `encode_emits_passphrase_warning.rs`, `encode_no_engraving_card.rs`, `cli_output_class.rs`, `gui_schema_emits_spec_v7_json.rs` (new flags appear in encode/split — confirm assertion-based, not exhaustive golden), and any other test asserting exact encode/split stdout. Grep `tests/` for `\n\n`, `chunked`, exact `ms10…` literals, and `--phrase`-then-stdout-equality.

---

## Task 1: `ms-cli` `format.rs` — pure fns + `parse_separator` (+ replace `chunked` unit tests)

**Files:** Modify `crates/ms-cli/src/format.rs`.

- [ ] **Step 1: Write failing unit tests.** In `format.rs`'s `#[cfg(test)] mod tests`, REPLACE the 3 `chunked_*` tests with:
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
    fn strip_display_separators_ws_hyphen_comma() {
        assert_eq!(strip_display_separators("ab cd-ef,gh"), "abcdefgh");
        assert_eq!(strip_display_separators("ms10\tentrs\r\nqq"), "ms10entrsqq");
        let once = strip_display_separators("a b-c,d");
        assert_eq!(strip_display_separators(&once), once);
    }
    #[test]
    fn parse_separator_keyword_and_literal() {
        assert_eq!(parse_separator("space").unwrap(), ' ');
        assert_eq!(parse_separator(" ").unwrap(), ' ');
        assert_eq!(parse_separator("hyphen").unwrap(), '-');
        assert_eq!(parse_separator("comma").unwrap(), ',');
        assert!(parse_separator("bogus").is_err());
    }
```
- [ ] **Step 2:** `cargo test -p ms-cli --bin ms render_grouped` → FAIL (fns absent).
- [ ] **Step 3: Implement** in `format.rs` (replace the `chunked` fn at `:9-32`):
```rust
/// True for any display separator on intake: ALL Unicode whitespace + `-` + `,`
/// (SPEC §3.2). None appear in the codex32 alphabet or `ms`/`1` structural chars.
pub fn is_display_separator(c: char) -> bool {
    c.is_whitespace() || c == '-' || c == ','
}

/// Insert `separator` after every `group_size` chars (SPEC §3.1). `group_size == 0`
/// returns the input unchanged. Single line (legacy wrap@10 removed).
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
/// Idempotent; strips ONLY separators (plain filter, NO doubling-dedup — that
/// heuristic is removed now that emit is print-once, §10).
pub fn strip_display_separators(s: &str) -> String {
    s.chars().filter(|&c| !is_display_separator(c)).collect()
}

/// Parse `--separator`: keyword (`space|hyphen|comma`) or literal (`" "|-|,`).
/// SPEC §5. clap value-parser; rejection is an exit-64 parse error.
pub fn parse_separator(s: &str) -> Result<char, String> {
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
(Drop the now-unused `chunked` doc references in the module header comment.)
- [ ] **Step 4:** `cargo test -p ms-cli --bin ms render_grouped strip_display_separators parse_separator` → PASS. **Do NOT delete `chunked`'s callers yet** — `chunked` is removed in Task 3 (which updates `encode.rs`/`split.rs`). To keep Task 1 compiling, `chunked` is REPLACED in this same edit only if its callers are updated; otherwise KEEP `chunked` through Task 1 and delete in Task 3. **Decision: keep `chunked` in Task 1** (add the new fns alongside; delete `chunked` + its callers in Task 3) so Task 1 compiles standalone. (Then the 3 `chunked_*` unit tests stay until Task 3 — adjust Step 1 to ADD the new unit tests WITHOUT deleting the `chunked_*` tests; the `chunked_*` tests delete in Task 3.)
- [ ] **Step 5: Commit** `crates/ms-cli/src/format.rs`.

## Task 2: Conformance vectors (copy + checksum + bin-crate test + CI)

**Files:** Create `design/display-grouping-vectors.tsv` (+ `.sha256`); add a `#[cfg(test)] mod conformance` to `format.rs` (or a small dedicated `#[cfg(test)]` block); modify `.github/workflows/rust.yml`.

- [ ] **Step 1:** `cp` the toolkit canonical TSV to `design/`; generate `.sha256`; `diff` byte-identical; `sha256sum -c`.
- [ ] **Step 2:** Add a conformance test reachable in the bin crate. Add to `format.rs`:
```rust
#[cfg(test)]
mod conformance {
    use super::{render_grouped, strip_display_separators};
    fn decode(f: &str) -> String { if f == "<empty>" { return String::new() }
        f.replace("<sp>"," ").replace("<tab>","\t").replace("<lf>","\n").replace("<cr>","\r") }
    fn sep(k: &str) -> char { match k { "space"=>' ',"hyphen"=>'-',"comma"=>',',"none"=>' ',o=>panic!("sep {o}") } }
    #[test]
    fn conformance_vectors_pass() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../design/display-grouping-vectors.tsv");
        let text = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read {path}: {e}"));
        let mut lines = text.lines();
        assert_eq!(lines.next().unwrap(), "op\tinput\tgroup_size\tseparator\texpected\tnote", "header drift");
        let mut n = 0;
        for (i, line) in lines.enumerate() {
            if line.is_empty() { continue }
            let c: Vec<&str> = line.split('\t').collect();
            assert_eq!(c.len(), 6, "row {} fields", i+2);
            let (op, input, gs, s, exp, note) = (c[0], decode(c[1]), c[2], c[3], decode(c[4]), c[5]);
            let gs: usize = gs.parse().unwrap();
            let got = match op { "render"=>render_grouped(&input, gs, sep(s)), "strip"=>strip_display_separators(&input), o=>panic!("op {o}") };
            assert_eq!(got, exp, "row {} ({note})", i+2); n += 1;
        }
        assert!(n >= 20, "got {n}");
    }
}
```
(`CARGO_MANIFEST_DIR` = `crates/ms-cli`; `../../design` = repo-root `design/`.)
- [ ] **Step 3:** `cargo test -p ms-cli --bin ms conformance_vectors_pass` → PASS.
- [ ] **Step 4: CI checksum gate.** `rust.yml` has no fmt job; add a step to the **`clippy`** job (single ubuntu, has `actions/checkout`) BEFORE the clippy run:
```yaml
      - name: conformance-vector checksum pin
        run: cd design && sha256sum -c display-grouping-vectors.tsv.sha256
```
- [ ] **Step 5: Commit** the TSV, `.sha256`, `format.rs`, `rust.yml`.

## Task 3: EMIT print-once — `ms encode` + `ms split` flags (+ delete `chunked`, fix breaking tests)

**Files:** `cmd/encode.rs`, `cmd/split.rs`, `src/format.rs` (delete `chunked` + its 3 unit tests); tests `encode_canonical_12_word.rs`, `encode_canonical_24_word.rs`, `encode_output_unchanged_after_split_refactor.rs`, `cli_split.rs`, **`encode_no_engraving_card.rs` (I1)**, **`encode_hex_input.rs` (I2)**, **`encode_mnem_japanese.rs` (I3)**.

- [ ] **Step 1: Failing CLI tests.** Append to (or add) encode/split flag tests: `encode_default_groups_space_5` (stdout line 1 has `' '` at index 5; space-stripped == canonical; NO `\n\n`), `encode_unbroken_group_size_0`, `encode_separator_hyphen`, `encode_rejects_bad_separator` (exit 64 — ms maps clap parse errors to 64, see `main.rs:169`), and a `ms split … ` grouped-default test (stdout = N grouped lines, NO labels on stdout; labels on stderr).
- [ ] **Step 2:** Run → FAIL (flags unknown / not grouped).
- [ ] **Step 3: Implement.**
  - `cmd/encode.rs`: `EncodeArgs` += `#[arg(long, default_value_t = 5)] pub group_size: u16,` + `#[arg(long, default_value = "space", value_parser = crate::format::parse_separator)] pub separator: char,`. `emit_text` signature += `group_size`/`separator`; body → `println!("{}", render_grouped(ms1, group_size as usize, separator));` (drop the `println!()` blank + `chunked`). Import `render_grouped` (drop `chunked`). Update `run` to pass `args.group_size`/`args.separator`.
  - `cmd/split.rs`: `SplitArgs` += same two args. `emit_text` signature += `group_size`/`separator`; stdout = `for share { writeln!(out, "{}", render_grouped(share, gs, sep)); }`; labels → stderr: `let mut err = io::stderr().lock(); for (i, share) in shares.iter().enumerate() { writeln!(err, "share {} of {}:", i+1, shares.len()); }`. Drop `chunked` import.
  - `src/format.rs`: DELETE `chunked` + its 3 `chunked_*` unit tests.
  - **Fix breaking tests:** `encode_canonical_12_word.rs` → rewrite (default = single line, `' '` at idx 5, NO `\n\n`, space-stripped starts with `ms10entrsqqqq`; stderr unchanged). `encode_canonical_24_word.rs` → assert default grouped single-line + space-stripped `starts_with("ms10entrsqqqq")` + stderr `word count: 24`. `encode_output_unchanged_after_split_refactor.rs` → update the 3 TEXT assertions to the print-once grouped single-line form (compute the grouped string = canonical with `' '` every 5; the japanese case is now ONE line, no trailing `l`); json tests unchanged. `cli_split.rs::split_english_phrase_emits_n_shares_text` → add `--group-size 0` to that invocation (keeps bare-share parse) AND add `split_grouped_default_labels_on_stderr` (stdout lines are grouped shares, no "share … of" on stdout; stderr contains "share 1 of 3").
- [ ] **Step 4:** `cargo test -p ms-cli` (encode/split/canonical/refactor) → PASS.
- [ ] **Step 5: Commit** all Task-3 files.

## Task 4: INTAKE — separator strip + `ms combine -`→stdin (remove dedup heuristic)

**Files:** `parse.rs`, `cmd/combine.rs`; tests `decode`/`combine` grouped-intake.

- [ ] **Step 1: Failing tests.** `decode_accepts_comma_grouped` (`ms decode <comma-grouped-ms1>` succeeds == unbroken — comma is the net-new separator, §finding 1); `combine_accepts_grouped_positional_shares` + `combine_dash_stdin_round_trips` (`ms split --group-size 0 … | …` then `ms combine -` via `write_stdin`, plus a comma-grouped positional share set). Use the existing `cli_combine.rs` share fixtures.
- [ ] **Step 2:** Run → FAIL (comma rejected; `-` not handled by combine).
- [ ] **Step 3: Implement.**
  - `parse.rs::strip_whitespace` → body becomes `crate::format::strip_display_separators(s)` (delegate; keeps whitespace, adds `-`/`,`; NO dedup). **DELETE ONLY `strip_whitespace_dedupes_doubled_content` (`:138`); LEAVE `strip_whitespace_handles_all_three_workflows` (`:122`) INTACT** — plan-R0-r2 m_new_2 confirmed it has no doubling clause (all three sub-cases are non-doubled and survive the body change unchanged). Update the `strip_whitespace` doc comment to reference `format::strip_display_separators` and drop the §3.2 doubling rationale. Keep the name `strip_whitespace` (callers unchanged) — its body now strips `-`/`,` too.
  - `cmd/combine.rs`: add `fn read_shares(args: &[String]) -> Result<Zeroizing<Vec<String>>>` — mirror mk's `read_mk1_strings`: a leading `-` reads stdin one-share-per-line; strip each positional/stdin share via `crate::format::strip_display_separators`; wrap `Zeroizing`. `run` uses `read_shares(&args.shares)?` (replacing the `mem::take` of `args.shares`) before `combine_shares`. (`--to ms1` output stays UNBROKEN — no emit flags on combine.)
- [ ] **Step 4:** `cargo test -p ms-cli` (decode/combine/parse) → PASS.
- [ ] **Step 5: Commit** `parse.rs`, `cmd/combine.rs`, tests.

## Task 5: Full suite + clippy + g6

- [ ] **Step 1:** `cargo test --workspace` → ALL green. Fix any remaining pre-existing test asserting old encode/split stdout (grep `tests/` per the suite-sweep list above; `gui_schema_emits_spec_v7_json.rs` — confirm assertion-based, update if it pins encode/split flag sets).
- [ ] **Step 2:** `cargo clippy --workspace --all-targets -- -D warnings` → clean. (`.map(|s| strip_display_separators(s))` over `&String` needs the closure for the `&String→&str` coercion — not a `redundant_closure`.)
- [ ] **Step 3:** Confirm `crates/ms-cli/src/mlock.rs` is **byte-unchanged** (`git diff --stat` shows no mlock.rs) — g6 invariant. Run `cargo fmt -p ms-cli` ONLY if it leaves mlock.rs untouched; otherwise hand-format changed files. (No CI fmt gate; clippy + g6 are the gates.)
- [ ] **Step 4: Commit** any test-fixups.

## Task 6: Sibling FOLLOWUP companion

- [ ] File `display-grouping-render-strip-v1` in mnemonic-secret's `FOLLOWUPS.md` (`git ls-files | grep -i followup`) with a `Companion:` line cross-citing the toolkit + md entries. Commit.

## Task 7: Version bump (release PREP)

**Files:** `crates/ms-cli/Cargo.toml` (0.7.0 → 0.8.0); CHANGELOG if present; `Cargo.lock`.

- [ ] **Step 1:** Bump `ms-cli` `0.7.0 → 0.8.0` (MINOR — default-output change). **ms-codec UNCHANGED (0.4.4); the `=0.4.4` pin stays.** Update CHANGELOG (`git ls-files | grep -i changelog`). `cargo build` to refresh `Cargo.lock`.
- [ ] **Step 2:** `cargo test --workspace && cargo clippy --workspace --all-targets -- -D warnings` → green.
- [ ] **Step 3: Commit** the bump.
- [ ] **Step 4: RELEASE (autonomous — authorized).** ff-merge `feature/mstring-display-grouping` → `master`, push. `cargo publish -p ms-cli --dry-run` then (if ms-cli is a crates.io crate — verify via the dry-run + prior `ms-cli-v*` tags) `cargo publish -p ms-cli`. Tag `ms-cli-v0.8.0` on the release commit; push the tag. Verify CI green on master. (ms-codec NOT published — unchanged.)

---

## Self-Review (write-time)

**Spec coverage:** §3 algorithm → Task 1 + Task 2 vectors. §5 separator parser → Task 1 `parse_separator`. §6 print-once + json/repair invariants → Task 3 (text emit only; json untouched) + Task 4 (combine `--to ms1` unbroken). §8 fn placement → Task 1 (deviation: ms-cli-local, ratify). §9.1 emit (encode/split) → Task 3. §9.2 intake (decode/inspect/repair/encode-hex via `read_input`; combine `-`→stdin + strip) → Task 4. §10 doubling-heuristic decommission → Task 4. §12-Phase-1 canonical-test rewrites → Task 3. SemVer MINOR (ms-cli only) → Task 7.

**Open items for plan-R0:** (1) ratify the §8 deviation (fns ms-cli-local; conformance = bin-crate `#[cfg(test)]`). (2) confirm EMIT-first/INTAKE-second ordering keeps every commit green (esp. `encode_pipe_to_decode.rs` + the dedup-removal). (3) confirm the breaking-test enumeration is COMPLETE — sweep `tests/` for any exact-encode/split-stdout pin I missed (P1 lesson: the architect found 3 the plan's grep missed). (4) confirm `ms combine` correctly gets intake-only (no emit flags) per spec §9.1 scope. (5) confirm `gui_schema_emits_spec_v7_json.rs` is assertion-based (new flags don't break it) or needs an update. (6) `ms encode --separator bogus` exit code: ms maps clap parse errors to **64** (`main.rs:165-170`), not 2 — confirm the test asserts 64.
