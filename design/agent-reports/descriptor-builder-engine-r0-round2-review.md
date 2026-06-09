<!-- VERBATIM opus-architect R0 review, round 2, descriptor-builder engine (Release A, v0.50.0). Persisted per CLAUDE.md convention. SPEC @ post-round-1-fold; source SHA b596d3f, miniscript 95fdd1c. Verdict: GREEN 0C/0I — cleared for implementation. -->

# R0 REVIEW (round 2) — descriptor-builder engine (Release A, v0.50.0) — **GREEN**

**Verdict: GREEN. 0 Critical / 0 Important.** The SPEC is cleared for implementation. All three round-1 Criticals and both Importants are correctly folded and verified against live source; the five minors are addressed. I scanned adversarially for fold-induced drift (focus areas 1–6) and found none at Critical/Important severity. Two Minors below are polish-only and explicitly non-blocking.

Verification basis: HEAD `b596d3f` (matches SPEC front-matter); miniscript git rev `95fdd1c` at the pinned checkout; `ExtParams::insane()` / `sanity_check` gated on `std`, compiler feature absent.

---

## What passes (the fold verification — stated for credibility)

**C1 (emit no longer cites `build_descriptor_string`) — CLOSED.** `build_descriptor_string` is confirmed a hardcoded `match template { Bip44 => "pkh(..)", … }` over `CliTemplate` with no recursion (`wallet_export/pipeline.rs:86-104`); it cannot render `wsh(andor(...))`. The SPEC §0 diagram + §4 bullet 1 now correctly render `PolicyNode→String` (recursive `Display`, §1) and canonicalize via the genuine 2-line idiom `MsDescriptor::from_str(&rendered)?.to_string()` at `pipeline.rs:28-30`. The bip388 reuse is correctly kept and confirmed shape-general: `descriptor_to_bip388_wallet_policy` at `pipeline.rs:166` is string-based (`iter_pk` + longest-first `replacen` `:199-203`), with the multipath guard `:171` and `/<0;1>/*` suffix requirement `:218`. The recon's "~60% reuse" over-count is corrected in-SPEC.

**C2 (node diagnostics via per-subtree re-check, NOT `sanity_check`) — CLOSED and SOUND.** I scrutinized the locality claim adversarially, which is the load-bearing keystone:
- `sanity_check()` confirmed a payload-less short-circuit if/else returning a single first-failure variant in fixed priority (`analyzable.rs:225-239`, priority Sigless→Malleable→ResourceLimits→RepeatedPubkeys→HeightTimelock) with zero sub-location — so §3 correctly uses it only as the fast all-clear gate.
- The five predicates are all `pub fn(&self) -> bool` (`analyzable.rs:187-208`).
- **`has_mixed_timelocks` is genuinely local** (this was the highest-risk claim). It reads `self.ext.timelock_info.contains_unspendable_path()` → `self.contains_combination` (`extra_props.rs:42`). `TimelockInfo` is computed **purely bottom-up** via `combine_and`/`combine_or`/`combine_threshold` (`extra_props.rs:45-86`) from a node's own children — it never reads parent context, so a standalone-parsed subtree gets the **identical** `TimelockInfo` it has when embedded. Moreover `contains_combination` is monotone-upward (the `|=` propagation at `:83`), so walking deepest-first and reporting the minimal failing subtree correctly lands on the nearest-common-ancestor `k>1` node where the height/time conflict first combines — exactly §3.4's stated semantics.
- **`within_resource_limits` is local** in the sense the mechanism needs: `Ctx::check_local_validity` (`context.rs:858-866` for Segwitv0) accounts over the subtree-as-its-own-Miniscript; a subtree's cost is a subset of the whole tree's, so the per-subtree predicate identifies the minimal subtree that itself exceeds limits, and a root-only cumulative overflow correctly localizes to the root. Matches "minimal failing subtree."
- The **B-type restriction** is verified at `mod.rs:848-849`: `from_str_ext` returns `Error::NonTopLevel` when `ms.ty.corr.base != Base::B`. The two-stage walk is implementable: step-2 type-only localization parses with `ExtParams::insane()` (confirmed `analyzable.rs:63-72`, all 5 sanity toggles `true`, `raw_pkh:false`) to isolate pure type/parse errors; step-3 re-runs the individual predicates on parsed subtrees. All v1 leaf nodes (`older`, `after`, `multi`, `sortedmulti`, `pk`, `pkh`, `sha256`, …) standalone-parse as B; only explicitly-wrapped (`v:`/`s:`/`a:`) children are non-B and correctly skip to the nearest B ancestor. The precision-floor framing is honest, not a soundness gap.

**C3 (`--allow` cut) — CLOSED.** §3.5 defers it cleanly to a FOLLOWUP mapping `--allow` → `ExtParams` toggles at parse stage; raw `--descriptor` is the escape hatch. The empirical "all 5 archetypes sane-parse → cut blocks nothing" is consistent with the verified `sanity_check` priority and the `andor(multi,older,…and_v(v:pk,after))` shape (an `older`+`after` that does not trip `HeightTimelockCombination` because the conflict rule fires only on same-kind height/time pairs under `k>1`).

**I1 (no-auto-wrap ruled strict; §6 typo) — CLOSED.** Strictly-explicit is ruled (§1); the GUI claim is softened to "validated emit substrate the GUI's wrapper-inference targets." §6 decaying-multisig is corrected to `and_v(v:pk,after)`. The "explicit `v:` is authorable" backstop is grounded — the corpus authors it directly: `or_d(pk(A),and_v(v:pk(B),older(144)))` (`cli_compare_cost.rs:439`, `:785`), `and_v(v:pk(A),sha256(H))` (`:836`).

**I2 (multipath projection) — CLOSED.** `strip::translate_descriptor` calls `derive_at_index(0)` on `has_wildcard()` (`strip.rs:26-28`); `derive_at_index` errors on multipath (doc condition `mod.rs:705`, fn `:706`); `into_single_descriptors()` exists at `mod.rs:946` and returns `Vec<Descriptor<DescriptorPublicKey>>`, so the §4 "split → enumerate over `[0]`, cost path-invariant" projection is well-defined.

**Minors — addressed.** Phasing re-sequenced (inputs P1 / goldens P3, §6+§9); `ContainsRawPkh` noted unreachable and asserted (§3 step 3 — verified: the `pkh` node carries a key → renders `pkh(<key>)` type B, never `Terminal::RawPkH`); the upstream typo `BranchExceedResouceLimits` preserved verbatim (`analyzable.rs:139`); Cargo.toml pin disambiguated (root `:17` rev-pin vs crate `:35` version line); `--format` committed to `descriptor`+`bip388` (§2). Build-time cap inputs (`enumerate.rs:111-121`) and the `BTreeSet<DescriptorPublicKey>` dedup caveat (`enumerate.rs:374-391`) are IR-computable and correctly cited.

## MINOR (polish only — do NOT block implementation; fold opportunistically in Phase 2/3)

- **M1 — `into_single_descriptors` dedup granularity, make explicit in Phase 3.** §3 step 5 says the IR cap must dedup keys "identically" to enumerate's `BTreeSet<DescriptorPublicKey>`. Enumerate dedups on the *translated* `DescriptorPublicKey` (full key-expression: origin + xpub + `/<0;1>/*` suffix), via `for_each_key` over the parsed descriptor (`enumerate.rs:376-391`), not the bare xpub. The implementer must dedup on the full rendered key expression, not the IR's bare `<key>` string, or a tree reusing one xpub at two origins/paths will count differently in the cap vs the actual enumeration. The SPEC wording is correct; this is an implementation note to surface in the Phase-3 test (a fixture with the same xpub under two origins would discriminate).

- **M2 — §3 step 5 / §4 line-range citations are ±1–2, harmless.** `enumerate.rs:115-120` (cap refusal) — the actual `return Err(...ConditionsTooMany)` is `:120` and the `checked_pow` guard `:113-115`; range is within bounds. `derive_at_index` "errors on multipath" cites `mod.rs:705` (the doc-comment condition line; the fn signature is `:706`). Neither is structurally wrong; tighten on the next grep-verify pass if convenient.

---

The architecture is sound, the three load-bearing reuse/locality claims that grounded the round-1 RED are now mechanically correct against the pinned source, and the per-subtree localization keystone is verified implementable (bottom-up `TimelockInfo` locality + monotone-upward `contains_combination` + the `ExtParams::insane()` two-stage walk). **GREEN — cleared for Phase-1 implementation.**

Files central to implementation (all absolute):
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/wallet_export/pipeline.rs` (`:28-30` canonicalize idiom; `:86-104` template coupling proving C1; `:166` shape-general bip388)
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cost/enumerate.rs` (`:111-121` cap; `:258` plan; `:374-391` key-dedup)
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cost/strip.rs` (`:26-28` `derive_at_index` multipath failure point)
- `/scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cost/translate.rs` (`:81-83` dual-context parse)
- `/home/bcg/.cargo/git/checkouts/rust-miniscript-ce5fa57e8900265e/95fdd1c/src/miniscript/analyzable.rs` (`:63-72` `insane()`; `:187-208` predicates; `:225-239` `sanity_check`)
- `/home/bcg/.cargo/git/checkouts/rust-miniscript-ce5fa57e8900265e/95fdd1c/src/miniscript/types/extra_props.rs` (`:42-86` `TimelockInfo` bottom-up locality — the C2 soundness anchor)
- `/home/bcg/.cargo/git/checkouts/rust-miniscript-ce5fa57e8900265e/95fdd1c/src/miniscript/mod.rs` (`:848-849` B-type restriction; `:946` `into_single_descriptors`)
