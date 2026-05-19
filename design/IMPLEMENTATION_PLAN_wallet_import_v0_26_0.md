# IMPLEMENTATION PLAN — `wallet-import` v0.26.0 (toolkit) + `mnemonic-gui` v0.11.0

> **For agentic workers:** REQUIRED SUB-SKILL — `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans`. Each task is opus-architect-R0-reviewed before implementation (per `[[feedback-opus-primary-review-agent]]`).

**Goal:** Add `mnemonic import-wallet` subcommand (BSMS Round-2 + Bitcoin Core `listdescriptors`; xpub-only; round-trip pairs with stderr diff). Add cross-cutting `@env:<VAR>` sentinel across 6 secret-flag surfaces. GUI lockstep ships static-form schema mirror only (dynamic widget deferred to v0.12.0).

**Architecture:** New `wallet_import/` module mirroring `wallet_export/` trait-dispatcher pattern. Match-dispatched `WalletFormatParser` trait with per-format zero-sized parser structs. Reuse `parse_descriptor::parse_descriptor()` + `md_codec::Descriptor` + `CosignerKeyInfo`. New cross-cutting `@env:VAR` resolution at clap-parse time.

**Tech Stack:** Rust 1.78+, miniscript v13.0, serde_json, regex, bitcoin-rs Network/DerivationPath/Xpub, `mnemonic-gui` egui+kittest, clap-derive.

**Predecessor:** toolkit master `7c1f874` (v0.25.1).
**Worktree:** `.claude/worktrees/wallet-import-export-multiformat-brainstorm` on branch `worktree-wallet-import-export-multiformat-brainstorm` (commit `fc6efa1` carries BRAINSTORM + SPEC).
**Cross-references:**
- BRAINSTORM: `design/BRAINSTORM_wallet_import_v0_26_0.md` (`fc6efa1`)
- SPEC: `design/SPEC_wallet_import_v0_26_0.md` (`fc6efa1`)

---

## §0 Context

The toolkit's export-side (`mnemonic export-wallet`) covers 8 third-party formats since v0.7..v0.8.1. The import direction is 0% built. The user's session-driving need: ingest a BSMS Round-2 blob (`BSMS 1.0\n<descriptor>#checksum`) describing a 2-of-2 decaying-multisig (`wsh(thresh(2, pk, s:pk, sln:older(32768)))`) and emit toolkit bundle(s) for engraving. This cycle ships that path end-to-end + Bitcoin Core `listdescriptors` (xpub-only) + cross-cutting env-var sentinel.

Out of scope (deferred FOLLOWUPs per BRAINSTORM §6): BSMS Round-1 ingest, BSMS HMAC signature verification, Bitcoin Core xprv handling, all other vendor formats (Sparrow/Specter/Electrum/Coldcard/Jade/Green), GUI dynamic per-cosigner widget, GUI argv-redaction.

## §1 File structure

### §1.1 Toolkit-side (new files)

```
crates/mnemonic-toolkit/src/
├── cmd/
│   └── import_wallet.rs              [NEW] CLI entry; clap glue; trait match-dispatch
├── wallet_import/
│   ├── mod.rs                        [NEW] pub(crate) WalletFormatParser + ParsedImport + BsmsAuditFields
│   ├── sniff.rs                      [NEW] format auto-detect; ambiguity → exit 1
│   ├── bsms.rs                       [NEW] BsmsParser (2-line + 6-line; declaration-order @N adapter)
│   ├── bitcoin_core.rs               [NEW] BitcoinCoreParser (xprv reject; multi-descriptor enum)
│   ├── pipeline.rs                   [NEW] concrete-keys → @N-placeholder adapter (regex-lex + substitute)
│   └── roundtrip.rs                  [NEW] canonicalize + unified-diff helper
└── tests/
    ├── cli_import_wallet_bsms.rs                  [NEW] 10-14 cells per SPEC §10.1
    ├── cli_import_wallet_bitcoin_core.rs          [NEW] 10-14 cells per SPEC §10.2
    ├── cli_import_wallet_sniff.rs                 [NEW] auto-detect coverage
    ├── cli_import_wallet_roundtrip.rs             [NEW] 24-30 cells per SPEC §7
    ├── cli_import_wallet_seed_overlay.rs          [NEW] --ms1 / --slot @N.phrase= overlay
    └── cli_env_var_sentinel.rs                    [NEW] cross-cutting @env:VAR coverage
```

### §1.2 Toolkit-side (modified files)

```
crates/mnemonic-toolkit/src/
├── main.rs                           Add ImportWallet variant to Command enum + clap subcommand
├── lib.rs                            Re-export wallet_import module pub(crate)
├── error.rs                          Add ImportWallet{Parse,SeedMismatch,AmbiguousFormat,FormatMismatch,XprvForbidden,WatchOnlyViolation} + EnvVarMissing variants; map to tiers per SPEC §2.3
├── secrets.rs                        Add @env:VAR resolution; consumed by all 6 secret-flag surfaces
├── cmd/
│   ├── verify_bundle.rs              Wire @env:VAR resolution through --ms1 / --passphrase flags
│   ├── convert.rs                    Wire @env:VAR resolution through --passphrase / --bip38-passphrase
│   ├── bundle.rs                     Wire @env:VAR resolution through --passphrase
│   ├── synthesize.rs                 Wire @env:VAR resolution through --slot @N.phrase= / @N.ms1=
│   ├── derive_child.rs               Wire @env:VAR resolution through --passphrase
│   ├── slip39.rs                     Wire @env:VAR resolution through --share / --passphrase
│   ├── seed_xor.rs                   Wire @env:VAR resolution through --share
│   └── gui_schema.rs                 Add import-wallet SubcommandSchema; mark global flag carry; v5 stays
├── slot_input.rs                     Extend @env:VAR support in slot-subkey grammar
└── repair.rs                         No-op (BCH auto-fire applies via existing helper at decode failure)
```

### §1.3 GUI-side (modified files)

```
mnemonic-gui/
├── Cargo.toml                                     Pin toolkit at v0.26.0 (post-release update)
├── pinned-upstream.toml                           Bump toolkit row to v0.26.0
├── src/
│   ├── schema/mnemonic.rs                         Add import-wallet SubcommandSchema entry
│   ├── runner.rs                                  Env-var seed-channel: set MNEMONIC_MS1_<i> / etc on subprocess spawn; clear after
│   └── secrets.rs                                 Recognize @env:VAR sentinel; treat sentinel-valued args as non-secret in run-confirm-modal
└── tests/
    ├── kittest_import_wallet_form.rs              [NEW] 6-8 cells per SPEC §9.4
    └── schema_mirror_secret_drift.rs              Auto-validates new import-wallet entry; no test code changes (version-tolerant gate)
```

### §1.4 Manual + docs (new + modified)

```
docs/manual/src/
└── 40-cli-reference/
    ├── 41-mnemonic.md                 Add ## import-wallet subsection (load-bearing per CLAUDE.md mirror invariant)
    └── 45-foreign-formats.md          [NEW] BSMS Round-2 + Bitcoin Core listdescriptors blob shapes (sibling to 41-mnemonic.md per existing numbering convention; verified against `41/42/43/44` chapter scheme)

docs/manual-gui/src/
└── 20-feature-walkthroughs/
    └── 24-import-wallet.md            [NEW] short static-form walkthrough (full wizard deferred to v0.12.0)

CHANGELOG.md (toolkit)                 v0.26.0 entry
CHANGELOG.md (mnemonic-gui)            v0.11.0 entry

design/
├── SPEC_mnemonic_toolkit_v0_5.md      Amend §5.11 + §6.11 + §7 per BRAINSTORM §7
└── FOLLOWUPS.md                       13 new entries per BRAINSTORM §6
```

## §2 Phase decomposition

Each phase: TDD per cell + per-phase opus architect R0/R1+ until 0 Critical / 0 Important. Cell budgets per SPEC §10.

### Phase 0 — Recon + verification gates (no implementation; no tests)

**Goal:** verify SPEC assumptions about codebase + existing wallet_export behavior before writing code.

> **GATE:** Phase 0 must run BEFORE Phase 1 dispatch. Failure of §0.6 or §0.7 blocks the cycle. Successful completion writes `/tmp/phase-0-recon.md` AND `/tmp/phase-0-bsms-parse-verdict.md` artifacts; Phase 1 R0 must verify both artifacts exist + report GREEN before any code lands.

> **CRITICAL PRE-PHASE-1 GATE — §7.0 SPEC + BRAINSTORM amendments must land in the FIRST commit of Phase 1**, BEFORE any toolkit code change. See [§7.0 below](#70-pre-execution-spec--brainstorm-amendments-from-holistic-architect-review) for the 4-item amendment checklist. Discipline: Phase 1's first commit is a documentation-only commit titled `design: pre-cycle SPEC + BRAINSTORM amendments for wallet-import v0.26.0`.

**Tasks:**

- [x] **§0.1 Verify wallet_export lexsort claim empirically.** (Phase 0 executed 2026-05-18.) Run with the existing wallet-export test-fixture xpubs (`COSIGNER_{A,B,C}_XPUB` in `tests/cli_export_wallet_jade.rs`):
  ```bash
  cargo run --release --bin mnemonic -- export-wallet \
      --format bitcoin-core --template wsh-sortedmulti \
      --threshold 2 \
      --slot @0.xpub=<xpub-A> --slot @0.fingerprint=<fp-A> --slot @0.path="m/48'/0'/0'/2'" \
      --slot @1.xpub=<xpub-B> --slot @1.fingerprint=<fp-B> --slot @1.path="m/48'/0'/0'/2'" \
      --slot @2.xpub=<xpub-C> --slot @2.fingerprint=<fp-C> --slot @2.path="m/48'/0'/0'/2'" \
      2>&1 | tee /tmp/lexsort-forward.txt
  ```
  Then reverse slot order (@0=C, @1=B, @2=A) and rerun. **Empirically confirmed (Phase 0 R0 R2 fold):** outputs DIFFER — toolkit preserves declaration order in the `sortedmulti(...)` argument list. Bitcoin's `sortedmulti` performs key-sort at SIGNATURE-MATERIALIZATION time, not at descriptor construction. Plan-doc's prior "byte-identical" expectation was incorrect. The import-side `@N` adapter must mirror this discipline (declaration-order preservation); round-trip byte-identical equality will fail for any BSMS blob whose cosigner order differs from toolkit's slot-index order, making semantic round-trip + unified-diff WARNING (SPEC §7.2) the load-bearing path.

- [x] **§0.2 Verify `md_codec::Descriptor` has no `cosigner_order` TLV.** (Phase 0 executed 2026-05-18 via direct source inspection of `descriptor-mnemonic/crates/md-codec/src/{encode.rs,tlv.rs}`.) Confirmed `Descriptor` fields: `{n, path_decl, use_site_path, tree, tlv}`. Confirmed `TlvSection` fields (per Phase 0 R0 R3 fold — plan-doc field-list was incomplete): `{use_site_path_overrides, fingerprints, pubkeys, origin_path_overrides, unknown}`. **NO `cosigner_order` field.** Phase 2 R0 checklist item: confirm `wallet_import/pipeline.rs` populates `origin_path_overrides` correctly when BSMS cosigners use non-default BIP-48 account paths.

- [x] **§0.3 Verify `gui_schema` `secret` field flow.** (Phase 0 executed 2026-05-18.) Plan-doc had `--classify-flags` — flag does NOT exist (only `--classify-descriptor` per `gui_schema.rs:1262`). Correct command (bare `gui-schema` emits the full JSON envelope):
  ```bash
  cargo run --release --bin mnemonic -- gui-schema 2>/dev/null | jq -c '.subcommands[] | {name, secret_flags: [.flags[] | select(.secret == true) | .name]}'
  ```
  **Empirically confirmed:** `--ms1` (verify-bundle, inspect, repair), `--passphrase` (bundle, convert, derive-child, slip39-{combine,split}, verify-bundle), `--bip38-passphrase` (convert), `--share` (seed-xor-combine, slip39-combine) all flow `secret: true`. This is the load-bearing pre-check for I9 per SPEC §9.5.

- [x] **§0.4 Verify mnemonic-gui run-confirm-modal renders argv verbatim** (Phase 0 executed 2026-05-18; pre-confirms architect's C2 from Section 4 review). Confirmed `mnemonic-gui/src/main.rs:686-688` renders verbatim via `for tok in &argv { ui.monospace(format!("  {}", tok)); }`. All `redact*` matches live in `persistence.rs` (on-disk JSON state redaction, not argv display) or `form/secret_widget.rs` (TextEdit input-masking). This guides Phase 6 design: env-var sentinel obviates argv-redaction need for v0.11.0.

- [x] **§0.5 Verify v0.25.1 empty-string sentinel semantics still active.** (Phase 0 executed 2026-05-18.) Confirmed `watch_only_empty_ms1_sentinel_marks_cosigner_skip_with_notice` passes (1153-cell baseline at v0.25.1).

- [x] **§0.6 Empirically verify the BSMS seed-case parses today.** (Phase 0 executed 2026-05-18; GO/NO-GO **GO**.) Plan-doc command `convert --from descriptor=<line2>` is REJECTED (`descriptor` is not a valid `NodeType`; `convert.rs:66-83`). Correct command:
  ```bash
  cat <<'EOF' > /tmp/bsms-seedcase.txt
  BSMS 1.0
  wsh(thresh(2,pkh([704c7836/48'/1'/3'/2']tpubDEgS9fUEpucKatmvKAv21v8nViHxR6rsV7ohMWK4YjsWd4EWT3w8YzMgMEvNrDfsUANbid74WRFpr3Gym8UHBSLnqg6b1Lzvibw87cLSctC/<0;1>/*),s:pk([97139860/48'/1'/2'/2']tpubDFiXyf7zmBhQrSHoAQB6SmMpF3rfSihAxQGMdQUtZfE8HWHkWLLNLTiYpMzvHnFiTmuUSYieHUYv4tFguzmiHeDrYV8TtWGCWt5qpqox4w3/<0;1>/*),sln:older(32768)))#zh0duts0
  EOF
  DESC_LINE=$(sed -n '2p' /tmp/bsms-seedcase.txt)
  cargo run --release --bin mnemonic -- export-wallet --descriptor "$DESC_LINE" --network testnet --format bitcoin-core
  ```
  **Empirically confirmed:** Exit 0; toolkit emits Bitcoin Core `importdescriptors` JSON, auto-splitting multipath `<0;1>/*` into receive `/0/*` + change `/1/*` pairs (re-checksummed `#ld0r6z6d` / `#7qqz5h4m`). rust-miniscript v13.0 accepts: `sln:` compound wrapper (Swap + OrI(False,_) + ZeroNotEqual), `tpub` testnet keys, BIP-389 multipath, BIP-380 checksum.

  **§0.6.b — I1 fold (Phase 0 R0 architect-review).** The `export-wallet --descriptor` path uses only `MsDescriptor::from_str` (`export_wallet.rs:257-263`); it does NOT exercise the Phase 2 SPEC §4.2 step 7 pipeline (`pipeline::concrete_keys_to_placeholders` → `parse_descriptor::parse_descriptor` → `walk_root` over the full AST). To close the architect's I1 finding, a regression test was added at `parse_descriptor.rs::tests::phase0_i1_bsms_decaying_multisig_walks_end_to_end` exercising `wsh(thresh(2, pkh(@0/<0;1>/*), s:pk(@1/<0;1>/*), sln:older(32768)))` end-to-end through the full pipeline. **Empirically passes.** All required walker arms (`PkH`, `PkK`, `Check`, `Swap`, `OrI`, `ZeroNotEqual`, `Older`, `False`, `Thresh`) compose correctly.

- [x] **§0.6.a Paired checksum-tamper check.** (Phase 0 executed 2026-05-18.) Plan-doc command was identical to §0.6's defective form. Correct:
  ```bash
  TAMPERED=$(echo "$DESC_LINE" | sed 's/#zh0duts0/#zh0duts1/')
  cargo run --release --bin mnemonic -- export-wallet --descriptor "$TAMPERED" --network testnet --format bitcoin-core
  ```
  **Confirmed:** Stderr `error: export-wallet --descriptor: invalid checksum zh0duts1; expected zh0duts0` (toolkit-side BIP-380 check fires before miniscript parse).

- [x] **§0.7 Verify `slip0132` exports a network-inference function.** (Phase 0 executed 2026-05-18.) Actual surface confirmed: `XpubPrefix::is_default`, `parse_xpub_prefix_arg`, `normalize_xpub_prefix`, `apply_xpub_prefix`, `neutral_for`, `render_slip0132_info_line`. **NO `detect_network` function exists.** **Decision LOCKED: option (b)** — extract `coin_type` (BIP-48 path component index 1) from the `[fp/path]` origin annotation: hardened `0'` → `bitcoin::Network::Bitcoin`, hardened `1'` → `bitcoin::Network::Testnet`. Rationale: option (a)'s xpub-prefix path collapses on `tpub`-canonicalized inputs (same false-mainnet trap); option (b) reads the authoritative BIP-48 coin-type signal. **Architect R0 I2 fold:** signet and regtest are not distinguishable from testnet via origin-path inspection in either BIP-129 BSMS or Bitcoin Core `listdescriptors` — both use coin-type `1`. The toolkit's canonical interpretation is `bitcoin::Network::Testnet`; users running signet/regtest workflows must supply `--network signet|regtest` post-import. FOLLOWUP: `wallet-import-signet-regtest-disambiguation` (v0.27+). Cosigner-to-cosigner coin-type heterogeneity → exit 2 `ImportWalletParse`. SPEC §4.2 step 8 amendment text captured in §7.0.a (Phase 1 first-commit).

- [x] **§0.8 Verify `similar` crate is NOT already in workspace.** (Phase 0 executed 2026-05-18.) 0 matches confirmed. Phase 4 §4.2 adds `similar = "2"` as toolkit-crate-level dep.

- [x] **§0.9 Recon report.** (Phase 0 executed 2026-05-18.) Recon report committed at `design/agent-reports/phase-0-recon.md`; opus architect R0 review at `design/agent-reports/phase-0-r0-review.md`. **Verdict: GREEN-after-fold** — 0 Critical / 2 Important folded inline (I1 + I2).

**Architect-review (Phase 0):** R0 — dispatch opus `feature-dev:code-architect` to review the `/tmp/phase-0-recon.md` against the SPEC's load-bearing assumptions. R1+ until 0 Critical.

### Phase 1 — Cross-cutting `@env:VAR` sentinel

**Goal:** sentinel resolution applies uniformly across all 6 secret-flag surfaces. 12-18 cells.

**Files:**
- Create: `crates/mnemonic-toolkit/src/secrets.rs` extension (resolver fn) — actually extension within existing file
- Create: `crates/mnemonic-toolkit/tests/cli_env_var_sentinel.rs` (12-18 cells)
- Modify: `secrets.rs`, `slot_input.rs`, and 7 callsite files per §1.2

**Sub-tasks:**

- [ ] **§1.1 Add `resolve_env_var_sentinel(value: &str, flag_name: &str) -> Result<String, ToolkitError>` helper in `secrets.rs`.**

  ```rust
  /// Resolve a `@env:VAR` sentinel to the env-var value, or pass through literal strings.
  /// Returns `Err(EnvVarMissing { flag, var })` if the sentinel is malformed or VAR is unset.
  pub(crate) fn resolve_env_var_sentinel(value: &str, flag_name: &str) -> Result<String, ToolkitError> {
      if let Some(varname) = value.strip_prefix("@env:") {
          if !is_valid_posix_env_var_name(varname) {
              return Err(ToolkitError::EnvVarMissing {
                  flag: flag_name.to_string(),
                  var: varname.to_string(),
                  reason: EnvVarMissingReason::InvalidName,
              });
          }
          std::env::var(varname).map_err(|_| ToolkitError::EnvVarMissing {
              flag: flag_name.to_string(),
              var: varname.to_string(),
              reason: EnvVarMissingReason::Unset,
          })
      } else {
          Ok(value.to_string())
      }
  }

  fn is_valid_posix_env_var_name(name: &str) -> bool {
      // POSIX: starts with [A-Z_], rest is [A-Z0-9_]
      let mut chars = name.chars();
      match chars.next() {
          Some(c) if c.is_ascii_uppercase() || c == '_' => {},
          _ => return false,
      }
      chars.all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_')
  }
  ```

- [ ] **§1.2 Add `EnvVarMissing` variant + `EnvVarMissingReason` enum to `error.rs`.** Tier-1 routing. Variant carries flag name + var name + reason.

- [ ] **§1.3 Cell: `env_var_happy_path_ms1`.** Set `MNEMONIC_MS1_0` to a valid ms1 string; invoke `mnemonic verify-bundle --ms1 @env:MNEMONIC_MS1_0 ...`; assert exit 0 + correct cross-check result.

- [ ] **§1.4 Cell: `env_var_unset_fails_exit_1`.** Invoke with `--ms1 @env:UNSET_VAR`; assert exit 1 + stderr matches "env-var UNSET_VAR ... is not set".

- [ ] **§1.5 Cell: `env_var_empty_string_preserves_v0_25_1_sentinel`.** Set `MNEMONIC_MS1_0=""`; invoke; assert v0.25.1 watch-only empty-string semantics fire (cosigner skipped with notice; exit 0).

- [ ] **§1.6 Cell: `env_var_invalid_name_fails`.** Test cases: `@env:foo bar`, `@env:1FOO`, `@env:`, `@env:lowercase`. Each → exit 1.

- [ ] **§1.7 Cell: `env_var_works_on_passphrase`.** Set `WALLET_PP`; invoke `mnemonic bundle --passphrase @env:WALLET_PP ...`; assert correct behavior.

- [ ] **§1.8 Cell: `env_var_works_on_bip38_passphrase`.** Same as §1.7 for `--bip38-passphrase` via `convert`.

- [ ] **§1.9 Cell: `env_var_works_on_share`.** Same for `--share` via `slip39 combine`.

- [ ] **§1.10 Cell: `env_var_works_on_slot_subkey_phrase`.** `--slot @0.phrase=@env:PHRASE_0`.

- [ ] **§1.11 Cell: `env_var_works_on_slot_subkey_ms1`.** `--slot @0.ms1=@env:MS1_0`.

- [ ] **§1.12 Cell: `env_var_repeated_same_var_resolves_consistently`.** `--ms1 @env:V --ms1 @env:V`; assert both resolve to same value (no error; both reads succeed).

- [ ] **§1.13 Cell: `env_var_mixed_with_literal_and_stdin`.** `--ms1 ms1xxx... --ms1 @env:VAR --ms1 -` (stdin); assert all three resolve correctly.

- [ ] **§1.14 Cell: `env_var_two_stdin_sentinels_fails`.** `--ms1 - --ms1 - ` (with stdin); assert exit 1 (multiple stdin readers per existing verify_bundle.rs:876 precedent).

- [ ] **§1.15 Cell: `env_var_literal_at_prefix_passes_through`.** `--ms1 prefix@env:VAR` (no whole-value-sentinel match); assert treated as literal, ms1 decode fails downstream.

- [ ] **§1.16 Cell: `env_var_on_non_secret_flag_passes_through`.** `--network @env:NET` → toolkit treats as literal "network" string; fails downstream parse (no env-var resolution attempted on non-secret-bearing flags). Decision: sentinel is opt-in per call-site; non-secret flags do NOT resolve sentinels (avoids surprise).

- [ ] **§1.17 Wire @env:VAR resolution through all 7 callsite files** per §1.2. Each callsite consumes `resolve_env_var_sentinel(raw, flag_name)?` at clap-parse-side or first-use-side. Diff per callsite kept small (≤3 lines).

- [ ] **§1.18 Cell: `env_var_lifecycle_no_leak_to_argv`.** Spawn a child mnemonic subprocess via `std::process::Command` with `--ms1 @env:MS1_0` and `env(MS1_0, secret_value)`. Read `/proc/<child-pid>/cmdline` from a tracer; assert secret value NOT present (only the sentinel string). End-to-end argv-leak audit.

**Phase 1 verification:**
```bash
cargo test -p mnemonic-toolkit --test cli_env_var_sentinel
```
Expected: all 12-18 cells pass.

**Architect-review (Phase 1):** R0 — dispatch opus on plan-doc + Phase 1 sub-tasks + test cell enumeration. Confirm cross-cutting coverage spans all 6 surfaces; confirm tier-1 mapping; confirm no escape mechanism for literal `@env:` strings (deferred to FOLLOWUP).

### Phase 2 — BSMS Round-2 parser

**Goal:** parse 2-line + 6-line BSMS Round-2 blobs to `ParsedImport`. 10-14 cells.

**Files:**
- Create: `crates/mnemonic-toolkit/src/wallet_import/{mod.rs, bsms.rs, pipeline.rs}` (per §1.1)
- Create: `crates/mnemonic-toolkit/tests/cli_import_wallet_bsms.rs` (10-14 cells)

**Sub-tasks:**

- [ ] **§2.1 Write `wallet_import/mod.rs` trait + ParsedImport + BsmsAuditFields skeleton.** Per SPEC §8.1. Match-dispatched.

- [ ] **§2.2 Write `wallet_import/pipeline.rs::concrete_keys_to_placeholders` adapter.**

  ```rust
  /// Convert a descriptor with concrete [fp/path]xpub keys into placeholder form
  /// (@N) + collected (ParsedKey, ParsedFingerprint) pairs, preserving declaration order.
  pub(crate) fn concrete_keys_to_placeholders(
      descriptor: &str,
  ) -> Result<(String, Vec<ParsedKey>, Vec<ParsedFingerprint>), ToolkitError> {
      // Regex: \[([0-9a-fA-F]{8})((?:/\d+'?)+)\]([xtuvyzYZ]pub[A-HJ-NP-Za-km-z1-9]+)
      // Greedy match; preserve order via Vec accumulator.
      // Replace match with @N placeholder.
      // Reuses ParsedKey / ParsedFingerprint structs from parse_descriptor.rs
      ...
  }
  ```

  Per SPEC §4.2 step 5. Inverse of `wallet_export::pipeline::descriptor_to_bip388_wallet_policy:160-205`. SLIP-132 prefix tolerance: regex accepts `xpub|tpub|ypub|Ypub|zpub|Zpub|upub|Upub|vpub|Vpub`.

- [ ] **§2.3 Write `wallet_import/bsms.rs::BsmsParser` skeleton.** `impl WalletFormatParser` with `sniff` + `parse` functions.

- [ ] **§2.4 Cell: `bsms_2_line_happy_path`.** Input = user's flagship seed-case blob. Assert exit 0 + correct `ParsedImport` (descriptor, 2 cosigners with byte-exact xpubs + paths + fingerprints, threshold = Some(2), bsms_audit = None, watch-only invariant holds).

- [ ] **§2.5 Cell: `bsms_6_line_happy_path`.** Synthesize a 6-line BSMS Round-2 blob from a known signer (test fixture). Assert `bsms_audit` is populated; no WARNING about reduced form.

- [ ] **§2.6 Cell: `bsms_first_address_mismatch_warning`.** 6-line blob with intentionally-mismatched first_address. Assert stderr WARNING fires; exit 0 (not hard-error this cycle).

- [ ] **§2.7 Cell: `bsms_2_line_warning_emitted`.** Assert stderr WARNING text "BSMS Round-2 excerpt..." matches §2.4 template.

- [ ] **§2.8 Cell: `bsms_decaying_multisig_N_144`.** `wsh(thresh(2,pk,s:pk,sln:older(144)))` (1-day fallback). Decaying-multisig fixture class per SPEC §10.1.

- [ ] **§2.9 Cell: `bsms_decaying_multisig_N_4032`.** Same but N = 4032 (~28-day).

- [ ] **§2.10 Cell: `bsms_decaying_multisig_N_32768`.** User's exact blob (matches kickoff seed-case).

- [ ] **§2.11 Cell: `bsms_sortedmulti_2_of_3`.** Standard sortedmulti.

- [ ] **§2.12 Cell: `bsms_multi_non_sorted_2_of_3`.** Non-sortedmulti; assert declaration order preserved (NOT lexsorted).

- [ ] **§2.13 Cell: `bsms_slip132_variants_ypub`.** Tests SLIP-132 ypub prefix detection via `slip0132.rs`.

- [ ] **§2.14 Cell: `bsms_bad_checksum_exit_2`.** Tamper with `#zh0duts0` checksum; assert exit 2 `ImportWalletParse` + stderr "parse error".

- [ ] **§2.15 Cell: `bsms_unsupported_version_exit_3`.** Blob with `BSMS 2.0` line 1. Assert exit 3 `FutureFormat`.

- [ ] **§2.16 Cell: `bsms_not_bsms_blob_exit_1`.** Blob with `Lol no` line 1. Assert exit 1 (parses via sniff fallthrough → ambiguous format).

**Phase 2 verification:**
```bash
cargo test -p mnemonic-toolkit --test cli_import_wallet_bsms
```

**Architect-review (Phase 2):** R0 — dispatch opus on Phase 2 sub-tasks + cell enumeration. Confirm `parse_descriptor::parse_descriptor` is called correctly (placeholder-form input + keys + fingerprints args); confirm watch-only invariant validation post-parse; confirm BIP-380 checksum auto-validation.

### Phase 3 — Bitcoin Core `listdescriptors` parser

**Goal:** parse Bitcoin Core `listdescriptors` JSON to `Vec<ParsedImport>`. 10-14 cells.

**Files:**
- Create: `crates/mnemonic-toolkit/src/wallet_import/bitcoin_core.rs`
- Create: `crates/mnemonic-toolkit/tests/cli_import_wallet_bitcoin_core.rs` (10-14 cells)

**Sub-tasks:**

- [ ] **§3.1 Write `BitcoinCoreParser` skeleton.** `impl WalletFormatParser`. JSON-parse via `serde_json::Value`; validate top-level shape.

- [ ] **§3.2 Cell: `core_single_descriptor_wpkh_happy_path`.** Single-sig BIP-84 P2WPKH. Assert ParsedImport correct.

- [ ] **§3.3 Cell: `core_multi_descriptor_emit_all`.** 4 entries (receive + change + 2 script-types). Default `--select-descriptor all`; assert 4 bundles emitted.

- [ ] **§3.4 Cell: `core_select_descriptor_by_index`.** `--select-descriptor 2`; assert only descriptors[2] emitted.

- [ ] **§3.5 Cell: `core_select_descriptor_active_receive`.** Filter `active: true, internal: false`; assert correct selection.

- [ ] **§3.6 Cell: `core_select_descriptor_active_change`.** Filter `active: true, internal: true`.

- [ ] **§3.7 Cell: `core_multisig_wsh_sortedmulti_2_of_3`.** Multisig case.

- [ ] **§3.8 Cell: `core_multipath_split_to_receive_change`.** Single multipath `<0;1>/*` entry; assert handled per existing export-side multipath logic.

- [ ] **§3.9 Cell: `core_xprv_rejected_exit_2`.** Blob with `xprv...` desc; assert exit 2 `ImportWalletXprvForbidden` + stderr template matches "re-run `bitcoin-cli listdescriptors` without `true`".

- [ ] **§3.10 Cell: `core_dropped_fields_notice`.** Blob with `timestamp: "now"` + `next: 5`; assert stderr NOTICE per SPEC §2.4 (informational; exit 0).

- [ ] **§3.11 Cell: `core_invalid_json_exit_2`.** Mangled JSON. Assert exit 2 + stderr "parse error".

- [ ] **§3.12 Cell: `core_missing_descriptors_key_exit_2`.** Top-level JSON missing `descriptors`; assert exit 2.

- [ ] **§3.13 Cell: `core_empty_descriptors_array_exit_2`.** `{"descriptors": []}`; assert exit 2 (no bundles to emit).

- [ ] **§3.14 Cell: `core_testnet_tpub_network_detected`.** All-tpub descriptors → Network::Testnet.

**Phase 3 verification:**
```bash
cargo test -p mnemonic-toolkit --test cli_import_wallet_bitcoin_core
```

**Architect-review (Phase 3):** R0 — dispatch opus on Phase 3 sub-tasks + cell enumeration. Confirm multi-descriptor selector logic + `--select-descriptor all` separator behavior + xprv refusal exit code routing.

### Phase 4 — Round-trip discipline

**Goal:** bundle struct-equality + semantic blob canonicalize with stderr diff. 24-30 cells (12-15 fixtures × 2 directions).

**Files:**
- Create: `crates/mnemonic-toolkit/src/wallet_import/roundtrip.rs` (canonicalize + diff helpers)
- Create: `crates/mnemonic-toolkit/tests/cli_import_wallet_roundtrip.rs` (24-30 cells)
- Create: `crates/mnemonic-toolkit/tests/fixtures/wallet_import/` directory tree (BSMS + Core fixtures + golden bundles)

**Sub-tasks:**

- [ ] **§4.1 Write `roundtrip.rs::canonicalize_bsms` + `canonicalize_bitcoin_core` per SPEC §7.3.**

  ```rust
  pub(crate) fn canonicalize_bsms(blob: &[u8]) -> Result<String, ToolkitError> {
      // 1. Normalize CRLF → LF.
      // 2. Strip trailing whitespace per line.
      // 3. Extract descriptor; parse via MsDescriptor::from_str; re-render; re-checksum.
      // 4. Drop audit lines.
      // 5. Re-emit canonical form.
  }

  pub(crate) fn canonicalize_bitcoin_core(blob: &[u8]) -> Result<String, ToolkitError> {
      // 1. Parse JSON.
      // 2. For each descriptor: re-checksum desc; preserve range/active/internal.
      // 3. Drop timestamp/next/next_index from compare.
      // 4. Re-serialize with sorted keys + 2-space indent.
  }
  ```

- [ ] **§4.2 Write `roundtrip.rs::unified_diff` helper.** Uses `similar = "2"` (toolkit-crate-level dep added to `crates/mnemonic-toolkit/Cargo.toml` in this task; MIT/Apache-2.0; widely-vetted). Output unified-diff RFC format:
  ```rust
  pub(crate) fn unified_diff(old: &str, new: &str) -> String {
      similar::TextDiff::from_lines(old, new)
          .unified_diff()
          .header("input", "output")
          .to_string()
  }
  ```
  `UnifiedDiff` implements `Display` (per similar 2.x docs); `.to_string()` materializes the RFC unified-diff text. Phase 0 §0.8 confirmed `similar` is NOT in workspace today.

- [ ] **§4.3 Fixture corpus: BSMS** (12-15 inputs):
  - 2-line decaying-multisig (N=144) `bsms-2line-decay-144.txt`
  - 2-line decaying-multisig (N=4032) `bsms-2line-decay-4032.txt`
  - 2-line decaying-multisig (N=32768) `bsms-2line-decay-32768.txt`
  - 6-line decaying-multisig (N=32768) `bsms-6line-decay-32768.txt`
  - 2-line sortedmulti 2-of-2 `bsms-2line-sortedmulti-2of2.txt`
  - 2-line sortedmulti 2-of-3 `bsms-2line-sortedmulti-2of3.txt`
  - 2-line sortedmulti 3-of-5 `bsms-2line-sortedmulti-3of5.txt`
  - 2-line multi 2-of-3 `bsms-2line-multi-2of3.txt`
  - 6-line sortedmulti 2-of-3 `bsms-6line-sortedmulti-2of3.txt`
  - testnet + tpub `bsms-testnet-2of2.txt`
  - mainnet + ypub `bsms-mainnet-ypub-2of2.txt`
  - mainnet + zpub `bsms-mainnet-zpub-2of2.txt`
  - 1-of-1 single-sig `bsms-1of1-singlesig.txt`
  - `tr(NUMS, …)` taproot (if rust-miniscript supports) `bsms-taproot-1of2-multipath.txt`
  - `sh(wsh(...))` legacy `bsms-shwsh-2of3.txt`

- [ ] **§4.4 Fixture corpus: Bitcoin Core** (12-15 inputs) per SPEC §10.2. Includes single-sig P2PKH (BIP-44), P2WPKH (BIP-84), P2SH-P2WPKH (BIP-49), P2TR (BIP-86), multisig wsh-sortedmulti 2-of-3 + 3-of-5, multipath `<0;1>`, receive+change pairs, active mixes, mainnet + testnet variants.

- [ ] **§4.5 Cells: bundle round-trip — BSMS (12-15 cells).** Pattern per fixture:
  ```rust
  let bundle_synth = mnemonic_synthesize(/* toolkit-args derived from fixture */);
  let blob = mnemonic_export_wallet(bundle_synth, "bsms");
  let bundles_imp = mnemonic_import_wallet(blob, "bsms");
  assert_eq!(bundles_imp.len(), 1);
  assert_eq!(bundles_imp[0], bundle_synth);
  ```

- [ ] **§4.6 Cells: bundle round-trip — Bitcoin Core (12-15 cells).** Same pattern.

- [ ] **§4.7 Cells: semantic blob round-trip — BSMS (12-15 cells).** Pattern:
  ```rust
  let bundle = mnemonic_import_wallet(fixture_blob, "bsms");
  let blob_re = mnemonic_export_wallet(bundle, "bsms");
  assert_eq!(canonicalize_bsms(fixture_blob), canonicalize_bsms(&blob_re));
  // If fixture_blob != blob_re bytewise: assert stderr WARNING + diff capture
  ```

- [ ] **§4.8 Cells: semantic blob round-trip — Bitcoin Core (12-15 cells).** Same pattern.

- [ ] **§4.9 Cell: `roundtrip_json_envelope_includes_byte_exact_field`.** Assert `--json` envelope contains `roundtrip: {byte_exact, semantic_match, diff}` field per SPEC §7.4.

**Phase 4 verification:**
```bash
cargo test -p mnemonic-toolkit --test cli_import_wallet_roundtrip
```
Expected: all 24-30 cells pass. Watch for budget overrun; if Phase 4 cells exceed 30, split into 4a (BSMS) + 4b (Core).

**Architect-review (Phase 4):** R0 — dispatch opus on canonicalize() correctness + fixture corpus completeness. Confirm BIP-380 checksum recompute logic; confirm Core JSON re-serialization with sorted keys.

### Phase 5 — Auto-detect + seed overlay

**Goal:** sniff format from blob; apply `--ms1` / `--slot @N.phrase=` overlay with auto-validation. 8-10 cells.

**Files:**
- Create: `crates/mnemonic-toolkit/src/wallet_import/sniff.rs`
- Create: `crates/mnemonic-toolkit/tests/cli_import_wallet_sniff.rs`
- Create: `crates/mnemonic-toolkit/tests/cli_import_wallet_seed_overlay.rs`

**Sub-tasks:**

- [ ] **§5.1 Write `sniff.rs::sniff_format` dispatcher** per SPEC §6. BSMS heuristic: prefix-match `BSMS 1.0\n`. Core heuristic: JSON-parse + top-level `descriptors: [{desc:String}]` + NO Specter/Sparrow vendor-marker keys (per architect I4 fold).

- [ ] **§5.2 Cell: `sniff_bsms_2line_detected`.** User's flagship blob. Assert format = bsms.

- [ ] **§5.3 Cell: `sniff_core_descriptors_detected`.** Synthesized `listdescriptors` blob. Assert format = bitcoin-core.

- [ ] **§5.4 Cell: `sniff_ambiguous_descriptors_with_specter_markers`.** Specter-like JSON with `descriptors` array + `chain` key. Assert sniff returns AMBIGUOUS (no parser claims it) → exit 1 ambiguous-format.

- [ ] **§5.5 Cell: `sniff_format_mismatch_explicit_override`.** Supply `--format bsms` for a Core blob → exit 1 `ImportWalletFormatMismatch`.

- [ ] **§5.6 Cell: `sniff_no_match_no_format_exit_1`.** Random text blob; assert exit 1 + stderr "could not detect format".

- [ ] **§5.7 Write seed-overlay logic** per SPEC §8.3.

- [ ] **§5.8 Cell: `seed_overlay_ms1_match_success`.** Supply correct `--ms1 <s>` matching blob xpub at declared path. Assert exit 0 + bundle's cosigner has `entropy: Some(...)`.

- [ ] **§5.9 Cell: `seed_overlay_ms1_mismatch_exit_4`.** Supply wrong `--ms1 <s>`. Assert exit 4 `ImportWalletSeedMismatch` + stderr template matches.

- [ ] **§5.10 Cell: `seed_overlay_partial_watch_only`.** 3-cosigner blob; supply `--ms1` for cosigner 0 + 2 only (cosigner 1 skipped). Assert cosigner 1 has `entropy: None` (watch-only); cosigner 0 + 2 have entropy populated.

- [ ] **§5.11 Cell: `seed_overlay_via_slot_subkey_phrase`.** Use `--slot @0.phrase=<bip39>` instead of `--ms1`. Assert phrase → entropy conversion + match.

- [ ] **§5.12 Cell: `sniff_path_roundtrip`.** Roundtrip exercise where sniff is invoked (no `--format` flag). End-to-end sniff → parse → export → re-sniff symmetry.

**Phase 5 verification:**
```bash
cargo test -p mnemonic-toolkit --test cli_import_wallet_sniff --test cli_import_wallet_seed_overlay
```

**Architect-review (Phase 5):** R0 — dispatch opus on sniff disambiguation logic + seed-overlay cross-check correctness. Confirm Specter/Sparrow exclusion. Confirm `derive_xpub_at_path` reuses existing toolkit derivation helpers (not a new derivation impl).

### Phase 6 — GUI lockstep + manual mirror

**Goal:** ship `mnemonic-gui` v0.11.0 with static-form import-wallet schema entry + env-var seed channel + manual updates. 6-8 cells (GUI-side) + manual mirror lint passing.

**Files (GUI side):**
- Modify: `mnemonic-gui/src/schema/mnemonic.rs`
- Modify: `mnemonic-gui/src/runner.rs`
- Modify: `mnemonic-gui/src/secrets.rs`
- Create: `mnemonic-gui/tests/kittest_import_wallet_form.rs` (6-8 cells)

**Files (manual side, toolkit repo):**
- Modify: `docs/manual/src/40-cli-reference/41-mnemonic.md` (add `## import-wallet` subsection)
- Create: `docs/manual/src/45-foreign-formats.md`
- Create: `docs/manual-gui/src/20-feature-walkthroughs/24-import-wallet.md`

**Sub-tasks:**

- [ ] **§6.1 Add `SubcommandSchema` for `import-wallet`** in `mnemonic-gui/src/schema/mnemonic.rs`. Per SPEC §9.1. Schema stays v5.

- [ ] **§6.2 Add per-secret env-var setter in `runner.rs::run`.** Pre-spawn: collect form's secret-field values into `MNEMONIC_<KIND>_<i>` env-vars; replace argv values with `@env:MNEMONIC_<KIND>_<i>` sentinels. Post-spawn: parent process must NOT retain env-vars in its own `std::env`.

- [ ] **§6.3 Cell (GUI): `form_renders_import_wallet_in_combobox`.** Open `mnemonic` top-level tab; assert combobox includes "import-wallet".

- [ ] **§6.4 Cell (GUI): `file_picker_blob_argv`.** Click `--blob` file picker; pick a file; assert argv contains `--blob /tmp/...`.

- [ ] **§6.4.a Cell (GUI): `file_picker_extension_filter`.** Assert the file picker filters for `.bsms`, `.txt`, `.json`, `*` (any). Files with other extensions are still selectable via "all files" filter (egui FilePicker convention).

- [ ] **§6.4.b Cell (GUI): `blob_paste_textarea_routes_to_stdin`.** Paste blob text into a multi-line textarea field (alternative to file-picker); on Run, GUI writes the textarea content to a tempfile and passes `--blob <tempfile-path>`, OR passes `--blob -` and pipes textarea content to subprocess stdin. Either route is acceptable; assert subprocess receives the blob bytes byte-for-byte.

- [ ] **§6.5 Cell (GUI): `repeating_ms1_text_inputs_sentinel_argv`.** Enter 2 seeds in 2 repeating `--ms1` inputs; assert argv contains `--ms1 @env:MNEMONIC_MS1_0 --ms1 @env:MNEMONIC_MS1_1` (NOT the literal seeds).

- [ ] **§6.6 Cell (GUI): `run_confirm_modal_shows_sentinels`.** Click Run; assert modal shows `@env:MNEMONIC_MS1_0` (not the seed).

- [ ] **§6.7 Cell (GUI): `select_descriptor_dropdown_argv`.** Pick `active-receive` from dropdown; assert argv contains `--select-descriptor active-receive`.

- [ ] **§6.8 Cell (GUI): `env_var_unset_subprocess_exits_1`.** Force empty seed input; subprocess receives `@env:VAR` sentinel with VAR unset; assert subprocess exits 1 + GUI output panel shows error.

- [ ] **§6.9 Cell (GUI): `env_var_no_parent_leak`.** Post-subprocess: assert `std::env::var("MNEMONIC_MS1_0").is_err()` in the parent GUI process state.

- [ ] **§6.10 Manual: `## import-wallet` subsection in `41-mnemonic.md`** mirroring clap help text byte-for-byte. Load-bearing per CLAUDE.md mirror invariant + `docs/manual/tests/lint.sh` gate.

- [ ] **§6.11 Manual: `45-foreign-formats.md`** chapter on BSMS Round-2 + Bitcoin Core blob shapes. Reference BIP-129, BIP-380, BIP-389 normatively.

- [ ] **§6.12 Manual-GUI: `24-import-wallet.md`** short walkthrough of the static-form import wizard.

- [ ] **§6.13 Toolkit manual lint:** `make -C docs/manual lint MNEMONIC_BIN=...` passes (bidirectional flag-coverage gate).

**Phase 6 verification:**
```bash
cd ../../../mnemonic-gui && cargo test --test kittest_import_wallet_form
cd /scratch/code/shibboleth/mnemonic-toolkit && make -C docs/manual lint MNEMONIC_BIN=target/release/mnemonic
```

**Architect-review (Phase 6):** R0 — dispatch opus on GUI form integration + manual mirror invariant compliance. Confirm `secret: true` flag flow through `secret_flag_keys()` consumer; confirm env-var-lifecycle gate cell semantics.

### Phase 7 — Cycle close

**Goal:** SPEC amendments + FOLLOWUPs + CHANGELOG + tag + GitHub Release.

#### §7.0 Pre-execution SPEC + BRAINSTORM amendments (FROM HOLISTIC ARCHITECT REVIEW)

These are foldable edits identified by the final architect review that **cannot be applied in plan-mode** (only the plan-doc file is editable here). Apply these as the **first commit of Phase 1 execution**, before any code change:

- [ ] **§7.0.a SPEC §4.2 step 8 — replace `slip0132.rs::detect_network` reference.** Per Phase 0 §0.7 verification: `slip0132.rs` has no `detect_network` function. Plan-doc Phase 0 §0.7 will lock the decision (existing `XpubPrefix` infrastructure OR `[fp/path]` coin-type sniff). Update SPEC §4.2 step 8 with the locked approach.

- [ ] **§7.0.b SPEC §7 anchor — rename to delta-style.** Per architect-review C2: existing v0.5 SPEC TOC goes §6.10 → §8 by intentional delta-only ordering. Replace "§7 (NEW)" naming with **§6.11.a** (mirrors `§4.12.a-g` precedent for non-canonical descriptors). Update BRAINSTORM §7 third bullet + SPEC §3.4 `§7 (NEW)` reference accordingly.

- [ ] **§7.0.c SPEC §8.1 `cosigners: Vec<CosignerKeyInfo>` → `Vec<ResolvedSlot>`.** Per architect-review I2: `CosignerKeyInfo` is a deprecated alias per `synthesize.rs:182-188`. Use canonical `ResolvedSlot` name. Add a one-line note that `CosignerKeyInfo` is the legacy alias retained for backward-compat.

- [ ] **§7.0.d SPEC §3.2 resolution scope explicit.** Per architect-review I7: SPEC §3.2 says "clap-parse-time substitution"; plan-doc §1.16 says "non-secret flags do NOT resolve sentinels." Reconcile: SPEC §3.2 amended to "Resolution applies ONLY at the 6 secret-flag surfaces enumerated in §3.1; non-secret flags treat `@env:VAR` as literal text." Lock the rule.

- [ ] **§7.0.e SPEC §3.1 row 1 `bip85` clarification.** Per architect-review N1 (convergent review): `bip85` is invoked via the `derive-child` subcommand (not a standalone subcommand). SPEC §3.1 row 1 "bundle, verify-bundle, convert, derive-child, slip39-{split,combine}, bip85" should be reworded to "...derive-child (covers BIP-85 path), slip39-{split,combine}". No new callsite; just clarifies the existing `derive_child.rs` covers BIP-85 invocations.

These 5 SPEC + BRAINSTORM edits are flagged as `pre-execution-amendments` to be made post-exit-plan-mode but BEFORE Phase 1 code lands. They preserve the artifact-trail integrity (every executed phase points to a SPEC version that the phase implements accurately).

**Execution discipline:** Phase 1's first commit is documentation-only: `design: pre-cycle SPEC + BRAINSTORM amendments for wallet-import v0.26.0` touching only `design/SPEC_wallet_import_v0_26_0.md` + `design/BRAINSTORM_wallet_import_v0_26_0.md`. Phase 1 §1.1 code work begins ONLY after this commit lands.

#### §7.1+ Standard cycle close

**Tasks:**

- [ ] **§7.1 Amend `design/SPEC_mnemonic_toolkit_v0_5.md`:**
  - Add `## §5.11 CLI value-source sentinels (NEW)` per SPEC `§3.4`.
  - Add `## §6.11 import-wallet CLI grammar (NEW)` mirroring §6.7 structure.
  - Add `## §6.11.a wallet_import round-trip discipline (NEW)` as sub-section of §6.11 (NOT a new top-level §7; preserves v0.5 SPEC's delta-only ordering convention per §7.0.b; mirrors `§4.12.a-g` precedent from v0.19.0).
  - Bump v0.5 SPEC version line if needed (delta-only ordering preserved).

- [ ] **§7.2 File 13 new FOLLOWUPs in `design/FOLLOWUPS.md`** per BRAINSTORM §6. Verify entries match BRAINSTORM list; cite SPEC sections.

- [ ] **§7.3 Write `CHANGELOG.md` entry for toolkit v0.26.0.** Sections: ### Added (new subcommand, new env-var sentinel), ### Changed (none), ### Deprecated (none), ### Removed (none), ### Fixed (none), ### Security (env-var sentinel obviates argv-leak for GUI subprocess; documented).

- [ ] **§7.4 Write `mnemonic-gui` `CHANGELOG.md` entry for v0.11.0.** Mirror style. Reference import-wallet form + env-var seed channel.

- [ ] **§7.5 End-of-cycle holistic architect review.** Dispatch opus on merged worktree branch reviewing all 7 phases together. Yellow → fold inline; Green → tag.

- [ ] **§7.6 Tag releases:**
  ```bash
  git tag -a mnemonic-toolkit-v0.26.0 -m "release(toolkit): mnemonic-toolkit v0.26.0 — wallet-import (BSMS + Bitcoin Core) + cross-cutting @env:VAR sentinel"
  ```
  Tagging mnemonic-gui v0.11.0 similarly in its own repo after Cargo.toml bump.

- [ ] **§7.7 GitHub Releases:**
  - `gh release create mnemonic-toolkit-v0.26.0 --title "..." --notes-file ...`
  - Same for `mnemonic-gui-v0.11.0`.

- [ ] **§7.8 Update memory** post-release per project convention. Add `project_v0_26_0_wallet_import_shipped.md` capturing tag + cell-count + reviewer-loop history + folded findings.

## §3 Verification

### §3.1 End-to-end smoke (post-Phase 6, pre-tag)

```bash
# BSMS user's flagship seed-case end-to-end
cargo run --release -- import-wallet --blob /tmp/bsms-seedcase.txt | head -30
# Expected: engraving card output (2 cosigners + 1 md1)

# Bitcoin Core multi-descriptor end-to-end (fixture-based, no running bitcoind needed)
cat crates/mnemonic-toolkit/tests/fixtures/wallet_import/core-bip84-multipath.json \
  | cargo run --release -- import-wallet --blob - --json | jq '.[0].bundle'
# Expected: JSON bundle

# Round-trip via shell pipeline
cargo run --release -- import-wallet --blob /tmp/bsms-seedcase.txt --json \
  | cargo run --release -- verify-bundle --bundle-json -
# Expected: verify-bundle output with all watch-only checks green

# Env-var sentinel end-to-end
MNEMONIC_MS1_0="ms1xxxxxxxxx..." cargo run --release -- verify-bundle --ms1 @env:MNEMONIC_MS1_0 --bundle-json bundle.json
# Expected: cross-check passes
```

### §3.2 Cell count budget

Target: 70-94 cells total. Watchpoints:
- Phase 1 (env-var sentinel): 12-18 cells.
- Phase 4 (round-trip): 24-30 cells. If exceeds 30, split into 4a (BSMS) + 4b (Core).

Cycle baseline: 1153 cells at v0.25.1. v0.26.0 target: ~1230-1250 cells.

### §3.3 CI gates

- `cargo test -p mnemonic-toolkit` — green.
- `cargo clippy --workspace -- -D warnings` — green.
- `cargo fmt --check` — green.
- `make -C docs/manual lint MNEMONIC_BIN=target/release/mnemonic` — green (mirror invariant).
- mnemonic-gui: `cargo test --workspace` — green; schema-mirror drift gate — green.

## §4 Open questions (resolved at per-phase architect-review or implementation time)

- **Q1 (Phase 0 finding):** If empirical lexsort check shows wallet_export DOES lexsort, scope expands to fix export side + write SPEC patch. (Architect predicts NO; verify empirically.)
- **Q2 (Phase 2 finding):** If `rust-miniscript v13.0` doesn't accept `sln:older(N)` directly in `wsh(thresh(...))` at the seed-case position, this is a Phase 0 blocker; either bump miniscript dep, vendor a wrapper, or document the limitation.
- **Q3 (Phase 4 finding):** Does the existing toolkit have a `similar`-crate dep for unified-diff? If not, vendor a ~30-LOC implementation.
- **Q4 (Phase 6 finding):** Does the mnemonic-gui's existing per-cosigner repeating-flag widget cleanly take per-row env-var-keyed input? Or does the GUI need a new form-field type for "secret-bearing repeating flag"?

## §5 Out of scope (deferred per BRAINSTORM §6)

13 FOLLOWUPs filed at cycle close; listed in BRAINSTORM §6. Summary: BSMS Round-1 ingest, BSMS verify-signatures, BSMS first-address hard-error mode, Bitcoin Core xprv handling, wallet-import-inspect flag for v0.12.0 dynamic widget, GUI dynamic per-cosigner widget, GUI argv-redaction, GUI roundtrip-diff panel, format parity with export-side vendors (Sparrow/Specter/etc.), env-var-sentinel-literal-escape, bip388-wallet-policy-import, sniff-bitcoin-core-tighten-heuristic, bsms-audit-field-regeneration.

---

**End of IMPLEMENTATION PLAN.**

## Self-review pass (writing-plans skill)

1. **Spec coverage** — every SPEC section is mapped:
   - §1-§2 (purpose + CLI surface) → Phase 6 SubcommandSchema + manual.
   - §3 (env-var sentinel) → Phase 1.
   - §4 (BSMS parser) → Phase 2.
   - §5 (Core parser) → Phase 3.
   - §6 (sniff) → Phase 5.
   - §7 (round-trip) → Phase 4.
   - §8 (module layout) → §1.1 file structure.
   - §9 (GUI lockstep) → Phase 6.
   - §10 (test corpus) → Phase 2/3/4 fixtures.
   - §11 (reviewer-loop) → per-phase architect-review steps.
   - §12 (cycle close) → Phase 7.

2. **Placeholder scan** — no TBDs, no "TODO", no "implement later". Every cell has a concrete name + expected behavior. The §4.3 BSMS taproot fixture is conditional ("if rust-miniscript supports") — that's a verified-at-Phase-0 conditional, not a placeholder.

3. **Type consistency** — `ParsedImport`, `WalletFormatParser`, `BsmsAuditFields`, `CosignerKeyInfo`, `ImportWalletWatchOnlyViolation`, `EnvVarMissing` used consistently. `.path` (not `.origin_path`) per SPEC §8.1.

4. **No spec gaps detected.**
