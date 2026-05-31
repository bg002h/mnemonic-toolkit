# R0 Review — IMPLEMENTATION_PLAN_toolkit_mk1_origin_path.md

Opus architect, mandatory pre-impl R0 gate. Branch `toolkit-mk-codec-0.4.0-repin`, base
`master` `a255060` + applied re-pin. Verified plan code vs live toolkit + mk-codec 0.4.0
registry source. Persisted by controller.

## Headline confirmations
- mk-codec 0.4.0 APIs verified: `KeyCard::new(Vec<[u8;4]>, Option<Fingerprint>, DerivationPath, Xpub)` (`key_card.rs:80-92`); `encode_with_chunk_set_id` (`lib.rs:51`); guard `encode.rs:38-48`; `Error::XpubOriginPathMismatch { xpub_depth:u8, path_depth:u8, xpub_child:ChildNumber, path_child:Option<ChildNumber> }` (`error.rs:176-185`). Helper + friendly arm field-correct.
- `KeyCard.origin_path: DerivationPath` IS on the decoded card (`key_card.rs:42`) — available to the cross-check. `PathComponent { hardened: bool, value: u32 }` is `Copy` (md-codec origin_path.rs:18-24).
- 8 `KeyCard::new` sites confirmed (path args at `:228,:245,:393,:563,:780,:796`). Reject loop `:494-506`; `paths` reused at `:509/:563` → no dead binding on removal; test-only path; safe.
- Phase-1 edits type/bounds-safe: `components[d-1]` gated `d∈[1,md_depth]`; `full[..d-1]` gated `d≥2 && d-1≤full.len()`.

## CRITICAL
**C1 — Check A (`d > md_depth → "internally inconsistent"`) spuriously fires on a correctly-emitted genuine 4→3 bundle; Phase 1 contradicts Phase 0's 4→3 extend.** For a genuine depth-4 leaf xpub with depth-3 md1 origin (the 20-test 4→3 bucket), the helper EXTENDS the mk1 path to depth-4 (SPEC §3.2 "4→3 extend"), so the correct card has `d=4 > md_depth=3` → Check A warns on a CORRECT bundle. The cards are actually consistent (md1's TLV pubkey == the depth-4 xpub; md1 merely under-annotates the origin). The SPEC §3.5 analysis verified only 3→4/4→4/tampering, omitting 4→3; §3.1's prefix invariant is violated by 4→3 (xpub deeper than md1 origin); Phase 3 dropped the recon's I1 4→3 audit. **Fix:** replace Check A + Check B with an **overlap-prefix comparison** of the decoded `card.origin_path` against `md_path` (compare on `min(len)` components; warn only on genuine overlap-disagreement). This passes 3→4 (mk1 ⊆ md1) AND 4→3 (md1 ⊆ mk1) by construction and still fires on tampering. Add a 4→3 no-false-positive test.

## IMPORTANT
- **I1 — Task 4.2 `.mk1.Single[*]` does not exist.** `MkField` is `#[serde(untagged)]` (`format.rs:64-69`) → `bundle --json` emits `"mk1": ["mk1qp…", …]` as a flat array (single-sig) (`json_envelope.rs:477`). Read `json["mk1"]` as a string array directly.
- **I2 — Task 3.1 misattributes the failing call.** The panic is in the ms1-harvest `synthesize_full(&entropy_a, fp_a, xpub_a, CliTemplate::Bip84, …).unwrap()` (`:2688`/`:2764`) — depth-4 `xpub_a` + Bip84 depth-3 template trips the guard. Fix = change the SHARED `let path = …` at `:2653`/`:2731` to `m/84'/0'/0'` so BOTH the harvest AND the `synthesize_multisig_watch_only` cards stay consistent (not just one site).

## MINOR
- **M1** FOLLOWUP `mk1-wif-bundle-depth0-invalid-card` does not exist in toolkit FOLLOWUPS.md (only `mk1-depth-child-compensating-check-watch` `:3335`). Task 4.3: FILE the new subsuming slug `mk1-card-origin-path-vs-xpub-depth-consistency` (resolved) + flip the one existing entry; drop the dangling reference.
- **M2** `MAINNET_WIF` is not a shared const (locally redeclared in `cli_argv_leakage.rs:46`). Task 4.2 must declare its own.
- **M3** friendly.rs test mod (`:283`) has only `use super::*`; Task 4.3 Step 3 needs `use bitcoin::bip32::ChildNumber;`.
- **M4** Task 0.2 cites `KeyCard::new` opening lines; path args are a few lines below (`:228` etc.). Cosmetic.
- **M5** Dropping the old depth-0 `child==0` check is acceptable (unreachable post-fix; no test asserts it).

## VERDICT: RED (1C / 2I / 5M)
Phase 0 sound; plan code compiles + is bounds-safe. Blocker C1: Check A contradicts the 4→3 extend → false "internally inconsistent" on correct bundles; replace Check A/B with the overlap-prefix `card.origin_path`-vs-`md_path` comparison + add a 4→3 no-false-positive test. Fold I1/I2 + the Minors. Re-dispatch after fold.
