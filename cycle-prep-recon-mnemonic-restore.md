# Cycle-Prep Recon — `mnemonic restore` (NET-NEW top-level subcommand)

**Recon ONLY** — no SPEC, no brainstorm, no code. Maps integration points + reuse
APIs + lockstep obligations against CURRENT source so a SPEC can be written with
accurate citations.

## Origin / sync state

- `git fetch -q origin` then `git rev-parse --short origin/master HEAD` →
  **origin/master == HEAD == `6566941`** (in sync; no rebase needed).
- `git status --porcelain` → working tree clean except untracked recon `.md`s,
  `CONTINUITY.md`, `.claude/` (no staged/modified source).
- Toolkit version: **`0.42.0`** (`crates/mnemonic-toolkit/Cargo.toml`). Sibling
  codecs are now **crates.io** pins (NOT git deps): `ms-codec = "0.4.0"`,
  `mk-codec = "0.4.0"`, `md-codec = "0.35"` (`Cargo.toml:20-27`; `Cargo.lock`
  md-codec 0.35.0 checksum present). The CLAUDE.md "git deps until v0.1" note is
  **DRIFTED** prose — they shipped to crates.io.

---

## 1. Subcommand wiring — **ACCURATE**

- Enum + dispatch both live in **`crates/mnemonic-toolkit/src/main.rs`** (NOT a
  separate `cli.rs`; there is none). `cmd/mod.rs` is only the `pub mod` list.
- `enum Command` at `main.rs:89-135` (22 variants, roughly alphabetical-by-feature
  but **not strictly sorted** — e.g. `Bundle`/`VerifyBundle`/`Convert` lead, then
  feature clusters). Each variant is `Variant(cmd::module::Args)`.
- Dispatch `match &cli.command` at `main.rs:153-192`. Each arm calls
  `cmd::module::run(args, stdin, stdout, stderr[, cli.no_auto_repair])`.
- `cmd/mod.rs:3-24` is an alphabetically-ordered `pub mod` list (one drift:
  `silent_payment` before `seed_xor`/`seedqr` — not strictly sorted).
- **To add `restore`:** (a) `pub mod restore;` in `cmd/mod.rs`; (b) new
  `Command::Restore(cmd::restore::RestoreArgs)` variant in `main.rs:89-135`;
  (c) a dispatch arm `Command::Restore(args) => cmd::restore::run(args, stdin,
  stdout, stderr, cli.no_auto_repair).map(|_| 0)` (or `.map(|_|0)` if `run`
  returns `Result<(), _>`); (d) `cmd/restore.rs` with `RestoreArgs: clap::Args`
  + `pub fn run<R: Read, W: Write, E: Write>(...) -> Result<u8, ToolkitError>`.
  The `run` signature template is `convert::run` (`convert.rs:737-743`) — it
  takes `no_auto_repair` and returns `Result<u8, ToolkitError>`.
- Clap parse-error exit-code remap (clap 2 → 64) is in `main.rs:140-147` and
  applies to all subcommands automatically.

---

## 2. Reuse APIs (the load-bearing part)

### 2a. seed → account xpub + MASTER fingerprint — **ACCURATE, callable-as-library** ✅

The single biggest reuse win: this is **NOT CLI-only**.

- **`crate::derive_slot::derive_bip32_from_entropy(entropy: &[u8], passphrase:
  &str, language: bip39::Language, network: CliNetwork, template: CliTemplate,
  account: u32) -> Result<DerivedAccount, ToolkitError>`**
  (`derive_slot.rs:42-58`). `pub(crate)` — directly callable from a new in-crate
  `cmd/restore.rs`.
- **`crate::derive::derive_full(phrase, passphrase, language: CliLanguage,
  network, template, account) -> Result<DerivedAccount, ToolkitError>`**
  (`derive.rs:60-81`) — phrase wrapper over the above. `pub`.
- **`DerivedAccount`** (`derive.rs:22-36`): public fields `entropy`,
  `master_fingerprint: Fingerprint`, `account_xpub: Xpub`, `account_xpriv:
  Xpriv`, `account_path: DerivationPath`. `master_fingerprint` is the
  passphrase-correctness ORACLE the feature leads with.
- `addresses.rs:214-222` already calls `derive_bip32_from_entropy` directly for
  phrase/entropy/seedqr — the exact pattern `restore` single-sig replays.
- For the four single-sig BIP types, map ScriptType→CliTemplate via
  **`addresses.rs:95-102 template_for(st)`** (bip44/49/84/86) — or just iterate
  the four `CliTemplate` single-sig variants.
- `convert.rs:1147-1238 compute_outputs` shows the full
  entropy/seedqr/phrase→Mnemonic→`to_entropy()`→derive spine if a node-dispatch
  is wanted; but `restore` can stay leaner (derive per template).

### 2b. export_wallet emitters — **ACCURATE, callable-as-library** ✅ (the `--format` payload)

- **`trait WalletFormatEmitter { fn collect_missing(&EmitInputs)->Vec<MissingField>;
  fn emit(&EmitInputs)->Result<String,ToolkitError>; fn extension()->&'static str; }`**
  (`wallet_export/mod.rs:397-401`). `pub(crate)`. Stateless (associated fns).
- **`struct EmitInputs<'a>`** (`wallet_export/mod.rs:466-518`) — 16 fields incl.
  `canonical_descriptor: CheckedDescriptor<'a>`, `resolved_slots: &[ResolvedSlot]`,
  `template: Option<CliTemplate>`, `script_type: WalletScriptType`, `network`,
  `account`, `threshold`, `threshold_user_supplied`, `master_xpub_at_0`,
  `wallet_name`, `wallet_name_is_non_default`, `taproot_internal_key`, `range`,
  `timestamp`, `bitcoin_core_version`, `bsms_form`. `pub(crate)`.
- **`CheckedDescriptor::new(&str)->Result<Self,ToolkitError>`**
  (`wallet_export/mod.rs:420-439`) — validates BIP-380 `#<8csum>` suffix.
- **`DescriptorEmitter`** (just shipped v0.42.0): re-exported at
  `wallet_export/mod.rs:44 (pub(crate) use descriptor::DescriptorEmitter)`; the
  `--format descriptor` CLI value at `export_wallet.rs:44-45`. This is the
  simplest emitter — emits the bare concrete descriptor — a natural default for
  restore's "importable payload."
- **`canonical_descriptor` build:** `build_descriptor_string(template, &resolved,
  k, network, account, taproot_internal_key) -> Result<String, ToolkitError>`
  (`wallet_export/pipeline.rs:18`, re-export `mod.rs:48`). This is **exactly** the
  template+slots→concrete-descriptor function restore needs. The whole emit spine
  is demonstrated in `export_wallet.rs:421-561` (template path) — restore can
  build `EmitInputs` and dispatch the same `match args.format` block. **All
  callable as library**; no CLI-only refactor.
- `CliExportFormat` enum (`export_wallet.rs:22-46`, 11 values) is the obvious
  type for restore's `--format <export-format>`; reuse it verbatim (and the
  `format_requires_template` partition at `export_wallet.rs:54-60`).

### 2c. address rendering — **ACCURATE, callable-as-library** ✅

- **`crate::address_render::render_address_from_xpub<C: Verification>(secp,
  child: &Xpub, script_type: ScriptType, network) -> String`**
  (`address_render.rs:18-34`). `pub(crate)`.
- **`network_from_xpub(&Xpub)->CliNetwork`** (`address_render.rs:39-44`).
- First-receive-address pattern: derive `m/0/0` child of account xpub, render.
  Full loop in `addresses.rs:236-251` (chain×index → `derive_pub` →
  `render_address_from_xpub`). Restore leads with first receive addr(s) per type
  — slice `index 0` (or a small `--count`).

### 2d. verify-bundle / bundle slot-binding + cross-check — **ACCURATE, callable-as-library** ✅

- **`crate::cmd::bundle::resolve_slots(slots: &[SlotInput], template, network,
  account, language: Option<CliLanguage>, passphrase: Option<&str>,
  multisig_path_family) -> Result<(Vec<ResolvedSlot>, Vec<(u8,&'static str)>),
  ToolkitError>`** (`bundle.rs:453-461`). `pub(crate)` — binds `@N` slots
  (xpub/phrase/entropy/seedqr/ms1) into `ResolvedSlot { xpub, fingerprint, path,
  entropy, master_xpub, language, _entropy_pin }`. `export_wallet.rs:358-366`
  calls it directly — the precedent.
- **`synthesize::synthesize_unified(...)`** (`synthesize.rs:745`) is the bundle
  emit hot-path; restore likely does NOT need it (it emits a descriptor, not 3
  cards) — but it is `pub` if needed.
- **Cross-check (multisig cosigner scope):** the canonical xpub-equality pattern
  is in `verify_bundle.rs::emit_watch_only_xpub_path_cross_check` (`:2135`) and
  the multisig checks (`:1748-1973`). Mechanism:
  - `md_codec::chunk::reassemble(&[&str]) -> Result<Descriptor, md_codec::Error>`
    (md-codec `chunk.rs:305`) reassembles the supplied `md1` strings into a
    `md_codec::Descriptor` exposing `.n`, `.tlv` (`.pubkeys`, `.fingerprints`,
    `.origin_path_overrides`), `.path_decl.paths` (`Shared`/`Divergent`), `.tree`.
  - `mk_codec::decode(&[&str]) -> Result<KeyCard, _>` → `KeyCard.xpub`.
  - `crate::synthesize::xpub_to_65(&xpub)` (used at `verify_bundle.rs:2196`)
    normalizes an xpub to its 65-byte form for byte-equality against
    `desc.tlv.pubkeys`.
  - The slot-auto-detect ("which slot the user's derived xpub fills") = find the
    `desc.tlv.pubkeys` entry equal to the derived xpub's 65-byte form — exact
    pattern at `verify_bundle.rs:2198-2208`.
  - Distinctness guard `parse_descriptor::check_key_vector_distinctness`
    (`parse_descriptor.rs:1208`) available if restore wants BIP-388 distinctness.
- **NB hot-path lesson (MEMORY):** `synthesize_unified` is the CLI bundle route;
  `synthesize_multisig_*` are tests-only. Restore should NOT route through
  bundle-emit — it composes `resolve_slots` + `build_descriptor_string` directly.

### 2e. md1 template handling — **ACCURATE** ✅

- `md_codec::chunk::reassemble(&md1_strs)` → `md_codec::Descriptor`. The
  Descriptor carries the `@N`-placeholder tree. The descriptor-mode bundle path
  (`bundle.rs:1138 bundle_run_unified_descriptor`) takes a raw `@N` template
  STRING (`--descriptor`), runs `lex_placeholders` (`parse_descriptor.rs:60`) +
  `resolve_placeholders` (`:156`) + `parse_descriptor` (`:747`) +
  `bind_descriptor_keys` (`:901`) to bind slots → concrete descriptor.
- **Two viable resolve routes for restore's multisig scope:**
  1. **Template+slots route (preferred):** decode `md1`→`Descriptor`, derive
     `CliTemplate` + threshold from it (`template_from_descriptor` at
     `wallet_export/mod.rs:262`; `extract_multisig_threshold(&d.tree)` used at
     `bundle.rs:1060`), build `ResolvedSlot`s from the other cosigners' `mk1`
     xpubs + own derived xpub, call `build_descriptor_string`.
  2. **Descriptor-string route:** render the reassembled `Descriptor` back to its
     `@N` template string (its Display) and feed the existing
     `lex/resolve/parse/bind` chain.
  Route 1 reuses the verified `build_descriptor_string` and matches the export
  path; **recommend route 1** for the SPEC. Either way, **a caller CAN resolve
  md1 + cosigner xpubs → concrete descriptor today** with existing `pub(crate)`
  APIs — no new crypto, no CLI-only refactor.
- Caveat to VERIFY in SPEC: which `mk1` xpub fills which `@N` is auto-detected
  via `desc.tlv.pubkeys` byte-match (wallet-policy mode) else positional — see
  `verify_bundle.rs:2189-2208`. Origin path per cosigner via `desc.path_decl` /
  `tlv.origin_path_overrides` (`verify_bundle.rs:2152-2162`).

### 2f. secret hygiene — **ACCURATE, all callable** ✅

- **`crate::cmd::convert::read_stdin_passphrase<R: Read>(&mut R) ->
  Result<String, ToolkitError>`** (`convert.rs:719-731`) — NULL-preserving,
  strips one trailing `\r?\n`. `pub(crate)`. Re-used by `addresses.rs:18`.
- **`crate::cmd::convert::read_stdin_to_string`** (`convert.rs:706-712`) — trims.
- **`crate::env_sentinel::resolve_env_var_sentinel(value: &str, flag: &str) ->
  Result<String, ToolkitError>`** (`env_sentinel.rs:56`). `pub(crate)`. Used in
  `addresses.rs:153,162`. NOTE: function name is `resolve_env_var_sentinel`
  (the prompt's `resolve_env_var_sentinel` is correct; ignore any "@env" var-name
  variants). Single-stdin-per-invocation mutex pattern: `addresses.rs:119-123`
  (passphrase-stdin vs `--from=-`) and `convert.rs:798-813`.
- **`crate::mlock::pin_pages_for(&[u8]) -> PinnedPageRange`** (`mlock.rs:90`) —
  pin secret heap pages; `convert.rs:841-847` is the template (pin passphrase +
  primary value). `mlock::report_at_exit()` runs in `main.rs:211` for all subs.
- **OutputClass advisory:** `crate::secret_advisory::worst_class_on_stdout(&[OutputClass])
  -> Option<OutputClass>` (`secret_advisory.rs:83`) +
  `emit_output_class_advisory(class, &mut stderr)` (`:97`). `enum OutputClass {
  Template, WatchOnly, PrivateKeyMaterial }` (`:80`). Restore is **secret-in /
  watch-only-out** → emit `WatchOnly` (note: stdout is watch-only). Pattern:
  `addresses.rs:258-261`. `secret_in_argv_warning(stderr, flag, alt)` (`:40`)
  for inline-secret-in-argv advisories (pattern `addresses.rs:126-146`).
- Redaction predicate for `--json` echo: `NodeType::is_argv_secret_bearing()`
  (`convert.rs:117-119`) — restore must NOT echo seed material in `--json`.

---

## 3. Error + exit-code scheme — **ACCURATE; likely NEW alphabetical-insert variant** ⚠

Exit-code scheme (`error.rs:471-524 exit_code()`):
- **1** — user-input class (`BadInput`, `Bip39`, `Bitcoin`, `Io`, `EnvVarMissing`,
  `NostrKeyParse`, `SilentPayment`, `VerifyMessage`, `DecodeAddress`,
  most `ImportWallet*`).
- **2** — format-violation / refusal class (`ConvertRefusal`, `DescriptorParse`,
  `ModeViolation`, `SlotInputViolation`, `HrpMismatch`, `ExportWalletSecretInput`,
  `ExportWalletMissingFields`, `Bip388Distinctness`, `Repair`, sibling-codec
  format errors).
- **3** — `FutureFormat` (reserved-tag / unsupported-version).
- **4** — **MISMATCH / verify class** (`BundleMismatch`, `Bip388VerifyDistinctness`,
  `DescriptorReparseFailed`, `ImportWalletSeedMismatch`, `XpubSearchNoMatch`).
- **5** — `RepairShortCircuit` (carried exit code).

**Cross-check MISMATCH / refuse-to-restore → exit 4** is the right tier (matches
`ImportWalletSeedMismatch`/`BundleMismatch` semantics: "supplied seed produces a
different xpub than the reference").

**Reuse vs new variant:**
- `ToolkitError::ImportWalletSeedMismatch { cosigner_index, derived_xpub,
  blob_xpub, path }` (`error.rs:215-220`, exit 4) is **semantically closest** —
  but its `message()` (`error.rs:714-722`) is `import-wallet:`-prefixed, so
  reusing it for a `restore` mismatch yields a misleading "import-wallet:" string.
- `BundleMismatch { card, message }` (`error.rs:70-73`, exit 4) message
  (`error.rs:638-641`) is bundle-specific ("if the engraved bundle was produced
  at a non-zero account...").
- **Recommendation:** add a NEW alphabetical-insert variant, e.g.
  `RestoreMismatch { reference: &'static str, derived: String, expected: String
  [, slot: Option<u8>] }` → exit 4, inserted alphabetically between `Repair*` and
  `SilentPayment` in the enum (`error.rs:266-279`) AND in the three exhaustive
  match blocks (`exit_code` `:516-518`, `kind` `:576-578`, `message` `:747-754`).
  **CLAUDE.md hard rule:** new variants + new match arms are **alphabetical-by-
  variant-name**; the three blocks are NOT yet retro-sorted (`error-rs-retroactive-
  alphabetical-sort` FOLLOWUP), so insert at the correct alpha position within
  each block's existing local ordering. `enum` is `#[non_exhaustive]` (`:9`) so
  adding a variant is non-breaking.
- For UNVERIFIED (no reference) path → just a loud `stderr` banner (no error;
  exit 0), no new variant. For arg/usage errors reuse `BadInput`(1) /
  `ModeViolation`(2) / `SlotInputViolation`(2) / `ConvertRefusal`(2).

---

## 4. gui-schema reflection — **ACCURATE; toolkit self-test WILL break** ⚠

- `mnemonic gui-schema` re-derives the schema from the live clap tree via
  `Cli::command()` (`main.rs:178-184` → `cmd::gui_schema::run(args, &root,
  stdout)`). Per-flag `secret` = `secrets::flag_is_secret(&name)`
  (`gui_schema.rs:1170`). **A new `Restore` clap variant auto-appears** — ran
  `target/debug/mnemonic gui-schema`: 28 subcommand entries today, `restore` NOT
  present (confirmed free). Adding the variant requires zero gui_schema.rs edits.
- **TOOLKIT self-test `tests/cli_gui_schema.rs` HARDCODES the full subcommand
  list + count** (`:76-110`): `assert_eq!(names, vec![...28 names...], "all 28
  user-facing subcommands must appear...")`. **Adding `restore` BREAKS this** —
  must add `"restore"` to the vec (alpha position after `repair`, before
  `seed-xor-combine`) and bump the "28"→"29" prose in the comment (`:74,108`).
  Other asserts in that file are per-flag-choice counts on `bundle`/`export-wallet`
  (`:180,292`) — unaffected by a new subcommand.
- `gui_schema_does_not_self_reference` (`:113`) — unaffected (restore != gui-schema).

---

## 5. GUI lockstep (`/scratch/code/shibboleth/mnemonic-gui` @ `4cac23b`) — **obligation NOTED**

- **`SUBCOMMANDS: &[SubcommandSchema]`** at `src/schema/mnemonic.rs:3191`. Add one
  `SubcommandSchema` entry. Struct (`src/schema/mod.rs:28-48`): `name`,
  `human_name`, `flags: &'static [FlagSchema]`, `positional_args: &[PositionalArgSchema]`,
  `allows_slots: bool` (TRUE for the multisig `--slot @N` scope),
  `conditional: Option<fn(&FormState)->FlagVisibility>` (None initially). Plus a
  `const RESTORE_FLAGS: &[FlagSchema]` (struct at `mod.rs:64-110`: `name`, `kind`,
  `required`, `repeating`, `help`, `secret`, `default_value`, `global`).
- **Secret-flag projection:** GUI `FlagSchema.secret` is HAND-CODED and must mirror
  toolkit's `gui-schema` output (driven by `secrets::flag_is_secret`, toolkit
  `src/secrets.rs:49-64`). `flag_is_secret` is a FLAT global flag-NAME matcher —
  `--passphrase`/`--passphrase-stdin`/`--ms1` already classify secret; restore's
  `--from <node>=<value>` channel is **NOT** in that set (intentional — value-
  dependent, same as convert's `--from`; runtime redaction uses
  `is_argv_secret_bearing`). So: **no new `flag_is_secret` entry needed** UNLESS
  restore introduces a brand-new literal secret flag name. If it does, add it to
  `secrets.rs:49-64` (+ the `known_secret_flags...` test `:71-84`) in the SAME
  toolkit PR, and set `secret: true` on the GUI FlagSchema.
- **Drift gates (GUI tests dir):**
  - `tests/schema_mirror.rs` — iterates **schema-DECLARED** subcommands, shells
    `<bin> <sub> --help` (prefers `gui-schema` JSON), set-equality on flag NAMES.
    A binary-only `restore` is **ungated until added to SUBCOMMANDS** (lagging
    indicator per MEMORY); once added, its flag set must match. NOT a wire-shape
    gate (`--json` payload self-updates manually).
  - `tests/schema_mirror_secret_drift.rs` — `{(sub,flag)|secret}` from
    `mnemonic gui-schema` must equal GUI `FlagSchema.secret==true` set.
  - `tests/pin_coherence.rs` — Cargo `mnemonic-toolkit.tag` must equal
    `pinned-upstream.toml [mnemonic].tag`. GUI must bump both pins to the new
    toolkit tag in lockstep.
- **Paired-PR rule (CLAUDE.md):** new subcommand → GUI v0.24.0 (the prompt's
  target) with the SUBCOMMANDS entry + pin bumps. Leading discipline = paired PR;
  the drift gate only fires on the GUI's next toolkit-pin bump.

---

## 6. Manual lockstep — **obligation NOTED**

- Per-subcommand sections in **`docs/manual/src/40-cli-reference/41-mnemonic.md`**
  follow `## mnemonic <sub>` → `### Synopsis` / `### Flags` / `### Worked example`
  (+ optional Notes/Refusals/Advisories/Exit codes). Examples: bundle (`:38`),
  convert (`:626`), export-wallet (`:677`), import-wallet (`:725`). Add a
  `## mnemonic restore` section with EVERY flag documented.
- **Flag-coverage lint** (`docs/manual/tests/lint.sh:62-96`): reads
  `docs/manual/tests/cli-subcommands.list` for the subcommand inventory, runs
  `$MNEMONIC_BIN <sub> --help`, greps every `--flag` token, asserts each is
  `grep -qF` present in `41-mnemonic.md`. **Add `mnemonic restore` to
  `cli-subcommands.list`** (confirmed absent — `restore` is free there) AND
  document all flags. Invoked via `make -C docs/manual lint` from
  `.github/workflows/manual.yml`.
- **Restore workflow recipe** belongs in **`docs/manual/src/30-workflows/35-recovery-paths.md`**
  (existing recovery chapter) — or a new `30-workflows/` file if the recipe is
  large. The 30-workflows dir is hex-numbered (`31`..`39`,`3A`).
- LESSON (MEMORY): re-capture golden artifacts ONLY when current behavior is
  proven correct; technical-manual is NOT CI-gated (manual/quickstart ARE).

---

## 7. SemVer + naming — **ACCURATE**

- New top-level subcommand = additive → **MINOR**: toolkit **v0.42.0 → v0.43.0**;
  paired GUI **v0.24.0** (prompt target).
- **`restore` is FREE** — no collision: not in `enum Command`, not in
  `cmd/mod.rs`, not a `gui-schema` subcommand (28 enumerated, none `restore`),
  not in `cli-subcommands.list`, not a `## ` manual section (only incidental
  prose "restore the..." at `41-mnemonic.md:2833`). `recover` also free.
- Phase-6 release checklist (MEMORY feedback): bump `Cargo.toml` + stage
  `Cargo.lock`; both README `<!-- toolkit-version: X -->` markers
  (`readme_version_current` guard); `scripts/install.sh` TAG self-pin
  (install-pin-check); CHANGELOG; FOLLOWUP Status flips; GUI-schema iff flag change.

---

## Cross-cutting observations

1. **Reuse surface is overwhelmingly library-callable — LOW refactor risk.** Every
   load-bearing helper (`derive_bip32_from_entropy`, `resolve_slots`,
   `build_descriptor_string`, `render_address_from_xpub`, the
   `WalletFormatEmitter`/`EmitInputs`/`CheckedDescriptor` trio, `read_stdin_passphrase`,
   `resolve_env_var_sentinel`, `worst_class_on_stdout`, `md_codec::chunk::reassemble`,
   `mk_codec::decode`, `xpub_to_65`) is `pub`/`pub(crate)` and already invoked
   cross-module. **No CLI-only-handler refactor is required.** This is an
   orchestration + output layer, as scoped.
2. **`restore` largely re-assembles `convert` + `addresses` + `export-wallet` +
   `verify-bundle` cross-check.** Strong reuse → small citation surface → likely
   1-2 architect rounds per the smaller-cycle-scope lesson. The 4-feature/4-phase
   shape below is at the upper end; if scoped tight (single-sig core only as P1)
   each phase is independently shippable.
3. **`--from <node>=<value>` model:** mirror `convert`'s `parse_from_input` +
   `NodeType` (`convert.rs:74-161`) — restore should accept
   `ms1|phrase|entropy|seedqr` primary nodes; `@env:`/`-` resolution via
   `resolve_env_var_sentinel`/`read_stdin_to_string`. Passphrase via
   `--passphrase`/`--passphrase-stdin`/`@env` with the single-stdin mutex
   (`convert.rs:798-813`). Reuse, don't reinvent.
4. **Mismatch policy maps cleanly:** reference present (mk1 slots /
   `--expect-fingerprint` / `--expect-xpub`) → HARD ERROR exit 4 (new
   `RestoreMismatch` variant); no reference → exit 0 + loud UNVERIFIED stderr
   banner. The fingerprint oracle = `DerivedAccount.master_fingerprint`.
5. **Watch-only-out invariant** keeps restore inside the no-signing boundary
   (MEMORY `feedback_no_signing_read_only_derivation_boundary`): emit xpub /
   fingerprint / addresses / concrete descriptor; NEVER xpriv/wif. Enforce with
   the `WatchOnly` OutputClass advisory + `is_argv_secret_bearing` JSON redaction.
6. **Multisig auto-detect** ("which slot the user's xpub fills") is a solved
   pattern at `verify_bundle.rs:2189-2208` (byte-match against `desc.tlv.pubkeys`,
   positional fallback) — lift it, don't re-derive.
7. **Codec deps are crates.io now** — CLAUDE.md "git deps" prose is stale; SPEC
   should cite crates.io versions (ms/mk 0.4.0, md 0.35) and re-grep
   md_codec/mk_codec API names against the pinned crate source at write time
   (per `feedback_verify_cited_apis_against_docs_rs`).

---

## Recommended decomposition + sizing

**Mandatory R0 gate before ANY code (CLAUDE.md): brainstorm spec + plan-doc must
pass opus R0 to 0C/0I; re-dispatch after every fold.**

- **P1 — single-sig core.** `cmd/restore.rs` + `RestoreArgs` + `Command::Restore`
  wiring (`main.rs`/`cmd/mod.rs`). `--from ms1|phrase|entropy|seedqr` +
  `--passphrase{,-stdin}` + `@env`/`-`. Derive the 4 BIP types
  (`derive_bip32_from_entropy` × {bip44,49,84,86}); lead with master fingerprint
  + first receive addr(s) (`render_address_from_xpub`); emit concrete
  descriptor(s) (`build_descriptor_string` single-sig). `--expect-fingerprint` /
  `--expect-xpub` reference → exit-4 mismatch (NEW `RestoreMismatch` variant) /
  loud UNVERIFIED banner. Text output. Toolkit-side `cli_gui_schema.rs` 28→29 fix.
  *Sizing: medium — most of the surface; the error-variant insert lands here.*
- **P2 — multisig cosigner + cross-check.** `--from` (own seed) + md1 template
  (`--md1`/`--descriptor`) + other cosigners' `--mk1` (or `--slot @N.xpub=`).
  `md_codec::chunk::reassemble` → derive own xpub → auto-detect slot
  (`tlv.pubkeys` byte-match) → cross-check all references → emit resolved concrete
  descriptor via `build_descriptor_string`. Mismatch → exit 4. *Sizing: medium —
  the load-bearing risk is md1→template resolution; recon route 1 keeps it on
  the verified `build_descriptor_string` path.*
- **P3 — import formats + `--json`.** `--format <CliExportFormat>` payload via the
  `WalletFormatEmitter` dispatch (`DescriptorEmitter` default; reuse the
  `export_wallet.rs:506-561` match block). Structured `--json` with seed redaction
  (`is_argv_secret_bearing`). `--json` wire-shape is NOT auto-gated → coordinate
  GUI consumer manually. *Sizing: small-medium.*
- **P4 — docs + GUI lockstep.** Manual `## mnemonic restore` (`41-mnemonic.md`) +
  `cli-subcommands.list` entry + `30-workflows/35-recovery-paths.md` recipe. GUI
  `SUBCOMMANDS` entry + `RESTORE_FLAGS` + `secret` bools + pin bumps
  (Cargo + pinned-upstream) → GUI v0.24.0. Release prep (versions, READMEs,
  install.sh pin, CHANGELOG, FOLLOWUP flips). *Sizing: small but checklist-heavy.*

## Lockstep flags (must-not-forget)

- [ ] `tests/cli_gui_schema.rs` 28→29 + add `"restore"` (alpha) — **breaks on P1.**
- [ ] GUI `src/schema/mnemonic.rs` `SUBCOMMANDS` + `RESTORE_FLAGS` (paired PR, GUI v0.24.0).
- [ ] GUI `FlagSchema.secret` mirror — only if a NEW literal secret flag name is added (then also `secrets.rs:49-64` + test).
- [ ] GUI pins: Cargo `mnemonic-toolkit.tag` + `pinned-upstream.toml [mnemonic].tag` (pin_coherence).
- [ ] Manual `41-mnemonic.md` `## mnemonic restore` + `cli-subcommands.list` (flag-coverage lint).
- [ ] Manual `30-workflows/35-recovery-paths.md` restore recipe.
- [ ] `error.rs` NEW `RestoreMismatch` variant in enum + exit_code + kind + message (alpha-insert each).
- [ ] Phase-6: Cargo.toml→0.43.0 + stage Cargo.lock + 2 README markers + install.sh TAG + CHANGELOG + FOLLOWUP flips.
