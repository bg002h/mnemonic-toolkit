# cycle-prep recon — 2026-06-06 — quick-wins batch (manual-repair-mutex + import-ms1-argv-advisory + electrum-refusal-wording)

**Origin/master SHA at recon time:** `2d6c940`
**Local branch:** `master` · **Sync state:** `up-to-date (0/0)` · **Untracked:** recon/survey scratch + `.claude/` (none load-bearing).

Slug(s) verified: 3. **All three are real, small, low-risk PATCH fixes; two are SHARPENED beyond their FOLLOWUP** — slug 1's "fix-the-class" extends to the `inspect` table (also wrong), and slug 3 is newly *timely*: since v0.47.0 shipped `addresses --from electrum-phrase`, the convert refusal should REDIRECT there, not just "honest about unimplemented." Line numbers drifted (manual grew); no structural errors.

---

## Per-slug verification

### `manual-repair-flag-mutex-inaccuracy`  — manual prose; FIX-THE-CLASS extends to inspect
- **WHAT:** The `repair` flag table says `--ms1/--mk1/--md1` are "mutually exclusive," but source allows combining them (multi-group, one HRP per card, per D35).
- **Citations:**
  - repair rows "mutually exclusive" — **DRIFTED** `:2277-2279` → actual **`:2751-2753`** (all three rows: `--ms1` "mutually exclusive with --mk1 / --md1", `--mk1`, `--md1`).
  - source `repair.rs:39,46,52` "May be combined per D35" — **ACCURATE (drift ~1)**: `:38-39`, `:45`, `:51` ("May be combined with --mk1 / --md1 … per D35"); the cross-HRP `conflicts_with_all` was dropped v0.24.0 (`:25`).
  - proving test `multi_group_both_emit_exit_5` — **ACCURATE** at `cli_indel.rs:239`.
  - **(FIX-THE-CLASS) inspect ALSO wrong:** the `inspect` table at **`:3023`** (+ adjacent `--mk1`/`--md1` rows) says "mutually exclusive with --mk1 / --md1" — but `inspect.rs:24-28` DROPPED `conflicts_with_all` ("`mnemonic inspect ms1xxx mk1yyy md1zzz` are valid") + doc-comment `:34` "May be combined per D35". So BOTH the repair AND inspect tables are inaccurate (the FOLLOWUP suspected this; source confirms inspect allows combining).
- **Action for brainstorm spec:** Reword the 3 repair rows (`:2751-2753`) + the 3 inspect rows (`:3023`+adjacent) from "mutually exclusive with" → "may be combined with --mk1 / --md1 (one HRP per card; per D35)", mirroring the source doc-comments. Pure manual prose — no code/flag change; `make audit` flag-coverage gates flag NAMES not descriptions, so nothing CI-blocks the wording (the manual IS the artifact). `make audit` GREEN after. Cite SHA `2d6c940`.

### `import-wallet-ms1-argv-advisory-gap`  — small additive runtime advisory + test
- **WHAT:** `import-wallet --ms1` is secret-bearing on argv but fires NO `secret_in_argv_warning` (unlike sibling secret flags).
- **Citations:**
  - `--ms1` intake `import_wallet.rs` — **ACCURATE** (`:171-172` `#[arg(long="ms1")] pub ms1: Vec<String>`; `@env:VAR` channel `:168-169`).
  - `secret_in_argv_warning` imported (`:80`) + used for `--decrypt-password` (`:472`) but NOT `--ms1` — **ACCURATE** (the gap). The advisory-placement model is the `--decrypt-password` call at `:472`.
  - Non-argv channel: **`@env:VAR` only** (no `import-wallet --ms1-stdin`; the `-`/stdin form is repair/inspect's `--ms1`, not import-wallet's). So the advisory's alternative-channel text should be `@env:VAR` (NOT "--ms1-stdin" as the FOLLOWUP loosely wrote). `secret_in_argv_warning(stderr, flag_label, alternative)` signature confirmed by the `:472` call `("--decrypt-password ", "--decrypt-password-stdin")`.
  - Not a missing-route leak (`lint_argv_secret_flags` anchors it on `@env:` already — passes); this is the runtime ADVISORY for consistency.
- **Action for brainstorm spec:** Fire `secret_in_argv_warning(stderr, "--ms1", "@env:VAR")` when an `--ms1` value is supplied inline (skip `@env:` values + the `""` watch-only sentinel), mirroring the `--decrypt-password` placement. **Changes runtime behavior (new stderr line) → needs a test cell** (assert the advisory fires for inline `--ms1`, NOT for `@env:`/empty). Confirm whether an existing `--slot @N.phrase` secret advisory cell exists to mirror (the `cli_import_wallet*` comment at `:1308` references "secret-bearing flag surfaces (--ms1, secret-bearing --slot)"). No clap/flag change → no GUI schema_mirror. Optional one-line manual note. Cite SHA `2d6c940`.

### `electrum-phrase-address-refusal-honest-wording`  — SHARPENED: redirect to the now-shipped `addresses --from electrum-phrase`
- **WHAT:** The `convert (electrum-phrase → address)` refusal uses the shared one-way-barrier message ("cryptographically unrecoverable … one-way derivation barrier") — imprecise: it's not unrecoverable, and as of **v0.47.0 it's SUPPORTED via `mnemonic addresses --from electrum-phrase`**.
- **Citations:**
  - shared `refusal_one_way` — **DRIFTED** `:460` → actual **`:458-466`** ("cryptographically unrecoverable from … one-way derivation barrier").
  - routing: `classify_edge` (`:649`) returns `refusal_one_way(from, to)` at `:696` for `(ElectrumPhrase, Address)` (the catch-all one-way fallback) — **ACCURATE** (no dedicated electrum→address arm today).
  - existing electrum refusal-helper pattern: `refusal_electrum_2fa_unsupported` (`:547`), `refusal_electrum_phrase_pivot` (`:553`), `refusal_electrum_invalid_format` (`:559`) — **ACCURATE** (the dedicated-arm precedent to mirror).
- **Action for brainstorm spec:** Add a dedicated `(ElectrumPhrase, Address)` arm in `classify_edge` BEFORE the `:696` one-way fallback, returning a new `refusal_electrum_phrase_to_address()` helper whose message **redirects**: e.g. *"convert does not derive addresses from an Electrum native seed; use `mnemonic addresses --from electrum-phrase=<seed> --address-type <p2pkh|p2wpkh>` (Electrum uses its own PBKDF2 salt + non-BIP-44 derivation)."* This is strictly better than the FOLLOWUP's original "honest unimplemented wording" — it points to the working path. **Changes the refusal message → needs a test cell** (assert exit + the new substring; confirm no existing test pins the old one-way message FOR this edge). No clap change → no GUI schema_mirror. Cite SHA `2d6c940` + the v0.47.0 addresses feature.

---

## Cross-cutting observations
1. **Two slugs sharpened by recon:** slug 1 is a fix-the-CLASS (inspect table also wrong — source-confirmed inspect allows combining), slug 3 is newly *timely* (redirect to the v0.47.0 `addresses --from electrum-phrase` rather than a vague "unimplemented" message). Both are better fixes than their FOLLOWUPs literally state.
2. **Line drift only** (manual grew + this session's electrum manual edit shifted the inspect/addresses region); no structural citation errors.
3. **No GUI schema_mirror for any** (no flag/value/subcommand change). slug 1 = manual prose; slug 2 = additive stderr advisory on an existing flag; slug 3 = reworded refusal on an existing edge.
4. Slugs 2 & 3 each change runtime output (a new stderr advisory / a reworded refusal) → each needs a Phase-1 RED cell. Slug 1 is manual-only (no RED cell; `make audit` is the guard).

---

## Recommended brainstorm-session scope
**ONE coherent quick-wins PATCH cycle** (3 small, independent fixes; ≤3 items = single R0 round per the smaller-scope heuristic). **SemVer: PATCH** (slug 1 docs; slug 2 additive advisory; slug 3 reworded refusal — all non-breaking). v0.47.1 → **v0.47.2**. **Size: small** — slug 1 ~6 manual rows; slug 2 ~5 LOC + 1 test cell; slug 3 ~1 helper + 1 arm + 1 test cell. **Locksteps: manual mirror only** (slug 1 IS the manual; slugs 2/3 optionally add a one-line manual note) — **no GUI schema_mirror, no sibling-codec change**. **Phasing:** Phase 1 RED = slug-2 advisory cell + slug-3 redirect-message cell (RED against current: no advisory / old one-way message); slug 1 has no RED (manual prose, `make audit` guard). Per-phase opus review. R0 MUST confirm: (i) inspect genuinely allows combining (so fixing its table is correct, not introducing a falsehood); (ii) the exact non-argv channel wording for slug 2 (`@env:VAR`, no import-wallet `--ms1-stdin`); (iii) no test pins the old electrum→address one-way message. Independent of all other open cycles.
