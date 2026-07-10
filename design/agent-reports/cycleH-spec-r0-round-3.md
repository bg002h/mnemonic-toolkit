# SPEC R0 review — F3 network fail-open (Cycle H) — round 3 (convergence)

**Reviewer:** Fable (SPEC R0 convergence round 3, read-only). SPEC + repo @ `713484c3`.
**Dispatched:** 2026-07-09 (Cycle H, SPEC R0 round 3). Persisted verbatim per CLAUDE.md.

## VERDICT: NOT GREEN — 0 Critical / 1 Important / 1 Minor

The round-2 folds are faithfully executed and the export-emitter audit is now **complete at the edge level — there is no 6th mint edge** (independent re-audit below). But the E5 guard **as written in the SPEC does not close E5**: it matches only `DescriptorPublicKey::XPub` and skips `MultiXPub`, so the BIP-129-**canonical** multipath descriptor shape still mints. Live-reproduced. One fold, then this converges.

## Important-D (NEW, same edge — E5 guard variant coverage): the spec'd `XPub`-only match leaves the multipath `<0;1>/*` mint live

**Live repro at `713484c3` (release binary v0.82.0):**
```
$ mnemonic export-wallet --descriptor "wpkh([00000000/84h/1h/0h]tpubDC8msFG…/<0;1>/*)" --format bsms
BSMS 1.0
wpkh([00000000/84'/1'/0']tpubDC8msFG…/<0;1>/*)#j2vvh9nv
/0/*,/1/*
bc1q6rz28mcfaxtmd6v789l9rrlrusdprr9p276ldv     ← MAINNET address from the testnet key
exit=0
```
- This vendored miniscript has **three** `DescriptorPublicKey` variants: `Single`, `XPub`, `MultiXPub` (`vendor/miniscript/src/descriptor/key.rs:23-30`). A `<0;1>/*` key parses as `MultiXPub` (BIP-389), which carries version bytes too (`key.rs:1047` `multi_xpub.xkey.network`). The SPEC's `if let DescriptorPublicKey::XPub(xk) = k` (SPEC §1 E5 snippet) leaves `mismatch = None` for it → no error → `derive_first_address` **explicitly splits multipath and derives** (`derive_address.rs:34-46`) → same mint.
- This is not a fringe shape: it is the **BIP-129-canonical** input — `path_restrictions_line`'s primary arm exists precisely for `<0;1>/*` (`bsms.rs:155-158`, emitting `/0/*,/1/*` as in the repro), and multipath xpubs are live `export-wallet --descriptor` test fixtures (`export_wallet.rs:1200/:1204`).
- The §4 cells would pass GREEN with the mint still live: E5a's vector is single-path `/0/*`.

**Fold (all in §1 E5 + §4):**
1. Replace the variant match with the per-key helper the vendored fork already ships: `DescriptorPublicKey::xkey_network(&self) -> Option<NetworkKind>` (`vendor/miniscript/src/descriptor/key.rs:1043-1049` — `Some` for XPub AND MultiXPub, `None` for Single, i.e. exactly the SPEC's skip semantics):
   ```rust
   parsed.for_each_key(|k| {
       if let Some(decoded) = k.xkey_network() {
           if decoded != inputs.network.network_kind() { mismatch = Some(decoded); return false; }
       }
       true
   });
   ```
   (Equivalently, the whole-descriptor `Descriptor::xkey_network() -> XKeyNetwork` at `vendor/miniscript/src/descriptor/mod.rs:1004-1028` — its own doc: "prevent accidentally using testnet keys on mainnet" — but then `XKeyNetwork::Mixed` needs an explicit refusal arm; the per-key form gets Mixed for free. Note in the SPEC which one is chosen.)
2. State the guard covers **both xkey variants**; keep the Single-skip wording.
3. Add cell **E5e**: multipath `<0;1>/*` tpub + `--format bsms` defaults → `NetworkMismatch` exit 2, no address (today: mints `bc1q6rz28…` exit 0 — the repro above); optionally the agreeing `--network testnet` multipath control (line 3 `/0/*,/1/*`, line 4 `tb1…`).

## Minor-E: stale edge counts after the E5 fold

Title says "the **three** unguarded edges", §0 opens "**Four** toolkit edges", §1 heading "The **four** edges" — the SPEC now defines five (E1-E5). Mechanical count fix; a future implementer greping "the four edges" for scope would under-count.

## Export-emitter audit — independently re-done, COMPLETE (no 6th edge)

All 12 `crates/mnemonic-toolkit/src/wallet_export/*.rs` re-audited for any derive/re-encode from `inputs.network` against version-byte-bearing keys, both arms:
- **sparrow.rs** — refuses without `--template` (`:104-108`) AND without slots (`:110-114`); descriptor arm unreachable. `:122` `sparrow_network` is a JSON label; `:250/:256` `origin_path_str` are path strings; xpubs emitted verbatim (`:136`). All slot-arm → covered by E4.
- **coldcard.rs** — both paths require `--template` (`:111-115`, `:261-263`); descriptor arm refused. The `:172-178` `/0/0` address render and `:187-196` `apply_xpub_prefix` are slot-sourced → E4-guarded before `EmitInputs`.
- **electrum.rs** — requires `--template` (`:52`); slot-sourced SLIP-0132 rewrites (`:110/:162`) → E4.
- **green.rs / jade.rs / specter.rs** — zero operational `network`/address/derive references (grep over all five verbatim-class files returned only doc comments); jade multisig delegates to coldcard's template-required text. No mint.
- **bitcoin_core.rs / descriptor.rs** — verbatim; bitcoin-core's multipath split re-emits `p.to_string()` with the key's own bytes, no `inputs.network` consumption (`:42-86`). (`import_array_single` `:92` serves `nostr --import readonly` with pre-built single-key descriptors — no xkey version bytes, out of class.)
- **bip388.rs** — descriptor branch verbatim (`:46-51`); template branch's only network use is the fallback origin-path *string* over E4-guarded slots (`:73`). No mint.
- **bsms.rs** — the E5 mint (`:113`), sole remaining leak; 2-line arm is `{line1}\n{line2}` only (`:97`) — **no derive, round-2 (d) confirmed**.

So the multipath finding is **variant coverage within E5, not a new edge** — the mint site (`bsms.rs:113`) and guard site are exactly where the SPEC puts them.

## Round-2 folds — verified resolved (modulo Important-D)

- **Important-A/E5 (a) compilability:** `for_each_key(|k| … true)` precedent in the same file (`bsms.rs:148`); `DescriptorPublicKey` already imported (`bsms.rs:40`); `xk.xkey.network: bitcoin::NetworkKind` (`key.rs:63-72,1046`); `inputs.network.network_kind()` precedent at `coldcard.rs:172`; capture-then-assert-after-walk is sound and covers all keys incl. multisig cosigners — **except the MultiXPub variant, Important-D**. (b) placement correct: both arms flow through the single `EmitInputs` (`export_wallet.rs:674`) → `emit_payload` (`:697`, dispatch `:97`) → `BsmsEmitter::emit` re-parses `canonical_descriptor` (`bsms.rs:104-110`); template arm post-E4 is a redundant no-op as claimed. (c) re-grepped: `descriptor.rs`/`bitcoin_core.rs` zero `inputs.network`; `bip388.rs` descriptor branch verbatim — non-guard adjudication correct. (d) 2-line: no derive, accepted-adjudication sound.
- **Minor-C:** `ResolvedSlot.master_xpub: Option<Xpub>` (`synthesize.rs:913`); by-value move out of the loop ref compiles (`Xpub: Copy`, precedent `export_wallet.rs:672`). Extended-loop line compiles as written.
- **Minor-B:** captured as §2 asymmetry note + §3 FOLLOWUP `restore-cosigner-bare-xpub-network-contradiction-not-diagnosed`; `restore.rs:2151-2152` and `synthesize.rs:1020-1031` verified; correctly NOT folded into scope.

## No regression from rounds 1-2

E1 (`convert.rs:1524-1526`), E2 (`address_of_xpub.rs:215-217`), E3 (`silent_payment.rs:125-135`) all as cited and still unguarded at this SHA; E4 sites verified; `assert_network_agrees` + PRECONDITION doc as cited; precondition respected by all five guards (E5's Single-skip included); `NetworkMismatch` exit **2** (`error.rs:621`); `--bsms-form` default `4-line` (`export_wallet.rs:319`); MINOR v0.83.0 / no schema_mirror / GUI deferral / §3 FOLLOWUPs / §4 cells / §5 corpus-lockstep reminder — all sound.

**Path to GREEN:** fold Important-D (three bullet edits, one test cell) + the Minor-E count fix, then round 4 should be a one-look convergence.
