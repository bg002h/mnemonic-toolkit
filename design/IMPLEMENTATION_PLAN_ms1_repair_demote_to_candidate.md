# IMPLEMENTATION PLAN — ms1-repair-demote-to-candidate (F4 Cycle 2 / Cycle F)

**SPEC:** `design/SPEC_ms1_repair_demote_to_candidate.md` (✅ R0-GREEN @ round 3, Fable). **This plan** is subject
to its own **Fable plan-R0** loop to 0C/0I BEFORE any implementation (per user directive "fable for review, opus
for fold" + CLAUDE.md). Per-phase: TDD (tests before src) + per-phase Fable R0 running the FULL `cargo test -p`
suite + fold-on-Opus-reenter-loop until 0C/0I. Post-impl: mandatory Fable whole-diff review over the whole
cross-repo diff.

**Source SHAs:** toolkit `bc023827` (v0.80.0 line + Cycle F design); ms-codec/ms-cli `mnemonic-secret master@c2fd4eb`.
**Target:** toolkit MINOR (`v0.81.0`) + ms-cli MINOR (`0.13.2`→`0.14.0`); ms-codec/mk-codec/md* NO-BUMP.
**Worktrees:** toolkit phases (P0/P1/P3) in a `mnemonic-toolkit` worktree (branch `feature/ms1-repair-demote`);
ms-cli phase (P2) in a `mnemonic-secret` worktree (branch `feature/ms1-repair-demote`). Single implementer,
sequential.

## Phase P0 (toolkit) — ms1 substitution-demotion + fall-through advisory + json verdict

**Files:** `crates/mnemonic-toolkit/src/repair.rs` (Ms1 arm `:1148`; the standalone-inline fall-through advisory);
`src/cmd/repair.rs` (`RepairJson.verdict`); NEW/extended tests under `crates/mnemonic-toolkit/tests/`.

**TDD — tests first (SPEC §5.1-5.3, §5.9):**
- **§5.1 (funds anchor)** `mnemonic repair --ms1 <subst-corrupted>` → exit 4 + advisory (NOT exit 5); the reason
  text present; clean ms1 → exit 0.
- **§5.3** auto-repair on a corrected ms1 at `convert`/`inspect`/`xpub-search` (drive default-TTY via
  `MNEMONIC_FORCE_TTY`) → does NOT short-circuit AND emits the one-line stderr advisory (I2). Assert NO silent
  apply, NO exit-5.
- **§5.9 / M1** `mnemonic repair --ms1 --json <corrupted>` → `RepairJson.verdict == "candidate"`; clean →
  `"blessed"`. (Indel path unaffected — separate `IndelJson`.)

**Src:**
- `repair_card` Ms1 arm (`:1148`): `SetVerify::Unverified{reason}` iff `!repairs.is_empty()`, else `Blessed`
  (ms1-specific `reason` per SPEC §2). No new type.
- Fall-through advisory (I2/M-R2-1): at the `Unverified` fall-through in `try_repair_and_short_circuit`
  (`:1690-1703`), emit the one-line stderr advisory **only** for the standalone-inline sites
  (convert/inspect/xpub-search) and **only** for ms1 (kind-gated). Do NOT emit at the 2 verify-bundle sites (P1
  gets the corrected string via a direct `repair_card` call). Do NOT change shipped mk1 partial-set behavior.
- `RepairJson.verdict` field (`cmd/repair.rs`): `"blessed"|"candidate"` from `outcome.set_verify`. Do NOT touch
  `IndelJson` (M-R2-2). Note the `--json` wire-shape addition (consumers self-update; not schema_mirror-gated).

**Per-phase Fable R0** (FULL `cargo test -p mnemonic-toolkit`) → fold-on-Opus → 0C/0I. Persist
`design/agent-reports/cycleF-phase-P0-r0-round-N.md`.

## Phase P1 (toolkit) — verify-bundle ground-truth comparison (the C1 mechanism)

**Files:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` (the 2 ms1 sites: single-sig `:2079`, multisig
`:2503`); tests.

**TDD — tests first (SPEC §5.4, §5.5, §8.6):**
- **§5.4 (MATCH)** verify-bundle with a subst-corrupted ms1 whose correction == `expected.ms1[i]` → `ms1_decode`
  + `ms1_entropy_match` PASS (verify proceeds), noted "recovered via auto-repair, confirmed against expected
  seed"; NO fall-through advisory on this path.
- **§5.5 (MISMATCH — FUNDS ANCHOR, the wrong-bundle attack)** `verify-bundle --template bip84
  --slot @0.phrase="<seed E>" --ms1 <corroded→corrects to wallet A's ms1> --mk1 <clean mk1 A> --md1 <md1 A>` →
  corrected(A) ≠ expected(E) → `ms1_entropy_match` FAILS → FULL check table + `result: mismatch` → **exit 4**;
  NOT "recovered", NO exit-5, NO exit-2 abort, NO short-circuit. Multisig analogue with a per-cosigner mismatch.
- **§8.6 (secret-hygiene guard)** assert the mismatch check-row detail / stderr / `--json` contains NO ms1 seed
  substring (see src decision below).

**Src:**
- At each ms1 site, obtain the corrected string via a DIRECT `repair_card` call (NOT `try_repair_and_short_circuit`
  — avoids the P0 advisory on this path, M-R2-1). Feed the corrected string into the existing `ms1_decode`/
  `ms1_entropy_match` comparison against `expected.ms1[i]`. Match → checks PASS. Mismatch → push the failed
  `ms1_entropy_match` row → the run finishes → exit 4 (I3; NO `?`-abort, NO new `ToolkitError` variant).
- **Secret-hygiene decision (SPEC §8.6 / R0 advisory-1):** the C1 mismatch row detail is **REDACTED** — do NOT
  echo the corrected-seed or expected-seed bytes into `VerifyCheck.expected`/`.actual`/detail (a corrected ms1 is
  a bearer secret; [[feedback_secret_hygiene_first_class_bar]]). Use a generic detail ("auto-repair candidate did
  not match the expected seed — this card is not a card for this seed"). (The pre-existing supplied-case
  `ms1_entropy_match` echo behavior is NOT widened here; if that itself is a concern it is a separate tracked
  item.) §5's §8.6 cell pins no-seed-bytes.
- `expected.ms1[i]` is non-empty by construction wherever the sites fire (watch-only slots skip) — assert it,
  don't add a spurious no-ground-truth branch.
- **Wire-value note:** `ms1_decode` can now be `pass` after auto-repair (was always `fail` in that arm) —
  document the `--json` value shift; confirm no verify-examples golden exercises a corrupted-ms1 bundle (likely
  none).

**Per-phase Fable R0** (FULL suite) → 0C/0I. Persist round(s).

## Phase P2 (ms-cli) — `ms repair` exit-4 demotion + advisory

**Files:** `mnemonic-secret/crates/ms-cli/src/cmd/repair.rs` (`:123-124`); tests.

**TDD — tests first (SPEC §5.2):** `ms repair <subst-corrupted>` → **exit 4** + stderr advisory; clean → exit
0; uncorrectable → exit 2 (already wired). `--json` verdict if ms-cli has an envelope (mirror P0's field shape if
present; else advisory-only).

**Src:** demote `Ok(if any_correction {5} else {0})` → exit 4 (Candidate) on any correction + emit the advisory;
keep 0 / 2. Update the `ms repair` exit-code doc-comment. NO `mk_codec`/`ms_codec` change. Do NOT bump version
(release phase). NO `cargo fmt --all` (per-package `cargo fmt -p ms-cli` only; respect any mlock exemption if the
repo shares it).

**Per-phase Fable R0** (FULL `cargo test -p ms-cli` in mnemonic-secret) → 0C/0I. Persist to the toolkit
`design/agent-reports/cycleF-phase-P2-r0-round-N.md` (cross-repo audit trail in toolkit).

## Phase P3 (docs) — manual lockstep (4 chapters) + json + transcript regen

**Files:** `docs/manual/src/40-cli-reference/{41-mnemonic.md, 42-md.md, 43-ms.md, 44-mk-cli.md}`; transcripts.
- Rewrite the blanket "exit-5 `REPAIR_APPLIED` consistent across all four CLIs" sentence (`41:750/:818-820`,
  `42:334`, `43:360`, `44:239`) → SPEC §4 principled model, phrased **"verified now, or verifiable-by-reassembly
  later"** (M2 — NOT "an oracle verified it", false for mk1 single-plate). Cover the shipped mk1-Candidate exit-4
  case AND ms1.
- Correct the per-kind auto-fire tables (`41:818-820`) — the "Auto-fire (exit 5 + repair report)" rows are false
  for ms1 (now: no short-circuit + advisory).
- Rewrite `41-mnemonic.md:3056-3059` ("ms1 … no analogous risk") → ms1 has a worse, undetectable
  substitution-miscorrection variant, demoted to exit-4 Candidate standalone, confirmed against the typed seed
  inside verify-bundle; indel stays exit-5 (full-checksum self-verify).
- `43-ms.md` `ms repair` chapter: exit 0/4/2 + advisory + no-self-verification caveat.
- verify-bundle chapter: the corrected-vs-expected comparison (pass on match, `ms1_entropy_match` fail → exit 4).
- Regen any ms1 `repair` transcript whose stderr/exit changed (5→4 + advisory) using the P0/P2 binaries; verify
  `make -C docs/manual verify-examples` + `lint` green (build P0 toolkit binary + P2 ms-cli binary; MK/MD/MS_BIN
  overrides). Confirm no NEW flag → no schema_mirror/GUI companion.

**Per-phase Fable R0** (docs correctness + lockstep) → 0C/0I.

## Post-implementation (mandatory) — Fable whole-diff review
Fresh Fable over the WHOLE cross-repo diff (toolkit branch + ms-cli branch): the funds property (no false-Bless;
corrected-vs-expected; §5.5 wrong-bundle exit-4; indel unique-only-exit-5), no regression, secret-hygiene (no
seed leak in the mismatch row), exit-code contract + manual accuracy, NO-BUMP holds. Persist
`cycleF-postimpl-whole-diff-review.md`. GREEN → release.

## Release ritual (only after post-impl GREEN)
**Order (SPEC §7):**
1. **ms-cli release FIRST:** bump `ms-cli/Cargo.toml` 0.13.2→0.14.0 (+ workspace Cargo.lock); FF `mnemonic-secret
   master` → the P2 commit; tag `ms-cli-v0.14.0`; **user-gated MANUAL `cargo publish -p ms-cli`** post-tag
   (ms-codec stays 0.7.0, already published — NO ms-codec publish); verify mnemonic-secret CI green (incl. its
   freebsd gate — watch for the same `@master` drift the mk-cli cycle hit; pin if it recurs). NO re-vendor
   (workspace-crate bump only).
2. **toolkit v0.81.0:** version sites (Cargo.toml + workspace Cargo.lock + fuzz/Cargo.lock + both READMEs +
   install.sh:32 self-pin) + **ADVANCE the 4 ms-cli sibling-pin refs** `ms-cli-v0.13.2`→`v0.14.0`
   (`install.sh:38`, `.github/workflows/{quickstart.yml:87, technical-manual.yml:117, manual.yml:90}`;
   `manual-gui.yml:165` OUT of scope — GUI cadence) + CHANGELOG `[0.81.0]` (leave `[0.76.0]` intact) + flip
   FOLLOWUPS `bch-repair-miscorrection-set-level-reverify` ms1-leg → RESOLVED in the shipping commit + regen
   `.examples-build/Examples.md` (version pin). **NO re-vendor** (ms-codec pin `"0.7"` unchanged — verify no
   Cargo.lock dep delta). Build 0.81.0; full suite green; FF master → release commit; tag
   `mnemonic-toolkit-v0.81.0`; push (admin-bypass `examples`); verify CI (sibling-pin-check + install-pin-check +
   examples + the 3 manual workflows that `cargo install --tag ms-cli-v0.14.0`).

## Guard-rails
- **G1** — the demotion never false-Blesses a touched correction (§5.1) nor false-Candidates a clean 0-correction
  decode (exit 0 preserved).
- **G2 (funds)** — verify-bundle blesses a corrected ms1 ONLY when it byte-equals `expected.ms1[i]`; the §5.5
  wrong-bundle attack exits 4, never "recovered".
- **G3** — mismatch = a failed check row → exit 4 (full table), NEVER a `?`-abort / typed-error / short-circuit
  (the `--json` envelope + remaining rows still emit).
- **G4** — the I2 advisory fires ONLY at convert/inspect/xpub (never verify-bundle); shipped mk1 partial-set
  behavior unchanged.
- **G5 (secret-hygiene)** — no ms1 seed bytes in the mismatch check-row detail / stderr / `--json`
  ([[feedback_secret_hygiene_first_class_bar]]); §8.6 test pins it.
- **G6** — indel keep-5 scoped to `mnemonic repair --max-indel` UNIQUE recoveries only (multi-hit → Ambiguous →
  4); the other 3 surfaces have no indel path.
- **G7** — codecs NO-BUMP (ms-codec no source change; mk-codec not read); no clap surface → no GUI/schema_mirror;
  4-site ms-cli pin advance in lockstep; ms-cli manual `cargo publish`.
- **G8** — the manual "principled distinction" phrased "verified now, or verifiable-by-reassembly later" (not
  "oracle-verified"); correct for all four CLIs incl. the shipped mk1 asymmetry.
