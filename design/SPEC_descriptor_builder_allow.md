# SPEC — `build-descriptor --allow` (reviewed sanity opt-out, v0.52.0)

**Status:** R0 GREEN (round 2, 0C/0I; 3 cosmetic minors folded) — implementation may begin
**Source grounding verified at:** toolkit `origin/master` = `adab5ac` (2026-06-09); miniscript pinned git rev `95fdd1c` (`Cargo.lock:675`)
**Resolves:** `design/FOLLOWUPS.md::descriptor-builder-allow-extparams-reviewed-optout` (Tier `v0.52+-feature`)
**Recon:** `cycle-prep-recon-descriptor-builder-allow-extparams.md` (untracked) — found the FOLLOWUP's integration guidance STRUCTURALLY-WRONG vs the shipped gate; this SPEC anchors on the corrected mechanism.
**Parent:** `design/SPEC_descriptor_builder_engine.md` §3 step 3 + §3.5 deferral; architect direction-consult (this session, stream B).

## 0. Scope & philosophy

`mnemonic build-descriptor --allow <variant>` — a **per-variant, per-invocation, reviewed opt-out** of the step-3 funds-footgun gate. The gate stays fail-closed by default; an allowance is a deliberate, *loud* review act: the run banner names every rule that actually fired, the `--json` envelope records it, and nothing about the emitted artifacts changes shape. Never silent.

- **The mechanism is a one-line swap, not a parse change (recon headline).** The shipped gate already parses with `from_str_ext(&ExtParams::insane())` (`gate.rs:132-134`) and gates via `sanity_check()` at `gate.rs:141`. `--allow` swaps that call to **`ext_check(&allowed)`** — the parameterized sanity check, present at the pinned rev (`analyzable.rs:242-258`). The FOLLOWUP's "applied at the parse stage — NOT at `sanity_check`" predates Release A and is superseded.
- **Baseline equivalence:** `ext_check(&ExtParams::new())` ≡ `sanity_check()` plus a `raw_pkh` arm that is vacuously false for IR-rendered miniscript (R0-r1 M2: the load-bearing fact is the IR surface — there is no raw-pkh node and `render()` can never emit the `expr_raw_pkh` fragment, per the existing `gate.rs:274` note; the string parser CAN parse `expr_raw_pkh`, so this is an input-space property, not a parser property). With no `--allow`, behavior is bit-identical for build-descriptor's input space; the byte-stability goldens must keep passing untouched.
- **Composes with BOTH input modes.** The gate is mode-agnostic (presets lower to the same `SpecDoc`); no clap restriction ties `--allow` to `--spec` or `--archetype`. The flagship preset user story: the Release-A archetype `degrading-threshold` was CUT because same-key degradation trips `RepeatedPubkeys` — `--allow repeated-keys` is its reviewed resurrection (via `--spec`; the preset registry itself is unchanged).
- **Supersession note (R0-r1 M1):** the ENGINE SPEC's step-3 line (`design/SPEC_descriptor_builder_engine.md:59`, echoed `:95`) names the raw `--descriptor` intake as THE escape hatch for a deliberately-insane policy (`--spec` runs the same gate — that is exactly why `--allow` exists); this SPEC adds the second, *reviewed* construction door. (Recon probe: the raw `--descriptor` ingest door is deliberately LENIENT — accepts insane descriptors today, exit 0 — so an allowed emit round-trips downstream without contradiction; build-descriptor remains the strict construction door.)

**Non-goals:** no change to steps 1/2/4 of the gate; no preset registry change; no `--allow` on any other subcommand; no GUI work (A1 absorbs the flag-name lockstep); no relaxation of the watch-only screen (SecretKey is step 1, not a sanity rule — **not allowable**).

## 1. CLI surface

One new flag on `build-descriptor`:

| Flag | clap type | Semantics | GUI FlagKind (A1 lockstep) |
|---|---|---|---|
| `--allow <ALLOW>` | `Vec<CliAllow>`, `ValueEnum`, `ArgAction::Append` | opt out of one sanity rule per occurrence; repeatable | `Dropdown` (5 values), `repeating: true` |

`CliAllow` (alphabetical declaration, kebab values, **names aligned 1:1 with `DiagnosticKind::as_str`** so refusals are self-teaching — a `mixed_timelock` diagnostic tells the user the exact `--allow mixed-timelock` token):

| CLI value | `ExtParams` field | `AnalysisError` / `DiagnosticKind` |
|---|---|---|
| `malleable` | `malleability` | `Malleable` / `malleable` |
| `mixed-timelock` | `timelock_mixing` | `HeightTimelockCombination` / `mixed_timelock` |
| `repeated-keys` | `repeated_pk` | `RepeatedPubkeys` / `repeated_keys` |
| `resource-limit` | `resource_limitations` | `BranchExceedResouceLimits` / `resource_limit` |
| `sigless-branch` | `top_unsafe` | `SiglessBranch` / `sigless_branch` |

- **`raw_pkh` is deliberately NOT exposed** (the 6th `ExtParams` field): unreachable from IR-rendered miniscript (script-parse-only); exposing it would be a dead toggle. Document in the manual.
- No `requires`/`conflicts` edges: valid with `--spec`, `--archetype`, `--emit-spec`, `--format`, `--json`. (`--spec-schema` ignores it via the existing short-circuit, `build_descriptor.rs:154` — same as every other flag; the known clap-`requires` nuance from the presets cycle does not apply since `--allow` carries no `requires`.)
- Duplicate occurrences of the same variant are idempotent (set semantics; no error).
- Refusal-message affordance: `localize_sanity` failures for an allowable rule append the hint `; rerun with --allow <kebab> after review` to the message (NOT for `SecretKey`/step-1/2/4 kinds).

## 2. Gate integration

```rust
// gate.rs — public surface change (additive):
pub struct ValidatedPolicy {
    pub descriptor: MsDescriptor<DescriptorPublicKey>,
    /// Sanity rules that were allowed AND actually fired (empty when no
    /// --allow, or when allowances were requested but unused). DiagnosticKind
    /// reuses the step-3 kinds 1:1.
    pub allowed_fired: Vec<DiagnosticKind>,
}

pub fn validate(doc) -> …                       // unchanged behavior: delegates, allow = none
pub fn validate_with_cap(doc, cap) -> …         // unchanged behavior: delegates, allow = none
pub fn validate_with_allow(doc: &SpecDoc, cap: usize, allow: &AllowSet) -> Result<ValidatedPolicy, Vec<Diagnostic>>
```

- `AllowSet` is a small gate-local struct (5 bools, `From<&[CliAllow]>` built in cmd) mapping to `ExtParams` — gate.rs does NOT depend on the clap enum (module direction stays cmd → gate).
- Step 3 becomes: `inner_ms.ext_check(&allow.to_ext_params())` (the one-line swap at `gate.rs:141`); `localize_sanity` (`gate.rs:257`) handles non-allowed failures unchanged (`ext_check` returns the same `AnalysisError`s).
- **Fired-vs-requested (the loud-banner substrate):** after `ext_check` passes, for each REQUESTED allowance, evaluate the corresponding public per-rule predicate on `inner_ms` (`requires_sig` `analyzable.rs:187`, `is_non_malleable` `:190`, `within_resource_limits` `:195`, `has_mixed_timelocks` `:198`, `has_repeated_keys` `:201`); the rule **fired** iff the predicate indicates the violation — POLARITY PINNED (R0-r1 M3): three are safety-positive, fired iff NEGATED (`!requires_sig`, `!is_non_malleable`, `!within_resource_limits`); two are violation-positive (`has_repeated_keys`, `has_mixed_timelocks`). The shipped `localize_sanity` dispatch (`gate.rs:261-273`) is the in-repo polarity template. Push the fired rule's `DiagnosticKind` into `allowed_fired` (in `ext_check`'s check order). All five predicates read pre-computed type/ext data — sound to evaluate post-`ext_check`, no panic path. An allowance that did not fire is detectable in cmd as requested∖fired.
- `ext_check`'s short-circuit order (`top_unsafe → malleability → resource_limitations → repeated_pk → timelock_mixing`) means allowing one rule still refuses on the next failing rule — single-diagnostic semantics, matching the existing step-3 behavior. Document; test (allow `malleable`, sigless tree → still exit 2 `sigless_branch`).

## 3. UX — never silent

- **Human (stderr), on success with `allowed_fired` non-empty:** an unmissable banner, e.g.
  `WARNING: sanity rules OVERRIDDEN by --allow and FIRED: repeated-keys. This descriptor failed miniscript's funds-safety analysis; you have accepted that risk after review.`
  One line per fired rule or a comma list — exact copy at implementation, but it MUST name each fired rule's kebab token and is emitted UNCONDITIONALLY on fired — every output mode including `--json` (stderr is free there; never-silent means never — R0-r1 M4).
- **Requested-but-unused (stderr, all modes):** `note: --allow <kebab> was requested but did not fire (the policy passes that rule without it)` — nudges the user to drop stale allowances.
- **`--json`:** the success envelope gains `"allowed_rules_fired": ["repeated_keys", …]` (snake_case `DiagnosticKind::as_str`), inserted ONLY when non-empty → the default envelope stays byte-identical (the `Diagnostic.flag` precedent). Failure envelope unchanged.
- **Cost-preview posture on an allowed-insane emit (R0-r1 C1 — load-bearing).** The cost pipeline re-parses into Tap context with a SANE-ONLY parse (`cost/strip.rs:65-70` → `ContextIncompat`) — it re-runs the very rules `--allow` waived, and hard-errors on all three constructible insane variants (probed). Therefore, when `allowed_fired` is non-empty the emit paths MUST NOT attempt the cost preview (deterministic skip, not try-and-catch): `--json` emits `"cost": null`; the human view prints one line `cost preview unavailable for a sanity-overridden descriptor` in the cost block's position on STDOUT (the banner is stderr — R0-r2 M-r2-2). `compare-cost` itself stays strict (its refusal is correct — a sigless/malleable tree voids the weight guarantees the comparison quotes). `--format descriptor|bip388` paths are unaffected (no cost run; bip388 conversion is lenient — probed). Test cells pin the posture per output mode.
- **`--emit-spec` does NOT record allowances.** An allowance is a per-invocation review act, not a document property: the emitted spec replayed WITHOUT `--allow` correctly refuses (test cell). The banner still prints on the `--emit-spec` run itself.

## 4. Manual mirror

`docs/manual/src/40-cli-reference/41-mnemonic.md` `#mnemonic-build-descriptor`: `--allow` flag row (variants, repeatability) + a short "Reviewed sanity opt-out" subsection: what each variant permits, the banner, fired-vs-unused, `raw_pkh` exclusion rationale, the `--emit-spec`-doesn't-record rule, the same-key degrading-threshold example (`--spec` + `--allow repeated-keys`), the cost-preview-unavailable behavior, and a note that an allowed repeated-keys policy emits duplicate `keys_info` entries in bip388 (signer registration behavior is signer-defined — R0-r1 M5). Synopsis line gains `[--allow <RULE>]…`. Full manual lint (incl. cspell) before push.

## 5. Testing (TDD; RED first)

- **Allow-success cells (integration):**
  - sigless: `or_d(pk(K1), after(100))` via `--spec` + `--allow sigless-branch` → exit 0, descriptor emitted, stderr banner contains `sigless-branch`, `--json` carries `allowed_rules_fired: ["sigless_branch"]`.
  - repeated-keys (the resurrected degrading-threshold story): same-key `or_d(multi(2,K1,K2), and_v(v:multi(1,K1,K2), older(1000)))` via `--spec` + `--allow repeated-keys` → exit 0 + banner.
  - mixed-timelock (R0-r1 I1 — keyed, so `top_unsafe` does not short-circuit first): `and_v(v:pk(K1), and_v(v:older(100), older(4194304)))` via `--spec` + `--allow mixed-timelock` → exit 0 + banner. (Probed: only `mixed_timelock` fires, at `root.and_v[1]`; 4194304 = bit-22 = time-based relative, within the step-1 `older < 2^31` bound. The keyless variant refuses `sigless_branch` FIRST — wrong cell.)
  - malleable / resource-limit: best-effort constructions; if a minimal in-envelope shape is disproportionate to author, cover via gate-level unit tests driving `ext_check` + the predicate directly and DOCUMENT the integration-cell gap (the mechanism is uniform across the 5 — the per-variant mapping is what needs pinning, and the unit cells pin it).
- **Still-refuses cells:** allow `malleable` on the sigless tree → exit 2 `sigless_branch` (short-circuit order); no `--allow` on each allow-success input → exit 2 with the SAME diagnostic as before PLUS the new `; rerun with --allow <kebab> after review` hint.
- **Requested-but-unused:** kofn-recovery preset (sane) + `--allow repeated-keys` → exit 0, `note: … did not fire` on stderr, NO `allowed_rules_fired` key in `--json`.
- **Preset composition:** a preset whose params trip a sanity rule (kofn + duplicate `--key`) + `--allow repeated-keys` → exit 0 + banner + (Phase-2-presets provenance untouched: no diagnostic is emitted on success).
- **`--emit-spec` interaction (R0-r1 I2 / R0-r2 M-r2-1 — must be a PRESET invocation; ground truth: `run()` honors `emit_spec` only inside the archetype branch (`build_descriptor.rs:196`), and clap drops a conflicted-out arg's `requires` so `--spec … --emit-spec` is silently ACCEPTED-and-ignored today — a pre-existing nuance, noted as a docs/FOLLOWUP one-liner at ship, out of this cycle's scope):** kofn-recovery + duplicate `--key` + `--allow repeated-keys --emit-spec` → spec printed + banner; replaying the emitted spec via `--spec -` WITHOUT `--allow` → exit 2. (No clap-edge changes to `--emit-spec` in this cycle.)
- **Cost-posture cells (R0-r1 C1):** sigless + `--allow sigless-branch --json` → exit 0, `cost` is `null`, `allowed_rules_fired` present; same input human view → exit 0, the `cost preview unavailable…` line present, no error; `--format descriptor` → exit 0, bare descriptor (no cost involvement).
- **bip388 shape note (R0-r1 M5):** allowed repeated-keys emit `--format bip388` → exit 0; the duplicate key appears as TWO `keys_info` entries (no dedup — probed); pin the shape in a cell and document in the manual (hardware-signer registration behavior on duplicate `keys_info` is out of toolkit scope).
- **Byte-stability regressions:** the existing spec-mode `--json` diagnostics golden and the success-envelope cell pass UNCHANGED (no `--allow` ⇒ no new key, banner absent); all 5 archetype goldens unchanged.
- **Unit (gate):** `AllowSet` → `ExtParams` mapping per variant (5 cells); fired-detection per variant via predicates; `validate`/`validate_with_cap` delegate with empty allow (behavior-identical — pin with the sigless tree refusing through both entry points); duplicate `--allow` tokens idempotent.
- **Drift self-test:** `CliAllow::value_variants()` kebab names == the corresponding `DiagnosticKind::as_str` values with `_`→`-` (pins the self-teaching alignment).

## 6. Locksteps, SemVer, release

- **SemVer: MINOR `v0.52.0`** (new flag = new authoring capability + additive `--json` field; v0.49.0/v0.51.0 precedent).
- **GUI `schema_mirror`:** `--allow` flag-NAME debt folds into A1's single pin bump — EXTEND `gui-build-descriptor-presets-pending-pin-bump` in BOTH repos at ship (add `--allow` — NOTE: a **repeating Dropdown**, a FlagKind combination the GUI schema has not used before (current repeats are Text) — plus the `allowed_rules_fired` + `cost: null` un-gated wire notes); do NOT file a separate FOLLOWUP. While extending the toolkit entry, fix its stale "(to be filed in the GUI repo)" companion line (the GUI entry exists — R0-r1 M6).
- **Manual mirror** in-PR (§4). No sibling-codec companions.
- **Release gate:** per `design/RELEASE_CHECKLIST.md` "Toolkit per-release ritual" — **item 1 now includes the `[0.52.0]` CHANGELOG section in the release commit (first live exercise of `changelog-check.yml`)**; one-commit bump sites; full suite after bump; ALL master CI green; tag = install.sh self-pin; install-pin-check + changelog-check green on the tag.

## 7. Phasing

Single phase (TDD: cells RED → implement → GREEN → reviewer-loop to 0C/0I) + release. The surface is one flag, one gate fn, one banner block, one manual section — splitting would be ceremony.

## 8. Source grounding (grep-verified at `adab5ac`; miniscript at `95fdd1c`)

- `crates/mnemonic-toolkit/src/descriptor_builder/gate.rs` — `:36` `ValidatedPolicy`; `:110` `validate`; `:115` `validate_with_cap`; `:132-134` `from_str_ext(&ExtParams::insane())`; `:141` `sanity_check()` (the swap site); `:257` `localize_sanity`; `DiagnosticKind` step-3 kinds + `as_str`.
- `crates/mnemonic-toolkit/src/cmd/build_descriptor.rs` — `:57` `archetype` / `:104` `emit_spec` (flag-block placement); `:147` `run`; `:154` `--spec-schema` short-circuit; `:183`/`:216` the two `gate::validate` call sites (preset + spec — BOTH thread the allow set); `:253` `emit_diagnostics` (hint suffix); `:288` `emit` / `:299` success `json!` envelope (`allowed_rules_fired` insertion).
- miniscript `95fdd1c` `src/miniscript/analyzable.rs` — `:28-42` `ExtParams` (6 fields incl. `raw_pkh`); `:46` `new()`; `:225-239` `sanity_check` (≡ `ext_check(new())` minus `raw_pkh`); `:242-258` `ext_check` (short-circuit order); predicates `:187/:190/:195/:198/:201`.
- Recon probe: `export-wallet --descriptor 'wsh(or_d(pk(K1/<0;1>/*),after(100)))'` → exit 0 (raw door lenient; no downstream contradiction).

---

## Fold log

- **R0 round 1 (YELLOW → folded, 2026-06-09; persisted at `design/agent-reports/descriptor-builder-allow-r0-r1-review.md`):** C1 cost preview is FATAL on allowed-insane trees (Tap re-parse re-runs waived rules; probed on all 3 constructible variants) → deterministic skip when `allowed_fired` non-empty (`cost: null` / human one-liner; compare-cost stays strict) + per-mode cells. I1 mixed-timelock cell re-constructed KEYED (`and_v(v:pk(K1), and_v(v:older(100), older(4194304)))` — the keyless variant refuses sigless_branch first; probed). I2 `--emit-spec` cell rewritten as a preset invocation (clap: emit-spec requires archetype, archetype conflicts spec). M1 supersession note re-attributed to the ENGINE SPEC :59 + quote fixed. M2 raw_pkh vacuousness re-grounded on the IR render surface (string parser CAN parse expr_raw_pkh). M3 predicate polarity pinned (3 negated / 2 direct; localize_sanity is the template). M4 banner unconditional on fired (incl. --json). M5 duplicate-keys_info bip388 shape pinned + manual note. M6 A1-FOLLOWUP extension notes the repeating-Dropdown novelty + fixes the stale companion line.
- **R0 round 2 (GREEN 0C/0I, 2026-06-09; persisted at `design/agent-reports/descriptor-builder-allow-r0-r2-review.md`):** all 9 round-1 folds verified RESOLVED (independent re-probes of the keyed mixed-timelock tree, the flagship repeated-keys tree, compare-cost fatality on the new cell, bip388 dup-keys_info). 3 cosmetic minors folded: M-r2-1 the I2 rationale re-grounded (run() honors emit_spec only in the archetype branch; clap drops a conflicted-out arg's requires — `--spec … --emit-spec` is silently accepted-and-ignored today, pre-existing, noted for a docs/FOLLOWUP one-liner at ship); M-r2-2 the human cost one-liner pinned to STDOUT in the cost block's position; M-r2-3 citation nits (gate.rs:274; engine echo :95 only). **Gate satisfied.**
