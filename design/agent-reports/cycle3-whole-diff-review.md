# cycle-3 whole-diff execution review

- **Cycle:** cycle-3 GUI funds-safety secret-leak fixes (H2 runner argv-leak + H3 minikey composite-secret routing)
- **Worktree:** `/scratch/code/shibboleth/wt-cycle3-gui` (`mnemonic-gui`), branch `feature/cycle3-gui-secret-leaks`
- **HEAD SHA reviewed:** `2cc9c9fd055a0faffcc394dcf0447a83f0b1b04f`
- **Base:** `0b1e024` (`origin/master`)
- **Reviewer:** opus software architect — mandatory non-deferrable independent adversarial execution review (final code gate)
- **Date:** 2026-06-21
- **Method:** full diff read + cross-check against R0-GREEN plan/spec + toolkit pin (`mnemonic-toolkit-v0.60.0`) source + post-diff re-grep of every `node_type_is_secret`/`SECRET_NODE_TYPES` caller + **mutation testing** (reverted each of the 4 wide→narrow surface swaps and the H2 log, confirmed each regression test turns RED) + full `cargo test` (524 pass) + `cargo clippy --all-targets -D warnings` (clean).

---

## Summary of the diff

Three commits, six source files (one comment-only) + five test files:

- **H2 (`runner.rs:119`)** — replaced `debug!(… argv = ?argv …)` (Debug-formatted the full cleartext argv → leaked phrase/entropy/WIF/minikey to stderr under `--debug`/`RUST_LOG`) with `debug!(… program = %argv[0], argv_len = argv.len(), stdin = …)`. Zero secret bytes.
- **H3** — added `node_type_is_argv_secret` predicate over the WIDE `SECRET_NODE_TYPES_ARGV` (= narrow 8 + `minikey`), and routed the four secrecy surfaces off the narrow `node_type_is_secret`/`SECRET_NODE_TYPES`:
  - (i) argv-mask — `form/invocation.rs:460`
  - (ii) run-confirm — `secrets.rs:246`
  - (iv) persist-redact — `persistence.rs:102` (+ import `:32`, docs `:16`/`:74`/`:94`)
  - (iii) paste-warn — `form/widget.rs:653-673` (new node-aware paste detection on the composite VALUE field, mirroring `SecretLineEdit::show`, gated on `node_type_is_argv_secret(node)`).
- Comment-only edits to `schema/mnemonic.rs:1119` + `:2095` (two `secret: false` lines — value unchanged, only trailing prose).

---

## Verification performed (load-bearing)

1. **Wide−narrow delta = exactly `{minikey}`** — confirmed against the pinned toolkit `git show mnemonic-toolkit-v0.60.0:.../secret_taxonomy.rs`: narrow = `{phrase, entropy, xprv, wif, ms1, bip38, electrum-phrase, seedqr}`; ARGV = same + `minikey`. The new `argv_canonical_fallback::ARGV_SNAPSHOT` is byte-identical to the toolkit's `SECRET_NODE_TYPES_ARGV`.
2. **Every secrecy surface routed to WIDE; no live narrow caller remains** — post-diff grep: the only non-comment reference to `node_type_is_secret` in `src/` is its own definition (`secrets.rs:164`). All four runtime surfaces use the wide predicate. No 5th surface exists (see hunt-item 1).
3. **`minikey` is reachable only via the `convert --from` `NodeValueComposite`** (`schema/mnemonic.rs:151`) — it is NOT a slot subkey (`SECRET_SLOT_SUBKEYS` has none; no slot `*_FROM_NODES` set includes it) and not a separate positional. So the composite path is the complete reachability set; the plan's "slot-field-paste-warn STAYS OPEN — no minikey path" claim is correct.
4. **Both persistence write paths covered** — autosave (`save_if_changed`→`serialize_redacted`→`redact_persisted_state`→`redact_for_persistence`) and on-exit (`main.rs:1138` `save()`→ same chain) funnel through the single widened `redact_for_persistence`. Single chokepoint; both drop minikey.
5. **Mutation test (non-vacuity proof)** — reverting all four surfaces wide→narrow turned RED exactly: `surface_i_minikey…masked`, `surface_ii_minikey…run_confirm`, `surface_iv_minikey…dropped_from_redacted_form`, `surface_iv_minikey_value_absent_from_on_disk_state_json` (panic printed the literal plaintext minikey leaking to state.json), and `surface_iii_minikey_over_threshold_paste_raises_warn`; all three xpub negative controls stayed GREEN. Reverting `runner.rs` to `argv = ?argv` turned `runner_no_argv_leak` RED with the panic printing the leaked `SENTINEL_SEED` token — proving the negative assertion sits behind the capture gate (no false-GREEN). Worktree restored pristine afterward (`git status` clean).
6. **Full suite + lint** — `MNEMONIC_BIN=/tmp/mnemonic-v0600/bin/mnemonic cargo test` = **524 passed / 0 failed / 1 ignored** (env-gated); `cargo clippy --all-targets -- -D warnings` clean. `schema_mirror` (21) + `schema_mirror_secret_drift` (1) GREEN — the comment-only schema edits changed no flag-name / secret-bool / dropdown-value.

---

## Critical

**None.** (0)

No remaining secret-leak surface, no behavioral regression, no broken funds-safety invariant found. H2 emits zero secret bytes; H3 closes all four minikey surfaces and both persistence write paths; mutation testing proves every fix is pinned by a non-vacuous test.

---

## Important

**None.** (0)

Two items considered and explicitly cleared:

- **Cargo.toml version is still `0.44.0`; plan §SemVer calls for MINOR `0.45.0`.** This is NOT a code-correctness defect and NOT a leak — it is a release-ritual step performed at the ship commit (alongside README/CHANGELOG), out of scope for the three feature commits under review. It is called out here as a **ship-checklist reminder, not a review finding**: the ship commit MUST bump `0.44.0 → 0.45.0` and update the GUI README/CHANGELOG version sites, per the standing release-ritual (MEMORY `project_toolkit_release_ritual_version_sites`). Does not block GREEN of the *diff*.
- **Persistence widened to drop `minikey`, which the toolkit's own `SECRET_NODE_TYPES` doc-comment says persistence "uses the narrower set."** Adversarially examined: this is a *deliberate, plan-R0-sanctioned over-redaction* (IMPLEMENTATION_PLAN Phase 2 §"Over-redaction check folded"). `minikey` is a Casascius mini **private key**; dropping it from `state.json` is the funds-safe direction. The wide−narrow delta is exactly `{minikey}`, and nothing argv-secret-but-persist-safe exists, so wide == correct for persistence. The toolkit comment is a historical narrow-set note, not a constraint the GUI must obey. Correct.

---

## Minor

**M1 — stale block-comment in `form/invocation.rs:389`.** The diff updated the inline comment at `invocation.rs:451-454` (and the module doc at `:147-150`) to say `node_type_is_argv_secret`, but the earlier block comment above `emit_one` still reads:
```
// ... Its bit is `flag_is_secret(flag) ||
// node_type_is_secret(node)` — covering both the secret flag `--share` and the
```
The actual code at `:460` now calls `node_type_is_argv_secret(node)`. Comment-only documentation drift; no behavioral effect.
**Required fix (non-blocking, fold-at-ship):** change `node_type_is_secret(node)` → `node_type_is_argv_secret(node)` in the `invocation.rs:389` block comment. Does not gate ship; recommend folding into the ship commit for hygiene.

---

## Adversarial hunt-item dispositions

1. **Remaining minikey/other leak surface (5th surface):** CLEAN. Post-diff, the only non-comment narrow-predicate reference is the definition itself; all four surfaces route wide; both persistence write paths (autosave + on-exit) funnel through the single widened `redact_for_persistence`; the copy-command Preview + run-confirm modal both derive from the widened `assemble_argv_with_secret_mask` mask (`any_secret = mask.iter().any`), so minikey is masked in Preview and labelled "reveals secret" on the copy button. The "reveals secret" copy is the documented informed-reveal (deferred per FOLLOWUPs), not a regression. minikey is `convert --from`-only — no slot/positional path.
2. **H2 correctness:** CLEAN. `argv[0]` index guarded by the `argv.is_empty()` early-return (`:112`); `%argv[0]` is Display of the GUI-resolved binary path (never user-secret-controlled); `argv_len`/`stdin.is_some()` carry no secret bytes; `run_with_stdin` signature + `RunResult` unchanged; `warn!`/exit-code paths (`:153`, `:176-178`) are argv-free; no other Debug-format of `argv`/`RunResult` anywhere in `src/`.
3. **Paste-widget wiring:** CLEAN. Faithful mirror of `SecretLineEdit::show` — same global `ui.input` event scan, same `response.changed()` attribution gate (correctly attributes the paste to the focused field; egui semantics mean a paste into another field does not change this TextEdit), same `paste_warn_id()` bus flag (single chokepoint the existing modal consumes — no parallel/never-consumed flag), same threshold. Adds the node-aware `node_type_is_argv_secret(node.as_str())` gate so xpub/path/fingerprint/mk1/address do NOT over-warn (negative control GREEN). No multi-row double-fire reachable (≤1 composite flag per subcommand) and `response.changed()` would discriminate even if two coexisted. No borrow/lifetime hazard (`node` read-only via `as_str()`).
4. **Persistence redaction:** CLEAN. Both autosave (`save_if_changed`) and on-exit (`save`) route through `serialize_redacted`→`redact_persisted_state`→`redact_for_persistence`; the on-disk-bytes test proves the real serialized `state.json` contains neither the minikey value nor the `"minikey"` node token. No over-redaction: xpub negative control survives both in-memory and on-disk. Wide−narrow delta verified == `{minikey}`.
5. **Test integrity (non-vacuous):** CLEAN. `h3_minikey_paste_warn.rs` queries `Role::TextInput` (the plain composite value field), NOT `PasswordInput` — correct, single unambiguous TextInput in harness. `h3_minikey_secret_surfaces.rs` `surface_iv` asserts on REAL `save()`d state.json bytes for the minikey-absent claim. H2 test's negative assertion sits behind the `"subprocess spawn"` capture gate with an `argv_len` positive control (no false-GREEN) + 3-attempt interest-cache retry. Mutation testing confirmed each test turns RED when its fix is reverted; pinning map: paste→`surface_iii_minikey_over_threshold…`, argv→`surface_i_minikey…masked`, confirm→`surface_ii_minikey…run_confirm`, persist→`surface_iv_minikey…(both)`, H2→`runner_no_argv_leak`.
6. **Regression to the 8 existing secret nodes:** CLEAN. Wide ⊇ narrow (drift-guard test `secret_node_types_argv_superset_of_narrow`); the 8 narrow nodes are all in the wide set, so every previously-secret surface stays secret. Full suite 524/524; clippy `-D warnings` clean.
7. **Deviation #2 (`tests/persistence.rs` `:161` audit loop + `:6`/`:317` docs narrow→wide):** SOUND. The audit loop is an **absence** assertion (`!on_disk.contains`); iterating the WIDER set is strictly more restrictive — it can only catch more leaks, never weaken. The fixture has only phrase/xpub composites so the minikey iteration is a no-op against THIS state (the genuine minikey on-disk proof is the dedicated `surface_iv` test). No assertion weakened. `xpub` (the negative control) is correctly in neither set so it is not asserted-absent.

---

## Verdict

**WHOLE-DIFF: 0C / 0I**

**GREEN (0C / 0I) — cleared to ship.**

Ship-checklist reminders (not review findings, do not gate GREEN): (a) bump Cargo.toml `0.44.0 → 0.45.0` + GUI README/CHANGELOG version sites in the ship commit; (b) optionally fold the M1 stale-comment one-liner (`invocation.rs:389`) into the ship commit. The full `cargo test` (524 pass) requires `MNEMONIC_BIN=/tmp/mnemonic-v0600/bin/mnemonic` (the stale `$PATH` v0.56.0 false-fails `schema_mirror` — documented pre-existing gotcha, not this diff).
