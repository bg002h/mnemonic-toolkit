# cycle-prep recon — 2026-06-12 — seven-fragment render tests (GAP 2)

Repo: descriptor-mnemonic (md-codec) `origin/main` = `422b049` (working tree clean, `main...origin/main`).
Companion: mnemonic-toolkit `origin/master` = `ca7d7bc`.
md-codec version in tree: 0.35.2. Workspace miniscript pin: crates.io **13.0.0** (`Cargo.toml:18`) — all experiments below ran against this exact pin.

## Verification

### 1. The 7 arms — all ACCURATE at the cited lines (`crates/md-codec/src/to_miniscript.rs`, fn `node_to_miniscript`)

| Tag | Line | Render logic (quoted) | Status |
|---|---|---|---|
| `DupIf` (`d:`) | :337-340 | `arity_eq(…, 1)?; Terminal::DupIf(Arc::new(node_to_miniscript::<Ctx>(&children[0], keys)?))` | ACCURATE |
| `NonZero` (`j:`) | :341-344 | same shape → `Terminal::NonZero(…)` | ACCURATE |
| `ZeroNotEqual` (`n:`) | :345-348 | same shape → `Terminal::ZeroNotEqual(…)` | ACCURATE |
| `OrB` | :368-373 | `arity_eq(…, 2)?; Terminal::OrB(Arc::new(l), Arc::new(r))` | ACCURATE |
| `OrC` | :374-379 | `arity_eq(…, 2)?; Terminal::OrC(Arc::new(l), Arc::new(r))` | ACCURATE |
| `False` | :458 | `(Tag::False, Body::Empty) => Terminal::False` | ACCURATE |
| `True` | :459 | `(Tag::True, Body::Empty) => Terminal::True` | ACCURATE |

All 7 funnel into `Miniscript::from_ast(term)` at :478 (rust-miniscript typechecks there; no `sanity_check` call anywhere in the file).

### 2. Test-surface membership — all claims CONFIRMED

- `to_miniscript.rs` has **no `#[cfg(test)] mod tests`** (grep: zero hits; 538 lines).
- The tag lists live in `tests/proptest_to_miniscript.rs`, NOT `tests/common/mod.rs` (minor brief drift):
  - `W_TARGET_TAGS: [Tag; 34]` at :727 — **contains all 7** (`True` :745, `False` :746, `DupIf` :751, `NonZero` :752, `ZeroNotEqual` :753, `OrB` :757, `OrC` :758).
  - `T_TARGET_TAGS: [Tag; 23]` at :765 ("All to_miniscript-supported tags the typed grammar emits") — **omits all 7** (also omits `Check`, `SortedMulti`, `SortedMultiA`, `RawPkH`, which have other deterministic coverage).
  - `w_generator_covers_all_fragments` (:811) loops `W_TARGET_TAGS` asserting `tags.contains(&t)` (:829-831) → wire-layer generation of all 7 IS anti-vacuity-enforced. `t_generator_covers_all_fragments` (:844) asserts only the 23.
- The W strategy feeds ONLY wire properties (P1/P2/P4/P5 in `proptest_roundtrip.rs` — no `to_miniscript` call there, grep-verified). The render leg P6 runs **only over the T strategy** (`p6_typed_to_miniscript_round_trip` :481). P7's deterministic refusal classes are SortedMultiA/RawPkH/SortedMulti-under-combinator/shape-C/bad-timelocks/oversize-multi — none of the 7.

### 3. Existing-cell sweep — all 7 genuinely have ZERO render-layer execution

- Deterministic P6 golden cells (`self_test_*`, proptest_to_miniscript.rs:135-308) cover AndV/Older, AndOr, Tr+Sha256, older-leniency ×2, Thresh/PkH/Swap, AndB/Alt, Sh/OrD/Multi — **none of the 7**.
- Fragment-string grep (`or_b(`, `or_c(`, `dv:`, `j:`, `n:`, `true()`, `false()`, case-insensitive variants) across `tests/` + `src/` + `tests/vectors/`: zero functional hits outside the generator tag lists.
- `tests/address_derivation.rs` (the other `to_miniscript` caller, via `derive_address`): tags used are PkK/PkH/Multi/Tr/TapTree/AndV/Verify/Older/Thresh/Swap/OrI — **zero of the 7** (grep count 0).
- `Tag::True`/`Tag::False` appear outside the render path only in `tests/common/mod.rs:495-496` (W leaf pool) and `src/tag.rs`/`src/tree.rs` (wire codec arms — wire layer is covered, as briefed).
- Conclusion: the 7 arms are not merely golden-less — they are **never executed at all** in the current suite. A mis-render (e.g. swapped `OrB`/`OrC` children, `True`/`False` transposed) would ship silently. 7/7 genuinely untested.

### 4. Constructibility — EMPIRICAL: all 7 pass the full P6 chain (experiment run 2026-06-12)

Scratch integration test (built with the repo's own `common::{descriptor_with_pubkeys, canon, …}` helpers, run against miniscript 13.0.0, then deleted) put one hand-built typed `Node` tree per fragment through encode→decode wire round-trip + `to_miniscript_descriptor` + `Descriptor::from_str` reparse fixed-point + mainnet `derive_address`. **All 7 passed**:

| Fragment | Node tree (typed) | Rendered (key elided) | Derived address (candidate golden) |
|---|---|---|---|
| `OrB` | `wsh(or_b(pk(@0), s:pk(@1)))` | `wsh(or_b(pk(…),s:pk(…)))#ywq2et2d` | `bc1q2epc9vj8hy2mzmh9uyaz9adhp4q9yvu0aygw2httyngv6c7ct5wseumd3l` |
| `OrC`+`True` | `wsh(and_v(or_c(pk(@0), v:pk(@1)), TRUE))` | `wsh(t:or_c(pk(…),v:pk(…)))#4hddhe69` | `bc1qh3wd6a5nn5ccgqg4hj7aj7mjtgwc39gjakyen2m25my4fx3vdx0q9nhznw` |
| `DupIf` | `wsh(or_i(pk(@0), DupIf(Verify(older(144)))))` | `wsh(or_i(pk(…),dv:older(144)))#xtal5qj5` | `bc1qre28e06mc7r8fyam0my2uegn096eygzvx52v9jzg07avehev3lws5nf5qc` |
| `NonZero` | `wsh(NonZero(pk(@0)))` | `wsh(j:pk(…))#qlzgpwq4` | `bc1qewdar8ze6tynzushrg7fmnedlw4xm6q7vj6tmcey2kalwryy7ens08c3x0` |
| `ZeroNotEqual` | `wsh(or_i(pk(@0), ZNE(and_v(v:pk(@1), older(144)))))` | `wsh(or_i(pk(…),n:and_v(v:pk(…),older(144))))#qkdzs9qu` | `bc1qjnwlx28qetpwdp3wfrv3emhhpc3dc2cya75qy3gzqfmzmn2ea36q8fx4y4` |
| `False` | `wsh(or_i(pk(@0), FALSE))` | `wsh(u:pk(…))#mjt9mr4g` | `bc1qly5mzr0gwyquj5jwllans468wnmwc89u27sf9xnqjcldswqwcdxsfms2dk` |
| `True` (standalone) | `wsh(and_v(v:pk(@0), TRUE))` | `wsh(tv:pk(…))#9f6svydl` | `bc1q779rp8l2cy6v63ea5elayzeryqguxq89ez7rh3w8ajx6pgf33p7qanlqr5` |

(Keys = the standard abandon-mnemonic test-xpub pool via `descriptor_with_pubkeys`. Addresses derived once in this run; the implementing cycle should prefix-verify per the existing golden-cell discipline — independence is anchored by the reparse fixed-point, exactly as the existing `self_test_*` cells document.)

Notable Display behavior (goldens must pin these forms): rust-miniscript SUGARS on render — `or_i(X,false)` Displays as `u:X` (no literal `0` in the string), `and_v(X,1)` as `t:X`, wrapper chains fuse (`dv:older`, `tv:pk`). The reparse fixed-point (AST `PartialEq`) holds through all of it, so the sugar costs nothing and the address pins close the loop.

**Verdict on the T-exclusion: (a) generator scope, NOT (b) type-construction difficulty.** The R4-GREEN spec (`design/BRAINSTORM_proptest_fragment_domain_expansion.md`) defines the T grammar as an explicit production list verified in rounds 1-3; the 7 were simply never in the list. The spec itself notes wsh/sh reparse is NOT sanity-checked ("sigless wsh shapes are legitimate extra coverage and are kept") — only tr() carries the from_str sanity branch. All 7 are type-validly hostable under `wsh`, empirically proven above. Zero P7-only fragments.

### 5. The deleted-coverage regression — CONFIRMED real, not a duplicate

- `crates/md-codec/src/bytecode/hand_ast_coverage.rs` (637 lines) was **deleted at commit `5350f8a` "release: md-codec v0.12.0 — strip v0.x, flatten v11"** (brief said "pre-v0.30" — it was the v0.12.0 strip; same conclusion). It carried hand-AST byte-form pins for `or_c` (×2 incl. `t:or_c` round-trip), `d:v:older(144)`, `j:`, `n:c:pk_k`, plus `True`/`False` as decoder-arm payload bytes.
- Caveat on what was lost: those were **bytecode-layer** pins (hand-built `Terminal` → wire bytes, v0.x encode direction), not render-layer (`Node` → `Terminal`) tests — so the render layer was arguably never covered. But they were the only hand-pins of these fragments anywhere, and nothing was migrated (current-tree sweep above: zero).
- Dangling cross-refs: `design/FOLLOWUPS.md` ~:920-929 still marks the `d:`-wrapper and `or_c` corpus-pin entries **resolved** pointing at the deleted file — those resolutions are now vacuous. The new cycle re-grounds them.
- **No FOLLOWUP exists** for the render-layer gap in either repo's `FOLLOWUPS.md` (grep-verified both) — the brief's "NO tracking FOLLOWUP" claim is ACCURATE.

### 6. Cross-repo note (toolkit)

The toolkit's `parse_descriptor.rs` walker is the OPPOSITE direction (descriptor string → Node) and is largely covered: per-arm tests exist for False (:2379), True (:2386), NonZero (:2426), ZeroNotEqual (:2436), OrB (:2478), OrC (:2485). But `arm_dup_if` (:2420) is `#[ignore = "DupIf descriptor-unreachable in rust-miniscript v13 — every d: example in ms_tests.rs is invalid_ms"]` — **that claim is DISPROVEN by this recon's experiment**: `wsh(or_i(pk(X),dv:older(144)))` parses via `Descriptor::from_str` on the same miniscript 13.0.0. The ignore reason conflated upstream's `ms_tests.rs` corpus with actual reachability. Cheap toolkit companion: de-ignore `arm_dup_if` with the `dv:older(144)` shape (1 test, NO-BUMP).

## Assessment

- **P6-testable vs P7-only split: 7/7 P6-testable with goldens, 0 P7-only.** Every fragment hosts in a valid `wsh` typed descriptor that renders, reparses to a fixed point, and derives an address. No fragment needs the wire-valid-but-type-invalid escape hatch.
- **No render bug found** in the representative shapes — the reparse oracle agreed on all 7, so this cycle is expected pure test-add (no PATCH). The risk being closed is future silent drift, plus any mis-render in shapes the experiment didn't try (the deterministic cells + optional grammar extension cover that).
- **Cycle size: SMALL.** Core deliverable = **7 deterministic golden cells** (mirroring the existing `self_test_*` house style: hand-built Node tree → full `p6_chain` → pinned rendered-string fragment + golden address literal). The experiment above is effectively the cells' first draft — descriptors and candidate goldens are already in hand.
  - Optional (recommended, still small): extend `t_segwit_tree` with one production per fragment (the proven shapes), add the 7 tags to `T_TARGET_TAGS` (23 → 30) so `t_generator_covers_all_fragments` anti-vacuity-enforces them under the P6 property forever. wsh-only (tap is sanity-gated; these shapes are mostly sigless-branch). This converts one-shot goldens into permanent property coverage and is the option that actually dissolves the gap class.
  - Optional toolkit companion: de-ignore `arm_dup_if` (§6).
- **Tier: test-only NO-BUMP, md-codec-local.** SemVer: none (unless a render bug surfaces during cell bring-up → PATCH, not expected). Cross-repo lockstep: none required; toolkit companion is independent and optional.
- **FOLLOWUP first?** Not needed if the cycle proceeds now — it's small, fully scoped by this recon, and filing-then-immediately-resolving is ceremony. If the cycle is NOT picked up this session, file `md-codec-seven-render-arms-untested` in descriptor-mnemonic `design/FOLLOWUPS.md` (+ toolkit companion line for the `arm_dup_if` de-ignore) so the gap stops being tracking-less.

## Recommended scope

**Verdict: GO — straight to a small md-codec test-only cycle (NO-BUMP), no FOLLOWUP detour needed if executed now.**

1. 7 deterministic P6 golden cells in `tests/proptest_to_miniscript.rs` (`self_test_wsh_or_b_pk_s_pk`, `self_test_wsh_t_or_c_true`, `self_test_wsh_or_i_dupif_v_older`, `self_test_wsh_nonzero_pk`, `self_test_wsh_or_i_zne_and_v`, `self_test_wsh_or_i_false_u_sugar`, `self_test_wsh_and_v_true_t_sugar`) — descriptors + candidate goldens per the table above, prefix-verified at implementation time.
2. Extend the T grammar (`t_segwit_tree`) with the 7 proven productions + grow `T_TARGET_TAGS` to 30 — permanent anti-vacuity + property coverage.
3. Re-ground the two dangling resolved FOLLOWUP entries (~:920-929) to cite the new cells instead of the deleted `hand_ast_coverage.rs`.
4. Optional paired toolkit micro-PR: de-ignore `parse_descriptor.rs::arm_dup_if` using `wsh(or_i(pk(@0/<0;1>/*),dv:older(144)))`.

Per CLAUDE.md the cycle still needs a brainstorm-lite spec + mandatory R0 to 0C/0I before code; this recon is the spec's evidence base (constructibility table = the round-1 evidence log equivalent).
