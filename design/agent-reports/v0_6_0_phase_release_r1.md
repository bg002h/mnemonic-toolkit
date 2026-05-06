# v0.6.0 Phase Release — code-architect r1

**Outcome:** 0C/0I/0L/1N — APPROVED.

## Scope reviewed
4 staged files for the v0.6.0 release-prep commit:
- `crates/mnemonic-toolkit/Cargo.toml` — version bump 0.5.2 → 0.6.0.
- `Cargo.lock` — workspace package bumped to 0.6.0.
- `CHANGELOG.md` — new `[0.6.0] — 2026-05-06` entry.
- `design/FOLLOWUPS.md` — 3 new entries (`secret-on-stdout-warning-bundle-retrofit`, `convert-seed-and-raw-privkey-nodes`, `convert-phrase-to-leaf-wif`).

## Plan-fidelity verification
1. **Version-bump alignment:** all four (Cargo.toml, Cargo.lock, CHANGELOG, FOLLOWUPS) consistent at 0.6.0.
2. **CHANGELOG accuracy:** 9-node graph, 3 refusal classes, SPEC §6/§6.a/§6.b features, 230 lib + 67 integration test count, 23 new convert tests across 4 files — all confirmed against working tree.
3. **CHANGELOG structure:** mirrors v0.5.x precedents.
4. **FOLLOWUPS entries:** three new entries correctly formatted; tiers reasonable.
5. **Wire-format-unchanged claim:** sound — convert is a new command path; no existing encode/decode paths modified.
6. **Semver:** v0.6.0 correct (subcommand addition under pre-1.0 = minor bump).
7. **Architect-review report references:** all cited files exist on disk.

## Nit (deferred)

CHANGELOG.md:27 "11 edges + mk1→xpub decode" phrasing slightly ambiguous (mk1→xpub IS one of the 11). Functionally harmless; reader-facing reconciliation is trivial against the test file.

**Verdict:** Ready for tag + push + GitHub release.
