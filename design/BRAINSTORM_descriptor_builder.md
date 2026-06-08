# BRAINSTORM — custom-miniscript descriptor builder ("descriptor wizard")

**Repo:** `mnemonic-toolkit` (engine) + `mnemonic-gui` (later UX). **Status:** brainstorm / design-converged — **pre-cycle-prep, pre-SPEC, pre-R0**. Building Release A is a separate cycle-prep → SPEC → mandatory R0 gate → TDD.
**Date:** 2026-06-08. **Source SHA:** `origin/master` == `fa72455`.
**FOLLOWUP:** new slug `descriptor-builder-engine` (filed this cycle). **NOT** `miniscript-beyond-bip388` — that entry (`FOLLOWUPS.md:1713`) is **`resolved` at v0.19.0** and covers the *ingest* side (accept/consume arbitrary miniscript via `--descriptor`, i.e. the raw-paste door). This brainstorm is the orthogonal *construction* side (a guided builder that CREATES custom descriptors). The two are complements: the v0.19.0 ingest path is the wizard's out-of-envelope escape hatch.
**Provenance:** converged through 4 opus-architect design-direction consults + 1 advisor pass during the 2026-06-08 brainstorm session (input-model comparison; and/or/andor + IR-shape; arbitrary-vs-archetypes; phasing). Citations below are architect-verified at `fa72455` but **MUST be re-grepped at SPEC-write time** (CLAUDE.md convention).

---

## 0. Origin & intent

Grew out of a user question ("can the toolkit do anything with this `wsh(andor(...))` vault?" → "is there a command to accept a wallet policy and emit a descriptor?"), which surfaced that the toolkit can *consume* arbitrary descriptors (`--descriptor`, `compare-cost`, `export-wallet`, `bundle`) but offers **no guided way to CREATE a custom miniscript descriptor**. The idea: a builder that helps users assemble + validate custom vault policies safely.

**End goal = a GUI wizard** in `mnemonic-gui` that lets users compose a policy interactively. The toolkit ships the **engine** that wizard consumes (constellation convention: toolkit = engine, gui = thin UX).

---

## 1. Locked design decisions (with provenance)

| Axis | Decision | Why |
|---|---|---|
| **Surface** | Toolkit **engine first**; GUI wizard consumes it in a later cross-repo cycle. | The toolkit is the testable/scriptable engine; the GUI is the approachable UX. Matches `bundle`/`export-wallet`. |
| **Scope** | **General structured policy-tree engine**, with archetypes as **presets** over it — NOT archetypes-only. | A *GUI wizard for custom descriptors needs structured composition* (build+validate a tree node-by-node). The `--descriptor` path is a string **passthrough** (paste a finished descriptor) — it does NOT serve guided composition. So the wizard can't be mere curation. And given the fragment IR is general by construction, "general" is nearly free vs archetypes — the validation layer is needed either way. |
| **Input model** | **Versioned JSON node-tree spec** (`--spec <FILE\|->`) canonical; **flag shorthand** for the 2 flat archetypes only. | Spec scales to tiered+hashlock shapes, is GUI-targetable, round-trippable, mirrors `--from-import-json`. JSON not TOML (`serde_json` already a dep). Flag shorthand for flat archetypes rides the existing `gui-schema` flag-mirror for free. |
| **IR shape** | **Fragment-level (Option F)**: explicit `andor`/`thresh`/`and_v`/`or_*`/timelocks/hashlocks, hand-built → emitted via the existing `from_str → Display → BIP-380-checksum` pipeline. | `andor` is **not** a `Concrete::Policy` constructor — it's a compiler optimization. The funds-safe canonical form *has* `andor` in it; only hand-built fragments can **pin** it. The rust-miniscript `compiler` feature stays OFF (`Cargo.toml:35` = `default-features=false, features=["std"]`). |
| **Combinators** | `and`/`or`/`andor` are **IR-internal** (archetypes/builder compose them); **no user-facing raw-combinator input** in v1. | Hand-authoring arbitrary policy *strings* already ships via `--descriptor` (v0.19.0); the wizard adds *structured* composition, not a second string door. |
| **Archetypes (v1)** | 5: `simple-timelocked-inheritance`, `decaying-multisig` (tiered), `kofn-recovery`, `degrading-threshold`, + **hashlock-gated paths**. | The user's selected set; they double as the IR's expressiveness acceptance test. |
| **Wrapper** | `wsh(…)` in v1; `tr(…)` later via the wrapper-strategy seam. | General policy under tr = multi-leaf taptrees (deferred); the cost engine already refuses `MultiLeafTr`, and `tr-sortedmulti-a` is partly upstream-blocked (md-codec manual-`Terminal` path; FOLLOWUPS:63-64). NOTE: string-emit via `build_descriptor_string` *does* handle `tr(sortedmulti_a)`, so the block is narrow — but general multi-leaf tr is out of v1. |
| **SemVer** | Each release = **MINOR** (new subcommand / additive capability). | New top-level subcommand. |

---

## 2. Architecture — producer → IR → consumer

```
PRODUCERS                         IR (the contract)              CONSUMERS
─────────────────────             ───────────────────            ─────────────────────────────────
• JSON node-tree spec  ─────▶     PolicyNode tree       ─────▶   • validate (typecheck+sanity+plan+cap)
• archetype presets    ─────▶     (fragment-level:               • emit → wsh descriptor (+ checksum)
  (Release B)                      Key/Thresh/Multi/And/          • BIP-388 wallet-policy JSON (reuse)
• [phase-2] Concrete::Policy       Or/Andor/Older/After/          • compare-cost per-condition preview
  compiler — attaches at the       Sha256/Hash160/…)             • structured per-node diagnostics
  STRING boundary, not the IR
```

The **IR is the load-bearing seam**: producers and consumers never couple to each other. The deferred general `Concrete::Policy` compiler lands later as just another producer at the **string boundary** (`Concrete::Policy` → `Display` → the same `from_str → Display → checksum` pipeline every descriptor already flows through) — **zero IR rework**.

### The five extensibility seams (baked into v1)
1. **IR-as-contract** — producer/consumer boundary (above).
2. **Archetype = trait + registry** — `Archetype { id, validate, lower→PolicyNode, field_schema }` + a registry; adding archetype #6 = one additive file, no core sweep.
3. **Wrapper = strategy** — `emit(node, Wrapper::Wsh | …)`; `Tr{internal_key}` / `ShWsh` are added variants later, IR untouched.
4. **Versioned, data-driven spec-schema** — `--spec-schema` emits the node-tree grammar + archetype field-specs as DATA, so the GUI renders new archetypes generically (drift-gated). Born versioned (node-tree-schema v1), **separate** from the flat `gui-schema` flag contract.
5. **Shared primitive AST** — `PolicyNode` variants align with what `cost/enumerate.rs` already classifies; new miniscript primitives extend builder + cost together. (SPEC decides: extract a shared `policy_ast` module vs reuse — `cost/enumerate.rs` types are currently *enumeration*-only, so the construction IR is net-new but small.)

**YAGNI guard:** no plugin loading, no archetype DSL, no speculative node kinds. The 5 seams map exactly to the 4 known growth directions (more archetypes; general compiler / expression input; tr output; GUI consumption).

---

## 3. The engine — Release A (`v0.50.0`)

New top-level subcommand (name TBD: `build-descriptor` / `descriptor` / `policy`). New module `src/descriptor_builder/`.

**Pipeline:** JSON spec (`--spec <FILE|->`) → deserialize to `PolicyNode` (versioned, `deny_unknown_fields`) → **validation gate** → IR→miniscript string → `wsh(M)` wrap → `build_descriptor_string` (multipath + BIP-380 checksum; `wallet_export/pipeline.rs:18`) → **reviewable output bundle**.

### 3.1 Validation gate (the funds-safety core — substrate largely exists)
Emit is **gated** on, in order:
1. **Parse/typecheck** — `MsDescriptor::from_str` in the wsh/Segwitv0 context (a successful `from_str` *is* the miniscript type-check; the dual-context parse pattern is in `cost/translate.rs`). Reject on failure.
2. **`Miniscript::sanity_check()`** — natively rejects the funds footguns *for free*: `SiglessBranch` (anyone-can-spend path), `Malleable`, `HeightTimelockCombination` (mixed height/time timelock → unspendable path — *the* "wrong timelock loses money" guard), `BranchExceedResourceLimits`, `RepeatedPubkeys`. Surface each as a **node-addressed structured diagnostic**, not an opaque string. Explicit, reviewed opt-outs (e.g. deliberate cross-branch timelock mixing) only via an `ext_check`-style escape, never silent.
3. **Per-branch satisfiability** — `plan()` over the asset powerset (reuse `cost/enumerate.rs:205-262`); flag any branch with no satisfying configuration (a dead branch = locked funds).
4. **Build-time complexity cap (NET-NEW)** — bound tree depth / leaf-count / distinct key+hash count at *construction* so the cost-preview enumeration ALWAYS renders. (`cost/enumerate.rs:111-121` today *refuses* past `--max-conditions` — which would make the review surface vanish exactly when a policy is most complex/dangerous. Move the cap upstream to "refuse to build an over-complex tree.") This defines the **"always-previewable envelope"** = the v1 wizard scope boundary; past-envelope policies route to raw `--descriptor`.

### 3.2 Output bundle (review-before-emit)
The engine returns, as one reviewable bundle:
- the concrete `wsh(…)#csum` descriptor;
- the **BIP-388 wallet-policy JSON** (`descriptor_to_bip388_wallet_policy` reuse) — round-trips straight back through the v0.49.0 `--descriptor` policy intake (`wallet_import::pipeline::expand_bip388_policy`), so wizard output re-enters bundle/export unchanged;
- the **`compare-cost` per-condition preview** (reuse `cost/enumerate.rs`; **keeps both wsh + tr columns** even though emit is wsh-only) + a human branch-summary;
- (optional) derived first addresses / `policy_id` for pre-funding verification.
`--json` for the GUI (which renders its own review pane); default human text shows the branch summary + cost table.

### 3.3 `--spec-schema` (ships in A — non-negotiable)
The node-tree JSON *is* the engine's wire contract → it must be **born versioned + drift-gated** so Release B and the GUI can't silently break it. A `--spec-schema` dump (node grammar + archetype field-specs) + a self-test that it matches the serde structs. Parallels `cmd/gui_schema.rs` (incl. the cross-repo lockstep mechanism) but is a **separate** schema axis (node-tree-schema-v1 vs gui-schema-v5).

### 3.4 Net-new pieces to size honestly (rest is reuse)
- the build-time complexity cap (§3.1.4);
- the **per-node diagnostic mapping** — miniscript's `AnalysisError` reports at the *fragment* level; mapping each back to the user's IR *node* requires threading node identity through translation. The fiddly part, and the GUI contract depends on it.
- the node-tree (de)serializer + versioned schema.

---

## 4. Archetype presets — Release B (`v0.51.0`)

5 thin **producers** over the IR (`Archetype::lower → PolicyNode`), each flowing through the *same* §3.1 validation gate (no second validation path) + flag shorthand for the 2 flat archetypes (`simple-timelocked-inheritance`, `kofn-recovery` — these ride the existing flag-mirror). Schema-additive; **no node-tree-schema version bump**. Risk here is golden-pinning discipline, not architecture.

**Keystone:** the 5 archetype node-trees are hand-authored as **Release-A R0 fixtures** (JSON instances, before any producer code) — proving IR expressiveness against all 5 targets, freezing the schema against reality, and pre-pinning the canonical descriptor forms B's producers must reproduce. This makes the 2-release split as safe as a single tag.

---

## 5. Phasing / roadmap

| Release | SemVer | Size/Risk | Contents |
|---|---|---|---|
| **A** | `v0.50.0` MINOR | **large / highest-risk** | IR + versioned node-tree (de)serialize; validation gate (typecheck + sanity_check + plan + build-time cap); IR→wsh emit; BIP-388 round-trip; compare-cost preview; per-node diagnostics; **`--spec-schema` + drift gate**. Independently shippable (power-user JSON + the round-trip; it's the structured-composition capability the GUI needs). |
| **B** | `v0.51.0` MINOR | **small / low-risk** | 5 archetype presets + flag shorthand (2 flat) + canonical goldens. Schema-additive. |
| **GUI** | (mnemonic-gui cycle) | — | archetype param-forms first (close to today's slot-grid infra); the **recursive node-tree builder is the dominant GUI cost center → deferred**; past-envelope → `--descriptor` escape hatch. |

**Eliminated alternatives:** full-v1-one-cycle (too large an R0 surface vs the repo's focused-MINOR norm); engine-only-defer-presets (the 5 archetypes are the IR's acceptance test — don't defer indefinitely); phase-within-one-tag (only real contender; its edge is neutralized by the R0-fixtures keystone).

---

## 6. Testing strategy
- **Golden-spec fixtures** per archetype + representative general trees: spec → exact descriptor + BIP-388 JSON + cost table. **Canonical-form goldens are mandatory** (funds crate): pin the descriptor string *and* its BIP-388 round-trip so a path-declaration nuance can't silently diverge md1/mk1 (the lesson from the `wsh(andor)` F4 path-decl divergence).
- **`assert_cmd` integration:** `<subcommand> --spec <fixture>` → exact output; round-trip (built descriptor → `export-wallet --format bip388` reproduces; → `bundle`).
- **Negative cells:** `sanity_check` rejections (sigless / malleable / mixed-timelock / resource-limit / repeated-key) each surface a node-addressed diagnostic; over-envelope tree refused at build; malformed spec; k>n; bad hex.
- **Invariant:** every emitted descriptor parses + round-trips via miniscript `from_str → Display`.
- **Schema:** `--spec-schema` matches the serde structs (self-test) + (later) GUI mirror drift gate.

---

## 7. Risks the SPEC(s) must nail (consolidated)
1. **Canonical-form stability** — pin per-archetype/per-fixture byte-exact `Display`-canonicalized descriptor + BIP-388 round-trip via goldens; residual drift surface = miniscript's `Display` layer (rare, CI-caught). A vault descriptor that changes shape changes its address/`policy_id`.
2. **The node-tree schema is a NEW versioned contract**, separate from the flat `gui-schema` flag projection (which collapses composites to `"text"`). Freeze + gate it **with the engine in A**, before B or the GUI pins it. The recursive GUI node-tree builder is the dominant later-GUI line item, not a footnote.
3. **Structured per-node diagnostics** — map each `AnalysisError` to the offending IR node (node path-addressing); the difference between "validates" and "guides."
4. **Always-previewable envelope** — specify the v1 complexity caps (depth/leaf/key+hash) chosen so the cost preview always renders; state explicitly that past-envelope policies are wizard-out-of-scope → raw `--descriptor`.
5. **IR/compiler convergence** — keep the `compiler` feature OFF in v1; ensure the deferred `Concrete::Policy` producer attaches at the string boundary (it does, by the crate's existing idiom).

---

## 8. Scope boundaries (explicitly deferred)
- General `Concrete::Policy` compiler + **policy-expression-string** input (phase 2; shares the IR via the string boundary).
- `tr(…)` taptree output (wrapper-strategy seam; partly upstream-blocked for general multi-leaf).
- The **GUI wizard** (separate cross-repo cycle): archetype forms, then the recursive node-tree builder.
- Past-envelope / arbitrary-complexity policies (stay on `--descriptor`).
- User-facing raw-combinator string composition (stays on `--descriptor`).

---

## 9. Locksteps & SemVer
- **Release A:** MINOR; new subcommand → GUI `schema_mirror` flag-name lockstep (for any flags on the subcommand) + manual mirror under `docs/manual/src/40-cli-reference/`; the new node-tree `--spec-schema` is its OWN gate (not the flag-mirror).
- **Release B:** MINOR; flag-shorthand subcommands → `schema_mirror` + manual lockstep; node-tree schema additive (no version bump).
- No sibling-codec companion (toolkit-local string/AST work; md-codec already parses/serializes the resulting concrete descriptor).

---

## 10. Source grounding (architect-verified at `fa72455`; re-grep at SPEC time)
- `crates/mnemonic-toolkit/src/cost/enumerate.rs` — `plan()` per-branch satisfiability (~:205-262); timelock/hashlock AST walks; the combinatorial precheck/cap (~:111-121) = the previewable-envelope boundary.
- `crates/mnemonic-toolkit/src/cost/translate.rs` — dual-context (`Segwitv0`/`Tap`) `from_str` typecheck + the `Miniscript` construction the IR emits into.
- `crates/mnemonic-toolkit/src/wallet_export/pipeline.rs` — `build_descriptor_string` (~:18-31, deterministic emit + multipath + checksum) + `descriptor_to_bip388_wallet_policy` (round-trip output).
- `crates/mnemonic-toolkit/src/cmd/gui_schema.rs` — the FLAT flag-schema; the node-tree schema must be a versioned sibling, not an extension.
- `crates/mnemonic-toolkit/src/cmd/{bundle,export_wallet}.rs` + `src/wallet_import/pipeline.rs::expand_bip388_policy` — the v0.49.0 `--descriptor` BIP-388 intake the output round-trips through.
- `Cargo.toml:35` — `miniscript = { version = "13", default-features = false, features = ["std"] }` (compiler feature OFF; `sanity_check()` is core API, available).
- `design/FOLLOWUPS.md:1713` — `miniscript-beyond-bip388` (**resolved v0.19.0**, the ingest counterpart — NOT this brainstorm's parent); `:63-64` — the tr-sortedmulti-a / `Terminal::SortedMultiA` upstream nuance.

---

## 11. Next steps (NOT this brainstorm)
1. cycle-prep the FOLLOWUP `descriptor-builder-engine` (the new construction-side slug; distinct from the resolved ingest-side `miniscript-beyond-bip388`).
2. Write `SPEC_descriptor_builder_engine.md` for **Release A** → mandatory opus R0 gate → 0C/0I → implement (per-phase TDD + reviewer-loop) → `v0.50.0`.
3. Release B SPEC (`v0.51.0`) reusing A's R0 archetype fixtures as goldens.
4. GUI cycle (mnemonic-gui) consuming the `--spec-schema`.
