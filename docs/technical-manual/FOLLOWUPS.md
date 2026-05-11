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

### `bibliography-bip-author-canonical-verification` — verify every BIP entry's author list against the canonical bitcoin/bips repo

- **Surfaced:** Phase 1.5 reviewer round of commit `ae5bb51` (L-5).
- **Where:** `docs/technical-manual/src/60-back-matter/66-bibliography.md` — every "BIP-NNN. <authors>." line.
- **What:** Each BIP bibliography entry currently names authors based on best-recollection. Reviewer flagged BIP-93's author list as incomplete (omitting a coauthor). A defensible v1.0 bibliography needs each BIP author list cross-checked against the canonical BIP header in [github.com/bitcoin/bips](https://github.com/bitcoin/bips). Phase 1.5's inline fold dropped the BIP-93 author attribution rather than fabricating a list; the same verification should cover all 11 BIP entries before v1.0.
- **Why deferred:** Local working tree has no vendored BIP-93 / BIP-39 / etc. mediawiki sources; canonical verification requires online access to the bitcoin/bips repo and isn't a Phase 1.5 blocker. Defer to Phase 5.5 bibliography completion.
- **Status:** `open`.
- **Tier:** `tech-manual-v1.0-nice-to-have`.

### `troubleshooting-mk-codec-variant-coverage-audit` — audit which mk-codec Error variants belong in the wire-format-layer troubleshooting subset

- **Surfaced:** Phase 1.5 reviewer round of commit `ae5bb51` (I-1).
- **Where:** `docs/technical-manual/src/60-back-matter/65-troubleshooting.md` mk1 section.
- **What:** Phase 1.5 covers a curated 17-of-22 subset of mk-codec's Error enum (`InvalidHrp`, `InvalidChar`, `UnexpectedEnd`, `TrailingBytes`, `CardPayloadTooLarge` are omitted; the first two are wire-format-layer adjacent and may belong in the subset). The intro now says "curated subset" rather than claiming completeness, but a Phase 4 / Phase 5 audit should re-evaluate the inclusion set against Part V (Rust API reference) coverage to ensure the troubleshooting appendix is complete-for-its-scope by tag-time at v1.0.
- **Why deferred:** Phase 5 back-matter completion is the natural place for the audit; v0.1 sets the scaffold.
- **Status:** `open`.
- **Tier:** `tech-manual-v0.4`.

## Resolved items

_None yet._
