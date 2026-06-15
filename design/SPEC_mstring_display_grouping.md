# SPEC — Standardized mstring display grouping across the constellation

**Status:** DRAFT (pre-R0). Brainstorm-approved by user 2026-06-15; pre-spec architect consult folded.
**Author cycle:** mstring-display-grouping v1
**Source SHAs at write time (grep-verified):** toolkit `origin/master` `8da9008`; `mnemonic-secret` `b616530`; `mnemonic-key` `21786dc`; `descriptor-mnemonic` `eb9f368`.
**Branch:** `feature/mstring-display-grouping` (toolkit).
**Affects:** all four constellation CLIs (`mnemonic` / `md` / `ms` / `mk`) + `mnemonic-gui` schema mirror + `docs/manual`.

> Mandatory gate: this SPEC MUST pass an opus architect **R0** review to **0 Critical / 0 Important** before ANY implementation begins (CLAUDE.md Conventions §1). The pre-spec consult below is advisory input, NOT the R0.

---

## 1. Motivation

The constellation emits the three m-format card strings (`ms1`, `mk1`, `md1`) with **divergent, non-configurable** display formatting:

- `ms1` / `mk1` (toolkit): 5-char groups, **space** separator, **wraps at 10 groups/line** — `mnemonic-toolkit/crates/mnemonic-toolkit/src/format.rs:10` (`chunk_5char`), `:32` (`chunk_mk1` → defers to `chunk_5char`).
- `md1` (toolkit): n-char groups, **hyphen** separator, **no wrap** — `format.rs:37` (`chunk_md1` → `md_codec::encode::render_codex32_grouped(s, 5)`), defined at `descriptor-mnemonic/crates/md-codec/src/encode.rs:98`.
- `ms encode` standalone: 5-char/space/wrap-10 — `mnemonic-secret/crates/ms-cli/src/format.rs::chunked`.
- `md encode` standalone: **unbroken, no grouping at all** — `descriptor-mnemonic/crates/md-cli/src/cmd/encode.rs:84` (`println!("{}", encode_md1_string(...))`).
- `mk encode` standalone: **unbroken, no grouping at all** — `mnemonic-key/crates/mk-cli/src/cmd/encode.rs` (`println!("{s}")`).

Result: the same logical artifact prints with two different separators (space vs hyphen), two wrap rules, and three groupings (5/space/wrap, 5/hyphen/no-wrap, none). Users cannot choose unbroken vs grouped, nor the separator/period. This SPEC standardizes a single, configurable **display-grouping** layer across all four CLIs.

## 2. Glossary (resolves a naming collision)

- **chunk / chunked** — RESERVED for the existing **wire-level** concept: an `mk1`/`md1`/`ms1` payload split across multiple physical engravable strings because the data exceeds one codex32 string (`mk-codec` `string_layer/header.rs`, `md-codec` `chunk.rs`). **Not** this feature.
- **group / grouping** — THIS feature: inserting a separator every N characters within a single string for human readability. All new symbols use "group", never "chunk".

The toolkit's existing display fns `chunk_5char` / `chunk_mk1` / `chunk_md1` are mis-named display-grouping fns; renaming them to one `render_grouped` is part of this work.

## 3. Canonical algorithm (single source of truth)

Two pure, ASCII-only, dependency-free functions, defined identically in every repo and pinned by shared conformance vectors (§8).

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
- Single line ALWAYS — **no newline wrapping** (the legacy 10-groups/line wrap is removed).
- `separator` is a single `char` drawn from the allowed set (§5).

### 3.2 `strip_display_separators(s) -> String`
```
return s.chars().filter(|c| !is_display_separator(c)).collect()
where is_display_separator(c) = c == ' ' || c == '-' || c == ','
```
- Strips ONLY the allowed separator set (§5). Other characters pass through unchanged, so a malformed card is NOT silently "cleaned" into validity.
- Idempotent: `strip(strip(s)) == strip(s)` (separators are outside the codex32 alphabet).

### 3.3 Edge cases (all enumerated in the conformance vectors, §8)
- empty string → `""` (both fns).
- `group_size >= len(s)` → input unchanged (no separator inserted).
- `group_size == 0` with a `separator` supplied → unbroken (separator ignored, no error/warning).
- consecutive separators on input (`"ab--cd"`) → both stripped (`"abcd"`); no guard.
- multibyte: codex32 alphabet, HRP, and the `1` separator are all ASCII; no m-format string contains multibyte chars — `chars()` and `bytes()` are equivalent. Implementations MAY use bytes; vectors are ASCII.
- `--group-size` type is `u16` (0..=65535); any value ≥ string length yields the input unchanged; no overflow in `i % group_size` because `i < len(s) ≤ ~few hundred`.

## 4. Separator-charset safety

bech32/codex32 data charset is `qpzry9x8gf2tvdw0s3jn54khce6mua7l` (lowercase; no `b/i/o/1`), HRPs are `ms`/`mk`/`md`, and the bech32 separator is `1`. The allowed display separators `{space, '-', ','}` are **all outside** that charset and outside `{ms,mk,md,1}`, so stripping them on intake is unambiguous and cannot corrupt a valid string.

## 5. Allowed separator set + value model

**Set (final):** `space` (DEFAULT), `-` (hyphen), `,` (comma). (`;` and `'` were considered and cut — YAGNI / shell-hostility.)

**CLI value parser** (identical in all four CLIs): accepts BOTH literal and keyword forms, case-sensitive keywords:
| keyword | literal | char |
|---|---|---|
| `space` | `" "` | `' '` |
| `hyphen` | `"-"` | `'-'` |
| `comma` | `","` | `','` |

- Implemented as a small `value_parser` fn (NOT clap `ValueEnum`, which mishandles a literal-space possible-value). Default `space`.
- Keyword forms exist to dodge shell-quoting and to be the GUI dropdown values.
- Docs/examples use keyword forms (`--separator hyphen`); scripting may use either.

## 6. CLI flag surface (identical on all four CLIs)

- `--group-size <u16>` — default `5`; `0` = unbroken.
- `--separator <space|hyphen|comma | " " | - | ,>` — default `space`.

**Output model — single, flag-controlled, print-once.** Each emit point prints the m-string in exactly ONE form (the legacy print-twice "unbroken line + grouped line" is removed). Default = grouped space/5.

**Invariants:**
- `--json` output ALWAYS carries the **canonical unbroken** string, regardless of `--group-size`/`--separator` (the flags affect text-mode stdout only).
- `verify-bundle`'s forensic `VerifyCheck.expected` / `.actual` strings (`format.rs:170-171`) ALWAYS carry the **unbroken canonical** form (they are diff data, not display).
- `--no-engraving-card` (toolkit) is orthogonal — it suppresses the stderr panel; the flags govern stdout.

## 7. Default-behavior change & SemVer

This changes default stdout for every affected command:
| String | Before | After (default) |
|---|---|---|
| `ms1` | space/5, wrap@10, printed twice | space/5, single line, printed once |
| `mk1` | space/5, wrap@10 (toolkit) / unbroken (`mk encode`) | space/5, single line |
| `md1` | hyphen/5 (toolkit) / unbroken (`md encode`) | space/5, single line |

A default-output change is a **behavioral break** for any consumer parsing text-mode stdout, but stdout was never a declared-stable wire format and `--json` is unaffected. Precedent (v0.48.0 NUMS flip, v0.49.0 format-add) ⇒ **MINOR** bump per crate: `ms-cli`, `mk-cli`, `md-cli`, `mnemonic-toolkit` (+ pin bumps).

## 8. Architecture — where the code lives + drift control

**Function placement:**
- `ms-cli`: `render_grouped` + `strip_display_separators` in `crates/ms-cli/src/format.rs`. Replaces `chunked`; the legacy 10-group wrap logic is deleted.
- `mk-cli`: same two fns in a `mk-cli` formatting module.
- `md-codec`: GENERALIZE the existing `render_codex32_grouped(s, group_size)` (`encode.rs:98`) to `render_grouped(s, group_size, separator)` (separator becomes a parameter; the hardcoded `'-'` at `encode.rs:105` is removed). `strip_display_separators` lives in `md-cli` (intake is a CLI concern). Keeping the renderer in the codec lib is the one exception — it already lives there and the toolkit consumes it; no churn benefit to moving it.
- `mnemonic-toolkit`: ONE local `render_grouped` in `format.rs`; DELETE `chunk_5char`, `chunk_mk1`, `chunk_md1`; the toolkit stops delegating md1 rendering to `md_codec` (display format is now a toolkit-local choice, byte-identity guaranteed by vectors). `strip_display_separators` local in the toolkit.

**Drift control — copy-with-checksum conformance vectors (no new crate):**
- Canonical `design/display-grouping-vectors.tsv` authored in **mnemonic-toolkit** (the integration hub). Columns: `input`, `group_size`, `separator`, `op` (`render`|`strip`), `expected`, `note`. Covers every §3.3 edge case + each separator + render/strip round-trips.
- Each sibling repo carries an **identical copy** of the file PLUS `display-grouping-vectors.tsv.sha256`.
- Every repo's CI runs: (a) `sha256sum -c display-grouping-vectors.tsv.sha256` (coupling pin — fails loudly if a copy drifts from canonical), and (b) a test that drives its `render_grouped`/`strip_display_separators` against every row.
- Changing the canonical file forces an explicit copy+checksum update PR in each sibling; the CI checksum failure makes divergence impossible to miss.

## 9. Scope — call-site inventory

> Line numbers are from the pre-spec consult at the §-header SHAs; each implementation PR re-greps and pins live numbers per CLAUDE.md.

### 9.1 Emit sites (apply `render_grouped` + add flags)
| Repo | Site | Notes |
|---|---|---|
| toolkit | `cmd/bundle.rs:978` (ms1), `:989`/`:1001` (mk1), `:1020` (md1) | the print-twice pair collapses to one |
| toolkit | `cmd/convert.rs` `--to ms1` / `--to mk1` arms | currently emit raw |
| toolkit | `cmd/ms_shares.rs` `run_split` / `run_combine --to ms1` | ms1 shares; round-trip preserved by §10 intake strip |
| toolkit | `cmd/repair.rs` `emit_repair_text` / `emit_indel_text` (corrected `ms1`/`mk1`/`md1`) | |
| ms-cli | `cmd/encode.rs:201`; `cmd/split.rs:159` | |
| mk-cli | `cmd/encode.rs` (`println!("{s}")`) | NEW: was unbroken → corrective alignment |
| md-cli | `cmd/encode.rs:84` (text) + `:81` (force-chunked path) | NEW: was unbroken → corrective alignment |

### 9.2 Intake sites (apply `strip_display_separators` before decode)
| Repo | Site | Notes |
|---|---|---|
| ms-cli | `parse.rs:97` `strip_whitespace` | EXTEND to strip `{space,-,,}`; see §10 |
| mk-cli | `cmd/mod.rs:84` `read_mk1_strings` (currently only `.trim()` @:93) | add strip; covers all 6 mk subcommands that call it |
| md-cli | `cmd/decode.rs`, `cmd/repair.rs` | no normalization today |
| toolkit | `slot_ms1.rs:42`; `cmd/verify_bundle.rs` (`--ms1/--mk1/--md1`); `cmd/repair.rs`; `cmd/convert.rs` `--from ms1/mk1`; `ms_shares` combine | |
| toolkit | `verify-bundle --bundle-json` (JSON) | EXCEPTION: JSON carries canonical unbroken; no strip needed |

## 10. Decommission the ms-cli doubling-detection heuristic

`ms-cli/src/parse.rs:97-100` (`strip_whitespace`) contains a doubling-dedup heuristic (`had_whitespace && exact-double → halve`) that exists to cope with `ms encode`'s print-twice stdout being piped into `ms decode -` (test `parse.rs:138`). Once emit is **print-once** (§6), that double-emission no longer occurs and the heuristic's trigger becomes unreachable. The heuristic is **removed** (it could otherwise mis-fire on a legitimately doubled-content input). The replacement `strip_display_separators` does plain filtering, no dedup. The `back_typed_chunked_form_decodes` test still passes (intake strips spaces).

## 11. Cross-repo lockstep (CLAUDE.md invariants)

- **GUI schema mirror** (BIGGEST risk): `--group-size` + `--separator` are new clap flags on every covered toolkit subcommand → `mnemonic-gui/src/schema/mnemonic.rs` must add them (+ `--separator` keyword dropdown) in a paired PR; run `schema_mirror` against the new toolkit binary before merge. The gate is lagging (fires on next pin bump) — paired-PR discipline is the leading control.
- **Manual mirror**: `docs/manual/src/40-cli-reference/` for all four CLIs. Because the flags are identical everywhere, document them once in a "common output-grouping flags" section with per-CLI cross-references to avoid 4-way drift. Run `docs/manual/tests/lint.sh` (bidirectional flag coverage).
- **Sibling FOLLOWUP companions**: file `display-grouping-render-strip-v1` in toolkit + each sibling `FOLLOWUPS.md` with cross-citing `Companion:` lines.
- **Examples.pdf**: regenerate (separators/print-once change every card's display).

## 12. Rollout order

1. **Phase 1 — codec CLIs (3 repos, MINOR each):** add `render_grouped`/`strip_display_separators`; add flags to every emit subcommand; add intake strip; (md-codec) generalize `render_codex32_grouped` separator; update unit tests + manual chapters; bump.
2. **Phase 2 — conformance vectors:** author canonical TSV in toolkit; copy + `.sha256` into each sibling; add CI checksum + driver tests in all four.
3. **Phase 3 — toolkit (MINOR):** pin-bump the three siblings; collapse `format.rs` to one `render_grouped`; add flags to `bundle`/`verify-bundle`/`repair`/`convert`/`ms-shares`; regenerate the 20 golden vectors in `tests/vectors/`; update manual for all four CLIs; regenerate `docs/Examples.pdf`.
4. **Phase 4 — GUI (paired PR):** update `schema/mnemonic.rs`; run `schema_mirror` against the new toolkit; pin the new toolkit MINOR.

Each repo cycle is independently R0-gated and TDD'd (tests before impl) per CLAUDE.md.

## 13. Testing strategy

- **Conformance vectors** (§8) — the cross-repo identity gate.
- **Per-CLI flag tests:** default = space/5 single-line; `--group-size 0` = unbroken; `--separator hyphen`/`comma` (+ literal forms); invalid separator rejected (exit 2); invalid/oversize group-size rejected.
- **Round-trip:** for each CLI, `encode --separator X | decode -` succeeds for X ∈ {default, hyphen, comma} and for `--group-size 0`; `ms split | ms combine` round-trips with grouped shares.
- **Invariants:** `--json` always unbroken; `verify-bundle` forensic strings unbroken.
- **Golden regen:** toolkit `tests/vectors/v0_1/*` + `v0_2` recaptured; ms-cli `encode_canonical_12_word.rs` (asserts print-twice `\n\n`) rewritten for print-once; ms-cli `format.rs` unit tests rewritten.
- **schema_mirror** green against the new toolkit binary.

## 14. Out of scope (YAGNI)

- Separators beyond `{space, -, ,}` (`;` and `'` cut).
- Free-form / multi-char separators.
- A new shared crate (copy-with-checksum chosen instead).
- **Engraving-card panel content:** the stderr card shows 4-hex `chunk_set_id` identifiers, NOT full m-strings (`format.rs:260` `engraving_card_unified`), so routing it through `render_grouped` is a no-op; the card is unchanged. (Originally proposed "panel follows the flags" — struck.)
- Newline wrapping / fixed-width engraving layout (single-line only; an optional `--wrap` is a possible future FOLLOWUP).

## 15. Open questions for R0

1. `ms split` round-trip: print-once grouped shares still pipe to `ms combine` (intake strips). Confirm no consumer depends on split's legacy bare-strings-first block beyond the pipe.
2. Exact default for `mk encode` / `md encode` now that they GAIN grouping (they were unbroken): default space/5 like the others (corrective alignment) — confirm acceptable as a MINOR default change for those two CLIs.
3. Whether `repair` corrected-output should honor the flags or always emit unbroken (it is a recovery aid; argue both ways).
