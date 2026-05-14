# Phase 3a Re-Scope Proposal v3 Review (R0-v3)

**Reviewer:** Opus 4.7 (1M context), `feature-dev:code-reviewer`
**Date:** 2026-05-13
**Proposal reviewed:** `/home/bcg/.claude/plans/2026-05-13-cycle-b-phase-3a-rescope-proposal.md`
**Prior round:** v2 R0 at `design/agent-reports/v0_9_B-phase-3a-rescope-r0.md` (4 Critical / 4 Important; declined)
**Verdict:** **RE-DRAFT NEEDED** — 1 Critical, 2 Important, 0 Nit

v3 is dramatically tighter than v2. C-1 (struct-sibling pin coverage), C-2 (per-handler anchors), C-3 (in-source tests instead of unrealizable integration plumbing), C-4 (honest engagement with the §10b cost calculus), I-1 (handler return-type framing), I-2 (bip85 names), and I-4 (lint mitigation) are all addressed correctly. I re-verified each against source. Only one foundational source-vs-narrative gap remains: v3 §3.3's `ResolvedSlot` struct skeleton names six fields that DO NOT MATCH the actual struct, and Task 2.1's compileable test code repeats those same wrong names. That's a hard compile-blocker for the test code v3 specifies, and it directly recreates the off-by-N narrative pattern the user has asked us to catch.

---

## CRITICAL findings

### C-1 (conf 100): v3 §3.3 `ResolvedSlot` field skeleton — every named field is wrong

**Source ground truth** (`crates/mnemonic-toolkit/src/synthesize.rs:578-592`, verified by direct read):

```rust
#[derive(Debug, Clone)]
pub struct ResolvedSlot {
    pub xpub: Xpub,
    pub fingerprint: Fingerprint,
    pub path: DerivationPath,
    pub path_raw: String,
    pub entropy: Option<Vec<u8>>,
    pub master_xpub: Option<Xpub>,
}
```

**Proposal claim (v3 §3.3, lines 167-176):**

```rust
#[derive(Debug, Clone)]
pub struct ResolvedSlot {
    pub kind: SlotKind,
    pub source: SlotSource,
    pub entropy: Option<Vec<u8>>,
    pub xpub: Option<String>,
    pub network: Network,
    pub language: Option<Language>,
    _entropy_pin: Option<Arc<PinnedPageRange>>,
}
```

Five of six existing field names are wrong:

| v3 claim | actual |
|---|---|
| `kind: SlotKind` | (no such field; no `SlotKind` type exists in synthesize.rs) |
| `source: SlotSource` | (no such field; `SlotSource` is not the type used here) |
| `entropy: Option<Vec<u8>>` | matches |
| `xpub: Option<String>` | actually `pub xpub: Xpub` (non-Option, `bitcoin::bip32::Xpub`, not `String`) |
| `network: Network` | (no `network` field on `ResolvedSlot`; and `Network` isn't even the toolkit's network type — it's `CliNetwork`) |
| `language: Option<Language>` | (no such field; `language` does not exist on `ResolvedSlot`) |
| (missing) | `pub fingerprint: Fingerprint` |
| (missing) | `pub path: DerivationPath` |
| (missing) | `pub path_raw: String` |
| (missing) | `pub master_xpub: Option<Xpub>` |

**Cascade into Task 2.1 (lines 291-301, the GREEN-time test):** the `let _slot = ResolvedSlot { kind: SlotKind::PrimarySigner, source: SlotSource::Phrase, entropy: ..., xpub: None, network: Network::Bitcoin, language: Some(Language::English), _entropy_pin: ... };` literal will fail to compile in Step 2.1 with at minimum:
- `error[E0560]: struct ResolvedSlot has no field named kind` (and same for `source`, `network`, `language`)
- `error[E0063]: missing fields fingerprint, path, path_raw, master_xpub in initializer of ResolvedSlot`
- `error[E0308]: mismatched types — expected Xpub, found Option<...>` on the `xpub: None` line

This is a compile-blocker per `feedback_r2_blocking_vs_cosmetic_gate`, and per `feedback_r0_must_read_source_off_by_n` it's the same off-by-N narrative pattern Phase 1, 2, 3a v1, and v2 all caught.

**Source citation:** `crates/mnemonic-toolkit/src/synthesize.rs:578-592`. Six existing ctor sites also confirm (`cmd/bundle.rs:348-355,417-424,449-456,491-498,1065-1072`; `synthesize.rs:1184-1196`).

**Correction:**
1. Rewrite v3 §3.3's struct skeleton to match the actual field set: `xpub: Xpub`, `fingerprint: Fingerprint`, `path: DerivationPath`, `path_raw: String`, `entropy: Option<Vec<u8>>`, `master_xpub: Option<Xpub>`, with `_entropy_pin: Option<Arc<PinnedPageRange>>` declared LAST.
2. Rewrite Task 2.1's `let _slot = ResolvedSlot {...}` literal to populate the correct field set. Use existing test fixtures (`synthesize.rs:1170-1198` already demonstrates this) — call `unified_fixture(1, &[0])` and assert against the resulting Vec[0]. Or write a `dummy_xpub()` helper inside the new test mod.
3. R1 task (Task 9.1, line 458) already lists struct-field-shape verification — but R1 fires AFTER Tasks 2-7. The error needs to be caught at proposal LOCK time, before any of Tasks 1-9 begin, so the Task 1 SPEC text and Task 2.1 RED test reference correct names from the start.

---

## IMPORTANT findings

### I-1 (conf 90): v3 §3.3 line 188 — `cmd/bundle.rs:417` is the Xpub arm (entropy: None), not the Entropy arm

**Proposal claim (v3 §3.3, lines 184-190):**

> 6 ResolvedSlot ctor sites updated:
> - `cmd/bundle.rs:348` (Phrase arm): ...
> - `cmd/bundle.rs:417` (Entropy arm — but verify; per reviewer this may be `:425-456`).
> - `cmd/bundle.rs:449`, `cmd/bundle.rs:491`, `cmd/bundle.rs:1065`.
> - `synthesize.rs:1184` (test ctor — pin a fresh test Vec).
>
> (Exact line numbers verified at GREEN time; one of the Bundle ctors may have shifted by a few lines after the v2 reviewer's read.)

I re-grepped: actual ctor lines are **348, 417, 449, 491, 1065** (`cmd/bundle.rs`) plus **1184** (`synthesize.rs`). Five of six v3 line numbers are correct as stated. The hedge "417 (Entropy arm — but verify; per reviewer this may be `:425-456`)" is wrong-direction: `:417` IS the **Xpub arm** (watch-only, `entropy: None`), not the Entropy arm. The Entropy arm is `:449`. The other five line numbers are pin-accurate.

The Xpub arm at `:417` and the Wif arm at `:491` both populate `entropy: None`, so both get `_entropy_pin: None` (no pin to construct). Only `:348`, `:449`, `:1065`, and `:1184` actually pin a real entropy buffer.

The closing parenthetical "Exact line numbers verified at GREEN time; one of the Bundle ctors may have shifted by a few lines" is unnecessary hedge — I just verified all six are at the asserted positions.

**Source citation:** `cmd/bundle.rs:417` (Xpub arm), `:449` (Entropy arm), `:491` (Wif arm); `synthesize.rs:1184` (test ctor).

**Correction:** Re-label the parenthetical: `cmd/bundle.rs:417` (Xpub arm; `entropy: None` — `_entropy_pin` populated as None). Drop the "Exact line numbers verified at GREEN time" hedge.

### I-2 (conf 85): v3 SPEC §6 G4.a rewrite text in Task 1.2 mis-describes Path B-lite

**Proposal claim (v3 Task 1.2, lines 246-248):**

> REMOVE the "Sites 2/3 use struct-field declaration order such that `entropy: Zeroizing<Vec<u8>>` drops BEFORE `_entropy_pin: PinnedPageRange`" sentence (Path B-lite has the OPPOSITE order: `entropy: Vec<u8>` drops first via Cycle A's `impl Drop` scrub, then `_entropy_pin` munlocks; rewrite as: "Sites 2/3: `_entropy_pin` is declared AFTER `entropy` so that on Drop, `entropy` scrubs first (via Cycle A's `impl Drop for DerivedAccount` for Site 3, or via Vec dealloc for Site 2), then `_entropy_pin` munlocks. The post-scrub-pre-munlock window is microseconds and not load-bearing.")

Two problems with this paragraph:

1. **"OPPOSITE order" is wrong.** The R0 v2 LOCK SPEC text already says "entropy drops BEFORE _entropy_pin." Path B-lite preserves that exact ordering — the field declaration is still `entropy, ..., _entropy_pin` (last). It is NOT "opposite" — it's the SAME ordering, just with a different `entropy` Drop semantic (Cycle A's `impl Drop` zeroize vs. Zeroizing's auto-zeroize). The drop ORDER of `entropy` then `_entropy_pin` is preserved.

2. **"Vec dealloc scrubs" is inflated for Site 2.** For Site 2 (`ResolvedSlot.entropy: Option<Vec<u8>>`) under Cycle A baseline, **there IS NO scrub** — the Vec is just deallocated without zeroizing (this is exactly why the deferred FOLLOWUP `resolved-slot-entropy-zeroizing-field` exists). Calling Vec dealloc "scrubs" inflates the security claim. The honest framing for Site 2 is: "entropy is deallocated (memory returned to allocator without scrub; bytes may persist on the heap until the next allocator-overwrite); then `_entropy_pin` munlocks."

**Correction:** Rewrite Task 1.2's proposed §6 G4.a sentence to:

> "Sites 2/3: `_entropy_pin` is declared AFTER `entropy` so that on Drop, `entropy` drops first — for Site 3 this triggers Cycle A's `impl Drop for DerivedAccount` zeroize; for Site 2 this is a plain Vec dealloc with no scrub (per the open FOLLOWUP `resolved-slot-derived-account-zeroizing-field`, deferred to v0.10.1). `_entropy_pin` then munlocks. For Site 3 the drop window is zeroize-then-munlock with the page still pinned during the zeroize. For Site 2 the bytes-may-persist-on-heap risk is unchanged from Cycle A (the page was pinned during the buffer's lifetime, but post-dealloc the bytes can persist in the freed allocation until allocator reuse; mlock does not address this)."

This honest framing surfaces the Site 2 gap rather than papering over it.

---

## NIT findings

(none — proposal is otherwise tight)

---

## v2 findings re-verification (each)

| v2 finding | v3 disposition | Verified |
|---|---|---|
| C-1 (Site 1 doesn't cover Sites 2/3) | Re-added struct-sibling pins on `ResolvedSlot._entropy_pin` (Arc-wrapped) and `DerivedAccount._entropy_pin` (plain). | YES — §1.1 lists this as KEPT; §3.3 declares the fields. (But see C-1 above for the field-name skeleton bug.) |
| C-2 (no `apply_stdin_substitutions` in convert/derive_child) | §3.1 has per-handler anchors with verified line numbers per handler. | YES — verified `bundle.rs:113-119`, `verify_bundle.rs:117-134`, `convert.rs:597-668`, `derive_child.rs:77-122`. All anchor positions correct. |
| C-3 (bip85 binary-private; integration plumbing unrealizable) | Switched to `#[cfg(test)] mod path_b_lite_pin_tests` in-source; uses existing `failure_count_for_test()` + `MNEMONIC_TEST_MLOCK_FAIL_MODE=eperm` pattern. | YES — `mlock.rs:226` `pub fn failure_count_for_test()` exists; subprocess `#[ignore]` pattern ships in Phase 2. The test code samples in §2.1-2.4 use this correctly. (But Task 2.1's struct literal still won't compile per C-1 above.) |
| C-4 (oversold deferral as no-cost) | §1.3 explicitly engages with §10b items 1+2; "structural-discipline gap stays at Cycle-A levels" framing is now honest. | YES — §1.3 paragraph 2 ("the trade-off is honest") is the explicit acknowledgement v2 demanded. |
| I-1 (R-3 wrong on 2 of 4 handlers) | §4 R-1..R-6 risk register no longer references handler return types. | YES — R-1..R-6 are about Arc lifetime, declaration order, mutating calls, structural-discipline gap, helper-macro nit, slots-clone gap. None reference handler return types. |
| I-2 (5 of 7 bip85 names wrong) | §3.2 has the verified table. | YES — re-grepped `bip85.rs:73,100,127,158,175,189,214` against §3.2 table; all 7 names match. |
| I-3 (SPEC supersession completeness) | Task 1.1 enumerates SPEC strip clauses; Task 1.2 §6 G4.a rewrite. | PARTIAL — Task 1.1 enumeration covers §2 row 5 and §4 P3a clauses correctly. But Task 1.2 §6 G4.a rewrite text itself has the inflated-Vec-dealloc-as-scrub bug per I-2 above. |
| I-4 (lint mitigation referenced but not scheduled) | §5 R-3 stance: "Audit at proposal-write time confirms zero such calls today. R1 reviewer re-confirms." | YES — clean audit-only stance; the v2 reviewer's audit confirmed zero `.push`/`.extend`/`.reserve`/etc. calls against the relevant fields, and R1 will re-confirm. Acceptable. |

---

## V3-specific verification

| Question | Result |
|---|---|
| V3-1 (ResolvedSlot fields) | **FAILED** — see C-1 above. |
| V3-2 (DerivedAccount fields) | OK — `derive.rs:20-26` actual fields are `entropy: Vec<u8>`, `master_fingerprint: Fingerprint`, `account_xpub: Xpub`, `account_xpriv: Xpriv`, `account_path: DerivationPath`. v3 §3.3 says "other fields unchanged" — that's accurate (no specific names asserted). Task 2.2 wisely doesn't include a struct literal sample; v3 just says "construct DerivedAccount with `_entropy_pin: pin_pages_for(...)`" without enumerating. Caveat: the GREEN-time test will need the same accuracy treatment as Task 2.1, so Task 2.2 implementer should grep `derive.rs:20` first. |
| V3-3 (ctor line numbers + caveat) | PARTIAL — five of six line numbers (348/449/491/1065/1184) verified correct; `:417` is mis-labeled as Entropy arm (it's actually the Xpub arm). See I-1 above. |
| V3-4 (R-6 slots-clone gap) | OK — verified `bundle.rs:203` `let slots = args.slot.clone();`. R-6's stance (b) is defensible: Sites 2/3 struct-sibling pins cover the derived bytes that are actually consumed downstream; Site 1 pin on `synthetic_args.slot[i].value` covers the substituted-argv String during the brief window between substitution and clone. The substituted bytes still leak briefly into the `slots[i].value` clone (which is unpinned) — but the substituted-argv String is always pinned at Site 1, and the derived bytes at `:354` `entropy: Some(entropy)` are pinned via Site 2's struct-sibling. The user-input String → cloned-String window is the only unpinned hop, and it's milliseconds. Acceptable as an explicit Path B-lite caveat; R-6 honestly enumerates this. |
| V3-5 (Arc not previously imported) | OK — `synthesize.rs` head imports list has no `std::sync::Arc`. v3's "add `use std::sync::Arc;`" phrasing is accurate. |
| V3-6 (DeriveChildArgs Default) | PARTIAL — `DeriveChildArgs` is `#[derive(Args, Debug, Clone)]` (NO Default) per `cmd/derive_child.rs:20`. So Task 2.4's sample requires every field be specified explicitly (no `..Default::default()` shortcut). The 9 fields are `from`, `application`, `length`, `index`, `network`, `passphrase_stdin`, `passphrase`, `language`, `dice_sides`. The `/* synthetic test args */` placeholder hides this — but the test author at GREEN time will be forced to spell out all 9 fields including building a `FromInput`. This is realizable, just unergonomic. Suggest adding a `cfg(test) fn make_synthetic_args() -> DeriveChildArgs` builder helper and call it from the test. Not a blocker; flagging for downstream awareness. |
| V3-7 (FOLLOWUP "superseded" precedent) | OK — verified three precedent uses in FOLLOWUPS.md. The existing `resolved-slot-entropy-zeroizing-field` entry has `Status: scheduled for closure in v0.9.0 Cycle B Phase 3a` (NOT just "open"; v3 §1.2 paraphrase mis-states "Status: open"). v3 should adjust the Task 1.3 phrasing accordingly. |
| V3-8 (G4.a rewrite honesty) | **FAILED** — see I-2 above. |

---

## Summary

| Severity | Count |
|---|---|
| Critical | 1 |
| Important | 2 |
| Nit | 0 |

**Verdict: RE-DRAFT NEEDED.**

v3 fixed every v2 finding except where new narrative was introduced. The remaining issues are concentrated in three places, all reachable by a 5-minute grep:

1. v3 §3.3 `ResolvedSlot` field skeleton + cascading Task 2.1 test code (read `synthesize.rs:578-592` and rewrite the literal).
2. v3 §3.3 `cmd/bundle.rs:417` arm label (read `bundle.rs:417` and re-label as Xpub arm with `entropy: None`).
3. v3 Task 1.2 SPEC §6 G4.a rewrite text (acknowledge Site 2 has no scrub under Path B-lite; don't conflate Vec dealloc with zeroize).

Once these three fold, v3 should LOCK on R0-v3-fold and proceed to Task 1.

The user has been thrashing on Phase 3a all day; v3 is genuinely close. C-1 here is a 5-minute grep-and-fix, not a re-architecture. The §6 G4.a rewrite (I-2) needs careful prose but no design rework. After fold, v3 can proceed.
