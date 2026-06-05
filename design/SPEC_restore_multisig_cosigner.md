# SPEC — `mnemonic restore` multisig-cosigner (v0.44.0)

**FOLLOWUP:** `restore-multisig-cosigner-scope`
**Source SHA (master at write time):** `8bd705e`
**Cycle type:** MINOR (`v0.43.1` → `v0.44.0`). New CLI surface on the existing `restore` subcommand.
**Lockstep:** GUI `schema_mirror` (paired `mnemonic-gui` PR) + `cli_gui_schema.rs` self-test + manual `41-mnemonic.md` restore chapter + `lint_argv_secret_flags`.
**Predecessor:** single-sig `restore` shipped v0.43.0 (`design/SPEC_mnemonic_restore.md`). This cycle builds the §11-deferred multisig half.
**Exploration:** code-explorer report (agent `ac521d83d375e3b36`) + runtime probes (2026-06-04, this session). Both recorded below.

---

## 1. The reframe (runtime-verified) — md1 is a WALLET-POLICY card

**The decisive fact that reorders SPEC_mnemonic_restore §11:** every md1 the constellation emits — single-sig AND multisig — carries the **concrete per-`@N` cosigner xpubs** in `tlv.pubkeys`, not `@N` placeholders. Runtime-proven this session: decoding a real toolkit-emitted 2-of-3 `wsh-sortedmulti` md1 gave `is_wallet_policy = true`, `n = 3`, and `expand_per_at_n` returned 3 concrete cosigners (each `fingerprint = Some(..)`, `xpub = Some(..)`, origin `m/87'/0'/0'`), and `to_miniscript_descriptor(&d, 0)` produced:

```
wsh(sortedmulti(2,[73c5da0a/87'/0'/0']xpub6.../0/*,[b8688df1/87'/0'/0']xpub6.../0/*,[28645006/87'/0'/0']xpub6.../0/*))#y65a0dtg
```

**Consequence:** the concrete watch-only multisig descriptor is reconstructible from **the md1 alone**. The user's own seed + the other cosigners' `mk1`/`xpub` are therefore **cross-check inputs, not build inputs** (this inverts the FOLLOWUP/§11 framing, which assumed the user supplies cosigner keys to *build* — that only describes a template-only md1, which the toolkit never emits). The shipped feature is a strict **superset** of the user's original ask: `restore --md1 <chunks> [--from <seed>] [--cosigner @N=mk1|xpub …]` reconstructs the descriptor and cross-checks whatever verification material is supplied.

## 2. Scope

**IN (this cycle):**
- `wsh(sortedmulti|multi)` (BIP-48 script-type 2) and `sh(wsh(sortedmulti|multi))` (BIP-48 script-type 1) multisig restore from a wallet-policy md1.
- Output: the concrete multipath `<0;1>/*` descriptor + BIP-380 checksum (matching single-sig restore's output shape), first receive address(es), and a per-cosigner verification block.
- Optional cross-check: own seed (`--from`, position inferred by fingerprint) and/or cosigner keys (`--cosigner @N=mk1|xpub`) against the md1's per-`@N` keys.
- Same fingerprint hard-gate semantics as single-sig: a cross-check mismatch is `RestoreMismatch` (exit 4); `--allow-mismatch` overrides; with no cross-check input the output carries the `UNVERIFIED` banner. Watch-only-out (NEVER xpriv/WIF).

**OUT (refused this cycle, with a pointer):**
- **Taproot multisig** (`tr(sortedmulti_a|multi_a)`). Runtime-proven blocked: `to_miniscript_descriptor` on a real `tr-sortedmulti-a` md1 errors `AddressDerivationFailed { "Tag::SortedMultiA must be a tap-leaf root child; rust-miniscript v13 has no Terminal::SortedMultiA fragment" }`. The toolkit refuses taproot reconstruction everywhere (from-import-json refuses it; `template_from_descriptor` refuses `Tr`, `wallet_export/mod.rs:~287`; the descriptor emitter only passes through a user-supplied string). md1→tr-descriptor reconstruction needs a bespoke string emitter + BIP-386 checksum and is its own mini-project. → refuse (exit 2) + new FOLLOWUP `restore-multisig-taproot-reconstruction`.
- **Template-only md1** (`!is_wallet_policy()` — `tlv.pubkeys` absent). The toolkit never emits one; a foreign template-only md1 has no concrete keys to reconstruct from → refuse (exit 2) + note (this is §11's I4 "auto-detect" branch, deferred).
- **`--format <export-format>` for multisig** (importable wallet payloads). Single-sig restore's `--format` requires a single `--template`, which does not fit multisig. → refuse `--format` in multisig mode (exit 2) for v0.44.0; FOLLOWUP `restore-multisig-format-payloads`.

## 3. Mode dispatch + CLI surface

**Mode trigger:** `--md1` present → **multisig restore mode** (md1-driven). The existing single-sig path (driven by `--from <seed>` + optional `--template`) is unchanged; the `is_multisig()` `--template` rejection (`restore.rs:170`) stays for single-sig mode.

**New flags on `RestoreArgs`:**
- `--md1 <STRING>` — repeating; the multisig md1 card chunk(s). Reassembled via `md_codec::chunk::reassemble(&[&str])`. **Required** for multisig mode. (Watch-only material → NOT secret.)
- `--cosigner <@N=mk1|xpub>` — repeating, **optional**. Cross-check assertion: cosigner at position `N` is this key. `mk1` decoded via `mk_codec::decode(&[&str])` → `KeyCard.xpub`; a raw `xpub` parsed directly. (Watch-only → NOT secret.)

**Reused/relaxed:**
- `--from` becomes **optional** in multisig mode (md1 alone reconstructs; `--from` cross-checks the own position). **Mechanism (R0-r1 I3):** change `RestoreArgs.from` from `pub from: String` (mandatory, `restore.rs:57`) to `pub from: Option<String>` with `#[arg(long, required_unless_present = "md1")]` — single-sig `--from` stays mandatory without a runtime guard; multisig (`--md1` present) allows its absence. Both current consumption sites must handle the `Option`: `restore.rs:154` (`parse_from_input(&args.from)`) and `restore.rs:209` (`args.from.split('=')`). Still secret-bearing (unchanged taxonomy).
- `--account`, `--network`, `--language`, `--passphrase`/`--passphrase-stdin`, `--count`, `--expect-fingerprint`, `--allow-mismatch`, `--json`, `--output` carry over.
- `--expect-xpub` (single-sig, requires `--template`) is N/A in multisig mode (refuse if combined, exit 2).

**Position inference:** derive the own seed at each cosigner's **per-`@N`** origin path (`expand_per_at_n[i].origin_path` — `m/87'/0'/0'` for wsh/BIP-87, `m/48'/coin'/account'/1'` for sh(wsh); read per-`ExpandedKey`, never hardcoded) → match `xpub_to_65(derived)` against `expand_per_at_n[i].xpub` (the 65-byte form is the inference key; stronger than a master-fp match). The matching `i` is the own position. No match across all positions → `RestoreMismatch` ("supplied seed is not a cosigner of this wallet").

## 4. Build + cross-check pipeline (option (c) + compose)

1. **Decode:** `md_codec::chunk::reassemble(&md1_chunks)` (or `decode_md1_string` for a single string) → `md_codec::Descriptor`. (`chunk.rs:305` / `decode.rs:79`.)
2. **Gate:** `d.is_wallet_policy()` (`md-codec encode.rs:50`) must be true → else template-only refusal (§2). Detect taproot via `d.tree.tag == Tr` **before** `to_miniscript_descriptor` (which errors unhelpfully) → taproot refusal (§2).
3. **Reconstruct miniscript:** `md_codec::to_miniscript::to_miniscript_descriptor(&d, 0)` (`to_miniscript.rs:53`) → `miniscript::Descriptor<DescriptorPublicKey>` (`ms0`, chain-0). Used for (a) classification, (b) first-address derivation (`ms0.at_derivation_index(0)?.address(network)`).
4. **Output descriptor (multipath `<0;1>/*`):** classify `ms0` → `CliTemplate` via `template_from_descriptor(&ms0)` (`wallet_export/mod.rs:262`; handles `Wsh`/`Sh(Wsh)`, refuses `Tr` — taproot already refused at step 2; `multi`-vs-`sortedmulti` is discriminated inside via `to_string().contains("sortedmulti(")`, `mod.rs:267` — no manual md1-tree tag inspection, R0-r1 M6). Build the cosigner `ResolvedSlot`s from `expand_per_at_n(&d)` — each carries `{xpub, fingerprint, path}` (all other `ResolvedSlot` fields `None`): the **65-byte `ExpandedKey.xpub` → `Xpub`** reconstruction MUST set `network = network.network_kind()` from `--network` (R0-r1 I2 — the md1 is network-agnostic, carrying only chain-code‖pubkey; md-codec's own `xpub_from_tlv_bytes` hardcodes `Main` and is `pub(crate)`/unreachable, so restore builds the `Xpub` itself with the `--network` authority); the `Option<[u8;4]>` fp → `Fingerprint`; the per-`@N` `OriginPath` → `DerivationPath` (`OriginPath.components`/`PathComponent.{hardened,value}` are `pub`, already used at `synthesize.rs:14`). Emit via `build_descriptor_string(template, &slots, k, network, account, None)` (`wallet_export/pipeline.rs:18`), where `k = extract_multisig_threshold(&d.tree)` (`bundle.rs:1015`, **bump to `pub(crate)`**) and `n = d.n`. This matches single-sig restore's `<0;1>/*` output shape (`restore.rs:44-45`). (`account` is inert here — `slots` carry explicit origins — so no account-matching logic, R0-r1 M7.)
5. **Cross-check (65-byte form, NEVER `Xpub ==`):** the three key sources normalize `depth`/`child_number`/`parent_fingerprint` differently (own-derivation: real; mk1: reconstructed from origin; md1 tlv: forced `depth:0`). Compare the **65-byte `[chain_code‖compressed_pubkey]`** only (`synthesize::xpub_to_65`, `synthesize.rs:98`):
   - own seed (if `--from`): for each candidate position `i`, derive at `expand_per_at_n[i].origin_path` (the **per-`@N`** origin — `m/87'/0'/0'` for wsh/BIP-87, `m/48'/coin'/account'/1'` for sh(wsh)/BIP-48 type-1; do NOT hardcode BIP-87, R0-r1 M2) → compare `xpub_to_65(derived)` to `expand_per_at_n[i].xpub` (R0-r1 M3: the 65-byte compare is the position-inference key — strictly stronger than the master-fp match and avoids fp-collision). The matching `i` is the own position; no match across all positions → `RestoreMismatch` ("supplied seed is not a cosigner of this wallet").
   - each `--cosigner @N`: `mk_codec::decode`/`xpub` → 65-byte vs `expand_per_at_n[N].xpub`.
   - any mismatch → `RestoreMismatch { reference: <&'static str literal>, derived, expected, slot: Some(N) }` (exit 4; variant fields `error.rs:279-284`, `reference` is `&'static str`) unless `--allow-mismatch`. No cross-check input at all → `UNVERIFIED` banner (reuse single-sig machinery).
6. **Emit:** a multisig restore document — the concrete descriptor, first receive address(es), and a per-cosigner verification table (position, fingerprint, origin, "← your seed" marker, match/UNVERIFIED). `--json` object + `--output` redirect mirror single-sig. Watch-only-out invariant: no xpriv/WIF ever (test-enforced).

## 5. Integration point in `restore.rs`

A sibling `run_multisig` dispatched from `run` when `--md1` is present (before the single-sig `is_multisig()` template guard). Reuses: `parse_from_input` + the seed→entropy block (`:248-283`, for the optional own-seed cross-check via `derive_bip32_from_entropy_at_path`), the `RestoreMismatch`/`--allow-mismatch`/`UNVERIFIED` machinery (`:366-406`, `:522-538`), the `WatchOnly` advisory, and `--json`/`--output` routing. A new `MultisigRestore` result shape (one descriptor + N cosigner positions + the own-position marker) parallels `WalletRow` (single-key, `:138`). Keep `run_multisig` in a focused new module section; do not bloat the single-sig `run`.

## 6. The five SPEC §11 divergences (recorded so R0 need not re-derive)

1. **(a)/(c) inversion:** §11 recommended option (a) (reuse `bundle_run_unified_descriptor` + user-supplied descriptor); the explorer + runtime proof show (a) is **BLOCKED** (emits cards not a descriptor; takes `&BundleArgs`; needs the `Display` md1 lacks) and **(c)** (`to_miniscript_descriptor`, which DOES have `Display`) is the viable path, gated on `is_wallet_policy()`. Option (a) is dropped.
2. **md1 is wallet-policy:** keys come FROM the md1; own seed + cosigners are cross-check, not build, inputs (§1).
3. **Output-shape:** §11 omits the single-path-vs-multipath question. Decided: multipath `<0;1>/*` via the (c)→`template_from_descriptor`→`build_descriptor_string` compose, matching single-sig restore.
4. **Cross-check normalization trap:** compare the **65-byte chain-code‖pubkey form**, never reconstructed `Xpub ==` (the three sources normalize depth/child/parent_fp differently). This is a real correctness bug avoided.
5. **Taproot blocked at the library layer:** `template_from_descriptor` refuses `Tr` AND `to_miniscript_descriptor` errors on `SortedMultiA` (rust-miniscript v13) — taproot reconstruction exists nowhere in the toolkit → refused this cycle + FOLLOWUP.

## 7. SemVer / lockstep

- **SemVer:** MINOR (`v0.44.0`) — new flags (`--md1`, `--cosigner`) on the `restore` subcommand; new functionality.
- **GUI `schema_mirror`** (`mnemonic-gui/src/schema/mnemonic.rs` `RESTORE_FLAGS`): add `--md1`, `--cosigner` (both public/non-secret) AND flip the restore `--from` entry `required: true` → `required: false` (R0-r1 I3 — `schema_mirror` is flag-NAME-only so it won't catch the `required` drift, but the GUI would mis-render `--from` as mandatory; the restore `--from` is at `mnemonic-gui/src/schema/mnemonic.rs:357-359`, distinct from the import-wallet/export-wallet `--from` entries which stay required). Paired `mnemonic-gui` PR (lagging gate; the paired-PR rule is the leading discipline).
- **Toolkit self-test** `tests/cli_gui_schema.rs`: the `restore` flag-name set changes → update; no subcommand-count change.
- **`lint_argv_secret_flags.rs`** (leading gate): **NO new route** (R0-r1 I1). The gate is three set-equality closures (`flag_axis`/`from_axis`/`slot_axis`, `:194-223`) over the gui-schema `secret==true` surface; `--md1`/`--cosigner` are non-secret (`--md1` already excluded `secrets.rs:97`; `--cosigner` falls through to `false`), so they belong to NONE of the three axes — adding either to `FLAG_ROUTES` would make `declared ⊋ live` and FAIL the gate. The existing `restore --from` (FROM_ROUTES) / `restore --passphrase` (FLAG_ROUTES) routes cover restore's secret surface unchanged. Phase 3 just verifies the three set-equality tests still pass.
- **Manual** `41-mnemonic.md`: the deferred-note (`:737-741`) flips to a real `### Multisig-cosigner restore` subsection (flags + worked example + the wallet-policy/taproot-refusal notes); a recovery recipe in `30-workflows/35-recovery-paths.md`. `docs/manual/tests/cli-subcommands.list` already has `restore`; the flag-coverage lint requires the new flags documented.
- **Sibling-codec companions:** none (toolkit-only; md-codec/mk-codec consumed as-is — only `extract_multisig_threshold` visibility bump is local to the toolkit).

## 8. Phased plan

**Phase 1 — TDD RED.** New `tests/cli_restore_multisig.rs`: (1) `--md1` (2-of-3 wsh-sortedmulti) alone → concrete `<0;1>/*` descriptor + first address + `UNVERIFIED`; (2) `--md1 --from <own seed>` → position inferred + cross-check ok (verified, no UNVERIFIED); (3) `--md1 --cosigner @N=mk1` (+ xpub) → cross-check ok; (4) `--md1 --from <wrong seed>` → `RestoreMismatch` exit 4; (5) `--allow-mismatch` override → exit 0 + banner; (6) `sh-wsh-sortedmulti` 2-of-3 round-trip (exercises the non-BIP-87 per-`@N` origin path, R0-r1 M2); (7) `tr-sortedmulti-a` md1 → refusal exit 2 + FOLLOWUP pointer; (8) watch-only-out: assert NO xpriv/WIF in any output (argv + stdout); (9) **`--network testnet`** 2-of-3 restore → descriptor carries `tpub` cosigners (R0-r1 I2 — guards the `--network`-authoritative `Xpub` reconstruction; restore is the first `expand_per_at_n` reconstruction consumer); (10) assert the emitted descriptor's cosigner xpubs are **depth-0** (R0-r1 M5 — novel input to `build_descriptor_string`). Fixtures: generate md1 via `bundle --template wsh-sortedmulti --threshold 2 …` at test time (or pinned constants). RED against the unpatched binary (multisig `--md1` → today errors / unrecognized). First-address derivation: use `md_codec::Descriptor::derive_address(chain, index, network)` (`derive.rs:92`, purpose-built) OR `ms0.at_derivation_index(0)?.address(network)` (R0-r1 M4 — note the choice).

**Phase 2 — Implement + GREEN.** `extract_multisig_threshold` → `pub(crate)`; `run_multisig` + decode/gate/reconstruct/cross-check/emit pipeline; new flags on `RestoreArgs`. Per-phase opus architect review → persist → fold to 0C/0I. Full `cargo test --no-fail-fast` workspace suite green.

**Phase 3 — Lockstep.** GUI `RESTORE_FLAGS` (+ `secret:false` for both); `cli_gui_schema.rs` restore entry; `lint_argv_secret_flags` routes; manual `### Multisig-cosigner restore` subsection + recovery recipe + flag-coverage lint green. (GUI is a paired `mnemonic-gui` PR — author it in lockstep; the toolkit-side `cli_gui_schema.rs` self-test must pass regardless.)

**Phase 4 — Release prep (v0.44.0).** Cargo.toml→0.44.0 + stage Cargo.lock; CHANGELOG; both README markers; install.sh pin `mnemonic-toolkit-v0.44.0`; FOLLOWUPS `restore-multisig-cosigner-scope` → resolved + new `restore-multisig-taproot-reconstruction` + `restore-multisig-format-payloads` filed; end-of-cycle opus review → 0C/0I.

**Phase 5 — Ship + tag.** Clean tree; checkout master → ff-only → tag `mnemonic-toolkit-v0.44.0` → push master+tag. Paired `mnemonic-gui` PR for the schema mirror.

## 9. Mandatory R0 gate

Per `CLAUDE.md`: this SPEC must pass an **opus architect R0 review to 0C/0I BEFORE any code.** Persist each round to `design/agent-reports/restore-multisig-cosigner-r0-rN-review.md`; fold → re-dispatch until GREEN. The architect must read source at `8bd705e`, re-verify every cited API/line (esp. `expand_per_at_n`/`ExpandedKey` fields, `to_miniscript_descriptor`, `template_from_descriptor` Tr-refusal, `build_descriptor_string` signature + multisig forms, `extract_multisig_threshold`, `RestoreMismatch` variant fields, `mk_codec::decode`/`KeyCard`), and confirm the wsh/sh-wsh build path + the 65-byte cross-check are implementable as written (the single-sig cycle's R0 descoped multisig precisely for an implementability gap — this R0 must not repeat it).
