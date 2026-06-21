# cycle-9 spec — R0 review, round 1

**Spec:** `design/BRAINSTORM_cycle9_mdcli_parser.md` (md-cli lexer/parser cluster: M5 funds + M2/M10/M11 + L4/L7/L19)
**descriptor-mnemonic source SHA:** `836faf8` (`836faf87c3d82b119a9f0f5c6589a7db1f8613a4`; md-codec 0.38.0 / md-cli 0.8.1)
**mnemonic-toolkit manual SHA:** `origin/master` `79e3387` (recon cited `8d2fe50`; spec §0 cited `8d2fe50` — both stale vs `79e3387`; the false-prose lines verified below against live `79e3387`)
**Reviewer:** opus software architect (mandatory R0 gate — NO code until 0C / 0I)
**Date:** 2026-06-21

All citations independently re-grepped against the BYTES of `origin/main:<path>` (descriptor-mnemonic) and `origin/master:<path>` (toolkit). Focus per the dispatch: M5 funds-fix + the H13 hardened/malformed-multipath REJECT preservation.

---

## Citation audit (vs `836faf8` / toolkit `79e3387`)

| Citation | Spec says | Live | Verdict |
|---|---|---|---|
| M5 lexer regex | `template.rs:55` | 55 (`@(\d+)((?:/\d+'?)*)(?:/<([^>]*)>)?(/\*(?:'|h)?)?`) | ✓ |
| M5 substitution regex | `template.rs:498` | 498 (`@(\d+)((?:/\d+'?)*)(?:/<[0-9;]+>)?(?:/\*(?:'|h)?)?`) | ✓ |
| M5 stitch (`parse_template`) | `:1874-1922` | `fn parse_template` @1874; `Ok(Descriptor{…})` block ends ~1925 | ✓ (range correct) |
| M5 `use_site_path:` field annot. | `:1916` | **1918** | off-by-2 (Minor) |
| M5 `tree,` field annot. | `:1915` | **1919** | off-by-4 + WRONG ORDER (see Minor-1) |
| M2 n-cast | `:307-312` | 307 (`let n = (by_i … +1) as u8`) | ✓ |
| M2 density loop / panic-index | `:313` / `:320` / `:324` | 313 / 320 / 324 | ✓ |
| M10 classifier (`ctx_for_template`) | `:1925-1932` | 1925 | ✓ |
| M10 depth gate | `keys.rs:67-77` | 67 (`let depth = bytes[4]`) … 77 | ✓ |
| M11 payload copy (no point check) | `keys.rs:78-79` | 78-79 (`copy_from_slice(&bytes[13..78])`) | ✓ (pubkey = `bytes[45..78]`) |
| L4 repair advisory | `repair.rs:156-159` | 156-159 (unconditional `Template`) | ✓ |
| L19 encode JSON / text | `encode.rs:73-76` / `:110-113` | both unconditional `Template` | ✓ |
| L7 stale epilog | `main.rs:241` | 241 (`after_long_help` string carries the false sentence) | ✓ |
| L7 toolkit-manual mirror | `42-md.md:367-379` | heading @367, "Non-chunked…" @371, "rejected with a wire-format error" @373 | ✓ (prose spans 367-379) |
| `is_wallet_policy()` | md-codec `encode.rs:50-52` | 50 (`matches!(&self.tlv.pubkeys, Some(v) if !v.is_empty())`), `pub` | ✓ |
| M11 secp256k1 already-dep | `template.rs:461` | 461 (`use bitcoin::secp256k1::{Secp256k1, SecretKey}`) | ✓ |
| L7 FOLLOWUP resolved | `FOLLOWUPS.md:124` | RESOLVED @ **123** | off-by-1 (Minor) |
| `OutputClass::Template` text | `output_advisory.rs` (no full path) | `crates/md-cli/src/output_advisory.rs:35` | ✓ (note: file is at crate root `src/`, NOT `src/cmd/`) |

**Verified mechanisms (all REPRODUCE @ `836faf8`):** M5 lexer truncates at `>` for `wpkh(@0/<2;3>/0'/*)` → `multipath=Some([2,3])` over a no-multipath tree (no cross-check between `resolved.use_site_path` and the substituted `tree`). M2 `(255+1) as u8 == 0` → `0..0` density skip → `by_i[&0]` panic (substitution's `caps[1].parse().unwrap_or(0)` parses 255 fine; the wrap is solely in `resolve_placeholders`). M10 `tr(` → `else` → `MultiSig` → depth-4 gate rejects depth-3 BIP-86 xpub. M11 no `PublicKey::from_slice`; only `use bitcoin::base58`. L4/L19/L7 confirmed. `address.rs:23` guards `WatchOnly` behind `is_wallet_policy()` (the sound "model to mirror"). `WatchOnly` variant EXISTS in the (cross-repo byte-parity-locked) `OutputClass` enum → L4/L19 fix is constructible.

---

## Critical

**None.**

The funds-fix decision (M5 REJECT) is correct and funds-safe, and the H13-preservation argument is sound *given the ordering constraint the spec already names as mandatory*. No finding rises to wrong-address / data-loss / silent-fail-open at the spec level. (The one substantive defect — the L7 lint-mechanism mis-description, Important-1 — is doc/process, not funds.)

---

## Important

### I-1 — L7 manual-mirror lockstep is justified by a FALSE lint mechanism; the stated v0.9.0-binary-pin sequencing is unnecessary and misleading

**Where:** spec §8 step 3, §9 ("L7" test bullet), §11 D10.

**Claim under review:** "The manual `lint.sh` runs `$MD_BIN repair --help` against the chapter; for it to pass, the toolkit manual edit must use the **v0.9.0** `md` binary's new epilog text. Sequence: land md-cli text → build the v0.9.0 `md` binary → update + lint the toolkit manual with `MD_BIN=<v0.9.0 md>`."

**Verified against `origin/master:docs/manual/tests/lint.sh` (`79e3387`):** the only `--help`-consuming step is **4/6 flag-coverage**, which runs `eval "$MD_BIN repair --help"`, extracts **flag NAMES only** (`grep -oE -- '--[a-z][a-z0-9-]+'`), and asserts each flag name appears *somewhere* in `42-md.md`. The remaining steps are markdownlint (style), cspell (spelling), lychee (links), glossary-coverage, index-bidirectional. **NO step diffs the `after_long_help` PROSE against the manual.** Therefore:

1. The lint will PASS whether or not the false "rejected with a wire-format error" prose is corrected — there is **no automated lockstep gate** on epilog prose (consistent with CLAUDE.md: the manual-mirror invariant is a *paired-PR discipline*, a *lagging* gate; the flag-name lint is the only mechanical check and it does not cover prose).
2. The spec's "for the lint to pass you must build the v0.9.0 binary and run `MD_BIN=<v0.9.0 md>`" is **factually wrong** — the epilog rewrite removes no flag, so flag-coverage is unaffected; the manual edit needs no binary-pin and no v0.9.0-binary sequencing to pass lint.

**Why this is Important (not Minor):** the spec elevates an unnecessary cross-repo binary-build sequencing step (§8.3) to a hard requirement on a non-existent dependency. An implementer following §8 will block the toolkit docs commit on building/pinning a v0.9.0 `md` binary that the lint never consults — wasted work and a false sense of gate coverage. Worse, the inverse risk: because the spec asserts the lint *enforces* the prose match, a reader may assume the lockstep is *gated* and under-prioritize the (actually discipline-only, easy-to-miss) paired edit.

**Required fix:** Reframe L7's manual lockstep accurately:
- The toolkit `42-md.md:367-379` edit IS required — by the CLAUDE.md manual-mirror **discipline** (any help-surface prose change mirrors into the manual in lockstep), NOT by `lint.sh`.
- It is a docs-only paired edit needing **no** v0.9.0-binary build and **no** `MD_BIN` pin to pass lint (flag-coverage extracts flag names only; the `repair` flag set is unchanged).
- Keep the paired-PR fallback. Drop / correct the "must build v0.9.0 md → lint with MD_BIN=<v0.9.0 md>" sequencing in §8.3 and the §9 "L7 Manual lockstep: lint.sh with MD_BIN=<v0.9.0 md> passes" claim (lint passes regardless of the prose edit; the meaningful check is a manual *human/PR* review of the corrected prose). Sanity-checking flag-coverage still passes (it does, trivially) is fine to keep.

This is the dispatch's item (c): the L7 manual-mirror lockstep **edit** is covered (§8.2, D10 require it), but the **justification/mechanism is materially wrong** and must be corrected before the plan lifts it.

---

## Minor

### Minor-1 — M5 stitch pseudocode reorders/mis-numbers the `Descriptor` fields
§3.1.1's pseudocode shows `use_site_path` (`:1916`) then `tree` (`:1915`) — but live order is `n`(1915-ish via `resolved.n`), `path_decl`, `use_site_path`(**1918**), `tree`(**1919**), `tlv`; and the comment "`tree` ← :1915 from SUBSTITUTED" is below the `use_site_path` line yet cites a *lower* line number than `use_site_path`'s `:1916`. Cosmetic (the mechanism narrative is correct), but fix the two line annotations to `use_site_path: :1918`, `tree: :1919` so the plan lifts live numbers.

### Minor-2 — D4 (`parse_template` cross-validation belt) is under-specified; `tree` carries no directly-comparable multipath field
The multipath lives in `resolved.use_site_path` (lexer) and `use_site_path_overrides`; the `tree` built by `walk_root` from the substituted `MsDescriptor` records tag/key_index/leaf shape (`Body::Tr/KeyArg/…`) — it does **not** carry a per-key "multipath alts" field to diff against the lexer view. So "assert lexer-view == substituted-structural-view" (D4) is not a one-liner; a real implementation must re-derive the multipath count from the substituted `DescriptorPublicKey`(s) and compare to `occ.multipath_alts`. The spec correctly labels edit-1 (residue reject) as "the real fix" and D4 as belt-and-suspenders, and §9 hedges D4's test ("if reachable via a crafted input"). Acceptable at brainstorm altitude, but the plan MUST either (a) concretize D4 to a constructible check (compare lexer `multipath_alts.len()` per `@i` to the substituted key's derivation-path count) or (b) explicitly downgrade D4 to "residue reject is sufficient; D4 deferred" rather than ship a vacuous always-true assertion. State which at plan time.

### Minor-3 — M10 analysis omits the SECOND consumer of `ctx_for_template`'s output (`synthetic_xpub_for` depth)
`ctx` from `ctx_for_template` feeds BOTH `parse_key`'s depth gate (`encode.rs:38`) AND `substitute_synthetic → synthetic_xpub_for(i, ctx)` (`template.rs:458-464`), which builds the synthetic xpub at depth 3 (SingleSig) / depth 4 (MultiSig). Flipping bare `tr(@0)` to SingleSig therefore also makes its synthetic xpub depth-3. This is internally consistent and address-neutral (depth is advisory; synthetic is discarded after `key_map`), and the existing `tr_key_only` test (`:1196`) pins `ScriptCtx::MultiSig` *explicitly* (not via `ctx_for_template`) so it does not break — but the spec's M10 section discusses only the depth-gate consumer. Add one sentence confirming the synthetic-xpub-depth change is harmless, and keep §9's end-to-end `tr(@0)` SingleSig assertion + the `md encode --key @0=<depth-3 xpub>` round-trip as the regression evidence. No code-correctness issue; completeness only.

### Minor-4 — existing `tests/cli_output_class.rs` is the L4/L19 regression home and isn't named
`crates/md-cli/tests/cli_output_class.rs` already asserts the `Template`/`WatchOnly` advisory lines (incl. cross-repo `byte_parity_advisory_lines`). The L4/L19 fix changes which class `repair`/`encode` emit; the plan should extend THIS test file (assert `WatchOnly` on keyed input, `Template` on keyless) and confirm `byte_parity_advisory_lines` stays GREEN (the fix touches call-sites, not the advisory strings, so parity is preserved — but state it). Spec §9 describes the assertions but doesn't point at the existing file.

### Minor-5 — `output_advisory.rs` path + minor line drifts
Spec §3.5 names "`output_advisory.rs`" (no path); it lives at `crates/md-cli/src/output_advisory.rs` (crate root, not `src/cmd/`). FOLLOWUP "RESOLVED in md-codec-v0.35.0" is at `FOLLOWUPS.md:123` (spec says `:124`). `origin_path_extracted` test is at `:170` (spec §3.1.1 says `:170`; recon said `:171`). All cosmetic; fix at plan time for grep-verifiability.

---

## H13-preservation: PROVABLY preserved? — VERDICT: YES, conditional on the ordering the spec already mandates

Traced against the live `lex_placeholders` (`:32-128`) and the five H13 guards (`:197/:211/:221/:228/:247`):

- **Disjoint regions:** H13 governs the group-3 `<…>` body (permissive `[^>]*` capture @ `:55` + strict in-loop validator @ `:77-110` rejecting `'`/`h`/non-integer alts) plus the substitution strip class `[0-9;]` (@ `:498`, the C1 revert). M5's residue reject fires on path text **outside** `<…>` (a residual `/NUM…` after `>`, or an `h`-bearing origin step). The spec's edit-1 does NOT widen, relax, or touch the group-3 capture, the validator loop, or the strip class. ✓
- **Ordering is load-bearing AND the spec names it:** the existing five H13 tests all put the multipath **last** (e.g. `@0/<0';1'>/*`), so none has a suffix — they are untouched by any residue check regardless of order. The ordering only matters for a FUSED input (hardened/malformed body + suffix, e.g. `@0/<0'';1>/0'/*`). For `lex_rejects_hardened_multipath_apostrophe` the assertion is `msg.contains("hardened")`; if a residue check fired *first* on a fused input it would emit the suffix message and (for a hardened+suffix fusion) violate that contract. **The spec §3.1.4 explicitly requires the group-3 validator to run FIRST and adds the fused regression test (`wsh(multi(2,@0/<0'';1>/0'/*,…))` must still error "hardened/malformed", not "suffix") in §9.** Given the current code returns the group-3 validator's `?` before reaching any later loop logic, placing the residue check AFTER the group-3 block (as the spec's "Preferred (i)" wording implies) satisfies this. ✓
- **No over-rejection of valid multipath-last:** `@0/<0;1>/*` has no post-`>` residue (the `/*` is group-4) → residue check inert → `lex_accepts_nonhardened_multipath` (`:247`), `multipath_arity_2/3`, `end_to_end_wsh_multi_template_only` (`:1939`) stay GREEN. ✓

**Caveat folded into the verdict:** "PROVABLY preserved" holds *at the spec/design level and contingent on the implementation honoring the stated ordering (residue check strictly after the group-3 validator) and the `replace_all`/`captures_iter` unanchored semantics not being changed to swallow the suffix.* The plan must (a) place the residue check after group-3 validation, (b) keep group-3/strip-class byte-identical, (c) run all five H13 tests + the new fused test + the full `cargo test -p md-cli` suite as the gate. The spec mandates all three. No Critical/Important here — the preservation argument is correct; this is the post-impl whole-diff review's #1 focus.

---

## M5 canonicalize-vs-reject: funds-safe + BIP-389-justified? — VERDICT: YES

- **BIP-389 framing is CORRECTED and accurate.** The spec (§3.1.2, D2) explicitly repudiates the recon's "multipath must be last" and states BIP-389 *permits* post-multipath fixed steps ("Followed by zero or more /NUM... path elements"). The reject is justified NOT as a BIP-389 prohibition but as an **md1 + md-cli `UseSitePath` representability limit** — `make_use_site_path` (`:339-356`) models the multipath as the single final pre-wildcard step (`multipath` + `wildcard_hardened`); there is no field for post-multipath fixed steps. A descriptor with fixed steps after `<…>` cannot be faithfully represented, and the current code does the worst thing (silently emits a divergent card). This framing is correct and matches the live structs.
- **REJECT (fail-closed) is the funds-safe call.** Canonicalize (option a) would require a new wire/`UseSitePath` field → md-codec change → breaks this cycle's "md-codec UNTOUCHED / no toolkit pin" invariant — out of scope, and silently accepting an exotic, un-restorable shape is exactly the cycle-1 H13 anti-pattern. REJECT with a typed `CliError::TemplateParse` (loud, actionable, exit≠0) is the precedent-matching, funds-safe choice. The deferred-capability FOLLOWUP (`md1-post-multipath-fixed-path-derivation-steps`) is correctly filed for genuine future demand.
- **The `h`-in-origin sub-case** (`@0/48h/…` → unconsumed `/48h` residue → malformed `XPUBh/…`) is the same family and is covered by the same residue reject (§3.1.3, §9). ✓

Decision is funds-safe and BIP-389-accurate. No finding.

---

## SemVer / publish / lockstep — confirmed

- md-cli **MINOR → 0.9.0** (M10 widens accepted input: BIP-86 `tr(@0)` now accepted) — correct driver. M5/M2/M11 are rejects (no newly-accepted input), L4/L19/L7 advisory/doc — all PATCH-class riding the MINOR umbrella. ✓
- md-codec **NO BUMP** (all 7 fixes md-cli-side; `is_wallet_policy()` already public; secp256k1 already a reachable dep). ✓
- **NO toolkit pin** — toolkit deps `md-codec` (lib) only, never `md-cli`. ✓ (The toolkit `42-md.md` edit is docs-only, not a pin.)
- Tag `descriptor-mnemonic-md-cli-v0.9.0` + `cargo publish -p md-cli`; precedent `58cc9ec`. ✓
- **GUI `schema_mirror`: NOT triggered** — none of the 7 add/rename/remove a clap flag, subcommand, or dropdown value. M10 widens descriptor-string input (not a flag); M5/M2/M11 are parse-internal rejects; L4/L19 advisory; L7 edits prose inside an existing `after_long_help` (flag NAMES unchanged). ✓
- **Manual mirror: TRIGGERED by L7 only** — by discipline, NOT by the flag-coverage lint (see I-1). M10/M2/M5/M11/L4/L19 change no `--help` text → no manual delta. The plan's "sanity-check `md encode` accepted-heads wording isn't contradicted" for M10 is a reasonable belt check. ✓

---

## TDD integrity — RED-first, adequate (one home-file gap)

- **M5:** `wpkh(@0/<2;3>/0'/*)` divergent-today → rejected; fused hardened+suffix still "hardened" (H13 ordering guard); five H13 guards re-asserted GREEN; positive multipath-last preserved; `h`-in-origin residue rejected. Genuinely RED-first (no current test covers multipath-not-last — verified: only `origin_path_extracted` @ `:170` is positional and puts multipath last). ✓
- **M2:** `wpkh(@255/*)` panic→typed reject; `@254` boundary still accepts; `@0`-present `@255` case. RED-first (panic today). ✓
- **M10:** `tr(@0/<0;1>/*)` + depth-3 xpub rejected-today→accepted; over-acceptance negatives (`tr(@0,{…})`, `tr(NUMS,multi_a(…))` still MultiSig); existing tr tests stay GREEN. RED-first. ✓ (add Minor-3's synthetic-depth note.)
- **M11:** off-curve `bytes[45..78]` accepted-today→rejected `BadXpub`; real depth-3/4 xpubs still parse. RED-first. ✓
- **L4/L19:** keyed→`WatchOnly`, keyless→`Template`. RED-first (today always `Template`). ✓ — but the home file (`tests/cli_output_class.rs`) is unnamed (Minor-4).
- **L7:** `--help` no longer contains "rejected with a wire-format error". ✓ The "manual lint with MD_BIN=<v0.9.0 md> passes" bullet is mechanically true but VACUOUS as a lockstep proof (I-1) — replace with "the corrected prose is present in `42-md.md` (human/PR-verified); flag-coverage lint passes (unchanged flag set)".

---

## Verdict

**R0 ROUND 1: 0C / 1I — RED**

- **Critical:** none.
- **Important:** I-1 — L7 manual-mirror lockstep is justified by a non-existent lint enforcement mechanism (flag-coverage extracts flag NAMES only; no step diffs the `repair` epilog prose), and the spec's "build v0.9.0 md → lint with `MD_BIN=<v0.9.0 md>` for it to pass" sequencing (§8.3, §9, D10) is therefore false/misleading. The manual EDIT is required (by discipline), but reframe its justification and drop the binary-pin lint dependency.

Fold I-1 (correct §8.3 / §9-L7 / D10 to "manual edit required by the mirror discipline, not by lint; docs-only, no v0.9.0-binary build/pin needed for lint-pass"), optionally fold Minors 1-5, then re-dispatch R0 round 2.

**Dispatch-item verdicts:**
- (a) Does M5's fix PROVABLY preserve H13's reject? **YES** — disjoint input region, ordering load-bearing and explicitly mandated, fused regression test added; contingent on the plan honoring residue-check-after-group-3-validator + byte-identical group-3/strip-class (all required by the spec).
- (b) Is the M5 canonicalize-vs-reject decision funds-safe + BIP-389-justified? **YES** — REJECT (fail-closed) is correct; framed accurately as an md1/`UseSitePath` representability limit (not a BIP-389 prohibition; BIP-389 permits post-multipath steps).
- (c) Is the L7 manual-mirror lockstep covered? **EDIT: yes (required, cited @ `42-md.md:367-379`). MECHANISM: NO — wrongly attributed to the lint; corrected by I-1.**
