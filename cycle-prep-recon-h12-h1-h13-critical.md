# cycle-prep recon — 2026-06-20 — H12 / H1 / H13 (constellation bug-hunt CRITICALs)

**Toolkit `origin/master` SHA at recon time:** `4d5872ed` (was `4d5872ed` in task header — UNCHANGED; v0.60.0 already merged, incl. P3a `c0f74994` + P4 `aaa67b74`)
**md-codec `origin/main` SHA at recon time:** `54dd765` (was `54dd765` — UNCHANGED, zero drift; **NO `origin/master` exists** — md/md-cli default branch is `main`)
**Local toolkit branch:** `feature/own-account-subset-search` (another instance's WIP — NOT trusted; verified against `origin/master` bytes)
**Sync state (toolkit):** 7 ahead / 0 behind `origin/master` (the 7 ahead are own-account-subset design commits; the 4 cited source files have an EMPTY diff vs `origin/master`, so working-tree == canonical for them — but all verification below is against `origin/master`/`origin/main` bytes regardless)
**Sync state (md):** 0 ahead / 0 behind `origin/main`
**Untracked (toolkit, persist across branches):** `cycle-prep-recon-bundle-md1-template-only-option.md`, `cycle-prep-recon-restore-md1-per-key-use-site-and-hardened-wildcard.md`, `cycle-prep-recon-restore-md1-taproot-use-site-override-arm.md`, `design/PLAN_constellation_bughunt_fix_program.md`, `design/agent-reports/constellation-bughunt-2026-06-20.md` (the hunt report + program plan that source these 3 findings)

Findings verified: **H12, H1, H13** (the 3 diff-oracle-escalated CRITICALs). Hunt-time snapshot @ toolkit `8967294d` / md `54dd765`; toolkit since drifted +2 commits onto `origin/master` (P3a `c0f74994` "restore", P4 `aaa67b74` "verify-bundle template completion"). **All 3 STILL REPRODUCE on current canonical source.** P3a/P4 added a *parallel* template-completion arm and did NOT touch the defective code paths.

---

## Per-finding verification

### H12 — descriptor-mode taproot multisig defaults cosigner origin to BIP-48 `2'` (P2WSH) instead of `3'` (P2TR)

- **WHAT:** `bundle --descriptor` (non-canonical descriptor, bare `@N` placeholders) infers a default cosigner origin via `compute_default_origin_path`, which hardcodes the BIP-48 script-type component to `2'` with NO taproot inspection. For `tr(NUMS,multi_a)` / `tr(sortedmulti_a)` every cosigner key lands in the `2'` (P2WSH) subtree instead of `3'` (P2TR) → every receive + change address diverges; coins unspendable by any BIP-48 coordinator (Sparrow/Coldcard/Jade re-derive at `3'`). Template-mode is correct (`H12-crossmode` proves template emits `3'`).

- **Citations:**
  - `cmd/bundle.rs::compute_default_origin_path` "reportedly still at `bundle.rs:2210`" — **ACCURATE.** `origin/master:crates/mnemonic-toolkit/src/cmd/bundle.rs:2210` `pub fn compute_default_origin_path(`. The body (2215-2234) builds a 4-component `OriginPath` with the 4th `PathComponent { hardened: true, value: 2 }` **hardcoded at line 2231**. Signature is `(network: CliNetwork, account: u32)` — takes NO descriptor/script-type/`Tag` argument, so it *structurally cannot* distinguish P2TR from P2WSH. The info-string at `:2280` also hardcodes `…/2'`.
  - mirror in `cmd/verify_bundle.rs` — **ACCURATE.** `origin/master:crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:1373` calls `crate::cmd::bundle::compute_default_origin_path(args.network, args.account)` (same 2-arg, same `2'` defect — symmetric verify mirrors the bug).
  - mirror in `xpub_search/descriptor_intake.rs` — **ACCURATE (separate helper, same defect).** `origin/master:.../descriptor_intake.rs:324` and `:345` call `bip48_default_path(network, account, 2)` with a **literal `2`**. The helper `bip48_default_path` (`:410-417`) *does* take a `script_type` param and forwards to `MultisigPathFamily::Bip48.default_origin_path(...)`, but both call sites pass `2`. Info-string `:397` hardcodes `…/2'`.
  - "CORRECT mapping `bip48_script_type()=3` for `TrMultiA/TrSortedMultiA` is in **descriptor-mnemonic** `crates/md-cli/src/parse/template.rs`" — **STRUCTURALLY-WRONG (wrong repo + wrong file).** `bip48_script_type` does **NOT exist anywhere in descriptor-mnemonic** (`git grep bip48_script_type origin/main` = 0 hits). It lives in the **TOOLKIT** at `origin/master:crates/mnemonic-toolkit/src/template.rs:231` — `pub fn bip48_script_type(&self) -> Option<u32>`, mapping `ShWsh* => Some(1)`, `Wsh* => Some(2)`, `TrMultiA | TrSortedMultiA => Some(3)`, `_ => None` (lines 232-237). The correct *template-mode* call site is `bundle.rs:530` (`template.bip48_script_type().unwrap_or(0)`) → `default_origin_path(...)`. So the fix-reuse target is in-repo, not a sibling-codec dep.

- **PROTOCOL FACT (BIP-48 `script_type`) — verified against PRIMARY SOURCE** (`bitcoin/bips` `bip-0048.mediawiki`, fetched live): BIP-48 standardizes **ONLY `1'` = Nested Segwit (P2SH-P2WSH)** and **`2'` = Native Segwit (P2WSH)**. It explicitly states "the only script types covered by this BIP are Native Segwit (p2wsh) and Nested Segwit (p2sh-p2wsh)" and does **NOT** define `3'` for P2TR/Taproot. The toolkit's own source agrees: `template.rs:243-262` + FOLLOWUP `multisig-tr-bip48-script-type-3-policy` document `3'`=taproot as a **toolkit/de-facto convention (Sparrow/Coldcard), NOT a BIP-48-standardized value** — honored under an explicit `--multisig-path-family bip48` with a non-standard-path stderr advisory. **Net:** the finding's "correct = `3'`" is right *as the constellation's own convention and what template-mode emits*; phrase the spec precisely (`3'` is the toolkit-consistent value, not a BIP-48 standard). The defect — descriptor-mode `2'` ≠ template-mode `3'` for the SAME taproot inputs → non-cosignable wallets — is real and reproduces regardless of the standardization nuance.

- **REPRODUCES: YES.** `compute_default_origin_path` (2-arg, hardcoded `2`) + the two mirrors are byte-present and taproot-blind on `origin/master`. P3a/P4 did not touch them.

- **Action for spec:** Make the default-origin inference taproot-aware: thread the descriptor's `Tag`/template into `compute_default_origin_path` (or branch at its 3 call sites) and emit `3'` when the wrapper is `Tag::Tr` (taproot `multi_a`/`sortedmulti_a`), reusing `template.rs::bip48_script_type():231` as the single source of the `1/2/3` mapping (fix-the-class: kill the literal `2` at `bundle.rs:2231`, `verify_bundle.rs:1373`, `descriptor_intake.rs:324,345` + the 2 info-strings `:2280`/`:397`). Fold `H12-crossmode` (descriptor-mode rejects `--multisig-path-family bip48` → no escape hatch) here. Cite source SHA `4d5872ed`. Preserve the existing non-standard-path stderr advisory semantics (`bip48_nonstandard_script_type_warning`).

---

### H1 — verify-bundle returns `result: ok` for an md1 that reconstructs a DIFFERENT wallet (threshold / sorted-vs-unsorted / script-type all uncompared)

- **WHAT:** For a keyed (wallet-policy) multisig md1, `verify-bundle`'s `md1_xpub_match` compares ONLY the **sorted pubkey multiset**. Threshold, policy-tree `Tag`, script-type wrapper (`wsh`/`sh`/`tr`), and key-order/slot binding are NEVER compared. So `sortedmulti(1,…)` (1-of-3 anyone-spends), unsorted `multi(2,…)`, and `sh(wsh(sortedmulti(2)))` all GREEN-light (exit 0) against an engraved `wsh(sortedmulti(2,A,B,C))`, even though every address differs. A genuinely-wrong cosigner xpub *does* surface (`result: mismatch`), proving the gate is structurally blind, not always-green.

- **Citations:**
  - `verify_bundle.rs:2406-2489` (`emit_multisig_checks` md1 block) — **DRIFTED-by-~+312 (function moved; defect intact).** P4 (`aaa67b74`) rewrote large parts of `verify_bundle.rs` (293 ± changed lines) but **did NOT touch this block's logic.** On `origin/master`: `fn emit_multisig_checks(` now starts at **`:2283`** (fn ends `:2808`); the md1 multiset-compare block is at **`:2718-2735`**:
    - `:2719-2730` extract `exp_pubs` / `act_pubs` (`tlv.pubkeys` bytes only),
    - `:2731-2734` `exp_sorted.sort(); act_sorted.sort();`,
    - `:2735` `let pubkeys_match = exp_sorted == act_sorted;` → the **sole** determinant of `md1_xpub_match`.
    The self-documenting comment at `:2718` reads "`md1_xpub_match (B.3: SPEC §5.7 multiset semantics, sort-then-compare)`." No threshold / `Tag` / wrapper / order compare anywhere in the arm.
  - "the keyless single-sig path at `:583` already does `expected.md1 == supplied.md1`" — **STRUCTURALLY-WRONG line (correct claim, wrong line).** `:583` on `origin/master` is now a `--origin`-override comment. The actual full-md1-string direct compare is at `verify_bundle.rs:645` — `let md1_match = expected.md1 == args.md1;` (the #28-phase-1 keyless single-sig recompose path). This is the exact fix-model the finding intends. **Also note** P4's NEW multisig-*template* arm at `:882` does `completed_template_id.as_bytes() == supplied_template_id.as_bytes()` — a full template-id compare — but that is the `--from`/`--cosigner` *template-completion* path, NOT the keyed-md1 `emit_multisig_checks` arm H1 targets. The two arms now diverge in rigor: template-completion is strict, keyed-md1 is multiset-blind.

- **PRIOR-ART (critical context):** FOLLOWUP `verify-bundle-multisig-md1-xpub-match-set-equality` (`origin/master:design/FOLLOWUPS.md:1635`) is marked **"resolved by v0.5.0 Phase B.3 (commit 9f1a4e7) — sort-then-compare multiset equality."** That historical "fix" deliberately moved the compare from *ordered-Vec pubkey equality* → *sorted-multiset pubkey equality* (to avoid descriptor-mode false-FAILs on `@N` reordering). **H1 is the regression-class of that very fix:** multiset-equality discards key-order binding (consensus-significant for unsorted `multi`/`multi_a`) AND was never threshold/script-type/tree-aware. The B.3 entry should be RE-OPENED, not treated as closed. (Sub-finding L24 in the hunt report — "sorted-multiset drops slot→key binding" — is "subsumed by H1's fix"; make the compare index-aware to close both.)

- **REPRODUCES: YES — and P4 did NOT close it.** P4 added a parallel strict template-completion arm but left the keyed-md1 multiset compare untouched at `:2718-2735`. The defect is structurally present on current `origin/master`.

- **Action for spec:** Replace the pubkey-multiset compare in `emit_multisig_checks` with a full canonical-md1 / policy-structure compare (mirror the single-sig `expected.md1 == args.md1` at `:645`, or a structural compare of decoded policy = threshold + `Tag` + wrapper + ordered/sorted slot binding). Re-open FOLLOWUP `verify-bundle-multisig-md1-xpub-match-set-equality` and fold L24. **De-dup with H12:** the bundle.rs↔verify_bundle.rs descriptor-mode binding is a shared zone (the PLAN's "S-VERIFY" workstream serializes H1 + H12 on one branch, Batch 0.5) — verify-bundle is the safety net that should also catch H12/H10/H13 structural drift and currently catches none. Cite source SHA `4d5872ed`.

---

### H13 — hardened multipath `<0h;1h>` / `<2';3'>` silently dropped at template lex → bare `/*` single-path key

- **WHAT:** The template/descriptor multipath-body lexer regex captures only `[0-9;]`, so a hardened alternative (`'`/`h`) never matches the multipath group and is silently dropped → the key collapses to a bare `/*` single-path. Separately, even an admitted multipath alt is hardcoded `hardened: false`. `md encode`/`md decode` (md-cli AND the toolkit `bundle --descriptor` md1) then render the bare-key wallet (e.g. `bcrt1qq0kxm9…`) instead of the intended hardened wallet (`bcrt1q5tgwjk…`). Core *rejects* `<0';1'>` ("not a valid uint32") → the correct behavior is to ERROR, not collapse. Two-repo lockstep (md-cli ↔ toolkit mirror). Note: non-hardened `<0;1>` is wire-correct → H13 is **hardened-specific**.

- **Citations (md-cli, `origin/main` @ `54dd765` — zero drift):**
  - `crates/md-cli/src/parse/template.rs:40` (multipath body regex `[0-9;]` can't match `'`/`h`) — **ACCURATE.** `origin/main:.../template.rs:40` `Regex::new(r"@(\d+)((?:/\d+'?)*)(?:/<([0-9;]+)>)?(/\*(?:'|h)?)?")`. Capture group 3 (multipath body) is `([0-9;]+)` — cannot match `'` or `h`. (The wildcard group 4 *does* accept `'|h`, and the origin-path group 2 accepts `'` — only the multipath alternatives are hardening-blind.)
  - `:220-233` (hardcoded `hardened: false`) — **ACCURATE.** `origin/main:.../template.rs:220-233` `fn make_use_site_path` maps each alt to `Alternative { hardened: false, value: *v }` (literal `false` at `:225`).
- **Citations (toolkit mirror, `origin/master` @ `4d5872ed` — zero drift):**
  - `crates/mnemonic-toolkit/src/parse_descriptor.rs:70` — **ACCURATE.** `origin/master:.../parse_descriptor.rs:70` regex `r"@(\d+)(?:\[([0-9a-fA-F]{8})((?:/\d+(?:'|h)?)*)\])?(?:/<([0-9;]+)>)?(/\*(?:'|h)?)?"`. Multipath group 4 is `([0-9;]+)` — same hardening-blind class. (The bracketed-origin group 3 accepts `'|h`; the multipath group does not.)
  - `:227-230` (hardcoded `hardened: false`) — **ACCURATE.** `origin/master:.../parse_descriptor.rs:223-231` `fn make_use_site_path` → `Alternative { hardened: false, value: *v }` (literal `false` at `:228`).

- **REPRODUCES: YES (both repos, zero citation drift).** All four citations are byte-exact on the canonical default branches.

- **Action for spec:** Two-repo lockstep. (1) Extend BOTH multipath regexes to admit `'`/`h` in the alternatives group (`[0-9;'h]` or `(?:\d+(?:'|h)?(?:;\d+(?:'|h)?)*)`), (2) parse the hardening per-alt instead of literal `hardened: false` in both `make_use_site_path`s, (3) Core-parity: ERROR on a hardened multipath alt that the wire/watch-only model can't faithfully represent rather than silently collapse (Core rejects `<0';1'>`). Order: **md-cli fix → tag → publish → toolkit pin bump** (codec-publish-before-pin). Companion FOLLOWUP entries in BOTH repos' `design/FOLLOWUPS.md` with cross-citing `Companion:` lines. Cite source SHAs toolkit `4d5872ed` + md `54dd765`.

---

## Cross-cutting observations

1. **NONE of the 3 findings was fixed by P3a/P4.** P3a (`c0f74994`) and P4 (`aaa67b74`) added a *parallel* keyless-template completion/recompose arm (`--from`/`--cosigner` permutation engine, strict `template_id` compare at `verify_bundle.rs:882`). The DEFECTIVE code paths — `compute_default_origin_path` (H12), the keyed-md1 `emit_multisig_checks` multiset compare (H1), and the multipath lexers (H13) — are all untouched and reproduce on `origin/master @ 4d5872ed`. P4 ironically *demonstrates* the correct strict-compare pattern (full-id equality) one arm away from where H1 needs it.

2. **`main`-vs-`master` branch gotcha (confirmed live):** descriptor-mnemonic (md-codec + md-cli) has **NO `origin/master`** — default is `origin/main`. Verifying H13's md-cli citations against `origin/master` would silently fail. md `origin/main @ 54dd765` matches the hunt snapshot exactly (zero drift). Toolkit default is `origin/master @ 4d5872ed`.

3. **One STRUCTURALLY-WRONG citation per CRITICAL, none fatal to the finding:**
   - H12 mis-attributed `bip48_script_type()` to **descriptor-mnemonic md-cli** — it actually lives in the **toolkit** (`template.rs:231`). The fix-reuse target is in-repo, NOT a sibling-codec dep (good — no codec tag/pin needed for H12).
   - H1's `:583` fix-model line drifted to `:645` (the `expected.md1 == args.md1` single-sig compare); the claim is correct, the line moved.
   - The H12 `bundle.rs:2210` / verify `:1373` and ALL H13 line numbers are byte-exact.

4. **H12 + H1 share the bundle.rs ↔ verify_bundle.rs descriptor-mode binding zone** (the PLAN's "S-VERIFY" workstream serializes them on one branch, Batch 0.5). verify-bundle (H1) is the safety net that *should* catch H12/H10/H13 structural drift and currently catches none — fixing H1 hardens the net against the whole wrong-address class. Do NOT parallelize H1 and H12 onto separate branches (shared-file merge hazard).

5. **H1 has stale prior-art that must be re-opened:** FOLLOWUP `verify-bundle-multisig-md1-xpub-match-set-equality` (`FOLLOWUPS.md:1635`) is marked RESOLVED by the very v0.5.0 multiset-equality change that H1 now flags as insufficient. Flip its status (re-open / supersede) in the shipping commit per the followup-status-discipline rule.

6. **Lockstep surfaces:** H13 is the only 2-repo item (md-cli↔toolkit, codec-publish-before-pin + companion FOLLOWUPs in both repos). H12 + H1 are toolkit-internal. **GUI `schema_mirror` + manual lockstep** trigger only if the cycle adds/renames a CLI flag, a dropdown value, or changes error/`--json` wire-shape — H12/H1 fixes are behavioral (default-origin value, compare semantics) and *should* be doable without a clap-flag change (PATCH-clean), but any new `--strict`/escape-hatch flag (e.g. an H12-crossmode `--multisig-path-family` opening in descriptor-mode, or an H1 `--allow-...`) WOULD drag both GUI schema-mirror (flag-name) and the manual under `docs/manual/src/40-cli-reference/`. The mandatory R0 gate applies before any code.

---

## Recommended brainstorm-session scope

**Group into ONE cycle (a single "S-VERIFY / wrong-address structural" brainstorm), three workstreams, this order:**

1. **H13 first (unblocks the lockstep clock).** 2-repo: md-cli regex+hardening fix → tag → publish to crates.io → toolkit mirror fix + pin bump. ~30-60 LOC across the two repos (2 regexes, 2 `make_use_site_path` bodies, + a Core-parity ERROR path) + companion FOLLOWUPs in both repos. **SemVer:** md-cli MINOR (behavioral correctness on a previously-silently-dropped input → it now errors; per the bug-hunt PLAN row both are MINOR) + toolkit MINOR (pin bump). Codec-publish-before-pin is a hard serial edge — start it early so the toolkit pin isn't the critical-path tail.

2. **H12 (toolkit-internal, fix-the-class).** Thread `Tag`/template into the default-origin inference; emit `3'` for taproot via `template.rs::bip48_script_type():231`; kill the literal `2` at all 3 call sites + 2 info-strings; preserve the non-standard-path advisory. Fold `H12-crossmode` (descriptor-mode `--multisig-path-family bip48` escape hatch). ~40-80 LOC + tests (bitcoind differential vs Core `deriveaddresses` for `tr(NUMS,multi_a)` / `sortedmulti_a`). **SemVer:** behavioral-correctness PATCH if no flag added; **MINOR if** the H12-crossmode escape hatch adds a descriptor-mode `--multisig-path-family` opening (→ GUI schema_mirror + manual lockstep).

3. **H1 (the structural anchor; same branch as H12).** Replace the keyed-md1 multiset compare in `emit_multisig_checks:2718-2735` with a full canonical-md1 / policy-structure compare (mirror `:645`). Re-open FOLLOWUP `:1635`; fold L24. ~30-60 LOC + tests (the 3 wrong-wallet GREEN-light cases from the hunt: `sortedmulti(1)`, unsorted `multi(2)`, `sh(wsh(sortedmulti(2)))` must all → `mismatch`). **SemVer:** behavioral-correctness PATCH if no flag added; MINOR if a `--strict`/escape flag is introduced. **Serialize H1 + H12 on one branch** (shared bundle.rs↔verify_bundle.rs zone).

**Total rough sizing:** ~100-200 LOC of fix + a meaningful test load (bitcoind differential oracle is the right gate for all three — all are wrong-address funds-safety). **Headline SemVer call:** PATCH/MINOR bug-fixes (no breaking API); H13 forces a toolkit MINOR via the pin bump and an md-cli MINOR via the publish. **Mandatory gates before any code:** R0 architect loop to 0C/0I on the brainstorm spec AND the plan-doc; single-subagent-per-phase TDD; post-impl whole-diff adversarial review. **Lockstep watch:** any clap-flag/dropdown/error-text/`--json`-shape change → GUI `schema_mirror` + `docs/manual/src/40-cli-reference/` in the same (or paired) PR; H13 companion FOLLOWUPs in both repos.

**Source SHAs to cite in the brainstorm spec:** toolkit `origin/master` `4d5872ed`; md `origin/main` `54dd765`. BIP-48 fact: `bitcoin/bips` `bip-0048.mediawiki` (`1'`/`2'` only; `3'`=P2TR is a toolkit/de-facto convention, NOT BIP-48-standardized).
