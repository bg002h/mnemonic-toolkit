**⚠ 2 Important (proposed, see "Proposed grades"), 0 Critical. The `[patch.crates-io]` CANNOT be dropped in any repo on 13.2.0, and 14 of the 26 toolkit tracker citations checked are false against current bytes.**

# cycle-prep recon — 2026-11-03 — taproot-coverage-cycle-on-miniscript-gt-13-1-0 + upstream-miniscript-taptree-depth2-display-asymmetry (toolkit) + upstream-miniscript-taptree-depth2-display-asymmetry, md-codec-sortedmulti-a-to-miniscript-rendering-gap, md-cli compile.rs tripwire (descriptor-mnemonic)

**Origin/master SHA at recon time:** `02052a17` (mnemonic-toolkit). Companions read at descriptor-mnemonic `origin/main` `c8bb97a5`, mnemonic-refugium `origin/master` `5ed5b66f`, mnemonic-transaction `origin/main` `0ea8bb9`.
**Local branch:** `recon/miniscript-13-2` (worktree `/scratch/code/shibboleth/tk-cycle-prep-ms132`, created from `origin/master`).
**Sync state:** up-to-date (`git rev-list --left-right --count HEAD...origin/master` = `0 0` before this report's commit). The main toolkit checkout is 78 commits behind `origin/master`, so every toolkit citation is read from this worktree's bytes (= origin/master).
**Untracked:** none in the worktree.

Slug(s) verified: `taproot-coverage-cycle-on-miniscript-gt-13-1-0`, `upstream-miniscript-taptree-depth2-display-asymmetry` (toolkit companion), and in descriptor-mnemonic `upstream-miniscript-taptree-depth2-display-asymmetry`, `md-codec-sortedmulti-a-to-miniscript-rendering-gap`, the md-cli compile.rs tripwire entry. Drift expectation: HIGH, and confirmed. The text predates the 2026-08-20 pin advance (95fdd1c -> ff4732e5, commit `5f88071c`) and the 2026-10-17 13.2.0 release; none of the five entries was touched by either event.

Settled facts cited, not re-derived (brief): 13.2.0 published 2026-10-17; CHANGELOG lists #1009 #1010 #1039 #1042 #1054 #1061 #1074 and names neither #953 nor #910; no `Terminal::SortedMultiA` in 13.2.0; toolkit, md and mr resolve miniscript to git `ff4732e5` via `[patch.crates-io]`; the tripwire is RED weekly since 2026-10-19. Re-checked here anyway because they were one command each: the crates.io tarball `miniscript-13.2.0.crate` (sha256 prefix `4d79651cdf97eaf4`, equal to the `checksum` in the re-locked Cargo.lock) has 0 matches for `SortedMultiA` and 0 for `fn derive_at_index` under `src/`.

Citation tags below are machine-counted by the closing script (see Appendix A). "Wrong" means the claim is false against current bytes, not that it was false when written.

---

## Per-slug verification

### taproot-coverage-cycle-on-miniscript-gt-13-1-0 (toolkit umbrella, `design/FOLLOWUPS.md:4546-4563`)
- **WHAT (from FOLLOWUPS.md):** one upstream event, the first rust-miniscript crates.io release > 13.1.0 carrying #953 and #910, dissolves three blockers (depth-2 Display bug, md-codec unable to render `SortedMultiA`, the toolkit `[patch.crates-io]`), then a cross-repo cycle lifts two restore gates and drops the patch.
- **Citations:**
  - `[C1]` `FOLLOWUPS.md:4548` tripwire `.github/workflows/miniscript-fork-tripwire.yml` weekly, `BASELINE` "currently `13.1.0`" — **ACCURATE** `miniscript-fork-tripwire.yml:75` `BASELINE: '13.1.0'`.
  - `[C2]` `FOLLOWUPS.md:4548` "verified highest published — 88 versions total" — **STRUCTURALLY-WRONG** (now false) the index query on 2026-11-03 gives 90 versions (74 not yanked), highest non-yanked non-prerelease `13.2.0` (`tripwire.py` run, Appendix C).
  - `[C3]` `FOLLOWUPS.md:4548` "NOTIFIES rather than auto-drops — a release > 13.1.0 is necessary but not sufficient, since it must actually contain #953 + #910" — **ACCURATE** the workflow header says the same (`miniscript-fork-tripwire.yml:31-34`, "a 13.2.0 backport that omits them would fire this job spuriously"); that is exactly what happened.
  - `[C4]` `FOLLOWUPS.md:4548` fail-closed/green/red logic (baseline 13.1.0 -> red when a newer release exists; raised baseline -> green) — **ACCURATE** the workflow's embedded python, run locally against the live index: `BASELINE=13.1.0` prints the NEXT STEPS block and exits 1; `BASELINE=13.2.0` prints "No release newer than the baseline" and exits 0.
  - `[C5]` `FOLLOWUPS.md:4551` "(b) published rust-miniscript v13 has no `Terminal::SortedMultiA`" — **ACCURATE** for 13.2.0 too (crate source grep: 0 matches; `Descriptor::derive_at_index` also absent).
  - `[C6]` `FOLLOWUPS.md:4552` "Latest crates.io release is 13.1.0 (2026-06-09)" — **STRUCTURALLY-WRONG** 13.2.0 was published 2026-10-17 (CHANGELOG 2026-10-15); also `Cargo.toml:30` "Latest crates.io is 13.1.0".
  - `[C7]` `FOLLOWUPS.md:4552` "13.1.0 does NOT contain either (#953, #910)" — **ACCURATE** probe on `=13.1.0`: `tr(K,{{A,B},C})` Displays wrong and reparse fails (T1, T2, T4); `tr(K,sortedmulti_a(..))` rejected (T5).
  - `[C8]` `FOLLOWUPS.md:4551-4552` the trigger: "the first release > 13.1.0 carrying BOTH #910 and #953" and "that one release simultaneously dissolves THREE blockers" — **STRUCTURALLY-WRONG** 13.2.0 is a 13.x release (tag commit `783da44`, not an ancestor of upstream master). It delivers blocker (a) only, as backport #1039 (merge `319e3c7` "13.x: descriptor: fix taproot tree formatting"); it has no #910, #915 or #911, so (b) and (c) stay. The "PRs target the next major" premise means the two halves can never ship in one 13.x release.
  - `[C9]` `FOLLOWUPS.md:4552` "The toolkit's pinned rev `95fdd1c` (2026-05-02) INCLUDES #910 but PREDATES #953" — **STRUCTURALLY-WRONG** the pin is `ff4732e5f75aa555682343cb180fa72ee3e8e9d5` since 2026-08-20 (root `Cargo.toml:12,35`); `git log ff4732e5 --grep '#953'` finds it (and #910, #915, #911, #929). `95fdd1c` survives only in `fuzz/Cargo.toml:28`.
  - `[C10]` `FOLLOWUPS.md:4555` step 3 "DROP the `[patch.crates-io]` entry (both `crates/mnemonic-toolkit/Cargo.toml` AND the replicated copy in `fuzz/Cargo.toml`)" — **STRUCTURALLY-WRONG** the toolkit's patch is in the ROOT `Cargo.toml:34-35`; `crates/mnemonic-toolkit/Cargo.toml` has only a comment (:35) and the dependency `miniscript = { version = "13", ... }` (:97). The fuzz copy exists (`fuzz/Cargo.toml:27-28`) but at the stale rev.
  - `[C11]` `FOLLOWUPS.md:4557` "LIFT `restore.rs::ensure_taptree_depth_le_one` (+ its `classify_taproot_restore` call) and flip the depth-2 left-heavy + right-spine cells in `tests/cli_restore_taproot.rs`" — **STRUCTURALLY-WRONG** already done by `5f88071c` (2026-08-20): the function is now `ensure_taptree_wellformed` (`restore.rs:3453`, called :3369); the cells are `left_heavy_3leaf_tr_restores_depth2` (`cli_restore_taproot.rs:262`), `right_spine_3leaf_tr_also_restores_depth2` (:293) and `restore_help_says_depth2_taproot_restores` (:320). The root `Cargo.toml:24-28` comment already says so.
  - `[C12]` `FOLLOWUPS.md:4558` "`src/cmd/restore.rs::subtree_contains_sortedmulti_a` gate -> reconstruct `sortedmulti_a`-under-a-taptree" — **ACCURATE** (location) `fn subtree_contains_sortedmulti_a` `restore.rs:3422`, gate :3362, refusal text :3366. The gate's stated reason is stale: md-codec renders `sortedmulti_a` at a tap-leaf root (`to_miniscript.rs:689-699`). Spike (Appendix E): with the gate bypassed in a scratch worktree, `restore --md1` on `tr(NUMS,{sortedmulti_a(2,K0,K1),pk(K2)})` and the swapped-leaf form both exit 0 and print a `sortedmulti_a` descriptor; the first receive address is identical for both leaf orders. Not oracle-verified.
  - `[C13]` `FOLLOWUPS.md:4559` component `restore-general-and-multi-leaf-taproot-roundtrip` remainder items (i)+(ii) — **ACCURATE** (slug exists, heading `FOLLOWUPS.md:4512`; item (i) is done per C11, item (ii) is the C12 gate).
  - `[C14]` `FOLLOWUPS.md:4560` fuzz workspace "replicates the patch ... must realign; see `fuzz-nightly-quarterly-bump`" — **ACCURATE** (`fuzz/Cargo.toml:27-28`; slug heading `FOLLOWUPS.md:4479`); the replica is stale, see cross-cutting 6.
  - `[C15]` `FOLLOWUPS.md:4561` "Status: `open` ... nothing toolkit-side is actionable until then" — **STRUCTURALLY-WRONG** five toolkit-side items are actionable now (Q3 below).
  - `[C16]` root `Cargo.toml:30-33` "DROP this [patch] when the first release > 13.1.0 ships carrying #953 AND #910" — **STRUCTURALLY-WRONG** (incomplete) the toolkit also needs #915 (`Terminal::SortedMulti`, `WshInner` reshaping) and #911 (`derive_at_index`, `into_definite`, `TryFrom`); the build-check shows these dominate the error list (27 errors).
- **Action for brainstorm spec:** rewrite the trigger as "a release (or the 14.0 line) carrying #910, #915 and #911, or a decision to port to the 13.x API"; record that the Display half was delivered by 13.2.0 / #1039; cite miniscript `release-13.x` tag commit `783da44` and toolkit `02052a17`.

### upstream-miniscript-taptree-depth2-display-asymmetry (toolkit companion, `design/FOLLOWUPS.md:4500-4510`)
- **WHAT:** the toolkit's pinned miniscript shares the depth-2+ taptree Display bug; latent until a general-tr reconstruction exists; lift the depth gate on the fixed release.
- **Citations:**
  - `[C17]` `FOLLOWUPS.md:4503` ">=3-leaf taptree Displays as `{{a,b,c}}` and does not reparse (<=13.1.0)" — **ACCURATE** probe on `=13.1.0`: T1 `{{A,B},C}`, T2 `{{A,B},{C,D}}`, T4 depth-3-left all `display_eq_input=false reparse-FAILS`.
  - `[C18]` `FOLLOWUPS.md:4503` "Fixed on miniscript master (#953, 2026-05-25) but UNRELEASED" — **STRUCTURALLY-WRONG** released in 13.2.0 as the 13.x backport #1039. Probe on `=13.2.0`: T1, T2, T3, T4 display unchanged and reparse equal (same as ff4732e5).
  - `[C19]` `FOLLOWUPS.md:4504` the pinned rev `95fdd1c` "shares the bug" — **STRUCTURALLY-WRONG** the toolkit pin is `ff4732e5` (contains #953). `95fdd1c` is only the fuzz pin.
  - `[C20]` `FOLLOWUPS.md:4504` classifier refuses non-multi_a/sortedmulti_a "BEFORE any Display (`src/cmd/restore.rs:685-689`, `:710`)" — **STRUCTURALLY-WRONG** those arms no longer exist; `classify_taproot_restore` (`restore.rs:3291`) admits `GeneralFaithful` (:3369-3370).
  - `[C21]` `FOLLOWUPS.md:4504` `faithful_multisig_descriptor` at `restore.rs:938`, "non-taproot-only" — **DRIFTED-by-2849** now `restore.rs:3787`; and it is no longer non-taproot-only (GeneralFaithful tr falls through to it, :3965, :4070).
  - `[C22]` `FOLLOWUPS.md:4505` the bug is "LEFT-branch-specific, not depth-specific" — **ACCURATE** 13.1.0 probe: T3 right-spine `{A,{B,C}}` Displays and reparses fine; T1, T2, T4 fail.
  - `[C23]` `FOLLOWUPS.md:4506` "v0.55.1 ... refuses depth >=2 structurally (`ensure_taptree_depth_le_one`, exit 2, citing this slug)" — **STRUCTURALLY-WRONG** gate lifted by `5f88071c`; see C11.
  - `[C24]` `FOLLOWUPS.md:4507` Actions on close: lift the gate and flip the two cells — **STRUCTURALLY-WRONG** already done; see C11.
  - `[C25]` `FOLLOWUPS.md:4508` "Status: `open` (the depth>=2 restore refusal is live and slug-citing ...)" — **STRUCTURALLY-WRONG** the refusal is gone, the display fix is released (13.2.0) and in the pin (ff4732e5). Nothing remains open on this entry except deleting it or marking it resolved.
  - `[C26]` `FOLLOWUPS.md:4510` companion pointer to descriptor-mnemonic's same-slug entry — **ACCURATE** md `design/FOLLOWUPS.md:2094`.
- **Action for brainstorm spec:** mark resolved: "fixed in the pin since 2026-08-20 (#953 in ff4732e5, `5f88071c`); released 2026-10-17 in 13.2.0 as #1039". Cite toolkit `02052a17`, miniscript `319e3c7`.

### upstream-miniscript-taptree-depth2-display-asymmetry (descriptor-mnemonic, `design/FOLLOWUPS.md:2094-2104`, origin/main `c8bb97a5`)
- **WHAT:** pinned miniscript 13.0.0 Displays depth-2 taptrees malformed; md-codec's generator is capped at depth <= 1 until upstream releases #953.
- **Citations:**
  - `[C27]` `:2096` repro `TapTree::combine(combine(a,b),c)` Displays `tr(K,{{pk(a),pk(b),pk(c)}})` and miniscript rejects it — **ACCURATE** for 13.1.0 (probe, C17; the entry's own 13.0.0 repro was not re-run); no longer true at 13.2.0 or ff4732e5.
  - `[C28]` `:2097` generator "minimally constrained to taptree depth <= 1 (`tests/common/mod.rs::t_tr_tree` doc comment)" — **STRUCTURALLY-WRONG** `t_tr_tree` (`tests/common/mod.rs:1540`, doc :1525-1539) is "DEPTH <= 2 since the ff4732e pin (2026-08-20)".
  - `[C29]` `:2097` characterization cell `upstream_taptree_depth2_display_asymmetry` asserts reparse `Err` with an "UPSTREAM FIXED?" flip message — **STRUCTURALLY-WRONG** the cell (`tests/proptest_to_miniscript.rs:605`) was inverted: it now asserts reparse Ok and panics with "UPSTREAM REGRESSED" (:632). Probe on 13.2.0 shows the inverted cell would still pass.
  - `[C30]` `:2098` "NOT in any release ... 13.1.0 latest ... No 13.x backport PR exists" — **STRUCTURALLY-WRONG** backport #1039 exists and is in 13.2.0.
  - `[C31]` `:2100` Actions on close (2) "the flip cell goes red -> invert it, restore the depth-2 arm in `t_tr_tree`" — **STRUCTURALLY-WRONG** done at the ff4732e pin (C28, C29).
  - `[C32]` `:2101` "Status: `open`" — **STRUCTURALLY-WRONG** same as C25.
  - `[C33]` `:2103` companion pointer to the toolkit same-slug entry — **ACCURATE** toolkit `design/FOLLOWUPS.md:4500`.
  - `[C34]` md root `Cargo.toml:22-30` pin rationale "#953 ... in NO release through 13.1.0 ... a crates.io version cannot carry it" — **STRUCTURALLY-WRONG** (for 13.2.0) a crates.io version now does carry the Display fix. The pin is still required, for a different reason the comment only half states: md-codec calls `Terminal::SortedMultiA` (`to_miniscript.rs:697`), `derive_at_index`/`into_definite` (`derive.rs:148,150`) and the #915 `Terminal::SortedMulti`/`WshInner` shapes (`descriptor_route.rs:432-474`).
- **Action for brainstorm spec:** mark resolved at the ff4732e pin; correct the Cargo.toml comment to name #910/#915/#911 as the live reasons. Cite md `c8bb97a5`.

### md-codec-sortedmulti-a-to-miniscript-rendering-gap (descriptor-mnemonic, `design/FOLLOWUPS.md:2074-2083`)
- **WHAT:** `to_miniscript` cannot render `Tag::SortedMultiA`; options (a) bump the pin and add the arm, (b) lower to sorted `multi_a`, (c) wontfix.
- **Citations:**
  - `[C35]` `:2076` "`crates/md-codec/src/to_miniscript.rs:407-411` `(Tag::SortedMultiA, Body::MultiKeys {..})` unconditionally `Err`" — **STRUCTURALLY-WRONG** at `c8bb97a5` the tap-leaf-root case converts to `Terminal::SortedMultiA(thresh)` (`to_miniscript.rs:689-699`, comment :685 "Writable at all only since the ff4732e pin"); the remaining `Err` arm (:869, message :889) is the deliberate BIP-386 refusal for every other position.
  - `[C36]` `:2076` "`Tag::MultiA` at `:394-398` DOES render" — **DRIFTED-by-446** now `to_miniscript.rs:840` `(Tag::MultiA, Body::MultiKeys { k, indices })`.
  - `[C37]` `:2076` "md-codec's `Cargo.toml` pins `miniscript = 13.0.0` (crates.io)" — **STRUCTURALLY-WRONG** the requirement text is still `13.0.0` (`Cargo.toml:18`) but it resolves to git `ff4732e5` through `[patch.crates-io]` (:43-44).
  - `[C38]` `:2076` "a pin + missing-arm gap, not a true upstream gap — rust-miniscript git rev `95fdd1c` HAS `Terminal::SortedMultiA`" — **ACCURATE** (diagnosis) `git grep SortedMultiA 95fdd1c5 -- src/miniscript/decode.rs` = 4 matches; ff4732e5 has it too; 13.2.0 does not.
  - `[C39]` `:2078` option (a) "bump md-codec's miniscript pin to a rev with `SortedMultiA` + add the arm" — **STRUCTURALLY-WRONG** (as an open option) shipped for the tap-leaf-root case on 2026-08-20; option (b) (lower to sorted `multi_a`) was not needed.
  - `[C40]` `:2080` "toolkit v0.49.1 reconstructs `tr-sortedmulti-a` by routing AROUND md-codec" — **ACCURATE** `restore.rs` still routes single-leaf `sortedmulti_a` to `TaprootRestore::Template(CliTemplate::TrSortedMultiA, ..)` (:3355-3359).
  - `[C41]` `:2081` "Status: `open`" — **STRUCTURALLY-WRONG** closed for the leaf root. What remains is not a gap but a standards rule: md-codec refuses `sortedmulti_a` outside a tap-leaf root (BIP-386 puts it in BIP-387's category, `to_miniscript.rs:869-889`).
  - `[C42]` `:2082` companion pointer toolkit `design/FOLLOWUPS.md::md-codec-sortedmulti-a-to-miniscript-rendering-gap` — **ACCURATE** heading at toolkit `FOLLOWUPS.md:698` (that entry is also stale: it cites `to_miniscript.rs:406-410` and `Status: open`).
- **Action for brainstorm spec:** mark resolved (leaf-root) with the remaining refusal recorded as intentional; the remaining open question is the toolkit gate (C12), a toolkit decision. Cite md `c8bb97a5`.

### md-cli compile.rs tripwire (`upstream_display_is_still_broken_delete_local_renderer_when_this_fails`; md `design/FOLLOWUPS.md` entry `all-features-suite-is-red-and-ungated-by-ci`, :2574 onward)
- **WHAT:** the tripwire test in `crates/md-cli/src/compile.rs` fired when upstream Display stopped flattening depth-2 taptrees; delete the local `render_tr_template` and widen CI to `--all-features`.
- **Citations:**
  - `[C43]` entry "the test is at `crates/md-cli/src/compile.rs`" — **STRUCTURALLY-WRONG** (now) the test and `render_tr_template` are gone (grep of origin/main `compile.rs`: the name occurs only in the comment at :287 "Since P1 deleted the local `render_tr_template` renderer"). Deleted by `1b36bf6b` (an ancestor of origin/main).
  - `[C44]` entry "Upstream miniscript's `Display` no longer flattens a depth-2 taptree" (left `tr(@4,{{pk(@3),pk(@2)},{pk(@1),pk(@0)}})`) — **ACCURATE** at ff4732e5 and at 13.2.0 (probe T1, T2).
  - `[C45]` entry "`ci.yml:48` runs `cargo test --workspace --all-targets` with no `--all-features`" — **DRIFTED-by-35** the job is now `ci.yml:83` `cargo nextest run --workspace --all-targets --all-features --no-fail-fast` (fixed by `400a2f77`, also an ancestor of origin/main).
  - `[C46]` entry "CLOSED by P1 (2026-08-30)", commits `1b36bf6b` and `400a2f77` — **ACCURATE** both commits resolve and are ancestors of `origin/main` `c8bb97a5`.
- **Would it fire on 13.2.0?** No md tripwire test is left to fire: this one fired and was consumed on 2026-08-30. The successor guard, `upstream_taptree_depth2_display_asymmetry` (`proptest_to_miniscript.rs:605`), asserts the fix and panics "UPSTREAM REGRESSED" on a regression; 13.2.0 passes the same shapes (probe a). Not run as a test (md does not build on 13.2.0, see Q1).
- **Action for brainstorm spec:** none for this entry beyond noting it is closed and its tripwire consumed. Cite md `c8bb97a5`.

---

## Cross-cutting observations

**Q1. Can `[patch.crates-io]` be dropped anywhere? No.** Method (each in a scratch worktree, never /tmp): delete the `[patch.crates-io]` block, delete the git `miniscript` `[[package]]` stanza from `Cargo.lock`, `cargo metadata` re-locks to `miniscript v13.2.0` (`cargo update -p miniscript --precise 13.2.0` itself fails first: "revspec '13.2.0' not found", because the lock still names the git source), then `cargo check --workspace --all-targets --keep-going`. Logs in `/scratch/code/shibboleth/tk-cycle-prep-ms132-logs/`.

| repo / workspace | result on 13.2.0 | first errors |
|---|---|---|
| mnemonic-toolkit (root) | FAIL, 3 errors, all in the md-codec git dep (`cf35d61a`, 0.47.0), before any toolkit code compiles (`tk-check.log`) | `md-codec/src/derive.rs:148` `derive_at_index`; `:150` `into_definite`; `to_miniscript.rs:670` `Terminal::SortedMultiA` |
| toolkit own code (md-codec scratch-stubbed with 2 edits so rustc reaches it) | FAIL, 27 errors in the `mnemonic` bin (and again in its test build) plus 12 integration-test targets that do not compile (`tk-check2.log`) | `derive_at_index` missing at `derive_address.rs:49,133`, `cost/strip.rs:55`, `cmd/restore.rs:1426`; `TryFrom<Descriptor<DescriptorPublicKey>> for Descriptor<DefiniteDescriptorKey>` missing at `restore.rs:1429`, `strip.rs:59`, `derive_address.rs:55,137`; `Terminal::SortedMulti` missing at `parse_descriptor.rs:801`, `wallet_export/mod.rs:317`; `Terminal::SortedMultiA` missing at `parse_descriptor.rs:814`; `ShInner::SortedMulti` arm required at `wallet_export/mod.rs:223,273`; `WshInner::requires_sig` missing at `descriptor_builder/allow.rs:188,190`; `WshInner` vs `Miniscript` mismatch at `strip.rs:66,71`, `parse_descriptor.rs:652,664`, `timelock_advisory.rs:190,194`, `wallet_export/mod.rs:321,323` |
| descriptor-mnemonic (`c8bb97a5`) | FAIL, `md-codec` lib 10 errors; `md-cli` never reached (`md-check.log`) | `derive.rs:148,150`; `descriptor_route.rs:432,434,440(x3),443,461,474`; `to_miniscript.rs:697` |
| mnemonic-refugium (`5ed5b66f`) | FAIL: `md-codec` 10 errors (`mr-check.log`); with md-codec stubbed, `mr-codec` 5 more (`mr-check2.log`) | `mr-codec/src/export.rs:1064,1142`, `render.rs:119`, `upgrade.rs:664` (`derive_at_index`); `cost.rs:145` (generic `Descriptor<bitcoin::PublicKey>::plan`, #929) |
| mnemonic-transaction (`0ea8bb9`) | not applicable | `Cargo.lock` has 0 `miniscript` packages; no patch |
| toolkit `fuzz/` workspace | follows the toolkit (path dep `../crates/mnemonic-toolkit`); not droppable. Separately `cargo check --locked` fails on current bytes (stale lock, `fuzz-check-orig.log`) | `fuzz/Cargo.lock` pins miniscript `95fdd1c5` and md-codec registry 0.42.0, while the toolkit wants md-codec git `cf35d61a`; CI (`fuzz-smoke.yml:86`) builds unlocked and is green |
| mr `scripts/factcheck` | follows mr; patch at `scripts/factcheck/Cargo.toml:13-14`; not built separately | |
| also carrying the patch, not built | `refugium-wallet/Cargo.toml:49-52`, `mnemonic-gui/Cargo.toml:127-134` (both `ff4732e5`); both consume toolkit/md/mr, so they inherit the failures | |

Fork-only API actually called (all from #910, #911, #915, #929, absent in 13.2.0): `Descriptor::derive_at_index`, `Descriptor::into_definite`/`TryFrom` (#911); `Terminal::SortedMulti`, `ShInner`/`WshInner` reshaping, `WshInner::requires_sig` (#915); `Terminal::SortedMultiA` (#910); generic `Descriptor::plan` (#929). 13.2.0 removed the `DescriptorKeyParseError::{Bip32Xpriv, Bip32Xpub}` variants (#1009); no source in toolkit, md, mr, refugium-wallet or mnemonic-gui names `Bip32Xpriv`, `Bip32Xpub` or `DescriptorKeyParseError`, so #1009 costs nothing here. `WalletPolicy`/`KeyInfo` (#1009, new in 13.2.0): no use in any repo (the grep hits are the unrelated `CosignerKeyInfo` type).

**Q2. Behaviour differences, ff4732e5 vs 13.2.0** (differential probe crate, one `main.rs`, built three ways: `=13.1.0`, `=13.2.0`, git `ff4732e5`; all three outputs reproduce from the saved binaries):

| probe row | 13.1.0 | 13.2.0 | ff4732e5 |
|---|---|---|---|
| T1/T2/T4 depth>=2 left taptree Display + reparse | BUG (display differs, reparse fails) | OK, unchanged, reparses equal | OK |
| T3 right-spine taptree | OK | OK | OK |
| T5 `tr(K,sortedmulti_a(2,B,C))` | parse error | parse error | OK |
| T6 `tr(K,{sortedmulti_a,pk})` | parse error | parse error | OK |
| T12 `wsh(and_v(v:pk,sortedmulti(..)))` nested sortedmulti | parse error | parse error | OK (more permissive than BIP-388) |
| T14 xpub with 256 unhardened steps (#1074) | OK | parse error ("deeper than BIP32's maximum depth of 255") | OK |
| T15 255 steps | OK | OK | OK |
| T16 hardened wildcard through `at_derivation_index` | PANIC | `Err` | `Err` |
| T8/T20/T21 raw pkh in a tap leaf, via `Descriptor::from_str` (#1061) | parse error | parse error | parse error |
| T19/T24 multipath tr | OK | OK | OK |

Reading it: the taptree Display behaviour is identical at 13.2.0 and ff4732e5 (this is the half 13.2.0 delivers); the differences that matter are the missing `sortedmulti_a` (T5, T6), the rejected nested `sortedmulti` (T12, which md-codec and the toolkit already refuse themselves, so only the error source changes) and the new depth-255 key rejection (T14). #1061 shows no delta through `Descriptor::from_str` (all three reject raw pkh in a tap leaf); its change is the error variant on the direct `Tr` parse path (`tr/mod.rs:584` `ContainsRawPkh`), which no repo calls (`Tr::from_str` has 0 uses). #1010 (key lookup outside the descriptor path) governs `GetKey`/PSBT signing; no repo source in toolkit, md or mr uses miniscript's `GetKey` (mr's `KeyRequest` is its own API struct in `mr-codec`). #1042 changes `Tr::translate_pk` to visit the internal key first (13.2.0 `tr/mod.rs:258`; ff4732e5 visits the tree first, `tr/mod.rs:257-262`), while `ForEachKey` order is the same in both (`tr/mod.rs:443-448` vs `:433-438`). The only two `Translator` impls in the three repos (`ReconstructTranslator`, `restore.rs:3686`; `Placeholders`, md-cli `decompose/walk.rs:316`) look keys up by value in a map, so call order is irrelevant. Tests that would change on a port: every test whose descriptor contains `sortedmulti_a` (files mentioning `sortedmulti_a`/`SortedMultiA`: 48 in the toolkit of which 21 under `tests/`; 30 in md of which 12; 21 in mr of which 9), all the 12 failing toolkit integration targets, and tests asserting the nested-`sortedmulti` error text (the only negative assertion found on an error string, `unrecognized name`, is `cli_bundle_keyless_descriptor.rs:131` and md-cli `cmd_decompose.rs:687` on 'garbage' input, unaffected). The depth-2 refusal tests are already inverted; none flips back.

**Q3. Actionable now (k = 5).**
1. **Tripwire.** Raise `BASELINE` `13.1.0` -> `13.2.0` (`miniscript-fork-tripwire.yml:75`) and add the comment its own NEXT STEPS item 3 asks for: 13.2.0 carries the Display fix as #1039 and none of #910/#915/#911, so the patch stays. Also reword NEXT STEPS item 1 ("check the CHANGELOG for PRs #953 and #910"): a reader doing that finds neither number in the 13.2.0 CHANGELOG (the backport is #1039) and would wrongly conclude the Display fix is missing; the real blockers are #910/#915/#911. Wording of the header lines 8-12, 18, 31-34 also names 13.1.0. A `.github/` change inside a normal reviewed PR is allowed by RULES.md; say so in the PR.
2. **Tracker and comment text (toolkit).** Fix the umbrella and companion (citations C2, C6, C8-C11, C15, C16, C18-C20, C23-C25), root `Cargo.toml:12-33`, `FOLLOWUPS.md:698` (`md-codec-sortedmulti-a-...` toolkit entry, stale `to_miniscript.rs:406-410`), and the manual `docs/manual/src/40-cli-reference/41-mnemonic.md:196-198` ("gated on the next rust-miniscript release (`Terminal::SortedMultiA`, > 13.1.0)"), which a reader of 13.2.0 would take as "satisfied". Manual lockstep applies to the manual edit.
3. **Stale pins.** `fuzz/Cargo.toml:28` is `95fdd1c5` against the root's `ff4732e5` (its comment claims "Kept byte-identical"); `fuzz/Cargo.lock` has miniscript `95fdd1c5` and md-codec registry 0.42.0; `ci/repro/remap-off-negative.sh:82` defaults `MINISCRIPT_REV` to `95fdd1c5`. The fuzz crate compiles on the old rev (95fdd1c already has #910 and #915), so this is drift, not breakage; the fuzz job fuzzes a different miniscript than the shipped binary.
4. **descriptor-mnemonic.** Correct the three entries (C27-C42), the `md-codec-unpublishable-while-patched-to-miniscript-master` entry (:3110-3122, "latest 13.1.0, checked 2026-09-17": the conclusion stands, 13.2.0 still lacks `derive_at_index`, `into_definite` and `Terminal::SortedMultiA`, but the date and version are stale) and the root `Cargo.toml:22-30` comment; delete the four vestigial CI steps that clone `bg002h/rust-miniscript` branch `md-codec-local-stack` into `../rust-miniscript-fork` (`ci.yml:62-67,113-118,172-177`, `bitcoind-differential.yml:70-74`): nothing in the tree references `../rust-miniscript-fork` outside the workflows and the historic CHANGELOG/MIGRATION text, and the patch is a git rev, so the clones do nothing.
5. **Toolkit `subtree_contains_sortedmulti_a` gate** (`restore.rs:3362`). The spike shows it can lift (C12): this is a feature decision (see Open questions), not a fix. If the gate stays, its refusal text (:3366, "md-codec cannot yet render it back as a non-root tap leaf") is false and should be reworded.

**Q3(b). Has any md gate or tripwire test fired, or would it?** The md-cli compile.rs tripwire fired on 2026-08-30 and was consumed (C43-C46). The inverted md-codec cell would not fire on 13.2.0. The toolkit's workflow tripwire has fired three weeks running (runs 2026-10-19, 10-26, 11-02 failure; 10-12 success). No md CI step watches for a new release.

**Q3(c). FOLLOWUPS text now false: "latest crates.io is 13.1.0".** Toolkit: `FOLLOWUPS.md:4548` (88 versions, highest 13.1.0), `:4552`; `Cargo.toml:30`; `miniscript-fork-tripwire.yml` (header). Descriptor-mnemonic: `FOLLOWUPS.md:3122` ("13.1.0 checked 2026-09-17"), `:2098` ("13.1.0 (latest tag)"), root `Cargo.toml:29` ("in NO release through 13.1.0").

**6. Stale-pin incidentals.** Beyond Q3 item 3: `ci/repro/vendor-freshness.sh:98`, `.github/workflows/man-pages.yml:140,223` and the committed `vendor/miniscript` all carry `ff4732e5` and must move together with any pin change (the repro-build lockstep); they are correct today.

**7. Tripwire design observation.** The tripwire compares the highest non-yanked release to a baseline. On 13.x it will fire again on every 13.x point release (13.2.1, 13.3.0) that cannot contain #910 (a release branch does not take features of the next major). It cannot see the event the toolkit actually waits for (a release carrying `SortedMultiA`, probably 14.0, which `miniscript = "13"` would never resolve; the workflow's own header makes this point). See Open questions.

**8. Sync/drift.** DRIFTED-by-N citations: 3 (C21 by 2849, C36 by 446, C45 by 35). The large majority of findings are not line drift but claims overtaken by the 2026-08-20 pin advance and the 2026-10-17 release.

### Proposed grades (not final; the controller grades)
The brief does not quote the likelihood test (Open question 4), so these use the CLAUDE.md definitions only.
- **Important (proposed) I-1:** the toolkit `miniscript-fork-tripwire` workflow is RED on master (3 consecutive scheduled runs). "A red suite is itself blocking." It is a notifier firing as designed, and the fix is the one-line baseline bump (Q3 item 1), but until someone acts the job cannot signal the real event (a release with `SortedMultiA`), and it trains readers to ignore red.
- **Important (proposed) I-2:** the toolkit umbrella and companion entries (and the root `Cargo.toml` drop rule) rest on an unsound premise: one release carrying #953 and #910 will dissolve three blockers. On the 13.x line that cannot happen, the Display half is already delivered, and the "lift the depth gate" work is already shipped. A brainstorm built on the entries as written would plan finished work and miss that the patch is also held by #915 and #911. 14 of the 26 toolkit citations (C1-C26) are STRUCTURALLY-WRONG.
- **Minor (proposed):** fuzz pin/lock drift (Q3.3); `remap-off-negative.sh` default; stale manual lines 196-198; stale refusal reason `restore.rs:3366` (fails closed, no printed command fails); vestigial md CI clone steps; stale md FOLLOWUPS statuses and the md `Cargo.toml` rationale; tripwire NEXT STEPS text omits #1039; toolkit `FOLLOWUPS.md:698` entry stale.
- Critical: none. Nothing here produced a wrong address, card or data.

---

## Recommended brainstorm-session scope

**Do not brainstorm a "taproot-coverage cycle on the next miniscript release" as written.** The premise is gone; split into one cheap cycle per repo and one optional feature.

1. **Cycle T1 (toolkit, NO-BUMP unless noted), ~120-200 lines, docs/CI only.** Q3 items 1, 2, 3. Lockstep: manual `41-mnemonic.md` edit (lockstep rule applies), `.github/` change declared in the PR description. If the refusal text in `restore.rs:3366` is reworded, that is a holder-facing string: PATCH and a class A review; if the gate is lifted in T3 instead, leave the string alone here. No clap change, so no GUI `schema_mirror`.
2. **Cycle M1 (descriptor-mnemonic, NO-BUMP), ~80-120 lines.** Q3 item 4. Independent of T1 and can run in parallel; companion text points both ways, so merge order is irrelevant but both PRs should cite each other.
3. **Cycle T3 (toolkit feature, optional, PATCH by precedent v0.55.3: a refusal becomes a faithful reconstruction, zero clap delta).** Lift `subtree_contains_sortedmulti_a` (`restore.rs:3362`), flip `tr_sortedmulti_a_in_2leaf_refuses` (`cli_restore_taproot.rs:352`), add Bitcoin Core oracle goldens for `sortedmulti_a` under a 2-leaf tree (the spike did not oracle-verify the address), update the manual line at `41-mnemonic.md:1313` ("a `sortedmulti_a` leaf inside a multi-leaf tree" is still refused). Class B (funds path: address derivation) for the whole-diff review, Opus s1 plus Sonnet s2. Needs a spec because the output is a printed descriptor and an address; ~150-250 lines with goldens. Depends on nothing upstream. Order: after T1 so the tracker text is already honest.
4. **Not scheduled: dropping the patch / publishing.** A port to the 13.2.0 API is a large, product-level change (toolkit 27 errors + 12 test targets, md-codec 10, mr 10 in md-codec plus 5 of its own, `sortedmulti_a` support would have to be hand-lowered or lost) and trades regression risk for crates.io publishability; or wait for the next-major line. This is a product decision (Open question 1). Do not size it until decided.
5. **Order:** T1 and M1 first (parallel), then T3 if wanted. The first piece to run is T1 item 1 (the tripwire), because it is red on master today.

### Open questions (product decisions not mine to make)
1. **Port or wait?** Port toolkit, md and mr to the 13.x API (13.2.0, hand-lowering `sortedmulti_a`), buying crates.io publishability of md-codec/md-cli/toolkit/gui at the cost of re-engineering `sortedmulti_a` parsing and every `derive_at_index` site; or keep the git pin and wait for the next-major release. Facts needed for the choice are in Q1.
2. **Tripwire semantics.** Keep "any release above BASELINE" (re-fires on each 13.x point release, needs a baseline bump each time), or change what it watches (e.g. a release whose crate source contains `Terminal::SortedMultiA`, or a 14.x line). Options only; not chosen.
3. **Lift the `sortedmulti_a`-under-taptree gate (T3)?** Yes/no and whether non-leaf positions stay refused (md-codec refuses them by a BIP-386 reading, `to_miniscript.rs:869-889`; the toolkit's string-builder path would still render them).
4. **The brief does not quote the likelihood test.** The two Important grades above are provisional until the controller applies it.

---

## Appendix

### A. Citation tally (machine-counted from this file)
Parsed by a short python script over the `[Cn]` lines (tag = the first bold ACCURATE / DRIFTED-by-N / STRUCTURALLY-WRONG on each line): 46 citations = 18 ACCURATE + 3 DRIFTED + 25 STRUCTURALLY-WRONG. By slug (accurate / drifted / wrong): toolkit umbrella C1-C16 = 8 / 0 / 8; toolkit companion C17-C26 = 3 / 1 / 6; md taptree C27-C34 = 2 / 0 / 6; md sortedmulti-a C35-C42 = 3 / 1 / 4; md-cli tripwire C43-C46 = 2 / 1 / 1.

### B. Scratch worktrees and logs (leave for the controller; not removed)
Worktrees: `/scratch/code/shibboleth/tk-cycle-prep-ms132-scratch` (toolkit, patch dropped), `/scratch/code/shibboleth/tk-cycle-prep-ms132-spike` (toolkit, gate bypass; detached), `/scratch/code/shibboleth/md-cycle-prep-ms132-scratch`, `/scratch/code/shibboleth/mr-cycle-prep-ms132-scratch`. Only `Cargo.toml`/`Cargo.lock` are modified in the three build-check worktrees; the spike also changes `restore.rs` (one condition) and appends one test. Logs, stubs and probes: `/scratch/code/shibboleth/tk-cycle-prep-ms132-logs/` (`tk-check.log`, `tk-check2.log`, `md-check.log`, `mr-check.log`, `mr-check2.log`, `fuzz-check-orig.log`, `spike-gate.log`, `miniscript.index`, `tripwire.py`, `probe/{a,b,c}`, `md-cf35d61-stub`, `md-6cf6599-stub`). Probe map: `a` = `=13.2.0`, `b` = git `ff4732e5`, `c` = `=13.1.0`.

### C. Commands run (as run)
- Re-lock: delete `[patch.crates-io]` and the git `miniscript` `[[package]]` stanza, then `cargo metadata` (log `*-meta.log`); check: `CARGO_TARGET_DIR=<logs>/target-{tk,md,mr} cargo check --workspace --all-targets --keep-going` (md with `--all-features`), `PATH=$HOME/.cargo/bin:$PATH`.
- Tripwire logic: the workflow's embedded python, extracted to `tripwire.py` and run against a saved copy of the live sparse-index response (`miniscript.index`), `BASELINE=13.1.0` then `BASELINE=13.2.0`. Result: `published versions : 90 (74 not yanked)`, `highest published : 13.2.0`; `13.1.0` -> NEXT STEPS block, exit 1; `13.2.0` -> "No release newer than the baseline", exit 0.
- Upstream facts: `git -C rust-miniscript-fork log --oneline miniscript-13.2.0 --grep='#NNN'` (tag `miniscript-13.2.0` = `783da44`): #1039, #1042, #1061, #1074, #1010, #1009 present; #910, #915, #911, #929 absent. `ff4732e5` history: #910, #915, #953, #911, #929 present; #1039, #1042, #1061, #1074, #1010, #1009 absent. `miniscript-13.2.0` is not an ancestor of `upstream/master`; `ff4732e5` is.

### D. Gate-bypass spike
In the detached worktree `tk-cycle-prep-ms132-spike`: `if subtree_contains_sortedmulti_a(inner)` -> `if false && subtree_contains_sortedmulti_a(inner)` at `restore.rs:3362`, plus a test that bundles `tr(NUMS,{sortedmulti_a(2,K0,K1),pk(K2)})` and the swapped-leaf form and prints `restore --md1` output. `cargo test --locked -p mnemonic-toolkit --test cli_restore_taproot spike_sortedmulti_a -- --nocapture` (log `spike-gate.log`): `test result: ok. 1 passed`; both runs `exit 0`, `bundle` descriptor equals the input, restore prints `descriptor: tr(5092...,{sortedmulti_a(2,...` and `first recv: bc1pw49n2w6ydsnmdcufryu6r3agpw3k4hmkhkzfr9z3wqs3xuvx74as88s86m` for both leaf orders. Not verified against Bitcoin Core or any second implementation.

Counts: citations 46 (accurate 18, drifted 3, wrong 25); patch droppable: none; actionable now: 5
