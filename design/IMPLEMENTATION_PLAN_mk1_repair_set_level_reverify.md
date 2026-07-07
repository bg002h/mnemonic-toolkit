# IMPLEMENTATION PLAN — mk1-repair-set-level-reverify (F4 Cycle 1)

**SPEC:** `design/SPEC_mk1_repair_set_level_reverify.md` (✅ R0-GREEN @ round 3). **This plan** is subject to its
own opus plan-R0 loop to 0C/0I BEFORE any implementation (CLAUDE.md). Per-phase: TDD (tests before src) +
per-phase opus R0 running the FULL `cargo test -p` suite + fold-reenter-loop until 0C/0I. Post-impl: mandatory
whole-diff review over the whole cross-repo diff.

**Status:** ✅ **plan-R0-GREEN (0C/0I) @ round 2** — rev-3 folded round-1 (PI-1 tri-state `RepairOutcome` verdict + 3-caller map + csid-fold; PI-2 pinned corrupted-STRING seed; PM-1/2/3) + round-2 Minors (PM-r2-1 Reject-not-indel-trigger; PM-r2-2 batch reject message; PM-r2-3 kind-agnostic verdict; PM-r2-4 optional json field). Reviews `cycleE-plan-r0-round-{1,2}.md`. **CLEARED for implementation** (CLAUDE.md phase-3: single implementer, TDD tests-before-src, worktree, per-phase R0 FULL suite, post-impl whole-diff).

**Source SHAs:** toolkit `998654f5`; mk-cli/mk-codec `mnemonic-key main@85bca69`.
**Target:** mk-cli MINOR + toolkit MINOR (`v0.80.0`); mk-codec/md-codec/ms-codec NO-BUMP.
**Worktrees:** toolkit phases in a `mnemonic-toolkit` worktree (branch `feature/mk1-repair-set-level-reverify`);
mk-cli phase in a `mnemonic-key` worktree (branch `feature/mk1-repair-set-level-reverify`). Single implementer,
sequential.

## The shared classifier (implemented independently in BOTH repos — no shared crate; NO-BUMP)

Both `mk repair` (mk-cli) and the toolkit Mk1 repair arm must classify a corrected chunk set per the SPEC §2
invariant. The classifier is small + pure; it is written TWICE (toolkit `src/repair.rs`; mk-cli
`crates/mk-cli/src/cmd/repair.rs`) because they are separate binaries and codecs get NO source change. Shape
(both repos):

```
// Given the corrected chunk strings for ONE chunk_set_id group:
//   parse total_chunks/chunk_index from each chunk header (public API:
//   DecodedString::data() -> StringLayerHeader::from_5bit_symbols -> Chunked{..});
//   complete_and_consistent = indices 0..total-1 each present exactly once & consistent total/id.
// Then:
//   if mk_codec::decode(&group_refs).is_ok()            => Bless      // exit 5 / short-circuit-apply
//   else if complete_and_consistent                     => Reject     // exit 2 / no short-circuit  (FUNDS FIX)
//   else /* incomplete */                               => Candidate  // exit 5 (mk repair) / exit 4 (mnemonic repair) + advisory
// Multi-group: fold groups; invocation outcome = max(reject > candidate > bless > clean); a Reject group's
//   chunks are NOT presented as recovered.
enum GroupVerdict { Bless, Reject, Candidate }
```
Single-group is the common case (one card = one chunk_set_id). Grouping handles the documented batch input.

### Return-shape + 3-caller map (plan-R0 PI-1 — REQUIRED before P0 src)

`repair_card` is `Result<RepairOutcome, RepairError>` (`repair.rs:760`) — BINARY, cannot carry three verdicts;
Bless(exit 5) and Candidate(exit 4 + advisory) would collide in the `Ok` arm, and `try_repair_and_short_circuit`
(`repair.rs:1340`) would wrongly short-circuit(5) a Candidate. Resolution:
- **Return shape:** add a verdict discriminant to `RepairOutcome` (`repair.rs:437-441`) — e.g. a field
  `set_verify: SetVerify { Blessed, Unverified }` (Unverified carries the advisory reason). The ms1/md1 arms
  default to `Blessed` (they already return only on decode success → no behavior change). A **Reject** is NOT an
  `Ok` — it maps to a `RepairError` (surfaces as the un-repaired decode error; auto-repair falls through).
- **`repair_card` owns the aggregation:** it does the csid-grouping + the per-group classify + the dominant fold
  (`reject > candidate > bless > clean`) and returns the aggregate verdict, keeping raw corrected chunks for the
  Candidate advisory. (Batch input reaches `repair_card(Mk1, [all strings])` as ONE call — `resolve_groups`
  groups by KIND, not csid — so the csid sub-grouping MUST live in `repair_card`, not the caller.)
- **3-consumer map:**
  - `try_repair_and_short_circuit` (auto-repair, `:1340`): **Bless → short-circuit(exit 5); Candidate → NO
    short-circuit; Reject(=Err) → NO short-circuit.** Auto-repair NEVER blesses an unverified/rejected set — the
    caller's original error surfaces (a partial/miscorrected card cannot convert anyway).
  - `mnemonic repair` (`cmd/repair.rs:143-144`): Bless → exit 5; **Candidate → exit 4 VERIFY-ME + advisory**
    (`indel_exit_code` convention); Reject → exit 2 (surfaced decode error).
  - (P1) `mk repair`: Bless → exit 5; Candidate → exit 5 + advisory; Reject → exit 2.
- **Ripple:** every existing `match` on `repair_card`'s return updates for the new discriminant (ms1/md1 =
  Blessed); the per-phase FULL `cargo test -p` catches any missed arm.

**plan-R0-round-2 Minors (fold in P0 — message precision, non-gating):**
- **PM-r2-1** — the Reject `RepairError` variant MUST NOT be in `is_indel_trigger` (`repair.rs:1105/1109`), OR
  short-circuit Reject BEFORE the `--max-indel>=1` indel check (`cmd/repair.rs:143-144`) — else a full-set
  miscorrection at `--max-indel>=1` routes through `recover_indel_card` and surfaces the generic "indel
  unrecoverable" message instead of the intended "corrected each chunk but the set does not reassemble" (still
  exit 2 / funds-safe, but wrong message). Prefer a dedicated Reject variant (e.g. `SetReassemblyMismatch`) not
  in the indel-trigger set.
- **PM-r2-2** — a batch that folds to a dominant Reject suppresses ALL output (fail-safe — a co-batched blessed
  group is NOT emitted as recovered); the Reject message MUST name WHICH `chunk_set_id` group failed so the
  user can re-run the good group alone.
- **PM-r2-3 (wording)** — `repair_card` returns the kind-agnostic `Unverified` verdict; the exit-4 mapping is a
  `mnemonic repair` CALLER concern — do NOT bake exit-4 into the shared engine.
- **PM-r2-4 (optional)** — a `--json` Candidate is signalled by exit code + stderr advisory only; adding an
  `unverified: true`/`set_verify` envelope field is a wire-shape change (consumers self-update, not gated) —
  defer unless trivially free.

---

## Phase P0 (toolkit) — Mk1 tri-state re-verify + auto-repair wiring + funds/harness tests

**Files:** `crates/mnemonic-toolkit/src/repair.rs` (the Mk1 classifier + `repair_card` Mk1 arm + the
`mnemonic repair` batch aggregation); `src/cmd/repair.rs` (exit aggregation if needed); NEW test file(s) under
`crates/mnemonic-toolkit/tests/`.

**TDD — tests first (SPEC §4):**
1. `prop_repair_never_wrong.rs` (or `cli_mk1_repair_reverify.rs`):
   - **§4.1 FUNDS ANCHOR (pinned seed — DETERMINISTIC, plan-R0 PI-2):** a helper `find_mk1_miscorrection_seed()`
     (bounded search: real ≥2-chunk mk1 card + a 5-substitution corruption in the regular-code trailing chunk
     that `bch_correct` aliases to a valid-but-≠-original codeword AND fails `mk_codec::decode`). Run it ONCE
     during impl. **PIN the fully-resolved corrupted chunk STRINGS directly** (`const CORRUPTED_SET: [&str; N]`)
     so the test needs NO re-encode — because `encode()` draws a RANDOM `chunk_set_id` (`pipeline.rs:45-47`) that
     sits in the BCH codeword, so pinning only `(payload, positions)` and re-encoding would NOT reproduce the
     miscorrection (flaky/vacuous). (Alternative if a card object is needed: pin `chunk_set_id` too + encode via
     `encode_with_chunk_set_id`, `pipeline.rs:67`.) The test asserts: toolkit Mk1 re-verify REJECTS the full
     corrected set (auto-repair does NOT short-circuit; `mnemonic repair` exit 2, not-recovered). On a future
     BCH change that invalidates the pinned set → fail with an explicit "re-pin the F4 miscorrection seed via
     `find_mk1_miscorrection_seed` (cap ~10⁷; if none found the rate is lower than assumed — escalate)" message.
     The search helper has an explicit iteration cap (~10⁷) that fails loudly rather than hanging (PM-2).
   - **§4.2 partial-set per-plate PRESERVED:** `mnemonic repair --mk1 <one plate of a 2-chunk card>` → exit-4
     VERIFY-ME candidate + the "unverified — reassemble to confirm" advisory (NOT a reject).
   - **§4.3 genuine ≤4 full-set correction still blesses** (toolkit applies; decode Ok).
   - **§4.4 clean card** — no correction / no auto-repair.
   - **§4.5 convert/inspect auto-repair** on the §4.1 full-set wrong-fit no longer silently emits the wrong
     card (drives the default-TTY path via `MNEMONIC_FORCE_TTY`).
   - **§4.5b BATCH reject-dominant:** `mnemonic repair --mk1` with {the §4.1 miscorrection group, one clean/
     partial group} → invocation reject; the miscorrected group's chunks NOT presented as recovered.
   - **§4.6 md1 regression-lock:** an md1 wrong-fit correction already rejected by the content-id check; AND
     assert md1 has no non-chunked decode path bypassing `reassemble`.
   - **§4.7 reachability-lock:** min-size real mk1 card produces ≥2 chunks; `SingleString` mk1 not
     encoder-emitted.
   - **§4.8 rate harness:** seeded `StdRng`, fixed N (size for E[hits]≫1 OR soft-warn observed-≥1; `--ignored`/
     env-gate if slow), Clopper-Pearson UPPER bound on the 5-substitution miscorrection rate; record the
     measured bound for the CHANGELOG/manual.
2. Run — all new cells RED (or compile-fail) before src.

**Src:**
- Add the `GroupVerdict` classifier (parse headers via public mk_codec API; group by chunk_set_id;
  complete_and_consistent; decode → verdict).
- `repair_card` `CardKind::Mk1` (`src/repair.rs:766-783`): after building `corrected_chunks`, run the classifier
  over the supplied group(s); on **Reject** → return `Err(RepairError)` (auto-repair does NOT short-circuit; the
  variant is NOT an indel-trigger, per PM-r2-1); on **Candidate** → return `Ok{set_verify: Unverified}` + the
  advisory reason (kind-agnostic — the exit-4 mapping is a `mnemonic repair` CALLER concern, PM-r2-3); on
  **Bless** → `Ok{Blessed}` as today. Do NOT change the ≤4 happy path's blessed behavior.
- `mnemonic repair` (`src/cmd/repair.rs:143-144` loop): fold per-group verdicts to the dominant invocation exit
  (reject > candidate > bless > clean).
- Confirm all 4 auto-repair callers (convert/inspect/verify-bundle/xpub via `try_repair_and_short_circuit`)
  inherit the Reject/Candidate-no-short-circuit behavior (no bypass).
- **Existing-test audit (plan-R0 PM-1):** the intended exit-code changes flip a subset of the 13+12 mk1 refs in
  `cli_repair.rs`/`cli_auto_repair.rs`: full-set miscorrection 5→2, and single-chunk `mnemonic repair --mk1`
  **5→4** (today exits 5 via `indel_exit_code` with no indel). Most cases (full-card ≤4 bless; >4-per-chunk
  `BchUncorrectable` before reassembly) are unaffected. Audit + update the flipped cases as intended-behavior
  changes (the per-phase FULL `cargo test -p` will surface every one — do not mis-triage a flip as a
  regression).

**Per-phase R0 (opus, FULL `cargo test -p mnemonic-toolkit`)** → fold-reenter-loop → 0C/0I. Persist to
`design/agent-reports/cycleE-phase-P0-r0-round-N.md`.

## Phase P1 (mk-cli) — `mk repair` tri-state re-verify

**Files:** `mnemonic-key/crates/mk-cli/src/cmd/repair.rs`; NEW test(s) under `mk-cli/tests/`.

**TDD — tests first:**
- **manual example STILL works:** `mk repair <single chunk of a 2-chunk card>` (the `44-mk-cli.md:247` string)
  → exit 5 + the "unverified — reassemble to confirm" advisory (the documented per-plate workflow preserved —
  the C1 regression guard).
- **full-set miscorrection REJECTED:** COPY the SAME pinned `CORRUPTED_SET` constant from P0 (no shared crate;
  same `mk_codec` → aliases identically) as a full set → exit 2 (`CliError::Codec`).
- **genuine full-set ≤4** → exit 5 (blessed, reassembles).
- **batch reject-dominant** — {miscorrection full-set group, clean/partial group} → exit 2; miscorrected chunks
  not presented as recovered.
- **clean** → exit 0.

**Src:** mirror the P0 classifier in `cmd/repair.rs` after the per-string loop (parse headers, group, decode,
verdict); map Bless→exit 5, Candidate(incomplete)→exit 5 + advisory, Reject(complete-and-consistent)→exit 2;
multi-group dominant exit. Update the mk-cli repair exit-code doc-comment.

**Per-phase R0 (opus, FULL `cargo test -p mk-cli` in mnemonic-key)** → loop → 0C/0I. Persist to the toolkit
`design/agent-reports/cycleE-phase-P1-r0-round-N.md` (keep the cross-repo audit trail in the toolkit).

## Phase P2 (docs lockstep) — manual caveat + golden verify + cite measured rate

**Files:** `docs/manual/src/40-cli-reference/{41-mnemonic.md (repair §2990-3037 + auto-fire §739-751),
44-mk-cli.md (mk repair chapter + the §247 example note)}`.
- Add the >4-error miscorrection caveat: a full-set repair now REJECTS an mk1 miscorrection (exit 2); a
  single-plate correction is UNVERIFIED until the full card is reassembled (advisory); BIP-93 recommends
  confirming a corrected codex32 string. Cite the §4.8 MEASURED rate bound (NOT `2⁻¹³·⁹`).
- **Verify the `44-mk-repair-text.out` verify-examples golden is UNAFFECTED** (partial-plate repair keeps exit
  5 + adds an advisory line — regenerate the golden if the advisory text lands in it; confirm the example
  command still exits 5). **(PM-1)** the manual's EXECUTED repair goldens are ms1-based (`41-repair-ms1.*`);
  there is NO `mnemonic repair --mk1` executed golden → the P2 golden-regen scope is `mk repair` only — state
  this explicitly so the exit 5→4 `mnemonic repair --mk1` change needs no golden regen.
- `make -C docs/manual lint` (bidirectional flag-coverage) — no CLI flag added, so no new flag rows; confirm.
- **GUI/schema_mirror:** confirm no flag added → no `mnemonic-gui` schema change (verify; §6/M4).

**Per-phase R0** (docs correctness + lockstep) → loop → 0C/0I. Persist round(s).

## Post-implementation (mandatory) — whole-diff review
Fresh opus over the WHOLE cross-repo diff (toolkit branch + mk-cli branch): funds property (no bless without
decode Ok; partial-plate preserved; batch reject-dominant), no regression, the pinned seed genuinely aliases +
is caught, the harness is non-vacuous, exit-code contract, manual lockstep, NO-BUMP holds. Persist
`cycleE-postimpl-whole-diff-review.md`. GREEN → release.

## Release ritual (only after post-impl GREEN)
**Order (SPEC §6):**
1. **mk-cli release FIRST:** bump mk-cli Cargo.toml MINOR (+ workspace lock), CHANGELOG, both READMEs if
   versioned; FF mnemonic-key main → the P1 commit; tag `mk-cli-vX.Y.0`; publish mk-codec-unchanged + mk-cli to
   crates.io per the mnemonic-key ritual; verify CI.
2. **toolkit v0.80.0:** version sites (Cargo.toml + workspace Cargo.lock + fuzz/Cargo.lock + both READMEs +
   install.sh:32 self-pin) + **ADVANCE all 5 mk-cli sibling-pin refs** to the new mk-cli tag (`install.sh:41`,
   `.github/workflows/{manual.yml:79, quickstart.yml:77, technical-manual.yml:109}`,
   `docs/manual/src/40-cli-reference/44-mk-cli.md:12`) + CHANGELOG new `[0.80.0]` (leave `[0.76.0]` intact) +
   flip FOLLOWUPS F4-Cycle-1 slug → RESOLVED in the shipping commit + regen `.examples-build/Examples.md`
   (version pin). **Re-vendor: NO (plan-R0 PM-3 default) — leave the toolkit's `mk-codec` git-dep pin PUT**
   (mk_codec has no source change this cycle); the 5 mk-cli pin refs are `cargo install` RECOMMENDATION/doc
   TEXT (not Cargo dependencies) → zero Cargo.lock/vendor impact. (Only re-vendor if some other change moved the
   toolkit Cargo.lock — verify `git diff` shows no unexpected lock delta.) Build 0.80.0; full suite green; FF
   master → toolkit release commit; tag `mnemonic-toolkit-v0.80.0`; push (admin-bypass `examples`); verify CI
   (incl. `sibling-pin-check` + `install-pin-check` GREEN).

**GOTCHAS (memory):** sibling-pin advance touches 5 refs — miss one → `sibling-pin-check` RED; `.examples-build`
corpus re-runs gen.sh on crates/Cargo/install.sh changes (version pin FATALs on mismatch); NEVER `cargo fmt
--all` (mlock.rs g6-exempt) — `cargo fmt -p` only.

## Guard-rails
- **G1** — no false-reject of a genuine full-set ≤4 correction (§4.3 pins Bless-on-decode-Ok).
- **G2** — partial-plate per-plate repair preserved at BOTH `mk repair` (exit 5) and `mnemonic repair` (exit 4)
  (§4.2 + P1 manual-example test).
- **G3** — funds proof is the PINNED seed (non-vacuous), not the Monte-Carlo (which may observe 0 at small N);
  the pinned seed reproduces DETERMINISTICALLY independent of the OS RNG (pin the corrupted STRINGS, or pin
  `chunk_set_id`) — plan-R0 PI-2.
- **G4** — the classifier discriminates completeness from parsed indices vs total_chunks, NOT the overloaded
  error string.
- **G5** — cross-repo release order (mk-cli tag → toolkit 5-ref pin advance); both pin-checks green.
- **G6** — codecs NO-BUMP; default = LEAVE the toolkit's mk-codec git-dep pin put (mk_codec unchanged); the 5
  mk-cli pin refs are install-recommendation text, not Cargo deps → no re-vendor (plan-R0 PM-3). Verify no
  unexpected Cargo.lock delta.
- **G7 (plan-R0 PI-1)** — the tri-state verdict propagates via a `RepairOutcome` discriminant, and
  `try_repair_and_short_circuit` treats **both Reject AND Candidate as NO-short-circuit** (auto-repair never
  blesses an unverified/rejected set); `repair_card` owns the csid-grouping + dominant fold. A test drives the
  auto-repair path on a Candidate (partial) set and asserts NO exit-5 short-circuit.
