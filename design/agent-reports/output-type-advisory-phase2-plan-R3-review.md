# PLAN R3 review — output-type advisory Phase 2 (mk + md) + Tier-0

> Persisted verbatim from the opus architect R3 review of
> `design/IMPLEMENTATION_PLAN_output_type_advisory_phase2_mk_md.md`.
> R2 returned RED (0C/1I) with one blocker (I1-R2) + two Minors (m8, m9); all three are
> now folded. R3 charge: verify the three folds against live source / probe, do a final
> no-drift sweep, spot-confirm the R2-cleared items remain intact, and gate GREEN only if
> genuinely ready for code.
> Repos verified at the plan's header SHAs: toolkit `64943f2` (master), mk `e5620ce` (main),
> md `c599292` (main), ms-cli precedent `mnemonic-secret/ms-cli`. md binary rebuilt with
> `--features cli-compiler`; toolkit `mnemonic` binary built against the live 0.34 pin for the
> Tier-0 failure-mode sweep (no source/lockfile mutation — read-only probe on the existing pin).

## Verdict: GREEN (0C/0I)

The three R2 folds are each correct against live source and empirical probe; the no-drift sweep
finds no unresolvable placeholder, no contradiction introduced by the folds, and every item R2
already cleared remains intact (unedited). The plan is runnable end-to-end: every CLI invocation
in every cell now has a concrete, exit-verified form, and the Tier-0 FAIL→PASS gate is empirically
sound. No new Critical or Important.

---

## R2 fold resolution (I1-R2, m8, m9 — each ✓/✗)

| Fold | Status | Evidence (live source / probe) |
|---|---|---|
| **I1-R2** md compile cell concrete invocation | **✓** | Plan line 312 now states `md compile "pk(@0)" --context segwitv0` (Class = Template) in the "md fixtures" note. LIVE: `crates/md-cli/src/main.rs:153-165` — `Compile { expr: String (positional, required), context: String #[arg(... required = true)], unspendable_key: Option<String>, json: bool }`; so BOTH `<EXPR>` and `--context` are required. `crates/md-cli/src/cmd/compile.rs` returns `Ok(0)` in the json branch (`:21`) AND text branch (`:26`) → Template advisory fires in both modes. File `tests/cmd_compile.rs:1-2` is `#![allow(missing_docs)] #![cfg(feature = "cli-compiler")]`; `:9` is the exact `.args(["compile", "pk(@0)", "--context", "segwitv0"])` the plan lifts. The dispatch (`main.rs:347-373`) gates the `cmd::compile::run` call behind `#[cfg(feature = "cli-compiler")]` and returns `BadArg` under `not(feature)` — so the emit (which lives inside the handler) and its cells are correctly cfg-gated everywhere. EMPIRICAL (`md --features cli-compiler`): text → stdout `wsh(pk(@0))`, exit 0; `--json` → `{ "context":"segwitv0","schema":"md-cli/1","template":"wsh(pk(@0))" }`, exit 0; both clean stderr. Negative: `md compile "pk(@0)"` (no `--context`) → `error: the following required arguments were not provided: --context <CTX>`, exit 2. The blocker is closed. |
| **m8** md verify inert cell template | **✓** | Plan line 331 now uses `md verify md1yqpqqxqq8xtwhw4xwn4qh --template "wpkh(@0/<0;1>/*)"`. LIVE: `smoke.rs:18-19` — `md encode "wpkh(@0/<0;1>/*)"` → `md1yqpqqxqq8xtwhw4xwn4qh` (so that IS MD1_FIXTURE's actual template). `md verify` clap requires `--template` (`main.rs:122 #[arg(long, required = true)]`). EMPIRICAL: `md verify md1yqpqqxqq8xtwhw4xwn4qh --template "wpkh(@0/<0;1>/*)"` → stdout `OK`, exit 0. verify handler (`cmd/verify.rs`) returns `Ok(0)`/`Err(Mismatch)` with NO advisory emit at either, so the inert assertion holds at any exit (m8 was Minor for exactly this reason); the chosen template makes it a clean exit-0 cell. |
| **m9** Tier-0 0.34 failure mode | **✓** | Plan C1-Step-2 (line 353-354) now says 0.34 returns exit 2 `PostCorrectionDecodeFailed` / `wire-format version mismatch: got 2, expected 4` (not "UnparseableInput / no correction"). EMPIRICAL on the live 0.34 pin (`Cargo.lock` resolves `md-codec 0.34.0`; toolkit binary built against it): a full sweep of all 21 single-data-symbol cyclic-shift corruptions of `md1yqpqqxqq8xtwhw4xwn4qh` → **21/21 exit 2 `error: repair: post-correction decode failed: wire-format version mismatch: got 2, expected 4`**. (Stronger still: the *uncorrupted* fixture also fails identically — the non-chunked decode path is wholesale broken on 0.34, not just for corrupted input.) None exits 5; none reports UnparseableInput. So C1-Step-1's TDD assertion (`exit 5 + recovers`) correctly FAILS on 0.34 (gets exit 2, no recovery) and would PASS on 0.35 — the red→green gate is sound, and the narrative now names the observed mode. |

## No-drift sweep result

- **Residual placeholder tokens:** `grep -nE '<[a-z_]+>|<placeholder>|<TODO>|TBD|FIXME'` over the plan returns 5 hits (lines 218, 331, 334, 353), **all benign and resolvable**:
  - `:218`, `:334` — `<toolkit>` is the documented cross-repo path-root convention for review-file destinations, not an unresolved fixture.
  - `:331` — `wpkh(@0/<0;1>/*)` is the literal BIP-389 multipath descriptor syntax (the `<0;1>` is a real descriptor token, not a placeholder); `<tempdir>` is named in-line as `tempfile::tempdir()`.
  - `:353` — `<corrupted>` is prose naming the in-test corrupted-fixture variable the C1 TDD step constructs; the full construction mechanism is given in the same step. Acceptable for a "write the failing test" instruction.
  No unresolvable fixture/invocation placeholder remains; every emitting/inert cell across A1–A3, B1–B3, and C1 now has a concrete, exit-verified invocation.
- **Fold-introduced contradictions:** none.
  - md-fixtures note (line 312, compile = `md compile "pk(@0)" --context segwitv0`, Template) vs B2 table (line 298, compile = Template, cfg-gated `compile_emits_template`): the note supplies the concrete invocation the table referenced abstractly — consistent.
  - C1-Step-1 (assert exit 5 + recovers) vs C1-Step-2 (on 0.34 it FAILS, exit 2 PostCorrectionDecodeFailed): complementary TDD red→green, not contradictory — empirically the corrupted fixture yields exit 2 on 0.34 (RED as Step-2 claims) and exit 5 on 0.35.
  - m8 template `wpkh(@0/<0;1>/*)` is consistent with MD1_FIXTURE's documented origin at plan lines 250-251 and live `smoke.rs:19`.

## Spot-confirmation of R2-cleared items (unedited — still intact)

| Item | Live re-confirmation |
|---|---|
| Dead-code mechanism | ms-cli precedent `advisory.rs:26-45`: `#[allow(dead_code)]` enum `{PrivateKeyMaterial, WatchOnly, Template}` + live caller in same task; both CLIs carry crate-level `#![allow(missing_docs)]` (md `main.rs:1`). Plan module (lines 106-125) is a faithful copy. ✓ |
| Byte-parity | ms-cli `advisory.rs:41-44` (`\u{2014}`) == toolkit `secret_advisory.rs:100-102` (literal `—` U+2014) == plan module lines 119-123 == plan byte-parity constants 191-198. Cross-repo em-dash parity holds. ✓ |
| 5-site pins | `install.sh:35` md v0.6.1, `:38` ms v0.5.0 (untouched), `:41` mk v0.6.0; `manual.yml:77` mk v0.6.0, `:84` md v0.6.1 (`--features cli-compiler`); `quickstart.yml:71` mk v0.6.0 — all 5 current versions verified exact; plan C2-Step-1 bump targets correct. ✓ |
| sibling-pin gate inline | `.github/workflows/sibling-pin-check.yml:42 run: |` is inline bash (not a standalone script) — C2-Step-2 guidance correct. ✓ |
| Sole transcript | (doc-level, per R0/R1/R2) `24-recover-md1` the only `$MD_BIN` transcript; `$MK_BIN` never invoked. ✓ |
| FOLLOWUP closures | sibling-sweep slug at `FOLLOWUPS.md:3394`; Tier-0 false-resolved entry header at `:396` (Status `resolved v0.24.0 cycle` line 403; Resolution line 402 claims non-chunked repair works "end-to-end" — empirically FALSE on 0.34, the precise prose C1-Step-7 corrects). ✓ |
| Versions | mk-cli `0.6.0`→0.6.1, md-cli `0.6.1`→0.6.2, toolkit `0.38.2`→0.38.3 — current versions confirmed in each Cargo.toml; PATCH bumps correct (stderr-only consumer + additive Tier-0). ✓ |
| Tier-0 additive premise | toolkit `repair.rs` `repair_via_md_codec` (~:803) is a thin wrapper over `md_codec::decode_with_correction`; no toolkit code change beyond the dep bump. md-codec 0.35.0 is published (`cargo search` + local registry cache `md-codec-0.35.0.crate`) so C1-Step-4 re-resolution will succeed. ✓ |
| Per-phase + end-of-cycle reviews | A3-Step-5 (mk phase-A R0), B3-Step-5 (md phase-B R0), C4-Step-4 (end-of-cycle R0) each persist to `design/agent-reports/` and loop to 0C/0I before tag. ✓ |
| Ship-gating on user authorization | A3/B3/C4 tag steps + line 417 self-review note: executor stops at the pre-tag commit and asks. ✓ |

## Critical / Important (new or unresolved)

None.

## Minor (new or unresolved)

None blocking. (Two opportunistic observations, neither requires a fold:)
- C1-Step-1 instructs "generate a non-chunked md1 via `md encode` of a small payload … re-grep an existing small-template fixture; confirm it's non-chunked." The canonical such fixture is `md encode "wpkh(@0/<0;1>/*)"` → `md1yqpqqxqq8xtwhw4xwn4qh` (= MD1_FIXTURE, single string, no chunk header) — confirmed non-chunked by direct probe. The implementer can reuse MD1_FIXTURE directly; the "re-grep" language is fine as-is.
- C1-Step-2's parenthetical "ALL 21 single-symbol corruptions exit 2 on 0.34" matches my independent 21/21 sweep exactly; no change needed.

## Notes

- **Convergence reached.** R0 (2C/3I) → R1 (0C/3I) → R2 (0C/1I) → R3 GREEN. The R2 blocker (the lone compile cell with no stated invocation — the last instance of the fixture-fidelity class that recurred across all three prior rounds) is closed by a single verbatim lift from `cmd_compile.rs:9`, and the two Minors (m8 verify template, m9 0.34 failure-mode naming) are folded. No fold introduced drift.
- **Tier-0 evidence is unusually strong.** Both the latent bug (0.34: 21/21 exit-2 PostCorrectionDecodeFailed, plus the clean fixture also failing) and the false-`resolved` FOLLOWUP prose (line 402 claims end-to-end success that does not exist on the live pin) are empirically demonstrated. The plan's C1 TDD shape (red on 0.34, green on 0.35) and C1-Step-7 prose correction are both warranted.
- Cleared for implementation. Code may begin at Phase A per the R0-gate rule; the per-phase and end-of-cycle reviewer loops + ship-on-authorization gates remain in force downstream.
