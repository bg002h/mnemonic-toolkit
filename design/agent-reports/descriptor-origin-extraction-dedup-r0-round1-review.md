# R0 Architect Review (round 1) — `SPEC_descriptor_origin_extraction_dedup.md`

**Reviewer:** opus `feature-dev:code-reviewer` (mandatory pre-implementation R0 gate). **Date:** 2026-06-06.
**Branch:** `descriptor-origin-extraction-dedup` (off master `e9ab49a`). **Verdict:** **0 Critical / 0 Important** (+ 3 Minor).

> Persisted verbatim per CLAUDE.md BEFORE the fold. All 3 Minors are documentary §1/§2/§4/§7 accuracy fixes (no design change). Fold → re-dispatch per the after-every-fold loop.

---

## VERDICT: 0 Critical / 0 Important (3 Minor)

**GREEN — implementation may proceed.**

Behavior-preserving consolidation with one deliberate superset widening (h-form acceptance). Every load-bearing claim verified against current source. The highest-risk items (bitcoin_core `entry_idx`, the superset claim, the RED cell) all hold up. The three Minors are SPEC-text accuracy fixes, none blocking.

---

### Critical / Important
None / None.

### Minor

**M1 — SPEC §1 + §4 under-list the converging messages; falsifies the "byte-identical modulo (i)+(ii)" claim.**
SPEC §1 claims the inner logic is "byte-identical modulo (i) error-message prefix (ii) regex," and §4 lists only coldcard's `(internal bug)` and electrum's `slot index N out of range` as converging. But the **xpub-decode** message carries per-slot context in **four** parsers that the SPEC's generic `finalize_slot_fields` flattens to `xpub decode: {e}`:
- `bitcoin_core.rs:463` — `descriptors[{entry_idx}]: xpub decode for slot {slot_idx}: {e}`
- `electrum.rs:949` — `xpub decode for slot {slot_idx}: {e}`
- `sparrow.rs:631` — `xpub decode for slot {slot_idx}: {e}`
- `specter.rs:410` — `xpub decode for slot {slot_idx}: {e}`

Fix: §4 must enumerate these four xpub-decode convergences (plus bitcoin_core's `descriptors[{entry_idx}]` context loss) and §1 should say "byte-identical modulo (i) prefix, (ii) regex, **(iii) per-slot/entry context in the xpub-decode + out-of-range messages**." Minor: none user-reachable (M2), none pinned (grep of `tests/` + `docs/manual/` returns only an unrelated comment at `cli_xpub_search_account_of_descriptor.rs:328`).

**M2 — SPEC should state explicitly that the xpub-decode branch is a proven can't-happen guard.**
Every parser calls `pipeline::concrete_keys_to_placeholders` BEFORE `build_slot_fields` (bitcoin_core:267, bsms:222, sparrow:406, specter:224, coldcard:313, electrum:373). That function (pipeline.rs:116-121) already decodes each `[fp/path]xpub` via the same `key_regex` → `normalize_xpub_prefix` → `Xpub::from_str`, erroring on a bad xpub. So by `build_slot_fields` the decode provably already succeeded (`debug_assert_eq!` at bitcoin_core:293 / pipeline:199). The `xpub decode for slot` branch is the same defensive "(internal bug)" class. Add one sentence to §4.

**M3 — SPEC should instruct KEEPING `entry_idx`/`slot_idx` in the per-parser SELECTION (out-of-range) message (in the wrapper).**
The out-of-range messages live in the `.nth(slot_idx).ok_or_else(...)` selection step the SPEC keeps in each wrapper (bitcoin_core:457 `descriptors[{entry_idx}]: slot index {slot_idx} out of range`; bsms:407 / sparrow:625 / specter:404; electrum:925). Retain this context for free — only `finalize_slot_fields`'s xpub-decode message converges. §2/§7 should say so to prevent over-flattening.

---

### What verified clean

1. **6/4/4 file sets + line numbers — exact, zero drift.** `build_slot_fields` in {bsms:400, bitcoin_core:449, sparrow:618, coldcard:501, specter:397, electrum:912}; `coldcard_multisig.rs` = **0** (SPEC's correction stands). `extract_origin_components` in {bsms:363, bitcoin_core:414, specter:363, sparrow:583}. `origin_capture_regex` in those 4 (bsms:514, bitcoin_core:557, specter:355, sparrow:565) + inline `Regex::new` in coldcard:508 + electrum:920. `key_regex` pipeline.rs:37/:40. Every §1 cite correct.

2. **bitcoin_core `entry_idx` (highest-risk) — factoring sound.** `build_slot_fields(body, slot_idx, entry_idx)` (bitcoin_core.rs:449-467) does `extract_origin_components(body) → .nth(slot_idx) → decode`. `entry_idx` is used ONLY in error-message formatting — never in extraction or selection arithmetic. The shared `extract_origin_components(body, format_name)` serves both bitcoin_core call sites; the wrapper keeps `.nth(slot_idx)` + `entry_idx` messages. SPEC's "thin wrapper keeps selection logic" works.

3. **h-form widening is a genuine superset.** All 4 `origin_capture_regex` + 2 inline copies are the byte-identical literal `\[([0-9a-fA-F]{8})((?:/\d+'?)+)\]([xtyzuvYZUV]pub[A-HJ-NP-Za-km-z1-9]+)`. `key_regex` differs ONLY in the path group `(?:/\d+(?:'|h)?)+` vs `(?:/\d+'?)+`. Capture groups, xpub class, anchoring identical. Every apostrophe input matches identically; only delta is added `h`-acceptance.

4. **Capture-group reuse + imports correct.** All 6 parsers index get(1)=fp/get(2)=path/get(3)=xpub. `pipeline.rs:16-23` already imports `ToolkitError`, `normalize_xpub_prefix`, `Fingerprint`/`DerivationPath`/`Xpub`, `FromStr`, `OnceLock`, `Regex` — helpers compile with no new imports.

5. **Convergent messages safe.** No test/manual pins any convergent string. The "no origin annotations in descriptor" message is ALREADY byte-identical across the 4 multisig `extract_origin_components`. All converging messages are internal can't-happen guards.

6. **SemVer + scope + FOLLOWUP dissolution.** PATCH correct (no clap change → no schema_mirror/manual mirror; no new variant). `import-parser-hform-origin-tolerance` (FOLLOWUPS.md:3497) scope = the 4 `origin_capture_regex` + inline coldcard/electrum copies — all 6 routed through `key_regex` → fully dissolved. Net-LOC negative.

7. **Phase-1 RED cell — well-formed.** `concrete_keys_to_placeholders` (pipeline.rs:99 uses `key_regex`) ALREADY accepts h-form (test `hform_hardened_paths_accepted` pipeline.rs:256); `parse_descriptor` + `DerivationPath::from_str("m/48h/...")` accept h-form end-to-end (`bundle_concrete_hform_converges_with_apostrophe`, cli_descriptor_concrete.rs:16-25). So today an h-form import descriptor passes placeholder+parse, then dies at the apostrophe-only `origin_capture_regex` ("no origin annotations") — exactly the widened step. After refactor it parses. No h→' normalize needed (rust-bitcoin `DerivationPath::from_str` treats `h`≡`'`). Cell should target a multisig parser (bitcoin_core/sparrow) via a Core/Sparrow `h`-form wallet-file / `--from-import-json` import.

---

### Recommendation
Fold M1/M2/M3 (§1/§2/§4/§7 wording) → re-persist → re-dispatch (single round expected; documentary). Then Phase 1.
