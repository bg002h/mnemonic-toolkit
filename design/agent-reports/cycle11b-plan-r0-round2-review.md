# R0 REVIEW — cycle-11b toolkit-hygiene plan-doc (L21 · L24 · L25) — Round 2

**Plan:** `design/IMPLEMENTATION_PLAN_cycle11b_toolkit_hygiene.md`
**Toolkit:** `origin/master = bea7a6076c4709f6e09c7006aa11242636ee16ea` (v0.65.0) — **matches the plan's claimed live HEAD exactly.**
**Verdict: GREEN — 0 Critical / 0 Important**

Both round-1 Important findings (I1, I2) are folded correctly and verified against live source; all three Minors are folded; no new drift was introduced by the folds. Every code citation in the plan was re-grepped against `origin/master` and holds. Implementation may begin.

## Round-1 fold verification

### I1 (L21 predicate scope) — FOLDED CORRECTLY

The fold is structurally and semantically sound, verified against live `convert.rs`:

- **`compute_outputs` boundary:** `fn compute_outputs(...)` runs from `:1217` to just before the next top-level `fn map_bip38_error` at `:1734`. The refusal site `:1350` is inside it.
- **Out-of-scope binding confirmed:** every occurrence of `effective_bip38_passphrase` (`:850, :866, :872, :932, :963, :980`) is inside `run()` — the **last** is `:980`. **Zero occurrences in `:1217-1733`** (the `compute_outputs` body). So `effective_bip38_passphrase.is_none()` would not compile at `:1350` — the round-1 defect was real.
- **In-scope param confirmed:** `bip38_passphrase: Option<&str>` is the `compute_outputs` signature param at `:1223`; the doc at `:1201` states "`bip38_passphrase` is `Some` only when the user passed `--bip38-passphrase`." The composite arm consumes it at `:1376` (`bip38_passphrase.unwrap_or("")`).
- **Fold is correct:** §1 row `:1223` now reads "predicate tests THIS (`bip38_passphrase.is_none()`)"; the §1 SCOPE NOTE is accurate (cites `:765/:850/:980/:1217/:1223`, all verified exact); the P2 snippet reads `if bip38_passphrase.is_none() {` — compiles.
- **`as_deref()` equivalence holds:** `:980` is `let bip38_passphrase = effective_bip38_passphrase.as_deref();`. `as_deref()` maps `None→None` and `Some(String::new())→Some("")`, so `bip38_passphrase.is_none() ⟺ effective_bip38_passphrase.is_none()`, preserving the `is_none()`-not-`is_empty()` invariant and the `--bip38-passphrase ""` (`Some("")`) GREEN path. The arm-position membership proof (`:1350` ⊂ `Seedqr | Phrase | Entropy =>` at `:1231`) is intact — all three sources incl. Seedqr covered structurally.

### I2 (L24 RED fixture) — FOLDED CORRECTLY

Verified against live `verify_bundle.rs` + `slot_input.rs`:

- **`SlotInput` holds one subkey:** `pub subkey: SlotSubkey` (`slot_input.rs:99`) — a single field, confirmed. So `--slot @2.path=…` alone ⇒ `@2={Path}`.
- **`[Path]` is NOT a legal set:** `is_legal_set` (`slot_input.rs:347-372`) singleton arms are `[Phrase] | [Seedqr] | [Entropy] | [Ms1] | [Xpub] | [Wif] | [Xprv]` — **no bare `[Path]`**. A bare `@2={Path}` fails `is_legal_set` → `SlotInputViolation{kind:"invalid-set"}` (exit 2) at `validate_slot_set` (`:1351`) — the vacuous-pass path the round-1 review flagged.
- **`[Phrase, Path]` IS legal:** present at `slot_input.rs:365` (exactly as the plan cites), and also `exempted_v0_19_0` at `:299`.
- **Fixture now reaches `:1435`:** the folded fixture gives `@2` BOTH `phrase` and `path` → `[Phrase, Path]`. This (a) passes `validate_slot_set` (legal set), (b) lands in `by_index_subkeys` (`:1417-1420`), and (c) clears the `:1427-1429` phrase-bearing filter (contains `Phrase`), so the loop does NOT `continue` at `:1431` and reaches the unguarded `new_paths[*idx as usize]` write at `:1435` — reproducing the OOB panic pre-fix. The CRITICAL callout and the M2 fixture-precondition pins are accurate.

### M1 / M2 / M3 — FOLDED CORRECTLY

- **M1 (SHA):** §0 now states live HEAD = `bea7a607` — matches `git rev-parse origin/master` exactly. Byte-identity `4e8ad792 → bea7a607` re-confirmed for all six files (`git diff --quiet` returns 0 for `convert.rs`, `verify_bundle.rs`, `bundle.rs`, `pipeline.rs`, `error.rs`, `slot_input.rs`).
- **M2 (gate position):** now prefers inserting after `validate_slot_set` (`:1351`) and **before the `canonicity_probe` parse at `:1361`**, matching bundle.rs ordering. The `canonicity_probe` parse (`:1359-1364`) is `parse_descriptor(&descriptor_str, &[], &[])` — does NOT consult slot indices, so the gate is functionally correct anywhere in the window and the bundle.rs-parity placement is the right tidiness call.
- **M3 (message line numbers):** now cites the "must carry a key origin" message at `:187-191` and "keyless script" at `:196-203`. Verified: the `(false,false)` arm is `:185`; the key-origin `Err(...)` spans `:188-192` (substring at `:190`, inside `:187-191`); the keyless `Err(...)` spans `:197-203` (substring at `:199`, inside `:196-203`). Both `.contains(...)` substring assertions land correctly.

## Re-check of previously-CORRECT parts for fold-introduced drift — all clean

- **L24 gate transcription:** `bundle.rs:1373-1388` verified byte-exact. `slots`→`args.slot` rebind correct (both `&[SlotInput]`, field `.index`). All override-loop lines exact: build `:1417-1420`, filter `:1427-1429`, `continue` `:1431`, write `:1435`. `validate_slot_set` (`:249`) is contiguity + per-slot subkey-set only, NOT range-vs-`n`.
- **L25 additive-anchor design:** regex `:56` retains `\b0[23][0-9a-fA-F]{64}\b`; a bare 64-hex x-only token (no `02/03`, 64≠66) is genuinely unmatched today → RED route confirmed. Regression test `:557` asserts the 66-hex `02`-compressed key (`wpkh(0279be...)`) stays keyed AND `sha256`(64-hex)/`ripemd160`(40-hex) stay keyless — exactly the M3 additive guard. `:529` keyless routing + `:185` dual-`Err` arms intact.
- **Manual path:** `docs/manual/src/50-comparing/56-bip39-vs-bip38-pass.md` edge table at `:49-54` confirmed — `(phrase,bip38)` `:53`, `(entropy,bip38)` `:54` ("defaults to `''` if unset; BREAKING"), no `(seedqr,bip38)` row. `41-mnemonic.md:802` `--bip38-passphrase` row confirmed.
- **FOLLOWUP slugs:** all three named slugs return **0 occurrences** in `FOLLOWUPS.md` — FILE-3-NEW (2 closed in shipping commit, S-VERIFY-dedup left OPEN carrying the L24 gate note) is correct.
- **Version sites + CHANGELOG:** all five at `0.65.0` (`Cargo.toml:3`, `README.md:13`, crate `README.md:9`, `install.sh:32`, `fuzz/Cargo.lock:574-575`) + CHANGELOG top entry `## mnemonic-toolkit [0.65.0] — 2026-06-21` at `:9`. CHANGELOG gate-enforced.
- **Bug-hunt ticks:** `### - [ ] L21` `:823`, `L24` `:939`, `L25` `:952` — all confirmed exact; re-grep-at-ship hedge present.
- **No new ToolkitError variant:** `ConvertRefusal(String)` `:89` (exit 2 `:562`), `DescriptorParse(String)` `:123` (exit 2 `:569`) — both reused, no schema_mirror/secret_drift trigger.

## Disposition

**GREEN — 0 Critical / 0 Important.** I1 and I2 are folded correctly with no semantic regression; M1/M2/M3 are accurate; no fold introduced drift. Every citation re-grepped against `origin/master = bea7a607` holds. The plan is R0-GREEN — the implementer may branch off live `origin/master` and begin TDD (P1→P2→P3) with per-phase R0, then the mandatory whole-diff review. Standing reminders for impl already in-plan: branch off live `origin/master` (not the own-account worktree), renumber-on-collision if cycle-10 lands first, and re-grep the bug-hunt tick line numbers at ship.
