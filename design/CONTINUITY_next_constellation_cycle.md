# CONTINUITY — next constellation cycle (post-Cycle-A resume anchor)

**Read this first on a fresh session.** Written 2026-07-06/07 after Cycle A shipped. The repo's `CLAUDE.md`
(auto-loaded) has the process rules; `MEMORY.md` (auto-loaded) has the standing prefs + shipped-cycle index.

---

## Where things stand
- **Cycle A (descriptor use-site collapse, constellation-eval C1) is DONE.** Shipped `mnemonic-toolkit-v0.76.0`
  (tag @ `3d48cfff`), on `origin/master` (HEAD `6a4a244e`), **all CI gates green**. Full record:
  memory `project_cycleA_use_site_collapse_shipped`, `design/CONTINUITY_cycleA_LIVE.md`,
  `design/agent-reports/cycleA-*`, `design/{SPEC,IMPLEMENTATION_PLAN}_cycleA_descriptor_use_site_collapse.md`.
- The fix: `lex_placeholders` fail-closed-rejects a fixed use-site step (`@0/0/*`) or the `/**` shorthand instead
  of silently collapsing `…/0/*`→`…/*`; verify-bundle concrete false-pass closed. Part-2 bitcoin-core pair-merge
  was SPLIT to a follow-up.

## The remaining queue (recommended priority order)
1. **`bitcoin-core-receive-change-pair-merge`** (Cycle A's split-out; `design/FOLLOWUPS.md`). Restores standard
   Bitcoin Core import, which is INTERIM-BROKEN since v0.76.0 (Core exports split `/0/*`+`/1/*`; they now hard-fail
   with a documented `<0;1>/*`+`--format descriptor` workaround). **Highest user-facing value.** Scope is
   funds-safety-sensitive + CROSS-REPO: a pre-pass in `bitcoin_core.rs::parse` pairing receive(internal:false)/
   change(internal:true) → `<0;1>/*` (order by `internal`, actual step values, never merge across accounts/keys);
   `CoreSourceMetadata.internal: bool → Option<bool>`; `apply_select_descriptor` rewrite; BOTH `--json` wire
   (`import_wallet.rs:1859`) + text-summary (`:2265`) sites; RECOMPUTE the BIP-380 checksum on the merged desc;
   PAIRED `mnemonic-gui` PR (`--json` wire-shape change). Merge-negative-control fixture already exists
   (`core_fixture_file_multipath_receive_change_pair_parses`, distinct keys); merge-INPUT fixture KEPT
   (`core-mainnet-receive-change-pair.json`). Needs its own oracle-guarded, funds-reviewed cycle.
2. **`bip389-double-star-shorthand-support`** (`design/FOLLOWUPS.md`). Accept `/**` (expand to `<0;1>/*`) instead
   of rejecting. Mainstream form; plan-R0 flagged possibly HIGHER user-impact than #1. TOOLKIT-only (smaller).
3. **`concrete-nonranged-xpub-implied-wildcard`** (`design/FOLLOWUPS.md`). A concrete non-ranged xpub (no `/*`)
   silently gains a wildcard on restore; must be handled UPSTREAM at the substitution layer (can't fix at the
   lexer). Narrow funds edge. TOOLKIT-only.
4. **Constellation-eval cycles B–G** (`design/agent-reports/constellation-eval-2026-07-06.md` §3, 6 IMPORTANT +
   remediation program). The eval recommends landing **F** early (branch protection + wire `wc-codec` tests into
   CI — NO repo's suite gates merges today, so the new regressions don't actually gate) and **C** next (BCH repair
   miscorrection incl. the unbounded ms1-seed path — next-highest funds risk).
5. Minor: `41-mnemonic.md:1402-1413` prose nit (pre-existing; claims Core emits `<0;1>/*` on export — contradicts
   `45-foreign-formats.md`). Non-gated. Fold into any manual-touching cycle or a doc-consistency pass.

**My recommendation:** start with #1 (pair-merge) to un-break Core import, OR #2 (`/**`, smaller + high-impact) if
a tighter cycle is preferred. Confirm with the user at cycle start.

## Process to follow (CLAUDE.md — non-negotiable)
R0-gated at every level: cycle-prep recon (re-grep citations vs current origin/master) → brainstorm/SPEC → **opus
architect R0 loop to 0C/0I BEFORE any code** → IMPLEMENTATION_PLAN → R0 loop to 0C/0I → single implementer subagent
in a worktree (TDD) → per-phase opus R0 (run the FULL suite) → mandatory post-impl whole-diff review → release
ritual → direct-FF + tag. Persist every review verbatim to `design/agent-reports/`. Model policy: opus for
R0/review dispatches, cheaper (sonnet/fable) for recon/implementers/mechanical. Ultracode default-ON.

## GOTCHAS the next cycle MUST remember (learned in Cycle A)
- **Release ritual has TWO version sites the standard memory list missed** (now in `project_cycleA_use_site_collapse_shipped`):
  (a) `.examples-build/gen.sh` version pin + `Examples.md` — `examples.yml` re-runs `gen.sh` on any
  crates/Cargo/install.sh change, FATALing on a version mismatch; refresh with
  `EXAMPLES_BIN_DIR="$PWD/target/debug" bash .examples-build/gen.sh > .examples-build/Examples.md` (gen.sh else
  uses the STALE installed `~/.cargo/bin/mnemonic`). (b) The manual `verify-examples` runs LIVE commands — any
  worked-example transcript exercising a CHANGED behavior DRIFTS; regenerate the transcripts with the ACTUAL new
  binary, prose-only is insufficient. **Run `make -C docs/manual lint` with the real new-behavior binary BEFORE
  tagging.**
- Standard release sites (memory `project_toolkit_release_ritual_version_sites`): Cargo.toml + Cargo.lock + BOTH
  READMEs (`<!-- toolkit-version -->`) + `fuzz/Cargo.lock` + `scripts/install.sh` SELF-pin (line 32; NOT the frozen
  md/ms/mk sibling pins — bumping those breaks `sibling-pin-check`) + CHANGELOG (tag-gated) + re-vendor iff a dep
  bumps. Toolkit ships direct-FF + tag (admin bypass of the required `examples` check is expected). NEVER
  `cargo fmt --all` (mlock.rs is fmt-exempt).
- Don't trust agent completion reports — POLL git/CI ground truth (agents sometimes stop mid-CI-wait).

## Resume protocol
`git fetch`; confirm `origin/master` == local; read this doc + `MEMORY.md`. Pick the next cycle (default #1 or ask).
Then run the CLAUDE.md R0-gated pipeline. Nothing is in-flight; no uncommitted work; no open worktrees of ours.
