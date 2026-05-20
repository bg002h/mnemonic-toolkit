# cycle-prep recon — 2026-05-20

**Origin/master SHA at recon time:** `8a60cdcb0e9a573204a5710512c31dfd779c312a`
**Local branch:** `master`
**Sync state:** `up-to-date` (0 ahead / 0 behind)
**Untracked:** `.claude/` (gitignored per session)

Slugs verified: `export-wallet-coldcard-multisig-alias`, `emitinputs-canonical-descriptor-checksum-invariant-enforcement`, `manual-md-bin-real-binary-promote`, `manual-ms-bin-real-binary-promote`, `manual-chapters-22-23-24-post-v0.15.0-wire-format-refresh`.

All 5 entries were filed in this session (commits `5d2c0a6` + `fe32e9e`), so drift is expected to be minimal. The recon surfaced one **STRUCTURAL** citation issue + one **claim-counting** ambiguity. No DRIFTED-by-N findings.

---

## Per-slug verification

### export-wallet-coldcard-multisig-alias

- **WHAT (from FOLLOWUPS.md):** add `coldcard-multisig` as a `CliExportFormat` variant aliasing `Coldcard` with multisig-template precheck; close flag-name asymmetry between import + export.
- **Citations:**
  - `crates/mnemonic-toolkit/src/wallet_import/coldcard_multisig.rs` (file existence) — **ACCURATE**
  - `cmd/import_wallet.rs` `--format coldcard-multisig` enum variant — **ACCURATE** (variant at line 114 in `PossibleValuesParser::new()` array; match-arm at line 327)
  - `crates/mnemonic-toolkit/src/wallet_export/mod.rs` `enum CliExportFormat` — **STRUCTURALLY-WRONG**. The `CliExportFormat` enum is at `crates/mnemonic-toolkit/src/cmd/export_wallet.rs:22-36`, NOT `wallet_export/mod.rs`. Current variants: `BitcoinCore, Bip388, Coldcard, Jade, Sparrow, Specter, Electrum`. The `Coldcard` variant alone exists at L28; no `ColdcardMultisig` (as the FOLLOWUP correctly predicts; this is the gap to close).
  - `wallet_export/coldcard.rs:42-55` multisig template dispatch — **ACCURATE** (`fn emit()` at L42-55; `WshMulti|WshSortedMulti|ShWshMulti|ShWshSortedMulti|TrMultiA|TrSortedMultiA` arm dispatches to `emit_coldcard_multisig_text` at L52)
- **Action for brainstorm spec:** correct the FOLLOWUP-body file path on the `CliExportFormat` citation to `cmd/export_wallet.rs:22-36`. Cite source SHA `8a60cdc`. The architectural plan (add `ColdcardMultisig` variant; multisig-template precheck; same body delegation to `Coldcard` dispatch) is otherwise grounded correctly.

### emitinputs-canonical-descriptor-checksum-invariant-enforcement

- **WHAT (from FOLLOWUPS.md):** enforce the "`canonical_descriptor` ends with `#<8-char-csum>`" invariant beyond convention (constructor assertion OR newtype wrapper).
- **Citations:**
  - `wallet_export/mod.rs` `EmitInputs.canonical_descriptor: &str` — **ACCURATE** (struct at L342; field at L345)
  - `wallet_export/bsms.rs:86-90` invariant comment — **ACCURATE** (quoted text matches verbatim: "Lines 1 + 2 are shared between the 2-line and 4-line shapes. Line 2 is `EmitInputs.canonical_descriptor` verbatim — the canonical builder (`wallet_export::pipeline::build_descriptor_string`) and descriptor-passthrough both produce strings with the `#<checksum>` suffix already attached.")
  - `cmd/export_wallet.rs` EmitInputs construction sites (3 paths) — **ACCURATE**:
    - `--template` path: L437 (in `run()`; canonical via `build_descriptor_string` at L378)
    - `--from-import-json` path: L608 (in `run_from_import_json()`; canonical via `parsed_ms.to_string()` at L596 — F9 v0.28.2 fix landed at `615b10e`)
    - `--descriptor` passthrough: L437 (in `run()`; canonical via miniscript `d.to_string()` at L311)
- **Action for brainstorm spec:** all citations are stable at HEAD `8a60cdc`. `EmitInputs` struct shape identical at F9 commit `615b10e` and HEAD. Newtype defense option remains structurally sound; constructor-assertion option needs to wrap all 3 construction sites in `cmd/export_wallet.rs`. Cite source SHA `8a60cdc`.

### manual-md-bin-real-binary-promote

- **WHAT (from FOLLOWUPS.md):** promote `MD_BIN=true` placeholder → real `md` binary in CI `manual.yml` (mirror the mk-cli install pattern at L72-77).
- **Citations:**
  - `.github/workflows/manual.yml` "Audit manual" step + `MD_BIN=true` — **ACCURATE** (step at L85-96; step name verbatim: "Audit manual (lint + verify-examples with real mnemonic binary)")
  - `docs/manual/tests/lint.sh` per-subcommand `--help` loop — **ACCURATE** (loop at L69-97; warn literal: "no flags parsed from `$cmd`; skipping" at L86)
  - `manual.yml:72-77` mk-cli install pattern — **ACCURATE** (cargo install command at L77: `cargo install --git https://github.com/bg002h/mnemonic-key --tag mk-cli-v0.2.0 mk-cli`)
  - `scripts/install.sh:35` md-cli tag pin — **ACCURATE** (pin value: `descriptor-mnemonic-md-cli-v0.6.0`)
- **Action for brainstorm spec:** all citations ACCURATE. **Note (informational only):** the mk-cli install pattern at `manual.yml:77` pins `mk-cli-v0.2.0`, but `scripts/install.sh:42` currently pins `mk-cli-v0.4.1` — this is an **existing** cross-pin tag-staleness in `manual.yml`, NOT a recon-error. When the brainstorm spec adds the md-cli (and ms-cli) install steps, consider also bumping the mk-cli pin in `manual.yml` to match `install.sh`'s `mk-cli-v0.4.1`. Cite source SHA `8a60cdc`.

### manual-ms-bin-real-binary-promote

- **WHAT (from FOLLOWUPS.md):** promote `MS_BIN=true` placeholder → real `ms` binary in CI `manual.yml`. Sibling-successor to `manual-md-bin-real-binary-promote`.
- **Citations:**
  - `.github/workflows/manual.yml` `MS_BIN=true` — **ACCURATE** (at L94 in the Audit manual step)
  - `scripts/install.sh:38` ms-cli tag pin — **ACCURATE** (pin value: `ms-cli-v0.4.0`)
  - `Companion: manual-md-bin-real-binary-promote` cross-cite — **ACCURATE** (FOLLOWUPS.md L2690 ↔ L2700 mutual cross-cite)
- **Action for brainstorm spec:** all citations ACCURATE; same install-step pattern as `manual-md-bin-real-binary-promote`. The two FOLLOWUPs can be folded in a single brainstorm/cycle. Cite source SHA `8a60cdc`.

### manual-chapters-22-23-24-post-v0.15.0-wire-format-refresh

- **WHAT (from FOLLOWUPS.md):** refresh transcripts + prose for the 3 quickstart chapters (22/23/24) post-v0.15.0 wire-format break; plus parallel grep + audit of "9 OTHER" chapters with stale card strings.
- **Citations:**
  - 8 transcript files at `docs/manual/transcripts/{22-first-bundle,23-verify,24-recover,24-recover-md1}.{cmd,out}` — **ACCURATE** (all 8 exist)
  - 3 quickstart chapter sources at `docs/manual/src/20-quickstart/{22-first-bundle,23-verify,24-recover}.md` — **ACCURATE** (all 3 exist)
  - `verify-examples.sh` SKIP_STEMS list — **ACCURATE** (4 entries: `22-first-bundle.cmd`, `23-verify.cmd`, `24-recover.cmd`, `24-recover-md1.cmd`)
  - "9 OTHER manual chapters" claim — **CLAIM-COUNTING AMBIGUITY**:
    - Actual `grep -l 'ms10entrsq\|mk1qprsqhp\|md1zsxdsp'` returns **9 total chapters**: 22-first-bundle.md, 23-verify.md, 24-recover.md, 31-singlesig-steel.md, 35-recovery-paths.md, 41-mnemonic.md, 42-md.md, 43-ms.md, 44-mk-cli.md.
    - The FOLLOWUP body says "9 OTHER manual chapters" which is ambiguous: if read as "9 in addition to the 3 quickstart", that's 12 total (too high). If read as "9 total (3 + 6 others)", that's accurate. The literal grep returns 9 — so the FOLLOWUP undercounts the "other" set or overloads the word "OTHER".
    - **Resolution:** the actual "other" set is **6 chapters** (31, 35, 41, 42, 43, 44). Plus the 3 base quickstart = 9 total. The brainstorm spec should restate this clearly.
- **Action for brainstorm spec:** correct the FOLLOWUP-body wording to "6 OTHER manual chapters (31/35/41/42/43/44) in addition to the 3 quickstart chapters (22/23/24) = 9 chapters total carrying pre-v0.15.0 wire-format card strings." Cite source SHA `8a60cdc`. Brainstorm scope is bounded to **9 chapters total**, not 12. SKIP_STEMS removal (4 entries) is the closure trigger only for the transcript-replay side; the prose refresh on the other 5 chapters (24-recover-mk1's chapter + 31/35/41/42/43/44 — wait, 24-recover-mk1.cmd is NOT in SKIP_STEMS because its replay still PASSES; this is a separate observation). Actually the 4 SKIP_STEMS entries map cleanly to 3 quickstart chapter sources because `24-recover.cmd` + `24-recover-md1.cmd` both live in chapter `24-recover.md`.

---

## Cross-cutting observations

1. **One STRUCTURAL citation error** — `export-wallet-coldcard-multisig-alias` cites `wallet_export/mod.rs` for the `CliExportFormat` enum, but the enum actually lives in `cmd/export_wallet.rs:22-36`. The brainstorm spec must use the corrected path. The misciting was a P1b-R0-architect-introduced wording artifact (the F4 fix-spec referenced the export-wallet module-tree generally; the FOLLOWUP body inherited that imprecision when filed in P2c).

2. **One claim-counting ambiguity** — `manual-chapters-22-23-24-post-v0.15.0-wire-format-refresh` says "9 OTHER" but the actual count is 9 TOTAL (or 6 OTHER if "in addition to" is the intent). The brainstorm spec should reword to "6 OTHER" + the 3 quickstart = 9 TOTAL. Scope estimate: 9-chapter audit.

3. **No DRIFTED-by-N findings** across all 5 slugs. Citations filed this session (`5d2c0a6` + `fe32e9e`) all align with HEAD `8a60cdc` line numbers. Expected, since the SHA delta is 0 commits.

4. **Existing cross-pin staleness in manual.yml** — `manual.yml:77` pins `mk-cli-v0.2.0`, while `install.sh:42` pins `mk-cli-v0.4.1`. This is a **pre-existing** cross-pin drift surfaced incidentally by the manual-md-bin recon; it's NOT a citation error but a candidate sub-task for the brainstorm spec that adds the md-cli + ms-cli install steps. Bump all three to the install.sh-locked tag values in one go.

5. **Sync state** — local master ≡ origin/master at `8a60cdc`. Recon source-of-truth verification was against HEAD bytes (which equal origin/master bytes); no `git show origin/master:<path>` fallback was needed.

---

## Recommended brainstorm-session scope ordering

The 5 slugs naturally group into 3 brainstorm cycles:

- **Group A — toolkit ergonomic-surface (small):** `export-wallet-coldcard-multisig-alias` + `emitinputs-canonical-descriptor-checksum-invariant-enforcement`. Both touch `wallet_export/` + `cmd/export_wallet.rs`. ~50-100 LOC. GUI schema-mirror lockstep required for the first; the second is a pure type-system refactor.

- **Group B — CI hygiene (small; tight pair):** `manual-md-bin-real-binary-promote` + `manual-ms-bin-real-binary-promote`. Both modify `manual.yml`. ~10-20 LOC across 2 cargo-install steps. Single brainstorm; single commit.

- **Group C — multi-chapter manual refresh (large):** `manual-chapters-22-23-24-post-v0.15.0-wire-format-refresh`. 9-chapter scope (3 quickstart + 6 cross-references). Independent cycle; manual-v0.3.0 target. Depends on Group B (real `md`/`ms` binaries in CI to validate chapter-42/43/44 prose post-refresh) — Group B is a structural prerequisite.
