# Phase 1 review — descriptor-builder presets — round 1
**Verdict: YELLOW** (0 Critical / 1 Important)

## Critical

None.

## Important

**I1 — Golden non-vacuity cell covers 1 of 5 archetypes; SPEC §7 mandates "per archetype", and the gap is exactly the hole layers 1+2 cannot see.**
`design/SPEC_descriptor_builder_presets.md:174`: "Golden non-vacuity: mutate one param (a threshold or timelock) **per archetype** → descriptor ≠ golden." The shipped suite has a single cell, `preset_negative_discrimination_mutated_param_breaks_golden` (`crates/mnemonic-toolkit/tests/cli_build_descriptor.rs:461-476`), mutating only tiered-recovery's `--older`. This is not pedantry: a lower fn that **hardcodes a fixture value instead of reading the param** (e.g. `lower_hashlock_gated` baking in `Older(144)` instead of `req(params.older, …)` at `archetype.rs:384`) passes layer 1 (fed fixture params, compared to fixture AST — hardcoded value == fixture value) AND layer 2 (same argv values) AND the key-order cell (kofn keys only). Only a mutated-param run per archetype discriminates. Today the code is visibly correct (all 5 lower fns read `params`, archetype.rs:344-447), but the test architecture's job is to keep it that way; 4 of 5 archetypes' params are currently non-vacuity-unpinned (decaying's 4 timelocks/2 thresholds, hashlock's `--older`, kofn's `--threshold`/`--older`, simple's `--older`). Fix is small: loop over `ARCHETYPES` with one mutated numeric param each, or extend the existing cell into the table.

## Minor

**M1 — SPEC §9 "the 10 param flags" arithmetic needs a one-line errata; the implementation's 9 is the correct reading.**
SPEC §1 lists 11 new flags: `--archetype` + 9 value params + `--emit-spec`; the "10 param flags carry `requires = "archetype"`" at SPEC:42 counts `--emit-spec` (SPEC:36 gives it `requires = "archetype"`). SPEC §9 Phase 1 (SPEC:188) says "the 10 param flags" while §9 Phase 2 (SPEC:189) owns `--emit-spec` — an internal contradiction. The commit ships 9 (`build_descriptor.rs:59-97`), which is the only consistent resolution. Recommend a SPEC errata note at fold: §9 Phase 1 → "the 9 value-param flags (the 10th `requires`-carrying flag, `--emit-spec`, is Phase 2)".

**M2 — Provenance table omits SPEC §3.3's third canonical entry form (`quorum-path.<kind>.keys`, None); outcome-equivalent, but Phase 2 must pin it.**
SPEC:118 lists `(quorum-path.<kind>.keys, None) → --key` as part of the canonical mapping; the committed tables (e.g. `archetype.rs:176-181` kofn) carry only the two quorum-path entries, relying on longest-**prefix** semantics so that `root.or_d[0].multi.keys[1]` (SecretKey path, gate.rs:159) matches `("root.or_d[0]", None, --key)`. That is correct under the SPEC's own resolution rule (SPEC:87-91) and the kind-override disambiguation still works (SchemaField at the quorum path → `--threshold`; RepeatedKeys/SecretKey → `--key`). But Phase 2's resolver tests must include a `keys[i]`-path case so the prefix (not exact-match) semantics get pinned — flag this forward.

**M3 — A second contractual `flag: None` site exists that the SPEC doesn't enumerate: decaying-multisig intra-`andor[2]` cross-tier duplicates.**
`--recovery-key X --final-key X` localizes (gate.rs `localize()` post-order) to `root.andor[2]`, which matches no provenance prefix (`archetype.rs:137-146` has `root.andor[2].andor[0]`… but not `root.andor[2]` itself). SPEC:118 only blesses the `root` case. Same contract, deeper path — acceptable, but Phase 2's `flag: None` cell should cover it or the SPEC sentence should generalize ("matches no entry" already technically covers it). Note for the Phase 2 reviewer.

**M4 — Under-`min_count` and non-repeatable-repeated negatives exist only as unit cells, not CLI cells.**
SPEC:170's negative list is covered category-complete, but two categories live only in `archetype.rs` unit tests (`validate_params_under_min_count` :611-619, `validate_params_non_repeatable_repeated` :621-630) while missing/inapplicable/decay have end-to-end CLI cells (`cli_build_descriptor.rs:347-401`). The CLI plumbing (validate_params → emit_diagnostics → exit 2) is proven by the latter, so this is placement, not coverage. Also: the SPEC's R0-r1 M2 "note the clap-rejects-scalar-repeats path separately" note is not recorded anywhere in the test file — add a comment in Phase 2.

**M5 — Preset success-path `--json` and `--network` compositions are untested (probed manually, both work).**
Goldens exercise `--format`; negatives exercise failure-`--json`. The success `--json` envelope and `--network` human view under `--archetype` have no cell. I probed both (see probes below) — correct output, exit 0. SPEC:44 declares composition "unchanged"; cheap to pin in Phase 2 alongside the byte-stability golden.

**M6 — Dead line in drift self-test (b).**
`let _ = BuildDescriptorArgs::augment_args(clap::Command::new("x"));` (`build_descriptor.rs:393`) is a no-op — the `Probe::command()` above it already realizes the derive surface. Misleading comment ("same derive surface"); delete in Phase 2.

**M7 — Pre-existing stale doc comment, not this commit's defect:** `descriptor_builder/mod.rs:11` references "the crate-internal `#![allow(dead_code)]` note in `main.rs`'s module decl", but `main.rs:12` carries no such attribute. Drive-by observation; the per-item allows in archetype.rs ARE therefore load-bearing (clippy confirms).

## SPEC-conformance checklist (clause → conforms/deviates + evidence)

| Clause | Status | Evidence |
|---|---|---|
| §1 flag table: names/types/clap semantics | **Conforms** | `--archetype` `Option<CliArchetype>` value-enum + `conflicts_with = "spec"` (build_descriptor.rs:56-57); 9 value params all `requires = "archetype"` (build_descriptor.rs:61-97); `Vec<String>` derive default = Append; kebab longs proven by drift test (b). `--emit-spec` correctly absent (Phase 2; see M1) |
| §1 dispatch order (R0-r1 I1) | **Conforms** | preset branch at build_descriptor.rs:152-185, after the `--spec-schema` short-circuit (:147-150), before `read_spec` (:187); never reads stdin |
| §1 exit codes | **Conforms** | param + gate refusals both `return Ok(2)` (build_descriptor.rs:169, 180); probed |
| §2 registry shape (table + fn-pointer, no `dyn`; kind-aware provenance triples; metadata-only `kind`; expect-message mandate) | **Conforms** | `ArchetypeDef`/`ParamSpec`/`ArchetypeParams` match SPEC structs field-for-field (archetype.rs:16-98); `kind` never read by `validate_params` (archetype.rs:235-305); every lower-side unwrap panics with "`--<flag>` declared required in ARCHETYPE_REGISTRY for `<id>`" (archetype.rs:332, 339, 350, 380, 446); `lower` is `fn(&ArchetypeParams) -> PolicyNode`, infallible-by-convention (archetype.rs:97) |
| §2.1 alphabetical order | **Conforms** | registry order at archetype.rs:122-220; `CliArchetype` variants (build_descriptor.rs:104-115); pinned by `registry_table_integrity` (archetype.rs:561-566) AND drift test (a) (build_descriptor.rs:368-377) |
| §3.1 producer checks = exactly three categories | **Conforms** | read line-by-line: applicability (archetype.rs:253-260), presence/arity incl. non-repeatable>1 and min_count (archetype.rs:263-287), decay ordering `recovery_older <= older` refused (archetype.rs:293-302). Nothing else |
| §3.2 no gate-rule duplication | **Conforms** | no k≤n, hex, timelock-bounds, or dup-key logic anywhere in archetype.rs; positively pinned by `validate_params_does_not_duplicate_gate_rules` (archetype.rs:648-657) and the two CLI flow-through cells (cli_build_descriptor.rs:408-441: `schema_field`/`repeated_keys` at `root.or_d[0]`) |
| §3.3 Phase-1 slice (`Param` variant + as_str only; `flag` field deferred) | **Conforms** | `DiagnosticKind::Param` first variant, as_str `"param"` (gate.rs:55-61, :87); `Diagnostic` struct untouched → spec-mode `--json` bytes unchanged; producer diags use `node_path: "params"` (archetype.rs:307-309) |
| §6 lowering shapes vs fixture canon | **Conforms** | all 5 lower fns (archetype.rs:344-447) eyeballed against §6 and the live fixture JSONs (tiered + decaying read in full: `s:` wraps on thresh subs 2..n at archetype.rs:421-429, `v:` wraps, nested andor chain, `v:pkh` heir at :411); proven by `producers_reproduce_fixture_asts` against `SpecDoc::parse(fixture)` (archetype.rs:546-555) and byte-equal CLI goldens. Fixtures untouched (last commit `3085330`, Release A) |
| §6/§7 provenance node-path prefixes (Phase-2 consumed) | **Conforms** | cross-checked every entry against gate.rs path grammar: `child_paths` (gate.rs:480-506: `andor[i]`, `or_d[i]`/`or_i[i]`/`and_v[i]`, `thresh.subs[i]`, `wrap.sub`), multi keys `{path}.{kind}.keys[{i}]` (gate.rs:159), threshold/hashlock fire at the node's own path (gate.rs:157, 163, 166). All 27 entries across 5 archetypes land on real paths of the actual lowered trees; deeper diagnostics (wrap.sub, keys[i], thresh.subs[i]) resolve via prefix (see M2, M3) |
| §7 test architecture, Phase-1 scope per §9 | **Deviates (I1) + conforms otherwise** | layer 1 ✓ (archetype.rs:546-555); layer 2 ✓ both formats (cli_build_descriptor.rs:278-307); stdin cells ✓ both directions (piped :316-328; no-stdin via assert_cmd's null stdin under the goldens, documented :311-314); producer negatives ✓ (M4 placement note); gate flow-through ✓ (over-delivery — Phase 2 only adds `flag` assertions); key-order ✓ discriminating (:445-458; a sorting impl makes swapped == golden → `assert_ne` fails); non-vacuity **1/5 archetypes — I1**; drift tests (a)+(b) ✓ (build_descriptor.rs:368-403); clap cells ✓ (:331-344) |
| §9 Phase boundary (nothing from Phase 2 leaked) | **Conforms** | no `--emit-spec`, no `Diagnostic.flag`, no schema archetypes section, no manual edits; Phase-2-only registry fields carry targeted `#[allow(dead_code)]` (archetype.rs:47, 72, 83, 92) — acceptable phase-scoped pattern, each annotated "remove when Phase 2 wires it"; no other unused items (clippy clean) |
| §11.2 canon integrity / TDD | **Conforms** | layer-1 pins against `SpecDoc::parse(include_str!(fixture))` (archetype.rs:459-461, 552), never captured output; no fixture diffs in the commit. Stubs-RED-then-GREEN is squash-unverifiable from the single commit — taken on the commit message's word, code structure consistent with it |
| ToolkitError / main.rs untouched | **Conforms** | neither in the 5-file diff; preset failures route through diagnostics, not new error variants |

## Empirical probes run

1. `cargo test -p mnemonic-toolkit --bin mnemonic archetype` → 10 passed (incl. all 8 new archetype.rs cells + both cmd drift tests), 929 filtered.
2. `cargo test -p mnemonic-toolkit --test cli_build_descriptor` → 22 passed (12 pre-existing + 10 new preset cells).
3. `cargo test -p mnemonic-toolkit --bin mnemonic` (full) → **937 passed, 2 ignored** — commit-message claim ("937 bin-crate + 22 integration") verified exactly.
4. `cargo clippy -p mnemonic-toolkit --all-targets` → finished with zero warnings — claim verified; the dead_code allows are load-bearing (no module-wide allow exists, main.rs:12).
5. Manual preset success probes (untested compositions, M5): `--archetype simple-timelocked-inheritance … --json` → full `{bip388, cost, descriptor, diagnostics:[]}` envelope, exit 0; `… --network testnet` → human view with `tb1q…` first receive address. Both correct.
6. `git log -1 -- tests/fixtures/descriptor_builder/` → `3085330` (pre-Release-B): fixture canon untouched, per SPEC §0's immutability red-flag rule.
7. Stdin-contract discrimination (reasoned, not mutated): a read_spec-first implementation reads the piped garbage (or empty null-stdin) and fails `SpecDoc::parse` → both `preset_ignores_piped_stdin` AND every preset golden would fail. Discriminates. Key-order cell: a key-sorting lower would render swapped argv `K2,K1,K3` identically to the golden (`K1<K2<K3` lexicographically) → `assert_ne` at cli_build_descriptor.rs:457 fails. Discriminates.

One Important (I1) — a small, mechanical test addition — stands between this and GREEN; no source changes to `archetype.rs`/`build_descriptor.rs` are required by any finding.
