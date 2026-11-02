r1: 0C/0I/0M/0N

Independent companion review of the complete toolkit diff from `a1813bfee2d6b6cde0003b67b75633e34be4c0cc` through exact head `23a110a254642a74d366582f5c40b8efef0ba044`, branch `docs/w14-stage2-fixed-manual`. The worktree was clean on entry. Read applicable workspace/ancestor/root guidance, the complete slice-1 brief and the accepted public-behavior contract. This repository changes one manual source file, `docs/manual/src/40-cli-reference/45-mr.md`, plus seven retained validation files. All source hunks and the validation receipt/logs were considered. There are no new Critical, Important, Minor or Nit findings.

The manual accurately distinguishes FIXED deposits from kit-confirmed ETERNAL rollover. FIXED means the intended deposit epoch, including a nonzero birthday epoch; ordinary change retains the source policy, while ordinary payments and recovery remain possible. Neither `current` nor identifying an owner supplies the missing kit choice. The correction preserves the separate heir/compromised routes and the ordinary retired-card recovery precedence. No new command, flag or promised guard bypass is introduced.

| Complete source-diff area | Review result |
| --- | --- |
| Lines 25–34, epoch/current introduction | Current is a receiving calculation, not authority to roll FIXED funds; birthday series explained |
| Lines 2243–2312, recipe choice and destination clauses | Source e/s explained before step (1); owner limitation complete; unknown holder's heir alternative first; retirement precedence expressly ordinary-only |
| Lines 2334–2353, migration paragraph and explanation | Full ETERNAL qualification; FIXED unspent outputs and source change preserved; refusal does not authorize another epoch/shape/slot |
| Lines 2529–2544, both deadline consumers | Full conditional remedy, actual query substitution explained, explicit same-member deadline consent, other guards retained; open-lock case offers no query |
| Lines 2614–2617 and 2635–2637, surrounding migration/current discussion | Existing advice scoped to confirmed ETERNAL; current output itself is not presented as FIXED metadata |
| Lines 2731–2733, worked-example introduction | Example correctly conditional on ETERNAL; no new argv spelling |
| Lines 3086–3089, closing-checklist overview | References the per-refuge advice block instead of unconditional migration |
| Lines 3798–3808, FIXED metadata explanation | Replaces the misleading “nobody moves” definition; correctly describes notes plus checklist while preserving card identity |
| Lines 4310–4331, complete checklist rule | Per-shape axis, ascending FIXED items, then one eligible ordinary item; singular/plural prefix, shared seats, epoch-less FIXED, missing later metadata all covered |
| Lines 4441–4445, epoch-less overview | Explicitly omits the new advice block without deleting existing notes |
| Lines 6778–6784, retired recovery page | Ordinary live-card selector and precedence corrected; known and unknown heir-page alternatives retained |

All seven exact advice blocks were compared in full with the accepted normative/focused texts and actual public candidate output: ordinary rollover, FIXED item, concrete choice, owner destination, unknown destination, deadline remedy and retired ordinary rule. Only presentation whitespace/Markdown angle escaping was normalized. All seven also survive in the rendered HTML with inline markup and typographic apostrophes accounted for. This checks prose beyond the flag-table lint.

The compact case table covers the manual's rules and the observable behavior used to verify them:

| Case | Required outcome and evidence |
| --- | --- |
| All FIXED / all unmarked with axes | FIXED items only / one qualified ordinary item; fresh synthetic signet creates |
| Mixed metadata, interleaved shapes and shared seats | Ordered FIXED items followed by one ordinary item naming only eligible shapes; tested with up to five shapes |
| Epoch-less FIXED and unmarked shapes | No new advice for them; existing FIXED note retained; prefix still names eligible axis-bearing shapes |
| FIXED gift / birthday | Source epochs 0 and 1 preserve their respective policies; current 1/2 is a different destination, not an automatic replacement |
| Owner / public-plates-only holder | Exact qualified destination; eight fresh receive/change address operations follow both FIXED and ETERNAL options with real kit checks |
| Known heir / compromised source | Separate destination route remains; no ordinary choice line |
| Address or non-spending member at deadline | First refuses; actual printed current query runs; separately consented retry keeps the source member |
| Retired source with reordered live shapes | Emitted mapping selects live shape 0 for retired shape 1; kit-distribution consent remains required, and the old source stays refused |
| Later card / upgrade without FIXED metadata | Choice is not inferred; qualified advice remains; successor freshly exercised, upgrade corroborated by the passed compiled journey and its real inputs |

The create-input/command review covers all relevant changed printed instructions. Exact executed argv and outputs are retained externally; placeholder notation below is explanatory.

| Instruction/command | Required inputs checked against the candidate CLI | Result |
| --- | --- | --- |
| Create closing checklist | Shapes file with template/slots/appropriate delta/FIXED metadata, signet tip, public exported records, published test seeds, roles and required consents | 24 metadata/axis combinations admitted with exact items/order/prefix |
| Upgrade checklist | Existing md1, explicit delta/preset, matching public key/seed inputs, roles and receive-zero witness/index | Existing executed upgrade journey passes and does not infer FIXED |
| Choice-directed `mr address` | Actual source/current epoch as intentionally chosen, source slot and shape, card, signet tip, index, keys/seeds and issued seat checks; change flag for second chain | Both ordinary holder routes execute; same-source consent is explicit, not an automatic retry |
| Deadline's printed `mr current ... --lead L` | Actual emitted card/network/tip/slot/options, L deliberately set to `30d` | Both consumers' commands execute and answer epoch 2 in the birthday fixture |
| Printed descriptor/policy `mr member` recipe reruns | Complete emitted argv with public seed placeholders filled; plate boundaries, chain cards and flags preserved | Actual commands execute across ordinary/heir/unknown/compromised/retired cases; descriptor reruns retain source policy |
| Retired page's `mr member CARD --card OTHER_CARD ...` | Real complete chain, source K/E/S, held seed/plates and NET/H/T | All three holder contexts execute; unknown reader retains the heir alternative |
| Retired ordinary live-card `mr address` | Actual replacement mapping, complete chain, index/tip and distribution/deadline consents where required | Refuses before distribution consent; admits mapped live receive/change; retired source still refuses |
| Surrounding Core recipe | Destination prose is corrected; existing command templates, member inputs, fees and path locks preserved | No Core construction changed; supplied Core31.1 execution evidence retained, no new node started |

Public synthetic checks used the explicitly supplied candidate, stamped `mr 0.1.0-dev (caa90bf8)`, whose runtime inputs match MR review head `41ee012f8236e850b2d38a6687d87fb1e7836eca`. The fresh case/command outputs are in `/scratch/w14-stage2-fixed-review-r1-s1-20261031/`, particularly `commands.json`, `probe-result.json`, `retired-result.json`, `held-destinations-result.json` and their stdout/stderr files. The results distinguish correct policy/admission from text parity; they do not claim a real transaction, wallet operation or independent consensus implementation.

The 544-page PDF and HTML match the manual receipt's hashes: PDF `0055f86495c3406c35f905ef885572a60faf98985df791c02fefffb0480be2bc`; HTML `142aa6fd59397edd19a0a61d135da1b43c1a19df143ca7ed53772493281c54a3`. Inspected the retained destination-choice and checklist page images (PDF pages 353 and 401) and freshly rendered retired-page continuation (PDF pages 449–450). The changed paragraphs and precedence sentence are legible, within margins and complete across the page break. Existing inline-code spacing/line-breaking limitations elsewhere on those pages are pre-existing, accurately disclosed, and not a newly introduced finding. The entire changed source diff was reviewed even where only representative PDF pages were visually inspected.

The validation receipt and six associated log hashes match. Retained evidence records local manual audit, 158 guard self-tests, 15 flag-lint mutation tests and successful PDF/HTML builds. Toolkit head `23a110a2` retains 25 SUCCESS / 6 SKIPPED in `w14-stage2-fixed-evidence/toolkit-ci-23a110a2-latest.json`. The later supplied MR snapshot explicitly identifies head `41ee012f` and records 10 SUCCESS / 1 SKIPPED, macOS skipped with successful decision; its SHA-256 is `905a1affa4c76ec66874fe1aa9fd5a3b5e3fc6506e82cc473b284fa4cc89d822`. Earlier “remote CI pending” receipt text is superseded by those completed snapshots. No completed gate was rerun merely to recover counts.

Deferred-item re-grade: the pre-existing main-spec create-grammar Minor remains deferred to its existing owner. This manual correctly describes ordinary payments/recovery and the new checklist behavior, so that old wording does not leak into this companion. The previous RED/control accounting Minor is separately resolved by the accepted plan and retained baseline evidence. No new manual follow-up or C/I is identified. Relative-wait explanations remain applicable to newly created outputs, consistent with the output-age basis in [BIP 68](https://github.com/bitcoin/bips/blob/master/bip-0068.mediawiki#specification); this correction promises preservation of epoch/policy, not unchanged age of a new output.

This repository's full source diff is accepted at 0C/0I. The inherited migration-advice Critical's behavior is corrected by the paired build/manual, but its formal closure still requires accepted integration: toolkit first, then MR's final toolkit-master-history pin and resulting-head checks. No source, tests, validation records or existing reports were edited here. Only this uncommitted companion report was written in toolkit; the MR report is separately preserved at its authorized path. No commit, push, tag, build, device or real-wallet operation was performed.
