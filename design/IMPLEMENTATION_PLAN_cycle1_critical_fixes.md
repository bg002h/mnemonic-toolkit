# IMPLEMENTATION PLAN — cycle-1 CRITICAL fixes (H13 / H12 / H1)

**Status:** PLAN-DOC — DESIGN/PLANNING ONLY. NO production code, NO source edits in this artifact. This
plan-doc MUST pass its own opus-architect **R0 review loop to 0 Critical / 0 Important BEFORE any
implementation begins** (CLAUDE.md hard-gate), folding each review verbatim into
`design/agent-reports/`, then per-phase TDD (tests RED first), single-subagent-per-phase in a worktree,
and a mandatory whole-diff adversarial post-implementation review.

**Upstream gate already cleared:** the brainstorm-spec `design/BRAINSTORM_cycle1_critical_fixes.md` is
**R0-GREEN (0C/0I)** — `design/agent-reports/cycle1-critical-fixes-spec-R0-round2.md`. This plan-doc folds
the four round-2 MINORS (m-1..m-4) and resolves the load-bearing release/wire questions.

**Plan-doc R0 round-1 folded (this revision):** the plan-doc's own R0 round-1 review
(`design/agent-reports/cycle1-critical-fixes-plan-R0-round1.md`, 1 Critical / 1 Important) is folded — see
the §12 fold log. C-PLAN-1 widened the H1 gate to `tree == && use_site_path ==` (origins still excluded);
I-PLAN-1 corrected the false "toolkit has no second strip regex" premise. The plan-doc R0 loop continues —
this revision must be re-dispatched to the plan-doc R0 reviewer (reviewer-loop continues after every fold,
CLAUDE.md). NO implementation begins until 0C/0I.

**Source SHAs pinned + re-grep-verified at write time (CLAUDE.md "citations grep-verified at write time").
Verified against canonical bytes — NOT the working tree (this checkout is on another instance's WIP
branch):**
- toolkit `origin/master` = **`4d5872ed489e706155b0d88b02686977e59a20b6`** (`4d5872ed`)
- descriptor-mnemonic (md-codec + md-cli) `origin/main` = **`54dd765a11d490dc3d8dec2c842dae718bd3ef2b`**
  (`54dd765`). No `origin/master`; default branch = `main`. Current versions: md-codec `0.37.0`,
  md-cli `0.7.1`.

**Citation drift folded since the spec's recon:** `descriptor_intake.rs` lives at
`crates/mnemonic-toolkit/src/cmd/xpub_search/descriptor_intake.rs` (the spec/recon dropped the `cmd/`
segment — corrected throughout this plan-doc). `VerifyCheck` is defined at `src/format.rs:132` (not
`cmd/format.rs`). All other cited lines re-verified byte-present below.

---

## 0. The three findings + this plan-doc's load-bearing resolutions (one-screen index)

| ID | fix (decided in spec, GREEN) | repo(s) | phase |
|---|---|---|---|
| **H13** | capture `'`/`h` at lex → typed parse-error REJECT (never silent-collapse) | md-cli **+** toolkit (INDEPENDENT — see §4) | P1 (md-cli), P2 (toolkit) |
| **H12** | taproot-aware default-origin (`3'` for `Tag::Tr`); per-site detection; NO new flag | toolkit | P3 |
| **H1** | structural decoded-policy compare via derived `tree == && use_site_path ==` (origins excluded); keep pubkey-set check subordinate | toolkit | P4 |

**Four load-bearing plan-doc resolutions (full detail in the cited sections):**
- **§3 (the 4 round-2 MINORS):** m-1 exit codes = md-cli **1** / toolkit **2** (verified); m-2 regex = strict
  alternation; m-3 strip-class widening = **md-cli widens; toolkit pre-empted (call-order)** — the toolkit
  DOES have a second strip regex (`substitute_synthetic:319`, `[0-9;]` class), but `resolve_placeholders`'s
  `make_use_site_path` reject fires at `:768` BEFORE `substitute_synthetic` at `:779`, so the production path
  never reaches it with hardened input (plan-doc R0 round-1 fold I-PLAN-1; see §3);
  m-4 = compare **`tree == && use_site_path ==`** (origins / `path_decl` / `tlv` STILL excluded — plan-doc R0
  round-1 fold C-PLAN-1 widened this; NOT `.tree` only, NOT whole-struct).
- **§4 (H13 release-dependency VERDICT):** the toolkit H13 fix is **INDEPENDENT** of the md-cli H13 fix and
  needs NO md-codec change → **no publish-before-pin; two separate per-repo releases.** (The spec's "md-cli
  publish → toolkit pin" framing is WRONG for H13 and is corrected here, with the SemVer call adjusted.)
- **§5 (Q-WIRE):** H1 keeps the `md1_xpub_match` check NAME, changes only its `passed` predicate → **zero
  `--json` wire-shape change → no GUI paired-PR obligation** (GUI non-consumption verified).
- **§7 (branch/worktree):** two fresh worktrees off `master`; the untracked design artifacts committed onto
  the cycle-1 branch first; the other instance's `feature/own-account-subset-search` is NOT disturbed.

---

## 1. Verified code facts (re-grepped at write time, against the pinned SHAs)

### H13 — md-cli (`origin/main` @ `54dd765`)
- `crates/md-cli/src/parse/template.rs:40` — lexer regex
  `Regex::new(r"@(\d+)((?:/\d+'?)*)(?:/<([0-9;]+)>)?(/\*(?:'|h)?)?")`. Multipath body = **group 3** =
  `([0-9;]+)`; cannot match `'`/`h`. ✅
- `:62-72` — group-3 parse: `if let Some(m)=caps.get(3) { split(';') → n.parse::<u32>() → multipath_alts }`;
  a trailing `'`/`h` token would make `parse::<u32>` fail today — but the regex group can't even MATCH it, so
  the marker is silently dropped before the parse. ✅
- `:74-76` — `wildcard_hardened = caps.get(4).map(|m| ends_with('\'') || ends_with('h'))` — **the existing
  per-token hardened-detect idiom to mirror for the multipath alts.** ✅
- `make_use_site_path` (`crates/md-cli/src/parse/template.rs`, body verified) — **already returns
  `Result<UseSitePath, CliError>`**; maps each alt to `Alternative { hardened: false, value: *v }`. ✅
- `substitute_synthetic` 2nd regex — `Regex::new(r"@(\d+)((?:/\d+'?)*)(?:/<[0-9;]+>)?(?:/\*(?:'|h)?)?")` —
  **non-capturing multipath strip** `(?:/<[0-9;]+>)?`, same `[0-9;]` class. ✅ (m-3 target — md-cli only.)
- `crates/md-cli/src/error.rs:6` — `CliError::TemplateParse(String)`; Display `:28`. **No numeric exit arm.** ✅
- `crates/md-cli/src/main.rs:246-259` — `fn main() -> ExitCode`: `Ok(code)=>from(code)`,
  `Err(CliError::BadArg(_))=>from(2)`, **`Err(e)=>from(1)`** (the catch-all). So a `TemplateParse` returned
  from `main` ⇒ **exit 1**, matching `tests/exit_codes.rs::encode_bad_template_returns_1` (`.code(1)`). ✅

### H13 — toolkit (`origin/master` @ `4d5872ed`)
- `crates/mnemonic-toolkit/src/parse_descriptor.rs:69-70` — regex
  `r"@(\d+)(?:\[([0-9a-fA-F]{8})((?:/\d+(?:'|h)?)*)\])?(?:/<([0-9;]+)>)?(/\*(?:'|h)?)?"`. Multipath = **group
  4** = `([0-9;]+)`; same hardening-blind class. ✅
- `:106-121` — group-4 parse: `split(';') → parse::<u32>() → multipath_alts: Vec<u32>`. ✅
- `:122-124` — `wildcard_hardened = caps.get(5).map(ends_with('\'')||ends_with('h'))` — mirror idiom. ✅
- `:50-56` — `pub struct PlaceholderOccurrence { … multipath_alts: Vec<u32>, wildcard_hardened: bool }`. ✅
- `:167-168` — dedup/consistency check compares `prev.multipath_alts`/`prev.wildcard_hardened`. ✅
- `make_use_site_path` (`:223-237`) — returns a **plain `UseSitePath`** (NOT `Result`); maps each alt to
  `Alternative { hardened: false, value: *v }`. Called at `:193` and `:197`. **MUST widen to
  `Result<UseSitePath, ToolkitError>`** (the only structural change on the toolkit side). ✅
- **Second strip regex DOES exist (I-PLAN-1 correction):** `substitute_synthetic` (`:313`, `pub`) holds a
  SECOND multipath-relevant regex at **`:319`**:
  `r"@(\d+)(?:\[[0-9a-fA-F]{8}(?:/\d+(?:'|h)?)*\])?(?:/<[0-9;]+>)?(?:/\*(?:'|h)?)?"` — a **non-capturing
  multipath strip** with the same `[0-9;]` class. It is the **structural twin** of md-cli's
  `substitute_synthetic` strip (`template.rs:365`). (The toolkit has FOUR `Regex::new` calls: `:69`
  lexer, `:277` `tr(NUMS\b`, `:299` `^tr\(`, **`:319`** strip. The earlier "none a multipath strip" claim
  was WRONG.) **The production-path pre-emption that makes widening `:319` OPTIONAL** is the call order in
  `parse_descriptor` (`:760-779`): `lex_placeholders` (`:767`) → `resolve_placeholders` (`:768`, which calls
  the now-`Result` `make_use_site_path` → the primary `DescriptorParse` reject) → `substitute_synthetic`
  (`:779`). So a hardened multipath body is rejected at `:768` BEFORE `:319` runs — the secondary regex is
  unreachable with hardened input on the production path. `substitute_synthetic` is `pub` with direct
  callers (tests `:1441`, `:2594`) that bypass `resolve_placeholders`, so widening `:319` is a
  defense-in-depth option (see m-3 decision in §3). ✅
- `crates/mnemonic-toolkit/src/error.rs:123` `DescriptorParse(String)`; **`exit_code` arm `:539` ⇒ `2`**;
  `kind` `:601`; `Display` `:732`. **No new variant needed** (reuse `DescriptorParse`). ✅

### H12 — toolkit (`origin/master` @ `4d5872ed`)
- `crates/mnemonic-toolkit/src/cmd/bundle.rs:2210-2235` — `pub fn compute_default_origin_path(network:
  CliNetwork, account: u32) -> OriginPath`; the 4th `PathComponent { hardened: true, value: 2 }` is
  **hardcoded** (verified `:2230-2233`). Signature takes NO `Tag`/script-type. ✅
- `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:1373` — calls the SAME 2-arg helper (symmetric mirror). ✅
- `crates/mnemonic-toolkit/src/cmd/xpub_search/descriptor_intake.rs:297` — `parsed =
  MsDescriptor::<DescriptorPublicKey>::from_str(...)` (rust-miniscript, NOT an md-codec `Tag`);
  `:324` and `:345` — both `bip48_default_path(network, account, 2)` (literal `2`). `bip48_default_path`
  (`:410-418`) already takes `script_type: u32`. ✅
- `crates/mnemonic-toolkit/src/template.rs:231-235` — `bip48_script_type(&self) -> Option<u32>`:
  `ShWshMulti|ShWshSortedMulti => Some(1)`, `WshMulti|WshSortedMulti => Some(2)`,
  `TrMultiA|TrSortedMultiA => Some(3)`. The 1/2/3 mapping authority. ✅
- `crates/mnemonic-toolkit/src/template.rs:253` — `bip48_nonstandard_script_type_warning` (`3'`=toolkit
  convention, not BIP-48). Preserve. ✅
- **Proven `Tr`-detection precedent:** `crates/mnemonic-toolkit/src/wallet_import/bsms.rs:295` —
  `let is_taproot = matches!(parsed, MsDescriptor::Tr(_));` (the exact pattern §6.3 prescribes for the
  third site). ✅

### H1 — toolkit (`origin/master` @ `4d5872ed`)
- `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:2718-2740` — the md1 multiset block in
  `emit_multisig_checks`. `:2719-2730` extract `exp_pubs`/`act_pubs` as `Vec<[u8;65]>` via `.map(|(_, b)|
  *b)` (**slot index `_` discarded**); `:2731-2735` `sort()` both then `pubkeys_match = exp_sorted ==
  act_sorted` — the **sole** `md1_xpub_match` determinant. **`expected_md_decoded` and `desc` (the decoded
  `md_codec::Descriptor`s) are BOTH already in scope** at `:2719`/`:2725`. ✅
- `crates/mnemonic-toolkit/src/format.rs:132` — `pub struct VerifyCheck { name: String, passed: bool,
  detail: String, expected/actual/diff_byte_offset/decode_error: Option<…> }` (`#[derive(Serialize)]`,
  free-form `name`). ✅
- **Derived structural primitive (md-codec `origin/main` @ `54dd765`):** `encode.rs:15-28` `Descriptor`
  (5 fields: `n, path_decl, use_site_path, tree, tlv`) derives `PartialEq, Eq`; `tree.rs:8` `Node`,
  `tree.rs:17` `Body` (incl. `Body::MultiKeys { k: u8, indices: Vec<u8> }`); `tag.rs:14` `Tag` — all derive
  `PartialEq, Eq`. ✅
- **`use_site_path` is funds-relevant AND `==`-safe (C-PLAN-1 basis):** `use_site_path.rs:48-53`
  `pub struct UseSitePath { multipath: Option<Vec<Alternative>>, wildcard_hardened: bool }` derives
  `PartialEq, Eq`; `:18-23` `Alternative { hardened: bool, value: u32 }` derives `PartialEq, Eq, Copy`. The
  `multipath` carries the change-chain branches (`<0;1>` vs `<2;3>` etc.). `derive.rs::derive_address:92`
  reads `self.use_site_path.multipath` (`:110-111`): `chain` selects an `Alternative` whose `value` becomes
  the derivation step → **`use_site_path` fixes the WATCHED ADDRESS SET.** `use_site_path.rs` has **NO**
  sort/canonicalize/normalize (grep `sort|canonical|normal` = none) — a faithful, order-significant 1:1
  representation, so an `==` on it does NOT false-fail a legitimately-equal wallet (UNLIKE origins, which
  DO have elision/canonicalization → correctly excluded). `.tree` does NOT contain `use_site_path` (the
  `tree: Node` AST is operator/key structure only) → a `.tree`-only check is SILENT on the multipath. ✅
- **`use_site_path` is bound by NO other multisig-path check:** `verify_bundle.rs` `emit_multisig_checks`
  md1 block emits only `md1_decode` / `md1_wallet_policy` / `md1_xpub_match` (+ per-cosigner `mk1_*`); there
  is NO `use_site_path`/`multipath`/`derive_address` comparison in this path (the only `verify_bundle.rs`
  `use_site_path`/`derive` sites are single-sig/search-path, NOT the multisig block). The single-sig path
  DOES bind it (keyless single-sig `:645` `expected.md1 == args.md1` byte-exact; keyless-template `:882`
  `compute_wallet_descriptor_template_id`) — only the keyed-multisig path lacked the binding, which the
  widened gate restores. ✅
- `crates/md-codec/src/decode.rs:35-38` — root `Tag::Sh` covers BOTH `sh(multi)` AND `sh(wsh(multi))` (differ
  only in nested body) → a root-tag-only check would false-equate; `tree ==` distinguishes. ✅
- `design/FOLLOWUPS.md` — entry header `verify-bundle-multisig-md1-xpub-match-set-equality` at **`:1635`**,
  status line `resolved by v0.5.0 Phase B.3 …` at **`:1641`**. The stale entry to RE-OPEN. ✅

### Test harness (`crates/mnemonic-toolkit/tests/bitcoind_differential.rs`)
- `derive_receive(desc, count)` `:233`; `core_addresses(...)` `:319`; `#[ignore]`/env-gated
  `bitcoind_end_to_end_differential` `:345`; DEFAULT-CI anti-vacuity legs `divergent_differential_golden`
  `:486` and `template_completion_anti_vacuity_leg` `:791`; env-gated
  `bitcoind_template_completion_differential` `:817`. **Patterns to mirror for the new rows.** ✅

### Dependency facts (decisive for §4)
- `crates/mnemonic-toolkit/Cargo.toml:36` — `md-codec = "0.37"` (crates.io). `:44` — `miniscript =
  "13"`. **NO `md-cli` dependency anywhere in the toolkit** (verified across workspace + crate manifests). ✅
- `crates/md-cli/Cargo.toml` — `[[bin]] name="md"` (binary crate); depends on `md-codec { path=...,
  version="=0.37.0" }`. **md-cli is NOT a library; the toolkit cannot and does not depend on it.** ✅
- `parse_descriptor.rs:17-21` — toolkit imports `md_codec::{origin_path, tag::Tag, tree, use_site_path::
  {Alternative, UseSitePath}, Descriptor, TlvSection}` — **all already present in md-codec 0.37.0** (no new
  md-codec API needed for H13). ✅

---

## 2. Phase breakdown (single-subagent-per-phase TDD, ordered)

**Ordering rationale (from the spec/PLAN Tier-0 model):** H13 is a separate concurrent two-repo workstream;
H12 + H1 share the `bundle.rs`/`verify_bundle.rs` "S-VERIFY zone" and **CANNOT be two concurrent agents** —
they serialize on ONE branch. Realistic Tier-0 concurrency peak = **2 agents** (H13 branch ‖ S-VERIFY-zone
branch). Phases are numbered for clarity; P1/P2 (H13) run as one workstream-agent, P3/P4 (H12+H1) as the
other. **Each phase is TDD: the unit tests + the `bitcoind_differential.rs` row are written RED FIRST, then
a single subagent makes them GREEN.** A class-A fix (H12, H13) is "done" only when its differential-oracle
row is GREEN; H1 (class-B false-verdict) is "done" when its verify-bundle discriminator rows are GREEN.

> **Phase 0 (oracle harness) is NOT re-built here.** The program's Phase-0 differential-oracle expansion is
> its own formal workstream (PLAN §3). This cycle-1 plan **adds rows to the existing
> `bitcoind_differential.rs`** mirroring its established `#[ignore]`/env-gated + DEFAULT-CI-anti-vacuity
> pattern. If a row needs a harness helper the current file lacks, that is called out per-phase (§2.x) as a
> small in-file helper add, NOT a Phase-0 dependency.

### Phase P1 — H13 md-cli lexer-capture → typed REJECT (workstream A, repo: descriptor-mnemonic)

**Files/fns:** `crates/md-cli/src/parse/template.rs` — the lexer regex (`:40`), the group-3 parse (`:62-72`),
`make_use_site_path` (already `Result`, `:220`), and the `substitute_synthetic` 2nd regex (fn `:359`, regex
`:365`).

**Tests written FIRST (RED), in `crates/md-cli/tests/` (mirror `exit_codes.rs`/template tests):**
1. `encode_hardened_multipath_alt_rejects` — `md encode "wsh(multi(2,@0/<0';1'>/*,@1/<0';1'>/*))"` (and a
   `<0h;1h>` variant) ⇒ **exit 1** (m-1), stderr `template parse error: …` naming the hardened alt. Assert
   stdout does NOT contain a bare `md1…` (no silent-collapse emission).
2. `encode_malformed_hardened_multipath_rejects` — `<0'';1>` / stray `'h` ⇒ exit 1, typed `TemplateParse`.
3. `encode_nonhardened_multipath_roundtrips` (CLEAN-NEGATIVE) — `<0;1>` still encodes AND its decoded
   address == an independent reference for receive AND change (no over-reject).
4. Unit: `make_use_site_path` returns `Err(CliError::TemplateParse(_))` when an alt carries a hardened
   marker; still `Ok` for non-hardened.

**Change (after RED):**
- Widen group 3 to **capture** the marker — m-2: use the **strict alternation**
  `((?:\d+(?:'|h)?)(?:;\d+(?:'|h)?)*)` (so a malformed body yields the *primary* typed reject, not a generic
  catch-all). The point is to SEE the `'`/`h`, not encode it.
- In the group-3 parse loop, per token mirror the `wildcard_hardened` idiom (`ends_with('\'')||
  ends_with('h')`); if ANY alt is hardened, return `CliError::TemplateParse` naming the offending alt and
  stating hardened derivation is impossible on a watch-only (xpub) card. On the non-hardened path,
  `parse::<u32>()` as today into `multipath_alts: Vec<u32>` (no struct widening — REJECT-and-discard).
- `make_use_site_path`: keep `Alternative { hardened: false, value }` only on the now-exclusively-
  non-hardened path (the reject fires earlier).
- **m-3 (call-order pre-emption is the funds-safe basis; widen `:365` as symmetric defense-in-depth):**
  In the production `parse_template` pipeline (`:1741`), `lex_placeholders` (`:1747`) and
  `make_use_site_path` (the `Result` reject) run BEFORE `substitute_synthetic` (`:1750`) — so the primary
  `CliError::TemplateParse` reject **pre-empts** the `:365` strip regex; the strip never sees hardened input
  on the production path. The funds-safe close is therefore the lexer reject (option (a)), NOT the strip
  widening. As **symmetric defense-in-depth for direct `pub`/test callers** of `substitute_synthetic`, ALSO
  widen the `:365` non-capturing strip to `[0-9;'h]` (or the matching alternation) so a hardened body is
  matched-and-stripped rather than mis-parsed; keep the two regexes in sync (bug-hunt M5 family). This
  mirrors the toolkit decision (§3 m-3): both lexers pre-empt; both optionally widen their strip regex in
  lockstep.

**Exit criteria:** all P1 unit tests GREEN; full `cargo test -p md-cli` GREEN; clippy `-D warnings`; the H13
md-cli leg of the differential row (§2.5) GREEN.

### Phase P2 — H13 toolkit `parse_descriptor.rs` mirror → typed REJECT (workstream A, repo: toolkit)

**Files/fns:** `crates/mnemonic-toolkit/src/parse_descriptor.rs` — lexer regex (`:69`), group-4 parse
(`:106-121`), `make_use_site_path` (`:223-237`, **widen to `Result`**), call sites in
`resolve_placeholders` (`:194` `make_use_site_path(at0)`, `:198` `make_use_site_path(occ)`), and the
**SECOND strip regex** in `substitute_synthetic` (fn `:313`, regex `:319`) — see m-3 (§3).

**Tests written FIRST (RED), in `parse_descriptor.rs` `#[cfg(test)]` + a CLI-level test:**
1. Unit: `lex_placeholders` on `@0/<0';1'>/*` captures the marker; `make_use_site_path` (new `Result` sig)
   returns `Err(ToolkitError::DescriptorParse(_))` for a hardened alt; `Ok` for `<0;1>`.
2. CLI: `bundle --descriptor "wsh(multi(2,@0/<0';1'>/*,@1/<0';1'>/*))"` ⇒ **exit 2** (m-1, toolkit), typed
   `DescriptorParse`; stdout emits NO `md1…` (no silent bare-key collapse to `bcrt1qq0kxm9…`).
3. CLEAN-NEGATIVE: `bundle --descriptor` with `<0;1>` still produces a correct md1 (receive+change).

**Change (after RED):**
- Widen group 4 to capture (same strict alternation as P1, kept symmetric with md-cli).
- In the group-4 parse, detect a hardened alt (mirror `wildcard_hardened`); route to
  `ToolkitError::DescriptorParse`.
- Widen `make_use_site_path` signature to `Result<UseSitePath, ToolkitError>`; propagate at the
  `resolve_placeholders` call sites `:194`/`:198` with `?` (and `resolve_placeholders` already returns
  `Result<…, ToolkitError>`, so no signature churn there). Kill the silent `hardened: false` collapse for
  hardened input.
- **m-3 (I-PLAN-1 correction — the toolkit DOES have a second strip regex):** widen the
  `substitute_synthetic:319` non-capturing strip to `[0-9;'h]` (matching md-cli `:365`) as symmetric
  defense-in-depth for direct `pub`/test callers of `substitute_synthetic`. NOTE the production path is
  already funds-safe WITHOUT this: in `parse_descriptor` (`:760-779`) `resolve_placeholders` (`:768`, which
  calls the now-`Result` `make_use_site_path` → the primary `DescriptorParse` reject) runs BEFORE
  `substitute_synthetic` (`:779`), so the reject pre-empts `:319` on hardened input. The `:319` widen is
  optional-robustness (lockstep with md-cli `:365`), NOT the funds-safe close. (Earlier plan drafts wrongly
  asserted no toolkit strip regex existed — corrected.)
- **No md-codec change** (the wire/derive facts are spec-GREEN: md-codec already refuses hardened public
  derivation; we simply never WRITE a hardened multipath from the lexer).

**Exit criteria:** P2 unit + CLI tests GREEN; full `cargo test -p mnemonic-toolkit` GREEN; clippy
`-D warnings`; H13 toolkit leg of the differential row GREEN.

### Phase P3 — H12 taproot-aware default-origin (workstream B / S-VERIFY zone, repo: toolkit)

**Files/fns (3 call sites + info-strings):**
- `cmd/bundle.rs::compute_default_origin_path:2210` — add `script_type: u32` param; use it for the 4th
  `PathComponent.value` (was hardcoded `2`).
- `cmd/bundle.rs` caller of `compute_default_origin_path` — compute `script_type` from the md-codec `Tag`
  in scope (`canonicity_probe.tree`) via `template.rs::bip48_script_type()`-equivalent mapping
  (`Tr → 3`, wsh → `2`, sh-wsh → `1`).
- `cmd/verify_bundle.rs:1373` — pass the `Tag`-derived `script_type` to the same helper (symmetric mirror).
- `cmd/xpub_search/descriptor_intake.rs:324, 345` — replace literal `2` with a taproot-detected value:
  `let st = if matches!(parsed, MsDescriptor::Tr(_)) { 3 } else { 2 };` then
  `bip48_default_path(network, account, st)` (per-site detection — `parsed` is a miniscript `Descriptor`,
  NOT a `Tag`; precedent `bsms.rs:295`).
- Info-strings: the "defaulting origin path … to m/48'/…/2'" stderr notices render the **actual** computed
  component, not a hardcoded `2'`.
- Preserve `bip48_nonstandard_script_type_warning` for bip48-family taproot.

**Tests written FIRST (RED):**
1. Unit `compute_default_origin_path` (new sig): `…/3'` for `script_type=3`, `…/2'` for 2, `…/1'` for 1.
2. Unit/CLI: `bundle --descriptor "tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<0;1>/*))"` emits origin
   `m/48'/<coin>'/<account>'/3'`; non-taproot `wsh(sortedmulti(2,…))` still `2'`.
3. Unit: `descriptor_intake` taproot path emits `3'` via `bip48_default_path(.., 3)`; non-taproot `2'`.
4. Assert `bip48_nonstandard_script_type_warning` still fires for bip48-family taproot.

**Differential row (class-A; §2.5 row H12):** GREEN before P3 is "done."

**Exit criteria:** P3 unit + CLI tests GREEN; differential row H12 GREEN; full `cargo test -p
mnemonic-toolkit` GREEN; clippy `-D warnings`.

### Phase P4 — H1 structural decoded-policy compare (workstream B / S-VERIFY zone, repo: toolkit)

**Files/fns:** `cmd/verify_bundle.rs::emit_multisig_checks` md1 block `:2718-2740`.

**Change (after RED):**
- Replace the sorted-multiset-only `pubkeys_match` as the SOLE determinant with the derived structural
  compare: `let policy_match = expected_md_decoded.tree == desc.tree && expected_md_decoded.use_site_path ==
  desc.use_site_path;` (m-4, C-PLAN-1 fold: compare **`tree == && use_site_path ==`** — origins /
  `path_decl` / `tlv` STILL excluded; see §3). KEEP the existing sorted-pubkey-multiset check **subordinate**
  (it binds the cosigner pubkey SET; a key SUBSTITUTION is caught by the set, a key PERMUTATION in unsorted
  shapes by `tree`'s order-sensitive `indices`, Tag/k/wrapper/nesting by `tree`, and the change-chain /
  multipath that fixes the WATCHED ADDRESS SET by `use_site_path`).
- **Per-`@N` override completeness (n-3):** also bind the per-`@N` `tlv.use_site_path_overrides` map, which
  ALSO fixes the address set (it is validated against the baseline by md-codec `validate_multipath_consistency`).
  Compare the address-fixing override map alongside baseline `use_site_path`; do NOT pull in the
  origin/fingerprint TLV entries (those carry the same elision/canonicalization brittleness as `path_decl`
  → excluded). If the P4 subagent finds the override map is structurally subsumed by baseline
  `use_site_path` for the engraved-vs-supplied shapes in scope, it MAY narrow to baseline-only with an
  in-code justification; otherwise bind both.
- **Q-WIRE decision (§5): keep the check NAME `md1_xpub_match`; make `passed` reflect
  `policy_match && pubkeys_match`** (zero `--json` wire-shape churn — adding the `use_site_path` term to
  `policy_match` does NOT change the check NAME or the `checks[]` shape). Update the `detail` string to name
  the structural mismatch class (incl. a multipath/change-chain divergence). Do NOT hand-roll a
  root-tag-only wrapper check (the `sh(multi)` vs `sh(wsh(multi))` trap — `tree ==` handles it).
- Retain `extract_multisig_threshold` (`bundle.rs`) ONLY for human-readable mismatch-detail, not the verdict.

**Tests written FIRST (RED):**
1. `wsh(sortedmulti(2,A,B,C))` engraved bundle vs `sortedmulti(1,…)` ⇒ `passed:false` (wrong-k).
2. vs unsorted `multi(2,A,B,C)` ⇒ `passed:false` (Tag drift).
3. vs `sh(wsh(sortedmulti(2,…)))` ⇒ `passed:false` — **explicit `sh(multi)` vs `sh(wsh(multi))`
   discrimination assertion** (the nested-body trap).
4. Index-permuted unsorted `multi` (same multiset, different slot order) ⇒ `passed:false`
   (`Body::MultiKeys.indices` order-sensitivity).
5. **C-PLAN-1 RED test — `use_site_path`-divergent, `.tree`-equal:** engraved
   `wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))` vs supplied identical-`.tree` but
   `wsh(sortedmulti(2,@0/<2;3>/*,@1/<2;3>/*))` (change-chains `<0;1>` vs `<2;3>` → DIFFERENT watched-address
   set) ⇒ `passed:false`. This is the case a `.tree`-only gate would have GREENed (funds-safety). Add a bare
   `/*` vs `<0;1>/*` (multipath presence/count divergence) variant.
6. Genuine match ⇒ `passed:true` (no over-rejection) — identical `.tree` AND identical `use_site_path`.
7. Legitimately origin-elided-but-equal md1 ⇒ `passed:true` (no false-fail — origins remain EXCLUDED from
   the gate, so the elision brittleness guard `tree ==`/`use_site_path ==` provides over byte/policy-id
   compare; this test is NOT in tension with test 5: origins excluded, only `use_site_path` added).

**Differential anchoring (§2.5 row H1):** the wrong-wallet md1 corpus's "different addresses" premise
(including the `use_site_path`-divergent shape in test 5: `<0;1>` vs `<2;3>` derive different addresses —
anchorable by `derive_receive`, which the harness already exercises for divergent multipath groups) is
anchored by the oracle, but the H1 verdict assertions are verify-bundle-behavioral (exit≠0 / exit 0), no
Core derive needed for the mismatch assertion itself.

**Exit criteria:** P4 tests GREEN incl. the `cli_json_envelopes.rs`-style check-name/count asserts (NAME
unchanged → minimal churn); full `cargo test -p mnemonic-toolkit` GREEN; clippy `-D warnings`.

### 2.5 Differential-oracle rows (the class-A gates — mirror existing harness pattern)

Each row adds an `#[ignore]`/env-gated heavy leg (Core `deriveaddresses` per `core_addresses:319`) PLUS a
DEFAULT-CI anti-vacuity leg (`derive_receive:233` vs the toolkit's reported address), mirroring
`divergent_differential_golden:486` / `template_completion_anti_vacuity_leg:791`.

- **Row H12 (taproot `2'`-vs-`3'`):** `bundle --descriptor "tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<0;1>/*))"`
  + a `tr(sortedmulti_a)` companion. Assert (a) emitted origin = `…/3'`; (b) Core `deriveaddresses` ==
  toolkit addresses at every receive+change index; (c) descriptor-mode == template-mode at every index
  (crossmode). Anti-vacuity: toolkit addr == `derive_receive` of the `3'`-origin descriptor. **Highest-
  priority row — gates P3.**
- **Row H13 (hardened-multipath REJECTED):** `wsh(multi(2,@0/<0';1'>/*,@1/<0';1'>/*))` (and `<0h;1h>`) via
  BOTH `md encode` and `bundle --descriptor` ⇒ typed ERROR (md-cli exit **1** / toolkit exit **2**), stderr
  message class asserted, NEVER a bare-`/*` collapse to `bcrt1qq0kxm9…`. CLEAN-NEGATIVE: non-hardened
  `<0;1>` round-trips (receive AND change) == `derive_receive`. (No derived-address leg for the reject case
  — reject produces none; assert exit-code + stderr class.)
- **Row H1 (false-GREEN discriminator):** engrave `wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))`,
  `verify-bundle` against `sortedmulti(1,…)` / unsorted `multi(2,…)` / `sh(wsh(sortedmulti(2,…)))` /
  **the `use_site_path`-divergent `wsh(sortedmulti(2,@0/<2;3>/*,@1/<2;3>/*))` (C-PLAN-1 — same `.tree`,
  different change-chains → different watched-address set)** ⇒ all `result:mismatch` (exit≠0); genuine match
  (same `.tree` AND same `use_site_path`) ⇒ `result:ok` (exit 0). The `<0;1>`-vs-`<2;3>` "different
  addresses" premise is anchored by `derive_receive` (the harness already exercises divergent multipath
  groups, `bitcoind_differential.rs:62-67/116/143/488`); the verdict assertion itself is verify-bundle
  exit-code-behavioral.

---

## 3. The four round-2 MINORS — explicit resolutions

- **m-1 (exit-code precision) — RESOLVED, pin per repo.** md-cli `CliError::TemplateParse` has no numeric
  arm; `main.rs:256-258`'s catch-all `Err(e)=>ExitCode::from(1)` ⇒ **md-cli exit 1** (matches
  `exit_codes.rs::encode_bad_template_returns_1`). Toolkit `ToolkitError::DescriptorParse` ⇒ `exit_code`
  arm `error.rs:539` = **exit 2**. **Tests assert the exact code per repo** (md-cli `.code(1)`, toolkit
  `.code(2)`), NOT a loose `≠0`.
- **m-2 (regex form) — RESOLVED: strict alternation.** Use
  `((?:\d+(?:'|h)?)(?:;\d+(?:'|h)?)*)` (NOT the looser `[0-9;'h]+`) for the capture group in BOTH lexers, so
  a malformed body (`0''`, stray `'h`) produces the *primary* typed `TemplateParse`/`DescriptorParse`
  reject rather than a generic catch-all, and a per-alt hardened marker is cleanly isolable for the message.
- **m-3 (`substitute_synthetic` strip-class) — RESOLVED (I-PLAN-1 fold): BOTH lexers pre-empt at lex/resolve;
  widen BOTH strip regexes in lockstep as defense-in-depth.** The earlier plan claim "the toolkit has NO
  second strip regex" was FACTUALLY WRONG. **The toolkit DOES have a second strip regex** — `substitute_synthetic`
  (`crates/mnemonic-toolkit/src/parse_descriptor.rs:313`, `pub`), regex at **`:319`**:
  `r"@(\d+)(?:\[[0-9a-fA-F]{8}(?:/\d+(?:'|h)?)*\])?(?:/<[0-9;]+>)?(?:/\*(?:'|h)?)?"` — the structural twin of
  md-cli's `template.rs:365`. (Toolkit has FOUR `Regex::new`: `:69` lexer, `:277` `tr(NUMS\b`, `:299` `^tr\(`,
  `:319` strip.)
  **Funds-safe basis = call-order pre-emption (option (a)), symmetric in BOTH repos:** the hardened reject
  fires in the lexer/resolve step BEFORE `substitute_synthetic` runs, so the strip regex is never reached
  with hardened input on the production path —
  • toolkit `parse_descriptor` (`:760-779`): `lex_placeholders` (`:767`) → `resolve_placeholders` (`:768`,
    calls the now-`Result` `make_use_site_path` → primary `DescriptorParse` reject) → `substitute_synthetic`
    (`:779`);
  • md-cli `parse_template` (`:1741`): `lex_placeholders` (`:1747`) + `make_use_site_path` (`Result` reject)
    → `substitute_synthetic` (`:1750`).
  **Decision: widen BOTH `:319` (toolkit) and `:365` (md-cli) strip classes to `[0-9;'h]`** as defense-in-depth
  for direct `pub`/test callers of `substitute_synthetic` that bypass the lexer/resolve reject (toolkit tests
  `:1441`/`:2594`; md-cli has `pub fn substitute_synthetic` test callers) — keep the two regexes in sync
  (bug-hunt M5 family). The widen is OPTIONAL-robustness, NOT the funds-safe close (the reject is); md-cli and
  toolkit are now handled SYMMETRICALLY, eliminating the false-asymmetry the prior draft recorded.
- **m-4 (`use_site_path` add-on) — RESOLVED (C-PLAN-1 fold; the prior `.tree`-ONLY resolution was WRONG and
  is CORRECTED): compare `tree == && use_site_path ==`.** The H1 gate compares
  `expected_md_decoded.tree == desc.tree && expected_md_decoded.use_site_path == desc.use_site_path` (plus
  the per-`@N` `tlv.use_site_path_overrides` address-fixing map; n-3), and STILL EXCLUDES `path_decl` and the
  origin/fingerprint `tlv` entries.
  **Why `use_site_path` MUST be in the gate (the prior exclusion was a funds-safety hole):** `use_site_path`
  carries the change-chain / multipath alternatives (`<0;1>` vs `<2;3>`, presence/count) and drives
  `md-codec derive_address` (`derive.rs:92`/`:110` — `chain` selects an `Alternative` whose `value` is the
  derivation step) → it **fixes the WATCHED ADDRESS SET.** It is excluded from `.tree` (the `tree: Node` AST
  is operator/key structure only) and bound by NO other multisig-path check, so a `.tree`-ONLY gate would
  return `passed:true` on a `use_site_path`-divergent md1 that watches a DIFFERENT address set — the worst
  failure class for a verification tool (false assurance). The prior rationale's three props are REFUTED:
  (1) "the oracle polices derivation" — the oracle is a CI test harness, NOT a runtime `verify-bundle` check;
  it cannot police a user's actual invocation. (2) "H12 closes path-divergence" — H12 fixes the `path_decl`
  ORIGIN default (`m/48'/…/2'` vs `/3'`), an ORTHOGONAL field; it does nothing to bind `use_site_path`.
  (3) "binding `use_site_path` risks false-fails on canonically-equal-differently-encoded multipaths" —
  UNSUBSTANTIATED: `md-codec/src/use_site_path.rs` has NO sort/canonicalize/normalize (grep = none) and
  `UseSitePath`/`Alternative` derive `Eq` (`:48`/`:18`) — a faithful, order-significant 1:1, so `==` does
  NOT false-fail a legitimately-equal wallet.
  **Why origins STAY excluded (this part of the prior rationale WAS correct):** `path_decl`/origin-`tlv`
  carry elision/canonicalization (the L14 brittleness — `canonical_origin.rs`,
  `canonicalize_placeholder_indices`), so binding them WOULD false-FAIL legitimately origin-elided
  descriptor-mode bundles (the B.3 multiset change exists to avoid exactly this). `use_site_path` is NOT in
  that brittleness category. **`tree == && use_site_path ==` (origins excluded) is the funds-safe gate;
  `.tree` alone is NOT** (it misses the change-chain/multipath divergence — a different watched-address set).
  H13's reject closes hardened `use_site_path` at parse, so the residual binding concern is non-hardened
  multipath value/count/presence divergence — fully reachable, fully funds-relevant, and now bound.

---

## 4. H13 release-dependency VERDICT (load-bearing — the spec's framing is CORRECTED)

**Question:** does the toolkit's H13 fix (`parse_descriptor.rs`) have any compile/runtime dependency on the
md-cli H13 change or on md-codec — i.e. is a "md-cli MINOR publish → toolkit pin" gate actually required?

**VERDICT: NO. The two lexers are INDEPENDENT — same defect, separate code, separate crates. NO
publish-before-pin gate for H13; they land as TWO SEPARATE per-repo releases.** Evidence (verified):
1. **The toolkit does NOT depend on md-cli.** `crates/mnemonic-toolkit/Cargo.toml` lists `md-codec = "0.37"`
   and `miniscript = "13"`; there is **no `md-cli` dependency anywhere** in the workspace or crate
   manifests. md-cli is a **binary crate** (`[[bin]] name="md"`, no `lib.rs`); the toolkit cannot consume
   it as a library. The two `make_use_site_path` / lexer-regex implementations are **independent copies of
   the same defect** in two unrelated crates.
2. **The toolkit's H13 fix needs NO md-codec change.** It widens a `regex::Regex` and changes a local
   `make_use_site_path` to return `Result`, using `md_codec::use_site_path::{Alternative, UseSitePath}` and
   `ToolkitError::DescriptorParse` — **all already present in md-codec 0.37.0** (the imports at
   `parse_descriptor.rs:17-21` resolve against the current pin). The spec's §7 row "md-codec: NONE" is
   correct; this plan-doc confirms the toolkit side is self-contained.
3. **Therefore the spec/PLAN "md-cli tag+publish → toolkit pin" lockstep is FACTUALLY WRONG for H13's
   *compile* graph.** The toolkit fix compiles and ships against the EXISTING `md-codec = "0.37"` pin with
   no md-cli/md-codec bump. The genuine coupling is **behavioral parity** (the m-format mirror invariant:
   both lexers must reject the same input class so a card produced/validated by either is consistent), NOT a
   build dependency.

**Corrected release sequence (REPLACES the spec's publish-before-pin claim for H13):**
- **md-cli:** independent **MINOR** tag+publish to crates.io (`0.7.1` → `0.8.0`) — behavioral change
  (silent-collapse → typed reject) on a non-empty input class. Publishing is for crates.io consumers of
  `md`, NOT a toolkit gate.
- **toolkit:** independent release carrying the `parse_descriptor.rs` mirror (P2) + H12 (P3) + H1 (P4),
  **against the unchanged `md-codec = "0.37"` pin.** No md-cli/md-codec pin bump.
- **No serial publish edge between them.** They can release in either order or simultaneously. **The
  parity/mirror invariant is satisfied by shipping BOTH in the same cycle-1 wave (both fixes land, neither
  half is left behind) — a coordination requirement, NOT a build-ordering one.** The companion FOLLOWUP
  entries (§8) cross-cite to record the parity intent.

**SemVer impact of the correction:** the toolkit is **MINOR regardless** — H13's mirror is a behavioral
change (newly rejects a previously-silently-mis-encoded input class), which is MINOR under the
constellation convention **on its own merits**, NOT merely "because of a pin bump." (The spec's §7 attributed
the toolkit MINOR to "the pin bump forces ≥ MINOR"; with no pin bump, the toolkit MINOR is justified
directly by the H13 mirror's behavioral change — the SemVer floor is unchanged, the *reason* is corrected.)
md-cli MINOR is unchanged. **No GUI/manual leg** (no flag/dropdown/subcommand; reuses existing error
variants; error TEXT is not `schema_mirror`-gated).

> **Q-PLAN-R0-1 (for the plan-doc reviewer):** confirm the toolkit MAY ship cycle-1 against the unchanged
> `md-codec = "0.37"` pin (no bump). I find no md-codec API need; the reviewer should sanity-check there is
> no transitive reason (e.g. a `Cargo.lock` refresh that would pull a newer md-codec) to bump.

---

## 5. Q-WIRE resolution (does H1 change the `verify-bundle --json` `checks[]` shape?)

**VERDICT: NO new check id; no `--json` wire-shape change → no GUI/consumer paired-PR obligation.**
- The H1 fix **keeps the existing check NAME `md1_xpub_match`** (`verify_bundle.rs:2740`) and changes only
  its `passed` predicate (and `detail` text) to reflect the `tree == && use_site_path ==` structural verdict.
  **C-PLAN-1's widening of the predicate (adding the `use_site_path` term) does NOT change the check NAME
  or the wire-shape** — it is still a `passed`-value change on the same `md1_xpub_match` check. The `checks[]`
  array shape (`VerifyCheck { name, passed, detail, … }`, `format.rs:132`) is **unchanged** — no field
  added, no new array element, no rename. This is an **internal pass/fail change**, not an array-shape
  change. **SemVer and Q-WIRE are therefore UNAFFECTED by C-PLAN-1** (confirmed): no new flag, no wire-shape
  delta, no GUI/manual leg.
- **GUI non-consumption verified:** `mnemonic-gui` references `verify-bundle` ONLY for clap-flag
  conditional-visibility modeling (`src/form/conditional.rs:381` `verify_bundle()`, `src/runner.rs:61`
  doc-comment, `src/schema/mnemonic.rs:5` listing) — a grep for `md1_xpub_match` / `md1_policy_match` /
  `"checks"` / `checks[` across `mnemonic-gui/src` returns **nothing**. The GUI runner is a generic
  subprocess capture; it does NOT key on any `checks[]` entry. So even an APPEND-only check id would carry
  no GUI obligation — and we are not even appending one.
- **schema_mirror is irrelevant** here regardless: it gates clap **flag-NAMES** (+ dropdown value enums),
  NOT `--json` wire-shape (CLAUDE.md). No flag changes this cycle.

**Net:** H1 is the lowest-risk shape — a pure `passed`-value change on an existing check. The only in-repo
churn is same-PR test updates (e.g. `cli_json_envelopes.rs` name/count asserts), which are part of P4. **No
external paired-PR is owed to mnemonic-gui or any other consumer.** (If a future plan revision instead
appended a distinct `md1_policy_match` check, that would be an append-only in-repo wire change — still no
external gate, but it would touch the envelope test goldens; we deliberately avoid it by reusing the name.)

---

## 6. Detailed change surface per fix (consolidated)

### 6.1 H13 (P1 md-cli + P2 toolkit) — INDEPENDENT mirrors
- md-cli `parse/template.rs`: widen group-3 regex (strict alternation, m-2); per-alt hardened detect →
  `CliError::TemplateParse`; `make_use_site_path` rejects hardened (already `Result`, `:220`); widen
  `substitute_synthetic` strip class (`:365`, m-3 defense-in-depth — reject pre-empts on the production path).
- toolkit `parse_descriptor.rs`: widen group-4 regex (symmetric); per-alt hardened detect →
  `ToolkitError::DescriptorParse`; **widen `make_use_site_path` to `Result<_, ToolkitError>`** (`:223`) +
  propagate at the `resolve_placeholders` call sites `:194`/`:198`. **SECOND strip regex DOES exist**
  (`substitute_synthetic:319`, I-PLAN-1 correction) — widen its `[0-9;]` class to `[0-9;'h]` for md-cli
  symmetry (defense-in-depth for direct `pub`/test callers; the production-path reject at `:768` pre-empts
  `:319` at `:779`).
- **No md-codec change. No new error variant** (reuse `TemplateParse`/`DescriptorParse`).

### 6.2 H12 (P3, toolkit) — per-site taproot detection
- `compute_default_origin_path:2210` gains `script_type: u32`; 4th component = `script_type` not `2`.
- `bundle.rs` caller + `verify_bundle.rs:1373`: compute `script_type` from the in-scope md-codec `Tag`
  (`bip48_script_type()` mapping: `Tr→3`, wsh→`2`, sh-wsh→`1`).
- `descriptor_intake.rs:324, 345`: `matches!(parsed, MsDescriptor::Tr(_)) ? 3 : 2` → `bip48_default_path(.., st)`
  (per-site; precedent `bsms.rs:295`).
- Info-strings render the actual component; preserve `bip48_nonstandard_script_type_warning`.
- **No new flag** (value is tree-deterministic — PATCH-clean in isolation; see Q-H12-1 in §10). No new
  error variant.

### 6.3 H1 (P4, toolkit) — derived `tree == && use_site_path ==` compare (C-PLAN-1)
- `verify_bundle.rs:2718-2740`: `policy_match = expected_md_decoded.tree == desc.tree &&
  expected_md_decoded.use_site_path == desc.use_site_path` (both `Descriptor`s already decoded/in-scope;
  `UseSitePath` derives `Eq`, no canonicalization → no false-fail) PLUS the per-`@N`
  `tlv.use_site_path_overrides` address-fixing map (n-3); origins (`path_decl` + origin/fingerprint `tlv`)
  STAY excluded (L14 elision brittleness). Keep `pubkeys_match` subordinate; `passed = policy_match &&
  pubkeys_match`; NAME `md1_xpub_match` unchanged (Q-WIRE — the widened predicate does not touch the NAME or
  wire-shape). No hand-rolled wrapper check. No new error variant.

### 6.4 Designed for the later S-VERIFY dedup (no re-drift)
Implement H1's compare and H12's helper as **single shared functions** (not duplicated bundle-/verify-side
copies) so the later (out-of-scope) S-VERIFY dedup subsumes them without rewriting (spec §5.4). The
`verify_bundle.rs:1373` H12 mirror + the `emit_multisig_checks` H1 compare are in the SAME file — design
them together (one S-VERIFY-zone branch) so the verify side cannot re-drift.

---

## 7. Branch / worktree plan (per-instance branch-ownership model)

**Constraints honored (CLAUDE.md + PLAN §6.4 + the other instance's WIP):**
- The local checkout is on another instance's WIP (`feature/own-account-subset-search` /
  `feature/bundle-md1-template-multisig`). **Do NOT disturb that working tree.** Both cycle-1 worktrees are
  **fresh, off `master` (toolkit) / `main` (md-cli)** via `git worktree add` — never a `checkout` in the
  shared tree.
- A **SINGLE subagent per phase**, each in its own `isolation: "worktree"` (never parallel re-impls of the
  same bug).

**Untracked design artifacts must be committed onto the cycle-1 branch FIRST.** The brainstorm-spec, its R0
reviews, the recon, the program plan, and THIS plan-doc are currently untracked/uncommitted. They are
committed onto the cycle-1 design branch (staged explicitly, no `git add -A`) before any code branch forks,
so the audit trail exists on-branch. Candidate set (verify with `git status` at branch time — stage only
the cycle-1 design artifacts, not the other instance's untracked files):
`design/BRAINSTORM_cycle1_critical_fixes.md`,
`design/agent-reports/cycle1-critical-fixes-spec-R0-round1.md`,
`design/agent-reports/cycle1-critical-fixes-spec-R0-round2.md`,
`design/IMPLEMENTATION_PLAN_cycle1_critical_fixes.md`,
`design/PLAN_constellation_bughunt_fix_program.md`, and the cycle-1 recon doc(s).

**Branch names (two workstream branches):**
- **Workstream A (H13, two-repo):**
  - md-cli: `fix/h13-hardened-multipath-reject` off `descriptor-mnemonic` `origin/main`.
  - toolkit mirror: rides the toolkit cycle-1 branch (below) — `parse_descriptor.rs` is a different file
    from `bundle.rs`/`verify_bundle.rs`, so no content conflict with workstream B.
- **Workstream B (H12 + H1, S-VERIFY zone, serialized):** `fix/cycle1-sverify-h12-h1` off toolkit
  `origin/master`.
- **Toolkit single-release coordination:** the H13 toolkit mirror (P2) and the S-VERIFY-zone hunks (P3/P4)
  both land in ONE toolkit release. They edit disjoint files (`parse_descriptor.rs` vs
  `bundle.rs`/`verify_bundle.rs`), so they can live on one toolkit branch (`fix/cycle1-sverify-h12-h1`,
  with P2 added) or two branches merged in either order. Recommended: **one toolkit branch carrying P2+P3+P4**
  (simplest single-release path), since they never conflict.
- Stage paths explicitly. Do NOT `cargo fmt --all` / fmt `mlock.rs` (g6 exemption). The GUI is not touched.

---

## 8. Release / SemVer / lockstep matrix + version-site checklist

| repo | fix(es) | SemVer | publish? | what it drags | gate-enforced? |
|---|---|---|---|---|---|
| **md-cli** | H13 lexer-capture + reject (P1) | **MINOR** `0.7.1`→`0.8.0` | **yes (crates.io)** | behavioral (typed-reject vs silent-collapse); reuses `CliError::TemplateParse`; companion FOLLOWUP in `descriptor-mnemonic/design/FOLLOWUPS.md` | crates.io publish (for `md` consumers — NOT a toolkit gate, §4) |
| **md-codec** | NONE | — | — | — | — |
| **toolkit** | H13 mirror (P2) + H12 (P3) + H1 (P4) | **MINOR** | crates.io n/a (git-dep era) / tag | behavioral H13-mirror change (the MINOR floor, §4) + H12/H1 (PATCH-clean alone); re-opens stale B.3 FOLLOWUP; version-site ritual | pin-check CI; **NO** schema_mirror/manual (no flag) |
| **GUI** | NONE | — | — | — | schema_mirror NOT triggered (no flag); checks[] not consumed (§5) |
| **manual** | NONE | — | — | — | manual-lint NOT triggered |

**Release sequencing (CORRECTED, §4): md-cli and toolkit ship INDEPENDENTLY — no publish-before-pin gate.**
Coordinate so BOTH land in cycle-1 (parity), in either order.

**C-PLAN-1 does NOT change SemVer:** widening the H1 gate to `tree == && use_site_path ==` is still an
internal `passed`-predicate change on the existing `md1_xpub_match` check — no new flag, no `--json`
wire-shape delta (§5), no GUI/manual leg. The toolkit stays **MINOR** (driven by the H13-mirror behavioral
floor, §4); H1 remains PATCH-clean in isolation. md-cli stays MINOR `0.7.1`→`0.8.0`.

**Version-site checklist (MEMORY `project_toolkit_release_ritual_version_sites`) — toolkit:**
- [ ] toolkit crate `Cargo.toml` version → new MINOR.
- [ ] **BOTH READMEs** (the two version-bearing READMEs — silent-drift sites).
- [ ] `fuzz/Cargo.lock` (silent-drift site).
- [ ] `scripts/install.sh` self-pin.
- [ ] Re-run **full** `cargo test -p mnemonic-toolkit` + clippy `-D warnings` + **fuzz suite** AFTER the
      bump, BEFORE the tag.
- [ ] **No** `md-codec` pin bump (stays `0.37`, §4); `Cargo.lock` = `cargo check --workspace` (never
      `cargo update -w`).

**Version-site checklist — md-cli (descriptor-mnemonic):**
- [ ] md-cli `Cargo.toml` version → `0.8.0`.
- [ ] CHANGELOG entry.
- [ ] `cargo test -p md-cli` + clippy GREEN; tag + `cargo publish`.

**NOT triggered (confirmed):** GUI `schema_mirror` (no flag/dropdown/subcommand change), `docs/manual/`
flag-reference (no CLI-surface change), the `--json` wire-shape consumer obligation (§5).

---

## 9. error.rs ordering

If any phase adds a NEW `ToolkitError`/`CliError` variant, insert it **alphabetically-by-variant-name** in
the declaration + every exhaustive match (`Display`/`exit_code`/`kind`) from the first commit (CLAUDE.md
concurrent-PR-conflict rule). **Expected: NO new variant** — H13 reuses `DescriptorParse`/`TemplateParse`,
H12/H1 add no error. Verify before adding any variant.

---

## 10. FOLLOWUP actions (LIST ONLY — applied in the SHIPPING commit, NOT now)

> Contention rule: another instance is editing `design/`. FOLLOWUPS edits are deferred to the shipping
> commit (also per the followup-status-discipline rule: statuses flip in the shipping commit).

**3 new slugs to file:**
1. `h13-hardened-multipath-reject` (toolkit **+** md-cli companion, cross-citing `Companion:` lines) —
   record the REJECT decision (capture-then-typed-error), the **two INDEPENDENT lexers / no publish-before-
   pin** correction (§4), the md-codec derive-time-refusal fact, and Q-H13-1 resolved REJECT with no
   md-codec change.
2. `h12-descriptor-mode-taproot-default-origin-3prime` (toolkit) — taproot-aware default-origin helper,
   `bip48_script_type()` reuse, 3-call-site per-site detection, `3'`=de-facto (Sparrow/Coldcard/Jade)
   convention.
3. `h1-verify-bundle-structural-policy-compare` (toolkit) — derived `tree == && use_site_path ==`
   (C-PLAN-1): `tree` binds Tag+k+wrapper+index-aware slot binding+nesting (distinguishes
   `sh(multi)`/`sh(wsh(multi))`); `use_site_path` binds the change-chain/multipath that fixes the watched
   address set (+ per-`@N` `tlv.use_site_path_overrides`); `compute_wallet_policy_id` disqualified
   (origin-instability); **origins deliberately EXCLUDED** (L14 elision brittleness), but **`use_site_path`
   IS in the gate** (no canonicalization ambiguity, funds-relevant — corrected from the `.tree`-only draft).

**1 stale slug to RE-OPEN (toolkit):**
4. `verify-bundle-multisig-md1-xpub-match-set-equality` (`FOLLOWUPS.md:1635`, status `:1641`) — flip
   "resolved by v0.5.0 Phase B.3" → **re-opened/superseded by H1** (multiset-equality proven insufficient).
   Fold the downgraded multiset-index item into H1's closure.

**Companion:** H13 files a companion in `descriptor-mnemonic/design/FOLLOWUPS.md` (md-cli half) with
cross-citing `Companion:` lines. H12/H1 are toolkit-internal (no companion).

---

## 11. Per-phase exit criteria, post-impl review, open risks

### 11.1 Per-phase exit criteria (ALL must hold to close a phase / advance)
- Unit tests + the phase's differential-oracle row written **RED first**, then GREEN.
- **Full package suite** — `cargo test -p md-cli` (P1) / `cargo test -p mnemonic-toolkit` (P2/P3/P4) — NOT
  targeted `--test` targets (MEMORY `feedback_r0_review_run_full_package_suite`) — GREEN, plus clippy
  `-D warnings`.
- Class-A (H12, H13): the `bitcoind_differential.rs` row GREEN (DEFAULT-CI anti-vacuity leg everywhere;
  env-gated bitcoind leg at the integration-PR CI moment). A class-A fix does NOT merge with a RED/absent
  row.
- Per-phase reviewer-loop to 0C/0I; each review persisted verbatim to
  `design/agent-reports/cycle1-critical-fixes-phase-<P>-R<n>.md` BEFORE fold-and-commit.

### 11.2 Mandatory post-implementation adversarial review
After all phases, a **single independent adversarial review over the WHOLE cycle-1 diff** (both repos) —
catches impl-introduced regressions TDD misses (separate from R0 plan-correctness). Persist verbatim. It
MUST confirm the full `bitcoind_differential` suite is GREEN with the new H12/H13 rows + the H1
discriminator. If Agent-API dispatch fails mid-session, FLAG explicitly and DEFER the review to recovery —
never silently substitute inline self-review.

### 11.3 Open risks
1. **H13 over-rejection.** The impl must NOT reject non-hardened `<0;1>` (clean-negative legs guard both
   repos). Fail-closed direction is safe; the only risk is over-rejection.
2. **H1 false-negative brittleness vs false-GREEN completeness (RESOLVED by C-PLAN-1).** `tree ==` (not
   byte/policy-id) dodges the v0.5.0-era origin-elision false-FAILs (the §2 P4 test 7 guard — origins stay
   EXCLUDED). The `use_site_path` membership in the gate (m-4 / C-PLAN-1) is now DECIDED IN: `use_site_path`
   has no canonicalization ambiguity (no false-fail risk) and is funds-relevant (a `.tree`-only gate would
   false-GREEN a divergent-change-chain wallet, §2 P4 test 5). The residual brittleness risk is therefore
   bounded to the origin exclusion, which is correct; `use_site_path ==` adds no false-fail risk.
3. **H12 third-site detection.** The miniscript site must NOT be left at literal `2` for "no Tag"
   (§2 P3 test 3 guards). Precedent `bsms.rs:295`.
4. **Single-toolkit-release coordination.** P2 (H13 mirror) + P3/P4 (S-VERIFY) ride one toolkit release;
   they edit disjoint files (no conflict), but the release must carry ALL THREE (never half).
5. **Parity, not publish-ordering (§4).** md-cli and toolkit H13 are independent; the ONLY coupling is
   shipping both in cycle-1 (mirror invariant). Mitigation: the companion FOLLOWUP records the parity intent;
   the post-impl review confirms both halves landed.
6. **`error.rs` collisions.** None expected (no new variant); alphabetical rule if one is added (§9).

### 11.4 Open questions for the plan-doc R0 reviewer
*(All three were addressed in plan-doc R0 round-1 — `cycle1-critical-fixes-plan-R0-round1.md`: Q-PLAN-R0-1
CONFIRMED, Q-PLAN-R0-3 CONFIRMED, Q-PLAN-R0-2 REFUTED → folded as C-PLAN-1. Retained here for the round-2
reviewer to re-confirm post-fold; see the §12 fold log.)*
- **Q-PLAN-R0-1 (§4) — round-1 CONFIRMED:** the toolkit may ship cycle-1 against the unchanged
  `md-codec = "0.37"` pin (no bump); no md-codec API need, no transitive `Cargo.lock` reason to bump.
- **Q-PLAN-R0-2 (m-4 / §3) — round-1 REFUTED → folded as C-PLAN-1:** `.tree`-only is NOT the funds-safe
  minimal H1 gate (it false-GREENs a `use_site_path`-divergent / different-watched-address wallet); the gate
  is now `tree == && use_site_path ==` (origins excluded). Round-2 reviewer: re-confirm the widened gate is
  funds-safe and false-fail-free.
- **Q-PLAN-R0-3 (Q-H12-1, carried from spec §4.4) — round-1 CONFIRMED:** NO legitimate use-case wants a
  taproot descriptor's cosigners defaulted to `2'` → no escape-hatch flag (a flag would flip H12 to
  MINOR-with-GUI-schema+manual lockstep). Always emit `3'` for `Tag::Tr`.

---

## 12. Plan-doc R0 round-1 fold log

Source review (persisted verbatim): `design/agent-reports/cycle1-critical-fixes-plan-R0-round1.md` — VERDICT
**NOT-GREEN, 1 Critical / 1 Important**. Both folded in this revision:

- **C-PLAN-1 (Critical) — `.tree`-only H1 gate false-GREENs a `use_site_path`-divergent wallet — RESOLVED
  by widening the gate.** The review showed `use_site_path` (the change-chain / multipath) drives
  `md-codec derive_address` (`derive.rs:92`/`:110-111`) and therefore fixes the WATCHED ADDRESS SET; it is
  excluded from `.tree` and bound by NO other multisig-path check, so `.tree`-only would return `passed:true`
  on a different-address wallet (false assurance). It has NO canonicalization ambiguity (`use_site_path.rs`
  has no sort/canonicalize/normalize; `UseSitePath`/`Alternative` derive `Eq` at `:48`/`:18`) so `==` does
  NOT false-fail. **Fold:** H1 gate is now `expected_md_decoded.tree == desc.tree &&
  expected_md_decoded.use_site_path == desc.use_site_path` (+ per-`@N` `tlv.use_site_path_overrides`, n-3),
  with `path_decl`/origin-`tlv` STILL excluded (L14 elision brittleness — that exclusion was correct).
  Updated: §0 index + four-resolutions summary, §1 H1 facts (added the `use_site_path` funds-relevance +
  `Eq`/no-canonicalization + no-other-check facts), §2.5 row H1 (added the `<0;1>`-vs-`<2;3>` divergent
  shape), §3 m-4 (rewritten — the prior `.tree`-only resolution corrected, each refuted prop recorded), §6.3,
  §10 slug 3, §11.3 risk 2. **RED test added (P4 test 5):** engraved
  `wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))` vs supplied identical-`.tree` but `<2;3>` change-chains (and a
  bare-`/*` vs `<0;1>/*` variant) ⇒ `passed:false`; genuine-match (test 6) and origin-elided-but-equal
  (test 7, origins excluded) stay `passed:true`. Corresponding `bitcoind_differential.rs` H1 shape added,
  anchored by `derive_receive` (harness already exercises divergent multipath groups). **Does NOT change
  SemVer or Q-WIRE** — check NAME is still `md1_xpub_match`, only its predicate widens (§5, §8 note).

- **I-PLAN-1 (Important) — false "toolkit has no second strip regex" premise — RESOLVED by correcting the
  premise + a symmetric decision.** The toolkit DOES have a second strip regex:
  `parse_descriptor.rs::substitute_synthetic` (fn `:313`, regex `:319`, `[0-9;]` class) — the structural
  twin of md-cli `parse/template.rs:365`. **Decision = option (a) pre-empted + symmetric defense-in-depth
  widen.** Call-order evidence (verified on canonical source): in the production `parse_descriptor` pipeline
  (`:760-779`), `lex_placeholders` (`:767`) → `resolve_placeholders` (`:768`, which calls the now-`Result`
  `make_use_site_path` → the primary `DescriptorParse` reject) runs BEFORE `substitute_synthetic` (`:779`),
  so the hardened reject pre-empts the `:319` strip — it is never reached with hardened input on the
  production path (the funds-safe close is the reject, NOT the strip). md-cli has the SAME ordering
  (`parse_template:1741` → `lex_placeholders:1747` + `Result` `make_use_site_path` → `substitute_synthetic:1750`).
  For symmetry/defense-in-depth (direct `pub`/test callers of `substitute_synthetic` that bypass the
  lexer/resolve reject — toolkit `:1441`/`:2594`), **BOTH** strip classes (toolkit `:319` + md-cli `:365`)
  are widened to `[0-9;'h]` in lockstep — md-cli and toolkit are now handled SYMMETRICALLY (the prior
  false-asymmetry is removed). Updated: §0 four-resolutions summary, §1 H13-toolkit facts, §2 P1 + P2 (files
  + change bullets), §3 m-3 (rewritten), §6.1.

**Internal consistency confirmed:** the H1 compare reads `tree == && use_site_path ==` (origins excluded)
uniformly across §0/§1/§2.5/§3/§6.3/§10/§11; the m-3 resolution reads "both pre-empted, both widen for
symmetry" uniformly across §0/§1/§2/§3/§6.1; SemVer (toolkit MINOR / md-cli MINOR), Q-WIRE (NAME unchanged),
the version-site checklist, the FOLLOWUP list, and per-phase/exit-criteria are all consistent with the folds.
The plan-doc R0 loop continues — re-dispatch this revision (per CLAUDE.md, the reviewer-loop continues after
every fold). NO implementation until 0C/0I.
