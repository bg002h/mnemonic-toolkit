r1: 0C/0I/0M/0N

Independent toolkit companion review for W14 stage2 FIXED correction, slice 2. Reviewed the complete diff from `a1813bfee2d6b6cde0003b67b75633e34be4c0cc` through exact head `23a110a254642a74d366582f5c40b8efef0ba044`, branch `docs/w14-stage2-fixed-manual`. The source chapter is the same as `76c2698c`; the later seven files preserve validation evidence. All eight changed files were included. Applicable workspace/repository guidance and the complete review brief were followed. No toolkit source, evidence, workflow, followup or artifact was edited; this uncommitted report is the only reviewer write in this repository. No commit, push, release or real wallet operation was performed.

There are no Critical, Important, Minor or Nit findings in this companion. The chapter accurately carries the accepted FIXED/ETERNAL correction, including conditional card-only advice and retirement precedence. The inherited FIXED-advice issue remains an integration obligation until the accepted manual is merged and its master commit is pinned by the consumer. This report does not perform or claim that integration.

**Complete source review and compact case table.** I compared every changed hunk in `docs/manual/src/40-cli-reference/45-mr.md` with the accepted behavior, including surrounding instructions. An independent whitespace/Markdown-angle-escape normalization check found all seven exact advice paragraphs in the chapter. This comparison preserves their words rather than paraphrasing away conditions.

| Case or surface | Reviewed result |
| --- | --- |
| Introductory current/epoch explanation | Current computes a receiving epoch; kit notes establish FIXED/ETERNAL. A birthday series can have several intended epochs. |
| Ordinary owner/co-signer recipe | Concrete source epoch/slot choice precedes step1. FIXED preserves source epoch and policy; ETERNAL can use current after kit confirmation. |
| Unknown holder | Heir external-wallet alternative remains first; ordinary choice remains conditional, with explicit ordinary retired-card precedence. |
| Known heir and compromise | Their distinct external/recovery destinations remain intact. Neither receives the ordinary choice line. Ordinary payment and necessary recovery remain possible. |
| Migration and current examples | Routine duties are qualified as kit-confirmed ETERNAL; an answer from current is not proof of kit choice or permission to advance a FIXED deposit. |
| Deadline refusal | Both address and non-spending member are described. Retained current query is the ETERNAL option; explicit deadline consent retries the same FIXED member and leaves other guards active. Open-lock/ended branches are distinguished. |
| Create/upgrade checklist | After Coldcard, ascending FIXED axis-bearing items precede one eligible ordinary item. Any FIXED metadata, including epoch-less, controls singular/plural eligible-shape prefix. Shared seats do not merge choices. |
| Epoch-less/later metadata | Own-axis eligibility and all-epoch-less absence are explicit. Existing FIXED notes remain; later/upgrade runs do not infer missing metadata from IDs, names or older cards. |
| FIXED grammar explanation | No routine rollover; intended epoch/source policy retained; ordinary payments and compromise recovery allowed. Card strings/identity/current remain unchanged by metadata. |
| Retired-card page | Ordinary owner route uses the live card; neither ordinary FIXED nor migration text permits return to the retired source. Both heir alternatives retain external no-change destination. |

**Real inputs and command review.** The chapter changes advice surrounding existing commands, not their flags or command construction. I checked relevant candidate help/argument contracts and actual public synthetic operations in the paired review. No command was assumed runnable merely because flag lint or a text guard passed.

| Advice/command surface | Required real inputs and execution evidence |
| --- | --- |
| Create checklist | Public signet keys/seeds and roles, real shapes input, current public tip, no-other-cards declaration and explicit short-phrase consents. Ten fresh combinations of FIXED, unmarked, mixed and epoch-less shapes admitted successfully with exact expected item lists. |
| Later-card/upgrade explanation | Complete parent/successor relationship and legitimate next-round public origins for the former; actual md1, explicit delta, keys/roles and receive-zero seating witness for the latter. A seating witness is not a funding claim. Accepted test evidence exercises these admitted paths. |
| Source choice leading to address | Real card/network/tip/shape/e/s/index, held public material and receive/change choice. Both chains were checked at epochs0/1/2 against expected policies, public key material, locks1260000/1312560/1365120 and previously recorded public addresses. |
| Recipe descriptor/policy reruns | Actual backticked `mr member` argv was extracted; only named public seed placeholders were filled, and stdout was captured in place of `> import.json`. Six fresh emitted commands passed and retained the source policy/lock. |
| Deadline current query | Complete emitted query retained, with intentional lead filled as30d. Both address and non-spending member queries passed; refused source requests were separately retried with deliberate same-member deadline consent. Distinct slot refusal remained a refusal. |
| Retired-page source and destination | Page member operation uses the complete card chain and held-seat inputs. Ordinary recovery uses the actual live shape mapping and required distribution/deadline consent; heir routes remain external/no-change. Accepted journey evidence covers these branches. The no-seed unknown variant explicitly omits the optional seed pair; it is not claimed to be a verbatim replay of the seed-bearing example. |
| Core transaction commands | Existing construction is unchanged by this manual correction. Existing isolated Core validation is retained in the paired evidence; this review did not broadcast a transaction or operate a real wallet. |

The fresh public probes used the verified corrected candidate, not installed mr: version `mr 0.1.0-dev (caa90bf8)`, SHA256 `83b325f2f8f30ca3f411322a0834b169986ba70f468adc9266391f548711d4a1`. Its runtime source matches the reviewed paired head. Probes used transient in-memory shapes data and no real secrets or devices. They establish the affected advice journeys, not an assertion that every command in this large chapter has been replayed.

**Validation evidence and its limits.** I read both audit logs, guard/flag-mutation logs, PDF/HTML logs and the receipt, verified all six recorded log hashes, and checked both ignored rendered artifacts against the receipt. Initial audit failure was exactly two MD038 inline-code edge spaces; their correction changes formatting without changing advice semantics. Final audit passes, all158 documentation-guard selftests pass, all15 flag-lint mutation cases pass, and PDF/HTML builds pass.

| Artifact | Verified result |
| --- | --- |
| `docs/manual/build/m-format-manual.pdf` | 544 pages, 1931359 bytes, SHA256 `0055f86495c3406c35f905ef885572a60faf98985df791c02fefffb0480be2bc`. |
| `docs/manual/build/m-format-manual.html` | 1927139 bytes, SHA256 `142aa6fd59397edd19a0a61d135da1b43c1a19df143ca7ed53772493281c54a3`. |
| `design/agent-reports/w14-stage2-fixed-manual-validation/receipt.json` | Source revision76c2698c and seven-paragraph claim agree with the source/evidence split and independent text comparison. Its historical pending-review/CI notes are superseded by this review and the later CI snapshot, not silently rewritten. |
| `/scratch/code/shibboleth/w14-stage2-fixed-evidence/toolkit-ci-23a110a2-latest.json` | 25SUCCESS/6SKIPPED. Manual, technical-manual, quickstart, platform/test/build and other required jobs succeeded; recorded skips are not failures. The snapshot is an array of check records, so its head association comes from the supplied exact-head dispatch and filename, not an invented embedded SHA field. |

I inspected the manual workflow as well as its results. It builds its pinned tools and runs audit, example verification, guard tests, flag mutations and render checks. Toolkit's lint intentionally leaves MR_BIN unset and skips the mr verbs; the consumer's provisional companion-manual lint covers those flags against the corrected candidate. Existing `verify-examples-skips-mr` remains a bounded followup: neither the manual workflow nor successful flag lint proves that all mr examples executed. The direct advice probes and source review above address this correction's affected behavior without closing that broader followup.

I viewed the supplied page353 and page401 images and independently rendered/viewed PDF pages354,360,449 and450. The new source-choice, owner/unknown clauses, deadline text and per-shape checklist are legible and within margins. The retired ordinary selector is readable on449 and its precedence sentence continues correctly on450; no new text is lost at the page break. PDF page numbers are one greater than printed page labels here. Text extraction independently confirmed the changed content.

Inherited inline-code spacing/wrapping defects are visible in nearby pre-existing command examples; the validation receipt already bounds that limitation. The changed prose and quoted advice do not introduce an unrunnable command or missing condition. This is not whole-manual typography acceptance. The complete diff's raw TeX log retains trailing whitespace, so a blanket clean whole-diff-whitespace claim would be inaccurate; the preserved log bytes are evidence, not a source-formatting regression to normalize during review.

**Deferred items and integration.** The main-spec grammar Minor stays with its existing specification-maintenance owner; this companion's corrected FIXED explanation is accurate. The RED/control accounting Minor was separately resolved and evidenced in the paired build, not by this manual's green jobs. The existing mr-example replay and inline-code PDF limitations are unchanged and remain with their existing maintenance scope. There is no new companion followup or ungraded blocking finding.

The exact toolkit product head and clean worktree were rechecked immediately before writing this report. This companion has 0C/0I and completed required review-head CI. Toolkit-first merge and the consumer's master-pin/final-integration validation remain the parent's authorized landing work; they are not performed by this read-only reviewer.
