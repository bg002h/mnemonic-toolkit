# Follow-up tracker — Technical manual

Mirrors the format of `mnemonic-toolkit/design/FOLLOWUPS.md` and `docs/manual/FOLLOWUPS.md`. Single source of truth for items that surfaced during a technical-manual review or implementation pass but were not fixed in the same commit.

## How to use this file

**Format for each entry:**

```markdown
### `<short-id>` — <one-line title>

- **Surfaced:** Phase X.Y review of commit <SHA>, or "inline TODO at <file>:<line>"
- **Where:** `<file>:<line>` or "design — SPEC §X"
- **What:** 1–3 sentences describing the gap or improvement opportunity
- **Why deferred:** the reason it didn't ship in the original commit
- **Status:** `open` | `resolved <COMMIT>` | `wont-fix — <one-line reason>`
- **Tier:** `tech-manual-v0.1-blocker` | `tech-manual-v0.1-nice-to-have` | `tech-manual-v0.2` | ... | `cross-repo` | `v1+` | `external`
```

Reference `<short-id>` in commit messages when closing: `closes FOLLOWUPS.md <short-id>`.

## Tiers (definitions)

- **`tech-manual-vX.Y-blocker`** — must fix before tagging the corresponding cut. Failing to fix blocks the release.
- **`tech-manual-vX.Y-nice-to-have`** — should fix before that cut if time permits; non-blocking.
- **`tech-manual-vX.Y`** — explicitly deferred to that cut by a phase decision or spec note.
- **`cross-repo`** — depends on coordination with sibling repos (`descriptor-mnemonic`, `mnemonic-key`, `mnemonic-secret`). Mirrored by a companion entry in the affected sibling's tracker; both cite each other.
- **`v1+`** — deferred indefinitely. May be revisited only at a future major version revision.
- **`external`** — depends on work outside the constellation (e.g., upstream BIP-DKG standardization for the FROST chapter).

---

## Open items

### `cross-repo md1-wsh-multi-unsorted-integration-test` — add paired-derivation test for `wsh(multi(...))` in md1

- **Surfaced:** Phase 2.2 reviewer round of commit `7f05e50` (I-2).
- **Where:** `descriptor-mnemonic/crates/md-codec/tests/address_derivation.rs` (new test).
- **What:** The integration test suite covers `wsh(sortedmulti(...))` (`address_derivation.rs:252-331`) but lacks a paired-derivation test for the unsorted `wsh(multi(...))` variant. The `Terminal::Multi` arm at `to_miniscript.rs:365-373` handles it via the fall-through `node_to_miniscript::<Segwitv0>` path, but the path is untested against an independent `miniscript::Descriptor::from_str(...)` derivation. Add a `wsh_multi_2_of_3_address` test mirroring the sortedmulti shape with `Tag::Multi` substituted; assert byte-identical agreement. Once landed, §III.2 Bucket 7's prose can cite the new test directly instead of noting the absence.
- **Why deferred:** out-of-scope for a technical-manual cut; the gap surfaced during prose review, not via a wire-format bug.
- **Status:** `open`.
- **Tier:** `cross-repo` (lands in `descriptor-mnemonic`; mirror entry to be filed in that repo's `design/FOLLOWUPS.md` when md1 work begins).

## Resolved items

### `bibliography-bip-author-canonical-verification` — verify every BIP entry's author list against the canonical bitcoin/bips repo

- **Surfaced:** Phase 1.5 reviewer round of commit `ae5bb51` (L-5).
- **Where:** `docs/technical-manual/src/60-back-matter/66-bibliography.md` — every "BIP-NNN. <authors>." line.
- **What:** Each BIP bibliography entry was named with authors based on best-recollection. Reviewer flagged BIP-93's author list as incomplete; Phase 1.5's fold dropped the BIP-93 attribution rather than fabricating. **Resolution:** fetched each cited BIP's canonical mediawiki from `raw.githubusercontent.com/bitcoin/bips/master/` and reconciled the bibliography's author lists against the canonical headers. Updates landed for BIP-93 (added Leon Olsson Curr / Pearlwort Sneed pseudonyms + Andrew Poelstra), BIP-379 (added Antoine Poinsot, Ava Chow), BIP-380 (Andrew Chow → Ava Chow), BIP-389 (Andrew Chow → Ava Chow). All other entries (BIP-32, BIP-39, BIP-173, BIP-340, BIP-341, BIP-342, BIP-388) were verified to match the canonical headers exactly; no changes required.
- **Status:** `resolved` (2026-05-11, this commit).
- **Tier:** `tech-manual-v1.0-nice-to-have` (closed ahead of schedule during the v0.1→v0.2 transition).

### `troubleshooting-mk-codec-variant-coverage-audit` — audit which mk-codec Error variants belong in the wire-format-layer troubleshooting subset

- **Surfaced:** Phase 1.5 reviewer round of commit `ae5bb51` (I-1).
- **Where:** `docs/technical-manual/src/60-back-matter/65-troubleshooting.md` mk1 section.
- **What:** Phase 1.5 covered a curated 17-of-22 subset of mk-codec's Error enum; the 5 omitted variants were `InvalidHrp`, `InvalidChar`, `UnexpectedEnd`, `TrailingBytes`, `CardPayloadTooLarge`. **Resolution:** all 5 are reachable wire-format-layer surface and warrant inclusion. Added rows for each to the mk1 troubleshooting table with per-variant cause + remediation pointer; the mk1 section now covers 22/22 mk-codec Error variants (full coverage).
- **Status:** `resolved` (2026-05-11, this commit).
- **Tier:** `tech-manual-v0.4` (closed ahead of schedule during the v0.1→v0.2 transition).
