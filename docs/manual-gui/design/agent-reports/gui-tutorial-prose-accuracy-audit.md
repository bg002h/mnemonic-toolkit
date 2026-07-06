# GUI-tutorial PROSE accuracy audit

Scope: every prose CLAIM in `docs/manual-gui/tutorial/*.md` (Ch0 + J1–J5) cross-checked
against the byte-committed transcripts `docs/manual-gui/transcripts/tutorial/*.{stdout,stderr,exit}.txt`
(real `mnemonic 0.75.0` runs), the figures `docs/manual-gui/figures/tutorial/*.png`, the
`.examples-build/` source-of-truth (`gen.sh`, `policy.json`, `render_desc.py`), and the
toolkit CLI source. Analysis only — no files changed except this report.

## Headline

**2 new mismatches found beyond the two already-fixed ones: 1 CERTAIN, 1 POSSIBLE.**
The two known fixes are confirmed correct. Everything else (all fingerprints, xpubs,
descriptor checksums, first-receive addresses, exit codes, card counts, cosigner counts,
reveal/masking behavior, refusal messages) verifies byte-for-byte.

## Mismatch table

| # | Sev | Prose cite | Wrong claim | Correct value + source | Suggested fix |
|---|-----|-----------|-------------|------------------------|---------------|
| 1 | **CERTAIN** | `10-ch0-orientation.md:55-57` | "only **two** steps keep their modal shot (Journey 1's single-sig bundle **and Journey 2's all-seeds bundle**)" | Only **ONE** step has a modal figure: `tut-j1-01-bundle-single-sig-modal.png` is the sole `*-modal.png` in `figures/tutorial/` (every other step has exactly 2 shots: form + run). J2-07 has **no** modal figure, and J2-07's own prose `30-j2-multisig.md:231-233` says its modal "is **not re-photographed**." | "only **one** step keeps its modal shot (Journey 1's single-sig bundle)"; drop the "and Journey 2's all-seeds bundle" clause. |
| 2 | POSSIBLE | `40-j3-degrading-vault.md:76` | "all **three** `multi(...)` thresholds across the **four** `or_i` branches" | The byte-committed descriptor (`tut-j3-10-canonicalise.stdout.txt` + `.examples-build/work/policy.json`) has **FOUR** `multi()` nodes — `multi(3)`, `multi(2)`, `multi(2)`, `multi(1)` — one per branch/tier. The Examples source `.examples-build/gen.sh:453` phrases it "`multi(...)` threshold across all **four** `or_i` branches." "Three" only reads true if it means "three *distinct* threshold values {3,2,1}", which the 1:1 pairing with "the four `or_i` branches" doesn't support. | "the `multi(...)` threshold in each of the four `or_i` branches" (or "all **four** `multi(...)` thresholds"). |

## The two already-fixed claims — CONFIRMED CORRECT

- **`20-j1-single-sig.md:37-39` — card printed once, grouped.** stdout `tut-j1-01-bundle-single-sig.stdout.txt` shows each card type once (`ms1` 1 line, `mk1` 2 lines, `md1` 3 lines), grouped into five-char blocks (`ms10e ntrsq qqqqq …`). The run figure `tut-j1-01-bundle-single-sig-run.png` shows the same. No "printed twice." ✓
- **No residual `bip84` restore-default.** All four restore chapters (`30-j2:271`, `40-j3:170`, `50-j4:126`, `60-j5:51`) say the restore form's default template is `bip44`. Verified against `template.rs:16-18`: `Bip44` is the first `CliTemplate` ValueEnum variant and `restore`'s `--template` has no clap default, so the GUI dropdown defaults to `bip44`. ✓ The only `bip84` mentions are the **bundle** form default (Ch0:39,177-178; J1:17) and J1's single-sig template — both correct (bundle stderr prints `# Template: bip84`; Ch0 orientation figure's Preview line reads exactly `mnemonic bundle --network mainnet --template bip84 --language english`). ✓

## Clean chapters (no further mismatches)

- **`00-frontmatter.md`** — fingerprints `73c5da0a`/`b8688df1`/`28645006` and the three demo seeds all verified; masking narrative consistent.
- **`20-j1-single-sig.md`** — clean; both known fixes correct (see above).
- **`30-j2-multisig.md`** — clean. Verified: `fingerprint: 73c5da0a`/`b8688df1`/`28645006`; xpub prefixes/suffixes for all three cosigners; canonicalise checksum `#4wup4at0`; BSMS + restore first-address `bc1qkssenl2m6t3aynza394sr9m86vt6md2v76kj52jun2xlwrdeaa4q84qtpl`; watch-only card set (ms1 omitted, 3× mk1, 1× md1); all-seeds full secret set (3× ms1 + 3× mk1 + md1, one argv-warning per slot + `can spend`); restore default-format `bitcoin-core` → importdescriptors; the `(none)`/single-sig-template-refused-exit-2 behavior. Reveal toggles verified in figures for steps 02/03 (`--from phrase=` revealed) and 07 (last of three rows revealed, other two masked), Preview/modal/argv all masked.
- **`50-j4-taproot-twin.md`** — clean. Verified: depth-2 refusal `taptree branch must have 2 children, but found 1` exit 2; PR-#953 attribution matches `gen.sh:494`; Kint `[73c5da0a/84'/0'/4']`; canonicalise checksum `#snerswx7`; twelve mk1 cards / 12 cosigners; restore address `bc1p9stcwz5597fmkxae9343k8edzkcvdczf9qp65r6p447pg0et82yqst3d2c`; BSMS-unsupported message + exit 2; compare-cost `+68`–`+71` vB deltas, no-stderr, both closing notes; NUMS checksum `#8nz0lwja`, NUMS `H` point `50929b74…803ac0`.
- **`60-j5-watch-only.md`** — clean. Verified: restore-to-descriptor stdout `wsh(sortedmulti(2,…))#yjp7hj7w`; first address matches J2; restore-core importdescriptors with `"active": true`, `/0/*` + `/1/*`.

## Notably verified (spot-checks that could have gone wrong)

- **`opensessame` hashlock (J3:19-20).** The descriptor commits `sha256(a84dce40…9a08ad)`. A naive single `sha256("opensessame")` does NOT match — but `gen.sh:389-390` defines `H = sha256(sha256(word))`, and `sha256(sha256("opensessame"))` = `a84dce40975727c398023cfbd50d5db3b9662375521d0f1ac62dbd829b9a08ad` exactly. Prose is **correct**.
- **`Ch0:39,177-178` orientation defaults & Preview line** — verified pixel-exact against `tut-ch0-00-orientation-form.png`.
- **11 / 12 distinct-key counts** (J3:5, J4:8) — engrave stderr lists cosigners `@0..@10` (11) and `@0..@11` (12) respectively.
- **Timelock table values** (J3:11-14) — `after(1000000)`, `after(1893456000)`, `older(65535)`, `older(4255898)` all present in the descriptor and match `gen.sh:368-386`.

## Could-not-verify (no transcript/figure; noted, not flagged)

- Byte-parity cross-references to `Examples.pdf` sections 2/3/3.4/4/5/6 (that PDF is not in scope here). The section structure is consistent with `.examples-build/gen.sh`.
- The narrative *causal* attribution of the depth-2 refusal to "the upstream PR-#953 bug" (J4:22) — not in the transcript, but matches the Examples source `gen.sh:494`, so treated as consistent.

## Fix list (for the orchestrator to apply + review)

1. **CERTAIN — `docs/manual-gui/tutorial/10-ch0-orientation.md:55-57`:** replace
   "only two steps keep their modal shot (Journey 1's single-sig bundle and Journey 2's all-seeds bundle)"
   with "only one step keeps its modal shot (Journey 1's single-sig bundle)". (The
   surrounding "every other secret step still passes through the same modal, it just is not
   photographed twice" stays.) `Ch0:46` "two (sometimes three) moments" needs no change — it is
   already consistent with a single three-shot step.
2. **POSSIBLE — `docs/manual-gui/tutorial/40-j3-degrading-vault.md:76`:** replace
   "all three `multi(...)` thresholds across the four `or_i` branches" with
   "the `multi(...)` threshold in each of the four `or_i` branches" (matches the four
   byte-committed `multi()` nodes and the Examples source `gen.sh:453`).
