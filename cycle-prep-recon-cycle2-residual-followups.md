# cycle-prep recon — 2026-06-07 — technical-manual-residual-line-ref-drift + error-rs-bip388-distinctness-stale-raw-string-comment + synthesize-descriptor-vestigial-dead-code-allow

**Origin/master SHA at recon time:** `3ea612a`
**Local branch:** `master`
**Sync state:** `up-to-date (0 ahead / 0 behind)`
**Untracked:** `.claude/`, `CONTINUITY.md`, `cycle-prep-recon-*.md`, `feature-coverage-survey-*.md` — no tracked-tree modifications.

Slug(s) verified: the 3 follow-ups filed by cycle 2 (api-harvest). **Slug 1 ACCURATE (with structural sub-findings). Slug 2 ACCURATE. Slug 3 STRUCTURALLY-WRONG (mis-cited symbol; re-scopes to a valid action).**

---

## Per-slug verification

### `technical-manual-residual-line-ref-drift` — ACCURATE (real residual drift)
- **WHAT:** the technical manual is broadly line-ref-stale beyond the synthesize/distinctness scope (3 deferred classes).
- **Citations (vs `3ea612a`):**
  - mermaid node `docs/technical-manual/src/40-bundle-formation/41-bundle-anatomy.md:55` `synthesize_unified … synthesize.rs:593` — **ACCURATE/DRIFTED** (current `:745`); intentionally left at `:593` (editing it rehashes the cached figure; `make figures-cache` needs `chromium`, absent).
  - `bundle.rs:724` (mk1/ms1 5-hex `chunk_set_id` format) — **DRIFTED → `bundle.rs:1078`** (`format!("{:05x}", derive_mk1_chunk_set_id(...))`).
  - `bundle.rs:707` (md1 4-hex `chunk_set_id` format) — **STRUCTURALLY-WRONG**: there is NO `{:04x}` format in `cmd/bundle.rs` at all (the md1 stub render is not a `bundle.rs` 4-hex format-string at `:707`; `md1_chunk_set_id` is computed at `bundle.rs:1054` and rendered elsewhere). Cited at `42-anti-collision-invariants.md:13,147`, `41-bundle-anatomy.md:138,193`, `61-glossary.md:89`.
  - `verify_bundle.rs:831-836` (`MappingFailure`) — **DRIFTED → `:1527`**; `verify_bundle.rs:838-1277` (`emit_multisig_checks`) — **DRIFTED → `:1533`**. Cited at `42-anti-collision-invariants.md:24,71,91,141-144`.
- **Action for brainstorm spec:** a dedicated technical-manual line-ref refresh — but two hard parts: (a) the mermaid regen needs `chromium`/docker (environmental blocker); (b) the md1 4-hex `chunk_set_id` ref needs a where-does-md1-render-its-stub audit (not a simple line swap). Recommend pairing with a **gating mechanism** (the root enabler is technical-manual has NO CI + `api-surface-coverage.sh` is advisory name-only). Cite `3ea612a`.

### `error-rs-bip388-distinctness-stale-raw-string-comment` — ACCURATE
- **WHAT:** `error.rs` `Bip388Distinctness` doc-comment says "raw-string" but the behavior is typed.
- **Citations (vs `3ea612a`):** `crates/mnemonic-toolkit/src/error.rs:13-16` — **ACCURATE**: `:15` reads "`(xpub, derivation_path_string)` raw-string equality per §4.11.b". Both layers are typed (`bundle.rs:429`, `parse_descriptor.rs:1208`) — confirmed in the api-harvest cycle. The `bundle.rs:423-428` comment was already resynced; `error.rs:13-16` is the lone source-comment lag.
- **Action for brainstorm spec:** reword `error.rs:13-16` to "typed `DerivationPath` equality (`h`/`'` folds)". Code-comment-only; no behavior/API change. Cite `3ea612a`.

### `synthesize-descriptor-vestigial-dead-code-allow` — STRUCTURALLY-WRONG (mis-cited symbol → re-scope)
- **WHAT (as filed):** "`#[allow(dead_code)]` at `synthesize.rs:218` on `synthesize_descriptor` is vestigial — remove it."
- **Citations (vs `3ea612a`):**
  - `synthesize.rs:218` `#[allow(dead_code)]` "above synthesize_descriptor" — **STRUCTURALLY-WRONG**: `:218` is the allow on **`pub type CosignerKeyInfo = ResolvedSlot`** (`:219`), NOT `synthesize_descriptor`. The full allow inventory in `synthesize.rs`: `:108`(build_descriptor), `:141`(synthesize_full), `:180`(synthesize_watch_only), `:218`(**CosignerKeyInfo**), `:323`+`:339`(synthesize_multisig_full), `:485`(synthesize_multisig_watch_only), `:689`(is_secret_bearing). **`synthesize_descriptor` (`:229`) has NO `#[allow(dead_code)]`** (correctly — it's live, called by `synthesize_unified` at `:826`). So there is nothing on `synthesize_descriptor` to remove; the FOLLOWUP's premise is false.
  - **No falsehood was shipped:** the cycle-2 chapter (`54-…:56`/`:68`) correctly states `synthesize_descriptor` is "live", lists ONLY the 4 truly-dead variants behind `#[allow(dead_code)]`, and does NOT claim `synthesize_descriptor` carries the attribute. (The R0/SPEC text mis-attributed `:218`, but the implementer's applied text is accurate.)
- **Re-scope:** the `:218` allow on **`CosignerKeyInfo`** IS genuinely vestigial — `CosignerKeyInfo` is USED at `synthesize.rs:231` (synthesize_descriptor's signature `cosigners: &[CosignerKeyInfo]`, a live fn) + test sites. So the valid action is: **remove the `#[allow(dead_code)]` at `:218` on `CosignerKeyInfo` and verify the build is warning-clean** (build-clean proves it was vestigial; if a dead_code warning fires, it wasn't — revert). Re-title the FOLLOWUP → `cosigner-key-info-vestigial-dead-code-allow`. Cite `3ea612a`.

---

## Cross-cutting observations
1. **A cycle-prep-caught mis-citation in a same-day FOLLOWUP (again).** Slug 3's `synthesize.rs:218` was attributed to `synthesize_descriptor` by the api-harvest R0/SPEC; it's actually `CosignerKeyInfo`'s allow. The implementer didn't propagate the error into the shipped docs (good), but the FOLLOWUP carries it. This is the 3rd session instance of a freshly-filed FOLLOWUP mis-citing (cf. `manual-gui-…:422`, `gui-timestamp-…default_value`).
2. **Slug 1 has a non-mechanical sub-finding** (md1 4-hex `chunk_set_id` `bundle.rs:707` doesn't exist) + an environmental blocker (chromium for the mermaid figure regen) → it is NOT a pure line-swap cycle.
3. No DRIFTED-by-N on slugs 2; slug 1 has multiple DRIFTED + 1 STRUCTURALLY-WRONG; slug 3 STRUCTURALLY-WRONG-but-rescopable.

---

## Recommended brainstorm-session scope
- **Cycle A — tiny code-hygiene (slug 2 + corrected slug 3).** `error.rs:13-16` comment reword + remove the vestigial `#[allow(dead_code)]:218` on `CosignerKeyInfo` (verify build warning-clean). Both source, NO behavior/API/CLI change → **no GUI schema_mirror, no manual mirror.** SemVer: comment + attribute-removal with no behavior change → **R0 to rule no-bump-commit vs PATCH+tag** (lean PATCH+tag since it's a `src/*.rs` change that a release should record; but it's borderline). RED-equivalent for slug-3 = "build is warning-clean after removal" (if it isn't, the allow wasn't vestigial). Tiny (~2-line). Mandatory R0 (1 round likely).
- **Cycle B — technical-manual residual drift (slug 1).** Bigger + decision-heavy: (i) the `chromium` blocker for the mermaid figure regen (defer the mermaid node or install/docker?), (ii) the md1 4-hex `chunk_set_id` where-does-it-render audit, (iii) whether to build a CI/lint **gating mechanism** (the durable fix) vs another manual sweep. **Recommend: a brainstorm with a scope decision FIRST** (this is the same ballooning risk as api-harvest — it could surface yet more drift). Own cycle. Docs-only, no version bump/tag.
- **Ordering:** Cycle A first (clean, fast). Cycle B is independent + needs a scope/blocker decision.
