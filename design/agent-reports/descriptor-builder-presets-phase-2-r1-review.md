# Phase 2 review — descriptor-builder presets — round 1
**Verdict: GREEN** (0C/0I)

## Critical
None.

## Important
None.

## Minor

1. **Param-diag human suffix is redundant** — `crates/mnemonic-toolkit/src/cmd/build_descriptor.rs:273-274` appends ` (from --<flag>)` unconditionally when `flag` is set, and `param_diag` (`archetype.rs:301-308`) always sets `flag: Some` while every §3.1 message already names the flag. Empirically: `[param] params: kofn-recovery requires --older (missing) (from --older)`. Cosmetic only; the structured `--json` field is exactly right and the suffix is genuinely useful on gate diagnostics (`[repeated_keys] root.or_d[0]: ... (from --key)` probed). If you care, suppress the suffix for `kind == Param` in `emit_diagnostics`; equally defensible to leave as-is for uniformity. Non-blocking.

2. **`--spec-schema` "ignores all other inputs" is now slightly overbroad** — manual `docs/manual/src/40-cli-reference/41-mnemonic.md` (the `--spec-schema` row) and the clap doc (`build_descriptor.rs:48-49`): `--spec-schema --emit-spec` (no `--archetype`) is a clap usage error (probed: exit 64), because `requires = "archetype"` is evaluated before `run`'s short-circuit at `build_descriptor.rs:154`. This wording predates Phase 2 (and was already imprecise after Phase 1's nine `requires`-bearing flags), so no action required this cycle; flagging it so it doesn't read as a Phase-2 regression later.

3. **Release-step residue (reminder, not a defect of this commit):** SPEC §9 Phase-2 bullet also lists "release ritual + tag; file the GUI FOLLOWUP pair; flip `descriptor-builder-engine` → resolved" — none of which is in 8730ec8 (correct: it belongs with the v0.51.0 bump commit, task #5). Note the manual header already says "archetype presets v0.51.0" (`41-mnemonic.md:3878`) while `Cargo.toml` is still 0.50.0 — true only once the bump lands; don't ship the manual without it.

4. **Observation, no action:** `resolve_flag`'s `rest.starts_with('[')` boundary arm (`archetype.rs:328`) is unreachable under the current path grammar — every child segment is dot-prefixed (`gate.rs:490-516`: `.kind[i]`, `.kind.keys[i]`, `.thresh.subs[i]`, `.wrap.sub`), and all table prefixes end at a closed segment. Harmless defensive robustness.

## SPEC-conformance + carry-forward checklist

**§4 `--emit-spec`** — CONFORMS.
- Clap semantics: `#[arg(long, requires = "archetype", conflicts_with_all = ["format", "json"])]` (`build_descriptor.rs:103`); both conflicts proven by `emit_spec_conflicts_with_format_and_json` (`tests/cli_build_descriptor.rs:526-539`, loops over both tails) and the suite is green. `--emit-spec` without `--archetype` → clap usage error (probed, exit 64).
- Gate-before-print: dispatch at `build_descriptor.rs:161-210` runs `validate_params` (`:174`) → `gate::validate(&doc)` (`:183`) → only then the `args.emit_spec` branch (`:196-207`) prints **the same `doc`** that was validated (constructed at `:178-182`, borrowed by both). Gate refusal emits diagnostics and returns 2 before the print; probed: stdout is 0 bytes on a `k=5/n=2` refusal, and pinned by `emit_spec_runs_the_gate_before_printing` (`:544-557`).
- Value-equality + pipe-back: `emit_spec_value_equals_fixture_and_round_trips` (`:495-516`) loops ALL 5 `ARCHETYPES`, asserts `serde_json::Value` equality against the fixture `.json` AND byte-equal descriptor on `--spec -` pipe-back. Emitted `schema_version` is 1 (probed).
- `--network` accepted-and-ignored: probed — output byte-identical with/without `--network testnet`; manual states it (the `--emit-spec` flag row).

**§3.3 provenance** — CONFORMS.
- `resolve_flag` (`archetype.rs:317-333`): boundary discipline is sound. The `or_d[1]`-vs-`or_d[10]` worry dissolves on inspection: every table prefix ends with a **closed** bracket or full segment (`archetype.rs:128-137,150-155,167-171,184-187,202-207`), so `"root.or_d[10]".strip_prefix("root.or_d[1]")` fails at the `]`-vs-`0` mismatch — no false match is constructible. The `.`/`[` check guards segment-name collisions (hypothetical `root.or` vs `root.or_d` → rest `_d[0]` rejected). Indices in the real trees are 0–2 anyway.
- Kind-override ordering: `max_by_key(|(prefix, k, _)| (prefix.len(), k.is_some()))` (`:331`) — lexicographic tuple gives prefix-length priority, kind-specificity only at equal length. That is exactly SPEC §3.3/§2's rule. A shorter kind-specific entry can never compete with a longer catch-all in the actual tables (kind-overrides exist only at quorum paths that also carry the equal-length catch-all). Non-matching `Some(kind)` entries are excluded by the filter (`:329`), so ties between two surviving entries are impossible per-table. I traced every gate diagnostic source against every table: `check_threshold` SchemaField at the quorum path → kind-override (`--threshold`/`--recovery-threshold`); SecretKey at `path.<kind>.keys[i]` (`gate.rs:165`) → quorum catch-all via prefix; older/after/hashlock SchemaField at their own node paths → exact-match catch-alls; TypeError localizing to a quorum/pk node → catch-all (correctly key-flavored, since `localize_parse_failure` can't descend below `child_paths`, `gate.rs:332-345,490-516`). The SPEC's "only SchemaField at a quorum node is threshold-flavored" claim holds: `check_threshold`'s `n==0` branch (`gate.rs:229`) is unreachable under `min_count ≥ 2`/required params, and `check_secret_key` is the only deeper-path emitter and uses `SecretKey`.
- All 6 `Diagnostic` construction sites in gate.rs carry `flag: None` (`gate.rs:222,291,326,424,531,539`); `param_diag` carries `Some` (`archetype.rs:306`); no other construction sites exist (grepped).
- Byte-stability golden: `spec_mode_json_diagnostics_byte_stable_no_flag_key` (`tests/cli_build_descriptor.rs:684-699`) — current binary's output byte-matches the golden (probed independently: `kind, message, node_path` alphabetical, raw UTF-8 `≤`, 2-space pretty, trailing newline). Matches the pre-change binary by construction: `emit_diagnostics` routes through `json!`→`serde_json::Value` (BTreeMap, alphabetical keys) exactly as before, and the new `flag` field is `skip_serializing_if = "Option::is_none"` (`gate.rs:53`) with spec mode never setting it (`build_descriptor.rs:216-222` does not annotate).

**§5 schema** — CONFORMS. `archetypes` generated FROM `ARCHETYPE_REGISTRY` (`schema.rs:59-78`), no hand copy; wire keys `flag/kind/required/repeatable/min` with `min_count → min` and snake_case `ParamKind::as_str` (`archetype.rs:47-55`) per R0-r2 M3; `SPEC_SCHEMA_VERSION` untouched at 1; drift test `schema_archetypes_match_registry` (`schema.rs:135-156`) pins ids + every projected param, and combined with `registry_ids_match_cli_archetype_variants` (`build_descriptor.rs:403-412`) gives CLI == registry == schema id-sync; Release-A `spec_schema_dumps_versioned_grammar` (`tests/cli_build_descriptor.rs:154-163`) is field-asserting, still green; integration cell `spec_schema_carries_archetypes_section` (`:732-774`) additionally spot-pins kofn `--key`. Module graph: schema→archetype→{gate,ir}, gate→ir — acyclic (and intra-crate anyway).

**Producer diagnostics** — `param_diag` sets `flag: Some` (verified end-to-end by `producer_diagnostics_carry_flag_and_human_suffix`, `:653-680`); decay-ordering picks `RECOVERY_OLDER` (`archetype.rs:287-294`) — sensible (the tier-2 value is the one violating "must exceed"), message names both flags. Human suffix redundancy → Minor 1.

**Carry-forwards** — ALL PRESENT.
- M2 `keys[i]` prefix: integration xprv case asserting `node_path == "root.or_d[0].multi.keys[0]"` + `flag == "--key"` (`tests/cli_build_descriptor.rs:626-646`) AND unit cell (`archetype.rs:662-665`). ✓
- M3 BOTH flag-absent cells: cross-branch root + decaying intra-`andor[2]` (`cross_branch_duplicates_carry_no_flag`, `:565-606`, asserting the key is ABSENT, not null) + unit cells (`archetype.rs:671-678`). ✓
- M4 clap-scalar-repeats note recorded at `archetype.rs:683-685`. ✓
- M5 success-path `--json`/`--network` composition (`preset_success_json_and_network_compose`, `:704-726`: envelope fields + `tb1q` testnet address). ✓

**Manual** — accurate vs `--help` (ran the binary; all 11 flag rows, value placeholders `<KEY>/<THRESHOLD>/…/<HASH>`, archetype kebab values, and conflict/ignore semantics match verbatim); archetype table matches the fixture shapes in §6 and the `ARCHETYPES.preset_args` (`tests/cli_build_descriptor.rs:33-95`); `--after` BIP-65 height-vs-time claim matches the clap doc (`build_descriptor.rs:90-92`) and the locktime-neutral `ParamKind::AbsoluteLocktime` stance; section appears exactly ONCE (`grep -c '### Archetype presets'` = 1; `{#mnemonic-build-descriptor}` = 1 — the 23× mishap is not in the committed file); cspell adds only `timelocked/kofn/hashlock` (sane).

**Suites/locksteps** — full `cargo test -p mnemonic-toolkit`: 939 unit + 31 integration (cli_build_descriptor) + all other targets, 0 failed. `cargo clippy --all-targets`: exit 0, zero warnings (dead_code allows removed at `archetype.rs:47,69,84` — all three consumers now wired). No fixture diffs in the commit (7 files, none under `tests/fixtures/`). No mnemonic-gui edits; toolkit-side gui-schema tests pin only the subcommand NAME list (`tests/cli_gui_schema.rs:79-`) and `build_subcommand_conditional_rules` still has no build-descriptor arm (`gui_schema.rs:336-345`) — the 11 new flags flow through introspection un-pinned, as planned. Secrets taxonomy: `flag_is_secret` (`secrets.rs:49-64`) correctly excludes `--key`/`--recovery-key`/`--final-key`/`--hash` — challenged and upheld: keys are extended PUBLIC keys by contract (the gate's xprv screen at `gate.rs:212-225` enforces watch-only-out, so an xprv in argv is refused, and `cli_secret_in_argv_warning` covers inline-secret advisories elsewhere); `--hash` is a SHA-256 **digest**, not a preimage — it commits to nothing recoverable and ends up in the public descriptor anyway. `--threshold` was already in the non-secret test list (`secrets.rs:104`).

## Empirical probes run
1. `cargo build -p mnemonic-toolkit` — clean.
2. `cargo test -p mnemonic-toolkit` (full) — all green (939 unit; 31 cli_build_descriptor integration).
3. `cargo clippy -p mnemonic-toolkit --all-targets` — exit 0, zero warnings.
4. `make -C docs/manual lint` with all 4 pinned binaries — all 6 gates OK (markdownlint 0, cspell 0, lychee 0 errors, flag-coverage, glossary, index).
5. `--emit-spec` vs `--emit-spec --network testnet` — byte-identical (accepted+ignored).
6. `--emit-spec` without `--archetype` → clap usage error, exit 64; `--spec-schema --emit-spec` likewise (Minor 2); `--spec-schema` alone exit 0.
7. Gate-refused `--emit-spec` run (`k=5,n=2`) → exit 2, stdout 0 bytes.
8. Human renders: gate diag `... (RepeatedPubkeys) (from --key)`; param diag `... requires --older (missing) (from --older)` (Minor 1 evidence).
9. Spec-mode `--json` failing-spec probe — byte-matches the test golden; no `flag` key.
10. `build-descriptor --help` dumped and cross-checked row-by-row against the manual table.
