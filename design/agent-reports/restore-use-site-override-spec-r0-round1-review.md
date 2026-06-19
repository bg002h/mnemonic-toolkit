# SPEC R0 round 1 — restore per-key use-site override (opus architect, verbatim)

> Reviewer: opus architect (Plan agent; full read+bash, verified every citation against descriptor-mnemonic `c85cd49` / toolkit `4783f02`, and demonstrated both Criticals against existing pinned tests).
> SPEC: `design/SPEC_restore_per_key_use_site_override_2026-06-19.md`. **Verdict: RED (2 Critical, 2 Important, 5 Minor).** Folded → re-dispatched (see round 2).

---

This is the final confirming evidence. Look at the existing pinned test `per_key_use_site_override_refused` at `:414-415`:

```
let md1 = bundle_general("wsh(multi(2,@0/<0;1>/*,@1/*))");
```

This is EXACTLY the divergent-override shape with a STANDARD `@0` baseline (`@0/<0;1>/*`) and a diverging `@1/*` (bare wildcard, no multipath = override). Currently it is REFUSED at `:1247`. The SPEC wants to flip this to SUCCEED faithfully. But:
- `@0` baseline = `<0;1>` = standard → `plain_template_from_tree` returns `Some(WshMulti)` → PLAIN arm.
- Plain arm renders BOTH keys as `/<0;1>/*` (`build_descriptor_string_inner:85`), so `@1` becomes `<0;1>/*` instead of the bare `/*` the card encodes. SILENT MIS-RENDER.

This concretely proves my CRITICAL #1: the SPEC's "plain arm needs nothing beyond D1" is false; this very shape routes through the plain arm and mis-renders. And it proves CRITICAL #2: even forced to the faithful arm, `@1`'s `multipath: None` (bare `/*`) vs `@0`'s `<0;1>` cannot be reconstructed because `to_miniscript_descriptor(d, 0)` + the baseline-`self.multipath` translator would render `@1` with the baseline group.

I have everything. Final verdict: RED.

---

# R0 Review — SPEC_restore_per_key_use_site_override_2026-06-19.md

**Verdict: RED.** Two Criticals and two Importants. The design intent and most citations are sound, but the SPEC's implementation plan leaves the exact silent-mis-render hole it exists to close, because it mis-models how `restore` ROUTES and RENDERS override cards (it fixes the per-key *value* in `to_miniscript_descriptor` but not the two toolkit sites that actually emit the descriptor STRING). Both Criticals are concretely demonstrated by an existing pinned test.

## Citation verification (all confirmed against `c85cd49` / `4783f02`)

Every file:line in the SPEC checks out (modulo the `crates/` path prefix shorthand, which is fine):
- `to_miniscript.rs:60` passes `&d.use_site_path`; `:90` hardcodes `Wildcard::Unhardened`. ✓
- `canonicalize.rs:344` `ExpandedKey.use_site_path`; `:458-460` resolves per-`@N` (actual is `.cloned().unwrap_or_else(|| d.use_site_path.clone())` — SPEC's `.unwrap_or(...)` is cosmetically loose but semantically identical). ✓
- `restore.rs:1247` blanket override refusal, `:1254` baseline-hardened refusal, `:1109` faithful arm, `:1125` baseline-multipath clobber, `:1060-1084` `MultiXPub` arm, `:1289` plain-arm routing on `&d.use_site_path`, `:1380-1389` plain-arm `derive_address`. ✓
- `derive.rs:99/110` baseline-only hardened checks; `derive.rs:120` sole md-codec `to_miniscript_descriptor` caller. ✓
- `cmd/address.rs:23` `is_wallet_policy` only, `:38-39` `derive_address` (live bug, no guard). ✓
- `tlv.rs:26` override field, tag `0x00` (`:11`); `tlv.rs:323-332` `read_sparse_tlv_idx` checks `idx<n`+ascending, NOT `idx≥1`. ✓
- `validate.rs:117` `validate_multipath_consistency`, `:124` skips `None` entries (alt-COUNT only); `decode.rs:57-58` call site. ✓
- `parse_descriptor.rs:194-201` + md-cli `parse/template.rs:201-209`: baseline=`@0`, push iff `usp_i != baseline`. ✓
- `unrestorable_advisory.rs:81-85` (PerKeyUseSiteOverrides), `:86-90` (HardenedWildcard). ✓
- Versions: md-codec `0.36.0`; toolkit pin `"0.36"`; md-cli `=0.36.0`. `has_hardened_use_site` does not yet exist. ✓
- Both differential harnesses + `prop_backup_restore_roundtrip.rs` exist. Manual `### Unrestorable descriptor shapes` exists (`41-mnemonic.md:58`, lists overrides + `/*h`). ✓

## CRITICAL

### C1 — The plain arm silently mis-renders the very override cards the guard now admits (the funds-safety hole is not closed; it's relocated and exposed)

The SPEC §4.2 asserts: *"The plain `wsh/sh-wsh` arm (`~:1380-1389`) uses `d.derive_address` → fixed by D1 alone."* This is **false for the descriptor STRING**.

- Routing is keyed on the BASELINE only: `plain_template_from_tree(&d.tree, &d.use_site_path)` (`restore.rs:1289`), which returns `Some(...)` whenever `d.use_site_path == standard_multipath()` (`restore.rs:1148`). It never consults `d.tlv.use_site_path_overrides`.
- The encoder sets the baseline to `@0` and only pushes overrides for `i≥1` (`parse_descriptor.rs:193-199`). So the natural shape `wsh(multi(2, @0/<0;1>/*, @1/<2;3>/*))` has a STANDARD `@0` baseline and routes to the **plain arm**.
- The plain arm's renderer hardcodes the suffix for every key: `format!("{origin}{}/<0;1>/*", s.xpub)` (`wallet_export/pipeline.rs:85`). It is structurally incapable of expressing a divergent per-key suffix. `@1` renders as `/<0;1>/*` instead of `/<2;3>/*` — a SILENT wrong descriptor. (The printed receive addresses come from `d.derive_address(0,i)` and are correct post-D1, so the mismatch between a CORRECT address list and a WRONG descriptor is maximally deceptive.)

This is proven by the existing pinned test `cli_restore_multisig_general.rs:414-415`, which the SPEC explicitly wants to flip to SUCCESS:
```
let md1 = bundle_general("wsh(multi(2,@0/<0;1>/*,@1/*))");  // @0 baseline standard, @1 diverges
```
This shape routes to the plain arm and mis-renders `@1`. The SPEC's own toolkit differential (test 5.3, a `wsh(multi)` divergent shape) would therefore fail RED against the SPEC's stated plain-arm fix ("nothing beyond D1") — the SPEC is internally inconsistent.

**Fix to fold:** the routing/guard must steer ALL override cards to the faithful arm. Minimum: `plain_template_from_tree` must return `None` when `d.tlv.use_site_path_overrides.is_some()` (add an early `if use_site_path_overrides present → None`), so override cards never reach `build_descriptor_string`. Equivalently, gate the plain arm on "no overrides AND baseline standard." State this explicitly in §4.2; it is not implied by D1.

### C2 — Point A's data source is wrong: the faithful arm cannot reconstruct the per-key multipath GROUP from a single `to_miniscript_descriptor(d, 0)` call

SPEC §4.2 Point A says the translator must *"preserve/consume the per-key path already on each `DescriptorPublicKey`."* But `faithful_multisig_descriptor` calls `to_miniscript_descriptor(d, 0)` (`restore.rs:1109`) — **chain 0 only**. Post-D1 that yields, for each key, a SINGLE-path `XPub` whose `derivation_path` is just `[alt0_of_that_key]` (e.g. `@1 → [2]`), with the full group `<2;3>` discarded. The `ReconstructTranslator` then re-promotes to `MultiXPub` from `self.multipath` (`restore.rs:1060-1084`), a SINGLE baseline group cloned from `d.use_site_path.multipath` (`:1125`).

So "preserving the per-key path D1 produces" gives the translator only the chain-0 child number, not the per-key multipath group. Whether the translator (a) re-promotes via the baseline group → renders `@1` as `<0;1>` (wrong), or (b) keeps the single chain-0 path → renders `@1` as `/2/*` (loses the change chain) — **either way the divergent multipath group is unrecoverable from this call.** The SPEC's Point A is underspecified and, as worded, cannot produce a faithful divergent string.

The required data lives in `ExpandedKey.use_site_path.multipath` (`canonicalize.rs:344`, full per-`@N` group).

**Fix to fold:** re-architect `ReconstructTranslator` to carry a per-`@N` map of the full `UseSitePath.multipath` (from `expand_per_at_n(d)`), keyed so each key gets ITS group, not `self.multipath`. The translator must match each incoming key to its `@N` and emit that key's `DerivPaths`. (Matching keys positionally is fragile because `iter_pk` order need not equal `@N` order in general policies; the implementation plan must specify a sound `@N`→key correspondence, e.g. carrying the resolved keys out of `to_miniscript_descriptor` rather than re-deriving via `translate_pk`.) Also note the bare-`/*` override case (`@1/*`, `multipath: None`) must round-trip to a non-multipath single key while `@0` stays multipath — the `Some`/`None`-mix the SPEC flags in D5(b) is exactly this faithful-arm case, not merely a test-coverage item.

## IMPORTANT

### I1 — The md-codec differential (SPEC test 5.1) is self-referential for divergent shapes and would pass even if D1 were still buggy

`tests/bitcoind_differential.rs` feeds bitcoind the string from `to_miniscript_descriptor(&shape.desc, chain)` (`:681,715`) and compares to md-codec `derive_address` (`:738`) — both sides derive from the SAME rendering. For a divergent shape, the original `:60` bug renders `@1` with the baseline suffix; bitcoind and md-codec then AGREE on the same wrong address → the test passes vacuously. The only external anchor is the `wpkh` BIP-84 golden (`:751-757`), which does not exercise divergence. As written, this "most direct D1 oracle" does NOT catch the D1 bug for the divergent key.

**Fix to fold:** the added divergent shape MUST pin an INDEPENDENTLY-computed golden address for the diverging cosigner (a known BIP-32 derivation of `@1` at `<2;3>/0`), not just bitcoind self-agreement. (The toolkit differential's `derive_receive` at `tests/bitcoind_differential.rs:203` IS an independent rust-miniscript oracle and is sound — lean on that for the end-to-end anchor, but the md-codec-level golden must still be pinned.)

### I2 — D1's wildcard/hardened change breaks the SPEC's own "stays faithful for text" claim for override-hardened-ALT cards, because `to_miniscript_descriptor` itself rejects hardened alts at `to_miniscript.rs:126`

SPEC §4.1 D1: *"Do NOT reject inside `to_miniscript_descriptor` (it must stay faithful for text)."* But `use_site_to_derivation_path` already returns `Err(HardenedPublicDerivation)` on a hardened ALT (`to_miniscript.rs:125-127`), independent of the `:90` wildcard line. After D1 switches the call to `&e.use_site_path`, an override whose multipath has a hardened alt will make `to_miniscript_descriptor` ERROR rather than render — so "faithful text for `/*h`" is only half-true: it holds for hardened-WILDCARD (`:90` honored) but NOT for hardened-ALT-in-override (`:126` still errors). This is harmless for `derive_address` (the Point B predicate pre-flights and rejects first) but matters for any direct `to_miniscript_descriptor` text caller and contradicts the SPEC's stated invariant.

**Fix to fold:** either (a) acknowledge `to_miniscript_descriptor` is NOT a faithful text renderer for hardened-alt overrides (it errors), and scope the "faithful text" claim to non-hardened, or (b) if faithful `/*h`-and-hardened-alt text is genuinely wanted, the `:126` reject must move out of the shared path. Given Point B routes all hardened cases to a loud refusal anyway, option (a) (a one-line scope correction in §4.1) is sufficient — but it must be stated so the predicate's single-source-of-truth claim isn't undermined by a second, divergent hardened-reject at `:126`.

## Minor (non-blocking, fold if convenient)

- **M1** §2 fact-table writes the resolution as `.unwrap_or(d.use_site_path.clone())`; actual is `.unwrap_or_else(|| d.use_site_path.clone())` (`canonicalize.rs:460`). Cosmetic.
- **M2** §4.1 D5(a) proposes error names `RedundantUseSiteOverride`/`BaselineIndexOverride`; the existing enum uses a `{ idx }`-style and names like `OverrideOrderViolation` (`error.rs:137`). Suggest `RedundantUseSiteOverride { idx }` and `BaselineUseSiteOverride { idx }` (or `UseSiteOverrideAtBaseline`) for naming consistency. Additive variants — MINOR-safe.
- **M3** §4.2 names the new advisory `TaprootUseSiteOverride`. Detection must match the guard EXACTLY (`Tag::Tr` root AND `use_site_path_overrides.is_some()`) for the parity invariant; spell that out so the advisory predicate and the guard predicate are the same expression (single source).
- **M4** The D5(a) redundant-override reject and `@0`-override reject are safe against the existing corpus (both encoders, `parse_descriptor.rs:198` and `template.rs:207`, push only `i≥1` and only when `!=` baseline, so neither a `@0` entry nor a redundant entry is ever emitted) — confirmed, no valid corpus card breaks. Worth a one-line note in §7 that this was verified, not just asserted.
- **M5** `Some`/`None` multipath-mix derives correctly TODAY only for the chain-0 ADDRESS via `use_site_to_derivation_path` (`to_miniscript.rs:118` handles `None` by emitting an empty path). The DESCRIPTOR-STRING faithful reconstruction of a `None`-override does NOT work today (it is the C2 case). The SPEC frames it purely as "test-coverage + documentation, not a reject" — that's right for `derive_address`, but the faithful-arm string is part of C2, so don't let D5(b)'s "legal-and-covered" framing imply the reconstruction string is already correct.

## Answers to the eight verification questions (concise)

1. **D1 simplicity:** Correct mechanism (`&d.use_site_path → &e.use_site_path` at `:60`, wildcard from resolved path replacing `:90`). No hidden coupling for the VALUE in `derive_address`. BUT `to_miniscript.rs:126` independently rejects hardened alts (see I2), and the rendered STRING consumed by the toolkit is re-clobbered downstream (C1/C2) — so D1 alone does not make the toolkit faithful.
2. **Point A:** The re-clobber is real (`:1060-1084` rebuild from `self.multipath`; `:1125` baseline). The SPEC's fix is NOT sufficient as worded (C2): a single chain-0 call cannot supply the per-key group. The plain `wsh/sh-wsh` arm does NOT "need nothing beyond D1" — it mis-renders override cards (C1).
3. **Guard narrowing safety:** Current `:1247` catches ALL override cards across every shape. Narrowed guard (`has_hardened_use_site OR Tag::Tr+overrides`) correctly keeps taproot+hardened guarded, and taproot-override detection (`Tag::Tr` root + `use_site_path_overrides.is_some()`) is correct and reachable. `is_wallet_policy` (`:1232`) + multisig routing DO guarantee all wallet-policy override cards reach the guard. **But narrowing EXPOSES the C1/C2 mis-render path for non-taproot non-hardened plain-arm shapes** (`wsh(multi)`, `wsh(sortedmulti)`, `sh(wsh(multi/sortedmulti))` with standard `@0` baseline). Bare `sh(multi)` and non-standard-`@0`-baseline cards correctly route to faithful. The guard narrowing therefore is NOT safe until C1+C2 are fixed.
4. **Point B predicate:** Scanning `wildcard_hardened` + `Alternative.hardened` across baseline+overrides fully covers the hardened space (struct confirmed `use_site_path.rs:19-54`). `derive_address` (`derive.rs:120`) is the ONLY md-codec derivation entry. Replacing `:99/:110` suffices for md-codec. (Toolkit reuses the `pub fn` cross-repo — sound.)
5. **D5(a)/(b):** Decode (`decode.rs:57-58`) is the right place. `@0`-override + redundant-override rejects break NO existing corpus card (encoders never emit them — verified at `parse_descriptor.rs:198` / `template.rs:207`). `Some/None` mix derives correctly for the ADDRESS today but NOT for the faithful STRING (part of C2) — so it is more than a test-coverage item.
6. **Oracle:** Both harnesses exist and accept added shapes. md-codec differential is self-referential for divergence (I1 — needs an independent golden). Toolkit differential's `single_chain_desc`/`derive_receive` use rust-miniscript `into_single_descriptors`, which handle per-key multipath natively — NO machinery change needed; just add a divergent-suffix corpus entry. (The uniform `/<0;1>/*` is only in corpus construction `:81`, not the split logic.)
7. **SemVer:** MINOR is correct (new `pub fn` is additive; additive error variants; behavior change is a bugfix on previously-guarded shapes). md-cli `=0.37.0`, toolkit `0.37`. No other public-surface change found.
8. **Scope leaks:** (a) No 3rd md-codec derivation site (only `derive.rs:120`). (b) The TOOLKIT has a second descriptor-emitting site the SPEC under-weights — `build_descriptor_string` / `build_tr_multi_a_descriptor` (`wallet_export/pipeline.rs:85,132`) hardcode `/<0;1>/*`; this is the locus of C1. (c) `TaprootUseSiteOverride` advisory is warranted and consistent (parity with the new taproot guard) — fold M3. (d) No schema/`schema_mirror` concern (PATCH, no flag/dropdown — SPEC §6 correct). (e) Manual section exists and the update is in scope.

## What turns this GREEN
Fold into the SPEC: **(C1)** route ALL override cards away from the plain arm (`plain_template_from_tree` returns `None` on `use_site_path_overrides.is_some()`); **(C2)** re-architect `ReconstructTranslator` to consume per-`@N` `ExpandedKey.use_site_path.multipath` (full group, incl. `None`-mix), with a specified sound `@N`→key correspondence; **(I1)** pin an independent golden for the divergent cosigner in the md-codec differential; **(I2)** correct the "faithful text" claim re `to_miniscript.rs:126`. Re-run R0 after folding.
