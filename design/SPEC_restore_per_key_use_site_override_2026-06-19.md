# SPEC — faithful `restore --md1` of per-cosigner use-site path overrides

**Date:** 2026-06-19
**Slug:** `restore-md1-per-key-use-site-and-hardened-wildcard` (the wsh/sh-wsh + general-policy leg; taproot leg deferred to `restore-md1-taproot-use-site-override-arm`).
**Source SHAs (grep-verified at write time):** mnemonic-toolkit `4783f02` (master; code identical to `74659f6`, only FOLLOWUPS.md added since), descriptor-mnemonic `c85cd49` (main), mnemonic-key `913febc`.
**SemVer:** md-codec **MINOR** (`0.36.0 → 0.37.0`; adds a `pub` hardened-anywhere predicate + faithful per-key reconstruction — see §6), mnemonic-toolkit **PATCH**, md-cli **PATCH**. No GUI surface change.

This SPEC passes the mandatory opus R0 gate (0C/0I) BEFORE any code. Per-phase TDD + per-phase R0 thereafter.

---

## 1. Motivation (funds-safety)

An md1 multisig card can carry **per-cosigner use-site path overrides** — cosigners whose derivation suffix diverges from the shared baseline, e.g. `wsh(multi(2, @0/<0;1>/*, @1/<2;3>/*))`. The override data is losslessly on the card (`md-codec tlv.rs:26 use_site_path_overrides: Option<Vec<(u8, UseSitePath)>>`, per-`@N`). But **two derivation paths silently apply the baseline suffix to every key**, ignoring the overrides:

1. **md-codec `to_miniscript_descriptor`** (`to_miniscript.rs:60`) passes the shared `&d.use_site_path` to every key instead of the per-key `&e.use_site_path`.
2. **toolkit `ReconstructTranslator`** (`restore.rs:~1125`) holds `multipath: d.use_site_path.multipath.clone()` (baseline) and `Translator::pk` rebuilds every key's `MultiXPub` from it, discarding each key's resolved path.

**Live bug:** `md address <divergent-card>` (md-cli) → `derive_address` → `to_miniscript_descriptor` → **silently returns WRONG addresses** (no guard; `cmd/address.rs:38` gates only `is_wallet_policy`). The toolkit `restore` is shielded only because it *refuses* override cards (`restore.rs:1247`). This SPEC closes the silent-wrong-derivation bug and makes `restore` reconstruct faithfully, for the non-taproot leg.

## 2. Verified facts (recon + architect pre-spec confirmation)

| Fact | Evidence |
|---|---|
| Override data is on the card, per-`@N` | `tlv.rs:26 Vec<(u8, UseSitePath)>`; TLV tag `0x00` |
| The per-key path is **already resolved** in `ExpandedKey` | `canonicalize.rs:344 pub use_site_path: UseSitePath`; `expand_per_at_n` `:458-460` `sparse_lookup(use_site_path_overrides, idx).cloned().unwrap_or_else(|| d.use_site_path.clone())`; per-`@N` correspondence is unambiguous — `idx` == position in the returned `Vec` (`canonicalize.rs:339`) |
| **Plain-arm routing is baseline-only** (C1 locus) | `restore.rs:1289 plain_template_from_tree(&d.tree, &d.use_site_path)` returns `Some` iff baseline == `standard_multipath()` (`:1148`); never consults overrides. Plain renderer hardcodes `/<0;1>/*` per key (`wallet_export/pipeline.rs:85`, `:132` for `tr` multi_a) |
| **`to_miniscript_descriptor` errors on a hardened ALT** (I2) | `to_miniscript.rs:125-127 use_site_to_derivation_path → Err(HardenedPublicDerivation)`, independent of the `:90` wildcard line |
| `to_miniscript_descriptor` discards it | `to_miniscript.rs:60` passes `&d.use_site_path` (baseline), `:90` hardcodes `Wildcard::Unhardened` |
| Both consumers converge on `to_miniscript_descriptor` | toolkit `restore.rs:1109 faithful_multisig_descriptor`; md-cli `derive.rs:120` (via `derive_address`) |
| **Point A:** toolkit faithful arm clobbers per-key paths a 2nd time | `restore.rs:~1125 multipath: d.use_site_path.multipath.clone()`; `Translator::pk` rebuilds `MultiXPub` from baseline |
| Encoder is canonical/divergent-only; `@0` is the baseline | `parse_descriptor.rs:194-201` + md-cli `parse/template.rs:201-209` (push iff `usp_i != baseline`, baseline=`@0`) |
| `@0`-baseline NOT enforced at decode | `tlv.rs:323-332` checks only `idx<n` + ascending, not `idx≥1` |
| Divergent alt-COUNT already rejected at decode | `validate.rs:117 validate_multipath_consistency` (called `decode.rs:57-58`); but skips `None` entries (`:124`) |
| sortedmulti sort delegated to rust-miniscript (sorts derived keys) | `to_miniscript.rs:205/231/248` |
| Overrides are multisig-only | emit loop `for i in 1..n`; all `--md1` → `run_multisig` (`restore.rs:178`) |
| md-cli `address` has NO override/hardened-override guard | `cmd/address.rs:23` (`is_wallet_policy` only), `:38` `derive_address` |
| Hardened check is baseline-only in both paths | `restore.rs:1254`, `derive.rs:99/110` |

## 3. Scope

### IN
- **md-codec:** `to_miniscript_descriptor` consumes the already-resolved per-`@N` `ExpandedKey.use_site_path`; faithful wildcard. A shared `pub` hardened-anywhere predicate. Decode-time D5(a)/(b) hardening.
- **toolkit:** `ReconstructTranslator` per-`@N` aware (Point A); narrow the `restore.rs:1247` guard for the now-restorable shapes; advisory parity; reuse the md-codec predicate; address-equivalence round-trip test; md-codec pin bump; manual update.
- **md-cli:** inherits the fix via `to_miniscript_descriptor`; add a regression test proving correct addresses for a divergent card.
- Covers `wsh(multi)`, `sh(wsh(multi))`, `sh(multi)` and **general wsh/sh-wsh policies** (the faithful arm) with divergent **value** suffixes.

### OUT (explicitly)
- **Taproot override cards at toolkit `restore`** — stay GUARDED (the Template arm + `sortedmulti_a` gap route around the fix). Deferred → FOLLOWUP `restore-md1-taproot-use-site-override-arm` (filed `4783f02`). NOTE: md-cli `address`/`derive_address` of taproot override cards IS fixed (goes through `to_miniscript_descriptor`); only toolkit restore's taproot arm is deferred.
- **Hardened wildcard `/*h`** (baseline OR any override) — WONTFIX for watch-only (cannot derive hardened from an xpub). Stays a LOUD refusal everywhere via the shared predicate.

## 4. Design

### 4.1 md-codec (descriptor-mnemonic) — MINOR

- **D1 — faithful per-key reconstruction.** In `to_miniscript_descriptor` (`to_miniscript.rs:53`), pass `&e.use_site_path` (the already-resolved per-key path) instead of `&d.use_site_path` at the `:60` call site. In `build_descriptor_public_key`, set `wildcard` from the resolved path: `if use_site.wildcard_hardened { Wildcard::Hardened } else { Wildcard::Unhardened }` (replacing the hardcoded `:90`). Note this fixes the per-key derivation VALUE for `derive_address`; the toolkit's faithful descriptor STRING needs C1+C2 below (D1 alone does NOT make the toolkit faithful — round-1 R0 C1/C2).
  - **I2 scope correction:** `to_miniscript_descriptor` is faithful-text for a hardened **wildcard** (`/*h`, `:90` honored) but NOT for a hardened **alt inside an override** — `use_site_to_derivation_path` independently `Err(HardenedPublicDerivation)`s at `to_miniscript.rs:125-127`. That is acceptable: ALL hardened cases (baseline/override, wildcard/alt) route to a loud refusal via the Point B predicate BEFORE any render/derive, so `to_miniscript_descriptor` is never asked to text-render a hardened-alt override on a restorable path. Do NOT move the `:126` reject; just scope the "faithful text" claim to non-hardened + hardened-wildcard.
- **Point B — shared hardened-anywhere predicate at the derivation boundary.** Add `pub fn has_hardened_use_site(d: &Descriptor) -> bool` scanning `d.use_site_path` AND every entry in `d.tlv.use_site_path_overrides` for `wildcard_hardened == true` OR any `Alternative.hardened == true` in `multipath`. In `derive_address`, replace the baseline-only checks (`derive.rs:99/110`) with this predicate (returns `Error::HardenedPublicDerivation` cleanly — closes the latent gap where a hardened alt *inside an override* currently surfaces only as a generic `AddressDerivationFailed`). Do NOT reject inside `to_miniscript_descriptor` (it must stay faithful for text). Export the predicate `pub` for toolkit reuse (single source of truth — Point B rationale).
- **D5(a) — decode canonical-form check.** In decode (`decode.rs` post-`expand`/validate), reject: (i) any `use_site_path_overrides` entry with `idx == 0` (the `@0` baseline cannot be overridden — guards the decode invariant gap, Point D); (ii) any override whose `UseSitePath` equals the resolved baseline (redundant; non-canonical). New `Error` variants `RedundantUseSiteOverride { idx }` and `BaselineUseSiteOverride { idx }` (M2 — `{ idx }` style matching the existing enum, e.g. `OverrideOrderViolation` `error.rs:137`). Alphabetical-ordered if the error enum is sorted.
- **D5(b) — tighten multipath consistency.** Extend `validate_multipath_consistency` (`validate.rs:117`, currently skips `None` entries at `:124`) so a `Some`-multipath baseline mixed with a `None`-multipath override (a legal divergent *structure*, e.g. `@0/<0;1>/*` + `@1/*`) is recognized. This is **legal-and-must-be-supported**, NOT a reject: `use_site_to_derivation_path` already derives the `None`-key ADDRESS correctly (`to_miniscript.rs:118`), but the faithful descriptor STRING for a `None`-override is exactly the C2 faithful-arm case (a `None`-override must round-trip to a single non-multipath key while `@0` stays multipath) — so D5(b) is NOT merely test-coverage; the C2 reconstruction must handle it, and the oracle MUST include a `Some`/`None`-mix shape. Divergent alt-COUNT stays rejected (already is).

### 4.2 mnemonic-toolkit — PATCH

The toolkit emits the descriptor STRING at TWO sites that D1 does not touch; both must be fixed or the narrowed guard exposes a silent-wrong-descriptor (round-1 R0 C1/C2, both proven by the existing pinned test `cli_restore_multisig_general.rs:414` = `wsh(multi(2,@0/<0;1>/*,@1/*))`).

- **C1 — route ALL override cards to the faithful arm.** The plain template arm cannot express divergent suffixes: its renderer hardcodes `/<0;1>/*` per key (`wallet_export/pipeline.rs:85`, `:132` for `tr` multi_a). But `plain_template_from_tree` (`restore.rs:1289`) returns `Some(...)` whenever the `@0` baseline is standard (`:1148`), ignoring overrides — so the natural shape (standard `@0`, divergent `@1`) routes to the plain arm and mis-renders `@1` while printing CORRECT receive addresses (maximally deceptive). **Fix:** `plain_template_from_tree` returns `None` when `d.tlv.use_site_path_overrides.is_some()` (gate the plain arm on "no overrides AND baseline standard"). All non-taproot non-hardened override cards then route to the faithful arm. This is NOT implied by D1 and must be explicit.
- **C2 — faithful arm must reconstruct per-`@N` multipath GROUPS, not the chain-0 collapse.** `faithful_multisig_descriptor` (`restore.rs:1109`) calls `to_miniscript_descriptor(d, 0)` — chain-0, single-path — so each key's full group (`@1 → <2;3>`) collapses to one child number; the `ReconstructTranslator` then re-promotes every key from the single baseline `self.multipath` (`:1060-1084`, `:1125`), discarding the per-key group. Re-deriving `@N`→key in the toolkit is fragile (`iter_pk` order ≠ `@N` order in general policies). **Fix — md-codec owns the correspondence:** md-codec exposes the resolved per-`@N` keys WITH their full multipath from `ExpandedKey.use_site_path.multipath` (`canonicalize.rs:344`), where `@N` == Vec position (`:339`, unambiguous). The toolkit faithful arm consumes these directly — REPLACING the `ReconstructTranslator` baseline-`self.multipath` re-promote — so each key gets ITS group. A `None`-multipath override → a single non-multipath `XPub` key while `@0` stays `MultiXPub` (the D5(b) `Some`/`None`-mix). Exact API shape (a `pub fn` returning the per-`@N` `DescriptorPublicKey` set, vs a multipath-faithful descriptor builder) is an implementation-plan decision (its own per-phase R0); the SPEC REQUIRES faithful per-`@N` multipath reconstruction with the `@N`=position correspondence sourced from md-codec, covering the `Some`/`None` mix.
- **Guard narrowing (`restore.rs:1240-1260`).** Replace the blanket `if d.tlv.use_site_path_overrides.is_some()` refusal (`:1247`) with: refuse iff `md_codec::has_hardened_use_site(d)` (covers baseline AND override hardened — supersedes the baseline-only `:1254`), OR `taproot_override_card(d)` (Point C — `Tag::Tr` root AND `use_site_path_overrides.is_some()`). Define `taproot_override_card` as ONE named predicate reused by the guard AND the advisory (M3). Non-taproot, non-hardened override cards proceed (via C1→faithful + C2). §5.6 enumerates that every previously-caught shape is either faithfully reconstructed or still loudly refused.
- **Advisory parity (`unrestorable_advisory.rs`).** Drop the `PerKeyUseSiteOverrides` arm (`:81-85`) — now restorable. Make `HardenedWildcard` (`:86`) override-aware by reusing `md_codec::has_hardened_use_site`. Add a `TaprootUseSiteOverride` shape whose detector is the SAME `taproot_override_card(d)` expression the guard uses (M3 — single source ⇒ exact parity: advisory fires IFF restore refuses). Keep `SortedMultiInCombinator`.
- **Pin bump + manual.** Bump the md-codec dep to `0.37.0`; update the manual `### Unrestorable descriptor shapes` section (non-taproot overrides now restorable; taproot overrides + hardened still listed). Run `make -C docs/manual audit` (captured-output discipline).

### 4.3 md-cli (descriptor-mnemonic) — PATCH

- Inherits the override fix transitively (its `derive_address` → `to_miniscript_descriptor`) and the clean hardened refusal (Point B predicate in `derive_address`). Add a regression test: `md address` on a divergent-suffix multisig card yields the CORRECT per-cosigner addresses (was silently wrong). No md-cli code change beyond the test (the fix is in md-codec it depends on).

## 5. Test / oracle strategy (the funds-safety gate)

The bar is **address-equivalence**, not exit-0 or string-equality (canonicalization differs). Oracle = Bitcoin Core `deriveaddresses` via the existing differential harnesses.

1. **md-codec differential** (`descriptor-mnemonic tests/bitcoind_differential.rs`, calls `to_miniscript_descriptor` directly): ADD a `wsh(multi)` divergent shape, a `tr(multi_a)` divergent shape, and a `Some`/`None`-multipath-mix shape. **I1 — avoid self-reference:** this harness feeds bitcoind the string from `to_miniscript_descriptor` and compares to md-codec `derive_address` (`:681/:715/:738`) — BOTH sides derive from the SAME rendering, so a divergent shape passes VACUOUSLY even if D1 is still buggy. Each divergent shape MUST pin an INDEPENDENTLY-computed golden address for the diverging cosigner (a known BIP-32 derivation of `@1` at its own `<…>/0`), not bitcoind self-agreement.
2. **md-codec unit** (`to_miniscript` / `derive`): per-`@N` faithful reconstruction; `has_hardened_use_site` truth table (baseline-hardened, override-hardened-wildcard, override-hardened-alt, all-unhardened); D5(a) decode rejects (`@0` override, redundant override) — RED-first.
3. **toolkit differential** (`tests/bitcoind_differential.rs`): ADD a `wsh(multi)` divergent shape → full `bundle → restore` round-trip, assert reconstructed addresses == original (exercises C1 routing + C2 faithful arm). The harness's `single_chain_desc`/`derive_receive` (`:203`) use rust-miniscript `into_single_descriptors`, which handle per-key multipath natively — NO machinery change needed, just a new corpus entry; `derive_receive` IS an independent rust-miniscript oracle (sound for the end-to-end anchor). Additionally assert the reconstructed descriptor STRING carries `@1`'s divergent suffix (catches a wrong-string-but-right-address regression directly).
4. **toolkit guard/parity tests** (`prop_backup_restore_roundtrip.rs` + restore tests): (a) non-taproot non-hardened override card → restore SUCCEEDS faithfully (flip the existing refuse-pin); (b) `tr(multi_a)` override card → restore REFUSES loudly (Point C guard); (c) hardened override (non-taproot) → restore REFUSES loudly (Point B); (d) baseline `/*h` → still refuses. Advisory PARITY tests: advisory fires IFF restore refuses, for each shape.
5. **md-cli regression** (§4.3): `md address` divergent card == correct addresses.

Anti-vacuity: each new RED test must fail on current `origin/master` before the fix.

### 5.6 — Funds-safety shape enumeration (the gate evidence)

After the folds, EVERY override-card shape resolves to faithful-reconstruction, loud-refusal, or decode-reject — NO silent-mis-render path remains (validated independently in R0 round 2):

| Override-card shape | Outcome | Mechanism |
|---|---|---|
| `wsh(multi)` / `wsh(sortedmulti)` / `sh(wsh(…))`, any `@0` baseline, non-hardened, non-taproot, `Some`/`Some` OR `Some`/`None` divergent | **Faithful** | C1 (`plain_template_from_tree → None` on overrides) routes to the faithful arm; C2 reconstructs per-`@N` groups |
| `sh(multi)` (bare P2SH) override, non-hardened | **Faithful** | `plain_template_from_tree` matches only Wsh/Sh-Wsh → `None` → faithful arm; C2 |
| ANY override card where baseline OR any override is hardened (`/*h` wildcard OR hardened alt) | **Loud refuse** | `has_hardened_use_site(d)` guard (Point B); advisory `HardenedWildcard` (parity) |
| `tr(…)` root WITH overrides (`multi_a` / `sortedmulti_a`) | **Loud refuse** | `taproot_override_card(d)` guard (Point C); advisory `TaprootUseSiteOverride` (parity); FOLLOWUP `restore-md1-taproot-use-site-override-arm` |
| Adversarial wire: override at `@0`, or override `==` baseline | **Decode reject** | D5(a) `BaselineUseSiteOverride` / `RedundantUseSiteOverride` (never emitted by our encoders — §7 M4) |
| Non-override card (no override TLV) | Unchanged | plain or faithful arm exactly as today |

(md-cli `address`/`derive_address` of a non-taproot non-hardened override card → faithful addresses via the D1 + Point B fix; taproot override cards via md-cli are also fixed since they go through `to_miniscript_descriptor` — only the toolkit restore taproot arm is deferred.)

## 6. SemVer, ordering, locksteps

- **md-codec MINOR `0.37.0`** — driven by the new `pub fn has_hardened_use_site` (additive stable API) + the faithful-reconstruction behavior change (which is itself a bugfix for the silent md-cli mis-derivation). Publish to crates.io. (If R0 prefers to keep the predicate `pub(crate)` and have the toolkit duplicate the scan, this drops to PATCH `0.36.1` — but the shared predicate is the Point-B single-source-of-truth recommendation; default MINOR.)
- **toolkit PATCH** — no new flag/subcommand/dropdown ⇒ **no GUI `schema_mirror`**, no manual flag-coverage change (only the prose `### Unrestorable descriptor shapes` updates → `make audit`). Pins md-codec `0.37.0`.
- **md-cli PATCH** — test-only + the md-codec dep bump (exact-pin per its convention).
- **Ordering:** md-codec ships + publishes FIRST → md-cli pin/test → toolkit pin-bump + Point-A + guard + advisory + tests (single coupled toolkit PR; guard + advisory + parity tests MUST land together).
- **Companions:** descriptor-mnemonic companion FOLLOWUP for the md-codec change; the taproot deferral FOLLOWUP is already filed (`4783f02`).

## 7. Risks / R0 focus

- **Funds-safety is the whole point.** R0 + impl-review must weight: (a) the address-equivalence differential actually covers divergent suffixes with an INDEPENDENT golden (I1 — not the harness's self-referential or uniform `/0/i` derivation); (b) C1 routing + C2 faithful per-`@N` reconstruction together close the descriptor-STRING hole (the general-policy arm is the subtle one, and the bare-`/*` `None`-override is the proof case); (c) the guard narrowing leaves NO silent-mis-render hole — every previously-caught shape is either faithfully reconstructed or still loudly refused (§5.6 enumeration); (d) advisory parity holds exactly (the `taproot_override_card`/`has_hardened_use_site` predicates are shared by guard + advisory).
- **Decode hardening (D5) must not break valid cards** — VERIFIED (M4): both encoders push overrides only for `i≥1` and only when `usp_i != baseline` (`parse_descriptor.rs:198`, md-cli `parse/template.rs:207`), so neither a `@0` entry nor a redundant entry is ever emitted; the D5(a) rejects fire only on hand-crafted/adversarial wire. Impl-review must still confirm round-trip of all existing corpus cards passes.
- Per-phase R0 gate applies to the implementation plan-doc and each phase before any code.

## 8. Source-of-truth citations (re-grep at plan-doc time)
All line numbers are snapshots at the SHAs in the header; re-grep against current `origin/master`/`origin/main` when lifting into the implementation plan-doc (per CLAUDE.md citation-decay discipline).
