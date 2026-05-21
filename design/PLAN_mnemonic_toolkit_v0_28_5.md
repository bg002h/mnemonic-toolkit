# mnemonic-toolkit-v0.28.5 Implementation Plan (Cycle 1 / Wave 1 first)

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close 2 doc-only FOLLOWUPs: `plan-smoke-step4-ms1-on-bundle-not-supported` (replace nonexistent `--ms1` flag with `--slot @0.phrase=` in a plan-doc smoke recipe) + `import-wallet-envelope-schema-version-narrative-drift` (disambiguate the dual `schema_version` constants in `cmd/import_wallet.rs`). Tag `mnemonic-toolkit-v0.28.5`.

**Architecture:** Pure documentation/source-comment edits. No toolkit functional change. No test cell changes. No GUI lockstep. Two small edits across 2 files. The plan-doc edit replaces 1 flag-shape in a smoke recipe; the schema_version disambiguation adds cross-reference comments at both constant sites (the source-of-truth is `cmd/import_wallet.rs:87` "1" + `:975` "4" per recon).

**Tech Stack:** Markdown (plan-doc) + Rust doc-comments; `cargo build` to confirm no break; `actionlint` not needed (no workflow changes).

**Brainstorm spec:** `design/BRAINSTORM_v0_28_plus_residual_followups.md` § "Cycle 1 — `mnemonic-toolkit-v0.28.5` (docs)".

**Source SHA at plan-write time:** `f9fbe6a`.

---

## File structure

- **Modify:** `design/PLAN_v0_27_0_bsms_round_trip_and_wallet_import_handoff.md`
  - §6.3 step 4 (around L793): replace `--ms1 <value>` (nonexistent on `bundle` subcommand) with `--slot @0.phrase=<phrase>` (the canonical seed-overlay syntax per `mnemonic bundle --help`).
- **Modify:** `crates/mnemonic-toolkit/src/cmd/import_wallet.rs`
  - L87 (`IMPORT_WALLET_ENVELOPE_SCHEMA_VERSION = "1"`): add a doc-comment cross-referencing the inner BundleJson schema_version at L975.
  - L975 (inner BundleJson literal `schema_version: "4"`): add a doc-comment cross-referencing the envelope schema_version at L87.
- **Modify:** `crates/mnemonic-toolkit/Cargo.toml` — `version = "0.28.4"` → `"0.28.5"`.
- **Modify:** `CHANGELOG.md` — new `## mnemonic-toolkit [0.28.5] — <YYYY-MM-DD>` section above `[0.28.4]`.
- **Modify:** `scripts/install.sh:32` — `mnemonic-toolkit-v0.28.4` → `mnemonic-toolkit-v0.28.5` (install-pin-check.yml CI gate).
- **Modify:** `design/FOLLOWUPS.md` — 2 Status flips.

That's the entire file scope. No new files.

---

## Tasks

### Task 1: Recon — confirm citations

**Files:** none modified (read-only).

- [ ] **Step 1: Verify plan-doc citation at L793**

Run:
```bash
sed -n '790,800p' /scratch/code/shibboleth/mnemonic-toolkit/design/PLAN_v0_27_0_bsms_round_trip_and_wallet_import_handoff.md
```

Expected: line ~793 contains a smoke recipe step 4 that uses `--ms1 <something>` on the `bundle` subcommand. Note the exact line number (may have shifted since recon at SHA `0ca86b5`; use the current actual line).

- [ ] **Step 2: Verify `mnemonic bundle --help` has no `--ms1` flag**

Run:
```bash
/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic bundle --help | grep -E '^\s*--ms1' || echo "(confirmed: no --ms1 on bundle subcommand)"
```

Expected: empty output + the "confirmed" message. This confirms the FOLLOWUP's correctness.

- [ ] **Step 3: Verify the dual schema_version constants in cmd/import_wallet.rs**

Run:
```bash
grep -n 'IMPORT_WALLET_ENVELOPE_SCHEMA_VERSION\|schema_version:\s*"' /scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/import_wallet.rs
```

Expected:
- `:87` — `IMPORT_WALLET_ENVELOPE_SCHEMA_VERSION: &str = "1";`
- `:975` (or nearby; verify current line) — `schema_version: "4",` (inner BundleJson literal)

Note exact line numbers post-grep. Plan-doc cites :87 + :975 from SHA `0ca86b5` recon.

---

### Task 2: Apply plan-doc edit (plan-smoke fix)

**Files:**
- Modify: `design/PLAN_v0_27_0_bsms_round_trip_and_wallet_import_handoff.md`

- [ ] **Step 1: Read the smoke recipe context**

Run:
```bash
sed -n '785,805p' /scratch/code/shibboleth/mnemonic-toolkit/design/PLAN_v0_27_0_bsms_round_trip_and_wallet_import_handoff.md
```

Identify step 4. The step uses `--ms1 <value>` on `mnemonic bundle` — which is wrong. The correct flag for supplying a seed phrase at slot 0 to the `bundle` subcommand is `--slot @0.phrase=<phrase>`.

- [ ] **Step 2: Apply the edit via the Edit tool**

Old (representative — verify exact text at Task 1 Step 1):
```
mnemonic bundle --network mainnet --template bip84 --ms1 <ms1-string>
```

New:
```
mnemonic bundle --network mainnet --template bip84 --slot @0.phrase="abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
```

(The replacement uses the BIP-39 abandon-test-vector phrase for the BIP-84 wallet; it produces the canonical bundle whose ms1 is `ms10entrsqqqq...cj9sxraq34v7f` per `docs/manual/transcripts/22-first-bundle.out`. The exact phrase value can be any valid BIP-39 phrase; abandon-test-vector matches the cycle's existing convention.)

If the smoke recipe is referencing a specific ms1 string instead of any-phrase, the implementer should replace with a phrase that yields THAT ms1 — but for the abandon-vector ms1, the phrase is the canonical abandon-test-vector.

- [ ] **Step 3: Verify the edit applied**

Run:
```bash
grep -n -- '--ms1' /scratch/code/shibboleth/mnemonic-toolkit/design/PLAN_v0_27_0_bsms_round_trip_and_wallet_import_handoff.md
```

Expected: zero hits in step 4. (Other `--ms1` references elsewhere in the plan-doc may exist — those refer to `verify-bundle --ms1` or `convert --from ms1=` which ARE valid flags; verify the only `--ms1` removed was the one on the `bundle` subcommand context.)

---

### Task 3: Apply schema_version cross-reference comments (cmd/import_wallet.rs)

**Files:**
- Modify: `crates/mnemonic-toolkit/src/cmd/import_wallet.rs`

- [ ] **Step 1: Read current context at L83-92 (envelope constant + nearby)**

Run:
```bash
sed -n '83,92p' /scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/import_wallet.rs
```

Expected: `pub(crate) const IMPORT_WALLET_ENVELOPE_SCHEMA_VERSION: &str = "1";` at L87 with whatever doc-comment is currently above it.

- [ ] **Step 2: Add cross-reference doc-comment at L87**

Replace the existing doc-comment (or insert one if absent) immediately above L87 to read:

```rust
/// SPEC v0.28.x — OUTER envelope schema version (current: "1").
///
/// **Disambiguation:** the toolkit carries TWO `schema_version` fields:
/// 1. This OUTER constant at `import_wallet.rs:87` — the `--json`
///    envelope wire-shape version (governs `--from-import-json` array
///    semantics + `import_provenance` field set).
/// 2. The INNER `BundleJson.schema_version` literal at
///    `import_wallet.rs:~975` (current: "4") — governs the bundle
///    payload wire-shape (governs `bundle.mk1`/`bundle.md1`/etc.
///    field set inside each envelope entry).
///
/// Both fields share the name `schema_version` but evolve independently.
/// Future readers / parser authors: when extending the envelope
/// wire-shape, bump THIS constant; when extending the bundle payload
/// wire-shape, bump the inner BundleJson literal. Cross-cite both when
/// either changes. Tracked as FOLLOWUP
/// `import-wallet-envelope-schema-version-narrative-drift` (resolved
/// v0.28.5).
```

- [ ] **Step 3: Read current context at L970-980 (inner BundleJson literal)**

Run:
```bash
sed -n '970,985p' /scratch/code/shibboleth/mnemonic-toolkit/crates/mnemonic-toolkit/src/cmd/import_wallet.rs
```

Expected: `schema_version: "4",` at L975 (or nearby) inside a `BundleJson { ... }` literal.

- [ ] **Step 4: Add cross-reference inline comment at the inner literal**

Replace the `schema_version: "4",` line with:

```rust
            // INNER BundleJson schema_version (current: "4"). Governs
            // the bundle payload wire-shape (mk1/md1/etc fields). See
            // the OUTER envelope schema_version doc-comment at L87 for
            // the disambiguation rule; cross-cite both when either
            // changes. FOLLOWUP `import-wallet-envelope-schema-version-
            // narrative-drift` resolved v0.28.5.
            schema_version: "4",
```

- [ ] **Step 5: Run cargo build**

Run:
```bash
cargo build --package mnemonic-toolkit --bin mnemonic 2>&1 | tail -3
```

Expected: `Finished` line; no errors. (Comments-only edits should not break compilation.)

---

### Task 4: Version bump + CHANGELOG + install.sh

**Files:**
- Modify: `crates/mnemonic-toolkit/Cargo.toml`
- Modify: `CHANGELOG.md`
- Modify: `scripts/install.sh`

- [ ] **Step 1: Bump Cargo.toml**

Edit `crates/mnemonic-toolkit/Cargo.toml`:
- Old: `version = "0.28.4"`
- New: `version = "0.28.5"`

- [ ] **Step 2: Add CHANGELOG.md entry**

Insert IMMEDIATELY above `## mnemonic-toolkit [0.28.4]`:

```markdown
## mnemonic-toolkit [0.28.5] — <YYYY-MM-DD>

Patch release: 2 doc-only fixes closing v0.28+ FOLLOWUPs surfaced in the post-A/B/C recon dossier.

### Documentation

- **`design/PLAN_v0_27_0_bsms_round_trip_and_wallet_import_handoff.md` §6.3 step 4** — Replace nonexistent `--ms1` flag (which doesn't exist on the `bundle` subcommand) with `--slot @0.phrase=` per `mnemonic bundle --help`. Closes FOLLOWUP `plan-smoke-step4-ms1-on-bundle-not-supported`.

- **`cmd/import_wallet.rs:87 + :~975`** — Add cross-reference doc-comments at both `schema_version` constant sites (outer envelope `"1"` + inner BundleJson `"4"`). The two constants share the name but evolve independently; comments now make the disambiguation explicit at-site. Closes FOLLOWUP `import-wallet-envelope-schema-version-narrative-drift`.

### Note

Cycle 1 of the v0.28+ residual FOLLOWUP release plan (see `design/BRAINSTORM_v0_28_plus_residual_followups.md`). Wave 1 first ship. No CLI surface change; no test cell changes; no GUI lockstep.
```

Replace `<YYYY-MM-DD>` with today's date.

- [ ] **Step 3: Bump scripts/install.sh:32**

Edit `scripts/install.sh:32`:
- Old: `echo "mnemonic-toolkit|https://github.com/bg002h/mnemonic-toolkit|mnemonic-toolkit-v0.28.4|no|"`
- New: `echo "mnemonic-toolkit|https://github.com/bg002h/mnemonic-toolkit|mnemonic-toolkit-v0.28.5|no|"`

This bump is REQUIRED by `install-pin-check.yml` CI gate.

- [ ] **Step 4: Rebuild + verify binary version**

Run:
```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
cargo build --bin mnemonic 2>&1 | tail -3
target/debug/mnemonic --version
```

Expected: `mnemonic 0.28.5`.

---

### Task 5: Sonnet reviewer fold-verify

**Files:** none modified.

- [ ] **Step 1: Dispatch sonnet via Agent tool**

Use the `Agent` tool with:
- `subagent_type: feature-dev:code-reviewer`
- `model: sonnet`
- Prompt that asks the reviewer to verify:
  1. Plan-doc edit at `design/PLAN_v0_27_0_bsms_round_trip_and_wallet_import_handoff.md` §6.3 step 4 replaces `--ms1` with `--slot @0.phrase=` correctly.
  2. `cmd/import_wallet.rs:87` doc-comment added; cross-references inner literal at `:~975`.
  3. `cmd/import_wallet.rs:~975` inline comment added; cross-references outer constant at `:87`.
  4. No functional code changes (comments-only / version bumps only).
  5. `Cargo.toml` version + `CHANGELOG.md` entry + `scripts/install.sh:32` all bumped to v0.28.5.
  6. Cargo build clean.

Gate: 0 critical / 0 important to proceed.

- [ ] **Step 2: Fold any Important findings inline**

Loop until 0 Important.

---

### Task 6: Flip FOLLOWUPS Status (2 entries)

**Files:**
- Modify: `design/FOLLOWUPS.md`

- [ ] **Step 1: Locate both FOLLOWUP entries**

Run:
```bash
grep -n '^### `plan-smoke-step4-ms1-on-bundle\|^### `import-wallet-envelope-schema-version-narrative-drift' /scratch/code/shibboleth/mnemonic-toolkit/design/FOLLOWUPS.md
```

Expected: two line numbers.

- [ ] **Step 2: Flip Status for `plan-smoke-step4-ms1-on-bundle-not-supported`**

In the entry's `- **Status:**` line:

Old:
```markdown
- **Status:** `open`
```

New (with `<PLACEHOLDER-COMMIT-SHA>` literal; backfill after Task 7):
```markdown
- **Status:** `resolved <PLACEHOLDER-COMMIT-SHA>` — mnemonic-toolkit-v0.28.5 cycle replaced the nonexistent `--ms1` flag in §6.3 step 4 with `--slot @0.phrase=` per `mnemonic bundle --help`.
```

- [ ] **Step 3: Flip Status for `import-wallet-envelope-schema-version-narrative-drift`**

Old:
```markdown
- **Status:** `open`
```

New:
```markdown
- **Status:** `resolved <PLACEHOLDER-COMMIT-SHA>` — mnemonic-toolkit-v0.28.5 cycle added cross-reference doc-comments at both `schema_version` constant sites in `cmd/import_wallet.rs` (outer envelope L87 + inner BundleJson literal at L~975); future readers / parser authors now have at-site disambiguation between the two fields.
```

(Both Status flips reference `<PLACEHOLDER-COMMIT-SHA>`; Task 7 sed-replaces with the actual commit SHA.)

---

### Task 7: Commit + tag + push

**Files:** all modified files staged.

- [ ] **Step 1: Verify working tree state**

Run:
```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
git status --short
```

Expected modified files:
- `design/PLAN_v0_27_0_bsms_round_trip_and_wallet_import_handoff.md`
- `crates/mnemonic-toolkit/src/cmd/import_wallet.rs`
- `crates/mnemonic-toolkit/Cargo.toml`
- `Cargo.lock` (from cargo build)
- `CHANGELOG.md`
- `scripts/install.sh`
- `design/FOLLOWUPS.md`

No untracked files other than `.claude/` (gitignored).

- [ ] **Step 2: Stage explicit paths**

Run:
```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
git add design/PLAN_v0_27_0_bsms_round_trip_and_wallet_import_handoff.md \
        crates/mnemonic-toolkit/src/cmd/import_wallet.rs \
        crates/mnemonic-toolkit/Cargo.toml \
        Cargo.lock \
        CHANGELOG.md \
        scripts/install.sh \
        design/FOLLOWUPS.md
git diff --cached --stat
```

Expected: 7 files changed; ~30-50 LOC insertions; minimal deletions.

- [ ] **Step 3: Commit**

Run:
```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
git commit -m "$(cat <<'EOF'
release(toolkit): mnemonic-toolkit v0.28.5 — doc-only fixes (plan-smoke + envelope-schema-version)

Closes 2 v0.28+ FOLLOWUPs:
- plan-smoke-step4-ms1-on-bundle-not-supported — plan-doc §6.3 step 4
  in PLAN_v0_27_0_bsms_round_trip_and_wallet_import_handoff.md
  referenced a nonexistent `--ms1` flag on `mnemonic bundle`;
  corrected to `--slot @0.phrase=`.
- import-wallet-envelope-schema-version-narrative-drift — added
  cross-reference doc-comments at the two `schema_version` constant
  sites in cmd/import_wallet.rs (outer envelope L87 + inner
  BundleJson literal at L~975) so future readers have at-site
  disambiguation between the two independently-evolving fields.

No functional code change. No test cells changed. No GUI lockstep.
No CLI surface change.

Cycle 1 of v0.28+ residual FOLLOWUP release plan (Wave 1 first
ship). See design/BRAINSTORM_v0_28_plus_residual_followups.md.
Sonnet reviewer GREEN.

Tooling: Cargo.toml version 0.28.4 → 0.28.5; CHANGELOG entry;
scripts/install.sh:32 self-pin bumped (install-pin-check.yml CI gate
green on tag push).
EOF
)"
```

- [ ] **Step 4: Backfill the FOLLOWUPS Status SHA**

Run:
```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
SHA=$(git rev-parse HEAD)
sed -i "s/resolved <PLACEHOLDER-COMMIT-SHA>/resolved $SHA/g" design/FOLLOWUPS.md
git add design/FOLLOWUPS.md
git commit --amend --no-edit
```

(Per A/B/C lesson: amending changes SHA; Status notes reference orphaned object. Acceptable — tag is the durable audit anchor.)

- [ ] **Step 5: Tag mnemonic-toolkit-v0.28.5**

Run:
```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
git tag mnemonic-toolkit-v0.28.5
git tag -l 'mnemonic-toolkit-v0.28*'
```

Expected: tags v0.28.0 through v0.28.5 present.

- [ ] **Step 6: Push master + tag**

Run:
```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
git push origin master
git push origin mnemonic-toolkit-v0.28.5
```

Expected: both pushes succeed; tag push triggers `install-pin-check.yml` CI workflow.

---

### Task 8: Monitor CI + create GH Release

**Files:** none modified.

- [ ] **Step 1: Monitor CI runs**

Use Monitor tool poll script (mirrors A/B/C precedent):

```bash
prev=""
while true; do
  s=$(gh run list --limit 4 --json databaseId,name,headBranch,status,conclusion,createdAt 2>/dev/null || echo '[]')
  cur=$(jq -r '.[] | select(.headBranch == "master" or .headBranch == "mnemonic-toolkit-v0.28.5") | "\(.databaseId) \(.headBranch) \(.name): \(.status)/\(.conclusion // "-")"' <<<"$s" | sort)
  comm -13 <(echo "$prev") <(echo "$cur")
  prev=$cur
  remaining=$(jq -r '[.[] | select(.headBranch == "master" or .headBranch == "mnemonic-toolkit-v0.28.5") | select(.status != "completed")] | length' <<<"$s")
  [ "$remaining" = "0" ] && break
  sleep 30
done
```

Expected:
- `install-pin-check` on `mnemonic-toolkit-v0.28.5` tag: PASS (10s).
- `rust` on master: likely SKIPPED (no `crates/` changes — verify paths filter); IF triggered, expect PASS.
- `manual` on master: likely SKIPPED (no `docs/manual/` changes); IF triggered, expect PASS.

- [ ] **Step 2: Create GH Release**

Per project convention (manual `gh release create` post-tag-push for toolkit tags):

```bash
gh release create mnemonic-toolkit-v0.28.5 \
  --title 'mnemonic-toolkit v0.28.5 — doc-only fixes (plan-smoke + envelope-schema-version)' \
  --notes "$(cat <<'EOF'
Patch release: 2 doc-only fixes closing v0.28+ FOLLOWUPs surfaced in the post-A/B/C recon dossier.

### Documentation

- **`design/PLAN_v0_27_0_bsms_round_trip_and_wallet_import_handoff.md` §6.3 step 4** — Replace nonexistent \`--ms1\` flag (which doesn't exist on the \`bundle\` subcommand) with \`--slot @0.phrase=\` per \`mnemonic bundle --help\`. Closes FOLLOWUP \`plan-smoke-step4-ms1-on-bundle-not-supported\`.

- **\`cmd/import_wallet.rs:87 + :~975\`** — Add cross-reference doc-comments at both \`schema_version\` constant sites (outer envelope \`"1"\` + inner BundleJson \`"4"\`). The two constants share the name but evolve independently; comments now make the disambiguation explicit at-site. Closes FOLLOWUP \`import-wallet-envelope-schema-version-narrative-drift\`.

### Cycle context

Cycle 1 of v0.28+ residual FOLLOWUP release plan (Wave 1 first ship). See [\`design/BRAINSTORM_v0_28_plus_residual_followups.md\`](https://github.com/bg002h/mnemonic-toolkit/blob/master/design/BRAINSTORM_v0_28_plus_residual_followups.md). No CLI surface change; no test cells changed; no GUI lockstep.
EOF
)"
```

---

## Self-review

After completing all 8 tasks, verify against the brainstorm spec:

1. **Spec coverage:**
   - Cycle 1 Phase 0 (recon) → Task 1 ✓
   - Phase 1 (plan-smoke fix) → Task 2 ✓
   - Phase 2 (schema_version cross-refs) → Task 3 ✓
   - Phase 3 (commit + tag + push + GH Release) → Tasks 4, 7, 8 ✓
   - Phase 4 (FOLLOWUPS Status flips × 2) → Task 6 ✓

2. **No-placeholder check:** `<YYYY-MM-DD>` (CHANGELOG date) and `<PLACEHOLDER-COMMIT-SHA>` (Status flip) are template placeholders backfilled at execution time per A/B/C precedent. No TBD/TODO blocks.

3. **Type consistency:** N/A (markdown + doc-comments only; no Rust types).

4. **Effort estimate sanity-check:** ~2 hours per brainstorm. Task 1 (~5 min recon); Task 2 (~10 min); Task 3 (~15 min); Task 4 (~15 min); Task 5 (~10 min reviewer); Tasks 6-8 (~30 min release tooling + CI watch). Realistic.

---

## Risk flags

- **`mnemonic bundle --help` regression** — Task 1 Step 2 confirms `--ms1` is absent. If a future toolkit cycle adds `--ms1` to `bundle`, Task 2's edit would become unnecessary (but harmless). Low risk.

- **Plan-doc line drift** — Task 1 Step 1 captures the actual L793 (or wherever step 4 lives at execution time). Plan-doc cite at SHA `0ca86b5` is the recon anchor; verify at execution.

- **schema_version comment placement** — Task 3 Step 4 inserts an INLINE comment just above the `schema_version: "4",` line inside the `BundleJson { ... }` literal. Some Rust formatters may reformat the multi-line struct literal; if `cargo fmt` is run, the inline comment may shift. Mitigation: don't run `cargo fmt` in this cycle (no other code changes warrant a fmt pass).

- **Sub-skill expectations.** This plan assumes the executor uses `superpowers:subagent-driven-development` or `superpowers:executing-plans`. Direct manual execution is fine for a doc-only cycle this small but loses the per-task review gate.
