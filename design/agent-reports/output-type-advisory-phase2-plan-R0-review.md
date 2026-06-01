# PLAN R0 review — output-type advisory Phase 2 (mk + md) + Tier-0

> Persisted verbatim from the opus architect R0 review of `design/IMPLEMENTATION_PLAN_output_type_advisory_phase2_mk_md.md`, BEFORE any fold/implementation, per CLAUDE.md R0-gate discipline. Reviewer ran against LIVE source at toolkit `64943f2`, mk `e5620ce`, md `c599292`, ms-cli precedent `mnemonic-secret 4e5266a`. Dead-code mechanism verified by an empirical clippy probe (`/tmp/deadcode_probe2`).

## Verdict: RED (2C / 3I)

The plan is structurally sound, honors all SPEC-R0 findings in design (dead-code allow, ms-cli template not toolkit, multi-return enumeration, 5 pin sites, git-tag deliverable), and its TDD/phase/lockstep/Tier-0 mechanics are mostly correct. But two Critical fixture-fidelity defects and three Important defects would block a clean first cut: (C1) the mk fixture placeholder is unresolvable as written — no bare `mk1` literal exists in mk-cli tests; (C2) the `mk address` cell's reuse-an-existing-fixture instruction picks a multisig fixture that the multisig-refuse guard rejects → the cell fails; (I1) the `md compile` emit-site/staging points at the WRONG file (`src/compile.rs` lib vs `src/cmd/compile.rs` handler); (I2) the mk `repair` emit-site citation points at the helper `Ok(())` returns (`:169`/`:230`) instead of the single `run()` success return (`:86`); (I3) the `md vectors` inert cell writes to cwd (`./vectors`) and will pollute the test working dir.

---

## Critical (must fix before code)

- **[C1] The mk `MK1_FIXTURE` placeholder is unresolvable — there is NO bare `mk1…` literal in `crates/mk-cli/tests/`, so "lift the mk1 literal an existing passing decode test uses" cannot be followed.** Evidence: `grep -rhoE 'mk1[a-z0-9]{20,}' crates/mk-cli/tests/` returns **zero** matches. Every mk-cli test generates mk1 strings at runtime: `round_trip.rs:51-72` shells `mk encode --xpub … --origin-path …` and captures stdout; `cli_repair.rs:33-42 generate_valid_mk1_chunks()` constructs a `mk_codec::KeyCard` and calls `mk_codec::encode(&card)`. The constants (`V1_XPUB`, `V1_PATH`) are redeclared **per-file** (`round_trip.rs:14-16`, `cli_repair.rs:28-29`), not in a shared module. The plan's Task A1-Step-1 scaffold imports only `std::process::Command` + `assert_cmd::cargo::CommandCargoExt` and binds `MK1_FIXTURE` to a paste-a-literal placeholder — but no such literal can be lifted. **Why Critical:** the placeholder is a plan-failure (writing-plans forbids placeholders); the TDD Step-2 ("run it; verify it fails") won't even compile with an empty/invalid `MK1_FIXTURE`. **Fix:** specify the actual mk fixture idiom: either (a) the `cli_repair.rs` lib idiom — add `mk_codec`/`bitcoin::bip32` as the fixture source, construct a single-chunk `KeyCard`, call `mk_codec::encode`; or (b) the `round_trip.rs` idiom — a one-time `mk encode` invocation in the test that captures stdout into a `fn fixture_mk1() -> String`. Also note V1 emits **two** chunks (`cli_repair.rs` comment "Two-chunk emission… chunk 0 long, chunk 1 regular"); a `decode`/`inspect` cell passing a single `MK1_FIXTURE` arg must pass ALL chunks or use a single-chunk card. The plan must name the concrete fixture-generation function, not a placeholder literal.

- **[C2] The `mk address` cell instruction "reuse an existing… fixture" selects a multisig fixture that `mk address` REFUSES → the cell fails (non-success exit), not success+advisory.** Evidence: the only mk1 fixtures in-tree derive from origin path `m/48'/0'/0'/2'` (`cli_repair.rs:29`, `round_trip.rs:16` — both BIP-48 **multisig** cosigner paths). `mk address`'s `resolve_address_type` does "multisig-refuse FIRST, then explicit" (`cmd/address.rs:129-138`): a BIP-48/BIP-87 origin → hard error ("multisig cosigner xpub… single-key addresses would not match"), and supplying `--address-type p2wpkh` does NOT bypass the refusal (refuse precedes the explicit-type branch). The plan's Task A2 `address` row reuses `MK1_FIXTURE`/an existing fixture + `--address-type p2wpkh --count 1`. **Why Critical:** the cell asserts `status.success()` + advisory presence; against the in-tree multisig fixture the command exits non-zero with NO stdout/advisory → the cell cannot pass as written, and the engineer following "reuse an existing fixture" lands directly on this trap. **Fix:** the `mk address` cell needs a dedicated **single-sig** fixture (e.g. encode a card at `m/84'/0'/0'` — BIP-84, depth 3, account-depth heuristic infers p2wpkh) generated in-test; the plan must call this out explicitly rather than "reuse an existing repair-test fixture." (mk `derive` has no such guard — `cmd/derive.rs` has no multisig/depth refusal — so the `derive` cell on the multisig fixture is fine; only `address` is affected.)

## Important (must fix before code)

- **[I1] The `md compile` emit-site and the Task B2 commit stage the WRONG file: `crates/md-cli/src/compile.rs` (the lib: ScriptContext/compile_policy_to_template) instead of `crates/md-cli/src/cmd/compile.rs` (the CLI handler with `run()`).** Evidence: the compile CLI handler is `crates/md-cli/src/cmd/compile.rs` — `pub fn run(...)` at `:4`, json early-return `return Ok(0)` at `:21`, text return `Ok(0)` at `:26` (the `:25` the table cites is the `println!("{template}")`, not the return). The file `crates/md-cli/src/compile.rs` is the lib logic with returns at `:81`/`:105` — NOT an output/emit site. Task B2's table row says "`compile.rs` (json `:21`, text `:25`)" and Task B2-Step-6's `git add` line (plan line 289) stages `crates/md-cli/src/compile.rs`. **Why Important:** following the table/staging literally edits/stages the wrong file → the compile emit lands nowhere (or in the lib, which has no `json`/stdout), the cfg-gated cell can't pass, and the commit misses the real handler file. The SPEC §3 correctly says `cmd/compile.rs:25`; the plan dropped the `cmd/` prefix and put the wrong path in the commit. **Fix:** change the Task B2 table file to `crates/md-cli/src/cmd/compile.rs` (json `:21`, text return `:26`), and stage `crates/md-cli/src/cmd/compile.rs` in B2-Step-6 (drop / correct `src/compile.rs`).

- **[I2] The mk `repair` emit-site citation points at the helper `Ok(())` returns (`~:169`/`:230`), not the single `run()` success return (`:86`) — contradicting the SPEC/R1's verified "emit in `run()` after dispatch" guidance.** Evidence: mk `cmd/repair.rs::run()` dispatches `emit_json`/`emit_text` then has its sole success return `Ok(if any_correction { 5 } else { 0 })` at **`:86`** (verified: `grep -n 'Ok(if any_correction' → :86`; context `:78-87`). Lines `:172` and `:231` are the `Ok(())` returns of the `emit_text`/`emit_json` HELPERS (`:160-172`, `:225-232`); `:169` is a `println!` inside `emit_text`. The SPEC §3 cites `:169` as the **stdout-write** location (where the corrected mk1 is printed); the plan copied that stdout citation into the **emit-site** column. The R1 review explicitly resolved this: "mk repair `Ok(5/0)`@:86 … emit must go in `run()` after dispatch, covering exit 0 AND exit 5." **Why Important:** if the engineer inserts the emit at `:169`/`:230` (inside the helpers), it fires once per helper and BEFORE the dispatch returns — it works for exit 0/5 (helpers are only called on the success path) but is the wrong site per spec, duplicates the call across two helpers, and risks an ordering bug (advisory before the stdout write flushes). It also diverges from the single-emit-per-`run()` pattern used for every other mk handler. **Fix:** Task A2's `repair` row must cite the single emit slot in `run()` immediately before `:86` (`Ok(if any_correction {5} else {0})`), NOT the `:169`/`:230` helper region. (The plan's prose "covers exit 0 AND exit 5 at one return and must NOT emit on its `?`-propagated error path" is correct — only the line cite is wrong.)

- **[I3] The `md vectors` inert cell, invoked without `--out`, writes to `./vectors` in the test's cwd → working-dir pollution (and a non-deterministic/dirty-tree hazard).** Evidence: `cmd/vectors.rs:15-25` — `run(out: Option<String>)` defaults `None → PathBuf::from("./vectors")` and `fs::create_dir_all`. The SPEC §3 note "writes files; stdout empty" is about the default behavior. Task B3-Step-2 says the inert cell is "`vectors` (writes files; assert stdout empty AND no advisory line on stderr)" with no `--out`. **Why Important:** running `md vectors` with no `--out` creates/writes a `./vectors` directory under wherever the test process runs (the crate root), leaving an untracked dir that can dirty the tree, break a follow-up `git status`/audit, or collide across parallel test cells. The existing pattern (`mk round_trip.rs:86-90`) always passes `--out <tempfile::tempdir()>`. **Fix:** the `md vectors` inert cell must pass `--out <tempdir>` (add `tempfile` usage — already a md-cli dev-dep, `Cargo.toml:43`) so it writes to a scratch dir; assert no advisory on stderr there.

## Minor (fold opportunistically)

- **[m1] `missing_docs` footgun is effectively moot — both crates set crate-level `#![allow(missing_docs)]`.** `mk-cli/src/main.rs:7` `#![allow(missing_docs)]` and `md-cli/src/main.rs:1` `#![allow(missing_docs)]`. So the SPEC M1 / plan self-review line 383 framing of `missing_docs` as a live CI footgun does not actually apply here (the gate is suppressed crate-wide). Copying ms-cli's `///` docs is still good hygiene and harmless. Downgrade the plan's emphasis; not load-bearing.

- **[m2] The `md compile` emit is only clippy-checked under `--features cli-compiler`, which the plan's B3-Step-3 default-feature clippy run does NOT cover.** md CI runs default-feature `clippy --workspace --all-targets` (`ci.yml:47`) — `cmd/compile.rs` is `#[cfg(feature="cli-compiler")]`-gated (`cmd/mod.rs:3-4`) and not built there. The toolkit's `manual.yml:84` installs md-cli `--features cli-compiler`, so a clippy defect in the compile emit would only surface at toolkit-install time. B2-Step-5 runs the compile *test* under the feature but B3-Step-3 only clippy-checks default features. **Fix:** add `cargo clippy -p md-cli --all-targets --features cli-compiler -- -D warnings` to B3-Step-3. Low risk (the emit is a verbatim 3-line copy of a working pattern), hence Minor.

- **[m3] The Tier-0 smoke-test fixture generation is correct in premise but the corruption locus is underspecified.** VERIFIED: plain `md encode <short-template>` emits a **non-chunked single-string** md1 (`smoke.rs:19` shows `md encode wpkh(@0/<0;1>/*)` → one line `md1yqpqqxqq8xtwhw4xwn4qh`), and `md-cli/tests/cli_repair.rs:30-37,107-112` documents that non-chunked single-string md1 is exactly the form that fails the chunked-only path — so the plan's fixture premise (C1-Step-1) and the FAIL-on-0.34/PASS-on-0.35 TDD gate are sound. md-codec 0.35.0 IS published (registry cache `~/.cargo/registry/{cache,src}/…/md-codec-0.35.0` present). **Fix (Minor):** C1-Step-1 should specify corrupting a **data** symbol (offset past the `md1` HRP + `1` separator), mirroring `cli_repair.rs::corrupt_at` (hrp_len=3), so the corruption stays in BCH-correctable territory and doesn't accidentally hit the HRP/separator (which would route to HrpMismatch/UnparseableInput instead of exercising the codec correction). One symbol is well within capacity.

- **[m4] Task C2-Step-2 "run the `sibling-pin-check.yml` logic" is not directly executable — the logic is an inline `run:` bash block, not a standalone script.** `.github/workflows/sibling-pin-check.yml:42-120` is an inline `run: |` block using bash-isms (`set -eu`, `local` in a function, process substitution `< <(...)`). It must be extracted and invoked with `bash` (the plan says "`bash` the parse-and-compare inline" — acceptable, but note the harness shell here is not bash; the engineer must explicitly `bash <script>`). The 5-site set is confirmed exact (see verified-citations). Minor — guidance is loose but workable.

- **[m5] Toolkit target version is unnamed (`mnemonic-toolkit-vX.Y.Z`).** Current is `0.38.2` (`crates/mnemonic-toolkit/Cargo.toml:3`); PATCH → `0.38.3`. C4-Step-2 leaves it as `X.Y.Z`. Naming it (0.38.3) removes ambiguity. Minor.

---

## Spec-coverage matrix (each spec §/finding → plan task → ✓/gap)

| SPEC §/finding | Plan task | Status |
|---|---|---|
| §3 class map (mk all WatchOnly; md Template ×6 + address WatchOnly) | A1/A2 (mk), B1/B2 (md), header "Class map" | ✓ consistent (handler counts mk 6 / md 7 match §8) |
| §3 inert set (verify/vectors/gui-schema) | A3-Step-2 (mk negatives), B3-Step-2 (md negatives) | ✓ (but I3: md vectors cwd pollution) |
| §4 helper = ms-cli template (not toolkit lib) | A1-Step-3 module / B1-Step-3 | ✓ verbatim copy of `advisory.rs:26-44`; matches live precedent |
| §4 C1 `#[allow(dead_code)]` enum | A1-Step-3 / B1 module | ✓ present; empirically validated below |
| §4 C1 live-caller-same-task | A1 (decode wired in same task), B1 (decode both branches same task) | ✓ verified: module+live caller in one commit prevents dead-code red |
| §4 C2 no Ord / no worst_class_on_stdout / no card_kind_class | A1/B1 module | ✓ enum is non-Ord; neither fn ported |
| §4 I1 emit at EVERY success return (md json early-returns) | B1 (decode both), B2 (per-handler table) | ✓ enumerated — but I1 (compile file path) + I2 (mk repair site) defects |
| §5 byte-parity test (3 literals) | A3-Step-1 / B3-Step-1 | ✓ models `byte_parity_advisory_lines` |
| §5 per-subcommand cells incl. all `--json` modes | A2/B2 cells | ✓ (compile cell cfg-gated) — modulo C1/C2 fixture defects |
| §5 inert negatives + compile cfg-gate + encode --key still Template + mk repair exit-5 | A3/B3 negatives, B2-Step-1 (--key) | ✓ (clap invocations verified valid) |
| §6 transcript re-capture `24-recover-md1` (sole) + idempotency (M3) | C3 (all 6 steps) | ✓ sole-affected confirmed; idempotency re-run specified |
| §7 Tier-0 0.34→0.35 pin + smoke test + FOLLOWUP correction | C1 (8 steps) | ✓ premise sound (m3 corruption-locus detail) |
| §7 caret pin requires explicit "0.35" + stale-lock cargo-before-stage | C1-Step-3/Step-4 | ✓ Cargo.toml:22 is `"0.34.0"` caret; cargo build before staging Cargo.lock |
| §8 phasing mk→md→toolkit, 3 independent units | Phase A→B→C, hard-dep header | ✓ |
| §8 I2 five pin sites (install.sh×2 + manual.yml×2 + quickstart.yml×1) | C2-Step-1 | ✓ all 5 lines verified exact |
| §8 I3 git tag = gate deliverable (crates.io optional) | A3/B3 tag steps, C2 pins | ✓ |
| §9 footguns (dead-code, multi-return, 5-site, stale-lock, missing_docs) | self-review §383 | ✓ enumerated (m1: missing_docs moot) |
| Gates: per-phase opus review + end-of-cycle R0 + persist before commit | A3-Step-5, B3-Step-5, C4-Step-4 | ✓ all specified, persisted to design/agent-reports/ |
| Tag/publish gated on user authorization | A3/B3/C4 commented tag lines + self-review note | ✓ explicit "on ship authorization" |

No spec requirement is wholly unmapped. The gaps are fidelity defects within mapped tasks (C1/C2 fixtures, I1 file path, I2 emit site, I3 vectors dir), not missing tasks.

## Citations & invocations verified (line refs, clap args, fixtures, dev-deps — ✓/✗)

**mk emit/return sites (`mnemonic-key e5620ce`):**
- mk `decode` run() single `Ok(0)` @ `:38` (json/text → helpers `emit_json`/`emit_text`) — ✓ (plan "single trailing Ok" correct).
- mk `encode` run() single `Ok(0)` @ `:97` — ✓ (plan "~:97" correct).
- mk `inspect` run() single `Ok(0)` @ `:40` — ✓.
- mk `repair` run() single `Ok(if…{5}else{0})` @ **`:86`** — ✗ plan cites `:169/:230` (helper `Ok(())` returns) [I2].
- mk `derive` run() single `Ok(0)` @ `:99` (json/text inline, no early return) — ✓.
- mk `address` run() single `Ok(0)` @ `:126` (json/text → helpers) — ✓ site; ✗ fixture incompatible [C2].
- mk `verify` inert (run `:113`, json `:117-133`); `vectors` inert (stdout corpus); `gui-schema` inert — ✓.

**md emit/return sites (`descriptor-mnemonic c599292`):**
- md `decode` json `:24` / text `:30` — ✓ EXACT (matches plan + R1).
- md `encode` json `:69` / text `:95` — ✓ EXACT.
- md `inspect` json `:42` / text `:59` — ✓ EXACT.
- md `bytecode` json `:30` / text `:41` — ✓ EXACT.
- md `repair` single success `Ok(…{5}else{0})` @ `:153` (emit slot `:152`); error `return Ok(2)` @ `:121` (no stdout, inert) — ✓ EXACT.
- md `address` json `:57` / text `:65` — ✓ (plan "json :57, text").
- md `compile` handler in **`cmd/compile.rs`** json `:21` / text return `:26` (`:25` is the println) — ✗ plan cites `compile.rs` + stages `src/compile.rs` (wrong file) [I1].
- md `verify` `:45` inert; `vectors` `:73` writes `./vectors` default [I3]; `gui-schema` `:57` inert — ✓ classes.

**Module registration:** mk `main.rs:9-11` (`mod cmd; mod error; mod process_hardening;`) — ✓; md `main.rs:3-9` mod block — ✓.

**Byte-parity source-of-truth:** toolkit `secret_advisory.rs` enum `:80`, `worst_class_on_stdout :83`, `emit_output_class_advisory :97`, three literal lines `:99-102` — ✓ EXACT; literal `—` (U+2014) == ms-cli `\u{2014}` byte-identical. ms-cli precedent `advisory.rs:26-32` (enum) + `:37-45` (emit fn) — ✓ matches plan module verbatim.

**clap invocations:** `mk verify <mk1…> --xpub` (`verify.rs:18-23`) ✓; `mk address <mk1…> --address-type p2wpkh --count` (`address.rs:22-33`, AddressType variants p2pkh/p2sh-p2wpkh/p2wpkh/p2tr `derive_support.rs:16-20`) ✓ flag-valid (but fixture trap [C2]); `mk derive <mk1…> --index` ✓ no guard; `md verify <phrase> --template` (cmd_verify.rs:25) ✓; `md encode <T> --key @i=XPUB` (cmd_address.rs:50) ✓; `md decode --json` ✓.

**Bin names / dev-deps:** `Command::cargo_bin("mk")`/`("md")` resolve — mk `[[bin]] name="mk"` (Cargo.toml:15-16), md `[[bin]] name="md"` (Cargo.toml:18-19) ✓. `assert_cmd`+`predicates` dev-deps present in BOTH (mk `Cargo.toml:33-34`, md `:40-41`); `tempfile` present in both (mk `:35`, md `:43`) — no Cargo.toml dev-dep add needed ✓.

**Fixtures:** mk — ✗ NO bare `mk1` literal in tests; runtime-generated only [C1]; in-tree fixtures are multisig (m/48') [C2]. md — md1 literal `md1yqpqqxqq8xtwhw4xwn4qh` exists (`smoke.rs:19`, output of `md encode wpkh(@0/<0;1>/*)`) but the idiomatic source is runtime `encode(template)` (`cmd_decode.rs:6-13`); md address xpub via in-test `account_xpub()` (`cmd_address.rs:12`). Plan should name these concretely, not placeholder.

**Pin sites (toolkit `64943f2`):** `install.sh:35` md `descriptor-mnemonic-md-cli-v0.6.1`→v0.6.2 ✓; `install.sh:41` mk `mk-cli-v0.6.0`→v0.6.1 ✓; `install.sh:38` ms `ms-cli-v0.5.0` (untouched, matches `manual.yml:88`) ✓; `manual.yml:77` mk ✓; `manual.yml:84` md `--features cli-compiler` ✓; `quickstart.yml:71` mk ✓. sibling-pin-check scans all `*.yml` and matches pkg→tag against install.sh `component_info` — confirmed `:42-120`.

**Tier-0:** toolkit `Cargo.toml:22` = `md-codec = "0.34.0"` (caret) ✓; `Cargo.lock:643-646` resolves 0.34.0 from crates.io ✓; md-codec 0.35.0 published (registry cache+src present) ✓; FOLLOWUPS.md Tier-0 heading `:396`, false-Resolution prose `:403`, `Status: resolved v0.24.0 cycle` `:404` ✓; NO toolkit transcript invokes `mnemonic repair` (zero transcript drift) ✓.

**Dead-code mechanism (empirical, `/tmp/deadcode_probe2`):** `#[allow(dead_code)]` enum + ONE live (main-reachable) `emit_output_class_advisory` caller → `cargo clippy --all-targets -- -D warnings` **exits 0**. Removing the live caller (only `#[cfg(test)]` call left) → `error: function emit_output_class_advisory is never used` (exit non-zero). CONFIRMS: (a) Task A1's module+decode-in-same-task design correctly prevents dead-code red; (b) the SPEC C1 requirement is real and honored. Both crates lack a crate-level `#![allow(dead_code)]`, so the enum-level `#[allow(dead_code)]` is load-bearing — plan includes it ✓.

## Notes / risks

- **Task ordering within A1/B1 is safe.** A1 commits once (Step 7) after Step 6's clippy-green check; there is no intermediate commit where the module lands without a live caller. A1-Step-2's "verify it fails" runs before the module exists, so the test fails on a missing advisory (runtime), not a compile error — correct TDD, contingent on a compilable `MK1_FIXTURE` (see C1).
- **Tier-0 is genuinely low-risk and correctly scoped** (additive 0.34→0.35 `chunk.rs` pre-pass; chunked path untouched; toolkit consumes via unchanged `repair_via_md_codec`; zero transcript drift). The C1-Step-2 FAIL / C1-Step-5 PASS empirical gate is the right way to prove the latent bug + fix.
- **`24-recover-md1` pair-mode interleave (SPEC M3) is real and correctly handled by C3-Step-4** (re-run verify-examples twice, confirm `.out` stable). The transcript decodes 3 chunked md1 via `$MD_BIN decode` → new `.out` gains the `template` note; sole `$MD_BIN` transcript, `$MK_BIN` never invoked — re-verified.
- **mk `address` already emits a pre-existing depth-≠3 stderr warning** (`address.rs:83-89`); the WatchOnly advisory is additive on stderr. A single-sig depth-3 fixture (the C2 fix) also avoids that warning, keeping the cell's stderr clean for the `contains(WATCH_ONLY_LINE)` assertion (which would pass regardless, but cleaner).
- **Re-grep discipline:** all md `--json` early-return line numbers (`:24/:30`, `:69/:95`, `:42/:59`, `:30/:41`, `:57/:65`, `:21/:26`) and mk single-returns (`:38/:97/:40/:86/:99/:126`) are current at the header SHAs; the plan correctly instructs re-grep at impl time, but the mk repair `:86` and md compile `cmd/compile.rs:26` corrections (I2/I1) must be baked into the plan now so the re-grep targets the right function/file.
- **After the SPEC R1 GREEN, this plan is the second R0 gate.** Fix C1/C2/I1/I2/I3, re-dispatch for R1; the design skeleton (module shape, phasing, lockstep, Tier-0, gate placement) is otherwise complete and correctly honors every SPEC-R0 finding.
