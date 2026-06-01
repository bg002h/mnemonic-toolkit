# PLAN R2 review — output-type advisory Phase 2 (mk + md) + Tier-0

> Persisted verbatim from the opus architect R2 review of
> `design/IMPLEMENTATION_PLAN_output_type_advisory_phase2_mk_md.md` (folded twice:
> R0 RED 2C/3I → R1 RED 0C/3I). R2 is charged with CONVERGING the fixture-fidelity /
> test-invocation class that R0 and R1 both found. Method: verify each R1 fold against
> live source AND empirically exercise EVERY CLI invocation across all plan cells
> against the live `mk`/`md`/`mnemonic` binaries built from the header SHAs.
> Repos at: toolkit `64943f2` (master), mk `e5620ce` (main), md `c599292` (main),
> ms-cli precedent `mnemonic-secret/ms-cli`. Binaries built: `mk target/debug/mk`,
> `md target/debug/md` (default + `--features cli-compiler`), `mnemonic target/debug/mnemonic`
> (built throwaway against md-codec 0.35 to prove the Tier-0 FAIL→PASS gate, then reverted).

## Verdict: RED (0C / 1I)

The R1 folds are **all substantively correct** against live source (I4/I5/I6/m2'/m6/m7 each ✓),
and the comprehensive invocation sweep confirms that **every cell the plan gives a concrete
invocation for would exit exactly as asserted** — verified empirically, not by inspection.
The design skeleton, dead-code mechanism, phasing, 5-site lockstep, sole-transcript scope,
byte-parity, FOLLOWUP closures, and version targets (mk 0.6.1 / md 0.6.2 / toolkit 0.38.3)
all re-confirm.

But the convergence is **one cell short**. The same fixture-fidelity class that R0 (C1/C2)
and R1 (I4/I5/I6) found — an output cell whose CLI invocation is unstated and whose naive form
exits non-zero — survives on **exactly one md cell: `compile_emits_template`**. Every other
md output cell (decode/inspect/bytecode/repair/encode/address) received a concrete runnable
invocation in the I5/I6 fold; `compile` did not. `md compile` requires BOTH `--context <CTX>`
AND `<EXPR>` (empirically: omitting either → clap **exit 2**), and the plan names the cell but
never states the args anywhere (grep-confirmed: lines 19/289/298/315/317/321/331/415 reference
`compile`, none give an invocation). An engineer writing the cell from the plan as-is has no
runnable invocation — the identical blocker class this R2 round exists to close.

Fix is one line (add the concrete invocation, recoverable verbatim from the live
`tests/cmd_compile.rs:9`): **`md compile pk(@0) --context segwitv0`** (text → `wsh(pk(@0))`,
exit 0; `--json` variant → exit 0; both clean stderr → Template advisory fires as sole line).
Two Minors below are non-blocking.

---

## R1 fold resolution (live-source / empirical)

| Fold | Status | Evidence |
|---|---|---|
| **I4** mk encode cell | **✓** | EMPIRICAL: `mk encode --xpub <V2_84_MAIN> --origin-path "m/84h/0h/0h" --policy-id-stub deadbeef --privacy-preserving` → **exit 0**, emits a 2-chunk mk1 on stdout (WatchOnly advisory will fire). `--privacy-preserving` is compatible with omitting `--origin-fingerprint` (they are the mutually-exclusive *pair*; supplying neither + privacy-preserving is fine). Negative confirmed: same line WITHOUT `--policy-id-stub`/`--from-md1` → `error: at least one of --policy-id-stub or --from-md1 is required`, **exit 64**. Plan line 167 matches. |
| **I5** md MD1_FIXTURE | **✓** | `MD1_FIXTURE = "md1yqpqqxqq8xtwhw4xwn4qh"` is the sole valid md1 literal in md `tests/` (at `smoke.rs:19`, = output of `md encode "wpkh(@0/<0;1>/*)"`). EMPIRICAL: `md decode <F>` exit 0; `md decode --json <F>` exit 0; `md inspect <F>` / `md inspect --json <F>` exit 0; `md bytecode <F>` / `--json` exit 0. Template advisory fires in every mode. Plan lines 249-251 cite `smoke.rs:19` correctly (no longer the wrong `cmd_decode.rs`). |
| **I6** md address cell | **✓** | `account_xpub` is at `cmd_address.rs:12` (plan correct); the proven invocation is `cmd_address.rs:66-85`. EMPIRICAL: derived the abandon `m/84'/0'/0'` mainnet account xpub (`xpub6CatWdiZiodmU…7PW6V`) and ran `md address --template "wpkh(@0/<0;1>/*)" --key @0=<xpub>` → **exit 0**, stdout `bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu` (BIP-84 abandon receive-0), clean stderr; `--json` variant → exit 0, clean stderr. WatchOnly (derived from a public xpub, no spend key). The existing test `address_template_mode_emits_bip84_receive_0` (uses the exact plan invocation) passes. `bip39`/`bitcoin` are md-cli workspace deps (`Cargo.toml:33-34`). |
| **m2'** md compile-feature clippy | **✓** | B3-Step-3 (line 331) now includes the explicit `cargo clippy -p md-cli --all-targets --features cli-compiler -- -D warnings` line (the R0 m2 ask), in addition to the default-feature clippy + both test runs. |
| **m6** mk repair `flip_at` cite | **✓** | `fn flip_at` is at `cli_repair.rs:46` (verified); plan line 169 cites `crates/mk-cli/tests/cli_repair.rs:46` — correct (the R1 review text said `:48` for the body, but the plan author used the precise fn-declaration line `:46`). EMPIRICAL: a single in-alphabet cyclic-shift of one DATA symbol of `mk1_fixture()[0]` → `mk repair <chunk>` **exit 5** with `1 correction`, recovers the original, clean stderr → WatchOnly advisory fires as the sole stderr line. The corruption keeps the chunk parseable-but-BCH-invalid (→ exit 5, NOT exit 2). |
| **m7** mk fixture multi-chunk note | **✓** | The `card(V2_84_MAIN, "m/84h/0h/0h")` fixture emits **2 chunks** (verified: regenerated it via the equivalent `mk encode … --policy-id-stub deadbeef --origin-fingerprint 73c5da0a`). Plan line 79 carries the explicit "multi-chunk `Vec<String>` … spread via `.args(mk1_fixture())`, do NOT simplify to a single `.arg(...)`" note. EMPIRICAL: `mk decode/inspect/derive/address/verify` all accept the 2-chunk spread → exit 0. |

## Comprehensive invocation sweep (every cell → expected exit/advisory → ✓ vs live)

**Phase A — mk (all WatchOnly; single-return handlers = one cell each, no json variant):**

| Cell | Invocation (live-verified) | Exit | Advisory | ✓ |
|---|---|---|---|---|
| A1 decode | `mk decode <chunk0> <chunk1>` | 0 | WatchOnly | ✓ |
| A1/A3 decode --json | `mk decode --json <c0> <c1>` | 0 | WatchOnly | ✓ |
| A2 encode | `mk encode --xpub <V2> --origin-path m/84h/0h/0h --policy-id-stub deadbeef --privacy-preserving` | 0 | WatchOnly | ✓ (I4) |
| A2 inspect | `mk inspect <c0> <c1>` | 0 | WatchOnly | ✓ |
| A2 repair | `mk repair <1-data-symbol-corrupted c0>` | **5** | WatchOnly | ✓ (m6) |
| A2 derive | `mk derive <c0> <c1> --index 0` | 0 | WatchOnly | ✓ |
| A2 address | `mk address <c0> <c1> --count 1` (84' → p2wpkh, no `--address-type`) | 0 | WatchOnly; **NO depth advisory** (xpub depth=3 ✓ empty stderr) | ✓ (C2) |
| A3 verify (inert) | `mk verify <c0> <c1>` | 0 | none (stdout "OK", empty stderr) | ✓ |
| A3 vectors (inert) | `mk vectors` | 0 | none (34KB stdout corpus, empty stderr) | ✓ |
| A3 gui-schema (inert) | `mk gui-schema` | 0 | none (stdout schema, empty stderr) | ✓ |

> mk emit-sites re-grepped vs live `e5620ce`: decode `:38`, encode `:97`, inspect `:40`,
> repair `:86` (helpers `:149`/`:202` return `Ok(())` at `:172`/`:231`; error `?`-propagated at
> `:67` never reaches `:86` — I2 ✓), derive `:99`, address `:126`. mk `vectors` is a stdout
> corpus (NOT a file-writer) so the inert cell needs NO `--out` (correctly scoped — only md
> vectors writes files).

**Phase B — md (Template ×6, address WatchOnly; json early-`return Ok(0)` → emit at BOTH sites):**

| Cell | Invocation (live-verified) | Exit | Advisory | ✓ |
|---|---|---|---|---|
| B1 decode text | `md decode md1yqpqqxqq8xtwhw4xwn4qh` | 0 | Template | ✓ |
| B1 decode json | `md decode --json <F>` | 0 | Template | ✓ |
| B2 encode text/json | `md encode "wpkh(@0/<0;1>/*)"` (+ `--json`) | 0 | Template | ✓ |
| B2 inspect text/json | `md inspect <F>` (+ `--json`) | 0 | Template | ✓ |
| B2 bytecode text/json | `md bytecode <F>` (+ `--json`) | 0 | Template | ✓ |
| B2 repair | `md repair <1-data-symbol-corrupted F>` | **5** | Template (report on stdout; advisory sole stderr line) | ✓ |
| B2 address text/json | `md address --template "wpkh(@0/<0;1>/*)" --key @0=<xpub>` (+ `--json`) | 0 | **WatchOnly** | ✓ (I6) |
| **B2 compile** | **UNSTATED in plan**; naive `md compile pk(@0)` → **exit 2** (missing `--context`). Runnable form `md compile pk(@0) --context segwitv0` (+ `--json`) → exit 0, Template | 0 (only with the unstated args) | Template | **✗ → I1-R2** |
| B3 verify (inert) | `md verify <F> --template "wpkh(@0/<0;1>/*)"` | 0 ("OK") | none | ✓ (verify is inert at ANY exit — wrong T still emits no advisory; `<T>` looseness is Minor m8) |
| B3 vectors (inert) | `md vectors --out <tempdir>` | 0 | none (stdout 0 bytes, empty stderr, 40 files in tempdir — NO cwd pollution) | ✓ (I3) |
| B3 gui-schema (inert) | `md gui-schema` | 0 | none (stdout schema, empty stderr) | ✓ |

> md emit-sites re-grepped vs live `c599292`: decode json`:24`/text`:30`; encode json`:69`/text`:95`;
> inspect json`:42`/text`:59`; bytecode json`:30`/text`:41`; repair single `:153` (slot `:152`),
> error `return Ok(2)` `:121` inert; address json`:57`/text`:65`; compile (handler `cmd/compile.rs`,
> NOT lib `src/compile.rs`) json`:21`/text`:26` (println `:20`/`:25`) — I1 file ✓. md `verify` `:45`,
> `vectors run(out: Option<String>)` defaults `./vectors`.

**Phase C — toolkit + Tier-0:**

| Cell | Check | Result | ✓ |
|---|---|---|---|
| C1 Tier-0 premise | Non-chunked md1 `md1yqpqqxqq8xtwhw4xwn4qh` + 1-data-symbol corruption + `mnemonic repair --md1` | EMPIRICAL on live 0.34 pin: **ALL 21 data-symbol corruptions → exit 2** `post-correction decode failed: wire-format version mismatch: got 2, expected 4` (NOT a recovery). After throwaway bump to md-codec 0.35 + rebuild: **ALL 21 → exit 5, recover the original** (sweep 21/21). | ✓ premise sound (see Minor m9: the plan's C1-Step-2 "FAIL — UnparseableInput / no correction" mis-names the actual 0.34 mode, which is `PostCorrectionDecodeFailed` exit 2; correct it for an accurate TDD assertion) |
| C1 fix mechanism | md-codec 0.35 `chunk.rs:579-608` adds a `strings.len()==1` pre-pass inspecting corrected first-symbol bit-0; flag==0 → routes through `decode_md1_string` instead of `reassemble` (the 0.34 bug) | Source-verified; explains the exact `got 2, expected 4` failure 0.34 produced | ✓ |
| C1 false-resolved FOLLOWUP | `FOLLOWUPS.md:402-403` claims "RESOLVED in v0.24.0 … `mnemonic repair --md1` now accepts non-chunked … end-to-end", Status `resolved v0.24.0 cycle` | EMPIRICALLY FALSE on current 0.34 pin (exit 2) — confirms the false-resolved bug the plan's C1-Step-7 corrects. Pin was never bumped to 0.35. | ✓ premise validated |
| C1 pin + lockfile | `Cargo.toml:22 md-codec = "0.34.0"` (caret); `Cargo.lock` resolves 0.34.0; 0.35.0 published + cached | Confirmed; throwaway `cargo build` re-resolved 0.34.0→0.35.0 cleanly, then reverted | ✓ |
| C2 5 pin sites | `install.sh:35` md v0.6.1→v0.6.2; `:38` ms v0.5.0 (untouched); `:41` mk v0.6.0→v0.6.1; `manual.yml:77` mk; `:84` md (`--features cli-compiler`); `quickstart.yml:71` mk | All 5 lines + current versions verified exact | ✓ |
| C2 sibling-pin gate | `.github/workflows/sibling-pin-check.yml:42 run: \|` inline bash (`set -eu`, process-sub) | Confirmed inline (not a standalone script) — C2-Step-2 guidance correct | ✓ |
| C3 transcript | sole `$MD_BIN` transcript `24-recover-md1`; `$MK_BIN` never invoked in transcripts | (Per R0/R1 — re-confirmed at doc level; the re-capture/idempotency steps are correct) | ✓ |
| C4 versions | mk-cli `0.6.0`, md-cli `0.6.1`, toolkit `0.38.2` (current) → bump to `0.6.1`/`0.6.2`/`0.38.3` | Confirmed current versions; PATCH bumps correct | ✓ |

## Critical (new or unresolved)

None.

## Important (new or unresolved)

- **[I1-R2 — NEW] The md `compile` cell (`compile_emits_template`, B2 table line 298) has NO
  stated invocation, and `md compile` is not runnable without the unstated required flags.**
  Evidence (empirical, live `md --features cli-compiler`): `md compile` requires BOTH `<EXPR>`
  and `--context <CTX>` — `md compile pk(@0)` → `error: the following required arguments were not
  provided: --context <CTX>`, **exit 2**; `md compile --context segwitv0` (no expr) → exit 2.
  Grep of the plan (lines 19/289/298/315/317/321/331/415) confirms `compile` is referenced but
  NO invocation args appear anywhere — unlike every other md cell, which got a concrete invocation
  in the I5/I6 fold (decode→`MD1_FIXTURE`, encode→`"wpkh(@0/<0;1>/*)"`, address→`--template/--key`,
  repair→corrupted `MD1_FIXTURE`). **Why Important:** this is the exact fixture-fidelity / un-runnable
  -invocation class R0 (C1/C2) and R1 (I4/I5/I6) flagged — the cell cannot be written from the plan
  as-is, and the naive guess (`md compile <expr>`) exits 2, breaking the TDD `status.success()`
  assertion. It is the one cell this R2 convergence round must close to reach GREEN.
  **Fix:** B2 must state the concrete compile invocation — lift it verbatim from the live
  `crates/md-cli/tests/cmd_compile.rs:9`: **`md compile pk(@0) --context segwitv0`**
  (EMPIRICAL: text → stdout `wsh(pk(@0))`, exit 0; `--json` → exit 0, `{"context":"segwitv0",
  "schema":"md-cli/1",…}`; both clean stderr → Template advisory fires as sole line). The cell
  stays `#[cfg(feature="cli-compiler")]`-gated (test file model: `cmd_compile.rs:2`
  `#![cfg(feature = "cli-compiler")]`). One-line fold.

## Minor (fold opportunistically)

- **[m8] md `verify` inert cell leaves `<T>` as a meta-placeholder (line 330).** For the cell to
  exit 0 ("OK"), `<T>` must be `wpkh(@0/<0;1>/*)` (the template `MD1_FIXTURE` was encoded from —
  documented at plan lines 250-251). EMPIRICAL: correct T → exit 0; a *wrong* T → exit 1 MISMATCH —
  BUT verify emits NO advisory either way, so the cell's assertion ("stderr has no advisory line")
  passes regardless of T. Hence Minor, not Important. Recommend naming `--template "wpkh(@0/<0;1>/*)"`
  concretely for a clean exit-0 cell.

- **[m9] C1-Step-2 mis-describes the 0.34 failure mode.** Plan line 353 says "Expected: FAIL —
  UnparseableInput / no correction." EMPIRICAL: the actual 0.34 mode for a BCH-correctable
  single-data-symbol corruption is **exit 2 `post-correction decode failed: wire-format version
  mismatch: got 2, expected 4`** (= `RepairError::PostCorrectionDecodeFailed`, the second of the two
  modes the FOLLOWUP itself documents at `:399`), NOT `UnparseableInput`. The smoke test (asserts
  exit 5 + recovery) still correctly FAILS on 0.34 and PASSES on 0.35 — the TDD gate is sound — but
  the step's narrative should name `PostCorrectionDecodeFailed`/exit-2 so the assertion matches the
  observed behavior. Cosmetic; does not affect the test.

## Re-confirmed (held from R1)

- **Design skeleton** complete: each bin-only CLI gets its own `output_advisory.rs` (byte-for-byte
  copy of `ms-cli/src/advisory.rs:26-45`, verified); per-handler emit at every success return.
- **Dead-code mechanism**: enum-level `#[allow(dead_code)]` + a live (main-reachable) caller in the
  SAME task (A1/B1 wire decode alongside the module) — both crates have crate-level
  `#![allow(missing_docs)]` (mk `main.rs:7`, md `main.rs:1`) so m1 stays moot; module registration
  sites confirmed (mk `main.rs:9-11`, md `main.rs:3-9`, where `mod compile` at md `:4-5` is the
  compiler LIB, distinct from the `cmd/compile.rs` handler — I1-R0 ✓).
- **Byte-parity**: toolkit `secret_advisory.rs:100-102` literal `—` (U+2014) == ms-cli `\u{2014}`
  == plan module lines 119-123 == byte-parity constants 191-198. Cross-repo parity holds.
- **Phasing** mk→md→toolkit (hard tag dependency); **lockstep 5-site** pins; **sole transcript**
  `24-recover-md1`; **Tier-0 scope** (additive, toolkit consumes via unchanged
  `repair_via_md_codec`); **FOLLOWUP closures** (`output-type-stderr-advisory-sibling-sweep-mk-md`
  at `:3394`, Tier-0 false-resolved at `:396`); **version 0.38.3** — all confirmed.
- **Dev-deps**: mk (`assert_cmd`/`predicates`/`tempfile` dev; `mk-codec`/`bitcoin` deps);
  md (`assert_cmd`/`predicates`/`insta`/`tempfile` dev; `bitcoin`/`bip39` workspace deps) — all
  test imports resolve; no Cargo.toml dev-dep additions needed.

## Notes

- **The convergence is 95% there.** R1 closed I4/I5/I6 flawlessly; the fold's residual blind spot is
  uniform with the prior rounds — exactly one output cell (`compile`) was not given the concrete-
  invocation treatment the others received, and it is the one md subcommand with TWO required args
  whose omission is a hard clap exit-2. One folded line (`md compile pk(@0) --context segwitv0`,
  liftable verbatim from `cmd_compile.rs:9`) closes the class. Fold I1-R2 (+ m8/m9 opportunistically),
  re-dispatch for R3 — the plan is otherwise runnable end-to-end and every other invocation was
  empirically proven to exit as asserted.
- **Empirical-probe hygiene:** the Tier-0 FAIL→PASS proof required a throwaway `Cargo.toml`/`Cargo.lock`
  bump-build-revert; all three git trees are clean post-probe (the toolkit `target/debug/mnemonic`
  binary is now stale-built against 0.35 but cargo rebuilds it at impl time — no source/lockfile
  residue). The 0.34→exit-2 / 0.35→exit-5 split across all 21 corruption loci is the strongest
  possible confirmation of both the latent bug and the fix.
