# v0.28.0 cycle — worktree cleanup tracker

**Purpose:** durable record of all agent-isolation worktrees created during the v0.28.0 cycle, so they can be torn down at cycle-end. Mitigates plan-doc hazard #6 (claude-code#51596: worktree branch names derive from 8-hex-char agentId prefix; stale prior-session worktrees can be silently reused).

**Cleanup procedure (run at end-of-cycle, post-P15 tag):**

```bash
cd /scratch/code/shibboleth/mnemonic-toolkit
for path in $(awk '/^- /{print $2}' design/v0_28_0-worktree-tracker.md); do
  git worktree remove -f -f "$path" 2>/dev/null || echo "already gone: $path"
done
# Then delete any orphaned branches:
git branch | grep -E "^[+ ]+(worktree-agent-|v0\.28\.0/)" | awk '{print $NF}' | xargs -r git branch -D
```

**CRITICAL:** do NOT delete a worktree while its branch is still active on an open PR. Verify all PRs are merged or closed first.

---

## Wave 0 worktrees (post-Wave-0-merge; can be deleted any time)

All Wave-0 PRs have been merged to `release/v0.28.0` (or closed-as-superseded for #35). Safe to delete.

- `/scratch/code/shibboleth/mnemonic-toolkit/.claude/worktrees/agent-a03e966d91667aeeb` — P0D (PR #33 merged @ `7281c46`) — branch `v0.28.0/p0d-sniff-format-consult-all`
- `/scratch/code/shibboleth/mnemonic-toolkit/.claude/worktrees/agent-a95bb223c397c3538` — P0B.1 (PR #35 closed-superseded by P0D) — branch `v0.28.0/p0b1-sniffoutcome-alphabetical-sort`
- `/scratch/code/shibboleth/mnemonic-toolkit/.claude/worktrees/agent-abfc0423c30471b3f` — P0B.2 (PR #32 merged @ `5283d85`) — branch `v0.28.0/p0b2-importprovenance-alphabetical-reorder`
- `/scratch/code/shibboleth/mnemonic-toolkit/.claude/worktrees/agent-ae866c5414bf2061d` — P0C (PR #34 merged @ `de3dc61`) — branch `v0.28.0/p0c-cli-dispatch-pre-stub`

## Wave 1 worktrees (active; do NOT delete until PRs merged)

Dispatched 2026-05-19 at `release/v0.28.0` @ `71592bc`. 10 instances; E (Jade P5) deferred per D→E dependency.

- `/scratch/code/shibboleth/mnemonic-toolkit/.claude/worktrees/agent-aa74aea6602d044ab` — P1 Sparrow (agentId `aa74aea6602d044ab`) — branch `v0.28.0/p1-sparrow`
- `/scratch/code/shibboleth/mnemonic-toolkit/.claude/worktrees/agent-ac5f72bf17658f95d` — P2 Specter — branch `v0.28.0/p2-specter`
- `/scratch/code/shibboleth/mnemonic-toolkit/.claude/worktrees/agent-ac2ba851956f8f8b3` — P3 Coldcard — branch `v0.28.0/p3-coldcard`
- `/scratch/code/shibboleth/mnemonic-toolkit/.claude/worktrees/agent-a3714405951085594` — P4 Coldcard-multisig — branch `v0.28.0/p4-coldcard-multisig`
- `/scratch/code/shibboleth/mnemonic-toolkit/.claude/worktrees/agent-aafd81d1108e8aebb` — P6 Electrum — branch `v0.28.0/p6-electrum`
- `/scratch/code/shibboleth/mnemonic-toolkit/.claude/worktrees/agent-a3917338e685941c1` — P7 BSMS 4-line (G1) — branch `v0.28.0/g1-bsms-4line`
- `/scratch/code/shibboleth/mnemonic-toolkit/.claude/worktrees/agent-a950bd1cb7e5d1166` — P8 BSMS taproot refusal (G2) — branch `worktree-agent-a950bd1cb7e5d1166` (will rename to `v0.28.0/g2-bsms-taproot` post-agent-completion)
- `/scratch/code/shibboleth/mnemonic-toolkit/.claude/worktrees/agent-a8a2f98e085b1ef9e` — P9 BSMS fixtures (G3) — branch `v0.28.0/g3-bsms-fixtures`
- `/scratch/code/shibboleth/mnemonic-toolkit/.claude/worktrees/agent-a55a47fbde322da73` — P10 Core fixtures (H) — branch `v0.28.0/h-core-fixtures`
- `/scratch/code/shibboleth/mnemonic-toolkit/.claude/worktrees/agent-abfb2c519f7dec3ae` — P12 compare-cost-tr (I) — branch `v0.28.0/i-compare-cost-tr`

## Deferred Wave-1 instance (E — Jade P5)

Not yet dispatched; D→E dependency per plan-doc R3-C1 fold. Will be added to this tracker upon dispatch.

## End-of-cycle bulk-delete script

After P15 (cycle tag), once `release/v0.28.0` is merged to master and tag fires:

```bash
cd /scratch/code/shibboleth/mnemonic-toolkit

# 1. Verify all v0.28.0 PRs are merged or closed
gh pr list --state all --base release/v0.28.0 --limit 30

# 2. Force-remove worktrees (force-flag pair needed because of lock-by-runtime)
git worktree list | awk '/locked/{print $1}' | xargs -r -I{} git worktree remove -f -f {}

# 3. Prune worktree records
git worktree prune

# 4. Delete orphaned branches
git branch | grep -E "^[+ ]+(worktree-agent-|v0\.28\.0/)" | awk '{print $NF}' | xargs -r git branch -D

# 5. Verify clean state
git worktree list
git branch | head -10
```

If a worktree refuses to remove due to active claude-code process holding the lock, the manual fallback is:
```bash
rm -rf <worktree-path>
git worktree prune
```
This is more forceful but works when `git worktree remove` cannot unlock.

## Wave-1 deferred-then-dispatched (E — Jade P5)

Dispatched 2026-05-19 post-D-merge at `release/v0.28.0` @ `387a709`.

- `/scratch/code/shibboleth/mnemonic-toolkit/.claude/worktrees/agent-a76b092f01263f9bb` — P5 Jade (agentId `a76b092f01263f9bb`) — branch `v0.28.0/p5-jade`
