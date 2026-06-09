# R0 review — SPEC_descriptor_builder_presets — round 2
**Verdict: GREEN** (0 Critical / 0 Important; 3 new Minors, all one-sentence folds)

Verified against tracked source at `e6e4a06` (local master == origin/master). All round-1 findings re-checked against the folded text AND against source; new §10 citations read in full; probes below run against the v0.50.0 debug binary and the live suite.

## Round-1 fold verification

**C1 — RESOLVED.** The kind-aware provenance as written in §2 (`&[(&'static str, Option<DiagnosticKind>, &'static str)]`, longest-prefix + kind-beats-catch-all at equal prefix) is internally consistent with §3.3's canonical quorum-node mapping and with every §7 gate-flow-through cell, including the new cross-branch-dup → `root` → flag-ABSENT cell. I pressure-tested the "min_count ≥ 2 ⇒ the only SchemaField at a quorum node is threshold-flavored" claim against `validate_fields` exhaustively — it is airtight:
- Every SchemaField producer pushes at its **own** node's path: `check_threshold` at the quorum node (`gate.rs:149`, `:155`), `check_hashlock` at the hash node (`gate.rs:157-161`), timelock rules at the older/after node (`gate.rs:163-175`). A hash/timelock node can never *be* a quorum node, and in none of the 5 fixture shapes does a hash/timelock node sit *under* a quorum-path prefix (verified against all 5 fixture JSONs: kofn's `older` is at `root.or_d[1].and_v[1]` vs multi at `root.or_d[0]`; tiered-recovery's `older` is at `root.or_i[1].and_v[0].wrap.sub` vs thresh at `root.or_i[1].and_v[1]`; decaying-multisig's three timelocks are all on sibling `andor` arms). So no foreign SchemaField can longest-prefix-match a quorum entry.
- `min_count ≥ 2` (producer-enforced before lowering) makes `check_threshold`'s `n == 0` arm (`gate.rs:213-214`, the one SchemaField at a quorum path that would be `--key`'s fault) unreachable; the surviving `k == 0 || k > n` arm is threshold-flavored. Empirical: thresh `k=5` over 2 subs in the tiered-recovery shape → `schema_field` at `root.or_i[1].and_v[1]` (probe 3) — the mapping covers `thresh` as well as multi/sortedmulti.
- The only diagnostics under a quorum prefix with a LONGER path are `SecretKey` at `path.<kind>.keys[i]` (probe 4 confirms the path form: `root.sortedmulti.keys[0]`) and, for tiered-recovery's thresh, `SecretKey`/`TypeError` at `thresh.subs[i](.wrap.sub)` — both kind-mismatch the `Some(SchemaField)` entry and resolve via the `None` catch-all to the correct key flag.
- Cross-branch dup → `root`: probed end-to-end (probe 2) — `repeated_keys` at `root`, exit 2. The §7 cell matches reality.

**I1 — RESOLVED.** §1's dispatch-order bullet is explicit (branch BEFORE `read_spec`, never touch stdin) and cites the hazard (`build_descriptor.rs:75`, `:91-114` — both verified). The §7 cells are testable as written: assert_cmd's `.assert()` runs with stdin null/closed by default (no harness work needed), and a wrong implementation that routes presets through `read_spec` would read EOF → empty string → `SpecDoc::parse` failure → exit 2 with "spec JSON parse error" (the shipped `unparseable_spec_exit_2` path), failing the cell's exit-0-plus-golden assertion. The TTY case itself is not CI-testable, but the null-stdin cell discriminates the same wrong ordering, and §1 pins the TTY contract in prose. Sufficient.

**I2 — RESOLVED.** §3.3's phasing note and §9's Phase 1/2 split now agree: `Param` variant + `as_str` in Phase 1; `flag` field + annotation pass + flow-through cells + byte-stability (c) in Phase 2. I grepped the SPEC for residual contradictions: §9 Phase 1's "drift self-tests (a)/(b)" vs Phase-2's schema section is handled explicitly ("(a) extended to schema ids" in Phase 2); no §7 cell assigned to Phase 1 needs the `flag` field. The Phase-1 producer-negative discriminator is unambiguous without `flag`: §3.3 pins kind `"param"` + `node_path: "params"` + flag names in `message` — three independent discriminators.

**M1 — RESOLVED.** §2's `expect("--<flag> declared required in ARCHETYPE_REGISTRY for <id>")` mandate folded verbatim; no new error.
**M2 — RESOLVED.** §7 pins the producer cell to `--key`×2 on simple-timelocked-inheritance (Vec-typed, reaches `repeatable: false`) and notes the clap-handled scalar path. Clap's scalar-repeat rejection verified empirically: `--spec a --spec b` → "cannot be used multiple times" (probe 5).
**M3 — RESOLVED.** §4 states `--network` accepted-and-ignored under `--emit-spec` with the correct citation (`build_descriptor.rs:194-203` verified: network feeds only `emit_human`'s address).
**M4 — RESOLVED.** §8 enumerates the mutex, the 10 `requires` edges, the `--emit-spec` conflicts, the wire-shape additions, and the compare-cost precedent. (See new M2 below for one omission the fold did not cover.)
**M5 — RESOLVED.** §7(c) pins a NEW literal golden + flag-key-absent assertion, with the alphabetical-serde note. Probe-confirmed: current diagnostics serialize `kind, message, node_path` (probes 2-4) and `flag` sorts first.
**M6 — RESOLVED.** §1's value list is now alphabetical and matches §2.1 exactly.

## Critical
None.

## Important
None.

## Minor

**M1 — `ParamKind`'s timelock vocabulary misdescribes `--after`'s canonical usage, and the SPEC never states that `kind` is metadata-only.** §2's `ParamKind = Key | Threshold | Blocks | UnixTime | HexDigest` — but the decaying-multisig fixture's `after` is `500000`, a block **height** (< the 500000000 time threshold), so the natural `--after → UnixTime` mapping would make the §5 schema advertise wrong semantics to the GUI (a `Timestamp` widget would generate values the canon never uses; note §1 correctly gives `--after` FlagKind `Number`, not `Timestamp`). Also, since `validate_params` is specified as applicability/arity/decay ONLY (§3.1), `kind` must drive **no** producer validation (hex/timelock checks are the gate's — §3.2); a reader of §2's "drives generic applicability/arity validation AND the `--spec-schema` section" could over-read `kind` into a second validation path. Fix: rename `UnixTime` → something locktime-neutral (e.g. `AbsoluteLocktime`) or bind flags→kinds explicitly, and add one sentence: "`kind` is schema/manual metadata only — no producer check keys off it."

**M2 — §8's GUI FOLLOWUP enumeration omits the §5 `--spec-schema` `archetypes` extension.** The FOLLOWUP lists the `--json` wire-shape additions and the un-projected clap rules, but the new `archetypes` schema section is precisely the surface the GUI wizard cycle is supposed to consume ("the grammar the GUI + presets consume" — `schema.rs:1-2`; brainstorm seam quoted in §5). The GUI currently parses no spec-schema output (verified: `mnemonic-gui` greps show only the `--spec-schema` flag NAME in `schema/mnemonic.rs:3426` and `tests/build_descriptor_schema.rs`), so nothing breaks — but the FOLLOWUP is the only channel, per the SPEC's own rule (§3.3: "never assumed"). Add the `archetypes` key to the §8 enumeration list.

**M3 — §5's JSON sketch uses `"min"` where `ParamSpec` declares `min_count` (and `"kind": "key"` implies a `ParamKind::as_str` not specced).** Both are obvious projections, but the SPEC is otherwise exact about wire keys; one parenthetical ("`min_count` projects as `min`; `ParamKind` gets a snake_case `as_str`") removes the only naming ambiguity in the schema contract.

## Citation audit (new/corrected citations)

All verified by reading the files at `e6e4a06`:
- Fold-log correction `Diagnostic` struct at `gate.rs:45` (derive `:44`) — **OK** (struct keyword on 45, fields 46-48).
- `gate.rs:338-364` `localize()` post-order — **OK** (fn signature 338, closing brace 364; post-order children-first at 343-347, deepest-defect semantics as described).
- `build_descriptor.rs:75` unconditional `read_spec` call — **OK** (exact line).
- `build_descriptor.rs:91-114` `read_spec` (TTY error `:104-108` / stdin-to-EOF `:110`) — **OK** (fn spans exactly 91-114).
- `build_descriptor.rs:123` `json!({ "diagnostics": diags })` — **OK** (exact line; `serde_json::Value` route confirmed → alphabetical keys, probe-confirmed).
- `build_descriptor.rs:194-203` network → human-view address only — **OK** (`network` consumed at 194, address block 198-203; no other consumer).
- §2's implicit requirements on `DiagnosticKind`: derives `Copy` (`gate.rs:52`) and is `pub` — const-table-compatible (`Option<DiagnosticKind>` unit variants are const-constructible); no module cycle (archetype → gate only). **OK**.
- §4's "SpecDoc already derives Serialize" — **OK** (`ir.rs:57`, Serialize + Deserialize; `WrapperKind` `rename_all = "lowercase"` at `:71` ⇒ emitted `"wrapper":"wsh"` value-equals the fixtures).
- §7's claim that no pre-existing golden pins the spec-schema bytes — **OK**: `cli_build_descriptor.rs:114` `spec_schema_dumps_versioned_grammar` asserts exactly three fields (`spec_schema_version == 1`, `node_kinds` contains `"andor"`, `multipath_suffix == "/<0;1>/*"`); `schema.rs` self-tests (`:80`, `:96`) assert field equality + nodes length only. The additive `archetypes` sibling key breaks neither; GUI side pins nothing spec-schema-shaped (only the flag-name set, `mnemonic-gui/tests/build_descriptor_schema.rs:28` vs the v0.50.0 6-flag surface — the already-tracked pin-bump lagging gate).

## Empirical probes run

1. `cargo test -p mnemonic-toolkit --test cli_build_descriptor` → **10 passed, 0 failed** (canon green at `e6e4a06`).
2. Cross-branch duplicate key (simple-timelocked-inheritance shape, same xpub as `--key`/`--recovery-key` would supply) piped to `build-descriptor --json` → exit 2, `repeated_keys` at `node_path: "root"` — the §7 flag-ABSENT cell's expected localization confirmed end-to-end.
3. Tiered-recovery shape with thresh `k=5` over 2 subs → `schema_field` at `root.or_i[1].and_v[1]` (the thresh node's OWN path) — confirms §3.3's quorum mapping covers `thresh`, not just multi/sortedmulti, and that C1's "SchemaField at quorum node = threshold-flavored" holds for the third quorum kind.
4. `sortedmulti` with an xprv key → `secret_key` at `root.sortedmulti.keys[0]` — confirms the `(quorum-path.<kind>.keys, None)` path form in §3.3 matches `gate.rs:151` exactly.
5. `build-descriptor --spec a --spec b` → clap "cannot be used multiple times" usage error — M2's scalar-repeat claim verified on clap 4.6.1 (Cargo.lock).
6. `--spec-schema` dump → 8 top-level keys (`doc_shape, multipath_suffix, node_kinds, node_tagging, nodes, spec_schema_version, supported_doc_schema_version, wrapper`); adding `archetypes` is purely additive against everything that asserts on this output.
7. Probes 2-4 also re-confirm diagnostics keys serialize alphabetically (`kind, message, node_path`) — §7(c)'s golden-pinning note is correct.
