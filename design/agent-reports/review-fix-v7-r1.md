**⚠ 2 Important (I-1 fail-fast RED command; I-2 item-12 book commands fail as printed).**
r1: 0C/2I/3M/4N

# FIX-V7 review r1 (Opus, high): `design/FIX_v7_bundle_recovery.md` at 9c5aaad5 (redirected from 18bdaf18 by the coordinator; the delta 18bdaf18..9c5aaad5 only drops architect T10(1) and T11)

Reviewer reproduction: a detached worktree `/scratch/cache/review-fix-v7-r1/wt` on `origin/master` 12e013bc,
built per the FIX header (`git cherry-pick d19256e8 ac3ff70a && git cherry-pick ac3ff70a..18bdaf18`, both exit 0),
own `CARGO_TARGET_DIR=/scratch/cache/review-fix-v7-r1/target`, cargo 1.85.0, cargo-nextest 0.9.140.
Logs: `/scratch/cache/review-fix-v7-r1/*.log`.

## Findings

### I-1. Section 4's RED command is fail-fast, so its printed `30 tests run: 6 passed, 24 failed` is not reproducible [author text, 9f610b3a; Opus]

- **Where:** `design/FIX_v7_bundle_recovery.md:90` (the command). It is reused by §6.2 step 1 (`:197`, "Expect exit 100 and `30 tests run: 6 passed, 24 failed, …`"), step 2 (`:200`) and §7 item 13 (`:276`).
- **Failure (measured):** I ran the command exactly as printed on the base build tree (12e013bc + the header's cherry-picks, fresh target dir, `Compiling mnemonic-toolkit` present). It exited 100 with
  `Summary [   0.073s] 24/30 tests run: 6 passed, 18 failed, 4111 skipped` and `Cancelling due to test failure: 23 tests still running` (`/scratch/cache/review-fix-v7-r1/red.log`).
  No nextest config sets `fail-fast` (no `.config/nextest.toml`), so nextest's default fail-fast applies. 30 rows on 24 test threads means 6 rows queue. Whether they ever start depends on timing. The author's run happened to finish all 30; mine did not.
  A Sonnet builder following step 1 literally sees a count other than 30. Section 4 says "A matched count other than 30 is a FAIL", so the RED evidence step cannot pass as printed, and the 24 named failures cannot be checked.
- **Fix text** (replace the command at `:90`):
  `PATH=$HOME/.cargo/bin:$PATH cargo nextest run --locked --no-fail-fast -p mnemonic-toolkit -E "$(cat design/agent-reports/fix-doc-tk-v7-evidence/fix-rows-filter.txt)"`
  Run as printed:
  - On base: exit 100, `Summary [   0.125s] 30 tests run: 6 passed, 24 failed, 4111 skipped`. The 24 failures are exactly the FAIL rows of the table, with the same panic lines: t:96 ×10, t:167, t:218 ×5, t:240, t:424, t:433, t:453, t:467, t:494, t:519, `format.rs:510` (`red2.log`).
  - After step 2 with row 24 narrowed: exit 0, `30 tests run: 30 passed, 4111 skipped` (`green.log`).

### I-2. Section 7 item 12: three book-gate commands fail as printed outside a sibling-repo layout, and inside one they test the wrong inputs [author text, 50f355e5; the GUI sentence was re-touched by fold 18bdaf18 (T3); Opus]

- **Where:** `design/FIX_v7_bundle_recovery.md:258` (technical manual `make verify-examples`), `:266` (quickstart `make verify-examples`) and `:267` (GUI manual "`make lint` (`manual-gui.yml:160`) and `make html` (`:156`) are re-run locally").
- **Failure (measured, worktree under `/scratch/cache`, which §6.2 allows since it only forbids `/tmp`):**
  - Technical manual: bare `make verify-examples` exits 2 with `[verify-examples] FAILED (18 transcripts checked)`. The Makefile defaults `MD_BIN`/`MS_BIN`/`MK_BIN` to `cargo run --manifest-path $(WORKSPACE_ROOT)/<sibling>/Cargo.toml` (`docs/technical-manual/Makefile:49-52`). With no siblings next to the worktree, it prints `error: manifest path …/mnemonic-secret/Cargo.toml does not exist` (`tm-ve.log`).
  - Quickstart: bare `make verify-examples` also exits 2, for the same reason (`docs/quickstart/Makefile:41-44`; `qs-ve.log`).
  - GUI manual: `make lint MNEMONIC_BIN=true MD_BIN=true MS_BIN=true MK_BIN=true` exits 2. Phases 4, 5 and 9-12 fail with `MANUAL_GUI_UPSTREAM_ROOT not a directory: …/mnemonic-gui`, and the run ends `[lint] FAILED` (`gui-lint.log`). CI clones `mnemonic-gui-v0.63.0` first and exports that variable (`manual-gui.yml:97-114`). The FIX prints no such step.
  - In a worktree directly under `/scratch/code/shibboleth` (the author's layout), these commands pass for the wrong reason. They `cargo run` whatever HEAD each sibling checkout is on, not the pinned tags, and the GUI lint byte-compares against `../mnemonic-gui` at its current HEAD, not v0.63.0. Item 12's result then depends on unrelated checkouts.
  - Every printed failure is a hard FAIL that stops the builder; nothing wrong ships. Still, these are printed commands that fail as printed, so the grade stays I.
- **Fix text** (replace the three commands; each is CI's own form):
  - Technical manual, from `docs/technical-manual`:
    `make verify-examples MNEMONIC_BIN="$PWD/../../target/debug/mnemonic" MD_BIN=md MS_BIN=ms MK_BIN=mk EXAMPLES_DIR="$PWD/examples"` (`technical-manual.yml:204-209`).
    Expected: `[verify-examples] OK (18 transcripts pass)`.
  - Quickstart, from `docs/quickstart`:
    `make verify-examples MNEMONIC_BIN="$PWD/../../target/debug/mnemonic" MD_BIN=md MS_BIN=ms MK_BIN=mk FIXTURES_DIR="$PWD/../../crates/mnemonic-toolkit/tests/fixtures/wallet_import"` (`quickstart.yml:179-184`).
    Expected: `[verify-examples] OK (62 transcripts pass)`.
  - GUI manual:
    `G=/scratch/cache/tk-v7-build-gui-pinned; git clone --depth 1 --branch mnemonic-gui-v0.63.0 https://github.com/bg002h/mnemonic-gui "$G"`,
    then, from `docs/manual-gui`, `make html`, then
    `MANUAL_GUI_UPSTREAM_ROOT="$G" make lint MNEMONIC_BIN=true MD_BIN=true MS_BIN=true MK_BIN=true` (`manual-gui.yml:97-114`, `:156`, `:160`).
    Expected: `CSpell: Files checked: 104, Issues found: 0 in 0 files.` and `[lint] OK`.
  - Each was run as printed on the fully patched tree (T1, T8, T9, T10(2), the evidence patches, row 24 narrowed): tm exit 0 `OK (18 transcripts pass)`; quickstart exit 0 `OK (62 transcripts pass)`; GUI clone exit 0, `make html` exit 0, lint exit 0 `[lint] OK`. For the GUI clone I used `G=/scratch/cache/review-fix-v7-r1/gui-clone`, because the builder's path is the builder's (`fix-tm-ve.log`, `fix-qs-ve.log`, `fix-gui-lint.log`).

### Minors and Nits

- **M-1** [author, 50f355e5] `:259` expects `CSpell: Files checked: 26`. A tree that runs `make lint` first, in the order item 12 prints, gets `Files checked: 25`. The 26th file is the ignored, generated `src/99-build-banner.md`, which `make pdf`/`make html` create (author tree: `!! docs/technical-manual/src/99-build-banner.md`). Fails closed: a builder would stop on the count. Fix: "`Files checked: 25` (26 once `make pdf`/`make html` have generated `src/99-build-banner.md`)".
- **M-2** [author] Coverage map `:347` cites row 8 for "`tr` with a non-NUMS internal key is general". Row 8 covers `Slot` only; no row pins `InternalKey::LianaUnspendable` as general (2.1, ruling 1). A predicate that accepted it as plain would survive every row. Not graded I: ruling 1 records that a single-leaf Liana tree does not occur, and a multi-leaf tree is general anyway. Fix: one guard row with a Liana-unspendable internal key and a single `multi_a` leaf, expecting the general line, or a sentence saying no md1 can express it.
- **M-3** [ruling text, architect T1] Both GUI interim notes say "Toolkit releases after 0.105.1 print no `# Threshold` line for this policy". That is false for any toolkit release cut between 0.105.1 and the V7 merge (for example, a V5-only release, since §header lets V5 merge first). Fails safe: the note still tells the holder to keep everything. Fix, if the coordinator re-opens the text: "Toolkit releases that carry FIX-V7".
- **N-1** [author] §2.3 `:58` says "the other template formats keep their existing general-policy refusal, at exit 1, with the unchanged text" and then quotes sparrow's text. On a general envelope, `specter` exits 2 (missing `wallet_name`) and `green` exits 1 with its own multisig refusal. Behaviour is unchanged: I measured all 12 `--format` values, integer envelope vs `null` envelope, and every output was byte-identical (`export-matrix.log`). Only the sentence over-generalises.
- **N-2** [coordinator, 18bdaf18 T2] `:204`: `git status --short docs/manual-gui` prints `docs/manual-gui/tutorial/40-j3-degrading-vault.md` (repo-root paths), not `tutorial/40-…` as written.
- **N-3** [author] §6.2 step 2 never says to run `cargo fmt` after narrowing row 24. A one-line `assert!(out.status.success(), "stderr: {}", …)` fails item 1 (`cargo +1.95.0 fmt --all -- --check`; measured on my first narrowing). Add "then `cargo +1.95.0 fmt --all`".
- **N-4** [author] §7 "reuse the V5 build's root if it exists" names no path. The root exists at `/scratch/cache/tk-v5-build-ci-tools`, and its `.crates.toml` shows exactly the three pinned tags.

## Create-input table (every printed command)

| command | ran as printed | result |
|---|---|---|
| header cherry-pick chain on `origin/master` | yes (detached) | exit 0 both picks |
| §4 RED/GREEN filter command | yes | **I-1** (24/30 run, fail-fast); with `--no-fail-fast` 30: 6/24 then 30/30 |
| `git apply` production, tests, docs, examples, changelog | yes | all apply on 12e013bc+picks |
| T8, T9, T1 (×2), T10(2) asserting replaces | yes | each old string once |
| §7.1 fmt (`+1.95.0`) | yes | exit 0 (after `cargo fmt`, N-3) |
| §7.2/7.3 build + `nextest --workspace --no-fail-fast` | yes | `4215 tests run: 4215 passed, 38 skipped` (`full.log`) |
| §7.4 doc tests; §7.5 clippy `-D warnings` | yes | exit 0; exit 0 |
| §7.6 install-*.test.sh ×6, docs-only, code-docs-check (×2) | yes | all exit 0 (`73 passed`, `41 passed`, `ok (446 files…)`) |
| §7.6 mlock g2 rows, `mlock_g6_invariant` | no | untouched by V7; author-measured |
| §7.7 doc-gate-guard | yes | `158 passed, 0 failed` |
| §7.8 `make audit` | yes | `[lint] OK`, `OK (62 transcripts pass)`, `9 danglers … match baseline` |
| §7.9 doc-flag-lint mutation self-test | yes | `15 passed, 0 failed` |
| §7.10 examples regen + `git diff --exit-code` (after `git add`) | yes | exit 0 (no diff) |
| §7.11 manual pdf/html; tm pdf/html | yes | all exit 0 |
| §7.12 tm `make lint` | yes | `[lint] OK`, cspell 25 files (M-1) |
| §7.12 tm/quickstart `make verify-examples`, GUI `make lint` | yes | **I-2** (fail bare); CI form passes |
| §7.12 symbol-ref-check CI-layout emulation | yes | `OK (301 \`::\` anchors checked, 427 skipped)` |
| §7.12 quickstart `make lint` | yes | `[lint] OK` |
| §7.14 mutation gate | no | brief forbids (OOM); no production Rust on the branch yet |

## Case table (2.1-2.4, measured on the patched binary unless marked)

| input | decided by | card | JSON `multisig.threshold` |
|---|---|---|---|
| `wsh(multi(2,K0,K1,K2))`, `sh(multi(2,..))`, `sh(wsh(sortedmulti(2,..)))`, `tr(NUMS,multi_a(1,K0,K1))` | 2.1 r1/r2 | `2 of 3` / `2 of 2` / `2 of 3` / `1 of 2` + any-line | k |
| `wsh(thresh(2,pk,s:pk,s:pk))`, `tr(NUMS,thresh(..))`, `wsh(thresh(1,pk,s:pk))` | 2.1 pure-key | plain | k |
| thresh with `sln:older`/`sln:after` arg; `and_v(v:pk,pk)` | 2.1 general | general line, no Threshold | null |
| `wsh(multi(1,K0))`, `tr(NUMS,pk(K0))`, `tr(K0)`, `wsh(c:pk_k(K0))` | n == 1 | neither line | `multisig` absent |
| invalid thresh typing (`pk,pk,pk`; `s:` first; `s:pkh`) | md typecheck | exit 2 refusal (fails closed) | — |
| repeated key (`K0/<0;1>`, `K0/<2;3>` concrete; `@0` twice in template form; BIP-388 JSON) | BIP-388 distinctness (bip-0388.mediawiki:193-195) | exit 2 refusal | — |
| import-wallet text, `wsh(multi(1,K0))` / `wpkh` / general n==1 | 2.4 | `threshold=1` / `none` / `none` | — |
| export-wallet, 12 formats × {integer, null} envelope, general and plain | 2.3 | byte-identical per format (N-1) | — |

No undecided cell. The "distinct single keys" condition in ruling 1, absent from 2.1's text, is enforced upstream by the toolkit's BIP-388 distinct-key refusals: three spellings measured, all exit 2.

## Advice lens

- General line, followed literally ("keep all N signing keys, any hash preimages it names, and md1"): the holder keeps every input that any spend path can need. It cannot lead to a lost spend path. N counts every slot, including a `tr` slot internal key (J4: 12).
- Plain line: reached only by shapes whose spend condition is exactly one k-of-n over distinct keys (table above), so "any k of n" is true. Template mode is plain by construction.
- STOP-2 (b) text in the spec (T8), manual (T9) and CHANGELOG (T10(2)): matches runtime. `--threshold 1`/`9` on a plain envelope prints `2 of 3`; on a general envelope it prints the general line (row 24 narrowed, GREEN).
- GUI interim notes (T1): the facts hold against the J3/J4 descriptors (`tut-j3-10`/`tut-j4-15` stdout): tiers 1-2 need `sha256`, tier 4 is `multi(1,K8..K10)`, and J4's internal key is a slot. See M-3 for the release wording.
- Coordinator's sentence in §7 item 12 ("under docs/manual-gui only the two interim notes change"): true (`git status --short docs/manual-gui` lists exactly those two files). Its "re-run locally" commands are I-2.
- No old-card notice is graded, per the coordinator's redirect to 9c5aaad5.

## Not covered by me

- mlock g2/g6 rows, the release/miri/musl CI-only jobs and `bitcoind-differential.yml` (CI only).
- The mutation gate (brief).

## Provenance

This is round 1, not a re-review, so no prior fix commit is graded. Of the findings, 0 come from the coordinator's fold commits 18bdaf18/9c5aaad5, except N-2 and the GUI sentence inside I-2, which 18bdaf18 re-touched. The rest come from the FIX author's text (Opus, per the commit trailers; effort not recorded in the commits) or from architect T1 (M-3).

Design concern (ungraded): §8's Note stands. `restore` writes `null` for every `thresh`, while `bundle`/`import-wallet` now write k for a pure-key `thresh`, so the toolkit's two JSON emitters disagree on the same descriptor.

C / I / M / N: 0 / 2 / 3 / 4
