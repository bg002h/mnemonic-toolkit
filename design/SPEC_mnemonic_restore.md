# `mnemonic restore` — Brainstorm SPEC (single-sig; v0.43.0)

**Goal:** a new top-level `mnemonic restore` subcommand that takes **secret seed material + an optional BIP-39 passphrase** and emits a **watch-only "restore document"** (verification block + optional importable payload) to facilitate restoring a wallet on a PC. Secret-in → public-out (read-only derivation; **NO signing** — `feedback_no_signing_read_only_derivation_boundary`).

**Scope this cycle = SINGLE-SIG only.** The multisig-cosigner half is **deferred** to a follow-on cycle (SPEC R0 round-0 C1: the md1→concrete-descriptor route is not implementable from the cited APIs and needs its own R0 — see §11). The user asked for both scopes; this sequences multisig rather than dropping it (FOLLOWUP `restore-multisig-cosigner-scope`).

**Source SHA:** citations grep-verified against **base `6566941`** (toolkit v0.42.0); SPEC committed at `7dfba5c`+. Sibling codecs are **crates.io** pins: `ms-codec 0.4.0`, `mk-codec 0.4.0`, `md-codec 0.35` (CLAUDE.md "git deps" prose is stale). Branch `mnemonic-restore`. Recon: `cycle-prep-recon-mnemonic-restore.md`. R0-round-0 review: `design/agent-reports/restore-spec-R0-review.md`.

**SemVer:** new top-level subcommand = additive → toolkit **v0.43.0** + paired GUI **v0.24.0**.

---

## 1. Scope

- **Single-sig restore (this cycle):** own seed (`ms1`/`phrase`/`entropy`/`seedqr`) + optional passphrase → the 4 BIP single-sig wallet types (BIP-44/49/84/86), or one via `--template`.
- **One artifact**, leading with the **master fingerprint** (the passphrase-correctness oracle) + first receive address(es); then the concrete descriptor(s); `--json` structured; `--format <export-format>` adds the importable payload (single `--template` required — see §3.5/I1).
- **Mismatch policy (user-locked): hard-fail by default, explicit override.**
  - **Reference present** (`--expect-fingerprint`/`--expect-xpub`) and derived material does **not** match → **HARD ERROR, exit 4** (`RestoreMismatch`); the verification block prints derived-vs-expected + `✗ MISMATCH`; **no descriptors emitted**.
  - **`--allow-mismatch`** override → emit the descriptors the supplied passphrase produced under a loud `✗ MISMATCH (overridden)` banner, exit 0.
  - **No reference at all** → emit, with a loud `UNVERIFIED` stderr banner + the fingerprint front-and-center.
- **Watch-only-out invariant:** restore emits xpub / fingerprint / addresses / concrete descriptor only. It MUST NEVER emit `account_xpriv` / WIF / any private material (even though `DerivedAccount` carries `account_xpriv`). Enforced by the `WatchOnly` OutputClass advisory + `--json` redaction (`is_argv_secret_bearing`) + a negative test (no `xprv`/`tprv` token in any output).

---

## 2. CLI surface — `RestoreArgs` (clap::Args)

`run` mirrors `convert::run` (`convert.rs:737`): `pub fn run<R: Read, W: Write, E: Write>(args, stdin, stdout, stderr, no_auto_repair: bool) -> Result<u8, ToolkitError>`.

**Seed input (required, secret):**
- `--from <node=value>` — node ∈ `{ms1, phrase, entropy, seedqr}` (seed-bearing only). **I2 fold:** any other node (xpub/xprv/wif/…) → `ToolkitError::BadInput` (exit 1), pattern `addresses.rs:223`. Value supports `@env:VAR` (`resolve_env_var_sentinel`) and `-` (stdin, `read_stdin_to_string`).
- `--passphrase <P>` / `--passphrase-stdin` — BIP-39 extension passphrase; `--passphrase` accepts `@env:VAR`. Mutex identical to `convert` (`convert.rs:798-813`): `--passphrase` XOR `--passphrase-stdin`; `--passphrase-stdin` XOR any `--from <node>=-`. Both secrets can ride `@env:` to stay off argv (`addresses.rs:153,162`).
- `--language <L>` (phrase/seedqr; default english); `--network <N>` (default mainnet); `--account <n>` (default 0).

**Single-sig selection:**
- `--template <T>` — `Option<CliTemplate>`; `None` (default) = all four single-sig types (bip44/49/84/86). **I-B fold:** a multisig template (any `CliTemplate::is_multisig()` variant, `template.rs:47`) → `ToolkitError::BadInput` (exit 1): "restore is single-sig only; --template ∈ {bip44,bip49,bip84,bip86}".

**Reference / verification:**
- `--expect-fingerprint <hex>` — 8 lowercase hex; master fingerprint MUST equal → else exit 4.
- `--expect-xpub <xpub>` — account xpub MUST equal (requires `--template Some`) → else exit 4.
- `--allow-mismatch` — override: proceed despite reference mismatch (loud banner, exit 0).

**Output:**
- `--format <CliExportFormat>` — importable payload via an `export-wallet` emitter (11-value enum, `export_wallet.rs:22-46`). **I1/I-A fold: `--format` REQUIRES `--template Some` (one type)** — emitters are one-descriptor-in/one-out; `--format` with `--template None` (the all-4 default) → `ToolkitError::ModeViolation` **exit 2** (NOT `BadInput`/exit 1 — code pinned). Default (no `--format`) = descriptor inline in the verify-doc.
- `--json` — structured (seed redacted). `--output <FILE>`. `--count <n>` — first-receive addresses per type (default 1).

---

## 3. Behavior / control flow (single-sig)

### 3.1 Input resolution
Resolve `--from` value (`@env:`/`-`/literal) + passphrase channel under the convert mutex rules. Convert the seed node → entropy + derive-language:
- phrase / seedqr → `Mnemonic::to_entropy`; entropy → raw bytes; derive-language = `--language`.
- **ms1 (I-C fold):** `slot_ms1::resolve_ms1_slot(value, flag_language=--language, idx) -> Ms1SlotResolution { entropy, derive_language, emit_language }` (`slot_ms1.rs:37`). ms1 `Mnem` payloads carry the wordlist language ON THE WIRE (drives BIP-39 PBKDF2), so a bare `ms_codec::decode` is wrong for non-English `mnem` cards. `resolve_ms1_slot` applies wire-wins / refuse-on-`--language`-conflict (`SlotInputViolation` exit 2). Use `res.derive_language` downstream. (Builds on `project_ms_mnem_v0_2_shipped` / `project_ms1_slot_v0_41_0_shipped`.)

`mlock::pin_pages_for` the secret buffers + passphrase (pattern `convert.rs:841-847`). `secret_in_argv_warning` if a secret was passed inline on argv (pattern `addresses.rs:126-146`).

### 3.2 Derivation
For each selected template T (all 4, or the single `--template`):
1. `derive_slot::derive_bip32_from_entropy(entropy, passphrase, derive_language, network, T, account)` → `DerivedAccount` (`derive_slot.rs:42`) — `derive_language` = `res.derive_language` for ms1, else `--language`. Use `master_fingerprint` + `account_xpub` only — **never `account_xpriv`**.
2. First receive address(es): derive `m/0/0..0/n-1` children of `account_xpub`; `address_render::render_address_from_xpub(secp, &child, script_type, network)` (`address_render.rs:18`). **M-c:** there is no in-tree `CliTemplate→ScriptType` helper (only the forward `template_for(ScriptType)`, `addresses.rs:95`); restore hand-writes the 4-way inverse map (bip44→`P2pkh`, bip49→`P2shP2wpkh`, bip84→`P2wpkh`, bip86→`P2tr`). `--count` controls n.
3. Concrete descriptor: build a single-element `ResolvedSlot { xpub: account_xpub, fingerprint: master_fingerprint, path: T's origin, .. }` (so the origin renders `[mfp/84'/0'/0']`, not `[00000000/…]` — `key_origin_str`, `pipeline.rs:33`); `wallet_export::build_descriptor_string(T, &[slot], 1, network, account, None)` (`pipeline.rs:18`) → `<descriptor>#<checksum>`.

The master fingerprint is computed once (path-independent — identical across all 4 types). The reference gate (§3.4) runs on it.

### 3.3 (multisig — DEFERRED, see §11)

### 3.4 Mismatch policy
- **Reference comparison:** `--expect-fingerprint` vs `master_fingerprint`; `--expect-xpub` vs `account_xpub` (single `--template`).
- **Match** → emit normally.
- **Mismatch + no `--allow-mismatch`** → `Err(ToolkitError::RestoreMismatch { reference, derived, expected, slot: None })` → exit 4; message prints derived-vs-expected; **no descriptors**.
- **Mismatch + `--allow-mismatch`** → emit under `✗ MISMATCH (overridden)` stderr banner, exit 0.
- **No reference** → emit + loud `UNVERIFIED` stderr banner ("no --expect-fingerprint/--expect-xpub supplied; verify the fingerprint above against your records"), exit 0.

### 3.5 Output document
- **Text (default):** header `master fingerprint: <fp>  (passphrase: applied|none)` + a CONFIRM line; per-type block `descriptor:` + `first recv:`.
- **`--format <fmt>` (requires single `--template`):** emit the payload via the `WalletFormatEmitter` dispatch (`export_wallet.rs:507-561`; `DescriptorEmitter` natural default).
- **`--json`:** `{ master_fingerprint, passphrase_applied, network, verification: {status, expected?, derived?}, wallets: [{type, descriptor, first_addresses[]}], import_payload? }`. Seed NEVER echoed. **I3 fold:** restore's mismatch surfaces via `ToolkitError::message()` only (NOT the `details()` JSON-error envelope), so the error obligation is the 3 forced-exhaustive blocks.
- **Advisory:** emit the `WatchOnly` OutputClass stderr line (`secret_advisory::emit_output_class_advisory`, `:97`; pattern `addresses.rs:258-261`).

---

## 4. Reuse APIs (grep-verified @ 6566941; cosmetics M3-M5 corrected)

| Need | API | Cite |
|---|---|---|
| seed+passphrase → account xpub + MASTER fingerprint | `derive_slot::derive_bip32_from_entropy(entropy:&[u8], passphrase:&str, language:Bip39Language, network:CliNetwork, template:CliTemplate, account:u32) -> Result<DerivedAccount,_>` | `derive_slot.rs:42` |
| derived bundle | `DerivedAccount { entropy:Zeroizing<Vec<u8>>, master_fingerprint:Fingerprint, account_xpub:Xpub, account_xpriv:Xpriv (DO NOT EMIT), account_path, _entropy_pin }` | `derive.rs:23-39` |
| ms1 → entropy + wire-language | `slot_ms1::resolve_ms1_slot(value, flag_language, idx) -> Ms1SlotResolution { entropy, derive_language, emit_language }` | `slot_ms1.rs:37` |
| reject multisig `--template` | `CliTemplate::is_multisig() -> bool` | `template.rs:47` |
| template+slots → concrete descriptor (`#csum`) | `wallet_export::build_descriptor_string(template, slots:&[ResolvedSlot], k:u8, network, account, Option<TaprootInternalKey>) -> Result<String,_>` | `pipeline.rs:18` |
| `ResolvedSlot` origin → `[fp/path]` | `ResolvedSlot { xpub, fingerprint, path, .. }`; `key_origin_str` | `synthesize.rs:642`; `pipeline.rs:33` |
| import payload | `WalletFormatEmitter`/`EmitInputs`/`CheckedDescriptor`/`DescriptorEmitter`; `CliExportFormat` (11) | `wallet_export/mod.rs:397,420,466`; `export_wallet.rs:22-46` |
| first receive address | `address_render::render_address_from_xpub<C>(secp, child:&Xpub, script_type:ScriptType, network) -> String` | `address_render.rs:18` |
| secret stdin | `convert::read_stdin_passphrase` (NULL-preserving), `convert::read_stdin_to_string` (trims) | `convert.rs:719,706` |
| env sentinel | `env_sentinel::resolve_env_var_sentinel(value, flag) -> Result<String,_>` | `env_sentinel.rs:56` |
| mlock | `mlock::pin_pages_for(&[u8]) -> PinnedPageRange` | `mlock.rs:90` |
| advisory | `secret_advisory::{worst_class_on_stdout, emit_output_class_advisory, secret_in_argv_warning}`; `OutputClass::{Template,WatchOnly,PrivateKeyMaterial}` | `secret_advisory.rs:40,83,97` |
| json redaction | `NodeType::is_argv_secret_bearing` | `convert.rs:117` |

Every P1 helper is `pub`/`pub(crate)` + cross-module-invoked — no refactor. (NB: the multisig-only `extract_multisig_threshold`, `bundle.rs:1015`, IS private — deferred with §11.)

---

## 5. Error variant
Add `ToolkitError::RestoreMismatch { reference: &'static str, derived: String, expected: String, slot: Option<u8> }` → **exit 4** (verify/mismatch tier, alongside `BundleMismatch`/`ImportWalletSeedMismatch`). `enum` is `#[non_exhaustive]` (non-breaking). **CLAUDE.md alphabetical rule:** insert after `RepairShortCircuit`, before `SilentPayment`, in the enum AND the THREE forced-exhaustive blocks: `exit_code` (`error.rs:471`), `kind` (`:529`), `message` (`:588`) (M-a: anchors re-grep at plan-write — off-by-1 from snapshot). **I3:** the `details()` block (`:775`, `_=>None`) is NOT touched — restore mismatch uses `message()` only (no JSON-error envelope). `message()` is `restore:`-prefixed (no reuse of `import-wallet:`/`bundle:` strings). UNVERIFIED + usage errors reuse `BadInput`(1)/`ModeViolation`(2).

---

## 6. Subcommand wiring + gui-schema self-test
- `cmd/mod.rs`: `pub mod restore;` (slot after `repair`, before `silent_payment`; the list is alpha-ish but already drifted — M6: don't re-sort).
- `main.rs:~90` enum `Command` + `~:153` dispatch (M-d: both are feature-clustered, NOT alpha — anchors approximate, place the new variant/arm anywhere; don't re-sort the existing order). `Command::Restore(cmd::restore::RestoreArgs)`; arm `Command::Restore(args) => cmd::restore::run(args, stdin, stdout, stderr, cli.no_auto_repair)` (returns `Result<u8,_>` — no `.map`).
- gui-schema auto-reflects (zero `gui_schema.rs` edits).
- **TOOLKIT self-test `tests/cli_gui_schema.rs` BREAKS** — add `"restore"` to the name vec (alpha: after `repair`, before `seed-xor-combine`) and bump the "28" count at **`:74` and `:108`** (M2). Lands in P1.

---

## 7. Lockstep obligations
**GUI (`/scratch/code/shibboleth/mnemonic-gui`, paired v0.24.0):** one `SubcommandSchema` in `SUBCOMMANDS` (`src/schema/mnemonic.rs:3191`) + `const RESTORE_FLAGS`. Secret-flag projection: `flag_is_secret` (`secrets.rs:49-64`) already classifies `--passphrase`/`--passphrase-stdin` secret; `--from` is value-dependent (no new entry — restore adds no new literal secret flag name). Pins: bump Cargo `mnemonic-toolkit.tag` + `pinned-upstream.toml [mnemonic].tag` → v0.43.0 (`pin_coherence`); `schema_mirror` flag-NAME parity once SUBCOMMANDS entry added.
**Manual:** `## mnemonic restore` in `41-mnemonic.md` (Synopsis/Flags/Worked example, every flag); add `mnemonic restore` to `docs/manual/tests/cli-subcommands.list` (flag-coverage lint, `lint.sh:62-96`); restore recipe in `30-workflows/35-recovery-paths.md`.

---

## 8. Phasing (each: TDD, per-phase opus R0 to 0C/0I, persist to `design/agent-reports/`)
- **P1 — single-sig core.** `cmd/restore.rs` + `RestoreArgs` + `Command::Restore` + `cmd/mod.rs` wiring. `--from {ms1,phrase,entropy,seedqr}` (+ non-seed rejection) + `--passphrase{,-stdin}` + `@env`/`-` + `--language`/`--network`/`--account` + `--template`. Derive 4 types; lead with fingerprint + first recv addr(s); concrete descriptor(s); `--expect-fingerprint`/`--expect-xpub` → exit-4 `RestoreMismatch` (NEW variant) / `--allow-mismatch` / UNVERIFIED banner. Text output. `cli_gui_schema.rs` 28→29.
- **P2 — import formats + `--json`.** `--format <CliExportFormat>` (requires single `--template`) via the emitter dispatch; `--json` + redaction; `--output`; `--count`.
- **P3 — docs + GUI lockstep + release.** Manual section + `cli-subcommands.list` + recovery recipe; GUI `SUBCOMMANDS` + `RESTORE_FLAGS` + pin bumps (v0.24.0); Phase-6 release prep (v0.43.0).

---

## 9. Tests (per phase, TDD-first)
- **P1:** single-sig restore from each of ms1/phrase/entropy/seedqr; fingerprint + descriptor + first addr **exact** for abandon×11+about (no-pp fp `73c5da0a` — asserted in-tree `cli_export_wallet.rs:27`; **I5: TREZOR-pp fp must be derived+confirmed by the implementer before baking — controller pre-confirmed `b4e3f5ed` at runtime, but re-derive in-test per `feedback_recapture_golden_only_when_current_correct`**); all-4 default vs `--template` single; `--expect-fingerprint` match→0 / mismatch→exit4 / mismatch+`--allow-mismatch`→0+banner; no-reference→UNVERIFIED; watch-only advisory present; **negative: NO `xprv`/`tprv` token in any output**; `@env:`+`--passphrase-stdin` channel; stdin-mutex rejection; non-seed `--from` → BadInput exit 1; **I-B: multisig `--template` (e.g. wsh-sortedmulti) → BadInput exit 1**; **I-C: a non-English ms1 `mnem` card derives the wire-language seed (not english) + `--language` conflicting with the wire → SlotInputViolation exit 2**.
- **P2:** each `--format` payload (with `--template`); **I-A: `--format` + `--template None` (all-4 default) → ModeViolation exit 2**; `--json` shape + seed-redaction negative-assert; `--output`; `--count`.
- **P3:** `make -C docs/manual audit` EXIT=0; `cli_gui_schema.rs` 29 green; GUI `schema_mirror`+`pin_coherence`+`secret_drift` green at `cargo +1.94.0`.
- **Regression each phase:** `cargo test -p mnemonic-toolkit --no-fail-fast` 0 fail; `clippy --all-targets -D warnings`.

---

## 10. SemVer + release checklist
MINOR → toolkit **v0.43.0** + GUI **v0.24.0**. Phase-6 (P3): `Cargo.toml` 0.42.0→0.43.0 + stage `Cargo.lock`; BOTH README `<!-- toolkit-version: -->` markers + both `Status:` prose lines; `CHANGELOG.md`; `scripts/install.sh` self-pin TAG; FOLLOWUP flips; `readme_version_current` PASS. GUI: 0.23.0→0.24.0 + CHANGELOG + banner + pins.

---

## 11. DEFERRED — multisig-cosigner scope (follow-on SPEC + R0)

**Why deferred (R0 C1):** the md1→concrete-descriptor route is not implementable from the originally-cited APIs. `template_from_descriptor` (`wallet_export/mod.rs:262`) takes a miniscript `MsDescriptor`, not `md_codec::Descriptor`; `md_codec::Descriptor` has no `Display`; `to_miniscript_descriptor` (`md-codec-0.35.0/to_miniscript.rs:53`) errors `MissingPubkey` on a template-only md1; `extract_multisig_threshold` (`bundle.rs:1015`) is private. The production md1→concrete path (`bundle_run_unified_descriptor`, `bundle.rs:1138`) takes a `--descriptor` STRING through lex/resolve/parse/bind. **The follow-on SPEC must choose + R0 one of:** (a) `--descriptor '<@N-template-string>'` input only (reuse the verified bundle bind path), with `--md1` cross-check-only; (b) derive `CliTemplate` from md1 **policy params** (script-type + k via `extract_multisig_threshold(&d.tree)` [bump to pub(crate)] + `d.n`) → `build_descriptor_string`; or (c) `to_miniscript_descriptor` with its wallet-policy-mode constraint spelled out. Plus I4 (wallet-policy-vs-template-only `tlv.pubkeys` auto-detect branch), the cosigner cross-check, and `--cosigner @N=mk1|xpub` (decode mk1 via `mk_codec::decode`; no new slot subkey). FOLLOWUP: `restore-multisig-cosigner-scope`. Single-sig `restore` ships first (v0.43.0); multisig is additive (v0.44.0) — one more GUI `RESTORE_FLAGS` delta under the paired-PR rule.

---

## 12. R0 fold log
**Round 0** (RED 1C/5I) → **C1** descope multisig to §11 (deferred). **I1** `--format` requires single `--template`. **I2** non-seed `--from` → BadInput exit 1. **I3** mismatch via `message()` only. **I5** implementer re-derives TREZOR-pp fp. **M1** SHA prose. **M2** cli_gui_schema `:74/:108`. **M3** `extract_multisig_threshold` private. **M4** `entropy: Zeroizing`. **M5** `resolve_slots` `&'static str`. **M6** `cmd/mod.rs` drift. (I4 deferred with multisig.)
**Round 1** (RED 0C/3I — 0 leakage, single-sig design proven) → **I-A** `--format` + `--template None` → `ModeViolation` **exit 2** (NOT BadInput); `--template` becomes `Option`. **I-B** multisig `--template` → `BadInput` exit 1 via `is_multisig()`. **I-C** ms1 via `slot_ms1::resolve_ms1_slot` (preserve `mnem` wire-language; use `derive_language`). **M-a** error anchors `471/529/588/775`. **M-b** `DerivedAccount` `derive.rs:23-39`. **M-c** hand-write `CliTemplate→ScriptType` inverse map. **M-d** `main.rs` enum/dispatch feature-clustered — don't re-sort. Re-grep at plan-write; re-dispatch R0 round 2.
