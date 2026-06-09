# SPEC — descriptor-builder archetype presets (Release B, `v0.51.0`)

**Status:** draft for R0 review
**Source grounding verified at:** toolkit `origin/master` = `e6e4a06` (2026-06-09)
**Parent artifacts:** `design/BRAINSTORM_descriptor_builder.md` (§4 Release B, §5 roadmap, §7 risks), `design/SPEC_descriptor_builder_engine.md` (Release A, shipped v0.50.0), `design/FOLLOWUPS.md::descriptor-builder-engine` (Tier `v0.51-feature`), recon `cycle-prep-recon-descriptor-builder-engine-release-b.md`, architect direction-consult (this session — ranked CLI shape (iv) hybrid first).

## 0. Scope & philosophy

Release B adds the 5 curated archetype **presets** to `mnemonic build-descriptor`: thin producers that lower flag-supplied parameters into the **frozen** Release-A `PolicyNode` IR and flow through the **same** validation gate — so a user gets a reviewed vault descriptor without hand-authoring node-tree JSON.

- **Producers over the IR, nothing else.** The IR (`ir.rs:26` `SUPPORTED_SCHEMA_VERSION: u32 = 1`, `ir.rs:87` `enum PolicyNode`, 17 variants) is untouched. No node-tree schema version bump; `schema.rs:13` `SPEC_SCHEMA_VERSION` stays 1 (the archetype param-spec extension in §5 is an additive sibling key, within `schema.rs`'s own stated bump rule — "when the node set / field shapes change", which they don't).
- **Single validation path (brainstorm §4 mandate).** Every preset lowers to a `SpecDoc` and calls `gate::validate` (`gate.rs:96`). The producer layer validates ONLY what the gate cannot know (§3); duplicating any gate rule at the producer is a defect.
- **Watch-only-out unchanged** — key params are extended PUBLIC keys; xprv screening stays in the gate (`gate.rs:198` `check_secret_key`), which the producer feeds.
- **Canon = the Release-A fixtures.** The 5 hand-authored fixtures at `crates/mnemonic-toolkit/tests/fixtures/descriptor_builder/` (5 × {`.json`, `.descriptor`, `.bip388`}) pre-pin the shapes B's producers must reproduce (engine-SPEC §6 keystone). **The fixtures are immutable in this cycle** — any fixture diff in a Release-B commit is a review-stopping red flag.

**Non-goals:** GUI wizard/forms (separate mnemonic-gui cycle); `tr` wrapper (Release A seam, later); `--allow`/`ExtParams` opt-out (`descriptor-builder-allow-extparams-reviewed-optout`, v0.52+); optimizing compiler (lives in `md`); any new archetype beyond the canonical 5; any change to spec-mode (`--spec`) behavior.

## 1. CLI surface

Direction-consult ranking adopted: **(iv) hybrid** — `--archetype <name>` + a generic, reusable parameter vocabulary, lower→gate→emit in one shot, plus `--emit-spec` to print the lowered node-tree JSON instead of building. One uniform surface for all 5 archetypes (the brainstorm's "flag shorthand for the 2 flat archetypes" dissolves: with a generic vocabulary, all 5 ride the flat flag-mirror — over-delivery, recorded as a deliberate deviation from brainstorm §4's letter).

New flags on `build-descriptor` (`cmd/build_descriptor.rs:23` `BuildDescriptorArgs`), all additive; the existing 5 flags + global `--no-auto-repair` are unchanged:

| Flag | clap type | Semantics | GUI FlagKind (lockstep, §8) |
|---|---|---|---|
| `--archetype <NAME>` | `Option<CliArchetype>` value-enum, `conflicts_with = "spec"` | preset selector; kebab values (declaration order, alphabetical — §2.1): `decaying-multisig`, `hashlock-gated`, `kofn-recovery`, `simple-timelocked-inheritance`, `tiered-recovery` | `Dropdown` (5 values) |
| `--key <KEY>` | `Vec<String>`, `ArgAction::Append`, `requires = "archetype"` | primary-path key(s); argv order preserved | `Text`, `repeating: true` |
| `--threshold <K>` | `Option<u32>`, `requires = "archetype"` | primary quorum k | `Number` |
| `--recovery-key <KEY>` | `Vec<String>`, `Append`, `requires = "archetype"` | recovery-path key(s) | `Text`, `repeating: true` |
| `--recovery-threshold <K>` | `Option<u32>`, `requires = "archetype"` | recovery quorum k | `Number` |
| `--final-key <KEY>` | `Option<String>`, `requires = "archetype"` | last-resort key (decaying-multisig tier 3) | `Text` |
| `--older <N>` | `Option<u32>`, `requires = "archetype"` | relative timelock on the gated path (tier-1 timelock for decaying-multisig) | `Number` |
| `--recovery-older <N>` | `Option<u32>`, `requires = "archetype"` | decaying-multisig tier-2 relative timelock | `Number` |
| `--after <N>` | `Option<u32>`, `requires = "archetype"` | decaying-multisig tier-3 absolute timelock | `Number` |
| `--hash <HEX>` | `Option<String>`, `requires = "archetype"` | hashlock-gated SHA-256 digest (64 hex chars) | `Text` |
| `--emit-spec` | `bool`, `requires = "archetype"`, `conflicts_with_all = ["format", "json"]` | print the lowered+validated node-tree JSON to stdout instead of building (§4) | `Boolean` |

clap semantics, pinned:

- `--archetype` ↔ `--spec` mutual exclusion is **clap-level** (`conflicts_with`); if neither is given and stdin is a TTY, the existing no-input error path applies unchanged. `--spec-schema` retains its existing dump-and-exit precedence (`build_descriptor.rs:70`).
- **Dispatch order (R0-r1 I1):** `--archetype` dispatch branches BEFORE spec intake — preset mode MUST NOT call `read_spec` or touch stdin at all (`read_spec` at `build_descriptor.rs:91-114` errors on TTY-no-`--spec` and otherwise reads stdin to EOF; routing presets through it would make `--archetype` fail spuriously at a terminal or block on an open pipe). §7 pins both directions: a preset run succeeds with no stdin attached, and piped stdin content is ignored.
- All 10 param flags carry clap-level `requires = "archetype"` — a bare `--key` without `--archetype` is a clap usage error, not a runtime diagnostic.
- **Per-archetype applicability is NOT encoded in clap.** clap cannot condition `requires` on a dropdown *value*, and Release A deliberately avoided conditional rules to dodge the GUI conditional-rule projection (`gui_schema.rs::build_subcommand_conditional_rules` has no build-descriptor arm — keep it that way). Applicability/arity live in the producer (§3), with messages in flag vocabulary.
- `--network`, `--format`, `--json` compose with preset mode unchanged — they act downstream of `ValidatedPolicy` (`gate.rs:36`), exactly as in spec mode.
- New `CliArchetype` value-enum lives in `cmd/build_descriptor.rs` beside `CliBuildFormat`, `rename_all = "kebab-case"`.

Exit codes unchanged: 0 success; 2 = refused input (param/gate diagnostics), matching the shipped diagnostics path (`build_descriptor.rs:82` `emit_diagnostics`).

## 2. Archetype registry + producers

New module `crates/mnemonic-toolkit/src/descriptor_builder/archetype.rs` (sibling of `ir`/`gate`/`schema`, registered in `mod.rs`). Engine-SPEC §5 seam 5 said "trait+registry"; the minimal shape honoring its intent (adding archetype #6 = one additive entry) is a **static data table with fn-pointer lowering — no `dyn`**, because all 5 producers consume the same flat param struct:

```rust
/// Flat clap-collected preset parameters (one struct for all archetypes).
pub struct ArchetypeParams {
    pub keys: Vec<String>,
    pub threshold: Option<u32>,
    pub recovery_keys: Vec<String>,
    pub recovery_threshold: Option<u32>,
    pub final_key: Option<String>,
    pub older: Option<u32>,
    pub recovery_older: Option<u32>,
    pub after: Option<u32>,
    pub hash: Option<String>,
}

/// One param's declaration — drives generic applicability/arity validation
/// AND the `--spec-schema` archetypes section (§5). `flag` is the literal
/// clap long name (with leading `--`). `kind` is schema/manual METADATA ONLY —
/// no producer check keys off it (hex/timelock validation is the gate's, §3.2);
/// `AbsoluteLocktime` is deliberately locktime-neutral (the decaying-multisig
/// canon uses a block HEIGHT `after(500000)`, not a unix time — R0-r2 M1).
pub struct ParamSpec {
    pub flag: &'static str,
    pub required: bool,
    pub repeatable: bool,
    pub min_count: usize, // 1 for scalars; ≥2 where a quorum needs it
    pub kind: ParamKind,  // Key | Threshold | Blocks | AbsoluteLocktime | HexDigest
}

pub struct ArchetypeDef {
    pub id: &'static str, // == the CliArchetype kebab name
    pub summary: &'static str, // one-line human description (manual + schema)
    pub params: &'static [ParamSpec],
    /// Kind-aware provenance for gate-diagnostic annotation (§3.3, R0-r1 C1):
    /// (node_path prefix, kind override, flag). Resolution = longest prefix;
    /// at equal prefix a Some(kind)-matching entry beats a None (catch-all)
    /// entry. Needed because two flags can land diagnostics on the SAME node
    /// path (a quorum node's k-vs-n SchemaField is `--threshold`'s fault; its
    /// RepeatedKeys / keys[i] SecretKey are `--key`'s).
    pub provenance: &'static [(&'static str, Option<DiagnosticKind>, &'static str)],
    pub lower: fn(&ArchetypeParams) -> PolicyNode,
}

pub const ARCHETYPE_REGISTRY: &[ArchetypeDef] = &[ /* 5 entries, §2.1 order */ ];
```

- **Generic validation first, lowering second.** One shared `validate_params(def, &params) -> Result<(), Vec<Diagnostic>>` driven entirely off `def.params`: rejects params supplied but not declared for this archetype (names the flag), missing `required` params, count < `min_count`, plus the per-archetype semantic checks in §3.2. `lower` is only called on validated params and is **infallible by convention** (returns `PolicyNode`, not `Result`) — every failure mode belongs to either `validate_params` or the gate. **R0-r1 M1 caveat:** this infallibility is convention-coupled, not type-enforced — `lower` unwraps Options/indexes Vecs on the promise that `def.params` declared them required/min-counted, so a mis-declared registry row for a future archetype #6 would panic. The `Result`-free signature stays (layer-1 tests make panic unreachable for the shipped 5), but every such unwrap MUST use `expect("--<flag> declared required in ARCHETYPE_REGISTRY for <id>")`-style messages naming the coupling.
- `lower` builds the **exact fixture shapes** (§6 table) — structural wrappers (`v:`, `s:`), combinator skeletons, and the multipath-free key strings slot user params into the same positions the fixtures use. Key order = argv order, untouched (`ir.rs:233` `render` is order-faithful for `multi`; `sortedmulti`'s descriptor string also preserves authored order — sorting is script-time).
- The lowered tree is wrapped as `SpecDoc { schema_version: 1, wrapper: Wsh, root }` and handed to the **same** code path spec mode uses: `gate::validate` (`gate.rs:96`) → emit. No bespoke emit.

### 2.1 Registry order
Registry entries in the `CliArchetype` declaration order = alphabetical by id: `decaying-multisig`, `hashlock-gated`, `kofn-recovery`, `simple-timelocked-inheritance`, `tiered-recovery`. (Matches the repo's alphabetical-variant convention for new enums; the dropdown the GUI mirrors uses the same order.)

## 3. Validation — division of labor (the no-second-path rule, made precise)

### 3.1 Producer-level (param-addressed) — ONLY what the gate cannot know
1. **Applicability:** a supplied param not declared for the chosen archetype → refused, naming both (`--hash is not a parameter of kofn-recovery`).
2. **Presence/arity:** missing required param; fewer than `min_count` values (e.g. kofn-recovery needs ≥ 2 `--key`; tiered-recovery needs ≥ 2 `--key` AND ≥ 2 `--recovery-key`; simple-timelocked-inheritance needs exactly 1 `--key` and 1 `--recovery-key` — `repeatable: false` params given twice are refused).
3. **Decay ordering (decaying-multisig only):** require `--recovery-older` > `--older`. Both values are individually gate-valid and the inverted tree is sane, yet inversion silently defeats the archetype's purpose (tier 2 would unlock before tier 1). A user who genuinely wants inverted timelocks has `--spec`.

### 3.2 Gate-level (node-addressed) — everything else flows through unchanged
k ≤ n (`gate.rs:212` `check_threshold`), duplicate keys → `RepeatedKeys` (sanity step), hex digest length/charset (`gate.rs:223` `check_hashlock`), timelock bounds (`older`/`after` field rules in `validate_fields`), xprv screen (`gate.rs:198`), type errors, malleability, resource limits, envelope cap. **The producer MUST NOT pre-check any of these** — e.g. no k≤n check in `validate_params`; `--threshold 5` with 2 keys lowers and the gate's `SchemaField` diagnostic fires.

### 3.3 Bridging the addressing gap (param provenance on gate diagnostics)
In preset mode, a gate diagnostic carries a `node_path` into a tree the user never authored (e.g. `root.or_d[0].multi.keys[1]`). Two additive changes:

- `Diagnostic` (`gate.rs:45`) gains `#[serde(skip_serializing_if = "Option::is_none")] pub flag: Option<String>` — `None` in spec mode (existing `--json` output stays **byte-identical**), populated in preset mode by resolving `def.provenance` (longest prefix; kind-specific entry beats catch-all at equal prefix — §2). Canonical quorum-node mapping: `(quorum-path, Some(SchemaField)) → --threshold`; `(quorum-path, None) → --key`/`--recovery-key`; `(quorum-path.<kind>.keys, None) → --key`/`--recovery-key` (covers `keys[i]` `SecretKey` paths). Producer `min_count ≥ 2` guarantees the only `SchemaField` reaching the gate at a quorum node is threshold-flavored, so the mapping is unambiguous. A diagnostic whose path matches no entry (e.g. a cross-branch duplicate key — `--key X --recovery-key X` — which `localize()` post-order resolves to `root`) carries `flag: None`; that is acceptable and tested.
- New `DiagnosticKind::Param` variant (as_str `"param"`) for §3.1 producer diagnostics, placed FIRST in the enum (the enum orders by gate step — `gate.rs:54` — and producer checks are step 0; this enum is not `ToolkitError`, so the alphabetical-variant convention does not apply, but note it for the reviewer). Producer diagnostics use `node_path: "params"` and name the offending flag(s) in `message`; once the `flag` field exists they also set `flag: Some("--<flag>")`. **Phasing (R0-r1 I2): the `Param` variant + its `as_str` arm land in Phase 1** (Phase 1's producer-negative cells construct it; `Diagnostic.kind` is non-optional); the `flag` field + provenance annotation are Phase 2.
- Human (`stderr`) rendering appends ` (from --<flag>)` when provenance resolves.
- This is an **additive `--json` wire-shape change** — NOT gated by GUI `schema_mirror` (flag-NAME-only); the GUI must be informed via the §8 FOLLOWUP channel, never assumed.

## 4. `--emit-spec`

`--archetype <a> [params] --emit-spec`: lower → `validate_params` → `gate::validate` → on success, pretty-print the lowered `SpecDoc` JSON to stdout (serde; `SpecDoc` already derives `Serialize`), exit 0; on failure, the standard human diagnostics on stderr, exit 2.

- **The gate runs before printing.** Presets never emit ANY artifact — even a reviewable one — that the gate refuses; preset mode stays behaviorally identical to spec mode except for which artifact prints.
- `conflicts_with_all = ["format", "json"]` (clap-level): the spec JSON **is** the machine-readable output; no envelope variant is invented. `--network` is accepted and **ignored** under `--emit-spec` (it only feeds the human-view address, `build_descriptor.rs:194-203`; the SpecDoc has no network field) — consistent with `--format descriptor` ignoring it; the manual states this (R0-r1 M3).
- Contract: the emitted document is **value-equal** (parsed-JSON equality) to what `--spec` would accept; pretty-printing/key-ordering is non-contractual. Round-trip guarantee: piping the emitted spec back via `--spec -` produces byte-identical descriptor/bip388 output to the one-shot preset run (§7 layer 3).

## 5. `--spec-schema` archetypes extension

Extend `spec_schema_json()` (`schema.rs:46`) with an additive sibling key, generated FROM `ARCHETYPE_REGISTRY` (no hand-maintained copy):

```json
"archetypes": [
  { "id": "decaying-multisig", "summary": "…",
    "params": [ { "flag": "--key", "kind": "key", "required": true, "repeatable": true, "min": 2 }, … ] },
  …
]
```

- Rationale: brainstorm §2 seam "versioned data-driven `--spec-schema` (so the GUI + Release-B presets render/validate generically)" — B creates the archetypes, so B ships their field-specs. **No new `--archetype-schema` flag** (avoids another GUI mirror flag).
- Wire-key projection (R0-r2 M3): `min_count` projects as `min`; `ParamKind` projects via a snake_case `as_str` (`key`, `threshold`, `blocks`, `absolute_locktime`, `hex_digest`).
- `SPEC_SCHEMA_VERSION` stays 1 (§0).

## 6. Per-archetype parameter table (ground truth = the immutable fixtures)

| Archetype | Pre-pinned shape (fixture `.json`/`.descriptor`) | Flag → IR slot | Structural (not user-settable) |
|---|---|---|---|
| `simple-timelocked-inheritance` | `or_d(pk(P), and_v(v:pkh(H), older(N)))` | `--key`×1→P; `--recovery-key`×1→H; `--older`→N | heir is `pkh` under `v:`; `or_d` skeleton |
| `kofn-recovery` | `or_d(multi(k,K…), and_v(v:pk(R), older(N)))` | `--key`×n(≥2)+`--threshold`→multi; `--recovery-key`×1→R; `--older`→N | unsorted `multi`; `v:pk` recovery; `or_d` |
| `decaying-multisig` | `andor(multi(k1,T1…), older(N1), andor(multi(k2,T2…), older(N2), and_v(v:pk(F), after(T))))` | `--key`×n1(≥2)+`--threshold`→tier1; `--older`→N1; `--recovery-key`×n2(≥2)+`--recovery-threshold`→tier2; `--recovery-older`→N2; `--final-key`×1→F; `--after`→T | nested `andor` chain; `v:pk` tier 3 |
| `tiered-recovery` | `or_i(sortedmulti(k1,P…), and_v(v:older(N), thresh(k2, pk, s:pk…)))` | `--key`×n1(≥2)+`--threshold`→sortedmulti; `--older`→N; `--recovery-key`×n2(≥2)+`--recovery-threshold`→thresh | `s:` wraps on thresh subs 2..n; `v:older`; `or_i` |
| `hashlock-gated` | `andor(pk(A), sha256(H), and_v(v:pk(B), older(N)))` | `--key`×1→A; `--hash`(64-hex)→H; `--recovery-key`×1→B; `--older`→N | hash fn fixed `sha256` in v1; `andor`; `v:pk` |

Notes the manual must carry verbatim: (a) **key order is significant** — argv order maps to quorum order, and even `sortedmulti`'s descriptor string preserves authored order (script-time sorting); (b) `--older` means "the timelock gating the recovery path" everywhere except decaying-multisig, where it is the tier-1 timelock — the per-archetype table is the disambiguator; (c) `--threshold` defaults: none — required wherever a quorum exists.

## 7. Testing (TDD; the three-golden-layer + negatives architecture)

**Layer 1 — IR-level producer-vs-fixture equivalence (the keystone; unit tests in `archetype.rs`).** For each archetype: build `ArchetypeParams` from the fixture's own values, assert `lower(&params) == SpecDoc::parse(include_str!(<fixture>.json)).unwrap().root` — exact `PolicyNode` AST equality (`PartialEq`, `ir.rs:87`). Pins the producer to the **Release-A JSON**, never to its own captured output (a self-golden would bless a fork of the canon).

**Layer 2 — end-to-end CLI goldens (`tests/cli_build_descriptor.rs`).** Extend the existing `ARCHETYPES` table (`:22`) with `preset_args: &'static [&'static str]`; for each archetype run `--archetype … --format descriptor` and `--format bip388`, assert **byte-equal to the SAME `.descriptor`/`.bip388` files** Release A pins (`archetype_descriptor_goldens` `:60`). One fixture set, two producers, one canon.

**Layer 3 — `--emit-spec` round-trip.** (a) emitted JSON value-equals the fixture JSON (`serde_json::Value` equality); (b) pipe it back through `--spec -` → byte-identical descriptor output to the one-shot run.

**Stdin contract cells (R0-r1 I1):** a preset run with no stdin attached (closed/null) succeeds; a preset run with content piped to stdin ignores it (output byte-equal to the no-stdin run).

**Negative cells (each asserting exit 2 + the discriminating diagnostic):**
- Per-archetype: missing required param; inapplicable param (`--hash` with `kofn-recovery`); under-`min_count` keys; non-repeatable param repeated — **pin this cell to a Vec-typed flag** (e.g. `--key`×2 on `simple-timelocked-inheritance`, which reaches `validate_params`' `repeatable: false` check); scalar `Option<_>` flags repeated are rejected by clap itself as a usage error — note that path separately, no producer cell possible (R0-r1 M2).
- Decay ordering: `--recovery-older` ≤ `--older` → producer `Param` diagnostic naming both flags.
- Gate flow-through (proves no producer duplication AND provenance): `--threshold 5` with 2 keys → gate `SchemaField` + `flag: "--threshold"`; duplicate `--key` values (same quorum) → `RepeatedKeys` + `flag: "--key"` (both land on the SAME quorum node path — this pair is exactly what the kind-aware provenance disambiguates, §3.3); bad `--hash` hex → `SchemaField` + `flag: "--hash"`; cross-branch duplicate (`--key X … --recovery-key X`) → `RepeatedKeys` at `root` with `flag` ABSENT (the `None` case is contractual, not a bug).
- Key-order discrimination: swap two `--key` values on a `multi` archetype → descriptor ≠ golden (proves order preservation is load-bearing and tested).
- Golden non-vacuity: mutate one param (a threshold or timelock) per archetype → descriptor ≠ golden (mirrors `negative_discrimination_mutated_threshold_breaks_golden` `:220`).

**Drift self-tests:** (a) registry ids == `CliArchetype` value-enum variants == `--spec-schema` `archetypes[].id` (same discipline as `schema.rs`'s `grammar_matches_node_kinds_hand_list`); (b) every `ParamSpec.flag` names a real clap long on `BuildDescriptorArgs` (introspect via `clap::Command` + `augment_args`, iterate `get_arguments`); (c) spec-mode `--json` diagnostics byte-stability: **no pre-existing byte-golden exists** (current tests assert fields only, `cli_build_descriptor.rs:126-138`), so this test PINS a literal golden now (a known failing spec → exact `{"diagnostics":[…]}` bytes) and asserts the `"flag"` key is absent. Implementation note (R0-r1 M5): `emit_diagnostics` routes through `serde_json::Value` (`json!` at `build_descriptor.rs:123`), so object keys serialize **alphabetically** (`kind, message, node_path`; a present `flag` key sorts first) — pin the golden accordingly, don't fight serde field order.

## 8. Locksteps, SemVer, release gate

- **SemVer: MINOR `v0.51.0`** per the locked roadmap (brainstorm §5) and the v0.49.0 precedent (new authoring capability = MINOR, despite additive-flags-PATCH heuristic).
- **GUI `schema_mirror` (2-release arc):** 11 new flag NAMES + 1 dropdown enum ⇒ `mnemonic-gui/src/schema/mnemonic.rs` `BUILD_DESCRIPTOR_FLAGS` must grow in a paired GUI PR **after** the toolkit tag (chicken-and-egg: the gate runs against the pinned binary). At ship, file `gui-build-descriptor-presets-pending-pin-bump` in BOTH repos' FOLLOWUPS (pattern: `design/FOLLOWUPS.md::gui-build-descriptor-schema-mirror-pending-pin-bump`, resolved GUI v0.29.0). FlagKind picks per §1 table; key flags are `Text` (xpub strings — NOT `Path`; the `--spec` Path lesson cuts the other way here). The FOLLOWUP must also enumerate (R0-r1 M4, R0-r2 M2): the §3.3 `--json` wire-shape addition (`flag` field + `param` kind + `node_path: "params"` sentinel — un-gated surface); the §5 `--spec-schema` `archetypes` section (the surface the GUI wizard cycle consumes — also un-gated); and the deliberately UN-projected clap rules the GUI cycle must decide on knowingly — the `--archetype`↔`--spec` mutex, the 10 `requires = "archetype"` edges, and the `--emit-spec` conflicts (precedent: compare-cost's mutexes ARE hand-projected for drift-gating, `gui_schema.rs:374-409`, while build-descriptor's `SubcommandSchema.conditional` is currently `None` — GUI forms could emit argv clap refuses; acceptable since the CLI is the gate, but it must be a recorded decision).
- **Manual mirror (in-PR):** extend `docs/manual/src/40-cli-reference/41-mnemonic.md` `#mnemonic-build-descriptor` (`:3878`): synopsis (`:3897`), flags table (11 rows), a per-archetype parameter table (§6 is the template), `--emit-spec` + provenance prose. **Run the FULL manual lint (incl. cspell) locally before pushing** — archetype/flag names are cspell bait (the v0.50.0 `ecba644` ship-detour lesson).
- **No sibling-codec companions** (toolkit-local). **CHANGELOG.md:** stays untouched per the last-4-releases precedent (`changelog-md-release-ritual-lapsed-since-v0-47-4` is the user's open call).
- **Release gate (CONTINUITY ritual):** Cargo.toml + Cargo.lock + both README `<!-- toolkit-version -->` markers + `scripts/install.sh` self-pin in ONE commit → full suite AFTER bump → push → ALL master CI green (incl. `manual.yml`) → annotated tag = install.sh self-pin → push tag → `install-pin-check` green.

## 9. Phasing (per-phase TDD + reviewer-loop to 0C/0I)

- **Phase 1 — registry + producers + goldens.** `archetype.rs` (registry, `ArchetypeParams`, `validate_params`, 5 `lower` fns); **`DiagnosticKind::Param` + its `as_str` arm** (R0-r1 I2 — Phase 1's producer negatives construct it); `CliArchetype` + the 9 value-param flags (the 10th `requires`-carrying flag, `--emit-spec`, is Phase 2 — P1-r1 M1 errata) + preset dispatch (before spec intake, §1) in `cmd/build_descriptor.rs`; tests: layer 1 + layer 2 + stdin-contract cells + producer negatives + key-order + non-vacuity + drift self-tests (a)/(b). RED first.
- **Phase 2 — affordances + surface polish + ship.** `--emit-spec` (+ layer 3); §3.3 provenance (`Diagnostic.flag` field, kind-aware annotation pass, gate flow-through provenance cells, byte-stability self-test (c)); `--spec-schema` archetypes section (+ id-sync self-test (a) extended to schema ids); manual mirror + full manual lint; release ritual + tag; file the GUI FOLLOWUP pair; flip `descriptor-builder-engine` → resolved (Release B shipped; note the GUI wizard remains tracked via the companion line).

Sizing: ~500–700 src + ~400–500 test LOC (architect estimate; recon's 300–500 was pre-provenance/schema-extension).

## 10. Source grounding (grep-verified at `e6e4a06`; re-verify on fold)

- `crates/mnemonic-toolkit/src/descriptor_builder/ir.rs` — `:26` `SUPPORTED_SCHEMA_VERSION=1`; `:87` `PolicyNode` (17 variants, `PartialEq`); `:179` `SpecDoc::parse`; `:233` `render` (order-faithful `multi`/`sortedmulti` at `:237-241`).
- `crates/mnemonic-toolkit/src/descriptor_builder/gate.rs` — `:96` `validate`; `:101` `validate_with_cap`; `:45` `Diagnostic {node_path, kind, message}` (derive line `:44`); `:54` `DiagnosticKind` (step-ordered); `:198` `check_secret_key`; `:212` `check_threshold` (pushes at the quorum node's own `path` — C1 evidence); `:223` `check_hashlock`; timelock field rules in `validate_fields` (`older` `:163-170`, `after` `:171-175`); `:338-364` `localize()` post-order (RepeatedKeys lands on the deepest exhibiting subtree — same quorum node).
- `crates/mnemonic-toolkit/src/descriptor_builder/schema.rs` — `:13` `SPEC_SCHEMA_VERSION=1` (+ its bump rule comment); `:46` `spec_schema_json`; `:71` `spec_schema_string`; self-test `grammar_matches_node_kinds_hand_list` (`:80` region).
- `crates/mnemonic-toolkit/src/cmd/build_descriptor.rs` — `:23` `BuildDescriptorArgs` (5 flags); `:63` `run`; `:70` `--spec-schema` short-circuit; `:75` unconditional `read_spec` call (the I1 dispatch-order hazard); `:91-114` `read_spec` (TTY error / stdin-to-EOF); `:82`/`:116` `emit_diagnostics` (`:123` `json!` → serde_json::Value → alphabetical keys); `:194-203` network→human-view address only.
- `crates/mnemonic-toolkit/tests/cli_build_descriptor.rs` — `:15` test-local `Archetype`; `:22` `ARCHETYPES` table; `:60` `archetype_descriptor_goldens`; `:220` `negative_discrimination_mutated_threshold_breaks_golden`.
- Fixtures: `crates/mnemonic-toolkit/tests/fixtures/descriptor_builder/{simple-timelocked-inheritance,decaying-multisig,kofn-recovery,tiered-recovery,hashlock-gated}.{json,descriptor,bip388}` — shapes transcribed in §6 from the live files.
- GUI (lockstep reference): `mnemonic-gui/src/schema/mod.rs:114` `FlagKind` (`Text`/`Number{min,max}`/`Dropdown`/`Boolean`/`Path{stdio_sentinel}` …); `:58`/`:73` `repeating: bool`; `mnemonic-gui/src/schema/mnemonic.rs` `BUILD_DESCRIPTOR_FLAGS` (v0.29.0).
- Manual: `docs/manual/src/40-cli-reference/41-mnemonic.md:3878` `#mnemonic-build-descriptor`; synopsis `:3897-3898`.

## 11. Risks the implementation must hold the line on

1. **No second validation path** — producer checks are exactly §3.1's three categories; any gate-rule duplication (esp. the tempting k≤n) is a review-blocking defect.
2. **Canon integrity** — layer-1 tests compare against `SpecDoc::parse(fixture)`, never captured producer output; fixtures immutable this cycle.
3. **Key-order preservation** — argv → `MultiSpec.keys` untouched; discriminating test mandatory.
4. **Spec-mode byte-stability** — the `flag: Option<String>` serde addition must leave existing spec-mode `--json` output byte-identical (self-test (c)).
5. **Schema-version discipline** — both version constants stay 1; the archetypes key is additive and registry-generated.
6. **GUI/manual locksteps** — 11 flag names (schema_mirror, lagging gate — the paired-PR FOLLOWUP is the leading discipline) + full manual lint pre-push.

---

## Fold log

- **R0 round 2 (GREEN 0C/0I, 2026-06-09; review persisted at `design/agent-reports/descriptor-builder-presets-r0-r2-review.md`):** all 9 round-1 folds verified RESOLVED (no fold-drift); C1's quorum-mapping claim pressure-tested airtight against `validate_fields` + probed end-to-end (thresh covered; `secret_key` path form confirmed; cross-branch dup → `root` confirmed). 3 new Minors folded: M1 `ParamKind::UnixTime` → `AbsoluteLocktime` + "kind is metadata-only" sentence (§2 — decaying canon's `after(500000)` is a HEIGHT); M2 §8 FOLLOWUP enumeration gains the §5 `archetypes` schema section; M3 §5 wire-key projection pinned (`min_count`→`min`, `ParamKind` snake_case `as_str`). **Gate satisfied — implementation may begin.**
- **R0 round 1 (RED → folded, 2026-06-09; review persisted at `design/agent-reports/descriptor-builder-presets-r0-r1-review.md`):** C1 provenance made kind-aware (`(prefix, Option<DiagnosticKind>, flag)`, longest-prefix + kind-override; quorum-node `SchemaField→--threshold` vs `None→--key`; cross-branch dup → `root` → `flag: None` contractual) — §2/§3.3/§7. I1 preset dispatch pinned BEFORE spec intake (never `read_spec`/stdin) + stdin-contract cells — §1/§7. I2 `DiagnosticKind::Param` moved to Phase 1 (`flag` field stays Phase 2) — §3.3/§9. M1 `expect()`-message mandate on `lower`'s convention-coupled infallibility — §2. M2 repeated-param producer cell pinned to a Vec-typed flag (clap rejects scalar repeats itself) — §7. M3 `--network` accepted-and-ignored under `--emit-spec`, manual states it — §4. M4 GUI FOLLOWUP must enumerate un-projected clap rules + wire-shape additions — §8. M5 byte-stability test pins a NEW literal golden (none pre-exists; serde_json::Value keys are alphabetical) — §7. M6 §1 archetype value list aligned to §2.1 alphabetical order. Citation fix: `Diagnostic` struct at `gate.rs:45` (derive `:44`).
- **Phase 1 review round 1 (YELLOW → folded, 2026-06-09; persisted at `design/agent-reports/descriptor-builder-presets-phase-1-r1-review.md`):** I1 non-vacuity widened to ALL 5 archetypes (per-archetype mutated-param loop — the only layer that catches a lower fn hardcoding a fixture value). M1 §9 errata: Phase 1 = 9 value-param flags (`--emit-spec` is the 10th `requires`-carrying flag, Phase 2). M6 dead `augment_args` line removed from drift test (b). M7 stale `mod.rs` doc (referencing a nonexistent main.rs allow) rewritten. **Carried into Phase 2:** M2 resolver tests must pin a `keys[i]`-path (prefix-semantics) case; M3 `flag: None` cell should cover the decaying intra-`andor[2]` cross-tier dup (or rely on "matches no entry" generality); M4 record the clap-rejects-scalar-repeats note in the test file; M5 pin preset success-path `--json` + `--network` composition cells.
- **Phase 2 review round 1 (GREEN 0C/0I, 2026-06-09; persisted at `design/agent-reports/descriptor-builder-presets-phase-2-r1-review.md`):** all carry-forwards (P1 M2-M5) verified present; provenance resolver, byte-stability, schema section, emit-spec gate-before-print all conform. 4 non-blocking minors recorded, none folded by decision: (1) param-diag human suffix redundant with the message (kept for uniformity); (2) `--spec-schema` "ignores all other inputs" wording slightly overbroad vs clap `requires` evaluation order (pre-existing); (3) release-step residue belongs to the bump commit; (4) unreachable `[` boundary arm in `resolve_flag` is defensive robustness.
