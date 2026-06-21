# cycle-9 spec — R0 review, round 2

**Spec (folded):** `design/BRAINSTORM_cycle9_mdcli_parser.md` (md-cli lexer/parser cluster: M5 funds + M2/M10/M11 + L4/L7/L19)
**descriptor-mnemonic source SHA:** `836faf8` (`836faf87c3d82b119a9f0f5c6589a7db1f8613a4`; md-codec 0.38.0 / md-cli 0.8.1)
**mnemonic-toolkit manual SHA (live):** `origin/master` `d6398b57` (round 1 cited `79e3387`; spec §0 table still cites recon-time `8d2fe50` — Minor, see below; the false-prose lines re-verified against live `d6398b57`)
**Reviewer:** opus software architect (mandatory R0 gate — NO code until 0C / 0I)
**Date:** 2026-06-21

Round 1 was **0C / 1I (RED)** on I-1 (L7 manual-mirror justified by a non-existent lint mechanism). The I-1 fold + Minors 1-5 were applied. This round re-verifies each fold against the BYTES of `origin/main:<path>` (descriptor-mnemonic `836faf8`) and `origin/master:<path>` (toolkit `d6398b57`), adversarially, and re-confirms the funds-critical M5 H13-preservation core was NOT weakened.

---

## Fold verification (round-1 findings)

### I-1 (L7 manual-mirror reframe) — RESOLVED ✓

Round-1 I-1: the spec justified the toolkit `42-md.md` edit by a FALSE lint mechanism ("build the v0.9.0 `md` binary → lint with `MD_BIN=<v0.9.0 md>` for it to pass") and asserted the lint *enforces* the epilog prose.

**Re-grepped the spec for every L7 surface** (§3.6:206, §7:224, §8:231/235/237, §9:272, §10:285, D10:305) and **for every `MD_BIN` / "v0.9.0 binary" / "enforce" token:**

- **NO surviving claim that the lint enforces the prose.** Every occurrence of `MD_BIN` / "v0.9.0-`md`-binary build" is now a NEGATION — e.g. §3.6:206 "with **no v0.9.0-`md`-binary build and no `MD_BIN` pin** needed for lint to pass"; §8.3:235 "the L7 edit needs **no v0.9.0-`md`-binary build and no `MD_BIN` pin**"; §9:272 "needs **no v0.9.0-`md` binary / `MD_BIN` pin**"; D10:305 "the edit needs no v0.9.0-`md` build and no `MD_BIN` pin." There is no leftover "for the lint to pass you must build/pin" sequencing anywhere.
- **The edit is reframed as CLAUDE.md paired-PR DISCIPLINE (docs-only, NOT lint-enforced)** at all five surfaces — §3.6:206 "paired-PR DISCIPLINE, NOT lint-gated"; §7:224 "by paired-PR DISCIPLINE, NOT by the lint"; §8:231 "by CLAUDE.md manual-mirror DISCIPLINE, NOT a lint-enforced gate"; §8.3:235 "discipline-only, not gate-enforced"; D10:305 "required by CLAUDE.md manual-mirror DISCIPLINE, NOT lint-enforced."
- **The manual edit STILL appears as a required deliverable:** FOLLOWUP table §10:285 (`repair-help-epilog-stale-rejects-nonchunked-claim` … "paired toolkit-manual edit"), §8:237 ("the manual edit stays a required deliverable of this cycle (paired-PR discipline)"), §9:272 (the L7 test bullet asserts the corrected prose is present, human/PR-verified).

**Live-source corroboration of the new framing (the fold's factual basis):**
- `origin/master:docs/manual/tests/lint.sh` — the only `--help`-consuming step is **4/6 flag-coverage** (`step "4/6 flag-coverage"` @63), which runs `eval "$MD_BIN $sub --help"` (@74) and extracts **flag NAMES only** via `grep -oE -- '--[a-z][a-z0-9-]+'` (@84). The remaining 5 steps are markdownlint / cspell / lychee / glossary / index. **NO step diffs the `repair` epilog prose.** The spec's reframe is factually correct: the lint passes regardless of the prose edit, and no `MD_BIN` pin / v0.9.0-binary build is needed for it to pass.
- `origin/master:docs/manual/src/40-cli-reference/42-md.md` — the false prose is live at the cited span: heading "### v0.6.0 limitation: chunked-form only" @367, "Non-chunked single-string md1 … is rejected with a wire-format error — use `md decode` …" @371-374, FOLLOWUP-pointer prose through @379. Span `367-379` is correct.

**Verdict: I-1 RESOLVED.** The justification is now accurate, the binary-pin dependency is dropped, and the edit remains a required (discipline-gated) deliverable. No new drift introduced by the fold.

### Minor-1 (stitch field numbers) — RESOLVED ✓
Spec §0:22 now cites `use_site_path@:1918` (from lexer) / `tree@:1919` (from substituted), and the §3.1.1 pseudocode (spec :82-83) shows `:1918` / `:1919`. **Live:** the `Ok(Descriptor { … })` block is `n` @1916, `path_decl` @1917, `use_site_path` @1918, `tree` @1919, `tlv` @1920 (block opens @1915). Matches.

### Minor-2 / D4 concretization — RESOLVED, now CHECKABLE ✓
Spec §3.1.3 edit-2 (:120) and D4 (:299) now mandate: for each `@i`, compare the lexer's `occ.multipath_alts.len()` (0 when no `<…>`, else alt-count) to the **multipath-step count of the substituted `DescriptorPublicKey`** — explicitly NOT against `tree` (which "carries no per-key multipath field"). **Live-source confirms this is constructible:**
- `Occurrence.multipath_alts: Vec<u32>` (template.rs:27) → `occ.multipath_alts.len()` is a real value.
- The substituted descriptor is `MsDescriptor<DescriptorPublicKey>` (template.rs:570-574, `from_str` output); `DescriptorPublicKey` carries derivation paths / multipath — a real type to count multipath steps on.
- `tree` is `walk_root`'s tag/key_index/leaf structure (template.rs:573-705) — correctly identified as having no per-key multipath field.

D4 is now a CHECKABLE assertion (compare two real counts), not a vacuous always-true one. The fold also preserves the correct priority ("the residue reject in edit-1 is the real fix; this is the checkable belt against drift").

### Minor-3 (`synthetic_xpub_for` second consumer) — RESOLVED ✓
Spec §3.3:160 now documents that `ctx_for_template`'s output feeds TWO sites: `parse_key`'s depth gate AND `synthetic_xpub_for(i, ctx)`, and that flipping bare `tr(@0)`→SingleSig also makes its synthetic xpub depth-3 — "harmless and address-neutral" (synthetic discarded after `key_map`; depth advisory). **Live:** `synthetic_xpub_for(i, ctx)` (template.rs:458, called @506) keys depth off `ctx` (3 SingleSig / 4 MultiSig @463-465). The `tr_key_only` / synthetic tests pin `ScriptCtx::MultiSig` **explicitly** (not via `ctx_for_template`, template.rs:523-563), so they don't break. Note documented correctly.

### Minor-4 (L4/L19 regression home) — RESOLVED ✓
Spec §9:265 names `crates/md-cli/tests/cli_output_class.rs` as the regression home and §9:268 requires `byte_parity_advisory_lines` stay GREEN. **Live:** that test file exists with `fn byte_parity_advisory_lines()` @23 and asserts the `Template`/`WatchOnly` advisory lines. Correct.

### Minor-5 (`output_advisory.rs` path + line drifts) — RESOLVED ✓
Spec §3.5:187 now states `output_advisory.rs` is at "crate root `src/`, NOT `src/cmd/`"; §3.6:202 cites the L7 FOLLOWUP as RESOLVED at `FOLLOWUPS.md:123`. Both match round-1's live findings.

---

## M5 H13-PRESERVATION CORE — UNCHANGED / UNWEAKENED (the funds gate) ✓

Re-read §3.1.4 in full and re-grepped the live H13 functions. The four mandated elements are present and byte-identical to what round 1 R0-CONFIRMED sound:

1. **group-3-validator-FIRST ordering** — §3.1.4 bullet 2: "H13's group-3 validator (`:90-110`) runs during the same capture iteration and returns the typed 'hardened'/'not a bare unsigned integer' error **before** any M5 residue logic would matter." Present, unchanged.
2. **byte-for-byte preservation of H13's group-3 body + `[0-9;]` strip class** — §3.1.4 bullet 1: M5 "does **not** relax, widen, or touch the group-3 capture or its validator loop (`:90-110`) or the substitution strip class `[0-9;]` (`:498`)"; "Net … byte-for-byte preserved." Present, unchanged.
3. **fused hardened+suffix regression test** — §9 M5 bullet (:247): `wsh(multi(2,@0/<0'';1>/0'/*,@1/<0'';1>/0'/*))` "still errors with the **hardened/malformed** message (H13 fires before M5), not the suffix message." Present, unchanged.
4. **H13-reject-stays-RED guard** — §3.1.4 + §9:247 re-assert all five: `lex_rejects_hardened_multipath_apostrophe`, `lex_rejects_hardened_multipath_h_form`, `lex_rejects_mixed_hardened_multipath`, `lex_rejects_malformed_double_marker_multipath`, plus accept `lex_accepts_nonhardened_multipath`. Present, unchanged.

**Live-source re-grep of the H13 functions @ `836faf8` confirms the reject is provably intact under M5:**
- Lexer regex (template.rs:55) = `@(\d+)((?:/\d+'?)*)(?:/<([^>]*)>)?(/\*(?:'|h)?)?` — group-3 still permissive `([^>]*)`.
- Group-3 validator (template.rs:77-110) — splits the body on `;`, rejects any alt ending `'` or `h` (hardened) and any non-`u32` (malformed/residue), accepts bare integers. This is the C1-revert validator M5 must not touch.
- Substitution strip class (template.rs:498) = `(?:/<[0-9;]+>)?` — the narrow `[0-9;]`, NOT the widened `[0-9;'h]`.
- All five guard tests present at the cited lines (template.rs:197/211/221/228/247).

M5's residue check operates on path text OUTSIDE `<…>` (a `/NUM…` after `>`, or an `h`-bearing origin step before `<`) — a DISJOINT span from group-3. A hardened `<0h;1h>` or malformed `<0'';1>` body still hits the unchanged validator at template.rs:77-110 FIRST and rejects there, before any residue logic. The fold did NOT weaken the regex change, the ordering, the fused test, or the RED guard. **M5 H13-preservation core is intact.**

---

## No-new-drift audit

- **Resolved-decisions table (§11)** — consistent; header still "no open questions"; D1-D13 unchanged in substance. The §0:288 "verify each slug's 'open' status" is the FOLLOWUP-status memory rule, not a spec open-question.
- **No new open question / TBD / TODO** introduced (grep clean except the §11 header and §0 FOLLOWUP-status note).
- **canonicalize-vs-REJECT decision (D1/D2, §3.1.2)** — unchanged: REJECT (fail-closed) with the **BIP-389-representability-limit** framing (BIP-389 *permits* post-multipath steps; the reject is an md1/`UseSitePath` representability limit, not a BIP-389 prohibition). `make_use_site_path` (template.rs:339-356) reads only `multipath_alts` + `wildcard_hardened` — corroborates "no field for post-multipath fixed steps." Intact.
- **SemVer (§7, D11)** — unchanged: md-cli **MINOR 0.9.0** (M10 driver), md-codec **NO BUMP** (0.38.0), **no toolkit pin** (toolkit deps md-codec only, never md-cli), tag `descriptor-mnemonic-md-cli-v0.9.0` + `cargo publish -p md-cli`. Intact.
- **GUI schema_mirror (§7, D12)** — unchanged: NOT triggered (no clap flag/subcommand/dropdown add/rename/remove). Intact.

---

## Minor (carried, NOT fold-introduced; does not gate)

### Minor-A — §0 table caption still cites the recon-time toolkit manual SHA `8d2fe50`
Spec §0 table (:16) labels the toolkit manual lockstep target as `8d2fe50` (master, recon-time). Round 1 noted live was `79e3387`; live is now `d6398b57`. The false prose at `42-md.md:367-379` is byte-unchanged across all three SHAs, so NO fold or citation is invalidated — but the caption SHA is stale. Cosmetic; same class as round-1's accepted line-drift Minors; refresh the SHA at plan-doc time (the plan-doc re-greps citations against current `origin/master` per the CLAUDE.md grep-at-write-time rule, so this self-corrects there). Does not rise to Important.

---

## Verdict

**R0 ROUND 2: 0C / 0I — GREEN (0C/0I)**

- **Critical:** none.
- **Important:** none. I-1 (round 1's sole Important) is RESOLVED — the L7 manual-mirror is now correctly framed as CLAUDE.md paired-PR DISCIPLINE (docs-only, NOT lint-enforced), every `MD_BIN`/v0.9.0-binary token is a negation, the lint mechanism is described accurately (flag-NAMES only; no prose diff — verified against live `lint.sh`), and the manual edit remains a required deliverable.
- **Minors:** all five round-1 Minors RESOLVED; one carried cosmetic SHA-caption drift (Minor-A, non-gating, self-corrects at plan re-grep).

**Fold integrity:** I-1 fold resolved the finding and introduced no new drift. The M5 H13-preservation core (§3.1.4) is byte-for-byte UNCHANGED and UNWEAKENED — re-confirmed sound against the live `lex_placeholders` / `substitute_synthetic` functions @ `836faf8`. D4 is now a CHECKABLE assertion. SemVer / decisions / canonicalize-vs-REJECT all unchanged.

**Spec is R0-GREEN.** Cleared to proceed to the plan-doc (which carries its own mandatory R0 loop).
