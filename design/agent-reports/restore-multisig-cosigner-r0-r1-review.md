# R0 Review (Round 1) — SPEC_restore_multisig_cosigner.md (v0.44.0)

**Reviewer:** opus `feature-dev:code-reviewer` (R0 mandatory pre-impl gate)
**Date:** 2026-06-04
**Source SHA:** `8bd705e` (branch `restore-multisig-cosigner`)
**Verdict:** 0 Critical / 3 Important / 7 Minor — **GATE: RED**

> Persisted verbatim per CLAUDE.md (the architect had no file-write tool; orchestrator persisted before folding). Fold log appended at bottom.

---

**Reviewed at SHA `8bd705e`.** Pins verified from `Cargo.lock`: `md-codec 0.35.0` (registry), `mk-codec 0.4.0` (registry), `miniscript 13.0.0` (git-rev `95fdd1c5`). Every cited API was re-read from source — not trusted from the SPEC or the prior exploration.

## Verification ledger (the implementability crux — all confirmed)

The load-bearing wsh/sh(wsh) reconstruction path **is genuinely implementable**. The single-sig cycle's descope does not recur. Evidence:

- `md_codec::chunk::reassemble(&[&str]) -> Result<Descriptor, Error>` — `chunk.rs:305`, `pub`, re-exported `lib.rs:43`. `decode_md1_string(&str) -> Result<Descriptor, Error>` — `decode.rs:79`, re-exported `lib.rs:45`. Both cited lines exact. The toolkit already calls `reassemble` at `bundle.rs:1049`.
- `Descriptor::is_wallet_policy(&self) -> bool` — `encode.rs:50` (exact). Tests `matches!(&self.tlv.pubkeys, Some(v) if !v.is_empty())` — present-and-non-empty `Pubkeys` TLV. Correct gate.
- `canonicalize::expand_per_at_n(&Descriptor) -> Result<Vec<ExpandedKey>, Error>` — `canonicalize.rs:420`, `pub`, `pub mod`. `ExpandedKey` (`:338`) fields exactly as cited and all `pub`: `idx: u8`, `origin_path: OriginPath`, `use_site_path: UseSitePath`, `fingerprint: Option<[u8;4]>`, `xpub: Option<[u8;65]>`. The 65-byte form is documented `[32 chain-code ‖ 33 compressed pubkey]` (`:347-349`) — matches the toolkit's `xpub_to_65` layout.
- `to_miniscript::to_miniscript_descriptor(&Descriptor, u32) -> Result<miniscript::Descriptor<DescriptorPublicKey>, Error>` — `to_miniscript.rs:53` (exact). Returns single-path (chain-resolved); the multipath `<0;1>/*` must come from `build_descriptor_string` — confirmed, the SPEC's compose is correct. Return type has `Display` (it's `miniscript::Descriptor`).
- **Taproot pre-gate is implementable WITHOUT calling `to_miniscript_descriptor`.** `Node.tag` is a `pub` field (`tree.rs:11`); `Tag` is a `pub enum` with variant `Tr` (`tag.rs:15,19`); both re-exported (`Tag` at `lib.rs:55`). So `d.tree.tag == Tag::Tr` is a valid pre-gate. `d.n` is `pub` (`encode.rs:19`).
- `template_from_descriptor(&MsDescriptor<DescriptorPublicKey>) -> Result<CliTemplate, ToolkitError>` — `wallet_export/mod.rs:262` (exact). Returns the four cited variants (`WshSortedMulti`/`WshMulti`/`ShWshSortedMulti`/`ShWshMulti`), discriminates sorted via `to_string().contains("sortedmulti(")` (`:267`), and **refuses `Tr`** (`:287`). `MsDescriptor` = `miniscript::Descriptor` alias (`pipeline.rs:10`), so `to_miniscript_descriptor`'s return type matches the param.
- `build_descriptor_string(template: CliTemplate, slots: &[ResolvedSlot], k: u8, network: CliNetwork, account: u32, taproot_internal_key: Option<TaprootInternalKey>) -> Result<String, ToolkitError>` — `pipeline.rs:18` (exact signature/order). Emits multipath `<0;1>/*` for wsh/sh-wsh multisig (`:82, :91-100`). **The load-bearing integration claim holds:** `ResolvedSlot` requires only `{xpub: Xpub, fingerprint: Fingerprint, path: DerivationPath}` (all other fields `None`-able — `synthesize.rs:642-686`), and all three are derivable from an `ExpandedKey` (the 65-byte xpub → `Xpub`, the `Option<[u8;4]>` fp → `Fingerprint`, the `OriginPath` → `DerivationPath`). `OriginPath.components` / `PathComponent.{hardened,value}` are all `pub` (`origin_path.rs:21,23,49`), and the toolkit **already imports and uses these types** (`synthesize.rs:14`), so the `OriginPath → DerivationPath` conversion is precedented. Checksum is correct by construction (`from_str`→`to_string` round-trip appends `#checksum`, `pipeline.rs:28-30`).
- `extract_multisig_threshold(&md_codec::tree::Node) -> Option<u8>` — `bundle.rs:1015` (exact), currently private `fn` (SPEC's `pub(crate)` bump is correct and sufficient). `&d.tree` (a `Node`) matches the param.
- `synthesize::xpub_to_65(&Xpub) -> [u8;65]` — **at `synthesize.rs:98`, NOT `:116` as cited** (see Minor). Returns `[chain_code‖compressed_pubkey]`; usable for the 65-byte compare. Because it reads only `chain_code`+`public_key` (ignoring depth/child/parent_fp), the §6.4 normalization trap is genuinely avoided.
- `derive_bip32_from_entropy_at_path(entropy, passphrase, language, network, path: &DerivationPath) -> Result<DerivedAccount, ToolkitError>` — `derive_slot.rs:65` (exact). `DerivedAccount` exposes `master_fingerprint` (the **master** fp via `master.fingerprint(&secp)`, `:83`) + `account_xpub` (at the supplied path, `:88`) + Zeroizing entropy. Usable for own-seed cross-check.
- First-address: `ms0.at_derivation_index(0)?.address(network)` — `at_derivation_index` (`miniscript .../descriptor/mod.rs:661`) and `address` (`:453`) both exist in miniscript 13; md-codec itself calls `at_derivation_index` (`derive.rs:122`). **Valid as cited.** Note a simpler purpose-built alternative exists: `md_codec::Descriptor::derive_address(chain, index, network)` (`derive.rs:92`, `pub`).
- `mk_codec::decode(&[&str]) -> Result<KeyCard>` — re-exported `mk-codec lib.rs:51`. `KeyCard.xpub: Xpub` (`key_card.rs:53`), `origin_fingerprint: Option<Fingerprint>` (`:36`), `origin_path` (`:42`). Usable for `--cosigner @N=mk1`.
- `RestoreMismatch { reference: &'static str, derived: String, expected: String, slot: Option<u8> }` — `error.rs:279-284` (exact), exit 4 (`:528`). Cross-check's `slot: Some(N)` is valid. Note `reference` is `&'static str` (cross-check must use a literal).

**Fingerprint-soundness (#5) — VERIFIED, not assumed.** `ExpandedKey.fingerprint` is documented the "4-byte **master** fingerprint" (`canonicalize.rs:345`). The emit path proves this: `synthesize.rs:791-795` populates `tlv.fingerprints` from `ResolvedSlot.fingerprint`, which for a seed-derived BIP-87 cosigner is `DerivedAccount.master_fingerprint`. So matching the own-derived master fp (`derive_bip32_from_entropy_at_path` → `master_fingerprint`) against `expand_per_at_n[i].fingerprint` is **sound** — both are master fps. The recurring master-vs-account-node fp bug class does **not** bite here. (A 65-byte-xpub-based inference would be marginally more robust — see Minor.)

---

## Critical

**None.** The wsh/sh(wsh) build path from `expand_per_at_n` + `build_descriptor_string`, the taproot pre-gate, the 65-byte cross-check, and the position-inference fingerprint semantics are all implementable and correct as architected. The crate-boundary visibility sweep (the class that descoped the prior cycle, and that surfaced one near-miss here — `xpub_from_tlv_bytes` is `pub(crate)`/unreachable) cleared: every type the toolkit must touch is `pub` in a `pub mod`, and the only md-codec→`DerivationPath` conversion the toolkit needs is reimplementable from `pub` fields it already imports.

## Important

**I1 — §7 lockstep over-specifies `lint_argv_secret_flags.rs` and, if followed literally, breaks the gate.** §7 says "declare the `--md1`/`--cosigner` routes." This is wrong. The gate (`tests/lint_argv_secret_flags.rs`) is three **set-equality closures** (`flag_axis`/`from_axis`/`slot_axis`, `:194-223`). Axis-1's live set is gui-schema flags with `secret==true` (driven by `secrets::flag_is_secret`, an allow-list at `secrets.rs:49-64`). `--md1`/`--cosigner` are non-secret (`--md1` explicitly excluded `secrets.rs:97`; `--cosigner` falls through to `false`) → they are in **none** of the three axes. Adding either to `FLAG_ROUTES` makes `declared ⊋ live` → the `stale` branch fails the test (`:196-197`). **Correct action: NO change to `lint_argv_secret_flags.rs`.** The existing `restore --from` (FROM_ROUTES `:119`) and `restore --passphrase` (FLAG_ROUTES `:102`) routes already cover restore's secret surface and stay as-is. Fix §7 to say "no new lint route — the new flags are non-secret; verify the three set-equality tests still pass unchanged."

**I2 — `Xpub` reconstruction from the 65-byte md1 form must set `network` from `--network`; the SPEC is silent.** md-codec's internal reconstruction hardcodes `NetworkKind::Main` (`derive.rs:57`, and it's `pub(crate)` so unreachable anyway). The toolkit must build its own `Xpub` from the 65 bytes for `ResolvedSlot.xpub`, and `synthesize`/emit consumers cross-check `xpub.network` against `--network` (`synthesize.rs:766`). If restore mirrors the hardcoded `Main`, a `--network testnet` restore emits mainnet `xpub` (and/or trips the network cross-check). Add to §4 step 4: when reconstructing each cosigner `Xpub`, set `network = network.network_kind()` (the md1 is network-agnostic — it carries only chain-code‖pubkey — so `--network` is the authority). Phase 1 must include a **testnet** multisig restore test exercising this; restore appears to be the **first** toolkit consumer of `expand_per_at_n` for descriptor reconstruction, so network + depth-0 reconstruction are genuinely novel input shapes (no prior call site to inherit correctness from).

**I3 — `--from` optionality mechanism is unspecified, and the change touches the single-sig `run`.** `RestoreArgs.from` is currently `pub from: String` — a mandatory clap arg (`restore.rs:57-58`), consumed unconditionally at `restore.rs:154` (`parse_from_input(&args.from)`). Making it optional in multisig mode requires `Option<String>`, which forces the single-sig path to re-assert the requirement. State the mechanism: `#[arg(long, required_unless_present = "md1")]` on `pub from: Option<String>` (verified: `required_unless_present` is a valid `Arg` builder usable as a raw clap attribute). This keeps single-sig `--from` mandatory without a separate runtime guard, but §3/§5 must specify it and update the `restore.rs:154` unwrap site. **Lockstep consequence:** the GUI schema `RESTORE_FLAGS` pins `--from { required: true }` (`mnemonic-gui/src/schema/mnemonic.rs:359`) — this must flip to `required: false` in the paired GUI PR (the `schema_mirror` gate is flag-NAME-only per CLAUDE.md so it won't catch this, but the GUI would mis-render `--from` as required; §7 omits this implication).

## Minor

- **M1 — `xpub_to_65` cited line is wrong.** SPEC §4 step 5 / §6.4 cite `synthesize.rs:116`; actual is `synthesize.rs:98`. (Line 116 is inside `build_descriptor`.) Citation-only; correct before write.
- **M2 — `derive_bip32_from_entropy_at_path` path family for sh(wsh).** The own-seed cross-check derives at "each cosigner's origin path." Confirm the implementation reads the **actual** per-cosigner `expand_per_at_n[i].origin_path` (which for sh(wsh)/BIP-48 script-type-1 is `m/48'/coin'/account'/1'`, NOT `m/87'/0'/0'`). §3 says "all `m/87'/0'/0'` for BIP-87" — true only for the wsh/BIP-87 case; the sh(wsh) case has a different origin and the code must use the per-`@N` origin, not a hardcoded BIP-87 path. The data supports this (origin is read per-`ExpandedKey`); just ensure the prose doesn't lead an implementer to hardcode `87'/0'/0'`.
- **M3 — prefer 65-byte xpub for position inference (robustness, optional).** Since restore derives the own key at each candidate origin anyway, comparing `xpub_to_65(derived) == expand_per_at_n[i].xpub` is strictly stronger than the master-fp match (it also catches a fp-collision and aligns with the project's "derive from unambiguous source" lesson). The fp match is correct as-is; this is a hardening suggestion, not a defect.
- **M4 — first-address: prefer the purpose-built helper.** `md_codec::Descriptor::derive_address(chain, index, network)` (`derive.rs:92`, `pub`) takes the md1 `Descriptor` directly and is simpler than re-deriving via `ms0.at_derivation_index(0)?.address(network)`. Both work; note the choice.
- **M5 — depth-0 reconstructed xpubs are a novel input to `build_descriptor_string`** (historically fed depth-3 account xpubs). Low risk (miniscript is depth-agnostic; `to_miniscript_descriptor` proves the form parses), but Phase 1 should assert the emitted descriptor's xpubs are depth-0 explicitly rather than assume. `<0;1>/*` is hardcoded in `build_descriptor_string` (ignores the md1's `use_site_path`) — consistent with single-sig restore; acceptable.
- **M6 — `multi` vs `sortedmulti` discrimination** is handled by `template_from_descriptor` (`to_string().contains("sortedmulti(")`, `mod.rs:267`) on the `to_miniscript_descriptor` output — no separate md1-tree inspection needed. Confirm the SPEC relies on this and not a manual tag check.
- **M7 — `account` param to `build_descriptor_string` is inert for this path** (it only feeds `key_origin_str`'s fallback, bypassed when `slot.path` is non-default — restore's slots always carry explicit origins). `account=0` vs the md1's account is a non-issue; do not add account-matching logic.

---

## Disposition note

This is a **first-round R0 with three confirmed SPEC defects** (I1 breaks an existing gate if followed; I2 is a real testnet correctness gap; I3 under-specifies a mechanism that touches the single-sig path). Per CLAUDE.md the SPEC must fold these and re-dispatch the architect before any code.

**VERDICT: 0 Critical / 3 Important**
**GATE: RED**

---

## Fold log (applied after persisting; each independently grep-verified)

- **I1 — FOLDED.** §7 corrected: NO change to `lint_argv_secret_flags.rs` (new flags non-secret; the three set-equality closures must still pass unchanged). Grep-verified `secrets.rs:97` excludes `--md1`; the gate is set-equality.
- **I2 — FOLDED.** §4 step 4/5 now mandate reconstructing each cosigner `Xpub` with `network.network_kind()` (md1 is network-agnostic; `--network` authoritative). Phase 1 gains a testnet cell. Grep-verified `md-codec derive.rs:57` hardcodes `Main` + is `pub(crate)`.
- **I3 — FOLDED.** §3/§5 now specify `pub from: Option<String>` + `#[arg(required_unless_present = "md1")]` + the `restore.rs:154` unwrap-site update; §7 GUI lockstep now flips `--from { required: false }`. Grep-verified `RestoreArgs.from: String` mandatory at `restore.rs:57`, consumed `:154`; GUI `RESTORE_FLAGS --from required:true`.
- **M1 — FOLDED.** `xpub_to_65` citation `synthesize.rs:116` → `:98`.
- **M2 — FOLDED.** §3/§4 prose clarified: use the per-`@N` `expand_per_at_n[i].origin_path` (sh(wsh) is `m/48'/.../1'`, not BIP-87); do NOT hardcode `87'/0'/0'`.
- **M5 — FOLDED.** Phase 1 asserts emitted-descriptor xpubs are depth-0.
- **M3/M4/M6/M7 — noted; M3 (65-byte position inference) adopted as the stronger check.**
- Re-dispatched R0 round 2 after fold.
