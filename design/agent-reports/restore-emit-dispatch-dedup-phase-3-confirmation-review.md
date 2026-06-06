# End-of-cycle confirmation review — `restore-emit-dispatch-dedup` → `mnemonic-toolkit-v0.46.1`

**Reviewer:** opus `feature-dev:code-reviewer` (pre-tag confirmation). **Date:** 2026-06-05.
**Branch:** `restore-emit-dispatch-dedup`. **Verdict:** **0 Critical / 0 Important** — cleared to tag.

> Persisted verbatim per CLAUDE.md. The reviewer had no Bash this run; it verified all source substance and left 3 git-mechanics checks for the operator. **Operator ran all 3 (recorded at the bottom): tree clean, diff scope exact, clean linear ff — all GREEN.**

---

### VERDICT: 0 Critical / 0 Important (+ 0 Minor)

All verifiable substance is GREEN. Three release-mechanics checks require git plumbing absent from the reviewer's environment and were operator-confirmed before the tag.

---

### What verified clean (with confirming evidence)

**1. Version coherence — all 5 self-pin sites at 0.46.1, no stale 0.46.0.**
- `crates/mnemonic-toolkit/Cargo.toml:3` = `0.46.1`
- `Cargo.lock:706` `mnemonic-toolkit` entry = `0.46.1`
- `README.md:13` `<!-- toolkit-version: 0.46.1 -->`
- `crates/mnemonic-toolkit/README.md:9` `<!-- toolkit-version: 0.46.1 -->`
- `scripts/install.sh:32` `mnemonic-toolkit-v0.46.1`

Every other `0.46.0` hit is legitimate historical provenance (CHANGELOG `[0.46.0]`, prior-cycle SPECs/agent-reports, a `lint_zeroize_discipline.rs:239` provenance comment). None is a stale pin.

**2. CHANGELOG accuracy.** `[0.46.1]` (CHANGELOG.md:9-16) — all load-bearing claims verified: 4-way (all 4 sites route through the helper: `export_wallet.rs:580`, `:779`; `restore.rs:622`, `:669`; helper at `export_wallet.rs:74`); exit code unchanged (1, `BadInput` at `export_wallet.rs:123-126`); no CLI-surface change; no new error variant. ("net −124 LOC" — operator-confirmed via `git diff --stat`: source files export_wallet.rs + restore.rs = +104/−228 ≈ −124.)

**3. FOLLOWUP flip.** `design/FOLLOWUPS.md:98` Status = `resolved` toolkit-**v0.46.1** with the 3→4-way correction + divergent single-sig arm + audit trail documented. (Header `:92` retains the original "3 byte-identical copies" descriptive slug; the Status line supersedes it.)

**4. SemVer-PATCH is correct.** No subcommand/flag/value-enum added. The single user-visible delta is a reworded refusal *string* on an error path whose exit code (1), flag surface, and output structure are unchanged. No machine-consumable contract keyed on the old wording; no test/manual pinned it. PATCH is right; MINOR would over-classify.

**5. No new error variant.** `export_wallet.rs:74-137` references only `ExportWalletMissingFields` + `BadInput` (both pre-existing).

**6. Old single-sig string fully removed; test cell faithful.** The old `"requires a multisig wallet; restore is single-sig"` string has zero occurrences in `src/` and zero assertion occurrences in `tests/` (only an explanatory comment at `cli_restore.rs:605`). The new cell `restore_format_coldcard_multisig_single_sig_refused_exit_1` (`cli_restore.rs:598-635`) is non-vacuous: real `TREZOR_12` vector, asserts exit 1 + `stderr.contains("requires a multisig --template")` + empty stdout.

**7. Lockstep = NONE confirmed by source.** No clap surface changed; helper is a plain `pub(crate) fn` with no `#[arg]`/`#[value]`. `CliExportFormat`'s 11 variants unchanged. No GUI `schema_mirror`, manual mirror, or sibling-codec update owed.

---

### Operator-run mechanics checks (the reviewer could not run; all GREEN)

1. **Working tree clean:** `git status --porcelain | grep -vE '^\?\?'` → empty (only untracked recon/scratch). No tracked modifications.
2. **Diff scope (no creep):** `git diff 33db764..HEAD --stat` → exactly the 15 expected files (export_wallet.rs, restore.rs, cli_restore.rs, CHANGELOG, Cargo.toml, Cargo.lock, 2 READMEs, install.sh, FOLLOWUPS.md, SPEC, recon, 3 agent-reports). 450 ins / 235 del. No creep.
3. **Linear descendant:** `git merge-base --is-ancestor 33db764 HEAD` → exit 0 (clean ff). Master tip IS `33db764` → the ff-merge is a true fast-forward.
4. **Full suite:** `cargo test -p mnemonic-toolkit --no-fail-fast` fail-check → empty (all GREEN). Clippy `--all-targets` GREEN (Phase 2).

---

### GREEN — cleared to tag `mnemonic-toolkit-v0.46.1`.
