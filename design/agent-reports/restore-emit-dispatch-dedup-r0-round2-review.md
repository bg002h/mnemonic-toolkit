# R0 Architect Review (round 2) — `SPEC_restore_emit_dispatch_dedup.md`

**Reviewer:** opus `feature-dev:code-reviewer` (mandatory pre-implementation R0 gate). **Date:** 2026-06-05.
**Branch:** `restore-emit-dispatch-dedup`. **Verdict:** **0 Critical / 0 Important** (+ 1 Minor). **GREEN — implementation may proceed.**

> Persisted verbatim per CLAUDE.md. Round-1 folds (I1/M1/M2) all verified correct + complete; no new drift. New M3 (use real `TREZOR_12` phrase for the RED cell) is non-blocking → folded without re-dispatch.

---

**VERDICT: 0 Critical / 0 Important** (+ 1 Minor)

**GREEN — implementation may proceed.**

The three Round-1 folds (I1, M1, M2) are all correct and complete, and the fold introduced no new drift. The verbatim-lift premise still holds against source. One new Minor (phrase-validity placeholder) surfaced on this adversarial pass — non-blocking, but worth a one-line SPEC note so the implementer doesn't plug in junk.

---

## Critical
None.

## Important
None.

## Minor

### M3 (new) — The Phase-1 RED cell needs a VALID 12-word phrase, or it goes RED→RED (never GREEN in Phase 2).
The SPEC writes the cell as `restore --from phrase=<12-word> …` — `<12-word>` is a placeholder. The reachability chain to the divergent emit arm is: clap gates → upfront refusals (`restore.rs:200-206`, `:222`) → **phrase parse + `rows` build** → `build_import_payload(format, &rows[0], …)` at `restore.rs:452-453`. An **invalid** `phrase=` fails at row-building BEFORE `:452`, so the cell would (a) go RED for the wrong reason against the current binary (parse error, not the coldcard message) and (b) stay RED after the refactor (never reaches `emit`) — i.e. Phase 2 never goes GREEN.

Not Critical (it's a placeholder, not a pinned invalid literal). Fix: add one sentence to §5/§7 — *"`<12-word>` must be a real BIP-39 vector; use the existing `TREZOR_12` constant (`tests/cli_restore.rs:17`), already driving the `--template bip84` cells at `:76` and `:483+`. An invalid phrase fails at row-building before `restore.rs:452` and the cell never reaches the emit arm."*

*(Optional, same off-by-N class as M2: §4/M1 cites the upfront refusal as `restore.rs:199-203`; the actual `if`-block is `:199-206` with the literal at `:202-204`. Navigable; fold or leave.)*

---

## Folds verified clean (Round-1 → source)

**I1 fold — correct + complete.** §5 and §7 now name the discriminator `"requires a multisig --template"` and state exit is 1 both ways. Verified against source:
- Substring PRESENT in the NEW unified arm site 3 routes to: `export_wallet.rs:549`. ABSENT from the OLD single-sig message `restore.rs:650` ("requires a multisig **wallet**; restore is single-sig"). The discriminator is genuine.
- Reachability confirmed: `--template bip84` is single-sig (clears `:200-206`); `--format`+`--template` set clears the `:222` exit-2 ModeViolation; `build_import_payload` is invoked at `:452-453`; `ColdcardEmitter::collect_missing` does not short-circuit (Round-1 confirmed `coldcard.rs` returns empty), so the `:649` divergent arm is live today. After refactor, `inputs.template = Some(row.template) = Some(Bip84)` (`restore.rs:598`) deterministically hits the helper's `_ =>` arm → unified message. The cell goes RED-for-the-right-reason (message substring; exit 1 unchanged). Substring proven as a working discriminator at `cli_export_wallet_coldcard.rs:536`.

**M1 fold — accurate.** §4 + Phase 3 (§7) note the two-clause wrinkle. Source confirms: unified message (`export_wallet.rs:549`) first clause steers at `wsh-sortedmulti, wsh-multi, …` (multisig templates restore refuses upfront at `restore.rs:200-206`), second clause carries the correct `--format coldcard with bip44/bip49/bip84` pointer. Note is accurate.

**M2 fold — accurate.** §1 now cites `:649` for the `Err(bad(…))` arm and `:650` for the string literal. Source: `restore.rs:649` opens `Err(bad(`, literal at `:650`. Matches.

---

## What verified clean (no new drift; Round-1 items re-confirmed)

- **4 dispatch sites, divergence is EXACTLY site 3.** Collect_missing/emit positions all match §1: site 1 (`run`) collect `:508`/emit `:527`; site 2 (`run_from_import_json` `:584`) collect `:763`/emit `:783`; site 3 (`build_import_payload` `:587`) collect `:626`/emit `:645`/divergent `:649-651`; site 4 (`build_multisig_import_payload` `:673`) collect `:707`/emit `:728`. Site 4's coldcard-multisig arm (`:732-746`) is the 6-variant `CliTemplate` match — byte-identical to sites 1/2, confirming site 3 is the lone divergence.
- **Site 2 is test-covered.** The unified message also lives at `export_wallet.rs:805` (site 2); `cli_export_wallet_from_import_json.rs:916` allows `"requires a multisig --template"` as a refusal substring — so §5's "add a smoke cell if uncovered" is belt-and-suspenders, not load-bearing.
- **No import cycle.** `CliExportFormat` defined `export_wallet.rs:23`; `restore.rs:31` already imports it from `crate::cmd::export_wallet`, so `crate::cmd::export_wallet::emit_payload` is callable with no new cycle. §2 home decision sound.
- **Signature sufficient.** `fn emit_payload(inputs: &EmitInputs, format: CliExportFormat) -> Result<String, ToolkitError>` — both matches at all 4 sites reference only `inputs`, `format`/`args.format`, emitter types, `CliTemplate`, `ToolkitError`, and format-name literals. No per-site local. Holds.
- **Internal consistency post-fold.** §2 helper signature, §3 four call-site replacements, §4 Decision (a), §6 lockstep=NONE all mutually consistent. No section contradicts another; no section stale after the fold. SemVer PATCH (`Cargo.toml:3` = 0.46.0 → 0.46.1), no clap surface change, no new error variant — all hold.

---

**Gate status: GREEN — 0 Critical / 0 Important. Implementation may proceed.** The optional M3/M2-style line-cite tweaks can be folded opportunistically at Phase-1 write time (the implementer should use `TREZOR_12` for the RED cell phrase regardless); they do not gate.
