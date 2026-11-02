# FIX review r1: tk bug-hunt G2 (`design/FIX_bughunt_tk_g2.md` at e6504ded)

Reviewer: independent adversarial review, round 1 (no previous round, so no finding comes from a fix commit).
Scratch worktree: `/scratch/code/shibboleth/tk-worktrees/review-fix-r1-g2` (detached at 7115c685; the reference patch then applied there as the scratch commit 134dde27 for the build and the RED rows), removed at the end.

## Findings (appended as found)


No Critical and no Important. Every behaviour in section 2 was built from section 2 alone into a case table (below). Each cell is decided, except the two noted in M2 and M3. Both of those fail closed or are harmless.

### Minors

- **M1** §3 :529 and items 1-13. "the reference patch is the verified text" and the `(fix-NN)` pointers send the builder to patches it does not have. Section 2 together with the RED rows of section 6 decides every behaviour, so nothing is lost. Fix: :529 → "All in `crates/mnemonic-toolkit/src`. Section 2 is the behaviour and its texts; the `fix-NN` names are the triage's patches, for provenance only."
- **M2** 2.5 :185 and S6 :560. "wherever the key appears" / "wherever it appears" can be read as "anywhere in the file". The reference checks the top level only: a non-integer `account` nested in a bipNN block, which is never read, exits 0 under the reference and would exit 2 under the broader reading. Neither reading gives a wrong key. Conditions: (1) an `account` inside a bipNN block, which Coldcard never writes; (2) a non-integer value. Fix: :185 → "at the top level of the file (the only place Coldcard writes it), including when a bipNN block makes it unused; an `account` inside a bipNN block is not read and not checked". :560 → "at the top level, even when a bipNN block makes it unused".
- **M3** 2.11 :354. "joined per HRP" does not say whether the HRP compare ignores case. The reference compares the first three characters lowercased, so an upper-case chunk 0 followed by a lower-case chunk 1 of the same card gives exit 0. A case-sensitive builder splits them into two groups, and the card is refused (exit 2, fails closed). Conditions: (1) the chunks are given one per `--from` value or stdin line; (2) chunks of one card are typed in different case. Fix: "joined per HRP," → "joined per HRP (its first three characters compared case-insensitively, so `MK1…` and `mk1…` are one HRP),".
- **M4** manual A6 (:608). "corrected like any `ms1`" overstates the share path. A share with one dropped character under `--max-indel 1` exits 2 with `repair: chunk could not be recovered within --max-indel 1 (HRP 'ms'); …`. The same drop in a seed card exits 5 and is recovered (measured on the reference build; the share answer is the same on the unpatched build). Fails closed. Conditions: (1) the card is a K-of-N share; (2) the error is a dropped or inserted character; (3) `--max-indel` is given. Fix: "is\n  corrected like any `ms1`:" → "is\n  corrected like any `ms1` for substituted characters (`--max-indel` does not recover a share with a dropped or inserted character; it exits `2`):".
- **M5** manual B3 (:659-662) and 2.3 :134. "When every cosigner is at `m/45'` … the descriptor is `sh(sortedmulti(…))`" holds only for `Format: P2SH`. A P2WSH or P2SH-P2WSH file at `m/45'` imports as `wsh(sortedmulti…)` or `sh(wsh(sortedmulti…))`. That is correct for the file's `Format`; only the manual's claim is wrong. Condition: a `Derivation: m/45'` file whose `Format` is not P2SH (Coldcard writes `m/45'` only for P2SH, so the file must be hand-edited or come from another exporter). Fix: B3 "and the descriptor is" → "and, for `Format: P2SH`, the descriptor is". At the end of that sentence add " (the file's `Format` picks the script type, as for any other path)". The same wording goes at :134.

### Nits

- **N1** The reference patch's `repeated_digest_cap_agrees_with_enumerate` keeps stale assertion messages ("≤ 8", "at 7", "raw == 8"). §7 item 1 describes the corrected test, so no change to the document is needed.
- **N2** `<v>` is defined two ways: "serde_json's rendering" at 2.5 :188 and "the JSON text as written" in RED row 29 at :723. The two agree for `"5"`, `-1`, `1.5` and `null`, but `1e3` renders as `1000.0`. Fix: at :723 replace "the JSON text as written" with "serde_json's rendering (as 2.5)".
- **N3** The 2.11 consequence sentence at :357 names only "several multi-chunk cards … given chunk by chunk". A whole single-chunk md1 next to another md1's chunks is also joined and refused: `word-card --from md1fn3lnqqpqqqxqqzltp2wcfn0fqa --from $MD0 --from $MD1 --from $MD2` exits 2 with `md1 chunks disagree on version/chunk-set-id/count`. That input was also refused before the fix. The printed remedy works as written: `--from md1fn3l… --from "$MD0 $MD1 $MD2"` exits 0. Fix: "several multi-chunk cards of one HRP given chunk by chunk" → "chunks of one HRP that come from more than one card (several cards given chunk by chunk, or a whole card next to another card's chunks)".
- **N4** §7 :772 greps `cli_hrp_case_insensitive.rs:6`, which still says "md-codec is lenient". A8 fixes that same stale claim in the manual (and `inspect_mixed_case_md1_rejected` in the same file shows it is stale). Optionally update the test comment in the same build (pre-existing text, not G2's).

## Coverage (what was run)

- (a) Every section 2 behaviour was compared with the reference patch, hunk by hunk. Each prescribed edit gives the reference behaviour except M2 and M3, where the text admits a second, fail-closed reading. The inspect guard of #51 sits in the `CardKind::Ms1` arm, so mk1, md1 and a single ms1 are unaffected (run).
- (b) `doc-commands.sh` (every command section 2 prints) was run against the reference build. The output was identical to the author's transcript, with each printed stdout, stderr and exit matching.
- (c) The doc-edit script was applied after G1's manual patch: every one-occurrence assert passed. The diff was read: spec edits S1-S9; manual edits A1-A10, B1-B5, C1, D1-D2; the Sparrow transcript diff `15c15 "bip84-0" → "Keystore 1"`; the examples line `2 hashes` → `1 hashes`. A5 "echoed as typed" was re-measured on md1 and mk1: an upper-case chunk that needed no correction stays upper-case, and the corrected chunk prints in lower case.
- (d) Advice lens. The commands below were run as printed and each gives the right result:
  - the #56 remedy forms (all give the same `sp1` address);
  - the X3 recombine command (`ms-shares combine` after a share correction) recovers the split entropy;
  - the #51 per-card advice;
  - the #50 joined-card remedy;
  - the #21 ruled refusal's advice (`--format descriptor` and `--format bitcoin-core` each exit 0 on the NUMS taproot multisig).
- (e) A case table for each fix:
  - #57: pairs that should merge still merge (Core blobs built with BIP-380 checksums). A same-key pair with different script bodies is refused; for example, a pair with timelocks 144 and 145, which was merged before.
  - #19: `xpub` → mainnet and `tpub` → testnet; the round trip keeps `45'`; a mixed-path file is refused.
  - #29/#31/#49: each value class gives the text of 2.5.
  - #26: the warning appears for an address target with a passphrase.
  - X1: the right chunk index for every kind, with and without `--max-indel`.
  - X3: single and multiple substitutions in a share; a clean share exits 0.
  - X4: ms1, mk1 and md1 in capitals.
  - The undecided cells are M2 and M3 only.
- (f) The RED rows of section 6 were run on the reference build with `cargo nextest run --locked` (CARGO_TARGET_DIR inside the scratch worktree, cargo 1.85.0): 49 run, 49 passed. Every row has an exact assertion and a stated failure on the unpatched code.
- External facts were checked against the raw text:
  - BIP-129 :81 ("sign the first four lines") supports #18 as written.
  - BIP-388 :147-151 and :310 ("Expression with a non-KP key present") support the #21 ruling.
- Pre-existing, not G2's, not graded: `word-card --from "$MK0 <MK1 with one substituted character>"` silently BCH-corrects and renders the true card (exit 0, no note) on both builds, whereas a one-character md1 typo is refused (`BCH checksum verification failed`). After #50 the chunk-by-chunk form behaves the same way as the joined form.

## The author's choices beyond the grade (§11 lists five; the brief says three, plus :828)

1. X1 under `--max-indel`: sound. It is the grade's "every kind" rule applied to the one path that still answered with the misleading "could not be recovered". The RED x1 row pins it.
2. #18 refusing a record whose line 3 was edited after signing (row 18b): sound. BIP-129 :81 signs "the first four lines" as written.
3. #20 changing the manual's Sparrow round-trip promise: forced by the grade's `Keystore <n>` ruling, not a STOP. The new B2 text matches the measured transcript diff.
4. #56 checking depth before network: sound. Both orders fail closed, and the depth refusal names the real problem.
5. X3's defensive arm: unreachable from `IsShareNotSingleString` and harmless.
6. :828 "Open for the architect": comes from the grade (`bughunt-grade-tk-g2.md` :159), not from the author. It sits in a deferred-minor entry that the G2 build does not build, so it is correctly not a STOP here. Measured: `export-wallet --format bip388 --template tr-sortedmulti-a --taproot-internal-key nums …` exits 0, and its `description_template` starts `tr(50929b74c1a0…`, the literal NUMS key that BIP-388 :310 lists as invalid.

## Overlaps O1-O10 (G1 applied first)

All ten were checked against G1's §10 (`FIX_bughunt_tk_g1.md` :1780-1800):
- Each exact G2 edit still applies once after G1: the doc script's one-occurrence asserts pass on G1's patched manual.
- The reference patch applies cleanly on top of G1.
- O2: G1 only appends to `toolkit-repair-ms1-stdin-second-line-ignored`; its re-scope is G2's 9.5 #2.
- No conflict.

Design concern (ungraded): the bip388 template path above emits a policy that a BIP-388 signer must reject, while the `--descriptor` path refuses. Schedule the architect item with the #21 text so both paths give the same answer.

r1: 0C/0I/5M/4N
