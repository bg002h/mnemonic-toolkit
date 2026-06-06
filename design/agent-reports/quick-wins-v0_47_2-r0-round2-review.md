# R0 Architect Review (round 2) — `SPEC_quick_wins_v0_47_2.md`

**Reviewer:** opus `feature-dev:code-reviewer` (mandatory pre-implementation R0 gate). **Date:** 2026-06-06.
**Branch:** `quick-wins-v0.47.2`. **Verdict:** **0 Critical / 1 Important (I-new)** (+ 1 Minor). NOT GREEN.

> Persisted verbatim per CLAUDE.md. Round-1 folds I1/I2/I3 + M1/M2 verified sound. I-new: the I4 fold rested on a FALSE clap-USAGE premise. Fold → re-dispatch round 3. (Operator subsequently ran `--help` empirically — confirmed below.)

---

## VERDICT: 0 Critical / 1 Important (+ 1 Minor) — NOT GREEN

### IMPORTANT
**I-new — the I4 fold's "mirror the EXACT USAGE line" instruction is based on a false clap-rendering premise; following it literally makes the synopsis strictly worse.**
`repair.rs:31-34`/`inspect.rs:27-30` declare `ArgGroup::new("kind").required(false).multiple(true)` over 3 `required(false)` `#[arg(long)]` flags + a positional. clap puts ONLY required flags in USAGE and collapses optional flags to `[OPTIONS]`. **Operator-confirmed via live `--help`:** `Usage: mnemonic repair [OPTIONS] [STRING]...` and `Usage: mnemonic inspect [OPTIONS] [STRING]...` — the 3 HRP flags do NOT appear in USAGE. So "mirror the EXACT USAGE line" would replace the didactic synopsis with `mnemonic repair [OPTIONS] [STRING]...`, deleting all flag detail. Also: the curated synopsis already diverges from clap (omits `--max-indel`/`--max-subst`); `docs/manual/tests/lint.sh:84-96` gates flag-NAMES appearing *somewhere* in the chapter (the flag table satisfies it), NOT a verbatim synopsis. So the brace-pipe doesn't "violate the clap-mirror invariant"; it's just a wrong-and-misleading false-mutex in a curated abridgment.
**Fix:** Strike "mirror the EXACT USAGE line." Reword the brace-pipe `{--ms1 | --mk1 | --md1}` → curated independently-optional `[--ms1 <MS1>] [--mk1 <MK1> [--mk1 <MK1>...]] [--md1 <MD1> [--md1 <MD1>...]] [--json]` (preserve flag detail, fix the false mutex). Add a note: the synopsis is an intentionally-curated abridgment, NOT a verbatim clap USAGE mirror (clap collapses these to `[OPTIONS]`); flag parity is enforced by the table + lint.sh. Implementer runs `--help` only to confirm flag NAMES.

### MINOR
**M-new — slot advisory label.** The SPEC's `--slot @N.phrase=` (literal `N`, fire-once) diverges from the in-file per-index precedent `import_wallet.rs:1329` `format!("--slot @{}.{}=", s.index, s.subkey.as_str())` → `--slot @0.phrase=`, and from `secret_advisory.rs:5-9` ("one advisory per (flag, slot-index)"). Pick consciously + add a one-line rationale. Self-consistent as written (SPEC + test both use literal `N`).

---

## What verified clean
- **I1 sound — `--slot @N.phrase` detection clean pre-rebind + phrase-only scoping correct.** `--slot` is `Vec<SlotInput>` (`import_wallet.rs:182`); `SlotInput { index: u8, subkey: SlotSubkey, value: String }` (`slot_input.rs:96-101`) — typed + live on raw `&args`. Detection: `s.subkey == SlotSubkey::Phrase && !s.value.is_empty() && !s.value.starts_with("@env:")` (no grammar re-parse). Phrase-only is correct (NOT fix-the-instance): import-wallet rejects every non-`phrase` slot subkey at `:1052-1061` (exit 1), so other subkeys have no working channel. Secret-on-argv + `@env:`-only confirmed (`lint_argv_secret_flags.rs:127`); no advisory today (`:472`/`:2173` only).
- **I2 placement correct.** Raw `args.ms1`/`args.slot` at top-of-`run` (before the `:283-289` `env_resolved_owned` rebind that resolves `@env:`) sees un-resolved prefixes; firing before the `:293` early-return is right; no borrow issue (immutable reads before the shadow).
- **I3 accurate.** `:3023` is the ONLY inspect row with the false mutex; `:3024`/`:3025` say only "use `-` for stdin." Repair `:2751-2753` all three wrong.
- **I4 location facts accurate** (only the rendering-target was wrong): synopses `:2744`/`:3016` are the brace-pipe; no test/trycmd pins them.
- **Real USAGE (item 3):** `mnemonic repair [OPTIONS] [STRING]...` / `mnemonic inspect [OPTIONS] [STRING]...` (operator-confirmed via `--help`).
- **M1/M2** accurate (stale `cli_convert_electrum.rs:557-563` comment; no-trailing-space label).
- **Convergence:** import-wallet secret-argv set is exactly {`--ms1`, `--slot @N.phrase`, `--decrypt-password`} (`lint_argv_secret_flags.rs:127`) — no 4th surface. The four false-mutex rows + 2 synopses are the only occurrences for these flags; other "mutually exclusive" manual hits are legitimate. No trycmd pins `--help`.
- **SemVer/scope:** v0.47.1→v0.47.2 PATCH; no GUI schema_mirror; manual edit (slug 1) fires the `manual` CI workflow.

### REQUIRED FOLD
1. I-new — strike "mirror EXACT USAGE"; curated independently-optional synopsis + abridgment note.
2. M-new (optional) — slot-label rationale.
Re-dispatch round 3.
