# cycle-7 whole-diff execution review (M8 build-descriptor extra-suffix; L23 ecies zero-scalar)

- **HEAD:** `e394a88b` (`fix(cycle7-l23): reject zero scalar in ecies_decrypt_message`)
- **Base:** `d6398b57` (`origin/master`)
- **Commits reviewed:** `4ec38f78` (M8), `e394a88b` (L23)
- **Worktree:** `/scratch/code/shibboleth/wt-cycle7` (`mnemonic-toolkit`), branch `feature/cycle7-m8-build-descriptor`
- **Date:** 2026-06-21
- **Reviewer:** opus software architect (mandatory independent adversarial execution review, pre-tag)
- **Diff scope:** 3 files — `descriptor_builder/gate.rs` (+20), `electrum_crypto.rs` (+33/−3), `tests/cli_build_descriptor.rs` (+396, 7 new tests).

---

## Funds item under review

A build-descriptor key carrying an EXTRA derivation suffix (`xpub…/5`, `/5/6`, `/0h`, `/<0;1>`, `/*`) must FAIL CLOSED. Before the fix, the renderer (`ir.rs::with_multipath` = `format!("{key}{MULTIPATH_SUFFIX}")`, `MULTIPATH_SUFFIX = "/<0;1>/*"`) blind-concats the receive/change suffix onto the raw key, so `xpub…/5` rendered to `xpub…/5/<0;1>/*` — a DEEPER, wrong subtree — and was ACCEPTED by `Descriptor::from_str`. Funds would land on addresses the holder's backup does not derive. The fix adds an `else if key_part.contains('/')` arm in `check_secret_key` (gate.rs:359), after the `[origin]` strip + xprv screen, emitting a `SchemaField` `field_diag` (no key echo, no `--json` delta).

---

## Critical

**None.**

## Important

**None.**

## Minor

**M-1 (informational, NOT a defect) — `check_secret_key` does not detect a derivation tail that the descriptor grammar places *before* the final `]`.**
File: `gate.rs:348` (`let key_part = key.rsplit(']').next().unwrap_or(key);`).
`rsplit(']').next()` takes only the substring after the LAST `]`. A crafted input such as `[a][b/c]xpub…` or `xpub…/5]xpub…` has a `/` that lives before the final `]`, so `key_part.contains('/')` is false and the M8 arm does not fire. **This is not a bypass:** there is no descriptor-key grammar in which a derivation tail before a `]` survives `Descriptor::from_str` AND resolves to a deeper subtree once the `/<0;1>/*` suffix is appended. Verified empirically through the real CLI: `[a][b/c]xpub…` is refused at Step-2 with `type_error: master fingerprint should be 8 characters long` (exit 2). The funds bug requires the smuggled tail to appear *after* a valid `[origin]` and before the appended suffix — i.e. `[origin]xpub…/5` — which ALWAYS yields a `/` after the last `]` and is therefore caught. No action required; recorded only for completeness of the adversarial sweep.

**M-2 (informational) — the per-archetype `flag` annotation for a quorum key is `--threshold`, not `--key`.**
File: `archetype.rs::resolve_flag` (archetype.rs:395) + provenance tables. For a quorum archetype (`decaying-multisig`) the M8 `SchemaField` diagnostic at a `…multi.keys[i]` node resolves its `flag` annotation to `--threshold` (the kind-specific provenance override wins the `max_by_key((prefix.len(), k.is_some()))` tiebreak over the catch-all `--key`). This is a provenance-system artifact, not a defect: the diagnostic `node_path` and `message` still name the offending key node, and **exit 2 fires regardless of the `flag` value**. Already pinned by the new test `m8_preset_flag_provenance_per_archetype` (asserts BOTH classes: single→`--key`, quorum→`--threshold`, plus that `node_path` contains `keys[`). Matches the plan (plan-R0 I-1). No action.

---

## Adversarial findings — evidence

### 1. Guard is load-bearing (mutation proof)
Reverted the predicate (`} else if key_part.contains('/') {` → `} else if false {`), rebuilt:
- 5 M8 tests went RED: `m8_preset_single_sig_extra_suffix_refused_exit2`, `m8_spec_intake_pk_and_multi_extra_suffix_refused_exit2`, `m8_multi_segment_and_hardened_suffix_refused_exit2`, `m8_nested_extra_suffix_refused_exit2`, `m8_preset_flag_provenance_per_archetype`.
- 2 stayed GREEN *correctly*: `m8_trailing_wildcard_still_refused_exit2` (the `/*` tail is independently caught at Step-2 by `InvalidWildcardInDerivationPath`; the test is deliberately step-agnostic) and `m8_positive_control_normal_key_still_builds_both_paths` (over-rejection guard, unaffected).
- **Funds-bug demonstrated** with the guard off: `{"pk":"xpub…/5"}` → `wsh(pk(xpub…/5/<0;1>/*))#nad29qlh`, exit 0 — a silent wrong (deeper) subtree. With the guard ON: exit 2, `schema_field` refusal, no descriptor.
- Mutation reverted; working tree confirmed clean (`git diff HEAD` empty, `git status --short` empty).

### 2. No wrong-subtree key bypasses the guard
- **Single gate, single path.** `validate_with_allow` (gate.rs:157) runs Step-1 `validate_fields` FIRST and returns `Err` on any field diagnostic BEFORE Step-2 render/`from_str` (gate.rs:164–167). The `ValidatedPolicy` (carrying the descriptor) is producible ONLY by an `Ok` return; `emit` consumes only `&ValidatedPolicy`. There is no render/emit path that skips Step 1.
- **Both intake paths converge.** `cmd/build_descriptor.rs`: preset path (line 287) and `--spec` JSON path (line 326) both call `gate::validate_with_allow`; each `emit`s only on `Ok`. `archetype.rs` header confirms "same validation gate … No second validation path." `--emit-spec` (line 305) also runs only post-gate. `--spec-schema` short-circuit emits no key.
- **Every key-bearing node is checked.** Key variants are exactly `Pk`, `Pkh`, `Multi`, `Sortedmulti`. `validate_fields` (gate.rs:232) calls `check_secret_key` for `Pk`/`Pkh` (line 235) and per-key for `Multi`/`Sortedmulti` (line 240). Combinators (`AndV`/`OrD`/`OrI`/`OrB`/`Andor`/`Thresh`/`Wrap`) hold NO keys directly; their children are reached by the unconditional `child_paths` recursion (gate.rs:333) — no early return in the match. `m8_nested_extra_suffix_refused_exit2` pins refusal under `and_v` / `andor` / nested `thresh` at depth.
- **Predicate coverage.** A 15-case standalone probe of `rsplit(']').contains('/')` (incl. SLIP-132 zpub, multi-bracket, hardened, multipath, wildcard tails) returned 0 mismatches against the expected reject/accept oracle. CLI cross-check confirmed `xpub…/5` → exit 2 `schema_field`. The only non-rejecting weird shapes (M-1) cannot produce a valid deeper-subtree descriptor (rejected at Step 2). **No bypass.**

### 3. No legit key over-rejected
- `m8_positive_control_normal_key_still_builds_both_paths` builds successfully for: a bare xpub, a `[origin]xpub`, a `--spec` Pk, and a `--spec` Multi — across BOTH preset `--key` and `--spec` paths.
- CLI cross-check: bare xpub → `wsh(pk(xpub…/<0;1>/*))#5aalkqeh`, exit 0. `[origin]xpub` (origin path inside brackets, body bare) builds.
- `rsplit(']')` on a bracket-less key returns the whole string (no `]` → single fragment), and a legit bare xpub body has zero `/`, so the guard never trips a valid key. SLIP-132 prefixes (zpub/ypub) are prefix swaps with no `/` → unaffected. **No over-rejection.**

### 4. Annotation reality
- Per-archetype `flag`: single→`--key`, quorum→`--threshold` (M-2; pinned by `m8_preset_flag_provenance_per_archetype`). `node_path` + `message` always name the offending key node; exit 2 fires regardless.
- **No key-material leak.** The message is `"{kind} key carries an extra derivation path; …"` where `{kind}` is the node kind (`pk`/`multi`), never the key body. New tests assert the xpub body appears in NEITHER stderr, stdout, NOR `--json` for every refusal case.

### 5. L23 — zero-scalar guard
- `electrum_crypto.rs:350–352`: `if privkey.iter().all(|&b| b == 0) { return Err(InvalidScalar); }` fires BEFORE `Scalar::from_be_bytes` (353) and BEFORE `mul_tweak(&secp,&scalar).expect(…)` (358–360). The `.expect` is now a provable invariant (scalar ∈ [1, n−1]).
- **Reuses** the pre-existing `EciesDecryptError::InvalidScalar` (declared line 247, already used by `derive_storage_eckey` at line 310) — NO new enum variant, NO `Display` arm change.
- **Latent / not CLI-reachable.** The sole non-test in-tree caller is `ecies_decrypt_storage` (line 402), whose `privkey` comes from `derive_storage_eckey(password)?` (line 401) which already rejects zero (line 309). Guard is robustness for a future/downstream caller of the `pub fn`. Test `ecies_decrypt_message_zero_scalar_typed_error_not_panic` (line 811) pins typed-error-not-panic with a valid BIE1 blob + all-zero privkey. Valid (nonzero) scalars unaffected — all KAT decrypt tests still GREEN.

### 6. No regression / no `--json` / schema_mirror delta
- `DiagnosticKind::SchemaField` (gate.rs:99) + `as_str → "schema_field"` (gate.rs:124) are PRE-EXISTING. The M8 reject reuses `field_diag` (gate.rs:699) → no new `DiagnosticKind` discriminant → **no `--json` wire-shape change**.
- Diff touches `src/cmd/` not at all; no new `#[arg(…)]`, subcommand, or `ValueEnum` dropdown → **no `schema_mirror` trigger, no manual-mirror obligation**.
- **Full suite:** `cargo test -p mnemonic-toolkit` → 3358 passed / 0 failed / 15 ignored (matches the implementer claim). `cargo clippy --all-targets -- -D warnings` → clean.

---

## Verdict

**CYCLE-7 WHOLE-DIFF: 0C / 0I** — GREEN (0C/0I, cleared to tag).

The M8 guard correctly fails-closed on every extra-derivation-suffix key across both intake paths and all key-bearing node positions at any depth; mutation-proven load-bearing; no wrong-subtree bypass; no legit-key over-rejection; no key-material leak. L23 hardens a latent panic into a typed error reusing an existing variant with no CLI reachability and no behavioral change for valid scalars. No `--json`/schema_mirror delta. 3358/0/15, clippy clean.
