# FIX-sec-archetype-older-blocks-bound: preset `--older` / `--recovery-older` take blocks only

Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-23. Base: origin/master `68ea05e0` (toolkit 0.107.1).
Security-sweep unit `archetype-older-blocks-bound` (coordinator `reports/security-sweep/grade/routing.md`), grade I,
row `archetype-older-blocks-flag-accepts-time-units` (duplicates B053-K2, B056 R16, B058 row 1). Track: small-fix (no
new flag, type, command or public API; four spec sentences). RED script: `design/fix-sec-archetype-older-blocks-bound-red.sh`.

## 1. Defect

`build-descriptor --archetype <preset>` documents `--older` and `--recovery-older` as relative timelocks in blocks
(clap help "Relative timelock (blocks)", `cmd/build_descriptor.rs:80-89`; `ParamKind::Blocks` at
`descriptor_builder/archetype.rs:127,130,162,179,196,214`). `validate_params` puts no bound on them, and the gate
accepts every clean BIP-68 operand, including the time-based window `0x400001..=0x40FFFF`. So
`--archetype hashlock-gated ... --older 4200000` exits 0 and emits `older(4200000)`: BIP-68 sets bit 22
(`0x400000`), so consensus reads it as 5696 units of 512 seconds (2916352 seconds, about 33.75 days), not as a block
count, and no line says so. The recovery path opens on a schedule the holder did not type. RED rows R1-R5.

BIP-68 (bitcoin/bips `bip-0068.mediawiki`, fetched 2026-11-23): line 36 "If the bit is set, the relative lock-time
specifies a timespan in units of 512 seconds granularity ... If the bit is not set, the relative lock-time specifies
a number of blocks"; line 40 "a mask of 0x0000ffff MUST be applied to the sequence field"; line 79
`SEQUENCE_LOCKTIME_TYPE_FLAG = (1 << 22)`; line 92 `SEQUENCE_LOCKTIME_GRANULARITY = 9`.

## 2. Decision and the records it departs from

The routing cell decides: preset `--older` / `--recovery-older` (`ParamKind::Blocks`) refuse values outside
`1..=65535`, naming the 512-second-unit reading. It is a refusal, not a warning, because the wrong lock is a funds
risk (Brian's warn-and-proceed ruling covers only no-funds-risk cases). Old files: none involved (presets build
from flags; `--spec` and `--descriptor` are unchanged).

This departs from two earlier records, which the FIX does not edit (history), and which the spec delta (§8)
supersedes:

- `design/BRAINSTORM_cycle6_timelock_decay.md` §6 R5 chose "do NOT bound `ParamKind::Blocks` to `1..=65535`
  globally", keeping a both-512s decaying-multisig ladder on the preset path. After this FIX that ladder is built
  with `--spec` (row R5). Justification: the sweep grade (I, wrong result at exit 0) lifts R5's "near-nil
  exposure" premise, and the routing cell rules the bound in.
- `design/SPEC_descriptor_builder_presets.md` §3.2 / §11 risk 1 ("no second validation path"). The bound is a
  preset meaning ("this flag is blocks"), not a gate rule: the gate keeps accepting the 512-second window for
  `--spec`. §8 adds it as a fourth producer category.

## 3. Holder-visible behaviour (decided)

Scope: `build-descriptor --archetype <any of the 5>`. A supplied `--older` or `--recovery-older` that the chosen
archetype declares, whose value N is not in `1..=65535`, is refused by `validate_params` with one `param`
diagnostic per offending flag. Exit 2. Stdout empty in human mode; nothing is built, no descriptor, cost table or
address is printed.

Exact message (the `message` field; `{id}` is the archetype id, `{flag}` is `--older` or `--recovery-older`):

- Form A, N in the 512-second window (`N & 0xFFFF0000 == 0x00400000` and `N & 0xFFFF != 0`), with
  `V = N & 0xFFFF` and `S = V * 512`, all printed as plain decimal integers:

      {id} {flag} is a relative timelock in blocks and takes 1..=65535; got {N}, which BIP-68 reads as a time-based lock of {V} x 512 seconds ({S} seconds), not {N} blocks; for a time-based lock, author the policy with --spec

- Form B, any other N outside `1..=65535` (0, 65536..=4194304, 4259840..=4294967295):

      {id} {flag} is a relative timelock in blocks and takes 1..=65535; got {N}

Human mode (unchanged frame, `cmd/build_descriptor.rs:291-310`) prints on stderr:

    build-descriptor: refused — <k> diagnostic(s):
      [param] params: <message> (from <flag>)

`--json` prints on stdout `{"diagnostics": [...]}` with, per refusal, `kind` `"param"`, `node_path` `"params"`,
`flag` the flag, `message` as above.

Edge cases (decided):

1. Bounds: 1 and 65535 build (C1, C2). 0, 65536, 4194304 (`0x400000`, zero value) and 105120 get Form B (R8-R11);
   4194305 and 4259839 (`0x40FFFF`) get Form A (R2, R3).
2. Values the gate refuses today (zero value or bits outside `0x0040FFFF`) are now refused at the preset with
   Form B before the gate runs: in preset mode the gate's `older()` mask diagnostic no longer appears (R8-R13).
   `--spec` keeps the gate text unchanged.
3. Both flags out of range: two diagnostics, `--older` first, then `--recovery-older` (R5).
4. Order among producer diagnostics: applicability, then presence/arity (both unchanged), then the bound, then
   decay ordering.
5. Decay ordering (decaying-multisig) runs only when both `--older` and `--recovery-older` are present and inside
   `1..=65535`. With both in range they share one unit, so the cross-unit refusal ("use different BIP-68 timelock
   units") can no longer fire and is removed; the ordering test becomes `--recovery-older` > `--older` on the
   values, with the unchanged "progressively later" text (C3, C4). `--older 145 --recovery-older 4194305` now
   gets one Form A diagnostic on `--recovery-older` (R6); `--older 1000 --recovery-older 70000` one Form B
   diagnostic on `--recovery-older` (R13).
6. A flag the archetype does not declare gets only the applicability diagnostic, never the bound (C5).
7. A missing required flag gets only the existing "requires ... (missing)" diagnostic.
8. Unchanged: clap help texts, `--spec` and the gate, `--spec-schema` (kind stays `"blocks"`), every other
   command that reads `older()` (intake advisory, compare-cost), exit codes of every other path.

## 4. Code delta (production)

- `descriptor_builder/archetype.rs` `validate_params`: after the presence/arity loop, for each `ParamSpec` of
  kind `ParamKind::Blocks` the archetype declares and the holder supplied, push `param_diag(flag, <Form A|B>)`
  when the value is outside `1..=65535`, in `def.params` order (which puts `--older` before `--recovery-older`
  in decaying-multisig). Guard the decaying-multisig relative-ordering block on both values in `1..=65535`;
  replace the unit-aware compare with `recovery_older <= older` and delete the cross-unit branch and its
  message. The D-decay-abs `--after` block is unchanged.
- Doc comments: the module doc (lines 1-7), the `ParamKind` doc ("no producer check keys off it") and the
  `validate_params` doc say the producer now bounds `Blocks` params.
- `timelock_advisory::older_unit_value` loses its only caller. The bin target compiles `timelock_advisory` as a private module (`src/main.rs:38`), so leaving it would raise `dead_code` and fail `clippy -D warnings`. Delete `older_unit_value` and its unit test `older_unit_value_classifies_clean_operands`, and drop the sentence naming it in the `older_consensus_masked`/module docs if any. No caller exists in this workspace, mnemonic-gui or refugium-wallet. (Public lib API removal is accepted on the small-fix track: the crate is 0.x and the fn was added for this check only.)

## 5. RED rows

Script: `design/fix-sec-archetype-older-blocks-bound-red.sh <mnemonic-binary>` (this branch). Keys are the public
BIP-32 test-vector xpubs of `tests/cli_build_descriptor.rs` K1..K5; no seed is involved. Build:
`cargo build --locked -p mnemonic-toolkit --bin mnemonic`, then run the script on `target/debug/mnemonic`
(`TMPDIR` under `/scratch/cache` locally).

Run on `68ea05e0`, seen to show the defect (R1-R5, R7 accept; R6, R8-R13 the texts that change) and the controls:

    R1-hg-4200000 rc=0 older=[older(4200000)]
    R2-sti-4194305 rc=0 older=[older(4194305)]
    R3-kofn-4259839 rc=0 older=[older(4259839)]
    R4-tr-4200000 rc=0 older=[older(4200000)]
    R5-dm-both-512s rc=0 older=[older(4194305) older(4194306)]
    R6-dm-cross-unit rc=2 older=[]
        R6-dm-cross-unit   [param] params: decaying-multisig --older (145) and --recovery-older (4194305) use different BIP-68 timelock units (one block-height, one 512-second) and cannot be ordered; express both in the same unit, or author the policy with --spec (from --recovery-older)
    R7-hg-4200000-json rc=0 n=0
    R8-kofn-0 rc=2 older=[]
        R8-kofn-0   [schema_field] root.or_d[1].and_v[1]: older(N) encodes a BIP-68 relative timelock: only the low 16 bits are the value, and bit 22 (0x400000) selects 512-second units. All other bits — including the bit-31 disable flag — must be clear, and the 16-bit value must be non-zero. got 0 (0x00000000); consensus would silently mask this to an effective value of 0 blocks, weakening or nullifying the timelock. Use 1..=65535 (blocks) or 0x400000|(1..=65535) (512-second units). (from --older)
    R9-kofn-65536 rc=2 older=[]
        R9-kofn-65536   [schema_field] root.or_d[1].and_v[1]: older(N) encodes a BIP-68 relative timelock: only the low 16 bits are the value, and bit 22 (0x400000) selects 512-second units. All other bits — including the bit-31 disable flag — must be clear, and the 16-bit value must be non-zero. got 65536 (0x00010000); consensus would silently mask this to an effective value of 0 blocks, weakening or nullifying the timelock. Use 1..=65535 (blocks) or 0x400000|(1..=65535) (512-second units). (from --older)
    R10-kofn-4194304 rc=2 older=[]
        R10-kofn-4194304   [schema_field] root.or_d[1].and_v[1]: older(N) encodes a BIP-68 relative timelock: only the low 16 bits are the value, and bit 22 (0x400000) selects 512-second units. All other bits — including the bit-31 disable flag — must be clear, and the 16-bit value must be non-zero. got 4194304 (0x00400000); consensus would silently mask this to an effective value of 0 (512-second units), weakening or nullifying the timelock. Use 1..=65535 (blocks) or 0x400000|(1..=65535) (512-second units). (from --older)
    R11-kofn-105120 rc=2 older=[]
        R11-kofn-105120   [schema_field] root.or_d[1].and_v[1]: older(N) encodes a BIP-68 relative timelock: only the low 16 bits are the value, and bit 22 (0x400000) selects 512-second units. All other bits — including the bit-31 disable flag — must be clear, and the 16-bit value must be non-zero. got 105120 (0x00019aa0); consensus would silently mask this to an effective value of 39584 blocks, weakening or nullifying the timelock. Use 1..=65535 (blocks) or 0x400000|(1..=65535) (512-second units). (from --older)
    R12-kofn-105120-json rc=2 n=1 kind=schema_field node_path=root.or_d[1].and_v[1] flag=--older
    R13-dm-recovery-70000 rc=2 older=[]
        R13-dm-recovery-70000   [schema_field] root.andor[2].andor[1]: older(N) encodes a BIP-68 relative timelock: only the low 16 bits are the value, and bit 22 (0x400000) selects 512-second units. All other bits — including the bit-31 disable flag — must be clear, and the 16-bit value must be non-zero. got 70000 (0x00011170); consensus would silently mask this to an effective value of 4464 blocks, weakening or nullifying the timelock. Use 1..=65535 (blocks) or 0x400000|(1..=65535) (512-second units). (from --recovery-older)
    C1-sti-65535 rc=0 older=[older(65535)]
    C2-hg-1 rc=0 older=[older(1)]
    C3-dm-1000-2000 rc=0 older=[older(1000) older(2000)]
    C4-dm-2000-1000 rc=2 older=[]
        C4-dm-2000-1000   [param] params: decaying-multisig requires --recovery-older (1000) > --older (2000): tiers must unlock progressively later (from --recovery-older)
    C5-kofn-inapplicable-recovery-older rc=2 older=[]
        C5-kofn-inapplicable-recovery-older   [param] params: --recovery-older is not a parameter of kofn-recovery (from --recovery-older)

Expected after the fix (C1-C5 byte-identical to the block above):

    R1-hg-4200000 rc=2 older=[]
        R1-hg-4200000   [param] params: hashlock-gated --older is a relative timelock in blocks and takes 1..=65535; got 4200000, which BIP-68 reads as a time-based lock of 5696 x 512 seconds (2916352 seconds), not 4200000 blocks; for a time-based lock, author the policy with --spec (from --older)
    R2-sti-4194305 rc=2 older=[]
        R2-sti-4194305   [param] params: simple-timelocked-inheritance --older is a relative timelock in blocks and takes 1..=65535; got 4194305, which BIP-68 reads as a time-based lock of 1 x 512 seconds (512 seconds), not 4194305 blocks; for a time-based lock, author the policy with --spec (from --older)
    R3-kofn-4259839 rc=2 older=[]
        R3-kofn-4259839   [param] params: kofn-recovery --older is a relative timelock in blocks and takes 1..=65535; got 4259839, which BIP-68 reads as a time-based lock of 65535 x 512 seconds (33553920 seconds), not 4259839 blocks; for a time-based lock, author the policy with --spec (from --older)
    R4-tr-4200000 rc=2 older=[]
        R4-tr-4200000   [param] params: tiered-recovery --older is a relative timelock in blocks and takes 1..=65535; got 4200000, which BIP-68 reads as a time-based lock of 5696 x 512 seconds (2916352 seconds), not 4200000 blocks; for a time-based lock, author the policy with --spec (from --older)
    R5-dm-both-512s rc=2 older=[]
        R5-dm-both-512s   [param] params: decaying-multisig --older is a relative timelock in blocks and takes 1..=65535; got 4194305, which BIP-68 reads as a time-based lock of 1 x 512 seconds (512 seconds), not 4194305 blocks; for a time-based lock, author the policy with --spec (from --older)
        R5-dm-both-512s   [param] params: decaying-multisig --recovery-older is a relative timelock in blocks and takes 1..=65535; got 4194306, which BIP-68 reads as a time-based lock of 2 x 512 seconds (1024 seconds), not 4194306 blocks; for a time-based lock, author the policy with --spec (from --recovery-older)
    R6-dm-cross-unit rc=2 older=[]
        R6-dm-cross-unit   [param] params: decaying-multisig --recovery-older is a relative timelock in blocks and takes 1..=65535; got 4194305, which BIP-68 reads as a time-based lock of 1 x 512 seconds (512 seconds), not 4194305 blocks; for a time-based lock, author the policy with --spec (from --recovery-older)
    R7-hg-4200000-json rc=2 n=1 kind=param node_path=params flag=--older
    R8-kofn-0 rc=2 older=[]
        R8-kofn-0   [param] params: kofn-recovery --older is a relative timelock in blocks and takes 1..=65535; got 0 (from --older)
    R9-kofn-65536 rc=2 older=[]
        R9-kofn-65536   [param] params: kofn-recovery --older is a relative timelock in blocks and takes 1..=65535; got 65536 (from --older)
    R10-kofn-4194304 rc=2 older=[]
        R10-kofn-4194304   [param] params: kofn-recovery --older is a relative timelock in blocks and takes 1..=65535; got 4194304 (from --older)
    R11-kofn-105120 rc=2 older=[]
        R11-kofn-105120   [param] params: kofn-recovery --older is a relative timelock in blocks and takes 1..=65535; got 105120 (from --older)
    R12-kofn-105120-json rc=2 n=1 kind=param node_path=params flag=--older
    R13-dm-recovery-70000 rc=2 older=[]
        R13-dm-recovery-70000   [param] params: decaying-multisig --recovery-older is a relative timelock in blocks and takes 1..=65535; got 70000 (from --recovery-older)

## 6. Tests that change

All in `crates/mnemonic-toolkit`. New test functions carry the names below exactly.

Changed (`tests/cli_build_descriptor.rs`):

- `preset_decay_cross_unit_refused_exit_2` (:586): same argv; assert exit 2 and stderr containing the R6 Form A
  message exactly and `(from --recovery-older)`; assert stderr does NOT contain "different BIP-68 timelock units"
  and contains `refused — 1 diagnostic(s):`. Doc comment updated (the preset bound now refuses it).
- `preset_decay_same_unit_both_512s_builds` (:660): renamed `preset_decay_both_512s_refused_exit_2`; same argv;
  assert exit 2, `refused — 2 diagnostic(s):`, and both R5 messages in that order.
- `preset_masked_older_refused_with_older_provenance` (:1159): same argv; `diagnostics[0]` now has `kind`
  `"param"`, `node_path` `"params"`, `flag` `"--older"`, `message` equal to the R11 Form B text; diagnostics length 1.
  Doc comment updated (refused by the preset bound before the gate).

New (`tests/cli_build_descriptor.rs`):

- `preset_older_512s_window_refused_exit_2`: table over R1-R4 argv; each exits 2, stdout empty, stderr contains the
  row's exact expected line (`[param] params: ... (from --older)`) and no `older(`.
- `preset_older_512s_window_json_param_diag`: R7 argv with `--json`; exit 2, one diagnostic, `kind` `"param"`,
  `node_path` `"params"`, `flag` `"--older"`, `message` the R1 Form A text.
- `preset_older_out_of_range_form_b_exit_2`: table over R8, R9, R10, R13 argv; exit 2, the row's exact Form B line,
  stderr without `schema_field`.
- `preset_older_bounds_build`: C1 and C2 argv build (exit 0); stdout contains `older(65535)` and `older(1)`.

Changed (`src/descriptor_builder/archetype.rs` unit tests):

- New `validate_params_older_blocks_bound`: on `fixture_params` of each archetype, `--older` in {1, 65535} passes on the four archetypes other than decaying-multisig; on decaying-multisig the pass cases are the pairs (`older`, `recovery_older`) = (1, 2), (1, 65535) and (65534, 65535); in {0, 65536, 0x400000, 0x410000, u32::MAX} gives exactly one Form B diagnostic on `--older`;
  in {0x400001, 0x40FFFF} exactly one Form A diagnostic; on decaying-multisig, with `older` = 1000 (fixture), `recovery_older` in the same refused sets gives exactly one diagnostic of the same form on `--recovery-older` and no "progressively later" diagnostic; both out of range gives two, `--older` first; a decaying-multisig pair with one value out of
  range gives no "progressively later" diagnostic.
- `validate_params_does_not_duplicate_gate_rules` (:820) and `validate_params_decay_ordering` (:804): unchanged,
  must stay GREEN.

Unchanged and must stay GREEN: every golden and preset argv (all use 144, 1000, 2000, 4032, 52560, 65535, 65534,
4033, 145), `gate.rs` tests (the gate still accepts `0x0040_0001`, `0x0040_FFFF` at :1039), `cli_older_advisory.rs`,
`cli_compare_cost.rs`.

## 7. Manual delta (lockstep)

`docs/manual/src/40-cli-reference/41-mnemonic.md`:

- :4876-4882, the refusal bullet, becomes: "A parameter that does not belong to the chosen archetype, a missing
  required parameter, too few keys, an `--older` or `--recovery-older` outside `1..=65535`, or `--recovery-older`
  ≤ `--older` is refused with a `param`-kind diagnostic naming the flag (exit 2). The two timelock flags take
  blocks only: a value BIP-68 would read as 512-second units (`0x400001..=0x40FFFF`) is refused with that reading
  spelled out; author a time-based lock with `--spec`. Everything else — k > n, duplicate keys, bad hex, an
  `xprv` — flows to the SAME validation gate as `--spec` and is refused there; in preset mode those node-addressed
  diagnostics also name the responsible flag (`(from --key)` in the human output, `flag` in `--json`)."
- :4915 flag row: "relative timelock in blocks, `1..=65535`, gating the recovery path (`decaying-multisig`: the
  tier-1 timelock)".
- :4916 flag row: "`decaying-multisig` tier-2 relative timelock in blocks, `1..=65535`; must be **greater than**
  `--older` (tiers must unlock progressively later)".

`docs/manual-gui/src/40-mnemonic/4e-build-descriptor.md`:

- :152-157 (`--older` section) becomes: "Relative timelock in blocks gating the recovery path (`decaying-multisig`:
  the tier-1 timelock). Takes `1..=65535`; any other value is refused, and a value BIP-68 would read as 512-second
  units (`0x400001..=0x40FFFF`) is refused with that reading spelled out. For a time-based lock, author the policy
  as a tree. The GUI renders this as a Number widget."
- :161-163 (`--recovery-older`): insert "in blocks, `1..=65535`," after "relative timelock".
- Refusals table: replace the row ":322 `--older` with any disallowed BIP-68 bit set or a zero value | gate
  refusal ..." with "`--older` or `--recovery-older` outside `1..=65535` | `param`-kind diagnostic naming the flag;
  a 512-second-unit value has its reading spelled out (exit 2)".

No anchor changes; no GUI schema change (no flag added, removed or renamed; help texts unchanged).

## 8. Spec delta

`design/SPEC_descriptor_builder_presets.md`:

- §3.1: add item 4: "**Timelock unit (all archetypes):** `--older` and `--recovery-older` (`ParamKind::Blocks`)
  take blocks, `1..=65535`. Any other value is refused at the producer, naming the flag; a value in the BIP-68
  512-second window `0x400001..=0x40FFFF` has that reading spelled out. The gate accepts that window for `--spec`;
  the bound is the presets' meaning of the flag, not a gate rule, so it does not break §3.2."
- §3.1 item 3: append "It runs only when both values pass item 4."
- §3.2: "timelock bounds (`older`/`after` field rules in `validate_fields`)" becomes "timelock bounds (`after`
  field rules, and the `older` field rule, which item 4 pre-empts in preset mode)".
- §7 negative cells (after the "Decay ordering" cell): add "- Timelock unit: `--older 4200000` → producer `Param` diagnostic naming `--older` and
  the 512-second reading; `--older 0` / `65536` → producer `Param` diagnostic, no gate diagnostic."
- §11 risk 1: "producer checks are exactly §3.1's three categories" becomes "four categories".
- The `ParamKind` comment in the §2 code block ("no producer check keys off it") becomes "the §3.1 item 4 bound
  keys off `Blocks`".

`design/SPEC_older_timelock_mask_gate.md`: append under "Ritual / FOLLOWUPS" one line: "Erratum 2026-11-23
(FIX_sec_archetype_older_blocks_bound): the preset bound now refuses out-of-range `--older` before the gate, so
the Tests (2) preset cell sees a `param` diagnostic; the deferred M1 item is done."

## 9. CHANGELOG (`CHANGELOG.md`, under `## [Unreleased]`)

    ### `build-descriptor` presets take `--older` / `--recovery-older` in blocks only

    - `--archetype` presets refuse an `--older` or `--recovery-older` outside `1..=65535` with a `param`
      diagnostic (exit 2). A value BIP-68 reads as 512-second units (`0x400001..=0x40FFFF`, e.g. `4200000`,
      about 33.75 days) used to build silently; the refusal now spells out that reading. Author a time-based lock
      with `--spec`. The decaying-multisig cross-unit refusal is gone (both tiers are now blocks).

## 10. Wording proposals (WORDING_REVIEW rows; the toolkit has no list, the coordinator files them)

- AOB-W1 (new): Form A, §3.
- AOB-W2 (new): Form B, §3.
- AOB-W3 (removed): "decaying-multisig --older ({older}) and --recovery-older ({recovery_older}) use different BIP-68
  timelock units (one block-height, one 512-second) and cannot be ordered; express both in the same unit, or author
  the policy with --spec".
- AOB-W4 (new manual prose): the §7 bullets.

## 11. Coverage map (rule -> row)

| Rule | Rows |
|---|---|
| §3 Form A, each archetype, exit 2, nothing built | R1-R4; `preset_older_512s_window_refused_exit_2`; unit test |
| §3 `--json` kind/node_path/flag | R7, R12; `preset_older_512s_window_json_param_diag`, `preset_masked_older_refused_with_older_provenance` |
| edge 1 bounds | C1, C2, R2, R3, R8-R10; `preset_older_bounds_build`; unit test (0x410000, u32::MAX) |
| edge 2 preset pre-empts the gate | R8-R13; `preset_older_out_of_range_form_b_exit_2` |
| edge 3 two flags, order | R5; `preset_decay_both_512s_refused_exit_2`; unit test |
| edge 5 decay skipped, cross-unit gone | R6, R13, C3, C4; `preset_decay_cross_unit_refused_exit_2`; unit test |
| edge 6 inapplicable flag | C5 |
| edge 8 `--spec` unchanged | gate tests at `gate.rs:1039`, `cli_build_descriptor.rs:1423` (unchanged, GREEN) |

## 12. Gate (build agent; full CI locally, `$HOME/.cargo/bin` first, cargo 1.85.0)

    cargo fmt --all --check
    cargo clippy --locked --all-targets -- -D warnings
    cargo nextest run --locked --workspace --no-fail-fast
    cargo nextest run --locked -p mnemonic-toolkit -E 'test(/preset_older_|preset_decay_cross_unit_refused_exit_2|preset_decay_both_512s_refused_exit_2|preset_masked_older_refused_with_older_provenance|validate_params_older_blocks_bound/)'
    cargo test --locked --workspace --doc
    cargo build --locked -p mnemonic-toolkit --bin mnemonic && design/fix-sec-archetype-older-blocks-bound-red.sh target/debug/mnemonic
    make -C docs/manual lint ...        (the CI manual job, as CLAUDE.md)
    the manual-gui job (docs/manual-gui lint, as .github/workflows/manual-gui.yml)

The filtered nextest run must report 8 tests (4 new CLI, 3 changed CLI, 1 new unit); run on 68ea05e0 it starts
"2 tests" (the two kept names), both PASS. Not covered: the SPEC and
CHANGELOG prose (read the diff).

## 13. FOLLOWUPS

Closes `archetype-older-blocks-flag-accepts-time-units` (FOLLOWUPS.md:488) once the build merges; the
closing line is appended after merge (no FOLLOWUPS lines inside the PR). `older_unit_value` is deleted (see §4); no
FOLLOWUPS line for it.
