# cycle-9 PLAN — R0 review, round 1

**Plan-doc:** `design/IMPLEMENTATION_PLAN_cycle9_mdcli_parser.md` (executes the R0-GREEN `BRAINSTORM_cycle9_mdcli_parser.md`).
**Findings:** M5 (funds — post-multipath suffix → divergent card) + M2 / M10 / M11 + advisory L4 / L19 / L7. md-cli-side; md-codec UNTOUCHED.
**descriptor-mnemonic source SHA:** `836faf8` (`836faf87c3d82b119a9f0f5c6589a7db1f8613a4`; md-codec 0.38.0 / md-cli 0.8.1) — independently re-verified `origin/main` == `836faf8` (local `main` = `54dd765`, STALE, as the plan warns).
**mnemonic-toolkit manual SHA (live):** `origin/master` `d6398b574292e536f9c0c9be5fa186771660a5a7` (`d6398b57`) — matches the plan.
**Reviewer:** opus software architect (mandatory R0 gate — NO code until 0C / 0I).
**Date:** 2026-06-21

Every citation independently re-grepped against the BYTES of `git show origin/main:<path>` (descriptor-mnemonic) and `git show origin/master:<path>` (toolkit). Primary focus per dispatch: the M5 §3.1.4 / §P3.4 H13-preservation (funds gate), and the load-bearing `cargo fmt` CI claim.

---

## Citation audit (vs `836faf8` / toolkit `d6398b57`)

| Plan citation | Plan says | Live @ `836faf8` | Verdict |
|---|---|---|---|
| M5 lexer regex | `template.rs:55` | `:55` `@(\d+)((?:/\d+'?)*)(?:/<([^>]*)>)?(/\*(?:'|h)?)?` | ✓ exact |
| M5 substitution regex | `template.rs:498` | `:498` `@(\d+)((?:/\d+'?)*)(?:/<[0-9;]+>)?(?:/\*(?:'|h)?)?` | ✓ exact |
| M5 group-3 validator (H13) | `:77-110` (validator loop `:90-110`) | `multipath_alts` block opens `:77` (`if let Some(m)=caps.get(3)`), per-alt `.split(';').map(…)` loop `:90-107`, collect `:108`; reject `'`/`h` + non-`u32` | ✓ (`:77-110` block; loop `:90-110` accurate) |
| M5 stitch (`parse_template` `Ok(Descriptor{…})`) | `:1915-1921`; `n`@1916 `path_decl`@1917 `use_site_path`@1918 (lexer) `tree`@1919 (substituted) `tlv`@1920 | block opens `:1915`; `n`@1916 / `path_decl`@1917 / `use_site_path`@1918 / `tree`@1919 / `tlv`@1920 | ✓ exact (incl. lexer-vs-substituted source split) |
| `Occurrence.multipath_alts` | `template.rs:27` | `:27` `pub multipath_alts: Vec<u32>` | ✓ |
| M2 n-cast | `:307-312` | `let n = (by_i.keys().max().copied()…? as usize + 1) as u8;` spans `:307-312` | ✓ |
| M2 density loop / panic-index | `:313` / `:320` (`by_i[&0]`) / `:324` (`by_i[&i]`) | `for i in 0..n`@313; `let at0 = by_i[&0];`@320; `let occ = by_i[&i];`@324 | ✓ exact |
| M10 classifier | `template.rs:1925-1932` | `ctx_for_template` body `:1925-1932`; `tr(` falls to `else`→MultiSig | ✓ |
| M10 synthetic 2nd consumer | `synthetic_xpub_for`@`:458`, called `:506` | `fn synthetic_xpub_for(i,ctx)`@458; call `:506` | ✓ |
| M10 depth gate | `keys.rs:67-77` | `let depth = bytes[4]`@67 … reject `:72-77` (SingleSig⇒3 / MultiSig⇒4) | ✓ |
| M11 payload copy | `keys.rs:78-79` | `let mut payload=[0u8;65];`@78 `payload.copy_from_slice(&bytes[13..78]);`@79 (pubkey = `bytes[45..78]`) | ✓ |
| M11 secp256k1 already-dep | `template.rs:461` | `:461` `use bitcoin::secp256k1::{Secp256k1, SecretKey}` | ✓ no new Cargo dep |
| L4 repair advisory | `repair.rs:156-159` | `:156-159` unconditional `OutputClass::Template` | ✓ |
| L19 encode advisory (JSON) | `encode.rs:73-76` | `:73-76` unconditional `Template` | ✓ |
| L19 encode advisory (text) | `encode.rs:110-113` | `:110-113` unconditional `Template` | ✓ |
| L7 stale epilog | `main.rs:241` | `:241` `after_long_help` carries "Non-chunked … rejected with a wire-format error" | ✓ |
| L7 toolkit-manual mirror | `42-md.md:367-379` @ `d6398b57` | heading "### v0.6.0 limitation: chunked-form only"@367; "rejected with a wire-format error"@373; FOLLOWUP-pointer through @379 | ✓ |
| `is_wallet_policy()` | md-codec `encode.rs:50-52`, `pub` | `:50` `pub fn is_wallet_policy(&self)`, `:51` `matches!(&self.tlv.pubkeys, Some(v) if !v.is_empty())` | ✓ |
| md-cli version site | `Cargo.toml:3` `0.8.1`, `:28` `md-codec {path,version="=0.38.0"}` | `:3` `version = "0.8.1"`; `:28` `md-codec = { path="../md-codec", version="=0.38.0" }` | ✓ |
| CHANGELOG crate-prefix | `## md-cli [0.8.1] — 2026-06-21` | live head exactly that + `## md-codec [0.38.0]` | ✓ format correct |
| L7 underlying FOLLOWUP | RESOLVED @ `FOLLOWUPS.md:123` | `- **Status:** RESOLVED in md-codec-v0.35.0`@123 | ✓ |
| `cli_output_class.rs` homes | `byte_parity_advisory_lines`@23, `encode_text_emits_template`@74, `encode_json_emits_template`@94, `repair_emits_template`@241 | `:23` / `:74` / `:94` / `:241` | ✓ exact |
| H13 guard tests | `:197 :211 :221 :228 :247` | `lex_rejects_hardened_multipath_apostrophe`@197, `…_h_form`@211, `…_mixed_hardened`@221, `…_malformed_double_marker`@228, `lex_accepts_nonhardened_multipath`@247 | ✓ all five exact |
| `end_to_end_wsh_multi_template_only` | `:1939` | `:1939` | ✓ |
| tr tests (M10 stay-green) | `tr_with_and_v_verify_older_inheritance`, `tr_tap_leaf_bare_pk_on_wire`, `tr_multi_branch_three_leaf_right_unbalanced`, `tr_key_only` | `:1240` / `:1804` / `:1501` / `:1195` | ✓ (plan gives no line; all present) |
| manual `tr(@0)` belt | `42-md.md:166` | `# tr(@0)` comment in `md compile` worked-example @166 | ✓ |
| bughunt checkboxes | M5`:274` M2`:223` M10`:732` M11`:747` L4`:331` L7`:364` L19`:779` | all seven `### - [ ]` lines exact | ✓ |
| CI fmt gate | `.github/workflows/ci.yml:59` `cargo fmt --all --check` | `:59` `- run: cargo fmt --all --check` | ✓ |
| CI clippy gate | `ci.yml:47` `cargo clippy --workspace --all-targets -- -D warnings` | `:47` exact | ✓ |
| `CliError::BadXpub{i,why}` / `TemplateParse(String)` | used by M11 / M5 / M2 fixes | `error.rs:6` `TemplateParse(String)`, `:7` `BadXpub { i, why }`, `:21` `BadArg(String)` | ✓ constructible |

**No mis-cited or moved edit site.** All M5/M2/M10/M11/L4/L19/L7 line numbers are byte-exact against `836faf8`; toolkit manual `42-md.md:367-379` is byte-exact against `d6398b57`. The plan honored the grep-at-write-time rule and the STALE-LOCAL caveat.

**Independently re-verified mechanisms (all REPRODUCE @ `836faf8`):**
- **M5:** `wpkh(@0/<2;3>/0'/*)` — lexer (`:55`) match ends at `>` (g3=`2;3`, g4 empty); `/0'/*` unconsumed → `resolved.use_site_path.multipath=Some([2,3])`. Substitution (`:498`, `[0-9;]` strip) replaces `@0/<2;3>`→bare `XPUB`, leaves `/0'/*` literal → `wpkh(XPUB/0'/*)` parses single-path origin `/0'` NO multipath. Stitch (`:1915-1920`) takes `use_site_path` from lexer (`:1918`) and `tree` from substituted (`:1919`) with no cross-check → divergent card, exit 0. **The synthetic xpub is BARE (no embedded path; `synthetic_xpub_for` `:458-477` sets only version/depth/chaincode/pubkey)** — so the substituted `DescriptorPublicKey`'s multipath comes solely from the template suffix, which is exactly why the D4 count-compare detects the divergence (lexer `multipath_alts.len()=2` vs substituted-DPK multipath-steps=0). D4 is genuinely constructible + non-vacuous. ✓
- **M2:** `@255` lexes fine (`:60` `let i: u8 = caps[1].parse()`); `(255+1) as u8 == 0` (`:307-312`); `for i in 0..0` (`:313`) skips the density check; `by_i[&0]` (`:320`) panics when `@0` absent. ✓
- **M10:** `tr(` → `else` → MultiSig (`:1929-1931`) → depth-4 gate (`keys.rs:72-77`) rejects depth-3 BIP-86 xpub. ✓
- **M11:** `keys.rs:78-79` copies `bytes[13..78]` with no `PublicKey::from_slice`; file imports only `bitcoin::base58` — off-curve `bytes[45..78]` admitted. ✓
- **L4/L19/L7:** all confirmed; `is_wallet_policy()` `pub`; `WatchOnly` variant exists (cross-repo byte-parity-locked).

---

## Critical

**None.**

The M5 H13-preservation argument (§P3.4) is carried VERBATIM from the R0-GREEN spec §3.1.4, the group-3-validator-FIRST ordering is mandated, the residue check is constrained to text OUTSIDE `<…>`, the REJECT decision is funds-safe and correctly framed (md1/`UseSitePath` representability limit, NOT a BIP-389 prohibition), and the fused hardened+suffix test (H13-stays-rejected) is mandated. No finding rises to wrong-address / data-loss / silent-fail-open at the plan level. The two substantive defects (Important-1/Important-2) are tracking/process precision, not funds.

---

## Important

### I-1 — §P4.2 "flip 'open' → 'resolved'" presumes pre-filed descriptor-mnemonic FOLLOWUPS slugs that DO NOT EXIST; the slugs also do not match the bughunt-report ids

**Where:** plan §P4.2 (FOLLOWUP status flips), cross-referenced spec §10.

**Verified against `origin/main:design/FOLLOWUPS.md` (`836faf8`):** NONE of the six cycle-9 resolve-this-cycle slugs the plan lists are present as filed FOLLOWUP entries — grep for `multipath-not-last`, `u8-overflow-panic`, `bip86`, `secp256k1-point`, `mislabels-watch`, `epilog-stale` returns **zero cycle-9 entries** (the one `bip86` hit at `FOLLOWUPS.md:614` is an unrelated renamed-test reference). The only RESOLVED entry is the L7 *underlying* slug `md-codec-decode-with-correction-supports-non-chunked-md1` (`:123`), which the plan correctly says NOT to flip.

The seven findings live ONLY as entries in the toolkit bughunt report `constellation-bughunt-2026-06-20.md`, where their **ids differ from the plan's slug names:**
- M5 bughunt id = `lexer-substitution-divergence-multipath-not-last` (`:276`); plan = `md-cli-lexer-substitution-divergence-multipath-not-last` (extra `md-cli-` prefix).
- M2 = `placeholder-count-u8-overflow-panic` (`:225`) — matches ✓.
- M10 bughunt id = `w3-mdcli-01` (`:733`); plan = `tr-singlekey-bip86-depth3-false-reject`.
- M11 bughunt id = `w3-mdcli-04` (`:748`); plan = `parse-key-missing-secp256k1-point-check`.
- L4 bughunt id = `repair-advisory-mislabels-watch-only-as-keyless-template` (`:332`); plan = `repair-encode-advisory-mislabels-watch-only-as-keyless-template` (adds `-encode-`, folds L19 in).
- L19 bughunt id = `w3-mdcli-03` (`:780`, "sibling of L4"); plan folds it into the L4 slug.
- L7 = `repair-help-epilog-stale-rejects-nonchunked-claim` (`:365`) — matches ✓.

**Why Important (not Minor):** the memory rule `feedback_followup_status_discipline` ("verify 'open' status at decision time; flip in the shipping commit") that the plan invokes presumes the entries EXIST and are OPEN. They are not filed in descriptor-mnemonic at all, so the literal §P4.2 instruction ("flip to `resolved <SHA>`") is unexecutable as written and an implementer will either (a) waste time hunting for non-existent entries or (b) silently skip the tracking step, leaving the cycle's funds-fix (M5) with no FOLLOWUP audit trail in the repo where it ships. The plan's *real* tracking artifact is §P4.5 (tick the bughunt-report checkboxes) — which IS correct and complete — but §P4.2 conflates "tick the bughunt boxes" with "flip filed FOLLOWUPS."

**Required fix (pick one, state it):**
- **(a)** Reframe §P4.2 to "**FILE** the six resolve-this-cycle slugs as FOLLOWUP entries in `descriptor-mnemonic/design/FOLLOWUPS.md` with `Status: resolved <md-cli-v0.9.0 SHA>` **in the shipping commit**" (file-and-resolve in one commit, the normal pattern for a finding first formalized at ship time), AND reconcile the slug names to the bughunt-report ids (or explicitly note the plan's slugs are the canonical FOLLOWUP names superseding the bughunt working-ids — but pick consistent names so the audit trail cross-references). The NEW deferred slug `md1-post-multipath-fixed-path-derivation-steps` is correctly framed as filed-new already.
- **(b)** OR drop §P4.2's "flip" language entirely and rely solely on §P4.5 (tick the bughunt boxes) as the tracking mechanism, explicitly stating no descriptor-mnemonic FOLLOWUPS entries pre-exist or are created. (Weaker — loses the in-repo FOLLOWUP trail; (a) is preferred for the M5 funds fix.)

Either way: the §P4.2 slug names must be reconciled with the bughunt-report ids so a future reader can cross-reference finding → slug → ship commit.

### I-2 — L4 fix site binds the decoded descriptor as `_descriptor` (underscore-discarded); the plan's `descriptor.is_wallet_policy()` snippet will not compile as written without un-underscoring

**Where:** plan §P1.c fix snippet + §3.5; live `repair.rs:118`.

**Verified @ `836faf8`:** at `repair.rs:118` the decode result is bound `let (_descriptor, details) = match md_codec::decode_with_correction(&str_refs) { … }`. The leading underscore means the binding is currently **intentionally unused** — the L4 advisory site at `:156-159` has NO `descriptor` in scope. The plan's fix snippet (§P1.c, lifted from spec §3.5) writes `if descriptor.is_wallet_policy()` and asserts "these commands hold a decoded `Descriptor`." That assertion is *true for `encode.rs`* (live `:45` `let mut descriptor = parse_template(...)` — directly usable) but **NOT directly true for `repair.rs`**: the implementer MUST rename `_descriptor` → `descriptor` (and confirm no `unused_variables`/clippy regression elsewhere) before the snippet compiles.

**Why Important (not Minor):** the plan presents L4 as a drop-in three-site one-branch fold and explicitly claims the `Descriptor` is in scope at all three. For the repair site that is a per-call rename, not a drop-in — and because the repository's CI hard-gates `cargo clippy --workspace --all-targets -- -D warnings` (`ci.yml:47`, verified), an un-renamed `_descriptor` left in place would either fail to compile (if used) or, if the implementer instead introduces a *second* decode, duplicate work / risk divergence. Small but load-bearing for a clean first compile.

**Required fix:** in §P1.c, note that `repair.rs` binds `_descriptor` (`:118`) and the fix must un-underscore it to `descriptor` (encode.rs `:45` already binds `descriptor` usably); confirm the rename clears clippy `-D warnings`. (The fix remains correct and md-codec-untouched — this is a one-token implementation note the plan currently omits, asserting the opposite.)

---

## Minor

### Minor-1 — §P3.4 ordering precision: "group-3 validator at `:90-110`" vs the block at `:77-110`
The plan refers to the group-3 validator interchangeably as `:77-110` (the `if let Some(m)=caps.get(3)` block) and `:90-110` (the per-alt `.split(';').map(…)` loop). Live: the block opens at `:77`, the per-alt reject loop is `:90-107`, collect at `:108`, the block closes at `:110`. Both numberings are defensible; the residue-check-AFTER-the-block invariant (§P3.4(a)) is satisfied by placing the residue check after the `?` on the collect (`:108`) / after the block closes (`:110`). No correctness issue — keep both numbers but state "after the group-3 block closes at `:110`" as the precise insertion fence so the implementer cannot place the residue check mid-loop.

### Minor-2 — §P3.3 edit-2 (D4) must map substituted keys back to `@i` via `key_map`
The D4 cross-check compares per-`@i` `occ.multipath_alts.len()` to the substituted `DescriptorPublicKey`'s multipath-step count. The substituted descriptor keys synthetic xpubs to `@i` via `substitute_synthetic`'s returned `key_map: BTreeMap<String,u8>` (xpub-string → `i`), available in `parse_template` at `:1883`. The plan should name `key_map` as the join key (iterate the substituted descriptor's keys, look up `i` via `key_map`, compare to `occs[i].multipath_alts.len()`); it currently says "for each `@i` compare … the substituted key's multipath-step count" without naming the join. miniscript 13.0.0's `DescriptorPublicKey` exposes the multipath via `MultiXPub`/`is_multipath()` — the count is reachable. Constructible as specified; just name the join for the implementer.

### Minor-3 — §P4.4 belt check cites `42-md.md:166` for `tr(@0)`; live `tr(@0)` is a `# tr(@0)` comment in a `md compile` worked example
Confirmed present at `42-md.md:166` (`# tr(@0)` inside the `md compile 'pk(@0)' --context tap` example), so the belt check ("manual already lists `tr(@0)` → likely no new prose") holds. Note it is a `compile` example, not an `md encode` accepted-heads list — the M10 widening concerns `md encode`/parse classification, so the belt check is satisfied (no contradiction) but the citation is a compile example, not the encode accepted-heads prose. Cosmetic; the conclusion (no new manual prose needed for M10) stands.

### Minor-4 — `tr_key_only` line is `:1195`, not `:1196`
§P1.b / §P3 reference the existing tr tests; round-1 spec review cited `tr_key_only` at `:1196`; live is `:1195`. The plan does not cite a line for it (lists it by name), so no plan drift — noted for the implementer's grep.

---

## H13-PRESERVATION — does the PLAN preserve H13's reject? VERDICT: YES (verbatim + fused test mandated)

Traced §P3.4 against the live `lex_placeholders` (`:32-128`) and the five H13 guards:

- **§3.1.4 carried VERBATIM.** §P3.4 reproduces the spec's three load-bearing elements: (a) disjoint regions — M5 touches path text OUTSIDE `<…>`, never the group-3 capture `([^>]*)` (`:55`), the validator loop (`:90-110`), or the substitution strip class `[0-9;]` (`:498`); (b) group-3-validator-FIRST ordering — "H13's group-3 validator (`:90-110`) runs … and returns the typed 'hardened'/'not a bare unsigned integer' error **before** any M5 residue logic"; (c) the residue check placed strictly AFTER the group-3 validator block. §P3.4's "Implementation invariants this phase MUST honor" restates (a)/(b)/(c) as review-checkable: group-3 capture + validator loop + strip class **BYTE-IDENTICAL**, and `replace_all`/`captures_iter` unanchored semantics not changed to swallow the suffix. This matches the R0-GREEN spec §3.1.4 word-for-word.
- **REJECT decision carried (D1/D2):** §P3.2 — fail-closed typed `CliError::TemplateParse`; framed as an md1/`UseSitePath` representability limit (`make_use_site_path` reads only `multipath_alts` + `wildcard_hardened`), NOT a BIP-389 prohibition; canonicalize rejected (would need a wire/`UseSitePath` field → md-codec change → break the no-toolkit-pin invariant). Deferred-capability FOLLOWUP `md1-post-multipath-fixed-path-derivation-steps` filed. Correct and funds-safe.
- **RED tests mandated (§P3.5):** divergent `wpkh(@0/<2;3>/0'/*)` REJECTED; the **fused** `wsh(multi(2,@0/<0'';1>/0'/*,@1/<0'';1>/0'/*))` still errors with the **hardened/malformed** message (H13 before M5) — i.e. H13's `<0'';1>` reject STAYS RED; all five H13 guards (`:197/:211/:221/:228/:247`) re-asserted GREEN; normal multipath-last `wpkh(@0/<0;1>/*)` + `wsh(multi(2,…))` still build (`end_to_end_wsh_multi_template_only` `:1939` GREEN); `h`-in-origin `wpkh(@0/48h/0h/0h/<0;1>/*)` rejected; the D4 per-`@i` lexer-vs-substituted-DPK count cross-check.
- **M5 is genuinely LAST.** §0 phase order 1→2→3→4 with M5 in P3; disjointness table confirms P1 (M11/M10/L4/L19/L7) and P2 (M2 in `resolve_placeholders`) never touch the M5 regexes/validator; M2 lands FIRST so M5 builds on a bounded `n`. M5 cannot destabilize the earlier phases (different functions) and lands with all five H13 guards + the full suite loaded. ✓

**Caveat folded into the verdict (same as the spec R0):** "preserved" is provable at plan/design level and CONTINGENT on the implementation honoring residue-check-after-`:110` + byte-identical group-3/validator/strip-class + unchanged unanchored regex semantics — all three are mandated by §P3.4 and re-checked by the post-impl whole-diff gate (§Mandatory post-implementation, item 1). No Critical/Important here.

---

## M2 / M10 / M11 — bound, classify, point-check: VERDICT sound

- **M2:** §Phase-2 bounds `max >= 255` BEFORE `(max+1) as u8` (`:307-312`) → typed `TemplateParse` "at most @254", and replaces `by_i[&0]` (`:320`) / `by_i[&i]` (`:324`) with checked `.get(…).ok_or_else(…)?`. RED: `wpkh(@255/*)` panics today → typed reject. Boundary: `@254` accepts, `@255` rejects; `@0`-present `wsh(multi(2,@0/*,@255/*))` no silent `n=0`. Correct and RED-first (panic today). ✓
- **M10:** §P1.b classifies bare `tr(@i)` (no top-level `,`, no `{`) → SingleSig depth-3; over-accept guard keeps `tr(@0,{…})` / `tr(NUMS,multi_a(…))` → MultiSig depth-4; depth gate (`keys.rs:67-77`) kept strict. The §P1.b "SECOND consumer" note (synthetic xpub depth flip via `synthetic_xpub_for` `:458`/called `:506`) is documented as harmless/address-neutral, with `tr_key_only` pinning `ScriptCtx::MultiSig` explicitly so it does not break. RED: `tr(@0/<0;1>/*)` + depth-3 xpub rejected today → accepted; NEGATIVE over-accept guards present. Sound, no wrong-depth tr accepted. ✓
- **M11:** §P1.a inserts `bitcoin::secp256k1::PublicKey::from_slice(&bytes[45..78])` BEFORE the copy at `keys.rs:78`, mapping to `CliError::BadXpub{i,why}` — `secp256k1` already reachable (`template.rs:461`), no new dep. RED: off-curve (all-zero `bytes[45..78]`) accepted today → rejected; positive control `XPUB_DEPTH4` + real depth-3 still parse (a valid BIP-32 xpub's `bytes[45..78]` is on-curve → no valid-xpub break). Sound, RED-first. ✓

---

## L4 / L19 / L7 — advisory + help + cross-repo manual: VERDICT sound (modulo I-2)

- **L4/L19:** `is_wallet_policy()`-gated branch (`WatchOnly` if keyed, else `Template`) at three sites; regression home = existing `tests/cli_output_class.rs` (`byte_parity_advisory_lines`@23 stays GREEN — touches call-sites, not advisory strings; keyless `repair_emits_template`@241 / `encode_text_emits_template`@74 / `encode_json_emits_template`@94 kept + keyed assertions added). `encode.rs` `descriptor` in scope (`:45`); **`repair.rs` binds `_descriptor` (`:118`) — see I-2.** Otherwise sound. ✓
- **L7:** delete/rewrite the false "rejected with a wire-format error" sentence in `main.rs:241` `after_long_help`; KEEP the "ATOMIC SEMANTICS (multi-chunk)" note (live, accurate); no underlying-FOLLOWUP flip (already RESOLVED `:123`). RED: `md repair --help` no longer contains the false phrase. ✓
- **Cross-repo manual (P4.4):** correctly framed as DOCS-ONLY paired-PR DISCIPLINE, **NOT lint-gated** — verified against live `origin/master:docs/manual/tests/lint.sh` (the only `--help`-consuming step is 4/6 flag-coverage, `grep -oE -- '--[a-z…]+'`, flag-NAMES only; NO step diffs the `repair` epilog prose), so the lint passes regardless of the prose edit, needs **no `MD_BIN` v0.9.0 pin**, lands with the toolkit cycle-9 design-trail commit. This is the R0-GREEN spec's I-1 fold carried correctly. The M10 belt check (manual `tr(@0)` not contradicted) holds (Minor-3). ✓

---

## Repo convention (`cargo fmt` CI gate) — VERDICT: CLAIM CORRECT

**Verified against `origin/main:.github/workflows/ci.yml` (`836faf8`):** `:59` `- run: cargo fmt --all --check` (a hard `fmt` job gate), `:47` `- run: cargo clippy --workspace --all-targets -- -D warnings`, `:30-31` `cargo test --workspace --all-targets` + `--doc`. The plan's claim (§0 execution model) that descriptor-mnemonic CI enforces `cargo fmt --all --check` with NO mlock-style exemption is **CORRECT** — this repo, unlike the toolkit (where `mlock.rs` is fmt-exempt and `cargo fmt` is forbidden), has no fmt exemption. The plan correctly mandates `cargo fmt --all` → `cargo fmt --all --check` clean each phase, distinct from the toolkit rule. Getting this right avoids a red CI. ✓ Per-phase gate = full `cargo test -p md-cli` + clippy `-D warnings` + fmt-check — matches CI.

---

## SemVer / publish — VERDICT: correct

- md-cli **MINOR → 0.9.0** (M10 widens accepted input — BIP-86 `tr(@0)` now accepted — the additive driver; M5/M2/M11 rejects, L4/L19/L7 advisory/doc all PATCH-class riding the MINOR umbrella). `Cargo.toml:3` `0.8.1`→`0.9.0`. ✓
- md-codec **NO BUMP** — `=0.38.0` path-pin (`Cargo.toml:28`) UNCHANGED; all fixes md-cli-side; `is_wallet_policy()` already `pub`, secp256k1 already reachable. ✓
- **NO toolkit pin** — toolkit deps `md-codec` (lib) only, never `md-cli`. ✓
- Tag `descriptor-mnemonic-md-cli-v0.9.0` + `cargo publish -p md-cli` (precedent `58cc9ec` md-cli-0.8.0 md-cli-only release). md-codec NOT published (NO BUMP; `=0.38.0` already on crates.io → path-dep `version="=0.38.0"` resolves at publish). ✓
- CHANGELOG crate-prefixed `## md-cli [0.9.0]` entry; version-site grep ritual flagged. ✓
- **GUI schema_mirror NOT triggered** (no clap flag/subcommand/dropdown add/rename/remove — M10 widens descriptor-string input not a flag; M5/M2/M11 parse-internal rejects; L4/L19 advisory; L7 edits prose inside existing `after_long_help`). Sibling-codec companions: none (md-codec untouched). ✓

---

## TDD integrity — RED-first + non-vacuous: confirmed

- **M5 divergent:** `wpkh(@0/<2;3>/0'/*)` genuinely RED-first — verified NO current test covers multipath-not-last (only `origin_path_extracted` `:170` is positional and puts multipath last). The divergence (`multipath=Some([2,3])` over no-multipath tree) is assertable TODAY. Non-vacuous. ✓
- **M5 H13-stays-rejected fused test:** `wsh(multi(2,@0/<0'';1>/0'/*,…))` — H13's validator at `:90-110` fires on `<0'';1>` returning the "hardened"/"not a bare unsigned integer" `?` before any residue logic; the assertion (`msg.contains("hardened")`/malformed, NOT the suffix message) is genuinely discriminating and proves ordering. Non-vacuous. ✓
- **M5 D4 belt:** the synthetic xpub is BARE (verified `synthetic_xpub_for` embeds no path), so the substituted `DescriptorPublicKey` for `wpkh(XPUB/0'/*)` carries NO multipath while the lexer recorded 2 alts → the count-compare DETECTS the divergence. Constructible, not a vacuous always-true assertion. ✓
- **M2:** panic→typed reject, RED-first (panics today). ✓  **M10:** rejected-today→accepted, RED-first + over-accept negatives. ✓  **M11:** off-curve accepted-today→rejected, RED-first. ✓  **L4/L19:** always-`Template`-today→branch, RED-first in `cli_output_class.rs`. ✓  **L7:** false phrase present-today→absent, RED-first. ✓

---

## Verdict

**PLAN R0 ROUND 1: 0C / 2I — RED**

- **Critical:** none.
- **Important I-1:** §P4.2 "flip 'open' → 'resolved'" presumes pre-filed descriptor-mnemonic FOLLOWUPS slugs that DO NOT EXIST (verified zero cycle-9 entries in `FOLLOWUPS.md`@`836faf8`); the plan's slug names also diverge from the bughunt-report ids (`w3-mdcli-01/03/04` + prefix/fold differences). Reframe to FILE-and-resolve the slugs in the shipping commit (or rely solely on §P4.5 bughunt-box ticks) AND reconcile the slug names.
- **Important I-2:** the L4 fix site `repair.rs:118` binds the decoded descriptor as `_descriptor` (underscore-discarded, currently unused); the plan's `descriptor.is_wallet_policy()` snippet asserts the `Descriptor` is in scope at all three L4/L19 sites, but for `repair.rs` the implementer must un-underscore `_descriptor`→`descriptor` for a clean compile under CI's `clippy -D warnings`. Add the one-token implementation note.

Fold I-1 + I-2 (optionally Minors 1-4), persist this review, re-dispatch PLAN R0 round 2. The M5 H13-preservation core (§P3.4) is carried VERBATIM and is R0-sound — do NOT weaken it on the fold.

**Dispatch-item verdicts:**
- **(a) Does the plan preserve H13's reject (M5 §3.1.4 verbatim + fused test)?** **YES** — §P3.4 carries spec §3.1.4 word-for-word (disjoint regions, group-3-validator-FIRST ordering, byte-identical group-3 capture/validator/`[0-9;]` strip, residue check only OUTSIDE `<…>`, residue check AFTER the validator block); the fused `<0'';1>/0'/*` test (H13 stays rejected, hardened/malformed message not suffix) + all five H13 guards GREEN + valid multipath-last preserved + the constructible D4 count-compare are all mandated. M5 is genuinely LAST (P3).
- **(b) Is the `cargo fmt` CI-gate claim correct?** **YES** — `cargo fmt --all --check` is a hard CI gate at `ci.yml:59` (+ clippy `-D warnings` `:47`); descriptor-mnemonic has NO mlock-style fmt exemption; the plan correctly mandates `cargo fmt --all` per phase, distinct from the toolkit rule.
- **(c) Is the cross-repo manual edit correctly framed (docs-only discipline)?** **YES** — §P4.4 frames `42-md.md:367-379` as a docs-only paired-PR DISCIPLINE edit, NOT lint-gated (verified against live `lint.sh`: flag-NAMES-only flag-coverage, no prose diff), landing with the toolkit cycle-9 design-trail commit, no `MD_BIN` pin. Correct.
