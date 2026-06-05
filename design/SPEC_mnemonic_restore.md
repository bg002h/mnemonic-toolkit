# `mnemonic restore` — Brainstorm SPEC

**Goal:** a new top-level `mnemonic restore` subcommand that takes **secret seed material + an optional BIP-39 passphrase** and emits a **watch-only "restore document"** (one artifact: a verification block + an importable payload) to facilitate restoring a wallet on a PC. Secret-in → public-out (read-only derivation; **NO signing** — stays inside `feedback_no_signing_read_only_derivation_boundary`).

**Source SHA:** all citations grep-verified against `origin/master == HEAD == 6566941` (toolkit v0.42.0). Sibling codecs are **crates.io** pins (NOT git deps): `ms-codec 0.4.0`, `mk-codec 0.4.0`, `md-codec 0.35`. Branch `mnemonic-restore`. Recon: `cycle-prep-recon-mnemonic-restore.md`.

**SemVer:** new top-level subcommand = additive → toolkit **v0.43.0** + paired GUI **v0.24.0**.

---

## 1. Scope (locked with the user)

- **Both restore cases:**
  - **Single-sig** — own seed (`ms1`/`phrase`/`entropy`/`seedqr`) + passphrase → the 4 BIP single-sig wallet types (BIP-44/49/84/86).
  - **Multisig cosigner** — user supplies the `md1` template + the *other* cosigners' `mk1` cards (or raw xpubs) + their own seed; restore derives their account xpub, auto-detects which `@N` slot it fills, cross-checks, and emits the resolved concrete descriptor.
- **One artifact**, leading with the **master fingerprint** (the passphrase-correctness oracle) + first receive address(es); then the concrete descriptor(s); `--json` structured; `--format <export-format>` adds the importable payload.
- **Mismatch policy (user-locked): hard-fail by default, explicit override.**
  - **Reference present** (multisig cosigner slots, or single-sig `--expect-fingerprint`/`--expect-xpub`) and the derived material does **not** match → **HARD ERROR, exit 4** (`RestoreMismatch`), the verification block prints derived-vs-expected + `✗ MISMATCH`, **no descriptors emitted**.
  - **`--allow-mismatch`** override → proceed anyway: emit the descriptors the supplied passphrase produced under a loud `✗ MISMATCH (overridden)` banner, exit 0. The dangerous case is never silent; the candidate-trying workflow still has a door.
  - **No reference at all** (bare seed+passphrase, nothing to check) → emit, with a loud `UNVERIFIED` stderr banner + the fingerprint front-and-center.
- **Watch-only-out invariant:** restore emits xpub / fingerprint / addresses / concrete descriptor only. It MUST NEVER emit `account_xpriv` / WIF / any private material (even though `DerivedAccount` carries `account_xpriv`). Enforced by the `WatchOnly` OutputClass advisory + `--json` redaction via `is_argv_secret_bearing`.

---

## 2. CLI surface — `RestoreArgs` (clap::Args)

`run` signature mirrors `convert::run` (`convert.rs:737`): `pub fn run<R: Read, W: Write, E: Write>(args: &RestoreArgs, stdin, stdout, stderr, no_auto_repair: bool) -> Result<u8, ToolkitError>`.

**Seed input (required, secret):**
- `--from <node=value>` — seed source; node ∈ `{ms1, phrase, entropy, seedqr}` (seed-bearing only; an already-derived xpub/xprv needs no passphrase so is out of scope). Value supports `@env:VAR` (`resolve_env_var_sentinel`) and `-` (stdin, `read_stdin_to_string`). Mirrors `convert`/`addresses` `--from` parsing.
- `--passphrase <P>` / `--passphrase-stdin` — BIP-39 extension passphrase. `--passphrase` accepts `@env:VAR`. Mutex rules identical to `convert` (`convert.rs:798-813`): `--passphrase` XOR `--passphrase-stdin`; `--passphrase-stdin` XOR any `--from <node>=-` (single stdin per invocation).
- `--language <L>` — BIP-39 wordlist for `phrase`/`seedqr` (default english).
- `--network <N>` — mainnet (default) / testnet / signet / regtest.
- `--account <n>` — BIP-32 account index (default 0).

**Single-sig selection:**
- `--template <T>` — restrict to one single-sig type; **default = all four (bip44/49/84/86)**.

**Multisig (P2):**
- `--md1 <STR>...` — the md1 template card string(s); decoded via `md_codec::chunk::reassemble`. OR
- `--descriptor <D>` — a bare `@N` template (e.g. `wsh(sortedmulti(2,@0,@1,@2))`) for power users.
- `--cosigner <@N=mk1|xpub>...` — the *other* cosigners' key cards. An `mk1` value is decoded via `mk_codec::decode` → xpub; an `xpub` value is used directly. (Restore-local parsing → internal `@N.xpub=` `ResolvedSlot`s; the shared `slot_input.rs` subkey set is **not** extended — there is no `mk1` slot subkey.) The user's own seed (`--from`) fills the one remaining slot, or — if all slots are given — restore confirms its derived xpub equals exactly one.
- `--my-slot <@N>` — force which slot is the user's (override auto-detect).

**Reference / verification:**
- `--expect-fingerprint <hex>` — single-sig hard-gate: master fingerprint MUST equal this (8 lowercase hex). Mismatch → exit 4.
- `--expect-xpub <xpub>` — single-sig hard-gate against a specific account xpub (for a given `--template`).
- `--allow-mismatch` — override: proceed despite any reference mismatch (loud banner, exit 0).

**Output:**
- `--format <CliExportFormat>` — emit the importable payload in an `export-wallet` format (`bitcoin-core|bip388|descriptor|sparrow|…`, the existing 11-value enum, `export_wallet.rs:22-46`). Default (no `--format`) = the descriptor inline in the verify-doc.
- `--json` — structured output (seed material redacted).
- `--output <FILE>` — write to a file instead of stdout.
- `--count <n>` — first-receive addresses to show per type (default 1).

---

## 3. Behavior / control flow

### 3.1 Input resolution
Resolve `--from` value (`@env:`/`-`/literal) and the passphrase channel under the convert mutex rules. Convert the seed node → entropy (`Mnemonic::to_entropy` for phrase/seedqr; raw for entropy; `ms_codec` decode for ms1 → entropy, preserving wire language where applicable). `mlock::pin_pages_for` the secret buffers; the passphrase too (pattern `convert.rs:841-847`). Emit a `secret_in_argv_warning` if the value/passphrase was passed inline on argv (pattern `addresses.rs:126-146`).

### 3.2 Single-sig path (P1)
For each selected template T ∈ {bip44, bip49, bip84, bip86} (or the single `--template`):
1. `derive_slot::derive_bip32_from_entropy(entropy, passphrase, language, network, T, account)` → `DerivedAccount` (`derive_slot.rs:42`). Use `master_fingerprint` + `account_xpub` only.
2. First receive address(es): derive `m/0/0..0/n-1` children of `account_xpub`, render via `address_render::render_address_from_xpub(secp, &child, script_type, network)` (`address_render.rs:18`).
3. Concrete descriptor: build a single-element `ResolvedSlot` from `account_xpub` + `master_fingerprint` + T's origin path, then `wallet_export::build_descriptor_string(T, &[slot], 1, network, account, None)` (`pipeline.rs:18`) → `<descriptor>#<checksum>`.

The master fingerprint is computed once (it is path-independent — same for all four types). Reference gate (§3.4) runs on it.

### 3.3 Multisig cosigner path (P2)
1. Obtain the template: `--md1` → `md_codec::chunk::reassemble(&md1_strs)` → `md_codec::Descriptor`; derive `CliTemplate` (`wallet_export/mod.rs:262 template_from_descriptor`) + threshold k (`extract_multisig_threshold(&d.tree)`, used at `bundle.rs:1060`). Or `--descriptor` template string directly.
2. Build `ResolvedSlot`s for the provided `--cosigner @N=` entries (decode mk1→xpub as needed). Derive the user's own account xpub from `--from`+passphrase at the multisig path.
3. **Auto-detect the user's slot** (lift `verify_bundle.rs:2189-2208`): `synthesize::xpub_to_65(&own_xpub)` byte-matched against `desc.tlv.pubkeys`; positional fallback. `--my-slot` overrides. Bind own xpub into its slot.
4. **Cross-check:** own derived xpub MUST appear at exactly one slot of the presented bundle (or fill the one unfilled slot). No match → reference mismatch (§3.4).
5. Emit the resolved concrete descriptor via `build_descriptor_string` (route 1 — keeps it on the verified path; no bundle-emit). Verification block reports `cosigner match: ✓ fills slot @N of <policy>`.

### 3.4 Mismatch policy
- Compute the reference comparison: single-sig vs `--expect-fingerprint`/`--expect-xpub`; multisig vs the presented bundle (does own xpub fill a slot?).
- **Match** → emit normally.
- **Mismatch + no `--allow-mismatch`** → `Err(ToolkitError::RestoreMismatch { reference, derived, expected, slot })` → exit 4; the message prints derived-vs-expected; **no descriptors**.
- **Mismatch + `--allow-mismatch`** → emit descriptors under `✗ MISMATCH (overridden)` banner (stderr), exit 0.
- **No reference supplied at all** → emit + loud `UNVERIFIED` banner (stderr): "no --expect-fingerprint/--expect-xpub/cosigner reference supplied; verify the fingerprint above against your records." exit 0.

### 3.5 Output document
- **Text (default):** header (`master fingerprint: <fp>  (passphrase: applied|none)` + a CONFIRM line); per-type (single-sig) or single (multisig) block with `descriptor:` + `first recv:`. Multisig adds the `cosigner match:` line.
- **`--format <fmt>`:** append/emit the importable payload via the `WalletFormatEmitter` dispatch (`export_wallet.rs:506-561` match block; `DescriptorEmitter` is the natural default). For single-sig all-4, `bitcoin-core` (importdescriptors array) is the one format that holds several descriptors in one artifact (per the v0.42.0 all-single-sig recipe note).
- **`--json`:** structured `{ master_fingerprint, passphrase_applied, network, verification: {...}, wallets: [{type, descriptor, first_addresses[...]}], import_payload? }`. Seed material NEVER echoed (`is_argv_secret_bearing` redaction, `convert.rs:117`). `--json` wire-shape is NOT auto-gated → coordinate the GUI consumer manually (paired-PR note).
- **Advisory:** emit the `WatchOnly` OutputClass stderr line (`secret_advisory::emit_output_class_advisory`, `secret_advisory.rs:97`; pattern `addresses.rs:258-261`).

---

## 4. Reuse APIs (grep-verified @ 6566941)

| Need | API | Cite |
|---|---|---|
| seed+passphrase → account xpub + MASTER fingerprint | `derive_slot::derive_bip32_from_entropy(entropy:&[u8], passphrase:&str, language:Bip39Language, network:CliNetwork, template:CliTemplate, account:u32) -> Result<DerivedAccount,_>` | `derive_slot.rs:42` |
| derived bundle | `DerivedAccount { entropy, master_fingerprint:Fingerprint, account_xpub:Xpub, account_xpriv:Xpriv (DO NOT EMIT), account_path, _entropy_pin }` | `derive.rs:22-36` |
| template+slots → concrete descriptor (`#csum`) | `wallet_export::build_descriptor_string(template, slots:&[ResolvedSlot], k:u8, network, account, taproot_internal_key:Option<_>) -> Result<String,_>` | `pipeline.rs:18` |
| import payload formats | `WalletFormatEmitter` / `EmitInputs` / `CheckedDescriptor` / `DescriptorEmitter`; `CliExportFormat` (11) | `wallet_export/mod.rs:397,420,466`; `export_wallet.rs:22-46` |
| first receive address | `address_render::render_address_from_xpub<C>(secp, child:&Xpub, script_type:ScriptType, network) -> String` | `address_render.rs:18` |
| slot binding | `cmd::bundle::resolve_slots(slots, template, network, account, language, passphrase, family) -> (Vec<ResolvedSlot>, Vec<(u8,&str)>)` | `bundle.rs:453` |
| multisig auto-detect / cross-check | `md_codec::chunk::reassemble`; `mk_codec::decode`; `synthesize::xpub_to_65(&Xpub)->[u8;65]` byte-match vs `desc.tlv.pubkeys` | `verify_bundle.rs:2189-2208`; `synthesize.rs:98` |
| md1 → template + k | `template_from_descriptor` (`mod.rs:262`); `extract_multisig_threshold(&d.tree)` (`bundle.rs:1060`) | — |
| secret stdin | `convert::read_stdin_passphrase` (NULL-preserving), `convert::read_stdin_to_string` (trims) | `convert.rs:719,706` |
| env sentinel | `env_sentinel::resolve_env_var_sentinel(value, flag) -> Result<String,_>` | `env_sentinel.rs:56` |
| mlock | `mlock::pin_pages_for(&[u8]) -> PinnedPageRange` | `mlock.rs:90` |
| advisory | `secret_advisory::{worst_class_on_stdout, emit_output_class_advisory, secret_in_argv_warning}`; `OutputClass::{Template,WatchOnly,PrivateKeyMaterial}` | `secret_advisory.rs:40,83,97` |

**No CLI-only-handler refactor required** — every helper is `pub`/`pub(crate)` and already cross-module-invoked.

---

## 5. Error variant

Add `ToolkitError::RestoreMismatch { reference: &'static str, derived: String, expected: String, slot: Option<u8> }` → **exit 4** (verify/mismatch tier, alongside `BundleMismatch`/`ImportWalletSeedMismatch`). `enum` is `#[non_exhaustive]` (non-breaking add). **CLAUDE.md alphabetical rule:** insert at the correct alpha position (`RestoreMismatch` after `RepairShortCircuit`, before `SilentPayment`) in the enum AND in all THREE exhaustive blocks: `exit_code` (`error.rs:516-518`), `kind`, `message`. (The three blocks aren't retro-sorted — `error-rs-retroactive-alphabetical-sort` FOLLOWUP — insert at the right local alpha slot.) `message()` is `restore:`-prefixed (no misleading reuse of `import-wallet:`/`bundle:` strings). UNVERIFIED + usage errors reuse existing variants (`BadInput`/`ModeViolation`/`SlotInputViolation`).

---

## 6. Subcommand wiring + gui-schema self-test

- `cmd/mod.rs`: add `pub mod restore;` (alpha).
- `main.rs:89-135`: add `Command::Restore(cmd::restore::RestoreArgs)`.
- `main.rs:153-192`: dispatch arm `Command::Restore(args) => cmd::restore::run(args, stdin, stdout, stderr, cli.no_auto_repair)`.
- `gui-schema` auto-reflects the new clap variant (zero `gui_schema.rs` edits).
- **TOOLKIT self-test `tests/cli_gui_schema.rs:72-110` BREAKS** — it `assert_eq!`s the full 28-name vec + "28" prose. Add `"restore"` (alpha: after `repair`, before `seed-xor-combine`) and bump 28→29 (the count comment at `:3,:37`). This lands in P1.

---

## 7. Lockstep obligations

**GUI (`/scratch/code/shibboleth/mnemonic-gui`, paired v0.24.0):**
- `src/schema/mnemonic.rs`: one `SubcommandSchema` entry in `SUBCOMMANDS` (`:3191`) + a `const RESTORE_FLAGS: &[FlagSchema]`. `allows_slots` per the multisig `--cosigner`/slot design.
- Secret-flag projection: `flag_is_secret` (`toolkit src/secrets.rs:49-64`) already classifies `--passphrase`/`--passphrase-stdin`/`--ms1` secret; `--from <node>=value` is value-dependent (not a flat secret flag), same as convert. **No new `flag_is_secret` entry** UNLESS restore adds a brand-new literal secret flag name — it does not (it reuses `--passphrase*` + `--from`). So GUI `FlagSchema.secret` mirrors the existing classification.
- Pins: bump Cargo `mnemonic-toolkit.tag` + `pinned-upstream.toml [mnemonic].tag` to v0.43.0 in lockstep (`pin_coherence`). `schema_mirror` (flag-NAME set-equality) must pass once the SUBCOMMANDS entry is added.

**Manual:**
- `## mnemonic restore` section in `docs/manual/src/40-cli-reference/41-mnemonic.md` (Synopsis/Flags/Worked example), every flag documented.
- Add `mnemonic restore` to `docs/manual/tests/cli-subcommands.list` (flag-coverage lint `lint.sh:62-96`).
- Restore recipe in `docs/manual/src/30-workflows/35-recovery-paths.md`.

---

## 8. Phasing (each phase: TDD, then per-phase opus R0 to 0C/0I, persist verbatim to `design/agent-reports/`)

- **P1 — single-sig core.** `cmd/restore.rs` + `RestoreArgs` + `Command::Restore` wiring + `cmd/mod.rs`. `--from {ms1,phrase,entropy,seedqr}` + `--passphrase{,-stdin}` + `@env`/`-` + `--language`/`--network`/`--account`. Derive the 4 BIP types; lead with master fingerprint + first recv addr(s); emit concrete descriptor(s). `--expect-fingerprint`/`--expect-xpub` reference → exit-4 `RestoreMismatch` (NEW variant lands here) / `--allow-mismatch` override / UNVERIFIED banner. Text output. `cli_gui_schema.rs` 28→29 fix.
- **P2 — multisig cosigner + cross-check.** `--md1`/`--descriptor` + `--cosigner @N=` + `--my-slot`. reassemble → derive own xpub → auto-detect slot (`tlv.pubkeys` byte-match) → cross-check → emit resolved descriptor. Mismatch → exit 4.
- **P3 — import formats + `--json`.** `--format <CliExportFormat>` payload via the `WalletFormatEmitter` dispatch; `--json` structured + seed redaction; `--output`; `--count`.
- **P4 — docs + GUI lockstep + release.** Manual section + `cli-subcommands.list` + recovery recipe; GUI `SUBCOMMANDS` + `RESTORE_FLAGS` + pin bumps (v0.24.0); Phase-6 release prep (v0.43.0).

(If R0/complexity pressure mounts, P1 is independently shippable as v0.43.0 with P2-P4 following — but the default is one coherent cycle so the GUI takes one SUBCOMMANDS entry + one pin bump.)

---

## 9. Tests (per phase, TDD-first)

- **P1:** single-sig restore from each of ms1/phrase/entropy/seedqr (fingerprint + descriptor + first addr exact for the abandon×11+about vector, w/ and w/o `TREZOR` passphrase — fingerprints `73c5da0a` vs `b4e3f5ed`); all-4 default vs `--template` single; `--expect-fingerprint` match→0, mismatch→exit4, mismatch+`--allow-mismatch`→0+banner; no-reference→UNVERIFIED banner; watch-only advisory present; NO xpriv/wif in any output (grep the output for `xprv`/`tprv`/`L`/`K` WIF le@ negative assertion); `@env:`+`--passphrase-stdin` channel; stdin-mutex rejection.
- **P2:** multisig restore (md1 + cosigners + own seed) → resolved descriptor byte-matches the bundle's; auto-detect slot; `--my-slot` override; wrong-passphrase → no slot match → exit 4; `--allow-mismatch` override; mk1-card vs raw-xpub cosigner inputs.
- **P3:** each `--format` payload; `--json` shape + seed redaction (negative-assert no seed); `--output` file; `--count`.
- **P4:** `make -C docs/manual audit` EXIT=0; `cli_gui_schema.rs` 29 green; GUI `schema_mirror`+`pin_coherence`+`secret_drift` green at `cargo +1.94.0`.
- **Regression each phase:** `cargo test -p mnemonic-toolkit --no-fail-fast` 0 fail; `clippy --all-targets -D warnings`.

---

## 10. SemVer + release checklist

MINOR → toolkit **v0.43.0** + GUI **v0.24.0**. Phase-6 (P4): `Cargo.toml` 0.42.0→0.43.0 + stage `Cargo.lock`; BOTH README `<!-- toolkit-version: -->` markers + both `Status:` prose lines; `CHANGELOG.md`; `scripts/install.sh` self-pin TAG; FOLLOWUP flips; `readme_version_current` PASS. GUI: version 0.23.0→0.24.0 + CHANGELOG + module-doc/pinned_version banner + pins.

---

## 11. Open decisions (resolved here; R0 may challenge)

1. **`--from` scope = seed-bearing nodes only** (`ms1`/`phrase`/`entropy`/`seedqr`). xprv/xpub/wif excluded (a passphrase doesn't apply; already-derived keys need no "restore"). 
2. **Single-sig default = all 4 types**; `--template` narrows. (User restoring may not recall the type.)
3. **Multisig cosigner presentation via restore-local `--cosigner @N=mk1|xpub`** (decode mk1 internally) — NOT a new `mk1` slot subkey (avoids churning the shared `slot_input.rs` legal-set machinery).
4. **Mismatch = hard-fail (exit 4) + `--allow-mismatch` override; no-reference = UNVERIFIED banner.** (User-locked.)
5. **Watch-only-out**: `account_xpub`/`master_fingerprint`/addresses/descriptor only; never `account_xpriv`/WIF.

## 12. Citation note

Toolkit citations verified @ `6566941`. Sibling APIs (`md_codec::chunk::reassemble`, `mk_codec::decode`) to be re-grepped against the crates.io-pinned source (`md-codec 0.35`, `mk-codec 0.4.0`) at plan-write time (`feedback_verify_cited_apis_against_docs_rs`). CLAUDE.md "git deps until v0.1" prose is stale (now crates.io) — do not cite git-dep paths.
