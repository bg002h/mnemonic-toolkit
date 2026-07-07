# CONTINUITY (LIVE) — Cycle A: descriptor use-site collapse fix

**RESUME ANCHOR — read this FIRST on any resume.** Updated at every gate. The immutable
original handoff is `design/CONTINUITY_cycleA_descriptor_use_site_collapse.md`; this doc is the
live state that supersedes it.

**Working model:** structure every phase to survive a usage-limit interruption — persist all
outputs to disk (design/ + design/agent-reports/), update this doc + the task list at each gate,
never hold critical state only in conversation.

---

## STATUS: ✅ WHOLE-DIFF REVIEW GREEN (post-fold). NEXT = RELEASE RITUAL (MINOR v0.75.0→v0.76.0). Master @ `b59b7a47`+fold.
Whole-diff review (`cycleA-postimpl-whole-diff.md`): 0C + funds-fix CONFIRMED correct/coherent/release-ready
(residue floor fails closed on ALL 20+ callers; verify-bundle false-pass genuinely closed; migration STRENGTHENS
coverage; fixtures+docs accurate; Sparrow unaffected verified in code). 1I = I-1 clippy `doc_lazy_continuation`
(`cli_import_wallet_bitcoin_core.rs:920` `+ `-bullet) — FOLDED (`then import via `) + M-2 nit tightened. CONFIRMED
`cargo clippy --all-targets -D warnings`=exit0, suite 3583/0 + wc-codec 100/0. **Whole-diff GREEN.**
**RELEASE RITUAL (Phase 4b — NEXT):** current v0.75.0 → **MINOR v0.76.0** (breaking: previously-accepted `/0/*`|`/**`
imports now hard-fail). Sites: `crates/mnemonic-toolkit/Cargo.toml` version; Cargo.lock; BOTH READMEs (root +
crate); `fuzz/Cargo.lock`; `install.sh` SELF-pin (v0.75.0→v0.76.0; NOT the frozen md-cli sibling pin — breaks
`sibling-pin-check`); CHANGELOG (funds fix C1 + Core interim + `/**` hard-fail + D1 residual disclosure); re-vendor
iff dep bump (NONE). FOLLOWUPS: C1/this cycle RESOLVED; file `bitcoin-core-receive-change-pair-merge`,
`concrete-nonranged-xpub-implied-wildcard`, `bip389-double-star-shorthand-support`. md/mk/ms NO-BUMP; do NOT touch
md-codec. Then direct-FF push + tag `mnemonic-toolkit-v0.76.0`; verify sibling-pin-check + changelog-check + repro
binary post-tag.
### (superseded) PHASES 1+2+3 INTEGRATED. NEXT = Phase 4a WHOLE-DIFF REVIEW.
Phase 3 (docs) integrated `b59b7a47`: manual `41-mnemonic.md` (authoritative non-representable-use-site anchor +
verify-bundle exit2/exit4 split + refusal rows + --import-json note) + `45-foreign-formats.md` (Core interim
hard-fail + hand-combine-to-`<0;1>` workaround [receive=0/change=1] + Specter/BSMS notes). Docs accuracy VERIFIED
inline by orchestrator (workaround funds-safe; exit codes match impl). Examples sweep = NONE. GUI byte-unchanged
(gui-schema md5 identical + schema_mirror 21/21). Manual lint PASS. No separate Phase-3 R0 (docs low-risk + inline-
verified + covered by the Phase-4 whole-diff review). Full Cycle-A code+docs diff = 17 files +1378/-181.
**Phase 4 (NEXT):** (4a) MANDATORY independent adversarial opus WHOLE-DIFF review over `8c8b9183..HEAD -- crates/ docs/`
→ persist `cycleA-postimpl-whole-diff.md`; folds RE-ENTER the loop. THEN (4b) release ritual: MINOR version bump
(confirm current — likely v0.75.0→v0.76.0) + BOTH READMEs + fuzz/Cargo.lock + install.sh SELF-pin (NOT md-cli
sibling) + CHANGELOG (funds fix + Core interim + `/**` + D1 residual) + FOLLOWUPS (C1 RESOLVED; file
`bitcoin-core-receive-change-pair-merge`, `concrete-nonranged-xpub-implied-wildcard`, `bip389-double-star-shorthand-support`)
→ (4d) direct-FF + tag. md/mk/ms NO-BUMP. Master 8 ahead of origin, push at ship.
### (superseded) PHASES 1+2 DONE. NEXT = Phase 3.
Phase-2 per-phase R0 = GREEN (0C/0I); `cycleA-phase-2-r0-round-1.md`. Test 4 = genuine DIRECT first-recv address
proof (`restore --md1` derives+emits `bc1qcr8te4...`). wc-codec reconciled = TRUE 100/0 (impl's 40 was partial).
Phase 2 cherry-picked CLEAN → master `0de6e1f6`. Suite 3583+100 green. Worktree pruned.
**Phase 3 (NEXT — docs, plan §3 minus CHANGELOG which moves to P4 w/ the version bump):** (3a) manual
`docs/manual/src/40-cli-reference/` — document the import-wallet/`bundle --descriptor`/verify-bundle fixed-step
reject + error text; the INTERIM bitcoin-core limitation (Core split imports hard-fail until pair-merge follow-up;
workaround `<0;1>/*`+`--format descriptor`); the `/**` shorthand hard-fail (same workaround). Check CLI-ref `@N`
examples for a fixed-step form (none expected). **STANDING RULE: any CLI-output shown in docs must be
binary-generated/identical (verify-examples-style), NOT hand-written.** `make -C docs/manual lint` (no flag change).
(3c) examples sweep `examples.yml`/`docs/Examples.pdf` (none expected). (3d) GUI NONE — confirm `mnemonic gui-schema`
byte-unchanged (no flag/enum). Then per-phase R0 (docs). **CHANGELOG → Phase 4** (coupled w/ MINOR version bump).
### (superseded) Phase 2 impl GREEN → per-phase R0.
Phase-2 commit `ea0d3019` (branch `worktree-agent-a264febc9c385a6f2`, ff-merged master first then +1) = 5 born-green
funds-proof tests in `tests/cli_cycleA_phase2_funds_proof.rs`, no src change: verify-bundle concrete `/0/*`→exit2
`DescriptorParse` (before card compare) + `@N`-template→exit4 `DescriptorReparseFailed`; BIP-84 POSITIVE (`restore
--md1` DIRECT first-recv = `bc1qcr8te4...`) + independent wrong-value confirm (`bc1q8vph849...`) + NEGATIVE (`/0/*`
rejects at encode). Suite `mnemonic-toolkit` 3583 pass/0 fail. **wc-codec count discrepancy: 40 here vs 100 in
Phase-1 R0 — R0 to RECONCILE (Cycle A doesn't touch wc-codec).** Trailer flag = FALSE ALARM (subagent on sonnet saw
own prompt; trailer correctly = main session Opus 4.8 per env git instr). Per-phase R0 dispatched (opus). Integrate
ea0d3019 to master after GREEN. Then P3 ripples, P4 whole-diff + MINOR bump + ship.
### (superseded) PHASE 1 DONE + INTEGRATED. NEXT = Phase 2.
Phase-1 per-phase R0: round 1 = 0C/1I → fold I-1/M-1 → round 2 = GREEN (0C/0I). Reviews: `cycleA-phase-1-r0-round-1.md`,
`cycleA-phase-1-r0-round-2.md`. Suite on branch 3578 pass/0 fail; cherry-picked CLEAN onto master (code orthogonal
to design commits). Impl worktree pruned. Master 5 ahead of origin (3 design + 2 impl), unpushed — push at ship.
**Phase 2 (NEXT):** born-green funds regressions (per plan §Phase 2 / m3): 2a verify-bundle false-pass closure —
CONCRETE `/0/*` descriptor verified vs any card → exit 2 / `DescriptorParse` PRIMARY (+ optional `@N`-template →
exit 4 / `DescriptorReparseFailed`); 2b BIP-84 oracle — POSITIVE (correct `<0;1>/*` card restores
`bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu`) + NEGATIVE (`/0/*` rejects at encode, never the collapsed
`bc1q8vph849...`). Then per-phase R0. Then P3 ripples (manual + CHANGELOG incl. `/**` + Core-interim; GUI none),
P4 whole-diff review + MINOR bump + FOLLOWUPS + ship.
### (superseded) Phase-1 R0 fold done → R0 round 2 (convergence).
Fold commit `27f27bed` (parent `72e82d1c`) added 5 born-green reject tests: I-1 `--format descriptor` `/0/*`+`/**`
(`descriptor_fixed_use_site_step_rejected...`, `descriptor_double_star_shorthand_rejected...`), M-1 specter/old-json-
replay/BSMS single-branch. Suite 3578 pass/0 fail (+5); no src/fixture change; 0 deletions. R0 round 2 dispatched
(opus, convergence: non-vacuous asserts + valid checksums + suite green) → persist `cycleA-phase-1-r0-round-2.md`.
### (superseded) Phase-1 R0 round 1 = 0C/1I → folding I-1(+M-1).
Phase-1 R0 round 1 (`cycleA-phase-1-r0-round-1.md`): NO-WEAKENING AUDIT CLEAN (Group A/`:898`/Group B/11-extra all
correct; suite 3573+100 re-verified; fixtures valid; `:898` untouched). Deviation-1 (generic msg) ACCEPTABLE
(full workaround → Phase 3 docs). 11 extra cells assert-reject CORRECT (false-green covering the bug; `<0;1>` swap
would be WRONG — Core never exports combined). **1I blocker I-1:** missing end-to-end `import-wallet --format
descriptor` reject tests for `/0/*` AND `/**` (highest-impact surface, zero CLI proof). **M-1:** add specter/old-json/
BSMS single-branch `/0/*` reject cells too (before ship). FOLD = resume implementer a1c2bfd30353a5ba9 to add these
born-green tests to branch `worktree-agent-a1c2bfd30353a5ba9` → full suite GREEN → commit → re-dispatch per-phase R0.
M-2 (multisig canonicalize `--json`) DEFERRED to pair-merge follow-up; M-3 (msg) → Phase 3 docs.
### (superseded) Phase-1 impl GREEN → per-phase R0.
Phase-1 impl commit `72e82d1c` on branch `worktree-agent-a1c2bfd30353a5ba9` (worktree
`.claude/worktrees/agent-a1c2bfd30353a5ba9`), based on `8c8b9183` (pure code diff — integrate to master by
cherry-pick after R0 GREEN). Suite GREEN: `mnemonic-toolkit` 3573 pass/0 fail/16 ign, `wc-codec` 100/0. Invariants
verified: `core-mainnet-receive-change-pair.json` UNTOUCHED, residue check at parse_descriptor.rs:202-207.
**Impl flagged 2 deviations the R0 must rule:** (1) only the GENERIC surface-tagged reject message added, NOT
SPEC §6's hand-written bitcoin-core-specific verbiage; (2) **11 EXTRA reject-now cells the static grep MISSED**
(runtime-built blobs) — 6 `cli_import_wallet_roundtrip.rs::core_bundle_roundtrip_*` + 5
`cli_wallet_cross_format_convergence.rs` (c1-c4, h_hop) — converted to assert-reject / bitcoin-core-excluded;
R0 must audit no coverage silently dropped. Per-phase R0 dispatched (opus) → persist `cycleA-phase-1-r0-round-N.md`.
### (superseded) ▶ RESUMED: Phase-1 implementer re-dispatched.
## ⏸ (prior) PAUSED at: both R0 gates GREEN, implementation NOT started. Clean resume point.
**Main tree:** master @ `9767b3aa` (all design artifacts committed; `git status` clean of tracked changes).
**RESUME:** re-dispatch ONE Phase-1 implementer subagent (sonnet, `isolation: worktree`) with the exact prompt used
this session — execute Phase 1 (ATOMIC) of `design/IMPLEMENTATION_PLAN_...md`: write all failing tests → residue
check in `lex_placeholders` (re-grep: after the multipath validator ~:178, before `out.push` ~:183) → migrate ALL
22 incumbent cells (Group A + `:898` assert-reject / Group B `<0;1>` swap / OOB fixture-swap; sweep is the checklist)
→ FULL `cargo test -p mnemonic-toolkit` + `wc-codec` GREEN → opus per-phase R0 → integrate to master. Then P2 funds
regressions, P3 ripples, P4 whole-diff review + MINOR-bump release. The first (stopped) implementer left a
disposable partial worktree `.claude/worktrees/agent-aa7ece60b99e468d9` (branch `worktree-agent-aa7ece60b99e468d9`,
uncommitted, ~started writing tests) — ignore/prune it; re-dispatch fresh.
**Guardrails for the implementer:** TDD; keep `lex_bare_at_zero` unchanged (D1 deferred); NO `cargo fmt --all`
(mlock.rs fmt-exempt — `cargo fmt -p` only); no version/README/CHANGELOG/manual/install.sh edits (later phases);
stage explicitly; NEVER swap a Group-A or `:898` cell to `<0;1>` (no-weakening rule).
### (superseded) ✅✅ BOTH R0 GATES GREEN (SPEC + PLAN). Ready for Phase-1 IMPLEMENTATION.
PLAN R0 round 2 = GREEN (0C/0I); `cycleA-plan-r0-round-2.md`. m1 (SPEC §8 verify-variant) + m2 (plan citations
`:853`/`:391`) tidied. NEXT = dispatch ONE implementer subagent in a git WORKTREE to execute the GREEN plan,
Phase 1 (ATOMIC): write all failing tests → residue check in `lex_placeholders` (after :178, before :183) →
migrate ALL 22 incumbent cells (Group A + `:898` assert-reject / Group B `<0;1>` swap / OOB fixture-swap) → FULL
`cargo test -p mnemonic-toolkit` + `wc-codec` GREEN → per-phase opus R0. Then P2 funds regressions, P3 ripples,
P4 whole-diff review + MINOR-bump release. **Implementer must TDD, stage explicitly, NOT touch mlock.rs/fmt, and
persist per-phase R0 to `design/agent-reports/cycleA-phase-N-r0-round-M.md`.**
### (superseded) PLAN R0 round 1 = 0C/4I → folded (rev-2) → R0 round 2.
PLAN rev-2 folds: I-A merge Phase1+2 ATOMIC (residue floor + full 22-cell migration → GREEN together, no red
boundary); I-B verify-path per-path (concrete→`DescriptorParse`/exit2 PRIMARY, template→`DescriptorReparseFailed`/exit4);
I-C `:898` assert-reject + KEEP `core-mainnet-receive-change-pair.json` (legacy-split regression + pair-merge input);
I-D `/**` mainstream shorthand disclosure (CHANGELOG+manual) + CLI reject test + follow-up; M-a inline-literal swaps,
M-b SPEC §8 export line superseded (STAYS exit 1), M-c sparrow discharge+positive-control, M-d MINOR bump, M-e a4/a5
both legs. SPEC edited (D2 per-path verify variant, §8 export line, §10 MINOR); sweep `:898` re-bucketed.
Plan R0 round 1 persisted: `cycleA-plan-r0-round-1.md`. NOW = R0 round 2 → persist `cycleA-plan-r0-round-2.md`.
### (superseded) PLAN-DOC written → PLAN R0 gate round 1.
PLAN = `design/IMPLEMENTATION_PLAN_cycleA_descriptor_use_site_collapse.md` (5 phases: P1 residue-reject floor,
P2 reject-with-remediation + test migration [Group A assert-reject / Group B fixture-swap-to-`<0;1>`], P3 funds
regressions [verify-bundle false-pass + BIP-84 oracle], P4 ripples, P5 whole-diff review + release). Sweep persisted:
`design/agent-reports/cycleA-migration-sweep.md` (22 REJECTS-NOW / 19 STAYS-PASSING / 0 ambiguous). M-9(ii sparrow),
M-9(iii `/**`) resolved (both in plan). PLAN R0 must rule the 4 open items (Group A/B no-weakening faithfulness;
fixture-swap correctness; `/**` reject-not-expand; Phase1→2 RED-window / atomicity). Persist to
`cycleA-plan-r0-round-N.md`; fold → re-dispatch until 0C/0I; THEN implement.
### (superseded) ✅ SPEC R0 GREEN (0C/0I) round 2.
SPEC = `design/SPEC_cycleA_descriptor_use_site_collapse.md` (rev-2, Part 1 floor + Part 3 reject-with-remediation;
Part 2 pair-merge SPLIT to follow-up). R0 reviews persisted: `cycleA-spec-r0-round-1.md` (0C/2I),
`cycleA-spec-r0-round-2.md` (GREEN). M-8 folded (cosmetic §-label). **M-9 carry-forwards the PLAN R0 must verify:**
(i) grep-sweep proof no surviving Cycle-A test implies a merge round-trip; (ii) sparrow descriptor-passthrough
branch can never forward a fixed use-site step; (iii) BSMS `/**` residue handling + `wallet_export/bsms.rs:159-161`
self-round-trip reject. **Design artifacts committed at SPEC-GREEN checkpoint.**
### (superseded) I-2 DECIDED = SPLIT Part 2 (user 2026-07-06). Revising SPEC → R0 round 2.
**Cycle A scope NOW = Part 1 (residue-reject floor) + Part 3 (uniform reject-with-remediation across ALL import
surfaces incl. bitcoin-core).** Part 2 (bitcoin-core pair-merge) SPLIT → own follow-up cycle
`bitcoin-core-receive-change-pair-merge` (carries the full I-2: internal bool→Option, select-descriptor rewrite,
both wire sites `import_wallet.rs:1859`+`:2265`, merged-desc checksum recompute, GUI paired-PR, + P10B.3
`core_fixture_file_multipath_receive_change_pair_parses` different-keys merge-NEGATIVE-control). Interim: standard
bitcoin-core receive+change `/0/*` imports HARD-FAIL with a pointed message (workaround: combine to `<0;1>/*` +
`--format descriptor`) until that follow-up ships. This split REMOVES all GUI/wire-shape/checksum work from Cycle A.
Fold I-1 + all MINORs; re-dispatch R0 round 2.
### (superseded) SPEC R0 round 1 = NOT GREEN (0C / 2I):
SPEC = `design/SPEC_cycleA_descriptor_use_site_collapse.md` (P1 residue-reject floor [R0-VERIFIED correct/fail-closed],
P2 bitcoin-core pair-merge [I-2: ripple bigger than estimated — architect recommends SPLIT], P3 wider-surface rulings).
R0 round 1 persisted: `design/agent-reports/cycleA-spec-r0-round-1.md`.
- **I-1 (fold regardless):** §9 migration set incomplete + no-weakening rule. Mark §9 non-exhaustive; mandate
  `grep -rn '/0/\*\|/1/\*' crates/mnemonic-toolkit/{src,tests}` + classify-every-hit in the plan; FORBID silent
  `/0/*`→`/<0;1>/*` assertion-swaps; name `bitcoin_core.rs:898` `bundles=2→1` flip; pin `:915`
  (`core_fixture_file_multipath_receive_change_pair_parses`, DIFFERENT keys FP_A-bip84 + FP_B-bip49) as a
  REQUIRED merge negative-control (different keys ⇒ NOT merged). Add `cli_descriptor_concrete.rs:174`,
  `cli_import_wallet_sniff.rs:79`, `coldcard.rs:728`, `cli_import_wallet_bsms.rs`.
- **I-2 (needs user call):** Part 2's `--select-descriptor`/`--json` ripple is bigger than the first architect
  estimated — needs `CoreSourceMetadata.internal: bool → Option<bool>`, rewrite both active-* arms, update BOTH
  wire sites (`import_wallet.rs:1859` json + `:2265` text), merged-descriptor string-assembly + BIP-380 checksum
  RECOMPUTE (`verify_checksum` runs on merged `desc` — a synthesized `<0;1>/*` has no valid Core checksum), AND a
  paired mnemonic-gui PR. Architect RECOMMENDS SPLIT (ship P1+P3 now; P2 → own oracle-guarded cycle). But SPLIT
  reintroduces the Core-import hard-fail the user chose B-full to avoid (loud/pointed, not silent wrong card).
- **MINORs to fold:** M-1 (file D1 FOLLOWUP w/ funds framing + CHANGELOG disclosure), M-2 (restate
  sparrow/coldcard/electrum/coldcard_multisig sweep; confirm sparrow passthrough), M-3 (verify path reject is
  `DescriptorReparseFailed{detail}` not `DescriptorParse` — trap-#9 test asserts that shape), M-4 (reword `#`-strip
  claim), M-5 (citation nit :77-110), M-6 (add `@0/48h/…` bare-unbracketed-origin negative test), M-7 (verify-bundle
  false-pass test can't build a `/0/*` bundle post-fix — verify a `/0/*` descriptor vs any card & assert reparse
  rejects, OR use a pre-generated wrong-card fixture).

## RESUME PROTOCOL
On interruption: (1) read this doc, (2) `git status` + check untracked `design/` + `cycle-prep-recon-*.md`,
(3) read latest `design/agent-reports/cycleA-*`, (4) check the task list (TaskList). Latest persisted
design artifacts are ground truth. Continue from "NEXT STEPS" below.

---

## LOCKED DECISIONS
- **Scope = Option B-full** (user 2026-07-06, accepting Fable architect rec):
  1. **Residue-reject floor** — ships FIRST, the CRITICAL funds fix. Non-negotiable: must NOT wait on
     merge design; if merge/wider-surface R0 stalls, ship the reject floor alone.
  2. **bitcoin-core receive/change pair-merge** → `<0;1>/*` (so standard Core imports keep working correctly).
  3. **Explicit rulings for the wider surfaces**: Specter, `--format descriptor`, old `--json` replay
     (`bundle --import-json`) — reject-with-pointed-remediation by default; NO silent assume-paired.
- **D1 = DEFER to a follow-up FOLLOWUP (my sweep REFUTED the architect's "reject bare `@N`" ruling).**
  The architect said reject bare `@N` + flip `lex_bare_at_zero`. **The bare-`@N` sweep (done) proves that is
  WRONG:** bare `@N` is the CANONICAL, pervasive, load-bearing MULTISIG keyless-template form — documented
  (`docs/manual/src/30-workflows/32-multisig-2of3.md:82` "emitted descriptor will be `wsh(sortedmulti(2,@0,@1,@2))`";
  `42-multisig-watch-only.md:86`), stored in `--json` envelopes (`json_envelope.rs:543`), named
  `CANONICAL_DESC` (`cli_ms1_slot.rs:64`), and exercised by PASSING `bundle_two_cosigner(...)` tests that flow
  through `lex_placeholders` (`cli_unrestorable_shape_advisory.rs:111,323-325,371`). A blanket lex-level reject
  breaks the shipped `bundle --md1-form=template` (v0.60.0) feature. The GENUINE sub-concern (a CONCRETE
  non-ranged xpub with no `/*`, e.g. `wpkh([fp/84'/0'/0']xpub)`, silently gaining `/*`) is a DIFFERENT mechanism
  than the dropped-fixed-step collapse, and is INDISTINGUISHABLE from a keyless template at the lexer (the
  wildcard-presence signal is lost upstream at `concrete_keys_to_placeholders`). It cannot be correctly solved
  in `lex_placeholders`; it must be handled UPSTREAM at substitution where wildcard-presence is still known.
  **DECISION: keep `lex_bare_at_zero` as-is (do NOT flip); file a FOLLOWUP `concrete-nonranged-xpub-implied-wildcard`
  for a separate upstream cycle. D1 is orthogonal to the residue-reject floor (bare `@0)` passes the terminator
  check) and to the Core pair-merge — deferring it does not weaken either.** The SPEC R0 gate must independently
  re-examine this override.
- **D2 = reuse `ToolkitError::DescriptorParse(String)`** (exit 2). No new variant (H13 hardened-multipath
  reject already uses it; zero schema ripple).
- **Model policy**: opus for formal R0 gates (spec, plan, per-phase, post-impl whole-diff); cheaper
  (sonnet/fable) for recon/mechanical. (User used fable for the scope-check specifically.)

## VERIFIED KEY FACTS (source @ origin/master 8c8b9183)
- `parse_descriptor.rs`: `lex_placeholders`:60, regex:97-98, `captures_iter`:103, `make_use_site_path`:290-303.
  **NO residue check present** (the bug). Multipath-body validator already mirrored at :146-178.
- md-cli twin residue reject to PORT: `descriptor-mnemonic/crates/md-cli/src/parse/template.rs:128-137`
  (`let match_end = caps.get(0)...end(); if next not in {')',',','}',whitespace,EOS} → reject`). Place it
  AFTER the multipath validator (md-cli ordering :121-127).
- **Adapt, don't copy**: toolkit regex captures origin path ONLY inside brackets (no bare-origin group 2
  like md-cli), so a bare post-`@N` `/0` is *use-site* residue (correct to reject), not an origin path.
- md-codec `UseSitePath` (use_site_path.rs:49-53) = `{multipath: Option<Vec<Alternative>>, wildcard_hardened}`;
  `MIN_ALT_COUNT=2` (:43) → single fixed step un-representable. `wildcard_for` (to_miniscript.rs:133-140)
  ALWAYS emits a wildcard.
- BIP-84 oracle (authoritative-verified): correct first receive `bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu`
  @ m/84'/0'/0'/0/0 for abandon×11 about; collapsed card (wrongly) restores `bc1q8vph849lf3e9rrj85hsxrzlv949rtahe794k6p`.
- R1 pipeline: `concrete_keys_to_placeholders` (pipeline.rs:330-400) preserves `/0/*` verbatim → `@0[fp]/0/*`
  at lexer. `bitcoin_core.rs::parse` (:142-213) processes entries INDEPENDENTLY; **NO recombination anywhere**;
  `apply_select_descriptor` (mod.rs:394-444) only FILTERS by `internal`.
- verify-bundle false-pass: descriptor-mode re-parses via same path (verify_bundle.rs:1307,1352-1357) → both
  sides collapse identically → PASS on a wrong card. (Core receive+change entries collapse to BYTE-IDENTICAL cards.)

## 10 FUNDS-SAFETY TRAPS (architect — must guard in the spec)
1. Merge-pairing predicate: pair only entries whose (fp, origin path, xpub) tuples are identical for EVERY key
   and differ in exactly ONE unbracketed step at the same position; never merge across accounts or across keys
   within an entry (`wsh(multi(2,[a]x/0/*,[b]y/1/*))` = per-key divergence → reject, don't merge).
2. Order alternatives by the `internal` flag (external entry = first alt = md1 chain 0 = receive), NOT numeric sort.
   Rule the >2-candidate case.
3. Never hardcode change==1; use the actual step values (`/4/*`+`/5/*` → `<4;5>`).
4. Residue terminator set = `)` `,` `}` whitespace / EOS (match md-cli). Verify `#` checksum trailers are stripped
   before EVERY `lex_placeholders` call site (they are, via `verify_checksum`) — a surviving `#` would false-reject.
5. BSMS `/**` form: `xpub/**` → `@0[fp]/**`; wild group eats `/*` leaving residue `*`. Verify bsms.rs handling;
   reject must be a pointed message, not a mystery. Add a test.
6. Ordering: residue check AFTER the `<…>` body validator (preserve H13 byte-exact hardened-multipath error).
7. `--select-descriptor` semantics shift under B: merge in `parse()`, selection after (mod.rs:366-369) — `ByIndex(N)`
   indices shift; `active-receive`/`active-change` ill-defined on a merged entry; `bundles[i].internal` in the
   `--json` envelope (import_wallet.rs:1859,2265) changes meaning. Define + check GUI schema-mirror/wire-shape ripple.
8. Old `--json` envelopes: `original_descriptor` stores raw `/0/*` (bitcoin_core.rs:347); `bundle --import-json`
   replay re-parses through same adapter → hard-fails under BOTH options. Explicit migration ruling + error text.
9. verify-bundle UX: verifying a merged-`<0;1>` bundle against a Core receive descriptor rejects at parse —
   error must point at the `<0;1>` form, not read as card corruption.
10. Test BOTH residue directions: pre-multipath (`@0/0/<0;1>/*`) and post-multipath (`@0/<0;1>/2/*`).

## WIDER SURFACES to rule on (architect correction — reject blast radius > bitcoin-core)
- **Specter** (specter.rs:190-232): single `descriptor` field, fed verbatim; NO in-blob pairing data →
  pair-merge impossible. specter.rs:142-144 *assumes* `<0;1>` (UNVERIFIED).
- **`--format descriptor`** (descriptor.rs:41-78): single descriptor; `/0/*` rejects, not rescued by merge.
- **old `--json` replay** (bundle --import-json): one descriptor per envelope; hard-fails under both options.
- Unaffected (synthesize their own `/<0;1>/*`): sparrow, electrum, coldcard, coldcard-multisig.

## EXTERNAL FACTS — SOURCE-VERIFICATION RESULTS
- [x] **Bitcoin Core `listdescriptors`** (verified vs bitcoin/bitcoin master + PR #22838, `gh api`): ALWAYS
      TWO SEPARATE objects — `.../0/*` `"internal": false` (receive) + `.../1/*` `"internal": true` (change);
      scalar `internal` bool per object IS the receive/change signal (`src/wallet/rpc/backup.cpp`,
      `ExportDescriptors` in `src/wallet/export.cpp` loops one desc per ScriptPubKeyMan). Multipath `<0;1>` is
      **IMPORT-ONLY** (PR #22838, milestone 29.0): *"The wallet will not output the multipath descriptors (yet)...
      a multipath descriptor is expanded to the two descriptors"* — current master still `CHECK_NONFATAL(descs.size()==1)`
      on export; `doc/descriptors.md` tells users to manually assemble `<0;1>`. ⇒ **Pair-merge validated**:
      receive(internal:false) + change(internal:true), identical keys, → `<0;1>/*`, order by `internal` flag
      (receive=first alt=0), actual step values. No combined form ever appears in Core exports.
- [x] **Specter** (verified vs `cryptoadvance/specter-desktop` src, v2.1.10): NEVER emits `<0;1>`.
      User-facing "Export Wallet" QR/JSON `account_map` (`wallet.py:1146-1153`) = **receive-only `/0/*`, NO change
      branch in blob** (open issue #2494). Internal wallet-file JSON (`to_json`) = BOTH `recv_descriptor` +
      `change_descriptor` as SEPARATE fields. ⇒ Specter's common shared export (`account_map`) is a single
      receive-only `/0/*` → **hard-rejects under the floor, cannot pair-merge** (no change branch present). The
      toolkit's `specter.rs:142-144` `<0;1>` assumption is WRONG. **Spec wider-surface ruling for Specter:
      reject-with-pointed-remediation** ("Specter's QR/JSON export omits the change branch; import fails
      fail-closed rather than guess `/1`"). MUST check what form the toolkit's specter fixtures/importer actually
      consume today (does Specter import even work now, or does it rely on the collapse bug?).
- [x] **bare-`@N` sweep** (done): bare `@N` is the CANONICAL multisig keyless-template form (see D1 above). →
      D1 DEFERRED. Do NOT flip `lex_bare_at_zero`.

## NEXT STEPS
1. Fold external-fact recon + bare-`@N` sweep results here.
2. Write `design/SPEC_cycleA_descriptor_use_site_collapse.md` (single author).
3. R0 architect loop (opus) → 0C/0I. Persist each review verbatim to `design/agent-reports/cycleA-spec-r0-round-N.md`. Re-dispatch after every fold.
4. `design/IMPLEMENTATION_PLAN_cycleA_*.md` (TDD phases, failing tests first: every dropped shape, D1, verify-bundle false-pass regression vs BIP-84 oracle, restore --md1 wrong-address regression, both residue directions, pair-merge cases) → R0 loop → GREEN.
5. Single implementer subagent in a worktree; TDD to green. Full suite: `cargo test -p mnemonic-toolkit` + `cargo test -p wc-codec`.
6. Lockstep ripples: manual (docs/manual/src/40-cli-reference/ if behavior/error text documented); GUI schema_mirror only if a flag NAME/enum changes (D2 reuse → likely none, but confirm; trap #7 may touch --json wire-shape which is NOT schema-gated → manual GUI-consumer coordination).
7. Post-impl mandatory independent adversarial whole-diff review (opus). Folds re-enter the review loop.
8. Release ritual: PATCH bump; BOTH READMEs; fuzz/Cargo.lock; install.sh SELF-pin (NOT md-cli sibling pin); re-vendor if dep bump (none expected); CHANGELOG (tag-gated); file+flip a new FOLLOWUP slug; direct-FF + tag. md/mk/ms NO-BUMP.
