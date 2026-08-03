# SPEC — `verify-bundle` descriptor-intake exit tier: 4 → 2 for input errors

**Status:** ✅ **R0 GREEN** at round 3 (0C/0I). Rounds 1 (1C/2I) + 2 (3I) + 3 (4 Minor) all folded. Reviews persisted verbatim at `design/agent-reports/verify-bundle-exit-tier-r0-round-{1,2}.md`.
**Target:** toolkit **v0.97.0** (SemVer-MINOR; documented-CLI-contract behavior change).
**Source SHA:** `origin/master` @ v0.96.0 (`4c89891a`). All citations re-grepped at that SHA per the project's citation-decay rule.
**Origin:** fable design consult, 2026-08-03, itself triggered by the Wave-4 L1 post-implementation audit.

> **This SPEC changes behavior pinned by a prior R0-reviewed test** — cycleA plan-R0 finding **I-B** (`tests/cli_cycleA_phase2_funds_proof.rs:191-231`). **R0 round 1 corrected the framing:** I-B was an *assertion-accuracy* correction — it fixed the cycleA plan's Phase-3a test, which asserted the wrong variant for the **concrete** fork, and pinned the @N fork's incumbent exit 4 while marking that test **"(optional)"**. It was never a de-novo ruling that 4 is the right tier. So this is a smaller reversal than originally stated; it is still run as a gated micro-cycle. §7 states the case.

---

## 1. Problem

`mnemonic verify-bundle --descriptor <D>` (and `--descriptor-file`) takes a **user-supplied** descriptor. When `<D>` fails lex / resolve / parse, the verify path re-wraps the error as `ToolkitError::DescriptorReparseFailed` → **exit 4**. The emit path (`bundle --descriptor <D>`) maps the identical malformed input to `DescriptorParse` → **exit 2**.

Exit 4 is this project's **BundleMismatch / VERIFY-ME** tier. So a user who mistypes a descriptor is told, in effect, *"your engraved bundle may be corrupt — confirm it out-of-band"*, when the tool never looked at a single card.

Three independent facts make this a defect rather than a taste question:

1. **Exit 4's contract presupposes a verification RESULT.** The GUI's canonical gloss (`mnemonic-gui/src/app_window.rs:1180-1206`): exit 4 = "a result with NO self-oracle that the user MUST confirm out-of-band before trusting". A parse failure produced no result. The GUI renders an amber **VERIFY-ME** badge for a typo.
2. **The same command already exits 2 for every OTHER malformed-descriptor shape.** A malformed *concrete* descriptor → exit 2 (`DescriptorParse`); mixed-form / origin-less / keyless → exit 2 (`classify_descriptor_form`, `wallet_import/pipeline.rs:188-219`); slot-coverage mismatch → exit 2 (from the *shared* `bind_descriptor_mode_paths`, `bundle.rs:2255-2259`, even when called from verify). Only the `@N` lex/resolve/probe path re-wraps to 4.
   *Qualification (R0 I-1):* one deliberate verify=4-on-input precedent DOES exist — `Bip388VerifyDistinctness` (`error.rs:23-24`, `:599`), fired pre-card at `verify_bundle.rs:1429-1431` / `:1717-1719`, where the emit path maps the same condition to `Bip388Distinctness` → exit 2. That is a considered v0.19.0 SPEC §4.11.c per-surface split, and it is **distinguishable** — see §7.
3. **The manual's stated RATIONALE describes a mechanism that is not the one firing.** `docs/manual/src/40-cli-reference/41-mnemonic.md:199-201` says an `@N`-template descriptor "rejects at exit 4 (`DescriptorReparseFailed`) **when the completed wallet is re-parsed**". But the pinning test's own comment (`tests/cli_cycleA_phase2_funds_proof.rs:204-207`) states the reject "fires inside `lex_placeholders` **before any card is consulted**" — i.e. site `:1445`, not the completed-wallet re-parse at `:1722`. The documented justification for the tier does not match where the error originates. The manual also contradicts itself: `:231` describes the same fixed-use-site class as **exit 2**.

**Not a funds risk.** Every path refuses with a non-zero exit; no wrong result is produced. This is a mis-tiering / wrong-alarm defect.

## 2. Provenance is uniform — so the split must be by STAGE, not source

The obvious fix ("route user-supplied to 2, keep artifact-preserved at 4") is **not available, and not needed**:

- `descriptor_mode_verify_run` has exactly one call site (`verify_bundle.rs:432`), entered only when `args.descriptor.is_some() || args.descriptor_file.is_some()` (`:423`).
- `--bundle-json` intake (`load_bundle_json_into_args`, `:1965-2027`) extracts **only** `ms1`/`mk1`/`md1` and carries the rest via `..args.clone()`; it never populates `descriptor` from the artifact.
- `verify-bundle` has **no** `--from-import-json` flag (grep: zero hits).

**A preserved-in-artifact descriptor can never reach these sites.** All 7 are user-provenance. The first draft then reached for a *pipeline-stage* distinction instead — R0 C-1 showed that distinction is also empty, because the only supposedly post-binding site is unreachable (§3). **Every reachable site is an input/usage error.**

## 3. The change — ALL SEVEN sites are input-stage

**R0 round 1 (C-1) disproved this SPEC's original structure.** The first draft kept `:1722` at exit 4, calling it "the completed keyed wallet fails re-parse". That site is **structurally unreachable**:

`parse_descriptor(input, keys, fps)` (`parse_descriptor.rs:933-1012`) is a deterministic function of `input` alone. Every fallible step — `lex_placeholders` (`:963`), `resolve_placeholders` (`:964`), `substitute_synthetic` (`:975`, which substitutes *synthetic* xpubs, not the caller's keys), `MsDescriptor::from_str` (`:976`), `walk_root` (`:978`) — reads only `input`. The caller's `keys`/`fingerprints` participate solely in infallible map/sort/assign plumbing afterwards (`:980-1003`). In `verify_bundle.rs`, `descriptor_str` is a non-`mut` binding after `:1412` (also read at `:1422`, `:1424`, `:1445`), so the probe `:1467` and the final parse `:1721` see a byte-identical string. **If `:1467` succeeded, `:1721` cannot fail.** There is no key-dependent re-parse. Independently re-verified for this fold.

So the honest split is not 6-vs-1 by stage. **All seven sites are input/usage-stage. SIX flip to exit 2; FIVE of those are REACHABLE** (`:1514` flips but is dead). The reachable five are what §6's mandatory per-site cells cover:

| Site | What failed | Now | Becomes | Reachable? |
|---|---|---|---|---|
| `:1371` | `--descriptor-file` read (fs error) | 4 | **2** | yes |
| `:1445` | `lex_placeholders` | 4 | **2** | yes |
| `:1449` | `resolve_placeholders` | 4 | **2** | yes |
| `:1468` | canonicity probe | 4 | **2** | yes |
| `:1514` | missing `--slot @idx` | 4 | **2** | **NO — dead** (R-2) |
| `:1678` | unsupported `--slot` subkey set | 4 | **2** | yes |
| `:1722` | final parse with bound keys | 4 | **4, defensive** | **NO — unreachable** (C-1) |

`:1514` is dead: `validate_slot_set` enforces contiguity from `@0` (`slot_input.rs:263-272`) at `:1454`, and the shared coverage gate enforces `max_idx+1 == n` (`bundle.rs:2244-2259`) at `:1484` — both before the loop. Live probe: n=3 with one slot → **exit 2**, "descriptor has n=3 placeholders but --slot vec covers 1 slots". Re-verified for this fold.

**Disposition of `:1722` (R-3 ruling):** keep it as an explicitly **defensive** arm, reworded to honest internal-invariant language — precedent `repair.rs:900-918`'s `PostCorrectionDecodeFailed` ("this never triggers…; a failure here would indicate a divergence"). Rationale: if parse determinism ever breaks (e.g. a miniscript bump), failing loudly in a distinct tier is a useful tripwire, and the arm costs nothing while unreachable. The enum variant is **retained, not retired** (precedent: `ExportWalletFormatStub`, `error.rs:141-145` — a retained variant with no construction sites); retiring it would be a needless API break.

**Also in the rewording scope (R0 round 2 MINOR):** `error.rs:124-126`'s rustdoc still carries the disproven artifact-provenance framing ("corrupted JSON, manual edit, upstream library version mismatch"). It is rewritten to the defensive internal-invariant wording alongside the arm itself.

**NON-NEGOTIABLE (R-3):** because the arm is unreachable, **no user-facing document may present "completed wallet failed re-parse" as a live exit-4 meaning** — not the manual, not the CHANGELOG, not `docs/technical-manual/src/60-back-matter/65-troubleshooting.md:179`, not `docs/technical-manual/src/50-rust-api/54-mnemonic-toolkit-api.md:163`. Doing so would recreate the exact defect this SPEC's §1 fact 3 condemns.

**Mechanism:** `parse_descriptor.rs` already produces `DescriptorParse` natively (doc `:48`, ~20 sites). `:1445/:1449/:1468` are pure `.map_err(...)` re-wraps — the edit deletes the wrapper. `:1371/:1514/:1678` construct a `DescriptorParse` with the same detail string.

## 4. Blast radius (verified, §9)

- **Toolkit tests:** exactly ONE pins this — `tests/cli_cycleA_phase2_funds_proof.rs:191-231` (asserts `Some(4)` + `"re-parse failed"` + `"multipath"`). It must flip, loudly and deliberately (§7). Other verify-bundle exit-4 assertions (`cli_descriptor_mode.rs:201`, `cli_verify_bundle_entropy_slot.rs:230`) are genuine card-mismatch / wrong-passphrase cases — **untouched**.
- **GUI:** `app_window.rs:1189-1207` branches on exit 4 *generically* for all subcommands. **Zero GUI edits needed**; no GUI test asserts `code(4)`/`Some(4)`. The change turns a misleading amber VERIFY-ME badge into a plain `exit: 2` + stderr — strictly better. `schema_mirror` gates flag names only; no flag changes.
- **Docs:** `41-mnemonic.md:196-201` and `:728-733` rewritten in lockstep (and the `:231` self-contradiction resolved). `docs/technical-manual/src/60-back-matter/65-troubleshooting.md:179` + `docs/technical-manual/src/50-rust-api/54-mnemonic-toolkit-api.md:163` keep the variant row with narrowed scope.
- **Scripts/CI:** no `scripts/`, `.github/`, Makefile, or manual-lint branching on exit 4 (grep: zero).
- **Other verify-bundle exit-4 cells (R0 round 2 MINOR):** `cli_verify_bundle_partial.rs` (descriptor-path partial cell `:312-338` asserts exit 4; its sibling `:341-364` asserts exit 0; four more exit-4 cells are template-mode) and `cli_verify_bundle_md1_template.rs:323/:627` also assert exit 4 — all genuine result-tier, untouched, gated by the full suite.
- **Do NOT confuse with a lookalike:** `bundle.rs:2109` emits `"--import-json: descriptor re-parse failed:"` at **exit 2** on the `--import-json` path. Different surface, different tier, out of scope — an implementer greping the prefix must not "fix" it.
- **Doc-comments in tests (R0 MINOR):** `cli_verify_bundle_entropy_slot.rs:6` names the catch-all as "(`DescriptorReparseFailed`, exit 4)" and `cli_cycleA_phase2_funds_proof.rs:187` cites the stale `verify_bundle.rs:1375`. Both update in lockstep.
- **Not silent:** the split IS documented, so this is a documented-contract change caught by a pinned test — not a silent break. Precedent: v0.85.0 changed `--bsms-round1` failure exit 0 → 4 with an explicit `$?`-migration note (`41-mnemonic.md:1375`).

## 5. SemVer + migration

**MINOR → v0.97.0.** Not a wire/flag change (no schema-mirror, no manual flag-lint impact).

CHANGELOG + both manual paragraphs carry (note the enumerated classes — R0 MINOR — and the corrected exit-4 meaning set — R0 I-1):

> `verify-bundle --descriptor` / `--descriptor-file`: descriptor **input** errors now exit **2** (`DescriptorParse`) — same as `bundle` — instead of 4. This covers a malformed/unparseable descriptor (lex, placeholder-resolve, or parse), an unreadable `--descriptor-file`, and unsupported `--slot` subkeys. Exit 4 from descriptor-mode verify now means: **the cards mismatched**, **a dead/pathless card partial-decoded** (`result: partial`, v0.88.0), or **BIP-388 key-distinctness failed** (`Bip388VerifyDistinctness`). `$?`-gated scripts treating exit 4 from this path as "possible bundle corruption" should treat exit 2 as "fix your input".

**Also user-visible:** the stderr prefix `descriptor re-parse failed during verify-bundle: ` disappears for these classes; the message becomes the native `DescriptorParse` text. No consumer greps it except the flipped test (§9).

**`result: partial` (R0 round 2 I-2):** the v0.88.0 pathless/dead-card verdict fires on this very path — `tests/cli_verify_bundle_partial.rs:312-338` runs `verify-bundle --descriptor` with an elided md1 and asserts exit 4 + `result: partial`; producer `verify_bundle.rs:1808` in the shared `verify_emit_from_expected` tail. It needs no new test (already comprehensively pinned) but MUST appear in the meaning set — omitting it would repeat §1 fact 3's defect.

**Explicitly NOT in the meaning set:** "the completed keyed wallet failed re-parse". That mechanism is unreachable (§3 C-1) and must not appear in any user-facing doc.

## 6. Test surface (TDD — RED first)

R0 I-2: the first draft pinned only `:1445` and `:1371`, so an implementer could delete two re-wraps, miss `:1449`/`:1468`/`:1678`, and ship a half-fixed contract with the whole suite GREEN. **One RED-first cell per REACHABLE flipped site is mandatory.** Repros below are runtime-verified.

- **T1 (flip the pin, `:1445`).** `cli_cycleA_phase2_funds_proof.rs:191-231` → assert exit **2** + the multipath remedy. `"re-parse failed"` disappears with the wrapper. Also refresh its stale `verify_bundle.rs:1375` citation → `:1445` (R0 MINOR).
- **T2 (`:1449` resolve-gap).** `wsh(multi(2,@0/<0;1>/*,@2/<0;1>/*))` → currently 4, must become 2.
- **T3 (`:1468` probe-only).** `wsh(pk(@0/<0;1>/*),pk(@1/<0;1>/*))` → currently 4 ("unrecognized name 'wsh'"), must become 2.
- **T4 (`:1678` slot-subkey catch-all).** `--slot @0.wif=…` → currently 4, must become 2. History shows this is a live drift site (`cli_verify_bundle_entropy_slot.rs:1-10`).
- **T5 (`:1371` file read).** Missing `--descriptor-file` → exit 2 on **both** surfaces.
- **T6 (emit↔verify parity).** One malformed `@N` descriptor to BOTH `bundle` and `verify-bundle`, asserting both exit 2 — pins the asymmetry that triggered this cycle so it cannot reopen.
- **T7 (no collateral).** `cli_descriptor_mode.rs:201` and `cli_verify_bundle_entropy_slot.rs:230` stay GREEN at exit 4 — proves the card-mismatch tier is untouched. Add a cell pinning `Bip388VerifyDistinctness` still exits 4, since §5 now advertises it.
- **T8 (dead sites, honesty not ceremony).** `:1514` and `:1722` are unreachable (R-2, C-1). Do **NOT** contrive tests for them. Record their status in the test-module doc and in a code comment at each site.

T1-T6 must each be shown RED against v0.96.0 before the edit lands.

## 7. The reversal case (for the reviewer to judge directly)

**What I-B actually was (corrected by R0 round 1).** cycleA plan-R0 finding I-B was an *assertion-accuracy* correction: the cycleA plan's Phase-3a test asserted the wrong variant for the **concrete** fork; the reviewer fixed that and pinned the @N fork's incumbent exit 4, marking the exit-4 test **"(optional)"** and noting "only the assertion shape is wrong". It was not a de-novo ruling that 4 is the right tier for @N intake errors, and it did not weigh the GUI's VERIFY-ME semantics or the emit/verify parity cost. There is no considered design decision here to defer to — only an incumbent-behavior pin.

**On the merits:**
1. Exit 4's contract presupposes a verification *result* (`app_window.rs:1180-1207`); a lex reject on flag intake produces none.
2. The identical input exits 2 on `bundle` (runtime-verified, §6 T6).
3. The pin's own stated rationale — the manual's "when the completed wallet is re-parsed" — describes a mechanism proven never to fire (§3 C-1).

**Distinguishing the one legitimate verify=4-on-input precedent (R0 I-1).** `Bip388VerifyDistinctness` (v0.19.0 SPEC §4.11.c) deliberately tiers a user-input condition to 4 on the verify surface. It is **not** a counter-example to this SPEC's principle — it *supports* it. §4.11.c targets legacy v0.2 self-multisig **artifacts** being re-verified: a distinctness failure there says something about the artifact's construction, so "confirm out-of-band" is the honest advice. A malformed descriptor string says nothing about any artifact. The principle stands: **exit 4 speaks about an artifact or a result; exit 2 rejects input.** `Bip388VerifyDistinctness` is explicitly OUT OF SCOPE here and keeps its exit 4 (pinned by T7).

**If the reviewer judges the pin should stand,** the correct outcome is not "leave as-is silently" but: keep exit 4 **and** fix the manual so its rationale cites the actual firing site (`lex_placeholders`), plus resolve the `:231` self-contradiction. Say so explicitly.

## 8. Out of scope

- The `DescriptorReparseFailed` **enum variant** (retained; its only remaining arm is the unreachable defensive `:1722`).
- `Bip388VerifyDistinctness` and the v0.19.0 SPEC §4.11.c per-surface split (keeps exit 4; see §7).
- Any other exit-tier review across the CLI.
- The GUI (no edits required; behavior improves for free).

## 9. Verification performed for this SPEC

Re-grepped at v0.96.0: the 7 `DescriptorReparseFailed` construction sites are exactly `verify_bundle.rs:1371, 1445, 1449, 1468, 1514, 1678, 1722` — none elsewhere in `crates/`. `grep 'from_import|import_json' verify_bundle.rs` → zero. Quickstart/ultraquickstart/examples for `reparse`/`exit 4` → zero. `scripts/`, `.github/`, Makefile, GUI tests for `-eq 4`/`== 4`/`code(4)`/`Some(4)` → zero. Toolkit tests for `"re-parse failed"` → only `cli_cycleA_phase2_funds_proof.rs:228` (plus an inert doc-comment hit at `cli_bundle_import_json.rs:909`).

**R-1 / R-2 / R-3 — all RESOLVED by R0 round 1, re-verified for this fold:**
- **R-1 → `:1722` is structurally unreachable** (not merely un-reproduced). Confirmed here: `parse_descriptor`'s fallible steps read `input` only; `keys`/`fingerprints` touch infallible map/sort/assign at `:980-1003`.
- **R-2 → `:1514` is dead.** Confirmed here by live probe: n=3 with one slot → exit 2 from the shared coverage gate.
- **R-3 → triggered.** Ruling folded into §3: retain the enum variant, keep `:1722` as an honest defensive arm, and bar "completed wallet re-parse" from every user-facing doc.

**Status after fold:** R0 round 1 returned 1C/2I; round 2 returned 3I — **two of which were defects the round-1 fold itself introduced** (a reachable-site miscount, and a migration note that omitted `result: partial`), which is the concrete justification for this project's re-review-after-every-fold rule. Round 2's third finding was a process violation: the round-1 review had not been persisted before its fold. Both reviews are now on disk. Round 3 required.
