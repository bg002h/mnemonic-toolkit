# Phase 3 (Task 3) — code-quality / per-phase architect review (verbatim)

> Persisted verbatim per CLAUDE.md BEFORE the fold. Two-stage review: spec-compliance = ✅ compliant
> (refusal in None branch only, routing verified, all 5 test changes, 20 passed); code-quality (this doc)
> = **Approve with recommended fixes, 0C/2I/1m** — BOTH Importants are test-coverage strengthening, NOT
> behavioral bugs (the reviewer's matrix table confirms NO silent-wrong-emit path remains for any format).
> Dispatched via Agent (feature-dev:code-reviewer, Opus 4.8). Diff BASE `1279f7e` → HEAD `11cd75e`.
> I1+I2+m1 folded before Task 4.

---

**Reviewing:** `git diff 1279f7e 11cd75e` — Task 3, branch `restore-non-nums-tr-internal-key`. Files: `crates/mnemonic-toolkit/src/cmd/restore.rs`, `crates/mnemonic-toolkit/tests/cli_restore_taproot.rs`.

## Strengths

**Explicit guard placement is correct.** The bip388 guard at `restore.rs:900-909` sits in the `None` (route-around) arm of `build_multisig_import_payload` — the only arm with `template == None`. The `Some(t)` Template arm never reaches it; the Template arm's `bip388.rs` `Cosigner(idx)` emission is the working path (N6).

**The non-NUMS multipath-trunk hole is correctly closed.** The old NUMS-only incidental failure (bare x-only `Single`, no `/<0;1>/*`) would NOT fire for a non-NUMS card (a `MultiXPub` with a valid suffix). The new explicit guard catches BOTH `P2tr` and `P2trMulti` — the correct union.

**`P2trMulti` arm is load-bearing.** A `multi_a`-under-TapTree general card is `P2trMulti` (substring check, `mod.rs:237`); without that arm it would fall through to `descriptor_to_bip388_wallet_policy` and produce a garbled/erroring payload. The guard pre-empts that.

**Message quality good** (names the shape, points to descriptor/bitcoin-core, names the Template exception). **N6 assertion robust** (`tr(@` + `multi_a(` pins the structural shape).

## Issues

### Critical — None.

### Important

**I1 — N8 does not assert exit code or message (confidence 85).** `cli_restore_taproot.rs:665-666` asserts only `.failure()` for non-NUMS general-tr `--format green`. The NUMS counterpart asserts `.code(1).stderr(contains("singlesig-only"))`. Same code path (the `P2tr` green guard). `.failure()` passes on ANY non-zero exit (incl. an unexpected exit 2 or Io error). Fix: `.code(1).stderr(predicate::str::contains("singlesig-only"))`.

**I2 — No format-matrix coverage for non-NUMS general-tr on the template-requiring formats (confidence 82).** No test pins `--format {bsms,coldcard,jade,electrum,sparrow,specter}` against a non-NUMS general-tr md1. Code inspection confirms all refuse (template-requiring formats via `inputs.template.ok_or_else`; bsms via the `P2tr|P2trMulti` gate at `bsms.rs:77-84`; specter via `collect_missing → MissingField::WalletName`), but they're untested at integration level — a future emitter change dropping a template guard would not be caught. Fix: add a compact matrix test iterating those formats against a non-NUMS general-tr md1, asserting `failure()` each (specter included).

### Minor

**m1 — Comment at `restore.rs:875` attributes specter's refusal to "template/is_multisig branches" (confidence 50).** Specter refuses via `collect_missing → MissingField::WalletName`, not a template gate. Comment not wrong (specter does refuse) but the attribution is inaccurate for specter specifically. Low impact.

## Explicit Answer: Silent-Wrong-Emit Paths After This Change
Full matrix (general route-around taproot, P2tr/P2trMulti): bip388 → BadInput exit 1 (guard); green → BadInput exit 1 (P2tr guard); descriptor/bitcoin-core → faithful (correct); bsms → BsmsTaprootRefused; coldcard/jade/electrum/sparrow → template.ok_or_else refusal; specter → MissingField::WalletName loud refusal. Template path (Some(t)) unaffected, N6 pins bip388, @-in-both guard refuses the only dangerous shape. **There is NO path by which a non-NUMS taproot (route-around OR template) silently emits a wrong/misleading payload for ANY format after this change.**

## Assessment: **Approve with recommended fixes.**
Core refusal logic correct; no silent-wrong-emit path remains. The two Important issues are test-quality weaknesses (N8 weak assertion; unpinned should-refuse matrix), not behavioral bugs — but for this funds-safety cycle, pinning those paths is the right discipline. Minor m1 is a comment accuracy tweak.
