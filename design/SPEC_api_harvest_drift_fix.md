# SPEC — technical-manual synthesize/distinctness drift audit (api-harvest-drift-on-synthesize-descriptor-signature)

**FOLLOWUP:** `api-harvest-drift-on-synthesize-descriptor-signature` (the literal "synthesize_descriptor signature" is the tip of a broad drift).
**Source SHA:** `cacd4a8` (post-v0.47.4). **User decision (2026-06-06):** FULL audit now.
**Cycle type:** docs-only in `docs/technical-manual/` (separate cadence, NOT CI-wired). **NO toolkit version bump, NO tag.** Plain commit to `master`.
**Locksteps:** `make -C docs/technical-manual lint` GREEN. No schema_mirror / sibling / docs/manual.

---

## 1. Verified CURRENT source facts (all re-grepped at `cacd4a8`)

**Symbols / line numbers (synthesize.rs unless noted):**
`Bundle` struct `:22-30` · `Bundle::any_secret_bearing` `:35` · `derive_mk1_chunk_set_id` `:44` (packing `:45`) · `xpub_to_65` `:98-103` · `build_descriptor` `:109` · `synthesize_full` `:142` · `synthesize_watch_only` `:181` · `CosignerKeyInfo` (`pub type … = ResolvedSlot`) `:219` · `synthesize_descriptor` `:229` · `synthesize_multisig_full` `:344` · `synthesize_multisig_watch_only` `:489` · `ResolvedSlot` struct `:642-686` · `ResolvedSlot::origin_path_bare` `:705` · `ResolvedSlot::bracketed_origin` `:720` · `ResolvedSlot::is_secret_bearing` `:690` · `synthesize_unified` `:745-827`.

**Signatures (the 2 with wrong arg-lists):**
- `synthesize_descriptor(descriptor, cosigners, privacy_preserving, run_language)` — 4-arg (NOT the doc's `…, entropy, privacy_preserving`, NOT the FOLLOWUP's "3-arg").
- `synthesize_unified(slots, template, threshold, network, privacy_preserving, run_language)` — 6-arg (doc missing `run_language`).

**`ResolvedSlot` fields:** `xpub, fingerprint, path: DerivationPath, …, entropy: Option<zeroize::Zeroizing<Vec<u8>>>, master_xpub: Option<Xpub>`. **NO `path_raw`** (deleted v0.37.9; replaced by typed `path` + the `origin_path_bare()`/`bracketed_origin()` accessors).

**BIP-388 distinctness — BOTH layers now use TYPED `DerivationPath` equality (no bifurcation):**
- CLI layer `cmd::bundle::check_resolved_slots_distinctness` `bundle.rs:429` compares `xpub.to_string() == … && slots[i].path == slots[j].path` (TYPED). Its source doc-comment (`bundle.rs:423-428`) ALREADY says "TYPED `DerivationPath` … `h`/`'` folds … converges with the descriptor-mode twin" (v0.5 §4.11.b deliberate-reversal / `SPEC_path_raw_bracketed_bare_unification.md` A2).
- Descriptor layer `parse_descriptor::check_key_vector_distinctness` `:1208` compares `cs[i].path == cs[j].path` (TYPED).
- **Only remaining source-comment lag:** `error.rs:15` (the `Bip388Distinctness` doc) STILL says "`(xpub, derivation_path_string)` raw-string equality" — a real source bug (the behavior is typed). → code FOLLOWUP (§2e).

**`schema_version: "4"` construction sites (7 total):** `synthesize.rs:1732`, `cmd/bundle.rs:906`, `cmd/import_wallet.rs:1499`, `cmd/verify_bundle.rs:329`, `cmd/verify_bundle.rs:1017`, `wallet_import/json_envelope.rs:595`, `wallet_import/json_envelope.rs:761`. (Doc cites `synthesize.rs:1296` + `cmd/bundle.rs:572` — both WRONG + incomplete.)

**`verify_bundle::run` `:143`** (doc cites `:98`).

## 2. Changes — exact edits

### 2a. `transcripts/api-harvest-mnemonic-toolkit.md` (unrendered scaffolding; FIX in full)
- `:133` + `:427`: `synthesize.rs:1296` → `synthesize.rs:1732`; `cmd/bundle.rs:572` → `cmd/bundle.rs:906` (and note "+5 more sites" or list all 7 — at minimum correct the 2 wrong refs).
- `:257` `xpub_to_65 :69`→`:98`; `:258` `build_descriptor :80`→`:109`; `:259` `synthesize_full :113`→`:142`; `:260` `synthesize_watch_only :152`→`:181`.
- `:261` `synthesize_descriptor(descriptor, cosigners, entropy, privacy_preserving) :196` → `synthesize_descriptor(descriptor, cosigners, privacy_preserving, run_language) :229`.
- `:262` `synthesize_multisig_full :288`→`:344`; `:263` `synthesize_multisig_watch_only :413`→`:489`.
- `:264` `synthesize_unified(slots, template, threshold, network, privacy_preserving) :593` → `…, run_language) :745`.
- `:268` `Bundle :20`→`:22`; `:273` `ResolvedSlot :569`→`:642`; `:277` DROP `pub path_raw: String` line (deleted); `:278` `entropy: Option<Vec<u8>>`→`Option<zeroize::Zeroizing<Vec<u8>>>`; `:279` `is_secret_bearing :579`→`:690`; `:280` `CosignerKeyInfo :190`→`:219`.
- `:469` (distinctness footnote): rewrite — `check_resolved_slots_distinctness` now uses TYPED `path` equality (its bundle.rs comment was updated); the ONLY remaining lag is `error.rs:15` (still raw-string). Correct the `bundle.rs:259-260`→`bundle.rs:423-429`, `error.rs:68-71`→`error.rs:13-16`, `parse_descriptor.rs:1104/1108`→`:1208/:1212` refs.

### 2b. `src/50-rust-api/54-mnemonic-toolkit-api.md` (RENDERED)
- `:56` prose: `synthesize.rs:593`→`:745`; lift `synthesize_descriptor` OUT of the "`#[allow(dead_code)]` legacy variants" list — it is the LIVE v0.47.1 delegation target of `synthesize_unified` (`synthesize.rs:826`); keep `synthesize_full`/`synthesize_watch_only`/`synthesize_multisig_*` as the dead group. ("the CLI no longer calls them directly" stays true.)
- table `:60` `Bundle :20`→`:22`; `:61` `any_secret_bearing :33`→`:35`; `:62` `ResolvedSlot … path_raw … :569` → drop `path_raw`, list `xpub, fingerprint, typed path, entropy, master_xpub`, `:642`; `:63` `is_secret_bearing :579`→`:690`; `:64` `CosignerKeyInfo :190`→`:219`; `:65` `xpub_to_65 :69`→`:98`; `:66` `build_descriptor :80`→`:109`; `:67` `synthesize_unified :593`→`:745`; `:68` split `synthesize_descriptor` out of the dead-variants row.
- `:182` + `:737` schema_version site list: `synthesize.rs:1296`→`:1732`, `cmd/bundle.rs:572`→`:906`; note 7 sites total (add import_wallet/json_envelope or say "and 5 further sites").
- `:738` distinctness paragraph: `check_resolved_slots_distinctness` now uses TYPED `path` equality (NOT raw-string `path_raw`); correct `parse_descriptor.rs:1104/1108`→`:1208/:1212`; the bundle.rs comment is fixed; only `error.rs` (now `:13-16`) retains the raw-string lag.
- **(R0-r2 I1) `:72` prose:** "`cmd::bundle::check_resolved_slots_distinctness` … uses raw-string equality" → "… uses typed `DerivationPath ==` equality (converges with the descriptor-mode twin per the v0.5 §4.11.b reversal)"; correct inline `parse_descriptor.rs:1104`→`:1208` AND `parse_descriptor.rs:1108`→`:1212` (the line carries both).
- **(R0-r2 I1) `:89` table row:** `check_key_vector_distinctness … parse_descriptor.rs:1104` → `:1208`.

### 2c. `src/40-bundle-formation/41-bundle-anatomy.md` (RENDERED)
- `:5` `synthesize.rs:593`→`:745`; `verify_bundle.rs:98`→`:143`. `:19` `Bundle … synthesize.rs:20-28`→`:22-30`. `:55` mermaid `synthesize.rs:593`→`:745`. `:96` `any_secret_bearing … synthesize.rs:33-35`→`:35`. `:138` derive_mk1 `synthesize.rs:42-44`→`:44`. `:200` `Bundle … synthesize.rs:20-28`→`:22-30`. `:201` `synthesize_unified … synthesize.rs:593-725`→`:745-827`.
- `:87` ResolvedSlot prose: drop `path_raw` from the field list (now `xpub, fingerprint, path, entropy`); `synthesize.rs:568-582`→`:642-686`.
- **(R0-r3 M2/M3) `:144` + `:209`:** `verify_bundle.rs:98-201` → `:143-201`.

### 2d. `src/40-bundle-formation/42-anti-collision-invariants.md` (RENDERED) — the distinctness-narrative rewrite (the meatiest)
- `:14` derive_mk1 `synthesize.rs:42-44`→`:44`; `bundle.rs:724` re-verify. `:40` `xpub_to_65 … synthesize.rs:69-74`→`:98-103`. `:149` resource refs `synthesize.rs:42-44`→`:44`.
- **(R0-r3 M4) `:5`** preamble: `verify_bundle.rs:98`→`:143`, `parse_descriptor.rs:1104-1117`→`:1208-1212`, `bundle.rs:261-275`→`bundle.rs:423-429`. **`:101`** + **`:145`:** `parse_descriptor.rs:1104-1117`→`:1208-1212`.
- `:117-129` the "bifurcation" section: **REWRITE** — there is NO raw-string-vs-typed bifurcation since v0.37.9/v0.5. `check_resolved_slots_distinctness` (`bundle.rs:429`) compares the TYPED `DerivationPath` (`slots[i].path == slots[j].path`), exactly like the descriptor-layer `check_key_vector_distinctness` (`parse_descriptor.rs:1208`); `h`/`'`-notation folds in BOTH, so `48h/..` and `48'/..` collide at synthesis AND verify. The former `path_raw` raw-string field was deleted (v0.37.9 unification, `SPEC_path_raw_bracketed_bare_unification.md` A2). The ONLY residual is a stale SOURCE doc-comment at `error.rs:13-16` (`Bip388Distinctness` still says "raw-string"), which mis-describes the now-typed behavior. (Drop the §119 "practical consequence" false-bifurcation example; keep the §129 same-phrase collision example — still valid.)
- **(R0-r2 M1) `:129`:** "Both resolve to the same `(xpub, path_raw)` pair" → "same `(xpub, path)` pair" (the field is deleted; the comparison is typed `path`).
- **(R0-r2 M2) `:146` source-pointer label:** "`bundle.rs:261-275` — raw-string `check_resolved_slots_distinctness` (template-mode path; v0.4 doc-comment stale relative to v0.5 SPEC)" → "`bundle.rs:423-443` — typed-`DerivationPath` `check_resolved_slots_distinctness`; doc-comment updated (v0.5 §4.11.b)."

### 2e. `src/60-back-matter/61-glossary.md` (RENDERED)
- `:53` `synthesize_unified … synthesize.rs:593`→`:745`; `verify_bundle.rs:98`→`:143`.
- **(R0-r3 M1) `:405`** (verify-bundle glossary entry): `verify_bundle.rs:98-201`→`:143-201`.

### 2f. Code-side FOLLOWUPs (filed in `design/FOLLOWUPS.md`, NOT fixed here)
- `error-rs-bip388-distinctness-stale-raw-string-comment`: `error.rs:15` doc says "raw-string equality" but both distinctness layers are typed since v0.5 — fix the comment.
- `synthesize-descriptor-vestigial-dead-code-allow`: `#[allow(dead_code)]` at `synthesize.rs:218` is vestigial (synthesize_descriptor is called by synthesize_unified `:826`).

## 3. Verification (no RED — docs)
- `make -C docs/technical-manual lint` GREEN (markdownlint/cspell/lychee).
- Enumerate-and-verify-ALL: `grep -rn 'synthesize\.rs:[0-9]' docs/technical-manual/ | grep -v /build/` → every ref manually matched to `synthesize.rs` at `cacd4a8`; `grep -rn 'verify_bundle\.rs:98' docs/technical-manual/` → none; `grep -rn 'path_raw' docs/technical-manual/src` → no ACTIVE-field doc (only historical "deleted path_raw" mentions OK); `grep -rn 'synthesize.rs:1296\|cmd/bundle.rs:572\|, entropy, privacy_preserving)\|:593\b' docs/technical-manual/` → none.

## 4. Phasing
- **Phase 1 (implement):** 2a-2e (re-verify each line number at edit time). Run §3.
- **Phase 2 (review + ship):** per-phase opus review (re-grep both drift classes + check the distinctness rewrite is behavior-accurate) → 0C/0I → file the 2f FOLLOWUPs + flip `api-harvest-drift-on-synthesize-descriptor-signature` resolved → ff-merge to `master` → push. No tag/bump.

## 5. R0 must confirm
1. The §1 current-source facts are correct (esp. both distinctness layers = typed; schema_version 7 sites; synthesize_descriptor/unified arg-lists).
2. §2 covers EVERY stale ref (re-grep `synthesize\.rs:[0-9]` + `verify_bundle.rs:98` + `path_raw` across ALL of `docs/technical-manual/`).
3. The §2d distinctness rewrite introduces no new falsehood (the bifurcation is genuinely gone; error.rs is the only residual lag).
4. No-bump/no-tag + lint-safe.
