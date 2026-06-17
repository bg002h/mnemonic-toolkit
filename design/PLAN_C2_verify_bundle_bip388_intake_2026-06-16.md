# PLAN — C2: verify-bundle BIP-388 wallet-policy intake (2026-06-16)

> Tier-2 item C2 from `design/PLAN_remaining_open_items_tiered_2026-06-16.md`.
> Resolves FOLLOWUP `verify-bundle-bip388-policy-intake` (FOLLOWUPS.md:4168).
> **Source SHA: toolkit `origin/master` `a69a9e3`** (all citations below grep-verified
> against this SHA at write time). MINOR (new accepted input FORMAT on the existing
> `--descriptor` flag → new capability; v0.49.0 precedent). Toolkit git-tag only, no publish.
> This plan-doc is the R0 gate: **no code until R0 → GREEN (0C/0I)**.

---

## Gap

`bundle --descriptor` and `export-wallet --descriptor` auto-detect a leading-`{` BIP-388
wallet-policy JSON and expand it to a concrete descriptor before processing (shipped v0.49.0
via the shared `wallet_import::pipeline::expand_bip388_policy`). `verify-bundle --descriptor`
has **no such probe** — it routes a raw policy JSON straight into `classify_descriptor_form`,
which sees BOTH the `@N` template placeholders (`description_template` field) AND the inline
`[fp/path]xpub` keys (`keys_info` field) and refuses with exit 2 "descriptor mixes @N
placeholders with inline keys". So a user who can `bundle` from a wallet-policy JSON cannot
`verify-bundle` against the same JSON. **Intake asymmetry, not a funds bug.**

This refusal is currently PINNED as a contract cell:
`tests/cli_verify_bundle_hashlock_and_bip388.rs::verify_bundle_refuses_bip388_policy_json`
(`:87`), whose own docstring (`:85`) states the `verify-bundle-bip388-policy-intake` FOLLOWUP
"would flip this cell red→green". **This plan does that flip.**

## Citations (grep-verified @ `a69a9e3`)

| Surface | Location |
|---|---|
| verify-bundle descriptor-mode entry | `src/cmd/verify_bundle.rs::descriptor_mode_verify_run` `:662` |
| descriptor string read (insertion point) | `verify_bundle.rs:676-685` |
| bare-concrete fork (`classify_descriptor_form`) | `verify_bundle.rs:687-714` (call at `:694`) |
| shared shape-sniff | `pipeline.rs::is_bip388_policy_shape` `:172` (`s.trim_start().starts_with('{')`) |
| shared expander | `pipeline.rs::expand_bip388_policy` `:187` (returns concrete descriptor String) |
| `classify_descriptor_form` | `pipeline.rs:132` |
| `is_at_n_form` | `pipeline.rs:115` |
| bundle call site (mirror target) | `bundle.rs:319-320` (`is_bip388_policy_shape(&body)` → `expand_bip388_policy(&body)?`) |
| export-wallet call site (name-preserving variant — NOT mirrored here) | `export_wallet.rs:434-436` (also calls `bip388_policy_name` `:435`) |
| pinned-refusal cell to invert | `tests/cli_verify_bundle_hashlock_and_bip388.rs:87` |
| v0.49.0 positive policy-intake tests (mirror source) | `tests/cli_bip388_policy_intake.rs` bundle watch-only cells `:251` (multisig), `:269` (singlesig) |
| v0.49.0 negative `@N`-beyond-`keys_info` cell (cell-3 mirror) | `tests/cli_bip388_policy_intake.rs:230` `export_wallet_bip388_policy_at_n_beyond_keys_info_refused` |
| watch-only multisig verify-bundle-via-bundle-json precedent (cell-1 template) | `tests/cli_verify_bundle_multi_cosigner_mk1.rs:94` `audit_i10_same_xpub_two_paths_2of2_round_trips` (no `--mk1`/`--md1`; envelope supplies cards; `--bundle-json` conflicts_with ms1/mk1/md1 — verify_bundle.rs:76,82,89,96) |
| manual verify-bundle `--descriptor` row | `docs/manual/src/40-cli-reference/41-mnemonic.md:517` |
| FOLLOWUP entry | `design/FOLLOWUPS.md:4168-4177` |

## Implementation (the ~6-line insertion)

Insert into `descriptor_mode_verify_run`, **between `verify_bundle.rs:685` and `:687`** —
immediately after the `descriptor_str` is read, BEFORE the bare-concrete fork's
`classify_descriptor_form` call. **Mirror `bundle.rs:319-320`, NOT `export_wallet.rs`** —
verify-bundle is read-only / verify semantics; it does not round-trip the policy NAME (the
name-preservation that `export_wallet.rs:435` does via `bip388_policy_name` is irrelevant to a
verifier and would be dead code here).

```rust
// BIP-388 wallet-policy intake (mirror bundle.rs:319): a leading-`{`
// policy JSON expands to a concrete descriptor BEFORE classify — a raw
// policy trips BOTH classify's @N and key-regex probes (the v0.49.0
// ordering invariant). verify-bundle is read-only → no policy-name
// preservation (unlike export-wallet).
let descriptor_str = if crate::wallet_import::pipeline::is_bip388_policy_shape(&descriptor_str) {
    crate::wallet_import::pipeline::expand_bip388_policy(&descriptor_str)?
} else {
    descriptor_str
};
```

After expansion the template is `@N`-free concrete (each `@N/**` → `keys_info[N]/<0;1>/*`),
so `classify_descriptor_form` → `Concrete` → `descriptor_concrete_to_resolved_slots` →
`verify_emit_from_expected` — the SAME path a bare concrete descriptor already takes
(`verify_bundle.rs:694-713`). No other code path changes. `expand_bip388_policy`'s own
malformed-input + `@N`-beyond-`keys_info` errors (`pipeline.rs:187-206`) surface as the loud
`ToolkitError::BadInput` / `DescriptorParse` they already are.

**Scope guard:** only the descriptor-mode path (`descriptor`/`descriptor_file`) gains this; the
template-mode path is untouched. The expansion runs identically for `--descriptor` (string) and
`--descriptor-file` (file) because both funnel through `descriptor_str` at `:685`.

## TDD — tests are the deliverable (write first, prove non-vacuity)

All in `tests/cli_verify_bundle_hashlock_and_bip388.rs` (the file that already owns the GAP-5b
verify-bundle/BIP-388 cells), updating its module docstring (`:7-13`) from "pinned refusal" to
"now-expanded intake".

1. **INVERT the pinned-refusal cell → real round-trip (red→green).**
   `verify_bundle_refuses_bip388_policy_json` → `verify_bundle_accepts_bip388_policy_json`.
   The current cell fed dummy `mk1qqq`/`md1qqq` cards because the refusal fired BEFORE card
   decode. Post-feature the refusal is gone, so dummy cards would now fail at decode/mismatch —
   the cell MUST become a real **bundle → verify-bundle** round-trip:
   - `bundle --descriptor <policy-JSON> --network mainnet --account 0 --slot @0.xpub=… --slot @1.xpub=… --json` (watch-only; bundle already accepts the policy — v0.49.0) → capture the bundle JSON.
   - `verify-bundle --descriptor <SAME policy-JSON> --network mainnet --account 0 --slot … --bundle-json <file>` → `.success()`.
   - Use the same 2-of-2 `wsh(sortedmulti(2,@0/**,@1/**))` policy + the two mainnet xpubs `A`/`B` already in the file (`:24-25`), so the diff from the old cell is minimal and auditable.
   - **Structural template = `cli_verify_bundle_multi_cosigner_mk1.rs:94` `audit_i10_same_xpub_two_paths_2of2_round_trips`** (R0-r1 M2): the closest watch-only-multisig-via-`--bundle-json` analogue. KEY: with `--bundle-json` the cosigner cards come from the ENVELOPE — NO `--mk1`/`--md1` flags are passed (`--bundle-json` `conflicts_with` ms1/mk1/md1 at verify_bundle.rs:76,82,89,96). So the inverted cell drops the old dummy `--mk1 mk1qqq --md1 md1qqq` entirely and uses `--bundle-json <file from the bundle step>`. (Cell (1) `hashlock_wsh_and_v_sha256_round_trips_via_bundle_json` `:31` is the single-card structural reference.)

2. **Positive single-sig cell** (optional but recommended — minimal, no cosigner cards):
   a `wpkh(@0/**)` singlesig policy → bundle → verify-bundle → success. Mirrors
   `cli_bip388_policy_intake.rs::bundle_descriptor_bip388_singlesig_policy_watch_only` (`:269`).
   This isolates the expand→concrete→verify path from multisig cosigner-card machinery.

3. **Negative cell — malformed policy still refused LOUDLY (non-vacuity guard).** A policy whose
   `description_template` references `@2/**` with only 2 `keys_info` entries (`@N` beyond
   `keys_info`) → verify-bundle exits non-zero with the `expand_bip388_policy` error. Mirrors
   `cli_bip388_policy_intake.rs::export_wallet_bip388_policy_at_n_beyond_keys_info_refused`
   (`:230`). Proves the probe routes through the real expander, not a bypass.
   - **R0-r1 M1 — the assertion MUST key on the SPECIFIC expander message, NOT the exit code.**
     This malformed policy exits **2 in BOTH worlds**: pre-feature via `classify_descriptor_form`'s
     `(true,true)` → `"mixes @N placeholders with inline keys"` (`pipeline.rs:136-138`);
     post-feature via the expander's residual-`@N` → `DescriptorParse("…@N beyond keys_info")`
     (`pipeline.rs:201-205`). The ONLY thing that flips on revert is the message. So the cell
     asserts `.stderr(predicate::str::contains("@N beyond keys_info"))` and must NOT use a bare
     `.code(2)` as its non-vacuity hook (exit-2 coincides). Optionally also assert the message is
     NOT `"mixes @N placeholders"` to make the discriminator explicit.

**Non-vacuity:** revert the 6-line insertion → cell 1 + cell 2 flip exit 0→2 (the "mixes @N…"
mixed-form refusal returns); cell 3 stays exit 2 but its `"@N beyond keys_info"` stderr assertion
fails (the message reverts to "mixes @N…"). So all three cells fail if the feature is removed or
mis-wired — cells 1/2 on exit code, cell 3 on the message (per M1 above).

## Lockstep / SemVer

- **MINOR → toolkit `v0.56.0` → `v0.57.0`.** New accepted input FORMAT = new capability
  (v0.49.0 `bip388-wallet-policy-to-descriptor-expansion` set this precedent: input-format
  additions are `feat:` = MINOR, distinct from schema_mirror flag-NAME parity).
- **schema_mirror: NO surface.** No new flag/option/subcommand/dropdown-value — `--descriptor`
  already exists; only the set of strings it accepts widens. `gui-schema` output is unchanged →
  no `mnemonic-gui/src/schema/mnemonic.rs` update, no paired GUI PR. (Confirm by running the
  toolkit suite incl. `schema_mirror` against the built binary post-change — expect PASS with no
  diff.)
- **Manual: one-line note (mandatory mirror).** `docs/manual/src/40-cli-reference/41-mnemonic.md`
  `--descriptor` row (`:517`) currently says "accepts either a BIP-388 `@N` template … or a bare
  concrete descriptor". Add that it ALSO accepts a BIP-388 **wallet-policy JSON** (leading-`{`,
  auto-expanded), matching the `bundle`/`export-wallet` rows. This is a doc addition, not a
  flag-coverage-lint change (the lint gates flag NAMES; `--descriptor` already listed) — but the
  manual is the constellation's single source of truth, so the prose must mirror. Run
  `make -C docs/manual lint` + the verify-examples/build to confirm no regression.
- **Version-marker lockstep (release ritual) — exact sites (CORRECTED by impl-review C1; R0-r1 M7
  was WRONG to say "exactly ONE README marker"):** there are **TWO** guard-enforced
  `<!-- toolkit-version: -->` markers — `README.md:13` AND `crates/mnemonic-toolkit/README.md:9`
  (both asserted by `tests/readme_version_current.rs::both_readmes_carry_current_version_marker`,
  which iterates `["README.md", "../../README.md"]`). PLUS `fuzz/Cargo.lock` carries the
  `mnemonic-toolkit` package version (separate cargo workspace, NOT gated by any Rust test → silent
  drift if missed; bump via `cargo update -p mnemonic-toolkit --precise <ver>` in `fuzz/`). PLUS ONE
  `scripts/install.sh` self-pin (`:32` `mnemonic-toolkit-v0.56.0`). ALL bumped to `v0.57.0` in
  the release commit. No toolkit version-bump self-reference in `pinned-upstream.toml` (the only
  such file, `docs/manual-gui/pinned-upstream.toml`, is a GUI-manual pin bumped by a GUI-manual
  cycle, NOT by a toolkit release). NO sibling pin changes → `manual.yml`/`quickstart.yml`/cross-tool
  stay FROZEN. CHANGELOG
  entry for v0.57.0. (Impl-review C1 CORRECTED the earlier R0-r1 M7 claim of "only one README
  marker": there ARE two, both guard-enforced — see the TWO-markers text above. The "×2"
  recollection from prior cycles was RIGHT.)
- **fmt gate:** `cargo +1.95.0 fmt --all` then REVERT `mlock.rs` (g6 exemption — NEVER fmt mlock.rs).
- **FOLLOWUP flip:** `verify-bundle-bip388-policy-intake` (FOLLOWUPS.md:4174) `open → resolved`
  in the shipping commit, citing the inverted cell.

## Execution sequence

1. R0 architect review of THIS plan-doc → loop to GREEN (0C/0I), persist verbatim to
   `design/agent-reports/c2-verify-bundle-bip388-plan-r0-round1-review.md` (+ subsequent rounds).
2. TDD: write the 3 cells (cell 1 = invert the refusal), confirm they RED against current code.
3. Implement the 6-line insertion. Confirm the 3 cells GREEN + full suite GREEN.
4. Per-phase impl review → 0C/0I, persist to `design/agent-reports/`.
5. Version bump v0.57.0 + manual note + CHANGELOG + version-marker/install.sh lockstep +
   FOLLOWUP flip. Full suite + manual lint + schema_mirror (expect no diff) + fmt gate.
6. Commit, tag `mnemonic-toolkit-v0.57.0`, push (per standing release authorization).
