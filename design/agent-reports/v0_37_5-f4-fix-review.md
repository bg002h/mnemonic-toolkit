# v0.37.5 — F4 fix (elided-origin PathDecl::Divergent→Shared collapse) — architect review (retroactive R0)

**Context:** F4 was surfaced by cross-start convergence cell A8 (non-canonical `wsh(andor)` descriptor ≡ BSMS wallet-file): `bundle --descriptor` with elided origins emitted `PathDecl::Divergent([p,p,p])` for identical inferred paths, while the explicit-origin / wallet-import path emitted `PathDecl::Shared(p)` — byte-different md1 for the same wallet. The user chose "fix now" after a deeper investigation. This document is the persisted architect review (retroactive R0 per CLAUDE.md) of the fix, before tagging v0.37.5. Reviewer: feature-dev:code-reviewer (opus). Base `bde8267` (v0.37.4).

## Verdict
**The F4 fix is correct, SPEC-aligned, and safe to ship.** Right direction (not a mask). No SPEC clause or test requires `Divergent` for a uniform-path elided-origin descriptor — including `--account != 0`.

## Critical
None.

## Important
**I1 — Phase-6 release prep.** At review time the version was still `0.37.4` everywhere and there was no `0.37.5` CHANGELOG entry — a tagged PATCH requires Cargo.toml + Cargo.lock + both README markers + CHANGELOG + install.sh pin bumped to 0.37.5. **Resolved during ship**: all Phase-6 items bumped to 0.37.5; CHANGELOG `[0.37.5]` added.

## Minor
**M1 — verify-bundle symmetric inference still hardcoded `Divergent` (`verify_bundle.rs:683-684`).** Benign today (the only md1 comparison, `md1_xpub_match`, is pubkey-multiset and never inspects `path_decl`), but the two mirrored default-inference sites had diverged — a drift hazard if md1 comparison is ever tightened. **Resolved during ship**: applied the same `all_same → Shared` collapse at `verify_bundle.rs:683-684`, keeping the two sites symmetric.

**SPEC clarification.** `SPEC_mnemonic_toolkit_v0_2.md:53` ("non-default `--account` → Divergent") could be misread as unconditional. **Resolved during ship**: added a clarifying note — `Divergent` arises only when cosigners' accounts/paths *differ*; a uniform non-zero account stays `Shared` per §4.2.

## SPEC alignment (the key question)
The authoritative synthesis rules are unanimous that `Divergent` means **distinct** paths only: `SPEC_v0_2.md:236-237` (§4.2: `Shared` if all cosigners share an origin path; `Divergent` only if they diverge), `:271`/`:284` (§4.6/§4.6.2), `SPEC_v0_1.md:344` (v0.2 adds `Divergent` "when cosigners use **distinct** paths"). `SPEC_v0_2.md:53` defers "(per §4.6 delta)" and `:137` clarifies the trigger as *differing* accounts. The §4.12.b elided-origin inference assigns the *same* `m/48'/<coin>'/<account>'/2'` to every placeholder, so §4.2's "all share → Shared" governs. `--account 5` uniform → `Shared(m/48'/0'/5'/2')` is correct.

## Correctness / blast radius
- `new_paths` always non-empty (`n ≥ 1`); 1-element vec → `windows(2)` empty → `all()` true → `Shared`. ✓
- Flows into md1 via `bundle.rs` `descriptor.path_decl.paths = resolved_placeholders…clone()` → `synthesize_descriptor` → `md_codec::chunk::split`. ✓
- Mirrors `parse_descriptor.rs:202` + `synthesize.rs:674-679` exactly; convergence target (import/explicit-origin) already emits `Shared` → fix moves elided path toward canonical form (not a mask). A8 is the regression guard.
- **Blast radius limited to elided-origin uniform-path descriptor-mode bundles.** Canonical templates never enter the F4 block; inline-origin + wallet-import already emit `Shared`. Existing `cli_non_canonical_descriptor` / `cli_descriptor_mode` tests assert path *strings* + info-notices, not the Shared/Divergent shape or exact md1 bytes; no test asserts `Divergent` for a uniform case. Full suite 2437/0 consistent.
- **GUI schema-mirror: NOT tripped** (function-body change only; no `BundleArgs` clap field change). No paired GUI PR.

## Ship recommendation
Ship as PATCH **v0.37.5** with the B1–B6 bijection tests + A8 (now green) + Phase-6 + the M1 collapse + the SPEC §4.2:53 clarifying note. No Critical/Important blocker in the F4 logic.
