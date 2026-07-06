# CONTINUITY — Cycle A: CRITICAL descriptor use-site collapse fix

**Status:** authorized by user 2026-07-06, NOT started. Next session picks this up fresh.
**Source of truth for the bug:** `design/agent-reports/constellation-eval-2026-07-06.md` §1 finding **C1**.
**This doc:** everything a fresh session needs to execute Cycle A without re-deriving.

> Line numbers below were grep-verified against current `origin/master` on 2026-07-06 at write time.
> Per CLAUDE.md, **re-grep them at cycle start** — they decay every merge.

---

## The bug (one paragraph)

`toolkit:crates/mnemonic-toolkit/src/parse_descriptor.rs::lex_placeholders` (fn at **:60**, regex built at
**:97-102**, `captures_iter` loop at **:103**) matches `@N` + optional `/<mpath>` + optional `/*` and then
pushes the placeholder occurrence **with no unconsumed-residue check**. A fixed use-site step such as `/0/*`
(the standard Bitcoin Core / Sparrow / Blockstream Green single-descriptor receive form) matches none of the
trailing optional groups, so `/0/*` is silently dropped. `make_use_site_path` (**:290**) records only
`{multipath_alts, wildcard_hardened}`, and `md:crates/md-codec/src/use_site_path.rs` `UseSitePath` cannot
represent a fixed step — so **fail-closed reject is the correct behavior**, exactly as md-cli ruled.

Observed silent rewrites (all reproduced this session against a from-source `mnemonic 0.75.0`):
- `@0/0/*` → `@0/*`   · `@0/<0;1>/0/*` → `@0/<0;1>/*`   · `@0/0h/*` → `@0/*`   · bare `@0` → `@0/*` (wildcard **added**)

Impact: `bundle --descriptor` and `import-wallet --format descriptor|bitcoin-core` encode a **different
wallet** into the md1 card; `verify-bundle` re-parses through the same collapsing lexer and **false-passes
all 9 checks (exit 0)** even with the user's original descriptor + seed slot; `restore --md1` (watch-only /
multisig) derives the wrong address set. Single-sig-with-seed is masked by canonical regeneration; everything
else is unmasked. **Zero test coverage of these shapes today** (grep: 0 hits).

## The reference fix to mirror

md-cli already fixed this class ("M5", cycle-9 funds fix). The twin lives at
`descriptor-mnemonic/crates/md-cli/src/parse/template.rs` — same fn name `lex_placeholders` (**:32**); the
**stray-residue reject** is the block around **:81-102** (comments explicitly describe rejecting "any other
non-`[0-9;]` residue — fail-closed", covering hardened `<0';1'>` and malformed double-marker shapes). Mirror
that post-match check into the toolkit's `lex_placeholders`: after the mpath/wildcard captures, assert the
character immediately following the full match is a legal terminator (`)`, `,`, `}`, whitespace, or EOS);
otherwise return a typed error. **Note the agent's original citation (`:128-137`) had already drifted — use
the live grep.**

## Two design decisions to settle in the spec (don't skip)

1. **Bare `@N` implied-wildcard:** today bare `@0` silently *gains* `/*`. Decide explicitly — reject as
   incomplete, or keep the implied wildcard but make it a documented, tested rule (not an accident of the
   regex). The critic/refuter both flagged this needs its own ruling.
2. **Error taxonomy:** new `ToolkitError` variant(s) go in `error.rs` — **alphabetical-by-variant-name**
   ordering for new variants + their `match` arms (CLAUDE.md convention; avoids merge conflicts).

## Execution checklist (TDD + mandatory R0 gate)

- [ ] **cycle-prep / brainstorm → SPEC**, then **R0 architect review (opus) until 0C/0I** BEFORE any code
      (CLAUDE.md hard gate). Persist reviews verbatim to `design/agent-reports/`.
- [ ] Write failing tests FIRST for every dropped shape above + the two design decisions + a `verify-bundle`
      false-pass regression + a `restore --md1` wrong-address regression. Oracle = the official BIP-84 vector:
      correct first receive for `abandon×11 about` is `bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu`; the
      collapsed card currently (wrongly) restores `bc1q8vph849lf3e9rrj85hsxrzlv949rtahe794k6p`.
- [ ] Single implementer subagent in a worktree; TDD to green.
- [ ] Full-package suite: `cargo test -p mnemonic-toolkit` (R0 reviews run the FULL suite — CLI/parse
      changes ripple into argv/schema/version lints). Also `cargo test -p wc-codec` (currently un-CI'd).
- [ ] **Lockstep ripples:** (a) manual — any CLI-surface/behavior change touches
      `docs/manual/src/40-cli-reference/`; (b) GUI schema mirror only if a flag NAME/enum changes (this fix
      likely doesn't add flags — a new reject path is behavior, not surface — but confirm). No fmt on GUI.
- [ ] **Post-impl mandatory independent adversarial whole-diff review (opus)** — non-deferrable.
- [ ] **Release ritual** per memory `project_toolkit_release_ritual_version_sites`: bump version, BOTH
      READMEs, `fuzz/Cargo.lock`, `install.sh` SELF-pin (NOT the frozen md-cli sibling pin — bumping that
      breaks `sibling-pin-check`), re-vendor on any dep bump, CHANGELOG (gated on the tag). Toolkit ships
      direct-FF + tag. md/mk/ms unaffected (NO-BUMP) unless the spec decides to touch md-codec.
- [ ] File/flip FOLLOWUPS: add a slug for this (none existed), mark RESOLVED in the shipping commit.

## Commit trailer for the resuming session
Read the live session model + URL from the harness prompt — do NOT hardcode this session's values.
`Co-Authored-By: Claude <model> <noreply@anthropic.com>` + `Claude-Session: <live url>`.

## After Cycle A ships
The other authorized-but-deferred cycles are B–G in the eval report §3. Recommend F's items #1 (branch
protection — **no repo's suite gates merges today**) and #2 (wire wc-codec tests into CI) land early so the
new C1 regression tests actually gate. C (BCH repair miscorrection, incl. the un-bounded ms1-seed path) is
the next-highest funds risk.
