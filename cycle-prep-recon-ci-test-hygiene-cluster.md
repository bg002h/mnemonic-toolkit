# cycle-prep recon — 2026-06-11 — CI/test-hygiene cluster (4 slugs)

**Origin/master SHA:** `5fc805f` (v0.53.8, CI in flight) · **Local:** master, up-to-date.
Four verification-robustness slugs (2 `[minor]`, 2 `[obs]`). Recommend splitting by mechanism: a small concrete cycle (g6 + synthesize) and a deferred enumerator-machinery cycle (policynode + lint).

---

## Per-slug verification

### 1. `g6-invariant-sibling-master-not-pin` `[minor]` — CONCRETE CI BUG
- **WHAT:** the `g6-invariant` CI job checks out the sibling `mnemonic-secret` at `ref: master` (not the pinned tag), so the cross-repo `mlock.rs` byte-equality runs against code the toolkit doesn't pin.
- **Citation:** `.github/workflows/rust.yml:219` `ref: master` (job at `:200-229`) — **ACCURATE.**
- **Canonical pin:** `scripts/install.sh:38` → `ms-cli-v0.7.0` (the sibling-pin-check source of truth). Fix: `ref: master` → `ref: ms-cli-v0.7.0`.
- **KEY RISKS for R0/impl:** (a) the byte-equality must HOLD at the pinned tag — if the toolkit's `mlock.rs` was synced to a NEWER mnemonic-secret commit than `ms-cli-v0.7.0`, pinning would RED the job; verify (the CI run after the change is the proof, or check out the sibling tag + diff). If it fails, either the toolkit copy re-syncs or the pin is a newer tag. (b) DRIFT COVERAGE: `sibling-pin-check` scans `cargo install --tag` lines, NOT checkout `ref:` — so a hardcoded `ref: ms-cli-v0.7.0` could go stale on a future ms-cli bump without detection. Options: hardcode + comment (simple, drift-prone), extend sibling-pin-check to also verify the g6 `ref:`, OR read the tag from install.sh dynamically in the job. R0 to choose.
- **Tier:** CI-only NO-BUMP.

### 2. `policynode-grammar-coverage-vacuous-on-joint-omission` `[minor]` — KNOWN+DOCUMENTED
- **WHAT:** `node_kinds_cover_enum` compares `all_variant_samples()` tags == `NODE_KINDS` — both hand-lists — so a PolicyNode variant omitted from BOTH passes vacuously.
- **Citations:** `descriptor_builder/ir.rs:289` (`all_variant_samples`), `:345-355` (`node_kinds_cover_enum`) — **ACCURATE.** The fn doc (`:283-288`) + test doc (`:337-343`) ALREADY document this exact limitation ("does NOT catch a variant omitted from all three together — would need a variant-enumerator macro").
- **Partial mitigation EXISTS:** `:309-333` an exhaustiveness-REMINDER match compile-errors on a new variant, FORCING the author to visit the helper (so the silent-omission risk is LOW, not zero — the gap is the author adding the match arm but forgetting the `samples` vec).
- **Fix:** a variant-enumerator macro (precedent `declare_node_type_variants!` at `cmd/convert.rs:1767`) that generates `samples`/`NODE_KINDS` from one source. MEDIUM effort. Marginal value given the existing compile-error mitigation.
- **Tier:** test-only NO-BUMP.

### 3. `hand-frozen-lint-canons-no-completeness` `[obs]` — ENUMERATOR-MACHINERY
- **WHAT:** `lint_zeroize_discipline` ZEROIZE_ROWS is hand-maintained; the only completeness signal is a count-range + per-row `source.contains` anchors (unlike the argv lint which derives from the schema).
- **Citations:** `tests/lint_zeroize_discipline.rs:251-294` (count `:252-266`, evidence loop `:268-294`); contrast `lint_argv_secret_flags.rs:184-223` — **ACCURATE** (count range now `18..=42` post-v0.53.6).
- **Fix:** a completeness mechanism (derive the canonical owned-secret site list, or a source-scan that enumerates `Zeroizing`/`SecretString` sites and cross-checks). MEDIUM. `[obs]` = low priority.
- **Tier:** test-only NO-BUMP.

### 4. `synthesize-incrate-presence-not-correctness` `[obs]` — TEST-STRENGTH
- **WHAT:** in-crate synthesize sanity tests assert `starts_with("ms1"/"mk1"/"md1")` + non-empty — PRESENCE not CORRECTNESS. A syntactically-valid but WRONG card (wrong key/network/entropy) would pass.
- **Citations:** `synthesize.rs:980-1000` (`full_bundle_emits_three_cards`), `:1002-1013` (`watch_only_bundle_omits_ms1`), `:1015+` (`mk1_chunk_set_id...deterministic`) — **ACCURATE** (drifted from FOLLOWUP's `:959-992`/`:994-1022`).
- **NOTE:** correctness IS covered elsewhere (cli_self_check, verify-bundle round-trip, golden vectors); these in-crate tests are intentionally light smoke. Fix: add round-trip decode assertions (decode emitted ms1 → entropy == input; mk1 → xpub == input). SMALL-MEDIUM. Real strengthening but partly redundant with the integration coverage.
- **Tier:** test-only NO-BUMP.

---

## Cross-cutting
- All 4 are NO-BUMP (no binary/wire change) → no schema_mirror/manual/GUI/sibling lockstep.
- Citation drift only in slug 4 (`:959`→`:980`). Slugs 1-3 accurate.
- 2 are real gaps with clean fixes (g6 = CI bug; synthesize = test strength); 2 need enumerator machinery + are already documented-as-known with partial mitigation (policynode, lint).

## Recommended scope
**Cycle A (do now, NO-BUMP):** `g6-invariant-sibling-master-not-pin` (the actual CI bug) + `synthesize-incrate-presence-not-correctness` (round-trip decode assertions). Both concrete, no macro machinery. R0 gate.
**Cycle B (deferred/optional):** `policynode-grammar-coverage-vacuous-on-joint-omission` + `hand-frozen-lint-canons-no-completeness` — both need a variant-enumerator/completeness macro; both already documented as known limitations with partial compile-error mitigation (lower marginal value). Bundle as one "enumerator-completeness" cycle if pursued.
