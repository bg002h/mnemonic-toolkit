# R0 Architect Review (round 1) — `SPEC_quick_wins_v0_47_2.md`

**Reviewer:** opus `feature-dev:code-reviewer` (mandatory pre-implementation R0 gate). **Date:** 2026-06-06.
**Branch:** `quick-wins-v0.47.2` (off master `2d6c940`). **Verdict:** **0 Critical / 4 Important** (+ 4 Minor). NOT GREEN.

> Persisted verbatim per CLAUDE.md BEFORE the fold. The two load-bearing claims PASS (inspect genuinely allows combining → slug-1 reword is TRUE + already test-backed; slug-3 redirect is the real shipped surface). 4 Importants = 2 fix-the-class expansions (I1, I4) + 1 placement correctness (I2) + 1 off-by-N (I3). Fold → re-dispatch.

---

## VERDICT: 0 Critical / 4 Important (+ 4 Minor) — NOT GREEN

### IMPORTANT

**I1 — Slug 2 fixes `--ms1` but ignores its sibling `--slot @N.phrase` on the SAME command (fix-the-instance, not the class).**
`import-wallet` has a SECOND inline-secret-on-argv surface: `--slot @N.phrase=<inline>` (`import_wallet.rs:174-182`, processed `:1048-1055`/`:1180`), `@env:`-only (no stdin — `lint_argv_secret_flags.rs:127`), doc-comment "Equivalent to `--ms1`." `secret_in_argv_warning` fires ONLY for `--decrypt-password` (`:472`/`:2173`), NOT `--ms1` OR `--slot @N.phrase`. Fixing `--ms1` while leaving its twin uncovered defeats the SPEC's "consistency with sibling secret flags" rationale.
**Fix:** Extend slug 2 to also fire for inline `--slot @N.phrase` (skip `@env:`/empty), own RED/GREEN cell — OR explicitly scope out + FOLLOWUP. Don't silently leave it.

**I2 — Slug 2 advisory MUST read raw `args.ms1` BEFORE the `:284` env-sentinel rebind.**
At `import_wallet.rs:282-289`, `args` is rebound to `env_resolved_owned` with `@env:VAR` ALREADY resolved to secret values. If the "skip if `@env:`-prefixed" check runs post-rebind, a legitimate `@env:VAR` user gets a FALSE advisory (the negative cell fails). SPEC's "near `:282`" is ambiguous.
**Fix:** Place at top of `run` (after `:271`, before `:284`) iterating the ORIGINAL `args`, also before the early-return validation `:293`.

**I3 — Slug 1 inspect table: only ONE row carries the false claim, not three (off-by-N).**
Only `:3023` (the inspect `--ms1` row) says "mutually exclusive with --mk1 / --md1." The `--mk1` (`:3024`) + `--md1` (`:3025`) inspect rows say only "use `-` for stdin" — nothing to reword. (The repair table `:2751-2753` genuinely has all three wrong.)
**Fix:** SPEC slug-1 action → "reword the 1 inspect row `:3023` + the 3 repair rows `:2751-2753`."

**I4 — Slug 1 leaves the repair/inspect SYNOPSIS brace-pipe `{--ms1 | --mk1 | --md1}` uncorrected — same false mutex, more prominent, violates the clap-mirror invariant.**
Synopses `:2744` (repair) + `:3016` (inspect) render `{--ms1 <MS1> | --mk1 <MK1> | --md1 <MD1>}` — brace-pipe = "pick exactly one," the identical false claim, directly above the table being fixed (self-contradictory). With `ArgGroup::new("kind").required(false).multiple(true)` + 3 independently-`#[arg(long)]` flags (`repair.rs:23-53`, `inspect.rs:22-48`), clap `--help` renders `[--ms1 …] [--mk1 …] [--md1 …]` (independently optional), so the hand-authored synopsis ALSO fails CLAUDE.md's "manual mirrors clap `--help`" invariant.
**Fix:** Fold the synopsis correction into slug 1 + the implementer verifies the exact USAGE line via `mnemonic repair --help` / `inspect --help` (architect-must-RUN discipline) before committing.

### MINOR
**M1 — Slug 3 stale test comment.** `cli_convert_electrum.rs:557-563` (above `electrum_phrase_to_address_is_refused` `:565`) says the toolkit "deliberately does NOT derive addresses … would produce WRONG addresses" — predates v0.47.0 (which shipped correct derivation in `addresses`). Test stays GREEN (asserts only `.failure()` + `contains("electrum-phrase")`), but slug 3 owns updating the comment. Scope-completion.
**M2 — Slug 2 flag label NO trailing space.** `--decrypt-password` uses `"--decrypt-password "` (trailing space → `(--decrypt-password )`). SPEC's `"--ms1"` (no space) matches its RED assertion `(--ms1)`. Pin no-trailing-space.
**M3 — "pipe via @env:VAR" slightly awkward** (env-sentinel, not a pipe) but reuses the uniform template; acceptable.
**M4 — redirect `<p2pkh|p2wpkh>` could read as free choice** (Electrum fixes it by seed version, `addresses.rs:244-255`); optional one-clause hint.

---

### WHAT VERIFIED CLEAN
- **Slug 1 — inspect REALLY allows combining (load-bearing).** `inspect.rs:22-31` drops `conflicts_with_all`, `ArgGroup….required(false).multiple(true)`, "`mnemonic inspect ms1xxx mk1yyy md1zzz` are valid"; PROVEN by `cli_positional_hrp_autodetect.rs:196 inspect_positional_mixed_hrp_d35_allows` + `:213`. Reword is TRUE, already test-backed (no new inspect test needed). Repair: `repair.rs:23-53` + `cli_indel.rs:239`.
- **Slug 2 — channel/signature/gap.** `import-wallet --ms1` has NO stdin/`-`; `@env:VAR` is the only non-argv route (`import_wallet.rs:168-172`; route table `lint_argv_secret_flags.rs:84`). `@env:VAR` correct (FOLLOWUP's "--ms1-stdin" wrong). Signature `secret_advisory.rs:40`; imported `:80`. Gap real. Advisory-only (lint route passes).
- **Slug 3 — routing/redirect/exit/no-pin (load-bearing).** `(ElectrumPhrase, Address)` absent from `is_supported_direct_edge` (`convert.rs:600-645`) → `classify_edge` `:696` `refusal_one_way` fallback; electrum arm `:681-686` only matches `↔Phrase`. Insert after `:686`/before `:695`. Redirect is real: `addresses --from electrum-phrase=` (`addresses.rs:224-272`, Electrum-vector-tested `cli_addresses_electrum.rs`); `--address-type` required (`:35-36`), `p2pkh`/`p2wpkh` real. Exit preserved (`ConvertRefusal`). No test pins old message (`cli_convert_electrum.rs:565` asserts only `.failure()`+`contains("electrum-phrase")`). Manual doesn't quote it.
- **SemVer/scope.** v0.47.1→v0.47.2 PATCH. No clap flag/value/subcommand change → no GUI schema_mirror; flag-coverage gates NAMES not descriptions (`docs/manual/tests/lint.sh:84-96`). Batching the 3 sound (independent; shared files only CHANGELOG/Cargo/README/install.sh).

---

### REQUIRED FOLDS
1. I1 — cover `--slot @N.phrase` in slug 2 (or scope out + FOLLOWUP).
2. I2 — pin advisory to top-of-`run`, raw `args`, before `:284`.
3. I3 — slug-1 inspect = 1 row (`:3023`).
4. I4 — fold synopsis brace-pipe correction (`:2744`, `:3016`); verify vs `--help`.
5. M1 + M2 inline.
