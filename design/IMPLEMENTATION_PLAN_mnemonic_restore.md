# `mnemonic restore` Implementation Plan (single-sig; v0.43.0)

> REQUIRED SUB-SKILL: superpowers:subagent-driven-development. Steps use `- [ ]`.

**Goal:** implement the single-sig `mnemonic restore` subcommand per `design/SPEC_mnemonic_restore.md` (R0-GREEN, `56d48d1`). Secret seed (ms1/phrase/entropy/seedqr) + optional BIP-39 passphrase → watch-only restore document (verify block leading with master fingerprint + first receive address(es), then concrete single-sig descriptor(s) for bip44/49/84/86; optional `--format` payload). Hard-fail-on-mismatch (exit 4) + `--allow-mismatch`; UNVERIFIED banner. No-signing / watch-only-out. Multisig DEFERRED (SPEC §11). Toolkit **v0.43.0** + paired GUI **v0.24.0**.

**Base:** branch `mnemonic-restore`, base master `6566941`. **Re-grep all cited line numbers before editing** (SPEC notes off-by-N decay; SPEC §4/§5 lines verified @ `6566941`). Gate per phase: `cargo test -p mnemonic-toolkit --no-fail-fast` (0 fail) + `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings`. NO `cargo fmt`. Mandatory opus R0 per phase + end-of-cycle; persist verbatim to `design/agent-reports/` BEFORE fold-commit.

**Test seed (public):** abandon×11+about. Master fingerprint: no-passphrase `73c5da0a` (in-tree `cli_export_wallet.rs:27`); passphrase `TREZOR` → `b4e3f5ed` (confirmed at runtime in SPEC R0/R1 — re-derive in-test, do NOT assert from memory, per `feedback_recapture_golden_only_when_current_correct`). bip84 account xpub (no pp) = `xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V`; `--format descriptor` single-sig = `wpkh([73c5da0a/84'/0'/0']xpub6CatW…/<0;1>/*)#hpg6d6w2`.

---

## Phase 1 — single-sig core

**Files:** `crates/mnemonic-toolkit/src/error.rs`; `crates/mnemonic-toolkit/src/cmd/restore.rs` (NEW); `crates/mnemonic-toolkit/src/cmd/mod.rs`; `crates/mnemonic-toolkit/src/main.rs`; `crates/mnemonic-toolkit/src/cmd/convert.rs` (1-line `pub(crate)` bump); `crates/mnemonic-toolkit/tests/cli_gui_schema.rs`; `crates/mnemonic-toolkit/tests/cli_restore.rs` (NEW).

### Task 1.1 — `RestoreMismatch` error variant
- [ ] **Step 1 (test):** in `tests/cli_restore.rs` (new), a unit-ish test deferred to 1.5 (exit-4 behavior is integration). Here just ensure the build compiles with the new variant — covered by 1.5's exit-4 assertion. (No standalone test for the variant; it is exercised by 1.5.)
- [ ] **Step 2 (impl):** add to `error.rs` `enum ToolkitError` (`#[non_exhaustive]`, `:9`) the variant — **re-grep the alpha slot**, insert after `RepairShortCircuit`, before `SilentPayment`:
  ```rust
  /// `restore` reference cross-check failed: derived material ≠ supplied --expect-* (or, future, cosigner slot).
  RestoreMismatch { reference: &'static str, derived: String, expected: String, slot: Option<u8> },
  ```
  Add an arm at the same alpha position in the THREE forced-exhaustive blocks (re-grep; SPEC says ~`exit_code:471`, `kind:529`, `message:588`):
  - `exit_code`: `ToolkitError::RestoreMismatch { .. } => 4,`
  - `kind`: `ToolkitError::RestoreMismatch { .. } => "RestoreMismatch",` (**plan-R0 Minor-1:** neighbor arms return the **PascalCase variant name**, NOT kebab — `kind_strings_stable` test `error.rs:1182` enforces it; do NOT write `"restore-mismatch"`).
  - `message`: `ToolkitError::RestoreMismatch { reference, derived, expected, slot } => format!("restore: {reference} mismatch{} — derived {derived}, expected {expected}", slot.map(|s| format!(" at slot @{s}")).unwrap_or_default()),`
  - Do NOT add a `details()` arm (it has `_ => None`; SPEC I3 — restore uses `message()` only).
- [ ] **Step 3 (build):** `cargo build -p mnemonic-toolkit` — compiles (all exhaustive matches satisfied).
- [ ] (committed with 1.2.)

### Task 1.2 — subcommand scaffold + wiring + gui-schema fix
- [ ] **Step 1 (test, failing):** in `tests/cli_restore.rs`, a smoke test: run the bin `restore --from phrase=<test-seed-literal> --template bip84` and assert stdout contains `master fingerprint:` and `73c5da0a` and `wpkh(` and `<0;1>`. Use the `assert_cmd`/`Command::cargo_bin("mnemonic")` pattern from `tests/cli_export_wallet_descriptor.rs` (copy its harness header). Expected: FAIL (clap rejects `restore`).
- [ ] **Step 2 (impl scaffold):** create `cmd/restore.rs`:
  ```rust
  use std::io::{Read, Write};
  use crate::error::ToolkitError;
  #[derive(clap::Args, Debug)]
  pub struct RestoreArgs { /* flags — Task 1.3/1.4/1.5 fill in */ }
  pub fn run<R: Read, W: Write, E: Write>(
      args: &RestoreArgs, stdin: &mut R, stdout: &mut W, stderr: &mut E, _no_auto_repair: bool,
  ) -> Result<u8, ToolkitError> { todo!() }
  ```
  `cmd/mod.rs`: add `pub mod restore;` (slot after `repair`, before `silent_payment`; M6 — don't re-sort existing drift). `main.rs`: add `Command::Restore(cmd::restore::RestoreArgs)` to `enum Command` (feature-clustered — anywhere is fine, M-d) + dispatch arm `Command::Restore(args) => cmd::restore::run(args, stdin, stdout, stderr, cli.no_auto_repair),` (**plan-R0 Minor-2:** pass the bindings BARE — `stdin/stdout/stderr` are already `&mut io::Stdin` etc. at `main.rs:149-151`; no spurious `&mut`. Mirror `convert`'s arm at `main.rs:156`).
- [ ] **Step 3 (gui-schema fix):** `tests/cli_gui_schema.rs` — re-grep the 28-name vec (`:~77`) + the "28" literals (`:~74`, `:~108`); add `"restore"` (alpha: after `"repair"`, before `"seed-xor-combine"`) and bump 28→29.
- [ ] **Step 4:** Task 1.3-1.5 implement `run`; the 1.2 smoke test stays failing (todo!) until 1.4. Commit 1.1+1.2 scaffold together once it builds: `git add` the 6 files → `feat(restore): RestoreMismatch error + restore subcommand scaffold + gui-schema 28→29 (P1.1-1.2)`.

### Task 1.3 — input resolution (secret channels)
- [ ] **Step 1 (impl):** in `RestoreArgs` add flags (mirror `convert`/`addresses` clap attrs — re-grep their derive attrs):
  - `--from <FROM>`: a `String` parsed `node=value` (reuse `convert`'s `--from` parse approach; restrict node ∈ {ms1,phrase,entropy,seedqr}; any other → `ToolkitError::BadInput` exit 1, pattern `addresses.rs:223`).
  - `--passphrase <P>` / `--passphrase-stdin` (mutex; `@env:` on `--passphrase`); `--language <L>` (`Option<CliLanguage>`); `--network` (default mainnet); `--account u32` (default 0); `--template <T>` (`Option<CliTemplate>`).
- [ ] **Step 2 (impl resolution):** resolve `--from` value (`@env:` via `resolve_env_var_sentinel`, `-` via `read_stdin_to_string`); passphrase via `--passphrase`(@env) / `read_stdin_passphrase` under the convert stdin-mutex (`--passphrase-stdin` XOR `--from=-`; reuse the check shape at `convert.rs:798-813`). Convert seed node → `(entropy, derive_language)`:
    - phrase/seedqr → `Mnemonic`/`seedqr::decode` → `to_entropy()`; derive_language = `--language` (default english).
    - entropy → hex decode; derive_language = english (irrelevant — no wordlist).
    - **ms1 → `slot_ms1::resolve_ms1_slot(value, --language, 0)`** → use `.entropy` + `.derive_language` (SPEC I-C; `slot_ms1.rs:37`).
  `mlock::pin_pages_for` the entropy + passphrase (pattern `convert.rs:841-847`); `secret_in_argv_warning` on inline-argv secrets (pattern `addresses.rs:126-146`).
- [ ] **Step 3 (test):** add tests — `@env:`+`--passphrase-stdin` path derives `b4e3f5ed` (re-derive the expected fp in-test via a second `convert`-equivalent OR hardcode after a local check); non-seed `--from xpub=…` → exit 1; stdin-mutex violation → exit 1; ms1 `mnem` non-english card derives the wire-language seed + `--language` conflict → `SlotInputViolation` exit 2. (ms1 test vectors: generate via `ms`/`convert --to ms1` for the test seed at write time.)
- [ ] (committed with 1.4.)

### Task 1.4 — derivation + descriptor + address (the core)
- [ ] **Step 1 (impl):** bump `convert::script_type_from_template` to `pub(crate)` (`convert.rs:393`). In `run`: if `--template` is `Some(t)` and `t.is_multisig()` (`template.rs:47`) → `BadInput` exit 1 ("restore is single-sig only; --template ∈ {bip44,bip49,bip84,bip86}"). Determine the type set: `Some(t)` → `[t]`; `None` → `[Bip44,Bip49,Bip84,Bip86]`. For each T:
  ```rust
  let acct = derive_slot::derive_bip32_from_entropy(&entropy, &passphrase, derive_language, network, T, account)?;
  // master_fingerprint identical across T (path-independent) — capture once.
  let st = convert::script_type_from_template(T).expect("single-sig template has a ScriptType");
  // first recv addr(s): for i in 0..count { child = acct.account_xpub.derive_pub(&secp, &[0,i]); render_address_from_xpub(&secp, &child, st, network) }
  let slot = /* ResolvedSlot — plan-R0 Minor-3: NO Default; spell ALL 7 pub fields, mirror the watch-only ctor at wallet_import/pipeline.rs:200-208: { xpub: acct.account_xpub, fingerprint: acct.master_fingerprint, path: acct.account_path, entropy: None, master_xpub: None, language: None, _entropy_pin: None } */;
  let descriptor = wallet_export::build_descriptor_string(T, &[slot], 1, network, account, None)?;
  ```
  Use `Secp256k1::verification_only()` (watch-only; pattern `addresses.rs:232`). NEVER touch `acct.account_xpriv`. Re-grep `ResolvedSlot` field names + how secret-slot resolution builds one (`synthesize.rs:642`, `bundle.rs:~528` `into_parts`) to construct a valid watch-only slot.
- [ ] **Step 2 (test → pass):** the 1.2 smoke test now passes; add: single-sig exact descriptor + first-addr for bip84 (no-pp) = `wpkh([73c5da0a/84'/0'/0']xpub6CatW…/<0;1>/*)#hpg6d6w2`, first recv `bc1q…` (capture real); from ms1/entropy/seedqr sources (same fp); all-4 default vs `--template bip84`; **negative: no `xprv`/`tprv` token anywhere in output**.
- [ ] **Step 3 (commit):** `git add` → `feat(restore): single-sig derivation → fingerprint + descriptor + first addr (P1.3-1.4)`.

### Task 1.5 — verify-gate + text output + advisory
- [ ] **Step 1 (impl):** add flags `--expect-fingerprint <hex>`, `--expect-xpub <xpub>` (requires `--template Some` else `ModeViolation` exit 2), `--allow-mismatch`, `--count <n>` (default 1). Verify-gate:
  - compute reference comparison: `--expect-fingerprint` vs `master_fingerprint`; `--expect-xpub` vs the single-`--template` `account_xpub`.
  - mismatch && !`--allow-mismatch` → `Err(RestoreMismatch { reference, derived, expected, slot: None })` (exit 4); print derived-vs-expected; NO descriptors.
  - mismatch && `--allow-mismatch` → emit under `✗ MISMATCH (overridden)` stderr banner, exit 0.
  - no reference → emit + loud `UNVERIFIED` stderr banner, exit 0.
  Text output: header `master fingerprint: <fp>  (passphrase: applied|none)` + CONFIRM line; per-type `descriptor:` + `first recv:` block. Emit `WatchOnly` OutputClass advisory (`secret_advisory::emit_output_class_advisory`, pattern `addresses.rs:258-261`).
- [ ] **Step 2 (test → pass):** `--expect-fingerprint` match → exit 0; mismatch → exit 4 + no `wpkh(`; mismatch + `--allow-mismatch` → exit 0 + `MISMATCH (overridden)`; no-reference → `UNVERIFIED` on stderr; `--expect-xpub` without `--template` → `ModeViolation` exit 2; watch-only advisory present.
- [ ] **Step 3 (full gate):** `cargo test -p mnemonic-toolkit --no-fail-fast 2>&1 | grep -cE '^test .* FAILED'` → 0; clippy clean.
- [ ] **Step 4 (commit):** `feat(restore): verify-gate (expect-*/allow-mismatch/UNVERIFIED) + text doc + advisory (P1.5)`.

### Phase 1 gate
- [ ] Full suite green, clippy clean. **Persist opus R0** to `design/agent-reports/restore-phase-1-R0-review.md` BEFORE proceeding; loop to 0C/0I.

---

## Phase 2 — import formats + `--json`

**Files:** `cmd/restore.rs`; `tests/cli_restore.rs`.

### Task 2.1 — `--format` payload
- [ ] **Step 1 (impl):** add `--format <CliExportFormat>` (reuse the enum, `export_wallet.rs:22-46`). If `Some` and `--template` is `None` → `ModeViolation` exit 2 (SPEC I-A — one-descriptor-per-emitter). Build `EmitInputs` for the single `--template`'s descriptor and dispatch the `WalletFormatEmitter` match (mirror the 16-field ctor at `export_wallet.rs:483-500` + dispatch `:507-561` — re-grep. **plan-R0 Minor-4:** `EmitInputs.script_type` is `wallet_export::WalletScriptType` (`mod.rs:165`), a DIFFERENT enum from the `convert::ScriptType` used for P1 address rendering — use `wallet_export::script_type_from_template` `mod.rs:193`, not the P1 helper). Append/emit the payload after the verify-doc (or as the sole stdout when `--format` given — decide + document; recommend: payload to stdout, verify-block to stderr when `--format` set, so the payload pipes cleanly).
- [ ] **Step 2 (test):** each format (descriptor/bitcoin-core/bip388) with `--template bip84`; `--format` + no `--template` → exit 2.
- [ ] **Step 3 (commit):** `feat(restore): --format importable payload (requires single --template) (P2.1)`.

### Task 2.2 — `--json` + `--output`
- [ ] **Step 1 (impl):** `--json` → structured `{ master_fingerprint, passphrase_applied, network, verification:{status,expected?,derived?}, wallets:[{type,descriptor,first_addresses[]}], import_payload? }`. Seed NEVER echoed (`is_argv_secret_bearing` guard). `--output <FILE>` writes to file.
- [ ] **Step 2 (test):** `--json` shape; **negative: no seed/`xprv` in json**; mismatch in `--json` (verification.status="mismatch", exit 4); `--output` file.
- [ ] **Step 3 (gate + commit):** full suite + clippy; `test(restore): --json + --output + redaction (P2.2)`.

### Phase 2 gate
- [ ] Green + clippy. **Persist opus R0** to `design/agent-reports/restore-phase-2-R0-review.md`; loop 0C/0I.

---

## Phase 3 — docs + GUI lockstep + release

**Files:** `docs/manual/src/40-cli-reference/41-mnemonic.md`; `docs/manual/tests/cli-subcommands.list`; `docs/manual/src/30-workflows/35-recovery-paths.md`; toolkit `Cargo.toml`/`Cargo.lock`/READMEs/`CHANGELOG.md`/`scripts/install.sh`/`design/FOLLOWUPS.md`; GUI repo (`src/schema/mnemonic.rs`, `Cargo.toml`, `pinned-upstream.toml`, `CHANGELOG.md`).

### Task 3.1 — manual
- [ ] **Step 1:** `## mnemonic restore` section in `41-mnemonic.md` (Synopsis/Flags/Worked example — every flag documented, run each example against the built v0.43.0 bin). Add `mnemonic restore` to `cli-subcommands.list` (flag-coverage lint). Recovery recipe in `35-recovery-paths.md` (seed+passphrase → restore on PC; lead with fingerprint verification).
- [ ] **Step 2 (audit):** build 4 CLIs; `make -C docs/manual audit MNEMONIC_BIN=… MD_BIN=… MS_BIN=… MK_BIN=… FIXTURES_DIR=… ; echo EXIT=$?` → 0.
- [ ] **Step 3 (commit):** `docs(manual): mnemonic restore section + recovery recipe (P3.1)`.

### Task 3.2 — toolkit release-prep v0.42.0 → v0.43.0
- [ ] **Step 1:** `Cargo.toml:3` → 0.43.0; BOTH README `<!-- toolkit-version: -->` markers + both `Status:` prose lines → v0.43.x; `CHANGELOG.md` v0.43.0 entry (MINOR — `mnemonic restore` single-sig); `scripts/install.sh` self-pin TAG → v0.43.0; relock + stage `Cargo.lock`; `readme_version_current` PASS. File FOLLOWUP `restore-multisig-cosigner-scope` in `design/FOLLOWUPS.md` (the deferred §11 multisig half + the correct `to_miniscript`/policy-params bridge).
- [ ] **Step 2 (commit):** `release(toolkit): v0.43.0 — mnemonic restore (single-sig) (P3.2)`.

### Phase 3 gate + ship (authorized: autonomous through tag)
- [ ] Green + clippy + `make audit` EXIT=0 + `readme_version_current`. **Persist opus R0** `design/agent-reports/restore-phase-3-R0-review.md`; loop 0C/0I.
- [ ] **End-of-cycle opus R0** over `master..HEAD` → `design/agent-reports/restore-end-of-cycle-R0-review.md`; loop 0C/0I.
- [ ] **Toolkit ship:** clean tree → `git checkout master && git merge --ff-only mnemonic-restore` → annotated tag `mnemonic-toolkit-v0.43.0` → push master + tag → confirm CI green (rust/manual/sibling-pin-check/install-pin-check).
- [ ] **Paired GUI v0.24.0 mini-cycle (after the toolkit tag is on origin):** branch off GUI master (v0.23.0). Add the `restore` `SubcommandSchema` to `SUBCOMMANDS` (`src/schema/mnemonic.rs:3191`) + `const RESTORE_FLAGS` (every flag; `secret` bools mirror `gui-schema` — `--passphrase*` secret, `--from` not). Bump toolkit pin v0.43.0 (`Cargo.toml` + `pinned-upstream.toml [mnemonic].tag`, pin_coherence) + relock; `pinned_version` + module-doc banner → mnemonic 0.43.0; GUI version → 0.24.0; CHANGELOG. Gate: build v0.43.0 mnemonic bin, `cargo +1.94.0 test --workspace` (4 `*_BIN` set) incl `schema_mirror`+`pin_coherence`+`secret_drift`; clippy. Per-phase + end-of-cycle R0. Ship tag `mnemonic-gui-v0.24.0`.
- [ ] Update CONTINUITY.md + memory; flip nothing (net-new) but record the shipped slug + the deferred multisig FOLLOWUP.

---

## Self-review (SPEC coverage)
SPEC §1 scope→P1-P3. §2 CLI surface→1.3/1.4/1.5/2.1/2.2. §3 control flow→1.3-1.5. §4 reuse APIs→1.4 (+ `script_type_from_template` pub(crate) bump). §5 error variant→1.1. §6 wiring+gui-schema→1.2. §7 lockstep→3.1/3.2 + GUI mini-cycle. §8 phasing→P1/P2/P3. §9 tests→per-task. §10 SemVer→3.2. §11 multisig deferred→FOLLOWUP (3.2). §12 fold log→addressed. No placeholders; all code sketches cite verified APIs; re-grep line numbers at task time.
