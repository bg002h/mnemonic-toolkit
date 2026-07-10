# SPEC R0 review — F3 network fail-open (Cycle H) — round 2 (convergence)

**Reviewer:** Fable (SPEC R0 convergence, read-only), per user directive. SPEC @ `713484c3`.
**Dispatched:** 2026-07-09 (Cycle H, SPEC R0 round 2). Persisted verbatim per CLAUDE.md.

## VERDICT: NOT GREEN — 0 Critical / 1 Important / 2 Minor

The round-1 folds are correctly executed (Important-1 and Important-2 both resolved as folded; all three Minors adequately captured; E1/E2/E3 and all guard-rails re-confirmed — details below). However, the deep-check this round mandated on E4's `--descriptor`-arm adjudication ((c)/(d)) surfaced a **live-reproduced 5th mint edge** the E4 fold's narrowly-worded confirmation mandate would not catch.

## Important-A (NEW): BSMS 4-line on the `--descriptor` arm is a live wrong-network address mint — the E4 slots-guard is inert there, and the SPEC's audit mandate as worded will miss it

**Live repro at `713484c3` (release binary v0.82.0):**
```
$ mnemonic export-wallet --descriptor "wpkh([00000000/84h/1h/0h]tpubDC8msFGeGuwnKG…/0/*)" --format bsms
BSMS 1.0
wpkh([00000000/84'/1'/0']tpubDC8msFG…/0/*)#94qtvq0a
/0/*
bc1q6rz28mcfaxtmd6v789l9rrlrusdprr9p276ldv     ← MAINNET address from the testnet key
exit=0
```
No `--network` (default mainnet, `export_wallet.rs:226-227`), no `--bsms-form` (default `4-line`, `:319`). Line 2 carries the tpub descriptor verbatim; line 4 is a mainnet-HRP first address derived from it — an internally-contradictory BSMS file and exactly F3's mint class, on the DEFAULT network like E4.

**Chain:** `wallet_export/bsms.rs:113` `derive_first_address(&parsed, network_to_bitcoin(inputs.network))` → `derive_address.rs:62-64` `definite.address(network)` — miniscript encodes at the passed network with no version-byte cross-check. `BsmsEmitter::collect_missing` returns empty (`bsms.rs:58-62`); `emit()` refuses only taproot; `emit_payload`'s unsorted-multi gate (`export_wallet.rs:128+`) covers electrum/coldcard/jade only. Reachable for any non-taproot concrete descriptor.

**Why the fold doesn't cover it:** the E4 bullet's impl mandate is scoped to "`apply_xpub_prefix` / version-byte-rewrite". That grep passes — `apply_xpub_prefix` is reached only from `electrum.rs:110/:162` and `coldcard.rs:187-196`, both slot-sourced — so a faithful implementer adjudicates the `--descriptor` arm "confirmed inert" and ships with this mint live. The SPEC's sentence "placing the guard over `resolved_slots_ref` … is inert for `--descriptor` (no over-rejection)" is true for the slot-rewrite but false as a no-hazard claim for the arm. §2's "E1-E4 are the known set" is now factually wrong at review time; per the SPEC's own rule, the scope decision belongs in the SPEC now.

**Fold required — add E5 (scope-decided, not left to the impl audit):**
- Guard at the mint site: in `BsmsEmitter::emit`'s `FourLine` arm (or equivalently in export_wallet's `--descriptor` arm gated on `format == Bsms && bsms_form == FourLine`), walk the parsed descriptor's keys (`for_each_key`) and for each `DescriptorPublicKey::XPub`-bearing key call `assert_network_agrees(xpub.network, inputs.network.network_kind(), "export: bsms first-address")`. Raw hex pubkeys carry no version bytes — skip (precondition-respecting; no over-rejection).
- Do NOT guard the whole `--descriptor` arm unconditionally: `--format descriptor`/`bitcoin-core`/`bip388` re-emit the descriptor verbatim-truthfully (no `inputs.network` consumption — verified: `descriptor.rs`, `bitcoin_core.rs` have zero `inputs.network` uses; `bip388.rs:46-52` descriptor branch is verbatim), so refusing a tpub descriptor passthrough at default mainnet would over-reject previously-legitimate canonicalization workflows. The 2-line BSMS form has no line-4 → no mint → adjudicate accepted (or scope-decide otherwise, but say so in the SPEC).
- Template-arm BSMS needs nothing: post-E4-guard the canonical descriptor is synthesized from network-agreeing slots.
- Test cells: E5a `--descriptor <tpub-desc> --format bsms` (defaults) → `NetworkMismatch` exit 2, no address; E5b same + `--network testnet` → exit 0, `tb1…` line 4; E5c `--bsms-form 2-line` + tpub descriptor → per adjudication; E5d hex-key-only descriptor + bsms 4-line → exit 0 (skip-arm pin).

## Minor-B: restore/verify-bundle bare-xpub `--cosigner` adjudication should acknowledge the bundle asymmetry

`restore.rs:2152` (`let _ = network; // network kind is informational`) is verified as cited and the §2 benign adjudication is *defensible* (the md1 card is network-less raw-65-byte authority; the search binds key bytes; worst case is contradiction-not-diagnosed, not a key-network override). But it is asymmetric with bundle's prior art `synthesize.rs:1019-1031`, which refuses the *identical shape* (`s.xpub.network != network.network_kind()` → `CosignerSpec`) — and restore's `--network` is DEFAULTED (`restore.rs:1376/:3056` `unwrap_or(Mainnet)`), then the winning key bytes are re-minted at `--network` (`xpub_from_65_bytes`, `:2830/:3194`). Fold: one sentence in §2 acknowledging the bundle-vs-restore asymmetry, plus add it to the §3 diagnosis-class FOLLOWUP (or a sibling slug, e.g. `restore-cosigner-bare-xpub-network-contradiction-not-diagnosed`). Do not widen the cycle.

## Minor-C: `master_xpub_at_0` coherence note (optional)

`coldcard.rs:220` emits `inputs.master_xpub_at_0` verbatim via `to_string()` — truthful bytes, no re-encode, out of F3's mint class — but a testnet `@0.master_xpub=` + mainnet slot yields an internally-inconsistent coldcard JSON (`chain: "BTC"` + top-level `tpub…`). Either extend the E4 loop one line to also check `slot.master_xpub` when present, or record the adjudicated-truthful call in E4's bullet. Non-blocking.

## Round-1 findings — all verified RESOLVED

- **Important-1 (E4):** (a) fix location correct — `resolved_slots_ref` bound at `export_wallet.rs:625-632` (`&[]` on the descriptor arm; slot-populated on template arm via `resolve_slots` at `:542`), `EmitInputs` at `:674` with `resolved_slots` at `:676`; the `~:672` placement is right. (b) compiles as written — `slot.xpub.network` (`synthesize.rs:896-897`, `bitcoin::bip32::Xpub`) and `args.network.network_kind()` are the byte-level pattern of the existing `:793-796` guard. (c) slot-rewrite claim TRUE (only electrum/coldcard reach `apply_xpub_prefix`, both slot-sourced; coldcard's `/0/0` address render `:172-178` also slot-sourced and covered) — but see Important-A for the non-rewrite mint the mandate's wording misses. (d) sparrow/electrum/coldcard refuse without template+slots; green/jade/specter/bitcoin-core/descriptor never consume `inputs.network`; bip388 descriptor branch verbatim — BSMS is the only leak. **E4 itself live-reproduced** (mainnet `zpub6qRfxLnns15…` + `m/84'/0'/0'` from the tpub, exit 0).
- **No E4 over-rejection:** export slots are xpub-only (`validate_watch_only`), every `Xpub` carries a truthful `NetworkKind`, and `normalize_xpub_prefix` is network-preserving (`slip0132.rs:80-92`: vpub/upub→tpub, zpub/ypub→xpub) — no originless or re-minted slot exists on this path.
- **Important-2 (§2 adjudications):** all spot-checks pass — `addresses.rs:199/:237` seed/electrum-phrase arms minted AT `--network`; `convert.rs:1257` feeds seed arms (WIF guarded at `:1554`); `restore.rs:451/:812/:1376/:3056` seed/template-completion bindings, keyed-md1 mints from card raw 65-byte keys (network-less by design); `derive_child.rs:191-194` BIP-85 master pinned to internal `NetworkKind::Main`, `:214` emission-only. `restore.rs:2152` = Minor-B above.
- **Minors 1-3:** §4 carries the mk1-target cell (well-founded — mk1 wire encodes NetworkKind, `vendor/mk-codec/src/bytecode/xpub_compact.rs:56-66`, so no tautology/false-reject) + the signet-accept cell + E4a-d; §5 carries the corpus-lockstep reminder; §3 files the xpub-search FOLLOWUP.
- **No regressions:** E1 (`convert.rs:1524-1526`), E2 (`address_of_xpub.rs:215-217`), E3 (`silent_payment.rs:53` non-Option network, `:125` xprv branch, `:250/:255` coin-type/hrp) all as cited; precondition doc `network.rs:80-93` respected by all proposed guards; `NetworkMismatch` exit 2 (`error.rs:621`); MINOR v0.83.0 / no schema_mirror / GUI deferral all sound; §5 docs claim re-verified (prose-only tpub mentions, no transcript regen needed).

## What still needs folding

Exactly one blocking item: fold **E5 (BSMS 4-line descriptor-arm guard)** per Important-A — edge description, live-repro evidence, mint-site guard placement, the descriptor-passthrough non-guard adjudication, and E5a-d test cells — plus correct §2's "E1-E4 are the known set" and the E4 bullet's "inert for `--descriptor`" wording. Minors B/C at author's discretion. Then re-dispatch round 3.
