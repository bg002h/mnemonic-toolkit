# R0 round-1 architect review — PLAN_secret_flag_gate_non_circular (2026-06-10)

Reviewer: Fable 5 architect agent (R0, pre-implementation). Plan @ design/PLAN_secret_flag_gate_non_circular.md, source e2a09ba (v0.53.0). Verdict: RED (0 Critical / 3 Important). Review verbatim below.

---

## Critical (must-fix before code)

None.

## Important (must-fix before code)

**I-1 — The TDD RED matrix is wrong: Cell 2 will NOT be RED on `--phrase-stdin` pre-D1.**
Plan line 35 says "Cell 2 on `--phrase-stdin`/`--ms1-stdin`" and Phase 1 (line 64) gates on "expect exactly Cells 1-3 RED on the documented names." Cell 2's own semantics (line 30: toggle `secret` must **equal** the base flag's `secret`) make `--phrase-stdin` GREEN at e2a09ba: I ran the live v0.53.0 `gui-schema` and for all 3 xpub-search modes `--phrase-stdin` toggle_secret=absent(false) and base `--phrase` secret=absent(false) — equal, passes. Only `--ms1-stdin` mismatches (base `--ms1` secret=true, toggle false, ×3 subcommands). Pre-D1, `--phrase-stdin` is caught **only by Cell 3** (14-literal vs live 11); it becomes Cell-2-protected only after D1 flips `--phrase`. An implementer following the plan's RED expectation will see an "unexpectedly green" Cell 2 and may distort the cell to force it red. **Fix:** rewrite the RED matrix — Cell 1 RED on `--phrase` ×3; Cell 2 RED on `--ms1-stdin` ×3 only; Cell 3 RED on the 3-name set difference; note explicitly that `--phrase-stdin`'s pre-D1 coverage is Cell 3 (Cell 2 covers it transitively post-flip).

**I-2 — "all 6 today's stdin toggles" is wrong: there are 8 distinct `--X-stdin` booleans (26 instances), including two the plan never mentions.**
Live enumeration: `--passphrase-stdin`, `--bip38-passphrase-stdin`, `--decrypt-password-stdin`, `--secret-stdin`, `--ms1-stdin`, `--phrase-stdin` **plus `--message-stdin` (verify-message, base `--message`) and `--xpub-stdin` (xpub-search-address-of-xpub, base `--xpub`)**. The plan's existence conclusion happens to survive — all 8 have a same-subcommand base flag, so Cell 2 won't RED incorrectly — but the count claim presented as verified is false, and Cell 2's domain silently includes two non-secret toggle/base pairs (false==false, passes; arguably desirable extra coverage). **Fix:** correct the plan to 8 distinct names / 26 instances, state that `--message-stdin`/`--xpub-stdin` are in Cell 2's domain with non-secret bases by design, and have the implementer's Cell 2 failure message print the (subcommand, toggle, base) triple so future non-secret pairs are legible.

**I-3 — The 3 new Route rows' evidence anchors are vacuous: `"phrase_stdin"`/`"phrase-stdin"` are substrings of `"passphrase_stdin"`/`"passphrase-stdin"`.**
The evidence check is plain `source.contains(needle)` (`tests/lint_argv_secret_flags.rs:240`), and all 3 cited files already contain `passphrase_stdin`/`passphrase-stdin` for the **existing** `--passphrase` routes (e.g. `path_of_xpub.rs:64,69,253`). `"passphrase_stdin"` ends with `"phrase_stdin"`, so the proposed anchors are satisfied even if the `--phrase-stdin` wiring were deleted — zero discriminating power, defeating the evidence cell's stated purpose (module doc :16-18: "proving the `*-stdin` route is actually WIRED in source, not merely named"). The anchors do *happen* to exist standalone today (`pub phrase_stdin: bool` at `path_of_xpub.rs:45`, `passphrase_of_xpub.rs:63`, `account_of_descriptor.rs:46`; `fn phrase_stdin` at :124/:155/:136), so the rows aren't lying — they're just inert. **Fix:** use discriminating needles present in all 3 files, e.g. `&["pub phrase_stdin", "fn phrase_stdin", "[\"phrase_stdin\""]` (none of these is a substring of any `passphrase` token; verified present at the lines above).

## Minor (may defer)

**M-1 — GUI companion should cross-cite the two EXISTING open GUI FOLLOWUPs, and the "Unblocks I4" claim is partial.**
The GUI repo already carries open entries `xpub-search-inline-phrase-not-secret-classified` (`mnemonic-gui/FOLLOWUPS.md:73`) and `ms-repair-ms1-not-secret-classified` (`:81`) — the audit's "FOLLOWUPs already filed and OPEN" (I4). D4's plan to file a fresh `gui-secret-mirror-phrase-ms1-stdin` GUI-side line without binding to those entries would fragment the record; add `Companion:` lines to the existing entries instead of (or in addition to) a third. Also: I4's `src/schema/ms.rs:321` half (`ms repair --ms1`) mirrors the **ms-cli** surface, not the toolkit's gui-schema — the toolkit's own `repair --ms1` is already `secret:true` (verified live) — so this cycle unblocks only the mnemonic.rs half; say so in the companion.

**M-2 — Companion site count: the GUI flip is 9 hand-coded `secret:` sites, not 3.** Audit I4's `:2286/2448/2718` are the three `--phrase` entries only (verified — all three are the `--phrase` help text); `--phrase-stdin` and `--ms1-stdin` entries in the same three subcommand tables also need flips. Enumerate 3 flags × 3 subcommands in the companion entry so the GUI cycle doesn't under-scope.

**M-3 — Cell 1 net non-vacuity note:** `priv` currently matches nothing in the included kinds (`--privacy-preserving` is boolean-excluded), i.e. it's speculative vocabulary. Fine to keep; worth a one-line comment in the cell so a future reader doesn't assume it's load-bearing today.

**M-4 — D1's "a no-leading-dashes row stays non-secret" already exists** (`src/secrets.rs:120` `flag_is_secret("passphrase")`); phrase the plan as "keep/extend" not "add".

## Verified-clean (for the record)

- Tautology claim **correct**: `gui_schema.rs:1196` (`let secret = mnemonic_toolkit::secrets::flag_is_secret(&name);` in `emit_flag`) vs the :284 cell's `expected_secret = …flag_is_secret(name)` (`cli_gui_schema_v5_extensions.rs:298`). Keeping :284 as a renamed plumbing check is sound; the fn name is referenced nowhere else (zero hits in `docs/technical-manual/src/`, so no symbol-pin G2 breakage on rename).
- Cell 1 sim against the FULL live surface (30 subcommands; kinds text 102 / dropdown 54 / boolean 88 / number 44 / path 20): **exactly 3 REDs**, all `--phrase` (text) on the 3 xpub-search modes; post-D1 GREEN with EXEMPT empty — every other net match is correctly kind-excluded (`--shares` number, `--*-file`/`--passphrase-candidates-file` path, `--privacy-preserving`/`--reveal-secret` boolean) or already secret. The plan's net is correctly stronger than the audit's (audit's regex at `constellation-architecture-audit-2026-06-10.md` I3 lacks bare `phrase` and `ms1` and would have missed all 3 names).
- Frozen literal: live distinct `secret:true` names = exactly 11 (independently counted); 11 + 3 = **14** ✓.
- Blast radius: `flag_is_secret` has exactly one non-test consumer (`gui_schema.rs:1196`); swept all of `tests/` — no test pins any of the 3 names as non-secret (env-sentinel/argv-leakage tests assert runtime stderr, unaffected; `non_secret_flags_omit_secret_field` pins only `--account`/`--mk1`/`--md1`; no full-schema goldens exist). "Only lint axis-1 REDs after D1" **confirmed**.
- Lint rows: subcommand names `xpub-search-path-of-xpub` etc. match both the live schema and existing rows (`lint_argv_secret_flags.rs:91-96`); only the 3 xpub-search modes carry `--phrase`, so exactly 3 rows restore set-equality. Clap field cites `path_of_xpub.rs:38` / `passphrase_of_xpub.rs:56` / `account_of_descriptor.rs:39` all accurate.
- Ritual surface: FOLLOWUPS backlog lines exist (`design/FOLLOWUPS.md:15,:28`), `readme_version_current.rs` and `.github/workflows/changelog-check.yml` exist, `run_gui_schema` uses `cargo_bin` (no stale-binary trap), manual untouched correctly (no name/help change), SemVer PATCH call is defensible with the cited precedents and the advisor checkpoint is retained.

## Verdict

**RED** — 0 Critical / 3 Important. All three are plan-doc corrections (RED-matrix attribution, stdin-toggle census 6→8, discriminating evidence anchors); none undermines the design itself. Fold and re-dispatch — this should converge GREEN in one round.
