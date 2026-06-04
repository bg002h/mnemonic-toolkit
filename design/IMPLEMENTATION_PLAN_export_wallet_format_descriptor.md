# `export-wallet --format descriptor` Implementation Plan

> REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Steps use `- [ ]`.

**Goal:** Add `mnemonic export-wallet --format descriptor` → bare canonical multipath `<descriptor>#<checksum>` on stdout; document the concrete↔bundle round-trip. Toolkit **v0.42.0**, paired GUI **v0.23.0**.

**Source:** `design/SPEC_export_wallet_format_descriptor.md` (R0-GREEN, `6dc1805`). Branch `export-wallet-format-descriptor`, base master `a26377e`. **Re-grep all line numbers before editing** (inserting the enum variant shifts downstream lines +1 — SPEC R0-m5). Gate per phase: `cargo test -p mnemonic-toolkit --no-fail-fast` (0 fail) + `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings`. NO `cargo fmt`. Mandatory opus R0 per phase + end-of-cycle; persist to `design/agent-reports/`.

---

## Phase 1 — code (enum + emitter + dispatch + tests)

**Files:** `crates/mnemonic-toolkit/src/cmd/export_wallet.rs`, `crates/mnemonic-toolkit/src/wallet_export/descriptor.rs` (NEW), `crates/mnemonic-toolkit/src/wallet_export/mod.rs`, `crates/mnemonic-toolkit/tests/cli_export_wallet_descriptor.rs` (NEW).

### Task 1.1 — enum value + the 5 exhaustive-match arms + partition test
- [ ] **Step 1 (write failing test):** in a new `tests/cli_export_wallet_descriptor.rs`, a smoke test: `export-wallet --network mainnet --template bip84 --slot @0.xpub=<a valid acct xpub> --format descriptor` → stdout is one line ending `#<8 alnum>\n`, starts `wpkh(`, contains `<0;1>`. (Get a valid bip84 account xpub via `mnemonic convert --from phrase=- --to xpub --template bip84` on the test seed.)
- [ ] **Step 2 (run → fail):** `cargo test -p mnemonic-toolkit --test cli_export_wallet_descriptor 2>&1 | tail` — fails (clap rejects `descriptor` as an invalid `--format` value).
- [ ] **Step 3 (impl):** in `cmd/export_wallet.rs`: add `#[value(name = "descriptor")] Descriptor,` to `enum CliExportFormat`. Add `CliExportFormat::Descriptor => false` to `format_requires_template`'s match (group with the `BitcoinCore | Bip388 | Bsms | Green | Specter => false` arm — passthrough, no template required). Re-grep the exact line (was `:53`/`:55`).
- [ ] **Step 4:** add `Descriptor` to the passthrough (false) array in `format_requires_template_tests::partition_is_exact` (was `:838-846`) — it's a LOGIC test, not compile-forced (SPEC R0-M2).
- [ ] (Step 2 still fails until 1.2 wires the emitter dispatch — that's expected; commit 1.1+1.2 together after green.)

### Task 1.2 — DescriptorEmitter + the 4 emit/collect_missing dispatch arms
- [ ] **Step 1 (impl):** create `wallet_export/descriptor.rs` with `DescriptorEmitter` implementing all THREE `WalletFormatEmitter` methods (SPEC §2): `collect_missing → Vec::new()`; `emit → Ok(inputs.canonical_descriptor.to_string())` (NO trailing `\n`); `extension → "txt"`. Register `mod descriptor; pub(crate) use descriptor::DescriptorEmitter;` in `wallet_export/mod.rs` (mirror green's registration).
- [ ] **Step 2 (impl):** add `CliExportFormat::Descriptor => …` arms to the 4 remaining match sites: `run()` collect_missing (`:504`) + emit (`:523`); `run_from_import_json()` collect_missing (`:756`) + emit (`:777`). collect_missing arm: `(DescriptorEmitter::collect_missing(&inputs), "descriptor")`; emit arm: `DescriptorEmitter::emit(&inputs)`. Import `DescriptorEmitter` at each site (mirror `GreenEmitter`).
- [ ] **Step 3 (run → pass):** `cargo test -p mnemonic-toolkit --test cli_export_wallet_descriptor` — the 1.1 smoke test passes (single trailing `\n`, checksummed multipath).
- [ ] **Step 4 (commit):** `git add` the 4 files → `feat(export-wallet): --format descriptor (bare canonical <descriptor>#<checksum>) (P1.1-1.2)`.

### Task 1.3 — full integration tests
- [ ] **Step 1:** add to `tests/cli_export_wallet_descriptor.rs`:
  - single-sig (1.1, already) — assert exact `wpkh([fp/84'/0'/0']xpub…/<0;1>/*)#<csum>\n` for the test seed (capture the real value).
  - **multisig:** `--template wsh-sortedmulti --threshold 2 --slot @0.xpub= --slot @1.xpub= --format descriptor` → `wsh(sortedmulti(2,…))#<csum>` (NOT refused).
  - **round-trip (headline):** `bundle --descriptor '<wsh concrete>' --slot…` → its `--import-json`-able output (or build the envelope) → `export-wallet --from-import-json <env> --format descriptor` → canonical form == original (modulo checksum). Single-sig + wsh-multisig. **Do NOT taproot via from-import-json** (refused `:672-682`; SPEC R0-M3).
  - **taproot via passthrough:** `export-wallet --descriptor 'tr(NUMS,…)' --slot… --format descriptor` → emits (passthrough has no taproot refusal).
  - **flags ignored:** `--range 0,5 --timestamp 1700000000 --format descriptor` → no error, same descriptor line.
  - **`--output`:** `--output desc.txt` writes the one-line descriptor.
- [ ] **Step 2 (run → pass):** `cargo test -p mnemonic-toolkit --test cli_export_wallet_descriptor`. If the round-trip reveals lossiness, STOP + report.
- [ ] **Step 3 (full gate):** `cargo test -p mnemonic-toolkit --no-fail-fast 2>&1 | grep -cE '^test .* FAILED'` → `0`; `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` clean.
- [ ] **Step 4 (commit):** `test(export-wallet): descriptor format — multisig/round-trip/passthrough/flags/output (P1.3)`.

### Phase 1 gate
- [ ] Full suite green, clippy clean. **Persist opus R0** to `design/agent-reports/exportdesc-phase-1-R0-review.md` BEFORE proceeding; loop to 0C/0I.

---

## Phase 2 — docs + toolkit release-prep

**Files:** `docs/manual/src/30-workflows/37-wallet-export.md`, `docs/manual/src/40-cli-reference/41-mnemonic.md`, `crates/mnemonic-toolkit/Cargo.toml`, `Cargo.lock`, `README.md`, `crates/mnemonic-toolkit/README.md`, `CHANGELOG.md`, `scripts/install.sh`.

### Task 2.1 — manual recipe (B) + --format value list
- [ ] **Step 1:** `41-mnemonic.md` export-wallet `--format` value list → add `descriptor` (bare canonical descriptor + checksum). (Flag-coverage is flag-NAME-only, so this is mirror-discipline, not a hard gate — but do it.)
- [ ] **Step 2:** `37-wallet-export.md` — add a "Concrete descriptor ↔ bundle round-trip" section per SPEC §9: md1-keyless framing; IN `bundle --descriptor`; OUT `export-wallet --from-import-json --format descriptor`; taproot-via-passthrough caveat (R0-M3); distinguish from `--format green`. Use verified commands (run them against the built binary first).
- [ ] **Step 3 (audit):** build the 4 CLIs (mnemonic from this branch; ms/md/mk from their repos), `make -C docs/manual audit MNEMONIC_BIN=… MD_BIN=… MS_BIN=… MK_BIN=… FIXTURES_DIR=… ; echo "EXIT=$?"` (literal exit, no pipe-to-tail). Re-capture transcripts only if provably correct. EXIT=0.
- [ ] **Step 4 (commit):** `docs(manual): --format descriptor + concrete↔bundle round-trip recipe (P2.1)`.

### Task 2.2 — toolkit version bump v0.41.0 → v0.42.0
- [ ] **Step 1:** `crates/mnemonic-toolkit/Cargo.toml:3` → `0.42.0`; both README `<!-- toolkit-version: -->` markers → 0.42.0; `CHANGELOG.md` v0.42.0 entry (`--format descriptor` + the round-trip recipe; MINOR); `scripts/install.sh` self-pin `mnemonic-toolkit-v0.41.0` → `v0.42.0`; `cargo build` relock + stage `Cargo.lock`; `cargo test -p mnemonic-toolkit --test readme_version_current` PASS.
- [ ] **Step 2 (commit):** `release(toolkit): v0.42.0 — export-wallet --format descriptor (P2.2)`.

### Phase 2 gate
- [ ] Full suite green; clippy clean; `make audit` EXIT=0; readme_version_current PASS. **Persist opus R0** to `design/agent-reports/exportdesc-phase-2-R0-review.md`; loop to 0C/0I.

---

## End-of-cycle + ship (authorized: autonomous through tag)

- [ ] **End-of-cycle opus R0** over `master..HEAD` → `design/agent-reports/exportdesc-end-of-cycle-R0-review.md`; loop to 0C/0I.
- [ ] **Toolkit ship:** clean tree → `git checkout master && git merge --ff-only export-wallet-format-descriptor` → tag `mnemonic-toolkit-v0.42.0` (annotated) → push master + tag. (Tag-only; NOT crates.io.)
- [ ] **Paired GUI mini-cycle (after the toolkit tag is on origin)** — branch `export-wallet-format-descriptor-gui` off GUI master (v0.22.0): bump Cargo `mnemonic-toolkit` pin v0.41.0 → v0.42.0 + `pinned-upstream.toml [mnemonic].tag` → v0.42.0 (the `pin_coherence` guard enforces lockstep) + relock; add `"descriptor"` to `src/schema/mnemonic.rs` `EXPORT_FORMATS`; bump `pinned_version` "mnemonic 0.41.0" → "0.42.0"; version → GUI 0.23.0; CHANGELOG. Gate: build the v0.42.0 mnemonic binary, `cargo +1.94.0 test --workspace` (MNEMONIC_BIN=… etc.) 0-fail incl. `schema_mirror` + `pin_coherence`; clippy. Per-phase + end-of-cycle R0. Ship: merge ff → GUI master, tag `mnemonic-gui-v0.23.0`, push.
- [ ] Update CONTINUITY.md + save a memory record. File the slug `export-wallet-format-descriptor` as resolved (or note net-new shipped).

---

## Self-review (spec coverage)
SPEC §1 feature → P1. §2 emitter → 1.2. §3 enum+5 dispatch+`false` → 1.1-1.2. §4 decisions → 1.2/1.3. §5 lockstep → 2.1 (manual) + GUI mini-cycle. §6 phasing → P1/P2. §7 tests 1-7 → 1.1/1.3 + GUI parity. §8 SemVer → 2.2. §9 recipe → 2.1. §10 citations → re-grep per task. No placeholders.
