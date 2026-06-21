# cycle-9 PLAN — R0 review, round 2

**Plan-doc:** `design/IMPLEMENTATION_PLAN_cycle9_mdcli_parser.md` (executes the R0-GREEN `BRAINSTORM_cycle9_mdcli_parser.md`).
**Round 1:** `0C / 2I — RED` (`design/agent-reports/cycle9-plan-r0-round1-review.md`). I-1 (§P4.2 invented/non-existent FOLLOWUPS-flip) + I-2 (`repair.rs:118` `_descriptor` rename). Both folded.
**descriptor-mnemonic source SHA:** `836faf8` (`836faf87c3d82b119a9f0f5c6589a7db1f8613a4`; md-codec 0.38.0 / md-cli 0.8.1) — independently re-verified `origin/main` == `836faf8`; local `main` = `54dd765a11d490dc3d8dec2c842dae718bd3ef2b` (STALE, as the plan's §0 caveat warns).
**mnemonic-toolkit SHA (manual mirror + bughunt report):** `origin/master` `d6398b57`.
**Reviewer:** opus software architect (mandatory R0 gate — NO code until 0C / 0I).
**Date:** 2026-06-21

Scope of round 2: verify the two folds RESOLVE I-1/I-2 without introducing new drift; re-confirm the funds-critical M5 §P3.4 H13-preservation core was NOT weakened (the fold claims it only strengthened the insertion-fence precision); confirm no new Critical/Important. Every claim re-grepped against the BYTES of `git show origin/main:<path>` (descriptor-mnemonic) and `git show origin/master:<path>` (toolkit) at this write.

---

## Fold verification (vs `836faf8` / toolkit `d6398b57`)

### I-1 — §P4.2 FOLLOWUP reframe — RESOLVED

| Round-1 defect | Fold claim | Live verification | Verdict |
|---|---|---|---|
| §P4.2 instructed flipping pre-existing descriptor-mnemonic FOLLOWUPS that do not exist | "ZERO pre-filed cycle-9 FOLLOWUP entries to flip … that instruction is dropped" (plan `:244`) | `git show origin/main:design/FOLLOWUPS.md \| grep -E 'multipath-not-last\|u8-overflow-panic\|bip86\|secp256k1-point\|mislabels-watch\|epilog-stale\|w3-mdcli'` → only `:614` (unrelated renamed-test `bip86` ref). Zero cycle-9 entries. | ✓ premise confirmed |
| Canonical completion record | "= the TOOLKIT bughunt-report box ticks (§P4.5)" (plan `:246`) | §P4.5 ticks 7 `### - [ ]` lines, all byte-exact in the local report: M5`:274` M2`:223` M10`:732` M11`:747` L4`:331` L7`:364` L19`:779` | ✓ all 7 exact |
| Slug names reconciled to report ids | M5=`lexer-substitution-divergence-multipath-not-last`, M2=`placeholder-count-u8-overflow-panic`, M10=`w3-mdcli-01`, M11=`w3-mdcli-04`, L4=`repair-advisory-mislabels-watch-only-as-keyless-template`, L19=`w3-mdcli-03`, L7=`repair-help-epilog-stale-rejects-nonchunked-claim` (plan `:247-253`) | All 7 ids present in the local bughunt report at the cited lines (`:276/:225/:733/:748/:332/:780/:365`) AND in `origin/master` (offset by the report's working-tree-only status). | ✓ ids exist, citations correct |
| Filing NEW RESOLVED entries optional | "(b) OPTIONAL — file NEW RESOLVED entries … NOT a required flip … each NEW entry created with `Status: resolved <SHA>` in the shipping commit" (plan `:255`) | Correct framing: file-and-resolve in one commit is the normal pattern for a finding first formalized at ship. | ✓ |
| L7 underlying slug left RESOLVED | "(c) … `md-codec-decode-with-correction-supports-non-chunked-md1` is already `RESOLVED in md-codec-v0.35.0` (`FOLLOWUPS.md:123`); no flip, no edit" (plan `:257`) | `FOLLOWUPS.md:123` = `- **Status:** RESOLVED in md-codec-v0.35.0` (byte-exact). | ✓ |
| No surviving invented slug names | round-1 invented `md-cli-lexer-substitution-divergence-multipath-not-last` / `tr-singlekey-bip86-depth3-false-reject` / `parse-key-missing-secp256k1-point-check` / `repair-encode-advisory-mislabels-…` | `grep -iE 'md-cli-lexer-substitution\|tr-singlekey-bip86\|parse-key-missing-secp256k1\|repair-encode-advisory'` over the plan → **zero hits.** | ✓ removed |
| No surviving "flip open→resolved" of pre-existing entries | — | The only "flip" tokens in the plan are: `:99`/`:292` M10 classifier "Flipping bare `tr(@0)`→SingleSig" / "synthetic_xpub_for depth flip" (semantic depth-flip, unrelated); `:133`/`:257`/D9`:305` "No FOLLOWUP status flip" (correctly asserting NO flip); `:244` the reframe ("no … flip … that instruction is dropped"). NO instruction to flip a pre-existing entry survives. | ✓ |

**I-1 RESOLVED.** §P4.2 no longer presumes pre-existing entries; the canonical record is the bughunt-box ticks (§P4.5), slug names are reconciled to the bughunt-report ids, new RESOLVED entries are explicitly optional, and L7's underlying `:123` slug is left RESOLVED. The new deferred slug `md1-post-multipath-fixed-path-derivation-steps` is correctly framed as a genuinely-new filing (plan `:259`), not a flip.

### I-2 — `repair.rs:118` `_descriptor` rename — RESOLVED

| Round-1 defect | Fold | Live verification | Verdict |
|---|---|---|---|
| L4 snippet `descriptor.is_wallet_policy()` won't compile at `repair.rs` because the decode binds `_descriptor` (underscore-discarded) | New §P1.c "Implementation note (R0-round-1 I-2)" (plan `:121`): "`repair.rs:118` … `let (_descriptor, details)` … requires the binding be **un-underscored: rename `_descriptor` → `descriptor` AND use it** … `encode.rs:45` already binds `descriptor` usably — only `repair.rs` needs the rename. Do NOT introduce a second decode." | `repair.rs:118` (exact) = `let (_descriptor, details) = match md_codec::decode_with_correction(&str_refs) {`. `encode.rs:45` (exact) = `let mut descriptor = parse_template(args.template, &parsed_keys, &parsed_fps)?;`. | ✓ both exact |
| clippy `-D warnings` rationale | "(An underscore-prefixed var used in an expression, OR an unused non-underscore var, both trip CI's `cargo clippy --workspace --all-targets -- -D warnings`; rename-and-use clears both.)" | `ci.yml:47` (exact) = `- run: cargo clippy --workspace --all-targets -- -D warnings`. Rationale is technically correct: using a `_`-prefixed binding is an established clippy/rustc smell, and an unused non-`_` binding trips `unused_variables`; rename-and-use is the only clean path. | ✓ |

**I-2 RESOLVED.** The plan now names the exact `repair.rs:118` `_descriptor` binding, mandates rename-and-use, correctly scopes the rename to `repair.rs` only (`encode.rs:45` already usable), forbids a second decode, and cites the clippy gate. The fold is byte-accurate against live source.

### Minor folds — all verified

| Round-1 Minor | Fold | Live | Verdict |
|---|---|---|---|
| **M-1** insertion-fence precision | §P3.4 invariant (a) (plan `:217`): "residue check placed strictly **AFTER the group-3 block closes at `:110`** (block opens `:77`, per-alt reject loop `:90-107`, collect `:108`, block closes `:110`) — i.e. insert after the `?` on the collect / after the block-close brace at `:110`, so it can NEVER land mid-loop (Minor-1 fence)" | Live loop: `if let Some(m) = caps.get(3)` opens `:77`; `.split(';').map(…)` reject loop spans the per-alt body; `.collect::<Result<Vec<_>, _>>()?` ends the block; `else { Vec::new() }` closes `:110`; `out.push(…)` follows. Geometry exact. | ✓ |
| **M-2** D4 join `key_map` | §P3.3 edit-2 (plan `:203`): "map each substituted key back to its `@i` via the **`key_map: BTreeMap<String,u8>`** … that `substitute_synthetic` returns and `parse_template` binds at `template.rs:1883`" | `template.rs:1883` (exact) = `let (substituted, key_map) = substitute_synthetic(template, ctx)?;`. `substitute_synthetic` builds + returns `key_map: BTreeMap<String,u8>` (xpub→`i`). `PlaceholderOccurrence.multipath_alts: Vec<u32>` @ `:27`. | ✓ |
| **M-3** `42-md.md:166` `tr(@0)` is a compile example | §P4.4 belt check (plan `:274`): "The `tr(@0)` at `42-md.md:166` is a `# tr(@0)` comment inside an `md compile 'pk(@0)' --context tap` worked EXAMPLE … a `compile` example, NOT the `md encode` accepted-heads prose" | `42-md.md:166` (exact) = `# tr(@0)`. | ✓ |
| **M-4** `tr_key_only` `:1195` | §P1.b (plan `:104`): "`tr_key_only` (`template.rs:1195` — Minor-4 grep note)" | `grep 'fn tr_key_only'` → `1195:    fn tr_key_only() {`. | ✓ |

---

## M5 §P3.4 H13-PRESERVATION — UNCHANGED / UNWEAKENED (the gate)

Re-confirmed every load-bearing element of the funds core against live source. The fold touched ONLY the insertion-fence precision (Minor-1) — the preservation argument is byte-for-byte the same as round 1, which was R0-SOUND.

1. **Group-3-validator-FIRST ordering — INTACT + now provable against live loop geometry.** Live `lex_placeholders` (`template.rs:40-128`): the group-3 validator (`if let Some(m) = caps.get(3)` `:77`) runs `m.as_str().split(';').map(|n| { if n.ends_with('\'') || n.ends_with('h') { return Err(…hardened…) } n.parse::<u32>().map_err(…not a bare unsigned integer…) }).collect::<Result<Vec<_>, _>>()?`. The `?` on the collect propagates the hardened/malformed error and **aborts the iteration before any code after the `:110` block-close executes.** The M5 residue check, fenced to land after `:110` (per §P3.4(a)) and before `out.push(…)` (`:117`), is therefore UNREACHABLE for a marker-bearing body. For the fused `@0/<0'';1>/0'/*`: `<0'';1>` splits to `["0''","1"]`; `"0''"` ends with `'` → typed "hardened" Err → `?` fires → M5 never runs. H13 stays REJECTED. ✓

2. **Byte-identical group-3 capture / validator / strip class — CONFIRMED.** §P3.4(b) mandates "group-3 capture (`([^>]*)` @ `:55`), validator loop (`:90-110`), and substitution strip class (`[0-9;]` @ `:498`) left BYTE-IDENTICAL."
   - `:55` lexer regex (exact): `@(\d+)((?:/\d+'?)*)(?:/<([^>]*)>)?(/\*(?:'|h)?)?` — group-3 = `([^>]*)`. ✓
   - validator `:77-110`: the `.split(';').map(…hardened reject + u32 reject…).collect()?` block — UNTOUCHED by any M5 edit (M5 edits are the post-`:110` residue check + the `parse_template` D4 belt). ✓
   - `:498` substitution regex (exact): `@(\d+)((?:/\d+'?)*)(?:/<[0-9;]+>)?(?:/\*(?:'|h)?)?` — strip class `[0-9;]`. The live in-source comment at `:489-497` independently states the strip "is moot now that the lexer rejects hardened / malformed bodies FIRST … this strip never sees a marker-bearing body. Keep this class in sync with the group-3 lexer ACCEPT set (`[0-9;]`)." This CORROBORATES the disjoint-region design at the source level — the substitution path cannot silently collapse a hardened multipath because the lexer rejects it upstream. ✓

3. **Fused `<0'';1>` H13-stays-rejected test — MANDATED + non-vacuous.** §P3.5 (plan `:223`): `wsh(multi(2,@0/<0'';1>/0'/*,@1/<0'';1>/0'/*))` "still errors with the **hardened/malformed** message (H13 before M5), **NOT** the suffix message." The discriminating assertion (message contains "hardened", not the multipath-not-final suffix text) genuinely proves ordering — it would FAIL if M5 were placed mid-loop or before the validator. ✓

4. **The 5 H13 guard tests — MANDATED GREEN, all line-exact.** §P3.5 (plan `:222`) re-asserts all five; live: `lex_rejects_hardened_multipath_apostrophe` `:197`, `…_h_form` `:211`, `lex_rejects_mixed_hardened_multipath` `:221`, `lex_rejects_malformed_double_marker_multipath` `:228`, `lex_accepts_nonhardened_multipath` `:247`. All five exact. ✓

5. **REJECT decision — INTACT (D1/D2).** §P3.2 (plan `:189-194`): fail-closed typed `CliError::TemplateParse`; framed as an md1/`UseSitePath` representability limit (NOT a BIP-389 prohibition); canonicalize rejected (would force an md-codec wire/`UseSitePath` field → break the no-toolkit-pin invariant); deferred FOLLOWUP `md1-post-multipath-fixed-path-derivation-steps` filed. The "do NOT over-claim … NOT a BIP-389 prohibition" protocol-fact correction (plan `:194`, `:215`) is preserved verbatim. ✓

6. **M5 is genuinely LAST + disjoint.** §0 phase order 1→2→3→4 (M5 in P3); the disjointness table (plan `:60-64`) confirms P1 and P2 never touch the M5 regexes/validator; M2 lands FIRST so M5 builds on a bounded `n`. ✓

**The fold did NOT weaken the H13 preservation.** The only delta vs round 1 is the §P3.4(a) fence text becoming byte-precise about the `:90-107` loop / `:108` collect / `:110` block-close geometry (Minor-1) — this STRENGTHENS the implementer's insertion constraint and matches the live loop structure exactly. The disjoint-regions claim, the validator-FIRST ordering, the byte-identical capture/validator/strip-class mandate, the fused test, the 5 guards, and the REJECT decision are all carried forward unchanged. Re-grep of live `lex_placeholders` + `substitute_synthetic` confirms the plan's M5 change provably **cannot** let a hardened multipath through (validator `?` fires upstream; strip-class never sees a marker per the live in-source invariant comment).

---

## No-new-drift sweep (round-1 GREEN items re-confirmed UNCHANGED)

- **Funds core (M5 mechanism / stitch divergence):** stitch `:1915-1921` exact — `use_site_path: resolved.use_site_path` (lexer) + `tree` (substituted), no cross-check → the divergence the D4 belt detects. UNCHANGED. ✓
- **`cargo fmt --all --check` CI-gate mandate:** `ci.yml:59` (exact) = `- run: cargo fmt --all --check`; clippy `:47` (exact) = `- run: cargo clippy --workspace --all-targets -- -D warnings`. Plan §0 (`:51-52`) mandates per-phase `fmt --all` → `fmt --all --check` + clippy `-D warnings`, correctly distinguished from the toolkit mlock.rs exemption. UNCHANGED + correct. ✓
- **Cross-repo manual = docs-only paired-PR discipline (NOT lint-gated):** §P4.4 framing UNCHANGED; `42-md.md:367-379` carries the false "rejected with a wire-format error" prose (verified, exact heading "### v0.6.0 limitation: chunked-form only" `:367`). Belt-check Minor-3 fold folded in (`:274`). ✓
- **SemVer:** md-cli MINOR `0.8.1`→`0.9.0` (`Cargo.toml:3` = `version = "0.8.1"`), md-codec NO-BUMP (`Cargo.toml:28` = `md-codec = { path = "../md-codec", version = "=0.38.0" }`), no toolkit pin, GUI schema_mirror NOT triggered, no sibling-codec companions. §P4.1/P4.3/P4.6 + D11/D12 UNCHANGED + correct. ✓
- **All citations:** every line number re-grepped this round (lexer `:55`, sub `:498`, validator `:77-110`, stitch `:1915-1921`, `multipath_alts` `:27`, `key_map` `:1883`, `repair.rs:118`, `encode.rs:45`, `FOLLOWUPS.md:123`, `tr_key_only:1195`, 5 H13 guards, `ci.yml:47/:59`, `Cargo.toml:3/:28`, manual `:166`/`:367-379`, 7 bughunt ids + 7 tick-boxes) is byte-exact. No mis-cited or moved edit site. ✓
- **M2 / M10 / M11 / L4 / L19 / L7:** mechanisms unchanged from round 1; L4's `repair.rs` rename now correctly noted (I-2). All RED-first + non-vacuous. ✓

No new drift introduced by either fold.

---

## Critical

**None.** The M5 §P3.4 H13-preservation core is carried forward unweakened (verified provably against live `lex_placeholders`/`substitute_synthetic`); no wrong-address / data-loss / silent-fail-open at the plan level.

## Important

**None.** Both round-1 Importants are RESOLVED:
- **I-1** — §P4.2 reframed: no pre-existing-FOLLOWUP flip (verified zero cycle-9 entries @ `836faf8`), canonical record = bughunt-box ticks (§P4.5, all 7 lines exact), slug names reconciled to the bughunt-report ids (all 7 present), new RESOLVED entries optional, L7 `:123` slug left RESOLVED, invented slugs removed (grep-confirmed).
- **I-2** — §P1.c adds the `repair.rs:118` `_descriptor`→`descriptor` rename-and-use note (verified `_descriptor` binds at `:118`, `encode.rs:45` already usable), with the correct clippy `-D warnings` rationale.

## Minor

All four round-1 Minors were folded and verified (M-1 fence precision, M-2 `key_map` join @ `:1883`, M-3 `42-md.md:166` compile-example flag, M-4 `tr_key_only:1195`). No new Minors.

---

## Verdict

**PLAN R0 ROUND 2: 0C / 0I — GREEN**

- **Critical:** none.
- **Important:** none — I-1 and I-2 both RESOLVED; folds are byte-accurate against `836faf8` and introduce no new drift.
- **M5 §P3.4 H13-preservation:** UNWEAKENED — disjoint regions, group-3-validator-FIRST ordering, byte-identical group-3 capture (`:55`) / validator (`:77-110`) / strip class (`:498`), the fused `<0'';1>` H13-stays-rejected test, the 5 H13 guards (`:197/:211/:221/:228/:247`), and the REJECT decision (D1/D2) are all INTACT. The fold only sharpened the insertion-fence precision (Minor-1), which matches the live loop geometry exactly. Re-grep of `lex_placeholders` + `substitute_synthetic` confirms the M5 change provably cannot let a hardened multipath through.

**The plan is R0-GREEN. The HARD R0 gate is cleared — implementation (single implementer in a worktree off `origin/main`, TDD RED-first, M5 LAST, per-phase full-suite + clippy + fmt gates, per-phase architect review, mandatory whole-diff post-impl review) MAY proceed.**
