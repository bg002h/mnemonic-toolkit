# Verification Report — Phase 2 R0 Fix B fold

**Reviewer:** Opus 4.7 (1M context) as verifier on Phase 2 R0 Fix B SPEC + plan patches.
**Date:** 2026-05-13.
**Scope:** Cycle B Phase 2 R0 Fix B patches against SPEC (`design/SPEC_secret_memory_hygiene_v0_9_B.md`, commit `a49386f`) and plan (`~/.claude/plans/v0_9_B-secret-memory-hygiene-cycle-b.md`, not in git — operational doc). Cross-checked against R0 v1 design lock (`design/agent-reports/v0_9_B-phase-2-mlock-module-r0.md`, commit `8193e22`) and Phase 1 R0/R1 reports.
**Verdict:** **FOLD** — 0 Critical / 1 Important / 0 Nit. SPEC clean; plan had residual `MlockedZeroizing` drift in three Phase 3a/3b prescriptive sub-sections (P3a.T1 line 382, P3a.T3 commit-message template lines 444-446, Phase 3b sections lines 498/535/541/554/601). Folded after this verifier pass — see post-verify commit for the resolution.

---

## Summary

Total findings at confidence ≥ 80: **0 Critical / 1 Important / 0 Nit**.

The SPEC patches at commit `a49386f` correctly absorb Fix B in every load-bearing location: §2 row 1, §2 row 5, §4 P2, §4 P3a, §6 G4, §6 G6, and the frontmatter status line. No SPEC drift.

The plan correctly captures Fix B in the Phase 2 sections (P2.T1 DONE summary, P2.T2 RED-test list, P2.T3 GREEN impl order, P2.T4 CI workflow expansion, P2.T5 R1 checklist), the Phase 3a P3a header narrative, the file inventory, and the LOC totals. The single Important finding (I-1) — pre-Fix-B `MlockedZeroizing` vocabulary in three Phase 3a/3b prescriptive sub-sections — was folded after this verification.

---

## §1. SPEC patch verification (against R0 v1 recommendations)

| R0 v1 recommendation | SPEC location | Status |
|---|---|---|
| §2 row 1: wrapper API removed; slice-fn API is sole primitive | §2 row 1 line 34 | LANDED. Row 1 is renamed "API: slice fn (single primitive)" and lists `pin_pages_for`. The rejection of `Box<Zeroizing<T>>` is explicitly cited with the C-1 finding ID and commit `8193e22`. |
| §2 row 5: Sites 2-4 use `_entropy_pin` sibling field / function-local pin (NOT type swap) | §2 row 5 line 38 | LANDED. Site 2 uses `Option<PinnedPageRange>` sibling, Site 3 uses `PinnedPageRange` sibling, Site 4 uses function-local. Declaration-order discipline cited (RFC 1857). Site 4 reverse-drop trade-off honestly documented. |
| §4 P2: `MlockedZeroizing<T>` paragraph removed | §4 P2 lines 81-97 | LANDED. The "Phase 2's design was patched in-flight" prefix at line 83 notes the retirement. Module surface lists only `pin_pages_for` + `PinnedPageRange` + `MlockState` + `report_at_exit` + `page_size`. |
| §4 P2: `lib.rs` shape locked (Option C) | §4 P2 line 85 | LANDED. "Crate-shape change: create `crates/mnemonic-toolkit/src/lib.rs` exposing `pub mod mlock;`". |
| §4 P2: CI workflow scope expanded (rust.yml from scratch) | §4 P2 line 92 | LANDED. Three jobs (test/miri/clippy) named with matrix. |
| §4 P2: lint_safety_first_party_mlock cited | §4 P2 line 95 | LANDED. Peer-of-third-party-blocked lint named. |
| §4 P3a: Sites 2-4 retyped per Fix B | §4 P3a lines 105-108 | LANDED. Sites 2/3 sibling fields with declaration order; Site 4 function-local pin with reverse-drop honest trade-off. |
| §6 G4: reframed (no wrapper drop-probe); Cycle A discipline for G4.a; Miri for G4.b | §6 G4 lines 193-199 | LANDED. G4 split into G4.a (Cycle A discipline + lint_zeroize_discipline) and G4.b (Miri on 2 unsafe blocks). Explicit "No new cfg(test) drop-probe wrapper API is needed (Fix B eliminates `MlockedZeroizing<T>`)" at line 197. |
| §6 G6: no `MlockedZeroizing<T>` carve-out; full equivalence | §6 G6 lines 207-230 | LANDED. "Under Fix B (no wrapper type) the manifest is the complete `mlock.rs` surface — no toolkit-only carve-out". Name-export parity explicit. |
| Frontmatter status: Phase 1 SHIPPED + Phase 0 R3 Fix B fold | line 4 + line 8 | LANDED. Phase 1 commits listed (`4465940`/`3be9b77`/`c3509af`/`eae66c6`); Phase 0 R3 Fix B fold cited at commit `8193e22`. |

SPEC: clean. 0 findings.

---

## §2. Plan patch verification (against R0 v1 recommendations)

Plan-Phase-2 sections, file inventory, and Phase 3a narrative header all reflect Fix B. The single residual drift found is captured as I-1 in §3 (post-verify resolved).

---

## §3. SPEC ↔ plan drift check

### I-1 (Important, conf 90, RESOLVED post-verify): Plan had pre-Fix-B `MlockedZeroizing` language in three Phase 3a/3b prescriptive locations

The plan's Phase 2 section, file inventory, and Phase 3a narrative header were clean. But the following prescriptive sub-sections still used pre-Fix-B vocabulary at the time of this verification pass:

1. **Plan line 382 (P3a.T1 R0 design-pass deliverable):**
   > "4. Site 2/3/4: confirm no other callers depend on the exact `Zeroizing<Vec<u8>>` type signature (a swap to `MlockedZeroizing<Vec<u8>>` may cascade through derive.rs/synthesize.rs/bip85.rs). Enumerate cascade points."

   Under Fix B there is no type swap. **Resolution:** replaced with sibling-field construction-site enumeration narrative including Vec reallocation immunity audit per R0 v1 Open question 1.

2. **Plan lines 444-446 (P3a.T3 GREEN commit-message template):**
   ```
   Site 2: ResolvedSlot.entropy -> MlockedZeroizing<Vec<u8>>.
   Site 3: DerivedAccount.entropy -> MlockedZeroizing<Vec<u8>>.
   Site 4: bip85 entropy wrapped in MlockedZeroizing at callsites.
   ```
   **Resolution:** replaced with sibling-field / function-local-pin language and commit-prefix updated to `feat(mlock): Cycle B P3a — apply slice-fn mlock at toolkit sites 1-4`.

3. **Plan lines 498, 535, 541, 554, 601 (Phase 3b — "minus `MlockedZeroizing<T>`" framing):**
   **Resolution:** scope bullets, GREEN-task narrative, commit-message template, and R1 checklist all retitled to "Full inline copy of toolkit's slice-fn surface (no carve-out under Fix B)". P3b R1 review checklist's "MlockedZeroizing<T> is NOT in ms-cli's mlock.rs" line replaced with "Name-export parity: toolkit and ms-cli `mlock.rs` export identical sets".

**Why Important and not Critical:** The plan's Phase 2 sections (P2.T1-T6) were clean and the file inventory was internally consistent on Fix B. The drift only surfaced in the prescriptive sub-sections downstream of Phase 2, and the commit-message templates were real implementation-time hazards (copy-paste would have misdescribed the changes).

**Note on historical references that should NOT be touched:** Plan lines 43, 47, 102 reference `MlockedZeroizing` inside Phase 1 (P1.T1 + commit-message template). Phase 1 SHIPPED at `eae66c6` with those commits already in git history; rewriting the historical commit-message template is not appropriate. Similarly, lines 170, 179, 267, 291, 343, 743 (the explicit "Removed under Fix B" enumerations + the C-1 narrative) correctly mention `MlockedZeroizing` because they're describing the retired design or contrasting against it. Those are not drift.

---

## §4. Phase 1 consistency check

- SPEC frontmatter line 4: "Phase 1 shipped (commits `4465940`/`3be9b77`/`c3509af`/`eae66c6`)". Matches the four-commit chain in the R0 v1 report.
- Plan line 5: "Phase 1 SHIPPED 2026-05-13 at commits ... → `eae66c6` (R1 CLEAR). Push at `eae66c6` reached origin/master." Consistent with SPEC.
- Phase 1's 7-callee count (DICE app) is correctly absorbed into SPEC §2 row 4 ("7 callees in `format_*` functions updated"). Plan still has "Update 6 callees" inside the P1.T3 task block, but P1 has SHIPPED and the actual implementation landed all 7 per Phase 1 R0/R1. Same historical-narrative-not-drift consideration as §3 above.

No findings.

---

## §5. Fix B soundness check

### Sites 2/3 struct field drop order (forward per RFC 1857)

Rust drops struct fields in declaration order, top-to-bottom (RFC 1857). Both SPEC §4 P3a and the plan's P3a narrative state Sites 2/3 declare `entropy: Zeroizing<Vec<u8>>` BEFORE `_entropy_pin: PinnedPageRange`. Verified drop sequence:

1. `entropy.drop()` — `Zeroizing<Vec<u8>>::drop` calls `Vec::zeroize` (which scrubs the full capacity of the data buffer); then `Vec`'s own Drop deallocates the data buffer. **Bytes are zeroed BEFORE the underlying pages are unpinned.**
2. `_entropy_pin.drop()` — calls `libc::munlock(start, page_count * page_size())` on the page range. The bytes-already-zeroed-by-step-1 are now unpinned.

This is the "zeroize-while-still-pinned" ordering, the strictest threat-model ordering. SPEC §6 G4.a correctly cites it.

**Soundness note:** `Vec::zeroize` (per zeroize crate's docs) zeros the full capacity, but Vec's Drop then calls the global allocator's dealloc. The dealloc'd pages remain mlocked (and still referenced by `_entropy_pin.start`) until step 2 munlocks. Calling munlock on a region whose pages have been dealloc'd is technically valid (POSIX munlock takes an address range, doesn't care about allocation status), and Linux + macOS both accept it. The kernel may have already reused those pages by step 2's munlock — but munlock of a not-currently-locked page returns success (or quietly does nothing). No UB risk. Verified.

### Site 4 function-local drop order (reverse per Rust Reference)

Rust local bindings drop in reverse declaration order (Rust Reference §"destructors"). Both SPEC §4 P3a and the plan's P3a narrative honestly document this: `let entropy = derive_entropy(...)?; let _pin = pin_pages_for(&entropy[..]);` produces drop order `_pin` (munlock) first, then `entropy` (zeroize). The post-munlock-pre-zeroize window is microseconds; the bytes were mlocked during use; zeroize scrubs immediately on return. Acceptable for the threat model.

Honest trade-off documented. Verified.

### `pin_pages_for(&entropy[..])` operates on the Vec's data buffer

`&entropy[..]` (where `entropy: Zeroizing<Vec<u8>>`) deref-chains: `Zeroizing` derefs to `Vec<u8>`, slicing yields `&[u8]` pointing at `Vec::as_ptr()` (the data buffer's heap address). `pin_pages_for(buf: &[u8])` takes `buf.as_ptr()` and `buf.len()`, computes the page-rounded address range, and mlocks those pages. The mlocked pages contain the Vec data buffer (the actual secret bytes), not the Vec header. Fix B correctly pins the secret. Verified.

### Vec reallocation immunity

R0 v1 Open question 1 raises Vec reallocation as a concern. Both SPEC and plan implicitly assume the entropy Vec is constructed-and-frozen (e.g., `Zeroizing::new(vec![0u8; 64])` followed by `copy_from_slice` — never `.push()`/`.extend()`/`.reserve()`). This discipline is the existing bip85 pattern. Phase 3a R0 reviewer should audit each apply site for reallocation immunity per R0 v1's recommendation. SPEC and plan (post-I-1-fold) reference this discipline in P3a.T1 deliverable item 4. Acceptable.

No soundness findings.

---

## §6. Open R0 issues fold check

| R0 issue | SPEC fold | Plan fold | Status |
|---|---|---|---|
| I-R0-1: no Rust CI workflow today; create rust.yml from scratch | §4 P2 line 92 explicitly says "toolkit has no Rust CI today — `manual.yml` + `quickstart.yml` are docs-build only" and lists three jobs. | P2.T4 explicitly says "toolkit has NO Rust CI workflow today (only `manual.yml` + `quickstart.yml` for docs)" and creates `.github/workflows/rust.yml` from scratch with test/miri/clippy. | LANDED. |
| I-R0-2: macOS 16 KiB page; use page_size() not 4096 | §4 P2 line 91 ("Linux x86_64 = 4096; macOS aarch64 = 16384. All page-rounding tests express sizes as `n * page_size()`, never hard-coded."). §6 G1.4 references `page_size()`. | P2.T2 uses `page_size()` for the page-aligned test. P2.T3 sources from `libc::sysconf`. cfg(miri) shim stubs 4096 for Miri only. | LANDED. |
| I-R0-3: lint_safety_first_party_mlock peer lint | §4 P2 line 95 names the new peer lint with ±5-line SAFETY-comment discipline. §2 row 7(d) cites the discipline. | P2.T2 creates `tests/lint_safety_first_party_mlock.rs` (~50 LOC) in the RED commit. P2.T5 audits enforcement. File inventory lists the file. | LANDED. |
| I-R0-4: cfg(test) drop-probe unreachable; under Fix B the probe is moot | §6 G4.a line 197 "No new cfg(test) drop-probe wrapper API is needed (Fix B eliminates `MlockedZeroizing<T>`)". | P2.T1 "moot under Fix B (no wrapper means no drop-probe at all; Cycle A's existing zeroize-on-drop discipline covers G4)". P2.T2 G4 test reframed as `g4_a_zeroize_on_drop_via_zeroizing_vec`. | LANDED. |

All four Important findings folded into both SPEC and plan.

---

## §7. Verdict + next step

**Initial verdict: FOLD** (1 Important; I-1 plan drift on Phase 3a/3b prescriptive sub-sections).

**Post-fold verdict: LOCK** — I-1 deltas applied per the specific replacement language in §3. Plan now reflects Fix B consistently across all sections; no contradictions between Phase 3a/3b prescriptive text and the file inventory.

**Next operational step:** Phase 2 P2.T2 (TDD-RED) — create `src/lib.rs`, stub `src/mlock.rs`, RED tests in `tests/mlock_unit.rs`, `tests/lint_safety_first_party_mlock.rs`, add `libc = "0.2"` to Cargo.toml.
