# R0 Architect Review — api-harvest-drift-fix — Round 2

> Persisted verbatim from the opus `feature-dev:code-architect` agent
> (`agentId: a013e30e1c4d7336d`). Verified the rewritten full-audit SPEC.

---

## VERDICT: 0 Critical / 1 Important (+ 2 Minor) — NOT YET GREEN

### Verified clean (all §1 facts + the §2d distinctness rewrite)
- Both distinctness layers TYPED: `cmd/bundle.rs::check_resolved_slots_distinctness:429` `slots[i].path == slots[j].path`; `parse_descriptor::check_key_vector_distinctness:1208` `cs[i].path == cs[j].path`. ✓
- `bundle.rs:423-428` doc-comment ALREADY says "TYPED `DerivationPath`" → the §2d claim "the bundle.rs comment is fixed" is accurate. ✓
- `ResolvedSlot` (struct `:642-686`) has NO `path_raw`. ✓
- `error.rs:13-16` `Bip388Distinctness` doc STILL says "raw-string" — the lone residual lag. ✓
- All §1 line numbers correct (Bundle :22-30, any_secret_bearing :35, derive_mk1 :44, xpub_to_65 :98-103, build_descriptor :109, synthesize_full :142, synthesize_watch_only :181, CosignerKeyInfo :219, synthesize_descriptor :229, synthesize_multisig_full :344, synthesize_multisig_watch_only :489, ResolvedSlot :642-686, is_secret_bearing :690, synthesize_unified :745-827). ✓
- 7 `schema_version: "4"` sites correct + complete. ✓ `verify_bundle::run :143` ✓ `check_key_vector_distinctness :1208` ✓.
- §2d rewrite introduces no new falsehood; "no bifurcation since v0.37.9" correct; dropping §119 correct; keeping §129 correct. ✓
- synthesize_descriptor is the live v0.47.1 delegation target (`:826`); `#[allow(dead_code)]:218` vestigial → §2f FOLLOWUP correct. ✓
- No-bump/no-tag correct; `run_language`/`Zeroizing` absent from rendered prose → no cspell risk. ✓ Transcript unrendered → no lint/PDF impact. ✓ §3 greps adequate. ✓

### Important
**I1: `54-mnemonic-toolkit-api.md:72` + `:89` stale, not in §2b.**
- `:72` prose: "`cmd::bundle::check_resolved_slots_distinctness` … uses raw-string equality" — FALSE (typed since v0.37.9); + inline `parse_descriptor.rs:1104` stale (→`:1208`).
- `:89` table: `check_key_vector_distinctness … parse_descriptor.rs:1104` → `:1208`.
Fix: add both to §2b. *(Folded.)*

### Minor
- **M1:** `42-anti-collision:129` "same `(xpub, path_raw)` pair" → `(xpub, path)`. *(Folded into §2d.)*
- **M2:** `42-anti-collision:146` source-pointer label "raw-string `check_resolved_slots_distinctness` … doc-comment stale" → "typed-`DerivationPath` … doc-comment updated"; `bundle.rs:261-275`→`:423-443`. *(Folded into §2d.)*

After folding I1 (+ M1/M2) → expected 0C/0I.
