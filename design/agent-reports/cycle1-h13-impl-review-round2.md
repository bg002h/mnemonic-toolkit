# cycle-1 H13 — per-phase implementation review (round 2)

**Reviewer:** opus adversarial implementation-review agent
**Scope:** H13 only (both legs) — confirm round-1 **C1** (malformed double-marker
multipath silently collapsed) is closed AND the fix introduced no new Critical/Important.
The fix changed the multipath capture to a permissive `/<([^>]*)>` + a `;`-split
validator, and **reverted** the m-3 strip-class widening (`[0-9;'h]` → `[0-9;]`).
Both could have edge effects; both were probed.

**Artifacts reviewed:**
- md-cli leg — worktree `…/descriptor-mnemonic/.claude/worktrees/cycle1-h13-mdcli`,
  branch `fix/cycle1-h13-md-cli-reject`, HEAD `ddddeff`.
  Diff: `git diff origin/main...HEAD` — `crates/md-cli/src/parse/template.rs`
  + `crates/md-cli/tests/h13_hardened_multipath_reject.rs`.
- toolkit leg — worktree `…/mnemonic-toolkit/.claude/worktrees/cycle1-critical`,
  branch `fix/cycle1-critical-fixes`, HEAD `1e1e3f3d`.
  Diff: `git diff origin/master...HEAD` — `crates/mnemonic-toolkit/src/parse_descriptor.rs` only.
- Round-1 review (the C1 this round must close):
  `design/agent-reports/cycle1-h13-impl-review-round1.md`.
- GREEN design: `BRAINSTORM_cycle1_critical_fixes.md` §3/§6;
  `IMPLEMENTATION_PLAN_cycle1_critical_fixes.md` (H13, P1/P2).

**Method:** built BOTH binaries from the worktrees; ran the actual malformed /
hardened / degenerate / out-of-range / legit inputs empirically through the
production CLI surface (`mnemonic bundle --descriptor … --slot @N.xpub=…` /
`md encode … --key @N=…`), recording the real process exit code and whether an
`md1` phrase reached stdout; decoded the legit `<0;1>` md1 to prove no collapse;
isolated the hardened-wildcard path at the correct key depth; traced the
production `lex → resolve → substitute_synthetic` call order in both orchestrators
and confirmed `substitute_synthetic` has no out-of-module / lex-bypassing caller;
cross-checked the `;`-split validator logic against `str::parse::<u32>` semantics;
ran both full test suites.

---

## VERDICT: **GREEN — 0 Critical / 0 Important**

C1 is **closed on both legs**: the three malformed double-marker bodies
(`<0'';1>`, `<0'h;1>`, `<0h';1>`) now produce a typed, loud reject (toolkit exit
2 `DescriptorParse`; md-cli exit 1 `TemplateParse`) with **no `md1` on stdout and
no bare-`/*` collapse** — empirically verified, not merely asserted in a test.
The permissive `[^>]*` capture + `;`-split validator and the `[0-9;]` strip
revert introduced **no new Critical or Important** finding. Every malformed,
empty, whitespace, out-of-range, and anchoring-adversarial body fails closed and
loud; every legitimate non-hardened body (`<0;1>`, `<0;1;2>`) still accepts and
round-trips byte-for-byte; the hardened-wildcard path (`/*'`, `/*h`) is untouched
and still accepts. The plan-mandated malformed-reject test that round-1 flagged
as missing is now present and passing on **both** legs. Scope is clean (2 lexer
files only), no new error variants, no version/fmt/README churn. Both full suites
are green.

---

## Critical

None.

---

## Important

None.

---

## Minor

### m1 — `<0H;1H>` (uppercase H) and stray-residue bodies reject with a *generic* message rather than the friendly "hardened" guidance.
Uppercase-`H` and non-`'`/`h` residue land on the `… is not a bare unsigned
integer` arm (md-cli/toolkit) or a downstream `base58 encoding error` /
`descriptor parse failed`, not the friendly "hardened derivation is impossible on
a watch-only (xpub) card" line. This is funds-SAFE — every such input fails
closed (exit 1 / exit 2, no `md1`, no collapse). Carried over from round-1 m1;
non-blocking polish. (Note: this is correct behavior, not a defect — `H` is not a
recognized hardened form anywhere in the constellation, per rust-bitcoin
`ChildNumber::FromStr` accepting only `'`/`h`. The generic message is merely less
friendly, never less safe.)

### m2 — toolkit `make_use_site_path` doc-comment still says the `Result` "fails closed should a hardened alt ever reach it via a future caller", but the function itself contains no hardened check (it builds `hardened: false` unconditionally; the fail-closed property lives entirely in `lex_placeholders`). Cosmetic over-statement, carried from round-1 m2. The doc was lightly reworded this round but the residual claim remains. Non-blocking.

### m3 — bin-private test fns trip a `never used` dead-code lint under some build configs (pre-existing pattern; not introduced here). No action.

(No Minor is blocking. None gate GREEN.)

---

## C1 closure — confirmation

Round-1 C1 was: malformed double-marker bodies (`<0'';1>`, `<0'h;1>`, `<0h';1>`)
**silently collapse to bare `/*`** (exit 0 / parse-Ok with `multipath = None`),
because the strict-alternation capture inside the *optional* `(?:/<…>)?` group
skip-matched empty, and the H13-widened strip class `[0-9;'h]` then cleanly
stripped the marker-bearing residual. Funds-bug: a card watching different
addresses than typed, with no error.

**Root-cause fix verified in both diffs:**
1. **Capture widened to permissive `[^>]*`** — md-cli `template.rs:55`
   (`@(\d+)((?:/\d+'?)*)(?:/<([^>]*)>)?(/\*(?:'|h)?)?`); toolkit
   `parse_descriptor.rs:83`
   (`…(?:/<([^>]*)>)?(/\*(?:'|h)?)?`). A present `/<…>` delimiter now ALWAYS
   captures its body up to the first `>`, so a malformed body can no longer
   skip-match empty.
2. **Strict `;`-split validator** runs on every captured body — md-cli
   `template.rs:88-107`, toolkit `parse_descriptor.rs:132-148`: each alt is
   rejected if it `ends_with('\'') || ends_with('h')` (hardened) else must
   `parse::<u32>()` (bare integer). Identical predicate on both legs.
3. **m-3 strip revert to `[0-9;]`** — md-cli `template.rs:494`, toolkit
   `parse_descriptor.rs:353-370`. The widening (the direct silencer) is removed;
   it is moot because the lexer now rejects marker/malformed bodies FIRST.

**Empirically confirmed (built binaries, real exit codes):** all three malformed
bodies REJECT — toolkit exit **2** / md-cli exit **1**, typed parse error naming
the hardened alt, **zero `md1` on stdout, no `wsh(multi(2,@0/*,@1/*))` collapse**.
The well-formed hardened (`<0';1'>`, `<0h;1h>`) and single-marker (`<0;1'>`) still
reject; `<0H;1H>` still fails closed. The plan-mandated reject test
(`encode_malformed_hardened_multipath_rejects` md-cli;
`parse_descriptor_rejects_malformed_double_marker_no_silent_collapse` +
`lex_rejects_malformed_double_marker_multipath` toolkit) — **absent in round-1,
the reason C1 slipped** — is now present and green on both legs. **C1 CLOSED.**

---

## Edge-case probe results (empirical; built binaries)

`bundle --descriptor "wsh(multi(2,@0/<BODY>/*,@1/<BODY>/*))"` (toolkit, two
distinct valid tpubs for the accept rows) and
`md encode "wsh(multi(2,@0/<BODY>/*,@1/<BODY>/*))"` (md-cli). "rc" = real process
exit; "md1?" = did an `md1` phrase reach stdout.

| input `<BODY>` | md-cli outcome | toolkit outcome | classification |
|---|---|---|---|
| `<0'';1>` (malformed dbl) | rc 1, `template parse error … 0'' is hardened`, no md1 | rc 2, `… 0'' is hardened …`, no md1 | **REJECT — C1 closed** ✓ |
| `<0'h;1>` (malformed dbl) | rc 1, `… 0'h is hardened`, no md1 | rc 2, `… 0'h is hardened`, no md1 | **REJECT — C1 closed** ✓ |
| `<0h';1>` (malformed dbl) | rc 1, `… 0h' is hardened`, no md1 | rc 2, `… 0h' is hardened`, no md1 | **REJECT — C1 closed** ✓ |
| `<0';1'>` (well-formed hardened) | rc 1, `… 0' is hardened`, no md1 | rc 2, `… 0' is hardened`, no md1 | REJECT ✓ |
| `<0h;1h>` (h-form hardened) | rc 1, `… 0h is hardened`, no md1 | rc 2, `… 0h is hardened`, no md1 | REJECT ✓ |
| `<0H;1H>` (uppercase H) | rc 1, `… 0H is not a bare unsigned integer`, no md1 | rc 2, same, no md1 | REJECT (fail-closed) ✓ |
| `<>` (empty) | rc 1, `… `` … not a bare unsigned integer`, no md1 | rc 2, same, no md1 | REJECT ✓ |
| `<;>` (two empty alts) | rc 1, empty-alt reject, no md1 | rc 2, same, no md1 | REJECT ✓ |
| `<0;>` (trailing empty) | rc 1, empty-alt reject, no md1 | rc 2, same, no md1 | REJECT ✓ |
| `<;1>` (leading empty) | rc 1, empty-alt reject, no md1 | rc 2, same, no md1 | REJECT ✓ |
| `<0;;1>` (interior empty alt) | rc 1, empty-alt reject, no md1 | rc 2, same, no md1 | REJECT ✓ |
| `< 0 ; 1 >` (whitespace) | rc 1, ` 0 ` not bare u32, no md1 | rc 2, same, no md1 | REJECT ✓ |
| `<0;4294967296>` (> u32::MAX) | rc 1, `4294967296` not bare u32, no md1 | rc 2, same, no md1 | REJECT (no silent accept) ✓ |
| `<0/1>` (anchoring) | rc 1, `0/1` not bare u32, no md1 | rc 2, same, no md1 | REJECT ✓ |
| `<0>1>` (anchoring) | rc 1, downstream base58 reject, no md1 | rc 2, downstream parse reject, no md1 | REJECT (no over-run/collapse) ✓ |
| `<0;1>extra` (trailing junk) | rc 1, downstream base58 reject, no md1 | rc 2, downstream parse reject, no md1 | REJECT ✓ |
| `<<0;1>>` (nested) | rc 1, `<0` not bare u32, no md1 | rc 2, same, no md1 | REJECT ✓ |
| `<0>` (single alt) | rc 1, downstream `alt-count 1 out of range; require 2≤count` | rc 2, `md1 multipath alt-count 1 out of range (2..=9)` | REJECT (pre-existing md1 ≥2 constraint, NOT a collapse; lexer captured `[0]` correctly) ✓ |
| `<0;1>` (legit) | rc 0, **md1 emitted**; decode → `wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))` (no collapse) | rc 0, **md1 emitted** | **ACCEPT — no over-reject** ✓ |
| `<0;1;2>` (legit) | rc 0, **md1 emitted** | rc 0, **md1 emitted** | **ACCEPT** ✓ |
| `/*'` hardened WILDCARD (depth-correct key) | rc 0, **md1 emitted** | rc 0, **md1 emitted** | ACCEPT (untouched by fix) ✓ |
| `/*h` hardened WILDCARD (depth-correct key) | rc 0, **md1 emitted** | rc 0, **md1 emitted** | ACCEPT (untouched by fix) ✓ |

**Not one input produced a silent collapse to bare `/*`.** Every non-legit input
is a loud typed error with no `md1`; every legit input accepts. The hardened
wildcard accept was verified at the correct key depth — an earlier apparent
rejection was solely a depth-3-vs-4 key mismatch (plain `/*` failed identically),
unrelated to H13; the `wildcard_hardened` capture group (md-cli group 4 / toolkit
group 5) is untouched by the diff.

---

## m-3 strip-revert check

- **Strip class reverted to `[0-9;]` on both legs** (md-cli `template.rs:494`;
  toolkit `parse_descriptor.rs:353-370`). The H13 widening to `[0-9;'h]` — the
  round-1 silencer — is gone.
- **No reintroduced silent-strip path.** The revert is safe because the lexer
  rejects every marker/malformed body BEFORE `substitute_synthetic` runs.
  Verified the production call order is `lex → resolve → substitute_synthetic`
  with `?`-propagation that short-circuits on the lexer reject:
  - toolkit `parse_descriptor` — `lex_placeholders` `:819`, `resolve_placeholders`
    `:820`, `substitute_synthetic` `:831`.
  - md-cli `parse_template` — `lex_placeholders` `:1880`, `resolve_placeholders`
    `:1881`, `substitute_synthetic` `:1883` (the orchestrator called by
    `cmd/encode.rs:45`, `cmd/address.rs`, `cmd/verify.rs`, `format/json.rs`).
- **No lex-bypassing caller.** Toolkit `substitute_synthetic` is `pub fn` but has
  **no caller outside `parse_descriptor.rs`** (grep-confirmed); the only other
  parse entry (`wallet_import/descriptor.rs:68`) routes through the
  `parse_descriptor` orchestrator, so it too is lex-first. md-cli
  `substitute_synthetic` is private. The strip therefore never sees a
  marker-bearing body on any production path.
- **No legit case broken by the revert.** `<0;1>` / `<0;1;2>` accept and
  round-trip; the `[0-9;]` strip cleanly removes the synthetic multipath segment
  for legit bodies exactly as pre-H13.

---

## Parity (Critical if broken): EXACT

md-cli and toolkit reject/accept the **same set with the same semantics**:

- **Capture regex** — both widened the multipath body to permissive `([^>]*)`
  inside the optional `(?:/<…>)?` group (md-cli group 3, `template.rs:55`;
  toolkit group 4, `parse_descriptor.rs:83`).
- **Validator** — both run the identical `;`-split predicate: reject if
  `n.ends_with('\'') || n.ends_with('h')` (typed hardened error), else
  `n.parse::<u32>()` (bare-integer error). md-cli `template.rs:91-105`; toolkit
  `parse_descriptor.rs:135-146`.
- **Strip class** — both `[0-9;]` (md-cli `template.rs:494`; toolkit
  `parse_descriptor.rs` group). Lockstep.
- The full empirical probe table above is **byte-for-byte identical in
  classification** between the two legs (only the exit code 1↔2 and the
  `template parse error`↔`descriptor parse failed` prefix differ, per each CLI's
  own error taxonomy). No input is rejected by one and collapsed/accepted by the
  other.

---

## Tests, suites, scope

- **Plan-mandated malformed-reject test now present (round-1 gap closed):**
  - md-cli integration `tests/h13_hardened_multipath_reject.rs::encode_malformed_hardened_multipath_rejects`
    — loops `<0'';1>`/`<0'h;1>`/`<0h';1>`, asserts exit 1, `template parse error`,
    and `md1`-absent on stdout (anti-collapse). **5/5 pass.**
  - md-cli bin unit `parse::template::lex_tests::lex_rejects_malformed_double_marker_multipath`.
  - toolkit `lex_rejects_malformed_double_marker_multipath` +
    `parse_descriptor_rejects_malformed_double_marker_no_silent_collapse` +
    clean-negative `parse_descriptor_nonhardened_multipath_ok`.
- **Validator-logic cross-check:** a `str::parse::<u32>`-faithful simulation of
  the `;`-split validator reproduces the empirical CLI verdicts for all 16 body
  classes exactly (empty/whitespace/marker/out-of-range reject; `0`,`0;1`,`0;1;2`
  accept). The new tests are non-vacuous (they assert reject on inputs that the
  pre-fix build accepted-and-collapsed, per round-1's empirical baseline).
- **Full suites GREEN:**
  - md-cli `cargo test -p md-cli`: **213 passed / 0 failed** (incl. the 5 H13
    integration tests + 6 H13 lex_tests + all 15 lex_tests).
  - toolkit `cargo test -p mnemonic-toolkit --bin mnemonic`: **1014 passed /
    0 failed / 1 ignored** (the H13 tests live in the binary-private
    `parse_descriptor` module).
- **Scope clean.** md-cli diff: only `parse/template.rs` + the new test file.
  toolkit diff: only `parse_descriptor.rs`. **No `error.rs`** (no new error
  variants — H13 reuses `TemplateParse`/`DescriptorParse`), **no `Cargo.toml`/
  version bump**, no README, no `mlock.rs`, no fmt churn.
- **Exit codes** preserved per each leg's taxonomy: md-cli `TemplateParse` ⇒ 1;
  toolkit `DescriptorParse` ⇒ 2.

---

## Required to reach GREEN

Nothing. **GREEN at 0 Critical / 0 Important.** C1 is closed on both legs, the
permissive-capture + validator + strip-revert fix introduced no new Critical or
Important finding, parity is exact, both suites are green, and scope is clean.
Minors m1–m3 are non-blocking polish and may be folded opportunistically.
