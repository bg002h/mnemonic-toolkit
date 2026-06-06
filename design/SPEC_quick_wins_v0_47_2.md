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
- **repair table** (`docs/manual/src/40-cli-reference/41-mnemonic.md:2751-2753`): all 3 rows (`--ms1`/`--mk1`/`--md1`) end "mutually exclusive with `--mk1` / `--md1`" (etc.) → reword to "may be combined with `--mk1` / `--md1` (one HRP per card; per D35)".
- **inspect table — (R0 I3) only ONE row** (`:3023`, the inspect `--ms1` row) carries "mutually exclusive with `--mk1` / `--md1`"; the `--mk1` (`:3024`) + `--md1` (`:3025`) inspect rows say only "use `-` for stdin" (nothing to reword). Reword the single `:3023` row (FIX-THE-CLASS — inspect also dropped the cross-HRP conflict, source-confirmed `inspect.rs:24-28` + tests `cli_positional_hrp_autodetect.rs:196/:213`).
- **(R0 I4, corrected by R0 round-2 I-new) ALSO fix the SYNOPSIS brace-pipe** at `:2744` (repair) + `:3016` (inspect): both render `{--ms1 <MS1> | --mk1 <MK1> [--mk1 <MK1>...] | --md1 <MD1> [--md1 <MD1>...]} [--json]…` — the brace-pipe wrongly implies "pick exactly one" (the same false mutex). Reword the `{ … | … | … }` to **curated independently-optional** form, preserving the didactic flag detail:
  - repair `:2744` → `mnemonic repair [--ms1 <MS1>] [--mk1 <MK1> [--mk1 <MK1>...]] [--md1 <MD1> [--md1 <MD1>...]] [--json]`
  - inspect `:3016` → same + ` [--reveal-secret]`
  - **NOT** the raw clap USAGE: the live `--help` USAGE is `mnemonic repair [OPTIONS] [STRING]...` / `mnemonic inspect [OPTIONS] [STRING]...` (the 3 optional HRP flags collapse into `[OPTIONS]`). Mirroring that verbatim would DELETE the flag detail. The synopsis is an **intentionally-curated abridgment, NOT a verbatim clap USAGE mirror** — clap-flag-NAME parity is enforced by the flag TABLE + `docs/manual/tests/lint.sh:84-96`, not by the synopsis. The implementer runs `--help` only to confirm flag NAMES, not to paste the USAGE line.
- Pure prose; preserve the rest of each row (the `-`/stdin clause stays). `make -C docs/manual audit` GREEN after (flag-coverage gates flag-NAMES, not descriptions).

## 3. Slug 2 — `import-wallet` inline-secret-on-argv advisory for BOTH `--ms1` AND `--slot @N.phrase` (additive)
`import-wallet --ms1 <value>` and its functional twin `--slot @N.phrase=<value>` (`import_wallet.rs:174-182`; doc-comment "Equivalent to `--ms1`") are both secret-bearing on argv, both `@env:`-only (no stdin channel), but NEITHER fires `secret_in_argv_warning` (only `--decrypt-password` does, `:472`). **(R0 I1 — fix the CLASS)** add the advisory for BOTH, for consistency with sibling secret flags.
- **(R0 I2) Where — CRITICAL placement:** at the TOP of `import_wallet::run` (after `:271`, **BEFORE the `:282-289` `env_resolved_owned` rebind** and before the early-return validation `:293`), iterating the ORIGINAL `args` param. The rebind RESOLVES `@env:VAR` to its secret value, so a post-rebind `@env:`-prefix check would mis-classify a legitimate `@env:` user and fire a FALSE advisory. Read raw `args`.
- **What to fire:** if ANY `args.ms1` entry is inline-secret-bearing — non-empty AND not `@env:`-prefixed (the `""` watch-only sentinel + `@env:VAR` values are NOT argv leaks) — fire once `secret_in_argv_warning(stderr, "--ms1", "@env:VAR")` (the `--ms1` positional-cosigner-index isn't surfaced in the label; one line suffices). For `--slot` phrase slots, **(R0 M-new) fire per inline phrase slot using its ACTUAL index** — `secret_in_argv_warning(stderr, &format!("--slot @{}.phrase=", s.index), "@env:VAR")` for each `s` with `subkey==Phrase && !value.is_empty() && !value.starts_with("@env:")` — matching the in-file per-leak-site label precedent (`import_wallet.rs:1329` `format!("--slot @{}.{}=", s.index, s.subkey.as_str())`) + `secret_advisory.rs:5-9` ("one advisory per (flag, slot-index)"). So a user leaking `@0.phrase` + `@2.phrase` sees both leak sites.
- **(R0 M2) Flag label has NO trailing space** — `"--ms1"` (not `"--ms1 "`), so the output reads `(--ms1)` matching the test assertion (the `--decrypt-password` precedent's trailing space is a quirk to NOT copy).
- **Channel wording:** `@env:VAR` (import-wallet `--ms1`/`--slot` have NO stdin channel — confirmed; the `-` form is repair/inspect's `--ms1`). Signature `secret_in_argv_warning(stderr, flag, alternative)` (`secret_advisory.rs:40`, imported `:80`).
- Not a missing-route leak (`lint_argv_secret_flags` already anchors both on `@env:` and passes) — purely the runtime advisory.

## 4. Slug 3 — `convert (electrum-phrase → address)` refusal: redirect to the shipped `addresses --from electrum-phrase`
`convert --from electrum-phrase --to address` hits the generic `refusal_one_way` ("cryptographically unrecoverable … one-way derivation barrier", `convert.rs:458-466`) via `classify_edge`'s catch-all (`:696`). That's imprecise — and as of v0.47.0 the operation IS supported by a different command. Replace with a dedicated, redirecting refusal.
- Add `fn refusal_electrum_phrase_to_address() -> ToolkitError` (mirror the existing `refusal_electrum_*` helpers `:547-559`): `ConvertRefusal("convert does not derive addresses from an Electrum native seed (Electrum uses its own PBKDF2 salt + non-BIP-44 derivation, not a convert edge). Use `mnemonic addresses --from electrum-phrase=<seed> --address-type <p2pkh|p2wpkh>`.")`.
- In `classify_edge` (`:649`), add a `(ElectrumPhrase, Address)` arm returning it BEFORE the `:696` one-way fallback (place near the existing electrum interceptors at `:685`). Scope: `Address` target only (the now-supported redirect); other `electrum-phrase → X` edges keep their current refusals (out of scope).
- Exit code unchanged (`ConvertRefusal` → same exit as `refusal_one_way`). No test pins the OLD message (R0-confirmed: `cli_convert_electrum.rs:565 electrum_phrase_to_address_is_refused` asserts only `.failure()` + `contains("electrum-phrase")`, stays GREEN).
- **(R0 M1) Update the stale test rationale comment** `cli_convert_electrum.rs:557-563` (above that test) — it predates v0.47.0 and claims the toolkit "deliberately does NOT derive addresses from an Electrum native seed … would produce WRONG addresses," which now contradicts the redirect three lines below. Reword to: convert isn't the edge; `addresses --from electrum-phrase` (v0.47.0) IS, Electrum-vector-tested. Strengthen the test to also assert `contains("addresses --from electrum-phrase")`.

## 5. Tests
- **Phase 1 (RED):**
  - **slug 2 cells** (`cli_import_wallet*` / `cli_secret_in_argv_warning`): (a) `import-wallet --ms1 <inline-ms1> …` → stderr contains `secret material on argv (--ms1)`; paired negative: `--ms1 @env:VAR` (and `""`) does NOT fire. (b) **(R0 I1+M-new)** `--slot @0.phrase=<inline>` → stderr contains `secret material on argv (--slot @0.phrase=)` (ACTUAL index); paired negative `@0.phrase=@env:VAR`. All RED now (no advisory).
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
