# R0 REVIEW — BRAINSTORM_cycle1_critical_fixes (H13 / H12 / H1) — Round 2

**Reviewer role:** opus architect, mandatory R0 gate (CLAUDE.md hard-gate: NO implementation before 0C/0I).
**Spec under review:** `design/BRAINSTORM_cycle1_critical_fixes.md` (post round-1 fold).
**Round-1 review folded:** `design/agent-reports/cycle1-critical-fixes-spec-R0-round1.md` (was NOT-GREEN: 1 Critical + 2 Important).
**Date:** 2026-06-20 (round-2), source SHAs verified live.
**Canonical sources verified (NOT working tree):**
- toolkit `origin/master` = `4d5872ed489e706155b0d88b02686977e59a20b6` (confirmed `git rev-parse`)
- descriptor-mnemonic (md-codec + md-cli) `origin/main` = `54dd765a11d490dc3d8dec2c842dae718bd3ef2b` (confirmed; NO `origin/master` — default is `main`, as the spec states)

Every load-bearing citation re-grepped against `git show origin/<branch>:<path>`. The verdict rests on first-hand source reads, not the spec's assertions. Round-2 brief: (1) confirm C1/I1/I2 are correctly resolved by the fold, (2) confirm the fold introduced no new Critical/Important (folds drift).

---

## VERDICT: **GREEN — 0 Critical / 0 Important**

The round-1 C1 (H13 faithful-represent → REJECT), I1 (H1 hand-rolled predicate → derived `tree ==`), and I2 (H12 third call-site is a miniscript `Descriptor`, not a `Tag`) are each **correctly and completely resolved** against canonical source. The fold introduced **no new Critical or Important issues**: §0 summary, §3/§4/§5 bodies, §6 test rows, §7 SemVer/lockstep matrix, §8 risks, §9 FOLLOWUPs, §10 resolutions, and the §11 fold-log are internally consistent and citation-accurate; no stale "faithfully represent" language survives. The only residual items are Minor (plan-doc-level precision), listed below. **R0 is GREEN — the spec may proceed to plan-doc R0.**

---

## CRITICAL FINDINGS

**None.**

---

## IMPORTANT FINDINGS

**None.**

---

## ROUND-1 FINDING CLOSURE

### C1 — H13 FAITHFUL-REPRESENT → REJECT — **RESOLVED ✓**

The fold flips the load-bearing decision from "set the `Alternative.hardened` bit and encode" to **"capture the `'`/`h` marker at lex, then return a typed parse error; never silent-collapse to `/*`."** Confirmed correct on every dimension:

- **Lexers DETECT-then-REJECT, never collapse.** §3.4 specifies both lexers widen the multipath group so the `'`/`h` marker is *seen* (md-cli `template.rs` group 3, toolkit `parse_descriptor.rs` group 4), then route a detected hardened alternative to a typed error (`CliError::TemplateParse` / `ToolkitError::DescriptorParse`). The §0 table, §1 thesis, §3 (whole section), §6.1 row-H13, §6.2, §7 matrix, §8.4 risk 1, §9 slug 1, §10, and §11 fold-log are all aligned on REJECT. **Zero "faithfully represent" / "set the bit" language survives** (grep-confirmed against the spec body; the only occurrences of "faithful" now appear in the REJECT *rationale* — "faithful-encode would manufacture an un-restorable card" — which is correct usage).

- **Rationale verified against md-codec source (NOT the draft):**
  - `md-codec/src/derive.rs` — `derive_address`'s **first** pre-flight is `if crate::to_miniscript::has_hardened_use_site(self) { return Err(Error::HardenedPublicDerivation); }`, doc-commented as the BIP-32 hardened-public-derivation refusal covering "a hardened wildcard OR any hardened multipath alternative, anywhere." md-codec UNCONDITIONALLY REFUSES. ✓ (spec cite `:105-107` accurate.)
  - `md-codec/src/to_miniscript.rs` — `use_site_is_hardened` returns true if `u.wildcard_hardened || any(a.hardened)` over multipath alternatives; `has_hardened_use_site` lifts it across every per-`@N` override. So a `<0';1'>` card trips the refusal. ✓ (spec cite `:101-108` accurate.)
  - `md-codec/src/tlv.rs` — `TLV_PUBKEYS` stores "chain-code || compressed pubkey, 65 bytes each" — **xpubs / watch-only, no private keys.** ✓ (spec cite `:14` accurate.)
  - BIP-32/BIP-389: hardened derivation requires private keys; a public key cannot perform it. The spec's quotation of BIP-389 ("`xpub/<0h;1h>` … technically invalid since public keys cannot perform hardened derivation") matches the round-1 live fetch. ✓
  - End-to-end refusal: toolkit `restore.rs` ModeViolation pre-check (`has_hardened_use_site` → "Faithful reconstruction is not supported") verified byte-present; the engrave-time `unrestorable_advisory.rs` fires the same predicate. ✓

- **Typed error is the right call (vs. alternatives).** Faithful-encode manufactures a steel card the constellation *already classifies* as un-restorable (the toolkit warns at engrave time) — strictly funds-unsafe. Silent-collapse mis-encodes to a wrong-address single-path key (the empirical `bcrt1qq0kxm9…` divergence). REJECT fails closed — the only safe terminal action — and matches Core ("not a valid uint32"), md-codec, BIP-32, and the bug-hunt's verbatim recommendation. The typed error reuses the **existing** `CliError::TemplateParse` / `ToolkitError::DescriptorParse` variants (both verified byte-present: md-cli `error.rs:6`; toolkit `error.rs:123` with full `Display`/`exit_code`/`kind` arms) → no new variant, no alphabetical-ordering work, no error.rs churn.

- **No md-codec change.** Confirmed correct: the wire/struct/serializer already carry the bit, and md-codec's derive-time refusal is correct BIP-32 behavior, NOT a gap. §3.4.3 / §7 md-codec row / §9 correctly strike the round-1 "separate md-codec workstream / companion FOLLOWUP" conditional. ✓

- **No over-rejection — specific to the `'`/`h` marker.** §3.3 explicitly fences a non-hardened `<0;1>` as wire-correct and derivable ("do NOT over-reject; the differential oracle proved `<0;1>` is fine"); §6.1/§6.2 keep a non-hardened `<0;1>` clean-negative round-trip (receive AND change) as a mandatory leg. Source-confirmed: md-cli's group-3 parse already does `split(';')` then `parse::<u32>()` (a trailing `'`/`h` token makes `parse::<u32>` fail today, falling to silent bare-`/*` because the regex group cannot even match `'`); the REJECT design replaces both the silent-collapse and the generic non-u32 path with a *specific, typed, marker-named* reject — narrower and safer, with the non-hardened path untouched. **No reachable legitimate input is wrongly refused.** ✓

**C1 CLOSED.**

### I1 — H1 hand-rolled four-conjunct predicate → derived `tree ==` — **RESOLVED ✓**

The fold replaces the bespoke `tags_equal && thresholds_equal && wrapper_equal && pubkey_binding_equal` predicate with the single derived comparison `decoded_expected.tree == decoded_supplied.tree` (§5.3). Verified against `origin/main` @ `54dd765`:

- **The derives genuinely exist (cited byte-exact):**
  - `md-codec/src/encode.rs` — `#[derive(Debug, Clone, PartialEq, Eq)] struct Descriptor { n, path_decl, use_site_path, tree, tlv }`. ✓
  - `md-codec/src/tree.rs` — `#[derive(… PartialEq, Eq)] struct Node { tag, body }`; `#[derive(… PartialEq, Eq)] enum Body`. ✓
  - `md-codec/src/tag.rs` — `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] enum Tag` (Wpkh/Tr/Wsh/Sh/Pkh/Multi/SortedMulti/MultiA/SortedMultiA/…). ✓

- **`tree ==` catches ALL of H1's failure modes:**
  - **Wrong threshold `k`** — `Body::MultiKeys { k: u8, indices }` carries `k`; `Body::Variable { k, children }` and `Body::Tr` likewise. `==` compares `k`. ✓ (catches `sortedmulti(1)` vs `(2)`.)
  - **Unsorted-vs-sorted** — the wrapper IS the root `tree.tag`; `Multi`/`SortedMulti`/`MultiA`/`SortedMultiA` are distinct `Tag` variants → `==` distinguishes them. ✓
  - **Script-type / wrapper drift** — root `Tag` (`Wsh` vs `Sh` vs `Tr`) compared. AND the critical `sh(multi)` vs `sh(wsh(multi))` sub-trap: `md-codec/src/decode.rs` documents verbatim that "`Sh` covers both `sh(multi)` and `sh(wsh(multi))` which are distinct BIP-388 shapes sharing the same root tag." Confirmed: a root-tag-only check would falsely equate them, but `tree ==` compares the **nested body** (one has an extra `Wsh` child node) → distinguished. §5.3 calls this out explicitly with the `decode.rs:35-38` citation and instructs the implementer to NOT hand-roll a root-tag-only `wrapper_equal`. ✓
  - **Key-order permutation** — does `tree ==` compare pubkey IDENTITY/slot-binding, or only `@N` structure? **CRITICAL CHECK ANSWERED:** `Body::MultiKeys.indices: Vec<u8>` holds the placeholder `@i` indices **in slot order**; `Vec` equality is order-sensitive, so `tree ==` is order-significant on `indices` — an index-permuted unsorted `multi` (same multiset, different slot order) → NOT equal → mismatch. §5.2.4 / §6.2 assert exactly this leg. ✓

- **PUBKEY-IDENTITY question — combined fix is complete.** `tree` compares the placeholder `@i` *index structure* (slot→index binding), NOT the raw pubkey bytes (which live in the `tlv.pubkeys` `Vec<(u8,[u8;65])>`). The spec's design is layered and complete: the **existing** sorted-pubkey-multiset check (which the fold *subordinates*, not removes — §7 keeps the `md1_xpub_match` NAME and changes only its `passed` predicate) continues to bind that the SAME set of cosigner pubkeys is present, while `tree ==` binds threshold/Tag/wrapper/slot-order/nesting. The two together close H1's full failure surface: a key SUBSTITUTION is caught by the pubkey-set comparison; a key PERMUTATION in an unsorted shape is caught by `indices` order; a Tag/k/wrapper drift is caught by `tree ==`. **The spec keeps both — confirmed complete.** (§5.2.4 item 4 + §7's "subordinate to the policy compare" framing.)

- **No false-positive on legitimate origin-elision (L14).** `tree ==` compares the keyed `tree` (placeholder indices), NOT origins (which live in `path_decl`/TLV and vary across legitimate elision). §5.3 disqualifies `compute_wallet_policy_id` (origin/fingerprint/xpub-presence-significant, `identity.rs:194-237`, bug-hunt L14) and §5.3/§10/Q-H1-2 deliberately do NOT bind origins in the gate — the §6 differential oracle is the funds-truth backstop, and the H12 fix independently closes the origin-divergence wrong-address class. So `tree ==` is **origin-independent where it should be** → an origin-elided-but-equal md1 still `passed: true` (the §6.2 guard). ✓

**I1 CLOSED.**

### I2 — H12 per-site taproot detection (`bundle.rs`/`verify_bundle.rs` hold `Tag`; `descriptor_intake.rs` holds a miniscript `Descriptor`) — **RESOLVED ✓**

The fold corrects the round-1 "thread the `Tag`" over-generalization to a **per-call-site** detection mechanism (§4.3, Q-H12-2). Verified the types at each call site on canonical source:

- **`bundle.rs` call site** (`compute_default_origin_path`, helper at `:2210`, called at `:1445`): the helper hardcodes the 4th `PathComponent { hardened: true, value: 2 }` (verified byte-present); the body uses `network.coin_type()` for the coin component (correct — so the defect is genuinely ONLY the `2`). The call site has `canonicity_probe = parse_descriptor(...)` (`:1396`) whose `.tree` feeds `canonical_origin(&canonicity_probe.tree)` (`:1398`) immediately before — so the **md-codec tree/`Tag` IS in scope** → detect via `bip48_script_type()`. ✓
- **`verify_bundle.rs:1373` mirror:** same helper, and the symmetric path holds its own `canonicity_probe = parse_descriptor(...)` with `.tree` → `canonical_origin(&canonicity_probe.tree)` immediately above the `compute_default_origin_path` call. **md-codec `Tag` in scope.** ✓ (Same S-VERIFY shared zone as H1 — correctly serialized.)
- **`descriptor_intake.rs` call sites** (`:324` and `:345`, both `bip48_default_path(network, account, 2)` with a literal `2`): these live in `parse_literal_xpub`, which operates on `parsed = MsDescriptor::<DescriptorPublicKey>::from_str(...)` (`:297`) — a **rust-miniscript `Descriptor`, NOT an md-codec `Tag`.** Confirmed. The fold's prescription — detect via `matches!(parsed, miniscript::Descriptor::Tr(_))` and map `Tr → 3`, then pass to `bip48_default_path` (whose signature already takes `script_type: u32`, `:410`) — is correct AND is a **proven in-codebase pattern**: `wallet_import/bsms.rs:295` already does `matches!(parsed, MsDescriptor::Tr(_))` for the same taproot-detection purpose (also used in `cost/strip.rs:49`, `timelock_advisory.rs:170`). §4.3 explicitly forbids leaving the literal `2` at the third site or plumbing an unnecessary md-codec re-parse, and §6.2 unit-test-gates the third site emitting `3'`. ✓
- `template.rs::bip48_script_type()` mapping (`TrMultiA|TrSortedMultiA => Some(3)`, `Wsh* => Some(2)`, `ShWsh* => Some(1)`) verified byte-exact; it remains the 1/2/3 authority **only for the `Tag`-holding callers** (§4.3 mapping-authority note is correct — the miniscript caller maps `Tr → 3` directly). The `bip48_nonstandard_script_type_warning` advisory (`3'` = toolkit convention, not BIP-48) is verified byte-present and §4.3 item 5 preserves it. ✓

**I2 CLOSED.**

---

## ADVERSARIAL FOLD-DRIFT CHECKS (new issues the fold may have introduced)

All clear. Specifics:

- **H13-as-REJECT SemVer/lockstep ("md-cli MINOR, no GUI/manual") — STILL CORRECT.** The new typed error reuses `CliError::TemplateParse` (md-cli) / `ToolkitError::DescriptorParse` (toolkit) — error TEXT, not a clap flag/dropdown/subcommand → `schema_mirror` (flag-NAMES only, per CLAUDE.md) is NOT triggered. Verified: no error-text golden/snapshot gate covers this — `cli_gui_schema_classify_descriptor.rs` gates only the **exit-code class** (exit 0 success / exit 2 `DescriptorParse` failure), which a typed reject satisfies; the manual's `42-md.md` references `<0;1>` only as a non-hardened example with NO hardened/error-catalog text, so **no manual leg**. md-cli MINOR (`0.7.1`→`0.8.0`) is right: rejecting previously-(broken)-accepted input changes observable behavior for a non-empty input class (silent-collapse → typed error) — the spec explicitly calls this out as MINOR-not-PATCH (§0 SemVer bullet, §3.5, §7) and correctly frames it as "tightening validation of previously-accepted-but-broken input," not a breaking public API (md-cli is a binary crate — M-b). **The breaking-of-previously-accepted-input concern IS called out.** ✓
- **No snapshot churn from the REJECT.** Confirmed NO existing test or golden in either repo exercises a hardened multipath `<0';1'>`/`<0h;1h>` (grepped md-cli `tests/` and toolkit `tests/`/`parse_descriptor.rs`) — so the behavior change adds NEW coverage without rewriting any captured-broken-behavior snapshot. No hidden golden-update obligation. ✓
- **Test-gating row H13 is correct.** §6.1/§6.2 now assert "hardened-multipath REJECTED" (typed-error exit ≠ 0, `CliError::TemplateParse`/`ToolkitError::DescriptorParse`, NEVER a bare-`/*` collapse to `bcrt1qq0kxm9…`) + a non-hardened `<0;1>` clean-negative that still round-trips (receive AND change). The default-CI leg asserts exit-code + stderr message class (no derived address, since reject produces none) — the right gate now that it is no longer a derive case. The §6 note "NB — hardened-multipath-REJECTED, NOT a derive case (per R0 round-1 C1)" is present. ✓ (Minor exit-code-precision note below.)
- **Internal consistency — clean.** §0 executive-summary table, §0 SemVer bullets, and the §3/§4/§5/§7/§8/§9/§10/§11 bodies agree end-to-end on REJECT (H13), `tree ==` (H1), and per-site detection (H12). No stale "faithfully represent," no contradictory SemVer call, no orphaned Q-H13-1 "validate derive honors the bit" open question (it is RESOLVED → REJECT in §3.1, §10, §11). The §11 fold-log accurately records what changed and where. Citations re-checked: the fold-log's `bundle.rs:2231` (helper-body line for the hardcoded `value: 2`) and the §2 `bundle.rs:2210` (helper signature) are both accurate referents; the round-1 review's `bundle.rs:1445` (call site) is the same hunk's call site — no contradiction, just three correct line referents for one defect. ✓

---

## MINOR / NITS (fold into the plan-doc; none block GREEN)

- **m-1 (exit-code precision):** md-cli's `CliError::TemplateParse` has no numeric `=> N` arm in `error.rs`; per the existing `encode_bad_template_returns_1` precedent (`exit_codes.rs:20`), a `CliError` returned from `main` yields exit **1**, while toolkit `ToolkitError::DescriptorParse` maps to exit **2** (`error.rs:539`). The spec asserts "exit ≠ 0" throughout (satisfied either way), but the plan-doc should pin the *exact* expected exit code per repo for the assertion (md-cli → 1, toolkit → 2) to avoid a brittle/over-specified test. (Spec is correctly non-over-committal; this is a plan-doc precision item.)
- **m-2 (§3.4 regex form, already deferred):** the exact widened-capture form (`[0-9;'h]+` vs the stricter `(?:\d+(?:'|h)?)(?:;\d+(?:'|h)?)*` alternation) is correctly left to the plan-doc (Q-H13-2). Recommend the stricter alternation so a malformed `0''`/`'h` body produces the *primary* typed reject rather than a generic catch-all — but this is plan-doc shape, not a spec gate.
- **m-3 (§3.4.4 `substitute_synthetic` strip-class):** widening the md-cli second-regex non-capturing strip to `[0-9;'h]` (so a hardened body is matched-and-stripped, avoiding a confusing *secondary* parse error before the primary reject fires) is correctly flagged as cosmetic robustness (same root-cause family as bug-hunt M5) and deferred to the plan-doc. ✓ Confirmed the second regex is byte-present with the `[0-9;]` non-capturing class.
- **m-4 (§5.3 `use_site_path` add-on):** the open "compare `use_site_path` equality too, iff use-site binding must be pinned" is correctly left as a plan-doc shape question. Note for the plan-doc: `Descriptor` also derives `Eq`, so `decoded_expected == decoded_supplied` (whole-struct) would additionally bind `use_site_path`, `path_decl`, and `tlv` — but that re-introduces origin/elision brittleness (path_decl/tlv carry origins). The spec's lean (compare `.tree`, optionally `.use_site_path`, NOT the whole struct) is the funds-safe choice; the plan-doc should make the `use_site_path`-inclusion call explicitly and justify excluding `path_decl`/`tlv`.

---

## SUPPORTING-MATERIAL ASSESSMENTS

- **All round-1 corrected citations remain byte-exact on canonical source** (re-verified this round): md-codec `derive.rs` refusal, `tlv.rs` 65-byte xpub TLV, `to_miniscript.rs` `use_site_is_hardened`, `tree.rs` `MultiKeys{k,indices}`, `tag.rs` `Tag` derives, `decode.rs` Sh-covers-both note; toolkit `bundle.rs:2210` hardcoded `value:2`, `verify_bundle.rs:1373` mirror, `descriptor_intake.rs:297` `MsDescriptor::from_str` + `:324/:345` literal `2` + `:410` `bip48_default_path(script_type)`, `template.rs` `bip48_script_type` + advisory, `restore.rs` ModeViolation, both lexer regexes + both `make_use_site_path` bodies (toolkit returns plain `UseSitePath`, must widen to `Result` — §3.4.2 captures this), `error.rs` variants.
- **Differential-oracle gating** remains an adequate class-A gate with the H13-row REJECT adjustment (typed-error exit, not a derived address) — confirmed.
- **SemVer/lockstep matrix** (§7) is correct: md-cli MINOR + publish (serial edge), toolkit MINOR via pin bump (+ H12/H1 PATCH-clean), no GUI/manual leg, no schema_mirror drag.

---

## WHAT MUST CHANGE BEFORE PROCEEDING

**Nothing blocks GREEN.** The four Minors (m-1..m-4) fold into the plan-doc, not the brainstorm-spec. The spec is **R0-GREEN at 0 Critical / 0 Important** and may advance to the plan-doc R0 loop (which is itself gated to 0C/0I before any code, per CLAUDE.md).
