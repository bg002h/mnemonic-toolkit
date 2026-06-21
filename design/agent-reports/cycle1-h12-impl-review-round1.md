# cycle-1 H12 — per-phase implementation review (round 1)

**Reviewer:** opus adversarial implementation reviewer (review-only; no production code written).
**Worktree:** `/scratch/code/shibboleth/mnemonic-toolkit/.claude/worktrees/cycle1-h12h1`
**Branch / commit:** `fix/cycle1-h12-h1` @ `c4b46624`
**Diff under review:** `git diff origin/master...HEAD` (8 files, +550/-18).
**Design:** `design/IMPLEMENTATION_PLAN_cycle1_critical_fixes.md` (P3 / H12);
`design/BRAINSTORM_cycle1_critical_fixes.md`.
**Canonical facts verified against:** toolkit `origin/master`; `md-codec 0.37.0` (the toolkit's
pinned dep) `src/canonical_origin.rs`.

---

## VERDICT: **GREEN — 0 Critical / 0 Important.**

The `Tag::Tr → 3'` (and `Descriptor::Tr(_) → 3'`) blanket is **safe given actual reachability** (see
the dedicated section below). The helper + 3 sites are correct; bundle/verify mirror agreement holds;
the two re-captured goldens are genuinely-correct (independently anchored), not made-to-pass;
`emit_multisig_checks` (P4) is untouched; no flag/CLI/error-variant/GUI/manual surface change. Full
`cargo test -p mnemonic-toolkit` GREEN (0 failed across every test binary); `cargo clippy
--all-targets` clean. One Minor + two notes recorded below.

---

## Reachability: is `Tr → 3'` ever hit for single-key taproot? (the CENTRAL adversarial check)

**Conclusion: NO new wrong-origin bug. At the funds-critical `bundle.rs`/`verify_bundle.rs` site,
single-key BIP-86 taproot is gated OUT of the default-origin path entirely (it is canonical). At the
`descriptor_intake.rs` (xpub-search) site, single-key taproot CAN reach the default, but it ALREADY
got a `m/48'/.../2'` BIP-48-family default pre-H12 — so H12 only flips the leaf `2'→3'` within a
pre-existing (mis)default; it introduces no NEW divergence class.**

### Call trace — `bundle.rs` / `verify_bundle.rs` (the funds-critical site)

`bundle_run_unified` (`bundle.rs:395-396`) dispatches to `bundle_run_unified_descriptor` for ANY
`--descriptor`/`--descriptor-file` invocation — **single-sig and multisig alike** (no multisig gate at
dispatch). Inside (`:1396-1450`):
- `canonicity_probe = parse_descriptor(...)` then
  `is_non_canonical = md_codec::canonical_origin::canonical_origin(&canonicity_probe.tree).is_none()`
  (`:1397-1398`).
- The default-origin inference (and therefore `bip48_script_type_for_root_tag` → `compute_default_origin_path`)
  fires **only inside `if is_non_canonical {` (`:1450`).** `default_script_type` is computed
  unconditionally at `:1447-1448` but is consumed only inside that guard (and by the notice, which is
  itself suppressed when `defaulted_indices.is_empty()`).

The decisive fact is what `canonical_origin` returns for taproot (verified in
`md-codec-0.37.0/src/canonical_origin.rs:45-78`):
- **`tr(@N)` key-path-only — `Body::Tr { tree: None, .. }`** (BIP-86 single-key taproot) →
  `Some(m/86'/0'/0')` (`:52-54`) → `is_non_canonical = FALSE` → **default-origin inference does NOT
  fire.** The `Tr → 3'` mapping is therefore **never reached for plain single-key BIP-86 taproot at
  this site.** (Empirically confirmed by the new clean-negative tests, and by the gate's structure.)
- **`tr(_, TapTree)` — `Body::Tr { tree: Some(_), .. }`** → `None` (`:56`) → `is_non_canonical = TRUE`
  → default fires → `Tag::Tr → 3'`. This branch is the multisig-taproot family the fix targets:
  `tr(NUMS, multi_a/sortedmulti_a)` AND the key-path-spendable `tr(@realkey, <ms>)` (real key at trunk
  + script leaves). For BOTH, `3'` (the BIP-48 taproot leaf) is correct or strictly-more-correct than
  the pre-H12 `2'` (which put taproot cosigner keys in the P2WSH subtree). A taproot descriptor with a
  script tree is never a plain BIP-86 single-key wallet, so emitting the BIP-48-taproot leaf is right.

So at the funds-critical site, `Tr → 3'` is reachable ONLY for `tr(_, Some(taptree))`, i.e. exactly
the (multisig / script-path) taproot family. **No single-key BIP-86 leak.** Confirmed.

### Call trace — `descriptor_intake.rs::parse_literal_xpub` (xpub-search intake)

Here `parsed` is a rust-miniscript `Descriptor`, and the H12 detection is
`let default_script_type = if matches!(parsed, MsDescriptor::Tr(_)) { 3 } else { 2 };` (`:303`),
applied to ANY origin-elided xpub cosigner (`:330`, `:352`). This site is reached for any literal-xpub
descriptor passed to `xpub-search --descriptor` (`intake_from_shape → parse_literal_xpub`, `:141`),
with NO multisig gate before the default — the only guard is `xpub_count == 0` (`:387`). Therefore a
single-key BIP-86 `tr(xpub)` with an elided origin DOES reach the default and gets
`m/48'/coin'/account'/3'` — which is not BIP-86-correct (`m/86'/...`).

**Why this is NOT a Critical (no new bug):** pre-H12 this exact path hardcoded
`bip48_default_path(network, account, 2)` (verified on `origin/master`, `descriptor_intake.rs:324,345`),
so the same single-key `tr(xpub)` already got `m/48'/coin'/account'/2'` — likewise a BIP-48-family
multisig-cosigner default, equally non-BIP-86. H12 only changes the leaf `2' → 3'` for taproot roots.
The structural mis-handling (a single-key taproot reaching a BIP-48-cosigner default in xpub-search
descriptor mode) is **pre-existing**, latent, and orthogonal to H12. H12 does not widen the set of
inputs that reach the default, nor create a new wrong-origin class — it preserves the pre-existing
behavior's shape. (Recorded as Note 1; not in scope for this funds-safety cycle, which targets the
multisig taproot `2'`→`3'` funds bug.)

**Net reachability conclusion:** the `Tr → 3'` blanket is safe. The one site where single-key taproot
can hit it (`descriptor_intake.rs`) was already emitting a `48'`-family default pre-H12, so H12 is not
the origin of any single-key mis-default — it is leaf-correct for the multisig case it targets and
leaf-neutral (no new divergence class) for the pre-existing single-key edge.

---

## Critical
None.

## Important
None.

## Minor

**M-1 — legacy `sh(sortedmulti)` (BIP-45 P2SH, not sh-wsh) now defaults to the `1'` leaf instead of
`2'`.** `canonical_origin` returns `None` for legacy `sh(multi)`/`sh(sortedmulti)` (it only blesses
`sh(wsh(multi/sortedmulti))`, `canonical_origin.rs:65-75`), so such a descriptor reaches
`bip48_script_type_for_root_tag`, where `Tag::Sh → ShWshSortedMulti → 1'`. Empirically confirmed:
`bundle --descriptor "sh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))"` now emits `m/48'/1'/0'/1'`
(pre-H12: `m/48'/1'/0'/2'`). Neither leaf is BIP-45-correct (BIP-45 is `m/45'/...` with no
script-type component), so this is a heuristic best-effort default for an under-specified descriptor in
either case; `1'` is as defensible as `2'` for an `sh`-rooted multisig, the user can always override
with an explicit `[fp/path]@N` / `--slot @N.path=`, and the plan-doc explicitly decided `Sh → 1'`.
It also means legacy `sh(sortedmulti)` and `sh(wsh(sortedmulti))` now collide on the same `1'` default.
This is a default-inference heuristic, not a correctness regression — recorded for completeness; no
action required this cycle. (Optional: a one-line FOLLOWUP noting the `sh`-default heuristic is BIP-48-
biased, if a future cycle wants BIP-45-aware handling.)

## Notes (non-findings)

- **Note 1 (single-key taproot in xpub-search descriptor mode):** as analyzed above, a single-key
  origin-elided `tr(xpub)` in `xpub-search --descriptor` gets a `48'`-family default both before
  (`2'`) and after (`3'`) H12. Pre-existing, out-of-scope; mention for the program backlog if a future
  cycle wants xpub-search to honor BIP-86 single-key origins. NOT introduced by this diff.
- **Note 2 (`tr(@realkey, <ms>)` key-path-spendable):** these reach `is_non_canonical` (tree: Some) and
  get `3'`. Correct — a key-path-spendable taproot with a script tree is a taproot wallet whose
  BIP-48-family leaf is `3'`; `2'` (pre-H12) was wrong. The toolkit accepts this shape (`parse_descriptor`
  `tr(@N, <ms>)`); `3'` is the right default.

---

## Correctness of the 3 sites + helper

- **Helper (`template.rs:296-303`):** `bip48_script_type_for_root_tag(&Tag)` maps
  `Tag::Tr → CliTemplate::TrSortedMultiA`, `Tag::Sh → CliTemplate::ShWshSortedMulti`,
  `_ → CliTemplate::WshSortedMulti`, then reuses `CliTemplate::bip48_script_type()` (the single 1/2/3
  authority, `template.rs:231-238`), `.unwrap_or(2)`. The representatives map to `3'`/`1'`/`2'`
  respectively — verified against `bip48_script_type()` source. **No duplicated 1/2/3 mapping** — it is
  the single-source-of-truth reuse the plan prescribed. `.unwrap_or(2)` is unreachable for the three
  representatives (all multisig → `Some`); the `2` fallback is a defensive default that preserves
  pre-H12 wsh behavior. Correct.
- **`bundle.rs` site (`:1447-1452`, `:2226-2249`):** `compute_default_origin_path` gains a
  `script_type: u32` param; the 4th `PathComponent { hardened: true, value }` is now `script_type`
  (was hardcoded `2`). Caller computes it from `canonicity_probe.tree.tag`. The notice
  (`emit_default_path_notice`, `:2282-2300`) renders the actual `{script_type}'` component (was
  hardcoded `2'`). Correct; tests assert both the JSON `origin_path` and the stderr notice.
- **`verify_bundle.rs:1371-1383` (mirror):** uses the SAME
  `bip48_script_type_for_root_tag(&canonicity_probe.tree.tag)` and passes it to the SAME
  `crate::cmd::bundle::compute_default_origin_path`. Bundle and verify agree by construction — no
  verify/bundle divergence (no H12-crossmode-like gap). Confirmed identical derivation. Correct.
- **`descriptor_intake.rs:303,330,352`:** per-site `matches!(parsed, MsDescriptor::Tr(_)) ? 3 : 2`
  (precedent `bsms.rs:295`, as the plan cites); both default sites (`XPub` and `MultiXPub`) and the
  notice (`:399-411`) consume `default_script_type`. Correct.
- **`Sh → 1'` mapping:** correct for `sh(wsh(sortedmulti))` (the BIP-48 sh-wsh leaf). See M-1 for the
  legacy-`sh(sortedmulti)` collision (Minor).
- **`else → 2'` default:** covers `Tag::Wsh` and any indeterminate root; preserves pre-H12 behavior.
  No missed case masked (the only `is_non_canonical` taproot/sh/wsh roots are covered; everything else
  legitimately defaults to the wsh leaf, as pre-H12).

## Differential proof

- **DEFAULT-CI anti-vacuity leg (`bitcoind_differential.rs::h12_taproot_default_origin_anti_vacuity_leg`):**
  RAN GREEN. For both `tr(NUMS,multi_a)` and `tr(NUMS,sortedmulti_a)`: (a) emitted origin == `m/48'/0'/0'/3'`;
  (b) bundled addresses == INDEPENDENT `48'/0'/0'/3'` derivation; (c) bundled addresses != `48'/0'/0'/2'`;
  plus an anti-vacuity sanity `assert_ne!(addrs_3, addrs_2)`. The DEFAULT == `3'`-derivation /
  diverges-from-`2'` row logic is sound and non-vacuous.
- **Env-gated heavy leg (`bitcoind_h12_taproot_default_origin_differential`):** `#[ignore]`-gated;
  asserts toolkit == Core `deriveaddresses` on `3'` and != Core on `2'`. Structure mirrors the existing
  harness pattern; not executed here (no node), as designed.
- **Re-captured golden 1 (`cli_non_canonical_descriptor.rs`):** the `tr(NUMS)` default-path notice
  assertion updated `m/48'/0'/0'/2'` → `m/48'/0'/0'/3'`. RAN GREEN. Genuinely-correct: the notice now
  reflects the taproot `3'` leaf.
- **Re-captured golden 2 (`DIVERGENT_TR_MULTI_A_CHAIN0_IDX0_GOLDEN`,
  `cli_restore_multisig_general.rs:736`):** changed
  `bc1pjzz9k…` → `bc1p6ufc…`. **Genuinely-correct, NOT made-to-pass:** the generator/anchor
  `divergent_taproot_golden_differs_from_baseline_and_anchors` independently re-derives the address via
  rust-miniscript's own `into_single_descriptors`/`derive_at_index` from the reconstructed `3'`-origin
  descriptor and pins the const (`assert_eq!`). RAN GREEN. The address changed because the cosigner
  origin moved `2'→3'` (different xpub derivation → different P2TR address) — exactly the funds-fix.
  The `assert_ne!` divergence anchor (`<2;3>` alt0 vs baseline `<0;1>` alt0) is orthogonal to the
  script-type and **still holds** (verified GREEN in the same run).

## Mirror agreement / scope

- **`verify_bundle.rs:1373` mirror:** uses the same `Tag`-based detection as bundle (verified above) →
  no new verify/bundle divergence.
- **`emit_multisig_checks` UNTOUCHED (reserved for P4):** confirmed — the diff on `verify_bundle.rs`
  touches only the `descriptor_mode_verify_run` default-origin block (`:1371-1383`); grep for
  `emit_multisig_checks`/`md1_xpub_match`/`policy_match`/`use_site_path ==` in the diff returns
  nothing.
- **`template.rs` helper is justified single-source-of-truth**, not scope-creep: it centralizes the
  `Tag → CliTemplate → bip48_script_type()` mapping so the three call sites (bundle, verify, intake)
  share one authority and the later S-VERIFY dedup subsumes it. No duplicated 1/2/3 mapping anywhere.
- **No new error variant; no flag/CLI-surface change** (`git diff --stat` touches only
  bundle.rs/verify_bundle.rs/descriptor_intake.rs/template.rs + tests) → **no GUI schema-mirror leg,
  no manual leg.** No `error.rs` edit. No fmt/version churn (no `Cargo.toml`/README/version-site
  changes in the diff).

## Verification run summary

- `cargo test -p mnemonic-toolkit --lib` → 158 passed, 0 failed, 3 ignored, 0 filtered (the 4 H12 unit
  tests in `bundle.rs` + `descriptor_intake.rs` compile into the lib and are included in this count;
  confirmed present by source grep).
- `cargo test -p mnemonic-toolkit` (full integration) → every test binary `0 failed`; only
  `#[ignore]`-gated bitcoind heavy legs deferred.
- New `cli_bundle_h12_taproot_origin.rs` → 4 passed (incl. all clean-negatives: `wsh`/`sh(wsh)` NOT
  given `3'`).
- `cli_non_canonical_descriptor` (re-captured notice) → 10 passed.
- `h12_taproot_default_origin_anti_vacuity_leg` → passed.
- `divergent_taproot_golden_differs_from_baseline_and_anchors` → passed.
- `cargo clippy -p mnemonic-toolkit --all-targets` → clean (no warnings/errors).

**GREEN. Proceed.** (M-1 + the two notes are non-blocking; M-1 optionally warrants a one-line FOLLOWUP
on the `sh`-default heuristic, and Note 1 a backlog mention for single-key taproot in xpub-search.)
