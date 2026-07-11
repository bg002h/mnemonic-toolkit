# RECON — burndown plan for 6 FOLLOWUPs generated this session

Read-only recon. Source SHAs at time of verification: toolkit `HEAD=14127582962a12948376cd5ea6f90fc4e5e19f98` (worktree = `origin/master`, `mnemonic-toolkit v0.85.0`), gui `HEAD=fac2521b49107cb119b39f913fd05bf8c677678a` (`mnemonic-gui v0.58.0`). All citations below are grep-verified against these SHAs, not lifted from `FOLLOWUPS.md` snapshots.

---

## 1. `slip39-library-layer-passphrase-message-sweep` (toolkit)

- **Locus (confirmed, unchanged from FOLLOWUP):** `crates/mnemonic-toolkit/src/slip39/error.rs`
  - Doc comment lines 76–79 (on `DigestVerificationFailed`): *"Most commonly: wrong passphrase, or a substituted share whose metadata matches but value bytes diverge."*
  - `Display` impl lines 176–179: `"slip39: reconstructed master digest mismatch (wrong passphrase or substituted share)"`.
  - M2's fixed CLI-facing wording lives at `crates/mnemonic-toolkit/src/cmd/slip39.rs:757` — confirmed correct, states the digest check runs **before** passphrase decryption and explicitly does **not** verify `--passphrase`.
- **CLI-unreachable, confirmed:** grepped every `Slip39Error` reference in `src/`; the only consumer is `map_slip39_error` (`cmd/slip39.rs:724`), called from 3 sites (513/641/647), which uses its own independent match arms (`cmd/slip39.rs:725+`) — it never calls `Slip39Error::Display` or the `?`-based `From` conversion. No `impl From<Slip39Error> for ToolkitError` exists. So the library-layer string is dead from the shipped binary's perspective; only reachable by an external crate consuming `mnemonic-toolkit` as a lib dep (the `slip39` module is `pub` unconditionally at `lib.rs:112`, **not** gated by `#[cfg(fuzzing)]`).
- **Fix:** sweep the doc comment (4 lines) + `Display` string (2 lines, ~1 sentence) to the M2 wording ("digest catches share substitution/corruption/wrong-secret; runs pre-decryption; does not verify passphrase").
- **LOC:** ~8–12 lines changed, one file.
- **Tests affected:** none. Grepped for any test asserting the old wrong string verbatim — zero hits. Zero RED risk.
- **SemVer:** touches compiled `src/` (not just comments — the `Display` arm's message text is executable), but zero observable CLI behavior change (unreachable). **PATCH-tier at most**; realistically bundled into the next release rather than shipped standalone (mirrors how M2/M3/M4 were bundled into v0.85.0).
- **Lockstep:** none — no manual/GUI surface touches a library `Display` string that's CLI-unreachable.
- **Decision needed:** No. Purely mechanical text sweep, zero ambiguity.

---

## 2. `bsms-round1-lenient-exit4-workflow-doc` (toolkit)

- **Locus (confirmed):** `docs/manual/src/30-workflows/3A-bsms-round1-verify.md`, "Lenient default behavior" prose at lines ~65–72 and "Strict mode" section ~73–96. Grepped the whole file for `exit 4` / `lenient` / `strict` — the lenient-failure exit code is **never mentioned**; only "Strict mode... flips failure to fatal `BsmsSignatureMismatch` exit 2" is documented.
- **Ground-truthed the underlying behavior it's missing:** `crates/mnemonic-toolkit/src/cmd/import_wallet.rs` — the standalone early-return (lines ~420–427) and the combined `--blob` tail (lines ~1380–1383) both compute `any_failed`/`any_round1_failed` over `Round1VerificationStatus::Failed` and `return Ok(if any_failed { 4 } else { 0 })`. Confirmed live in v0.85.0 (CHANGELOG `[0.85.0]` M4 entry corroborates). The manual's flag-reference row (`41-mnemonic.md:1350`) already documents this fully; only the workflow chapter is silent.
- **Fix:** one-line addition to 3A's lenient-vs-strict prose noting lenient-failure = exit 4 (report still printed), per the FOLLOWUP's own scoping.
- **LOC:** ~3–6 lines, prose only. No `{.text include=...}` transcript blocks anywhere in this file (grepped — zero hits), so **no `verify-examples`/transcript regen is triggered**.
- **SemVer:** NO-BUMP. Pure prose; precedent for standalone doc commits with no version bump exists throughout the git log (e.g. `88fa3845 docs(manual): P2 — ms1-repair demote exit-code lockstep...`, `99cba5e1 docs(cycleC-P0b): ... (NO-BUMP)`).
- **Lockstep:** none.
- **Decision needed:** No.

---

## 3. `mnemonic-stderr-template-encrypted-notice-label` (toolkit)

- **Locus (confirmed exact):** `docs/manual/src/40-cli-reference/41-mnemonic.md:1421`:
  `| NOTICE (exit 0) | `notice: import-wallet: --bsms-round1: BIP-129 encrypted Round-1 record <i> decrypted (token width <N> hex chars; MAC verified)` (v0.32.1) |`
  This is the encrypted-Round-1 **decrypt** NOTICE (not the sig-fail NOTICE). Confirmed via `import_wallet.rs` line ~2603 (`writeln!(stderr, "notice: ... decrypted (token width {} hex chars; MAC verified)"...)`) that this NOTICE fires independently of, and can co-occur with, the post-M4 exit-4 sig-fail path.
- **Also confirmed the "pre-existing gap":** grepped `41-mnemonic.md` for the sig-fail NOTICE text (`"signature verification failed for record"`, emitted at `import_wallet.rs:2639`) — **zero hits**. That NOTICE has no row in the table at all, exactly as the FOLLOWUP states.
- **Fix:** drop/qualify the "(exit 0)" label on the row-1421 NOTICE (it labels the notice's own class correctly but the invocation's overall exit code can now be 4); optionally add a new row for the sig-fail NOTICE (`notice: import-wallet: --bsms-round1: signature verification failed for record <i> (signer pubkey <X>): <reason>`, exit 4).
- **LOC:** ~2–8 lines (1 label edit; +1 optional new table row).
- **SemVer:** NO-BUMP, same reasoning as #2. No transcript regen (this table is hand-written prose, not a golden include).
- **Lockstep:** none.
- **Decision needed:** Minor judgment call — whether to also add the sig-fail NOTICE row (the FOLLOWUP says "optionally"). Recommend: do it now since it's ~4 extra lines and closes a real gap while the file is already open.

---

## 4. `gui-canonicity-suffix-origin-h-fixture` (gui)

- **Locus (confirmed):** `mnemonic-gui/tests/canonicity_drift.rs`, `FIXTURES` const (lines 105–175). Confirmed the existing h-notation grid (T5, lines 136–174) covers: origin-fingerprint-h prefix (`147`), use-site-h suffix marker only (`149`), `tr` prefix-h (`151`), mixed apostrophe+h prefix (`153`), double-marker rejection (`161`), `wsh(multi)` prefix-h (`172`) — **none** is a suffix-origin `@N[fp/path]` bracket group with `h`. The one suffix-origin row that exists (`119`: `wpkh(@0[deadbeef/84'/0'/0']/<0;1>/*)`) is apostrophe-only. FOLLOWUP claim confirmed accurate — real blind spot.
- **Empirically re-verified the proposed fixture** (not just trusted the FOLLOWUP's prior R0 claim): ran the actual pinned binary —
  ```
  $ mnemonic --version
  mnemonic 0.75.0          # matches gui's pinned-upstream.toml:22 tag exactly
  $ mnemonic gui-schema --classify-descriptor 'wpkh(@0[deadbeef/84h/0h/0h]/<0;1>/*)'
  canonical                 # exit 0
  ```
  Confirmed live, not just cited.
- **Fix:** add one row to `FIXTURES`: `("wpkh(@0[deadbeef/84h/0h/0h]/<0;1>/*)", Expect::Canonical),` — matches the array's existing shape exactly.
- **LOC:** 1 line (+ a one-line comment, as the existing rows do).
- **SemVer:** NO-BUMP — test-only, zero `src/` touch.
- **Lockstep:** none (pure fixture add; gate is self-contained).
- **Decision needed:** No.

---

## 5. `repair-single-string-fully-valid-alias-second-oracle` (toolkit, FUNDS-ADJACENT) — see full analysis below.

## 6. `schema-mirror-defaults-drift-md-ms-mk-extension` (gui)

- **Locus (confirmed):** `mnemonic-gui/tests/schema_mirror_defaults_drift.rs` docstring `## Scope` at lines 28–31: *"`mnemonic` only... Extending to `md`/`ms`/`mk` is a natural follow-on... deliberately out of this cycle to stay a bounded add."* The single test fn `mnemonic_defaults_and_choices_match_pinned_gui_schema` (lines 93–158) iterates `schema::mnemonic::SCHEMA.subcommands` only.
- **Confirmed the extension is mechanical**, not a design problem:
  - `mnemonic_gui::schema_check::json_flag_defaults`/`json_flag_choices` (`src/schema_check.rs:442+`) already take a `cli_name: &str` first argument and resolve the binary generically via `pinned-upstream.toml`'s `PinnedRoot` (which already has `mnemonic`/`md`/`ms`/`mk` entries, `MD_BIN`/`MS_BIN`/`MK_BIN` env overrides, `src/schema_check.rs:22-56`).
  - `schema::md::SCHEMA` / `schema::ms::SCHEMA` / `schema::mk::SCHEMA` already exist (`src/schema/{md,mk,ms}.rs`) with a `cli_name` field, and the sibling flag-NAME-only gate (`tests/schema_mirror.rs:122-140`) already has 4 separate `#[test]` fns (`mnemonic_schema_flag_names_match_help_text`, `md_...`, `ms_...`, `mk_...`) — the exact pattern to replicate.
  - The only non-mechanical part: `resolvable()` (lines 82-91) is currently hardcoded to probe `mnemonic --help` only; needs a per-CLI variant (mirror `schema_mirror.rs`'s `resolve_bin(cli_name)` at line 44-47) before each of the 3 new test fns can independently skip if that CLI's binary isn't resolvable.
- **Fix:** refactor the body of the existing test into a shared `fn assert_defaults_and_choices_match(schema: &schema::Schema)` (parameterized on `schema.cli_name` instead of the hardcoded `"mnemonic"` string), add 3 new `#[test]` fns (`md_defaults_and_choices_match_pinned_gui_schema`, `ms_...`, `mk_...`), update the `## Scope` docstring, and iterate to close any real `DEFAULT_VALUE_ALLOWLIST`-style divergences the run surfaces on md/ms/mk (unknown count until run — historically small, e.g. the existing 1-entry allowlist for `mnemonic`).
- **LOC:** ~80–130 lines (refactor + 3 test fns + doc update + likely 0–3 new allowlist entries discovered at execution time).
- **SemVer:** NO-BUMP — test-only, zero `src/` touch (unless `resolvable()`'s refactor needs a shared helper in `schema_check.rs`, which would still be test-infra, not shipped behavior).
- **Lockstep:** none required for this fix; it improves lockstep enforcement for the OTHER 3 CLIs going forward. Note: no companion FOLLOWUPS.md entry exists in gui's own `FOLLOWUPS.md` (checked — no hit for either #4 or #6's slug there); toolkit's `design/FOLLOWUPS.md` is the sole tracker for both gui-side items per this session's convention.
- **Decision needed:** No structural decision; only implementation-time judgment on any newly-discovered md/ms/mk allowlist entries (expected to be trivial/self-evident, same shape as the existing `compare-cost --feerate` entry).

---

# 5. DEEP DIVE — `repair-single-string-fully-valid-alias-second-oracle`

## Ground truth of the current architecture

Confirmed by reading `crates/mnemonic-toolkit/src/repair.rs` + the vendored `md-codec` source (`vendor/md-codec/src/chunk.rs`):

| Card class | Oracle beyond BCH residue | Locus |
|---|---|---|
| mk1, `Chunked` (≥1 real chunk_set_id group) | **Yes** — `mk_codec::decode` on the reassembled group is a genuine structural decode; multi-chunk groups reassemble through a real cross-chunk check | `repair.rs::verify_mk1_set` lines 1049–1062 |
| mk1, `SingleString` header variant | **No** — `group_is_complete_and_consistent` trivially returns `headers.len()==1`, then `mk_codec::decode(&refs)` is just a structural decode of the one string (no cross-string hash possible with one string) | `repair.rs::verify_mk1_set` lines 1049-1062, `GroupKey::SingleString` arm. **Confirmed unreachable from real v0.1 encoders** per the doc comment at `repair.rs:842-843` ("unreachable from real v0.1 encoders per SPEC §1 count=1 reachability... handled uniformly rather than assumed away") |
| md1, multi-chunk (`strings.len() > 1`, or `strings.len()==1` with `chunked_flag==1`) | **Yes** — `md_codec::chunk::reassemble` step 7 (`vendor/md-codec/src/chunk.rs:379-387`) re-derives `chunk_set_id` from the FULLY DECODED payload (`compute_md1_encoding_id` → `derive_chunk_set_id`) and requires it match every chunk header's embedded id — a genuine payload-derived cross-check, ~2⁻²⁰ false-accept | `md_codec::chunk::reassemble`, delegated via `repair.rs::repair_via_md_codec` (1586-1638) |
| md1, single-string (`strings.len()==1`, `chunked_flag==0`, the v0.30 §2.3 non-chunked form) | **No** — routes directly to `crate::decode::decode_md1_string` (`vendor/md-codec/src/chunk.rs:615-631`); the ONLY check performed is the BCH residue (already spent by the correction) + structural descriptor-template parse. No cross-chunk id, no payload-derived confirmation of any kind. | `md_codec::chunk::decode_with_correction`, `strings.len()==1` pre-pass, `chunked_flag==0` branch |

`repair_via_md_codec` (`repair.rs:1586-1638`) is **kind-blind** to this distinction — it unconditionally sets `set_verify: SetVerify::Blessed` regardless of whether the decode went through the cross-chunk-hash path or the unprotected single-payload path. This is the actual bug: the doc comment at `RepairOutcome::set_verify` (`repair.rs:451-453`) claims "`Md1` always reports `Blessed` (its sibling-codec delegate only ever returns `Ok` on full decode success already, so there is no behavior change from Cycle E)" — true as a statement about `Ok`-vs-`Err`, but it elides that "full decode success" for the single-string sub-case is a **much weaker** bar than for the multi-chunk sub-case.

**The `f4_c` KAT, confirmed:** `crates/mnemonic-toolkit/src/prop_repair_never_wrong.rs:441-456`, fn `f4_c_single_string_md1_correction_blesses_documented_residual`. Uses fixture `VALID_SINGLE_MD1 = "md1yqpqqxqq8xtwhw4xwn4qh"` (the same string as `cli_repair_md1_non_chunked.rs`'s `VALID_NON_CHUNKED_MD1`), flips one char, asserts `repair_card` returns `Ok` with `outcome.set_verify == SetVerify::Blessed`, and its own comment says: *"Flips RED-first when FOLLOWUP `repair-single-string-fully-valid-alias-second-oracle` adds a payload-derived second oracle / demotion."* — i.e. this test is deliberately written as a tripwire for whichever fix ships. **It WILL flip RED under DEMOTE** (must be updated as part of the fix, not a regression).

**A second test also flips**, not mentioned in the FOLLOWUP text but found by grep: `crates/mnemonic-toolkit/tests/cli_repair_md1_non_chunked.rs::non_chunked_md1_single_error_repair_exits_5_and_recovers` (lines 51-68) asserts `.code(5)` on `mnemonic repair --md1 <corrupted-single-string-fixture>`, using the identical fixture. Under DEMOTE this must become `.code(4)` and be renamed. No other test sites hit — grepped all md1 fixtures in `cli_auto_repair.rs` and `cli_repair.rs`; both use the 3-chunk `MD1_C0/C1/C2` fixture set, which retains the multi-chunk cross-check and is unaffected.

## Option DEMOTE — mirror the ms1 Cycle-F demotion

**What changes, mechanically:**
1. `repair.rs::repair_via_md_codec` (1586-1638): instead of unconditionally `set_verify: SetVerify::Blessed`, compute it conditionally. The clean, precise signal available to the toolkit at this call site is `chunks.len() == 1` (the caller supplied exactly one wire string) — proven above to correspond EXACTLY to the two single-string sub-cases (true non-chunked, or the rare legitimate "chunked-of-1" edge acknowledged in `md-codec`'s own comment at `chunk.rs:610-613`). When `chunks.len() == 1 && !repairs.is_empty()` (i.e. the correction actually touched something), report `SetVerify::Unverified { reason: ... }` (same reason-string shape as the ms1 arm at `repair.rs:1166-1171`) instead of `Blessed`. `chunks.len() > 1` is unaffected (always has the genuine `reassemble()` cross-chunk oracle) — **zero behavior change for chunked md1**.
2. `repair.rs::verify_mk1_set` (1049-1062): add a match on `key` — when `key` is `GroupKey::SingleString(_)` (as opposed to `Chunked`) and the group is touched, force `GroupVerdict::Candidate` rather than running `mk_codec::decode` for a `Bless` verdict. This branch is dead in practice today (SingleString mk1 unreachable from real encoders) — pure defensive hardening, zero live blast radius.
3. **No changes needed** to `cmd/repair.rs` or `try_repair_and_short_circuit` (`repair.rs:1704-1751`) — both are **already kind-generic** on `SetVerify`: `cmd/repair.rs`'s exit-code mapping (`repair.rs:173`, `verdict_str` at 361-365) treats any `Unverified` uniformly regardless of kind, and `try_repair_and_short_circuit`'s fall-through gate (`if !matches!(outcome.set_verify, SetVerify::Blessed) { ... return Ok(()); }`, line 1732) already routes ANY kind's Unverified outcome away from auto-fire short-circuit. This means the 4 auto-fire call sites (`cmd/inspect.rs:133`, `cmd/convert.rs:1051`, `cmd/xpub_search/seed_intake.rs:189`, `cmd/verify_bundle.rs` ×4 sites) need **zero code changes** — they inherit the new demotion automatically through the shared helper. The one small optional polish: the stderr forward-pointer advisory at `repair.rs:1738` ("run `mnemonic repair --ms1 …` to inspect it") is currently gated `matches!(kind, CardKind::Ms1)` — for UX parity it should widen to include `Md1` (and defensively `Mk1`), else md1 auto-fire callers just silently fall through to their original typed error with no forward pointer. Small (~5 LOC), not required for correctness.
4. Test updates: flip `f4_c_single_string_md1_correction_blesses_documented_residual` (rename + invert assertion to `Unverified`, drop the "documented residual" framing since it's now fixed) and `cli_repair_md1_non_chunked.rs`'s `.code(5)` → `.code(4)` (rename). Add one new positive smoke mirroring `f4_a_touched_ms1_correction_is_unverified` for md1 single-string, and (recommended) a companion negative test proving `chunks.len() > 1` md1 is UNCHANGED (still Blessed) as an explicit non-regression pin.
5. Manual: `41-mnemonic.md` currently frames md1 uniformly as blessed on a "content-id check" (line 3083: *"md1 (content-id check passes)"*) and "the reassembly hash mk1/md1 are ALREADY blessed on" (line ~3189) — both now become **inaccurate as blanket statements** once single-string demotes. Needs a new subsection analogous to the existing "ms1 substitution-correction demotion" one (anchor `#mnemonic-repair-ms1-substitution-demotion`, lines ~3155-3195) describing the md1 single-string carve-out, plus 1-2 new golden-transcript worked examples (this manual's convention gates CLI-output blocks via `verify-examples`/`{.text include=...}`, so a new worked example does require a transcript regen — bounded, ~1-2 new fixture files).
6. `CHANGELOG.md` entry, `[funds-adjacent]` tag, mirroring the F4/M4 style.

**LOC estimate:** ~15-25 (repair.rs core fix) + ~40-70 (test updates + new tests) + ~60-120 (manual subsection + transcript regen) + CHANGELOG ≈ **~150-220 LOC total**. Toolkit-only, no md-codec/mk-codec API change required (the `chunks.len()==1` proxy is precise enough — see reasoning above), no crates.io publish beyond the toolkit tag, no re-vendor, **no GUI schema-mirror change** (no new CLI flag — the existing `repair` contract already documents "ms1/mk1/md1" can all report VERIFY-ME candidates generically at `cmd/repair.rs:14` module doc and `:74`, so this is filling in an already-declared-generic contract, not adding new surface).

**SemVer:** MINOR (the repo's pre-1.0 "0.X = breaking axis" convention), mirroring the M4 exit-code-change precedent (`0.84.0 → 0.85.0`) and the F3/F4/F2 funds-tier fixes — this changes a real, user-reachable behavior (`mnemonic repair --md1 <single-string-card>` with a genuine correction goes from exit 5 to exit 4) that `$?`-gated automation could observe, so it needs the same BREAKING-callout treatment v0.85.0 gave M4.

**Does `f4_c` flip RED?** Yes, confirmed above — by design (the test's own comment says so). This is the expected/required RED-first proof, not a regression.

## Option SECOND-ORACLE — payload-derived re-derivation + user confirmation

Sketch of what this needs, to make the complexity delta concrete:

1. A new `repair` (and/or `convert`/`inspect`) flag analogous to `restore`'s `--expect-wallet-id`/`--expect-xpub` (`41-mnemonic.md:1034/1043-1044`) — e.g. `--expect-wallet-id <PREFIX>` for md1, `--expect-xpub <XPUB>` for mk1 — supplied by the user as an independent ground truth.
2. New wiring: after a single-string correction, re-derive the relevant identity from the corrected payload (a `WalletPolicyId`-style hash for md1 descriptors; the xpub bytes themselves for mk1, which is somewhat circular since the "identity" of an xpub-bearing card largely *is* its payload — the strongest available second-oracle for mk1 single-string is arguably re-deriving a receive address and asking the user to eyeball it against a known-good backup, which is a UX flow, not a machine check) — compare against the user-supplied reference; Bless only on match, else stay Unverified.
3. Without a supplied reference, **there is no way to autonomously Bless** — so this option's "Bless" path is only reachable in an interactive/scripted-confirmation flow; the *default* no-flag behavior would have to be the SAME as DEMOTE (Unverified) anyway. Net effect: SECOND-ORACLE is a strict superset of DEMOTE — DEMOTE is required first as the safe default, and SECOND-ORACLE is an *additive* opt-in escape hatch for callers who have an independent reference available.
4. This is new CLI surface (new flags) → triggers the GUI schema-mirror lockstep (`mnemonic-gui/src/schema/mnemonic.rs`) and manual mirror in the SAME PR per the CLAUDE.md mirror invariant, plus its own design pass (the FOLLOWUP's own text for the *sibling* item `repair-single-string-fully-valid-alias-second-oracle`'s design-adjacent cousin — `pathless-wallet-backup-partial-decode` — explicitly calls out that this class of "what does a placeholder/confirmation contract look like" question "need[s] their own design pass, not a quick patch"). Realistically this is its own brainstorm→SPEC→R0→plan→R0→impl→post-impl pipeline, not a bugfix cycle.
5. LOC: rough order of magnitude 4-6x DEMOTE's (new flags × 2 kinds, wiring, GUI schema mirror, new manual sections, a whole new confirmation-UX design decision) — **300-500+ LOC**, plus a real, currently-undecided product question (what does "confirm" mean for an xpub card where the payload mostly *is* the identity?).

## Recommendation: **DEMOTE**

Funds-safety and consistency both favor it, exactly as the task brief anticipated:
- It's strictly the *safer default* — SECOND-ORACLE's own no-reference-supplied path degrades to DEMOTE's behavior anyway, so DEMOTE is not superseded by SECOND-ORACLE, it's *subsumed* by it as the mandatory floor.
- It brings md1/mk1 single-string to parity with ms1's existing Cycle-F treatment — the constellation already has exactly this pattern shipped and manual-documented for ms1; single-string md1 is the one remaining kind without it.
- **~150-220 LOC, toolkit-only, no GUI/schema-mirror change, no new CLI surface, most of the "blast radius" absorbed for free by already-generic `try_repair_and_short_circuit`/`cmd/repair.rs` code** — small, bounded, single-cycle work.
- SECOND-ORACLE is real, valuable, *additive* future work (an opt-in Bless path for callers who DO have a reference) but is a separate, larger, product-decision-bearing feature that can ship later without blocking the funds-safety floor.

**Blast radius of DEMOTE** (which real flows lose auto-bless):
- `mnemonic repair --md1 <corrupted-single-string-card>` (standalone): any invocation where the supplied card is a **single-string (non-chunked) md1 that needs an actual BCH correction** flips from exit 5 ("REPAIR_APPLIED, confidently") to exit 4 ("VERIFY-ME candidate"). This is the realistic, non-rare case — single-string md1 is the DEFAULT compact form `md encode` emits for descriptors that fit in one codeword (confirmed via the `cli_repair_md1_non_chunked.rs` background comment: *"the form emitted by plain `md encode` for small payloads"*), so this is a live, commonly-hit UX change, not a theoretical edge.
- The 4 auto-fire sites (`convert`, `inspect`, `xpub-search`, `verify-bundle`) on a raw single-string md1 that fails initial decode but is BCH-correctable: previously silently substituted the corrected card and proceeded (exit 5); now falls through to the ORIGINAL typed decode error (no silent substitution), matching ms1's existing auto-fire behavior exactly.
- mk1 `SingleString` blast radius: **effectively zero** — unreachable from real v0.1 encoders per the code's own documentation; this is pure defensive hardening with no live-user-facing change.
- Chunked/multi-chunk md1 and mk1: **zero change** — both retain their genuine cross-chunk/reassembly oracle and stay Blessed exactly as today.

---

# Batched execution plan

| Batch | Items | Repo | SemVer | Same-tree? | R0 gate needed? |
|---|---|---|---|---|---|
| **A — toolkit-docs-NO-BUMP** | #2 + #3 | toolkit | NO-BUMP | Same repo, different files (`3A-bsms-round1-verify.md`, `41-mnemonic.md`) — trivially combinable into one commit/PR, or 2 parallel edits then one commit | Lightweight review sufficient (mechanical prose); full brainstorm/SPEC/R0 pipeline is overkill for a 2-8 line doc fix, consistent with the many bare `docs(...)  (NO-BUMP)` commits already in the git log |
| **B — toolkit-hardening-PATCH** | #1 | toolkit | PATCH (bundle into whatever release ships next, or a standalone `0.85.1`) | Same repo as A, disjoint files (`slip39/error.rs`) — can run in the SAME PR as batch A with zero conflict risk | Same — mechanical text sweep, no RED-first needed (nothing pins the old text) |
| **C — toolkit-release (funds-adjacent)** | #5 (DEMOTE) | toolkit | MINOR, `0.85.0 → 0.86.0`, BREAKING-callout for `$?`-gated `repair --md1` callers | Same repo as A/B but **touches different files** (`repair.rs`, `prop_repair_never_wrong.rs`, `cli_repair_md1_non_chunked.rs`, `41-mnemonic.md`) — **should still be its own release/tag**, sequenced AFTER or independently of A/B (recommend after, so the trivial bundle ships first and doesn't wait on the R0 cycle) | **MANDATORY full R0 gate per CLAUDE.md** — funds-adjacent, brainstorm/SPEC/plan/per-phase/post-impl loop to 0C/0I, exactly as the FOLLOWUP itself flags ("Tier: funds-adjacent... Severity: MEDIUM") |
| **D — gui-tests-NO-BUMP** | #4 + #6 | gui | NO-BUMP | Different repo entirely — **fully parallel with A/B/C**, no coordination needed. Within gui: #4 (`canonicity_drift.rs`) and #6 (`schema_mirror_defaults_drift.rs`) are different files, could run as 2 parallel sub-agents or 1 combined PR | Lightweight review; both are mechanical (1-line fixture add; parameterize-and-replicate an existing 4-CLI test pattern) |

**Sequencing recommendation:** run **A+B** (toolkit, ~30 min) and **D** (gui, ~1-2 hrs, mostly #6's refactor) in parallel immediately — zero cross-repo or same-tree conflicts. Run **C** (#5, the funds-adjacent DEMOTE cycle) as its own R0-gated cycle, either right after A+B lands (so it isn't blocked waiting, and to avoid two concurrent toolkit worktrees touching `repair.rs`/manual files that A+B's docs-only changes don't touch anyway — genuinely no file overlap, so C *could* run concurrently with A+B in a separate worktree, but the R0 ceremony makes it the long pole regardless).

**Test/gate flip inventory for C (#5 DEMOTE):**
- `crates/mnemonic-toolkit/src/prop_repair_never_wrong.rs::f4_c_single_string_md1_correction_blesses_documented_residual` (line 441) — MUST flip (by design; the test's own comment predicts this).
- `crates/mnemonic-toolkit/tests/cli_repair_md1_non_chunked.rs::non_chunked_md1_single_error_repair_exits_5_and_recovers` (line 52) — MUST flip `.code(5)` → `.code(4)`, found by grep, **not mentioned in the FOLLOWUP text itself** — flag this explicitly to the implementer.
- `docs/manual/src/40-cli-reference/41-mnemonic.md` lines 3083 and ~3186-3189 — prose overclaims ("md1 content-id check passes" / "reassembly hash mk1/md1 are ALREADY blessed on") that become inaccurate blanket statements; need qualification regardless of exact new-subsection wording chosen.
- No other test sites hit (`cli_auto_repair.rs`, `cli_repair.rs` both use only the 3-chunk `MD1_C0/C1/C2` fixture, unaffected).
