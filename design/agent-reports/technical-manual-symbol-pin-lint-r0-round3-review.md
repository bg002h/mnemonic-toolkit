# R0 Architect Review — technical-manual-symbol-pin-lint — Round 3

> Persisted verbatim from the opus `feature-dev:code-architect` agent
> (`agentId: ab05c483e7dd0dcf6`). Verified against source at toolkit `3d9d38e`.
> Persisted BEFORE folding, per CLAUDE.md.

## VERDICT: 1 Critical / 1 Important / 2 Minor — **RED**

The Round-3 edits genuinely close the three Round-2 Criticals at the *design/prose* level (C2 nesting convention + multi-segment G2; C3 colliding-basename rule; the no-auto-pin directive). M1, M2, I2, I3 are all correctly folded. But the no-auto-pin guarantee is **prose-only** — the materialized audit-input artifact still splits 826 into a 637-member queue + a separate 189-member autopin file, and the SPEC names the partial queue while providing no step to unify them or audit the 189. A literal P2 executor re-enters R2 C1. Plus the C3 colliding-basename rule has an unowned blast radius (256 members, the three biggest chapters).

### Critical

**C1 — The "audit EVERY citation, no auto-pin bucket" guarantee is prose-only; the materialized audit input is provably partial (637 of 826), and the SPEC names the partial queue with no step to audit the 189-member autopin remainder. R2 C1 re-enters through the recon artifact.**

The SPEC §3/§4 state "there is no auto-pin bucket: every member is audited" and §6 P2(3a) says "fan-out audit workflow over ALL ~826 refs." But the only materialized enumeration is `cycle-b-audit-queue.tsv`, named in the SPEC header (`SPEC:5`) and in §5 verification. Counts: `cycle-b-autopin.tsv` 189 + `cycle-b-audit-queue.tsv` 637 = 826. `cycle-b-manifest2.py` builds the queue as the *complement* of autopin (line 113 `continue`s autopin out of `audit[]`; line 121 `aud = total − AUTOPIN`; lines 132-135 write the 189 autopin rows to a *separate* file). There is **no SPEC step** to regenerate one unified all-826 queue or retire the split. An executor following §5/§3a literally feeds agents the 637-member queue and **silently skips the 189** — precisely R2 C1, re-entered through the artifact.

Concrete vacuous anchor in the skipped set: `cycle-b-autopin.tsv:2` — `crates/ms-codec/src/decode.rs:50-53` → anchor `decode` (lines 50-53 sit inside fn `decode`; `grep -wqF decode` passes vacuously). 189 >> 50 → **Critical** (R2 C1 reborn).

**Fix:** (a) name a single all-826 enumeration as the §3a audit input (regenerate one unified queue, or explicitly add the 189 to the fan-out); (b) retire the autopin/audit split in the recon tooling so no member can be skipped; (c) §5/§6 P2 reference that unified input. (The "corroborated = one-line confirm" fast-path is fine — but the 189 must enter the workflow.)

### Important

**I1 — The C3 colliding-basename gate rule has an unowned blast radius: it forces repo-qualification on 256 members (concentrated in the three biggest chapters) even where the chapter default already resolves correctly, defeating the chapter-default conciseness mechanism wholesale. The SPEC presents §2 as a narrow C3 fix and never owns this.**

§2's rule: a bare colliding basename "does NOT silently accept the default — it requires the token to be repo-qualified … OR errors." `cycle-b-audit-queue.tsv` carries **256 `collision(...)` members**, concentrated in chapters 51/52/53. Worked example: `bch.rs` exists in three repos (md/mk/ms, NOT toolkit — verified by Glob). Chapter `52-mk-codec-api.md` defaults to `mk` and resolves all **43** bare `bch.rs::` cites (`52-…:189-285`) correctly — yet §2 forces every one to `crates/mk-codec/src/string_layer/bch.rs::…`. Generalized across 256 members (`error.rs`/`lib.rs`/`tlv.rs`/`consts.rs` collisions), this rewrites a large fraction of the three biggest chapters' source-column tokens to full crate-qualified paths, bloating the PDF source column, even where chapter-default is already correct.

Mechanically **sound** (no false-pass; already-qualified paths pass; manifest scopes the rule to `not explicit`, `cycle-b-manifest2.py:97-98`) — so this is a design omission, not a correctness bug: the SPEC frames §2 as a targeted C3 fix and never acknowledges it defeats chapter-default conciseness for ~256 refs. **Fix:** either (a) explicitly accept the blast radius with the count + conciseness trade-off stated, or (b) narrow the rule — require qualification only where the chapter default resolves to the *wrong* repo (the actual C3 case: `derive.rs` in a toolkit-default chapter), letting a correct chapter-default bare basename stand.

### Minor

**M1 — `cycle-b-manifest2.py` still materializes the autopin split it is now meant not to honor** (lines 112-113, 132-135). After C1's fix, if the artifacts are kept as "fast-path ordering hints," say so explicitly and state they are NOT a skip-set; else a future reader re-introduces the skip.

**M2 — G2's bare-`:N` ban and the §2 "repo segment" predicate are prose, not pinned to the checker's regex.** §2 says repo-qualification = "path contains a `crates/<codec>/` or repo segment." The manifest uses `("crates/" in pp) or pp.startswith("src/")` (`:97`). `src/`-prefixed-but-not-`crates/`-qualified paths would satisfy `explicit` but NOT disambiguate the repo — pin the predicate precisely in P1.

### Verified clean (R1+R2 fold status)
- **R2 C1 — fold INTENDED but incomplete → see C1** (prose correct; materialized artifact still splits).
- **R2 C2 (vacuous method anchors) — FOLDED.** §1 four-row nesting table; §2 multi-segment G2. For `tests::fn`, `tests` is vacuous but the fn segment carries discrimination; for `T::method`, both segments must exist (hallucination/rename backstop). Convention forms verified vs source: `mk-codec/src/lib.rs:42` `pub mod test_vectors` + `:44-49` `pub use consts::{…}` (→ bare-file); `json_envelope.rs:473-474` `#[cfg(test)] mod tests` (→ `tests::fn`). Sound.
- **R2 C3 — FOLDED for correctness** (residual is scope, I1). Wrong-repo `derive.rs` now caught; no false-pass class remains.
- **R2 I1 (tail-leak) — FOLDED** (manifest v2 routes all `idx>0` to audit, `:107,110-111`; queue carries `tail->…` rows).
- **R2 I2 (figures-cache) — FOLDED** (empirically settled in §3; P2 confirms `figures-cache-verify` GREEN).
- **R2 I3 (QA bucket) — MOOT** correctly (no auto-pin bucket; §3b re-verifies every anchor-differs-from-guess ref).
- **R2 M1 (synthesize boundaries) — FOLDED, VERIFIED EXACT:** `synthesize_descriptor` 228-318, `derive_xpub_at_path` 323, `synthesize_multisig_full` 343, `synthesize_multisig_watch_only` 488, `synthesize_unified` 744; line 593 `// 7. Policy id + stubs.` inside multisig_watch_only.
- **R2 M2 (six→seven + false-CI) — FOLDED, grep-VERIFIED:** `AUTHORING.md:181-183` enumeration, `:185` "CI runs all six" (genuinely false — no technical-manual.yml), `Makefile:13`, `lint.sh:10-16` — all four sites + false-CI correction in P1.
- **R1 findings + SemVer no-bump/no-tag + no-lockstep** re-confirmed.

**Bottom line:** Forward gate, nesting convention, colliding-basename correctness fix, and audit-all intent are ship-worthy. RED = (C1) no-auto-pin is prose-only; materialized input is 637/826 with a 189-member skip and a concrete vacuous anchor (`ms-codec/src/decode.rs:50-53 → decode`); (I1) collision rule has an unowned 256-member blast radius. Fix both — single all-826 audit input + retire split (C1); own-or-narrow collision rule with the count (I1) — then re-dispatch Round 4.
