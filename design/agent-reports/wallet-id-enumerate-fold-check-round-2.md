FOLD-CHECK — round 2 on `design/SPEC_restore_wallet_id_prefix_enumerate.md`

Mechanical verification pass, scope fixed by dispatch brief: did commit
`2142398b` correctly close NEW-1 and NEW-2 from
`design/agent-reports/wallet-id-enumerate-fold-check-round-1.md`, and did it
introduce anything new. Not a re-audit; C1's operator ruling, §5's
verify-bundle ruling (option a), and the 16 original R0 findings' ADDRESSED
disposition are taken as settled per the brief.

Artifact: `git diff 5b4e9a0b..2142398b -- design/SPEC_restore_wallet_id_prefix_enumerate.md`
(single commit, single file, one hunk: `git diff --stat` shows 22
insertions / 4 deletions, `git log --oneline 5b4e9a0b..2142398b` shows only
`2142398b`). `bash scripts/spec-citation-gate.sh` re-run: PASS (23 citation +
6 arithmetic assertions `ok`, including the pre-existing `S=6 ... 0.01% at
b=2` row this fold's new text is consistent with).

SECOND FOLD: CLEAN

NEW-1: CLOSED — §3.3 now specifies the list-path summary line verbatim (the
`N assignments match prefix <hex> …` block), states which realized
cardinality (`n!`/`s_own`/`s_opt`, R0 N2) it names, and ends "This line is
what §3.7, §5 and §8 refer to as 'the summary'/'the hint'." Grepped every
"summary"/"hint" occurrence in the file (8 hits) and traced each: §3.7's
"§3.3's own summary line tells the operator to 're-run with
`--search-address`'" now resolves to real text; §5's "render the list +
summary" resolves to it; §8's "stdout or stderr for the summary hint? stderr
… so the hint is not the only carrier" resolves to it. All three referencing
sites now point at something that exists.

NEW-2: CLOSED — recomputed independently (script, S=6, 5 other assignments,
p_match(b) = 2^-8b, P(≥1 of 5 matches) ≈ 1-(1-p)^5):
  b=2 (16 bits): p=2^-16=1.525879e-05 → P=7.629e-05 ≈ **7.6e-5** (spec: "~7.6e-5 at the 2-byte floor") ✓
  b=4 (32 bits): p=2^-32=2.328306e-10 → P=1.164e-09 ≈ **1.2e-9** (spec: "~1.2e-9 at 4 bytes") ✓
  ratio = 65536 = 2^16 exactly, matching the 2-byte prefix-length gap between the two figures — same model at both lengths, unlike the deleted "~2⁻³²" text that mixed an unrelated 4-byte bound into a 2-byte-floor sentence.
Both figures in the new sentence are internally consistent and match the spec's own numbers.

Item 3 verdict — CONFIRMED, no inconsistency: §3.7 states verbatim "Required:
`--expect-wallet-id` and `--search-address` become mutually exclusive (clap
`conflicts_with`)" — mutual exclusion is genuinely required, not assumed. The
new §3.3 wording ("or re-run with --search-address instead") is consistent
with it and explicitly cites §3.7 as the reason. §8's resolved-question entry
("stdout or stderr for the summary hint? stderr …") never quotes the flag
wording at all, so it carries no stale "or" phrasing — the brief's suspicion
did not materialize there. One pre-existing, out-of-scope observation: §3.7's
own quote of the summary line ("tells the operator to 're-run with
`--search-address`'") still omits "instead" — but that quote is untouched by
this diff (git diff confirms the only hunk is in the §3.3 block), is framed
as the historical justification for *why* mutual exclusion was required
rather than a live restatement of current wording, and is not contradicted by
the fix — the fragment it quotes is still literally present in the current
text. Not a new defect from this fold.

No new issue found (item 4): the diff is confined to the §3.3 block; the
three referencing sections (§3.7, §5, §8) all resolve cleanly against it, and
the citation/arithmetic gate is unaffected (PASS, unchanged assertion count).

Full report: `design/agent-reports/wallet-id-enumerate-fold-check-round-2.md` (this file).
