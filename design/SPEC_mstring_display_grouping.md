# SPEC — Standardized mstring display grouping across the constellation

**Status:** R0 GREEN (round 3, 0C/0I). Spec gate MET — ready for implementation planning.
**Author cycle:** mstring-display-grouping v1
**Source SHAs at write time (grep-verified):** toolkit `origin/master` `8da9008`; `mnemonic-secret` `b616530`; `mnemonic-key` `21786dc`; `descriptor-mnemonic` `eb9f368`.
**Branch:** `feature/mstring-display-grouping` (toolkit).
**Affects:** all four CLIs (`mnemonic`/`md`/`ms`/`mk`) + `mnemonic-gui` schema mirror + `docs/manual` + `docs/technical-manual`.
**R0 history:** round 1 NOT GREEN (3C/9I/8m) → round 2 NOT GREEN (1C/3I/3m) → round 3 **GREEN (0C/0I)**; all three persisted to `design/agent-reports/mstring-display-grouping-r0-round{1,2,3}-review.md`; all findings folded below.

> Mandatory gate: MUST pass opus architect **R0** to **0 Critical / 0 Important** before ANY implementation (CLAUDE.md §1).

---

## 1. Motivation

The constellation emits the three card strings with **divergent, non-configurable** display formatting:

- `ms1`/`mk1` (toolkit): 5-char groups, **space**, **wrap@10 groups/line** — `mnemonic-toolkit/crates/mnemonic-toolkit/src/format.rs:10` (`chunk_5char`), `:32` (`chunk_mk1`→`chunk_5char`).
- `md1` (toolkit): n-char groups, **hyphen**, **no wrap** — `format.rs:37` (`chunk_md1`→`md_codec::encode::render_codex32_grouped`, called `:38`), defined `descriptor-mnemonic/crates/md-codec/src/encode.rs:98` (hardcoded `'-'` at `:105`).
- `ms encode` standalone: 5/space/wrap-10 — `mnemonic-secret/crates/ms-cli/src/format.rs::chunked`; emitted via `emit_text` (`cmd/encode.rs` ~`:198`, bare at `:199`, grouped at `:201`).
- `ms split`: **two-part** — bare share per line THEN labeled grouped blocks — `ms-cli/src/cmd/split.rs::emit_text` (`:147`; bare loop `:152-154`, grouped `:159`).
- `md encode`: **unbroken, no grouping** — `descriptor-mnemonic/crates/md-cli/src/cmd/encode.rs:84` (text), `:81` (force-chunked).
- `mk encode`: **unbroken, no grouping** — `mnemonic-key/crates/mk-cli/src/cmd/encode.rs` (`println!("{s}")`).

Result: two separators (space vs hyphen), two wrap rules, three groupings (5/space/wrap, 5/hyphen/no-wrap, none). No user choice. This SPEC standardizes one configurable **display-grouping** layer across all four CLIs.

## 2. Glossary (resolves a naming collision)

- **chunk / chunked** — RESERVED for the existing **wire-level** split of a payload across multiple physical strings (`mk-codec` `string_layer/header.rs`, `md-codec` `chunk.rs`). NOT this feature.
- **group / grouping** — THIS feature: inserting a separator every N chars within one string. All new symbols use "group".

The toolkit's `chunk_5char`/`chunk_mk1`/`chunk_md1` are mis-named display fns; they collapse into one `render_grouped`. md-codec's public `render_codex32_grouped` is NOT renamed (it is documented public API, §11/I1) — instead md-codec **adds** `render_grouped` and keeps `render_codex32_grouped` as a thin back-compat wrapper.

## 3. Canonical algorithm (single source of truth)

Two pure, dependency-free fns, defined identically in every repo, pinned by conformance vectors (§8).

### 3.1 `render_grouped(s, group_size, separator) -> String`
```
if group_size == 0 { return s }          // unbroken; separator ignored
out = ""
for (i, ch) in s.chars().enumerate() {
    if i > 0 && i % group_size == 0 { out.push(separator) }
    out.push(ch)
}
return out
```
Single line ALWAYS — **no newline wrapping** (legacy wrap@10 removed). `separator` is one `char` from the output set (§5).

### 3.2 `strip_display_separators(s) -> String`
```
return s.chars().filter(|c| !is_display_separator(c)).collect()
where is_display_separator(c) = c.is_whitespace() || c == '-' || c == ','
```
- **Strips ALL Unicode whitespace PLUS `-` and `,`.** (Folds I4/I5: preserves today's full-whitespace tolerance — tabs, CR, LF, NBSP — and ADDS hyphen/comma. The output separator set `{space,-,,}` is a strict subset, so every emitted form re-ingests.)
- Strips ONLY those; every other char (including any codex32-alphabet char) passes through, so a malformed card is never silently "cleaned" into validity.
- Idempotent: `strip(strip(s)) == strip(s)` (separators are outside the codex32 alphabet, §4).

### 3.3 Edge cases (all in the conformance vectors, §8)
empty→`""`; `group_size ≥ len`→input unchanged; `group_size == 0` + separator→unbroken (separator ignored, no error); consecutive separators on input→all stripped, no guard; tabs/CRLF on input→stripped (I4); multibyte: codex32 alphabet/HRP/`1`-sep are ASCII, no m-string is multibyte — `chars()`≡`bytes()` (vectors are ASCII; equivalence stated so a future non-ASCII HRP can't silently break it, m4); `--group-size` is `u16` (0..=65535), any value ≥ len yields input unchanged, no `i % group_size` overflow (`i < len ≪ usize::MAX`).

## 4. Separator-charset safety

codex32/bech32 data charset is `qpzry9x8gf2tvdw0s3jn54khce6mua7l` (confirmed `descriptor-mnemonic/crates/md-cli/src/cmd/repair.rs:34`); HRPs `ms`/`mk`/`md`; bech32 separator `1`. None of `{space,-,,}` (nor any whitespace) appears in that set or in `{ms,mk,md,1}`, so stripping is unambiguous: it can neither corrupt a valid string nor mask a malformed one.

## 5. Allowed separator set + value model

**Output set (final):** `space` (DEFAULT), `-` (hyphen), `,` (comma). (`;`/`'` cut — YAGNI/shell-hostility.)

**CLI value parser** (identical in all four CLIs): a small `value_parser` fn (NOT clap `ValueEnum` — it mishandles a literal-space possible-value). Accepts BOTH literal and keyword forms; rejects anything else with a clap parse error (exit 2 — NO new `ToolkitError` variant needed; rejection is at the clap layer before command dispatch, folding I8):
| keyword | literal | char |
|---|---|---|
| `space` | `" "` | `' '` |
| `hyphen` | `"-"` | `'-'` |
| `comma` | `","` | `','` |
Default `space`. **GUI constraint (I7):** `mnemonic-gui`'s dropdown MUST emit the KEYWORD values (`space`/`hyphen`/`comma`), never a literal space, to avoid argv/whitespace ambiguity through the GUI→argv path. Docs/examples use keywords.

## 6. CLI flag surface (identical on all four CLIs)

- `--group-size <u16>` — default `5`; `0` = unbroken.
- `--separator <space|hyphen|comma | " "|-|,>` — default `space`.

**Output model — single, flag-controlled, print-once.** Each emit point prints the m-string in exactly ONE form (legacy print-twice removed). Default = grouped space/5.

**`ms split` / `ms-shares --split` (C1/C2 resolution):** print-once. **stdout** carries the N share strings, one per line, in the flag-controlled form (default grouped); all human labels ("share N of M", headers) move to **stderr** (mirrors the engraving-card panel). For `ms split | ms combine -` to round-trip, **`ms combine` GAINS `-`→stdin multiline share intake** (one share per line; parallel to mk-cli `read_mk1_strings`, `mk-cli/src/cmd/mod.rs:84`) — today `ms combine` takes positionals only with no `-` handling (`combine.rs:36-52`, R0-r2 C1), so this is a REQUIRED Phase-1 addition, not a no-op. The receiver's intake strip (§9.2) removes separators per line. (`ms decode` is single-secret and is NOT a share-combine target — there is no `ms split | ms decode` claim.) This eliminates the bare+grouped duplication that the now-removed doubling heuristic used to absorb (§10).

**`repair` (C/m8 resolution):** corrected output is ALWAYS emitted **unbroken** (canonical), regardless of `--group-size`/`--separator`. `repair` is a recovery precision tool; grouping would inject separators into a string the user is visually re-inspecting. `repair` does NOT take the grouping flags. (Removed from the flag-honoring emit list in §9.1.)

**Invariants:**
- `--json` ALWAYS carries the **canonical unbroken** string (flags affect text-mode stdout only).
- `verify-bundle` forensic `VerifyCheck.expected`/`.actual` (`format.rs:170-171`) ALWAYS unbroken canonical.
- `--no-engraving-card` (toolkit) is orthogonal (governs the stderr panel).

## 7. Default-behavior change & SemVer

| String | Before | After (default) |
|---|---|---|
| `ms1` | space/5, wrap@10, printed twice | space/5, single line, once |
| `mk1` | space/5 wrap@10 (toolkit) / unbroken (`mk encode`) | space/5, single line |
| `md1` | hyphen/5 (toolkit) / unbroken (`md encode`) | space/5, single line |
| `ms split` | bare-per-line + labeled grouped blocks (stdout) | shares one-per-line grouped (stdout); labels→stderr |

Default-output change ⇒ **MINOR** per crate (`ms-cli`, `mk-cli`, `md-cli`, `mnemonic-toolkit` + pin bumps). stdout text was never a declared-stable interface and `--json` is unaffected (precedent: v0.48.0, v0.49.0). **Note (m7):** `ms split`'s removal of the bare-strings-first block is a stdout-format change that can break scripts parsing that block — still MINOR, but called out.

## 8. Architecture — code placement + drift control

**Function placement:**
- `ms-cli`: `render_grouped` + `strip_display_separators` in `crates/ms-cli/src/format.rs`; `chunked` and its wrap logic deleted; `parse.rs::strip_whitespace` becomes (or is replaced by) `strip_display_separators` with the §3.2 definition (KEEPS full-whitespace stripping, ADDS `-`/`,`).
- `mk-cli`: the two fns in a `mk-cli` formatting module.
- `md-codec`: **ADD** `pub fn render_grouped(s, group_size, separator)` to `encode.rs`; **keep** `render_codex32_grouped(s, group_size)` as a thin wrapper delegating with `'-'` (preserves the documented public API + the toolkit's pinned surface, folding I1/I6 — no rename, no atomic cross-phase break). `strip_display_separators` lives in `md-cli`.
- `mnemonic-toolkit`: ONE local `render_grouped` in `format.rs`; DELETE `chunk_5char`/`chunk_mk1`/`chunk_md1`; the toolkit no longer calls `md_codec::...render_codex32_grouped` for display (byte-identity guaranteed by vectors). `strip_display_separators` local.

**Drift control — copy-with-checksum conformance vectors (no new crate):**
- Canonical `design/display-grouping-vectors.tsv` authored in **mnemonic-toolkit**. Columns: `op` (`render`|`strip`), `input`, `group_size`, `separator`, `expected`, `note`.
- **TSV encoding convention (I2):** the `separator` column holds a KEYWORD, never a literal — `space`|`hyphen`|`comma`, or `none` for `op=render group_size=0` and for `op=strip` (separator inapplicable). For `op=strip` rows the `group_size` column is `0` (ignored by strip; R0-r2 m2). Empty-string `input`/`expected` is the literal sentinel `<empty>`. No raw spaces ever appear in a TSV field, so a plain tab-split parser is unambiguous. The driver maps keyword→char.
- Each sibling repo carries an identical copy + `display-grouping-vectors.tsv.sha256`. Every CI runs `sha256sum -c` (coupling pin) AND a driver test over every row.
- **Lagging-indicator caveat (m6):** the checksum only fires once a sibling has copied the file; a canonical change does not auto-break siblings until copied. The leading control is the paired-PR discipline (§11); the SPEC states this gap explicitly. (A future cross-repo CI probe is a possible FOLLOWUP.)

## 9. Scope — call-site inventory

> Lines from grep-verification at the §-header SHAs (citations corrected per m1/m2/I3). Each impl PR re-greps live numbers.

### 9.1 Emit sites (apply `render_grouped` + add flags; repair excluded — emits unbroken)
| Repo | Site | Notes |
|---|---|---|
| toolkit | `cmd/bundle.rs:978` (ms1), `:989`/`:1001` (mk1), `:1020` (md1) | print-twice → once |
| toolkit | `cmd/convert.rs` `--to ms1` / `--to mk1` arms | raw today |
| toolkit | `cmd/ms_shares.rs` `run_split` (`:296-310`) — ALREADY one-per-line + advisory on stderr; change is purely ADDITIVE (wrap stdout shares with `render_grouped`), not a restructure (R0-r2 I3) | |
| toolkit | `cmd/ms_shares.rs` `run_combine --to ms1` | apply `render_grouped` to the ms1 output (R0-r2 I3) |
| ms-cli | `cmd/encode.rs` `emit_text` (~`:198`; bare `:199` + grouped `:201` → one) | m1 |
| ms-cli | `cmd/split.rs` `emit_text` (`:147`; bare loop `:152-154` + grouped `:159`) → shares one-per-line grouped on stdout, labels→stderr | I3/m2; C1/C2 |
| mk-cli | `cmd/encode.rs` (`println!("{s}")`) | NEW grouping (was unbroken) — corrective |
| md-cli | `cmd/encode.rs:84` (text) + `:81` (force-chunked) | NEW grouping (was unbroken) — corrective |

### 9.2 Intake sites (apply `strip_display_separators` before decode)
| Repo | Site | Notes |
|---|---|---|
| ms-cli | `parse.rs:97` (`strip_whitespace`→`strip_display_separators`, §3.2 def) — used by `read_input` | covers `cmd/decode.rs:42` (I4), `cmd/encode.rs --hex` |
| ms-cli | `cmd/combine.rs:36-52` — (a) ADD `-`→stdin multiline share intake (one share/line; parallel to mk `read_mk1_strings`) — ABSENT today (R0-r2 C1); (b) strip each positional/stdin share | **C1+C3** — enables `ms split \| ms combine -` AND grouped-steel positional recovery |
| mk-cli | `cmd/mod.rs:84` `read_mk1_strings` (was `.trim()` only `:93`) → add interior strip | I5; covers all 6 mk subcommands |
| md-cli | `cmd/decode.rs`, `cmd/repair.rs` | no normalization today |
| toolkit | `slot_ms1.rs:42`; `cmd/verify_bundle.rs` (`--ms1/--mk1/--md1`); `cmd/repair.rs`; `cmd/convert.rs` `--from ms1/mk1`; `cmd/ms_shares.rs` combine | |
| toolkit | `verify-bundle --bundle-json` (JSON) | EXCEPTION: JSON is canonical unbroken; no strip |

## 10. Decommission the ms-cli doubling-detection heuristic

`ms-cli/src/parse.rs:97-100` (`strip_whitespace`) has a doubling-dedup heuristic (`had_whitespace && exact-double → halve`); its tests are `strip_whitespace_dedupes_doubled_content` and `strip_whitespace_handles_all_three_workflows` (`:122`,`:138`) (m3 — corrected names). It exists to absorb `ms encode`/`ms split` print-twice stdout piped into decode/combine. Once emit is **print-once** everywhere (§6, incl. `ms split` shares one-per-line with labels on stderr), the double-emission no longer occurs and the trigger is unreachable. The heuristic is **removed**; `strip_display_separators` does plain filtering (no dedup). New/updated tests prove `ms encode | ms decode -` and `ms split | ms combine -` (the latter via the new `-`→stdin intake, C1) round-trip for default, `hyphen`, `comma`, and `--group-size 0`, plus `ms combine <grouped positional shares>` (replacing the dedup tests).

## 11. Cross-repo lockstep (CLAUDE.md invariants)

- **GUI schema mirror** (biggest risk): `--group-size`/`--separator` are new clap flags on every covered toolkit subcommand → `mnemonic-gui/src/schema/mnemonic.rs` adds them (+ a `--separator` KEYWORD dropdown per I7) in a paired PR; run `schema_mirror` against the new toolkit binary before merge. Lagging gate; paired-PR is the leading control.
- **End-user manual**: `docs/manual/src/40-cli-reference/` for all four CLIs — document the identical flags once in a "common output-grouping flags" section with per-CLI cross-references; run `docs/manual/tests/lint.sh` (bidirectional flag coverage).
- **Technical manual (I1):** `51-md-codec-api.md:~189` documents `render_codex32_grouped` — KEPT (wrapper, §8), so no break; ADD an entry for the new `render_grouped`. `54-mnemonic-toolkit-api.md:50-51` documents the toolkit's `chunk_5char` (`:50`) and `chunk_md1` (`:51`) — both DELETED (§8), so REMOVE those two rows in Phase 3 and add the toolkit's local `render_grouped` (`chunk_mk1` has no manual row — nothing to remove for it; R0-r2 I1 / R0-r3 m1/m3). Run the technical-manual lint gate.
- **Sibling FOLLOWUP companions**: file `display-grouping-render-strip-v1` in toolkit + each sibling `FOLLOWUPS.md` with cross-citing `Companion:` lines.
- **Examples.pdf**: regenerate (separators/print-once change every card).

## 12. Rollout order

1. **Phase 1 — codec CLIs (3 repos, MINOR each):** add `render_grouped`/`strip_display_separators`; add flags to every emit subcommand; add intake strip (incl. ms `combine` — **also ADD `-`→stdin multiline share intake, C1**; mk `read_mk1_strings`; md decode/repair); `ms split` print-once (labels→stderr); md-codec ADDS `render_grouped` + keeps wrapper; remove the doubling-dedup heuristic + its tests (§10); rewrite the print-twice test `encode_canonical_12_word.rs` (asserts `\n\n` → goes RED) and the `format.rs` grouping unit tests; refresh `encode_canonical_24_word.rs` for positive single-line coverage (it does not assert `\n\n`, so it won't go RED — R0-r2 I2 / R0-r3 m2); update end-user manual chapters; bump.
2. **Phase 2 — conformance vectors:** author canonical TSV (keyword/`<empty>` encoding) in toolkit; copy + `.sha256` into each sibling; add CI checksum + driver tests in all four.
3. **Phase 3 — toolkit (MINOR):** pin-bump siblings; collapse `format.rs` to one `render_grouped` (delete `chunk_5char`/`chunk_mk1`/`chunk_md1`); add flags to `bundle`/`verify-bundle`/`convert`/`ms-shares` (NOT `repair`); regenerate the 20 golden vectors (`tests/vectors/v0_1/*`+`v0_2`); re-examine fuzz-corpus seeds for embedded formatted strings (the `cli_cross_tool_differential.rs` harness compares LIVE binaries on format-independent IDs and has NO disk artifacts to regen — R0-r2 m1); update both manuals for all four CLIs (incl. REMOVING the dead `chunk_*` rows from `54-mnemonic-toolkit-api.md`, §11); regenerate `docs/Examples.pdf`.
4. **Phase 4 — GUI (paired PR):** update `schema/mnemonic.rs` (flags + keyword dropdown); run `schema_mirror`; pin the new toolkit MINOR.

Each repo cycle is independently R0-gated + TDD'd.

## 13. Testing strategy

- **Conformance vectors** (§8) — cross-repo identity gate; include empty, group_size≥len, group_size 0+sep, consecutive seps, **tab/CRLF strip**, idempotent strip, each separator, render/strip round-trips.
- **Per-CLI flag tests:** default space/5 single-line; `--group-size 0` unbroken; `--separator hyphen|comma` (+ literals + keywords); invalid separator/oversize group-size → clap exit 2.
- **Round-trip:** `encode --separator X | decode -` for X∈{default,hyphen,comma} and `--group-size 0`; **`ms split | ms combine -`** via the new `-`→stdin intake (C1 guard); `ms combine <grouped-positional-share>` decodes (C3 guard).
- **Invariants:** `--json` unbroken; `verify-bundle` forensic strings unbroken; `repair` output unbroken.
- **Golden/artifact regen:** toolkit `tests/vectors/*` recaptured; ms-cli `encode_canonical_12_word.rs` (asserts print-twice `\n\n`) rewritten for print-once + `encode_canonical_24_word.rs` refreshed for single-line coverage; ms-cli `format.rs` unit tests rewritten; fuzz-corpus seeds re-examined (the differential harness needs no regen — format-independent IDs, no disk artifacts, R0-r2 m1).
- **schema_mirror** green vs the new toolkit binary.

## 14. Out of scope (YAGNI)

Separators beyond `{space,-,,}`; free-form/multi-char separators; a new shared crate; newline wrapping / fixed-width engraving layout (single-line only; optional `--wrap` = future FOLLOWUP). **Engraving-card panel content** is unchanged: the stderr card shows 4-hex `chunk_set_id` identifiers, NOT full m-strings (`format.rs:260` `engraving_card_unified`), so routing it through `render_grouped` is a no-op (the earlier "panel follows the flags" proposal is struck).

## 15. Resolved decisions (from R0 round 1)

1. **`ms split` round-trip (C1/C2):** print-once — shares one-per-line on stdout (flag-controlled); labels→stderr. **`ms combine` GAINS `-`→stdin multiline share intake** (parallel to mk `read_mk1_strings`) so `ms split | ms combine -` works; positional grouped shares strip too (C3). Round-trip preserved without the doubling heuristic. (`ms decode` is single-secret — not a share target.)
2. **`mk encode` / `md encode` gaining grouping:** YES — default space/5 like the others (corrective alignment of a pre-existing inconsistency); a MINOR default-output change for those two CLIs.
3. **`repair` output:** ALWAYS unbroken canonical; `repair` does not take the grouping flags (m8).
