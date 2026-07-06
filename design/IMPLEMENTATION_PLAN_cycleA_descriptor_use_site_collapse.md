# IMPLEMENTATION PLAN — Cycle A: descriptor use-site collapse fix (rev-2, post plan-R0 round 1)

**Status:** DRAFT rev-2 — pre-R0-round-2. NO code until this passes opus R0 to 0C/0I (CLAUDE.md hard gate).
**SPEC:** `design/SPEC_cycleA_descriptor_use_site_collapse.md` (GREEN). **Sweep:** `design/agent-reports/cycleA-migration-sweep.md`.
**Plan R0 round 1:** `design/agent-reports/cycleA-plan-r0-round-1.md` (0C/4I — all folded here).
**Source SHA:** `origin/master @ 8c8b9183`. **Executed by:** ONE implementer subagent in a git worktree, TDD,
phase-by-phase; per-phase opus R0 + FULL suite (`cargo test -p mnemonic-toolkit` + `cargo test -p wc-codec`)
GREEN before advancing; post-impl whole-diff review before ship.

---

## Load-bearing facts (verified; do NOT re-derive wrongly)
- `lex_placeholders` (`parse_descriptor.rs:60`) is called ONLY from `parse_descriptor` (`:853`), via (a) direct
  `@N`-template input (`bundle --descriptor <TEMPLATE>`, `cmd/bundle.rs:1389`; the a4/a5 seed leg) or (b)
  `concrete_keys_to_placeholders` (`pipeline.rs:330`, MANDATORY `[fp/path]xpub` bracket) used by
  `bundle --descriptor <CONCRETE>`, `import-wallet` (all vendor parsers), `verify-bundle`.
- **`export-wallet --descriptor` and `compare-cost --descriptor` parse via `MsDescriptor::from_str` DIRECTLY —
  never `lex_placeholders`.** Their concrete `/0/*` tests STAY-PASSING (incl. `descriptor_to_bip388_non_multipath_refused`
  which STAYS exit 1 — SPEC §8's old "exit 2" line is superseded, plan-R0 M-b). If any starts emitting exit-2/the
  residue message, that is itself a regression (wrongly entered the `@N` lexer).
- **Verify-path error variant is PER-PATH (plan-R0 I-B):** concrete-descriptor verify (`verify_bundle.rs:1352-1357`
  → `descriptor_concrete_to_resolved_slots`, `pipeline.rs:417-418`) → `DescriptorParse` / **exit 2** (the false-pass
  site SPEC §1 names). `@N`-template verify (`verify_bundle.rs:1375`) → `DescriptorReparseFailed{detail}` / exit 4.
- `/**` reaches the lexer UN-expanded (`pipeline.rs:391` `push_str(&descriptor[last_end..])` copies `/**)` verbatim → `@0[fp…]/**` → wild eats `/*`,
  residue `*` → REJECT). No pre-lexer `/**`→`<0;1>` expansion exists. Sparrow is NOT affected — it self-expands
  `@i/**`→`[fp/path]xpub/<0;1>/*` at `sparrow.rs` Step 5 (`:353-387`) before lexing (plan-R0 M-c).
- Terminator set `) , }` + whitespace + EOS is complete (R0-verified). `#` never directly follows a placeholder.

## The change (core, in atomic Phase 1)
In `lex_placeholders`, AFTER the multipath-body validator (`:146-178`) and BEFORE `out.push(...)` (`:183`), per
occurrence:
```rust
let match_end = caps.get(0).map(|m| m.end()).unwrap_or(0);
if let Some(next) = descriptor[match_end..].chars().next() {
    if !matches!(next, ')' | ',' | '}') && !next.is_whitespace() {
        let residue: String = descriptor[match_end..].chars().take(24).collect();
        return Err(ToolkitError::DescriptorParse(format!(
            "@{i}: derivation steps after the placeholder are not representable in md1; the use-site \
             path must be a multipath `/<a;b>/*` (or bare `/*`) as the final step — a fixed single step \
             like `/0/*` (or the `/**` shorthand) is un-representable (found residue near `{residue}`)"
        )));
    }
}
```
Mirrors md-cli `template.rs:128-137`, ADAPTED (toolkit regex has no bare-origin group — do NOT add one).
Placement after the validator preserves the H13 byte-exact hardened-multipath error (validator `.transpose()?`
at :177 returns first). Panic-safe (regex match ends are char boundaries; `.chars()` is codepoint-wise).

---

## Phase 1 (ATOMIC) — residue-reject floor + reject-with-remediation + FULL test migration → GREEN
Plan-R0 I-A: the residue check and the 22 cells it flips are one indivisible unit — a split boundary leaves the
suite RED. TDD is preserved WITHIN this phase; the phase COMMITS only when the FULL suite is GREEN.

**1a. Write ALL new failing tests (red):**
- Unit lex REJECT tests (`parse_descriptor.rs`): `wpkh(@0/0/*)`, `wpkh(@0/0h/*)`, `wpkh(@0[deadbeef/84'/0'/0']/0/*)`,
  post-mp `wpkh(@0/<0;1>/0/*)`, pre-mp `wpkh(@0/0/<0;1>/*)`, bare-unbracketed-origin `wpkh(@0/48h/0h/0h/<0;1>/*)`,
  `/**` `wpkh(@0[deadbeef/84'/0'/0']/**)`, multisig non-first-slot `wsh(multi(2,@0/<0;1>/*,@1/<0;1>/0/*))`. Assert
  `DescriptorParse`.
- Unit POSITIVE controls (must lex-pass): `wpkh(@0/<0;1>/*)`, `wpkh(@0/*)`, `wpkh(@0/*h)`, `wpkh(@0/<0;1>)`,
  bare `wpkh(@0)` (D1 deferred — MUST still succeed; keep `lex_bare_at_zero` unchanged), keyless multisig
  `wsh(sortedmulti(2,@0,@1))`, `tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<2;3>/*))`, nested
  `sh(wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*)))`, `#`-guard (`wpkh(@0/<0;1>/*)#csum` must NOT residue-reject on `#`).
- Per-surface CLI REJECT tests (NEW, dedicated — these are the sole coverage of each reject shape, enabling the
  Group-B swaps below): bitcoin-core `/0/*` blob → exit 2 + workaround message; **`--format descriptor` `/**`**
  → exit 2 + `<0;1>/*` workaround (plan-R0 I-D); `--format descriptor` `/0/*`; specter receive-only `/0/*`; old
  `--json` replay `/0/*`; BSMS single-branch `/0/*`.
- Sparrow taproot-multisig-passthrough POSITIVE control (plan-R0 M-c): a Sparrow passthrough import STILL succeeds
  (its descriptor is `[fp/path]xpub/<0;1>/*`) — an over-rejection guard.

**1b. Implement** the residue check (above). New reject tests → green; the 22 incumbent cells flip red. (Optional
in-phase aid: run scoped tests first to see the fallout — NOT a committed boundary.)

**1c. Migrate all incumbent cells to GREEN** (sweep report is the checklist — NON-EXHAUSTIVE; re-run
`grep -rn '/0/\*\|/1/\*' crates/mnemonic-toolkit/{src,tests}` and classify any new hit):
- **Group A — assert the reject, NEVER swap to `<0;1>`** (they encode the collapse as "convergence"):
  `cli_cross_start_convergence.rs::a4_...`/`a5_...`. **BOTH legs assert the reject** (plan-R0 M-e): the direct
  `bundle --descriptor "wpkh(@0[{fp}/84'/0'/0']/0/*)"` leg AND the `walletfile_to_bundle("bitcoin-core", …)` leg
  (the latter via the bitcoin-core workaround message).
- **`:898` — assert reject, KEEP fixture** (plan-R0 I-C): `core_fixture_file_mainnet_receive_change_pair_parses`
  flips `bundles=2 .success()` → reject (exit≠0 + workaround message). Leave `core-mainnet-receive-change-pair.json`
  UNCHANGED — it is the canonical legacy same-key `/0/*`+`/1/*` split regression AND the pair-merge follow-up's
  input fixture. Do NOT swap it.
- **Group B — swap the incidental `/0/*` to `<0;1>/*` to PRESERVE feature coverage** (the reject shape is covered
  by 1a's dedicated tests; entry counts survive — no card-dedup): the bitcoin-core multi/select/network/advisory
  cells (`core_multi_descriptor_emit_all`, `core_select_descriptor_by_index`/`_active_receive`/`_active_change`,
  `core_multisig_wsh_sortedmulti_2_of_3`, `core_testnet_tpub_network_detected`, `core_fixture_file_multi_bip84_all`,
  `core_masked_older_emits_advisory`/`core_clean_older_emits_no_advisory`), `cli_older_advisory.rs::fires.../clean...`,
  `cli_output_class.rs::bundle_descriptor_emits_watch_only`,
  `cli_import_wallet_network_override.rs::homogeneous_two_mainnet_blob_override_mainnet_ok` (HIGHEST-SIGNAL
  over-rejection control), `cli_import_wallet_envelope_v0_27_0.rs::bitcoin_core_multi_descriptor_yields_one_envelope_per_entry`,
  `cli_import_wallet_roundtrip.rs::fixture_core_multi_bip84_emit_four_bundles`/`fixture_core_bip49_mainnet_...`.
  **plan-R0 M-a: swap in-body LITERALS for inline-blob cells** (`build_core_multi`/`build_core_single` `d0=…/0/*`;
  the two `older_advisory` inline cells), and swap the `.json` bodies for fixture-file cells (`core-multi-bip84.json`,
  `core-bip49-mainnet.json`, `core-two-mainnet.json`) to per-key-identical `<0;1>/*` entries preserving each
  test's assertion COUNT. (`active_receive`/`active_change` fixtures keep separate internal=false/true entries —
  the `active && !internal`/`active && internal` selectors survive.)
- **⚠ Special (fixture-swap for a different reason):** `core_select_index_out_of_range_errors` (:713) — entry 0
  now rejects before the `--select 99` OOB check; swap its fixture to a `<0;1>` ≥2-entry blob so it still tests
  OOB selection (else its `"99"`/`"range"` assertion is vacuous).
- **Do NOT touch the 19 STAYS-PASSING controls** — re-run to prove NO over-rejection; especially
  `core_fixture_file_multipath_receive_change_pair_parses` (:915, different keys) and the export-wallet /
  compare-cost bypass controls.

**1d. Gate:** FULL `cargo test -p mnemonic-toolkit` + `cargo test -p wc-codec` **GREEN**; opus per-phase R0
(persist `cycleA-phase-1-r0-round-N.md`); folds re-enter the review loop; commit only on GREEN.

## Phase 2 — funds-proof regressions (TDD; all GREEN at phase end)
**2a. verify-bundle false-pass closure (plan-R0 I-B / M-7):** PRIMARY = the **concrete** form matching SPEC §1's
cited site — `verify-bundle` a concrete `wpkh([fp/84'/0'/0']xpub…/0/*)` descriptor against any card → assert
**exit 2 + `DescriptorParse` + the multipath-remedy message** (the reparse rejects BEFORE card comparison, closing
the false-pass). OPTIONAL secondary = the `@N`-template verify path → exit 4 + `DescriptorReparseFailed`. (Encode
can no longer BUILD a `/0/*` bundle, so use a pre-generated wrong-card fixture if a card is needed.)
**2b. BIP-84 oracle:** (i) POSITIVE — a correctly-encoded `<0;1>/*` single-sig card for `abandon×11 about`
restores/derives first receive `bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu`; (ii) NEGATIVE — `/0/*` to
`bundle --descriptor` REJECTS at encode (exit 2), never producing the collapsed
`bc1q8vph849lf3e9rrj85hsxrzlv949rtahe794k6p` card.
**2c. Gate:** full suite GREEN; opus per-phase R0.

## Phase 3 — lockstep ripples
**3a. Manual** (`docs/manual/src/40-cli-reference/`): document the `import-wallet`/`bundle --descriptor` fixed-step
reject + error text; the **interim bitcoin-core limitation** note (Core receive+change hard-fail until the
pair-merge follow-up; workaround `<0;1>/*` + `--format descriptor`); AND the **`/**` shorthand hard-fail** note
(plan-R0 I-D — same workaround). Grep the CLI-reference for any `@N`-placeholder example with a fixed use-site
step (none expected). `make -C docs/manual lint` — no flag change.
**3b. CHANGELOG** (tag-gated): (a) the funds fix (C1); (b) interim Core-import limitation + workaround; (c) **`/**`
shorthand hard-fail + workaround** (plan-R0 I-D); (d) deferred concrete-nonranged-xpub residual (M-1).
**3c. Examples gate** (`examples.yml`/`docs/Examples.pdf`): sweep for `@N` fixed-step examples; regenerate if
touched (none expected). `examples` CI guard green.
**3d. GUI:** NONE — no flag/enum (no `schema_mirror` ripple), no `--json` wire-shape change (pair-merge split out).
Confirm `mnemonic gui-schema` output byte-unchanged.
**3e. Gate:** manual lint + examples guard green; opus per-phase R0 (docs).

## Phase 4 — post-impl whole-diff review + release + FOLLOWUPS + ship
**4a. PRECONDITION:** post-impl mandatory independent adversarial opus **whole-diff review** GREEN (persist
`cycleA-postimpl-whole-diff.md`; folds RE-ENTER the loop).
**4b. Release ritual:** toolkit **MINOR** bump (plan-R0 M-d — breaking behavior change; matches prior funds-cycle
precedent; e.g. current v0.75.x → v0.76.0, CONFIRM the live version at release); BOTH READMEs; `fuzz/Cargo.lock`;
`install.sh` SELF-pin (NOT the frozen md-cli sibling pin); re-vendor iff a dep bumps (none expected). md/mk/ms
NO-BUMP; do NOT touch md-codec.
**4c. FOLLOWUPS** (`design/FOLLOWUPS.md`): mark this cycle / C1 **RESOLVED** in the shipping commit; FILE
`bitcoin-core-receive-change-pair-merge` (former Part 2 — full I-2 scope incl. `:915` merge-negative-control +
the KEPT `core-mainnet-receive-change-pair.json` as merge input); FILE `concrete-nonranged-xpub-implied-wildcard`
(D1, funds framing); FILE `bip389-double-star-shorthand-support` (`/**`→`<0;1>` expansion; plan-R0 I-D notes this
may be higher user-impact than the pair-merge — flag for prioritization). Sibling companions: none.
**4d. Ship:** direct-FF to master + tag. Verify published repro binary + `sibling-pin-check` + `changelog-check`
green post-tag.

---

## Per-phase discipline (every phase)
1. Tests FIRST (red) → implement (green). 2. FULL `cargo test -p mnemonic-toolkit` + `cargo test -p wc-codec`.
3. Opus per-phase R0; persist verbatim to `design/agent-reports/cycleA-phase-N-r0-round-M.md` BEFORE fold-and-commit;
folds re-enter the review loop. 4. Stage paths explicitly (no `git add -A`). 5. Update `CONTINUITY_cycleA_LIVE.md`
at each gate. Commit a phase ONLY when its full suite is GREEN.

## Plan-R0 round-1 dispositions (all folded above)
I-A merged Phase 1+2 (atomic) ✓ · I-B per-path verify variant (concrete→DescriptorParse/exit2) ✓ · I-C `:898`
assert-reject keep-fixture ✓ · I-D `/**` disclosure + CLI test + follow-up ✓ · M-a inline-literal swaps ✓ ·
M-b SPEC §8 export line superseded ✓ · M-c sparrow discharge + positive control ✓ · M-d MINOR bump ✓ · M-e a4/a5
both legs ✓.
