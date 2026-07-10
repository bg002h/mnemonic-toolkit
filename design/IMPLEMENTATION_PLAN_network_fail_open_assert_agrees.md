# IMPLEMENTATION PLAN — Cycle H: F3 network fail-open (assert_network_agrees on 5 edges)

**SPEC:** `design/SPEC_network_fail_open_assert_agrees.md` (✅ R0-GREEN @ round 4, Fable — `design/agent-reports/cycleH-spec-r0-round-{1,2,3,4}.md`). The SPEC carries the exact per-edge fix, the completeness audit, the test cells, and the manual/release lockstep; this plan is the phase split + guard-rails + release. Subject to its own Fable **plan-R0** to 0C/0I BEFORE implementation.

**Source SHA:** toolkit `713484c3` (master; v0.82.0 line + g4a fix). **Target:** MINOR `v0.83.0`, toolkit-only. md/mk/ms NO-BUMP; no clap flag → no `schema_mirror`; no re-vendor; no sibling-pin change; no crates.io publish. **Worktree:** one `mnemonic-toolkit` worktree (branch `feature/cycleH-network-fail-open`). Single implementer, TDD.

## Phase P0 (single phase — 5 small independent guards, one file each) — all edges + tests
The five guards are independent (5 distinct files) and each mirrors the shipped `addresses.rs` / WIF→xpub / `--from-import-json` prior-art, so one implementer does all five + the new test file in one TDD pass (tests RED-first). NO cross-file coupling; NO shared type change; reuse `ToolkitError::NetworkMismatch` (exit 2) — no new error variant, no `match self` ordering churn.

**Edges (SPEC §1 — exact snippets there):**
- **E1** `src/cmd/convert.rs:1524-1526` — xpub→address `match { Some(n) => assert_network_agrees(xpub.network, n.network_kind(), "convert: xpub→address")?; n, None => infer }`.
- **E2** `src/cmd/xpub_search/address_of_xpub.rs:215-217` — same shape, ctx `"xpub-search address-of-xpub"`.
- **E3** `src/cmd/silent_payment.rs:125-135` — xprv/tprv branch: `assert_network_agrees(xpriv.network, network.network_kind(), "silent-payment: xprv/tprv master")?` before returning the master.
- **E4** `src/cmd/export_wallet.rs:~672` — after `resolved_slots_ref` is bound, before `EmitInputs` (`:674`): loop asserting each `slot.xpub.network` AND `slot.master_xpub` (Minor-C) vs `args.network.network_kind()`, ctx `"export: --template/--slot"`. `--descriptor` = empty slots = inert no-op.
- **E5** `src/wallet_export/bsms.rs` `BsmsEmitter::emit` `FourLine` arm — after `parsed`, before `derive_first_address`: walk `parsed.for_each_key`, `k.xkey_network()` (Some XPub+MultiXPub / None Single), capture mismatch, `assert_network_agrees(decoded, inputs.network.network_kind(), "export: bsms first-address")?`. Covers template arm (redundant no-op) + `--descriptor` arm (the mint). Do NOT guard the whole `--descriptor` arm.

**TDD — new `tests/cli_network_fail_open.rs` (RED-first, public never-fund vectors):** all §4 cells — E1a-d, E2a-b (+ mk1-target cell + signet-accept cell), E3a-c, E4a-d, E5a-e. Every REJECT cell asserts `NetworkMismatch` exit **2** + stderr names decoded/expected; every ACCEPT/inference/seed cell asserts exit 0 with the right address (the over-rejection non-regression pins: E1c inference, E3c seed, E4d descriptor-passthrough, E5c 2-line, E5d hex-single, E5e-testnet).

**Completeness audit (SPEC §2 — MUST do, then declare done):** the intake-arm audit (not a single grep); the 8 benign hits are pre-adjudicated (leave); if a 6th mint edge surfaces, FLAG for a scope decision — do NOT silently widen. The export-emitter audit is confirmed complete (R0 round 3) — E1-E5 is the set.

**Per-phase Fable R0** (FULL `cargo test -p mnemonic-toolkit`) → 0C/0I. Persist `cycleH-phase-P0-r0-round-N.md`.

## Manual lockstep (SPEC §5 — prose only, no flag change)
Mirror the `addresses.rs` refusal wording into `docs/manual/src/40-cli-reference/41-mnemonic.md` for `convert` (xpub→address), `xpub-search address-of-xpub`, `silent-payment`, and `export-wallet` (the `--template/--slot` + `bsms` first-address refusal): an explicit/effective `--network` disagreeing with a key's version bytes now refuses fail-closed. Re-grep anchors at impl. R0 confirmed no golden `.out` transcript exercises a now-refused path (nothing to regen for a diff). Run `make -C docs/manual lint` if flag tables are touched (they are NOT — no flag change).

## Post-implementation (mandatory) — Fable whole-diff
Fresh Fable over the whole diff: all 5 guards correct + precondition-respecting (no over-rejection of inference/seed/verbatim-descriptor paths), the completeness audit honored, `NetworkMismatch` reused, no regression, SemVer. Persist `cycleH-postimpl-whole-diff-review.md`.

## Release ritual (only after whole-diff GREEN) — toolkit v0.83.0
Standard toolkit (no sibling/publish): version sites (Cargo.toml `:3` + workspace `Cargo.lock:731` + `fuzz/Cargo.lock:579` + both READMEs `<!-- toolkit-version -->` + `scripts/install.sh:32` self-pin `v0.82.0`→`v0.83.0`) + **`.examples-build/gen.sh` — bump ALL 6 `0.82.0` occurrences** (`:3/:44/:109/:126/:711/:724`; SPEC §5 Minor-2 / Cycle-A gotcha: `examples.yml` re-runs gen.sh on `crates/` changes → self-pin FATALs on drift) + **regen `.examples-build/Examples.md` LAST** (M2: `Examples.md:102/115/116` embed install.sh's self-pin via a live dry-run transcript, so regen must run AFTER the install.sh + gen.sh bumps in the same tree; pure version-string diff expected — verify no transcript content moved) + CHANGELOG `[0.83.0]` + **file the FOLLOWUPs** in `design/FOLLOWUPS.md`: the 2 out-of-scope observations (`xpub-search-network-contradiction-not-diagnosed`, `restore-cosigner-bare-xpub-network-contradiction-not-diagnosed`) + **the GUI-companion mirror** (M3: a toolkit-side entry for the deferred `mnemonic-gui` `--network` dropdown-default fix, with a cross-citing `Companion:` line; mirror a companion entry into `mnemonic-gui/design/FOLLOWUPS.md` too per the CLAUDE.md cross-repo convention) + note constellation-eval **F3 RESOLVED** + NO re-vendor + NO sibling-pin change. Build; FULL suite; FF master → tag `mnemonic-toolkit-v0.83.0` → push (admin-bypass `examples`) → **verify the FULL fired set (M1):** required — `examples`, `changelog-check`, `install-pin-check`, `sibling-pin-check`, `rust`/miri; also fired — `man-pages` (→ `reproducible-musl-build` release-asset leg, the v0.74.0-recut culprit — watch it), `vendor-freshness`, `fuzz-smoke`, `manual` (the 41-mnemonic.md prose edit), `gui-pin-drift-check` (warn-only). **USE `git commit -F <file>` (backtick gotcha).**

## Guard-rails (cycle)
- **G-A** Precondition: guard only the asserted-network arm; inference/None + seed + verbatim-descriptor paths stay unguarded (E1c/E3c/E4d/E5c/E5d are the non-regression pins).
- **G-B** Reuse `NetworkMismatch` (exit 2); no new error variant / no `match self` reorder.
- **G-C** Compare against the KEY's embedded network (`xpub.network`/`xpriv.network`/`xkey_network()`), not a re-inference.
- **G-D** E5 uses `xkey_network()` (covers XPub+MultiXPub); do NOT hand-match `XPub` only.
- **G-E** codecs/GUI untouched — NO-BUMP; no clap surface; no schema_mirror; GUI dropdown-default is a deferred companion (file a `mnemonic-gui` companion FOLLOWUP note at ship).
