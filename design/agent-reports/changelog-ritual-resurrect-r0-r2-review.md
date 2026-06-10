# R0 review — PLAN_changelog_ritual_resurrect — round 2

**Verdict: GREEN** (0 Critical / 0 Important / 2 Minor, both note-only — all 8 round-1 folds verified correctly applied, no fold-drift, no new defects)

## Round-1 fold verification (I1, I2, M1-M6)

**I1 — RESOLVED.** Plan line 24 (`[0.49.0]` row) now reads "byte-stable for `description_template`+`keys_info` — `name` lossy, see open FOLLOWUP `bip388-policy-roundtrip-wallet-name-not-honored`". Re-verified against `design/FOLLOWUPS.md`: the resolved v0.49.0 entry's exact sentence is "Round-trip is byte-stable for `description_template`+`keys_info` (lossy on `name`, pre-existing)", and the referenced fast-follow slug exists verbatim (`### bip388-policy-roundtrip-wallet-name-not-honored`, Status: open). The slug name in the plan matches the registry character-for-character. "byte-perfect" appears nowhere in the plan except the Fold log's description of what was removed.

**I2 — RESOLVED, no residual contradictions.** §4 (plan lines 96-98) is rewritten as CUT with all four staleness sites enumerated correctly (matching my round-1 probe data exactly: `:14`/`:44`/`:10`/`:32`, two wrong counts, 3 missing subcommands, ground truth 24) and files FOLLOWUP `readme-subcommand-inventory-stale-4-sites` in the same commit. Cross-checked every other section: header Shape line (line 7) says "new readme-staleness FOLLOWUP; ride-along CUT per R0-r1 I2" — consistent; §6 "No other files touched" — consistent (no README in the diff set); a full-plan sweep for `ride-along|twenty|bc1p|byte-perfect|grep -qF` hits only §4's cut description, the Shape line, and the Fold log. Nothing still treats the README fix as in-scope.

**M1 — RESOLVED.** `[0.48.0]` row now reads "release commit `223b538` + tag annotation (tag → `dcbd14c`; ...)". Re-verified: `git rev-list -1 mnemonic-toolkit-v0.48.0` = `dcbd14c`; `223b538` = "feat(template)!: emit NUMS internal key for bundled tr-multisig (v0.48.0)". Labels now match reality. BIP-341 attribution retained per round-1 guidance (FOLLOWUPS `toolkit-trmultia-nums-internal-key` entry: "BIP-341 NUMS H-point `50929b74…`").

**M2 — RESOLVED.** Workflow comment (plan line 47) now lists all five non-toolkit namespaces including `ultraquickstart-v*`. Re-verified namespace census: `manual`(5), `manual-gui`(3), `mnemonic-toolkit`(130), `quickstart`(2), `tech-manual`(6), `ultraquickstart`(1) — comment is now complete.

**M3 — RESOLVED, empirically clean.** YAML line 68 is now `grep -q "^## mnemonic-toolkit \[$VERSION\]" CHANGELOG.md`. Probed against the real CHANGELOG (see probes): matches `0.47.4` and `0.46.0`, rejects `0.52.0`, retains the closing-bracket prefix safety (`0.4` no match), and `0.48.0` correctly no-matches pre-backfill (rehearsal discriminator sound). The §6 count-grep (line 108) uses the same anchored-escaped form — internally consistent. actionlint on the plan's YAML block: exit 0, no findings.

**M4 — RESOLVED.** `[0.49.1]` row says "golden receive-address tests"; "bc1p" is gone (Fold log mention only). Matches FOLLOWUPS `:67` region ("Golden-address tests").

**M5 — RESOLVED.** §3 (plan line 80) now carries the preamble touch-up instruction, and the quoted framing matches the actual file: `design/RELEASE_CHECKLIST.md:9-11` reads "the **cross-repo pins** that no single repo's CI can verify without network calls into sibling repos" — the plan's quote and proposed amendment are accurate against current text.

**M6 — RESOLVED.** §6 line 107 loops all five backfilled versions must-PASS plus v0.52.0 must-FAIL; line 108 pins the count at exactly 124 (119 + 5). Re-verified: `grep -c '^## mnemonic-toolkit \[' CHANGELOG.md` = 119 at `2228ad3` today, and local `master` == `origin/master` == `2228ad31160e` — the plan's grounding line still holds.

## Critical

None.

## Important

None.

## Minor

**M-r2-1 (note-only) — two different "4 README sites" in §0 vs §4.** §0 line 11 says "4 README sites bill `CHANGELOG.md` as 'the full release history'" (link sites `README.md:16`/`:61`, `crates/…/README.md:10`/`:207` — verified true round 1); §4 says "the README staleness is 4 sites" (a different four: `:14`/`:44`/`:10`/`:32`, with `crates/…/README.md:10` in both lists). Both statements are individually correct, but the coincidental count invites a reader to conflate them. Optional one-word disambiguation ("4 README **link** sites" in §0); no factual error, no change required.

**M-r2-2 (note-only) — actionlint's shell-level lint was partial in this environment.** `shellcheck` is not installed here, so actionlint's exit 0 covers workflow/YAML semantics but not shellcheck rules on the `run:` block. Manual inspection of the script: `set -eu`, properly quoted `"$TAG"`-derived expansion, `if ! grep -q` (safe under `set -e`) — nothing shellcheck would flag. Implementation's pre-push `actionlint` step will pick up shellcheck wherever it is installed; nothing for the plan to change.

## Empirical probes run

1. **actionlint on the plan's §2 YAML** (lines 34-73 piped to `/usr/bin/actionlint -`): exit 0, no findings — the M3 grep change did not break the workflow lint.
2. **Anchored BRE vs real CHANGELOG:** `VERSION=0.47.4` → MATCH; `0.46.0` → MATCH; `0.52.0` → no match; `0.4` → no match (prefix safety preserved by `\]`); `0.48.0` → no match pre-backfill. `grep -c '^## mnemonic-toolkit \['` = 119.
3. **Header format ground truth:** `CHANGELOG.md:9` = `## mnemonic-toolkit [0.47.4] — 2026-06-06` — the anchored pattern's shape matches the real format; 5 new sections insert between the 8-line preamble and line 9 as planned.
4. **v0.48.0 relabel:** `git rev-list -1 mnemonic-toolkit-v0.48.0` = `dcbd14c`; `git log -1 223b538` = the NUMS feature commit — plan's relabeled sourcing is correct.
5. **Tag namespaces:** six namespaces re-counted; the workflow comment's five-item not-gated list is now complete.
6. **FOLLOWUPS citations:** `:3718` region (byte-stable sentence + open name-lossy fast-follow slug), `:67`/`:77` region (v0.49.1 golden-address tests; v0.48.0 BIP-341 NUMS, MINOR wire-content change), `:3746` region (Release A/B ship lines), `:3776` region (the resurrect slug, Status: open) — all present and matching the plan's rows.
7. **RELEASE_CHECKLIST.md:1-12:** preamble framing quoted in §3's M5 instruction matches the file verbatim.
8. **Residual-wording sweep:** `grep -in 'ride-along|twenty|bc1p|byte-perfect|grep -qF'` over the plan — hits only in §4 (the cut rationale), the Shape line, and the Fold log; no live section still depends on the cut ride-along or the corrected round-1 wording.
9. **SHA stability:** local `master` == `origin/master` == `2228ad31160e` — all round-1 fact-audit results remain valid without re-derivation.
