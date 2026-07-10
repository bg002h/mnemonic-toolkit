# SPEC R0 review — F3 network fail-open (Cycle H) — round 1

**Reviewer:** Fable (SPEC R0, read-only), per user directive. SPEC @ `713484c3`.
**Dispatched:** 2026-07-09 (Cycle H, SPEC R0 round 1). Persisted verbatim per CLAUDE.md.

# R0 Verdict: **NOT GREEN — 0 Critical / 2 Important / 3 Minor**

All claims verified live at `713484c3` (confirmed = HEAD, version 0.82.0 → v0.83.0 MINOR correct). The three specified edges are real, open, and the fixes are correct as written — but the completeness sweep missed a live, same-class, worse-behaved edge that I reproduced against the actual binary.

## Verified correct (no findings)

- **E1** `convert.rs:1524-1526` — exact cited pattern, unguarded. Fix type-checks: `args.network` is `Option<CliNetwork>`, `xpub.network` is `NetworkKind` (same comparand the shipped guards at `convert.rs:1144-1148` and `addresses.rs:186` use), `CliNetwork::network_kind()` exists (`network.rs:30`), `?` is legal in that match arm (enclosing `compute_outputs` returns `Result<_, ToolkitError>`).
- **E2** `address_of_xpub.rs:215-217` — byte-identical pattern; local binding is `xpub` (bound at `:191`). Bonus: `resolve_target_xpub` accepts mk1 cards, so the guard covers mk1-decoded xpubs automatically.
- **E3** `silent_payment.rs:125-134` — the xprv branch returns without checking `xpriv.network`; `args.network` is non-Optional `CliNetwork`; the mismatch is real (`coin_type()` at `:250`, `sp_hrp` at `:255`). `resolve_master_xpriv` has exactly **one** production caller (`:249`), so the guard can't ripple. The seed branches build via `Xpriv::new_master(network.network_kind(), …)` (`:120`, `:175`) — constructed **at** the asserted network, a mismatch is impossible; the SPEC is right to leave them unguarded (a guard there would be dead code, and E3c correctly pins non-regression).
- **Precondition** (`network.rs:89-93`) respected: E1/E2 guard only the `Some` arm; E3's network is always asserted. Inference/None arm needs no guard — `network_from_xpub` derives the right network from the key itself.
- **Granularity**: version bytes are 2-way (Main `0488b21e` / Test `043587cf`), exactly `NetworkKind`'s partition. `--network signet` + tpub → accepted → renders at test coin-type/`tb1` HRP — correct and information-theoretically forced. No Main/Test under-rejection exists.
- **Error**: `NetworkMismatch` exists (`error.rs:286`), `exit_code` = **2** (`:621`), `kind` (`:693`), `Display` (`:876`). No new variant, no ordering churn.
- **Already-guarded list**: `convert.rs:1144`, `:1554`, `pipeline.rs:156` (`assert_slots_network_agrees`), `addresses.rs:183-194` all confirmed guarded. `export_wallet.rs:793` is guarded **but see Important-1** — it guards only one of three intake arms.
- **SemVer/lockstep**: no clap change → no `schema_mirror` ✓; GUI deferral sound (post-pin-bump the GUI default-mainnet case surfaces a refusal instead of a wrong address — louder, not wrong) ✓; no manual transcript exercises a now-refused flow (0 hits for tpub/tprv + `--network mainnet` across `docs/manual/` and `.examples-build/`) ✓.

## IMPORTANT-1 — Missed same-class edge: `export-wallet --slot`/`--descriptor` arms mint rewritten wrong-network version bytes, firing on the **default** `--network` (live-reproduced)

`export-wallet`'s `--network` is `#[arg(long, default_value = "mainnet")]` (`export_wallet.rs:226-227`). The `--template --slot @N.xpub=` arm goes `bundle::resolve_slots` (xpub arm at `bundle.rs:589-656` — **no network check**) → `build_descriptor_string` (`wallet_export/pipeline.rs:18` — **no check**) → emitters that **rewrite version bytes from `--network`**: `electrum.rs:110/:162` → `render_slip132_xpub` → `apply_xpub_prefix(xpub, Zpub, network)` (`slip0132.rs:108-112` splices `swap_target_for(variant, network)` over the key's own bytes); same at `coldcard.rs:187-196`. Unlike `bundle` (required `--network` + the `synthesize_unified` §4.3 check at `synthesize.rs:1019-1031`, which refuses), export never routes through a check.

I reproduced it live at `713484c3`:

```
$ mnemonic export-wallet --template bip84 --slot @0.xpub=tpubD6Nz… --format electrum
  → "xpub": "zpub6jft…", "derivation": "m/84'/0'/0'"   (exit 0, no warning)
```

A testnet `tpub`, **no `--network` flag at all**, emitted as a mainnet `zpub` at mainnet coin-type — exactly the minting hazard the in-code M13 comment (`export_wallet.rs:786-791`) describes, on a sibling arm of the same command the SPEC's §2 lists as "ALREADY-GUARDED (leave): … export_wallet.rs:793". That citation is false-safe: only the `--from-import-json` arm is guarded. This is arguably worse than E1-E3 (version-byte rewrite + fires on the default, not just an explicit flag).

**Fold (either resolves):** (a) add **E4**: after `resolve_slots` at `export_wallet.rs:~545`, assert each resolved slot's `xpub.network` against `args.network.network_kind()` (mirror `assert_slots_network_agrees`); note the design wrinkle that export's network is a clap **default** rather than always-explicit — but an export file must commit to one network and `bundle` is the fail-closed precedent, so guarding the effective value is right. Adjudicate the `--descriptor` concrete-keys arm at fold time (its slots list is empty so it can't reach the SLIP-0132 rewrite; exposure is the network-labeled envelope only). Or (b) explicitly de-scope: file a FOLLOWUP (`export-wallet-slot-network-fail-open`), correct the §2 false-safe wording, and record the eval-F3-scope rationale. Given the cycle's charter is "close the fail-open class" and E4 is ~6 lines + 2 test cells, (a) is the better call.

## IMPORTANT-2 — §2's sweep mechanism can't find the shape it's guarding against, and flag-storms on 8 benign hits

The grep `'network.*unwrap_or_else.*network_from_xpub\|\.network\.unwrap_or'` catches only the `Option`-default idiom. It would find **neither E3** (non-Option `args.network`) **nor the Important-1 export gap** — the two hardest cases. Meanwhile it DOES hit 8 sites absent from the leave-list, forcing the implementer into the SPEC's mandated flag-and-stall on each: `addresses.rs:199/:237`, `convert.rs:1257`, `restore.rs:451/:812/:1376/:3056`, `derive_child.rs:214`.

**Fold:** (a) restate the sweep as an audit of **every asserted network (explicit flag OR clap default) used against version-byte-bearing input** (Xpub/Xpriv/WIF/mk1/SLIP-0132) without `assert_network_agrees`, per command intake arm; (b) pre-adjudicate the benign hits so no mid-cycle stall: `addresses.rs:199/:237` (seed/Electrum-phrase arms, network-agnostic), `convert.rs:1257` (feeds seed arms + the WIF arm already guarded at `:1554`), `restore.rs` ×4 (seed inputs; the keyed-md1 arm stores **raw 65-byte keys** — xpubs are minted at `--network` via `xpub_from_65_bytes`, no embedded bytes to contradict; bare-xpub cosigners explicitly adjudicated in-code at `restore.rs:2152` — `let _ = network; // network kind is informational`), `derive_child.rs:214` (BIP-85: emission-only network; master's network affects no derivation byte per the in-code comment at `:191-194` — out of F3's class, guarding it would over-reject).

## Minor

1. **§4 test gaps**: add an E2 cell with an **mk1 card** target (pins that the guard covers mk1-decoded xpubs), and a `--network signet` + tpub **ACCEPT** cell (pins the 2-way `NetworkKind` semantics against future over-tightening).
2. **§5/G5**: add the standing `.examples-build/` corpus-lockstep reminder — `examples.yml` re-runs `gen.sh` on any `crates/` change and the corpus self-pin FATALs on version drift (Cycle A gotcha), so the v0.83.0 release step must regen the corpus even though no transcript hits the refused path.
3. **Observation, candidate FOLLOWUP (not scope)**: the xpub-search *search*-family (`path-of-xpub`, `passphrase-of-xpub`, `passphrase-search`) with a tpub target + `--network mainnet` derives mainnet candidates against a testnet target → exhausts and reports NOT-FOUND instead of diagnosing the network contradiction. Wrong-diagnosis papercut, no wrong-address mint — file, don't fold.

**Fold Important-1 and Important-2 (plus Minors at author's discretion), then re-dispatch for convergence per the reviewer-loop rule.**
