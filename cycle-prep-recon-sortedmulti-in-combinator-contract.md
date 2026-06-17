# cycle-prep recon — 2026-06-12 — sortedmulti-in-combinator contract (GAP 3)

**Origin/master SHA at recon time:** `ca7d7bc` (toolkit; HEAD == origin/master, 0 ahead / 0 behind)
**Companion SHA:** descriptor-mnemonic `origin/main` = `422b049`
**Local branch:** `master`
**Sync state:** up-to-date
**Untracked:** prior cycle-prep recon docs, cycle-b-* scripts, CONTINUITY.md, .claude/ (none relevant)

Slug(s) verified: `bundle-accepts-sortedmulti-in-combinator-restore-cannot`. Expectation: low drift (filed 2026-06-11, one merge ago); the open question was severity class (silent-engrave vs loud-refuse) — **empirically resolved below: silent-ACCEPT at all 3 creation surfaces, loud-and-safe REFUSE at restore.**

---

## Per-slug verification

### `bundle-accepts-sortedmulti-in-combinator-restore-cannot`

- **WHAT (from FOLLOWUPS.md):** `build-descriptor` / `bundle --descriptor` / `export-wallet --descriptor` all ACCEPT a descriptor with `sortedmulti` nested inside a combinator and emit an md1 card, but `restore --md1` REFUSES to reconstruct it — a user can engrave a card the toolkit cannot mechanically restore. Fix design call: (a) creation-time reject vs (b) md-codec wire extension. Tier: deferred.

- **Citations:**
  - `design/FOLLOWUPS.md:53` entry heading — **ACCURATE** (`git show origin/master:design/FOLLOWUPS.md | grep -n` → line 53, same on working tree).
  - Example descriptor + checksum `#qy7ka0ay` — **ACCURATE, empirically reproduced.** `build-descriptor --spec -` with `{"or_d":[{"sortedmulti":...},{"and_v":[v:pk, older(144)]}]}` emits exactly `wsh(or_d(sortedmulti(2,…),and_v(v:pk(…),older(144))))#qy7ka0ay`, exit 0.
  - Refusal message "`Tag::SortedMulti` must be the sole child of wsh/sh; cannot appear as a miniscript leaf" — **ACCURATE.** Source: descriptor-mnemonic `crates/md-codec/src/to_miniscript.rs:417-421` (the `(Tag::SortedMulti, Body::MultiKeys { .. })` leaf arm returns `Err(failed(...))`). Surfaced by toolkit `crates/mnemonic-toolkit/src/cmd/restore.rs:914-927` (`faithful_multisig_descriptor` → `to_miniscript_descriptor` error map).
  - "`sortedmulti` is a BIP-380 DESCRIPTOR-level wrapper … md1 wire only represents it as the sole wsh/sh child" — **PARTIALLY STRUCTURALLY-WRONG (important nuance).** The BIP-380 claim is accurate, but the **md1 WIRE is NOT the limitation**: `tree.rs:244` encodes `Body::MultiKeys` for `Tag::SortedMulti` generically at ANY tree position, and md-codec's own P7 oracle (`crates/md-codec/tests/proptest_to_miniscript.rs:101-117` + `self_test_bad_sortedmulti_under_combinator` :380-393) asserts the nested shape **wire-round-trips EXACTLY** (`encode_payload`→`decode_payload` == identity). The refusal lives solely in the RECONSTRUCTION layer: md-codec's `to_miniscript.rs` special-cases sole-child `SortedMulti` at :198/:222/:241 and refuses it as a miniscript leaf at :417 because md-codec pins **crates.io miniscript 13.0.0** (descriptor-mnemonic `Cargo.toml:18`), which has no `Terminal::SortedMulti`. The TOOLKIT pins the git fork rev `95fdd1c` (toolkit `Cargo.toml:17`), which post-#915 HAS `Terminal::SortedMulti` as a parseable nested fragment (`parse_descriptor.rs:577-580` walks it into `Tag::SortedMulti` at any position) — the same two-miniscripts split documented in the v0.49.1 cycle. So the FOLLOWUP's option (b) "extend md-codec to encode sortedmulti as a non-sole-child" is mis-framed: the ENCODING already works; what's missing is the RENDERER.
  - "found on the FIRST proptest run" / harness exclusion — **ACCURATE.** `crates/mnemonic-toolkit/tests/prop_backup_restore_roundtrip.rs:91-96` is the `allow_sorted` doc comment citing this FOLLOWUP by name; `allow_sorted=true` is passed ONLY at schema 0 (top-level, :153); all 8 nested `multi(...)` call sites pass `false` (:159, :167, :175-176, :185, :193, :202, :224-225, :235). The property suite permanently EXCLUDES the shape rather than pinning the refusal.
  - "NOT silent funds-loss (restore refuses LOUDLY, and the engraved md1 is a faithful backup)" — **ACCURATE, empirically confirmed** (see severity determination below).

- **Action for brainstorm spec:** keep the entry's severity framing; CORRECT the root-cause sentence ("md1 wire only represents it as the sole wsh/sh child" → "the md1 wire round-trips it exactly at any position; md-codec's `to_miniscript` renderer refuses non-sole-child `Tag::SortedMulti` because its pinned crates.io miniscript 13.0.0 lacks `Terminal::SortedMulti`, which the toolkit's pinned git rev `95fdd1c` HAS"). Cite toolkit `ca7d7bc` + descriptor-mnemonic `422b049`.

---

## Severity determination — silent-ENGRAVE, loud-SAFE-restore (empirical, binary at `ca7d7bc`, v0.55.0)

**Creation side (the gap): all three surfaces accept with NO unrestorability warning.**

1. `bundle --network mainnet --descriptor 'wsh(or_d(sortedmulti(2,A,B),and_v(v:pk(C),older(144))))' --json` → **exit 0**, emits a 6-card `md1` array; the ONLY stderr output is the generic `note: stdout is watch-only — public keys only, cannot spend`. Human (non-JSON) output likewise carries no restorability note (grep for warn/restor/advis over the full card output: only the watch-only note).
2. `build-descriptor --spec -` → exit 0, descriptor on stdout, no warning.
3. `export-wallet --descriptor … --format descriptor` → exit 0, no warning. (Note: export-wallet emits no md1 card, so its acceptance is correct behavior, not part of the engrave gap.)

**Restore side: clean, loud, funds-safe refusal.** Feeding the 6 emitted cards back:

```
$ mnemonic restore --network mainnet --md1 <c1> … --md1 <c6>
error: --md1 → descriptor: address derivation failed: Tag::SortedMulti must be the
sole child of wsh/sh; cannot appear as a miniscript leaf. The engraved card remains
a faithful backup.
exit=1, stdout empty
```

No wrong reconstruction, no exit-0, nothing on stdout. Refusal site: `restore.rs:911-927` `faithful_multisig_descriptor` — note its conditional slug-attribution `hint` (:918-923) fires only for `"cannot wrap"` (the resolved PART-2 shape); the sortedmulti refusal names NO follow-up slug in its error text.

**Verdict: loud-refuse class (robustness gap), NOT the dangerous silent-engrave-silent-restore class.** The engraved card is a faithful backup in the strong sense: md-codec's P7 proves the wire round-trip is byte-exact, so a FUTURE toolkit (or a human reading the BIP draft) can recover the policy. The user-facing harm is operational: a user discovers at restore time — possibly years later, possibly under duress — that mechanical restore refuses, with no signal at engrave time and no slug pointer in the error.

## Test-absence confirmed (toolkit level) — but md-codec ALREADY pins its half

- Toolkit `tests/`: NO test exercises sortedmulti-in-a-combinator on any path. The only "sole child" mention is the harness comment (`prop_backup_restore_roundtrip.rs:92`). The existing negative property `negative_property_unreconstructable_shapes_refuse_loudly` (:551-588) covers ONLY the per-key-use-site-override and hardened-wildcard shapes (the `restore-md1-per-key-use-site-and-hardened-wildcard` slug) — sortedmulti-in-combinator is absent. All other `sortedmulti` test hits are top-level/template/import-side.
- md-codec (`422b049`) ALREADY pins the codec half: `proptest_to_miniscript.rs::self_test_bad_sortedmulti_under_combinator` (:380-393, from STRESS-B) asserts `to_miniscript_descriptor` cleanly Errs AND the wire round-trip stays exact for precisely this shape. **What is NOT pinned anywhere is the end-to-end CLI contract:** bundle-accepts → restore-refuses-loudly-with-exit-nonzero-and-empty-stdout.

---

## Assessment

- **Severity:** moderate-low. Loud-refuse, funds-safe, wire-faithful — robustness/UX, not funds-safety. The dangerous variants of this family (silent collapse C1/C2, use-site silent mis-render) were all closed in v0.54.x; this is the residual "bundle warns nothing about unrestorable shapes" family member.
- **Related slug `bundle-engraves-unrestorable-pk-keyed-cards` (FOLLOWUPS.md:85) is STALE as filed.** It is gated on `to-miniscript-check-pkh-double-wrap`, which is ✓ RESOLVED (md-codec 0.35.1 + v0.54.1); empirically confirmed at `ca7d7bc`: `bundle` → `restore` of `wsh(and_v(v:pk(A),older(144)))` now round-trips exit-0 with the correct descriptor. Its "bundle-time advisory" idea did NOT dissolve, though — the advisory family now correctly attaches to the REMAINING unrestorable shapes: sortedmulti-in-combinator (this slug) + per-key use-site overrides + hardened wildcard (`restore-md1-per-key-use-site-and-hardened-wildcard`). Recommend: re-scope :85 into a single `bundle-unrestorable-shape-advisory` umbrella (or resolve it and let this slug carry the advisory), rather than two parallel advisory slugs.
- **Pin-the-contract (cheap-do-now), ~30-60 LOC, NO-BUMP:**
  1. Add the sortedmulti-in-combinator shape to `negative_property_unreconstructable_shapes_refuse_loudly` (or a sibling cell). Unlike the two existing shapes it needs no slots — concrete origin-annotated keys via `bundle --descriptor` suffice (demonstrated above). Assert bundle `.success()` → restore `.failure()` + stderr contains `sole child` + `faithful backup`. Test-only ⇒ NO-BUMP.
  2. (Optional ride-along, error-text only) extend the `restore.rs:918-923` hint so the `"sole child"` error names this slug, mirroring the `"cannot wrap"` precedent — symmetric diagnostics, trivially small.
- **Bundle-time advisory (the real UX fix), ~80-150 LOC, PATCH:** a non-blocking stderr advisory at bundle emit when the walked tree contains `Tag::SortedMulti` NOT as the sole wsh/sh child ("this policy engraves faithfully but `restore --md1` cannot yet mechanically reconstruct it; tracked as `bundle-accepts-sortedmulti-in-combinator-restore-cannot`"). No clap flag change ⇒ no `schema_mirror` lockstep, no manual 40-cli-reference change (advisories are not flags); a short manual note is optional polish. Behavior change on stderr ⇒ PATCH. Best done as the UMBRELLA advisory covering all three remaining unrestorable shapes (one detection walk, one advisory mechanism), resolving the re-scoped :85 in the same cycle.
- **Creation-time REJECT (FOLLOWUP option (a)) — recommend AGAINST.** It is a breaking behavior change (previously-accepted descriptors refused ⇒ MINOR pre-1.0), it forbids cards that are provably wire-faithful and likely to BECOME restorable (see next), and rust-miniscript itself admits the nested form post-#915 — the "standard position is sole-child anyway" rationale in the FOLLOWUP is stale against the toolkit's own pinned parser.
- **Faithful fix (reconstruct nested sortedmulti) — feasible, cross-repo, moderate.** The wire already carries everything (P7-proven). Two routes:
  - (i) md-codec bumps its miniscript dep to a release containing #915's `Terminal::SortedMulti` and deletes the :417 refusal arm — cleanest, but blocked on an upstream crates.io release (13.0.0 is current; #915 is git-only at `95fdd1c`).
  - (ii) the v0.49.1 route-around precedent: render the tree to a descriptor STRING without entering rust-miniscript types (md-cli's `format/text.rs` already has a textual renderer; md-codec could expose a `to_descriptor_string`), then the toolkit parses the string with ITS pinned miniscript (which accepts nested sortedmulti) and derives addresses via the existing `derive_receive_addresses`-on-string path (`restore.rs:1070-1149` precedent). md-codec MINOR + exact-pin lockstep + toolkit pin bump + a restore fallback arm; ~1-2 day cycle, R0-gated, crates.io publish required.
- **Repo split:** contract pin + advisory = toolkit-only. Faithful fix = descriptor-mnemonic (companion FOLLOWUP entry required per cross-repo convention) + toolkit.

---

## Recommended scope

**Verdict: loud-refuse / robustness — confirm tier `deferred` for the faithful fix, but promote a cheap contract-pinning cycle now.**

1. **Cycle now (NO-BUMP / PATCH, toolkit-only, small):** pin the end-to-end refusal contract in `negative_property_unreconstructable_shapes_refuse_loudly` (+ the slug-naming hint in `restore.rs`), and add the non-blocking bundle-time unrestorable-shape advisory as the umbrella for {sortedmulti-in-combinator, use-site overrides, hardened wildcard} — re-scoping/resolving the stale `bundle-engraves-unrestorable-pk-keyed-cards` (:85) in the same stroke. Test-only half is NO-BUMP; the advisory makes the cycle PATCH. No schema_mirror, no manual-flag lockstep.
2. **Deferred (cross-repo, MINOR-ish):** faithful nested-sortedmulti reconstruction via route (ii) (md-codec textual renderer + toolkit string-parse fallback), or wait for an upstream miniscript release with #915 and take route (i). File/update the descriptor-mnemonic companion either way.
3. **Brainstorm-spec correction (mandatory):** fix the FOLLOWUP's root-cause sentence — the md1 wire is NOT the limitation; the renderer/miniscript-pin split is. Cite `ca7d7bc` / `422b049`.

R0 gate applies before any implementation (CLAUDE.md Conventions).
