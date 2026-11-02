# STOP: FIX_bughunt_tk_g1 manual 5.1 text fails the manual job's markdownlint (MD038)

Worktree head 661c8117 (all FIX items built and committed; this is the only open point).

## The point

FIX section 5.1 (`design/FIX_bughunt_tk_g1.md` line 1451) gives the `--import-json` row text:
"... and give the words alone, without the `phrase: ` label, on stdin to `--slot @N.phrase=-` ...".
The code span `` `phrase: ` `` ends in a space. Applied verbatim at
`docs/manual/src/40-cli-reference/41-mnemonic.md:368`, the manual job's lint step 1/7 fails:

    src/40-cli-reference/41-mnemonic.md:368:777 MD038/no-space-in-code Spaces inside code span elements [Context: "`phrase: `"]
    [lint] FAIL: markdownlint reported issues     (make audit -> lint Error 1)

Nothing else fails: with the pinned mk-cli v0.13.0 installed from its tag, `make verify-examples`
(62 transcripts), `make anchor-check` and `ci/doc-flag-lint.test.sh` (15 passed, 0 failed) are clean
(log dir: /scratch/cache/bughunt/tk/build-g1/). The FIX's claim "lint unchanged" was measured before the
I-1 fold added this phrase.

Every other place that quotes the label is outside the manual (the CLI error text in FIX 2.22 puts
`"phrase: "` in double quotes, no markdown), so only this one manual line is affected.

## Options (the wording is a product point, so not chosen here)

1. Drop the trailing space inside the span: "without the `phrase:` label" (the label as printed, minus its space).
2. Move the label outside code: "without the label word `phrase` and its colon".
3. Keep the text and add `<!-- markdownlint-disable-next-line MD038 -->` (not applicable inside a table row;
   would need the whole row exempted, so least preferred).
4. Quote the label as in the CLI text: `without the "phrase: " label` (double quotes, no code span).

Option 1 or 4 keeps the sentence closest to the FIX and to the CLI text. Whichever is chosen also needs
the same wording in FIX 5.1 so the document and the manual agree.

## State left

Committed unchanged from the FIX text. Nothing further committed. Gate verdict is FAIL on this one item only.
