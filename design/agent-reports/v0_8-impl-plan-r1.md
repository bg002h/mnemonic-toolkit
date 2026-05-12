# v0.8 IMPLEMENTATION_PLAN review — r1

Date: 2026-05-11
Reviewer: opus-architect (r1) via general-purpose agent

## Summary

- Phase atomicity & sequencing: 0C / 0I / 0L / 0N
- Source file:line refs: 0C / 3I / 1L / 0N
- Phase 4 spike feasibility: 0C / 0I / 0L / 1N
- Manual-mirror invariant: 1C / 1I / 0L / 0N
- cargo command syntax: 0C / 0I / 0L / 0N
- Test corpus consistency: 0C / 1I / 0L / 0N
- `--wallet-name` handling: 0C / 0I / 1L / 0N
- Phase 6 release tasks: 1C / 1I / 0L / 0N
- FOLLOWUPS coordination: 0C / 1I / 0L / 0N

Total: 2C / 7I / 2L / 1N

---

## Findings — Phase atomicity & sequencing

(none — the R1-I3 incremental-deletion fix holds: Phase 1 step 6 keeps stub arms untouched; Phase 2 step 4 deletes Sparrow only; Phase 3 step 4 deletes Specter only; the v0.7 byte-exact-refusal contract for the not-yet-shipped format is preserved between phases. Verified at `crates/mnemonic-toolkit/src/cmd/export_wallet.rs:146-155`. Step-numbering nit: Phase 3 has two steps numbered `4.` — flagged as L-1 below.)

---

## Findings — Source file:line refs

### I-1 — `wallet_export.rs:17-25` REFUSAL_SECRET_INPUT range is wrong

**Location:** `design/IMPLEMENTATION_PLAN_v0_8_export_wallet_expansion.md:12`, `design/SPEC_export_wallet_v0_8.md:42`

**Evidence:** The plan + SPEC cite `src/wallet_export.rs:17-25` for `REFUSAL_SECRET_INPUT`. Reading `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/wallet_export.rs:17-25`: `REFUSAL_SECRET_INPUT` occupies lines 17-18 only (declaration + value); lines 20-25 contain a different symbol (`format_stub_message` — the helper §12 says moves to `wallet_export/mod.rs`). The cited range conflates two distinct symbols. Per `grep -n "REFUSAL_SECRET_INPUT" wallet_export.rs` → `17:pub const REFUSAL_SECRET_INPUT: &str =`.

**Fix:** In both files, replace `src/wallet_export.rs:17-25` with `src/wallet_export.rs:17-18` (REFUSAL_SECRET_INPUT proper). If the intent was to also point at `format_stub_message`, cite it separately at `src/wallet_export.rs:21-25`.

### I-2 — `slip0132.rs:138` BIP84_REF_ZPUB citation is stale (off by 31 lines)

**Location:** `design/IMPLEMENTATION_PLAN_v0_8_export_wallet_expansion.md:199`

**Evidence:** The plan's "Reuse opportunities" line: "BIP-84 reference vectors (`slip0132.rs:138` `BIP84_REF_ZPUB`)". Reading `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/slip0132.rs:130-140`: line 138 is mid-format-string inside `render_slip0132_info_line`. `grep -n "BIP84_REF_ZPUB" slip0132.rs` → declaration is at line **169**, with usages at 189, 198. The `slip0132.rs:138` citation is verbatim inherited from `IMPLEMENTATION_PLAN_v0_7.md:272` — it was already stale there, and `v0.8 phase 2` (commit `5dc83eb`) likely shifted lines.

**Fix:** Replace `slip0132.rs:138` with `slip0132.rs:169` in the IMPLEMENTATION_PLAN. Optional: file a sibling correction in `IMPLEMENTATION_PLAN_v0_7.md` (cosmetic; not in plan scope).

### I-3 — `cmd/export_wallet.rs:148-155` cited five times; actual stub-arm span is 148-153 (Sparrow 148-150 + Specter 151-153), wildcard 154, match-close 155

**Location:** `design/IMPLEMENTATION_PLAN_v0_8_export_wallet_expansion.md:10, 59, 77, 92, 176`; `design/SPEC_export_wallet_v0_8.md:38, 378`

**Evidence:** The plan/SPEC cite `cmd/export_wallet.rs:148-155` for the Sparrow/Specter stub arms five times. Per `awk` slice of `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/export_wallet.rs`:
- Line 147: `match args.format {`
- Line 148: `    CliExportFormat::Sparrow => {`
- Line 149: return-Err
- Line 150: `}`
- Line 151: `    CliExportFormat::Specter => {`
- Line 152: return-Err
- Line 153: `}`
- Line 154: `_ => {}`
- Line 155: `    }` (close-match)

The stub-arm match block is 148-153; the wildcard at 154 and the enclosing `match`'s close-brace at 155 are not stub-arm body to delete. The plan/SPEC use both `:148-154` and `:148-155` inconsistently across 7 sites. Mirror SPEC C-2 fix.

**Fix:** Globally normalize all `cmd/export_wallet.rs:148-155` citations to `cmd/export_wallet.rs:148-153` in the IMPLEMENTATION_PLAN and SPEC.

### L-1 — Phase 3 has two steps numbered `4.`

**Location:** `design/IMPLEMENTATION_PLAN_v0_8_export_wallet_expansion.md:92, 95`

**Evidence:** Phase 3 step list reads:
- "2. Implement `specter.rs`…"
- "3. Pin `specter_missing_wallet_name_refusal.stderr` fixture."
- "4. Wire `CliExportFormat::Specter`. **Delete the v0.7 Specter stub arm at `cmd/export_wallet.rs:148-155`**…"
- "Phase 3 reviewer-loop:"
- "4. Run reviewer-loop until 0C/0I; persist…"

Step `4.` appears twice (line 92 and line 95) — the reviewer-loop step should be `5.`. v0.7 plan uses contiguous numbering per phase.

**Fix:** In `design/IMPLEMENTATION_PLAN_v0_8_export_wallet_expansion.md:95` change `4. Run reviewer-loop` to `5. Run reviewer-loop`.

---

## Findings — Phase 4 spike feasibility

### N-1 — `electrum -o restore <xpub>` invocation is correct, but the wallet-path default needs to be pinned

**Location:** `design/IMPLEMENTATION_PLAN_v0_8_export_wallet_expansion.md:103-104`

**Evidence:** I verified `electrum/commands.py` (master) at https://raw.githubusercontent.com/spesmilo/electrum/master/electrum/commands.py — the `restore` command signature is `async def restore(self, text, passphrase=None, password=None, encrypt_file=True, wallet_path=None)` with docstring "Restore a wallet from text. Text can be a seed phrase, **a master public key**, …". So `electrum -o restore <xpub>` (where `-o` = `--offline`, confirmed by https://raw.githubusercontent.com/spesmilo/electrum-docs/master/cmdline.rst) is a valid invocation. However, the plan tells the spike-runner to "inspect the resulting wallet file at `~/.electrum/wallets/<name>`" — but with no `wallet_path` passed to `restore`, Electrum 4.5.x writes to a default-named path (`default_wallet` or a timestamped path depending on version). The spike report risks ambiguity if the runner can't locate the file.

**Fix:** In Phase 4 step 0, change the third bullet from "inspect the resulting wallet file at `~/.electrum/wallets/<name>`" to "pass `--wallet_path /tmp/electrum-spike-single.json` to `restore`, then read `/tmp/electrum-spike-single.json` directly. (`electrum --offline restore <xpub> --wallet_path /tmp/electrum-spike-single.json`)".

---

## Findings — Manual-mirror invariant

### C-1 — `tests/lint.sh` does not exist at repo root; plan's `bash tests/lint.sh` will hard-fail every phase exit gate

**Location:** `design/IMPLEMENTATION_PLAN_v0_8_export_wallet_expansion.md:46, 61, 211`

**Evidence:** The plan's Phase 0 exit gate runs `bash tests/lint.sh`; Phase 1 step 8 runs `bash tests/lint.sh flag-coverage`; final Verification at line 211 runs `bash tests/lint.sh`. Per `ls /scratch/code/shibboleth/mnemonic-toolkit/tests/lint.sh` → `No such file or directory`. The repo has no top-level `tests/` directory at all. `find . -name "lint*"` returns three matches: `docs/manual/tests/lint.sh`, `docs/quickstart/tests/lint.sh`, `docs/technical-manual/tests/lint.sh`. The one with `flag-coverage` is `docs/manual/tests/lint.sh`, and its header documents it is "Called from the Makefile as `make lint`" with required positional args `SRC_DIR=…`, `TESTS_DIR=…`, `MNEMONIC_BIN=…`, `MD_BIN=…`, `MS_BIN=…`, `MK_BIN=…`. A bare `bash tests/lint.sh` from repo root cannot work; even from `docs/manual/`, the script requires args. Both CI workflows (`.github/workflows/manual.yml:81` and `quickstart.yml:75`) invoke `make lint MNEMONIC_BIN=true MD_BIN=true MS_BIN=true MK_BIN=mk` — NOT `bash tests/lint.sh`.

This means: every per-phase exit gate as written will fail with "No such file or directory" and the manual-mirror check will never run.

Note: `CLAUDE.md` itself contains the same misstatement at line 24 ("The bidirectional `tests/lint.sh flag-coverage` step…") — the plan inherited it, but the plan is the artifact that drives execution, so the fix has to land there. The CLAUDE.md drift is a separate sibling cleanup.

**Fix:** Replace `bash tests/lint.sh` with `make -C docs/manual lint MNEMONIC_BIN="cargo run -q --manifest-path crates/mnemonic-toolkit/Cargo.toml --bin mnemonic --" MD_BIN=true MS_BIN=true MK_BIN=mk` in all three locations (lines 46, 61, 211). For Phase 1 step 8 (which wants `flag-coverage` alone), the invocation becomes `make -C docs/manual lint MNEMONIC_BIN=... MD_BIN=true MS_BIN=true MK_BIN=mk` (the granular sub-target is not exposed; `make lint` runs all six checks sequentially and short-circuits on the first failure).

### I-4 — `mnemonic` binary is not in `MNEMONIC_BIN=true` path in CI; flag-coverage for `mnemonic` is silently vacuous per FOLLOWUPS entry `lint-md-flag-coverage-vacuous-with-md_bin-true`

**Location:** `design/IMPLEMENTATION_PLAN_v0_8_export_wallet_expansion.md:46, 61, 211` (compounding C-1)

**Evidence:** `FOLLOWUPS.md:66-70` documents that CI's `flag-coverage` step substitutes `MNEMONIC_BIN=true MD_BIN=true MS_BIN=true MK_BIN=mk`; the `true` builtin emits no flags, so the script's `warn "no flags parsed"` path skips silently. The plan's per-phase claim that "the bidirectional check gates CI on every CI run" is therefore false for `mnemonic` specifically — and `mnemonic` is the only binary this plan adds flags to (`--wallet-name`). Per-phase exit gates as written would NOT catch a missing manual-mirror entry for `--wallet-name` or for the six new `--format` values when run in CI mode.

**Fix:** In Phase 0 step 4 OR Phase 1 step 7, add an explicit step: "Build the `mnemonic` binary once (`cargo build --workspace --bin mnemonic`), then run `make -C docs/manual lint MNEMONIC_BIN="$(pwd)/target/debug/mnemonic" MD_BIN=true MS_BIN=true MK_BIN=mk` so flag-coverage actually exercises `mnemonic --help`. The CI invocation continues to use `MNEMONIC_BIN=true`, but local per-phase exit gates must run against the real binary to catch drift before push." Cross-link to FOLLOWUPS entry `lint-md-flag-coverage-vacuous-with-md_bin-true`.

---

## Findings — cargo command syntax

(none — Phase 0 exit gate matches `IMPLEMENTATION_PLAN_v0_7.md`'s gauntlet style (`cargo build --workspace`, `cargo test --workspace --no-fail-fast`, `cargo clippy --workspace --all-targets -- -D warnings`); Phase 6 final gauntlet adds `cargo doc --workspace --no-deps` which also matches v0.7 final-Verification. No cargo-syntax drift.)

---

## Findings — Test corpus consistency

### I-5 — `electrum_single.json` / `electrum_multi_2of4.json` row note still references "Coldcard's stale sample fixtures" after R1-C1 resolution

**Location:** `design/SPEC_export_wallet_v0_8.md:396-397`

**Evidence:** The §13 fixture table row for `electrum_single.json` reads "Coverage: pinned to Coldcard's `electrum-single.json`" and `electrum_multi_2of4.json` reads "pinned to Coldcard's `electrum-2-of-4.json`". But R1-C1 (resolved inline at IMPLEMENTATION_PLAN line 287) demoted Coldcard's sample fixtures to "structural reference only, not authoritative" and re-pointed the authority to the Phase 4 spike. SPEC §9 second paragraph reflects the C1 fix correctly. The §13 table row still says "pinned to Coldcard's …" — a self-contradiction inside the SPEC.

**Fix:** In `design/SPEC_export_wallet_v0_8.md`, change the §13 rows for `electrum_single.json` and `electrum_multi_2of4.json` Coverage column from "pinned to Coldcard's `electrum-single.json`" / "pinned to Coldcard's `electrum-2-of-4.json`" to "pinned to Phase 4 step 0 spike-observed byte shape (singlesig)" / "pinned to Phase 4 step 0 spike-observed byte shape (2-of-4 multisig)". The IMPLEMENTATION_PLAN's Phase 4 RED step at line 110 already uses the correct phrasing; the SPEC table must match.

---

## Findings — `--wallet-name` handling

### L-2 — `--wallet-name` default `<template-human-name>-<account>` is unambiguous in SPEC §2 but the IMPLEMENTATION_PLAN never specifies clap-derive shape

**Location:** `design/IMPLEMENTATION_PLAN_v0_8_export_wallet_expansion.md:60`

**Evidence:** Phase 1 step 7 says "Add `--wallet-name` clap flag." but does not say whether the flag is `Option<String>` (default `None` at parse, resolved post-template-resolution) or `String` (with `default_value_t` computed at parse time). SPEC §2 line 30 specifies "default: `<template-human-name>-<account>` (e.g., `bip84-0`)" — but the default depends on resolved `--template` and `--account`, which clap-derive's `default_value_t` cannot evaluate (no cross-field dependency at parse time). The Phase 3 R1-L1 hardening then says `--wallet-name` is REQUIRED when `--format specter` — again a cross-flag dependency clap-derive can't enforce natively. The plan leaves the clap shape ambiguous; an implementer might pick `String` with a literal default, then fail to make it required for specter.

**Fix:** In Phase 1 step 7, expand to: "Add `--wallet-name` as `Option<String>` (clap-derive). Default resolution happens in `cmd::export_wallet::run` AFTER template + account are resolved: `let wallet_name = args.wallet_name.clone().unwrap_or_else(|| format!("{}-{}", template_human_name(template), account));`. The specter-required check happens in `SpecterEmitter::collect_missing` via `if inputs.wallet_name_was_user_supplied { vec![] } else { vec![MissingField::WalletName] }` — which means `EmitInputs` needs an extra `wallet_name_was_user_supplied: bool` field (Phase 0 step 6 add to the struct)."

---

## Findings — Phase 6 release tasks

### C-2 — `mnemonic-toolkit-v0.8.0` is ALREADY tagged; Phase 6 release-tag clause is incoherent

**Location:** `design/IMPLEMENTATION_PLAN_v0_8_export_wallet_expansion.md:144, 156`

**Evidence:** `git tag --sort=-v:refname | grep mnemonic-toolkit | head` returns:
```
mnemonic-toolkit-v0.8.0
mnemonic-toolkit-v0.7.1
mnemonic-toolkit-v0.7.0
…
```
v0.8.0 shipped at commit `7bb722a` (2026-05-07) as a `[BREAKING]` cut that closed 14 v0.8 FOLLOWUPS. The plan's Phase 6 step 2 says "Update `CHANGELOG.md` with `[0.8.X] — YYYY-MM-DD` entry" and step 5 says "Tag `mnemonic-toolkit-v0.8.X`" — but v0.8.0 is taken AND v0.8.0 was a `[BREAKING]` cut. The next tag in the v0.8 series must be v0.8.1 (these six new emitters are additive, not breaking).

The plan's Context paragraph at line 14 says "The work folds into the **v0.8 series** alongside in-flight v0.8 work (taproot-internal-key, electrum-version-info-stderr)" — but taproot-internal-key + electrum-version-info-stderr were both v0.8.0-cut work, not "in-flight."

**Fix:** In `design/IMPLEMENTATION_PLAN_v0_8_export_wallet_expansion.md`:
1. Line 14 Context paragraph: replace "The work folds into the **v0.8 series** alongside in-flight v0.8 work (taproot-internal-key, electrum-version-info-stderr)." with "The work folds into the **v0.8 series** as the v0.8.1 cut. v0.8.0 (commit 7bb722a, 2026-05-07) shipped taproot-internal-key, electrum-version-info-stderr, and 12 other v0.8 FOLLOWUPS as a `[BREAKING]` cut; v0.8.1 ships these six new export-wallet emitters as additive (no breaking change)."
2. Line 144 (Phase 6 step 2): replace `[0.8.X] — YYYY-MM-DD` with `[0.8.1] — 2026-05-??` (date filled at release).
3. Line 156 (Phase 6 step 5): replace `mnemonic-toolkit-v0.8.X` with `mnemonic-toolkit-v0.8.1`.

### I-6 — Phase 6 CHANGELOG entry doesn't mention v0.8.0 `[BREAKING]` precedent → users reading the changelog won't know if v0.8.1 is additive

**Location:** `design/IMPLEMENTATION_PLAN_v0_8_export_wallet_expansion.md:144`

**Evidence:** Phase 6 step 2: "Update `CHANGELOG.md` with `[0.8.X] — YYYY-MM-DD` entry listing six new formats, `--wallet-name` flag, module reorganization (internal)." Given `[0.8.0]` carried `[BREAKING]`, the v0.8.1 entry needs to either (a) include `[BREAKING]` if any new behavior is breaking, or (b) explicitly clarify "additive — no breaking changes" so the reader can scan the second-axis.

**Fix:** Reword Phase 6 step 2 to: "Update `CHANGELOG.md` with `## mnemonic-toolkit [0.8.1] — 2026-05-??` entry listing six new formats (`coldcard`, `jade`, `sparrow`, `specter`, `electrum`, `green`), `--wallet-name` flag (optional, required-for-specter), module reorganization (`wallet_export.rs` → `wallet_export/` submodule, internal-only). **No breaking changes** — v0.7 stable `--format bitcoin-core` / `--format bip388` byte-exact fixtures continue to pass through the new submodule dispatch."

---

## Findings — FOLLOWUPS coordination

### I-7 — Six "new" FOLLOWUPS slugs not yet checked against the entire FOLLOWUPS.md namespace; lookup confirmed none collide BUT the plan's "add-vs-resolve ownership" is inconsistent for `wallet-export-industry-formats`

**Location:** `design/IMPLEMENTATION_PLAN_v0_8_export_wallet_expansion.md:5, 65, 137`

**Evidence:** I greped `design/FOLLOWUPS.md` for the six new slugs (`coldcard-tr-multi-a-pending-firmware`, `coldcard-bip86-generic-export-pending-firmware`, `jade-tr-multi-a-pending-firmware`, `electrum-final-seed-version-drift`, `electrum-tr-multi-a-pending-libsecp-taproot`, `green-native-multisig-pending-server-support`) — zero hits. No slug collisions; that part is clean.

However, the plan's `wallet-export-industry-formats` handling is contradictory across three references:
- Header line 5: "Sibling FOLLOWUPS touched: `wallet-export-industry-formats` (resolved → partial-progress reopen)" — implies entry will be reopened.
- Phase 1 step 10 (line 65): "Mark `wallet-export-industry-formats` partial-progress note (Coldcard + Jade shipped)" — implies the entry stays as-is with an additive note.
- Phase 5 step 6 (line 137): "Flip `wallet-export-industry-formats` to RESOLVED (all six formats shipped)" — but per `FOLLOWUPS.md:872` the entry is **already** at `Status: resolved 3821f66` (v0.7 Phase 5 close).

**Fix:** Pick one model and apply it uniformly. Recommended:
1. Line 5: change "(resolved → partial-progress reopen)" to "(resolved 3821f66 by v0.7 Phase 5 close; v0.8.1 cycle extends coverage from 2 → 8 formats, recorded as `Resolution-extended:` notes appended to the existing entry — no reopen)".
2. Line 65: change "Mark `wallet-export-industry-formats` partial-progress note" to "Append `Resolution-extended (v0.8.1 Phase 1):` line to `wallet-export-industry-formats` listing Coldcard + Jade shipped."
3. Line 137: change "Flip `wallet-export-industry-formats` to RESOLVED" to "Append final `Resolution-extended (v0.8.1 Phase 5):` line to `wallet-export-industry-formats` listing all six new formats shipped; entry stays Status: resolved."

---

## Findings — Verification command exists

(rolled into C-1 above — the `bash tests/lint.sh` path is wrong at three callsites including the Verification section.)

---

## Closing note

The plan is structurally sound — atomicity, sequencing, R1-C1/C2/I1/I2/I3 resolutions hold, and the Electrum `restore <xpub>` spike command is valid against current upstream. The 2 Criticals are both about external-state drift that the plan inherited rather than introduced: the lint-script path (CLAUDE.md mis-states the same path) and the v0.8.0 tag (already cut). Both must land before Phase 0 begins, or Phase 0's exit gate will hard-fail and Phase 6 will tag-collide. The 7 Importants are routine fold-in work.
