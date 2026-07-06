# cycle-prep recon — 2026-07-06 — cycleA-descriptor-use-site-collapse

**Origin/master SHA at recon time:** `8c8b9183`
**Local branch:** `master`
**Sync state:** `up-to-date (0 ahead / 0 behind)`
**Untracked:** many `cycle-prep-recon-*.md` + `design/*` banked-recon/spec drafts (none touch `crates/`); this recon adds one more.

No FOLLOWUP slug exists for this cycle (the continuity doc notes "add a slug … (none existed)"). The **citation source is `design/CONTINUITY_cycleA_descriptor_use_site_collapse.md`**; every path/line/fact below was re-verified against the working tree at `8c8b9183` (== origin/master). Expectation going in: continuity line numbers were grep-verified 2026-07-06 at write time, so low drift — confirmed, with **two substantive refinements** and **one newly-surfaced risk** the doc does not address.

---

## Per-slug verification

### cycleA-descriptor-use-site-collapse

- **WHAT (from CONTINUITY doc):** `toolkit parse_descriptor.rs::lex_placeholders` matches `@N[..]/<mpath>/*` but performs **no unconsumed-residue check**, so a fixed use-site step (`/0`, `/0h`) after `@N` is silently dropped — `@0/0/*` collapses to `@0/*`, encoding a **different wallet** into the md1 card. `bundle --descriptor` / `import-wallet --format descriptor|bitcoin-core` mis-encode; `verify-bundle` re-parses through the same lexer and false-passes; `restore --md1` derives the wrong address set. Fix = mirror md-cli's M5 "stray-residue reject" (fail-closed, because md1 `UseSitePath` cannot represent a fixed step).

- **Citations:**
  - `crates/mnemonic-toolkit/src/parse_descriptor.rs:60` `fn lex_placeholders` — **ACCURATE**. Confirmed: `pub fn lex_placeholders(descriptor: &str) -> Result<Vec<PlaceholderOccurrence>, ToolkitError>` at :60.
  - `…parse_descriptor.rs:97-102` regex built — **ACCURATE**. `Regex::new(` at :97; pattern string at :98; `.expect(...)` at :100. Pattern: `(?:\[pfx\])?@(?P<idx>\d+)(?:\[sfx\])?(?:/<(?P<mpath>[^>]*)>)?(?P<wild>/\*(?:'|h)?)?`.
  - `…parse_descriptor.rs:103` `captures_iter` loop — **ACCURATE**. `for caps in re.captures_iter(descriptor)` at :103.
  - `…parse_descriptor.rs:290` `make_use_site_path` — **ACCURATE**. `fn make_use_site_path(occ: &PlaceholderOccurrence) -> Result<UseSitePath, ToolkitError>` at :290; body reads **only** `occ.multipath_alts` + `occ.wildcard_hardened` (:291-302) — no carrier for a fixed step, confirming any fixed step is already lost before this fn.
  - **Residue/terminator check ABSENT in toolkit lexer** — **CONFIRMED (this is the bug)**. `grep -n 'match_end\|caps.get(0)\|unconsumed\|terminator'` over `parse_descriptor.rs` → 0 code hits (only prose in comments at :95, :152, :154, :1794). The loop at :103-190 pushes `PlaceholderOccurrence` with no post-match residue assertion.
  - `md-codec/src/use_site_path.rs` `UseSitePath` cannot represent a fixed step — **ACCURATE (fail-closed reject is correct)**. `struct UseSitePath` at **:49**; fields are `multipath: Option<Vec<Alternative>>` (:51) and `wildcard_hardened: bool` (:53) — **no fixed-step field**. `MIN_ALT_COUNT = 2` (:43), so a single-element multipath is also invalid → a lone `/0/*` receive-chain form is genuinely un-representable. *(Minor doc nit: continuity names the field `multipath_alts`; the struct field is `multipath`. `multipath_alts` is the toolkit `PlaceholderOccurrence` field, not the md-codec struct field. Cosmetic.)*
  - **md-cli reference fix** `descriptor-mnemonic/crates/md-cli/src/parse/template.rs:32` `fn lex_placeholders` — **ACCURATE**. But the continuity re-citation of the reject **"block around :81-102" is IMPRECISE — see Refinement 1 below.** The actual M5 unconsumed-residue terminator check is the `match_end` block at **:128-137** (comment header :111-127).
  - **Crypto/protocol vector** — BIP-84 first receive `bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu` at `m/84'/0'/0'/0/0` for `abandon×11 about` — **ACCURATE, verified against PRIMARY SOURCE** (subagent fetched `bitcoin/bips` `bip-0084.mediawiki` Test-vectors section; both the mnemonic string and the address quote verbatim). The "currently-wrong" restore `bc1q8vph849lf3e9rrj85hsxrzlv949rtahe794k6p` is an observed-output claim, not a spec fact — leave to the failing-test oracle to reproduce.
  - **"Zero test coverage of these shapes today (grep: 0 hits)"** — **ACCURATE for the *dropped-shape lex* (`@N/0/*`)**, but **see Risk R1**: `/0/*` appears heavily in *full-descriptor* fixtures (Bitcoin Core import), which are a regression-blast-radius the fix must not break.

- **Action for brainstorm spec:** Mirror md-cli `template.rs:128-137`'s `match_end` residue/terminator check into toolkit `lex_placeholders` after the multipath validator (toolkit :146-178) — **adapted, not copy-pasted** (Refinement 2). Cite source SHA `8c8b9183` (toolkit) + the md-cli twin's current SHA in the spec. Settle both continuity design decisions (bare `@N`; error taxonomy) and **resolve Risk R1 before writing code.**

---

## Cross-cutting observations

**Refinement 1 — the md-cli reject citation points at the wrong sub-block.** md-cli `template.rs` has **two** distinct guards: (a) the multipath-body `[0-9;]` validator at :77-110 (comments :81-102) — **which the toolkit ALREADY mirrors** at `parse_descriptor.rs:146-178` (the H13/C1 fix, verbatim comment parallel); and (b) the M5 unconsumed-residue **terminator check** at :128-137 (`let match_end = caps.get(0)...; if next not in {')',',','}',whitespace,EOS} → reject`). The continuity doc's re-cite to ":81-102" is guard (a), which is already present. **The thing Cycle A must actually port is guard (b).** Cycle A ≈ porting one ~10-line block, not re-deriving the whole validator.

**Refinement 2 — the two regexes differ structurally; the port must be adapted.** md-cli's template regex is `@(\d+)((?:/\d+'?)*)(?:/<([^>]*)>)?(/\*…)?` — group 2 captures a **bare origin path** after `@N`, so md-cli's `@0/0/*` consumes `/0` as origin-path (no residue). The **toolkit** regex captures origin path **only inside brackets** (`pfx_path`/`sfx_path`); it has **no bare-path group after `@idx`**, so `@0/0/*` leaves `/0/*` as residue. Net: the residue *check* ports cleanly, but its *meaning* differs — in the toolkit a bare post-`@N` `/0` is an un-representable **use-site** fixed step (correct to reject); a verbatim copy of md-cli's terminator set is right, but the spec must state the toolkit-specific rationale (use-site, not origin) so a future reader doesn't "fix" it by adding a group-2 origin capture.

**Risk R1 (NEW — not in the continuity doc; highest-priority spec question).** `/0/*` is the standard Bitcoin Core single-descriptor receive form and appears in **many** existing passing tests/fixtures (`cli_import_wallet_bitcoin_core.rs`, `cli_cross_start_convergence.rs` — F1: "we take the `/0/*` entry", `core-*.json` fixtures, `cli_output_class.rs`, `cli_older_advisory.rs`, …). `parse_descriptor` (:834) lexes `@N`-**already-substituted** input (order: `substitute_nums`→`lex`:853→`resolve`:854→`substitute_synthetic`:865); the xpub→`@N` substitution happens **upstream in the command layer**. So whether the new reject **breaks legitimate Core imports** hinges on whether the import/bundle layer **recombines** Core's split `/0/*`+`/1/*` into `@0/<0;1>/*` *before* `parse_descriptor`, or passes `@0/0/*` straight through. The codebase's dominant internal form IS multipath (`descriptor_builder/gate.rs:361-376` rejects extra paths and appends `/<0;1>/*` itself), which is reassuring — but F1's "take the `/0/*` entry" wording is a red flag. **The spec MUST trace the full `import-wallet --format bitcoin-core|descriptor` and `bundle --descriptor` codepaths end-to-end and enumerate: (i) which real user inputs now reject, (ii) whether a receive-only `/0/*`-only wallet SHOULD reject (likely yes — un-representable) vs. a `/0/*`+`/1/*` pair that must recombine, (iii) exactly which existing tests change.** This is the make-or-break correctness question; it dwarfs the two named design decisions.

**Design decision D1 (from doc) — bare `@N` implied wildcard.** The residue check will NOT catch bare `@0` (followed by `)` = valid terminator). Today `make_use_site_path` yields `{multipath:None, wildcard_hardened:false}` = `/*` — i.e. bare `@0` silently *gains* a wildcard. This needs its own explicit ruling (reject-as-incomplete vs. documented-and-tested implied `/*`), independent of the residue fix.

**Design decision D2 (from doc) — error taxonomy.** New `ToolkitError` variant(s) in `error.rs`: **alphabetical-by-variant-name** for new variants + their `Display`/`exit_code`/`kind` match arms (CLAUDE.md convention). Decide whether to reuse the existing `DescriptorParse(String)` (all current lexer rejects use it) or add a dedicated `UseSiteFixedStepUnrepresentable`-style variant (better exit-code/`kind` granularity for GUI/JSON consumers). Reusing `DescriptorParse` = zero new variant, zero schema ripple; a new variant = alphabetical-insert + possible `--json` error-shape consumers.

**Sync/claim-counting.** Clean tree at origin/master; no DRIFTED-by-N line findings (all citations ACCURATE); the only inaccuracies are the *sub-block* imprecision (Refinement 1) and a cosmetic field-name nit. No cross-pin/version staleness surfaced.

---

## Recommended brainstorm-session scope

**One cycle, one SemVer PATCH bump on the toolkit** (behavior change — a new fail-closed reject path — **not** a CLI-surface change: no new flag/subcommand/enum value). Rough sizing: **~10-line lexer guard + error plumbing + a substantial test file** (every dropped shape, the two design decisions, a `verify-bundle` false-pass regression, a `restore --md1` wrong-address regression vs the BIP-84 oracle) + whatever Risk-R1 resolution demands (possibly an upstream recombination assertion + fixture audit). Net `src/` LOC is small; **test LOC + the R1 pipeline audit dominate the effort.**

**Locksteps:**
- **GUI `schema_mirror`:** **NOT triggered** if the fix adds no clap flag/enum-value (a reject path is behavior, not surface). Confirm during impl — if D2 chooses a new error variant that surfaces in `gui-schema` output, re-check. Most likely no GUI ripple.
- **Manual mirror (`docs/manual/src/40-cli-reference/`):** only if user-visible CLI behavior/error text is documented there; a new reject *message* may warrant a one-line note under the relevant subcommand's error/exit-status section. Low.
- **Sibling-codec companions:** md-cli already shipped the twin (M5, cycle-9); **no sibling bump** — this is a one-directional catch-up of the toolkit to md-cli's already-shipped fix. NO-BUMP for md/mk/ms unless the spec elects to touch md-codec (it should not — `UseSitePath`'s inability to hold a fixed step is the *correct* invariant we're enforcing).

**Ordering / dependencies:** none external. Internal ordering inside the cycle: **(1) resolve Risk R1 via a pipeline trace** (gate before spec-freeze) → (2) settle D1 + D2 in the spec → (3) R0 architect loop to 0C/0I → (4) TDD impl in a worktree → (5) full-package `cargo test -p mnemonic-toolkit` + `cargo test -p wc-codec` → (6) post-impl whole-diff review → (7) release ritual (PATCH bump, both READMEs, `fuzz/Cargo.lock`, `install.sh` SELF-pin, re-vendor if any dep bump [none expected], CHANGELOG) + file/flip a new FOLLOWUP slug.

**Standing gate reminder:** cycle-prep is recon only. A spec/plan-doc for this MUST pass an **opus architect R0 review to 0 Critical / 0 Important BEFORE any code** (CLAUDE.md hard gate); persist reviews verbatim to `design/agent-reports/`.
