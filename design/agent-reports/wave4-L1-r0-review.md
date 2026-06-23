## R0 Review — Wave-4 L1 verify-bundle ↔ bundle descriptor-mode dedup spec

**VERDICT: GREEN (0 Critical / 0 Important / 3 Minor). Cleared to proceed to plan-doc + plan-R0.**

This is a funds-path refactor and I scrutinized it as such. Every load-bearing claim was re-grepped against live HEAD and the critical ones were verified *empirically* by running the binary. The spec — already through one fold round — holds up.

### Re-pin / citation integrity
- Spec pinned at `940abe9e`; **HEAD has advanced to `fbce2243`** (Wave-4 L4 manual-docs commit). I verified `fbce2243` touched ONLY `design/FOLLOWUPS.md` (+8 lines at ~:1321) and manual books — **NOT** `bundle.rs` / `verify_bundle.rs` / the test files. So all source citations remain valid against `fbce2243`.
- **`FOLLOWUPS.md:103`** (the OPEN slug) — re-grepped: STILL at line 103 (the post-spec +8 lines landed far below). §6 flip target is correct.
- All bundle.rs anchors EXACT: gate `1387-1402`, probe `1410-1412`, §4.12.g `1414-1421`, §6.6-row-4 `1423-1448`, H12 `1461-1462`, row-19 `1530-1543`, F4 `1550+`, **fp-mismatch refusal `1634`**, helper fns `compute_default_origin_path:2259` / `derivation_path_to_origin:2291` / `origin_to_derivation_path:2344` / `emit_default_path_notice:2313`.
- Helper-type citations all EXACT: `template.rs:11`(Tag import)/`:295`(`bip48_script_type_for_root_tag`), `parse_descriptor.rs:204`(`pub path_decl: PathDecl`), `cmd/mod.rs:5`(`pub mod bundle`), `slot_input.rs:105`(`pub value: SecretString`). `bip48_script_type_for_root_tag` maps Tr→3 / Sh→1 / else→2 — matches the H12 leaf claims.
- Verify-side fully-qualified helper refs EXACT: `verify_bundle.rs:1440`(compute_default_origin_path), `:1496`(derivation_path_to_origin).

### Funds-path boundary — the central question, ANSWERED CORRECTLY
**Does the shared-core vs caller-only boundary keep the bundle refusals + notice OUT of verify and IN bundle, without changing either side's accept/refusal set?** Yes, and I confirmed the mechanism at the source level:

- **D3 (emit-only refusals + notice) never enter the fn** — §4.12.g `--account` refusal (`bundle.rs:1414-1421`), §6.6-row-4 `[Phrase,Path]` refusal (`1423-1448`), and the stderr notice (`1869`) all stay structurally at the bundle call site, *ahead of* the call. Verify never had them. Confirmed they live outside the span being lifted.
- **D2 (row-19 path-mismatch refusal) is the ONLY behavioral knob** — mode-gated on `DescriptorBindMode::Emit` inside the shared fn. Live bundle override loop (`1514-1548`) has the row-19 refuse; live verify loop (`1483-1497`) does NOT. The spec's `if mode == Emit && ...` gate reproduces this exactly.
- **D1 (`defaulted_indices`) cannot leak into verify's output** — the decisive funds check. The shared fn runs `defaulted_indices.push`/`.retain` unconditionally (incl. in Verify mode), which the current verify loop does not. But **verify discards the returned vec** (`let _defaulted = ...`) and `new_paths` VALUES are computed identically. The sole consumed output, `path_decl.paths`, is therefore **byte-preserved** for verify. I diffed the live bundle `new_paths` build (`1467-1496`) against the spec fn body and the verify build (`1445-1464`): the path VALUES are identical; only the discarded bookkeeping differs.

**Could the dedup change verify's accept-set or bundle's refusal-set?** No. Verify's accept-set is a pure function of `path_decl.paths`, which is byte-identical pre/post. Bundle keeps all three caller-side refusals + retains row-19 via Emit mode.

### Empirical verification (ran the binary against live source)
- **Finding-1 fps VERIFIED LIVE:** `bundle ... [73c5da0a/...]@0 [b8688df1/...]@1` → **exit 0** (accepted). `[deadbeef/...]@0` → **`error: --slot @0.phrase derives master fingerprint 73c5da0a but descriptor @0 annotation specifies deadbeef`** (the error literally states the derived fp = 73c5da0a, and b8688df1@1 was accepted). Both fixture fps confirmed; the fp-mismatch refusal at `bundle.rs:1634` is real and would false-RED any inline-origin cell using a wrong fp. The fold is necessary and correct.
- **Finding-2 nested-mk1 VERIFIED LIVE:** n=2 `--json` mk1 is `[[chunk,chunk],[chunk,chunk]]` (2 cosigners × 2 chunks), md1 flat (4), ms1 length-2. A single-sig flat-mk1 harness produced **exit 2 / non-JSON** (mangled). The nested double-loop + every-`--ms1` harness produced **`result: ok`, 15/15 checks passed**. Finding-2's harness mandate AND the §4(2) mandatory self-test are both validated as constructible and load-bearing.
- **Parity matrix realizability VERIFIED LIVE:** all-elided n=2 → `result: ok` 15/15; divergent (both inline, correct fps) → exit 0; **mixed (@0 inline-correct-fp + @1 elided) → `result: ok`, all passed**. The full (root × shape) matrix is constructible.
- **The 2 L24 cells stay GREEN:** baseline `cargo test --test cli_non_canonical_descriptor` = 12/12 pass (incl. both L24 cells); `--test cli_descriptor_mode` = 7/7. The gate cell at `:338` uses a CLEAN-PARSING non-canonical descriptor (verified: probe succeeds), so the verify-side gate-after-probe re-order leaves it green — I reproduced the exact cell input and got the identical `descriptor has n=2 placeholders but --slot vec covers 3 slots` message.

### Error-precedence re-order (the §3.2 finding) — sound
Three re-orders confirmed real (gate-vs-probe, gate-vs-account emit-only, gate-vs-row4 emit-only), all on doubly-malformed input that exits ≠0 either way. **No test pins ANY of the three precedences** — verified: the ONLY gate-message assertion in the suite is `cli_non_canonical_descriptor.rs:372` (inside the verify L24 cell at 338) plus the negative `:412`; that cell sets no `--account`, has no row-4 conflict, and parses cleanly. The NO-BUMP rationale (precedence reshuffle is the sole observable delta; every path stays exit-≠0) is accurate. The §5 retraction of the blanket "zero observable change" is correct.

### Convention / CI-coupling checks
- **No new `ToolkitError` variant** — reuses `DescriptorParse`/`BadInput`/`SlotInputViolation` (all present at the cited sites). Alphabetical-ordering convention does not engage. ✓
- **CI-coupling NONE** confirmed — internal helpers + a `pub(crate)` enum, no clap surface → no `schema_mirror`, no manual flag-coverage, no sibling lockstep. `SecretString` deref is unchanged (no secret-hygiene/lint-floor move). ✓
- **Full-suite mandate** present (§4 test-run discipline cites `feedback_r0_review_run_full_package_suite`). ✓
- **`PathDeclPaths`/`DerivationPath` scope:** both module-imported in bundle.rs (`:1353`, `:353`), so the new fn at ~2249 compiles; verify.rs uses them fully-qualified everywhere (no dead `use` to remove — §3.4's grep-caveat correctly anticipates this; they're still consumed at verify_bundle.rs:1538-1541). ✓

### Minors (no fold required for the gate; the §0 anchor-based re-grep mandate already neutralizes them)
1. Verify-side spans snapshot-decayed a few lines (gate `1386-1412`→`1396-1413`; inference `1428-1510`→`1432-1511`). Edits are driven off stable grep anchors, so this never reaches code.
2. §3.2's "replace 1450-1563" vs "keep 1461-1462" superficial tension — `default_script_type`(1461)/`defaulted_indices`(1463) are outside the `if is_non_canonical` block and survive naturally; the spec already defers recompute-vs-return to the plan.
3. The in-code S-VERIFY stale ref (`bundle.rs:1373-1388`) confirmed still stale; the spec deletes the whole comment block, resolving it.

### Mandate for plan-R0 (carry forward)
- Confirm the surviving call-site `default_script_type` binding + accept the shared-fn internal recompute.
- Re-affirm (adversarially) no test pins gate-vs-probe / gate-vs-account / gate-vs-row4 precedence at plan-write time (re-grep — citations decay).
- Pin the §4(2) self-test (wsh n=2 all-elided green ON CURRENT SOURCE via nested-mk1 harness) as the FIRST-landed cell before any RED is trusted.
- Run FULL `cargo test -p mnemonic-toolkit` + clippy at every gate.

**The shared-core/caller boundary is correct, the dedup cannot change verify's accept-set or bundle's refusal-set, the 2 L24 cells stay green, and the parity matrix is an exhaustive, constructible derivation-desync oracle. GREEN — proceed to plan-doc.**