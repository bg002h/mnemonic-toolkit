# R0 Review — faithful general-policy restore (PART 1) — ROUND 1

**Source SHA:** `5d599f7` (toolkit) · sibling `descriptor-mnemonic` md-codec read at workspace HEAD · pinned miniscript verified at `~/.cargo/git/checkouts/rust-miniscript-ce5fa57e8900265e/95fdd1c`.

**Verdict: 🟡 — 0 Critical / 4 Important / 7 Minor**

## Pivot + translate_pk + discriminator INDEPENDENTLY VERIFIED

**Pivot (crux 1) — CONFIRMED from source.** `restore.rs:839` computes `ms0 = to_miniscript_descriptor(&d, 0)` and `:841` discards it into `template_from_descriptor` whose `Wsh(_) =>` arm (`wallet_export/mod.rs:283-287`) collapses any wsh; `:881-882 build_descriptor_string` rebuilds plain `wsh(multi(k,…))` with hardcoded `<0;1>` (`pipeline.rs:82,91`). I walked every arm of md-codec `to_miniscript.rs::node_to_descriptor`/`node_to_miniscript` (:134-462): each tag maps 1:1 onto a `Terminal`, timelocks are full-width `from_consensus` (:412-418), hash digests byte-exact (:420-435), `Multi` preserves index order (:385-392), and every conversion passes `Miniscript::from_ast` type-checking. **No shape that succeeds but mis-renders** — the only re-interpretation sites are the `PkK`/`PkH` arms (:290-302) which re-apply `Check`; either correct or `from_ast` errors LOUDLY. `pk(@N)`/`pkh(@N)` toolkit shapes (`Check(Check(PkH))`) error at `from_ast` → `Err`, never panic/collapse. Watch-only by construction (`DescriptorPublicKey::XPub` from `[chain_code‖pubkey]`, :67-92).

**translate_pk (crux 2) — API CONFIRMED at rev 95fdd1c.** `Descriptor::translate_pk` (`descriptor/mod.rs:405`), `Translator` (`lib.rs:291`), `translate_hash_clone!` (`pub_macros.rs:96`), `MultiXPub(DescriptorMultiXKey{derivation_paths: DerivPaths, wildcard})` (`descriptor/key.rs:29,97-103`), `sanity_check` (`descriptor/mod.rs:317`), checksum via Display (`mod.rs:1182-1195`). 3 caveats real: md-codec `derive.rs:49-64` hardcodes `NetworkKind::Main`, depth-0, default parent-fp → **byte-equality with `export-wallet --descriptor` genuinely impossible**, SPEC right to reject that oracle. `use_site_path.multipath` is `pub` (`use_site_path.rs:51`). `[patch.crates-io]` unifies miniscript — `ms0` IS the toolkit's type. NOT verifiable: the Translator (doesn't exist yet) — see I3/M6.

**Discriminator (crux 3) — SOUND.** Decode-time `validate_placeholder_usage` (md-codec `decode.rs:56`) rejects unreferenced/out-of-order-first-occurrence placeholders → only constructible non-identity plain shape is duplicate-bearing (`multi(2,@0,@1,@0)`), correctly routed to faithful (plain arm would silently drop the dup). Plain-arm byte-identity holds (both render via Display, identical keys); the consistency unit test gates it. All fixtures are toolkit-emitted plain bundles → stay on the untouched path.

**Addresses (crux 4) — self-consistent**: general arm derives from the EMITTED string via `derive_receive_addresses` (`derive_address.rs:79-118`), multipath-aware; print and address can't diverge.

**Format emitters (crux 5) — traced with `template: None` + `P2wshMulti`:** coldcard refuses (`coldcard.rs:111-115`), coldcard-multisig (`export_wallet.rs:114-127`), jade (`jade.rs:36-40`), electrum (`electrum.rs:52-56`), sparrow (`sparrow.rs:104-108`), green on `is_multisig()` (`green.rs:36-40`), specter via `wallet_name_is_non_default==false` (`specter.rs:34-36`, restore passes `false` `restore.rs:706`). bitcoin-core/descriptor/bip388 descriptor-driven + faithful. **BSMS does NOT refuse — see I2.**

## Critical
None. No path to a silent wrong/partial reconstruction found — every general-arm failure mode is loud or faithful.

## Important
**I1 — The k-gate blocks the flagship pk-keyed policy BEFORE the clear refusal, and falsifies the PART-2 "zero toolkit changes" claim.** `extract_multisig_threshold` (`bundle.rs:1036-1045`) matches only `MultiKeys`/`Variable`(thresh). The flagship `wsh(andor(pkh(@0),after(12000000),or_i(and_v(v:pkh(@1),older(4032)),and_v(v:pkh(@2),older(32768)))))` has neither → `restore.rs:843-844` errors `"--md1 is not a multisig descriptor (no threshold present)"` (cryptic k-gate), never reaching the clear pk-keyed refusal. And "after PART 2, zero toolkit changes" is FALSE for pk-keyed-without-multi (k-gate refuses forever). Fix: make `k` optional for the general arm (only labels/`EmitInputs.threshold` consume it); gate "is multisig" on the plain arm only; route key-bearing wallet-policy md1s without a threshold into `faithful_multisig_descriptor`. Choose the pkh-leaf RED-cell to exercise the new error path (a pkh-leaf shape that ALSO contains a multi; the flagship doesn't).

**I2 — BSMS refusal claim factually wrong.** `bsms.rs::emit` (:64-119) gates ONLY `P2tr|P2trMulti`; with `template:None`+`P2wshMulti` it EMITS (faithfully — line 2 = descriptor verbatim). Not Critical (payload faithful) but the SPEC's mandated bsms refusal cell won't pass. DECIDE: (a) accept faithful BSMS emit (BIP-129 line 2 is a descriptor record — defensible) + pin THAT, or (b) add explicit refusal to `bsms.rs` (new code, specify). Un-deciding guarantees implementer drift.

**I3 — `multipath == None` translator arm unspecified, constructible.** `bundle --descriptor "wsh(multi(2,[fp/…]xpub/*,…))"` (wildcard-only, no `<0;1>`) → md1 with `use_site_path.multipath == None` (alternatives min count 2, `use_site_path.rs:42-44`). Discriminator routes to faithful; translator recipe would unwrap-panic or fabricate `<0;1>`. Specify: `multipath:None` → pass `XPub` network-corrected only (keep single/wildcard, do NOT promote to `MultiXPub`) + test cell. `ms0`'s per-key path already empty here (`to_miniscript.rs:116-130`).

**I4 — Label fold incomplete.** SPEC fixes `:1139 wallet_type` + `:1158` header but misses `restore.rs:1189` (`--format` stderr `"{k}-of-{n} multisig restore"`) and the JSON envelope top-level `"threshold": k` (`:1136`). Decay vault `wsh(or_d(multi(2,…),and_v(v:multi(3,…),older(…))))` reports `"threshold": 2` as if it were the wallet threshold. Enumerate ALL changed/retained wire fields in the GUI paired-PR FOLLOWUP.

## Minor
- **M1 — Citation drift:** cell counts are 13 (`cli_restore_multisig.rs`) + 12 (`cli_restore_multisig_format.rs`), not 10+16; `ms0` at `restore.rs:839` not `:838`. Re-grep per write-time rule.
- **M2 — Silent scope widening to legacy `sh(…)`:** `sh(multi)`/`sh(<ms>)` md1 currently refuses loudly at `template_from_descriptor`'s `ShInner::Ms` arm (`wallet_export/mod.rs:279-281`); new routing reconstructs it faithfully (`to_miniscript.rs:241-253` Legacy). Good, but outside the stated "wsh/sh(wsh)" boundary — pin with a cell or gate.
- **M3 — "permuted-indices" RED cell unconstructible** (`validate_placeholder_usage`). Use a duplicate-index fixture (`wsh(multi(2,@0,@1,@0))`).
- **M4 — Refusal-message ergonomics:** inherited refusals say "requires --template"/"supply --slot @N.xpub=" (flags absent on restore). Loud, acceptable; consider a restore-side message audit.
- **M5 — General-arm `EmitInputs` under-specified:** `wallet_name` derives from `template.human_name()` (`restore.rs:694`) — `None` arm needs a fallback (e.g. `"imported-descriptor"`); state whether `threshold`/`threshold_user_supplied` stay `Some(k)`/`true`.
- **M6 — Translator totality:** non-`XPub` variants (`Single`,`MultiXPub`) → typed internal error from `pk()`, no panic (unreachable but must be total).
- **M7 — `CheckedDescriptor::new` (restore.rs:696) re-validates only on the `--format` path**, not the bare print (harmless; string is Display of a parsed Descriptor) — SPEC overstates "the caller re-validates".

## Scope
MINOR correct (v0.53.9 → v0.54.0). NO `schema_mirror`; GUI paired-PR for `--json` wire-shape (must enumerate all changed fields per I4); manual restore chapter; FOLLOWUPS as listed. **PART 1 shippable without PART 2** (pk-keyed toolkit cards fail loud either way; md-cli-authored bare-key cards already flow faithfully — a bonus). Coherent as one cycle once I1-I4 folded. **Gate status: implementation MUST NOT begin** — fold I1-I4 (+ Minors), persist, re-dispatch ROUND 2.
