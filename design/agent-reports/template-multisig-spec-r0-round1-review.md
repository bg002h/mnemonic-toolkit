# SPEC R0 round 1 — bundle-md1-template-multisig (#28 phase 2) (opus architect, verbatim)

> Reviewer: opus architect (read+bash; toolkit HEAD `e97e8470` == base SHA, src `cbdadbb7`; md-codec 0.37.0 registry copy; rust-miniscript 13.1.0/`95fdd1c`; mk-cli v0.10.0). **Verdict: RED — 0 Critical, 2 Important.** Both Importants are §4.2a origin-assembly respecifications (mis-identified reuse target + unflagged net-new mk1 origin extraction); no silent-wrong-wallet (origin-exact gate refuses loud); no closed brainstorm finding regressed. 3 Minors. Fold → re-dispatch.

---

**Verdict: RED — 0 Critical, 2 Important.**

## Verified correct
- **Emit guards A/B/C** exist as cited (`synthesize.rs:987-994`/`:1005-1012`/`:1013-1021`); single-slot keyless mutation `Shared(OriginPath{empty})` `:1031` (comment `:1027-1030` "canonical_origin re-supplies on decode"); card back-half `:1047-1078`. `cli_template_from_tree` Some only for pkh/wpkh/tr-keypath (`:228-237`).
- **§3.2 C1 decode** correct: `validate_explicit_origin_required` no-op when `canonical_origin.is_some()` (`validate.rs:222-224`), else `MissingExplicitOrigin` on empty (`:243`); canonical Some for `wsh(multi/sortedmulti)`/`sh(wsh)` (`canonical_origin.rs:57-71`), `_=>None` general (`:76-78`). `synthesize_unified` builds `Shared`/`Divergent` (`:899-908`).
- **§4.2a C1 invariant mechanism sound:** `compute_wallet_policy_id` never consults `canonical_origin` at hash time (`identity.rs:160-171`); reads `OriginPathOverrides` else `path_decl`; `expand_per_at_n` override-precedence (`canonicalize.rs:437-444`). Template-id origin-invariant (`identity.rs:71-104`).
- **§6.2 I_new ladder exact** (reproduced: list→8B/2.2e-12, K=32→11B/2.2e-10, fixed-8B@K=32→1-in-275). Address-search collision-free (non-sorted stored slot order, rust-miniscript `astelem.rs:155-188`).
- **§6.1 sortedmulti** (id never sorts `identity.rs:172-229`; address sorts `to_miniscript.rs:402-409`). **Refusal floors** (`tr(sortedmulti_a)`→`to_miniscript.rs:584-589`; `MissingPubkey`→`:122`). **Restore premises** (`:1655-1661`/`:1751-1784`/`:1987`/`:2030-2042`). **Single-sig non-regression** (`:867-911`). **Binding stub** (`bundle.rs:1151-1159`). **verify-bundle gap** (`:478`/`:2475-2479`). **NO-BUMP** (tree write_node exhaustive; MultiKeys key-independent). **No rayon**.

## CRITICAL
None.

## IMPORTANT
### I-A — §4.2a mis-identifies the reuse target; the cited `verify_bundle.rs:~1080-1150` rebuild produces a single BIP-48 default for EVERY slot and reads NO cosigner mk1 origins → degrade2 (BIP-84-per-account) never matches → feature dead-on-arrival under a literal impl.
- `verify_bundle.rs:1082-1083` builds the baseline from `compute_default_origin_path(network, account)` = hardcoded `m/48'/coin'/account'/2'` (BIP-48; `bundle.rs:2144-2168`); assigns ALL slots that one path (`(0..n).map(|_| default.clone())` `:1088`); overlays only `--slot @N.path=` (`:1144-1146`). Binding loop reads `anno_path` from the rebuilt `path_decl`, NOT cosigner cards (`:1175-1182`).
- degrade2 origins are BIP-84 `m/84'/0'/N'` — not BIP-48 → every candidate gets wrong origins → wrong child keys → wrong spk/id → search REFUSES every permutation.
- **Fix:** §4.2a must specify per-slot origins BUILT from the permuted (key+origin) supplied-key assignment — own from `--account`/`--origin` (honoring actual purpose, NOT forced BIP-48); cosigner from mk1 `origin_path` — explicitly NOT `compute_default_origin_path`. Cite the rebuild only as the overwrite-before-gate DISCIPLINE. Add a degrade2 (BIP-84) completion test to §7.

### I-B — the cosigner-mk1 origin EXTRACTION §4.2a depends on does not exist today; SPEC asserts it as shipped.
- Current `--cosigner @N=mk1` parse (`restore.rs:1955-1990`) decodes to `supplied65` for the cross-check ONLY — never reads `origin_path`/`origin_fingerprint`. The unassigned-`--cosigner` search mode doesn't exist at all.
- **Fix:** §4.2a/§9 enumerate NET-NEW mk1 intake: decode `--cosigner` mk1 → `KeyCard` → `(key65, origin_fingerprint, origin_path)`; these `(key+origin)` pairs are the search's permutable units; flag as net-new (`verify_bundle.rs:1907-1944` mk1 read is an advisory cross-check on the keyed path, not an origin source).

## MINOR
1. **§2: `--expect-wallet-id` is NOT "new on verify-bundle"** — exists (`VerifyBundleArgs.expect_wallet_id` `:85`/`:62-63`) single-sig-only (`:643-680`). Only MULTISIG wiring is new; flag-name already in schema mirror. Reword.
2. **§4.2a/§7-floor-6 `OriginPathOverrides` vs `path_decl` write target** — the cited rebuild writes `path_decl.paths` directly (`:1151`); brainstorm-r4 said `OriginPathOverrides`. Both reach the same id; pick one.
3. **§6 `astelem.rs:155-188` is rust-miniscript (toolkit dep), not md-codec** — label the crate. The md-codec WIRE-layer order property is `tree.rs:115-139` (orthogonal, also true).

## To turn GREEN
1. I-A — respecify §4.2a per-slot origin assembly from the permuted supplied-key (key+origin) assignment (NOT compute_default_origin_path); add a degrade2 BIP-84 completion test.
2. I-B — enumerate the net-new mk1 cosigner origin intake + unassigned-`--cosigner` parse; make (key+origin) the search unit.
3. M1 — reword §2 `--expect-wallet-id`.
4. M2 — reconcile the OriginPathOverrides-vs-path_decl write target.
5. M3 — label astelem.rs as rust-miniscript.
Architecture sound; bounded respecifications, not a redesign.
