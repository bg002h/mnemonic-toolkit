# SPEC — quick-wins PATCH v0.47.2 (repair/inspect mutex docs + import-wallet --ms1 argv advisory + electrum→address refusal redirect)

**Status:** R0 gate (pre-implementation). MUST converge to 0 Critical / 0 Important before any code.
**Resolves:** FOLLOWUPs `manual-repair-flag-mutex-inaccuracy`, `import-wallet-ms1-argv-advisory-gap`, `electrum-phrase-address-refusal-honest-wording`.
**Source SHA:** branch `quick-wins-v0.47.2` off master `2d6c940`.
**SemVer:** PATCH — docs fix + additive stderr advisory + reworded refusal; all non-breaking. v0.47.1 → **v0.47.2**.

---

## 1. Summary
Three independent small fixes batched into one PATCH cycle. No clap flag/value/subcommand change → **no GUI `schema_mirror`**. Lockstep = manual-mirror only (slug 1 IS the manual; slugs 2/3 optionally add a one-line note).

## 2. Slug 1 — repair AND inspect flag tables: "mutually exclusive" → "may be combined" (manual prose)
Both the `repair` and `inspect` HRP-flag tables describe `--ms1/--mk1/--md1` as "mutually exclusive," but the source DROPPED the cross-HRP `conflicts_with_all` (v0.24.0 D35 fold) — they may be combined (one HRP per card). Source ground truth: `repair.rs:38-51` + `inspect.rs:24-28` ("`mnemonic inspect ms1xxx mk1yyy md1zzz` are valid") + doc-comments "May be combined … per D35"; proven by `cli_indel.rs:239 multi_group_both_emit_exit_5`.
- **repair table** (`docs/manual/src/40-cli-reference/41-mnemonic.md:2751-2753`): the 3 rows (`--ms1`/`--mk1`/`--md1`) each end "mutually exclusive with `--mk1` / `--md1`" (etc.) → reword to "may be combined with `--mk1` / `--md1` (one HRP per card; per D35)".
- **inspect table** (`:3023` + the adjacent `--mk1`/`--md1` rows): same reword (FIX-THE-CLASS — inspect also dropped the conflict).
- Pure prose; preserve the rest of each row (the `-`/stdin clause stays). `make -C docs/manual audit` GREEN after (flag-coverage gates flag-NAMES, not descriptions).

## 3. Slug 2 — `import-wallet --ms1` secret-in-argv advisory (additive)
`import-wallet --ms1 <value>` is secret-bearing on argv but fires no `secret_in_argv_warning` (only `--decrypt-password` does, `import_wallet.rs:472`). Add the advisory for consistency with sibling secret flags.
- **Where:** early in `import_wallet::run`, after args are available + before the heavy work (mirror the existing secret-resolution region near `:282`). Iterate `args.ms1`; if ANY entry is inline-secret-bearing — non-empty AND not `@env:`-prefixed (the `""` watch-only sentinel + `@env:VAR` values are NOT argv leaks) — fire once: `secret_in_argv_warning(stderr, "--ms1", "@env:VAR")` (signature `secret_advisory.rs:40`, already imported `:80`). Fire at most once even with multiple inline `--ms1`.
- **Channel wording:** `@env:VAR` (import-wallet `--ms1` has NO `--ms1-stdin`; the `-`/stdin form is repair/inspect's, not this command — the FOLLOWUP's "--ms1-stdin" was loose). R0 to confirm `import-wallet --ms1` has no stdin/`-` channel.
- Not a missing-route leak (`lint_argv_secret_flags` already anchors `--ms1` on `@env:` and passes) — this is purely the runtime advisory.

## 4. Slug 3 — `convert (electrum-phrase → address)` refusal: redirect to the shipped `addresses --from electrum-phrase`
`convert --from electrum-phrase --to address` hits the generic `refusal_one_way` ("cryptographically unrecoverable … one-way derivation barrier", `convert.rs:458-466`) via `classify_edge`'s catch-all (`:696`). That's imprecise — and as of v0.47.0 the operation IS supported by a different command. Replace with a dedicated, redirecting refusal.
- Add `fn refusal_electrum_phrase_to_address() -> ToolkitError` (mirror the existing `refusal_electrum_*` helpers `:547-559`): `ConvertRefusal("convert does not derive addresses from an Electrum native seed (Electrum uses its own PBKDF2 salt + non-BIP-44 derivation, not a convert edge). Use `mnemonic addresses --from electrum-phrase=<seed> --address-type <p2pkh|p2wpkh>`.")`.
- In `classify_edge` (`:649`), add a `(ElectrumPhrase, Address)` arm returning it BEFORE the `:696` one-way fallback (place near the existing electrum interceptors at `:685`). Scope: `Address` target only (the now-supported redirect); other `electrum-phrase → X` edges keep their current refusals (out of scope).
- Exit code unchanged (`ConvertRefusal` → same exit as `refusal_one_way`). R0 to confirm no test pins the OLD one-way message for the electrum→address edge specifically.

## 5. Tests
- **Phase 1 (RED):**
  - **slug 2 cell** (`cli_import_wallet*`): `import-wallet --ms1 <inline-ms1> …` → assert stderr contains `secret material on argv (--ms1)`; a paired cell that `--ms1 @env:VAR` (or `""`) does NOT fire it. RED now (no advisory).
  - **slug 3 cell** (`cli_convert*` / electrum): `convert --from electrum-phrase=<seed> --to address` → assert exit (unchanged) AND stderr/err contains the redirect substring `addresses --from electrum-phrase`. RED now (current message is the one-way barrier).
  - **slug 1:** no RED cell (manual prose; `make audit` is the guard).
- **Phase 2 GREEN:** slugs land; cells flip GREEN. Full `cargo test --no-fail-fast` + clippy `--all-targets` + `make -C docs/manual audit` GREEN.

## 6. Lockstep / scope
- **No GUI `schema_mirror`** (no flag/value/subcommand change). **No sibling-codec change.** No new `ToolkitError` variant (slug 3 reuses `ConvertRefusal`; slug 2 reuses the advisory).
- **Manual:** slug 1 IS the manual edit. Slugs 2/3: optional one-line notes (the `import-wallet --ms1` row could note the argv advisory; the convert electrum refusal isn't a flag-table item). R0 to decide if any manual note is required vs optional.

## 7. Phased plan
- **Phase 1 (RED):** slug-2 + slug-3 cells (§5). Verify RED-for-the-right-reason.
- **Phase 2 (GREEN):** slug 1 manual reword + slug 2 advisory + slug 3 refusal helper/arm. Full suite + clippy + `make audit` GREEN. Per-phase opus review → persist.
- **Phase 3 (release):** CHANGELOG `[0.47.2]`; version v0.47.1 → **v0.47.2** (Cargo.toml/lock + 2 READMEs + install.sh self-pin); flip all 3 FOLLOWUPs → resolved. Per-phase review.
- **Phase 4 (ship):** clean tree → ff-merge → tag `mnemonic-toolkit-v0.47.2` → push → watch CI (rust, install/sibling-pin-check, manual — fires because slug 1 changes a manual file).

## 8. Risk
Very low. Slug 1 is prose (corrects a known-false claim, source-confirmed for BOTH tables). Slug 2 is an additive stderr line (no behavior change to stdout/exit). Slug 3 rewords one refusal (exit-preserving) into a more-helpful redirect. R0 MUST confirm: (i) inspect genuinely allows combining (so the inspect-table reword is correct, not a new falsehood) — `inspect.rs:24-28` says yes; (ii) `import-wallet --ms1` has no stdin channel (so `@env:VAR` is the right advisory text); (iii) no test pins the old electrum→address one-way message; (iv) the slug-2 advisory fires once + skips `@env:`/`""`.
