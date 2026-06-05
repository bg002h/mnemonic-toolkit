# mnemonic restore — SPEC R0 Review (round 0)

**Verdict: RED (1C / 5I).** Fold + descope → re-dispatch.

Built on overwhelmingly accurate citations; single-sig (P1) design is GREEN-ready with zero private-key-leakage path. RED on the P2 multisig route (C1 — structurally non-implementable as cited) + 5 Importants (the SPEC's self-flagged weak areas: multisig + `--format` composition). **Resolution: descope to single-sig P1 as v0.43.0** (SPEC §8 authorized this); fold the P1-touching Importants (I1→--format-requires-template, I2, I3, I5) + Minors now; **defer multisig (C1, I4, I1-multi) to a follow-on SPEC+R0** on the correct `to_miniscript_descriptor` / policy-params bridge.

## Critical

**C1 — Multisig route 1 has a STRUCTURALLY-WRONG type chain (§3.3 step 1, §4).** `template_from_descriptor` (`wallet_export/mod.rs:262`) consumes a **miniscript** `&MsDescriptor<DescriptorPublicKey>`, NOT an `md_codec::Descriptor` — passing `reassemble()`'s output is a type error. `md_codec::Descriptor` has **no `Display`** (so the recon's "route 2" render-to-string is also unavailable). The real bridge is `md_codec::to_miniscript::to_miniscript_descriptor(d, chain)` (`md-codec-0.35.0/src/to_miniscript.rs:53`) — **uncited**, AND it `expand_per_at_n`s keys → errors `MissingPubkey` on a template-only md1 (exactly the restore use case), and is unused/untested in the toolkit today. Also `extract_multisig_threshold` (`bundle.rs:1015`) is a **private fn** (not pub(crate)) — the SPEC's blanket "no CLI-only refactor / every helper pub(crate)" (§4) is false for it. The production md1→concrete path (`bundle_run_unified_descriptor`, `bundle.rs:1138`) operates on a `--descriptor` STRING via lex/resolve/parse/bind, never on a reassembled `md_codec::Descriptor`→`CliTemplate`. → DEFER multisig; re-spec on the correct bridge (or the policy-params→CliTemplate route, or `--descriptor`-string-only input) in a fresh R0. Does NOT affect P1.

## Important

- **I1 — `--format` + single-sig all-4 is undefined; "bitcoin-core packs several descriptors" is INACCURATE (§3.5, §2).** `BitcoinCoreEmitter::emit` (`bitcoin_core.rs:24`) is one-descriptor-in; `format_bitcoin_core_importdescriptors` splits ONE descriptor's `<0;1>` into 2 (receive/change), errors if `!=2`. Not 4 BIP types. Every emitter is 1-in/1-out. **Fold (P1): `--format` REQUIRES a single `--template` (refuse `--format` with the all-4 default).**
- **I2 — `--from` node-scope rejection unspecified (§2, §11.1).** `convert::NodeType::from_token` accepts 14 nodes; restore scopes to 4. Precedent `addresses.rs:223` rejects non-seed nodes with `BadInput` (exit 1). **Fold: name the gate + `BadInput` exit 1 for non-{ms1,phrase,entropy,seedqr}.**
- **I3 — `details()` arm + JSON-error envelope (§5).** error.rs has FOUR `match self` blocks: exit_code(:472)/kind(:530)/message(:589) forced-exhaustive + `details()`(:776) with `_=>None`. **Fold: state restore's mismatch surfaces via `message()` only (no JSON-error envelope), so the 3-block obligation is complete.**
- **I4 — md1 auto-detect via `tlv.pubkeys` only works in wallet-policy mode (§3.3 step 3).** `desc.tlv.pubkeys: Option<Vec<(u8,[u8;65])>>` is `Some` only when `is_wallet_policy`; template-only md1 → positional fallback. (Multisig — DEFERRED with C1; the follow-on spec must spell out wallet-policy-vs-template-only branches.)
- **I5 — `b4e3f5ed` (TREZOR-passphrase fp, §9) is UNVERIFIABLE from source.** Only `73c5da0a` (no-pp) is asserted in-tree (`cli_export_wallet.rs:27`). **Fold: implementer must derive+confirm `b4e3f5ed` before baking the P1 test (`feedback_recapture_golden_only_when_current_correct`).** (Controller already confirmed b4e3f5ed at runtime this session via `convert --to fingerprint --passphrase`.)

## Minor (fold)
- **M1** HEAD is `7dfba5c` (SPEC's own commit); `6566941` is the base. Update prose.
- **M2** cli_gui_schema 28-count lives at `:74`/`:108` (not `:3`/`:37`); alpha slot (after `repair`, before `seed-xor-combine`) ACCURATE.
- **M3** `extract_multisig_threshold` def is `bundle.rs:1015` (`:1060` is the use); private (see C1).
- **M4** `DerivedAccount.entropy` is `Zeroizing<Vec<u8>>` (do-not-emit `account_xpriv` point still ACCURATE).
- **M5** `resolve_slots` returns `Vec<(u8,&'static str)>`.
- **M6** `cmd/mod.rs` list alpha-ish but drifted (`silent_payment` before `seed_xor`); `restore` slots after `repair` before `silent_payment` — don't "fix" the existing drift.

## Verification ledger (highlights — all citations checked against source)
P1 load-bearing APIs ALL ACCURATE: `derive_bip32_from_entropy` (`derive_slot.rs:42`), `DerivedAccount` incl `account_xpriv` (`derive.rs:24-37`), `build_descriptor_string` (`pipeline.rs:18`), `render_address_from_xpub` (`address_render.rs:18`), `resolve_slots` (`bundle.rs:453`), `read_stdin_passphrase`/`_to_string` (`convert.rs:719`/`706`), `resolve_env_var_sentinel` (`env_sentinel.rs:56`), `pin_pages_for` (`mlock.rs:90`), `secret_advisory::*` (`:40,83,97`), `is_argv_secret_bearing` (`convert.rs:117`), both `@env:` channels usable (`addresses.rs:153,162`), `CliExportFormat` 11 + dispatch (`export_wallet.rs:22-46,507-561`), `RestoreMismatch` alpha slot + exit-4 tier ACCURATE, `slot_input.rs` has NO `mk1` subkey (§11.3 ACCURATE), `ResolvedSlot` carries fingerprint+path origin (`synthesize.rs:642`; `key_origin_str` `pipeline.rs:33`), `flag_is_secret` classifies passphrase*/ms1 not --from (`secrets.rs:49-64`), GUI/manual lockstep cites ACCURATE. Sibling (md 0.35 / mk 0.4.0) re-grepped: `chunk::reassemble` (`chunk.rs:305`), `mk_codec::decode`+`KeyCard.xpub` ACCURATE; the C1 type chain WRONG.

**Bottom line:** P1 single-sig is GREEN-ready (zero leakage). Descope to P1 v0.43.0, fold I1/I2/I3/I5 + Minors, defer multisig to a fresh SPEC+R0 on the correct bridge.
