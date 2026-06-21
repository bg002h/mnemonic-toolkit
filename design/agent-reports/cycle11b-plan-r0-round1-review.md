# R0 REVIEW — cycle-11b toolkit-hygiene plan-doc (L21 · L24 · L25) — Round 1

**Plan:** `design/IMPLEMENTATION_PLAN_cycle11b_toolkit_hygiene.md`
**Toolkit:** `origin/master = bea7a607` (v0.65.0) — one design-only commit *past* the plan's cited `9b3a6a3a`; the six source files remain byte-identical, so all line citations hold.
**Verdict: NOT GREEN — 0 Critical / 2 Important**

The plan is overwhelmingly accurate: every code citation across `convert.rs`, `verify_bundle.rs`, `bundle.rs`, `pipeline.rs`, `error.rs`, `slot_input.rs` verified exactly; the manual path correction, the CHANGELOG gate, the version-site table, the FOLLOWUP-slug analysis, and the version-coordination note are all correct. Two implementation-blocking defects remain — both in the load-bearing funds-safety / panic-fix axes the author flagged for scrutiny. Both would surface at TDD (won't compile / won't reproduce RED), but the plan must specify the correct binding/fixture so the implementer doesn't churn or, worse, "fix" the test into a vacuous pass.

## IMPORTANT

### I1 — L21 refusal predicate cites an OUT-OF-SCOPE binding at the chosen `:1350` enforcement site (won't compile as written)

The plan (P2, snippet at plan:229) and spec (D2/D3) place the POSITION-BASED refusal at the head of the composite `Bip38 =>` sub-arm (`convert.rs:1350`), with predicate `effective_bip38_passphrase.is_none()`. **But `:1350` is inside `fn compute_outputs` (`convert.rs:1217`), whose only passphrase parameter is `bip38_passphrase: Option<&str>` (`:1223`). `effective_bip38_passphrase` (the `Option<String>` from `run`, `:850`) is NOT in scope inside `compute_outputs`** — verified: zero occurrences of `effective_bip38_passphrase` in lines 1217-1600.

The `:932` guard the round-1 review cross-referenced lives in `run()` (`:765`), a *different* function where `effective_bip38_passphrase` IS live — which is exactly the conflation that produced the bug.

- **Required:** the arm-head predicate MUST be **`bip38_passphrase.is_none()`** (the in-scope `Option<&str>` parameter at `:1223`, set to `effective_bip38_passphrase.as_deref()` at `:980`). This is semantically identical — `as_deref()` maps `None→None`, `Some("")→Some("")` — so it preserves the `is_none()`-not-`is_empty()` invariant and the `--bip38-passphrase ""` (`Some("")`) GREEN path. Fix the snippet at plan:229 (and the spec's D2/§2.1 reference variable) to `bip38_passphrase`. The position-based framing and the §3 RED/GREEN test matrix are otherwise correct and unaffected.

(If the implementer instead chose the alternative `:932`-guard site — which the spec blesses — `effective_bip38_passphrase.is_none()` *would* compile, but the plan's chosen/preferred site is `:1350`, and at `:1350` the predicate as written is a compile error. The plan must name the right binding for its own chosen site.)

### I2 — L24 M2 RED fixture as literally written cannot reach `:1435`; it is rejected earlier by `validate_slot_set` (vacuous pass, panic never exercised)

The plan's L24 RED fixture (plan:157, mirrored from spec:272) is:
`--slot @0.phrase=<seed> --slot @1.phrase=<seed> --slot @2.path=m/84'/0'/0'`

Each `SlotInput` carries exactly **one** `subkey` (`slot_input.rs:99`). So `--slot @2.path=...` alone yields `@2`'s subkey-set = `{Path}`. Two independent failures result:

1. **`validate_slot_set` rejects `{Path}` first (`:1351`, before the inserted gate and before `:1435`).** `is_legal_set` (`slot_input.rs`) has no bare-`[Path]` arm — legal sets are `[Phrase,Path]`, `[Seedqr,Path]`, `[Ms1,Path]`, the `xpub` families, etc. So `@2={Path}` returns `SlotInputViolation{kind:"invalid-set"}` (exit 2) at `:1351`. The test asserts exit 2 and **passes for the wrong reason** — it never reaches the inserted gate, never reproduces the OOB panic, and would certify L24 "fixed" against a no-op.
2. **Even if it passed validation, the override loop's `:1427-1429` filter (`subkeys.contains(Phrase|Seedqr|Ms1)`) would `continue` at `:1431`** because `{Path}` contains no phrase-bearing subkey — never reaching the `:1435` write. This is precisely the M2 precondition (2) the round-1 review demanded, but the literal fixture string doesn't satisfy it.

The plan's prose hint ("@2.path=… **riding a phrase-bearing slot**", plan:164) gestures at the fix, but the explicit fixture string omits the co-located phrase subkey.

- **Required:** the fixture MUST give `@2` the set `[Phrase, Path]` (the legal + `exempted_v0_19_0` + phrase-bearing set), i.e. **`--slot @2.phrase=<seed> --slot @2.path=m/84'/0'/0'`** (two `--slot @2.*` flags for the same index — the established multi-subkey-per-slot pattern, cf. `cli_export_wallet_coldcard.rs:494-496`). With that, `@2` passes `validate_slot_set`, clears the `:1427-1429` filter, and reaches the unguarded `new_paths[2]` write at `:1435` → reproduces the panic pre-fix and the `DescriptorParse` reject post-fix. Update plan:157 and spec:272 to the corrected fixture, and keep the M2 fixture-comment assertions (`canonical_origin(...).is_none()` + `@2` phrase-bearing).

## MINOR (non-blocking)

- **M1 — stale "current origin/master" SHA.** Plan §0 states current `origin/master = 9b3a6a3a`; the live HEAD is `bea7a607` (one further design-only commit, "folded specs + plan-doc + R0 review trail"). The six source files are byte-identical at both, so every citation holds and the byte-identical claim is *still true* against the real HEAD — but the implementer branches off `origin/master`, which is now `bea7a607`. No action needed beyond awareness; the `git worktree add ../wt-cycle11b origin/master` instruction already resolves to live HEAD.
- **M2 — L24 gate position within the window.** The plan says "insert after `:1351`, before `:1371`." For exact parity with `bundle.rs` (whose gate precedes its canonicity probe), prefer inserting *before* the `canonicity_probe` parse at `:1361` rather than after it — so an over-`n` slot set is rejected before the parse, identical to bundle.rs ordering. Functionally equivalent either way (the parse doesn't consult slot indices); a tidiness note only.
- **M3 — L25 message line citations off by ±2.** Plan cites the "must carry a key origin" message at `:189-192` and "keyless script" at `:197-204`; live exact spans are `:187-191` and `:196-203`. The cited ranges still land inside the correct messages, and the tests assert on `.contains(...)` substrings, so this does not affect correctness — standardize at impl time.

## CORRECT and verified (no action)

- **L21 structure:** `Bip38 =>` sub-arm (`:1350`) is inside the `Seedqr | Phrase | Entropy =>` outer arm (`:1231`) — position genuinely proves all three sources incl. Seedqr; the refusal fires before `:1376`'s `unwrap_or("")`. `:932` guard requires BOTH passphrases unset, so `--passphrase X` alone sails past it → footgun is genuinely live. Direct `(wif↔bip38)` arms (`:1518`/`:1537`, `:1523`/`:1543`) are separate outer arms — unaffected. RED-1/2/3 (phrase/entropy/seedqr) + GREEN-1/1b (`--bip38-passphrase ""` still encrypts) + direct-edge regression all sound. `ConvertRefusal` (`:89`, exit 2 `:562`) reused — no new variant/flag.
- **L24 gate transcription:** `bundle.rs:1373-1388` (`slots.iter().map(|s| s.index as usize + 1).max().unwrap_or(0) != n` → `DescriptorParse`) verified byte-exact; the `slots`→`args.slot` rebinding is correct (same `&[SlotInput]`, field `.index`). Insertion window `:1351`→`:1371` is clean (only `canonicity_probe` parse + `is_non_canonical` between, neither depends on the override loop). All subkey-gate refined line numbers (`:1417-1420` build / `:1427-1429` filter / `:1431` continue / `:1435` write) verified exact. `validate_slot_set` checks contiguity + subkey-set only, NOT range-vs-`n` — the gap is real.
- **L25:** additive design sound — existing regex (`:56`) retains the `\b0[23]…{64}\b` 66-hex alternation, so the `:557` compressed-key-is-keyed and `sha256`/`ripemd160`-keyless assertions stay GREEN; `:529` keyless routing intact; both arms `Err` (display-only). New `tr(<64hex>`/`pk(<64hex>` anchors are purely additive.
- **Manual path:** `docs/manual/src/50-comparing/56-bip39-vs-bip38-pass.md` confirmed real (NOT `40-cli-reference/56-...`); edge table at `:49-54`, `(phrase,bip38)` `:53` / `(entropy,bip38)` `:54`, no `(seedqr,bip38)` row — exactly as stated. `41-mnemonic.md` `--bip38-passphrase` row at `:802` confirmed. The flag-coverage lint (`docs/manual/tests/lint.sh` step 4) checks only `grep -qF -- "$flag" "$chapter"` (flag-NAME presence) → prose-only edits stay GREEN.
- **FOLLOWUP slugs:** all three named slugs absent from `FOLLOWUPS.md`; the four "nearest existing" slugs verified `resolved` (`bip38-distinct-passphrase-flag` `:1968` resolved v0.8; `synthesize-…-with-unified` `:2577` resolved v0.47.1; `restore-emit-dispatch-3way-dedup` `:419` resolved v0.46.1; `bundle-keyless-descriptor-honest-refusal` `:81` ✓RESOLVED). FILE-3-NEW (2 closed in shipping commit, S-VERIFY-dedup left OPEN carrying the L24 gate note) is correct — no double-file, no mis-close. **Leaving the dedup slug OPEN is the right call:** the L24 fix lands a standalone gate (a deliberate duplicate of bundle.rs's), and the genuine dedup-into-shared-fn work is future; an OPEN slug correctly tracks it and the gate comment cites it.
- **Version sites + gates:** all five sites + CHANGELOG top entry verified at 0.65.0 (`Cargo.toml:3`, `README.md:13`, crate `README.md:9`, `install.sh:32`, `fuzz/Cargo.lock:574-575`, `CHANGELOG.md:9`). CHANGELOG IS gate-enforced — `changelog-check.yml` fires on `mnemonic-toolkit-v*` tags, greps `^## mnemonic-toolkit \[$VERSION\]`. No new `ToolkitError` variant (both exit 2, already present) → no schema_mirror/secret_drift. Bug-hunt ticks verified at `:823`/`:939`/`:952` with the re-grep-at-ship hedge present. Version coordination (11b→0.65.1 first; cycle-10 pin-bump→0.65.2) confirmed consistent with cycle-10's own plan §6.

## Disposition

**NOT GREEN — 2 Important.** Both are surgical text fixes to the plan, no design change:
- **I1:** change the L21 arm-head predicate from `effective_bip38_passphrase.is_none()` to `bip38_passphrase.is_none()` (the in-scope `compute_outputs` parameter at `:1223`).
- **I2:** change the L24 RED fixture from `--slot @2.path=...` to `--slot @2.phrase=<seed> --slot @2.path=...` so `@2={Phrase,Path}` reaches `:1435`.

Fold M1–M3 opportunistically. Per CLAUDE.md, persist this review verbatim to `design/agent-reports/cycle11b-plan-r0-round1-review.md`, fold I1+I2, then **re-dispatch R0** (the loop continues until 0C/0I — folds can introduce drift). Do not start TDD until the plan is R0-GREEN.
