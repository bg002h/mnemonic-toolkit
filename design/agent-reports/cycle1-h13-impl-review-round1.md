# cycle-1 H13 — per-phase implementation review (round 1)

**Reviewer:** opus adversarial implementation-review agent
**Scope:** H13 only (both legs). H12/H1 are NOT in either commit under review (toolkit
`080ac03e` is H13-only; H12/H1 are a separate S-VERIFY-zone workstream).
**Artifacts reviewed:**
- md-cli leg — worktree `…/descriptor-mnemonic/.claude/worktrees/cycle1-h13-mdcli`,
  branch `fix/cycle1-h13-md-cli-reject`, commit `081f61c5`.
  Diff: `git diff origin/main...HEAD` (touches `crates/md-cli/src/parse/template.rs` +
  new `crates/md-cli/tests/h13_hardened_multipath_reject.rs`).
- toolkit leg — worktree `…/mnemonic-toolkit/.claude/worktrees/cycle1-critical`,
  branch `fix/cycle1-critical-fixes`, commit `080ac03e`.
  Diff: `git diff origin/master...HEAD` (touches `crates/mnemonic-toolkit/src/parse_descriptor.rs` only).
- GREEN design: `BRAINSTORM_cycle1_critical_fixes.md` §3, §6; `IMPLEMENTATION_PLAN_cycle1_critical_fixes.md` P1/P2, §3.

**Method:** built BOTH binaries and ran the actual inputs empirically (md `encode` / mnemonic
`bundle --descriptor`), inspected `parse_descriptor` return values via a temporary probe test
(reverted), built `origin/main` md in a throwaway worktree to establish pre-fix baseline, and
read the authoritative rust-bitcoin 0.32.8 `ChildNumber::FromStr` source.

---

## VERDICT: **NOT-GREEN — 1 Critical / 0 Important**

The well-formed primary case (`<0';1'>` / `<0h;1h>` / single-marker / mixed-single-marker) is
fixed correctly and at exact md-cli↔toolkit parity. BUT the implementation **introduces a NEW
silent-collapse funds-bug** on the **malformed double-marker** body class (`<0'';1>`, `<0'h;1>`,
`<0h';1>`) that the plan EXPLICITLY required to be a typed reject (plan §2 P1 test 2,
§3 m-2, brainstorm §3.3(b)/§6.2). This is a regression: pre-fix these inputs errored LOUDLY
(exit 1); post-fix they SILENTLY COLLAPSE to a bare-`/*` single-path card (exit 0). The
plan-mandated `encode_malformed_hardened_multipath_rejects` test was never written, which is
why it slipped through.

---

## Critical

### C1 — malformed double-marker multipath bodies (`<0'';1>`, `<0'h;1>`, `<0h';1>`) SILENTLY COLLAPSE to bare `/*` (funds-bug REGRESSION); plan-mandated reject test never written. BOTH legs.

**Empirical proof (md-cli, commit `081f61c5`):**
```
$ md encode "wsh(multi(2,@0/<0'';1>/*,@1/<0'';1>/*))" --key @0=<tpub> --key @1=<tpub> --network regtest
md1yp qqscy ...            ← EXIT 0, emits an md1
$ md decode <that phrase>
wsh(multi(2,@0/*,@1/*))    ← multipath SILENTLY DROPPED → bare /* single-path card
```
Same silent collapse for `<0'h;1>` and `<0h';1>` (all → `wsh(multi(2,@0/*,@1/*))`, exit 0).

**Empirical proof (toolkit, commit `080ac03e`)** via a temporary probe test calling
`parse_descriptor("wsh(multi(2,@0/<0'';1>/*,@1/<0'';1>/*))", &[], &[])` directly (probe
reverted after):
```
MALFORMED <0'';1> PARSED (no reject). use_site_path.multipath = None  ← SILENT COLLAPSE to bare /*
MALFORMED <0h';1> PARSED. multipath = None                            ← same
```
(The CLI surface masks this behind the incidental BIP-388 distinct-key check when both slots
share an xpub, but the parse itself succeeds with the multipath dropped — with distinct keys it
would emit a collapsed bare-`/*` md1.)

**Pre-fix baseline (built `origin/main` md `0.7.1` in a throwaway worktree):**
| input | pre-fix (origin/main) | post-fix (this PR) |
|---|---|---|
| `<0';1'>` (well-formed) | exit 0, silent-collapse — the ORIGINAL H13 funds-bug | exit 1, typed reject ✓ FIXED |
| `<0'';1>` (malformed)   | **exit 1, loud miniscript error** | **exit 0, SILENT COLLAPSE** ✗ REGRESSED |

So the change fixes the well-formed case but **regresses the malformed case from loud-error to
silent-collapse** — the exact failure class H13 exists to eliminate, now reachable by a plausible
user typo (double apostrophe, or mixing `'` and `h`).

**Root cause.** Two interacting pieces:
1. The capture regex is a strict alternation `(?:\d+(?:'|h)?)(?:;\d+(?:'|h)?)*` inside an
   **optional** group `(?:/<…>)?` (md-cli `template.rs:49` group 3; toolkit `parse_descriptor.rs:74`
   group 4). When a body like `0'';1` does NOT fully match the strict alternation (the second `'`
   has no following `;`), the **whole optional `(?:/<…>)?` group skip-matches empty** rather than
   failing — so the placeholder lexes as if there were no multipath at all, and the literal
   `<0'';1>` text becomes residual. The per-alt `ends_with('\''/'h')` reject never runs because
   group 3/4 captured nothing.
2. The H13 strip-regex widening to `[0-9;'h]` (md-cli `template.rs:463`; toolkit
   `parse_descriptor.rs:354`) then **matches-and-strips** the residual `<0'';1>` (every char is in
   `[0-9;'h]`), so `substitute_synthetic` removes it cleanly → `@0/*` → a valid keyless template,
   NO error. Pre-fix the strip class was `[0-9;]`, which could NOT match the `'`, leaving a
   residual that miniscript rejected loudly. **The strip-widening is the direct silencer.** (The
   plan called the strip-widen "optional defense-in-depth"; in fact it is the regression vector,
   because it strips malformed-but-`'h`-containing residuals that previously errored.)

This directly refutes plan §3 m-2's claim that the strict alternation "yields the *primary* typed
reject on a malformed body rather than a generic catch-all" — the strict alternation inside an
**optional** group does the opposite: it falls through to silent skip + strip.

**Plan requirement violated.** The reject of malformed hardened markers was a GREEN-design
requirement, not a nice-to-have:
- Plan §2 P1 test 2: `encode_malformed_hardened_multipath_rejects` — `<0'';1>` / stray `'h` ⇒
  exit 1, typed `TemplateParse`. **NOT WRITTEN** — the md-cli test file has only
  `_apostrophe_rejects`, `_h_form_rejects`, `_mixed_rejects`, `_nonhardened_roundtrips`.
- Plan §3 m-2; brainstorm §3.3(b) ("Malformed hardened markers … also a typed parse error") and
  §6.2 ("malformed `<0'';1>` → typed `CliError::TemplateParse`"). Neither leg implements or tests
  this.

**Why Critical (not Minor):** it is a wrong-address / un-restorable silent-collapse — the toolkit
emits a card whose multipath is dropped, watching different addresses than the user typed, with no
error. It is the precise funds-safety invariant H13 is chartered to enforce ("fail-closed, loud,
typed — NEVER silently collapse to bare `/*`", brainstorm §3.2). That the trigger is a malformed
body does not downgrade it: the safe outcome for malformed input is a loud error (which pre-fix
already produced), and the PR replaced that with silence.

**Suggested fix direction (for the implementer, not implemented here):**
- Make the malformed body reject rather than skip. Options: (a) widen the *capture* group to a
  permissive class that ALWAYS captures the `<…>` body if a `/<` opener is present (e.g.
  `(?:/<([^>]*)>)?` or `[0-9;'hH]+`), THEN validate each `;`-split token (a token that is not
  `\d+` with at most one trailing `'`/`h`, or that carries any marker, is a typed reject); this
  guarantees the per-alt validator SEES every body. (b) Keep the strict alternation but do NOT
  widen the strip regex to `[0-9;'h]` for bodies that failed the lexer — i.e. ensure a residual
  `<…>` containing a marker reaches a loud error rather than being stripped. (a) is cleaner and
  also naturally covers the uppercase-`H` body for free.
- Add the plan-mandated tests on BOTH legs: `<0'';1>`, `<0'h;1>`, `<0h';1>` ⇒ typed reject
  (exit 1 md-cli / exit 2 toolkit), stdout has no `md1` / parse returns `Err`. Add an
  anti-collapse assertion (decode must NOT yield `@N/*`).

---

## Important

None.

---

## Minor

### m1 — uppercase-`H` and malformed bodies reject with a *generic* miniscript/base58 message, not the H13 "hardened" message.
For inputs the H13 capture does not match (uppercase `<0H;1H>`; and, were C1 fixed, any
non-canonical body), the user sees `miniscript parse failed: at derivation index '0H': invalid
child number format` (md-cli) / `descriptor parse failed: …` (toolkit) instead of the friendly
"hardened derivation is impossible on a watch-only (xpub) card" guidance. Funds-safe (these all
fail closed once C1 is fixed), but a worse message. Optional polish; not blocking.

### m2 — the H13 reject error message is good but the toolkit `make_use_site_path` doc-comment claims the `Result` "fails closed should a hardened alt ever reach it via a future caller" — but `make_use_site_path` itself contains NO hardened check (it builds `hardened: false` unconditionally). The fail-closed property lives entirely in `lex_placeholders`. The doc slightly overstates the function's own guarantee. Cosmetic.

### m3 — md-cli test `make_use_site_path_nonhardened_alts_are_normal` and toolkit `parse_descriptor_nonhardened_multipath_ok` trip a dead-code `never used` warning under some build configs (binary-private test fns). Pre-existing pattern in the file; not introduced here. No action.

---

## Bypass hunt: uppercase-`H` / mixed-marker

**Empirical, per leg (binaries built and run):**

| input | md-cli (`md encode`) | toolkit (`bundle --descriptor`) | classification |
|---|---|---|---|
| `<0';1'>` | exit 1, "…`0'` is hardened…", no md1 | exit 2, "…`0'` is hardened…", no md1 | (a) correctly REJECT ✓ |
| `<0h;1h>` | exit 1, "…`0h` is hardened…", no md1 | exit 2, "…`0h` is hardened…", no md1 | (a) correctly REJECT ✓ |
| `<0;1'>` / `<0h;1'>` (mixed single markers) | exit 1, hardened reject | exit 2, hardened reject | (a) correctly REJECT ✓ |
| **`<0H;1H>` (UPPERCASE H)** | exit 1, "miniscript parse failed: at derivation index '0H': invalid child number format", **no md1** | exit 2, "descriptor parse failed: at derivation index '0H': …", **no md1** | (b) errors differently — funds-SAFE |
| **`<0';1H>` (mixed ' + H)** | exit 1, "…index '1H': invalid child number…", no md1 | exit 2, "…index '1H': …", no md1 | (b) errors differently — funds-SAFE |
| **`<0H;1'>` (mixed H + ')** | exit 1, "…index '0H': …", no md1 | exit 2, "…index '0H': …", no md1 | (b) errors differently — funds-SAFE |
| **`<0'';1>` / `<0'h;1>` / `<0h';1>` (malformed double-marker)** | **exit 0, decodes to `wsh(multi(2,@0/*,@1/*))`** | **parse Ok, `use_site_path.multipath = None`** | **(c) SILENTLY COLLAPSE — see C1** |
| `<0;1>` (clean non-hardened) | exit 0, round-trips to `wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))` | parses past lexer (no over-reject) | no over-reject ✓ |
| `/*'` / `/*h` hardened WILDCARD | accepted (`wildcard_hardened` path intact) | accepted | hardened-wildcard NOT broken ✓ |

**Uppercase-`H` verdict — NOT a bypass.** Both legs are funds-SAFE on `H`. The detection covering
only `'`/`h` is CONSISTENT with the codebase's own acceptance set, which I verified against the
authoritative parser:
- rust-bitcoin 0.32.8 `ChildNumber::FromStr` (`bip32.rs:227`):
  `let is_hardened = inp.chars().last().map_or(false, |l| l == '\'' || l == 'h');` — accepts ONLY
  `'` and `h`, **NOT `H`**. (BIP-380 permits `'`/`h`/`H`, but rust-bitcoin does not implement `H`.)
- Empirically, an uppercase **origin** path `[fp/48H/1H/0H/2H]` is REJECTED ("base58 encoding
  error"), whereas `48h` and `48'` are accepted (toolkit `hform_hardened_paths_accepted` test
  confirms `h`-form acceptance). So `H` is NOT a recognized hardened form anywhere in the
  constellation — neither in multipath bodies nor in origin paths — so the multipath detect is not
  obligated to cover it, and `H` cannot reach a hardened derivation. Both legs correctly fail
  closed on `H` (no silent collapse, no md1 emitted).

**Mixed single-marker (`<0;1'>`, `<0h;1'>`) verdict — correctly REJECT.** The strict alternation
matches these fully, the per-alt `ends_with` detect fires, typed reject. Parity holds.

**Malformed double-marker verdict — (c) SILENT COLLAPSE — this is C1.** See above. This is the one
real bypass: not via `H`, but via malformed `''`/`'h`/`h'` bodies that escape the capture group and
get silently stripped.

---

## Other checks

- **md-cli↔toolkit PARITY (Critical if broken): EXACT.** Capture regex
  `(?:\d+(?:'|h)?)(?:;\d+(?:'|h)?)*` identical on both legs; per-alt detect
  `n.ends_with('\'') || n.ends_with('h')` identical; strip regex `[0-9;'h]` identical. Both reject
  the SAME input set with the SAME semantics, AND share the SAME C1 silent-collapse on malformed
  bodies (the bug is symmetric — not a parity divergence, but a shared defect). No input is rejected
  by one and collapsed by the other.
- **`make_use_site_path` returns Result and call sites propagate.** md-cli: already `Result`; both
  production callers (`template.rs:287`,`:291`) use `?`. toolkit: widened to
  `Result<UseSitePath, ToolkitError>` (`parse_descriptor.rs:249`); both callers in
  `resolve_placeholders` (`:209`,`:213`) use `?`. ✓
- **Exit codes:** md-cli `TemplateParse` ⇒ exit **1** (`main.rs` catch-all `Err(e)=>from(1)`,
  empirically confirmed); toolkit `DescriptorParse` ⇒ exit **2** (`exit_code` arm; empirically
  confirmed). ✓
- **No new error variants.** Neither `error.rs` is in either diff; H13 reuses
  `TemplateParse`/`DescriptorParse`. ✓
- **No over-reject.** Clean `<0;1>` round-trips on both legs (md-cli decodes back to the same
  template byte-for-byte; toolkit parses past the lexer). `/*'`/`/*h` hardened WILDCARD path
  unbroken (existing `wildcard_hardened` tests pass: `at0_hardened_wildcard*`,
  `address_hardened_wildcard_card_refuses_loudly`). ✓
- **Tests.** md-cli `cargo test -p md-cli`: ALL GREEN incl. the 4 new H13 tests. toolkit
  `cargo test -p mnemonic-toolkit --bin mnemonic` (where `parse_descriptor` lives — note it is a
  **binary**-private module mounted in `main.rs`, NOT `--lib`, so `--lib` does NOT exercise these
  tests): 1012 passed / 0 failed incl. all 7 new H13 tests. The suites pin the well-formed RED→GREEN
  correctly, but have a **test gap**: the plan-mandated malformed-double-marker reject test is
  absent on both legs (the reason C1 slipped).
- **SemVer / scope.** md-cli touched only `parse/template.rs` + a new test file; toolkit touched
  only `parse_descriptor.rs`. No `Cargo.toml` version bump, no README, no `mlock.rs`, no `error.rs`,
  no fmt churn. Clean scope. ✓

---

## Required to reach GREEN

1. Fix C1 on BOTH legs: malformed double-marker bodies (`<0'';1>`, `<0'h;1>`, `<0h';1>`) must
   produce a typed reject (md-cli exit 1 / toolkit exit 2), never a silent bare-`/*` collapse.
   Ensure the fix does not re-introduce over-rejection of clean `<0;1>`.
2. Add the plan-mandated `encode_malformed_hardened_multipath_rejects` (md-cli) and an equivalent
   `parse_descriptor`/`bundle` malformed-reject test (toolkit), each with an anti-collapse
   assertion (decoded output must not be `@N/*`; no `md1` on stdout). Written RED-first against the
   current (buggy) build to confirm they catch the regression.
3. Re-dispatch this review after the fold (reviewer-loop continues until 0C/0I).

(Minors m1–m3 are non-blocking and may be folded opportunistically.)
