# cycle-6 review — LENS B (canon-migration integrity + test correctness + build)

- **Cycle:** cycle-6 — timelock decaying-multisig decay-ordering (D-decay-rel + D-decay-abs)
- **Lens:** B — CANON-MIGRATION INTEGRITY + TEST CORRECTNESS + BUILD
- **HEAD:** `1617aa77` (`feat(cycle6-timelock): P2+P3 — decay-ordering predicates + canon migration`)
- **Base:** `3fa2925b` (`origin/master`)
- **Worktree:** `/scratch/code/shibboleth/wt-cycle6` (branch `feature/cycle6-timelock-decay`, 2 commits)
- **Date:** 2026-06-21
- **Reviewer:** opus architect (2-reviewer adversarial panel)

---

## Summary of verification performed

The fix adds two new producer checks for the `decaying-multisig` archetype in
`validate_params` (`descriptor_builder/archetype.rs`):
- **D-decay-rel** — unit-aware relative-timelock ordering (`older_unit_value`
  classifies BIP-68 bit-22 unit + low-16 value; cross-unit → refuse, same-unit
  `v_r <= v_p` → refuse).
- **D-decay-abs** — `after(T)` must be FUTURE (BIP-65 500_000_000 height/time
  split, strict `<` against static `ABS_HEIGHT_PAST_FLOOR=900_000` /
  `ABS_TIME_PAST_FLOOR=1_750_000_000` floors).

Because D-decay-abs fires on the canon's past `after(500000)`, the canon golden
set was migrated to a future `after(4000000)`, cascading through a regenerated
BIP-380 checksum `#llvl05j9` → `#9fqrjy7e`.

---

## Critical

**NONE.**

## Important

**NONE.**

## Minor

**M1 (cosmetic, not actionable for ship).** The implementer's "3350 passed/0
failed" is correct; an initial aggregation of mine over `test result:` lines
mis-summed one column-misaligned variant and reported a spurious `failed=1`. A
robust re-parse confirmed **3350 / 0 / 15-ignored**. No real failure exists —
recording only so a future reader who runs a naïve `awk '{f+=$6}'` is not alarmed.

**M2 (forward-decay note, informational).** `ABS_TIME_PAST_FLOOR` (~2025-06-15)
and `ABS_HEIGHT_PAST_FLOOR=900_000` are static. The strict-`<` monotone-safe
design means the only failure mode over time is a *false-negative* on a
borderline-recent locktime (never a false-positive on a legit future value), so
this is correctly fail-closed-leaning and matches BRAINSTORM §4.2. No action;
the design comment already documents this. Not a migration defect.

---

## Evidence — by hunt item

### 1. Descriptor checksum validity (`#9fqrjy7e`) — VERIFIED CORRECT (triple-confirmed)

The `.descriptor` fixture now ends `…after(4000000)))))#9fqrjy7e`. Verified by
three independent methods:

1. **Independent Python BIP-380 reference implementation** (the canonical
   `descsum_create` from the BIP, INPUT_CHARSET + polymod, written fresh, not
   the in-tree engine) computed `9fqrjy7e` from the migrated descriptor string.
2. **`miniscript` v13 (the in-tree dependency)** `Descriptor::<DescriptorPublicKey>::from_str`
   PARSED the full fixture WITH `#9fqrjy7e` → `PARSE_OK`, and round-trips
   byte-identically (the checksum is validated on parse).
3. **Negative control:** corrupting to `#9fqrjy7f` → miniscript REJECTS with
   `invalid checksum 9fqrjy7f; expected 9fqrjy7e` — proving the parse is genuinely
   checksum-validating, not vacuously accepting.

Conclusion: `#9fqrjy7e` is the correct BIP-380 checksum for the new descriptor,
and the descriptor parses. (Throwaway miniscript example + Python script were
removed; worktree restored clean.)

### 2. Migration consistency — NO orphaned `500000` — VERIFIED COMPLETE

Grepped the whole post-diff tree (`crates/`, `docs/`, excluding `target/` and
the unrelated `1500000000` future-Unix-time site). Every surviving `500000` /
`500001` is an EXPECTED leave-as-is:

- `mod.rs` string golden, all 3 fixtures (`.json`/`.descriptor`/`.bip388`), the
  canon CLI `preset_args` (`:82`), the `:570-571` cleanliness site, the in-crate
  `fixture_params` (`archetype.rs:595`) + doc (`:33`), and the
  `cross_branch_duplicates` repeated_keys site (`:1244`) are ALL uniformly
  `4000000`. ✓
- The `:683`-era mutation test migrated to **`--after 4000001`** (`cli_build_descriptor.rs:907`). ✓
- The `:1019-1020`-era repeated_keys test migrated to **`4000000`** (`cli_build_descriptor.rs:1244`, fn `cross_branch_duplicates_carry_no_flag`). ✓

The only surviving `500000`/`500001` occurrences (3 UNAFFECTED + 1 intentional
negative):
- `ir.rs:390` — pure `PolicyNode::After(500000).render()` unit test (no
  `validate_params` path). UNAFFECTED per plan §7.
- `cli_build_descriptor.rs:691,721` — the NEW negative test
  `preset_decay_past_after_height_refused_exit_2`, which DELIBERATELY uses
  `--after 500000` to drive the abs-reject. Intentional.
- `specter-descriptor-with-checksum.json:3` — `"blockheight": 500000`
  wallet-import metadata (wpkh single-sig). UNAFFECTED per plan §7.

No orphan. Migration is uniform and complete.

### 3. Migrated tests still exercise their INTENDED concern (NON-VACUOUS)

(a) **`cross_branch_duplicates_carry_no_flag`** (the repeated_keys test, `:1244`):
uses `--final-key K3` == `--recovery-key K3` (dup) + `--after 4000000` (future).
Since `4000000` is future-height (`> 900_000` floor, `< 500_000_000`), `validate_params`
adds NO abs diag and does NOT short-circuit → the dup flows to the gate →
asserts `kind=="repeated_keys"` AND `node_path=="root.andor[2]"`. Reaches the
intended diagnostic; not passing for the wrong reason. ✓

(b) **`preset_negative_discrimination_mutated_param_breaks_golden`** (`:907`):
mutates `decaying-multisig --after` to `4000001`. `4000001` is future-height
(`> 900_000`, `< 500_000_000`) → builds (`.success()` genuine), and
`after(4000001) != after(4000000)` → golden differs (`assert_ne!` genuine). Still
tests the producer-reads-the-param concern. ✓

(c) **`validate_params_decay_ordering`** (in-crate, `archetype.rs:796`):
`fixture_params` now has `older:1000`(Blocks), `after:4000000`(future-Blocks),
and sets `recovery_older` to bad `[1000, 999]`(Blocks). Same-unit `v_r <= v_p`
fires exactly 1 diag; the future `after(4000000)` adds NO abs diag (verified
arithmetically: `0x3D0900`, bit-22 clear ⇒ Blocks, value 2304, `> 900_000` floor).
`assert_eq!(diags.len(), 1)` HOLDS. ✓

### 4. New negative/positive tests non-vacuous — MUTATION-TESTED RED

- **D-decay-rel cross-unit** (`preset_decay_cross_unit_refused_exit_2`,
  `--older 145`-Blocks vs `--recovery-older 4194305`-Sec512): mutating
  `if u_p != u_r` → `if false` made the test go **RED**
  (`failed var.contains(different BIP-68 timelock units)`). Predicate restored. ✓
- **D-decay-abs past** (`preset_decay_past_after_height_refused_exit_2`,
  `--after 500000`): forcing `past = false` made the test go **RED** (past height
  now accepted, expected exit-2 + "already in the past" gone). Predicate restored. ✓
- **Positive controls** (`preset_decay_same_unit_ordered_builds`,
  `preset_decay_same_unit_both_512s_builds`, `preset_decay_future_after_height_builds`)
  all build (exit 0) as asserted. The `older_unit_value` unit test
  classifications (`145→(Blocks,145)`, `4194305→(Sec512,1)`,
  `4000000→(Blocks,2304)`, `0x400002→(Sec512,2)`) independently re-derived correct. ✓

After all mutation tests, `git diff HEAD` is EMPTY (worktree byte-restored).

### 5. Build / suite reality

- `cargo test -p mnemonic-toolkit` → **3350 passed / 0 failed / 15 ignored**,
  exit 0 (robust per-line re-parse; the bin-target in-crate decay tests —
  `validate_params_decay_ordering`, `fixtures_test::decaying_multisig`,
  `older_unit_value_classifies_clean_operands` — all `ok` in the `--bin mnemonic`
  target, 1050 passed). No flake observed across runs.
- `cargo clippy --all-targets -- -D warnings` → clean, exit 0.

Implementer's "3350 passed/0 failed, clippy clean" claim CONFIRMED.

---

## Verdict

**LENS-B MIGRATION: 0C / 0I — GREEN**
