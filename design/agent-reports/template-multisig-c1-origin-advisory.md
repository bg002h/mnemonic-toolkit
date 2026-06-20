# C1 origin-handling advisory — bundle-md1-template-multisig (#28 phase 2) (opus architect, verbatim)

> Advisory (NOT an R0 gate). Reviewer: opus architect (read+bash; md-codec 0.37.0 / descriptor-mnemonic working tree). User favored option (1) "carry source origins"; architect endorses a CONDITIONAL form + fixes the brainstorm's broken "elide ALL to Shared(empty)" text. UNCOMMITTED.

---

**Bottom line:** carry the source's per-`@N` origins **only when `canonical_origin == None`**; keep empty-elide when `canonical_origin == Some`. Supplied-key origins always win at completion (carried = decode+display only). The brainstorm's current "elide ALL origins to `Shared(empty)`" is WRONG for general policies (makes degrade2 undecodable) and must be fixed.

## Verified at source
- `validate_explicit_origin_required` (`validate.rs:221-246`, called `decode.rs:75`) → `Err(MissingExplicitOrigin{idx})` when `canonical_origin(&tree).is_none()` AND that idx's origin is empty. Unconditional. degrade2 (`wsh(or_i(...))`) is `canonical_origin==None` (`canonical_origin.rs:76-77` `_ => None`) → empty-origin keyless template **cannot decode**. Canonical multisig `wsh(sortedmulti)`→`m/48'/0'/0'/2'` (`canonical_origin.rs:57-62`) elides fine.
- WDT id origin- AND fp/xpub-invariant (`identity.rs:71-104` hashes `use_site_path ‖ tree ‖ use_site_path_overrides` only; `wdt_id_invariant_to_origin_path_change`). Any origin choice → identical binding.
- Keyless mutation nulls fp + pubkeys (`synthesize.rs:1025-1026`). Origin (`path_decl`) is the only user-varying wire byte left → account number is the only thing varying md1 across users.
- D8 completion real + shipped for single-sig: origins from `--account`/`--origin` + `--from` (`restore.rs:792-812`, `verify_bundle.rs:528-554`); non-canonical multisig path rebuilds per-`@N` origins from `compute_default_origin_path(network,account)` + `--slot` overrides (`verify_bundle.rs:1074-1146`) — NEVER from a carried template origin. mk1 carries `origin_fingerprint`+`origin_path`, cross-checked supplied-vs-reconstructed (`verify_bundle.rs:1907-1944`).
- Funds-safety gate (`compute_wallet_policy_id`) origin- + key-sensitive (`identity.rs:106-153`), resolves origins from the COMPLETED descriptor's `OriginPathOverrides`/`path_decl` (`:166-171`) = supplied keys. Wrong/stale origin → different id → no match → refuse.

## Recommendation: scoped option (1)
```
per @N:  carried_origin = if canonical_origin(tree).is_some() { empty }
                          else { source_origin[N] }   // decode + display only
```
Reject (2) fixed-placeholder (byte-identity already delivered by origin-invariant WDT-id; adds synthetic-path surface for zero binding gain) and (3) account-normalize (synthetic + erases the divergence the card could honestly show). (1) reuses bytes that already exist + is the most honest display. Caveat: carried origins are the CREATOR's; for a different completer they're cosmetically stale (never funds-load-bearing) — a one-line emit/card note suffices.

## Crux (a) precedence — RESOLVED: supplied-key origins win, unconditionally
Decode stage: carried origin only satisfies `validate_explicit_origin_required` + renders the card. Completion stage: origins ENTIRELY from supplied keys + `--account`/`--origin` (D8, as the shipped non-canonical multisig path already does, `verify_bundle.rs:1074-1146`); the carried origin MUST be ignored for derivation/id/address. Not a new rule — existing behavior. Disagreement carried-vs-supplied → **D9 SOFT ADVISORY** (the brainstorm already has this, line 37/70), NOT a gate.

## Crux (b) funds-safety — CONFIRMED safe
A wrong/stale carried origin cannot produce a wrong wallet: the id/address match is computed on the completed supplied-key descriptor (`identity.rs:166-171`; restore/verify rebuild `path_decl` from keys first). Carried filler is overwritten before it can influence derivation. Worst case cosmetic. Same posture as D9/floor-6.

## Crux (c) degrade2 decode/template-id — option (1) clean; current brainstorm text BROKEN
Option (1): each `@N` carries its real origin (accounts 0–3 → `Divergent` path_decl); `validate_explicit_origin_required` passes; decodes; template-id stable (origin-invariant). Brainstorm's D7/emit-delta as written (elide ALL → `Shared(empty)`): degrade2 → `MissingExplicitOrigin{idx:0}`, undecodable. **Hard contradiction; fix before R0 GREEN.**

## Crux (d) cleaner framing
> Template carries origins iff wrapper non-canonical (`canonical_origin==None`); else elide to empty. Either way carried/elided origin = decode + display only; at completion origins always re-derived from supplied keys (D8). Single-user creator=completer sees carried origins rendered directly; shareable/different-completer supplies their own (keys + `--account`/`--origin`), carried superseded (D9 soft advisory on disagreement).
Unifies the model: "origin on the card is informational; origin for funds is from the keys" — single-sig + multisig, canonical + general alike. Keeps the elide (zero variance) for the common canonical case; pays carried-origin bytes only where decode forces it.

## Funds-safety risks NOT accepted
1. Shipping "elide ALL origins to `Shared(empty)`" unmodified — general-policy templates undecodable (crux c). Must become conditional on `canonical_origin`. **Load-bearing, independent of option 1/2/3.**
2. Promoting carried-vs-supplied origin comparison to a reject — real wallets are non-canonical (degrade2 = BIP-84 in a multisig); keep D9 soft advisory.
3. Any framing that lets the carried origin reach `compute_wallet_policy_id`/derivation — must be provably overwritten by supplied-key origins first; route general-policy completion through the existing `verify_bundle.rs:1074-1146` rebuild, NOT trust as-decoded `path_decl`.

## Plan-doc flag
Emit-delta (brainstorm line 77) lifts guard C `canonical_origin(&tree).is_none()` (`synthesize.rs:1013-1021`, verified). The correct lift: admit non-canonical multisig shapes, and for `canonical_origin==None` write the source's real per-`@N` origins (`Divergent` for degrade2 0–3) into `path_decl`, NOT `Shared(empty)`. Keep `Shared(empty)` only for `canonical_origin==Some`.
