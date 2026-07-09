# IMPLEMENTATION PLAN — ms1-repair-demote-to-candidate (F4 Cycle 2 / Cycle F)

**SPEC:** `design/SPEC_ms1_repair_demote_to_candidate.md` (✅ R0-GREEN @ round 3, Fable). **This plan** is subject
to its own **Fable plan-R0** loop to 0C/0I BEFORE any implementation (user directive "fable for review, opus for
fold" + CLAUDE.md). Per-phase: TDD (tests before src) + per-phase Fable R0 running the FULL `cargo test -p` suite
+ fold-on-Opus-reenter-loop until 0C/0I. Post-impl: mandatory Fable whole-diff review over the whole cross-repo
diff.

**Status:** rev-2 — folded plan-R0-round-1 (0C/2I/4M): **I1 → MERGED the two toolkit phases into one P0** (so the
verify-bundle ms1 cells flip ONCE to final semantics + the advisory is standalone-only from the start — no P0→P1
transient) + added the **test-flip inventory**; I2 → homed §5.6/§5.7/§5.8; M1 (redaction precision + Zeroizing);
M2 (D20-short-circuit-envelope unreachable wire note + prose sweep); M3 (CHANGELOG head is `[0.80.0]`); M4 (ms-cli
`RepairJson.verdict` REQUIRED at identical field position + stale doc-comments). Review `cycleF-plan-r0-round-1.md`.

**Source SHAs:** toolkit `b20e3ce7`; ms-codec/ms-cli `mnemonic-secret master@c2fd4eb`.
**Target:** toolkit MINOR (`v0.81.0`) + ms-cli MINOR (`0.13.2`→`0.14.0`); ms-codec/mk-codec/md* NO-BUMP.
**Worktrees:** toolkit (P0/P2) in a `mnemonic-toolkit` worktree (branch `feature/ms1-repair-demote`); ms-cli (P1)
in a `mnemonic-secret` worktree (branch `feature/ms1-repair-demote`). Single implementer, sequential.

## Phase P0 (toolkit) — demotion + fall-through advisory + verify-bundle ground-truth compare + json verdict (MERGED)

One phase so the verify-bundle ms1 cells flip ONCE (no transient double-flip). **Files:** `src/repair.rs` (Ms1 arm
`:1148` + fall-through advisory + doc-comments `:443-444`/`:1145-1147`); `src/cmd/repair.rs` (`RepairJson.verdict`);
`src/cmd/verify_bundle.rs` (2 ms1 sites `:2079`, `:2503`); tests.

**TDD — tests first (SPEC §5.1,5.3-5.9,§8.6):**
- **§5.1 (funds anchor)** `mnemonic repair --ms1 <subst-corrupted>` → exit 4 + advisory (NOT 5); clean → exit 0.
- **§5.3** auto-repair on a corrected ms1 at `convert`/`inspect`/`xpub-search` (default-TTY via `MNEMONIC_FORCE_TTY`)
  → NO short-circuit + one-line stderr advisory; the caller's ORIGINAL decode error surfaces (exit 2), NOT exit 4
  (per the flip inventory — cell_19 class).
- **§5.4 (verify-bundle MATCH)** subst-corrupted ms1 whose correction == `expected.ms1[i]` → `ms1_decode` +
  `ms1_entropy_match` PASS (verify proceeds), "recovered via auto-repair, confirmed against expected seed"; NO
  advisory on this path; overall exit 0 if everything else passes.
- **§5.5 (verify-bundle MISMATCH — FUNDS ANCHOR, wrong-bundle)** `--slot @0.phrase="<seed E>" --ms1
  <corroded→corrects to wallet A's ms1> --mk1 <clean mk1 A> --md1 <md1 A>` → corrected(A) ≠ expected(E) →
  `ms1_entropy_match` FAILS → FULL table + `result: mismatch` → **exit 4**; NOT "recovered", NO 5, NO 2-abort, NO
  short-circuit. Multisig analogue.
- **§5.6 (indel keep-5)** `mnemonic repair --ms1 --max-indel 1 <single-indel, unique checksum-valid>` → **exit 5**;
  a multi-hit indel → `Ambiguous` → exit 4. **CLI-level** (not the unit `indel_exit_code_precedence` @:2689).
- **§5.7 (mixed-kind)** `mnemonic repair --ms1 <corrupted> --mk1 <clean>` → **exit 4 dominates** (candidate OR-fold).
- **§5.8 (`--no-auto-repair`)** suppresses BOTH the advisory (standalone-inline) AND the verify-bundle compare
  (extend cell_28 `cli_auto_repair.rs:500`: the direct `repair_card` call stays inside the `if !no_auto_repair`
  guard).
- **§5.9 / M1 / M4** `mnemonic repair --ms1 --json` → `RepairJson.verdict == "candidate"`; clean → `"blessed"`.
- **§8.6 / M1 (secret-hygiene guard)** the mismatch check-row detail / stderr / `--json` contains NO ms1 seed
  substring — scan for BOTH the corrected AND the expected ms1 strings.

**TEST-FLIP INVENTORY (I1 — pre-existing cells pinning OLD ms1 semantics; update as intended-behavior changes):**
| Test | Old | New (P0-final) |
|---|---|---|
| `cli_repair.rs:47` cell_9 (ms1 happy-path) | exit 5 + report | **exit 4** + advisory (Candidate) |
| `cli_auto_repair.rs:52` cell_19 (convert ms1 auto-fire) | exit 5 | **exit 2** (original decode error surfaces) + advisory — NOT 4 |
| `cli_auto_repair.rs:143` cell_18b (inspect) | exit 5 | **exit 2** + advisory |
| `cli_auto_repair.rs:228` cell_24 (convert `--json`, D20 `kind=ms1`) | short-circuit envelope | **no short-circuit** (original error) + advisory |
| `cli_auto_repair.rs:472` cell_27 (verify-bundle TTY) | `code(5)` + "# Repair report" | **exit 0** MATCH (`synth_corrupted_bundle_json` corrupts the same seed's own card → corrected==expected), "recovered" note |
| `cli_auto_repair.rs:557` cell_30 (verify-bundle D20 envelope) | `auto_repair_short_circuit:true, exit_code:5` | **VerifyBundleJson**, ms1 checks pass, exit 0 (D20 short-circuit envelope now UNREACHABLE for ms1 — M2) |

**Src:**
- Ms1 arm (`:1148`): `Unverified{reason}` iff `!repairs.is_empty()` else `Blessed`; ms1-specific `reason` (§2). No
  new type. Update the false doc-comments `repair.rs:443-444` ("Ms1/Md1 always Blessed") + `:1145-1147` (M4).
- Advisory (I2/M-R2-1): at the `Unverified` fall-through in `try_repair_and_short_circuit` (`:1690-1703`), emit the
  one-line stderr advisory, **kind-gated to Ms1**. Because P0 ALSO rewires the 2 verify-bundle sites to a direct
  `repair_card` call (below), the only Ms1 `try_repair_and_short_circuit` callers left are convert/inspect/xpub →
  the advisory is standalone-inline-only from the start (no transient). Do NOT change shipped mk1 behavior.
- verify-bundle (C1, both sites): obtain the corrected string via a DIRECT `repair_card(CardKind::Ms1, &[supplied])`
  call (pure — no advisory), inside the existing `if !no_auto_repair` guard; feed the corrected string into the
  existing `ms1_decode`/`ms1_entropy_match` compare vs `expected.ms1[i]`. Match → checks PASS. Mismatch → push the
  failed `ms1_entropy_match` row → run finishes → exit 4 (`Ok(if any_fail {4} else {0})`; NO `?`-abort, NO new
  `ToolkitError`). **REDACT** the mismatch row: generic detail ("auto-repair candidate did not match the expected
  seed — this card is not a card for this seed"), NO `expected`/`actual` seed bytes, pin `diff_byte_offset: None`
  (M1). Hold the corrected string in `Zeroizing` at the call-site (M1, proactive). `expected.ms1[i]` non-empty by
  construction (watch-only skips) — assert, no spurious branch.
- `RepairJson.verdict` (`cmd/repair.rs`, M4): `"blessed"|"candidate"` from `set_verify`, at a FIXED field position
  (mirror what ms-cli P1 must match — D27 byte-match). Do NOT touch `IndelJson` (M-R2-2).

**Per-phase Fable R0** (FULL `cargo test -p mnemonic-toolkit`) → fold-on-Opus → 0C/0I. Persist
`design/agent-reports/cycleF-phase-P0-r0-round-N.md`.

## Phase P1 (ms-cli) — `ms repair` exit-4 demotion + advisory + json verdict

**Files:** `mnemonic-secret/crates/ms-cli/src/cmd/repair.rs` (`:123-124`, `RepairJson` @:204-210, doc-comment
`:16-22`); tests.

**TDD — tests first (SPEC §5.2):** `ms repair <subst-corrupted>` → **exit 4** + stderr advisory; clean → 0;
uncorrectable → 2. `ms repair --json <corrupted>` → `RepairJson.verdict == "candidate"` (M4).

**Src:** demote `Ok(if any_correction {5} else {0})` → exit 4 on any correction + advisory; keep 0/2. **Add
`verdict` to ms-cli `RepairJson` at the IDENTICAL field position as toolkit's** (D27 byte-match invariant,
`ms-cli/src/cmd/repair.rs:200-203` "Field order is part of the schema"). Update the exit-code doc-comment
(:16-22). NO codec change. Do NOT bump version. `cargo fmt -p ms-cli` only.

**Per-phase Fable R0** (FULL `cargo test -p ms-cli`) → 0C/0I. Persist toolkit `cycleF-phase-P1-r0-round-N.md`.

## Phase P2 (docs) — manual lockstep (4 chapters) + wire notes + transcript regen

**Files:** `docs/manual/src/40-cli-reference/{41-mnemonic.md, 42-md.md, 43-ms.md, 44-mk-cli.md}`; transcripts.
- Rewrite the blanket "exit-5 `REPAIR_APPLIED` consistent across all four CLIs" (`41:750/:818-820`, `42:334`,
  `43:360`, `44:239`) → SPEC §4 model, phrased **"verified now, or verifiable-by-reassembly later"** (M2/M-R2 —
  NOT "oracle-verified"). Correct the per-kind auto-fire tables (`41:818-820`).
- Rewrite `41-mnemonic.md:3056-3059` ("ms1 no analogous risk") → the demoted-standalone / confirmed-in-verify-bundle
  / indel-stays-5 model. **Enumerate + fix** (M2): `41:756/:777` (D20-for-ms1 prose — the short-circuit envelope is
  now ms1-unreachable), `41:3042` (exit-code table "5…incl ms1" → 4), `41:3105` worked example (same corrupted ms1
  as `43-ms.md`; transcript + "Exit code: 5" → 4).
- `43-ms.md` `ms repair` chapter: exit 0/4/2 + advisory + no-self-verification caveat. verify-bundle chapter: the
  corrected-vs-expected compare (pass on match, `ms1_entropy_match` fail → exit 4).
- Regen every ms1 `repair`/auto-fire transcript whose stderr/exit/envelope changed, using the P0 toolkit + P1
  ms-cli binaries; `make -C docs/manual verify-examples` + `lint` green. No new flag → no schema_mirror/GUI.

**Per-phase Fable R0** (docs correctness + lockstep) → 0C/0I.

## Post-implementation (mandatory) — Fable whole-diff review
Fresh Fable over the WHOLE cross-repo diff: funds property (no false-Bless; corrected-vs-expected; §5.5 exit-4;
indel unique-only-5), no regression, secret-hygiene (no seed leak, `diff_byte_offset:None`, Zeroizing),
exit-code+manual accuracy, wire-shape notes complete, NO-BUMP. Persist `cycleF-postimpl-whole-diff-review.md`.

## Release ritual (only after post-impl GREEN)
1. **ms-cli FIRST:** bump `ms-cli/Cargo.toml` 0.13.2→0.14.0 (+ workspace Cargo.lock); FF `mnemonic-secret master`
   → P1 commit; tag `ms-cli-v0.14.0`; **user-gated MANUAL `cargo publish -p ms-cli`** post-tag (NO ms-codec
   publish — 0.7.0 unchanged); verify mnemonic-secret CI green. NO re-vendor.
2. **toolkit v0.81.0:** version sites (Cargo.toml + workspace/fuzz Cargo.lock + both READMEs + install.sh:32
   self-pin) + **ADVANCE the 4 ms-cli pin refs** `v0.13.2`→`v0.14.0` (`install.sh:38`, `.github/workflows/
   {quickstart.yml:87, technical-manual.yml:117, manual.yml:90}` — SAME commit; `manual-gui.yml:165` OUT) +
   CHANGELOG new `[0.81.0]` (head is `[0.80.0]`; leave prior intact — M3) + flip FOLLOWUPS ms1-leg → RESOLVED +
   regen `.examples-build/Examples.md`. Optional: `rust.yml:45` stale comment touch-up (M3). **NO re-vendor**
   (ms-codec pin `"0.7"` unchanged — verify no Cargo.lock dep delta). Build; full suite green; FF master → tag
   `mnemonic-toolkit-v0.81.0`; push (admin-bypass `examples`); verify CI (sibling-pin-check + install-pin-check +
   examples + the 3 manual workflows installing `ms-cli-v0.14.0`).

## Guard-rails
- **G1** — demotion never false-Blesses a touched correction (§5.1) nor false-Candidates a clean 0-corr decode.
- **G2 (funds)** — verify-bundle blesses a corrected ms1 ONLY when byte-equal to `expected.ms1[i]`; §5.5 exits 4.
- **G3** — mismatch = failed check row → exit 4 (full table), NEVER `?`-abort/typed-error/short-circuit.
- **G4** — the I2 advisory fires ONLY at convert/inspect/xpub (never verify-bundle — the direct `repair_card`
  call bypasses the helper); shipped mk1 partial-set behavior unchanged.
- **G5 (secret-hygiene)** — no ms1 seed bytes in the mismatch detail/stderr/`--json`; `diff_byte_offset:None`;
  Zeroizing corrected-string holder; §8.6 test scans for corrected AND expected substrings.
- **G6** — indel keep-5 scoped to `mnemonic repair --max-indel` UNIQUE recoveries only (multi-hit→Ambiguous→4);
  §5.6 CLI-level pin; the other 3 surfaces have no indel path.
- **G7** — codecs NO-BUMP; no clap surface → no GUI/schema_mirror; 4-site ms-cli pin advance lockstep; ms-cli
  manual `cargo publish`.
- **G8** — manual "principled distinction" = "verified now, or verifiable-by-reassembly later" (not
  "oracle-verified"); correct for all four CLIs incl. the shipped mk1 asymmetry.
- **G9 (D27)** — ms-cli `RepairJson.verdict` byte-matches toolkit's field position.
