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

_None at Phase 1.0 close. Filed as cuts proceed._

## Resolved items

_None yet._
