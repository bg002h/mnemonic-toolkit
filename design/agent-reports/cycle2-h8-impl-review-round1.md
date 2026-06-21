# cycle-2 WS-A / H8 — per-phase implementation review (round 1)

**Reviewer:** opus adversarial execution review (post-implementation, single-phase).
**Worktree:** `/scratch/code/shibboleth/mnemonic-toolkit/.claude/worktrees/cycle2-h8`
**Branch / commit under review:** `fix/cycle2-h8` @ `53787cbb`
**Baseline:** toolkit `origin/master` = `f9467cc5` (release 0.61.0).
**Diff:** `git diff origin/master...HEAD` — `crates/mnemonic-toolkit/src/synthesize.rs` ONLY (+222 / −6).
**Design:** `design/IMPLEMENTATION_PLAN_cycle2_funds_loss_fixes.md` §1 (WS-A/H8) +
`design/BRAINSTORM_cycle2_funds_loss_fixes.md` (H8).

---

## VERDICT: GREEN — 0 Critical / 0 Important

The H8 fix is correct, complete, and the 5 new tests are real RED→GREEN discriminators
(behaviorally verified by reverting the fix in-place — 4 of 5 RED, the English regression
guard correctly stays GREEN). Scope is `synthesize.rs`-only; no version/error-variant/fmt
churn; full suite green (3265 passed / 0 failed across 183 binaries). No other
language-dropping template emit path exists. Ship.

---

## Critical

NONE.

## Important

NONE.

## Minor

- **m-1 (cosmetic, doc-line drift).** The new template-site comment (`synthesize.rs:1270`)
  says "byte-identical to the keyed path `:547`"; the keyed `emit_lang = c.language.unwrap_or(run_language)`
  is at LIVE `:552` (the `:547` figure is the plan-doc's pre-fold citation; on HEAD the keyed
  assignment line is `:552`, with `:547` being the doc-comment three lines above). Harmless —
  off-by-a-few-lines on a comment, not load-bearing; the byte-identity claim itself is TRUE
  (both sites are now character-for-character identical, which is itself evidence of the
  keyed↔template parity the §1.2 test 4 pins). No action required this cycle.

- **m-2 (no-op observation, not a defect).** The `synthesize_full` helper (`:373`,
  `#[allow(dead_code)]`) still emits a hardcoded-English `Payload::Entr` (`:383`). This is NOT
  a template path and NOT reachable from `--md1-form=template`: its only callers are
  `#[cfg(test)]` modules (`bundle.rs:2810`, `parse_descriptor.rs:2209/2240/2271`,
  `verify_bundle.rs`, and in-module tests). The runtime template form routes
  `synthesize_descriptor → synthesize_template_descriptor` exclusively. Out of H8 scope;
  noted only to document that the completeness sweep saw and dismissed it.

---

## Hardcoded-English completeness confirmation

Grepped `git show origin/master:…/synthesize.rs` for every `Language::English`,
`Payload::Entr`, `Payload::Mnem`, `emit_lang`, `unwrap_or(...)`, and `.language()` site.
The PRODUCTION ms1 emit sites are exactly three:

| site | fn | language source | status |
|---|---|---|---|
| `:547`→`:552` | `synthesize_descriptor` (keyed) | `c.language.unwrap_or(run_language)` | already correct (the twin) |
| `:709-719` | `synthesize_unified` | `seed_mnemonic.language()` (the seed's real wordlist) | already correct |
| `:1265`→`:1275` | `synthesize_template_descriptor` (template) | **WAS** `unwrap_or(English)` → **NOW** `unwrap_or(run_language)` | **THE FIX** |

`:383` (`synthesize_full`) is a dead-code/test-only helper, NOT a template path (m-2).

**Conclusion: `:1275` was the SOLE hardcoded-English template-emit site, and it is now fixed.
No sibling template-emit path still drops the wordlist language.** The template ms1 loop
(`:1271-1287`) is a SINGLE loop serving BOTH single-sig and multisig template forms (iterates
`for c in cosigners`, one push per slot), so the one-line change covers both — and the two new
`template_{singlesig,multisig}_…` tests pin both.

---

## Per-criterion findings

### 1. Correctness — PASS
- The new param `run_language: bip39::Language` is threaded into
  `synthesize_template_descriptor` (`:1167`), forwarded from the SOLE non-test caller `:487-492`
  (`synthesize_descriptor`'s `md1_form.is_template()` dispatch), and reaches the ms1 emit at
  `:1275` (`c.language.unwrap_or(run_language)`) — byte-identical to the keyed twin `:552`.
- All THREE runtime `synthesize_descriptor` call sites in `cmd/bundle.rs` supply a real
  `run_language`: `:1793` phrase/entropy mode (`args.language.unwrap_or_default()` — the
  bug-bite path), `:1910` concrete-descriptor mode (watch-only → English default, no emit, so
  moot), `:2154` import-json (`run_language_import`; slot `Some(wire_lang)` wins via
  `unwrap_or`). The non-English `--md1-form=template` phrase path now emits `Payload::Mnem`
  with the correct wire language and round-trips (verified in-test via `ms_codec::decode` +
  `wire_code_to_bip39` + master-fp reconstruction).

### 2. Completeness — PASS
- Sole-site confirmed (table above). Both single-sig and multisig covered by the shared loop.
- Precedence correct: `c.language: Option<bip39::Language>` (`:929`); per-slot `Some(x)` wins
  over `run_language` via `unwrap_or` — matches the keyed-path semantics
  (import-json mnem source overrides; `None` descriptor-@N phrase/entropy falls to run-level).
- Mixed-language multisig handled per-slot: the loop reads each `c.language` independently and
  pushes a per-slot payload, so slot-0-Spanish / slot-1-French would each emit their own wire
  code — structurally identical to the keyed path.

### 3. Test quality — PASS (RED verified by behavioral revert)
Reverting `:1275` back to `unwrap_or(bip39::Language::English)` (keeping the new signature) and
re-running: `template_singlesig_non_english_…`, `template_multisig_non_english_…`,
`template_non_english_master_fp_diverges_from_english`, and
`template_keyed_ms1_parity_across_languages` all FAIL; `template_english_run_language_still_emits_entr`
stays GREEN. This proves all four non-English/parity tests are genuine discriminators and the
English guard is non-vacuous (correctly insensitive to the fix).
- `template_non_english_master_fp_diverges_from_english` COMPUTES both fingerprints in-test
  (`bip39::Mnemonic::from_entropy_in(Spanish|English) → to_seed → Xpriv::new_master →
  fingerprint`), asserts `spanish_master_fp != english_master_fp`, and asserts the template
  card reconstructs the Spanish fp — the `1b6aef92`/`73c5da0a` oracle is a doc comment ONLY,
  never the assertion RHS (R0-M3 honored; a transcription typo cannot vacuously pass it). The
  RED revert panics precisely at the reconstructed-fp-equals-Spanish assertion.
- Two-stage RED is sound: the added param forces a compile-fail on the old call shape
  (signature stage); the `:1275` value drives the behavioral stage (verified above).

### 4. Scope — PASS
- `git diff --name-only origin/master...HEAD` = `synthesize.rs` only.
- `Cargo.toml` untouched (version churn deferred to release per plan §8 — correct).
- No new `ToolkitError` variant (H8 needs none).
- No `cargo fmt` / mlock churn (targeted edits only, per the WS-A note).
- The 3 updated in-module test call sites (`:2417`, `:2459`, `:2612`) pass
  `bip39::Language::English`, preserving prior behavior.
- Full suite: **3265 passed, 0 failed across 183 test binaries**; background run exit 0.

---

_Review only — no source edited (the in-place revert was reverted; `git diff --stat` clean,
both emit sites read `unwrap_or(run_language)`). Persisted before any fold/commit step._
