# BRAINSTORM — cycle-1 CRITICAL fixes (H13 / H12 / H1)

**Status:** DESIGN DOC ONLY — no code, no source edits. This is the formal brainstorm-spec for
"cycle-1" of the constellation bug-fix program: the **3 differential-oracle-escalated CRITICAL
findings** (H13, H12, H1). It MUST pass an opus-architect **R0 review loop to 0 Critical / 0 Important
BEFORE any implementation begins** (CLAUDE.md hard-gate), then a plan-doc R0 loop, then per-phase TDD,
then a mandatory whole-diff adversarial post-impl review.

**Source SHAs pinned at write time (re-grep-verified per CLAUDE.md "citations grep-verified at write
time"):**
- toolkit `origin/master` = **`4d5872ed489e706155b0d88b02686977e59a20b6`** (`4d5872ed`)
- descriptor-mnemonic (md-codec + md-cli) `origin/main` = **`54dd765a11d490dc3d8dec2c842dae718bd3ef2b`**
  (`54dd765`). **NB:** descriptor-mnemonic has **NO `origin/master`** — default branch is `origin/main`.
  Current versions: md-codec `0.37.0`, md-cli `0.7.1`.

**Working-tree caveat:** the local toolkit checkout is on another instance's WIP branch
(`feature/own-account-subset-search` / `feature/bundle-md1-template-multisig`) — NOT trusted. **Every
citation below was verified against `origin/master` / `origin/main` bytes**, not the working tree.

**Authoritative inputs folded into this spec:**
- cycle-prep recon — `cycle-prep-recon-h12-h1-h13-critical.md` (verified citations + repro verdicts + SHAs)
- bug-hunt report — `design/agent-reports/constellation-bughunt-2026-06-20.md` (full finding detail +
  the EMPIRICAL differential-oracle section with diverging regtest addresses)
- fix program plan — `design/PLAN_constellation_bughunt_fix_program.md` (Tier 0 = these 3; the S-VERIFY
  serialization model; batch concurrency schedule)

**All three findings STILL REPRODUCE on current canonical source.** The recent P3a (`c0f74994`) and P4
(`aaa67b74`) commits added a *parallel* keyless-template completion arm and did **not** touch any of the
three defective code paths.

---

## 0. Executive summary — the 3 findings + the 3 design decisions

| ID | one-line | repo(s) | class | decision made (see §) |
|---|---|---|---|---|
| **H13** | hardened multipath `<0';1'>`/`<0h;1h>` silently dropped at template lex → bare `/*` single-path key → wrong addresses | md-cli **+** toolkit (lockstep) | B→A (silent policy-collapse → wrong-address) | **REJECT** (R0 round-1 C1 flip) — capture the `'`/`h` marker at lex (so it's no longer silently invisible), then return a **typed parse error**. md1 cosigner keys are xpubs; BIP-32 forbids hardened derivation from a public key, so the constellation (md-codec / md / `restore --md1` / rust-miniscript) **cannot derive** a hardened-multipath card — faithful-encode would manufacture a permanently un-restorable backup. Fail-closed, never silent-collapse. (§3) |
| **H12** | descriptor-mode taproot multisig defaults cosigner BIP-48 origin to `2'` (P2WSH) not `3'` (P2TR) → every address diverges; coins un-cosignable | toolkit | A (wrong-address) | **taproot-aware default-origin helper** emits `3'` for `Tag::Tr`, reusing `template.rs::bip48_script_type()`; **NO new flag** → PATCH-clean. `3'`=P2TR documented as a de-facto interop convention. (§4) |
| **H1** | `verify-bundle` GREEN-lights a wrong-threshold/unsorted/script-type md1 (multiset-only compare) → false verdict | toolkit | B (false assurance) | **structural compare of the decoded policy** — reuse the derived **`tree ==` equality** (`md_codec::Node`/`Body`/`Tag` derive `PartialEq/Eq`), which covers Tag + threshold k + wrapper + index-aware slot binding + nesting in ONE compare and correctly distinguishes `sh(multi)` from `sh(wsh(multi))` (R0 round-1 I1). NOT byte-exact md1 string equality (origin-elision brittleness) and NOT `compute_wallet_policy_id` (origin-significant — disqualified). (§5) |

**Headline SemVer / lockstep call (§7):**
- **md-cli MINOR** (`0.7.1`→`0.8.0`) tag+publish to crates.io — H13 now *errors* (typed parse error) on a
  class of input (hardened multipath `<0';1'>`/`<0h;1h>`) it previously silently mis-encoded. This
  tightens validation of previously-accepted-but-broken input — a bugfix in spirit, but it changes
  observable behavior for a non-empty input class (silent-collapse → typed error), which is MINOR under
  the constellation SemVer convention, NOT a bare PATCH. No breaking public API (md-cli is a binary crate
  — M-b). The new error reuses the existing `CliError::TemplateParse` (no new variant); the error TEXT is
  CLI stderr surface but is NOT a clap flag/dropdown/subcommand — so it does **not** drag `schema_mirror`
  (flag-NAMES only, per CLAUDE.md) and does **not** touch the manual's flag-reference. It is therefore a
  2-repo md-cli↔toolkit lockstep with NO GUI/manual leg.
- **toolkit MINOR** (one release) — carries the H13 `parse_descriptor.rs` mirror (rides the md-cli pin
  bump) **and** the H12 + H1 fixes. The pin bump alone forces ≥ MINOR; H12/H1 are PATCH-clean in
  isolation (no new flag/CLI surface).
- **GUI schema_mirror + manual:** **NOT triggered** — none of the three fixes adds/renames a clap
  flag, dropdown value, or subcommand. (All three are behavioral: a regex class, a default-origin
  value, and a compare predicate.) If R0 directs adding any escape-hatch flag (explicitly recommended
  AGAINST in §4 / §5), the call flips to MINOR-with-GUI-and-manual-lockstep.

**Serialization (from the PLAN's Tier-0 / Batch 0.5 model):** H12 and H1 both live in the
`bundle.rs`/`verify_bundle.rs` "S-VERIFY zone" and **CANNOT be two concurrent agents** — they serialize
onto **one** branch. H13 is a **separate concurrent two-repo-lockstep** workstream. Tier-0 concurrency
peak = **2 agents** (§8).

---

## 1. Scope, non-scope, and the funds-safety thesis

**In scope (cycle-1):** H13, H12, H1 — the three findings the differential-oracle wave escalated to
CRITICAL with empirical proof (real diverging regtest addresses / a false GREEN verdict). Each is a
live wrong-address-or-false-verdict funds-safety defect on current `origin/master` / `origin/main`.

**Explicitly OUT of scope (deferred to later program tiers — see PLAN §2):** the structural cluster
fixes S-NET / S-TEMPLATE, the remaining HIGHs (H10, H7, H6/M4, M6), every MEDIUM/LOW, and the
secret-hygiene cluster. This spec does NOT pre-empt the broader S-VERIFY dedup — it lands only the
**Tier-0 anchor hunks** (H1's compare + H12's helper) on the S-VERIFY branch; the broader bundle↔verify
dedup (L24, H7-lexer, multiset) completes in a later batch. **One caveat we DO honor now:** H1's compare
and H12's helper are designed so the later dedup can subsume them without re-drift (§5.4, §8).

**Funds-safety thesis (why all three are CRITICAL, not cosmetic):** each one breaks the steel-backup
invariant that *the engraved card reconstructs exactly the wallet the user intended, and the
verification tool confirms it.* **H12 silently changes which addresses a restore derives** (the user
funds an address no participant can spend from). **H13 (pre-fix) silently collapses a hardened multipath
to a bare-`/*` single-path key → wrong addresses; the fix REJECTS the input** (the card is genuinely
un-restorable — xpub keys cannot do hardened derivation — so failing closed at encode is the safe
outcome, vs. silently emitting a wrong-address or permanently-undecodable card). **H1** is the safety net
that should catch H12/H10/H13 structural drift and currently catches **none** of it. Empirical proof is
in the bug-hunt report's "Differential-oracle wave — EMPIRICAL results" section (diverging `bcrt1p…`
addresses for H12; `sortedmulti(1)`/unsorted-`multi(2)`/`sh(wsh(…))` all GREEN-lit for H1;
`bcrt1qq0kxm9…` bare-key collapse vs `bcrt1q5tgwjk…` intended for H13 — now fixed by REJECT, not
faithful-encode).

---

## 2. Verified citations (against the pinned SHAs)

All re-grepped at write time. **Drift notes** flag where the recon's snapshot line moved.

### H13 — md-cli (`origin/main` @ `54dd765`)
- `crates/md-cli/src/parse/template.rs:40` — lexer regex
  `Regex::new(r"@(\d+)((?:/\d+'?)*)(?:/<([0-9;]+)>)?(/\*(?:'|h)?)?")`. **Multipath body = capture group 3
  = `([0-9;]+)` — cannot match `'` or `h`.** ✅ byte-exact.
- `crates/md-cli/src/parse/template.rs:62-72` — group-3 parse: `split(';')` then `n.parse::<u32>()` into
  `multipath_alts: Vec<u32>` (no per-alt hardened capture). ✅
- `crates/md-cli/src/parse/template.rs:24-28` — `struct PlaceholderOccurrence { … multipath_alts:
  Vec<u32>, wildcard_hardened: bool }` — **the alt vector is bare `u32`; only the trailing wildcard
  carries a hardened flag.** ✅
- `crates/md-cli/src/parse/template.rs:220-233` — `make_use_site_path` maps each alt to `Alternative {
  hardened: false, value: *v }` (**literal `false` at `:225`**). Already returns `Result<_, CliError>`
  (an error path exists). ✅
- `crates/md-cli/src/parse/template.rs:357-381` — `substitute_synthetic` second regex
  `r"@(\d+)((?:/\d+'?)*)(?:/<[0-9;]+>)?(?:/\*(?:'|h)?)?"`. The multipath group here is a
  **non-capturing strip** (`(?:/<[0-9;]+>)?`) — same `[0-9;]` class; it discards the multipath from the
  substituted descriptor entirely (the use-site path is reconstructed separately by `make_use_site_path`).
  ✅ (Recon cited `:365`; the strip is on that line inside this fn.)

### H13 — md1 WIRE + DERIVE FACTS (decisive for the represent-vs-reject decision → **REJECT**)
The wire *can* serialize a hardened bit, BUT the constellation can never *derive* a hardened-multipath
xpub card — so the wire bit is **moot for this xpub path**, and faithful-encode is funds-unsafe (R0 C1).

**Wire-can-carry-it (necessary but NOT sufficient):**
- `crates/md-codec/src/use_site_path.rs:19-23` — `pub struct Alternative { pub hardened: bool, … pub
  value: u32 }`; `:26-38` `Alternative::write`/`read` serialize one hardened bit per alternative; header
  doc-comment `:9-11` documents `alternative: [hardened: 1 bit][value: LP4-ext varint]`. ✅

**…but the keys are xpubs and hardened public derivation is impossible (the decisive facts):**
- `crates/md-codec/src/tlv.rs:14` — the Pubkeys TLV (`TLV_PUBKEYS`) stores "chain-code || compressed
  pubkey, 65 bytes each" — **xpubs / watch-only, no private keys.** ✅
- `crates/md-codec/src/derive.rs:105-107` — `derive_address`'s **first** pre-flight: `if
  crate::to_miniscript::has_hardened_use_site(self) { return Err(Error::HardenedPublicDerivation); }`
  (doc `:98-104`: refuses a hardened wildcard OR any hardened multipath alternative). md-codec
  **UNCONDITIONALLY REFUSES** a hardened use-site alternative. ✅
- `crates/md-codec/src/to_miniscript.rs:101-108` — `use_site_is_hardened` returns true if **any**
  multipath alternative has `a.hardened` → a `<0';1'>` card trips the refusal. ✅
- **BIP-32 / BIP-389:** hardened derivation requires private keys; a public key cannot perform it.
  BIP-389 confirms `xpub/<0h;1h>` is "technically invalid since public keys cannot perform hardened
  derivation (per BIP 32 rules)." Bitcoin Core rejects it ("not a valid uint32") for the same reason. ✅
- **End-to-end refusal across the whole constellation** (confirmed in R0 round-1):
  `md address` → `HardenedPublicDerivation`; toolkit `restore --md1` pre-check at `restore.rs:2779-2784`
  → `ToolkitError::ModeViolation` ("watch-only addresses cannot be derived … Faithful reconstruction is
  not supported"); rust-miniscript (pinned `rev 95fdd1c5`) errors at `derive_at_index` ("key with
  hardened derivation steps cannot be a DerivedDescriptorKey"). The toolkit already warns at *engrave*
  time (`unrestorable_advisory.rs`, same `has_hardened_use_site` predicate) that such a card is
  unrestorable. ✅
- **CONCLUSION:** faithfully encoding the hardened bit on an xpub card manufactures a permanently
  un-restorable (funds-unsafe) artifact the constellation *already classifies* as unrestorable. The
  `Alternative.hardened` wire bit is **moot for this path** — the safe behavior is to **REJECT** at lex,
  loudly and typed, never silently collapse to bare `/*`. (Verified against md-codec/toolkit source, not
  the draft.)

### H13 — toolkit mirror (`origin/master` @ `4d5872ed`)
- `crates/mnemonic-toolkit/src/parse_descriptor.rs:70` — regex
  `r"@(\d+)(?:\[([0-9a-fA-F]{8})((?:/\d+(?:'|h)?)*)\])?(?:/<([0-9;]+)>)?(/\*(?:'|h)?)?"`. **Multipath
  group 4 = `([0-9;]+)` — same hardening-blind class.** ✅
- `crates/mnemonic-toolkit/src/parse_descriptor.rs:223-231` — `make_use_site_path` → `Alternative {
  hardened: false, value: *v }` (**literal `false` at `:228`**). ✅

### H12 — toolkit (`origin/master` @ `4d5872ed`)
- `crates/mnemonic-toolkit/src/cmd/bundle.rs:2210` — `pub fn compute_default_origin_path(network:
  CliNetwork, account: u32) -> OriginPath`. Body builds a 4-component `m/48'/<coin>'/<account>'/2'`;
  **the 4th `PathComponent { hardened: true, value: 2 }` is hardcoded** (verified at `:2228-2231`). The
  signature takes **NO** descriptor / `Tag` / script-type argument — it *structurally cannot*
  distinguish P2TR from P2WSH. ✅
- `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:1373` — calls
  `crate::cmd::bundle::compute_default_origin_path(args.network, args.account)` (same 2-arg, same `2'`
  defect — the symmetric verify mirror reproduces the bug). ✅
- `crates/mnemonic-toolkit/src/cmd/xpub_search/descriptor_intake.rs:324, 345` — both call
  `bip48_default_path(network, account, 2)` with a **literal `2`**. The helper `bip48_default_path`
  (`:410-418`) already takes a `script_type: u32` param and forwards to
  `MultisigPathFamily::Bip48.default_origin_path(...)` — so the fix here is to thread the real
  script-type to these two call sites. ✅ (Recon cited `:324, :345`; verified.)
- `crates/mnemonic-toolkit/src/template.rs:231-237` — `pub fn bip48_script_type(&self) -> Option<u32>`:
  `ShWsh* => Some(1)`, `Wsh* => Some(2)`, `TrMultiA | TrSortedMultiA => Some(3)`, `_ => None`. **This is
  the single source of the 1/2/3 mapping and template-mode already uses it correctly.** ✅
- `crates/mnemonic-toolkit/src/template.rs:243-262` — `bip48_nonstandard_script_type_warning` documents
  `3'`=taproot as **a toolkit convention, NOT part of BIP-48** (resolves FOLLOWUP
  `multisig-tr-bip48-script-type-3-policy`: bless + warn). **Preserve this advisory semantics.** ✅

### H1 — toolkit (`origin/master` @ `4d5872ed`)
- `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:2718-2735` — the md1 multiset block inside
  `emit_multisig_checks`. Comment at `:2718`: "`md1_xpub_match (B.3: SPEC §5.7 multiset semantics,
  sort-then-compare)`." `:2719-2730` extract `exp_pubs`/`act_pubs` as `Vec<[u8;65]>` via
  `.map(|(_, b)| *b)` (**discarding the slot index** `_`); `:2731-2734` `exp_sorted.sort();
  act_sorted.sort();`; `:2735` `let pubkeys_match = exp_sorted == act_sorted;` → the **sole** determinant
  of `md1_xpub_match`. **No threshold / `Tag` / wrapper / order compared anywhere in the arm.** ✅
  (Recon cited the fn as drifted +312 to `:2283`; the multiset block confirmed at `:2718-2735`.)
- `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:645` — the keyless single-sig fix-model:
  `let md1_match = expected.md1 == args.md1;` (full-md1-string direct compare). ✅ (Recon: the model
  drifted from the report's `:583` to `:645`; the claim is correct, the line moved.)
- `design/FOLLOWUPS.md:1635` — entry `verify-bundle-multisig-md1-xpub-match-set-equality`, **status line
  at `:1641`** = "`resolved by v0.5.0 Phase B.3 (commit 9f1a4e7) — sort-then-compare multiset equality`."
  **This is the stale entry to RE-OPEN** (the very change H1 flags as insufficient). ✅

**Citation drift summary:** all defective code is byte-present on canonical source. Two fix-model lines
moved (H1's single-sig compare `:583`→`:645`; H1's fn `:2283`, block `:2718-2735`); the recon's "wrong
repo / wrong file" note for `bip48_script_type` is resolved here — it lives in the **toolkit**
(`template.rs:231`), confirmed in-repo (no sibling-codec dep for H12).

---

## 3. H13 — DESIGN DECISION: REJECT a hardened multipath alternative (capture the marker, return a typed parse error; never silent-collapse)

### 3.1 The question
The recon framed it precisely: **does the md1 wire's `Alternative` carry a per-alternative `hardened`
bit?** If YES → could we represent faithfully? If NO (or unusable) → the safe behavior is to ERROR (Core
itself rejects `<0';1'>` as "not a valid uint32"); never silently collapse. R0 round-1 (C1) settled this:
the wire *can* carry the bit, but the keys are xpubs and the constellation can never *derive* such a
card — so the answer is **REJECT**.

### 3.2 The verified facts → the decision (REJECT)
The wire-carries-the-bit fact is **necessary but NOT sufficient**. `md-codec/src/use_site_path.rs:19-23`
does define `Alternative { hardened: bool, value: u32 }` with a serialized hardened bit (`:26-38`,
doc `:9-11`). But md1 cosigner keys are **xpubs / watch-only** (`md-codec/src/tlv.rs:14` — Pubkeys TLV =
"chain-code || compressed pubkey, 65 bytes"; no private keys), and **hardened derivation from a public
key is cryptographically impossible** (BIP-32; BIP-389 confirms `xpub/<0h;1h>` is "technically invalid
since public keys cannot perform hardened derivation"). Consequently `md-codec/src/derive.rs:105-107`
**unconditionally REFUSES** a hardened use-site alternative as its first pre-flight (`→
Error::HardenedPublicDerivation`), and the whole constellation rejects it end-to-end (confirmed R0
round-1): `md address`, toolkit `restore --md1` (`restore.rs:2779-2784` `ModeViolation`: "Faithful
reconstruction is not supported"), and rust-miniscript (`"key with hardened derivation steps cannot be a
DerivedDescriptorKey"`). The toolkit *already* warns at engrave time that such a card is unrestorable
(`unrestorable_advisory.rs`, same `has_hardened_use_site` predicate).

**DECISION: REJECT.** Faithfully encoding the bit would manufacture a steel card the constellation
*already knows* is permanently un-restorable — funds-unsafe (the user funds an address no participant,
including this very toolkit, can derive). The `Alternative.hardened` wire bit is therefore **moot for
this xpub path**. The correct behavior is: extend both lexers to **capture** the `'`/`h` marker (so the
hardening is no longer silently invisible), then **return a typed parse error** — `CliError::TemplateParse`
in md-cli, `ToolkitError::DescriptorParse` in toolkit (both exist; no new variant, no alphabetical-
ordering work). **Key correctness property: fail-closed, loud, typed — NEVER silently collapse to bare
`/*`.** This is strictly safer than the current silent collapse (which mis-encodes to a wrong-address
single-path key) AND than faithful-encode (which stores a permanently-undecodable artifact). It matches
Core, md-codec, BIP-32, and the bug-hunt's empirical recommendation verbatim ("the correct behavior is
to ERROR, not collapse").

### 3.3 Scope of the reject (well-formed hardened, malformed, and out-of-range)
The typed-error reject covers, with a clear distinct message per class:
- **(a) A well-formed hardened multipath alternative** `<0';1'>` / `<0h;1h>` — the primary case. Capture
  the marker, then reject with a message naming the hardened alt and stating that hardened derivation is
  impossible on an xpub (watch-only) card. This is the C1 flip: it was previously the "faithful happy
  path"; it is now a **typed reject**.
- **(b) Malformed hardened markers** (a stray `'h`, double-hardening `0''`, or a non-numeric body) — also
  a typed parse error naming the offending alt.
- **(c) Value out of the wire's encodable range** (`Alternative.value` is an LP4-ext varint capping below
  `2^31-1`, cf. bug-hunt L16) — must error, not silently truncate. (Pre-existing wire constraint.)
- **NOT a reject:** a non-hardened multipath `<0;1>` — that is wire-correct and derivable, and MUST
  continue to round-trip to the intended addresses (do NOT over-reject; the differential oracle proved
  `<0;1>` is fine).

**Core-parity (recon framing reconciled, now aligned with the decision):** Bitcoin Core rejects `<0';1'>`
("not a valid uint32") for exactly the BIP-32 reason above — a watch-only descriptor cannot perform a
hardened derivation step. The m-format md1 wire *physically* has a hardened bit, but there is **no
downstream tool** in the constellation that derives it — every derive/restore/address boundary is here,
and they all refuse. So reject is correct AND matches Core; the "never silently collapse" invariant is
honored by failing closed at lex.

> **R0 OPEN QUESTION (Q-H13-1) — RESOLVED → REJECT (R0 round-1 C1).** md-codec `derive_address` does NOT
> honor `Alternative.hardened` for derivation — it **refuses** it (`derive.rs:105-107` →
> `Error::HardenedPublicDerivation`), as do `md address`, toolkit `restore --md1` (`restore.rs:2779`), and
> rust-miniscript. This is correct BIP-32 behavior on xpub keys, **not** a gap — so there is **no md-codec
> change and no companion md-codec FOLLOWUP** for a derive gap. H13 = capture-then-typed-error (fail-closed).

### 3.4 Concrete change surface (both repos, lockstep) — capture-then-REJECT
1. **md-cli `parse/template.rs`:**
   - Widen the lexer regex group 3 multipath body so the hardened marker is **seen** (e.g. `([0-9;'h]+)`,
     or the stricter `((?:\d+(?:'|h)?)(?:;\d+(?:'|h)?)*)`). **R0/plan-doc to pick the exact form.** The
     point is to **capture** the `'`/`h` so it is no longer silently invisible to the lexer — NOT to
     encode it.
   - Detect a hardened alternative during the `split(';')` parse (`:62-72`) (a trailing `'`/`h` per
     token, mirroring the existing `wildcard_hardened` logic at `:74-76`) and route it to the typed
     `CliError::TemplateParse` reject. `PlaceholderOccurrence.multipath_alts: Vec<u32>` may not even need
     widening under REJECT — capture the marker, reject, and discard (the simplest shape; plan-doc to
     confirm whether any downstream consumer needs the per-alt flag retained, e.g. for a clearer message).
   - `make_use_site_path` (`:220-233`): it already returns `Result`; on a hardened alt return the typed
     error instead of building `Alternative { hardened: false, value }`. Keep the literal `false` only on
     the genuinely non-hardened path (which is now the only path that reaches construction).
   - `substitute_synthetic` second regex (`:365`): widen the non-capturing multipath strip class to
     `[0-9;'h]` so a hardened body is **matched-and-stripped** rather than leaving a residual `'>` that
     produces a confusing *secondary* parse error before the primary reject fires (M-a; keep the two
     regexes in sync — same root-cause family as bug-hunt M5). Cosmetic robustness; fold into the plan-doc.
   - Update the dedup/consistency check (`:175-176`) only if the occurrence type changes (under the
     capture-then-reject-and-discard shape it likely does not).
2. **toolkit `parse_descriptor.rs`:** the EXACT mirror — widen group 4 so the marker is seen, detect a
   hardened alternative, and return `ToolkitError::DescriptorParse`. `make_use_site_path` currently
   returns a **plain `UseSitePath`** (`parse_descriptor.rs:223`; called at `:193`, `:197`) — it must be
   **widened to `Result<UseSitePath, ToolkitError>`** to surface the typed reject (the only structural
   change on the toolkit side). Kill the silent `hardened: false` collapse for the hardened-input case.
3. **NO md-codec change, NO md-codec FOLLOWUP** — the wire/struct/serializer already carry the bit, and
   md-codec's derive-time refusal is correct BIP-32 behavior (not a bug). We simply never *write* a
   hardened multipath from the lexer; md-codec's refusal is the backstop. (The previous draft's "if
   `derive_address` doesn't honor it → separate md-codec workstream" conditional is **struck** — md-codec
   is already correct; Q-H13-1 resolved.)

### 3.5 Lockstep & ordering (H13)
**Two-repo lockstep** (m-format mirror invariant). The md-cli fix is a behavioral correctness change
(it now **rejects** a previously-silently-mis-encoded input with a typed error) → **md-cli MINOR tag
+ publish to crates.io FIRST**, then the **toolkit pins the new md-cli** and ships its `parse_descriptor.rs`
mirror **in the same toolkit release** (never one half). Companion `FOLLOWUPS.md` entries in BOTH repos
with cross-citing `Companion:` lines (§9). This is the **earliest cross-repo publish gate in the
program** (PLAN §6.1) — schedule the md-cli tag first. NO GUI/manual leg (no flag/dropdown/subcommand
change; the new error reuses `CliError::TemplateParse`/`ToolkitError::DescriptorParse`; error-TEXT is not
gated by `schema_mirror`, which covers flag-NAMES only).

**Why MINOR not PATCH for md-cli:** the change tightens validation of previously-accepted-but-broken
input — a bugfix in spirit, but it alters observable behavior for a non-empty input class (hardened
multipath went from silent-mis-encode → **typed reject**). Per the PLAN's SemVer convention for
"behavioral correctness on a previously-silently-dropped input → it now errors," that is MINOR, not a
bare PATCH. **No public API/type is broken — md-cli is a binary crate** (`[[bin]] name="md"`, no
`lib.rs`; `mod parse` is private in `main.rs`), so `PlaceholderOccurrence` is NOT a cross-crate public
API; and under REJECT the occurrence type may not even need widening (M-b, resolved).

---

## 4. H12 — DESIGN DECISION: taproot-aware default-origin helper (`3'` for `Tag::Tr`), NO new flag → PATCH-clean

### 4.1 The defect (verified)
`compute_default_origin_path(network, account)` hardcodes the BIP-48 script-type component to `2'`
(P2WSH) with **no taproot inspection** — its signature literally cannot see the descriptor's `Tag`. For
`tr(NUMS, multi_a)` / `tr(sortedmulti_a)` every cosigner key whose `[fp/path]` origin is elided lands in
the `2'` (P2WSH) subtree instead of `3'` (P2TR). md-codec's `canonical_origin` returns `None` for
`tr(@N, TapTree)`, so taproot multisig **always** enters this default-inference branch. Result: every
receive + change address diverges; coins are un-cosignable by any BIP-48 coordinator
(Sparrow/Coldcard/Jade re-derive at `3'`). **Template-mode is already correct** (it threads
`bip48_script_type()` → emits `3'`); only descriptor-mode is wrong — the same taproot inputs produce two
divergent origins (the **H12-crossmode** facet, folded in here).

**Empirical proof (bug-hunt report):** descriptor-mode `tr(NUMS,multi_a)` receive[0] toolkit
`bcrt1pe8q8h9a67gq6fpuycxu8zuskwg6e93vu4qryfekclcj7lly8atqsjv2ww7` (`2'`) vs Core/intended
`bcrt1p20ad3q3errr7h4p06j6vxj7sygppgnvjylnyyejy8nuz77jxgv5qmqyk3v` (`3'`); **all 6 receive + change[0]
differ**; reproduced for `sortedmulti_a` too. The mk-decoded cosigner xpub is byte-for-byte the
independent `2'` derivation — keys live in the wrong subtree, not just a mislabel.

### 4.2 The `3'`=P2TR standardization nuance (state precisely)
**BIP-48 standardizes ONLY `1'` (P2SH-P2WSH / nested-segwit) and `2'` (P2WSH / native-segwit).** The
BIP text explicitly says "the only script types covered by this BIP are Native Segwit (p2wsh) and Nested
Segwit (p2sh-p2wsh)" and does **NOT** define `3'` for taproot. **`3'` = P2TR is a *de-facto interop
convention*** adopted by the major coordinators — **Sparrow, Coldcard, and Jade** all derive taproot
multisig at `m/48'/coin'/account'/3'` — and the differential-oracle confirmed Core derives the wallet at
`3'`. The toolkit's own source already encodes this: `template.rs:231-237` maps `TrMultiA/TrSortedMultiA
=> Some(3)`, and `template.rs:243-262` documents `3'` as "a toolkit convention, NOT part of BIP-48"
(resolving FOLLOWUP `multisig-tr-bip48-script-type-3-policy`: bless-and-warn).

So the spec phrases it precisely: **the "correct" value is `3'` *as the constellation's own convention
and what the major coordinators + template-mode already emit*, NOT a BIP-48 standard.** The defect
(descriptor-mode `2'` ≠ template-mode `3'` for the SAME taproot inputs → non-cosignable wallets) is real
and reproduces regardless of the standardization nuance.

### 4.3 The fix — taproot-detection mechanism is PER call site (R0 round-1 I2)
Make the default-origin inference **taproot-aware**, killing the literal `2`/`3` divergence at every site.
**The detection mechanism differs by site because the three call sites hold DIFFERENT descriptor
representations** (R0 round-1 I2 — verified): the first two hold an **md-codec tree/`Tag`**; the third
holds a **rust-miniscript `Descriptor`**, which has no md-codec `Tag` to thread. The helper receives a
precomputed `script_type: u32`, but each caller computes that `u32` from the representation IT holds:
1. **`compute_default_origin_path` (`bundle.rs:2210`):** add a `script_type: u32` parameter and use it for
   the 4th `PathComponent.value` instead of the hardcoded `2`. The `bundle.rs` caller has the **md-codec
   tree/`Tag`** in scope (`canonicity_probe = parse_descriptor(...)`, whose `.tree` already feeds
   `canonical_origin(&...tree)` immediately before this call) → compute `script_type` via the md-codec
   `Tag` / **`template.rs::bip48_script_type()`** (the 1/2/3 mapping authority for the `Tag`-holding
   callers): `Tag::Tr` → `3`, wsh → `2`, sh-wsh → `1`.
2. **`verify_bundle.rs:1373` mirror:** the symmetric verify path calls the SAME helper and ALSO holds the
   md-codec `Tag` — once the helper takes `script_type` and the call site passes the `Tag`-derived value,
   the verify mirror is fixed in lockstep. (This is *why* H12 belongs on the S-VERIFY branch with H1 —
   same file zone; the verify side must not re-drift.)
3. **`descriptor_intake.rs:324, 345`:** these live in `parse_literal_xpub`, which operates on
   `parsed = MsDescriptor::<DescriptorPublicKey>::from_str(...)` (`descriptor_intake.rs:297`) — a
   **rust-miniscript `Descriptor`, NOT an md-codec `Tag`.** There is **no `Tag` to thread here.** Detect
   taproot directly on the miniscript descriptor — `matches!(parsed, miniscript::Descriptor::Tr(_))` (or
   an equivalent `is_taproot`) → `script_type = 3`, else `2` — and pass it to `bip48_default_path`
   (which already takes a `script_type` param, `:410`). Do **NOT** plumb an md-codec re-parse just to
   obtain a `Tag`, and do **NOT** leave the literal `2` here ("no Tag available") — that would
   re-introduce the exact H12 defect on the xpub-search intake path (which `compute_default_origin_path`
   does NOT cover — this path has its own `bip48_default_path` helper).
4. **Info-strings:** the "defaulting origin path … to m/48'/…/2'" stderr notices (`bundle.rs:~2280`,
   `descriptor_intake.rs:~395`) must render the actual computed component, not a hardcoded `2'`.
5. **Preserve** the `bip48_nonstandard_script_type_warning` advisory semantics — taproot-multisig under
   the bip48 family still emits the "3' is a toolkit convention, not BIP-48" stderr advisory.

**Mapping-authority note:** `template.rs::bip48_script_type()` remains the 1/2/3 authority **only for the
`Tag`-holding callers** (`bundle.rs`/`verify_bundle.rs`). The miniscript caller (`descriptor_intake.rs`)
maps `Descriptor::Tr → 3` directly, since it cannot reach the md-codec `Tag`.

### 4.4 The flag question — DECISION: NO new flag → PATCH-clean
The recon asked whether a user override/flag is warranted. **DECISION: NO new flag.** Rationale:
- The correct value is **deterministic from the descriptor's tree** (`Tag::Tr` → `3'`). There is no
  ambiguity for the user to resolve — descriptor-mode should simply match what template-mode and every
  coordinator already do. Adding a flag would invite the user to re-introduce the wrong value.
- **H12-crossmode** (descriptor-mode currently *rejects* `--multisig-path-family bip48`, so there's "no
  escape hatch") is resolved by making the **default correct**, not by opening a descriptor-mode
  `--multisig-path-family` flag. Once the default emits `3'` for taproot, the crossmode divergence
  (template `3'` vs descriptor `2'`) is closed without any new surface.
- **No flag → no GUI schema_mirror drag, no manual lockstep → PATCH-clean in isolation.** (The toolkit
  release is MINOR overall only because it co-ships the H13 pin bump.)

> **R0 OPEN QUESTION (Q-H12-1):** Confirm there is genuinely no legitimate use-case where a user wants a
> *taproot* descriptor's cosigners defaulted to `2'` (e.g. an odd coordinator that mis-derives). If R0
> finds one, the fallback is a `--multisig-path-family bip48`-in-descriptor-mode escape hatch — which
> would flip H12 to MINOR-with-GUI-schema-and-manual lockstep. The spec RECOMMENDS no-flag (the value is
> tree-deterministic and matches all coordinators).

> **Q-H12-2 — RESOLVED (R0 round-1 I2):** the helper **receives a precomputed `script_type: u32`** (not a
> threaded `&Tag`), because the three call sites do NOT all hold a `Tag`. `bundle.rs` and
> `verify_bundle.rs:1373` hold the md-codec tree/`Tag` → compute `script_type` via `bip48_script_type()`.
> `descriptor_intake.rs:324,345` holds a **rust-miniscript `Descriptor`** (`from_str` at `:297`), NOT a
> `Tag` → detect taproot via `Descriptor::Tr(_)` and map `Tr → 3` directly. So "thread the `Tag`" is
> wrong for the third site; the detection mechanism is per-site (§4.3). `bip48_script_type()` stays the
> mapping authority only for the `Tag`-holding callers.

---

## 5. H1 — DESIGN DECISION: structural decoded-policy compare (Tag + k + wrapper + index-aware binding), NOT byte-exact md1 string

### 5.1 The defect (verified)
For a keyed multisig md1, `emit_multisig_checks`'s `md1_xpub_match` compares ONLY the **sorted pubkey
multiset** (`verify_bundle.rs:2718-2735`). Threshold `k`, policy-tree `Tag` (Multi/SortedMulti/Tr/…),
script-type wrapper (`wsh`/`sh-wsh`/`tr`), and key-order/slot binding are **never** compared. So
`sortedmulti(1,…)` (1-of-3 anyone-spends), unsorted `multi(2,…)`, and `sh(wsh(sortedmulti(2)))` all
GREEN-light (exit 0) against an engraved `wsh(sortedmulti(2,A,B,C))` — every one reconstructs a
*different* wallet with different addresses. A genuinely-wrong cosigner xpub DOES surface (`result:
mismatch`), proving the gate is **structurally blind, not always-green** — it's a false-assurance hole,
the worst kind for a verification tool.

### 5.2 The compare-strategy decision: structural decoded-policy compare
Two candidate strategies (recon framing):
- **(A) Byte-exact `expected.md1 == supplied.md1`** — mirrors the keyless single-sig path
  (`verify_bundle.rs:645`).
- **(B) Structural compare of the decoded `Descriptor`** — tree `Tag` + threshold `k` + wrapper + an
  index-aware pubkey/slot binding.

**DECISION: (B) — structural decoded-policy compare, reusing the derived `tree ==` equality primitive
(R0 round-1 I1).** Rationale (the origin-elision / canonicalization edge cases the recon flagged):
- **Byte-exact md1 equality is too brittle for the KEYED multisig path.** The keyed md1 carries
  per-cosigner origin paths, fingerprints, and a `path_decl` that can be encoded in *canonically
  different-but-semantically-equal* forms (origin elision, Shared-vs-Divergent path-decl modes, the very
  multiset-reorder the v0.5.0 B.3 change introduced multiset-compare to tolerate). Two md1 strings can
  encode the *same wallet* yet differ byte-for-byte (e.g. an elided empty origin vs an explicit canonical
  origin — cf. bug-hunt L14: `WalletPolicyId` is NOT stable across origin elision). A byte-exact compare
  would FALSE-FAIL legitimate descriptor-mode bundles — re-introducing exactly the class of false-negative
  the B.3 multiset change was created to avoid. The single-sig path can use byte-exact because it
  *recomposes* the md1 from canonical inputs (`expected.md1` is the toolkit's own canonical emission);
  the keyed path compares a *supplied external* md1 whose canonical form may legitimately differ.
- **What MUST be compared (the consensus-significant + policy-defining fields) — all covered by `tree ==`:**
  1. **Threshold `k`** — distinguishes 1-of-3 from 2-of-3 (the `sortedmulti(1)` anyone-spends case).
  2. **Tree `Tag`** — `Multi` vs `SortedMulti` vs `MultiA`/`SortedMultiA` (sorted-vs-unsorted is
     consensus-significant; for unsorted `multi`/`multi_a` the **key order** is also consensus-
     significant — see index-aware binding below).
  3. **Script-type wrapper** — `wsh` vs `sh(wsh(…))` vs `tr(…)` (distinguishes the `sh(wsh(sortedmulti(2)))`
     P2SH-nested case; different address type).
  4. **Index-aware pubkey/slot binding** — for **sorted** shapes (`sortedmulti`/`sortedmulti_a`) order is
     not consensus-significant (BIP-67 sorts at derive); for **unsorted** `multi`/`multi_a` the compare
     MUST pin **per-slot** pubkey equality (order matters). This **subsumes the downgraded L24-adjacent
     finding** ("sorted-multiset drops slot→key binding"). (The report's L24 is a separate OOB-panic
     guard-asymmetry; the *multiset-index* downgraded Wave-1-appendix item is the one subsumed here.)
  5. **Nested policy structure** (timelocks / hashlocks / branches in a general policy) — covered by the
     decoded-tree compare; this also hardens against the general-policy-collapse class fixed before.

### 5.3 Implementation shape — reuse the derived `tree ==` equality (do NOT hand-roll a predicate)
**The codebase already has the exact, index-aware structural primitive: the decoded md-codec tree derives
full `PartialEq/Eq` end-to-end**, so a single `==` subsumes all five fields above (R0 round-1 I1, verified
against `origin/main` @ `54dd765`):
- `md-codec/src/encode.rs:16` — `struct Descriptor { n, path_decl, use_site_path, tree, tlv }` derives
  `PartialEq, Eq`.
- `md-codec/src/tree.rs:8` — `struct Node { tag, body }` derives `PartialEq, Eq`.
- `md-codec/src/tree.rs:17` — `enum Body`, where `Body::MultiKeys { k: u8, indices: Vec<u8> }` carries
  **both threshold `k` AND the index-ordered per-slot binding** (the `@i` placeholders in slot order) — so
  `tree == tree` is order-significant on `indices`, the index-aware binding §5.2.4 needs, for free.
  `Body::Tr { is_nums, key_index, tree }` and `Body::Variable { k, children }` likewise.
- `md-codec/src/tag.rs:14` — `enum Tag` (Multi/SortedMulti/MultiA/SortedMultiA/Wsh/Sh/Tr…) derives
  `PartialEq, Eq`; the wrapper IS the root `tree.tag`.

So decode both md1s to their `md_codec::Descriptor` (the verify path already decodes both —
`expected_md_decoded` and `desc` in hand at `:2719-2730`) and replace the sorted-multiset-only
`pubkeys_match` with:
```
md1_policy_match  :=  decoded_expected.tree == decoded_supplied.tree
//  (and `use_site_path` equality too, iff use-site binding must be pinned — plan-doc to confirm)
```
This single derived compare covers §5.2.4 (1)-(5) — Tag + k + wrapper + index-aware slot-order + nested
policy structure — with no hand-rolled traversal, materially simpler and harder to drift than four
separate conjuncts.

**CRITICAL sub-nuance — the `sh(multi)` vs `sh(wsh(multi))` trap (`md-codec/src/decode.rs:35-38`):** root
`Tag::Sh` covers BOTH `sh(multi)` AND `sh(wsh(multi))` — they share the root tag and differ ONLY in the
nested body (one extra `Wsh` child node). A naive "compare root tag + k" predicate would treat
`sh(wsh(sortedmulti(2)))` and `sh(sortedmulti(2))` as EQUAL — a false-GREEN, and `sh(wsh(…))` is one of
the §6.1 row-H1 discriminator cases. **`tree ==` handles this correctly** (the nesting differs); a
hand-rolled root-tag-only `wrapper_equal` would NOT. The implementer MUST NOT hand-roll a root-tag-only
wrapper check — this is the decisive reason to reuse the derived `PartialEq`.

- **`compute_wallet_policy_id` is DISQUALIFIED** and MUST stay out: it is origin/fingerprint/xpub-presence
  -significant (`md-codec/src/identity.rs:194-237` hashes the per-`@N` origin path verbatim, bug-hunt L14
  confirmed; its "stable across origin elision" doc-comment is a lie, test `walletpolicyid_stable_across_
  origin_elision` vacuous per L17) — it would FALSE-FAIL the same way byte-exact md1 would.
- **Distinct from the keyless-template arm:** `verify_bundle.rs:882` correctly uses
  `compute_wallet_descriptor_template_id` (origin-INVARIANT, `identity.rs:73-106`) — the right precedent
  for *that* (keyless) arm, but it is template-id (no keys); do not conflate. The keyed-multisig arm uses
  `tree ==` on the decoded keyed descriptors.
- Retain `extract_multisig_threshold` (`bundle.rs:1197`, `pub(crate)`, returns `Body::MultiKeys{k,..}`)
  ONLY for human-readable mismatch-detail strings, NOT the verdict.
- **Wire-shape:** the verify-bundle `checks[]` array (`VerifyCheck { name: String, passed, … }`,
  `format.rs:131`) has a free-form name and is NOT consumed/keyed by the GUI (R0 round-1 Q-WIRE
  confirmed). Lowest-risk path: **keep the `md1_xpub_match` check NAME and change only its `passed`
  predicate** to reflect the full `tree ==` verdict (zero wire churn, costs only same-PR in-repo test
  updates). No external paired-PR obligation exists. (See §7.)

> **Q-H1-1 — RESOLVED (R0 round-1 I1):** reuse the derived `tree ==` primitive (`md_codec::Node`/`Body`/
> `Tag` `PartialEq/Eq`). `compute_wallet_policy_id` is DISQUALIFIED (origin-instability, bug-hunt L14). A
> hand-rolled four-conjunct predicate is over-engineered and itself a false-GREEN surface — rejected.

> **Q-H1-2 — RESOLVED (R0 round-1 I1):** do NOT compare per-cosigner ORIGIN paths in `md1_policy_match`.
> `tree ==` compares the keyed placeholder `indices`, not origins (origins live in `path_decl`/TLV);
> binding origins in the fast gate would re-introduce the L14 origin-elision brittleness. The §6
> differential-oracle row (Core-derived address equality) is the funds-truth backstop; the structural
> compare is the necessary-not-sufficient fast gate. The H12 fix independently closes the
> origin-divergence wrong-address class, so the gate need not also police origins to be funds-safe.

### 5.4 Designed so the later S-VERIFY dedup subsumes it
H1's compare and H12's helper are the **Tier-0 anchor hunks** on the S-VERIFY branch; the broader
bundle↔verify dedup (L24, H7-lexer, the full shared-binding extraction) lands later. To avoid re-drift:
implement H1's `md1_policy_match` and H12's taproot-aware helper as **single shared functions** (not
duplicated bundle-side / verify-side copies), so when the later dedup consolidates the descriptor-mode
binding it absorbs these without rewriting them. **RE-OPEN the stale FOLLOWUP**
`verify-bundle-multisig-md1-xpub-match-set-equality` (`FOLLOWUPS.md:1635`, status `:1641`) — flip its
status from "resolved by v0.5.0 Phase B.3" to re-opened/superseded **in the shipping commit** (per the
followup-status-discipline rule; do NOT edit FOLLOWUPS.md now — see §9).

---

## 6. Test plan / gating — `bitcoind_differential.rs` regression rows + unit tests

**Gating contract (PLAN §3):** a class-A fix is NOT done until its differential row is GREEN. All three
findings are class-A (wrong-address or false-verdict). Each fix MUST add its triggering shape to
`crates/mnemonic-toolkit/tests/bitcoind_differential.rs` (the Phase-0 oracle harness) as a regression
gate, mirroring the existing pattern: a `#[ignore]`/env-gated heavy row against Core `deriveaddresses`
PLUS a DEFAULT-CI anti-vacuity leg that asserts the toolkit address == an independent rust-miniscript
`derive_receive` (the harness already has `derive_receive` at `:233`, `core_addresses` at `:319`,
`divergent_differential_golden` and `template_completion_anti_vacuity_leg` as the default-CI legs to
mirror).

### 6.1 Differential-oracle rows (the proof-shapes)

**Row H12 (taproot `2'`-vs-`3'`):** `bundle --descriptor "tr(NUMS,multi_a(2,@0/<0;1>/*,@1/<0;1>/*))"`
(and a `tr(sortedmulti_a)` companion) with bare-`@N` cosigners. Assert: (a) the emitted origin is
`m/48'/<coin>'/<account>'/3'` (NOT `2'`); (b) Core `deriveaddresses` of the reconstructed descriptor ==
the toolkit's reported addresses at every receive + change index; (c) **descriptor-mode == template-mode**
at every index (the crossmode assertion). Default-CI anti-vacuity leg: toolkit address ==
`derive_receive` of the `3'`-origin descriptor. **This is the highest-priority oracle row — it gates the
first impl batch (PLAN §3 row 5).**

**Row H1 (false-GREEN discriminator):** engrave a real `wsh(sortedmulti(2,A,B,C))` bundle, then run
`verify-bundle` against md1s that reconstruct DIFFERENTLY: (a) `sortedmulti(1,…)` 1-of-3, (b) unsorted
`multi(2,…)`, (c) `sh(wsh(sortedmulti(2,…)))`. Assert ALL THREE → `result: mismatch` (exit ≠ 0) — they
currently GREEN-light. Clean-negative companion: a genuinely-matching md1 → `result: ok` (exit 0) — must
stay green (no over-rejection). This is a verify-bundle behavioral test (no Core derive needed for the
mismatch assertion itself, but the corpus addresses come from the oracle so the "different addresses"
premise is anchored).

**Row H13 (hardened-multipath REJECTED / fail-closed):** `md encode`/`bundle --descriptor` a
`wsh(multi(2,@0/<0';1'>/*,@1/<0';1'>/*))` (and a `<0h;1h>` form). Assert a **typed parse ERROR at
encode/parse (exit ≠ 0)** — `CliError::TemplateParse` (md-cli) / `ToolkitError::DescriptorParse`
(toolkit) — NEVER a silent bare-`/*` collapse to the wrong-address single-path key (`bcrt1qq0kxm9…`).
This is the C1 REJECT decision (md1 cosigner keys are xpubs; hardened public derivation is impossible —
the card is permanently un-restorable, so encode fails closed). Since reject produces no address, the
default-CI leg asserts the **exit code + stderr message class**, not a derived address. Clean-negative
companion: non-hardened `<0;1>` MUST still round-trip correctly for receive AND change (do NOT
over-reject — the oracle proved `<0;1>` is fine; its address == `derive_receive`). Both md-cli and the
toolkit `bundle --descriptor` must produce the SAME reject verdict (the two-repo lockstep). **NB — this
is the hardened-multipath-REJECTED case, NOT a derive case** (per R0 round-1 C1).

### 6.2 Unit tests (per repo, TDD — written RED first)
- **H13 md-cli (REJECT):** lexer **captures** the `'`/`h` marker in `<0';1'>` (no longer silently
  invisible); `make_use_site_path` returns a typed `CliError::TemplateParse` for the hardened alt (never
  builds `Alternative { hardened: false, … }` from a hardened input); malformed `<0'';1>` → typed
  `CliError::TemplateParse`; non-hardened `<0;1>` still parses and round-trips (no over-reject).
- **H13 toolkit (REJECT):** the `parse_descriptor.rs` mirror — `make_use_site_path` widened to `Result`,
  returns `ToolkitError::DescriptorParse` for a hardened alt; `bundle --descriptor` with a hardened
  multipath EXITS with the typed error (no silent bare-key md1 emitted); non-hardened `<0;1>` still
  produces a correct md1.
- **H12 toolkit:** `compute_default_origin_path` (new `script_type: u32` signature) returns `…/3'` for
  taproot, `…/2'`/`…/1'` for wsh/sh-wsh; the `Tag`-holding call sites (`bundle.rs`, `verify_bundle.rs:1373`)
  emit `3'` for `Tag::Tr` via `bip48_script_type()`; the miniscript call sites
  (`descriptor_intake.rs:324,345`) emit `3'` for `Descriptor::Tr(_)` via `bip48_default_path(.., 3)`
  (NOT a threaded `Tag` — the per-site detection of §4.3); the stderr notice renders the actual
  component; the `bip48_nonstandard_script_type_warning` advisory still fires for bip48-family taproot.
- **H1 toolkit:** `emit_multisig_checks` — the 3 wrong-wallet md1s (`sortedmulti(1)`, unsorted
  `multi(2)`, `sh(wsh(sortedmulti(2)))`) each fail the `tree ==` predicate (→ `result: mismatch`);
  **explicitly assert the `sh(multi)` vs `sh(wsh(multi))` discrimination** (the nested-body trap a
  root-tag-only check would miss); the matching md1 yields `passed: true`; an index-permuted unsorted
  `multi` (same multiset, different order) → mismatch (the index-aware `Body::MultiKeys.indices` binding);
  a legitimately origin-elided-but-equal md1 → still `passed: true` (no false-fail — the brittleness
  guard that `tree ==`, unlike a byte/policy-id compare, provides).

### 6.3 Full-suite discipline (MEMORY `feedback_r0_review_run_full_package_suite`)
Run the **full package suite** (`cargo test -p mnemonic-toolkit`, `cargo test -p md-cli`) — NOT targeted
`--test` targets — plus clippy `-D warnings`, after each phase and before any tag. Re-run the fuzz suite
after the toolkit version bump, before the tag (release ritual, MEMORY
`project_toolkit_release_ritual_version_sites`).

---

## 7. Lockstep & SemVer matrix

| repo | fix(es) | SemVer | what it drags | gate-enforced? |
|---|---|---|---|---|
| **md-cli** (`descriptor-mnemonic`) | H13 lexer-capture + `make_use_site_path` **REJECT** | **MINOR** (`0.7.1`→`0.8.0`) tag + **publish crates.io** | behavioral correctness (now **typed-rejects** hardened multipath, was silent-collapse); reuses `CliError::TemplateParse` (no new variant); companion FOLLOWUP in `descriptor-mnemonic/design/FOLLOWUPS.md` | crates.io publish is the hard serial edge before the toolkit pin |
| **md-codec** | NONE (wire carries the bit; derive-time refusal is correct BIP-32 behavior, NOT a gap) | — | — (no derive-gap workstream, no companion FOLLOWUP — Q-H13-1 resolved REJECT) | — |
| **toolkit** | H13 mirror (`parse_descriptor.rs` REJECT, `make_use_site_path`→`Result`) **+** H12 (3 sites + info-strings) **+** H1 (`emit_multisig_checks`, `tree ==`) | **MINOR** (one release) — the md-cli **pin bump** forces ≥ MINOR; H12 + H1 are PATCH-clean in isolation | pins new md-cli; companion FOLLOWUP; re-opens the stale B.3 FOLLOWUP; version-site ritual (BOTH READMEs + `fuzz/Cargo.lock` + `scripts/install.sh` self-pin) | pin-check CI; NO schema_mirror/manual (no flag change) |
| **GUI** (`mnemonic-gui`) | NONE | — | — (no flag / dropdown / subcommand change) | schema_mirror NOT triggered |
| **manual** (`docs/manual/`) | NONE | — | — (no CLI-surface change) | manual-lint NOT triggered |

**Wire-shape caveat (RESOLVED — no external obligation, R0 round-1 Q-WIRE):** the `verify-bundle --json`
`checks[]` array (`VerifyCheck { name: String, passed, … }`, `format.rs:131`) has a free-form name and is
**NOT consumed/keyed by the GUI** (the GUI runner is a generic subprocess capture; grep finds no keying
on `md1_xpub_match`/`"checks"`). No `schema_mirror` or wire-shape gate covers `checks[]` (schema_mirror
gates clap flag-NAMES only). The lowest-risk H1 shape — **keep the `md1_xpub_match` check NAME and change
only its `passed` predicate to the `tree ==` verdict** — is a pure value change with ZERO wire churn (only
same-PR in-repo test updates, e.g. `cli_json_envelopes.rs` count/name asserts). **No paired-PR GUI
obligation exists.** (If the plan-doc instead appends a `md1_policy_match` check, that is an append-only
in-repo wire change, still no external gate.)

**error.rs ordering (CLAUDE.md):** if H12/H13/H1 add any `ToolkitError` variant (e.g. a
hardened-multipath malformed error, or an H1 structural-mismatch variant), it MUST be inserted
**alphabetically-by-variant-name** in the declaration + `Display` + `exit_code` + `kind` match blocks
from the first commit (concurrent-PR conflict avoidance). Most likely H13's toolkit-mirror error reuses
the existing `DescriptorParse`/`TemplateParse` family — verify before adding a new variant.

---

## 8. Per-bug execution order, worktree/branch plan, R0 gate, open risks

### 8.1 Execution order (PLAN Tier-0 / Batch 0.5)
1. **H13 first (unblocks the lockstep clock).** Two-repo: md-cli fix → MINOR tag → **publish to
   crates.io** → toolkit `parse_descriptor.rs` mirror + pin bump (same toolkit release). Codec-publish-
   before-pin is a hard serial edge — start the md-cli tag earliest so the toolkit pin isn't the
   critical-path tail. **Its own concurrent workstream** (`WS-MD-CLI-LEX-H13`).
2. **H12 + H1 — ONE serialized S-VERIFY-zone branch.** Both edit `bundle.rs`/`verify_bundle.rs`; they
   **cannot be two concurrent agents.** Land H12's taproot-aware helper + H1's structural compare on one
   branch (single subagent, per-phase TDD). H12's `verify_bundle.rs:1373` mirror and H1's
   `emit_multisig_checks` are in the same file — design them together so the verify side is fixed in
   lockstep and the wrong-path fix can't re-drift. **Implement H1's compare and H12's helper as single
   shared functions** so the later (out-of-scope) S-VERIFY dedup subsumes them (§5.4).

→ **Tier-0 concurrency peak = 2 agents** (H13 branch ‖ S-VERIFY-zone branch). The ≤10 cap is trivially
honored.

### 8.2 Worktree / branch plan
- Per CLAUDE.md + PLAN §6.4: a SINGLE subagent per phase, in a **git worktree** (`isolation: "worktree"`)
  — never parallel re-impls of the same bug.
- Two feature branches: `WS-MD-CLI-LEX-H13` (md-cli + the toolkit mirror half) and the
  `S-VERIFY-zone` branch (H12 + H1). The toolkit `parse_descriptor.rs` mirror and the H12/H1 toolkit
  hunks ride **one toolkit release** — coordinate the two toolkit-touching branches' merge order
  (the H13 mirror + the S-VERIFY-zone hunks both land before the single toolkit tag; they edit different
  files — `parse_descriptor.rs` vs `bundle.rs`/`verify_bundle.rs` — so no content conflict).
- Stage paths explicitly (no `git add -A`, CLAUDE.md). Do NOT `cargo fmt --all` / fmt `mlock.rs` (g6
  exemption). Do NOT `cargo fmt` the GUI (no fmt gate) — though the GUI isn't touched this cycle.

### 8.3 R0 / review gate (CLAUDE.md hard-gate — restated)
- **This brainstorm-spec** → opus-architect R0 loop, persist each review verbatim to
  `design/agent-reports/cycle1-critical-fixes-brainstorm-R<n>.md`, fold → re-dispatch → repeat **until
  0C/0I**. **NO plan-doc work before this is GREEN.**
- **Then the plan-doc** → its own R0 loop to 0C/0I (NO code before GREEN).
- **Per-phase TDD** (tests RED first) → single subagent → per-phase reviewer-loop to 0C/0I (persist each).
- **Mandatory independent adversarial post-implementation review over the WHOLE diff** (catches
  impl-introduced regressions TDD misses). Persist verbatim. If Agent-API dispatch fails mid-session,
  FLAG it and DEFER the review to recovery — never silently substitute inline self-review.
- **Class-A gating:** the post-impl review confirms the `bitcoind_differential` suite (default-CI legs +
  env-gated rows) is GREEN with the new H12/H13/H1 rows BEFORE merge.

### 8.4 Open risks
1. **H13 derive-path (Q-H13-1 — RESOLVED to REJECT, R0 round-1 C1).** md-codec `derive_address`
   **refuses** a hardened use-site alternative (`derive.rs:105-107`), as do `md address`, toolkit
   `restore --md1` (`restore.rs:2779`), and rust-miniscript — md1 keys are xpubs and BIP-32 forbids
   hardened public derivation. So the decision is REJECT (capture-then-typed-error). The residual risk is
   purely **over-rejection**: the H13 impl must NOT also reject non-hardened `<0;1>` (the §6 clean-negative
   leg guards this). No funds-safety risk remains in the reject direction (fail-closed is safe).
2. **H1 false-negative brittleness.** A too-strict compare (byte-exact, or an origin-significant policy-id)
   re-introduces the v0.5.0-era false-FAILs on legitimately origin-elided descriptor-mode bundles
   (exactly why B.3 went multiset). The derived `tree ==` compare (§5.3) dodges this — it compares the
   keyed `tree`, not the origin-significant policy-id — so it is neither too strict (origin-elided-but-
   equal md1 → still `passed:true`, the §6.2 guard) nor too loose (it IS index-aware and nesting-aware,
   distinguishing `sh(multi)`/`sh(wsh(multi))`). Residual: the `use_site_path`-equality add-on (if needed)
   is the only open shape — plan-doc to confirm.
3. **H12 per-site detection (Q-H12-2 — RESOLVED).** The third call site (`descriptor_intake.rs`) holds a
   rust-miniscript `Descriptor`, NOT an md-codec `Tag`, so it detects taproot via `Descriptor::Tr(_)`
   (§4.3); the first two use the md-codec `Tag`/`bip48_script_type()`. Residual risk: the implementer must
   NOT leave the literal `2` at the third site for lack of a `Tag` (the §6.2 unit test guards it).
4. **verify-bundle `--json` wire-shape (Q-WIRE — RESOLVED, no obligation).** The GUI does not consume
   `checks[]`; keeping the `md1_xpub_match` NAME + changing only its `passed` predicate is zero wire churn
   (§7). No external paired-PR obligation.
5. **Cross-repo publish timing.** md-cli MINOR must publish to crates.io before the toolkit pin — the
   tightest cross-repo coupling and the earliest publish gate in the whole program. Mitigation: schedule
   the md-cli tag first (§8.1).
6. **Concurrent-PR `error.rs` collisions.** Alphabetical-by-variant ordering from the first commit (§7).

---

## 9. FOLLOWUP actions (LIST ONLY — do NOT edit FOLLOWUPS.md this cycle)

> Another instance is doing spec/planning in `design/`; FOLLOWUPS edits are **deferred to the shipping
> commit** to avoid contention (and per the followup-status-discipline rule, statuses flip in the
> shipping commit). These are the slugs to file/flip THEN.

**3 new slugs to file (cross-repo where noted):**
1. **`h13-hardened-multipath-reject`** (toolkit **+** md-cli companion, cross-citing `Companion:` lines
   per CLAUDE.md cross-repo rule) — record the **REJECT** decision (capture-then-typed-error; R0 round-1
   C1 flipped this from faithful-represent), the two-repo lexer-capture + `make_use_site_path` reject fix,
   the md-codec derive-time refusal fact (xpub keys + BIP-32 → hardened public derivation impossible), and
   that the Q-H13-1 derive-path validation resolved to REJECT with NO md-codec change.
2. **`h12-descriptor-mode-taproot-default-origin-3prime`** (toolkit) — record the taproot-aware
   default-origin helper, the `bip48_script_type()` reuse, the 3-call-site fix, and the `3'`=de-facto
   (Sparrow/Coldcard/Jade) convention rationale.
3. **`h1-verify-bundle-structural-policy-compare`** (toolkit) — record the derived **`tree ==`** equality
   compare (covers Tag + k + wrapper + index-aware slot binding + nesting; distinguishes `sh(multi)` from
   `sh(wsh(multi))`) replacing the multiset-only check; note `compute_wallet_policy_id` is disqualified
   (origin-instability) and origin paths are deliberately not compared in the gate.

**1 stale slug to RE-OPEN (toolkit):**
4. **`verify-bundle-multisig-md1-xpub-match-set-equality`** (`FOLLOWUPS.md:1635`, status `:1641`) — flip
   from "resolved by v0.5.0 Phase B.3" → **re-opened/superseded by H1** (the multiset-equality change H1
   proves insufficient). Fold the downgraded multiset-index item (Wave-1 appendix) into H1's closure.

**Companion-repo entries:** H13 files a companion entry in `descriptor-mnemonic/design/FOLLOWUPS.md` (the
md-cli half) with cross-citing `Companion:` lines back to the toolkit entry, per the CLAUDE.md cross-repo
rule. H12 and H1 are toolkit-internal (no companion).

---

## 10. Decisions (R0 round-1 resolutions) + remaining open questions

**RESOLVED by R0 round-1 (see the fold log, §11):**
- **Q-H13-1 (HIGH) — RESOLVED → REJECT.** md-codec `derive_address` does NOT honor `Alternative.hardened`
  — it **refuses** it (`derive.rs:105-107` → `Error::HardenedPublicDerivation`), as do `md address`,
  `restore --md1` (`restore.rs:2779`), and rust-miniscript. md1 keys are xpubs; BIP-32 forbids hardened
  public derivation. H13 = capture-then-typed-error (fail-closed). **No md-codec change, no md-codec
  FOLLOWUP** (its refusal is correct, not a gap).
- **Q-H12-2 — RESOLVED.** Helper receives a precomputed `script_type: u32`. The `Tag`-holding sites
  (`bundle.rs`, `verify_bundle.rs:1373`) compute it via `bip48_script_type()`; the miniscript site
  (`descriptor_intake.rs:324,345`, holds `MsDescriptor`, NO `Tag`) detects `Descriptor::Tr(_)` and maps
  `Tr → 3` directly. Detection mechanism is per-site (§4.3).
- **Q-H1-1 — RESOLVED.** Reuse the derived `tree ==` primitive (`md_codec::Node`/`Body`/`Tag`
  `PartialEq/Eq`) — covers Tag + k + wrapper + index-aware `Body::MultiKeys.indices` + nesting, and
  distinguishes `sh(multi)` from `sh(wsh(multi))`. `compute_wallet_policy_id` is DISQUALIFIED
  (origin-instability, bug-hunt L14). No hand-rolled predicate.
- **Q-H1-2 — RESOLVED.** Do NOT compare per-cosigner ORIGIN paths in the gate (re-introduces L14
  brittleness); the differential oracle arbitrates "same wallet"; the H12 fix independently closes the
  origin-divergence class.
- **Q-WIRE — RESOLVED.** No external paired-PR obligation: the GUI does not consume `checks[]`; keep the
  `md1_xpub_match` NAME, change only its `passed` predicate → zero wire churn.
- **Q-SEMVER — CONFIRMED.** md-cli MINOR (`0.7.1`→`0.8.0`) for the H13 typed-reject behavioral change
  (binary crate, no broken API); toolkit MINOR purely via the pin bump (H12/H1 PATCH-clean). No GUI/manual.

**STILL OPEN for the plan-doc R0 (minor, implementation-shape only):**
- **Q-H13-2:** Exact lexer regex form to **capture** the `'`/`h` marker — looser `[0-9;'h]+` (defer to the
  per-alt reject) vs stricter alternation. (`PlaceholderOccurrence`/`multipath_alts` `pub`-across-crate
  concern RESOLVED — md-cli is a binary crate, M-b; under REJECT the type likely needs no widening at all.)
- **Q-H12-1:** Any legitimate use-case for a taproot descriptor defaulting cosigners to `2'`? Spec
  RECOMMENDS no-flag (value is tree-deterministic + matches all coordinators); if a use-case surfaces →
  flag → MINOR + GUI/manual lockstep. (Plan-doc R0 to close.)

---

## 11. R0 round-1 fold log

Folds from `design/agent-reports/cycle1-critical-fixes-spec-R0-round1.md` (verdict: NOT-GREEN, 1 Critical
+ 2 Important). Each finding's source facts were re-verified against canonical bytes
(toolkit `origin/master` `4d5872ed`, md-codec/md-cli `origin/main` `54dd765`) before folding.

- **C1 (Critical) — H13 flipped FAITHFUL-REPRESENT → REJECT.** The wire carries an `Alternative.hardened`
  bit, but md1 cosigner keys are xpubs (`md-codec/src/tlv.rs:14`) and BIP-32/BIP-389 forbid hardened
  derivation from a public key, so md-codec `derive_address` **unconditionally refuses** a hardened
  use-site alternative (`derive.rs:105-107` → `Error::HardenedPublicDerivation`), as do `md address`,
  toolkit `restore --md1` (`restore.rs:2779`), and rust-miniscript. Faithful-encode would manufacture a
  permanently un-restorable (funds-unsafe) card the toolkit already flags unrestorable at engrave time.
  H13 now **captures** the `'`/`h` marker at lex (md-cli `template.rs:40`/`:220-233`; toolkit
  `parse_descriptor.rs:70`/`~228`) then returns a **typed parse error** (`CliError::TemplateParse` /
  `ToolkitError::DescriptorParse`) — never silently collapsing to bare `/*`. The `make_use_site_path`
  toolkit mirror widens to `Result`. No md-codec change, no md-codec FOLLOWUP (its refusal is correct).
  SemVer unchanged: md-cli MINOR (typed-reject is a behavioral change on a non-empty input class, but no
  flag/GUI/manual leg; tightened validation of previously-accepted-but-broken input). Q-H13-1 RESOLVED →
  reject. Folded across §0 table, §0 SemVer bullet, §1 thesis, §2 WIRE+DERIVE facts, §3 (whole section),
  §6.1 row-H13, §6.2 H13, §7 (md-codec + md-cli + toolkit rows), §8.4 risk 1, §9 slug 1, §10.
- **I1 (Important) — H1 adopts the derived `tree ==` equality primitive.** Replaced the hand-rolled
  four-conjunct predicate with `decoded_expected.tree == decoded_supplied.tree` — `md_codec::Descriptor`
  (`encode.rs:16`), `Node` (`tree.rs:8`), `Body::MultiKeys { k, indices }` (`tree.rs:17`), and `Tag`
  (`tag.rs:14`) all derive `PartialEq/Eq`, so one compare covers Tag + threshold k + wrapper +
  index-aware slot binding + nesting, and correctly distinguishes `sh(multi)` from `sh(wsh(multi))`
  (`decode.rs:35-38` — shared root `Tag::Sh`, differing nested body), which a root-tag-only check would
  miss. `compute_wallet_policy_id` confirmed DISQUALIFIED (origin/fingerprint-significant,
  `identity.rs:194-237`, bug-hunt L14). Origin paths deliberately not compared in the gate. Q-H1-1 and
  Q-H1-2 RESOLVED. Folded across §0 table, §5.2/§5.3, §6.2 H1, §8.4 risk 2, §9 slug 3, §10.
- **I2 (Important) — H12 third call-site detection corrected.** `bundle.rs:2231` and
  `verify_bundle.rs:1373` hold an md-codec `Tag` → detect taproot via `bip48_script_type()`; but
  `descriptor_intake.rs:324,345` (`parse_literal_xpub`) holds a **rust-miniscript `MsDescriptor`**
  (`from_str` at `:297`), NOT a `Tag` — so it must detect taproot via `Descriptor::Tr(_)` and map
  `Tr → 3` directly. "Thread the `Tag`" is wrong for that site. §4.3 now specifies the detection
  mechanism per call site; Q-H12-2 RESOLVED. Folded across §4.3, §4.4 Q-H12-2, §6.2 H12, §8.4 risk 3, §10.
- **Minors M-a..M-f:** noted inline (M-a `substitute_synthetic` strip-class widen for robustness, §3.4;
  M-b md-cli binary-crate resolves the `pub`-API concern, §3.5/§10; M-c/M-d/M-e/M-f citation + harness
  confirmations) — most deferred to the plan-doc per the review.

A fresh R0 pass follows this fold (per CLAUDE.md: the reviewer-loop continues after every fold).
