# IMPLEMENTATION PLAN — mk1-repair-set-level-reverify (F4 Cycle 1)

**SPEC:** `design/SPEC_mk1_repair_set_level_reverify.md` (✅ R0-GREEN @ round 3). **This plan** is subject to its
own opus plan-R0 loop to 0C/0I BEFORE any implementation (CLAUDE.md). Per-phase: TDD (tests before src) +
per-phase opus R0 running the FULL `cargo test -p` suite + fold-reenter-loop until 0C/0I. Post-impl: mandatory
whole-diff review over the whole cross-repo diff.

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

---

## Phase P0 (toolkit) — Mk1 tri-state re-verify + auto-repair wiring + funds/harness tests

**Files:** `crates/mnemonic-toolkit/src/repair.rs` (the Mk1 classifier + `repair_card` Mk1 arm + the
`mnemonic repair` batch aggregation); `src/cmd/repair.rs` (exit aggregation if needed); NEW test file(s) under
`crates/mnemonic-toolkit/tests/`.

**TDD — tests first (SPEC §4):**
1. `prop_repair_never_wrong.rs` (or `cli_mk1_repair_reverify.rs`):
   - **§4.1 FUNDS ANCHOR (pinned seed):** a helper `find_mk1_miscorrection_seed()` (a bounded search: real
     ≥2-chunk mk1 card + a 5-substitution corruption in the regular-code trailing chunk that `bch_correct`
     aliases to a valid-but-≠-original codeword AND fails `mk_codec::decode`). Run it ONCE during impl; PIN the
     `(payload_seed, chunk_index, positions, from→to)` as a test constant. The test asserts: toolkit Mk1
     re-verify REJECTS the full corrected set (auto-repair does NOT short-circuit; `mnemonic repair` surfaces
     not-recovered). On a future BCH change that invalidates the seed → fail with an explicit "re-pin the F4
     miscorrection seed via `find_mk1_miscorrection_seed`" message.
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
  over the supplied group(s); on **Reject** → return the repair-not-applied/error outcome (auto-repair does NOT
  short-circuit); on **Candidate** → the VERIFY-ME (exit-4) candidate outcome + advisory; on **Bless** →
  proceed as today. Do NOT change the ≤4 happy path's blessed behavior.
- `mnemonic repair` (`src/cmd/repair.rs:143-144` loop): fold per-group verdicts to the dominant invocation exit
  (reject > candidate > bless > clean).
- Confirm all 4 auto-repair callers (convert/inspect/verify-bundle/xpub via `try_repair_and_short_circuit`)
  inherit the Reject-no-short-circuit behavior (no bypass).

**Per-phase R0 (opus, FULL `cargo test -p mnemonic-toolkit`)** → fold-reenter-loop → 0C/0I. Persist to
`design/agent-reports/cycleE-phase-P0-r0-round-N.md`.

## Phase P1 (mk-cli) — `mk repair` tri-state re-verify

**Files:** `mnemonic-key/crates/mk-cli/src/cmd/repair.rs`; NEW test(s) under `mk-cli/tests/`.

**TDD — tests first:**
- **manual example STILL works:** `mk repair <single chunk of a 2-chunk card>` (the `44-mk-cli.md:247` string)
  → exit 5 + the "unverified — reassemble to confirm" advisory (the documented per-plate workflow preserved —
  the C1 regression guard).
- **full-set miscorrection REJECTED:** the pinned §4.1-class seed as a full set → exit 2 (`CliError::Codec`).
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
  command still exits 5).
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
   (version pin) + re-vendor N/A (mk_codec unchanged — but the git-dep pin may move; **re-vendor IF the
   mk-codec/mk-cli git pin in the toolkit Cargo.lock changed** — verify vendor-freshness). Build 0.80.0; full
   suite green; FF master → toolkit release commit; tag `mnemonic-toolkit-v0.80.0`; push (admin-bypass
   `examples`); verify CI (incl. `sibling-pin-check` + `install-pin-check` GREEN).

**GOTCHAS (memory):** sibling-pin advance touches 5 refs — miss one → `sibling-pin-check` RED; `.examples-build`
corpus re-runs gen.sh on crates/Cargo/install.sh changes (version pin FATALs on mismatch); NEVER `cargo fmt
--all` (mlock.rs g6-exempt) — `cargo fmt -p` only.

## Guard-rails
- **G1** — no false-reject of a genuine full-set ≤4 correction (§4.3 pins Bless-on-decode-Ok).
- **G2** — partial-plate per-plate repair preserved at BOTH `mk repair` (exit 5) and `mnemonic repair` (exit 4)
  (§4.2 + P1 manual-example test).
- **G3** — funds proof is the PINNED seed (non-vacuous), not the Monte-Carlo (which may observe 0 at small N).
- **G4** — the classifier discriminates completeness from parsed indices vs total_chunks, NOT the overloaded
  error string.
- **G5** — cross-repo release order (mk-cli tag → toolkit 5-ref pin advance); both pin-checks green.
- **G6** — codecs NO-BUMP (verify the toolkit git-dep pin to mk-codec is unchanged, or re-vendor if it moved).
