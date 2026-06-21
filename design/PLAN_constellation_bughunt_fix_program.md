# PLAN — Constellation bug-hunt fix PROGRAM (tiered / phased)

**Status:** PLANNING ONLY — no code, no edits. This is the program-management plan-doc that
sequences the fixes for the 55 confirmed + 5 downgraded-but-real findings from the 2026-06-20
adversarial funds-safety bug-hunt.

**DIFFERENTIAL-ORACLE UPDATE (2026-06-20, `wf_8c03549a-c7c`).** The empirical diff-oracle wave (real
regtest addresses + Bitcoin Core v27.0 `deriveaddresses` + independent BIP32/secp256k1) re-tiered the
findings; **these empirical severities SUPERSEDE the static ratings.** Net effect on this plan:
- **3 findings ESCALATED to CRITICAL (were HIGH), now a new Tier 0** that ships FIRST: **H12** (descriptor-mode
  taproot multisig derives cosigner keys at BIP-48 `2'` not `3'` — every address diverges, proven at the
  key level), **H1** (`verify-bundle` returns `result:ok` for a wrong-threshold/unsorted/script-type bundle
  — false GREEN on the verification tool itself), **H13** (hardened multipath `<0';1'>`/`<0h;1h>` silently
  collapsed to bare `/*` → wrong addresses; md-cli + toolkit lockstep). **H12-crossmode** is the same root
  cause as H12 (template `3'` vs descriptor `2'`) and folds into H12's workstream.
- **7 findings DEMOTED to metadata/fidelity-only** (mechanism reproduces but derived **addresses are
  IDENTICAL** — fidelity / PSBT-device-matching / availability, NOT wrong-address funds-loss): **H14, H11,
  M7, M1, L3, M3, M10**. Re-ranked downward into the lower tiers below.
- **H15 → MEDIUM** (corrupt/edited-input-only; legitimate network-consistent round-trips are a clean-negative).
- Result: **3 CRITICAL · 12 HIGH · 14 MEDIUM · 26 LOW** + 5 downgraded-but-real. _(See `constellation-bughunt-2026-06-20.md`
  → "Differential-oracle wave — EMPIRICAL results".)_

**Source of findings:** `design/agent-reports/constellation-bughunt-2026-06-20.md`
(post-oracle: 3 CRITICAL · 12 HIGH · 14 MEDIUM · 26 LOW + 5 downgraded). Cite IDs are H1..H15 / M1..M14 /
L1..L26 + the four downgraded-but-real items, named below as **D-decay-rel**, **D-decay-abs**,
**D-mdcli-coin**, **D-md-chunk-budget**, and the **D-mk-crosschunk** asymmetry.

**Repo HEADs at hunt time** (the bases each workstream branches from): md `54dd765`, mk `1279ef9`,
ms `6b28918`, toolkit `8967294d` (feature/bundle-md1-template-multisig), gui `5ee127c`.
**Citation-decay caveat (CLAUDE.md):** every file:line in this plan is a snapshot from the report.
Each workstream's brainstorm/plan-doc MUST re-grep against current `origin/master` (or the integration
branch HEAD) per the "plan-doc + spec citations are grep-verified at write time" convention and pin a
source SHA before any code.

> **HARD-GATE REMINDER (CLAUDE.md, non-negotiable):** every NON-trivial fix passes a brainstorm-spec
> R0 review loop (→0C/0I) AND a plan-doc R0 review loop (→0C/0I) BEFORE any code; per-phase TDD
> (tests first); a SINGLE subagent per phase in a worktree (never parallel re-impls of the same bug);
> and a MANDATORY independent adversarial post-implementation review over the whole diff. Every
> review persists verbatim to `design/agent-reports/` BEFORE the fold-and-commit. No advancing past
> any gate (start-code / next-phase / tag / ship) with an open Critical or Important.

---

## 0. Executive summary

**Structure:** 1 gating phase (Phase 0) + **Tier 0 (diff-oracle CRITICALs, ship-first)** + 5 funds-impact
tiers. Within each tier, findings are bundled into **workstreams** (one per merge-conflict zone / lockstep
unit); the 3 high-leverage **structural fixes** are elevated to land at the head of Tiers 1–2 because each
one closes a *cluster* of individual findings.

- **Phase 0 (gating, serial-first):** the differential-oracle harness — extend
  `tests/bitcoind_differential.rs` into the validation oracle every class-A (wrong-address) fix is gated on.
  Nothing in Tier 0+ class-A merges until its shape is GREEN in Phase 0's corpus.
- **Tier 0 — CRITICAL (empirically-proven funds-loss; diff-oracle-escalated; SHIPS FIRST):** **H12**
  (+**H12-crossmode**, descriptor-mode taproot BIP-48 `2'`→`3'`), **H1** (`verify-bundle` false-GREEN on
  wrong-policy bundles), **H13** (hardened-multipath collapse → wrong addresses, md-cli+toolkit lockstep).
  H12+H1 are ONE serialized **S-VERIFY-zone** workstream (`bundle.rs`/`verify_bundle.rs`, can't be two
  concurrent agents); H13 is a separate concurrent two-repo-lockstep workstream. **These three ship before
  the structural Tier-1 cluster-closers.**
- **Tier 1 — structural funds-safety (closes clusters):** S-NET (network-provenance invariant → closes
  H15/M13/M14 + L1/L2/L10/L11/L12/H9 + D-mdcli-coin; absorbs the now-DEMOTED L3 origin-fidelity item),
  S-VERIFY (bundle↔verify-bundle dedup → **anchors Tier-0 H1+H12** above, also closes L24, absorbs
  H7-mirror + the downgraded multiset item), S-TEMPLATE (template path mirrors keyed path → closes H8/L9;
  absorbs the now-DEMOTED M7 JSON-threshold item).
- **Tier 2 — non-structural HIGH funds-loss:** H10, H7, H6/M4, M6. _(H12/H13 promoted OUT to Tier 0; H11/H14
  demoted DOWN to metadata/fidelity — see Tiers 3/5.)_
- **Tier 3 — panics/DoS + MEDIUM funds/fidelity:** H4, H5, M2, M8, M5, M11, M12, **H15 (↓MEDIUM,
  corrupt-input)**; **+ demoted-to-metadata** M1, M3, M10 (addresses correct — fidelity/availability only).
- **Tier 4 — secret-hygiene (GUI cluster + toolkit):** H2, H3, M9, L21, L22, L23, L5, FU-ZEROIZE.
- **Tier 5 — LOW fidelity/UX/library + downgraded + demoted-to-metadata:** L4, L6, L7, L8, L13, L14, L15,
  L16, L17, L18, L19, L20, L25, L26, D-decay-rel, D-decay-abs, D-md-chunk-budget, D-mk-crosschunk; **+ the
  demoted-to-metadata HIGHs/LOW H11, H14, L3** (origin/fingerprint-fidelity only — addresses correct).

**Batch concurrency schedule (≤10 parallel implementer agents per batch, each itself single-subagent-per-phase):**
- **Batch 0:** Phase-0 oracle (1 workstream, runs alone first — it is the gate).
- **Batch 0.5 — Tier-0 CRITICALs (≤2 concurrent; realistic peak = 2):** the diff-oracle-escalated
  funds-loss items, the **first implementation batch after the oracle**. Just TWO concurrent workstreams:
  **(a) S-VERIFY-zone serialized workstream = H1 + H12 (+H12-crossmode)** — both live in the
  `bundle.rs`/`verify_bundle.rs` zone (H12's taproot-aware default-origin helper + H1's policy-structure
  compare), so they **CANNOT be two concurrent agents — they serialize onto the S-VERIFY branch together**;
  and **(b) WS-MD-CLI-LEX-H13 = H13** — the md-cli `parse/template.rs` + toolkit `parse_descriptor.rs`
  two-repo lockstep, a separate concurrent workstream. (S-VERIFY's broader dedup scope continues in Batch 1,
  but its Tier-0 anchor hunks land here first.)
- **Batch 1 (≤4 concurrent):** S-NET, the remainder of S-VERIFY's dedup, S-TEMPLATE, plus codec WS-MD-BCH
  (H6/M4) — all disjoint file zones; S-* must precede the Tier-1/2 findings they subsume.
- **Batch 2 (≤8 concurrent):** the Tier-2 non-structural HIGHs that don't collide with a still-open S-* —
  WS-EXPORT-MULTISIG (H10 + the demoted-fidelity H11), WS-MS-CODEC (H4/H5/M6/L5/L26),
  WS-MD-CLI-LEX (M2/M5/M10/M11/D-mdcli-coin — the H13 half already shipped in Batch 0.5),
  WS-MK (M12/L20), WS-GUI-SECRET (H2/H3/M9/L12/L13), plus residual import-side findings rebased onto S-NET.
- **Batch 3 (≤8 concurrent):** Tier-3/4/5 leftovers grouped by file zone (see §6 schedule).

**Counts:** **55 confirmed (post-oracle: 3 CRITICAL · 12 HIGH · 14 MEDIUM · 26 LOW) + 4 named-downgraded-but-real
+ 1 asymmetry + 1 folded-in FOLLOWUP (FU-ZEROIZE) = 61 items.** _The diff-oracle re-tiering changed
SEVERITIES and TIER placement, not the item count or the formal/trivial lane split below._
- **Formal (full brainstorm→R0→plan→R0→TDD→impl→adversarial-review):** **31**
  (the 3 CRITICAL + 12 HIGH = all 15 former-HIGH IDs, all MEDIUM except the trivially-mechanical M12, plus
  the funds-relevant downgraded decay items and L8/L18/L21 which alter funds-safety behavior). Several are
  *bundled* into a single structural workstream, so they share one formal workflow (e.g. S-NET runs ONE
  formal workflow that closes H15+M13+M14+L1+L2+L3+L10+L11+H9; the Tier-0 S-VERIFY-zone runs ONE workflow
  for H1+H12).
- **Trivial/mechanical (reviewed-patch lane, one reviewer pass, no full spec/plan):** **30**
  (most LOWs that are doc/advisory/display/regex-class/dead-code/test-vacuity edits + M12 + the folded-in
  secret-hygiene **FU-ZEROIZE** `Zeroizing`-wrap, absorbed into the `bundle.rs`-zone branch).

**Critical path (longest serial chain):**
`Phase-0 oracle GREEN` → **Tier-0 CRITICALs** (`S-VERIFY-zone H1+H12 serialized branch` ‖ `H13 md-cli+toolkit
lockstep` — the FIRST implementation batch, both class-A so both gated on the oracle) → `S-NET formal
workflow (toolkit MINOR)` → (its class-A reverifications gated on Phase-0) → for any codec-rooted fix the
chain lengthens: **H6/M4 (md-codec tag+publish) → toolkit pin-bump**, and **H13 (md-cli tag+publish, lockstep
with the toolkit mirror) → toolkit pin-bump** (H13 is itself a Tier-0 item, so its publish chain is on the
critical path EARLY). The single longest toolkit-only path is **Phase-0 → Tier-0 S-VERIFY-zone (H1+H12) →
S-NET → toolkit class-A reverify**, but the **codec-publish-before-pin chains (H13 in Tier-0; H6/M4, M6 in
Tier-2) are the schedule-critical risk** because they cross a crates.io publish boundary the toolkit cannot
shortcut. **H13 being CRITICAL makes its md-cli tag+publish→toolkit-pin lockstep the earliest cross-repo
publish gate in the program.**

**Sequencing risks (full list §7):** (0) **Tier-0 is the first impl batch and H12+H1 CANNOT be two
concurrent agents** — both edit the `bundle.rs`/`verify_bundle.rs` zone, so they serialize on ONE
S-VERIFY-zone branch (H12's taproot-aware default-origin helper + H1's policy-structure compare land
together); H13 runs concurrently as its own two-repo-lockstep workstream → realistic Tier-0 peak = 2 agents;
(1) codec tag/publish must land before any toolkit pin that needs it (**H13 in Tier-0**; H6/M4, M6, M10/M11
in later tiers if the toolkit wants them); (2) S-NET must land before its subsumed import-parser findings or
they conflict in the same files; (3) **H12 is now CRITICAL and its fix IS part of the S-VERIFY-zone Tier-0
work** — H12's default-origin helper is mirrored to `verify_bundle.rs` + `xpub_search/descriptor_intake.rs`
and rides the SAME S-VERIFY dedup as H1 so the wrong-path fixes don't drift again; (4) H13 is a TWO-repo
lockstep (md-cli + toolkit `parse_descriptor.rs`) — never ship one half (and it's a Tier-0 CRITICAL, so it
ships first); (5) every toolkit CLI-surface change drags the GUI schema-mirror + manual in lockstep (the
gate is a LAGGING indicator — paired-PR discipline is the leading control); (6) `error.rs` new variants are
a guaranteed multi-PR conflict generator unless every workstream uses alphabetical-by-variant ordering from
the first commit.

---

## 1. Triage table (all 61 items — 60 hunt/downgraded + 1 folded-in FOLLOWUP FU-ZEROIZE)

**Trivial-vs-formal classification RULE (decides which lane a finding takes):**

A finding is **TRIVIAL/MECHANICAL** (→ reviewed-patch lane: one reviewer pass, no brainstorm-spec, no
plan-doc) **iff ALL of:**
1. **No funds-safety behavior change** — it does not alter which address/descriptor/seed/network is
   produced or accepted, nor what is rejected. (Doc/advisory/label/display/JSON-metadata/dead-code/test
   edits qualify; anything that changes derive/accept/reject does NOT.)
2. **Single, mechanically-obvious edit** — one localized change (a string, a threshold constant, a regex
   character class widened to a documented set, an enum-arm pick, a dead-variant wiring, a vacuous-test
   rewrite) with no new control-flow design decision.
3. **No new public-API / wire / CLI-surface shape** — no new flag/option/subcommand/error-semantics that
   would drag the GUI schema-mirror, the manual, or a codec wire change. (A new typed error *message* on an
   already-failing path is allowed; a new *reject* of a previously-accepted input is NOT trivial.)
4. **No cross-repo publish dependency** — does not require a codec tag/publish-then-pin to land.

If ANY of the four fails → **FORMAL** (full workflow). When in doubt, a finding is FORMAL — the lane is a
floor, not a ceiling. **Bundled structural workstreams (S-NET/S-VERIFY/S-TEMPLATE) are always FORMAL** and
run ONE formal workflow that covers all their member findings.

Column legend: **WS** = workstream (§4); **MCZ** = merge-conflict zone (the file/module that forces
serialization); **lockstep** = cross-repo partners; **bump** = target repo + likely SemVer; **subsumed-by** =
the structural fix that closes/absorbs it.

| ID | repo | sev | class | lane | WS | MCZ (file/module) | lockstep partners | target + bump | subsumed-by |
|---|---|---|---|---|---|---|---|---|---|
| **H1** | toolkit | **CRIT** (↑HIGH) | B | FORMAL | **S-VERIFY (Tier 0)** | `cmd/verify_bundle.rs` ↔ `cmd/bundle.rs` (shared md1-bind) | manual if CLI text changes | toolkit MINOR | — (this IS the structural anchor; **diff-oracle CRITICAL — serializes w/ H12 on S-VERIFY branch, Batch 0.5**) |
| **H2** | gui | HIGH | D | FORMAL | WS-GUI-SECRET | `gui/src/runner.rs` | — (GUI-internal) | gui MINOR | — |
| **H3** | gui | HIGH | D | FORMAL | WS-GUI-SECRET | `gui/src/secrets.rs`,`form/invocation.rs`,`persistence.rs`,`main.rs`,`schema/mnemonic.rs:918` | toolkit `secret_taxonomy` (read-only import of `SECRET_NODE_TYPES_ARGV`) | gui MINOR | — |
| **H4** | ms-cli | HIGH | E | FORMAL | WS-MS-CODEC | `ms-cli/src/cmd/derive.rs` | — | ms-cli MINOR (tag+publish) | — |
| **H5** | ms-cli | HIGH | E | FORMAL | WS-MS-CODEC | `ms-cli/src/cmd/verify.rs` | — | ms-cli MINOR (tag+publish) | — |
| **H6** | md-codec | HIGH | C | FORMAL | WS-MD-BCH | `md-codec/src/codex32.rs`,`encode.rs`,`bch_decode.rs` (+M4) | md-cli (repair surfaces it); toolkit pin after | md-codec MINOR (tag+publish) | — (pairs w/ M4) |
| **H7** | toolkit | HIGH | B | FORMAL | S-VERIFY (shared lexer) | `parse_descriptor.rs` (`lex_placeholders`) shared by bundle+verify | manual §41 (prefix-form doc) | toolkit MINOR | partially — shares the lexer S-VERIFY consolidates |
| **H8** | toolkit | HIGH | B | FORMAL | S-TEMPLATE | `synthesize.rs` (`synthesize_template_descriptor`) | — | toolkit MINOR | — (this IS the structural anchor) |
| **H9** | toolkit | HIGH | A | FORMAL | S-NET | `cmd/import_wallet.rs`,`wallet_import/bitcoin_core.rs` | — | toolkit MINOR | **S-NET** |
| **H10** | toolkit | HIGH | A | FORMAL | WS-EXPORT-MULTISIG | `wallet_export/{coldcard,jade,electrum}.rs`,`cmd/export_wallet.rs` | manual (new refusal / `--allow-…` flag → GUI schema if a flag is added) | toolkit MINOR | — |
| **H11** | toolkit | **LOW** (↓HIGH; metadata-only) | B | FORMAL | WS-EXPORT-MULTISIG | `wallet_export/{coldcard,jade}.rs` | — | toolkit MINOR | **diff-oracle DEMOTED — origins corrupted but addresses unchanged → Tier 5 fidelity** |
| **H12** | toolkit | **CRIT** (↑HIGH) | A | FORMAL | **S-VERIFY-zone (Tier 0)** | `cmd/bundle.rs::compute_default_origin_path:2210`,`cmd/verify_bundle.rs:1373`,`xpub_search/descriptor_intake.rs` | — | toolkit MINOR | **diff-oracle CRITICAL — fix = taproot-aware default-origin helper (`3'` for `Tag::Tr`, reuse `template.rs::bip48_script_type():231`); SERIALIZES w/ H1 on the S-VERIFY branch, Batch 0.5. +H12-crossmode folds in here** |
| **H13** | md-cli (+toolkit mirror) | **CRIT** (↑HIGH) | B | FORMAL | **WS-MD-CLI-LEX-H13 (Tier 0)** | `md-cli/src/parse/template.rs:40,220-233` AND `toolkit/parse_descriptor.rs:70,227-230` | **TWO-repo lockstep**; companion FOLLOWUPs both repos; md-cli tag+publish → toolkit pin | md-cli MINOR (tag) + toolkit MINOR (pin) | — (**diff-oracle CRITICAL — own concurrent Tier-0 workstream, Batch 0.5; rest of WS-MD-CLI-LEX follows in Batch 2**) |
| **H14** | toolkit | **MED** (↓HIGH; metadata-only) | A | FORMAL | WS-IMPORT-FP (rides S-NET) | `wallet_import/coldcard_multisig.rs` | SPEC §11.4.1 note + fixtures | toolkit MINOR | **diff-oracle DEMOTED — wrong master-fp but xpub/addresses correct (breaks PSBT device-matching only) → Tier 5 fidelity**; same file family as S-NET |
| **H15** | toolkit | **MED** (↓HIGH; corrupt-input-only) | A | FORMAL | S-NET | `wallet_import/{descriptor,specter,sparrow,bitcoin_core,bsms,coldcard_multisig,electrum}.rs`,`pipeline.rs`,`slip0132.rs` | wires dead `ToolkitError::NetworkMismatch` | toolkit MINOR | — (this IS the structural anchor; **diff-oracle: legit round-trips clean-negative → MEDIUM, still S-NET-anchored**) |
| **M1** | toolkit | MED (metadata-only) | B | FORMAL | WS-EXPORT-ORIGIN | `export_wallet.rs:825`,`wallet_export/{electrum,coldcard,sparrow}.rs`,`import_wallet.rs:1547` | — | toolkit MINOR | **diff-oracle: account→0 origin wrong but addresses correct → fidelity, Tier 3** |
| **M2** | md-cli | MED | E | FORMAL | WS-MD-CLI-LEX | `md-cli/src/parse/template.rs:188-201` | md-cli tag+publish | md-cli PATCH (tag) | — |
| **M3** | md-codec | MED (fail-closed) | B | FORMAL | WS-MD-DERIVE | `md-codec/src/derive.rs:108-122` | toolkit pin after (if it wants the fix) | md-codec MINOR (tag) | **diff-oracle: change addrs UNDERIVABLE (fail-closed), NOT wrong-address → availability, Tier 3** |
| **M4** | md-codec | MED | C | FORMAL | WS-MD-BCH | `md-codec/src/bch_decode.rs` | pairs H6; toolkit pin after | md-codec MINOR (tag) | — (pairs w/ H6) |
| **M5** | md-cli | MED | B | FORMAL | WS-MD-CLI-LEX | `md-cli/src/parse/template.rs:32-91,357-381` | md-cli tag | md-cli PATCH/MINOR (tag) | adjacent H13 (same lexer file) |
| **M6** | ms-codec | MED | C | FORMAL | WS-MS-CODEC | `ms-codec/src/shares.rs`,`envelope.rs` | new `Error::InconsistentShareSet` (ms-codec wire-adjacent); ms-cli + toolkit pin after | ms-codec MINOR (tag+publish) | — |
| **M7** | toolkit | MED (metadata-only) | B | FORMAL | S-TEMPLATE | `cmd/bundle.rs:915,922,924` (JSON branch) | — | toolkit MINOR | **S-TEMPLATE** (template/keyed parity); **diff-oracle: `--json` threshold=N wrong but embedded descriptor+md1 correct → metadata, Tier 5-ish fidelity (still rides S-TEMPLATE)** |
| **M8** | toolkit | MED | A | FORMAL | WS-DESCBUILD | `descriptor_builder/ir.rs`,`gate.rs` | — | toolkit MINOR | — |
| **M9** | gui | MED | D | FORMAL | WS-GUI-SECRET | `gui/src/secrets.rs:278-310`,`schema/mod.rs`,`form/tree_model.rs` | — | gui MINOR | — |
| **M10** | md-cli | MED (availability) | E | FORMAL | WS-MD-CLI-LEX | `md-cli/src/parse/{template.rs:1792,keys.rs:67}` | md-cli tag | md-cli MINOR (tag) | **diff-oracle: BIP-86 depth-3 tr false-REJECT, NOT wrong-address → availability, Tier 3** |
| **M11** | md-cli | MED | C | TRIVIAL→FORMAL | WS-MD-CLI-LEX | `md-cli/src/parse/keys.rs:33-80` | md-cli tag | md-cli PATCH (tag) | — (new reject ⇒ FORMAL) |
| **M12** | mk-cli | MED | other | TRIVIAL | WS-MK | `mk-cli/src/cmd/repair.rs`,`mod.rs:97` | mk-cli tag+publish | mk-cli PATCH (tag) | — |
| **M13** | toolkit | MED | A | FORMAL | S-NET | `cmd/export_wallet.rs:711`,`json_envelope.rs`,`slip0132.rs:108` | — | toolkit MINOR | **S-NET** |
| **M14** | toolkit | MED | A | FORMAL | S-NET | `cmd/convert.rs:1100-1113`,`slip0132.rs:197` | — | toolkit MINOR | **S-NET** |
| **L1** | toolkit | LOW | A(disp) | FORMAL | S-NET | `cmd/build_descriptor.rs:476-485` | — | toolkit PATCH | **S-NET** |
| **L2** | toolkit | LOW | A | FORMAL | S-NET | `wallet_import/electrum.rs:698-718` | — | toolkit PATCH | **S-NET** |
| **L3** | toolkit | LOW (metadata-only) | A | FORMAL | S-NET | `wallet_import/coldcard.rs:237-241` | — | toolkit PATCH | **S-NET** (file zone); **diff-oracle: `as u32` account truncation → origin metadata wrong, addresses correct → fidelity** |
| **L4** | md-cli | LOW | D(priv) | TRIVIAL | WS-MD-CLI-ADVISORY | `md-cli/src/cmd/repair.rs:156-159` | md-cli tag | md-cli PATCH (tag) | sibling L19 |
| **L5** | ms-cli | LOW | D(latent) | TRIVIAL | WS-MS-CODEC | `ms-cli/src/error.rs:20` | ms-cli tag | ms-cli PATCH (tag) | — |
| **L6** | md-codec | LOW | E | TRIVIAL→FORMAL | WS-MD-DERIVE | `md-codec/src/canonicalize.rs:206-219` | md-codec tag | md-codec PATCH (tag) | — (guard add; bundle w/ M3) |
| **L7** | md-cli | LOW | other | TRIVIAL | WS-MD-CLI-ADVISORY | `md-cli/src/main.rs:241` (help epilog) | manual if epilog mirrored; FOLLOWUP reconcile | md-cli PATCH (tag) | — |
| **L8** | toolkit (+md-codec) | LOW | A | FORMAL | WS-RESTORE-NET | `cmd/restore.rs:1342`,`md-codec/canonical_origin.rs:61,70`,`synthesize.rs:1195` | md-codec tag if codec side; toolkit pin | toolkit PATCH (+ md-codec tag) | network-family (relate S-NET) |
| **L9** | toolkit | LOW | B | FORMAL | S-TEMPLATE | `cmd/restore.rs:1159-1585` (completion guards) | — | toolkit PATCH | **S-TEMPLATE** |
| **L10** | toolkit | LOW | A | FORMAL | S-NET | `wallet_import/bsms.rs:386-413` | — | toolkit PATCH | **S-NET** |
| **L11** | toolkit | LOW | A | FORMAL | S-NET | `cmd/convert.rs:1480-1491` | — | toolkit PATCH | **S-NET** |
| **L12** | gui | LOW | A | TRIVIAL | WS-GUI-SECRET | `gui/src/form/conditional.rs:99-126` | — (GUI-internal regex) | gui PATCH | relates S-NET (annotation form) |
| **L13** | gui | LOW | B(cov) | TRIVIAL | WS-GUI-SECRET | `gui/src/schema/mnemonic.rs:130-144` (`NODE_TYPES`) | dropdown value add → paired-PR discipline | gui PATCH | — |
| **L14** | md-codec | LOW | B | FORMAL | WS-MD-IDENTITY | `md-codec/src/identity.rs:106-113,172-240` | md-codec tag | md-codec MINOR (tag) | pairs L15/L17 |
| **L15** | md-codec | LOW | B | FORMAL | WS-MD-IDENTITY | `md-codec/src/identity.rs:71-104` | md-codec tag | md-codec MINOR (tag) | pairs L14 |
| **L16** | md-codec | LOW | E(graceful) | FORMAL | WS-MD-DERIVE | `md-codec/src/varint.rs:15-42`,`origin_path.rs`,`use_site_path.rs` | md-codec tag | md-codec MINOR (tag) | — |
| **L17** | md-codec | LOW | other(test) | TRIVIAL | WS-MD-IDENTITY | `md-codec/src/identity.rs:571-588` (vacuous test) | md-codec tag | md-codec PATCH (tag) | de-vacuify w/ L14 |
| **L18** | toolkit | LOW | E(false-rej) | FORMAL | WS-ELECTRUM-IMPORT (rides S-NET) | `wallet_import/electrum.rs:513-531,796-813` | — | toolkit PATCH | same file as L2 (S-NET zone) |
| **L19** | md-cli | LOW | D(priv) | TRIVIAL | WS-MD-CLI-ADVISORY | `md-cli/src/cmd/encode.rs:73-76,110-113` | md-cli tag | md-cli PATCH (tag) | sibling L4 |
| **L20** | mk-cli | LOW | other(disp) | TRIVIAL | WS-MK | `mk-cli/src/cmd/mod.rs:131-140` | mk-cli tag | mk-cli PATCH (tag) | pairs M12 |
| **L21** | toolkit | LOW | D | FORMAL | WS-CONVERT-BIP38 | `cmd/convert.rs:1366,932,1502` | — | toolkit PATCH | — (new refusal/warning ⇒ FORMAL) |
| **L22** | toolkit | LOW | D(tracked) | TRIVIAL→deferred | WS-SECRET-MEM-B | `slot_input.rs:203-232`,`cmd/convert.rs:747` | tracked Cycle-B `secret-memory-hygiene-cycle-b` | toolkit PATCH | DEFER to Cycle-B |
| **L23** | toolkit | LOW | E(latent) | TRIVIAL | WS-DESCBUILD | `electrum_crypto.rs:345-351` | — | toolkit PATCH | — |
| **L24** | toolkit | LOW | E | FORMAL | S-VERIFY | `cmd/verify_bundle.rs:1374-1393` (guard-asymmetry vs bundle.rs) | — | toolkit PATCH | **S-VERIFY** |
| **L25** | toolkit | LOW | other | TRIVIAL | WS-IMPORT-CLASSIFY | `wallet_import/pipeline.rs:53-60` | — | toolkit PATCH | relates S-NET file zone |
| **L26** | ms-cli | LOW | B | TRIVIAL | WS-MS-CODEC | `ms-cli/src/cmd/combine.rs:91-117` | ms-cli tag | ms-cli PATCH (tag) | — |
| **D-decay-rel** | toolkit | MED(↓high) | timelock | FORMAL | WS-DECAY | `descriptor_builder/archetype.rs:305-317` | — | toolkit MINOR | pairs D-decay-abs |
| **D-decay-abs** | toolkit | LOW(↓med) | timelock | FORMAL | WS-DECAY | `descriptor_builder/archetype.rs:305-317`,`gate.rs:306-324` | — | toolkit MINOR | pairs D-decay-rel |
| **D-mdcli-coin** | md-cli | LOW(↓med) | A | FORMAL | WS-MD-CLI-LEX | `md-cli/src/parse/path.rs:25-34`,`cmd/encode.rs:46-49` | md-cli tag | md-cli MINOR (tag) | network-family |
| **D-md-chunk-budget** | md-codec | LOW | other | TRIVIAL→FORMAL | WS-MD-DERIVE | `md-codec/src/chunk.rs:219-289` | md-codec tag | md-codec PATCH (tag) | budget arithmetic ⇒ FORMAL |
| **D-mk-crosschunk** | md-codec | LOW | C | FORMAL | WS-MD-IDENTITY | `md-codec/src/chunk.rs:305-389,175-179` | align w/ mk-codec 32-bit hash | md-codec MINOR (tag) | defense-in-depth |
| **FU-ZEROIZE** | toolkit | LOW | D | TRIVIAL | WS-SELFCHECK-ZEROIZE (rides S-VERIFY/S-TEMPLATE `bundle.rs` zone) | `cmd/bundle.rs:2473` (`self_check_bundle` ms1 decode-probe) | none (toolkit-local); fork-L1 convergence | toolkit PATCH / NO-BUMP | serialized into `bundle.rs` file-zone (rides S-VERIFY/S-TEMPLATE) |

**Formal count = 31** (the 3 CRITICAL H1/H12/H13 + the 12 remaining HIGH = all 15 former-HIGH IDs; M1,M2,M3,M4,M5,M6,M7,M8,M9,M10,M11,M13,M14; L1,L2,L3,L8,L9,L10,L11,L14,L15,L16,L18,L21,L24; D-decay-rel, D-decay-abs, D-mdcli-coin, D-md-chunk-budget, D-mk-crosschunk — *as workstream-grouped, several share one workflow*). **The diff-oracle re-tiering moves items between TIERS but NOT between LANES** — every escalated/demoted item keeps its FORMAL/TRIVIAL classification, so the 31/30 split is unchanged.
**Trivial count = 30** (M12; L4,L5,L6,L7,L12,L13,L17,L19,L20,L22,L23,L25,L26 + the remaining LOWs that are doc/display/dead-code; **+ FU-ZEROIZE**, the folded-in fork-L1-convergence secret-hygiene item). Note L6/M11/D-md-chunk-budget are TRIVIAL-leaning but tip to FORMAL because each adds a *reject* of previously-accepted input (rule clause 1/3) — counted FORMAL above.

**FU-ZEROIZE classification (TRIVIAL — applied the rule explicitly):** clause 1 ✓ (no funds-safety behavior change — a `Zeroizing` wrap around a decode-probe that is used only as an equality oracle; the `payload.as_bytes() != expected_bytes` compare and its accept/reject are byte-identical before/after — pure secret-hygiene/scrub-on-drop); clause 2 ✓ (single, mechanically-obvious edit — move the decoded entropy out of `ms_codec::Payload` into `Zeroizing<Vec<u8>>` at the ONE miss-site `bundle.rs:2473`, mirroring the in-file sibling idiom at `bundle.rs:2028-2039`; no new control-flow decision); clause 3 ✓ (no public-API / wire / CLI-surface shape — internal-only; no flag/error/codec change → no GUI schema-mirror, no manual touch); clause 4 ✓ (no cross-repo publish dependency — toolkit-local, NO codec tag/pin). ALL four hold ⇒ TRIVIAL → reviewed-patch lane (one reviewer pass, no brainstorm-spec/plan-doc), and — per §5.2 — **absorbed into the `bundle.rs`-zone workstream's branch** (rides S-VERIFY/S-TEMPLATE), not run as a separate concurrent agent. (`FU-` prefix = a folded-in FOLLOWUP item, NOT one of the H/M/L/D hunt findings.)

---

## 2. Tiering (funds-impact order; structural fixes ELEVATED; diff-oracle CRITICALs at the very top)

Tiers order by funds-safety impact. **Tier 0 (the three diff-oracle-escalated CRITICALs) sits ABOVE every
other tier and ships FIRST** — they are empirically-proven wrong-address / false-verdict funds-loss (real
diverging regtest addresses, key-level proof). Below Tier 0, each of the 3 structural fixes is hoisted to
the FRONT of the tier that holds the findings it subsumes, because landing it first **deletes work** from
the individual findings (they collapse into the shared rule rather than being patched one-by-one) and
prevents same-file re-drift.

### Phase 0 (GATING) — differential-oracle harness
See §3. Lands **before** any Tier-0/Tier-1+ class-A fix can be validated/merged.

### Tier 0 — CRITICAL (empirically-proven funds-loss; diff-oracle-escalated; the FIRST implementation batch)
The differential-oracle wave (real Bitcoin Core `deriveaddresses` + independent BIP32/secp256k1) proved
three static-HIGH findings are wrong-address / false-verdict funds-loss with diverging addresses at the key
level. **These supersede their static ratings and MUST ship before the structural Tier-1 cluster-closers.**
They form the first implementation batch after the Phase-0 oracle (Batch 0.5, §6.5) — **realistic peak = 2
concurrent agents** because two of the three serialize on one file zone.

1. **H12 (+H12-crossmode) — descriptor-mode taproot multisig derives cosigner keys at BIP-48 `2'` not `3'`
   → every address diverges.** Proof: the mk-decoded cosigner xpub is byte-for-byte the independent `2'`
   derivation (≠ `3'`); Core derives a *different* P2TR address at every index for `tr(NUMS,multi_a)` and
   `sortedmulti_a`; any BIP-48 coordinator (Sparrow/Coldcard/Jade) re-derives at `3'` → coins unspendable by
   any participant. **H12-crossmode is the same root cause** (template-mode correctly emits `3'`,
   descriptor-mode emits `2'`; no `--multisig-path-family bip48` escape hatch in descriptor-mode) → folded
   into the SAME workstream. **Fix:** a **taproot-aware default-origin helper** in
   `cmd/bundle.rs::compute_default_origin_path` (`:2210`) — emit `3'` for `Tag::Tr` instead of the hardcoded
   `2`, **reusing `template.rs::bip48_script_type()` (`:231`)** which already returns `3` for
   `TrMultiA/TrSortedMultiA` — **mirrored to its two other call sites** `verify_bundle.rs:1373` and
   `xpub_search/descriptor_intake.rs`. **This lives in the `bundle.rs`/`verify_bundle.rs` zone → it MUST ride
   / merge with S-VERIFY (the bundle↔verify_bundle dedup), NOT run as a separate concurrent agent.** class-A
   → gated on a Phase-0 oracle row asserting Core derives the `3'` P2TR address the toolkit reports.
2. **H1 — `verify-bundle` returns `result:ok` (exit 0) for a wallet that reconstructs DIFFERENTLY.** Proof:
   a real `wsh(sortedmulti(2,A,B,C))` bundle GREEN-lit a `sortedmulti(1,…)` 1-of-3 anyone-spends, an unsorted
   `multi(2,…)`, AND a `sh(wsh(…))` P2SH-nested — all with different addresses; `md1_xpub_match` only compares
   a sorted-pubkey multiset (tree Tag / threshold / wrapper never compared). **verify-bundle blindness
   COMPOUNDS H12/H10/H13 — it is the safety net that should catch all three structural-drift classes and
   doesn't.** **Fix:** add a policy-structure equality check (simplest: `expected.md1 == supplied.md1`, which
   the keyless single-sig path at `:583` already does) in `verify_bundle.rs`. **Same file zone as H12 →
   serializes with H12 on the S-VERIFY branch.**
3. **H13 — hardened multipath `<0';1'>`/`<0h;1h>` silently collapsed to bare `/*` → wrong addresses.** Proof:
   `md encode` exits 0; `md decode` (both md-cli AND the toolkit `bundle --descriptor` md1) returns the
   collapsed `wsh(multi(2,@0/*,@1/*))`; `md address` renders the bare-key address, not the intended hardened
   wallet. Core *rejects* `<0';1'>` ("not a valid uint32") → the CORRECT behavior is to ERROR, not collapse.
   **Fix = the hardened-multipath lexer fix** in md-cli `parse/template.rs:40,220-233` (`[0-9;]` class can't
   match `'`/`h`; hardcoded `hardened:false`) **AND** toolkit `parse_descriptor.rs:70,227-230` mirror — a
   **two-repo LOCKSTEP** (md-cli first or together; never one half), **its own concurrent workstream**
   (WS-MD-CLI-LEX-H13). md-cli MINOR tag+publish → toolkit pin (the toolkit mirror ships in the same toolkit
   release). class-A → gated on a Phase-0 oracle row.

**Serialization note (critical):** **H12 and H1 are ONE serialized S-VERIFY-zone workstream** — they share
`bundle.rs`/`verify_bundle.rs` and CANNOT be two concurrent agents. **H13 is a separate concurrent
workstream.** → Tier-0 realistic concurrency peak = **2 agents**. (See §6.5 Batch 0.5.)

### Tier 1 — STRUCTURAL funds-safety (cluster-closers); each lands before its subsumed findings
1. **S-NET** — fail-closed "decoded xpub network MUST agree with the asserted network / coin-type, else
   reject" across import/export/convert/build. **Closes/absorbs:** H15 (now MEDIUM, corrupt-input), M13,
   M14, H9, L1, L2, L10, L11, **L3** (now metadata-only but same `coldcard.rs` file zone) (+ network-family
   relatives L8/L12/D-mdcli-coin coordinate but live in other repos/files). Ports the existing
   `synthesize.rs:771-783` `CosignerSpec` cross-check; wires the dead `ToolkitError::NetworkMismatch`
   variant (`error.rs:266`). **Why first:** every subsumed import-parser finding edits the SAME parser files
   — patching them individually then adding the rule = guaranteed self-conflict. One rule, all parsers.
2. **S-VERIFY** — deduplicate the `bundle.rs ↔ verify_bundle.rs` descriptor-mode binding into one shared
   function. **ANCHORS the Tier-0 CRITICALs H1 + H12** (both in this zone — H1's policy-structure compare +
   H12's taproot-aware default-origin helper; they serialize onto this branch in Batch 0.5 BEFORE the rest of
   S-VERIFY's dedup completes in Batch 1). **Also closes/absorbs:** L24 (guard-asymmetry OOB), the downgraded
   multiset-index item; **mitigates** H7's shared-lexer half. **Why early:** H1/H12/L24 are all "the verify
   side drifted from the bundle side"; the only durable fix is to stop having two copies.
3. **S-TEMPLATE** — make `--md1-form=template` mirror the keyed path. **Closes/absorbs:** H8 (wordlist-language
   drop → wrong seed, *highest-impact static funds-loss this hunt*), L9 (completion-path missing refusals),
   M7 (now metadata-only: JSON threshold = N not K, path_family hardcode — addresses correct, still rides
   this parity workstream). **Why grouped:** all three are "the keyless template path is a second-class
   citizen that regressed a guard the keyed path has."

### Tier 2 — non-structural HIGH funds-loss
H10 (unsorted→sortedmulti export — empirically proven, 4/6 indices diverge), H7 (prefix-form origin dropped;
rides S-VERIFY shared lexer), H6/M4 (BCH out-of-domain — codec), M6 (ms-codec Shamir wrong-secret).
_(H12 and H13 PROMOTED OUT to Tier 0; H11 and H14 DEMOTED to metadata/fidelity — see Tiers 3/5.)_

### Tier 3 — panics/DoS + remaining MEDIUM funds/fidelity
H4, H5 (ms non-English `unreachable!`), M2 (placeholder-255 overflow panic), M8 (build-descriptor
extra-suffix), M5 (lexer/subst divergence), M11 (off-curve xpub accept), M12 (mk repair mixed-case),
**H15 (↓MEDIUM, corrupt-input-only network-byte mismatch)**.
**Diff-oracle DEMOTED-to-metadata items (mechanism reproduces, addresses CORRECT — fidelity/availability,
NOT wrong-address):** M1 (export account→0 origin), M3 (chain-gate change-addrs underivable — fail-closed
availability), M10 (BIP-86 tr false-reject — availability).

### Tier 4 — secret hygiene
H2 (GUI runner unmasked argv), H3 (minikey cleartext on disk + preview), M9 (tree key not zeroized),
L21 (bip38 empty-passphrase), L22 (stdin residue — DEFER to Cycle-B), L23 (ecies zero-scalar panic),
L5 (ms-cli Debug leak latent), **FU-ZEROIZE** (`self_check_bundle` ms1 decode-probe dropped un-scrubbed —
the lone toolkit ms1-decode miss-site; FOLLOWUP `self-check-ms1-decode-not-zeroizing`, fork-L1 convergence;
TRIVIAL `Zeroizing`-wrap, **rides the `bundle.rs` file-zone** — see §4 WS-SELFCHECK-ZEROIZE + §6.5/§7.4).

### Tier 5 — LOW fidelity / UX / library / downgraded + diff-oracle-demoted metadata-only
L4, L6, L7, L8, L13, L14, L15, L16, L17, L18, L19, L20, L25, L26, D-decay-rel, D-decay-abs,
D-md-chunk-budget, D-mk-crosschunk.
**Diff-oracle DEMOTED-to-metadata (origin/fingerprint-fidelity only — addresses CORRECT):**
**H11** (↓HIGH→LOW: coldcard/jade path-collapse to `m/0'/0'` — origins corrupted, watch-only addresses
unchanged; still in WS-EXPORT-MULTISIG file zone, rides H10's branch), **H14** (↓HIGH→MEDIUM-fidelity:
coldcard-multisig account-fp-as-master-fp — breaks PSBT device-matching, xpub/addresses correct; rides S-NET
file family), **L3** (already LOW; `as u32` account truncation — origin metadata wrong, addresses correct;
rides S-NET). _These rank BELOW the genuine LOW wrong-address/library items because they are not funds-loss._

---

## 3. Phase 0 — differential-oracle harness (GATING)

**Goal:** turn `crates/mnemonic-toolkit/tests/bitcoind_differential.rs` into the regression oracle that
(a) **gates every class-A (wrong-address) fix** — a class-A fix MUST add its triggering shape to the corpus
and the suite MUST be GREEN before that fix merges — and (b) **independently hunts the one class the static
hunt could not exclude: silent wrong-address at the wire.**

**Phase 0 is its own formal workflow** (brainstorm→R0→plan→R0→TDD→single-subagent→adversarial review), and
runs **alone in Batch 0** because every later class-A workstream depends on it.

**What the harness MUST cover** (derive addresses *every way* and diff against Bitcoin Core
`deriveaddresses` + rust-miniscript, building on the existing 9+2 shape corpus):
1. **Every derivation entry path the toolkit exposes:** `bundle --descriptor → restore` (present),
   **`import-wallet … → export-wallet`** round-trip (NEW — covers H10/H11/H14/H15/M1/M13: import a foreign
   blob, export, diff the emitted descriptor's addresses vs Core), **`build-descriptor`** human + canonical
   view (NEW — covers L1/M8), **`convert`** xpub/prefix/network edges (NEW — covers M14/L11), and the
   **template-completion** path (present).
2. **The network-provenance matrix (S-NET validation):** for each parser, a row where coin-type and
   xpub-version AGREE (must pass + derive correct HRP) and a row where they DISAGREE (must be REJECTED —
   the new fail-closed rule; assert exit≠0, NOT an address). Mainnet/testnet/signet/regtest coin-type
   coverage (covers H9/L2/L3/L8/L10).
3. **The sorted-vs-unsorted multisig discriminator** (covers H10): a `wsh(multi(...))` whose per-index
   pubkeys are NOT already in BIP-67 order, so a silent sortedmulti coercion produces a *different* address
   — the corpus must include this discriminating shape and assert the exported/Core addresses match the
   UNSORTED original (or that export is refused).
4. **Divergent-cosigner-path shapes** (covers H11/M1 — now metadata-only, but the origin-survives-round-trip
   assertion is still the right corpus coverage): cosigners at different accounts; assert per-cosigner origin
   survives import→export and Core derives identically.
5. **Taproot script-type origin** (covers **Tier-0 CRITICAL H12 + H12-crossmode**): `tr(NUMS,multi_a(...))`
   AND `tr(NUMS,sortedmulti_a(...))` descriptor-mode bundle must emit `…/3'` (NOT `2'`) and Core must derive
   the P2TR address the toolkit reports; assert descriptor-mode == template-mode at every receive+change index
   (the crossmode divergence). This is the highest-priority oracle row — it gates the first impl batch.
6. **Hardened-multipath fidelity / fail-closed** (covers **Tier-0 CRITICAL H13**): a `wsh(multi(2,@0,@1))`
   with a hardened multipath `<0';1'>`/`<0h;1h>` must EITHER round-trip to the intended hardened-derived
   addresses Core computes, OR be ERRORED at encode (Core rejects `<0';1'>` as "not a valid uint32") — assert
   it is NEVER silently collapsed to the bare-`/*` address. Clean-negative companion row: non-hardened
   `<0;1>` is wire-correct for receive AND change (do not over-reject). md-cli + toolkit both produce the same
   verdict (the two-repo lockstep).
7. **Anti-vacuity discipline (MANDATORY, mirror the existing pattern):** every NEW corpus row asserts the
   toolkit address == an INDEPENDENT rust-miniscript `derive_receive` of the original BEFORE the Core
   compare, and pins a golden; a default-CI (non-`#[ignore]`) anti-vacuity leg gates the
   toolkit↔independent-oracle equivalence even without a node (mirror `template_completion_anti_vacuity_leg`).

**Gating contract for class-A fixes (Tier 0 + Tiers 1–3):** a class-A workstream's plan-doc MUST name the corpus
row(s) it adds/turns-GREEN; its post-implementation adversarial review MUST confirm the full
`bitcoind_differential` suite (env-gated rows run in the integration-PR CI moment with the pinned node;
default-CI anti-vacuity legs run everywhere) is GREEN on the workstream branch BEFORE merge. **No class-A
fix advances to merge with a RED or absent oracle row.** The DEFAULT-CI anti-vacuity legs are the
leading control (they fire on every push); the env-gated bitcoind rows are the heavy confirmation at the
integration-PR moment.

---

## 4. Workstream grouping (by merge-conflict zone + lockstep)

Two fixes that touch the same file/module are in the SAME workstream (serialized internally,
single-subagent-per-phase) — they can NEVER be two concurrent agents. Workstreams in DIFFERENT file zones
may run as concurrent agents (subject to the structural-precedence and codec-publish ordering in §6).

**Toolkit workstreams**
- **S-NET** (FORMAL, cluster) — H15, M13, M14, H9, L1, L2, L3, L10, L11. Zone: all `wallet_import/*` parsers +
  `cmd/{export_wallet,convert,build_descriptor}.rs` + `slip0132.rs` + `error.rs` (`NetworkMismatch`). The
  single largest toolkit workstream; ONE rule, applied at every decode site.
- **S-VERIFY** (FORMAL, cluster) — **anchors the Tier-0 CRITICALs H1 + H12** (serialize together on this
  branch in Batch 0.5; see §6.5), + L24, downgraded-multiset, + H7 shared-lexer half. Zone: `cmd/bundle.rs`
  (incl. `compute_default_origin_path:2210` for H12), `cmd/verify_bundle.rs` (incl. `:1373` H12 mirror,
  `:2406-2489` H1 policy compare), `parse_descriptor.rs`, `xpub_search/descriptor_intake.rs` (third H12 call
  site). **H12 and H1 CANNOT be two concurrent agents — same file zone → ONE serialized workstream.** The
  Tier-0 anchor hunks (H1 policy-structure compare + H12 taproot-aware default-origin helper reusing
  `template.rs::bip48_script_type():231`) land FIRST in Batch 0.5; the remaining dedup (L24/H7/multiset)
  completes in Batch 1.
- **S-TEMPLATE** (FORMAL, cluster) — H8, L9, M7 (M7 now metadata-only — still rides the parity fix). Zone:
  `synthesize.rs` (`synthesize_template_descriptor`), `cmd/bundle.rs` (JSON branch), `cmd/restore.rs`
  (completion guards). **Note `cmd/bundle.rs` overlap with S-VERIFY (incl. the Tier-0 H1/H12 hunks)** → these
  serialize on `bundle.rs`; S-TEMPLATE rebases AFTER the Tier-0 S-VERIFY-zone branch lands (see §6).
- **WS-EXPORT-MULTISIG** (FORMAL) — H10 (HIGH, wrong-address); **H11 (DEMOTED HIGH→LOW metadata-only —
  path-collapse, addresses unchanged; rides this branch)**. Zone: `wallet_export/{coldcard,jade,electrum}.rs`,
  `cmd/export_wallet.rs`.
- **WS-EXPORT-ORIGIN** (FORMAL) — M1. Zone: `wallet_export/{electrum,coldcard,sparrow}.rs`, `import_wallet.rs:1547`.
  **Overlaps WS-EXPORT-MULTISIG on coldcard/electrum** → serialize (see §6).
- **WS-TAPROOT-ORIGIN** (FORMAL, **Tier-0 CRITICAL — MERGED INTO S-VERIFY**) — **H12 (+H12-crossmode)**. Zone:
  `cmd/bundle.rs::compute_default_origin_path:2210`, `cmd/verify_bundle.rs:1373`, `xpub_search/descriptor_intake.rs`.
  **NOT a separate concurrent agent** — its fix (the taproot-aware default-origin helper emitting `3'` for
  `Tag::Tr`, reusing `template.rs::bip48_script_type():231`, mirrored to all three call sites) lives in the
  `bundle.rs`/`verify_bundle.rs` zone, so it **serializes onto the S-VERIFY branch together with H1** in
  Batch 0.5. Folds H12-crossmode (template `3'` vs descriptor `2'`) into the same change set.
- **WS-IMPORT-FP** (FORMAL, **DEMOTED HIGH→MEDIUM-fidelity**) — H14 (account-fp-as-master-fp — breaks PSBT
  device-matching only; xpub/addresses correct). Zone: `wallet_import/coldcard_multisig.rs`. **Same file
  family as S-NET** → rebase onto S-NET; ranks below the wrong-address findings now.
- **WS-ELECTRUM-IMPORT** (FORMAL) — L18. Zone: `wallet_import/electrum.rs`. **Same file as L2 (S-NET)** → rebase onto S-NET.
- **WS-IMPORT-CLASSIFY** (TRIVIAL) — L25. Zone: `wallet_import/pipeline.rs`. Rebase onto S-NET (shares pipeline.rs with H15).
- **WS-RESTORE-NET** (FORMAL) — L8. Zone: `cmd/restore.rs`, `synthesize.rs`, `md-codec/canonical_origin.rs`.
  **Overlaps S-TEMPLATE on restore.rs/synthesize.rs**; the md-codec half is a codec tag → see §6.
- **WS-DESCBUILD** (FORMAL+TRIVIAL) — M8 (formal), L23 (trivial). Zone: `descriptor_builder/{ir,gate}.rs`,
  `electrum_crypto.rs`. Disjoint from the others.
- **WS-CONVERT-BIP38** (FORMAL) — L21. Zone: `cmd/convert.rs`. **Overlaps S-NET (M14/L11 also in convert.rs)** → serialize.
- **WS-DECAY** (FORMAL) — D-decay-rel, D-decay-abs. Zone: `descriptor_builder/archetype.rs`, `gate.rs`.
  **Overlaps WS-DESCBUILD on `descriptor_builder/`** → serialize (or fold into WS-DESCBUILD).
- **WS-SECRET-MEM-B** (DEFERRED) — L22. Tracked under `secret-memory-hygiene-cycle-b`; not in this program.
- **WS-SELFCHECK-ZEROIZE** (TRIVIAL) — FU-ZEROIZE (FOLLOWUP `self-check-ms1-decode-not-zeroizing`). Zone:
  `cmd/bundle.rs:2473` (`self_check_bundle` ms1 decode-probe, inside the `pub fn` at `:2312`). **Same FILE as
  S-VERIFY and S-TEMPLATE** (`cmd/bundle.rs`) — but a DISJOINT edit region: S-VERIFY's hunks are the
  descriptor-mode/md1-bind dedup (H1/L24, shared with `verify_bundle.rs`) and S-TEMPLATE's are the JSON branch
  (`bundle.rs:915,922,924`, M7), both far above the `self_check_bundle` leaf at `:2312-2480+`. Per §5.2 a
  trivial item in a FORMAL workstream's file zone is **absorbed into that workstream's branch** (rides its
  tag/review) rather than run as a separate concurrent agent → **serialize into the `bundle.rs` file-zone**
  (land with/after whichever of S-VERIFY/S-TEMPLATE owns the `bundle.rs` branch — see §6.5/§7.4). Adds one row
  to the existing `lint_zeroize_discipline` test; existing `bundle.rs:2687/2694` tests guard the equality
  behavior (pure refactor). NO-BUMP candidate (no wire/API/CLI change).

**md-codec / md-cli workstreams (codec = tag+publish→toolkit pin)**
- **WS-MD-BCH** (FORMAL) — H6, M4. Zone: `md-codec/src/{codex32,encode,bch_decode}.rs`. **Pairs the encode+decode
  length caps as one change set.** md-codec tag+publish, then toolkit pins (the md `repair` surface + toolkit
  consumers).
- **WS-MD-CLI-LEX-H13** (FORMAL, **Tier-0 CRITICAL, own concurrent workstream**) — **H13** (hardened-multipath
  lexer fix, **two-repo LOCKSTEP**: md-cli `parse/template.rs:40,220-233` + toolkit `parse_descriptor.rs:70,227-230`).
  Ships FIRST in Batch 0.5 as a separate concurrent workstream (disjoint file zone from the S-VERIFY-zone
  H1/H12 branch). md-cli MINOR tag+publish → toolkit pin (the toolkit mirror ships in the SAME toolkit release;
  never one half). Companion FOLLOWUPs in BOTH repos.
- **WS-MD-CLI-LEX** (FORMAL) — M2, M5, M10, M11, D-mdcli-coin (**the H13 half split out to WS-MD-CLI-LEX-H13
  above and ships in Batch 0.5; the rest follows in Batch 2**). Zone: `md-cli/src/parse/{template,keys,path}.rs`,
  `cmd/encode.rs`. All in the md-cli parser — serialize with/after WS-MD-CLI-LEX-H13 (same `template.rs` file,
  one md-cli tag).
- **WS-MD-CLI-ADVISORY** (TRIVIAL) — L4, L7, L19. Zone: `md-cli/src/cmd/{repair,encode}.rs`, `main.rs`,
  `output_advisory.rs`. Disjoint from WS-MD-CLI-LEX files mostly; can serialize after it to share one md-cli tag.
- **WS-MD-DERIVE** (FORMAL) — M3, L6, L16, D-md-chunk-budget. Zone: `md-codec/src/{derive,canonicalize,varint,
  origin_path,use_site_path,chunk}.rs`.
- **WS-MD-IDENTITY** (FORMAL+TRIVIAL) — L14, L15, L17, D-mk-crosschunk. Zone: `md-codec/src/identity.rs`,
  `chunk.rs`. **Overlaps WS-MD-DERIVE on `chunk.rs`** → serialize the two md-codec workstreams (or fold).

**ms-codec / ms-cli workstream**
- **WS-MS-CODEC** (FORMAL) — H4, H5, M6, L5, L26. Zone: `ms-cli/src/cmd/{derive,verify,combine}.rs`,
  `ms-cli/src/error.rs`, `ms-codec/src/{shares,envelope}.rs`. M6 is the codec wire-adjacent change
  (new `Error::InconsistentShareSet`); H4/H5/L5/L26 are ms-cli. ms-codec tags first if M6 lands, then ms-cli.

**mk-codec / mk-cli workstream**
- **WS-MK** (TRIVIAL) — M12, L20. Zone: `mk-cli/src/cmd/{repair,mod}.rs`. mk-cli tag+publish.

**GUI workstream**
- **WS-GUI-SECRET** (FORMAL+TRIVIAL) — H2 (formal), H3 (formal), M9 (formal), L12 (trivial), L13 (trivial).
  Zone: `gui/src/{runner,secrets,persistence,main}.rs`, `form/{invocation,conditional,tree_model}.rs`,
  `schema/mnemonic.rs`. **Blocked on the toolkit tag** for any pin bump it needs (H3 reads the toolkit's
  `SECRET_NODE_TYPES_ARGV` — if that constant already ships in the pinned toolkit, GUI can proceed against
  the current pin; otherwise it waits for the toolkit release that exposes it).

---

## 5. Per-bug execution spec (the workflow each lane follows)

### 5.1 FORMAL lane (every non-trivial workstream)
Per CLAUDE.md hard-gate. For each FORMAL workstream:
1. **Brainstorm spec** → `design/BRAINSTORM_<ws>.md`. Re-grep all cited file:lines vs current
   `origin/master`/integration-HEAD; pin source SHA. **R0 review loop:** dispatch opus architect →
   persist verbatim to `design/agent-reports/bughunt-fix-<ws>-brainstorm-R<n>.md` → fold → re-dispatch →
   repeat **until 0C/0I**. NO plan-doc work starts before brainstorm GREEN.
2. **Plan-doc** → `design/IMPLEMENTATION_PLAN_<ws>.md` (per-phase TDD breakdown, lockstep checklist,
   version-site list, oracle rows for class-A). **R0 review loop** (same persist→fold→re-dispatch→0C/0I).
   NO code before plan-doc GREEN.
3. **Per-phase execution:** tests FIRST (RED), then a **SINGLE subagent** implements that phase in a
   **git worktree** (`isolation: "worktree"`) — never parallel re-impls of the same bug. Per-phase
   reviewer-loop to 0C/0I; persist each review.
4. **Mandatory independent adversarial post-implementation review** over the WHOLE diff (catches
   impl-introduced regressions TDD misses; this is separate from R0 plan-correctness). Persist verbatim.
   If Agent-API dispatch fails mid-session, FLAG it explicitly and DEFER the formal review to recovery —
   never silently substitute inline self-review.
5. **For class-A:** the adversarial review confirms the `bitcoind_differential` oracle (default-CI legs +
   the env-gated rows in the integration-PR CI moment) is GREEN with the new shape(s) before merge.

### 5.2 TRIVIAL/MECHANICAL lane (reviewed-patch)
Skips brainstorm-spec + plan-doc; STILL gets **one reviewer pass** (per CLAUDE.md). Workflow: write the
test (still TDD where a behavior is observable; for pure doc/advisory edits, a snapshot/lint assertion) →
single edit → one opus reviewer pass persisted to `design/agent-reports/bughunt-fix-<id>-review.md` →
fold to 0C/0I → commit. Trivial items in the SAME file zone as a FORMAL workstream are **absorbed into that
workstream's branch** (they ride its tag/pin and its review), not run as separate agents.

### 5.3 Artifact / persistence locations (all under this repo unless a sibling repo is named)
- Brainstorm specs: `design/BRAINSTORM_bughunt_<ws>.md`
- Plan-docs: `design/IMPLEMENTATION_PLAN_bughunt_<ws>.md`
- All R0 + per-phase + post-impl reviews: `design/agent-reports/bughunt-fix-<ws>-<stage>-R<n>.md`
- FOLLOWUPs: `design/FOLLOWUPS.md` (+ companion entries in the sibling repo for any cross-repo item, per
  CLAUDE.md "Companion:" cross-citing).
- The master fix-checklist (tick `- [ ]` with the fixing commit) stays in
  `design/agent-reports/constellation-bughunt-2026-06-20.md`.

---

## 6. Cross-repo release sequencing + concurrency schedule

### 6.1 Codec publish-before-pin (hard ordering)
A codec fix the toolkit needs requires the codec **tag+publish to crates.io FIRST**, then a toolkit
**pin-bump** PR. Affected chains:
- **WS-MD-BCH (H6/M4):** md-codec MINOR tag+publish → toolkit pin-bump (PATCH) to consume the length-cap.
  md-cli also re-releases (its `repair` surfaces the cap).
- **WS-MD-CLI-LEX-H13 (H13 — Tier-0 CRITICAL):** md-cli MINOR tag+publish → toolkit pin **AND** the toolkit's
  own `parse_descriptor.rs` mirror ships in the SAME toolkit release (TWO-repo lockstep — never one half).
  **This is the earliest cross-repo publish gate in the program** (Tier-0, Batch 0.5) — schedule its md-cli
  tag first. (The non-H13 remainder of WS-MD-CLI-LEX shares the same md-cli tag in Batch 2.)
- **WS-MS-CODEC (M6):** ms-codec MINOR tag+publish (`Error::InconsistentShareSet`) → ms-cli release →
  toolkit pin if it consumes `combine_shares` (verify whether `ms-shares combine` rides the codec directly).
  **Caveat (MEMORY g6):** the toolkit's g6 ms-cli pin is FROZEN at a tag — a new ms-cli behavior the toolkit
  needs requires a NEW public ms-cli tag and a coordinated pin bump.
- **WS-MD-DERIVE / WS-MD-IDENTITY (M3,L6,L14-17,L16,D-*):** md-codec MINOR tag → toolkit pin only if it
  consumes the changed API (most are library-internal / latent — verify per-item whether a pin is needed).
- **WS-MK (M12/L20):** mk-cli PATCH tag+publish; toolkit pin only if it shells mk for these surfaces.

### 6.2 GUI schema-mirror + manual lockstep (toolkit CLI-surface changes)
Any toolkit flag/option/subcommand/dropdown add/remove/rename MUST update **both**
`mnemonic-gui/src/schema/mnemonic.rs` (paired PR; the `schema_mirror` gate is a LAGGING indicator — the
paired-PR rule is the leading control) **and** `docs/manual/src/40-cli-reference/` in the SAME (or paired)
PR. In-scope triggers this program: H10 if it adds `--allow-sortedmulti-coercion`; any new typed error is
message-only (no schema). L13 adds a GUI dropdown VALUE (`seedqr`) — the schema_mirror gates flag-NAMES not
VALUES, so this is **paired-PR discipline, not gate-enforced** (MEMORY: GUI value-adds are manual).
**Do NOT `cargo fmt` the GUI** (no fmt CI gate) and **NEVER `cargo fmt --all` / fmt `mlock.rs`** in toolkit
(g6 exemption).

### 6.3 error.rs ordering (multi-PR conflict avoidance)
Every workstream adding a `ToolkitError` variant uses **alphabetical-by-variant-name** ordering in the
declaration + `Display` + `exit_code` + `kind` match blocks from its first commit (CLAUDE.md). S-NET
*wires the existing* `NetworkMismatch` (no new variant). New variants likely: WS-EXPORT-MULTISIG
(coercion-refusal), S-TEMPLATE (none — reuses), WS-CONVERT-BIP38 (empty-passphrase refusal). Coordinate
these on the integration branch with the alphabetical rule so concurrent toolkit workstreams don't collide.

### 6.4 Integration-branch / per-instance branch-ownership model (≤10 concurrent agents)
Adopt the `PLAN_v0_26_0_three_way_merge.md` topology:
- **Master stays put**; create per-repo **release integration branches** (e.g.
  `release/bughunt-fix` in toolkit, mirrors in md/ms/mk/gui). Each WORKSTREAM owns ONE feature branch
  retargeted onto its repo's integration branch; the final integration PR per repo is the single
  squash-merge that the tag fires on. This isolates conflict-resolution from master and gives one
  dry-run-CI moment per repo (the integration PR is the dry-run of exactly what gets tagged).
- **Branch ownership = workstream ownership.** No agent pushes to a branch another workstream owns.
  Two workstreams sharing a file zone (the §4 overlaps) are NOT given two branches — they serialize on one
  branch, or are explicitly sequenced via rebase (the later one rebases onto the earlier's merge).
- **Per-file conflict cheat-sheet** (mirror the v0.26.0 doc): `Cargo.lock` = accept-theirs +
  `cargo check --workspace` (never `cargo update -w`); `CHANGELOG.md` = single version header, bullets in
  PR-merge order; `error.rs` = alphabetical; `cli_gui_schema.rs` = rename count fn + alphabetical vec union;
  `gui_schema.rs` = `grep -c '=> .*_conditional_rules()'` monotonic count check per rebase.

### 6.5 Batch concurrency schedule

**Batch 0 — Phase-0 oracle (1 agent, runs alone).** It is the gate; class-A workstreams can't validate
without it. (Non-class-A workstreams MAY start their brainstorm/plan R0 loops in parallel — those are
design-only, no merge — but no class-A *merge* happens before Batch 0 is GREEN.)

**Batch 0.5 — Tier-0 CRITICALs (the FIRST impl batch after the oracle; realistic peak = 2 concurrent):**
The diff-oracle-escalated funds-loss items, ahead of the structural cluster-closers. Only TWO concurrent
workstreams because two of the three CRITICALs share one file zone.
| WS | repo | findings | concurrent? | serialization / dependency |
|---|---|---|---|---|
| **S-VERIFY-zone (Tier-0 anchor)** | toolkit | **H1 + H12 (+H12-crossmode)** | **the two SERIALIZE — ONE agent/branch** | both edit `cmd/bundle.rs`/`cmd/verify_bundle.rs` (H1 = `verify_bundle.rs:2406-2489` policy-structure compare; H12 = `compute_default_origin_path:2210` taproot-aware `3'` helper reusing `template.rs::bip48_script_type():231`, mirrored to `verify_bundle.rs:1373` + `xpub_search/descriptor_intake.rs`) → CANNOT be two concurrent agents; they land the Tier-0 anchor hunks on the S-VERIFY branch FIRST, the rest of S-VERIFY's dedup completes in Batch 1. Both class-A → gated on Batch-0 oracle rows (taproot `3'`; verify-bundle false-GREEN discriminator) |
| **WS-MD-CLI-LEX-H13** | md-cli (+toolkit mirror) | **H13** | **yes — separate concurrent workstream** | disjoint file zone (`md-cli/src/parse/template.rs` + toolkit `parse_descriptor.rs` mirror); TWO-repo LOCKSTEP — md-cli MINOR tag+publish → toolkit pin (mirror in the same toolkit release); class-A → gated on the Batch-0 hardened-multipath oracle row |

→ **Tier-0 concurrency = 2 agents** (≤10 cap honored). S-VERIFY-zone H1+H12 = 1 serialized branch; H13 = 1
separate branch. Batch 1's structural cluster-closers do NOT start their `bundle.rs`/`verify_bundle.rs`-zone
merges until this Tier-0 anchor lands (S-VERIFY's remaining dedup + S-TEMPLATE rebase onto it).

**Batch 1 — structural + codec-root (≤4 concurrent; all disjoint top-level zones):**
| WS | repo | concurrent? | serialization note |
|---|---|---|---|
| S-NET | toolkit | yes | head-of-Tier-1; its subsumed import findings rebase onto it |
| S-VERIFY (remaining dedup: L24/H7/multiset) | toolkit | yes | continues on the SAME branch the Tier-0 H1+H12 anchor landed on in Batch 0.5; disjoint from S-NET files |
| S-TEMPLATE | toolkit | **partial — after Tier-0** | shares `cmd/bundle.rs` with S-VERIFY (incl. the Tier-0 H1/H12 hunks) → S-TEMPLATE rebases onto the S-VERIFY-zone branch (or coordinates the bundle.rs hunks) |
| WS-MD-BCH | md-codec | yes | codec; tag+publish gates the toolkit pin that follows in Batch 2 |

**Batch 2 — Tier-2 HIGHs + demoted-fidelity + codec/cli/gui (≤8 concurrent):**
| WS | repo | concurrent? | serialization / dependency |
|---|---|---|---|
| WS-EXPORT-MULTISIG (H10 + demoted-fidelity H11) | toolkit | yes | H10 class-A → gated on Batch-0 oracle row; H11 (metadata-only) rides the same branch |
| WS-EXPORT-ORIGIN (M1, metadata-only) | toolkit | **no — serialize** | shares coldcard/electrum export files w/ WS-EXPORT-MULTISIG → same branch or rebase-after |
| WS-IMPORT-FP (H14, demoted fidelity) | toolkit | **no — rebase onto S-NET** | same `coldcard_multisig.rs` file zone as H15 |
| WS-MS-CODEC (H4/H5/M6/L5/L26) | ms | yes | ms-codec tag (M6) → ms-cli → toolkit pin chain |
| WS-MD-CLI-LEX (M2/M5/M10/M11/D-mdcli-coin) | md-cli | yes | **H13 already shipped in Batch 0.5**; this remainder shares the same md-cli tag (serialize with/after WS-MD-CLI-LEX-H13 on `template.rs`) |
| WS-MK (M12/L20) | mk-cli | yes | independent tag |
| WS-GUI-SECRET (H2/H3/M9/L12/L13) | gui | yes | blocked only on a toolkit pin if H3 needs a newer `SECRET_NODE_TYPES_ARGV` |

_(H12 and H13 are NO LONGER in Batch 2 — both promoted to Tier-0/Batch 0.5. WS-TAPROOT-ORIGIN is merged into
the S-VERIFY-zone branch above.)_

**Batch 3 — Tier-3/4/5 remainder (≤8 concurrent, grouped by zone to avoid collisions):**
| WS | repo | concurrent? | note |
|---|---|---|---|
| WS-CONVERT-BIP38 (L21) | toolkit | **no — serialize after S-NET** | shares `convert.rs` w/ M14/L11 |
| WS-DESCBUILD (M8/L23) + WS-DECAY (D-decay-rel/abs) | toolkit | **serialize together** | both in `descriptor_builder/` → one branch |
| WS-RESTORE-NET (L8) | toolkit+md-codec | **after S-TEMPLATE** | shares restore.rs/synthesize.rs; md-codec half = codec tag |
| WS-ELECTRUM-IMPORT (L18) / WS-IMPORT-CLASSIFY (L25) | toolkit | rebase onto S-NET | shared `electrum.rs`/`pipeline.rs` |
| WS-MD-DERIVE (M3/L6/L16/D-md-chunk-budget) + WS-MD-IDENTITY (L14/L15/L17/D-mk-crosschunk) | md-codec | **serialize** | both touch `chunk.rs` → one branch / one tag |
| WS-MD-CLI-ADVISORY (L4/L7/L19) | md-cli | after WS-MD-CLI-LEX | shares md-cli tag |
| WS-SELFCHECK-ZEROIZE (FU-ZEROIZE) | toolkit | **no — absorbed into the `bundle.rs` branch** | rides S-VERIFY/S-TEMPLATE (`cmd/bundle.rs` zone); disjoint edit region (`self_check_bundle:2473`) so no content conflict, but NOT a separate concurrent agent — lands with/after that branch's tag/review (§5.2) |

---

## 7. Dependency graph + critical path + per-tier exit criteria

### 7.1 Dependency edges (→ = "must land before")
```
Phase-0 oracle ──────────────► every class-A WS merge — Tier-0 FIRST: H12(+crossmode), H13;
                                then H9,H10,H15,M8,M13,M14,L1,L2,L8,L10,L11,L18
                                (H1=false-verdict gated too; H11/H14/M1/M3/M10/L3 now metadata — addr-correct)
Tier-0 S-VERIFY-zone (H1+H12) ► S-VERIFY remaining dedup (L24/H7/multiset) ► S-TEMPLATE (bundle.rs rebase)
Tier-0 WS-MD-CLI-LEX-H13 (md-cli tag) ► toolkit pin + parse_descriptor.rs mirror (lockstep, same toolkit release)
                                       ► rest of WS-MD-CLI-LEX (M2/M5/M10/M11/D-mdcli-coin) shares the tag
S-NET ───────────────────────► H14(rebase, fidelity), L18(rebase), L25(rebase), WS-CONVERT-BIP38(serialize)
S-TEMPLATE ──────────────────► WS-RESTORE-NET(restore.rs overlap)
WS-MD-BCH tag+publish ───────► toolkit pin (H6/M4 consume)
WS-MS-CODEC (ms-codec M6 tag)► ms-cli release ──► toolkit pin (if it consumes combine_shares)
toolkit release (any CLI-surface change) ──► GUI schema-mirror + manual (paired/lockstep)
WS-EXPORT-MULTISIG ──────────► WS-EXPORT-ORIGIN (coldcard/electrum file overlap)
WS-DESCBUILD ────────────────► WS-DECAY (descriptor_builder/ overlap)  [or fold]
```

### 7.2 Critical path (longest serial chain)
**Tier-0 leads, then codec-rooted chains dominate** (they cross a crates.io publish boundary). The
program-critical chain now starts with the diff-oracle CRITICALs:
`Phase-0 oracle (formal, 1 cycle)` → **Tier-0 Batch 0.5** [`S-VERIFY-zone H1+H12 serialized branch (formal,
toolkit MINOR)` ‖ `WS-MD-CLI-LEX-H13 (md-cli MINOR tag+publish → toolkit pin + mirror, two-repo lockstep)`]
→ `S-NET (formal, large) → S-NET class-A reverify GREEN on the oracle → toolkit MINOR release → GUI
schema-mirror/manual lockstep`. The **H13 two-repo lockstep** (md-cli tag → toolkit pin + mirror, same
toolkit release) is the tightest cross-repo coupling AND — being a Tier-0 CRITICAL — the **earliest** and
highest sequencing-risk publish gate in the program. In parallel the codec-only chain is
`WS-MD-BCH md-codec brainstorm→R0→plan→R0→TDD→impl→review→tag→publish → toolkit pin-bump PATCH (H6/M4)`.
**Schedule the codec workstreams — H13's md-cli tag in Tier-0/Batch 0.5; WS-MD-BCH, WS-MS-CODEC in Batch
1/early Batch 2 — to START their formal loops early** so their tags publish before the toolkit needs to pin
them.

### 7.3 Per-tier exit criteria (ALL must hold to advance)
- **All R0 loops 0C/0I**, reviews persisted verbatim to `design/agent-reports/` before fold-and-commit.
- **Per-phase TDD GREEN** + the mandatory whole-diff adversarial post-impl review GREEN (0C/0I).
- **CI green** on the integration-PR dry-run (full `cargo test -p` package suite — NOT targeted `--test`
  targets, per MEMORY `feedback_r0_review_run_full_package_suite`; clippy `-D warnings`; manual lint;
  install-pin-check; schema-mirror).
- **Differential-oracle GREEN** for every class-A fix's added corpus row (default-CI legs everywhere; the
  env-gated bitcoind rows in the integration-PR CI moment).
- **Lockstep done:** GUI schema-mirror + manual updated for any CLI-surface change; codec tag+publish landed
  before the toolkit pin; FOLLOWUPS companion entries filed in both repos; the
  `constellation-bughunt-2026-06-20.md` checkbox ticked with the fixing commit; FOLLOWUP status flipped in
  the shipping commit (MEMORY `feedback_followup_status_discipline`).
- **Version sites bumped** (MEMORY `project_toolkit_release_ritual_version_sites`): toolkit BOTH READMEs +
  `fuzz/Cargo.lock` + `scripts/install.sh` self-pin; re-run suite + fuzz after the bump, before the tag.

### 7.4 Explicit "do NOT parallelize" callouts
0. **(TIER-0) H1 ‖ H12 on the S-VERIFY-zone branch — CANNOT be two concurrent agents.** Both edit
   `cmd/bundle.rs` (H12's `compute_default_origin_path:2210`) and `cmd/verify_bundle.rs` (H1's policy-structure
   compare `:2406-2489` + H12's mirror `:1373`) → they SERIALIZE onto ONE S-VERIFY branch in Batch 0.5. H13
   is the only Tier-0 item that runs concurrently (separate `parse/template.rs`/`parse_descriptor.rs` zone) →
   Tier-0 peak = 2 agents.
1. **S-TEMPLATE ‖ S-VERIFY (incl. Tier-0 H1/H12) ‖ WS-SELFCHECK-ZEROIZE (FU-ZEROIZE) on `cmd/bundle.rs`** —
   all touch bundle.rs; coordinate hunks or serialize. The Tier-0 H1/H12 hunks land FIRST (Batch 0.5);
   S-VERIFY's remaining dedup + S-TEMPLATE's JSON-branch hunks rebase onto that branch in Batch 1.
   FU-ZEROIZE's region (`self_check_bundle:2473`, the leaf at `:2312`) is DISJOINT from all of them, so it
   carries NO content-conflict risk — but it still rides the SAME `bundle.rs` file-zone branch (absorbed per
   §5.2, NOT a concurrent agent). Land it with/after whichever of S-VERIFY/S-TEMPLATE owns that branch.
2. **WS-EXPORT-ORIGIN ‖ WS-EXPORT-MULTISIG** — shared coldcard/electrum export files.
3. **WS-CONVERT-BIP38 ‖ S-NET** — shared `cmd/convert.rs` (M14/L11/L21).
4. **WS-IMPORT-FP / WS-ELECTRUM-IMPORT / WS-IMPORT-CLASSIFY ‖ S-NET** — shared parser files
   (`coldcard_multisig.rs`, `electrum.rs`, `pipeline.rs`) — all rebase onto S-NET, never concurrent.
5. **WS-TAPROOT-ORIGIN (H12) is MERGED INTO the Tier-0 S-VERIFY-zone branch** (see callout 0) — not a
   separate agent; its `verify_bundle.rs:1373` + `descriptor_intake.rs` mirrors ride the same dedup so the
   wrong-path fix can't re-drift.
6. **WS-MD-DERIVE ‖ WS-MD-IDENTITY on `chunk.rs`** — same md-codec file → one branch/one tag.
7. **WS-DESCBUILD ‖ WS-DECAY on `descriptor_builder/`** — fold or serialize.
8. **H13 halves (TIER-0)** — never ship md-cli without the toolkit `parse_descriptor.rs` mirror (two-repo
   lockstep); ships first in Batch 0.5.
9. **Any two toolkit workstreams adding `error.rs` variants** — not a hard serialize (alphabetical ordering
   makes the merge mechanical), but the integration branch resolves them in alphabetical order per §6.3.

---

## 8. Open program decisions (for the user / coordinator)
1. **Scope cut for v1 of the program:** ship Phase-0 + **Tier-0 (the diff-oracle CRITICALs — these are the
   must-ship-first wave)** + Tier-1 + Tier-2 as one release wave, defer Tier-3/4/5 to follow-on waves?
   (Recommended — the empirically-proven funds-loss is now CONCENTRATED in Tier-0 (H12/H1/H13), with the
   structural cluster-closers + remaining wrong-address HIGHs in Tiers 1–2; Tiers 3–5 are
   panics/UX/library/latent + the demoted-to-metadata items (H11/H14/M1/M3/M10/L3 — addresses correct).
   **Consider an even tighter first cut: Phase-0 + Tier-0 ONLY as an emergency funds-safety hotfix wave**,
   since H12/H1/H13 are live wrong-address/false-verdict bugs in the current `feature/bundle-md1-template-multisig`
   branch HEAD.)
2. **L22 (stdin residue)** — keep DEFERRED to the tracked `secret-memory-hygiene-cycle-b`, or pull into
   Tier-4 WS-GUI-SECRET adjacency? (Recommended: keep deferred — it's already tracked and Cycle-B-scoped.)
3. **Codec pin strategy** — batch ALL md-codec fixes (WS-MD-BCH + WS-MD-DERIVE + WS-MD-IDENTITY) into a
   SINGLE md-codec release + one toolkit pin, vs. per-workstream tags? (Recommended: one md-codec release
   for the whole program to minimize pin-bump churn — but WS-MD-BCH is funds-critical and may warrant an
   earlier standalone tag.)
4. **H10 disposition** — hard-refuse unsorted multisig for coldcard/jade/electrum (no new flag, no GUI
   lockstep) vs. gate behind `--allow-sortedmulti-coercion` (new flag → GUI schema + manual lockstep)?
   (Recommended: hard-refuse for the steel-backup threat model; pointer to a faithful format. Less surface.)
```
